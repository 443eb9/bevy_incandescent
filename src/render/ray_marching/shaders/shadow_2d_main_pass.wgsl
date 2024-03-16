#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_incandescent::ray_marching::shadow_2d_types::SdfMeta

@group(0) @binding(0)
var sdf_tex: texture_storage_2d<rgba32float, read>;

@group(0) @binding(1)
var<uniform> sdf_meta: SdfMeta;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4f {
    return textureLoad(sdf_tex, vec2i(in.uv * vec2f(sdf_meta.size)));
}
