use bevy::{
    ecs::system::Resource,
    math::{Mat4, UVec4, Vec2, Vec4, Vec4Swizzles},
    render::{
        camera::OrthographicProjection,
        render_resource::{BindingResource, DynamicUniformBuffer, ShaderType},
        renderer::{RenderDevice, RenderQueue},
        view::ExtractedView,
    },
    transform::components::GlobalTransform,
};

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

#[derive(Resource, Default)]
pub struct GpuLights2d {
    point: DynamicUniformBuffer<GpuPointLight2d>,
}

impl GpuLights2d {
    #[inline]
    pub fn add_point_light(&mut self, light: GpuPointLight2d) {
        self.point.push(&light);
    }

    #[inline]
    pub fn point_lights_binding(&self) -> BindingResource {
        self.point.binding().unwrap()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.point.clear();
    }

    #[inline]
    pub fn write_buffers(&mut self, render_device: &RenderDevice, render_queue: &RenderQueue) {
        self.point.write_buffer(render_device, render_queue);
    }
}

#[derive(Resource)]
pub struct PointLight2dShadowMap {
    pub size: usize,
}

impl Default for PointLight2dShadowMap {
    fn default() -> Self {
        Self { size: 1024 }
    }
}

impl PointLight2dShadowMap {
    pub fn get_proj_mat(&self, proj: &OrthographicProjection) -> Mat4 {
        let t = self.size as f32;
        Mat4::orthographic_rh(t, t, t, t, proj.near, proj.far)
    }

    pub fn get_view_port(&self) -> UVec4 {
        UVec4::new(0, 0, self.size as u32, self.size as u32)
    }
}
