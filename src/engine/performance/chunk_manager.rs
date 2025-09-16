// Chunk Loading and Management System for Robin Engine
// Provides streaming of world data for large-scale environments

use crate::engine::error::RobinResult;
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::Instant;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChunkCoordinate {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl ChunkCoordinate {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn distance_to(&self, other: &ChunkCoordinate) -> f32 {
        let dx = (self.x - other.x) as f32;
        let dy = (self.y - other.y) as f32;
        let dz = (self.z - other.z) as f32;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    pub fn manhattan_distance_to(&self, other: &ChunkCoordinate) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs() + (self.z - other.z).abs()
    }

    pub fn neighbors(&self) -> Vec<ChunkCoordinate> {
        vec![
            ChunkCoordinate::new(self.x + 1, self.y, self.z),
            ChunkCoordinate::new(self.x - 1, self.y, self.z),
            ChunkCoordinate::new(self.x, self.y + 1, self.z),
            ChunkCoordinate::new(self.x, self.y - 1, self.z),
            ChunkCoordinate::new(self.x, self.y, self.z + 1),
            ChunkCoordinate::new(self.x, self.y, self.z - 1),
        ]
    }

    pub fn from_world_position(world_pos: [f32; 3], chunk_size: f32) -> Self {
        Self {
            x: (world_pos[0] / chunk_size).floor() as i32,
            y: (world_pos[1] / chunk_size).floor() as i32,
            z: (world_pos[2] / chunk_size).floor() as i32,
        }
    }

    pub fn to_world_position(&self, chunk_size: f32) -> [f32; 3] {
        [
            self.x as f32 * chunk_size,
            self.y as f32 * chunk_size,
            self.z as f32 * chunk_size,
        ]
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChunkState {
    Unloaded,
    Loading,
    Loaded,
    Unloading,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct ChunkLoadRequest {
    pub coordinate: ChunkCoordinate,
    pub priority: i32,
    pub requested_at: Instant,
    pub requester_distance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkConfig {
    pub enabled: bool,
    pub chunk_size: f32,
    pub view_distance: i32,
    pub preload_distance: i32,
    pub max_concurrent_loads: usize,
    pub max_loaded_chunks: usize,
    pub unload_delay_seconds: f32,
    pub priority_loading: bool,
    pub background_generation: bool,
    pub cache_size_mb: u32,
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            chunk_size: 32.0,
            view_distance: 16,
            preload_distance: 2,
            max_concurrent_loads: 4,
            max_loaded_chunks: 1000,
            unload_delay_seconds: 5.0,
            priority_loading: true,
            background_generation: true,
            cache_size_mb: 256,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChunkData {
    pub coordinate: ChunkCoordinate,
    pub blocks: Vec<u8>, // Simplified block data (material IDs)
    pub metadata: HashMap<String, String>,
    pub last_accessed: Instant,
    pub size_bytes: usize,
    pub generation_seed: u64,
    pub is_modified: bool,
}

impl ChunkData {
    pub fn new(coordinate: ChunkCoordinate, size: usize) -> Self {
        Self {
            coordinate,
            blocks: vec![0; size], // Fill with air blocks
            metadata: HashMap::new(),
            last_accessed: Instant::now(),
            size_bytes: size,
            generation_seed: 0,
            is_modified: false,
        }
    }

    pub fn generate_terrain(&mut self, config: &ChunkConfig) {
        // Simple terrain generation for demonstration
        let world_pos = self.coordinate.to_world_position(config.chunk_size);
        let chunk_size = config.chunk_size as usize;
        
        for x in 0..chunk_size {
            for y in 0..chunk_size {
                for z in 0..chunk_size {
                    let world_x = world_pos[0] + x as f32;
                    let world_y = world_pos[1] + y as f32;
                    let world_z = world_pos[2] + z as f32;
                    
                    let idx = x * chunk_size * chunk_size + y * chunk_size + z;
                    
                    // Simple height-based terrain
                    let height = (world_x * 0.01).sin() * 10.0 + (world_z * 0.01).cos() * 8.0;
                    
                    if world_y < height {
                        if world_y < height - 5.0 {
                            self.blocks[idx] = 1; // Stone
                        } else if world_y < height - 1.0 {
                            self.blocks[idx] = 2; // Dirt
                        } else {
                            self.blocks[idx] = 3; // Grass
                        }
                    } else {
                        self.blocks[idx] = 0; // Air
                    }
                }
            }
        }
        
        self.last_accessed = Instant::now();
    }
}

#[derive(Debug)]
pub struct Chunk {
    pub coordinate: ChunkCoordinate,
    pub state: ChunkState,
    pub data: Option<ChunkData>,
    pub last_accessed: Instant,
    pub load_started: Option<Instant>,
    pub dependent_chunks: HashSet<ChunkCoordinate>,
    pub reference_count: u32,
}

impl Chunk {
    pub fn new(coordinate: ChunkCoordinate) -> Self {
        Self {
            coordinate,
            state: ChunkState::Unloaded,
            data: None,
            last_accessed: Instant::now(),
            load_started: None,
            dependent_chunks: HashSet::new(),
            reference_count: 0,
        }
    }

    pub fn is_loaded(&self) -> bool {
        matches!(self.state, ChunkState::Loaded)
    }

    pub fn is_loading(&self) -> bool {
        matches!(self.state, ChunkState::Loading)
    }

    pub fn can_unload(&self) -> bool {
        self.reference_count == 0 && matches!(self.state, ChunkState::Loaded)
    }

    pub fn get_memory_usage(&self) -> usize {
        self.data.as_ref().map_or(0, |data| data.size_bytes) + 
        std::mem::size_of::<Self>() +
        self.dependent_chunks.len() * std::mem::size_of::<ChunkCoordinate>()
    }
}

#[derive(Debug, Default, Clone)]
pub struct ChunkManagerMetrics {
    pub total_chunks: u32,
    pub loaded_chunks: u32,
    pub loading_chunks: u32,
    pub unloading_chunks: u32,
    pub memory_usage_mb: f32,
    pub load_requests_pending: u32,
    pub loads_completed: u32,
    pub unloads_completed: u32,
    pub cache_hits: u32,
    pub cache_misses: u32,
    pub average_load_time_ms: f32,
}

#[derive(Debug)]
pub struct ChunkManager {
    config: ChunkConfig,
    chunks: HashMap<ChunkCoordinate, Chunk>,
    load_queue: VecDeque<ChunkLoadRequest>,
    unload_queue: VecDeque<ChunkCoordinate>,
    currently_loading: HashSet<ChunkCoordinate>,
    camera_chunk: ChunkCoordinate,
    quality_multiplier: f32,
    metrics: ChunkManagerMetrics,
    chunk_cache: HashMap<ChunkCoordinate, ChunkData>,
    enabled: bool,
}

impl ChunkManager {
    pub fn new(config: ChunkConfig) -> RobinResult<Self> {
        Ok(Self {
            enabled: config.enabled,
            config,
            chunks: HashMap::new(),
            load_queue: VecDeque::new(),
            unload_queue: VecDeque::new(),
            currently_loading: HashSet::new(),
            camera_chunk: ChunkCoordinate::new(0, 0, 0),
            quality_multiplier: 1.0,
            metrics: ChunkManagerMetrics::default(),
            chunk_cache: HashMap::new(),
        })
    }

    pub fn initialize(&mut self) -> RobinResult<()> {
        self.metrics = ChunkManagerMetrics::default();
        
        if self.enabled {
            println!("Chunk Manager initialized:");
            println!("  Chunk Size: {:.1}m", self.config.chunk_size);
            println!("  View Distance: {} chunks", self.config.view_distance);
            println!("  Max Loaded Chunks: {}", self.config.max_loaded_chunks);
            println!("  Cache Size: {}MB", self.config.cache_size_mb);
        }

        Ok(())
    }

    pub fn update(&mut self, camera_position: [f32; 3]) -> RobinResult<()> {
        if !self.enabled {
            return Ok(());
        }

        // Update camera chunk position
        self.camera_chunk = ChunkCoordinate::from_world_position(camera_position, self.config.chunk_size);

        // Process loading and unloading
        self.process_load_queue()?;
        self.process_unload_queue()?;
        self.update_chunk_loading_priorities()?;
        self.manage_memory_usage()?;
        self.update_metrics();

        Ok(())
    }

    fn process_load_queue(&mut self) -> RobinResult<()> {
        // Limit concurrent loads
        while self.currently_loading.len() < self.config.max_concurrent_loads && !self.load_queue.is_empty() {
            if let Some(request) = self.load_queue.pop_front() {
                self.start_chunk_load(request)?;
            }
        }

        // Check for completed loads (simulated)
        let completed_loads: Vec<ChunkCoordinate> = self.currently_loading.iter()
            .filter_map(|&coord| {
                if let Some(chunk) = self.chunks.get(&coord) {
                    if chunk.load_started.map_or(false, |start| start.elapsed().as_millis() > 100) {
                        Some(coord)
                    } else {
                        None
                    }
                } else {
                    Some(coord) // Remove from loading if chunk doesn't exist
                }
            })
            .collect();

        for coord in completed_loads {
            self.complete_chunk_load(coord)?;
        }

        Ok(())
    }

    fn start_chunk_load(&mut self, request: ChunkLoadRequest) -> RobinResult<()> {
        let coord = request.coordinate;
        
        // Check cache first
        if let Some(cached_data) = self.chunk_cache.get(&coord).cloned() {
            let chunk = Chunk {
                coordinate: coord,
                state: ChunkState::Loaded,
                data: Some(cached_data),
                last_accessed: Instant::now(),
                load_started: None,
                dependent_chunks: HashSet::new(),
                reference_count: 1,
            };
            
            self.chunks.insert(coord, chunk);
            self.metrics.cache_hits += 1;
            return Ok(());
        }

        // Start loading
        let chunk = Chunk {
            coordinate: coord,
            state: ChunkState::Loading,
            data: None,
            last_accessed: Instant::now(),
            load_started: Some(Instant::now()),
            dependent_chunks: HashSet::new(),
            reference_count: 1,
        };

        self.chunks.insert(coord, chunk);
        self.currently_loading.insert(coord);
        self.metrics.cache_misses += 1;

        Ok(())
    }

    fn complete_chunk_load(&mut self, coord: ChunkCoordinate) -> RobinResult<()> {
        self.currently_loading.remove(&coord);

        if let Some(chunk) = self.chunks.get_mut(&coord) {
            // Generate chunk data
            let chunk_size = (self.config.chunk_size as usize).pow(3);
            let mut chunk_data = ChunkData::new(coord, chunk_size);
            chunk_data.generate_terrain(&self.config);

            chunk.data = Some(chunk_data.clone());
            chunk.state = ChunkState::Loaded;
            chunk.last_accessed = Instant::now();

            // Cache the loaded data
            if self.chunk_cache.len() < (self.config.cache_size_mb * 1024 * 1024) as usize / chunk_size {
                self.chunk_cache.insert(coord, chunk_data);
            }

            self.metrics.loads_completed += 1;
        }

        Ok(())
    }

    fn process_unload_queue(&mut self) -> RobinResult<()> {
        let current_time = Instant::now();
        
        while let Some(&coord) = self.unload_queue.front() {
            if let Some(chunk) = self.chunks.get(&coord) {
                let should_unload = chunk.can_unload() &&
                    chunk.last_accessed.elapsed().as_secs_f32() >= self.config.unload_delay_seconds;
                
                if should_unload {
                    self.unload_queue.pop_front();
                    self.unload_chunk(coord)?;
                } else {
                    break; // Queue is ordered by time, so we can stop here
                }
            } else {
                self.unload_queue.pop_front(); // Remove invalid coordinate
            }
        }

        Ok(())
    }

    fn unload_chunk(&mut self, coord: ChunkCoordinate) -> RobinResult<()> {
        if let Some(mut chunk) = self.chunks.remove(&coord) {
            chunk.state = ChunkState::Unloading;
            
            // Cache data before unloading if modified
            if let Some(data) = &chunk.data {
                if data.is_modified {
                    self.chunk_cache.insert(coord, data.clone());
                }
            }

            self.metrics.unloads_completed += 1;
        }

        Ok(())
    }

    fn update_chunk_loading_priorities(&mut self) -> RobinResult<()> {
        if !self.config.priority_loading {
            return Ok(());
        }

        // Sort load queue by priority (distance from camera)
        let camera_chunk = self.camera_chunk;
        self.load_queue.make_contiguous().sort_by(|a, b| {
            let dist_a = a.coordinate.distance_to(&camera_chunk);
            let dist_b = b.coordinate.distance_to(&camera_chunk);
            dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Request nearby chunks for loading
        let view_distance = (self.config.view_distance as f32 * self.quality_multiplier) as i32;
        let preload_distance = self.config.preload_distance;

        for dx in -view_distance..=view_distance {
            for dy in -view_distance..=view_distance {
                for dz in -view_distance..=view_distance {
                    let coord = ChunkCoordinate::new(
                        camera_chunk.x + dx,
                        camera_chunk.y + dy,
                        camera_chunk.z + dz,
                    );

                    let distance = coord.distance_to(&camera_chunk);
                    
                    if distance <= view_distance as f32 {
                        self.request_chunk_load_internal(coord, 100 - distance as i32)?;
                    }
                }
            }
        }

        // Schedule chunks for unloading if they're too far
        let unload_distance = view_distance + preload_distance;
        let chunks_to_unload: Vec<ChunkCoordinate> = self.chunks.keys()
            .filter(|&coord| {
                let distance = coord.distance_to(&camera_chunk);
                distance > unload_distance as f32
            })
            .copied()
            .collect();

        for coord in chunks_to_unload {
            if !self.unload_queue.contains(&coord) {
                self.unload_queue.push_back(coord);
            }
        }

        Ok(())
    }

    fn manage_memory_usage(&mut self) -> RobinResult<()> {
        // Force unload chunks if we exceed the limit
        if self.chunks.len() > self.config.max_loaded_chunks {
            let mut chunks_by_access: Vec<(ChunkCoordinate, Instant)> = self.chunks.iter()
                .filter(|(_, chunk)| chunk.can_unload())
                .map(|(&coord, chunk)| (coord, chunk.last_accessed))
                .collect();

            // Sort by last accessed time (oldest first)
            chunks_by_access.sort_by_key(|(_, time)| *time);

            let chunks_to_remove = chunks_by_access.len().saturating_sub(self.config.max_loaded_chunks);
            for (coord, _) in chunks_by_access.into_iter().take(chunks_to_remove) {
                self.unload_chunk(coord)?;
            }
        }

        Ok(())
    }

    fn update_metrics(&mut self) {
        let mut total_chunks = 0;
        let mut loaded_chunks = 0;
        let mut loading_chunks = 0;
        let mut unloading_chunks = 0;
        let mut total_memory = 0;

        for chunk in self.chunks.values() {
            total_chunks += 1;
            match chunk.state {
                ChunkState::Loaded => loaded_chunks += 1,
                ChunkState::Loading => loading_chunks += 1,
                ChunkState::Unloading => unloading_chunks += 1,
                _ => {}
            }
            total_memory += chunk.get_memory_usage();
        }

        self.metrics.total_chunks = total_chunks;
        self.metrics.loaded_chunks = loaded_chunks;
        self.metrics.loading_chunks = loading_chunks;
        self.metrics.unloading_chunks = unloading_chunks;
        self.metrics.memory_usage_mb = total_memory as f32 / (1024.0 * 1024.0);
        self.metrics.load_requests_pending = self.load_queue.len() as u32;
    }

    // Public API
    pub fn request_chunk_load(&mut self, coordinate: ChunkCoordinate) -> RobinResult<()> {
        self.request_chunk_load_internal(coordinate, 50)
    }

    fn request_chunk_load_internal(&mut self, coordinate: ChunkCoordinate, priority: i32) -> RobinResult<()> {
        // Skip if already loaded or loading
        if let Some(chunk) = self.chunks.get(&coordinate) {
            if chunk.is_loaded() || chunk.is_loading() {
                return Ok(());
            }
        }

        // Skip if already in queue
        if self.load_queue.iter().any(|req| req.coordinate == coordinate) {
            return Ok(());
        }

        let distance = coordinate.distance_to(&self.camera_chunk);
        let request = ChunkLoadRequest {
            coordinate,
            priority,
            requested_at: Instant::now(),
            requester_distance: distance,
        };

        self.load_queue.push_back(request);
        Ok(())
    }

    pub fn request_chunk_unload(&mut self, coordinate: ChunkCoordinate) -> RobinResult<()> {
        if let Some(chunk) = self.chunks.get_mut(&coordinate) {
            if chunk.reference_count > 0 {
                chunk.reference_count -= 1;
            }

            if chunk.can_unload() && !self.unload_queue.contains(&coordinate) {
                self.unload_queue.push_back(coordinate);
            }
        }

        Ok(())
    }

    pub fn get_chunk(&self, coordinate: ChunkCoordinate) -> Option<&Chunk> {
        self.chunks.get(&coordinate)
    }

    pub fn get_chunk_mut(&mut self, coordinate: ChunkCoordinate) -> Option<&mut Chunk> {
        if let Some(chunk) = self.chunks.get_mut(&coordinate) {
            chunk.last_accessed = Instant::now();
            Some(chunk)
        } else {
            None
        }
    }

    pub fn is_chunk_loaded(&self, coordinate: ChunkCoordinate) -> bool {
        self.chunks.get(&coordinate).map_or(false, |chunk| chunk.is_loaded())
    }

    pub fn get_loaded_chunk_count(&self) -> u32 {
        self.metrics.loaded_chunks
    }

    pub fn get_unloaded_chunk_count(&self) -> u32 {
        self.metrics.unloads_completed
    }

    pub fn set_quality_multiplier(&mut self, multiplier: f32) {
        self.quality_multiplier = multiplier.clamp(0.1, 2.0);
    }

    pub fn get_quality_multiplier(&self) -> f32 {
        self.quality_multiplier
    }

    pub fn get_metrics(&self) -> &ChunkManagerMetrics {
        &self.metrics
    }

    pub fn get_memory_usage_mb(&self) -> f32 {
        self.metrics.memory_usage_mb
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn clear_cache(&mut self) {
        self.chunk_cache.clear();
    }

    pub fn get_cache_usage(&self) -> (usize, usize) {
        let total_size = self.chunk_cache.values().map(|data| data.size_bytes).sum();
        (self.chunk_cache.len(), total_size)
    }

    pub fn shutdown(&mut self) -> RobinResult<()> {
        if self.enabled {
            println!("Chunk Manager shutdown:");
            println!("  Chunks loaded: {}", self.metrics.loaded_chunks);
            println!("  Peak memory usage: {:.1}MB", self.metrics.memory_usage_mb);
            println!("  Total loads completed: {}", self.metrics.loads_completed);
            println!("  Cache hit rate: {:.1}%", 
                if self.metrics.cache_hits + self.metrics.cache_misses > 0 {
                    self.metrics.cache_hits as f32 / (self.metrics.cache_hits + self.metrics.cache_misses) as f32 * 100.0
                } else {
                    0.0
                });
        }

        self.chunks.clear();
        self.chunk_cache.clear();
        self.load_queue.clear();
        self.unload_queue.clear();
        self.currently_loading.clear();

        Ok(())
    }
}