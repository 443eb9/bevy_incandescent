use bevy::{
    ecs::{
        entity::Entity,
        system::{ParallelCommands, Query},
    },
    math::Vec3A,
    render::primitives::Aabb,
    transform::components::GlobalTransform,
};

use crate::ecs::PointLight2d;

pub fn calc_light_bounds(
    commands: ParallelCommands,
    lights_query: Query<(Entity, &GlobalTransform, &PointLight2d)>,
) {
    lights_query
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
