use bevy::{
    ecs::bundle::Bundle,
    render::view::{InheritedVisibility, RenderLayers, ViewVisibility, Visibility},
    transform::components::{GlobalTransform, Transform},
};

use crate::render::DEFAULT_SHADOW_CASTER_LAYER;

use super::light::{PointLight2d, ShadowCaster2d};

#[derive(Bundle, Default)]
pub struct PointLight2dBundle {
    pub point_light: PointLight2d,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

#[derive(Bundle)]
pub struct ShadowCaster2dBundle {
    pub render_layer: RenderLayers,
    pub shadow_caster: ShadowCaster2d,
}

impl Default for ShadowCaster2dBundle {
    fn default() -> Self {
        Self {
            render_layer: DEFAULT_SHADOW_CASTER_LAYER,
            shadow_caster: ShadowCaster2d,
        }
    }
}
