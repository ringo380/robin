/*!
 * Comprehensive Player Progression System for Robin Engine
 *
 * Integrates XP, levels, skill trees, unlockable content, and achievements
 * into a cohesive progression system that rewards building activities.
 */

use crate::engine::{
    error::{RobinError, RobinResult},
    save_system::{PlayerData, GameProgress},
    world::VoxelType,
    math::Vec3,
};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc};

/// Central progression system that orchestrates player advancement
pub struct ProgressionSystem {
    /// Player level and experience tracking
    pub player_level: PlayerLevel,
    /// Skill tree progression
    pub skill_trees: SkillTreeManager,
    /// Content unlocking system
    pub unlockables: UnlockableContentManager,
    /// Achievement tracking
    pub achievements: AchievementTracker,
    /// Configuration for progression rates
    pub config: ProgressionConfig,
}

impl ProgressionSystem {
    pub fn new() -> Self {
        Self {
            player_level: PlayerLevel::new(),
            skill_trees: SkillTreeManager::new(),
            unlockables: UnlockableContentManager::new(),
            achievements: AchievementTracker::new(),
            config: ProgressionConfig::default(),
        }
    }

    /// Initialize from saved player data
    pub fn load_from_save(&mut self, player_data: &PlayerData) -> RobinResult<()> {
        // Load player level from stats
        if let Some(level) = player_data.stats.custom_stats.get("player_level") {
            self.player_level.level = *level as u32;
        }
        if let Some(xp) = player_data.stats.custom_stats.get("player_xp") {
            self.player_level.experience = *xp as u64;
        }
        if let Some(total_xp) = player_data.stats.custom_stats.get("total_xp") {
            self.player_level.total_experience = *total_xp as u64;
        }

        // Load skill tree progress
        self.skill_trees.load_from_save(player_data)?;
        
        // Load unlocked content
        self.unlockables.load_from_save(player_data)?;
        
        // Load achievement progress
        self.achievements.load_from_save(player_data)?;

        Ok(())
    }

    /// Save current progression to player data
    pub fn save_to_data(&self, player_data: &mut PlayerData) -> RobinResult<()> {
        // Save player level
        player_data.stats.custom_stats.insert("player_level".to_string(), self.player_level.level as f64);
        player_data.stats.custom_stats.insert("player_xp".to_string(), self.player_level.experience as f64);
        player_data.stats.custom_stats.insert("total_xp".to_string(), self.player_level.total_experience as f64);

        // Save subsystem data
        self.skill_trees.save_to_data(player_data)?;
        self.unlockables.save_to_data(player_data)?;
        self.achievements.save_to_data(player_data)?;

        Ok(())
    }

    /// Award experience for a building action
    pub fn award_building_xp(&mut self, action: BuildingAction, complexity: f32, player_data: &mut PlayerData) -> RobinResult<Vec<ProgressionEvent>> {
        let mut events = Vec::new();

        // Calculate base XP for action
        let base_xp = match action {
            BuildingAction::PlaceBlock { voxel_type, .. } => {
                self.config.get_block_placement_xp(voxel_type) * complexity
            },
            BuildingAction::RemoveBlock { .. } => self.config.base_mining_xp * complexity,
            BuildingAction::CompleteStructure { size, .. } => {
                self.config.structure_completion_xp * (size as f32).sqrt()
            },
            BuildingAction::UseTool { tool_tier, .. } => {
                self.config.tool_usage_xp * (tool_tier as f32)
            },
            BuildingAction::CreateBlueprint { .. } => self.config.blueprint_creation_xp * complexity,
            BuildingAction::ShareProject { .. } => self.config.collaboration_xp,
        };

        let final_xp = (base_xp * self.get_xp_multiplier()) as u64;

        // Award player XP
        if let Some(level_up) = self.player_level.add_experience(final_xp) {
            events.push(ProgressionEvent::PlayerLevelUp(level_up));
            
            // Check for level-based unlocks
            let unlocks = self.unlockables.check_level_unlocks(self.player_level.level);
            for unlock in unlocks {
                events.push(ProgressionEvent::ContentUnlocked(unlock));
            }
        }

        // Award skill tree XP
        let skill_events = self.skill_trees.award_action_xp(&action, final_xp, player_data)?;
        events.extend(skill_events.into_iter().map(ProgressionEvent::SkillProgress));

        // Check achievements
        let achievement_events = self.achievements.check_building_action(&action, player_data)?;
        events.extend(achievement_events.into_iter().map(ProgressionEvent::Achievement));

        // Update player data
        self.save_to_data(player_data)?;

        Ok(events)
    }

    /// Get current XP multiplier based on unlocked bonuses
    fn get_xp_multiplier(&self) -> f32 {
        let mut multiplier = 1.0;
        
        // Skill tree bonuses
        multiplier += self.skill_trees.get_xp_bonus();
        
        // Achievement bonuses
        multiplier += self.achievements.get_xp_bonus();
        
        // Configuration limits
        multiplier.min(self.config.max_xp_multiplier)
    }

    /// Check if content is unlocked
    pub fn is_content_unlocked(&self, content_id: &str) -> bool {
        self.unlockables.is_unlocked(content_id)
    }

    /// Get all available materials for building
    pub fn get_available_materials(&self) -> Vec<VoxelType> {
        self.unlockables.get_unlocked_materials()
    }

    /// Get available build modes
    pub fn get_available_build_modes(&self) -> Vec<String> {
        self.unlockables.get_unlocked_build_modes()
    }

    /// Get current progression summary for UI
    pub fn get_progression_summary(&self) -> ProgressionSummary {
        ProgressionSummary {
            player_level: self.player_level.level,
            player_xp: self.player_level.experience,
            xp_to_next_level: self.player_level.experience_to_next_level(),
            total_xp: self.player_level.total_experience,
            skill_trees: self.skill_trees.get_summary(),
            recent_unlocks: self.unlockables.get_recent_unlocks(10),
            achievement_progress: self.achievements.get_progress_summary(),
            available_materials: self.get_available_materials().len(),
            available_build_modes: self.get_available_build_modes().len(),
        }
    }
}

/// Player's overall level and experience
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerLevel {
    pub level: u32,
    pub experience: u64,
    pub total_experience: u64,
}

impl PlayerLevel {
    pub fn new() -> Self {
        Self {
            level: 1,
            experience: 0,
            total_experience: 0,
        }
    }

    /// Add experience and check for level up
    pub fn add_experience(&mut self, xp: u64) -> Option<LevelUpInfo> {
        self.experience += xp;
        self.total_experience += xp;

        // Check for level up
        let required_xp = self.experience_required_for_level(self.level + 1);
        if self.experience >= required_xp {
            let old_level = self.level;
            self.level += 1;
            self.experience -= required_xp;

            Some(LevelUpInfo {
                old_level,
                new_level: self.level,
                bonus_xp: xp,
            })
        } else {
            None
        }
    }

    /// Calculate XP required for a specific level
    pub fn experience_required_for_level(&self, level: u32) -> u64 {
        // Exponential scaling: 1000 * level^2.2
        (1000.0 * (level as f64).powf(2.2)) as u64
    }

    /// Calculate XP needed for next level
    pub fn experience_to_next_level(&self) -> u64 {
        let required = self.experience_required_for_level(self.level + 1);
        required.saturating_sub(self.experience)
    }
}

/// Manages all skill trees (Architecture, Engineering, Artistry, Collaboration)
#[derive(Debug, Serialize, Deserialize)]
pub struct SkillTreeManager {
    pub trees: HashMap<SkillTree, SkillTreeProgress>,
}

impl SkillTreeManager {
    pub fn new() -> Self {
        let mut trees = HashMap::new();
        
        // Initialize all skill trees
        for tree in [SkillTree::Architecture, SkillTree::Engineering, SkillTree::Artistry, SkillTree::Collaboration] {
            trees.insert(tree, SkillTreeProgress::new());
        }
        
        Self { trees }
    }

    /// Award XP to appropriate skill trees based on action
    pub fn award_action_xp(&mut self, action: &BuildingAction, base_xp: u64, _player_data: &mut PlayerData) -> RobinResult<Vec<SkillEvent>> {
        let mut events = Vec::new();
        
        // Determine which skills should receive XP
        let skill_allocations = match action {
            BuildingAction::PlaceBlock { voxel_type, .. } => {
                match voxel_type {
                    VoxelType::Stone | VoxelType::Earth => vec![(SkillTree::Architecture, 1.0)],
                    VoxelType::Water => vec![(SkillTree::Engineering, 1.0)],
                    VoxelType::Grass => vec![(SkillTree::Artistry, 1.0)],
                    VoxelType::Sand => vec![(SkillTree::Architecture, 0.8), (SkillTree::Artistry, 0.2)],
                    _ => vec![(SkillTree::Architecture, 1.0)],
                }
            },
            BuildingAction::CompleteStructure { .. } => {
                vec![(SkillTree::Architecture, 0.7), (SkillTree::Engineering, 0.3)]
            },
            BuildingAction::UseTool { .. } => {
                vec![(SkillTree::Engineering, 1.0)]
            },
            BuildingAction::CreateBlueprint { .. } => {
                vec![(SkillTree::Architecture, 0.6), (SkillTree::Artistry, 0.4)]
            },
            BuildingAction::ShareProject { .. } => {
                vec![(SkillTree::Collaboration, 1.0)]
            },
            _ => vec![(SkillTree::Architecture, 1.0)],
        };

        // Award XP to each relevant skill tree
        for (skill_tree, allocation) in skill_allocations {
            let skill_xp = (base_xp as f64 * allocation) as u64;
            
            if let Some(progress) = self.trees.get_mut(&skill_tree) {
                if let Some(skill_event) = progress.add_experience(skill_xp, skill_tree) {
                    events.push(skill_event);
                }
            }
        }

        Ok(events)
    }

    /// Get XP bonus from all skill trees
    pub fn get_xp_bonus(&self) -> f32 {
        self.trees.values()
            .map(|progress| progress.get_xp_bonus())
            .sum()
    }

    /// Get skill tree summaries for UI
    pub fn get_summary(&self) -> HashMap<SkillTree, SkillTreeSummary> {
        self.trees.iter()
            .map(|(tree, progress)| (*tree, progress.get_summary()))
            .collect()
    }

    /// Load from player data
    pub fn load_from_save(&mut self, player_data: &PlayerData) -> RobinResult<()> {
        for (tree, progress) in &mut self.trees {
            let tree_name = format!("{:?}", tree).to_lowercase();
            
            if let Some(level) = player_data.stats.custom_stats.get(&format!("{}_level", tree_name)) {
                progress.level = *level as u32;
            }
            if let Some(xp) = player_data.stats.custom_stats.get(&format!("{}_xp", tree_name)) {
                progress.experience = *xp as u64;
            }
        }
        Ok(())
    }

    /// Save to player data
    pub fn save_to_data(&self, player_data: &mut PlayerData) -> RobinResult<()> {
        for (tree, progress) in &self.trees {
            let tree_name = format!("{:?}", tree).to_lowercase();
            
            player_data.stats.custom_stats.insert(
                format!("{}_level", tree_name),
                progress.level as f64
            );
            player_data.stats.custom_stats.insert(
                format!("{}_xp", tree_name),
                progress.experience as f64
            );
        }
        Ok(())
    }
}

/// Different skill trees players can progress in
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillTree {
    /// Aesthetic materials, decorative blocks, structural design
    Architecture,
    /// Functional blocks, automation tools, mechanical systems
    Engineering,
    /// Creative tools, color palettes, textures, artistic expression
    Artistry,
    /// Social features, team building bonuses, sharing tools
    Collaboration,
}

impl SkillTree {
    pub fn description(&self) -> &'static str {
        match self {
            SkillTree::Architecture => "Unlock aesthetic materials, decorative blocks, and advanced structural design tools",
            SkillTree::Engineering => "Unlock functional blocks, automation systems, and mechanical engineering tools",
            SkillTree::Artistry => "Unlock creative tools, color palettes, textures, and artistic expression features",
            SkillTree::Collaboration => "Unlock social features, team building bonuses, and collaborative creation tools",
        }
    }

    pub fn get_unlocks(&self, level: u32) -> Vec<String> {
        match self {
            SkillTree::Architecture => self.get_architecture_unlocks(level),
            SkillTree::Engineering => self.get_engineering_unlocks(level),
            SkillTree::Artistry => self.get_artistry_unlocks(level),
            SkillTree::Collaboration => self.get_collaboration_unlocks(level),
        }
    }

    fn get_architecture_unlocks(&self, level: u32) -> Vec<String> {
        let mut unlocks = Vec::new();
        
        if level >= 5 { unlocks.push("marble_blocks".to_string()); }
        if level >= 10 { unlocks.push("decorative_pillars".to_string()); }
        if level >= 15 { unlocks.push("stained_glass".to_string()); }
        if level >= 20 { unlocks.push("blueprint_designer".to_string()); }
        if level >= 25 { unlocks.push("architectural_styles".to_string()); }
        if level >= 30 { unlocks.push("monument_templates".to_string()); }
        if level >= 40 { unlocks.push("city_planning_tools".to_string()); }
        if level >= 50 { unlocks.push("legendary_materials".to_string()); }
        
        unlocks
    }

    fn get_engineering_unlocks(&self, level: u32) -> Vec<String> {
        let mut unlocks = Vec::new();
        
        if level >= 5 { unlocks.push("basic_logic_gates".to_string()); }
        if level >= 10 { unlocks.push("conveyor_blocks".to_string()); }
        if level >= 15 { unlocks.push("automated_builders".to_string()); }
        if level >= 20 { unlocks.push("power_systems".to_string()); }
        if level >= 25 { unlocks.push("advanced_automation".to_string()); }
        if level >= 30 { unlocks.push("macro_building".to_string()); }
        if level >= 40 { unlocks.push("intelligent_systems".to_string()); }
        if level >= 50 { unlocks.push("experimental_tech".to_string()); }
        
        unlocks
    }

    fn get_artistry_unlocks(&self, level: u32) -> Vec<String> {
        let mut unlocks = Vec::new();
        
        if level >= 5 { unlocks.push("color_palette_basic".to_string()); }
        if level >= 10 { unlocks.push("texture_brush".to_string()); }
        if level >= 15 { unlocks.push("pattern_tools".to_string()); }
        if level >= 20 { unlocks.push("gradient_effects".to_string()); }
        if level >= 25 { unlocks.push("artistic_materials".to_string()); }
        if level >= 30 { unlocks.push("sculpture_mode".to_string()); }
        if level >= 40 { unlocks.push("dynamic_lighting".to_string()); }
        if level >= 50 { unlocks.push("masterwork_tools".to_string()); }
        
        unlocks
    }

    fn get_collaboration_unlocks(&self, level: u32) -> Vec<String> {
        let mut unlocks = Vec::new();
        
        if level >= 5 { unlocks.push("project_sharing".to_string()); }
        if level >= 10 { unlocks.push("real_time_collaboration".to_string()); }
        if level >= 15 { unlocks.push("team_voice_chat".to_string()); }
        if level >= 20 { unlocks.push("role_permissions".to_string()); }
        if level >= 25 { unlocks.push("project_galleries".to_string()); }
        if level >= 30 { unlocks.push("community_challenges".to_string()); }
        if level >= 40 { unlocks.push("mentorship_system".to_string()); }
        if level >= 50 { unlocks.push("global_showcase".to_string()); }
        
        unlocks
    }
}

/// Progress in a specific skill tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillTreeProgress {
    pub level: u32,
    pub experience: u64,
    pub unlocked_content: HashSet<String>,
}

impl SkillTreeProgress {
    pub fn new() -> Self {
        Self {
            level: 1,
            experience: 0,
            unlocked_content: HashSet::new(),
        }
    }

    /// Add experience and check for level up
    pub fn add_experience(&mut self, xp: u64, skill_tree: SkillTree) -> Option<SkillEvent> {
        self.experience += xp;
        
        let required_xp = self.experience_required_for_level(self.level + 1);
        if self.experience >= required_xp {
            let old_level = self.level;
            self.level += 1;
            self.experience -= required_xp;

            // Check for new unlocks
            let new_unlocks = skill_tree.get_unlocks(self.level);
            let mut newly_unlocked = Vec::new();
            
            for unlock in new_unlocks {
                if self.unlocked_content.insert(unlock.clone()) {
                    newly_unlocked.push(unlock);
                }
            }

            Some(SkillEvent::LevelUp {
                skill_tree,
                old_level,
                new_level: self.level,
                new_unlocks: newly_unlocked,
            })
        } else {
            None
        }
    }

    /// Calculate XP required for a specific level
    fn experience_required_for_level(&self, level: u32) -> u64 {
        // Skill trees level faster than player level
        (500.0 * (level as f64).powf(1.8)) as u64
    }

    /// Get XP bonus from this skill tree
    pub fn get_xp_bonus(&self) -> f32 {
        // 1% XP bonus per 10 levels
        (self.level as f32 / 10.0) * 0.01
    }

    /// Get summary for UI display
    pub fn get_summary(&self) -> SkillTreeSummary {
        let xp_to_next = self.experience_required_for_level(self.level + 1) - self.experience;
        
        SkillTreeSummary {
            level: self.level,
            experience: self.experience,
            experience_to_next_level: xp_to_next,
            unlocked_count: self.unlocked_content.len(),
            xp_bonus: self.get_xp_bonus(),
        }
    }
}

/// Manages unlockable content based on progression
#[derive(Debug, Serialize, Deserialize)]
pub struct UnlockableContentManager {
    pub unlocked_materials: HashSet<VoxelType>,
    pub unlocked_build_modes: HashSet<String>,
    pub unlocked_tools: HashSet<String>,
    pub unlocked_features: HashSet<String>,
    pub recent_unlocks: Vec<UnlockEvent>,
}

impl UnlockableContentManager {
    pub fn new() -> Self {
        let mut manager = Self {
            unlocked_materials: HashSet::new(),
            unlocked_build_modes: HashSet::new(),
            unlocked_tools: HashSet::new(),
            unlocked_features: HashSet::new(),
            recent_unlocks: Vec::new(),
        };

        // Start with basic materials unlocked
        manager.unlocked_materials.insert(VoxelType::Earth);
        manager.unlocked_materials.insert(VoxelType::Stone);
        manager.unlocked_build_modes.insert("basic_building".to_string());
        
        manager
    }

    /// Check for level-based unlocks
    pub fn check_level_unlocks(&mut self, player_level: u32) -> Vec<UnlockEvent> {
        let mut unlocks = Vec::new();

        // Material unlocks based on player level
        let level_unlocks = [
            (3, VoxelType::Grass, "Basic grass blocks for landscaping"),
            (5, VoxelType::Water, "Water blocks for pools and rivers"),
            (8, VoxelType::Sand, "Sand blocks for beaches and deserts"),
        ];

        for (level, material, description) in level_unlocks {
            if player_level >= level && self.unlocked_materials.insert(material) {
                let unlock = UnlockEvent {
                    content_type: UnlockableContentType::Material,
                    content_id: format!("{:?}", material).to_lowercase(),
                    description: description.to_string(),
                    unlock_time: Utc::now(),
                };
                unlocks.push(unlock.clone());
                self.recent_unlocks.push(unlock);
            }
        }

        // Build mode unlocks
        let mode_unlocks = [
            (10, "creative_mode", "Creative mode with unlimited resources"),
            (15, "blueprint_mode", "Blueprint creation and sharing"),
            (25, "collaboration_mode", "Real-time collaborative building"),
            (35, "automation_mode", "Automated building systems"),
        ];

        for (level, mode, description) in mode_unlocks {
            if player_level >= level && self.unlocked_build_modes.insert(mode.to_string()) {
                let unlock = UnlockEvent {
                    content_type: UnlockableContentType::BuildMode,
                    content_id: mode.to_string(),
                    description: description.to_string(),
                    unlock_time: Utc::now(),
                };
                unlocks.push(unlock.clone());
                self.recent_unlocks.push(unlock);
            }
        }

        // Keep only recent unlocks (last 50)
        if self.recent_unlocks.len() > 50 {
            self.recent_unlocks.drain(0..self.recent_unlocks.len() - 50);
        }

        unlocks
    }

    /// Check if specific content is unlocked
    pub fn is_unlocked(&self, content_id: &str) -> bool {
        self.unlocked_features.contains(content_id) ||
        self.unlocked_tools.contains(content_id) ||
        self.unlocked_build_modes.contains(content_id)
    }

    /// Get all unlocked materials
    pub fn get_unlocked_materials(&self) -> Vec<VoxelType> {
        self.unlocked_materials.iter().cloned().collect()
    }

    /// Get all unlocked build modes
    pub fn get_unlocked_build_modes(&self) -> Vec<String> {
        self.unlocked_build_modes.iter().cloned().collect()
    }

    /// Get recent unlocks for UI
    pub fn get_recent_unlocks(&self, count: usize) -> Vec<UnlockEvent> {
        self.recent_unlocks.iter()
            .rev()
            .take(count)
            .cloned()
            .collect()
    }

    /// Load from player data
    pub fn load_from_save(&mut self, _player_data: &PlayerData) -> RobinResult<()> {
        // Implementation would load from player data JSON
        Ok(())
    }

    /// Save to player data
    pub fn save_to_data(&self, _player_data: &mut PlayerData) -> RobinResult<()> {
        // Implementation would save to player data JSON
        Ok(())
    }
}

/// Tracks and manages achievements
#[derive(Debug, Serialize, Deserialize)]
pub struct AchievementTracker {
    pub achievements: HashMap<String, Achievement>,
    pub unlocked_achievements: HashSet<String>,
    pub progress_tracking: HashMap<String, u64>,
}

impl AchievementTracker {
    pub fn new() -> Self {
        let mut tracker = Self {
            achievements: HashMap::new(),
            unlocked_achievements: HashSet::new(),
            progress_tracking: HashMap::new(),
        };
        
        tracker.initialize_achievements();
        tracker
    }

    /// Initialize all achievements
    fn initialize_achievements(&mut self) {
        let achievements = [
            // Building milestones
            Achievement {
                id: "first_block".to_string(),
                name: "First Steps".to_string(),
                description: "Place your first block".to_string(),
                category: AchievementCategory::Building,
                condition: AchievementCondition::BlocksPlaced(1),
                reward: AchievementReward::Experience(100),
                hidden: false,
            },
            Achievement {
                id: "hundred_blocks".to_string(),
                name: "Getting Started".to_string(),
                description: "Place 100 blocks".to_string(),
                category: AchievementCategory::Building,
                condition: AchievementCondition::BlocksPlaced(100),
                reward: AchievementReward::Experience(500),
                hidden: false,
            },
            Achievement {
                id: "thousand_blocks".to_string(),
                name: "Dedicated Builder".to_string(),
                description: "Place 1,000 blocks".to_string(),
                category: AchievementCategory::Building,
                condition: AchievementCondition::BlocksPlaced(1000),
                reward: AchievementReward::MaterialUnlock(VoxelType::Sand),
                hidden: false,
            },
            Achievement {
                id: "first_tower".to_string(),
                name: "Reaching for the Sky".to_string(),
                description: "Build a structure 20 blocks tall".to_string(),
                category: AchievementCategory::Building,
                condition: AchievementCondition::StructureHeight(20),
                reward: AchievementReward::Experience(300),
                hidden: false,
            },
            
            // Creative achievements
            Achievement {
                id: "material_master".to_string(),
                name: "Material Master".to_string(),
                description: "Use all available material types in a single build".to_string(),
                category: AchievementCategory::Creative,
                condition: AchievementCondition::UseAllMaterials,
                reward: AchievementReward::BuildModeUnlock("advanced_palette".to_string()),
                hidden: false,
            },
            Achievement {
                id: "complex_structure".to_string(),
                name: "Architectural Wonder".to_string(),
                description: "Complete a structure with over 5,000 blocks".to_string(),
                category: AchievementCategory::Creative,
                condition: AchievementCondition::LargeStructure(5000),
                reward: AchievementReward::Experience(2000),
                hidden: false,
            },
            
            // Social achievements
            Achievement {
                id: "first_share".to_string(),
                name: "Show and Tell".to_string(),
                description: "Share your first project".to_string(),
                category: AchievementCategory::Social,
                condition: AchievementCondition::ProjectsShared(1),
                reward: AchievementReward::Experience(200),
                hidden: false,
            },
            Achievement {
                id: "collaborative_build".to_string(),
                name: "Teamwork".to_string(),
                description: "Complete a project with another player".to_string(),
                category: AchievementCategory::Social,
                condition: AchievementCondition::CollaborativeProjects(1),
                reward: AchievementReward::MultiplierBonus(0.1),
                hidden: false,
            },
            
            // Hidden achievements
            Achievement {
                id: "secret_rainbow".to_string(),
                name: "Rainbow Bridge".to_string(),
                description: "Build a structure using blocks in rainbow order".to_string(),
                category: AchievementCategory::Creative,
                condition: AchievementCondition::RainbowPattern,
                reward: AchievementReward::MaterialUnlock(VoxelType::Grass),
                hidden: true,
            },
        ];

        for achievement in achievements {
            self.achievements.insert(achievement.id.clone(), achievement);
        }
    }

    /// Check building action for achievement progress
    pub fn check_building_action(&mut self, action: &BuildingAction, player_data: &PlayerData) -> RobinResult<Vec<AchievementEvent>> {
        let mut events = Vec::new();

        // Update progress tracking
        match action {
            BuildingAction::PlaceBlock { .. } => {
                *self.progress_tracking.entry("blocks_placed".to_string()).or_insert(0) += 1;
            },
            BuildingAction::CompleteStructure { size, .. } => {
                let current_max = *self.progress_tracking.entry("max_structure_size".to_string()).or_insert(0);
                if *size as u64 > current_max {
                    self.progress_tracking.insert("max_structure_size".to_string(), *size as u64);
                }
            },
            BuildingAction::ShareProject { .. } => {
                *self.progress_tracking.entry("projects_shared".to_string()).or_insert(0) += 1;
            },
            _ => {},
        }

        // Check for achievement unlocks
        for (id, achievement) in &self.achievements {
            if self.unlocked_achievements.contains(id) {
                continue; // Already unlocked
            }

            let progress_met = match &achievement.condition {
                AchievementCondition::BlocksPlaced(target) => {
                    self.progress_tracking.get("blocks_placed").unwrap_or(&0) >= target
                },
                AchievementCondition::StructureHeight(target) => {
                    // Would need structure height tracking
                    false
                },
                AchievementCondition::LargeStructure(target) => {
                    self.progress_tracking.get("max_structure_size").unwrap_or(&0) >= target
                },
                AchievementCondition::ProjectsShared(target) => {
                    self.progress_tracking.get("projects_shared").unwrap_or(&0) >= target
                },
                AchievementCondition::CollaborativeProjects(target) => {
                    // Would need collaboration tracking
                    self.progress_tracking.get("collaborative_projects").unwrap_or(&0) >= target
                },
                AchievementCondition::UseAllMaterials => {
                    // Would need material usage tracking
                    false
                },
                AchievementCondition::RainbowPattern => {
                    // Would need pattern detection
                    false
                },
            };

            if progress_met {
                self.unlocked_achievements.insert(id.clone());
                events.push(AchievementEvent {
                    achievement: achievement.clone(),
                    unlock_time: Utc::now(),
                });
            }
        }

        Ok(events)
    }

    /// Get XP bonus from achievements
    pub fn get_xp_bonus(&self) -> f32 {
        let mut bonus = 0.0;
        
        for achievement_id in &self.unlocked_achievements {
            if let Some(achievement) = self.achievements.get(achievement_id) {
                if let AchievementReward::MultiplierBonus(multiplier) = achievement.reward {
                    bonus += multiplier;
                }
            }
        }
        
        bonus
    }

    /// Get achievement progress summary
    pub fn get_progress_summary(&self) -> AchievementProgressSummary {
        let total_achievements = self.achievements.len();
        let unlocked_count = self.unlocked_achievements.len();
        let hidden_unlocked = self.achievements.iter()
            .filter(|(id, achievement)| achievement.hidden && self.unlocked_achievements.contains(*id))
            .count();

        AchievementProgressSummary {
            total_achievements,
            unlocked_count,
            hidden_unlocked,
            completion_percentage: (unlocked_count as f32 / total_achievements as f32) * 100.0,
        }
    }

    /// Load from player data
    pub fn load_from_save(&mut self, _player_data: &PlayerData) -> RobinResult<()> {
        // Implementation would load from player data
        Ok(())
    }

    /// Save to player data
    pub fn save_to_data(&self, _player_data: &mut PlayerData) -> RobinResult<()> {
        // Implementation would save to player data
        Ok(())
    }
}

/// Configuration for progression rates and bonuses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressionConfig {
    pub base_building_xp: f32,
    pub base_mining_xp: f32,
    pub structure_completion_xp: f32,
    pub tool_usage_xp: f32,
    pub blueprint_creation_xp: f32,
    pub collaboration_xp: f32,
    pub max_xp_multiplier: f32,
    pub block_xp_multipliers: HashMap<VoxelType, f32>,
}

impl Default for ProgressionConfig {
    fn default() -> Self {
        let mut config = Self {
            base_building_xp: 10.0,
            base_mining_xp: 5.0,
            structure_completion_xp: 100.0,
            tool_usage_xp: 15.0,
            blueprint_creation_xp: 50.0,
            collaboration_xp: 25.0,
            max_xp_multiplier: 3.0,
            block_xp_multipliers: HashMap::new(),
        };

        // Different blocks give different XP amounts
        config.block_xp_multipliers.insert(VoxelType::Earth, 1.0);
        config.block_xp_multipliers.insert(VoxelType::Stone, 1.2);
        config.block_xp_multipliers.insert(VoxelType::Water, 1.5);
        config.block_xp_multipliers.insert(VoxelType::Grass, 1.1);
        config.block_xp_multipliers.insert(VoxelType::Sand, 1.0);

        config
    }
}

impl ProgressionConfig {
    pub fn get_block_placement_xp(&self, voxel_type: VoxelType) -> f32 {
        let multiplier = self.block_xp_multipliers.get(&voxel_type).copied().unwrap_or(1.0);
        self.base_building_xp * multiplier
    }
}

// Event types for progression system

#[derive(Debug, Clone)]
pub enum ProgressionEvent {
    PlayerLevelUp(LevelUpInfo),
    SkillProgress(SkillEvent),
    ContentUnlocked(UnlockEvent),
    Achievement(AchievementEvent),
}

#[derive(Debug, Clone)]
pub struct LevelUpInfo {
    pub old_level: u32,
    pub new_level: u32,
    pub bonus_xp: u64,
}

#[derive(Debug, Clone)]
pub enum SkillEvent {
    LevelUp {
        skill_tree: SkillTree,
        old_level: u32,
        new_level: u32,
        new_unlocks: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnlockEvent {
    pub content_type: UnlockableContentType,
    pub content_id: String,
    pub description: String,
    pub unlock_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnlockableContentType {
    Material,
    BuildMode,
    Tool,
    Feature,
}

#[derive(Debug, Clone)]
pub struct AchievementEvent {
    pub achievement: Achievement,
    pub unlock_time: DateTime<Utc>,
}

// Building actions that can trigger progression

#[derive(Debug, Clone)]
pub enum BuildingAction {
    PlaceBlock {
        voxel_type: VoxelType,
        position: Vec3,
    },
    RemoveBlock {
        voxel_type: VoxelType,
        position: Vec3,
    },
    CompleteStructure {
        structure_type: String,
        size: u32,
        position: Vec3,
    },
    UseTool {
        tool_name: String,
        tool_tier: u32,
    },
    CreateBlueprint {
        blueprint_name: String,
        complexity: f32,
    },
    ShareProject {
        project_name: String,
    },
}

// Achievement system types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: AchievementCategory,
    pub condition: AchievementCondition,
    pub reward: AchievementReward,
    pub hidden: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AchievementCategory {
    Building,
    Creative,
    Social,
    Exploration,
    Technical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AchievementCondition {
    BlocksPlaced(u64),
    StructureHeight(u32),
    LargeStructure(u64),
    ProjectsShared(u64),
    CollaborativeProjects(u64),
    UseAllMaterials,
    RainbowPattern,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AchievementReward {
    Experience(u64),
    MaterialUnlock(VoxelType),
    BuildModeUnlock(String),
    MultiplierBonus(f32),
    CosmeticUnlock(String),
}

// Summary types for UI display

#[derive(Debug, Clone)]
pub struct ProgressionSummary {
    pub player_level: u32,
    pub player_xp: u64,
    pub xp_to_next_level: u64,
    pub total_xp: u64,
    pub skill_trees: HashMap<SkillTree, SkillTreeSummary>,
    pub recent_unlocks: Vec<UnlockEvent>,
    pub achievement_progress: AchievementProgressSummary,
    pub available_materials: usize,
    pub available_build_modes: usize,
}

#[derive(Debug, Clone)]
pub struct SkillTreeSummary {
    pub level: u32,
    pub experience: u64,
    pub experience_to_next_level: u64,
    pub unlocked_count: usize,
    pub xp_bonus: f32,
}

#[derive(Debug, Clone)]
pub struct AchievementProgressSummary {
    pub total_achievements: usize,
    pub unlocked_count: usize,
    pub hidden_unlocked: usize,
    pub completion_percentage: f32,
}

impl Default for ProgressionSystem {
    fn default() -> Self {
        Self::new()
    }
}