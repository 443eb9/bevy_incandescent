use bevy::{
    core_pipeline::core_2d::Transparent2d,
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, Query},
    },
    math::{Mat4, UVec4},
    render::{
        color::Color,
        render_phase::RenderPhase,
        view::{ColorGrading, ExtractedView, VisibleEntities},
        Extract,
    },
    transform::components::GlobalTransform,
};

use crate::ecs::light::{PointLight2d, ShadowCaster2dVisibility};

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
                        // This is not important for light rendering
                        // but without it, 2d meshes won't think it's a camera
                        // then they won't queue themselves to the phase
                        ExtractedView {
                            projection: Mat4::IDENTITY,
                            transform: GlobalTransform::IDENTITY,
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
