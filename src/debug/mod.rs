use bevy::{
    app::{App, Plugin, Update},
    ecs::system::Query,
    gizmos::gizmos::Gizmos,
    math::Vec3Swizzles,
    transform::components::GlobalTransform,
};

use crate::ecs::light::PointLight2d;

pub struct IncandescentDebugPlugin;

impl Plugin for IncandescentDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_light_range);
    }
}

fn draw_light_range(mut gizmos: Gizmos, lights_query: Query<(&GlobalTransform, &PointLight2d)>) {
    for (transform, light) in lights_query.iter() {
        gizmos.circle_2d(transform.translation().xy(), light.range, light.color);
    }
}
