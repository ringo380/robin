use crate::engine::math::Vec2;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};
use serde::{Deserialize, Serialize};

pub mod hot_reload;
pub mod registry;
pub mod watcher;
pub mod pipeline;
pub mod importer;
pub mod database;
pub mod hot_reload_optimized;
pub mod parallel_processor;
pub mod optimized_database;
pub mod optimized_hot_reload;

pub use hot_reload::*;
pub use registry::*;
pub use watcher::*;
pub use pipeline::*;
pub use importer::*;
pub use database::*;
pub use hot_reload_optimized::{
    OptimizedHotReloadSystem, HotReloadConfig, HotReloadMetrics, HotReloadPerformanceReport
};
pub use parallel_processor::{
    ParallelAssetProcessor, ParallelProcessorConfig, ProcessingTask, ProcessingResult,
    TaskPriority, MappedFile, ProcessorStats
};
pub use optimized_database::{
    OptimizedAssetDatabase, OptimizedDatabaseConfig, DatabaseMetrics
};
pub use optimized_hot_reload::{
    OptimizedHotReloadSystem as NewOptimizedHotReloadSystem,
    OptimizedHotReloadConfig, HotReloadMetrics as NewHotReloadMetrics,
    HotReloadPerformanceReport as NewHotReloadPerformanceReport
};

/// Legacy asset trait for compatibility
pub trait Asset {
    fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized;
}

/// Asset types that can be hot-reloaded
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AssetType {
    #[default]
    Texture,
    Audio,
    Config,
    Shader,
    Scene,
    Font,
    Data,
}

/// Asset metadata for tracking and reloading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMetadata {
    pub id: String,
    pub asset_type: AssetType,
    pub file_path: PathBuf,
    pub last_modified: SystemTime,
    pub dependencies: Vec<String>,
    pub load_count: u32,
    pub memory_size: usize,
}

impl AssetMetadata {
    pub fn new(id: String, asset_type: AssetType, file_path: PathBuf) -> Self {
        let last_modified = std::fs::metadata(&file_path)
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);

        Self {
            id,
            asset_type,
            file_path,
            last_modified,
            dependencies: Vec::new(),
            load_count: 0,
            memory_size: 0,
        }
    }

    pub fn needs_reload(&self) -> bool {
        std::fs::metadata(&self.file_path)
            .and_then(|m| m.modified())
            .map(|modified| modified > self.last_modified)
            .unwrap_or(false)
    }

    pub fn update_modified_time(&mut self) {
        self.last_modified = std::fs::metadata(&self.file_path)
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);
    }
}

/// Hot reload event types
#[derive(Debug, Clone)]
pub enum HotReloadEvent {
    AssetModified {
        asset_id: String,
        asset_type: AssetType,
        file_path: PathBuf,
    },
    AssetDeleted {
        asset_id: String,
        file_path: PathBuf,
    },
    AssetCreated {
        file_path: PathBuf,
        asset_type: AssetType,
    },
    ReloadFailed {
        asset_id: String,
        error: String,
    },
    ReloadComplete {
        asset_id: String,
        reload_time: Duration,
    },
}

/// Asset loading configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetConfig {
    pub base_path: PathBuf,
    pub texture_path: PathBuf,
    pub audio_path: PathBuf,
    pub config_path: PathBuf,
    pub scene_path: PathBuf,
    pub watch_enabled: bool,
    pub reload_delay: Duration,
    pub max_file_size: usize,
    pub supported_texture_formats: Vec<String>,
    pub supported_audio_formats: Vec<String>,
}

impl Default for AssetConfig {
    fn default() -> Self {
        Self {
            base_path: PathBuf::from("assets"),
            texture_path: PathBuf::from("assets/textures"),
            audio_path: PathBuf::from("assets/audio"),
            config_path: PathBuf::from("assets/config"),
            scene_path: PathBuf::from("assets/scenes"),
            watch_enabled: true,
            reload_delay: Duration::from_millis(100),
            max_file_size: 50 * 1024 * 1024, // 50MB
            supported_texture_formats: vec![
                "png".to_string(),
                "jpg".to_string(),
                "jpeg".to_string(),
                "gif".to_string(),
                "bmp".to_string(),
                "tga".to_string(),
            ],
            supported_audio_formats: vec![
                "wav".to_string(),
                "mp3".to_string(),
                "ogg".to_string(),
                "flac".to_string(),
            ],
        }
    }
}

/// Asset loading result
pub type AssetResult<T> = Result<T, AssetError>;

/// Asset loading errors
#[derive(Debug, Clone)]
pub enum AssetError {
    FileNotFound(PathBuf),
    LoadFailed(String),
    UnsupportedFormat(String),
    FileTooLarge(usize),
    PermissionDenied(PathBuf),
    WatcherError(String),
    SerializationError(String),
    InvalidPath(String),
}

impl std::fmt::Display for AssetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetError::FileNotFound(path) => write!(f, "Asset file not found: {}", path.display()),
            AssetError::LoadFailed(msg) => write!(f, "Failed to load asset: {}", msg),
            AssetError::UnsupportedFormat(format) => write!(f, "Unsupported asset format: {}", format),
            AssetError::FileTooLarge(size) => write!(f, "Asset file too large: {} bytes", size),
            AssetError::PermissionDenied(path) => write!(f, "Permission denied: {}", path.display()),
            AssetError::WatcherError(msg) => write!(f, "File watcher error: {}", msg),
            AssetError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            AssetError::InvalidPath(msg) => write!(f, "Invalid path: {}", msg),
        }
    }
}

impl std::error::Error for AssetError {}

// From implementations for common error conversions
impl From<std::io::Error> for AssetError {
    fn from(error: std::io::Error) -> Self {
        match error.kind() {
            std::io::ErrorKind::NotFound => AssetError::FileNotFound(PathBuf::from("unknown")),
            std::io::ErrorKind::PermissionDenied => AssetError::PermissionDenied(PathBuf::from("unknown")),
            _ => AssetError::LoadFailed(error.to_string()),
        }
    }
}

impl From<serde_json::Error> for AssetError {
    fn from(error: serde_json::Error) -> Self {
        AssetError::SerializationError(error.to_string())
    }
}

impl From<std::path::StripPrefixError> for AssetError {
    fn from(error: std::path::StripPrefixError) -> Self {
        AssetError::InvalidPath(error.to_string())
    }
}

impl From<toml::de::Error> for AssetError {
    fn from(error: toml::de::Error) -> Self {
        AssetError::SerializationError(error.to_string())
    }
}

impl From<bincode::Error> for AssetError {
    fn from(error: bincode::Error) -> Self {
        AssetError::SerializationError(error.to_string())
    }
}

/// Callback type for asset reload events
pub type ReloadCallback = Arc<dyn Fn(&HotReloadEvent) + Send + Sync>;

/// Asset reloading statistics
#[derive(Debug, Clone, Default)]
pub struct ReloadStats {
    pub total_reloads: u32,
    pub successful_reloads: u32,
    pub failed_reloads: u32,
    pub average_reload_time: Duration,
    pub last_reload_time: Option<SystemTime>,
    pub assets_watched: u32,
}

impl ReloadStats {
    pub fn add_reload(&mut self, success: bool, duration: Duration) {
        self.total_reloads += 1;
        if success {
            self.successful_reloads += 1;
        } else {
            self.failed_reloads += 1;
        }
        
        // Update average reload time
        let total_time = self.average_reload_time.as_nanos() as f64 * (self.total_reloads - 1) as f64;
        let new_average = (total_time + duration.as_nanos() as f64) / self.total_reloads as f64;
        self.average_reload_time = Duration::from_nanos(new_average as u64);
        
        self.last_reload_time = Some(SystemTime::now());
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_reloads == 0 {
            0.0
        } else {
            self.successful_reloads as f64 / self.total_reloads as f64
        }
    }
}

/// Legacy AssetManager for backward compatibility
pub struct AssetManager {
    textures: HashMap<String, crate::engine::graphics::Texture>,
    // Add more asset types as needed
}

impl AssetManager {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
        }
    }

    pub fn load_texture<P: AsRef<Path>>(&mut self, name: &str, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let texture = crate::engine::graphics::Texture::load_from_file(path)?;
        self.textures.insert(name.to_string(), texture);
        Ok(())
    }

    pub fn get_texture(&self, name: &str) -> Option<&crate::engine::graphics::Texture> {
        self.textures.get(name)
    }
}

/// Utility functions for asset management
pub mod utils {
    use super::*;

    /// Get asset type from file extension
    pub fn get_asset_type_from_extension(extension: &str) -> Option<AssetType> {
        match extension.to_lowercase().as_str() {
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "tga" => Some(AssetType::Texture),
            "wav" | "mp3" | "ogg" | "flac" => Some(AssetType::Audio),
            "wgsl" | "glsl" | "hlsl" => Some(AssetType::Shader),
            "json" | "toml" | "yaml" | "yml" => Some(AssetType::Config),
            "scene" => Some(AssetType::Scene),
            "ttf" | "otf" | "woff" | "woff2" => Some(AssetType::Font),
            _ => Some(AssetType::Data),
        }
    }

    /// Generate asset ID from file path
    pub fn generate_asset_id(path: &Path, base_path: &Path) -> String {
        path.strip_prefix(base_path)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/")
            .replace("..", "")
    }

    /// Check if file size is within limits
    pub fn check_file_size(path: &Path, max_size: usize) -> AssetResult<()> {
        let metadata = std::fs::metadata(path)
            .map_err(|_| AssetError::FileNotFound(path.to_path_buf()))?;
        
        let size = metadata.len() as usize;
        if size > max_size {
            return Err(AssetError::FileTooLarge(size));
        }
        
        Ok(())
    }

    /// Create asset directories if they don't exist
    pub fn ensure_asset_directories(config: &AssetConfig) -> AssetResult<()> {
        let dirs = [
            &config.base_path,
            &config.texture_path,
            &config.audio_path,
            &config.config_path,
            &config.scene_path,
        ];

        for dir in &dirs {
            if !dir.exists() {
                std::fs::create_dir_all(dir)
                    .map_err(|e| AssetError::LoadFailed(format!("Failed to create directory {}: {}", dir.display(), e)))?;
                log::info!("Created asset directory: {}", dir.display());
            }
        }

        Ok(())
    }

    /// Validate asset file format
    pub fn validate_asset_format(path: &Path, config: &AssetConfig) -> AssetResult<AssetType> {
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| AssetError::UnsupportedFormat("No file extension".to_string()))?;

        let asset_type = get_asset_type_from_extension(extension)
            .ok_or_else(|| AssetError::UnsupportedFormat(extension.to_string()))?;

        // Check if format is supported
        let supported = match asset_type {
            AssetType::Texture => config.supported_texture_formats.contains(&extension.to_lowercase()),
            AssetType::Audio => config.supported_audio_formats.contains(&extension.to_lowercase()),
            _ => true, // Other formats are generally supported
        };

        if !supported {
            return Err(AssetError::UnsupportedFormat(extension.to_string()));
        }

        Ok(asset_type)
    }

    /// Get relative path for logging and display
    pub fn get_relative_path(path: &Path, base: &Path) -> String {
        path.strip_prefix(base)
            .unwrap_or(path)
            .display()
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_asset_metadata_creation() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.png");
        fs::write(&file_path, b"fake png data").unwrap();

        let metadata = AssetMetadata::new(
            "test_texture".to_string(),
            AssetType::Texture,
            file_path.clone(),
        );

        assert_eq!(metadata.id, "test_texture");
        assert_eq!(metadata.asset_type, AssetType::Texture);
        assert_eq!(metadata.file_path, file_path);
    }

    #[test]
    fn test_asset_type_from_extension() {
        assert_eq!(utils::get_asset_type_from_extension("png"), Some(AssetType::Texture));
        assert_eq!(utils::get_asset_type_from_extension("wav"), Some(AssetType::Audio));
        assert_eq!(utils::get_asset_type_from_extension("json"), Some(AssetType::Config));
        assert_eq!(utils::get_asset_type_from_extension("unknown"), Some(AssetType::Data));
    }

    #[test]
    fn test_asset_id_generation() {
        let base_path = PathBuf::from("/assets");
        let file_path = PathBuf::from("/assets/textures/player.png");
        
        let id = utils::generate_asset_id(&file_path, &base_path);
        assert_eq!(id, "textures/player.png");
    }

    #[test]
    fn test_reload_stats() {
        let mut stats = ReloadStats::default();
        
        stats.add_reload(true, Duration::from_millis(100));
        assert_eq!(stats.successful_reloads, 1);
        assert_eq!(stats.success_rate(), 1.0);
        
        stats.add_reload(false, Duration::from_millis(200));
        assert_eq!(stats.failed_reloads, 1);
        assert_eq!(stats.success_rate(), 0.5);
    }

    #[test]
    fn test_asset_config_default() {
        let config = AssetConfig::default();
        assert_eq!(config.base_path, PathBuf::from("assets"));
        assert!(config.watch_enabled);
        assert!(config.supported_texture_formats.contains(&"png".to_string()));
    }
}