#import bevy_incandescent::ray_marching::types::SdfMeta

@group(0) @binding(0)
var sdf_tex: texture_storage_2d<rgba32float, read_write>;

@group(0) @binding(1)
var<uniform> sdf_meta: SdfMeta;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3u) {
    let px = invocation_id.xy;
    if px.x >= sdf_meta.size.x || px.y >= sdf_meta.size.y {
        return;
    }

    let px_data = textureLoad(sdf_tex, px);
    let interior_closest = distance(vec2f(px), textureLoad(sdf_tex, px).rg);
    let exterior_closest = distance(vec2f(px), textureLoad(sdf_tex, px).ba);
    // let interior_closest = distance(vec2f(px) / vec2f(sdf_meta.size), textureLoad(sdf_tex, px).rg / vec2f(sdf_meta.size));
    // let exterior_closest = distance(vec2f(px) / vec2f(sdf_meta.size), textureLoad(sdf_tex, px).ba / vec2f(sdf_meta.size));
    textureStore(sdf_tex, px, vec4f(interior_closest, exterior_closest, 0., 1.));
}
