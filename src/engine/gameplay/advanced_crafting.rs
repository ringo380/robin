//! Advanced Crafting System for Robin Engine
//!
//! Multi-stage crafting, quality grades, tool requirements, recipe discovery,
//! and environmental conditions for sophisticated item creation.
//! Integrates with reputation system and character progression.

use crate::engine::{
    error::{RobinError, RobinResult},
    save_system::PlayerData,
    gameplay::{
        resources::ResourceType,
        progression::SkillManager,
        BuildingSkill,
        reputation::{ReputationManager, FactionId},
        crafting::UnlockCondition,
    },
    math::Vec3,
};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};

/// Advanced crafting system with multi-stage recipes and quality control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedCraftingManager {
    /// Advanced multi-stage recipes
    pub advanced_recipes: HashMap<String, AdvancedRecipe>,
    /// Recipe categories with difficulty ratings
    pub categories: HashMap<String, RecipeCategory>,
    /// Crafting stations and their capabilities
    pub crafting_stations: HashMap<String, CraftingStation>,
    /// Quality control system
    pub quality_system: QualityControlSystem,
    /// Recipe discovery tracking
    pub recipe_discovery: RecipeDiscoverySystem,
    /// Environmental crafting conditions
    pub environmental_system: EnvironmentalCraftingSystem,
    /// Tool requirements and effects
    pub tool_system: ToolRequirementSystem,
    /// Active crafting processes
    pub active_processes: HashMap<String, ActiveCraftingProcess>,
}

impl AdvancedCraftingManager {
    pub fn new() -> Self {
        let mut manager = Self {
            advanced_recipes: HashMap::new(),
            categories: HashMap::new(),
            crafting_stations: HashMap::new(),
            quality_system: QualityControlSystem::new(),
            recipe_discovery: RecipeDiscoverySystem::new(),
            environmental_system: EnvironmentalCraftingSystem::new(),
            tool_system: ToolRequirementSystem::new(),
            active_processes: HashMap::new(),
        };

        manager.initialize_advanced_recipes();
        manager.initialize_crafting_stations();
        manager.initialize_categories();
        manager
    }

    /// Initialize advanced multi-stage recipes
    fn initialize_advanced_recipes(&mut self) {
        // Master Craftsman Tools - Multi-stage Recipe
        self.advanced_recipes.insert("master_hammer".to_string(), AdvancedRecipe {
            id: "master_hammer".to_string(),
            name: "Master Craftsman's Hammer".to_string(),
            description: "Legendary crafting tool that improves success rates and unlocks master recipes".to_string(),
            category: "master_tools".to_string(),

            stages: vec![
                CraftingStage {
                    stage_id: "forge_head".to_string(),
                    name: "Forge Hammer Head".to_string(),
                    inputs: vec![
                        (ResourceType::EnhancedMetal, 3),
                        (ResourceType::Crystal, 1),
                    ],
                    outputs: vec![("component_hammer_head".to_string(), 1)],
                    required_station: Some("master_forge".to_string()),
                    crafting_time: 300.0, // 5 minutes
                    skill_requirement: Some((BuildingSkill::Crafting, 40)),
                    tool_requirements: vec!["tool_metal_tongs".to_string(), "tool_precision_hammer".to_string()],
                    environmental_conditions: vec![
                        EnvironmentalCondition::Temperature { min: 1200.0, max: 1400.0 },
                        EnvironmentalCondition::TimeOfDay { start: 6, end: 18 }, // Daylight hours
                    ],
                },
                CraftingStage {
                    stage_id: "craft_handle".to_string(),
                    name: "Craft Enchanted Handle".to_string(),
                    inputs: vec![
                        (ResourceType::Wood, 2),
                        (ResourceType::Crystal, 1),
                    ],
                    outputs: vec![("component_enchanted_handle".to_string(), 1)],
                    required_station: Some("enchanting_table".to_string()),
                    crafting_time: 180.0, // 3 minutes
                    skill_requirement: Some((BuildingSkill::Engineering, 30)),
                    tool_requirements: vec!["tool_precision_carver".to_string()],
                    environmental_conditions: vec![
                        EnvironmentalCondition::Location { area_type: "magical_workshop".to_string() },
                    ],
                },
                CraftingStage {
                    stage_id: "assembly".to_string(),
                    name: "Final Assembly".to_string(),
                    inputs: vec![
                        ("component_hammer_head".to_string(), 1),
                        ("component_enchanted_handle".to_string(), 1),
                    ],
                    outputs: vec![("tool_master_hammer".to_string(), 1)],
                    required_station: Some("master_workbench".to_string()),
                    crafting_time: 120.0, // 2 minutes
                    skill_requirement: Some((BuildingSkill::Crafting, 50)),
                    tool_requirements: vec!["tool_precision_assembly_kit".to_string()],
                    environmental_conditions: vec![],
                },
            ],

            total_experience_reward: 1500,
            quality_base_chance: 0.7, // 70% base chance for good quality
            discovery_method: Some(DiscoveryMethod::Experimentation {
                base_materials: vec![ResourceType::EnhancedMetal, ResourceType::Crystal, ResourceType::Wood],
                required_discoveries: 3,
            }),
            unlock_conditions: vec![
                UnlockCondition::SkillLevel(BuildingSkill::Crafting, 40),
                UnlockCondition::AchievementUnlocked("master_crafter".to_string()),
            ],
            reputation_requirements: vec![
                ReputationRequirement {
                    faction: "Crafters Consortium".to_string(),
                    minimum_standing: 750,
                },
            ],
            special_effects: vec![
                SpecialCraftingEffect::QualityBonus { bonus_percentage: 25.0 },
                SpecialCraftingEffect::SpeedBonus { bonus_percentage: 15.0 },
            ],
        });

        // Advanced Building Materials
        self.advanced_recipes.insert("reinforced_composite".to_string(), AdvancedRecipe {
            id: "reinforced_composite".to_string(),
            name: "Reinforced Composite Material".to_string(),
            description: "Ultra-strong building material combining multiple advanced components".to_string(),
            category: "advanced_materials".to_string(),

            stages: vec![
                CraftingStage {
                    stage_id: "prepare_matrix".to_string(),
                    name: "Prepare Polymer Matrix".to_string(),
                    inputs: vec![
                        (ResourceType::RefinedStone, 4),
                        (ResourceType::Crystal, 2),
                    ],
                    outputs: vec![("component_polymer_matrix".to_string(), 1)],
                    required_station: Some("chemical_processor".to_string()),
                    crafting_time: 240.0,
                    skill_requirement: Some((BuildingSkill::Engineering, 35)),
                    tool_requirements: vec!["tool_chemical_mixer".to_string()],
                    environmental_conditions: vec![
                        EnvironmentalCondition::Temperature { min: 200.0, max: 300.0 },
                        EnvironmentalCondition::Humidity { max: 30.0 },
                    ],
                },
                CraftingStage {
                    stage_id: "weave_fibers".to_string(),
                    name: "Weave Reinforcement Fibers".to_string(),
                    inputs: vec![
                        (ResourceType::EnhancedMetal, 3),
                        (ResourceType::Crystal, 1),
                    ],
                    outputs: vec![("component_reinforcement_fibers".to_string(), 1)],
                    required_station: Some("fiber_weaver".to_string()),
                    crafting_time: 180.0,
                    skill_requirement: Some((BuildingSkill::Crafting, 30)),
                    tool_requirements: vec!["tool_precision_weaver".to_string()],
                    environmental_conditions: vec![],
                },
                CraftingStage {
                    stage_id: "composite_fusion".to_string(),
                    name: "Composite Fusion Process".to_string(),
                    inputs: vec![
                        ("component_polymer_matrix".to_string(), 1),
                        ("component_reinforcement_fibers".to_string(), 1),
                    ],
                    outputs: vec![("material_reinforced_composite".to_string(), 3)],
                    required_station: Some("fusion_chamber".to_string()),
                    crafting_time: 360.0, // 6 minutes
                    skill_requirement: Some((BuildingSkill::Engineering, 45)),
                    tool_requirements: vec!["tool_fusion_controller".to_string()],
                    environmental_conditions: vec![
                        EnvironmentalCondition::Temperature { min: 800.0, max: 1000.0 },
                        EnvironmentalCondition::Location { area_type: "advanced_laboratory".to_string() },
                    ],
                },
            ],

            total_experience_reward: 800,
            quality_base_chance: 0.6,
            discovery_method: Some(DiscoveryMethod::Research {
                research_points_required: 500,
                prerequisite_recipes: vec!["composite_material".to_string()],
            }),
            unlock_conditions: vec![
                UnlockCondition::SkillLevel(BuildingSkill::Engineering, 35),
                UnlockCondition::RecipeCrafted("composite_material".to_string()),
            ],
            reputation_requirements: vec![
                ReputationRequirement {
                    faction: "Engineers Collective".to_string(),
                    minimum_standing: 500,
                },
            ],
            special_effects: vec![],
        });

        // Magical/Technological Fusion Recipe
        self.advanced_recipes.insert("arcane_processor".to_string(), AdvancedRecipe {
            id: "arcane_processor".to_string(),
            name: "Arcane Processing Unit".to_string(),
            description: "Mystical technology that automates and enhances crafting processes".to_string(),
            category: "automation_tools".to_string(),

            stages: vec![
                CraftingStage {
                    stage_id: "crystal_core".to_string(),
                    name: "Attune Crystal Core".to_string(),
                    inputs: vec![
                        (ResourceType::Crystal, 5),
                    ],
                    outputs: vec![("component_attuned_crystal_core".to_string(), 1)],
                    required_station: Some("crystal_attunement_altar".to_string()),
                    crafting_time: 600.0, // 10 minutes
                    skill_requirement: Some((BuildingSkill::Engineering, 50)),
                    tool_requirements: vec!["tool_crystal_resonator".to_string()],
                    environmental_conditions: vec![
                        EnvironmentalCondition::TimeOfDay { start: 22, end: 4 }, // Night time
                        EnvironmentalCondition::Weather { condition: "clear".to_string() },
                        EnvironmentalCondition::Location { area_type: "mystical_sanctuary".to_string() },
                    ],
                },
                CraftingStage {
                    stage_id: "tech_housing".to_string(),
                    name: "Fabricate Tech Housing".to_string(),
                    inputs: vec![
                        (ResourceType::EnhancedMetal, 4),
                        (ResourceType::Glass, 2),
                    ],
                    outputs: vec![("component_tech_housing".to_string(), 1)],
                    required_station: Some("precision_fabricator".to_string()),
                    crafting_time: 300.0,
                    skill_requirement: Some((BuildingSkill::Engineering, 40)),
                    tool_requirements: vec!["tool_precision_fabricator".to_string()],
                    environmental_conditions: vec![
                        EnvironmentalCondition::Temperature { min: 20.0, max: 25.0 }, // Room temperature
                        EnvironmentalCondition::Humidity { max: 40.0 },
                    ],
                },
                CraftingStage {
                    stage_id: "mystical_integration".to_string(),
                    name: "Mystical-Tech Integration".to_string(),
                    inputs: vec![
                        ("component_attuned_crystal_core".to_string(), 1),
                        ("component_tech_housing".to_string(), 1),
                        (ResourceType::EnhancedMetal, 2),
                    ],
                    outputs: vec![("tool_arcane_processor".to_string(), 1)],
                    required_station: Some("integration_nexus".to_string()),
                    crafting_time: 900.0, // 15 minutes
                    skill_requirement: Some((BuildingSkill::Engineering, 55)),
                    tool_requirements: vec![
                        "tool_mystical_conduit".to_string(),
                        "tool_tech_integrator".to_string(),
                    ],
                    environmental_conditions: vec![
                        EnvironmentalCondition::Location { area_type: "techno_mystical_lab".to_string() },
                        EnvironmentalCondition::Weather { condition: "thunderstorm".to_string() }, // Lightning energy
                    ],
                },
            ],

            total_experience_reward: 2500,
            quality_base_chance: 0.5, // Difficult recipe
            discovery_method: Some(DiscoveryMethod::QuestReward {
                quest_id: "arcane_mysteries".to_string(),
            }),
            unlock_conditions: vec![
                UnlockCondition::SkillLevel(BuildingSkill::Engineering, 50),
                UnlockCondition::AchievementUnlocked("tech_mystic".to_string()),
            ],
            reputation_requirements: vec![
                ReputationRequirement {
                    faction: "Engineers Collective".to_string(),
                    minimum_standing: 800,
                },
                ReputationRequirement {
                    faction: "Mystical Order".to_string(),
                    minimum_standing: 600,
                },
            ],
            special_effects: vec![
                SpecialCraftingEffect::AutomationUnlock { automation_type: "advanced_crafting".to_string() },
                SpecialCraftingEffect::QualityBonus { bonus_percentage: 50.0 },
            ],
        });
    }

    /// Initialize crafting stations with capabilities
    fn initialize_crafting_stations(&mut self) {
        self.crafting_stations.insert("master_forge".to_string(), CraftingStation {
            id: "master_forge".to_string(),
            name: "Master's Forge".to_string(),
            description: "Advanced forge capable of working with the highest grade materials".to_string(),
            station_type: StationType::MetalWorking,
            quality_bonus: 0.15,
            speed_bonus: 0.10,
            supported_categories: vec!["tools".to_string(), "weapons".to_string(), "master_tools".to_string()],
            required_fuel: Some("refined_coal".to_string()),
            fuel_consumption_rate: 2.0,
            maintenance_requirement: MaintenanceRequirement {
                resource_type: ResourceType::Metal,
                amount_per_hour: 1,
                max_durability: 1000.0,
                current_durability: 1000.0,
            },
        });

        self.crafting_stations.insert("enchanting_table".to_string(), CraftingStation {
            id: "enchanting_table".to_string(),
            name: "Enchanting Table".to_string(),
            description: "Mystical workstation for imbuing items with magical properties".to_string(),
            station_type: StationType::Mystical,
            quality_bonus: 0.20,
            speed_bonus: 0.05,
            supported_categories: vec!["enchanted_items".to_string(), "master_tools".to_string()],
            required_fuel: Some("mana_crystals".to_string()),
            fuel_consumption_rate: 1.5,
            maintenance_requirement: MaintenanceRequirement {
                resource_type: ResourceType::Crystal,
                amount_per_hour: 1,
                max_durability: 800.0,
                current_durability: 800.0,
            },
        });

        self.crafting_stations.insert("chemical_processor".to_string(), CraftingStation {
            id: "chemical_processor".to_string(),
            name: "Chemical Processing Unit".to_string(),
            description: "Advanced equipment for material synthesis and chemical reactions".to_string(),
            station_type: StationType::Chemical,
            quality_bonus: 0.10,
            speed_bonus: 0.20,
            supported_categories: vec!["advanced_materials".to_string(), "chemicals".to_string()],
            required_fuel: Some("energy_cells".to_string()),
            fuel_consumption_rate: 3.0,
            maintenance_requirement: MaintenanceRequirement {
                resource_type: ResourceType::EnhancedMetal,
                amount_per_hour: 2,
                max_durability: 1200.0,
                current_durability: 1200.0,
            },
        });

        self.crafting_stations.insert("fusion_chamber".to_string(), CraftingStation {
            id: "fusion_chamber".to_string(),
            name: "Material Fusion Chamber".to_string(),
            description: "Ultimate crafting station for fusing materials at the molecular level".to_string(),
            station_type: StationType::HighTech,
            quality_bonus: 0.25,
            speed_bonus: 0.15,
            supported_categories: vec!["fusion_materials".to_string(), "advanced_materials".to_string()],
            required_fuel: Some("fusion_cores".to_string()),
            fuel_consumption_rate: 5.0,
            maintenance_requirement: MaintenanceRequirement {
                resource_type: ResourceType::Crystal,
                amount_per_hour: 3,
                max_durability: 1500.0,
                current_durability: 1500.0,
            },
        });
    }

    /// Initialize recipe categories with metadata
    fn initialize_categories(&mut self) {
        self.categories.insert("master_tools".to_string(), RecipeCategory {
            name: "Master Tools".to_string(),
            description: "Legendary crafting tools for master artisans".to_string(),
            difficulty_rating: 5,
            unlock_level: 40,
            icon: "master_tools_icon".to_string(),
            color: [0.9, 0.7, 0.2], // Gold
        });

        self.categories.insert("advanced_materials".to_string(), RecipeCategory {
            name: "Advanced Materials".to_string(),
            description: "High-tech and mystical building materials".to_string(),
            difficulty_rating: 4,
            unlock_level: 30,
            icon: "advanced_materials_icon".to_string(),
            color: [0.3, 0.7, 0.9], // Blue
        });

        self.categories.insert("automation_tools".to_string(), RecipeCategory {
            name: "Automation Tools".to_string(),
            description: "Devices that automate and enhance crafting processes".to_string(),
            difficulty_rating: 5,
            unlock_level: 50,
            icon: "automation_icon".to_string(),
            color: [0.7, 0.3, 0.9], // Purple
        });
    }

    /// Update active crafting processes
    pub fn update(&mut self, delta_time: f32, player_data: &mut PlayerData, reputation: &ReputationManager) -> RobinResult<()> {
        let mut completed_processes = Vec::new();

        // Update active crafting processes
        for (process_id, process) in self.active_processes.iter_mut() {
            process.remaining_time -= delta_time;

            if process.remaining_time <= 0.0 {
                // Process completed
                completed_processes.push(process_id.clone());
            }
        }

        // Handle completed processes
        for process_id in completed_processes {
            if let Some(process) = self.active_processes.remove(&process_id) {
                self.complete_crafting_process(process, player_data, reputation)?;
            }
        }

        // Update recipe discovery system
        self.recipe_discovery.update(delta_time, player_data)?;

        // Update environmental conditions
        self.environmental_system.update(delta_time)?;

        Ok(())
    }

    /// Start a multi-stage advanced recipe
    pub fn start_advanced_recipe(&mut self,
                                recipe_id: &str,
                                player_data: &PlayerData,
                                skills: &SkillManager,
                                reputation: &ReputationManager,
                                station_id: Option<&str>) -> RobinResult<String> {
        let recipe = self.advanced_recipes.get(recipe_id)
            .ok_or_else(|| RobinError::NotFound(format!("Recipe '{}' not found", recipe_id)))?;

        // Check unlock conditions
        if !self.is_recipe_unlocked(recipe, player_data, skills, reputation) {
            return Err(RobinError::InvalidInput("Recipe not unlocked".to_string()));
        }

        // Validate first stage requirements
        let first_stage = &recipe.stages[0];
        self.validate_stage_requirements(first_stage, player_data, station_id)?;

        // Create new crafting process
        let process_id = format!("craft_{}_{}", recipe_id, chrono::Utc::now().timestamp());
        let process = ActiveCraftingProcess {
            process_id: process_id.clone(),
            recipe_id: recipe_id.to_string(),
            current_stage: 0,
            remaining_time: first_stage.crafting_time,
            station_id: station_id.map(|s| s.to_string()),
            quality_modifiers: self.calculate_quality_modifiers(first_stage, player_data, skills, station_id),
            intermediate_components: HashMap::new(),
            start_time: chrono::Utc::now(),
        };

        // Consume resources for first stage
        self.consume_stage_resources(first_stage, player_data)?;

        self.active_processes.insert(process_id.clone(), process);

        println!("🔧 Started advanced crafting: {} (Process: {})", recipe.name, process_id);
        Ok(process_id)
    }

    /// Check if advanced recipe is unlocked
    fn is_recipe_unlocked(&self,
                         recipe: &AdvancedRecipe,
                         player_data: &PlayerData,
                         skills: &SkillManager,
                         reputation: &ReputationManager) -> bool {
        // Check standard unlock conditions
        for condition in &recipe.unlock_conditions {
            match condition {
                UnlockCondition::SkillLevel(skill, level) => {
                    if skills.get_skill_level(skill) < *level {
                        return false;
                    }
                }
                UnlockCondition::AchievementUnlocked(achievement) => {
                    if !player_data.achievements.contains(achievement) {
                        return false;
                    }
                }
                UnlockCondition::RecipeCrafted(recipe_id) => {
                    let stat_key = format!("recipe_crafted_{}", recipe_id);
                    if player_data.stats.custom_stats.get(&stat_key).unwrap_or(&0.0) <= &0.0 {
                        return false;
                    }
                }
                _ => {}
            }
        }

        // Check reputation requirements
        for req in &recipe.reputation_requirements {
            if let Some(standing) = reputation.faction_standings.values()
                .find(|s| s.faction_id.name == req.faction) {
                if standing.reputation_value < req.minimum_standing {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Check discovery status
        if let Some(discovery_method) = &recipe.discovery_method {
            if !self.recipe_discovery.is_recipe_discovered(recipe.id.as_str()) {
                return false;
            }
        }

        true
    }

    /// Validate stage requirements (tools, station, environment)
    fn validate_stage_requirements(&self,
                                  stage: &CraftingStage,
                                  player_data: &PlayerData,
                                  station_id: Option<&str>) -> RobinResult<()> {
        // Check station requirement
        if let Some(required_station) = &stage.required_station {
            if let Some(provided_station) = station_id {
                if provided_station != required_station {
                    return Err(RobinError::InvalidInput(
                        format!("Wrong crafting station. Required: {}, Provided: {}",
                               required_station, provided_station)
                    ));
                }
            } else {
                return Err(RobinError::InvalidInput(
                    format!("Crafting station required: {}", required_station)
                ));
            }
        }

        // Check tool requirements
        for tool in &stage.tool_requirements {
            if player_data.get_item_count(tool) == 0 {
                return Err(RobinError::InsufficientResources(
                    format!("Required tool missing: {}", tool)
                ));
            }
        }

        // Check environmental conditions
        for condition in &stage.environmental_conditions {
            if !self.environmental_system.check_condition(condition) {
                return Err(RobinError::InvalidInput(
                    format!("Environmental condition not met: {:?}", condition)
                ));
            }
        }

        // Check resource availability
        for (resource_type, amount) in &stage.inputs {
            if let Ok(resource_type) = resource_type.to_item_id().parse::<ResourceType>() {
                let item_id = resource_type.to_item_id();
                if player_data.get_item_count(&item_id) < *amount {
                    return Err(RobinError::InsufficientResources(
                        format!("Insufficient {}: need {}, have {}",
                               item_id, amount, player_data.get_item_count(&item_id))
                    ));
                }
            } else {
                // Component from previous stage
                if player_data.get_item_count(resource_type) < *amount {
                    return Err(RobinError::InsufficientResources(
                        format!("Insufficient component: {}", resource_type)
                    ));
                }
            }
        }

        Ok(())
    }

    /// Consume resources for a crafting stage
    fn consume_stage_resources(&self, stage: &CraftingStage, player_data: &mut PlayerData) -> RobinResult<()> {
        for (resource_id, amount) in &stage.inputs {
            if !player_data.remove_item(resource_id, *amount) {
                return Err(RobinError::InsufficientResources(
                    format!("Failed to consume {}", resource_id)
                ));
            }
        }
        Ok(())
    }

    /// Calculate quality modifiers for a crafting process
    fn calculate_quality_modifiers(&self,
                                  stage: &CraftingStage,
                                  player_data: &PlayerData,
                                  skills: &SkillManager,
                                  station_id: Option<&str>) -> QualityModifiers {
        let mut modifiers = QualityModifiers {
            skill_bonus: 0.0,
            tool_bonus: 0.0,
            station_bonus: 0.0,
            environmental_bonus: 0.0,
            reputation_bonus: 0.0,
        };

        // Skill-based bonus
        if let Some((skill, _)) = &stage.skill_requirement {
            let skill_level = skills.get_skill_level(skill);
            modifiers.skill_bonus = (skill_level as f32 - 20.0) * 0.01; // 1% per level above 20
        }

        // Station bonus
        if let Some(station_id) = station_id {
            if let Some(station) = self.crafting_stations.get(station_id) {
                modifiers.station_bonus = station.quality_bonus;
            }
        }

        // Tool bonus
        for tool in &stage.tool_requirements {
            if player_data.get_item_count(tool) > 0 {
                modifiers.tool_bonus += self.tool_system.get_tool_quality_bonus(tool);
            }
        }

        // Environmental bonus
        for condition in &stage.environmental_conditions {
            if self.environmental_system.check_condition(condition) {
                modifiers.environmental_bonus += 0.05; // 5% per perfect condition
            }
        }

        modifiers
    }

    /// Complete a crafting process and award results
    fn complete_crafting_process(&mut self,
                                process: ActiveCraftingProcess,
                                player_data: &mut PlayerData,
                                reputation: &ReputationManager) -> RobinResult<()> {
        let recipe = self.advanced_recipes.get(&process.recipe_id).unwrap();
        let current_stage = &recipe.stages[process.current_stage];

        // Add stage outputs
        for (item_id, amount) in &current_stage.outputs {
            player_data.add_item(item_id, *amount);
        }

        // Check if recipe is complete
        if process.current_stage + 1 >= recipe.stages.len() {
            // Recipe completed - apply final quality and bonuses
            let final_quality = self.quality_system.determine_final_quality(
                recipe.quality_base_chance,
                &process.quality_modifiers,
                player_data
            );

            // Apply quality modifiers to final output
            self.apply_quality_to_final_output(recipe, final_quality, player_data);

            // Award experience
            let experience_bonus = self.calculate_experience_bonus(&process.quality_modifiers);
            let total_experience = (recipe.total_experience_reward as f32 * (1.0 + experience_bonus)) as u32;

            // Award experience to appropriate skills
            if let Some((skill, _)) = recipe.stages.last().and_then(|s| s.skill_requirement.as_ref()) {
                // Award to skill manager (would need to integrate with SkillManager)
                player_data.stats.custom_stats.insert(
                    format!("experience_{:?}", skill),
                    player_data.stats.custom_stats.get(&format!("experience_{:?}", skill)).unwrap_or(&0.0) + total_experience as f32
                );
            }

            // Track completion
            let completion_key = format!("advanced_recipe_completed_{}", recipe.id);
            player_data.stats.custom_stats.entry(completion_key).and_modify(|v| *v += 1.0).or_insert(1.0);

            println!("✨ Completed advanced recipe: {} with quality: {:?}", recipe.name, final_quality);

        } else {
            // Continue to next stage
            let next_stage_index = process.current_stage + 1;
            let next_stage = &recipe.stages[next_stage_index];

            // Create new process for next stage
            let next_process = ActiveCraftingProcess {
                process_id: format!("{}_stage_{}", process.process_id, next_stage_index),
                recipe_id: process.recipe_id,
                current_stage: next_stage_index,
                remaining_time: next_stage.crafting_time,
                station_id: process.station_id, // May need to change station
                quality_modifiers: process.quality_modifiers, // Carry forward modifiers
                intermediate_components: process.intermediate_components,
                start_time: chrono::Utc::now(),
            };

            self.active_processes.insert(next_process.process_id.clone(), next_process);
            println!("🔄 Advanced recipe {} proceeding to stage {}", recipe.name, next_stage_index + 1);
        }

        Ok(())
    }

    /// Apply quality modifiers to final crafted output
    fn apply_quality_to_final_output(&self, recipe: &AdvancedRecipe, quality: ItemQuality, player_data: &mut PlayerData) {
        // Get the final output from the last stage
        if let Some(last_stage) = recipe.stages.last() {
            for (item_id, base_amount) in &last_stage.outputs {
                // Remove base amount and add quality-modified amount
                player_data.remove_item(item_id, *base_amount);

                let quality_item_id = format!("{}_{:?}", item_id, quality).to_lowercase();
                let bonus_amount = match quality {
                    ItemQuality::Poor => 0,
                    ItemQuality::Common => 0,
                    ItemQuality::Uncommon => 1,
                    ItemQuality::Rare => 2,
                    ItemQuality::Epic => 3,
                    ItemQuality::Legendary => 5,
                };

                player_data.add_item(&quality_item_id, base_amount + bonus_amount);

                // Track quality crafting statistics
                let quality_stat_key = format!("quality_crafted_{:?}", quality);
                player_data.stats.custom_stats.entry(quality_stat_key).and_modify(|v| *v += 1.0).or_insert(1.0);
            }
        }
    }

    /// Calculate experience bonus based on quality modifiers
    fn calculate_experience_bonus(&self, modifiers: &QualityModifiers) -> f32 {
        (modifiers.skill_bonus + modifiers.tool_bonus + modifiers.station_bonus +
         modifiers.environmental_bonus + modifiers.reputation_bonus).max(0.0).min(2.0) // Cap at 200% bonus
    }

    /// Get available advanced recipes for player
    pub fn get_available_advanced_recipes(&self,
                                        player_data: &PlayerData,
                                        skills: &SkillManager,
                                        reputation: &ReputationManager) -> Vec<&AdvancedRecipe> {
        self.advanced_recipes.values()
            .filter(|recipe| self.is_recipe_unlocked(recipe, player_data, skills, reputation))
            .collect()
    }

    /// Get recipes by category
    pub fn get_advanced_recipes_by_category(&self,
                                          category: &str,
                                          player_data: &PlayerData,
                                          skills: &SkillManager,
                                          reputation: &ReputationManager) -> Vec<&AdvancedRecipe> {
        self.advanced_recipes.values()
            .filter(|recipe| recipe.category == category)
            .filter(|recipe| self.is_recipe_unlocked(recipe, player_data, skills, reputation))
            .collect()
    }

    /// Get active crafting processes
    pub fn get_active_processes(&self) -> &HashMap<String, ActiveCraftingProcess> {
        &self.active_processes
    }

    /// Get crafting station information
    pub fn get_crafting_station(&self, station_id: &str) -> Option<&CraftingStation> {
        self.crafting_stations.get(station_id)
    }

    /// Cancel an active crafting process
    pub fn cancel_crafting_process(&mut self, process_id: &str) -> RobinResult<()> {
        if let Some(_process) = self.active_processes.remove(process_id) {
            // TODO: Optionally return some resources
            println!("❌ Cancelled crafting process: {}", process_id);
            Ok(())
        } else {
            Err(RobinError::NotFound(format!("Crafting process '{}' not found", process_id)))
        }
    }

    /// Attempt recipe discovery through experimentation
    pub fn attempt_recipe_discovery(&mut self,
                                   materials: &[ResourceType],
                                   player_data: &mut PlayerData,
                                   skills: &SkillManager) -> RobinResult<Option<String>> {
        self.recipe_discovery.attempt_discovery(materials, player_data, skills, &self.advanced_recipes)
    }

    /// Get recipe discovery progress
    pub fn get_discovery_progress(&self, recipe_id: &str) -> Option<&DiscoveryProgress> {
        self.recipe_discovery.get_progress(recipe_id)
    }
}

// Supporting types and systems...

/// Advanced multi-stage recipe definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedRecipe {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub stages: Vec<CraftingStage>,
    pub total_experience_reward: u32,
    pub quality_base_chance: f32,
    pub discovery_method: Option<DiscoveryMethod>,
    pub unlock_conditions: Vec<UnlockCondition>,
    pub reputation_requirements: Vec<ReputationRequirement>,
    pub special_effects: Vec<SpecialCraftingEffect>,
}

/// Individual stage in a multi-stage recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CraftingStage {
    pub stage_id: String,
    pub name: String,
    pub inputs: Vec<(String, u32)>, // Can be resources or components
    pub outputs: Vec<(String, u32)>,
    pub required_station: Option<String>,
    pub crafting_time: f32,
    pub skill_requirement: Option<(BuildingSkill, u32)>,
    pub tool_requirements: Vec<String>,
    pub environmental_conditions: Vec<EnvironmentalCondition>,
}

/// Environmental conditions required for crafting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnvironmentalCondition {
    Temperature { min: f32, max: f32 },
    Humidity { max: f32 },
    TimeOfDay { start: u8, end: u8 }, // Hours 0-23
    Weather { condition: String },
    Location { area_type: String },
}

/// Methods for discovering new recipes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryMethod {
    Experimentation {
        base_materials: Vec<ResourceType>,
        required_discoveries: u32,
    },
    Research {
        research_points_required: u32,
        prerequisite_recipes: Vec<String>,
    },
    QuestReward {
        quest_id: String,
    },
    TeacherUnlock {
        teacher_npc: String,
        reputation_required: i32,
    },
}

/// Reputation requirements for recipes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationRequirement {
    pub faction: String,
    pub minimum_standing: i32,
}

/// Special effects that recipes can provide
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpecialCraftingEffect {
    QualityBonus { bonus_percentage: f32 },
    SpeedBonus { bonus_percentage: f32 },
    AutomationUnlock { automation_type: String },
    RecipeUnlock { recipe_id: String },
}

/// Crafting station definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CraftingStation {
    pub id: String,
    pub name: String,
    pub description: String,
    pub station_type: StationType,
    pub quality_bonus: f32,
    pub speed_bonus: f32,
    pub supported_categories: Vec<String>,
    pub required_fuel: Option<String>,
    pub fuel_consumption_rate: f32,
    pub maintenance_requirement: MaintenanceRequirement,
}

/// Types of crafting stations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StationType {
    BasicWorkbench,
    MetalWorking,
    Chemical,
    Mystical,
    HighTech,
    Biological,
}

/// Maintenance requirements for stations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceRequirement {
    pub resource_type: ResourceType,
    pub amount_per_hour: u32,
    pub max_durability: f32,
    pub current_durability: f32,
}

/// Recipe category metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeCategory {
    pub name: String,
    pub description: String,
    pub difficulty_rating: u32, // 1-5
    pub unlock_level: u32,
    pub icon: String,
    pub color: [f32; 3], // RGB color
}

/// Active crafting process tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveCraftingProcess {
    pub process_id: String,
    pub recipe_id: String,
    pub current_stage: usize,
    pub remaining_time: f32,
    pub station_id: Option<String>,
    pub quality_modifiers: QualityModifiers,
    pub intermediate_components: HashMap<String, u32>,
    pub start_time: DateTime<Utc>,
}

/// Quality modifiers affecting crafting outcome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityModifiers {
    pub skill_bonus: f32,
    pub tool_bonus: f32,
    pub station_bonus: f32,
    pub environmental_bonus: f32,
    pub reputation_bonus: f32,
}

/// Item quality levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemQuality {
    Poor,
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

/// Quality control system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityControlSystem {
    quality_thresholds: Vec<f32>,
}

impl QualityControlSystem {
    pub fn new() -> Self {
        Self {
            quality_thresholds: vec![0.1, 0.3, 0.5, 0.7, 0.85, 1.0], // Poor to Legendary
        }
    }

    pub fn determine_final_quality(&self, base_chance: f32, modifiers: &QualityModifiers, _player_data: &PlayerData) -> ItemQuality {
        let total_bonus = modifiers.skill_bonus + modifiers.tool_bonus +
                         modifiers.station_bonus + modifiers.environmental_bonus +
                         modifiers.reputation_bonus;

        let final_chance = (base_chance + total_bonus).clamp(0.0, 1.0);
        let random_roll = rand::random::<f32>();

        if random_roll < final_chance {
            // Determine quality tier based on how much the roll exceeded thresholds
            let quality_roll = rand::random::<f32>() + total_bonus * 0.5;

            for (i, &threshold) in self.quality_thresholds.iter().enumerate().rev() {
                if quality_roll >= threshold {
                    return match i {
                        0 => ItemQuality::Poor,
                        1 => ItemQuality::Common,
                        2 => ItemQuality::Uncommon,
                        3 => ItemQuality::Rare,
                        4 => ItemQuality::Epic,
                        5 => ItemQuality::Legendary,
                        _ => ItemQuality::Common,
                    };
                }
            }
        }

        ItemQuality::Common
    }
}

/// Recipe discovery system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeDiscoverySystem {
    discovered_recipes: HashMap<String, DateTime<Utc>>,
    discovery_progress: HashMap<String, DiscoveryProgress>,
    experimentation_history: Vec<ExperimentationAttempt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryProgress {
    pub recipe_id: String,
    pub method: DiscoveryMethod,
    pub progress: f32, // 0.0 to 1.0
    pub discoveries_made: u32,
    pub last_attempt: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentationAttempt {
    pub materials_used: Vec<ResourceType>,
    pub timestamp: DateTime<Utc>,
    pub result: ExperimentationResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExperimentationResult {
    Failure,
    PartialSuccess { hints: Vec<String> },
    RecipeDiscovered { recipe_id: String },
}

impl RecipeDiscoverySystem {
    pub fn new() -> Self {
        Self {
            discovered_recipes: HashMap::new(),
            discovery_progress: HashMap::new(),
            experimentation_history: Vec::new(),
        }
    }

    pub fn update(&mut self, _delta_time: f32, _player_data: &mut PlayerData) -> RobinResult<()> {
        // Update discovery progress over time
        // Research-based discoveries could progress passively
        Ok(())
    }

    pub fn is_recipe_discovered(&self, recipe_id: &str) -> bool {
        self.discovered_recipes.contains_key(recipe_id)
    }

    pub fn get_progress(&self, recipe_id: &str) -> Option<&DiscoveryProgress> {
        self.discovery_progress.get(recipe_id)
    }

    pub fn attempt_discovery(&mut self,
                           materials: &[ResourceType],
                           player_data: &mut PlayerData,
                           skills: &SkillManager,
                           recipes: &HashMap<String, AdvancedRecipe>) -> RobinResult<Option<String>> {
        // Find matching experimentation-based recipes
        for (recipe_id, recipe) in recipes {
            if let Some(DiscoveryMethod::Experimentation { base_materials, required_discoveries }) = &recipe.discovery_method {
                if materials.iter().all(|m| base_materials.contains(m)) && materials.len() >= base_materials.len() {
                    // Valid experimentation attempt
                    let progress = self.discovery_progress.entry(recipe_id.clone())
                        .or_insert_with(|| DiscoveryProgress {
                            recipe_id: recipe_id.clone(),
                            method: recipe.discovery_method.clone().unwrap(),
                            progress: 0.0,
                            discoveries_made: 0,
                            last_attempt: Utc::now(),
                        });

                    progress.discoveries_made += 1;
                    progress.last_attempt = Utc::now();

                    // Calculate success chance based on skill and attempts
                    let skill_bonus = skills.get_skill_level(&BuildingSkill::Crafting) as f32 * 0.01;
                    let attempt_bonus = (progress.discoveries_made as f32 / *required_discoveries as f32) * 0.5;
                    let success_chance = 0.1 + skill_bonus + attempt_bonus;

                    if rand::random::<f32>() < success_chance {
                        // Recipe discovered!
                        self.discovered_recipes.insert(recipe_id.clone(), Utc::now());
                        progress.progress = 1.0;

                        // Track discovery in player stats
                        player_data.stats.custom_stats.entry("recipes_discovered".to_string())
                            .and_modify(|v| *v += 1.0).or_insert(1.0);

                        return Ok(Some(recipe_id.clone()));
                    } else {
                        // Partial progress
                        progress.progress = (progress.discoveries_made as f32 / *required_discoveries as f32).min(0.9);
                    }
                }
            }
        }

        // Record experimentation attempt
        self.experimentation_history.push(ExperimentationAttempt {
            materials_used: materials.to_vec(),
            timestamp: Utc::now(),
            result: ExperimentationResult::Failure,
        });

        Ok(None)
    }
}

/// Environmental crafting system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentalCraftingSystem {
    current_temperature: f32,
    current_humidity: f32,
    current_time: u8, // Hour 0-23
    current_weather: String,
    current_location: String,
}

impl EnvironmentalCraftingSystem {
    pub fn new() -> Self {
        Self {
            current_temperature: 20.0, // Room temperature
            current_humidity: 50.0,
            current_time: 12, // Noon
            current_weather: "clear".to_string(),
            current_location: "workshop".to_string(),
        }
    }

    pub fn update(&mut self, _delta_time: f32) -> RobinResult<()> {
        // Simulate environmental changes
        // This would integrate with world/weather systems
        Ok(())
    }

    pub fn check_condition(&self, condition: &EnvironmentalCondition) -> bool {
        match condition {
            EnvironmentalCondition::Temperature { min, max } => {
                self.current_temperature >= *min && self.current_temperature <= *max
            }
            EnvironmentalCondition::Humidity { max } => {
                self.current_humidity <= *max
            }
            EnvironmentalCondition::TimeOfDay { start, end } => {
                if start <= end {
                    self.current_time >= *start && self.current_time <= *end
                } else {
                    // Wraps around midnight
                    self.current_time >= *start || self.current_time <= *end
                }
            }
            EnvironmentalCondition::Weather { condition } => {
                self.current_weather == *condition
            }
            EnvironmentalCondition::Location { area_type } => {
                self.current_location == *area_type
            }
        }
    }

    pub fn set_location(&mut self, location: String) {
        self.current_location = location;
    }

    pub fn set_weather(&mut self, weather: String) {
        self.current_weather = weather;
    }
}

/// Tool requirement system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRequirementSystem {
    tool_quality_bonuses: HashMap<String, f32>,
}

impl ToolRequirementSystem {
    pub fn new() -> Self {
        let mut tool_bonuses = HashMap::new();

        // Basic tools
        tool_bonuses.insert("tool_stone_pickaxe".to_string(), 0.05);
        tool_bonuses.insert("tool_metal_pickaxe".to_string(), 0.10);

        // Precision tools
        tool_bonuses.insert("tool_precision_hammer".to_string(), 0.15);
        tool_bonuses.insert("tool_precision_carver".to_string(), 0.12);
        tool_bonuses.insert("tool_precision_weaver".to_string(), 0.18);

        // Advanced tools
        tool_bonuses.insert("tool_master_hammer".to_string(), 0.25);
        tool_bonuses.insert("tool_arcane_processor".to_string(), 0.50);

        // Specialized tools
        tool_bonuses.insert("tool_chemical_mixer".to_string(), 0.20);
        tool_bonuses.insert("tool_fusion_controller".to_string(), 0.30);
        tool_bonuses.insert("tool_crystal_resonator".to_string(), 0.35);

        Self {
            tool_quality_bonuses: tool_bonuses,
        }
    }

    pub fn get_tool_quality_bonus(&self, tool_id: &str) -> f32 {
        self.tool_quality_bonuses.get(tool_id).copied().unwrap_or(0.0)
    }
}

/// Result of an advanced crafting operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CraftingResult {
    pub completed: bool,
    pub current_stage: u32,
    pub total_stages: u32,
    pub item_id: Option<String>,
    pub quality_achieved: Option<ItemQuality>,
    pub experience_gained: Option<u32>,
    pub time_remaining: Option<f32>,
    pub next_stage_requirements: Option<Vec<String>>,
    pub errors: Vec<String>,
}

impl Default for AdvancedCraftingManager {
    fn default() -> Self {
        Self::new()
    }
}