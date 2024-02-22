use std::ops::Range;

use bevy::{
    app::{App, Plugin},
    asset::Handle,
    core_pipeline::core_2d::graph::{Core2d, Node2d},
    ecs::{entity::Entity, schedule::IntoSystemConfigs},
    render::{
        render_graph::{RenderGraphApp, RenderLabel},
        render_phase::{CachedRenderPipelinePhaseItem, DrawFunctionId, PhaseItem},
        render_resource::{CachedRenderPipelineId, Shader},
        view::RenderLayers,
        ExtractSchedule, Render, RenderApp, RenderSet,
    },
    utils::nonmax::NonMaxU32,
};

use self::{graph::Shadow2dPrepassNode, resource::PointLight2dShadowMap};

pub mod extract;
pub mod graph;
pub mod pipeline;
pub mod prepare;
pub mod resource;

pub const DEFAULT_SHADOW_CASTER_LAYER: RenderLayers = RenderLayers::layer(31);
pub const SHADOW_PREPASS_SHADER: Handle<Shader> = Handle::weak_from_u128(532136841321852148563134);

pub struct IncandescentRenderPlugin;

impl Plugin for IncandescentRenderPlugin {
    fn build(&self, app: &mut App) {
        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.init_resource::<PointLight2dShadowMap>();

        render_app
            .add_systems(
                ExtractSchedule,
                (
                    extract::extract_camera_projections,
                    extract::extract_point_lights,
                ),
            )
            .add_systems(Render, prepare::prepare_lights.in_set(RenderSet::Prepare));

        render_app
            .add_render_graph_node::<Shadow2dPrepassNode>(Core2d, IncandescentNode::Shadow2dPrepass)
            .add_render_graph_edges(
                Core2d,
                (
                    Node2d::MainPass,
                    IncandescentNode::Shadow2dPrepass,
                    Node2d::Bloom,
                ),
            );
    }
}

#[derive(RenderLabel, Debug, Hash, PartialEq, Eq, Clone)]
pub enum IncandescentNode {
    Shadow2dPrepass,
}
