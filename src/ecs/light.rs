use bevy::{ecs::component::Component, math::UVec2, render::color::Color};

pub trait Light2d {
    fn get_affected_square(&self) -> UVec2;
}

#[derive(Component)]
pub struct ShadowCaster2d;

#[derive(Component)]
pub struct PointLight2d {
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
    pub radius: f32,
}

impl Light2d for PointLight2d {
    #[inline]
    fn get_affected_square(&self) -> UVec2 {
        UVec2::splat(self.range.ceil() as u32)
    }
}
