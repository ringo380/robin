// Vertex shader

struct CameraUniform {
    view_proj: mat4x4<f32>,
}

struct Light {
    position: vec2<f32>,
    color: vec3<f32>,
    intensity: f32,
    radius: f32,
    _padding: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(0) @binding(1)
var<uniform> lights: array<Light, 64>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct InstanceInput {
    @location(5) instance_position: vec2<f32>,
    @location(6) instance_size: vec2<f32>,
    @location(7) instance_rotation: f32,
    @location(8) instance_color: vec4<f32>,
    // UV coords will be passed via storage buffer or calculated
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) world_position: vec2<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    
    // Calculate rotated and scaled position
    let cos_r = cos(instance.instance_rotation);
    let sin_r = sin(instance.instance_rotation);
    
    let scaled_pos = model.position.xy * instance.instance_size;
    let rotated_pos = vec2<f32>(
        scaled_pos.x * cos_r - scaled_pos.y * sin_r,
        scaled_pos.x * sin_r + scaled_pos.y * cos_r
    );
    
    let world_pos = rotated_pos + instance.instance_position;
    
    out.clip_position = camera.view_proj * vec4<f32>(world_pos, 0.0, 1.0);
    out.tex_coords = model.tex_coords;
    out.color = model.color * instance.instance_color;
    out.world_position = world_pos;
    
    return out;
}

// Fragment shader

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

fn calculate_lighting(world_pos: vec2<f32>, base_color: vec3<f32>) -> vec3<f32> {
    var final_color = base_color * 0.1; // Ambient lighting
    
    // Dynamic lighting calculation
    for (var i = 0; i < 64; i = i + 1) {
        let light = lights[i];
        if (light.intensity <= 0.0) {
            continue;
        }
        
        let light_dir = light.position - world_pos;
        let distance = length(light_dir);
        
        if (distance < light.radius) {
            let attenuation = 1.0 - (distance / light.radius);
            let attenuation_smooth = attenuation * attenuation;
            
            let light_contribution = light.color * light.intensity * attenuation_smooth;
            final_color = final_color + base_color * light_contribution;
        }
    }
    
    return min(final_color, vec3<f32>(1.0, 1.0, 1.0)); // Clamp to prevent overexposure
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample texture
    let texture_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    
    // Combine with sprite color
    let sprite_color = texture_color * in.color;
    
    // Apply lighting
    let lit_color = calculate_lighting(in.world_position, sprite_color.rgb);
    
    return vec4<f32>(lit_color, sprite_color.a);
}