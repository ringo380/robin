// Temporal Upsampling Compute Shader
// Improves volumetric quality through temporal accumulation and upsampling

struct VolumetricUniforms {
    view_matrix: mat4x4<f32>,
    proj_matrix: mat4x4<f32>,
    inv_view_proj_matrix: mat4x4<f32>,
    camera_position: vec3<f32>,
    _padding1: f32,
    fog_density: f32,
    fog_color: vec3<f32>,
    scattering_coefficient: f32,
    absorption_coefficient: f32,
    phase_function_g: f32,
    ray_marching_steps: u32,
    frame_index: u32,
    volume_resolution: vec3<f32>,
    _padding2: f32,
    z_near: f32,
    z_far: f32,
    time: f32,
    _padding3: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: VolumetricUniforms;

@group(0) @binding(2)
var volume_output: texture_storage_3d<rgba16float, write>;

@group(0) @binding(3)
var volume_history: texture_3d<f32>;

@group(0) @binding(6)
var volume_sampler: sampler;

// Blue noise for temporal dithering
fn blue_noise_3d(coord: vec3<f32>, frame: f32) -> vec3<f32> {
    let p = coord + frame * 0.618034;
    return fract(sin(vec3<f32>(
        dot(p, vec3<f32>(12.9898, 78.233, 37.719)),
        dot(p, vec3<f32>(39.346, 11.135, 83.155)),
        dot(p, vec3<f32>(73.156, 52.235, 91.756))
    )) * 43758.5453);
}

// Catmull-Rom interpolation for smooth upsampling
fn catmull_rom_weight(d: f32) -> f32 {
    let d2 = d * d;
    let d3 = d2 * d;

    if (d <= 1.0) {
        return 1.5 * d3 - 2.5 * d2 + 1.0;
    } else if (d <= 2.0) {
        return -0.5 * d3 + 2.5 * d2 - 4.0 * d + 2.0;
    }
    return 0.0;
}

// Sample volume with Catmull-Rom interpolation
fn sample_volume_catmull_rom(
    volume_tex: texture_3d<f32>,
    samp: sampler,
    coord: vec3<f32>
) -> vec4<f32> {
    let volume_size = vec3<f32>(uniforms.volume_resolution);
    let scaled_coord = coord * volume_size - 0.5;
    let base_coord = floor(scaled_coord);
    let frac_coord = scaled_coord - base_coord;

    var result = vec4<f32>(0.0);
    var weight_sum = 0.0;

    // 4x4x4 Catmull-Rom kernel
    for (var z = -1; z <= 2; z++) {
        for (var y = -1; y <= 2; y++) {
            for (var x = -1; x <= 2; x++) {
                let sample_coord = (base_coord + vec3<f32>(f32(x), f32(y), f32(z)) + 0.5) / volume_size;

                if (all(sample_coord >= vec3<f32>(0.0)) && all(sample_coord <= vec3<f32>(1.0))) {
                    let weight = catmull_rom_weight(abs(frac_coord.x - f32(x))) *
                                catmull_rom_weight(abs(frac_coord.y - f32(y))) *
                                catmull_rom_weight(abs(frac_coord.z - f32(z)));

                    if (weight > 0.0) {
                        let sample_value = textureSampleLevel(volume_tex, samp, sample_coord, 0.0);
                        result += sample_value * weight;
                        weight_sum += weight;
                    }
                }
            }
        }
    }

    return select(vec4<f32>(0.0), result / weight_sum, weight_sum > 0.0);
}

// Motion vector calculation for temporal reprojection
fn calculate_motion_vector(world_pos: vec3<f32>, prev_view_proj: mat4x4<f32>) -> vec2<f32> {
    // Current screen position
    let current_clip = uniforms.proj_matrix * uniforms.view_matrix * vec4<f32>(world_pos, 1.0);
    let current_screen = current_clip.xy / current_clip.w;

    // Previous screen position (simplified - assumes static camera for now)
    let prev_clip = prev_view_proj * vec4<f32>(world_pos, 1.0);
    let prev_screen = prev_clip.xy / prev_clip.w;

    return current_screen - prev_screen;
}

// Neighborhood clamping for temporal stability
fn clamp_to_neighborhood(
    current_sample: vec4<f32>,
    history_sample: vec4<f32>,
    coord: vec3<f32>
) -> vec4<f32> {
    var neighborhood_min = vec4<f32>(1000.0);
    var neighborhood_max = vec4<f32>(-1000.0);
    var neighborhood_mean = vec4<f32>(0.0);
    var sample_count = 0.0;

    let volume_size = vec3<f32>(uniforms.volume_resolution);
    let texel_size = 1.0 / volume_size;

    // Sample 3x3x3 neighborhood
    for (var z = -1; z <= 1; z++) {
        for (var y = -1; y <= 1; y++) {
            for (var x = -1; x <= 1; x++) {
                let offset_coord = coord + vec3<f32>(f32(x), f32(y), f32(z)) * texel_size;

                if (all(offset_coord >= vec3<f32>(0.0)) && all(offset_coord <= vec3<f32>(1.0))) {
                    let sample = textureSampleLevel(volume_history, volume_sampler, offset_coord, 0.0);
                    neighborhood_min = min(neighborhood_min, sample);
                    neighborhood_max = max(neighborhood_max, sample);
                    neighborhood_mean += sample;
                    sample_count += 1.0;
                }
            }
        }
    }

    if (sample_count > 0.0) {
        neighborhood_mean /= sample_count;

        // Expand bounding box slightly for stability
        let variance = (neighborhood_max - neighborhood_min) * 0.5;
        neighborhood_min -= variance * 0.1;
        neighborhood_max += variance * 0.1;

        // Clamp history sample to neighborhood
        return clamp(history_sample, neighborhood_min, neighborhood_max);
    }

    return history_sample;
}

// Convert 3D texture coordinates to world position
fn texture_coord_to_world_pos(tex_coord: vec3<f32>) -> vec3<f32> {
    let ndc = tex_coord * 2.0 - 1.0;
    let linear_depth = tex_coord.z;
    let exp_depth = mix(uniforms.z_near, uniforms.z_far, linear_depth);

    let view_pos = uniforms.inv_view_proj_matrix * vec4<f32>(ndc.x, ndc.y, exp_depth, 1.0);
    return view_pos.xyz / view_pos.w;
}

@compute @workgroup_size(8, 8, 4)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let volume_coord = vec3<i32>(global_id);
    let volume_size = vec3<i32>(uniforms.volume_resolution);

    // Bounds check
    if (volume_coord.x >= volume_size.x ||
        volume_coord.y >= volume_size.y ||
        volume_coord.z >= volume_size.z) {
        return;
    }

    let tex_coord = vec3<f32>(volume_coord) / vec3<f32>(volume_size);

    // Sample current frame
    let current_sample = textureSampleLevel(volume_history, volume_sampler, tex_coord, 0.0);

    // Early exit if no history
    if (uniforms.frame_index == 0u) {
        textureStore(volume_output, volume_coord, current_sample);
        return;
    }

    // Calculate world position
    let world_pos = texture_coord_to_world_pos(tex_coord);

    // Apply temporal jittering for sub-voxel sampling
    let blue_noise = blue_noise_3d(vec3<f32>(volume_coord), f32(uniforms.frame_index));
    let jitter_offset = (blue_noise - 0.5) / vec3<f32>(volume_size);
    let jittered_coord = clamp(tex_coord + jitter_offset, vec3<f32>(0.0), vec3<f32>(1.0));

    // Sample with high-quality interpolation
    let upsampled_current = sample_volume_catmull_rom(volume_history, volume_sampler, jittered_coord);

    // Sample history at same location (simplified - no motion vectors for now)
    let history_sample = textureSampleLevel(volume_history, volume_sampler, tex_coord, 0.0);

    // Apply neighborhood clamping
    let clamped_history = clamp_to_neighborhood(upsampled_current, history_sample, tex_coord);

    // Calculate temporal blend weight based on confidence
    let volume_difference = length(upsampled_current - clamped_history);
    let confidence = exp(-volume_difference * 10.0);
    let temporal_weight = mix(0.1, 0.95, confidence);

    // Temporal accumulation
    let final_result = mix(upsampled_current, clamped_history, temporal_weight);

    // Add subtle temporal dithering to reduce banding
    let dither_strength = 0.005;
    let dither = (blue_noise.x - 0.5) * dither_strength;
    let dithered_result = final_result + vec4<f32>(dither, dither, dither, 0.0);

    textureStore(volume_output, volume_coord, dithered_result);
}