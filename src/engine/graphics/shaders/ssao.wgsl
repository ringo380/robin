// Screen-Space Ambient Occlusion (SSAO) Compute Shader
// Implements hemisphere sampling with temporal noise reduction

struct SSAOUniforms {
    view_matrix: mat4x4<f32>,
    proj_matrix: mat4x4<f32>,
    inv_proj_matrix: mat4x4<f32>,
    sample_count: u32,
    radius: f32,
    bias: f32,
    power: f32,
    intensity: f32,
    noise_scale: f32,
    screen_size: vec2<f32>,
}

@group(0) @binding(0)
var depth_texture: texture_2d<f32>;

@group(0) @binding(1)
var normal_texture: texture_2d<f32>;

@group(0) @binding(2)
var noise_texture: texture_2d<f32>;

@group(0) @binding(3)
var texture_sampler: sampler;

@group(0) @binding(4)
var ssao_output: texture_storage_2d<r32float, write>;

@group(0) @binding(5)
var<uniform> uniforms: SSAOUniforms;

@group(0) @binding(6)
var<storage, read> kernel_samples: array<vec3<f32>>;

// Convert screen position to view space position
fn screen_to_view_pos(screen_pos: vec2<f32>, depth: f32) -> vec3<f32> {
    // Convert to NDC
    let ndc = vec3<f32>(
        (screen_pos.x / uniforms.screen_size.x) * 2.0 - 1.0,
        (screen_pos.y / uniforms.screen_size.y) * 2.0 - 1.0,
        depth
    );

    // Transform to view space
    let view_pos = uniforms.inv_proj_matrix * vec4<f32>(ndc, 1.0);
    return view_pos.xyz / view_pos.w;
}

// Convert view space position back to screen space
fn view_to_screen_pos(view_pos: vec3<f32>) -> vec3<f32> {
    let clip_pos = uniforms.proj_matrix * vec4<f32>(view_pos, 1.0);
    let ndc = clip_pos.xyz / clip_pos.w;

    return vec3<f32>(
        (ndc.x * 0.5 + 0.5) * uniforms.screen_size.x,
        (ndc.y * 0.5 + 0.5) * uniforms.screen_size.y,
        ndc.z
    );
}

// Generate TBN matrix for hemisphere sampling
fn generate_tbn_matrix(normal: vec3<f32>, noise: vec3<f32>) -> mat3x3<f32> {
    // Create tangent vector perpendicular to normal
    let tangent = normalize(noise - normal * dot(noise, normal));
    let bitangent = cross(normal, tangent);

    return mat3x3<f32>(tangent, bitangent, normal);
}

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pixel_coords = vec2<i32>(global_id.xy);
    let screen_size_i = vec2<i32>(uniforms.screen_size);

    // Bounds check
    if (pixel_coords.x >= screen_size_i.x || pixel_coords.y >= screen_size_i.y) {
        return;
    }

    let pixel_pos = vec2<f32>(pixel_coords);

    // Sample depth and normal
    let depth = textureLoad(depth_texture, pixel_coords, 0).r;
    let normal_sample = textureLoad(normal_texture, pixel_coords, 0).xyz;

    // Early exit if no geometry (depth = 1.0 typically means sky)
    if (depth >= 0.999) {
        textureStore(ssao_output, pixel_coords, vec4<f32>(1.0));
        return;
    }

    // Reconstruct view space position
    let view_pos = screen_to_view_pos(pixel_pos, depth);
    let view_normal = normalize((uniforms.view_matrix * vec4<f32>(normal_sample, 0.0)).xyz);

    // Sample noise for rotation
    let noise_coords = vec2<i32>(pixel_coords) % 4; // 4x4 noise texture
    let noise = textureLoad(noise_texture, noise_coords, 0).xyz;

    // Generate TBN matrix for hemisphere sampling
    let tbn = generate_tbn_matrix(view_normal, noise);

    // Accumulate occlusion
    var occlusion = 0.0;
    let sample_count_f = f32(uniforms.sample_count);

    for (var i = 0u; i < uniforms.sample_count; i++) {
        // Get sample vector in hemisphere
        let sample_vec = tbn * kernel_samples[i];
        let sample_pos = view_pos + sample_vec * uniforms.radius;

        // Project sample to screen space
        let sample_screen = view_to_screen_pos(sample_pos);

        // Check if sample is within screen bounds
        if (sample_screen.x < 0.0 || sample_screen.x >= uniforms.screen_size.x ||
            sample_screen.y < 0.0 || sample_screen.y >= uniforms.screen_size.y) {
            continue;
        }

        // Sample depth at projected position
        let sample_coords = vec2<i32>(sample_screen.xy);
        let sample_depth = textureLoad(depth_texture, sample_coords, 0).r;
        let sample_view_pos = screen_to_view_pos(sample_screen.xy, sample_depth);

        // Range check - prevent false occlusion from distant geometry
        let range_check = abs(view_pos.z - sample_view_pos.z) < uniforms.radius;

        // Occlusion test
        let depth_diff = sample_view_pos.z - sample_pos.z;
        let occlusion_factor = f32(depth_diff > uniforms.bias) * f32(range_check);

        occlusion += occlusion_factor;
    }

    // Normalize and apply intensity
    occlusion = occlusion / sample_count_f;
    occlusion = 1.0 - occlusion;
    occlusion = pow(occlusion, uniforms.power);
    occlusion = mix(1.0, occlusion, uniforms.intensity);

    // Store result
    textureStore(ssao_output, pixel_coords, vec4<f32>(occlusion));
}