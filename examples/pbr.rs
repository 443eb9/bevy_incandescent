use bevy::{
    app::{App, Startup, Update},
    asset::{AssetServer, Assets},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::{
        component::Component,
        query::With,
        system::{Commands, Query, Res, ResMut},
    },
    input::{keyboard::KeyCode, ButtonInput},
    math::{primitives::Rectangle, Vec2},
    render::{
        camera::{CameraProjection, OrthographicProjection},
        color::Color,
        mesh::Mesh,
        view::{Msaa, NoFrustumCulling},
    },
    transform::components::{GlobalTransform, Transform},
    window::Window,
    DefaultPlugins,
};
use bevy_incandescent::{
    ecs::{
        pbr::{PbrMesh2dBundle, StandardMaterial2d},
        PointLight2d, PointLight2dBundle, ShadowCaster2dBundle,
    },
    IncandescentPlugin,
};
use helpers::HelpersPlugin;

mod helpers;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            IncandescentPlugin::default(),
            HelpersPlugin { inspector: true },
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, light_follow)
        .insert_resource(Msaa::Off)
        .run();
}

#[derive(Component)]
struct ControllableLight;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial2d>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
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
        PbrMesh2dBundle {
            material: materials.add(StandardMaterial2d {
                base_color_texture: Some(asset_server.load("pbr/suzzane-color.png")),
                normal_map_texture: Some(asset_server.load("pbr/suzzane-normal.png")),
                ..Default::default()
            }),
            mesh: meshes.add(Rectangle::new(512., 512.)),
            ..Default::default()
        },
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
    let cursor_pos_ws = camera_transform.compute_matrix()
        * camera_proj.get_projection_matrix().inverse()
        * cursor_pos_ndc;

    light_transform.translation = cursor_pos_ws.truncate();
}
