use bevy::{ecs::system::Resource, render::extract_resource::ExtractResource};

#[derive(Resource, ExtractResource, Clone, Copy)]
pub struct RayMarchingConfig {
    pub scale: f32,
    pub alpha_threshold: f32,
}

impl Default for RayMarchingConfig {
    fn default() -> Self {
        Self {
            scale: 1.,
            alpha_threshold: 0.9,
        }
    }
}
