use bevy::{
    ecs::{reflect::ReflectResource, system::Resource},
    reflect::Reflect,
    render::extract_resource::ExtractResource,
};

#[derive(Resource, ExtractResource, Clone, Copy, Reflect)]
#[reflect(Resource)]
pub struct RayMarchingConfig {
    pub scale: f32,
    pub alpha_threshold: f32,
    pub edge_lighting: f32,
    pub hardness: f32,
}

impl Default for RayMarchingConfig {
    fn default() -> Self {
        Self {
            scale: 1.,
            alpha_threshold: 0.9,
            edge_lighting: 5.,
            hardness: 16.,
        }
    }
}
