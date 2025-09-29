/*!
 * Network Protocol for Robin Engine
 *
 * Defines the message format and communication protocol for multiplayer.
 */

use crate::engine::{
    world::VoxelType,
    save_system::PlayerData,
};
use serde::{Serialize, Deserialize};
use crate::engine::math::Vec3;
use std::time::SystemTime;

/// Network message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    /// Player connection/disconnection
    PlayerJoin { player_id: u32, player_data: PlayerData },
    PlayerLeave { player_id: u32 },

    /// Player movement and actions
    PlayerMove { player_id: u32, position: Vec3, rotation: Vec3 },
    PlayerAction { player_id: u32, action: PlayerAction },

    /// Voxel world updates
    VoxelPlace { position: cgmath::Vector3<i32>, voxel_type: VoxelType, player_id: u32 },
    VoxelRemove { position: cgmath::Vector3<i32>, player_id: u32 },
    ChunkUpdate { chunk_pos: (i32, i32, i32), data: CompressedChunkData },

    /// Chat and communication
    ChatMessage { player_id: u32, message: String, timestamp: SystemTime },
    ServerMessage { message: String, severity: MessageSeverity },

    /// Synchronization
    WorldSync { chunks: Vec<CompressedChunkData>, players: Vec<PlayerData> },
    Heartbeat { timestamp: SystemTime },
    Ping { timestamp: SystemTime },
    Pong { timestamp: SystemTime },

    /// Building collaboration
    StructureStart { structure_id: String, player_id: u32, position: Vec3 },
    StructureUpdate { structure_id: String, blocks: Vec<BlockUpdate> },
    StructureComplete { structure_id: String },

    /// Server control
    ServerShutdown { reason: String },
    KickPlayer { player_id: u32, reason: String },
}

/// Player actions that can be performed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlayerAction {
    /// Basic actions
    Jump,
    Crouch,
    Sprint,

    /// Building actions
    StartBuilding,
    StopBuilding,
    SelectMaterial(VoxelType),
    SelectBuildMode(BuildMode),

    /// Tool actions
    UseTool(ToolType),
    SwitchTool(ToolType),

    /// Emotes and gestures
    Emote(EmoteType),
    Point(Vec3),
}

/// Network events that occur
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    /// Connection events
    Connected { player_id: u32 },
    Disconnected { player_id: u32 },
    ConnectionLost,

    /// Player events
    PlayerJoined { player_id: u32, name: String },
    PlayerLeft { player_id: u32, name: String },
    PlayerMoved { player_id: u32, position: Vec3 },

    /// World events
    VoxelPlaced { position: cgmath::Vector3<i32>, voxel_type: VoxelType, player_id: u32 },
    VoxelRemoved { position: cgmath::Vector3<i32>, player_id: u32 },
    ChunkModified { chunk_pos: (i32, i32, i32) },

    /// Chat events
    ChatReceived { player_id: u32, message: String },
    ServerMessageReceived { message: String },

    /// Sync events
    WorldSyncReceived,
    PingUpdate { ping_ms: u32 },
}

/// Build modes for collaborative construction
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BuildMode {
    Single,
    Line,
    Plane,
    Box,
    Sphere,
    Template(u32),
}

/// Tool types available to players
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ToolType {
    Pickaxe,
    Hammer,
    Paintbrush,
    Ruler,
    Blueprint,
}

/// Emote types for player expression
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EmoteType {
    Wave,
    ThumbsUp,
    ThumbsDown,
    Dance,
    Cheer,
    Confused,
}

/// Message severity levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MessageSeverity {
    Info,
    Warning,
    Error,
    System,
}

/// Compressed chunk data for network transmission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedChunkData {
    pub position: (i32, i32, i32),
    pub voxels: Vec<u8>, // Compressed voxel data
    pub metadata: ChunkMetadata,
}

/// Chunk metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMetadata {
    pub last_modified: SystemTime,
    pub modified_by: u32,
    pub version: u32,
}

/// Block update for structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockUpdate {
    pub relative_position: cgmath::Vector3<i32>,
    pub voxel_type: VoxelType,
}

/// Network packet for transmission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPacket {
    pub sequence_id: u64,
    pub timestamp: SystemTime,
    pub message: NetworkMessage,
}

impl NetworkPacket {
    pub fn new(sequence_id: u64, message: NetworkMessage) -> Self {
        Self {
            sequence_id,
            timestamp: SystemTime::now(),
            message,
        }
    }

    /// Serialize packet to bytes
    pub fn to_bytes(&self) -> RobinResult<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| RobinError::NetworkError(format!("Failed to serialize packet: {}", e)))
    }

    /// Deserialize packet from bytes
    pub fn from_bytes(bytes: &[u8]) -> RobinResult<Self> {
        bincode::deserialize(bytes)
            .map_err(|e| RobinError::NetworkError(format!("Failed to deserialize packet: {}", e)))
    }
}

/// Connection handshake protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Handshake {
    pub version: u32,
    pub player_name: String,
    pub password: Option<String>,
}

/// Handshake response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HandshakeResponse {
    Accepted { player_id: u32, world_seed: u64 },
    Rejected { reason: String },
    RequiresPassword,
}

use crate::engine::error::{RobinError, RobinResult};

/// Protocol version for compatibility checking
pub const PROTOCOL_VERSION: u32 = 1;

/// Maximum packet size (1 MB)
pub const MAX_PACKET_SIZE: usize = 1024 * 1024;

/// Connection timeout in seconds
pub const CONNECTION_TIMEOUT: u64 = 30;

/// Heartbeat interval in seconds
pub const HEARTBEAT_INTERVAL: u64 = 5;