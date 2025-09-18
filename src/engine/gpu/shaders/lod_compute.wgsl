// LOD Compute Shader for GPU-based Level of Detail selection
// Calculates optimal LOD levels based on distance from camera

struct CameraData {
    view_matrix: mat4x4<f32>,
    position: vec4<f32>,
};

struct LODEntry {
    position: vec3<f32>,
    radius: f32,
};

struct LODSettings {
    base_distance: f32,
    distance_multiplier: f32,
    max_lod_level: u32,
    quality_bias: f32,
};

@group(0) @binding(0) var<uniform> camera: CameraData;
@group(0) @binding(1) var<uniform> settings: LODSettings;
@group(0) @binding(2) var<storage, read> lod_entries: array<LODEntry>;
@group(0) @binding(3) var<storage, read_write> lod_results: array<u32>;

@compute @workgroup_size(64)
fn compute_lod(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;

    // Bounds check
    if (index >= arrayLength(&lod_entries)) {
        return;
    }

    let entry = lod_entries[index];
    let camera_pos = camera.position.xyz;
    let object_pos = entry.position;

    // Calculate distance from camera to object
    let distance = length(object_pos - camera_pos);

    // Account for object radius in distance calculation
    let effective_distance = max(0.0, distance - entry.radius);

    // Calculate LOD level based on distance
    let normalized_distance = effective_distance / settings.base_distance;
    let lod_level_f = log2(max(1.0, normalized_distance * settings.distance_multiplier));

    // Apply quality bias (higher bias = higher quality = lower LOD levels)
    let biased_lod = lod_level_f - settings.quality_bias;

    // Clamp to valid LOD range
    let final_lod = clamp(u32(biased_lod), 0u, settings.max_lod_level);

    // Store result
    lod_results[index] = final_lod;
}

// Alternative frustum culling + LOD computation
@compute @workgroup_size(64)
fn compute_lod_with_culling(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;

    if (index >= arrayLength(&lod_entries)) {
        return;
    }

    let entry = lod_entries[index];
    let camera_pos = camera.position.xyz;
    let object_pos = entry.position;

    // Transform object position to clip space for frustum culling
    let clip_pos = camera.view_matrix * vec4<f32>(object_pos, 1.0);

    // Simple frustum culling check
    let in_frustum = clip_pos.z > 0.0 &&
                    abs(clip_pos.x) < clip_pos.w + entry.radius &&
                    abs(clip_pos.y) < clip_pos.w + entry.radius;

    if (!in_frustum) {
        // Object is outside frustum, set to max LOD (or cull completely)
        lod_results[index] = 0xFFFFFFFFu; // Special value indicating culled
        return;
    }

    // Calculate distance-based LOD as before
    let distance = length(object_pos - camera_pos);
    let effective_distance = max(0.0, distance - entry.radius);
    let normalized_distance = effective_distance / settings.base_distance;
    let lod_level_f = log2(max(1.0, normalized_distance * settings.distance_multiplier));
    let biased_lod = lod_level_f - settings.quality_bias;
    let final_lod = clamp(u32(biased_lod), 0u, settings.max_lod_level);

    lod_results[index] = final_lod;
}