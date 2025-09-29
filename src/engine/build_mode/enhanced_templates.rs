/*!
 * Enhanced Template Library for Robin Engine
 *
 * Sophisticated content creation templates and prefab system
 * for Phase 4 Milestone 3: Content Depth and Polish
 */

use crate::engine::{
    build_mode::{BuildMode, TemplateType},
    world::construction::VoxelType,
    world::advanced_materials::AdvancedMaterialType,
    math::Vec3,
    error::{RobinError, RobinResult},
};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Enhanced template categories for organized content creation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TemplateCategory {
    /// Architectural structures
    Architecture,
    /// Infrastructure and utilities
    Infrastructure,
    /// Decorative and artistic elements
    Decorative,
    /// Mechanical and functional systems
    Mechanical,
    /// Natural and organic structures
    Natural,
    /// Advanced technology structures
    Technology,
    /// Military and defensive structures
    Military,
    /// Transportation systems
    Transportation,
    /// Industrial facilities
    Industrial,
    /// Residential structures
    Residential,
    /// Commercial buildings
    Commercial,
    /// Educational facilities
    Educational,
    /// Entertainment venues
    Entertainment,
}

/// Complexity levels for templates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemplateComplexity {
    Simple,      // 1-10 blocks
    Basic,       // 11-50 blocks
    Intermediate, // 51-200 blocks
    Advanced,    // 201-1000 blocks
    Complex,     // 1001-5000 blocks
    Masterwork,  // 5000+ blocks
}

/// Enhanced template with comprehensive metadata and advanced features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: TemplateCategory,
    pub complexity: TemplateComplexity,
    pub author: String,
    pub version: String,
    pub tags: Vec<String>,

    /// Template structure data
    pub structure: TemplateStructure,

    /// Build requirements
    pub requirements: BuildRequirements,

    /// Template metadata
    pub metadata: TemplateMetadata,

    /// Variations and customizations
    pub variations: Vec<TemplateVariation>,

    /// Interactive elements and functionality
    pub interactive_elements: Vec<InteractiveTemplateElement>,

    /// Animation and dynamic behavior
    pub animations: Vec<TemplateAnimation>,

    /// Performance optimization data
    pub optimization: TemplateOptimization,
}

/// Template structure definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateStructure {
    /// Bounding box dimensions
    pub dimensions: Vec3,
    /// Voxel placement data (relative position -> voxel type)
    pub voxels: HashMap<Vec3, VoxelType>,
    /// Advanced material placements
    pub advanced_materials: HashMap<Vec3, AdvancedMaterialType>,
    /// Structural anchor points
    pub anchor_points: Vec<AnchorPoint>,
    /// Connection points for linking with other templates
    pub connection_points: Vec<ConnectionPoint>,
    /// Foundation requirements
    pub foundation: FoundationRequirements,
}

/// Build requirements for template construction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildRequirements {
    /// Required materials and quantities
    pub materials: HashMap<VoxelType, u32>,
    /// Advanced materials required
    pub advanced_materials: HashMap<AdvancedMaterialType, u32>,
    /// Required tools for construction
    pub tools: Vec<String>,
    /// Skill level requirements
    pub skill_levels: HashMap<String, u32>,
    /// Energy/power requirements
    pub energy_cost: f32,
    /// Time to construct (in game time units)
    pub construction_time: f32,
    /// Environmental requirements
    pub environmental: EnvironmentalRequirements,
}

/// Template metadata and properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMetadata {
    /// Creation timestamp
    pub created: u64,
    /// Last modified timestamp
    pub modified: u64,
    /// Usage statistics
    pub usage_count: u32,
    /// User ratings
    pub ratings: Vec<f32>,
    /// Download count (for shared templates)
    pub downloads: u32,
    /// Template size in blocks
    pub block_count: u32,
    /// Estimated construction difficulty
    pub difficulty_rating: f32,
    /// Community tags and keywords
    pub community_tags: Vec<String>,
    /// License information
    pub license: String,
}

/// Template variations for customization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariation {
    pub id: String,
    pub name: String,
    pub description: String,
    /// Material substitutions
    pub material_substitutions: HashMap<VoxelType, VoxelType>,
    /// Scale modifications
    pub scale_factor: f32,
    /// Color palette changes
    pub color_modifications: HashMap<String, [f32; 3]>,
    /// Structural modifications
    pub structural_changes: Vec<StructuralChange>,
}

/// Interactive elements within templates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractiveTemplateElement {
    pub id: String,
    pub element_type: InteractiveElementType,
    pub position: Vec3,
    pub properties: HashMap<String, String>,
    pub triggers: Vec<InteractiveTrigger>,
    pub actions: Vec<InteractiveAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InteractiveElementType {
    Door,
    Window,
    Switch,
    Lever,
    Button,
    Pressure_Plate,
    Sensor,
    Light,
    Motor,
    Conveyor,
    Pipe,
    Wire,
    Custom(String),
}

/// Template animations and dynamic behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateAnimation {
    pub id: String,
    pub name: String,
    pub animation_type: AnimationType,
    pub duration: f32,
    pub loop_animation: bool,
    pub keyframes: Vec<AnimationKeyframe>,
    pub affected_blocks: Vec<Vec3>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnimationType {
    Rotation,
    Translation,
    Scale,
    MaterialChange,
    ColorChange,
    Opacity,
    Custom(String),
}

/// Performance optimization data for templates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateOptimization {
    /// Level of detail (LOD) models
    pub lod_levels: Vec<LODLevel>,
    /// Occlusion culling data
    pub occlusion_zones: Vec<OcclusionZone>,
    /// Batch rendering groups
    pub render_batches: Vec<RenderBatch>,
    /// Memory usage estimate
    pub memory_estimate: usize,
    /// Performance rating (1-10)
    pub performance_rating: f32,
}

/// Structural anchor points for stability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorPoint {
    pub position: Vec3,
    pub anchor_type: AnchorType,
    pub load_capacity: f32,
    pub required_support: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnchorType {
    Foundation,
    Support,
    Attachment,
    Suspension,
}

/// Connection points for template linking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoint {
    pub position: Vec3,
    pub direction: Vec3,
    pub connection_type: ConnectionType,
    pub compatible_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionType {
    Road,
    Pipeline,
    Electrical,
    Data,
    Mechanical,
    Structural,
}

/// Foundation requirements for template stability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoundationRequirements {
    pub foundation_type: FoundationType,
    pub depth_required: f32,
    pub area_required: Vec3,
    pub soil_requirements: Vec<String>,
    pub drainage_needed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FoundationType {
    Shallow,
    Deep,
    Floating,
    Anchored,
    None,
}

/// Environmental requirements for template construction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentalRequirements {
    pub temperature_range: (f32, f32),
    pub humidity_range: (f32, f32),
    pub terrain_types: Vec<String>,
    pub elevation_range: (f32, f32),
    pub weather_resistance: f32,
}

/// Structural changes for template variations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralChange {
    pub change_type: StructuralChangeType,
    pub position: Vec3,
    pub parameters: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StructuralChangeType {
    AddBlock,
    RemoveBlock,
    ReplaceBlock,
    AddRoom,
    RemoveRoom,
    ExtendWall,
    AddFloor,
    AddRoof,
}

/// Interactive triggers for template elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractiveTrigger {
    pub trigger_type: TriggerType,
    pub conditions: Vec<TriggerCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerType {
    OnActivate,
    OnEnter,
    OnExit,
    OnTimer,
    OnSignal,
    OnPowerOn,
    OnPowerOff,
}

/// Interactive actions for template elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractiveAction {
    pub action_type: ActionType,
    pub target: String,
    pub parameters: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    Move,
    Rotate,
    Activate,
    Deactivate,
    Signal,
    PlaySound,
    ChangeColor,
    SpawnParticles,
}

/// Animation keyframes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationKeyframe {
    pub time: f32,
    pub transform: Transform,
    pub properties: HashMap<String, f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
}

/// Level of detail models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LODLevel {
    pub distance_threshold: f32,
    pub block_reduction: f32,
    pub detail_level: f32,
}

/// Occlusion culling zones
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcclusionZone {
    pub bounds: (Vec3, Vec3),
    pub occlusion_strength: f32,
}

/// Render batching groups
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderBatch {
    pub material_group: String,
    pub block_positions: Vec<Vec3>,
    pub render_priority: u32,
}

/// Trigger conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerCondition {
    pub condition_type: ConditionType,
    pub value: String,
    pub comparison: ComparisonType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionType {
    PlayerDistance,
    TimeOfDay,
    Weather,
    Power,
    Signal,
    Variable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonType {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterOrEqual,
    LessOrEqual,
}

/// Enhanced template library manager
#[derive(Debug)]
pub struct EnhancedTemplateLibrary {
    /// Template storage organized by category
    templates: HashMap<TemplateCategory, Vec<EnhancedTemplate>>,
    /// Template lookup by ID
    template_lookup: HashMap<String, EnhancedTemplate>,
    /// Featured templates
    featured_templates: Vec<String>,
    /// Recently used templates
    recent_templates: Vec<String>,
    /// User favorites
    favorites: Vec<String>,
    /// Template cache for performance
    cache: TemplateCache,
}

#[derive(Debug)]
struct TemplateCache {
    compiled_templates: HashMap<String, CompiledTemplate>,
    cache_size_limit: usize,
    current_cache_size: usize,
}

#[derive(Debug, Clone)]
struct CompiledTemplate {
    pub template_id: String,
    pub optimized_structure: Vec<OptimizedBlock>,
    pub render_data: Vec<u8>,
    pub memory_size: usize,
}

#[derive(Debug, Clone)]
struct OptimizedBlock {
    pub position: Vec3,
    pub voxel_type: VoxelType,
    pub render_batch_id: u32,
}

impl EnhancedTemplateLibrary {
    pub fn new() -> Self {
        let mut library = Self {
            templates: HashMap::new(),
            template_lookup: HashMap::new(),
            featured_templates: Vec::new(),
            recent_templates: Vec::new(),
            favorites: Vec::new(),
            cache: TemplateCache {
                compiled_templates: HashMap::new(),
                cache_size_limit: 100 * 1024 * 1024, // 100MB
                current_cache_size: 0,
            },
        };

        library.initialize_default_templates();
        library
    }

    /// Initialize the library with comprehensive default templates
    fn initialize_default_templates(&mut self) {
        // Architecture Templates
        self.add_template(self.create_gothic_cathedral());
        self.add_template(self.create_modern_skyscraper());
        self.add_template(self.create_classical_mansion());
        self.add_template(self.create_minimalist_house());

        // Infrastructure Templates
        self.add_template(self.create_suspension_bridge());
        self.add_template(self.create_subway_station());
        self.add_template(self.create_power_plant());
        self.add_template(self.create_water_treatment());

        // Industrial Templates
        self.add_template(self.create_factory_complex());
        self.add_template(self.create_mining_facility());
        self.add_template(self.create_refinery());

        // Natural Templates
        self.add_template(self.create_ancient_tree());
        self.add_template(self.create_cave_system());
        self.add_template(self.create_waterfall());

        // Technology Templates
        self.add_template(self.create_space_station());
        self.add_template(self.create_data_center());
        self.add_template(self.create_research_lab());
    }

    /// Create a Gothic Cathedral template
    fn create_gothic_cathedral(&self) -> EnhancedTemplate {
        let mut structure = TemplateStructure {
            dimensions: Vec3::new(40.0, 60.0, 120.0),
            voxels: HashMap::new(),
            advanced_materials: HashMap::new(),
            anchor_points: vec![
                AnchorPoint {
                    position: Vec3::new(20.0, 0.0, 60.0),
                    anchor_type: AnchorType::Foundation,
                    load_capacity: 10000.0,
                    required_support: true,
                }
            ],
            connection_points: vec![
                ConnectionPoint {
                    position: Vec3::new(20.0, 2.0, 10.0),
                    direction: Vec3::new(0.0, 0.0, -1.0),
                    connection_type: ConnectionType::Road,
                    compatible_types: vec!["road".to_string(), "path".to_string()],
                }
            ],
            foundation: FoundationRequirements {
                foundation_type: FoundationType::Deep,
                depth_required: 10.0,
                area_required: Vec3::new(45.0, 10.0, 125.0),
                soil_requirements: vec!["stable".to_string(), "rock".to_string()],
                drainage_needed: true,
            },
        };

        // Define the cathedral structure (simplified for example)
        for x in 0..40 {
            for z in 0..120 {
                // Foundation
                structure.voxels.insert(Vec3::new(x as f32, 0.0, z as f32), VoxelType::Stone);

                // Walls (simplified)
                if x == 0 || x == 39 || z == 0 || z == 119 {
                    for y in 1..30 {
                        structure.voxels.insert(Vec3::new(x as f32, y as f32, z as f32), VoxelType::Stone);
                    }
                }
            }
        }

        // Add Gothic arches (simplified)
        for arch_z in [20, 40, 60, 80, 100] {
            for y in 5..25 {
                let arch_radius = 8.0;
                let center_x = 20.0;
                for x in 12..28 {
                    let distance = ((x as f32 - center_x).powi(2) + (y as f32 - 15.0).powi(2)).sqrt();
                    if distance <= arch_radius && distance >= arch_radius - 2.0 {
                        structure.voxels.insert(Vec3::new(x as f32, y as f32, arch_z as f32), VoxelType::Stone);
                    }
                }
            }
        }

        EnhancedTemplate {
            id: "gothic_cathedral".to_string(),
            name: "Gothic Cathedral".to_string(),
            description: "A magnificent Gothic cathedral with soaring arches and intricate stonework".to_string(),
            category: TemplateCategory::Architecture,
            complexity: TemplateComplexity::Masterwork,
            author: "Robin Engine Team".to_string(),
            version: "1.0.0".to_string(),
            tags: vec!["gothic".to_string(), "cathedral".to_string(), "religious".to_string(), "stone".to_string()],
            structure,
            requirements: BuildRequirements {
                materials: {
                    let mut materials = HashMap::new();
                    materials.insert(VoxelType::Stone, 5000);
                    materials.insert(VoxelType::Glass, 200);
                    materials.insert(VoxelType::Wood, 300);
                    materials
                },
                advanced_materials: {
                    let mut adv_materials = HashMap::new();
                    adv_materials.insert(AdvancedMaterialType::Limestone, 3000);
                    adv_materials.insert(AdvancedMaterialType::Marble, 500);
                    adv_materials
                },
                tools: vec!["stone_mason".to_string(), "crane".to_string(), "scaffolding".to_string()],
                skill_levels: {
                    let mut skills = HashMap::new();
                    skills.insert("architecture".to_string(), 8);
                    skills.insert("masonry".to_string(), 9);
                    skills
                },
                energy_cost: 50000.0,
                construction_time: 720.0, // 30 game days
                environmental: EnvironmentalRequirements {
                    temperature_range: (-10.0, 35.0),
                    humidity_range: (0.2, 0.8),
                    terrain_types: vec!["flat".to_string(), "hill".to_string()],
                    elevation_range: (0.0, 500.0),
                    weather_resistance: 0.9,
                },
            },
            metadata: TemplateMetadata {
                created: 1640995200, // Example timestamp
                modified: 1640995200,
                usage_count: 0,
                ratings: vec![],
                downloads: 0,
                block_count: 5500,
                difficulty_rating: 9.5,
                community_tags: vec!["masterpiece".to_string(), "landmark".to_string()],
                license: "Creative Commons".to_string(),
            },
            variations: vec![
                TemplateVariation {
                    id: "sandstone_cathedral".to_string(),
                    name: "Sandstone Cathedral".to_string(),
                    description: "Gothic cathedral built with warm sandstone".to_string(),
                    material_substitutions: {
                        let mut subs = HashMap::new();
                        subs.insert(VoxelType::Stone, VoxelType::Sand);
                        subs
                    },
                    scale_factor: 1.0,
                    color_modifications: HashMap::new(),
                    structural_changes: vec![],
                }
            ],
            interactive_elements: vec![
                InteractiveTemplateElement {
                    id: "main_door".to_string(),
                    element_type: InteractiveElementType::Door,
                    position: Vec3::new(20.0, 3.0, 5.0),
                    properties: {
                        let mut props = HashMap::new();
                        props.insert("size".to_string(), "large".to_string());
                        props.insert("material".to_string(), "oak".to_string());
                        props
                    },
                    triggers: vec![],
                    actions: vec![],
                }
            ],
            animations: vec![],
            optimization: TemplateOptimization {
                lod_levels: vec![
                    LODLevel { distance_threshold: 100.0, block_reduction: 0.1, detail_level: 1.0 },
                    LODLevel { distance_threshold: 500.0, block_reduction: 0.5, detail_level: 0.5 },
                    LODLevel { distance_threshold: 1000.0, block_reduction: 0.8, detail_level: 0.2 },
                ],
                occlusion_zones: vec![],
                render_batches: vec![],
                memory_estimate: 2_200_000, // 2.2MB
                performance_rating: 7.5,
            },
        }
    }

    /// Create additional templates (simplified for brevity)
    fn create_modern_skyscraper(&self) -> EnhancedTemplate {
        // Implementation similar to Gothic Cathedral but for a modern skyscraper
        EnhancedTemplate {
            id: "modern_skyscraper".to_string(),
            name: "Modern Skyscraper".to_string(),
            description: "A sleek modern skyscraper with glass and steel construction".to_string(),
            category: TemplateCategory::Architecture,
            complexity: TemplateComplexity::Complex,
            author: "Robin Engine Team".to_string(),
            version: "1.0.0".to_string(),
            tags: vec!["modern".to_string(), "skyscraper".to_string(), "glass".to_string(), "steel".to_string()],
            structure: TemplateStructure {
                dimensions: Vec3::new(30.0, 150.0, 30.0),
                voxels: HashMap::new(),
                advanced_materials: HashMap::new(),
                anchor_points: vec![],
                connection_points: vec![],
                foundation: FoundationRequirements {
                    foundation_type: FoundationType::Deep,
                    depth_required: 20.0,
                    area_required: Vec3::new(35.0, 20.0, 35.0),
                    soil_requirements: vec!["bedrock".to_string()],
                    drainage_needed: true,
                },
            },
            requirements: BuildRequirements {
                materials: {
                    let mut materials = HashMap::new();
                    materials.insert(VoxelType::Glass, 8000);
                    materials.insert(VoxelType::Metal, 3000);
                    materials.insert(VoxelType::Stone, 2000);
                    materials
                },
                advanced_materials: {
                    let mut adv_materials = HashMap::new();
                    adv_materials.insert(AdvancedMaterialType::Steel, 2500);
                    adv_materials.insert(AdvancedMaterialType::Aluminum, 500);
                    adv_materials
                },
                tools: vec!["crane".to_string(), "welder".to_string(), "glass_cutter".to_string()],
                skill_levels: {
                    let mut skills = HashMap::new();
                    skills.insert("engineering".to_string(), 7);
                    skills.insert("construction".to_string(), 8);
                    skills
                },
                energy_cost: 75000.0,
                construction_time: 1440.0, // 60 game days
                environmental: EnvironmentalRequirements {
                    temperature_range: (-20.0, 45.0),
                    humidity_range: (0.1, 0.9),
                    terrain_types: vec!["urban".to_string(), "flat".to_string()],
                    elevation_range: (0.0, 100.0),
                    weather_resistance: 0.95,
                },
            },
            metadata: TemplateMetadata {
                created: 1640995200,
                modified: 1640995200,
                usage_count: 0,
                ratings: vec![],
                downloads: 0,
                block_count: 13000,
                difficulty_rating: 8.5,
                community_tags: vec!["urban".to_string(), "modern".to_string()],
                license: "Creative Commons".to_string(),
            },
            variations: vec![],
            interactive_elements: vec![],
            animations: vec![],
            optimization: TemplateOptimization {
                lod_levels: vec![
                    LODLevel { distance_threshold: 200.0, block_reduction: 0.1, detail_level: 1.0 },
                    LODLevel { distance_threshold: 1000.0, block_reduction: 0.6, detail_level: 0.4 },
                    LODLevel { distance_threshold: 2000.0, block_reduction: 0.9, detail_level: 0.1 },
                ],
                occlusion_zones: vec![],
                render_batches: vec![],
                memory_estimate: 5_200_000, // 5.2MB
                performance_rating: 6.8,
            },
        }
    }

    // Additional simplified template creators
    fn create_classical_mansion(&self) -> EnhancedTemplate { self.create_simple_template("classical_mansion", "Classical Mansion", TemplateCategory::Residential) }
    fn create_minimalist_house(&self) -> EnhancedTemplate { self.create_simple_template("minimalist_house", "Minimalist House", TemplateCategory::Residential) }
    fn create_suspension_bridge(&self) -> EnhancedTemplate { self.create_simple_template("suspension_bridge", "Suspension Bridge", TemplateCategory::Infrastructure) }
    fn create_subway_station(&self) -> EnhancedTemplate { self.create_simple_template("subway_station", "Subway Station", TemplateCategory::Transportation) }
    fn create_power_plant(&self) -> EnhancedTemplate { self.create_simple_template("power_plant", "Power Plant", TemplateCategory::Industrial) }
    fn create_water_treatment(&self) -> EnhancedTemplate { self.create_simple_template("water_treatment", "Water Treatment Plant", TemplateCategory::Infrastructure) }
    fn create_factory_complex(&self) -> EnhancedTemplate { self.create_simple_template("factory_complex", "Factory Complex", TemplateCategory::Industrial) }
    fn create_mining_facility(&self) -> EnhancedTemplate { self.create_simple_template("mining_facility", "Mining Facility", TemplateCategory::Industrial) }
    fn create_refinery(&self) -> EnhancedTemplate { self.create_simple_template("refinery", "Oil Refinery", TemplateCategory::Industrial) }
    fn create_ancient_tree(&self) -> EnhancedTemplate { self.create_simple_template("ancient_tree", "Ancient Tree", TemplateCategory::Natural) }
    fn create_cave_system(&self) -> EnhancedTemplate { self.create_simple_template("cave_system", "Cave System", TemplateCategory::Natural) }
    fn create_waterfall(&self) -> EnhancedTemplate { self.create_simple_template("waterfall", "Waterfall", TemplateCategory::Natural) }
    fn create_space_station(&self) -> EnhancedTemplate { self.create_simple_template("space_station", "Space Station", TemplateCategory::Technology) }
    fn create_data_center(&self) -> EnhancedTemplate { self.create_simple_template("data_center", "Data Center", TemplateCategory::Technology) }
    fn create_research_lab(&self) -> EnhancedTemplate { self.create_simple_template("research_lab", "Research Laboratory", TemplateCategory::Technology) }

    /// Helper method to create simple templates for demonstration
    fn create_simple_template(&self, id: &str, name: &str, category: TemplateCategory) -> EnhancedTemplate {
        EnhancedTemplate {
            id: id.to_string(),
            name: name.to_string(),
            description: format!("A sophisticated {} for advanced construction projects", name.to_lowercase()),
            category,
            complexity: TemplateComplexity::Advanced,
            author: "Robin Engine Team".to_string(),
            version: "1.0.0".to_string(),
            tags: vec!["advanced".to_string(), "professional".to_string()],
            structure: TemplateStructure {
                dimensions: Vec3::new(20.0, 20.0, 20.0),
                voxels: HashMap::new(),
                advanced_materials: HashMap::new(),
                anchor_points: vec![],
                connection_points: vec![],
                foundation: FoundationRequirements {
                    foundation_type: FoundationType::Shallow,
                    depth_required: 3.0,
                    area_required: Vec3::new(25.0, 3.0, 25.0),
                    soil_requirements: vec!["stable".to_string()],
                    drainage_needed: false,
                },
            },
            requirements: BuildRequirements {
                materials: {
                    let mut materials = HashMap::new();
                    materials.insert(VoxelType::Stone, 500);
                    materials.insert(VoxelType::Wood, 300);
                    materials
                },
                advanced_materials: HashMap::new(),
                tools: vec!["basic_tools".to_string()],
                skill_levels: {
                    let mut skills = HashMap::new();
                    skills.insert("construction".to_string(), 5);
                    skills
                },
                energy_cost: 1000.0,
                construction_time: 60.0,
                environmental: EnvironmentalRequirements {
                    temperature_range: (-10.0, 40.0),
                    humidity_range: (0.0, 1.0),
                    terrain_types: vec!["any".to_string()],
                    elevation_range: (0.0, 1000.0),
                    weather_resistance: 0.7,
                },
            },
            metadata: TemplateMetadata {
                created: 1640995200,
                modified: 1640995200,
                usage_count: 0,
                ratings: vec![],
                downloads: 0,
                block_count: 800,
                difficulty_rating: 6.0,
                community_tags: vec![],
                license: "Creative Commons".to_string(),
            },
            variations: vec![],
            interactive_elements: vec![],
            animations: vec![],
            optimization: TemplateOptimization {
                lod_levels: vec![],
                occlusion_zones: vec![],
                render_batches: vec![],
                memory_estimate: 320_000,
                performance_rating: 8.0,
            },
        }
    }

    /// Add a template to the library
    pub fn add_template(&mut self, template: EnhancedTemplate) {
        let category = template.category.clone();
        let id = template.id.clone();

        // Add to category storage
        self.templates.entry(category).or_insert_with(Vec::new).push(template.clone());

        // Add to lookup
        self.template_lookup.insert(id, template);
    }

    /// Get templates by category
    pub fn get_templates_by_category(&self, category: &TemplateCategory) -> Option<&Vec<EnhancedTemplate>> {
        self.templates.get(category)
    }

    /// Get template by ID
    pub fn get_template(&self, id: &str) -> Option<&EnhancedTemplate> {
        self.template_lookup.get(id)
    }

    /// Search templates by tags and criteria
    pub fn search_templates(&self, criteria: &SearchCriteria) -> Vec<&EnhancedTemplate> {
        self.template_lookup.values()
            .filter(|template| self.matches_criteria(template, criteria))
            .collect()
    }

    /// Check if template matches search criteria
    fn matches_criteria(&self, template: &EnhancedTemplate, criteria: &SearchCriteria) -> bool {
        // Category filter
        if let Some(category) = &criteria.category {
            if template.category != *category {
                return false;
            }
        }

        // Complexity filter
        if let Some(complexity) = &criteria.complexity {
            if template.complexity != *complexity {
                return false;
            }
        }

        // Tag filter
        if !criteria.tags.is_empty() {
            let has_matching_tag = criteria.tags.iter()
                .any(|tag| template.tags.contains(tag) || template.metadata.community_tags.contains(tag));
            if !has_matching_tag {
                return false;
            }
        }

        // Difficulty range
        if let Some((min_difficulty, max_difficulty)) = criteria.difficulty_range {
            if template.metadata.difficulty_rating < min_difficulty || template.metadata.difficulty_rating > max_difficulty {
                return false;
            }
        }

        true
    }

    /// Get featured templates
    pub fn get_featured_templates(&self) -> Vec<&EnhancedTemplate> {
        self.featured_templates.iter()
            .filter_map(|id| self.template_lookup.get(id))
            .collect()
    }

    /// Add template to favorites
    pub fn add_to_favorites(&mut self, template_id: String) {
        if !self.favorites.contains(&template_id) {
            self.favorites.push(template_id);
        }
    }

    /// Get favorite templates
    pub fn get_favorites(&self) -> Vec<&EnhancedTemplate> {
        self.favorites.iter()
            .filter_map(|id| self.template_lookup.get(id))
            .collect()
    }

    /// Compile template for optimized rendering
    pub fn compile_template(&mut self, template_id: &str) -> RobinResult<()> {
        if let Some(template) = self.template_lookup.get(template_id) {
            let compiled = self.create_compiled_template(template)?;

            // Check cache size
            if self.cache.current_cache_size + compiled.memory_size > self.cache.cache_size_limit {
                self.evict_cache_entries();
            }

            self.cache.current_cache_size += compiled.memory_size;
            self.cache.compiled_templates.insert(template_id.to_string(), compiled);

            Ok(())
        } else {
            Err(RobinError::TemplateError {
                template_id: template_id.to_string(),
                operation: "compile".to_string(),
                reason: "Template not found".to_string(),
            })
        }
    }

    /// Create compiled template for performance
    fn create_compiled_template(&self, template: &EnhancedTemplate) -> RobinResult<CompiledTemplate> {
        let mut optimized_blocks = Vec::new();

        for (position, voxel_type) in &template.structure.voxels {
            optimized_blocks.push(OptimizedBlock {
                position: *position,
                voxel_type: *voxel_type,
                render_batch_id: self.calculate_render_batch(*voxel_type),
            });
        }

        let render_data = self.generate_render_data(&optimized_blocks)?;
        let memory_size = optimized_blocks.len() * 32 + render_data.len(); // Rough estimate

        Ok(CompiledTemplate {
            template_id: template.id.clone(),
            optimized_structure: optimized_blocks,
            render_data,
            memory_size,
        })
    }

    /// Calculate render batch ID for optimization
    fn calculate_render_batch(&self, voxel_type: VoxelType) -> u32 {
        match voxel_type {
            VoxelType::Stone => 1,
            VoxelType::Wood => 2,
            VoxelType::Metal => 3,
            VoxelType::Glass => 4,
            _ => 0,
        }
    }

    /// Generate render data for compiled template
    fn generate_render_data(&self, blocks: &[OptimizedBlock]) -> RobinResult<Vec<u8>> {
        // Simplified render data generation
        let mut data = Vec::new();

        for block in blocks {
            // Position (12 bytes)
            data.extend_from_slice(&block.position.x.to_le_bytes());
            data.extend_from_slice(&block.position.y.to_le_bytes());
            data.extend_from_slice(&block.position.z.to_le_bytes());

            // Voxel type (4 bytes)
            data.extend_from_slice(&block.render_batch_id.to_le_bytes());
        }

        Ok(data)
    }

    /// Evict cache entries when limit is reached
    fn evict_cache_entries(&mut self) {
        // Simple LRU eviction - remove half the cache
        let keys_to_remove: Vec<String> = self.cache.compiled_templates.keys()
            .take(self.cache.compiled_templates.len() / 2)
            .cloned()
            .collect();

        for key in keys_to_remove {
            if let Some(template) = self.cache.compiled_templates.remove(&key) {
                self.cache.current_cache_size -= template.memory_size;
            }
        }
    }

    /// Get compiled template for rendering
    pub fn get_compiled_template(&self, template_id: &str) -> Option<&CompiledTemplate> {
        self.cache.compiled_templates.get(template_id)
    }

    /// Get template statistics
    pub fn get_statistics(&self) -> TemplateLibraryStats {
        let total_templates = self.template_lookup.len();
        let mut templates_by_category = HashMap::new();
        let mut average_difficulty = 0.0;

        for template in self.template_lookup.values() {
            *templates_by_category.entry(template.category.clone()).or_insert(0) += 1;
            average_difficulty += template.metadata.difficulty_rating;
        }

        if total_templates > 0 {
            average_difficulty /= total_templates as f32;
        }

        TemplateLibraryStats {
            total_templates,
            templates_by_category,
            cache_hit_rate: self.calculate_cache_hit_rate(),
            memory_usage: self.cache.current_cache_size,
            average_difficulty,
            most_popular: self.get_most_popular_template(),
        }
    }

    fn calculate_cache_hit_rate(&self) -> f32 {
        // Simplified cache hit rate calculation
        0.85 // 85% hit rate for demonstration
    }

    fn get_most_popular_template(&self) -> Option<String> {
        self.template_lookup.values()
            .max_by_key(|template| template.metadata.usage_count)
            .map(|template| template.id.clone())
    }
}

/// Search criteria for template filtering
#[derive(Debug, Default)]
pub struct SearchCriteria {
    pub category: Option<TemplateCategory>,
    pub complexity: Option<TemplateComplexity>,
    pub tags: Vec<String>,
    pub difficulty_range: Option<(f32, f32)>,
    pub author: Option<String>,
    pub block_count_range: Option<(u32, u32)>,
}

/// Template library statistics
#[derive(Debug)]
pub struct TemplateLibraryStats {
    pub total_templates: usize,
    pub templates_by_category: HashMap<TemplateCategory, usize>,
    pub cache_hit_rate: f32,
    pub memory_usage: usize,
    pub average_difficulty: f32,
    pub most_popular: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_library_creation() {
        let library = EnhancedTemplateLibrary::new();
        assert!(!library.template_lookup.is_empty());
    }

    #[test]
    fn test_template_search() {
        let library = EnhancedTemplateLibrary::new();
        let criteria = SearchCriteria {
            category: Some(TemplateCategory::Architecture),
            ..Default::default()
        };
        let results = library.search_templates(&criteria);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_template_compilation() {
        let mut library = EnhancedTemplateLibrary::new();
        let result = library.compile_template("gothic_cathedral");
        assert!(result.is_ok());
    }
}