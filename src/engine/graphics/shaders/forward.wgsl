// Forward rendering shader for 3D graphics

struct CameraUniform {
    view_matrix: mat4x4<f32>,
    projection_matrix: mat4x4<f32>,
    view_projection_matrix: mat4x4<f32>,
    camera_position: vec4<f32>,
    camera_direction: vec4<f32>,
    near_plane: f32,
    far_plane: f32,
    fov: f32,
    aspect_ratio: f32,
}

struct LightUniform {
    light_type: u32,
    position: vec3<f32>,
    direction: vec3<f32>,
    color: vec3<f32>,
    intensity: f32,
    range: f32,
    inner_cone: f32,
    outer_cone: f32,
    _padding: u32,
}

struct MaterialUniform {
    albedo: vec4<f32>,
    metallic: f32,
    roughness: f32,
    emission: vec3<f32>,
    alpha_cutoff: f32,
    texture_flags: u32,
    _padding: array<u32, 2>,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) color: vec4<f32>,
}

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
    @location(9) normal_matrix_0: vec4<f32>,
    @location(10) normal_matrix_1: vec4<f32>,
    @location(11) normal_matrix_2: vec4<f32>,
    @location(12) normal_matrix_3: vec4<f32>,
    @location(13) material_id: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
    @location(5) color: vec4<f32>,
    @location(6) material_id: u32,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> lights: array<LightUniform, 64>;

@group(2) @binding(0)
var<uniform> materials: array<MaterialUniform, 256>;

@vertex
fn vs_main(
    vertex: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    
    let normal_matrix = mat4x4<f32>(
        instance.normal_matrix_0,
        instance.normal_matrix_1,
        instance.normal_matrix_2,
        instance.normal_matrix_3,
    );
    
    let world_position = model_matrix * vec4<f32>(vertex.position, 1.0);
    let world_normal = normalize((normal_matrix * vec4<f32>(vertex.normal, 0.0)).xyz);
    let world_tangent = normalize((model_matrix * vec4<f32>(vertex.tangent, 0.0)).xyz);
    let bitangent = cross(world_normal, world_tangent);
    
    var out: VertexOutput;
    out.clip_position = camera.view_projection_matrix * world_position;
    out.world_position = world_position.xyz;
    out.world_normal = world_normal;
    out.tex_coords = vertex.tex_coords;
    out.tangent = world_tangent;
    out.bitangent = bitangent;
    out.color = vertex.color;
    out.material_id = instance.material_id;
    
    return out;
}

// PBR lighting calculations

fn distribution_ggx(n_dot_h: f32, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let n_dot_h2 = n_dot_h * n_dot_h;
    
    let num = a2;
    var denom = n_dot_h2 * (a2 - 1.0) + 1.0;
    denom = 3.14159265359 * denom * denom;
    
    return num / denom;
}

fn geometry_schlick_ggx(n_dot_v: f32, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;
    
    let num = n_dot_v;
    let denom = n_dot_v * (1.0 - k) + k;
    
    return num / denom;
}

fn geometry_smith(n: vec3<f32>, v: vec3<f32>, l: vec3<f32>, roughness: f32) -> f32 {
    let n_dot_v = max(dot(n, v), 0.0);
    let n_dot_l = max(dot(n, l), 0.0);
    let ggx2 = geometry_schlick_ggx(n_dot_v, roughness);
    let ggx1 = geometry_schlick_ggx(n_dot_l, roughness);
    
    return ggx1 * ggx2;
}

fn fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
    return f0 + (1.0 - f0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

fn calculate_directional_light(
    light: LightUniform,
    normal: vec3<f32>,
    view_dir: vec3<f32>,
    albedo: vec3<f32>,
    metallic: f32,
    roughness: f32,
    f0: vec3<f32>,
) -> vec3<f32> {
    let light_dir = normalize(-light.direction);
    let halfway_dir = normalize(view_dir + light_dir);
    
    let n_dot_l = max(dot(normal, light_dir), 0.0);
    let n_dot_v = max(dot(normal, view_dir), 0.0);
    let n_dot_h = max(dot(normal, halfway_dir), 0.0);
    
    // Cook-Torrance BRDF
    let ndf = distribution_ggx(n_dot_h, roughness);
    let g = geometry_smith(normal, view_dir, light_dir, roughness);
    let f = fresnel_schlick(max(dot(halfway_dir, view_dir), 0.0), f0);
    
    let numerator = ndf * g * f;
    let denominator = 4.0 * n_dot_v * n_dot_l + 0.0001;
    let specular = numerator / denominator;
    
    let ks = f;
    var kd = vec3<f32>(1.0) - ks;
    kd *= 1.0 - metallic;
    
    return (kd * albedo / 3.14159265359 + specular) * light.color * light.intensity * n_dot_l;
}

fn calculate_point_light(
    light: LightUniform,
    world_pos: vec3<f32>,
    normal: vec3<f32>,
    view_dir: vec3<f32>,
    albedo: vec3<f32>,
    metallic: f32,
    roughness: f32,
    f0: vec3<f32>,
) -> vec3<f32> {
    let light_dir = normalize(light.position - world_pos);
    let distance = length(light.position - world_pos);
    let attenuation = 1.0 / (1.0 + 0.09 * distance + 0.032 * distance * distance);
    
    if distance > light.range {
        return vec3<f32>(0.0);
    }
    
    let halfway_dir = normalize(view_dir + light_dir);
    
    let n_dot_l = max(dot(normal, light_dir), 0.0);
    let n_dot_v = max(dot(normal, view_dir), 0.0);
    let n_dot_h = max(dot(normal, halfway_dir), 0.0);
    
    // Cook-Torrance BRDF
    let ndf = distribution_ggx(n_dot_h, roughness);
    let g = geometry_smith(normal, view_dir, light_dir, roughness);
    let f = fresnel_schlick(max(dot(halfway_dir, view_dir), 0.0), f0);
    
    let numerator = ndf * g * f;
    let denominator = 4.0 * n_dot_v * n_dot_l + 0.0001;
    let specular = numerator / denominator;
    
    let ks = f;
    var kd = vec3<f32>(1.0) - ks;
    kd *= 1.0 - metallic;
    
    let radiance = light.color * light.intensity * attenuation;
    return (kd * albedo / 3.14159265359 + specular) * radiance * n_dot_l;
}

fn calculate_spot_light(
    light: LightUniform,
    world_pos: vec3<f32>,
    normal: vec3<f32>,
    view_dir: vec3<f32>,
    albedo: vec3<f32>,
    metallic: f32,
    roughness: f32,
    f0: vec3<f32>,
) -> vec3<f32> {
    let light_dir = normalize(light.position - world_pos);
    let distance = length(light.position - world_pos);
    
    if distance > light.range {
        return vec3<f32>(0.0);
    }
    
    // Spot light cone calculation
    let theta = dot(light_dir, normalize(-light.direction));
    let epsilon = cos(radians(light.inner_cone)) - cos(radians(light.outer_cone));
    let intensity = clamp((theta - cos(radians(light.outer_cone))) / epsilon, 0.0, 1.0);
    
    if intensity <= 0.0 {
        return vec3<f32>(0.0);
    }
    
    let attenuation = 1.0 / (1.0 + 0.09 * distance + 0.032 * distance * distance);
    let halfway_dir = normalize(view_dir + light_dir);
    
    let n_dot_l = max(dot(normal, light_dir), 0.0);
    let n_dot_v = max(dot(normal, view_dir), 0.0);
    let n_dot_h = max(dot(normal, halfway_dir), 0.0);
    
    // Cook-Torrance BRDF
    let ndf = distribution_ggx(n_dot_h, roughness);
    let g = geometry_smith(normal, view_dir, light_dir, roughness);
    let f = fresnel_schlick(max(dot(halfway_dir, view_dir), 0.0), f0);
    
    let numerator = ndf * g * f;
    let denominator = 4.0 * n_dot_v * n_dot_l + 0.0001;
    let specular = numerator / denominator;
    
    let ks = f;
    var kd = vec3<f32>(1.0) - ks;
    kd *= 1.0 - metallic;
    
    let radiance = light.color * light.intensity * attenuation * intensity;
    return (kd * albedo / 3.14159265359 + specular) * radiance * n_dot_l;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let material = materials[in.material_id];
    
    // Material properties
    let albedo = material.albedo.rgb * in.color.rgb;
    let metallic = material.metallic;
    let roughness = clamp(material.roughness, 0.04, 1.0);
    let emission = material.emission;
    
    // Calculate F0 for dielectrics and metals
    let f0 = mix(vec3<f32>(0.04), albedo, metallic);
    
    let world_pos = in.world_position;
    let normal = normalize(in.world_normal);
    let view_dir = normalize(camera.camera_position.xyz - world_pos);
    
    // Initialize lighting
    var lo = vec3<f32>(0.0);
    
    // Calculate lighting from all lights
    for (var i: i32 = 0; i < 64; i++) {
        let light = lights[i];
        
        // Skip inactive lights (intensity <= 0)
        if light.intensity <= 0.0 {
            continue;
        }
        
        if light.light_type == 0u { // Directional light
            lo += calculate_directional_light(light, normal, view_dir, albedo, metallic, roughness, f0);
        } else if light.light_type == 1u { // Point light
            lo += calculate_point_light(light, world_pos, normal, view_dir, albedo, metallic, roughness, f0);
        } else if light.light_type == 2u { // Spot light
            lo += calculate_spot_light(light, world_pos, normal, view_dir, albedo, metallic, roughness, f0);
        }
    }
    
    // Ambient lighting (simplified)
    let ambient = vec3<f32>(0.03) * albedo;
    
    // Add emission
    let color = ambient + lo + emission;
    
    // HDR tonemapping (Reinhard)
    let mapped = color / (color + vec3<f32>(1.0));
    
    // Gamma correction
    let gamma_corrected = pow(mapped, vec3<f32>(1.0 / 2.2));
    
    return vec4<f32>(gamma_corrected, material.albedo.a * in.color.a);
}