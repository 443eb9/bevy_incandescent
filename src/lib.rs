use bevy::app::{App, Plugin};
use render::IncandescentRenderPlugin;

pub mod ecs;
pub mod render;

pub struct IncandescentPlugin;

impl Plugin for IncandescentPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(IncandescentRenderPlugin);
    }
}
