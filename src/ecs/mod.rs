use bevy::{
    app::{App, Plugin},
    ecs::{bundle::Bundle, component::Component, reflect::ReflectResource, system::Resource},
    reflect::Reflect,
    render::{
        color::Color,
        extract_resource::ExtractResource,
        primitives::Frustum,
        view::{InheritedVisibility, ViewVisibility, Visibility, VisibleEntities},
    },
    transform::components::{GlobalTransform, Transform},
};

use crate::math::CircularSector;

#[cfg(feature = "catalinzz")]
pub mod catalinzz;
#[cfg(feature = "pbr")]
pub mod pbr;

pub struct IncandescentEcsPlugin;

impl Plugin for IncandescentEcsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PointLight2d>()
            .register_type::<SpotLight2d>()
            .register_type::<AmbientLight2d>();
    }
}

#[derive(Component, Default, Clone, Copy, Reflect)]
pub struct PointLight2d {
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
    pub radius: f32,
}

#[derive(Component, Default, Clone, Copy, Reflect)]
pub struct SpotLight2d {
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
    pub radius: f32,
    pub sector: CircularSector,
}

#[derive(Component)]
pub struct ShadowCaster2d;

#[derive(Resource, ExtractResource, Clone, Copy, Reflect)]
#[reflect(Resource)]
pub struct AmbientLight2d {
    pub color: Color,
    pub intensity: f32,
}

impl Default for AmbientLight2d {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            intensity: 1.,
        }
    }
}

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

#[derive(Bundle, Default)]
pub struct SpotLight2dBundle {
    pub spot_light: SpotLight2d,
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
