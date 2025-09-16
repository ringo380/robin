use crate::engine::error::RobinResult;
use std::collections::{HashMap, BTreeMap, VecDeque};
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant};
use std::alloc::{GlobalAlloc, Layout, System};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    pub total_budget_mb: f32,
    pub texture_budget_mb: f32,
    pub mesh_budget_mb: f32,
    pub audio_budget_mb: f32,
    pub world_data_budget_mb: f32,
    pub enable_garbage_collection: bool,
    pub gc_threshold_percentage: f32,
    pub enable_memory_pools: bool,
    pub enable_compression: bool,
    pub memory_warning_threshold: f32,
    pub memory_critical_threshold: f32,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            total_budget_mb: 2048.0, // 2GB default
            texture_budget_mb: 512.0,
            mesh_budget_mb: 256.0,
            audio_budget_mb: 128.0,
            world_data_budget_mb: 1024.0,
            enable_garbage_collection: true,
            gc_threshold_percentage: 80.0,
            enable_memory_pools: true,
            enable_compression: true,
            memory_warning_threshold: 85.0,
            memory_critical_threshold: 95.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceType {
    Texture,
    Mesh,
    Audio,
    WorldData,
    Shader,
    Animation,
    Script,
    UserInterface,
    Physics,
    Temporary,
}

#[derive(Debug, Clone)]
pub enum AllocationStrategy {
    FirstFit,
    BestFit,
    WorstFit,
    NextFit,
    Pooled,
}

#[derive(Debug, Clone)]
pub struct MemoryBlock {
    pub id: String,
    pub ptr: usize, // Simulated memory address
    pub size: usize,
    pub alignment: usize,
    pub resource_type: ResourceType,
    pub allocated_at: Instant,
}

impl MemoryBlock {
    pub fn new(id: String, ptr: usize, size: usize, alignment: usize, resource_type: ResourceType) -> Self {
        Self {
            id,
            ptr,
            size,
            alignment,
            resource_type,
            allocated_at: Instant::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ResourcePriority {
    Critical = 0,    // Never garbage collect
    High = 1,        // Collect only under severe pressure
    Normal = 2,      // Standard collection priority
    Low = 3,         // First to be collected
    Cache = 4,       // Aggressive collection
}

#[derive(Debug)]
pub struct ResourceMetadata {
    pub id: String,
    pub resource_type: ResourceType,
    pub priority: ResourcePriority,
    pub size_bytes: usize,
    pub created_at: Instant,
    pub last_accessed: Instant,
    pub access_count: u32,
    pub compressed: bool,
    pub ref_count: usize,
}

#[derive(Debug)]
pub struct MemoryPool {
    pub pool_type: ResourceType,
    pub chunk_size: usize,
    pub max_chunks: usize,
    pub available_chunks: VecDeque<Vec<u8>>,
    pub allocated_chunks: HashMap<String, Vec<u8>>,
    pub total_allocated: usize,
}

#[derive(Debug)]
pub struct MemoryBudget {
    pub resource_type: ResourceType,
    pub allocated_mb: f32,
    pub budget_mb: f32,
    pub peak_usage_mb: f32,
    pub allocation_count: u32,
}

#[derive(Debug, Default, Clone)]
pub struct MemoryStats {
    pub total_allocated_mb: f32,
    pub total_freed_mb: f32,
    pub peak_usage_mb: f32,
    pub allocation_count: u64,
    pub deallocation_count: u64,
    pub gc_runs: u32,
    pub gc_freed_mb: f32,
    pub compression_saved_mb: f32,
    pub pool_hit_ratio: f32,
    pub fragmentation_ratio: f32,
}

pub struct MemoryManager {
    config: MemoryConfig,
    resources: HashMap<String, Arc<Mutex<ResourceMetadata>>>,
    memory_pools: HashMap<ResourceType, MemoryPool>,
    budgets: HashMap<ResourceType, MemoryBudget>,
    stats: MemoryStats,
    last_gc_time: Instant,
    warning_callbacks: Vec<Box<dyn Fn(f32) + Send + Sync>>,
    critical_callbacks: Vec<Box<dyn Fn(f32) + Send + Sync>>,
    allocation_history: VecDeque<(Instant, f32)>,
    allocation_strategy: AllocationStrategy,
    allocated_blocks: HashMap<String, MemoryBlock>,
    next_address: usize,
    budget_mb: u32,
}

impl std::fmt::Debug for MemoryManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryManager")
            .field("config", &self.config)
            .field("resources", &self.resources)
            .field("memory_pools", &self.memory_pools)
            .field("budgets", &self.budgets)
            .field("stats", &self.stats)
            .field("last_gc_time", &self.last_gc_time)
            .field("warning_callbacks", &format!("<{} callbacks>", self.warning_callbacks.len()))
            .field("critical_callbacks", &format!("<{} callbacks>", self.critical_callbacks.len()))
            .field("allocation_history", &self.allocation_history)
            .field("allocation_strategy", &self.allocation_strategy)
            .field("allocated_blocks", &self.allocated_blocks)
            .field("next_address", &self.next_address)
            .field("budget_mb", &self.budget_mb)
            .finish()
    }
}

impl MemoryPool {
    pub fn new(pool_type: ResourceType, chunk_size: usize, max_chunks: usize) -> Self {
        Self {
            pool_type,
            chunk_size,
            max_chunks,
            available_chunks: VecDeque::new(),
            allocated_chunks: HashMap::new(),
            total_allocated: 0,
        }
    }

    pub fn allocate(&mut self, id: &str) -> Option<Vec<u8>> {
        if let Some(chunk) = self.available_chunks.pop_front() {
            self.allocated_chunks.insert(id.to_string(), chunk.clone());
            self.total_allocated += self.chunk_size;
            Some(chunk)
        } else if self.allocated_chunks.len() < self.max_chunks {
            let chunk = vec![0u8; self.chunk_size];
            self.allocated_chunks.insert(id.to_string(), chunk.clone());
            self.total_allocated += self.chunk_size;
            Some(chunk)
        } else {
            None
        }
    }

    pub fn deallocate(&mut self, id: &str) -> bool {
        if let Some(chunk) = self.allocated_chunks.remove(id) {
            self.available_chunks.push_back(chunk);
            self.total_allocated = self.total_allocated.saturating_sub(self.chunk_size);
            true
        } else {
            false
        }
    }

    pub fn get_utilization(&self) -> f32 {
        if self.max_chunks == 0 {
            0.0
        } else {
            self.allocated_chunks.len() as f32 / self.max_chunks as f32
        }
    }

    pub fn shrink(&mut self, target_size: usize) {
        while self.available_chunks.len() > target_size {
            self.available_chunks.pop_back();
        }
    }
}

impl MemoryManager {
    pub fn new(budget_mb: u32) -> RobinResult<Self> {
        let mut config = MemoryConfig::default();
        config.total_budget_mb = budget_mb as f32;
        
        let mut manager = Self {
            config: config.clone(),
            resources: HashMap::new(),
            memory_pools: HashMap::new(),
            budgets: HashMap::new(),
            stats: MemoryStats::default(),
            last_gc_time: Instant::now(),
            warning_callbacks: Vec::new(),
            critical_callbacks: Vec::new(),
            allocation_history: VecDeque::new(),
            allocation_strategy: AllocationStrategy::Pooled,
            allocated_blocks: HashMap::new(),
            next_address: 0x10000000, // Start at a reasonable simulated address
            budget_mb,
        };

        manager.initialize_pools()?;
        manager.initialize_budgets()?;
        
        Ok(manager)
    }

    pub fn from_config(config: MemoryConfig) -> RobinResult<Self> {
        let budget_mb = config.total_budget_mb as u32;
        
        let mut manager = Self {
            config: config.clone(),
            resources: HashMap::new(),
            memory_pools: HashMap::new(),
            budgets: HashMap::new(),
            stats: MemoryStats::default(),
            last_gc_time: Instant::now(),
            warning_callbacks: Vec::new(),
            critical_callbacks: Vec::new(),
            allocation_history: VecDeque::new(),
            allocation_strategy: AllocationStrategy::Pooled,
            allocated_blocks: HashMap::new(),
            next_address: 0x10000000,
            budget_mb,
        };

        manager.initialize_pools()?;
        manager.initialize_budgets()?;
        
        Ok(manager)
    }

    pub fn initialize(&mut self) -> RobinResult<()> {
        println!("Memory Manager initialized:");
        println!("  Total Budget: {} MB", self.budget_mb);
        println!("  GC Enabled: {}", self.config.enable_garbage_collection);
        println!("  Memory Pools: {}", self.config.enable_memory_pools);
        println!("  Compression: {}", self.config.enable_compression);
        Ok(())
    }

    pub fn allocate_resource(&mut self, 
        id: &str, 
        size_bytes: usize, 
        resource_type: ResourceType, 
        priority: ResourcePriority
    ) -> RobinResult<bool> {
        if self.resources.contains_key(id) {
            return Ok(false); // Resource already exists
        }

        let size_mb = size_bytes as f32 / 1_048_576.0;
        
        // Check budget constraints
        if !self.check_budget_availability(resource_type.clone(), size_mb)? {
            if self.config.enable_garbage_collection {
                self.run_garbage_collection()?;
                if !self.check_budget_availability(resource_type.clone(), size_mb)? {
                    return Ok(false); // Still can't allocate after GC
                }
            } else {
                return Ok(false);
            }
        }

        let metadata = ResourceMetadata {
            id: id.to_string(),
            resource_type: resource_type.clone(),
            priority,
            size_bytes,
            created_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 1,
            compressed: false,
            ref_count: 1,
        };

        // Try pool allocation first
        let allocated_from_pool = if self.config.enable_memory_pools {
            self.try_pool_allocation(id, &resource_type, size_bytes)?
        } else {
            false
        };

        if !allocated_from_pool {
            // Direct allocation
            self.stats.allocation_count += 1;
        }

        // Update budget tracking
        if let Some(budget) = self.budgets.get_mut(&resource_type) {
            budget.allocated_mb += size_mb;
            budget.peak_usage_mb = budget.peak_usage_mb.max(budget.allocated_mb);
            budget.allocation_count += 1;
        }

        // Update global stats
        self.stats.total_allocated_mb += size_mb;
        self.stats.peak_usage_mb = self.stats.peak_usage_mb.max(self.stats.total_allocated_mb);
        
        // Record allocation history for trending
        self.allocation_history.push_back((Instant::now(), self.stats.total_allocated_mb));
        if self.allocation_history.len() > 1000 {
            self.allocation_history.pop_front();
        }

        self.resources.insert(id.to_string(), Arc::new(Mutex::new(metadata)));

        // Check for memory pressure warnings
        self.check_memory_pressure();

        Ok(true)
    }

    pub fn deallocate_resource(&mut self, id: &str) -> RobinResult<bool> {
        if let Some(resource_arc) = self.resources.remove(id) {
            let metadata = resource_arc.lock().unwrap();
            let size_mb = metadata.size_bytes as f32 / 1_048_576.0;
            let resource_type = metadata.resource_type.clone();
            
            // Try pool deallocation first
            if self.config.enable_memory_pools {
                if let Some(pool) = self.memory_pools.get_mut(&resource_type) {
                    pool.deallocate(id);
                }
            }

            // Update budget tracking
            if let Some(budget) = self.budgets.get_mut(&resource_type) {
                budget.allocated_mb = (budget.allocated_mb - size_mb).max(0.0);
            }

            // Update global stats
            self.stats.total_freed_mb += size_mb;
            self.stats.total_allocated_mb = (self.stats.total_allocated_mb - size_mb).max(0.0);
            self.stats.deallocation_count += 1;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn access_resource(&mut self, id: &str) -> RobinResult<()> {
        if let Some(resource_arc) = self.resources.get(id) {
            let mut metadata = resource_arc.lock().unwrap();
            metadata.last_accessed = Instant::now();
            metadata.access_count += 1;
        }
        Ok(())
    }

    pub fn update(&mut self, delta_time: f32) -> RobinResult<()> {
        // Periodic garbage collection
        if self.config.enable_garbage_collection {
            let time_since_gc = self.last_gc_time.elapsed();
            let current_usage_percent = self.get_memory_usage_percentage();
            
            let should_gc = time_since_gc > Duration::from_secs(30) || 
                           current_usage_percent > self.config.gc_threshold_percentage;

            if should_gc {
                self.run_garbage_collection()?;
            }
        }

        // Pool optimization
        if self.config.enable_memory_pools {
            self.optimize_pools(delta_time)?;
        }

        // Update fragmentation metrics
        self.update_fragmentation_metrics();

        Ok(())
    }

    pub fn run_garbage_collection(&mut self) -> RobinResult<f32> {
        let start_time = Instant::now();
        let initial_memory = self.stats.total_allocated_mb;
        let mut freed_mb = 0.0;
        let mut resources_to_remove = Vec::new();

        // Identify candidates for collection
        for (id, resource_arc) in &self.resources {
            let metadata = resource_arc.lock().unwrap();
            let age = metadata.created_at.elapsed();
            let idle_time = metadata.last_accessed.elapsed();
            
            let should_collect = match metadata.priority {
                ResourcePriority::Critical => false,
                ResourcePriority::High => idle_time > Duration::from_secs(300), // 5 minutes
                ResourcePriority::Normal => idle_time > Duration::from_secs(120), // 2 minutes
                ResourcePriority::Low => idle_time > Duration::from_secs(60), // 1 minute
                ResourcePriority::Cache => idle_time > Duration::from_secs(10), // 10 seconds
            };

            if should_collect && metadata.ref_count <= 1 {
                resources_to_remove.push(id.clone());
                freed_mb += metadata.size_bytes as f32 / 1_048_576.0;
            }
        }

        // Actually remove the resources
        for id in resources_to_remove {
            self.deallocate_resource(&id)?;
        }

        // Compress remaining resources if enabled
        if self.config.enable_compression {
            freed_mb += self.compress_resources()?;
        }

        // Shrink pools
        for pool in self.memory_pools.values_mut() {
            let target_size = (pool.max_chunks as f32 * 0.7) as usize;
            pool.shrink(target_size);
        }

        self.last_gc_time = Instant::now();
        self.stats.gc_runs += 1;
        self.stats.gc_freed_mb += freed_mb;

        println!("GC completed: freed {:.1} MB in {:?}", freed_mb, start_time.elapsed());
        Ok(freed_mb)
    }

    fn compress_resources(&mut self) -> RobinResult<f32> {
        let mut compression_saved = 0.0;
        
        for resource_arc in self.resources.values() {
            let mut metadata = resource_arc.lock().unwrap();
            if !metadata.compressed && metadata.size_bytes > 1024 { // Only compress > 1KB
                let original_size = metadata.size_bytes;
                let compressed_size = (original_size as f32 * 0.6) as usize; // Simulate 40% compression
                let saved = (original_size - compressed_size) as f32 / 1_048_576.0;
                
                compression_saved += saved;
                metadata.compressed = true;
                metadata.size_bytes = compressed_size;
            }
        }

        self.stats.compression_saved_mb += compression_saved;
        Ok(compression_saved)
    }

    fn initialize_pools(&mut self) -> RobinResult<()> {
        if !self.config.enable_memory_pools {
            return Ok(());
        }

        let pool_configs = [
            (ResourceType::Texture, 1024 * 1024, 50),    // 1MB chunks, 50 max
            (ResourceType::Mesh, 512 * 1024, 100),       // 512KB chunks, 100 max  
            (ResourceType::Audio, 256 * 1024, 30),       // 256KB chunks, 30 max
            (ResourceType::WorldData, 64 * 1024, 200),   // 64KB chunks, 200 max
            (ResourceType::Shader, 32 * 1024, 50),       // 32KB chunks, 50 max
            (ResourceType::Temporary, 4 * 1024, 500),    // 4KB chunks, 500 max
        ];

        for (resource_type, chunk_size, max_chunks) in pool_configs {
            let pool = MemoryPool::new(resource_type.clone(), chunk_size, max_chunks);
            self.memory_pools.insert(resource_type, pool);
        }

        Ok(())
    }

    fn initialize_budgets(&mut self) -> RobinResult<()> {
        let budget_configs = [
            (ResourceType::Texture, self.config.texture_budget_mb),
            (ResourceType::Mesh, self.config.mesh_budget_mb),
            (ResourceType::Audio, self.config.audio_budget_mb),
            (ResourceType::WorldData, self.config.world_data_budget_mb),
            (ResourceType::Shader, 64.0),
            (ResourceType::Animation, 128.0),
            (ResourceType::Script, 32.0),
            (ResourceType::UserInterface, 64.0),
            (ResourceType::Physics, 128.0),
            (ResourceType::Temporary, 256.0),
        ];

        for (resource_type, budget_mb) in budget_configs {
            let budget = MemoryBudget {
                resource_type: resource_type.clone(),
                allocated_mb: 0.0,
                budget_mb,
                peak_usage_mb: 0.0,
                allocation_count: 0,
            };
            self.budgets.insert(resource_type, budget);
        }

        Ok(())
    }

    fn try_pool_allocation(&mut self, id: &str, resource_type: &ResourceType, size_bytes: usize) -> RobinResult<bool> {
        if let Some(pool) = self.memory_pools.get_mut(resource_type) {
            if size_bytes <= pool.chunk_size {
                if let Some(_chunk) = pool.allocate(id) {
                    self.stats.pool_hit_ratio = 
                        (self.stats.pool_hit_ratio * 0.95) + 0.05; // Moving average
                    return Ok(true);
                }
            }
        }
        
        self.stats.pool_hit_ratio *= 0.95; // Decay on miss
        Ok(false)
    }

    fn check_budget_availability(&self, resource_type: ResourceType, size_mb: f32) -> RobinResult<bool> {
        if let Some(budget) = self.budgets.get(&resource_type) {
            Ok(budget.allocated_mb + size_mb <= budget.budget_mb)
        } else {
            Ok(true)
        }
    }

    fn check_memory_pressure(&self) {
        let usage_percent = self.get_memory_usage_percentage();
        
        if usage_percent >= self.config.memory_critical_threshold {
            for callback in &self.critical_callbacks {
                callback(usage_percent);
            }
        } else if usage_percent >= self.config.memory_warning_threshold {
            for callback in &self.warning_callbacks {
                callback(usage_percent);
            }
        }
    }

    fn optimize_pools(&mut self, _delta_time: f32) -> RobinResult<()> {
        for pool in self.memory_pools.values_mut() {
            let utilization = pool.get_utilization();
            
            // Shrink under-utilized pools
            if utilization < 0.3 && pool.available_chunks.len() > 10 {
                let target_size = pool.available_chunks.len() / 2;
                pool.shrink(target_size);
            }
        }
        Ok(())
    }

    fn update_fragmentation_metrics(&mut self) {
        let total_pools_capacity: usize = self.memory_pools.values()
            .map(|p| p.max_chunks * p.chunk_size)
            .sum();
        
        let total_pools_used: usize = self.memory_pools.values()
            .map(|p| p.allocated_chunks.len() * p.chunk_size)
            .sum();

        if total_pools_capacity > 0 {
            self.stats.fragmentation_ratio = 1.0 - (total_pools_used as f32 / total_pools_capacity as f32);
        }
    }

    pub fn get_memory_usage_percentage(&self) -> f32 {
        (self.stats.total_allocated_mb / self.config.total_budget_mb) * 100.0
    }

    pub fn get_memory_usage_mb(&self) -> f32 {
        self.stats.total_allocated_mb
    }

    pub fn get_budget_usage(&self, resource_type: &ResourceType) -> Option<(f32, f32)> {
        self.budgets.get(resource_type).map(|b| (b.allocated_mb, b.budget_mb))
    }

    pub fn get_resource_count(&self) -> usize {
        self.resources.len()
    }

    pub fn get_pool_stats(&self, resource_type: &ResourceType) -> Option<(usize, usize, f32)> {
        self.memory_pools.get(resource_type).map(|p| 
            (p.allocated_chunks.len(), p.max_chunks, p.get_utilization())
        )
    }

    pub fn get_stats(&self) -> &MemoryStats {
        &self.stats
    }

    pub fn add_warning_callback<F>(&mut self, callback: F) 
    where 
        F: Fn(f32) + Send + Sync + 'static 
    {
        self.warning_callbacks.push(Box::new(callback));
    }

    pub fn add_critical_callback<F>(&mut self, callback: F) 
    where 
        F: Fn(f32) + Send + Sync + 'static 
    {
        self.critical_callbacks.push(Box::new(callback));
    }

    pub fn force_garbage_collection(&mut self) -> RobinResult<f32> {
        self.run_garbage_collection()
    }

    pub fn defragment(&mut self) -> RobinResult<()> {
        // Rebuild memory pools to reduce fragmentation
        for pool in self.memory_pools.values_mut() {
            // Compact available chunks
            pool.available_chunks = pool.available_chunks
                .iter()
                .take(pool.max_chunks / 2)
                .cloned()
                .collect();
        }
        Ok(())
    }

    // New API methods expected by the performance system
    pub fn allocate(&mut self, size_bytes: usize, alignment: usize) -> RobinResult<MemoryBlock> {
        let block_id = format!("block_{}", self.next_address);
        
        // Align the address
        let aligned_address = (self.next_address + alignment - 1) & !(alignment - 1);
        
        // Check if we have enough budget
        let size_mb = size_bytes as f32 / 1_048_576.0;
        if self.stats.total_allocated_mb + size_mb > self.budget_mb as f32 {
            if self.config.enable_garbage_collection {
                self.run_garbage_collection()?;
                if self.stats.total_allocated_mb + size_mb > self.budget_mb as f32 {
                    return Err(crate::engine::error::RobinError::new(
                        "Memory allocation failed: insufficient budget"
                    ));
                }
            } else {
                return Err(crate::engine::error::RobinError::new(
                    "Memory allocation failed: insufficient budget"
                ));
            }
        }

        let block = MemoryBlock::new(
            block_id.clone(),
            aligned_address,
            size_bytes,
            alignment,
            ResourceType::Temporary,
        );

        self.allocated_blocks.insert(block_id, block.clone());
        self.next_address = aligned_address + size_bytes;
        
        // Update stats
        self.stats.total_allocated_mb += size_mb;
        self.stats.allocation_count += 1;
        self.stats.peak_usage_mb = self.stats.peak_usage_mb.max(self.stats.total_allocated_mb);

        Ok(block)
    }

    pub fn deallocate(&mut self, block: MemoryBlock) -> RobinResult<()> {
        if let Some(_removed_block) = self.allocated_blocks.remove(&block.id) {
            let size_mb = block.size as f32 / 1_048_576.0;
            self.stats.total_allocated_mb = (self.stats.total_allocated_mb - size_mb).max(0.0);
            self.stats.total_freed_mb += size_mb;
            self.stats.deallocation_count += 1;
        }
        Ok(())
    }

    pub fn get_usage_mb(&self) -> f32 {
        self.stats.total_allocated_mb
    }

    pub fn shutdown(&mut self) -> RobinResult<()> {
        println!("Memory Manager shutdown:");
        println!("  Total allocated: {:.1} MB", self.stats.total_allocated_mb);
        println!("  Peak usage: {:.1} MB", self.stats.peak_usage_mb);
        println!("  GC runs: {}", self.stats.gc_runs);
        println!("  Compression saved: {:.1} MB", self.stats.compression_saved_mb);
        println!("  Active blocks: {}", self.allocated_blocks.len());
        
        self.resources.clear();
        self.memory_pools.clear();
        self.budgets.clear();
        self.allocation_history.clear();
        self.warning_callbacks.clear();
        self.critical_callbacks.clear();
        self.allocated_blocks.clear();
        
        Ok(())
    }
}

pub type ResourceManager = MemoryManager;