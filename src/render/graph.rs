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
        view::ViewTarget,
    },
};

use crate::ecs::{
    camera::ShadowCameraDriver,
    light::{ShadowView2d, VisibleLight2dEntities},
};

use super::{
    pipeline::{Shadow2dMainPassPipeline, Shadow2dPrepassPipeline},
    prepare::DynamicUniformIndex,
    resource::{GpuLights2d, GpuShadowMap2dMeta, GpuShadowMap2dMetaBuffer, ShadowMap2dStorage},
};

#[derive(RenderLabel, Debug, Hash, PartialEq, Eq, Clone)]
pub enum Shadow2dNode {
    Shadow2dMeshPass,
    Shadow2dPrepass,
    Shadow2dReductionPass,
    Shadow2dMainPass,
}

pub struct Shadow2dMeshPassNode {
    main_view_query: QueryState<Read<VisibleLight2dEntities>, With<ShadowCameraDriver>>,
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
        let Ok(VisibleLight2dEntities(view_lights)) =
            self.main_view_query.get_manual(world, main_view_entity)
        else {
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
    main_view_query: QueryState<Read<VisibleLight2dEntities>, With<ShadowCameraDriver>>,
    light_view_query: QueryState<(
        Read<ShadowView2d>,
        Read<DynamicUniformIndex<GpuShadowMap2dMeta>>,
    )>,
}

impl FromWorld for Shadow2dPrepassNode {
    fn from_world(world: &mut World) -> Self {
        Self {
            main_view_query: world.query_filtered(),
            light_view_query: world.query_filtered(),
        }
    }
}

impl Node for Shadow2dPrepassNode {
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
        let Ok(VisibleLight2dEntities(view_lights)) =
            self.main_view_query.get_manual(world, main_view_entity)
        else {
            return Ok(());
        };

        let pipeline = world.resource::<Shadow2dPrepassPipeline>();
        let Some(compute_pipeline) = world
            .resource::<PipelineCache>()
            .get_compute_pipeline(pipeline.cached_id)
        else {
            return Ok(());
        };

        let shadow_map_meta = world.resource::<GpuShadowMap2dMetaBuffer>();
        let shadow_map_storage = world.resource::<ShadowMap2dStorage>();
        let work_group_count = shadow_map_storage.work_group_count_per_light();

        for shadow_view in view_lights.iter().copied() {
            let Ok((shadow_view, uniform_index)) =
                self.light_view_query.get_manual(world, shadow_view)
            else {
                continue;
            };

            let bind_group = render_context.render_device().create_bind_group(
                "shadow_2d_prepass_bind_group",
                &pipeline.prepass_layout,
                &BindGroupEntries::sequential((
                    &shadow_view.attachment.texture.default_view,
                    shadow_map_storage.texture_view(),
                    shadow_map_meta.binding(),
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

pub struct Shadow2dReductionNode {
    main_view_query: QueryState<Read<VisibleLight2dEntities>, With<ShadowCameraDriver>>,
}

impl FromWorld for Shadow2dReductionNode {
    fn from_world(world: &mut World) -> Self {
        Self {
            main_view_query: world.query_filtered(),
        }
    }
}

impl Node for Shadow2dReductionNode {
    fn run<'w>(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        Ok(())
    }
}

pub struct Shadow2dMainPass {
    main_view_query: QueryState<(Read<ViewTarget>, Read<VisibleLight2dEntities>)>,
    light_view_query: QueryState<Read<ShadowView2d>>,
}

impl FromWorld for Shadow2dMainPass {
    fn from_world(world: &mut World) -> Self {
        Self {
            main_view_query: world.query_filtered(),
            light_view_query: world.query_filtered(),
        }
    }
}

impl Node for Shadow2dMainPass {
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
        let Ok((view_target, VisibleLight2dEntities(_))) =
            self.main_view_query.get_manual(world, main_view_entity)
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

        let post_process = view_target.post_process_write();
        let shadow_map_storage = world.resource::<ShadowMap2dStorage>();
        let gpu_lights = world.resource::<GpuLights2d>();

        let bind_group = render_context.render_device().create_bind_group(
            "shadow_2d_main_pass",
            &pipeline.main_pass_layout,
            &BindGroupEntries::sequential((
                post_process.source,
                &pipeline.main_texture_sampler,
                shadow_map_storage.texture_view(),
                gpu_lights.point_lights_binding(),
                gpu_lights.shadow_views_binding(),
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
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}
