// Cluster Light Culling Compute Shader
// Assigns lights to 3D clusters for efficient forward rendering

struct ClusterUniforms {
    view_matrix: mat4x4<f32>,
    proj_matrix: mat4x4<f32>,
    inv_proj_matrix: mat4x4<f32>,
    screen_size: vec2<f32>,
    z_near: f32,
    z_far: f32,
    cluster_dimensions: vec3<u32>,
    z_slice_scale: f32,
    z_slice_bias: f32,
    light_count: u32,
}

struct DynamicLight {
    position: vec3<f32>,
    light_type: u32,  // 0=point, 1=spot, 2=directional
    color: vec3<f32>,
    intensity: f32,
    direction: vec3<f32>,
    range: f32,
    inner_cone: f32,
    outer_cone: f32,
}

struct ClusterAABB {
    min_bounds: vec3<f32>,
    max_bounds: vec3<f32>,
}

struct LightList {
    count: u32,
    light_indices: array<u32, 256>,
}

@group(0) @binding(0)
var<uniform> uniforms: ClusterUniforms;

@group(0) @binding(1)
var<storage, read> lights: array<DynamicLight>;

@group(0) @binding(2)
var<storage, read_write> cluster_aabbs: array<ClusterAABB>;

@group(0) @binding(3)
var<storage, read_write> cluster_light_lists: array<LightList>;

// Convert cluster index to 3D coordinates
fn cluster_id_to_coord(cluster_id: u32) -> vec3<u32> {
    let z = cluster_id / (uniforms.cluster_dimensions.x * uniforms.cluster_dimensions.y);
    let y = (cluster_id % (uniforms.cluster_dimensions.x * uniforms.cluster_dimensions.y)) / uniforms.cluster_dimensions.x;
    let x = cluster_id % uniforms.cluster_dimensions.x;
    return vec3<u32>(x, y, z);
}

// Convert 3D cluster coordinates to linear index
fn cluster_coord_to_id(coord: vec3<u32>) -> u32 {
    return coord.z * uniforms.cluster_dimensions.x * uniforms.cluster_dimensions.y +
           coord.y * uniforms.cluster_dimensions.x + coord.x;
}

// Calculate cluster AABB in view space
fn calculate_cluster_aabb(cluster_coord: vec3<u32>) -> ClusterAABB {
    let cluster_size = vec3<f32>(
        2.0 / f32(uniforms.cluster_dimensions.x),
        2.0 / f32(uniforms.cluster_dimensions.y),
        1.0 / f32(uniforms.cluster_dimensions.z)
    );

    // Calculate NDC bounds
    let ndc_min = vec3<f32>(
        f32(cluster_coord.x) * cluster_size.x - 1.0,
        f32(cluster_coord.y) * cluster_size.y - 1.0,
        f32(cluster_coord.z) * cluster_size.z
    );

    let ndc_max = vec3<f32>(
        f32(cluster_coord.x + 1u) * cluster_size.x - 1.0,
        f32(cluster_coord.y + 1u) * cluster_size.y - 1.0,
        f32(cluster_coord.z + 1u) * cluster_size.z
    );

    // Convert Z from linear to exponential distribution
    let z_near_exp = -uniforms.z_near;
    let z_far_exp = -uniforms.z_far;
    let z_min = -exp(mix(log(-z_near_exp), log(-z_far_exp), ndc_min.z));
    let z_max = -exp(mix(log(-z_near_exp), log(-z_far_exp), ndc_max.z));

    // Calculate view space bounds (frustum corners)
    let inv_proj = uniforms.inv_proj_matrix;

    // Calculate all 8 corners of the cluster frustum
    var min_bounds = vec3<f32>(1000000.0);
    var max_bounds = vec3<f32>(-1000000.0);

    for (var corner = 0u; corner < 8u; corner++) {
        let x = select(ndc_min.x, ndc_max.x, (corner & 1u) != 0u);
        let y = select(ndc_min.y, ndc_max.y, (corner & 2u) != 0u);
        let z = select(z_min, z_max, (corner & 4u) != 0u);

        // Project to view space
        let view_pos = inv_proj * vec4<f32>(x, y, z, 1.0);
        let view_pos_3d = view_pos.xyz / view_pos.w;

        min_bounds = min(min_bounds, view_pos_3d);
        max_bounds = max(max_bounds, view_pos_3d);
    }

    return ClusterAABB(min_bounds, max_bounds);
}

// Test if light intersects with cluster AABB
fn light_intersects_cluster(light: DynamicLight, aabb: ClusterAABB) -> bool {
    // Transform light to view space
    let light_view_pos = (uniforms.view_matrix * vec4<f32>(light.position, 1.0)).xyz;

    if (light.light_type == 0u) { // Point light
        // Sphere-AABB intersection
        let closest_point = clamp(light_view_pos, aabb.min_bounds, aabb.max_bounds);
        let distance = length(light_view_pos - closest_point);
        return distance <= light.range;
    } else if (light.light_type == 1u) { // Spot light
        // Cone-AABB intersection (simplified to sphere for now)
        let closest_point = clamp(light_view_pos, aabb.min_bounds, aabb.max_bounds);
        let distance = length(light_view_pos - closest_point);
        return distance <= light.range;
    } else { // Directional light
        // Directional lights affect all clusters
        return true;
    }
}

@compute @workgroup_size(64, 1, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let cluster_id = global_id.x;
    let total_clusters = uniforms.cluster_dimensions.x * uniforms.cluster_dimensions.y * uniforms.cluster_dimensions.z;

    if (cluster_id >= total_clusters) {
        return;
    }

    // Calculate cluster coordinates and AABB
    let cluster_coord = cluster_id_to_coord(cluster_id);
    let aabb = calculate_cluster_aabb(cluster_coord);

    // Store AABB for debugging/visualization
    cluster_aabbs[cluster_id] = aabb;

    // Reset light list for this cluster
    cluster_light_lists[cluster_id].count = 0u;

    // Test each light against this cluster
    var light_count = 0u;
    for (var light_idx = 0u; light_idx < uniforms.light_count && light_count < 256u; light_idx++) {
        let light = lights[light_idx];

        if (light_intersects_cluster(light, aabb)) {
            cluster_light_lists[cluster_id].light_indices[light_count] = light_idx;
            light_count++;
        }
    }

    cluster_light_lists[cluster_id].count = light_count;
}