use bevy::{
    ecs::{
        system::Resource,
        world::{FromWorld, World},
    },
    math::{Mat4, UVec3, UVec4, Vec2, Vec4},
    render::{
        camera::OrthographicProjection,
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
pub struct GpuShadowMap2dMeta {
    pub index: u32,
    pub size: u32,
}

#[derive(Resource, Default)]
pub struct GpuShadowMap2dMetaBuffer(DynamicUniformBuffer<GpuShadowMap2dMeta>);

impl GpuShadowMap2dMetaBuffer {
    #[inline]
    pub fn push(&mut self, meta: GpuShadowMap2dMeta) -> DynamicUniformIndex<GpuShadowMap2dMeta> {
        DynamicUniformIndex::new(self.0.push(&meta))
    }

    #[inline]
    pub fn write_buffer(&mut self, render_device: &RenderDevice, render_queue: &RenderQueue) {
        self.0.write_buffer(render_device, render_queue);
    }

    #[inline]
    pub fn clear(&mut self) {
        self.0.clear();
    }

    #[inline]
    pub fn binding(&self) -> BindingResource {
        self.0.binding().unwrap()
    }
}

#[derive(Resource)]
pub struct ShadowMap2dConfig {
    pub near: f32,
    pub far: f32,
    pub size: usize,
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
    pub fn get_proj_mat(&self, proj: &OrthographicProjection) -> Mat4 {
        let t = self.size as f32;
        Mat4::orthographic_rh(t, t, t, t, proj.near, proj.far)
    }

    pub fn get_view_port(&self) -> UVec4 {
        UVec4::new(0, 0, self.size as u32, self.size as u32)
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
    work_group_count_per_light: UVec3,
    shadow_map: Option<GpuImage>,
}

impl ShadowMap2dStorage {
    pub fn try_update(&mut self, meta: ShadowMap2dMeta, render_device: &RenderDevice) {
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

        self.shadow_map = Some(GpuImage {
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
        self.work_group_count_per_light = UVec3 {
            x: meta.size.div_ceil(SHADOW_PREPASS_WORKGROUP_SIZE.x),
            y: meta.size.div_ceil(SHADOW_PREPASS_WORKGROUP_SIZE.y),
            z: 1,
        };
        self.meta = meta;
    }

    #[inline]
    pub fn texture_view(&self) -> &TextureView {
        &self.shadow_map.as_ref().unwrap().texture_view
    }

    #[inline]
    pub fn work_group_count_per_light(&self) -> UVec3 {
        self.work_group_count_per_light
    }
}
