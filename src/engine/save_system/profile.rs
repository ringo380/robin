use super::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::SystemTime,
};

/// User profile that manages multiple save games and user preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    /// Profile ID (username or UUID)
    pub id: String,
    /// Display name
    pub display_name: String,
    /// Profile creation time
    pub created_at: SystemTime,
    /// Last accessed time
    pub last_accessed: SystemTime,
    /// Total playtime across all saves (seconds)
    pub total_playtime: u64,
    /// User statistics
    pub statistics: ProfileStats,
    /// User preferences
    pub preferences: UserPreferences,
    /// Achievement progress
    pub achievements: AchievementProgress,
    /// Save game references
    pub save_games: HashMap<usize, SaveGameInfo>,
    /// Profile avatar/icon
    pub avatar: Option<String>,
}

impl UserProfile {
    /// Create a new user profile
    pub fn new(id: String, display_name: String) -> Self {
        let now = SystemTime::now();
        Self {
            id,
            display_name,
            created_at: now,
            last_accessed: now,
            total_playtime: 0,
            statistics: ProfileStats::default(),
            preferences: UserPreferences::default(),
            achievements: AchievementProgress::default(),
            save_games: HashMap::new(),
            avatar: None,
        }
    }

    /// Update last accessed time
    pub fn update_access_time(&mut self) {
        self.last_accessed = SystemTime::now();
    }

    /// Add playtime to the profile
    pub fn add_playtime(&mut self, seconds: u64) {
        self.total_playtime += seconds;
        self.statistics.total_sessions += 1;
        self.update_access_time();
    }

    /// Register a save game with this profile
    pub fn register_save_game(&mut self, slot: usize, save_info: SaveGameInfo) {
        self.save_games.insert(slot, save_info);
    }

    /// Remove a save game from this profile
    pub fn remove_save_game(&mut self, slot: usize) {
        self.save_games.remove(&slot);
    }

    /// Get formatted total playtime
    pub fn get_total_playtime_string(&self) -> String {
        let hours = self.total_playtime / 3600;
        let minutes = (self.total_playtime % 3600) / 60;
        format!("{}h {}m", hours, minutes)
    }

    /// Update achievement progress
    pub fn update_achievement(&mut self, achievement_id: &str, progress: f32) {
        self.achievements.update_progress(achievement_id, progress);
    }

    /// Unlock an achievement
    pub fn unlock_achievement(&mut self, achievement_id: &str) {
        self.achievements.unlock(achievement_id);
        self.statistics.achievements_unlocked += 1;
    }
}

/// Profile statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfileStats {
    /// Total number of game sessions
    pub total_sessions: u32,
    /// Total saves created
    pub saves_created: u32,
    /// Total games completed
    pub games_completed: u32,
    /// Achievements unlocked
    pub achievements_unlocked: u32,
    /// Favorite genre/game mode
    pub favorite_genre: Option<String>,
    /// Average session length (seconds)
    pub average_session_length: u64,
    /// Longest single session (seconds)
    pub longest_session: u64,
}

/// User preferences that persist across all games
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    /// Language setting
    pub language: String,
    /// UI scale factor
    pub ui_scale: f32,
    /// Color theme
    pub color_theme: ColorTheme,
    /// Notification settings
    pub notifications: NotificationSettings,
    /// Privacy settings
    pub privacy: PrivacySettings,
    /// Default game settings
    pub default_game_settings: GameSettings,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            ui_scale: 1.0,
            color_theme: ColorTheme::Default,
            notifications: NotificationSettings::default(),
            privacy: PrivacySettings::default(),
            default_game_settings: GameSettings::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColorTheme {
    Default,
    Dark,
    Light,
    HighContrast,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    /// Enable achievement notifications
    pub achievement_notifications: bool,
    /// Enable save reminders
    pub save_reminders: bool,
    /// Enable update notifications
    pub update_notifications: bool,
    /// Sound for notifications
    pub notification_sound: bool,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            achievement_notifications: true,
            save_reminders: true,
            update_notifications: true,
            notification_sound: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySettings {
    /// Share statistics with developers
    pub share_statistics: bool,
    /// Enable crash reporting
    pub crash_reporting: bool,
    /// Allow telemetry collection
    pub telemetry: bool,
}

impl Default for PrivacySettings {
    fn default() -> Self {
        Self {
            share_statistics: false,
            crash_reporting: true,
            telemetry: false,
        }
    }
}

/// Achievement progress tracking
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AchievementProgress {
    /// Unlocked achievements
    pub unlocked: Vec<String>,
    /// Achievement progress (0.0 to 1.0)
    pub progress: HashMap<String, f32>,
    /// Achievement unlock timestamps
    pub unlock_times: HashMap<String, SystemTime>,
}

impl AchievementProgress {
    /// Update progress for an achievement
    pub fn update_progress(&mut self, achievement_id: &str, progress: f32) {
        let clamped_progress = progress.clamp(0.0, 1.0);
        self.progress.insert(achievement_id.to_string(), clamped_progress);
        
        // Auto-unlock if progress reaches 100%
        if clamped_progress >= 1.0 {
            self.unlock(achievement_id);
        }
    }

    /// Unlock an achievement
    pub fn unlock(&mut self, achievement_id: &str) {
        let id = achievement_id.to_string();
        if !self.unlocked.contains(&id) {
            self.unlocked.push(id.clone());
            self.unlock_times.insert(id, SystemTime::now());
            self.progress.insert(achievement_id.to_string(), 1.0);
        }
    }

    /// Check if achievement is unlocked
    pub fn is_unlocked(&self, achievement_id: &str) -> bool {
        self.unlocked.contains(&achievement_id.to_string())
    }

    /// Get achievement progress
    pub fn get_progress(&self, achievement_id: &str) -> f32 {
        self.progress.get(achievement_id).copied().unwrap_or(0.0)
    }

    /// Get completion percentage
    pub fn get_completion_percentage(&self, total_achievements: usize) -> f32 {
        if total_achievements == 0 {
            return 100.0;
        }
        (self.unlocked.len() as f32 / total_achievements as f32) * 100.0
    }
}

/// Reference to a save game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveGameInfo {
    /// Save slot number
    pub slot: usize,
    /// Save name
    pub name: String,
    /// Last modified time
    pub last_modified: SystemTime,
    /// Playtime for this save
    pub playtime: u64,
    /// Progress percentage
    pub progress: f32,
    /// Current scene/level
    pub current_scene: String,
    /// Quick access thumbnail
    pub thumbnail: Option<String>,
}

impl SaveGameInfo {
    pub fn new(slot: usize, metadata: &SaveMetadata) -> Self {
        Self {
            slot,
            name: metadata.name.clone(),
            last_modified: metadata.modified_at,
            playtime: metadata.play_time,
            progress: metadata.progress,
            current_scene: metadata.current_scene.clone(),
            thumbnail: metadata.thumbnail.clone(),
        }
    }

    /// Update info from save metadata
    pub fn update_from_metadata(&mut self, metadata: &SaveMetadata) {
        self.name = metadata.name.clone();
        self.last_modified = metadata.modified_at;
        self.playtime = metadata.play_time;
        self.progress = metadata.progress;
        self.current_scene = metadata.current_scene.clone();
        self.thumbnail = metadata.thumbnail.clone();
    }
}

/// Profile manager handles multiple user profiles
pub struct ProfileManager {
    /// Configuration
    config: SaveSystemConfig,
    /// Currently loaded profiles
    profiles: HashMap<String, UserProfile>,
    /// Active profile ID
    active_profile_id: Option<String>,
    /// Profile directory
    profile_directory: PathBuf,
}

impl ProfileManager {
    /// Create a new profile manager
    pub fn new(config: SaveSystemConfig) -> SaveResult<Self> {
        let profile_directory = config.save_directory.join("profiles");
        std::fs::create_dir_all(&profile_directory)
            .map_err(|e| SaveError::IoError(e.to_string()))?;

        let mut manager = Self {
            config,
            profiles: HashMap::new(),
            active_profile_id: None,
            profile_directory,
        };

        manager.scan_profiles()?;
        Ok(manager)
    }

    /// Scan for existing profiles
    fn scan_profiles(&mut self) -> SaveResult<()> {
        if !self.profile_directory.exists() {
            return Ok(());
        }

        let entries = std::fs::read_dir(&self.profile_directory)
            .map_err(|e| SaveError::IoError(e.to_string()))?;

        for entry in entries {
            let entry = entry.map_err(|e| SaveError::IoError(e.to_string()))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("profile") {
                if let Ok(profile) = self.load_profile_from_file(&path) {
                    self.profiles.insert(profile.id.clone(), profile);
                }
            }
        }

        Ok(())
    }

    /// Create a new user profile
    pub fn create_profile(&mut self, id: String, display_name: String) -> SaveResult<()> {
        if self.profiles.contains_key(&id) {
            return Err(SaveError::IoError("Profile already exists".to_string()));
        }

        let profile = UserProfile::new(id.clone(), display_name);
        self.save_profile(&profile)?;
        self.profiles.insert(id, profile);

        Ok(())
    }

    /// Set the active profile
    pub fn set_active_profile(&mut self, profile_id: &str) -> SaveResult<()> {
        if !self.profiles.contains_key(profile_id) {
            return Err(SaveError::IoError("Profile not found".to_string()));
        }

        self.active_profile_id = Some(profile_id.to_string());
        
        // Update access time
        if let Some(profile) = self.profiles.get_mut(profile_id) {
            profile.update_access_time();
        }
        
        // Save the profile (separate borrow)
        if let Some(profile) = self.profiles.get(profile_id) {
            self.save_profile(profile)?;
        }

        Ok(())
    }

    /// Get the active profile
    pub fn get_active_profile(&self) -> Option<&UserProfile> {
        self.active_profile_id.as_ref()
            .and_then(|id| self.profiles.get(id))
    }

    /// Get mutable reference to active profile
    pub fn get_active_profile_mut(&mut self) -> Option<&mut UserProfile> {
        let profile_id = self.active_profile_id.clone()?;
        self.profiles.get_mut(&profile_id)
    }

    /// Get a profile by ID
    pub fn get_profile(&self, profile_id: &str) -> Option<&UserProfile> {
        self.profiles.get(profile_id)
    }

    /// Get all profiles
    pub fn get_all_profiles(&self) -> &HashMap<String, UserProfile> {
        &self.profiles
    }

    /// Delete a profile
    pub fn delete_profile(&mut self, profile_id: &str) -> SaveResult<()> {
        if let Some(_profile) = self.profiles.remove(profile_id) {
            let profile_path = self.get_profile_path(profile_id);
            if profile_path.exists() {
                std::fs::remove_file(&profile_path)
                    .map_err(|e| SaveError::IoError(e.to_string()))?;
            }

            // If this was the active profile, clear it
            if self.active_profile_id.as_deref() == Some(profile_id) {
                self.active_profile_id = None;
            }
        }

        Ok(())
    }

    /// Save a profile to disk
    pub fn save_profile(&self, profile: &UserProfile) -> SaveResult<()> {
        let profile_path = self.get_profile_path(&profile.id);
        let data = serde_json::to_vec_pretty(profile)
            .map_err(|e| SaveError::SerializationError(e.to_string()))?;

        std::fs::write(&profile_path, data)
            .map_err(|e| SaveError::IoError(e.to_string()))?;

        Ok(())
    }

    /// Load a profile from disk
    fn load_profile_from_file(&self, path: &Path) -> SaveResult<UserProfile> {
        let data = std::fs::read(path)
            .map_err(|e| SaveError::IoError(e.to_string()))?;

        let profile: UserProfile = serde_json::from_slice(&data)
            .map_err(|e| SaveError::SerializationError(e.to_string()))?;

        Ok(profile)
    }

    /// Get the file path for a profile
    fn get_profile_path(&self, profile_id: &str) -> PathBuf {
        let filename = format!("{}.profile", profile_id);
        self.profile_directory.join(filename)
    }

    /// Update active profile with playtime and save info
    pub fn update_active_profile_from_save(&mut self, save_metadata: &SaveMetadata, playtime_delta: u64) -> SaveResult<()> {
        // Get the profile ID first
        let profile_id = self.active_profile_id.clone();
        
        if let Some(id) = profile_id {
            // Update the profile
            if let Some(profile) = self.profiles.get_mut(&id) {
                profile.add_playtime(playtime_delta);
                
                let save_info = SaveGameInfo::new(save_metadata.slot, save_metadata);
                profile.register_save_game(save_metadata.slot, save_info);
            }
            
            // Save the profile (separate borrow)
            if let Some(profile) = self.profiles.get(&id) {
                self.save_profile(profile)?;
            }
        }
        Ok(())
    }

    /// Get recommended next profile to use (most recently accessed)
    pub fn get_recent_profile_id(&self) -> Option<String> {
        let mut profiles: Vec<_> = self.profiles.values().collect();
        profiles.sort_by(|a, b| b.last_accessed.cmp(&a.last_accessed));
        profiles.first().map(|p| p.id.clone())
    }

    pub fn save_active_profile(&self) -> SaveResult<()> {
        if let Some(active_id) = &self.active_profile_id {
            if let Some(profile) = self.profiles.get(active_id) {
                return self.save_profile(profile);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_config() -> SaveSystemConfig {
        let temp_dir = TempDir::new().unwrap();
        SaveSystemConfig {
            save_directory: temp_dir.path().to_path_buf(),
            ..SaveSystemConfig::default()
        }
    }

    #[test]
    fn test_user_profile_creation() {
        let profile = UserProfile::new("test_user".to_string(), "Test User".to_string());
        
        assert_eq!(profile.id, "test_user");
        assert_eq!(profile.display_name, "Test User");
        assert_eq!(profile.total_playtime, 0);
    }

    #[test]
    fn test_achievement_progress() {
        let mut achievements = AchievementProgress::default();
        
        achievements.update_progress("first_kill", 0.5);
        assert_eq!(achievements.get_progress("first_kill"), 0.5);
        assert!(!achievements.is_unlocked("first_kill"));
        
        achievements.update_progress("first_kill", 1.0);
        assert!(achievements.is_unlocked("first_kill"));
    }

    #[test]
    fn test_profile_manager() {
        let config = create_test_config();
        let mut manager = ProfileManager::new(config).unwrap();
        
        // Create a profile
        manager.create_profile("test".to_string(), "Test User".to_string()).unwrap();
        assert!(manager.profiles.contains_key("test"));
        
        // Set active profile
        manager.set_active_profile("test").unwrap();
        assert!(manager.get_active_profile().is_some());
        
        // Delete profile
        manager.delete_profile("test").unwrap();
        assert!(!manager.profiles.contains_key("test"));
        assert!(manager.get_active_profile().is_none());
    }

    #[test]
    fn test_save_game_info() {
        let metadata = SaveMetadata::new(1, "Test Save".to_string());
        let save_info = SaveGameInfo::new(1, &metadata);
        
        assert_eq!(save_info.slot, 1);
        assert_eq!(save_info.name, "Test Save");
    }
}