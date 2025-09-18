// GPU-Accelerated Voxel Mesh Generation Compute Shader
// Generates mesh vertices and indices for voxel chunks in parallel

struct VoxelComputeUniforms {
    chunk_size: vec3<u32>,
    chunk_position: vec3<f32>,
    face_culling_enabled: u32,
    lod_level: u32,
    vertex_count: u32,
    index_count: u32,
    padding: vec2<u32>,
}

struct GPUVoxel {
    voxel_type: u32,
    material_id: u32,
    ao_data: u32,
    light_level: u32,
}

struct GPUVertex {
    position: vec3<f32>,
    normal: vec3<f32>,
    uv: vec2<f32>,
    material_id: u32,
    ao_factor: f32,
    padding: vec2<u32>,
}

@group(0) @binding(0)
var<uniform> uniforms: VoxelComputeUniforms;

@group(0) @binding(1)
var<storage, read> voxel_data: array<GPUVoxel>;

@group(0) @binding(2)
var<storage, read_write> vertex_output: array<GPUVertex>;

@group(0) @binding(3)
var<storage, read_write> index_output: array<u32>;

@group(0) @binding(4)
var<storage, read_write> vertex_counter: atomic<u32>;

@group(0) @binding(5)
var<storage, read_write> index_counter: atomic<u32>;

// Face directions and normals
const FACE_DIRECTIONS = array<vec3<i32>, 6>(
    vec3<i32>(1, 0, 0),   // Right  (+X)
    vec3<i32>(-1, 0, 0),  // Left   (-X)
    vec3<i32>(0, 1, 0),   // Up     (+Y)
    vec3<i32>(0, -1, 0),  // Down   (-Y)
    vec3<i32>(0, 0, 1),   // Front  (+Z)
    vec3<i32>(0, 0, -1),  // Back   (-Z)
);

const FACE_NORMALS = array<vec3<f32>, 6>(
    vec3<f32>(1.0, 0.0, 0.0),   // Right
    vec3<f32>(-1.0, 0.0, 0.0),  // Left
    vec3<f32>(0.0, 1.0, 0.0),   // Up
    vec3<f32>(0.0, -1.0, 0.0),  // Down
    vec3<f32>(0.0, 0.0, 1.0),   // Front
    vec3<f32>(0.0, 0.0, -1.0),  // Back
);

// Vertex positions for each face (relative to voxel center)
const FACE_VERTICES = array<array<vec3<f32>, 4>, 6>(
    // Right face (+X)
    array<vec3<f32>, 4>(
        vec3<f32>(0.5, -0.5, -0.5),
        vec3<f32>(0.5,  0.5, -0.5),
        vec3<f32>(0.5,  0.5,  0.5),
        vec3<f32>(0.5, -0.5,  0.5),
    ),
    // Left face (-X)
    array<vec3<f32>, 4>(
        vec3<f32>(-0.5, -0.5,  0.5),
        vec3<f32>(-0.5,  0.5,  0.5),
        vec3<f32>(-0.5,  0.5, -0.5),
        vec3<f32>(-0.5, -0.5, -0.5),
    ),
    // Up face (+Y)
    array<vec3<f32>, 4>(
        vec3<f32>(-0.5, 0.5, -0.5),
        vec3<f32>( 0.5, 0.5, -0.5),
        vec3<f32>( 0.5, 0.5,  0.5),
        vec3<f32>(-0.5, 0.5,  0.5),
    ),
    // Down face (-Y)
    array<vec3<f32>, 4>(
        vec3<f32>(-0.5, -0.5,  0.5),
        vec3<f32>( 0.5, -0.5,  0.5),
        vec3<f32>( 0.5, -0.5, -0.5),
        vec3<f32>(-0.5, -0.5, -0.5),
    ),
    // Front face (+Z)
    array<vec3<f32>, 4>(
        vec3<f32>(-0.5, -0.5, 0.5),
        vec3<f32>( 0.5, -0.5, 0.5),
        vec3<f32>( 0.5,  0.5, 0.5),
        vec3<f32>(-0.5,  0.5, 0.5),
    ),
    // Back face (-Z)
    array<vec3<f32>, 4>(
        vec3<f32>( 0.5, -0.5, -0.5),
        vec3<f32>(-0.5, -0.5, -0.5),
        vec3<f32>(-0.5,  0.5, -0.5),
        vec3<f32>( 0.5,  0.5, -0.5),
    ),
);

// UV coordinates for faces
const FACE_UVS = array<vec2<f32>, 4>(
    vec2<f32>(0.0, 1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(1.0, 0.0),
    vec2<f32>(0.0, 0.0),
);

// Face indices (two triangles per face)
const FACE_INDICES = array<u32, 6>(0u, 1u, 2u, 0u, 2u, 3u);

fn get_voxel_index(pos: vec3<i32>) -> u32 {
    let chunk_size = vec3<i32>(uniforms.chunk_size);
    if (pos.x < 0 || pos.x >= chunk_size.x ||
        pos.y < 0 || pos.y >= chunk_size.y ||
        pos.z < 0 || pos.z >= chunk_size.z) {
        return 0u; // Out of bounds = empty
    }
    return u32(pos.x + pos.y * chunk_size.x + pos.z * chunk_size.x * chunk_size.y);
}

fn is_voxel_solid(voxel: GPUVoxel) -> bool {
    return voxel.voxel_type != 0u; // 0 = empty/air
}

fn should_render_face(voxel_pos: vec3<i32>, face_dir: vec3<i32>) -> bool {
    let neighbor_pos = voxel_pos + face_dir;
    let neighbor_index = get_voxel_index(neighbor_pos);

    // If neighbor is out of bounds, render the face
    let chunk_size = vec3<i32>(uniforms.chunk_size);
    if (neighbor_pos.x < 0 || neighbor_pos.x >= chunk_size.x ||
        neighbor_pos.y < 0 || neighbor_pos.y >= chunk_size.y ||
        neighbor_pos.z < 0 || neighbor_pos.z >= chunk_size.z) {
        return true;
    }

    // Render face if neighbor is not solid
    return !is_voxel_solid(voxel_data[neighbor_index]);
}

fn calculate_ambient_occlusion(voxel_pos: vec3<i32>, face_normal: vec3<f32>) -> f32 {
    // Simplified AO calculation
    // In a full implementation, this would sample neighboring voxels
    return 1.0; // No occlusion for now
}

@compute @workgroup_size(8, 8, 8)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let voxel_pos = vec3<i32>(global_id);
    let chunk_size = vec3<i32>(uniforms.chunk_size);

    // Check bounds
    if (voxel_pos.x >= chunk_size.x ||
        voxel_pos.y >= chunk_size.y ||
        voxel_pos.z >= chunk_size.z) {
        return;
    }

    let voxel_index = get_voxel_index(voxel_pos);
    let voxel = voxel_data[voxel_index];

    // Skip empty voxels
    if (!is_voxel_solid(voxel)) {
        return;
    }

    let world_pos = vec3<f32>(voxel_pos) + uniforms.chunk_position;

    // Generate faces for this voxel
    for (var face_id = 0u; face_id < 6u; face_id++) {
        let face_dir = FACE_DIRECTIONS[face_id];

        // Skip hidden faces if face culling is enabled
        if (uniforms.face_culling_enabled != 0u && !should_render_face(voxel_pos, face_dir)) {
            continue;
        }

        let face_normal = FACE_NORMALS[face_id];
        let ao_factor = calculate_ambient_occlusion(voxel_pos, face_normal);

        // Generate 4 vertices for this face
        let base_vertex_index = atomicAdd(&vertex_counter, 4u);
        let base_index_index = atomicAdd(&index_counter, 6u);

        for (var i = 0u; i < 4u; i++) {
            let vertex_pos = world_pos + FACE_VERTICES[face_id][i];

            vertex_output[base_vertex_index + i] = GPUVertex(
                vertex_pos,
                face_normal,
                FACE_UVS[i],
                voxel.material_id,
                ao_factor,
                vec2<u32>(0u, 0u),
            );
        }

        // Generate indices for two triangles
        for (var i = 0u; i < 6u; i++) {
            index_output[base_index_index + i] = base_vertex_index + FACE_INDICES[i];
        }
    }
}