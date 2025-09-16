use crate::engine::error::RobinResult;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, VecDeque, BinaryHeap};
use std::cmp::Ordering;
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingConfig {
    pub max_loading_distance: f32,
    pub max_concurrent_loads: usize,
    pub memory_budget_mb: f32,
    pub enable_predictive_loading: bool,
    pub compression_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChunkPriority {
    Critical = 0, // Player is inside
    High = 1,     // Adjacent to player
    Normal = 2,   // Within view distance
    Low = 3,      // Background loading
    Deferred = 4, // Load when idle
}

#[derive(Debug, Clone)]
pub struct ChunkLoadRequest {
    pub coords: [i32; 3],
    pub priority: ChunkPriority,
    pub distance: f32,
    pub requested_at: Instant,
}

impl PartialEq for ChunkLoadRequest {
    fn eq(&self, other: &Self) -> bool {
        self.coords == other.coords
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
        self.priority.cmp(&other.priority)
            .then_with(|| self.distance.partial_cmp(&other.distance).unwrap_or(Ordering::Equal))
    }
}

#[derive(Debug)]
pub struct ChunkData {
    pub coords: [i32; 3],
    pub data: Vec<u8>,
    pub compressed: bool,
    pub loaded_at: Instant,
    pub last_accessed: Instant,
    pub memory_size: usize,
}

#[derive(Debug)]
pub struct ChunkStreamer {
    config: StreamingConfig,
    loaded_chunks: HashMap<[i32; 3], Arc<Mutex<ChunkData>>>,
    loading_queue: BinaryHeap<ChunkLoadRequest>,
    active_loads: HashMap<[i32; 3], Instant>,
    memory_usage: f32,
    current_position: [f32; 3],
    stats: StreamingStats,
}

#[derive(Debug, Default)]
struct StreamingStats {
    chunks_loaded: u32,
    chunks_unloaded: u32,
    cache_hits: u32,
    cache_misses: u32,
    bytes_loaded: u64,
    bytes_compressed: u64,
}

impl ChunkStreamer {
    pub fn new(config: StreamingConfig) -> RobinResult<Self> {
        Ok(Self {
            config,
            loaded_chunks: HashMap::new(),
            loading_queue: BinaryHeap::new(),
            active_loads: HashMap::new(),
            memory_usage: 0.0,
            current_position: [0.0, 0.0, 0.0],
            stats: StreamingStats::default(),
        })
    }

    pub fn update(&mut self, _delta_time: f32, player_position: [f32; 3]) -> RobinResult<()> {
        self.current_position = player_position;
        
        // Update chunk priorities based on distance
        self.update_chunk_priorities();
        
        // Process loading queue
        self.process_loading_queue()?;
        
        // Unload distant chunks if memory pressure is high
        if self.memory_usage > self.config.memory_budget_mb * 0.8 {
            self.unload_distant_chunks()?;
        }
        
        // Predictive loading
        if self.config.enable_predictive_loading {
            self.predict_and_queue_chunks()?;
        }

        Ok(())
    }

    pub fn request_chunk_load(&mut self, coords: [i32; 3], priority: ChunkPriority) -> RobinResult<()> {
        if self.loaded_chunks.contains_key(&coords) || self.active_loads.contains_key(&coords) {
            return Ok(());
        }

        let distance = self.calculate_distance_to_chunk(coords);
        
        if distance <= self.config.max_loading_distance {
            let request = ChunkLoadRequest {
                coords,
                priority,
                distance,
                requested_at: Instant::now(),
            };
            
            self.loading_queue.push(request);
        }

        Ok(())
    }

    fn process_loading_queue(&mut self) -> RobinResult<()> {
        while self.active_loads.len() < self.config.max_concurrent_loads {
            if let Some(request) = self.loading_queue.pop() {
                if !self.loaded_chunks.contains_key(&request.coords) {
                    self.start_chunk_load(request)?;
                }
            } else {
                break;
            }
        }
        Ok(())
    }

    fn start_chunk_load(&mut self, request: ChunkLoadRequest) -> RobinResult<()> {
        self.active_loads.insert(request.coords, Instant::now());
        
        // Simulate chunk loading - in reality this would be async
        let chunk_data = self.generate_chunk_data(request.coords);
        self.finish_chunk_load(request.coords, chunk_data)?;
        
        Ok(())
    }

    fn finish_chunk_load(&mut self, coords: [i32; 3], data: Vec<u8>) -> RobinResult<()> {
        self.active_loads.remove(&coords);
        
        let memory_size = data.len();
        let chunk = ChunkData {
            coords,
            data,
            compressed: self.config.compression_enabled,
            loaded_at: Instant::now(),
            last_accessed: Instant::now(),
            memory_size,
        };
        
        self.loaded_chunks.insert(coords, Arc::new(Mutex::new(chunk)));
        self.memory_usage += memory_size as f32 / 1_048_576.0; // Convert to MB
        self.stats.chunks_loaded += 1;
        self.stats.bytes_loaded += memory_size as u64;
        
        Ok(())
    }

    pub fn unload_distant_chunks(&mut self) -> RobinResult<()> {
        let max_distance = self.config.max_loading_distance * 1.2; // 20% buffer
        let mut to_unload = Vec::new();
        
        for (coords, chunk_arc) in &self.loaded_chunks {
            let distance = self.calculate_distance_to_chunk(*coords);
            if distance > max_distance {
                to_unload.push(*coords);
            }
        }
        
        for coords in to_unload {
            if let Some(chunk_arc) = self.loaded_chunks.remove(&coords) {
                let chunk = chunk_arc.lock().unwrap();
                self.memory_usage -= chunk.memory_size as f32 / 1_048_576.0;
                self.stats.chunks_unloaded += 1;
            }
        }
        
        Ok(())
    }

    fn update_chunk_priorities(&mut self) {
        let mut new_queue = BinaryHeap::new();
        
        while let Some(mut request) = self.loading_queue.pop() {
            request.distance = self.calculate_distance_to_chunk(request.coords);
            request.priority = self.calculate_chunk_priority(request.coords, request.distance);
            
            if request.distance <= self.config.max_loading_distance {
                new_queue.push(request);
            }
        }
        
        self.loading_queue = new_queue;
    }

    fn predict_and_queue_chunks(&mut self) -> RobinResult<()> {
        // Simple predictive loading: load chunks in movement direction
        let prediction_distance = 64.0; // Predict 64 units ahead
        
        let predicted_coords = self.world_to_chunk_coords([
            self.current_position[0] + prediction_distance,
            self.current_position[1],
            self.current_position[2] + prediction_distance,
        ]);
        
        self.request_chunk_load(predicted_coords, ChunkPriority::Low)?;
        
        Ok(())
    }

    fn calculate_distance_to_chunk(&self, coords: [i32; 3]) -> f32 {
        let chunk_center = [
            coords[0] as f32 * 32.0 + 16.0, // Assuming 32x32 chunk size
            coords[1] as f32 * 32.0 + 16.0,
            coords[2] as f32 * 32.0 + 16.0,
        ];
        
        let dx = self.current_position[0] - chunk_center[0];
        let dy = self.current_position[1] - chunk_center[1];
        let dz = self.current_position[2] - chunk_center[2];
        
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    fn calculate_chunk_priority(&self, coords: [i32; 3], distance: f32) -> ChunkPriority {
        let player_chunk = self.world_to_chunk_coords(self.current_position);
        
        // Player is inside this chunk
        if coords == player_chunk {
            return ChunkPriority::Critical;
        }
        
        // Adjacent chunks
        let dx = (coords[0] - player_chunk[0]).abs();
        let dy = (coords[1] - player_chunk[1]).abs();
        let dz = (coords[2] - player_chunk[2]).abs();
        
        if dx <= 1 && dy <= 1 && dz <= 1 {
            return ChunkPriority::High;
        }
        
        // Based on distance
        if distance < 64.0 {
            ChunkPriority::Normal
        } else if distance < 128.0 {
            ChunkPriority::Low
        } else {
            ChunkPriority::Deferred
        }
    }

    fn world_to_chunk_coords(&self, world_pos: [f32; 3]) -> [i32; 3] {
        [
            (world_pos[0] / 32.0).floor() as i32,
            (world_pos[1] / 32.0).floor() as i32,
            (world_pos[2] / 32.0).floor() as i32,
        ]
    }

    fn generate_chunk_data(&self, coords: [i32; 3]) -> Vec<u8> {
        // Placeholder chunk generation
        let mut data = vec![0u8; 32 * 32 * 32]; // 32KB per chunk
        
        // Simple noise-based generation
        for i in 0..data.len() {
            data[i] = ((coords[0] + coords[1] + coords[2] + i as i32) % 256) as u8;
        }
        
        if self.config.compression_enabled {
            // Simulate compression (would use actual compression in practice)
            data.truncate(data.len() / 2);
        }
        
        data
    }

    pub fn set_loading_distance(&mut self, distance: f32) {
        self.config.max_loading_distance = distance;
    }

    pub fn get_loaded_chunk_count(&self) -> u32 {
        self.loaded_chunks.len() as u32
    }

    pub fn get_memory_usage_mb(&self) -> f32 {
        self.memory_usage
    }

    pub fn shutdown(&mut self) -> RobinResult<()> {
        println!("Chunk streamer shutdown:");
        println!("  Chunks loaded: {}", self.stats.chunks_loaded);
        println!("  Chunks unloaded: {}", self.stats.chunks_unloaded);
        println!("  Memory usage: {:.1} MB", self.memory_usage);
        
        self.loaded_chunks.clear();
        self.loading_queue.clear();
        self.active_loads.clear();
        self.memory_usage = 0.0;
        
        Ok(())
    }
}

pub type ChunkLoadingSystem = ChunkStreamer;