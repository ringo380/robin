use super::*;
use std::{
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter},
    path::PathBuf,
    sync::{Arc, Mutex},
    time::SystemTime,
};

/// Main save manager that handles all save/load operations
pub struct SaveManager {
    /// Configuration
    config: SaveSystemConfig,
    /// Currently loaded saves metadata
    save_slots: Arc<Mutex<HashMap<usize, SaveMetadata>>>,
    /// Auto-save manager
    auto_save: Box<AutoSaveManager>,
    /// Save validation
    validator: SaveValidator,
}

impl SaveManager {
    /// Create a new save manager
    pub fn new(config: SaveSystemConfig) -> Self {
        // Ensure save directory exists
        if let Err(e) = config.ensure_save_directory() {
            log::error!("Failed to create save directory: {}", e);
        }

        let auto_save = Box::new(AutoSaveManager::new(config.clone()));
        let validator = SaveValidator::new(config.clone());

        let mut manager = Self {
            config: config.clone(),
            save_slots: Arc::new(Mutex::new(HashMap::new())),
            auto_save,
            validator,
        };

        // Scan existing saves
        if let Err(e) = manager.scan_existing_saves() {
            log::error!("Failed to scan existing saves: {}", e);
        }

        manager
    }

    /// Scan the save directory for existing saves
    fn scan_existing_saves(&mut self) -> SaveResult<()> {
        if !self.config.save_directory.exists() {
            return Ok(());
        }

        let entries = std::fs::read_dir(&self.config.save_directory)
            .map_err(|e| SaveError::IoError(e.to_string()))?;

        let mut slots = HashMap::new();

        for entry in entries {
            let entry = entry.map_err(|e| SaveError::IoError(e.to_string()))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some(&self.config.save_extension) {
                if let Some(metadata) = self.load_save_metadata(&path)? {
                    slots.insert(metadata.slot, metadata);
                }
            }
        }

        if let Ok(mut save_slots) = self.save_slots.lock() {
            *save_slots = slots;
        }

        Ok(())
    }

    /// Save a game state to the specified slot
    pub fn save_game(&mut self, slot: usize, game_state: &GameState) -> SaveResult<()> {
        if slot >= self.config.max_slots {
            return Err(SaveError::SlotFull(slot));
        }

        let save_path = self.get_save_path(slot);
        
        // Create backup if save already exists
        if save_path.exists() {
            self.create_backup(&save_path, slot)?;
        }

        // Validate the game state before saving
        self.validator.validate_game_state(game_state)?;

        // Serialize the game state
        let serialized = serde_json::to_vec_pretty(game_state)
            .map_err(|e| SaveError::SerializationError(e.to_string()))?;

        let mut data = serialized;

        // Apply compression if enabled
        if self.config.enable_compression {
            data = compression::compress_data(&data)?;
        }

        // Apply encryption if enabled
        if self.config.enable_encryption {
            // TODO: Generate or retrieve encryption key
            let key = b"dummy_key_32_bytes_long_for_demo";
            data = encryption::encrypt_data(&data, key)?;
        }

        // Write to file
        let mut file = File::create(&save_path)
            .map_err(|e| SaveError::IoError(e.to_string()))?;

        std::io::Write::write_all(&mut file, &data)
            .map_err(|e| SaveError::IoError(e.to_string()))?;

        // Update metadata in memory
        if let Ok(mut save_slots) = self.save_slots.lock() {
            save_slots.insert(slot, game_state.metadata.clone());
        }

        log::info!("Game saved to slot {} at {}", slot, save_path.display());
        Ok(())
    }

    /// Load a game state from the specified slot
    pub fn load_game(&self, slot: usize) -> SaveResult<GameState> {
        let save_path = self.get_save_path(slot);

        if !save_path.exists() {
            return Err(SaveError::SlotNotFound(slot));
        }

        // Read file data
        let mut data = std::fs::read(&save_path)
            .map_err(|e| SaveError::IoError(e.to_string()))?;

        // Apply decryption if enabled
        if self.config.enable_encryption {
            let key = b"dummy_key_32_bytes_long_for_demo";
            data = encryption::decrypt_data(&data, key)?;
        }

        // Apply decompression if enabled
        if self.config.enable_compression {
            data = compression::decompress_data(&data)?;
        }

        // Deserialize the game state
        let game_state: GameState = serde_json::from_slice(&data)
            .map_err(|e| SaveError::SerializationError(e.to_string()))?;

        // Validate the loaded game state
        self.validator.validate_game_state(&game_state)?;

        log::info!("Game loaded from slot {} at {}", slot, save_path.display());
        Ok(game_state)
    }

    /// Delete a save from the specified slot
    pub fn delete_save(&mut self, slot: usize) -> SaveResult<()> {
        let save_path = self.get_save_path(slot);

        if save_path.exists() {
            std::fs::remove_file(&save_path)
                .map_err(|e| SaveError::IoError(e.to_string()))?;

            // Remove from memory
            if let Ok(mut save_slots) = self.save_slots.lock() {
                save_slots.remove(&slot);
            }

            log::info!("Deleted save from slot {}", slot);
        }

        Ok(())
    }

    /// Get metadata for a specific save slot
    pub fn get_save_metadata(&self, slot: usize) -> Option<SaveMetadata> {
        if let Ok(save_slots) = self.save_slots.lock() {
            save_slots.get(&slot).cloned()
        } else {
            None
        }
    }

    /// Get metadata for all save slots
    pub fn get_all_save_metadata(&self) -> HashMap<usize, SaveMetadata> {
        if let Ok(save_slots) = self.save_slots.lock() {
            save_slots.clone()
        } else {
            HashMap::new()
        }
    }

    /// Check if a save slot exists
    pub fn save_exists(&self, slot: usize) -> bool {
        self.get_save_path(slot).exists()
    }

    /// Get available save slots
    pub fn get_available_slots(&self) -> Vec<usize> {
        (0..self.config.max_slots)
            .filter(|&slot| !self.save_exists(slot))
            .collect()
    }

    /// Get used save slots
    pub fn get_used_slots(&self) -> Vec<usize> {
        if let Ok(save_slots) = self.save_slots.lock() {
            save_slots.keys().copied().collect()
        } else {
            Vec::new()
        }
    }

    /// Import a save file from an external location
    pub fn import_save(&mut self, slot: usize, import_path: &Path) -> SaveResult<()> {
        if slot >= self.config.max_slots {
            return Err(SaveError::SlotFull(slot));
        }

        // Load and validate the imported save
        let data = std::fs::read(import_path)
            .map_err(|e| SaveError::IoError(e.to_string()))?;

        let game_state: GameState = serde_json::from_slice(&data)
            .map_err(|e| SaveError::SerializationError(e.to_string()))?;

        self.validator.validate_game_state(&game_state)?;

        // Save to the specified slot
        self.save_game(slot, &game_state)?;

        log::info!("Imported save from {} to slot {}", import_path.display(), slot);
        Ok(())
    }

    /// Export a save file to an external location
    pub fn export_save(&self, slot: usize, export_path: &Path) -> SaveResult<()> {
        let game_state = self.load_game(slot)?;

        let data = serde_json::to_vec_pretty(&game_state)
            .map_err(|e| SaveError::SerializationError(e.to_string()))?;

        std::fs::write(export_path, &data)
            .map_err(|e| SaveError::IoError(e.to_string()))?;

        log::info!("Exported save from slot {} to {}", slot, export_path.display());
        Ok(())
    }

    /// Create a backup of an existing save
    fn create_backup(&self, save_path: &Path, slot: usize) -> SaveResult<()> {
        let backup_dir = self.config.save_directory.join("backups");
        std::fs::create_dir_all(&backup_dir)
            .map_err(|e| SaveError::IoError(e.to_string()))?;

        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let backup_name = format!("slot_{}_backup_{}.{}", 
            slot, timestamp, self.config.save_extension);
        let backup_path = backup_dir.join(backup_name);

        std::fs::copy(save_path, &backup_path)
            .map_err(|e| SaveError::IoError(e.to_string()))?;

        // Clean up old backups
        self.cleanup_old_backups(slot)?;

        Ok(())
    }

    /// Clean up old backups for a specific slot
    fn cleanup_old_backups(&self, slot: usize) -> SaveResult<()> {
        let backup_dir = self.config.save_directory.join("backups");
        if !backup_dir.exists() {
            return Ok(());
        }

        let pattern = format!("slot_{}_backup_", slot);
        let mut backups = Vec::new();

        let entries = std::fs::read_dir(&backup_dir)
            .map_err(|e| SaveError::IoError(e.to_string()))?;

        for entry in entries {
            let entry = entry.map_err(|e| SaveError::IoError(e.to_string()))?;
            let path = entry.path();
            
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with(&pattern) {
                    if let Ok(metadata) = entry.metadata() {
                        if let Ok(created) = metadata.created() {
                            backups.push((path, created));
                        }
                    }
                }
            }
        }

        // Sort by creation time (newest first)
        backups.sort_by(|a, b| b.1.cmp(&a.1));

        // Remove old backups beyond the limit
        if backups.len() > self.config.max_backups {
            for (path, _) in backups.into_iter().skip(self.config.max_backups) {
                if let Err(e) = std::fs::remove_file(&path) {
                    log::warn!("Failed to remove old backup {}: {}", path.display(), e);
                }
            }
        }

        Ok(())
    }

    /// Get the file path for a save slot
    pub fn get_save_path(&self, slot: usize) -> PathBuf {
        let filename = format!("save_slot_{}.{}", slot, self.config.save_extension);
        self.config.save_directory.join(filename)
    }

    /// Load only the metadata from a save file
    fn load_save_metadata(&self, path: &Path) -> SaveResult<Option<SaveMetadata>> {
        let data = std::fs::read(path)
            .map_err(|e| SaveError::IoError(e.to_string()))?;

        // Try to parse as GameState and extract metadata
        match serde_json::from_slice::<GameState>(&data) {
            Ok(game_state) => Ok(Some(game_state.metadata)),
            Err(_) => {
                // If it fails, the save might be corrupted
                log::warn!("Failed to load metadata from {}", path.display());
                Ok(None)
            }
        }
    }

    /// Get auto-save manager
    pub fn get_auto_save_manager(&mut self) -> &mut AutoSaveManager {
        &mut self.auto_save
    }

    /// Update auto-save system
    pub fn update_auto_save(&mut self, game_state: &GameState) -> SaveResult<()> {
        if self.auto_save.should_auto_save() {
            self.auto_save.auto_save(game_state)?;
        }
        Ok(())
    }
}

/// Save file validator
pub struct SaveValidator {
    config: SaveSystemConfig,
}

impl SaveValidator {
    pub fn new(config: SaveSystemConfig) -> Self {
        Self { config }
    }

    /// Validate a game state before saving/loading
    pub fn validate_game_state(&self, game_state: &GameState) -> SaveResult<()> {
        // Check version compatibility
        let current_version = env!("CARGO_PKG_VERSION");
        if game_state.metadata.version != current_version {
            log::warn!("Save version mismatch: save={}, current={}", 
                game_state.metadata.version, current_version);
            // Allow loading of older saves but warn
        }

        // Validate player data
        if game_state.player_data.name.is_empty() {
            return Err(SaveError::CorruptedSave("Player name is empty".to_string()));
        }

        if game_state.player_data.health < 0.0 || game_state.player_data.max_health <= 0.0 {
            return Err(SaveError::CorruptedSave("Invalid player health values".to_string()));
        }

        if game_state.player_data.health > game_state.player_data.max_health {
            return Err(SaveError::CorruptedSave("Player health exceeds max health".to_string()));
        }

        // Validate progress
        if game_state.progress.completion_percentage < 0.0 || 
           game_state.progress.completion_percentage > 100.0 {
            return Err(SaveError::CorruptedSave("Invalid completion percentage".to_string()));
        }

        // Validate scene state
        if game_state.scene_state.name.is_empty() {
            return Err(SaveError::CorruptedSave("Scene name is empty".to_string()));
        }

        Ok(())
    }
}

/// Save utilities
pub mod utils {
    use super::*;

    /// Get human-readable file size
    pub fn format_file_size(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        format!("{:.1} {}", size, UNITS[unit_index])
    }

    /// Get save file size
    pub fn get_save_file_size(save_path: &Path) -> SaveResult<u64> {
        let metadata = std::fs::metadata(save_path)
            .map_err(|e| SaveError::IoError(e.to_string()))?;
        Ok(metadata.len())
    }

    /// Check if save directory has enough space (approximate)
    pub fn check_disk_space(save_dir: &Path, required_bytes: u64) -> bool {
        // This is a simplified check - in a real implementation,
        // you'd use platform-specific APIs to check available disk space
        if let Ok(metadata) = std::fs::metadata(save_dir) {
            // Assume we have enough space if we can read the directory
            // In practice, you'd check available disk space here
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_config() -> SaveSystemConfig {
        let temp_dir = TempDir::new().unwrap();
        SaveSystemConfig {
            save_directory: temp_dir.path().to_path_buf(),
            ..SaveSystemConfig::default()
        }
    }

    #[test]
    fn test_save_manager_creation() {
        let config = create_test_config();
        let manager = SaveManager::new(config.clone());
        
        assert!(config.save_directory.exists());
    }

    #[test]
    fn test_save_and_load() {
        let config = create_test_config();
        let mut manager = SaveManager::new(config);
        
        let mut game_state = GameState::new(0, "Test Save".to_string());
        game_state.player_data.name = "Test Player".to_string();
        game_state.player_data.level = 5;
        
        // Save the game
        assert!(manager.save_game(0, &game_state).is_ok());
        
        // Load the game
        let loaded_state = manager.load_game(0).unwrap();
        assert_eq!(loaded_state.player_data.name, "Test Player");
        assert_eq!(loaded_state.player_data.level, 5);
    }

    #[test]
    fn test_save_metadata() {
        let config = create_test_config();
        let mut manager = SaveManager::new(config);
        
        let game_state = GameState::new(1, "Test Save".to_string());
        manager.save_game(1, &game_state).unwrap();
        
        let metadata = manager.get_save_metadata(1).unwrap();
        assert_eq!(metadata.name, "Test Save");
        assert_eq!(metadata.slot, 1);
    }

    #[test]
    fn test_delete_save() {
        let config = create_test_config();
        let mut manager = SaveManager::new(config);
        
        let game_state = GameState::new(2, "Test Save".to_string());
        manager.save_game(2, &game_state).unwrap();
        
        assert!(manager.save_exists(2));
        
        manager.delete_save(2).unwrap();
        assert!(!manager.save_exists(2));
    }

    #[test]
    fn test_utils() {
        assert_eq!(utils::format_file_size(1024), "1.0 KB");
        assert_eq!(utils::format_file_size(1024 * 1024), "1.0 MB");
        assert_eq!(utils::format_file_size(512), "512.0 B");
    }
}