/*!
 * Achievement System for Robin Engine
 *
 * Recognizes player accomplishments, milestones, and special feats
 * in the voxel building and engineering gameplay experience.
 */

use crate::engine::{
    error::{RobinError, RobinResult},
    save_system::PlayerData,
    gameplay::{BuildingSkill, SessionStats},
};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::SystemTime;

/// Core achievement management system
pub struct AchievementManager {
    /// All available achievements
    achievements: HashMap<String, Achievement>,
    /// Achievement categories for organization
    categories: HashMap<String, Vec<String>>,
    /// Recently unlocked achievements (for UI notifications)
    recent_unlocks: Vec<(String, SystemTime)>,
}

impl AchievementManager {
    pub fn new() -> Self {
        let mut manager = Self {
            achievements: HashMap::new(),
            categories: HashMap::new(),
            recent_unlocks: Vec::new(),
        };

        manager.initialize_achievements();
        manager
    }

    /// Initialize all achievement definitions
    fn initialize_achievements(&mut self) {
        // Building Achievements
        self.add_achievement("first_block", Achievement {
            name: "First Steps".to_string(),
            description: "Place your very first voxel block".to_string(),
            icon: "🧱".to_string(),
            category: "building".to_string(),
            rarity: AchievementRarity::Common,
            conditions: vec![AchievementCondition::BlocksPlaced(1)],
            rewards: vec![AchievementReward::Experience(BuildingSkill::Construction, 50)],
            hidden: false,
            prerequisite: None,
        });

        self.add_achievement("hundred_builder", Achievement {
            name: "Hundred Builder".to_string(),
            description: "Place 100 voxel blocks".to_string(),
            icon: "🏗️".to_string(),
            category: "building".to_string(),
            rarity: AchievementRarity::Common,
            conditions: vec![AchievementCondition::BlocksPlaced(100)],
            rewards: vec![
                AchievementReward::Experience(BuildingSkill::Construction, 200),
                AchievementReward::Title("Builder".to_string()),
            ],
            hidden: false,
            prerequisite: Some("first_block".to_string()),
        });

        self.add_achievement("thousand_architect", Achievement {
            name: "Thousand Architect".to_string(),
            description: "Place 1,000 voxel blocks with style".to_string(),
            icon: "🏛️".to_string(),
            category: "building".to_string(),
            rarity: AchievementRarity::Uncommon,
            conditions: vec![AchievementCondition::BlocksPlaced(1000)],
            rewards: vec![
                AchievementReward::Experience(BuildingSkill::Construction, 500),
                AchievementReward::Experience(BuildingSkill::Architecture, 300),
                AchievementReward::Title("Architect".to_string()),
            ],
            hidden: false,
            prerequisite: Some("hundred_builder".to_string()),
        });

        self.add_achievement("master_builder", Achievement {
            name: "Master Builder".to_string(),
            description: "Place 10,000 voxel blocks and become a true master".to_string(),
            icon: "👑".to_string(),
            category: "building".to_string(),
            rarity: AchievementRarity::Rare,
            conditions: vec![AchievementCondition::BlocksPlaced(10000)],
            rewards: vec![
                AchievementReward::Experience(BuildingSkill::Construction, 1000),
                AchievementReward::Experience(BuildingSkill::Architecture, 800),
                AchievementReward::Title("Master Builder".to_string()),
                AchievementReward::UnlockContent("master_templates".to_string()),
            ],
            hidden: false,
            prerequisite: Some("thousand_architect".to_string()),
        });

        // Mining Achievements
        self.add_achievement("first_dig", Achievement {
            name: "First Dig".to_string(),
            description: "Mine your first voxel block".to_string(),
            icon: "⛏️".to_string(),
            category: "mining".to_string(),
            rarity: AchievementRarity::Common,
            conditions: vec![AchievementCondition::BlocksMined(1)],
            rewards: vec![AchievementReward::Experience(BuildingSkill::Mining, 25)],
            hidden: false,
            prerequisite: None,
        });

        self.add_achievement("cave_explorer", Achievement {
            name: "Cave Explorer".to_string(),
            description: "Mine 500 blocks underground".to_string(),
            icon: "🕳️".to_string(),
            category: "mining".to_string(),
            rarity: AchievementRarity::Uncommon,
            conditions: vec![AchievementCondition::BlocksMined(500)],
            rewards: vec![
                AchievementReward::Experience(BuildingSkill::Mining, 400),
                AchievementReward::Items(vec![("tool_advanced_pickaxe".to_string(), 1)]),
            ],
            hidden: false,
            prerequisite: Some("first_dig".to_string()),
        });

        // Crafting Achievements
        self.add_achievement("first_craft", Achievement {
            name: "Craftsperson".to_string(),
            description: "Craft your first item".to_string(),
            icon: "🔨".to_string(),
            category: "crafting".to_string(),
            rarity: AchievementRarity::Common,
            conditions: vec![AchievementCondition::ItemsCrafted(1)],
            rewards: vec![AchievementReward::Experience(BuildingSkill::Crafting, 75)],
            hidden: false,
            prerequisite: None,
        });

        self.add_achievement("production_line", Achievement {
            name: "Production Line".to_string(),
            description: "Craft 100 items total".to_string(),
            icon: "🏭".to_string(),
            category: "crafting".to_string(),
            rarity: AchievementRarity::Uncommon,
            conditions: vec![AchievementCondition::ItemsCrafted(100)],
            rewards: vec![
                AchievementReward::Experience(BuildingSkill::Crafting, 600),
                AchievementReward::UnlockRecipe("automated_crafter".to_string()),
            ],
            hidden: false,
            prerequisite: Some("first_craft".to_string()),
        });

        // Skill-based Achievements
        self.add_achievement("skilled_constructor", Achievement {
            name: "Skilled Constructor".to_string(),
            description: "Reach level 25 in Construction skill".to_string(),
            icon: "📈".to_string(),
            category: "skills".to_string(),
            rarity: AchievementRarity::Uncommon,
            conditions: vec![AchievementCondition::SkillLevel(BuildingSkill::Construction, 25)],
            rewards: vec![
                AchievementReward::Experience(BuildingSkill::Construction, 500),
                AchievementReward::Title("Skilled Constructor".to_string()),
            ],
            hidden: false,
            prerequisite: None,
        });

        self.add_achievement("engineering_genius", Achievement {
            name: "Engineering Genius".to_string(),
            description: "Reach level 50 in Engineering skill".to_string(),
            icon: "🧠".to_string(),
            category: "skills".to_string(),
            rarity: AchievementRarity::Rare,
            conditions: vec![AchievementCondition::SkillLevel(BuildingSkill::Engineering, 50)],
            rewards: vec![
                AchievementReward::Experience(BuildingSkill::Engineering, 2000),
                AchievementReward::Title("Engineering Genius".to_string()),
                AchievementReward::UnlockContent("genius_blueprints".to_string()),
            ],
            hidden: false,
            prerequisite: None,
        });

        // Special Achievements
        self.add_achievement("speed_builder", Achievement {
            name: "Speed Builder".to_string(),
            description: "Place 50 blocks in under 60 seconds".to_string(),
            icon: "⚡".to_string(),
            category: "special".to_string(),
            rarity: AchievementRarity::Uncommon,
            conditions: vec![AchievementCondition::SpeedBuilding(50, 60.0)],
            rewards: vec![
                AchievementReward::Experience(BuildingSkill::Construction, 300),
                AchievementReward::Title("Speed Builder".to_string()),
            ],
            hidden: false,
            prerequisite: None,
        });

        self.add_achievement("marathon_session", Achievement {
            name: "Marathon Session".to_string(),
            description: "Play for 4 continuous hours".to_string(),
            icon: "🏃".to_string(),
            category: "special".to_string(),
            rarity: AchievementRarity::Uncommon,
            conditions: vec![AchievementCondition::PlayTime(14400.0)], // 4 hours
            rewards: vec![
                AchievementReward::Items(vec![
                    ("resource_stone".to_string(), 100),
                    ("resource_metal".to_string(), 50),
                ]),
                AchievementReward::Title("Marathon Builder".to_string()),
            ],
            hidden: false,
            prerequisite: None,
        });

        // Hidden/Secret Achievements
        self.add_achievement("perfectionist", Achievement {
            name: "Perfectionist".to_string(),
            description: "Complete 10 objectives without failing any".to_string(),
            icon: "💎".to_string(),
            category: "secret".to_string(),
            rarity: AchievementRarity::Epic,
            conditions: vec![
                AchievementCondition::ObjectivesCompleted(10),
                AchievementCondition::ObjectivesFailed(0),
            ],
            rewards: vec![
                AchievementReward::Experience(BuildingSkill::Construction, 1000),
                AchievementReward::Title("Perfectionist".to_string()),
                AchievementReward::UnlockContent("perfectionist_tools".to_string()),
            ],
            hidden: true,
            prerequisite: None,
        });

        self.add_achievement("legendary_engineer", Achievement {
            name: "Legendary Engineer".to_string(),
            description: "Achieve mastery in all building skills".to_string(),
            icon: "🌟".to_string(),
            category: "legendary".to_string(),
            rarity: AchievementRarity::Legendary,
            conditions: vec![
                AchievementCondition::SkillLevel(BuildingSkill::Construction, 75),
                AchievementCondition::SkillLevel(BuildingSkill::Mining, 75),
                AchievementCondition::SkillLevel(BuildingSkill::Crafting, 75),
                AchievementCondition::SkillLevel(BuildingSkill::Engineering, 75),
                AchievementCondition::SkillLevel(BuildingSkill::Architecture, 75),
                AchievementCondition::SkillLevel(BuildingSkill::ResourceManagement, 75),
            ],
            rewards: vec![
                AchievementReward::Title("Legendary Engineer".to_string()),
                AchievementReward::UnlockContent("legendary_abilities".to_string()),
                AchievementReward::Items(vec![("trophy_legendary_engineer".to_string(), 1)]),
            ],
            hidden: true,
            prerequisite: None,
        });

        // Initialize categories
        self.categories.insert("building".to_string(), vec![
            "first_block".to_string(),
            "hundred_builder".to_string(),
            "thousand_architect".to_string(),
            "master_builder".to_string(),
        ]);

        self.categories.insert("mining".to_string(), vec![
            "first_dig".to_string(),
            "cave_explorer".to_string(),
        ]);

        self.categories.insert("crafting".to_string(), vec![
            "first_craft".to_string(),
            "production_line".to_string(),
        ]);

        self.categories.insert("skills".to_string(), vec![
            "skilled_constructor".to_string(),
            "engineering_genius".to_string(),
        ]);

        self.categories.insert("special".to_string(), vec![
            "speed_builder".to_string(),
            "marathon_session".to_string(),
        ]);

        self.categories.insert("secret".to_string(), vec![
            "perfectionist".to_string(),
        ]);

        self.categories.insert("legendary".to_string(), vec![
            "legendary_engineer".to_string(),
        ]);
    }

    /// Add an achievement to the system
    fn add_achievement(&mut self, id: &str, achievement: Achievement) {
        self.achievements.insert(id.to_string(), achievement);
    }

    /// Check for achievement unlocks based on current player state
    pub fn check_achievements(&mut self, player_data: &PlayerData, session_stats: &SessionStats) -> RobinResult<Vec<String>> {
        let mut newly_unlocked = Vec::new();

        for (achievement_id, achievement) in &self.achievements {
            // Skip if already unlocked
            if player_data.achievements.contains(achievement_id) {
                continue;
            }

            // Check prerequisite
            if let Some(prerequisite) = &achievement.prerequisite {
                if !player_data.achievements.contains(prerequisite) {
                    continue;
                }
            }

            // Check all conditions
            if achievement.conditions.iter().all(|condition| self.check_condition(condition, player_data, session_stats)) {
                newly_unlocked.push(achievement_id.clone());
            }
        }

        Ok(newly_unlocked)
    }

    /// Check if a specific achievement condition is met
    fn check_condition(&self, condition: &AchievementCondition, player_data: &PlayerData, session_stats: &SessionStats) -> bool {
        match condition {
            AchievementCondition::BlocksPlaced(target) => {
                player_data.stats.custom_stats.get("blocks_placed").unwrap_or(&0.0) >= &(*target as f64)
            }
            AchievementCondition::BlocksMined(target) => {
                player_data.stats.custom_stats.get("blocks_mined").unwrap_or(&0.0) >= &(*target as f64)
            }
            AchievementCondition::ItemsCrafted(target) => {
                player_data.stats.custom_stats.get("items_crafted").unwrap_or(&0.0) >= &(*target as f64)
            }
            AchievementCondition::SkillLevel(skill, target_level) => {
                let skill_key = format!("{}_level", skill.to_string().to_lowercase());
                player_data.stats.custom_stats.get(&skill_key).unwrap_or(&0.0) >= &(*target_level as f64)
            }
            AchievementCondition::PlayTime(target_seconds) => {
                session_stats.total_play_time >= *target_seconds ||
                player_data.stats.time_played >= (*target_seconds as u64)
            }
            AchievementCondition::ObjectivesCompleted(target) => {
                session_stats.objectives_completed >= *target ||
                player_data.stats.custom_stats.get("objectives_completed_total").unwrap_or(&0.0) >= &(*target as f64)
            }
            AchievementCondition::ObjectivesFailed(max_failures) => {
                let failures = player_data.stats.custom_stats.get("objectives_failed_total").unwrap_or(&0.0);
                failures <= &(*max_failures as f64)
            }
            AchievementCondition::SpeedBuilding(blocks, time_limit) => {
                // This would require session tracking of building speed
                let recent_build_rate = session_stats.blocks_placed as f32 / session_stats.total_play_time.max(1.0);
                recent_build_rate >= (*blocks as f32) / time_limit
            }
            AchievementCondition::ResourcesCollected(resource_type, target) => {
                player_data.get_item_count(resource_type) >= *target
            }
            AchievementCondition::StructuresBuilt(target) => {
                player_data.stats.custom_stats.get("structures_built").unwrap_or(&0.0) >= &(*target as f64)
            }
            AchievementCondition::ConsecutiveDays(target) => {
                // This would require tracking login streaks
                player_data.stats.custom_stats.get("login_streak").unwrap_or(&0.0) >= &(*target as f64)
            }
        }
    }

    /// Award achievement to player
    pub fn unlock_achievement(&mut self, achievement_id: &str, player_data: &mut PlayerData) -> RobinResult<()> {
        if let Some(achievement) = self.achievements.get(achievement_id) {
            if !player_data.achievements.contains(&achievement_id.to_string()) {
                // Unlock the achievement
                player_data.unlock_achievement(achievement_id);

                // Award rewards
                for reward in &achievement.rewards {
                    match reward {
                        AchievementReward::Experience(skill, amount) => {
                            let stat_key = format!("{}_experience_bonus", skill.to_string().to_lowercase());
                            player_data.stats.custom_stats.entry(stat_key).and_modify(|v| *v += *amount as f64).or_insert(*amount as f64);
                        }
                        AchievementReward::Items(items) => {
                            for (item_id, quantity) in items {
                                player_data.add_item(item_id, *quantity);
                            }
                        }
                        AchievementReward::Title(title) => {
                            let titles_key = "unlocked_titles".to_string();
                            let current_titles = player_data.stats.custom_stats.get(&titles_key).unwrap_or(&0.0);
                            // In a full implementation, this would manage a titles collection
                            player_data.stats.custom_stats.insert(format!("title_{}", title.replace(' ', "_").to_lowercase()), 1.0);
                        }
                        AchievementReward::UnlockContent(content_id) => {
                            let unlock_key = format!("content_unlocked_{}", content_id);
                            player_data.stats.custom_stats.insert(unlock_key, 1.0);
                        }
                        AchievementReward::UnlockRecipe(recipe_id) => {
                            let unlock_key = format!("recipe_unlocked_{}", recipe_id);
                            player_data.stats.custom_stats.insert(unlock_key, 1.0);
                        }
                    }
                }

                // Track recent unlock for UI notifications
                self.recent_unlocks.push((achievement_id.to_string(), SystemTime::now()));

                // Keep only recent unlocks (last 10)
                if self.recent_unlocks.len() > 10 {
                    self.recent_unlocks.drain(0..self.recent_unlocks.len() - 10);
                }
            }

            Ok(())
        } else {
            Err(RobinError::InvalidInput(format!("Unknown achievement: {}", achievement_id)))
        }
    }

    /// Get all achievements in a category
    pub fn get_achievements_by_category(&self, category: &str, include_hidden: bool) -> Vec<(&String, &Achievement)> {
        if let Some(achievement_ids) = self.categories.get(category) {
            achievement_ids.iter()
                .filter_map(|id| self.achievements.get(id).map(|achievement| (id, achievement)))
                .filter(|(_, achievement)| include_hidden || !achievement.hidden)
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get all unlocked achievements for a player
    pub fn get_unlocked_achievements(&self, player_data: &PlayerData) -> Vec<(&String, &Achievement)> {
        self.achievements.iter()
            .filter(|(id, _)| player_data.achievements.contains(*id))
            .collect()
    }

    /// Get recent achievement unlocks
    pub fn get_recent_unlocks(&self) -> &Vec<(String, SystemTime)> {
        &self.recent_unlocks
    }

    /// Get achievement progress for display
    pub fn get_achievement_progress(&self, achievement_id: &str, player_data: &PlayerData, session_stats: &SessionStats) -> Option<AchievementProgress> {
        if let Some(achievement) = self.achievements.get(achievement_id) {
            let is_unlocked = player_data.achievements.contains(&achievement_id.to_string());

            // Calculate progress for the first condition (assuming single condition for progress display)
            let progress = if let Some(condition) = achievement.conditions.first() {
                self.get_condition_progress(condition, player_data, session_stats)
            } else {
                (0, 1)
            };

            Some(AchievementProgress {
                achievement_id: achievement_id.to_string(),
                name: achievement.name.clone(),
                description: achievement.description.clone(),
                icon: achievement.icon.clone(),
                rarity: achievement.rarity,
                is_unlocked,
                current_progress: progress.0,
                target_progress: progress.1,
                progress_percentage: (progress.0 as f32 / progress.1 as f32 * 100.0).min(100.0),
            })
        } else {
            None
        }
    }

    /// Get progress for a specific condition
    fn get_condition_progress(&self, condition: &AchievementCondition, player_data: &PlayerData, session_stats: &SessionStats) -> (u32, u32) {
        match condition {
            AchievementCondition::BlocksPlaced(target) => {
                let current = *player_data.stats.custom_stats.get("blocks_placed").unwrap_or(&0.0) as u32;
                (current, *target)
            }
            AchievementCondition::BlocksMined(target) => {
                let current = *player_data.stats.custom_stats.get("blocks_mined").unwrap_or(&0.0) as u32;
                (current, *target)
            }
            AchievementCondition::ItemsCrafted(target) => {
                let current = *player_data.stats.custom_stats.get("items_crafted").unwrap_or(&0.0) as u32;
                (current, *target)
            }
            AchievementCondition::SkillLevel(skill, target_level) => {
                let skill_key = format!("{}_level", skill.to_string().to_lowercase());
                let current = *player_data.stats.custom_stats.get(&skill_key).unwrap_or(&0.0) as u32;
                (current, *target_level)
            }
            _ => (0, 1), // Default for conditions that don't have clear progress
        }
    }

    /// Get all category names
    pub fn get_categories(&self) -> Vec<&String> {
        self.categories.keys().collect()
    }
}

/// Achievement definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub name: String,
    pub description: String,
    pub icon: String,
    pub category: String,
    pub rarity: AchievementRarity,
    pub conditions: Vec<AchievementCondition>,
    pub rewards: Vec<AchievementReward>,
    pub hidden: bool,
    pub prerequisite: Option<String>,
}

/// Rarity levels for achievements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AchievementRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

/// Conditions that must be met to unlock achievements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AchievementCondition {
    BlocksPlaced(u32),
    BlocksMined(u32),
    ItemsCrafted(u32),
    SkillLevel(BuildingSkill, u32),
    PlayTime(f32), // seconds
    ObjectivesCompleted(u32),
    ObjectivesFailed(u32),
    SpeedBuilding(u32, f32), // blocks in time_limit seconds
    ResourcesCollected(String, u32),
    StructuresBuilt(u32),
    ConsecutiveDays(u32),
}

/// Rewards granted for unlocking achievements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AchievementReward {
    Experience(BuildingSkill, u32),
    Items(Vec<(String, u32)>),
    Title(String),
    UnlockContent(String),
    UnlockRecipe(String),
}

/// Achievement progress information for UI display
#[derive(Debug, Clone)]
pub struct AchievementProgress {
    pub achievement_id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub rarity: AchievementRarity,
    pub is_unlocked: bool,
    pub current_progress: u32,
    pub target_progress: u32,
    pub progress_percentage: f32,
}

impl Default for AchievementManager {
    fn default() -> Self {
        Self::new()
    }
}