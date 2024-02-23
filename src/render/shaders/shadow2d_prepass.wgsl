#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

struct LightMeta {
    shadow_map_size: u32,
    light_index: u32,
}

@group(0) @binding(0)
var main_tex: texture_2d<f32>;

@group(0) @binding(1)
var main_tex_sampler: sampler;

@group(0) @binding(3)
var<uniform> shadow_map_meta: ShadowMapMeta;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec2f {
    let caster_value = textureSample(shadow_map_precursor, main_tex_sampler, in.uv);
    return vec4f(0.);
}
