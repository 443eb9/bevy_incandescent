use bevy::app::{App, Plugin};

pub mod bundle;
pub mod light;

pub struct IncandescentECSPlugin;

impl Plugin for IncandescentECSPlugin {
    fn build(&self, _app: &mut App) {}
}
