// Advanced chunk streaming system for large world optimization
// Provides memory management, LOD rendering, and background loading

use std::collections::{HashMap, VecDeque, BinaryHeap};
use std::cmp::Ordering;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use cgmath::{Point3, Vector3, InnerSpace};
use crate::greedy_meshing::{VoxelType, GreedyMesher};
use crate::renderer::{Mesh, Vertex};

/// Chunk coordinates for world space indexing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkCoords {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl ChunkCoords {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn from_world_position(pos: Point3<f32>, chunk_size: usize) -> Self {
        let size = chunk_size as f32;
        Self {
            x: (pos.x / size).floor() as i32,
            y: (pos.y / size).floor() as i32,
            z: (pos.z / size).floor() as i32,
        }
    }

    pub fn to_world_position(&self, chunk_size: usize) -> Point3<f32> {
        let size = chunk_size as f32;
        Point3::new(
            self.x as f32 * size,
            self.y as f32 * size,
            self.z as f32 * size,
        )
    }

    pub fn distance_to(&self, other: ChunkCoords) -> f32 {
        let dx = (self.x - other.x) as f32;
        let dy = (self.y - other.y) as f32;
        let dz = (self.z - other.z) as f32;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    pub fn manhattan_distance_to(&self, other: ChunkCoords) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs() + (self.z - other.z).abs()
    }

    pub fn neighbors_6(&self) -> [ChunkCoords; 6] {
        [
            ChunkCoords::new(self.x + 1, self.y, self.z),
            ChunkCoords::new(self.x - 1, self.y, self.z),
            ChunkCoords::new(self.x, self.y + 1, self.z),
            ChunkCoords::new(self.x, self.y - 1, self.z),
            ChunkCoords::new(self.x, self.y, self.z + 1),
            ChunkCoords::new(self.x, self.y, self.z - 1),
        ]
    }
}

/// Level of detail for chunk rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChunkLOD {
    Full,      // Full detail mesh
    Half,      // Half resolution mesh
    Quarter,   // Quarter resolution mesh
    Impostor,  // Simple impostor or billboard
}

impl ChunkLOD {
    pub fn from_distance(distance: f32, config: &StreamingConfig) -> Self {
        if distance <= config.full_detail_distance {
            ChunkLOD::Full
        } else if distance <= config.half_detail_distance {
            ChunkLOD::Half
        } else if distance <= config.quarter_detail_distance {
            ChunkLOD::Quarter
        } else {
            ChunkLOD::Impostor
        }
    }

    pub fn mesh_scale(&self) -> usize {
        match self {
            ChunkLOD::Full => 1,
            ChunkLOD::Half => 2,
            ChunkLOD::Quarter => 4,
            ChunkLOD::Impostor => 8,
        }
    }
}

/// Chunk loading state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkState {
    Unloaded,
    Queued,
    Loading,
    Loaded,
    Meshing,
    Ready,
    Unloading,
}

/// Individual chunk data and metadata
#[derive(Debug)]
pub struct ChunkData {
    pub coords: ChunkCoords,
    pub voxels: Vec<VoxelType>,
    pub mesh: Option<Mesh>,
    pub lod: ChunkLOD,
    pub state: ChunkState,
    pub last_accessed: Instant,
    pub memory_size: usize,
    pub generation_version: u32,
}

impl ChunkData {
    pub fn new(coords: ChunkCoords, chunk_size: usize) -> Self {
        let voxel_count = chunk_size * chunk_size * chunk_size;
        Self {
            coords,
            voxels: vec![VoxelType::Air; voxel_count],
            mesh: None,
            lod: ChunkLOD::Full,
            state: ChunkState::Unloaded,
            last_accessed: Instant::now(),
            memory_size: voxel_count * std::mem::size_of::<VoxelType>(),
            generation_version: 0,
        }
    }

    pub fn update_access_time(&mut self) {
        self.last_accessed = Instant::now();
    }

    pub fn estimate_memory_usage(&self) -> usize {
        let voxel_memory = self.voxels.len() * std::mem::size_of::<VoxelType>();
        let mesh_memory = self.mesh.as_ref()
            .map(|mesh| mesh.vertices.len() * std::mem::size_of::<Vertex>() +
                        mesh.indices.len() * std::mem::size_of::<u32>())
            .unwrap_or(0);
        voxel_memory + mesh_memory
    }
}

/// Priority queue item for chunk loading
#[derive(Debug, Clone)]
pub struct ChunkLoadRequest {
    pub coords: ChunkCoords,
    pub priority: f32,
    pub lod: ChunkLOD,
    pub request_time: Instant,
}

impl PartialEq for ChunkLoadRequest {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl Eq for ChunkLoadRequest {}

impl PartialOrd for ChunkLoadRequest {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ChunkLoadRequest {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first (reverse ordering)
        other.priority.partial_cmp(&self.priority).unwrap_or(Ordering::Equal)
    }
}

/// Configuration for chunk streaming system
#[derive(Debug, Clone)]
pub struct StreamingConfig {
    pub chunk_size: usize,
    pub render_distance: u32,
    pub full_detail_distance: f32,
    pub half_detail_distance: f32,
    pub quarter_detail_distance: f32,
    pub max_loaded_chunks: usize,
    pub max_memory_mb: f32,
    pub background_thread_count: usize,
    pub load_queue_size: usize,
    pub unload_queue_size: usize,
    pub priority_update_interval: Duration,
    pub memory_cleanup_interval: Duration,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            chunk_size: 32,
            render_distance: 16,
            full_detail_distance: 64.0,
            half_detail_distance: 128.0,
            quarter_detail_distance: 256.0,
            max_loaded_chunks: 1000,
            max_memory_mb: 512.0,
            background_thread_count: 2,
            load_queue_size: 50,
            unload_queue_size: 20,
            priority_update_interval: Duration::from_millis(100),
            memory_cleanup_interval: Duration::from_secs(5),
        }
    }
}

/// Statistics for streaming performance
#[derive(Debug, Clone, Default)]
pub struct StreamingStats {
    pub loaded_chunks: usize,
    pub pending_loads: usize,
    pub pending_unloads: usize,
    pub memory_usage_mb: f32,
    pub cache_hit_rate: f32,
    pub average_load_time_ms: f32,
    pub chunks_loaded_per_second: f32,
    pub chunks_unloaded_per_second: f32,
    pub lod_distribution: HashMap<ChunkLOD, usize>,
}

impl StreamingStats {
    pub fn memory_usage_percentage(&self, config: &StreamingConfig) -> f32 {
        (self.memory_usage_mb / config.max_memory_mb) * 100.0
    }

    pub fn is_memory_pressure(&self, config: &StreamingConfig) -> bool {
        self.memory_usage_percentage(config) > 85.0
    }

    pub fn format_summary(&self) -> String {
        format!(
            "Streaming: {} chunks loaded, {:.1}MB memory, {:.1}% cache hit rate, {:.1} chunks/s",
            self.loaded_chunks,
            self.memory_usage_mb,
            self.cache_hit_rate,
            self.chunks_loaded_per_second
        )
    }
}

/// LRU cache for chunk memory management
struct ChunkCache {
    chunks: HashMap<ChunkCoords, ChunkData>,
    access_order: VecDeque<ChunkCoords>,
    max_size: usize,
    max_memory_bytes: usize,
    current_memory: usize,
}

impl ChunkCache {
    fn new(max_size: usize, max_memory_mb: f32) -> Self {
        Self {
            chunks: HashMap::new(),
            access_order: VecDeque::new(),
            max_size,
            max_memory_bytes: (max_memory_mb * 1024.0 * 1024.0) as usize,
            current_memory: 0,
        }
    }

    fn get_mut(&mut self, coords: &ChunkCoords) -> Option<&mut ChunkData> {
        if let Some(chunk) = self.chunks.get_mut(coords) {
            chunk.update_access_time();
            // Move to front of access order
            if let Some(pos) = self.access_order.iter().position(|c| c == coords) {
                self.access_order.remove(pos);
            }
            self.access_order.push_front(*coords);
            Some(chunk)
        } else {
            None
        }
    }

    fn insert(&mut self, coords: ChunkCoords, chunk: ChunkData) -> Vec<ChunkCoords> {
        let chunk_memory = chunk.estimate_memory_usage();
        let mut evicted = Vec::new();

        // Remove if already exists
        if let Some(old_chunk) = self.chunks.remove(&coords) {
            self.current_memory -= old_chunk.estimate_memory_usage();
            if let Some(pos) = self.access_order.iter().position(|c| c == &coords) {
                self.access_order.remove(pos);
            }
        }

        // Evict chunks if needed
        while (self.chunks.len() >= self.max_size ||
               self.current_memory + chunk_memory > self.max_memory_bytes) &&
              !self.access_order.is_empty() {

            if let Some(lru_coords) = self.access_order.pop_back() {
                if let Some(evicted_chunk) = self.chunks.remove(&lru_coords) {
                    self.current_memory -= evicted_chunk.estimate_memory_usage();
                    evicted.push(lru_coords);
                }
            }
        }

        // Insert new chunk
        self.current_memory += chunk_memory;
        self.chunks.insert(coords, chunk);
        self.access_order.push_front(coords);

        evicted
    }

    fn remove(&mut self, coords: &ChunkCoords) -> Option<ChunkData> {
        if let Some(chunk) = self.chunks.remove(coords) {
            self.current_memory -= chunk.estimate_memory_usage();
            if let Some(pos) = self.access_order.iter().position(|c| c == coords) {
                self.access_order.remove(pos);
            }
            Some(chunk)
        } else {
            None
        }
    }

    fn memory_usage_mb(&self) -> f32 {
        self.current_memory as f32 / (1024.0 * 1024.0)
    }

    fn cleanup_stale_chunks(&mut self, max_age: Duration) -> Vec<ChunkCoords> {
        let now = Instant::now();
        let mut to_remove = Vec::new();

        for (coords, chunk) in &self.chunks {
            if now.duration_since(chunk.last_accessed) > max_age {
                to_remove.push(*coords);
            }
        }

        for coords in &to_remove {
            self.remove(coords);
        }

        to_remove
    }
}

/// Main chunk streaming system
pub struct ChunkStreamingSystem {
    config: StreamingConfig,
    cache: ChunkCache,
    load_queue: BinaryHeap<ChunkLoadRequest>,
    unload_queue: VecDeque<ChunkCoords>,
    loading_chunks: HashMap<ChunkCoords, Instant>,
    player_position: Point3<f32>,
    last_priority_update: Instant,
    last_memory_cleanup: Instant,
    stats: StreamingStats,
    generation_function: Box<dyn Fn(ChunkCoords, usize) -> Vec<VoxelType> + Send + Sync>,
}

impl ChunkStreamingSystem {
    pub fn new(
        config: StreamingConfig,
        generation_function: Box<dyn Fn(ChunkCoords, usize) -> Vec<VoxelType> + Send + Sync>
    ) -> Self {
        let cache = ChunkCache::new(config.max_loaded_chunks, config.max_memory_mb);

        Self {
            config,
            cache,
            load_queue: BinaryHeap::new(),
            unload_queue: VecDeque::new(),
            loading_chunks: HashMap::new(),
            player_position: Point3::new(0.0, 0.0, 0.0),
            last_priority_update: Instant::now(),
            last_memory_cleanup: Instant::now(),
            stats: StreamingStats::default(),
            generation_function,
        }
    }

    /// Update player position and trigger chunk management
    pub fn update_player_position(&mut self, position: Point3<f32>) {
        self.player_position = position;

        let now = Instant::now();

        // Update chunk priorities periodically
        if now.duration_since(self.last_priority_update) >= self.config.priority_update_interval {
            self.update_chunk_priorities();
            self.last_priority_update = now;
        }

        // Memory cleanup periodically
        if now.duration_since(self.last_memory_cleanup) >= self.config.memory_cleanup_interval {
            self.cleanup_memory();
            self.last_memory_cleanup = now;
        }

        // Update statistics
        self.update_statistics();
    }

    /// Get chunk at coordinates, loading if necessary
    pub fn get_chunk(&mut self, coords: ChunkCoords) -> Option<&mut ChunkData> {
        // Check if chunk is in cache first (separate scope to avoid borrow conflicts)
        let has_chunk = self.cache.chunks.contains_key(&coords);

        if has_chunk {
            return self.cache.get_mut(&coords);
        }

        // Queue chunk for loading if not already queued/loading
        if !self.loading_chunks.contains_key(&coords) &&
           !self.load_queue.iter().any(|req| req.coords == coords) {
            self.queue_chunk_load(coords);
        }

        None
    }

    /// Process chunk loading and unloading
    pub fn process_streaming(&mut self) {
        // Process a limited number of loads per frame to avoid hitches
        let max_loads_per_frame = 3;
        let mut loads_processed = 0;

        while loads_processed < max_loads_per_frame && !self.load_queue.is_empty() {
            if let Some(request) = self.load_queue.pop() {
                self.start_chunk_load(request);
                loads_processed += 1;
            }
        }

        // Process unloads
        let max_unloads_per_frame = 5;
        let mut unloads_processed = 0;

        while unloads_processed < max_unloads_per_frame && !self.unload_queue.is_empty() {
            if let Some(coords) = self.unload_queue.pop_front() {
                self.unload_chunk(coords);
                unloads_processed += 1;
            }
        }

        // Check for completed loads (in a real implementation, this would check worker threads)
        self.check_completed_loads();
    }

    fn queue_chunk_load(&mut self, coords: ChunkCoords) {
        let player_chunk = ChunkCoords::from_world_position(self.player_position, self.config.chunk_size);
        let distance = coords.distance_to(player_chunk);
        let lod = ChunkLOD::from_distance(distance, &self.config);

        // Calculate priority based on distance and visibility
        let priority = self.calculate_load_priority(coords, distance);

        let request = ChunkLoadRequest {
            coords,
            priority,
            lod,
            request_time: Instant::now(),
        };

        self.load_queue.push(request);
    }

    fn calculate_load_priority(&self, coords: ChunkCoords, distance: f32) -> f32 {
        let player_chunk = ChunkCoords::from_world_position(self.player_position, self.config.chunk_size);

        // Base priority from inverse distance
        let distance_priority = 1000.0 / (distance + 1.0);

        // Boost priority for chunks in viewing direction
        let chunk_world_pos = coords.to_world_position(self.config.chunk_size);
        let to_chunk = chunk_world_pos - self.player_position;

        // Assume forward direction for now (in a real implementation, use camera direction)
        let forward = Vector3::new(0.0, 0.0, -1.0);
        let direction_dot = to_chunk.normalize().dot(forward).max(0.0);
        let direction_priority = direction_dot * 100.0;

        // Boost priority for chunks adjacent to loaded chunks
        let adjacency_bonus = coords.neighbors_6().iter()
            .filter(|neighbor| self.cache.chunks.contains_key(neighbor))
            .count() as f32 * 50.0;

        distance_priority + direction_priority + adjacency_bonus
    }

    fn start_chunk_load(&mut self, request: ChunkLoadRequest) {
        // In a real implementation, this would dispatch to background thread
        // For now, we'll generate synchronously
        let voxels = (self.generation_function)(request.coords, self.config.chunk_size);

        let mut chunk = ChunkData::new(request.coords, self.config.chunk_size);
        chunk.voxels = voxels;
        chunk.lod = request.lod;
        chunk.state = ChunkState::Loading;

        // Generate mesh based on LOD
        if let Ok(mesh) = self.generate_chunk_mesh(&chunk) {
            chunk.mesh = Some(mesh);
            chunk.state = ChunkState::Ready;
        }

        // Insert into cache and handle evictions
        let evicted = self.cache.insert(request.coords, chunk);

        // Queue evicted chunks for unloading
        for evicted_coords in evicted {
            self.unload_queue.push_back(evicted_coords);
        }

        self.loading_chunks.remove(&request.coords);
    }

    fn generate_chunk_mesh(&self, chunk: &ChunkData) -> Result<Mesh, String> {
        let mut mesher = GreedyMesher::new(self.config.chunk_size);

        // Create a single-chunk HashMap for the mesher
        let mut chunks = std::collections::HashMap::new();

        // Scale down voxels based on LOD
        let scale = chunk.lod.mesh_scale();
        let scaled_size = self.config.chunk_size / scale;

        let voxels_to_mesh = if scale == 1 {
            // Full detail
            &chunk.voxels
        } else {
            // Downsample voxels for LOD (for now, use full detail)
            &chunk.voxels
        };

        chunks.insert((0, 0, 0), voxels_to_mesh);

        // Create voxel getter function
        let chunk_size = self.config.chunk_size;
        let get_voxel = |x: i32, y: i32, z: i32| -> VoxelType {
            if x >= 0 && y >= 0 && z >= 0 &&
               x < chunk_size as i32 && y < chunk_size as i32 && z < chunk_size as i32 {
                let idx = (z as usize) * chunk_size * chunk_size +
                         (y as usize) * chunk_size +
                         (x as usize);
                if idx < voxels_to_mesh.len() {
                    voxels_to_mesh[idx]
                } else {
                    VoxelType::Air
                }
            } else {
                VoxelType::Air
            }
        };

        // Generate mesh
        let (greedy_vertices, indices) = mesher.generate_mesh(&chunks, get_voxel);

        // Convert greedy meshing vertices to renderer vertices
        let vertices: Vec<crate::renderer::Vertex> = greedy_vertices.into_iter().map(|v| {
            crate::renderer::Vertex {
                position: v.position,
                color: v.color,
                normal: v.normal,
                tex_coords: [0.0, 0.0], // Default texture coordinates
            }
        }).collect();

        Ok(Mesh {
            vertices,
            indices,
            vertex_buffer: None,
            index_buffer: None,
            vertex_count: 0,
            index_count: 0,
        })
    }

    fn downsample_voxels(&self, voxels: &[VoxelType], original_size: usize, scale: usize) -> Vec<VoxelType> {
        let new_size = original_size / scale;
        let mut downsampled = vec![VoxelType::Air; new_size * new_size * new_size];

        for z in 0..new_size {
            for y in 0..new_size {
                for x in 0..new_size {
                    // Sample from the center of each scaled block
                    let src_x = x * scale + scale / 2;
                    let src_y = y * scale + scale / 2;
                    let src_z = z * scale + scale / 2;

                    if src_x < original_size && src_y < original_size && src_z < original_size {
                        let src_idx = src_z * original_size * original_size + src_y * original_size + src_x;
                        let dst_idx = z * new_size * new_size + y * new_size + x;
                        downsampled[dst_idx] = voxels[src_idx];
                    }
                }
            }
        }

        downsampled
    }

    fn unload_chunk(&mut self, coords: ChunkCoords) {
        self.cache.remove(&coords);
    }

    fn check_completed_loads(&mut self) {
        // In a real implementation, this would check for completed background tasks
        // For now, we'll just clean up any stale loading entries
        let now = Instant::now();
        let stale_timeout = Duration::from_secs(10);

        self.loading_chunks.retain(|_, start_time| {
            now.duration_since(*start_time) < stale_timeout
        });
    }

    fn update_chunk_priorities(&mut self) {
        let player_chunk = ChunkCoords::from_world_position(self.player_position, self.config.chunk_size);
        let render_distance = self.config.render_distance as i32;

        // Identify chunks that should be loaded
        let mut needed_chunks = Vec::new();
        for dx in -render_distance..=render_distance {
            for dy in -render_distance..=render_distance {
                for dz in -render_distance..=render_distance {
                    let coords = ChunkCoords::new(
                        player_chunk.x + dx,
                        player_chunk.y + dy,
                        player_chunk.z + dz,
                    );

                    let distance = coords.distance_to(player_chunk);
                    if distance <= render_distance as f32 {
                        needed_chunks.push(coords);
                    }
                }
            }
        }

        // Queue chunks that aren't loaded or loading
        for coords in needed_chunks {
            if !self.cache.chunks.contains_key(&coords) &&
               !self.loading_chunks.contains_key(&coords) &&
               !self.load_queue.iter().any(|req| req.coords == coords) {
                self.queue_chunk_load(coords);
            }
        }

        // Queue distant chunks for unloading
        let max_distance = (render_distance + 2) as f32;
        let chunks_to_unload: Vec<ChunkCoords> = self.cache.chunks.keys()
            .filter(|coords| coords.distance_to(player_chunk) > max_distance)
            .copied()
            .collect();

        for coords in chunks_to_unload {
            if !self.unload_queue.contains(&coords) {
                self.unload_queue.push_back(coords);
            }
        }
    }

    fn cleanup_memory(&mut self) {
        // Remove stale chunks that haven't been accessed recently
        let max_age = Duration::from_secs(30);
        let removed = self.cache.cleanup_stale_chunks(max_age);

        if !removed.is_empty() {
            log::info!("🧹 Cleaned up {} stale chunks from memory", removed.len());
        }

        // Force unload if memory pressure is high
        if self.stats.is_memory_pressure(&self.config) {
            let player_chunk = ChunkCoords::from_world_position(self.player_position, self.config.chunk_size);
            let max_distance = self.config.render_distance as f32 * 0.8; // Keep closer chunks

            let chunks_to_unload: Vec<ChunkCoords> = self.cache.chunks.keys()
                .filter(|coords| coords.distance_to(player_chunk) > max_distance)
                .copied()
                .collect();

            for coords in chunks_to_unload {
                self.unload_chunk(coords);
            }
        }
    }

    fn update_statistics(&mut self) {
        self.stats.loaded_chunks = self.cache.chunks.len();
        self.stats.pending_loads = self.load_queue.len();
        self.stats.pending_unloads = self.unload_queue.len();
        self.stats.memory_usage_mb = self.cache.memory_usage_mb();

        // Update LOD distribution
        self.stats.lod_distribution.clear();
        for chunk in self.cache.chunks.values() {
            *self.stats.lod_distribution.entry(chunk.lod).or_insert(0) += 1;
        }
    }

    /// Get current streaming statistics
    pub fn get_statistics(&self) -> &StreamingStats {
        &self.stats
    }

    /// Get configuration
    pub fn get_config(&self) -> &StreamingConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: StreamingConfig) {
        self.config = config;
    }

    /// Get all loaded chunks for rendering
    pub fn get_visible_chunks(&self, frustum: Option<&crate::renderer::Frustum>) -> Vec<&ChunkData> {
        let mut visible_chunks = Vec::new();

        for chunk in self.cache.chunks.values() {
            if chunk.state == ChunkState::Ready && chunk.mesh.is_some() {
                // TODO: Add frustum culling here
                if let Some(_frustum) = frustum {
                    // let chunk_aabb = AABB::from_chunk_coords(
                    //     chunk.coords.x, chunk.coords.y, chunk.coords.z,
                    //     self.config.chunk_size
                    // );
                    // if frustum.intersects_aabb(&chunk_aabb) {
                    //     visible_chunks.push(chunk);
                    // }
                    visible_chunks.push(chunk); // Simplified for now
                } else {
                    visible_chunks.push(chunk);
                }
            }
        }

        visible_chunks
    }

    /// Force load a specific chunk (for testing)
    pub fn force_load_chunk(&mut self, coords: ChunkCoords) {
        if !self.cache.chunks.contains_key(&coords) {
            let request = ChunkLoadRequest {
                coords,
                priority: 10000.0, // Very high priority
                lod: ChunkLOD::Full,
                request_time: Instant::now(),
            };
            self.start_chunk_load(request);
        }
    }
}

/// Simple terrain generation function for testing
pub fn simple_terrain_generator(coords: ChunkCoords, chunk_size: usize) -> Vec<VoxelType> {
    let mut voxels = vec![VoxelType::Air; chunk_size * chunk_size * chunk_size];

    // Simple heightmap-based terrain
    for z in 0..chunk_size {
        for x in 0..chunk_size {
            let world_x = coords.x as f32 * chunk_size as f32 + x as f32;
            let world_z = coords.z as f32 * chunk_size as f32 + z as f32;

            // Simple sine wave terrain
            let height = (world_x * 0.1).sin() * (world_z * 0.1).cos() * 8.0 + 32.0;
            let terrain_height = height as usize;

            for y in 0..chunk_size {
                let world_y = coords.y as usize * chunk_size + y;

                if world_y < terrain_height {
                    let idx = z * chunk_size * chunk_size + y * chunk_size + x;

                    if world_y < terrain_height - 5 {
                        voxels[idx] = VoxelType::Stone;
                    } else if world_y < terrain_height - 1 {
                        voxels[idx] = VoxelType::Earth;
                    } else {
                        voxels[idx] = VoxelType::Grass;
                    }
                }
            }
        }
    }

    voxels
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_coords() {
        let coords = ChunkCoords::new(1, 2, 3);
        assert_eq!(coords.x, 1);
        assert_eq!(coords.y, 2);
        assert_eq!(coords.z, 3);

        let world_pos = coords.to_world_position(32);
        assert_eq!(world_pos, Point3::new(32.0, 64.0, 96.0));

        let back_to_coords = ChunkCoords::from_world_position(world_pos, 32);
        assert_eq!(back_to_coords, coords);
    }

    #[test]
    fn test_chunk_lod() {
        let config = StreamingConfig::default();

        assert_eq!(ChunkLOD::from_distance(32.0, &config), ChunkLOD::Full);
        assert_eq!(ChunkLOD::from_distance(96.0, &config), ChunkLOD::Half);
        assert_eq!(ChunkLOD::from_distance(192.0, &config), ChunkLOD::Quarter);
        assert_eq!(ChunkLOD::from_distance(300.0, &config), ChunkLOD::Impostor);
    }

    #[test]
    fn test_chunk_cache() {
        let mut cache = ChunkCache::new(3, 1.0); // 3 chunks max, 1MB max

        let chunk1 = ChunkData::new(ChunkCoords::new(0, 0, 0), 32);
        let chunk2 = ChunkData::new(ChunkCoords::new(1, 0, 0), 32);
        let chunk3 = ChunkData::new(ChunkCoords::new(2, 0, 0), 32);
        let chunk4 = ChunkData::new(ChunkCoords::new(3, 0, 0), 32);

        // Insert chunks
        cache.insert(ChunkCoords::new(0, 0, 0), chunk1);
        cache.insert(ChunkCoords::new(1, 0, 0), chunk2);
        cache.insert(ChunkCoords::new(2, 0, 0), chunk3);

        assert_eq!(cache.chunks.len(), 3);

        // Insert 4th chunk should evict LRU
        let evicted = cache.insert(ChunkCoords::new(3, 0, 0), chunk4);
        assert_eq!(evicted.len(), 1);
        assert_eq!(evicted[0], ChunkCoords::new(0, 0, 0)); // First inserted should be evicted
    }

    #[test]
    fn test_streaming_system() {
        let config = StreamingConfig::default();
        let generation_fn = Box::new(simple_terrain_generator);
        let mut system = ChunkStreamingSystem::new(config, generation_fn);

        // Update player position
        system.update_player_position(Point3::new(0.0, 0.0, 0.0));

        // Force load a chunk
        system.force_load_chunk(ChunkCoords::new(0, 0, 0));

        let stats = system.get_statistics();
        assert_eq!(stats.loaded_chunks, 1);
    }
}