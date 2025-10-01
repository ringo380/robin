// Robin Engine Real-Time Collaboration System
// Multi-user building sessions with live synchronization

use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use tokio::sync::{broadcast, RwLock};
use std::sync::Arc;

use crate::engine::{
    error::{RobinResult, RobinError},
    math::{Vec3, Transform},
    generation::voxel_system::{VoxelWorld, VoxelType as VoxelSystemType},
    collaboration::version_control::VoxelChange,
    world::construction::MaterialType,
    platform::cloud_saves::SaveData,
};

/// Real-time collaboration engine
pub struct CollaborationEngine {
    /// Active collaboration sessions
    sessions: HashMap<Uuid, CollaborationSession>,

    /// User session mapping
    user_sessions: HashMap<Uuid, Uuid>, // user_id -> session_id

    /// Session templates and presets
    templates: HashMap<String, SessionTemplate>,

    /// Permission management
    permission_manager: PermissionManager,

    /// Live synchronization
    sync_manager: SyncManager,

    /// Event broadcasting
    event_sender: broadcast::Sender<CollaborationEvent>,

    /// Configuration
    config: CollaborationConfig,

    /// Storage path
    storage_path: std::path::PathBuf,

    /// Feature enabled
    enabled: bool,
}

impl CollaborationEngine {
    /// Create a new collaboration engine
    pub fn new() -> RobinResult<Self> {
        let (event_sender, _) = broadcast::channel(1000);

        Ok(Self {
            sessions: HashMap::new(),
            user_sessions: HashMap::new(),
            templates: Self::create_default_templates(),
            permission_manager: PermissionManager::new(),
            sync_manager: SyncManager::new(),
            event_sender,
            config: CollaborationConfig::default(),
            storage_path: std::path::PathBuf::from("data/collaboration"),
            enabled: true,
        })
    }

    /// Initialize the collaboration system
    pub async fn initialize(&mut self) -> RobinResult<()> {
        if !self.enabled {
            return Ok(());
        }

        log::info!("🤝 Initializing Collaboration Engine");

        // Create storage directory
        std::fs::create_dir_all(&self.storage_path)
            .map_err(|e| RobinError::Community(format!("Failed to create collaboration directory: {}", e)))?;

        // Initialize sync manager
        self.sync_manager.initialize().await?;

        log::info!("✅ Collaboration Engine initialized");
        Ok(())
    }

    /// Start a new collaboration session
    pub async fn start_session(&mut self, params: SessionParams) -> RobinResult<Uuid> {
        if !self.enabled {
            return Err(RobinError::Community("Collaboration disabled".to_string()));
        }

        // Validate parameters
        self.validate_session_params(&params)?;

        let session_id = Uuid::new_v4();
        let now = SystemTime::now();

        // Create permissions FIRST before moving any fields
        let permissions = self.create_session_permissions(&params);

        // Create shared world for collaboration (move params.base_world after permissions)
        let world = if let Some(existing_world) = params.base_world {
            existing_world
        } else {
            {
                let world_size = params.world_size.unwrap_or([64, 64, 64]);
                VoxelWorld::new(
                    params.name.clone(),
                    (world_size[0] as usize, world_size[1] as usize, world_size[2] as usize)
                )
            }
        };

        // Now extract all fields after borrowing is complete
        let name = params.name;
        let description = params.description.unwrap_or_default();
        let host_id = params.host_id;
        let max_participants = params.max_participants.unwrap_or(self.config.default_max_participants);
        let session_type = params.session_type;
        let settings = params.settings.unwrap_or_default();

        let session = CollaborationSession {
            id: session_id,
            name,
            description,
            host_id,
            participants: HashSet::new(),
            max_participants,
            session_type,
            world: Arc::new(RwLock::new(world)),
            creation_time: now,
            last_activity: now,
            status: SessionStatus::Active,
            settings,
            permissions,
            build_history: VecDeque::with_capacity(self.config.max_history_size),
            chat_history: VecDeque::with_capacity(100),
            checkpoints: Vec::new(),
            session_stats: SessionStatistics::new(),
            voice_enabled: params.voice_enabled.unwrap_or(false),
            screen_sharing: params.screen_sharing.unwrap_or(false),
        };

        // Add host as first participant
        self.sessions.insert(session_id, session);
        self.add_participant_to_session(session_id, params.host_id, ParticipantRole::Host).await?;

        // Register with sync manager
        self.sync_manager.register_session(session_id).await?;

        // Broadcast event
        let event = CollaborationEvent::SessionStarted {
            session_id,
            host_id: params.host_id,
            session_name: self.sessions[&session_id].name.clone(),
            session_type: params.session_type,
        };
        self.broadcast_event(event);

        log::info!("🤝 Started collaboration session '{}' ({})",
                  self.sessions[&session_id].name, session_id);

        Ok(session_id)
    }

    /// Join a collaboration session
    pub async fn join_session(&mut self, user_id: Uuid, session_id: Uuid, invite_code: Option<String>) -> RobinResult<SessionJoinInfo> {
        // Extract all needed data from session in a single scope to drop the borrow early
        let (role, permissions) = {
            let session = self.sessions.get_mut(&session_id)
                .ok_or_else(|| RobinError::Community("Session not found".to_string()))?;

            // Check session status
            if session.status != SessionStatus::Active {
                return Err(RobinError::Community("Session is not active".to_string()));
            }

            // Check capacity
            if session.participants.len() >= session.max_participants {
                return Err(RobinError::Community("Session is full".to_string()));
            }

            // Check invite code if required
            if session.settings.require_invite && session.host_id != user_id {
                match (invite_code.as_ref(), session.settings.invite_code.as_ref()) {
                    (Some(provided), Some(required)) if provided == required => {},
                    (None, None) => {},
                    _ => return Err(RobinError::Community("Invalid or missing invite code".to_string())),
                }
            }

            // Calculate role inline to avoid borrow conflicts
            let role = if user_id == session.host_id {
                ParticipantRole::Host
            } else {
                ParticipantRole::Builder
            };

            (role, session.permissions.clone())
        }; // session borrow is dropped here

        // Check permissions
        if !self.permission_manager.can_join_session(&permissions, user_id, role) {
            return Err(RobinError::Community("Insufficient permissions to join session".to_string()));
        }

        // Add participant (now self can be borrowed mutably again)
        self.add_participant_to_session(session_id, user_id, role).await?;

        // Get session again and extract all needed data
        let join_info = {
            let session = self.sessions.get_mut(&session_id)
                .ok_or_else(|| RobinError::Community("Session not found after join".to_string()))?;

            // Get current world state for synchronization
            let world_snapshot = {
                let world = session.world.read().await;
                // TODO: Implement proper snapshot serialization
                // For now, just create a basic serialization
                let snapshot = crate::engine::collaboration::version_control::WorldSnapshot {
                    voxel_data: HashMap::new(), // Would need to iterate through world to populate
                    structures: HashMap::new(),
                    terrain_modifications: Vec::new(),
                    captured_at: SystemTime::now(),
                };
                bincode::serialize(&snapshot)
                    .map_err(|e| RobinError::Community(format!("Failed to serialize snapshot: {}", e)))?
            };

            let participant_count = session.participants.len();

            let info = SessionJoinInfo {
                session_id,
                world_snapshot,
                current_participants: session.participants.clone(),
                session_settings: session.settings.clone(),
                your_role: role,
                build_permissions: session.permissions.clone(),
            };

            // Update activity
            session.last_activity = SystemTime::now();

            info
        }; // session borrow dropped here

        // Broadcast join event
        let participant_count = self.sessions.get(&session_id)
            .map(|s| s.participants.len())
            .unwrap_or(0);
        let event = CollaborationEvent::UserJoined {
            session_id,
            user_id,
            participant_count,
        };
        self.broadcast_event(event);

        log::info!("👤 User {} joined collaboration session {}", user_id, session_id);
        Ok(join_info)
    }

    /// Leave a collaboration session
    pub async fn leave_session(&mut self, user_id: Uuid) -> RobinResult<()> {
        let session_id = self.user_sessions.remove(&user_id)
            .ok_or_else(|| RobinError::Community("User not in any session".to_string()))?;

        let session = self.sessions.get_mut(&session_id)
            .ok_or_else(|| RobinError::Community("Session not found".to_string()))?;

        // Remove from participants
        session.participants.retain(|p| p.user_id != user_id);

        // If host leaves, transfer ownership or end session
        if session.host_id == user_id {
            if let Some(new_host) = session.participants.iter().find(|p| p.role == ParticipantRole::CoHost) {
                session.host_id = new_host.user_id;
                log::info!("👑 Transferred session {} ownership to {}", session_id, new_host.user_id);
            } else if !session.participants.is_empty() {
                let new_host = session.participants.iter().next().unwrap().user_id;
                session.host_id = new_host;
                // Promote to host (HashSet elements are immutable, so remove and re-insert)
                if let Some(mut participant) = session.participants.iter().find(|p| p.user_id == new_host).cloned() {
                    session.participants.remove(&participant);
                    participant.role = ParticipantRole::Host;
                    session.participants.insert(participant);
                }
                log::info!("👑 Promoted user {} to host of session {}", new_host, session_id);
            } else {
                // No participants left, end session
                return self.end_session(session_id).await;
            }
        }

        // Update activity
        session.last_activity = SystemTime::now();

        // Broadcast leave event
        let event = CollaborationEvent::UserLeft {
            session_id,
            user_id,
            participant_count: session.participants.len(),
        };
        self.broadcast_event(event);

        log::info!("👋 User {} left collaboration session {}", user_id, session_id);
        Ok(())
    }

    /// Handle collaborative voxel change
    pub async fn handle_voxel_change(&mut self, user_id: Uuid, change: VoxelChange) -> RobinResult<()> {
        let session_id = self.user_sessions.get(&user_id)
            .ok_or_else(|| RobinError::Community("User not in any session".to_string()))?;

        let session_id = *session_id;

        // Check build permissions first with immutable borrow
        {
            let session = self.sessions.get(&session_id)
                .ok_or_else(|| RobinError::Community("Session not found".to_string()))?;
            if !self.can_user_build(user_id, session, &change)? {
                return Err(RobinError::Community("Insufficient build permissions".to_string()));
            }
        }

        // Now get mutable access for applying changes
        let session = self.sessions.get_mut(&session_id)
            .ok_or_else(|| RobinError::Community("Session not found".to_string()))?;

        // Apply change to shared world
        {
            let mut world = session.world.write().await;
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
                    crate::engine::world::VoxelType::Metal => VoxelSystemType::Stone,
                    crate::engine::world::VoxelType::Brick => VoxelSystemType::Concrete,
                    crate::engine::world::VoxelType::Ice => VoxelSystemType::Solid,
                    crate::engine::world::VoxelType::Obsidian => VoxelSystemType::Stone,
                };
                world.set_voxel(change.position, voxel_system_type);
            }
        }

        // Create collaboration action
        let action = CollaborationAction {
            id: Uuid::new_v4(),
            user_id,
            action_type: ActionType::VoxelChange(change.clone()),
            timestamp: SystemTime::now(),
            synchronized: false,
        };

        // Add to build history
        session.build_history.push_back(action.clone());

        // Keep history within limits
        if session.build_history.len() > self.config.max_history_size {
            session.build_history.pop_front();
        }

        // Update statistics
        session.session_stats.total_changes += 1;
        session.session_stats.last_change = SystemTime::now();

        // Update activity
        session.last_activity = SystemTime::now();

        // Synchronize with all participants
        self.sync_manager.broadcast_change(session_id, action).await?;

        // Broadcast collaboration event
        let event = CollaborationEvent::VoxelChanged {
            session_id,
            user_id,
            change,
        };
        self.broadcast_event(event);

        log::debug!("🧱 Collaborative voxel change applied in session {} by user {}", session_id, user_id);
        Ok(())
    }

    /// Send chat message in collaboration session
    pub async fn send_chat_message(&mut self, user_id: Uuid, message: String) -> RobinResult<()> {
        let session_id = self.user_sessions.get(&user_id)
            .ok_or_else(|| RobinError::Community("User not in any session".to_string()))?;

        let session_id = *session_id;
        let session = self.sessions.get_mut(&session_id)
            .ok_or_else(|| RobinError::Community("Session not found".to_string()))?;

        // Create chat message
        let chat_message = ChatMessage {
            id: Uuid::new_v4(),
            user_id,
            content: message.clone(),
            timestamp: SystemTime::now(),
            message_type: MessageType::Chat,
        };

        // Add to chat history
        session.chat_history.push_back(chat_message.clone());

        // Keep chat history within limits
        if session.chat_history.len() > 100 {
            session.chat_history.pop_front();
        }

        // Broadcast chat message
        let event = CollaborationEvent::ChatMessage {
            session_id,
            message: chat_message,
        };
        self.broadcast_event(event);

        Ok(())
    }

    /// Create checkpoint (save state)
    pub async fn create_checkpoint(&mut self, session_id: Uuid, created_by: Uuid, name: String) -> RobinResult<Uuid> {
        let checkpoint_id = Uuid::new_v4();
        let checkpoint_name;

        {
            let session = self.sessions.get_mut(&session_id)
                .ok_or_else(|| RobinError::Community("Session not found".to_string()))?;

            // Check permissions
            let participant = session.participants.iter().find(|p| p.user_id == created_by)
                .ok_or_else(|| RobinError::Community("User not in session".to_string()))?;

            if !matches!(participant.role, ParticipantRole::Host | ParticipantRole::CoHost) {
                return Err(RobinError::Community("Only hosts can create checkpoints".to_string()));
            }

            // Create world snapshot
            let world_data = {
                let _world = session.world.read().await;
                Vec::new() // TODO: Implement world serialization
            };

            let checkpoint = SessionCheckpoint {
                id: checkpoint_id,
                name: name.clone(),
                created_by,
                created_at: SystemTime::now(),
                world_data,
                description: format!("Checkpoint created during session '{}'", session.name),
            };

            session.checkpoints.push(checkpoint);
            checkpoint_name = name;
        } // session borrow ends here

        // Broadcast checkpoint event
        let event = CollaborationEvent::CheckpointCreated {
            session_id,
            checkpoint_id,
            created_by,
            name: checkpoint_name.clone(),
        };
        self.broadcast_event(event);

        log::info!("💾 Created checkpoint '{}' in session {}", checkpoint_name, session_id);

        Ok(checkpoint_id)
    }

    /// End a collaboration session
    pub async fn end_session(&mut self, session_id: Uuid) -> RobinResult<()> {
        let session = self.sessions.remove(&session_id)
            .ok_or_else(|| RobinError::Community("Session not found".to_string()))?;

        // Remove all user session mappings
        self.user_sessions.retain(|_, sid| *sid != session_id);

        // Unregister from sync manager
        self.sync_manager.unregister_session(session_id).await?;

        // Calculate session duration
        let duration = SystemTime::now().duration_since(session.creation_time)
            .unwrap_or(Duration::from_secs(0));

        // Collect participant list
        let participants: Vec<Uuid> = session.participants.iter().map(|p| p.user_id).collect();

        // Save session data for history
        self.save_session_history(&session).await?;

        // Broadcast session end event
        let event = CollaborationEvent::SessionEnded {
            session_id,
            duration,
            participants,
            final_stats: session.session_stats,
        };
        self.broadcast_event(event);

        log::info!("🏁 Ended collaboration session '{}' ({})", session.name, session_id);
        Ok(())
    }

    /// Get active sessions
    pub fn get_active_sessions(&self) -> Vec<SessionSummary> {
        self.sessions.values()
            .filter(|s| s.status == SessionStatus::Active)
            .map(|s| SessionSummary {
                id: s.id,
                name: s.name.clone(),
                description: s.description.clone(),
                host_id: s.host_id,
                participant_count: s.participants.len(),
                max_participants: s.max_participants,
                session_type: s.session_type,
                creation_time: s.creation_time,
                last_activity: s.last_activity,
                requires_invite: s.settings.require_invite,
                voice_enabled: s.voice_enabled,
                screen_sharing: s.screen_sharing,
            })
            .collect()
    }

    /// Get session details
    pub fn get_session(&self, session_id: &Uuid) -> Option<&CollaborationSession> {
        self.sessions.get(session_id)
    }

    /// Update user cursor position
    pub async fn update_cursor_position(&mut self, user_id: Uuid, position: Vec3, target: Option<Vec3>) -> RobinResult<()> {
        let session_id = self.user_sessions.get(&user_id)
            .ok_or_else(|| RobinError::Community("User not in any session".to_string()))?;

        let session_id = *session_id;

        // Create cursor update action
        let action = CollaborationAction {
            id: Uuid::new_v4(),
            user_id,
            action_type: ActionType::CursorUpdate { position, target },
            timestamp: SystemTime::now(),
            synchronized: false,
        };

        // Broadcast cursor update
        self.sync_manager.broadcast_change(session_id, action).await?;

        Ok(())
    }

    // Helper methods

    fn create_default_templates() -> HashMap<String, SessionTemplate> {
        let mut templates = HashMap::new();

        templates.insert("creative".to_string(), SessionTemplate {
            name: "Creative Building".to_string(),
            description: "Open creative building session".to_string(),
            max_participants: 10,
            default_permissions: BuildPermissions::Creative,
            settings: SessionSettings {
                allow_terrain_modification: true,
                allow_structure_modification: true,
                enable_undo_redo: true,
                auto_save_interval: Duration::from_secs(300),
                require_invite: false,
                invite_code: None,
                enable_voice_chat: false,
                enable_screen_sharing: false,
            },
        });

        templates.insert("guided".to_string(), SessionTemplate {
            name: "Guided Tutorial".to_string(),
            description: "Guided building session with instructor".to_string(),
            max_participants: 20,
            default_permissions: BuildPermissions::Restricted,
            settings: SessionSettings {
                allow_terrain_modification: false,
                allow_structure_modification: true,
                enable_undo_redo: false,
                auto_save_interval: Duration::from_secs(600),
                require_invite: true,
                invite_code: None,
                enable_voice_chat: true,
                enable_screen_sharing: true,
            },
        });

        templates
    }

    fn validate_session_params(&self, params: &SessionParams) -> RobinResult<()> {
        if params.name.trim().is_empty() {
            return Err(RobinError::Community("Session name cannot be empty".to_string()));
        }

        if params.name.len() > 100 {
            return Err(RobinError::Community("Session name too long".to_string()));
        }

        if let Some(max_participants) = params.max_participants {
            if max_participants > self.config.absolute_max_participants {
                return Err(RobinError::Community("Too many participants".to_string()));
            }
        }

        Ok(())
    }

    fn create_session_permissions(&self, params: &SessionParams) -> SessionPermissions {
        SessionPermissions {
            build_permissions: params.build_permissions.unwrap_or(BuildPermissions::Standard),
            who_can_join: params.who_can_join.unwrap_or(JoinPermissions::Anyone),
            who_can_invite: params.who_can_invite.unwrap_or(InvitePermissions::Hosts),
            who_can_modify_settings: SettingsPermissions::HostsOnly,
        }
    }

    async fn add_participant_to_session(&mut self, session_id: Uuid, user_id: Uuid, role: ParticipantRole) -> RobinResult<()> {
        let session = self.sessions.get_mut(&session_id)
            .ok_or_else(|| RobinError::Community("Session not found".to_string()))?;

        let participant = SessionParticipant {
            user_id,
            role,
            joined_at: SystemTime::now(),
            last_activity: SystemTime::now(),
            permissions: ParticipantPermissions::from_role(role),
            current_tool: None,
            cursor_position: Vec3::new(0.0, 0.0, 0.0),
            is_building: false,
        };

        session.participants.insert(participant);
        self.user_sessions.insert(user_id, session_id);

        Ok(())
    }

    fn determine_participant_role(&self, session: &CollaborationSession, user_id: Uuid) -> ParticipantRole {
        if user_id == session.host_id {
            ParticipantRole::Host
        } else {
            ParticipantRole::Builder
        }
    }

    fn can_user_build(&self, user_id: Uuid, session: &CollaborationSession, _change: &VoxelChange) -> RobinResult<bool> {
        let participant = session.participants.iter().find(|p| p.user_id == user_id)
            .ok_or_else(|| RobinError::Community("User not in session".to_string()))?;

        Ok(match participant.role {
            ParticipantRole::Host | ParticipantRole::CoHost => true,
            ParticipantRole::Builder => participant.permissions.can_build,
            ParticipantRole::Observer => false,
        })
    }

    async fn save_session_history(&self, _session: &CollaborationSession) -> RobinResult<()> {
        // TODO: Implement session history persistence
        Ok(())
    }

    fn broadcast_event(&self, event: CollaborationEvent) {
        if let Err(e) = self.event_sender.send(event) {
            log::warn!("Failed to broadcast collaboration event: {}", e);
        }
    }
}

/// Collaboration session
#[derive(Debug, Clone)]
pub struct CollaborationSession {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub host_id: Uuid,
    pub participants: HashSet<SessionParticipant>,
    pub max_participants: usize,
    pub session_type: SessionType,
    pub world: Arc<RwLock<VoxelWorld>>,
    pub creation_time: SystemTime,
    pub last_activity: SystemTime,
    pub status: SessionStatus,
    pub settings: SessionSettings,
    pub permissions: SessionPermissions,
    pub build_history: VecDeque<CollaborationAction>,
    pub chat_history: VecDeque<ChatMessage>,
    pub checkpoints: Vec<SessionCheckpoint>,
    pub session_stats: SessionStatistics,
    pub voice_enabled: bool,
    pub screen_sharing: bool,
}

/// Session creation parameters
#[derive(Debug, Clone)]
pub struct SessionParams {
    pub name: String,
    pub description: Option<String>,
    pub host_id: Uuid,
    pub session_type: SessionType,
    pub max_participants: Option<usize>,
    pub base_world: Option<VoxelWorld>,
    pub world_size: Option<[u32; 3]>,
    pub settings: Option<SessionSettings>,
    pub build_permissions: Option<BuildPermissions>,
    pub who_can_join: Option<JoinPermissions>,
    pub who_can_invite: Option<InvitePermissions>,
    pub voice_enabled: Option<bool>,
    pub screen_sharing: Option<bool>,
}

/// Session join information
#[derive(Debug, Clone)]
pub struct SessionJoinInfo {
    pub session_id: Uuid,
    pub world_snapshot: Vec<u8>, // Serialized world state
    pub current_participants: HashSet<SessionParticipant>,
    pub session_settings: SessionSettings,
    pub your_role: ParticipantRole,
    pub build_permissions: SessionPermissions,
}

/// Session summary for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub host_id: Uuid,
    pub participant_count: usize,
    pub max_participants: usize,
    pub session_type: SessionType,
    pub creation_time: SystemTime,
    pub last_activity: SystemTime,
    pub requires_invite: bool,
    pub voice_enabled: bool,
    pub screen_sharing: bool,
}

/// Session participant
#[derive(Debug, Clone)]
pub struct SessionParticipant {
    pub user_id: Uuid,
    pub role: ParticipantRole,
    pub joined_at: SystemTime,
    pub last_activity: SystemTime,
    pub permissions: ParticipantPermissions,
    pub current_tool: Option<String>,
    pub cursor_position: Vec3,
    pub is_building: bool,
}

// Implement Eq and Hash based on user_id only (unique identifier)
impl PartialEq for SessionParticipant {
    fn eq(&self, other: &Self) -> bool {
        self.user_id == other.user_id
    }
}

impl Eq for SessionParticipant {}

impl std::hash::Hash for SessionParticipant {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.user_id.hash(state);
    }
}

/// Session types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionType {
    Creative,
    Guided,
    Competition,
    Tutorial,
    Presentation,
}

/// Session status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionStatus {
    Active,
    Paused,
    Ended,
}

/// Participant roles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ParticipantRole {
    Host,
    CoHost,
    Builder,
    Observer,
}

/// Session settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSettings {
    pub allow_terrain_modification: bool,
    pub allow_structure_modification: bool,
    pub enable_undo_redo: bool,
    pub auto_save_interval: Duration,
    pub require_invite: bool,
    pub invite_code: Option<String>,
    pub enable_voice_chat: bool,
    pub enable_screen_sharing: bool,
}

impl Default for SessionSettings {
    fn default() -> Self {
        Self {
            allow_terrain_modification: true,
            allow_structure_modification: true,
            enable_undo_redo: true,
            auto_save_interval: Duration::from_secs(300),
            require_invite: false,
            invite_code: None,
            enable_voice_chat: false,
            enable_screen_sharing: false,
        }
    }
}

/// Session permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionPermissions {
    pub build_permissions: BuildPermissions,
    pub who_can_join: JoinPermissions,
    pub who_can_invite: InvitePermissions,
    pub who_can_modify_settings: SettingsPermissions,
}

/// Build permission levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildPermissions {
    Creative,      // Full building access
    Standard,      // Standard building with some restrictions
    Restricted,    // Limited building capabilities
    ReadOnly,      // View only, no building
}

/// Join permission levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JoinPermissions {
    Anyone,
    InviteOnly,
    FriendsOnly,
}

/// Invite permission levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvitePermissions {
    Anyone,
    Hosts,
    HostsOnly,
}

/// Settings modification permissions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettingsPermissions {
    HostsOnly,
    CoHostsAndUp,
    Anyone,
}

/// Participant permissions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ParticipantPermissions {
    pub can_build: bool,
    pub can_modify_terrain: bool,
    pub can_use_advanced_tools: bool,
    pub can_invite_others: bool,
    pub can_kick_users: bool,
    pub can_modify_settings: bool,
    pub can_create_checkpoints: bool,
}

impl ParticipantPermissions {
    pub fn from_role(role: ParticipantRole) -> Self {
        match role {
            ParticipantRole::Host => Self {
                can_build: true,
                can_modify_terrain: true,
                can_use_advanced_tools: true,
                can_invite_others: true,
                can_kick_users: true,
                can_modify_settings: true,
                can_create_checkpoints: true,
            },
            ParticipantRole::CoHost => Self {
                can_build: true,
                can_modify_terrain: true,
                can_use_advanced_tools: true,
                can_invite_others: true,
                can_kick_users: true,
                can_modify_settings: false,
                can_create_checkpoints: true,
            },
            ParticipantRole::Builder => Self {
                can_build: true,
                can_modify_terrain: false,
                can_use_advanced_tools: false,
                can_invite_others: false,
                can_kick_users: false,
                can_modify_settings: false,
                can_create_checkpoints: false,
            },
            ParticipantRole::Observer => Self {
                can_build: false,
                can_modify_terrain: false,
                can_use_advanced_tools: false,
                can_invite_others: false,
                can_kick_users: false,
                can_modify_settings: false,
                can_create_checkpoints: false,
            },
        }
    }
}

/// Collaboration action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationAction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub action_type: ActionType,
    pub timestamp: SystemTime,
    pub synchronized: bool,
}

/// Action types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    VoxelChange(VoxelChange),
    CursorUpdate { position: Vec3, target: Option<Vec3> },
    ToolChange { tool: String },
    SelectionChange { selection: Vec<[i32; 3]> },
    UndoAction,
    RedoAction,
}

/// Chat message
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
    Voice, // Voice message transcription
}

/// Session checkpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCheckpoint {
    pub id: Uuid,
    pub name: String,
    pub created_by: Uuid,
    pub created_at: SystemTime,
    pub world_data: Vec<u8>,
    pub description: String,
}

/// Session statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStatistics {
    pub total_changes: u64,
    pub total_participants: u64,
    pub peak_concurrent_users: usize,
    pub total_chat_messages: u64,
    pub checkpoints_created: u64,
    pub last_change: SystemTime,
}

impl SessionStatistics {
    pub fn new() -> Self {
        Self {
            total_changes: 0,
            total_participants: 0,
            peak_concurrent_users: 0,
            total_chat_messages: 0,
            checkpoints_created: 0,
            last_change: SystemTime::now(),
        }
    }
}

/// Session template
#[derive(Debug, Clone)]
pub struct SessionTemplate {
    pub name: String,
    pub description: String,
    pub max_participants: usize,
    pub default_permissions: BuildPermissions,
    pub settings: SessionSettings,
}

/// Collaboration events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollaborationEvent {
    SessionStarted {
        session_id: Uuid,
        host_id: Uuid,
        session_name: String,
        session_type: SessionType,
    },
    SessionEnded {
        session_id: Uuid,
        duration: Duration,
        participants: Vec<Uuid>,
        final_stats: SessionStatistics,
    },
    UserJoined {
        session_id: Uuid,
        user_id: Uuid,
        participant_count: usize,
    },
    UserLeft {
        session_id: Uuid,
        user_id: Uuid,
        participant_count: usize,
    },
    VoxelChanged {
        session_id: Uuid,
        user_id: Uuid,
        change: VoxelChange,
    },
    ChatMessage {
        session_id: Uuid,
        message: ChatMessage,
    },
    CheckpointCreated {
        session_id: Uuid,
        checkpoint_id: Uuid,
        created_by: Uuid,
        name: String,
    },
    RoleChanged {
        session_id: Uuid,
        user_id: Uuid,
        old_role: ParticipantRole,
        new_role: ParticipantRole,
    },
}

/// Permission manager
pub struct PermissionManager {
    role_permissions: HashMap<ParticipantRole, ParticipantPermissions>,
}

impl PermissionManager {
    pub fn new() -> Self {
        let mut role_permissions = HashMap::new();

        for role in [ParticipantRole::Host, ParticipantRole::CoHost, ParticipantRole::Builder, ParticipantRole::Observer] {
            role_permissions.insert(role, ParticipantPermissions::from_role(role));
        }

        Self { role_permissions }
    }

    pub fn can_join_session(&self, _permissions: &SessionPermissions, _user_id: Uuid, _role: ParticipantRole) -> bool {
        // TODO: Implement permission checking logic
        true
    }
}

/// Live synchronization manager
pub struct SyncManager {
    active_sessions: HashSet<Uuid>,
}

impl SyncManager {
    pub fn new() -> Self {
        Self {
            active_sessions: HashSet::new(),
        }
    }

    pub async fn initialize(&mut self) -> RobinResult<()> {
        log::info!("🔄 Initializing Sync Manager");
        Ok(())
    }

    pub async fn register_session(&mut self, session_id: Uuid) -> RobinResult<()> {
        self.active_sessions.insert(session_id);
        log::debug!("📡 Registered session {} for synchronization", session_id);
        Ok(())
    }

    pub async fn unregister_session(&mut self, session_id: Uuid) -> RobinResult<()> {
        self.active_sessions.remove(&session_id);
        log::debug!("📡 Unregistered session {} from synchronization", session_id);
        Ok(())
    }

    pub async fn broadcast_change(&self, session_id: Uuid, action: CollaborationAction) -> RobinResult<()> {
        if !self.active_sessions.contains(&session_id) {
            return Err(RobinError::Community("Session not registered for sync".to_string()));
        }

        // TODO: Implement real-time synchronization
        // This would typically involve WebSocket connections, message queues, etc.
        log::debug!("🔄 Broadcasting action {} to session {}", action.id, session_id);
        Ok(())
    }
}

/// Collaboration configuration
#[derive(Debug, Clone)]
pub struct CollaborationConfig {
    pub default_max_participants: usize,
    pub absolute_max_participants: usize,
    pub max_history_size: usize,
    pub sync_interval: Duration,
    pub auto_save_interval: Duration,
    pub session_timeout: Duration,
}

impl Default for CollaborationConfig {
    fn default() -> Self {
        Self {
            default_max_participants: 10,
            absolute_max_participants: 50,
            max_history_size: 1000,
            sync_interval: Duration::from_millis(100),
            auto_save_interval: Duration::from_secs(300),
            session_timeout: Duration::from_secs(3600),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_creation() {
        let mut engine = CollaborationEngine::new().unwrap();
        engine.initialize().await.unwrap();

        let host_id = Uuid::new_v4();
        let params = SessionParams {
            name: "Test Session".to_string(),
            description: Some("A test collaboration session".to_string()),
            host_id,
            session_type: SessionType::Creative,
            max_participants: Some(5),
            base_world: None,
            world_size: Some([32, 32, 32]),
            settings: None,
            build_permissions: None,
            who_can_join: None,
            who_can_invite: None,
            voice_enabled: Some(false),
            screen_sharing: Some(false),
        };

        let session_id = engine.start_session(params).await.unwrap();
        assert!(engine.sessions.contains_key(&session_id));

        let session = &engine.sessions[&session_id];
        assert_eq!(session.name, "Test Session");
        assert_eq!(session.host_id, host_id);
        assert_eq!(session.max_participants, 5);
        assert_eq!(session.participants.len(), 1); // Host automatically added
    }

    #[tokio::test]
    async fn test_join_leave_session() {
        let mut engine = CollaborationEngine::new().unwrap();
        engine.initialize().await.unwrap();

        let host_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let params = SessionParams {
            name: "Test Session".to_string(),
            description: None,
            host_id,
            session_type: SessionType::Creative,
            max_participants: Some(5),
            base_world: None,
            world_size: None,
            settings: None,
            build_permissions: None,
            who_can_join: None,
            who_can_invite: None,
            voice_enabled: None,
            screen_sharing: None,
        };

        let session_id = engine.start_session(params).await.unwrap();

        // Join session
        let join_info = engine.join_session(user_id, session_id, None).await.unwrap();
        assert_eq!(join_info.session_id, session_id);
        assert_eq!(join_info.your_role, ParticipantRole::Builder);
        assert_eq!(engine.sessions[&session_id].participants.len(), 2);

        // Leave session
        engine.leave_session(user_id).await.unwrap();
        assert_eq!(engine.sessions[&session_id].participants.len(), 1);
        assert!(!engine.user_sessions.contains_key(&user_id));
    }

    #[tokio::test]
    async fn test_collaborative_building() {
        let mut engine = CollaborationEngine::new().unwrap();
        engine.initialize().await.unwrap();

        let host_id = Uuid::new_v4();
        let params = SessionParams {
            name: "Build Session".to_string(),
            description: None,
            host_id,
            session_type: SessionType::Creative,
            max_participants: None,
            base_world: None,
            world_size: None,
            settings: None,
            build_permissions: None,
            who_can_join: None,
            who_can_invite: None,
            voice_enabled: None,
            screen_sharing: None,
        };

        let session_id = engine.start_session(params).await.unwrap();

        // Test voxel change
        let change = VoxelChange {
            position: [0, 1, 0],
            old_material: MaterialType::Air,
            new_material: MaterialType::Stone,
            // change_type: crate::engine::world::construction::ChangeType::Place, // TODO: Add when ChangeType is defined
        };

        engine.handle_voxel_change(host_id, change.clone()).await.unwrap();

        let session = &engine.sessions[&session_id];
        assert_eq!(session.build_history.len(), 1);
        assert_eq!(session.session_stats.total_changes, 1);
    }
}