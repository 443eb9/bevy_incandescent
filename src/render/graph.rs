use bevy::{
    ecs::{
        entity::Entity,
        query::{QueryState, With},
        system::lifetimeless::Read,
        world::{FromWorld, World},
    },
    render::{
        extract_component::DynamicUniformIndex,
        render_graph::{Node, NodeRunError, RenderGraphContext},
        render_phase::{RenderPhase, TrackedRenderPass},
        render_resource::{
            BindGroupEntries, CommandEncoderDescriptor, PipelineCache, RenderPassDescriptor,
        },
        renderer::RenderContext,
        view::ViewTarget,
    },
};

use crate::ecs::light::{ShadowView2d, ViewLight2dEntities};

use super::{
    extract::ExtractedPointLight2d, pipeline::Shadow2dPrepassPipeline, resource::GpuLights2d,
};

use bevy::render::render_resource::binding_types as binding;

pub struct Shadow2dPrepassNode {
    main_view_query: QueryState<(Read<ViewTarget>, Read<ViewLight2dEntities>)>,
    view_light_query: QueryState<(
        Read<DynamicUniformIndex<ExtractedPointLight2d>>,
        Read<ShadowView2d>,
    )>,
}

impl FromWorld for Shadow2dPrepassNode {
    fn from_world(world: &mut World) -> Self {
        Self {
            main_view_query: world.query_filtered(),
            view_light_query: world.query_filtered(),
        }
    }
}

impl Node for Shadow2dPrepassNode {
    fn update(&mut self, world: &mut World) {
        self.main_view_query.update_archetypes(world);
        self.view_light_query.update_archetypes(world);
    }

    fn run<'w>(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let view_entity = graph.view_entity();
        let Ok((view_target, view_lights)) = self.main_view_query.get_manual(world, view_entity)
        else {
            return Ok(());
        };
        let post_process = view_target.post_process_write();
        let pipeline = world.resource::<Shadow2dPrepassPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let Some(render_pipeline) = pipeline_cache.get_render_pipeline(pipeline.cached_id) else {
            return Ok(());
        };
        let lights_data = world.resource::<GpuLights2d>();

        for view_light_entity in view_lights.0.iter().copied() {
            let (light_index, shadow_view) = self
                .view_light_query
                .get_manual(world, view_light_entity)
                .unwrap();

            // let color_attachment = shadow_view.attachment.get_attachment();
            let color_attachment = bevy::render::render_resource::RenderPassColorAttachment {
                view: &post_process.destination,
                resolve_target: None,
                ops: bevy::render::render_resource::Operations::default(),
            };

            let shadow2d_prepass_bind_group = render_context.render_device().create_bind_group(
                "shadow2d_prepass_bind_group",
                &pipeline.prepass_layout,
                &BindGroupEntries::sequential((
                    post_process.source,
                    &pipeline.shadow_sampler,
                    lights_data.point_lights_binding(),
                )),
            );

            let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some("shadow2d_prepass_render_pass"),
                color_attachments: &[Some(color_attachment)],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render_pass.set_render_pipeline(render_pipeline);
            render_pass.set_bind_group(0, &shadow2d_prepass_bind_group, &[light_index.index()]);
            render_pass.draw(0..3, 0..1);
            println!("Shadow2dPrepassNode::run");
        }

        Ok(())
    }
}
