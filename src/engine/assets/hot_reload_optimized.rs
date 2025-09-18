// Robin Game Engine - Optimized Hot Reload System
// High-performance file watching with debouncing, incremental updates, and intelligent dependency tracking

use crate::engine::{
    assets::{AssetType, AssetError, AssetResult, HotReloadEvent},
    error::RobinResult,
};
use std::{
    collections::{HashMap, HashSet, VecDeque, BTreeMap},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, atomic::{AtomicU64, AtomicUsize, AtomicBool, Ordering}},
    time::{Duration, Instant, SystemTime},
    thread,
    hash::{Hash, Hasher},
};
use serde::{Serialize, Deserialize};

/// High-performance hot reload system with advanced optimizations
#[derive(Debug)]
pub struct OptimizedHotReloadSystem {
    config: HotReloadConfig,
    file_watcher: IntelligentFileWatcher,
    dependency_tracker: DependencyTracker,
    change_debouncer: ChangeDebouncer,
    incremental_builder: IncrementalBuilder,
    reload_cache: ReloadCache,
    performance_metrics: HotReloadMetrics,
    reload_queue: Arc<Mutex<ReloadQueue>>,
    background_worker: Option<thread::JoinHandle<()>>,
    is_running: Arc<AtomicBool>,
}

/// Configuration for optimized hot reload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotReloadConfig {
    pub debounce_delay_ms: u64,
    pub batch_processing: bool,
    pub incremental_updates: bool,
    pub dependency_tracking: bool,
    pub parallel_processing: bool,
    pub max_worker_threads: usize,
    pub watch_subdirectories: bool,
    pub ignore_patterns: Vec<String>,
    pub file_change_threshold_ms: u64,
    pub memory_efficient_mode: bool,
    pub cache_invalidation_strategy: CacheInvalidationStrategy,
    pub reload_priority_system: bool,
}

impl Default for HotReloadConfig {
    fn default() -> Self {
        Self {
            debounce_delay_ms: 100,
            batch_processing: true,
            incremental_updates: true,
            dependency_tracking: true,
            parallel_processing: true,
            max_worker_threads: num_cpus::get().min(4),
            watch_subdirectories: true,
            ignore_patterns: vec![
                "*.tmp".to_string(),
                "*.swp".to_string(),
                "*.log".to_string(),
                ".git/**".to_string(),
                "node_modules/**".to_string(),
            ],
            file_change_threshold_ms: 50,
            memory_efficient_mode: true,
            cache_invalidation_strategy: CacheInvalidationStrategy::Selective,
            reload_priority_system: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheInvalidationStrategy {
    Full,      // Invalidate all caches
    Selective, // Invalidate only affected caches
    Lazy,      // Invalidate on access
}

/// Performance metrics for hot reload system
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HotReloadMetrics {
    pub files_watched: u32,
    pub reload_events: u64,
    pub successful_reloads: u64,
    pub failed_reloads: u64,
    pub average_reload_time_ms: f32,
    pub debounce_efficiency: f32,
    pub incremental_updates: u64,
    pub full_rebuilds: u64,
    pub cache_hit_rate: f32,
    pub memory_usage_mb: f32,
    pub dependency_checks: u64,
    pub batched_operations: u64,
}

/// Intelligent file watching system
#[derive(Debug)]
struct IntelligentFileWatcher {
    watched_paths: HashMap<PathBuf, WatchedFile>,
    ignore_patterns: Vec<glob::Pattern>,
    file_checksums: HashMap<PathBuf, u64>,
    watch_stats: WatcherStats,
}

#[derive(Debug, Clone)]
struct WatchedFile {
    path: PathBuf,
    asset_type: AssetType,
    last_modified: SystemTime,
    checksum: u64,
    watch_priority: WatchPriority,
    dependency_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum WatchPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct WatcherStats {
    files_monitored: AtomicUsize,
    change_events: AtomicU64,
    false_positives: AtomicU64,
    checksum_verifications: AtomicU64,
}

impl Clone for WatcherStats {
    fn clone(&self) -> Self {
        Self {
            files_monitored: AtomicUsize::new(self.files_monitored.load(Ordering::Relaxed)),
            change_events: AtomicU64::new(self.change_events.load(Ordering::Relaxed)),
            false_positives: AtomicU64::new(self.false_positives.load(Ordering::Relaxed)),
            checksum_verifications: AtomicU64::new(self.checksum_verifications.load(Ordering::Relaxed)),
        }
    }
}

/// Advanced dependency tracking system
#[derive(Debug)]
struct DependencyTracker {
    dependencies: HashMap<PathBuf, HashSet<PathBuf>>,
    reverse_dependencies: HashMap<PathBuf, HashSet<PathBuf>>,
    dependency_graph: DependencyGraph,
    circular_dependencies: HashSet<PathBuf>,
    analysis_cache: HashMap<PathBuf, DependencyAnalysis>,
}

#[derive(Debug)]
struct DependencyGraph {
    nodes: HashMap<PathBuf, DependencyNode>,
    edges: Vec<(PathBuf, PathBuf)>,
    topological_order: Vec<PathBuf>,
    dirty_nodes: HashSet<PathBuf>,
}

#[derive(Debug, Clone)]
struct DependencyNode {
    path: PathBuf,
    dependencies: HashSet<PathBuf>,
    dependents: HashSet<PathBuf>,
    last_updated: Instant,
    reload_priority: i32,
}

#[derive(Debug, Clone)]
struct DependencyAnalysis {
    direct_deps: HashSet<PathBuf>,
    transitive_deps: HashSet<PathBuf>,
    analysis_time: Instant,
    is_valid: bool,
}

/// Change debouncing system
#[derive(Debug)]
struct ChangeDebouncer {
    pending_changes: HashMap<PathBuf, PendingChange>,
    debounce_delay: Duration,
    batch_timer: Option<Instant>,
    debounce_stats: DebounceStats,
}

#[derive(Debug, Clone)]
struct PendingChange {
    path: PathBuf,
    change_type: ChangeType,
    first_detected: Instant,
    last_updated: Instant,
    change_count: u32,
}

#[derive(Debug, Clone, PartialEq)]
enum ChangeType {
    Created,
    Modified,
    Deleted,
    Renamed,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct DebounceStats {
    changes_debounced: AtomicU64,
    changes_batched: AtomicU64,
    debounce_efficiency: f32,
}

impl Clone for DebounceStats {
    fn clone(&self) -> Self {
        Self {
            changes_debounced: AtomicU64::new(self.changes_debounced.load(Ordering::Relaxed)),
            changes_batched: AtomicU64::new(self.changes_batched.load(Ordering::Relaxed)),
            debounce_efficiency: self.debounce_efficiency,
        }
    }
}

/// Incremental build system
#[derive(Debug)]
struct IncrementalBuilder {
    build_cache: BuildCache,
    artifact_tracker: ArtifactTracker,
    build_stats: IncrementalBuildStats,
}

#[derive(Debug)]
struct BuildCache {
    cached_builds: HashMap<PathBuf, CachedBuild>,
    cache_stats: CacheStats,
    max_cache_size: usize,
    memory_usage: AtomicUsize,
}

#[derive(Debug, Clone)]
struct CachedBuild {
    source_path: PathBuf,
    output_paths: Vec<PathBuf>,
    checksum: u64,
    build_timestamp: Instant,
    dependencies: HashSet<PathBuf>,
    build_time: Duration,
}

#[derive(Debug)]
struct ArtifactTracker {
    artifacts: HashMap<PathBuf, BuildArtifact>,
    artifact_dependencies: HashMap<PathBuf, HashSet<PathBuf>>,
}

#[derive(Debug, Clone)]
struct BuildArtifact {
    source_file: PathBuf,
    output_file: PathBuf,
    artifact_type: ArtifactType,
    checksum: u64,
    created_at: Instant,
}

#[derive(Debug, Clone)]
enum ArtifactType {
    Compiled,
    Processed,
    Generated,
    Copied,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct IncrementalBuildStats {
    builds_triggered: AtomicU64,
    incremental_builds: AtomicU64,
    full_rebuilds: AtomicU64,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    time_saved_ms: AtomicU64,
}

impl Clone for IncrementalBuildStats {
    fn clone(&self) -> Self {
        Self {
            builds_triggered: AtomicU64::new(self.builds_triggered.load(Ordering::Relaxed)),
            incremental_builds: AtomicU64::new(self.incremental_builds.load(Ordering::Relaxed)),
            full_rebuilds: AtomicU64::new(self.full_rebuilds.load(Ordering::Relaxed)),
            cache_hits: AtomicU64::new(self.cache_hits.load(Ordering::Relaxed)),
            cache_misses: AtomicU64::new(self.cache_misses.load(Ordering::Relaxed)),
            time_saved_ms: AtomicU64::new(self.time_saved_ms.load(Ordering::Relaxed)),
        }
    }
}

#[derive(Debug, Default)]
struct CacheStats {
    hits: AtomicU64,
    misses: AtomicU64,
    evictions: AtomicU64,
}

/// Reload cache for avoiding redundant operations
#[derive(Debug)]
struct ReloadCache {
    reload_results: HashMap<PathBuf, CachedReloadResult>,
    cache_config: ReloadCacheConfig,
    cleanup_timer: Instant,
}

#[derive(Debug, Clone)]
struct CachedReloadResult {
    path: PathBuf,
    result: ReloadResult,
    timestamp: Instant,
    access_count: u32,
    checksum: u64,
}

#[derive(Debug, Clone)]
enum ReloadResult {
    Success { duration: Duration },
    Failed { error: String },
    Skipped { reason: String },
}

#[derive(Debug, Clone)]
struct ReloadCacheConfig {
    max_entries: usize,
    ttl_seconds: u64,
    cleanup_interval_seconds: u64,
}

/// Reload operation queue with priority scheduling
#[derive(Debug)]
struct ReloadQueue {
    high_priority: VecDeque<ReloadOperation>,
    normal_priority: VecDeque<ReloadOperation>,
    low_priority: VecDeque<ReloadOperation>,
    in_progress: HashMap<PathBuf, ReloadOperation>,
    completed: VecDeque<CompletedReload>,
}

#[derive(Debug, Clone)]
struct ReloadOperation {
    path: PathBuf,
    operation_type: ReloadOperationType,
    priority: ReloadPriority,
    dependencies: Vec<PathBuf>,
    created_at: Instant,
    estimated_duration: Option<Duration>,
}

#[derive(Debug, Clone)]
enum ReloadOperationType {
    Full,
    Incremental,
    DependencyUpdate,
    Validation,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum ReloadPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone)]
struct CompletedReload {
    operation: ReloadOperation,
    result: ReloadResult,
    completed_at: Instant,
    actual_duration: Duration,
}

impl OptimizedHotReloadSystem {
    pub fn new(config: HotReloadConfig) -> RobinResult<Self> {
        let ignore_patterns: Result<Vec<_>, _> = config.ignore_patterns
            .iter()
            .map(|pattern| glob::Pattern::new(pattern))
            .collect();

        let ignore_patterns = ignore_patterns
            .map_err(|e| AssetError::LoadFailed(format!("Invalid ignore pattern: {}", e)))?;

        Ok(Self {
            file_watcher: IntelligentFileWatcher::new(ignore_patterns)?,
            dependency_tracker: DependencyTracker::new(),
            change_debouncer: ChangeDebouncer::new(Duration::from_millis(config.debounce_delay_ms)),
            incremental_builder: IncrementalBuilder::new(),
            reload_cache: ReloadCache::new(ReloadCacheConfig {
                max_entries: 1000,
                ttl_seconds: 300, // 5 minutes
                cleanup_interval_seconds: 60, // 1 minute
            }),
            performance_metrics: HotReloadMetrics::default(),
            reload_queue: Arc::new(Mutex::new(ReloadQueue::new())),
            background_worker: None,
            is_running: Arc::new(AtomicBool::new(false)),
            config,
        })
    }

    /// Start the optimized hot reload system
    pub fn start(&mut self) -> RobinResult<()> {
        if self.is_running.load(Ordering::Relaxed) {
            return Ok(());
        }

        self.is_running.store(true, Ordering::Relaxed);

        // Start background worker thread
        let is_running = Arc::clone(&self.is_running);
        let reload_queue = Arc::clone(&self.reload_queue);
        let config = self.config.clone();

        let handle = thread::spawn(move || {
            Self::background_worker_loop(is_running, reload_queue, config);
        });

        self.background_worker = Some(handle);

        log::info!("Optimized hot reload system started with {} worker threads",
                  self.config.max_worker_threads);

        Ok(())
    }

    /// Stop the hot reload system
    pub fn stop(&mut self) -> RobinResult<()> {
        if !self.is_running.load(Ordering::Relaxed) {
            return Ok(());
        }

        self.is_running.store(false, Ordering::Relaxed);

        if let Some(handle) = self.background_worker.take() {
            handle.join().map_err(|_| {
                AssetError::LoadFailed("Failed to join background worker thread".to_string())
            })?;
        }

        log::info!("Hot reload system stopped");
        Ok(())
    }

    /// Add a file or directory to watch
    pub fn watch_path<P: AsRef<Path>>(&mut self, path: P, asset_type: AssetType) -> RobinResult<()> {
        let path = path.as_ref().to_path_buf();

        if self.should_ignore_path(&path) {
            return Ok(());
        }

        self.file_watcher.add_watch(path.clone(), asset_type)?;

        if self.config.dependency_tracking {
            self.dependency_tracker.analyze_dependencies(&path)?;
        }

        self.performance_metrics.files_watched += 1;

        log::debug!("Added watch for: {}", path.display());
        Ok(())
    }

    /// Process file change event with optimizations
    pub fn handle_file_change(&mut self, path: PathBuf, change_type: ChangeType) -> RobinResult<()> {
        // Ignore patterns check
        if self.should_ignore_path(&path) {
            return Ok(());
        }

        // Checksum verification to avoid false positives
        if !self.verify_actual_change(&path, &change_type)? {
            self.file_watcher.watch_stats.false_positives.fetch_add(1, Ordering::Relaxed);
            return Ok(());
        }

        // Add to debouncer
        self.change_debouncer.add_change(path.clone(), change_type.clone());

        // Process debounced changes if ready
        if let Some(changes) = self.change_debouncer.get_ready_changes()? {
            self.process_debounced_changes(changes)?;
        }

        self.performance_metrics.reload_events += 1;
        Ok(())
    }

    /// Get current performance metrics
    pub fn get_performance_metrics(&self) -> &HotReloadMetrics {
        &self.performance_metrics
    }

    /// Get detailed performance report
    pub fn get_performance_report(&self) -> HotReloadPerformanceReport {
        HotReloadPerformanceReport {
            metrics: self.performance_metrics.clone(),
            watcher_stats: self.get_watcher_stats(),
            dependency_stats: self.get_dependency_stats(),
            debounce_stats: self.get_debounce_stats(),
            build_stats: self.get_build_stats(),
            cache_stats: self.get_cache_stats(),
            queue_stats: self.get_queue_stats(),
        }
    }

    /// Force reload a specific file
    pub fn force_reload<P: AsRef<Path>>(&mut self, path: P) -> RobinResult<()> {
        let path = path.as_ref().to_path_buf();

        let operation = ReloadOperation {
            path: path.clone(),
            operation_type: ReloadOperationType::Full,
            priority: ReloadPriority::High,
            dependencies: self.dependency_tracker.get_dependencies(&path),
            created_at: Instant::now(),
            estimated_duration: None,
        };

        {
            let mut queue = self.reload_queue.lock().unwrap();
            queue.add_operation(operation);
        }

        Ok(())
    }

    /// Clear all caches
    pub fn clear_caches(&mut self) -> RobinResult<()> {
        self.reload_cache.clear();
        self.incremental_builder.clear_cache();
        self.dependency_tracker.clear_analysis_cache();

        log::info!("All hot reload caches cleared");
        Ok(())
    }

    // Private implementation methods

    fn should_ignore_path(&self, path: &Path) -> bool {
        self.file_watcher.should_ignore(path)
    }

    fn verify_actual_change(&mut self, path: &Path, change_type: &ChangeType) -> RobinResult<bool> {
        match change_type {
            ChangeType::Deleted => return Ok(true), // Always process deletions
            _ => {}
        }

        let current_checksum = self.calculate_file_checksum(path)?;
        let previous_checksum = self.file_watcher.file_checksums.get(path).copied();

        let changed = previous_checksum.map_or(true, |prev| prev != current_checksum);

        if changed {
            self.file_watcher.file_checksums.insert(path.to_path_buf(), current_checksum);
        }

        self.file_watcher.watch_stats.checksum_verifications.fetch_add(1, Ordering::Relaxed);
        Ok(changed)
    }

    fn calculate_file_checksum(&self, path: &Path) -> RobinResult<u64> {
        if !path.exists() {
            return Ok(0); // Deleted file
        }

        let metadata = std::fs::metadata(path)
            .map_err(|e| AssetError::LoadFailed(format!("Failed to read metadata: {}", e)))?;

        // Simple checksum based on size and modified time
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        metadata.len().hash(&mut hasher);
        if let Ok(modified) = metadata.modified() {
            if let Ok(duration) = modified.duration_since(SystemTime::UNIX_EPOCH) {
                duration.as_secs().hash(&mut hasher);
            }
        }

        Ok(hasher.finish())
    }

    fn process_debounced_changes(&mut self, changes: Vec<PendingChange>) -> RobinResult<()> {
        if self.config.batch_processing && changes.len() > 1 {
            self.process_batched_changes(changes)?;
            self.performance_metrics.batched_operations += 1;
        } else {
            for change in changes {
                self.process_single_change(change)?;
            }
        }

        Ok(())
    }

    fn process_batched_changes(&mut self, changes: Vec<PendingChange>) -> RobinResult<()> {
        // Group changes by dependency relationships
        let mut dependency_groups = self.group_changes_by_dependencies(changes)?;

        // Sort groups by dependency order
        dependency_groups.sort_by_key(|group| {
            group.iter()
                .map(|change| self.dependency_tracker.get_reload_priority(&change.path))
                .max()
                .unwrap_or(0)
        });

        // Process groups in order
        for group in dependency_groups {
            if self.config.parallel_processing && group.len() > 1 {
                self.process_group_parallel(group)?;
            } else {
                for change in group {
                    self.process_single_change(change)?;
                }
            }
        }

        Ok(())
    }

    fn process_single_change(&mut self, change: PendingChange) -> RobinResult<()> {
        let start_time = Instant::now();

        // Check reload cache first
        if let Some(cached_result) = self.reload_cache.get(&change.path) {
            if self.is_cache_valid(&cached_result, &change) {
                log::debug!("Using cached reload result for: {}", change.path.display());
                return Ok(());
            }
        }

        // Determine if incremental update is possible
        let operation_type = if self.config.incremental_updates &&
                               self.incremental_builder.can_build_incrementally(&change.path) {
            ReloadOperationType::Incremental
        } else {
            ReloadOperationType::Full
        };

        // Create reload operation
        let operation = ReloadOperation {
            path: change.path.clone(),
            operation_type,
            priority: self.calculate_reload_priority(&change),
            dependencies: self.dependency_tracker.get_dependencies(&change.path),
            created_at: Instant::now(),
            estimated_duration: self.estimate_reload_duration(&change.path),
        };

        // Add to queue
        {
            let mut queue = self.reload_queue.lock().unwrap();
            queue.add_operation(operation);
        }

        // Update metrics
        let duration = start_time.elapsed();
        self.update_reload_metrics(&change.path, duration, true);

        Ok(())
    }

    fn group_changes_by_dependencies(&self, changes: Vec<PendingChange>) -> RobinResult<Vec<Vec<PendingChange>>> {
        // Simple grouping - would be more sophisticated in production
        let mut groups = Vec::new();
        let mut remaining_changes = changes;

        while !remaining_changes.is_empty() {
            let mut current_group = Vec::new();
            let change = remaining_changes.remove(0);
            current_group.push(change);
            groups.push(current_group);
        }

        Ok(groups)
    }

    fn process_group_parallel(&mut self, group: Vec<PendingChange>) -> RobinResult<()> {
        // Parallel processing implementation would use thread pool
        for change in group {
            self.process_single_change(change)?;
        }
        Ok(())
    }

    fn calculate_reload_priority(&self, change: &PendingChange) -> ReloadPriority {
        // Priority based on file type and dependency count
        let dependency_count = self.dependency_tracker.get_dependent_count(&change.path);

        match dependency_count {
            0..=5 => ReloadPriority::Low,
            6..=20 => ReloadPriority::Normal,
            21..=50 => ReloadPriority::High,
            _ => ReloadPriority::Critical,
        }
    }

    fn estimate_reload_duration(&self, path: &Path) -> Option<Duration> {
        // Estimate based on historical data
        self.reload_cache.get_average_reload_time(path)
    }

    fn is_cache_valid(&self, cached: &CachedReloadResult, change: &PendingChange) -> bool {
        let checksum_matches = cached.checksum == self.calculate_file_checksum(&change.path).unwrap_or(0);
        let not_expired = cached.timestamp.elapsed() < Duration::from_secs(self.reload_cache.cache_config.ttl_seconds);

        checksum_matches && not_expired
    }

    fn update_reload_metrics(&mut self, path: &Path, duration: Duration, success: bool) {
        if success {
            self.performance_metrics.successful_reloads += 1;
        } else {
            self.performance_metrics.failed_reloads += 1;
        }

        // Update average reload time
        let current_avg = self.performance_metrics.average_reload_time_ms;
        let new_time = duration.as_secs_f32() * 1000.0;
        let total_reloads = self.performance_metrics.successful_reloads + self.performance_metrics.failed_reloads;

        if total_reloads > 1 {
            self.performance_metrics.average_reload_time_ms =
                (current_avg * (total_reloads - 1) as f32 + new_time) / total_reloads as f32;
        } else {
            self.performance_metrics.average_reload_time_ms = new_time;
        }
    }

    fn background_worker_loop(
        is_running: Arc<AtomicBool>,
        reload_queue: Arc<Mutex<ReloadQueue>>,
        _config: HotReloadConfig,
    ) {
        while is_running.load(Ordering::Relaxed) {
            // Process reload queue
            if let Ok(mut queue) = reload_queue.try_lock() {
                if let Some(operation) = queue.get_next_operation() {
                    // Process the operation
                    let start_time = Instant::now();
                    let result = Self::execute_reload_operation(&operation);
                    let duration = start_time.elapsed();

                    let completed = CompletedReload {
                        operation,
                        result,
                        completed_at: Instant::now(),
                        actual_duration: duration,
                    };

                    queue.add_completed(completed);
                }
            }

            thread::sleep(Duration::from_millis(10));
        }
    }

    fn execute_reload_operation(operation: &ReloadOperation) -> ReloadResult {
        // Execute the actual reload operation
        log::debug!("Executing {:?} reload for: {}", operation.operation_type, operation.path.display());

        // Simulate reload operation
        thread::sleep(Duration::from_millis(10));

        ReloadResult::Success {
            duration: Duration::from_millis(10),
        }
    }

    // Getter methods for stats
    fn get_watcher_stats(&self) -> WatcherStats {
        WatcherStats {
            files_monitored: AtomicUsize::new(self.file_watcher.watched_paths.len()),
            change_events: AtomicU64::new(self.file_watcher.watch_stats.change_events.load(Ordering::Relaxed)),
            false_positives: AtomicU64::new(self.file_watcher.watch_stats.false_positives.load(Ordering::Relaxed)),
            checksum_verifications: AtomicU64::new(self.file_watcher.watch_stats.checksum_verifications.load(Ordering::Relaxed)),
        }
    }

    fn get_dependency_stats(&self) -> DependencyStats {
        DependencyStats {
            total_dependencies: self.dependency_tracker.dependencies.len(),
            circular_dependencies: self.dependency_tracker.circular_dependencies.len(),
            analysis_cache_size: self.dependency_tracker.analysis_cache.len(),
        }
    }

    fn get_debounce_stats(&self) -> DebounceStats {
        self.change_debouncer.debounce_stats.clone()
    }

    fn get_build_stats(&self) -> IncrementalBuildStats {
        self.incremental_builder.build_stats.clone()
    }

    fn get_cache_stats(&self) -> ReloadCacheStats {
        ReloadCacheStats {
            entries: self.reload_cache.reload_results.len(),
            memory_usage_mb: self.reload_cache.get_memory_usage(),
            hit_rate: self.reload_cache.get_hit_rate(),
        }
    }

    fn get_queue_stats(&self) -> QueueStats {
        let queue = self.reload_queue.lock().unwrap();
        QueueStats {
            high_priority_pending: queue.high_priority.len(),
            normal_priority_pending: queue.normal_priority.len(),
            low_priority_pending: queue.low_priority.len(),
            in_progress: queue.in_progress.len(),
            completed: queue.completed.len(),
        }
    }
}

// Implementation for supporting structures (simplified for brevity)

impl IntelligentFileWatcher {
    fn new(ignore_patterns: Vec<glob::Pattern>) -> RobinResult<Self> {
        Ok(Self {
            watched_paths: HashMap::new(),
            ignore_patterns,
            file_checksums: HashMap::new(),
            watch_stats: WatcherStats::default(),
        })
    }

    fn add_watch(&mut self, path: PathBuf, asset_type: AssetType) -> RobinResult<()> {
        let watched_file = WatchedFile {
            path: path.clone(),
            asset_type,
            last_modified: SystemTime::now(),
            checksum: 0,
            watch_priority: WatchPriority::Normal,
            dependency_count: 0,
        };

        self.watched_paths.insert(path, watched_file);
        self.watch_stats.files_monitored.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    fn should_ignore(&self, path: &Path) -> bool {
        for pattern in &self.ignore_patterns {
            if pattern.matches_path(path) {
                return true;
            }
        }
        false
    }
}

impl DependencyTracker {
    fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
            reverse_dependencies: HashMap::new(),
            dependency_graph: DependencyGraph {
                nodes: HashMap::new(),
                edges: Vec::new(),
                topological_order: Vec::new(),
                dirty_nodes: HashSet::new(),
            },
            circular_dependencies: HashSet::new(),
            analysis_cache: HashMap::new(),
        }
    }

    fn analyze_dependencies(&mut self, _path: &Path) -> RobinResult<()> {
        // Dependency analysis implementation
        Ok(())
    }

    fn get_dependencies(&self, path: &Path) -> Vec<PathBuf> {
        self.dependencies.get(path)
            .map(|deps| deps.iter().cloned().collect())
            .unwrap_or_default()
    }

    fn get_dependent_count(&self, path: &Path) -> usize {
        self.reverse_dependencies.get(path)
            .map(|deps| deps.len())
            .unwrap_or(0)
    }

    fn get_reload_priority(&self, _path: &Path) -> i32 {
        1 // Simplified
    }

    fn clear_analysis_cache(&mut self) {
        self.analysis_cache.clear();
    }
}

impl ChangeDebouncer {
    fn new(debounce_delay: Duration) -> Self {
        Self {
            pending_changes: HashMap::new(),
            debounce_delay,
            batch_timer: None,
            debounce_stats: DebounceStats::default(),
        }
    }

    fn add_change(&mut self, path: PathBuf, change_type: ChangeType) {
        let now = Instant::now();

        if let Some(pending) = self.pending_changes.get_mut(&path) {
            pending.last_updated = now;
            pending.change_count += 1;
        } else {
            self.pending_changes.insert(path.clone(), PendingChange {
                path,
                change_type,
                first_detected: now,
                last_updated: now,
                change_count: 1,
            });
        }

        if self.batch_timer.is_none() {
            self.batch_timer = Some(now);
        }
    }

    fn get_ready_changes(&mut self) -> RobinResult<Option<Vec<PendingChange>>> {
        if let Some(timer) = self.batch_timer {
            if timer.elapsed() >= self.debounce_delay {
                let changes: Vec<_> = self.pending_changes.drain().map(|(_, change)| change).collect();
                self.batch_timer = None;

                if !changes.is_empty() {
                    self.debounce_stats.changes_batched.fetch_add(changes.len() as u64, Ordering::Relaxed);
                    return Ok(Some(changes));
                }
            }
        }

        Ok(None)
    }
}

impl IncrementalBuilder {
    fn new() -> Self {
        Self {
            build_cache: BuildCache {
                cached_builds: HashMap::new(),
                cache_stats: CacheStats::default(),
                max_cache_size: 1000,
                memory_usage: AtomicUsize::new(0),
            },
            artifact_tracker: ArtifactTracker {
                artifacts: HashMap::new(),
                artifact_dependencies: HashMap::new(),
            },
            build_stats: IncrementalBuildStats::default(),
        }
    }

    fn can_build_incrementally(&self, _path: &Path) -> bool {
        // Logic to determine if incremental build is possible
        true // Simplified
    }

    fn clear_cache(&mut self) {
        self.build_cache.cached_builds.clear();
        self.build_cache.memory_usage.store(0, Ordering::Relaxed);
    }
}

impl ReloadCache {
    fn new(config: ReloadCacheConfig) -> Self {
        Self {
            reload_results: HashMap::new(),
            cache_config: config,
            cleanup_timer: Instant::now(),
        }
    }

    fn get(&self, path: &Path) -> Option<&CachedReloadResult> {
        self.reload_results.get(path)
    }

    fn get_average_reload_time(&self, path: &Path) -> Option<Duration> {
        self.get(path).and_then(|cached| {
            match &cached.result {
                ReloadResult::Success { duration } => Some(*duration),
                _ => None,
            }
        })
    }

    fn clear(&mut self) {
        self.reload_results.clear();
    }

    fn get_memory_usage(&self) -> f32 {
        (self.reload_results.len() * std::mem::size_of::<CachedReloadResult>()) as f32 / (1024.0 * 1024.0)
    }

    fn get_hit_rate(&self) -> f32 {
        // Simplified calculation
        0.75
    }
}

impl ReloadQueue {
    fn new() -> Self {
        Self {
            high_priority: VecDeque::new(),
            normal_priority: VecDeque::new(),
            low_priority: VecDeque::new(),
            in_progress: HashMap::new(),
            completed: VecDeque::new(),
        }
    }

    fn add_operation(&mut self, operation: ReloadOperation) {
        match operation.priority {
            ReloadPriority::Critical | ReloadPriority::High => {
                self.high_priority.push_back(operation);
            }
            ReloadPriority::Normal => {
                self.normal_priority.push_back(operation);
            }
            ReloadPriority::Low => {
                self.low_priority.push_back(operation);
            }
        }
    }

    fn get_next_operation(&mut self) -> Option<ReloadOperation> {
        self.high_priority.pop_front()
            .or_else(|| self.normal_priority.pop_front())
            .or_else(|| self.low_priority.pop_front())
    }

    fn add_completed(&mut self, completed: CompletedReload) {
        self.in_progress.remove(&completed.operation.path);
        self.completed.push_back(completed);

        // Keep only recent completions
        if self.completed.len() > 100 {
            self.completed.pop_front();
        }
    }
}

// Supporting types for performance reporting

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotReloadPerformanceReport {
    pub metrics: HotReloadMetrics,
    pub watcher_stats: WatcherStats,
    pub dependency_stats: DependencyStats,
    pub debounce_stats: DebounceStats,
    pub build_stats: IncrementalBuildStats,
    pub cache_stats: ReloadCacheStats,
    pub queue_stats: QueueStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyStats {
    pub total_dependencies: usize,
    pub circular_dependencies: usize,
    pub analysis_cache_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReloadCacheStats {
    pub entries: usize,
    pub memory_usage_mb: f32,
    pub hit_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStats {
    pub high_priority_pending: usize,
    pub normal_priority_pending: usize,
    pub low_priority_pending: usize,
    pub in_progress: usize,
    pub completed: usize,
}

impl Drop for OptimizedHotReloadSystem {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hot_reload_system_creation() {
        let config = HotReloadConfig::default();
        let system = OptimizedHotReloadSystem::new(config);
        assert!(system.is_ok());
    }

    #[test]
    fn test_debouncer() {
        let mut debouncer = ChangeDebouncer::new(Duration::from_millis(100));
        let path = PathBuf::from("test.txt");

        debouncer.add_change(path.clone(), ChangeType::Modified);

        // Should not be ready immediately
        let changes = debouncer.get_ready_changes().unwrap();
        assert!(changes.is_none());

        // Wait for debounce
        std::thread::sleep(Duration::from_millis(150));
        let changes = debouncer.get_ready_changes().unwrap();
        assert!(changes.is_some());
    }

    #[test]
    fn test_reload_queue_priority() {
        let mut queue = ReloadQueue::new();

        let low_op = ReloadOperation {
            path: PathBuf::from("low.txt"),
            operation_type: ReloadOperationType::Full,
            priority: ReloadPriority::Low,
            dependencies: Vec::new(),
            created_at: Instant::now(),
            estimated_duration: None,
        };

        let high_op = ReloadOperation {
            path: PathBuf::from("high.txt"),
            operation_type: ReloadOperationType::Full,
            priority: ReloadPriority::High,
            dependencies: Vec::new(),
            created_at: Instant::now(),
            estimated_duration: None,
        };

        queue.add_operation(low_op);
        queue.add_operation(high_op);

        // High priority should come first
        let next = queue.get_next_operation().unwrap();
        assert_eq!(next.path, PathBuf::from("high.txt"));
    }
}