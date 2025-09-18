// Motion Vector Generation Compute Shader
// Calculates per-pixel motion vectors for temporal effects

struct MotionUniforms {
    view_proj_matrix: mat4x4<f32>,
    prev_view_proj_matrix: mat4x4<f32>,
    inv_view_proj_matrix: mat4x4<f32>,
    screen_size: vec2<f32>,
    _padding: vec2<f32>,
}

@group(0) @binding(0)
var depth_texture: texture_2d<f32>;

@group(0) @binding(1)
var motion_output: texture_storage_2d<rg16float, write>;

@group(0) @binding(2)
var<uniform> uniforms: MotionUniforms;

// Reconstruct world position from depth
fn screen_to_world_pos(screen_pos: vec2<f32>, depth: f32, inv_view_proj: mat4x4<f32>) -> vec3<f32> {
    // Convert to NDC
    let ndc = vec3<f32>(
        (screen_pos.x / uniforms.screen_size.x) * 2.0 - 1.0,
        (screen_pos.y / uniforms.screen_size.y) * 2.0 - 1.0,
        depth
    );

    // Transform to world space
    let world_pos = inv_view_proj * vec4<f32>(ndc, 1.0);
    return world_pos.xyz / world_pos.w;
}

// Project world position to screen space
fn world_to_screen_pos(world_pos: vec3<f32>, view_proj: mat4x4<f32>) -> vec3<f32> {
    let clip_pos = view_proj * vec4<f32>(world_pos, 1.0);
    let ndc = clip_pos.xyz / clip_pos.w;

    return vec3<f32>(
        (ndc.x * 0.5 + 0.5) * uniforms.screen_size.x,
        (ndc.y * 0.5 + 0.5) * uniforms.screen_size.y,
        ndc.z
    );
}

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pixel_coord = vec2<i32>(global_id.xy);
    let screen_size_i = vec2<i32>(uniforms.screen_size);

    // Bounds check
    if (pixel_coord.x >= screen_size_i.x || pixel_coord.y >= screen_size_i.y) {
        return;
    }

    let pixel_pos = vec2<f32>(pixel_coord);

    // Sample depth
    let depth = textureLoad(depth_texture, pixel_coord, 0).r;

    // Skip sky pixels (depth = 1.0 typically means no geometry)
    if (depth >= 0.999) {
        textureStore(motion_output, pixel_coord, vec4<f32>(0.0, 0.0, 0.0, 0.0));
        return;
    }

    // Reconstruct world position from current frame
    let world_pos = screen_to_world_pos(pixel_pos, depth, uniforms.inv_view_proj_matrix);

    // Project world position using previous frame's matrix
    let prev_screen_pos = world_to_screen_pos(world_pos, uniforms.prev_view_proj_matrix);

    // Calculate motion vector (current - previous)
    let motion_vector = (pixel_pos - prev_screen_pos.xy) / uniforms.screen_size;

    // Store motion vector
    textureStore(motion_output, pixel_coord, vec4<f32>(motion_vector, 0.0, 0.0));
}