use bevy::ecs::reflect::ReflectResource;
use bevy::ecs::system::Resource;
use bevy::render::extract_resource::ExtractResource;
use bevy::{ecs::component::Component, math::bounding::Aabb2d, render::color::Color};

pub mod bundle;
pub mod catalinzz;

use bevy::reflect::Reflect;

#[derive(Component, Default, Clone, Copy, Reflect)]
pub struct PointLight2d {
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
    pub radius: f32,
}

#[derive(Component)]
pub struct Light2dAabb(pub Aabb2d);

#[derive(Component)]
pub struct ShadowCaster2d;

#[derive(Resource, ExtractResource, Clone, Copy, Reflect)]
#[reflect(Resource)]
pub struct AmbientLight2d {
    pub color: Color,
    pub intensity: f32,
}

impl Default for AmbientLight2d {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            intensity: 1.,
        }
    }
}
