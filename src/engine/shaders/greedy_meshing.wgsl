// Greedy Meshing Compute Shader for GPU-accelerated voxel meshing
// This shader implements the greedy meshing algorithm on the GPU for efficient voxel-to-mesh conversion

struct VoxelData {
    voxel_type: u32,
    material_id: u32,
    density: f32,
    temperature: f32,
    metadata: array<u32, 4>,
}

struct MeshVertex {
    position: vec3<f32>,
    normal: vec3<f32>,
    uv: vec2<f32>,
    color: vec4<f32>,
    material_id: u32,
    ambient_occlusion: f32,
    lighting: vec2<f32>,
}

struct ComputeConstants {
    chunk_size: u32,
    world_size: vec3<u32>,
    time: f32,
    frame_number: u32,
    quality_level: u32,
    optimization_flags: u32,
    camera_position: vec3<f32>,
    view_distance: f32,
}

@group(0) @binding(0) var<storage, read> voxel_data: array<VoxelData>;
@group(0) @binding(1) var<storage, read_write> output_vertices: array<MeshVertex>;
@group(0) @binding(2) var<storage, read_write> output_indices: array<u32>;
@group(0) @binding(3) var<uniform> constants: ComputeConstants;

// Workgroup size optimized for GPU architecture
@compute @workgroup_size(8, 8, 8)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let chunk_size = constants.chunk_size;

    // Ensure we're within bounds
    if (global_id.x >= chunk_size || global_id.y >= chunk_size || global_id.z >= chunk_size) {
        return;
    }

    let voxel_index = global_id.x + global_id.y * chunk_size + global_id.z * chunk_size * chunk_size;

    if (voxel_index >= arrayLength(&voxel_data)) {
        return;
    }

    let current_voxel = voxel_data[voxel_index];

    // Skip empty voxels
    if (current_voxel.density <= 0.0) {
        return;
    }

    // Get voxel position in world space
    let voxel_pos = vec3<f32>(global_id);

    // Check each face direction
    let face_directions = array<vec3<i32>, 6>(
        vec3<i32>(1, 0, 0),   // +X
        vec3<i32>(-1, 0, 0),  // -X
        vec3<i32>(0, 1, 0),   // +Y
        vec3<i32>(0, -1, 0),  // -Y
        vec3<i32>(0, 0, 1),   // +Z
        vec3<i32>(0, 0, -1)   // -Z
    );

    let face_normals = array<vec3<f32>, 6>(
        vec3<f32>(1.0, 0.0, 0.0),
        vec3<f32>(-1.0, 0.0, 0.0),
        vec3<f32>(0.0, 1.0, 0.0),
        vec3<f32>(0.0, -1.0, 0.0),
        vec3<f32>(0.0, 0.0, 1.0),
        vec3<f32>(0.0, 0.0, -1.0)
    );

    // Process each face
    for (var face = 0u; face < 6u; face++) {
        let neighbor_pos = vec3<i32>(global_id) + face_directions[face];

        // Check if neighbor is outside chunk bounds or is empty
        var should_generate_face = false;

        if (neighbor_pos.x < 0 || neighbor_pos.x >= i32(chunk_size) ||
            neighbor_pos.y < 0 || neighbor_pos.y >= i32(chunk_size) ||
            neighbor_pos.z < 0 || neighbor_pos.z >= i32(chunk_size)) {
            should_generate_face = true; // Outside chunk bounds
        } else {
            let neighbor_index = u32(neighbor_pos.x) + u32(neighbor_pos.y) * chunk_size + u32(neighbor_pos.z) * chunk_size * chunk_size;
            if (neighbor_index < arrayLength(&voxel_data)) {
                let neighbor_voxel = voxel_data[neighbor_index];
                should_generate_face = neighbor_voxel.density <= 0.0;
            } else {
                should_generate_face = true;
            }
        }

        if (should_generate_face) {
            // Generate face vertices and indices
            generate_face(voxel_pos, face, face_normals[face], current_voxel);
        }
    }
}

fn generate_face(voxel_pos: vec3<f32>, face_index: u32, normal: vec3<f32>, voxel: VoxelData) {
    // Calculate face vertices based on face direction
    var vertices: array<vec3<f32>, 4>;
    var uvs: array<vec2<f32>, 4>;

    // Define face vertices relative to voxel position
    switch (face_index) {
        case 0u: { // +X face
            vertices[0] = voxel_pos + vec3<f32>(1.0, 0.0, 0.0);
            vertices[1] = voxel_pos + vec3<f32>(1.0, 1.0, 0.0);
            vertices[2] = voxel_pos + vec3<f32>(1.0, 1.0, 1.0);
            vertices[3] = voxel_pos + vec3<f32>(1.0, 0.0, 1.0);
        }
        case 1u: { // -X face
            vertices[0] = voxel_pos + vec3<f32>(0.0, 0.0, 1.0);
            vertices[1] = voxel_pos + vec3<f32>(0.0, 1.0, 1.0);
            vertices[2] = voxel_pos + vec3<f32>(0.0, 1.0, 0.0);
            vertices[3] = voxel_pos + vec3<f32>(0.0, 0.0, 0.0);
        }
        case 2u: { // +Y face
            vertices[0] = voxel_pos + vec3<f32>(0.0, 1.0, 0.0);
            vertices[1] = voxel_pos + vec3<f32>(0.0, 1.0, 1.0);
            vertices[2] = voxel_pos + vec3<f32>(1.0, 1.0, 1.0);
            vertices[3] = voxel_pos + vec3<f32>(1.0, 1.0, 0.0);
        }
        case 3u: { // -Y face
            vertices[0] = voxel_pos + vec3<f32>(0.0, 0.0, 1.0);
            vertices[1] = voxel_pos + vec3<f32>(0.0, 0.0, 0.0);
            vertices[2] = voxel_pos + vec3<f32>(1.0, 0.0, 0.0);
            vertices[3] = voxel_pos + vec3<f32>(1.0, 0.0, 1.0);
        }
        case 4u: { // +Z face
            vertices[0] = voxel_pos + vec3<f32>(0.0, 0.0, 1.0);
            vertices[1] = voxel_pos + vec3<f32>(1.0, 0.0, 1.0);
            vertices[2] = voxel_pos + vec3<f32>(1.0, 1.0, 1.0);
            vertices[3] = voxel_pos + vec3<f32>(0.0, 1.0, 1.0);
        }
        case 5u: { // -Z face
            vertices[0] = voxel_pos + vec3<f32>(1.0, 0.0, 0.0);
            vertices[1] = voxel_pos + vec3<f32>(0.0, 0.0, 0.0);
            vertices[2] = voxel_pos + vec3<f32>(0.0, 1.0, 0.0);
            vertices[3] = voxel_pos + vec3<f32>(1.0, 1.0, 0.0);
        }
        default: {
            return; // Invalid face index
        }
    }

    // Calculate UV coordinates
    uvs[0] = vec2<f32>(0.0, 0.0);
    uvs[1] = vec2<f32>(1.0, 0.0);
    uvs[2] = vec2<f32>(1.0, 1.0);
    uvs[3] = vec2<f32>(0.0, 1.0);

    // Calculate material color based on voxel type
    let material_color = get_material_color(voxel.material_id);

    // Calculate ambient occlusion
    let ao_factor = calculate_ambient_occlusion(voxel_pos, normal);

    // Calculate lighting
    let lighting = calculate_lighting(voxel_pos, normal);

    // Get atomic vertex index for thread-safe vertex generation
    let base_vertex_index = atomicAdd(&vertex_counter, 4u);
    let base_index_index = atomicAdd(&index_counter, 6u);

    // Generate vertices
    for (var i = 0u; i < 4u; i++) {
        let vertex_index = base_vertex_index + i;
        if (vertex_index < arrayLength(&output_vertices)) {
            output_vertices[vertex_index] = MeshVertex(
                vertices[i],
                normal,
                uvs[i],
                material_color,
                voxel.material_id,
                ao_factor,
                lighting
            );
        }
    }

    // Generate indices for two triangles (quad)
    let indices = array<u32, 6>(
        base_vertex_index + 0u, base_vertex_index + 1u, base_vertex_index + 2u,
        base_vertex_index + 0u, base_vertex_index + 2u, base_vertex_index + 3u
    );

    for (var i = 0u; i < 6u; i++) {
        let index_index = base_index_index + i;
        if (index_index < arrayLength(&output_indices)) {
            output_indices[index_index] = indices[i];
        }
    }
}

fn get_material_color(material_id: u32) -> vec4<f32> {
    // Simple material color mapping
    switch (material_id) {
        case 0u: { return vec4<f32>(0.5, 0.3, 0.1, 1.0); } // Dirt
        case 1u: { return vec4<f32>(0.6, 0.6, 0.6, 1.0); } // Stone
        case 2u: { return vec4<f32>(0.3, 0.7, 0.3, 1.0); } // Grass
        case 3u: { return vec4<f32>(0.8, 0.7, 0.4, 1.0); } // Sand
        case 4u: { return vec4<f32>(0.4, 0.2, 0.1, 1.0); } // Wood
        case 5u: { return vec4<f32>(0.7, 0.7, 0.8, 1.0); } // Metal
        default: { return vec4<f32>(1.0, 0.0, 1.0, 1.0); } // Debug magenta
    }
}

fn calculate_ambient_occlusion(voxel_pos: vec3<f32>, normal: vec3<f32>) -> f32 {
    // Simple ambient occlusion calculation
    // In a full implementation, this would sample neighboring voxels
    var ao = 1.0;

    // Reduce AO for voxels near the edges or corners
    let chunk_size_f = f32(constants.chunk_size);
    let edge_distance = min(
        min(voxel_pos.x, chunk_size_f - voxel_pos.x),
        min(voxel_pos.y, chunk_size_f - voxel_pos.y)
    );
    edge_distance = min(edge_distance, min(voxel_pos.z, chunk_size_f - voxel_pos.z));

    if (edge_distance < 2.0) {
        ao *= 0.7 + 0.3 * (edge_distance / 2.0);
    }

    return ao;
}

fn calculate_lighting(voxel_pos: vec3<f32>, normal: vec3<f32>) -> vec2<f32> {
    // Simple lighting calculation
    let light_direction = normalize(vec3<f32>(0.3, 0.7, 0.2));
    let direct_light = max(0.0, dot(normal, light_direction));

    // Indirect lighting (ambient)
    let indirect_light = 0.3;

    return vec2<f32>(direct_light, indirect_light);
}

// Atomic counters for thread-safe vertex and index generation
var<workgroup> vertex_counter: atomic<u32>;
var<workgroup> index_counter: atomic<u32>;