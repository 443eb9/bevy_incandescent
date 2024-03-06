#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::view::View
#import bevy_incandescent::{
    lighting::get_distance_attenuation,
    shadow_2d_types::{AmbientLight2d, PointLight2d, ShadowMapMeta}
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
    return textureLoad(shadow_map, vec2i(px), i_light).r;
}

fn get_caster_distance_v(rel_ss: vec2f, i_light: u32) -> f32 {
    let v0 = (rel_ss.x / abs(rel_ss.y) + 1.) / 2.;
    let px = vec2f(2., f32(shadow_map_meta.size)) * vec2f(rel_ss.y / 2. + 1., v0);
    return textureLoad(shadow_map, vec2i(px), i_light).g;
}

fn get_caster_distance(rel_ss: vec2f, i_light: u32) -> f32 {
    // Because the proj mat is doubled, here the dist should also be doubled
    if abs(rel_ss.y) < abs(rel_ss.x) {
        return get_caster_distance_h(rel_ss, i_light) * 2.;
    } else {
        return get_caster_distance_v(rel_ss, i_light) * 2.;
    }
}

fn pcf(rel_ss: vec2f, sample_count: u32, sample_radius: f32, i_light: u32) -> f32 {
    var POISSON_DISK: array<vec2f, 32> = array<vec2f, 32>(
        vec2f(0.8342270007200882,0.28814735667820845),
        vec2f(0.8264199719660169,0.1090369643376499),
        vec2f(0.6784681277016904,0.36535885311678634),
        vec2f(0.7381973795416519,0.22695634447974777),
        vec2f(0.7702680023357095,0.4722784529353279),
        vec2f(0.9351229206585461,0.16905050325597692),
        vec2f(0.8540762434176998,0.3917249981922568),
        vec2f(0.6003790707176412,0.13589749953695904),
        vec2f(0.7140843172626038,0.03457204051621679),
        vec2f(0.5260926483017618,0.3944808363265693),
        vec2f(0.645610498490519,0.5349442850139658),
        vec2f(0.6066682573717601,0.2878050901544499),
        vec2f(0.4538111047369613,0.18876124547827403),
        vec2f(0.48079338169183666,0.2890199911997478),
        vec2f(0.9916624489016049,0.046454018661305704),
        vec2f(0.5983695471298044,0.027983816213303994),
        vec2f(0.4077904101544584,0.09934487505973388),
        vec2f(0.20901604650710173,0.1043784189081576),
        vec2f(0.2666233350480739,0.004601707618502468),
        vec2f(0.2656651572591358,0.18996114777855283),
        vec2f(0.37877176238082616,0.26500770719269534),
        vec2f(0.8062043548051135,0.5682004999710749),
        vec2f(0.4592241169624123,0.548029806967463),
        vec2f(0.6137166995912418,0.6632439118229136),
        vec2f(0.10904976594891624,0.16744767716904238),
        vec2f(0.25558191979309847,0.326757155507577),
        vec2f(0.12420603976718372,0.30352975263809234),
        vec2f(0.5050651579251138,0.7027035802838815),
        vec2f(0.11555416536095653,0.04112114851879525),
        vec2f(0.005179508593920384,0.24291966974019186),
        vec2f(0.009549777408563165,0.047482323406035754),
        vec2f(0.32312849030711477,0.7701802894233629),
    );
    
    var visibility = 0.;
    for (var i: u32 = 0; i < sample_count; i++) {
        let sample_ss = rel_ss + POISSON_DISK[i] * sample_radius;
        let dist = get_caster_distance(sample_ss, i_light);
        if dist > length(sample_ss) + shadow_map_meta.bias {
            visibility += 1.;
        }
    }
    visibility /= f32(sample_count);
    return visibility;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4f {
    // let scale = 512u >> 8u;
    // let px = vec2f(in.uv.x, 1. - in.uv.y) * vec2f(f32(scale), 512.);
    // let color = pow(textureLoad(shadow_map, vec2i(px), 0), vec4f(2.2));
    // // return vec4f(color.r, 0., 0., 1.);
    // return color;

    // return vec4f(px / 512., 0., 1.);

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
