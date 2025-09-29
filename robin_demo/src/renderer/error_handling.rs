// Metal renderer error handling and recovery system
use std::fmt;

#[derive(Debug, Clone)]
pub enum MetalError {
    DeviceNotFound,
    ShaderCompilationFailed(String),
    PipelineCreationFailed(String),
    BufferCreationFailed(String),
    TextureCreationFailed(String),
    DrawableAcquisitionFailed,
    CommandBufferCreationFailed,
    RenderEncoderCreationFailed,
    ResourceValidationFailed(String),
    MemoryAllocationFailed,
    DeviceLost,
}

impl fmt::Display for MetalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetalError::DeviceNotFound => write!(f, "No Metal device found"),
            MetalError::ShaderCompilationFailed(msg) => write!(f, "Shader compilation failed: {}", msg),
            MetalError::PipelineCreationFailed(msg) => write!(f, "Pipeline creation failed: {}", msg),
            MetalError::BufferCreationFailed(msg) => write!(f, "Buffer creation failed: {}", msg),
            MetalError::TextureCreationFailed(msg) => write!(f, "Texture creation failed: {}", msg),
            MetalError::DrawableAcquisitionFailed => write!(f, "Failed to acquire drawable"),
            MetalError::CommandBufferCreationFailed => write!(f, "Failed to create command buffer"),
            MetalError::RenderEncoderCreationFailed => write!(f, "Failed to create render encoder"),
            MetalError::ResourceValidationFailed(msg) => write!(f, "Resource validation failed: {}", msg),
            MetalError::MemoryAllocationFailed => write!(f, "Metal memory allocation failed"),
            MetalError::DeviceLost => write!(f, "Metal device was lost"),
        }
    }
}

impl std::error::Error for MetalError {}

pub type MetalResult<T> = Result<T, MetalError>;

/// Error recovery strategies for Metal operations
pub struct ErrorRecovery;

impl ErrorRecovery {
    /// Attempt to recover from device loss by recreating resources
    pub fn recover_from_device_loss() -> MetalResult<()> {
        log::warn!("🔄 Attempting to recover from Metal device loss");
        // Implementation would recreate the Metal device and all resources
        // For now, we'll return an error to indicate recovery is needed
        Err(MetalError::DeviceLost)
    }

    /// Validate Metal resource before use
    pub fn validate_resource<T>(resource: &Option<T>, resource_name: &str) -> MetalResult<()> {
        match resource {
            Some(_) => Ok(()),
            None => Err(MetalError::ResourceValidationFailed(
                format!("{} is not initialized", resource_name)
            )),
        }
    }

    /// Retry operation with exponential backoff
    pub fn retry_with_backoff<F, T>(
        mut operation: F,
        max_retries: u32,
        operation_name: &str,
    ) -> MetalResult<T>
    where
        F: FnMut() -> MetalResult<T>,
    {
        let mut retries = 0;
        loop {
            match operation() {
                Ok(result) => return Ok(result),
                Err(e) => {
                    retries += 1;
                    if retries > max_retries {
                        log::error!("❌ {} failed after {} retries: {}", operation_name, max_retries, e);
                        return Err(e);
                    }

                    let delay = std::time::Duration::from_millis(100 * (1 << retries));
                    log::warn!("⚠️  {} failed (attempt {}), retrying in {:?}: {}",
                              operation_name, retries, delay, e);
                    std::thread::sleep(delay);
                }
            }
        }
    }

    /// Graceful shutdown procedure for Metal resources
    pub fn graceful_shutdown() {
        log::info!("🔄 Initiating graceful Metal renderer shutdown");
        // This would properly release all Metal resources
        // and wait for pending operations to complete
    }
}

/// Performance monitoring for Metal operations
pub struct PerformanceMonitor {
    frame_times: Vec<f32>,
    draw_calls: u32,
    vertex_count: u32,
    triangle_count: u32,
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            frame_times: Vec::with_capacity(60), // Store last 60 frames
            draw_calls: 0,
            vertex_count: 0,
            triangle_count: 0,
        }
    }

    pub fn record_frame_time(&mut self, frame_time: f32) {
        self.frame_times.push(frame_time);
        if self.frame_times.len() > 60 {
            self.frame_times.remove(0);
        }
    }

    pub fn record_draw_call(&mut self, vertices: u32, triangles: u32) {
        self.draw_calls += 1;
        self.vertex_count += vertices;
        self.triangle_count += triangles;
    }

    pub fn reset_frame_stats(&mut self) {
        self.draw_calls = 0;
        self.vertex_count = 0;
        self.triangle_count = 0;
    }

    pub fn record_culling_stats(&mut self, total_chunks: usize, visible_chunks: usize, culled_chunks: usize) {
        // Store culling stats in the context for debugging/profiling
        // For now, we'll just log them occasionally for monitoring
        if self.frame_times.len() % 60 == 0 { // Log every 60 frames
            log::debug!("Culling stats: {}/{} chunks visible ({} culled)",
                       visible_chunks, total_chunks, culled_chunks);
        }
    }

    pub fn get_average_fps(&self) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        let avg_frame_time: f32 = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
        if avg_frame_time > 0.0 {
            1.0 / avg_frame_time
        } else {
            0.0
        }
    }

    pub fn get_frame_stats(&self) -> (u32, u32, u32) {
        (self.draw_calls, self.vertex_count, self.triangle_count)
    }

    pub fn log_performance_warning(&self) {
        let fps = self.get_average_fps();
        let (draw_calls, vertices, triangles) = self.get_frame_stats();

        if fps < 30.0 {
            log::warn!("⚠️  Low FPS detected: {:.1} fps", fps);
        }
        if draw_calls > 1000 {
            log::warn!("⚠️  High draw call count: {} calls", draw_calls);
        }
        if vertices > 1_000_000 {
            log::warn!("⚠️  High vertex count: {} vertices", vertices);
        }
    }
}

/// Resource validation utilities
pub struct ResourceValidator;

impl ResourceValidator {
    pub fn validate_texture_size(width: u32, height: u32) -> MetalResult<()> {
        const MAX_TEXTURE_SIZE: u32 = 16384; // Common Metal limit

        if width == 0 || height == 0 {
            return Err(MetalError::TextureCreationFailed(
                "Texture dimensions cannot be zero".to_string()
            ));
        }

        if width > MAX_TEXTURE_SIZE || height > MAX_TEXTURE_SIZE {
            return Err(MetalError::TextureCreationFailed(
                format!("Texture size {}x{} exceeds maximum {}", width, height, MAX_TEXTURE_SIZE)
            ));
        }

        Ok(())
    }

    pub fn validate_vertex_count(vertex_count: usize) -> MetalResult<()> {
        const MAX_VERTICES: usize = 10_000_000; // Reasonable limit for performance

        if vertex_count > MAX_VERTICES {
            return Err(MetalError::ResourceValidationFailed(
                format!("Vertex count {} exceeds maximum {}", vertex_count, MAX_VERTICES)
            ));
        }

        Ok(())
    }

    pub fn validate_buffer_size(size: u64) -> MetalResult<()> {
        const MAX_BUFFER_SIZE: u64 = 268_435_456; // 256MB limit

        if size == 0 {
            return Err(MetalError::BufferCreationFailed(
                "Buffer size cannot be zero".to_string()
            ));
        }

        if size > MAX_BUFFER_SIZE {
            return Err(MetalError::BufferCreationFailed(
                format!("Buffer size {} exceeds maximum {}", size, MAX_BUFFER_SIZE)
            ));
        }

        Ok(())
    }
}