/*!
 * Robin Engine - RAII GPU Resource Management
 *
 * Provides safe, automatic resource management for GPU resources:
 * - RAII wrappers for buffers, textures, and render pipelines
 * - Automatic cleanup on drop
 * - Memory usage tracking and validation
 * - Resource pool management for performance
 * - Safe error handling for resource allocation
 */

use crate::error::{RobinError, RobinResult};
use log::{info, warn, debug};
use std::sync::{Arc, Mutex, Weak};
use std::collections::HashMap;
use wgpu::{Device, Buffer, Texture, RenderPipeline, BindGroup, util::DeviceExt};

/// Resource usage statistics for monitoring
#[derive(Debug, Clone, Default)]
pub struct ResourceStats {
    pub total_buffers: u32,
    pub total_textures: u32,
    pub total_pipelines: u32,
    pub total_bind_groups: u32,
    pub memory_used: u64,
    pub peak_memory: u64,
}

/// Resource manager for centralized GPU resource tracking
pub struct GpuResourceManager {
    device: Arc<Device>,
    resource_stats: Arc<Mutex<ResourceStats>>,
    active_buffers: Arc<Mutex<HashMap<usize, Weak<ManagedBuffer>>>>,
    active_textures: Arc<Mutex<HashMap<usize, Weak<ManagedTexture>>>>,
    resource_id_counter: Arc<Mutex<usize>>,
}

/// RAII wrapper for GPU buffers
pub struct ManagedBuffer {
    buffer: Buffer,
    size: u64,
    resource_id: usize,
    manager: Weak<Mutex<GpuResourceManager>>,
    label: String,
}

/// RAII wrapper for GPU textures
pub struct ManagedTexture {
    texture: Texture,
    size: u64,
    resource_id: usize,
    manager: Weak<Mutex<GpuResourceManager>>,
    label: String,
}

/// RAII wrapper for render pipelines
pub struct ManagedRenderPipeline {
    pipeline: RenderPipeline,
    resource_id: usize,
    manager: Weak<Mutex<GpuResourceManager>>,
    label: String,
}

/// RAII wrapper for bind groups
pub struct ManagedBindGroup {
    bind_group: BindGroup,
    resource_id: usize,
    manager: Weak<Mutex<GpuResourceManager>>,
    label: String,
}

impl GpuResourceManager {
    /// Create a new GPU resource manager
    pub fn new(device: Arc<Device>) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            device,
            resource_stats: Arc::new(Mutex::new(ResourceStats::default())),
            active_buffers: Arc::new(Mutex::new(HashMap::new())),
            active_textures: Arc::new(Mutex::new(HashMap::new())),
            resource_id_counter: Arc::new(Mutex::new(0)),
        }))
    }

    /// Create a managed buffer with automatic cleanup
    pub fn create_buffer(
        manager: Arc<Mutex<Self>>,
        descriptor: &wgpu::BufferDescriptor,
    ) -> RobinResult<Arc<ManagedBuffer>> {
        let (buffer, resource_id, size) = {
            let manager_guard = manager.lock().map_err(|_| {
                RobinError::GpuResource {
                    message: "Failed to lock resource manager".to_string(),
                    resource_type: "buffer".to_string(),
                    available_memory: None,
                    requested_memory: Some(descriptor.size),
                }
            })?;

            // Validate buffer size against limits
            if descriptor.size > 4 * 1024 * 1024 * 1024 { // 4GB limit
                return Err(RobinError::GpuResource {
                    message: "Buffer size exceeds 4GB limit".to_string(),
                    resource_type: "buffer".to_string(),
                    available_memory: Some(4 * 1024 * 1024 * 1024),
                    requested_memory: Some(descriptor.size),
                });
            }

            let buffer = manager_guard.device.create_buffer(descriptor);

            let resource_id = {
                let mut counter = manager_guard.resource_id_counter.lock().map_err(|_| {
                    RobinError::GpuResource {
                        message: "Failed to generate resource ID".to_string(),
                        resource_type: "buffer".to_string(),
                        available_memory: None,
                        requested_memory: Some(descriptor.size),
                    }
                })?;
                *counter += 1;
                *counter
            };

            // Update statistics
            if let Ok(mut stats) = manager_guard.resource_stats.lock() {
                stats.total_buffers += 1;
                stats.memory_used += descriptor.size;
                if stats.memory_used > stats.peak_memory {
                    stats.peak_memory = stats.memory_used;
                }
            }

            debug!("🔧 Created buffer '{}' (ID: {}, Size: {} bytes)",
                   descriptor.label.unwrap_or("unnamed"), resource_id, descriptor.size);

            (buffer, resource_id, descriptor.size)
        };

        let managed_buffer = Arc::new(ManagedBuffer {
            buffer,
            size,
            resource_id,
            manager: Arc::downgrade(&manager),
            label: descriptor.label.unwrap_or("unnamed").to_string(),
        });

        // Register the buffer for tracking
        if let Ok(manager_guard) = manager.lock() {
            if let Ok(mut active_buffers) = manager_guard.active_buffers.lock() {
                active_buffers.insert(resource_id, Arc::downgrade(&managed_buffer));
            }
        }

        Ok(managed_buffer)
    }

    /// Create a managed buffer with initial data
    pub fn create_buffer_init(
        manager: Arc<Mutex<Self>>,
        descriptor: &wgpu::util::BufferInitDescriptor,
    ) -> RobinResult<Arc<ManagedBuffer>> {
        let (buffer, resource_id, size) = {
            let manager_guard = manager.lock().map_err(|_| {
                RobinError::GpuResource {
                    message: "Failed to lock resource manager".to_string(),
                    resource_type: "buffer".to_string(),
                    available_memory: None,
                    requested_memory: Some(descriptor.contents.len() as u64),
                }
            })?;

            // Validate buffer size
            let size = descriptor.contents.len() as u64;
            if size > 4 * 1024 * 1024 * 1024 { // 4GB limit
                return Err(RobinError::GpuResource {
                    message: "Buffer size exceeds 4GB limit".to_string(),
                    resource_type: "buffer".to_string(),
                    available_memory: Some(4 * 1024 * 1024 * 1024),
                    requested_memory: Some(size),
                });
            }

            let buffer = manager_guard.device.create_buffer_init(descriptor);

            let resource_id = {
                let mut counter = manager_guard.resource_id_counter.lock().map_err(|_| {
                    RobinError::GpuResource {
                        message: "Failed to generate resource ID".to_string(),
                        resource_type: "buffer".to_string(),
                        available_memory: None,
                        requested_memory: Some(size),
                    }
                })?;
                *counter += 1;
                *counter
            };

            // Update statistics
            if let Ok(mut stats) = manager_guard.resource_stats.lock() {
                stats.total_buffers += 1;
                stats.memory_used += size;
                if stats.memory_used > stats.peak_memory {
                    stats.peak_memory = stats.memory_used;
                }
            }

            debug!("🔧 Created initialized buffer '{}' (ID: {}, Size: {} bytes)",
                   descriptor.label.unwrap_or("unnamed"), resource_id, size);

            (buffer, resource_id, size)
        };

        let managed_buffer = Arc::new(ManagedBuffer {
            buffer,
            size,
            resource_id,
            manager: Arc::downgrade(&manager),
            label: descriptor.label.unwrap_or("unnamed").to_string(),
        });

        // Register the buffer for tracking
        if let Ok(manager_guard) = manager.lock() {
            if let Ok(mut active_buffers) = manager_guard.active_buffers.lock() {
                active_buffers.insert(resource_id, Arc::downgrade(&managed_buffer));
            }
        }

        Ok(managed_buffer)
    }

    /// Create a managed texture with automatic cleanup
    pub fn create_texture(
        manager: Arc<Mutex<Self>>,
        descriptor: &wgpu::TextureDescriptor,
    ) -> RobinResult<Arc<ManagedTexture>> {
        let (texture, resource_id, size) = {
            let manager_guard = manager.lock().map_err(|_| {
                RobinError::GpuResource {
                    message: "Failed to lock resource manager".to_string(),
                    resource_type: "texture".to_string(),
                    available_memory: None,
                    requested_memory: None,
                }
            })?;

            // Calculate approximate texture memory usage
            let bytes_per_pixel = match descriptor.format {
                wgpu::TextureFormat::Rgba8Unorm => 4,
                wgpu::TextureFormat::Rgba8UnormSrgb => 4,
                wgpu::TextureFormat::Rg8Unorm => 2,
                wgpu::TextureFormat::R8Unorm => 1,
                wgpu::TextureFormat::Rgba16Float => 8,
                wgpu::TextureFormat::Rgba32Float => 16,
                _ => 4, // Default estimate
            };

            let size = (descriptor.size.width as u64) *
                      (descriptor.size.height as u64) *
                      (descriptor.size.depth_or_array_layers as u64) *
                      (bytes_per_pixel as u64);

            let texture = manager_guard.device.create_texture(descriptor);

            let resource_id = {
                let mut counter = manager_guard.resource_id_counter.lock().map_err(|_| {
                    RobinError::GpuResource {
                        message: "Failed to generate resource ID".to_string(),
                        resource_type: "texture".to_string(),
                        available_memory: None,
                        requested_memory: Some(size),
                    }
                })?;
                *counter += 1;
                *counter
            };

            // Update statistics
            if let Ok(mut stats) = manager_guard.resource_stats.lock() {
                stats.total_textures += 1;
                stats.memory_used += size;
                if stats.memory_used > stats.peak_memory {
                    stats.peak_memory = stats.memory_used;
                }
            }

            debug!("🖼️ Created texture '{}' (ID: {}, Size: {}x{}x{}, Memory: {} bytes)",
                   descriptor.label.unwrap_or("unnamed"), resource_id,
                   descriptor.size.width, descriptor.size.height, descriptor.size.depth_or_array_layers,
                   size);

            (texture, resource_id, size)
        };

        let managed_texture = Arc::new(ManagedTexture {
            texture,
            size,
            resource_id,
            manager: Arc::downgrade(&manager),
            label: descriptor.label.unwrap_or("unnamed").to_string(),
        });

        // Register the texture for tracking
        if let Ok(manager_guard) = manager.lock() {
            if let Ok(mut active_textures) = manager_guard.active_textures.lock() {
                active_textures.insert(resource_id, Arc::downgrade(&managed_texture));
            }
        }

        Ok(managed_texture)
    }

    /// Get current resource statistics
    pub fn get_stats(&self) -> ResourceStats {
        match self.resource_stats.lock() {
            Ok(stats) => stats.clone(),
            Err(_) => ResourceStats::default()
        }
    }

    /// Clean up dead weak references
    pub fn cleanup_dead_references(&self) {
        if let Ok(mut buffers) = self.active_buffers.lock() {
            buffers.retain(|_, weak_ref| weak_ref.strong_count() > 0);
        }

        if let Ok(mut textures) = self.active_textures.lock() {
            textures.retain(|_, weak_ref| weak_ref.strong_count() > 0);
        }
    }

    /// Get a summary of resource usage
    pub fn get_usage_summary(&self) -> String {
        let stats = self.get_stats();
        format!(
            "🔧 GPU Resources: {} buffers, {} textures, {} pipelines, {} bind groups | Memory: {:.2} MB (Peak: {:.2} MB)",
            stats.total_buffers,
            stats.total_textures,
            stats.total_pipelines,
            stats.total_bind_groups,
            stats.memory_used as f64 / 1024.0 / 1024.0,
            stats.peak_memory as f64 / 1024.0 / 1024.0
        )
    }
}

impl ManagedBuffer {
    /// Get a reference to the underlying buffer
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    /// Get the buffer size
    pub fn size(&self) -> u64 {
        self.size
    }

    /// Get the resource ID for debugging
    pub fn resource_id(&self) -> usize {
        self.resource_id
    }

    /// Get buffer label
    pub fn label(&self) -> &str {
        &self.label
    }
}

impl ManagedTexture {
    /// Get a reference to the underlying texture
    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    /// Get the texture memory size
    pub fn size(&self) -> u64 {
        self.size
    }

    /// Get the resource ID for debugging
    pub fn resource_id(&self) -> usize {
        self.resource_id
    }

    /// Get texture label
    pub fn label(&self) -> &str {
        &self.label
    }
}

// RAII cleanup implementation
impl Drop for ManagedBuffer {
    fn drop(&mut self) {
        debug!("🗑️ Dropping buffer '{}' (ID: {}, Size: {} bytes)",
               self.label, self.resource_id, self.size);

        // Update statistics when the buffer is dropped
        if let Some(manager) = self.manager.upgrade() {
            if let Ok(manager_guard) = manager.lock() {
                if let Ok(mut stats) = manager_guard.resource_stats.lock() {
                    stats.total_buffers = stats.total_buffers.saturating_sub(1);
                    stats.memory_used = stats.memory_used.saturating_sub(self.size);
                }

                // Remove from active tracking
                if let Ok(mut active_buffers) = manager_guard.active_buffers.lock() {
                    active_buffers.remove(&self.resource_id);
                }
            }
        }
    }
}

impl Drop for ManagedTexture {
    fn drop(&mut self) {
        debug!("🗑️ Dropping texture '{}' (ID: {}, Size: {} bytes)",
               self.label, self.resource_id, self.size);

        // Update statistics when the texture is dropped
        if let Some(manager) = self.manager.upgrade() {
            if let Ok(manager_guard) = manager.lock() {
                if let Ok(mut stats) = manager_guard.resource_stats.lock() {
                    stats.total_textures = stats.total_textures.saturating_sub(1);
                    stats.memory_used = stats.memory_used.saturating_sub(self.size);
                }

                // Remove from active tracking
                if let Ok(mut active_textures) = manager_guard.active_textures.lock() {
                    active_textures.remove(&self.resource_id);
                }
            }
        }
    }
}

/// Convenience trait for creating managed resources
pub trait ManagedResourceExt {
    fn create_managed_buffer(
        &self,
        descriptor: &wgpu::BufferDescriptor,
    ) -> RobinResult<Arc<ManagedBuffer>>;

    fn create_managed_buffer_init(
        &self,
        descriptor: &wgpu::util::BufferInitDescriptor,
    ) -> RobinResult<Arc<ManagedBuffer>>;

    fn create_managed_texture(
        &self,
        descriptor: &wgpu::TextureDescriptor,
    ) -> RobinResult<Arc<ManagedTexture>>;
}

impl ManagedResourceExt for Arc<Mutex<GpuResourceManager>> {
    fn create_managed_buffer(
        &self,
        descriptor: &wgpu::BufferDescriptor,
    ) -> RobinResult<Arc<ManagedBuffer>> {
        GpuResourceManager::create_buffer(self.clone(), descriptor)
    }

    fn create_managed_buffer_init(
        &self,
        descriptor: &wgpu::util::BufferInitDescriptor,
    ) -> RobinResult<Arc<ManagedBuffer>> {
        GpuResourceManager::create_buffer_init(self.clone(), descriptor)
    }

    fn create_managed_texture(
        &self,
        descriptor: &wgpu::TextureDescriptor,
    ) -> RobinResult<Arc<ManagedTexture>> {
        GpuResourceManager::create_texture(self.clone(), descriptor)
    }
}