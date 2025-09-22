// Robin Shadow Mapping Shader

struct ShadowUniforms {
    light_view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> shadow_uniforms: ShadowUniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = shadow_uniforms.light_view_proj * vec4<f32>(model.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @builtin(frag_depth) f32 {
    // Just output depth - the depth buffer handles the rest
    return in.clip_position.z;
}