use bevy::app::{App, Plugin};

use self::resources::ShadowMap2dConfig;

pub mod bundle;
pub mod camera;
pub mod light;
pub mod resources;

pub struct IncandescentECSPlugin;

impl Plugin for IncandescentECSPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShadowMap2dConfig>();

        #[cfg(feature = "debug")]
        {
            use light::*;
            app.register_type::<PointLight2d>();
        }
    }
}
