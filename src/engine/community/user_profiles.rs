// Robin Engine User Profiles and Achievement System
// User identity, progression, and social features

use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use tokio::sync::broadcast;

use crate::engine::{
    error::{RobinResult, RobinError},
    math::Vec3,
    platform::cloud_saves::SaveData,
};

/// User profile manager
pub struct UserProfileManager {
    /// User profiles
    profiles: HashMap<Uuid, UserProfile>,

    /// Achievement system
    achievement_system: AchievementSystem,

    /// Friend system
    friend_manager: FriendManager,

    /// Activity tracking
    activity_tracker: ActivityTracker,

    /// Event broadcasting
    event_sender: broadcast::Sender<ProfileEvent>,

    /// Configuration
    config: ProfileConfig,

    /// Storage path
    storage_path: std::path::PathBuf,

    /// Feature enabled
    enabled: bool,
}

impl UserProfileManager {
    /// Create a new user profile manager
    pub fn new() -> RobinResult<Self> {
        let (event_sender, _) = broadcast::channel(1000);

        Ok(Self {
            profiles: HashMap::new(),
            achievement_system: AchievementSystem::new(),
            friend_manager: FriendManager::new(),
            activity_tracker: ActivityTracker::new(),
            event_sender,
            config: ProfileConfig::default(),
            storage_path: std::path::PathBuf::from("data/profiles"),
            enabled: true,
        })
    }

    /// Initialize the profile system
    pub async fn initialize(&mut self) -> RobinResult<()> {
        if !self.enabled {
            return Ok(());
        }

        log::info!("👤 Initializing User Profile Manager");

        // Create storage directory
        std::fs::create_dir_all(&self.storage_path)
            .map_err(|e| RobinError::Community(format!("Failed to create profiles directory: {}", e)))?;

        // Initialize achievement system
        self.achievement_system.initialize().await?;

        // Load existing profiles
        self.load_profiles().await?;

        log::info!("✅ User Profile Manager initialized with {} profiles", self.profiles.len());
        Ok(())
    }

    /// Create or update user profile
    pub async fn create_or_update_profile(&mut self, params: ProfileParams) -> RobinResult<Uuid> {
        let user_id = params.user_id;

        // Check if profile exists
        if let Some(existing_profile) = self.profiles.get_mut(&user_id) {
            // Update existing profile
            if let Some(display_name) = params.display_name {
                existing_profile.display_name = display_name;
            }
            if let Some(bio) = params.bio {
                existing_profile.bio = bio;
            }
            if let Some(avatar_url) = params.avatar_url {
                existing_profile.avatar_url = Some(avatar_url);
            }
            if let Some(social_links) = params.social_links {
                existing_profile.social_links = social_links;
            }

            existing_profile.last_updated = SystemTime::now();

            // Broadcast update event
            let event = ProfileEvent::ProfileUpdated {
                user_id,
                display_name: existing_profile.display_name.clone(),
            };
            self.broadcast_event(event);

            log::info!("📝 Updated profile for user {}", user_id);
        } else {
            // Create new profile
            let profile = UserProfile {
                user_id,
                display_name: params.display_name.unwrap_or_else(|| format!("User_{}", user_id)),
                bio: params.bio.unwrap_or_default(),
                avatar_url: params.avatar_url,
                social_links: params.social_links.unwrap_or_default(),
                join_date: SystemTime::now(),
                last_active: SystemTime::now(),
                last_updated: SystemTime::now(),
                level: 1,
                experience_points: 0,
                total_build_time: Duration::from_secs(0),
                total_voxels_placed: 0,
                total_projects_created: 0,
                total_collaborations: 0,
                achievements: HashSet::new(),
                badges: Vec::new(),
                reputation: 0,
                settings: UserSettings::default(),
                privacy: PrivacySettings::default(),
                statistics: UserStatistics::new(),
                activity_history: VecDeque::with_capacity(1000),
                featured_creations: Vec::new(),
            };

            self.profiles.insert(user_id, profile);

            // Broadcast creation event
            let event = ProfileEvent::ProfileCreated {
                user_id,
                display_name: self.profiles[&user_id].display_name.clone(),
            };
            self.broadcast_event(event);

            log::info!("👤 Created profile for user {}", user_id);
        }

        // Save profile
        self.save_profile(&user_id).await?;

        Ok(user_id)
    }

    /// Get user profile
    pub fn get_profile(&self, user_id: &Uuid) -> Option<&UserProfile> {
        self.profiles.get(user_id)
    }

    /// Get public profile (respecting privacy settings)
    pub fn get_public_profile(&self, user_id: &Uuid, viewer_id: Option<Uuid>) -> Option<PublicProfile> {
        let profile = self.profiles.get(user_id)?;

        // Check privacy settings
        let is_friend = viewer_id
            .map(|viewer| self.friend_manager.are_friends(user_id, &viewer))
            .unwrap_or(false);

        let show_full_profile = profile.privacy.profile_visibility == ProfileVisibility::Public ||
            (profile.privacy.profile_visibility == ProfileVisibility::Friends && is_friend) ||
            viewer_id == Some(*user_id);

        if !show_full_profile && profile.privacy.profile_visibility == ProfileVisibility::Private {
            return None;
        }

        Some(PublicProfile {
            user_id: profile.user_id,
            display_name: profile.display_name.clone(),
            bio: if show_full_profile { Some(profile.bio.clone()) } else { None },
            avatar_url: profile.avatar_url.clone(),
            join_date: profile.join_date,
            level: profile.level,
            achievements: if profile.privacy.show_achievements || show_full_profile {
                profile.achievements.clone()
            } else {
                HashSet::new()
            },
            badges: if profile.privacy.show_badges || show_full_profile {
                profile.badges.clone()
            } else {
                Vec::new()
            },
            reputation: if profile.privacy.show_reputation || show_full_profile {
                Some(profile.reputation)
            } else {
                None
            },
            statistics: if profile.privacy.show_statistics || show_full_profile {
                Some(profile.statistics.clone())
            } else {
                None
            },
            featured_creations: profile.featured_creations.clone(),
        })
    }

    /// Update user activity
    pub async fn update_activity(&mut self, user_id: Uuid, activity: ActivityType) -> RobinResult<()> {
        let profile = self.profiles.get_mut(&user_id)
            .ok_or_else(|| RobinError::Community("User profile not found".to_string()))?;

        profile.last_active = SystemTime::now();

        // Track activity
        self.activity_tracker.track_activity(user_id, activity.clone()).await?;

        // Add to activity history
        let activity_entry = ActivityEntry {
            activity_type: activity.clone(),
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
        };

        profile.activity_history.push_back(activity_entry);

        // Keep history within limits
        if profile.activity_history.len() > 1000 {
            profile.activity_history.pop_front();
        }

        // Update statistics and check for achievements
        self.update_user_statistics(user_id, &activity).await?;
        self.check_achievements(user_id).await?;

        Ok(())
    }

    /// Award achievement to user
    pub async fn award_achievement(&mut self, user_id: Uuid, achievement_id: String) -> RobinResult<()> {
        let profile = self.profiles.get_mut(&user_id)
            .ok_or_else(|| RobinError::Community("User profile not found".to_string()))?;

        // Check if user already has this achievement
        if profile.achievements.contains(&achievement_id) {
            return Ok(());
        }

        // Get achievement details
        let achievement = self.achievement_system.get_achievement(&achievement_id)
            .ok_or_else(|| RobinError::Community("Achievement not found".to_string()))?;

        // Award achievement
        profile.achievements.insert(achievement_id.clone());
        profile.experience_points += achievement.experience_reward;

        // Calculate level inline to avoid borrow conflicts
        let experience_points = profile.experience_points;
        let mut new_level: u32 = 1;
        let mut total_xp_needed: u64 = 0;
        let mut xp_for_next_level: u64 = 100;

        while total_xp_needed + xp_for_next_level <= experience_points {
            total_xp_needed += xp_for_next_level;
            new_level += 1;
            xp_for_next_level = 100 * (new_level as u64);
        }

        // Check for level up
        let level_up = new_level > profile.level;
        profile.level = new_level;

        // Add badge if achievement grants one
        if let Some(badge) = &achievement.badge {
            profile.badges.push(badge.clone());
        }

        // Broadcast achievement event
        let event = ProfileEvent::AchievementEarned {
            user_id,
            achievement_id: achievement_id.clone(),
            achievement_name: achievement.name.clone(),
            experience_gained: achievement.experience_reward,
            level_up,
            new_level: profile.level,
        };
        self.broadcast_event(event);

        // Save profile
        self.save_profile(&user_id).await?;

        log::info!("🏆 User {} earned achievement: {}", user_id, achievement.name);
        Ok(())
    }

    /// Get user achievements
    pub fn get_user_achievements(&self, user_id: &Uuid) -> Vec<Achievement> {
        if let Some(profile) = self.profiles.get(user_id) {
            profile.achievements.iter()
                .filter_map(|id| self.achievement_system.get_achievement(id))
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get leaderboard
    pub fn get_leaderboard(&self, criteria: LeaderboardCriteria, limit: usize) -> Vec<LeaderboardEntry> {
        let mut entries: Vec<LeaderboardEntry> = self.profiles.values()
            .filter(|p| p.privacy.show_on_leaderboards)
            .map(|profile| {
                let score = match criteria {
                    LeaderboardCriteria::Level => profile.level as f64,
                    LeaderboardCriteria::ExperiencePoints => profile.experience_points as f64,
                    LeaderboardCriteria::BuildTime => profile.total_build_time.as_secs() as f64,
                    LeaderboardCriteria::VoxelsPlaced => profile.total_voxels_placed as f64,
                    LeaderboardCriteria::ProjectsCreated => profile.total_projects_created as f64,
                    LeaderboardCriteria::Reputation => profile.reputation as f64,
                    LeaderboardCriteria::Achievements => profile.achievements.len() as f64,
                };

                LeaderboardEntry {
                    user_id: profile.user_id,
                    display_name: profile.display_name.clone(),
                    avatar_url: profile.avatar_url.clone(),
                    score,
                    rank: 0, // Will be set after sorting
                }
            })
            .collect();

        // Sort by score descending
        entries.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Set ranks
        for (index, entry) in entries.iter_mut().enumerate() {
            entry.rank = index + 1;
        }

        entries.into_iter().take(limit).collect()
    }

    /// Search users
    pub fn search_users(&self, query: &str, limit: usize) -> Vec<UserSearchResult> {
        let query_lower = query.to_lowercase();

        self.profiles.values()
            .filter(|profile| {
                profile.privacy.profile_visibility == ProfileVisibility::Public &&
                (profile.display_name.to_lowercase().contains(&query_lower) ||
                 profile.bio.to_lowercase().contains(&query_lower))
            })
            .take(limit)
            .map(|profile| UserSearchResult {
                user_id: profile.user_id,
                display_name: profile.display_name.clone(),
                avatar_url: profile.avatar_url.clone(),
                level: profile.level,
                achievement_count: profile.achievements.len(),
                reputation: profile.reputation,
            })
            .collect()
    }

    /// Get total user count
    pub fn get_user_count(&self) -> u64 {
        self.profiles.len() as u64
    }

    /// Get friend manager
    pub fn friend_manager(&self) -> &FriendManager {
        &self.friend_manager
    }

    /// Get mutable friend manager
    pub fn friend_manager_mut(&mut self) -> &mut FriendManager {
        &mut self.friend_manager
    }

    // Helper methods

    async fn update_user_statistics(&mut self, user_id: Uuid, activity: &ActivityType) -> RobinResult<()> {
        let profile = self.profiles.get_mut(&user_id)
            .ok_or_else(|| RobinError::Community("User profile not found".to_string()))?;

        match activity {
            ActivityType::VoxelPlaced { count, .. } => {
                profile.total_voxels_placed += count;
                profile.statistics.blocks_placed += count;
            },
            ActivityType::VoxelRemoved { count, .. } => {
                profile.statistics.blocks_removed += count;
            },
            ActivityType::ProjectCreated { .. } => {
                profile.total_projects_created += 1;
                profile.statistics.projects_created += 1;
            },
            ActivityType::BuildSession { duration, .. } => {
                profile.total_build_time += *duration;
                profile.statistics.total_build_time += *duration;
            },
            ActivityType::RoomJoined { .. } => {
                profile.statistics.rooms_joined += 1;
            },
            ActivityType::CollaborationStarted { .. } => {
                profile.total_collaborations += 1;
                profile.statistics.collaborations_started += 1;
            },
            _ => {}, // Other activities don't affect core statistics
        }

        Ok(())
    }

    async fn check_achievements(&mut self, user_id: Uuid) -> RobinResult<()> {
        let profile = self.profiles.get(&user_id).cloned()
            .ok_or_else(|| RobinError::Community("User profile not found".to_string()))?;

        // Collect achievements to avoid borrow conflicts
        let available_achievements: Vec<_> = self.achievement_system.get_all_achievements().into_iter().collect();

        // First collect achievements that should be awarded
        let mut achievements_to_award = Vec::new();

        for achievement in available_achievements {
            // Skip if user already has this achievement
            if profile.achievements.contains(&achievement.id) {
                continue;
            }

            // Check if achievement criteria are met
            if self.achievement_system.check_criteria(&achievement, &profile).await? {
                achievements_to_award.push(achievement.id.clone());
            }
        }

        // Now award all qualified achievements
        for achievement_id in achievements_to_award {
            self.award_achievement(user_id, achievement_id).await?;
        }

        Ok(())
    }

    fn calculate_level(&self, experience_points: u64) -> u32 {
        // Progressive XP requirements: 100, 300, 600, 1000, 1500, ...
        let mut level: u32 = 1;
        let mut total_xp_needed: u64 = 0;
        let mut xp_for_next_level: u64 = 100;

        while total_xp_needed + xp_for_next_level <= experience_points {
            total_xp_needed += xp_for_next_level;
            level += 1;
            xp_for_next_level = 100 * (level as u64); // Increasing XP requirement
        }

        level
    }

    fn broadcast_event(&self, event: ProfileEvent) {
        if let Err(e) = self.event_sender.send(event) {
            log::warn!("Failed to broadcast profile event: {}", e);
        }
    }

    async fn save_profile(&self, _user_id: &Uuid) -> RobinResult<()> {
        // TODO: Implement profile persistence
        Ok(())
    }

    async fn load_profiles(&mut self) -> RobinResult<()> {
        // TODO: Implement profile loading
        Ok(())
    }
}

/// User profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub user_id: Uuid,
    pub display_name: String,
    pub bio: String,
    pub avatar_url: Option<String>,
    pub social_links: HashMap<String, String>,
    pub join_date: SystemTime,
    pub last_active: SystemTime,
    pub last_updated: SystemTime,

    // Progression
    pub level: u32,
    pub experience_points: u64,
    pub achievements: HashSet<String>,
    pub badges: Vec<Badge>,
    pub reputation: i32,

    // Building statistics
    pub total_build_time: Duration,
    pub total_voxels_placed: u64,
    pub total_projects_created: u64,
    pub total_collaborations: u64,

    // Settings
    pub settings: UserSettings,
    pub privacy: PrivacySettings,

    // Detailed statistics
    pub statistics: UserStatistics,

    // Activity tracking
    pub activity_history: VecDeque<ActivityEntry>,

    // Featured content
    pub featured_creations: Vec<Uuid>,
}

/// Profile creation/update parameters
#[derive(Debug, Clone)]
pub struct ProfileParams {
    pub user_id: Uuid,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub social_links: Option<HashMap<String, String>>,
}

/// Public profile (respecting privacy settings)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicProfile {
    pub user_id: Uuid,
    pub display_name: String,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub join_date: SystemTime,
    pub level: u32,
    pub achievements: HashSet<String>,
    pub badges: Vec<Badge>,
    pub reputation: Option<i32>,
    pub statistics: Option<UserStatistics>,
    pub featured_creations: Vec<Uuid>,
}

/// User settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub language: String,
    pub timezone: String,
    pub notification_preferences: NotificationPreferences,
    pub ui_preferences: UiPreferences,
    pub gameplay_preferences: GameplayPreferences,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            timezone: "UTC".to_string(),
            notification_preferences: NotificationPreferences::default(),
            ui_preferences: UiPreferences::default(),
            gameplay_preferences: GameplayPreferences::default(),
        }
    }
}

/// Privacy settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySettings {
    pub profile_visibility: ProfileVisibility,
    pub show_online_status: bool,
    pub show_achievements: bool,
    pub show_badges: bool,
    pub show_statistics: bool,
    pub show_reputation: bool,
    pub show_on_leaderboards: bool,
    pub allow_friend_requests: bool,
    pub allow_collaboration_invites: bool,
}

impl Default for PrivacySettings {
    fn default() -> Self {
        Self {
            profile_visibility: ProfileVisibility::Public,
            show_online_status: true,
            show_achievements: true,
            show_badges: true,
            show_statistics: true,
            show_reputation: true,
            show_on_leaderboards: true,
            allow_friend_requests: true,
            allow_collaboration_invites: true,
        }
    }
}

/// Profile visibility levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProfileVisibility {
    Public,
    Friends,
    Private,
}

/// Notification preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub achievement_notifications: bool,
    pub friend_notifications: bool,
    pub collaboration_notifications: bool,
    pub gallery_notifications: bool,
    pub system_notifications: bool,
}

impl Default for NotificationPreferences {
    fn default() -> Self {
        Self {
            achievement_notifications: true,
            friend_notifications: true,
            collaboration_notifications: true,
            gallery_notifications: true,
            system_notifications: true,
        }
    }
}

/// UI preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPreferences {
    pub theme: String,
    pub ui_scale: f32,
    pub show_tooltips: bool,
    pub auto_save_interval: Duration,
}

impl Default for UiPreferences {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            ui_scale: 1.0,
            show_tooltips: true,
            auto_save_interval: Duration::from_secs(300), // 5 minutes
        }
    }
}

/// Gameplay preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameplayPreferences {
    pub default_build_mode: String,
    pub grid_snap: bool,
    pub show_coordinates: bool,
    pub camera_sensitivity: f32,
    pub movement_speed: f32,
}

impl Default for GameplayPreferences {
    fn default() -> Self {
        Self {
            default_build_mode: "Creative".to_string(),
            grid_snap: true,
            show_coordinates: false,
            camera_sensitivity: 1.0,
            movement_speed: 1.0,
        }
    }
}

/// User statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStatistics {
    pub blocks_placed: u64,
    pub blocks_removed: u64,
    pub projects_created: u64,
    pub projects_shared: u64,
    pub collaborations_started: u64,
    pub collaborations_joined: u64,
    pub rooms_created: u64,
    pub rooms_joined: u64,
    pub total_build_time: Duration,
    pub longest_build_session: Duration,
    pub gallery_submissions: u64,
    pub gallery_likes_received: u64,
    pub gallery_downloads: u64,
    pub friends_count: u64,
    pub achievements_earned: u64,
}

impl UserStatistics {
    pub fn new() -> Self {
        Self {
            blocks_placed: 0,
            blocks_removed: 0,
            projects_created: 0,
            projects_shared: 0,
            collaborations_started: 0,
            collaborations_joined: 0,
            rooms_created: 0,
            rooms_joined: 0,
            total_build_time: Duration::from_secs(0),
            longest_build_session: Duration::from_secs(0),
            gallery_submissions: 0,
            gallery_likes_received: 0,
            gallery_downloads: 0,
            friends_count: 0,
            achievements_earned: 0,
        }
    }
}

/// Activity entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEntry {
    pub activity_type: ActivityType,
    pub timestamp: SystemTime,
    pub metadata: HashMap<String, String>,
}

/// Activity types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivityType {
    Login,
    Logout,
    VoxelPlaced { count: u64, material: String },
    VoxelRemoved { count: u64, material: String },
    ProjectCreated { project_id: Uuid, name: String },
    ProjectShared { project_id: Uuid },
    BuildSession { duration: Duration },
    RoomJoined { room_id: Uuid, room_name: String },
    RoomCreated { room_id: Uuid, room_name: String },
    CollaborationStarted { session_id: Uuid },
    GallerySubmission { submission_id: Uuid },
    AchievementEarned { achievement_id: String },
    FriendAdded { friend_id: Uuid },
}

/// Badge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Badge {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon_url: String,
    pub rarity: BadgeRarity,
    pub earned_date: SystemTime,
}

/// Badge rarity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BadgeRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

/// Leaderboard criteria
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaderboardCriteria {
    Level,
    ExperiencePoints,
    BuildTime,
    VoxelsPlaced,
    ProjectsCreated,
    Reputation,
    Achievements,
}

/// Leaderboard entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub user_id: Uuid,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub rank: usize,
    pub score: f64,
}

/// User search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSearchResult {
    pub user_id: Uuid,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub level: u32,
    pub achievement_count: usize,
    pub reputation: i32,
}

/// Profile events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProfileEvent {
    ProfileCreated {
        user_id: Uuid,
        display_name: String,
    },
    ProfileUpdated {
        user_id: Uuid,
        display_name: String,
    },
    AchievementEarned {
        user_id: Uuid,
        achievement_id: String,
        achievement_name: String,
        experience_gained: u64,
        level_up: bool,
        new_level: u32,
    },
    LevelUp {
        user_id: Uuid,
        old_level: u32,
        new_level: u32,
    },
    BadgeEarned {
        user_id: Uuid,
        badge: Badge,
    },
}

/// Achievement system
pub struct AchievementSystem {
    achievements: HashMap<String, Achievement>,
}

impl AchievementSystem {
    pub fn new() -> Self {
        Self {
            achievements: HashMap::new(),
        }
    }

    pub async fn initialize(&mut self) -> RobinResult<()> {
        self.create_default_achievements();
        Ok(())
    }

    pub fn get_achievement(&self, id: &str) -> Option<&Achievement> {
        self.achievements.get(id)
    }

    pub fn get_all_achievements(&self) -> Vec<&Achievement> {
        self.achievements.values().collect()
    }

    pub fn check_criteria<'a>(&'a self, achievement: &'a Achievement, profile: &'a UserProfile) -> std::pin::Pin<Box<dyn std::future::Future<Output = RobinResult<bool>> + 'a>> {
        Box::pin(async move {
            match &achievement.criteria {
                AchievementCriteria::VoxelsPlaced(count) => {
                    Ok(profile.total_voxels_placed >= *count)
                },
                AchievementCriteria::ProjectsCreated(count) => {
                    Ok(profile.total_projects_created >= *count)
                },
                AchievementCriteria::BuildTime(duration) => {
                    Ok(profile.total_build_time >= *duration)
                },
                AchievementCriteria::Level(level) => {
                    Ok(profile.level >= *level)
                },
                AchievementCriteria::Collaborations(count) => {
                    Ok(profile.total_collaborations >= *count)
                },
                AchievementCriteria::Reputation(points) => {
                    Ok(profile.reputation >= *points)
                },
                AchievementCriteria::Multiple(criteria) => {
                    for criterion in criteria {
                        let temp_achievement = Achievement {
                            id: String::new(),
                            name: String::new(),
                            description: String::new(),
                            criteria: criterion.clone(),
                            experience_reward: 0,
                            badge: None,
                            category: AchievementCategory::Building,
                            rarity: AchievementRarity::Common,
                        };
                        if !self.check_criteria(&temp_achievement, profile).await? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                },
            }
        })
    }

    fn create_default_achievements(&mut self) {
        let achievements = vec![
            Achievement {
                id: "first_block".to_string(),
                name: "First Block".to_string(),
                description: "Place your first voxel block".to_string(),
                criteria: AchievementCriteria::VoxelsPlaced(1),
                experience_reward: 10,
                badge: Some(Badge {
                    id: "first_block_badge".to_string(),
                    name: "Builder".to_string(),
                    description: "Placed your first block".to_string(),
                    icon_url: "badges/first_block.png".to_string(),
                    rarity: BadgeRarity::Common,
                    earned_date: SystemTime::now(),
                }),
                category: AchievementCategory::Building,
                rarity: AchievementRarity::Common,
            },
            Achievement {
                id: "hundred_blocks".to_string(),
                name: "Block Builder".to_string(),
                description: "Place 100 voxel blocks".to_string(),
                criteria: AchievementCriteria::VoxelsPlaced(100),
                experience_reward: 50,
                badge: None,
                category: AchievementCategory::Building,
                rarity: AchievementRarity::Common,
            },
            Achievement {
                id: "thousand_blocks".to_string(),
                name: "Constructor".to_string(),
                description: "Place 1,000 voxel blocks".to_string(),
                criteria: AchievementCriteria::VoxelsPlaced(1000),
                experience_reward: 200,
                badge: Some(Badge {
                    id: "constructor_badge".to_string(),
                    name: "Constructor".to_string(),
                    description: "Built with 1,000+ blocks".to_string(),
                    icon_url: "badges/constructor.png".to_string(),
                    rarity: BadgeRarity::Uncommon,
                    earned_date: SystemTime::now(),
                }),
                category: AchievementCategory::Building,
                rarity: AchievementRarity::Uncommon,
            },
            Achievement {
                id: "first_project".to_string(),
                name: "Project Pioneer".to_string(),
                description: "Create your first project".to_string(),
                criteria: AchievementCriteria::ProjectsCreated(1),
                experience_reward: 25,
                badge: None,
                category: AchievementCategory::Creative,
                rarity: AchievementRarity::Common,
            },
            Achievement {
                id: "hour_builder".to_string(),
                name: "Dedicated Builder".to_string(),
                description: "Spend 1 hour building".to_string(),
                criteria: AchievementCriteria::BuildTime(Duration::from_secs(3600)),
                experience_reward: 100,
                badge: None,
                category: AchievementCategory::Building,
                rarity: AchievementRarity::Common,
            },
            Achievement {
                id: "collaborator".to_string(),
                name: "Team Player".to_string(),
                description: "Participate in 5 collaborations".to_string(),
                criteria: AchievementCriteria::Collaborations(5),
                experience_reward: 150,
                badge: Some(Badge {
                    id: "collaborator_badge".to_string(),
                    name: "Team Player".to_string(),
                    description: "Participated in 5+ collaborations".to_string(),
                    icon_url: "badges/team_player.png".to_string(),
                    rarity: BadgeRarity::Uncommon,
                    earned_date: SystemTime::now(),
                }),
                category: AchievementCategory::Social,
                rarity: AchievementRarity::Uncommon,
            },
            Achievement {
                id: "level_10".to_string(),
                name: "Rising Star".to_string(),
                description: "Reach level 10".to_string(),
                criteria: AchievementCriteria::Level(10),
                experience_reward: 500,
                badge: Some(Badge {
                    id: "rising_star_badge".to_string(),
                    name: "Rising Star".to_string(),
                    description: "Reached level 10".to_string(),
                    icon_url: "badges/rising_star.png".to_string(),
                    rarity: BadgeRarity::Rare,
                    earned_date: SystemTime::now(),
                }),
                category: AchievementCategory::Progression,
                rarity: AchievementRarity::Rare,
            },
        ];

        for achievement in achievements {
            self.achievements.insert(achievement.id.clone(), achievement);
        }
    }
}

/// Achievement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub id: String,
    pub name: String,
    pub description: String,
    pub criteria: AchievementCriteria,
    pub experience_reward: u64,
    pub badge: Option<Badge>,
    pub category: AchievementCategory,
    pub rarity: AchievementRarity,
}

/// Achievement criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AchievementCriteria {
    VoxelsPlaced(u64),
    ProjectsCreated(u64),
    BuildTime(Duration),
    Level(u32),
    Collaborations(u64),
    Reputation(i32),
    Multiple(Vec<AchievementCriteria>),
}

/// Achievement categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AchievementCategory {
    Building,
    Creative,
    Social,
    Progression,
    Special,
}

/// Achievement rarity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AchievementRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

/// Friend management system
pub struct FriendManager {
    friendships: HashMap<Uuid, HashSet<Uuid>>,
    friend_requests: HashMap<Uuid, Vec<FriendRequest>>,
}

impl FriendManager {
    pub fn new() -> Self {
        Self {
            friendships: HashMap::new(),
            friend_requests: HashMap::new(),
        }
    }

    pub fn are_friends(&self, user1: &Uuid, user2: &Uuid) -> bool {
        self.friendships.get(user1)
            .map(|friends| friends.contains(user2))
            .unwrap_or(false)
    }

    pub fn send_friend_request(&mut self, from: Uuid, to: Uuid) -> RobinResult<()> {
        if self.are_friends(&from, &to) {
            return Err(RobinError::Community("Already friends".to_string()));
        }

        let request = FriendRequest {
            from,
            to,
            timestamp: SystemTime::now(),
            status: FriendRequestStatus::Pending,
        };

        self.friend_requests.entry(to).or_insert_with(Vec::new).push(request);
        Ok(())
    }

    pub fn accept_friend_request(&mut self, user_id: Uuid, from: Uuid) -> RobinResult<()> {
        let requests = self.friend_requests.get_mut(&user_id)
            .ok_or_else(|| RobinError::Community("No friend requests found".to_string()))?;

        if let Some(request) = requests.iter_mut().find(|r| r.from == from) {
            request.status = FriendRequestStatus::Accepted;

            // Add friendship both ways
            self.friendships.entry(user_id).or_insert_with(HashSet::new).insert(from);
            self.friendships.entry(from).or_insert_with(HashSet::new).insert(user_id);

            Ok(())
        } else {
            Err(RobinError::Community("Friend request not found".to_string()))
        }
    }
}

/// Friend request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendRequest {
    pub from: Uuid,
    pub to: Uuid,
    pub timestamp: SystemTime,
    pub status: FriendRequestStatus,
}

/// Friend request status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FriendRequestStatus {
    Pending,
    Accepted,
    Rejected,
}

/// Activity tracking system
pub struct ActivityTracker {
    recent_activities: HashMap<Uuid, VecDeque<ActivityEntry>>,
}

impl ActivityTracker {
    pub fn new() -> Self {
        Self {
            recent_activities: HashMap::new(),
        }
    }

    pub async fn track_activity(&mut self, user_id: Uuid, activity: ActivityType) -> RobinResult<()> {
        let entry = ActivityEntry {
            activity_type: activity,
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
        };

        let activities = self.recent_activities.entry(user_id).or_insert_with(|| VecDeque::with_capacity(100));
        activities.push_back(entry);

        // Keep only recent activities
        if activities.len() > 100 {
            activities.pop_front();
        }

        Ok(())
    }
}

/// Profile configuration
#[derive(Debug, Clone)]
pub struct ProfileConfig {
    pub max_bio_length: usize,
    pub max_display_name_length: usize,
    pub max_social_links: usize,
    pub max_featured_creations: usize,
    pub activity_history_limit: usize,
}

impl Default for ProfileConfig {
    fn default() -> Self {
        Self {
            max_bio_length: 500,
            max_display_name_length: 50,
            max_social_links: 10,
            max_featured_creations: 5,
            activity_history_limit: 1000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_profile_creation() {
        let mut manager = UserProfileManager::new().unwrap();
        manager.initialize().await.unwrap();

        let user_id = Uuid::new_v4();
        let params = ProfileParams {
            user_id,
            display_name: Some("Test User".to_string()),
            bio: Some("A test user profile".to_string()),
            avatar_url: None,
            social_links: None,
        };

        let created_id = manager.create_or_update_profile(params).await.unwrap();
        assert_eq!(created_id, user_id);

        let profile = manager.get_profile(&user_id).unwrap();
        assert_eq!(profile.display_name, "Test User");
        assert_eq!(profile.bio, "A test user profile");
        assert_eq!(profile.level, 1);
        assert_eq!(profile.experience_points, 0);
    }

    #[tokio::test]
    async fn test_achievement_system() {
        let mut manager = UserProfileManager::new().unwrap();
        manager.initialize().await.unwrap();

        let user_id = Uuid::new_v4();
        let params = ProfileParams {
            user_id,
            display_name: Some("Test User".to_string()),
            bio: None,
            avatar_url: None,
            social_links: None,
        };

        manager.create_or_update_profile(params).await.unwrap();

        // Award first block achievement
        manager.award_achievement(user_id, "first_block".to_string()).await.unwrap();

        let profile = manager.get_profile(&user_id).unwrap();
        assert!(profile.achievements.contains("first_block"));
        assert_eq!(profile.experience_points, 10);

        let achievements = manager.get_user_achievements(&user_id);
        assert_eq!(achievements.len(), 1);
        assert_eq!(achievements[0].name, "First Block");
    }

    #[tokio::test]
    async fn test_activity_tracking() {
        let mut manager = UserProfileManager::new().unwrap();
        manager.initialize().await.unwrap();

        let user_id = Uuid::new_v4();
        let params = ProfileParams {
            user_id,
            display_name: Some("Builder".to_string()),
            bio: None,
            avatar_url: None,
            social_links: None,
        };

        manager.create_or_update_profile(params).await.unwrap();

        // Track voxel placement activity
        let activity = ActivityType::VoxelPlaced {
            count: 5,
            material: "Stone".to_string(),
        };

        manager.update_activity(user_id, activity).await.unwrap();

        let profile = manager.get_profile(&user_id).unwrap();
        assert_eq!(profile.total_voxels_placed, 5);
        assert_eq!(profile.statistics.blocks_placed, 5);
        assert_eq!(profile.activity_history.len(), 1);
    }

    #[test]
    fn test_level_calculation() {
        let manager = UserProfileManager::new().unwrap();

        assert_eq!(manager.calculate_level(0), 1);
        assert_eq!(manager.calculate_level(50), 1);
        assert_eq!(manager.calculate_level(100), 2);
        assert_eq!(manager.calculate_level(400), 3);
        assert_eq!(manager.calculate_level(1000), 4);
    }
}