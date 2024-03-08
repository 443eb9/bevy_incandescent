use bevy::app::{App, Plugin};

use self::{
    light::PointLight2d,
    resources::{AmbientLight2d, ShadowMap2dConfig},
};

pub mod bundle;
pub mod camera;
pub mod light;
pub mod pbr;
pub mod resources;

pub struct IncandescentECSPlugin;

impl Plugin for IncandescentECSPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShadowMap2dConfig>()
            .init_resource::<AmbientLight2d>();

        app.register_type::<PointLight2d>();

        app.register_type::<ShadowMap2dConfig>()
            .register_type::<AmbientLight2d>();
    }
}
