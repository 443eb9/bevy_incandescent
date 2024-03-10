#define_import_path bevy_incandescent::math

const PI: f32 = 3.14159265358979323846;
const FRAC_PI_2: f32 = 1.57079632679489661923;

fn is_point_inside_sector(
    point: vec2f,
    center: vec2f,
    radius: f32,
    angles: array<f32, 2>,
) -> bool {
    let p0 = point - center;
    let sqr_dist = p0.x * p0.x + p0.y * p0.y;
    if sqr_dist > radius * radius {
        return false;
    }
    var pol = atan(p0.y / p0.x);
    if p0.x < 0.0 {
        pol += PI;
    }
    return pol > angles[0] && pol < angles[1];
}
