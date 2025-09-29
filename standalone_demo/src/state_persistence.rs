/*!
 * Robin Engine - Crash-Safe State Persistence System
 *
 * Provides atomic save operations and crash recovery:
 * - Atomic file operations using temp files + rename
 * - Auto-save with debouncing to prevent excessive I/O
 * - Crash detection and state restoration
 * - State validation and integrity checking
 * - Incremental save optimization
 */

use crate::error::{RobinError, RobinResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};
use log::{info, warn, error, debug};

/// Application state snapshot for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    /// Timestamp when state was saved
    pub timestamp: u64,
    /// Application version for compatibility checking
    pub version: String,
    /// Camera position and orientation
    pub camera_state: CameraState,
    /// World state including voxel data
    pub world_state: WorldState,
    /// Current build mode and selected voxel type
    pub build_state: BuildState,
    /// Performance and quality settings
    pub performance_state: PerformanceState,
    /// Checksum for integrity validation
    pub checksum: u64,
}

/// Camera position and view state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraState {
    pub position: [f32; 3],
    pub rotation: [f32; 2], // yaw, pitch
    pub move_speed: f32,
    pub mouse_sensitivity: f32,
}

/// World state including voxel data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    /// Serialized voxel data (simplified for demo)
    pub voxel_data: Vec<VoxelData>,
    /// World generation seed
    pub world_seed: u64,
    /// Last modified timestamp
    pub last_modified: u64,
}

/// Individual voxel data for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoxelData {
    pub position: [i32; 3],
    pub voxel_type: u8, // Simplified voxel type as u8
}

/// Build mode and tool state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildState {
    pub current_build_mode: u8, // Simplified build mode as u8
    pub current_voxel_type: u8,
    pub show_hud: bool,
    pub show_mode_selector: bool,
}

/// Performance and degradation state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceState {
    pub performance_mode: u8, // PerformanceMode as u8
    pub adaptive_quality_enabled: bool,
    pub degradation_active: bool,
    pub memory_pressure_level: f32,
}

/// State persistence manager with atomic operations
pub struct StatePersistenceManager {
    /// Base directory for state files
    state_dir: PathBuf,
    /// Main state file path
    state_file: PathBuf,
    /// Backup state file path
    backup_file: PathBuf,
    /// Lock file for atomic operations
    lock_file: PathBuf,
    /// Last save timestamp for debouncing
    last_save: Option<Instant>,
    /// Save debounce duration
    save_debounce: Duration,
    /// Whether the last shutdown was clean
    clean_shutdown: bool,
    /// Current application version
    app_version: String,
}

/// Save operation result
#[derive(Debug, Clone)]
pub enum SaveResult {
    /// Save completed successfully
    Success,
    /// Save skipped due to debouncing
    Debounced,
    /// Save failed with error
    Failed(String),
}

impl StatePersistenceManager {
    /// Create a new state persistence manager
    pub fn new(state_dir: impl AsRef<Path>) -> RobinResult<Self> {
        let state_dir = state_dir.as_ref().to_path_buf();

        // Create state directory if it doesn't exist
        if !state_dir.exists() {
            fs::create_dir_all(&state_dir).map_err(|e| {
                RobinError::FileSystem {
                    message: format!("Failed to create state directory: {}", e),
                    path: Some(state_dir.to_string_lossy().to_string()),
                    operation: "create_directory".to_string(),
                }
            })?;
        }

        let state_file = state_dir.join("state.json");
        let backup_file = state_dir.join("state.backup.json");
        let lock_file = state_dir.join("state.lock");

        let manager = Self {
            state_dir,
            state_file,
            backup_file,
            lock_file,
            last_save: None,
            save_debounce: Duration::from_secs(5), // 5 second debounce
            clean_shutdown: false,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
        };

        // Check for crash on startup
        manager.check_for_crash()?;

        info!("🏪 State persistence manager initialized at: {}",
              manager.state_dir.display());

        Ok(manager)
    }

    /// Check if the last shutdown was due to a crash
    fn check_for_crash(&self) -> RobinResult<()> {
        if self.lock_file.exists() {
            warn!("🚨 Lock file exists - detected unclean shutdown from previous session");

            // Try to recover from backup if main state is corrupted
            if self.backup_file.exists() && self.is_main_state_corrupted()? {
                warn!("🔄 Main state file corrupted, attempting recovery from backup");
                self.restore_from_backup()?;
            }

            // Remove the lock file
            if let Err(e) = fs::remove_file(&self.lock_file) {
                warn!("⚠️ Failed to remove lock file: {}", e);
            }
        } else {
            debug!("✅ Clean shutdown detected from previous session");
        }

        Ok(())
    }

    /// Check if main state file is corrupted
    fn is_main_state_corrupted(&self) -> RobinResult<bool> {
        if !self.state_file.exists() {
            return Ok(true);
        }

        match self.load_state_from_file(&self.state_file) {
            Ok(_) => Ok(false),
            Err(_) => {
                warn!("🔍 Main state file failed validation check");
                Ok(true)
            }
        }
    }

    /// Restore state from backup file
    fn restore_from_backup(&self) -> RobinResult<()> {
        if !self.backup_file.exists() {
            return Err(RobinError::FileSystem {
                message: "No backup file available for recovery".to_string(),
                path: Some(self.backup_file.to_string_lossy().to_string()),
                operation: "restore_backup".to_string(),
            });
        }

        fs::copy(&self.backup_file, &self.state_file).map_err(|e| {
            RobinError::FileSystem {
                message: format!("Failed to restore from backup: {}", e),
                path: Some(self.state_file.to_string_lossy().to_string()),
                operation: "restore_backup".to_string(),
            }
        })?;

        info!("🔄 Successfully restored state from backup");
        Ok(())
    }

    /// Create application lock to detect crashes
    pub fn create_lock(&self) -> RobinResult<()> {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        fs::write(&self.lock_file, timestamp.to_string()).map_err(|e| {
            RobinError::FileSystem {
                message: format!("Failed to create lock file: {}", e),
                path: Some(self.lock_file.to_string_lossy().to_string()),
                operation: "create_lock".to_string(),
            }
        })?;

        debug!("🔒 Application lock created");
        Ok(())
    }

    /// Remove application lock on clean shutdown
    pub fn remove_lock(&self) -> RobinResult<()> {
        if self.lock_file.exists() {
            fs::remove_file(&self.lock_file).map_err(|e| {
                RobinError::FileSystem {
                    message: format!("Failed to remove lock file: {}", e),
                    path: Some(self.lock_file.to_string_lossy().to_string()),
                    operation: "remove_lock".to_string(),
                }
            })?;
            debug!("🔓 Application lock removed");
        }
        Ok(())
    }

    /// Save state with debouncing and atomic operations
    pub fn save_state(&mut self, state: &StateSnapshot) -> SaveResult {
        // Check debouncing
        if let Some(last_save) = self.last_save {
            if last_save.elapsed() < self.save_debounce {
                return SaveResult::Debounced;
            }
        }

        match self.save_state_atomic(state) {
            Ok(()) => {
                self.last_save = Some(Instant::now());
                SaveResult::Success
            }
            Err(error) => {
                error!("💾 Failed to save state: {}", error);
                SaveResult::Failed(error.to_string())
            }
        }
    }

    /// Force save state immediately (bypasses debouncing)
    pub fn force_save_state(&mut self, state: &StateSnapshot) -> RobinResult<()> {
        self.save_state_atomic(state)?;
        self.last_save = Some(Instant::now());
        Ok(())
    }

    /// Atomic state save operation using temp file + rename
    fn save_state_atomic(&self, state: &StateSnapshot) -> RobinResult<()> {
        let temp_file = self.state_file.with_extension("tmp");

        // Create state with checksum
        let mut state_with_checksum = state.clone();
        state_with_checksum.checksum = self.calculate_checksum(state);

        // Serialize to JSON
        let json_data = serde_json::to_string_pretty(&state_with_checksum).map_err(|e| {
            RobinError::FileSystem {
                message: format!("Failed to serialize state: {}", e),
                path: Some(self.state_file.to_string_lossy().to_string()),
                operation: "serialize_state".to_string(),
            }
        })?;

        // Write to temporary file first
        fs::write(&temp_file, &json_data).map_err(|e| {
            RobinError::FileSystem {
                message: format!("Failed to write temporary state file: {}", e),
                path: Some(temp_file.to_string_lossy().to_string()),
                operation: "write_temp".to_string(),
            }
        })?;

        // Create backup of previous state
        if self.state_file.exists() {
            if let Err(e) = fs::copy(&self.state_file, &self.backup_file) {
                warn!("⚠️ Failed to create backup: {}", e);
            }
        }

        // Atomic rename (this is the critical atomic operation)
        fs::rename(&temp_file, &self.state_file).map_err(|e| {
            RobinError::FileSystem {
                message: format!("Failed to atomically rename state file: {}", e),
                path: Some(self.state_file.to_string_lossy().to_string()),
                operation: "atomic_rename".to_string(),
            }
        })?;

        debug!("💾 State saved atomically: {} bytes", json_data.len());
        Ok(())
    }

    /// Load state from file with validation
    pub fn load_state(&self) -> RobinResult<StateSnapshot> {
        if !self.state_file.exists() {
            return Err(RobinError::FileSystem {
                message: "No saved state file found".to_string(),
                path: Some(self.state_file.to_string_lossy().to_string()),
                operation: "load_state".to_string(),
            });
        }

        self.load_state_from_file(&self.state_file)
    }

    /// Load state from specific file path
    fn load_state_from_file(&self, file_path: &Path) -> RobinResult<StateSnapshot> {
        let json_data = fs::read_to_string(file_path).map_err(|e| {
            RobinError::FileSystem {
                message: format!("Failed to read state file: {}", e),
                path: Some(file_path.to_string_lossy().to_string()),
                operation: "read_state".to_string(),
            }
        })?;

        let state: StateSnapshot = serde_json::from_str(&json_data).map_err(|e| {
            RobinError::FileSystem {
                message: format!("Failed to deserialize state: {}", e),
                path: Some(file_path.to_string_lossy().to_string()),
                operation: "deserialize_state".to_string(),
            }
        })?;

        // Validate checksum
        let calculated_checksum = self.calculate_checksum(&state);
        if state.checksum != calculated_checksum {
            return Err(RobinError::FileSystem {
                message: format!("State file checksum mismatch: expected {}, got {}",
                               calculated_checksum, state.checksum),
                path: Some(file_path.to_string_lossy().to_string()),
                operation: "validate_checksum".to_string(),
            });
        }

        // Validate version compatibility
        if state.version != self.app_version {
            warn!("📦 State file version mismatch: saved={}, current={}",
                  state.version, self.app_version);
        }

        info!("📂 State loaded successfully from: {}", file_path.display());
        Ok(state)
    }

    /// Calculate checksum for state validation
    fn calculate_checksum(&self, state: &StateSnapshot) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // Hash key fields (excluding checksum itself)
        state.timestamp.hash(&mut hasher);
        state.version.hash(&mut hasher);
        state.camera_state.position.iter().for_each(|&f| {
            f.to_bits().hash(&mut hasher);
        });
        state.world_state.voxel_data.len().hash(&mut hasher);
        state.build_state.current_build_mode.hash(&mut hasher);
        state.performance_state.performance_mode.hash(&mut hasher);

        hasher.finish()
    }

    /// Get state file information
    pub fn get_state_info(&self) -> StateInfo {
        StateInfo {
            state_file_exists: self.state_file.exists(),
            backup_file_exists: self.backup_file.exists(),
            lock_file_exists: self.lock_file.exists(),
            state_file_size: self.get_file_size(&self.state_file),
            backup_file_size: self.get_file_size(&self.backup_file),
            last_save_time: self.last_save,
        }
    }

    /// Get file size safely
    fn get_file_size(&self, path: &Path) -> Option<u64> {
        fs::metadata(path).ok().map(|m| m.len())
    }

    /// Set save debounce duration
    pub fn set_save_debounce(&mut self, duration: Duration) {
        self.save_debounce = duration;
        debug!("⏱️ Save debounce set to: {:?}", duration);
    }
}

/// Information about state files
#[derive(Debug, Clone)]
pub struct StateInfo {
    pub state_file_exists: bool,
    pub backup_file_exists: bool,
    pub lock_file_exists: bool,
    pub state_file_size: Option<u64>,
    pub backup_file_size: Option<u64>,
    pub last_save_time: Option<Instant>,
}

impl StateSnapshot {
    /// Create a new state snapshot with current timestamp
    pub fn new() -> Self {
        Self {
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            camera_state: CameraState::default(),
            world_state: WorldState::default(),
            build_state: BuildState::default(),
            performance_state: PerformanceState::default(),
            checksum: 0, // Will be calculated during save
        }
    }
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            position: [0.0, 5.0, 10.0],
            rotation: [0.0, 0.0],
            move_speed: 5.0,
            mouse_sensitivity: 0.002,
        }
    }
}

impl Default for WorldState {
    fn default() -> Self {
        Self {
            voxel_data: Vec::new(),
            world_seed: 12345,
            last_modified: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

impl Default for BuildState {
    fn default() -> Self {
        Self {
            current_build_mode: 0, // Default build mode
            current_voxel_type: 0, // Default voxel type
            show_hud: true,
            show_mode_selector: false,
        }
    }
}

impl Default for PerformanceState {
    fn default() -> Self {
        Self {
            performance_mode: 0, // High performance mode
            adaptive_quality_enabled: true,
            degradation_active: false,
            memory_pressure_level: 0.0,
        }
    }
}