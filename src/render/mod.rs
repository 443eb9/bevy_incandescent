use bevy::{
    app::{App, Plugin},
    asset::{load_internal_asset, Handle},
    core_pipeline::core_2d::graph::{Core2d, Node2d},
    ecs::schedule::IntoSystemConfigs,
    math::UVec3,
    render::{
        render_graph::RenderGraphApp, render_resource::Shader, view::RenderLayers, ExtractSchedule,
        Render, RenderApp, RenderSet,
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
        Shadow2dDebugDisplayPipeline, Shadow2dDistortPassPipeline, Shadow2dMainPassPipeline,
        Shadow2dPrepassPipeline, Shadow2dReductionPipeline,
    },
    resource::{GpuLights2d, ShadowMap2dConfig, ShadowMap2dStorage},
};

pub mod extract;
pub mod graph;
pub mod pipeline;
pub mod prepare;
pub mod resource;
pub mod visibility;

pub const DEFAULT_SHADOW_CASTER_LAYER: ShadowLayers = ShadowLayers(RenderLayers::layer(31));
pub const SHADOW_TYPES: Handle<Shader> = Handle::weak_from_u128(1123087897454135486384145234748455);
pub const SHADOW_DEBUG_DISPLAY_SHADER: Handle<Shader> = Handle::weak_from_u128(4518564135421563415);
pub const SHADOW_DISTORT_PASS_SHADER: Handle<Shader> = Handle::weak_from_u128(13745315343641643643);
pub const SHADOW_PREPASS_SHADER: Handle<Shader> = Handle::weak_from_u128(5321368413218521485631341);
pub const SHADOW_REDUCTION_PASS_SHADER: Handle<Shader> = Handle::weak_from_u128(485648964891315351);
pub const SHADOW_MAIN_PASS_SHADER: Handle<Shader> = Handle::weak_from_u128(13643651896413518964153);
pub const SHADOW_PREPASS_WORKGROUP_SIZE: UVec3 = UVec3 { x: 16, y: 16, z: 1 };

pub struct IncandescentRenderPlugin;

impl Plugin for IncandescentRenderPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            SHADOW_TYPES,
            "shaders/shadow_2d_types.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            SHADOW_DEBUG_DISPLAY_SHADER,
            "shaders/shadow_2d_debug_display.wgsl",
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

        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<ShadowMap2dConfig>()
            .init_resource::<GpuMetaBuffers>();

        render_app
            .add_systems(ExtractSchedule, extract::extract_point_lights)
            .add_systems(Render, prepare::prepare_lights.in_set(RenderSet::Prepare));

        render_app
            // .add_render_graph_node::<Shadow2dDebugDisplayPassNode>(
            //     Core2d,
            //     Shadow2dNode::Shadow2dDebugDisplayPass,
            // )
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
                    // Shadow2dNode::Shadow2dDebugDisplayPass,
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
            .init_resource::<Shadow2dDebugDisplayPipeline>()
            .init_resource::<Shadow2dPrepassPipeline>()
            .init_resource::<Shadow2dDistortPassPipeline>()
            .init_resource::<Shadow2dReductionPipeline>()
            .init_resource::<Shadow2dMainPassPipeline>()
            .init_resource::<GpuLights2d>()
            .init_resource::<ShadowMap2dStorage>();
    }
}
