use crate::engine::{
    assets::{AssetType, AssetConfig, AssetResult, AssetError, AssetMetadata},
    error::{RobinError, RobinResult, ErrorContext},
    logging::PerformanceMetrics,
};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock},
    time::{Duration, Instant, SystemTime},
    fs,
};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// Comprehensive asset management pipeline
pub struct AssetPipeline {
    config: PipelineConfig,
    processors: Arc<RwLock<HashMap<AssetType, Box<dyn AssetProcessor + Send + Sync>>>>,
    cache: Arc<RwLock<AssetCache>>,
    dependency_graph: Arc<RwLock<DependencyGraph>>,
    build_queue: Arc<Mutex<BuildQueue>>,
    statistics: Arc<Mutex<PipelineStatistics>>,
    watchers: Arc<Mutex<Vec<PathBuf>>>,
}

/// Pipeline configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub source_dir: PathBuf,
    pub output_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub temp_dir: PathBuf,
    pub parallel_builds: usize,
    pub cache_enabled: bool,
    pub compression_enabled: bool,
    pub optimization_level: OptimizationLevel,
    pub target_platforms: Vec<TargetPlatform>,
    pub build_on_change: bool,
    pub incremental_builds: bool,
    pub validation_enabled: bool,
    pub asset_bundling: BundlingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationLevel {
    None,
    Basic,
    Aggressive,
    Custom(HashMap<AssetType, String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TargetPlatform {
    Desktop,
    Web,
    Mobile,
    Console,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundlingConfig {
    pub enabled: bool,
    pub bundle_size_limit: usize,
    pub bundle_by_type: bool,
    pub bundle_by_scene: bool,
    pub compression_format: CompressionFormat,
}

impl Default for BundlingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bundle_size_limit: 10 * 1024 * 1024, // 10MB
            bundle_by_type: true,
            bundle_by_scene: false,
            compression_format: CompressionFormat::None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionFormat {
    None,
    Gzip,
    Brotli,
    Zstd,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            source_dir: PathBuf::from("assets/src"),
            output_dir: PathBuf::from("assets/built"),
            cache_dir: PathBuf::from("assets/cache"),
            temp_dir: PathBuf::from("assets/temp"),
            parallel_builds: num_cpus::get(),
            cache_enabled: true,
            compression_enabled: true,
            optimization_level: OptimizationLevel::Basic,
            target_platforms: vec![TargetPlatform::Desktop],
            build_on_change: true,
            incremental_builds: true,
            validation_enabled: true,
            asset_bundling: BundlingConfig {
                enabled: false,
                bundle_size_limit: 10 * 1024 * 1024, // 10MB
                bundle_by_type: true,
                bundle_by_scene: false,
                compression_format: CompressionFormat::Zstd,
            },
        }
    }
}

/// Asset processing trait
pub trait AssetProcessor {
    /// Process a raw asset into game-ready format
    fn process(&self, input: &ProcessingInput) -> AssetResult<ProcessingOutput>;
    
    /// Get the output file extension for this processor
    fn output_extension(&self) -> &str;
    
    /// Check if this processor supports the given file
    fn supports_file(&self, path: &Path) -> bool;
    
    /// Get processing dependencies for this asset
    fn get_dependencies(&self, path: &Path) -> AssetResult<Vec<PathBuf>>;
    
    /// Validate the processed asset
    fn validate(&self, output: &ProcessingOutput) -> AssetResult<()>;
    
    /// Get optimization options for this processor
    fn optimization_options(&self) -> Vec<String>;
}

#[derive(Debug, Clone)]
pub struct ProcessingInput {
    pub source_path: PathBuf,
    pub output_path: PathBuf,
    pub asset_type: AssetType,
    pub target_platform: TargetPlatform,
    pub optimization_level: OptimizationLevel,
    pub metadata: Option<AssetMetadata>,
    pub dependencies: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ProcessingOutput {
    pub output_path: PathBuf,
    pub processed_data: Vec<u8>,
    pub metadata: ProcessedAssetMetadata,
    pub dependencies: Vec<PathBuf>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedAssetMetadata {
    pub original_path: PathBuf,
    pub processed_path: PathBuf,
    pub asset_type: AssetType,
    pub file_size: usize,
    pub processed_size: usize,
    pub compression_ratio: f32,
    pub processing_time: Duration,
    pub checksum: String,
    pub dependencies: Vec<String>,
    pub platform: TargetPlatform,
    pub optimization_applied: Vec<String>,
    pub created_at: DateTime<Utc>,
}

/// Asset cache system
struct AssetCache {
    entries: HashMap<String, CacheEntry>,
    max_size: usize,
    current_size: usize,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    key: String,
    path: PathBuf,
    metadata: ProcessedAssetMetadata,
    last_accessed: SystemTime,
    access_count: u32,
}

/// Dependency tracking
struct DependencyGraph {
    dependencies: HashMap<PathBuf, HashSet<PathBuf>>,
    dependents: HashMap<PathBuf, HashSet<PathBuf>>,
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self {
            dependencies: HashMap::new(),
            dependents: HashMap::new(),
        }
    }
}

/// Build queue for processing assets
struct BuildQueue {
    pending: Vec<BuildJob>,
    processing: HashMap<PathBuf, BuildJob>,
    completed: Vec<BuildResult>,
    failed: Vec<BuildResult>,
}

#[derive(Debug, Clone)]
struct BuildJob {
    asset_path: PathBuf,
    asset_type: AssetType,
    priority: BuildPriority,
    created_at: Instant,
    dependencies: Vec<PathBuf>,
    target_platforms: Vec<TargetPlatform>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum BuildPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone)]
struct BuildResult {
    job: BuildJob,
    result: AssetResult<ProcessingOutput>,
    processing_time: Duration,
    completed_at: Instant,
}

/// Pipeline statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStatistics {
    pub total_assets: usize,
    pub processed_assets: usize,
    pub cached_assets: usize,
    pub failed_assets: usize,
    pub total_processing_time: Duration,
    pub cache_hit_rate: f32,
    pub average_processing_time: Duration,
    pub compression_savings: usize,
    pub last_build_time: Option<DateTime<Utc>>,
    pub build_count: u32,
    pub asset_type_stats: HashMap<AssetType, AssetTypeStats>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetTypeStats {
    pub count: usize,
    pub total_size: usize,
    pub processed_size: usize,
    pub average_processing_time: Duration,
    pub cache_hits: usize,
    pub cache_misses: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BuildStats {
    pub total_assets: usize,
    pub processed_assets: usize,
    pub failed_assets: usize,
    pub total_build_time: Duration,
    pub assets_processed: usize,
    pub assets_optimized: usize,
    pub cache_hits: usize,
    pub average_build_time_ms: f32,
}

impl AssetPipeline {
    /// Create a new asset pipeline
    pub fn new(config: PipelineConfig) -> AssetResult<Self> {
        // Create necessary directories
        for dir in [&config.source_dir, &config.output_dir, &config.cache_dir, &config.temp_dir] {
            if !dir.exists() {
                fs::create_dir_all(dir)
                    .map_err(|e| AssetError::LoadFailed(format!("Creating pipeline directory {}: {}", dir.display(), e)))?;
            }
        }

        let mut pipeline = Self {
            config,
            processors: Arc::new(RwLock::new(HashMap::new())),
            cache: Arc::new(RwLock::new(AssetCache {
                entries: HashMap::new(),
                max_size: 100 * 1024 * 1024, // 100MB cache
                current_size: 0,
            })),
            dependency_graph: Arc::new(RwLock::new(DependencyGraph {
                dependencies: HashMap::new(),
                dependents: HashMap::new(),
            })),
            build_queue: Arc::new(Mutex::new(BuildQueue {
                pending: Vec::new(),
                processing: HashMap::new(),
                completed: Vec::new(),
                failed: Vec::new(),
            })),
            statistics: Arc::new(Mutex::new(PipelineStatistics::default())),
            watchers: Arc::new(Mutex::new(Vec::new())),
        };

        // Register default processors
        pipeline.register_default_processors()?;

        log::info!("Asset pipeline initialized with {} parallel builds", pipeline.config.parallel_builds);
        Ok(pipeline)
    }

    pub fn dummy() -> Self {
        use std::env;
        let temp_dir = env::temp_dir().join("robin_assets_dummy");
        let config = PipelineConfig {
            source_dir: temp_dir.join("src"),
            output_dir: temp_dir.join("out"),
            cache_dir: temp_dir.join("cache"),
            temp_dir: temp_dir.join("temp"),
            parallel_builds: 1,
            cache_enabled: false,
            compression_enabled: false,
            optimization_level: OptimizationLevel::None,
            target_platforms: vec![],
            build_on_change: false,
            incremental_builds: false,
            validation_enabled: false,
            asset_bundling: BundlingConfig::default(),
        };
        
        Self {
            config,
            processors: Arc::new(RwLock::new(HashMap::new())),
            cache: Arc::new(RwLock::new(AssetCache {
                entries: HashMap::new(),
                max_size: 0,
                current_size: 0,
            })),
            dependency_graph: Arc::new(RwLock::new(DependencyGraph::default())),
            build_queue: Arc::new(Mutex::new(BuildQueue {
                pending: Vec::new(),
                processing: HashMap::new(),
                completed: Vec::new(),
                failed: Vec::new(),
            })),
            statistics: Arc::new(Mutex::new(PipelineStatistics::default())),
            watchers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Register a custom asset processor
    pub fn register_processor<P>(&self, asset_type: AssetType, processor: P) -> AssetResult<()>
    where
        P: AssetProcessor + Send + Sync + 'static,
    {
        let mut processors = self.processors.write().unwrap();
        processors.insert(asset_type.clone(), Box::new(processor));
        log::info!("Registered processor for asset type: {:?}", asset_type);
        Ok(())
    }

    /// Build all assets in the source directory
    pub fn build_all(&self) -> AssetResult<BuildReport> {
        log::info!("Starting full asset build");
        let start_time = Instant::now();

        // Discover all source assets
        let source_assets = self.discover_assets()?;
        log::info!("Discovered {} source assets", source_assets.len());

        // Build dependency graph
        self.build_dependency_graph(&source_assets)?;

        // Queue assets for processing
        self.queue_assets_for_build(&source_assets)?;

        // Process build queue
        let results = self.process_build_queue()?;

        // Generate build report
        let build_time = start_time.elapsed();
        let report = BuildReport {
            total_assets: source_assets.len(),
            processed_assets: results.processed,
            cached_assets: results.cached,
            failed_assets: results.failed,
            build_time,
            start_time: Utc::now() - chrono::Duration::from_std(build_time).unwrap_or_default(),
            warnings: results.warnings,
            errors: results.errors,
        };

        // Update statistics
        self.update_statistics(&report)?;

        log::info!("Asset build completed in {:.2}s: {} processed, {} cached, {} failed",
            build_time.as_secs_f64(), results.processed, results.cached, results.failed);

        Ok(report)
    }

    /// Build a specific asset and its dependencies
    pub fn build_asset<P: AsRef<Path>>(&self, asset_path: P) -> AssetResult<ProcessingOutput> {
        let path = asset_path.as_ref();
        log::debug!("Building single asset: {}", path.display());

        // Check cache first
        if let Some(cached) = self.check_cache(path)? {
            log::debug!("Asset found in cache: {}", path.display());
            return Ok(cached);
        }

        // Determine asset type
        let asset_type = self.determine_asset_type(path)?;

        // Get processor
        let processors = self.processors.read().unwrap();
        let processor = processors.get(&asset_type)
            .ok_or_else(|| AssetError::UnsupportedFormat(
                format!("No processor for asset type: {:?}", asset_type)
            ))?;

        // Get dependencies
        let dependencies = processor.get_dependencies(path)?;

        // Build dependencies first
        for dep in &dependencies {
            self.build_asset(dep)?;
        }

        // Create processing input
        let output_path = self.get_output_path(path, &asset_type)?;
        let input = ProcessingInput {
            source_path: path.to_path_buf(),
            output_path: output_path.clone(),
            asset_type: asset_type.clone(),
            target_platform: self.config.target_platforms.first()
                .cloned()
                .unwrap_or(TargetPlatform::Desktop),
            optimization_level: self.config.optimization_level.clone(),
            metadata: None,
            dependencies,
        };

        // Process the asset
        let start_time = Instant::now();
        let output = processor.process(&input)?;
        let processing_time = start_time.elapsed();

        // Validate output
        processor.validate(&output)?;

        // Write to disk
        fs::write(&output.output_path, &output.processed_data)?;

        // Update cache
        self.update_cache(path, &output)?;

        log::debug!("Asset processed in {:.2}ms: {} -> {}",
            processing_time.as_millis(),
            path.display(),
            output.output_path.display()
        );

        Ok(output)
    }

    /// Watch for file changes and rebuild automatically
    pub fn start_watching(&self) -> AssetResult<()> {
        if !self.config.build_on_change {
            return Ok(());
        }

        log::info!("Starting asset pipeline file watching");
        // Implementation would use a file watcher like notify
        // For now, we'll just record that watching is enabled
        Ok(())
    }

    /// Clean all built assets and cache
    pub fn clean(&self) -> AssetResult<()> {
        log::info!("Cleaning asset pipeline");

        // Clear cache
        {
            let mut cache = self.cache.write().unwrap();
            cache.entries.clear();
            cache.current_size = 0;
        }

        // Remove output directory
        if self.config.output_dir.exists() {
            fs::remove_dir_all(&self.config.output_dir)?;
            fs::create_dir_all(&self.config.output_dir)?;
        }

        // Remove cache directory
        if self.config.cache_dir.exists() {
            fs::remove_dir_all(&self.config.cache_dir)?;
            fs::create_dir_all(&self.config.cache_dir)?;
        }

        log::info!("Asset pipeline cleaned");
        Ok(())
    }

    /// Get pipeline statistics
    pub fn get_statistics(&self) -> PipelineStatistics {
        self.statistics.lock().unwrap().clone()
    }

    /// Generate asset bundles for deployment
    pub fn generate_bundles(&self) -> AssetResult<Vec<AssetBundle>> {
        if !self.config.asset_bundling.enabled {
            return Ok(Vec::new());
        }

        log::info!("Generating asset bundles");
        
        let mut bundles = Vec::new();
        
        if self.config.asset_bundling.bundle_by_type {
            bundles.extend(self.create_type_based_bundles()?);
        }

        if self.config.asset_bundling.bundle_by_scene {
            bundles.extend(self.create_scene_based_bundles()?);
        }

        log::info!("Generated {} asset bundles", bundles.len());
        Ok(bundles)
    }

    // Private implementation methods
    fn register_default_processors(&mut self) -> AssetResult<()> {
        // Register built-in processors
        self.register_processor(AssetType::Texture, TextureProcessor::new())?;
        self.register_processor(AssetType::Audio, AudioProcessor::new())?;
        self.register_processor(AssetType::Config, ConfigProcessor::new())?;
        self.register_processor(AssetType::Scene, SceneProcessor::new())?;
        Ok(())
    }

    fn discover_assets(&self) -> AssetResult<Vec<PathBuf>> {
        let mut assets = Vec::new();
        self.discover_assets_recursive(&self.config.source_dir, &mut assets)?;
        Ok(assets)
    }

    fn discover_assets_recursive(&self, dir: &Path, assets: &mut Vec<PathBuf>) -> AssetResult<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                self.discover_assets_recursive(&path, assets)?;
            } else if self.is_asset_file(&path) {
                assets.push(path);
            }
        }
        Ok(())
    }

    fn is_asset_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            matches!(ext.to_lowercase().as_str(),
                "png" | "jpg" | "jpeg" | "gif" | "bmp" | "tga" |
                "wav" | "mp3" | "ogg" | "flac" |
                "json" | "toml" | "yaml" | "yml" |
                "wgsl" | "glsl" | "hlsl" |
                "scene" | "prefab"
            )
        } else {
            false
        }
    }

    fn determine_asset_type(&self, path: &Path) -> AssetResult<AssetType> {
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| AssetError::UnsupportedFormat("No file extension".to_string()))?;

        match ext.to_lowercase().as_str() {
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "tga" => Ok(AssetType::Texture),
            "wav" | "mp3" | "ogg" | "flac" => Ok(AssetType::Audio),
            "json" | "toml" | "yaml" | "yml" => Ok(AssetType::Config),
            "wgsl" | "glsl" | "hlsl" => Ok(AssetType::Shader),
            "scene" | "prefab" => Ok(AssetType::Scene),
            "ttf" | "otf" => Ok(AssetType::Font),
            _ => Ok(AssetType::Data),
        }
    }

    fn get_output_path(&self, source_path: &Path, asset_type: &AssetType) -> AssetResult<PathBuf> {
        let relative_path = source_path.strip_prefix(&self.config.source_dir)
            .map_err(|_| AssetError::InvalidPath("Asset not in source directory".to_string()))?;

        let processors = self.processors.read().unwrap();
        let processor = processors.get(asset_type).unwrap();
        let output_ext = processor.output_extension();

        let mut output_path = self.config.output_dir.join(relative_path);
        output_path.set_extension(output_ext);

        Ok(output_path)
    }

    fn check_cache(&self, _path: &Path) -> AssetResult<Option<ProcessingOutput>> {
        // Cache implementation would check if the asset is cached and up-to-date
        Ok(None) // For now, always miss cache
    }

    fn update_cache(&self, _path: &Path, _output: &ProcessingOutput) -> AssetResult<()> {
        // Cache implementation would store the processed asset
        Ok(())
    }

    fn build_dependency_graph(&self, _assets: &[PathBuf]) -> AssetResult<()> {
        // Build dependency graph implementation
        Ok(())
    }

    fn queue_assets_for_build(&self, _assets: &[PathBuf]) -> AssetResult<()> {
        // Queue implementation
        Ok(())
    }

    fn process_build_queue(&self) -> AssetResult<BuildResults> {
        // Process build queue implementation
        Ok(BuildResults {
            processed: 0,
            cached: 0,
            failed: 0,
            warnings: Vec::new(),
            errors: Vec::new(),
        })
    }

    fn update_statistics(&self, _report: &BuildReport) -> AssetResult<()> {
        // Statistics update implementation
        Ok(())
    }

    fn create_type_based_bundles(&self) -> AssetResult<Vec<AssetBundle>> {
        // Bundle creation implementation
        Ok(Vec::new())
    }

    fn create_scene_based_bundles(&self) -> AssetResult<Vec<AssetBundle>> {
        // Scene-based bundle creation implementation
        Ok(Vec::new())
    }

    pub fn build_all_assets(&self) -> AssetResult<()> {
        // Build all assets in the pipeline
        Ok(())
    }

    pub fn build_assets_for_platform(&self, _platform: &str) -> AssetResult<()> {
        // Build assets for specific platform
        Ok(())
    }

    pub fn get_build_stats(&self) -> BuildStats {
        BuildStats::default()
    }

    pub fn clear_cache(&self) -> AssetResult<()> {
        // Clear pipeline cache
        Ok(())
    }

    pub fn update(&mut self) -> AssetResult<()> {
        // Update pipeline state
        Ok(())
    }
}

// Default processors
struct TextureProcessor;
impl TextureProcessor {
    fn new() -> Self { Self }
}

impl AssetProcessor for TextureProcessor {
    fn process(&self, input: &ProcessingInput) -> AssetResult<ProcessingOutput> {
        let data = fs::read(&input.source_path)?;
        
        // Basic texture processing (in real implementation, would optimize, resize, convert format, etc.)
        let processed_data = data; // Placeholder
        
        Ok(ProcessingOutput {
            output_path: input.output_path.clone(),
            processed_data,
            metadata: ProcessedAssetMetadata {
                original_path: input.source_path.clone(),
                processed_path: input.output_path.clone(),
                asset_type: input.asset_type.clone(),
                file_size: 0,
                processed_size: 0,
                compression_ratio: 1.0,
                processing_time: Duration::default(),
                checksum: "".to_string(),
                dependencies: Vec::new(),
                platform: input.target_platform.clone(),
                optimization_applied: Vec::new(),
                created_at: Utc::now(),
            },
            dependencies: Vec::new(),
            warnings: Vec::new(),
        })
    }

    fn output_extension(&self) -> &str { "texture" }
    fn supports_file(&self, path: &Path) -> bool {
        matches!(path.extension().and_then(|e| e.to_str()),
            Some("png") | Some("jpg") | Some("jpeg") | Some("gif") | Some("bmp") | Some("tga"))
    }
    fn get_dependencies(&self, _path: &Path) -> AssetResult<Vec<PathBuf>> { Ok(Vec::new()) }
    fn validate(&self, _output: &ProcessingOutput) -> AssetResult<()> { Ok(()) }
    fn optimization_options(&self) -> Vec<String> {
        vec!["compress".to_string(), "resize".to_string(), "format_convert".to_string()]
    }
}

// Similar implementations for other processors...
struct AudioProcessor;
impl AudioProcessor {
    fn new() -> Self { Self }
}

impl AssetProcessor for AudioProcessor {
    fn process(&self, input: &ProcessingInput) -> AssetResult<ProcessingOutput> {
        let data = fs::read(&input.source_path)?;
        Ok(ProcessingOutput {
            output_path: input.output_path.clone(),
            processed_data: data,
            metadata: ProcessedAssetMetadata {
                original_path: input.source_path.clone(),
                processed_path: input.output_path.clone(),
                asset_type: input.asset_type.clone(),
                file_size: 0,
                processed_size: 0,
                compression_ratio: 1.0,
                processing_time: Duration::default(),
                checksum: "".to_string(),
                dependencies: Vec::new(),
                platform: input.target_platform.clone(),
                optimization_applied: Vec::new(),
                created_at: Utc::now(),
            },
            dependencies: Vec::new(),
            warnings: Vec::new(),
        })
    }

    fn output_extension(&self) -> &str { "audio" }
    fn supports_file(&self, path: &Path) -> bool {
        matches!(path.extension().and_then(|e| e.to_str()),
            Some("wav") | Some("mp3") | Some("ogg") | Some("flac"))
    }
    fn get_dependencies(&self, _path: &Path) -> AssetResult<Vec<PathBuf>> { Ok(Vec::new()) }
    fn validate(&self, _output: &ProcessingOutput) -> AssetResult<()> { Ok(()) }
    fn optimization_options(&self) -> Vec<String> {
        vec!["compress".to_string(), "normalize".to_string(), "format_convert".to_string()]
    }
}

struct ConfigProcessor;
impl ConfigProcessor {
    fn new() -> Self { Self }
}

impl AssetProcessor for ConfigProcessor {
    fn process(&self, input: &ProcessingInput) -> AssetResult<ProcessingOutput> {
        let data = fs::read(&input.source_path)?;
        Ok(ProcessingOutput {
            output_path: input.output_path.clone(),
            processed_data: data,
            metadata: ProcessedAssetMetadata {
                original_path: input.source_path.clone(),
                processed_path: input.output_path.clone(),
                asset_type: input.asset_type.clone(),
                file_size: 0,
                processed_size: 0,
                compression_ratio: 1.0,
                processing_time: Duration::default(),
                checksum: "".to_string(),
                dependencies: Vec::new(),
                platform: input.target_platform.clone(),
                optimization_applied: Vec::new(),
                created_at: Utc::now(),
            },
            dependencies: Vec::new(),
            warnings: Vec::new(),
        })
    }

    fn output_extension(&self) -> &str { "config" }
    fn supports_file(&self, path: &Path) -> bool {
        matches!(path.extension().and_then(|e| e.to_str()),
            Some("json") | Some("toml") | Some("yaml") | Some("yml"))
    }
    fn get_dependencies(&self, _path: &Path) -> AssetResult<Vec<PathBuf>> { Ok(Vec::new()) }
    fn validate(&self, _output: &ProcessingOutput) -> AssetResult<()> { Ok(()) }
    fn optimization_options(&self) -> Vec<String> {
        vec!["minify".to_string(), "validate_schema".to_string()]
    }
}

struct SceneProcessor;
impl SceneProcessor {
    fn new() -> Self { Self }
}

impl AssetProcessor for SceneProcessor {
    fn process(&self, input: &ProcessingInput) -> AssetResult<ProcessingOutput> {
        let data = fs::read(&input.source_path)?;
        Ok(ProcessingOutput {
            output_path: input.output_path.clone(),
            processed_data: data,
            metadata: ProcessedAssetMetadata {
                original_path: input.source_path.clone(),
                processed_path: input.output_path.clone(),
                asset_type: input.asset_type.clone(),
                file_size: 0,
                processed_size: 0,
                compression_ratio: 1.0,
                processing_time: Duration::default(),
                checksum: "".to_string(),
                dependencies: Vec::new(),
                platform: input.target_platform.clone(),
                optimization_applied: Vec::new(),
                created_at: Utc::now(),
            },
            dependencies: Vec::new(),
            warnings: Vec::new(),
        })
    }

    fn output_extension(&self) -> &str { "scene" }
    fn supports_file(&self, path: &Path) -> bool {
        matches!(path.extension().and_then(|e| e.to_str()),
            Some("scene") | Some("prefab"))
    }
    fn get_dependencies(&self, _path: &Path) -> AssetResult<Vec<PathBuf>> { Ok(Vec::new()) }
    fn validate(&self, _output: &ProcessingOutput) -> AssetResult<()> { Ok(()) }
    fn optimization_options(&self) -> Vec<String> {
        vec!["optimize_meshes".to_string(), "bake_lighting".to_string()]
    }
}

// Supporting types
#[derive(Debug, Clone)]
pub struct BuildReport {
    pub total_assets: usize,
    pub processed_assets: usize,
    pub cached_assets: usize,
    pub failed_assets: usize,
    pub build_time: Duration,
    pub start_time: DateTime<Utc>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug)]
struct BuildResults {
    pub processed: usize,
    pub cached: usize,
    pub failed: usize,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetBundle {
    pub name: String,
    pub assets: Vec<PathBuf>,
    pub compressed_size: usize,
    pub uncompressed_size: usize,
    pub compression_format: CompressionFormat,
    pub checksum: String,
}

impl Default for PipelineStatistics {
    fn default() -> Self {
        Self {
            total_assets: 0,
            processed_assets: 0,
            cached_assets: 0,
            failed_assets: 0,
            total_processing_time: Duration::default(),
            cache_hit_rate: 0.0,
            average_processing_time: Duration::default(),
            compression_savings: 0,
            last_build_time: None,
            build_count: 0,
            asset_type_stats: HashMap::new(),
        }
    }
}

impl Default for AssetTypeStats {
    fn default() -> Self {
        Self {
            count: 0,
            total_size: 0,
            processed_size: 0,
            average_processing_time: Duration::default(),
            cache_hits: 0,
            cache_misses: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_pipeline_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = PipelineConfig {
            source_dir: temp_dir.path().join("src"),
            output_dir: temp_dir.path().join("out"),
            cache_dir: temp_dir.path().join("cache"),
            temp_dir: temp_dir.path().join("temp"),
            ..Default::default()
        };

        let pipeline = AssetPipeline::new(config).unwrap();
        let stats = pipeline.get_statistics();
        assert_eq!(stats.total_assets, 0);
    }

    #[test]
    fn test_asset_type_detection() {
        let temp_dir = TempDir::new().unwrap();
        let config = PipelineConfig {
            source_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        let pipeline = AssetPipeline::new(config).unwrap();

        assert_eq!(pipeline.determine_asset_type(Path::new("test.png")).unwrap(), AssetType::Texture);
        assert_eq!(pipeline.determine_asset_type(Path::new("test.wav")).unwrap(), AssetType::Audio);
        assert_eq!(pipeline.determine_asset_type(Path::new("test.json")).unwrap(), AssetType::Config);
    }

    #[test]
    fn test_processor_registration() {
        let temp_dir = TempDir::new().unwrap();
        let config = PipelineConfig {
            source_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        let pipeline = AssetPipeline::new(config).unwrap();

        // Default processors should be registered
        let processors = pipeline.processors.read().unwrap();
        assert!(processors.contains_key(&AssetType::Texture));
        assert!(processors.contains_key(&AssetType::Audio));
        assert!(processors.contains_key(&AssetType::Config));
    }
}