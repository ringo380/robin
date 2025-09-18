// Robin Voxel Engine Shader

struct Uniforms {
    view_proj: mat4x4<f32>,
    view_pos: vec4<f32>,
    light_pos: vec4<f32>,
    time: f32,
    _padding: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    let world_position = model.position;
    out.world_position = world_position;
    out.clip_position = uniforms.view_proj * vec4<f32>(world_position, 1.0);
    out.color = model.color;
    out.normal = model.normal;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Calculate lighting
    let light_dir = normalize(uniforms.light_pos.xyz - in.world_position);
    let view_dir = normalize(uniforms.view_pos.xyz - in.world_position);
    let half_dir = normalize(light_dir + view_dir);

    // Ambient
    let ambient = 0.3 * in.color;

    // Diffuse
    let diff = max(dot(in.normal, light_dir), 0.0);
    let diffuse = diff * in.color;

    // Specular
    let spec = pow(max(dot(in.normal, half_dir), 0.0), 32.0);
    let specular = spec * vec3<f32>(0.3);

    // Simple fog effect based on distance
    let distance = length(uniforms.view_pos.xyz - in.world_position);
    let fog_factor = 1.0 - smoothstep(50.0, 200.0, distance);

    // Combine lighting
    var final_color = ambient + diffuse + specular;

    // Add emissive glow for certain materials (crystal and lava)
    if (in.color.r > 0.6 && in.color.b > 0.9) || // Crystal (purple-ish)
       (in.color.r > 0.9 && in.color.g < 0.4) {   // Lava (orange-red)
        final_color += in.color * 0.5 * (1.0 + sin(uniforms.time * 3.0) * 0.3);
    }

    // Apply fog
    let fog_color = vec3<f32>(0.5, 0.8, 1.0);
    final_color = mix(final_color, fog_color, 1.0 - fog_factor);

    return vec4<f32>(final_color, 1.0);
}