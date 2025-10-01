/*!
 * Core Gameplay Systems for Robin Engine
 *
 * Voxel-focused gameplay mechanics including resource management,
 * crafting systems, building progression, and achievement tracking.
 * Integrates with existing PlayerData and GameProgress systems.
 */

use crate::engine::{
    error::{RobinError, RobinResult},
    save_system::{PlayerData, GameProgress},
    world::VoxelType,
    math::Vec3,
};
use self::blueprint_system::IPos3;
use serde::{Serialize, Deserialize};

pub mod resources;
pub mod crafting;
pub mod advanced_crafting;
pub mod blueprint_system;
pub mod automated_building;
pub mod interactive_building;
pub mod progression;
pub mod skill_tree;
pub mod player_attributes;
pub mod stat_monitoring;
pub mod character_progression;
pub mod objectives;
pub mod achievements;
pub mod quest_system;
pub mod reputation;
pub mod social_systems;

// Core gameplay exports
pub use resources::{ResourceManager, Resource, ResourceType, ResourceProperty};
pub use crafting::{CraftingManager, Recipe, CraftingRecipe, CraftingResult};
pub use advanced_crafting::{
    AdvancedCraftingManager, AdvancedRecipe, CraftingStation,
    EnvironmentalCondition, RecipeDiscoverySystem, ActiveCraftingProcess,
    QualityControlSystem, ItemQuality, CraftingResult as AdvancedCraftingResult
};
pub use blueprint_system::{
    BlueprintManager, Blueprint, CommunityTemplate, ActiveConstruction,
    PatternRecognitionEngine, MaterialOptimizer, ConstructionAI, TemplateSharingNetwork,
    ConstructionStage, StructuralPattern,
    ConstructionProgress, StructureData
};
pub use automated_building::{
    AutomatedBuildingManager, AutomatedProject, TerrainAnalyzer, MaterialLogisticsSystem,
    ConstructionDroneFleet, ConstructionOptimizer, BuildingAssistant, AutomationLevel,
    ConstructionPhase, ProjectStatusReport, OptimizationResult, SmartRecommendation,
    BuildingContext, AutomationControlAction, ConstructionRecommendation
};
pub use interactive_building::{
    InteractiveBuildingManager, GestureTracker, CollaborativeBuildingInterface,
    VisualizationEngine, SnappingSystem, ToolPalette, InteractionState,
    BuildingGesture, InteractionMode, GestureEvent, CollaborativeUpdate,
    InteractionPreview, SmartSuggestion, UserPreferences, AccessibilityFeatures
};
pub use progression::{SkillManager, BuildingSkill, SkillLevel, MasteryBonus};
pub use skill_tree::{
    EnhancedSkillManager, SpecializationPath, SkillTree, SkillNode,
    SkillAllocationResult, SpecializationSummary, TalentPoints
};
pub use player_attributes::{
    PlayerAttributeManager, CoreAttributes, DerivedStats, EquipmentModifiers,
    TemporaryEffect, AttributePerformanceMonitor
};
pub use stat_monitoring::{
    StatMonitoringSystem, StatDashboardData, PerformanceReport, StatTrends,
    StatEvent, StatEventType, EventSource, StatAlert, AlertSeverity
};
pub use character_progression::{
    CharacterProgressionManager, ExperienceType, ProgressionEvent, LevelUpResult,
    CharacterOverview, ProgressionRecommendation, ResetType, ExperienceManager,
    ProgressionAnalytics, AttributeAllocationResult
};
pub use objectives::{ObjectiveManager, Objective, ObjectiveType, ObjectiveStatus};
pub use achievements::{AchievementManager, Achievement, AchievementCondition, AchievementReward};
pub use quest_system::{QuestManager, Quest, QuestObjective, QuestType, QuestEvent};
pub use reputation::{
    ReputationManager, FactionId, FactionType, FactionStanding, ReputationTier,
    NpcId, NpcType, NpcRelationship, RelationshipStatus, InteractionType,
    CommunityReputation, CommunityStanding, ReputationChange, RelationshipChange,
    ReputationModifiers, ReputationSummary
};
pub use social_systems::{
    SocialSystemsManager, GuildManager, GuildId, GuildType, Guild, GuildMembership,
    CollaborationSystem, CollaborationProject, CommunityEventManager, CommunityEvent,
    SocialNetworkManager, MentorshipSystem, SocialOverview, CommunityContributions
};

/// Core gameplay manager that orchestrates all gameplay systems
pub struct GameplayManager {
    pub resources: ResourceManager,
    pub crafting: CraftingManager,
    pub advanced_crafting: AdvancedCraftingManager,
    pub blueprints: BlueprintManager,
    pub automated_building: AutomatedBuildingManager,
    pub interactive_building: InteractiveBuildingManager,
    pub skills: SkillManager,
    pub objectives: ObjectiveManager,
    pub achievements: AchievementManager,
    pub quests: QuestManager,

    // Enhanced progression system
    pub character_progression: CharacterProgressionManager,
    pub stat_monitoring: StatMonitoringSystem,

    // Social and reputation systems
    pub reputation: ReputationManager,
    pub social_systems: SocialSystemsManager,

    // Statistics tracking
    pub session_stats: SessionStats,
}

impl GameplayManager {
    pub fn new() -> Self {
        Self {
            resources: ResourceManager::new(),
            crafting: CraftingManager::new(),
            advanced_crafting: AdvancedCraftingManager::new(),
            blueprints: BlueprintManager::new(),
            automated_building: AutomatedBuildingManager::new(),
            interactive_building: InteractiveBuildingManager::new(),
            skills: SkillManager::new(),
            objectives: ObjectiveManager::new(),
            achievements: AchievementManager::new(),
            quests: QuestManager::new(),
            character_progression: CharacterProgressionManager::new(),
            stat_monitoring: StatMonitoringSystem::new(),
            reputation: ReputationManager::new(),
            social_systems: SocialSystemsManager::new(),
            session_stats: SessionStats::default(),
        }
    }

    /// Initialize all gameplay systems
    pub fn initialize(&mut self, player_data: &PlayerData) -> RobinResult<()> {
        // Initialize unified character progression system
        self.character_progression.initialize(player_data)?;

        // Initialize monitoring system with progression manager integration
        self.stat_monitoring.initialize(&self.character_progression.attribute_manager)?;

        // Initialize advanced crafting system
        self.advanced_crafting.initialize();

        // Initialize blueprint and template system
        self.blueprints.initialize(player_data)?;

        // Initialize automated building system
        self.automated_building.initialize(player_data)?;

        // Initialize interactive building tools
        // Note: Interactive building doesn't need player data initialization as it manages real-time state
        println!("🔧 Interactive Building Tools initialized with gesture recognition, collaborative interface, and intelligent snapping");

        // Initialize reputation and social systems
        self.reputation.initialize(player_data)?;
        self.social_systems.initialize(player_data, &self.reputation)?;

        println!("🎮 GameplayManager initialized with unified systems: character progression, advanced crafting, blueprints, automated building, interactive tools, reputation, and social systems");
        Ok(())
    }

    /// Update all gameplay systems
    pub fn update(&mut self, delta_time: f32, player_data: &mut PlayerData, progress: &mut GameProgress) -> RobinResult<()> {
        // Update session stats
        self.session_stats.total_play_time += delta_time;

        // Update unified character progression system
        self.character_progression.update(delta_time, player_data, &mut self.stat_monitoring)?;

        // Update stat monitoring and performance analytics
        self.stat_monitoring.update(&self.character_progression.attribute_manager, delta_time)?;

        // Update advanced crafting system (process active crafting)
        self.advanced_crafting.update(delta_time, player_data, &self.reputation)?;

        // Update blueprint system (process active constructions)
        // TODO: Pass actual voxel world reference when integrated with world system
        let mut temp_voxel_world = std::collections::HashMap::new();
        self.blueprints.update(delta_time, player_data, &mut temp_voxel_world)?;

        // Update automated building system (drones, optimization, logistics)
        self.automated_building.update(delta_time)?;

        // Update interactive building tools (gesture tracking, collaborative interfaces, snapping)
        self.interactive_building.update(delta_time)?;

        // Update objectives
        self.objectives.update(delta_time, player_data, progress)?;

        // Check for achievement unlocks
        self.achievements.check_achievements(player_data, &self.session_stats)?;

        // Update skill progression based on recent actions
        self.skills.process_skill_gains(player_data)?;

        // Update reputation and social systems
        self.reputation.update(delta_time, player_data)?;
        self.social_systems.update(delta_time, player_data, &mut self.reputation)?;

        Ok(())
    }

    /// Get real-time attribute dashboard data
    pub fn get_attribute_dashboard(&self) -> StatDashboardData {
        self.stat_monitoring.get_dashboard_data()
    }

    /// Get performance analytics report
    pub fn get_performance_report(&self) -> PerformanceReport {
        self.stat_monitoring.get_performance_report()
    }

    /// Get stat trend analysis
    pub fn get_stat_trends(&self) -> StatTrends {
        self.stat_monitoring.get_stat_trends()
    }

    /// Record a stat event for monitoring
    pub fn record_stat_event(&mut self, event: StatEvent) {
        self.stat_monitoring.record_stat_event(event);
    }

    /// Get comprehensive character overview
    pub fn get_character_overview(&self) -> CharacterOverview {
        self.character_progression.get_character_overview()
    }

    /// Award experience to player
    pub fn award_experience(&mut self,
                           experience_type: ExperienceType,
                           amount: u32,
                           player_data: &mut PlayerData) -> RobinResult<Vec<ProgressionEvent>> {
        self.character_progression.award_experience(experience_type, amount, player_data)
    }

    /// Allocate attribute point
    pub fn allocate_attribute_point(&mut self,
                                   attribute: character_progression::CoreAttribute,
                                   player_data: &mut PlayerData) -> RobinResult<character_progression::AttributeAllocationResult> {
        self.character_progression.allocate_attribute_point(attribute, player_data)
    }

    /// Allocate talent point in skill tree
    pub fn allocate_talent_point(&mut self,
                                specialization: SpecializationPath,
                                node_id: &str,
                                player_data: &mut PlayerData) -> RobinResult<SkillAllocationResult> {
        self.character_progression.allocate_talent_point(specialization, node_id, player_data)
    }

    /// Get progression recommendations
    pub fn get_progression_recommendations(&self) -> Vec<ProgressionRecommendation> {
        self.character_progression.get_progression_recommendations()
    }

    /// Reset character progression
    pub fn reset_character_progression(&mut self,
                                      reset_type: character_progression::ResetType,
                                      player_data: &mut PlayerData) -> RobinResult<character_progression::ResetResult> {
        self.character_progression.reset_character_progression(reset_type, player_data)
    }

    /// Get progression analytics
    pub fn get_progression_analytics(&self) -> ProgressionAnalytics {
        self.character_progression.get_progression_analytics()
    }

    /// Modify faction standing
    pub fn modify_faction_standing(&mut self,
                                  faction_id: FactionId,
                                  amount: i32,
                                  reason: String,
                                  player_data: &mut PlayerData) -> RobinResult<ReputationChange> {
        self.reputation.modify_faction_standing(faction_id, amount, reason, player_data)
    }

    /// Modify NPC relationship
    pub fn modify_npc_relationship(&mut self,
                                  npc_id: NpcId,
                                  amount: i32,
                                  interaction_type: InteractionType,
                                  player_data: &mut PlayerData) -> RobinResult<RelationshipChange> {
        self.reputation.modify_npc_relationship(npc_id, amount, interaction_type, player_data)
    }

    /// Get reputation summary
    pub fn get_reputation_summary(&self) -> ReputationSummary {
        self.reputation.get_reputation_summary()
    }

    /// Get reputation-based modifiers
    pub fn get_reputation_modifiers(&self, player_data: &PlayerData) -> ReputationModifiers {
        self.reputation.get_reputation_modifiers(player_data)
    }

    /// Check faction access permissions
    pub fn can_access_faction_content(&self, faction_id: &FactionId, required_tier: ReputationTier) -> bool {
        self.reputation.can_access_faction_content(faction_id, required_tier)
    }

    /// Get comprehensive social overview
    pub fn get_social_overview(&self) -> SocialOverview {
        self.social_systems.get_social_overview()
    }

    /// Create a new guild
    pub fn create_guild(&mut self,
                       founder_id: String,
                       guild_name: String,
                       guild_type: GuildType,
                       description: String) -> RobinResult<GuildId> {
        self.social_systems.guild_manager.create_guild(founder_id, guild_name, guild_type, description)
    }

    /// Join an existing guild
    pub fn join_guild(&mut self,
                     player_id: String,
                     guild_id: &GuildId,
                     invitation_code: Option<String>) -> RobinResult<GuildMembership> {
        self.social_systems.guild_manager.join_guild(player_id, guild_id, invitation_code)
    }

    /// Start a collaboration project
    pub fn start_collaboration(&mut self,
                              initiator_id: String,
                              project_type: social_systems::CollaborationType,
                              description: String,
                              requirements: social_systems::CollaborationRequirements) -> RobinResult<String> {
        self.social_systems.collaboration_system.start_collaboration(initiator_id, project_type, description, requirements)
    }

    /// Join a collaboration project
    pub fn join_collaboration(&mut self,
                             player_id: String,
                             project_id: String,
                             contribution_type: social_systems::ContributionType) -> RobinResult<()> {
        self.social_systems.collaboration_system.join_collaboration(player_id, project_id, contribution_type)
    }

    /// Handle voxel placement (core building mechanic)
    pub fn handle_voxel_placed(&mut self, voxel_type: VoxelType, position: Vec3, player_data: &mut PlayerData) -> RobinResult<()> {
        // Consume resources from inventory
        let resource_type = ResourceType::from_voxel(voxel_type);
        if !self.resources.consume_resource(player_data, &resource_type, 1) {
            return Err(RobinError::InsufficientResources(format!("Need 1 {:?}", resource_type)));
        }

        // Track building stats
        self.session_stats.blocks_placed += 1;
        player_data.stats.custom_stats.entry("blocks_placed".to_string()).and_modify(|v| *v += 1.0).or_insert(1.0);

        // Award building experience
        self.skills.award_experience(BuildingSkill::Construction, 10, player_data)?;

        // Check for building-related objectives
        self.objectives.handle_event(ObjectiveEvent::VoxelPlaced { voxel_type, position }, player_data)?;

        // Award reputation for construction activities (Builders Guild)
        if let Some(builders_guild) = self.reputation.faction_standings.keys()
            .find(|f| f.name == "Builders Guild") {
            let _ = self.reputation.modify_faction_standing(
                builders_guild.clone(),
                2, // Small reputation gain for building
                "Block placed".to_string(),
                player_data
            );
        }

        Ok(())
    }

    /// Handle voxel removal (mining/destruction)
    pub fn handle_voxel_removed(&mut self, voxel_type: VoxelType, position: Vec3, player_data: &mut PlayerData) -> RobinResult<()> {
        // Award resources to inventory
        let resource_type = ResourceType::from_voxel(voxel_type);
        let resource_yield = self.resources.get_mining_yield(&resource_type);

        for _ in 0..resource_yield {
            player_data.add_item(&resource_type.to_item_id(), 1);
        }

        // Track mining stats
        self.session_stats.blocks_mined += 1;
        player_data.stats.custom_stats.entry("blocks_mined".to_string()).and_modify(|v| *v += 1.0).or_insert(1.0);

        // Award mining experience
        self.skills.award_experience(BuildingSkill::Mining, 5, player_data)?;

        // Check for mining-related objectives
        self.objectives.handle_event(ObjectiveEvent::VoxelMined { voxel_type, position }, player_data)?;

        // Award reputation for mining activities (Miners Union)
        if let Some(miners_union) = self.reputation.faction_standings.keys()
            .find(|f| f.name == "Miners Union") {
            let _ = self.reputation.modify_faction_standing(
                miners_union.clone(),
                3, // Slightly more reputation for mining (it's harder work)
                "Block mined".to_string(),
                player_data
            );
        }

        Ok(())
    }

    /// Handle crafting attempt
    pub fn handle_crafting(&mut self, recipe_id: &str, player_data: &mut PlayerData) -> RobinResult<CraftingResult> {
        let result = self.crafting.craft_item(recipe_id, player_data, &self.skills)?;

        if result.success {
            // Track crafting stats
            self.session_stats.items_crafted += 1;
            player_data.stats.custom_stats.entry("items_crafted".to_string()).and_modify(|v| *v += 1.0).or_insert(1.0);

            // Award crafting experience
            self.skills.award_experience(BuildingSkill::Crafting, result.experience_gained, player_data)?;

            // Check for crafting objectives
            self.objectives.handle_event(ObjectiveEvent::ItemCrafted { recipe_id: recipe_id.to_string() }, player_data)?;

            // Award reputation for crafting activities (Crafters Consortium)
            if let Some(crafters_consortium) = self.reputation.faction_standings.keys()
                .find(|f| f.name == "Crafters Consortium") {
                let _ = self.reputation.modify_faction_standing(
                    crafters_consortium.clone(),
                    5, // Good reputation gain for successful crafting
                    format!("Crafted {}", recipe_id),
                    player_data
                );
            }
        }

        Ok(result)
    }

    /// Start an advanced multi-stage crafting process
    pub fn start_advanced_crafting(&mut self,
                                  recipe_id: &str,
                                  crafting_station_id: &str,
                                  player_data: &mut PlayerData) -> RobinResult<String> {
        let process_id = self.advanced_crafting.start_crafting_process(
            recipe_id,
            player_data
        )?;

        // Track advanced crafting attempt
        self.session_stats.items_crafted += 1;
        player_data.stats.custom_stats.entry("advanced_crafts_started".to_string())
            .and_modify(|v| *v += 1.0)
            .or_insert(1.0);

        Ok(process_id)
    }

    /// Continue a multi-stage crafting process
    pub fn continue_advanced_crafting(&mut self,
                                     process_id: &str,
                                     _player_data: &mut PlayerData) -> RobinResult<AdvancedCraftingResult> {
        self.advanced_crafting.continue_crafting_process(
            process_id
        )?;

        // TODO: Return actual result from crafting process
        Ok(AdvancedCraftingResult {
            completed: false,
            current_stage: 0,
            total_stages: 1,
            item_id: None,
            quality_achieved: None,
            experience_gained: None,
            time_remaining: None,
            next_stage_requirements: None,
            errors: Vec::new(),
        })
    }

    /// Discover new recipes through experimentation
    pub fn attempt_recipe_discovery(&mut self,
                                   materials: Vec<String>,
                                   player_data: &mut PlayerData) -> RobinResult<Option<String>> {
        // Convert material strings to ResourceType - default to Earth for unknown materials
        let resource_types: Vec<ResourceType> = materials.iter().map(|s| {
            match s.to_lowercase().as_str() {
                "earth" | "dirt" => ResourceType::Earth,
                "stone" => ResourceType::Stone,
                "water" => ResourceType::Water,
                "grass" => ResourceType::Grass,
                "sand" => ResourceType::Sand,
                "metal" => ResourceType::Metal,
                "crystal" => ResourceType::Crystal,
                "wood" => ResourceType::Wood,
                "lava" => ResourceType::Lava,
                "glass" => ResourceType::Glass,
                "refinedstone" => ResourceType::RefinedStone,
                "enhancedmetal" => ResourceType::EnhancedMetal,
                _ => ResourceType::Earth, // Default for unknown materials
            }
        }).collect();

        let discovered_recipe = self.advanced_crafting.attempt_recipe_discovery(
            &resource_types,
            player_data,
            &self.character_progression.building_skills
        )?;

        if discovered_recipe.is_some() {
            // Award experience for recipe discovery
            let _ = self.character_progression.award_experience(
                crate::engine::gameplay::character_progression::ExperienceType::Crafting,
                100, // Significant experience for discovery
                player_data
            );

            // Track discovery stats
            player_data.stats.custom_stats.entry("recipes_discovered".to_string())
                .and_modify(|v| *v += 1.0)
                .or_insert(1.0);
        }

        Ok(discovered_recipe)
    }

    /// Get available advanced crafting recipes for player
    pub fn get_available_advanced_recipes(&self, player_data: &PlayerData) -> Vec<String> {
        self.advanced_crafting.get_available_recipes(player_data)
    }

    /// Get crafting station information
    pub fn get_crafting_station_info(&self, station_id: &str) -> Option<CraftingStation> {
        self.advanced_crafting.get_crafting_station(station_id).cloned()
    }

    /// Save current area as blueprint
    pub fn save_blueprint(&mut self,
                         blueprint_name: String,
                         origin: Vec3,
                         size: Vec3,
                         voxel_data: &std::collections::HashMap<IPos3, crate::engine::world::VoxelType>,
                         player_data: &mut PlayerData) -> RobinResult<String> {
        self.blueprints.save_blueprint_from_structure(blueprint_name, origin, size, voxel_data, player_data)
    }

    /// Start auto-construction from blueprint
    pub fn start_auto_construction(&mut self,
                                  blueprint_id: &str,
                                  construction_origin: Vec3,
                                  player_data: &mut PlayerData) -> RobinResult<String> {
        let construction_id = self.blueprints.start_auto_construction(
            blueprint_id,
            construction_origin,
            player_data,
            &self.advanced_crafting,
            &self.character_progression
        )?;

        // Track automated construction
        player_data.stats.custom_stats.entry("auto_constructions_started".to_string())
            .and_modify(|v| *v += 1.0)
            .or_insert(1.0);

        // Award experience for automation usage
        let _ = self.character_progression.award_experience(
            crate::engine::gameplay::character_progression::ExperienceType::Building,
            25, // Experience for starting automated construction
            player_data
        );

        Ok(construction_id)
    }

    /// Get available blueprints for player
    pub fn get_available_blueprints(&self) -> Vec<String> {
        self.blueprints.saved_blueprints.keys().cloned().collect()
    }

    /// Get active constructions for player
    pub fn get_active_constructions(&self) -> Vec<String> {
        self.blueprints.active_constructions.keys().cloned().collect()
    }

    /// Start an automated construction project with intelligent planning
    pub fn start_automated_construction_project(&mut self,
                                               blueprint_id: &str,
                                               project_name: String,
                                               construction_site: Vec3,
                                               automation_level: AutomationLevel,
                                               player_data: &mut PlayerData) -> RobinResult<String> {
        let project_id = self.automated_building.start_automated_project(
            blueprint_id,
            project_name,
            construction_site,
            automation_level,
            player_data,
            &self.blueprints
        )?;

        // Track automated project starts
        player_data.stats.custom_stats.entry("automated_projects_initiated".to_string())
            .and_modify(|v| *v += 1.0)
            .or_insert(1.0);

        // Award experience for project management
        let _ = self.character_progression.award_experience(
            crate::engine::gameplay::character_progression::ExperienceType::Building,
            75, // Significant experience for project initiation
            player_data
        );

        // Award reputation for advanced construction methods
        if let Some(builders_guild) = self.reputation.faction_standings.keys()
            .find(|f| f.name == "Builders Guild") {
            let _ = self.reputation.modify_faction_standing(
                builders_guild.clone(),
                20, // Good reputation gain for automation usage
                "Automated construction project started".to_string(),
                player_data
            );
        }

        Ok(project_id)
    }

    /// Get intelligent terrain analysis for construction planning
    pub fn analyze_construction_site(&mut self,
                                   site_location: Vec3,
                                   structure_data: &StructureData) -> RobinResult<automated_building::SiteAnalysis> {
        self.automated_building.analyze_construction_site(site_location, structure_data)
    }

    /// Get AI-powered construction recommendations
    pub fn get_construction_recommendations(&self,
                                          site_location: Vec3,
                                          player_data: &PlayerData) -> Vec<ConstructionRecommendation> {
        // Create user preferences from player data
        let user_preferences = automated_building::UserPreferences {
            user_id: player_data.name.clone(),
            building_style_preferences: std::collections::HashMap::new(),
            automation_comfort_level: automated_building::AutomationComfortLevel::Moderate,
            preferred_materials: vec![],
            budget_constraints: automated_building::BudgetConstraints {
                max_budget: 10000.0,
                cost_priorities: std::collections::HashMap::new(),
            },
            quality_priorities: automated_building::QualityPriorities {
                priority_weights: std::collections::HashMap::new(),
            },
            environmental_consciousness: 0.8,
            innovation_openness: 0.7,
        };

        self.automated_building.get_construction_recommendations(site_location, &user_preferences)
    }

    /// Get real-time status of automated construction project
    pub fn get_automated_project_status(&self, project_id: &str) -> Option<ProjectStatusReport> {
        self.automated_building.get_project_status(project_id)
    }

    /// Optimize ongoing automated construction
    pub fn optimize_automated_construction(&mut self, project_id: &str) -> RobinResult<OptimizationResult> {
        self.automated_building.optimize_construction_process(project_id)
    }

    /// Get smart assistant recommendations for current building context
    pub fn get_smart_building_recommendations(&self,
                                            current_location: Vec3,
                                            available_resources: std::collections::HashMap<crate::engine::gameplay::resources::ResourceType, u32>,
                                            player_data: &PlayerData) -> Vec<SmartRecommendation> {

        let context = BuildingContext {
            current_location,
            current_project: None,
            available_resources,
            recent_actions: vec![],
            environmental_conditions: automated_building::EnvironmentalConditions {
                temperature_range: (15.0, 25.0),
                humidity_levels: 0.6,
                wind_exposure: 0.3,
                precipitation_patterns: vec![0.1, 0.2, 0.15],
                sunlight_exposure: 0.8,
                natural_hazards: vec![],
            },
        };

        self.automated_building.get_smart_recommendations(context, player_data)
    }

    /// Control automated construction (pause, resume, stop)
    pub fn control_automated_construction(&mut self,
                                        project_id: &str,
                                        action: AutomationControlAction,
                                        player_data: &mut PlayerData) -> RobinResult<()> {
        self.automated_building.control_automation(project_id, action)?;

        // Track automation control usage
        player_data.stats.custom_stats.entry("automation_controls_used".to_string())
            .and_modify(|v| *v += 1.0)
            .or_insert(1.0);

        Ok(())
    }

    /// Get active automated construction projects
    pub fn get_active_automated_projects(&self) -> Vec<String> {
        self.automated_building.active_projects.keys().cloned().collect()
    }

    /// Get construction drone fleet status
    pub fn get_drone_fleet_status(&self) -> std::collections::HashMap<String, String> {
        self.automated_building.construction_drones.active_drones.iter()
            .map(|(id, drone)| (id.clone(), drone.current_status.status.clone()))
            .collect()
    }

    /// Get material logistics status for automated projects
    pub fn get_material_logistics_status(&self) -> Vec<String> {
        self.automated_building.material_logistics.automated_sourcing.keys().cloned().collect()
    }

    // === Interactive Building Tools Methods ===

    /// Process input for gesture-based building controls
    pub fn process_building_input(&mut self, input_event: interactive_building::InputEvent) -> RobinResult<GestureEvent> {
        self.interactive_building.process_input(input_event)
    }

    /// Switch the current interaction mode for building tools
    pub fn switch_building_mode(&mut self, new_mode: InteractionMode) -> RobinResult<()> {
        self.interactive_building.switch_interaction_mode(new_mode)
    }

    /// Start a collaborative building session
    pub fn start_collaborative_building(&mut self,
                                       session_id: String,
                                       participants: Vec<String>) -> RobinResult<interactive_building::CollaborativeSession> {
        self.interactive_building.start_collaboration_session(session_id, participants)
    }

    /// Update collaborative building state with real-time updates
    pub fn update_collaborative_building(&mut self, updates: Vec<CollaborativeUpdate>) -> RobinResult<()> {
        self.interactive_building.update_collaborative_state(updates)
    }

    /// Apply intelligent snapping to a position based on context
    pub fn apply_intelligent_snapping(&self,
                                     position: cgmath::Point3<f32>,
                                     context: &interactive_building::SnappingContext) -> cgmath::Point3<f32> {
        self.interactive_building.apply_snapping(position, context)
    }

    /// Generate real-time preview for current building interaction
    pub fn generate_building_preview(&self,
                                    interaction: &interactive_building::CurrentInteraction) -> RobinResult<InteractionPreview> {
        self.interactive_building.generate_preview(interaction)
    }

    /// Get intelligent building suggestions based on current context
    pub fn get_intelligent_building_suggestions(&self,
                                               context: &interactive_building::BuildingContext) -> Vec<interactive_building::SmartSuggestion> {
        self.interactive_building.get_building_suggestions(context)
    }

    /// Configure user preferences for interactive building
    pub fn set_building_preferences(&mut self, preferences: UserPreferences) {
        self.interactive_building.user_preferences = preferences;
    }

    /// Get current building tool palette
    pub fn get_building_tool_palette(&self) -> &ToolPalette {
        &self.interactive_building.tool_palette
    }

    /// Switch active building tool
    pub fn switch_building_tool(&mut self, tool_id: String) -> RobinResult<()> {
        if self.interactive_building.tool_palette.available_tools.contains_key(&tool_id) {
            self.interactive_building.tool_palette.active_tool = tool_id;
            Ok(())
        } else {
            Err(format!("Tool '{}' not available", tool_id).into())
        }
    }

    /// Get current interaction state for building tools
    pub fn get_building_interaction_state(&self) -> &InteractionState {
        &self.interactive_building.interaction_state
    }

    /// Get building gesture tracker state
    pub fn get_gesture_tracker_state(&self) -> &GestureTracker {
        &self.interactive_building.gesture_tracker
    }

    /// Get collaborative building interface state
    pub fn get_collaborative_interface_state(&self) -> &CollaborativeBuildingInterface {
        &self.interactive_building.collaborative_interface
    }

    /// Get visualization engine for building previews
    pub fn get_visualization_engine(&self) -> &VisualizationEngine {
        &self.interactive_building.visualization_engine
    }

    /// Get snapping system configuration
    pub fn get_snapping_system(&self) -> &SnappingSystem {
        &self.interactive_building.snapping_system
    }

    /// Configure grid snapping settings
    pub fn configure_grid_snapping(&mut self,
                                  enabled: bool,
                                  grid_size: f32,
                                  adaptive: bool) {
        self.interactive_building.snapping_system.grid_snapping.enabled = enabled;
        self.interactive_building.snapping_system.grid_snapping.grid_size = grid_size;
        self.interactive_building.snapping_system.grid_snapping.adaptive_grid = adaptive;
    }

    /// Get accessibility features for building tools
    pub fn get_building_accessibility_features(&self) -> &AccessibilityFeatures {
        &self.interactive_building.accessibility_features
    }

    /// Track building tool usage statistics
    pub fn track_tool_usage(&mut self, tool_id: &str, usage_duration: f32) {
        if let Some(tool_stats) = self.interactive_building.tool_palette.tool_usage_stats.get_mut(tool_id) {
            tool_stats.total_usage_time += usage_duration;
            tool_stats.usage_count += 1;
            tool_stats.last_used = std::time::Instant::now();
        }
    }

    /// Get performance metrics for interactive building systems
    pub fn get_interactive_building_performance(&self) -> interactive_building::InteractionPerformanceMonitor {
        self.interactive_building.performance_monitor.clone()
    }

    /// Get current gameplay statistics
    pub fn get_session_summary(&self) -> SessionSummary {
        SessionSummary {
            play_time: self.session_stats.total_play_time,
            blocks_placed: self.session_stats.blocks_placed,
            blocks_mined: self.session_stats.blocks_mined,
            items_crafted: self.session_stats.items_crafted,
            objectives_completed: self.session_stats.objectives_completed,
            achievements_unlocked: self.session_stats.achievements_unlocked,
        }
    }
}

/// Session-specific statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionStats {
    pub total_play_time: f32,
    pub blocks_placed: u32,
    pub blocks_mined: u32,
    pub items_crafted: u32,
    pub objectives_completed: u32,
    pub achievements_unlocked: u32,
}

/// Summary of gameplay session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub play_time: f32,
    pub blocks_placed: u32,
    pub blocks_mined: u32,
    pub items_crafted: u32,
    pub objectives_completed: u32,
    pub achievements_unlocked: u32,
}

/// Events that can trigger objectives and achievements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectiveEvent {
    VoxelPlaced { voxel_type: VoxelType, position: Vec3 },
    VoxelMined { voxel_type: VoxelType, position: Vec3 },
    ItemCrafted { recipe_id: String },
    StructureCompleted { structure_type: String, size: u32 },
    SkillLevelUp { skill: BuildingSkill, new_level: u32 },
    AchievementUnlocked { achievement_id: String },
}

impl Default for GameplayManager {
    fn default() -> Self {
        Self::new()
    }
}