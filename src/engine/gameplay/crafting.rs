/*!
 * Crafting System for Robin Engine
 *
 * Advanced crafting system for creating tools, building components,
 * and special items that enhance the voxel building experience.
 */

use crate::engine::{
    error::{RobinError, RobinResult},
    save_system::PlayerData,
    gameplay::{resources::ResourceType, progression::SkillManager, BuildingSkill},
};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Core crafting management system
pub struct CraftingManager {
    /// Available crafting recipes
    recipes: HashMap<String, CraftingRecipe>,
    /// Recipe categories for organization
    categories: HashMap<String, Vec<String>>,
}

impl CraftingManager {
    pub fn new() -> Self {
        let mut manager = Self {
            recipes: HashMap::new(),
            categories: HashMap::new(),
        };

        manager.initialize_recipes();
        manager
    }

    /// Initialize all crafting recipes
    fn initialize_recipes(&mut self) {
        // Basic Tools
        self.add_recipe("stone_pickaxe", CraftingRecipe {
            name: "Stone Pickaxe".to_string(),
            description: "Basic mining tool for gathering resources efficiently".to_string(),
            category: "tools".to_string(),
            inputs: vec![
                (ResourceType::Stone, 3),
                (ResourceType::Wood, 2),
            ],
            outputs: vec![("tool_stone_pickaxe".to_string(), 1)],
            crafting_time: 5.0,
            experience_reward: 50,
            skill_requirement: None,
            unlock_condition: None,
        });

        self.add_recipe("metal_pickaxe", CraftingRecipe {
            name: "Metal Pickaxe".to_string(),
            description: "Advanced mining tool with increased durability and efficiency".to_string(),
            category: "tools".to_string(),
            inputs: vec![
                (ResourceType::Metal, 2),
                (ResourceType::Wood, 1),
            ],
            outputs: vec![("tool_metal_pickaxe".to_string(), 1)],
            crafting_time: 8.0,
            experience_reward: 100,
            skill_requirement: Some((BuildingSkill::Crafting, 10)),
            unlock_condition: Some(UnlockCondition::RecipeCrafted("stone_pickaxe".to_string())),
        });

        // Building Components
        self.add_recipe("reinforced_block", CraftingRecipe {
            name: "Reinforced Block".to_string(),
            description: "Extra durable building block for structural support".to_string(),
            category: "building".to_string(),
            inputs: vec![
                (ResourceType::Stone, 4),
                (ResourceType::Metal, 1),
            ],
            outputs: vec![("building_reinforced_block".to_string(), 2)],
            crafting_time: 3.0,
            experience_reward: 25,
            skill_requirement: Some((BuildingSkill::Construction, 5)),
            unlock_condition: None,
        });

        self.add_recipe("glass_panel", CraftingRecipe {
            name: "Glass Panel".to_string(),
            description: "Transparent building component for windows and decorative structures".to_string(),
            category: "building".to_string(),
            inputs: vec![
                (ResourceType::Glass, 2),
                (ResourceType::Metal, 1),
            ],
            outputs: vec![("building_glass_panel".to_string(), 3)],
            crafting_time: 4.0,
            experience_reward: 35,
            skill_requirement: Some((BuildingSkill::Crafting, 15)),
            unlock_condition: None,
        });

        // Advanced Materials
        self.add_recipe("composite_material", CraftingRecipe {
            name: "Composite Material".to_string(),
            description: "Advanced building material combining multiple resource properties".to_string(),
            category: "materials".to_string(),
            inputs: vec![
                (ResourceType::RefinedStone, 2),
                (ResourceType::Metal, 1),
                (ResourceType::Crystal, 1),
            ],
            outputs: vec![("material_composite".to_string(), 1)],
            crafting_time: 12.0,
            experience_reward: 200,
            skill_requirement: Some((BuildingSkill::Crafting, 30)),
            unlock_condition: Some(UnlockCondition::SkillLevel(BuildingSkill::Construction, 25)),
        });

        // Specialized Tools
        self.add_recipe("blueprint_scanner", CraftingRecipe {
            name: "Blueprint Scanner".to_string(),
            description: "Advanced tool for analyzing and copying building structures".to_string(),
            category: "advanced_tools".to_string(),
            inputs: vec![
                (ResourceType::Crystal, 3),
                (ResourceType::EnhancedMetal, 2),
                (ResourceType::Glass, 1),
            ],
            outputs: vec![("tool_blueprint_scanner".to_string(), 1)],
            crafting_time: 20.0,
            experience_reward: 500,
            skill_requirement: Some((BuildingSkill::Engineering, 40)),
            unlock_condition: Some(UnlockCondition::AchievementUnlocked("master_builder".to_string())),
        });

        // Initialize categories
        self.categories.insert("tools".to_string(), vec![
            "stone_pickaxe".to_string(),
            "metal_pickaxe".to_string(),
        ]);

        self.categories.insert("building".to_string(), vec![
            "reinforced_block".to_string(),
            "glass_panel".to_string(),
        ]);

        self.categories.insert("materials".to_string(), vec![
            "composite_material".to_string(),
        ]);

        self.categories.insert("advanced_tools".to_string(), vec![
            "blueprint_scanner".to_string(),
        ]);
    }

    /// Add a recipe to the system
    fn add_recipe(&mut self, id: &str, recipe: CraftingRecipe) {
        self.recipes.insert(id.to_string(), recipe);
    }

    /// Get all recipes available to player
    pub fn get_available_recipes(&self, player_data: &PlayerData, skills: &SkillManager) -> Vec<(&String, &CraftingRecipe)> {
        self.recipes.iter()
            .filter(|(_, recipe)| self.is_recipe_unlocked(recipe, player_data, skills))
            .collect()
    }

    /// Get recipes by category
    pub fn get_recipes_by_category(&self, category: &str, player_data: &PlayerData, skills: &SkillManager) -> Vec<(&String, &CraftingRecipe)> {
        if let Some(recipe_ids) = self.categories.get(category) {
            recipe_ids.iter()
                .filter_map(|id| self.recipes.get(id).map(|recipe| (id, recipe)))
                .filter(|(_, recipe)| self.is_recipe_unlocked(recipe, player_data, skills))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Check if a recipe is unlocked for the player
    fn is_recipe_unlocked(&self, recipe: &CraftingRecipe, player_data: &PlayerData, skills: &SkillManager) -> bool {
        // Check skill requirements
        if let Some((required_skill, required_level)) = &recipe.skill_requirement {
            if skills.get_skill_level(required_skill) < *required_level {
                return false;
            }
        }

        // Check unlock conditions
        if let Some(condition) = &recipe.unlock_condition {
            match condition {
                UnlockCondition::RecipeCrafted(recipe_id) => {
                    let stat_key = format!("recipe_crafted_{}", recipe_id);
                    player_data.stats.custom_stats.get(&stat_key).unwrap_or(&0.0) > &0.0
                }
                UnlockCondition::SkillLevel(skill, level) => {
                    skills.get_skill_level(skill) >= *level
                }
                UnlockCondition::AchievementUnlocked(achievement_id) => {
                    player_data.achievements.contains(achievement_id)
                }
                UnlockCondition::ItemsCollected(item_id, count) => {
                    player_data.get_item_count(item_id) >= *count
                }
            }
        } else {
            true
        }
    }

    /// Attempt to craft an item
    pub fn craft_item(&self, recipe_id: &str, player_data: &mut PlayerData, skills: &SkillManager) -> RobinResult<CraftingResult> {
        let recipe = self.recipes.get(recipe_id)
            .ok_or_else(|| RobinError::InvalidInput(format!("Unknown recipe: {}", recipe_id)))?;

        // Check if recipe is unlocked
        if !self.is_recipe_unlocked(recipe, player_data, skills) {
            return Ok(CraftingResult {
                success: false,
                message: "Recipe not unlocked".to_string(),
                items_created: Vec::new(),
                experience_gained: 0,
            });
        }

        // Check resource availability
        let mut missing_resources = Vec::new();
        for (resource_type, required_amount) in &recipe.inputs {
            let item_id = resource_type.to_item_id();
            let available = player_data.get_item_count(&item_id);
            if available < *required_amount {
                missing_resources.push(format!("{} (need {}, have {})",
                    resource_type.to_item_id(), required_amount, available));
            }
        }

        if !missing_resources.is_empty() {
            return Ok(CraftingResult {
                success: false,
                message: format!("Missing resources: {}", missing_resources.join(", ")),
                items_created: Vec::new(),
                experience_gained: 0,
            });
        }

        // Consume input resources
        for (resource_type, amount) in &recipe.inputs {
            let item_id = resource_type.to_item_id();
            if !player_data.remove_item(&item_id, *amount) {
                return Err(RobinError::InsufficientResources(format!("Failed to consume {}", item_id)));
            }
        }

        // Add output items
        let mut created_items = Vec::new();
        for (item_id, amount) in &recipe.outputs {
            player_data.add_item(item_id, *amount);
            created_items.push((item_id.clone(), *amount));
        }

        // Track crafting statistics
        let stat_key = format!("recipe_crafted_{}", recipe_id);
        player_data.stats.custom_stats.entry(stat_key).and_modify(|v| *v += 1.0).or_insert(1.0);

        Ok(CraftingResult {
            success: true,
            message: format!("Successfully crafted {}", recipe.name),
            items_created: created_items,
            experience_gained: recipe.experience_reward,
        })
    }

    /// Check if player can craft a specific recipe
    pub fn can_craft(&self, recipe_id: &str, player_data: &PlayerData, skills: &SkillManager) -> bool {
        if let Some(recipe) = self.recipes.get(recipe_id) {
            if !self.is_recipe_unlocked(recipe, player_data, skills) {
                return false;
            }

            recipe.inputs.iter().all(|(resource_type, required_amount)| {
                let item_id = resource_type.to_item_id();
                player_data.get_item_count(&item_id) >= *required_amount
            })
        } else {
            false
        }
    }

    /// Get recipe by ID
    pub fn get_recipe(&self, recipe_id: &str) -> Option<&CraftingRecipe> {
        self.recipes.get(recipe_id)
    }

    /// Get all category names
    pub fn get_categories(&self) -> Vec<&String> {
        self.categories.keys().collect()
    }
}

/// A crafting recipe definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CraftingRecipe {
    pub name: String,
    pub description: String,
    pub category: String,
    pub inputs: Vec<(ResourceType, u32)>,
    pub outputs: Vec<(String, u32)>,
    pub crafting_time: f32, // in seconds
    pub experience_reward: u32,
    pub skill_requirement: Option<(BuildingSkill, u32)>,
    pub unlock_condition: Option<UnlockCondition>,
}

/// Conditions that must be met to unlock a recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnlockCondition {
    RecipeCrafted(String),
    SkillLevel(BuildingSkill, u32),
    AchievementUnlocked(String),
    ItemsCollected(String, u32),
}

/// Result of a crafting attempt
#[derive(Debug, Clone)]
pub struct CraftingResult {
    pub success: bool,
    pub message: String,
    pub items_created: Vec<(String, u32)>,
    pub experience_gained: u32,
}

/// Legacy Recipe struct for compatibility
pub type Recipe = CraftingRecipe;

impl Default for CraftingManager {
    fn default() -> Self {
        Self::new()
    }
}