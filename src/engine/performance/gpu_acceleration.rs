use crate::engine::error::RobinResult;
use std::collections::{HashMap, VecDeque};
use std::time::Instant;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GPUConfig {
    pub enabled: bool,
    pub max_compute_dispatches_per_frame: u32,
    pub max_buffer_size_mb: u32,
    pub prefer_dedicated_gpu: bool,
    pub enable_debug_layers: bool,
    pub compute_queue_priority: u32,
    pub memory_pool_size_mb: u32,
    pub async_compute: bool,
}

impl Default for GPUConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_compute_dispatches_per_frame: 8,
            max_buffer_size_mb: 256,
            prefer_dedicated_gpu: true,
            enable_debug_layers: false,
            compute_queue_priority: 1,
            memory_pool_size_mb: 512,
            async_compute: true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum GPUTaskType {
    PhysicsSimulation,
    ParticleSystem,
    VolumeRendering,
    LightingCalculation,
    TerrainGeneration,
    ImageProcessing,
    CustomCompute(String),
}

#[derive(Debug, Clone)]
pub struct GPUTask {
    pub id: String,
    pub task_type: GPUTaskType,
    pub input_data: Vec<f32>,
    pub parameters: HashMap<String, f32>,
    pub work_groups: [u32; 3],
    pub priority: u32,
    pub created_at: Instant,
    pub timeout_ms: Option<u64>,
}

impl GPUTask {
    pub fn new(id: String, task_type: GPUTaskType, input_data: Vec<f32>) -> Self {
        Self {
            id,
            task_type,
            input_data,
            parameters: HashMap::new(),
            work_groups: [1, 1, 1],
            priority: 0,
            created_at: Instant::now(),
            timeout_ms: Some(5000),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TaskResult {
    pub task_id: String,
    pub output_data: Vec<f32>,
    pub execution_time_ms: f32,
    pub success: bool,
    pub error_message: Option<String>,
    pub completed_at: Instant,
}

#[derive(Debug, Default, Clone)]
pub struct GPUMetrics {
    pub active_buffers: u32,
    pub completed_tasks: u32,
    pub pending_tasks: u32,
    pub failed_tasks: u32,
    pub gpu_memory_usage_mb: f32,
    pub average_task_time_ms: f32,
    pub dispatches_per_second: f32,
    pub buffer_allocation_count: u32,
    pub buffer_deallocation_count: u32,
    pub compute_utilization_percent: f32,
}

#[derive(Debug)]
pub struct GPUAccelerator {
    config: GPUConfig,
    enabled: bool,
    compute_shaders: HashMap<String, ComputeShader>,
    gpu_buffers: HashMap<String, GPUBuffer>,
    memory_manager: GPUMemoryManager,
    quality_multiplier: f32,
    task_queue: VecDeque<GPUTask>,
    executing_tasks: HashMap<String, Instant>,
    completed_results: HashMap<String, TaskResult>,
    metrics: GPUMetrics,
    last_metrics_update: Instant,
}

#[derive(Debug)]
pub struct ComputeShader {
    pub name: String,
    pub source: String,
    pub compiled: bool,
}

#[derive(Debug, Clone)]
pub struct GPUBuffer {
    pub id: String,
    pub size_bytes: usize,
    pub usage: GPUBufferUsage,
    pub data: Vec<u8>,
    pub binding: u32,
    pub created_at: Instant,
    pub last_accessed: Instant,
}

#[derive(Debug, Clone)]
pub enum GPUBufferUsage {
    Vertex,
    Index,
    Uniform,
    Storage,
    Compute,
}

#[derive(Debug)]
pub struct GPUMemoryManager {
    total_memory_mb: f32,
    used_memory_mb: f32,
    allocated_buffers: HashMap<String, usize>,
}

impl GPUAccelerator {
    pub fn new(config: GPUConfig) -> RobinResult<Self> {
        Ok(Self {
            enabled: config.enabled,
            config,
            compute_shaders: HashMap::new(),
            gpu_buffers: HashMap::new(),
            memory_manager: GPUMemoryManager {
                total_memory_mb: 4096.0, // 4GB default
                used_memory_mb: 0.0,
                allocated_buffers: HashMap::new(),
            },
            quality_multiplier: 1.0,
            task_queue: VecDeque::new(),
            executing_tasks: HashMap::new(),
            completed_results: HashMap::new(),
            metrics: GPUMetrics::default(),
            last_metrics_update: Instant::now(),
        })
    }

    pub fn initialize(&mut self) -> RobinResult<()> {
        if !self.enabled {
            return Ok(());
        }

        self.initialize_compute_shaders()?;
        
        println!("GPU Accelerator initialized:");
        println!("  Max Compute Dispatches: {}/frame", self.config.max_compute_dispatches_per_frame);
        println!("  Max Buffer Size: {}MB", self.config.max_buffer_size_mb);
        println!("  Memory Pool: {}MB", self.config.memory_pool_size_mb);
        println!("  Async Compute: {}", self.config.async_compute);

        Ok(())
    }

    pub fn update(&mut self, delta_time: f32) -> RobinResult<()> {
        if !self.enabled {
            return Ok(());
        }
        
        self.process_task_queue()?;
        self.update_executing_tasks()?;
        self.cleanup_old_results()?;
        self.update_gpu_memory_manager()?;
        self.optimize_buffer_usage(delta_time)?;
        
        if self.last_metrics_update.elapsed().as_secs_f32() >= 1.0 {
            self.update_metrics();
            self.last_metrics_update = Instant::now();
        }
        
        Ok(())
    }

    fn process_task_queue(&mut self) -> RobinResult<()> {
        let max_dispatches = self.config.max_compute_dispatches_per_frame as usize;
        let current_dispatches = self.executing_tasks.len();

        if current_dispatches >= max_dispatches {
            return Ok(());
        }

        let tasks_to_execute = (max_dispatches - current_dispatches).min(self.task_queue.len());
        
        for _ in 0..tasks_to_execute {
            if let Some(task) = self.task_queue.pop_front() {
                self.execute_task(task)?;
            }
        }

        Ok(())
    }

    fn execute_task(&mut self, task: GPUTask) -> RobinResult<()> {
        let task_id = task.id.clone();
        self.executing_tasks.insert(task_id, Instant::now());
        
        // Simulate task execution based on type
        match task.task_type {
            GPUTaskType::PhysicsSimulation => self.accelerate_physics_batch(task.input_data.len())?,
            GPUTaskType::ParticleSystem => self.accelerate_rendering_batch(task.input_data.len())?,
            _ => {
                // Generic compute task
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
        }
        
        Ok(())
    }

    fn update_executing_tasks(&mut self) -> RobinResult<()> {
        let mut completed_tasks = Vec::new();

        for (task_id, start_time) in &self.executing_tasks {
            if start_time.elapsed().as_millis() > 10 { // Simulate completion after 10ms
                completed_tasks.push(task_id.clone());
            }
        }

        for task_id in completed_tasks {
            self.complete_task(task_id)?;
        }

        Ok(())
    }

    fn complete_task(&mut self, task_id: String) -> RobinResult<()> {
        if let Some(start_time) = self.executing_tasks.remove(&task_id) {
            let execution_time = start_time.elapsed().as_secs_f32() * 1000.0;
            
            let result = TaskResult {
                task_id: task_id.clone(),
                output_data: vec![1.0; 100], // Dummy output data
                execution_time_ms: execution_time,
                success: true,
                error_message: None,
                completed_at: Instant::now(),
            };

            self.completed_results.insert(task_id, result);
            self.metrics.completed_tasks += 1;
        }

        Ok(())
    }

    fn cleanup_old_results(&mut self) -> RobinResult<()> {
        let cutoff_time = Instant::now() - std::time::Duration::from_secs(60);
        
        self.completed_results.retain(|_, result| {
            result.completed_at > cutoff_time
        });

        Ok(())
    }

    fn update_metrics(&mut self) {
        self.metrics.active_buffers = self.gpu_buffers.len() as u32;
        self.metrics.pending_tasks = self.task_queue.len() as u32;
        self.metrics.gpu_memory_usage_mb = self.get_memory_usage_mb();
        
        // Calculate compute utilization
        self.metrics.compute_utilization_percent = 
            (self.executing_tasks.len() as f32 / self.config.max_compute_dispatches_per_frame as f32) * 100.0;
    }

    pub fn accelerate_physics_batch(&mut self, particle_count: usize) -> RobinResult<()> {
        if !self.enabled || particle_count == 0 {
            return Ok(());
        }

        let buffer_name = format!("physics_batch_{}", particle_count);
        self.ensure_buffer_capacity(&buffer_name, particle_count * 64)?; // 64 bytes per particle
        
        // Simulate GPU physics computation
        std::thread::sleep(std::time::Duration::from_micros(
            (particle_count as u64 / 100).max(10) // Simulate GPU processing time
        ));
        
        Ok(())
    }

    pub fn accelerate_rendering_batch(&mut self, vertex_count: usize) -> RobinResult<()> {
        if !self.enabled || vertex_count == 0 {
            return Ok(());
        }

        let buffer_name = format!("vertex_batch_{}", vertex_count);
        self.ensure_buffer_capacity(&buffer_name, vertex_count * 32)?; // 32 bytes per vertex
        
        // Simulate GPU rendering acceleration
        let processing_time = (vertex_count as f32 * self.quality_multiplier) / 50000.0;
        std::thread::sleep(std::time::Duration::from_micros(
            (processing_time * 1000.0) as u64
        ));
        
        Ok(())
    }

    pub fn compile_compute_shader(&mut self, name: &str, source: &str) -> RobinResult<()> {
        if !self.enabled {
            return Ok(());
        }

        let shader = ComputeShader {
            name: name.to_string(),
            source: source.to_string(),
            compiled: true, // Simulate successful compilation
        };

        self.compute_shaders.insert(name.to_string(), shader);
        Ok(())
    }

    pub fn dispatch_compute(&mut self, shader_name: &str, workgroup_size: [u32; 3]) -> RobinResult<()> {
        if !self.enabled {
            return Ok(());
        }

        if !self.compute_shaders.contains_key(shader_name) {
            return Err(crate::engine::error::RobinError::new(
                &format!("Compute shader '{}' not found", shader_name)
            ));
        }

        // Simulate GPU compute dispatch
        let total_workgroups = workgroup_size[0] * workgroup_size[1] * workgroup_size[2];
        let processing_time = (total_workgroups as f32 / 10000.0).max(0.001);
        std::thread::sleep(std::time::Duration::from_micros(
            (processing_time * 1000.0) as u64
        ));

        Ok(())
    }

    pub fn create_buffer(&mut self, name: &str, size_bytes: usize, usage: GPUBufferUsage) -> RobinResult<()> {
        if !self.enabled {
            return Ok(());
        }

        let buffer = GPUBuffer {
            id: name.to_string(),
            size_bytes,
            usage,
            data: vec![0; size_bytes],
            binding: 0,
            created_at: Instant::now(),
            last_accessed: Instant::now(),
        };

        self.gpu_buffers.insert(name.to_string(), buffer);
        self.memory_manager.allocated_buffers.insert(name.to_string(), size_bytes);
        self.memory_manager.used_memory_mb += size_bytes as f32 / 1_048_576.0;
        self.metrics.buffer_allocation_count += 1;

        Ok(())
    }

    pub fn release_buffer(&mut self, name: &str) -> RobinResult<()> {
        if let Some(buffer) = self.gpu_buffers.remove(name) {
            if let Some(size) = self.memory_manager.allocated_buffers.remove(name) {
                self.memory_manager.used_memory_mb -= size as f32 / 1_048_576.0;
                self.metrics.buffer_deallocation_count += 1;
            }
        }
        Ok(())
    }

    fn initialize_compute_shaders(&mut self) -> RobinResult<()> {
        // Initialize common compute shaders for physics and rendering
        let physics_shader_source = r#"
            #version 450
            layout(local_size_x = 64) in;
            layout(set = 0, binding = 0) buffer ParticleBuffer {
                float positions[];
            };
            void main() {
                uint index = gl_GlobalInvocationID.x;
                if (index >= positions.length()) return;
                // Physics computation would go here
            }
        "#;

        let rendering_shader_source = r#"
            #version 450
            layout(local_size_x = 64) in;
            layout(set = 0, binding = 0) buffer VertexBuffer {
                float vertices[];
            };
            void main() {
                uint index = gl_GlobalInvocationID.x;
                if (index >= vertices.length()) return;
                // Vertex processing would go here
            }
        "#;

        self.compile_compute_shader("physics_particles", physics_shader_source)?;
        self.compile_compute_shader("vertex_transform", rendering_shader_source)?;

        Ok(())
    }

    fn ensure_buffer_capacity(&mut self, name: &str, required_bytes: usize) -> RobinResult<()> {
        if let Some(buffer) = self.gpu_buffers.get(name) {
            if buffer.size_bytes >= required_bytes {
                return Ok(());
            }
            // Buffer exists but is too small, resize it
            self.release_buffer(name)?;
        }

        // Create new buffer with required capacity
        self.create_buffer(name, required_bytes, GPUBufferUsage::Compute)?;
        Ok(())
    }

    fn update_gpu_memory_manager(&mut self) -> RobinResult<()> {
        // Update memory usage statistics
        let total_allocated: usize = self.memory_manager.allocated_buffers.values().sum();
        self.memory_manager.used_memory_mb = total_allocated as f32 / 1_048_576.0;
        
        // Garbage collect unused buffers if memory usage is high
        if self.memory_manager.used_memory_mb > self.memory_manager.total_memory_mb * 0.9 {
            self.garbage_collect_buffers()?;
        }
        
        Ok(())
    }

    fn optimize_buffer_usage(&mut self, _delta_time: f32) -> RobinResult<()> {
        // Remove buffers that haven't been used recently
        let buffer_names: Vec<String> = self.gpu_buffers.keys().cloned().collect();
        let mut removed_count = 0;
        
        for name in buffer_names {
            if name.starts_with("temp_") && removed_count < 5 {
                self.release_buffer(&name)?;
                removed_count += 1;
            }
        }
        
        Ok(())
    }

    fn garbage_collect_buffers(&mut self) -> RobinResult<()> {
        // Implement aggressive garbage collection when memory is low
        let buffer_names: Vec<String> = self.gpu_buffers.keys().cloned().collect();
        let mut freed_mb = 0.0;
        
        for name in buffer_names {
            if name.contains("cache") || name.starts_with("temp_") {
                if let Some(buffer) = self.gpu_buffers.get(&name) {
                    freed_mb += buffer.size_bytes as f32 / 1_048_576.0;
                    self.release_buffer(&name)?;
                    
                    if freed_mb > 500.0 { // Stop after freeing 500MB
                        break;
                    }
                }
            }
        }
        
        Ok(())
    }

    pub fn set_quality_multiplier(&mut self, multiplier: f32) -> RobinResult<()> {
        self.quality_multiplier = multiplier.clamp(0.1, 4.0);
        Ok(())
    }

    pub fn get_memory_usage_mb(&self) -> f32 {
        self.memory_manager.used_memory_mb
    }

    pub fn get_total_memory_mb(&self) -> f32 {
        self.memory_manager.total_memory_mb
    }

    pub fn get_buffer_count(&self) -> usize {
        self.gpu_buffers.len()
    }

    pub fn get_shader_count(&self) -> usize {
        self.compute_shaders.len()
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn enable(&mut self) -> RobinResult<()> {
        if !self.enabled {
            self.enabled = true;
            self.initialize_compute_shaders()?;
        }
        Ok(())
    }

    pub fn disable(&mut self) -> RobinResult<()> {
        if self.enabled {
            self.shutdown()?;
            self.enabled = false;
        }
        Ok(())
    }

    // New public API methods to match the performance system interface
    pub fn submit_task(&mut self, task: GPUTask) -> RobinResult<String> {
        let task_id = task.id.clone();
        self.task_queue.push_back(task);
        Ok(task_id)
    }

    pub fn get_task_result(&mut self, task_id: &str) -> Option<TaskResult> {
        self.completed_results.remove(task_id)
    }

    pub fn is_task_complete(&self, task_id: &str) -> bool {
        self.completed_results.contains_key(task_id)
    }

    pub fn get_metrics(&self) -> &GPUMetrics {
        &self.metrics
    }

    pub fn get_completed_task_count(&self) -> u32 {
        self.metrics.completed_tasks
    }


    pub fn shutdown(&mut self) -> RobinResult<()> {
        if self.enabled {
            println!("GPU Accelerator shutdown:");
            println!("  Tasks completed: {}", self.metrics.completed_tasks);
            println!("  Tasks failed: {}", self.metrics.failed_tasks);
            println!("  Peak GPU memory: {:.1}MB", self.metrics.gpu_memory_usage_mb);
            println!("  Buffers allocated: {}", self.metrics.buffer_allocation_count);
            println!("  Average task time: {:.1}ms", self.metrics.average_task_time_ms);
        }

        self.compute_shaders.clear();
        self.gpu_buffers.clear();
        self.task_queue.clear();
        self.executing_tasks.clear();
        self.completed_results.clear();
        self.memory_manager.allocated_buffers.clear();
        self.memory_manager.used_memory_mb = 0.0;
        Ok(())
    }
}