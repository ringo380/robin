// Shadow Depth Vertex Shader
// Renders geometry from light's perspective for shadow mapping

struct ShadowUniforms {
    light_view_proj_matrices: array<mat4x4<f32>, 4>,
    cascade_splits: array<f32, 5>,
    light_direction: vec3<f32>,
    _padding1: f32,
    depth_bias: f32,
    normal_bias: f32,
    pcf_radius: f32,
    cascade_count: u32,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
}

struct InstanceData {
    @location(2) model_matrix_0: vec4<f32>,
    @location(3) model_matrix_1: vec4<f32>,
    @location(4) model_matrix_2: vec4<f32>,
    @location(5) model_matrix_3: vec4<f32>,
    @location(6) cascade_index: u32,
}

@group(0) @binding(0)
var<uniform> uniforms: ShadowUniforms;

@vertex
fn vs_main(
    vertex: VertexInput,
    instance: InstanceData
) -> VertexOutput {
    // Reconstruct model matrix from instance data
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3
    );

    // Transform to world space
    let world_position = model_matrix * vec4<f32>(vertex.position, 1.0);
    let world_normal = normalize((model_matrix * vec4<f32>(vertex.normal, 0.0)).xyz);

    // Apply normal bias to reduce shadow acne
    let biased_world_pos = world_position.xyz + world_normal * uniforms.normal_bias;

    // Transform to light clip space using appropriate cascade
    let cascade_idx = min(instance.cascade_index, uniforms.cascade_count - 1u);
    let light_view_proj = uniforms.light_view_proj_matrices[cascade_idx];
    let clip_position = light_view_proj * vec4<f32>(biased_world_pos, 1.0);

    var output: VertexOutput;
    output.clip_position = clip_position;
    output.world_position = world_position.xyz;
    output.world_normal = world_normal;

    return output;
}

// Optional fragment shader for alpha testing
@fragment
fn fs_main(input: VertexOutput) -> @location(0) f32 {
    // For alpha-tested materials, discard fragments here
    // For now, just output depth
    return input.clip_position.z;
}