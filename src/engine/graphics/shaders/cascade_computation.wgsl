// Cascade Computation Shader
// Calculates optimal cascade splits and shadow matrices

struct CascadeUniforms {
    view_matrix: mat4x4<f32>,
    proj_matrix: mat4x4<f32>,
    light_direction: vec3<f32>,
    _padding1: f32,
    camera_near: f32,
    camera_far: f32,
    cascade_count: u32,
    _padding2: f32,
}

struct CascadeData {
    view_proj_matrix: mat4x4<f32>,
    far_plane: f32,
    _padding: vec3<f32>,
}

struct FrustumCorners {
    corners: array<vec3<f32>, 8>,
}

@group(0) @binding(0)
var<uniform> uniforms: CascadeUniforms;

@group(0) @binding(1)
var<storage, read_write> cascade_data: array<CascadeData>;

@group(0) @binding(2)
var<storage, read_write> cascade_splits: array<f32>;

// Calculate logarithmic cascade split
fn calculate_cascade_split(index: u32, total_cascades: u32, near: f32, far: f32) -> f32 {
    let ratio = far / near;
    let linear_split = near + (far - near) * f32(index) / f32(total_cascades);
    let log_split = near * pow(ratio, f32(index) / f32(total_cascades));

    // Blend between linear and logarithmic (0.9 weight for logarithmic)
    return mix(linear_split, log_split, 0.9);
}

// Create orthographic matrix
fn create_ortho_matrix(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> mat4x4<f32> {
    let rl = right - left;
    let tb = top - bottom;
    let fn_val = far - near;

    return mat4x4<f32>(
        vec4<f32>(2.0 / rl, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, 2.0 / tb, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, -2.0 / fn_val, 0.0),
        vec4<f32>(-(right + left) / rl, -(top + bottom) / tb, -(far + near) / fn_val, 1.0)
    );
}

// Create look-at matrix
fn create_look_at_matrix(eye: vec3<f32>, center: vec3<f32>, up: vec3<f32>) -> mat4x4<f32> {
    let f = normalize(center - eye);
    let s = normalize(cross(f, up));
    let u = cross(s, f);

    return mat4x4<f32>(
        vec4<f32>(s.x, u.x, -f.x, 0.0),
        vec4<f32>(s.y, u.y, -f.y, 0.0),
        vec4<f32>(s.z, u.z, -f.z, 0.0),
        vec4<f32>(-dot(s, eye), -dot(u, eye), dot(f, eye), 1.0)
    );
}

// Get frustum corners in world space
fn get_frustum_corners_world_space(view_matrix: mat4x4<f32>, proj_matrix: mat4x4<f32>) -> array<vec3<f32>, 8> {
    let inv = inverse(proj_matrix * view_matrix);

    var frustum_corners: array<vec3<f32>, 8>;
    var index = 0u;

    for (var x = 0u; x < 2u; x++) {
        for (var y = 0u; y < 2u; y++) {
            for (var z = 0u; z < 2u; z++) {
                let pt = inv * vec4<f32>(
                    f32(x) * 2.0 - 1.0,
                    f32(y) * 2.0 - 1.0,
                    f32(z) * 2.0 - 1.0,
                    1.0
                );
                frustum_corners[index] = pt.xyz / pt.w;
                index++;
            }
        }
    }

    return frustum_corners;
}

// Calculate bounding sphere for frustum corners
fn calculate_bounding_sphere(corners: array<vec3<f32>, 8>) -> vec4<f32> {
    // Calculate center
    var center = vec3<f32>(0.0);
    for (var i = 0u; i < 8u; i++) {
        center += corners[i];
    }
    center /= 8.0;

    // Calculate radius
    var radius = 0.0;
    for (var i = 0u; i < 8u; i++) {
        let dist = distance(corners[i], center);
        radius = max(radius, dist);
    }

    return vec4<f32>(center, radius);
}

@compute @workgroup_size(1, 1, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let cascade_index = global_id.x;

    if (cascade_index >= uniforms.cascade_count) {
        return;
    }

    // Calculate cascade split distances
    let near_split = select(
        uniforms.camera_near,
        cascade_splits[cascade_index - 1u],
        cascade_index > 0u
    );

    let far_split = calculate_cascade_split(
        cascade_index + 1u,
        uniforms.cascade_count,
        uniforms.camera_near,
        uniforms.camera_far
    );

    // Store split distance
    cascade_splits[cascade_index] = far_split;

    // Create projection matrix for this cascade
    let cascade_proj = perspective(
        45.0 * 3.14159265 / 180.0, // 45 degree FOV in radians
        1.0, // Aspect ratio (will be corrected)
        near_split,
        far_split
    );

    // Get frustum corners for this cascade
    let frustum_corners = get_frustum_corners_world_space(uniforms.view_matrix, cascade_proj);

    // Calculate bounding sphere
    let bounding_sphere = calculate_bounding_sphere(frustum_corners);
    let center = bounding_sphere.xyz;
    let radius = bounding_sphere.w;

    // Snap center to texel grid to reduce shimmering
    let texel_size = radius * 2.0 / 2048.0; // Assuming 2048x2048 shadow maps
    let snapped_center = vec3<f32>(
        floor(center.x / texel_size) * texel_size,
        floor(center.y / texel_size) * texel_size,
        floor(center.z / texel_size) * texel_size
    );

    // Create light view matrix
    let light_pos = snapped_center - normalize(uniforms.light_direction) * radius;
    let light_view = create_look_at_matrix(
        light_pos,
        snapped_center,
        vec3<f32>(0.0, 1.0, 0.0)
    );

    // Create light projection matrix (orthographic)
    let light_proj = create_ortho_matrix(
        -radius, radius,
        -radius, radius,
        0.1, radius * 2.0
    );

    // Store cascade data
    cascade_data[cascade_index] = CascadeData(
        light_proj * light_view,
        far_split,
        vec3<f32>(0.0)
    );
}

// Helper function to create perspective matrix
fn perspective(fovy: f32, aspect: f32, near: f32, far: f32) -> mat4x4<f32> {
    let f = 1.0 / tan(fovy / 2.0);
    let nf = 1.0 / (near - far);

    return mat4x4<f32>(
        vec4<f32>(f / aspect, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, f, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, (far + near) * nf, -1.0),
        vec4<f32>(0.0, 0.0, 2.0 * far * near * nf, 0.0)
    );
}