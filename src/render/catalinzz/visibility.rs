use std::cell::Cell;

use bevy::{
    ecs::{
        entity::Entity,
        query::{Changed, Has, Or, With},
        system::{Local, Query, Res},
    },
    render::{
        primitives::{Aabb, Frustum, Sphere},
        view::{
            InheritedVisibility, NoFrustumCulling, RenderLayers, ViewVisibility, VisibleEntities,
        },
    },
    transform::components::GlobalTransform,
};
use thread_local::ThreadLocal;

use crate::ecs::{catalinzz::ShadowMap2dConfig, PointLight2d, ShadowCaster2d};

pub fn update_light_frusta(
    mut lights_query: Query<
        (&GlobalTransform, &mut Frustum, &PointLight2d),
        Or<(Changed<GlobalTransform>, Changed<PointLight2d>)>,
    >,
    shadow_map_config: Res<ShadowMap2dConfig>,
) {
    lights_query
        .par_iter_mut()
        .for_each(|(transform, mut frustum, light)| {
            let view_proj =
                shadow_map_config.get_proj_mat(light.range) * transform.compute_matrix().inverse();
            *frustum = Frustum::from_view_projection_custom_far(
                &view_proj,
                &transform.translation(),
                &transform.back(),
                shadow_map_config.far,
            );
        });
}

// Almost as the same as the one in bevy_render/src/visibility.rs
pub fn check_caster_visibility(
    mut thread_queues: Local<ThreadLocal<Cell<Vec<Entity>>>>,
    mut view_query: Query<
        (&mut VisibleEntities, &Frustum, Option<&RenderLayers>),
        With<PointLight2d>,
    >,
    mut visible_aabb_query: Query<
        (
            Entity,
            &InheritedVisibility,
            &mut ViewVisibility,
            Option<&RenderLayers>,
            Option<&Aabb>,
            &GlobalTransform,
            Has<NoFrustumCulling>,
        ),
        With<ShadowCaster2d>,
    >,
) {
    for (mut visible_entities, frustum, maybe_view_mask) in &mut view_query {
        let view_mask = maybe_view_mask.copied().unwrap_or_default();

        visible_entities.entities.clear();
        visible_aabb_query.par_iter_mut().for_each(|query_item| {
            let (
                entity,
                inherited_visibility,
                mut view_visibility,
                maybe_entity_mask,
                maybe_model_aabb,
                transform,
                no_frustum_culling,
            ) = query_item;

            // Skip computing visibility for entities that are configured to be hidden.
            // ViewVisibility has already been reset in `reset_view_visibility`.
            if !inherited_visibility.get() {
                return;
            }

            let entity_mask = maybe_entity_mask.copied().unwrap_or_default();
            if !view_mask.intersects(&entity_mask) {
                return;
            }

            // If we have an aabb, do frustum culling
            if !no_frustum_culling {
                if let Some(model_aabb) = maybe_model_aabb {
                    let model = transform.affine();
                    let model_sphere = Sphere {
                        center: model.transform_point3a(model_aabb.center),
                        radius: transform.radius_vec3a(model_aabb.half_extents),
                    };
                    // Do quick sphere-based frustum culling
                    if !frustum.intersects_sphere(&model_sphere, false) {
                        return;
                    }
                    // Do aabb-based frustum culling
                    if !frustum.intersects_obb(model_aabb, &model, true, false) {
                        return;
                    }
                }
            }

            view_visibility.set();
            let cell = thread_queues.get_or_default();
            let mut queue = cell.take();
            queue.push(entity);
            cell.set(queue);
        });

        for cell in &mut thread_queues {
            visible_entities.entities.append(cell.get_mut());
        }
    }
}
