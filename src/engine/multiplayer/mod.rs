use crate::engine::error::RobinResult;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Instant, Duration};

pub mod networking;
pub mod collaboration;
pub mod version_control;
pub mod permissions;
pub mod communication;
pub mod shared_assets;
pub mod synchronization;

pub use networking::{NetworkManager, NetworkConfig, ConnectionInfo, NetworkStats};
// Remove this line as we define these types below
// pub use collaboration::{CollaborationManager, CollaborationSession, ChangeEvent, SyncManager};
pub use version_control::{VersionControl, Branch, Commit, MergeStrategy, ConflictResolution};
pub use permissions::{PermissionManager, Role, Permission, AccessLevel, UserPermissions};
pub use communication::{CommunicationManager, ChatManager, VoiceManager, MessageType};
pub use shared_assets::{SharedAssetManager, AssetLibrary, AssetMetadata, AssetSyncStatus};
pub use synchronization::{UserCursor, SelectionData, BoundingBox, SynchronizationEngine, SyncStats};

// Type aliases and additional definitions for collaboration system
pub use collaboration::CollaborationTools as CollaborationManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationSession {
    pub session_id: SessionId,
    pub participants: Vec<UserId>,
    pub created_at: u64,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeEvent {
    pub event_id: uuid::Uuid,
    pub user_id: UserId,
    pub event_type: String,
    pub data: Vec<u8>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncManager {
    pub sync_stats: SyncStats,
    pub last_sync: u64,
    pub pending_syncs: u32,
}

// Additional types needed by synchronization module
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorldSnapshot {
    pub version: u64,
    pub timestamp: u64,
    pub entities: Vec<EntitySnapshot>,
    pub terrain_chunks: Vec<TerrainChunk>,
    pub user_constructions: Vec<UserConstruction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EntitySnapshot {
    pub id: uuid::Uuid,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub entity_type: String,
    pub components: Vec<ComponentSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComponentSnapshot {
    pub component_type: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TerrainChunk {
    pub chunk_id: [i32; 2],
    pub modifications: Vec<TerrainModification>,
    pub last_modified: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiplayerConfig {
    pub max_concurrent_users: usize,
    pub session_timeout_minutes: u32,
    pub auto_sync_interval_seconds: f32,
    pub enable_voice_chat: bool,
    pub enable_version_control: bool,
    pub chunk_size_bytes: usize,
    pub compression_enabled: bool,
    pub encryption_enabled: bool,
}

impl Default for MultiplayerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_users: 16,
            session_timeout_minutes: 30,
            auto_sync_interval_seconds: 5.0,
            enable_voice_chat: true,
            enable_version_control: true,
            chunk_size_bytes: 64 * 1024, // 64KB chunks
            compression_enabled: true,
            encryption_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct UserId(pub String);

// Additional type definitions needed by collaboration module
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SessionId(pub String);

impl SessionId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserConstruction {
    pub id: uuid::Uuid,
    pub user_id: UserId,
    pub construction_data: Vec<u8>,
    pub timestamp: u64,
    pub construction_type: String,
    pub modified_at: u64,
    pub components: Vec<ComponentSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TerrainModification {
    pub user_id: UserId,
    pub position: [f32; 3],
    pub modification_type: TerrainModificationType,
    pub data: Vec<u8>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TerrainModificationType {
    Add,
    Remove,
    Sculpt,
    Paint,
    Plant,
}

impl UserId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn anonymous() -> Self {
        Self(format!("anon_{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub display_name: String,
    pub role: Role,
    pub connected_at: f64,
    pub last_activity: f64,
    pub location: UserLocation,
    pub status: UserStatus,
    pub capabilities: UserCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLocation {
    pub world_position: [f32; 3],
    pub view_direction: [f32; 3],
    pub current_chunk: String,
    pub selected_objects: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserStatus {
    Online,
    Away,
    Busy,
    Building,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCapabilities {
    pub can_edit_world: bool,
    pub can_use_advanced_tools: bool,
    pub can_manage_permissions: bool,
    pub can_access_voice_chat: bool,
    pub can_share_assets: bool,
    pub bandwidth_limit_mbps: Option<f32>,
}

impl Default for UserCapabilities {
    fn default() -> Self {
        Self {
            can_edit_world: true,
            can_use_advanced_tools: true,
            can_manage_permissions: false,
            can_access_voice_chat: true,
            can_share_assets: true,
            bandwidth_limit_mbps: None,
        }
    }
}

#[derive(Debug)]
pub struct MultiplayerManager {
    config: MultiplayerConfig,
    network_manager: NetworkManager,
    collaboration_manager: collaboration::CollaborationTools,
    version_control: VersionControl,
    permission_manager: PermissionManager,
    communication_manager: CommunicationManager,
    shared_asset_manager: SharedAssetManager,
    active_users: HashMap<UserId, User>,
    session_stats: SessionStats,
    start_time: Instant,
}

#[derive(Debug, Default)]
pub struct SessionStats {
    pub total_connections: u64,
    pub active_connections: u32,
    pub data_sent_bytes: u64,
    pub data_received_bytes: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub sync_operations: u64,
    pub conflicts_resolved: u64,
    pub uptime_seconds: u64,
}

impl MultiplayerManager {
    pub fn new(config: MultiplayerConfig) -> RobinResult<Self> {
        let network_config = NetworkConfig {
            max_connections: config.max_concurrent_users,
            timeout_seconds: config.session_timeout_minutes * 60,
            enable_compression: config.compression_enabled,
            enable_encryption: config.encryption_enabled,
            chunk_size: config.chunk_size_bytes,
        };

        Ok(Self {
            network_manager: NetworkManager::new(network_config)?,
            collaboration_manager: collaboration::CollaborationTools::new()?,
            version_control: VersionControl::new()?,
            permission_manager: PermissionManager::new()?,
            communication_manager: CommunicationManager::new(config.enable_voice_chat)?,
            shared_asset_manager: SharedAssetManager::new()?,
            config,
            active_users: HashMap::new(),
            session_stats: SessionStats::default(),
            start_time: Instant::now(),
        })
    }

    pub fn start_server(&mut self, bind_address: SocketAddr) -> RobinResult<()> {
        self.network_manager.start_server(bind_address)?;
        
        if self.config.enable_version_control {
            self.version_control.initialize_repository()?;
        }

        println!("Multiplayer server started on {}", bind_address);
        println!("Max concurrent users: {}", self.config.max_concurrent_users);
        println!("Version control: {}", if self.config.enable_version_control { "enabled" } else { "disabled" });
        println!("Voice chat: {}", if self.config.enable_voice_chat { "enabled" } else { "disabled" });

        Ok(())
    }

    pub fn connect_client(&mut self, server_address: SocketAddr, user_info: User) -> RobinResult<ConnectionInfo> {
        let connection_info = self.network_manager.connect_to_server(server_address)?;
        
        self.register_user(user_info)?;
        self.session_stats.total_connections += 1;
        self.session_stats.active_connections += 1;

        Ok(connection_info)
    }

    pub fn register_user(&mut self, mut user: User) -> RobinResult<()> {
        if self.active_users.len() >= self.config.max_concurrent_users {
            return Err(crate::engine::error::RobinError::NetworkError {
                operation: "register_user".to_string(),
                endpoint: "server".to_string(),
                reason: "Server at capacity".to_string()
            });
        }

        user.connected_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        user.last_activity = user.connected_at;

        let user_id = user.id.clone();
        
        // Set up user permissions
        self.permission_manager.grant_user_permissions(
            user_id.clone(), 
            UserPermissions::from_capabilities(&user.capabilities)
        )?;

        // Initialize collaboration session for user
        self.collaboration_manager.add_user(user_id.clone())?;

        // Set up communication channels
        if user.capabilities.can_access_voice_chat {
            self.communication_manager.add_user_to_voice_channel(user_id.clone())?;
        }

        self.active_users.insert(user_id, user);
        Ok(())
    }

    pub fn disconnect_user(&mut self, user_id: &UserId) -> RobinResult<()> {
        if let Some(user) = self.active_users.remove(user_id) {
            self.collaboration_manager.remove_user(user_id)?;
            self.permission_manager.revoke_user_permissions(user_id)?;
            self.communication_manager.remove_user_from_voice_channel(user_id)?;
            
            self.session_stats.active_connections = self.session_stats.active_connections.saturating_sub(1);
            
            println!("User {} ({}) disconnected after {} minutes", 
                user.username, 
                user_id.0,
                (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64() - user.connected_at) / 60.0
            );
        }
        Ok(())
    }

    pub fn update(&mut self, delta_time: f32) -> RobinResult<()> {
        self.network_manager.update(delta_time)?;
        self.collaboration_manager.update(delta_time)?;
        self.communication_manager.update(delta_time)?;
        self.shared_asset_manager.update(delta_time)?;

        // Update session statistics
        self.session_stats.uptime_seconds = self.start_time.elapsed().as_secs();
        
        // Handle periodic synchronization
        static mut LAST_SYNC_TIME: f32 = 0.0;
        unsafe {
            LAST_SYNC_TIME += delta_time;
            if LAST_SYNC_TIME >= self.config.auto_sync_interval_seconds {
                self.perform_sync_operations()?;
                LAST_SYNC_TIME = 0.0;
            }
        }

        // Check for inactive users
        self.cleanup_inactive_users()?;

        Ok(())
    }

    fn perform_sync_operations(&mut self) -> RobinResult<()> {
        let sync_count = self.collaboration_manager.sync_all_changes()?;
        self.session_stats.sync_operations += sync_count as u64;
        Ok(())
    }

    fn cleanup_inactive_users(&mut self) -> RobinResult<()> {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        
        let timeout_seconds = self.config.session_timeout_minutes as f64 * 60.0;
        let inactive_users: Vec<_> = self.active_users
            .iter()
            .filter(|(_, user)| current_time - user.last_activity > timeout_seconds)
            .map(|(id, _)| id.clone())
            .collect();

        for user_id in inactive_users {
            println!("Disconnecting inactive user: {}", user_id.0);
            self.disconnect_user(&user_id)?;
        }

        Ok(())
    }

    pub fn get_active_users(&self) -> Vec<&User> {
        self.active_users.values().collect()
    }

    pub fn get_user(&self, user_id: &UserId) -> Option<&User> {
        self.active_users.get(user_id)
    }

    pub fn update_user_location(&mut self, user_id: &UserId, location: UserLocation) -> RobinResult<()> {
        if let Some(user) = self.active_users.get_mut(user_id) {
            user.location = location;
            user.last_activity = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64();
            
            // Broadcast location update to other users
            self.collaboration_manager.broadcast_user_update(user_id, user)?;
        }
        Ok(())
    }

    pub fn update_user_status(&mut self, user_id: &UserId, status: UserStatus) -> RobinResult<()> {
        if let Some(user) = self.active_users.get_mut(user_id) {
            user.status = status;
            user.last_activity = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64();
        }
        Ok(())
    }

    pub fn get_collaboration_manager(&mut self) -> &mut collaboration::CollaborationTools {
        &mut self.collaboration_manager
    }

    pub fn get_version_control(&mut self) -> &mut VersionControl {
        &mut self.version_control
    }

    pub fn get_permission_manager(&mut self) -> &mut PermissionManager {
        &mut self.permission_manager
    }

    pub fn get_communication_manager(&mut self) -> &mut CommunicationManager {
        &mut self.communication_manager
    }

    pub fn get_shared_asset_manager(&mut self) -> &mut SharedAssetManager {
        &mut self.shared_asset_manager
    }

    pub fn get_network_stats(&self) -> NetworkStats {
        self.network_manager.get_stats()
    }

    pub fn get_session_stats(&self) -> &SessionStats {
        &self.session_stats
    }

    pub fn shutdown(&mut self) -> RobinResult<()> {
        println!("Shutting down multiplayer session...");
        
        // Disconnect all users gracefully
        let user_ids: Vec<_> = self.active_users.keys().cloned().collect();
        for user_id in user_ids {
            self.disconnect_user(&user_id)?;
        }

        // Save version control state if enabled
        if self.config.enable_version_control {
            self.version_control.save_state()?;
        }

        // Clean up network connections
        self.network_manager.shutdown()?;

        println!("Multiplayer session ended. Stats:");
        println!("  Total connections: {}", self.session_stats.total_connections);
        println!("  Data sent: {:.2} MB", self.session_stats.data_sent_bytes as f64 / 1_048_576.0);
        println!("  Data received: {:.2} MB", self.session_stats.data_received_bytes as f64 / 1_048_576.0);
        println!("  Sync operations: {}", self.session_stats.sync_operations);
        println!("  Conflicts resolved: {}", self.session_stats.conflicts_resolved);
        println!("  Uptime: {} minutes", self.session_stats.uptime_seconds / 60);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_id_creation() {
        let user_id = UserId::new("test_user".to_string());
        assert_eq!(user_id.0, "test_user");

        let anon_id = UserId::anonymous();
        assert!(anon_id.0.starts_with("anon_"));
    }

    #[test]
    fn test_multiplayer_config() {
        let config = MultiplayerConfig::default();
        assert_eq!(config.max_concurrent_users, 16);
        assert_eq!(config.session_timeout_minutes, 30);
        assert!(config.enable_voice_chat);
        assert!(config.enable_version_control);
    }

    #[test]
    fn test_user_capabilities() {
        let caps = UserCapabilities::default();
        assert!(caps.can_edit_world);
        assert!(caps.can_use_advanced_tools);
        assert!(!caps.can_manage_permissions);
        assert!(caps.can_access_voice_chat);
    }

    #[test]
    fn test_multiplayer_manager_creation() {
        let config = MultiplayerConfig::default();
        let manager = MultiplayerManager::new(config);
        assert!(manager.is_ok());
    }

    #[test]
    fn test_user_status_variants() {
        assert_eq!(UserStatus::Online, UserStatus::Online);
        assert_ne!(UserStatus::Online, UserStatus::Away);
        assert_ne!(UserStatus::Busy, UserStatus::Building);
    }
}