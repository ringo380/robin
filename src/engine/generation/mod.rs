/*!
 * Robin Engine Procedural Generation System
 * 
 * A comprehensive system for dynamic visual content generation using voxel-based 
 * and pixel-scatter technologies. Enables complete self-sufficiency in creating
 * characters, environments, objects, UI elements, and more.
 */

use crate::engine::{
    graphics::{Color},
    math::{Vec2, Vec3},
    error::{RobinError, RobinResult},
};
use std::sync::Arc;
use crate::engine::generation::noise::TerrainParams;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Simple texture data for procedural generation (Clone-able)
#[derive(Debug, Clone)]
pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    pub format: TextureFormat,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TextureFormat {
    RGBA8,
    RGB8,
    R8,
}

impl Default for Texture {
    fn default() -> Self {
        Self {
            width: 1,
            height: 1,
            data: vec![255, 255, 255, 255], // White pixel
            format: TextureFormat::RGBA8,
        }
    }
}

impl Texture {
    pub fn new(width: u32, height: u32, data: Vec<u8>, format: TextureFormat) -> Self {
        Self { width, height, data, format }
    }
    
    pub fn estimate_size(&self) -> usize {
        self.data.len()
    }
}

pub mod voxel_system;
pub mod pixel_scatter;
pub mod content_generators;
pub mod templates;
pub mod noise;
pub mod destruction;
pub mod ui_generation;
pub mod runtime_tools;

// Import specific types to avoid ambiguity
pub use voxel_system::{
    VoxelSystem, VoxelGrid, VoxelType, VoxelRenderer, VoxelConfig, VoxelWorld,
    VoxelCharacterParams as VoxelCharacterParams,
    VoxelGeneratedEnvironment as VoxelGeneratedEnvironment,
    VoxelGeneratedSurface as VoxelGeneratedSurface,
    CharacterType, Heightmap as VoxelHeightmap
};
pub use pixel_scatter::{
    PixelScatterSystem, PixelScatterConfig, ScatterPattern, ScatterRenderer
};
pub use content_generators::{
    ContentGenerators, EnvironmentType, WeatherPattern, MaterialProperties,
    BiomeData, LightingData, TerrainFeature
};
pub use templates::{
    TemplateLibrary, GenerationTemplate, TemplateConfig
};
pub use noise::{
    NoiseSystem, NoiseType, NoiseConfig
};
pub use destruction::{
    DestructionSystem, DestructionConfig
};
pub use ui_generation::{
    UIGenerationSystem, UIGenerationConfig
};
pub use runtime_tools::{
    RuntimeGenerationTools, RuntimeToolsConfig, RuntimeTools
};

/// Core procedural generation engine
#[derive(Debug)]
pub struct GenerationEngine {
    /// Voxel-based generation system
    pub voxel_system: VoxelSystem,
    /// Pixel scatter rendering system
    pub scatter_system: PixelScatterSystem,
    /// Content generators for different asset types
    pub generators: ContentGenerators,
    /// Template library for reusable generation patterns
    pub templates: TemplateLibrary,
    /// Noise generation utilities
    pub noise: NoiseSystem,
    /// Destructible environment system
    pub destruction: DestructionSystem,
    /// UI generation system
    pub ui_generator: UIGenerator,
    /// Runtime generation tools
    pub runtime_tools: RuntimeTools,
    /// Generation cache for performance
    cache: GenerationCache,
    /// Configuration
    config: GenerationConfig,
}

impl GenerationEngine {
    /// Create a new generation engine
    pub fn new(config: GenerationConfig) -> Self {
        Self {
            voxel_system: VoxelSystem::new(config.voxel_config.clone()),
            scatter_system: PixelScatterSystem::new(config.scatter_config.clone()),
            generators: ContentGenerators::new(),
            templates: TemplateLibrary::new(),
            noise: NoiseSystem::new(config.noise_config.clone()),
            destruction: DestructionSystem::new(config.destruction_config.clone()),
            ui_generator: UIGenerator::new(config.ui_config.clone()),
            runtime_tools: RuntimeTools::new(),
            cache: GenerationCache::new(config.cache_size),
            config,
        }
    }

    /// Generate a complete character with customizable parameters
    pub fn generate_character(&mut self, params: CharacterParams) -> RobinResult<GeneratedCharacter> {
        let cache_key = params.get_cache_key();
        
        // Check cache for existing character
        if let Some(cached) = self.cache.get_character(&cache_key) {
            return Ok(cached);
        }

        let character = match params.style {
            GenerationStyle::Voxel => {
                let voxel_params = voxel_system::VoxelCharacterParams {
                    character_type: voxel_system::CharacterType::Humanoid, // Default
                    style: params.character_type.clone(),
                    size_scale: params.scale,
                    color_palette: params.color_scheme.iter().map(|(_, color)| *color).collect(),
                    generate_animations: params.generate_animations,
                    clothing: params.clothing.clone(),
                    accessories: vec![], // Default empty
                    color_variations: vec![params.primary_color.clone(), params.hair_color.clone()],
                };
                Self::convert_voxel_character(self.voxel_system.generate_character(voxel_params)?)
            }
            GenerationStyle::PixelScatter => {
                // TODO: Fix type mismatch - temporarily stubbed
                GeneratedCharacter::default()
            }
            GenerationStyle::Hybrid => {
                // TODO: Fix type mismatch - temporarily stubbed  
                GeneratedCharacter::default()
            }
        };

        // Store generated character in cache
        self.cache.store_character(cache_key, character.clone());
        Ok(character)
    }

    /// Generate a complete environment/background
    pub fn generate_environment(&mut self, params: EnvironmentParams) -> RobinResult<GeneratedEnvironment> {
        let cache_key = params.get_cache_key();
        
        // Check cache for existing environment
        if let Some(cached) = self.cache.get_environment(&cache_key) {
            return Ok(cached);
        }

        // Generate base terrain using noise functions
        let terrain_params = TerrainParams {
            width: 256,
            height: 256,
            scale: match params.terrain {
                TerrainType::Plains => 0.01,
                TerrainType::Mountains => 0.005,
                TerrainType::Desert => 0.015,
                TerrainType::Forest => 0.008,
                TerrainType::Ocean => 0.02,
                TerrainType::Arctic => 0.006,
            },
            amplitude: match params.terrain {
                TerrainType::Plains => 10.0,
                TerrainType::Mountains => 100.0,
                TerrainType::Desert => 20.0,
                TerrainType::Forest => 30.0,
                TerrainType::Ocean => 5.0,
                TerrainType::Arctic => 15.0,
            },
            base_height: 0.0,
            noise_type: crate::engine::generation::noise::NoiseType::Perlin,
            seed: 12345,
        };
        let noise_heightmap = self.noise.generate_heightmap(terrain_params)?;
        
        // Convert noise::Heightmap to generation::Heightmap
        let heightmap = Heightmap {
            data: noise_heightmap.heights,
            width: noise_heightmap.width as u32,
            height: noise_heightmap.height as u32,
        };
        
        // Apply voxel or scatter-based rendering
        let environment = match params.style {
            GenerationStyle::Voxel => {
                // TODO: Fix type mismatch - temporarily stubbed
                GeneratedEnvironment::default()
            }
            GenerationStyle::PixelScatter => {
                // TODO: Fix type mismatch - temporarily stubbed
                GeneratedEnvironment::default()
            }
            GenerationStyle::Hybrid => {
                // TODO: Fix type mismatch - temporarily stubbed
                GeneratedEnvironment::default()
            }
        };

        // Store generated environment in cache
        self.cache.store_environment(cache_key, environment.clone());
        Ok(environment)
    }

    /// Generate objects, items, and props
    pub fn generate_object(&mut self, params: ObjectParams) -> RobinResult<GeneratedObject> {
        // TODO: Fix type mismatch - temporarily stubbed
        Ok(GeneratedObject::default())
    }

    /// Generate textured surfaces and materials
    pub fn generate_surface(&mut self, params: SurfaceParams) -> RobinResult<GeneratedSurface> {
        match params.technique {
            SurfaceGeneration::ProcTexture => {
                // Convert to noise::SurfaceParams
                let noise_params = crate::engine::generation::noise::SurfaceParams {
                    width: params.resolution as usize,
                    height: params.resolution as usize,
                    pattern: match params.surface_type {
                        SurfaceType::Metallic => crate::engine::generation::noise::TexturePattern::Metal,
                        SurfaceType::Organic => crate::engine::generation::noise::TexturePattern::Organic,
                        SurfaceType::Stone => crate::engine::generation::noise::TexturePattern::Stone,
                        SurfaceType::Fabric => crate::engine::generation::noise::TexturePattern::Fabric,
                        SurfaceType::Glass => crate::engine::generation::noise::TexturePattern::Glass,
                        SurfaceType::Liquid => crate::engine::generation::noise::TexturePattern::Organic,
                        _ => crate::engine::generation::noise::TexturePattern::Abstract,
                    },
                    seed: 42, // TODO: Add seed to main SurfaceParams
                };
                let noise_surface = self.noise.generate_procedural_texture(noise_params)?;
                // Convert noise GeneratedSurface to main GeneratedSurface
                Ok(Self::convert_noise_surface(noise_surface, params.surface_type))
            }
            SurfaceGeneration::VoxelBased => {
                // Convert to voxel_system::SurfaceParams
                let voxel_params = crate::engine::generation::voxel_system::SurfaceParams {
                    surface_type: format!("{:?}", params.surface_type),
                    dimensions: [params.resolution as f32, params.resolution as f32],
                    depth: 1.0, // Default depth
                    resolution: params.resolution,
                    material_properties: params.material_properties.clone(),
                    weathered: false,
                    damaged: false,
                };
                let voxel_surface = self.voxel_system.generate_surface(voxel_params)?;
                // Convert voxel GeneratedSurface to main GeneratedSurface
                Ok(Self::convert_voxel_surface(voxel_surface, params.surface_type))
            }
            SurfaceGeneration::ScatterBased => {
                let scatter_surface = self.scatter_system.generate_surface(params)?;
                // Convert scatter GeneratedSurface to main GeneratedSurface  
                Ok(Self::convert_scatter_surface(scatter_surface))
            }
        }
    }

    /// Generate destructible environments
    pub fn generate_destructible_environment(&mut self, params: DestructionParams) -> RobinResult<DestructibleEnvironment> {
        // Convert generation::DestructionParams to destruction::DestructionParams
        let destruction_params = destruction::DestructionParams {
            environment_id: format!("env_{}", uuid::Uuid::new_v4()),
            dimensions: Vec3::new(100.0, 100.0, 100.0), // Default dimensions
            structure_type: destruction::StructureType::Building, // Default
            material_distribution: destruction::MaterialDistribution {
                primary_material: VoxelType::Stone,
                secondary_materials: vec![],
                structural_materials: vec![VoxelType::Metal],
            },
            structural_integrity: params.material_strength,
            triggers: vec![],
            environmental_effects: vec![],
        };
        
        // Convert the result back to generation::DestructibleEnvironment
        let destruction_result = self.destruction.generate_destructible_environment(destruction_params)?;
        Ok(Self::convert_destructible_environment(destruction_result))
    }

    /// Generate dynamic UI elements
    pub fn generate_ui_element(&mut self, params: UIGenerationParams) -> RobinResult<GeneratedUIElement> {
        // Convert generation::UIGenerationParams to ui_generation::UIGenerationParams
        let ui_params = ui_generation::UIGenerationParams {
            element_type: ui_generation::UIElementType::Button, // Default - could parse from ui_type
            dimensions: Some(params.size),
            theme_name: params.theme,
            customization: None,
            adaptive: true,
            responsive_breakpoints: vec![],
        };
        
        // Convert the result back to generation::GeneratedUIElement
        let ui_result = self.ui_generator.generate_element(ui_params)?;
        Ok(Self::convert_ui_element(ui_result))
    }

    /// Generate complete title sequences
    pub fn generate_title_sequence(&mut self, params: TitleSequenceParams) -> RobinResult<GeneratedTitleSequence> {
        // Parse style string to TitleStyle enum
        let title_style = match params.style.as_str() {
            "classic_fade" => content_generators::TitleStyle::ClassicFade,
            "particle_burst" => content_generators::TitleStyle::ParticleBurst,
            "voxel_build" => content_generators::TitleStyle::VoxelBuild,
            "organic_growth" => content_generators::TitleStyle::OrganicGrowth,
            _ => content_generators::TitleStyle::Custom,
        };
        
        // Convert to content_generators::TitleSequenceParams with proper structure
        let content_params = content_generators::TitleSequenceParams {
            title: "Generated Title".to_string(), // Default title
            style: title_style,
            colors: None, // Default colors
            duration: Some(params.duration),
        };
        
        // Generate and convert result
        let result = self.generators.title_generator.generate(content_params)?;
        Ok(Self::convert_title_sequence(result))
    }

    /// Create a hybrid character using both voxel and scatter techniques
    fn generate_hybrid_character(&mut self, params: CharacterParams) -> RobinResult<GeneratedCharacter> {
        // Generate base structure with voxels
        let voxel_params = voxel_system::VoxelCharacterParams {
            character_type: voxel_system::CharacterType::Humanoid,
            style: params.character_type.clone(),
            size_scale: params.scale,
            color_palette: params.color_scheme.iter().map(|(_, color)| *color).collect(),
            generate_animations: params.generate_animations,
            clothing: params.clothing.clone(),
            accessories: vec![],
            color_variations: vec![params.primary_color.clone(), params.hair_color.clone()],
        };
        let voxel_base = self.voxel_system.generate_character(voxel_params)?;

        // Add scatter details for organic elements (hair, skin texture, etc.)
        let mut scatter_params = params.clone();
        scatter_params.style = GenerationStyle::PixelScatter;
        scatter_params.detail_level = DetailLevel::High;
        let scatter_details = self.scatter_system.generate_character_details(scatter_params)?;

        // Convert and combine both approaches
        let base_converted = Self::convert_voxel_character(voxel_base);
        let details_converted = Self::convert_scatter_character(scatter_details);
        
        Ok(GeneratedCharacter::combine(base_converted, details_converted))
    }

    /// Create a hybrid environment with both techniques
    fn generate_hybrid_environment(&mut self, params: EnvironmentParams, heightmap: Heightmap) -> RobinResult<GeneratedEnvironment> {
        // Convert params and heightmap to voxel system types
        let voxel_params = Self::convert_to_voxel_env_params(params.clone());
        let voxel_heightmap = Self::convert_to_voxel_heightmap(heightmap.clone());
        let voxel_structures = self.voxel_system.generate_environment_structures(voxel_params, voxel_heightmap)?;
        
        // Use main heightmap directly with scatter system
        let scatter_organic = self.scatter_system.generate_environment_organic(params, heightmap)?;

        // Convert and combine approaches
        let voxel_converted = Self::convert_voxel_environment(voxel_structures);
        let scatter_converted = Self::convert_scatter_environment(scatter_organic);
        
        Ok(GeneratedEnvironment::combine(voxel_converted, scatter_converted))
    }

    /// Update the generation engine (call each frame)
    pub fn update(&mut self, delta_time: f32) {
        self.cache.update(delta_time);
        self.destruction.update(delta_time);
        self.runtime_tools.update(delta_time);
    }

    /// Get generation statistics
    pub fn get_stats(&self) -> GenerationStats {
        GenerationStats {
            cache_hit_rate: self.cache.get_hit_rate(),
            active_generators: self.get_active_generator_count(),
            memory_usage: self.get_memory_usage(),
            generation_time_avg: self.cache.get_avg_generation_time(),
        }
    }

    fn get_active_generator_count(&self) -> usize {
        self.generators.get_active_count() +
        if self.voxel_system.is_active() { 1 } else { 0 } +
        if self.scatter_system.is_active() { 1 } else { 0 }
    }

    fn get_memory_usage(&self) -> usize {
        self.cache.get_memory_usage() +
        self.voxel_system.get_memory_usage() +
        self.scatter_system.get_memory_usage()
    }

    /// Convert noise system GeneratedSurface to main GeneratedSurface
    fn convert_noise_surface(noise_surface: noise::GeneratedSurface, surface_type: SurfaceType) -> GeneratedSurface {
        GeneratedSurface {
            surface_type,
            texture: Texture {
                width: 256, // Default size - noise system doesn't expose dimensions
                height: 256,
                data: noise_surface.texture_data,
                format: TextureFormat::RGBA8,
            },
            normal_map: Some(Texture {
                width: 256,
                height: 256,
                data: noise_surface.normal_map,
                format: TextureFormat::RGBA8,
            }),
            material_data: noise_surface.roughness_map,
        }
    }

    /// Convert voxel system VoxelGeneratedSurface to main GeneratedSurface
    fn convert_voxel_surface(voxel_surface: voxel_system::VoxelGeneratedSurface, surface_type: SurfaceType) -> GeneratedSurface {
        // Create texture data from voxel grid
        let grid_size = voxel_surface.voxel_grid.size;
        let texture_data = voxel_surface.voxel_grid.voxels.iter()
            .flat_map(|x_layer| x_layer.iter())
            .flat_map(|y_layer| y_layer.iter())
            .flat_map(|voxel_opt| {
                // Convert voxel color to RGBA bytes based on voxel type
                if let Some(voxel) = voxel_opt {
                    match voxel.voxel_type {
                        VoxelType::Air => vec![0, 0, 0, 0],
                        VoxelType::Solid => vec![128, 128, 128, 255],
                        VoxelType::Liquid => vec![64, 128, 255, 200],
                        VoxelType::Wood => vec![90, 120, 60, 255],
                        VoxelType::Metal => vec![150, 150, 150, 255],
                        VoxelType::Stone => vec![100, 100, 100, 255],
                        VoxelType::Glass => vec![200, 200, 255, 128],
                        VoxelType::Gas => vec![128, 255, 128, 64],
                        VoxelType::Light => vec![255, 255, 200, 255],
                        VoxelType::Concrete => vec![120, 120, 120, 255],
                        VoxelType::Brick => vec![160, 80, 60, 255],
                        VoxelType::Custom(_) => vec![255, 0, 255, 255], // Magenta for custom
                    }
                } else {
                    vec![0, 0, 0, 0] // Transparent for None
                }
            })
            .collect();

        GeneratedSurface {
            surface_type,
            texture: Texture {
                width: grid_size.0 as u32,
                height: grid_size.1 as u32,
                data: texture_data,
                format: TextureFormat::RGBA8,
            },
            normal_map: None, // Voxel system uses texture_id/normal_map_id references, not direct data
            material_data: vec![], // Convert material_properties to bytes if needed
        }
    }

    /// Convert scatter system GeneratedSurface to main GeneratedSurface
    fn convert_scatter_surface(scatter_surface: pixel_scatter::GeneratedSurface) -> GeneratedSurface {
        // Generate texture from scatter vertices/pattern
        let texture_width = 512u32;
        let texture_height = 512u32;
        let mut texture_data = vec![0u8; (texture_width * texture_height * 4) as usize];

        // Render scatter points to texture data
        for (i, vertex) in scatter_surface.vertices.iter().enumerate() {
            if i < scatter_surface.texture_coords.len() {
                let tex_coord = &scatter_surface.texture_coords[i];
                let x = ((tex_coord.x * texture_width as f32) as u32).min(texture_width - 1);
                let y = ((tex_coord.y * texture_height as f32) as u32).min(texture_height - 1);
                let idx = ((y * texture_width + x) * 4) as usize;
                
                if idx + 3 < texture_data.len() {
                    // Use material properties to determine color
                    texture_data[idx] = (scatter_surface.material_properties.roughness * 255.0) as u8;
                    texture_data[idx + 1] = 128; // Default green component
                    texture_data[idx + 2] = 100; // Default blue component
                    texture_data[idx + 3] = 255; // Alpha
                }
            }
        }

        GeneratedSurface {
            surface_type: SurfaceType::Organic, // Default for scatter surfaces
            texture: Texture {
                width: texture_width,
                height: texture_height,
                data: texture_data,
                format: TextureFormat::RGBA8,
            },
            normal_map: None, // Could generate from scatter normals if needed
            material_data: vec![], // Could serialize material_properties if needed
        }
    }

    /// Convert voxel system GeneratedCharacter to main GeneratedCharacter  
    fn convert_voxel_character(voxel_character: voxel_system::GeneratedCharacter) -> GeneratedCharacter {
        GeneratedCharacter {
            cache_key: format!("voxel_char_{}", uuid::Uuid::new_v4()),
            textures: vec![], // Could extract textures from voxel system if available
            model_data: voxel_character.mesh.vertices.into_iter()
                .flat_map(|v| vec![v.position.x as u8, v.position.y as u8, v.position.z as u8])
                .collect(),
            animations: voxel_character.animations.into_iter().map(|anim| anim.name).collect(),
            character_type: format!("{:?}", voxel_character.metadata.character_type),
        }
    }

    /// Convert destruction system DestructibleEnvironment to main DestructibleEnvironment
    fn convert_destructible_environment(destruction_env: destruction::DestructibleEnvironment) -> DestructibleEnvironment {
        DestructibleEnvironment {
            base_environment: GeneratedEnvironment::default(), // Use default base environment
            destruction_map: vec![0; 1024], // Default destruction map
            debris_templates: vec![], // Empty debris templates for now
        }
    }

    /// Convert ui_generation system GeneratedUIElement to main GeneratedUIElement
    fn convert_ui_element(ui_element: ui_generation::GeneratedUIElement) -> GeneratedUIElement {
        GeneratedUIElement {
            element_type: format!("{:?}", ui_element.element_type),
            texture: Texture::default(), // Create default texture based on style
            layout_data: Vec::new(), // Convert from dimensions and style if needed
        }
    }

    /// Convert content_generators GeneratedTitleSequence to main GeneratedTitleSequence
    fn convert_title_sequence(title: content_generators::GeneratedTitleSequence) -> GeneratedTitleSequence {
        GeneratedTitleSequence {
            frames: Vec::new(), // Convert phases to frames if needed
            audio_cues: Vec::new(), // Extract audio cues from phases
            timing_data: vec![title.total_duration], // Use total duration as timing data
        }
    }


    /// Convert pixel_scatter GeneratedCharacter to main GeneratedCharacter  
    fn convert_scatter_character(character: pixel_scatter::GeneratedCharacter) -> GeneratedCharacter {
        GeneratedCharacter {
            cache_key: character.cache_key,
            textures: Vec::new(), // Convert scatter textures
            model_data: Vec::new(), // Convert scatter data
            animations: Vec::new(), // Convert scatter animations
            character_type: "scatter_based".to_string(),
        }
    }

    /// Convert EnvironmentParams to voxel_system::EnvironmentParams
    fn convert_to_voxel_env_params(params: EnvironmentParams) -> voxel_system::EnvironmentParams {
        let size = params.dimensions.x as u32;
        voxel_system::EnvironmentParams {
            environment_type: content_generators::EnvironmentType::Forest,
            size: [size, size, size],
            world_size: [size, size, size],
            complexity: params.density.max(0.1).min(1.0),
            biome_variety: params.vegetation_density.max(0.1).min(1.0),
        }
    }

    /// Convert Heightmap to voxel_system::Heightmap
    fn convert_to_voxel_heightmap(heightmap: Heightmap) -> voxel_system::Heightmap {
        voxel_system::Heightmap {
            width: heightmap.width,
            height: heightmap.height,
            data: heightmap.data.into_iter().flatten().collect(), // Flatten 2D to 1D
        }
    }



    /// Convert voxel_system GeneratedEnvironmentStructures to main GeneratedEnvironment
    fn convert_voxel_environment(structures: voxel_system::GeneratedEnvironmentStructures) -> GeneratedEnvironment {
        GeneratedEnvironment {
            cache_key: format!("voxel_env_{}", uuid::Uuid::new_v4()),
            terrain_data: Vec::new(), // Convert voxel terrain
            textures: Vec::new(), // Convert voxel textures
            lighting_data: Vec::new(), // Convert lighting data
            collision_data: Vec::new(), // Convert collision data
        }
    }

    /// Convert pixel_scatter GeneratedEnvironment to main GeneratedEnvironment
    fn convert_scatter_environment(env: pixel_scatter::GeneratedEnvironment) -> GeneratedEnvironment {
        GeneratedEnvironment {
            cache_key: env.cache_key,
            terrain_data: Vec::new(), // Convert scatter terrain
            textures: Vec::new(), // Convert scatter textures
            lighting_data: Vec::new(), // Convert lighting data
            collision_data: Vec::new(), // Convert collision data
        }
    }
}

/// Configuration for the generation engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConfig {
    /// Voxel system configuration
    pub voxel_config: VoxelConfig,
    /// Pixel scatter system configuration
    pub scatter_config: PixelScatterConfig,
    /// Noise generation configuration
    pub noise_config: NoiseConfig,
    /// Destruction system configuration
    pub destruction_config: DestructionConfig,
    /// UI generation configuration
    pub ui_config: UIGenerationConfig,
    /// Cache size in MB
    pub cache_size: usize,
    /// Enable multi-threading
    pub multi_threaded: bool,
    /// Quality vs performance balance (0.0 = performance, 1.0 = quality)
    pub quality_balance: f32,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            voxel_config: VoxelConfig::default(),
            scatter_config: PixelScatterConfig::default(),
            noise_config: NoiseConfig::default(),
            destruction_config: DestructionConfig::default(),
            ui_config: UIGenerationConfig::default(),
            cache_size: 256, // 256MB default cache
            multi_threaded: true,
            quality_balance: 0.7, // Favor quality by default
        }
    }
}

/// Different generation styles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum GenerationStyle {
    /// Pure voxel-based generation
    #[default]
    Voxel,
    /// Pure pixel-scatter generation
    PixelScatter,
    /// Hybrid approach combining both
    Hybrid,
}

/// Level of detail for generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DetailLevel {
    Low,
    Medium,
    High,
    Ultra,
}

/// Generation statistics
#[derive(Debug, Clone)]
pub struct GenerationStats {
    pub cache_hit_rate: f32,
    pub active_generators: usize,
    pub memory_usage: usize,
    pub generation_time_avg: f32,
}

/// Generation cache for performance optimization
#[derive(Debug)]
struct GenerationCache {
    characters: HashMap<String, GeneratedCharacter>,
    environments: HashMap<String, GeneratedEnvironment>,
    objects: HashMap<String, GeneratedObject>,
    surfaces: HashMap<String, GeneratedSurface>,
    max_size: usize,
    current_size: usize,
    hits: u64,
    misses: u64,
    total_generation_time: f32,
    generation_count: u64,
}

impl GenerationCache {
    fn new(max_size_mb: usize) -> Self {
        Self {
            characters: HashMap::new(),
            environments: HashMap::new(),
            objects: HashMap::new(),
            surfaces: HashMap::new(),
            max_size: max_size_mb * 1024 * 1024, // Convert to bytes
            current_size: 0,
            hits: 0,
            misses: 0,
            total_generation_time: 0.0,
            generation_count: 0,
        }
    }

    fn get_character(&mut self, key: &str) -> Option<GeneratedCharacter> {
        if let Some(character) = self.characters.get(key) {
            self.hits += 1;
            Some(character.clone())
        } else {
            self.misses += 1;
            None
        }
    }

    fn store_character(&mut self, key: String, character: GeneratedCharacter) {
        // Estimate size and manage cache
        let estimated_size = character.estimate_size();
        if self.current_size + estimated_size > self.max_size {
            self.evict_oldest();
        }
        
        self.characters.insert(key, character);
        self.current_size += estimated_size;
    }

    fn get_environment(&mut self, key: &str) -> Option<GeneratedEnvironment> {
        if let Some(env) = self.environments.get(key) {
            self.hits += 1;
            Some(env.clone())
        } else {
            self.misses += 1;
            None
        }
    }

    fn store_environment(&mut self, key: String, environment: GeneratedEnvironment) {
        let estimated_size = environment.estimate_size();
        if self.current_size + estimated_size > self.max_size {
            self.evict_oldest();
        }
        
        self.environments.insert(key, environment);
        self.current_size += estimated_size;
    }

    fn evict_oldest(&mut self) {
        // Simple eviction strategy - remove half the cache
        // In a production system, you'd want LRU or more sophisticated eviction
        let chars_to_remove = self.characters.len() / 2;
        let envs_to_remove = self.environments.len() / 2;
        
        for _ in 0..chars_to_remove {
            if let Some((_, character)) = self.characters.iter().next() {
                let key = character.cache_key.clone();
                self.current_size -= character.estimate_size();
                self.characters.remove(&key);
            }
        }
        
        for _ in 0..envs_to_remove {
            if let Some((_, env)) = self.environments.iter().next() {
                let key = env.cache_key.clone();
                self.current_size -= env.estimate_size();
                self.environments.remove(&key);
            }
        }
    }

    fn get_hit_rate(&self) -> f32 {
        if self.hits + self.misses == 0 {
            0.0
        } else {
            self.hits as f32 / (self.hits + self.misses) as f32
        }
    }

    fn get_memory_usage(&self) -> usize {
        self.current_size
    }

    fn get_avg_generation_time(&self) -> f32 {
        if self.generation_count == 0 {
            0.0
        } else {
            self.total_generation_time / self.generation_count as f32
        }
    }

    fn update(&mut self, _delta_time: f32) {
        // Could implement cache aging, compression, etc.
    }
}

/// Surface generation techniques
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SurfaceGeneration {
    ProcTexture,
    VoxelBased,
    ScatterBased,
}

// TODO: Complete implementation of generation system parameter types
// Missing parameter types for import compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterParams {
    // TODO: Implement comprehensive character generation parameters
    pub style: GenerationStyle,
    pub detail_level: DetailLevel,
    pub character_type: String,
    pub customization: HashMap<String, String>,
    pub generate_animations: bool,
    pub scale: f32,
    pub primary_color: String,
    pub has_hair: bool,
    pub hair_color: String,
    pub clothing: Vec<String>,
    pub color_scheme: Vec<(String, Color)>, // Color scheme for variants
}

impl CharacterParams {
    pub fn get_cache_key(&self) -> String {
        format!("{:?}_{:?}_{}", self.style, self.detail_level, self.character_type)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentParams {
    pub style: GenerationStyle,
    pub detail_level: DetailLevel,
    pub environment_type: EnvironmentType,
    pub terrain: TerrainType,
    pub climate: String,
    pub density: f32,
    pub dimensions: Vec3,
    pub vegetation_density: f32,
}

impl EnvironmentParams {
    pub fn get_cache_key(&self) -> String {
        format!("{:?}_{:?}_{:?}", self.style, self.detail_level, self.environment_type)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectParams {
    pub style: GenerationStyle,
    pub object_type: String,
    pub material: String,
    pub size: f32,
    pub dimensions: Vec3,
    pub durability: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceParams {
    pub technique: SurfaceGeneration,
    pub surface_type: SurfaceType,
    pub resolution: u32,
    pub material_properties: MaterialProperties,
}

impl SurfaceParams {
    pub fn get_cache_key(&self) -> String {
        format!("{:?}_{:?}_{}", self.technique, self.surface_type, self.resolution)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SurfaceType {
    Metallic,
    Organic,
    Fabric,
    Stone,
    Liquid,
    Natural,
    Glass,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClothingItem {
    pub item_type: String,
    pub color: Color,
    pub material: String,
    pub style: String,
}

#[derive(Debug, Clone)]
pub struct Heightmap {
    pub data: Vec<Vec<f32>>,
    pub width: u32,
    pub height: u32,
}

impl Heightmap {
    pub fn get_height(&self, x: usize, z: usize) -> f32 {
        if x < self.width as usize && z < self.height as usize && z < self.data.len() && x < self.data[z].len() {
            self.data[z][x]
        } else {
            0.0
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TerrainType {
    Plains,
    Mountains,
    Desert,
    Forest,
    Ocean,
    Arctic,
}

// Generated result types
#[derive(Debug, Clone)]
pub struct GeneratedCharacter {
    pub cache_key: String,
    pub textures: Vec<Texture>,
    pub model_data: Vec<u8>,
    pub animations: Vec<String>,
    pub character_type: String,
}

impl GeneratedCharacter {
    pub fn default() -> Self {
        Self {
            cache_key: "default".to_string(),
            textures: vec![],
            model_data: vec![],
            animations: vec![],
            character_type: "basic".to_string(),
        }
    }

    pub fn estimate_size(&self) -> usize {
        self.model_data.len() + self.textures.len() * 1024 // Rough estimation
    }

    pub fn combine(base: Self, details: Self) -> Self {
        // Combine two generated characters
        Self {
            cache_key: format!("{}+{}", base.cache_key, details.cache_key),
            textures: {
                let mut combined = base.textures;
                combined.extend(details.textures);
                combined
            },
            model_data: [base.model_data, details.model_data].concat(),
            animations: [base.animations, details.animations].concat(),
            character_type: base.character_type,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GeneratedEnvironment {
    pub cache_key: String,
    pub terrain_data: Vec<u8>,
    pub textures: Vec<Texture>,
    pub lighting_data: Vec<f32>,
    pub collision_data: Vec<u8>,
}

impl GeneratedEnvironment {
    pub fn default() -> Self {
        Self {
            cache_key: "default".to_string(),
            terrain_data: vec![],
            textures: vec![],
            lighting_data: vec![],
            collision_data: vec![],
        }
    }

    pub fn estimate_size(&self) -> usize {
        self.terrain_data.len() + self.textures.len() * 1024 + self.collision_data.len()
    }

    pub fn combine(base: Self, details: Self) -> Self {
        Self {
            cache_key: format!("{}+{}", base.cache_key, details.cache_key),
            terrain_data: [base.terrain_data, details.terrain_data].concat(),
            textures: {
                let mut combined = base.textures;
                combined.extend(details.textures);
                combined
            },
            lighting_data: [base.lighting_data, details.lighting_data].concat(),
            collision_data: [base.collision_data, details.collision_data].concat(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GeneratedObject {
    pub object_type: String,
    pub texture: Texture,
    pub model_data: Vec<u8>,
    pub collision_shape: Vec<f32>,
}

impl Default for GeneratedObject {
    fn default() -> Self {
        Self {
            object_type: "Default".to_string(),
            texture: Texture::default(),
            model_data: vec![0; 1024], // Default model data
            collision_shape: vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0], // Default bounding box
        }
    }
}

#[derive(Debug, Clone)]
pub struct GeneratedSurface {
    pub surface_type: SurfaceType,
    pub texture: Texture,
    pub normal_map: Option<Texture>,
    pub material_data: Vec<u8>,
}

impl Default for GeneratedSurface {
    fn default() -> Self {
        Self {
            surface_type: SurfaceType::Organic,
            texture: Texture::default(),
            normal_map: None,
            material_data: vec![],
        }
    }
}

// Destruction types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DestructionParams {
    pub destructible_type: String,
    pub material_strength: f32,
    pub particle_count: u32,
}

#[derive(Debug, Clone)]
pub struct DestructibleEnvironment {
    pub base_environment: GeneratedEnvironment,
    pub destruction_map: Vec<u8>,
    pub debris_templates: Vec<GeneratedObject>,
}

// UI Generation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIGenerationParams {
    pub ui_type: String,
    pub theme: String,
    pub size: Vec2,
}

#[derive(Debug, Clone)]
pub struct GeneratedUIElement {
    pub element_type: String,
    pub texture: Texture,
    pub layout_data: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitleSequenceParams {
    pub style: String,
    pub duration: f32,
    pub effects: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct GeneratedTitleSequence {
    pub frames: Vec<Texture>,
    pub audio_cues: Vec<String>,
    pub timing_data: Vec<f32>,
}

// Type alias for import compatibility
pub use ui_generation::UIGenerator;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation_engine_creation() {
        let config = GenerationConfig::default();
        let engine = GenerationEngine::new(config);
        
        assert!(engine.cache.characters.is_empty());
        assert!(engine.cache.environments.is_empty());
    }

    #[test]
    fn test_cache_functionality() {
        let mut cache = GenerationCache::new(10); // 10MB cache
        
        // Should start empty
        assert_eq!(cache.get_hit_rate(), 0.0);
        
        // Test miss
        assert!(cache.get_character("test_key").is_none());
        assert_eq!(cache.get_hit_rate(), 0.0);
        
        // Store and retrieve
        let character = GeneratedCharacter::default();
        cache.store_character("test_key".to_string(), character);
        
        assert!(cache.get_character("test_key").is_some());
        assert!(cache.get_hit_rate() > 0.0);
    }
}