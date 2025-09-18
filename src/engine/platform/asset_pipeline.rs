/*!
 * Robin Engine Asset Pipeline
 *
 * Advanced asset management system with hot-reloading, compression,
 * platform-specific optimizations, and real-time asset processing.
 */

use crate::engine::error::{RobinResult, RobinError};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, SystemTime};
use notify::{Watcher, RecursiveMode, watcher};
use serde::{Serialize, Deserialize};

/// Core asset pipeline system
#[derive(Debug)]
pub struct AssetPipeline {
    config: AssetPipelineConfig,
    asset_registry: Arc<Mutex<AssetRegistry>>,
    processors: HashMap<AssetType, Box<dyn AssetProcessor>>,
    watcher: Option<AssetWatcher>,
    hot_reload_manager: HotReloadManager,
    compression_manager: CompressionManager,
    platform_optimizer: PlatformAssetOptimizer,
}

impl AssetPipeline {
    pub fn new() -> RobinResult<Self> {
        let config = AssetPipelineConfig::default();
        let asset_registry = Arc::new(Mutex::new(AssetRegistry::new()));

        Ok(Self {
            config,
            asset_registry: asset_registry.clone(),
            processors: Self::initialize_processors()?,
            watcher: None,
            hot_reload_manager: HotReloadManager::new(asset_registry.clone())?,
            compression_manager: CompressionManager::new()?,
            platform_optimizer: PlatformAssetOptimizer::new()?,
        })
    }

    /// Start watching for asset changes
    pub fn start_watching(&mut self) -> RobinResult<()> {
        let mut watcher = AssetWatcher::new(self.asset_registry.clone())?;
        watcher.start_watching(&self.config.asset_directories)?;
        self.watcher = Some(watcher);

        println!("🔍 Asset Pipeline: Started watching for changes");
        Ok(())
    }

    /// Process all assets in the asset directories
    pub fn process_all_assets(&mut self) -> RobinResult<AssetProcessingResult> {
        let mut result = AssetProcessingResult::new();

        for asset_dir in &self.config.asset_directories {
            result.merge(self.process_directory(asset_dir)?);
        }

        println!("📦 Asset Pipeline: Processed {} assets", result.processed_count);
        Ok(result)
    }

    /// Process assets in a specific directory
    pub fn process_directory(&mut self, directory: &Path) -> RobinResult<AssetProcessingResult> {
        let mut result = AssetProcessingResult::new();

        if !directory.exists() {
            return Err(RobinError::Assets(format!("Asset directory does not exist: {:?}", directory)));
        }

        for entry in std::fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(asset_type) = AssetType::from_path(&path) {
                    match self.process_asset(&path, asset_type) {
                        Ok(asset_id) => {
                            result.processed_count += 1;
                            result.processed_assets.push(asset_id);
                        }
                        Err(e) => {
                            result.failed_count += 1;
                            result.failed_assets.push((path, e));
                        }
                    }
                }
            } else if path.is_dir() {
                result.merge(self.process_directory(&path)?);
            }
        }

        Ok(result)
    }

    /// Process a single asset
    pub fn process_asset(&mut self, path: &Path, asset_type: AssetType) -> RobinResult<AssetId> {
        let processor = self.processors.get(&asset_type)
            .ok_or_else(|| RobinError::Assets(format!("No processor for asset type: {:?}", asset_type)))?;

        // Process the asset
        let processed_asset = processor.process(path, &self.config)?;

        // Apply platform optimizations
        let optimized_asset = self.platform_optimizer.optimize(processed_asset, &self.config.target_platform)?;

        // Apply compression if enabled
        let final_asset = if self.config.enable_compression {
            self.compression_manager.compress(optimized_asset)?
        } else {
            optimized_asset
        };

        // Register the asset
        let asset_id = AssetId::from_path(path);
        let mut registry = self.asset_registry.lock().unwrap();
        registry.register_asset(asset_id.clone(), final_asset)?;

        Ok(asset_id)
    }

    /// Update the asset pipeline (call this each frame)
    pub fn update(&mut self, _delta_time: f32) -> RobinResult<()> {
        // Process hot reload events
        self.hot_reload_manager.process_events()?;

        // Update asset watchers
        if let Some(watcher) = &mut self.watcher {
            watcher.update()?;
        }

        Ok(())
    }

    /// Get an asset by ID
    pub fn get_asset(&self, asset_id: &AssetId) -> Option<ProcessedAsset> {
        let registry = self.asset_registry.lock().unwrap();
        registry.get_asset(asset_id).cloned()
    }

    /// Get all assets of a specific type
    pub fn get_assets_by_type(&self, asset_type: AssetType) -> Vec<(AssetId, ProcessedAsset)> {
        let registry = self.asset_registry.lock().unwrap();
        registry.get_assets_by_type(asset_type)
    }

    /// Reload an asset
    pub fn reload_asset(&mut self, asset_id: &AssetId) -> RobinResult<()> {
        let registry = self.asset_registry.lock().unwrap();
        if let Some(asset) = registry.get_asset(asset_id) {
            let path = asset.source_path.clone();
            drop(registry);

            if let Some(asset_type) = AssetType::from_path(&path) {
                self.process_asset(&path, asset_type)?;
                self.hot_reload_manager.notify_asset_reloaded(asset_id.clone())?;
            }
        }

        Ok(())
    }

    /// Shutdown the asset pipeline
    pub fn shutdown(&mut self) -> RobinResult<()> {
        if let Some(watcher) = &mut self.watcher {
            watcher.stop()?;
        }
        self.hot_reload_manager.shutdown()?;

        println!("🛑 Asset Pipeline: Shutdown complete");
        Ok(())
    }

    fn initialize_processors() -> RobinResult<HashMap<AssetType, Box<dyn AssetProcessor>>> {
        let mut processors: HashMap<AssetType, Box<dyn AssetProcessor>> = HashMap::new();

        processors.insert(AssetType::Texture, Box::new(TextureProcessor::new()?));
        processors.insert(AssetType::Model, Box::new(ModelProcessor::new()?));
        processors.insert(AssetType::Audio, Box::new(AudioProcessor::new()?));
        processors.insert(AssetType::Shader, Box::new(ShaderProcessor::new()?));
        processors.insert(AssetType::Font, Box::new(FontProcessor::new()?));
        processors.insert(AssetType::Animation, Box::new(AnimationProcessor::new()?));
        processors.insert(AssetType::Material, Box::new(MaterialProcessor::new()?));
        processors.insert(AssetType::Scene, Box::new(SceneProcessor::new()?));

        Ok(processors)
    }
}

/// Asset pipeline configuration
#[derive(Debug, Clone)]
pub struct AssetPipelineConfig {
    pub asset_directories: Vec<PathBuf>,
    pub output_directory: PathBuf,
    pub cache_directory: PathBuf,
    pub enable_hot_reload: bool,
    pub enable_compression: bool,
    pub compression_level: CompressionLevel,
    pub target_platform: crate::engine::platform::Platform,
    pub optimization_level: OptimizationLevel,
    pub texture_compression: TextureCompressionConfig,
    pub audio_compression: AudioCompressionConfig,
    pub model_optimization: ModelOptimizationConfig,
    pub max_parallel_jobs: usize,
}

impl Default for AssetPipelineConfig {
    fn default() -> Self {
        Self {
            asset_directories: vec![
                PathBuf::from("assets"),
                PathBuf::from("content"),
                PathBuf::from("resources"),
            ],
            output_directory: PathBuf::from("build/assets"),
            cache_directory: PathBuf::from("cache/assets"),
            enable_hot_reload: true,
            enable_compression: true,
            compression_level: CompressionLevel::Medium,
            target_platform: crate::engine::platform::Platform::detect_current(),
            optimization_level: OptimizationLevel::High,
            texture_compression: TextureCompressionConfig::default(),
            audio_compression: AudioCompressionConfig::default(),
            model_optimization: ModelOptimizationConfig::default(),
            max_parallel_jobs: num_cpus::get(),
        }
    }
}

/// Asset types supported by the pipeline
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetType {
    Texture,
    Model,
    Audio,
    Shader,
    Font,
    Animation,
    Material,
    Scene,
    Data,
    Script,
}

impl AssetType {
    pub fn from_path(path: &Path) -> Option<Self> {
        path.extension()?.to_str().map(|ext| {
            match ext.to_lowercase().as_str() {
                "png" | "jpg" | "jpeg" | "tga" | "bmp" | "exr" | "hdr" => AssetType::Texture,
                "obj" | "fbx" | "gltf" | "glb" | "dae" | "3ds" => AssetType::Model,
                "wav" | "mp3" | "ogg" | "flac" | "aac" => AssetType::Audio,
                "wgsl" | "glsl" | "hlsl" | "spv" => AssetType::Shader,
                "ttf" | "otf" | "woff" | "woff2" => AssetType::Font,
                "anim" | "skeletal" | "motion" => AssetType::Animation,
                "mat" | "material" => AssetType::Material,
                "scene" | "level" | "map" => AssetType::Scene,
                "json" | "toml" | "yaml" | "xml" => AssetType::Data,
                "lua" | "js" | "py" | "rs" => AssetType::Script,
                _ => AssetType::Data,
            }
        })
    }

    pub fn get_preferred_extensions(&self) -> &'static [&'static str] {
        match self {
            AssetType::Texture => &["png", "jpg", "exr"],
            AssetType::Model => &["gltf", "fbx", "obj"],
            AssetType::Audio => &["ogg", "wav", "mp3"],
            AssetType::Shader => &["wgsl", "glsl"],
            AssetType::Font => &["ttf", "otf"],
            AssetType::Animation => &["anim"],
            AssetType::Material => &["mat"],
            AssetType::Scene => &["scene"],
            AssetType::Data => &["json", "toml"],
            AssetType::Script => &["lua", "js"],
        }
    }
}

/// Unique identifier for an asset
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetId {
    pub path: String,
    pub hash: u64,
}

impl AssetId {
    pub fn from_path(path: &Path) -> Self {
        let path_str = path.to_string_lossy().to_string();
        let hash = Self::calculate_hash(&path_str);

        Self {
            path: path_str,
            hash,
        }
    }

    fn calculate_hash(input: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        hasher.finish()
    }
}

/// Processed asset data
#[derive(Debug, Clone)]
pub struct ProcessedAsset {
    pub id: AssetId,
    pub asset_type: AssetType,
    pub data: Vec<u8>,
    pub metadata: AssetMetadata,
    pub source_path: PathBuf,
    pub processed_path: PathBuf,
    pub dependencies: Vec<AssetId>,
    pub platform_variants: HashMap<crate::engine::platform::Platform, Vec<u8>>,
}

/// Asset metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMetadata {
    pub size: u64,
    pub creation_time: SystemTime,
    pub modification_time: SystemTime,
    pub compression_ratio: f32,
    pub platform_optimizations: Vec<String>,
    pub custom_properties: HashMap<String, String>,
}

/// Asset processing result
#[derive(Debug)]
pub struct AssetProcessingResult {
    pub processed_count: usize,
    pub failed_count: usize,
    pub processed_assets: Vec<AssetId>,
    pub failed_assets: Vec<(PathBuf, RobinError)>,
    pub processing_time: Duration,
}

impl AssetProcessingResult {
    pub fn new() -> Self {
        Self {
            processed_count: 0,
            failed_count: 0,
            processed_assets: Vec::new(),
            failed_assets: Vec::new(),
            processing_time: Duration::from_secs(0),
        }
    }

    pub fn merge(&mut self, other: AssetProcessingResult) {
        self.processed_count += other.processed_count;
        self.failed_count += other.failed_count;
        self.processed_assets.extend(other.processed_assets);
        self.failed_assets.extend(other.failed_assets);
        self.processing_time += other.processing_time;
    }
}

/// Asset processor trait
pub trait AssetProcessor: Send + Sync {
    fn process(&self, path: &Path, config: &AssetPipelineConfig) -> RobinResult<ProcessedAsset>;
    fn get_supported_extensions(&self) -> &[&str];
    fn get_asset_type(&self) -> AssetType;
}

/// Asset registry for managing loaded assets
#[derive(Debug)]
pub struct AssetRegistry {
    assets: HashMap<AssetId, ProcessedAsset>,
    assets_by_type: HashMap<AssetType, Vec<AssetId>>,
    dependency_graph: HashMap<AssetId, Vec<AssetId>>,
}

impl AssetRegistry {
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
            assets_by_type: HashMap::new(),
            dependency_graph: HashMap::new(),
        }
    }

    pub fn register_asset(&mut self, id: AssetId, asset: ProcessedAsset) -> RobinResult<()> {
        let asset_type = asset.asset_type;

        // Add to main registry
        self.assets.insert(id.clone(), asset);

        // Add to type index
        self.assets_by_type.entry(asset_type)
            .or_insert_with(Vec::new)
            .push(id);

        Ok(())
    }

    pub fn get_asset(&self, id: &AssetId) -> Option<&ProcessedAsset> {
        self.assets.get(id)
    }

    pub fn get_assets_by_type(&self, asset_type: AssetType) -> Vec<(AssetId, ProcessedAsset)> {
        self.assets_by_type.get(&asset_type)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.assets.get(id).map(|asset| (id.clone(), asset.clone())))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn remove_asset(&mut self, id: &AssetId) -> Option<ProcessedAsset> {
        if let Some(asset) = self.assets.remove(id) {
            // Remove from type index
            if let Some(type_list) = self.assets_by_type.get_mut(&asset.asset_type) {
                type_list.retain(|asset_id| asset_id != id);
            }

            // Remove from dependency graph
            self.dependency_graph.remove(id);

            Some(asset)
        } else {
            None
        }
    }
}

/// Asset file system watcher
#[derive(Debug)]
pub struct AssetWatcher {
    watcher: notify::RecommendedWatcher,
    event_receiver: mpsc::Receiver<notify::DebouncedEvent>,
    asset_registry: Arc<Mutex<AssetRegistry>>,
}

impl AssetWatcher {
    pub fn new(asset_registry: Arc<Mutex<AssetRegistry>>) -> RobinResult<Self> {
        let (tx, rx) = mpsc::channel();
        let watcher = watcher(tx, Duration::from_millis(100))
            .map_err(|e| RobinError::Assets(format!("Failed to create file watcher: {}", e)))?;

        Ok(Self {
            watcher,
            event_receiver: rx,
            asset_registry,
        })
    }

    pub fn start_watching(&mut self, directories: &[PathBuf]) -> RobinResult<()> {
        for dir in directories {
            if dir.exists() {
                self.watcher.watch(dir, RecursiveMode::Recursive)
                    .map_err(|e| RobinError::Assets(format!("Failed to watch directory {:?}: {}", dir, e)))?;
            }
        }

        Ok(())
    }

    pub fn update(&mut self) -> RobinResult<()> {
        while let Ok(event) = self.event_receiver.try_recv() {
            self.handle_file_event(event)?;
        }
        Ok(())
    }

    pub fn stop(&mut self) -> RobinResult<()> {
        // Watcher will be dropped automatically
        Ok(())
    }

    fn handle_file_event(&mut self, event: notify::DebouncedEvent) -> RobinResult<()> {
        use notify::DebouncedEvent::*;

        match event {
            Write(path) | Create(path) => {
                if let Some(asset_type) = AssetType::from_path(&path) {
                    println!("🔄 Asset changed: {:?}", path);
                    // Asset pipeline will handle the actual reprocessing
                }
            }
            Remove(path) => {
                let asset_id = AssetId::from_path(&path);
                let mut registry = self.asset_registry.lock().unwrap();
                if registry.remove_asset(&asset_id).is_some() {
                    println!("🗑️ Asset removed: {:?}", path);
                }
            }
            _ => {}
        }

        Ok(())
    }
}

/// Hot reload manager for real-time asset updates
#[derive(Debug)]
pub struct HotReloadManager {
    asset_registry: Arc<Mutex<AssetRegistry>>,
    reload_callbacks: HashMap<AssetType, Vec<Box<dyn Fn(&AssetId) + Send + Sync>>>,
    pending_reloads: Vec<AssetId>,
}

impl HotReloadManager {
    pub fn new(asset_registry: Arc<Mutex<AssetRegistry>>) -> RobinResult<Self> {
        Ok(Self {
            asset_registry,
            reload_callbacks: HashMap::new(),
            pending_reloads: Vec::new(),
        })
    }

    pub fn register_reload_callback<F>(&mut self, asset_type: AssetType, callback: F)
    where
        F: Fn(&AssetId) + Send + Sync + 'static,
    {
        self.reload_callbacks.entry(asset_type)
            .or_insert_with(Vec::new)
            .push(Box::new(callback));
    }

    pub fn notify_asset_reloaded(&mut self, asset_id: AssetId) -> RobinResult<()> {
        self.pending_reloads.push(asset_id);
        Ok(())
    }

    pub fn process_events(&mut self) -> RobinResult<()> {
        let pending = std::mem::take(&mut self.pending_reloads);

        for asset_id in pending {
            let registry = self.asset_registry.lock().unwrap();
            if let Some(asset) = registry.get_asset(&asset_id) {
                let asset_type = asset.asset_type;
                drop(registry);

                // Call registered callbacks
                if let Some(callbacks) = self.reload_callbacks.get(&asset_type) {
                    for callback in callbacks {
                        callback(&asset_id);
                    }
                }
            }
        }

        Ok(())
    }

    pub fn shutdown(&mut self) -> RobinResult<()> {
        self.reload_callbacks.clear();
        self.pending_reloads.clear();
        Ok(())
    }
}

// Asset processor implementations
pub struct TextureProcessor;
pub struct ModelProcessor;
pub struct AudioProcessor;
pub struct ShaderProcessor;
pub struct FontProcessor;
pub struct AnimationProcessor;
pub struct MaterialProcessor;
pub struct SceneProcessor;

// Compression and optimization systems
#[derive(Debug)]
pub struct CompressionManager;

#[derive(Debug)]
pub struct PlatformAssetOptimizer;

// Configuration structures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionLevel {
    None,
    Low,
    Medium,
    High,
    Maximum,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationLevel {
    None,
    Low,
    Medium,
    High,
    Aggressive,
}

#[derive(Debug, Clone)]
pub struct TextureCompressionConfig {
    pub format: TextureCompressionFormat,
    pub quality: f32,
    pub generate_mipmaps: bool,
    pub max_resolution: (u32, u32),
}

#[derive(Debug, Clone)]
pub enum TextureCompressionFormat {
    None,
    DXT1,
    DXT5,
    BC7,
    ASTC,
    ETC2,
}

#[derive(Debug, Clone)]
pub struct AudioCompressionConfig {
    pub format: AudioCompressionFormat,
    pub quality: f32,
    pub sample_rate: u32,
    pub channels: u32,
}

#[derive(Debug, Clone)]
pub enum AudioCompressionFormat {
    None,
    OggVorbis,
    MP3,
    AAC,
    FLAC,
}

#[derive(Debug, Clone)]
pub struct ModelOptimizationConfig {
    pub merge_vertices: bool,
    pub optimize_indices: bool,
    pub generate_lods: bool,
    pub max_vertices_per_lod: u32,
    pub tangent_generation: bool,
}

// Implementation stubs for processors and managers
impl TextureProcessor {
    pub fn new() -> RobinResult<Self> { Ok(Self) }
}

impl AssetProcessor for TextureProcessor {
    fn process(&self, path: &Path, _config: &AssetPipelineConfig) -> RobinResult<ProcessedAsset> {
        let id = AssetId::from_path(path);
        let data = std::fs::read(path)?;

        Ok(ProcessedAsset {
            id: id.clone(),
            asset_type: AssetType::Texture,
            data,
            metadata: AssetMetadata {
                size: 0,
                creation_time: SystemTime::now(),
                modification_time: SystemTime::now(),
                compression_ratio: 1.0,
                platform_optimizations: Vec::new(),
                custom_properties: HashMap::new(),
            },
            source_path: path.to_path_buf(),
            processed_path: path.to_path_buf(),
            dependencies: Vec::new(),
            platform_variants: HashMap::new(),
        })
    }

    fn get_supported_extensions(&self) -> &[&str] {
        &["png", "jpg", "jpeg", "tga", "bmp", "exr", "hdr"]
    }

    fn get_asset_type(&self) -> AssetType {
        AssetType::Texture
    }
}

// Similar implementations for other processors...
impl ModelProcessor { pub fn new() -> RobinResult<Self> { Ok(Self) } }
impl AudioProcessor { pub fn new() -> RobinResult<Self> { Ok(Self) } }
impl ShaderProcessor { pub fn new() -> RobinResult<Self> { Ok(Self) } }
impl FontProcessor { pub fn new() -> RobinResult<Self> { Ok(Self) } }
impl AnimationProcessor { pub fn new() -> RobinResult<Self> { Ok(Self) } }
impl MaterialProcessor { pub fn new() -> RobinResult<Self> { Ok(Self) } }
impl SceneProcessor { pub fn new() -> RobinResult<Self> { Ok(Self) } }

impl CompressionManager {
    pub fn new() -> RobinResult<Self> { Ok(Self) }
    pub fn compress(&self, asset: ProcessedAsset) -> RobinResult<ProcessedAsset> { Ok(asset) }
}

impl PlatformAssetOptimizer {
    pub fn new() -> RobinResult<Self> { Ok(Self) }
    pub fn optimize(&self, asset: ProcessedAsset, _platform: &crate::engine::platform::Platform) -> RobinResult<ProcessedAsset> { Ok(asset) }
}

// Default implementations for other processors
macro_rules! impl_basic_processor {
    ($processor:ident, $asset_type:ident, $extensions:expr) => {
        impl AssetProcessor for $processor {
            fn process(&self, path: &Path, _config: &AssetPipelineConfig) -> RobinResult<ProcessedAsset> {
                let id = AssetId::from_path(path);
                let data = std::fs::read(path)?;

                Ok(ProcessedAsset {
                    id: id.clone(),
                    asset_type: AssetType::$asset_type,
                    data,
                    metadata: AssetMetadata {
                        size: 0,
                        creation_time: SystemTime::now(),
                        modification_time: SystemTime::now(),
                        compression_ratio: 1.0,
                        platform_optimizations: Vec::new(),
                        custom_properties: HashMap::new(),
                    },
                    source_path: path.to_path_buf(),
                    processed_path: path.to_path_buf(),
                    dependencies: Vec::new(),
                    platform_variants: HashMap::new(),
                })
            }

            fn get_supported_extensions(&self) -> &[&str] {
                $extensions
            }

            fn get_asset_type(&self) -> AssetType {
                AssetType::$asset_type
            }
        }
    };
}

impl_basic_processor!(ModelProcessor, Model, &["obj", "fbx", "gltf", "glb"]);
impl_basic_processor!(AudioProcessor, Audio, &["wav", "mp3", "ogg", "flac"]);
impl_basic_processor!(ShaderProcessor, Shader, &["wgsl", "glsl", "hlsl"]);
impl_basic_processor!(FontProcessor, Font, &["ttf", "otf", "woff"]);
impl_basic_processor!(AnimationProcessor, Animation, &["anim"]);
impl_basic_processor!(MaterialProcessor, Material, &["mat"]);
impl_basic_processor!(SceneProcessor, Scene, &["scene"]);

// Default implementations for configuration
impl Default for TextureCompressionConfig {
    fn default() -> Self {
        Self {
            format: TextureCompressionFormat::BC7,
            quality: 0.8,
            generate_mipmaps: true,
            max_resolution: (2048, 2048),
        }
    }
}

impl Default for AudioCompressionConfig {
    fn default() -> Self {
        Self {
            format: AudioCompressionFormat::OggVorbis,
            quality: 0.8,
            sample_rate: 44100,
            channels: 2,
        }
    }
}

impl Default for ModelOptimizationConfig {
    fn default() -> Self {
        Self {
            merge_vertices: true,
            optimize_indices: true,
            generate_lods: true,
            max_vertices_per_lod: 65536,
            tangent_generation: true,
        }
    }
}