/*!
 * Robin Engine GPU Acceleration System
 * 
 * High-performance GPU-based computation for procedural generation,
 * physics simulation, and rendering optimization using compute shaders.
 */

use crate::engine::{
    graphics::{GraphicsContext, Texture},
    error::{RobinError, RobinResult},
};
use wgpu::Buffer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod compute;
pub mod buffers;
pub mod shaders;
pub mod memory;
pub mod integration;

pub use compute::*;
pub use buffers::*;
pub use shaders::*;
pub use memory::*;
pub use integration::*;

/// Main GPU acceleration system
#[derive(Debug)]
pub struct GPUAccelerationSystem {
    config: GPUConfig,
    /// Compute shader manager
    compute_manager: ComputeManager,
    /// GPU memory manager
    memory_manager: GPUMemoryManager,
    /// Buffer pool for efficient memory reuse
    buffer_pool: BufferPool,
    /// Shader cache for fast loading
    shader_cache: ShaderCache,
    /// GPU profiler
    profiler: GPUProfiler,
    /// Device capabilities
    device_caps: DeviceCapabilities,
}

impl GPUAccelerationSystem {
    pub fn new(graphics_context: &GraphicsContext, config: GPUConfig) -> RobinResult<Self> {
        let device_caps = DeviceCapabilities::query(graphics_context)?;
        
        // Validate compute shader support
        if !device_caps.supports_compute_shaders {
            return Err(RobinError::GPUError("Compute shaders not supported on this device".to_string()));
        }

        let mut system = Self {
            compute_manager: ComputeManager::new(graphics_context, &device_caps)?,
            memory_manager: GPUMemoryManager::new(graphics_context, &device_caps, config.memory_config.clone())?,
            buffer_pool: BufferPool::new(config.buffer_pool_config.clone()),
            shader_cache: ShaderCache::new(),
            profiler: GPUProfiler::new(),
            device_caps,
            config,
        };

        // Load built-in compute shaders
        system.load_builtin_shaders(graphics_context)?;

        Ok(system)
    }

    /// Initialize GPU acceleration for the engine
    pub fn initialize(&mut self, graphics_context: &GraphicsContext) -> RobinResult<()> {
        self.profiler.start_frame();

        // Pre-allocate common buffers
        self.preallocate_buffers(graphics_context)?;

        // Warm up shaders
        self.warmup_shaders(graphics_context)?;

        Ok(())
    }

    /// Dispatch a compute shader
    pub fn dispatch_compute(&mut self, graphics_context: &GraphicsContext, dispatch: ComputeDispatch) -> RobinResult<ComputeResult> {
        self.profiler.start_dispatch(&dispatch.shader_name);
        
        let result = self.compute_manager.dispatch(graphics_context, dispatch)?;
        
        self.profiler.end_dispatch(&result.shader_name, result.execution_time);
        Ok(result)
    }

    /// Get GPU memory statistics
    pub fn get_memory_stats(&self) -> GPUMemoryStats {
        let detailed_stats = self.memory_manager.get_stats();
        // Convert detailed memory stats to simplified GPU stats
        GPUMemoryStats {
            total_allocated: detailed_stats.total_memory_used as u64 + detailed_stats.total_memory_free as u64,
            total_used: detailed_stats.total_memory_used as u64,
            buffer_memory: detailed_stats.buffer_memory as u64,
            texture_memory: detailed_stats.texture_memory as u64,
            fragmentation_ratio: detailed_stats.fragmentation_ratio,
            allocation_count: detailed_stats.total_allocations as u32,
        }
    }

    /// Get GPU performance metrics
    pub fn get_performance_metrics(&self) -> &GPUPerformanceMetrics {
        self.profiler.get_metrics()
    }

    /// Synchronize GPU operations (blocking)
    pub fn sync(&self, graphics_context: &GraphicsContext) -> RobinResult<()> {
        self.compute_manager.sync(graphics_context)
    }

    /// Begin frame for GPU operations
    pub fn begin_frame(&mut self) {
        self.profiler.start_frame();
        self.buffer_pool.begin_frame();
        self.memory_manager.begin_frame();
    }

    /// End frame for GPU operations
    pub fn end_frame(&mut self, graphics_context: &GraphicsContext) -> RobinResult<()> {
        self.buffer_pool.end_frame(graphics_context)?;
        self.memory_manager.end_frame(graphics_context)?;
        self.profiler.end_frame();
        Ok(())
    }

    /// Create a GPU buffer
    pub fn create_buffer(&mut self, graphics_context: &GraphicsContext, desc: BufferDescriptor) -> RobinResult<GPUBufferHandle> {
        self.buffer_pool.create_buffer(graphics_context, desc)
    }

    /// Map buffer for CPU access
    pub fn map_buffer<T>(&mut self, graphics_context: &GraphicsContext, handle: GPUBufferHandle) -> RobinResult<BufferMapping<T>> {
        self.buffer_pool.map_buffer(graphics_context, handle)
    }

    /// Unmap buffer
    pub fn unmap_buffer(&mut self, graphics_context: &GraphicsContext, handle: GPUBufferHandle) -> RobinResult<()> {
        self.buffer_pool.unmap_buffer(graphics_context, handle)
    }

    /// Copy data to GPU buffer
    pub fn upload_buffer_data<T>(&mut self, graphics_context: &GraphicsContext, handle: GPUBufferHandle, data: &[T]) -> RobinResult<()> {
        self.buffer_pool.upload_data(graphics_context, handle, data)
    }

    /// Copy data from GPU buffer
    pub fn download_buffer_data<T>(&mut self, graphics_context: &GraphicsContext, handle: GPUBufferHandle, data: &mut [T]) -> RobinResult<()> {
        self.buffer_pool.download_data(graphics_context, handle, data)
    }

    /// Load a compute shader
    pub fn load_compute_shader(&mut self, graphics_context: &GraphicsContext, name: String, source: ComputeShaderSource) -> RobinResult<()> {
        self.shader_cache.load_compute_shader(graphics_context, name, source)
    }

    /// Check if device supports required features
    pub fn check_device_support(&self, requirements: &DeviceRequirements) -> bool {
        self.device_caps.meets_requirements(requirements)
    }

    /// Get optimal work group size for a shader
    pub fn get_optimal_work_group_size(&self, shader_name: &str) -> RobinResult<(u32, u32, u32)> {
        self.compute_manager.get_optimal_work_group_size(shader_name)
    }

    /// Create GPU texture for compute operations
    pub fn create_compute_texture(&mut self, graphics_context: &GraphicsContext, desc: TextureDescriptor) -> RobinResult<GPUTextureHandle> {
        self.memory_manager.create_compute_texture(graphics_context, desc)
    }

    fn load_builtin_shaders(&mut self, graphics_context: &GraphicsContext) -> RobinResult<()> {
        // Load noise generation shaders
        self.load_compute_shader(graphics_context, "perlin_noise_2d".to_string(), ComputeShaderSource::GLSL(include_str!("shaders/perlin_noise_2d.comp").to_string()))?;
        self.load_compute_shader(graphics_context, "perlin_noise_3d".to_string(), ComputeShaderSource::GLSL(include_str!("shaders/perlin_noise_3d.comp").to_string()))?;
        self.load_compute_shader(graphics_context, "simplex_noise_2d".to_string(), ComputeShaderSource::GLSL(include_str!("shaders/simplex_noise_2d.comp").to_string()))?;
        self.load_compute_shader(graphics_context, "simplex_noise_3d".to_string(), ComputeShaderSource::GLSL(include_str!("shaders/simplex_noise_3d.comp").to_string()))?;

        // Load voxel processing shaders
        self.load_compute_shader(graphics_context, "voxel_marching_cubes".to_string(), ComputeShaderSource::GLSL(include_str!("shaders/voxel_marching_cubes.comp").to_string()))?;
        self.load_compute_shader(graphics_context, "voxel_meshing".to_string(), ComputeShaderSource::GLSL(include_str!("shaders/voxel_meshing.comp").to_string()))?;
        self.load_compute_shader(graphics_context, "voxel_culling".to_string(), ComputeShaderSource::GLSL(include_str!("shaders/voxel_culling.comp").to_string()))?;

        // Load particle system shaders
        self.load_compute_shader(graphics_context, "particle_update".to_string(), ComputeShaderSource::GLSL(include_str!("shaders/particle_update.comp").to_string()))?;
        self.load_compute_shader(graphics_context, "particle_emission".to_string(), ComputeShaderSource::GLSL(include_str!("shaders/particle_emission.comp").to_string()))?;
        self.load_compute_shader(graphics_context, "particle_collisions".to_string(), ComputeShaderSource::GLSL(include_str!("shaders/particle_collisions.comp").to_string()))?;

        // Load scatter system shaders
        self.load_compute_shader(graphics_context, "scatter_distribution".to_string(), ComputeShaderSource::GLSL(include_str!("shaders/scatter_distribution.comp").to_string()))?;
        self.load_compute_shader(graphics_context, "scatter_culling".to_string(), ComputeShaderSource::GLSL(include_str!("shaders/scatter_culling.comp").to_string()))?;

        // Load utility shaders
        self.load_compute_shader(graphics_context, "prefix_sum".to_string(), ComputeShaderSource::GLSL(include_str!("shaders/prefix_sum.comp").to_string()))?;
        self.load_compute_shader(graphics_context, "parallel_reduction".to_string(), ComputeShaderSource::GLSL(include_str!("shaders/parallel_reduction.comp").to_string()))?;

        Ok(())
    }

    fn preallocate_buffers(&mut self, graphics_context: &GraphicsContext) -> RobinResult<()> {
        // Pre-allocate common buffer sizes
        let common_sizes = vec![
            1024,      // 1KB
            4096,      // 4KB  
            16384,     // 16KB
            65536,     // 64KB
            262144,    // 256KB
            1048576,   // 1MB
            4194304,   // 4MB
        ];

        for size in common_sizes {
            // Storage buffers
            let desc = BufferDescriptor {
                size,
                usage: BufferUsage::Storage,
                memory_type: MemoryType::DeviceLocal,
                initial_data: None,
            };
            let _ = self.create_buffer(graphics_context, desc)?;

            // Uniform buffers
            let desc = BufferDescriptor {
                size,
                usage: BufferUsage::Uniform,
                memory_type: MemoryType::DeviceLocal,
                initial_data: None,
            };
            let _ = self.create_buffer(graphics_context, desc)?;
        }

        Ok(())
    }

    fn warmup_shaders(&mut self, graphics_context: &GraphicsContext) -> RobinResult<()> {
        // Dispatch small workloads to warm up shaders
        let warmup_shaders = vec![
            "perlin_noise_2d",
            "perlin_noise_3d",
            "voxel_marching_cubes",
            "particle_update",
        ];

        for shader_name in warmup_shaders {
            let dispatch = ComputeDispatch {
                shader_name: shader_name.to_string(),
                work_groups: (1, 1, 1),
                buffers: vec![],
                textures: vec![],
                uniforms: HashMap::new(),
            };

            let _ = self.dispatch_compute(graphics_context, dispatch)?;
        }

        // Wait for warmup to complete
        self.sync(graphics_context)?;

        Ok(())
    }
}

/// GPU configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GPUConfig {
    /// Memory management configuration
    pub memory_config: GPUMemoryConfig,
    /// Buffer pool configuration
    pub buffer_pool_config: BufferPoolConfig,
    /// Enable GPU profiling
    pub enable_profiling: bool,
    /// Maximum compute dispatches per frame
    pub max_dispatches_per_frame: u32,
    /// Enable automatic memory defragmentation
    pub enable_memory_defrag: bool,
}

impl Default for GPUConfig {
    fn default() -> Self {
        Self {
            memory_config: GPUMemoryConfig::default(),
            buffer_pool_config: BufferPoolConfig::default(),
            enable_profiling: true,
            max_dispatches_per_frame: 1000,
            enable_memory_defrag: true,
        }
    }
}

/// Device capabilities query
#[derive(Debug, Clone)]
pub struct DeviceCapabilities {
    pub supports_compute_shaders: bool,
    pub max_compute_work_group_size: (u32, u32, u32),
    pub max_compute_work_group_invocations: u32,
    pub max_compute_shared_memory_size: u32,
    pub max_storage_buffer_bindings: u32,
    pub max_uniform_buffer_bindings: u32,
    pub supports_storage_buffer_arrays: bool,
    pub supports_variable_pointers: bool,
    pub memory_heap_count: u32,
    pub total_device_memory: u64,
    pub total_host_memory: u64,
}

impl DeviceCapabilities {
    pub fn query(graphics_context: &GraphicsContext) -> RobinResult<Self> {
        // In a real implementation, this would query the actual GPU capabilities
        // For now, we'll provide reasonable defaults for modern GPUs
        Ok(Self {
            supports_compute_shaders: true,
            max_compute_work_group_size: (1024, 1024, 64),
            max_compute_work_group_invocations: 1024,
            max_compute_shared_memory_size: 49152, // 48KB
            max_storage_buffer_bindings: 16,
            max_uniform_buffer_bindings: 16,
            supports_storage_buffer_arrays: true,
            supports_variable_pointers: true,
            memory_heap_count: 2,
            total_device_memory: 8 * 1024 * 1024 * 1024, // 8GB
            total_host_memory: 16 * 1024 * 1024 * 1024,  // 16GB
        })
    }

    pub fn meets_requirements(&self, requirements: &DeviceRequirements) -> bool {
        self.supports_compute_shaders >= requirements.requires_compute_shaders &&
        self.max_compute_work_group_invocations >= requirements.min_work_group_invocations &&
        self.max_compute_shared_memory_size >= requirements.min_shared_memory &&
        self.total_device_memory >= requirements.min_device_memory
    }
}

/// Device requirements for GPU acceleration
#[derive(Debug, Clone)]
pub struct DeviceRequirements {
    pub requires_compute_shaders: bool,
    pub min_work_group_invocations: u32,
    pub min_shared_memory: u32,
    pub min_device_memory: u64,
}

impl Default for DeviceRequirements {
    fn default() -> Self {
        Self {
            requires_compute_shaders: true,
            min_work_group_invocations: 256,
            min_shared_memory: 16384, // 16KB
            min_device_memory: 1024 * 1024 * 1024, // 1GB
        }
    }
}

/// GPU memory statistics
#[derive(Debug, Clone, Default)]
pub struct GPUMemoryStats {
    pub total_allocated: u64,
    pub total_used: u64,
    pub buffer_memory: u64,
    pub texture_memory: u64,
    pub fragmentation_ratio: f32,
    pub allocation_count: u32,
}

/// GPU performance metrics
#[derive(Debug, Clone, Default)]
pub struct GPUPerformanceMetrics {
    pub total_dispatches: u64,
    pub average_dispatch_time: f32,
    pub peak_dispatch_time: f32,
    pub total_compute_time: f32,
    pub memory_bandwidth_utilization: f32,
    pub compute_utilization: f32,
}

/// GPU profiler for performance monitoring
#[derive(Debug)]
pub struct GPUProfiler {
    metrics: GPUPerformanceMetrics,
    frame_start: std::time::Instant,
    dispatch_samples: Vec<DispatchSample>,
    current_frame: u64,
}

impl GPUProfiler {
    fn new() -> Self {
        Self {
            metrics: GPUPerformanceMetrics::default(),
            frame_start: std::time::Instant::now(),
            dispatch_samples: Vec::new(),
            current_frame: 0,
        }
    }

    fn start_frame(&mut self) {
        self.frame_start = std::time::Instant::now();
        self.dispatch_samples.clear();
    }

    fn end_frame(&mut self) {
        let frame_time = self.frame_start.elapsed().as_secs_f32();
        self.current_frame += 1;
        
        // Update metrics
        if !self.dispatch_samples.is_empty() {
            let total_dispatch_time: f32 = self.dispatch_samples.iter()
                .map(|sample| sample.execution_time)
                .sum();
            
            self.metrics.total_compute_time += total_dispatch_time;
            self.metrics.compute_utilization = total_dispatch_time / frame_time;
            
            let avg_dispatch_time = total_dispatch_time / self.dispatch_samples.len() as f32;
            self.metrics.average_dispatch_time = 
                (self.metrics.average_dispatch_time * 0.9) + (avg_dispatch_time * 0.1);
        }
    }

    fn start_dispatch(&mut self, shader_name: &str) {
        // Record dispatch start time
    }

    fn end_dispatch(&mut self, shader_name: &str, execution_time: f32) {
        self.dispatch_samples.push(DispatchSample {
            shader_name: shader_name.to_string(),
            execution_time,
            timestamp: std::time::Instant::now(),
        });

        self.metrics.total_dispatches += 1;
        
        if execution_time > self.metrics.peak_dispatch_time {
            self.metrics.peak_dispatch_time = execution_time;
        }
    }

    fn get_metrics(&self) -> &GPUPerformanceMetrics {
        &self.metrics
    }
}

#[derive(Debug)]
struct DispatchSample {
    shader_name: String,
    execution_time: f32,
    timestamp: std::time::Instant,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::graphics::MockGraphicsContext;

    #[test]
    fn test_device_capabilities_query() {
        let graphics_context = MockGraphicsContext::new();
        let caps = DeviceCapabilities::query(&graphics_context);
        
        assert!(caps.is_ok());
        let caps = caps.unwrap();
        assert!(caps.supports_compute_shaders);
        assert!(caps.max_compute_work_group_invocations > 0);
    }

    #[test]
    fn test_device_requirements_check() {
        let graphics_context = MockGraphicsContext::new();
        let caps = DeviceCapabilities::query(&graphics_context).unwrap();
        let requirements = DeviceRequirements::default();
        
        assert!(caps.meets_requirements(&requirements));
    }

    #[test]
    fn test_gpu_config_defaults() {
        let config = GPUConfig::default();
        assert!(config.enable_profiling);
        assert!(config.max_dispatches_per_frame > 0);
        assert!(config.enable_memory_defrag);
    }

    #[test]
    fn test_gpu_profiler() {
        let mut profiler = GPUProfiler::new();
        
        profiler.start_frame();
        profiler.start_dispatch("test_shader");
        profiler.end_dispatch("test_shader", 1.5);
        profiler.end_frame();
        
        let metrics = profiler.get_metrics();
        assert_eq!(metrics.total_dispatches, 1);
        assert!(metrics.peak_dispatch_time >= 1.5);
    }
}