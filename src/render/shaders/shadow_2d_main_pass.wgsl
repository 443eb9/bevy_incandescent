#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::view::View
#import bevy_incandescent::shadow_2d_types::ShadowMapMeta

struct PointLight {
    position_ndc: vec2f,
    range_ndc: vec2f,
    radius_ndc: vec2f,
    color: vec4f,
}

@group(0) @binding(0)
var main_tex: texture_2d<f32>;

@group(0) @binding(1)
var main_tex_sampler: sampler;

@group(0) @binding(2)
var shadow_map: texture_storage_2d_array<rg32float, read>;

@group(0) @binding(3)
var<uniform> main_view: View;

@group(0) @binding(4)
var<uniform> shadow_map_meta: ShadowMapMeta;

@group(0) @binding(5)
var<storage> point_lights: array<PointLight>;

fn get_caster_distance_h(sample_ndc: vec2f, i_light: u32) -> f32 {
    let v0 = (sample_ndc.y / abs(sample_ndc.x) + 1.) / 2.;
    let px = vec2f(2., f32(shadow_map_meta.size)) * vec2f(sample_ndc.x / 2. + 1., v0);
    return textureLoad(shadow_map, vec2i(px), i_light).r;
}

fn get_caster_distance_v(sample_ndc: vec2f, i_light: u32) -> f32 {
    let v0 = (sample_ndc.x / abs(sample_ndc.y) + 1.) / 2.;
    let px = vec2f(2., f32(shadow_map_meta.size)) * vec2f(sample_ndc.y / 2. + 1., v0);
    return textureLoad(shadow_map, vec2i(px), i_light).g;
}

fn is_inside(p: vec2f, o: vec2f, a: f32, b: f32) -> bool {
    let p0 = p - o;
    return p0.x * p0.x / a / a + p0.y * p0.y / b / b < 1.;
}

fn intersect(p: vec2f, o: vec2f, a: f32, b: f32) -> vec2f {
    let p0 = p - o;
    let t = (a * b) / sqrt(a * a * p0.y * p0.y + b * b * p0.x * p0.x);
    return vec2f(p0.x * t, p0.y * t);
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4f {
    // let scale = 512u >> 0u;
    // let px = vec2f(in.uv.x, 1. - in.uv.y) * vec2f(f32(scale), 512.);
    // let color = pow(textureLoad(shadow_map, vec2i(px), 0), vec4f(2.2));
    // // return vec4f(color.r, 0., 0., 1.);
    // return color;

    // return vec4f(px / 512., 0., 1.);

    // return textureSample(main_tex, main_tex_sampler, in.uv);
    // return textureLoad(shadow_map, vec2i(in.uv * vec2f(512., 512.)), 0);
    // return vec4f(f32(arrayLength(point_lights)), 1., 1., 1.);
    // return point_lights[1].color;

    // var color = textureSample(main_tex, main_tex_sampler, in.uv);
    // let aspect = main_view.projection[1][1] / main_view.projection[0][0];
    // var uv_ndc = in.uv * 2. - 1.;
    // uv_ndc.y = -uv_ndc.y;

    // for (var i_light: u32 = 0; i_light < arrayLength(&point_lights); i_light++) {
    //     let light = &point_lights[i_light];
    //     // Orthographic projection, no need to divide by w
    //     var min_ndc_pos = (main_view.view_proj * (*light).min_world_pos).xy;
    //     var max_ndc_pos = (main_view.view_proj * (*light).max_world_pos).xy;
    //     let area = max_ndc_pos - min_ndc_pos;

    //     if min_ndc_pos.x < uv_ndc.x && max_ndc_pos.x > uv_ndc.x
    //        && min_ndc_pos.y < uv_ndc.y && max_ndc_pos.y > uv_ndc.y {

    //         let light_ndc = (min_ndc_pos + max_ndc_pos) / 2.;
    //         var sample_ndc = (uv_ndc - light_ndc) / area / 2.;
    //         sample_ndc.y = -sample_ndc.y;

    //         var caster_dist = 0.;
    //         if abs(sample_ndc.y) < abs(sample_ndc.x) {
    //             caster_dist = get_caster_distance_h(sample_ndc, i_light);
    //         } else {
    //             caster_dist = get_caster_distance_v(sample_ndc, i_light);
    //         }

    //         if caster_dist > length(sample_ndc) {
    //             color += (*light).color;
    //         }
    //     }
    // }

    // return color;

    var ndc = in.uv * 2. - 1.;
    ndc.y = -ndc.y;

    var color = textureSample(main_tex, main_tex_sampler, in.uv);
    for (var i_light: u32 = 0; i_light < arrayLength(&point_lights); i_light++) {
        let light = &point_lights[i_light];
        let light_range_ndc = (*light).range_ndc;
        let light_radius_ndc = (*light).radius_ndc;

        if is_inside(ndc, (*light).position_ndc, light_range_ndc.x, light_range_ndc.y) {
            let rel_ndc = ndc - (*light).position_ndc;
            // TODO because the size of projection matrix is doubled, we need to divide by 4
            let sample_ndc = rel_ndc / light_range_ndc / 4.;

            var caster_dist = 0.;
            if abs(sample_ndc.y) < abs(sample_ndc.x) {
                caster_dist = get_caster_distance_h(sample_ndc, i_light);
            } else {
                caster_dist = get_caster_distance_v(sample_ndc, i_light);
            }

            if caster_dist > length(sample_ndc) {
                var atten = 1.;
                if light_radius_ndc.x <= 0. || light_radius_ndc.y <= 0. {
                    atten -= length(rel_ndc / light_range_ndc);
                } else if !is_inside(ndc, (*light).position_ndc, light_radius_ndc.x, light_radius_ndc.y) {
                    let itsec_min = intersect(rel_ndc, vec2f(0.), light_radius_ndc.x, light_radius_ndc.y);
                    let itsec_max = intersect(rel_ndc, vec2f(0.), light_range_ndc.x, light_range_ndc.y);
                    atten -= length(itsec_min - rel_ndc) / length(itsec_max - itsec_min);
                }
                let light_color = (*light).color * atten;
                color += vec4f(pow(light_color.rgb, vec3f(2.2)), light_color.a);
            }
        }
    }

    return color;
}
