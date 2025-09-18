// Temporal Anti-Aliasing (TAA) Compute Shader
// Implements temporal accumulation with motion vector reprojection

struct TAAUniforms {
    temporal_weight: f32,
    motion_threshold: f32,
    sharpness: f32,
    ghosting_reduction: f32,
    screen_size: vec2<f32>,
    jitter_offset: vec2<f32>,
    frame_index: u32,
    velocity_rejection: u32,
    luminance_weighting: u32,
    neighborhood_clamping: u32,
}

@group(0) @binding(0)
var current_color: texture_2d<f32>;

@group(0) @binding(1)
var history_color: texture_2d<f32>;

@group(0) @binding(2)
var motion_vectors: texture_2d<f32>;

@group(0) @binding(3)
var output_color: texture_storage_2d<rgba16float, write>;

@group(0) @binding(4)
var linear_sampler: sampler;

@group(0) @binding(5)
var<uniform> uniforms: TAAUniforms;

// Calculate luminance for perceptual weighting
fn rgb_to_luminance(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.299, 0.587, 0.114));
}

// Sample with catmull-rom bicubic filtering for high quality history sampling
fn sample_catmull_rom(tex: texture_2d<f32>, coord: vec2<f32>) -> vec4<f32> {
    let tex_size = vec2<f32>(textureDimensions(tex));
    let sample_pos = coord * tex_size - 0.5;
    let f = fract(sample_pos);
    let int_pos = floor(sample_pos);

    // Catmull-Rom weights
    let w0 = f * (-0.5 + f * (1.0 - 0.5 * f));
    let w1 = 1.0 + f * f * (-2.5 + 1.5 * f);
    let w2 = f * (0.5 + f * (2.0 - 1.5 * f));
    let w3 = f * f * (-0.5 + 0.5 * f);

    let w12 = w1 + w2;
    let offset12 = w2 / w12;

    let tex_pos0 = (int_pos + vec2<f32>(-1.0 + offset12)) / tex_size;
    let tex_pos3 = (int_pos + vec2<f32>(2.0 + offset12)) / tex_size;

    let sample0 = textureSampleLevel(tex, linear_sampler, tex_pos0, 0.0);
    let sample3 = textureSampleLevel(tex, linear_sampler, tex_pos3, 0.0);

    let lerp_factor = offset12 / (1.0 + offset12);
    return mix(sample0, sample3, lerp_factor);
}

// Neighborhood color clamping to reduce ghosting
fn clamp_to_neighborhood(history_color: vec3<f32>, pixel_coord: vec2<i32>) -> vec3<f32> {
    if (uniforms.neighborhood_clamping == 0u) {
        return history_color;
    }

    // Sample 3x3 neighborhood
    var min_color = vec3<f32>(1e10);
    var max_color = vec3<f32>(-1e10);

    for (var y = -1; y <= 1; y++) {
        for (var x = -1; x <= 1; x++) {
            let sample_coord = pixel_coord + vec2<i32>(x, y);
            let sample_color = textureLoad(current_color, sample_coord, 0).rgb;

            min_color = min(min_color, sample_color);
            max_color = max(max_color, sample_color);
        }
    }

    // Clamp history to neighborhood bounds
    return clamp(history_color, min_color, max_color);
}

// Calculate temporal weight based on motion and luminance
fn calculate_temporal_weight(
    current_luma: f32,
    history_luma: f32,
    motion_length: f32
) -> f32 {
    var weight = uniforms.temporal_weight;

    // Reduce weight for fast motion
    if (uniforms.velocity_rejection != 0u) {
        let motion_factor = exp(-motion_length * 10.0);
        weight *= motion_factor;
    }

    // Reduce weight for luminance differences
    if (uniforms.luminance_weighting != 0u) {
        let luma_diff = abs(current_luma - history_luma);
        let luma_factor = exp(-luma_diff * 5.0);
        weight *= luma_factor;
    }

    return weight;
}

// Apply sharpening filter to combat TAA blur
fn apply_sharpening(color: vec3<f32>, pixel_coord: vec2<i32>) -> vec3<f32> {
    if (uniforms.sharpness <= 0.0) {
        return color;
    }

    // 5-tap sharpening kernel
    let center = textureLoad(current_color, pixel_coord, 0).rgb;
    let up = textureLoad(current_color, pixel_coord + vec2<i32>(0, -1), 0).rgb;
    let down = textureLoad(current_color, pixel_coord + vec2<i32>(0, 1), 0).rgb;
    let left = textureLoad(current_color, pixel_coord + vec2<i32>(-1, 0), 0).rgb;
    let right = textureLoad(current_color, pixel_coord + vec2<i32>(1, 0), 0).rgb;

    let neighbors = (up + down + left + right) * 0.25;
    let sharpened = center + (center - neighbors) * uniforms.sharpness;

    return mix(color, sharpened, 0.5);
}

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pixel_coord = vec2<i32>(global_id.xy);
    let screen_size_i = vec2<i32>(uniforms.screen_size);

    // Bounds check
    if (pixel_coord.x >= screen_size_i.x || pixel_coord.y >= screen_size_i.y) {
        return;
    }

    let pixel_pos = vec2<f32>(pixel_coord);
    let uv = pixel_pos / uniforms.screen_size;

    // Sample current frame
    let current_sample = textureLoad(current_color, pixel_coord, 0);
    var final_color = current_sample.rgb;

    // Skip TAA on first frame or if disabled
    if (uniforms.frame_index == 0u) {
        textureStore(output_color, pixel_coord, vec4<f32>(final_color, current_sample.a));
        return;
    }

    // Sample motion vector
    let motion_vector = textureLoad(motion_vectors, pixel_coord, 0).rg;
    let motion_length = length(motion_vector);

    // Calculate history UV with motion compensation
    let history_uv = uv - motion_vector;

    // Check if history sample is valid (within screen bounds)
    if (history_uv.x < 0.0 || history_uv.x > 1.0 ||
        history_uv.y < 0.0 || history_uv.y > 1.0) {
        // History sample is off-screen, use current frame only
        final_color = apply_sharpening(final_color, pixel_coord);
        textureStore(output_color, pixel_coord, vec4<f32>(final_color, current_sample.a));
        return;
    }

    // Sample history color with high-quality filtering
    let history_sample = sample_catmull_rom(history_color, history_uv);
    var history_rgb = history_sample.rgb;

    // Apply neighborhood clamping to reduce ghosting
    history_rgb = clamp_to_neighborhood(history_rgb, pixel_coord);

    // Calculate luminance for weighting
    let current_luma = rgb_to_luminance(final_color);
    let history_luma = rgb_to_luminance(history_rgb);

    // Calculate temporal blending weight
    let temporal_weight = calculate_temporal_weight(current_luma, history_luma, motion_length);

    // Blend current and history
    final_color = mix(final_color, history_rgb, temporal_weight);

    // Apply anti-ghosting based on motion
    if (motion_length > uniforms.motion_threshold) {
        let ghosting_factor = min(motion_length / uniforms.motion_threshold, 1.0);
        let anti_ghost_weight = 1.0 - (ghosting_factor * uniforms.ghosting_reduction);
        final_color = mix(current_sample.rgb, final_color, anti_ghost_weight);
    }

    // Apply sharpening to combat TAA blur
    final_color = apply_sharpening(final_color, pixel_coord);

    // Store final result
    textureStore(output_color, pixel_coord, vec4<f32>(final_color, current_sample.a));
}