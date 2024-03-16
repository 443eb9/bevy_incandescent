#import bevy_incandescent::ray_marching::shadow_2d_types::SdfMeta

@group(0) @binding(0)
var main_tex: texture_2d<f32>;

@group(0) @binding(1)
var sdf_tex: texture_storage_2d<rgba32float, write>;

@group(0) @binding(2)
var<uniform> sdf_meta: SdfMeta;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3u) {
    let px = invocation_id.xy;

    if px.x >= sdf_meta.size.x || px.y >= sdf_meta.size.x {
        return;
    }

    let uv = vec2f(px) / vec2f(sdf_meta.size);
    if textureLoad(main_tex, px, 0).a > sdf_meta.alpha_threshold {
        // Is inside
        textureStore(sdf_tex, px, vec4<f32>(uv, 0., 0.));
    } else {
        // Is outside
        // textureStore(sdf_tex, px, vec4<f32>(-1., -1., uv));
        textureStore(sdf_tex, px, vec4f(0.));
    }
}
