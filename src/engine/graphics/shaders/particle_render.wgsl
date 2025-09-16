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

struct Particle {
    position: vec4<f32>,        // xyz + life
    velocity: vec4<f32>,        // xyz + mass
    color: vec4<f32>,           // rgba
    size_rotation: vec4<f32>,   // size, rotation, angular_vel, padding
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) world_position: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> camera_data: CameraUniform; // Duplicate binding for compatibility

@group(1) @binding(1)
var particle_texture: texture_2d<f32>;

@group(1) @binding(2)
var particle_sampler: sampler;

// Vertex shader generates a billboard quad for each particle
@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
    @location(0) particle_position: vec4<f32>,
    @location(1) particle_velocity: vec4<f32>,
    @location(2) particle_color: vec4<f32>,
    @location(3) particle_size_rotation: vec4<f32>,
) -> VertexOutput {
    // Skip dead particles
    if particle_position.w <= 0.0 {
        var out: VertexOutput;
        out.clip_position = vec4<f32>(0.0, 0.0, 0.0, 0.0);
        out.tex_coords = vec2<f32>(0.0, 0.0);
        out.color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
        out.world_position = vec3<f32>(0.0, 0.0, 0.0);
        return out;
    }
    
    // Generate quad vertices (triangle strip)
    var local_pos: vec2<f32>;
    var tex_coords: vec2<f32>;
    
    switch vertex_index {
        case 0u: {
            local_pos = vec2<f32>(-1.0, -1.0);
            tex_coords = vec2<f32>(0.0, 1.0);
        }
        case 1u: {
            local_pos = vec2<f32>(1.0, -1.0);
            tex_coords = vec2<f32>(1.0, 1.0);
        }
        case 2u: {
            local_pos = vec2<f32>(-1.0, 1.0);
            tex_coords = vec2<f32>(0.0, 0.0);
        }
        case 3u: {
            local_pos = vec2<f32>(1.0, 1.0);
            tex_coords = vec2<f32>(1.0, 0.0);
        }
        default: {
            local_pos = vec2<f32>(0.0, 0.0);
            tex_coords = vec2<f32>(0.5, 0.5);
        }
    }
    
    // Apply particle size
    local_pos *= particle_size_rotation.x;
    
    // Apply rotation
    let rotation = particle_size_rotation.y;
    let cos_rot = cos(rotation);
    let sin_rot = sin(rotation);
    let rotated_pos = vec2<f32>(
        local_pos.x * cos_rot - local_pos.y * sin_rot,
        local_pos.x * sin_rot + local_pos.y * cos_rot
    );
    
    // Create billboard matrix (face camera)
    let particle_world_pos = particle_position.xyz;
    let to_camera = normalize(camera.camera_position.xyz - particle_world_pos);
    let up = vec3<f32>(0.0, 1.0, 0.0);
    let right = normalize(cross(to_camera, up));
    let billboard_up = cross(right, to_camera);
    
    // Transform to world space
    let world_position = particle_world_pos + 
                        right * rotated_pos.x + 
                        billboard_up * rotated_pos.y;
    
    // Transform to clip space
    let clip_position = camera.view_projection_matrix * vec4<f32>(world_position, 1.0);
    
    var out: VertexOutput;
    out.clip_position = clip_position;
    out.tex_coords = tex_coords;
    out.color = particle_color;
    out.world_position = world_position;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample the particle texture
    var texture_color = textureSample(particle_texture, particle_sampler, in.tex_coords);
    
    // Apply particle color
    var final_color = texture_color * in.color;
    
    // Apply distance fade
    let distance_to_camera = length(camera.camera_position.xyz - in.world_position);
    let fade_start = 50.0;
    let fade_end = 100.0;
    let fade_factor = 1.0 - smoothstep(fade_start, fade_end, distance_to_camera);
    final_color.a *= fade_factor;
    
    // Soft particles effect (depth-based alpha)
    // This would require depth texture sampling, simplified for now
    
    // Alpha test
    if final_color.a < 0.01 {
        discard;
    }
    
    return final_color;
}

// Alternative simplified vertex shader for point sprites
@vertex
fn vs_point_sprite(
    @location(0) particle_position: vec4<f32>,
    @location(1) particle_velocity: vec4<f32>,
    @location(2) particle_color: vec4<f32>,
    @location(3) particle_size_rotation: vec4<f32>,
) -> VertexOutput {
    // Skip dead particles
    if particle_position.w <= 0.0 {
        var out: VertexOutput;
        out.clip_position = vec4<f32>(0.0, 0.0, 0.0, 0.0);
        out.tex_coords = vec2<f32>(0.0, 0.0);
        out.color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
        out.world_position = vec3<f32>(0.0, 0.0, 0.0);
        return out;
    }
    
    let world_position = particle_position.xyz;
    let clip_position = camera.view_projection_matrix * vec4<f32>(world_position, 1.0);
    
    var out: VertexOutput;
    out.clip_position = clip_position;
    out.tex_coords = vec2<f32>(0.5, 0.5);
    out.color = particle_color;
    out.world_position = world_position;
    
    return out;
}

// Fragment shader for point sprites
@fragment
fn fs_point_sprite(in: VertexOutput) -> @location(0) vec4<f32> {
    // Generate circular texture coordinates for point sprites
    let center = vec2<f32>(0.5, 0.5);
    let tex_coord = vec2<f32>(gl_PointCoord.x, 1.0 - gl_PointCoord.y);
    let distance_from_center = length(tex_coord - center);
    
    // Create circular particle
    if distance_from_center > 0.5 {
        discard;
    }
    
    // Soft edge falloff
    let alpha = 1.0 - smoothstep(0.3, 0.5, distance_from_center);
    
    var final_color = in.color;
    final_color.a *= alpha;
    
    // Apply distance fade
    let distance_to_camera = length(camera.camera_position.xyz - in.world_position);
    let fade_start = 50.0;
    let fade_end = 100.0;
    let fade_factor = 1.0 - smoothstep(fade_start, fade_end, distance_to_camera);
    final_color.a *= fade_factor;
    
    return final_color;
}