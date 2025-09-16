// Robin Engine 2.0 - 3D Graphics Shader
// Modern WGSL shader for 3D rendering with dynamic effects

struct Uniforms {
    view_proj: mat4x4<f32>,
    time: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) world_position: vec3<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // Apply rotation animation based on time
    let rotation_y = uniforms.time * 0.5;
    let cos_y = cos(rotation_y);
    let sin_y = sin(rotation_y);
    
    // Rotation matrix around Y axis
    let rotation_matrix = mat4x4<f32>(
        cos_y, 0.0, sin_y, 0.0,
        0.0, 1.0, 0.0, 0.0,
        -sin_y, 0.0, cos_y, 0.0,
        0.0, 0.0, 0.0, 1.0
    );
    
    // Apply rotation to position
    let rotated_pos = rotation_matrix * vec4<f32>(model.position, 1.0);
    
    out.world_position = rotated_pos.xyz;
    out.clip_position = uniforms.view_proj * rotated_pos;
    
    // Enhance colors with time-based animation
    let color_intensity = 0.8 + 0.2 * sin(uniforms.time * 2.0);
    out.color = model.color * color_intensity;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Calculate distance from center for additional effects
    let distance_from_center = length(in.world_position);
    
    // Create a pulsing effect based on time and distance
    let pulse = 0.9 + 0.1 * sin(uniforms.time * 3.0 + distance_from_center);
    
    // Add some sparkle effect
    let sparkle = 0.95 + 0.05 * sin(uniforms.time * 10.0 + in.world_position.x * 5.0);
    
    // Combine effects
    let final_color = in.color * pulse * sparkle;
    
    // Add subtle alpha variation for visual interest
    let alpha = 0.95 + 0.05 * sin(uniforms.time + in.world_position.y);
    
    return vec4<f32>(final_color, alpha);
}