/*!
 * Robin Engine Plugin System
 *
 * Dynamic plugin loading and management system supporting hot-swappable
 * modules, dependency resolution, and cross-platform plugin architecture.
 */

use crate::engine::error::{RobinResult, RobinError};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::any::{Any, TypeId};
use serde::{Serialize, Deserialize};
use libloading::{Library, Symbol};

/// Core plugin management system
#[derive(Debug)]
pub struct PluginManager {
    config: PluginConfig,
    registry: Arc<RwLock<PluginRegistry>>,
    loader: PluginLoader,
    loaded_plugins: HashMap<PluginId, LoadedPlugin>,
    dependency_resolver: DependencyResolver,
    event_bus: PluginEventBus,
    security_manager: PluginSecurityManager,
}

impl PluginManager {
    pub fn new() -> RobinResult<Self> {
        let config = PluginConfig::default();
        let registry = Arc::new(RwLock::new(PluginRegistry::new()));

        Ok(Self {
            config,
            registry: registry.clone(),
            loader: PluginLoader::new(registry.clone())?,
            loaded_plugins: HashMap::new(),
            dependency_resolver: DependencyResolver::new(),
            event_bus: PluginEventBus::new(),
            security_manager: PluginSecurityManager::new()?,
        })
    }

    /// Scan and load all plugins from configured directories
    pub fn scan_and_load_plugins(&mut self) -> RobinResult<PluginScanResult> {
        let mut result = PluginScanResult::new();

        for plugin_dir in &self.config.plugin_directories {
            if plugin_dir.exists() {
                result.merge(self.scan_directory(plugin_dir)?);
            }
        }

        // Resolve dependencies and load plugins in correct order
        let load_order = self.dependency_resolver.resolve_load_order(&result.discovered_plugins)?;

        for plugin_info in load_order {
            match self.load_plugin(&plugin_info) {
                Ok(plugin_id) => {
                    result.loaded_plugins.push(plugin_id);
                    result.loaded_count += 1;
                }
                Err(e) => {
                    result.failed_plugins.push((plugin_info.manifest.id.clone(), e));
                    result.failed_count += 1;
                }
            }
        }

        println!("🔌 Plugin Manager: Loaded {}/{} plugins", result.loaded_count, result.discovered_count);
        Ok(result)
    }

    /// Load a specific plugin
    pub fn load_plugin(&mut self, plugin_info: &PluginInfo) -> RobinResult<PluginId> {
        // Security check
        self.security_manager.validate_plugin(&plugin_info.manifest)?;

        // Load the plugin library
        let loaded_plugin = self.loader.load_plugin(&plugin_info.path)?;

        // Initialize the plugin
        let plugin_instance = loaded_plugin.create_instance()?;
        plugin_instance.initialize(&self.create_plugin_context())?;

        // Register with the system
        let plugin_id = plugin_info.manifest.id.clone();
        self.loaded_plugins.insert(plugin_id.clone(), loaded_plugin);

        {
            let mut registry = self.registry.write().unwrap();
            registry.register_plugin(plugin_id.clone(), plugin_info.clone())?;
        }

        // Notify other plugins
        self.event_bus.publish(PluginEvent::PluginLoaded(plugin_id.clone()));

        println!("✅ Loaded plugin: {} v{}", plugin_info.manifest.name, plugin_info.manifest.version);
        Ok(plugin_id)
    }

    /// Unload a plugin
    pub fn unload_plugin(&mut self, plugin_id: &PluginId) -> RobinResult<()> {
        if let Some(loaded_plugin) = self.loaded_plugins.remove(plugin_id) {
            // Shutdown the plugin
            loaded_plugin.instance.shutdown()?;

            // Unregister from the system
            {
                let mut registry = self.registry.write().unwrap();
                registry.unregister_plugin(plugin_id)?;
            }

            // Notify other plugins
            self.event_bus.publish(PluginEvent::PluginUnloaded(plugin_id.clone()));

            println!("🔌 Unloaded plugin: {}", plugin_id);
        }

        Ok(())
    }

    /// Reload a plugin (hot-swapping)
    pub fn reload_plugin(&mut self, plugin_id: &PluginId) -> RobinResult<()> {
        let registry = self.registry.read().unwrap();
        if let Some(plugin_info) = registry.get_plugin_info(plugin_id) {
            let plugin_info = plugin_info.clone();
            drop(registry);

            // Unload the existing plugin
            self.unload_plugin(plugin_id)?;

            // Reload the plugin
            self.load_plugin(&plugin_info)?;

            println!("🔄 Reloaded plugin: {}", plugin_id);
        }

        Ok(())
    }

    /// Get a plugin interface
    pub fn get_plugin<T: Plugin + 'static>(&self, plugin_id: &PluginId) -> Option<Arc<T>> {
        self.loaded_plugins.get(plugin_id)
            .and_then(|loaded_plugin| loaded_plugin.instance.as_any().downcast_ref::<T>())
            .map(|plugin| Arc::new(unsafe { std::ptr::read(plugin) }))
    }

    /// Get all plugins implementing a specific interface
    pub fn get_plugins_by_interface<T: Plugin + 'static>(&self) -> Vec<Arc<T>> {
        self.loaded_plugins.values()
            .filter_map(|loaded_plugin| {
                loaded_plugin.instance.as_any().downcast_ref::<T>()
                    .map(|plugin| Arc::new(unsafe { std::ptr::read(plugin) }))
            })
            .collect()
    }

    /// Update all loaded plugins
    pub fn update(&mut self, delta_time: f32) -> RobinResult<()> {
        // Process plugin events
        self.event_bus.process_events()?;

        // Update all plugins
        for (plugin_id, loaded_plugin) in &mut self.loaded_plugins {
            if let Err(e) = loaded_plugin.instance.update(delta_time) {
                eprintln!("⚠️ Plugin update failed for {}: {}", plugin_id, e);
            }
        }

        Ok(())
    }

    /// Shutdown the plugin system
    pub fn shutdown(&mut self) -> RobinResult<()> {
        // Unload all plugins in reverse dependency order
        let plugin_ids: Vec<_> = self.loaded_plugins.keys().cloned().collect();

        for plugin_id in plugin_ids.into_iter().rev() {
            self.unload_plugin(&plugin_id)?;
        }

        println!("🛑 Plugin Manager: Shutdown complete");
        Ok(())
    }

    fn scan_directory(&self, directory: &Path) -> RobinResult<PluginScanResult> {
        let mut result = PluginScanResult::new();

        for entry in std::fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Look for plugin manifest
                let manifest_path = path.join("plugin.toml");
                if manifest_path.exists() {
                    match self.load_plugin_manifest(&manifest_path) {
                        Ok(manifest) => {
                            let plugin_info = PluginInfo {
                                manifest,
                                path: path.clone(),
                                manifest_path,
                            };
                            result.discovered_plugins.push(plugin_info);
                            result.discovered_count += 1;
                        }
                        Err(e) => {
                            result.failed_manifests.push((path, e));
                        }
                    }
                }
            }
        }

        Ok(result)
    }

    fn load_plugin_manifest(&self, manifest_path: &Path) -> RobinResult<PluginManifest> {
        let manifest_content = std::fs::read_to_string(manifest_path)?;
        let manifest: PluginManifest = toml::from_str(&manifest_content)
            .map_err(|e| RobinError::Plugin(format!("Failed to parse plugin manifest: {}", e)))?;

        Ok(manifest)
    }

    fn create_plugin_context(&self) -> PluginContext {
        PluginContext {
            registry: self.registry.clone(),
            event_bus: self.event_bus.create_sender(),
        }
    }
}

/// Plugin configuration
#[derive(Debug, Clone)]
pub struct PluginConfig {
    pub plugin_directories: Vec<PathBuf>,
    pub enable_hot_reload: bool,
    pub security_level: SecurityLevel,
    pub max_plugins: usize,
    pub plugin_timeout: std::time::Duration,
    pub sandbox_enabled: bool,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            plugin_directories: vec![
                PathBuf::from("plugins"),
                PathBuf::from("addons"),
                PathBuf::from("extensions"),
            ],
            enable_hot_reload: true,
            security_level: SecurityLevel::Medium,
            max_plugins: 64,
            plugin_timeout: std::time::Duration::from_secs(30),
            sandbox_enabled: true,
        }
    }
}

/// Security levels for plugin loading
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityLevel {
    Low,
    Medium,
    High,
    Strict,
}

/// Plugin unique identifier
pub type PluginId = String;

/// Plugin manifest loaded from plugin.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: PluginId,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub dependencies: Vec<PluginDependency>,
    pub provides: Vec<String>,
    pub permissions: Vec<Permission>,
    pub entry_point: String,
    pub supported_platforms: Vec<String>,
    pub min_engine_version: String,
    pub max_engine_version: Option<String>,
}

/// Plugin dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    pub id: PluginId,
    pub version: String,
    pub optional: bool,
}

/// Plugin permission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Permission {
    FileSystem(FileSystemPermission),
    Network(NetworkPermission),
    Graphics,
    Audio,
    Input,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSystemPermission {
    pub read: Vec<String>,
    pub write: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPermission {
    pub hosts: Vec<String>,
    pub ports: Vec<u16>,
}

/// Plugin information
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub manifest: PluginManifest,
    pub path: PathBuf,
    pub manifest_path: PathBuf,
}

/// Plugin scan result
#[derive(Debug)]
pub struct PluginScanResult {
    pub discovered_count: usize,
    pub loaded_count: usize,
    pub failed_count: usize,
    pub discovered_plugins: Vec<PluginInfo>,
    pub loaded_plugins: Vec<PluginId>,
    pub failed_plugins: Vec<(PluginId, RobinError)>,
    pub failed_manifests: Vec<(PathBuf, RobinError)>,
}

impl PluginScanResult {
    pub fn new() -> Self {
        Self {
            discovered_count: 0,
            loaded_count: 0,
            failed_count: 0,
            discovered_plugins: Vec::new(),
            loaded_plugins: Vec::new(),
            failed_plugins: Vec::new(),
            failed_manifests: Vec::new(),
        }
    }

    pub fn merge(&mut self, other: PluginScanResult) {
        self.discovered_count += other.discovered_count;
        self.loaded_count += other.loaded_count;
        self.failed_count += other.failed_count;
        self.discovered_plugins.extend(other.discovered_plugins);
        self.loaded_plugins.extend(other.loaded_plugins);
        self.failed_plugins.extend(other.failed_plugins);
        self.failed_manifests.extend(other.failed_manifests);
    }
}

/// Main plugin trait that all plugins must implement
pub trait Plugin: Send + Sync {
    fn initialize(&mut self, context: &PluginContext) -> RobinResult<()>;
    fn shutdown(&mut self) -> RobinResult<()>;
    fn update(&mut self, delta_time: f32) -> RobinResult<()>;
    fn get_info(&self) -> PluginInfo;
    fn as_any(&self) -> &dyn Any;
}

/// Plugin context provided to plugins for engine interaction
#[derive(Debug, Clone)]
pub struct PluginContext {
    pub registry: Arc<RwLock<PluginRegistry>>,
    pub event_bus: PluginEventSender,
}

/// Plugin registry for tracking loaded plugins
#[derive(Debug)]
pub struct PluginRegistry {
    plugins: HashMap<PluginId, PluginInfo>,
    interfaces: HashMap<TypeId, Vec<PluginId>>,
    services: HashMap<String, PluginId>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            interfaces: HashMap::new(),
            services: HashMap::new(),
        }
    }

    pub fn register_plugin(&mut self, id: PluginId, info: PluginInfo) -> RobinResult<()> {
        self.plugins.insert(id, info);
        Ok(())
    }

    pub fn unregister_plugin(&mut self, id: &PluginId) -> RobinResult<()> {
        self.plugins.remove(id);
        self.services.retain(|_, plugin_id| plugin_id != id);

        for interface_plugins in self.interfaces.values_mut() {
            interface_plugins.retain(|plugin_id| plugin_id != id);
        }

        Ok(())
    }

    pub fn get_plugin_info(&self, id: &PluginId) -> Option<&PluginInfo> {
        self.plugins.get(id)
    }

    pub fn register_service(&mut self, service_name: String, plugin_id: PluginId) {
        self.services.insert(service_name, plugin_id);
    }

    pub fn get_service_provider(&self, service_name: &str) -> Option<&PluginId> {
        self.services.get(service_name)
    }

    pub fn get_all_plugins(&self) -> Vec<&PluginInfo> {
        self.plugins.values().collect()
    }
}

/// Plugin loader for dynamic library loading
#[derive(Debug)]
pub struct PluginLoader {
    registry: Arc<RwLock<PluginRegistry>>,
}

impl PluginLoader {
    pub fn new(registry: Arc<RwLock<PluginRegistry>>) -> RobinResult<Self> {
        Ok(Self { registry })
    }

    pub fn load_plugin(&self, plugin_path: &Path) -> RobinResult<LoadedPlugin> {
        // Determine library path based on platform
        let library_path = self.find_plugin_library(plugin_path)?;

        // Load the dynamic library
        let library = unsafe {
            Library::new(&library_path)
                .map_err(|e| RobinError::Plugin(format!("Failed to load plugin library: {}", e)))?
        };

        // Get the plugin creation function
        let create_plugin: Symbol<extern "C" fn() -> *mut dyn Plugin> = unsafe {
            library.get(b"create_plugin")
                .map_err(|e| RobinError::Plugin(format!("Plugin missing create_plugin function: {}", e)))?
        };

        // Create the plugin instance
        let plugin_ptr = create_plugin();
        let plugin_instance = unsafe { Box::from_raw(plugin_ptr) };

        Ok(LoadedPlugin {
            library,
            instance: plugin_instance,
        })
    }

    fn find_plugin_library(&self, plugin_path: &Path) -> RobinResult<PathBuf> {
        let platform = crate::engine::platform::Platform::detect_current();
        let extension = platform.get_library_extension();

        // Look for the library file
        for entry in std::fs::read_dir(plugin_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().map_or(false, |ext| {
                format!(".{}", ext.to_string_lossy()) == extension
            }) {
                return Ok(path);
            }
        }

        Err(RobinError::Plugin(format!(
            "No plugin library found in {:?} with extension {}",
            plugin_path, extension
        )))
    }
}

/// Loaded plugin instance
#[derive(Debug)]
pub struct LoadedPlugin {
    #[allow(dead_code)]
    library: Library,
    instance: Box<dyn Plugin>,
}

impl LoadedPlugin {
    pub fn create_instance(&self) -> RobinResult<&dyn Plugin> {
        Ok(self.instance.as_ref())
    }
}

/// Dependency resolver for determining plugin load order
#[derive(Debug)]
pub struct DependencyResolver;

impl DependencyResolver {
    pub fn new() -> Self {
        Self
    }

    pub fn resolve_load_order(&self, plugins: &[PluginInfo]) -> RobinResult<Vec<PluginInfo>> {
        // Simple topological sort implementation
        let mut sorted = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut temp_visited = std::collections::HashSet::new();

        for plugin in plugins {
            if !visited.contains(&plugin.manifest.id) {
                self.visit_plugin(plugin, plugins, &mut visited, &mut temp_visited, &mut sorted)?;
            }
        }

        Ok(sorted)
    }

    fn visit_plugin(
        &self,
        plugin: &PluginInfo,
        all_plugins: &[PluginInfo],
        visited: &mut std::collections::HashSet<PluginId>,
        temp_visited: &mut std::collections::HashSet<PluginId>,
        sorted: &mut Vec<PluginInfo>,
    ) -> RobinResult<()> {
        if temp_visited.contains(&plugin.manifest.id) {
            return Err(RobinError::Plugin(format!(
                "Circular dependency detected involving plugin: {}",
                plugin.manifest.id
            )));
        }

        if visited.contains(&plugin.manifest.id) {
            return Ok(());
        }

        temp_visited.insert(plugin.manifest.id.clone());

        // Visit dependencies first
        for dependency in &plugin.manifest.dependencies {
            if let Some(dep_plugin) = all_plugins.iter().find(|p| p.manifest.id == dependency.id) {
                self.visit_plugin(dep_plugin, all_plugins, visited, temp_visited, sorted)?;
            } else if !dependency.optional {
                return Err(RobinError::Plugin(format!(
                    "Required dependency not found: {} for plugin {}",
                    dependency.id, plugin.manifest.id
                )));
            }
        }

        temp_visited.remove(&plugin.manifest.id);
        visited.insert(plugin.manifest.id.clone());
        sorted.push(plugin.clone());

        Ok(())
    }
}

/// Plugin event system
#[derive(Debug)]
pub struct PluginEventBus {
    events: Arc<Mutex<Vec<PluginEvent>>>,
}

impl PluginEventBus {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn publish(&self, event: PluginEvent) {
        let mut events = self.events.lock().unwrap();
        events.push(event);
    }

    pub fn process_events(&self) -> RobinResult<()> {
        let mut events = self.events.lock().unwrap();
        let current_events = std::mem::take(&mut *events);
        drop(events);

        for event in current_events {
            self.handle_event(event)?;
        }

        Ok(())
    }

    pub fn create_sender(&self) -> PluginEventSender {
        PluginEventSender {
            events: self.events.clone(),
        }
    }

    fn handle_event(&self, event: PluginEvent) -> RobinResult<()> {
        match event {
            PluginEvent::PluginLoaded(plugin_id) => {
                println!("📢 Plugin loaded event: {}", plugin_id);
            }
            PluginEvent::PluginUnloaded(plugin_id) => {
                println!("📢 Plugin unloaded event: {}", plugin_id);
            }
            PluginEvent::ServiceRegistered(service_name, plugin_id) => {
                println!("📢 Service '{}' registered by plugin: {}", service_name, plugin_id);
            }
        }

        Ok(())
    }
}

/// Plugin event sender for plugins to publish events
#[derive(Debug, Clone)]
pub struct PluginEventSender {
    events: Arc<Mutex<Vec<PluginEvent>>>,
}

impl PluginEventSender {
    pub fn send(&self, event: PluginEvent) {
        let mut events = self.events.lock().unwrap();
        events.push(event);
    }
}

/// Plugin events
#[derive(Debug, Clone)]
pub enum PluginEvent {
    PluginLoaded(PluginId),
    PluginUnloaded(PluginId),
    ServiceRegistered(String, PluginId),
}

/// Plugin security manager
#[derive(Debug)]
pub struct PluginSecurityManager {
    security_level: SecurityLevel,
}

impl PluginSecurityManager {
    pub fn new() -> RobinResult<Self> {
        Ok(Self {
            security_level: SecurityLevel::Medium,
        })
    }

    pub fn validate_plugin(&self, manifest: &PluginManifest) -> RobinResult<()> {
        match self.security_level {
            SecurityLevel::Low => Ok(()),
            SecurityLevel::Medium => self.validate_medium_security(manifest),
            SecurityLevel::High => self.validate_high_security(manifest),
            SecurityLevel::Strict => self.validate_strict_security(manifest),
        }
    }

    fn validate_medium_security(&self, manifest: &PluginManifest) -> RobinResult<()> {
        // Check for dangerous permissions
        for permission in &manifest.permissions {
            match permission {
                Permission::System => {
                    return Err(RobinError::Plugin(format!(
                        "Plugin {} requires system permissions which are not allowed",
                        manifest.id
                    )));
                }
                _ => {} // Other permissions are allowed
            }
        }

        Ok(())
    }

    fn validate_high_security(&self, manifest: &PluginManifest) -> RobinResult<()> {
        self.validate_medium_security(manifest)?;

        // Additional checks for high security
        if manifest.permissions.iter().any(|p| matches!(p, Permission::Network(_))) {
            return Err(RobinError::Plugin(format!(
                "Plugin {} requires network permissions which are not allowed in high security mode",
                manifest.id
            )));
        }

        Ok(())
    }

    fn validate_strict_security(&self, manifest: &PluginManifest) -> RobinResult<()> {
        self.validate_high_security(manifest)?;

        // In strict mode, only allow graphics, audio, and minimal file system access
        for permission in &manifest.permissions {
            match permission {
                Permission::Graphics | Permission::Audio => {} // Allowed
                Permission::FileSystem(fs_perm) => {
                    if !fs_perm.write.is_empty() {
                        return Err(RobinError::Plugin(format!(
                            "Plugin {} requires write permissions which are not allowed in strict mode",
                            manifest.id
                        )));
                    }
                }
                _ => {
                    return Err(RobinError::Plugin(format!(
                        "Plugin {} requires permissions that are not allowed in strict security mode",
                        manifest.id
                    )));
                }
            }
        }

        Ok(())
    }
}

/// Example plugin interfaces
pub trait RenderPlugin: Plugin {
    fn render(&self, context: &RenderContext) -> RobinResult<()>;
}

pub trait AudioPlugin: Plugin {
    fn process_audio(&self, buffer: &mut [f32]) -> RobinResult<()>;
}

pub trait GameLogicPlugin: Plugin {
    fn update_game_logic(&self, delta_time: f32) -> RobinResult<()>;
}

/// Render context for render plugins
#[derive(Debug)]
pub struct RenderContext {
    // Placeholder for render context data
}

/// Macro for easier plugin creation
#[macro_export]
macro_rules! create_plugin {
    ($plugin_type:ty) => {
        #[no_mangle]
        pub extern "C" fn create_plugin() -> *mut dyn Plugin {
            let plugin = <$plugin_type>::new();
            Box::into_raw(Box::new(plugin)) as *mut dyn Plugin
        }
    };
}