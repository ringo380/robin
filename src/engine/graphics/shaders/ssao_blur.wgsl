// SSAO Blur Compute Shader
// Applies bilateral blur to reduce noise while preserving edges

@group(0) @binding(0)
var ssao_input: texture_2d<f32>;

@group(0) @binding(1)
var ssao_output: texture_storage_2d<r32float, write>;

// Bilateral blur kernel to preserve edges
const BLUR_RADIUS: i32 = 2;
const BLUR_SAMPLES: array<vec2<i32>, 25> = array<vec2<i32>, 25>(
    vec2<i32>(-2, -2), vec2<i32>(-1, -2), vec2<i32>(0, -2), vec2<i32>(1, -2), vec2<i32>(2, -2),
    vec2<i32>(-2, -1), vec2<i32>(-1, -1), vec2<i32>(0, -1), vec2<i32>(1, -1), vec2<i32>(2, -1),
    vec2<i32>(-2,  0), vec2<i32>(-1,  0), vec2<i32>(0,  0), vec2<i32>(1,  0), vec2<i32>(2,  0),
    vec2<i32>(-2,  1), vec2<i32>(-1,  1), vec2<i32>(0,  1), vec2<i32>(1,  1), vec2<i32>(2,  1),
    vec2<i32>(-2,  2), vec2<i32>(-1,  2), vec2<i32>(0,  2), vec2<i32>(1,  2), vec2<i32>(2,  2),
);

// Gaussian weights for blur kernel
const BLUR_WEIGHTS: array<f32, 25> = array<f32, 25>(
    0.003765, 0.015019, 0.023792, 0.015019, 0.003765,
    0.015019, 0.059912, 0.094907, 0.059912, 0.015019,
    0.023792, 0.094907, 0.150342, 0.094907, 0.023792,
    0.015019, 0.059912, 0.094907, 0.059912, 0.015019,
    0.003765, 0.015019, 0.023792, 0.015019, 0.003765,
);

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pixel_coords = vec2<i32>(global_id.xy);
    let texture_size = textureDimensions(ssao_input);

    // Bounds check
    if (pixel_coords.x >= i32(texture_size.x) || pixel_coords.y >= i32(texture_size.y)) {
        return;
    }

    let center_ao = textureLoad(ssao_input, pixel_coords, 0).r;

    // Bilateral blur - weight samples based on AO similarity
    var total_weight = 0.0;
    var blurred_ao = 0.0;

    for (var i = 0; i < 25; i++) {
        let sample_coords = pixel_coords + BLUR_SAMPLES[i];

        // Bounds check for sample
        if (sample_coords.x < 0 || sample_coords.x >= i32(texture_size.x) ||
            sample_coords.y < 0 || sample_coords.y >= i32(texture_size.y)) {
            continue;
        }

        let sample_ao = textureLoad(ssao_input, sample_coords, 0).r;

        // Bilateral weight based on AO difference
        let ao_diff = abs(center_ao - sample_ao);
        let bilateral_weight = exp(-ao_diff * ao_diff * 10.0); // Preserve edges

        // Combine spatial and bilateral weights
        let weight = BLUR_WEIGHTS[i] * bilateral_weight;

        blurred_ao += sample_ao * weight;
        total_weight += weight;
    }

    // Normalize result
    if (total_weight > 0.0) {
        blurred_ao /= total_weight;
    } else {
        blurred_ao = center_ao; // Fallback to original value
    }

    // Store result
    textureStore(ssao_output, pixel_coords, vec4<f32>(blurred_ao));
}