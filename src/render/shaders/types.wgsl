#define_import_path bevy_incandescent::types

struct AmbientLight2d {
    color: vec4f,
    intensity: f32,
}

// Here the screen space position means:
// - x: 0 is the left edge of the screen, 1 is the right edge
// - y: 0 is the top edge of the screen, 1 is the bottom edge
// which is equivalent to (NDC + 1.) / 2.

struct PointLight2d {
    intensity: f32,
    position_ss: vec2f,
    radius_ss: f32,
    range_ss: f32,
    color: vec4f,
    angles: array<f32, 2>,
}
