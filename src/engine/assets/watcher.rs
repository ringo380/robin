use super::{AssetConfig, AssetError, AssetResult, HotReloadEvent, AssetType, utils};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, mpsc},
    thread,
    time::{Duration, SystemTime, Instant},
};

/// Cross-platform file watcher for asset hot-reloading
pub struct AssetWatcher {
    config: AssetConfig,
    event_sender: mpsc::Sender<HotReloadEvent>,
    event_receiver: Arc<Mutex<mpsc::Receiver<HotReloadEvent>>>,
    watched_files: Arc<Mutex<HashMap<PathBuf, FileState>>>,
    running: Arc<Mutex<bool>>,
    thread_handle: Option<thread::JoinHandle<()>>,
}

/// Internal state of watched files
#[derive(Debug, Clone)]
struct FileState {
    last_modified: SystemTime,
    asset_type: AssetType,
    asset_id: String,
    size: u64,
    last_check: Instant,
}

impl FileState {
    fn new(path: &Path, asset_type: AssetType, asset_id: String) -> AssetResult<Self> {
        let metadata = std::fs::metadata(path)
            .map_err(|_| AssetError::FileNotFound(path.to_path_buf()))?;
        
        Ok(Self {
            last_modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            asset_type,
            asset_id,
            size: metadata.len(),
            last_check: Instant::now(),
        })
    }

    fn needs_update(&self, path: &Path) -> bool {
        // Rate limit file checking to avoid excessive I/O
        if self.last_check.elapsed() < Duration::from_millis(50) {
            return false;
        }

        if let Ok(metadata) = std::fs::metadata(path) {
            let current_modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            let current_size = metadata.len();
            
            current_modified > self.last_modified || current_size != self.size
        } else {
            false // File doesn't exist anymore
        }
    }

    fn update(&mut self, path: &Path) -> AssetResult<()> {
        let metadata = std::fs::metadata(path)
            .map_err(|_| AssetError::FileNotFound(path.to_path_buf()))?;
        
        self.last_modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        self.size = metadata.len();
        self.last_check = Instant::now();
        Ok(())
    }
}

impl AssetWatcher {
    /// Create a new asset watcher with the given configuration
    pub fn new(config: AssetConfig) -> Self {
        let (sender, receiver) = mpsc::channel();
        
        Self {
            config,
            event_sender: sender,
            event_receiver: Arc::new(Mutex::new(receiver)),
            watched_files: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new(Mutex::new(false)),
            thread_handle: None,
        }
    }

    /// Start watching for file changes
    pub fn start(&mut self) -> AssetResult<()> {
        if !self.config.watch_enabled {
            log::info!("Asset watching is disabled in configuration");
            return Ok(());
        }

        let mut running = self.running.lock().unwrap();
        if *running {
            return Err(AssetError::WatcherError("Watcher is already running".to_string()));
        }
        *running = true;

        log::info!("Starting asset watcher for directory: {}", self.config.base_path.display());

        // Create asset directories if they don't exist
        utils::ensure_asset_directories(&self.config)?;

        // Start the watching thread
        let config = self.config.clone();
        let sender = self.event_sender.clone();
        let watched_files = Arc::clone(&self.watched_files);
        let running_flag = Arc::clone(&self.running);

        let handle = thread::spawn(move || {
            Self::watch_thread(config, sender, watched_files, running_flag);
        });

        self.thread_handle = Some(handle);

        log::info!("Asset watcher started successfully");
        Ok(())
    }

    /// Stop watching for file changes
    pub fn stop(&mut self) {
        let mut running = self.running.lock().unwrap();
        if !*running {
            return;
        }
        *running = false;
        drop(running);

        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }

        log::info!("Asset watcher stopped");
    }

    /// Watch a specific file for changes
    pub fn watch_file<P: AsRef<Path>>(&self, path: P, asset_id: String) -> AssetResult<()> {
        let path = path.as_ref().to_path_buf();
        
        // Validate the file
        let asset_type = utils::validate_asset_format(&path, &self.config)?;
        utils::check_file_size(&path, self.config.max_file_size)?;

        let file_state = FileState::new(&path, asset_type, asset_id.clone())?;
        
        let mut watched_files = self.watched_files.lock().unwrap();
        watched_files.insert(path.clone(), file_state);
        
        log::debug!("Now watching file: {} ({})", utils::get_relative_path(&path, &self.config.base_path), asset_id);
        Ok(())
    }

    /// Stop watching a specific file
    pub fn unwatch_file<P: AsRef<Path>>(&self, path: P) {
        let path = path.as_ref().to_path_buf();
        let mut watched_files = self.watched_files.lock().unwrap();
        watched_files.remove(&path);
        log::debug!("Stopped watching file: {}", utils::get_relative_path(&path, &self.config.base_path));
    }

    /// Get the next reload event (non-blocking)
    pub fn poll_events(&self) -> Vec<HotReloadEvent> {
        let receiver = self.event_receiver.lock().unwrap();
        let mut events = Vec::new();
        
        while let Ok(event) = receiver.try_recv() {
            events.push(event);
        }
        
        events
    }

    /// Get the next reload event (blocking with timeout)
    pub fn wait_for_event(&self, timeout: Duration) -> Option<HotReloadEvent> {
        let receiver = self.event_receiver.lock().unwrap();
        receiver.recv_timeout(timeout).ok()
    }

    /// Scan all asset directories for new files
    pub fn scan_for_assets(&self) -> AssetResult<Vec<PathBuf>> {
        let mut found_assets = Vec::new();
        
        // Scan each asset directory
        let dirs_to_scan = [
            (&self.config.texture_path, AssetType::Texture),
            (&self.config.audio_path, AssetType::Audio),
            (&self.config.config_path, AssetType::Config),
            (&self.config.scene_path, AssetType::Scene),
        ];

        for (dir_path, asset_type) in &dirs_to_scan {
            if dir_path.exists() {
                self.scan_directory_recursive(dir_path, asset_type.clone(), &mut found_assets)?;
            }
        }

        log::info!("Asset scan complete: found {} assets", found_assets.len());
        Ok(found_assets)
    }

    /// Get statistics about watched files
    pub fn get_stats(&self) -> WatcherStats {
        let watched_files = self.watched_files.lock().unwrap();
        let running = *self.running.lock().unwrap();
        
        let mut type_counts = HashMap::new();
        for file_state in watched_files.values() {
            *type_counts.entry(file_state.asset_type.clone()).or_insert(0) += 1;
        }

        WatcherStats {
            total_watched: watched_files.len(),
            running,
            type_counts,
        }
    }

    // === PRIVATE METHODS ===

    fn watch_thread(
        config: AssetConfig,
        sender: mpsc::Sender<HotReloadEvent>,
        watched_files: Arc<Mutex<HashMap<PathBuf, FileState>>>,
        running: Arc<Mutex<bool>>,
    ) {
        log::debug!("Asset watcher thread started");
        
        let mut last_cleanup = Instant::now();
        let cleanup_interval = Duration::from_secs(30);
        
        while *running.lock().unwrap() {
            // Check all watched files for changes
            Self::check_watched_files(&watched_files, &sender);
            
            // Periodic cleanup of deleted files
            if last_cleanup.elapsed() > cleanup_interval {
                Self::cleanup_deleted_files(&watched_files, &sender);
                last_cleanup = Instant::now();
            }
            
            // Sleep for the configured delay
            thread::sleep(config.reload_delay);
        }
        
        log::debug!("Asset watcher thread stopped");
    }

    fn check_watched_files(
        watched_files: &Arc<Mutex<HashMap<PathBuf, FileState>>>,
        sender: &mpsc::Sender<HotReloadEvent>,
    ) {
        let mut files_to_update = Vec::new();
        
        // First pass: identify files that need updates
        {
            let files = watched_files.lock().unwrap();
            for (path, file_state) in files.iter() {
                if file_state.needs_update(path) {
                    files_to_update.push((path.clone(), file_state.clone()));
                }
            }
        }
        
        // Second pass: update files and send events
        for (path, mut file_state) in files_to_update {
            match file_state.update(&path) {
                Ok(()) => {
                    // Update the file state
                    {
                        let mut files = watched_files.lock().unwrap();
                        files.insert(path.clone(), file_state.clone());
                    }
                    
                    // Send modification event
                    let event = HotReloadEvent::AssetModified {
                        asset_id: file_state.asset_id.clone(),
                        asset_type: file_state.asset_type,
                        file_path: path.clone(),
                    };
                    
                    if let Err(e) = sender.send(event) {
                        log::error!("Failed to send reload event: {}", e);
                    } else {
                        log::debug!("Asset modified: {} ({})", 
                            path.display(), file_state.asset_id);
                    }
                }
                Err(AssetError::FileNotFound(_)) => {
                    // File was deleted
                    let event = HotReloadEvent::AssetDeleted {
                        asset_id: file_state.asset_id.clone(),
                        file_path: path.clone(),
                    };
                    
                    if let Err(e) = sender.send(event) {
                        log::error!("Failed to send deletion event: {}", e);
                    } else {
                        log::debug!("Asset deleted: {} ({})", 
                            path.display(), file_state.asset_id);
                    }
                }
                Err(e) => {
                    log::error!("Error checking file {}: {}", path.display(), e);
                }
            }
        }
    }

    fn cleanup_deleted_files(
        watched_files: &Arc<Mutex<HashMap<PathBuf, FileState>>>,
        sender: &mpsc::Sender<HotReloadEvent>,
    ) {
        let mut files_to_remove = Vec::new();
        
        {
            let files = watched_files.lock().unwrap();
            for (path, file_state) in files.iter() {
                if !path.exists() {
                    files_to_remove.push((path.clone(), file_state.asset_id.clone()));
                }
            }
        }
        
        for (path, asset_id) in files_to_remove {
            // Remove from watched files
            {
                let mut files = watched_files.lock().unwrap();
                files.remove(&path);
            }
            
            // Send deletion event
            let event = HotReloadEvent::AssetDeleted {
                asset_id: asset_id.clone(),
                file_path: path.clone(),
            };
            
            if let Err(e) = sender.send(event) {
                log::error!("Failed to send cleanup deletion event: {}", e);
            } else {
                log::debug!("Cleaned up deleted asset: {} ({})", path.display(), asset_id);
            }
        }
    }

    fn scan_directory_recursive(
        &self,
        dir_path: &Path,
        asset_type: AssetType,
        found_assets: &mut Vec<PathBuf>,
    ) -> AssetResult<()> {
        let entries = std::fs::read_dir(dir_path)
            .map_err(|e| AssetError::LoadFailed(format!("Failed to read directory {}: {}", dir_path.display(), e)))?;
        
        for entry in entries {
            let entry = entry
                .map_err(|e| AssetError::LoadFailed(format!("Failed to read directory entry: {}", e)))?;
            
            let path = entry.path();
            
            if path.is_dir() {
                // Recursively scan subdirectories
                self.scan_directory_recursive(&path, asset_type.clone(), found_assets)?;
            } else if path.is_file() {
                // Check if this is a valid asset file
                if let Ok(detected_type) = utils::validate_asset_format(&path, &self.config) {
                    if detected_type == asset_type {
                        found_assets.push(path);
                    }
                }
            }
        }
        
        Ok(())
    }
}

impl Drop for AssetWatcher {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Statistics about the file watcher
#[derive(Debug, Clone)]
pub struct WatcherStats {
    pub total_watched: usize,
    pub running: bool,
    pub type_counts: HashMap<AssetType, usize>,
}

impl WatcherStats {
    pub fn get_count_for_type(&self, asset_type: &AssetType) -> usize {
        self.type_counts.get(asset_type).copied().unwrap_or(0)
    }
}

/// Utility for batch watching multiple files
pub struct BatchWatcher {
    watcher: AssetWatcher,
    batch_delay: Duration,
    pending_files: Vec<(PathBuf, String)>,
}

impl BatchWatcher {
    pub fn new(config: AssetConfig, batch_delay: Duration) -> Self {
        Self {
            watcher: AssetWatcher::new(config),
            batch_delay,
            pending_files: Vec::new(),
        }
    }

    /// Add a file to the batch
    pub fn add_file<P: AsRef<Path>>(&mut self, path: P, asset_id: String) {
        self.pending_files.push((path.as_ref().to_path_buf(), asset_id));
    }

    /// Process all pending files at once
    pub fn process_batch(&mut self) -> AssetResult<()> {
        if self.pending_files.is_empty() {
            return Ok(());
        }

        log::info!("Processing batch of {} files for watching", self.pending_files.len());
        
        for (path, asset_id) in self.pending_files.drain(..) {
            if let Err(e) = self.watcher.watch_file(path.clone(), asset_id) {
                log::warn!("Failed to watch file {}: {}", path.display(), e);
            }
        }

        // Small delay to allow file system to settle
        thread::sleep(self.batch_delay);
        Ok(())
    }

    /// Start the underlying watcher
    pub fn start(&mut self) -> AssetResult<()> {
        self.watcher.start()
    }

    /// Stop the underlying watcher
    pub fn stop(&mut self) {
        self.watcher.stop();
    }

    /// Get events from the underlying watcher
    pub fn poll_events(&self) -> Vec<HotReloadEvent> {
        self.watcher.poll_events()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_watcher_creation() {
        let config = AssetConfig::default();
        let watcher = AssetWatcher::new(config);
        
        let stats = watcher.get_stats();
        assert_eq!(stats.total_watched, 0);
        assert!(!stats.running);
    }

    #[test]
    fn test_file_state_creation() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.png");
        fs::write(&file_path, b"test content").unwrap();

        let state = FileState::new(&file_path, AssetType::Texture, "test".to_string()).unwrap();
        assert_eq!(state.asset_id, "test");
        assert_eq!(state.asset_type, AssetType::Texture);
        assert_eq!(state.size, 12); // "test content".len()
    }

    #[test]
    fn test_batch_watcher() {
        let config = AssetConfig::default();
        let mut batch_watcher = BatchWatcher::new(config, Duration::from_millis(10));
        
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.png");
        fs::write(&file_path, b"test").unwrap();
        
        batch_watcher.add_file(file_path, "test_asset".to_string());
        assert_eq!(batch_watcher.pending_files.len(), 1);
    }

    #[test]
    fn test_watcher_stats() {
        let config = AssetConfig::default();
        let watcher = AssetWatcher::new(config);
        
        let stats = watcher.get_stats();
        assert_eq!(stats.get_count_for_type(&AssetType::Texture), 0);
        assert_eq!(stats.get_count_for_type(&AssetType::Audio), 0);
    }
}