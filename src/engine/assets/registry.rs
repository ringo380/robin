use super::{
    AssetConfig, AssetError, AssetResult, AssetMetadata, AssetType, 
    HotReloadEvent, ReloadCallback, ReloadStats, utils
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock},
    time::{Duration, Instant, SystemTime},
};
use serde::{Deserialize, Serialize};

/// Central registry for all game assets with hot-reloading capabilities
#[derive(Clone)]
pub struct AssetRegistry {
    config: AssetConfig,
    assets: Arc<RwLock<HashMap<String, AssetMetadata>>>,
    reload_callbacks: Arc<Mutex<HashMap<String, Vec<ReloadCallback>>>>,
    global_callbacks: Arc<Mutex<Vec<ReloadCallback>>>,
    stats: Arc<Mutex<ReloadStats>>,
    
    // Asset data storage (in a real engine, this would be more sophisticated)
    texture_data: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    audio_data: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    config_data: Arc<Mutex<HashMap<String, String>>>,
}

/// Handle for a registered asset
#[derive(Debug, Clone)]
pub struct AssetHandle {
    pub id: String,
    pub asset_type: AssetType,
    pub version: u64,
    pub loaded: bool,
}

/// Asset loading request
#[derive(Debug, Clone)]
pub struct LoadRequest {
    pub id: String,
    pub file_path: PathBuf,
    pub asset_type: AssetType,
    pub priority: LoadPriority,
    pub force_reload: bool,
}

/// Loading priority for assets
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LoadPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Asset dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetDependency {
    pub asset_id: String,
    pub dependency_id: String,
    pub dependency_type: DependencyType,
}

/// Types of asset dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    /// Hard dependency - asset cannot load without this
    Required,
    /// Soft dependency - asset can load but may have reduced functionality
    Optional,
    /// Reference dependency - used by this asset but can be loaded later
    Reference,
}

impl AssetRegistry {
    /// Create a new asset registry with the given configuration
    pub fn new(config: AssetConfig) -> Self {
        Self {
            config,
            assets: Arc::new(RwLock::new(HashMap::new())),
            reload_callbacks: Arc::new(Mutex::new(HashMap::new())),
            global_callbacks: Arc::new(Mutex::new(Vec::new())),
            stats: Arc::new(Mutex::new(ReloadStats::default())),
            texture_data: Arc::new(Mutex::new(HashMap::new())),
            audio_data: Arc::new(Mutex::new(HashMap::new())),
            config_data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register a new asset in the registry
    pub fn register_asset<P: AsRef<Path>>(&self, id: String, file_path: P) -> AssetResult<AssetHandle> {
        let path = file_path.as_ref().to_path_buf();
        
        // Validate the asset file
        let asset_type = utils::validate_asset_format(&path, &self.config)?;
        utils::check_file_size(&path, self.config.max_file_size)?;
        
        // Create metadata
        let metadata = AssetMetadata::new(id.clone(), asset_type.clone(), path);
        
        // Register in the registry
        {
            let mut assets = self.assets.write().unwrap();
            assets.insert(id.clone(), metadata);
        }
        
        // Update stats
        {
            let mut stats = self.stats.lock().unwrap();
            stats.assets_watched += 1;
        }
        
        log::debug!("Registered asset: {} ({:?})", id, asset_type);
        
        Ok(AssetHandle {
            id,
            asset_type,
            version: 0,
            loaded: false,
        })
    }

    /// Unregister an asset from the registry
    pub fn unregister_asset(&self, id: &str) -> AssetResult<()> {
        let removed = {
            let mut assets = self.assets.write().unwrap();
            assets.remove(id).is_some()
        };
        
        if removed {
            // Remove callbacks for this asset
            {
                let mut callbacks = self.reload_callbacks.lock().unwrap();
                callbacks.remove(id);
            }
            
            // Remove asset data
            {
                let mut texture_data = self.texture_data.lock().unwrap();
                texture_data.remove(id);
                
                let mut audio_data = self.audio_data.lock().unwrap();
                audio_data.remove(id);
                
                let mut config_data = self.config_data.lock().unwrap();
                config_data.remove(id);
            }
            
            // Update stats
            {
                let mut stats = self.stats.lock().unwrap();
                if stats.assets_watched > 0 {
                    stats.assets_watched -= 1;
                }
            }
            
            log::debug!("Unregistered asset: {}", id);
            Ok(())
        } else {
            Err(AssetError::LoadFailed(format!("Asset not found: {}", id)))
        }
    }

    /// Load an asset synchronously
    pub fn load_asset(&self, request: LoadRequest) -> AssetResult<AssetHandle> {
        let start_time = Instant::now();
        log::debug!("Loading asset: {} from {}", request.id, request.file_path.display());
        
        // Check if asset exists and needs reloading
        let needs_load = {
            let assets = self.assets.read().unwrap();
            if let Some(metadata) = assets.get(&request.id) {
                request.force_reload || metadata.needs_reload()
            } else {
                true // Asset not registered yet
            }
        };
        
        if !needs_load {
            // Asset is up to date
            let assets = self.assets.read().unwrap();
            let metadata = assets.get(&request.id).unwrap();
            
            return Ok(AssetHandle {
                id: request.id,
                asset_type: metadata.asset_type.clone(),
                version: metadata.load_count as u64,
                loaded: true,
            });
        }
        
        // Load the asset data
        let load_result = self.load_asset_data(&request);
        let load_duration = start_time.elapsed();
        
        match load_result {
            Ok(handle) => {
                // Update metadata
                {
                    let mut assets = self.assets.write().unwrap();
                    if let Some(metadata) = assets.get_mut(&request.id) {
                        metadata.update_modified_time();
                        metadata.load_count += 1;
                        
                        // Estimate memory size (simplified)
                        metadata.memory_size = self.estimate_asset_memory_size(&request.id, &request.asset_type);
                    }
                }
                
                // Update stats
                {
                    let mut stats = self.stats.lock().unwrap();
                    stats.add_reload(true, load_duration);
                }
                
                // Trigger reload callbacks
                let event = HotReloadEvent::ReloadComplete {
                    asset_id: request.id.clone(),
                    reload_time: load_duration,
                };
                self.trigger_callbacks(&request.id, &event);
                
                log::debug!("Successfully loaded asset: {} in {:.2?}", request.id, load_duration);
                Ok(handle)
            }
            Err(e) => {
                // Update stats
                {
                    let mut stats = self.stats.lock().unwrap();
                    stats.add_reload(false, load_duration);
                }
                
                // Trigger error callbacks
                let event = HotReloadEvent::ReloadFailed {
                    asset_id: request.id.clone(),
                    error: e.to_string(),
                };
                self.trigger_callbacks(&request.id, &event);
                
                log::error!("Failed to load asset: {} - {}", request.id, e);
                Err(e)
            }
        }
    }

    /// Load multiple assets in batch
    pub fn load_batch(&self, requests: Vec<LoadRequest>) -> Vec<AssetResult<AssetHandle>> {
        let start_time = Instant::now();
        log::info!("Loading batch of {} assets", requests.len());
        
        let mut results = Vec::with_capacity(requests.len());
        let mut successful = 0;
        
        // Sort by priority (highest first)
        let mut sorted_requests = requests;
        sorted_requests.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        for request in sorted_requests {
            let result = self.load_asset(request);
            if result.is_ok() {
                successful += 1;
            }
            results.push(result);
        }
        
        log::info!("Batch load complete: {}/{} successful in {:.2?}", 
            successful, results.len(), start_time.elapsed());
        
        results
    }

    /// Get asset metadata by ID
    pub fn get_metadata(&self, id: &str) -> Option<AssetMetadata> {
        let assets = self.assets.read().unwrap();
        assets.get(id).cloned()
    }

    /// Get all registered asset IDs
    pub fn get_all_asset_ids(&self) -> Vec<String> {
        let assets = self.assets.read().unwrap();
        assets.keys().cloned().collect()
    }

    /// Get assets by type
    pub fn get_assets_by_type(&self, asset_type: AssetType) -> Vec<String> {
        let assets = self.assets.read().unwrap();
        assets.values()
            .filter(|metadata| metadata.asset_type == asset_type)
            .map(|metadata| metadata.id.clone())
            .collect()
    }

    /// Check if an asset is registered
    pub fn has_asset(&self, id: &str) -> bool {
        let assets = self.assets.read().unwrap();
        assets.contains_key(id)
    }

    /// Get asset data (for textures)
    pub fn get_texture_data(&self, id: &str) -> Option<Vec<u8>> {
        let texture_data = self.texture_data.lock().unwrap();
        texture_data.get(id).cloned()
    }

    /// Get asset data (for audio)
    pub fn get_audio_data(&self, id: &str) -> Option<Vec<u8>> {
        let audio_data = self.audio_data.lock().unwrap();
        audio_data.get(id).cloned()
    }

    /// Get asset data (for config)
    pub fn get_config_data(&self, id: &str) -> Option<String> {
        let config_data = self.config_data.lock().unwrap();
        config_data.get(id).cloned()
    }

    /// Add a reload callback for a specific asset
    pub fn add_reload_callback(&self, asset_id: String, callback: ReloadCallback) {
        let mut callbacks = self.reload_callbacks.lock().unwrap();
        callbacks.entry(asset_id)
            .or_insert_with(Vec::new)
            .push(callback);
    }

    /// Add a global reload callback that receives all events
    pub fn add_global_callback(&self, callback: ReloadCallback) {
        let mut callbacks = self.global_callbacks.lock().unwrap();
        callbacks.push(callback);
    }

    /// Handle a hot reload event
    pub fn handle_reload_event(&self, event: &HotReloadEvent) {
        match event {
            HotReloadEvent::AssetModified { asset_id, file_path, .. } => {
                log::debug!("Handling asset modification: {}", asset_id);
                
                // Create a reload request
                let request = LoadRequest {
                    id: asset_id.clone(),
                    file_path: file_path.clone(),
                    asset_type: self.get_metadata(asset_id)
                        .map(|m| m.asset_type)
                        .unwrap_or(AssetType::Data),
                    priority: LoadPriority::High,
                    force_reload: true,
                };
                
                // Reload the asset
                let _ = self.load_asset(request);
            }
            HotReloadEvent::AssetDeleted { asset_id, .. } => {
                log::debug!("Handling asset deletion: {}", asset_id);
                let _ = self.unregister_asset(asset_id);
            }
            HotReloadEvent::AssetCreated { file_path, asset_type } => {
                log::debug!("Handling new asset creation: {}", file_path.display());
                
                // Generate an asset ID
                let asset_id = utils::generate_asset_id(file_path, &self.config.base_path);
                
                // Register the new asset
                if let Ok(handle) = self.register_asset(asset_id.clone(), file_path) {
                    // Auto-load new assets
                    let request = LoadRequest {
                        id: asset_id,
                        file_path: file_path.clone(),
                        asset_type: asset_type.clone(),
                        priority: LoadPriority::Normal,
                        force_reload: false,
                    };
                    let _ = self.load_asset(request);
                }
            }
            _ => {
                // Handle other event types
                self.trigger_global_callbacks(event);
            }
        }
    }

    /// Get registry statistics
    pub fn get_stats(&self) -> ReloadStats {
        let stats = self.stats.lock().unwrap();
        stats.clone()
    }

    /// Clear all assets from the registry
    pub fn clear(&self) {
        {
            let mut assets = self.assets.write().unwrap();
            assets.clear();
        }
        
        {
            let mut callbacks = self.reload_callbacks.lock().unwrap();
            callbacks.clear();
        }
        
        {
            let mut texture_data = self.texture_data.lock().unwrap();
            texture_data.clear();
            
            let mut audio_data = self.audio_data.lock().unwrap();
            audio_data.clear();
            
            let mut config_data = self.config_data.lock().unwrap();
            config_data.clear();
        }
        
        {
            let mut stats = self.stats.lock().unwrap();
            *stats = ReloadStats::default();
        }
        
        log::info!("Asset registry cleared");
    }

    /// Export asset manifest
    pub fn export_manifest(&self) -> AssetResult<String> {
        let assets = self.assets.read().unwrap();
        let manifest: Vec<&AssetMetadata> = assets.values().collect();
        
        serde_json::to_string_pretty(&manifest)
            .map_err(|e| AssetError::SerializationError(e.to_string()))
    }

    // === PRIVATE METHODS ===

    fn load_asset_data(&self, request: &LoadRequest) -> AssetResult<AssetHandle> {
        match request.asset_type {
            AssetType::Texture => self.load_texture_data(request),
            AssetType::Audio => self.load_audio_data(request),
            AssetType::Config => self.load_config_data(request),
            AssetType::Scene => self.load_scene_data(request),
            _ => self.load_generic_data(request),
        }
    }

    fn load_texture_data(&self, request: &LoadRequest) -> AssetResult<AssetHandle> {
        let data = std::fs::read(&request.file_path)
            .map_err(|e| AssetError::LoadFailed(format!("Failed to read texture file: {}", e)))?;
        
        // Store texture data (in a real engine, you'd decode and create GPU textures here)
        {
            let mut texture_data = self.texture_data.lock().unwrap();
            texture_data.insert(request.id.clone(), data);
        }
        
        Ok(AssetHandle {
            id: request.id.clone(),
            asset_type: AssetType::Texture,
            version: 1,
            loaded: true,
        })
    }

    fn load_audio_data(&self, request: &LoadRequest) -> AssetResult<AssetHandle> {
        let data = std::fs::read(&request.file_path)
            .map_err(|e| AssetError::LoadFailed(format!("Failed to read audio file: {}", e)))?;
        
        // Store audio data (in a real engine, you'd decode and create audio sources here)
        {
            let mut audio_data = self.audio_data.lock().unwrap();
            audio_data.insert(request.id.clone(), data);
        }
        
        Ok(AssetHandle {
            id: request.id.clone(),
            asset_type: AssetType::Audio,
            version: 1,
            loaded: true,
        })
    }

    fn load_config_data(&self, request: &LoadRequest) -> AssetResult<AssetHandle> {
        let data = std::fs::read_to_string(&request.file_path)
            .map_err(|e| AssetError::LoadFailed(format!("Failed to read config file: {}", e)))?;
        
        // Validate JSON/TOML/YAML (simplified validation)
        let extension = request.file_path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        
        match extension.to_lowercase().as_str() {
            "json" => {
                serde_json::from_str::<serde_json::Value>(&data)
                    .map_err(|e| AssetError::LoadFailed(format!("Invalid JSON: {}", e)))?;
            }
            _ => {
                // For other formats, just store as string
            }
        }
        
        // Store config data
        {
            let mut config_data = self.config_data.lock().unwrap();
            config_data.insert(request.id.clone(), data);
        }
        
        Ok(AssetHandle {
            id: request.id.clone(),
            asset_type: AssetType::Config,
            version: 1,
            loaded: true,
        })
    }

    fn load_scene_data(&self, request: &LoadRequest) -> AssetResult<AssetHandle> {
        // Scene loading would be more complex in a real engine
        self.load_config_data(request)
    }

    fn load_generic_data(&self, request: &LoadRequest) -> AssetResult<AssetHandle> {
        // For unknown asset types, just verify the file exists
        if !request.file_path.exists() {
            return Err(AssetError::FileNotFound(request.file_path.clone()));
        }
        
        Ok(AssetHandle {
            id: request.id.clone(),
            asset_type: request.asset_type.clone(),
            version: 1,
            loaded: true,
        })
    }

    fn estimate_asset_memory_size(&self, id: &str, asset_type: &AssetType) -> usize {
        match asset_type {
            AssetType::Texture => {
                let texture_data = self.texture_data.lock().unwrap();
                texture_data.get(id).map(|data| data.len()).unwrap_or(0)
            }
            AssetType::Audio => {
                let audio_data = self.audio_data.lock().unwrap();
                audio_data.get(id).map(|data| data.len()).unwrap_or(0)
            }
            AssetType::Config => {
                let config_data = self.config_data.lock().unwrap();
                config_data.get(id).map(|data| data.len()).unwrap_or(0)
            }
            _ => 0,
        }
    }

    fn trigger_callbacks(&self, asset_id: &str, event: &HotReloadEvent) {
        // Asset-specific callbacks
        {
            let callbacks = self.reload_callbacks.lock().unwrap();
            if let Some(asset_callbacks) = callbacks.get(asset_id) {
                for callback in asset_callbacks {
                    callback(event);
                }
            }
        }
        
        // Global callbacks
        self.trigger_global_callbacks(event);
    }

    fn trigger_global_callbacks(&self, event: &HotReloadEvent) {
        let callbacks = self.global_callbacks.lock().unwrap();
        for callback in callbacks.iter() {
            callback(event);
        }
    }
}

/// Builder for creating load requests
pub struct LoadRequestBuilder {
    id: String,
    file_path: PathBuf,
    asset_type: AssetType,
    priority: LoadPriority,
    force_reload: bool,
}

impl LoadRequestBuilder {
    pub fn new<P: AsRef<Path>>(id: String, file_path: P, asset_type: AssetType) -> Self {
        Self {
            id,
            file_path: file_path.as_ref().to_path_buf(),
            asset_type,
            priority: LoadPriority::Normal,
            force_reload: false,
        }
    }

    pub fn priority(mut self, priority: LoadPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn force_reload(mut self, force: bool) -> Self {
        self.force_reload = force;
        self
    }

    pub fn build(self) -> LoadRequest {
        LoadRequest {
            id: self.id,
            file_path: self.file_path,
            asset_type: self.asset_type,
            priority: self.priority,
            force_reload: self.force_reload,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_registry_creation() {
        let config = AssetConfig::default();
        let registry = AssetRegistry::new(config);
        
        assert_eq!(registry.get_all_asset_ids().len(), 0);
        let stats = registry.get_stats();
        assert_eq!(stats.total_reloads, 0);
    }

    #[test]
    fn test_asset_registration() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.png");
        fs::write(&file_path, b"fake png data").unwrap();

        let config = AssetConfig::default();
        let registry = AssetRegistry::new(config);
        
        let handle = registry.register_asset("test_texture".to_string(), &file_path).unwrap();
        assert_eq!(handle.id, "test_texture");
        assert_eq!(handle.asset_type, AssetType::Texture);
        assert!(!handle.loaded);

        assert!(registry.has_asset("test_texture"));
        let metadata = registry.get_metadata("test_texture").unwrap();
        assert_eq!(metadata.asset_type, AssetType::Texture);
    }

    #[test]
    fn test_load_request_builder() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.png");
        
        let request = LoadRequestBuilder::new("test".to_string(), &file_path, AssetType::Texture)
            .priority(LoadPriority::High)
            .force_reload(true)
            .build();
        
        assert_eq!(request.priority, LoadPriority::High);
        assert!(request.force_reload);
    }

    #[test]
    fn test_batch_loading() {
        let temp_dir = TempDir::new().unwrap();
        let config = AssetConfig::default();
        let registry = AssetRegistry::new(config);
        
        // Create test files
        let files = vec!["test1.png", "test2.wav", "test3.json"];
        let mut requests = Vec::new();
        
        for (i, filename) in files.iter().enumerate() {
            let file_path = temp_dir.path().join(filename);
            fs::write(&file_path, format!("test data {}", i)).unwrap();
            
            let asset_type = match filename.split('.').last().unwrap() {
                "png" => AssetType::Texture,
                "wav" => AssetType::Audio,
                "json" => AssetType::Config,
                _ => AssetType::Data,
            };
            
            registry.register_asset(format!("test_{}", i), &file_path).unwrap();
            
            requests.push(LoadRequest {
                id: format!("test_{}", i),
                file_path,
                asset_type,
                priority: LoadPriority::Normal,
                force_reload: false,
            });
        }
        
        let results = registry.load_batch(requests);
        assert_eq!(results.len(), 3);
        
        // Check that all loaded successfully
        for result in results {
            assert!(result.is_ok());
        }
    }
}