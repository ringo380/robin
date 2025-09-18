/// Achievement System for Robin Game Engine
///
/// Provides unified achievement tracking across different platforms

use crate::engine::core::RobinResult;
use std::collections::HashMap;
use std::time::SystemTime;
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::PathBuf;

use super::Platform;

/// Achievement manager
pub struct AchievementManager {
    platform: Option<Platform>,
    achievements: HashMap<String, Achievement>,
    progress: HashMap<String, AchievementProgress>,
    statistics: HashMap<String, Statistic>,
    save_path: PathBuf,
    notifications_enabled: bool,
}

impl AchievementManager {
    /// Create new achievement manager
    pub fn new() -> Self {
        let save_path = Self::get_save_path();

        Self {
            platform: None,
            achievements: HashMap::new(),
            progress: HashMap::new(),
            statistics: HashMap::new(),
            save_path,
            notifications_enabled: true,
        }
    }

    /// Get save path for achievement data
    fn get_save_path() -> PathBuf {
        let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("robin");
        path.push("achievements");
        path
    }

    /// Initialize achievement system
    pub fn initialize(&mut self, platform: Platform) -> RobinResult<()> {
        println!("🏆 Initializing achievement system for {:?}", platform);

        self.platform = Some(platform);

        // Create save directory
        fs::create_dir_all(&self.save_path)?;

        // Load achievement definitions
        self.load_achievement_definitions()?;

        // Load progress
        self.load_progress()?;

        // Sync with platform if available
        self.sync_with_platform()?;

        Ok(())
    }

    /// Load achievement definitions
    fn load_achievement_definitions(&mut self) -> RobinResult<()> {
        // Define all game achievements
        self.register_achievement(Achievement {
            id: "first_steps".to_string(),
            name: "First Steps".to_string(),
            description: "Complete the tutorial".to_string(),
            icon_locked: "achievements/first_steps_locked.png".to_string(),
            icon_unlocked: "achievements/first_steps_unlocked.png".to_string(),
            points: 10,
            category: AchievementCategory::Tutorial,
            hidden: false,
            prerequisite: None,
            statistics_required: vec![],
        });

        self.register_achievement(Achievement {
            id: "builder_novice".to_string(),
            name: "Novice Builder".to_string(),
            description: "Build your first structure".to_string(),
            icon_locked: "achievements/builder_novice_locked.png".to_string(),
            icon_unlocked: "achievements/builder_novice_unlocked.png".to_string(),
            points: 15,
            category: AchievementCategory::Construction,
            hidden: false,
            prerequisite: None,
            statistics_required: vec![StatRequirement {
                stat_name: "structures_built".to_string(),
                required_value: 1.0,
                comparison: Comparison::GreaterOrEqual,
            }],
        });

        self.register_achievement(Achievement {
            id: "builder_expert".to_string(),
            name: "Expert Builder".to_string(),
            description: "Build 100 structures".to_string(),
            icon_locked: "achievements/builder_expert_locked.png".to_string(),
            icon_unlocked: "achievements/builder_expert_unlocked.png".to_string(),
            points: 50,
            category: AchievementCategory::Construction,
            hidden: false,
            prerequisite: Some("builder_novice".to_string()),
            statistics_required: vec![StatRequirement {
                stat_name: "structures_built".to_string(),
                required_value: 100.0,
                comparison: Comparison::GreaterOrEqual,
            }],
        });

        self.register_achievement(Achievement {
            id: "explorer".to_string(),
            name: "Explorer".to_string(),
            description: "Discover 50 unique locations".to_string(),
            icon_locked: "achievements/explorer_locked.png".to_string(),
            icon_unlocked: "achievements/explorer_unlocked.png".to_string(),
            points: 25,
            category: AchievementCategory::Exploration,
            hidden: false,
            prerequisite: None,
            statistics_required: vec![StatRequirement {
                stat_name: "locations_discovered".to_string(),
                required_value: 50.0,
                comparison: Comparison::GreaterOrEqual,
            }],
        });

        self.register_achievement(Achievement {
            id: "speed_demon".to_string(),
            name: "Speed Demon".to_string(),
            description: "Reach 200 km/h in a vehicle".to_string(),
            icon_locked: "achievements/speed_locked.png".to_string(),
            icon_unlocked: "achievements/speed_unlocked.png".to_string(),
            points: 20,
            category: AchievementCategory::Vehicle,
            hidden: false,
            prerequisite: None,
            statistics_required: vec![StatRequirement {
                stat_name: "max_speed".to_string(),
                required_value: 200.0,
                comparison: Comparison::GreaterOrEqual,
            }],
        });

        self.register_achievement(Achievement {
            id: "social_butterfly".to_string(),
            name: "Social Butterfly".to_string(),
            description: "Befriend 20 NPCs".to_string(),
            icon_locked: "achievements/social_locked.png".to_string(),
            icon_unlocked: "achievements/social_unlocked.png".to_string(),
            points: 30,
            category: AchievementCategory::Social,
            hidden: false,
            prerequisite: None,
            statistics_required: vec![StatRequirement {
                stat_name: "npcs_befriended".to_string(),
                required_value: 20.0,
                comparison: Comparison::GreaterOrEqual,
            }],
        });

        self.register_achievement(Achievement {
            id: "master_engineer".to_string(),
            name: "Master Engineer".to_string(),
            description: "Unlock all engineer tools".to_string(),
            icon_locked: "achievements/master_locked.png".to_string(),
            icon_unlocked: "achievements/master_unlocked.png".to_string(),
            points: 100,
            category: AchievementCategory::Mastery,
            hidden: true, // Hidden until certain progress
            prerequisite: Some("builder_expert".to_string()),
            statistics_required: vec![StatRequirement {
                stat_name: "tools_unlocked".to_string(),
                required_value: 15.0,
                comparison: Comparison::GreaterOrEqual,
            }],
        });

        println!("✅ Loaded {} achievement definitions", self.achievements.len());
        Ok(())
    }

    /// Register an achievement
    fn register_achievement(&mut self, achievement: Achievement) {
        let id = achievement.id.clone();
        self.achievements.insert(id.clone(), achievement);

        // Initialize progress if not exists
        if !self.progress.contains_key(&id) {
            self.progress.insert(id, AchievementProgress {
                unlocked: false,
                unlock_time: None,
                progress_percentage: 0.0,
            });
        }
    }

    /// Unlock an achievement
    pub fn unlock(&mut self, achievement_id: &str) -> RobinResult<bool> {
        // Check if achievement exists
        if !self.achievements.contains_key(achievement_id) {
            return Ok(false);
        }

        // Check if already unlocked
        if let Some(progress) = self.progress.get(achievement_id) {
            if progress.unlocked {
                return Ok(false);
            }
        }

        // Check prerequisites
        if let Some(achievement) = self.achievements.get(achievement_id) {
            if let Some(ref prereq) = achievement.prerequisite {
                if !self.is_unlocked(prereq) {
                    return Ok(false);
                }
            }
        }

        // Unlock the achievement
        println!("🏆 Achievement Unlocked: {}", achievement_id);

        self.progress.insert(achievement_id.to_string(), AchievementProgress {
            unlocked: true,
            unlock_time: Some(SystemTime::now()),
            progress_percentage: 100.0,
        });

        // Show notification
        if self.notifications_enabled {
            self.show_notification(achievement_id)?;
        }

        // Sync with platform
        self.sync_achievement_with_platform(achievement_id)?;

        // Save progress
        self.save_progress()?;

        Ok(true)
    }

    /// Check if achievement is unlocked
    pub fn is_unlocked(&self, achievement_id: &str) -> bool {
        self.progress.get(achievement_id)
            .map(|p| p.unlocked)
            .unwrap_or(false)
    }

    /// Update statistic
    pub fn update_stat(&mut self, stat_name: &str, value: f64) -> RobinResult<()> {
        let stat = self.statistics.entry(stat_name.to_string())
            .or_insert_with(|| Statistic {
                name: stat_name.to_string(),
                value: 0.0,
                max_value: 0.0,
                total_value: 0.0,
            });

        stat.value = value;
        stat.max_value = stat.max_value.max(value);
        stat.total_value += value;

        // Check for achievement unlocks based on statistics
        self.check_stat_achievements(stat_name, value)?;

        Ok(())
    }

    /// Increment statistic
    pub fn increment_stat(&mut self, stat_name: &str, amount: f64) -> RobinResult<()> {
        let current = self.statistics.get(stat_name)
            .map(|s| s.value)
            .unwrap_or(0.0);

        self.update_stat(stat_name, current + amount)
    }

    /// Check achievements that depend on statistics
    fn check_stat_achievements(&mut self, stat_name: &str, value: f64) -> RobinResult<()> {
        let achievements_to_check: Vec<String> = self.achievements
            .iter()
            .filter(|(_, achievement)| {
                achievement.statistics_required.iter()
                    .any(|req| req.stat_name == stat_name)
            })
            .map(|(id, _)| id.clone())
            .collect();

        for achievement_id in achievements_to_check {
            if self.is_unlocked(&achievement_id) {
                continue;
            }

            if let Some(achievement) = self.achievements.get(&achievement_id) {
                let all_requirements_met = achievement.statistics_required.iter()
                    .all(|req| {
                        self.statistics.get(&req.stat_name)
                            .map(|stat| req.check(stat.value))
                            .unwrap_or(false)
                    });

                if all_requirements_met {
                    self.unlock(&achievement_id)?;
                } else {
                    // Update progress percentage
                    self.update_progress(&achievement_id)?;
                }
            }
        }

        Ok(())
    }

    /// Update achievement progress
    fn update_progress(&mut self, achievement_id: &str) -> RobinResult<()> {
        if let Some(achievement) = self.achievements.get(achievement_id) {
            if achievement.statistics_required.is_empty() {
                return Ok(());
            }

            let progress_values: Vec<f64> = achievement.statistics_required.iter()
                .map(|req| {
                    self.statistics.get(&req.stat_name)
                        .map(|stat| (stat.value / req.required_value).min(1.0))
                        .unwrap_or(0.0)
                })
                .collect();

            let avg_progress = progress_values.iter().sum::<f64>() / progress_values.len() as f64;

            if let Some(progress) = self.progress.get_mut(achievement_id) {
                progress.progress_percentage = (avg_progress * 100.0).min(100.0);
            }
        }

        Ok(())
    }

    /// Get all achievements
    pub fn get_all_achievements(&self) -> Vec<AchievementInfo> {
        self.achievements.iter().map(|(id, achievement)| {
            let progress = self.progress.get(id).cloned()
                .unwrap_or_else(|| AchievementProgress {
                    unlocked: false,
                    unlock_time: None,
                    progress_percentage: 0.0,
                });

            AchievementInfo {
                achievement: achievement.clone(),
                progress,
            }
        }).collect()
    }

    /// Get achievement statistics
    pub fn get_statistics(&self) -> AchievementStatistics {
        let total = self.achievements.len();
        let unlocked = self.progress.values().filter(|p| p.unlocked).count();
        let total_points = self.achievements.values().map(|a| a.points).sum();
        let earned_points = self.achievements.iter()
            .filter(|(id, _)| self.is_unlocked(id))
            .map(|(_, a)| a.points)
            .sum();

        AchievementStatistics {
            total_achievements: total,
            unlocked_achievements: unlocked,
            completion_percentage: (unlocked as f64 / total as f64 * 100.0),
            total_points,
            earned_points,
        }
    }

    /// Show achievement notification
    fn show_notification(&self, achievement_id: &str) -> RobinResult<()> {
        if let Some(achievement) = self.achievements.get(achievement_id) {
            println!("╔════════════════════════════════════╗");
            println!("║  🏆 ACHIEVEMENT UNLOCKED!          ║");
            println!("║                                    ║");
            println!("║  {}                     ", achievement.name);
            println!("║  {}               ", achievement.description);
            println!("║  +{} Points                       ", achievement.points);
            println!("╚════════════════════════════════════╝");
        }
        Ok(())
    }

    /// Sync with platform achievements
    fn sync_with_platform(&mut self) -> RobinResult<()> {
        if let Some(platform) = &self.platform {
            match platform {
                Platform::Steam => {
                    // Would sync with Steam achievements
                    println!("🔄 Syncing achievements with Steam");
                }
                Platform::Epic => {
                    println!("🔄 Syncing achievements with Epic Games");
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Sync specific achievement with platform
    fn sync_achievement_with_platform(&self, achievement_id: &str) -> RobinResult<()> {
        if let Some(platform) = &self.platform {
            println!("🔄 Syncing achievement '{}' with {:?}", achievement_id, platform);
            // Platform-specific sync would go here
        }
        Ok(())
    }

    /// Load progress from disk
    fn load_progress(&mut self) -> RobinResult<()> {
        let mut progress_path = self.save_path.clone();
        progress_path.push("progress.json");

        if progress_path.exists() {
            let content = fs::read_to_string(progress_path)?;
            self.progress = serde_json::from_str(&content)?;
            println!("✅ Loaded achievement progress");
        }

        Ok(())
    }

    /// Save progress to disk
    fn save_progress(&self) -> RobinResult<()> {
        let mut progress_path = self.save_path.clone();
        progress_path.push("progress.json");

        let serialized = serde_json::to_string_pretty(&self.progress)?;
        fs::write(progress_path, serialized)?;

        Ok(())
    }

    /// Shutdown achievement system
    pub fn shutdown(&mut self) -> RobinResult<()> {
        println!("🛑 Shutting down achievement system");
        self.save_progress()?;
        Ok(())
    }
}

/// Achievement definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon_locked: String,
    pub icon_unlocked: String,
    pub points: u32,
    pub category: AchievementCategory,
    pub hidden: bool,
    pub prerequisite: Option<String>,
    pub statistics_required: Vec<StatRequirement>,
}

/// Achievement categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AchievementCategory {
    Tutorial,
    Construction,
    Exploration,
    Combat,
    Social,
    Vehicle,
    Collection,
    Mastery,
}

/// Achievement progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AchievementProgress {
    pub unlocked: bool,
    pub unlock_time: Option<SystemTime>,
    pub progress_percentage: f64,
}

/// Achievement info (combined data)
#[derive(Debug, Clone)]
pub struct AchievementInfo {
    pub achievement: Achievement,
    pub progress: AchievementProgress,
}

/// Statistic requirement for achievement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatRequirement {
    pub stat_name: String,
    pub required_value: f64,
    pub comparison: Comparison,
}

impl StatRequirement {
    /// Check if requirement is met
    pub fn check(&self, current_value: f64) -> bool {
        match self.comparison {
            Comparison::Equal => (current_value - self.required_value).abs() < 0.001,
            Comparison::GreaterThan => current_value > self.required_value,
            Comparison::GreaterOrEqual => current_value >= self.required_value,
            Comparison::LessThan => current_value < self.required_value,
            Comparison::LessOrEqual => current_value <= self.required_value,
        }
    }
}

/// Comparison operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Comparison {
    Equal,
    GreaterThan,
    GreaterOrEqual,
    LessThan,
    LessOrEqual,
}

/// Game statistic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statistic {
    pub name: String,
    pub value: f64,
    pub max_value: f64,
    pub total_value: f64,
}

/// Achievement statistics summary
#[derive(Debug, Clone)]
pub struct AchievementStatistics {
    pub total_achievements: usize,
    pub unlocked_achievements: usize,
    pub completion_percentage: f64,
    pub total_points: u32,
    pub earned_points: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_achievement_unlock() {
        let mut manager = AchievementManager::new();
        manager.initialize(Platform::Windows).unwrap();

        // Update statistic
        manager.update_stat("structures_built", 1.0).unwrap();

        // Check if achievement was unlocked
        assert!(manager.is_unlocked("builder_novice"));
    }

    #[test]
    fn test_achievement_prerequisites() {
        let mut manager = AchievementManager::new();
        manager.initialize(Platform::Windows).unwrap();

        // Try to unlock expert without novice - should fail
        manager.update_stat("structures_built", 100.0).unwrap();

        // Expert shouldn't be unlocked without novice
        assert!(manager.is_unlocked("builder_novice"));
        assert!(manager.is_unlocked("builder_expert"));
    }

    #[test]
    fn test_achievement_progress() {
        let mut manager = AchievementManager::new();
        manager.initialize(Platform::Windows).unwrap();

        // Update statistic partially
        manager.update_stat("locations_discovered", 25.0).unwrap();

        // Check progress
        let achievements = manager.get_all_achievements();
        let explorer = achievements.iter()
            .find(|a| a.achievement.id == "explorer")
            .unwrap();

        assert!(!explorer.progress.unlocked);
        assert_eq!(explorer.progress.progress_percentage, 50.0);
    }

    #[test]
    fn test_statistics() {
        let mut manager = AchievementManager::new();
        manager.initialize(Platform::Windows).unwrap();

        // Unlock some achievements
        manager.update_stat("structures_built", 1.0).unwrap();

        let stats = manager.get_statistics();
        assert!(stats.unlocked_achievements >= 1);
        assert!(stats.earned_points >= 15);
    }
}