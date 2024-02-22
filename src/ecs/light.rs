use bevy::{
    ecs::{component::Component, entity::Entity},
    render::{color::Color, texture::ColorAttachment},
};

#[derive(Component)]
pub struct ShadowCaster2d;

#[derive(Component)]
pub struct ShadowView2d {
    pub attachment: ColorAttachment,
}

#[derive(Component)]
pub struct ViewLight2dEntities(pub Vec<Entity>);

#[derive(Component, Default, Clone, Copy)]
pub struct PointLight2d {
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
    pub radius: f32,
}
