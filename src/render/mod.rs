use bevy::{
    app::{App, Plugin},
    asset::{load_internal_asset, Handle},
    core_pipeline::core_2d::graph::{Core2d, Node2d},
    ecs::schedule::IntoSystemConfigs,
    render::{
        render_graph::RenderGraphApp, render_resource::Shader, view::RenderLayers, ExtractSchedule,
        Render, RenderApp, RenderSet,
    },
};

use crate::ecs::light::ShadowLayers;

use self::{
    graph::{Shadow2dMeshPassNode, Shadow2dNode, Shadow2dPrepassNode},
    pipeline::Shadow2dPrepassPipeline,
    resource::{GpuLights2d, ShadowMap2dConfig},
};

pub mod draw;
pub mod extract;
pub mod graph;
pub mod pipeline;
pub mod prepare;
pub mod resource;
pub mod visibility;

pub const DEFAULT_SHADOW_CASTER_LAYER: ShadowLayers = ShadowLayers(RenderLayers::layer(31));
pub const SHADOW_PREPASS_SHADER: Handle<Shader> = Handle::weak_from_u128(532136841321852148563134);
pub const SHADOW_MAIN_PASS_SHADER: Handle<Shader> = Handle::weak_from_u128(13643651896413518964153);

pub struct IncandescentRenderPlugin;

impl Plugin for IncandescentRenderPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            SHADOW_PREPASS_SHADER,
            "shaders/shadow2d_prepass.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            SHADOW_MAIN_PASS_SHADER,
            "shaders/shadow2d_main_pass.wgsl",
            Shader::from_wgsl
        );

        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<ShadowMap2dConfig>()
            .init_resource::<GpuLights2d>();

        render_app
            .add_systems(ExtractSchedule, extract::extract_point_lights)
            .add_systems(Render, prepare::prepare_lights.in_set(RenderSet::Prepare));

        render_app
            .add_render_graph_node::<Shadow2dMeshPassNode>(Core2d, Shadow2dNode::Shadow2dMeshPass)
            .add_render_graph_node::<Shadow2dPrepassNode>(Core2d, Shadow2dNode::Shadow2dPrepass)
            .add_render_graph_node::<Shadow2dPrepassNode>(
                Core2d,
                Shadow2dNode::Shadow2dReductionPass,
            )
            .add_render_graph_node::<Shadow2dPrepassNode>(Core2d, Shadow2dNode::Shadow2dMainPass)
            .add_render_graph_edges(
                Core2d,
                (
                    Node2d::MainPass,
                    Shadow2dNode::Shadow2dMeshPass,
                    Shadow2dNode::Shadow2dPrepass,
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

        render_app.init_resource::<Shadow2dPrepassPipeline>();
    }
}
