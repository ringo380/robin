/*!
 * Objectives and Mission System for Robin Engine
 *
 * Provides structured goals, challenges, and guided progression
 * for the Engineer Build Mode gameplay experience.
 */

use crate::engine::{
    error::{RobinError, RobinResult},
    save_system::{PlayerData, GameProgress},
    world::VoxelType,
    math::Vec3,
    gameplay::{BuildingSkill, ObjectiveEvent},
};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, VecDeque};

/// Core objective management system
pub struct ObjectiveManager {
    /// Currently active objectives
    active_objectives: HashMap<String, Objective>,
    /// Completed objectives
    completed_objectives: HashMap<String, Objective>,
    /// Available objective templates
    objective_templates: HashMap<String, ObjectiveTemplate>,
    /// Objective progression chains
    progression_chains: Vec<ObjectiveChain>,
    /// Daily/weekly challenge objectives
    challenge_objectives: VecDeque<ChallengeObjective>,
}

impl ObjectiveManager {
    pub fn new() -> Self {
        let mut manager = Self {
            active_objectives: HashMap::new(),
            completed_objectives: HashMap::new(),
            objective_templates: HashMap::new(),
            progression_chains: Vec::new(),
            challenge_objectives: VecDeque::new(),
        };

        manager.initialize_objective_templates();
        manager.initialize_progression_chains();
        manager.initialize_starter_objectives();
        manager
    }

    /// Initialize objective templates
    fn initialize_objective_templates(&mut self) {
        // Beginner objectives
        self.add_template("first_build", ObjectiveTemplate {
            title: "First Steps".to_string(),
            description: "Place your first 10 voxel blocks".to_string(),
            objective_type: ObjectiveType::PlaceBlocks { count: 10, voxel_type: None },
            rewards: vec![
                ObjectiveReward::Experience(BuildingSkill::Construction, 100),
                ObjectiveReward::Items(vec![("resource_stone".to_string(), 20)]),
            ],
            difficulty: ObjectiveDifficulty::Beginner,
            estimated_time: 300, // 5 minutes
            unlock_condition: None,
        });

        self.add_template("basic_mining", ObjectiveTemplate {
            title: "Resource Gathering".to_string(),
            description: "Mine 25 stone blocks to gather building materials".to_string(),
            objective_type: ObjectiveType::MineBlocks { count: 25, voxel_type: Some(VoxelType::Stone) },
            rewards: vec![
                ObjectiveReward::Experience(BuildingSkill::Mining, 150),
                ObjectiveReward::UnlockRecipe("stone_pickaxe".to_string()),
            ],
            difficulty: ObjectiveDifficulty::Beginner,
            estimated_time: 600, // 10 minutes
            unlock_condition: Some(UnlockCondition::ObjectiveCompleted("first_build".to_string())),
        });

        self.add_template("first_craft", ObjectiveTemplate {
            title: "Tools of the Trade".to_string(),
            description: "Craft your first stone pickaxe".to_string(),
            objective_type: ObjectiveType::CraftItems { recipe_id: "stone_pickaxe".to_string(), count: 1 },
            rewards: vec![
                ObjectiveReward::Experience(BuildingSkill::Crafting, 200),
                ObjectiveReward::Items(vec![("resource_wood".to_string(), 10)]),
            ],
            difficulty: ObjectiveDifficulty::Beginner,
            estimated_time: 300, // 5 minutes
            unlock_condition: Some(UnlockCondition::ObjectiveCompleted("basic_mining".to_string())),
        });

        // Intermediate objectives
        self.add_template("structure_builder", ObjectiveTemplate {
            title: "Structural Engineer".to_string(),
            description: "Build a structure using at least 100 blocks".to_string(),
            objective_type: ObjectiveType::BuildStructure { min_blocks: 100, structure_type: None },
            rewards: vec![
                ObjectiveReward::Experience(BuildingSkill::Construction, 500),
                ObjectiveReward::Experience(BuildingSkill::Architecture, 300),
                ObjectiveReward::UnlockBlueprint("basic_house".to_string()),
            ],
            difficulty: ObjectiveDifficulty::Intermediate,
            estimated_time: 1800, // 30 minutes
            unlock_condition: Some(UnlockCondition::SkillLevel(BuildingSkill::Construction, 10)),
        });

        self.add_template("material_master", ObjectiveTemplate {
            title: "Material Mastery".to_string(),
            description: "Collect 10 units each of 5 different resource types".to_string(),
            objective_type: ObjectiveType::CollectResources {
                requirements: vec![
                    ("resource_stone".to_string(), 10),
                    ("resource_earth".to_string(), 10),
                    ("resource_wood".to_string(), 10),
                    ("resource_sand".to_string(), 10),
                    ("resource_metal".to_string(), 10),
                ],
            },
            rewards: vec![
                ObjectiveReward::Experience(BuildingSkill::ResourceManagement, 400),
                ObjectiveReward::Items(vec![("tool_resource_scanner".to_string(), 1)]),
            ],
            difficulty: ObjectiveDifficulty::Intermediate,
            estimated_time: 2400, // 40 minutes
            unlock_condition: Some(UnlockCondition::SkillLevel(BuildingSkill::Mining, 15)),
        });

        // Advanced objectives
        self.add_template("master_builder", ObjectiveTemplate {
            title: "Master Builder".to_string(),
            description: "Complete a complex structure with over 500 blocks using at least 3 different materials".to_string(),
            objective_type: ObjectiveType::AdvancedBuild {
                min_blocks: 500,
                min_materials: 3,
                complexity_score: 100,
            },
            rewards: vec![
                ObjectiveReward::Experience(BuildingSkill::Construction, 1000),
                ObjectiveReward::Experience(BuildingSkill::Architecture, 800),
                ObjectiveReward::Achievement("master_builder".to_string()),
                ObjectiveReward::UnlockContent("advanced_templates".to_string()),
            ],
            difficulty: ObjectiveDifficulty::Advanced,
            estimated_time: 7200, // 2 hours
            unlock_condition: Some(UnlockCondition::MultipleConditions(vec![
                UnlockCondition::SkillLevel(BuildingSkill::Construction, 25),
                UnlockCondition::SkillLevel(BuildingSkill::Architecture, 20),
                UnlockCondition::ObjectiveCompleted("structure_builder".to_string()),
            ])),
        });

        self.add_template("automation_engineer", ObjectiveTemplate {
            title: "Automation Engineer".to_string(),
            description: "Create a functioning automated system using logic components".to_string(),
            objective_type: ObjectiveType::CreateAutomation {
                logic_components: 5,
                system_complexity: 3,
            },
            rewards: vec![
                ObjectiveReward::Experience(BuildingSkill::Engineering, 1500),
                ObjectiveReward::UnlockContent("advanced_logic".to_string()),
                ObjectiveReward::Achievement("automation_master".to_string()),
            ],
            difficulty: ObjectiveDifficulty::Expert,
            estimated_time: 10800, // 3 hours
            unlock_condition: Some(UnlockCondition::SkillLevel(BuildingSkill::Engineering, 30)),
        });
    }

    /// Initialize objective progression chains
    fn initialize_progression_chains(&mut self) {
        // Beginner chain
        self.progression_chains.push(ObjectiveChain {
            name: "Builder's Journey".to_string(),
            description: "Learn the fundamentals of construction in Robin".to_string(),
            objectives: vec![
                "first_build".to_string(),
                "basic_mining".to_string(),
                "first_craft".to_string(),
            ],
            chain_reward: ObjectiveReward::Achievement("novice_builder".to_string()),
        });

        // Intermediate chain
        self.progression_chains.push(ObjectiveChain {
            name: "Architect's Path".to_string(),
            description: "Develop advanced building and design skills".to_string(),
            objectives: vec![
                "structure_builder".to_string(),
                "material_master".to_string(),
            ],
            chain_reward: ObjectiveReward::Achievement("skilled_architect".to_string()),
        });

        // Expert chain
        self.progression_chains.push(ObjectiveChain {
            name: "Engineering Mastery".to_string(),
            description: "Master the most advanced building techniques".to_string(),
            objectives: vec![
                "master_builder".to_string(),
                "automation_engineer".to_string(),
            ],
            chain_reward: ObjectiveReward::Achievement("legendary_engineer".to_string()),
        });
    }

    /// Initialize starter objectives for new players
    fn initialize_starter_objectives(&mut self) {
        self.activate_objective("first_build").ok();
    }

    /// Add an objective template
    fn add_template(&mut self, id: &str, template: ObjectiveTemplate) {
        self.objective_templates.insert(id.to_string(), template);
    }

    /// Activate an objective for the player
    pub fn activate_objective(&mut self, objective_id: &str) -> RobinResult<()> {
        if let Some(template) = self.objective_templates.get(objective_id) {
            let objective = Objective {
                id: objective_id.to_string(),
                title: template.title.clone(),
                description: template.description.clone(),
                objective_type: template.objective_type.clone(),
                status: ObjectiveStatus::Active,
                progress: 0,
                target: self.get_objective_target(&template.objective_type),
                rewards: template.rewards.clone(),
                start_time: std::time::SystemTime::now(),
                completion_time: None,
            };

            self.active_objectives.insert(objective_id.to_string(), objective);
            Ok(())
        } else {
            Err(RobinError::InvalidInput(format!("Unknown objective: {}", objective_id)))
        }
    }

    /// Get the target value for an objective type
    fn get_objective_target(&self, objective_type: &ObjectiveType) -> u32 {
        match objective_type {
            ObjectiveType::PlaceBlocks { count, .. } => *count,
            ObjectiveType::MineBlocks { count, .. } => *count,
            ObjectiveType::CraftItems { count, .. } => *count,
            ObjectiveType::BuildStructure { min_blocks, .. } => *min_blocks,
            ObjectiveType::CollectResources { requirements } => requirements.len() as u32,
            ObjectiveType::AdvancedBuild { min_blocks, .. } => *min_blocks,
            ObjectiveType::CreateAutomation { logic_components, .. } => *logic_components,
            ObjectiveType::ReachSkillLevel { target_level, .. } => *target_level,
            ObjectiveType::CompleteChallenge => 1,
        }
    }

    /// Update objectives based on player actions
    pub fn update(&mut self, _delta_time: f32, player_data: &mut PlayerData, progress: &mut GameProgress) -> RobinResult<()> {
        // Check for objective unlocks
        let unlockable_objectives = self.check_objective_unlocks(player_data);
        for objective_id in unlockable_objectives {
            self.activate_objective(&objective_id)?;
        }

        // Update progression chains
        self.update_progression_chains(player_data, progress)?;

        Ok(())
    }

    /// Handle gameplay events that might progress objectives
    pub fn handle_event(&mut self, event: ObjectiveEvent, player_data: &mut PlayerData) -> RobinResult<Vec<String>> {
        let mut completed_objectives = Vec::new();
        let mut rewards_to_award = Vec::new();

        for (objective_id, objective) in self.active_objectives.iter_mut() {
            if objective.status != ObjectiveStatus::Active {
                continue;
            }

            let progress_made = match (&objective.objective_type, &event) {
                (ObjectiveType::PlaceBlocks { voxel_type, .. }, ObjectiveEvent::VoxelPlaced { voxel_type: placed_type, .. }) => {
                    if voxel_type.is_none() || voxel_type.as_ref() == Some(placed_type) { 1 } else { 0 }
                }
                (ObjectiveType::MineBlocks { voxel_type, .. }, ObjectiveEvent::VoxelMined { voxel_type: mined_type, .. }) => {
                    if voxel_type.is_none() || voxel_type.as_ref() == Some(mined_type) { 1 } else { 0 }
                }
                (ObjectiveType::CraftItems { recipe_id, .. }, ObjectiveEvent::ItemCrafted { recipe_id: crafted_recipe }) => {
                    if recipe_id == crafted_recipe { 1 } else { 0 }
                }
                _ => 0,
            };

            if progress_made > 0 {
                objective.progress += progress_made;

                if objective.progress >= objective.target {
                    objective.status = ObjectiveStatus::Completed;
                    objective.completion_time = Some(std::time::SystemTime::now());

                    // Collect rewards to award later
                    rewards_to_award.push(objective.rewards.clone());

                    completed_objectives.push(objective_id.clone());
                }
            }
        }

        // Award rewards after the loop
        for rewards in rewards_to_award {
            self.award_objective_rewards(&rewards, player_data)?;
        }

        // Move completed objectives
        for objective_id in &completed_objectives {
            if let Some(objective) = self.active_objectives.remove(objective_id) {
                self.completed_objectives.insert(objective_id.clone(), objective);
            }
        }

        Ok(completed_objectives)
    }

    /// Award rewards from completed objectives
    fn award_objective_rewards(&self, rewards: &[ObjectiveReward], player_data: &mut PlayerData) -> RobinResult<()> {
        for reward in rewards {
            match reward {
                ObjectiveReward::Experience(skill, amount) => {
                    // Note: This would normally interact with the skill system
                    let stat_key = format!("{}_experience_bonus", skill.to_string().to_lowercase());
                    player_data.stats.custom_stats.entry(stat_key).and_modify(|v| *v += *amount as f64).or_insert(*amount as f64);
                }
                ObjectiveReward::Items(items) => {
                    for (item_id, quantity) in items {
                        player_data.add_item(item_id, *quantity);
                    }
                }
                ObjectiveReward::Achievement(achievement_id) => {
                    player_data.unlock_achievement(achievement_id);
                }
                ObjectiveReward::UnlockRecipe(recipe_id) => {
                    let unlock_key = format!("recipe_unlocked_{}", recipe_id);
                    player_data.stats.custom_stats.insert(unlock_key, 1.0);
                }
                ObjectiveReward::UnlockBlueprint(blueprint_id) => {
                    let unlock_key = format!("blueprint_unlocked_{}", blueprint_id);
                    player_data.stats.custom_stats.insert(unlock_key, 1.0);
                }
                ObjectiveReward::UnlockContent(content_id) => {
                    let unlock_key = format!("content_unlocked_{}", content_id);
                    player_data.stats.custom_stats.insert(unlock_key, 1.0);
                }
            }
        }
        Ok(())
    }

    /// Check which objectives can be unlocked based on current progress
    fn check_objective_unlocks(&self, player_data: &PlayerData) -> Vec<String> {
        let mut unlockable = Vec::new();

        for (objective_id, template) in &self.objective_templates {
            // Skip if already active or completed
            if self.active_objectives.contains_key(objective_id) ||
               self.completed_objectives.contains_key(objective_id) {
                continue;
            }

            if let Some(condition) = &template.unlock_condition {
                if self.check_unlock_condition(condition, player_data) {
                    unlockable.push(objective_id.clone());
                }
            }
        }

        unlockable
    }

    /// Check if an unlock condition is met
    fn check_unlock_condition(&self, condition: &UnlockCondition, player_data: &PlayerData) -> bool {
        match condition {
            UnlockCondition::ObjectiveCompleted(objective_id) => {
                self.completed_objectives.contains_key(objective_id)
            }
            UnlockCondition::SkillLevel(skill, level) => {
                let stat_key = format!("{}_level", skill.to_string().to_lowercase());
                player_data.stats.custom_stats.get(&stat_key).unwrap_or(&0.0) >= &(*level as f64)
            }
            UnlockCondition::ItemCount(item_id, count) => {
                player_data.get_item_count(item_id) >= *count
            }
            UnlockCondition::AchievementUnlocked(achievement_id) => {
                player_data.achievements.contains(achievement_id)
            }
            UnlockCondition::MultipleConditions(conditions) => {
                conditions.iter().all(|c| self.check_unlock_condition(c, player_data))
            }
        }
    }

    /// Update progression chains and award chain completion rewards
    fn update_progression_chains(&mut self, player_data: &mut PlayerData, _progress: &mut GameProgress) -> RobinResult<()> {
        for chain in &self.progression_chains {
            // Check if all objectives in the chain are completed
            let all_completed = chain.objectives.iter()
                .all(|obj_id| self.completed_objectives.contains_key(obj_id));

            if all_completed {
                // Check if chain reward hasn't been given yet
                let chain_reward_key = format!("chain_completed_{}", chain.name.replace(' ', "_").to_lowercase());
                if player_data.stats.custom_stats.get(&chain_reward_key).unwrap_or(&0.0) == &0.0 {
                    self.award_objective_rewards(&[chain.chain_reward.clone()], player_data)?;
                    player_data.stats.custom_stats.insert(chain_reward_key, 1.0);
                }
            }
        }

        Ok(())
    }

    /// Get all active objectives
    pub fn get_active_objectives(&self) -> Vec<&Objective> {
        self.active_objectives.values().collect()
    }

    /// Get completed objectives
    pub fn get_completed_objectives(&self) -> Vec<&Objective> {
        self.completed_objectives.values().collect()
    }

    /// Get progression chains
    pub fn get_progression_chains(&self) -> &Vec<ObjectiveChain> {
        &self.progression_chains
    }
}

/// A specific objective instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Objective {
    pub id: String,
    pub title: String,
    pub description: String,
    pub objective_type: ObjectiveType,
    pub status: ObjectiveStatus,
    pub progress: u32,
    pub target: u32,
    pub rewards: Vec<ObjectiveReward>,
    pub start_time: std::time::SystemTime,
    pub completion_time: Option<std::time::SystemTime>,
}

/// Template for creating objectives
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveTemplate {
    pub title: String,
    pub description: String,
    pub objective_type: ObjectiveType,
    pub rewards: Vec<ObjectiveReward>,
    pub difficulty: ObjectiveDifficulty,
    pub estimated_time: u32, // seconds
    pub unlock_condition: Option<UnlockCondition>,
}

/// Types of objectives players can complete
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectiveType {
    PlaceBlocks { count: u32, voxel_type: Option<VoxelType> },
    MineBlocks { count: u32, voxel_type: Option<VoxelType> },
    CraftItems { recipe_id: String, count: u32 },
    BuildStructure { min_blocks: u32, structure_type: Option<String> },
    CollectResources { requirements: Vec<(String, u32)> },
    AdvancedBuild { min_blocks: u32, min_materials: u32, complexity_score: u32 },
    CreateAutomation { logic_components: u32, system_complexity: u32 },
    ReachSkillLevel { skill: BuildingSkill, target_level: u32 },
    CompleteChallenge,
}

/// Current status of an objective
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectiveStatus {
    Locked,
    Active,
    Completed,
    Failed,
}

/// Difficulty levels for objectives
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectiveDifficulty {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
    Legendary,
}

/// Rewards that can be earned from objectives
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectiveReward {
    Experience(BuildingSkill, u32),
    Items(Vec<(String, u32)>),
    Achievement(String),
    UnlockRecipe(String),
    UnlockBlueprint(String),
    UnlockContent(String),
}

/// Conditions that must be met to unlock objectives
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnlockCondition {
    ObjectiveCompleted(String),
    SkillLevel(BuildingSkill, u32),
    ItemCount(String, u32),
    AchievementUnlocked(String),
    MultipleConditions(Vec<UnlockCondition>),
}

/// Chain of related objectives
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveChain {
    pub name: String,
    pub description: String,
    pub objectives: Vec<String>,
    pub chain_reward: ObjectiveReward,
}

/// Time-limited challenge objective
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeObjective {
    pub objective: Objective,
    pub deadline: std::time::SystemTime,
    pub bonus_rewards: Vec<ObjectiveReward>,
}

impl Default for ObjectiveManager {
    fn default() -> Self {
        Self::new()
    }
}