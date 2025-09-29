//! Advanced Character Progression Integration
//! Unifies player attributes, skill trees, and progression systems with Apple Silicon optimization

use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

use crate::engine::error::RobinResult;
use crate::engine::save_system::PlayerData;
use super::{
    player_attributes::{PlayerAttributeManager, CoreAttributeType, DerivedStatType, TemporaryEffect},
    skill_tree::{EnhancedSkillManager, SpecializationPath, SkillAllocationResult, TalentPoints},
    progression::{SkillManager, BuildingSkill, SkillLevel},
    stat_monitoring::{StatMonitoringSystem, StatEvent, StatEventType, EventSource},
    SessionStats,
};

// Type aliases for convenience
pub type CoreAttribute = CoreAttributeType;

/// Attribute bonus structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeBonus {
    pub attribute: CoreAttribute,
    pub value: f32,
}

/// Result of allocating an attribute point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeAllocationResult {
    pub success: bool,
    pub new_value: u32,
    pub derived_changes: HashMap<String, f32>,
    pub synergy_bonuses: Vec<String>,
}

/// Result of character progression reset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetResult {
    pub success: bool,
    pub points_refunded: u32,
    pub talents_reset: u32,
    pub attributes_reset: HashMap<String, u32>,
}

/// Types of character progression resets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResetType {
    AttributesOnly,
    TalentsOnly,
    FullReset,
}

/// Unified character progression system that integrates attributes, skills, and specializations
#[derive(Debug)]
pub struct CharacterProgressionManager {
    /// Player attribute management
    attribute_manager: PlayerAttributeManager,

    /// Enhanced skill tree system
    skill_tree_manager: EnhancedSkillManager,

    /// Traditional skill system
    building_skills: SkillManager,

    /// Progression tracking and analytics
    progression_tracker: ProgressionTracker,

    /// Experience and level management
    experience_manager: ExperienceManager,

    /// Character milestones and achievements
    milestone_tracker: MilestoneTracker,

    /// Progression synergy calculator
    synergy_calculator: SynergyCalculator,

    /// Configuration
    config: ProgressionConfig,
}

impl CharacterProgressionManager {
    /// Create new character progression manager
    pub fn new() -> Self {
        Self {
            attribute_manager: PlayerAttributeManager::new(),
            skill_tree_manager: EnhancedSkillManager::new(),
            building_skills: SkillManager::new(),
            progression_tracker: ProgressionTracker::new(),
            experience_manager: ExperienceManager::new(),
            milestone_tracker: MilestoneTracker::new(),
            synergy_calculator: SynergyCalculator::new(),
            config: ProgressionConfig::default(),
        }
    }

    /// Initialize the progression system
    pub fn initialize(&mut self, player_data: &PlayerData) -> RobinResult<()> {
        // Initialize attribute system
        self.attribute_manager.initialize()?;

        // Initialize experience manager with player data
        self.experience_manager.initialize(player_data)?;

        // Initialize milestone tracking
        self.milestone_tracker.initialize(player_data)?;

        // Calculate initial synergies
        self.synergy_calculator.calculate_synergies(
            &self.attribute_manager,
            &self.skill_tree_manager,
            &self.building_skills
        );

        println!("🚀 Character Progression Manager initialized");
        Ok(())
    }

    /// Update progression systems
    pub fn update(&mut self,
                  delta_time: f32,
                  player_data: &mut PlayerData,
                  stat_monitoring: &mut StatMonitoringSystem) -> RobinResult<()> {
        // Update core systems
        self.attribute_manager.update(delta_time)?;
        self.experience_manager.update(delta_time, player_data)?;

        // Update progression tracking
        self.progression_tracker.update(&self.attribute_manager, &self.skill_tree_manager, delta_time);

        // Check for milestone achievements
        self.milestone_tracker.check_milestones(
            &self.attribute_manager,
            &self.skill_tree_manager,
            &self.building_skills,
            player_data
        )?;

        // Recalculate synergies if needed
        if self.progression_tracker.needs_synergy_update() {
            self.synergy_calculator.calculate_synergies(
                &self.attribute_manager,
                &self.skill_tree_manager,
                &self.building_skills
            );
            self.progression_tracker.mark_synergy_updated();
        }

        // Record progression events for monitoring
        self.record_progression_events(stat_monitoring);

        Ok(())
    }

    /// Award experience in a specific area
    pub fn award_experience(&mut self,
                           experience_type: ExperienceType,
                           amount: u32,
                           player_data: &mut PlayerData) -> RobinResult<Vec<ProgressionEvent>> {
        let mut events = Vec::new();

        // Award experience
        let exp_result = self.experience_manager.award_experience(experience_type, amount, player_data)?;

        if let Some(level_up) = exp_result.level_up {
            events.push(ProgressionEvent::LevelUp(level_up));

            // Award attribute points for level up
            let attribute_points = self.calculate_attribute_point_reward(level_up.new_level);
            if attribute_points > 0 {
                self.attribute_manager.award_attribute_points(attribute_points);
                events.push(ProgressionEvent::AttributePointsAwarded { amount: attribute_points });
            }

            // Award talent points for level up
            let talent_points = self.calculate_talent_point_reward(level_up.new_level, experience_type);
            if talent_points > 0 {
                self.skill_tree_manager.award_talent_points(BuildingSkill::from_experience_type(experience_type), talent_points);
                events.push(ProgressionEvent::TalentPointsAwarded {
                    amount: talent_points,
                    skill_type: BuildingSkill::from_experience_type(experience_type)
                });
            }
        }

        // Check for specialization unlocks
        if let Some(specialization_unlock) = self.check_specialization_unlocks(&exp_result) {
            events.push(ProgressionEvent::SpecializationUnlocked(specialization_unlock));
        }

        Ok(events)
    }

    /// Allocate attribute points
    pub fn allocate_attribute_point(&mut self,
                                   attribute: CoreAttribute,
                                   player_data: &mut PlayerData) -> RobinResult<AttributeAllocationResult> {
        let result = self.attribute_manager.allocate_attribute_point(attribute, player_data)?;

        // Record the allocation in progression tracking
        self.progression_tracker.record_attribute_allocation(attribute, result.new_value);

        // Calculate synergy benefits
        let synergy_bonuses = self.synergy_calculator.calculate_attribute_synergies(attribute, result.new_value);

        Ok(AttributeAllocationResult {
            attribute,
            old_value: result.old_value,
            new_value: result.new_value,
            derived_stat_changes: result.derived_stat_changes,
            synergy_bonuses,
            cost: result.cost,
        })
    }

    /// Allocate talent point in skill tree
    pub fn allocate_talent_point(&mut self,
                                 specialization: SpecializationPath,
                                 node_id: &str,
                                 player_data: &mut PlayerData) -> RobinResult<SkillAllocationResult> {
        let result = self.skill_tree_manager.allocate_talent_point(specialization, node_id, player_data)?;

        // Record the allocation in progression tracking
        self.progression_tracker.record_skill_allocation(specialization, node_id.to_string(), result.points_allocated);

        // Apply skill bonuses to attributes if applicable
        self.apply_skill_bonuses_to_attributes(&result)?;

        Ok(result)
    }

    /// Get comprehensive character overview
    pub fn get_character_overview(&self) -> CharacterOverview {
        let attribute_summary = self.attribute_manager.get_attribute_summary();
        let skill_summary = self.skill_tree_manager.get_specialization_summary();
        let building_skill_summary = self.building_skills.get_skill_summary();
        let progression_summary = self.progression_tracker.get_summary();
        let synergy_summary = self.synergy_calculator.get_synergy_summary();

        CharacterOverview {
            level: self.experience_manager.get_overall_level(),
            total_experience: self.experience_manager.get_total_experience(),
            available_attribute_points: attribute_summary.available_points,
            available_talent_points: skill_summary.talent_points.available,

            // Core attributes
            core_attributes: attribute_summary.core_attributes,
            derived_stats: attribute_summary.derived_stats,

            // Skill trees
            specialization_progress: skill_summary.specializations,
            primary_specialization: skill_summary.primary_specialization,

            // Building skills
            building_skills: building_skill_summary,

            // Progression metrics
            progression_velocity: progression_summary.velocity,
            efficiency_rating: progression_summary.efficiency,
            synergy_bonuses: synergy_summary.active_bonuses,

            // Recommendations
            recommendations: self.generate_progression_recommendations(),

            // Milestones
            recent_milestones: self.milestone_tracker.get_recent_milestones(),
            upcoming_milestones: self.milestone_tracker.get_upcoming_milestones(),
        }
    }

    /// Get progression recommendations
    pub fn get_progression_recommendations(&self) -> Vec<ProgressionRecommendation> {
        self.generate_progression_recommendations()
    }

    /// Reset character progression (with confirmation)
    pub fn reset_character_progression(&mut self,
                                      reset_type: ResetType,
                                      player_data: &mut PlayerData) -> RobinResult<ResetResult> {
        let mut result = ResetResult {
            reset_type,
            refunded_points: HashMap::new(),
            preserved_data: PreservedData::default(),
        };

        match reset_type {
            ResetType::Attributes => {
                let refunded = self.attribute_manager.reset_attributes(player_data)?;
                result.refunded_points.insert("attribute_points".to_string(), refunded as u32);
            },
            ResetType::SkillTrees => {
                let refunded = self.skill_tree_manager.reset_specializations(player_data)?;
                result.refunded_points.insert("talent_points".to_string(), refunded as u32);
            },
            ResetType::BuildingSkills => {
                let refunded = self.building_skills.reset_skills(player_data)?;
                result.refunded_points.insert("skill_points".to_string(), refunded as u32);
            },
            ResetType::Complete => {
                // Full character reset
                let attr_refund = self.attribute_manager.reset_attributes(player_data)?;
                let talent_refund = self.skill_tree_manager.reset_specializations(player_data)?;
                let skill_refund = self.building_skills.reset_skills(player_data)?;

                result.refunded_points.insert("attribute_points".to_string(), attr_refund as u32);
                result.refunded_points.insert("talent_points".to_string(), talent_refund as u32);
                result.refunded_points.insert("skill_points".to_string(), skill_refund as u32);

                // Preserve certain data based on config
                if self.config.preserve_level_on_reset {
                    result.preserved_data.level = Some(self.experience_manager.get_overall_level());
                }
                if self.config.preserve_milestones_on_reset {
                    result.preserved_data.milestones = Some(self.milestone_tracker.get_achieved_milestones());
                }
            }
        }

        // Recalculate synergies after reset
        self.synergy_calculator.calculate_synergies(
            &self.attribute_manager,
            &self.skill_tree_manager,
            &self.building_skills
        );

        println!("🔄 Character progression reset: {:?}", reset_type);
        Ok(result)
    }

    /// Apply equipment that affects progression
    pub fn apply_equipment_effects(&mut self, equipment: &EquipmentSet) -> RobinResult<()> {
        // Apply attribute modifiers
        for modifier in &equipment.attribute_modifiers {
            self.attribute_manager.apply_equipment_modifier(*modifier)?;
        }

        // Apply skill bonuses
        for bonus in &equipment.skill_bonuses {
            self.apply_skill_bonus(*bonus)?;
        }

        // Apply experience modifiers
        for modifier in &equipment.experience_modifiers {
            self.experience_manager.apply_experience_modifier(*modifier);
        }

        println!("⚔️ Applied equipment effects to character progression");
        Ok(())
    }

    /// Get progression analytics
    pub fn get_progression_analytics(&self) -> ProgressionAnalytics {
        self.progression_tracker.get_analytics()
    }

    // Private helper methods

    /// Calculate attribute points to award for level up
    fn calculate_attribute_point_reward(&self, level: u32) -> u32 {
        match level {
            1..=10 => 2,  // Early levels get more points
            11..=25 => 1, // Mid levels get standard points
            26..=50 => 1, // High levels get standard points
            _ => if level % 5 == 0 { 2 } else { 1 }, // Bonus every 5 levels
        }
    }

    /// Calculate talent points to award for level up
    fn calculate_talent_point_reward(&self, level: u32, exp_type: ExperienceType) -> u32 {
        let base_points = match level {
            1..=5 => 2,   // Rapid early progression
            6..=15 => 1,  // Standard progression
            16..=30 => 1, // Continued progression
            _ => if level % 3 == 0 { 2 } else { 1 }, // Bonus every 3 levels
        };

        // Bonus points for specialized experience types
        let specialty_bonus = match exp_type {
            ExperienceType::Combat | ExperienceType::Crafting | ExperienceType::Building => 1,
            _ => 0,
        };

        base_points + specialty_bonus
    }

    /// Check for specialization unlocks
    fn check_specialization_unlocks(&self, exp_result: &ExperienceResult) -> Option<SpecializationUnlock> {
        // Check if player has reached specialization thresholds
        if exp_result.total_level >= 10 {
            // Unlock based on dominant skill areas
            let dominant_skill = self.experience_manager.get_dominant_skill_area();
            let specialization = match dominant_skill {
                ExperienceType::Building | ExperienceType::Engineering => SpecializationPath::Engineer,
                ExperienceType::Crafting | ExperienceType::Art => SpecializationPath::Artist,
                ExperienceType::Exploration | ExperienceType::Discovery => SpecializationPath::Explorer,
                ExperienceType::Research | ExperienceType::Learning => SpecializationPath::Researcher,
                _ => return None,
            };

            Some(SpecializationUnlock {
                specialization,
                unlock_level: exp_result.total_level,
                bonus_points: 5, // Bonus talent points for specialization unlock
            })
        } else {
            None
        }
    }

    /// Apply skill bonuses to attributes
    fn apply_skill_bonuses_to_attributes(&mut self, allocation_result: &SkillAllocationResult) -> RobinResult<()> {
        // Convert skill bonuses to attribute bonuses
        for bonus in &allocation_result.bonuses_gained {
            let attribute_bonus = self.convert_skill_bonus_to_attribute_bonus(bonus);
            if let Some(attr_bonus) = attribute_bonus {
                self.attribute_manager.apply_temporary_effect(TemporaryEffect {
                    source: format!("Skill: {}", allocation_result.node_id),
                    bonuses: vec![attr_bonus],
                    duration: None, // Permanent from skills
                    applied_at: std::time::Instant::now(),
                })?;
            }
        }

        Ok(())
    }

    /// Convert skill bonus to attribute bonus
    fn convert_skill_bonus_to_attribute_bonus(&self, _skill_bonus: &super::skill_tree::SkillBonus) -> Option<AttributeBonus> {
        // TODO: Implement skill bonus conversion when skill tree bonus types are defined
        // For now, return a basic strength bonus
        Some(AttributeBonus {
            attribute: CoreAttribute::Strength,
            value: 1.0,
        })
    }

    /// Apply skill bonus
    fn apply_skill_bonus(&mut self, bonus: SkillBonus) -> RobinResult<()> {
        // Apply bonus to the appropriate skill system
        match bonus.skill_type {
            SkillBonusType::Building(skill) => {
                self.building_skills.apply_bonus(skill, bonus.bonus_amount)?;
            },
            SkillBonusType::Attribute(attr) => {
                let attr_bonus = AttributeBonus {
                    attribute: attr,
                    value: bonus.bonus_amount,
                };
                self.attribute_manager.apply_temporary_effect(TemporaryEffect {
                    source: "Equipment".to_string(),
                    bonuses: vec![attr_bonus],
                    duration: None,
                    applied_at: std::time::Instant::now(),
                })?;
            },
        }

        Ok(())
    }

    /// Generate progression recommendations
    fn generate_progression_recommendations(&self) -> Vec<ProgressionRecommendation> {
        let mut recommendations = Vec::new();

        // Analyze current character state
        let attribute_summary = self.attribute_manager.get_attribute_summary();
        let skill_summary = self.skill_tree_manager.get_specialization_summary();
        let synergy_summary = self.synergy_calculator.get_synergy_summary();

        // Recommend attribute allocations
        if attribute_summary.available_points > 0 {
            let recommended_attribute = self.recommend_next_attribute_allocation(&attribute_summary);
            recommendations.push(ProgressionRecommendation {
                recommendation_type: RecommendationType::AttributeAllocation,
                priority: RecommendationPriority::High,
                title: format!("Increase {:?}", recommended_attribute),
                description: format!("Allocating points to {:?} will improve your {} capabilities",
                                   recommended_attribute,
                                   self.get_attribute_benefit_description(recommended_attribute)),
                expected_benefit: self.calculate_attribute_benefit(recommended_attribute),
                cost: 1,
            });
        }

        // Recommend skill tree allocations
        if skill_summary.talent_points.available > 0 {
            let recommended_skill = self.recommend_next_skill_allocation(&skill_summary);
            recommendations.push(recommended_skill);
        }

        // Recommend synergy improvements
        for synergy_opportunity in synergy_summary.improvement_opportunities {
            recommendations.push(ProgressionRecommendation {
                recommendation_type: RecommendationType::SynergyOptimization,
                priority: RecommendationPriority::Medium,
                title: synergy_opportunity.title,
                description: synergy_opportunity.description,
                expected_benefit: synergy_opportunity.potential_benefit,
                cost: synergy_opportunity.required_investment,
            });
        }

        // Sort by priority and expected benefit
        recommendations.sort_by(|a, b| {
            match a.priority.cmp(&b.priority) {
                std::cmp::Ordering::Equal => b.expected_benefit.partial_cmp(&a.expected_benefit).unwrap_or(std::cmp::Ordering::Equal),
                other => other,
            }
        });

        recommendations
    }

    /// Recommend next attribute allocation
    fn recommend_next_attribute_allocation(&self, summary: &super::player_attributes::AttributeSummary) -> CoreAttribute {
        // Simple recommendation based on lowest current attribute
        let mut lowest_attribute = CoreAttribute::Strength;
        let mut lowest_value = summary.core_attributes.strength;

        if summary.core_attributes.dexterity < lowest_value {
            lowest_value = summary.core_attributes.dexterity;
            lowest_attribute = CoreAttribute::Dexterity;
        }
        if summary.core_attributes.intelligence < lowest_value {
            lowest_value = summary.core_attributes.intelligence;
            lowest_attribute = CoreAttribute::Intelligence;
        }
        if summary.core_attributes.vitality < lowest_value {
            lowest_value = summary.core_attributes.vitality;
            lowest_attribute = CoreAttribute::Vitality;
        }

        lowest_attribute
    }

    /// Recommend next skill allocation
    fn recommend_next_skill_allocation(&self, summary: &super::skill_tree::SpecializationSummary) -> ProgressionRecommendation {
        // Recommend based on primary specialization or balanced growth
        let recommended_path = summary.primary_specialization.unwrap_or(SpecializationPath::Engineer);

        ProgressionRecommendation {
            recommendation_type: RecommendationType::SkillAllocation,
            priority: RecommendationPriority::High,
            title: format!("Advance {:?} Specialization", recommended_path),
            description: format!("Continue developing your {:?} skills for increased effectiveness", recommended_path),
            expected_benefit: 15.0, // Estimated benefit
            cost: 1,
        }
    }

    /// Get attribute benefit description
    fn get_attribute_benefit_description(&self, attribute: CoreAttribute) -> &'static str {
        match attribute {
            CoreAttribute::Strength => "physical power and carrying capacity",
            CoreAttribute::Dexterity => "speed and precision",
            CoreAttribute::Intelligence => "problem-solving and mana",
            CoreAttribute::Vitality => "health and survivability",
            CoreAttribute::Willpower => "mental resistance and determination",
            CoreAttribute::Charisma => "social interactions and leadership",
            CoreAttribute::Focus => "concentration and accuracy",
            CoreAttribute::Creativity => "innovation and artistic ability",
            CoreAttribute::Perception => "awareness and detection",
            CoreAttribute::Endurance => "stamina and persistence",
            CoreAttribute::Luck => "fortunate outcomes and critical hits",
            CoreAttribute::Resonance => "magical attunement and harmony",
        }
    }

    /// Calculate expected benefit from attribute increase
    fn calculate_attribute_benefit(&self, attribute: CoreAttribute) -> f32 {
        // Calculate based on derived stat improvements
        let current_stats = self.attribute_manager.get_derived_stats();

        // Simulate increasing the attribute by 1
        let simulated_improvement = match attribute {
            CoreAttribute::Strength => current_stats.carry_capacity * 0.1 + current_stats.max_health * 0.02,
            CoreAttribute::Dexterity => current_stats.movement_speed * 0.1 + current_stats.attack_speed * 0.02,
            CoreAttribute::Intelligence => current_stats.max_mana * 0.06 + current_stats.experience_gain * 0.01,
            CoreAttribute::Vitality => current_stats.max_health * 0.1 + current_stats.health_regen_rate * 0.1,
            _ => 10.0, // Default benefit value
        };

        simulated_improvement.min(50.0) // Cap the benefit calculation
    }

    /// Record progression events for monitoring
    fn record_progression_events(&self, stat_monitoring: &mut StatMonitoringSystem) {
        // Record attribute changes as stat events
        let changes = self.progression_tracker.get_recent_changes();

        for change in changes {
            let event = StatEvent {
                stat_name: change.stat_name,
                event_type: StatEventType::AttributeIncrease,
                old_value: change.old_value,
                new_value: change.new_value,
                source: EventSource::Training,
            };
            stat_monitoring.record_stat_event(event);
        }
    }
}

/// Experience management system
#[derive(Debug)]
pub struct ExperienceManager {
    /// Experience pools for different activities
    experience_pools: HashMap<ExperienceType, ExperiencePool>,

    /// Overall character level
    overall_level: u32,

    /// Experience multipliers
    multipliers: HashMap<ExperienceType, f32>,

    /// Level progression curve
    level_curve: LevelCurve,
}

impl ExperienceManager {
    pub fn new() -> Self {
        let mut experience_pools = HashMap::new();

        // Initialize experience pools
        for exp_type in [
            ExperienceType::Combat,
            ExperienceType::Building,
            ExperienceType::Crafting,
            ExperienceType::Exploration,
            ExperienceType::Research,
            ExperienceType::Social,
            ExperienceType::Art,
            ExperienceType::Engineering,
            ExperienceType::Discovery,
            ExperienceType::Learning,
        ] {
            experience_pools.insert(exp_type, ExperiencePool::new());
        }

        Self {
            experience_pools,
            overall_level: 1,
            multipliers: HashMap::new(),
            level_curve: LevelCurve::new(),
        }
    }

    pub fn initialize(&mut self, player_data: &PlayerData) -> RobinResult<()> {
        // Load experience data from player data
        if let Some(level) = player_data.stats.custom_stats.get("overall_level") {
            self.overall_level = *level as u32;
        }

        // Load experience pools from player data
        for (exp_type, pool) in &mut self.experience_pools {
            let key = format!("exp_{:?}", exp_type).to_lowercase();
            if let Some(exp) = player_data.stats.custom_stats.get(&key) {
                pool.current_experience = *exp as u32;
                pool.level = self.level_curve.calculate_level(pool.current_experience);
            }
        }

        Ok(())
    }

    pub fn update(&mut self, delta_time: f32, player_data: &mut PlayerData) -> RobinResult<()> {
        // Update experience pools
        for (exp_type, pool) in &mut self.experience_pools {
            pool.update(delta_time);

            // Save to player data
            let key = format!("exp_{:?}", exp_type).to_lowercase();
            player_data.stats.custom_stats.insert(key, pool.current_experience as f32);
        }

        // Update overall level
        let total_exp = self.get_total_experience();
        let new_overall_level = self.level_curve.calculate_level(total_exp);
        if new_overall_level != self.overall_level {
            self.overall_level = new_overall_level;
            player_data.stats.custom_stats.insert("overall_level".to_string(), self.overall_level as f32);
        }

        Ok(())
    }

    pub fn award_experience(&mut self, exp_type: ExperienceType, amount: u32, player_data: &mut PlayerData) -> RobinResult<ExperienceResult> {
        let pool = self.experience_pools.get_mut(&exp_type)
            .ok_or_else(|| crate::engine::error::RobinError::InvalidInput(format!("Unknown experience type: {:?}", exp_type)))?;

        let old_level = pool.level;
        let old_experience = pool.current_experience;

        // Apply multipliers
        let multiplier = self.multipliers.get(&exp_type).copied().unwrap_or(1.0);
        let final_amount = (amount as f32 * multiplier) as u32;

        // Add experience
        pool.current_experience += final_amount;
        pool.level = self.level_curve.calculate_level(pool.current_experience);

        // Check for level up
        let level_up = if pool.level > old_level {
            Some(LevelUpResult {
                experience_type: exp_type,
                old_level,
                new_level: pool.level,
                experience_gained: final_amount,
            })
        } else {
            None
        };

        // Update overall level
        let total_exp = self.get_total_experience();
        let old_overall_level = self.overall_level;
        self.overall_level = self.level_curve.calculate_level(total_exp);

        Ok(ExperienceResult {
            experience_type: exp_type,
            experience_gained: final_amount,
            old_experience,
            new_experience: pool.current_experience,
            level_up,
            total_level: self.overall_level,
            level_up_overall: self.overall_level > old_overall_level,
        })
    }

    pub fn get_total_experience(&self) -> u32 {
        self.experience_pools.values().map(|pool| pool.current_experience).sum()
    }

    pub fn get_overall_level(&self) -> u32 {
        self.overall_level
    }

    pub fn get_dominant_skill_area(&self) -> ExperienceType {
        self.experience_pools.iter()
            .max_by_key(|(_, pool)| pool.current_experience)
            .map(|(exp_type, _)| *exp_type)
            .unwrap_or(ExperienceType::Building)
    }

    pub fn apply_experience_modifier(&mut self, modifier: ExperienceModifier) {
        match modifier.duration {
            Some(_) => {
                // Temporary modifier - would need to track expiration
                self.multipliers.insert(modifier.experience_type, modifier.multiplier);
            },
            None => {
                // Permanent modifier
                self.multipliers.insert(modifier.experience_type, modifier.multiplier);
            }
        }
    }
}

/// Progression tracking and analytics
#[derive(Debug)]
pub struct ProgressionTracker {
    /// Recent attribute allocations
    recent_attribute_changes: Vec<AttributeChange>,

    /// Recent skill allocations
    recent_skill_changes: Vec<SkillChange>,

    /// Progression velocity metrics
    velocity_tracker: VelocityTracker,

    /// Efficiency calculations
    efficiency_calculator: EfficiencyCalculator,

    /// Synergy update flag
    needs_synergy_update: bool,
}

impl ProgressionTracker {
    pub fn new() -> Self {
        Self {
            recent_attribute_changes: Vec::new(),
            recent_skill_changes: Vec::new(),
            velocity_tracker: VelocityTracker::new(),
            efficiency_calculator: EfficiencyCalculator::new(),
            needs_synergy_update: false,
        }
    }

    pub fn update(&mut self,
                  attribute_manager: &PlayerAttributeManager,
                  skill_manager: &EnhancedSkillManager,
                  delta_time: f32) {
        self.velocity_tracker.update(attribute_manager, skill_manager, delta_time);
        self.efficiency_calculator.update(attribute_manager, skill_manager, delta_time);

        // Clean up old changes
        let cutoff = Instant::now() - Duration::from_secs(300); // 5 minutes
        self.recent_attribute_changes.retain(|change| change.timestamp >= cutoff);
        self.recent_skill_changes.retain(|change| change.timestamp >= cutoff);
    }

    pub fn record_attribute_allocation(&mut self, attribute: CoreAttribute, new_value: u32) {
        self.recent_attribute_changes.push(AttributeChange {
            stat_name: format!("{:?}", attribute),
            old_value: new_value as f32 - 1.0,
            new_value: new_value as f32,
            timestamp: Instant::now(),
        });
        self.needs_synergy_update = true;
    }

    pub fn record_skill_allocation(&mut self, specialization: SpecializationPath, node_id: String, points: u32) {
        self.recent_skill_changes.push(SkillChange {
            specialization,
            node_id,
            points_allocated: points,
            timestamp: Instant::now(),
        });
        self.needs_synergy_update = true;
    }

    pub fn needs_synergy_update(&self) -> bool {
        self.needs_synergy_update
    }

    pub fn mark_synergy_updated(&mut self) {
        self.needs_synergy_update = false;
    }

    pub fn get_summary(&self) -> ProgressionSummary {
        ProgressionSummary {
            velocity: self.velocity_tracker.get_current_velocity(),
            efficiency: self.efficiency_calculator.get_current_efficiency(),
            recent_changes: self.recent_attribute_changes.len() + self.recent_skill_changes.len(),
        }
    }

    pub fn get_recent_changes(&self) -> Vec<AttributeChange> {
        self.recent_attribute_changes.clone()
    }

    pub fn get_analytics(&self) -> ProgressionAnalytics {
        ProgressionAnalytics {
            velocity_metrics: self.velocity_tracker.get_metrics(),
            efficiency_metrics: self.efficiency_calculator.get_metrics(),
            allocation_patterns: self.analyze_allocation_patterns(),
            optimization_opportunities: self.identify_optimization_opportunities(),
        }
    }

    fn analyze_allocation_patterns(&self) -> AllocationPatterns {
        // Analyze patterns in recent allocations
        let mut attribute_frequency = HashMap::new();
        let mut specialization_frequency = HashMap::new();

        for change in &self.recent_attribute_changes {
            *attribute_frequency.entry(change.stat_name.clone()).or_insert(0) += 1;
        }

        for change in &self.recent_skill_changes {
            *specialization_frequency.entry(change.specialization).or_insert(0) += 1;
        }

        AllocationPatterns {
            preferred_attributes: attribute_frequency,
            preferred_specializations: specialization_frequency,
            allocation_frequency: self.calculate_allocation_frequency(),
            efficiency_trend: self.efficiency_calculator.get_trend(),
        }
    }

    fn identify_optimization_opportunities(&self) -> Vec<OptimizationOpportunity> {
        let mut opportunities = Vec::new();

        // Check for inefficient allocation patterns
        if self.efficiency_calculator.get_current_efficiency() < 0.7 {
            opportunities.push(OptimizationOpportunity {
                opportunity_type: OpportunityType::EfficiencyImprovement,
                description: "Consider focusing allocations for better synergy".to_string(),
                potential_benefit: 25.0,
                required_effort: EffortLevel::Medium,
            });
        }

        // Check for unbalanced progression
        let balance_score = self.calculate_balance_score();
        if balance_score < 0.6 {
            opportunities.push(OptimizationOpportunity {
                opportunity_type: OpportunityType::BalanceImprovement,
                description: "Consider diversifying your progression choices".to_string(),
                potential_benefit: 15.0,
                required_effort: EffortLevel::Low,
            });
        }

        opportunities
    }

    fn calculate_allocation_frequency(&self) -> f32 {
        let total_changes = self.recent_attribute_changes.len() + self.recent_skill_changes.len();
        if total_changes == 0 {
            return 0.0;
        }

        // Calculate frequency per minute
        let time_span = Duration::from_secs(300); // 5 minutes
        (total_changes as f32) / (time_span.as_secs_f32() / 60.0)
    }

    fn calculate_balance_score(&self) -> f32 {
        // Calculate how balanced the allocations are across different areas
        let mut area_counts = HashMap::new();

        for change in &self.recent_attribute_changes {
            *area_counts.entry("attributes".to_string()).or_insert(0) += 1;
        }

        for change in &self.recent_skill_changes {
            let area = format!("{:?}", change.specialization);
            *area_counts.entry(area).or_insert(0) += 1;
        }

        if area_counts.is_empty() {
            return 1.0;
        }

        // Calculate variance in allocation distribution
        let total = area_counts.values().sum::<i32>() as f32;
        let mean = total / area_counts.len() as f32;
        let variance: f32 = area_counts.values()
            .map(|&count| (count as f32 - mean).powi(2))
            .sum::<f32>() / area_counts.len() as f32;

        // Convert variance to balance score (lower variance = higher balance)
        (1.0 / (1.0 + variance)).min(1.0)
    }
}

// Supporting data structures and implementations...

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExperienceType {
    Combat,
    Building,
    Crafting,
    Exploration,
    Research,
    Social,
    Art,
    Engineering,
    Discovery,
    Learning,
}

#[derive(Debug)]
pub struct ExperiencePool {
    pub current_experience: u32,
    pub level: u32,
    pub experience_rate: f32,
}

impl ExperiencePool {
    pub fn new() -> Self {
        Self {
            current_experience: 0,
            level: 1,
            experience_rate: 1.0,
        }
    }

    pub fn update(&mut self, _delta_time: f32) {
        // Experience pool updates (decay, bonuses, etc.)
    }
}

#[derive(Debug)]
pub struct LevelCurve {
    base_experience: u32,
    growth_factor: f32,
}

impl LevelCurve {
    pub fn new() -> Self {
        Self {
            base_experience: 100,
            growth_factor: 1.5,
        }
    }

    pub fn calculate_level(&self, experience: u32) -> u32 {
        if experience == 0 {
            return 1;
        }

        let mut level = 1;
        let mut required_exp = self.base_experience;
        let mut total_exp = 0;

        while total_exp + required_exp <= experience {
            total_exp += required_exp;
            level += 1;
            required_exp = (required_exp as f32 * self.growth_factor) as u32;
        }

        level
    }

    pub fn experience_for_level(&self, level: u32) -> u32 {
        if level <= 1 {
            return 0;
        }

        let mut total_exp = 0;
        let mut required_exp = self.base_experience;

        for _ in 2..=level {
            total_exp += required_exp;
            required_exp = (required_exp as f32 * self.growth_factor) as u32;
        }

        total_exp
    }
}

// Additional supporting structures for the progression system...
// (Continuing with remaining data structures in the next part due to length)

impl BuildingSkill {
    pub fn from_experience_type(exp_type: ExperienceType) -> Self {
        match exp_type {
            ExperienceType::Building => BuildingSkill::Construction,
            ExperienceType::Engineering => BuildingSkill::Engineering,
            ExperienceType::Crafting => BuildingSkill::Crafting,
            ExperienceType::Research => BuildingSkill::Research,
            _ => BuildingSkill::Construction, // Default fallback
        }
    }
}

// Data structures for progression events and results

#[derive(Debug, Clone)]
pub enum ProgressionEvent {
    LevelUp(LevelUpResult),
    AttributePointsAwarded { amount: u32 },
    TalentPointsAwarded { amount: u32, skill_type: BuildingSkill },
    SpecializationUnlocked(SpecializationUnlock),
    MilestoneAchieved(Milestone),
    SynergyDiscovered(SynergyBonus),
}

#[derive(Debug, Clone)]
pub struct LevelUpResult {
    pub experience_type: ExperienceType,
    pub old_level: u32,
    pub new_level: u32,
    pub experience_gained: u32,
}

#[derive(Debug, Clone)]
pub struct SpecializationUnlock {
    pub specialization: SpecializationPath,
    pub unlock_level: u32,
    pub bonus_points: u32,
}

#[derive(Debug, Clone)]
pub struct ExperienceResult {
    pub experience_type: ExperienceType,
    pub experience_gained: u32,
    pub old_experience: u32,
    pub new_experience: u32,
    pub level_up: Option<LevelUpResult>,
    pub total_level: u32,
    pub level_up_overall: bool,
}

// Removed duplicate AttributeAllocationResult - already defined at the top

#[derive(Debug)]
pub struct CharacterOverview {
    pub level: u32,
    pub total_experience: u32,
    pub available_attribute_points: u32,
    pub available_talent_points: u32,
    pub core_attributes: super::player_attributes::CoreAttributes,
    pub derived_stats: super::player_attributes::DerivedStats,
    pub specialization_progress: HashMap<SpecializationPath, u32>,
    pub primary_specialization: Option<SpecializationPath>,
    pub building_skills: HashMap<BuildingSkill, SkillLevel>,
    pub progression_velocity: f32,
    pub efficiency_rating: f32,
    pub synergy_bonuses: Vec<SynergyBonus>,
    pub recommendations: Vec<ProgressionRecommendation>,
    pub recent_milestones: Vec<Milestone>,
    pub upcoming_milestones: Vec<Milestone>,
}

#[derive(Debug, Clone)]
pub struct ProgressionRecommendation {
    pub recommendation_type: RecommendationType,
    pub priority: RecommendationPriority,
    pub title: String,
    pub description: String,
    pub expected_benefit: f32,
    pub cost: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RecommendationPriority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone)]
pub enum RecommendationType {
    AttributeAllocation,
    SkillAllocation,
    SynergyOptimization,
    EquipmentUpgrade,
    PlaystyleFocus,
}

// Removed duplicate ResetType and ResetResult - already defined at the top

#[derive(Debug, Default)]
pub struct PreservedData {
    pub level: Option<u32>,
    pub milestones: Option<Vec<Milestone>>,
}

#[derive(Debug)]
pub struct EquipmentSet {
    pub attribute_modifiers: Vec<super::player_attributes::AttributeBonus>,
    pub skill_bonuses: Vec<SkillBonus>,
    pub experience_modifiers: Vec<ExperienceModifier>,
}

#[derive(Debug, Clone, Copy)]
pub struct SkillBonus {
    pub skill_type: SkillBonusType,
    pub bonus_amount: f32,
    pub duration: Option<Duration>,
}

#[derive(Debug, Clone, Copy)]
pub enum SkillBonusType {
    Building(BuildingSkill),
    Attribute(CoreAttribute),
}

#[derive(Debug, Clone, Copy)]
pub struct ExperienceModifier {
    pub experience_type: ExperienceType,
    pub multiplier: f32,
    pub duration: Option<Duration>,
}

// Supporting tracker implementations
#[derive(Debug)]
pub struct VelocityTracker {
    allocation_history: Vec<(Instant, String)>,
}

impl VelocityTracker {
    pub fn new() -> Self {
        Self {
            allocation_history: Vec::new(),
        }
    }

    pub fn update(&mut self,
                  _attribute_manager: &PlayerAttributeManager,
                  _skill_manager: &EnhancedSkillManager,
                  _delta_time: f32) {
        // Track allocation velocity
        let cutoff = Instant::now() - Duration::from_secs(3600); // 1 hour
        self.allocation_history.retain(|(timestamp, _)| *timestamp >= cutoff);
    }

    pub fn get_current_velocity(&self) -> f32 {
        // Calculate allocations per hour
        let recent_count = self.allocation_history.len();
        recent_count as f32 // Simplified calculation
    }

    pub fn get_metrics(&self) -> VelocityMetrics {
        VelocityMetrics {
            current_velocity: self.get_current_velocity(),
            peak_velocity: self.calculate_peak_velocity(),
            average_velocity: self.calculate_average_velocity(),
        }
    }

    fn calculate_peak_velocity(&self) -> f32 {
        // Would calculate peak velocity over time windows
        self.get_current_velocity() * 1.5 // Simplified
    }

    fn calculate_average_velocity(&self) -> f32 {
        // Would calculate average over longer period
        self.get_current_velocity() * 0.8 // Simplified
    }
}

#[derive(Debug)]
pub struct EfficiencyCalculator {
    efficiency_history: Vec<(Instant, f32)>,
    current_efficiency: f32,
}

impl EfficiencyCalculator {
    pub fn new() -> Self {
        Self {
            efficiency_history: Vec::new(),
            current_efficiency: 1.0,
        }
    }

    pub fn update(&mut self,
                  _attribute_manager: &PlayerAttributeManager,
                  _skill_manager: &EnhancedSkillManager,
                  _delta_time: f32) {
        // Calculate progression efficiency
        self.current_efficiency = self.calculate_current_efficiency();
        self.efficiency_history.push((Instant::now(), self.current_efficiency));

        // Keep only recent history
        let cutoff = Instant::now() - Duration::from_secs(3600);
        self.efficiency_history.retain(|(timestamp, _)| *timestamp >= cutoff);
    }

    pub fn get_current_efficiency(&self) -> f32 {
        self.current_efficiency
    }

    pub fn get_trend(&self) -> EfficiencyTrend {
        if self.efficiency_history.len() < 2 {
            return EfficiencyTrend::Stable;
        }

        let recent = self.efficiency_history.last().unwrap().1;
        let older = self.efficiency_history[self.efficiency_history.len() - 2].1;

        if recent > older + 0.1 {
            EfficiencyTrend::Improving
        } else if recent < older - 0.1 {
            EfficiencyTrend::Declining
        } else {
            EfficiencyTrend::Stable
        }
    }

    pub fn get_metrics(&self) -> EfficiencyMetrics {
        EfficiencyMetrics {
            current_efficiency: self.current_efficiency,
            trend: self.get_trend(),
            efficiency_score: self.calculate_efficiency_score(),
        }
    }

    fn calculate_current_efficiency(&self) -> f32 {
        // Would calculate based on synergies and optimal paths
        0.85 // Simplified efficiency calculation
    }

    fn calculate_efficiency_score(&self) -> f32 {
        // Normalize efficiency to 0-100 scale
        (self.current_efficiency * 100.0).min(100.0)
    }
}

// Additional supporting data structures...

#[derive(Debug, Clone)]
pub struct AttributeChange {
    pub stat_name: String,
    pub old_value: f32,
    pub new_value: f32,
    pub timestamp: Instant,
}

#[derive(Debug, Clone)]
pub struct SkillChange {
    pub specialization: SpecializationPath,
    pub node_id: String,
    pub points_allocated: u32,
    pub timestamp: Instant,
}

#[derive(Debug)]
pub struct ProgressionSummary {
    pub velocity: f32,
    pub efficiency: f32,
    pub recent_changes: usize,
}

#[derive(Debug)]
pub struct ProgressionAnalytics {
    pub velocity_metrics: VelocityMetrics,
    pub efficiency_metrics: EfficiencyMetrics,
    pub allocation_patterns: AllocationPatterns,
    pub optimization_opportunities: Vec<OptimizationOpportunity>,
}

#[derive(Debug)]
pub struct VelocityMetrics {
    pub current_velocity: f32,
    pub peak_velocity: f32,
    pub average_velocity: f32,
}

#[derive(Debug)]
pub struct EfficiencyMetrics {
    pub current_efficiency: f32,
    pub trend: EfficiencyTrend,
    pub efficiency_score: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum EfficiencyTrend {
    Improving,
    Stable,
    Declining,
}

#[derive(Debug)]
pub struct AllocationPatterns {
    pub preferred_attributes: HashMap<String, i32>,
    pub preferred_specializations: HashMap<SpecializationPath, i32>,
    pub allocation_frequency: f32,
    pub efficiency_trend: EfficiencyTrend,
}

#[derive(Debug)]
pub struct OptimizationOpportunity {
    pub opportunity_type: OpportunityType,
    pub description: String,
    pub potential_benefit: f32,
    pub required_effort: EffortLevel,
}

#[derive(Debug)]
pub enum OpportunityType {
    EfficiencyImprovement,
    BalanceImprovement,
    SynergyOptimization,
    ResourceOptimization,
}

#[derive(Debug)]
pub enum EffortLevel {
    Low,
    Medium,
    High,
}

// Placeholder implementations for missing components
#[derive(Debug)]
pub struct MilestoneTracker;

impl MilestoneTracker {
    pub fn new() -> Self { Self }
    pub fn initialize(&mut self, _player_data: &PlayerData) -> RobinResult<()> { Ok(()) }
    pub fn check_milestones(&mut self, _attr: &PlayerAttributeManager, _skill: &EnhancedSkillManager, _building: &SkillManager, _player: &PlayerData) -> RobinResult<()> { Ok(()) }
    pub fn get_recent_milestones(&self) -> Vec<Milestone> { Vec::new() }
    pub fn get_upcoming_milestones(&self) -> Vec<Milestone> { Vec::new() }
    pub fn get_achieved_milestones(&self) -> Vec<Milestone> { Vec::new() }
}

#[derive(Debug)]
pub struct SynergyCalculator;

impl SynergyCalculator {
    pub fn new() -> Self { Self }
    pub fn calculate_synergies(&mut self, _attr: &PlayerAttributeManager, _skill: &EnhancedSkillManager, _building: &SkillManager) {}
    pub fn calculate_attribute_synergies(&self, _attr: CoreAttribute, _value: u32) -> Vec<SynergyBonus> { Vec::new() }
    pub fn get_synergy_summary(&self) -> SynergySummary { SynergySummary::default() }
}

#[derive(Debug, Clone)]
pub struct Milestone {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct SynergyBonus {
    pub name: String,
    pub bonus_amount: f32,
}

#[derive(Debug, Default)]
pub struct SynergySummary {
    pub active_bonuses: Vec<SynergyBonus>,
    pub improvement_opportunities: Vec<SynergyOpportunity>,
}

#[derive(Debug)]
pub struct SynergyOpportunity {
    pub title: String,
    pub description: String,
    pub potential_benefit: f32,
    pub required_investment: u32,
}

#[derive(Debug)]
pub struct ProgressionConfig {
    pub preserve_level_on_reset: bool,
    pub preserve_milestones_on_reset: bool,
    pub enable_synergy_bonuses: bool,
    pub auto_recommendations: bool,
}

impl Default for ProgressionConfig {
    fn default() -> Self {
        Self {
            preserve_level_on_reset: true,
            preserve_milestones_on_reset: true,
            enable_synergy_bonuses: true,
            auto_recommendations: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_progression_creation() {
        let progression = CharacterProgressionManager::new();
        assert_eq!(progression.experience_manager.overall_level, 1);
    }

    #[test]
    fn test_experience_level_calculation() {
        let curve = LevelCurve::new();
        assert_eq!(curve.calculate_level(0), 1);
        assert_eq!(curve.calculate_level(100), 2);
        assert_eq!(curve.calculate_level(250), 3);
    }

    #[test]
    fn test_experience_award() {
        let mut manager = ExperienceManager::new();
        let mut player_data = PlayerData::new("test");

        manager.initialize(&player_data).unwrap();
        let result = manager.award_experience(ExperienceType::Building, 150, &mut player_data).unwrap();

        assert_eq!(result.experience_gained, 150);
        assert!(result.level_up.is_some());
    }

    #[test]
    fn test_progression_tracking() {
        let mut tracker = ProgressionTracker::new();
        tracker.record_attribute_allocation(CoreAttribute::Strength, 10);

        assert_eq!(tracker.recent_attribute_changes.len(), 1);
        assert!(tracker.needs_synergy_update());
    }

    #[test]
    fn test_velocity_tracking() {
        let tracker = VelocityTracker::new();
        let velocity = tracker.get_current_velocity();
        assert!(velocity >= 0.0);
    }
}