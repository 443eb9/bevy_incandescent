struct ShadowMapMeta {
    index: u32,
    size: u32,
}

@group(0) @binding(0)
var main_tex: texture_2d<f32>;

@group(0) @binding(1)
var shadow_map: texture_storage_2d_array<rg32float, write>;

@group(0) @binding(2)
var<uniform> shadow_map_meta: ShadowMapMeta;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3u) {
    let px = invocation_id.xy;

    if px.x >= shadow_map_meta.size || px.y >= shadow_map_meta.size {
        return;
    }

    let uv = vec2f(px) / vec2f(f32(shadow_map_meta.size));
    let uv0 = uv * 2. - 1.;
    let v0 = uv0.y * abs(uv0.x);
    let coord = vec2f(uv.x, (v0 + 1.) * 0.5) * f32(shadow_map_meta.size);
    let color = vec4f(
        textureLoad(main_tex, vec2u(coord), 0).a,
        textureLoad(main_tex, vec2u(coord.yx), 0).a,
        0.,
        0.,
    );

    storageBarrier();

    textureStore(
        shadow_map,
        px,
        shadow_map_meta.index,
        color,
    );
}
