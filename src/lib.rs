use bevy::app::{App, Plugin};
use ecs::IncandescentEcsPlugin;
use render::IncandescentRenderPlugin;

#[cfg(feature = "debug")]
pub mod debug;
pub mod ecs;
pub mod math;
pub mod render;

#[cfg(not(any(feature = "catalinzz", feature = "sdf")))]
compile_error!("Incandescent requires at least one render approach feature to be enabled!");

pub struct IncandescentPlugin;

impl Plugin for IncandescentPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            IncandescentRenderPlugin,
            IncandescentEcsPlugin,
            #[cfg(feature = "debug")]
            debug::IncandescentDebugPlugin,
        ));
    }
}
