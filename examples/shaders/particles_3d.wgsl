// GPU Particle System Shader for Robin Engine
// Handles particle rendering with billboarding and soft particles

struct ParticleInput {
    @location(0) position: vec3<f32>,
    @location(1) life: f32,
    @location(2) velocity: vec3<f32>,
    @location(3) size: f32,
    @location(4) color: vec4<f32>,
    @location(5) rotation: f32,
    @location(6) angular_velocity: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) world_position: vec3<f32>,
    @location(3) life_factor: f32,
}

struct CameraUniform {
    view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    time: f32,
    _padding: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var particle_texture: texture_2d<f32>;

@group(1) @binding(1)
var particle_sampler: sampler;

@group(1) @binding(2)
var depth_texture: texture_2d<f32>;

// Vertex shader - generates billboard quad for each particle
@vertex
fn vs_particle(
    particle: ParticleInput,
    @builtin(vertex_index) vertex_idx: u32
) -> VertexOutput {
    var out: VertexOutput;
    
    // Generate billboard vertices
    let quad_vertices = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0,  1.0)
    );
    
    let quad_uvs = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 0.0)
    );
    
    let vertex_pos = quad_vertices[vertex_idx % 6u];
    out.tex_coords = quad_uvs[vertex_idx % 6u];
    
    // Apply rotation
    let cos_rot = cos(particle.rotation);
    let sin_rot = sin(particle.rotation);
    let rotated_pos = vec2<f32>(
        vertex_pos.x * cos_rot - vertex_pos.y * sin_rot,
        vertex_pos.x * sin_rot + vertex_pos.y * cos_rot
    );
    
    // Billboard calculation - face camera
    let world_pos = vec4<f32>(particle.position, 1.0);
    let view_pos = camera.view * world_pos;
    
    // Add billboard offset in view space
    var billboard_pos = view_pos;
    billboard_pos.x += rotated_pos.x * particle.size;
    billboard_pos.y += rotated_pos.y * particle.size;
    
    // Project to clip space
    out.clip_position = camera.proj * billboard_pos;
    out.world_position = particle.position;
    
    // Pass particle data to fragment shader
    out.color = particle.color;
    out.life_factor = particle.life;
    
    return out;
}

// Fragment shader - renders particles with soft edges
@fragment
fn fs_particle(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample particle texture
    var tex_color = textureSample(particle_texture, particle_sampler, in.tex_coords);
    
    // If no texture bound, use procedural circle
    if (tex_color.a < 0.01) {
        let center_dist = length(in.tex_coords - vec2<f32>(0.5));
        if (center_dist > 0.5) {
            discard;
        }
        tex_color = vec4<f32>(1.0, 1.0, 1.0, 1.0 - smoothstep(0.3, 0.5, center_dist));
    }
    
    // Apply particle color and life-based fade
    var final_color = in.color * tex_color;
    
    // Fade out based on life
    let fade_in = smoothstep(0.0, 0.1, in.life_factor);
    let fade_out = smoothstep(0.0, 0.3, in.life_factor);
    final_color.a *= fade_in * fade_out;
    
    // Soft particles - fade near geometry
    let frag_depth = in.clip_position.z;
    let scene_depth = textureLoad(depth_texture, vec2<i32>(in.clip_position.xy), 0).r;
    let depth_fade = saturate((scene_depth - frag_depth) * 10.0);
    final_color.a *= depth_fade;
    
    // Energy/glow effect for emissive particles
    if (in.color.r + in.color.g + in.color.b > 2.5) {
        final_color.rgb *= 2.0; // Boost brightness for glowing particles
    }
    
    return final_color;
}

// Compute shader for particle physics simulation
@compute @workgroup_size(64)
fn cs_particle_update(
    @builtin(global_invocation_id) global_id: vec3<u32>
) {
    // Particle physics update would go here
    // This would update position, velocity, life, etc.
    // For now, this is handled on the CPU
}