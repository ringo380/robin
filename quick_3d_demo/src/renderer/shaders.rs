// Metal Shading Language (MSL) shaders as Rust strings
// Combined shader source to avoid duplicate definitions

pub const COMBINED_SHADER_SOURCE: &str = r#"
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
        alpha = 0.4; // Make ghost blocks 40% opaque (60% transparent)
    }

    return float4(final_color, alpha);
}

// ImGui UI shaders
struct UIVertexIn {
    float2 position [[attribute(0)]];
    float2 tex_coords [[attribute(1)]];
    float4 color [[attribute(2)]];
};

struct UIVertexOut {
    float4 position [[position]];
    float2 tex_coords;
    float4 color;
};

struct UIUniforms {
    float4x4 projection;
};

vertex UIVertexOut ui_vertex_main(
    UIVertexIn in [[stage_in]],
    constant UIUniforms& uniforms [[buffer(1)]]
) {
    UIVertexOut out;

    out.position = uniforms.projection * float4(in.position, 0.0, 1.0);
    out.tex_coords = in.tex_coords;
    out.color = in.color;

    return out;
}

fragment float4 ui_fragment_main(
    UIVertexOut in [[stage_in]],
    texture2d<float> font_texture [[texture(0)]],
    sampler font_sampler [[sampler(0)]]
) {
    float alpha = font_texture.sample(font_sampler, in.tex_coords).r;
    return float4(in.color.rgb, in.color.a * alpha);
}

// Sky rendering shaders
struct SkyVertexIn {
    float3 position [[attribute(0)]];
};

struct SkyVertexOut {
    float4 position [[position]];
    float3 world_position;
    float3 sky_direction;
};

vertex SkyVertexOut sky_vertex_main(
    SkyVertexIn in [[stage_in]],
    constant Uniforms& uniforms [[buffer(1)]]
) {
    SkyVertexOut out;

    // Create a large cube around the camera
    float3 world_pos = in.position * 1000.0 + uniforms.view_pos.xyz;
    out.position = uniforms.view_proj * float4(world_pos, 1.0);

    // Ensure sky is always rendered at maximum depth
    out.position.z = out.position.w * 0.999999;

    out.world_position = world_pos;
    out.sky_direction = normalize(in.position);

    return out;
}

// Sky gradient calculation based on time of day
float3 calculate_sky_color(float3 direction, float time_of_day) {
    // Normalize time to 0-1 (0 = midnight, 0.5 = noon)
    float day_progress = fmod(time_of_day, 24.0) / 24.0;

    // Calculate sun position (0 = horizon at sunrise, 1 = zenith at noon)
    float sun_height = sin(day_progress * 2.0 * M_PI_F - M_PI_F/2.0);

    // Sky colors for different times
    float3 night_color = float3(0.02, 0.02, 0.08);        // Dark blue
    float3 dawn_color = float3(0.8, 0.4, 0.2);            // Orange
    float3 day_color = float3(0.5, 0.7, 1.0);             // Light blue
    float3 dusk_color = float3(0.9, 0.5, 0.3);            // Orange-red

    // Horizon gradient effect based on view direction
    float horizon_factor = abs(direction.y);

    float3 sky_color;

    if (sun_height < -0.1) {
        // Night time
        sky_color = night_color;
    } else if (sun_height < 0.1) {
        // Dawn/dusk transition
        float transition = (sun_height + 0.1) / 0.2;
        if (day_progress < 0.5) {
            // Dawn
            sky_color = mix(night_color, dawn_color, transition);
        } else {
            // Dusk
            sky_color = mix(night_color, dusk_color, transition);
        }
    } else if (sun_height < 0.8) {
        // Day transition
        float transition = (sun_height - 0.1) / 0.7;
        if (day_progress < 0.5) {
            // Morning
            sky_color = mix(dawn_color, day_color, transition);
        } else {
            // Evening
            sky_color = mix(dusk_color, day_color, transition);
        }
    } else {
        // Full day
        sky_color = day_color;
    }

    // Add horizon effect
    float3 horizon_color = sky_color * 1.2;
    sky_color = mix(horizon_color, sky_color, horizon_factor);

    return sky_color;
}

fragment float4 sky_fragment_main(
    SkyVertexOut in [[stage_in]],
    constant Uniforms& uniforms [[buffer(1)]]
) {
    // Calculate time of day from uniforms.time (convert to hours)
    float time_of_day = fmod(uniforms.time * 0.1, 24.0); // 0.1 = time speed multiplier

    float3 sky_color = calculate_sky_color(in.sky_direction, time_of_day);

    // Add slight atmospheric scattering effect
    float atmosphere = pow(1.0 - abs(in.sky_direction.y), 2.0);
    sky_color += atmosphere * 0.1;

    return float4(sky_color, 1.0);
}

// Celestial body (Sun/Moon) shaders
struct CelestialVertexIn {
    float3 position [[attribute(0)]];
    float2 tex_coords [[attribute(1)]];
};

struct CelestialVertexOut {
    float4 position [[position]];
    float3 world_position;
    float2 tex_coords;
    float3 view_direction;
};

struct CelestialUniforms {
    float4x4 view_proj;
    float4 view_pos;
    float4 celestial_pos;  // xyz = position, w = scale
    float4 celestial_color; // rgb = color, a = intensity
    float time;
    float celestial_type;  // 0 = sun, 1 = moon
    float _padding0;
    float _padding1;
};

vertex CelestialVertexOut celestial_vertex_main(
    CelestialVertexIn in [[stage_in]],
    constant CelestialUniforms& uniforms [[buffer(1)]]
) {
    CelestialVertexOut out;

    // Scale and position the sphere
    float3 scaled_pos = in.position * uniforms.celestial_pos.w;
    float3 world_pos = scaled_pos + uniforms.celestial_pos.xyz;

    out.position = uniforms.view_proj * float4(world_pos, 1.0);
    out.world_position = world_pos;
    out.tex_coords = in.tex_coords;
    out.view_direction = normalize(uniforms.view_pos.xyz - world_pos);

    return out;
}

// Calculate sun/moon position based on time of day
float3 calculate_celestial_position(float time_of_day, float celestial_type, float3 view_pos) {
    // Normalize time to 0-1 (0 = midnight, 0.5 = noon)
    float day_progress = fmod(time_of_day, 24.0) / 24.0;

    // Calculate angle for celestial body movement
    float angle;
    if (celestial_type < 0.5) {
        // Sun: rises at 6AM (0.25), peaks at noon (0.5), sets at 6PM (0.75)
        angle = (day_progress - 0.25) * 2.0 * M_PI_F;
    } else {
        // Moon: opposite to sun, peaks at midnight
        angle = (day_progress + 0.25) * 2.0 * M_PI_F;
    }

    // Calculate position on arc across sky
    float radius = 800.0;  // Distance from camera
    float3 position;
    position.x = view_pos.x + radius * cos(angle);
    position.y = view_pos.y + radius * sin(angle);
    position.z = view_pos.z;

    return position;
}

fragment float4 celestial_fragment_main(
    CelestialVertexOut in [[stage_in]],
    constant CelestialUniforms& uniforms [[buffer(1)]]
) {
    // Calculate distance from center of sphere using UV coordinates
    float2 center = float2(0.5, 0.5);
    float2 uv_offset = in.tex_coords - center;
    float distance_from_center = length(uv_offset);

    // Create circular disc with soft edges
    float disc_factor = 1.0 - smoothstep(0.4, 0.5, distance_from_center);

    if (disc_factor <= 0.0) {
        discard_fragment();
    }

    // Calculate base color and glow effect
    float3 base_color = uniforms.celestial_color.rgb;
    float intensity = uniforms.celestial_color.a;

    // Add atmospheric glow
    float glow = pow(disc_factor, 0.5);
    float3 glow_color = base_color * intensity;

    // Different effects for sun vs moon
    if (uniforms.celestial_type < 0.5) {
        // Sun: bright yellow/orange with corona effect
        float corona = 1.0 - smoothstep(0.2, 0.5, distance_from_center);
        glow_color += float3(1.0, 0.8, 0.3) * corona * 0.5;
    } else {
        // Moon: softer white/blue with crater-like texture
        float crater_noise = sin(in.tex_coords.x * 20.0) * sin(in.tex_coords.y * 20.0) * 0.1;
        base_color += crater_noise * float3(0.2, 0.2, 0.3);
    }

    // Final color with glow
    float3 final_color = base_color * glow + glow_color * (1.0 - glow) * 0.5;

    // Alpha based on disc factor and intensity
    float alpha = disc_factor * intensity;

    return float4(final_color, alpha);
}
"#;