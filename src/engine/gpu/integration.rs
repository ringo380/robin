/*!
 * Robin Engine GPU Acceleration Integration
 * 
 * High-level integration layer that connects GPU acceleration with
 * the engine's procedural generation systems for maximum performance.
 */

use crate::engine::{
    graphics::GraphicsContext,
    error::{RobinError, RobinResult},
    math::Vec3,
    generation::{
        GenerationEngine, GenerationConfig,
        NoiseSystem, VoxelSystem, PixelScatterSystem,
        DestructionSystem, RuntimeTools,
        SurfaceParams,
        noise::{TerrainParams, NoiseType},
        voxel_system::{VoxelWorld},
        pixel_scatter::{PointCloud},
    },
};
use super::{
    GPUAccelerationSystem, ComputeDispatch, ComputeBufferBinding, ComputeTextureBinding,
    BufferAccessType, TextureAccessType, UniformValue,
    GPUBufferHandle, GPUTextureHandle, BufferDescriptor, TextureDescriptor,
    BufferUsage, TextureUsage, MemoryType, TextureFormat,
};
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct ScatterParams {
    pub num_points: u32,
    pub area_bounds: [f32; 4], // [min_x, min_y, max_x, max_y]
    pub density: f32,
    pub randomness: f32,
    pub pattern: ScatterPatternType,
    pub exclusion_zones: Vec<[f32; 4]>,
}

#[derive(Debug, Clone)]
pub enum ScatterPatternType {
    Random,
    PoissonDisk,
    Grid,
    Hexagonal,
    Organic,
    Fractal,
}

/// GPU-accelerated generation engine integration
#[derive(Debug)]
pub struct GPUGenerationEngine {
    /// Base generation engine (CPU fallback)
    cpu_engine: GenerationEngine,
    /// GPU acceleration system
    gpu_system: GPUAccelerationSystem,
    /// GPU-specific buffers and textures
    gpu_resources: GPUResourceCache,
    /// Performance monitoring
    performance_monitor: GPUPerformanceMonitor,
    /// Hybrid execution strategies
    execution_strategy: HybridExecutionStrategy,
    /// Configuration
    config: GPUGenerationConfig,
}

impl GPUGenerationEngine {
    pub fn new(graphics_context: &GraphicsContext, config: GPUGenerationConfig) -> RobinResult<Self> {
        let cpu_engine = GenerationEngine::new(config.base_generation_config.clone());
        let gpu_system = GPUAccelerationSystem::new(graphics_context, config.gpu_config.clone())?;
        
        Ok(Self {
            cpu_engine,
            gpu_system,
            gpu_resources: GPUResourceCache::new(),
            performance_monitor: GPUPerformanceMonitor::new(),
            execution_strategy: HybridExecutionStrategy::new(config.hybrid_config.clone()),
            config,
        })
    }

    /// Initialize the GPU generation engine
    pub fn initialize(&mut self, graphics_context: &GraphicsContext) -> RobinResult<()> {
        self.gpu_system.initialize(graphics_context)?;
        self.gpu_resources.initialize(graphics_context, &mut self.gpu_system)?;
        Ok(())
    }

    /// Generate heightmap using GPU acceleration
    pub fn generate_heightmap_gpu(&mut self, graphics_context: &GraphicsContext, params: TerrainParams) -> RobinResult<Vec<f32>> {
        let start_time = Instant::now();

        // Extract values before moving params
        let heightmap_size = params.width * params.height;

        // Decide execution strategy
        let use_gpu = self.execution_strategy.should_use_gpu_for_heightmap(&params);
        
        if use_gpu {
            let params_clone = params.clone();
            let result = self.generate_heightmap_gpu_impl(graphics_context, params);
            
            match result {
                Ok(heightmap) => {
                    self.performance_monitor.record_heightmap_generation(true, start_time.elapsed(), heightmap_size);
                    Ok(heightmap)
                },
                Err(e) => {
                    // Fallback to CPU
                    println!("GPU heightmap generation failed, falling back to CPU: {:?}", e);
                    let cpu_result = self.cpu_engine.noise.generate_heightmap(params_clone)?;
                    let heightmap_data = self.extract_heightmap_data(&cpu_result);
                    self.performance_monitor.record_heightmap_generation(false, start_time.elapsed(), heightmap_size);
                    Ok(heightmap_data)
                }
            }
        } else {
            // Use CPU directly
            let cpu_result = self.cpu_engine.noise.generate_heightmap(params)?;
            let heightmap_data = self.extract_heightmap_data(&cpu_result);
            self.performance_monitor.record_heightmap_generation(false, start_time.elapsed(), heightmap_size);
            Ok(heightmap_data)
        }
    }

    /// Generate voxel mesh using GPU marching cubes
    pub fn generate_voxel_mesh_gpu(&mut self, graphics_context: &GraphicsContext, voxel_world: &VoxelWorld) -> RobinResult<VoxelMesh> {
        let start_time = Instant::now();

        if self.execution_strategy.should_use_gpu_for_voxel_mesh(voxel_world) {
            let result = self.generate_voxel_mesh_gpu_impl(graphics_context, voxel_world);
            
            match result {
                Ok(mesh) => {
                    self.performance_monitor.record_voxel_mesh_generation(true, start_time.elapsed(), mesh.vertex_count);
                    Ok(mesh)
                },
                Err(e) => {
                    // Fallback to CPU marching cubes
                    println!("GPU voxel mesh generation failed, falling back to CPU: {:?}", e);
                    let cpu_mesh = self.cpu_engine.voxel_system.generate_mesh(voxel_world)?;
                    self.performance_monitor.record_voxel_mesh_generation(false, start_time.elapsed(), cpu_mesh.vertex_count);
                    Ok(cpu_mesh)
                }
            }
        } else {
            // Use CPU directly
            let cpu_mesh = self.cpu_engine.voxel_system.generate_mesh(voxel_world)?;
            self.performance_monitor.record_voxel_mesh_generation(false, start_time.elapsed(), cpu_mesh.vertex_count);
            Ok(cpu_mesh)
        }
    }

    /// Generate particle system using GPU compute
    pub fn update_particles_gpu(&mut self, graphics_context: &GraphicsContext, particle_system: &mut GPUParticleSystem, delta_time: f32) -> RobinResult<()> {
        let start_time = Instant::now();

        if self.execution_strategy.should_use_gpu_for_particles(particle_system) {
            let result = self.update_particles_gpu_impl(graphics_context, particle_system, delta_time);
            
            match result {
                Ok(_) => {
                    self.performance_monitor.record_particle_update(true, start_time.elapsed(), particle_system.particle_count);
                    Ok(())
                },
                Err(e) => {
                    // Fallback to CPU particle system
                    println!("GPU particle update failed, falling back to CPU: {:?}", e);
                    self.update_particles_cpu(particle_system, delta_time);
                    self.performance_monitor.record_particle_update(false, start_time.elapsed(), particle_system.particle_count);
                    Ok(())
                }
            }
        } else {
            // Use CPU directly
            self.update_particles_cpu(particle_system, delta_time);
            self.performance_monitor.record_particle_update(false, start_time.elapsed(), particle_system.particle_count);
            Ok(())
        }
    }

    /// Generate scatter distribution using GPU
    pub fn generate_scatter_distribution_gpu(&mut self, graphics_context: &GraphicsContext, params: ScatterParams) -> RobinResult<PointCloud> {
        let start_time = Instant::now();

        if self.execution_strategy.should_use_gpu_for_scatter(&params) {
            let params_clone = params.clone();
            let result = self.generate_scatter_distribution_gpu_impl(graphics_context, params);
            
            match result {
                Ok(point_cloud) => {
                    self.performance_monitor.record_scatter_generation(true, start_time.elapsed(), point_cloud.points.len());
                    Ok(point_cloud)
                },
                Err(e) => {
                    // Fallback to CPU
                    println!("GPU scatter generation failed, falling back to CPU: {:?}", e);
                    let surface_params = Self::convert_scatter_to_surface_params(&params_clone);
                    let cpu_result = self.cpu_engine.scatter_system.generate_distribution(&surface_params)?;
                    self.performance_monitor.record_scatter_generation(false, start_time.elapsed(), cpu_result.points.len());
                    Ok(cpu_result)
                }
            }
        } else {
            // Use CPU directly
            let surface_params = Self::convert_scatter_to_surface_params(&params);
            let cpu_result = self.cpu_engine.scatter_system.generate_distribution(&surface_params)?;
            self.performance_monitor.record_scatter_generation(false, start_time.elapsed(), cpu_result.points.len());
            Ok(cpu_result)
        }
    }

    /// Get performance metrics for GPU vs CPU execution
    pub fn get_performance_metrics(&self) -> GPUGenerationMetrics {
        let gpu_metrics = self.gpu_system.get_performance_metrics();
        let monitor_metrics = self.performance_monitor.get_metrics();
        
        GPUGenerationMetrics {
            gpu_compute_metrics: gpu_metrics.clone(),
            generation_metrics: monitor_metrics.clone(),
            memory_metrics: self.gpu_system.get_memory_stats(),
            hybrid_efficiency: self.execution_strategy.get_efficiency_metrics().clone(),
        }
    }

    /// Begin frame for GPU operations
    pub fn begin_frame(&mut self) {
        self.gpu_system.begin_frame();
        self.performance_monitor.begin_frame();
        self.execution_strategy.begin_frame();
    }

    /// End frame for GPU operations
    pub fn end_frame(&mut self, graphics_context: &GraphicsContext) -> RobinResult<()> {
        self.gpu_system.end_frame(graphics_context)?;
        self.performance_monitor.end_frame();
        self.execution_strategy.end_frame();
        Ok(())
    }

    // Private implementation methods

    fn generate_heightmap_gpu_impl(&mut self, graphics_context: &GraphicsContext, params: TerrainParams) -> RobinResult<Vec<f32>> {
        // Create output texture
        let texture_desc = TextureDescriptor {
            width: params.width as u32,
            height: params.height as u32,
            depth: 1,
            format: TextureFormat::R32F,
            usage: TextureUsage::Storage,
            memory_type: MemoryType::DeviceLocal,
        };

        let output_texture = self.gpu_system.create_compute_texture(graphics_context, texture_desc)?;

        // Set up uniforms
        let mut uniforms = HashMap::new();
        uniforms.insert("frequency".to_string(), UniformValue::Float(params.scale));
        uniforms.insert("amplitude".to_string(), UniformValue::Float(params.amplitude));
        uniforms.insert("seed".to_string(), UniformValue::UInt(params.seed));

        // Choose shader based on noise type
        let shader_name = match params.noise_type {
            NoiseType::Perlin => "perlin_noise_2d",
            NoiseType::Simplex => "simplex_noise_2d",
            _ => "perlin_noise_2d", // Default fallback
        };

        // Create compute dispatch
        let dispatch = ComputeDispatch {
            shader_name: shader_name.to_string(),
            work_groups: (
                (params.width as u32 + 15) / 16,
                (params.height as u32 + 15) / 16,
                1
            ),
            buffers: vec![],
            textures: vec![
                ComputeTextureBinding {
                    binding_point: 0,
                    texture_handle: output_texture,
                    access_type: TextureAccessType::WriteOnly,
                    mip_level: 0,
                },
            ],
            uniforms,
        };

        // Execute compute shader
        let _result = self.gpu_system.dispatch_compute(graphics_context, dispatch)?;

        // Read back results
        let heightmap_data = self.read_texture_data_f32(graphics_context, output_texture, params.width * params.height)?;

        Ok(heightmap_data)
    }

    fn generate_voxel_mesh_gpu_impl(&mut self, graphics_context: &GraphicsContext, voxel_world: &VoxelWorld) -> RobinResult<VoxelMesh> {
        // Upload voxel data to GPU
        let voxel_texture = self.upload_voxel_data_to_gpu(graphics_context, voxel_world)?;

        // Create output buffers
        let vertex_buffer_desc = BufferDescriptor {
            size: 1024 * 1024 * 16, // 1M vertices * 16 bytes
            usage: BufferUsage::Storage,
            memory_type: MemoryType::DeviceLocal,
            initial_data: None,
        };
        let vertex_buffer_size = vertex_buffer_desc.size;
        let vertex_buffer = self.gpu_system.create_buffer(graphics_context, vertex_buffer_desc)?;

        let index_buffer_desc = BufferDescriptor {
            size: 1024 * 1024 * 4, // 1M indices * 4 bytes
            usage: BufferUsage::Storage,
            memory_type: MemoryType::DeviceLocal,
            initial_data: None,
        };
        let index_buffer_size = index_buffer_desc.size;
        let index_buffer = self.gpu_system.create_buffer(graphics_context, index_buffer_desc)?;

        let counter_buffer_desc = BufferDescriptor {
            size: 16, // vertex_count, triangle_count
            usage: BufferUsage::Storage,
            memory_type: MemoryType::HostVisible,
            initial_data: Some(vec![0u8; 16]),
        };
        let counter_buffer_size = counter_buffer_desc.size;
        let counter_buffer = self.gpu_system.create_buffer(graphics_context, counter_buffer_desc)?;

        // Set up compute dispatch for marching cubes
        let dispatch = ComputeDispatch {
            shader_name: "voxel_marching_cubes".to_string(),
            work_groups: (
                (voxel_world.world_size.0 as u32 + 3) / 4,
                (voxel_world.world_size.1 as u32 + 3) / 4,
                (voxel_world.world_size.2 as u32 + 3) / 4,
            ),
            buffers: vec![
                ComputeBufferBinding {
                    binding_point: 1,
                    buffer_handle: vertex_buffer,
                    access_type: BufferAccessType::WriteOnly,
                    offset: 0,
                    size: vertex_buffer_size as u64,
                },
                ComputeBufferBinding {
                    binding_point: 2,
                    buffer_handle: index_buffer,
                    access_type: BufferAccessType::WriteOnly,
                    offset: 0,
                    size: index_buffer_size as u64,
                },
                ComputeBufferBinding {
                    binding_point: 3,
                    buffer_handle: counter_buffer,
                    access_type: BufferAccessType::ReadWrite,
                    offset: 0,
                    size: counter_buffer_size as u64,
                },
            ],
            textures: vec![
                ComputeTextureBinding {
                    binding_point: 0,
                    texture_handle: voxel_texture,
                    access_type: TextureAccessType::ReadOnly,
                    mip_level: 0,
                },
            ],
            uniforms: HashMap::new(),
        };

        // Execute marching cubes
        let _result = self.gpu_system.dispatch_compute(graphics_context, dispatch)?;

        // Read back mesh data
        let mesh = self.read_voxel_mesh_from_gpu(graphics_context, vertex_buffer, index_buffer, counter_buffer)?;

        Ok(mesh)
    }

    fn update_particles_gpu_impl(&mut self, graphics_context: &GraphicsContext, particle_system: &mut GPUParticleSystem, delta_time: f32) -> RobinResult<()> {
        // Set up compute dispatch for particle update
        let mut uniforms = HashMap::new();
        uniforms.insert("delta_time".to_string(), UniformValue::Float(delta_time));
        uniforms.insert("gravity".to_string(), UniformValue::Vec3(0.0, -9.81, 0.0));

        let dispatch = ComputeDispatch {
            shader_name: "particle_update".to_string(),
            work_groups: ((particle_system.particle_count + 63) / 64, 1, 1),
            buffers: vec![
                ComputeBufferBinding {
                    binding_point: 0,
                    buffer_handle: particle_system.particle_buffer,
                    access_type: BufferAccessType::ReadWrite,
                    offset: 0,
                    size: particle_system.buffer_size,
                },
            ],
            textures: vec![],
            uniforms,
        };

        // Execute particle update
        let _result = self.gpu_system.dispatch_compute(graphics_context, dispatch)?;

        Ok(())
    }

    fn generate_scatter_distribution_gpu_impl(&mut self, graphics_context: &GraphicsContext, params: ScatterParams) -> RobinResult<PointCloud> {
        // Implementation would generate scatter points using GPU compute
        // For now, return a placeholder
        Ok(PointCloud {
            points: vec![],
            density: 0.0,
            bounds: (Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0)),
            metadata: Default::default(),
        })
    }

    fn update_particles_cpu(&mut self, particle_system: &mut GPUParticleSystem, delta_time: f32) {
        // CPU fallback for particle updates
        // Implementation would update particles on CPU
    }

    fn extract_heightmap_data(&self, heightmap: &super::super::generation::noise::Heightmap) -> Vec<f32> {
        let mut data = Vec::new();
        for y in 0..heightmap.height {
            for x in 0..heightmap.width {
                data.push(heightmap.heights[x][y]);
            }
        }
        data
    }

    fn upload_voxel_data_to_gpu(&mut self, graphics_context: &GraphicsContext, voxel_world: &VoxelWorld) -> RobinResult<GPUTextureHandle> {
        let texture_desc = TextureDescriptor {
            width: voxel_world.world_size.0 as u32,
            height: voxel_world.world_size.1 as u32,
            depth: voxel_world.world_size.2 as u32,
            format: TextureFormat::R32F,
            usage: TextureUsage::Storage,
            memory_type: MemoryType::DeviceLocal,
        };

        let texture_handle = self.gpu_system.create_compute_texture(graphics_context, texture_desc)?;
        
        // Upload voxel data
        // In a real implementation, this would convert voxel data to texture format
        // and upload to GPU

        Ok(texture_handle)
    }

    fn read_texture_data_f32(&mut self, graphics_context: &GraphicsContext, texture_handle: GPUTextureHandle, element_count: usize) -> RobinResult<Vec<f32>> {
        // Create staging buffer
        let staging_buffer_desc = BufferDescriptor {
            size: element_count * 4, // f32 = 4 bytes
            usage: BufferUsage::Transfer,
            memory_type: MemoryType::HostVisible,
            initial_data: None,
        };

        let staging_buffer = self.gpu_system.create_buffer(graphics_context, staging_buffer_desc)?;

        // Copy texture to staging buffer (would be implemented in graphics backend)
        
        // Map and read buffer
        let mut data = vec![0.0f32; element_count];
        self.gpu_system.download_buffer_data(graphics_context, staging_buffer, &mut data)?;

        Ok(data)
    }

    fn read_voxel_mesh_from_gpu(&mut self, graphics_context: &GraphicsContext, vertex_buffer: GPUBufferHandle, index_buffer: GPUBufferHandle, counter_buffer: GPUBufferHandle) -> RobinResult<VoxelMesh> {
        // Read counters first
        let mut counters = vec![0u32; 4];
        self.gpu_system.download_buffer_data(graphics_context, counter_buffer, &mut counters)?;

        let vertex_count = counters[0] as usize;
        let triangle_count = counters[1] as usize;

        // Read vertex data
        let mut vertices = vec![VoxelVertex::default(); vertex_count];
        // Would download actual vertex data here

        // Read index data
        let mut indices = vec![0u32; triangle_count * 3];
        // Would download actual index data here

        Ok(VoxelMesh {
            vertices,
            indices,
            vertex_count,
            triangle_count,
        })
    }

    /// Convert ScatterParams to SurfaceParams for CPU fallback
    fn convert_scatter_to_surface_params(scatter: &ScatterParams) -> SurfaceParams {
        use crate::engine::generation::{SurfaceGeneration, SurfaceType, MaterialProperties};
        
        SurfaceParams {
            technique: SurfaceGeneration::ScatterBased,
            surface_type: SurfaceType::Natural, // Default to natural
            resolution: (scatter.density * 100.0) as u32, // Convert density to resolution
            material_properties: MaterialProperties {
                density: scatter.density,
                hardness: 1.0 - scatter.randomness, // Invert randomness to hardness
                transparency: 0.0,
                reflectivity: 0.5,
                color: crate::engine::graphics::Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
            },
        }
    }
}

/// GPU resource cache for commonly used buffers and textures
#[derive(Debug)]
struct GPUResourceCache {
    noise_permutation_buffer: Option<GPUBufferHandle>,
    marching_cubes_tables: Option<(GPUBufferHandle, GPUBufferHandle)>,
    common_textures: HashMap<String, GPUTextureHandle>,
}

impl GPUResourceCache {
    fn new() -> Self {
        Self {
            noise_permutation_buffer: None,
            marching_cubes_tables: None,
            common_textures: HashMap::new(),
        }
    }

    fn initialize(&mut self, graphics_context: &GraphicsContext, gpu_system: &mut GPUAccelerationSystem) -> RobinResult<()> {
        // Create permutation table buffer for noise generation
        let perm_table = self.generate_permutation_table();
        let perm_buffer_desc = BufferDescriptor {
            size: perm_table.len() * 4,
            usage: BufferUsage::Storage,
            memory_type: MemoryType::DeviceLocal,
            initial_data: Some(unsafe { 
                std::slice::from_raw_parts(perm_table.as_ptr() as *const u8, perm_table.len() * 4).to_vec()
            }),
        };
        
        let perm_buffer = gpu_system.create_buffer(graphics_context, perm_buffer_desc)?;
        self.noise_permutation_buffer = Some(perm_buffer);

        // Create marching cubes lookup tables
        let (edge_table, tri_table) = self.generate_marching_cubes_tables();
        
        let edge_buffer_desc = BufferDescriptor {
            size: edge_table.len() * 4,
            usage: BufferUsage::Storage,
            memory_type: MemoryType::DeviceLocal,
            initial_data: Some(unsafe { 
                std::slice::from_raw_parts(edge_table.as_ptr() as *const u8, edge_table.len() * 4).to_vec()
            }),
        };
        
        let tri_buffer_desc = BufferDescriptor {
            size: tri_table.len() * 4,
            usage: BufferUsage::Storage,
            memory_type: MemoryType::DeviceLocal,
            initial_data: Some(unsafe { 
                std::slice::from_raw_parts(tri_table.as_ptr() as *const u8, tri_table.len() * 4).to_vec()
            }),
        };

        let edge_buffer = gpu_system.create_buffer(graphics_context, edge_buffer_desc)?;
        let tri_buffer = gpu_system.create_buffer(graphics_context, tri_buffer_desc)?;
        
        self.marching_cubes_tables = Some((edge_buffer, tri_buffer));

        Ok(())
    }

    fn generate_permutation_table(&self) -> Vec<i32> {
        // Standard Perlin noise permutation table
        let mut perm = vec![
            151,160,137,91,90,15,131,13,201,95,96,53,194,233,7,225,140,36,103,30,69,142,8,99,37,240,21,10,
            23,190,6,148,247,120,234,75,0,26,197,62,94,252,219,203,117,35,11,32,57,177,33,88,237,149,56,
            87,174,20,125,136,171,168,68,175,74,165,71,134,139,48,27,166,77,146,158,231,83,111,229,122,
            60,211,133,230,220,105,92,41,55,46,245,40,244,102,143,54,65,25,63,161,1,216,80,73,209,76,
            132,187,208,89,18,169,200,196,135,130,116,188,159,86,164,100,109,198,173,186,3,64,52,217,
            226,250,124,123,5,202,38,147,118,126,255,82,85,212,207,206,59,227,47,16,58,17,182,189,28,
            42,223,183,170,213,119,248,152,2,44,154,163,70,221,153,101,155,167,43,172,9,129,22,39,253,
            19,98,108,110,79,113,224,232,178,185,112,104,218,246,97,228,251,34,242,193,238,210,144,12,
            191,179,162,241,81,51,145,235,249,14,239,107,49,192,214,31,181,199,106,157,184,84,204,176,
            115,121,50,45,127,4,150,254,138,236,205,93,222,114,67,29,24,72,243,141,128,195,78,66,215,
            61,156,180,
        ];

        // Duplicate the permutation table
        perm.extend_from_slice(&perm.clone());
        perm
    }

    fn generate_marching_cubes_tables(&self) -> (Vec<u32>, Vec<i32>) {
        // Marching cubes edge table (which edges are intersected)
        let edge_table = vec![
            0x0, 0x109, 0x203, 0x30a, 0x406, 0x50f, 0x605, 0x70c,
            0x80c, 0x905, 0xa0f, 0xb06, 0xc0a, 0xd03, 0xe09, 0xf00,
            // ... (complete table would have 256 entries)
        ];

        // Marching cubes triangle table (which triangles to generate)  
        let tri_table = vec![
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            0, 8, 3, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            // ... (complete table would have 256 * 16 entries)
        ];

        (edge_table, tri_table)
    }
}

/// Performance monitoring for GPU vs CPU execution
#[derive(Debug)]
struct GPUPerformanceMonitor {
    metrics: GenerationPerformanceMetrics,
    current_frame: u64,
}

impl GPUPerformanceMonitor {
    fn new() -> Self {
        Self {
            metrics: GenerationPerformanceMetrics::default(),
            current_frame: 0,
        }
    }

    fn begin_frame(&mut self) {
        self.current_frame += 1;
    }

    fn end_frame(&mut self) {
        // Update frame-based metrics
    }

    fn record_heightmap_generation(&mut self, used_gpu: bool, duration: std::time::Duration, element_count: usize) {
        if used_gpu {
            self.metrics.gpu_heightmap_count += 1;
            self.metrics.gpu_heightmap_time += duration.as_secs_f32();
        } else {
            self.metrics.cpu_heightmap_count += 1;
            self.metrics.cpu_heightmap_time += duration.as_secs_f32();
        }
    }

    fn record_voxel_mesh_generation(&mut self, used_gpu: bool, duration: std::time::Duration, vertex_count: usize) {
        if used_gpu {
            self.metrics.gpu_voxel_count += 1;
            self.metrics.gpu_voxel_time += duration.as_secs_f32();
        } else {
            self.metrics.cpu_voxel_count += 1;
            self.metrics.cpu_voxel_time += duration.as_secs_f32();
        }
    }

    fn record_particle_update(&mut self, used_gpu: bool, duration: std::time::Duration, particle_count: u32) {
        if used_gpu {
            self.metrics.gpu_particle_count += 1;
            self.metrics.gpu_particle_time += duration.as_secs_f32();
        } else {
            self.metrics.cpu_particle_count += 1;
            self.metrics.cpu_particle_time += duration.as_secs_f32();
        }
    }

    fn record_scatter_generation(&mut self, used_gpu: bool, duration: std::time::Duration, point_count: usize) {
        if used_gpu {
            self.metrics.gpu_scatter_count += 1;
            self.metrics.gpu_scatter_time += duration.as_secs_f32();
        } else {
            self.metrics.cpu_scatter_count += 1;
            self.metrics.cpu_scatter_time += duration.as_secs_f32();
        }
    }

    fn get_metrics(&self) -> &GenerationPerformanceMetrics {
        &self.metrics
    }
}

/// Hybrid execution strategy that decides GPU vs CPU execution
#[derive(Debug)]
struct HybridExecutionStrategy {
    config: HybridExecutionConfig,
    efficiency_tracker: EfficiencyTracker,
    adaptive_thresholds: AdaptiveThresholds,
}

impl HybridExecutionStrategy {
    fn new(config: HybridExecutionConfig) -> Self {
        Self {
            config,
            efficiency_tracker: EfficiencyTracker::new(),
            adaptive_thresholds: AdaptiveThresholds::new(),
        }
    }

    fn should_use_gpu_for_heightmap(&self, params: &TerrainParams) -> bool {
        let complexity = params.width * params.height;
        complexity >= self.adaptive_thresholds.heightmap_gpu_threshold
    }

    fn should_use_gpu_for_voxel_mesh(&self, voxel_world: &VoxelWorld) -> bool {
        let complexity = voxel_world.world_size.0 * voxel_world.world_size.1 * voxel_world.world_size.2;
        complexity >= self.adaptive_thresholds.voxel_gpu_threshold
    }

    fn should_use_gpu_for_particles(&self, particle_system: &GPUParticleSystem) -> bool {
        particle_system.particle_count >= self.adaptive_thresholds.particle_gpu_threshold
    }

    fn should_use_gpu_for_scatter(&self, params: &ScatterParams) -> bool {
        params.num_points as usize >= self.adaptive_thresholds.scatter_gpu_threshold
    }

    fn begin_frame(&mut self) {
        self.efficiency_tracker.begin_frame();
    }

    fn end_frame(&mut self) {
        self.efficiency_tracker.end_frame();
        self.adaptive_thresholds.update(&self.efficiency_tracker);
    }

    fn get_efficiency_metrics(&self) -> &EfficiencyMetrics {
        self.efficiency_tracker.get_metrics()
    }
}

// Configuration and data structures

/// GPU generation engine configuration
#[derive(Debug, Clone)]
pub struct GPUGenerationConfig {
    pub base_generation_config: GenerationConfig,
    pub gpu_config: super::GPUConfig,
    pub hybrid_config: HybridExecutionConfig,
    pub enable_fallback: bool,
}

impl Default for GPUGenerationConfig {
    fn default() -> Self {
        Self {
            base_generation_config: GenerationConfig::default(),
            gpu_config: super::GPUConfig::default(),
            hybrid_config: HybridExecutionConfig::default(),
            enable_fallback: true,
        }
    }
}

/// Hybrid execution configuration
#[derive(Debug, Clone)]
pub struct HybridExecutionConfig {
    pub initial_heightmap_threshold: usize,
    pub initial_voxel_threshold: usize,
    pub initial_particle_threshold: u32,
    pub initial_scatter_threshold: usize,
    pub adaptation_enabled: bool,
}

impl Default for HybridExecutionConfig {
    fn default() -> Self {
        Self {
            initial_heightmap_threshold: 64 * 64,    // 64x64 heightmaps
            initial_voxel_threshold: 32 * 32 * 32,   // 32^3 voxel worlds
            initial_particle_threshold: 1000,        // 1000 particles
            initial_scatter_threshold: 10000,        // 10K scatter points
            adaptation_enabled: true,
        }
    }
}

/// Combined GPU generation metrics
#[derive(Debug, Clone)]
pub struct GPUGenerationMetrics {
    pub gpu_compute_metrics: super::GPUPerformanceMetrics,
    pub generation_metrics: GenerationPerformanceMetrics,
    pub memory_metrics: super::GPUMemoryStats,
    pub hybrid_efficiency: EfficiencyMetrics,
}

/// Performance metrics for generation operations
#[derive(Debug, Clone, Default)]
pub struct GenerationPerformanceMetrics {
    pub gpu_heightmap_count: u64,
    pub cpu_heightmap_count: u64,
    pub gpu_heightmap_time: f32,
    pub cpu_heightmap_time: f32,
    
    pub gpu_voxel_count: u64,
    pub cpu_voxel_count: u64,
    pub gpu_voxel_time: f32,
    pub cpu_voxel_time: f32,
    
    pub gpu_particle_count: u64,
    pub cpu_particle_count: u64,
    pub gpu_particle_time: f32,
    pub cpu_particle_time: f32,
    
    pub gpu_scatter_count: u64,
    pub cpu_scatter_count: u64,
    pub gpu_scatter_time: f32,
    pub cpu_scatter_time: f32,
}

/// GPU particle system
#[derive(Debug)]
pub struct GPUParticleSystem {
    pub particle_buffer: GPUBufferHandle,
    pub particle_count: u32,
    pub max_particles: u32,
    pub buffer_size: u64,
}

/// Voxel mesh generated from GPU
#[derive(Debug)]
pub struct VoxelMesh {
    pub vertices: Vec<VoxelVertex>,
    pub indices: Vec<u32>,
    pub vertex_count: usize,
    pub triangle_count: usize,
}

#[derive(Debug, Default, Clone)]
pub struct VoxelVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub material_id: f32,
}

impl VoxelMesh {
    /// Convert VoxelMesh to standard Mesh
    pub fn to_mesh(&self, name: String) -> crate::engine::graphics::Mesh {
        use crate::engine::graphics::{Mesh, Vertex};
        use crate::engine::math::{Vec2, Vec3};
        
        let vertices: Vec<Vertex> = self.vertices.iter().map(|v| {
            Vertex {
                position: Vec3::new(v.position[0], v.position[1], v.position[2]),
                normal: Vec3::new(v.normal[0], v.normal[1], v.normal[2]),
                uv: Vec2::new(0.0, 0.0), // Default UV coordinates
                color: [1.0, 1.0, 1.0, 1.0], // Default white color
            }
        }).collect();
        
        Mesh {
            vertices,
            indices: self.indices.clone(),
            name,
        }
    }
}

/// Efficiency tracking for adaptive thresholds
#[derive(Debug)]
struct EfficiencyTracker {
    metrics: EfficiencyMetrics,
    recent_samples: std::collections::VecDeque<EfficiencySample>,
}

impl EfficiencyTracker {
    fn new() -> Self {
        Self {
            metrics: EfficiencyMetrics::default(),
            recent_samples: std::collections::VecDeque::new(),
        }
    }

    fn begin_frame(&mut self) {
        // Record frame start
    }

    fn end_frame(&mut self) {
        // Update efficiency metrics
    }

    fn get_metrics(&self) -> &EfficiencyMetrics {
        &self.metrics
    }
}

#[derive(Debug, Clone, Default)]
pub struct EfficiencyMetrics {
    pub gpu_efficiency_ratio: f32,
    pub cpu_efficiency_ratio: f32,
    pub optimal_gpu_usage: f32,
}

#[derive(Debug)]
struct EfficiencySample {
    operation_type: String,
    used_gpu: bool,
    performance_ratio: f32,
    complexity: usize,
}

/// Adaptive threshold management
#[derive(Debug)]
struct AdaptiveThresholds {
    pub heightmap_gpu_threshold: usize,
    pub voxel_gpu_threshold: usize,
    pub particle_gpu_threshold: u32,
    pub scatter_gpu_threshold: usize,
}

impl AdaptiveThresholds {
    fn new() -> Self {
        Self {
            heightmap_gpu_threshold: 64 * 64,
            voxel_gpu_threshold: 32 * 32 * 32,
            particle_gpu_threshold: 1000,
            scatter_gpu_threshold: 10000,
        }
    }

    fn update(&mut self, efficiency_tracker: &EfficiencyTracker) {
        // Update thresholds based on efficiency data
        let metrics = efficiency_tracker.get_metrics();
        
        if metrics.gpu_efficiency_ratio > 1.2 {
            // GPU is significantly faster, lower thresholds
            self.heightmap_gpu_threshold = (self.heightmap_gpu_threshold as f32 * 0.8) as usize;
            self.voxel_gpu_threshold = (self.voxel_gpu_threshold as f32 * 0.8) as usize;
            self.particle_gpu_threshold = (self.particle_gpu_threshold as f32 * 0.8) as u32;
            self.scatter_gpu_threshold = (self.scatter_gpu_threshold as f32 * 0.8) as usize;
        } else if metrics.gpu_efficiency_ratio < 0.8 {
            // GPU is slower, raise thresholds
            self.heightmap_gpu_threshold = (self.heightmap_gpu_threshold as f32 * 1.2) as usize;
            self.voxel_gpu_threshold = (self.voxel_gpu_threshold as f32 * 1.2) as usize;
            self.particle_gpu_threshold = (self.particle_gpu_threshold as f32 * 1.2) as u32;
            self.scatter_gpu_threshold = (self.scatter_gpu_threshold as f32 * 1.2) as usize;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::graphics::MockGraphicsContext;

    #[test]
    fn test_gpu_generation_engine_creation() {
        let graphics_context = MockGraphicsContext::new();
        let config = GPUGenerationConfig::default();
        let engine = GPUGenerationEngine::new(&graphics_context, config);
        
        assert!(engine.is_ok());
    }

    #[test]
    fn test_hybrid_execution_strategy() {
        let config = HybridExecutionConfig::default();
        let strategy = HybridExecutionStrategy::new(config);
        
        // Test heightmap decision
        let small_params = TerrainParams {
            width: 32,
            height: 32,
            scale: 0.1,
            amplitude: 10.0,
            base_height: 0.0,
            noise_type: NoiseType::Perlin,
            seed: 12345,
        };
        
        let large_params = TerrainParams {
            width: 256,
            height: 256,
            scale: 0.1,
            amplitude: 10.0,
            base_height: 0.0,
            noise_type: NoiseType::Perlin,
            seed: 12345,
        };
        
        assert!(!strategy.should_use_gpu_for_heightmap(&small_params));
        assert!(strategy.should_use_gpu_for_heightmap(&large_params));
    }

    #[test]
    fn test_performance_monitor() {
        let mut monitor = GPUPerformanceMonitor::new();
        
        monitor.record_heightmap_generation(true, std::time::Duration::from_millis(10), 1000);
        monitor.record_heightmap_generation(false, std::time::Duration::from_millis(50), 1000);
        
        let metrics = monitor.get_metrics();
        assert_eq!(metrics.gpu_heightmap_count, 1);
        assert_eq!(metrics.cpu_heightmap_count, 1);
        assert!(metrics.gpu_heightmap_time < metrics.cpu_heightmap_time);
    }
}