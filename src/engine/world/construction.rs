use nalgebra::{Vector3, Point3, Matrix4};
use std::collections::HashMap;
use super::building::BuildingSystem;

// Temporarily disabled due to nalgebra math type issues:
// pub mod voxel_engine;
// pub mod placement_system;
// pub mod terrain_modification;
// pub mod blueprint_system;

// Temporarily disabled exports:
// pub use voxel_engine::VoxelEngine;
// pub use placement_system::PlacementSystem;
// pub use terrain_modification::TerrainModifier;
// pub use blueprint_system::BlueprintSystem;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MaterialType {
    Wood,
    Stone,
    Metal,
    Glass,
    Concrete,
    Earth,
    Water,
    Air,
    Custom(String),
}

// Simplified VoxelType enum for compatibility with robin_demo
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum VoxelType {
    Air,
    Stone,
    Dirt,
    Grass,
    Sand,
    Water,
    Wood,
    Leaves,
    Crystal,
    Lava,
    // Enhanced Materials from robin_demo
    Glass,
    Metal,
    Brick,
    Ice,
    Obsidian,
}

impl VoxelType {
    pub fn get_color(&self) -> [f32; 3] {
        match self {
            VoxelType::Air => [0.0, 0.0, 0.0],
            VoxelType::Stone => [0.6, 0.6, 0.6],
            VoxelType::Dirt => [0.6, 0.4, 0.2],
            VoxelType::Grass => [0.2, 0.8, 0.2],
            VoxelType::Sand => [0.9, 0.8, 0.6],
            VoxelType::Water => [0.2, 0.4, 0.8],
            VoxelType::Wood => [0.6, 0.4, 0.2],
            VoxelType::Leaves => [0.1, 0.6, 0.1],
            VoxelType::Crystal => [0.7, 0.3, 0.9], // Purple
            VoxelType::Lava => [1.0, 0.3, 0.1], // Orange-red
            // Enhanced Materials
            VoxelType::Glass => [0.8, 0.9, 1.0], // Light blue-white
            VoxelType::Metal => [0.7, 0.7, 0.8], // Silver
            VoxelType::Brick => [0.8, 0.4, 0.3], // Red-brown
            VoxelType::Ice => [0.9, 0.95, 1.0], // Icy white
            VoxelType::Obsidian => [0.1, 0.1, 0.15], // Dark volcanic glass
        }
    }

    /// Get texture atlas coordinates for this voxel type
    /// Returns (u_min, v_min, u_max, v_max) for a 4x4 texture atlas
    pub fn get_texture_coords(&self) -> (f32, f32, f32, f32) {
        const ATLAS_SIZE: f32 = 4.0; // 4x4 texture atlas
        const CELL_SIZE: f32 = 1.0 / ATLAS_SIZE;

        let (atlas_x, atlas_y) = match self {
            VoxelType::Air => (0, 0),      // Transparent/empty
            VoxelType::Stone => (1, 0),    // Stone texture
            VoxelType::Dirt => (2, 0),     // Dirt texture
            VoxelType::Grass => (3, 0),    // Grass texture
            VoxelType::Sand => (0, 1),     // Sand texture
            VoxelType::Water => (1, 1),    // Water texture
            VoxelType::Wood => (2, 1),     // Wood texture
            VoxelType::Leaves => (3, 1),   // Leaves texture
            VoxelType::Crystal => (0, 2),  // Crystal texture
            VoxelType::Lava => (1, 2),     // Lava texture
            VoxelType::Glass => (2, 2),    // Glass texture
            VoxelType::Metal => (3, 2),    // Metal texture
            VoxelType::Brick => (0, 3),    // Brick texture
            VoxelType::Ice => (1, 3),      // Ice texture
            VoxelType::Obsidian => (2, 3), // Obsidian texture
        };

        let u_min = atlas_x as f32 * CELL_SIZE;
        let v_min = atlas_y as f32 * CELL_SIZE;
        let u_max = u_min + CELL_SIZE;
        let v_max = v_min + CELL_SIZE;

        (u_min, v_min, u_max, v_max)
    }

    /// Get texture coordinates for a specific face of the voxel
    /// Returns UV coordinates for the four corners of a face
    pub fn get_face_texture_coords(&self, _face_normal: [f32; 3]) -> [[f32; 2]; 4] {
        let (u_min, v_min, u_max, v_max) = self.get_texture_coords();

        // Return UV coordinates for quad: bottom-left, bottom-right, top-right, top-left
        [
            [u_min, v_max], // bottom-left
            [u_max, v_max], // bottom-right
            [u_max, v_min], // top-right
            [u_min, v_min], // top-left
        ]
    }

    pub fn to_material(&self) -> Material {
        let (material_type, density, hardness, transparency, color, properties) = match self {
            VoxelType::Air => (
                MaterialType::Air,
                0.0,
                0.0,
                1.0,
                [0.0, 0.0, 0.0, 0.0],
                MaterialProperties {
                    gas: true,
                    structural: false,
                    ..Default::default()
                }
            ),
            VoxelType::Stone => (
                MaterialType::Stone,
                2.7,
                8.0,
                0.0,
                [0.6, 0.6, 0.6, 1.0],
                MaterialProperties {
                    structural: true,
                    ..Default::default()
                }
            ),
            VoxelType::Dirt => (
                MaterialType::Earth,
                1.5,
                2.0,
                0.0,
                [0.6, 0.4, 0.2, 1.0],
                MaterialProperties {
                    structural: true,
                    ..Default::default()
                }
            ),
            VoxelType::Grass => (
                MaterialType::Earth,
                1.2,
                1.0,
                0.0,
                [0.2, 0.8, 0.2, 1.0],
                MaterialProperties {
                    flammable: true,
                    decorative: true,
                    ..Default::default()
                }
            ),
            VoxelType::Sand => (
                MaterialType::Earth,
                1.6,
                1.0,
                0.0,
                [0.9, 0.8, 0.6, 1.0],
                MaterialProperties {
                    structural: false,
                    ..Default::default()
                }
            ),
            VoxelType::Water => (
                MaterialType::Water,
                1.0,
                0.0,
                0.8,
                [0.2, 0.4, 0.8, 0.8],
                MaterialProperties {
                    liquid: true,
                    structural: false,
                    conductive: true,
                    ..Default::default()
                }
            ),
            VoxelType::Wood => (
                MaterialType::Wood,
                0.8,
                3.0,
                0.0,
                [0.6, 0.4, 0.2, 1.0],
                MaterialProperties {
                    flammable: true,
                    structural: true,
                    ..Default::default()
                }
            ),
            VoxelType::Leaves => (
                MaterialType::Wood,
                0.3,
                0.5,
                0.2,
                [0.1, 0.6, 0.1, 1.0],
                MaterialProperties {
                    flammable: true,
                    decorative: true,
                    structural: false,
                    ..Default::default()
                }
            ),
            VoxelType::Crystal => (
                MaterialType::Glass,
                2.2,
                7.0,
                0.3,
                [0.7, 0.3, 0.9, 1.0],
                MaterialProperties {
                    decorative: true,
                    structural: true,
                    ..Default::default()
                }
            ),
            VoxelType::Lava => (
                MaterialType::Custom("Lava".to_string()),
                3.0,
                0.1,
                0.1,
                [1.0, 0.3, 0.1, 1.0],
                MaterialProperties {
                    liquid: true,
                    structural: false,
                    ..Default::default()
                }
            ),
            VoxelType::Glass => (
                MaterialType::Glass,
                2.5,
                5.0,
                0.9,
                [0.8, 0.9, 1.0, 0.5],
                MaterialProperties {
                    structural: true,
                    decorative: true,
                    ..Default::default()
                }
            ),
            VoxelType::Metal => (
                MaterialType::Metal,
                7.8,
                9.0,
                0.0,
                [0.7, 0.7, 0.8, 1.0],
                MaterialProperties {
                    structural: true,
                    conductive: true,
                    magnetic: true,
                    ..Default::default()
                }
            ),
            VoxelType::Brick => (
                MaterialType::Concrete,
                2.0,
                6.0,
                0.0,
                [0.8, 0.4, 0.3, 1.0],
                MaterialProperties {
                    structural: true,
                    ..Default::default()
                }
            ),
            VoxelType::Ice => (
                MaterialType::Water,
                0.9,
                2.0,
                0.7,
                [0.9, 0.95, 1.0, 0.8],
                MaterialProperties {
                    structural: false,
                    ..Default::default()
                }
            ),
            VoxelType::Obsidian => (
                MaterialType::Glass,
                2.4,
                7.5,
                0.1,
                [0.1, 0.1, 0.15, 1.0],
                MaterialProperties {
                    structural: true,
                    ..Default::default()
                }
            ),
        };

        Material {
            material_type,
            density,
            hardness,
            transparency,
            conductivity: if properties.conductive { 1.0 } else { 0.0 },
            color,
            texture_id: None,
            properties,
        }
    }

    /// Get PBR roughness value for this material (0.0 = mirror, 1.0 = completely rough)
    pub fn get_roughness(&self) -> f32 {
        match self {
            VoxelType::Air => 1.0,          // Not visible, but rough
            VoxelType::Stone => 0.8,        // Rough natural stone
            VoxelType::Dirt => 0.9,         // Very rough earth
            VoxelType::Grass => 0.85,       // Rough organic surface
            VoxelType::Sand => 0.7,         // Moderately rough
            VoxelType::Water => 0.0,        // Perfect mirror when still
            VoxelType::Wood => 0.6,         // Moderately rough wood
            VoxelType::Leaves => 0.9,       // Very rough organic
            VoxelType::Crystal => 0.1,      // Very smooth crystal
            VoxelType::Lava => 0.4,         // Somewhat smooth when molten
            VoxelType::Glass => 0.05,       // Very smooth glass
            VoxelType::Metal => 0.2,        // Polished metal
            VoxelType::Brick => 0.7,        // Rough fired clay
            VoxelType::Ice => 0.15,         // Smooth but not perfect
            VoxelType::Obsidian => 0.1,     // Very smooth volcanic glass
        }
    }

    /// Get PBR metallic value for this material (0.0 = dielectric, 1.0 = pure metal)
    pub fn get_metallic(&self) -> f32 {
        match self {
            VoxelType::Air => 0.0,          // Not metallic
            VoxelType::Stone => 0.0,        // Non-metallic rock
            VoxelType::Dirt => 0.0,         // Non-metallic earth
            VoxelType::Grass => 0.0,        // Organic, non-metallic
            VoxelType::Sand => 0.0,         // Non-metallic silica
            VoxelType::Water => 0.0,        // Non-metallic liquid
            VoxelType::Wood => 0.0,         // Organic, non-metallic
            VoxelType::Leaves => 0.0,       // Organic, non-metallic
            VoxelType::Crystal => 0.0,      // Crystalline but not metallic
            VoxelType::Lava => 0.0,         // Molten rock, not metallic
            VoxelType::Glass => 0.0,        // Non-metallic transparent
            VoxelType::Metal => 1.0,        // Pure metallic
            VoxelType::Brick => 0.0,        // Fired clay, non-metallic
            VoxelType::Ice => 0.0,          // Frozen water, non-metallic
            VoxelType::Obsidian => 0.0,     // Volcanic glass, non-metallic
        }
    }

    /// Returns true if this voxel type is affected by gravity and should fall
    pub fn is_gravity_affected(&self) -> bool {
        match self {
            VoxelType::Sand => true,     // Sand falls
            VoxelType::Dirt => true,     // Loose dirt can fall
            VoxelType::Leaves => true,   // Leaves fall when not supported
            VoxelType::Ice => true,      // Ice can fall when melting/unstable
            _ => false,
        }
    }

    /// Returns true if this voxel type can provide structural support to other blocks
    pub fn is_structural_support(&self) -> bool {
        match self {
            VoxelType::Air => false,     // Air provides no support
            VoxelType::Water => false,   // Water provides no support
            VoxelType::Lava => false,    // Lava provides no support
            _ => true,                   // All solid blocks provide support
        }
    }

    /// Returns the density of this voxel type for physics calculations
    pub fn density(&self) -> f32 {
        match self {
            VoxelType::Air => 0.0,
            VoxelType::Stone => 2.7,
            VoxelType::Dirt => 1.5,
            VoxelType::Grass => 1.2,
            VoxelType::Sand => 1.6,
            VoxelType::Water => 1.0,
            VoxelType::Wood => 0.6,
            VoxelType::Leaves => 0.3,
            VoxelType::Crystal => 2.2,
            VoxelType::Lava => 3.0,
            VoxelType::Glass => 2.5,
            VoxelType::Metal => 7.8,
            VoxelType::Brick => 2.0,
            VoxelType::Ice => 0.9,
            VoxelType::Obsidian => 2.4,
        }
    }

}

#[derive(Clone, Debug)]
pub struct Material {
    pub material_type: MaterialType,
    pub density: f32,
    pub hardness: f32,
    pub transparency: f32,
    pub conductivity: f32,
    pub color: [f32; 4], // RGBA
    pub texture_id: Option<String>,
    pub properties: MaterialProperties,
}

#[derive(Clone, Debug)]
pub struct MaterialProperties {
    pub flammable: bool,
    pub conductive: bool,
    pub magnetic: bool,
    pub liquid: bool,
    pub gas: bool,
    pub structural: bool,
    pub decorative: bool,
}

impl Default for MaterialProperties {
    fn default() -> Self {
        Self {
            flammable: false,
            conductive: false,
            magnetic: false,
            liquid: false,
            gas: false,
            structural: true,
            decorative: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Voxel {
    pub material: Material,
    pub health: f32,
    pub temperature: f32,
    pub metadata: VoxelMetadata,
}

#[derive(Clone, Debug)]
pub struct VoxelMetadata {
    pub last_modified: u64,
    pub modified_by: String,
    pub structural_integrity: f32,
    pub connected_components: Vec<String>,
    pub custom_data: HashMap<String, String>,
}

#[derive(Clone, Debug)]
pub struct BuildTool {
    pub name: String,
    pub tool_type: BuildToolType,
    pub range: f32,
    pub precision: f32,
    pub power: f32,
    pub energy_cost: f32,
    pub materials_required: Vec<MaterialType>,
}

#[derive(Clone, Debug)]
pub enum BuildToolType {
    Place,
    Remove,
    Paint,
    Copy,
    Paste,
    Terraform,
    Sculpt,
    Weld,
    Cut,
    Measure,
}

#[derive(Clone, Debug)]
pub struct Construction {
    pub id: String,
    pub name: String,
    pub position: Point3<f32>,
    pub rotation: Matrix4<f32>,
    pub scale: Vector3<f32>,
    pub voxels: HashMap<Point3<i32>, Voxel>,
    pub bounds: (Point3<i32>, Point3<i32>),
    pub structural_nodes: Vec<StructuralNode>,
    pub construction_type: ConstructionType,
}

#[derive(Clone, Debug)]
pub enum ConstructionType {
    Building,
    Bridge,
    Road,
    Tunnel,
    Tower,
    Wall,
    Foundation,
    Decoration,
    Infrastructure,
    Vehicle,
    Custom(String),
}

#[derive(Clone, Debug)]
pub struct StructuralNode {
    pub position: Point3<i32>,
    pub node_type: StructuralNodeType,
    pub connections: Vec<Point3<i32>>,
    pub load_capacity: f32,
    pub current_load: f32,
    pub health: f32,
}

#[derive(Clone, Debug)]
pub enum StructuralNodeType {
    Support,
    Joint,
    Anchor,
    Connector,
    Hinge,
    Slider,
}

// Temporarily disabled due to nalgebra math type issues:
/*
pub struct WorldConstructionSystem {
    pub voxel_engine: VoxelEngine,
    pub placement_system: PlacementSystem,
    pub terrain_modifier: TerrainModifier,
    pub blueprint_system: BlueprintSystem,
    
    // Active constructions
    pub active_constructions: HashMap<String, Construction>,
    
    // Material library
    pub material_library: HashMap<String, Material>,
    
    // Tool inventory
    pub available_tools: HashMap<String, BuildTool>,
    
    // Construction history for undo/redo
    pub construction_history: Vec<ConstructionEvent>,
    pub history_index: usize,
    
    // Performance settings
    pub max_constructions: usize,
    pub voxel_update_range: f32,
    pub auto_save_interval: f32,
    pub last_auto_save: f32,
}

#[derive(Clone, Debug)]
pub struct ConstructionEvent {
    pub event_type: ConstructionEventType,
    pub construction_id: String,
    pub position: Point3<i32>,
    pub old_voxel: Option<Voxel>,
    pub new_voxel: Option<Voxel>,
    pub timestamp: u64,
    pub engineer_id: String,
}

#[derive(Clone, Debug)]
pub enum ConstructionEventType {
    Place,
    Remove,
    Modify,
    Paint,
    Copy,
    Paste,
    Terraform,
    StructuralChange,
}

impl WorldConstructionSystem {
    pub fn new() -> Self {
        let mut system = Self {
            voxel_engine: VoxelEngine::new(),
            placement_system: PlacementSystem::new(),
            terrain_modifier: TerrainModifier::new(),
            blueprint_system: BlueprintSystem::new(),
            
            active_constructions: HashMap::new(),
            material_library: HashMap::new(),
            available_tools: HashMap::new(),
            construction_history: Vec::new(),
            history_index: 0,
            
            max_constructions: 10000,
            voxel_update_range: 50.0,
            auto_save_interval: 300.0, // 5 minutes
            last_auto_save: 0.0,
        };
        
        system.initialize_default_materials();
        system.initialize_default_tools();
        system
    }

    fn initialize_default_materials(&mut self) {
        // Wood
        self.material_library.insert("wood".to_string(), Material {
            material_type: MaterialType::Wood,
            density: 0.6,
            hardness: 3.0,
            transparency: 0.0,
            conductivity: 0.1,
            color: [0.6, 0.4, 0.2, 1.0],
            texture_id: Some("wood_planks".to_string()),
            properties: MaterialProperties {
                flammable: true,
                structural: true,
                ..Default::default()
            },
        });

        // Stone
        self.material_library.insert("stone".to_string(), Material {
            material_type: MaterialType::Stone,
            density: 2.5,
            hardness: 8.0,
            transparency: 0.0,
            conductivity: 0.2,
            color: [0.5, 0.5, 0.5, 1.0],
            texture_id: Some("stone_blocks".to_string()),
            properties: MaterialProperties {
                structural: true,
                ..Default::default()
            },
        });

        // Metal
        self.material_library.insert("metal".to_string(), Material {
            material_type: MaterialType::Metal,
            density: 7.8,
            hardness: 9.0,
            transparency: 0.0,
            conductivity: 0.9,
            color: [0.7, 0.7, 0.8, 1.0],
            texture_id: Some("metal_plates".to_string()),
            properties: MaterialProperties {
                conductive: true,
                magnetic: true,
                structural: true,
                ..Default::default()
            },
        });

        // Glass
        self.material_library.insert("glass".to_string(), Material {
            material_type: MaterialType::Glass,
            density: 2.5,
            hardness: 5.0,
            transparency: 0.9,
            conductivity: 0.0,
            color: [0.9, 0.9, 1.0, 0.3],
            texture_id: Some("clear_glass".to_string()),
            properties: MaterialProperties {
                decorative: true,
                ..Default::default()
            },
        });

        // Earth/Dirt
        self.material_library.insert("earth".to_string(), Material {
            material_type: MaterialType::Earth,
            density: 1.3,
            hardness: 2.0,
            transparency: 0.0,
            conductivity: 0.1,
            color: [0.4, 0.3, 0.2, 1.0],
            texture_id: Some("dirt".to_string()),
            properties: MaterialProperties {
                flammable: false,
                structural: false,
                ..Default::default()
            },
        });
    }

    fn initialize_default_tools(&mut self) {
        // Placement tool
        self.available_tools.insert("placer".to_string(), BuildTool {
            name: "Block Placer".to_string(),
            tool_type: BuildToolType::Place,
            range: 10.0,
            precision: 1.0,
            power: 1.0,
            energy_cost: 1.0,
            materials_required: vec![],
        });

        // Removal tool
        self.available_tools.insert("remover".to_string(), BuildTool {
            name: "Block Remover".to_string(),
            tool_type: BuildToolType::Remove,
            range: 10.0,
            precision: 1.0,
            power: 2.0,
            energy_cost: 1.5,
            materials_required: vec![],
        });

        // Terrain tool
        self.available_tools.insert("terraform".to_string(), BuildTool {
            name: "Terrain Former".to_string(),
            tool_type: BuildToolType::Terraform,
            range: 20.0,
            precision: 0.5,
            power: 5.0,
            energy_cost: 3.0,
            materials_required: vec![MaterialType::Earth],
        });

        // Paint tool
        self.available_tools.insert("painter".to_string(), BuildTool {
            name: "Material Painter".to_string(),
            tool_type: BuildToolType::Paint,
            range: 8.0,
            precision: 1.0,
            power: 1.0,
            energy_cost: 0.5,
            materials_required: vec![],
        });

        // Copy tool
        self.available_tools.insert("copier".to_string(), BuildTool {
            name: "Structure Copier".to_string(),
            tool_type: BuildToolType::Copy,
            range: 15.0,
            precision: 1.0,
            power: 1.0,
            energy_cost: 2.0,
            materials_required: vec![],
        });
    }

    pub fn update(&mut self, delta_time: f32, engineer_position: Point3<f32>) {
        // Update voxel engine
        self.voxel_engine.update(delta_time, engineer_position);
        
        // Update placement system
        self.placement_system.update(delta_time);
        
        // Update terrain modifier
        self.terrain_modifier.update(delta_time);
        
        // Update blueprint system
        self.blueprint_system.update(delta_time);
        
        // Auto-save check
        self.last_auto_save += delta_time;
        if self.last_auto_save >= self.auto_save_interval {
            self.auto_save();
            self.last_auto_save = 0.0;
        }
        
        // Update active constructions
        self.update_constructions(delta_time, engineer_position);
    }

    fn update_constructions(&mut self, delta_time: f32, engineer_position: Point3<f32>) {
        for (_, construction) in &mut self.active_constructions {
            // Update structural integrity
            self.update_structural_integrity(construction);
            
            // Update voxel properties (temperature, wear, etc.)
            self.update_voxel_properties(construction, delta_time);
            
            // Check if construction is in update range
            let distance = (construction.position - engineer_position).magnitude();
            if distance <= self.voxel_update_range {
                self.perform_detailed_updates(construction, delta_time);
            }
        }
    }

    fn update_structural_integrity(&mut self, construction: &mut Construction) {
        for node in &mut construction.structural_nodes {
            // Calculate load distribution
            let load_ratio = node.current_load / node.load_capacity;
            
            // Update health based on load
            if load_ratio > 1.0 {
                node.health -= (load_ratio - 1.0) * 0.1; // Damage from overload
            } else if node.health < 1.0 && load_ratio < 0.8 {
                node.health += 0.01; // Slow repair when underloaded
            }
            
            // Clamp health
            node.health = node.health.clamp(0.0, 1.0);
        }
    }

    fn update_voxel_properties(&mut self, construction: &mut Construction, delta_time: f32) {
        for (_, voxel) in &mut construction.voxels {
            // Temperature equilibrium
            if voxel.temperature > 20.0 { // Ambient temperature
                voxel.temperature -= 10.0 * delta_time; // Cool down
            } else if voxel.temperature < 20.0 {
                voxel.temperature += 10.0 * delta_time; // Warm up
            }
            
            // Clamp temperature
            voxel.temperature = voxel.temperature.clamp(-50.0, 1000.0);
            
            // Update structural integrity based on material
            if voxel.material.properties.structural {
                voxel.metadata.structural_integrity = 
                    (voxel.health * voxel.material.hardness / 10.0).min(1.0);
            }
        }
    }

    fn perform_detailed_updates(&mut self, construction: &mut Construction, delta_time: f32) {
        // Detailed physics updates for nearby constructions
        // Material interactions, chemical reactions, etc.
        
        // Update connected components
        self.update_connected_components(construction);
        
        // Check for structural failures
        self.check_structural_failures(construction);
    }

    fn update_connected_components(&mut self, construction: &mut Construction) {
        // Analyze connectivity between voxels
        // Update structural node connections
        // This would implement flood-fill algorithms to find connected regions
        
        for node in &mut construction.structural_nodes {
            // Update connections based on surrounding voxels
            node.connections.clear();
            
            // Check 6-connected neighbors
            let neighbors = [
                Point3::new(1, 0, 0), Point3::new(-1, 0, 0),
                Point3::new(0, 1, 0), Point3::new(0, -1, 0),
                Point3::new(0, 0, 1), Point3::new(0, 0, -1),
            ];
            
            for neighbor_offset in &neighbors {
                let neighbor_pos = node.position + neighbor_offset;
                if construction.voxels.contains_key(&neighbor_pos) {
                    node.connections.push(neighbor_pos);
                }
            }
        }
    }

    fn check_structural_failures(&mut self, construction: &mut Construction) {
        let mut failed_nodes = Vec::new();
        
        for (i, node) in construction.structural_nodes.iter().enumerate() {
            if node.health <= 0.0 {
                failed_nodes.push(i);
            }
        }
        
        // Remove failed nodes and propagate structural damage
        for &index in failed_nodes.iter().rev() {
            let failed_node = construction.structural_nodes.remove(index);
            
            // Remove voxel at failed position
            construction.voxels.remove(&failed_node.position);
            
            // Redistribute load to connected nodes
            for connected_pos in &failed_node.connections {
                if let Some(connected_node) = construction.structural_nodes.iter_mut()
                    .find(|n| n.position == *connected_pos) {
                    connected_node.current_load += failed_node.current_load / failed_node.connections.len() as f32;
                }
            }
        }
    }

    pub fn place_voxel(&mut self, position: Point3<i32>, material_name: &str, engineer_id: &str) -> Result<(), String> {
        let material = self.material_library.get(material_name)
            .ok_or_else(|| format!("Unknown material: {}", material_name))?
            .clone();
        
        let voxel = Voxel {
            material,
            health: 1.0,
            temperature: 20.0,
            metadata: VoxelMetadata {
                last_modified: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                modified_by: engineer_id.to_string(),
                structural_integrity: 1.0,
                connected_components: Vec::new(),
                custom_data: HashMap::new(),
            },
        };
        
        // Find or create construction
        let construction_id = self.find_or_create_construction(position);
        
        if let Some(construction) = self.active_constructions.get_mut(&construction_id) {
            let old_voxel = construction.voxels.insert(position, voxel.clone());
            
            // Update bounds
            self.update_construction_bounds(construction, position);
            
            // Add structural node if material is structural
            if voxel.material.properties.structural {
                construction.structural_nodes.push(StructuralNode {
                    position,
                    node_type: StructuralNodeType::Support,
                    connections: Vec::new(),
                    load_capacity: voxel.material.hardness * 10.0,
                    current_load: 0.0,
                    health: 1.0,
                });
            }
            
            // Record construction event
            self.construction_history.push(ConstructionEvent {
                event_type: ConstructionEventType::Place,
                construction_id: construction_id.clone(),
                position,
                old_voxel,
                new_voxel: Some(voxel),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                engineer_id: engineer_id.to_string(),
            });
            
            // Trim history if too long
            if self.construction_history.len() > 1000 {
                self.construction_history.drain(0..100);
                self.history_index = self.history_index.saturating_sub(100);
            }
            
            self.history_index = self.construction_history.len();
        }
        
        Ok(())
    }

    pub fn remove_voxel(&mut self, position: Point3<i32>, engineer_id: &str) -> Result<(), String> {
        // Find construction containing this voxel
        let mut construction_to_update = None;
        
        for (id, construction) in &self.active_constructions {
            if construction.voxels.contains_key(&position) {
                construction_to_update = Some(id.clone());
                break;
            }
        }
        
        if let Some(construction_id) = construction_to_update {
            if let Some(construction) = self.active_constructions.get_mut(&construction_id) {
                let old_voxel = construction.voxels.remove(&position);
                
                // Remove associated structural node
                construction.structural_nodes.retain(|node| node.position != position);
                
                // Record construction event
                if let Some(voxel) = &old_voxel {
                    self.construction_history.push(ConstructionEvent {
                        event_type: ConstructionEventType::Remove,
                        construction_id: construction_id.clone(),
                        position,
                        old_voxel: old_voxel.clone(),
                        new_voxel: None,
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                        engineer_id: engineer_id.to_string(),
                    });
                }
                
                self.history_index = self.construction_history.len();
                
                return Ok(());
            }
        }
        
        Err("No voxel found at position".to_string())
    }

    fn find_or_create_construction(&mut self, position: Point3<i32>) -> String {
        // Look for nearby constructions to attach to
        for (id, construction) in &self.active_constructions {
            let distance = (construction.position - Point3::new(position.x as f32, position.y as f32, position.z as f32)).magnitude();
            if distance < 50.0 { // Merge threshold
                return id.clone();
            }
        }
        
        // Create new construction
        let construction_id = format!("construction_{}", self.active_constructions.len());
        let construction = Construction {
            id: construction_id.clone(),
            name: format!("Structure {}", self.active_constructions.len() + 1),
            position: Point3::new(position.x as f32, position.y as f32, position.z as f32),
            rotation: Matrix4::identity(),
            scale: Vector3::new(1.0, 1.0, 1.0),
            voxels: HashMap::new(),
            bounds: (position, position),
            structural_nodes: Vec::new(),
            construction_type: ConstructionType::Building,
        };
        
        self.active_constructions.insert(construction_id.clone(), construction);
        construction_id
    }

    fn update_construction_bounds(&mut self, construction: &mut Construction, new_position: Point3<i32>) {
        let (min_bound, max_bound) = construction.bounds;
        
        construction.bounds = (
            Point3::new(
                min_bound.x.min(new_position.x),
                min_bound.y.min(new_position.y),
                min_bound.z.min(new_position.z),
            ),
            Point3::new(
                max_bound.x.max(new_position.x),
                max_bound.y.max(new_position.y),
                max_bound.z.max(new_position.z),
            ),
        );
    }

    pub fn undo(&mut self) -> Result<(), String> {
        if self.history_index == 0 {
            return Err("Nothing to undo".to_string());
        }
        
        self.history_index -= 1;
        let event = &self.construction_history[self.history_index].clone();
        
        match event.event_type {
            ConstructionEventType::Place => {
                // Undo place = remove
                if let Some(construction) = self.active_constructions.get_mut(&event.construction_id) {
                    construction.voxels.remove(&event.position);
                    construction.structural_nodes.retain(|node| node.position != event.position);
                }
            }
            ConstructionEventType::Remove => {
                // Undo remove = place back
                if let Some(old_voxel) = &event.old_voxel {
                    if let Some(construction) = self.active_constructions.get_mut(&event.construction_id) {
                        construction.voxels.insert(event.position, old_voxel.clone());
                        
                        if old_voxel.material.properties.structural {
                            construction.structural_nodes.push(StructuralNode {
                                position: event.position,
                                node_type: StructuralNodeType::Support,
                                connections: Vec::new(),
                                load_capacity: old_voxel.material.hardness * 10.0,
                                current_load: 0.0,
                                health: old_voxel.health,
                            });
                        }
                    }
                }
            }
            _ => {}
        }
        
        Ok(())
    }

    pub fn redo(&mut self) -> Result<(), String> {
        if self.history_index >= self.construction_history.len() {
            return Err("Nothing to redo".to_string());
        }
        
        let event = &self.construction_history[self.history_index].clone();
        self.history_index += 1;
        
        match event.event_type {
            ConstructionEventType::Place => {
                // Redo place
                if let Some(new_voxel) = &event.new_voxel {
                    if let Some(construction) = self.active_constructions.get_mut(&event.construction_id) {
                        construction.voxels.insert(event.position, new_voxel.clone());
                        
                        if new_voxel.material.properties.structural {
                            construction.structural_nodes.push(StructuralNode {
                                position: event.position,
                                node_type: StructuralNodeType::Support,
                                connections: Vec::new(),
                                load_capacity: new_voxel.material.hardness * 10.0,
                                current_load: 0.0,
                                health: 1.0,
                            });
                        }
                    }
                }
            }
            ConstructionEventType::Remove => {
                // Redo remove
                if let Some(construction) = self.active_constructions.get_mut(&event.construction_id) {
                    construction.voxels.remove(&event.position);
                    construction.structural_nodes.retain(|node| node.position != event.position);
                }
            }
            _ => {}
        }
        
        Ok(())
    }

    fn auto_save(&mut self) {
        // Implementation would save construction state to disk
        println!("Auto-saving world constructions...");
    }

    // Getters
    pub fn get_construction(&self, id: &str) -> Option<&Construction> {
        self.active_constructions.get(id)
    }

    pub fn get_voxel_at(&self, position: Point3<i32>) -> Option<&Voxel> {
        for construction in self.active_constructions.values() {
            if let Some(voxel) = construction.voxels.get(&position) {
                return Some(voxel);
            }
        }
        None
    }

    pub fn get_material(&self, name: &str) -> Option<&Material> {
        self.material_library.get(name)
    }

    pub fn get_tool(&self, name: &str) -> Option<&BuildTool> {
        self.available_tools.get(name)
    }

    pub fn get_construction_count(&self) -> usize {
        self.active_constructions.len()
    }

    pub fn get_total_voxel_count(&self) -> usize {
        self.active_constructions.values()
            .map(|c| c.voxels.len())
            .sum()
    }
}
*/ // End of temporarily disabled WorldConstructionSystem

/// System for handling falling blocks physics
#[derive(Debug, Clone)]
pub struct FallingBlockSystem {
    /// Blocks that are currently falling (position -> voxel_type)
    pub falling_blocks: HashMap<(i32, i32, i32), VoxelType>,
    /// Blocks that need to be checked for stability
    pub blocks_to_check: Vec<(i32, i32, i32)>,
    /// Physics handles for falling blocks
    pub physics_handles: HashMap<(i32, i32, i32), crate::engine::physics3d::PhysicsHandle>,
}

impl Default for FallingBlockSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl FallingBlockSystem {
    pub fn new() -> Self {
        Self {
            falling_blocks: HashMap::new(),
            blocks_to_check: Vec::new(),
            physics_handles: HashMap::new(),
        }
    }

    /// Check if a block at the given position should start falling
    pub fn check_block_stability(&self, position: (i32, i32, i32), voxel_type: VoxelType, world_data: &HashMap<(i32, i32, i32), VoxelType>) -> bool {
        // Only gravity-affected blocks can fall
        if !voxel_type.is_gravity_affected() {
            return false;
        }

        // Check if there's solid support below
        let below_pos = (position.0, position.1 - 1, position.2);

        match world_data.get(&below_pos) {
            Some(below_voxel) => {
                // Block is supported if there's a structural support block below
                !below_voxel.is_structural_support()
            }
            None => {
                // No block below, should fall
                true
            }
        }
    }

    /// Add a block to the system to check for falling
    pub fn schedule_block_check(&mut self, position: (i32, i32, i32)) {
        if !self.blocks_to_check.contains(&position) {
            self.blocks_to_check.push(position);
        }
    }

    /// Make a block start falling by converting it to a dynamic physics body
    pub fn start_block_falling(&mut self,
        position: (i32, i32, i32),
        voxel_type: VoxelType,
        physics_world: &mut crate::engine::physics3d::PhysicsWorld3D
    ) -> Result<(), crate::engine::error::RobinError> {
        use crate::engine::physics3d::{BodyDescriptor, ColliderShape3D, BodyType3D};
        use crate::engine::math::Vec3;

        // Create a dynamic physics body for the falling block
        let world_pos = Vec3::new(position.0 as f32, position.1 as f32, position.2 as f32);
        let block_descriptor = BodyDescriptor {
            body_type: BodyType3D::Dynamic,
            position: world_pos,
            mass: voxel_type.density(),
            friction: 0.5,
            restitution: 0.1, // Small bounce
            linear_damping: 0.05,
            angular_damping: 0.8,
            gravity_scale: 1.0,
            can_sleep: true,
            lock_rotations: false,
        };

        let block_shape = ColliderShape3D::voxel_block();

        let handle = physics_world.create_body(
            block_descriptor,
            block_shape,
            Some(format!("falling_block_{}_{}_{}_{:?}", position.0, position.1, position.2, voxel_type)),
        )?;

        // Track the falling block
        self.falling_blocks.insert(position, voxel_type);
        self.physics_handles.insert(position, handle);

        Ok(())
    }

    /// Update falling blocks and check if they've landed
    pub fn update(&mut self,
        physics_world: &mut crate::engine::physics3d::PhysicsWorld3D,
        world_data: &mut HashMap<(i32, i32, i32), VoxelType>,
        delta_time: f32
    ) -> Result<(), crate::engine::error::RobinError> {
        use crate::engine::math::Vec3;

        // Check blocks that were scheduled for stability checks
        let blocks_to_check = std::mem::take(&mut self.blocks_to_check);
        for position in blocks_to_check {
            if let Some(&voxel_type) = world_data.get(&position) {
                if self.check_block_stability(position, voxel_type, world_data) {
                    // Remove from world and start falling
                    world_data.remove(&position);
                    self.start_block_falling(position, voxel_type, physics_world)?;
                }
            }
        }

        // Update falling blocks
        let mut blocks_to_land = Vec::new();

        for (&original_pos, &voxel_type) in &self.falling_blocks {
            if let Some(&handle) = self.physics_handles.get(&original_pos) {
                // Check if the block has landed (low velocity and supported)
                if let Some(current_pos) = physics_world.get_body_position(handle) {
                    if let Some(velocity) = physics_world.get_body_velocity(handle) {
                        // If the block is moving slowly and has solid ground below, it has landed
                        if velocity.magnitude() < 0.1 {
                            let landed_pos = (
                                current_pos.x.round() as i32,
                                current_pos.y.round() as i32,
                                current_pos.z.round() as i32,
                            );

                            // Check if there's support below the landing position
                            let below_pos = (landed_pos.0, landed_pos.1 - 1, landed_pos.2);
                            if let Some(below_voxel) = world_data.get(&below_pos) {
                                if below_voxel.is_structural_support() {
                                    blocks_to_land.push((original_pos, landed_pos, voxel_type, handle));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Land the blocks that have stopped
        for (original_pos, landed_pos, voxel_type, handle) in blocks_to_land {
            // Remove from physics world
            physics_world.remove_body(handle)?;

            // Place block in new position in world
            world_data.insert(landed_pos, voxel_type);

            // Remove from falling tracking
            self.falling_blocks.remove(&original_pos);
            self.physics_handles.remove(&original_pos);

            // Check if this landing destabilizes nearby blocks
            self.check_nearby_blocks_for_stability(landed_pos);
        }

        Ok(())
    }

    /// Check blocks around a position for stability (used when a block lands)
    fn check_nearby_blocks_for_stability(&mut self, position: (i32, i32, i32)) {
        // Check blocks above the landed block
        for y_offset in 1..=3 {
            let check_pos = (position.0, position.1 + y_offset, position.2);
            self.schedule_block_check(check_pos);
        }

        // Check adjacent blocks that might have been affected
        for x_offset in -1..=1 {
            for z_offset in -1..=1 {
                if x_offset == 0 && z_offset == 0 { continue; }
                let check_pos = (position.0 + x_offset, position.1 + 1, position.2 + z_offset);
                self.schedule_block_check(check_pos);
            }
        }
    }

    /// Trigger falling block checks for an area (used when blocks are removed)
    pub fn trigger_area_check(&mut self, center: (i32, i32, i32), radius: i32) {
        for x in (center.0 - radius)..=(center.0 + radius) {
            for y in (center.1 - radius)..=(center.1 + radius + 3) { // Check above too
                for z in (center.2 - radius)..=(center.2 + radius) {
                    self.schedule_block_check((x, y, z));
                }
            }
        }
    }

    /// Get the number of currently falling blocks
    pub fn falling_count(&self) -> usize {
        self.falling_blocks.len()
    }

    /// Clear all falling blocks (useful for cleanup)
    pub fn clear(&mut self, physics_world: &mut crate::engine::physics3d::PhysicsWorld3D) -> Result<(), crate::engine::error::RobinError> {
        // Remove all physics bodies
        for handle in self.physics_handles.values() {
            physics_world.remove_body(*handle)?;
        }

        self.falling_blocks.clear();
        self.physics_handles.clear();
        self.blocks_to_check.clear();

        Ok(())
    }
}