use bevy::{
    ecs::system::Resource,
    math::{Mat4, UVec2, UVec4, Vec2, Vec4, Vec4Swizzles},
    render::{
        camera::OrthographicProjection,
        render_resource::{BindingResource, DynamicUniformBuffer, ShaderType},
        renderer::{RenderDevice, RenderQueue},
        view::ExtractedView,
    },
    transform::components::GlobalTransform,
};

use crate::ecs::light::DynamicUniformIndex;

use super::extract::ExtractedPointLight2d;

#[derive(ShaderType)]
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

#[derive(ShaderType)]
pub struct ShadowView2dUniform {
    pub view: Mat4,
    pub projection: Mat4,
}

impl ShadowView2dUniform {
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

#[derive(Resource, Default)]
pub struct GpuLights2d {
    views: DynamicUniformBuffer<ShadowView2dUniform>,
    point_lights: DynamicUniformBuffer<GpuPointLight2d>,
}

impl GpuLights2d {
    #[inline]
    pub fn add_point_light(
        &mut self,
        view: ShadowView2dUniform,
        light: GpuPointLight2d,
    ) -> (
        DynamicUniformIndex<ShadowView2dUniform>,
        DynamicUniformIndex<GpuPointLight2d>,
    ) {
        (
            DynamicUniformIndex::<ShadowView2dUniform>::new(self.views.push(&view)),
            DynamicUniformIndex::<GpuPointLight2d>::new(self.point_lights.push(&light)),
        )
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
            size: 1024,
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
