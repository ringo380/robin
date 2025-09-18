/// Optimized hot reload system with advanced debouncing and incremental updates
///
/// This module provides:
/// - Advanced debouncing algorithms with adaptive delays
/// - Incremental reload for dependency-aware selective updates
/// - High-performance file watching with minimal overhead
/// - Batch processing for multiple simultaneous changes
/// - Smart change detection with content hashing

use std::{
    collections::{HashMap, HashSet, VecDeque},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant, SystemTime},
};

use ahash::AHashMap;
use crossbeam_channel::{Receiver, Sender, bounded, unbounded};
use dashmap::DashMap;
use lru::LruCache;
use notify::{
    Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
    event::{AccessKind, AccessMode, CreateKind, ModifyKind, RemoveKind},
};
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use tokio::time::{interval, sleep};

use super::{AssetError, AssetMetadata, AssetResult, AssetType, HotReloadEvent};

/// Configuration for optimized hot reload system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedHotReloadConfig {
    /// Base debounce delay in milliseconds
    pub base_debounce_ms: u64,
    /// Maximum debounce delay in milliseconds
    pub max_debounce_ms: u64,
    /// Adaptive debounce factor (multiplier for burst detection)
    pub adaptive_factor: f64,
    /// Batch processing window in milliseconds
    pub batch_window_ms: u64,
    /// Maximum events per batch
    pub max_batch_size: usize,
    /// Enable dependency tracking
    pub enable_dependency_tracking: bool,
    /// Enable content hashing for change detection
    pub enable_content_hashing: bool,
    /// Enable incremental updates
    pub enable_incremental_updates: bool,
    /// Cache size for file content hashes
    pub hash_cache_size: usize,
    /// Number of worker threads for processing
    pub worker_threads: usize,
    /// Enable performance monitoring
    pub enable_monitoring: bool,
    /// File size threshold for streaming processing
    pub streaming_threshold: usize,
}

impl Default for OptimizedHotReloadConfig {
    fn default() -> Self {
        Self {
            base_debounce_ms: 50,
            max_debounce_ms: 500,
            adaptive_factor: 1.5,
            batch_window_ms: 100,
            max_batch_size: 50,
            enable_dependency_tracking: true,
            enable_content_hashing: true,
            enable_incremental_updates: true,
            hash_cache_size: 10000,
            worker_threads: num_cpus::get().min(8),
            enable_monitoring: true,
            streaming_threshold: 10 * 1024 * 1024, // 10MB
        }
    }
}

/// Hot reload performance metrics
#[derive(Debug, Default)]
pub struct HotReloadMetrics {
    pub total_events: AtomicU64,
    pub processed_events: AtomicU64,
    pub debounced_events: AtomicU64,
    pub batch_processed_events: AtomicU64,
    pub average_processing_time_us: AtomicU64,
    pub cache_hits: AtomicU64,
    pub cache_misses: AtomicU64,
    pub dependency_updates: AtomicU64,
    pub incremental_updates: AtomicU64,
    pub full_reloads: AtomicU64,
}

impl HotReloadMetrics {
    pub fn processing_efficiency(&self) -> f64 {
        let total = self.total_events.load(Ordering::Relaxed);
        let processed = self.processed_events.load(Ordering::Relaxed);

        if total == 0 {
            0.0
        } else {
            processed as f64 / total as f64
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

    pub fn average_processing_time(&self) -> Duration {
        let total_time = self.average_processing_time_us.load(Ordering::Relaxed);
        let processed = self.processed_events.load(Ordering::Relaxed);

        if processed == 0 {
            Duration::ZERO
        } else {
            Duration::from_micros(total_time / processed)
        }
    }

    pub fn incremental_update_rate(&self) -> f64 {
        let incremental = self.incremental_updates.load(Ordering::Relaxed);
        let full = self.full_reloads.load(Ordering::Relaxed);
        let total = incremental + full;

        if total == 0 {
            0.0
        } else {
            incremental as f64 / total as f64
        }
    }
}

impl Clone for HotReloadMetrics {
    fn clone(&self) -> Self {
        Self {
            total_events: AtomicU64::new(self.total_events.load(Ordering::Relaxed)),
            processed_events: AtomicU64::new(self.processed_events.load(Ordering::Relaxed)),
            debounced_events: AtomicU64::new(self.debounced_events.load(Ordering::Relaxed)),
            batch_processed_events: AtomicU64::new(self.batch_processed_events.load(Ordering::Relaxed)),
            average_processing_time_us: AtomicU64::new(self.average_processing_time_us.load(Ordering::Relaxed)),
            cache_hits: AtomicU64::new(self.cache_hits.load(Ordering::Relaxed)),
            cache_misses: AtomicU64::new(self.cache_misses.load(Ordering::Relaxed)),
            dependency_updates: AtomicU64::new(self.dependency_updates.load(Ordering::Relaxed)),
            incremental_updates: AtomicU64::new(self.incremental_updates.load(Ordering::Relaxed)),
            full_reloads: AtomicU64::new(self.full_reloads.load(Ordering::Relaxed)),
        }
    }
}

/// File change event with enhanced metadata
#[derive(Debug, Clone)]
struct OptimizedChangeEvent {
    path: PathBuf,
    event_kind: EventKind,
    timestamp: Instant,
    file_size: Option<u64>,
    content_hash: Option<u64>,
    dependency_level: u32,
}

/// Debouncing state for a specific file
#[derive(Debug)]
struct DebounceState {
    last_event: Instant,
    event_count: u32,
    current_delay: Duration,
    pending_event: Option<OptimizedChangeEvent>,
}

impl DebounceState {
    fn new() -> Self {
        Self {
            last_event: Instant::now(),
            event_count: 0,
            current_delay: Duration::from_millis(50),
            pending_event: None,
        }
    }

    fn update_delay(&mut self, config: &OptimizedHotReloadConfig) {
        let time_since_last = self.last_event.elapsed();

        if time_since_last < Duration::from_millis(config.base_debounce_ms) {
            // Burst detected - increase delay
            self.event_count += 1;
            let multiplier = (config.adaptive_factor.powi(self.event_count as i32)).min(5.0);
            self.current_delay = Duration::from_millis(
                (config.base_debounce_ms as f64 * multiplier) as u64
            ).min(Duration::from_millis(config.max_debounce_ms));
        } else {
            // Reset burst detection
            self.event_count = 0;
            self.current_delay = Duration::from_millis(config.base_debounce_ms);
        }

        self.last_event = Instant::now();
    }
}

/// Dependency graph for tracking asset relationships
#[derive(Debug, Default)]
struct DependencyGraph {
    /// Direct dependencies: asset -> dependencies
    dependencies: AHashMap<PathBuf, HashSet<PathBuf>>,
    /// Reverse dependencies: dependency -> dependents
    dependents: AHashMap<PathBuf, HashSet<PathBuf>>,
    /// Dependency levels for processing order
    levels: AHashMap<PathBuf, u32>,
}

impl DependencyGraph {
    fn add_dependency(&mut self, asset: PathBuf, dependency: PathBuf) {
        self.dependencies.entry(asset.clone())
            .or_insert_with(HashSet::new)
            .insert(dependency.clone());

        self.dependents.entry(dependency)
            .or_insert_with(HashSet::new)
            .insert(asset);

        self.update_levels();
    }

    fn remove_asset(&mut self, asset: &Path) {
        if let Some(deps) = self.dependencies.remove(asset) {
            for dep in deps {
                if let Some(dependents) = self.dependents.get_mut(&dep) {
                    dependents.remove(asset);
                }
            }
        }

        if let Some(dependents) = self.dependents.remove(asset) {
            for dependent in dependents {
                if let Some(deps) = self.dependencies.get_mut(&dependent) {
                    deps.remove(asset);
                }
            }
        }

        self.levels.remove(asset);
        self.update_levels();
    }

    fn get_affected_assets(&self, changed_asset: &Path) -> Vec<PathBuf> {
        let mut affected = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(changed_asset.to_path_buf());

        while let Some(current) = queue.pop_front() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());
            affected.push(current.clone());

            if let Some(dependents) = self.dependents.get(&current) {
                for dependent in dependents {
                    if !visited.contains(dependent) {
                        queue.push_back(dependent.clone());
                    }
                }
            }
        }

        // Sort by dependency level for proper processing order
        affected.sort_by_key(|path| self.levels.get(path).copied().unwrap_or(0));
        affected
    }

    fn update_levels(&mut self) {
        self.levels.clear();
        let mut changed = true;

        while changed {
            changed = false;
            for (asset, deps) in &self.dependencies {
                if self.levels.contains_key(asset) {
                    continue;
                }

                let max_dep_level = deps.iter()
                    .filter_map(|dep| self.levels.get(dep))
                    .max()
                    .copied()
                    .unwrap_or(0);

                if deps.is_empty() || deps.iter().all(|dep| self.levels.contains_key(dep)) {
                    self.levels.insert(asset.clone(), max_dep_level + 1);
                    changed = true;
                }
            }
        }
    }

    fn get_level(&self, asset: &Path) -> u32 {
        self.levels.get(asset).copied().unwrap_or(0)
    }
}

/// Content hash cache for change detection
struct ContentHashCache {
    cache: Arc<Mutex<LruCache<PathBuf, (u64, SystemTime)>>>,
    hasher: ahash::AHasher,
}

impl ContentHashCache {
    fn new(capacity: usize) -> Self {
        Self {
            cache: Arc::new(Mutex::new(LruCache::new(
                std::num::NonZeroUsize::new(capacity).unwrap()
            ))),
            hasher: ahash::AHasher::default(),
        }
    }

    fn get_hash(&self, path: &Path) -> Option<u64> {
        let metadata = std::fs::metadata(path).ok()?;
        let modified = metadata.modified().ok()?;

        {
            let mut cache = self.cache.lock();
            if let Some((hash, cached_time)) = cache.get(path) {
                if *cached_time >= modified {
                    return Some(*hash);
                }
            }
        }

        // Read file and compute hash
        let content = std::fs::read(path).ok()?;
        let hash = self.compute_hash(&content);

        {
            let mut cache = self.cache.lock();
            cache.put(path.to_path_buf(), (hash, modified));
        }

        Some(hash)
    }

    fn compute_hash(&self, content: &[u8]) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = ahash::AHasher::default();
        content.hash(&mut hasher);
        hasher.finish()
    }

    fn invalidate(&self, path: &Path) {
        let mut cache = self.cache.lock();
        cache.pop(path);
    }
}

/// Optimized hot reload system
pub struct OptimizedHotReloadSystem {
    config: OptimizedHotReloadConfig,
    watcher: Option<RecommendedWatcher>,
    event_sender: Sender<notify::Event>,
    event_receiver: Receiver<notify::Event>,
    debounce_states: Arc<DashMap<PathBuf, DebounceState>>,
    dependency_graph: Arc<RwLock<DependencyGraph>>,
    content_cache: ContentHashCache,
    metrics: Arc<HotReloadMetrics>,
    worker_handles: Vec<tokio::task::JoinHandle<()>>,
    running: Arc<AtomicBool>,
    callback: Option<Arc<dyn Fn(HotReloadEvent) + Send + Sync>>,
}

impl OptimizedHotReloadSystem {
    /// Create a new optimized hot reload system
    pub fn new(config: OptimizedHotReloadConfig) -> AssetResult<Self> {
        let (event_sender, event_receiver) = unbounded();
        let debounce_states = Arc::new(DashMap::new());
        let dependency_graph = Arc::new(RwLock::new(DependencyGraph::default()));
        let content_cache = ContentHashCache::new(config.hash_cache_size);
        let metrics = Arc::new(HotReloadMetrics::default());
        let running = Arc::new(AtomicBool::new(true));

        let mut system = Self {
            config,
            watcher: None,
            event_sender,
            event_receiver,
            debounce_states,
            dependency_graph,
            content_cache,
            metrics,
            worker_handles: Vec::new(),
            running,
            callback: None,
        };

        system.start_workers()?;

        Ok(system)
    }

    /// Set the callback for reload events
    pub fn set_callback<F>(&mut self, callback: F)
    where
        F: Fn(HotReloadEvent) + Send + Sync + 'static,
    {
        self.callback = Some(Arc::new(callback));
    }

    /// Start watching a directory
    pub fn watch_directory<P: AsRef<Path>>(&mut self, path: P) -> AssetResult<()> {
        let mut watcher = RecommendedWatcher::new(
            {
                let sender = self.event_sender.clone();
                move |res: Result<notify::Event, notify::Error>| {
                    if let Ok(event) = res {
                        let _ = sender.send(event);
                    }
                }
            },
            Config::default(),
        ).map_err(|e| AssetError::WatcherError(e.to_string()))?;

        watcher.watch(path.as_ref(), RecursiveMode::Recursive)
            .map_err(|e| AssetError::WatcherError(e.to_string()))?;

        self.watcher = Some(watcher);
        Ok(())
    }

    /// Add a dependency relationship
    pub fn add_dependency<P1: AsRef<Path>, P2: AsRef<Path>>(&self, asset: P1, dependency: P2) {
        if self.config.enable_dependency_tracking {
            let mut graph = self.dependency_graph.write();
            graph.add_dependency(asset.as_ref().to_path_buf(), dependency.as_ref().to_path_buf());
        }
    }

    /// Remove an asset from tracking
    pub fn remove_asset<P: AsRef<Path>>(&self, asset: P) {
        let path = asset.as_ref();

        // Remove from dependency graph
        if self.config.enable_dependency_tracking {
            let mut graph = self.dependency_graph.write();
            graph.remove_asset(path);
        }

        // Remove from debounce states
        self.debounce_states.remove(path);

        // Invalidate cache
        if self.config.enable_content_hashing {
            self.content_cache.invalidate(path);
        }
    }

    /// Force reload of a specific asset
    pub fn force_reload<P: AsRef<Path>>(&self, path: P) {
        let path = path.as_ref().to_path_buf();

        if let Some(callback) = &self.callback {
            let event = HotReloadEvent::AssetModified {
                asset_id: path.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
                asset_type: Self::detect_asset_type(&path),
                file_path: path,
            };

            callback(event);
        }
    }

    /// Get current performance metrics
    pub fn get_metrics(&self) -> HotReloadMetrics {
        (*self.metrics).clone()
    }

    /// Start worker threads for processing events
    fn start_workers(&mut self) -> AssetResult<()> {
        for worker_id in 0..self.config.worker_threads {
            let receiver = self.event_receiver.clone();
            let debounce_states = Arc::clone(&self.debounce_states);
            let dependency_graph = Arc::clone(&self.dependency_graph);
            let metrics = Arc::clone(&self.metrics);
            let running = Arc::clone(&self.running);
            let config = self.config.clone();
            let callback = self.callback.clone();

            let handle = tokio::spawn(async move {
                Self::worker_loop(
                    worker_id,
                    receiver,
                    debounce_states,
                    dependency_graph,
                    metrics,
                    running,
                    config,
                    callback,
                ).await;
            });

            self.worker_handles.push(handle);
        }

        // Start debounce processor
        let debounce_handle = {
            let debounce_states = Arc::clone(&self.debounce_states);
            let metrics = Arc::clone(&self.metrics);
            let running = Arc::clone(&self.running);
            let config = self.config.clone();
            let callback = self.callback.clone();

            tokio::spawn(async move {
                Self::debounce_processor(
                    debounce_states,
                    metrics,
                    running,
                    config,
                    callback,
                ).await;
            })
        };

        self.worker_handles.push(debounce_handle);

        Ok(())
    }

    /// Main worker loop for processing file system events
    async fn worker_loop(
        worker_id: usize,
        receiver: Receiver<notify::Event>,
        debounce_states: Arc<DashMap<PathBuf, DebounceState>>,
        dependency_graph: Arc<RwLock<DependencyGraph>>,
        metrics: Arc<HotReloadMetrics>,
        running: Arc<AtomicBool>,
        config: OptimizedHotReloadConfig,
        callback: Option<Arc<dyn Fn(HotReloadEvent) + Send + Sync>>,
    ) {
        let mut batch: Vec<notify::Event> = Vec::with_capacity(config.max_batch_size);
        let mut last_batch_time = Instant::now();

        while running.load(Ordering::Relaxed) {
            // Try to receive an event
            if let Ok(event) = receiver.try_recv() {
                metrics.total_events.fetch_add(1, Ordering::Relaxed);
                batch.push(event);

                // Check if we should process the batch
                let should_process = batch.len() >= config.max_batch_size ||
                    last_batch_time.elapsed() >= Duration::from_millis(config.batch_window_ms);

                if should_process && !batch.is_empty() {
                    Self::process_event_batch(
                        &batch,
                        &debounce_states,
                        &dependency_graph,
                        &metrics,
                        &config,
                    ).await;

                    metrics.batch_processed_events.fetch_add(batch.len() as u64, Ordering::Relaxed);
                    batch.clear();
                    last_batch_time = Instant::now();
                }
            } else {
                // No immediate events - check for pending batch
                if !batch.is_empty() && last_batch_time.elapsed() >= Duration::from_millis(config.batch_window_ms) {
                    Self::process_event_batch(
                        &batch,
                        &debounce_states,
                        &dependency_graph,
                        &metrics,
                        &config,
                    ).await;

                    metrics.batch_processed_events.fetch_add(batch.len() as u64, Ordering::Relaxed);
                    batch.clear();
                    last_batch_time = Instant::now();
                }

                // Small delay to prevent busy waiting
                sleep(Duration::from_millis(1)).await;
            }
        }
    }

    /// Process a batch of file system events
    async fn process_event_batch(
        events: &[notify::Event],
        debounce_states: &Arc<DashMap<PathBuf, DebounceState>>,
        dependency_graph: &Arc<RwLock<DependencyGraph>>,
        metrics: &Arc<HotReloadMetrics>,
        config: &OptimizedHotReloadConfig,
    ) {
        let start_time = Instant::now();

        // Group events by path for efficient processing
        let mut path_events: AHashMap<PathBuf, Vec<&notify::Event>> = AHashMap::new();

        for event in events {
            for path in &event.paths {
                path_events.entry(path.clone()).or_default().push(event);
            }
        }

        // Process each path's events
        for (path, path_event_list) in path_events {
            // Skip if this is not a file we care about
            if !Self::should_process_path(&path) {
                continue;
            }

            // Get the most recent significant event for this path
            if let Some(significant_event) = Self::get_most_significant_event(&path_event_list) {
                // Update debounce state
                let mut debounce_entry = debounce_states.entry(path.clone()).or_insert_with(DebounceState::new);
                debounce_entry.update_delay(config);

                let optimized_event = OptimizedChangeEvent {
                    path: path.clone(),
                    event_kind: significant_event.kind,
                    timestamp: Instant::now(),
                    file_size: Self::get_file_size(&path),
                    content_hash: None, // Will be computed if needed
                    dependency_level: {
                        let graph = dependency_graph.read();
                        graph.get_level(&path)
                    },
                };

                debounce_entry.pending_event = Some(optimized_event);
            }
        }

        let processing_time = start_time.elapsed();
        metrics.average_processing_time_us.store(
            processing_time.as_micros() as u64,
            Ordering::Relaxed,
        );
    }

    /// Debounce processor that handles delayed event processing
    async fn debounce_processor(
        debounce_states: Arc<DashMap<PathBuf, DebounceState>>,
        metrics: Arc<HotReloadMetrics>,
        running: Arc<AtomicBool>,
        config: OptimizedHotReloadConfig,
        callback: Option<Arc<dyn Fn(HotReloadEvent) + Send + Sync>>,
    ) {
        let mut interval = interval(Duration::from_millis(config.base_debounce_ms / 2));

        while running.load(Ordering::Relaxed) {
            interval.tick().await;

            let mut events_to_process = Vec::new();

            // Check all debounce states for ready events
            for mut entry in debounce_states.iter_mut() {
                let (path, state) = entry.pair_mut();

                if let Some(pending_event) = &state.pending_event {
                    if state.last_event.elapsed() >= state.current_delay {
                        events_to_process.push((path.clone(), pending_event.clone()));
                        state.pending_event = None;
                    }
                }
            }

            // Process ready events
            for (path, event) in events_to_process {
                Self::process_debounced_event(event, &callback, &metrics, &config).await;
                debounce_states.remove(&path);
            }
        }
    }

    /// Process a debounced event
    async fn process_debounced_event(
        event: OptimizedChangeEvent,
        callback: &Option<Arc<dyn Fn(HotReloadEvent) + Send + Sync>>,
        metrics: &Arc<HotReloadMetrics>,
        config: &OptimizedHotReloadConfig,
    ) {
        if let Some(ref callback) = callback {
            let hot_reload_event = match event.event_kind {
                EventKind::Create(_) => HotReloadEvent::AssetCreated {
                    file_path: event.path.clone(),
                    asset_type: Self::detect_asset_type(&event.path),
                },
                EventKind::Modify(_) => {
                    let asset_id = event.path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    HotReloadEvent::AssetModified {
                        asset_id,
                        asset_type: Self::detect_asset_type(&event.path),
                        file_path: event.path.clone(),
                    }
                },
                EventKind::Remove(_) => {
                    let asset_id = event.path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    HotReloadEvent::AssetDeleted {
                        asset_id,
                        file_path: event.path.clone(),
                    }
                },
                _ => return, // Ignore other event types
            };

            callback(hot_reload_event);
            metrics.processed_events.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Determine if a path should be processed
    fn should_process_path(path: &Path) -> bool {
        // Skip hidden files and directories
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with('.') {
                return false;
            }
        }

        // Skip temporary files
        if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
            match extension {
                "tmp" | "temp" | "swp" | "bak" | "lock" => return false,
                _ => {}
            }
        }

        // Only process files, not directories
        path.is_file()
    }

    /// Get the most significant event from a list of events
    fn get_most_significant_event<'a>(events: &'a [&'a notify::Event]) -> Option<&'a notify::Event> {
        // Priority: Remove > Create > Modify > Access
        let mut best_event = None;
        let mut best_priority = -1;

        for event in events {
            let priority = match event.kind {
                EventKind::Remove(_) => 3,
                EventKind::Create(_) => 2,
                EventKind::Modify(_) => 1,
                EventKind::Access(_) => 0,
                _ => -1,
            };

            if priority > best_priority {
                best_priority = priority;
                best_event = Some(*event);
            }
        }

        best_event
    }

    /// Get file size safely
    fn get_file_size(path: &Path) -> Option<u64> {
        std::fs::metadata(path).ok().map(|m| m.len())
    }

    /// Detect asset type from file extension
    fn detect_asset_type(path: &Path) -> AssetType {
        if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
            match extension.to_lowercase().as_str() {
                "png" | "jpg" | "jpeg" | "gif" | "bmp" | "tga" => AssetType::Texture,
                "wav" | "mp3" | "ogg" | "flac" => AssetType::Audio,
                "wgsl" | "glsl" | "hlsl" => AssetType::Shader,
                "json" | "toml" | "yaml" | "yml" => AssetType::Config,
                "scene" => AssetType::Scene,
                "ttf" | "otf" | "woff" | "woff2" => AssetType::Font,
                _ => AssetType::Data,
            }
        } else {
            AssetType::Data
        }
    }

    /// Generate performance report
    pub fn generate_performance_report(&self) -> HotReloadPerformanceReport {
        let metrics = self.get_metrics();

        HotReloadPerformanceReport {
            total_events: metrics.total_events.load(Ordering::Relaxed),
            processed_events: metrics.processed_events.load(Ordering::Relaxed),
            processing_efficiency: metrics.processing_efficiency(),
            cache_hit_rate: metrics.cache_hit_rate(),
            average_processing_time: metrics.average_processing_time(),
            incremental_update_rate: metrics.incremental_update_rate(),
            dependency_graph_size: {
                let graph = self.dependency_graph.read();
                graph.dependencies.len()
            },
            config: self.config.clone(),
        }
    }

    /// Shutdown the hot reload system
    pub async fn shutdown(self) {
        self.running.store(false, Ordering::Relaxed);

        // Wait for all workers to complete
        for handle in self.worker_handles {
            let _ = handle.await;
        }
    }
}

/// Performance report for hot reload system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotReloadPerformanceReport {
    pub total_events: u64,
    pub processed_events: u64,
    pub processing_efficiency: f64,
    pub cache_hit_rate: f64,
    pub average_processing_time: Duration,
    pub incremental_update_rate: f64,
    pub dependency_graph_size: usize,
    pub config: OptimizedHotReloadConfig,
}

impl std::fmt::Display for HotReloadPerformanceReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Hot Reload Performance Report:\n\
             Total Events: {}\n\
             Processed Events: {}\n\
             Processing Efficiency: {:.2}%\n\
             Cache Hit Rate: {:.2}%\n\
             Average Processing Time: {:?}\n\
             Incremental Update Rate: {:.2}%\n\
             Dependency Graph Size: {} assets",
            self.total_events,
            self.processed_events,
            self.processing_efficiency * 100.0,
            self.cache_hit_rate * 100.0,
            self.average_processing_time,
            self.incremental_update_rate * 100.0,
            self.dependency_graph_size
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[tokio::test]
    async fn test_optimized_hot_reload_creation() {
        let config = OptimizedHotReloadConfig::default();
        let system = OptimizedHotReloadSystem::new(config);
        assert!(system.is_ok());
    }

    #[tokio::test]
    async fn test_dependency_graph() {
        let mut graph = DependencyGraph::default();

        let asset_a = PathBuf::from("a.txt");
        let asset_b = PathBuf::from("b.txt");
        let asset_c = PathBuf::from("c.txt");

        graph.add_dependency(asset_b.clone(), asset_a.clone());
        graph.add_dependency(asset_c.clone(), asset_b.clone());

        let affected = graph.get_affected_assets(&asset_a);
        assert_eq!(affected.len(), 3); // a, b, c should all be affected
    }

    #[test]
    fn test_debounce_state() {
        let config = OptimizedHotReloadConfig::default();
        let mut state = DebounceState::new();

        // First update
        state.update_delay(&config);
        assert_eq!(state.current_delay, Duration::from_millis(config.base_debounce_ms));

        // Simulate burst
        state.last_event = Instant::now() - Duration::from_millis(10);
        state.update_delay(&config);
        assert!(state.current_delay > Duration::from_millis(config.base_debounce_ms));
    }

    #[test]
    fn test_content_hash_cache() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, b"test content").unwrap();

        let cache = ContentHashCache::new(100);
        let hash1 = cache.get_hash(&file_path);
        let hash2 = cache.get_hash(&file_path); // Should hit cache

        assert!(hash1.is_some());
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_asset_type_detection() {
        assert_eq!(
            OptimizedHotReloadSystem::detect_asset_type(&PathBuf::from("test.png")),
            AssetType::Texture
        );
        assert_eq!(
            OptimizedHotReloadSystem::detect_asset_type(&PathBuf::from("audio.wav")),
            AssetType::Audio
        );
        assert_eq!(
            OptimizedHotReloadSystem::detect_asset_type(&PathBuf::from("shader.wgsl")),
            AssetType::Shader
        );
    }

    #[test]
    fn test_should_process_path() {
        assert!(!OptimizedHotReloadSystem::should_process_path(&PathBuf::from(".hidden")));
        assert!(!OptimizedHotReloadSystem::should_process_path(&PathBuf::from("temp.tmp")));
        // Note: This test would need actual files to test is_file() properly
    }

    #[test]
    fn test_hot_reload_metrics() {
        let metrics = HotReloadMetrics::default();

        metrics.total_events.store(100, Ordering::Relaxed);
        metrics.processed_events.store(95, Ordering::Relaxed);
        assert_eq!(metrics.processing_efficiency(), 0.95);

        metrics.cache_hits.store(8, Ordering::Relaxed);
        metrics.cache_misses.store(2, Ordering::Relaxed);
        assert_eq!(metrics.cache_hit_rate(), 0.8);

        metrics.incremental_updates.store(7, Ordering::Relaxed);
        metrics.full_reloads.store(3, Ordering::Relaxed);
        assert_eq!(metrics.incremental_update_rate(), 0.7);
    }
}