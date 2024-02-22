use bevy::{
    core_pipeline::core_2d::Transparent2d,
    ecs::{
        entity::Entity,
        query::With,
        system::{Commands, Query, Res, ResMut},
    },
    render::{
        camera::OrthographicProjection,
        color::Color,
        render_phase::RenderPhase,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::{ColorAttachment, TextureCache},
        view::{ExtractedView, VisibleEntities},
    },
    transform::components::GlobalTransform,
};

use crate::ecs::light::ViewLight2dEntities;

use super::{
    extract::ExtractedPointLight2d,
    resource::{GpuLights2d, GpuPointLight2d, PointLight2dShadowMap},
};

pub fn prepare_lights(
    mut commands: Commands,
    mut texture_cache: ResMut<TextureCache>,
    main_views: Query<
        (Entity, &ExtractedView, &OrthographicProjection),
        With<RenderPhase<Transparent2d>>,
    >,
    point_lights: Query<(
        Entity,
        &ExtractedPointLight2d,
        &GlobalTransform,
        &VisibleEntities,
    )>,
    point_light_shadow_map: Res<PointLight2dShadowMap>,
    mut gpu_lights: ResMut<GpuLights2d>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    let point_light_count = point_lights.iter().count();
    gpu_lights.clear();

    for (_, light, transform, _) in point_lights.iter() {
        gpu_lights.add_point_light(GpuPointLight2d::new(transform, light));
    }

    for (main_view_entity, main_view, main_view_proj) in &main_views {
        let mut view_lights = Vec::new();
        for (light_index, (light_entity, light, light_transform, visible_entities)) in
            point_lights.iter().enumerate()
        {
            let point_light_view_obstacle_texture = texture_cache.get(
                &render_device,
                TextureDescriptor {
                    label: Some("point_light_view_obstacle_texture"),
                    size: Extent3d {
                        width: point_light_shadow_map.size as u32,
                        height: point_light_shadow_map.size as u32,
                        depth_or_array_layers: (point_light_count as u32).max(1),
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: TextureDimension::D2,
                    format: TextureFormat::Depth32Float,
                    usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                },
            );

            view_lights.push(light_entity);
        }

        commands
            .entity(main_view_entity)
            .insert(ViewLight2dEntities(view_lights));
    }

    gpu_lights.write_buffers(&render_device, &render_queue);
}
