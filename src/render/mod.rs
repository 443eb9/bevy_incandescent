use bevy::{
    app::{App, Plugin, PostUpdate},
    asset::{load_internal_asset, Handle},
    core_pipeline::core_2d::graph::{Core2d, Node2d},
    ecs::schedule::IntoSystemConfigs,
    math::UVec3,
    render::{
        render_graph::RenderGraphApp,
        render_resource::{Shader, TextureFormat},
        view::{RenderLayers, VisibilitySystems},
        ExtractSchedule, Render, RenderApp, RenderSet,
    },
};

use crate::{
    ecs::light::ShadowLayers,
    render::{
        graph::{Shadow2dMainPass, Shadow2dPrepassNode, Shadow2dReductionNode},
        resource::GpuMetaBuffers,
    },
};

use self::{
    graph::{Shadow2dDistortPassNode, Shadow2dMeshPassNode, Shadow2dNode},
    pipeline::{
        Shadow2dDistortPassPipeline, Shadow2dMainPassPipeline, Shadow2dPrepassPipeline,
        Shadow2dReductionPipeline,
    },
    resource::ShadowMap2dStorage,
};

pub mod extract;
pub mod graph;
pub mod pipeline;
pub mod prepare;
pub mod resource;
pub mod visibility;

pub const DEFAULT_SHADOW_CASTER_LAYER: ShadowLayers = ShadowLayers(RenderLayers::layer(31));
pub const HASH_SHADER: Handle<Shader> = Handle::weak_from_u128(9489746513229684156489);
pub const LIGHTING_SHADER: Handle<Shader> = Handle::weak_from_u128(1351654315646451321546531153891);
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

pub struct IncandescentRenderPlugin;

impl Plugin for IncandescentRenderPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(app, HASH_SHADER, "shaders/hash.wgsl", Shader::from_wgsl);

        load_internal_asset!(
            app,
            LIGHTING_SHADER,
            "shaders/lighting.wgsl",
            Shader::from_wgsl
        );

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

        app.add_systems(
            PostUpdate,
            (
                visibility::calc_light_bounds.in_set(VisibilitySystems::CalculateBounds),
                visibility::update_light_frusta.in_set(VisibilitySystems::UpdateOrthographicFrusta),
                visibility::check_caster_visibility.in_set(VisibilitySystems::CheckVisibility),
            ),
        );

        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.init_resource::<GpuMetaBuffers>();

        render_app
            .add_systems(
                ExtractSchedule,
                (extract::extract_point_lights, extract::extract_resources),
            )
            .add_systems(
                Render,
                (prepare::prepare_lights, prepare::prepare_view_lights).in_set(RenderSet::Prepare),
            );

        render_app
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
        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<Shadow2dPrepassPipeline>()
            .init_resource::<Shadow2dDistortPassPipeline>()
            .init_resource::<Shadow2dReductionPipeline>()
            .init_resource::<Shadow2dMainPassPipeline>()
            .init_resource::<ShadowMap2dStorage>();
    }
}
