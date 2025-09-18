// Shadow Vertex Shader for depth-only rendering

struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct CameraUniforms {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniforms;

@vertex
fn vs_main(input: VertexInput) -> @builtin(position) vec4<f32> {
    return camera.view_proj * vec4<f32>(input.position, 1.0);
}