use bevy::app::{App, Plugin};
use ecs::IncandescentEcsPlugin;
use render::IncandescentRenderPlugin;

#[cfg(feature = "debug")]
pub mod debug;
pub mod ecs;
pub mod math;
pub mod render;

const APPRACHES: [LightingApproach; 1] = [
    #[cfg(feature = "catalinzz")]
    LightingApproach::Catalinzz,
    #[cfg(not(feature = "catalinzz"))]
    LightingApproach::None,
];

pub struct IncandescentPlugin {
    pub approach: LightingApproach,
}

impl Default for IncandescentPlugin {
    fn default() -> Self {
        Self {
            approach: *APPRACHES
                .iter()
                .find(|a| (**a) != LightingApproach::None)
                .unwrap_or_else(|| panic!("No lighting approach is enabled")),
        }
    }
}

impl Plugin for IncandescentPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            IncandescentRenderPlugin,
            IncandescentEcsPlugin,
            #[cfg(feature = "debug")]
            debug::IncandescentDebugPlugin,
        ));

        match self.approach {
            LightingApproach::None => unreachable!(),
            #[cfg(feature = "catalinzz")]
            LightingApproach::Catalinzz => {
                app.add_plugins(render::catalinzz::CatalinzzApproachPlugin);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightingApproach {
    None,
    #[cfg(feature = "catalinzz")]
    Catalinzz,
}
