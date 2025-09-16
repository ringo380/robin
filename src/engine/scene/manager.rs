use crate::engine::{
    scene::{Scene, SerializableScene, SceneSerializer, SceneResult, SceneMetadata, GlobalSceneSettings},
    assets::{AssetType, HotReloadEvent},
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Scene management system for loading, saving, and managing multiple scenes
pub struct SceneManager {
    scenes: HashMap<String, Scene>,
    active_scene: Option<String>,
    scene_paths: HashMap<String, PathBuf>,
    scene_metadata: HashMap<String, SceneMetadata>,
    auto_save_enabled: bool,
    auto_save_interval: std::time::Duration,
    last_save_time: std::time::Instant,
    scene_directory: PathBuf,
}

impl SceneManager {
    /// Create a new scene manager
    pub fn new() -> Self {
        Self {
            scenes: HashMap::new(),
            active_scene: None,
            scene_paths: HashMap::new(),
            scene_metadata: HashMap::new(),
            auto_save_enabled: false,
            auto_save_interval: std::time::Duration::from_secs(300), // 5 minutes
            last_save_time: std::time::Instant::now(),
            scene_directory: PathBuf::from("scenes"),
        }
    }

    /// Set the directory where scenes are stored
    pub fn set_scene_directory<P: AsRef<Path>>(&mut self, path: P) {
        self.scene_directory = path.as_ref().to_path_buf();
        
        // Create directory if it doesn't exist
        if !self.scene_directory.exists() {
            if let Err(e) = std::fs::create_dir_all(&self.scene_directory) {
                log::error!("Failed to create scene directory {}: {}", 
                    self.scene_directory.display(), e);
            }
        }
    }

    /// Enable/disable auto-save functionality
    pub fn set_auto_save(&mut self, enabled: bool, interval_seconds: Option<u64>) {
        self.auto_save_enabled = enabled;
        if let Some(seconds) = interval_seconds {
            self.auto_save_interval = std::time::Duration::from_secs(seconds);
        }
        log::info!("Auto-save {}: interval={}s", 
            if enabled { "enabled" } else { "disabled" },
            self.auto_save_interval.as_secs());
    }

    /// Create a new empty scene
    pub fn create_scene(&mut self, name: &str) -> &mut Scene {
        let scene = Scene::new();
        self.scenes.insert(name.to_string(), scene);
        self.scene_metadata.insert(name.to_string(), SceneMetadata {
            author: "Scene Manager".to_string(),
            description: format!("Scene created: {}", name),
            ..Default::default()
        });
        
        log::info!("Created new scene: {}", name);
        self.scenes.get_mut(name).unwrap()
    }

    /// Load a scene from file
    pub fn load_scene<P: AsRef<Path>>(&mut self, name: &str, path: P) -> SceneResult<()> {
        let path = path.as_ref();
        log::info!("Loading scene '{}' from {}", name, path.display());
        
        // Load serializable scene
        let serializable_scene = SceneSerializer::load_scene(path)?;
        
        // Validate version compatibility
        SceneSerializer::validate_version(&serializable_scene)?;
        
        // Convert to runtime scene
        let scene = SceneSerializer::serializable_to_scene(&serializable_scene)?;
        
        // Store scene and metadata
        self.scenes.insert(name.to_string(), scene);
        self.scene_paths.insert(name.to_string(), path.to_path_buf());
        self.scene_metadata.insert(name.to_string(), serializable_scene.metadata.clone());
        
        log::info!("Successfully loaded scene '{}' with {} objects", 
            name, serializable_scene.objects.len());
        Ok(())
    }

    /// Save a scene to file
    pub fn save_scene(&mut self, name: &str, path: Option<PathBuf>) -> SceneResult<()> {
        let scene = self.scenes.get(name)
            .ok_or_else(|| crate::engine::scene::serialization::SceneSerializationError::InvalidPath(
                PathBuf::from(format!("Scene '{}' not found", name))
            ))?;

        // Determine save path
        let save_path = match path {
            Some(p) => {
                self.scene_paths.insert(name.to_string(), p.clone());
                p
            }
            None => {
                self.scene_paths.get(name)
                    .cloned()
                    .unwrap_or_else(|| {
                        let filename = format!("{}.json", name.replace(' ', "_"));
                        self.scene_directory.join(filename)
                    })
            }
        };

        log::info!("Saving scene '{}' to {}", name, save_path.display());

        // Get metadata
        let metadata = self.scene_metadata.get(name)
            .cloned()
            .unwrap_or_else(|| {
                let mut meta = SceneMetadata::default();
                meta.description = format!("Scene: {}", name);
                meta
            });

        // Convert to serializable format
        let serializable_scene = SceneSerializer::scene_to_serializable(
            scene, 
            name.to_string(),
            Some(metadata),
            Some(GlobalSceneSettings::default())
        );

        // Save to file
        SceneSerializer::save_scene(&serializable_scene, &save_path, None)?;
        self.scene_paths.insert(name.to_string(), save_path);
        
        log::info!("Successfully saved scene '{}'", name);
        Ok(())
    }

    /// Set the active scene
    pub fn set_active_scene(&mut self, name: &str) -> Option<&mut Scene> {
        if self.scenes.contains_key(name) {
            self.active_scene = Some(name.to_string());
            log::info!("Set active scene to: {}", name);
            self.get_scene_mut(name)
        } else {
            log::warn!("Scene '{}' not found", name);
            None
        }
    }

    /// Get the active scene
    pub fn get_active_scene(&self) -> Option<&Scene> {
        self.active_scene.as_ref()
            .and_then(|name| self.scenes.get(name))
    }

    /// Get the active scene mutably
    pub fn get_active_scene_mut(&mut self) -> Option<&mut Scene> {
        self.active_scene.clone()
            .and_then(|name| self.scenes.get_mut(&name))
    }

    /// Get a scene by name
    pub fn get_scene(&self, name: &str) -> Option<&Scene> {
        self.scenes.get(name)
    }

    /// Get a scene by name mutably
    pub fn get_scene_mut(&mut self, name: &str) -> Option<&mut Scene> {
        self.scenes.get_mut(name)
    }

    /// Remove a scene
    pub fn remove_scene(&mut self, name: &str) -> Option<Scene> {
        self.scene_paths.remove(name);
        self.scene_metadata.remove(name);
        
        if self.active_scene.as_ref() == Some(&name.to_string()) {
            self.active_scene = None;
        }
        
        let removed = self.scenes.remove(name);
        if removed.is_some() {
            log::info!("Removed scene: {}", name);
        }
        removed
    }

    /// Get list of all scene names
    pub fn list_scenes(&self) -> Vec<&String> {
        self.scenes.keys().collect()
    }

    /// Duplicate a scene
    pub fn duplicate_scene(&mut self, source_name: &str, new_name: &str) -> SceneResult<()> {
        // First save the source scene to a temporary serializable format
        let source_scene = self.scenes.get(source_name)
            .ok_or_else(|| crate::engine::scene::serialization::SceneSerializationError::InvalidPath(
                PathBuf::from(format!("Source scene '{}' not found", source_name))
            ))?;

        let metadata = self.scene_metadata.get(source_name)
            .cloned()
            .unwrap_or_default();

        let serializable_scene = SceneSerializer::scene_to_serializable(
            source_scene,
            new_name.to_string(),
            Some(metadata),
            Some(GlobalSceneSettings::default())
        );

        // Convert back to runtime scene
        let new_scene = SceneSerializer::serializable_to_scene(&serializable_scene)?;
        
        // Store the duplicated scene
        self.scenes.insert(new_name.to_string(), new_scene);
        self.scene_metadata.insert(new_name.to_string(), serializable_scene.metadata);
        
        log::info!("Duplicated scene '{}' as '{}'", source_name, new_name);
        Ok(())
    }

    /// Update the scene manager (call this every frame)
    pub fn update(&mut self, delta_time: f32) {
        // Handle auto-save
        if self.auto_save_enabled && self.last_save_time.elapsed() > self.auto_save_interval {
            self.auto_save_all_scenes();
            self.last_save_time = std::time::Instant::now();
        }
        
        // Update active scene if any
        if let Some(scene) = self.get_active_scene_mut() {
            // Scene update logic would go here
            // For now, we just track that time passed
        }
    }

    /// Auto-save all scenes that have been modified
    fn auto_save_all_scenes(&mut self) {
        for name in self.scenes.keys().cloned().collect::<Vec<_>>() {
            if let Err(e) = self.save_scene(&name, None) {
                log::error!("Auto-save failed for scene '{}': {}", name, e);
            } else {
                log::debug!("Auto-saved scene: {}", name);
            }
        }
    }

    /// Scan directory for scene files and load them
    pub fn scan_and_load_scenes<P: AsRef<Path>>(&mut self, directory: P) -> SceneResult<Vec<String>> {
        let directory = directory.as_ref();
        let mut loaded_scenes = Vec::new();
        
        if !directory.exists() {
            return Ok(loaded_scenes);
        }

        let entries = std::fs::read_dir(directory)
            .map_err(crate::engine::scene::serialization::SceneSerializationError::IoError)?;

        for entry in entries {
            let entry = entry.map_err(crate::engine::scene::serialization::SceneSerializationError::IoError)?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
                    if matches!(extension, "json" | "toml" | "yaml" | "yml" | "scene") {
                        if let Some(name) = path.file_stem().and_then(|stem| stem.to_str()) {
                            match self.load_scene(name, &path) {
                                Ok(()) => {
                                    loaded_scenes.push(name.to_string());
                                    log::info!("Auto-loaded scene: {}", name);
                                }
                                Err(e) => {
                                    log::warn!("Failed to load scene from {}: {}", path.display(), e);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(loaded_scenes)
    }

    /// Get scene statistics
    pub fn get_statistics(&self) -> SceneManagerStats {
        let total_objects = self.scenes.values()
            .map(|scene| scene.objects().count())
            .sum();

        let active_scene_objects = self.get_active_scene()
            .map(|scene| scene.objects().count())
            .unwrap_or(0);

        SceneManagerStats {
            total_scenes: self.scenes.len(),
            active_scene: self.active_scene.clone(),
            total_objects,
            active_scene_objects,
            auto_save_enabled: self.auto_save_enabled,
            auto_save_interval_seconds: self.auto_save_interval.as_secs(),
            scene_directory: self.scene_directory.clone(),
        }
    }
}

/// Statistics about the scene manager
#[derive(Debug, Clone)]
pub struct SceneManagerStats {
    pub total_scenes: usize,
    pub active_scene: Option<String>,
    pub total_objects: usize,
    pub active_scene_objects: usize,
    pub auto_save_enabled: bool,
    pub auto_save_interval_seconds: u64,
    pub scene_directory: PathBuf,
}

impl std::fmt::Display for SceneManagerStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Scene Manager Statistics:")?;
        writeln!(f, "  Total scenes: {}", self.total_scenes)?;
        writeln!(f, "  Active scene: {}", 
            self.active_scene.as_deref().unwrap_or("None"))?;
        writeln!(f, "  Total objects: {}", self.total_objects)?;
        writeln!(f, "  Active scene objects: {}", self.active_scene_objects)?;
        writeln!(f, "  Auto-save: {} ({}s interval)", 
            if self.auto_save_enabled { "enabled" } else { "disabled" },
            self.auto_save_interval_seconds)?;
        write!(f, "  Scene directory: {}", self.scene_directory.display())
    }
}

/// Scene template system for creating common scene types
pub struct SceneTemplate;

impl SceneTemplate {
    /// Create a 2D platformer scene template
    pub fn create_2d_platformer<'a>(scene_manager: &'a mut SceneManager, name: &str) -> &'a mut Scene {
        let scene = scene_manager.create_scene(name);
        
        // Add ground platform
        let ground = scene.create_object();
        ground.transform.position.x = 0.0;
        ground.transform.position.y = -300.0;
        ground.transform.scale.x = 800.0;
        ground.transform.scale.y = 50.0;
        
        // Add player spawn point
        let player_spawn = scene.create_object();
        player_spawn.transform.position.x = 0.0;
        player_spawn.transform.position.y = 0.0;
        
        log::info!("Created 2D platformer template: {}", name);
        scene
    }

    /// Create a top-down scene template
    pub fn create_top_down<'a>(scene_manager: &'a mut SceneManager, name: &str) -> &'a mut Scene {
        let scene = scene_manager.create_scene(name);
        
        // Add camera anchor
        let camera = scene.create_object();
        camera.transform.position.z = 100.0;
        
        // Add player spawn
        let player_spawn = scene.create_object();
        player_spawn.transform.position.x = 0.0;
        player_spawn.transform.position.y = 0.0;
        
        log::info!("Created top-down template: {}", name);
        scene
    }

    /// Create an empty scene with basic lighting
    pub fn create_basic<'a>(scene_manager: &'a mut SceneManager, name: &str) -> &'a mut Scene {
        let scene = scene_manager.create_scene(name);
        
        // Add ambient light object (placeholder)
        let light = scene.create_object();
        light.transform.position.z = 50.0;
        
        log::info!("Created basic scene template: {}", name);
        scene
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_scene_manager_creation() {
        let mut manager = SceneManager::new();
        let scene = manager.create_scene("test_scene");
        assert!(scene.objects().count() == 0);
        
        let stats = manager.get_statistics();
        assert_eq!(stats.total_scenes, 1);
    }

    #[test]
    fn test_scene_activation() {
        let mut manager = SceneManager::new();
        manager.create_scene("scene1");
        manager.create_scene("scene2");
        
        assert!(manager.set_active_scene("scene1").is_some());
        assert_eq!(manager.get_statistics().active_scene, Some("scene1".to_string()));
        
        assert!(manager.set_active_scene("scene2").is_some());
        assert_eq!(manager.get_statistics().active_scene, Some("scene2".to_string()));
        
        assert!(manager.set_active_scene("nonexistent").is_none());
    }

    #[test]
    fn test_scene_template_creation() {
        let mut manager = SceneManager::new();
        
        let scene = SceneTemplate::create_2d_platformer(&mut manager, "platformer_test");
        assert!(scene.objects().count() >= 2); // Should have ground and player spawn
        
        let scene2 = SceneTemplate::create_top_down(&mut manager, "topdown_test");
        assert!(scene2.objects().count() >= 2); // Should have camera and player spawn
    }

    #[test]
    fn test_scene_duplication() {
        let mut manager = SceneManager::new();
        let original = manager.create_scene("original");
        original.create_object(); // Add an object
        
        manager.duplicate_scene("original", "copy").unwrap();
        
        let stats = manager.get_statistics();
        assert_eq!(stats.total_scenes, 2);
        assert_eq!(stats.total_objects, 2); // Both scenes should have 1 object each
    }
}