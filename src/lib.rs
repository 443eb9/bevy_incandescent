use bevy::app::{App, Plugin};
use ecs::IncandescentECSPlugin;
use render::IncandescentRenderPlugin;

#[cfg(feature = "debug")]
pub mod debug;
pub mod ecs;
pub mod render;

pub struct IncandescentPlugin;

impl Plugin for IncandescentPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            IncandescentRenderPlugin,
            IncandescentECSPlugin,
            #[cfg(feature = "debug")]
            debug::IncandescentDebugPlugin,
        ));
    }
}
