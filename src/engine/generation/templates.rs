/*!
 * Robin Engine Template System
 * 
 * A comprehensive library of reusable generation patterns and templates
 * for consistent, high-quality procedural content generation.
 */

use crate::engine::{
    graphics::{Texture, Color},
    math::{Vec2, Vec3, Transform},
    error::{RobinError, RobinResult},
};
use super::{
    VoxelType, GenerationStyle, DetailLevel,
    DestructibleEnvironment,
    destruction::DestructionParams,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Template library containing reusable generation patterns
#[derive(Debug)]
pub struct TemplateLibrary {
    /// Character archetypes and templates
    character_templates: HashMap<String, CharacterTemplate>,
    /// Environment and building templates
    environment_templates: HashMap<String, EnvironmentTemplate>,
    /// Object and item templates
    object_templates: HashMap<String, ObjectTemplate>,
    /// Architectural patterns
    architecture_templates: HashMap<String, ArchitectureTemplate>,
    /// Terrain generation templates
    terrain_templates: HashMap<String, TerrainTemplate>,
    /// UI design templates
    ui_templates: HashMap<String, UITemplate>,
}

impl TemplateLibrary {
    pub fn new() -> Self {
        let mut library = Self {
            character_templates: HashMap::new(),
            environment_templates: HashMap::new(),
            object_templates: HashMap::new(),
            architecture_templates: HashMap::new(),
            terrain_templates: HashMap::new(),
            ui_templates: HashMap::new(),
        };

        // Load default templates
        library.load_default_templates();
        library
    }

    /// Load default template library
    fn load_default_templates(&mut self) {
        self.load_default_character_templates();
        self.load_default_environment_templates();
        self.load_default_object_templates();
        self.load_default_architecture_templates();
        self.load_default_terrain_templates();
        self.load_default_ui_templates();
    }

    /// Get character template by name
    pub fn get_character_template(&self, name: &str) -> Option<&CharacterTemplate> {
        self.character_templates.get(name)
    }

    /// Get environment template by name
    pub fn get_environment_template(&self, name: &str) -> Option<&EnvironmentTemplate> {
        self.environment_templates.get(name)
    }

    /// Get object template by name
    pub fn get_object_template(&self, name: &str) -> Option<&ObjectTemplate> {
        self.object_templates.get(name)
    }

    /// Add custom character template
    pub fn add_character_template(&mut self, name: String, template: CharacterTemplate) {
        self.character_templates.insert(name, template);
    }

    /// Add custom environment template
    pub fn add_environment_template(&mut self, name: String, template: EnvironmentTemplate) {
        self.environment_templates.insert(name, template);
    }

    /// Generate character from template
    pub fn generate_character_from_template(&self, template_name: &str, customization: Option<CharacterCustomization>) -> RobinResult<CharacterParams> {
        let template = self.get_character_template(template_name)
            .ok_or_else(|| RobinError::TemplateError(format!("Character template not found: {}", template_name)))?;

        let mut params = template.base_params.clone();
        
        // Apply customizations
        if let Some(custom) = customization {
            if let Some(scale) = custom.scale_override {
                params.scale = scale;
            }
            if let Some(colors) = custom.color_overrides {
                params.color_scheme.extend(colors);
            }
            if let Some(style) = custom.style_override {
                params.style = style;
            }
        }

        Ok(params)
    }

    /// Generate environment from template
    pub fn generate_environment_from_template(&self, template_name: &str, customization: Option<EnvironmentCustomization>) -> RobinResult<EnvironmentParams> {
        let template = self.get_environment_template(template_name)
            .ok_or_else(|| RobinError::TemplateError(format!("Environment template not found: {}", template_name)))?;

        let mut params = template.base_params.clone();
        
        // Apply customizations
        if let Some(custom) = customization {
            if let Some(size) = custom.size_override {
                params.dimensions = size;
            }
            if let Some(biome) = custom.biome_override {
                params.biome = biome;
            }
            if let Some(weather) = custom.weather_override {
                params.weather_conditions = weather;
            }
        }

        Ok(params)
    }

    /// List available templates
    pub fn list_character_templates(&self) -> Vec<&String> {
        self.character_templates.keys().collect()
    }

    pub fn list_environment_templates(&self) -> Vec<&String> {
        self.environment_templates.keys().collect()
    }

    pub fn list_object_templates(&self) -> Vec<&String> {
        self.object_templates.keys().collect()
    }

    /// Load character templates
    fn load_default_character_templates(&mut self) {
        // Hero templates
        self.character_templates.insert("classic_hero".to_string(), CharacterTemplate {
            name: "Classic Hero".to_string(),
            description: "Traditional fantasy hero with sword and armor".to_string(),
            category: TemplateCategory::Hero,
            base_params: CharacterParams {
                archetype: CharacterArchetype::Hero,
                scale: Vec3::new(1.0, 1.0, 1.0),
                color_scheme: vec![
                    ("primary".to_string(), Color::new(0.8, 0.6, 0.4, 1.0)), // Skin
                    ("secondary".to_string(), Color::new(0.6, 0.4, 0.2, 1.0)), // Hair
                    ("armor".to_string(), Color::new(0.7, 0.7, 0.8, 1.0)), // Metal armor
                    ("accent".to_string(), Color::new(0.8, 0.2, 0.2, 1.0)), // Red accents
                ],
                style: GenerationStyle::Hybrid,
                detail_level: DetailLevel::High,
                equipment: vec!["sword".to_string(), "shield".to_string(), "armor".to_string()],
                animations: vec!["idle".to_string(), "walk".to_string(), "attack".to_string()],
                special_abilities: vec!["heroic_strike".to_string()],
            },
            variations: vec![
                ("paladin".to_string(), vec![
                    ("armor".to_string(), Color::new(0.9, 0.9, 0.6, 1.0)), // Golden armor
                    ("accent".to_string(), Color::new(0.9, 0.9, 0.9, 1.0)), // White accents
                ]),
                ("dark_knight".to_string(), vec![
                    ("armor".to_string(), Color::new(0.2, 0.2, 0.3, 1.0)), // Dark armor
                    ("accent".to_string(), Color::new(0.5, 0.1, 0.8, 1.0)), // Purple accents
                ]),
            ],
            tags: vec!["fantasy".to_string(), "warrior".to_string(), "melee".to_string()],
        });

        self.character_templates.insert("space_marine".to_string(), CharacterTemplate {
            name: "Space Marine".to_string(),
            description: "Futuristic soldier with power armor and energy weapons".to_string(),
            category: TemplateCategory::Hero,
            base_params: CharacterParams {
                archetype: CharacterArchetype::Hero,
                scale: Vec3::new(1.2, 1.2, 1.2),
                color_scheme: vec![
                    ("armor".to_string(), Color::new(0.3, 0.3, 0.4, 1.0)), // Dark metal
                    ("energy".to_string(), Color::new(0.2, 0.6, 1.0, 1.0)), // Blue energy
                    ("visor".to_string(), Color::new(0.8, 0.3, 0.0, 1.0)), // Orange visor
                ],
                style: GenerationStyle::Voxel,
                detail_level: DetailLevel::Ultra,
                equipment: vec!["plasma_rifle".to_string(), "power_armor".to_string()],
                animations: vec!["idle".to_string(), "march".to_string(), "shoot".to_string()],
                special_abilities: vec!["plasma_burst".to_string()],
            },
            variations: vec![
                ("assault".to_string(), vec![
                    ("energy".to_string(), Color::new(1.0, 0.3, 0.2, 1.0)), // Red energy
                ]),
                ("stealth".to_string(), vec![
                    ("armor".to_string(), Color::new(0.1, 0.1, 0.2, 1.0)), // Darker armor
                ]),
            ],
            tags: vec!["sci-fi".to_string(), "soldier".to_string(), "ranged".to_string()],
        });

        // Creature templates
        self.character_templates.insert("forest_dragon".to_string(), CharacterTemplate {
            name: "Forest Dragon".to_string(),
            description: "Majestic dragon with nature-themed appearance".to_string(),
            category: TemplateCategory::Creature,
            base_params: CharacterParams {
                archetype: CharacterArchetype::Creature,
                scale: Vec3::new(3.0, 2.5, 4.0),
                color_scheme: vec![
                    ("scales".to_string(), Color::new(0.2, 0.6, 0.3, 1.0)), // Green scales
                    ("belly".to_string(), Color::new(0.8, 0.8, 0.6, 1.0)), // Light belly
                    ("eyes".to_string(), Color::new(0.9, 0.8, 0.2, 1.0)), // Golden eyes
                    ("fire".to_string(), Color::new(0.9, 0.4, 0.1, 1.0)), // Fire breath
                ],
                style: GenerationStyle::PixelScatter,
                detail_level: DetailLevel::Ultra,
                equipment: vec![],
                animations: vec!["idle".to_string(), "fly".to_string(), "breathe_fire".to_string(), "roar".to_string()],
                special_abilities: vec!["fire_breath".to_string(), "flight".to_string()],
            },
            variations: vec![
                ("ice_dragon".to_string(), vec![
                    ("scales".to_string(), Color::new(0.6, 0.8, 0.9, 1.0)), // Blue scales
                    ("fire".to_string(), Color::new(0.8, 0.9, 1.0, 1.0)), // Ice breath
                ]),
                ("shadow_dragon".to_string(), vec![
                    ("scales".to_string(), Color::new(0.2, 0.1, 0.3, 1.0)), // Dark scales
                    ("fire".to_string(), Color::new(0.5, 0.2, 0.8, 1.0)), // Shadow breath
                ]),
            ],
            tags: vec!["fantasy".to_string(), "dragon".to_string(), "boss".to_string()],
        });

        // NPC templates
        self.character_templates.insert("merchant".to_string(), CharacterTemplate {
            name: "Merchant".to_string(),
            description: "Friendly shopkeeper with various wares".to_string(),
            category: TemplateCategory::NPC,
            base_params: CharacterParams {
                archetype: CharacterArchetype::NPC,
                scale: Vec3::new(0.9, 0.9, 0.9),
                color_scheme: vec![
                    ("clothing".to_string(), Color::new(0.6, 0.4, 0.3, 1.0)), // Brown clothes
                    ("apron".to_string(), Color::new(0.8, 0.8, 0.7, 1.0)), // Light apron
                    ("hair".to_string(), Color::new(0.5, 0.3, 0.2, 1.0)), // Brown hair
                ],
                style: GenerationStyle::Voxel,
                detail_level: DetailLevel::Medium,
                equipment: vec!["coin_purse".to_string(), "shop_counter".to_string()],
                animations: vec!["idle".to_string(), "wave".to_string(), "count_coins".to_string()],
                special_abilities: vec!["trade".to_string()],
            },
            variations: vec![
                ("blacksmith".to_string(), vec![
                    ("clothing".to_string(), Color::new(0.3, 0.2, 0.1, 1.0)), // Darker clothes
                    ("apron".to_string(), Color::new(0.4, 0.3, 0.2, 1.0)), // Leather apron
                ]),
                ("alchemist".to_string(), vec![
                    ("clothing".to_string(), Color::new(0.4, 0.2, 0.6, 1.0)), // Purple robes
                ]),
            ],
            tags: vec!["npc".to_string(), "shopkeeper".to_string(), "civilian".to_string()],
        });
    }

    /// Load environment templates
    fn load_default_environment_templates(&mut self) {
        self.environment_templates.insert("medieval_castle".to_string(), EnvironmentTemplate {
            name: "Medieval Castle".to_string(),
            description: "Imposing stone castle with towers and battlements".to_string(),
            category: EnvironmentCategory::Architecture,
            base_params: EnvironmentParams {
                dimensions: Vec3::new(100.0, 50.0, 100.0),
                biome: BiomeType::Temperate,
                style: GenerationStyle::Voxel,
                detail_level: DetailLevel::High,
                lighting_conditions: LightingType::Dynamic,
                weather_conditions: WeatherType::Clear,
                terrain: TerrainType::Hills,
                vegetation_density: 0.3,
                water_features: vec![WaterFeature::Moat],
                structures: vec![
                    StructureDefinition {
                        structure_type: StructureType::Castle,
                        position: Vec3::new(50.0, 0.0, 50.0),
                        dimensions: Vec3::new(60.0, 40.0, 60.0),
                        material: VoxelType::Stone,
                    }
                ],
                ambient_sounds: vec!["wind".to_string(), "birds".to_string()],
            },
            destructible_config: Some(DestructionParams {
                environment_id: "medieval_castle".to_string(),
                dimensions: Vec3::new(100.0, 50.0, 100.0),
                structure_type: super::destruction::StructureType::Building,
                material_distribution: super::destruction::MaterialDistribution {
                    primary_material: VoxelType::Stone,
                    secondary_materials: vec![
                        (VoxelType::Wood, 0.3),
                        (VoxelType::Metal, 0.1),
                    ],
                    structural_materials: vec![VoxelType::Stone, VoxelType::Metal],
                },
                structural_integrity: 0.8,
                triggers: vec![],
                environmental_effects: vec![],
            }),
            variations: vec![
                ("ruined_castle".to_string(), EnvironmentModifications {
                    structural_integrity: Some(0.3),
                    vegetation_density: Some(0.8),
                    weather_override: Some(WeatherType::Stormy),
                    biome_override: None,
                    lighting_override: None,
                }),
                ("snow_castle".to_string(), EnvironmentModifications {
                    structural_integrity: None,
                    vegetation_density: None,
                    weather_override: Some(WeatherType::Snow),
                    lighting_override: Some(LightingType::Cold),
                    biome_override: Some(BiomeType::Arctic),
                }),
            ],
            tags: vec!["medieval".to_string(), "castle".to_string(), "stone".to_string()],
        });

        self.environment_templates.insert("cyberpunk_city".to_string(), EnvironmentTemplate {
            name: "Cyberpunk City".to_string(),
            description: "Neon-lit futuristic cityscape with towering skyscrapers".to_string(),
            category: EnvironmentCategory::Urban,
            base_params: EnvironmentParams {
                dimensions: Vec3::new(200.0, 150.0, 200.0),
                biome: BiomeType::Urban,
                style: GenerationStyle::Hybrid,
                detail_level: DetailLevel::Ultra,
                lighting_conditions: LightingType::Neon,
                weather_conditions: WeatherType::Rain,
                terrain: TerrainType::Flat,
                vegetation_density: 0.1,
                water_features: vec![WaterFeature::Puddles],
                structures: vec![
                    StructureDefinition {
                        structure_type: StructureType::Skyscraper,
                        position: Vec3::new(100.0, 0.0, 100.0),
                        dimensions: Vec3::new(20.0, 120.0, 20.0),
                        material: VoxelType::Custom(3), // High-tech building material ID
                    }
                ],
                ambient_sounds: vec!["rain".to_string(), "traffic".to_string(), "neon_hum".to_string()],
            },
            destructible_config: Some(DestructionParams {
                environment_id: "cyberpunk_city".to_string(),
                dimensions: Vec3::new(200.0, 150.0, 200.0),
                structure_type: super::destruction::StructureType::Building,
                material_distribution: super::destruction::MaterialDistribution {
                    primary_material: VoxelType::Metal,
                    secondary_materials: vec![
                        (VoxelType::Glass, 0.4),
                        (VoxelType::Concrete, 0.2),
                    ],
                    structural_materials: vec![VoxelType::Metal, VoxelType::Concrete],
                },
                structural_integrity: 0.9,
                triggers: vec![],
                environmental_effects: vec![],
            }),
            variations: vec![
                ("abandoned_city".to_string(), EnvironmentModifications {
                    structural_integrity: Some(0.4),
                    vegetation_density: Some(0.6),
                    lighting_override: Some(LightingType::Dark),
                    weather_override: Some(WeatherType::Clear),
                    biome_override: Some(BiomeType::Urban),
                }),
            ],
            tags: vec!["cyberpunk".to_string(), "city".to_string(), "futuristic".to_string()],
        });

        self.environment_templates.insert("enchanted_forest".to_string(), EnvironmentTemplate {
            name: "Enchanted Forest".to_string(),
            description: "Magical forest with glowing plants and mystical atmosphere".to_string(),
            category: EnvironmentCategory::Natural,
            base_params: EnvironmentParams {
                dimensions: Vec3::new(150.0, 80.0, 150.0),
                biome: BiomeType::Forest,
                style: GenerationStyle::PixelScatter,
                detail_level: DetailLevel::High,
                lighting_conditions: LightingType::Magical,
                weather_conditions: WeatherType::Misty,
                terrain: TerrainType::Rolling,
                vegetation_density: 0.9,
                water_features: vec![WaterFeature::Stream, WaterFeature::Pond],
                structures: vec![
                    StructureDefinition {
                        structure_type: StructureType::TreeHouse,
                        position: Vec3::new(75.0, 20.0, 75.0),
                        dimensions: Vec3::new(10.0, 15.0, 10.0),
                        material: VoxelType::Wood,
                    }
                ],
                ambient_sounds: vec!["forest_ambience".to_string(), "magical_chimes".to_string()],
            },
            destructible_config: None, // Magical forest is not destructible by default
            variations: vec![
                ("dark_forest".to_string(), EnvironmentModifications {
                    structural_integrity: Some(0.9),
                    lighting_override: Some(LightingType::Dark),
                    weather_override: Some(WeatherType::Fog),
                    vegetation_density: Some(0.7),
                    biome_override: Some(BiomeType::Forest),
                }),
            ],
            tags: vec!["fantasy".to_string(), "forest".to_string(), "magical".to_string()],
        });
    }

    /// Load object templates
    fn load_default_object_templates(&mut self) {
        self.object_templates.insert("treasure_chest".to_string(), ObjectTemplate {
            name: "Treasure Chest".to_string(),
            description: "Ornate chest containing valuable items".to_string(),
            category: ObjectCategory::Container,
            base_params: ObjectParams {
                object_type: ObjectType::Container,
                dimensions: Vec3::new(2.0, 1.5, 1.0),
                material: VoxelType::Wood,
                style: GenerationStyle::Voxel,
                detail_level: DetailLevel::High,
                interactive: true,
                physics_enabled: false,
                durability: 100.0,
                special_properties: vec![
                    ("lockable".to_string(), "true".to_string()),
                    ("capacity".to_string(), "50".to_string()),
                ],
            },
            variations: vec![
                ("golden_chest".to_string(), vec![
                    ("material".to_string(), "gold".to_string()),
                    ("value_multiplier".to_string(), "3.0".to_string()),
                ]),
                ("locked_chest".to_string(), vec![
                    ("locked".to_string(), "true".to_string()),
                    ("lock_difficulty".to_string(), "medium".to_string()),
                ]),
            ],
            tags: vec!["container".to_string(), "treasure".to_string(), "interactive".to_string()],
        });

        self.object_templates.insert("magic_sword".to_string(), ObjectTemplate {
            name: "Magic Sword".to_string(),
            description: "Enchanted blade with magical properties".to_string(),
            category: ObjectCategory::Weapon,
            base_params: ObjectParams {
                object_type: ObjectType::Weapon,
                dimensions: Vec3::new(0.1, 1.2, 0.05),
                material: VoxelType::Metal,
                style: GenerationStyle::Hybrid,
                detail_level: DetailLevel::Ultra,
                interactive: true,
                physics_enabled: true,
                durability: 200.0,
                special_properties: vec![
                    ("damage".to_string(), "50".to_string()),
                    ("enchantment".to_string(), "fire".to_string()),
                    ("glow".to_string(), "true".to_string()),
                ],
            },
            variations: vec![
                ("ice_sword".to_string(), vec![
                    ("enchantment".to_string(), "ice".to_string()),
                    ("glow_color".to_string(), "blue".to_string()),
                ]),
                ("shadow_blade".to_string(), vec![
                    ("enchantment".to_string(), "shadow".to_string()),
                    ("stealth_bonus".to_string(), "20".to_string()),
                ]),
            ],
            tags: vec!["weapon".to_string(), "sword".to_string(), "magical".to_string()],
        });

        self.object_templates.insert("campfire".to_string(), ObjectTemplate {
            name: "Campfire".to_string(),
            description: "Warm fire for cooking and light".to_string(),
            category: ObjectCategory::Utility,
            base_params: ObjectParams {
                object_type: ObjectType::Utility,
                dimensions: Vec3::new(1.5, 0.8, 1.5),
                material: VoxelType::Wood,
                style: GenerationStyle::PixelScatter,
                detail_level: DetailLevel::Medium,
                interactive: true,
                physics_enabled: false,
                durability: 50.0,
                special_properties: vec![
                    ("light_radius".to_string(), "10.0".to_string()),
                    ("heat_radius".to_string(), "5.0".to_string()),
                    ("fuel_type".to_string(), "wood".to_string()),
                ],
            },
            variations: vec![
                ("magical_fire".to_string(), vec![
                    ("fuel_type".to_string(), "magic".to_string()),
                    ("light_radius".to_string(), "20.0".to_string()),
                    ("color".to_string(), "blue".to_string()),
                ]),
            ],
            tags: vec!["utility".to_string(), "fire".to_string(), "light".to_string()],
        });
    }

    /// Load architecture templates
    fn load_default_architecture_templates(&mut self) {
        // Implementation for architectural patterns
    }

    /// Load terrain templates
    fn load_default_terrain_templates(&mut self) {
        // Implementation for terrain generation patterns
    }

    /// Load UI templates
    fn load_default_ui_templates(&mut self) {
        // Implementation for UI generation templates
    }
}

/// Character template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterTemplate {
    pub name: String,
    pub description: String,
    pub category: TemplateCategory,
    pub base_params: CharacterParams,
    pub variations: Vec<(String, Vec<(String, Color)>)>, // name, color overrides
    pub tags: Vec<String>,
}

/// Environment template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentTemplate {
    pub name: String,
    pub description: String,
    pub category: EnvironmentCategory,
    pub base_params: EnvironmentParams,
    pub destructible_config: Option<DestructionParams>,
    pub variations: Vec<(String, EnvironmentModifications)>,
    pub tags: Vec<String>,
}

/// Object template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectTemplate {
    pub name: String,
    pub description: String,
    pub category: ObjectCategory,
    pub base_params: ObjectParams,
    pub variations: Vec<(String, Vec<(String, String)>)>, // name, property overrides
    pub tags: Vec<String>,
}

/// Architecture template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureTemplate {
    pub name: String,
    pub description: String,
    pub style: ArchitecturalStyle,
    pub components: Vec<ArchitecturalComponent>,
    pub proportions: ArchitecturalProportions,
}

/// Terrain template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainTemplate {
    pub name: String,
    pub description: String,
    pub height_pattern: HeightPattern,
    pub material_layers: Vec<TerrainLayer>,
    pub feature_distribution: FeatureDistribution,
}

/// UI template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UITemplate {
    pub name: String,
    pub description: String,
    pub theme: UITheme,
    pub components: Vec<UIComponent>,
    pub layout: UILayout,
}

/// Template categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemplateCategory {
    Hero,
    Villain,
    NPC,
    Creature,
    Robot,
    Elemental,
    Abstract,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnvironmentCategory {
    Natural,
    Urban,
    Architecture,
    Dungeon,
    Space,
    Underwater,
    Abstract,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectCategory {
    Weapon,
    Tool,
    Container,
    Furniture,
    Decoration,
    Utility,
    Consumable,
}

/// Customization options
#[derive(Debug, Clone)]
pub struct CharacterCustomization {
    pub scale_override: Option<Vec3>,
    pub color_overrides: Option<Vec<(String, Color)>>,
    pub style_override: Option<GenerationStyle>,
    pub equipment_overrides: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct EnvironmentCustomization {
    pub size_override: Option<Vec3>,
    pub biome_override: Option<BiomeType>,
    pub weather_override: Option<WeatherType>,
    pub lighting_override: Option<LightingType>,
}

/// Environment modifications for template variations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentModifications {
    pub structural_integrity: Option<f32>,
    pub vegetation_density: Option<f32>,
    pub weather_override: Option<WeatherType>,
    pub lighting_override: Option<LightingType>,
    pub biome_override: Option<BiomeType>,
}

/// Supporting enums and structs (these would be defined in their respective modules)

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharacterArchetype {
    Hero,
    Villain,
    NPC,
    Creature,
    Robot,
    Elemental,
    Abstract,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BiomeType {
    Temperate,
    Desert,
    Arctic,
    Tropical,
    Forest,
    Mountain,
    Swamp,
    Urban,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeatherType {
    Clear,
    Cloudy,
    Rain,
    Storm,
    Snow,
    Fog,
    Misty,
    Stormy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LightingType {
    Bright,
    Dim,
    Dark,
    Dynamic,
    Neon,
    Magical,
    Cold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerrainType {
    Flat,
    Hills,
    Mountains,
    Valley,
    Canyon,
    Rolling,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WaterFeature {
    River,
    Lake,
    Ocean,
    Stream,
    Pond,
    Waterfall,
    Moat,
    Puddles,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StructureType {
    Building,
    Castle,
    Tower,
    Bridge,
    Wall,
    Ruins,
    TreeHouse,
    Skyscraper,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureDefinition {
    pub structure_type: StructureType,
    pub position: Vec3,
    pub dimensions: Vec3,
    pub material: VoxelType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectType {
    Weapon,
    Tool,
    Container,
    Furniture,
    Decoration,
    Utility,
    Consumable,
    Vehicle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArchitecturalStyle {
    Medieval,
    Gothic,
    Renaissance,
    Modern,
    Futuristic,
    Fantasy,
    Industrial,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalComponent {
    pub component_type: ComponentType,
    pub dimensions: Vec3,
    pub material: VoxelType,
    pub position_rules: PositionRules,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComponentType {
    Wall,
    Roof,
    Door,
    Window,
    Column,
    Stair,
    Floor,
    Foundation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionRules {
    pub relative_to: ComponentType,
    pub offset: Vec3,
    pub alignment: Alignment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Alignment {
    Center,
    Left,
    Right,
    Top,
    Bottom,
    Front,
    Back,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalProportions {
    pub wall_height: f32,
    pub room_size: Vec2,
    pub window_ratio: f32,
    pub door_size: Vec2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeightPattern {
    Flat,
    Perlin,
    Ridged,
    Valleys,
    Peaks,
    Terraced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainLayer {
    pub material: VoxelType,
    pub thickness: f32,
    pub depth_range: (f32, f32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureDistribution {
    pub trees: f32,
    pub rocks: f32,
    pub water: f32,
    pub structures: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UITheme {
    Minimal,
    Fantasy,
    SciFi,
    Retro,
    Modern,
    Dark,
    Light,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIComponent {
    pub component_type: UIComponentType,
    pub style_properties: Vec<(String, String)>,
    pub layout_properties: UILayoutProperties,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UIComponentType {
    Button,
    Panel,
    Text,
    Image,
    Slider,
    ProgressBar,
    Menu,
    Dialog,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UILayoutProperties {
    pub position: Vec2,
    pub size: Vec2,
    pub anchor: UIAnchor,
    pub margin: UISpacing,
    pub padding: UISpacing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UIAnchor {
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    MiddleCenter,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UISpacing {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UILayout {
    Fixed,
    Flex,
    Grid,
    Stack,
}

// Placeholder structs for missing types (these would be properly implemented in their modules)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterParams {
    pub archetype: CharacterArchetype,
    pub scale: Vec3,
    pub color_scheme: Vec<(String, Color)>,
    pub style: GenerationStyle,
    pub detail_level: DetailLevel,
    pub equipment: Vec<String>,
    pub animations: Vec<String>,
    pub special_abilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentParams {
    pub dimensions: Vec3,
    pub biome: BiomeType,
    pub style: GenerationStyle,
    pub detail_level: DetailLevel,
    pub lighting_conditions: LightingType,
    pub weather_conditions: WeatherType,
    pub terrain: TerrainType,
    pub vegetation_density: f32,
    pub water_features: Vec<WaterFeature>,
    pub structures: Vec<StructureDefinition>,
    pub ambient_sounds: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectParams {
    pub object_type: ObjectType,
    pub dimensions: Vec3,
    pub material: VoxelType,
    pub style: GenerationStyle,
    pub detail_level: DetailLevel,
    pub interactive: bool,
    pub physics_enabled: bool,
    pub durability: f32,
    pub special_properties: Vec<(String, String)>,
}

// Placeholder for generated types
#[derive(Debug, Clone)]
pub struct GeneratedCharacter {
    pub cache_key: String,
    // Additional fields would be implemented
}

impl GeneratedCharacter {
    pub fn estimate_size(&self) -> usize {
        1024 // Placeholder
    }

    pub fn default() -> Self {
        Self {
            cache_key: "default".to_string(),
        }
    }

    pub fn combine(voxel_base: GeneratedCharacter, scatter_details: GeneratedCharacter) -> Self {
        Self {
            cache_key: format!("{}+{}", voxel_base.cache_key, scatter_details.cache_key),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GeneratedEnvironment {
    pub cache_key: String,
    // Additional fields would be implemented
}

impl GeneratedEnvironment {
    pub fn estimate_size(&self) -> usize {
        2048 // Placeholder
    }

    pub fn combine(voxel_structures: GeneratedEnvironment, scatter_organic: GeneratedEnvironment) -> Self {
        Self {
            cache_key: format!("{}+{}", voxel_structures.cache_key, scatter_organic.cache_key),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GeneratedObject {
    // Implementation would go here
}

// Missing type definitions for import resolution
#[derive(Debug, Clone)]
pub struct GenerationTemplate {
    pub name: String,
    pub template_type: String,
    pub parameters: Vec<String>,
    pub constraints: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TemplateConfig {
    pub enabled_templates: Vec<String>,
    pub quality_settings: String,
    pub generation_speed: f32,
    pub resource_limits: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_library_creation() {
        let library = TemplateLibrary::new();
        
        assert!(!library.character_templates.is_empty());
        assert!(!library.environment_templates.is_empty());
        assert!(!library.object_templates.is_empty());
    }

    #[test]
    fn test_character_template_retrieval() {
        let library = TemplateLibrary::new();
        
        let hero_template = library.get_character_template("classic_hero");
        assert!(hero_template.is_some());
        
        let template = hero_template.unwrap();
        assert_eq!(template.name, "Classic Hero");
        assert_eq!(template.category, TemplateCategory::Hero);
    }

    #[test]
    fn test_template_generation() {
        let library = TemplateLibrary::new();
        
        let character_params = library.generate_character_from_template("classic_hero", None);
        assert!(character_params.is_ok());
        
        let params = character_params.unwrap();
        assert_eq!(params.archetype, CharacterArchetype::Hero);
    }

    #[test]
    fn test_template_customization() {
        let library = TemplateLibrary::new();
        
        let customization = CharacterCustomization {
            scale_override: Some(Vec3::new(2.0, 2.0, 2.0)),
            color_overrides: None,
            style_override: Some(GenerationStyle::PixelScatter),
            equipment_overrides: None,
        };
        
        let character_params = library.generate_character_from_template("classic_hero", Some(customization));
        assert!(character_params.is_ok());
        
        let params = character_params.unwrap();
        assert_eq!(params.scale, Vec3::new(2.0, 2.0, 2.0));
        assert_eq!(params.style, GenerationStyle::PixelScatter);
    }
}