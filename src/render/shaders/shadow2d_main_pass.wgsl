#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

struct PointLight {
    world_position: vec4f,
    color: vec4f,
}

struct ShadowView2d {
    view: mat4x4f,
    projection: mat4x4f,
}

@group(0) @binding(0)
var shadow_map: texture_storage_2d_array<rg32float, write>;

@group(0) @binding(1)
var shadow_map_sampler: sampler;

@group(0) @binding(2)
var main_texture: texture_2d<f32>;

@group(0) @binding(3)
var main_texture_sampler: sampler;

@group(0) @binding(4)
var<storage> point_lights: array<PointLight>;

@group(0) @binding(5)
var<storage> shadow_views: array<ShadowView2d>;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4f {
    return vec4f(textureSample(main_texture, main_texure_sampler, in.uv), 0.0, 1.0);
}
