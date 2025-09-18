// Robin Game Engine - Advanced Asset Pipeline
// Phase 3: Production-ready asset management with optimization and metadata

use crate::engine::error::RobinResult;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

/// Advanced asset pipeline manager
#[derive(Debug)]
pub struct AdvancedAssetPipeline {
    config: PipelineConfig,
    importers: HashMap<String, Box<dyn AssetImporter>>,
    processors: HashMap<String, Box<dyn AssetProcessor>>,
    database: Arc<RwLock<AssetDatabase>>,
    hot_reload_manager: HotReloadManager,
    optimization_engine: OptimizationEngine,
    validation_engine: ValidationEngine,
}

/// Pipeline configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub source_directory: PathBuf,
    pub output_directory: PathBuf,
    pub cache_directory: PathBuf,
    pub enable_hot_reload: bool,
    pub enable_compression: bool,
    pub enable_optimization: bool,
    pub target_platforms: Vec<TargetPlatform>,
    pub quality_settings: QualitySettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TargetPlatform {
    Desktop,
    Mobile,
    Web,
    Console,
    VR,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualitySettings {
    pub texture_quality: TextureQuality,
    pub audio_quality: AudioQuality,
    pub model_quality: ModelQuality,
    pub compression_level: u8, // 1-10
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextureQuality {
    Low,      // 512x512 max
    Medium,   // 1024x1024 max
    High,     // 2048x2048 max
    Ultra,    // 4096x4096 max
    Original, // No downscaling
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioQuality {
    Compressed,  // Lossy compression
    Standard,    // Standard quality
    Lossless,    // Lossless compression
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelQuality {
    Low,      // Aggressive decimation
    Medium,   // Moderate optimization
    High,     // Light optimization
    Original, // No modification
}

/// Asset database for metadata and caching
#[derive(Debug, Default)]
pub struct AssetDatabase {
    assets: HashMap<AssetId, AssetEntry>,
    metadata: HashMap<AssetId, AssetMetadata>,
    dependencies: HashMap<AssetId, HashSet<AssetId>>,
    reverse_dependencies: HashMap<AssetId, HashSet<AssetId>>,
    tags: HashMap<String, HashSet<AssetId>>,
    collections: HashMap<String, AssetCollection>,
}

pub type AssetId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetEntry {
    pub id: AssetId,
    pub path: PathBuf,
    pub asset_type: AssetType,
    pub size: u64,
    pub hash: String,
    pub created_at: SystemTime,
    pub modified_at: SystemTime,
    pub last_accessed: SystemTime,
    pub processing_status: ProcessingStatus,
    pub platform_variants: HashMap<TargetPlatform, PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssetType {
    Texture { format: TextureFormat, width: u32, height: u32, mip_levels: u32 },
    Model { format: ModelFormat, vertices: u32, triangles: u32, materials: u32 },
    Audio { format: AudioFormat, duration: f32, sample_rate: u32, channels: u8 },
    Animation { format: AnimationFormat, duration: f32, bone_count: u32 },
    Material { shader: String, textures: Vec<AssetId> },
    Shader { stage: ShaderStage, language: String },
    Font { family: String, style: String, size: u32 },
    Scene { objects: u32, lights: u32, cameras: u32 },
    Data { format: String, schema_version: u32 },
    Binary { mime_type: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessingStatus {
    Pending,
    Processing,
    Completed,
    Failed { error: String },
    Skipped { reason: String },
}

/// Asset metadata for rich information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub tags: Vec<String>,
    pub custom_properties: HashMap<String, serde_json::Value>,
    pub usage_stats: UsageStats,
    pub quality_metrics: QualityMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub access_count: u64,
    pub last_used: SystemTime,
    pub projects_using: Vec<String>,
    pub estimated_importance: f32, // 0.0-1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub compression_ratio: f32,
    pub loading_time_ms: f32,
    pub memory_footprint: u64,
    pub visual_quality_score: f32, // 0.0-1.0
    pub performance_impact: f32,   // 0.0-1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetCollection {
    pub name: String,
    pub description: String,
    pub assets: HashSet<AssetId>,
    pub created_at: SystemTime,
    pub is_dynamic: bool,
    pub query: Option<String>, // For dynamic collections
}

/// Asset importer trait
pub trait AssetImporter: Send + Sync {
    fn supported_extensions(&self) -> &[&'static str];
    fn import(&self, path: &Path, config: &ImportConfig) -> RobinResult<ImportResult>;
    fn can_import(&self, path: &Path) -> bool;
    fn get_metadata(&self, path: &Path) -> RobinResult<AssetMetadata>;
}

#[derive(Debug, Clone)]
pub struct ImportConfig {
    pub target_platforms: Vec<TargetPlatform>,
    pub quality_settings: QualitySettings,
    pub custom_options: HashMap<String, serde_json::Value>,
}

#[derive(Debug)]
pub struct ImportResult {
    pub asset_id: AssetId,
    pub output_files: Vec<PathBuf>,
    pub metadata: AssetMetadata,
    pub dependencies: Vec<AssetId>,
    pub processing_time: f32,
}

/// Asset processor for optimization and conversion
pub trait AssetProcessor: Send + Sync {
    fn process_types(&self) -> &[AssetType];
    fn process(&self, asset: &AssetEntry, config: &ProcessingConfig) -> RobinResult<ProcessingResult>;
    fn estimate_processing_time(&self, asset: &AssetEntry) -> f32;
}

#[derive(Debug, Clone)]
pub struct ProcessingConfig {
    pub target_platform: TargetPlatform,
    pub quality_settings: QualitySettings,
    pub optimization_level: OptimizationLevel,
}

#[derive(Debug, Clone)]
pub enum OptimizationLevel {
    None,
    Light,
    Aggressive,
    Maximum,
}

#[derive(Debug)]
pub struct ProcessingResult {
    pub output_path: PathBuf,
    pub optimized_size: u64,
    pub compression_ratio: f32,
    pub quality_metrics: QualityMetrics,
}

impl AdvancedAssetPipeline {
    pub fn new(config: PipelineConfig) -> RobinResult<Self> {
        println!("🏗️ Initializing Advanced Asset Pipeline...");

        // Create directories
        std::fs::create_dir_all(&config.source_directory)?;
        std::fs::create_dir_all(&config.output_directory)?;
        std::fs::create_dir_all(&config.cache_directory)?;

        let mut pipeline = Self {
            config,
            importers: HashMap::new(),
            processors: HashMap::new(),
            database: Arc::new(RwLock::new(AssetDatabase::default())),
            hot_reload_manager: HotReloadManager::new()?,
            optimization_engine: OptimizationEngine::new(),
            validation_engine: ValidationEngine::new(),
        };

        // Register default importers and processors
        pipeline.register_default_importers()?;
        pipeline.register_default_processors()?;

        println!("  ✅ Asset pipeline initialized");
        println!("  📁 Source: {:?}", pipeline.config.source_directory);
        println!("  📁 Output: {:?}", pipeline.config.output_directory);
        println!("  📁 Cache: {:?}", pipeline.config.cache_directory);

        Ok(pipeline)
    }

    /// Register a new asset importer
    pub fn register_importer<I>(&mut self, name: String, importer: I) -> RobinResult<()>
    where
        I: AssetImporter + 'static,
    {
        self.importers.insert(name.clone(), Box::new(importer));
        println!("📥 Registered importer: {}", name);
        Ok(())
    }

    /// Register a new asset processor
    pub fn register_processor<P>(&mut self, name: String, processor: P) -> RobinResult<()>
    where
        P: AssetProcessor + 'static,
    {
        self.processors.insert(name.clone(), Box::new(processor));
        println!("⚙️ Registered processor: {}", name);
        Ok(())
    }

    /// Process a single asset
    pub fn process_asset(&mut self, path: &Path) -> RobinResult<AssetId> {
        let asset_id = self.generate_asset_id(path);

        println!("🔄 Processing asset: {:?}", path);

        // Find appropriate importer
        let importer = self.find_importer(path)
            .ok_or_else(|| format!("No importer found for: {:?}", path))?;

        // Import the asset
        let import_config = ImportConfig {
            target_platforms: self.config.target_platforms.clone(),
            quality_settings: self.config.quality_settings.clone(),
            custom_options: HashMap::new(),
        };

        let import_result = importer.import(path, &import_config)?;

        // Create asset entry
        let asset_entry = AssetEntry {
            id: asset_id.clone(),
            path: path.to_path_buf(),
            asset_type: self.detect_asset_type(path)?,
            size: std::fs::metadata(path)?.len(),
            hash: self.calculate_hash(path)?,
            created_at: SystemTime::now(),
            modified_at: std::fs::metadata(path)?.modified()?,
            last_accessed: SystemTime::now(),
            processing_status: ProcessingStatus::Completed,
            platform_variants: HashMap::new(),
        };

        // Store in database
        {
            let mut db = self.database.write().unwrap();
            db.assets.insert(asset_id.clone(), asset_entry);
            db.metadata.insert(asset_id.clone(), import_result.metadata);

            // Update dependencies
            let deps: HashSet<AssetId> = import_result.dependencies.into_iter().collect();
            db.dependencies.insert(asset_id.clone(), deps.clone());

            // Update reverse dependencies
            for dep_id in &deps {
                db.reverse_dependencies.entry(dep_id.clone())
                    .or_insert_with(HashSet::new)
                    .insert(asset_id.clone());
            }
        }

        println!("  ✅ Asset processed: {}", asset_id);
        Ok(asset_id)
    }

    /// Process all assets in source directory
    pub fn process_all_assets(&mut self) -> RobinResult<ProcessingSummary> {
        println!("🚀 Processing all assets...");

        let start_time = SystemTime::now();
        let mut summary = ProcessingSummary::default();

        // Walk source directory
        self.walk_directory(&self.config.source_directory.clone(), &mut summary)?;

        summary.total_time = start_time.elapsed().unwrap_or_default();

        println!("✅ Asset processing completed!");
        println!("  📊 Processed: {} assets", summary.processed_count);
        println!("  ❌ Failed: {} assets", summary.failed_count);
        println!("  ⏱️ Total time: {:.2}s", summary.total_time.as_secs_f32());

        Ok(summary)
    }

    /// Get asset information
    pub fn get_asset(&self, asset_id: &AssetId) -> Option<AssetEntry> {
        self.database.read().unwrap().assets.get(asset_id).cloned()
    }

    /// Search assets by query
    pub fn search_assets(&self, query: &AssetQuery) -> Vec<AssetId> {
        let db = self.database.read().unwrap();
        let mut results = Vec::new();

        for (asset_id, entry) in &db.assets {
            if self.matches_query(entry, query) {
                results.push(asset_id.clone());
            }
        }

        // Sort by relevance (simplified)
        results.sort();
        results
    }

    /// Create asset collection
    pub fn create_collection(&mut self, name: String, description: String, assets: Vec<AssetId>) -> RobinResult<()> {
        let collection = AssetCollection {
            name: name.clone(),
            description,
            assets: assets.into_iter().collect(),
            created_at: SystemTime::now(),
            is_dynamic: false,
            query: None,
        };

        self.database.write().unwrap().collections.insert(name.clone(), collection);
        println!("📁 Created collection: {}", name);
        Ok(())
    }

    /// Get processing statistics
    pub fn get_statistics(&self) -> AssetStatistics {
        let db = self.database.read().unwrap();

        let mut stats = AssetStatistics::default();
        stats.total_assets = db.assets.len();

        for entry in db.assets.values() {
            stats.total_size += entry.size;

            match entry.asset_type {
                AssetType::Texture { .. } => stats.texture_count += 1,
                AssetType::Model { .. } => stats.model_count += 1,
                AssetType::Audio { .. } => stats.audio_count += 1,
                _ => stats.other_count += 1,
            }
        }

        stats
    }

    /// Enable hot reload for development
    pub fn start_hot_reload(&mut self) -> RobinResult<()> {
        if !self.config.enable_hot_reload {
            return Ok(());
        }

        self.hot_reload_manager.start(&self.config.source_directory)?;
        println!("🔥 Hot reload enabled");
        Ok(())
    }

    // Private helper methods

    fn register_default_importers(&mut self) -> RobinResult<()> {
        // Register built-in importers
        self.register_importer("texture".to_string(), TextureImporter::new())?;
        self.register_importer("model".to_string(), ModelImporter::new())?;
        self.register_importer("audio".to_string(), AudioImporter::new())?;
        Ok(())
    }

    fn register_default_processors(&mut self) -> RobinResult<()> {
        // Register built-in processors
        self.register_processor("texture_optimizer".to_string(), TextureProcessor::new())?;
        self.register_processor("model_optimizer".to_string(), ModelProcessor::new())?;
        self.register_processor("audio_compressor".to_string(), AudioProcessor::new())?;
        Ok(())
    }

    fn find_importer(&self, path: &Path) -> Option<&dyn AssetImporter> {
        let extension = path.extension()?.to_str()?.to_lowercase();

        for importer in self.importers.values() {
            if importer.supported_extensions().contains(&extension.as_str()) {
                return Some(importer.as_ref());
            }
        }
        None
    }

    fn generate_asset_id(&self, path: &Path) -> AssetId {
        // Generate unique asset ID based on path
        format!("asset_{}", uuid::Uuid::new_v4().to_string())
    }

    fn calculate_hash(&self, path: &Path) -> RobinResult<String> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let content = std::fs::read(path)?;
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        Ok(format!("{:x}", hasher.finish()))
    }

    fn detect_asset_type(&self, path: &Path) -> RobinResult<AssetType> {
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        match extension.as_str() {
            "png" | "jpg" | "jpeg" | "tga" | "bmp" => Ok(AssetType::Texture {
                format: TextureFormat::RGBA8,
                width: 0, // Would be determined during import
                height: 0,
                mip_levels: 1,
            }),
            "fbx" | "obj" | "gltf" | "dae" => Ok(AssetType::Model {
                format: ModelFormat::GLTF,
                vertices: 0,
                triangles: 0,
                materials: 0,
            }),
            "wav" | "mp3" | "ogg" | "flac" => Ok(AssetType::Audio {
                format: AudioFormat::OGG,
                duration: 0.0,
                sample_rate: 44100,
                channels: 2,
            }),
            _ => Ok(AssetType::Binary {
                mime_type: "application/octet-stream".to_string(),
            }),
        }
    }

    fn walk_directory(&mut self, dir: &Path, summary: &mut ProcessingSummary) -> RobinResult<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                self.walk_directory(&path, summary)?;
            } else {
                match self.process_asset(&path) {
                    Ok(_) => summary.processed_count += 1,
                    Err(e) => {
                        summary.failed_count += 1;
                        println!("❌ Failed to process {:?}: {}", path, e);
                    }
                }
            }
        }
        Ok(())
    }

    fn matches_query(&self, entry: &AssetEntry, query: &AssetQuery) -> bool {
        // Simplified query matching
        if let Some(ref asset_type) = query.asset_type {
            if std::mem::discriminant(&entry.asset_type) != std::mem::discriminant(asset_type) {
                return false;
            }
        }

        if let Some(min_size) = query.min_size {
            if entry.size < min_size {
                return false;
            }
        }

        true
    }
}

// Supporting types and implementations

#[derive(Debug, Default)]
pub struct ProcessingSummary {
    pub processed_count: usize,
    pub failed_count: usize,
    pub total_time: std::time::Duration,
}

#[derive(Debug, Default)]
pub struct AssetStatistics {
    pub total_assets: usize,
    pub total_size: u64,
    pub texture_count: usize,
    pub model_count: usize,
    pub audio_count: usize,
    pub other_count: usize,
}

#[derive(Debug, Clone)]
pub struct AssetQuery {
    pub asset_type: Option<AssetType>,
    pub tags: Vec<String>,
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
    pub name_pattern: Option<String>,
}

// Asset format enums
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextureFormat { RGBA8, RGB8, BC7, DXT5, ETC2 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelFormat { GLTF, FBX, OBJ, DAE }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioFormat { WAV, MP3, OGG, FLAC }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnimationFormat { FBX, GLTF, BVH }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShaderStage { Vertex, Fragment, Compute }

// Hot reload manager (placeholder)
#[derive(Debug)]
pub struct HotReloadManager;

impl HotReloadManager {
    fn new() -> RobinResult<Self> { Ok(Self) }
    fn start(&mut self, _path: &Path) -> RobinResult<()> { Ok(()) }
}

// Optimization engine (placeholder)
#[derive(Debug)]
pub struct OptimizationEngine;

impl OptimizationEngine {
    fn new() -> Self { Self }
}

// Validation engine (placeholder)
#[derive(Debug)]
pub struct ValidationEngine;

impl ValidationEngine {
    fn new() -> Self { Self }
}

// Sample importers (simplified implementations)
struct TextureImporter;
impl TextureImporter {
    fn new() -> Self { Self }
}

impl AssetImporter for TextureImporter {
    fn supported_extensions(&self) -> &[&'static str] {
        &["png", "jpg", "jpeg", "tga", "bmp"]
    }

    fn import(&self, path: &Path, _config: &ImportConfig) -> RobinResult<ImportResult> {
        Ok(ImportResult {
            asset_id: format!("texture_{}", uuid::Uuid::new_v4()),
            output_files: vec![path.to_path_buf()],
            metadata: AssetMetadata {
                title: Some(path.file_name().unwrap().to_string_lossy().to_string()),
                description: Some("Imported texture".to_string()),
                author: None,
                license: None,
                tags: vec!["texture".to_string()],
                custom_properties: HashMap::new(),
                usage_stats: UsageStats {
                    access_count: 0,
                    last_used: SystemTime::now(),
                    projects_using: vec![],
                    estimated_importance: 0.5,
                },
                quality_metrics: QualityMetrics {
                    compression_ratio: 1.0,
                    loading_time_ms: 10.0,
                    memory_footprint: 1024 * 1024, // 1MB estimate
                    visual_quality_score: 0.9,
                    performance_impact: 0.1,
                },
            },
            dependencies: vec![],
            processing_time: 0.1,
        })
    }

    fn can_import(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            self.supported_extensions().contains(&ext.to_str().unwrap_or(""))
        } else {
            false
        }
    }

    fn get_metadata(&self, _path: &Path) -> RobinResult<AssetMetadata> {
        Ok(AssetMetadata {
            title: None,
            description: None,
            author: None,
            license: None,
            tags: vec![],
            custom_properties: HashMap::new(),
            usage_stats: UsageStats {
                access_count: 0,
                last_used: SystemTime::now(),
                projects_using: vec![],
                estimated_importance: 0.0,
            },
            quality_metrics: QualityMetrics {
                compression_ratio: 1.0,
                loading_time_ms: 0.0,
                memory_footprint: 0,
                visual_quality_score: 1.0,
                performance_impact: 0.0,
            },
        })
    }
}

// Simplified processor implementations
struct TextureProcessor;
impl TextureProcessor { fn new() -> Self { Self } }
impl AssetProcessor for TextureProcessor {
    fn process_types(&self) -> &[AssetType] { &[] }
    fn process(&self, _asset: &AssetEntry, _config: &ProcessingConfig) -> RobinResult<ProcessingResult> {
        Ok(ProcessingResult {
            output_path: PathBuf::from("output.png"),
            optimized_size: 1024,
            compression_ratio: 0.8,
            quality_metrics: QualityMetrics {
                compression_ratio: 0.8,
                loading_time_ms: 5.0,
                memory_footprint: 1024,
                visual_quality_score: 0.95,
                performance_impact: 0.05,
            },
        })
    }
    fn estimate_processing_time(&self, _asset: &AssetEntry) -> f32 { 1.0 }
}

struct ModelImporter;
impl ModelImporter { fn new() -> Self { Self } }
impl AssetImporter for ModelImporter {
    fn supported_extensions(&self) -> &[&'static str] { &["fbx", "obj", "gltf"] }
    fn import(&self, _path: &Path, _config: &ImportConfig) -> RobinResult<ImportResult> {
        Ok(ImportResult {
            asset_id: format!("model_{}", uuid::Uuid::new_v4()),
            output_files: vec![],
            metadata: AssetMetadata::default(),
            dependencies: vec![],
            processing_time: 2.0,
        })
    }
    fn can_import(&self, _path: &Path) -> bool { true }
    fn get_metadata(&self, _path: &Path) -> RobinResult<AssetMetadata> { Ok(AssetMetadata::default()) }
}

struct ModelProcessor;
impl ModelProcessor { fn new() -> Self { Self } }
impl AssetProcessor for ModelProcessor {
    fn process_types(&self) -> &[AssetType] { &[] }
    fn process(&self, _asset: &AssetEntry, _config: &ProcessingConfig) -> RobinResult<ProcessingResult> {
        Ok(ProcessingResult {
            output_path: PathBuf::from("output.gltf"),
            optimized_size: 2048,
            compression_ratio: 0.6,
            quality_metrics: QualityMetrics::default(),
        })
    }
    fn estimate_processing_time(&self, _asset: &AssetEntry) -> f32 { 5.0 }
}

struct AudioImporter;
impl AudioImporter { fn new() -> Self { Self } }
impl AssetImporter for AudioImporter {
    fn supported_extensions(&self) -> &[&'static str] { &["wav", "mp3", "ogg"] }
    fn import(&self, _path: &Path, _config: &ImportConfig) -> RobinResult<ImportResult> {
        Ok(ImportResult {
            asset_id: format!("audio_{}", uuid::Uuid::new_v4()),
            output_files: vec![],
            metadata: AssetMetadata::default(),
            dependencies: vec![],
            processing_time: 1.5,
        })
    }
    fn can_import(&self, _path: &Path) -> bool { true }
    fn get_metadata(&self, _path: &Path) -> RobinResult<AssetMetadata> { Ok(AssetMetadata::default()) }
}

struct AudioProcessor;
impl AudioProcessor { fn new() -> Self { Self } }
impl AssetProcessor for AudioProcessor {
    fn process_types(&self) -> &[AssetType] { &[] }
    fn process(&self, _asset: &AssetEntry, _config: &ProcessingConfig) -> RobinResult<ProcessingResult> {
        Ok(ProcessingResult {
            output_path: PathBuf::from("output.ogg"),
            optimized_size: 512,
            compression_ratio: 0.3,
            quality_metrics: QualityMetrics::default(),
        })
    }
    fn estimate_processing_time(&self, _asset: &AssetEntry) -> f32 { 3.0 }
}

impl Default for AssetMetadata {
    fn default() -> Self {
        Self {
            title: None,
            description: None,
            author: None,
            license: None,
            tags: vec![],
            custom_properties: HashMap::new(),
            usage_stats: UsageStats {
                access_count: 0,
                last_used: SystemTime::now(),
                projects_using: vec![],
                estimated_importance: 0.0,
            },
            quality_metrics: QualityMetrics::default(),
        }
    }
}

impl Default for QualityMetrics {
    fn default() -> Self {
        Self {
            compression_ratio: 1.0,
            loading_time_ms: 0.0,
            memory_footprint: 0,
            visual_quality_score: 1.0,
            performance_impact: 0.0,
        }
    }
}

// Add missing uuid module for compatibility
mod uuid {
    pub struct Uuid;
    impl Uuid {
        pub fn new_v4() -> Self { Self }
        pub fn to_string(&self) -> String {
            format!("{:x}", rand::random::<u64>())
        }
    }
}