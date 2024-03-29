#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::view::View
#import bevy_incandescent::{
    catalinzz::types::ShadowMapMeta,
    lighting::get_distance_attenuation,
    math::{is_point_inside_sector},
    types::{AmbientLight2d, PointLight2d},
}

@group(0) @binding(0)
var main_tex: texture_2d<f32>;

@group(0) @binding(1)
var main_tex_sampler: sampler;

@group(0) @binding(2)
var alpha_map: texture_2d<f32>;

@group(0) @binding(3)
var alpha_map_sampler: sampler;

@group(0) @binding(4)
var shadow_map: texture_storage_2d_array<
#ifdef COMPATIBILITY
    rgba32float,
#else
    rg32float,
#endif
    read
>;

@group(0) @binding(5)
var<uniform> main_view: View;

@group(0) @binding(6)
var<uniform> shadow_map_meta: ShadowMapMeta;

@group(0) @binding(7)
var<uniform> ambient_light: AmbientLight2d;

@group(0) @binding(8)
var<storage> poisson_disk: array<vec2f>;

@group(0) @binding(9)
var<storage> point_lights: array<PointLight2d>;

fn get_caster_distance_h(rel_ss: vec2f, i_light: u32) -> f32 {
    let v0 = (rel_ss.y / abs(rel_ss.x) + 1.) / 2.;
    let px = vec2f(2., f32(shadow_map_meta.size)) * vec2f(rel_ss.x / 2. + 1., v0);
    return textureLoad(shadow_map, vec2i(px), i_light).r * 2.;
}

fn get_caster_distance_v(rel_ss: vec2f, i_light: u32) -> f32 {
    let v0 = (rel_ss.x / abs(rel_ss.y) + 1.) / 2.;
    let px = vec2f(2., f32(shadow_map_meta.size)) * vec2f(rel_ss.y / 2. + 1., v0);
    return textureLoad(shadow_map, vec2i(px), i_light).g * 2.;
}

fn get_caster_distance(rel_ss: vec2f, i_light: u32) -> f32 {
    if abs(rel_ss.y) < abs(rel_ss.x) {
        return get_caster_distance_h(rel_ss, i_light);
    } else {
        return get_caster_distance_v(rel_ss, i_light);
    }
}

fn pcf(rel_ss: vec2f, sample_radius: f32, i_light: u32) -> f32 {
    var visibility = 0.;
    for (var i: u32 = 0; i < shadow_map_meta.pcf_samples; i++) {
        let sample_ss = rel_ss + poisson_disk[i] * sample_radius;
        let dist = get_caster_distance(sample_ss, i_light);
        
        if dist > length(sample_ss) - shadow_map_meta.bias {
            visibility += 1.;
        }
    }
    visibility /= f32(shadow_map_meta.pcf_samples);
    return visibility;
}

fn get_alpha(uv: vec2f, i_light: u32) -> f32 {
    return textureSample(alpha_map, alpha_map_sampler, uv).a;
}

@fragment
fn dbg_output_shadow_map(in: FullscreenVertexOutput) -> @location(0) vec4f {
    return textureLoad(shadow_map, vec2u(in.uv * vec2f(shadow_map_meta.size)), 0);
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4f {
    let screen_size = 2. * vec2f(main_view.inverse_projection[0][0], main_view.inverse_projection[1][1]);
    let px = in.uv * screen_size;

    var color = vec3f(0.);
    for (var i_light = 0u; i_light < arrayLength(&point_lights); i_light++) {
        let light = &point_lights[i_light];
        let light_pos_ss = (*light).position_ss * screen_size;
        let light_range_ss = max((*light).range_ss, 0.) * screen_size.x;
        let light_radius_ss = max((*light).radius_ss, 0.) * screen_size.x;
        let light_color = (*light).color;

        let rel_px_ss = px - light_pos_ss + shadow_map_meta.offset;
        let rel_px_dist = length(rel_px_ss);
        let rel_ss = rel_px_ss / light_range_ss;
        let rel_dist = length(rel_ss);
        let pcf_radius_rel = shadow_map_meta.pcf_radius / light_range_ss;

        if is_point_inside_sector(rel_px_ss * vec2f(1., -1.), vec2f(0.), light_range_ss, (*light).angles) {
            if get_alpha(in.uv, i_light) > shadow_map_meta.alpha_threshold {
                continue;
            }

            var visibility = pcf(rel_ss, pcf_radius_rel, i_light);
            visibility *= 1. - saturate(
                (rel_px_dist - light_radius_ss) / (light_range_ss - light_radius_ss)
            );
            let attend_color = visibility * visibility * (*light).intensity * light_color.rgb;
            color += attend_color;
        }
    }

    return textureSample(main_tex, main_tex_sampler, in.uv)
           * vec4f(ambient_light.color.rgb * ambient_light.intensity, 1.)
           + vec4f(color, 0.);
}
