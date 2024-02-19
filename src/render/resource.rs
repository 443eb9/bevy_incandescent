use bevy::{
    asset::Handle,
    ecs::{
        entity::{Entity, EntityHashMap},
        system::Resource,
    },
    math::UVec2,
    render::texture::Image,
};

#[derive(Resource, Default)]
pub struct ShadowRenderTargets {
    target_size: EntityHashMap<UVec2>,
    nearest_mapper: EntityHashMap<Handle<Image>>,
}

impl ShadowRenderTargets {
    #[inline]
    pub fn insert_view_target(&mut self, view: Entity, size: UVec2) {
        self.target_size.insert(view, size);
    }

    #[inline]
    pub fn get_nearest_mapper(&mut self, light: Entity) -> Option<&Handle<Image>> {
        self.nearest_mapper.get(&light)
    }
}
