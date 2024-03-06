#import bevy_incandescent::shadow_2d_types::ShadowMapMeta;

@group(0) @binding(0)
var source_shadow_map: texture_storage_2d_array<
#ifdef COMPATIBILITY
    rgba32float,
#else
    rg32float,
#endif
    read
>;

@group(0) @binding(1)
var dest_shadow_map: texture_storage_2d_array<
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
    let light_index = invocation_id.z;

    if px.x >= shadow_map_meta.size || px.y >= shadow_map_meta.size {
        return;
    }

    let uv = vec2f(px) / vec2f(f32(shadow_map_meta.size));
    let uv_ndc = uv * 2. - 1.;
    let v0 = (uv_ndc.y * abs(uv_ndc.x) + 1.) / 2.;
    let distorted_ndc = vec2f(uv.x, v0);
    let distorted_px = vec2i(distorted_ndc * vec2f(f32(shadow_map_meta.size)));
    
    let color = vec4f(
        textureLoad(source_shadow_map, distorted_px, light_index).r,
        textureLoad(source_shadow_map, distorted_px.yx, light_index).r,
        0.,
        0.,
    );

    textureStore(
        dest_shadow_map,
        px,
        light_index,
        color,
    );
}
