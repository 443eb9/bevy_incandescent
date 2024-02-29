use bevy::{
    ecs::bundle::Bundle,
    render::view::{InheritedVisibility, ViewVisibility, Visibility},
    transform::components::{GlobalTransform, Transform},
};

use crate::render::DEFAULT_SHADOW_CASTER_LAYER;

use super::light::{PointLight2d, ShadowCaster2d, ShadowCaster2dVisibility, ShadowLayers};

#[derive(Bundle)]
pub struct ShadowRenderedCameraBundle {
    pub shadow_layer: ShadowLayers,
}

impl Default for ShadowRenderedCameraBundle {
    fn default() -> Self {
        Self {
            shadow_layer: DEFAULT_SHADOW_CASTER_LAYER,
        }
    }
}

#[derive(Bundle, Default)]
pub struct PointLight2dBundle {
    pub point_light: PointLight2d,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
}

#[derive(Bundle)]
pub struct ShadowCaster2dBundle {
    pub shadow_layer: ShadowLayers,
    pub shadow_caster: ShadowCaster2d,
    pub shadow_caster_visibility: ShadowCaster2dVisibility,
}

impl Default for ShadowCaster2dBundle {
    fn default() -> Self {
        Self {
            shadow_layer: DEFAULT_SHADOW_CASTER_LAYER,
            shadow_caster: ShadowCaster2d,
            shadow_caster_visibility: ShadowCaster2dVisibility(true),
        }
    }
}
