// Atmospheric Scattering Precomputation Shader
// Generates lookup tables for realistic atmospheric effects

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

@group(0) @binding(1)
var<uniform> uniforms: AtmosphereUniforms;

@group(0) @binding(4)
var atmosphere_lut_output: texture_storage_2d<rgba16float, write>;

// Constants for atmospheric scattering
const PI = 3.14159265359;
const ATMOSPHERE_SAMPLES = 64u;
const LIGHT_SAMPLES = 16u;

// Rayleigh phase function
fn rayleigh_phase(cos_theta: f32) -> f32 {
    return 3.0 / (16.0 * PI) * (1.0 + cos_theta * cos_theta);
}

// Mie phase function (Henyey-Greenstein approximation)
fn mie_phase(cos_theta: f32, g: f32) -> f32 {
    let g2 = g * g;
    let denom = pow(1.0 + g2 - 2.0 * g * cos_theta, 1.5);
    return (1.0 - g2) / (4.0 * PI * denom);
}

// Sphere intersection for atmosphere bounds
fn ray_sphere_intersect(ray_origin: vec3<f32>, ray_dir: vec3<f32>, sphere_center: vec3<f32>, sphere_radius: f32) -> vec2<f32> {
    let oc = ray_origin - sphere_center;
    let a = dot(ray_dir, ray_dir);
    let b = 2.0 * dot(oc, ray_dir);
    let c = dot(oc, oc) - sphere_radius * sphere_radius;
    let discriminant = b * b - 4.0 * a * c;

    if (discriminant < 0.0) {
        return vec2<f32>(-1.0, -1.0);
    }

    let sqrt_discriminant = sqrt(discriminant);
    let t1 = (-b - sqrt_discriminant) / (2.0 * a);
    let t2 = (-b + sqrt_discriminant) / (2.0 * a);

    return vec2<f32>(t1, t2);
}

// Get density at a given height using exponential model
fn get_density(height: f32, scale_height: f32) -> f32 {
    return exp(-max(height, 0.0) / scale_height);
}

// Optical depth calculation using numerical integration
fn get_optical_depth(
    ray_origin: vec3<f32>,
    ray_dir: vec3<f32>,
    ray_length: f32,
    planet_radius: f32,
    atmosphere_radius: f32,
    scale_height: f32
) -> f32 {
    let sample_count = 10u;
    let step_size = ray_length / f32(sample_count);

    var optical_depth = 0.0;

    for (var i = 0u; i < sample_count; i++) {
        let sample_pos = ray_origin + ray_dir * (f32(i) + 0.5) * step_size;
        let height = length(sample_pos) - planet_radius;
        let density = get_density(height, scale_height);
        optical_depth += density * step_size;
    }

    return optical_depth;
}

// Calculate single scattering for atmospheric effects
fn calculate_scattering(
    ray_origin: vec3<f32>,
    ray_dir: vec3<f32>,
    sun_dir: vec3<f32>,
    sun_intensity: f32
) -> vec4<f32> {
    let planet_center = vec3<f32>(0.0, -uniforms.planet_radius, 0.0);

    // Intersect ray with atmosphere
    let atmosphere_intersect = ray_sphere_intersect(
        ray_origin,
        ray_dir,
        planet_center,
        uniforms.atmosphere_radius
    );

    if (atmosphere_intersect.x < 0.0) {
        return vec4<f32>(0.0); // Ray doesn't intersect atmosphere
    }

    // Check if ray hits planet surface
    let planet_intersect = ray_sphere_intersect(
        ray_origin,
        ray_dir,
        planet_center,
        uniforms.planet_radius
    );

    let ray_start = max(atmosphere_intersect.x, 0.0);
    let ray_end = select(atmosphere_intersect.y, planet_intersect.x, planet_intersect.x > 0.0);
    let ray_length = ray_end - ray_start;

    if (ray_length <= 0.0) {
        return vec4<f32>(0.0);
    }

    let step_size = ray_length / f32(ATMOSPHERE_SAMPLES);

    var total_rayleigh = vec3<f32>(0.0);
    var total_mie = vec3<f32>(0.0);

    var optical_depth_rayleigh = 0.0;
    var optical_depth_mie = 0.0;

    for (var i = 0u; i < ATMOSPHERE_SAMPLES; i++) {
        let sample_pos = ray_origin + ray_dir * (ray_start + (f32(i) + 0.5) * step_size);
        let height = length(sample_pos) - uniforms.planet_radius;

        // Get densities at sample point
        let rayleigh_density = get_density(height, uniforms.rayleigh_scale_height);
        let mie_density = get_density(height, uniforms.mie_scale_height);

        // Update optical depths
        optical_depth_rayleigh += rayleigh_density * step_size;
        optical_depth_mie += mie_density * step_size;

        // Calculate light ray optical depth
        let light_ray_intersect = ray_sphere_intersect(
            sample_pos,
            sun_dir,
            planet_center,
            uniforms.atmosphere_radius
        );

        if (light_ray_intersect.y > 0.0) {
            let light_ray_length = light_ray_intersect.y;

            let light_optical_depth_rayleigh = get_optical_depth(
                sample_pos,
                sun_dir,
                light_ray_length,
                uniforms.planet_radius,
                uniforms.atmosphere_radius,
                uniforms.rayleigh_scale_height
            );

            let light_optical_depth_mie = get_optical_depth(
                sample_pos,
                sun_dir,
                light_ray_length,
                uniforms.planet_radius,
                uniforms.atmosphere_radius,
                uniforms.mie_scale_height
            );

            // Calculate transmittance
            let rayleigh_transmittance = exp(-(uniforms.rayleigh_coefficient * (optical_depth_rayleigh + light_optical_depth_rayleigh)));
            let mie_transmittance = exp(-uniforms.mie_coefficient * (optical_depth_mie + light_optical_depth_mie));

            // Accumulate scattering
            total_rayleigh += rayleigh_density * rayleigh_transmittance * step_size;
            total_mie += mie_density * mie_transmittance * step_size;
        }
    }

    // Apply phase functions
    let cos_theta = dot(ray_dir, sun_dir);
    let rayleigh_phase_value = rayleigh_phase(cos_theta);
    let mie_phase_value = mie_phase(cos_theta, 0.8); // g = 0.8 for Mie scattering

    let rayleigh_scattering = uniforms.rayleigh_coefficient * total_rayleigh * rayleigh_phase_value;
    let mie_scattering = vec3<f32>(uniforms.mie_coefficient) * total_mie * mie_phase_value;

    let total_scattering = (rayleigh_scattering + mie_scattering) * sun_intensity;

    // Apply exposure and tone mapping
    let exposed = 1.0 - exp(-total_scattering * 0.0001);

    return vec4<f32>(exposed, 1.0);
}

// Convert UV coordinates to view direction
fn uv_to_view_direction(uv: vec2<f32>) -> vec3<f32> {
    let theta = uv.x * 2.0 * PI; // Azimuth: 0 to 2π
    let phi = uv.y * PI; // Inclination: 0 to π

    return vec3<f32>(
        sin(phi) * cos(theta),
        cos(phi),
        sin(phi) * sin(theta)
    );
}

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let coord = vec2<i32>(global_id.xy);
    let texture_size = vec2<i32>(256, 128);

    if (coord.x >= texture_size.x || coord.y >= texture_size.y) {
        return;
    }

    let uv = (vec2<f32>(coord) + 0.5) / vec2<f32>(texture_size);

    // Convert UV to view direction
    let view_dir = uv_to_view_direction(uv);

    // Set camera at sea level looking in the calculated direction
    let camera_pos = vec3<f32>(0.0, 1.0, 0.0); // 1m above planet surface
    let sun_dir = normalize(uniforms.sun_direction);

    // Calculate atmospheric scattering
    let scattering = calculate_scattering(
        camera_pos,
        view_dir,
        sun_dir,
        uniforms.sun_intensity
    );

    textureStore(atmosphere_lut_output, coord, scattering);
}