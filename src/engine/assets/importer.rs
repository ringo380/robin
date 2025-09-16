use crate::engine::{
    assets::{AssetType, AssetResult, AssetError, AssetMetadata},
    error::{RobinError, RobinResult},
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    fs,
};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// Asset importing system for bringing external assets into the engine
pub struct AssetImporter {
    importers: HashMap<String, Box<dyn FileImporter + Send + Sync>>,
    config: ImporterConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImporterConfig {
    pub import_settings_dir: PathBuf,
    pub auto_import: bool,
    pub preserve_directory_structure: bool,
    pub overwrite_existing: bool,
    pub generate_previews: bool,
    pub validate_imports: bool,
    pub default_settings: HashMap<AssetType, ImportSettings>,
}

impl Default for ImporterConfig {
    fn default() -> Self {
        let mut default_settings = HashMap::new();
        
        default_settings.insert(AssetType::Texture, ImportSettings::Texture(TextureImportSettings {
            filter_mode: TextureFilterMode::Linear,
            wrap_mode: TextureWrapMode::Repeat,
            compression: TextureCompression::Auto,
            generate_mipmaps: true,
            max_size: 2048,
            format: TextureFormat::Auto,
            alpha_handling: AlphaHandling::Auto,
        }));

        default_settings.insert(AssetType::Audio, ImportSettings::Audio(AudioImportSettings {
            format: AudioFormat::Auto,
            compression_quality: 0.7,
            sample_rate: SampleRate::Auto,
            channels: ChannelConfig::Auto,
            normalize: true,
            trim_silence: false,
            loop_settings: LoopSettings::default(),
        }));

        Self {
            import_settings_dir: PathBuf::from("import_settings"),
            auto_import: true,
            preserve_directory_structure: true,
            overwrite_existing: false,
            generate_previews: true,
            validate_imports: true,
            default_settings,
        }
    }
}

/// Import settings for different asset types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImportSettings {
    Texture(TextureImportSettings),
    Audio(AudioImportSettings),
    Model(ModelImportSettings),
    Animation(AnimationImportSettings),
    Font(FontImportSettings),
    Config(ConfigImportSettings),
}

impl Default for ImportSettings {
    fn default() -> Self {
        ImportSettings::Texture(TextureImportSettings {
            filter_mode: TextureFilterMode::Linear,
            wrap_mode: TextureWrapMode::Repeat,
            compression: TextureCompression::None,
            generate_mipmaps: true,
            max_size: 4096,
            format: TextureFormat::RGBA8,
            alpha_handling: AlphaHandling::Preserve,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureImportSettings {
    pub filter_mode: TextureFilterMode,
    pub wrap_mode: TextureWrapMode,
    pub compression: TextureCompression,
    pub generate_mipmaps: bool,
    pub max_size: u32,
    pub format: TextureFormat,
    pub alpha_handling: AlphaHandling,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextureFilterMode {
    Nearest,
    Linear,
    Bilinear,
    Trilinear,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextureWrapMode {
    Repeat,
    Clamp,
    Mirror,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextureCompression {
    None,
    Auto,
    DXT1,
    DXT5,
    BC7,
    ASTC,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextureFormat {
    Auto,
    RGBA8,
    RGB8,
    RGBA16,
    R8,
    RG8,
    SRGBA8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlphaHandling {
    Auto,
    Preserve,
    Remove,
    Premultiply,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioImportSettings {
    pub format: AudioFormat,
    pub compression_quality: f32,
    pub sample_rate: SampleRate,
    pub channels: ChannelConfig,
    pub normalize: bool,
    pub trim_silence: bool,
    pub loop_settings: LoopSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioFormat {
    Auto,
    WAV,
    OGG,
    MP3,
    FLAC,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SampleRate {
    Auto,
    Rate8000,
    Rate16000,
    Rate22050,
    Rate44100,
    Rate48000,
    Rate96000,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChannelConfig {
    Auto,
    Mono,
    Stereo,
    Surround5_1,
    Surround7_1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopSettings {
    pub enabled: bool,
    pub start_sample: Option<u64>,
    pub end_sample: Option<u64>,
}

impl Default for LoopSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            start_sample: None,
            end_sample: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelImportSettings {
    pub scale: f32,
    pub generate_normals: bool,
    pub generate_tangents: bool,
    pub optimize_meshes: bool,
    pub weld_vertices: bool,
    pub import_materials: bool,
    pub import_animations: bool,
    pub animation_settings: AnimationImportSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationImportSettings {
    pub compression: AnimationCompression,
    pub sample_rate: f32,
    pub remove_redundant_keys: bool,
    pub optimize_curves: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnimationCompression {
    None,
    Lossy,
    Keyframe,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontImportSettings {
    pub size_range: (u32, u32),
    pub charset: CharacterSet,
    pub anti_aliasing: bool,
    pub generate_sdf: bool,
    pub padding: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CharacterSet {
    ASCII,
    Latin1,
    Unicode,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigImportSettings {
    pub validate_schema: bool,
    pub schema_path: Option<PathBuf>,
    pub minify: bool,
    pub format: ConfigFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigFormat {
    Auto,
    JSON,
    YAML,
    TOML,
    Binary,
}

/// File importer trait
pub trait FileImporter {
    /// Check if this importer supports the given file
    fn supports_file(&self, path: &Path) -> bool;
    
    /// Import a file with the given settings
    fn import_file(&self, path: &Path, settings: &ImportSettings) -> AssetResult<ImportResult>;
    
    /// Get the default import settings for this importer
    fn default_settings(&self) -> ImportSettings;
    
    /// Validate import settings
    fn validate_settings(&self, settings: &ImportSettings) -> AssetResult<()>;
    
    /// Get supported file extensions
    fn supported_extensions(&self) -> Vec<&'static str>;
}

/// Import result
#[derive(Debug, Clone, Default)]
pub struct ImportResult {
    pub imported_path: PathBuf,
    pub asset_type: AssetType,
    pub metadata: ImportedAssetMetadata,
    pub dependencies: Vec<PathBuf>,
    pub warnings: Vec<String>,
    pub preview_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImportedAssetMetadata {
    pub source_path: PathBuf,
    pub imported_path: PathBuf,
    pub import_time: DateTime<Utc>,
    pub source_file_size: u64,
    pub imported_file_size: u64,
    pub importer_version: String,
    pub import_settings: ImportSettings,
    pub source_modification_time: DateTime<Utc>,
    pub checksum: String,
}

impl AssetImporter {
    /// Create a new asset importer
    pub fn new(config: ImporterConfig) -> Self {
        let mut importer = Self {
            importers: HashMap::new(),
            config,
        };
        
        // Register default importers
        importer.register_default_importers();
        importer
    }

    /// Register a custom file importer
    pub fn register_importer<I>(&mut self, importer: I) -> RobinResult<()>
    where
        I: FileImporter + Clone + Send + Sync + 'static,
    {
        for ext in importer.supported_extensions() {
            self.importers.insert(ext.to_string(), Box::new(importer.clone()));
        }
        Ok(())
    }

    /// Import a single file
    pub fn import_file<P: AsRef<Path>>(&self, source_path: P, dest_dir: P, settings: Option<ImportSettings>) -> AssetResult<ImportResult> {
        let source_path = source_path.as_ref();
        let dest_dir = dest_dir.as_ref();

        log::info!("Importing asset: {} -> {}", source_path.display(), dest_dir.display());

        // Find appropriate importer
        let extension = source_path.extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| AssetError::UnsupportedFormat("No file extension".to_string()))?;

        let importer = self.importers.get(extension)
            .ok_or_else(|| AssetError::UnsupportedFormat(format!("No importer for extension: {}", extension)))?;

        // Use provided settings or default
        let import_settings = settings.unwrap_or_else(|| importer.default_settings());

        // Validate settings
        importer.validate_settings(&import_settings)?;

        // Create destination directory
        fs::create_dir_all(dest_dir)?;

        // Perform import
        let result = importer.import_file(source_path, &import_settings)?;

        // Save import metadata
        self.save_import_metadata(&result)?;

        log::info!("Successfully imported: {} -> {}", 
            source_path.display(), result.imported_path.display());

        Ok(result)
    }

    /// Import all files from a directory
    pub fn import_directory<P: AsRef<Path>>(&self, source_dir: P, dest_dir: P, recursive: bool) -> AssetResult<Vec<ImportResult>> {
        let source_dir = source_dir.as_ref();
        let dest_dir = dest_dir.as_ref();

        log::info!("Importing directory: {} -> {} (recursive: {})", 
            source_dir.display(), dest_dir.display(), recursive);

        let mut results = Vec::new();
        let mut files_to_import = Vec::new();

        // Collect files to import
        self.collect_importable_files(source_dir, recursive, &mut files_to_import)?;

        let total_files = files_to_import.len();
        log::info!("Found {} files to import", total_files);

        // Import each file
        for source_file in files_to_import {
            let relative_path = source_file.strip_prefix(source_dir)?;
            let dest_file_dir = if self.config.preserve_directory_structure {
                dest_dir.join(relative_path.parent().unwrap_or(Path::new("")))
            } else {
                dest_dir.to_path_buf()
            };

            match self.import_file(&source_file, &dest_file_dir, None) {
                Ok(result) => results.push(result),
                Err(e) => {
                    log::warn!("Failed to import {}: {}", source_file.display(), e);
                    // Continue with other files
                }
            }
        }

        log::info!("Directory import completed: {}/{} files imported successfully", 
            results.len(), total_files);

        Ok(results)
    }

    /// Re-import assets that have changed
    pub fn reimport_changed_assets<P: AsRef<Path>>(&self, watch_dir: P) -> AssetResult<Vec<ImportResult>> {
        let watch_dir = watch_dir.as_ref();
        let mut results = Vec::new();

        // Find assets that need re-importing
        let changed_assets = self.find_changed_assets(watch_dir)?;

        log::info!("Found {} changed assets to re-import", changed_assets.len());

        for (source_path, metadata) in changed_assets {
            match self.import_file(&source_path, &metadata.imported_path.parent().unwrap().to_path_buf(), Some(metadata.import_settings)) {
                Ok(result) => {
                    log::info!("Re-imported: {}", source_path.display());
                    results.push(result);
                }
                Err(e) => {
                    log::error!("Failed to re-import {}: {}", source_path.display(), e);
                }
            }
        }

        Ok(results)
    }

    /// Get import settings for a file (load from disk or use defaults)
    pub fn get_import_settings<P: AsRef<Path>>(&self, path: P) -> AssetResult<ImportSettings> {
        let path = path.as_ref();
        
        // Try to load saved settings
        let settings_path = self.get_settings_path(path);
        if settings_path.exists() {
            let settings_data = fs::read_to_string(&settings_path)?;
            let settings: ImportSettings = serde_json::from_str(&settings_data)
                .map_err(|e| AssetError::SerializationError(e.to_string()))?;
            return Ok(settings);
        }

        // Use default settings based on asset type
        let asset_type = self.detect_asset_type(path)?;
        self.config.default_settings.get(&asset_type)
            .cloned()
            .ok_or_else(|| AssetError::UnsupportedFormat(format!("No default settings for asset type: {:?}", asset_type)))
    }

    /// Save import settings for a file
    pub fn save_import_settings<P: AsRef<Path>>(&self, path: P, settings: &ImportSettings) -> AssetResult<()> {
        let settings_path = self.get_settings_path(path.as_ref());
        
        if let Some(parent) = settings_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let settings_json = serde_json::to_string_pretty(settings)
            .map_err(|e| AssetError::SerializationError(e.to_string()))?;
        
        fs::write(&settings_path, settings_json)?;
        
        log::debug!("Saved import settings for: {}", path.as_ref().display());
        Ok(())
    }

    // Private implementation methods
    fn register_default_importers(&mut self) {
        // Register built-in importers
        let texture_importer = TextureImporter::new();
        let audio_importer = AudioImporter::new();
        let config_importer = ConfigImporter::new();

        self.importers.insert("png".to_string(), Box::new(texture_importer.clone()));
        self.importers.insert("jpg".to_string(), Box::new(texture_importer.clone()));
        self.importers.insert("jpeg".to_string(), Box::new(texture_importer.clone()));
        self.importers.insert("gif".to_string(), Box::new(texture_importer.clone()));
        self.importers.insert("bmp".to_string(), Box::new(texture_importer.clone()));
        self.importers.insert("tga".to_string(), Box::new(texture_importer));

        self.importers.insert("wav".to_string(), Box::new(audio_importer.clone()));
        self.importers.insert("mp3".to_string(), Box::new(audio_importer.clone()));
        self.importers.insert("ogg".to_string(), Box::new(audio_importer.clone()));
        self.importers.insert("flac".to_string(), Box::new(audio_importer));

        self.importers.insert("json".to_string(), Box::new(config_importer.clone()));
        self.importers.insert("yaml".to_string(), Box::new(config_importer.clone()));
        self.importers.insert("yml".to_string(), Box::new(config_importer.clone()));
        self.importers.insert("toml".to_string(), Box::new(config_importer));
    }

    fn collect_importable_files(&self, dir: &Path, recursive: bool, files: &mut Vec<PathBuf>) -> AssetResult<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if self.importers.contains_key(ext) {
                        files.push(path);
                    }
                }
            } else if path.is_dir() && recursive {
                self.collect_importable_files(&path, recursive, files)?;
            }
        }
        Ok(())
    }

    fn find_changed_assets(&self, _watch_dir: &Path) -> AssetResult<Vec<(PathBuf, ImportedAssetMetadata)>> {
        // Implementation would check modification times against saved metadata
        Ok(Vec::new()) // Placeholder
    }

    fn detect_asset_type(&self, path: &Path) -> AssetResult<AssetType> {
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| AssetError::UnsupportedFormat("No file extension".to_string()))?;

        match ext.to_lowercase().as_str() {
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "tga" => Ok(AssetType::Texture),
            "wav" | "mp3" | "ogg" | "flac" => Ok(AssetType::Audio),
            "json" | "yaml" | "yml" | "toml" => Ok(AssetType::Config),
            "ttf" | "otf" => Ok(AssetType::Font),
            _ => Ok(AssetType::Data),
        }
    }

    fn get_settings_path(&self, asset_path: &Path) -> PathBuf {
        let mut settings_path = self.config.import_settings_dir.join(asset_path);
        settings_path.set_extension("import");
        settings_path
    }

    fn save_import_metadata(&self, _result: &ImportResult) -> AssetResult<()> {
        // Save metadata for tracking imports
        Ok(()) // Placeholder
    }

    pub fn import_asset(&self, _asset_path: &str) -> AssetResult<ImportResult> {
        // Import a single asset
        Ok(ImportResult::default())
    }

    pub fn validate_all_assets(&self) -> AssetResult<()> {
        // Validate all imported assets
        Ok(())
    }
}

impl ImportResult {
    pub fn display(&self) -> String {
        format!("Imported: {}", self.imported_path.display())
    }

    pub fn to_str(&self) -> String {
        self.display()
    }
}

// Default importers
#[derive(Clone)]
struct TextureImporter;

impl TextureImporter {
    fn new() -> Self { Self }
}

impl FileImporter for TextureImporter {
    fn supports_file(&self, path: &Path) -> bool {
        matches!(path.extension().and_then(|e| e.to_str()),
            Some("png") | Some("jpg") | Some("jpeg") | Some("gif") | Some("bmp") | Some("tga"))
    }

    fn import_file(&self, path: &Path, _settings: &ImportSettings) -> AssetResult<ImportResult> {
        // Basic texture import implementation
        let data = fs::read(path)?;
        let imported_path = path.with_extension("texture");
        fs::write(&imported_path, data)?;

        Ok(ImportResult {
            imported_path: imported_path.clone(),
            asset_type: AssetType::Texture,
            metadata: ImportedAssetMetadata {
                source_path: path.to_path_buf(),
                imported_path,
                import_time: Utc::now(),
                source_file_size: fs::metadata(path)?.len(),
                imported_file_size: 0,
                importer_version: "1.0.0".to_string(),
                import_settings: _settings.clone(),
                source_modification_time: Utc::now(),
                checksum: "".to_string(),
            },
            dependencies: Vec::new(),
            warnings: Vec::new(),
            preview_path: None,
        })
    }

    fn default_settings(&self) -> ImportSettings {
        ImportSettings::Texture(TextureImportSettings {
            filter_mode: TextureFilterMode::Linear,
            wrap_mode: TextureWrapMode::Repeat,
            compression: TextureCompression::Auto,
            generate_mipmaps: true,
            max_size: 2048,
            format: TextureFormat::Auto,
            alpha_handling: AlphaHandling::Auto,
        })
    }

    fn validate_settings(&self, _settings: &ImportSettings) -> AssetResult<()> {
        // Validate texture import settings
        Ok(())
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["png", "jpg", "jpeg", "gif", "bmp", "tga"]
    }
}

#[derive(Clone)]
struct AudioImporter;

impl AudioImporter {
    fn new() -> Self { Self }
}

impl FileImporter for AudioImporter {
    fn supports_file(&self, path: &Path) -> bool {
        matches!(path.extension().and_then(|e| e.to_str()),
            Some("wav") | Some("mp3") | Some("ogg") | Some("flac"))
    }

    fn import_file(&self, path: &Path, _settings: &ImportSettings) -> AssetResult<ImportResult> {
        let data = fs::read(path)?;
        let imported_path = path.with_extension("audio");
        fs::write(&imported_path, data)?;

        Ok(ImportResult {
            imported_path: imported_path.clone(),
            asset_type: AssetType::Audio,
            metadata: ImportedAssetMetadata {
                source_path: path.to_path_buf(),
                imported_path,
                import_time: Utc::now(),
                source_file_size: fs::metadata(path)?.len(),
                imported_file_size: 0,
                importer_version: "1.0.0".to_string(),
                import_settings: _settings.clone(),
                source_modification_time: Utc::now(),
                checksum: "".to_string(),
            },
            dependencies: Vec::new(),
            warnings: Vec::new(),
            preview_path: None,
        })
    }

    fn default_settings(&self) -> ImportSettings {
        ImportSettings::Audio(AudioImportSettings {
            format: AudioFormat::Auto,
            compression_quality: 0.7,
            sample_rate: SampleRate::Auto,
            channels: ChannelConfig::Auto,
            normalize: true,
            trim_silence: false,
            loop_settings: LoopSettings::default(),
        })
    }

    fn validate_settings(&self, _settings: &ImportSettings) -> AssetResult<()> {
        Ok(())
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["wav", "mp3", "ogg", "flac"]
    }
}

#[derive(Clone)]
struct ConfigImporter;

impl ConfigImporter {
    fn new() -> Self { Self }
}

impl FileImporter for ConfigImporter {
    fn supports_file(&self, path: &Path) -> bool {
        matches!(path.extension().and_then(|e| e.to_str()),
            Some("json") | Some("yaml") | Some("yml") | Some("toml"))
    }

    fn import_file(&self, path: &Path, _settings: &ImportSettings) -> AssetResult<ImportResult> {
        let data = fs::read(path)?;
        let imported_path = path.with_extension("config");
        fs::write(&imported_path, data)?;

        Ok(ImportResult {
            imported_path: imported_path.clone(),
            asset_type: AssetType::Config,
            metadata: ImportedAssetMetadata {
                source_path: path.to_path_buf(),
                imported_path,
                import_time: Utc::now(),
                source_file_size: fs::metadata(path)?.len(),
                imported_file_size: 0,
                importer_version: "1.0.0".to_string(),
                import_settings: _settings.clone(),
                source_modification_time: Utc::now(),
                checksum: "".to_string(),
            },
            dependencies: Vec::new(),
            warnings: Vec::new(),
            preview_path: None,
        })
    }

    fn default_settings(&self) -> ImportSettings {
        ImportSettings::Config(ConfigImportSettings {
            validate_schema: true,
            schema_path: None,
            minify: false,
            format: ConfigFormat::Auto,
        })
    }

    fn validate_settings(&self, _settings: &ImportSettings) -> AssetResult<()> {
        Ok(())
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["json", "yaml", "yml", "toml"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_importer_creation() {
        let config = ImporterConfig::default();
        let importer = AssetImporter::new(config);
        
        // Should have default importers registered
        assert!(importer.importers.contains_key("png"));
        assert!(importer.importers.contains_key("wav"));
        assert!(importer.importers.contains_key("json"));
    }

    #[test]
    fn test_asset_type_detection() {
        let config = ImporterConfig::default();
        let importer = AssetImporter::new(config);

        assert_eq!(importer.detect_asset_type(Path::new("test.png")).unwrap(), AssetType::Texture);
        assert_eq!(importer.detect_asset_type(Path::new("test.wav")).unwrap(), AssetType::Audio);
        assert_eq!(importer.detect_asset_type(Path::new("test.json")).unwrap(), AssetType::Config);
    }

    #[test]
    fn test_import_settings_serialization() {
        let settings = ImportSettings::Texture(TextureImportSettings {
            filter_mode: TextureFilterMode::Linear,
            wrap_mode: TextureWrapMode::Repeat,
            compression: TextureCompression::Auto,
            generate_mipmaps: true,
            max_size: 2048,
            format: TextureFormat::Auto,
            alpha_handling: AlphaHandling::Auto,
        });

        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: ImportSettings = serde_json::from_str(&json).unwrap();

        match deserialized {
            ImportSettings::Texture(texture_settings) => {
                assert_eq!(texture_settings.max_size, 2048);
                assert!(texture_settings.generate_mipmaps);
            }
            _ => panic!("Wrong settings type"),
        }
    }
}