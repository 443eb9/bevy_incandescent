use bevy::{
    ecs::{
        system::Resource,
        world::{FromWorld, World},
    },
    math::{Mat4, UVec3, Vec2, Vec4},
    render::{
        render_resource::{
            AddressMode, BindingResource, DynamicUniformBuffer, Extent3d, FilterMode,
            GpuArrayBuffer, SamplerDescriptor, ShaderType, TextureAspect, TextureDescriptor,
            TextureDimension, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
            TextureViewDimension,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::GpuImage,
    },
    transform::components::GlobalTransform,
};

use super::{
    extract::ExtractedPointLight2d, prepare::DynamicUniformIndex, SHADOW_PREPASS_WORKGROUP_SIZE,
};

#[derive(ShaderType, Clone)]
pub struct GpuPointLight2d {
    pub world_position: Vec4,
    pub color: Vec4,
}

impl GpuPointLight2d {
    pub fn new(light_transform: &GlobalTransform, light: &ExtractedPointLight2d) -> Self {
        Self {
            world_position: light_transform.translation().extend(1.),
            color: light.color.rgba_to_vec4(),
        }
    }
}

#[derive(ShaderType, Clone)]
pub struct GpuShadowView2d {
    pub view: Mat4,
    pub projection: Mat4,
}

impl GpuShadowView2d {
    pub fn new(light_transform: &GlobalTransform, shadow_map_config: &ShadowMap2dConfig) -> Self {
        let size = shadow_map_config.size as f32;
        Self {
            view: light_transform.compute_matrix(),
            projection: Mat4::orthographic_rh(
                -size,
                size,
                -size,
                size,
                shadow_map_config.near,
                shadow_map_config.far,
            ),
        }
    }
}

#[derive(Resource)]
pub struct GpuLights2d {
    views: GpuArrayBuffer<GpuShadowView2d>,
    point_lights: GpuArrayBuffer<GpuPointLight2d>,
}

impl FromWorld for GpuLights2d {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        Self {
            views: GpuArrayBuffer::new(&render_device),
            point_lights: GpuArrayBuffer::new(&render_device),
        }
    }
}

impl GpuLights2d {
    #[inline]
    pub fn add_point_light(&mut self, view: GpuShadowView2d, light: GpuPointLight2d) {
        self.views.push(view);
        self.point_lights.push(light);
    }

    #[inline]
    pub fn shadow_views_binding(&self) -> BindingResource {
        self.views.binding().unwrap()
    }

    #[inline]
    pub fn point_lights_binding(&self) -> BindingResource {
        self.point_lights.binding().unwrap()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.views.clear();
        self.point_lights.clear();
    }

    #[inline]
    pub fn write_buffers(&mut self, render_device: &RenderDevice, render_queue: &RenderQueue) {
        self.views.write_buffer(render_device, render_queue);
        self.point_lights.write_buffer(render_device, render_queue);
    }
}

#[derive(ShaderType)]
pub struct GpuShadowMapMeta {
    pub index: u32,
    pub size: u32,
}

#[derive(Resource, Default)]
pub struct GpuMetaBuffers {
    light: DynamicUniformBuffer<GpuShadowMapMeta>,
    reduction_time: DynamicUniformBuffer<u32>,
    reduction_time_indices: Vec<u32>,
}

impl GpuMetaBuffers {
    #[inline]
    pub fn push_light_meta(
        &mut self,
        meta: GpuShadowMapMeta,
    ) -> DynamicUniformIndex<GpuShadowMapMeta> {
        DynamicUniformIndex::new(self.light.push(&meta))
    }

    #[inline]
    pub fn init_reduction_time_buffer(&mut self, reduction_time: u32) {
        self.reduction_time.clear();
        self.reduction_time_indices.clear();

        for i in 0..reduction_time {
            let idx = self.reduction_time.push(&(i + 1));
            self.reduction_time_indices.push(idx);
        }
    }

    #[inline]
    pub fn get_reduction_time_index(&self, reduction_time: u32) -> u32 {
        self.reduction_time_indices[reduction_time as usize]
    }

    #[inline]
    pub fn clear(&mut self) {
        self.light.clear();
    }

    #[inline]
    pub fn write_buffers(&mut self, render_device: &RenderDevice, render_queue: &RenderQueue) {
        self.light.write_buffer(render_device, render_queue);
        self.reduction_time
            .write_buffer(render_device, render_queue);
    }

    #[inline]
    pub fn shadow_map_meta_buffer_binding(&self) -> BindingResource {
        self.light.binding().unwrap()
    }

    #[inline]
    pub fn reduction_time_buffer_binding(&self) -> BindingResource {
        self.reduction_time.binding().unwrap()
    }
}

#[derive(Resource)]
pub struct ShadowMap2dConfig {
    pub near: f32,
    pub far: f32,
    pub size: u32,
}

impl Default for ShadowMap2dConfig {
    fn default() -> Self {
        Self {
            near: -1000.,
            far: 1000.,
            size: 512,
        }
    }
}

impl ShadowMap2dConfig {
    pub fn get_proj_mat(&self, scale: f32) -> Mat4 {
        Mat4::orthographic_rh(-scale, scale, -scale, scale, self.near, self.far)
    }
}

#[derive(Default, PartialEq, Eq)]
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
    reduction_time: u32,
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
            format: TextureFormat::Rg32Float,
            usage: TextureUsages::STORAGE_BINDING
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        self.primary_shadow_map = Some(GpuImage {
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
        });
        self.secondary_shadow_map = self.primary_shadow_map.clone();
        self.work_group_count_per_light = UVec3 {
            x: meta.size.div_ceil(SHADOW_PREPASS_WORKGROUP_SIZE.x),
            y: meta.size.div_ceil(SHADOW_PREPASS_WORKGROUP_SIZE.y),
            z: 1,
        };
        self.work_group_count_total = UVec3 {
            x: self.work_group_count_per_light.x * meta.size,
            y: self.work_group_count_per_light.y * meta.size,
            z: meta.count,
        };
        self.reduction_time = meta.size.trailing_zeros();
        self.meta = meta;

        assert_eq!(
            2u32.pow(self.reduction_time),
            self.meta.size,
            "Shadow map size must be a power of 2!"
        );

        meta_buffers.init_reduction_time_buffer(self.reduction_time);
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
    pub fn work_group_count_per_light(&self) -> UVec3 {
        self.work_group_count_per_light
    }

    #[inline]
    pub fn work_group_count_total(&self) -> UVec3 {
        self.work_group_count_total
    }

    #[inline]
    pub fn reduction_time(&self) -> u32 {
        self.reduction_time
    }

    #[inline]
    pub fn final_texture_view(&self) -> &TextureView {
        if self.reduction_time % 2 == 0 {
            &self.primary_shadow_map.as_ref().unwrap().texture_view
        } else {
            &self.secondary_shadow_map.as_ref().unwrap().texture_view
        }
    }
}
