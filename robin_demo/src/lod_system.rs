// Level of Detail (LOD) system for voxel chunks
// Optimizes rendering by reducing detail for distant chunks

use cgmath::{Vector3, InnerSpace};
use std::collections::HashMap;
use crate::culling::ChunkId;

/// Level of detail for chunk rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LodLevel {
    /// Full detail - all voxels rendered
    High = 0,
    /// Medium detail - 2x2x2 voxel groups simplified
    Medium = 1,
    /// Low detail - 4x4x4 voxel groups simplified
    Low = 2,
    /// Minimal detail - 8x8x8 voxel groups simplified
    Minimal = 3,
}

impl LodLevel {
    /// Get the simplification factor for this LOD level
    pub fn simplification_factor(&self) -> u32 {
        match self {
            LodLevel::High => 1,
            LodLevel::Medium => 2,
            LodLevel::Low => 4,
            LodLevel::Minimal => 8,
        }
    }

    /// Get the maximum distance for this LOD level
    pub fn max_distance(&self) -> f32 {
        match self {
            LodLevel::High => 128.0,      // 4 chunks at 32³ chunk size
            LodLevel::Medium => 256.0,    // 8 chunks
            LodLevel::Low => 512.0,       // 16 chunks
            LodLevel::Minimal => 1024.0,  // 32 chunks
        }
    }

    /// Get vertex reduction factor (approximate)
    pub fn vertex_reduction(&self) -> f32 {
        match self {
            LodLevel::High => 1.0,
            LodLevel::Medium => 0.25,    // 75% reduction
            LodLevel::Low => 0.0625,     // 93.75% reduction
            LodLevel::Minimal => 0.0156, // 98.44% reduction
        }
    }

    /// Get all LOD levels sorted by quality (high to low)
    pub fn all_levels() -> [LodLevel; 4] {
        [LodLevel::High, LodLevel::Medium, LodLevel::Low, LodLevel::Minimal]
    }
}

/// Configuration for the LOD system
#[derive(Debug, Clone)]
pub struct LodConfig {
    /// Enable/disable LOD system
    pub enabled: bool,
    /// Distance bias factor (higher = more aggressive LOD)
    pub distance_bias: f32,
    /// Hysteresis factor to prevent LOD flickering
    pub hysteresis_factor: f32,
    /// Maximum chunks to process per frame
    pub max_updates_per_frame: usize,
    /// Force minimum LOD level for performance
    pub min_lod_level: Option<LodLevel>,
}

impl Default for LodConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            distance_bias: 1.0,
            hysteresis_factor: 1.2, // 20% hysteresis
            max_updates_per_frame: 10,
            min_lod_level: None,
        }
    }
}

/// Chunk LOD information
#[derive(Debug, Clone)]
pub struct ChunkLod {
    pub chunk_id: ChunkId,
    pub current_lod: LodLevel,
    pub target_lod: LodLevel,
    pub distance: f32,
    pub needs_update: bool,
    pub last_update_frame: u64,
}

impl ChunkLod {
    pub fn new(chunk_id: ChunkId, initial_lod: LodLevel, distance: f32) -> Self {
        Self {
            chunk_id,
            current_lod: initial_lod,
            target_lod: initial_lod,
            distance,
            needs_update: false,
            last_update_frame: 0,
        }
    }

    /// Check if this chunk needs a LOD transition
    pub fn needs_transition(&self) -> bool {
        self.current_lod != self.target_lod
    }

    /// Get the effective vertex count reduction for this chunk
    pub fn vertex_reduction_factor(&self) -> f32 {
        self.current_lod.vertex_reduction()
    }
}

/// Main LOD system managing chunk detail levels
pub struct LodSystem {
    config: LodConfig,
    chunk_lods: HashMap<ChunkId, ChunkLod>,
    current_frame: u64,
    updates_this_frame: usize,
    total_chunks: usize,
    high_detail_chunks: usize,
    medium_detail_chunks: usize,
    low_detail_chunks: usize,
    minimal_detail_chunks: usize,
}

impl LodSystem {
    pub fn new(config: LodConfig) -> Self {
        Self {
            config,
            chunk_lods: HashMap::new(),
            current_frame: 0,
            updates_this_frame: 0,
            total_chunks: 0,
            high_detail_chunks: 0,
            medium_detail_chunks: 0,
            low_detail_chunks: 0,
            minimal_detail_chunks: 0,
        }
    }

    /// Update LOD levels for all chunks based on camera position
    pub fn update_lod_levels(&mut self, camera_position: &Vector3<f32>, chunk_positions: &HashMap<ChunkId, Vector3<f32>>) {
        self.current_frame += 1;
        self.updates_this_frame = 0;

        // Reset statistics
        self.total_chunks = chunk_positions.len();
        self.high_detail_chunks = 0;
        self.medium_detail_chunks = 0;
        self.low_detail_chunks = 0;
        self.minimal_detail_chunks = 0;

        // Update distances and calculate target LOD levels
        for (&chunk_id, &chunk_pos) in chunk_positions.iter() {
            let distance = (chunk_pos - camera_position).magnitude() * self.config.distance_bias;

            // Get or create chunk LOD info
            let chunk_lod = self.chunk_lods.entry(chunk_id).or_insert_with(|| {
                let initial_lod = Self::calculate_lod_for_distance(distance, &self.config);
                ChunkLod::new(chunk_id, initial_lod, distance)
            });

            // Update distance
            chunk_lod.distance = distance;

            // Calculate target LOD level
            let target_lod = Self::calculate_lod_for_distance(distance, &self.config);

            // Apply hysteresis to prevent flickering
            if chunk_lod.target_lod != target_lod {
                let hysteresis_distance = chunk_lod.distance * self.config.hysteresis_factor;
                let hysteresis_lod = Self::calculate_lod_for_distance(hysteresis_distance, &self.config);

                // Only change if hysteresis confirms the change
                if target_lod as u32 > chunk_lod.current_lod as u32 {
                    // Getting less detailed - apply hysteresis
                    if hysteresis_lod as u32 >= target_lod as u32 {
                        chunk_lod.target_lod = target_lod;
                    }
                } else {
                    // Getting more detailed - apply immediately
                    chunk_lod.target_lod = target_lod;
                }
            }

            // Mark for update if LOD changed and we haven't hit frame limit
            if chunk_lod.needs_transition() &&
               self.updates_this_frame < self.config.max_updates_per_frame {
                chunk_lod.needs_update = true;
                chunk_lod.current_lod = chunk_lod.target_lod;
                chunk_lod.last_update_frame = self.current_frame;
                self.updates_this_frame += 1;
            }

            // Update statistics
            match chunk_lod.current_lod {
                LodLevel::High => self.high_detail_chunks += 1,
                LodLevel::Medium => self.medium_detail_chunks += 1,
                LodLevel::Low => self.low_detail_chunks += 1,
                LodLevel::Minimal => self.minimal_detail_chunks += 1,
            }
        }

        // Remove chunks that no longer exist
        self.chunk_lods.retain(|&chunk_id, _| chunk_positions.contains_key(&chunk_id));
    }

    /// Calculate appropriate LOD level for a given distance
    fn calculate_lod_for_distance(distance: f32, config: &LodConfig) -> LodLevel {
        if !config.enabled {
            return LodLevel::High;
        }

        // Apply minimum LOD constraint if set
        let base_lod = if distance <= LodLevel::High.max_distance() {
            LodLevel::High
        } else if distance <= LodLevel::Medium.max_distance() {
            LodLevel::Medium
        } else if distance <= LodLevel::Low.max_distance() {
            LodLevel::Low
        } else {
            LodLevel::Minimal
        };

        // Apply minimum LOD constraint
        if let Some(min_lod) = config.min_lod_level {
            if (base_lod as u32) < (min_lod as u32) {
                return min_lod;
            }
        }

        base_lod
    }

    /// Get the current LOD level for a chunk
    pub fn get_chunk_lod(&self, chunk_id: &ChunkId) -> Option<LodLevel> {
        self.chunk_lods.get(chunk_id).map(|lod| lod.current_lod)
    }

    /// Get chunks that need LOD updates this frame
    pub fn get_chunks_needing_update(&mut self) -> Vec<ChunkId> {
        let mut chunks = Vec::new();

        for (chunk_id, chunk_lod) in self.chunk_lods.iter_mut() {
            if chunk_lod.needs_update {
                chunks.push(*chunk_id);
                chunk_lod.needs_update = false;
            }
        }

        chunks
    }

    /// Get performance statistics for the LOD system
    pub fn get_statistics(&self) -> LodStatistics {
        let total_vertex_reduction = self.calculate_total_vertex_reduction();

        LodStatistics {
            total_chunks: self.total_chunks,
            high_detail_chunks: self.high_detail_chunks,
            medium_detail_chunks: self.medium_detail_chunks,
            low_detail_chunks: self.low_detail_chunks,
            minimal_detail_chunks: self.minimal_detail_chunks,
            updates_this_frame: self.updates_this_frame,
            vertex_reduction_factor: total_vertex_reduction,
            frame_number: self.current_frame,
        }
    }

    /// Calculate the total vertex reduction achieved by LOD
    fn calculate_total_vertex_reduction(&self) -> f32 {
        if self.total_chunks == 0 {
            return 1.0;
        }

        let total_reduction =
            (self.high_detail_chunks as f32 * LodLevel::High.vertex_reduction()) +
            (self.medium_detail_chunks as f32 * LodLevel::Medium.vertex_reduction()) +
            (self.low_detail_chunks as f32 * LodLevel::Low.vertex_reduction()) +
            (self.minimal_detail_chunks as f32 * LodLevel::Minimal.vertex_reduction());

        total_reduction / self.total_chunks as f32
    }

    /// Update configuration
    pub fn update_config(&mut self, config: LodConfig) {
        self.config = config;
    }

    /// Get current configuration
    pub fn get_config(&self) -> &LodConfig {
        &self.config
    }

    /// Enable or disable the LOD system
    pub fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;

        if !enabled {
            // Reset all chunks to high detail
            for chunk_lod in self.chunk_lods.values_mut() {
                chunk_lod.target_lod = LodLevel::High;
                chunk_lod.needs_update = chunk_lod.current_lod != LodLevel::High;
            }
        }
    }

    /// Get the effective simplification factor for a chunk
    pub fn get_chunk_simplification(&self, chunk_id: &ChunkId) -> u32 {
        self.chunk_lods
            .get(chunk_id)
            .map(|lod| lod.current_lod.simplification_factor())
            .unwrap_or(1)
    }
}

/// Statistics for LOD system performance monitoring
#[derive(Debug, Clone)]
pub struct LodStatistics {
    pub total_chunks: usize,
    pub high_detail_chunks: usize,
    pub medium_detail_chunks: usize,
    pub low_detail_chunks: usize,
    pub minimal_detail_chunks: usize,
    pub updates_this_frame: usize,
    pub vertex_reduction_factor: f32,
    pub frame_number: u64,
}

impl LodStatistics {
    /// Get a formatted string representation of the statistics
    pub fn format_summary(&self) -> String {
        format!(
            "LOD Stats: {}/{}/{}/{} (H/M/L/Min) Total: {} Vertex Reduction: {:.1}% Updates: {}",
            self.high_detail_chunks,
            self.medium_detail_chunks,
            self.low_detail_chunks,
            self.minimal_detail_chunks,
            self.total_chunks,
            (1.0 - self.vertex_reduction_factor) * 100.0,
            self.updates_this_frame
        )
    }

    /// Log the statistics with appropriate log levels
    pub fn log_if_significant(&self) {
        // Log every 120 frames (about 2 seconds at 60fps)
        if self.frame_number % 120 == 0 {
            log::info!("📊 {}", self.format_summary());

            // Log warnings for performance issues
            if self.vertex_reduction_factor > 0.8 {
                log::warn!("⚠️  Low LOD efficiency: {:.1}% vertex reduction",
                          (1.0 - self.vertex_reduction_factor) * 100.0);
            }

            if self.updates_this_frame > 8 {
                log::warn!("⚠️  High LOD update count: {} chunks updated this frame",
                          self.updates_this_frame);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lod_level_properties() {
        assert_eq!(LodLevel::High.simplification_factor(), 1);
        assert_eq!(LodLevel::Medium.simplification_factor(), 2);
        assert_eq!(LodLevel::Low.simplification_factor(), 4);
        assert_eq!(LodLevel::Minimal.simplification_factor(), 8);
    }

    #[test]
    fn test_lod_distance_calculation() {
        let config = LodConfig::default();

        assert_eq!(LodSystem::calculate_lod_for_distance(64.0, &config), LodLevel::High);
        assert_eq!(LodSystem::calculate_lod_for_distance(200.0, &config), LodLevel::Medium);
        assert_eq!(LodSystem::calculate_lod_for_distance(400.0, &config), LodLevel::Low);
        assert_eq!(LodSystem::calculate_lod_for_distance(800.0, &config), LodLevel::Minimal);
    }

    #[test]
    fn test_vertex_reduction() {
        assert!(LodLevel::High.vertex_reduction() > LodLevel::Medium.vertex_reduction());
        assert!(LodLevel::Medium.vertex_reduction() > LodLevel::Low.vertex_reduction());
        assert!(LodLevel::Low.vertex_reduction() > LodLevel::Minimal.vertex_reduction());
    }

    #[test]
    fn test_lod_system_statistics() {
        let config = LodConfig::default();
        let mut lod_system = LodSystem::new(config);

        let mut chunk_positions = HashMap::new();
        chunk_positions.insert(1, Vector3::new(0.0, 0.0, 0.0));
        chunk_positions.insert(2, Vector3::new(200.0, 0.0, 0.0));

        let camera_pos = Vector3::new(0.0, 0.0, 0.0);
        lod_system.update_lod_levels(&camera_pos, &chunk_positions);

        let stats = lod_system.get_statistics();
        assert_eq!(stats.total_chunks, 2);
        assert!(stats.high_detail_chunks > 0);
    }
}