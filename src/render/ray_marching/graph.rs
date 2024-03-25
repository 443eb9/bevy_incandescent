use bevy::{
    core_pipeline::core_2d::Transparent2d,
    ecs::{
        query::QueryState,
        system::lifetimeless::Read,
        world::{FromWorld, World},
    },
    math::UVec3,
    render::{
        render_graph::{Node, NodeRunError, RenderGraphContext, RenderLabel},
        render_phase::RenderPhase,
        render_resource::{
            BindGroupEntries, ComputePassDescriptor, Operations, PipelineCache,
            RenderPassColorAttachment, RenderPassDescriptor,
        },
        renderer::RenderContext,
        view::{ViewTarget, ViewUniformOffset, ViewUniforms},
    },
};

use crate::{
    ecs::ShadowView2d,
    render::{
        light::{GpuAmbientLight2dBuffer, GpuLights2d},
        universal_buffers::NumberBuffer,
        DynamicUniformIndex,
    },
};

use super::{
    pipeline::{
        Shadow2dJfaPassPipeline, Shadow2dJfaPrepassPipeline, Shadow2dMainPassPipeline,
        Shadow2dSdfPassPipeline,
    },
    GpuMetaBuffers, SdfMeta, SdfTextureStorage, SHADOW_WORK_GROUP_SIZE,
};

#[derive(RenderLabel, Debug, Hash, PartialEq, Eq, Clone)]
pub enum Shadow2dNode {
    Shadow2dMeshPass,
    Shadow2dJfaPrepass,
    Shadow2dJfaPass,
    Shadow2dSdfPass,
    Shadow2dMainPass,
}

pub struct Shadow2dMeshPassNode {
    main_view_query: QueryState<(Read<RenderPhase<Transparent2d>>, Read<ShadowView2d>)>,
}

impl FromWorld for Shadow2dMeshPassNode {
    fn from_world(world: &mut World) -> Self {
        Self {
            main_view_query: world.query_filtered(),
        }
    }
}

impl Node for Shadow2dMeshPassNode {
    fn update(&mut self, world: &mut World) {
        self.main_view_query.update_archetypes(world);
    }

    fn run<'w>(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let Ok((render_phase, shadow_view)) =
            self.main_view_query.get_manual(world, graph.view_entity())
        else {
            return Ok(());
        };

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("light_2d_mesh_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &shadow_view.attachment.texture.default_view,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_phase.render(&mut render_pass, world, graph.view_entity());

        Ok(())
    }
}

pub struct Shadow2dJfaPrepassNode {
    main_view_query: QueryState<(Read<ShadowView2d>, Read<DynamicUniformIndex<SdfMeta>>)>,
}

impl FromWorld for Shadow2dJfaPrepassNode {
    fn from_world(world: &mut World) -> Self {
        Self {
            main_view_query: world.query_filtered(),
        }
    }
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
        let Ok((shadow_view, meta_offset)) =
            self.main_view_query.get_manual(world, graph.view_entity())
        else {
            return Ok(());
        };

        let pipeline = world.resource::<Shadow2dJfaPrepassPipeline>();
        let Some(compute_pipeline) = world
            .resource::<PipelineCache>()
            .get_compute_pipeline(pipeline.cached_id)
        else {
            return Ok(());
        };

        let main_texture_view = &shadow_view.attachment.texture.default_view;
        let sdf_textures = world.resource::<SdfTextureStorage>();
        let gpu_meta_buffers = world.resource::<GpuMetaBuffers>();

        let sdf_texture = sdf_textures.get_sdf_texture(graph.view_entity());
        let bind_group = render_context.render_device().create_bind_group(
            "light_2d_jfa_prepass_bind_group",
            &pipeline.jfa_pass_layout,
            &BindGroupEntries::sequential((
                main_texture_view,
                &sdf_texture.get_primary_texture().texture_view,
                gpu_meta_buffers.sdf_meta_binding(),
            )),
        );

        let work_group_count = UVec3 {
            x: sdf_texture.size.x.div_ceil(SHADOW_WORK_GROUP_SIZE.x),
            y: sdf_texture.size.y.div_ceil(SHADOW_WORK_GROUP_SIZE.y),
            z: SHADOW_WORK_GROUP_SIZE.z,
        };

        let mut compute_pass =
            render_context
                .command_encoder()
                .begin_compute_pass(&ComputePassDescriptor {
                    label: Some("light_2d_jfa_prepass_pass"),
                    timestamp_writes: None,
                });

        compute_pass.set_pipeline(compute_pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[meta_offset.index()]);
        compute_pass.dispatch_workgroups(
            work_group_count.x,
            work_group_count.y,
            work_group_count.z,
        );

        Ok(())
    }
}

pub struct Shadow2dJfaPassNode {
    main_view_query: QueryState<Read<DynamicUniformIndex<SdfMeta>>>,
}

impl FromWorld for Shadow2dJfaPassNode {
    fn from_world(world: &mut World) -> Self {
        Self {
            main_view_query: world.query_filtered(),
        }
    }
}

impl Node for Shadow2dJfaPassNode {
    fn update(&mut self, world: &mut World) {
        self.main_view_query.update_archetypes(world);
    }

    fn run<'w>(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let Ok(meta_offset) = self.main_view_query.get_manual(world, graph.view_entity()) else {
            return Ok(());
        };

        let pipeline = world.resource::<Shadow2dJfaPassPipeline>();
        let Some(compute_pipeline) = world
            .resource::<PipelineCache>()
            .get_compute_pipeline(pipeline.cached_id)
        else {
            return Ok(());
        };

        let main_view_entity = graph.view_entity();
        let sdf_textures = world.resource::<SdfTextureStorage>();
        let gpu_meta_buffers = world.resource::<GpuMetaBuffers>();
        let number_buffer = world.resource::<NumberBuffer>();

        let sdf_texture = sdf_textures.get_sdf_texture(main_view_entity);
        let bind_group_primary_source = render_context.render_device().create_bind_group(
            "light_2d_jfa_pass_bind_group",
            &pipeline.jfa_pass_layout,
            &BindGroupEntries::sequential((
                &sdf_texture.get_primary_texture().texture_view,
                &sdf_texture.get_secondary_texture().texture_view,
                gpu_meta_buffers.sdf_meta_binding(),
                number_buffer.binding(),
            )),
        );

        let bind_group_secondary_source = render_context.render_device().create_bind_group(
            "light_2d_jfa_pass_bind_group",
            &pipeline.jfa_pass_layout,
            &BindGroupEntries::sequential((
                &sdf_texture.get_secondary_texture().texture_view,
                &sdf_texture.get_primary_texture().texture_view,
                gpu_meta_buffers.sdf_meta_binding(),
                number_buffer.binding(),
            )),
        );

        let bind_groups = [&bind_group_primary_source, &bind_group_secondary_source];

        let work_group_count = UVec3 {
            x: sdf_texture.size.x.div_ceil(SHADOW_WORK_GROUP_SIZE.x),
            y: sdf_texture.size.y.div_ceil(SHADOW_WORK_GROUP_SIZE.y),
            z: SHADOW_WORK_GROUP_SIZE.z,
        };

        for i in 0..sdf_texture.jfa_iterations {
            let mut compute_pass =
                render_context
                    .command_encoder()
                    .begin_compute_pass(&ComputePassDescriptor {
                        label: Some("light_2d_jfa_pass_pass"),
                        timestamp_writes: None,
                    });
            let iteration_offset = number_buffer.get_index(i);

            compute_pass.set_pipeline(compute_pipeline);
            compute_pass.set_bind_group(
                0,
                bind_groups[(i % 2) as usize],
                &[meta_offset.index(), iteration_offset],
            );
            compute_pass.dispatch_workgroups(
                work_group_count.x,
                work_group_count.y,
                work_group_count.z,
            );
        }

        Ok(())
    }
}

pub struct Shadow2dSdfPassNode {
    main_view_query: QueryState<Read<DynamicUniformIndex<SdfMeta>>>,
}

impl FromWorld for Shadow2dSdfPassNode {
    fn from_world(world: &mut World) -> Self {
        Self {
            main_view_query: world.query_filtered(),
        }
    }
}

impl Node for Shadow2dSdfPassNode {
    fn update(&mut self, world: &mut World) {
        self.main_view_query.update_archetypes(world);
    }

    fn run<'w>(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let Ok(meta_offset) = self.main_view_query.get_manual(world, graph.view_entity()) else {
            return Ok(());
        };

        let pipeline = world.resource::<Shadow2dSdfPassPipeline>();
        let Some(compute_pipeline) = world
            .resource::<PipelineCache>()
            .get_compute_pipeline(pipeline.cached_id)
        else {
            return Ok(());
        };

        let sdf_textures = world.resource::<SdfTextureStorage>();
        let gpu_meta_buffers = world.resource::<GpuMetaBuffers>();

        let sdf_texture = sdf_textures.get_sdf_texture(graph.view_entity());
        let bind_group = render_context.render_device().create_bind_group(
            "light_2d_sdf_pass_bind_group",
            &pipeline.sdf_pass_layout,
            &BindGroupEntries::sequential((
                &sdf_textures
                    .get_sdf_texture(graph.view_entity())
                    .get_texture()
                    .texture_view,
                gpu_meta_buffers.sdf_meta_binding(),
            )),
        );

        let work_group_count = UVec3 {
            x: sdf_texture.size.x.div_ceil(SHADOW_WORK_GROUP_SIZE.x),
            y: sdf_texture.size.y.div_ceil(SHADOW_WORK_GROUP_SIZE.y),
            z: SHADOW_WORK_GROUP_SIZE.z,
        };

        let mut compute_pass =
            render_context
                .command_encoder()
                .begin_compute_pass(&ComputePassDescriptor {
                    label: Some("light_2d_sdf_pass_pass"),
                    timestamp_writes: None,
                });

        compute_pass.set_pipeline(compute_pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[meta_offset.index()]);
        compute_pass.dispatch_workgroups(
            work_group_count.x,
            work_group_count.y,
            work_group_count.z,
        );

        Ok(())
    }
}

pub struct Shadow2dMainPassNode {
    main_view_query: QueryState<(
        Read<ViewTarget>,
        Read<ViewUniformOffset>,
        Read<DynamicUniformIndex<SdfMeta>>,
        Read<GpuLights2d>,
    )>,
}

impl FromWorld for Shadow2dMainPassNode {
    fn from_world(world: &mut World) -> Self {
        Self {
            main_view_query: world.query_filtered(),
        }
    }
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
        let Ok((view_target, view_offset, meta_offset, gpu_lights)) =
            self.main_view_query.get_manual(world, graph.view_entity())
        else {
            return Ok(());
        };

        let pipeline = world.resource::<Shadow2dMainPassPipeline>();
        let Some(render_pipeline) = world
            .resource::<PipelineCache>()
            .get_render_pipeline(pipeline.cached_id)
        else {
            return Ok(());
        };

        let sdf_textures = world.resource::<SdfTextureStorage>();
        let gpu_meta_buffers = world.resource::<GpuMetaBuffers>();
        let gpu_ambient_light_buffer = world.resource::<GpuAmbientLight2dBuffer>();
        let view_uniforms = world.resource::<ViewUniforms>();
        let post_process = view_target.post_process_write();

        let sdf_texture = sdf_textures.get_sdf_texture(graph.view_entity());
        let bind_group = render_context.render_device().create_bind_group(
            "light_2d_main_pass_bind_group",
            &pipeline.main_pass_layout,
            &BindGroupEntries::sequential((
                post_process.source,
                &pipeline.main_texture_sampler,
                view_uniforms.uniforms.binding().unwrap(),
                &sdf_texture.get_texture().texture_view,
                gpu_meta_buffers.sdf_meta_binding(),
                gpu_ambient_light_buffer.binding(),
                gpu_lights.point_lights_binding(),
            )),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("light_2d_main_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &post_process.destination,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_render_pipeline(render_pipeline);
        render_pass.set_bind_group(0, &bind_group, &[view_offset.offset, meta_offset.index()]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}
