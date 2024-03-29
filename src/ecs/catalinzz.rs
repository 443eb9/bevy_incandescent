use bevy::{
    ecs::{component::Component, reflect::ReflectResource, system::Resource},
    math::{Mat4, Vec2},
    reflect::Reflect,
    render::extract_resource::ExtractResource,
};

#[derive(Component)]
pub struct MainShadowCameraDriver;

#[derive(Resource, ExtractResource, Clone, Copy, Reflect)]
#[reflect(Resource)]
pub struct ShadowMap2dConfig {
    pub near: f32,
    pub far: f32,
    pub size: u32,
    pub offset: Vec2,
    pub bias: f32,
    pub alpha_threshold: f32,
    pub pcf: PcfConfig,
}

impl Default for ShadowMap2dConfig {
    fn default() -> Self {
        Self {
            near: -1000.,
            far: 1000.,
            size: 512,
            offset: Vec2::ZERO,
            bias: 0.005,
            alpha_threshold: 0.9,
            pcf: Default::default(),
        }
    }
}

impl ShadowMap2dConfig {
    pub fn get_proj_mat(&self, size: f32) -> Mat4 {
        Mat4::orthographic_rh(-size, size, -size, size, self.near, self.far)
    }
}

#[derive(Clone, Copy, Reflect)]
pub struct PcfConfig {
    pub seed: u32,
    pub samples: u32,
    pub radius: f32,
}

impl Default for PcfConfig {
    fn default() -> Self {
        Self {
            seed: 1,
            samples: 32,
            radius: 2.,
        }
    }
}
