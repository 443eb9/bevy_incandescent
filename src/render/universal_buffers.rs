use bevy::{
    ecs::{
        system::Resource,
        world::{FromWorld, World},
    },
    render::{
        render_resource::{BindingResource, DynamicUniformBuffer},
        renderer::{RenderDevice, RenderQueue},
    },
};

pub const DEFAULT_MAX_NUMBER: u32 = 100;

#[derive(Resource)]
pub struct NumberBuffer {
    buffer: DynamicUniformBuffer<u32>,
    indices: Vec<u32>,
}

impl FromWorld for NumberBuffer {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let render_queue = world.resource::<RenderQueue>();

        let mut buffer = DynamicUniformBuffer::default();
        let mut indices = Vec::with_capacity(DEFAULT_MAX_NUMBER as usize);

        for i in 0..DEFAULT_MAX_NUMBER {
            let index = buffer.push(&i);
            indices.push(index);
        }

        buffer.write_buffer(render_device, render_queue);

        Self { buffer, indices }
    }
}

impl NumberBuffer {
    #[inline]
    pub fn get_index(&self, number: u32) -> u32 {
        self.indices[number as usize]
    }

    #[inline]
    pub fn binding(&self) -> BindingResource {
        self.buffer.binding().unwrap()
    }
}

#[derive(Resource)]
pub struct BooleanBuffer {
    buffer: DynamicUniformBuffer<u32>,
    indices: Vec<u32>,
}

impl FromWorld for BooleanBuffer {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let render_queue = world.resource::<RenderQueue>();

        let mut buffer = DynamicUniformBuffer::default();
        let mut indices = Vec::with_capacity(2);

        for i in 0..=1 {
            let index = buffer.push(&i);
            indices.push(index);
        }

        buffer.write_buffer(render_device, render_queue);

        Self { buffer, indices }
    }
}

impl BooleanBuffer {
    #[inline]
    pub fn get_index(&self, boolean: bool) -> u32 {
        self.indices[boolean as usize]
    }

    #[inline]
    pub fn binding(&self) -> BindingResource {
        self.buffer.binding().unwrap()
    }
}
