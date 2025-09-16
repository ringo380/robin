use nalgebra::{Vector3, Point3, Matrix4, UnitQuaternion};
use std::collections::HashMap;
use super::{Material, MaterialType, Construction, ConstructionType};

pub struct BlueprintSystem {
    // Blueprint library
    pub blueprints: HashMap<String, Blueprint>,
    pub categories: HashMap<String, Vec<String>>,
    
    // Active blueprint
    pub active_blueprint: Option<String>,
    pub blueprint_position: Point3<f32>,
    pub blueprint_rotation: UnitQuaternion<f32>,
    pub blueprint_scale: Vector3<f32>,
    
    // Blueprint creation
    pub creation_mode: BlueprintCreationMode,
    pub selection_bounds: Option<(Point3<i32>, Point3<i32>)>,
    pub captured_blocks: Vec<CapturedBlock>,
    
    // Smart features
    pub auto_materials: bool,
    pub structural_validation: bool,
    pub cost_estimation: bool,
    pub progress_tracking: bool,
    
    // Construction queue
    pub construction_queue: Vec<QueuedConstruction>,
    pub auto_build: bool,
    pub build_speed: f32,
    
    // Version control
    pub blueprint_versions: HashMap<String, Vec<BlueprintVersion>>,
    pub version_history: Vec<VersionChange>,
}

#[derive(Clone, Debug)]
pub struct Blueprint {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub category: String,
    pub tags: Vec<String>,
    
    // Structure data
    pub blocks: Vec<BlueprintBlock>,
    pub size: Vector3<i32>,
    pub center_point: Vector3<i32>,
    pub anchor_points: Vec<AnchorPoint>,
    
    // Requirements
    pub material_requirements: HashMap<String, u32>,
    pub tool_requirements: Vec<String>,
    pub skill_requirements: HashMap<String, u32>,
    pub energy_cost: f32,
    pub build_time: f32,
    
    // Metadata
    pub difficulty: f32,
    pub complexity: f32,
    pub structural_rating: f32,
    pub aesthetic_rating: f32,
    pub functionality_rating: f32,
    
    // Construction data
    pub build_order: Vec<BuildStep>,
    pub alternative_materials: HashMap<String, Vec<String>>,
    pub optional_components: Vec<String>,
    
    // Visual data
    pub thumbnail: Option<String>,
    pub preview_images: Vec<String>,
    pub schematic_views: Vec<SchematicView>,
    
    created_at: u64,
    modified_at: u64,
}

#[derive(Clone, Debug)]
pub struct BlueprintBlock {
    pub position: Vector3<i32>,
    pub material: String,
    pub rotation: UnitQuaternion<f32>,
    pub scale: Vector3<f32>,
    pub priority: u32,
    pub dependencies: Vec<Vector3<i32>>,
    pub properties: BlockProperties,
}

#[derive(Clone, Debug)]
pub struct BlockProperties {
    pub structural: bool,
    pub decorative: bool,
    pub functional: bool,
    pub replaceable: bool,
    pub custom_data: HashMap<String, String>,
}

#[derive(Clone, Debug)]
pub struct AnchorPoint {
    pub name: String,
    pub position: Vector3<i32>,
    pub anchor_type: AnchorType,
    pub connection_points: Vec<Vector3<i32>>,
}

#[derive(Clone, Debug)]
pub enum AnchorType {
    Foundation,
    Connection,
    Support,
    Entrance,
    Utility,
    Decoration,
}

#[derive(Clone, Debug)]
pub struct BuildStep {
    pub step_id: u32,
    pub description: String,
    pub blocks: Vec<Vector3<i32>>,
    pub required_tools: Vec<String>,
    pub estimated_time: f32,
    pub prerequisites: Vec<u32>,
}

#[derive(Clone, Debug)]
pub struct SchematicView {
    pub name: String,
    pub view_type: ViewType,
    pub data: Vec<u8>, // Compressed schematic data
}

#[derive(Clone, Debug)]
pub enum ViewType {
    TopDown,
    FrontView,
    SideView,
    Perspective,
    CrossSection,
    Wireframe,
}

#[derive(Clone, Debug)]
pub enum BlueprintCreationMode {
    SelectArea,
    CopyStructure,
    DesignNew,
    ModifyExisting,
    ImportFile,
}

#[derive(Clone, Debug)]
pub struct CapturedBlock {
    pub position: Vector3<i32>,
    pub material: String,
    pub properties: BlockProperties,
}

#[derive(Clone, Debug)]
pub struct QueuedConstruction {
    pub blueprint_id: String,
    pub position: Point3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub scale: Vector3<f32>,
    pub priority: u32,
    pub start_time: Option<u64>,
    pub estimated_completion: u64,
    pub assigned_engineer: String,
    pub status: ConstructionStatus,
    pub progress: f32,
}

#[derive(Clone, Debug)]
pub enum ConstructionStatus {
    Queued,
    InProgress,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug)]
pub struct BlueprintVersion {
    pub version: String,
    pub changes: String,
    pub timestamp: u64,
    pub author: String,
    pub blueprint_data: Blueprint,
}

#[derive(Clone, Debug)]
pub struct VersionChange {
    pub blueprint_id: String,
    pub old_version: String,
    pub new_version: String,
    pub change_type: ChangeType,
    pub description: String,
    pub timestamp: u64,
}

#[derive(Clone, Debug)]
pub enum ChangeType {
    Created,
    Modified,
    Deleted,
    Forked,
    Merged,
}

impl BlueprintSystem {
    pub fn new() -> Self {
        let mut system = Self {
            blueprints: HashMap::new(),
            categories: HashMap::new(),
            
            active_blueprint: None,
            blueprint_position: Point3::new(0.0, 0.0, 0.0),
            blueprint_rotation: UnitQuaternion::identity(),
            blueprint_scale: Vector3::new(1.0, 1.0, 1.0),
            
            creation_mode: BlueprintCreationMode::SelectArea,
            selection_bounds: None,
            captured_blocks: Vec::new(),
            
            auto_materials: true,
            structural_validation: true,
            cost_estimation: true,
            progress_tracking: true,
            
            construction_queue: Vec::new(),
            auto_build: false,
            build_speed: 1.0,
            
            blueprint_versions: HashMap::new(),
            version_history: Vec::new(),
        };
        
        system.initialize_default_blueprints();
        system.initialize_categories();
        system
    }

    fn initialize_default_blueprints(&mut self) {
        // Simple house blueprint
        let house_blueprint = Blueprint {
            id: "simple_house".to_string(),
            name: "Simple House".to_string(),
            description: "A basic wooden house with foundation".to_string(),
            author: "system".to_string(),
            version: "1.0".to_string(),
            category: "residential".to_string(),
            tags: vec!["house".to_string(), "basic".to_string(), "wood".to_string()],
            
            blocks: self.generate_house_blocks(),
            size: Vector3::new(8, 6, 8),
            center_point: Vector3::new(4, 0, 4),
            anchor_points: vec![
                AnchorPoint {
                    name: "foundation".to_string(),
                    position: Vector3::new(4, 0, 4),
                    anchor_type: AnchorType::Foundation,
                    connection_points: vec![],
                },
                AnchorPoint {
                    name: "entrance".to_string(),
                    position: Vector3::new(4, 1, 0),
                    anchor_type: AnchorType::Entrance,
                    connection_points: vec![],
                },
            ],
            
            material_requirements: {
                let mut map = HashMap::new();
                map.insert("wood".to_string(), 150);
                map.insert("stone".to_string(), 64);
                map.insert("glass".to_string(), 16);
                map
            },
            tool_requirements: vec!["placer".to_string()],
            skill_requirements: HashMap::new(),
            energy_cost: 200.0,
            build_time: 300.0,
            
            difficulty: 2.0,
            complexity: 3.0,
            structural_rating: 0.8,
            aesthetic_rating: 0.6,
            functionality_rating: 0.7,
            
            build_order: self.generate_house_build_order(),
            alternative_materials: {
                let mut map = HashMap::new();
                map.insert("wood".to_string(), vec!["stone".to_string(), "metal".to_string()]);
                map
            },
            optional_components: vec!["chimney".to_string(), "garden".to_string()],
            
            thumbnail: None,
            preview_images: vec![],
            schematic_views: vec![],
            
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            modified_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        self.blueprints.insert("simple_house".to_string(), house_blueprint);
        
        // Bridge blueprint
        let bridge_blueprint = self.create_bridge_blueprint();
        self.blueprints.insert("wooden_bridge".to_string(), bridge_blueprint);
        
        // Tower blueprint
        let tower_blueprint = self.create_tower_blueprint();
        self.blueprints.insert("stone_tower".to_string(), tower_blueprint);
    }

    fn initialize_categories(&mut self) {
        let categories = vec![
            ("residential", vec!["simple_house"]),
            ("infrastructure", vec!["wooden_bridge"]),
            ("defense", vec!["stone_tower"]),
            ("industrial", vec![]),
            ("decoration", vec![]),
            ("utility", vec![]),
        ];
        
        for (category, blueprints) in categories {
            self.categories.insert(
                category.to_string(), 
                blueprints.iter().map(|s| s.to_string()).collect()
            );
        }
    }

    fn generate_house_blocks(&self) -> Vec<BlueprintBlock> {
        let mut blocks = Vec::new();
        
        // Foundation (8x8 stone)
        for x in 0..8 {
            for z in 0..8 {
                blocks.push(BlueprintBlock {
                    position: Vector3::new(x, 0, z),
                    material: "stone".to_string(),
                    rotation: UnitQuaternion::identity(),
                    scale: Vector3::new(1.0, 1.0, 1.0),
                    priority: 1,
                    dependencies: vec![],
                    properties: BlockProperties {
                        structural: true,
                        decorative: false,
                        functional: true,
                        replaceable: false,
                        custom_data: HashMap::new(),
                    },
                });
            }
        }
        
        // Walls (wood frame)
        for y in 1..5 {
            // Front and back walls
            for x in 0..8 {
                if y == 1 && x == 4 {
                    continue; // Door opening
                }
                
                blocks.push(BlueprintBlock {
                    position: Vector3::new(x, y, 0),
                    material: "wood".to_string(),
                    rotation: UnitQuaternion::identity(),
                    scale: Vector3::new(1.0, 1.0, 1.0),
                    priority: 2,
                    dependencies: vec![Vector3::new(x, 0, 0)],
                    properties: BlockProperties {
                        structural: true,
                        decorative: false,
                        functional: true,
                        replaceable: true,
                        custom_data: HashMap::new(),
                    },
                });
                
                blocks.push(BlueprintBlock {
                    position: Vector3::new(x, y, 7),
                    material: "wood".to_string(),
                    rotation: UnitQuaternion::identity(),
                    scale: Vector3::new(1.0, 1.0, 1.0),
                    priority: 2,
                    dependencies: vec![Vector3::new(x, 0, 7)],
                    properties: BlockProperties {
                        structural: true,
                        decorative: false,
                        functional: true,
                        replaceable: true,
                        custom_data: HashMap::new(),
                    },
                });
            }
            
            // Side walls
            for z in 1..7 {
                if y == 2 && (z == 2 || z == 5) {
                    // Windows
                    blocks.push(BlueprintBlock {
                        position: Vector3::new(0, y, z),
                        material: "glass".to_string(),
                        rotation: UnitQuaternion::identity(),
                        scale: Vector3::new(1.0, 1.0, 1.0),
                        priority: 3,
                        dependencies: vec![Vector3::new(0, 0, z)],
                        properties: BlockProperties {
                            structural: false,
                            decorative: true,
                            functional: true,
                            replaceable: true,
                            custom_data: HashMap::new(),
                        },
                    });
                    
                    blocks.push(BlueprintBlock {
                        position: Vector3::new(7, y, z),
                        material: "glass".to_string(),
                        rotation: UnitQuaternion::identity(),
                        scale: Vector3::new(1.0, 1.0, 1.0),
                        priority: 3,
                        dependencies: vec![Vector3::new(7, 0, z)],
                        properties: BlockProperties {
                            structural: false,
                            decorative: true,
                            functional: true,
                            replaceable: true,
                            custom_data: HashMap::new(),
                        },
                    });
                } else {
                    blocks.push(BlueprintBlock {
                        position: Vector3::new(0, y, z),
                        material: "wood".to_string(),
                        rotation: UnitQuaternion::identity(),
                        scale: Vector3::new(1.0, 1.0, 1.0),
                        priority: 2,
                        dependencies: vec![Vector3::new(0, 0, z)],
                        properties: BlockProperties {
                            structural: true,
                            decorative: false,
                            functional: true,
                            replaceable: true,
                            custom_data: HashMap::new(),
                        },
                    });
                    
                    blocks.push(BlueprintBlock {
                        position: Vector3::new(7, y, z),
                        material: "wood".to_string(),
                        rotation: UnitQuaternion::identity(),
                        scale: Vector3::new(1.0, 1.0, 1.0),
                        priority: 2,
                        dependencies: vec![Vector3::new(7, 0, z)],
                        properties: BlockProperties {
                            structural: true,
                            decorative: false,
                            functional: true,
                            replaceable: true,
                            custom_data: HashMap::new(),
                        },
                    });
                }
            }
        }
        
        // Roof (wood)
        for x in 0..8 {
            for z in 0..8 {
                blocks.push(BlueprintBlock {
                    position: Vector3::new(x, 5, z),
                    material: "wood".to_string(),
                    rotation: UnitQuaternion::identity(),
                    scale: Vector3::new(1.0, 1.0, 1.0),
                    priority: 4,
                    dependencies: vec![Vector3::new(x, 4, z)],
                    properties: BlockProperties {
                        structural: false,
                        decorative: false,
                        functional: true,
                        replaceable: true,
                        custom_data: HashMap::new(),
                    },
                });
            }
        }
        
        blocks
    }

    fn generate_house_build_order(&self) -> Vec<BuildStep> {
        vec![
            BuildStep {
                step_id: 1,
                description: "Lay foundation".to_string(),
                blocks: (0..64).map(|i| Vector3::new(i % 8, 0, i / 8)).collect(),
                required_tools: vec!["placer".to_string()],
                estimated_time: 60.0,
                prerequisites: vec![],
            },
            BuildStep {
                step_id: 2,
                description: "Build walls".to_string(),
                blocks: vec![], // Would list all wall blocks
                required_tools: vec!["placer".to_string()],
                estimated_time: 120.0,
                prerequisites: vec![1],
            },
            BuildStep {
                step_id: 3,
                description: "Install windows".to_string(),
                blocks: vec![],
                required_tools: vec!["placer".to_string()],
                estimated_time: 30.0,
                prerequisites: vec![2],
            },
            BuildStep {
                step_id: 4,
                description: "Build roof".to_string(),
                blocks: vec![],
                required_tools: vec!["placer".to_string()],
                estimated_time: 60.0,
                prerequisites: vec![2],
            },
        ]
    }

    fn create_bridge_blueprint(&self) -> Blueprint {
        Blueprint {
            id: "wooden_bridge".to_string(),
            name: "Wooden Bridge".to_string(),
            description: "A sturdy wooden bridge for crossing gaps".to_string(),
            author: "system".to_string(),
            version: "1.0".to_string(),
            category: "infrastructure".to_string(),
            tags: vec!["bridge".to_string(), "wood".to_string(), "transport".to_string()],
            
            blocks: self.generate_bridge_blocks(),
            size: Vector3::new(12, 3, 3),
            center_point: Vector3::new(6, 1, 1),
            anchor_points: vec![
                AnchorPoint {
                    name: "start".to_string(),
                    position: Vector3::new(0, 1, 1),
                    anchor_type: AnchorType::Connection,
                    connection_points: vec![],
                },
                AnchorPoint {
                    name: "end".to_string(),
                    position: Vector3::new(11, 1, 1),
                    anchor_type: AnchorType::Connection,
                    connection_points: vec![],
                },
            ],
            
            material_requirements: {
                let mut map = HashMap::new();
                map.insert("wood".to_string(), 60);
                map.insert("stone".to_string(), 12);
                map
            },
            tool_requirements: vec!["placer".to_string()],
            skill_requirements: HashMap::new(),
            energy_cost: 80.0,
            build_time: 120.0,
            
            difficulty: 3.0,
            complexity: 2.0,
            structural_rating: 0.9,
            aesthetic_rating: 0.5,
            functionality_rating: 0.9,
            
            build_order: vec![],
            alternative_materials: HashMap::new(),
            optional_components: vec!["railings".to_string()],
            
            thumbnail: None,
            preview_images: vec![],
            schematic_views: vec![],
            
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            modified_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    fn generate_bridge_blocks(&self) -> Vec<BlueprintBlock> {
        let mut blocks = Vec::new();
        
        // Bridge deck
        for x in 0..12 {
            blocks.push(BlueprintBlock {
                position: Vector3::new(x, 1, 1),
                material: "wood".to_string(),
                rotation: UnitQuaternion::identity(),
                scale: Vector3::new(1.0, 1.0, 1.0),
                priority: 1,
                dependencies: vec![],
                properties: BlockProperties {
                    structural: true,
                    decorative: false,
                    functional: true,
                    replaceable: true,
                    custom_data: HashMap::new(),
                },
            });
        }
        
        // Support pillars every 3 blocks
        for x in (0..12).step_by(3) {
            blocks.push(BlueprintBlock {
                position: Vector3::new(x, 0, 1),
                material: "stone".to_string(),
                rotation: UnitQuaternion::identity(),
                scale: Vector3::new(1.0, 1.0, 1.0),
                priority: 0,
                dependencies: vec![],
                properties: BlockProperties {
                    structural: true,
                    decorative: false,
                    functional: true,
                    replaceable: false,
                    custom_data: HashMap::new(),
                },
            });
        }
        
        // Railings
        for x in 0..12 {
            blocks.push(BlueprintBlock {
                position: Vector3::new(x, 2, 0),
                material: "wood".to_string(),
                rotation: UnitQuaternion::identity(),
                scale: Vector3::new(1.0, 1.0, 1.0),
                priority: 2,
                dependencies: vec![Vector3::new(x, 1, 1)],
                properties: BlockProperties {
                    structural: false,
                    decorative: true,
                    functional: true,
                    replaceable: true,
                    custom_data: HashMap::new(),
                },
            });
            
            blocks.push(BlueprintBlock {
                position: Vector3::new(x, 2, 2),
                material: "wood".to_string(),
                rotation: UnitQuaternion::identity(),
                scale: Vector3::new(1.0, 1.0, 1.0),
                priority: 2,
                dependencies: vec![Vector3::new(x, 1, 1)],
                properties: BlockProperties {
                    structural: false,
                    decorative: true,
                    functional: true,
                    replaceable: true,
                    custom_data: HashMap::new(),
                },
            });
        }
        
        blocks
    }

    fn create_tower_blueprint(&self) -> Blueprint {
        Blueprint {
            id: "stone_tower".to_string(),
            name: "Stone Tower".to_string(),
            description: "A defensive stone tower with multiple levels".to_string(),
            author: "system".to_string(),
            version: "1.0".to_string(),
            category: "defense".to_string(),
            tags: vec!["tower".to_string(), "stone".to_string(), "defense".to_string()],
            
            blocks: self.generate_tower_blocks(),
            size: Vector3::new(5, 12, 5),
            center_point: Vector3::new(2, 6, 2),
            anchor_points: vec![
                AnchorPoint {
                    name: "base".to_string(),
                    position: Vector3::new(2, 0, 2),
                    anchor_type: AnchorType::Foundation,
                    connection_points: vec![],
                },
                AnchorPoint {
                    name: "entrance".to_string(),
                    position: Vector3::new(2, 1, 0),
                    anchor_type: AnchorType::Entrance,
                    connection_points: vec![],
                },
            ],
            
            material_requirements: {
                let mut map = HashMap::new();
                map.insert("stone".to_string(), 200);
                map.insert("wood".to_string(), 40);
                map
            },
            tool_requirements: vec!["placer".to_string()],
            skill_requirements: HashMap::new(),
            energy_cost: 300.0,
            build_time: 400.0,
            
            difficulty: 4.0,
            complexity: 4.0,
            structural_rating: 0.95,
            aesthetic_rating: 0.7,
            functionality_rating: 0.8,
            
            build_order: vec![],
            alternative_materials: HashMap::new(),
            optional_components: vec!["battlements".to_string(), "spiral_stairs".to_string()],
            
            thumbnail: None,
            preview_images: vec![],
            schematic_views: vec![],
            
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            modified_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    fn generate_tower_blocks(&self) -> Vec<BlueprintBlock> {
        let mut blocks = Vec::new();
        
        // Foundation and walls
        for y in 0..10 {
            for x in 0..5 {
                for z in 0..5 {
                    // Hollow interior except for foundation
                    if y == 0 || x == 0 || x == 4 || z == 0 || z == 4 {
                        let is_entrance = y == 1 && x == 2 && z == 0;
                        if !is_entrance {
                            blocks.push(BlueprintBlock {
                                position: Vector3::new(x, y, z),
                                material: "stone".to_string(),
                                rotation: UnitQuaternion::identity(),
                                scale: Vector3::new(1.0, 1.0, 1.0),
                                priority: if y == 0 { 1 } else { 2 },
                                dependencies: if y > 0 { vec![Vector3::new(x, y-1, z)] } else { vec![] },
                                properties: BlockProperties {
                                    structural: true,
                                    decorative: false,
                                    functional: true,
                                    replaceable: false,
                                    custom_data: HashMap::new(),
                                },
                            });
                        }
                    }
                }
            }
        }
        
        // Floors every 3 levels
        for y in (3..10).step_by(3) {
            for x in 1..4 {
                for z in 1..4 {
                    blocks.push(BlueprintBlock {
                        position: Vector3::new(x, y, z),
                        material: "wood".to_string(),
                        rotation: UnitQuaternion::identity(),
                        scale: Vector3::new(1.0, 1.0, 1.0),
                        priority: 3,
                        dependencies: vec![],
                        properties: BlockProperties {
                            structural: false,
                            decorative: false,
                            functional: true,
                            replaceable: true,
                            custom_data: HashMap::new(),
                        },
                    });
                }
            }
        }
        
        // Battlements on top
        for x in 0..5 {
            for z in 0..5 {
                if x == 0 || x == 4 || z == 0 || z == 4 {
                    if (x + z) % 2 == 0 { // Crenellations
                        blocks.push(BlueprintBlock {
                            position: Vector3::new(x, 11, z),
                            material: "stone".to_string(),
                            rotation: UnitQuaternion::identity(),
                            scale: Vector3::new(1.0, 1.0, 1.0),
                            priority: 4,
                            dependencies: vec![Vector3::new(x, 10, z)],
                            properties: BlockProperties {
                                structural: false,
                                decorative: true,
                                functional: true,
                                replaceable: true,
                                custom_data: HashMap::new(),
                            },
                        });
                    }
                }
            }
        }
        
        blocks
    }

    pub fn update(&mut self, delta_time: f32) {
        // Update construction queue
        self.update_construction_queue(delta_time);
        
        // Auto-build processing
        if self.auto_build {
            self.process_auto_build(delta_time);
        }
        
        // Update active blueprint preview
        if self.active_blueprint.is_some() {
            // Update blueprint positioning
        }
    }

    fn update_construction_queue(&mut self, _delta_time: f32) {
        // Update progress on active constructions
        for construction in &mut self.construction_queue {
            if matches!(construction.status, ConstructionStatus::InProgress) {
                // Update progress based on build speed
                // This would integrate with the placement system
            }
        }
    }

    fn process_auto_build(&mut self, delta_time: f32) {
        // Automatically progress queued constructions
        for construction in &mut self.construction_queue {
            if matches!(construction.status, ConstructionStatus::Queued) {
                construction.status = ConstructionStatus::InProgress;
                construction.start_time = Some(std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs());
            }
            
            if matches!(construction.status, ConstructionStatus::InProgress) {
                construction.progress += self.build_speed * delta_time / 100.0;
                construction.progress = construction.progress.clamp(0.0, 1.0);
                
                if construction.progress >= 1.0 {
                    construction.status = ConstructionStatus::Completed;
                }
            }
        }
    }

    pub fn create_blueprint_from_selection(&mut self, bounds: (Point3<i32>, Point3<i32>), name: &str, author: &str) -> Result<String, String> {
        let (min_bound, max_bound) = bounds;
        let size = max_bound - min_bound + Vector3::new(1, 1, 1);
        
        if self.captured_blocks.is_empty() {
            return Err("No blocks captured in selection".to_string());
        }
        
        // Analyze captured blocks
        let mut material_counts = HashMap::new();
        let mut blocks = Vec::new();
        
        for captured_block in &self.captured_blocks {
            let relative_pos = captured_block.position - min_bound;
            
            blocks.push(BlueprintBlock {
                position: relative_pos,
                material: captured_block.material.clone(),
                rotation: UnitQuaternion::identity(),
                scale: Vector3::new(1.0, 1.0, 1.0),
                priority: 1,
                dependencies: vec![],
                properties: captured_block.properties.clone(),
            });
            
            *material_counts.entry(captured_block.material.clone()).or_insert(0u32) += 1;
        }
        
        let blueprint_id = format!("custom_{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs());
        
        let blueprint = Blueprint {
            id: blueprint_id.clone(),
            name: name.to_string(),
            description: format!("Custom blueprint created by {}", author),
            author: author.to_string(),
            version: "1.0".to_string(),
            category: "custom".to_string(),
            tags: vec!["custom".to_string(), "player_made".to_string()],
            
            blocks,
            size,
            center_point: size / 2,
            anchor_points: vec![],
            
            material_requirements: material_counts,
            tool_requirements: vec!["placer".to_string()],
            skill_requirements: HashMap::new(),
            energy_cost: self.calculate_energy_cost(&self.captured_blocks),
            build_time: self.estimate_build_time(&self.captured_blocks),
            
            difficulty: self.calculate_difficulty(&self.captured_blocks),
            complexity: self.calculate_complexity(&self.captured_blocks),
            structural_rating: 0.5,
            aesthetic_rating: 0.5,
            functionality_rating: 0.5,
            
            build_order: vec![],
            alternative_materials: HashMap::new(),
            optional_components: vec![],
            
            thumbnail: None,
            preview_images: vec![],
            schematic_views: vec![],
            
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            modified_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        self.blueprints.insert(blueprint_id.clone(), blueprint);
        
        // Add to custom category
        self.categories.entry("custom".to_string())
            .or_insert_with(Vec::new)
            .push(blueprint_id.clone());
        
        self.captured_blocks.clear();
        self.selection_bounds = None;
        
        Ok(blueprint_id)
    }

    pub fn queue_construction(&mut self, blueprint_id: &str, position: Point3<f32>, engineer_id: &str) -> Result<(), String> {
        let blueprint = self.blueprints.get(blueprint_id)
            .ok_or_else(|| format!("Blueprint '{}' not found", blueprint_id))?;
        
        let construction = QueuedConstruction {
            blueprint_id: blueprint_id.to_string(),
            position,
            rotation: self.blueprint_rotation,
            scale: self.blueprint_scale,
            priority: 1,
            start_time: None,
            estimated_completion: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() + blueprint.build_time as u64,
            assigned_engineer: engineer_id.to_string(),
            status: ConstructionStatus::Queued,
            progress: 0.0,
        };
        
        self.construction_queue.push(construction);
        Ok(())
    }

    fn calculate_energy_cost(&self, blocks: &[CapturedBlock]) -> f32 {
        blocks.len() as f32 * 1.5
    }

    fn estimate_build_time(&self, blocks: &[CapturedBlock]) -> f32 {
        blocks.len() as f32 * 2.0
    }

    fn calculate_difficulty(&self, blocks: &[CapturedBlock]) -> f32 {
        let structural_blocks = blocks.iter()
            .filter(|b| b.properties.structural)
            .count();
        
        (structural_blocks as f32 / blocks.len() as f32) * 5.0
    }

    fn calculate_complexity(&self, blocks: &[CapturedBlock]) -> f32 {
        let unique_materials = blocks.iter()
            .map(|b| &b.material)
            .collect::<std::collections::HashSet<_>>()
            .len();
        
        (unique_materials as f32).log2()
    }

    // Getters and utility functions
    pub fn get_blueprint(&self, id: &str) -> Option<&Blueprint> {
        self.blueprints.get(id)
    }

    pub fn get_blueprints_by_category(&self, category: &str) -> Vec<&Blueprint> {
        if let Some(blueprint_ids) = self.categories.get(category) {
            blueprint_ids.iter()
                .filter_map(|id| self.blueprints.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn search_blueprints(&self, query: &str) -> Vec<&Blueprint> {
        let query_lower = query.to_lowercase();
        
        self.blueprints.values()
            .filter(|blueprint| {
                blueprint.name.to_lowercase().contains(&query_lower) ||
                blueprint.description.to_lowercase().contains(&query_lower) ||
                blueprint.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    pub fn get_construction_queue(&self) -> &[QueuedConstruction] {
        &self.construction_queue
    }

    pub fn get_active_constructions(&self) -> Vec<&QueuedConstruction> {
        self.construction_queue.iter()
            .filter(|c| matches!(c.status, ConstructionStatus::InProgress))
            .collect()
    }

    pub fn set_active_blueprint(&mut self, blueprint_id: &str) -> Result<(), String> {
        if self.blueprints.contains_key(blueprint_id) {
            self.active_blueprint = Some(blueprint_id.to_string());
            Ok(())
        } else {
            Err(format!("Blueprint '{}' not found", blueprint_id))
        }
    }

    pub fn clear_active_blueprint(&mut self) {
        self.active_blueprint = None;
    }

    pub fn set_blueprint_position(&mut self, position: Point3<f32>) {
        self.blueprint_position = position;
    }

    pub fn set_blueprint_rotation(&mut self, rotation: UnitQuaternion<f32>) {
        self.blueprint_rotation = rotation;
    }

    pub fn set_blueprint_scale(&mut self, scale: Vector3<f32>) {
        self.blueprint_scale = scale;
    }
}