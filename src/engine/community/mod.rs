// Robin Engine Community Framework
// Social building tools and collaborative features

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use tokio::sync::broadcast;

use crate::engine::{
    error::{RobinResult, RobinError},
    math::{Vec3, Transform},
    generation::voxel_system::VoxelWorld,
    platform::cloud_saves::SaveData,
    save_system::SaveManager,
};

pub mod project_sharing;
pub mod social_rooms;
pub mod community_gallery;
pub mod user_profiles;
pub mod collaboration;

// Re-exports for easy access
pub use project_sharing::*;
pub use social_rooms::*;
pub use community_gallery::*;
pub use user_profiles::*;
pub use collaboration::*;

/// Central community manager for Robin Engine
pub struct CommunityManager {
    /// Project sharing system
    project_manager: Arc<RwLock<ProjectSharingManager>>,

    /// Social building rooms
    room_manager: Arc<RwLock<SocialRoomManager>>,

    /// Community gallery
    gallery: Arc<RwLock<CommunityGallery>>,

    /// User profile system
    profile_manager: Arc<RwLock<UserProfileManager>>,

    /// Real-time collaboration
    collaboration: Arc<RwLock<CollaborationEngine>>,

    /// Event broadcasting
    event_sender: broadcast::Sender<CommunityEvent>,

    /// Community statistics
    stats: CommunityStats,
}

impl CommunityManager {
    /// Create a new community manager
    pub fn new() -> RobinResult<Self> {
        let (event_sender, _) = broadcast::channel(1000);

        Ok(Self {
            project_manager: Arc::new(RwLock::new(ProjectSharingManager::new()?)),
            room_manager: Arc::new(RwLock::new(SocialRoomManager::new()?)),
            gallery: Arc::new(RwLock::new(CommunityGallery::new()?)),
            profile_manager: Arc::new(RwLock::new(UserProfileManager::new()?)),
            collaboration: Arc::new(RwLock::new(CollaborationEngine::new()?)),
            event_sender,
            stats: CommunityStats::new(),
        })
    }

    /// Initialize community features
    pub async fn initialize(&mut self) -> RobinResult<()> {
        log::info!("🌍 Initializing Robin Engine Community Features");

        // Initialize all subsystems
        {
            let mut project_manager = self.project_manager.write().unwrap();
            project_manager.initialize().await?;
        }

        {
            let mut room_manager = self.room_manager.write().unwrap();
            room_manager.initialize().await?;
        }

        {
            let mut gallery = self.gallery.write().unwrap();
            gallery.initialize().await?;
        }

        {
            let mut profile_manager = self.profile_manager.write().unwrap();
            profile_manager.initialize().await?;
        }

        {
            let mut collaboration = self.collaboration.write().unwrap();
            collaboration.initialize().await?;
        }

        log::info!("✅ Community features initialized successfully");
        Ok(())
    }

    /// Get project sharing manager
    pub fn project_manager(&self) -> Arc<RwLock<ProjectSharingManager>> {
        Arc::clone(&self.project_manager)
    }

    /// Get social room manager
    pub fn room_manager(&self) -> Arc<RwLock<SocialRoomManager>> {
        Arc::clone(&self.room_manager)
    }

    /// Get community gallery
    pub fn gallery(&self) -> Arc<RwLock<CommunityGallery>> {
        Arc::clone(&self.gallery)
    }

    /// Get user profile manager
    pub fn profile_manager(&self) -> Arc<RwLock<UserProfileManager>> {
        Arc::clone(&self.profile_manager)
    }

    /// Get collaboration engine
    pub fn collaboration(&self) -> Arc<RwLock<CollaborationEngine>> {
        Arc::clone(&self.collaboration)
    }

    /// Subscribe to community events
    pub fn subscribe_events(&self) -> broadcast::Receiver<CommunityEvent> {
        self.event_sender.subscribe()
    }

    /// Broadcast a community event
    pub fn broadcast_event(&self, event: CommunityEvent) -> RobinResult<()> {
        self.event_sender.send(event)
            .map_err(|e| RobinError::Community(format!("Failed to broadcast event: {}", e)))?;
        Ok(())
    }

    /// Get community statistics
    pub fn get_stats(&self) -> &CommunityStats {
        &self.stats
    }

    /// Update community statistics
    pub fn update_stats(&mut self) -> RobinResult<()> {
        // Collect stats from all subsystems
        let project_count = {
            let project_manager = self.project_manager.read().unwrap();
            project_manager.get_project_count()
        };

        let active_users = {
            let room_manager = self.room_manager.read().unwrap();
            room_manager.get_active_user_count()
        };

        let gallery_items = {
            let gallery = self.gallery.read().unwrap();
            gallery.get_item_count()
        };

        let total_users = {
            let profile_manager = self.profile_manager.read().unwrap();
            profile_manager.get_user_count()
        };

        self.stats.update(project_count, active_users, gallery_items, total_users);
        Ok(())
    }
}

impl Default for CommunityManager {
    fn default() -> Self {
        Self::new().expect("Failed to create CommunityManager")
    }
}

/// Community event types for broadcasting updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommunityEvent {
    /// New project shared
    ProjectShared {
        project_id: Uuid,
        user_id: Uuid,
        title: String,
    },

    /// User joined a social room
    UserJoinedRoom {
        user_id: Uuid,
        room_id: Uuid,
        room_name: String,
    },

    /// User left a social room
    UserLeftRoom {
        user_id: Uuid,
        room_id: Uuid,
    },

    /// New gallery submission
    GallerySubmission {
        submission_id: Uuid,
        user_id: Uuid,
        title: String,
        category: String,
    },

    /// Achievement unlocked
    AchievementUnlocked {
        user_id: Uuid,
        achievement_id: String,
        achievement_name: String,
    },

    /// Collaboration started
    CollaborationStarted {
        session_id: Uuid,
        initiator_id: Uuid,
        project_name: String,
    },

    /// Collaboration ended
    CollaborationEnded {
        session_id: Uuid,
        duration: Duration,
        participants: Vec<Uuid>,
    },

    /// Community milestone reached
    CommunityMilestone {
        milestone_type: String,
        value: u64,
        description: String,
    },
}

/// Community statistics and metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityStats {
    /// Total number of shared projects
    pub total_projects: u64,

    /// Currently active users
    pub active_users: u64,

    /// Total gallery submissions
    pub gallery_items: u64,

    /// Total registered users
    pub total_users: u64,

    /// Active collaboration sessions
    pub active_collaborations: u64,

    /// Last updated timestamp
    pub last_updated: SystemTime,

    /// Daily engagement metrics
    pub daily_metrics: DailyMetrics,
}

impl CommunityStats {
    pub fn new() -> Self {
        Self {
            total_projects: 0,
            active_users: 0,
            gallery_items: 0,
            total_users: 0,
            active_collaborations: 0,
            last_updated: SystemTime::now(),
            daily_metrics: DailyMetrics::new(),
        }
    }

    pub fn update(&mut self, projects: u64, active: u64, gallery: u64, users: u64) {
        self.total_projects = projects;
        self.active_users = active;
        self.gallery_items = gallery;
        self.total_users = users;
        self.last_updated = SystemTime::now();

        // Update daily metrics
        self.daily_metrics.update_engagement(active);
    }
}

/// Daily engagement metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyMetrics {
    /// Peak concurrent users today
    pub peak_users: u64,

    /// Total unique users today
    pub unique_users: u64,

    /// Projects created today
    pub projects_created: u64,

    /// Collaboration sessions today
    pub collaborations: u64,

    /// Gallery submissions today
    pub gallery_submissions: u64,

    /// Date of these metrics
    pub date: SystemTime,
}

impl DailyMetrics {
    pub fn new() -> Self {
        Self {
            peak_users: 0,
            unique_users: 0,
            projects_created: 0,
            collaborations: 0,
            gallery_submissions: 0,
            date: SystemTime::now(),
        }
    }

    pub fn update_engagement(&mut self, current_users: u64) {
        if current_users > self.peak_users {
            self.peak_users = current_users;
        }
    }

    pub fn increment_projects(&mut self) {
        self.projects_created += 1;
    }

    pub fn increment_collaborations(&mut self) {
        self.collaborations += 1;
    }

    pub fn increment_gallery(&mut self) {
        self.gallery_submissions += 1;
    }
}

/// Community configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityConfig {
    /// Maximum users per social room
    pub max_room_users: usize,

    /// Maximum number of projects per user
    pub max_projects_per_user: usize,

    /// Gallery submission cooldown period
    pub gallery_cooldown: Duration,

    /// Collaboration session timeout
    pub collaboration_timeout: Duration,

    /// Enable community features
    pub enable_community: bool,

    /// Enable project sharing
    pub enable_project_sharing: bool,

    /// Enable social rooms
    pub enable_social_rooms: bool,

    /// Enable community gallery
    pub enable_gallery: bool,

    /// Enable user profiles
    pub enable_profiles: bool,

    /// Enable real-time collaboration
    pub enable_collaboration: bool,
}

impl Default for CommunityConfig {
    fn default() -> Self {
        Self {
            max_room_users: 50,
            max_projects_per_user: 100,
            gallery_cooldown: Duration::from_secs(3600), // 1 hour
            collaboration_timeout: Duration::from_secs(7200), // 2 hours
            enable_community: true,
            enable_project_sharing: true,
            enable_social_rooms: true,
            enable_gallery: true,
            enable_profiles: true,
            enable_collaboration: true,
        }
    }
}

/// Community feature traits for extensibility
pub trait CommunityFeature {
    /// Initialize the feature
    async fn initialize(&mut self) -> RobinResult<()>;

    /// Shutdown the feature
    async fn shutdown(&mut self) -> RobinResult<()>;

    /// Get feature name
    fn name(&self) -> &str;

    /// Check if feature is enabled
    fn is_enabled(&self) -> bool;
}

/// Community data persistence trait
pub trait CommunityDataStore {
    /// Save community data
    async fn save_data(&self, data: &SaveData) -> RobinResult<()>;

    /// Load community data
    async fn load_data(&self) -> RobinResult<SaveData>;

    /// Delete community data
    async fn delete_data(&self, id: &Uuid) -> RobinResult<()>;

    /// List available data
    async fn list_data(&self) -> RobinResult<Vec<Uuid>>;
}

/// Community moderation tools
pub struct ModerationTools {
    /// Banned users
    banned_users: HashMap<Uuid, BanInfo>,

    /// Flagged content
    flagged_content: Vec<FlaggedContent>,

    /// Moderation logs
    mod_logs: VecDeque<ModerationLog>,
}

impl ModerationTools {
    pub fn new() -> Self {
        Self {
            banned_users: HashMap::new(),
            flagged_content: Vec::new(),
            mod_logs: VecDeque::with_capacity(1000),
        }
    }

    /// Ban a user
    pub fn ban_user(&mut self, user_id: Uuid, reason: String, duration: Option<Duration>) -> RobinResult<()> {
        let ban_info = BanInfo {
            user_id,
            reason: reason.clone(),
            banned_at: SystemTime::now(),
            duration,
            active: true,
        };

        self.banned_users.insert(user_id, ban_info);

        let log = ModerationLog {
            action: ModerationAction::UserBanned,
            target_id: user_id,
            reason,
            timestamp: SystemTime::now(),
            moderator: None, // TODO: Add moderator tracking
        };

        self.mod_logs.push_back(log);

        // Keep only last 1000 logs
        if self.mod_logs.len() > 1000 {
            self.mod_logs.pop_front();
        }

        Ok(())
    }

    /// Check if user is banned
    pub fn is_user_banned(&self, user_id: &Uuid) -> bool {
        if let Some(ban_info) = self.banned_users.get(user_id) {
            if !ban_info.active {
                return false;
            }

            // Check if temporary ban has expired
            if let Some(duration) = ban_info.duration {
                if let Ok(elapsed) = ban_info.banned_at.elapsed() {
                    return elapsed < duration;
                }
            }

            true
        } else {
            false
        }
    }

    /// Flag content for review
    pub fn flag_content(&mut self, content_id: Uuid, content_type: ContentType, reason: String, reporter: Uuid) {
        let flagged = FlaggedContent {
            content_id,
            content_type,
            reason,
            reporter,
            flagged_at: SystemTime::now(),
            reviewed: false,
        };

        self.flagged_content.push(flagged);
    }
}

/// User ban information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BanInfo {
    pub user_id: Uuid,
    pub reason: String,
    pub banned_at: SystemTime,
    pub duration: Option<Duration>, // None for permanent ban
    pub active: bool,
}

/// Flagged content for moderation review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlaggedContent {
    pub content_id: Uuid,
    pub content_type: ContentType,
    pub reason: String,
    pub reporter: Uuid,
    pub flagged_at: SystemTime,
    pub reviewed: bool,
}

/// Content types that can be flagged
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentType {
    Project,
    GallerySubmission,
    UserProfile,
    ChatMessage,
    Comment,
}

/// Moderation log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModerationLog {
    pub action: ModerationAction,
    pub target_id: Uuid,
    pub reason: String,
    pub timestamp: SystemTime,
    pub moderator: Option<Uuid>,
}

/// Types of moderation actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModerationAction {
    UserBanned,
    UserUnbanned,
    ContentRemoved,
    ContentFlagged,
    ContentApproved,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_community_manager_creation() {
        let result = CommunityManager::new();
        assert!(result.is_ok());
    }

    #[test]
    fn test_community_stats() {
        let mut stats = CommunityStats::new();
        stats.update(10, 5, 20, 100);

        assert_eq!(stats.total_projects, 10);
        assert_eq!(stats.active_users, 5);
        assert_eq!(stats.gallery_items, 20);
        assert_eq!(stats.total_users, 100);
    }

    #[test]
    fn test_moderation_tools() {
        let mut mod_tools = ModerationTools::new();
        let user_id = Uuid::new_v4();

        assert!(!mod_tools.is_user_banned(&user_id));

        mod_tools.ban_user(user_id, "Test ban".to_string(), None).unwrap();
        assert!(mod_tools.is_user_banned(&user_id));
    }

    #[test]
    fn test_daily_metrics() {
        let mut metrics = DailyMetrics::new();

        metrics.update_engagement(10);
        assert_eq!(metrics.peak_users, 10);

        metrics.update_engagement(5);
        assert_eq!(metrics.peak_users, 10); // Should remain at peak

        metrics.update_engagement(15);
        assert_eq!(metrics.peak_users, 15); // Should update to new peak
    }
}