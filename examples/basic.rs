use bevy::{
    app::{App, Startup},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::system::Commands,
    math::Vec2,
    render::color::Color,
    sprite::{Sprite, SpriteBundle},
    transform::components::Transform,
    DefaultPlugins,
};
use bevy_incandescent::{
    ecs::{
        bundle::{PointLight2dBundle, ShadowCaster2dBundle},
        light::PointLight2d,
    },
    IncandescentPlugin,
};
use rand::Rng;

const OBSTALCE_AREA: Vec2 = Vec2 { x: 900., y: 600. };
const OBSTACLE_SIZE_MIN: Vec2 = Vec2 { x: 50., y: 50. };
const OBSTACLE_SIZE_MAX: Vec2 = Vec2 { x: 200., y: 200. };

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, IncandescentPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    let mut rd = rand::thread_rng();
    for _ in 0..10 {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(rd.gen(), rd.gen(), rd.gen()),
                    custom_size: Some(
                        Vec2::new(rd.gen(), rd.gen()) * (OBSTACLE_SIZE_MAX - OBSTACLE_SIZE_MIN)
                            + OBSTACLE_SIZE_MIN,
                    ),
                    ..Default::default()
                },
                transform: Transform::from_translation(
                    ((Vec2::new(rd.gen(), rd.gen()) - Vec2::splat(0.5)) * OBSTALCE_AREA).extend(0.),
                ),
                ..Default::default()
            },
            ShadowCaster2dBundle::default(),
        ));
    }

    commands.spawn(PointLight2dBundle {
        point_light: PointLight2d {
            color: Color::rgb(rd.gen(), rd.gen(), rd.gen()),
            intensity: 1000.,
            range: 1000.,
            radius: 100.,
        },
        ..Default::default()
    });
}
