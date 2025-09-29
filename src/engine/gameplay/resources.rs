/*!
 * Resource Management System for Robin Engine
 *
 * Manages voxel materials, their properties, gathering mechanics,
 * and resource conversion systems for the building gameplay.
 */

use crate::engine::{
    error::{RobinError, RobinResult},
    save_system::PlayerData,
    world::VoxelType,
};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Core resource management system
pub struct ResourceManager {
    /// Resource definitions and properties
    resource_definitions: HashMap<ResourceType, Resource>,
    /// Mining yield multipliers
    yield_multipliers: HashMap<ResourceType, u32>,
    /// Resource conversion recipes
    conversions: Vec<ResourceConversion>,
}

impl ResourceManager {
    pub fn new() -> Self {
        let mut manager = Self {
            resource_definitions: HashMap::new(),
            yield_multipliers: HashMap::new(),
            conversions: Vec::new(),
        };

        manager.initialize_default_resources();
        manager.initialize_conversions();
        manager
    }

    /// Initialize default voxel resources
    fn initialize_default_resources(&mut self) {
        // Basic terrain materials
        self.resource_definitions.insert(ResourceType::Earth, Resource {
            name: "Earth".to_string(),
            description: "Common soil material, easy to work with".to_string(),
            rarity: ResourceRarity::Common,
            durability: 50.0,
            properties: vec![ResourceProperty::Fertile, ResourceProperty::Soft],
        });

        self.resource_definitions.insert(ResourceType::Stone, Resource {
            name: "Stone".to_string(),
            description: "Durable building material with excellent structural properties".to_string(),
            rarity: ResourceRarity::Common,
            durability: 200.0,
            properties: vec![ResourceProperty::Durable, ResourceProperty::Structural],
        });

        self.resource_definitions.insert(ResourceType::Water, Resource {
            name: "Water".to_string(),
            description: "Liquid resource essential for life and certain crafting recipes".to_string(),
            rarity: ResourceRarity::Common,
            durability: 0.0, // Flows, doesn't break
            properties: vec![ResourceProperty::Liquid, ResourceProperty::Essential],
        });

        self.resource_definitions.insert(ResourceType::Grass, Resource {
            name: "Grass".to_string(),
            description: "Organic surface material that promotes growth".to_string(),
            rarity: ResourceRarity::Common,
            durability: 10.0,
            properties: vec![ResourceProperty::Organic, ResourceProperty::Renewable],
        });

        self.resource_definitions.insert(ResourceType::Sand, Resource {
            name: "Sand".to_string(),
            description: "Fine granular material useful for glass production".to_string(),
            rarity: ResourceRarity::Common,
            durability: 30.0,
            properties: vec![ResourceProperty::Granular, ResourceProperty::Craftable],
        });

        // Advanced materials
        self.resource_definitions.insert(ResourceType::Metal, Resource {
            name: "Metal".to_string(),
            description: "Strong metallic material for advanced construction".to_string(),
            rarity: ResourceRarity::Uncommon,
            durability: 500.0,
            properties: vec![ResourceProperty::Metallic, ResourceProperty::Conductive, ResourceProperty::Structural],
        });

        self.resource_definitions.insert(ResourceType::Crystal, Resource {
            name: "Crystal".to_string(),
            description: "Rare crystalline material with unique energy properties".to_string(),
            rarity: ResourceRarity::Rare,
            durability: 800.0,
            properties: vec![ResourceProperty::Energetic, ResourceProperty::Rare, ResourceProperty::Luminous],
        });

        self.resource_definitions.insert(ResourceType::Wood, Resource {
            name: "Wood".to_string(),
            description: "Renewable organic building material".to_string(),
            rarity: ResourceRarity::Common,
            durability: 100.0,
            properties: vec![ResourceProperty::Organic, ResourceProperty::Renewable, ResourceProperty::Flammable],
        });

        // Initialize mining yields
        self.yield_multipliers.insert(ResourceType::Earth, 2);
        self.yield_multipliers.insert(ResourceType::Stone, 1);
        self.yield_multipliers.insert(ResourceType::Water, 3);
        self.yield_multipliers.insert(ResourceType::Grass, 1);
        self.yield_multipliers.insert(ResourceType::Sand, 2);
        self.yield_multipliers.insert(ResourceType::Metal, 1);
        self.yield_multipliers.insert(ResourceType::Crystal, 1);
        self.yield_multipliers.insert(ResourceType::Wood, 1);
    }

    /// Initialize resource conversion recipes
    fn initialize_conversions(&mut self) {
        // Sand to Glass
        self.conversions.push(ResourceConversion {
            inputs: vec![(ResourceType::Sand, 3)],
            outputs: vec![(ResourceType::Glass, 1)],
            experience_reward: 25,
            skill_requirement: None,
        });

        // Stone to Refined Stone
        self.conversions.push(ResourceConversion {
            inputs: vec![(ResourceType::Stone, 2)],
            outputs: vec![(ResourceType::RefinedStone, 1)],
            experience_reward: 15,
            skill_requirement: None,
        });

        // Metal alloys
        self.conversions.push(ResourceConversion {
            inputs: vec![(ResourceType::Metal, 2), (ResourceType::Crystal, 1)],
            outputs: vec![(ResourceType::EnhancedMetal, 1)],
            experience_reward: 100,
            skill_requirement: Some((crate::engine::gameplay::BuildingSkill::Crafting, 25)),
        });
    }

    /// Get mining yield for a resource type
    pub fn get_mining_yield(&self, resource_type: &ResourceType) -> u32 {
        self.yield_multipliers.get(resource_type).copied().unwrap_or(1)
    }

    /// Get resource definition
    pub fn get_resource(&self, resource_type: &ResourceType) -> Option<&Resource> {
        self.resource_definitions.get(resource_type)
    }

    /// Check if player has required resources
    pub fn has_resources(&self, player_data: &PlayerData, requirements: &[(ResourceType, u32)]) -> bool {
        requirements.iter().all(|(resource_type, amount)| {
            let item_id = resource_type.to_item_id();
            player_data.get_item_count(&item_id) >= *amount
        })
    }

    /// Consume resources from player inventory
    pub fn consume_resource(&self, player_data: &mut PlayerData, resource_type: &ResourceType, amount: u32) -> bool {
        let item_id = resource_type.to_item_id();
        player_data.remove_item(&item_id, amount)
    }

    /// Consume multiple resources
    pub fn consume_resources(&self, player_data: &mut PlayerData, requirements: &[(ResourceType, u32)]) -> bool {
        // First check if we have all required resources
        if !self.has_resources(player_data, requirements) {
            return false;
        }

        // Then consume them
        for (resource_type, amount) in requirements {
            if !self.consume_resource(player_data, resource_type, *amount) {
                // This shouldn't happen if has_resources returned true, but safety check
                return false;
            }
        }
        true
    }

    /// Get all available resource conversions
    pub fn get_available_conversions(&self, player_data: &PlayerData) -> Vec<&ResourceConversion> {
        self.conversions.iter()
            .filter(|conversion| self.has_resources(player_data, &conversion.inputs))
            .collect()
    }

    /// Perform resource conversion
    pub fn convert_resources(&self, conversion_index: usize, player_data: &mut PlayerData) -> RobinResult<u32> {
        if conversion_index >= self.conversions.len() {
            return Err(RobinError::InvalidInput("Invalid conversion index".to_string()));
        }

        let conversion = &self.conversions[conversion_index];

        // Check skill requirements
        if let Some((required_skill, required_level)) = &conversion.skill_requirement {
            // TODO: Check skill level when skill system is implemented
        }

        // Check and consume inputs
        if !self.consume_resources(player_data, &conversion.inputs) {
            return Err(RobinError::InsufficientResources("Not enough resources for conversion".to_string()));
        }

        // Add outputs
        for (resource_type, amount) in &conversion.outputs {
            let item_id = resource_type.to_item_id();
            player_data.add_item(&item_id, *amount);
        }

        Ok(conversion.experience_reward)
    }
}

/// Resource types corresponding to voxel materials
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    // Basic materials
    Earth,
    Stone,
    Water,
    Grass,
    Sand,

    // Advanced materials
    Metal,
    Crystal,
    Wood,
    Lava,

    // Processed materials
    Glass,
    RefinedStone,
    EnhancedMetal,
}

impl ResourceType {
    /// Convert from VoxelType to ResourceType
    pub fn from_voxel(voxel_type: VoxelType) -> Self {
        match voxel_type {
            VoxelType::Dirt => ResourceType::Earth,
            VoxelType::Stone => ResourceType::Stone,
            VoxelType::Water => ResourceType::Water,
            VoxelType::Grass => ResourceType::Grass,
            VoxelType::Sand => ResourceType::Sand,
            VoxelType::Wood => ResourceType::Wood,
            VoxelType::Leaves => ResourceType::Grass, // Similar to grass
            VoxelType::Crystal => ResourceType::Crystal,
            VoxelType::Lava => ResourceType::Lava,
            VoxelType::Glass => ResourceType::Glass,
            VoxelType::Metal => ResourceType::Metal,
            VoxelType::Brick => ResourceType::Stone, // Similar to stone
            VoxelType::Ice => ResourceType::Water, // Similar to water
            VoxelType::Obsidian => ResourceType::Stone, // Similar to stone
            VoxelType::Air => ResourceType::Earth, // Default fallback
        }
    }

    /// Convert to inventory item ID
    pub fn to_item_id(&self) -> String {
        match self {
            ResourceType::Earth => "resource_earth".to_string(),
            ResourceType::Stone => "resource_stone".to_string(),
            ResourceType::Water => "resource_water".to_string(),
            ResourceType::Grass => "resource_grass".to_string(),
            ResourceType::Sand => "resource_sand".to_string(),
            ResourceType::Metal => "resource_metal".to_string(),
            ResourceType::Crystal => "resource_crystal".to_string(),
            ResourceType::Wood => "resource_wood".to_string(),
            ResourceType::Lava => "resource_lava".to_string(),
            ResourceType::Glass => "resource_glass".to_string(),
            ResourceType::RefinedStone => "resource_refined_stone".to_string(),
            ResourceType::EnhancedMetal => "resource_enhanced_metal".to_string(),
        }
    }
}

/// Resource definition with properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub name: String,
    pub description: String,
    pub rarity: ResourceRarity,
    pub durability: f32,
    pub properties: Vec<ResourceProperty>,
}

/// Resource rarity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

/// Properties that resources can have
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceProperty {
    // Physical properties
    Durable,
    Soft,
    Liquid,
    Granular,
    Structural,

    // Material properties
    Organic,
    Metallic,
    Crystalline,

    // Functional properties
    Renewable,
    Flammable,
    Conductive,
    Energetic,
    Essential,
    Fertile,
    Luminous,

    // Gameplay properties
    Rare,
    Craftable,
}

/// Resource conversion recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConversion {
    pub inputs: Vec<(ResourceType, u32)>,
    pub outputs: Vec<(ResourceType, u32)>,
    pub experience_reward: u32,
    pub skill_requirement: Option<(crate::engine::gameplay::BuildingSkill, u32)>,
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}