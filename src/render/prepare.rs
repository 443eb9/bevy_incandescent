use std::marker::PhantomData;

use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        query::With,
        system::{Commands, Query, Res, ResMut},
    },
    render::{
        color::Color,
        render_resource::{
            Extent3d, ShaderType, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::{BevyDefault, ColorAttachment, TextureCache},
        view::{Msaa, ViewTarget},
    },
    transform::components::GlobalTransform,
};

use crate::{
    ecs::{
        camera::ShadowCameraDriver,
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
    mut point_lights: Query<(Entity, &ExtractedPointLight2d, &GlobalTransform)>,
    shadow_map_config: Res<ShadowMap2dConfig>,
    mut shadow_map_storage: ResMut<ShadowMap2dStorage>,
    mut gpu_meta_buffers: ResMut<GpuMetaBuffers>,
    mut gpu_lights: ResMut<GpuLights2d>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    msaa: Res<Msaa>,
) {
    assert_eq!(*msaa, Msaa::Off, "MSAA is not supported yet!");

    let point_light_count = point_lights.iter().count();
    gpu_lights.clear();
    gpu_meta_buffers.clear();

    for (light_index, (light_entity, light, transform)) in point_lights.iter_mut().enumerate() {
        let uniform_indices = gpu_lights.add_point_light(GpuPointLight2d::new(transform, light));

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
            .insert((uniform_indices, meta_index, shadow_view));
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
    gpu_lights.write_buffers(&render_device, &render_queue);

    if let Some(shadow_camera) = main_views.iter().next() {
        // TODO add visible lights to all cameras
        commands.entity(shadow_camera).insert((
            VisibleLight2dEntities(point_lights.iter().map(|(e, ..)| e).collect::<Vec<_>>()),
            ShadowCameraDriver,
        ));
    }
}
