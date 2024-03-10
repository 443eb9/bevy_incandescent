use bevy::{
    ecs::{component::Component, system::Resource},
    math::{Vec2, Vec4},
    render::{
        render_resource::{BindingResource, DynamicUniformBuffer, GpuArrayBuffer, ShaderType},
        renderer::{RenderDevice, RenderQueue},
    },
};

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
    pub angles: [f32; 2],
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
