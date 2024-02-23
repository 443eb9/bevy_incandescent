use bevy::app::{App, Plugin};

pub mod bundle;
pub mod camera;
pub mod light;

pub struct IncandescentECSPlugin;

impl Plugin for IncandescentECSPlugin {
    fn build(&self, _app: &mut App) {}
}
