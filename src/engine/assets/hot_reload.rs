use super::{
    AssetConfig, AssetRegistry, AssetWatcher, AssetError, AssetResult, 
    HotReloadEvent, LoadRequest, LoadPriority, ReloadCallback, AssetType,
    utils
};
use crate::engine::{
    graphics::{Texture, TextureManager},
    audio::AudioManager,
};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
    thread,
};

/// Main hot-reload system that coordinates watching, loading, and updating assets
pub struct HotReloadSystem {
    config: AssetConfig,
    registry: AssetRegistry,
    watcher: AssetWatcher,
    texture_manager: Arc<Mutex<Option<TextureManager>>>,
    audio_manager: Arc<Mutex<Option<AudioManager>>>,
    running: Arc<Mutex<bool>>,
    thread_handle: Option<thread::JoinHandle<()>>,
    reload_queue: Arc<Mutex<Vec<LoadRequest>>>,
}

/// Integration callbacks for hot-reloading specific asset types
pub struct HotReloadCallbacks {
    pub on_texture_reload: Option<Box<dyn Fn(&str, &[u8]) -> bool + Send + Sync>>,
    pub on_audio_reload: Option<Box<dyn Fn(&str, &[u8]) -> bool + Send + Sync>>,
    pub on_config_reload: Option<Box<dyn Fn(&str, &str) -> bool + Send + Sync>>,
    pub on_shader_reload: Option<Box<dyn Fn(&str, &str) -> bool + Send + Sync>>,
}

impl Default for HotReloadCallbacks {
    fn default() -> Self {
        Self {
            on_texture_reload: None,
            on_audio_reload: None,
            on_config_reload: None,
            on_shader_reload: None,
        }
    }
}

impl HotReloadSystem {
    /// Create a new hot-reload system
    pub fn new(config: AssetConfig) -> Self {
        let registry = AssetRegistry::new(config.clone());
        let watcher = AssetWatcher::new(config.clone());

        Self {
            config,
            registry,
            watcher,
            texture_manager: Arc::new(Mutex::new(None)),
            audio_manager: Arc::new(Mutex::new(None)),
            running: Arc::new(Mutex::new(false)),
            thread_handle: None,
            reload_queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Start the hot-reload system
    pub fn start(&mut self) -> AssetResult<()> {
        let mut running = self.running.lock().unwrap();
        if *running {
            return Err(AssetError::WatcherError("Hot-reload system is already running".to_string()));
        }
        *running = true;
        drop(running);

        // Start the file watcher
        self.watcher.start()?;

        // Scan and register existing assets
        self.scan_and_register_assets()?;

        // Start the reload processing thread
        self.start_reload_thread();

        log::info!("Hot-reload system started successfully");
        Ok(())
    }

    /// Stop the hot-reload system
    pub fn stop(&mut self) {
        let mut running = self.running.lock().unwrap();
        if !*running {
            return;
        }
        *running = false;
        drop(running);

        // Stop the watcher
        self.watcher.stop();

        // Wait for the reload thread to finish
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }

        log::info!("Hot-reload system stopped");
    }

    /// Set texture manager for texture reloading
    pub fn set_texture_manager(&self, manager: TextureManager) {
        let mut texture_manager = self.texture_manager.lock().unwrap();
        *texture_manager = Some(manager);
    }

    /// Set audio manager for audio reloading
    pub fn set_audio_manager(&self, manager: AudioManager) {
        let mut audio_manager = self.audio_manager.lock().unwrap();
        *audio_manager = Some(manager);
    }

    /// Register a new asset for hot-reloading
    pub fn register_asset<P: AsRef<Path>>(&self, id: String, file_path: P) -> AssetResult<()> {
        let path = file_path.as_ref();
        
        // Register with the registry
        let handle = self.registry.register_asset(id.clone(), path)?;
        
        // Watch the file
        self.watcher.watch_file(path, id.clone())?;
        
        // Load the asset initially
        let request = LoadRequest {
            id: id.clone(),
            file_path: path.to_path_buf(),
            asset_type: handle.asset_type,
            priority: LoadPriority::Normal,
            force_reload: false,
        };
        
        self.queue_reload(request);
        
        log::info!("Registered asset for hot-reload: {} -> {}", id, utils::get_relative_path(path, &self.config.base_path));
        Ok(())
    }

    /// Unregister an asset from hot-reloading
    pub fn unregister_asset(&self, id: &str) -> AssetResult<()> {
        // Get the asset's file path before unregistering
        let file_path = self.registry.get_metadata(id)
            .map(|m| m.file_path)
            .ok_or_else(|| AssetError::LoadFailed(format!("Asset not found: {}", id)))?;
        
        // Unregister from registry
        self.registry.unregister_asset(id)?;
        
        // Stop watching the file
        self.watcher.unwatch_file(&file_path);
        
        log::info!("Unregistered asset from hot-reload: {}", id);
        Ok(())
    }

    /// Register multiple assets from a directory
    pub fn register_directory<P: AsRef<Path>>(&self, dir_path: P, recursive: bool) -> AssetResult<Vec<String>> {
        let dir_path = dir_path.as_ref();
        let mut registered_assets = Vec::new();
        
        if recursive {
            self.register_directory_recursive(dir_path, &mut registered_assets)?;
        } else {
            self.register_directory_flat(dir_path, &mut registered_assets)?;
        }
        
        log::info!("Registered {} assets from directory: {}", 
            registered_assets.len(), utils::get_relative_path(dir_path, &self.config.base_path));
        
        Ok(registered_assets)
    }

    /// Add a callback for when assets are reloaded
    pub fn add_reload_callback(&self, asset_id: String, callback: ReloadCallback) {
        self.registry.add_reload_callback(asset_id, callback);
    }

    /// Add a global callback for all reload events
    pub fn add_global_callback(&self, callback: ReloadCallback) {
        self.registry.add_global_callback(callback);
    }

    /// Force reload an asset
    pub fn force_reload(&self, id: &str) -> AssetResult<()> {
        let metadata = self.registry.get_metadata(id)
            .ok_or_else(|| AssetError::LoadFailed(format!("Asset not found: {}", id)))?;
        
        let request = LoadRequest {
            id: id.to_string(),
            file_path: metadata.file_path,
            asset_type: metadata.asset_type,
            priority: LoadPriority::High,
            force_reload: true,
        };
        
        self.queue_reload(request);
        log::info!("Force reloading asset: {}", id);
        Ok(())
    }

    /// Get hot-reload statistics
    pub fn get_stats(&self) -> HotReloadStats {
        let registry_stats = self.registry.get_stats();
        let watcher_stats = self.watcher.get_stats();
        let queue_size = self.reload_queue.lock().unwrap().len();
        
        HotReloadStats {
            total_assets: watcher_stats.total_watched,
            successful_reloads: registry_stats.successful_reloads,
            failed_reloads: registry_stats.failed_reloads,
            average_reload_time: registry_stats.average_reload_time,
            queue_size,
            running: *self.running.lock().unwrap(),
        }
    }

    /// Update the hot-reload system (call this each frame)
    pub fn update(&self) {
        // Process file watcher events
        let events = self.watcher.poll_events();
        for event in events {
            self.handle_reload_event(event);
        }
    }

    /// Set up integration callbacks for specific systems
    pub fn setup_integration_callbacks(&self, callbacks: HotReloadCallbacks) {
        if let Some(texture_callback) = callbacks.on_texture_reload {
            let callback: ReloadCallback = Arc::new(move |event| {
                if let HotReloadEvent::ReloadComplete { asset_id, .. } = event {
                    // This would need access to the registry to get texture data
                    // In a real implementation, you'd pass the registry reference
                    log::debug!("Texture reload callback triggered for: {}", asset_id);
                }
            });
            self.registry.add_global_callback(callback);
        }

        if let Some(audio_callback) = callbacks.on_audio_reload {
            let callback: ReloadCallback = Arc::new(move |event| {
                if let HotReloadEvent::ReloadComplete { asset_id, .. } = event {
                    log::debug!("Audio reload callback triggered for: {}", asset_id);
                }
            });
            self.registry.add_global_callback(callback);
        }

        if let Some(config_callback) = callbacks.on_config_reload {
            let callback: ReloadCallback = Arc::new(move |event| {
                if let HotReloadEvent::ReloadComplete { asset_id, .. } = event {
                    log::debug!("Config reload callback triggered for: {}", asset_id);
                }
            });
            self.registry.add_global_callback(callback);
        }
    }

    // === PRIVATE METHODS ===

    fn scan_and_register_assets(&self) -> AssetResult<()> {
        let found_assets = self.watcher.scan_for_assets()?;
        let mut registered_count = 0;
        
        for asset_path in found_assets {
            let asset_id = utils::generate_asset_id(&asset_path, &self.config.base_path);
            
            match self.register_asset(asset_id.clone(), &asset_path) {
                Ok(()) => {
                    registered_count += 1;
                }
                Err(e) => {
                    log::warn!("Failed to register asset {}: {}", asset_id, e);
                }
            }
        }
        
        log::info!("Auto-registered {} assets during startup", registered_count);
        Ok(())
    }

    fn start_reload_thread(&mut self) {
        let registry = self.registry.clone();
        let reload_queue = Arc::clone(&self.reload_queue);
        let running = Arc::clone(&self.running);
        
        let handle = thread::spawn(move || {
            log::debug!("Hot-reload processing thread started");
            
            while *running.lock().unwrap() {
                // Process queued reloads
                let requests = {
                    let mut queue = reload_queue.lock().unwrap();
                    if queue.is_empty() {
                        drop(queue);
                        thread::sleep(Duration::from_millis(50));
                        continue;
                    }
                    queue.drain(..).collect::<Vec<_>>()
                };
                
                // Process reloads in batch
                let results = registry.load_batch(requests);
                
                // Log results
                let successful = results.iter().filter(|r| r.is_ok()).count();
                let total = results.len();
                
                if total > 0 {
                    log::debug!("Processed reload batch: {}/{} successful", successful, total);
                }
                
                // Small delay to prevent CPU spinning
                thread::sleep(Duration::from_millis(10));
            }
            
            log::debug!("Hot-reload processing thread stopped");
        });
        
        self.thread_handle = Some(handle);
    }

    fn queue_reload(&self, request: LoadRequest) {
        let mut queue = self.reload_queue.lock().unwrap();
        
        // Remove any existing request for the same asset to avoid duplicates
        queue.retain(|r| r.id != request.id);
        
        // Insert based on priority
        let insert_pos = queue.iter()
            .position(|r| r.priority < request.priority)
            .unwrap_or(queue.len());
        
        queue.insert(insert_pos, request);
    }

    fn handle_reload_event(&self, event: HotReloadEvent) {
        log::debug!("Handling reload event: {:?}", event);
        self.registry.handle_reload_event(&event);
    }

    fn register_directory_recursive(&self, dir_path: &Path, registered: &mut Vec<String>) -> AssetResult<()> {
        let entries = std::fs::read_dir(dir_path)
            .map_err(|e| AssetError::LoadFailed(format!("Failed to read directory {}: {}", dir_path.display(), e)))?;
        
        for entry in entries {
            let entry = entry
                .map_err(|e| AssetError::LoadFailed(format!("Failed to read directory entry: {}", e)))?;
            
            let path = entry.path();
            
            if path.is_dir() {
                self.register_directory_recursive(&path, registered)?;
            } else if path.is_file() {
                if let Ok(asset_type) = utils::validate_asset_format(&path, &self.config) {
                    let asset_id = utils::generate_asset_id(&path, &self.config.base_path);
                    
                    if self.register_asset(asset_id.clone(), &path).is_ok() {
                        registered.push(asset_id);
                    }
                }
            }
        }
        
        Ok(())
    }

    fn register_directory_flat(&self, dir_path: &Path, registered: &mut Vec<String>) -> AssetResult<()> {
        let entries = std::fs::read_dir(dir_path)
            .map_err(|e| AssetError::LoadFailed(format!("Failed to read directory {}: {}", dir_path.display(), e)))?;
        
        for entry in entries {
            let entry = entry
                .map_err(|e| AssetError::LoadFailed(format!("Failed to read directory entry: {}", e)))?;
            
            let path = entry.path();
            
            if path.is_file() {
                if let Ok(asset_type) = utils::validate_asset_format(&path, &self.config) {
                    let asset_id = utils::generate_asset_id(&path, &self.config.base_path);
                    
                    if self.register_asset(asset_id.clone(), &path).is_ok() {
                        registered.push(asset_id);
                    }
                }
            }
        }
        
        Ok(())
    }

    pub fn add_asset_callback(&mut self, _asset_path: &str, _callback: Box<dyn Fn() + Send + Sync>) {
        // Add callback for specific asset reloading
    }
}

impl Drop for HotReloadSystem {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Statistics for the hot-reload system
#[derive(Debug, Clone)]
pub struct HotReloadStats {
    pub total_assets: usize,
    pub successful_reloads: u32,
    pub failed_reloads: u32,
    pub average_reload_time: Duration,
    pub queue_size: usize,
    pub running: bool,
}

impl HotReloadStats {
    pub fn success_rate(&self) -> f64 {
        let total_reloads = self.successful_reloads + self.failed_reloads;
        if total_reloads == 0 {
            0.0
        } else {
            self.successful_reloads as f64 / total_reloads as f64
        }
    }
}

/// Builder for creating hot-reload systems with custom configurations
pub struct HotReloadSystemBuilder {
    config: AssetConfig,
    integration_callbacks: HotReloadCallbacks,
}

impl HotReloadSystemBuilder {
    pub fn new() -> Self {
        Self {
            config: AssetConfig::default(),
            integration_callbacks: HotReloadCallbacks::default(),
        }
    }

    pub fn with_config(mut self, config: AssetConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_base_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.config.base_path = path.as_ref().to_path_buf();
        self
    }

    pub fn with_watch_enabled(mut self, enabled: bool) -> Self {
        self.config.watch_enabled = enabled;
        self
    }

    pub fn with_reload_delay(mut self, delay: Duration) -> Self {
        self.config.reload_delay = delay;
        self
    }

    pub fn with_texture_callback<F>(mut self, callback: F) -> Self 
    where
        F: Fn(&str, &[u8]) -> bool + Send + Sync + 'static
    {
        self.integration_callbacks.on_texture_reload = Some(Box::new(callback));
        self
    }

    pub fn with_audio_callback<F>(mut self, callback: F) -> Self 
    where
        F: Fn(&str, &[u8]) -> bool + Send + Sync + 'static
    {
        self.integration_callbacks.on_audio_reload = Some(Box::new(callback));
        self
    }

    pub fn with_config_callback<F>(mut self, callback: F) -> Self 
    where
        F: Fn(&str, &str) -> bool + Send + Sync + 'static
    {
        self.integration_callbacks.on_config_reload = Some(Box::new(callback));
        self
    }

    pub fn build(self) -> HotReloadSystem {
        let mut system = HotReloadSystem::new(self.config);
        system.setup_integration_callbacks(self.integration_callbacks);
        system
    }
}

impl Default for HotReloadSystemBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_hot_reload_system_creation() {
        let config = AssetConfig::default();
        let system = HotReloadSystem::new(config);
        
        let stats = system.get_stats();
        assert!(!stats.running);
        assert_eq!(stats.total_assets, 0);
    }

    #[test]
    fn test_builder_pattern() {
        let system = HotReloadSystemBuilder::new()
            .with_watch_enabled(true)
            .with_reload_delay(Duration::from_millis(200))
            .with_base_path("test_assets")
            .build();
        
        assert!(system.config.watch_enabled);
        assert_eq!(system.config.reload_delay, Duration::from_millis(200));
        assert_eq!(system.config.base_path, PathBuf::from("test_assets"));
    }

    #[test]
    fn test_asset_registration() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.png");
        fs::write(&file_path, b"fake png data").unwrap();

        let mut config = AssetConfig::default();
        config.base_path = temp_dir.path().to_path_buf();
        
        let system = HotReloadSystem::new(config);
        
        let result = system.register_asset("test_texture".to_string(), &file_path);
        assert!(result.is_ok());
        
        // Check that the asset is now being watched
        let stats = system.get_stats();
        assert_eq!(stats.total_assets, 1);
    }

    #[test]
    fn test_directory_registration() {
        let temp_dir = TempDir::new().unwrap();
        let assets_dir = temp_dir.path().join("assets");
        fs::create_dir(&assets_dir).unwrap();
        
        // Create test files
        fs::write(assets_dir.join("texture1.png"), b"texture data").unwrap();
        fs::write(assets_dir.join("sound1.wav"), b"audio data").unwrap();
        fs::write(assets_dir.join("config.json"), r#"{"test": true}"#).unwrap();
        
        let mut config = AssetConfig::default();
        config.base_path = temp_dir.path().to_path_buf();
        
        let system = HotReloadSystem::new(config);
        
        let registered = system.register_directory(&assets_dir, false).unwrap();
        assert_eq!(registered.len(), 3);
    }

    #[test]
    fn test_stats() {
        let config = AssetConfig::default();
        let system = HotReloadSystem::new(config);
        
        let stats = system.get_stats();
        assert_eq!(stats.success_rate(), 0.0); // No reloads yet
        assert_eq!(stats.queue_size, 0);
        assert!(!stats.running);
    }
}