// Shadow Compute Shaders for filtering

struct ShadowData {
    depth: f32,
    variance: f32,
}

@group(0) @binding(0) var shadow_texture: texture_2d<f32>;
@group(0) @binding(1) var shadow_sampler: sampler;
@group(0) @binding(2) var<storage, read_write> shadow_results: array<f32>;

// PCF (Percentage Closer Filtering) for soft shadows
@compute @workgroup_size(8, 8)
fn pcf_filter(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let coord = vec2<i32>(global_id.xy);
    let texture_size = textureDimensions(shadow_texture);

    if (coord.x >= i32(texture_size.x) || coord.y >= i32(texture_size.y)) {
        return;
    }

    let texel_size = 1.0 / vec2<f32>(texture_size);
    let base_coord = vec2<f32>(coord) * texel_size;

    var shadow_sum = 0.0;
    let filter_size = 3;
    let half_filter = filter_size / 2;

    // Sample surrounding texels for PCF
    for (var x = -half_filter; x <= half_filter; x++) {
        for (var y = -half_filter; y <= half_filter; y++) {
            let sample_coord = base_coord + vec2<f32>(f32(x), f32(y)) * texel_size;
            let depth_sample = textureSample(shadow_texture, shadow_sampler, sample_coord).r;
            shadow_sum += depth_sample;
        }
    }

    let average_depth = shadow_sum / f32(filter_size * filter_size);
    let linear_index = coord.y * i32(texture_size.x) + coord.x;
    shadow_results[linear_index] = average_depth;
}

// Variance Shadow Mapping for softer shadows
@compute @workgroup_size(8, 8)
fn variance_filter(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let coord = vec2<i32>(global_id.xy);
    let texture_size = textureDimensions(shadow_texture);

    if (coord.x >= i32(texture_size.x) || coord.y >= i32(texture_size.y)) {
        return;
    }

    let texel_size = 1.0 / vec2<f32>(texture_size);
    let base_coord = vec2<f32>(coord) * texel_size;

    // Sample depth and calculate variance
    let depth = textureSample(shadow_texture, shadow_sampler, base_coord).r;
    let depth_squared = depth * depth;

    // Simple variance calculation (in practice would use gaussian blur)
    var variance_sum = 0.0;
    let kernel_size = 3;
    let half_kernel = kernel_size / 2;

    for (var x = -half_kernel; x <= half_kernel; x++) {
        for (var y = -half_kernel; y <= half_kernel; y++) {
            let sample_coord = base_coord + vec2<f32>(f32(x), f32(y)) * texel_size;
            let sample_depth = textureSample(shadow_texture, shadow_sampler, sample_coord).r;
            let diff = sample_depth - depth;
            variance_sum += diff * diff;
        }
    }

    let variance = variance_sum / f32(kernel_size * kernel_size);
    let linear_index = coord.y * i32(texture_size.x) + coord.x;
    shadow_results[linear_index] = variance;
}