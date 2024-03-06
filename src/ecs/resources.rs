use bevy::{ecs::system::Resource, math::Mat4};

#[derive(Resource, Clone, Copy)]
pub struct ShadowMap2dConfig {
    pub near: f32,
    pub far: f32,
    pub size: u32,
}

impl Default for ShadowMap2dConfig {
    fn default() -> Self {
        Self {
            near: -1000.,
            far: 1000.,
            size: 512,
        }
    }
}

impl ShadowMap2dConfig {
    pub fn get_proj_mat(&self, size: f32) -> Mat4 {
        Mat4::orthographic_rh(-size, size, -size, size, self.near, self.far)
    }
}
