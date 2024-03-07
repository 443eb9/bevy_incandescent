#import bevy_incandescent::shadow_2d_types::ShadowMapMeta;

@group(0) @binding(0)
var main_tex: texture_2d<f32>;

@group(0) @binding(1)
var shadow_map: texture_storage_2d_array<
#ifdef COMPATIBILITY
    rgba32float,
#else
    rg32float,
#endif
    write
>;

@group(0) @binding(2)
var<uniform> shadow_map_meta: ShadowMapMeta;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3u) {
    let px = invocation_id.xy;

    if px.x >= shadow_map_meta.size || px.y >= shadow_map_meta.size {
        return;
    }

    var d = 1.;
    if textureLoad(main_tex, px, 0).a > shadow_map_meta.alpha_threshold {
        d = length(vec2f(px) / vec2f(f32(shadow_map_meta.size)) - vec2f(0.5)) * 2.;
    }

    textureStore(
        shadow_map,
        vec2u(px.x, px.y),
        shadow_map_meta.index,
        vec4f(d, 0., 0., 0.),
    );
}
