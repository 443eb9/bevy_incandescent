use bevy::{
    ecs::bundle::Bundle,
    render::{
        primitives::Frustum,
        view::{InheritedVisibility, ViewVisibility, Visibility, VisibleEntities},
    },
    transform::components::{GlobalTransform, Transform},
};

use super::{PointLight2d, ShadowCaster2d};

#[derive(Bundle, Default)]
pub struct PointLight2dBundle {
    pub point_light: PointLight2d,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    #[cfg(feature = "catalinzz")]
    pub frustum: Frustum,
    #[cfg(feature = "catalinzz")]
    pub visible_casters: VisibleEntities,
    pub visibility: Visibility,
    pub view_visibility: ViewVisibility,
    pub inherited_visibility: InheritedVisibility,
}

#[derive(Bundle)]
pub struct ShadowCaster2dBundle {
    pub shadow_caster: ShadowCaster2d,
}

impl Default for ShadowCaster2dBundle {
    fn default() -> Self {
        Self {
            shadow_caster: ShadowCaster2d,
        }
    }
}
