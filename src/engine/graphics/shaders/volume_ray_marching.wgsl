// Volumetric Ray Marching Compute Shader
// Simulates light scattering through participating media (fog, atmosphere)

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

struct AtmosphereUniforms {
    sun_direction: vec3<f32>,
    _padding1: f32,
    sun_intensity: f32,
    rayleigh_coefficient: vec3<f32>,
    mie_coefficient: f32,
    rayleigh_scale_height: f32,
    mie_scale_height: f32,
    planet_radius: f32,
    atmosphere_radius: f32,
    _padding2: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> volumetric_uniforms: VolumetricUniforms;

@group(0) @binding(1)
var<uniform> atmosphere_uniforms: AtmosphereUniforms;

@group(0) @binding(2)
var volume_output: texture_storage_3d<rgba16float, write>;

@group(0) @binding(3)
var volume_history: texture_3d<f32>;

@group(0) @binding(4)
var atmosphere_lut_output: texture_storage_2d<rgba16float, write>;

@group(0) @binding(5)
var atmosphere_lut: texture_2d<f32>;

@group(0) @binding(6)
var volume_sampler: sampler;

@group(0) @binding(7)
var atmosphere_sampler: sampler;

// Noise functions for temporal dithering
fn hash13(p3: vec3<f32>) -> f32 {
    var p = fract(p3 * 0.1031);
    p += dot(p, p.zyx + 31.32);
    return fract((p.x + p.y) * p.z);
}

fn noise3d(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);

    let u = f * f * (3.0 - 2.0 * f);

    return mix(
        mix(
            mix(hash13(i + vec3<f32>(0.0, 0.0, 0.0)),
                hash13(i + vec3<f32>(1.0, 0.0, 0.0)), u.x),
            mix(hash13(i + vec3<f32>(0.0, 1.0, 0.0)),
                hash13(i + vec3<f32>(1.0, 1.0, 0.0)), u.x), u.y),
        mix(
            mix(hash13(i + vec3<f32>(0.0, 0.0, 1.0)),
                hash13(i + vec3<f32>(1.0, 0.0, 1.0)), u.x),
            mix(hash13(i + vec3<f32>(0.0, 1.0, 1.0)),
                hash13(i + vec3<f32>(1.0, 1.0, 1.0)), u.x), u.y), u.z);
}

// Henyey-Greenstein phase function
fn phase_function_hg(cos_theta: f32, g: f32) -> f32 {
    let g2 = g * g;
    let denom = pow(1.0 + g2 - 2.0 * g * cos_theta, 1.5);
    return (1.0 - g2) / (4.0 * 3.14159265 * denom);
}

// Convert 3D texture coordinates to world position
fn texture_coord_to_world_pos(tex_coord: vec3<f32>) -> vec3<f32> {
    // Convert to NDC
    let ndc = tex_coord * 2.0 - 1.0;

    // Calculate depth in exponential distribution
    let linear_depth = tex_coord.z;
    let exp_depth = mix(volumetric_uniforms.z_near, volumetric_uniforms.z_far, linear_depth);

    // Reconstruct world position
    let view_pos = volumetric_uniforms.inv_view_proj_matrix * vec4<f32>(ndc.x, ndc.y, exp_depth, 1.0);
    return view_pos.xyz / view_pos.w;
}

// Sample atmospheric density at world position
fn sample_atmosphere_density(world_pos: vec3<f32>) -> f32 {
    let height = length(world_pos) - atmosphere_uniforms.planet_radius;

    // Simple exponential falloff
    let normalized_height = height / (atmosphere_uniforms.atmosphere_radius - atmosphere_uniforms.planet_radius);
    return exp(-normalized_height * 5.0);
}

// Sample fog density with procedural noise
fn sample_fog_density(world_pos: vec3<f32>) -> f32 {
    let base_density = volumetric_uniforms.fog_density;

    // Add procedural noise for natural variation
    let noise_scale = 0.01;
    let noise_time = volumetric_uniforms.time * 0.1;

    let noise_pos = world_pos * noise_scale + vec3<f32>(noise_time, 0.0, noise_time * 0.5);
    let noise_value = noise3d(noise_pos) * 0.5 + 0.5;

    // Height falloff
    let height_factor = exp(-max(world_pos.y - 10.0, 0.0) * 0.1);

    return base_density * noise_value * height_factor;
}

// Calculate single scattering from sun
fn calculate_sun_scattering(
    world_pos: vec3<f32>,
    view_dir: vec3<f32>,
    sun_dir: vec3<f32>
) -> vec3<f32> {
    let cos_theta = dot(view_dir, sun_dir);
    let phase = phase_function_hg(cos_theta, volumetric_uniforms.phase_function_g);

    // Simple atmospheric scattering approximation
    let height = max(world_pos.y, 0.0);
    let atmosphere_density = exp(-height * 0.0001);

    let rayleigh = atmosphere_uniforms.rayleigh_coefficient * atmosphere_density;
    let mie = atmosphere_uniforms.mie_coefficient * atmosphere_density;

    let sun_light = atmosphere_uniforms.sun_intensity * max(sun_dir.y, 0.0);

    return (rayleigh + mie * phase) * sun_light;
}

// Ray marching through volume
fn ray_march_volume(
    start_pos: vec3<f32>,
    end_pos: vec3<f32>,
    view_dir: vec3<f32>
) -> vec4<f32> {
    let ray_length = distance(start_pos, end_pos);
    let step_size = ray_length / f32(volumetric_uniforms.ray_marching_steps);

    var accumulated_scattering = vec3<f32>(0.0);
    var accumulated_transmittance = 1.0;

    // Temporal dithering for noise reduction
    let dither_offset = hash13(start_pos + f32(volumetric_uniforms.frame_index)) * step_size;

    for (var i = 0u; i < volumetric_uniforms.ray_marching_steps; i++) {
        let t = (f32(i) + 0.5 + dither_offset) / f32(volumetric_uniforms.ray_marching_steps);
        let sample_pos = mix(start_pos, end_pos, t);

        // Sample densities
        let fog_density = sample_fog_density(sample_pos);
        let atmosphere_density = sample_atmosphere_density(sample_pos);
        let total_density = fog_density + atmosphere_density;

        if (total_density > 0.001) {
            // Calculate extinction
            let extinction = total_density * (volumetric_uniforms.scattering_coefficient + volumetric_uniforms.absorption_coefficient);
            let transmittance = exp(-extinction * step_size);

            // Calculate in-scattering
            let sun_scattering = calculate_sun_scattering(sample_pos, view_dir, atmosphere_uniforms.sun_direction);
            let fog_scattering = volumetric_uniforms.fog_color * fog_density * volumetric_uniforms.scattering_coefficient;

            let total_scattering = sun_scattering + fog_scattering;

            // Accumulate lighting
            let scattering_contribution = total_scattering * (1.0 - transmittance) * accumulated_transmittance;
            accumulated_scattering += scattering_contribution;
            accumulated_transmittance *= transmittance;

            // Early exit if transmittance is too low
            if (accumulated_transmittance < 0.01) {
                break;
            }
        }
    }

    return vec4<f32>(accumulated_scattering, accumulated_transmittance);
}

@compute @workgroup_size(8, 8, 4)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let volume_coord = vec3<i32>(global_id);
    let volume_size = vec3<i32>(volumetric_uniforms.volume_resolution);

    // Bounds check
    if (volume_coord.x >= volume_size.x ||
        volume_coord.y >= volume_size.y ||
        volume_coord.z >= volume_size.z) {
        return;
    }

    // Convert to normalized texture coordinates
    let tex_coord = vec3<f32>(volume_coord) / vec3<f32>(volume_size);

    // Calculate world position for this voxel
    let world_pos = texture_coord_to_world_pos(tex_coord);

    // Calculate ray from camera to this voxel
    let view_dir = normalize(world_pos - volumetric_uniforms.camera_position);

    // Calculate ray marching bounds
    let ray_start = volumetric_uniforms.camera_position;
    let ray_end = world_pos;

    // Perform ray marching
    let volume_result = ray_march_volume(ray_start, ray_end, view_dir);

    // Apply temporal filtering with history
    var final_result = volume_result;

    if (volumetric_uniforms.frame_index > 0u) {
        let history_sample = textureSampleLevel(volume_history, volume_sampler, tex_coord, 0.0);

        // Blend with history (simple temporal accumulation)
        let history_weight = 0.9;
        final_result = mix(volume_result, history_sample, history_weight);
    }

    // Store result
    textureStore(volume_output, volume_coord, final_result);
}