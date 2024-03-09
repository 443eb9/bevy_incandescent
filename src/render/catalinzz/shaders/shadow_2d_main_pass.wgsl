#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::view::View
#import bevy_incandescent::{
    hash::hash23,
    lighting::get_distance_attenuation,
    catalinzz::shadow_2d_types::{AmbientLight2d, PointLight2d, ShadowMapMeta}
}

@group(0) @binding(0)
var main_tex: texture_2d<f32>;

@group(0) @binding(1)
var main_tex_sampler: sampler;

@group(0) @binding(2)
var shadow_map: texture_storage_2d_array<
#ifdef COMPATIBILITY
    rgba32float,
#else
    rg32float,
#endif
    read
>;

@group(0) @binding(3)
var<uniform> main_view: View;

@group(0) @binding(4)
var<uniform> shadow_map_meta: ShadowMapMeta;

@group(0) @binding(5)
var<uniform> ambient_light: AmbientLight2d;

@group(0) @binding(6)
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

fn pcf(rel_ss: vec2f, sample_count: u32, sample_radius: f32, i_light: u32) -> f32 {
    var visibility = 0.;
    for (var i: u32 = 0; i < sample_count; i++) {
        let hash = hash23(vec3f(rel_ss, f32(i)) * 10.);
        let offset = vec2f(cos(hash.x * 6.28318531), sin(hash.x * 6.28318531)) * hash.y;
        let sample_ss = rel_ss + offset * sample_radius;
        let dist = get_caster_distance(sample_ss, i_light);
        
        if dist > length(sample_ss) + shadow_map_meta.bias {
            visibility += 1.;
        }
    }
    visibility /= f32(sample_count);
    return visibility;
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
    for (var i_light: u32 = 0; i_light < arrayLength(&point_lights); i_light++) {
        let light = &point_lights[i_light];
        let light_pos_ss = (*light).position_ss;
        let light_range_ss = max((*light).range_ss, 0.);
        let light_radius_ss = max((*light).radius_ss, 0.);
        let light_color = (*light).color;

        let rel_px_ss = px - light_pos_ss + shadow_map_meta.offset;
        let rel_px_dist = length(rel_px_ss);
        let rel_ss = rel_px_ss / light_range_ss;
        let rel_dist = length(rel_ss);
        let pcf_radius_rel = shadow_map_meta.pcf_radius / light_range_ss;

        if length(rel_px_ss) < light_range_ss {
            var visibility = pcf(rel_ss, shadow_map_meta.pcf_samples, pcf_radius_rel, i_light);
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
