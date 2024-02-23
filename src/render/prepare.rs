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
        texture::{BevyDefault, ColorAttachment, TextureCache},
        view::{ExtractedView, ViewTarget, VisibleEntities},
    },
    transform::components::GlobalTransform,
};

use crate::ecs::light::{ShadowView2d, VisibleLight2dEntities};

use super::{
    extract::ExtractedPointLight2d,
    resource::{GpuLights2d, GpuPointLight2d, ShadowMap2dConfig, ShadowView2dUniform},
};

pub fn prepare_lights(
    mut commands: Commands,
    mut texture_cache: ResMut<TextureCache>,
    main_views: Query<Entity, With<ViewTarget>>,
    mut point_lights: Query<(Entity, &ExtractedPointLight2d, &GlobalTransform)>,
    shadow_map_config: Res<ShadowMap2dConfig>,
    mut gpu_lights: ResMut<GpuLights2d>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    let point_light_count = point_lights.iter().count();
    gpu_lights.clear();

    for (light_entity, light, transform) in point_lights.iter_mut() {
        let uniform_indices = gpu_lights.add_point_light(
            ShadowView2dUniform::new(transform, &shadow_map_config),
            GpuPointLight2d::new(transform, light),
        );

        let point_light_view_mesh_texture = texture_cache.get(
            &render_device,
            TextureDescriptor {
                label: Some("point_light_view_mesh_texture"),
                size: Extent3d {
                    width: shadow_map_config.size as u32,
                    height: shadow_map_config.size as u32,
                    depth_or_array_layers: (point_light_count as u32).max(1),
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::bevy_default(),
                usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
        );
        let shadow_view = ShadowView2d {
            attachment: ColorAttachment::new(
                point_light_view_mesh_texture,
                None,
                Some(Color::NONE),
            ),
        };

        commands.entity(light_entity).insert((
            uniform_indices,
            shadow_view,
            RenderPhase::<Transparent2d>::default(),
        ));
    }

    let lights = point_lights.iter().map(|(e, ..)| e).collect::<Vec<_>>();
    main_views.iter().for_each(|main_view_entity| {
        commands
            .entity(main_view_entity)
            .insert(VisibleLight2dEntities(lights.clone()));
    });

    gpu_lights.write_buffers(&render_device, &render_queue);
}
