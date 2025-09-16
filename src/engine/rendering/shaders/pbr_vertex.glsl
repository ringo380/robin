#version 450 core

// Robin Engine - PBR Vertex Shader
// High-quality physically based rendering with instancing support

layout(location = 0) in vec3 a_position;
layout(location = 1) in vec3 a_normal;
layout(location = 2) in vec2 a_uv;
layout(location = 3) in vec3 a_tangent;
layout(location = 4) in vec3 a_bitangent;

// Instance data
layout(location = 5) in mat4 a_instance_transform;
layout(location = 9) in mat3 a_instance_normal_matrix;

// Camera uniforms
layout(binding = 0) uniform CameraUniforms {
    mat4 view_matrix;
    mat4 projection_matrix;
    mat4 view_projection_matrix;
    vec3 camera_position;
    float camera_near;
    float camera_far;
    float camera_fov;
};

// Lighting uniforms
layout(binding = 1) uniform LightingUniforms {
    uint light_count;
    vec3 ambient_color;
    float ambient_intensity;
    
    // Shadow mapping
    mat4 shadow_matrices[8]; // Up to 8 shadow maps
    uint shadow_map_count;
};

// Output to fragment shader
out VertexData {
    vec3 world_position;
    vec3 world_normal;
    vec2 uv;
    vec3 tangent;
    vec3 bitangent;
    vec3 view_direction;
    
    // Shadow mapping
    vec4 shadow_coords[8];
} vs_out;

void main() {
    // Transform position to world space
    vec4 world_pos = a_instance_transform * vec4(a_position, 1.0);
    vs_out.world_position = world_pos.xyz;
    
    // Transform normal, tangent, bitangent to world space
    vs_out.world_normal = normalize(a_instance_normal_matrix * a_normal);
    vs_out.tangent = normalize(a_instance_normal_matrix * a_tangent);
    vs_out.bitangent = normalize(a_instance_normal_matrix * a_bitangent);
    
    // Pass through UV coordinates
    vs_out.uv = a_uv;
    
    // Calculate view direction
    vs_out.view_direction = normalize(camera_position - vs_out.world_position);
    
    // Calculate shadow coordinates for each shadow map
    for (uint i = 0; i < shadow_map_count && i < 8; i++) {
        vs_out.shadow_coords[i] = shadow_matrices[i] * world_pos;
    }
    
    // Transform to clip space
    gl_Position = view_projection_matrix * world_pos;
}