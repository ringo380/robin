use crate::engine::error::RobinResult;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PBRMaterialConfig {
    pub enable_image_based_lighting: bool,
    pub enable_normal_mapping: bool,
    pub enable_parallax_mapping: bool,
    pub enable_subsurface_scattering: bool,
    pub texture_quality: TextureQuality,
    pub anisotropic_filtering: u8,
}

impl Default for PBRMaterialConfig {
    fn default() -> Self {
        Self {
            enable_image_based_lighting: true,
            enable_normal_mapping: true,
            enable_parallax_mapping: false,
            enable_subsurface_scattering: false,
            texture_quality: TextureQuality::High,
            anisotropic_filtering: 16,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextureQuality {
    Ultra,
    High,
    Medium,
    Low,
}

impl TextureQuality {
    pub fn get_max_texture_size(&self) -> u32 {
        match self {
            TextureQuality::Ultra => 4096,
            TextureQuality::High => 2048,
            TextureQuality::Medium => 1024,
            TextureQuality::Low => 512,
        }
    }

    pub fn get_mip_levels(&self) -> u32 {
        match self {
            TextureQuality::Ultra => 12,
            TextureQuality::High => 11,
            TextureQuality::Medium => 10,
            TextureQuality::Low => 9,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PBRMaterial {
    pub name: String,
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub normal_strength: f32,
    pub emission_color: [f32; 3],
    pub emission_strength: f32,
    pub alpha_mode: AlphaMode,
    pub alpha_cutoff: f32,
    pub double_sided: bool,
    
    // Texture maps
    pub albedo_texture: Option<String>,
    pub normal_texture: Option<String>,
    pub metallic_roughness_texture: Option<String>,
    pub occlusion_texture: Option<String>,
    pub emission_texture: Option<String>,
    pub height_texture: Option<String>,
    
    // Advanced properties
    pub clearcoat: f32,
    pub clearcoat_roughness: f32,
    pub anisotropy: f32,
    pub transmission: f32,
    pub ior: f32, // Index of refraction
    pub thickness: f32,
    pub subsurface_color: [f32; 3],
    pub subsurface_radius: [f32; 3],
    
    // Animation properties
    pub uv_offset: [f32; 2],
    pub uv_scale: [f32; 2],
    pub uv_rotation: f32,
    pub animation_speed: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlphaMode {
    Opaque,
    Mask,
    Blend,
}

impl Default for PBRMaterial {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            base_color: [1.0, 1.0, 1.0, 1.0],
            metallic: 0.0,
            roughness: 0.5,
            normal_strength: 1.0,
            emission_color: [0.0, 0.0, 0.0],
            emission_strength: 0.0,
            alpha_mode: AlphaMode::Opaque,
            alpha_cutoff: 0.5,
            double_sided: false,
            
            albedo_texture: None,
            normal_texture: None,
            metallic_roughness_texture: None,
            occlusion_texture: None,
            emission_texture: None,
            height_texture: None,
            
            clearcoat: 0.0,
            clearcoat_roughness: 0.0,
            anisotropy: 0.0,
            transmission: 0.0,
            ior: 1.5,
            thickness: 0.0,
            subsurface_color: [1.0, 1.0, 1.0],
            subsurface_radius: [1.0, 1.0, 1.0],
            
            uv_offset: [0.0, 0.0],
            uv_scale: [1.0, 1.0],
            uv_rotation: 0.0,
            animation_speed: 0.0,
        }
    }
}

#[derive(Debug)]
pub struct PBRMaterialSystem {
    config: PBRMaterialConfig,
    materials: HashMap<String, PBRMaterial>,
    material_templates: HashMap<String, PBRMaterial>,
    shader_cache: HashMap<String, CompiledShader>,
    texture_cache: HashMap<String, TextureData>,
    precomputed_environments: Vec<EnvironmentMap>,
}

#[derive(Debug, Clone)]
pub struct CompiledShader {
    pub vertex_code: String,
    pub fragment_code: String,
    pub feature_flags: ShaderFeatures,
    pub uniform_layout: Vec<UniformBinding>,
}

#[derive(Debug, Clone)]
pub struct ShaderFeatures {
    pub has_albedo_texture: bool,
    pub has_normal_texture: bool,
    pub has_metallic_roughness_texture: bool,
    pub has_occlusion_texture: bool,
    pub has_emission_texture: bool,
    pub has_height_texture: bool,
    pub has_clearcoat: bool,
    pub has_transmission: bool,
    pub has_subsurface: bool,
    pub has_anisotropy: bool,
    pub has_ibl: bool,
    pub vertex_colors: bool,
    pub skinned_mesh: bool,
}

#[derive(Debug, Clone)]
pub struct UniformBinding {
    pub name: String,
    pub binding_type: UniformType,
    pub location: u32,
}

#[derive(Debug, Clone)]
pub enum UniformType {
    Float,
    Vec2,
    Vec3,
    Vec4,
    Mat3,
    Mat4,
    Sampler2D,
    SamplerCube,
}

#[derive(Debug, Clone)]
pub struct TextureData {
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub mip_levels: u32,
    pub data: Vec<u8>,
    pub srgb: bool,
}

#[derive(Debug, Clone)]
pub enum TextureFormat {
    RGBA8,
    RGBA16F,
    RGB8,
    RGB16F,
    RG8,
    R8,
    BC1, // DXT1
    BC3, // DXT5
    BC5, // Normal maps
    BC6H, // HDR
    BC7, // High quality
}

#[derive(Debug, Clone)]
pub struct EnvironmentMap {
    pub name: String,
    pub irradiance_map: String, // Texture ID for diffuse IBL
    pub prefiltered_map: String, // Texture ID for specular IBL
    pub brdf_lut: String, // Texture ID for BRDF lookup table
    pub intensity: f32,
    pub rotation: f32,
}

impl PBRMaterialSystem {
    pub fn new(config: PBRMaterialConfig) -> RobinResult<Self> {
        let mut system = Self {
            config,
            materials: HashMap::new(),
            material_templates: HashMap::new(),
            shader_cache: HashMap::new(),
            texture_cache: HashMap::new(),
            precomputed_environments: Vec::new(),
        };

        system.initialize_templates()?;
        system.precompute_brdf_lut()?;
        
        Ok(system)
    }

    pub fn create_material(&mut self, name: String, mut material: PBRMaterial) -> RobinResult<()> {
        material.name = name.clone();
        self.materials.insert(name, material);
        Ok(())
    }

    pub fn create_material_from_template(&mut self, name: String, template_name: &str) -> RobinResult<()> {
        if let Some(template) = self.material_templates.get(template_name) {
            let mut material = template.clone();
            material.name = name.clone();
            self.materials.insert(name, material);
            Ok(())
        } else {
            Err(crate::engine::error::RobinError::new(
                format!("Material template '{}' not found", template_name)
            ))
        }
    }

    pub fn update_material_property(&mut self, material_name: &str, property: MaterialProperty) -> RobinResult<()> {
        if let Some(material) = self.materials.get_mut(material_name) {
            match property {
                MaterialProperty::BaseColor(color) => material.base_color = color,
                MaterialProperty::Metallic(value) => material.metallic = value.clamp(0.0, 1.0),
                MaterialProperty::Roughness(value) => material.roughness = value.clamp(0.0, 1.0),
                MaterialProperty::NormalStrength(value) => material.normal_strength = value.clamp(0.0, 3.0),
                MaterialProperty::EmissionColor(color) => material.emission_color = color,
                MaterialProperty::EmissionStrength(value) => material.emission_strength = value.max(0.0),
                MaterialProperty::Clearcoat(value) => material.clearcoat = value.clamp(0.0, 1.0),
                MaterialProperty::Transmission(value) => material.transmission = value.clamp(0.0, 1.0),
                MaterialProperty::IOR(value) => material.ior = value.clamp(1.0, 3.0),
                MaterialProperty::UVOffset(offset) => material.uv_offset = offset,
                MaterialProperty::UVScale(scale) => material.uv_scale = scale,
            }
            
            // Invalidate shader cache for this material
            self.invalidate_shader_cache(material_name);
            
            Ok(())
        } else {
            Err(crate::engine::error::RobinError::new(
                format!("Material '{}' not found", material_name)
            ))
        }
    }

    pub fn get_material(&self, name: &str) -> Option<&PBRMaterial> {
        self.materials.get(name)
    }

    pub fn get_or_compile_shader(&mut self, material: &PBRMaterial) -> RobinResult<&CompiledShader> {
        let shader_key = self.generate_shader_key(material);
        
        if !self.shader_cache.contains_key(&shader_key) {
            let shader = self.compile_shader(material)?;
            self.shader_cache.insert(shader_key.clone(), shader);
        }
        
        Ok(self.shader_cache.get(&shader_key).unwrap())
    }

    pub fn load_texture(&mut self, name: String, path: &str, srgb: bool) -> RobinResult<()> {
        // Simulate loading texture data
        let texture_data = TextureData {
            width: self.config.texture_quality.get_max_texture_size(),
            height: self.config.texture_quality.get_max_texture_size(),
            format: if srgb { TextureFormat::RGBA8 } else { TextureFormat::RGBA8 },
            mip_levels: self.config.texture_quality.get_mip_levels(),
            data: vec![128; (self.config.texture_quality.get_max_texture_size() * 
                            self.config.texture_quality.get_max_texture_size() * 4) as usize],
            srgb,
        };

        self.texture_cache.insert(name, texture_data);
        Ok(())
    }

    pub fn create_environment_map(&mut self, name: String, hdr_path: &str, intensity: f32) -> RobinResult<()> {
        // Generate environment map textures
        let irradiance_id = format!("{}_irradiance", name);
        let prefiltered_id = format!("{}_prefiltered", name);
        
        // Create irradiance map (32x32 for diffuse lighting)
        self.load_texture(irradiance_id.clone(), &format!("{}_irr.hdr", hdr_path), false)?;
        
        // Create prefiltered environment map (256x256 with mipmaps for specular)
        self.load_texture(prefiltered_id.clone(), &format!("{}_spec.hdr", hdr_path), false)?;
        
        let env_map = EnvironmentMap {
            name: name.clone(),
            irradiance_map: irradiance_id,
            prefiltered_map: prefiltered_id,
            brdf_lut: "brdf_lut".to_string(),
            intensity,
            rotation: 0.0,
        };
        
        self.precomputed_environments.push(env_map);
        Ok(())
    }

    pub fn update_material_animation(&mut self, material_name: &str, delta_time: f32) -> RobinResult<()> {
        if let Some(material) = self.materials.get_mut(material_name) {
            if material.animation_speed != 0.0 {
                material.uv_offset[0] += material.animation_speed * delta_time;
                material.uv_offset[1] += material.animation_speed * delta_time * 0.5;
                
                // Wrap UV coordinates
                material.uv_offset[0] = material.uv_offset[0] % 1.0;
                material.uv_offset[1] = material.uv_offset[1] % 1.0;
            }
            Ok(())
        } else {
            Err(crate::engine::error::RobinError::new(
                format!("Material '{}' not found", material_name)
            ))
        }
    }

    fn initialize_templates(&mut self) -> RobinResult<()> {
        // Metal template
        let mut metal = PBRMaterial::default();
        metal.name = "Metal".to_string();
        metal.base_color = [0.7, 0.7, 0.8, 1.0];
        metal.metallic = 1.0;
        metal.roughness = 0.1;
        self.material_templates.insert("Metal".to_string(), metal);

        // Dielectric template (plastic, ceramic, etc.)
        let mut dielectric = PBRMaterial::default();
        dielectric.name = "Plastic".to_string();
        dielectric.base_color = [0.8, 0.2, 0.2, 1.0];
        dielectric.metallic = 0.0;
        dielectric.roughness = 0.3;
        self.material_templates.insert("Plastic".to_string(), dielectric);

        // Glass template
        let mut glass = PBRMaterial::default();
        glass.name = "Glass".to_string();
        glass.base_color = [1.0, 1.0, 1.0, 0.1];
        glass.metallic = 0.0;
        glass.roughness = 0.0;
        glass.transmission = 1.0;
        glass.ior = 1.5;
        glass.alpha_mode = AlphaMode::Blend;
        self.material_templates.insert("Glass".to_string(), glass);

        // Wood template
        let mut wood = PBRMaterial::default();
        wood.name = "Wood".to_string();
        wood.base_color = [0.6, 0.4, 0.2, 1.0];
        wood.metallic = 0.0;
        wood.roughness = 0.8;
        wood.albedo_texture = Some("wood_albedo".to_string());
        wood.normal_texture = Some("wood_normal".to_string());
        self.material_templates.insert("Wood".to_string(), wood);

        // Concrete template
        let mut concrete = PBRMaterial::default();
        concrete.name = "Concrete".to_string();
        concrete.base_color = [0.6, 0.6, 0.6, 1.0];
        concrete.metallic = 0.0;
        concrete.roughness = 0.9;
        concrete.albedo_texture = Some("concrete_albedo".to_string());
        concrete.normal_texture = Some("concrete_normal".to_string());
        concrete.occlusion_texture = Some("concrete_ao".to_string());
        self.material_templates.insert("Concrete".to_string(), concrete);

        // Emissive template
        let mut emissive = PBRMaterial::default();
        emissive.name = "Neon".to_string();
        emissive.base_color = [0.0, 1.0, 0.5, 1.0];
        emissive.metallic = 0.0;
        emissive.roughness = 0.1;
        emissive.emission_color = [0.0, 1.0, 0.5];
        emissive.emission_strength = 2.0;
        self.material_templates.insert("Neon".to_string(), emissive);

        Ok(())
    }

    fn compile_shader(&self, material: &PBRMaterial) -> RobinResult<CompiledShader> {
        let features = self.analyze_material_features(material);
        
        let vertex_code = self.generate_vertex_shader(&features);
        let fragment_code = self.generate_fragment_shader(&features);
        let uniform_layout = self.generate_uniform_layout(&features);

        Ok(CompiledShader {
            vertex_code,
            fragment_code,
            feature_flags: features,
            uniform_layout,
        })
    }

    fn analyze_material_features(&self, material: &PBRMaterial) -> ShaderFeatures {
        ShaderFeatures {
            has_albedo_texture: material.albedo_texture.is_some(),
            has_normal_texture: material.normal_texture.is_some(),
            has_metallic_roughness_texture: material.metallic_roughness_texture.is_some(),
            has_occlusion_texture: material.occlusion_texture.is_some(),
            has_emission_texture: material.emission_texture.is_some(),
            has_height_texture: material.height_texture.is_some(),
            has_clearcoat: material.clearcoat > 0.0,
            has_transmission: material.transmission > 0.0,
            has_subsurface: material.subsurface_radius != [1.0, 1.0, 1.0],
            has_anisotropy: material.anisotropy != 0.0,
            has_ibl: self.config.enable_image_based_lighting,
            vertex_colors: false, // Could be material property
            skinned_mesh: false, // Could be material property
        }
    }

    fn generate_vertex_shader(&self, features: &ShaderFeatures) -> String {
        let mut shader = String::from(r#"
#version 450 core

layout(location = 0) in vec3 a_position;
layout(location = 1) in vec3 a_normal;
layout(location = 2) in vec2 a_uv;
layout(location = 3) in vec3 a_tangent;

uniform mat4 u_model_matrix;
uniform mat4 u_view_matrix;
uniform mat4 u_projection_matrix;
uniform mat3 u_normal_matrix;

out vec3 v_world_position;
out vec3 v_normal;
out vec2 v_uv;
out vec3 v_tangent;
out vec3 v_bitangent;

void main() {
    vec4 world_pos = u_model_matrix * vec4(a_position, 1.0);
    v_world_position = world_pos.xyz;
    
    v_normal = normalize(u_normal_matrix * a_normal);
    v_uv = a_uv;
    
    v_tangent = normalize(u_normal_matrix * a_tangent);
    v_bitangent = cross(v_normal, v_tangent);
    
    gl_Position = u_projection_matrix * u_view_matrix * world_pos;
}
"#);

        if features.skinned_mesh {
            shader.insert_str(shader.find("layout(location = 3)").unwrap(), 
                "layout(location = 4) in ivec4 a_bone_ids;\nlayout(location = 5) in vec4 a_bone_weights;\n");
        }

        shader
    }

    fn generate_fragment_shader(&self, features: &ShaderFeatures) -> String {
        let mut shader = String::from(r#"
#version 450 core

in vec3 v_world_position;
in vec3 v_normal;
in vec2 v_uv;
in vec3 v_tangent;
in vec3 v_bitangent;

uniform vec3 u_camera_position;
uniform vec4 u_base_color;
uniform float u_metallic;
uniform float u_roughness;
uniform vec3 u_emission_color;
uniform float u_emission_strength;

out vec4 FragColor;

const float PI = 3.14159265359;

// PBR functions
vec3 getNormalFromMap(vec2 uv, vec3 normal, vec3 tangent, vec3 bitangent);
vec3 fresnel_schlick(float cos_theta, vec3 F0);
vec3 fresnel_schlick_roughness(float cos_theta, vec3 F0, float roughness);
float distribution_ggx(vec3 N, vec3 H, float roughness);
float geometry_schlick_ggx(float NdotV, float roughness);
float geometry_smith(vec3 N, vec3 V, vec3 L, float roughness);

void main() {
    vec3 albedo = u_base_color.rgb;
    float metallic = u_metallic;
    float roughness = u_roughness;
    float ao = 1.0;
    
    vec3 N = normalize(v_normal);
    vec3 V = normalize(u_camera_position - v_world_position);
    
"#);

        // Add texture sampling
        if features.has_albedo_texture {
            shader.insert_str(shader.find("uniform float u_roughness;").unwrap() + "uniform float u_roughness;".len(), 
                "\nuniform sampler2D u_albedo_texture;");
            shader.insert_str(shader.find("vec3 albedo = u_base_color.rgb;").unwrap(), 
                "vec4 albedo_sample = texture(u_albedo_texture, v_uv);\n    ");
            shader = shader.replace("vec3 albedo = u_base_color.rgb;", 
                "vec3 albedo = u_base_color.rgb * albedo_sample.rgb;");
        }

        if features.has_normal_texture {
            shader.insert_str(shader.find("uniform sampler2D u_albedo_texture;").unwrap_or(shader.find("uniform float u_emission_strength;").unwrap() + "uniform float u_emission_strength;".len()), 
                "\nuniform sampler2D u_normal_texture;");
            shader = shader.replace("vec3 N = normalize(v_normal);", 
                "vec3 N = getNormalFromMap(v_uv, normalize(v_normal), normalize(v_tangent), normalize(v_bitangent));");
        }

        if features.has_metallic_roughness_texture {
            shader.insert_str(shader.rfind("uniform sampler2D").unwrap() + shader[shader.rfind("uniform sampler2D").unwrap()..].find(';').unwrap() + 1, 
                "\nuniform sampler2D u_metallic_roughness_texture;");
            shader.insert_str(shader.find("float roughness = u_roughness;").unwrap(), 
                "vec3 mr_sample = texture(u_metallic_roughness_texture, v_uv).rgb;\n    ");
            shader = shader.replace("float metallic = u_metallic;", 
                "float metallic = u_metallic * mr_sample.b;");
            shader = shader.replace("float roughness = u_roughness;", 
                "float roughness = u_roughness * mr_sample.g;");
        }

        // Add PBR lighting calculation
        shader.push_str(r#"
    
    // Calculate reflectance at normal incidence
    vec3 F0 = vec3(0.04);
    F0 = mix(F0, albedo, metallic);
    
    // Reflectance equation
    vec3 Lo = vec3(0.0);
    
    // Add directional light (sun)
    vec3 L = normalize(vec3(-0.3, -0.8, -0.5));
    vec3 H = normalize(V + L);
    vec3 radiance = vec3(1.0, 0.95, 0.8) * 3.0;
    
    float NDF = distribution_ggx(N, H, roughness);
    float G = geometry_smith(N, V, L, roughness);
    vec3 F = fresnel_schlick(clamp(dot(H, V), 0.0, 1.0), F0);
    
    vec3 kS = F;
    vec3 kD = vec3(1.0) - kS;
    kD *= 1.0 - metallic;
    
    vec3 numerator = NDF * G * F;
    float denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001;
    vec3 specular = numerator / denominator;
    
    float NdotL = max(dot(N, L), 0.0);
    Lo += (kD * albedo / PI + specular) * radiance * NdotL;
    
    // Ambient lighting
    vec3 ambient = vec3(0.03) * albedo * ao;
    vec3 color = ambient + Lo;
    
    // Add emission
    color += u_emission_color * u_emission_strength;
    
    // HDR tonemapping
    color = color / (color + vec3(1.0));
    
    // Gamma correction
    color = pow(color, vec3(1.0/2.2));
    
    FragColor = vec4(color, u_base_color.a);
}

// PBR function implementations
vec3 getNormalFromMap(vec2 uv, vec3 normal, vec3 tangent, vec3 bitangent) {
    vec3 tangent_normal = texture(u_normal_texture, uv).rgb * 2.0 - 1.0;
    mat3 TBN = mat3(tangent, bitangent, normal);
    return normalize(TBN * tangent_normal);
}

vec3 fresnel_schlick(float cos_theta, vec3 F0) {
    return F0 + (1.0 - F0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

float distribution_ggx(vec3 N, vec3 H, float roughness) {
    float a = roughness * roughness;
    float a2 = a * a;
    float NdotH = max(dot(N, H), 0.0);
    float NdotH2 = NdotH * NdotH;
    
    float num = a2;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;
    
    return num / denom;
}

float geometry_schlick_ggx(float NdotV, float roughness) {
    float r = (roughness + 1.0);
    float k = (r * r) / 8.0;
    
    float num = NdotV;
    float denom = NdotV * (1.0 - k) + k;
    
    return num / denom;
}

float geometry_smith(vec3 N, vec3 V, vec3 L, float roughness) {
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    float ggx2 = geometry_schlick_ggx(NdotV, roughness);
    float ggx1 = geometry_schlick_ggx(NdotL, roughness);
    
    return ggx1 * ggx2;
}
"#);

        shader
    }

    fn generate_uniform_layout(&self, features: &ShaderFeatures) -> Vec<UniformBinding> {
        let mut uniforms = vec![
            UniformBinding { name: "u_model_matrix".to_string(), binding_type: UniformType::Mat4, location: 0 },
            UniformBinding { name: "u_view_matrix".to_string(), binding_type: UniformType::Mat4, location: 1 },
            UniformBinding { name: "u_projection_matrix".to_string(), binding_type: UniformType::Mat4, location: 2 },
            UniformBinding { name: "u_normal_matrix".to_string(), binding_type: UniformType::Mat3, location: 3 },
            UniformBinding { name: "u_camera_position".to_string(), binding_type: UniformType::Vec3, location: 4 },
            UniformBinding { name: "u_base_color".to_string(), binding_type: UniformType::Vec4, location: 5 },
            UniformBinding { name: "u_metallic".to_string(), binding_type: UniformType::Float, location: 6 },
            UniformBinding { name: "u_roughness".to_string(), binding_type: UniformType::Float, location: 7 },
            UniformBinding { name: "u_emission_color".to_string(), binding_type: UniformType::Vec3, location: 8 },
            UniformBinding { name: "u_emission_strength".to_string(), binding_type: UniformType::Float, location: 9 },
        ];

        let mut texture_unit = 0;
        
        if features.has_albedo_texture {
            uniforms.push(UniformBinding { 
                name: "u_albedo_texture".to_string(), 
                binding_type: UniformType::Sampler2D, 
                location: texture_unit 
            });
            texture_unit += 1;
        }

        if features.has_normal_texture {
            uniforms.push(UniformBinding { 
                name: "u_normal_texture".to_string(), 
                binding_type: UniformType::Sampler2D, 
                location: texture_unit 
            });
            texture_unit += 1;
        }

        if features.has_metallic_roughness_texture {
            uniforms.push(UniformBinding { 
                name: "u_metallic_roughness_texture".to_string(), 
                binding_type: UniformType::Sampler2D, 
                location: texture_unit 
            });
            texture_unit += 1;
        }

        uniforms
    }

    fn generate_shader_key(&self, material: &PBRMaterial) -> String {
        let features = self.analyze_material_features(material);
        format!(
            "pbr_{}_{}_{}_{}_{}_{}_{}_{}_{}_{}_{}", 
            features.has_albedo_texture as u8,
            features.has_normal_texture as u8,
            features.has_metallic_roughness_texture as u8,
            features.has_occlusion_texture as u8,
            features.has_emission_texture as u8,
            features.has_clearcoat as u8,
            features.has_transmission as u8,
            features.has_subsurface as u8,
            features.has_anisotropy as u8,
            features.has_ibl as u8,
            features.vertex_colors as u8,
        )
    }

    fn precompute_brdf_lut(&mut self) -> RobinResult<()> {
        // Generate BRDF integration lookup table (512x512)
        let brdf_lut = TextureData {
            width: 512,
            height: 512,
            format: TextureFormat::RG8,
            mip_levels: 1,
            data: vec![128; 512 * 512 * 2], // Placeholder - would be computed
            srgb: false,
        };

        self.texture_cache.insert("brdf_lut".to_string(), brdf_lut);
        Ok(())
    }

    fn invalidate_shader_cache(&mut self, material_name: &str) {
        // Remove cached shaders that use this material
        let keys_to_remove: Vec<String> = self.shader_cache.keys()
            .filter(|key| key.contains(material_name))
            .cloned()
            .collect();
        
        for key in keys_to_remove {
            self.shader_cache.remove(&key);
        }
    }

    pub fn get_material_names(&self) -> Vec<String> {
        self.materials.keys().cloned().collect()
    }

    pub fn get_template_names(&self) -> Vec<String> {
        self.material_templates.keys().cloned().collect()
    }

    pub fn get_texture_names(&self) -> Vec<String> {
        self.texture_cache.keys().cloned().collect()
    }

    pub fn get_shader_cache_size(&self) -> usize {
        self.shader_cache.len()
    }

    pub fn clear_unused_shaders(&mut self) {
        // In a real implementation, this would track shader usage and remove unused ones
        if self.shader_cache.len() > 100 {
            self.shader_cache.clear();
        }
    }

    pub fn shutdown(&mut self) -> RobinResult<()> {
        println!("PBR Material System shutdown:");
        println!("  Materials: {}", self.materials.len());
        println!("  Cached shaders: {}", self.shader_cache.len());
        println!("  Cached textures: {}", self.texture_cache.len());

        self.materials.clear();
        self.shader_cache.clear();
        self.texture_cache.clear();
        self.precomputed_environments.clear();

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum MaterialProperty {
    BaseColor([f32; 4]),
    Metallic(f32),
    Roughness(f32),
    NormalStrength(f32),
    EmissionColor([f32; 3]),
    EmissionStrength(f32),
    Clearcoat(f32),
    Transmission(f32),
    IOR(f32),
    UVOffset([f32; 2]),
    UVScale([f32; 2]),
}