use bevy::reflect::Reflect;

#[derive(Clone, Copy, Reflect)]
pub enum CircularSector {
    Extent { origin: f32, extent: f32 },
    Angles { start: f32, end: f32 },
}

impl Default for CircularSector {
    fn default() -> Self {
        Self::Angles {
            start: 0.,
            end: std::f32::consts::TAU,
        }
    }
}

impl CircularSector {
    pub fn into_extent(self) -> [f32; 2] {
        match self {
            Self::Extent { origin, extent } => [origin, extent],
            Self::Angles { start, end } => [(end + start) / 2., (end - start) / 2.],
        }
    }

    pub fn into_angles(self) -> [f32; 2] {
        match self {
            Self::Extent { origin, extent } => [origin - extent, origin + extent],
            Self::Angles { start, end } => [start, end],
        }
    }
}
