use bevy::{ecs::component::Component, math::Vec2};

#[derive(Component)]
pub struct ShadowCameraDriver;

#[derive(Component)]
pub struct ShadowCamera {
    pub proj_size: Vec2,
}
