use crate::engine::{
    scene::{Scene, SerializableScene, SceneSerializer},
    error::{RobinError, RobinResult},
    physics::PhysicsWorld,
    ui::UIManager,
    animation::AnimationManager,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub mod game_state;
pub mod save_manager;
pub mod profile;

pub use game_state::*;
pub use save_manager::*;
pub use profile::*;

/// Save system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveSystemConfig {
    /// Base directory for save files
    pub save_directory: PathBuf,
    /// Maximum number of save slots
    pub max_slots: usize,
    /// Enable auto-save feature
    pub auto_save_enabled: bool,
    /// Auto-save interval
    pub auto_save_interval: Duration,
    /// Maximum number of backups per save
    pub max_backups: usize,
    /// Compress save files
    pub enable_compression: bool,
    /// Encrypt save files
    pub enable_encryption: bool,
    /// Save file extension
    pub save_extension: String,
}

impl Default for SaveSystemConfig {
    fn default() -> Self {
        Self {
            save_directory: Self::get_default_save_directory(),
            max_slots: 10,
            auto_save_enabled: true,
            auto_save_interval: Duration::from_secs(300), // 5 minutes
            max_backups: 3,
            enable_compression: true,
            enable_encryption: false,
            save_extension: "robinsave".to_string(),
        }
    }
}

impl SaveSystemConfig {
    /// Get the default save directory based on the platform
    fn get_default_save_directory() -> PathBuf {
        if let Some(dirs) = directories::UserDirs::new() {
            dirs.document_dir()
                .map(|d| d.join("RobinEngine").join("Saves"))
                .unwrap_or_else(|| PathBuf::from("saves"))
        } else {
            PathBuf::from("saves")
        }
    }

    /// Create the save directory if it doesn't exist
    pub fn ensure_save_directory(&self) -> RobinResult<()> {
        if !self.save_directory.exists() {
            fs::create_dir_all(&self.save_directory).map_err(|e| {
                RobinError::FileAccessDenied(self.save_directory.clone())
            })?;
        }
        Ok(())
    }
}

/// Save file metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveMetadata {
    /// Save slot number
    pub slot: usize,
    /// Save file version
    pub version: String,
    /// Save name/description
    pub name: String,
    /// Timestamp when the save was created
    pub created_at: SystemTime,
    /// Timestamp when the save was last modified
    pub modified_at: SystemTime,
    /// Play time in seconds
    pub play_time: u64,
    /// Game progress percentage (0-100)
    pub progress: f32,
    /// Current level/scene name
    pub current_scene: String,
    /// Player level
    pub player_level: u32,
    /// Custom metadata (game-specific)
    pub custom_data: HashMap<String, String>,
    /// Thumbnail path (optional)
    pub thumbnail: Option<String>,
}

impl SaveMetadata {
    pub fn new(slot: usize, name: String) -> Self {
        let now = SystemTime::now();
        Self {
            slot,
            version: env!("CARGO_PKG_VERSION").to_string(),
            name,
            created_at: now,
            modified_at: now,
            play_time: 0,
            progress: 0.0,
            current_scene: String::new(),
            player_level: 1,
            custom_data: HashMap::new(),
            thumbnail: None,
        }
    }

    /// Update the modified timestamp
    pub fn update_modified(&mut self) {
        self.modified_at = SystemTime::now();
    }

    /// Get a human-readable timestamp
    pub fn get_timestamp_string(&self) -> String {
        let duration = self.modified_at
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        
        // Convert to a readable format
        let seconds = duration.as_secs();
        let datetime = chrono::DateTime::<chrono::Utc>::from_timestamp(seconds as i64, 0)
            .unwrap_or_default();
        
        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    /// Get formatted play time
    pub fn get_play_time_string(&self) -> String {
        let hours = self.play_time / 3600;
        let minutes = (self.play_time % 3600) / 60;
        let seconds = self.play_time % 60;
        
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }
}

/// Result type for save operations
pub type SaveResult<T> = Result<T, SaveError>;

/// Save system errors
#[derive(Debug, Clone)]
pub enum SaveError {
    /// Save slot not found
    SlotNotFound(usize),
    /// Save file corrupted
    CorruptedSave(String),
    /// IO error
    IoError(String),
    /// Serialization error
    SerializationError(String),
    /// Compression error
    CompressionError(String),
    /// Encryption error
    EncryptionError(String),
    /// Invalid save version
    VersionMismatch { expected: String, found: String },
    /// Save slot is full
    SlotFull(usize),
    /// Permission denied
    PermissionDenied(PathBuf),
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveError::SlotNotFound(slot) => write!(f, "Save slot {} not found", slot),
            SaveError::CorruptedSave(msg) => write!(f, "Save file corrupted: {}", msg),
            SaveError::IoError(msg) => write!(f, "IO error: {}", msg),
            SaveError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            SaveError::CompressionError(msg) => write!(f, "Compression error: {}", msg),
            SaveError::EncryptionError(msg) => write!(f, "Encryption error: {}", msg),
            SaveError::VersionMismatch { expected, found } => {
                write!(f, "Save version mismatch: expected {}, found {}", expected, found)
            }
            SaveError::SlotFull(slot) => write!(f, "Save slot {} is full", slot),
            SaveError::PermissionDenied(path) => {
                write!(f, "Permission denied: {}", path.display())
            }
        }
    }
}

impl std::error::Error for SaveError {}

/// Interface for saveable game components
pub trait Saveable {
    /// Save the component state
    fn save_state(&self) -> SaveResult<serde_json::Value>;
    
    /// Load the component state
    fn load_state(&mut self, state: &serde_json::Value) -> SaveResult<()>;
    
    /// Get the component identifier for save/load
    fn get_save_id(&self) -> String;
}

/// Auto-save manager
// TODO: Implement comprehensive auto-save functionality
pub struct AutoSaveManager {
    config: SaveSystemConfig,
    last_save_time: SystemTime,
    save_manager: Box<SaveManager>, // Box to break recursion
    current_slot: Option<usize>,
}

impl AutoSaveManager {
    pub fn new(config: SaveSystemConfig) -> Self {
        Self {
            last_save_time: SystemTime::now(),
            save_manager: Box::new(SaveManager::new(config.clone())),
            config,
            current_slot: None,
        }
    }

    /// Set the current save slot for auto-saving
    pub fn set_slot(&mut self, slot: usize) {
        self.current_slot = Some(slot);
    }

    /// Check if auto-save should trigger
    pub fn should_auto_save(&self) -> bool {
        if !self.config.auto_save_enabled || self.current_slot.is_none() {
            return false;
        }

        let elapsed = SystemTime::now()
            .duration_since(self.last_save_time)
            .unwrap_or(Duration::ZERO);

        elapsed >= self.config.auto_save_interval
    }

    /// Perform auto-save
    pub fn auto_save(&mut self, game_state: &GameState) -> SaveResult<()> {
        if let Some(slot) = self.current_slot {
            self.save_manager.save_game(slot, game_state)?;
            self.last_save_time = SystemTime::now();
            log::info!("Auto-save completed to slot {}", slot);
        }
        Ok(())
    }

    /// Force an immediate auto-save
    pub fn force_auto_save(&mut self, game_state: &GameState) -> SaveResult<()> {
        self.auto_save(game_state)
    }
}

/// Cloud save support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudSaveConfig {
    /// Enable cloud saves
    pub enabled: bool,
    /// Cloud service provider
    pub provider: CloudProvider,
    /// Sync interval
    pub sync_interval: Duration,
    /// Maximum cloud storage per user (in bytes)
    pub max_storage: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CloudProvider {
    Steam,
    Epic,
    Custom(String),
}

/// Save file compression utilities
pub mod compression {
    use super::*;
    use flate2::{write::GzEncoder, read::GzDecoder, Compression};

    /// Compress save data
    pub fn compress_data(data: &[u8]) -> SaveResult<Vec<u8>> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data).map_err(|e| {
            SaveError::CompressionError(e.to_string())
        })?;
        encoder.finish().map_err(|e| {
            SaveError::CompressionError(e.to_string())
        })
    }

    /// Decompress save data
    pub fn decompress_data(data: &[u8]) -> SaveResult<Vec<u8>> {
        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).map_err(|e| {
            SaveError::CompressionError(e.to_string())
        })?;
        Ok(decompressed)
    }
}

/// Save file encryption utilities (placeholder - requires crypto library)
pub mod encryption {
    use super::*;

    /// Encrypt save data (placeholder)
    pub fn encrypt_data(data: &[u8], _key: &[u8]) -> SaveResult<Vec<u8>> {
        // TODO: Implement actual encryption using a crypto library
        Ok(data.to_vec())
    }

    /// Decrypt save data (placeholder)
    pub fn decrypt_data(data: &[u8], _key: &[u8]) -> SaveResult<Vec<u8>> {
        // TODO: Implement actual decryption using a crypto library
        Ok(data.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_metadata() {
        let mut metadata = SaveMetadata::new(1, "Test Save".to_string());
        metadata.play_time = 3661; // 1 hour, 1 minute, 1 second
        
        assert_eq!(metadata.slot, 1);
        assert_eq!(metadata.name, "Test Save");
        assert_eq!(metadata.get_play_time_string(), "01:01:01");
    }

    #[test]
    fn test_save_config_default() {
        let config = SaveSystemConfig::default();
        
        assert_eq!(config.max_slots, 10);
        assert!(config.auto_save_enabled);
        assert_eq!(config.auto_save_interval, Duration::from_secs(300));
    }

    #[test]
    fn test_compression() {
        let original = b"This is test data for compression";
        let compressed = compression::compress_data(original).unwrap();
        let decompressed = compression::decompress_data(&compressed).unwrap();
        
        assert_eq!(original, &decompressed[..]);
    }
}