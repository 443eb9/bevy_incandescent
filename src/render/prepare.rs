use std::marker::PhantomData;

use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        query::With,
        system::{Commands, Query, Res, ResMut},
    },
    math::{Vec2, Vec3, Vec4Swizzles},
    render::{
        color::Color,
        render_resource::{
            Extent3d, ShaderType, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::{BevyDefault, ColorAttachment, TextureCache},
        view::{ExtractedView, Msaa, ViewTarget, VisibleEntities},
    },
    transform::components::GlobalTransform,
};

use crate::{
    ecs::{camera::MainShadowCameraDriver, light::ShadowView2d, resources::ShadowMap2dConfig},
    render::resource::GpuShadowMapMeta,
};

use super::{
    extract::ExtractedPointLight2d,
    resource::{GpuLights2d, GpuMetaBuffers, GpuPointLight2d, ShadowMap2dMeta, ShadowMap2dStorage},
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
    assert!(
        shadow_map_config.pcf_samples <= 32,
        "PCF samples must be less than 32!"
    );

    let point_light_count = point_lights.iter().count();
    gpu_meta_buffers.clear();

    for (light_index, light_entity) in point_lights.iter_mut().enumerate() {
        // TODO support different pcf settings for different lights
        let meta_index = gpu_meta_buffers.push_light_meta(GpuShadowMapMeta {
            index: light_index as u32,
            size: shadow_map_config.size,
            offset: shadow_map_config.offset,
            bias: shadow_map_config.bias,
            pcf_samples: shadow_map_config.pcf_samples,
            pcf_radius: shadow_map_config.pcf_radius,
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
        commands.entity(shadow_camera).insert(MainShadowCameraDriver);
    }
}

pub fn prepare_view_lights(
    mut commands: Commands,
    main_views: Query<(Entity, &ExtractedView, &VisibleEntities), With<ViewTarget>>,
    lights_query: Query<(&ExtractedPointLight2d, &GlobalTransform)>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    for (main_view_entity, main_view, visible_entities) in &main_views {
        let mut buffer = GpuLights2d::new(&render_device);

        let main_view_pos_ws = main_view.transform.translation();
        let view_proj = main_view.view_projection.unwrap_or_else(|| {
            main_view.projection * main_view.transform.compute_matrix().inverse()
        });

        for visible_light in visible_entities.entities.iter().copied() {
            let Ok((light, light_transform)) = lights_query.get(visible_light) else {
                continue;
            };

            let position_ws = light_transform.translation().extend(1.);
            let screen_size = 2.
                / Vec2::new(
                    main_view.projection.x_axis[0],
                    main_view.projection.y_axis[1],
                );

            let mut position_ndc = (view_proj * position_ws).xy();
            position_ndc.y = -position_ndc.y;
            let range_ndc =
                view_proj * (Vec3::new(light.range, 0., 0.) + main_view_pos_ws).extend(1.);

            let range_ndc = range_ndc.x / range_ndc.w / 2.;
            let radius_ndc = light.radius / light.range * range_ndc;

            buffer.add_point_light(GpuPointLight2d {
                intensity: light.intensity,
                position_ss: (position_ndc + 1.) / 2. * screen_size,
                radius_ss: radius_ndc * screen_size.x,
                range_ss: range_ndc * screen_size.x,
                color: light.color.rgba_linear_to_vec4(),
            });
        }

        buffer.write_buffers(&render_device, &render_queue);
        commands.entity(main_view_entity).insert(buffer);
    }
}
