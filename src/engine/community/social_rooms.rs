// Robin Engine Social Building Rooms
// Shared virtual spaces for collaborative building

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use tokio::sync::broadcast;

use crate::engine::{
    error::{RobinResult, RobinError},
    math::{Vec3, Transform},
    generation::voxel_system::{VoxelWorld, VoxelType as VoxelSystemType},
    collaboration::version_control::VoxelChange,
    world::construction::MaterialType,
    platform::cloud_saves::SaveData,
    save_system::SaveManager,
};

/// Social room manager for collaborative building spaces
pub struct SocialRoomManager {
    /// Active rooms
    rooms: HashMap<Uuid, SocialRoom>,

    /// Room categories and templates
    categories: HashMap<String, RoomCategory>,

    /// User sessions and presence
    user_sessions: HashMap<Uuid, UserSession>,

    /// Room search index
    search_index: RoomSearchIndex,

    /// Event broadcasting
    event_sender: broadcast::Sender<RoomEvent>,

    /// Configuration
    config: SocialRoomConfig,

    /// Room persistence
    storage_path: std::path::PathBuf,

    /// Feature enabled
    enabled: bool,
}

impl SocialRoomManager {
    /// Create a new social room manager
    pub fn new() -> RobinResult<Self> {
        let (event_sender, _) = broadcast::channel(1000);

        Ok(Self {
            rooms: HashMap::new(),
            categories: Self::create_default_categories(),
            user_sessions: HashMap::new(),
            search_index: RoomSearchIndex::new(),
            event_sender,
            config: SocialRoomConfig::default(),
            storage_path: std::path::PathBuf::from("data/social_rooms"),
            enabled: true,
        })
    }

    /// Initialize the social room system
    pub async fn initialize(&mut self) -> RobinResult<()> {
        if !self.enabled {
            return Ok(());
        }

        log::info!("🏠 Initializing Social Room Manager");

        // Create storage directory
        std::fs::create_dir_all(&self.storage_path)
            .map_err(|e| RobinError::Community(format!("Failed to create rooms directory: {}", e)))?;

        // Load existing rooms
        self.load_rooms().await?;

        // Create default lobby room
        if self.rooms.is_empty() {
            self.create_default_lobby().await?;
        }

        log::info!("✅ Social Room Manager initialized with {} rooms", self.rooms.len());
        Ok(())
    }

    /// Create a new social room
    pub async fn create_room(&mut self, creator_id: Uuid, params: CreateRoomParams) -> RobinResult<Uuid> {
        if !self.enabled {
            return Err(RobinError::Community("Social rooms disabled".to_string()));
        }

        // Validate parameters
        self.validate_room_params(&params)?;

        // Check user permissions
        self.check_user_can_create_room(&creator_id)?;

        let room_id = Uuid::new_v4();
        let now = SystemTime::now();

        // Create voxel world for the room
        let world_size = params.world_size.unwrap_or([64, 64, 64]);
        let world = VoxelWorld::new(
            params.name.clone(),
            (world_size[0] as usize, world_size[1] as usize, world_size[2] as usize)
        );

        let room = SocialRoom {
            id: room_id,
            name: params.name,
            description: params.description.unwrap_or_default(),
            category: params.category,
            creator_id,
            max_users: params.max_users.unwrap_or(self.config.default_max_users),
            is_public: params.is_public.unwrap_or(true),
            password: params.password,
            tags: params.tags.unwrap_or_default(),
            world: Arc::new(RwLock::new(world)),
            connected_users: HashSet::new(),
            creation_time: now,
            last_activity: now,
            total_joins: 0,
            room_settings: RoomSettings::default(),
            moderation: RoomModeration::new(creator_id),
            build_permissions: BuildPermissions::default(),
            chat_history: VecDeque::with_capacity(self.config.max_chat_history),
            room_state: RoomState::Active,
        };

        // Add to rooms
        self.rooms.insert(room_id, room);

        // Update search index
        self.search_index.add_room(&self.rooms[&room_id]);

        // Save room
        self.save_room(&room_id).await?;

        // Broadcast event
        let event = RoomEvent::RoomCreated {
            room_id,
            creator_id,
            room_name: self.rooms[&room_id].name.clone(),
            category: self.rooms[&room_id].category.clone(),
        };
        self.broadcast_event(event);

        log::info!("🏠 Created room '{}' ({})", self.rooms[&room_id].name, room_id);
        Ok(room_id)
    }

    /// Join a social room
    pub async fn join_room(&mut self, user_id: Uuid, room_id: Uuid, password: Option<String>) -> RobinResult<UserSession> {
        if !self.enabled {
            return Err(RobinError::Community("Social rooms disabled".to_string()));
        }

        // Get room
        let room = self.rooms.get_mut(&room_id)
            .ok_or_else(|| RobinError::Community("Room not found".to_string()))?;

        // Check if room is active
        if room.room_state != RoomState::Active {
            return Err(RobinError::Community("Room is not active".to_string()));
        }

        // Check password if required
        if let Some(ref room_password) = room.password {
            match password {
                Some(ref provided_password) if provided_password == room_password => {},
                _ => return Err(RobinError::Community("Invalid room password".to_string())),
            }
        }

        // Check capacity
        if room.connected_users.len() >= room.max_users {
            return Err(RobinError::Community("Room is full".to_string()));
        }

        // Check if user is banned
        if room.moderation.is_user_banned(&user_id) {
            return Err(RobinError::Community("User is banned from this room".to_string()));
        }

        // Calculate user permissions inline to avoid borrow conflicts
        let permissions = if user_id == room.creator_id {
            UserPermissions::Owner
        } else if room.moderation.is_moderator(&user_id) {
            UserPermissions::Moderator
        } else {
            UserPermissions::Builder
        };

        // Create user session
        let session = UserSession {
            user_id,
            room_id,
            join_time: SystemTime::now(),
            last_activity: SystemTime::now(),
            position: Vec3::new(0.0, 10.0, 0.0), // Spawn above ground
            rotation: 0.0,
            permissions,
            is_building: false,
            cursor_position: None,
        };

        // Add user to room
        room.connected_users.insert(user_id);
        room.total_joins += 1;
        room.last_activity = SystemTime::now();

        // Collect data for event before dropping mutable borrow
        let room_name = room.name.clone();
        let user_count = room.connected_users.len();

        // Store session
        self.user_sessions.insert(user_id, session.clone());

        // Drop the mutable borrow by ending the scope
        drop(room);

        // Broadcast event (now we can borrow self immutably)
        let event = RoomEvent::UserJoined {
            user_id,
            room_id,
            room_name: room_name.clone(),
            user_count,
        };
        self.broadcast_event(event);

        log::info!("👤 User {} joined room '{}'", user_id, room_name);
        Ok(session)
    }

    /// Leave a social room
    pub async fn leave_room(&mut self, user_id: Uuid) -> RobinResult<()> {
        let session = self.user_sessions.remove(&user_id)
            .ok_or_else(|| RobinError::Community("User not in any room".to_string()))?;

        let room_id = session.room_id;

        // Remove from room
        if let Some(room) = self.rooms.get_mut(&room_id) {
            room.connected_users.remove(&user_id);
            room.last_activity = SystemTime::now();

            // Collect data for event before dropping mutable borrow
            let user_count = room.connected_users.len();
            let room_name = room.name.clone();

            // Drop the mutable borrow by ending the scope
            drop(room);

            // Broadcast event (now we can borrow self immutably)
            let event = RoomEvent::UserLeft {
                user_id,
                room_id,
                user_count,
            };
            self.broadcast_event(event);

            log::info!("👋 User {} left room '{}'", user_id, room_name);
        }

        Ok(())
    }

    /// List available rooms
    pub fn list_rooms(&self, filter: RoomListFilter) -> Vec<RoomSummary> {
        self.rooms.values()
            .filter(|room| self.room_matches_filter(room, &filter))
            .map(|room| RoomSummary {
                id: room.id,
                name: room.name.clone(),
                description: room.description.clone(),
                category: room.category.clone(),
                user_count: room.connected_users.len(),
                max_users: room.max_users,
                is_public: room.is_public,
                has_password: room.password.is_some(),
                tags: room.tags.clone(),
                creation_time: room.creation_time,
                last_activity: room.last_activity,
                total_joins: room.total_joins,
            })
            .collect()
    }

    /// Search rooms
    pub fn search_rooms(&self, query: &str, limit: usize) -> Vec<RoomSummary> {
        self.search_index.search(query, limit)
            .into_iter()
            .filter_map(|room_id| self.rooms.get(&room_id))
            .map(|room| RoomSummary {
                id: room.id,
                name: room.name.clone(),
                description: room.description.clone(),
                category: room.category.clone(),
                user_count: room.connected_users.len(),
                max_users: room.max_users,
                is_public: room.is_public,
                has_password: room.password.is_some(),
                tags: room.tags.clone(),
                creation_time: room.creation_time,
                last_activity: room.last_activity,
                total_joins: room.total_joins,
            })
            .collect()
    }

    /// Handle voxel changes in a room
    pub async fn handle_voxel_change(&mut self, user_id: Uuid, change: VoxelChange) -> RobinResult<()> {
        let session = self.user_sessions.get_mut(&user_id)
            .ok_or_else(|| RobinError::Community("User not in any room".to_string()))?;

        let room_id = session.room_id;
        let room = self.rooms.get_mut(&room_id)
            .ok_or_else(|| RobinError::Community("Room not found".to_string()))?;

        // Check build permissions
        if !self.user_can_build(user_id, room, &change)? {
            return Err(RobinError::Community("Insufficient build permissions".to_string()));
        }

        // Apply change to world
        {
            let mut world = room.world.write().unwrap();
            // Apply voxel change based on the change type
            if let Some(new_type) = change.new_type {
                // Convert construction::VoxelType to voxel_system::VoxelType
                let voxel_system_type = match new_type {
                    crate::engine::world::VoxelType::Air => VoxelSystemType::Air,
                    crate::engine::world::VoxelType::Stone => VoxelSystemType::Stone,
                    crate::engine::world::VoxelType::Dirt => VoxelSystemType::Solid,
                    crate::engine::world::VoxelType::Grass => VoxelSystemType::Solid,
                    crate::engine::world::VoxelType::Sand => VoxelSystemType::Solid,
                    crate::engine::world::VoxelType::Water => VoxelSystemType::Liquid,
                    crate::engine::world::VoxelType::Wood => VoxelSystemType::Wood,
                    crate::engine::world::VoxelType::Leaves => VoxelSystemType::Solid,
                    crate::engine::world::VoxelType::Crystal => VoxelSystemType::Glass,
                    crate::engine::world::VoxelType::Lava => VoxelSystemType::Liquid,
                    crate::engine::world::VoxelType::Glass => VoxelSystemType::Glass,
                    crate::engine::world::VoxelType::Metal => VoxelSystemType::Metal,
                    crate::engine::world::VoxelType::Brick => VoxelSystemType::Concrete,
                    crate::engine::world::VoxelType::Ice => VoxelSystemType::Solid,
                    crate::engine::world::VoxelType::Obsidian => VoxelSystemType::Stone,
                };
                world.set_voxel(change.position, voxel_system_type);
            }
        }

        // Update activity
        session.last_activity = SystemTime::now();
        room.last_activity = SystemTime::now();

        // Broadcast voxel change to other users in room
        let event = RoomEvent::VoxelChanged {
            room_id,
            user_id,
            change: change.clone(),
        };
        self.broadcast_event(event);

        log::debug!("🧱 Voxel change applied in room {} by user {}", room_id, user_id);
        Ok(())
    }

    /// Send chat message in room
    pub async fn send_chat_message(&mut self, user_id: Uuid, message: String) -> RobinResult<()> {
        let session = self.user_sessions.get(&user_id)
            .ok_or_else(|| RobinError::Community("User not in any room".to_string()))?;

        let room_id = session.room_id;
        let room = self.rooms.get_mut(&room_id)
            .ok_or_else(|| RobinError::Community("Room not found".to_string()))?;

        // Check if user is muted
        if room.moderation.is_user_muted(&user_id) {
            return Err(RobinError::Community("User is muted in this room".to_string()));
        }

        // Create chat message
        let chat_message = ChatMessage {
            id: Uuid::new_v4(),
            user_id,
            content: message.clone(),
            timestamp: SystemTime::now(),
            message_type: MessageType::Chat,
        };

        // Add to chat history
        room.chat_history.push_back(chat_message.clone());

        // Keep chat history within limits
        if room.chat_history.len() > self.config.max_chat_history {
            room.chat_history.pop_front();
        }

        // Broadcast chat message
        let event = RoomEvent::ChatMessage {
            room_id,
            message: chat_message,
        };
        self.broadcast_event(event);

        Ok(())
    }

    /// Get active user count
    pub fn get_active_user_count(&self) -> u64 {
        self.user_sessions.len() as u64
    }

    /// Get room count
    pub fn get_room_count(&self) -> u64 {
        self.rooms.len() as u64
    }

    /// Update user position in room
    pub fn update_user_position(&mut self, user_id: Uuid, position: Vec3, rotation: f32) -> RobinResult<()> {
        let session = self.user_sessions.get_mut(&user_id)
            .ok_or_else(|| RobinError::Community("User not in any room".to_string()))?;

        session.position = position;
        session.rotation = rotation;
        session.last_activity = SystemTime::now();

        // Broadcast position update
        let event = RoomEvent::UserMoved {
            user_id,
            room_id: session.room_id,
            position,
            rotation,
        };
        self.broadcast_event(event);

        Ok(())
    }

    // Helper methods

    fn create_default_categories() -> HashMap<String, RoomCategory> {
        let mut categories = HashMap::new();

        categories.insert("General".to_string(), RoomCategory {
            name: "General".to_string(),
            description: "General building and socializing".to_string(),
            icon: "🏠".to_string(),
            color: "#4A90E2".to_string(),
        });

        categories.insert("Creative".to_string(), RoomCategory {
            name: "Creative".to_string(),
            description: "Creative building projects".to_string(),
            icon: "🎨".to_string(),
            color: "#F5A623".to_string(),
        });

        categories.insert("Collaborative".to_string(), RoomCategory {
            name: "Collaborative".to_string(),
            description: "Team building projects".to_string(),
            icon: "👥".to_string(),
            color: "#7ED321".to_string(),
        });

        categories.insert("Learning".to_string(), RoomCategory {
            name: "Learning".to_string(),
            description: "Educational building exercises".to_string(),
            icon: "📚".to_string(),
            color: "#9013FE".to_string(),
        });

        categories
    }

    async fn create_default_lobby(&mut self) -> RobinResult<()> {
        let creator_id = Uuid::new_v4(); // System user

        let params = CreateRoomParams {
            name: "Main Lobby".to_string(),
            description: Some("Welcome to Robin Engine! Start building together.".to_string()),
            category: "General".to_string(),
            max_users: Some(100),
            is_public: Some(true),
            password: None,
            tags: Some(vec!["lobby".to_string(), "welcome".to_string()]),
            world_size: Some([128, 64, 128]), // Larger world for lobby
        };

        self.create_room(creator_id, params).await?;
        Ok(())
    }

    fn validate_room_params(&self, params: &CreateRoomParams) -> RobinResult<()> {
        if params.name.trim().is_empty() {
            return Err(RobinError::Community("Room name cannot be empty".to_string()));
        }

        if params.name.len() > 100 {
            return Err(RobinError::Community("Room name too long".to_string()));
        }

        if !self.categories.contains_key(&params.category) {
            return Err(RobinError::Community("Invalid room category".to_string()));
        }

        if let Some(max_users) = params.max_users {
            if max_users > self.config.absolute_max_users {
                return Err(RobinError::Community("Max users exceeds limit".to_string()));
            }
        }

        Ok(())
    }

    fn check_user_can_create_room(&self, _user_id: &Uuid) -> RobinResult<()> {
        // TODO: Check user permissions, rate limits, etc.
        Ok(())
    }

    fn get_user_permissions(&self, user_id: Uuid, room: &SocialRoom) -> UserPermissions {
        if user_id == room.creator_id {
            UserPermissions::Owner
        } else if room.moderation.is_moderator(&user_id) {
            UserPermissions::Moderator
        } else {
            UserPermissions::Builder
        }
    }

    fn user_can_build(&self, user_id: Uuid, room: &SocialRoom, _change: &VoxelChange) -> RobinResult<bool> {
        if room.moderation.is_user_banned(&user_id) {
            return Ok(false);
        }

        let permissions = self.get_user_permissions(user_id, room);

        match permissions {
            UserPermissions::Owner | UserPermissions::Moderator => Ok(true),
            UserPermissions::Builder => {
                // Check build permissions based on room settings
                Ok(room.build_permissions.allow_all_users)
            },
            UserPermissions::Visitor => Ok(false),
        }
    }

    fn room_matches_filter(&self, room: &SocialRoom, filter: &RoomListFilter) -> bool {
        if !room.is_public && filter.include_private != Some(true) {
            return false;
        }

        if let Some(ref category) = filter.category {
            if room.category != *category {
                return false;
            }
        }

        if let Some(max_users) = filter.max_users {
            if room.connected_users.len() > max_users {
                return false;
            }
        }

        true
    }

    fn broadcast_event(&self, event: RoomEvent) {
        if let Err(e) = self.event_sender.send(event) {
            log::warn!("Failed to broadcast room event: {}", e);
        }
    }

    async fn save_room(&self, _room_id: &Uuid) -> RobinResult<()> {
        // TODO: Implement room persistence
        Ok(())
    }

    async fn load_rooms(&mut self) -> RobinResult<()> {
        // TODO: Implement room loading
        Ok(())
    }
}

/// Social room structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialRoom {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub category: String,
    pub creator_id: Uuid,
    pub max_users: usize,
    pub is_public: bool,
    pub password: Option<String>,
    pub tags: Vec<String>,

    /// Voxel world for this room
    #[serde(skip)]
    pub world: Arc<RwLock<VoxelWorld>>,

    /// Currently connected users
    #[serde(skip)]
    pub connected_users: HashSet<Uuid>,

    pub creation_time: SystemTime,
    pub last_activity: SystemTime,
    pub total_joins: u64,

    pub room_settings: RoomSettings,
    pub moderation: RoomModeration,
    pub build_permissions: BuildPermissions,

    #[serde(skip)]
    pub chat_history: VecDeque<ChatMessage>,

    pub room_state: RoomState,
}

/// User session in a room
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub user_id: Uuid,
    pub room_id: Uuid,
    pub join_time: SystemTime,
    pub last_activity: SystemTime,
    pub position: Vec3,
    pub rotation: f32,
    pub permissions: UserPermissions,
    pub is_building: bool,
    pub cursor_position: Option<Vec3>,
}

/// Room creation parameters
#[derive(Debug, Clone)]
pub struct CreateRoomParams {
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub max_users: Option<usize>,
    pub is_public: Option<bool>,
    pub password: Option<String>,
    pub tags: Option<Vec<String>>,
    pub world_size: Option<[u32; 3]>,
}

/// Room listing summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomSummary {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub category: String,
    pub user_count: usize,
    pub max_users: usize,
    pub is_public: bool,
    pub has_password: bool,
    pub tags: Vec<String>,
    pub creation_time: SystemTime,
    pub last_activity: SystemTime,
    pub total_joins: u64,
}

/// Room list filter
#[derive(Debug, Clone, Default)]
pub struct RoomListFilter {
    pub category: Option<String>,
    pub include_private: Option<bool>,
    pub max_users: Option<usize>,
    pub has_space: Option<bool>,
}

/// Room category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomCategory {
    pub name: String,
    pub description: String,
    pub icon: String,
    pub color: String,
}

/// Room settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomSettings {
    pub allow_spectators: bool,
    pub enable_voice_chat: bool,
    pub enable_text_chat: bool,
    pub world_backup_interval: Duration,
    pub auto_save: bool,
}

impl Default for RoomSettings {
    fn default() -> Self {
        Self {
            allow_spectators: true,
            enable_voice_chat: false,
            enable_text_chat: true,
            world_backup_interval: Duration::from_secs(300), // 5 minutes
            auto_save: true,
        }
    }
}

/// Room moderation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomModeration {
    pub owner_id: Uuid,
    pub moderators: HashSet<Uuid>,
    pub banned_users: HashSet<Uuid>,
    pub muted_users: HashSet<Uuid>,
    pub kick_history: Vec<ModerationAction>,
}

impl RoomModeration {
    pub fn new(owner_id: Uuid) -> Self {
        Self {
            owner_id,
            moderators: HashSet::new(),
            banned_users: HashSet::new(),
            muted_users: HashSet::new(),
            kick_history: Vec::new(),
        }
    }

    pub fn is_moderator(&self, user_id: &Uuid) -> bool {
        *user_id == self.owner_id || self.moderators.contains(user_id)
    }

    pub fn is_user_banned(&self, user_id: &Uuid) -> bool {
        self.banned_users.contains(user_id)
    }

    pub fn is_user_muted(&self, user_id: &Uuid) -> bool {
        self.muted_users.contains(user_id)
    }
}

/// Build permissions for room
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildPermissions {
    pub allow_all_users: bool,
    pub moderator_only_areas: Vec<[i32; 6]>, // Bounding boxes [x1,y1,z1,x2,y2,z2]
    pub protected_materials: HashSet<MaterialType>,
    pub max_build_rate: Option<u32>, // Blocks per minute
}

impl Default for BuildPermissions {
    fn default() -> Self {
        Self {
            allow_all_users: true,
            moderator_only_areas: Vec::new(),
            protected_materials: HashSet::new(),
            max_build_rate: None,
        }
    }
}

/// User permissions in room
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserPermissions {
    Owner,
    Moderator,
    Builder,
    Visitor,
}

/// Room state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomState {
    Active,
    Paused,
    Archived,
    Maintenance,
}

/// Chat message in room
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub timestamp: SystemTime,
    pub message_type: MessageType,
}

/// Chat message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Chat,
    System,
    Announcement,
    Whisper(Uuid), // Target user
}

/// Moderation action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModerationAction {
    pub action_type: ActionType,
    pub target_user: Uuid,
    pub moderator: Uuid,
    pub reason: String,
    pub timestamp: SystemTime,
    pub duration: Option<Duration>,
}

/// Types of moderation actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    Kick,
    Ban,
    Mute,
    Warn,
    Unban,
    Unmute,
}

/// Room events for broadcasting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoomEvent {
    RoomCreated {
        room_id: Uuid,
        creator_id: Uuid,
        room_name: String,
        category: String,
    },
    UserJoined {
        user_id: Uuid,
        room_id: Uuid,
        room_name: String,
        user_count: usize,
    },
    UserLeft {
        user_id: Uuid,
        room_id: Uuid,
        user_count: usize,
    },
    UserMoved {
        user_id: Uuid,
        room_id: Uuid,
        position: Vec3,
        rotation: f32,
    },
    VoxelChanged {
        room_id: Uuid,
        user_id: Uuid,
        change: VoxelChange,
    },
    ChatMessage {
        room_id: Uuid,
        message: ChatMessage,
    },
    RoomStateChanged {
        room_id: Uuid,
        new_state: RoomState,
    },
    ModerationAction {
        room_id: Uuid,
        action: ModerationAction,
    },
}

/// Room search index
pub struct RoomSearchIndex {
    keyword_map: HashMap<String, HashSet<Uuid>>,
}

impl RoomSearchIndex {
    pub fn new() -> Self {
        Self {
            keyword_map: HashMap::new(),
        }
    }

    pub fn add_room(&mut self, room: &SocialRoom) {
        let keywords = self.extract_keywords(room);

        for keyword in keywords {
            self.keyword_map.entry(keyword.to_lowercase())
                .or_insert_with(HashSet::new)
                .insert(room.id);
        }
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<Uuid> {
        let query_words: Vec<&str> = query.to_lowercase().split_whitespace().collect();
        let mut room_scores: HashMap<Uuid, usize> = HashMap::new();

        for word in query_words {
            if let Some(room_ids) = self.keyword_map.get(word) {
                for room_id in room_ids {
                    *room_scores.entry(*room_id).or_insert(0) += 1;
                }
            }
        }

        let mut results: Vec<(Uuid, usize)> = room_scores.into_iter().collect();
        results.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by score descending

        results.into_iter()
            .take(limit)
            .map(|(room_id, _)| room_id)
            .collect()
    }

    fn extract_keywords(&self, room: &SocialRoom) -> Vec<String> {
        let mut keywords = Vec::new();

        // Room name
        keywords.extend(room.name.split_whitespace().map(|s| s.to_string()));

        // Description
        keywords.extend(room.description.split_whitespace().map(|s| s.to_string()));

        // Category
        keywords.push(room.category.clone());

        // Tags
        keywords.extend(room.tags.clone());

        keywords
    }
}

/// Social room configuration
#[derive(Debug, Clone)]
pub struct SocialRoomConfig {
    pub default_max_users: usize,
    pub absolute_max_users: usize,
    pub max_chat_history: usize,
    pub room_timeout: Duration,
    pub enable_voice_chat: bool,
    pub enable_world_backups: bool,
}

impl Default for SocialRoomConfig {
    fn default() -> Self {
        Self {
            default_max_users: 20,
            absolute_max_users: 100,
            max_chat_history: 100,
            room_timeout: Duration::from_secs(1800), // 30 minutes
            enable_voice_chat: false,
            enable_world_backups: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_room_creation() {
        let mut manager = SocialRoomManager::new().unwrap();
        manager.initialize().await.unwrap();

        let creator_id = Uuid::new_v4();
        let params = CreateRoomParams {
            name: "Test Room".to_string(),
            description: Some("A test room".to_string()),
            category: "General".to_string(),
            max_users: Some(10),
            is_public: Some(true),
            password: None,
            tags: Some(vec!["test".to_string()]),
            world_size: Some([32, 32, 32]),
        };

        let room_id = manager.create_room(creator_id, params).await.unwrap();
        assert!(manager.rooms.contains_key(&room_id));

        let room = &manager.rooms[&room_id];
        assert_eq!(room.name, "Test Room");
        assert_eq!(room.creator_id, creator_id);
        assert_eq!(room.max_users, 10);
    }

    #[tokio::test]
    async fn test_join_leave_room() {
        let mut manager = SocialRoomManager::new().unwrap();
        manager.initialize().await.unwrap();

        let creator_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let params = CreateRoomParams {
            name: "Test Room".to_string(),
            description: None,
            category: "General".to_string(),
            max_users: Some(10),
            is_public: Some(true),
            password: None,
            tags: None,
            world_size: None,
        };

        let room_id = manager.create_room(creator_id, params).await.unwrap();

        // Join room
        let session = manager.join_room(user_id, room_id, None).await.unwrap();
        assert_eq!(session.user_id, user_id);
        assert_eq!(session.room_id, room_id);
        assert!(manager.rooms[&room_id].connected_users.contains(&user_id));

        // Leave room
        manager.leave_room(user_id).await.unwrap();
        assert!(!manager.rooms[&room_id].connected_users.contains(&user_id));
        assert!(!manager.user_sessions.contains_key(&user_id));
    }

    #[test]
    fn test_room_search() {
        let mut index = RoomSearchIndex::new();

        let room = SocialRoom {
            id: Uuid::new_v4(),
            name: "Creative Building".to_string(),
            description: "A place for creative projects".to_string(),
            category: "Creative".to_string(),
            creator_id: Uuid::new_v4(),
            max_users: 20,
            is_public: true,
            password: None,
            tags: vec!["creative".to_string(), "building".to_string()],
            world: Arc::new(RwLock::new(VoxelWorld::new().unwrap())),
            connected_users: HashSet::new(),
            creation_time: SystemTime::now(),
            last_activity: SystemTime::now(),
            total_joins: 0,
            room_settings: RoomSettings::default(),
            moderation: RoomModeration::new(Uuid::new_v4()),
            build_permissions: BuildPermissions::default(),
            chat_history: VecDeque::new(),
            room_state: RoomState::Active,
        };

        index.add_room(&room);

        let results = index.search("creative", 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], room.id);
    }
}