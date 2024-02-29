use bevy::{
    core_pipeline::core_2d::Transparent2d,
    ecs::{
        component::Component,
        entity::Entity,
        query::With,
        system::{Commands, Query, Res},
    },
    math::UVec4,
    render::{
        camera::{Camera, OrthographicProjection},
        color::Color,
        render_phase::RenderPhase,
        view::{ColorGrading, ExtractedView, VisibleEntities},
        Extract,
    },
    transform::components::GlobalTransform,
};

use crate::ecs::light::{PointLight2d, ShadowCaster2dVisibility, VisibleLight2dEntities};

use super::resource::ShadowMap2dConfig;

#[derive(Component, Clone, Copy)]
pub struct ExtractedPointLight2d {
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
    pub radius: f32,
    pub spot_light_angles: Option<(f32, f32)>,
}

pub fn extract_point_lights(
    mut commands: Commands,
    lights_query: Extract<Query<(Entity, &PointLight2d, &GlobalTransform)>>,
    caster_query: Extract<Query<(Entity, &ShadowCaster2dVisibility)>>,
    shadow_map_config: Res<ShadowMap2dConfig>,
) {
    let casters = caster_query
        .iter()
        .filter_map(|(e, v)| if v.0 { Some(e) } else { None })
        .collect::<Vec<_>>();
    commands.insert_or_spawn_batch(
        lights_query
            .iter()
            .map(|(entity, light, transform)| {
                (
                    entity,
                    (
                        ExtractedPointLight2d {
                            color: light.color,
                            intensity: light.intensity,
                            range: light.range,
                            radius: light.radius,
                            spot_light_angles: None,
                        },
                        *transform,
                        // TODO cull invisible casters
                        VisibleEntities {
                            entities: casters.clone(),
                        },
                        ExtractedView {
                            projection: shadow_map_config.get_proj_mat(light.range * 2.),
                            transform: *transform,
                            view_projection: None,
                            hdr: false,
                            viewport: UVec4::ZERO,
                            color_grading: ColorGrading::default(),
                        },
                        RenderPhase::<Transparent2d>::default(),
                    ),
                )
            })
            .collect::<Vec<_>>(),
    );
}

pub fn extract_shadow_cameras(
    mut commands: Commands,
    main_views: Extract<Query<Entity, With<Camera>>>,
    // TODO visibility check
    lights_query: Extract<Query<Entity, With<PointLight2d>>>,
) {
    for main_view_entity in &main_views {
        commands
            .get_or_spawn(main_view_entity)
            .insert(VisibleLight2dEntities(lights_query.iter().collect()));
    }
}
