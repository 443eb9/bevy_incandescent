use std::marker::PhantomData;

use bevy::{
    ecs::{component::Component, entity::Entity},
    render::{color::Color, render_resource::ShaderType, texture::ColorAttachment},
};

#[derive(Component)]
pub struct ShadowCaster2d;

#[derive(Component)]
pub struct ShadowView2d {
    pub attachment: ColorAttachment,
}

#[derive(Component)]
pub struct DynamicUniformIndex<U: ShaderType> {
    index: u32,
    _marker: PhantomData<U>,
}

impl<U: ShaderType> DynamicUniformIndex<U> {
    #[inline]
    pub fn new(index: u32) -> Self {
        Self {
            index,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn index(&self) -> u32 {
        self.index
    }
}

#[derive(Component)]
pub struct VisibleLight2dEntities(pub Vec<Entity>);

#[derive(Component, Default, Clone, Copy)]
pub struct PointLight2d {
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
    pub radius: f32,
}
