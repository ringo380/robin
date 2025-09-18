// Shadow Sampling Functions
// Provides PCF shadow sampling for use in main lighting shaders

struct ShadowUniforms {
    light_view_proj_matrices: array<mat4x4<f32>, 4>,
    cascade_splits: array<f32, 5>,
    light_direction: vec3<f32>,
    _padding1: f32,
    depth_bias: f32,
    normal_bias: f32,
    pcf_radius: f32,
    cascade_count: u32,
}

// PCF (Percentage Closer Filtering) offsets for soft shadows
const PCF_OFFSETS: array<vec2<f32>, 16> = array<vec2<f32>, 16>(
    vec2<f32>(-1.5, -1.5), vec2<f32>(-0.5, -1.5), vec2<f32>(0.5, -1.5), vec2<f32>(1.5, -1.5),
    vec2<f32>(-1.5, -0.5), vec2<f32>(-0.5, -0.5), vec2<f32>(0.5, -0.5), vec2<f32>(1.5, -0.5),
    vec2<f32>(-1.5,  0.5), vec2<f32>(-0.5,  0.5), vec2<f32>(0.5,  0.5), vec2<f32>(1.5,  0.5),
    vec2<f32>(-1.5,  1.5), vec2<f32>(-0.5,  1.5), vec2<f32>(0.5,  1.5), vec2<f32>(1.5,  1.5)
);

// Determine which cascade to use based on view depth
fn get_cascade_index(view_depth: f32, uniforms: ShadowUniforms) -> u32 {
    for (var i = 0u; i < uniforms.cascade_count; i++) {
        if (view_depth < uniforms.cascade_splits[i + 1u]) {
            return i;
        }
    }
    return uniforms.cascade_count - 1u;
}

// Sample shadow map with PCF
fn sample_shadow_pcf(
    shadow_map: texture_depth_2d_array,
    shadow_sampler: sampler_comparison,
    shadow_coord: vec3<f32>,
    cascade_index: u32,
    pcf_radius: f32
) -> f32 {
    let shadow_map_size = 2048.0; // Assuming 2048x2048 shadow maps
    let texel_size = 1.0 / shadow_map_size;

    var shadow_factor = 0.0;
    let sample_count = 16.0;

    for (var i = 0u; i < 16u; i++) {
        let offset = PCF_OFFSETS[i] * pcf_radius * texel_size;
        let sample_coord = vec3<f32>(
            shadow_coord.x + offset.x,
            shadow_coord.y + offset.y,
            shadow_coord.z
        );

        shadow_factor += textureSampleCompare(
            shadow_map,
            shadow_sampler,
            sample_coord.xy,
            cascade_index,
            sample_coord.z
        );
    }

    return shadow_factor / sample_count;
}

// Calculate shadow factor for a world position
fn calculate_shadow_factor(
    world_pos: vec3<f32>,
    view_depth: f32,
    uniforms: ShadowUniforms,
    shadow_map: texture_depth_2d_array,
    shadow_sampler: sampler_comparison
) -> f32 {
    // Get appropriate cascade
    let cascade_index = get_cascade_index(view_depth, uniforms);

    // Transform world position to light space
    let light_view_proj = uniforms.light_view_proj_matrices[cascade_index];
    let light_space_pos = light_view_proj * vec4<f32>(world_pos, 1.0);

    // Perform perspective divide
    let shadow_coord = light_space_pos.xyz / light_space_pos.w;

    // Transform to texture coordinates [0,1]
    let shadow_uv = vec3<f32>(
        shadow_coord.x * 0.5 + 0.5,
        shadow_coord.y * 0.5 + 0.5,
        shadow_coord.z
    );

    // Check if position is outside shadow map bounds
    if (shadow_uv.x < 0.0 || shadow_uv.x > 1.0 ||
        shadow_uv.y < 0.0 || shadow_uv.y > 1.0 ||
        shadow_uv.z < 0.0 || shadow_uv.z > 1.0) {
        return 1.0; // No shadow
    }

    // Sample shadow map with PCF
    return sample_shadow_pcf(
        shadow_map,
        shadow_sampler,
        shadow_uv,
        cascade_index,
        uniforms.pcf_radius
    );
}

// Calculate shadow factor with cascade visualization (for debugging)
fn calculate_shadow_factor_debug(
    world_pos: vec3<f32>,
    view_depth: f32,
    uniforms: ShadowUniforms,
    shadow_map: texture_depth_2d_array,
    shadow_sampler: sampler_comparison
) -> vec4<f32> {
    let cascade_index = get_cascade_index(view_depth, uniforms);
    let shadow_factor = calculate_shadow_factor(world_pos, view_depth, uniforms, shadow_map, shadow_sampler);

    // Color-code cascades for visualization
    var cascade_color = vec3<f32>(1.0);
    switch (cascade_index) {
        case 0u: { cascade_color = vec3<f32>(1.0, 0.0, 0.0); } // Red
        case 1u: { cascade_color = vec3<f32>(0.0, 1.0, 0.0); } // Green
        case 2u: { cascade_color = vec3<f32>(0.0, 0.0, 1.0); } // Blue
        case 3u: { cascade_color = vec3<f32>(1.0, 1.0, 0.0); } // Yellow
        default: { cascade_color = vec3<f32>(1.0, 0.0, 1.0); } // Magenta
    }

    return vec4<f32>(cascade_color * shadow_factor, shadow_factor);
}

// Soft shadow sampling with Poisson disk
const POISSON_DISK: array<vec2<f32>, 16> = array<vec2<f32>, 16>(
    vec2<f32>(-0.94201624, -0.39906216),
    vec2<f32>(0.94558609, -0.76890725),
    vec2<f32>(-0.094184101, -0.92938870),
    vec2<f32>(0.34495938, 0.29387760),
    vec2<f32>(-0.91588581, 0.45771432),
    vec2<f32>(-0.81544232, -0.87912464),
    vec2<f32>(-0.38277543, 0.27676845),
    vec2<f32>(0.97484398, 0.75648379),
    vec2<f32>(0.44323325, -0.97511554),
    vec2<f32>(0.53742981, -0.47373420),
    vec2<f32>(-0.26496911, -0.41893023),
    vec2<f32>(0.79197514, 0.19090188),
    vec2<f32>(-0.24188840, 0.99706507),
    vec2<f32>(-0.81409955, 0.91437590),
    vec2<f32>(0.19984126, 0.78641367),
    vec2<f32>(0.14383161, -0.14100790)
);

// Sample shadow with Poisson disk for softer shadows
fn sample_shadow_poisson(
    shadow_map: texture_depth_2d_array,
    shadow_sampler: sampler_comparison,
    shadow_coord: vec3<f32>,
    cascade_index: u32,
    radius: f32
) -> f32 {
    let shadow_map_size = 2048.0;
    let texel_size = 1.0 / shadow_map_size;

    var shadow_factor = 0.0;

    for (var i = 0u; i < 16u; i++) {
        let offset = POISSON_DISK[i] * radius * texel_size;
        let sample_coord = vec3<f32>(
            shadow_coord.x + offset.x,
            shadow_coord.y + offset.y,
            shadow_coord.z
        );

        shadow_factor += textureSampleCompare(
            shadow_map,
            shadow_sampler,
            sample_coord.xy,
            cascade_index,
            sample_coord.z
        );
    }

    return shadow_factor / 16.0;
}