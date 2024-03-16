#import bevy_incandescent::ray_marching::shadow_2d_types::SdfMeta

@group(0) @binding(0)
var source_sdf_tex: texture_storage_2d<rgba32float, read_write>;

@group(0) @binding(1)
var dest_sdf_tex: texture_storage_2d<rgba32float, read_write>;

@group(0) @binding(2)
var<uniform> sdf_meta: SdfMeta;

@group(0) @binding(3)
var<uniform> jfa_iter: u32;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3u) {
    let px = invocation_id.xy;

    if px.x >= sdf_meta.size.x || px.y >= sdf_meta.size.x {
        return;
    }

    let step = i32(max(max(sdf_meta.size.x, sdf_meta.size.y) >> (jfa_iter + 1u), 1u));
    let tex_fsize = vec2f(sdf_meta.size);
    let tex_isize = vec2i(sdf_meta.size);

    var best_dist_px = 10.;
    var best_seed_px = vec2i(0);

    for (var dx = -step; dx <= step; dx += step) {
        for (var dy = -step; dy <= step; dy += step) {
            let neighbor_px = vec2i(px) + vec2i(dx, dy);
            if neighbor_px.x < 0 || neighbor_px.x >= tex_isize.x
               || neighbor_px.y < 0 || neighbor_px.y >= tex_isize.y {
                continue;
            }

            let neighbor_best_seed_uv = textureLoad(source_sdf_tex, neighbor_px).rg;
            if neighbor_best_seed_uv.x > 0. && neighbor_best_seed_uv.y > 0. {
                let neighbor_best_seed_px = vec2i(neighbor_best_seed_uv * tex_fsize);
                let this_to_neighbor_best_seed_px = distance(vec2f(px), vec2f(neighbor_best_seed_px));
                if this_to_neighbor_best_seed_px < best_dist_px {
                    best_dist_px = this_to_neighbor_best_seed_px;
                    best_seed_px = vec2i(neighbor_best_seed_px);
                }
            }
        }
    }

    textureStore(dest_sdf_tex, px, vec4f(vec2f(best_seed_px) / tex_fsize, 0.0, 0.0));
}
