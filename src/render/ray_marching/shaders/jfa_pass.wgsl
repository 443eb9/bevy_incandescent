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

    var best_dist_interior = 9999999.;
    var best_px_interior = vec2f(0.);
    var best_dist_exterior = 9999999.;
    var best_px_exterior = vec2f(0.);

    let step = i32(max(max(sdf_meta.size.x, sdf_meta.size.y) >> (jfa_iter + 1u), 1u));
    let tex_isize = vec2i(sdf_meta.size);

    for (var dx = -1; dx <= 1; dx++) {
        for (var dy = -1; dy <= 1; dy++) {
            let neighbor_px = vec2i(px) + vec2i(dx, dy) * step;
            let neighbor_data = textureLoad(source_sdf_tex, neighbor_px);
            
            if neighbor_data.x > 0. && neighbor_data.y > 0. {
                let d = distance(vec2f(px), vec2f(neighbor_data.xy));
                if d < best_dist_interior {
                    best_dist_interior = d;
                    best_px_interior = neighbor_data.xy;
                }
            }

            if neighbor_data.z > 0. && neighbor_data.w > 0. {
                let d = distance(vec2f(px), vec2f(neighbor_data.zw));
                if d < best_dist_exterior {
                    best_dist_exterior = d;
                    best_px_exterior = neighbor_data.zw;
                }
            }
        }
    }

    textureStore(dest_sdf_tex, px, vec4f(best_px_interior, best_px_exterior));
}
