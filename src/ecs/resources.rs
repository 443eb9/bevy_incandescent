use bevy::{
    ecs::{reflect::ReflectResource, system::Resource},
    math::{Mat4, Vec2},
    reflect::Reflect,
    render::color::Color,
};

#[derive(Resource, Clone, Copy, Reflect)]
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

#[derive(Resource, Clone, Copy, Reflect)]
#[reflect(Resource)]
pub struct ShadowMap2dConfig {
    pub near: f32,
    pub far: f32,
    pub size: u32,
    pub offset: Vec2,
    pub bias: f32,
    pub alpha_threshold: f32,
    pub pcf_samples: u32,
    pub pcf_radius: f32,
}

impl Default for ShadowMap2dConfig {
    fn default() -> Self {
        Self {
            near: -1000.,
            far: 1000.,
            size: 512,
            offset: Vec2::ZERO,
            bias: 0.,
            alpha_threshold: 0.9,
            pcf_samples: 32,
            pcf_radius: 4.,
        }
    }
}

impl ShadowMap2dConfig {
    pub fn get_proj_mat(&self, size: f32) -> Mat4 {
        Mat4::orthographic_rh(-size, size, -size, size, self.near, self.far)
    }
}
