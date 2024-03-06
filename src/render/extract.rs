use bevy::{
    core_pipeline::core_2d::Transparent2d,
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, Query, Res},
    },
    math::UVec4,
    render::{
        color::Color,
        render_phase::RenderPhase,
        view::{ColorGrading, ExtractedView, VisibleEntities},
        Extract,
    },
    transform::components::GlobalTransform,
};

use crate::ecs::{light::PointLight2d, resources::{AmbientLight2d, ShadowMap2dConfig}};

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
    lights_query: Extract<Query<(Entity, &PointLight2d, &GlobalTransform, &VisibleEntities)>>,
    shadow_map_config: Extract<Res<ShadowMap2dConfig>>,
) {
    commands.insert_or_spawn_batch(
        lights_query
            .iter()
            .map(|(entity, light, transform, visible_entities)| {
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
                        visible_entities.clone(),
                        ExtractedView {
                            // TODO I have no idea why the size should be doubled
                            //      The shadow map will be clipped if the size is not doubled
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

pub fn extract_resources(
    mut commands: Commands,
    shadow_map_config: Extract<Res<ShadowMap2dConfig>>,
    ambient_light: Extract<Res<AmbientLight2d>>,
) {
    commands.insert_resource(shadow_map_config.clone());
    commands.insert_resource(ambient_light.clone());
}
