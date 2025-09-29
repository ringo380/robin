/*!
 * Skill Progression System for Robin Engine
 *
 * Tracks player proficiency in building, crafting, mining, and engineering
 * skills with experience points, levels, and mastery bonuses.
 */

use crate::engine::{
    error::RobinResult,
    save_system::PlayerData,
};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// Import the enhanced skill tree system
use super::skill_tree::{EnhancedSkillManager, SpecializationPath, SkillAllocationResult, SpecializationSummary};

/// Core skill management system with enhanced skill trees
pub struct SkillManager {
    /// Current skill levels and experience (legacy system)
    skills: HashMap<BuildingSkill, SkillLevel>,
    /// Experience multipliers for different activities
    experience_multipliers: HashMap<BuildingSkill, f32>,
    /// Mastery bonuses unlocked at certain levels
    mastery_bonuses: HashMap<BuildingSkill, Vec<MasteryBonus>>,
    /// Enhanced skill tree system with specializations
    enhanced_skills: EnhancedSkillManager,
}

impl SkillManager {
    pub fn new() -> Self {
        let mut manager = Self {
            skills: HashMap::new(),
            experience_multipliers: HashMap::new(),
            mastery_bonuses: HashMap::new(),
            enhanced_skills: EnhancedSkillManager::new(),
        };

        manager.initialize_skills();
        manager.initialize_mastery_bonuses();
        manager
    }

    /// Initialize all skills with default values
    fn initialize_skills(&mut self) {
        let skills = [
            BuildingSkill::Construction,
            BuildingSkill::Mining,
            BuildingSkill::Crafting,
            BuildingSkill::Engineering,
            BuildingSkill::Architecture,
            BuildingSkill::ResourceManagement,
        ];

        for skill in skills {
            self.skills.insert(skill, SkillLevel {
                level: 1,
                experience: 0,
                total_experience: 0,
            });

            // Set experience multipliers
            let multiplier = match skill {
                BuildingSkill::Construction => 1.0,
                BuildingSkill::Mining => 1.2, // Slightly faster to level
                BuildingSkill::Crafting => 0.8, // Slower, more valuable
                BuildingSkill::Engineering => 0.6, // Very slow, very valuable
                BuildingSkill::Architecture => 0.7,
                BuildingSkill::ResourceManagement => 1.1,
            };
            self.experience_multipliers.insert(skill, multiplier);
        }
    }

    /// Initialize mastery bonuses for each skill
    fn initialize_mastery_bonuses(&mut self) {
        // Construction mastery bonuses
        self.mastery_bonuses.insert(BuildingSkill::Construction, vec![
            MasteryBonus {
                unlock_level: 10,
                name: "Efficient Builder".to_string(),
                description: "Place blocks 10% faster".to_string(),
                bonus_type: BonusType::SpeedIncrease(0.1),
            },
            MasteryBonus {
                unlock_level: 25,
                name: "Structural Expert".to_string(),
                description: "Buildings have 20% more durability".to_string(),
                bonus_type: BonusType::DurabilityIncrease(0.2),
            },
            MasteryBonus {
                unlock_level: 50,
                name: "Master Constructor".to_string(),
                description: "Unlock advanced building templates".to_string(),
                bonus_type: BonusType::UnlockContent("advanced_templates".to_string()),
            },
        ]);

        // Mining mastery bonuses
        self.mastery_bonuses.insert(BuildingSkill::Mining, vec![
            MasteryBonus {
                unlock_level: 10,
                name: "Efficient Miner".to_string(),
                description: "10% chance for double resource yield".to_string(),
                bonus_type: BonusType::YieldIncrease(0.1),
            },
            MasteryBonus {
                unlock_level: 25,
                name: "Resource Prospector".to_string(),
                description: "Can identify rare resource deposits".to_string(),
                bonus_type: BonusType::UnlockAbility("rare_detection".to_string()),
            },
            MasteryBonus {
                unlock_level: 50,
                name: "Master Miner".to_string(),
                description: "25% chance for bonus rare materials".to_string(),
                bonus_type: BonusType::RareResourceBonus(0.25),
            },
        ]);

        // Crafting mastery bonuses
        self.mastery_bonuses.insert(BuildingSkill::Crafting, vec![
            MasteryBonus {
                unlock_level: 15,
                name: "Resourceful Crafter".to_string(),
                description: "5% chance to save materials when crafting".to_string(),
                bonus_type: BonusType::MaterialSavings(0.05),
            },
            MasteryBonus {
                unlock_level: 30,
                name: "Quality Artisan".to_string(),
                description: "Crafted items have enhanced properties".to_string(),
                bonus_type: BonusType::QualityBonus(0.15),
            },
            MasteryBonus {
                unlock_level: 60,
                name: "Legendary Crafter".to_string(),
                description: "Can craft legendary-tier items".to_string(),
                bonus_type: BonusType::UnlockContent("legendary_recipes".to_string()),
            },
        ]);

        // Engineering mastery bonuses
        self.mastery_bonuses.insert(BuildingSkill::Engineering, vec![
            MasteryBonus {
                unlock_level: 20,
                name: "Logic Designer".to_string(),
                description: "Unlock advanced logic components".to_string(),
                bonus_type: BonusType::UnlockContent("advanced_logic".to_string()),
            },
            MasteryBonus {
                unlock_level: 40,
                name: "Systems Architect".to_string(),
                description: "Can design automated systems".to_string(),
                bonus_type: BonusType::UnlockAbility("automation".to_string()),
            },
            MasteryBonus {
                unlock_level: 75,
                name: "Master Engineer".to_string(),
                description: "Unlock experimental technologies".to_string(),
                bonus_type: BonusType::UnlockContent("experimental_tech".to_string()),
            },
        ]);

        // Architecture mastery bonuses
        self.mastery_bonuses.insert(BuildingSkill::Architecture, vec![
            MasteryBonus {
                unlock_level: 15,
                name: "Design Intuition".to_string(),
                description: "Blueprint creation costs 25% less resources".to_string(),
                bonus_type: BonusType::CostReduction(0.25),
            },
            MasteryBonus {
                unlock_level: 35,
                name: "Structural Harmony".to_string(),
                description: "Large structures gain stability bonus".to_string(),
                bonus_type: BonusType::StabilityBonus(0.3),
            },
            MasteryBonus {
                unlock_level: 65,
                name: "Visionary Architect".to_string(),
                description: "Can design massive monuments and cities".to_string(),
                bonus_type: BonusType::UnlockContent("monumental_architecture".to_string()),
            },
        ]);

        // Resource Management mastery bonuses
        self.mastery_bonuses.insert(BuildingSkill::ResourceManagement, vec![
            MasteryBonus {
                unlock_level: 10,
                name: "Organized Storage".to_string(),
                description: "Inventory capacity increased by 25%".to_string(),
                bonus_type: BonusType::InventoryIncrease(0.25),
            },
            MasteryBonus {
                unlock_level: 30,
                name: "Supply Chain Expert".to_string(),
                description: "Resources convert with 10% efficiency bonus".to_string(),
                bonus_type: BonusType::ConversionEfficiency(0.1),
            },
            MasteryBonus {
                unlock_level: 55,
                name: "Resource Tycoon".to_string(),
                description: "Passive resource generation from managed stockpiles".to_string(),
                bonus_type: BonusType::UnlockAbility("passive_generation".to_string()),
            },
        ]);
    }

    /// Award experience in a specific skill
    pub fn award_experience(&mut self, skill: BuildingSkill, base_experience: u32, player_data: &mut PlayerData) -> RobinResult<Vec<SkillLevelUp>> {
        let multiplier = self.experience_multipliers.get(&skill).copied().unwrap_or(1.0);
        let final_experience = ((base_experience as f32) * multiplier) as u32;

        let mut level_ups = Vec::new();

        // Extract current values first
        let (mut current_level, mut current_exp) = if let Some(skill_level) = self.skills.get(&skill) {
            (skill_level.level, skill_level.experience + final_experience)
        } else {
            return Ok(level_ups); // Skill doesn't exist
        };

        // Calculate level ups without borrowing conflicts
        let mut leveled_up = false;
        while current_level < 100 { // Reasonable max level
            let required_exp = self.experience_required_for_next_level(current_level);
            if current_exp < required_exp {
                break;
            }

            current_exp -= required_exp;
            current_level += 1;
            leveled_up = true;

            let achievement_key = format!("skill_{}_{}", skill.to_string().to_lowercase(), current_level);
            player_data.unlock_achievement(&achievement_key);

            level_ups.push(SkillLevelUp {
                skill,
                old_level: current_level - 1,
                new_level: current_level,
                unlocked_bonuses: Vec::new(), // Will be populated below
            });
        }

        // Now update the skill level with final values
        if let Some(skill_level) = self.skills.get_mut(&skill) {
            skill_level.experience = current_exp;
            skill_level.total_experience += final_experience;
            skill_level.level = current_level;
        }

        // Update bonuses and player stats if leveled up
        if leveled_up {
            // Update bonuses for all level ups
            for level_up in &mut level_ups {
                level_up.unlocked_bonuses = self.check_mastery_unlocks(skill, level_up.new_level);
            }

            // Update player data with final skill progression
            if let Some(final_skill_level) = self.skills.get(&skill) {
                self.update_player_skill_stats(skill, final_skill_level, player_data);
            }

            // Award talent points for enhanced skill trees
            let total_levels_gained = level_ups.len() as u32;
            self.enhanced_skills.award_talent_points(skill, total_levels_gained);
        }

        Ok(level_ups)
    }

    /// Calculate experience required for the next level
    fn experience_required_for_next_level(&self, current_level: u32) -> u32 {
        // Exponential scaling: 100 * level^1.5
        (100.0 * (current_level as f32).powf(1.5)) as u32
    }

    /// Check for newly unlocked mastery bonuses
    fn check_mastery_unlocks(&self, skill: BuildingSkill, new_level: u32) -> Vec<MasteryBonus> {
        if let Some(bonuses) = self.mastery_bonuses.get(&skill) {
            bonuses.iter()
                .filter(|bonus| bonus.unlock_level == new_level)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Update player data with current skill statistics
    fn update_player_skill_stats(&self, skill: BuildingSkill, skill_level: &SkillLevel, player_data: &mut PlayerData) {
        let skill_name = skill.to_string().to_lowercase();

        player_data.stats.custom_stats.insert(
            format!("{}_level", skill_name),
            skill_level.level as f64
        );

        player_data.stats.custom_stats.insert(
            format!("{}_experience", skill_name),
            skill_level.total_experience as f64
        );
    }

    /// Get current level of a skill
    pub fn get_skill_level(&self, skill: &BuildingSkill) -> u32 {
        self.skills.get(skill).map(|s| s.level).unwrap_or(1)
    }

    /// Get current experience in a skill
    pub fn get_skill_experience(&self, skill: &BuildingSkill) -> (u32, u32, u32) {
        if let Some(skill_level) = self.skills.get(skill) {
            let required_for_next = self.experience_required_for_next_level(skill_level.level);
            (skill_level.experience, required_for_next, skill_level.total_experience)
        } else {
            (0, 100, 0)
        }
    }

    /// Get all unlocked mastery bonuses for a skill
    pub fn get_unlocked_bonuses(&self, skill: &BuildingSkill) -> Vec<&MasteryBonus> {
        let current_level = self.get_skill_level(skill);

        if let Some(bonuses) = self.mastery_bonuses.get(skill) {
            bonuses.iter()
                .filter(|bonus| bonus.unlock_level <= current_level)
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Process any pending skill gains
    pub fn process_skill_gains(&mut self, _player_data: &mut PlayerData) -> RobinResult<()> {
        // This could be used for time-based skill decay or passive gains in the future
        Ok(())
    }

    /// Get skill summary for display
    pub fn get_skill_summary(&self) -> HashMap<BuildingSkill, SkillSummary> {
        let mut summary = HashMap::new();

        for (&skill, skill_level) in &self.skills {
            let (current_exp, required_exp, total_exp) = self.get_skill_experience(&skill);
            let unlocked_bonuses = self.get_unlocked_bonuses(&skill).len();
            let next_bonus = self.mastery_bonuses.get(&skill)
                .and_then(|bonuses| bonuses.iter().find(|b| b.unlock_level > skill_level.level));

            summary.insert(skill, SkillSummary {
                level: skill_level.level,
                current_experience: current_exp,
                experience_to_next: required_exp,
                total_experience: total_exp,
                unlocked_bonuses,
                next_bonus_level: next_bonus.map(|b| b.unlock_level),
            });
        }

        summary
    }

    // Enhanced Skill Tree System Methods

    /// Allocate a talent point to a specialization node
    pub fn allocate_talent_point(&mut self, specialization: SpecializationPath, node_id: &str, player_data: &mut PlayerData) -> RobinResult<SkillAllocationResult> {
        self.enhanced_skills.allocate_talent_point(specialization, node_id, player_data)
    }

    /// Get current specialization summary
    pub fn get_specialization_summary(&self) -> SpecializationSummary {
        self.enhanced_skills.get_specialization_summary()
    }

    /// Reset all talent point allocations (respec)
    pub fn reset_specializations(&mut self, player_data: &mut PlayerData) -> RobinResult<u32> {
        self.enhanced_skills.reset_specializations(player_data)
    }

    /// Check available talent points
    pub fn get_available_talent_points(&self) -> u32 {
        self.enhanced_skills.get_specialization_summary().talent_points.available
    }

    /// Get player's primary specialization
    pub fn get_primary_specialization(&self) -> Option<SpecializationPath> {
        self.enhanced_skills.get_specialization_summary().primary_specialization
    }
}

/// Different building-related skills players can develop
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BuildingSkill {
    /// General building and placement proficiency
    Construction,
    /// Resource gathering and extraction
    Mining,
    /// Creating tools and advanced materials
    Crafting,
    /// Logic systems and automation
    Engineering,
    /// Design and aesthetics
    Architecture,
    /// Inventory and supply chain optimization
    ResourceManagement,
}

impl BuildingSkill {
    pub fn to_string(&self) -> &'static str {
        match self {
            BuildingSkill::Construction => "Construction",
            BuildingSkill::Mining => "Mining",
            BuildingSkill::Crafting => "Crafting",
            BuildingSkill::Engineering => "Engineering",
            BuildingSkill::Architecture => "Architecture",
            BuildingSkill::ResourceManagement => "ResourceManagement",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            BuildingSkill::Construction => "Proficiency in building structures and placing voxels efficiently",
            BuildingSkill::Mining => "Expertise in resource extraction and terrain modification",
            BuildingSkill::Crafting => "Skill in creating tools, materials, and advanced components",
            BuildingSkill::Engineering => "Knowledge of logic systems, automation, and complex mechanisms",
            BuildingSkill::Architecture => "Artistic vision and structural design capabilities",
            BuildingSkill::ResourceManagement => "Optimization of inventory, storage, and resource flows",
        }
    }
}

/// Current level and experience in a skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillLevel {
    pub level: u32,
    pub experience: u32, // Experience toward next level
    pub total_experience: u32, // All-time experience earned
}

/// Bonus unlocked at certain skill levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasteryBonus {
    pub unlock_level: u32,
    pub name: String,
    pub description: String,
    pub bonus_type: BonusType,
}

/// Types of bonuses that can be unlocked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BonusType {
    SpeedIncrease(f32),
    DurabilityIncrease(f32),
    YieldIncrease(f32),
    MaterialSavings(f32),
    QualityBonus(f32),
    CostReduction(f32),
    StabilityBonus(f32),
    InventoryIncrease(f32),
    ConversionEfficiency(f32),
    RareResourceBonus(f32),
    UnlockContent(String),
    UnlockAbility(String),
}

/// Information about a skill level up event
#[derive(Debug, Clone)]
pub struct SkillLevelUp {
    pub skill: BuildingSkill,
    pub old_level: u32,
    pub new_level: u32,
    pub unlocked_bonuses: Vec<MasteryBonus>,
}

/// Summary of a skill's current state
#[derive(Debug, Clone)]
pub struct SkillSummary {
    pub level: u32,
    pub current_experience: u32,
    pub experience_to_next: u32,
    pub total_experience: u32,
    pub unlocked_bonuses: usize,
    pub next_bonus_level: Option<u32>,
}

impl Default for SkillManager {
    fn default() -> Self {
        Self::new()
    }
}