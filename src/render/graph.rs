use bevy::{
    ecs::{
        query::QueryState,
        world::{FromWorld, World},
    },
    render::{
        camera::Camera,
        render_graph::{Node, NodeRunError, RenderGraphContext},
        renderer::RenderContext,
    },
};

pub struct IncandescentPrePassNode {
    view_query: QueryState<&'static Camera>,
}

impl FromWorld for IncandescentPrePassNode {
    fn from_world(world: &mut World) -> Self {
        Self {
            view_query: world.query_filtered(),
        }
    }
}

impl Node for IncandescentPrePassNode {
    fn update(&mut self, world: &mut World) {
        self.view_query.update_archetypes(world);
    }

    fn run<'w>(
        &self,
        _graph: &mut RenderGraphContext,
        _render_context: &mut RenderContext<'w>,
        _world: &'w World,
    ) -> Result<(), NodeRunError> {
        Ok(())
    }
}
