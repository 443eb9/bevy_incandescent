use bevy::{
    app::{App, Startup, Update},
    asset::AssetServer,
    core_pipeline::core_2d::Camera2dBundle,
    ecs::{
        component::Component,
        query::With,
        system::{Commands, Query, Res},
    },
    input::{keyboard::KeyCode, ButtonInput},
    math::Vec2,
    render::{
        camera::{Camera, CameraProjection, OrthographicProjection},
        color::Color,
        view::{Msaa, NoFrustumCulling},
    },
    sprite::{Sprite, SpriteBundle},
    transform::components::{GlobalTransform, Transform},
    window::Window,
    DefaultPlugins,
};
use bevy_incandescent::{
    ecs::{
        bundle::{PointLight2dBundle, ShadowCaster2dBundle},
        light::PointLight2d,
        pbr::{HeightTexture, NormalTexture},
    },
    IncandescentPlugin,
};
use helpers::HelpersPlugin;

mod helpers;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            IncandescentPlugin,
            HelpersPlugin { inspector: true },
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, light_follow)
        .insert_resource(Msaa::Off)
        .run();
}

#[derive(Component)]
struct ControllableLight;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        PointLight2dBundle {
            point_light: PointLight2d {
                color: Color::ORANGE,
                intensity: 1.,
                radius: 50.,
                range: 300.,
            },
            transform: Transform::from_xyz(0., 300., 0.),
            ..Default::default()
        },
        ControllableLight,
        NoFrustumCulling,
    ));

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::splat(512.)),
                ..Default::default()
            },
            texture: asset_server.load("pbr/suzzane-color.png"),
            ..Default::default()
        },
        HeightTexture::from(asset_server.load("pbr/suzzane-height.png")),
        NormalTexture::from(asset_server.load("pbr/suzzane-normal.png")),
        ShadowCaster2dBundle::default(),
        NoFrustumCulling,
    ));
}

fn light_follow(
    mut light: Query<&mut Transform, With<ControllableLight>>,
    window: Query<&Window>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    camera: Query<(&GlobalTransform, &OrthographicProjection)>,
) {
    if !keyboard_input.pressed(KeyCode::ShiftLeft) {
        return;
    }

    let (Ok(mut light_transform), Ok(window), Ok((camera_transform, camera_proj))) = (
        light.get_single_mut(),
        window.get_single(),
        camera.get_single(),
    ) else {
        return;
    };

    let Some(cursor_position) = window.physical_cursor_position() else {
        return;
    };

    let mut cursor_pos_ndc = (cursor_position
        / Vec2::new(
            window.physical_width() as f32,
            window.physical_height() as f32,
        )
        * 2.
        - 1.)
        .extend(0.)
        .extend(1.);
    cursor_pos_ndc.y = -cursor_pos_ndc.y;
    let cursor_pos_ws = camera_transform.compute_matrix().inverse()
        * camera_proj.get_projection_matrix().inverse()
        * cursor_pos_ndc;

    light_transform.translation = cursor_pos_ws.truncate();
}
