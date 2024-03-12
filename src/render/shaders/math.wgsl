#define_import_path bevy_incandescent::math

const PI: f32 = 3.14159265358979323846;
const TAU: f32 = 6.28318530717958647692;
const FRAC_PI_2: f32 = 1.57079632679489661923;

// https://www.cnblogs.com/miloyip/archive/2013/04/19/3029852.html
fn is_point_inside_sector(
    point: vec2f,
    center: vec2f,
    radius: f32,
    angles: array<f32, 2>,
) -> bool {
    if angles[1] >= TAU {
        return true;
    }

    let p0 = point - center;
    let sqr_dist = p0.x * p0.x + p0.y * p0.y;
    if sqr_dist > radius * radius {
        return false;
    }

    let ctr = vec2f(cos(angles[0]), sin(angles[0]));
    let cos_ext = cos(angles[1]);
    let d = dot(p0, ctr);

    if d >= 0. && cos_ext >= 0. {
        return d * d > sqr_dist * cos_ext * cos_ext;
    } else if d < 0. && cos_ext < 0. {
        return d * d < sqr_dist * cos_ext * cos_ext;
    } else {
        return d >= 0.;
    }
}
