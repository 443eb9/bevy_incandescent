use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, Query},
    },
    math::Mat4,
    render::{color::Color, Extract},
    transform::components::Transform,
};

use crate::ecs::light::PointLight2d;

#[derive(Component)]
pub struct ExtractedPointLight2d {
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
    pub radius: f32,
    pub transform: Mat4,
}

pub fn extract_point_lights(
    mut commands: Commands,
    lights_query: Extract<Query<(Entity, &PointLight2d, &Transform)>>,
) {
    commands.insert_or_spawn_batch(
        lights_query
            .iter()
            .map(|(entity, light, transform)| {
                (
                    entity,
                    ExtractedPointLight2d {
                        color: light.color,
                        intensity: light.intensity,
                        range: light.range,
                        radius: light.radius,
                        transform: transform.compute_matrix(),
                    },
                )
            })
            .collect::<Vec<_>>(),
    );
}
