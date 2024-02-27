use std::marker::PhantomData;

use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        query::With,
        system::{Commands, Query, Res, ResMut},
    },
    math::{Vec2, Vec3Swizzles, Vec4, Vec4Swizzles},
    render::{
        color::Color,
        render_resource::{
            Extent3d, ShaderType, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::{BevyDefault, ColorAttachment, TextureCache},
        view::{ExtractedView, Msaa, ViewTarget},
    },
    transform::components::GlobalTransform,
};

use crate::{
    ecs::{
        camera::{ShadowCamera, ShadowCameraDriver},
        light::{ShadowView2d, VisibleLight2dEntities},
    },
    render::resource::GpuShadowMapMeta,
};

use super::{
    extract::ExtractedPointLight2d,
    resource::{
        GpuLights2d, GpuMetaBuffers, GpuPointLight2d, ShadowMap2dConfig, ShadowMap2dMeta,
        ShadowMap2dStorage,
    },
};

#[derive(Component)]
pub struct DynamicUniformIndex<S: ShaderType> {
    index: u32,
    _marker: PhantomData<S>,
}

impl<S: ShaderType> DynamicUniformIndex<S> {
    pub fn new(index: u32) -> Self {
        Self {
            index,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn index(&self) -> u32 {
        self.index
    }
}

pub fn prepare_lights(
    mut commands: Commands,
    mut texture_cache: ResMut<TextureCache>,
    main_views: Query<Entity, With<ViewTarget>>,
    mut point_lights: Query<Entity, With<ExtractedPointLight2d>>,
    shadow_map_config: Res<ShadowMap2dConfig>,
    mut shadow_map_storage: ResMut<ShadowMap2dStorage>,
    mut gpu_meta_buffers: ResMut<GpuMetaBuffers>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    msaa: Res<Msaa>,
) {
    assert_eq!(*msaa, Msaa::Off, "MSAA is not supported yet!");

    let point_light_count = point_lights.iter().count();
    gpu_meta_buffers.clear();

    for (light_index, light_entity) in point_lights.iter_mut().enumerate() {
        let meta_index = gpu_meta_buffers.push_light_meta(GpuShadowMapMeta {
            index: light_index as u32,
            size: shadow_map_config.size,
        });

        let point_light_view_mesh_texture = texture_cache.get(
            &render_device,
            TextureDescriptor {
                label: Some("point_light_view_mesh_texture"),
                size: Extent3d {
                    width: shadow_map_config.size,
                    height: shadow_map_config.size,
                    depth_or_array_layers: 1,
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
                point_light_view_mesh_texture.clone(),
                None,
                Some(Color::NONE),
            ),
        };

        commands
            .entity(light_entity)
            .insert((meta_index, shadow_view));
    }

    shadow_map_storage.try_update(
        ShadowMap2dMeta {
            count: point_light_count as u32,
            size: shadow_map_config.size,
        },
        &render_device,
        &mut gpu_meta_buffers,
    );

    gpu_meta_buffers.write_buffers(&render_device, &render_queue);

    if let Some(shadow_camera) = main_views.iter().next() {
        // TODO add visible lights to all cameras
        commands.entity(shadow_camera).insert(ShadowCameraDriver);
    }
}

pub fn prepare_view_lights(
    mut commands: Commands,
    main_views: Query<
        (
            Entity,
            &ExtractedView,
            &VisibleLight2dEntities,
            &ShadowCamera,
        ),
        With<ViewTarget>,
    >,
    lights_query: Query<(&ExtractedPointLight2d, &GlobalTransform)>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    for (main_view_entity, main_view, visible_lights, shadow_camera) in &main_views {
        let mut buffer = GpuLights2d::new(&render_device);

        for visible_light in visible_lights.0.iter().copied() {
            let Ok((light, light_transform)) = lights_query.get(visible_light) else {
                continue;
            };

            let view_proj = main_view
                .view_projection
                .unwrap_or(main_view.projection * main_view.transform.compute_matrix().inverse());
            let position_ndc = (view_proj * light_transform.translation().extend(1.)).xy();
            let position_ws = light_transform.translation().xy();
            let min_position = position_ws - Vec2::splat(light.range);
            let max_position = position_ws + Vec2::splat(light.range);
            let min_ndc = (view_proj * min_position.extend(0.).extend(1.)).xy();
            let max_ndc = (view_proj * max_position.extend(0.).extend(1.)).xy();
            let range_ndc = max_ndc - min_ndc;

            buffer.add_point_light(GpuPointLight2d {
                position_ndc,
                range_ndc,
                radius_ndc: range_ndc * light.radius / light.range,
                color: light.color.rgba_to_vec4(),
            });

            // println!(
            //     "pos_ndc {:?} range_ndc {:?}",
            //     position_ndc,
            //     max_ndc - min_ndc
            // );
        }

        buffer.write_buffers(&render_device, &render_queue);
        commands.entity(main_view_entity).insert(buffer);
    }
}
