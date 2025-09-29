/*!
 * World Serialization System for Robin Engine
 *
 * Handles saving and loading of voxel worlds, including chunk data,
 * player-built structures, and world state.
 */

use crate::engine::{
    error::{RobinError, RobinResult},
    world::VoxelType,
    save_system::PlayerData,
    // TODO: Re-enable when VoxelWorld, Chunk, and SaveData are available
    // world::{Chunk, VoxelWorld},
    // save_system::SaveData,
};
use serde::{Serialize, Deserialize};
use std::fs::{self, File};
use std::io::{Write, Read, BufWriter, BufReader};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use crate::engine::math::Vec3;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use flate2::Compression;

/// World save manager
pub struct WorldSaveManager {
    save_directory: PathBuf,
    compression_level: Compression,
    auto_save_interval: Option<std::time::Duration>,
    last_save_time: std::time::Instant,
}

impl WorldSaveManager {
    pub fn new(save_directory: impl AsRef<Path>) -> Self {
        Self {
            save_directory: save_directory.as_ref().to_path_buf(),
            compression_level: Compression::default(),
            auto_save_interval: Some(std::time::Duration::from_secs(300)), // 5 minutes
            last_save_time: std::time::Instant::now(),
        }
    }

    /// Save the entire world to disk
    /// TODO: Re-implement when VoxelWorld is available
    pub fn save_world(&mut self, world_name: &str, _world_data: &[u8], player_data: &PlayerData) -> RobinResult<()> {
        println!("💾 Saving world: {}", world_name);

        // Create world directory
        let world_path = self.save_directory.join(world_name);
        fs::create_dir_all(&world_path)?;

        // Save world metadata
        let metadata = WorldMetadata {
            version: 1,
            name: world_name.to_string(),
            created_at: std::time::SystemTime::now(),
            last_modified: std::time::SystemTime::now(),
            seed: 12345, // TODO: Get from world when VoxelWorld is available
            player_position: Vec3::new(player_data.position.0, player_data.position.1, 0.0),
            // TODO: Fix when PlayerData has rotation field
            // player_rotation: player_data.rotation,
            play_time: player_data.stats.time_played,
            world_bounds: (cgmath::Vector3::new(0, 0, 0), cgmath::Vector3::new(1000, 256, 1000)), // TODO: Get from world when VoxelWorld is available
        };

        self.save_metadata(&world_path, &metadata)?;

        // Save chunks
        // TODO: Save chunks when VoxelWorld is available
        // self.save_chunks(&world_path, _world_data)?;

        // Save player data
        self.save_player_data(&world_path, player_data)?;

        // Save structures and entities
        // TODO: Save structures when VoxelWorld is available
        // self.save_structures(&world_path, _world_data)?;

        self.last_save_time = std::time::Instant::now();
        println!("✅ World saved successfully");

        Ok(())
    }

    /// Load a world from disk
    /// TODO: Re-implement when VoxelWorld is available
    pub fn load_world(&self, world_name: &str) -> RobinResult<(Vec<u8>, PlayerData, WorldMetadata)> {
        println!("📂 Loading world: {}", world_name);

        let world_path = self.save_directory.join(world_name);

        if !world_path.exists() {
            return Err(RobinError::FileNotFound(std::path::PathBuf::from(format!("World '{}' not found", world_name))));
        }

        // Load metadata
        let metadata = self.load_metadata(&world_path)?;

        // Create world with saved seed
        // TODO: Create VoxelWorld when available
        // let mut world = VoxelWorld::new_with_seed(metadata.seed);
        let world_data = Vec::new();

        // Load chunks
        // TODO: Load chunks when VoxelWorld is available
        // self.load_chunks(&world_path, &mut world_data)?;

        // Load player data
        let mut player_data = self.load_player_data(&world_path)?;
        player_data.position = (metadata.player_position.x, metadata.player_position.y);
        // TODO: Handle player rotation when PlayerData has rotation field
        // player_data.rotation = metadata.player_rotation;

        // Load structures
        // TODO: Load structures when VoxelWorld is available
        // self.load_structures(&world_path, &mut world_data)?;

        println!("✅ World loaded successfully");

        Ok((world_data, player_data, metadata))
    }

    /// Save world metadata
    fn save_metadata(&self, world_path: &Path, metadata: &WorldMetadata) -> RobinResult<()> {
        let metadata_path = world_path.join("metadata.json");
        let file = File::create(metadata_path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, metadata)?;
        Ok(())
    }

    /// Load world metadata
    fn load_metadata(&self, world_path: &Path) -> RobinResult<WorldMetadata> {
        let metadata_path = world_path.join("metadata.json");
        let file = File::open(metadata_path)?;
        let reader = BufReader::new(file);
        let metadata = serde_json::from_reader(reader)?;
        Ok(metadata)
    }

    /// Save all chunks to disk
    /// TODO: Re-implement when VoxelWorld is available
    fn save_chunks(&self, world_path: &Path, _world_data: &[u8]) -> RobinResult<()> {
        let chunks_dir = world_path.join("chunks");
        fs::create_dir_all(&chunks_dir)?;

        // TODO: Get all chunks that have been modified when VoxelWorld is available
        // let chunks = world.get_all_chunks();
        // let total_chunks = chunks.len();

        println!("  Saving chunks... (stub implementation)");

        // TODO: Implement chunk saving when VoxelWorld is available
        // for (chunk_pos, chunk) in chunks {
        //     // Only save non-empty chunks
        //     if !chunk.is_empty() {
        //         let chunk_data = ChunkData::from_chunk(chunk);
        //         self.save_chunk(&chunks_dir, chunk_pos, &chunk_data)?;
        //     }
        // }

        Ok(())
    }

    /// Save a single chunk
    fn save_chunk(&self, chunks_dir: &Path, pos: (i32, i32, i32), chunk_data: &ChunkData) -> RobinResult<()> {
        let filename = format!("chunk_{}_{}_{}.chunk.gz",
            pos.0, pos.1, pos.2);
        let chunk_path = chunks_dir.join(filename);

        // Serialize chunk data
        let serialized = bincode::serialize(chunk_data)?;

        // Compress and write
        let file = File::create(chunk_path)?;
        let mut encoder = GzEncoder::new(file, self.compression_level);
        encoder.write_all(&serialized)?;
        encoder.finish()?;

        Ok(())
    }

    /// Load all chunks from disk
    /// TODO: Re-implement when VoxelWorld is available
    fn load_chunks(&self, world_path: &Path, _world_data: &mut Vec<u8>) -> RobinResult<()> {
        let chunks_dir = world_path.join("chunks");

        if !chunks_dir.exists() {
            return Ok(());
        }

        let entries = fs::read_dir(chunks_dir)?;
        let mut loaded_count = 0;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("gz") {
                if let Some(_chunk_data) = self.load_chunk(&path)? {
                    // Parse position from filename
                    if let Some(_pos) = Self::parse_chunk_position(&path) {
                        // TODO: Load chunk data when VoxelWorld is available
                        // world.load_chunk_data(pos, chunk_data.to_chunk());
                        loaded_count += 1;
                    }
                }
            }
        }

        println!("  Loaded {} chunks (stub implementation)", loaded_count);
        Ok(())
    }

    /// Load a single chunk
    fn load_chunk(&self, path: &Path) -> RobinResult<Option<ChunkData>> {
        let file = File::open(path)?;
        let decoder = GzDecoder::new(file);
        let mut reader = BufReader::new(decoder);

        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;

        let chunk_data: ChunkData = bincode::deserialize(&buffer)?;
        Ok(Some(chunk_data))
    }

    /// Parse chunk position from filename
    fn parse_chunk_position(path: &Path) -> Option<(i32, i32, i32)> {
        let filename = path.file_stem()?.to_str()?;
        let parts: Vec<&str> = filename.split('_').collect();

        if parts.len() >= 4 && parts[0] == "chunk" {
            let x = parts[1].parse().ok()?;
            let y = parts[2].parse().ok()?;
            let z = parts[3].parse().ok()?;
            Some((x, y, z))
        } else {
            None
        }
    }

    /// Save player data
    fn save_player_data(&self, world_path: &Path, player_data: &PlayerData) -> RobinResult<()> {
        let player_path = world_path.join("player.json");
        let file = File::create(player_path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, player_data)?;
        Ok(())
    }

    /// Load player data
    fn load_player_data(&self, world_path: &Path) -> RobinResult<PlayerData> {
        let player_path = world_path.join("player.json");

        if !player_path.exists() {
            // Return default player data if file doesn't exist
            return Ok(PlayerData::default());
        }

        let file = File::open(player_path)?;
        let reader = BufReader::new(file);
        let player_data = serde_json::from_reader(reader)?;
        Ok(player_data)
    }

    /// Save structures (player-built constructions)
    /// TODO: Re-implement when VoxelWorld is available
    fn save_structures(&self, world_path: &Path, _world_data: &[u8]) -> RobinResult<()> {
        let structures_path = world_path.join("structures.json");

        // TODO: Get all structures from the world when VoxelWorld is available
        // let structures = world.get_structures();
        let structures: Vec<Structure> = Vec::new(); // Placeholder

        if structures.is_empty() {
            return Ok(());
        }

        let file = File::create(structures_path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &structures)?;

        println!("  Saved {} structures (stub implementation)", structures.len());
        Ok(())
    }

    /// Load structures
    /// TODO: Re-implement when VoxelWorld is available
    fn load_structures(&self, world_path: &Path, _world_data: &mut Vec<u8>) -> RobinResult<()> {
        let structures_path = world_path.join("structures.json");

        if !structures_path.exists() {
            return Ok(());
        }

        let file = File::open(structures_path)?;
        let reader = BufReader::new(file);
        let structures: Vec<Structure> = serde_json::from_reader(reader)?;

        // TODO: Add structures to world when VoxelWorld is available
        // for structure in structures {
        //     world.add_structure(structure);
        // }

        println!("  Loaded {} structures (stub implementation)", structures.len());
        Ok(())
    }

    /// Check if auto-save is needed
    pub fn should_auto_save(&self) -> bool {
        if let Some(interval) = self.auto_save_interval {
            self.last_save_time.elapsed() >= interval
        } else {
            false
        }
    }

    /// List all saved worlds
    pub fn list_worlds(&self) -> RobinResult<Vec<WorldInfo>> {
        let mut worlds = Vec::new();

        if !self.save_directory.exists() {
            return Ok(worlds);
        }

        for entry in fs::read_dir(&self.save_directory)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let metadata_path = path.join("metadata.json");

                if metadata_path.exists() {
                    if let Ok(metadata) = self.load_metadata(&path) {
                        worlds.push(WorldInfo {
                            name: metadata.name,
                            created_at: metadata.created_at,
                            last_modified: metadata.last_modified,
                            play_time: metadata.play_time,
                            size: Self::calculate_world_size(&path),
                        });
                    }
                }
            }
        }

        Ok(worlds)
    }

    /// Calculate world size on disk
    fn calculate_world_size(path: &Path) -> u64 {
        let mut size = 0;

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    if let Ok(metadata) = entry.metadata() {
                        size += metadata.len();
                    }
                }
            }
        }

        size
    }

    /// Delete a saved world
    pub fn delete_world(&self, world_name: &str) -> RobinResult<()> {
        let world_path = self.save_directory.join(world_name);

        if world_path.exists() {
            fs::remove_dir_all(world_path)?;
            println!("🗑️ Deleted world: {}", world_name);
        }

        Ok(())
    }

    /// Create a backup of a world
    pub fn backup_world(&self, world_name: &str) -> RobinResult<String> {
        let source_path = self.save_directory.join(world_name);

        if !source_path.exists() {
            return Err(RobinError::FileNotFound(std::path::PathBuf::from(format!("World '{}' not found", world_name))));
        }

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!("{}_{}", world_name, timestamp);
        let backup_path = self.save_directory.join(&backup_name);

        // Copy directory recursively
        Self::copy_dir_recursive(&source_path, &backup_path)?;

        println!("📦 Created backup: {}", backup_name);
        Ok(backup_name)
    }

    /// Copy directory recursively
    fn copy_dir_recursive(source: &Path, dest: &Path) -> RobinResult<()> {
        fs::create_dir_all(dest)?;

        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let source_path = entry.path();
            let dest_path = dest.join(entry.file_name());

            if source_path.is_dir() {
                Self::copy_dir_recursive(&source_path, &dest_path)?;
            } else {
                fs::copy(&source_path, &dest_path)?;
            }
        }

        Ok(())
    }
}

/// World metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldMetadata {
    pub version: u32,
    pub name: String,
    pub created_at: std::time::SystemTime,
    pub last_modified: std::time::SystemTime,
    pub seed: u64,
    pub player_position: Vec3,
    pub player_rotation: Vec3,
    pub play_time: u64,
    pub world_bounds: (cgmath::Vector3<i32>, cgmath::Vector3<i32>),
}

/// Compressed chunk data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkData {
    pub position: (i32, i32, i32),
    pub voxels: Vec<CompressedVoxel>,
    pub metadata: HashMap<String, String>,
}

impl ChunkData {
    /// Create from a chunk
    /// TODO: Re-implement when Chunk is available
    pub fn from_chunk(_chunk_data: &[u8]) -> Self {
        // TODO: Implement compression when Chunk type is available
        // let mut voxels = Vec::new();

        // Compress voxels using run-length encoding
        // let chunk_voxels = chunk.get_voxels();
        // let mut current_type = chunk_voxels[0];
        // let mut count = 1;

        // for voxel in chunk_voxels.iter().skip(1) {
        //     if *voxel == current_type {
        //         count += 1;
        //     } else {
        //         voxels.push(CompressedVoxel {
        //             voxel_type: current_type,
        //             count,
        //         });
        //         current_type = *voxel;
        //         count = 1;
        //     }
        // }

        // // Don't forget the last run
        // voxels.push(CompressedVoxel {
        //     voxel_type: current_type,
        //     count,
        // });

        // Placeholder implementation
        Self {
            position: (0, 0, 0), // TODO: Get from chunk when available
            voxels: Vec::new(),  // TODO: Compress voxels when available
            metadata: HashMap::new(),
        }
    }

    /// Convert back to a chunk
    /// TODO: Re-implement when Chunk is available
    pub fn to_chunk(&self) -> Vec<u8> {
        // TODO: Create and populate chunk when Chunk type is available
        // let mut chunk = Chunk::new(self.position);

        // Decompress voxels
        // let mut index = 0;
        // for compressed in &self.voxels {
        //     for _ in 0..compressed.count {
        //         if index < 32 * 32 * 32 {
        //             let x = index % 32;
        //             let y = (index / 32) % 32;
        //             let z = index / (32 * 32);
        //             chunk.set_voxel((x, y, z), compressed.voxel_type);
        //         }
        //         index += 1;
        //     }
        // }

        // Placeholder implementation
        Vec::new() // TODO: Return actual chunk data when Chunk type is available
    }
}

/// Compressed voxel representation
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CompressedVoxel {
    pub voxel_type: VoxelType,
    pub count: u32,
}

/// Structure representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Structure {
    pub id: String,
    pub name: String,
    pub position: Vec3,
    pub size: cgmath::Vector3<u32>,
    pub blocks: Vec<StructureBlock>,
    pub metadata: HashMap<String, String>,
}

/// Block in a structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureBlock {
    pub relative_position: cgmath::Vector3<i32>,
    pub voxel_type: VoxelType,
}

/// World information for listing
#[derive(Debug, Clone)]
pub struct WorldInfo {
    pub name: String,
    pub created_at: std::time::SystemTime,
    pub last_modified: std::time::SystemTime,
    pub play_time: u64,
    pub size: u64,
}

impl Default for WorldSaveManager {
    fn default() -> Self {
        Self::new("saves")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_compression() {
        // TODO: Create chunk when Chunk type is available
        // let mut chunk = Chunk::new((0, 0, 0));

        // Fill chunk with pattern
        for x in 0..32 {
            for y in 0..32 {
                for z in 0..32 {
                    let voxel_type = if (x + y + z) % 2 == 0 {
                        VoxelType::Stone
                    } else {
                        VoxelType::Air
                    };
                    chunk.set_voxel((x, y, z), voxel_type);
                }
            }
        }

        // Compress and decompress
        // TODO: Create chunk data when Chunk type is available
        // let chunk_data = ChunkData::from_chunk(&chunk);
        let restored = chunk_data.to_chunk();

        // Verify
        for x in 0..32 {
            for y in 0..32 {
                for z in 0..32 {
                    assert_eq!(
                        chunk.get_voxel((x, y, z)),
                        restored.get_voxel((x, y, z))
                    );
                }
            }
        }
    }

    #[test]
    fn test_world_save_load() {
        let temp_dir = std::env::temp_dir().join("robin_test_saves");
        let manager = WorldSaveManager::new(&temp_dir);

        // Create test world
        let world = VoxelWorld::new();
        let player_data = PlayerData::default();

        // Save
        manager.save_world("test_world", &world, &player_data).unwrap();

        // Load
        let (loaded_world, loaded_player, metadata) =
            manager.load_world("test_world").unwrap();

        assert_eq!(metadata.name, "test_world");

        // Cleanup
        let _ = fs::remove_dir_all(temp_dir);
    }
}