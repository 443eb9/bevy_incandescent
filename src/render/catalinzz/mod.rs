use bevy::{
    app::{App, Plugin, PostUpdate},
    asset::{load_internal_asset, Handle},
    core_pipeline::core_2d::graph::{Core2d, Node2d},
    ecs::{
        entity::Entity,
        query::With,
        schedule::IntoSystemConfigs,
        system::{Commands, Query, Res, ResMut},
    },
    math::{UVec3, UVec4},
    render::{
        camera::{camera_system, OrthographicProjection, PerspectiveProjection, Projection},
        color::Color,
        extract_resource::ExtractResourcePlugin,
        render_graph::RenderGraphApp,
        render_resource::{
            Extent3d, Shader, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::{BevyDefault, ColorAttachment, TextureCache},
        view::{ColorGrading, ExtractedView, Msaa, ViewTarget, VisibilitySystems},
        Extract, ExtractSchedule, Render, RenderApp, RenderSet,
    },
    transform::{components::GlobalTransform, TransformSystem},
};

use crate::{
    ecs::{
        catalinzz::{MainShadowCameraDriver, ShadowMap2dConfig, ShadowView2d},
        PointLight2d,
    },
    render::catalinzz::{
        graph::{
            Shadow2dDistortPassNode, Shadow2dMainPass, Shadow2dMeshPassNode, Shadow2dNode,
            Shadow2dPrepassNode, Shadow2dReductionNode,
        },
        resource::{GpuShadowMapMeta, ShadowMap2dMeta},
    },
};

use self::{
    pipeline::{
        Shadow2dDistortPassPipeline, Shadow2dMainPassPipeline, Shadow2dPrepassPipeline,
        Shadow2dReductionPipeline,
    },
    resource::{GpuMetaBuffers, ShadowMap2dStorage},
};

use bevy::render::view::visibility as bevy_visibility;

use super::ExtractedPointLight2d;

pub mod graph;
pub mod pipeline;
pub mod resource;
pub mod visibility;

pub const SHADOW_TYPES: Handle<Shader> = Handle::weak_from_u128(1123087897454135486384145234748455);
pub const SHADOW_DISTORT_PASS_SHADER: Handle<Shader> = Handle::weak_from_u128(13745315343641643643);
pub const SHADOW_PREPASS_SHADER: Handle<Shader> = Handle::weak_from_u128(5321368413218521485631341);
pub const SHADOW_REDUCTION_PASS_SHADER: Handle<Shader> = Handle::weak_from_u128(485648964891315351);
pub const SHADOW_MAIN_PASS_SHADER: Handle<Shader> = Handle::weak_from_u128(13643651896413518964153);
pub const SHADOW_PREPASS_WORKGROUP_SIZE: UVec3 = UVec3 { x: 16, y: 16, z: 1 };

#[cfg(feature = "compatibility")]
pub const SHADOW_MAP_FORMAT: TextureFormat = TextureFormat::Rgba32Float;
#[cfg(not(feature = "compatibility"))]
pub const SHADOW_MAP_FORMAT: TextureFormat = TextureFormat::Rg32Float;

pub struct CatalinzzApproachPlugin;

impl Plugin for CatalinzzApproachPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            SHADOW_TYPES,
            "shaders/shadow_2d_types.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            SHADOW_DISTORT_PASS_SHADER,
            "shaders/shadow_2d_distort_pass.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            SHADOW_PREPASS_SHADER,
            "shaders/shadow_2d_prepass.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            SHADOW_REDUCTION_PASS_SHADER,
            "shaders/shadow_2d_reduction_pass.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            SHADOW_MAIN_PASS_SHADER,
            "shaders/shadow_2d_main_pass.wgsl",
            Shader::from_wgsl
        );

        app.add_plugins(ExtractResourcePlugin::<ShadowMap2dConfig>::default())
            .init_resource::<ShadowMap2dConfig>()
            .add_systems(
                PostUpdate,
                (
                    visibility::update_light_frusta
                        .in_set(VisibilitySystems::UpdateOrthographicFrusta)
                        .after(camera_system::<OrthographicProjection>)
                        .after(TransformSystem::TransformPropagate)
                        .ambiguous_with(bevy_visibility::update_frusta::<PerspectiveProjection>)
                        .ambiguous_with(bevy_visibility::update_frusta::<Projection>),
                    visibility::check_caster_visibility
                        .in_set(VisibilitySystems::CheckVisibility)
                        .after(VisibilitySystems::CalculateBounds)
                        .after(VisibilitySystems::UpdateOrthographicFrusta)
                        .after(VisibilitySystems::UpdatePerspectiveFrusta)
                        .after(VisibilitySystems::UpdateProjectionFrusta)
                        .after(VisibilitySystems::VisibilityPropagate)
                        .after(TransformSystem::TransformPropagate)
                        .after(bevy_visibility::check_visibility),
                ),
            );

        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .init_resource::<GpuMetaBuffers>()
            .add_systems(ExtractSchedule, extract_light_view)
            .add_systems(Render, prepare_lights.in_set(RenderSet::Prepare))
            .add_render_graph_node::<Shadow2dMeshPassNode>(Core2d, Shadow2dNode::Shadow2dMeshPass)
            .add_render_graph_node::<Shadow2dPrepassNode>(Core2d, Shadow2dNode::Shadow2dPrepass)
            .add_render_graph_node::<Shadow2dDistortPassNode>(
                Core2d,
                Shadow2dNode::Shadow2dDistortPass,
            )
            .add_render_graph_node::<Shadow2dReductionNode>(
                Core2d,
                Shadow2dNode::Shadow2dReductionPass,
            )
            .add_render_graph_node::<Shadow2dMainPass>(Core2d, Shadow2dNode::Shadow2dMainPass)
            .add_render_graph_edges(
                Core2d,
                (
                    Node2d::MainPass,
                    Shadow2dNode::Shadow2dMeshPass,
                    Shadow2dNode::Shadow2dPrepass,
                    Shadow2dNode::Shadow2dDistortPass,
                    Shadow2dNode::Shadow2dReductionPass,
                    Shadow2dNode::Shadow2dMainPass,
                    Node2d::Bloom,
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .init_resource::<Shadow2dPrepassPipeline>()
            .init_resource::<Shadow2dDistortPassPipeline>()
            .init_resource::<Shadow2dReductionPipeline>()
            .init_resource::<Shadow2dMainPassPipeline>()
            .init_resource::<ShadowMap2dStorage>();
    }
}

pub fn extract_light_view(
    mut commands: Commands,
    lights_query: Extract<Query<(Entity, &PointLight2d, &GlobalTransform)>>,
    shadow_map_config: Extract<Res<ShadowMap2dConfig>>,
) {
    commands.insert_or_spawn_batch(
        lights_query
            .iter()
            .map(|(entity, light, transform)| {
                let transform = GlobalTransform::from_translation(transform.translation());
                (
                    entity,
                    ExtractedView {
                        projection: shadow_map_config.get_proj_mat(light.range * 2.),
                        transform,
                        view_projection: None,
                        hdr: false,
                        viewport: UVec4::ZERO,
                        color_grading: ColorGrading::default(),
                    },
                )
            })
            .collect::<Vec<_>>(),
    );
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
    if point_light_count == 0 {
        return;
    }

    gpu_meta_buffers.clear();

    for (light_index, light_entity) in point_lights.iter_mut().enumerate() {
        // TODO support different pcf settings for different lights
        let meta_index = gpu_meta_buffers.push_light_meta(GpuShadowMapMeta {
            index: light_index as u32,
            size: shadow_map_config.size,
            offset: shadow_map_config.offset,
            bias: shadow_map_config.bias,
            alpha_threshold: shadow_map_config.alpha_threshold,
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

    gpu_meta_buffers.write_buffers(&render_device, &render_queue);

    shadow_map_storage.try_update(
        ShadowMap2dMeta {
            count: point_light_count as u32,
            size: shadow_map_config.size,
        },
        &render_device,
        &mut gpu_meta_buffers,
    );

    if let Some(shadow_camera) = main_views.iter().next() {
        commands
            .entity(shadow_camera)
            .insert(MainShadowCameraDriver);
    }
}
