use bevy::{
    app::{App, PluginGroup, Startup},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::system::Commands,
    window::{PresentMode, Window, WindowPlugin},
    DefaultPlugins,
};
use bevy_incandescent::IncandescentPlugin;
use helpers::HelpersPlugin;

mod helpers;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: PresentMode::Immediate,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            IncandescentPlugin,
            HelpersPlugin { inspector: true },
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
