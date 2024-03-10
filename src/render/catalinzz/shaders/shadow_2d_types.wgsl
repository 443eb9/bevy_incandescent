#define_import_path bevy_incandescent::catalinzz::shadow_2d_types

struct AmbientLight2d {
    color: vec4f,
    intensity: f32,
}

struct PointLight2d {
    intensity: f32,
    position_ss: vec2f,
    radius_ss: f32,
    range_ss: f32,
    color: vec4f,
    angles: array<f32, 2>,
}

struct ShadowMapMeta {
    index: u32,
    size: u32,
    offset: vec2f,
    bias: f32,
    alpha_threshold: f32,
    pcf_samples: u32,
    pcf_radius: f32,
}
