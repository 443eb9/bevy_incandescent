use bevy::{
    ecs::{component::Component, entity::Entity},
    render::{color::Color, texture::ColorAttachment, view::RenderLayers},
};

#[cfg(feature = "debug")]
use bevy::reflect::Reflect;

#[derive(Component)]
pub struct ShadowCaster2d;

#[derive(Component)]
pub struct ShadowCaster2dVisibility(pub bool);

#[derive(Component)]
pub struct ShadowView2d {
    pub attachment: ColorAttachment,
}

#[derive(Component)]
pub struct VisibleLight2dEntities(pub Vec<Entity>);

#[derive(Component, Default, Clone, Copy)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub struct PointLight2d {
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
    pub radius: f32,
}

#[derive(Component)]
pub struct ShadowLayers(pub RenderLayers);
