// Main 3D rendering shader for Robin Engine
// Supports PBR materials, instanced rendering, and dynamic lighting

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
    @location(3) color: vec4<f32>,
    @location(4) tangent: vec3<f32>,
    // Instance attributes
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
    @location(9) instance_color: vec4<f32>,
    @location(10) material_id: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
    @location(3) vertex_color: vec4<f32>,
    @location(4) instance_color: vec4<f32>,
    @location(5) tangent: vec3<f32>,
    @location(6) bitangent: vec3<f32>,
    @location(7) view_position: vec3<f32>,
    @location(8) @interpolate(flat) material_id: u32,
}

struct CameraUniform {
    view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    time: f32,
    _padding: vec3<f32>,
}

struct LightData {
    position: vec3<f32>,
    light_type: u32,  // 0=directional, 1=point, 2=spot
    direction: vec3<f32>,
    intensity: f32,
    color: vec3<f32>,
    range: f32,
    inner_cone: f32,
    outer_cone: f32,
    _padding: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> lights: array<LightData, 16>;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // Reconstruct model matrix from instance data
    let model_matrix = mat4x4<f32>(
        in.model_matrix_0,
        in.model_matrix_1,
        in.model_matrix_2,
        in.model_matrix_3
    );
    
    // Transform position to world space
    let world_position = model_matrix * vec4<f32>(in.position, 1.0);
    out.world_position = world_position.xyz;
    
    // Transform to view space
    let view_position = camera.view * world_position;
    out.view_position = view_position.xyz;
    
    // Transform to clip space
    out.clip_position = camera.proj * view_position;
    
    // Transform normal to world space (assuming uniform scaling)
    let normal_matrix = mat3x3<f32>(
        model_matrix[0].xyz,
        model_matrix[1].xyz,
        model_matrix[2].xyz
    );
    out.world_normal = normalize(normal_matrix * in.normal);
    
    // Calculate tangent and bitangent for normal mapping
    out.tangent = normalize(normal_matrix * in.tangent);
    out.bitangent = cross(out.world_normal, out.tangent);
    
    // Pass through other attributes
    out.tex_coords = in.tex_coords;
    out.vertex_color = in.color;
    out.instance_color = in.instance_color;
    out.material_id = in.material_id;
    
    return out;
}

// PBR lighting calculation
fn calculate_pbr_lighting(
    albedo: vec3<f32>,
    metallic: f32,
    roughness: f32,
    normal: vec3<f32>,
    view_dir: vec3<f32>,
    light_dir: vec3<f32>,
    light_color: vec3<f32>,
    light_intensity: f32
) -> vec3<f32> {
    let h = normalize(view_dir + light_dir);
    let n_dot_v = max(dot(normal, view_dir), 0.0);
    let n_dot_l = max(dot(normal, light_dir), 0.0);
    let n_dot_h = max(dot(normal, h), 0.0);
    let v_dot_h = max(dot(view_dir, h), 0.0);
    
    // Fresnel (Schlick approximation)
    let f0 = mix(vec3<f32>(0.04), albedo, metallic);
    let fresnel = f0 + (1.0 - f0) * pow(1.0 - v_dot_h, 5.0);
    
    // Distribution (GGX/Trowbridge-Reitz)
    let alpha = roughness * roughness;
    let alpha_sq = alpha * alpha;
    let denom = n_dot_h * n_dot_h * (alpha_sq - 1.0) + 1.0;
    let d = alpha_sq / (3.14159265 * denom * denom);
    
    // Geometry (Smith's method)
    let k = (roughness + 1.0) * (roughness + 1.0) / 8.0;
    let g_v = n_dot_v / (n_dot_v * (1.0 - k) + k);
    let g_l = n_dot_l / (n_dot_l * (1.0 - k) + k);
    let g = g_v * g_l;
    
    // BRDF
    let specular = (d * g * fresnel) / max(4.0 * n_dot_v * n_dot_l, 0.001);
    let diffuse = (1.0 - fresnel) * (1.0 - metallic) * albedo / 3.14159265;
    
    return (diffuse + specular) * light_color * light_intensity * n_dot_l;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Material properties based on material_id
    var albedo: vec3<f32>;
    var metallic: f32;
    var roughness: f32;
    var emissive: f32;
    
    switch in.material_id {
        case 0u: { // Stone
            albedo = vec3<f32>(0.5, 0.5, 0.5);
            metallic = 0.0;
            roughness = 0.8;
            emissive = 0.0;
        }
        case 1u: { // Metal
            albedo = vec3<f32>(0.7, 0.7, 0.8);
            metallic = 0.9;
            roughness = 0.3;
            emissive = 0.0;
        }
        case 2u: { // Wood
            albedo = vec3<f32>(0.5, 0.3, 0.1);
            metallic = 0.0;
            roughness = 0.6;
            emissive = 0.0;
        }
        case 3u: { // Glass
            albedo = vec3<f32>(0.6, 0.8, 1.0);
            metallic = 0.0;
            roughness = 0.1;
            emissive = 0.0;
        }
        case 4u: { // Energy
            albedo = vec3<f32>(0.0, 0.8, 1.0);
            metallic = 0.5;
            roughness = 0.2;
            emissive = 2.0;
        }
        default: {
            albedo = in.instance_color.rgb;
            metallic = 0.5;
            roughness = 0.5;
            emissive = 0.0;
        }
    }
    
    // Mix with instance color
    albedo = mix(albedo, in.instance_color.rgb, 0.5);
    
    // Normalize vectors
    let normal = normalize(in.world_normal);
    let view_dir = normalize(camera.camera_pos.xyz - in.world_position);
    
    // Ambient lighting
    var final_color = albedo * 0.03;
    
    // Process each light
    for (var i = 0u; i < 16u; i = i + 1u) {
        let light = lights[i];
        
        // Skip if light is off (intensity == 0)
        if (light.intensity < 0.001) {
            continue;
        }
        
        var light_dir: vec3<f32>;
        var attenuation = 1.0;
        
        if (light.light_type == 0u) {
            // Directional light
            light_dir = normalize(-light.direction);
        } else if (light.light_type == 1u) {
            // Point light
            let light_vec = light.position - in.world_position;
            let distance = length(light_vec);
            light_dir = normalize(light_vec);
            
            // Attenuation
            attenuation = 1.0 / (1.0 + 0.09 * distance + 0.032 * distance * distance);
            attenuation *= smoothstep(light.range, 0.0, distance);
        } else if (light.light_type == 2u) {
            // Spot light
            let light_vec = light.position - in.world_position;
            let distance = length(light_vec);
            light_dir = normalize(light_vec);
            
            // Cone attenuation
            let theta = dot(light_dir, normalize(-light.direction));
            let epsilon = light.inner_cone - light.outer_cone;
            let intensity = clamp((theta - light.outer_cone) / epsilon, 0.0, 1.0);
            
            // Distance attenuation
            attenuation = intensity / (1.0 + 0.09 * distance + 0.032 * distance * distance);
            attenuation *= smoothstep(light.range, 0.0, distance);
        }
        
        // Calculate PBR lighting contribution
        let light_contribution = calculate_pbr_lighting(
            albedo,
            metallic,
            roughness,
            normal,
            view_dir,
            light_dir,
            light.color,
            light.intensity * attenuation
        );
        
        final_color += light_contribution;
    }
    
    // Add emissive contribution
    final_color += albedo * emissive;
    
    // Fog effect based on distance
    let fog_distance = length(in.view_position);
    let fog_factor = exp(-fog_distance * 0.01);
    let fog_color = vec3<f32>(0.05, 0.05, 0.1);
    final_color = mix(fog_color, final_color, fog_factor);
    
    // Tone mapping (Reinhard)
    final_color = final_color / (final_color + vec3<f32>(1.0));
    
    // Gamma correction
    final_color = pow(final_color, vec3<f32>(1.0 / 2.2));
    
    // Handle transparency for glass material
    var alpha = in.instance_color.a;
    if (in.material_id == 3u) { // Glass
        alpha = 0.5;
    }
    
    return vec4<f32>(final_color, alpha);
}