use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use crate::engine::error::RobinResult;
use crate::engine::performance::CacheSizeLimits;

pub struct CacheManager {
    texture_cache: LRUCache<u64, CachedTexture>,
    mesh_cache: LRUCache<u64, CachedMesh>,
    shader_cache: LRUCache<u64, CachedShader>,
    audio_cache: LRUCache<u64, CachedAudio>,
    script_cache: LRUCache<u64, CachedScript>,
    
    // Statistics
    stats: CacheStats,
    
    // Configuration
    auto_cleanup_enabled: bool,
    last_cleanup: Instant,
    cleanup_interval: Duration,
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub texture_hit_rate: f32,
    pub mesh_hit_rate: f32,
    pub shader_hit_rate: f32,
    pub audio_hit_rate: f32,
    pub script_hit_rate: f32,
    pub total_memory_used: u64,
    pub cache_efficiency: f32,
}

#[derive(Clone)]
pub struct CachedTexture {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub mip_levels: u32,
    pub last_accessed: Instant,
    pub access_count: u32,
    pub creation_cost: Duration,
}

#[derive(Clone)]
pub struct CachedMesh {
    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,
    pub vertex_count: u32,
    pub index_count: u32,
    pub last_accessed: Instant,
    pub access_count: u32,
    pub creation_cost: Duration,
}

#[derive(Clone)]
pub struct CachedShader {
    pub vertex_source: String,
    pub fragment_source: String,
    pub compute_source: Option<String>,
    pub compiled_binary: Vec<u8>,
    pub uniforms: Vec<String>,
    pub last_accessed: Instant,
    pub access_count: u32,
    pub creation_cost: Duration,
}

#[derive(Clone)]
pub struct CachedAudio {
    pub sample_data: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u32,
    pub duration: Duration,
    pub last_accessed: Instant,
    pub access_count: u32,
    pub creation_cost: Duration,
}

#[derive(Clone)]
pub struct CachedScript {
    pub source_code: String,
    pub compiled_bytecode: Vec<u8>,
    pub dependencies: Vec<String>,
    pub last_accessed: Instant,
    pub access_count: u32,
    pub creation_cost: Duration,
}

#[derive(Debug, Clone, Copy)]
pub enum TextureFormat {
    RGBA8,
    RGB8,
    RGBA16F,
    R8,
    RG8,
    Depth24Stencil8,
}

struct LRUCache<K: Clone + std::hash::Hash + Eq, V: Clone> {
    data: HashMap<K, CacheEntry<V>>,
    access_order: Vec<K>,
    max_size: usize,
    max_memory: u64,
    current_memory: u64,
    hits: u64,
    misses: u64,
}

#[derive(Clone)]
struct CacheEntry<V> {
    value: V,
    size: u64,
    last_accessed: Instant,
    access_count: u32,
}

impl<K: Clone + std::hash::Hash + Eq, V: Clone> LRUCache<K, V> {
    fn new(max_size: usize, max_memory: u64) -> Self {
        Self {
            data: HashMap::new(),
            access_order: Vec::new(),
            max_size,
            max_memory,
            current_memory: 0,
            hits: 0,
            misses: 0,
        }
    }
    
    fn get(&mut self, key: &K) -> Option<V> {
        if let Some(entry) = self.data.get_mut(key) {
            entry.last_accessed = Instant::now();
            entry.access_count += 1;
            
            // Update access order
            if let Some(pos) = self.access_order.iter().position(|k| k == key) {
                self.access_order.remove(pos);
            }
            self.access_order.push(key.clone());
            
            self.hits += 1;
            Some(entry.value.clone())
        } else {
            self.misses += 1;
            None
        }
    }
    
    fn insert(&mut self, key: K, value: V, size: u64) -> RobinResult<()> {
        // Remove existing entry if present
        if let Some(old_entry) = self.data.remove(&key) {
            self.current_memory -= old_entry.size;
            if let Some(pos) = self.access_order.iter().position(|k| k == &key) {
                self.access_order.remove(pos);
            }
        }
        
        // Make room if necessary
        while (self.data.len() >= self.max_size || 
               self.current_memory + size > self.max_memory) && 
              !self.access_order.is_empty() {
            
            let lru_key = self.access_order.remove(0);
            if let Some(removed_entry) = self.data.remove(&lru_key) {
                self.current_memory -= removed_entry.size;
            }
        }
        
        // Check if we can fit the new entry
        if size > self.max_memory {
            return Err(crate::engine::error::RobinError::Custom("Item too large for cache".to_string()));
        }
        
        // Insert new entry
        let entry = CacheEntry {
            value,
            size,
            last_accessed: Instant::now(),
            access_count: 1,
        };
        
        self.data.insert(key.clone(), entry);
        self.access_order.push(key);
        self.current_memory += size;
        
        Ok(())
    }
    
    fn remove(&mut self, key: &K) -> Option<V> {
        if let Some(entry) = self.data.remove(key) {
            self.current_memory -= entry.size;
            if let Some(pos) = self.access_order.iter().position(|k| k == key) {
                self.access_order.remove(pos);
            }
            Some(entry.value)
        } else {
            None
        }
    }
    
    fn clear(&mut self) {
        self.data.clear();
        self.access_order.clear();
        self.current_memory = 0;
        self.hits = 0;
        self.misses = 0;
    }
    
    fn hit_rate(&self) -> f32 {
        let total = self.hits + self.misses;
        if total > 0 {
            self.hits as f32 / total as f32
        } else {
            0.0
        }
    }
    
    fn memory_usage(&self) -> u64 {
        self.current_memory
    }
    
    fn entry_count(&self) -> usize {
        self.data.len()
    }
    
    fn cleanup_expired(&mut self, max_age: Duration) -> u32 {
        let now = Instant::now();
        let mut removed_count = 0;
        
        let expired_keys: Vec<K> = self.data.iter()
            .filter(|(_, entry)| now - entry.last_accessed > max_age)
            .map(|(key, _)| key.clone())
            .collect();
        
        for key in expired_keys {
            if self.remove(&key).is_some() {
                removed_count += 1;
            }
        }
        
        removed_count
    }
}

impl CacheManager {
    pub fn new(limits: &CacheSizeLimits) -> RobinResult<Self> {
        Ok(Self {
            texture_cache: LRUCache::new(1000, (limits.texture_cache_mb * 1024.0 * 1024.0) as u64),
            mesh_cache: LRUCache::new(500, (limits.mesh_cache_mb * 1024.0 * 1024.0) as u64),
            shader_cache: LRUCache::new(200, (limits.shader_cache_mb * 1024.0 * 1024.0) as u64),
            audio_cache: LRUCache::new(300, (limits.audio_cache_mb * 1024.0 * 1024.0) as u64),
            script_cache: LRUCache::new(100, (limits.script_cache_mb * 1024.0 * 1024.0) as u64),
            stats: CacheStats {
                texture_hit_rate: 0.0,
                mesh_hit_rate: 0.0,
                shader_hit_rate: 0.0,
                audio_hit_rate: 0.0,
                script_hit_rate: 0.0,
                total_memory_used: 0,
                cache_efficiency: 0.0,
            },
            auto_cleanup_enabled: true,
            last_cleanup: Instant::now(),
            cleanup_interval: Duration::from_secs(30),
        })
    }
    
    // Texture cache methods
    pub fn get_texture(&mut self, texture_id: u64) -> Option<CachedTexture> {
        self.texture_cache.get(&texture_id)
    }
    
    pub fn cache_texture(&mut self, texture_id: u64, texture: CachedTexture) -> RobinResult<()> {
        let size = (texture.data.len() as u64) + 
                  (texture.width * texture.height * 4) as u64; // Approximate GPU memory
        self.texture_cache.insert(texture_id, texture, size)
    }
    
    pub fn remove_texture(&mut self, texture_id: u64) -> Option<CachedTexture> {
        self.texture_cache.remove(&texture_id)
    }
    
    // Mesh cache methods
    pub fn get_mesh(&mut self, mesh_id: u64) -> Option<CachedMesh> {
        self.mesh_cache.get(&mesh_id)
    }
    
    pub fn cache_mesh(&mut self, mesh_id: u64, mesh: CachedMesh) -> RobinResult<()> {
        let size = (mesh.vertices.len() * 4) as u64 + (mesh.indices.len() * 4) as u64;
        self.mesh_cache.insert(mesh_id, mesh, size)
    }
    
    pub fn remove_mesh(&mut self, mesh_id: u64) -> Option<CachedMesh> {
        self.mesh_cache.remove(&mesh_id)
    }
    
    // Shader cache methods
    pub fn get_shader(&mut self, shader_id: u64) -> Option<CachedShader> {
        self.shader_cache.get(&shader_id)
    }
    
    pub fn cache_shader(&mut self, shader_id: u64, shader: CachedShader) -> RobinResult<()> {
        let size = shader.vertex_source.len() as u64 + 
                  shader.fragment_source.len() as u64 +
                  shader.compiled_binary.len() as u64;
        self.shader_cache.insert(shader_id, shader, size)
    }
    
    pub fn remove_shader(&mut self, shader_id: u64) -> Option<CachedShader> {
        self.shader_cache.remove(&shader_id)
    }
    
    // Audio cache methods
    pub fn get_audio(&mut self, audio_id: u64) -> Option<CachedAudio> {
        self.audio_cache.get(&audio_id)
    }
    
    pub fn cache_audio(&mut self, audio_id: u64, audio: CachedAudio) -> RobinResult<()> {
        let size = (audio.sample_data.len() * 4) as u64;
        self.audio_cache.insert(audio_id, audio, size)
    }
    
    pub fn remove_audio(&mut self, audio_id: u64) -> Option<CachedAudio> {
        self.audio_cache.remove(&audio_id)
    }
    
    // Script cache methods
    pub fn get_script(&mut self, script_id: u64) -> Option<CachedScript> {
        self.script_cache.get(&script_id)
    }
    
    pub fn cache_script(&mut self, script_id: u64, script: CachedScript) -> RobinResult<()> {
        let size = script.source_code.len() as u64 + script.compiled_bytecode.len() as u64;
        self.script_cache.insert(script_id, script, size)
    }
    
    pub fn remove_script(&mut self, script_id: u64) -> Option<CachedScript> {
        self.script_cache.remove(&script_id)
    }
    
    // Management methods
    pub fn update_stats(&mut self) {
        self.stats.texture_hit_rate = self.texture_cache.hit_rate();
        self.stats.mesh_hit_rate = self.mesh_cache.hit_rate();
        self.stats.shader_hit_rate = self.shader_cache.hit_rate();
        self.stats.audio_hit_rate = self.audio_cache.hit_rate();
        self.stats.script_hit_rate = self.script_cache.hit_rate();
        
        self.stats.total_memory_used = self.texture_cache.memory_usage() +
                                      self.mesh_cache.memory_usage() +
                                      self.shader_cache.memory_usage() +
                                      self.audio_cache.memory_usage() +
                                      self.script_cache.memory_usage();
        
        // Calculate overall cache efficiency
        let total_hits = self.texture_cache.hits + self.mesh_cache.hits + 
                        self.shader_cache.hits + self.audio_cache.hits + 
                        self.script_cache.hits;
        let total_requests = total_hits + self.texture_cache.misses + 
                           self.mesh_cache.misses + self.shader_cache.misses + 
                           self.audio_cache.misses + self.script_cache.misses;
        
        self.stats.cache_efficiency = if total_requests > 0 {
            total_hits as f32 / total_requests as f32
        } else {
            0.0
        };
        
        // Auto cleanup if enabled
        if self.auto_cleanup_enabled && 
           Instant::now() - self.last_cleanup > self.cleanup_interval {
            let _ = self.cleanup_expired_entries(Duration::from_secs(300)); // 5 minutes
            self.last_cleanup = Instant::now();
        }
    }
    
    pub fn get_stats(&self) -> CacheStats {
        self.stats.clone()
    }
    
    pub fn get_hit_rates(&self) -> crate::engine::performance::CacheHitRates {
        crate::engine::performance::CacheHitRates {
            texture_cache: self.stats.texture_hit_rate,
            mesh_cache: self.stats.mesh_hit_rate,
            shader_cache: self.stats.shader_hit_rate,
            audio_cache: self.stats.audio_hit_rate,
            script_cache: self.stats.script_hit_rate,
        }
    }
    
    pub fn clear_all(&mut self) -> RobinResult<()> {
        self.texture_cache.clear();
        self.mesh_cache.clear();
        self.shader_cache.clear();
        self.audio_cache.clear();
        self.script_cache.clear();
        self.update_stats();
        Ok(())
    }
    
    pub fn cleanup_expired_entries(&mut self, max_age: Duration) -> RobinResult<u32> {
        let mut total_removed = 0;
        
        total_removed += self.texture_cache.cleanup_expired(max_age);
        total_removed += self.mesh_cache.cleanup_expired(max_age);
        total_removed += self.shader_cache.cleanup_expired(max_age);
        total_removed += self.audio_cache.cleanup_expired(max_age);
        total_removed += self.script_cache.cleanup_expired(max_age);
        
        self.update_stats();
        Ok(total_removed)
    }
    
    pub fn get_cache_usage_report(&self) -> CacheUsageReport {
        CacheUsageReport {
            texture_cache: CacheCategoryReport {
                entry_count: self.texture_cache.entry_count(),
                memory_used: self.texture_cache.memory_usage(),
                hit_rate: self.texture_cache.hit_rate(),
                total_hits: self.texture_cache.hits,
                total_misses: self.texture_cache.misses,
            },
            mesh_cache: CacheCategoryReport {
                entry_count: self.mesh_cache.entry_count(),
                memory_used: self.mesh_cache.memory_usage(),
                hit_rate: self.mesh_cache.hit_rate(),
                total_hits: self.mesh_cache.hits,
                total_misses: self.mesh_cache.misses,
            },
            shader_cache: CacheCategoryReport {
                entry_count: self.shader_cache.entry_count(),
                memory_used: self.shader_cache.memory_usage(),
                hit_rate: self.shader_cache.hit_rate(),
                total_hits: self.shader_cache.hits,
                total_misses: self.shader_cache.misses,
            },
            audio_cache: CacheCategoryReport {
                entry_count: self.audio_cache.entry_count(),
                memory_used: self.audio_cache.memory_usage(),
                hit_rate: self.audio_cache.hit_rate(),
                total_hits: self.audio_cache.hits,
                total_misses: self.audio_cache.misses,
            },
            script_cache: CacheCategoryReport {
                entry_count: self.script_cache.entry_count(),
                memory_used: self.script_cache.memory_usage(),
                hit_rate: self.script_cache.hit_rate(),
                total_hits: self.script_cache.hits,
                total_misses: self.script_cache.misses,
            },
            total_memory_used: self.stats.total_memory_used,
            overall_efficiency: self.stats.cache_efficiency,
        }
    }
    
    pub fn optimize_cache_sizes(&mut self, usage_patterns: &CacheUsagePatterns) -> RobinResult<()> {
        // Adjust cache sizes based on usage patterns
        if usage_patterns.texture_access_frequency > 0.8 {
            // Increase texture cache size if heavily used
            self.texture_cache.max_memory = (self.texture_cache.max_memory as f32 * 1.2) as u64;
        }
        
        if usage_patterns.mesh_access_frequency < 0.3 {
            // Decrease mesh cache size if rarely used
            self.mesh_cache.max_memory = (self.mesh_cache.max_memory as f32 * 0.8) as u64;
        }
        
        // Similar optimizations for other caches...
        
        Ok(())
    }
    
    pub fn preload_assets(&mut self, asset_list: &[AssetPreloadInfo]) -> RobinResult<()> {
        for asset in asset_list {
            match &asset.asset_type {
                AssetType::Texture => {
                    // Would load and cache texture
                    println!("Preloading texture: {}", asset.asset_id);
                }
                AssetType::Mesh => {
                    // Would load and cache mesh
                    println!("Preloading mesh: {}", asset.asset_id);
                }
                AssetType::Shader => {
                    // Would compile and cache shader
                    println!("Preloading shader: {}", asset.asset_id);
                }
                AssetType::Audio => {
                    // Would load and cache audio
                    println!("Preloading audio: {}", asset.asset_id);
                }
                AssetType::Script => {
                    // Would compile and cache script
                    println!("Preloading script: {}", asset.asset_id);
                }
            }
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CacheUsageReport {
    pub texture_cache: CacheCategoryReport,
    pub mesh_cache: CacheCategoryReport,
    pub shader_cache: CacheCategoryReport,
    pub audio_cache: CacheCategoryReport,
    pub script_cache: CacheCategoryReport,
    pub total_memory_used: u64,
    pub overall_efficiency: f32,
}

#[derive(Debug, Clone)]
pub struct CacheCategoryReport {
    pub entry_count: usize,
    pub memory_used: u64,
    pub hit_rate: f32,
    pub total_hits: u64,
    pub total_misses: u64,
}

#[derive(Debug, Clone)]
pub struct CacheUsagePatterns {
    pub texture_access_frequency: f32,
    pub mesh_access_frequency: f32,
    pub shader_access_frequency: f32,
    pub audio_access_frequency: f32,
    pub script_access_frequency: f32,
    pub peak_usage_times: Vec<Duration>,
    pub cache_pressure_points: Vec<CachePressurePoint>,
}

#[derive(Debug, Clone)]
pub struct CachePressurePoint {
    pub timestamp: Instant,
    pub cache_type: String,
    pub pressure_level: f32,
    pub memory_usage: u64,
}

#[derive(Debug, Clone)]
pub struct AssetPreloadInfo {
    pub asset_id: u64,
    pub asset_type: AssetType,
    pub priority: PreloadPriority,
    pub estimated_size: u64,
}

#[derive(Debug, Clone)]
pub enum AssetType {
    Texture,
    Mesh,
    Shader,
    Audio,
    Script,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PreloadPriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}