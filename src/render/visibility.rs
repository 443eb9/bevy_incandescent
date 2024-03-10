use bevy::{
    ecs::{
        entity::Entity,
        system::{ParallelCommands, Query},
    },
    math::Vec3A,
    render::primitives::Aabb,
    transform::components::GlobalTransform,
};

use crate::ecs::{PointLight2d, SpotLight2d};

pub fn calc_light_bounds(
    commands: ParallelCommands,
    point_lights_query: Query<(Entity, &GlobalTransform, &PointLight2d)>,
    spot_lights_query: Query<(Entity, &GlobalTransform, &SpotLight2d)>,
) {
    point_lights_query
        .par_iter()
        .for_each(|(entity, transform, light)| {
            commands.command_scope(|mut c| {
                c.entity(entity).insert(Aabb {
                    center: transform.translation().into(),
                    half_extents: Vec3A::new(light.range, light.range, 1000.),
                });
            });
        });

    spot_lights_query
        .par_iter()
        .for_each(|(entity, transform, light)| {
            commands.command_scope(|mut c| {
                c.entity(entity).insert(Aabb {
                    center: transform.translation().into(),
                    half_extents: Vec3A::new(light.range, light.range, 1000.),
                });
            });
        });
}
