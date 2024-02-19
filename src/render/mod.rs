use bevy::{
    app::{App, Plugin},
    core_pipeline::core_2d::graph::{Core2d, Node2d},
    render::{
        render_graph::{RenderGraphApp, RenderLabel},
        ExtractSchedule, RenderApp,
    },
};

use self::graph::IncandescentPrePassNode;

pub mod extract;
pub mod graph;
pub mod pipeline;
pub mod resource;

pub struct IncandescentRenderPlugin;

impl Plugin for IncandescentRenderPlugin {
    fn build(&self, app: &mut App) {
        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.add_systems(ExtractSchedule, extract::extract_point_lights);

        render_app
            .add_render_graph_node::<IncandescentPrePassNode>(
                Core2d,
                IncandescentNode::ShadowPrePass,
            )
            .add_render_graph_edges(
                Core2d,
                (
                    Node2d::MainPass,
                    IncandescentNode::ShadowPrePass,
                    Node2d::Bloom,
                ),
            )
            .add_render_graph_edges(
                Core2d,
                (
                    Node2d::MainPass,
                    IncandescentNode::ShadowPrePass,
                    Node2d::Tonemapping,
                ),
            );
    }
}

#[derive(RenderLabel, Debug, Hash, PartialEq, Eq, Clone)]
pub enum IncandescentNode {
    ShadowPrePass,
}
