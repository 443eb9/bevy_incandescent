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
var main_tex: texture_2d<f32>;

@group(0) @binding(1)
var main_tex_sampler: sampler;

@group(0) @binding(2)
var shadow_map: texture_storage_2d_array<rg32float, read>;

@group(0) @binding(3)
var<storage> point_lights: array<PointLight>;

@group(0) @binding(4)
var<storage> shadow_views: array<ShadowView2d>;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4f {
    // let scale = 512u >> 8u;
    // let px = in.uv * vec2f(f32(scale), 512.);
    // return textureLoad(shadow_map, vec2i(px), 0);
    // return vec4f(px / 512., 0., 1.);

    return textureSample(main_tex, main_tex_sampler, in.uv);
}
