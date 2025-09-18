// GPU-Accelerated Voxel Face Culling Compute Shader
// Optimizes mesh generation by removing hidden faces

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

struct FaceCullingData {
    face_visible: array<u32, 6>, // Bit flags for each face
    neighbor_ao: array<f32, 6>,  // AO contribution from neighbors
}

@group(0) @binding(0)
var<uniform> uniforms: VoxelComputeUniforms;

@group(0) @binding(1)
var<storage, read> voxel_data: array<GPUVoxel>;

@group(0) @binding(2)
var<storage, read_write> culling_data: array<FaceCullingData>;

// Face direction vectors
const FACE_DIRECTIONS = array<vec3<i32>, 6>(
    vec3<i32>(1, 0, 0),   // Right  (+X)
    vec3<i32>(-1, 0, 0),  // Left   (-X)
    vec3<i32>(0, 1, 0),   // Up     (+Y)
    vec3<i32>(0, -1, 0),  // Down   (-Y)
    vec3<i32>(0, 0, 1),   // Front  (+Z)
    vec3<i32>(0, 0, -1),  // Back   (-Z)
);

// AO sample offsets for each face
const AO_SAMPLES = array<array<vec3<i32>, 8>, 6>(
    // Right face (+X) - sample around the face
    array<vec3<i32>, 8>(
        vec3<i32>(1, -1, -1), vec3<i32>(1, 0, -1), vec3<i32>(1, 1, -1),
        vec3<i32>(1, -1,  0),                       vec3<i32>(1, 1,  0),
        vec3<i32>(1, -1,  1), vec3<i32>(1, 0,  1), vec3<i32>(1, 1,  1),
    ),
    // Left face (-X)
    array<vec3<i32>, 8>(
        vec3<i32>(-1, -1, -1), vec3<i32>(-1, 0, -1), vec3<i32>(-1, 1, -1),
        vec3<i32>(-1, -1,  0),                        vec3<i32>(-1, 1,  0),
        vec3<i32>(-1, -1,  1), vec3<i32>(-1, 0,  1), vec3<i32>(-1, 1,  1),
    ),
    // Up face (+Y)
    array<vec3<i32>, 8>(
        vec3<i32>(-1, 1, -1), vec3<i32>(0, 1, -1), vec3<i32>(1, 1, -1),
        vec3<i32>(-1, 1,  0),                       vec3<i32>(1, 1,  0),
        vec3<i32>(-1, 1,  1), vec3<i32>(0, 1,  1), vec3<i32>(1, 1,  1),
    ),
    // Down face (-Y)
    array<vec3<i32>, 8>(
        vec3<i32>(-1, -1, -1), vec3<i32>(0, -1, -1), vec3<i32>(1, -1, -1),
        vec3<i32>(-1, -1,  0),                        vec3<i32>(1, -1,  0),
        vec3<i32>(-1, -1,  1), vec3<i32>(0, -1,  1), vec3<i32>(1, -1,  1),
    ),
    // Front face (+Z)
    array<vec3<i32>, 8>(
        vec3<i32>(-1, -1, 1), vec3<i32>(0, -1, 1), vec3<i32>(1, -1, 1),
        vec3<i32>(-1,  0, 1),                       vec3<i32>(1,  0, 1),
        vec3<i32>(-1,  1, 1), vec3<i32>(0,  1, 1), vec3<i32>(1,  1, 1),
    ),
    // Back face (-Z)
    array<vec3<i32>, 8>(
        vec3<i32>(-1, -1, -1), vec3<i32>(0, -1, -1), vec3<i32>(1, -1, -1),
        vec3<i32>(-1,  0, -1),                        vec3<i32>(1,  0, -1),
        vec3<i32>(-1,  1, -1), vec3<i32>(0,  1, -1), vec3<i32>(1,  1, -1),
    ),
);

fn get_voxel_index(pos: vec3<i32>) -> u32 {
    let chunk_size = vec3<i32>(uniforms.chunk_size);
    if (pos.x < 0 || pos.x >= chunk_size.x ||
        pos.y < 0 || pos.y >= chunk_size.y ||
        pos.z < 0 || pos.z >= chunk_size.z) {
        return 0u; // Out of bounds = empty
    }
    return u32(pos.x + pos.y * chunk_size.x + pos.z * chunk_size.x * chunk_size.y);
}

fn is_voxel_solid(pos: vec3<i32>) -> bool {
    let index = get_voxel_index(pos);
    let chunk_size = vec3<i32>(uniforms.chunk_size);

    // Out of bounds = empty (no occlusion)
    if (pos.x < 0 || pos.x >= chunk_size.x ||
        pos.y < 0 || pos.y >= chunk_size.y ||
        pos.z < 0 || pos.z >= chunk_size.z) {
        return false;
    }

    return voxel_data[index].voxel_type != 0u;
}

fn calculate_face_ao(voxel_pos: vec3<i32>, face_id: u32) -> f32 {
    var occlusion = 0.0;
    let samples = AO_SAMPLES[face_id];

    // Sample surrounding voxels
    for (var i = 0u; i < 8u; i++) {
        let sample_pos = voxel_pos + samples[i];
        if (is_voxel_solid(sample_pos)) {
            occlusion += 0.125; // 1/8 contribution per sample
        }
    }

    // Convert occlusion to brightness (1.0 = no occlusion, 0.0 = full occlusion)
    return 1.0 - occlusion;
}

fn should_render_face(voxel_pos: vec3<i32>, face_id: u32) -> bool {
    let face_dir = FACE_DIRECTIONS[face_id];
    let neighbor_pos = voxel_pos + face_dir;

    // If neighbor is solid, don't render this face
    return !is_voxel_solid(neighbor_pos);
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
    if (voxel.voxel_type == 0u) {
        return;
    }

    var face_data = FaceCullingData();

    // Analyze each face
    for (var face_id = 0u; face_id < 6u; face_id++) {
        // Check if face should be rendered
        if (should_render_face(voxel_pos, face_id)) {
            face_data.face_visible[face_id] = 1u;

            // Calculate ambient occlusion for this face
            face_data.neighbor_ao[face_id] = calculate_face_ao(voxel_pos, face_id);
        } else {
            face_data.face_visible[face_id] = 0u;
            face_data.neighbor_ao[face_id] = 0.0;
        }
    }

    // Store results
    culling_data[voxel_index] = face_data;
}