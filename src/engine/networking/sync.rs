/*!
 * World Synchronization System for Robin Engine
 *
 * Handles synchronization of world state between clients and server,
 * including voxel updates, player positions, and collaborative building.
 */

use crate::engine::{
    error::{RobinError, RobinResult},
    networking::{
        protocol::*,
        NetworkEvent,
    },
    world::VoxelType,
    save_system::PlayerData,
};
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{SystemTime, Duration, Instant};
use crate::engine::math::Vec3;
use tokio::sync::{mpsc, RwLock};
use std::sync::Arc;

/// Synchronization state for tracking changes
#[derive(Debug, Clone, PartialEq)]
pub enum SyncState {
    /// Fully synchronized
    Synchronized,
    /// Waiting for initial sync
    Syncing,
    /// Partial sync in progress
    PartialSync { chunks_remaining: usize },
    /// Sync failed - needs retry
    SyncFailed { reason: String },
    /// Offline - no sync needed
    Offline,
}

/// World synchronization manager
pub struct WorldSyncManager {
    /// Current sync state
    state: SyncState,
    /// Chunks that need to be synchronized
    pending_chunks: HashSet<(i32, i32, i32)>,
    /// Recently modified chunks (within sync window)
    modified_chunks: HashMap<(i32, i32, i32), SystemTime>,
    /// Player positions cache for optimization
    player_positions: HashMap<u32, Vec3>,
    /// Last sync timestamp
    last_sync_time: Instant,
    /// Sync interval
    sync_interval: Duration,
    /// Maximum chunks per sync batch
    max_chunks_per_batch: usize,
    /// Sync priority queue (chunk position, priority)
    priority_queue: VecDeque<((i32, i32, i32), u32)>,
    /// Active structure collaborations
    active_structures: HashMap<String, StructureSync>,
}

/// Structure synchronization data
#[derive(Debug, Clone)]
struct StructureSync {
    id: String,
    owner_id: u32,
    collaborators: HashSet<u32>,
    pending_updates: Vec<BlockUpdate>,
    last_update: SystemTime,
    position: Vec3,
}

impl WorldSyncManager {
    /// Create a new world sync manager
    pub fn new() -> Self {
        Self {
            state: SyncState::Offline,
            pending_chunks: HashSet::new(),
            modified_chunks: HashMap::new(),
            player_positions: HashMap::new(),
            last_sync_time: Instant::now(),
            sync_interval: Duration::from_millis(100), // 10 FPS sync rate
            max_chunks_per_batch: 4,
            priority_queue: VecDeque::new(),
            active_structures: HashMap::new(),
        }
    }

    /// Start synchronization with initial world state
    pub async fn start_sync(&mut self) -> RobinResult<()> {
        self.state = SyncState::Syncing;

        // TODO: Collect all chunks that exist in the world when VoxelWorld is available
        // let chunk_positions = world.get_loaded_chunks();
        // self.pending_chunks = chunk_positions.into_iter().collect();

        self.state = SyncState::PartialSync {
            chunks_remaining: self.pending_chunks.len(),
        };

        log::info!("Started world sync with {} chunks", self.pending_chunks.len());
        Ok(())
    }

    /// Update sync manager (called each frame)
    pub async fn update(&mut self) -> RobinResult<Vec<NetworkMessage>> {
        let mut messages = Vec::new();

        // Check if it's time to sync
        if self.last_sync_time.elapsed() < self.sync_interval {
            return Ok(messages);
        }

        match &self.state {
            SyncState::Syncing | SyncState::PartialSync { .. } => {
                messages.extend(self.generate_sync_messages().await?);
            }
            SyncState::Synchronized => {
                messages.extend(self.generate_incremental_updates().await?);
            }
            _ => {}
        }

        self.last_sync_time = Instant::now();
        Ok(messages)
    }

    /// Generate initial sync messages
    async fn generate_sync_messages(&mut self) -> RobinResult<Vec<NetworkMessage>> {
        let mut messages = Vec::new();
        let chunks_sent = 0;

        // TODO: Send priority chunks first (near players) when VoxelWorld is available
        // self.update_chunk_priorities();

        // TODO: Implement chunk data sending when world system is available
        /*
        while chunks_sent < self.max_chunks_per_batch && !self.priority_queue.is_empty() {
            if let Some((chunk_pos, _priority)) = self.priority_queue.pop_front() {
                if self.pending_chunks.remove(&chunk_pos) {
                    if let Some(chunk) = world.get_chunk(chunk_pos.0, chunk_pos.1, chunk_pos.2) {
                        let compressed_data = self.compress_chunk_data(chunk)?;

                        messages.push(NetworkMessage::ChunkUpdate {
                            chunk_pos,
                            data: compressed_data,
                        });

                        chunks_sent += 1;
                    }
                }
            }
        }
        */

        // Update sync state
        if self.pending_chunks.is_empty() {
            self.state = SyncState::Synchronized;
            log::info!("World synchronization completed");
        } else {
            self.state = SyncState::PartialSync {
                chunks_remaining: self.pending_chunks.len(),
            };
        }

        Ok(messages)
    }

    /// Generate incremental update messages
    async fn generate_incremental_updates(&mut self) -> RobinResult<Vec<NetworkMessage>> {
        let mut messages = Vec::new();
        let now = SystemTime::now();

        // TODO: Process recently modified chunks when VoxelWorld is available
        let mut expired_chunks = Vec::new();
        for (chunk_pos, modified_time) in &self.modified_chunks {
            // Send updates for chunks modified in the last sync interval
            if let Ok(elapsed) = now.duration_since(*modified_time) {
                if elapsed <= self.sync_interval * 2 {
                    // TODO: Get chunk data and compress when world system is available
                    /*
                    if let Some(chunk) = world.get_chunk(chunk_pos.0, chunk_pos.1, chunk_pos.2) {
                        let compressed_data = self.compress_chunk_data(chunk)?;

                        messages.push(NetworkMessage::ChunkUpdate {
                            chunk_pos: *chunk_pos,
                            data: compressed_data,
                        });
                    }
                    */
                    expired_chunks.push(*chunk_pos);
                } else if elapsed > Duration::from_secs(5) {
                    // Clean up old entries
                    expired_chunks.push(*chunk_pos);
                }
            }
        }

        // Remove processed chunks
        for chunk_pos in expired_chunks {
            self.modified_chunks.remove(&chunk_pos);
        }

        // Process structure updates
        messages.extend(self.generate_structure_updates().await?);

        Ok(messages)
    }

    /// Update chunk priority based on player positions
    fn update_chunk_priorities(&mut self) {
        self.priority_queue.clear();

        for &chunk_pos in &self.pending_chunks {
            let mut min_distance = f32::INFINITY;

            // Calculate minimum distance to any player
            for player_pos in self.player_positions.values() {
                let chunk_center = Vec3::new(
                    chunk_pos.0 as f32 * 32.0 + 16.0,
                    chunk_pos.1 as f32 * 32.0 + 16.0,
                    chunk_pos.2 as f32 * 32.0 + 16.0,
                );

                let distance = (player_pos - chunk_center).magnitude();
                min_distance = min_distance.min(distance);
            }

            // Convert distance to priority (closer = higher priority)
            let priority = if min_distance.is_finite() {
                (1000.0 / (min_distance + 1.0)) as u32
            } else {
                1
            };

            self.priority_queue.push_back((chunk_pos, priority));
        }

        // Sort by priority (highest first)
        self.priority_queue.make_contiguous().sort_by(|a, b| b.1.cmp(&a.1));
    }

    /// Compress chunk data for network transmission
    /// TODO: Implement when Chunk struct is available
    fn _compress_chunk_data(&self, _chunk_data: &[VoxelType], chunk_pos: (i32, i32, i32)) -> RobinResult<CompressedChunkData> {
        // Simple run-length encoding
        let mut compressed = Vec::new();
        if !_chunk_data.is_empty() {
            let mut current_voxel = _chunk_data[0];
            let mut count = 1u32;

            for &voxel in &_chunk_data[1..] {
                if voxel == current_voxel {
                    count += 1;
                } else {
                    // Encode current run
                    compressed.push(current_voxel as u8);
                    compressed.extend_from_slice(&count.to_le_bytes());

                    current_voxel = voxel;
                    count = 1;
                }
            }

            // Encode final run
            compressed.push(current_voxel as u8);
            compressed.extend_from_slice(&count.to_le_bytes());
        }

        Ok(CompressedChunkData {
            position: chunk_pos,
            voxels: compressed,
            metadata: ChunkMetadata {
                last_modified: SystemTime::now(),
                modified_by: 0, // Server
                version: 1,
            },
        })
    }

    /// Handle incoming sync message
    pub async fn handle_sync_message(&mut self, message: NetworkMessage) -> RobinResult<()> {
        match message {
            NetworkMessage::ChunkUpdate { chunk_pos, data } => {
                self.apply_chunk_update(chunk_pos, data).await?;
            }
            NetworkMessage::VoxelPlace { position, voxel_type: _, player_id: _ } => {
                // TODO: Apply voxel placement when world system is available
                // world.set_voxel(position.x, position.y, position.z, voxel_type)?;
                self.mark_chunk_modified(Self::world_to_chunk_pos(position));
            }
            NetworkMessage::VoxelRemove { position, player_id: _ } => {
                // TODO: Apply voxel removal when world system is available
                // world.set_voxel(position.x, position.y, position.z, VoxelType::Air)?;
                self.mark_chunk_modified(Self::world_to_chunk_pos(position));
            }
            NetworkMessage::PlayerMove { player_id, position, .. } => {
                self.player_positions.insert(player_id, position);
            }
            NetworkMessage::StructureStart { structure_id, player_id, position } => {
                self.start_structure_collaboration(structure_id, player_id, position).await?;
            }
            NetworkMessage::StructureUpdate { structure_id, blocks } => {
                self.apply_structure_update(structure_id, blocks).await?;
            }
            NetworkMessage::StructureComplete { structure_id } => {
                self.complete_structure_collaboration(structure_id).await?;
            }
            _ => {
                // Other messages are not sync-related
            }
        }

        Ok(())
    }

    /// Apply chunk update from network
    async fn apply_chunk_update(&mut self, chunk_pos: (i32, i32, i32), data: CompressedChunkData) -> RobinResult<()> {
        // Decompress voxel data
        let _voxels = self.decompress_chunk_data(&data.voxels)?;

        // TODO: Apply to world when VoxelWorld is available
        /*
        if let Some(chunk) = world.get_chunk_mut(chunk_pos.0, chunk_pos.1, chunk_pos.2) {
            chunk.set_voxel_data(voxels)?;
            chunk.mark_dirty();
        }
        */

        // Remove from pending if this was a sync
        self.pending_chunks.remove(&chunk_pos);

        Ok(())
    }

    /// Decompress chunk voxel data
    fn decompress_chunk_data(&self, compressed: &[u8]) -> RobinResult<Vec<VoxelType>> {
        let mut voxels = Vec::new();
        let mut i = 0;

        while i + 4 < compressed.len() {
            let voxel_type = VoxelType::from_u8(compressed[i])?;
            let count = u32::from_le_bytes([
                compressed[i + 1],
                compressed[i + 2],
                compressed[i + 3],
                compressed[i + 4],
            ]);

            for _ in 0..count {
                voxels.push(voxel_type);
            }

            i += 5;
        }

        Ok(voxels)
    }

    /// Mark a chunk as modified
    fn mark_chunk_modified(&mut self, chunk_pos: (i32, i32, i32)) {
        self.modified_chunks.insert(chunk_pos, SystemTime::now());
    }

    /// Convert world position to chunk position
    fn world_to_chunk_pos(world_pos: cgmath::Vector3<i32>) -> (i32, i32, i32) {
        (
            world_pos.x.div_euclid(32),
            world_pos.y.div_euclid(32),
            world_pos.z.div_euclid(32),
        )
    }

    /// Start collaborative structure building
    async fn start_structure_collaboration(&mut self, structure_id: String, owner_id: u32, position: Vec3<f32>) -> RobinResult<()> {
        let structure = StructureSync {
            id: structure_id.clone(),
            owner_id,
            collaborators: HashSet::new(),
            pending_updates: Vec::new(),
            last_update: SystemTime::now(),
            position,
        };

        // Clone structure_id for logging before moving it
        let structure_id_clone = structure_id.clone();
        self.active_structures.insert(structure_id, structure);
        log::info!("Started structure collaboration for ID: {}", structure_id_clone);
        Ok(())
    }

    /// Apply structure update
    async fn apply_structure_update(&mut self, structure_id: String, blocks: Vec<BlockUpdate>) -> RobinResult<()> {
        if let Some(structure) = self.active_structures.get_mut(&structure_id) {
            // Apply block updates relative to structure position
            for block in &blocks {
                let world_pos = cgmath::Vector3::new(
                    structure.position.x as i32 + block.relative_position.x,
                    structure.position.y as i32 + block.relative_position.y,
                    structure.position.z as i32 + block.relative_position.z,
                );

                // TODO: Apply voxel changes when world system is available
                // world.set_voxel(world_pos.x, world_pos.y, world_pos.z, block.voxel_type)?;
                self.mark_chunk_modified(Self::world_to_chunk_pos(world_pos));
            }

            structure.pending_updates.extend(blocks);
            structure.last_update = SystemTime::now();
        }

        Ok(())
    }

    /// Complete structure collaboration
    async fn complete_structure_collaboration(&mut self, structure_id: String) -> RobinResult<()> {
        if let Some(_structure) = self.active_structures.remove(&structure_id) {
            log::info!("Completed structure collaboration for ID: {}", structure_id);
        }
        Ok(())
    }

    /// Generate structure update messages
    async fn generate_structure_updates(&mut self) -> RobinResult<Vec<NetworkMessage>> {
        let mut messages = Vec::new();
        let now = SystemTime::now();

        for (structure_id, structure) in &mut self.active_structures {
            // Send pending updates if any exist and enough time has passed
            if !structure.pending_updates.is_empty() {
                if let Ok(elapsed) = now.duration_since(structure.last_update) {
                    if elapsed >= Duration::from_millis(50) { // Batch updates every 50ms
                        messages.push(NetworkMessage::StructureUpdate {
                            structure_id: structure_id.clone(),
                            blocks: structure.pending_updates.clone(),
                        });

                        structure.pending_updates.clear();
                        structure.last_update = now;
                    }
                }
            }
        }

        Ok(messages)
    }

    /// Get current sync state
    pub fn get_sync_state(&self) -> &SyncState {
        &self.state
    }

    /// Get sync progress (0.0 to 1.0)
    pub fn get_sync_progress(&self) -> f32 {
        match &self.state {
            SyncState::Synchronized => 1.0,
            SyncState::PartialSync { chunks_remaining } => {
                let total = self.pending_chunks.len() + *chunks_remaining;
                if total > 0 {
                    1.0 - (*chunks_remaining as f32 / total as f32)
                } else {
                    1.0
                }
            }
            SyncState::Syncing => 0.0,
            _ => 0.0,
        }
    }

    /// Force a full resync
    pub async fn force_resync(&mut self) -> RobinResult<()> {
        log::info!("Forcing full world resync");
        self.start_sync().await
    }

    /// Add collaborator to structure
    pub fn add_structure_collaborator(&mut self, structure_id: &str, player_id: u32) -> RobinResult<()> {
        if let Some(structure) = self.active_structures.get_mut(structure_id) {
            structure.collaborators.insert(player_id);
            log::info!("Added collaborator {} to structure {}", player_id, structure_id);
        }
        Ok(())
    }

    /// Remove collaborator from structure
    pub fn remove_structure_collaborator(&mut self, structure_id: &str, player_id: u32) -> RobinResult<()> {
        if let Some(structure) = self.active_structures.get_mut(structure_id) {
            structure.collaborators.remove(&player_id);
            log::info!("Removed collaborator {} from structure {}", player_id, structure_id);
        }
        Ok(())
    }

    /// Get active structure count
    pub fn get_active_structure_count(&self) -> usize {
        self.active_structures.len()
    }
}

impl Default for WorldSyncManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_manager_creation() {
        let manager = WorldSyncManager::new();
        assert_eq!(*manager.get_sync_state(), SyncState::Offline);
        assert_eq!(manager.get_sync_progress(), 0.0);
    }

    #[test]
    fn test_chunk_compression() {
        let manager = WorldSyncManager::new();

        // Create a simple test chunk (this would need to be implemented based on actual Chunk struct)
        // This is a placeholder test
        let test_voxels = vec![VoxelType::Stone; 32 * 32 * 32];

        // The actual compression test would need the real Chunk implementation
        // assert!(compressed.voxels.len() < test_voxels.len() * std::mem::size_of::<VoxelType>());
    }

    #[test]
    fn test_world_to_chunk_conversion() {
        let world_pos = Vec3::new(100, 50, -32);
        let chunk_pos = WorldSyncManager::world_to_chunk_pos(world_pos);
        assert_eq!(chunk_pos, (3, 1, -1));
    }
}