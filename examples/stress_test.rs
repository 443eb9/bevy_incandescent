// 10600KF 3070 rendering at 60fps with 250 lights

use bevy::{
    app::{App, Startup},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::system::Commands,
    prelude::PluginGroup,
    render::{color::Color, view::Msaa},
    window::{PresentMode, Window, WindowPlugin},
    DefaultPlugins,
};
use bevy_incandescent::{
    ecs::{bundle::PointLight2dBundle, light::PointLight2d},
    IncandescentPlugin,
};
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
            HelpersPlugin { inspector: false },
        ))
        .add_systems(Startup, setup)
        .insert_resource(Msaa::Off)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    for _ in 0..250 {
        commands.spawn(PointLight2dBundle {
            point_light: PointLight2d {
                color: Color::ORANGE_RED,
                intensity: 1.,
                radius: 100.,
                range: 100.,
            },
            ..Default::default()
        });
    }
}
