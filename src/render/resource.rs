use bevy::{
    ecs::{component::Component, system::Resource},
    math::{UVec3, Vec2, Vec4},
    render::{
        render_resource::{
            AddressMode, BindingResource, DynamicUniformBuffer, Extent3d, FilterMode,
            GpuArrayBuffer, SamplerDescriptor, ShaderType, TextureAspect, TextureDescriptor,
            TextureDimension, TextureUsages, TextureView, TextureViewDescriptor,
            TextureViewDimension,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::GpuImage,
    },
};

use super::{prepare::DynamicUniformIndex, SHADOW_MAP_FORMAT, SHADOW_PREPASS_WORKGROUP_SIZE};

#[derive(ShaderType)]
pub struct GpuAmbientLight2d {
    pub color: Vec4,
    pub intensity: f32,
}

#[derive(Resource, Default)]
pub struct GpuAmbientLight2dBuffer {
    buffer: DynamicUniformBuffer<GpuAmbientLight2d>,
}

impl GpuAmbientLight2dBuffer {
    pub fn new(
        light: GpuAmbientLight2d,
        render_device: &RenderDevice,
        render_queue: &RenderQueue,
    ) -> Self {
        let mut buffer = Self::default();
        buffer.buffer.clear();
        buffer.buffer.push(&light);
        buffer.buffer.write_buffer(render_device, render_queue);
        buffer
    }

    #[inline]
    pub fn binding(&self) -> BindingResource {
        self.buffer.binding().unwrap()
    }
}

#[derive(ShaderType, Clone)]
pub struct GpuPointLight2d {
    pub intensity: f32,
    pub position_ss: Vec2,
    pub radius_ss: f32,
    pub range_ss: f32,
    pub color: Vec4,
}

#[derive(Component)]
pub struct GpuLights2d {
    point_lights: GpuArrayBuffer<GpuPointLight2d>,
}

impl GpuLights2d {
    #[inline]
    pub fn new(render_device: &RenderDevice) -> Self {
        Self {
            point_lights: GpuArrayBuffer::new(render_device),
        }
    }

    #[inline]
    pub fn add_point_light(&mut self, light: GpuPointLight2d) {
        self.point_lights.push(light);
    }

    #[inline]
    pub fn point_lights_binding(&self) -> BindingResource {
        self.point_lights.binding().unwrap()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.point_lights.clear();
    }

    #[inline]
    pub fn write_buffers(&mut self, render_device: &RenderDevice, render_queue: &RenderQueue) {
        self.point_lights.write_buffer(render_device, render_queue);
    }
}

#[derive(ShaderType)]
pub struct GpuShadowMapMeta {
    pub index: u32,
    pub size: u32,
    pub offset: Vec2,
    pub bias: f32,
    pub pcf_samples: u32,
    pub pcf_radius: f32,
}

#[derive(Resource, Default)]
pub struct GpuMetaBuffers {
    shadow_map: DynamicUniformBuffer<GpuShadowMapMeta>,
    reduction: DynamicUniformBuffer<u32>,
    reduction_offsets: Vec<u32>,
}

impl GpuMetaBuffers {
    #[inline]
    pub fn push_light_meta(
        &mut self,
        meta: GpuShadowMapMeta,
    ) -> DynamicUniformIndex<GpuShadowMapMeta> {
        DynamicUniformIndex::new(self.shadow_map.push(&meta))
    }

    #[inline]
    pub fn init_reduction_time_buffer(&mut self, num_reductions: u32) {
        self.reduction.clear();
        self.reduction_offsets.clear();

        for i in 0..num_reductions {
            let idx = self.reduction.push(&(i + 1));
            self.reduction_offsets.push(idx);
        }
    }

    #[inline]
    pub fn get_reduction_index(&self, reduction: u32) -> u32 {
        self.reduction_offsets[reduction as usize]
    }

    #[inline]
    pub fn clear(&mut self) {
        self.shadow_map.clear();
    }

    #[inline]
    pub fn write_buffers(&mut self, render_device: &RenderDevice, render_queue: &RenderQueue) {
        self.shadow_map.write_buffer(render_device, render_queue);
        self.reduction.write_buffer(render_device, render_queue);
    }

    #[inline]
    pub fn shadow_map_meta_buffer_binding(&self) -> BindingResource {
        self.shadow_map.binding().unwrap()
    }

    // This buffer keeps panic if unwrap directly, not sure why
    #[inline]
    pub fn reduction_time_buffer_binding(&self) -> Option<BindingResource> {
        self.reduction.binding()
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct ShadowMap2dMeta {
    pub count: u32,
    pub size: u32,
}

#[derive(Resource, Default)]
pub struct ShadowMap2dStorage {
    meta: ShadowMap2dMeta,
    primary_shadow_map: Option<GpuImage>,
    secondary_shadow_map: Option<GpuImage>,
    work_group_count_per_light: UVec3,
    work_group_count_total: UVec3,
    num_reductions: u32,
}

impl ShadowMap2dStorage {
    pub fn try_update(
        &mut self,
        meta: ShadowMap2dMeta,
        render_device: &RenderDevice,
        meta_buffers: &mut GpuMetaBuffers,
    ) {
        if self.meta == meta {
            return;
        }

        self.meta = meta;
        self.primary_shadow_map = Some(self.create_shadow_map(render_device));
        self.secondary_shadow_map = Some(self.create_shadow_map(render_device));
        self.work_group_count_per_light = UVec3 {
            x: meta.size.div_ceil(SHADOW_PREPASS_WORKGROUP_SIZE.x),
            y: meta.size.div_ceil(SHADOW_PREPASS_WORKGROUP_SIZE.y),
            z: 1,
        };
        self.work_group_count_total = UVec3 {
            x: self.work_group_count_per_light.x,
            y: self.work_group_count_per_light.y,
            z: meta.count,
        };
        self.num_reductions = meta.size.trailing_zeros();

        assert_eq!(
            2u32.pow(self.num_reductions),
            self.meta.size,
            "Shadow map size must be a power of 2!"
        );

        meta_buffers.init_reduction_time_buffer(self.num_reductions);
    }

    #[inline]
    pub fn texture_view_primary(&self) -> &TextureView {
        &self.primary_shadow_map.as_ref().unwrap().texture_view
    }

    #[inline]
    pub fn texture_view_secondary(&self) -> &TextureView {
        &self.secondary_shadow_map.as_ref().unwrap().texture_view
    }

    #[inline]
    pub fn final_texture_view(&self) -> &TextureView {
        if self.num_reductions % 2 == 0 {
            self.texture_view_secondary()
        } else {
            self.texture_view_primary()
        }
    }

    #[inline]
    pub fn work_group_count_per_light(&self) -> UVec3 {
        self.work_group_count_per_light
    }

    #[inline]
    pub fn work_group_count_total(&self) -> UVec3 {
        self.work_group_count_total
    }

    #[inline]
    pub fn num_reductions(&self) -> u32 {
        self.num_reductions
    }

    fn create_shadow_map(&self, render_device: &RenderDevice) -> GpuImage {
        let meta = self.meta;

        let shadow_map = render_device.create_texture(&TextureDescriptor {
            label: Some("shadow_map_2d"),
            size: Extent3d {
                width: meta.size,
                height: meta.size,
                depth_or_array_layers: meta.count,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: SHADOW_MAP_FORMAT,
            usage: TextureUsages::STORAGE_BINDING
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        GpuImage {
            texture_view: shadow_map.create_view(&TextureViewDescriptor {
                label: Some("shadow_map_2d_view"),
                format: Some(shadow_map.format()),
                dimension: Some(TextureViewDimension::D2Array),
                aspect: TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: 0,
                array_layer_count: Some(meta.count),
            }),
            texture_format: shadow_map.format(),
            texture: shadow_map,
            sampler: render_device.create_sampler(&SamplerDescriptor {
                label: Some("shadow_map_2d_sampler"),
                address_mode_u: AddressMode::ClampToEdge,
                address_mode_v: AddressMode::ClampToEdge,
                address_mode_w: AddressMode::ClampToEdge,
                mag_filter: FilterMode::Nearest,
                min_filter: FilterMode::Nearest,
                mipmap_filter: FilterMode::Nearest,
                lod_min_clamp: 0.,
                lod_max_clamp: f32::MAX,
                compare: None,
                anisotropy_clamp: 1,
                border_color: None,
            }),
            size: Vec2::splat(meta.size as f32),
            mip_level_count: 0,
        }
    }
}
