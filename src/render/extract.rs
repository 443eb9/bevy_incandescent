use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, Query},
    },
    render::{camera::OrthographicProjection, color::Color, view::RenderLayers, Extract},
    transform::components::GlobalTransform,
};

use crate::ecs::light::PointLight2d;

use super::DEFAULT_SHADOW_CASTER_LAYER;

#[derive(Component, Clone, Copy)]
pub struct ExtractedPointLight2d {
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
    pub radius: f32,
    pub spot_light_angles: Option<(f32, f32)>,
}

pub fn extract_camera_projections(
    mut commands: Commands,
    cameras_query: Extract<Query<(Entity, &OrthographicProjection)>>,
) {
    commands.insert_or_spawn_batch(
        cameras_query
            .iter()
            .map(|(entity, projection)| (entity, projection.clone()))
            .collect::<Vec<_>>(),
    );
}

pub fn extract_point_lights(
    mut commands: Commands,
    lights_query: Extract<
        Query<(
            Entity,
            &PointLight2d,
            &GlobalTransform,
            Option<&RenderLayers>,
        )>,
    >,
) {
    commands.insert_or_spawn_batch(
        lights_query
            .iter()
            .map(|(entity, light, transform, render_layers)| {
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
                        *render_layers.unwrap_or(&DEFAULT_SHADOW_CASTER_LAYER),
                    ),
                )
            })
            .collect::<Vec<_>>(),
    );
}
