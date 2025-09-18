/// High-performance parallel asset processing system
///
/// This module provides optimized asset processing using:
/// - Rayon thread pools for parallel execution
/// - Memory-mapped I/O for large files
/// - Streaming for massive assets
/// - Cache-friendly data structures
/// - Smart memory management

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicUsize, AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use ahash::AHashMap;
use crossbeam_channel::{Receiver, Sender, unbounded};
use lru::LruCache;
use memmap2::Mmap;
use parking_lot::{RwLock, Mutex};
use rayon::prelude::*;
use smallvec::SmallVec;
use serde::{Deserialize, Serialize};

use super::{AssetType, AssetError, AssetResult, AssetMetadata};

/// Configuration for parallel asset processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelProcessorConfig {
    /// Maximum number of worker threads (0 = auto-detect)
    pub max_threads: usize,
    /// Size of the task queue
    pub queue_size: usize,
    /// Memory cache size in bytes
    pub cache_size: usize,
    /// Enable memory-mapped I/O for files larger than this threshold
    pub mmap_threshold: usize,
    /// Batch size for bulk operations
    pub batch_size: usize,
    /// Enable streaming for files larger than this threshold
    pub streaming_threshold: usize,
    /// Compression level for cached assets (0-9)
    pub compression_level: u32,
}

impl Default for ParallelProcessorConfig {
    fn default() -> Self {
        Self {
            max_threads: 0, // Auto-detect
            queue_size: 1024,
            cache_size: 256 * 1024 * 1024, // 256MB
            mmap_threshold: 4 * 1024 * 1024, // 4MB
            batch_size: 32,
            streaming_threshold: 64 * 1024 * 1024, // 64MB
            compression_level: 6,
        }
    }
}

/// Asset processing task
#[derive(Debug, Clone)]
pub struct ProcessingTask {
    pub id: String,
    pub asset_type: AssetType,
    pub file_path: PathBuf,
    pub priority: TaskPriority,
    pub metadata: AssetMetadata,
}

/// Task priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Processing result for assets
#[derive(Debug, Clone)]
pub struct ProcessingResult {
    pub task_id: String,
    pub success: bool,
    pub data: Option<Vec<u8>>,
    pub processing_time: Duration,
    pub memory_usage: usize,
    pub error: Option<String>,
}

/// Memory-mapped file wrapper
pub struct MappedFile {
    _file: std::fs::File,
    mmap: Mmap,
    size: usize,
}

impl MappedFile {
    pub fn new<P: AsRef<Path>>(path: P) -> AssetResult<Self> {
        let file = std::fs::File::open(&path)
            .map_err(|_| AssetError::FileNotFound(path.as_ref().to_path_buf()))?;

        let mmap = unsafe {
            Mmap::map(&file)
                .map_err(|e| AssetError::LoadFailed(format!("Memory mapping failed: {}", e)))?
        };

        let size = mmap.len();

        Ok(Self {
            _file: file,
            mmap,
            size,
        })
    }

    pub fn data(&self) -> &[u8] {
        &self.mmap
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

/// Cache entry for processed assets
#[derive(Debug)]
struct CacheEntry {
    data: Vec<u8>,
    last_accessed: Instant,
    access_count: AtomicUsize,
    compressed: bool,
}

impl CacheEntry {
    fn new(data: Vec<u8>, compressed: bool) -> Self {
        Self {
            data,
            last_accessed: Instant::now(),
            access_count: AtomicUsize::new(1),
            compressed,
        }
    }

    fn access(&self) -> &[u8] {
        self.access_count.fetch_add(1, Ordering::Relaxed);
        &self.data
    }
}

impl Clone for CacheEntry {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            last_accessed: self.last_accessed,
            access_count: AtomicUsize::new(self.access_count.load(Ordering::Relaxed)),
            compressed: self.compressed,
        }
    }
}

/// Statistics for performance monitoring
#[derive(Debug, Default)]
pub struct ProcessorStats {
    pub tasks_processed: AtomicUsize,
    pub tasks_failed: AtomicUsize,
    pub cache_hits: AtomicUsize,
    pub cache_misses: AtomicUsize,
    pub total_processing_time: AtomicUsize, // in microseconds
    pub memory_mapped_files: AtomicUsize,
    pub streamed_files: AtomicUsize,
    pub compressed_entries: AtomicUsize,
}

impl ProcessorStats {
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

    pub fn success_rate(&self) -> f64 {
        let processed = self.tasks_processed.load(Ordering::Relaxed);
        let failed = self.tasks_failed.load(Ordering::Relaxed);
        let total = processed + failed;

        if total == 0 {
            0.0
        } else {
            processed as f64 / total as f64
        }
    }

    pub fn average_processing_time(&self) -> Duration {
        let total_time = self.total_processing_time.load(Ordering::Relaxed);
        let processed = self.tasks_processed.load(Ordering::Relaxed);

        if processed == 0 {
            Duration::ZERO
        } else {
            Duration::from_micros((total_time / processed) as u64)
        }
    }
}

impl Clone for ProcessorStats {
    fn clone(&self) -> Self {
        Self {
            tasks_processed: AtomicUsize::new(self.tasks_processed.load(Ordering::Relaxed)),
            tasks_failed: AtomicUsize::new(self.tasks_failed.load(Ordering::Relaxed)),
            cache_hits: AtomicUsize::new(self.cache_hits.load(Ordering::Relaxed)),
            cache_misses: AtomicUsize::new(self.cache_misses.load(Ordering::Relaxed)),
            total_processing_time: AtomicUsize::new(self.total_processing_time.load(Ordering::Relaxed)),
            memory_mapped_files: AtomicUsize::new(self.memory_mapped_files.load(Ordering::Relaxed)),
            streamed_files: AtomicUsize::new(self.streamed_files.load(Ordering::Relaxed)),
            compressed_entries: AtomicUsize::new(self.compressed_entries.load(Ordering::Relaxed)),
        }
    }
}

/// High-performance parallel asset processor
pub struct ParallelAssetProcessor {
    config: ParallelProcessorConfig,
    task_sender: Sender<ProcessingTask>,
    result_receiver: Receiver<ProcessingResult>,
    cache: Arc<Mutex<LruCache<String, CacheEntry>>>,
    stats: Arc<ProcessorStats>,
    thread_pool: rayon::ThreadPool,
    running: Arc<AtomicBool>,
}

impl ParallelAssetProcessor {
    pub fn new(config: ParallelProcessorConfig) -> AssetResult<Self> {
        let num_threads = if config.max_threads == 0 {
            num_cpus::get()
        } else {
            config.max_threads
        };

        let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .thread_name(|index| format!("asset-processor-{}", index))
            .build()
            .map_err(|e| AssetError::LoadFailed(format!("Failed to create thread pool: {}", e)))?;

        let (task_sender, task_receiver) = unbounded::<ProcessingTask>();
        let (result_sender, result_receiver) = unbounded::<ProcessingResult>();

        let cache_entries = config.cache_size / 1024; // Estimate entries
        let cache = Arc::new(Mutex::new(LruCache::new(
            std::num::NonZeroUsize::new(cache_entries).unwrap_or(std::num::NonZeroUsize::new(1024).unwrap())
        )));

        let stats = Arc::new(ProcessorStats::default());
        let running = Arc::new(AtomicBool::new(true));

        // Spawn worker tasks
        let cache_clone = Arc::clone(&cache);
        let stats_clone = Arc::clone(&stats);
        let config_clone = config.clone();
        let running_clone = Arc::clone(&running);

        thread_pool.spawn(move || {
            Self::worker_loop(
                task_receiver,
                result_sender,
                cache_clone,
                stats_clone,
                config_clone,
                running_clone,
            );
        });

        Ok(Self {
            config,
            task_sender,
            result_receiver,
            cache,
            stats,
            thread_pool,
            running,
        })
    }

    /// Submit a task for processing
    pub fn submit_task(&self, task: ProcessingTask) -> AssetResult<()> {
        self.task_sender.send(task)
            .map_err(|e| AssetError::LoadFailed(format!("Failed to submit task: {}", e)))
    }

    /// Submit multiple tasks in batch for efficient processing
    pub fn submit_batch(&self, tasks: Vec<ProcessingTask>) -> AssetResult<()> {
        for task in tasks {
            self.submit_task(task)?;
        }
        Ok(())
    }

    /// Get next processing result (non-blocking)
    pub fn try_get_result(&self) -> Option<ProcessingResult> {
        self.result_receiver.try_recv().ok()
    }

    /// Get next processing result (blocking with timeout)
    pub fn get_result_timeout(&self, timeout: Duration) -> Option<ProcessingResult> {
        self.result_receiver.recv_timeout(timeout).ok()
    }

    /// Get cached asset data
    pub fn get_cached(&self, asset_id: &str) -> Option<Vec<u8>> {
        let mut cache = self.cache.lock();

        if let Some(entry) = cache.get(asset_id) {
            self.stats.cache_hits.fetch_add(1, Ordering::Relaxed);
            Some(entry.access().to_vec())
        } else {
            self.stats.cache_misses.fetch_add(1, Ordering::Relaxed);
            None
        }
    }

    /// Process multiple assets in parallel with optimal batching
    pub fn process_batch_parallel<P: AsRef<Path>>(
        &self,
        asset_paths: &[P],
        asset_type: AssetType,
    ) -> AssetResult<Vec<ProcessingResult>> {
        let tasks: Vec<ProcessingTask> = asset_paths
            .iter()
            .enumerate()
            .map(|(i, path)| {
                let file_path = path.as_ref().to_path_buf();
                let metadata = AssetMetadata::new(
                    format!("batch_asset_{}", i),
                    asset_type.clone(),
                    file_path.clone(),
                );

                ProcessingTask {
                    id: format!("batch_asset_{}", i),
                    asset_type: asset_type.clone(),
                    file_path,
                    priority: TaskPriority::Normal,
                    metadata,
                }
            })
            .collect();

        // Submit all tasks
        self.submit_batch(tasks)?;

        // Collect results
        let mut results = Vec::new();
        for _ in 0..asset_paths.len() {
            if let Some(result) = self.get_result_timeout(Duration::from_secs(30)) {
                results.push(result);
            }
        }

        Ok(results)
    }

    /// Get current performance statistics
    pub fn get_stats(&self) -> ProcessorStats {
        ProcessorStats {
            tasks_processed: AtomicUsize::new(self.stats.tasks_processed.load(Ordering::Relaxed)),
            tasks_failed: AtomicUsize::new(self.stats.tasks_failed.load(Ordering::Relaxed)),
            cache_hits: AtomicUsize::new(self.stats.cache_hits.load(Ordering::Relaxed)),
            cache_misses: AtomicUsize::new(self.stats.cache_misses.load(Ordering::Relaxed)),
            total_processing_time: AtomicUsize::new(self.stats.total_processing_time.load(Ordering::Relaxed)),
            memory_mapped_files: AtomicUsize::new(self.stats.memory_mapped_files.load(Ordering::Relaxed)),
            streamed_files: AtomicUsize::new(self.stats.streamed_files.load(Ordering::Relaxed)),
            compressed_entries: AtomicUsize::new(self.stats.compressed_entries.load(Ordering::Relaxed)),
        }
    }

    /// Worker loop for processing tasks
    fn worker_loop(
        task_receiver: Receiver<ProcessingTask>,
        result_sender: Sender<ProcessingResult>,
        cache: Arc<Mutex<LruCache<String, CacheEntry>>>,
        stats: Arc<ProcessorStats>,
        config: ParallelProcessorConfig,
        running: Arc<AtomicBool>,
    ) {
        // Create a priority queue for tasks
        let mut task_queue: SmallVec<[ProcessingTask; 32]> = SmallVec::new();

        while running.load(Ordering::Relaxed) {
            // Collect tasks in batches for better cache locality
            task_queue.clear();

            // Get initial task (blocking)
            if let Ok(task) = task_receiver.recv_timeout(Duration::from_millis(100)) {
                task_queue.push(task);

                // Try to get more tasks (non-blocking)
                while task_queue.len() < config.batch_size {
                    if let Ok(task) = task_receiver.try_recv() {
                        task_queue.push(task);
                    } else {
                        break;
                    }
                }

                // Sort by priority for optimal processing order
                task_queue.sort_by_key(|task| std::cmp::Reverse(task.priority));

                // Process tasks in parallel
                let results: Vec<ProcessingResult> = task_queue
                    .par_iter()
                    .map(|task| Self::process_single_task(task, &config, &stats))
                    .collect();

                // Cache results and send them
                for result in results {
                    if result.success {
                        if let Some(ref data) = result.data {
                            let compressed = data.len() > 1024; // Compress larger assets
                            let cache_data = if compressed {
                                Self::compress_data(data, config.compression_level)
                                    .unwrap_or_else(|| data.clone())
                            } else {
                                data.clone()
                            };

                            let entry = CacheEntry::new(cache_data, compressed);
                            cache.lock().put(result.task_id.clone(), entry);

                            if compressed {
                                stats.compressed_entries.fetch_add(1, Ordering::Relaxed);
                            }
                        }

                        stats.tasks_processed.fetch_add(1, Ordering::Relaxed);
                    } else {
                        stats.tasks_failed.fetch_add(1, Ordering::Relaxed);
                    }

                    stats.total_processing_time.fetch_add(
                        result.processing_time.as_micros() as usize,
                        Ordering::Relaxed,
                    );

                    let _ = result_sender.send(result);
                }
            }
        }
    }

    /// Process a single task with optimal I/O strategy
    fn process_single_task(
        task: &ProcessingTask,
        config: &ParallelProcessorConfig,
        stats: &ProcessorStats,
    ) -> ProcessingResult {
        let start_time = Instant::now();

        let result = match std::fs::metadata(&task.file_path) {
            Ok(metadata) => {
                let file_size = metadata.len() as usize;

                if file_size > config.streaming_threshold {
                    // Use streaming for very large files
                    stats.streamed_files.fetch_add(1, Ordering::Relaxed);
                    Self::process_with_streaming(task, config)
                } else if file_size > config.mmap_threshold {
                    // Use memory mapping for large files
                    stats.memory_mapped_files.fetch_add(1, Ordering::Relaxed);
                    Self::process_with_mmap(task)
                } else {
                    // Use regular file I/O for small files
                    Self::process_with_regular_io(task)
                }
            }
            Err(e) => Err(AssetError::FileNotFound(task.file_path.clone())),
        };

        let processing_time = start_time.elapsed();

        match result {
            Ok(data) => ProcessingResult {
                task_id: task.id.clone(),
                success: true,
                data: Some(data),
                processing_time,
                memory_usage: 0, // TODO: Track actual memory usage
                error: None,
            },
            Err(e) => ProcessingResult {
                task_id: task.id.clone(),
                success: false,
                data: None,
                processing_time,
                memory_usage: 0,
                error: Some(e.to_string()),
            },
        }
    }

    /// Process asset using memory-mapped I/O
    fn process_with_mmap(task: &ProcessingTask) -> AssetResult<Vec<u8>> {
        let mapped_file = MappedFile::new(&task.file_path)?;
        let data = mapped_file.data();

        // Perform asset-specific processing
        match task.asset_type {
            AssetType::Texture => Self::process_texture_data(data),
            AssetType::Audio => Self::process_audio_data(data),
            AssetType::Config => Self::process_config_data(data),
            _ => Ok(data.to_vec()),
        }
    }

    /// Process asset using streaming I/O for massive files
    fn process_with_streaming(
        task: &ProcessingTask,
        config: &ParallelProcessorConfig,
    ) -> AssetResult<Vec<u8>> {
        use std::io::{BufReader, Read};

        let file = std::fs::File::open(&task.file_path)
            .map_err(|_| AssetError::FileNotFound(task.file_path.clone()))?;

        let mut reader = BufReader::with_capacity(64 * 1024, file); // 64KB buffer
        let mut chunks = Vec::new();
        let mut buffer = vec![0u8; 64 * 1024];

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    chunks.push(buffer[..n].to_vec());
                }
                Err(e) => {
                    return Err(AssetError::LoadFailed(format!("Streaming read failed: {}", e)));
                }
            }
        }

        // Concatenate chunks efficiently
        let total_size: usize = chunks.iter().map(|chunk| chunk.len()).sum();
        let mut result = Vec::with_capacity(total_size);

        for chunk in chunks {
            result.extend_from_slice(&chunk);
        }

        // Process the complete data
        match task.asset_type {
            AssetType::Texture => Self::process_texture_data(&result),
            AssetType::Audio => Self::process_audio_data(&result),
            AssetType::Config => Self::process_config_data(&result),
            _ => Ok(result),
        }
    }

    /// Process asset using regular file I/O
    fn process_with_regular_io(task: &ProcessingTask) -> AssetResult<Vec<u8>> {
        let data = std::fs::read(&task.file_path)
            .map_err(|_| AssetError::FileNotFound(task.file_path.clone()))?;

        match task.asset_type {
            AssetType::Texture => Self::process_texture_data(&data),
            AssetType::Audio => Self::process_audio_data(&data),
            AssetType::Config => Self::process_config_data(&data),
            _ => Ok(data),
        }
    }

    /// Process texture data with validation and optimization
    fn process_texture_data(data: &[u8]) -> AssetResult<Vec<u8>> {
        // Basic texture validation and potential format conversion
        // In a real implementation, you'd use image processing libraries

        // Validate common image formats
        if data.len() < 8 {
            return Err(AssetError::LoadFailed("Invalid texture data: too small".to_string()));
        }

        // Check for PNG signature
        if data.starts_with(b"\x89PNG\r\n\x1a\n") {
            Ok(data.to_vec())
        }
        // Check for JPEG signature
        else if data.starts_with(b"\xFF\xD8\xFF") {
            Ok(data.to_vec())
        }
        // Add more format checks as needed
        else {
            Ok(data.to_vec()) // Pass through for now
        }
    }

    /// Process audio data with validation
    fn process_audio_data(data: &[u8]) -> AssetResult<Vec<u8>> {
        // Basic audio validation
        if data.len() < 16 {
            return Err(AssetError::LoadFailed("Invalid audio data: too small".to_string()));
        }

        // Check for WAV signature
        if data.starts_with(b"RIFF") && data[8..12] == *b"WAVE" {
            Ok(data.to_vec())
        }
        // Check for OGG signature
        else if data.starts_with(b"OggS") {
            Ok(data.to_vec())
        }
        else {
            Ok(data.to_vec()) // Pass through for now
        }
    }

    /// Process configuration data with validation
    fn process_config_data(data: &[u8]) -> AssetResult<Vec<u8>> {
        // Validate JSON/TOML/YAML configs
        let text = String::from_utf8(data.to_vec())
            .map_err(|e| AssetError::LoadFailed(format!("Invalid UTF-8 in config: {}", e)))?;

        // Try parsing as JSON first
        if serde_json::from_str::<serde_json::Value>(&text).is_ok() {
            return Ok(data.to_vec());
        }

        // Try parsing as TOML
        if toml::from_str::<toml::Value>(&text).is_ok() {
            return Ok(data.to_vec());
        }

        // If neither works, still pass through
        Ok(data.to_vec())
    }

    /// Compress data using flate2
    fn compress_data(data: &[u8], level: u32) -> Option<Vec<u8>> {
        use flate2::{Compression, write::GzEncoder};
        use std::io::Write;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::new(level));
        encoder.write_all(data).ok()?;
        encoder.finish().ok()
    }

    /// Decompress data using flate2
    fn decompress_data(compressed: &[u8]) -> Option<Vec<u8>> {
        use flate2::read::GzDecoder;
        use std::io::Read;

        let mut decoder = GzDecoder::new(compressed);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).ok()?;
        Some(decompressed)
    }

    /// Shutdown the processor gracefully
    pub fn shutdown(&self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

impl Drop for ParallelAssetProcessor {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_parallel_processor_creation() {
        let config = ParallelProcessorConfig::default();
        let processor = ParallelAssetProcessor::new(config);
        assert!(processor.is_ok());
    }

    #[test]
    fn test_memory_mapped_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let test_data = b"Hello, memory-mapped world!";

        fs::write(&file_path, test_data).unwrap();

        let mapped = MappedFile::new(&file_path).unwrap();
        assert_eq!(mapped.data(), test_data);
        assert_eq!(mapped.size(), test_data.len());
    }

    #[test]
    fn test_task_priority_ordering() {
        let mut tasks = vec![
            ProcessingTask {
                id: "low".to_string(),
                asset_type: AssetType::Texture,
                file_path: PathBuf::from("test"),
                priority: TaskPriority::Low,
                metadata: AssetMetadata::new("low".to_string(), AssetType::Texture, PathBuf::from("test")),
            },
            ProcessingTask {
                id: "critical".to_string(),
                asset_type: AssetType::Audio,
                file_path: PathBuf::from("test"),
                priority: TaskPriority::Critical,
                metadata: AssetMetadata::new("critical".to_string(), AssetType::Audio, PathBuf::from("test")),
            },
            ProcessingTask {
                id: "normal".to_string(),
                asset_type: AssetType::Config,
                file_path: PathBuf::from("test"),
                priority: TaskPriority::Normal,
                metadata: AssetMetadata::new("normal".to_string(), AssetType::Config, PathBuf::from("test")),
            },
        ];

        tasks.sort_by_key(|task| std::cmp::Reverse(task.priority));

        assert_eq!(tasks[0].id, "critical");
        assert_eq!(tasks[1].id, "normal");
        assert_eq!(tasks[2].id, "low");
    }

    #[test]
    fn test_compression() {
        let original_data = b"This is test data that should compress well when repeated. ".repeat(100);

        let compressed = ParallelAssetProcessor::compress_data(&original_data, 6);
        assert!(compressed.is_some());

        let compressed = compressed.unwrap();
        assert!(compressed.len() < original_data.len());

        let decompressed = ParallelAssetProcessor::decompress_data(&compressed);
        assert!(decompressed.is_some());
        assert_eq!(decompressed.unwrap(), original_data);
    }

    #[test]
    fn test_processor_stats() {
        let stats = ProcessorStats::default();

        assert_eq!(stats.cache_hit_rate(), 0.0);
        assert_eq!(stats.success_rate(), 0.0);
        assert_eq!(stats.average_processing_time(), Duration::ZERO);

        stats.cache_hits.store(7, Ordering::Relaxed);
        stats.cache_misses.store(3, Ordering::Relaxed);
        assert_eq!(stats.cache_hit_rate(), 0.7);

        stats.tasks_processed.store(9, Ordering::Relaxed);
        stats.tasks_failed.store(1, Ordering::Relaxed);
        assert_eq!(stats.success_rate(), 0.9);
    }
}