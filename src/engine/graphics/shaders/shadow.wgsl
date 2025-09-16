// Shadow mapping shader

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
    @location(0) depth: f32,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

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
    
    let world_position = model_matrix * vec4<f32>(vertex.position, 1.0);
    let clip_position = camera.view_projection_matrix * world_position;
    
    var out: VertexOutput;
    out.clip_position = clip_position;
    out.depth = clip_position.z / clip_position.w;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // For shadow mapping, we only need to write depth
    return vec4<f32>(in.depth, in.depth * in.depth, 0.0, 1.0);
}