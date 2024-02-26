#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::view::View
#import bevy_incandescent::shadow_2d_types::ShadowMapMeta

struct PointLight {
    min_world_pos: vec4f,
    max_world_pos: vec4f,
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

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4f {
    // let scale = 512u >> 8u;
    // let px = in.uv * vec2f(f32(scale), 512.);
    // let color = pow(textureLoad(shadow_map, vec2i(px), 0), vec4f(2.2));
    // return vec4f(color.r, 0., 0., 1.);
    // return color;

    // return vec4f(px / 512., 0., 1.);

    // return textureSample(main_tex, main_tex_sampler, in.uv);
    // return textureLoad(shadow_map, vec2i(in.uv * vec2f(512., 512.)), 0);
    // return vec4f(f32(arrayLength(point_lights)), 1., 1., 1.);
    // return point_lights[1].color;

    var color = textureSample(main_tex, main_tex_sampler, in.uv);
    let aspect = main_view.projection[1][1] / main_view.projection[0][0];
    var uv_ndc = in.uv * 2. - 1.;
    uv_ndc.y = -uv_ndc.y;

    for (var i_light: u32 = 0; i_light < arrayLength(&point_lights); i_light++) {
        let light = &point_lights[i_light];
        // Orthographic projection, no need to divide by w
        var min_ndc_pos = (main_view.view_proj * (*light).min_world_pos).xy;
        var max_ndc_pos = (main_view.view_proj * (*light).max_world_pos).xy;
        let area = max_ndc_pos - min_ndc_pos;

        if min_ndc_pos.x < uv_ndc.x && max_ndc_pos.x > uv_ndc.x
           && min_ndc_pos.y < uv_ndc.y && max_ndc_pos.y > uv_ndc.y {

            let light_ndc = (min_ndc_pos + max_ndc_pos) / 2.;
            var sample_ndc = (uv_ndc - light_ndc) / area / 2.;
            sample_ndc.y = -sample_ndc.y;

            var caster_dist = 0.;
            if abs(sample_ndc.y) < abs(sample_ndc.x) {
                caster_dist = get_caster_distance_h(sample_ndc, i_light);
            } else {
                caster_dist = get_caster_distance_v(sample_ndc, i_light);
            }

            if caster_dist > length(sample_ndc) {
                color += (*light).color;
            }
        }
    }

    return color;
}
