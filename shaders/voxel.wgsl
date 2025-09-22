// Robin Voxel Engine Shader

struct Uniforms {
    view_proj: mat4x4<f32>,
    view_pos: vec4<f32>,
    light_pos: vec4<f32>,
    light_color: vec4<f32>,
    // Cascaded shadow maps (3 cascades)
    light_space_matrix_0: mat4x4<f32>,
    light_space_matrix_1: mat4x4<f32>,
    light_space_matrix_2: mat4x4<f32>,
    cascade_splits: vec4<f32>, // x,y,z = split distances, w = num_cascades
    shadow_bias: vec4<f32>, // x = bias, y = normal_bias, z = pcf_radius, w = enable_shadows
    time: f32,
    _padding: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var texture_atlas: texture_2d<f32>;

@group(0) @binding(2)
var texture_sampler: sampler;

// Shadow maps for cascaded shadow mapping
@group(0) @binding(3)
var shadow_map_0: texture_depth_2d;

@group(0) @binding(4)
var shadow_map_1: texture_depth_2d;

@group(0) @binding(5)
var shadow_map_2: texture_depth_2d;

@group(0) @binding(6)
var shadow_sampler: sampler_comparison;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    let world_position = model.position;
    out.world_position = world_position;
    out.clip_position = uniforms.view_proj * vec4<f32>(world_position, 1.0);
    out.normal = model.normal;
    out.uv = model.uv;
    out.color = model.color;

    return out;
}

// Shadow mapping functions
fn sample_shadow_map(shadow_coord: vec3<f32>, cascade: i32) -> f32 {
    // Clamp shadow coordinates to valid range
    if (shadow_coord.x < 0.0 || shadow_coord.x > 1.0 ||
        shadow_coord.y < 0.0 || shadow_coord.y > 1.0 ||
        shadow_coord.z < 0.0 || shadow_coord.z > 1.0) {
        return 1.0; // Outside shadow map = no shadow
    }

    // Sample appropriate cascade
    if (cascade == 0) {
        return textureSampleCompare(shadow_map_0, shadow_sampler, shadow_coord.xy, shadow_coord.z);
    } else if (cascade == 1) {
        return textureSampleCompare(shadow_map_1, shadow_sampler, shadow_coord.xy, shadow_coord.z);
    } else {
        return textureSampleCompare(shadow_map_2, shadow_sampler, shadow_coord.xy, shadow_coord.z);
    }
}

fn pcf_shadow(shadow_coord: vec3<f32>, cascade: i32) -> f32 {
    let texel_size = 1.0 / 2048.0; // Shadow map size
    let pcf_radius = uniforms.shadow_bias.z;

    var shadow = 0.0;
    let samples = 9; // 3x3 PCF
    var count = 0;

    for (var x = -1; x <= 1; x++) {
        for (var y = -1; y <= 1; y++) {
            let offset = vec2<f32>(f32(x), f32(y)) * texel_size * pcf_radius;
            let coord = vec3<f32>(shadow_coord.xy + offset, shadow_coord.z);
            shadow += sample_shadow_map(coord, cascade);
            count++;
        }
    }

    return shadow / f32(count);
}

fn get_cascade_index(view_depth: f32) -> i32 {
    if (view_depth < uniforms.cascade_splits.x) {
        return 0;
    } else if (view_depth < uniforms.cascade_splits.y) {
        return 1;
    } else {
        return 2;
    }
}

fn calculate_shadow(world_pos: vec3<f32>, view_depth: f32, normal: vec3<f32>) -> f32 {
    // Early exit if shadows disabled
    if (uniforms.shadow_bias.w < 0.5) {
        return 1.0;
    }

    // Determine which cascade to use
    let cascade = get_cascade_index(view_depth);

    // Transform to light space for the appropriate cascade
    var light_space_pos: vec4<f32>;
    if (cascade == 0) {
        light_space_pos = uniforms.light_space_matrix_0 * vec4<f32>(world_pos, 1.0);
    } else if (cascade == 1) {
        light_space_pos = uniforms.light_space_matrix_1 * vec4<f32>(world_pos, 1.0);
    } else {
        light_space_pos = uniforms.light_space_matrix_2 * vec4<f32>(world_pos, 1.0);
    }

    // Perspective divide
    let shadow_coord = (light_space_pos.xyz / light_space_pos.w);

    // Transform from [-1,1] to [0,1]
    let shadow_uv = vec3<f32>(
        shadow_coord.x * 0.5 + 0.5,
        shadow_coord.y * 0.5 + 0.5,
        shadow_coord.z
    );

    // Apply bias to prevent shadow acne
    let light_dir = normalize(uniforms.light_pos.xyz - world_pos);
    let bias = max(uniforms.shadow_bias.y * (1.0 - dot(normal, light_dir)), uniforms.shadow_bias.x);
    let biased_coord = vec3<f32>(shadow_uv.xy, shadow_uv.z - bias);

    // Use PCF for smooth shadow edges
    return pcf_shadow(biased_coord, cascade);
}

// PBR helper functions
fn distribution_ggx(N: vec3<f32>, H: vec3<f32>, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let NdotH = max(dot(N, H), 0.0);
    let NdotH2 = NdotH * NdotH;

    let num = a2;
    let denom = (NdotH2 * (a2 - 1.0) + 1.0);
    let denom_final = 3.141592653589793 * denom * denom;

    return num / denom_final;
}

fn geometry_schlick_ggx(NdotV: f32, roughness: f32) -> f32 {
    let r = (roughness + 1.0);
    let k = (r * r) / 8.0;

    let num = NdotV;
    let denom = NdotV * (1.0 - k) + k;

    return num / denom;
}

fn geometry_smith(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, roughness: f32) -> f32 {
    let NdotV = max(dot(N, V), 0.0);
    let NdotL = max(dot(N, L), 0.0);
    let ggx2 = geometry_schlick_ggx(NdotV, roughness);
    let ggx1 = geometry_schlick_ggx(NdotL, roughness);

    return ggx1 * ggx2;
}

fn fresnel_schlick(cosTheta: f32, F0: vec3<f32>) -> vec3<f32> {
    return F0 + (1.0 - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample texture from atlas
    let tex_color = textureSample(texture_atlas, texture_sampler, in.uv);

    // Extract material properties from vertex color
    // Using color.g for roughness (0.0-1.0) and color.b for metallic (0.0-1.0)
    let roughness = in.color.g;
    let metallic = in.color.b;

    // Calculate lighting vectors
    let N = normalize(in.normal);
    let V = normalize(uniforms.view_pos.xyz - in.world_position);
    let L = normalize(uniforms.light_pos.xyz - in.world_position);
    let H = normalize(V + L);

    // Calculate base reflectivity (F0)
    var F0 = vec3<f32>(0.04); // Default dielectric F0
    F0 = mix(F0, tex_color.rgb, metallic);

    // Cook-Torrance BRDF
    let NDF = distribution_ggx(N, H, roughness);
    let G = geometry_smith(N, V, L, roughness);
    let F = fresnel_schlick(max(dot(H, V), 0.0), F0);

    let kS = F;
    var kD = vec3<f32>(1.0) - kS;
    kD *= 1.0 - metallic; // Metals have no diffuse lighting

    let numerator = NDF * G * F;
    let denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001;
    let specular = numerator / denominator;

    // Calculate shadow factor
    let view_depth = length(uniforms.view_pos.xyz - in.world_position);
    let shadow_factor = calculate_shadow(in.world_position, view_depth, N);

    // Radiance calculation
    let NdotL = max(dot(N, L), 0.0);
    let radiance = uniforms.light_color.rgb; // Dynamic light color

    // BRDF with shadow attenuation
    let Lo = (kD * tex_color.rgb / 3.141592653589793 + specular) * radiance * NdotL * shadow_factor;

    // Ambient lighting (IBL approximation) - varies with light intensity
    let ambient_strength = 0.05 + 0.15 * (uniforms.light_color.r + uniforms.light_color.g + uniforms.light_color.b) / 3.0;
    let ambient = ambient_strength * uniforms.light_color.rgb * tex_color.rgb;

    var final_color = ambient + Lo;

    // Add emissive glow for certain materials (crystal and lava)
    // Use vertex color.r as emissive identifier
    if (in.color.r > 0.9) { // Emissive materials
        final_color += tex_color.rgb * 0.5 * (1.0 + sin(uniforms.time * 3.0) * 0.3);
    }

    // HDR tonemapping (simple Reinhard)
    final_color = final_color / (final_color + vec3<f32>(1.0));

    // Gamma correction
    final_color = pow(final_color, vec3<f32>(1.0/2.2));

    // Simple fog effect based on distance
    let distance = length(uniforms.view_pos.xyz - in.world_position);
    let fog_factor = 1.0 - smoothstep(50.0, 200.0, distance);
    let fog_color = vec3<f32>(0.5, 0.8, 1.0);
    final_color = mix(final_color, fog_color, 1.0 - fog_factor);

    return vec4<f32>(final_color, tex_color.a);
}