use bevy::app::{App, Plugin};

pub mod bundle;
pub mod camera;
pub mod light;

pub struct IncandescentECSPlugin;

impl Plugin for IncandescentECSPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "debug")]
        {
            use light::*;
            app.register_type::<PointLight2d>();
        }
    }
}
