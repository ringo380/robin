#version 450 core

// Robin Engine - PBR Fragment Shader
// Physically Based Rendering with metallic-roughness workflow

// Input from vertex shader
in VertexData {
    vec3 world_position;
    vec3 world_normal;
    vec2 uv;
    vec3 tangent;
    vec3 bitangent;
    vec3 view_direction;
    
    // Shadow mapping
    vec4 shadow_coords[8];
} fs_in;

// Material uniforms
layout(binding = 2) uniform MaterialUniforms {
    vec4 base_color;
    vec3 emissive;
    float metallic;
    float roughness;
    float normal_scale;
    float emissive_strength;
    float alpha_cutoff;
    uint shader_flags;
    
    // Texture samplers
    int albedo_sampler;
    int metallic_roughness_sampler;
    int normal_sampler;
    int occlusion_sampler;
    int emissive_sampler;
};

// Light data structure
struct Light {
    vec3 position;
    uint type; // 0=directional, 1=point, 2=spot
    vec3 direction;
    float intensity;
    vec3 color;
    float range;
    float inner_cone_angle;
    float outer_cone_angle;
};

// Lighting uniforms
layout(binding = 1) uniform LightingUniforms {
    uint light_count;
    vec3 ambient_color;
    float ambient_intensity;
    
    // Shadow mapping
    mat4 shadow_matrices[8];
    uint shadow_map_count;
};

// Light buffer
layout(binding = 3, std430) readonly buffer LightBuffer {
    Light lights[];
};

// Material textures
layout(binding = 0) uniform sampler2D albedo_texture;
layout(binding = 1) uniform sampler2D metallic_roughness_texture;
layout(binding = 2) uniform sampler2D normal_texture;
layout(binding = 3) uniform sampler2D occlusion_texture;
layout(binding = 4) uniform sampler2D emissive_texture;

// Shadow maps
layout(binding = 5) uniform sampler2DShadow shadow_maps[8];

// Environment mapping
layout(binding = 6) uniform samplerCube environment_map;
layout(binding = 7) uniform samplerCube irradiance_map;
layout(binding = 8) uniform sampler2D brdf_lut;

// Shader flags
const uint HAS_ALBEDO_TEXTURE = 1u << 0;
const uint HAS_METALLIC_ROUGHNESS_TEXTURE = 1u << 1;
const uint HAS_NORMAL_TEXTURE = 1u << 2;
const uint HAS_OCCLUSION_TEXTURE = 1u << 3;
const uint HAS_EMISSIVE_TEXTURE = 1u << 4;
const uint DOUBLE_SIDED = 1u << 5;
const uint ALPHA_TEST = 1u << 6;

// Output
layout(location = 0) out vec4 frag_color;

// Constants
const float PI = 3.14159265359;
const float EPSILON = 0.0001;

// Utility functions
vec3 get_normal() {
    vec3 normal = normalize(fs_in.world_normal);
    
    if ((shader_flags & HAS_NORMAL_TEXTURE) != 0) {
        // Sample normal map
        vec3 tangent_normal = texture(normal_texture, fs_in.uv).rgb * 2.0 - 1.0;
        tangent_normal.xy *= normal_scale;
        
        // Transform from tangent space to world space
        vec3 T = normalize(fs_in.tangent);
        vec3 B = normalize(fs_in.bitangent);
        vec3 N = normalize(fs_in.world_normal);
        mat3 TBN = mat3(T, B, N);
        
        normal = normalize(TBN * tangent_normal);
    }
    
    // Handle double-sided materials
    if ((shader_flags & DOUBLE_SIDED) != 0 && !gl_FrontFacing) {
        normal = -normal;
    }
    
    return normal;
}

vec4 get_base_color() {
    vec4 color = base_color;
    
    if ((shader_flags & HAS_ALBEDO_TEXTURE) != 0) {
        vec4 tex_color = texture(albedo_texture, fs_in.uv);
        color *= tex_color;
    }
    
    return color;
}

vec3 get_metallic_roughness() {
    vec3 mr = vec3(1.0, roughness, metallic);
    
    if ((shader_flags & HAS_METALLIC_ROUGHNESS_TEXTURE) != 0) {
        vec3 tex_mr = texture(metallic_roughness_texture, fs_in.uv).rgb;
        mr.y *= tex_mr.g; // Roughness from green channel
        mr.z *= tex_mr.b; // Metallic from blue channel
    }
    
    return mr;
}

float get_occlusion() {
    float occlusion = 1.0;
    
    if ((shader_flags & HAS_OCCLUSION_TEXTURE) != 0) {
        occlusion = texture(occlusion_texture, fs_in.uv).r;
    }
    
    return occlusion;
}

vec3 get_emissive() {
    vec3 emission = emissive * emissive_strength;
    
    if ((shader_flags & HAS_EMISSIVE_TEXTURE) != 0) {
        vec3 tex_emissive = texture(emissive_texture, fs_in.uv).rgb;
        emission *= tex_emissive;
    }
    
    return emission;
}

// PBR functions
float distribution_GGX(vec3 N, vec3 H, float roughness) {
    float a = roughness * roughness;
    float a2 = a * a;
    float NdotH = max(dot(N, H), 0.0);
    float NdotH2 = NdotH * NdotH;
    
    float num = a2;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;
    
    return num / denom;
}

float geometry_schlick_GGX(float NdotV, float roughness) {
    float r = (roughness + 1.0);
    float k = (r * r) / 8.0;
    
    float num = NdotV;
    float denom = NdotV * (1.0 - k) + k;
    
    return num / denom;
}

float geometry_smith(vec3 N, vec3 V, vec3 L, float roughness) {
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    float ggx2 = geometry_schlick_GGX(NdotV, roughness);
    float ggx1 = geometry_schlick_GGX(NdotL, roughness);
    
    return ggx1 * ggx2;
}

vec3 fresnel_schlick(float cosTheta, vec3 F0) {
    return F0 + (1.0 - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}

vec3 fresnel_schlick_roughness(float cosTheta, vec3 F0, float roughness) {
    return F0 + (max(vec3(1.0 - roughness), F0) - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}

float calculate_shadow(int shadow_index) {
    if (shadow_index >= int(shadow_map_count) || shadow_index < 0) {
        return 1.0;
    }
    
    vec4 shadow_coord = fs_in.shadow_coords[shadow_index];
    shadow_coord = shadow_coord * 0.5 + 0.5;
    
    if (shadow_coord.x < 0.0 || shadow_coord.x > 1.0 ||
        shadow_coord.y < 0.0 || shadow_coord.y > 1.0 ||
        shadow_coord.z < 0.0 || shadow_coord.z > 1.0) {
        return 1.0;
    }
    
    // PCF (Percentage Closer Filtering)
    float shadow = 0.0;
    float bias = 0.005;
    vec2 texel_size = 1.0 / textureSize(shadow_maps[shadow_index], 0);
    
    for (int x = -1; x <= 1; ++x) {
        for (int y = -1; y <= 1; ++y) {
            vec2 offset = vec2(x, y) * texel_size;
            float depth = texture(shadow_maps[shadow_index], 
                                shadow_coord.xyz + vec3(offset, -bias)).r;
            shadow += depth;
        }
    }
    shadow /= 9.0;
    
    return shadow;
}

vec3 calculate_lighting(vec3 N, vec3 V, vec3 albedo, float metallic, float roughness, vec3 F0) {
    vec3 Lo = vec3(0.0);
    
    // Calculate lighting contribution for each light
    for (uint i = 0; i < light_count && i < lights.length(); i++) {
        Light light = lights[i];
        
        vec3 L;
        float attenuation = 1.0;
        
        if (light.type == 0) {
            // Directional light
            L = normalize(-light.direction);
        } else if (light.type == 1) {
            // Point light
            L = normalize(light.position - fs_in.world_position);
            float distance = length(light.position - fs_in.world_position);
            attenuation = 1.0 / (distance * distance);
            attenuation = min(attenuation, 1.0 / (light.range * light.range));
        } else if (light.type == 2) {
            // Spot light
            L = normalize(light.position - fs_in.world_position);
            float distance = length(light.position - fs_in.world_position);
            attenuation = 1.0 / (distance * distance);
            
            float theta = dot(L, normalize(-light.direction));
            float epsilon = light.inner_cone_angle - light.outer_cone_angle;
            float spot_intensity = clamp((theta - light.outer_cone_angle) / epsilon, 0.0, 1.0);
            attenuation *= spot_intensity;
        }
        
        vec3 H = normalize(V + L);
        vec3 radiance = light.color * light.intensity * attenuation;
        
        // Calculate shadow
        float shadow = 1.0;
        if (light.type == 0 && i < shadow_map_count) {
            shadow = calculate_shadow(int(i));
        }
        
        // Cook-Torrance BRDF
        float NDF = distribution_GGX(N, H, roughness);
        float G = geometry_smith(N, V, L, roughness);
        vec3 F = fresnel_schlick(max(dot(H, V), 0.0), F0);
        
        vec3 kS = F;
        vec3 kD = vec3(1.0) - kS;
        kD *= 1.0 - metallic;
        
        vec3 numerator = NDF * G * F;
        float denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + EPSILON;
        vec3 specular = numerator / denominator;
        
        float NdotL = max(dot(N, L), 0.0);
        Lo += (kD * albedo / PI + specular) * radiance * NdotL * shadow;
    }
    
    return Lo;
}

vec3 calculate_ibl(vec3 N, vec3 V, vec3 albedo, float metallic, float roughness, vec3 F0) {
    // Image-based lighting using environment maps
    vec3 F = fresnel_schlick_roughness(max(dot(N, V), 0.0), F0, roughness);
    
    vec3 kS = F;
    vec3 kD = 1.0 - kS;
    kD *= 1.0 - metallic;
    
    // Diffuse IBL
    vec3 irradiance = texture(irradiance_map, N).rgb;
    vec3 diffuse = irradiance * albedo;
    
    // Specular IBL
    vec3 R = reflect(-V, N);
    const float MAX_REFLECTION_LOD = 4.0;
    vec3 prefiltered_color = textureLod(environment_map, R, roughness * MAX_REFLECTION_LOD).rgb;
    vec2 brdf = texture(brdf_lut, vec2(max(dot(N, V), 0.0), roughness)).rg;
    vec3 specular = prefiltered_color * (F * brdf.x + brdf.y);
    
    return kD * diffuse + specular;
}

void main() {
    // Get material properties
    vec4 base_color_alpha = get_base_color();
    vec3 albedo = base_color_alpha.rgb;
    float alpha = base_color_alpha.a;
    
    // Alpha test
    if ((shader_flags & ALPHA_TEST) != 0 && alpha < alpha_cutoff) {
        discard;
    }
    
    vec3 mr = get_metallic_roughness();
    float material_metallic = mr.z;
    float material_roughness = max(mr.y, 0.04); // Minimum roughness to avoid artifacts
    
    float occlusion = get_occlusion();
    vec3 emission = get_emissive();
    
    // Calculate normals and vectors
    vec3 N = get_normal();
    vec3 V = normalize(fs_in.view_direction);
    
    // Calculate F0 (surface reflection at zero incidence)
    vec3 F0 = vec3(0.04);
    F0 = mix(F0, albedo, material_metallic);
    
    // Calculate lighting
    vec3 direct_lighting = calculate_lighting(N, V, albedo, material_metallic, material_roughness, F0);
    vec3 indirect_lighting = calculate_ibl(N, V, albedo, material_metallic, material_roughness, F0);
    
    // Ambient lighting
    vec3 ambient = ambient_color * ambient_intensity * albedo;
    
    // Combine lighting
    vec3 color = direct_lighting + indirect_lighting + ambient + emission;
    
    // Apply occlusion to ambient and indirect lighting
    color = mix(color, direct_lighting + emission, 1.0 - occlusion);
    
    // Tone mapping and gamma correction (simplified)
    color = color / (color + vec3(1.0));
    color = pow(color, vec3(1.0/2.2));
    
    frag_color = vec4(color, alpha);
}