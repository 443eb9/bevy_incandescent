#define_import_path bevy_incandescent::lighting

// The same as bevy_pbr::src::render::pbr_lighting
fn get_distance_attenuation(distance_square: f32, inverse_range_squared: f32) -> f32 {
    let factor = distance_square * inverse_range_squared;
    let smooth_factor = saturate(1.0 - factor * factor);
    let attenuation = smooth_factor * smooth_factor;
    return attenuation * 1.0 / max(distance_square, 0.0001);
}
