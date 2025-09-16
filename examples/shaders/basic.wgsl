// Basic WGSL shader for Robin 3D Engine
// Supports textured and colored materials with basic lighting

// Vertex shader input
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec3<f32>,
}

// Vertex shader output / Fragment shader input
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec3<f32>,
}

// Uniform buffer for camera and projection matrices
struct CameraUniform {
    view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
}

// Uniform buffer for model transform
struct ModelUniform {
    model: mat4x4<f32>,
    normal: mat4x4<f32>,
}

// Material properties
struct Material {
    albedo: vec3<f32>,
    metallic: f32,
    roughness: f32,
    emissive: vec3<f32>,
    use_texture: f32,
}

// Lighting data
struct Light {
    position: vec3<f32>,
    direction: vec3<f32>,
    color: vec3<f32>,
    intensity: f32,
    light_type: u32, // 0 = directional, 1 = point, 2 = spot
}

struct LightingData {
    num_lights: u32,
    ambient: vec3<f32>,
}

// Bind groups
@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var<uniform> model: ModelUniform;
@group(2) @binding(0) var<uniform> material: Material;
@group(2) @binding(1) var albedo_texture: texture_2d<f32>;
@group(2) @binding(2) var albedo_sampler: sampler;
@group(3) @binding(0) var<uniform> lighting: LightingData;
@group(3) @binding(1) var<storage, read> lights: array<Light>;

// Vertex shader
@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // Transform position to world space
    let world_position = model.model * vec4<f32>(input.position, 1.0);
    out.world_position = world_position.xyz;
    
    // Transform to clip space
    out.clip_position = camera.view_proj * world_position;
    
    // Transform normal to world space
    out.normal = normalize((model.normal * vec4<f32>(input.normal, 0.0)).xyz);
    
    // Pass through UV and color
    out.uv = input.uv;
    out.color = input.color;
    
    return out;
}

// Fragment shader
@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Base color from material or texture
    var albedo: vec3<f32>;
    if (material.use_texture > 0.5) {
        albedo = textureSample(albedo_texture, albedo_sampler, input.uv).rgb * material.albedo;
    } else {
        albedo = material.albedo * input.color;
    }
    
    // Normal and view direction
    let normal = normalize(input.normal);
    let view_dir = normalize(camera.camera_pos - input.world_position);
    
    // Start with ambient lighting
    var final_color = albedo * lighting.ambient;
    
    // Apply each light
    for (var i = 0u; i < lighting.num_lights; i = i + 1u) {
        let light = lights[i];
        
        var light_dir: vec3<f32>;
        var light_intensity: f32 = light.intensity;
        
        if (light.light_type == 0u) {
            // Directional light
            light_dir = normalize(-light.direction);
        } else {
            // Point/spot light
            light_dir = normalize(light.position - input.world_position);
            let distance = length(light.position - input.world_position);
            light_intensity = light.intensity / (1.0 + distance * distance * 0.01);
        }
        
        // Lambertian diffuse lighting
        let n_dot_l = max(dot(normal, light_dir), 0.0);
        let diffuse = albedo * light.color * n_dot_l * light_intensity;
        
        // Simple Phong specular
        let reflect_dir = reflect(-light_dir, normal);
        let spec = pow(max(dot(view_dir, reflect_dir), 0.0), 32.0);
        let specular = light.color * spec * light_intensity * (1.0 - material.roughness);
        
        final_color = final_color + diffuse + specular;
    }
    
    // Add emissive contribution
    final_color = final_color + material.emissive;
    
    // Simple tone mapping and gamma correction
    final_color = final_color / (final_color + vec3<f32>(1.0));
    final_color = pow(final_color, vec3<f32>(1.0 / 2.2));
    
    return vec4<f32>(final_color, 1.0);
}

// Simple unlit shader variants for UI and debug rendering
@vertex
fn vs_unlit(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * model.model * vec4<f32>(input.position, 1.0);
    out.world_position = (model.model * vec4<f32>(input.position, 1.0)).xyz;
    out.normal = input.normal;
    out.uv = input.uv;
    out.color = input.color;
    return out;
}

@fragment
fn fs_unlit(input: VertexOutput) -> @location(0) vec4<f32> {
    var color: vec3<f32>;
    if (material.use_texture > 0.5) {
        color = textureSample(albedo_texture, albedo_sampler, input.uv).rgb;
    } else {
        color = input.color;
    }
    return vec4<f32>(color, 1.0);
}