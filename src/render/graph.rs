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
        render_resource::RenderPassDescriptor,
        renderer::RenderContext,
    },
};

use crate::ecs::{
    camera::ShadowCameraDriver,
    light::{ShadowView2d, VisibleLight2dEntities},
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
                label: Some("shadow2d_mesh_pass"),
                color_attachments: &[Some(shadow_view.attachment.get_attachment())],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            println!("items: {:?}", transparent_phase.items.len());

            transparent_phase.render(&mut render_pass, world, light_entity);
        }
        Ok(())
    }
}

pub struct Shadow2dPrepassNode {
    main_view_query: QueryState<Read<VisibleLight2dEntities>, With<ShadowCameraDriver>>,
}

impl FromWorld for Shadow2dPrepassNode {
    fn from_world(world: &mut World) -> Self {
        Self {
            main_view_query: world.query_filtered(),
        }
    }
}

impl Node for Shadow2dPrepassNode {
    fn run<'w>(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
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
    main_view_query: QueryState<Read<VisibleLight2dEntities>>,
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
        let Ok(VisibleLight2dEntities(view_lights)) =
            self.main_view_query.get_manual(world, main_view_entity)
        else {
            return Ok(());
        };
        Ok(())
    }
}
