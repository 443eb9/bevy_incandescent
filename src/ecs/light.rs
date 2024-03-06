use bevy::{
    ecs::component::Component,
    math::bounding::Aabb2d,
    render::{color::Color, texture::ColorAttachment, view::RenderLayers},
};

use bevy::reflect::Reflect;

#[derive(Component)]
pub struct ShadowCaster2d;

#[derive(Component)]
pub struct ShadowCaster2dVisibility(pub bool);

#[derive(Component)]
pub struct ShadowView2d {
    pub attachment: ColorAttachment,
}

#[derive(Component, Default, Clone, Copy, Reflect)]
pub struct PointLight2d {
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
    pub radius: f32,
}

#[derive(Component)]
pub struct ShadowLayers(pub RenderLayers);

#[derive(Component)]
pub struct Light2dAabb(pub Aabb2d);
