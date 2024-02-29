use std::cell::Cell;

use bevy::{
    ecs::{
        entity::Entity,
        query::With,
        system::{Commands, Local, ParallelCommands, Query},
    },
    math::{
        bounding::{Aabb2d, IntersectsVolume},
        Vec2, Vec3Swizzles,
    },
    render::{
        camera::{Camera, OrthographicProjection},
        primitives::{Aabb, Frustum},
        view::{InheritedVisibility, RenderLayers, Visibility},
    },
    transform::components::GlobalTransform,
};
use thread_local::ThreadLocal;

use crate::ecs::light::{Light2dAabb, PointLight2d, ShadowLayers, VisibleLight2dEntities};

pub fn calc_light_bounds(
    commands: ParallelCommands,
    lights_query: Query<(Entity, &GlobalTransform, &PointLight2d)>,
) {
    lights_query
        .par_iter()
        .for_each(|(entity, transform, light)| {
            let range = Vec2::splat(light.range);
            let center = transform.translation().xy();

            commands.command_scope(|mut c| {
                c.entity(entity).insert(Light2dAabb(Aabb2d {
                    min: center - range,
                    max: center + range,
                }));
            });
        });
}

pub fn check_light_visibility(
    mut thread_queues: Local<ThreadLocal<Cell<Vec<Entity>>>>,
    mut main_view_query: Query<(
        &mut VisibleLight2dEntities,
        Option<&RenderLayers>,
        &Camera,
        &OrthographicProjection,
    )>,
    lights_query: Query<
        (
            Entity,
            &Light2dAabb,
            Option<&RenderLayers>,
            &InheritedVisibility,
            &Visibility,
        ),
        With<PointLight2d>,
    >,
) {
    for (mut visible_lights, camera_render_layers, camera, projection) in &mut main_view_query {
        if !camera.is_active {
            return;
        }

        let camera_aabb = Aabb2d {
            min: projection.area.min,
            max: projection.area.max,
        };

        visible_lights.0.clear();
        lights_query.par_iter().for_each(
            |(entity, aabb, light_render_layers, inherited_visibility, visibility)| {
                let camera_render_layers = camera_render_layers.copied().unwrap_or_default();
                let light_render_layers = light_render_layers.copied().unwrap_or_default();

                if !camera_render_layers.intersects(&light_render_layers)
                    || !inherited_visibility.get()
                    || *visibility == Visibility::Hidden
                    || !camera_aabb.intersects(&aabb.0)
                {
                    return;
                }

                let cell = thread_queues.get_or_default();
                let mut queue = cell.take();
                queue.push(entity);
                cell.set(queue);
            },
        );

        for cell in &mut thread_queues {
            visible_lights.0.append(cell.get_mut());
        }
    }
}
