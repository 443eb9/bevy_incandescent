use bevy::{
    ecs::{query::QueryState, system::lifetimeless::Read, world::World},
    math::UVec3,
    render::{
        render_graph::{Node, NodeRunError, RenderGraphContext, RenderLabel},
        render_resource::{
            BindGroupEntries, ComputePassDescriptor, Operations, PipelineCache,
            RenderPassColorAttachment, RenderPassDescriptor,
        },
        renderer::{RenderContext, RenderDevice},
        view::ViewTarget,
    },
};

use super::{
    pipeline::{Shadow2dJfaPrepassPipeline, Shadow2dMainPassPipeline},
    JfaTextureStorage, SHADOW_JFA_PREPASS_WORK_GROUP_SIZE,
};

#[derive(RenderLabel, Debug, Hash, PartialEq, Eq, Clone)]
pub enum Shadow2dNode {
    Shadow2dJfaPrepass,
    Shadow2dMainPass,
}

pub struct Shadow2dJfaPrepassNode {
    main_view_query: QueryState<Read<ViewTarget>>,
}

impl Node for Shadow2dJfaPrepassNode {
    fn update(&mut self, world: &mut World) {
        self.main_view_query.update_archetypes(world);
    }

    fn run<'w>(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let Ok(view_target) = self.main_view_query.get_manual(world, graph.view_entity()) else {
            return Ok(());
        };

        let pipeline = world.resource::<Shadow2dJfaPrepassPipeline>();
        let Some(compute_pipeline) = world
            .resource::<PipelineCache>()
            .get_compute_pipeline(pipeline.cached_id)
        else {
            return Ok(());
        };

        let main_texture_view = view_target.main_texture_view();
        let jfa_textures = world.resource::<JfaTextureStorage>();

        let (jfa_texture, texture_size) = jfa_textures.get_jfa_texture(graph.view_entity());
        let bind_group = render_context.render_device().create_bind_group(
            "shadow_2d_jfa_pass_bind_group",
            &pipeline.jfa_pass_layout,
            &BindGroupEntries::sequential((
                main_texture_view,
                &pipeline.main_texture_sampler,
                jfa_texture,
            )),
        );

        let work_group_count = UVec3 {
            x: texture_size
                .x
                .div_ceil(SHADOW_JFA_PREPASS_WORK_GROUP_SIZE.x),
            y: texture_size
                .y
                .div_ceil(SHADOW_JFA_PREPASS_WORK_GROUP_SIZE.y),
            z: SHADOW_JFA_PREPASS_WORK_GROUP_SIZE.z,
        };

        let mut compute_pass =
            render_context
                .command_encoder()
                .begin_compute_pass(&ComputePassDescriptor {
                    label: Some("shadow_2d_jfa_prepass_pass"),
                    timestamp_writes: None,
                });

        compute_pass.set_pipeline(compute_pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);
        compute_pass.dispatch_workgroups(
            work_group_count.x,
            work_group_count.y,
            work_group_count.z,
        );

        Ok(())
    }
}

pub struct Shadow2dMainPassNode {
    main_view_query: QueryState<Read<ViewTarget>>,
}

impl Node for Shadow2dMainPassNode {
    fn update(&mut self, world: &mut World) {
        self.main_view_query.update_archetypes(world);
    }

    fn run<'w>(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let Ok(view_target) = self.main_view_query.get_manual(world, graph.view_entity()) else {
            return Ok(());
        };

        let pipeline = world.resource::<Shadow2dMainPassPipeline>();
        let Some(render_pipeline) = world
            .resource::<PipelineCache>()
            .get_render_pipeline(pipeline.cached_id)
        else {
            return Ok(());
        };

        let jfa_textures = world.resource::<JfaTextureStorage>();
        let post_process = view_target.post_process_write();

        let (jfa_texture, _) = jfa_textures.get_jfa_texture(graph.view_entity());
        let bind_group = render_context.render_device().create_bind_group(
            "shadow_2d_main_pass_bind_group",
            &pipeline.main_pass_layout,
            &BindGroupEntries::sequential((jfa_texture,)),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("shadow_2d_main_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &post_process.source,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_render_pipeline(render_pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}
