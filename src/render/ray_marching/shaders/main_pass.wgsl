#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::view::View
#import bevy_incandescent::{
    ray_marching::types::SdfMeta,
    math::{is_point_inside_sector},
    types::{AmbientLight2d, PointLight2d},
}

@group(0) @binding(0)
var main_tex: texture_2d<f32>;

@group(0) @binding(1)
var main_tex_sampler: sampler;

@group(0) @binding(2)
var<uniform> main_view: View;

@group(0) @binding(3)
var sdf_tex: texture_storage_2d<rgba32float, read>;

@group(0) @binding(4)
var<uniform> sdf_meta: SdfMeta;

@group(0) @binding(5)
var<uniform> ambient_light: AmbientLight2d;

@group(0) @binding(6)
var<storage> point_lights: array<PointLight2d>;

fn ray_marching(px: vec2f, dir: vec2f, center: vec2f, radius: f32, screen_size: vec2f) -> f32 {
    let tex_fsize = vec2f(sdf_meta.size);
    let screen_to_tex = tex_fsize / screen_size;
    let center_tex_px = center * screen_to_tex;
    let scaled_hardness = length(screen_to_tex) * sdf_meta.hardness;
    var current = px * screen_to_tex;
    let max_progress = distance(center_tex_px, current);
    var progress = 0.001;
    var intensity = 999999.;

    var closest = textureLoad(sdf_tex, vec2i(current)).r;
    while current.x > 0. && current.x < tex_fsize.x
          && current.y > 0. && current.y < tex_fsize.y {
        let sdf_data = textureLoad(sdf_tex, vec2i(current)).rg;
        let step = sdf_data.r;
        if step < 0.1 {
            return 0.;
        }

        closest = min(closest, step);
        progress += step;
        current = px * screen_to_tex + dir * progress;
        intensity = min(intensity, scaled_hardness * step / progress);
        if progress >= max_progress {
            return saturate(intensity);
        }
    }
    return 1.;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4f {
    let screen_size = 2. * vec2f(main_view.inverse_projection[0][0], main_view.inverse_projection[1][1]);
    let fpx = in.uv * vec2f(screen_size);
    let px = vec2i(fpx);

    var color = vec3f(0.);
    for (var i_light = 0u; i_light < arrayLength(&point_lights); i_light++) {
        let light = &point_lights[i_light];
        let light_range_ss = max((*light).range_ss * screen_size.x, 0.);
        let light_radius_ss = max((*light).radius_ss * screen_size.x, 0.);
        let light_pos_ss = (*light).position_ss * screen_size;
        let dir = normalize(light_pos_ss - fpx);

        let angles = array<f32, 2>(
            // TODO WHY?
            -(*light).angles[0],
            (*light).angles[1],
        );

        if is_point_inside_sector(fpx, light_pos_ss, light_range_ss, angles) {
        // if is_point_inside_sector(fpx, light_pos_ss, light_range_ss, (*light).angles) {
            var visibility = ray_marching(fpx, dir, light_pos_ss, light_range_ss, screen_size);
            if visibility > 0.01 {
                visibility *= 1. - saturate(
                    (distance(fpx, light_pos_ss) - light_radius_ss) / (light_range_ss - light_radius_ss)
                );
                color += visibility * visibility * (*light).intensity * (*light).color.rgb;
            }
        }
    }
    
    return textureSample(main_tex, main_tex_sampler, in.uv)
           * vec4f(ambient_light.color.rgb * ambient_light.intensity, 1.)
           + vec4f(color, 0.);
}
