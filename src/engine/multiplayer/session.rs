use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

use crate::engine::error::RobinResult;
use super::{UserId, SessionId, UserInfo, SessionInfo, SessionSettings, UserPermissions};
use super::networking::{NetworkManager, NetworkMessage};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub id: SessionId,
    pub current_phase: SessionPhase,
    pub participants: HashMap<UserId, SessionParticipant>,
    pub world_state_version: u64,
    pub last_activity: u64,
    pub auto_save_enabled: bool,
    pub last_save: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionPhase {
    Planning,
    Building,
    Testing,
    Reviewing,
    Paused,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionParticipant {
    pub user_id: UserId,
    pub join_time: u64,
    pub last_activity: u64,
    pub connection_state: ParticipantConnectionState,
    pub current_tool: Option<String>,
    pub current_position: [f32; 3],
    pub permissions: UserPermissions,
    pub session_role: SessionRole,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParticipantConnectionState {
    Connected,
    Idle,
    Disconnected,
    Reconnecting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionRole {
    Host,
    CoHost,
    Builder,
    Observer,
    Guest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInvite {
    pub id: uuid::Uuid,
    pub session_id: SessionId,
    pub from_user: UserId,
    pub to_user: UserId,
    pub role: SessionRole,
    pub message: Option<String>,
    pub expires_at: u64,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionTemplate {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub category: SessionCategory,
    pub default_settings: SessionSettings,
    pub default_permissions: UserPermissions,
    pub max_participants: usize,
    pub estimated_duration: Duration,
    pub required_tools: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionCategory {
    Collaborative,
    Educational,
    Competitive,
    Showcase,
    Testing,
    Social,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetrics {
    pub session_id: SessionId,
    pub duration: Duration,
    pub participant_count: usize,
    pub peak_participants: usize,
    pub constructions_created: u32,
    pub terrain_modifications: u32,
    pub messages_sent: u32,
    pub saves_performed: u32,
    pub tools_used: HashMap<String, u32>,
    pub user_activity: HashMap<UserId, UserActivity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserActivity {
    pub user_id: UserId,
    pub active_time: Duration,
    pub constructions_built: u32,
    pub terrain_edits: u32,
    pub messages_sent: u32,
    pub tools_switched: u32,
    pub collaboration_events: u32,
}

pub struct SessionManager {
    pub active_sessions: HashMap<SessionId, SessionState>,
    pub session_templates: HashMap<uuid::Uuid, SessionTemplate>,
    pub pending_invites: HashMap<uuid::Uuid, SessionInvite>,
    pub session_metrics: HashMap<SessionId, SessionMetrics>,
    pub network_manager: Option<Arc<RwLock<NetworkManager>>>,
    pub save_system: SessionSaveSystem,
    pub invite_system: InviteSystem,
    pub metrics_collector: MetricsCollector,
    pub auto_cleanup: AutoCleanupSystem,
}

pub struct SessionSaveSystem {
    pub auto_save_interval: Duration,
    pub max_save_slots: usize,
    pub compression_enabled: bool,
    pub save_history: HashMap<SessionId, VecDeque<SaveSnapshot>>,
    pub pending_saves: HashMap<SessionId, SaveRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveSnapshot {
    pub id: uuid::Uuid,
    pub session_id: SessionId,
    pub timestamp: u64,
    pub size: usize,
    pub compressed_data: Vec<u8>,
    pub metadata: SaveMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveMetadata {
    pub version: u64,
    pub participant_count: usize,
    pub constructions_count: u32,
    pub terrain_chunks: u32,
    pub scripts_count: u32,
    pub checksum: String,
}

#[derive(Debug, Clone)]
pub struct SaveRequest {
    pub session_id: SessionId,
    pub requested_by: UserId,
    pub save_type: SaveType,
    pub description: Option<String>,
    pub scheduled_time: Option<Instant>,
}

#[derive(Debug, Clone)]
pub enum SaveType {
    Manual,
    Automatic,
    Checkpoint,
    Export,
}

pub struct InviteSystem {
    pub invite_templates: HashMap<String, InviteTemplate>,
    pub invite_history: HashMap<UserId, Vec<SessionInvite>>,
    pub blocked_users: HashMap<UserId, HashSet<UserId>>,
    pub invite_limits: HashMap<UserId, InviteLimits>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteTemplate {
    pub name: String,
    pub default_message: String,
    pub default_role: SessionRole,
    pub auto_accept: bool,
    pub expiry_duration: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteLimits {
    pub max_pending_invites: usize,
    pub max_invites_per_hour: usize,
    pub max_invites_per_day: usize,
    pub current_hour_count: usize,
    pub current_day_count: usize,
    pub hour_reset: u64,
    pub day_reset: u64,
}

impl Default for InviteLimits {
    fn default() -> Self {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        Self {
            max_pending_invites: 10,
            max_invites_per_hour: 20,
            max_invites_per_day: 100,
            current_hour_count: 0,
            current_day_count: 0,
            hour_reset: now + 3600,
            day_reset: now + 86400,
        }
    }
}

pub struct MetricsCollector {
    pub collection_interval: Duration,
    pub last_collection: Instant,
    pub metric_history: HashMap<SessionId, VecDeque<MetricsSnapshot>>,
    pub performance_alerts: Vec<PerformanceAlert>,
}

#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub timestamp: u64,
    pub session_metrics: SessionMetrics,
    pub performance_data: PerformanceData,
}

#[derive(Debug, Clone)]
pub struct PerformanceData {
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub network_latency: Duration,
    pub frame_rate: f32,
    pub bandwidth_usage: (f64, f64),
}

#[derive(Debug, Clone)]
pub struct PerformanceAlert {
    pub alert_type: AlertType,
    pub session_id: SessionId,
    pub message: String,
    pub severity: AlertSeverity,
    pub timestamp: Instant,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AlertType {
    HighLatency,
    LowFrameRate,
    HighMemoryUsage,
    NetworkIssues,
    ParticipantOverload,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

pub struct AutoCleanupSystem {
    pub cleanup_interval: Duration,
    pub last_cleanup: Instant,
    pub session_timeout: Duration,
    pub inactive_participant_timeout: Duration,
    pub max_session_history: usize,
}

impl Default for AutoCleanupSystem {
    fn default() -> Self {
        Self {
            cleanup_interval: Duration::from_secs(300), // 5 minutes
            last_cleanup: Instant::now(),
            session_timeout: Duration::from_secs(7200), // 2 hours
            inactive_participant_timeout: Duration::from_secs(1800), // 30 minutes
            max_session_history: 1000,
        }
    }
}

impl SessionManager {
    pub fn new() -> RobinResult<Self> {
        Ok(Self {
            active_sessions: HashMap::new(),
            session_templates: HashMap::new(),
            pending_invites: HashMap::new(),
            session_metrics: HashMap::new(),
            network_manager: None,
            save_system: SessionSaveSystem {
                auto_save_interval: Duration::from_secs(300),
                max_save_slots: 10,
                compression_enabled: true,
                save_history: HashMap::new(),
                pending_saves: HashMap::new(),
            },
            invite_system: InviteSystem {
                invite_templates: HashMap::new(),
                invite_history: HashMap::new(),
                blocked_users: HashMap::new(),
                invite_limits: HashMap::new(),
            },
            metrics_collector: MetricsCollector {
                collection_interval: Duration::from_secs(60),
                last_collection: Instant::now(),
                metric_history: HashMap::new(),
                performance_alerts: Vec::new(),
            },
            auto_cleanup: AutoCleanupSystem::default(),
        })
    }

    pub fn set_network_manager(&mut self, network_manager: Arc<RwLock<NetworkManager>>) {
        self.network_manager = Some(network_manager);
    }

    pub fn initialize_host_session(&mut self) -> RobinResult<()> {
        let session_id = SessionId::new();
        let host_id = UserId::new();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        let mut participants = HashMap::new();
        participants.insert(host_id.clone(), SessionParticipant {
            user_id: host_id.clone(),
            join_time: now,
            last_activity: now,
            connection_state: ParticipantConnectionState::Connected,
            current_tool: None,
            current_position: [0.0, 0.0, 0.0],
            permissions: UserPermissions::default(),
            session_role: SessionRole::Host,
        });

        let session_state = SessionState {
            id: session_id.clone(),
            current_phase: SessionPhase::Planning,
            participants,
            world_state_version: 0,
            last_activity: now,
            auto_save_enabled: true,
            last_save: now,
        };

        self.active_sessions.insert(session_id.clone(), session_state);
        self.initialize_session_metrics(session_id)?;
        
        Ok(())
    }

    pub fn create_session(&mut self, session_id: SessionId) -> RobinResult<()> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        let session_state = SessionState {
            id: session_id.clone(),
            current_phase: SessionPhase::Planning,
            participants: HashMap::new(),
            world_state_version: 0,
            last_activity: now,
            auto_save_enabled: true,
            last_save: now,
        };

        self.active_sessions.insert(session_id.clone(), session_state);
        self.initialize_session_metrics(session_id)?;
        
        Ok(())
    }

    pub fn join_session(&mut self, session_id: SessionId, user_id: UserId) -> RobinResult<()> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        if let Some(session) = self.active_sessions.get_mut(&session_id) {
            let participant = SessionParticipant {
                user_id: user_id.clone(),
                join_time: now,
                last_activity: now,
                connection_state: ParticipantConnectionState::Connected,
                current_tool: None,
                current_position: [0.0, 0.0, 0.0],
                permissions: UserPermissions::default(),
                session_role: if session.participants.is_empty() { SessionRole::Host } else { SessionRole::Builder },
            };

            session.participants.insert(user_id.clone(), participant);
            session.last_activity = now;

            if let Some(metrics) = self.session_metrics.get_mut(&session_id) {
                metrics.participant_count = session.participants.len();
                metrics.peak_participants = metrics.peak_participants.max(metrics.participant_count);
                
                metrics.user_activity.entry(user_id.clone()).or_insert(UserActivity {
                    user_id,
                    active_time: Duration::from_secs(0),
                    constructions_built: 0,
                    terrain_edits: 0,
                    messages_sent: 0,
                    tools_switched: 0,
                    collaboration_events: 0,
                });
            }

            self.broadcast_session_update(&session_id)?;
            Ok(())
        } else {
            Err(crate::engine::error::RobinError::SessionError("Session not found".to_string()))
        }
    }

    pub fn leave_session(&mut self, session_id: SessionId, user_id: UserId) -> RobinResult<()> {
        if let Some(session) = self.active_sessions.get_mut(&session_id) {
            session.participants.remove(&user_id);
            session.last_activity = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

            if let Some(metrics) = self.session_metrics.get_mut(&session_id) {
                metrics.participant_count = session.participants.len();
            }

            if session.participants.is_empty() {
                self.end_session(session_id)?;
            } else {
                self.broadcast_session_update(&session_id)?;
            }
            
            Ok(())
        } else {
            Err(crate::engine::error::RobinError::SessionError("Session not found".to_string()))
        }
    }

    pub fn send_invite(&mut self, session_id: SessionId, from_user: UserId, to_user: UserId, role: SessionRole, message: Option<String>) -> RobinResult<uuid::Uuid> {
        if self.invite_system.blocked_users
            .get(&to_user)
            .map_or(false, |blocked| blocked.contains(&from_user)) {
            return Err(crate::engine::error::RobinError::SessionError("User has blocked invitations from sender".to_string()));
        }

        let limits = self.invite_system.invite_limits.entry(from_user.clone()).or_insert_with(InviteLimits::default);
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        if now > limits.hour_reset {
            limits.current_hour_count = 0;
            limits.hour_reset = now + 3600;
        }
        if now > limits.day_reset {
            limits.current_day_count = 0;
            limits.day_reset = now + 86400;
        }

        if limits.current_hour_count >= limits.max_invites_per_hour {
            return Err(crate::engine::error::RobinError::SessionError("Hourly invite limit reached".to_string()));
        }
        if limits.current_day_count >= limits.max_invites_per_day {
            return Err(crate::engine::error::RobinError::SessionError("Daily invite limit reached".to_string()));
        }

        let pending_count = self.pending_invites.values()
            .filter(|invite| invite.from_user == from_user)
            .count();
        if pending_count >= limits.max_pending_invites {
            return Err(crate::engine::error::RobinError::SessionError("Too many pending invites".to_string()));
        }

        let invite_id = uuid::Uuid::new_v4();
        let invite = SessionInvite {
            id: invite_id,
            session_id,
            from_user: from_user.clone(),
            to_user: to_user.clone(),
            role,
            message,
            expires_at: now + 3600, // 1 hour expiry
            created_at: now,
        };

        self.pending_invites.insert(invite_id, invite.clone());
        self.invite_system.invite_history.entry(from_user.clone()).or_insert_with(Vec::new).push(invite);

        limits.current_hour_count += 1;
        limits.current_day_count += 1;

        if let Some(network_manager) = &self.network_manager {
            let message = NetworkMessage::SessionInvite {
                session_id,
                from_user,
                to_user,
            };
            network_manager.write().unwrap().send_message_to_user(&to_user, message)?;
        }

        Ok(invite_id)
    }

    pub fn accept_invite(&mut self, invite_id: uuid::Uuid) -> RobinResult<SessionId> {
        if let Some(invite) = self.pending_invites.remove(&invite_id) {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            if now > invite.expires_at {
                return Err(crate::engine::error::RobinError::SessionError("Invite has expired".to_string()));
            }

            self.join_session(invite.session_id.clone(), invite.to_user)?;
            Ok(invite.session_id)
        } else {
            Err(crate::engine::error::RobinError::SessionError("Invite not found".to_string()))
        }
    }

    pub fn decline_invite(&mut self, invite_id: uuid::Uuid) -> RobinResult<()> {
        if self.pending_invites.remove(&invite_id).is_some() {
            Ok(())
        } else {
            Err(crate::engine::error::RobinError::SessionError("Invite not found".to_string()))
        }
    }

    pub fn change_session_phase(&mut self, session_id: SessionId, new_phase: SessionPhase) -> RobinResult<()> {
        if let Some(session) = self.active_sessions.get_mut(&session_id) {
            session.current_phase = new_phase;
            session.last_activity = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            
            self.broadcast_session_update(&session_id)?;
            Ok(())
        } else {
            Err(crate::engine::error::RobinError::SessionError("Session not found".to_string()))
        }
    }

    pub fn update_participant(&mut self, session_id: SessionId, user_id: UserId, position: [f32; 3], tool: Option<String>) -> RobinResult<()> {
        if let Some(session) = self.active_sessions.get_mut(&session_id) {
            if let Some(participant) = session.participants.get_mut(&user_id) {
                participant.last_activity = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                participant.current_position = position;
                
                if participant.current_tool != tool {
                    participant.current_tool = tool;
                    if let Some(metrics) = self.session_metrics.get_mut(&session_id) {
                        if let Some(user_activity) = metrics.user_activity.get_mut(&user_id) {
                            user_activity.tools_switched += 1;
                        }
                    }
                }
                
                session.last_activity = participant.last_activity;
            }
        }
        Ok(())
    }

    pub fn save_session(&mut self, session_id: SessionId, user_id: UserId, save_type: SaveType, description: Option<String>) -> RobinResult<uuid::Uuid> {
        let save_id = uuid::Uuid::new_v4();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        if let Some(session) = self.active_sessions.get_mut(&session_id) {
            session.last_save = now;
            
            let save_data = bincode::serialize(session)
                .map_err(|e| crate::engine::error::RobinError::SerializationError { 
                    object_type: "SessionState".to_string(), 
                    reason: format!("Failed to serialize session: {}", e) 
                })?;

            let compressed_data = if self.save_system.compression_enabled {
                compress_data(&save_data)?
            } else {
                save_data
            };

            let metadata = SaveMetadata {
                version: session.world_state_version,
                participant_count: session.participants.len(),
                constructions_count: 0, // TODO: Get actual count
                terrain_chunks: 0, // TODO: Get actual count
                scripts_count: 0, // TODO: Get actual count
                checksum: calculate_checksum(&compressed_data),
            };

            let snapshot = SaveSnapshot {
                id: save_id,
                session_id: session_id.clone(),
                timestamp: now,
                size: compressed_data.len(),
                compressed_data,
                metadata,
            };

            let save_history = self.save_system.save_history.entry(session_id.clone()).or_insert_with(VecDeque::new);
            save_history.push_back(snapshot);
            
            if save_history.len() > self.save_system.max_save_slots {
                save_history.pop_front();
            }

            if let Some(metrics) = self.session_metrics.get_mut(&session_id) {
                metrics.saves_performed += 1;
            }

            Ok(save_id)
        } else {
            Err(crate::engine::error::RobinError::SessionError("Session not found".to_string()))
        }
    }

    pub fn load_session(&mut self, session_id: SessionId, save_id: uuid::Uuid) -> RobinResult<()> {
        if let Some(save_history) = self.save_system.save_history.get(&session_id) {
            if let Some(snapshot) = save_history.iter().find(|s| s.id == save_id) {
                let decompressed_data = if self.save_system.compression_enabled {
                    decompress_data(&snapshot.compressed_data)?
                } else {
                    snapshot.compressed_data.clone()
                };

                let session_state: SessionState = bincode::deserialize(&decompressed_data)
                    .map_err(|e| crate::engine::error::RobinError::SerializationError { 
                        object_type: "SessionState".to_string(), 
                        reason: format!("Failed to deserialize session: {}", e) 
                    })?;

                self.active_sessions.insert(session_id, session_state);
                Ok(())
            } else {
                Err(crate::engine::error::RobinError::SessionError("Save not found".to_string()))
            }
        } else {
            Err(crate::engine::error::RobinError::SessionError("No save history found for session".to_string()))
        }
    }

    pub fn update(&mut self, _delta_time: f32) -> RobinResult<()> {
        self.process_auto_saves()?;
        self.collect_metrics()?;
        self.cleanup_inactive_sessions()?;
        self.cleanup_expired_invites()?;
        Ok(())
    }

    fn initialize_session_metrics(&mut self, session_id: SessionId) -> RobinResult<()> {
        let metrics = SessionMetrics {
            session_id: session_id.clone(),
            duration: Duration::from_secs(0),
            participant_count: 0,
            peak_participants: 0,
            constructions_created: 0,
            terrain_modifications: 0,
            messages_sent: 0,
            saves_performed: 0,
            tools_used: HashMap::new(),
            user_activity: HashMap::new(),
        };

        self.session_metrics.insert(session_id, metrics);
        Ok(())
    }

    fn broadcast_session_update(&self, session_id: &SessionId) -> RobinResult<()> {
        if let Some(network_manager) = &self.network_manager {
            if let Some(session) = self.active_sessions.get(session_id) {
                for user_id in session.participants.keys() {
                    let user_update = NetworkMessage::UserUpdate {
                        user_info: UserInfo {
                            id: user_id.clone(),
                            username: "Unknown".to_string(), // TODO: Get actual username
                            avatar_url: None,
                            permissions: UserPermissions::default(),
                            current_position: [0.0, 0.0, 0.0],
                            current_tool: None,
                            last_activity: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                            ping: Duration::from_millis(0),
                        },
                    };
                    
                    network_manager.write().unwrap().broadcast_message(user_update, Some(user_id))?;
                }
            }
        }
        Ok(())
    }

    fn process_auto_saves(&mut self) -> RobinResult<()> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let mut sessions_to_save = Vec::new();

        for (session_id, session) in &self.active_sessions {
            if session.auto_save_enabled && 
               (now - session.last_save) >= self.save_system.auto_save_interval.as_secs() {
                sessions_to_save.push(*session_id);
            }
        }

        for session_id in sessions_to_save {
            let host_id = UserId::new(); // TODO: Get actual host ID
            let _ = self.save_session(session_id, host_id, SaveType::Automatic, Some("Auto-save".to_string()));
        }

        Ok(())
    }

    fn collect_metrics(&mut self) -> RobinResult<()> {
        let now = Instant::now();
        if now.duration_since(self.metrics_collector.last_collection) >= self.metrics_collector.collection_interval {
            for (session_id, session) in &self.active_sessions {
                if let Some(metrics) = self.session_metrics.get_mut(session_id) {
                    metrics.duration = Duration::from_secs(
                        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() - 
                        session.participants.values().map(|p| p.join_time).min().unwrap_or(0)
                    );
                    metrics.participant_count = session.participants.len();
                }
            }
            
            self.metrics_collector.last_collection = now;
        }
        Ok(())
    }

    fn cleanup_inactive_sessions(&mut self) -> RobinResult<()> {
        let now = Instant::now();
        if now.duration_since(self.auto_cleanup.last_cleanup) >= self.auto_cleanup.cleanup_interval {
            let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            let timeout_threshold = current_time - self.auto_cleanup.session_timeout.as_secs();

            let inactive_sessions: Vec<SessionId> = self.active_sessions.iter()
                .filter(|(_, session)| session.last_activity < timeout_threshold)
                .map(|(id, _)| *id)
                .collect();

            for session_id in inactive_sessions {
                self.end_session(session_id)?;
            }

            self.auto_cleanup.last_cleanup = now;
        }
        Ok(())
    }

    fn cleanup_expired_invites(&mut self) -> RobinResult<()> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        self.pending_invites.retain(|_, invite| invite.expires_at > now);
        Ok(())
    }

    fn end_session(&mut self, session_id: SessionId) -> RobinResult<()> {
        if let Some(session) = self.active_sessions.remove(&session_id) {
            let _ = self.save_session(session_id, UserId::new(), SaveType::Checkpoint, Some("Session ended".to_string()));
        }
        Ok(())
    }

    pub fn get_session_info(&self, session_id: &SessionId) -> Option<&SessionState> {
        self.active_sessions.get(session_id)
    }

    pub fn get_session_metrics(&self, session_id: &SessionId) -> Option<&SessionMetrics> {
        self.session_metrics.get(session_id)
    }

    pub fn get_pending_invites_for_user(&self, user_id: &UserId) -> Vec<SessionInvite> {
        self.pending_invites.values()
            .filter(|invite| invite.to_user == *user_id)
            .cloned()
            .collect()
    }

    pub fn get_user_sessions(&self, user_id: &UserId) -> Vec<SessionId> {
        self.active_sessions.iter()
            .filter(|(_, session)| session.participants.contains_key(user_id))
            .map(|(id, _)| *id)
            .collect()
    }
}

fn compress_data(data: &[u8]) -> RobinResult<Vec<u8>> {
    Ok(data.to_vec())
}

fn decompress_data(data: &[u8]) -> RobinResult<Vec<u8>> {
    Ok(data.to_vec())
}

fn calculate_checksum(data: &[u8]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}