use bevy::{
    core_pipeline::core_2d::Transparent2d,
    ecs::{
        query::{QueryState, With},
        system::lifetimeless::Read,
        world::{FromWorld, World},
    },
    render::{
        render_graph::{Node, NodeRunError, RenderGraphContext, RenderLabel},
        render_phase::RenderPhase,
        render_resource::{
            BindGroupEntries, ComputePassDescriptor, Operations, PipelineCache,
            RenderPassColorAttachment, RenderPassDescriptor,
        },
        renderer::RenderContext,
        view::{ViewTarget, ViewUniformOffset, ViewUniforms, VisibleEntities},
    },
};

use crate::{
    ecs::{catalinzz::MainShadowCameraDriver, ShadowView2d},
    render::{
        light::{GpuAmbientLight2dBuffer, GpuLights2d},
        universal_buffers::NumberBuffer,
        DynamicUniformIndex,
    },
};

use super::{
    pipeline::{
        Shadow2dDistortPassPipeline, Shadow2dMainPassPipeline, Shadow2dPrepassPipeline,
        Shadow2dReductionPipeline,
    },
    GpuMetaBuffers, GpuShadowMapMeta, PoissonDiskBuffer, ShadowMap2dStorage,
};

#[derive(RenderLabel, Debug, Hash, PartialEq, Eq, Clone)]
pub enum Shadow2dNode {
    Shadow2dMeshPass,
    Shadow2dPrepass,
    Shadow2dDistortPass,
    Shadow2dReductionPass,
    Shadow2dMainPass,
}

pub struct Shadow2dMeshPassNode {
    main_view_query: QueryState<Read<VisibleEntities>, With<MainShadowCameraDriver>>,
    light_view_query: QueryState<(Read<RenderPhase<Transparent2d>>, Read<ShadowView2d>)>,
}

impl FromWorld for Shadow2dMeshPassNode {
    fn from_world(world: &mut World) -> Self {
        Self {
            main_view_query: world.query_filtered(),
            light_view_query: world.query_filtered(),
        }
    }
}

impl Node for Shadow2dMeshPassNode {
    #[inline]
    fn update(&mut self, world: &mut World) {
        self.main_view_query.update_archetypes(world);
        self.light_view_query.update_archetypes(world);
    }

    fn run<'w>(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let main_view_entity = graph.view_entity();
        let Ok(view_lights) = self.main_view_query.get_manual(world, main_view_entity) else {
            return Ok(());
        };

        for light_entity in view_lights.iter().copied() {
            let Ok((transparent_phase, shadow_view)) =
                self.light_view_query.get_manual(world, light_entity)
            else {
                return Ok(());
            };

            let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some("shadow_2d_mesh_pass"),
                color_attachments: &[Some(shadow_view.attachment.get_attachment())],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            transparent_phase.render(&mut render_pass, world, light_entity);
        }

        Ok(())
    }
}

pub struct Shadow2dPrepassNode {
    light_view_query: QueryState<(
        Read<ShadowView2d>,
        Read<DynamicUniformIndex<GpuShadowMapMeta>>,
    )>,
}

impl FromWorld for Shadow2dPrepassNode {
    fn from_world(world: &mut World) -> Self {
        Self {
            light_view_query: world.query_filtered(),
        }
    }
}

impl Node for Shadow2dPrepassNode {
    #[inline]
    fn update(&mut self, world: &mut World) {
        self.light_view_query.update_archetypes(world);
    }

    fn run<'w>(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        if self.light_view_query.iter_manual(world).next().is_none() {
            return Ok(());
        }

        let pipeline = world.resource::<Shadow2dPrepassPipeline>();
        let Some(compute_pipeline) = world
            .resource::<PipelineCache>()
            .get_compute_pipeline(pipeline.cached_id)
        else {
            return Ok(());
        };

        let gpu_meta_buffers = world.resource::<GpuMetaBuffers>();
        let shadow_map_storage = world.resource::<ShadowMap2dStorage>();
        let work_group_count = shadow_map_storage.work_group_count_per_light();

        for (shadow_view, uniform_index) in self.light_view_query.iter_manual(world) {
            let bind_group = render_context.render_device().create_bind_group(
                "shadow_2d_prepass_bind_group",
                &pipeline.prepass_layout,
                &BindGroupEntries::sequential((
                    &shadow_view.attachment.texture.default_view,
                    shadow_map_storage.alpha_map_view(),
                    shadow_map_storage.texture_view_primary(),
                    gpu_meta_buffers.shadow_map_meta_buffer_binding(),
                )),
            );

            let mut compute_pass =
                render_context
                    .command_encoder()
                    .begin_compute_pass(&ComputePassDescriptor {
                        label: Some("shadow_2d_prepass"),
                        timestamp_writes: None,
                    });

            compute_pass.set_pipeline(compute_pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[uniform_index.index()]);
            compute_pass.dispatch_workgroups(
                work_group_count.x,
                work_group_count.y,
                work_group_count.z,
            );
        }

        Ok(())
    }
}

pub struct Shadow2dDistortPassNode {
    main_view_query: QueryState<(), With<MainShadowCameraDriver>>,
    light_view_query: QueryState<(), With<ShadowView2d>>,
}

impl FromWorld for Shadow2dDistortPassNode {
    fn from_world(world: &mut World) -> Self {
        Self {
            main_view_query: world.query_filtered(),
            light_view_query: world.query_filtered(),
        }
    }
}

impl Node for Shadow2dDistortPassNode {
    #[inline]
    fn update(&mut self, world: &mut World) {
        self.main_view_query.update_archetypes(world);
        self.light_view_query.update_archetypes(world);
    }

    fn run<'w>(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let Ok(_) = self.main_view_query.get_manual(world, graph.view_entity()) else {
            return Ok(());
        };

        if self.light_view_query.iter_manual(world).next().is_none() {
            return Ok(());
        }

        let pipeline = world.resource::<Shadow2dDistortPassPipeline>();
        let Some(compute_pipeline) = world
            .resource::<PipelineCache>()
            .get_compute_pipeline(pipeline.cached_id)
        else {
            return Ok(());
        };

        let gpu_meta_buffers = world.resource::<GpuMetaBuffers>();
        let shadow_map_storage = world.resource::<ShadowMap2dStorage>();
        let work_group_count = shadow_map_storage.work_group_count_total();

        let bind_group = render_context.render_device().create_bind_group(
            "shadow_2d_distort_pass_bind_group",
            &pipeline.distort_pass_layout,
            &BindGroupEntries::sequential((
                shadow_map_storage.texture_view_primary(),
                shadow_map_storage.texture_view_secondary(),
                gpu_meta_buffers.shadow_map_meta_buffer_binding(),
            )),
        );

        let mut compute_pass =
            render_context
                .command_encoder()
                .begin_compute_pass(&ComputePassDescriptor {
                    label: Some("shadow_2d_distort_pass"),
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

pub struct Shadow2dReductionNode {
    main_view_query: QueryState<(), With<MainShadowCameraDriver>>,
    light_view_query: QueryState<(), With<ShadowView2d>>,
}

impl FromWorld for Shadow2dReductionNode {
    fn from_world(world: &mut World) -> Self {
        Self {
            main_view_query: world.query_filtered(),
            light_view_query: world.query_filtered(),
        }
    }
}

impl Node for Shadow2dReductionNode {
    #[inline]
    fn update(&mut self, world: &mut World) {
        self.main_view_query.update_archetypes(world);
        self.light_view_query.update_archetypes(world);
    }

    fn run<'w>(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let Ok(_) = self.main_view_query.get_manual(world, graph.view_entity()) else {
            return Ok(());
        };

        if self.light_view_query.iter_manual(world).next().is_none() {
            return Ok(());
        }

        let pipeline = world.resource::<Shadow2dReductionPipeline>();
        let Some(compute_pipeline) = world
            .resource::<PipelineCache>()
            .get_compute_pipeline(pipeline.cached_id)
        else {
            return Ok(());
        };

        let shadow_map_storage = world.resource::<ShadowMap2dStorage>();
        let gpu_meta_buffers = world.resource::<GpuMetaBuffers>();
        let work_group_count = shadow_map_storage.work_group_count_total();
        let number_buffer = world.resource::<NumberBuffer>();

        let bind_group_primary_source = render_context.render_device().create_bind_group(
            "shadow_2d_reduction_pass_bind_group",
            &pipeline.reduction_layout,
            &BindGroupEntries::sequential((
                shadow_map_storage.texture_view_primary(),
                shadow_map_storage.texture_view_secondary(),
                gpu_meta_buffers.shadow_map_meta_buffer_binding(),
                number_buffer.binding(),
            )),
        );
        let bind_group_secondary_source = render_context.render_device().create_bind_group(
            "shadow_2d_reduction_pass_bind_group",
            &pipeline.reduction_layout,
            &BindGroupEntries::sequential((
                shadow_map_storage.texture_view_secondary(),
                shadow_map_storage.texture_view_primary(),
                gpu_meta_buffers.shadow_map_meta_buffer_binding(),
                number_buffer.binding(),
            )),
        );
        let bind_groups = [&bind_group_secondary_source, &bind_group_primary_source];

        for t in 0..shadow_map_storage.num_reductions() as usize {
            let mut compute_pass =
                render_context
                    .command_encoder()
                    .begin_compute_pass(&ComputePassDescriptor {
                        label: Some("shadow_2d_reduction_pass"),
                        timestamp_writes: None,
                    });
            let offset = number_buffer.get_index(t as u32);

            compute_pass.set_pipeline(compute_pipeline);
            compute_pass.set_bind_group(0, bind_groups[t % 2], &[offset]);
            compute_pass.dispatch_workgroups(
                work_group_count.x,
                work_group_count.y,
                work_group_count.z,
            );
        }

        Ok(())
    }
}

pub struct Shadow2dMainPassNode {
    main_view_query: QueryState<(Read<ViewTarget>, Read<ViewUniformOffset>, Read<GpuLights2d>)>,
    light_view_query: QueryState<(), With<ShadowView2d>>,
}

impl FromWorld for Shadow2dMainPassNode {
    fn from_world(world: &mut World) -> Self {
        Self {
            main_view_query: world.query_filtered(),
            light_view_query: world.query_filtered(),
        }
    }
}

impl Node for Shadow2dMainPassNode {
    #[inline]
    fn update(&mut self, world: &mut World) {
        self.main_view_query.update_archetypes(world);
        self.light_view_query.update_archetypes(world);
    }

    fn run<'w>(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let main_view_entity = graph.view_entity();
        let Ok((view_target, main_view_offset, gpu_lights)) =
            self.main_view_query.get_manual(world, main_view_entity)
        else {
            return Ok(());
        };

        if self.light_view_query.iter_manual(world).next().is_none() {
            return Ok(());
        }

        let pipeline = world.resource::<Shadow2dMainPassPipeline>();
        let Some(render_pipeline) = world
            .resource::<PipelineCache>()
            .get_render_pipeline(pipeline.cached_id)
        else {
            return Ok(());
        };

        let post_process = view_target.post_process_write();
        let shadow_map_storage = world.resource::<ShadowMap2dStorage>();
        let gpu_meta_buffers = world.resource::<GpuMetaBuffers>();
        let view_uniforms = world.resource::<ViewUniforms>();
        let gpu_ambient_light_buffer = world.resource::<GpuAmbientLight2dBuffer>();
        let poisson_disk_buffer = world.resource::<PoissonDiskBuffer>();

        let bind_group = render_context.render_device().create_bind_group(
            "shadow_2d_main_pass",
            &pipeline.main_pass_layout,
            &BindGroupEntries::sequential((
                post_process.source,
                &pipeline.main_texture_sampler,
                shadow_map_storage.alpha_map_view(),
                shadow_map_storage.final_texture_view(),
                view_uniforms.uniforms.binding().unwrap(),
                gpu_meta_buffers.shadow_map_meta_buffer_binding(),
                gpu_ambient_light_buffer.binding(),
                poisson_disk_buffer.binding(),
                gpu_lights.point_lights_binding(),
            )),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("shadow_2d_main_pass"),
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
        render_pass.set_bind_group(0, &bind_group, &[main_view_offset.offset]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}
