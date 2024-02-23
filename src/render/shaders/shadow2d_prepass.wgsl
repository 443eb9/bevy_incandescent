#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0)
var main_tex: texture_2d<f32>;

@group(0) @binding(1)
var main_tex_sampler: sampler;

@group(0) @binding(2)
var shadow_map_precursor: texture_storage_2d<rg32float, write>;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4f {
    return vec4f(1.0, 0.0, 0.0, 1.0);
}
