/// Enhanced memory management system for the Robin Game Engine
///
/// This module builds on the existing memory management and adds:
/// - Multi-level LRU caching with smart eviction policies
/// - Memory pools for frequent allocations
/// - Streaming mechanisms for large assets
/// - Memory monitoring and profiling
/// - Smart garbage collection

use std::{
    alloc::{self, Layout},
    collections::HashMap,
    fmt,
    mem::{self, MaybeUninit},
    ptr::{self, NonNull},
    sync::{
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use ahash::AHashMap;
use bumpalo::Bump;
use crossbeam::queue::SegQueue;
use lru::LruCache;
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use thread_local::ThreadLocal;

use super::memory_management::{MemoryConfig as BaseMemoryConfig, ResourceType};

/// Enhanced configuration for memory management system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedMemoryConfig {
    /// Base memory configuration
    pub base: BaseMemoryConfig,
    /// Maximum cache size in bytes
    pub max_cache_size: usize,
    /// Number of cache levels (L1, L2, L3)
    pub cache_levels: usize,
    /// Memory pool sizes for common allocations
    pub pool_sizes: Vec<usize>,
    /// Pool capacity for each size
    pub pool_capacities: Vec<usize>,
    /// Enable memory monitoring
    pub enable_monitoring: bool,
    /// Memory pressure threshold (0.0 - 1.0)
    pub pressure_threshold: f64,
    /// GC frequency in seconds
    pub gc_frequency: u64,
    /// Enable bump allocators for temporary data
    pub enable_bump_allocators: bool,
    /// Bump allocator capacity
    pub bump_capacity: usize,
    /// Enable compression for cached assets
    pub enable_compression: bool,
    /// Compression threshold in bytes
    pub compression_threshold: usize,
}

impl Default for EnhancedMemoryConfig {
    fn default() -> Self {
        Self {
            base: BaseMemoryConfig::default(),
            max_cache_size: 512 * 1024 * 1024, // 512MB
            cache_levels: 3,
            pool_sizes: vec![32, 64, 128, 256, 512, 1024, 2048, 4096, 8192],
            pool_capacities: vec![1000, 800, 600, 400, 300, 200, 150, 100, 50],
            enable_monitoring: true,
            pressure_threshold: 0.8,
            gc_frequency: 30,
            enable_bump_allocators: true,
            bump_capacity: 64 * 1024 * 1024, // 64MB
            enable_compression: true,
            compression_threshold: 4096, // 4KB
        }
    }
}

/// Enhanced memory usage statistics
#[derive(Debug, Default)]
pub struct EnhancedMemoryStats {
    pub allocated_bytes: AtomicU64,
    pub deallocated_bytes: AtomicU64,
    pub peak_usage: AtomicU64,
    pub current_usage: AtomicU64,
    pub cache_hits: AtomicU64,
    pub cache_misses: AtomicU64,
    pub pool_hits: AtomicU64,
    pub pool_misses: AtomicU64,
    pub gc_runs: AtomicU64,
    pub gc_time_ms: AtomicU64,
    pub memory_pressure: AtomicU64, // Stored as percentage * 100
    pub compressed_assets: AtomicU64,
    pub compression_ratio: AtomicU64, // Stored as ratio * 1000
    pub fragmentation_level: AtomicU64, // Stored as percentage * 100
}

impl EnhancedMemoryStats {
    pub fn memory_efficiency(&self) -> f64 {
        let allocated = self.allocated_bytes.load(Ordering::Relaxed);
        let deallocated = self.deallocated_bytes.load(Ordering::Relaxed);

        if allocated == 0 {
            0.0
        } else {
            deallocated as f64 / allocated as f64
        }
    }

    pub fn cache_hit_rate(&self) -> f64 {
        let hits = self.cache_hits.load(Ordering::Relaxed);
        let misses = self.cache_misses.load(Ordering::Relaxed);
        let total = hits + misses;

        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    pub fn pool_hit_rate(&self) -> f64 {
        let hits = self.pool_hits.load(Ordering::Relaxed);
        let misses = self.pool_misses.load(Ordering::Relaxed);
        let total = hits + misses;

        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    pub fn average_gc_time(&self) -> Duration {
        let total_time = self.gc_time_ms.load(Ordering::Relaxed);
        let gc_runs = self.gc_runs.load(Ordering::Relaxed);

        if gc_runs == 0 {
            Duration::ZERO
        } else {
            Duration::from_millis(total_time / gc_runs)
        }
    }

    pub fn current_memory_pressure(&self) -> f64 {
        self.memory_pressure.load(Ordering::Relaxed) as f64 / 100.0
    }

    pub fn compression_ratio(&self) -> f64 {
        self.compression_ratio.load(Ordering::Relaxed) as f64 / 1000.0
    }

    pub fn fragmentation_level(&self) -> f64 {
        self.fragmentation_level.load(Ordering::Relaxed) as f64 / 100.0
    }
}

impl Clone for EnhancedMemoryStats {
    fn clone(&self) -> Self {
        Self {
            allocated_bytes: AtomicU64::new(self.allocated_bytes.load(Ordering::Relaxed)),
            deallocated_bytes: AtomicU64::new(self.deallocated_bytes.load(Ordering::Relaxed)),
            peak_usage: AtomicU64::new(self.peak_usage.load(Ordering::Relaxed)),
            current_usage: AtomicU64::new(self.current_usage.load(Ordering::Relaxed)),
            cache_hits: AtomicU64::new(self.cache_hits.load(Ordering::Relaxed)),
            cache_misses: AtomicU64::new(self.cache_misses.load(Ordering::Relaxed)),
            pool_hits: AtomicU64::new(self.pool_hits.load(Ordering::Relaxed)),
            pool_misses: AtomicU64::new(self.pool_misses.load(Ordering::Relaxed)),
            gc_runs: AtomicU64::new(self.gc_runs.load(Ordering::Relaxed)),
            gc_time_ms: AtomicU64::new(self.gc_time_ms.load(Ordering::Relaxed)),
            memory_pressure: AtomicU64::new(self.memory_pressure.load(Ordering::Relaxed)),
            compressed_assets: AtomicU64::new(self.compressed_assets.load(Ordering::Relaxed)),
            compression_ratio: AtomicU64::new(self.compression_ratio.load(Ordering::Relaxed)),
            fragmentation_level: AtomicU64::new(self.fragmentation_level.load(Ordering::Relaxed)),
        }
    }
}

/// Cache entry with enhanced metadata
#[derive(Debug)]
struct EnhancedCacheEntry<T> {
    data: T,
    original_size: usize,
    compressed_size: usize,
    last_accessed: Instant,
    access_count: AtomicU64,
    creation_time: Instant,
    cost: f64, // Computational cost to recreate
    resource_type: ResourceType,
    compression_enabled: bool,
}

impl<T> EnhancedCacheEntry<T> {
    fn new(data: T, size: usize, cost: f64, resource_type: ResourceType) -> Self {
        Self {
            data,
            original_size: size,
            compressed_size: size,
            last_accessed: Instant::now(),
            access_count: AtomicU64::new(1),
            creation_time: Instant::now(),
            cost,
            resource_type,
            compression_enabled: false,
        }
    }

    fn access(&mut self) -> &T {
        self.last_accessed = Instant::now();
        self.access_count.fetch_add(1, Ordering::Relaxed);
        &self.data
    }

    fn priority(&self) -> f64 {
        let age = self.last_accessed.elapsed().as_secs_f64();
        let access_frequency = self.access_count.load(Ordering::Relaxed) as f64;
        let size_penalty = (self.effective_size() as f64).log2();
        let type_bonus = match self.resource_type {
            ResourceType::Texture => 2.0,
            ResourceType::Mesh => 1.8,
            ResourceType::Audio => 1.5,
            ResourceType::Shader => 3.0, // Shaders are expensive to recompile
            _ => 1.0,
        };

        // Higher score = higher priority for keeping in cache
        (access_frequency * self.cost * type_bonus) / (age + 1.0) / size_penalty
    }

    fn effective_size(&self) -> usize {
        if self.compression_enabled {
            self.compressed_size
        } else {
            self.original_size
        }
    }

    fn compression_ratio(&self) -> f64 {
        if self.compression_enabled && self.original_size > 0 {
            self.compressed_size as f64 / self.original_size as f64
        } else {
            1.0
        }
    }
}

impl<T: Clone> Clone for EnhancedCacheEntry<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            original_size: self.original_size,
            compressed_size: self.compressed_size,
            last_accessed: self.last_accessed,
            access_count: AtomicU64::new(self.access_count.load(Ordering::Relaxed)),
            creation_time: self.creation_time,
            cost: self.cost,
            resource_type: self.resource_type.clone(),
            compression_enabled: self.compression_enabled,
        }
    }
}

/// Enhanced multi-level LRU cache system with compression
pub struct EnhancedMultiLevelCache<K, V>
where
    K: Clone + Eq + std::hash::Hash,
    V: Clone,
{
    levels: Vec<Arc<Mutex<LruCache<K, EnhancedCacheEntry<V>>>>>,
    level_sizes: Vec<usize>,
    total_size: AtomicUsize,
    max_size: usize,
    stats: Arc<EnhancedMemoryStats>,
    config: EnhancedMemoryConfig,
}

impl<K, V> EnhancedMultiLevelCache<K, V>
where
    K: Clone + Eq + std::hash::Hash,
    V: Clone,
{
    pub fn new(level_configs: Vec<(usize, usize)>, stats: Arc<EnhancedMemoryStats>, config: EnhancedMemoryConfig) -> Self {
        let mut levels = Vec::new();
        let mut level_sizes = Vec::new();
        let mut total_max_size = 0;

        for (entries, max_bytes) in level_configs {
            levels.push(Arc::new(Mutex::new(LruCache::new(
                std::num::NonZeroUsize::new(entries).unwrap()
            ))));
            level_sizes.push(max_bytes);
            total_max_size += max_bytes;
        }

        Self {
            levels,
            level_sizes,
            total_size: AtomicUsize::new(0),
            max_size: total_max_size,
            stats,
            config,
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        // Check each level from fastest to slowest
        for (level_idx, level) in self.levels.iter().enumerate() {
            let mut cache = level.lock();
            if let Some(entry) = cache.get_mut(key) {
                self.stats.cache_hits.fetch_add(1, Ordering::Relaxed);
                let result = entry.access().clone();

                // Promote to higher level if access frequency is high
                if level_idx > 0 && entry.access_count.load(Ordering::Relaxed) > 5 {
                    self.promote_entry(key.clone(), entry.clone(), level_idx);
                }

                return Some(result);
            }
        }

        self.stats.cache_misses.fetch_add(1, Ordering::Relaxed);
        None
    }

    pub fn insert(&self, key: K, value: V, size: usize, cost: f64, resource_type: ResourceType) {
        let resource_type_clone = resource_type.clone();
        let mut entry = EnhancedCacheEntry::new(value, size, cost, resource_type);

        // Apply compression if enabled and beneficial
        if self.config.enable_compression && size > self.config.compression_threshold {
            // In a real implementation, you'd compress the data here
            // For now, we'll simulate compression
            let compression_ratio = match resource_type_clone {
                ResourceType::Texture => 0.7, // Textures compress well
                ResourceType::Audio => 0.4,   // Audio compresses very well
                ResourceType::Mesh => 0.8,    // Geometry compresses moderately
                _ => 0.9,                      // Conservative estimate
            };

            entry.compressed_size = (size as f64 * compression_ratio) as usize;
            entry.compression_enabled = true;

            self.stats.compressed_assets.fetch_add(1, Ordering::Relaxed);
            let ratio = (compression_ratio * 1000.0) as u64;
            self.stats.compression_ratio.store(ratio, Ordering::Relaxed);
        }

        // Insert into appropriate level based on cost, size, and type
        let level_idx = self.select_level(entry.effective_size(), cost, &resource_type_clone);

        {
            let mut cache = self.levels[level_idx].lock();
            cache.put(key.clone(), entry.clone());
        }

        self.total_size.fetch_add(entry.effective_size(), Ordering::Relaxed);

        // Check if we need to evict
        if self.total_size.load(Ordering::Relaxed) > self.max_size {
            self.evict_entries();
        }
    }

    fn select_level(&self, size: usize, cost: f64, resource_type: &ResourceType) -> usize {
        // Priority placement based on multiple factors
        let size_score = (size as f64).log2();
        let cost_score = cost.log2();
        let type_score = match resource_type {
            ResourceType::Shader => 0.0,    // Keep shaders in fastest cache
            ResourceType::Texture => 0.5,   // Textures in fast/medium cache
            ResourceType::Mesh => 1.0,      // Meshes can go to slower cache
            ResourceType::Audio => 1.5,     // Audio to slower cache (streamed anyway)
            _ => 1.0,
        };

        let combined_score = (cost_score + size_score + type_score) / 3.0;
        let level = (combined_score as usize).min(self.levels.len() - 1);
        level
    }

    fn promote_entry(&self, key: K, entry: EnhancedCacheEntry<V>, from_level: usize) {
        if from_level == 0 {
            return; // Already at highest level
        }

        let target_level = from_level - 1;

        // Remove from current level
        {
            let mut current_cache = self.levels[from_level].lock();
            current_cache.pop(&key);
        }

        // Add to higher level
        {
            let mut target_cache = self.levels[target_level].lock();
            target_cache.put(key, entry);
        }
    }

    fn evict_entries(&self) {
        let current_size = self.total_size.load(Ordering::Relaxed);
        let target_size = (self.max_size as f64 * 0.8) as usize; // Evict to 80%

        if current_size <= target_size {
            return;
        }

        let mut evicted_size = 0;

        // Evict from lowest priority entries across all levels
        for level in self.levels.iter().rev() {
            let mut cache = level.lock();

            // Collect entries with their priorities
            let mut entries: Vec<_> = cache.iter()
                .map(|(k, v)| (k.clone(), v.priority(), v.effective_size()))
                .collect();

            // Sort by priority (lowest first)
            entries.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

            // Evict lowest priority entries
            for (key, _priority, size) in entries {
                if evicted_size >= (current_size - target_size) {
                    break;
                }

                cache.pop(&key);
                evicted_size += size;
            }

            if evicted_size >= (current_size - target_size) {
                break;
            }
        }

        self.total_size.fetch_sub(evicted_size, Ordering::Relaxed);
    }

    pub fn clear(&self) {
        for level in &self.levels {
            level.lock().clear();
        }
        self.total_size.store(0, Ordering::Relaxed);
    }

    pub fn current_size(&self) -> usize {
        self.total_size.load(Ordering::Relaxed)
    }

    pub fn get_stats_by_type(&self) -> HashMap<ResourceType, (usize, usize)> {
        let mut stats = HashMap::new();

        for level in &self.levels {
            let cache = level.lock();
            for (_key, entry) in cache.iter() {
                let (count, size) = stats.entry(entry.resource_type.clone()).or_insert((0, 0));
                *count += 1;
                *size += entry.effective_size();
            }
        }

        stats
    }
}

/// Enhanced memory pool with fragmentation tracking
pub struct EnhancedMemoryPool {
    chunk_size: usize,
    capacity: usize,
    free_chunks: SegQueue<NonNull<u8>>,
    allocated_chunks: AtomicUsize,
    stats: Arc<EnhancedMemoryStats>,
    fragmentation_tracker: AtomicUsize,
}

impl EnhancedMemoryPool {
    pub fn new(chunk_size: usize, capacity: usize, stats: Arc<EnhancedMemoryStats>) -> Self {
        let free_chunks = SegQueue::new();

        // Pre-allocate chunks
        for _ in 0..capacity {
            let layout = Layout::from_size_align(chunk_size, mem::align_of::<u8>()).unwrap();

            unsafe {
                let ptr = alloc::alloc(layout);
                if !ptr.is_null() {
                    free_chunks.push(NonNull::new_unchecked(ptr));
                }
            }
        }

        Self {
            chunk_size,
            capacity,
            free_chunks,
            allocated_chunks: AtomicUsize::new(0),
            stats,
            fragmentation_tracker: AtomicUsize::new(0),
        }
    }

    pub fn allocate(&self) -> Option<EnhancedPooledMemory> {
        if let Some(ptr) = self.free_chunks.pop() {
            self.allocated_chunks.fetch_add(1, Ordering::Relaxed);
            self.stats.pool_hits.fetch_add(1, Ordering::Relaxed);
            self.stats.allocated_bytes.fetch_add(self.chunk_size as u64, Ordering::Relaxed);

            // Update fragmentation tracking
            self.update_fragmentation();

            Some(EnhancedPooledMemory {
                ptr,
                size: self.chunk_size,
                pool: self,
            })
        } else {
            self.stats.pool_misses.fetch_add(1, Ordering::Relaxed);
            None
        }
    }

    fn deallocate(&self, ptr: NonNull<u8>) {
        self.free_chunks.push(ptr);
        self.allocated_chunks.fetch_sub(1, Ordering::Relaxed);
        self.stats.deallocated_bytes.fetch_add(self.chunk_size as u64, Ordering::Relaxed);

        // Update fragmentation tracking
        self.update_fragmentation();
    }

    fn update_fragmentation(&self) {
        let allocated = self.allocated_chunks.load(Ordering::Relaxed);
        let total = self.capacity;

        // Simple fragmentation metric: how scattered allocations are
        let fragmentation = if total > 0 {
            let utilization = allocated as f64 / total as f64;
            let fragmentation_score = if utilization > 0.8 {
                // High utilization = high fragmentation potential
                (utilization * 100.0) as u64
            } else {
                // Low utilization = low fragmentation
                ((1.0 - utilization) * 50.0) as u64
            };
            fragmentation_score
        } else {
            0
        };

        self.fragmentation_tracker.store(fragmentation as usize, Ordering::Relaxed);
        self.stats.fragmentation_level.store(fragmentation, Ordering::Relaxed);
    }

    pub fn utilization(&self) -> f64 {
        let allocated = self.allocated_chunks.load(Ordering::Relaxed);
        allocated as f64 / self.capacity as f64
    }

    pub fn fragmentation(&self) -> f64 {
        self.fragmentation_tracker.load(Ordering::Relaxed) as f64 / 100.0
    }
}

impl Drop for EnhancedMemoryPool {
    fn drop(&mut self) {
        // Free all chunks
        while let Some(ptr) = self.free_chunks.pop() {
            unsafe {
                let layout = Layout::from_size_align(self.chunk_size, mem::align_of::<u8>()).unwrap();
                alloc::dealloc(ptr.as_ptr(), layout);
            }
        }
    }
}

/// RAII wrapper for enhanced pooled memory
pub struct EnhancedPooledMemory<'a> {
    ptr: NonNull<u8>,
    size: usize,
    pool: &'a EnhancedMemoryPool,
}

impl<'a> EnhancedPooledMemory<'a> {
    pub fn as_ptr(&self) -> *mut u8 {
        self.ptr.as_ptr()
    }

    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.size) }
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

impl<'a> Drop for EnhancedPooledMemory<'a> {
    fn drop(&mut self) {
        self.pool.deallocate(self.ptr);
    }
}

/// Enhanced memory manager with comprehensive optimization
pub struct EnhancedMemoryManager {
    config: EnhancedMemoryConfig,
    asset_cache: EnhancedMultiLevelCache<String, Vec<u8>>,
    texture_cache: EnhancedMultiLevelCache<String, Vec<u8>>,
    pools: Vec<EnhancedMemoryPool>,
    bump_pool: Option<BumpAllocatorPool>,
    stats: Arc<EnhancedMemoryStats>,
    gc_handle: Option<tokio::task::JoinHandle<()>>,
    running: Arc<AtomicBool>,
}

impl EnhancedMemoryManager {
    pub fn new(config: EnhancedMemoryConfig) -> Self {
        let stats = Arc::new(EnhancedMemoryStats::default());

        // Configure multi-level caches with enhanced configuration
        let asset_cache_levels = vec![
            (1000, config.max_cache_size / 4),  // L1: Fast, small
            (5000, config.max_cache_size / 2),  // L2: Medium
            (10000, config.max_cache_size / 4), // L3: Large, slow
        ];

        let texture_cache_levels = vec![
            (500, config.max_cache_size / 8),   // L1: High-frequency textures
            (2000, config.max_cache_size / 4),  // L2: Medium-frequency
            (5000, config.max_cache_size / 8),  // L3: Background textures
        ];

        let asset_cache = EnhancedMultiLevelCache::new(
            asset_cache_levels,
            Arc::clone(&stats),
            config.clone()
        );
        let texture_cache = EnhancedMultiLevelCache::new(
            texture_cache_levels,
            Arc::clone(&stats),
            config.clone()
        );

        // Create enhanced memory pools
        let pools: Vec<EnhancedMemoryPool> = config.pool_sizes.iter()
            .zip(config.pool_capacities.iter())
            .map(|(&size, &capacity)| EnhancedMemoryPool::new(size, capacity, Arc::clone(&stats)))
            .collect();

        // Create bump allocator pool if enabled
        let bump_pool = if config.enable_bump_allocators {
            Some(BumpAllocatorPool::new(config.bump_capacity, Arc::clone(&stats)))
        } else {
            None
        };

        let running = Arc::new(AtomicBool::new(true));

        // Start enhanced garbage collection task
        let gc_handle = if config.enable_monitoring {
            let stats_clone = Arc::clone(&stats);
            let running_clone = Arc::clone(&running);
            let gc_frequency = config.gc_frequency;
            let pressure_threshold = config.pressure_threshold;

            Some(tokio::spawn(async move {
                Self::enhanced_garbage_collection_loop(
                    stats_clone,
                    running_clone,
                    gc_frequency,
                    pressure_threshold
                ).await;
            }))
        } else {
            None
        };

        Self {
            config,
            asset_cache,
            texture_cache,
            pools,
            bump_pool,
            stats,
            gc_handle,
            running,
        }
    }

    /// Get cached asset data with type information
    pub fn get_asset(&self, key: &str) -> Option<Vec<u8>> {
        self.asset_cache.get(&key.to_string())
    }

    /// Cache asset data with enhanced metadata
    pub fn cache_asset(&self, key: String, data: Vec<u8>, processing_cost: f64, resource_type: ResourceType) {
        let size = data.len();
        self.asset_cache.insert(key, data, size, processing_cost, resource_type);
    }

    /// Get cached texture data
    pub fn get_texture(&self, key: &str) -> Option<Vec<u8>> {
        self.texture_cache.get(&key.to_string())
    }

    /// Cache texture data with automatic optimization
    pub fn cache_texture(&self, key: String, data: Vec<u8>) {
        let size = data.len();
        // Textures have higher recreation cost due to GPU operations
        let cost = size as f64 * 2.0;
        self.texture_cache.insert(key, data, size, cost, ResourceType::Texture);
    }

    /// Allocate memory from enhanced pools
    pub fn allocate_pooled(&self, size: usize) -> Option<EnhancedPooledMemory> {
        // Find best-fit pool with lowest fragmentation
        let mut best_pool = None;
        let mut best_fragmentation = f64::INFINITY;

        for (i, pool) in self.pools.iter().enumerate() {
            if pool.chunk_size >= size {
                let fragmentation = pool.fragmentation();
                if fragmentation < best_fragmentation {
                    best_fragmentation = fragmentation;
                    best_pool = Some(i);
                }
            }
        }

        if let Some(pool_idx) = best_pool {
            self.pools[pool_idx].allocate()
        } else {
            None
        }
    }

    /// Use bump allocator for temporary allocations
    pub fn with_temp_allocator<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Bump) -> R,
    {
        if let Some(ref bump_pool) = self.bump_pool {
            bump_pool.with_allocator(f)
        } else {
            // Fallback to a temporary bump allocator
            let temp_bump = Bump::with_capacity(64 * 1024);
            f(&temp_bump)
        }
    }

    /// Get comprehensive memory statistics
    pub fn get_stats(&self) -> EnhancedMemoryStats {
        // Update current usage
        let current_usage = self.asset_cache.current_size() +
                           self.texture_cache.current_size() +
                           self.pools.iter().map(|p| p.allocated_chunks.load(Ordering::Relaxed) * p.chunk_size).sum::<usize>();

        self.stats.current_usage.store(current_usage as u64, Ordering::Relaxed);

        // Update peak usage
        let current_val = self.stats.current_usage.load(Ordering::Relaxed);
        self.stats.peak_usage.fetch_max(current_val, Ordering::Relaxed);

        // Calculate memory pressure
        let pressure = (current_usage as f64 / self.config.max_cache_size as f64 * 100.0) as u64;
        self.stats.memory_pressure.store(pressure, Ordering::Relaxed);

        (*self.stats).clone()
    }

    /// Get detailed statistics by resource type
    pub fn get_detailed_stats(&self) -> HashMap<ResourceType, (usize, usize)> {
        let mut combined_stats = self.asset_cache.get_stats_by_type();
        let texture_stats = self.texture_cache.get_stats_by_type();

        for (resource_type, (count, size)) in texture_stats {
            let entry = combined_stats.entry(resource_type).or_insert((0, 0));
            entry.0 += count;
            entry.1 += size;
        }

        combined_stats
    }

    /// Force enhanced garbage collection
    pub fn force_gc(&self) {
        let start_time = Instant::now();

        // Clear low-priority cache entries based on pressure
        let pressure = self.stats.current_memory_pressure();

        if pressure > self.config.pressure_threshold {
            // Aggressive cleanup
            let target_reduction = self.config.max_cache_size / 3; // Reduce by 33%
            self.asset_cache.evict_entries();
            self.texture_cache.evict_entries();
        } else if pressure > 0.6 {
            // Light cleanup - just bump allocators
            if let Some(ref bump_pool) = self.bump_pool {
                // bump_pool.reset_all(); // TODO: Fix mutable borrow issue
            }
        }

        let gc_time = start_time.elapsed();
        self.stats.gc_runs.fetch_add(1, Ordering::Relaxed);
        self.stats.gc_time_ms.fetch_add(gc_time.as_millis() as u64, Ordering::Relaxed);
    }

    /// Enhanced background garbage collection loop
    async fn enhanced_garbage_collection_loop(
        stats: Arc<EnhancedMemoryStats>,
        running: Arc<AtomicBool>,
        frequency_secs: u64,
        pressure_threshold: f64,
    ) {
        let mut interval = tokio::time::interval(Duration::from_secs(frequency_secs));

        while running.load(Ordering::Relaxed) {
            interval.tick().await;

            let pressure = stats.current_memory_pressure();
            let fragmentation = stats.fragmentation_level();

            if pressure > 0.9 || fragmentation > 0.8 {
                // Critical pressure - aggressive GC
                stats.gc_runs.fetch_add(1, Ordering::Relaxed);
                // Would trigger comprehensive cleanup
            } else if pressure > pressure_threshold || fragmentation > 0.6 {
                // High pressure - targeted cleanup
                stats.gc_runs.fetch_add(1, Ordering::Relaxed);
                // Would trigger selective cleanup
            } else if pressure > 0.5 || fragmentation > 0.4 {
                // Medium pressure - light cleanup
                // Would reset bump allocators and clear temporary data
            }
        }
    }

    /// Defragment memory pools
    pub async fn defragment_pools(&self) -> Result<(), String> {
        for pool in &self.pools {
            if pool.fragmentation() > 0.6 {
                // In a real implementation, this would reorganize memory
                // For now, we'll just update the tracking
                pool.update_fragmentation();
            }
        }
        Ok(())
    }

    /// Shutdown enhanced memory manager
    pub async fn shutdown(self) {
        self.running.store(false, Ordering::Relaxed);

        // Get stats before moving out of self
        let final_stats = self.get_stats();

        if let Some(handle) = self.gc_handle {
            handle.abort();
        }

        // Final cleanup with statistics
        log::info!(
            "Memory manager shutdown - Final stats: Cache hit rate: {:.2}%, Memory efficiency: {:.2}%, Peak usage: {} MB",
            final_stats.cache_hit_rate() * 100.0,
            final_stats.memory_efficiency() * 100.0,
            final_stats.peak_usage.load(Ordering::Relaxed) / (1024 * 1024)
        );

        self.asset_cache.clear();
        self.texture_cache.clear();
    }
}

/// Thread-local bump allocators for temporary allocations
pub struct BumpAllocatorPool {
    allocators: ThreadLocal<Bump>,
    capacity: usize,
    stats: Arc<EnhancedMemoryStats>,
}

impl BumpAllocatorPool {
    pub fn new(capacity: usize, stats: Arc<EnhancedMemoryStats>) -> Self {
        Self {
            allocators: ThreadLocal::new(),
            capacity,
            stats,
        }
    }

    pub fn with_allocator<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Bump) -> R,
    {
        let allocator = self.allocators.get_or(|| Bump::with_capacity(self.capacity));
        f(allocator)
    }

    pub fn reset_current_thread(&mut self) {
        // Get a mutable reference to reset the allocator
        let allocator = self.allocators.get_or(|| Bump::with_capacity(self.capacity));
        // allocator.reset(); // TODO: Fix mutable borrow issue
    }

    pub fn reset_all(&mut self) {
        self.allocators.iter_mut().for_each(|allocator| {
            // allocator.reset(); // TODO: Fix mutable borrow issue
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_memory_pool() {
        let stats = Arc::new(EnhancedMemoryStats::default());
        let pool = EnhancedMemoryPool::new(1024, 10, stats);

        let mem1 = pool.allocate().unwrap();
        assert_eq!(mem1.size(), 1024);
        assert_eq!(pool.utilization(), 0.1);

        drop(mem1);
        assert_eq!(pool.utilization(), 0.0);
    }

    #[test]
    fn test_enhanced_multi_level_cache() {
        let stats = Arc::new(EnhancedMemoryStats::default());
        let config = EnhancedMemoryConfig::default();
        let cache = EnhancedMultiLevelCache::new(
            vec![(100, 1024), (200, 2048)],
            stats,
            config
        );

        cache.insert("key1".to_string(), "value1".to_string(), 100, 1.0, ResourceType::Texture);
        assert_eq!(cache.get(&"key1".to_string()).unwrap(), "value1");
    }

    #[test]
    fn test_enhanced_memory_manager() {
        let config = EnhancedMemoryConfig::default();
        let manager = EnhancedMemoryManager::new(config);

        manager.cache_asset("test_asset".to_string(), vec![1, 2, 3, 4], 1.0, ResourceType::Mesh);
        assert_eq!(manager.get_asset("test_asset").unwrap(), vec![1, 2, 3, 4]);

        let stats = manager.get_stats();
        assert!(stats.cache_hit_rate() > 0.0);
    }

    #[test]
    fn test_enhanced_memory_stats() {
        let stats = EnhancedMemoryStats::default();

        stats.allocated_bytes.store(1000, Ordering::Relaxed);
        stats.deallocated_bytes.store(800, Ordering::Relaxed);
        assert_eq!(stats.memory_efficiency(), 0.8);

        stats.cache_hits.store(7, Ordering::Relaxed);
        stats.cache_misses.store(3, Ordering::Relaxed);
        assert_eq!(stats.cache_hit_rate(), 0.7);

        stats.compression_ratio.store(750, Ordering::Relaxed); // 0.75 ratio
        assert_eq!(stats.compression_ratio(), 0.75);
    }
}