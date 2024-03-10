#![allow(unused)]
use bevy::{
    app::{App, Plugin, Update},
    ecs::{
        query::{Or, With},
        system::Query,
    },
    gizmos::gizmos::Gizmos,
    math::Vec3Swizzles,
    render::view::VisibleEntities,
    transform::components::GlobalTransform,
};

use crate::ecs::{PointLight2d, SpotLight2d};

pub struct IncandescentDebugPlugin;

impl Plugin for IncandescentDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                draw_light_range,
                // print_light_visible_entities,
            ),
        );
    }
}

fn draw_light_range(
    mut gizmos: Gizmos,
    point_lights_query: Query<(&GlobalTransform, &PointLight2d)>,
    spot_lights_query: Query<(&GlobalTransform, &SpotLight2d)>,
) {
    for (transform, light) in point_lights_query.iter() {
        gizmos.circle_2d(transform.translation().xy(), light.range, light.color);
    }
    for (transform, light) in spot_lights_query.iter() {
        gizmos.circle_2d(transform.translation().xy(), light.range, light.color);
    }
}

fn print_light_visible_entities(
    lights_query: Query<&VisibleEntities, Or<(With<PointLight2d>, With<SpotLight2d>)>>,
) {
    for visible_entities in lights_query.iter() {
        println!("{:?}", visible_entities);
    }
}
