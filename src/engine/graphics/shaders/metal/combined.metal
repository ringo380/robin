#include <metal_stdlib>
using namespace metal;

// Shared structs
struct Uniforms {
    float4x4 view_proj;
    float4 view_pos;
    float4 light_pos;
    float time;
    float ambient_factor;  // Ambient lighting intensity (0.0 - 1.0)
    float light_intensity; // Light source intensity
    float _padding0;
};

struct VertexIn {
    float3 position [[attribute(0)]];
    float3 color [[attribute(1)]];
    float3 normal [[attribute(2)]];
    float2 tex_coords [[attribute(3)]];
};

struct VertexOut {
    float4 position [[position]];
    float3 world_position;
    float3 color;
    float3 normal;
    float2 tex_coords;
};

// Main voxel shaders
vertex VertexOut vertex_main(
    VertexIn in [[stage_in]],
    constant Uniforms& uniforms [[buffer(1)]]
) {
    VertexOut out;

    // Transform position
    float4 world_pos = float4(in.position, 1.0);
    out.position = uniforms.view_proj * world_pos;
    out.world_position = in.position;

    // Pass through other attributes
    out.color = in.color;
    out.normal = in.normal;
    out.tex_coords = in.tex_coords;

    return out;
}

fragment float4 fragment_main(
    VertexOut in [[stage_in]],
    constant Uniforms& uniforms [[buffer(1)]],
    texture2d<float> atlas_texture [[texture(0)]],
    sampler atlas_sampler [[sampler(0)]]
) {
    // Sample texture from atlas using UV coordinates
    float4 texture_color = atlas_texture.sample(atlas_sampler, in.tex_coords);
    float3 base_color = texture_color.rgb;

    // Calculate lighting
    float3 light_dir = normalize(uniforms.light_pos.xyz - in.world_position);
    float3 view_dir = normalize(uniforms.view_pos.xyz - in.world_position);
    float3 half_dir = normalize(light_dir + view_dir);

    // Ambient lighting (varies with time of day)
    float3 ambient = uniforms.ambient_factor * base_color;

    // Diffuse lighting (modulated by light intensity)
    float diff = max(dot(in.normal, light_dir), 0.0);
    float3 diffuse = diff * base_color * uniforms.light_intensity;

    // Specular lighting (modulated by light intensity)
    float spec = pow(max(dot(in.normal, half_dir), 0.0), 32.0);
    float3 specular = spec * float3(0.3) * uniforms.light_intensity;

    // Simple fog effect based on distance
    float distance = length(uniforms.view_pos.xyz - in.world_position);
    float fog_factor = 1.0 - smoothstep(50.0, 200.0, distance);

    // Combine lighting
    float3 final_color = ambient + diffuse + specular;

    // Add emissive glow for special materials (crystal and lava)
    if ((in.color.r > 0.6 && in.color.b > 0.9) ||  // Crystal (purple-ish)
        (in.color.r > 0.9 && in.color.g < 0.4)) {   // Lava (orange-red)
        final_color += base_color * 0.5 * (1.0 + sin(uniforms.time * 3.0) * 0.3);
    }

    // Apply fog
    float3 fog_color = float3(0.5, 0.8, 1.0);
    final_color = mix(final_color, fog_color, 1.0 - fog_factor);

    // Check if this is a ghost block (preview) and apply transparency
    float alpha = 1.0;
    if ((in.color.r < 0.1 && in.color.g > 0.9 && in.color.b < 0.1) ||  // Green ghost block (valid)
        (in.color.r > 0.9 && in.color.g < 0.1 && in.color.b < 0.1)) {   // Red ghost block (invalid)
        alpha = 0.3;
    }

    return float4(final_color, alpha);
}

// Sky shaders
struct SkyVertexOut {
    float4 position [[position]];
    float3 world_position;
};

vertex SkyVertexOut sky_vertex(
    VertexIn in [[stage_in]],
    constant Uniforms& uniforms [[buffer(1)]]
) {
    SkyVertexOut out;

    // Transform position (sky should be at far distance)
    float4 world_pos = float4(in.position * 1000.0, 1.0);
    out.position = uniforms.view_proj * world_pos;
    out.world_position = world_pos.xyz;

    return out;
}

fragment float4 sky_fragment(
    SkyVertexOut in [[stage_in]],
    constant Uniforms& uniforms [[buffer(1)]]
) {
    // Create a gradient sky based on Y position
    float t = normalize(in.world_position).y;

    // Sky colors
    float3 sky_color = mix(
        float3(0.8, 0.9, 1.0),  // Light blue at horizon
        float3(0.3, 0.6, 1.0),  // Darker blue at zenith
        clamp(t, 0.0, 1.0)
    );

    // Add time-based variation for day/night cycle
    float day_factor = (sin(uniforms.time * 0.1) + 1.0) * 0.5;
    sky_color = mix(
        float3(0.05, 0.05, 0.2),  // Night sky
        sky_color,                 // Day sky
        day_factor
    );

    return float4(sky_color, 1.0);
}

// Celestial body shaders (sun/moon)
struct CelestialUniforms {
    float4x4 model_view_proj;
    float4 celestial_color;
    float4 position;
    float intensity;
    float size;
    float _padding[2];
};

vertex VertexOut celestial_vertex(
    VertexIn in [[stage_in]],
    constant CelestialUniforms& uniforms [[buffer(1)]]
) {
    VertexOut out;

    // Scale by size and transform
    float3 scaled_pos = in.position * uniforms.size;
    float4 world_pos = float4(scaled_pos + uniforms.position.xyz, 1.0);
    out.position = uniforms.model_view_proj * world_pos;
    out.world_position = world_pos.xyz;
    out.color = uniforms.celestial_color.rgb;
    out.normal = in.normal;
    out.tex_coords = in.tex_coords;

    return out;
}

fragment float4 celestial_fragment(
    VertexOut in [[stage_in]],
    constant CelestialUniforms& uniforms [[buffer(1)]]
) {
    // Create circular gradient for celestial bodies
    float2 center = float2(0.5, 0.5);
    float distance = length(in.tex_coords - center);
    float intensity = 1.0 - smoothstep(0.3, 0.5, distance);

    float3 color = in.color * intensity * uniforms.intensity;

    // Add glow effect
    float glow = 1.0 - smoothstep(0.1, 0.8, distance);
    color += in.color * glow * 0.3;

    return float4(color, intensity);
}

// UI shaders for imgui integration
struct UIVertexIn {
    float2 position [[attribute(0)]];
    float2 tex_coords [[attribute(1)]];
    uint color [[attribute(2)]];
};

struct UIVertexOut {
    float4 position [[position]];
    float2 tex_coords;
    float4 color;
};

vertex UIVertexOut ui_vertex(
    UIVertexIn in [[stage_in]],
    constant float4x4& projection_matrix [[buffer(1)]]
) {
    UIVertexOut out;

    out.position = projection_matrix * float4(in.position, 0.0, 1.0);
    out.tex_coords = in.tex_coords;

    // Unpack RGBA color from uint
    uint color = in.color;
    out.color = float4(
        float((color >>  0) & 0xFF) / 255.0,
        float((color >>  8) & 0xFF) / 255.0,
        float((color >> 16) & 0xFF) / 255.0,
        float((color >> 24) & 0xFF) / 255.0
    );

    return out;
}

fragment float4 ui_fragment(
    UIVertexOut in [[stage_in]],
    texture2d<float> font_texture [[texture(0)]],
    sampler font_sampler [[sampler(0)]]
) {
    float4 texture_color = font_texture.sample(font_sampler, in.tex_coords);
    return in.color * texture_color;
}