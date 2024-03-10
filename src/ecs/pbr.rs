use bevy::{
    asset::{Asset, Handle},
    ecs::bundle::Bundle,
    reflect::TypePath,
    render::{
        color::Color,
        mesh::Mesh,
        render_resource::AsBindGroup,
        texture::Image,
        view::{InheritedVisibility, ViewVisibility, Visibility},
    },
    transform::components::{GlobalTransform, Transform},
};

#[derive(Asset, AsBindGroup, TypePath, Default, Clone)]
pub struct StandardMaterial2d {
    pub base_color: Color,
    pub base_color_texture: Option<Handle<Image>>,
    pub normal_map_texture: Option<Handle<Image>>,
}

#[derive(Bundle, Default)]
pub struct PbrMesh2dBundle {
    pub material: Handle<StandardMaterial2d>,
    pub mesh: Handle<Mesh>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}
