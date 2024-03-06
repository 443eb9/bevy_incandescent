#define_import_path bevy_incandescent::shadow_2d_types

struct ShadowMapMeta {
    index: u32,
    size: u32,
    offset: vec2f,
    bias: f32,
    pcf_samples: u32,
    pcf_radius: f32,
}
