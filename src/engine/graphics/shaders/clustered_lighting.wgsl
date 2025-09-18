// Clustered Forward Lighting Compute Shader
// Performs lighting calculations using clustered light data

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

struct LightList {
    count: u32,
    light_indices: array<u32, 256>,
}

struct MaterialData {
    albedo: vec3<f32>,
    metallic: f32,
    roughness: f32,
    normal: vec3<f32>,
    ao: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: ClusterUniforms;

@group(0) @binding(1)
var<storage, read> lights: array<DynamicLight>;

@group(0) @binding(2)
var<storage, read> cluster_light_lists: array<LightList>;

@group(0) @binding(3)
var gbuffer_albedo: texture_2d<f32>;

@group(0) @binding(4)
var gbuffer_normal: texture_2d<f32>;

@group(0) @binding(5)
var gbuffer_material: texture_2d<f32>;

@group(0) @binding(6)
var depth_texture: texture_2d<f32>;

@group(0) @binding(7)
var lighting_output: texture_storage_2d<rgba16float, write>;

@group(0) @binding(8)
var gbuffer_sampler: sampler;

// Get cluster ID from world position
fn get_cluster_id(world_pos: vec3<f32>) -> u32 {
    // Transform to view space
    let view_pos = (uniforms.view_matrix * vec4<f32>(world_pos, 1.0)).xyz;

    // Calculate cluster coordinates
    let ndc_xy = (uniforms.proj_matrix * vec4<f32>(view_pos, 1.0)).xy;
    let cluster_x = u32(clamp((ndc_xy.x * 0.5 + 0.5) * f32(uniforms.cluster_dimensions.x), 0.0, f32(uniforms.cluster_dimensions.x - 1u)));
    let cluster_y = u32(clamp((ndc_xy.y * 0.5 + 0.5) * f32(uniforms.cluster_dimensions.y), 0.0, f32(uniforms.cluster_dimensions.y - 1u)));

    // Calculate Z slice using exponential distribution
    let z_linear = clamp((-view_pos.z - uniforms.z_near) / (uniforms.z_far - uniforms.z_near), 0.0, 1.0);
    let z_slice = log(mix(uniforms.z_near, uniforms.z_far, z_linear)) * uniforms.z_slice_scale + uniforms.z_slice_bias;
    let cluster_z = u32(clamp(z_slice * f32(uniforms.cluster_dimensions.z), 0.0, f32(uniforms.cluster_dimensions.z - 1u)));

    return cluster_z * uniforms.cluster_dimensions.x * uniforms.cluster_dimensions.y +
           cluster_y * uniforms.cluster_dimensions.x + cluster_x;
}

// Reconstruct world position from depth
fn screen_to_world_pos(screen_pos: vec2<f32>, depth: f32) -> vec3<f32> {
    let ndc = vec3<f32>(
        (screen_pos.x / uniforms.screen_size.x) * 2.0 - 1.0,
        (screen_pos.y / uniforms.screen_size.y) * 2.0 - 1.0,
        depth
    );

    let inv_view_proj = uniforms.inv_proj_matrix * inverse(uniforms.view_matrix);
    let world_pos = inv_view_proj * vec4<f32>(ndc, 1.0);
    return world_pos.xyz / world_pos.w;
}

// PBR lighting calculations
fn fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
    return f0 + (1.0 - f0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

fn distribution_ggx(n: vec3<f32>, h: vec3<f32>, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let ndoth = max(dot(n, h), 0.0);
    let ndoth2 = ndoth * ndoth;

    let num = a2;
    let denom = ndoth2 * (a2 - 1.0) + 1.0;
    return num / (3.14159265 * denom * denom);
}

fn geometry_schlick_ggx(ndotv: f32, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;
    return ndotv / (ndotv * (1.0 - k) + k);
}

fn geometry_smith(n: vec3<f32>, v: vec3<f32>, l: vec3<f32>, roughness: f32) -> f32 {
    let ndotv = max(dot(n, v), 0.0);
    let ndotl = max(dot(n, l), 0.0);
    let ggx2 = geometry_schlick_ggx(ndotv, roughness);
    let ggx1 = geometry_schlick_ggx(ndotl, roughness);
    return ggx1 * ggx2;
}

// Calculate lighting contribution from a single light
fn calculate_light_contribution(
    light: DynamicLight,
    world_pos: vec3<f32>,
    material: MaterialData
) -> vec3<f32> {
    let view_dir = normalize(-world_pos); // Camera at origin in world space
    let normal = normalize(material.normal);

    var light_dir: vec3<f32>;
    var attenuation = 1.0;
    var radiance: vec3<f32>;

    if (light.light_type == 0u) { // Point light
        light_dir = normalize(light.position - world_pos);
        let distance = length(light.position - world_pos);
        attenuation = 1.0 / (1.0 + 0.09 * distance + 0.032 * distance * distance);

        // Range check
        if (distance > light.range) {
            return vec3<f32>(0.0);
        }

        radiance = light.color * light.intensity * attenuation;
    } else if (light.light_type == 1u) { // Spot light
        light_dir = normalize(light.position - world_pos);
        let distance = length(light.position - world_pos);
        attenuation = 1.0 / (1.0 + 0.09 * distance + 0.032 * distance * distance);

        // Range check
        if (distance > light.range) {
            return vec3<f32>(0.0);
        }

        // Spot light cone
        let light_to_pixel = normalize(world_pos - light.position);
        let spot_factor = dot(light_to_pixel, normalize(light.direction));
        let spot_attenuation = smoothstep(light.outer_cone, light.inner_cone, spot_factor);

        radiance = light.color * light.intensity * attenuation * spot_attenuation;
    } else { // Directional light
        light_dir = normalize(-light.direction);
        radiance = light.color * light.intensity;
    }

    // PBR calculation
    let halfway = normalize(view_dir + light_dir);

    // Calculate F0 for dielectrics and metals
    let f0 = mix(vec3<f32>(0.04), material.albedo, material.metallic);

    // Cook-Torrance BRDF
    let ndf = distribution_ggx(normal, halfway, material.roughness);
    let g = geometry_smith(normal, view_dir, light_dir, material.roughness);
    let f = fresnel_schlick(max(dot(halfway, view_dir), 0.0), f0);

    let numerator = ndf * g * f;
    let denominator = 4.0 * max(dot(normal, view_dir), 0.0) * max(dot(normal, light_dir), 0.0) + 0.0001;
    let specular = numerator / denominator;

    // Energy conservation
    let ks = f;
    let kd = (vec3<f32>(1.0) - ks) * (1.0 - material.metallic);

    let ndotl = max(dot(normal, light_dir), 0.0);
    let diffuse = kd * material.albedo / 3.14159265;

    return (diffuse + specular) * radiance * ndotl;
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

    // Sample G-buffer
    let albedo = textureLoad(gbuffer_albedo, pixel_coord, 0).rgb;
    let normal = textureLoad(gbuffer_normal, pixel_coord, 0).rgb * 2.0 - 1.0;
    let material_data = textureLoad(gbuffer_material, pixel_coord, 0);
    let depth = textureLoad(depth_texture, pixel_coord, 0).r;

    // Skip sky pixels
    if (depth >= 0.999) {
        textureStore(lighting_output, pixel_coord, vec4<f32>(0.0, 0.0, 0.0, 1.0));
        return;
    }

    // Reconstruct world position
    let world_pos = screen_to_world_pos(pixel_pos, depth);

    // Create material structure
    let material = MaterialData(
        albedo,
        material_data.r, // metallic
        material_data.g, // roughness
        normal,
        material_data.b  // ao
    );

    // Get cluster for this pixel
    let cluster_id = get_cluster_id(world_pos);
    let light_list = cluster_light_lists[cluster_id];

    // Accumulate lighting
    var final_color = vec3<f32>(0.0);

    // Add ambient lighting
    let ambient = vec3<f32>(0.03) * albedo * material.ao;
    final_color += ambient;

    // Process all lights in this cluster
    for (var i = 0u; i < light_list.count; i++) {
        let light_idx = light_list.light_indices[i];
        let light = lights[light_idx];
        final_color += calculate_light_contribution(light, world_pos, material);
    }

    // Tone mapping (simple Reinhard)
    final_color = final_color / (final_color + vec3<f32>(1.0));

    // Gamma correction
    final_color = pow(final_color, vec3<f32>(1.0 / 2.2));

    textureStore(lighting_output, pixel_coord, vec4<f32>(final_color, 1.0));
}