#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::view::View
#import bevy_incandescent::{
    ray_marching::shadow_2d_types::SdfMeta,
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

fn ray_marching(px: vec2f, dir: vec2f, center: vec2f, radius: f32) -> bool {
    var current = px;
    let tex_fsize = vec2f(sdf_meta.size);

    while current.x > 0. && current.x < tex_fsize.x
          && current.y > 0. && current.y < tex_fsize.y {
        let sdf_data = textureLoad(sdf_tex, vec2i(current)).rg;
        let closest = sdf_data.r;
        if closest < 0.1 {
            return false;
        }
        current += dir * min(closest, distance(current, center));
        if distance(current, center) < 0.1 {
            return true;
        }
    }
    return true;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4f {
    let screen_size = 2. * vec2f(main_view.inverse_projection[0][0], main_view.inverse_projection[1][1]);
    let px = vec2i(in.uv * vec2f(screen_size));

    var color = vec3f(0.);
    for (var i_light = 0u; i_light < arrayLength(&point_lights); i_light++) {
        let light = &point_lights[i_light];
        let light_range_ss = max((*light).range_ss * screen_size.x, 0.);
        let light_radius_ss = max((*light).radius_ss * screen_size.x, 0.);
        let light_pos_ss = (*light).position_ss * screen_size;
        let dir = normalize(light_pos_ss - vec2f(px));

        if is_point_inside_sector(vec2f(px), light_pos_ss, light_range_ss, (*light).angles) {
            if ray_marching(vec2f(px), dir, light_pos_ss, light_range_ss) {
                let atten = saturate(
                    (distance(vec2f(px), light_pos_ss) - light_radius_ss) / (light_range_ss - light_radius_ss)
                );
                color += (*light).color.rgb * (1. - atten);
            }
        }
    }
    
    return textureSample(main_tex, main_tex_sampler, in.uv)
           * vec4f(ambient_light.color.rgb * ambient_light.intensity, 1.)
           + vec4f(color, 0.);
}
