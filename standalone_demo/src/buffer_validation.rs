/*!
 * Robin Engine - Buffer Overflow Protection and Validation
 *
 * Provides comprehensive buffer safety mechanisms:
 * - Pre-allocation buffer size validation
 * - Runtime bounds checking
 * - Memory pressure monitoring
 * - Safe buffer operations with overflow prevention
 * - Performance metrics for buffer usage
 */

use crate::error::{RobinError, RobinResult};
use log::{warn, info, debug};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Buffer usage statistics for monitoring
#[derive(Debug, Clone, Default)]
pub struct BufferUsageStats {
    pub vertex_buffer_usage: u64,
    pub index_buffer_usage: u64,
    pub uniform_buffer_usage: u64,
    pub total_gpu_memory: u64,
    pub peak_usage: u64,
    pub allocation_count: u32,
    pub overflow_prevention_count: u32,
}

/// Buffer size limits for different types
#[derive(Debug, Clone)]
pub struct BufferLimits {
    pub max_vertex_buffer_size: u64,
    pub max_index_buffer_size: u64,
    pub max_uniform_buffer_size: u64,
    pub max_total_gpu_memory: u64,
    pub warning_threshold: f32, // Percentage at which to warn
}

impl Default for BufferLimits {
    fn default() -> Self {
        Self {
            max_vertex_buffer_size: 4 * 1024 * 1024,      // 4MB
            max_index_buffer_size: 4 * 1024 * 1024,       // 4MB
            max_uniform_buffer_size: 64 * 1024,           // 64KB
            max_total_gpu_memory: 256 * 1024 * 1024,      // 256MB
            warning_threshold: 0.80,                       // 80%
        }
    }
}

/// Buffer validation and protection system
pub struct BufferValidator {
    limits: BufferLimits,
    usage_stats: Arc<Mutex<BufferUsageStats>>,
    allocation_history: Arc<Mutex<HashMap<String, Vec<u64>>>>,
}

/// Safe buffer operation wrapper
pub struct SafeBufferOperation<'a> {
    validator: &'a BufferValidator,
    buffer_type: BufferType,
    operation_name: String,
}

/// Types of buffers for validation
#[derive(Debug, Clone, Copy)]
pub enum BufferType {
    Vertex,
    Index,
    Uniform,
    Generic,
}

impl BufferValidator {
    /// Create a new buffer validator with default limits
    pub fn new() -> Self {
        Self::with_limits(BufferLimits::default())
    }

    /// Create a buffer validator with custom limits
    pub fn with_limits(limits: BufferLimits) -> Self {
        info!("🛡️ Initializing buffer validator with limits: Vertex={}MB, Index={}MB, Uniform={}KB, Total={}MB",
              limits.max_vertex_buffer_size / (1024 * 1024),
              limits.max_index_buffer_size / (1024 * 1024),
              limits.max_uniform_buffer_size / 1024,
              limits.max_total_gpu_memory / (1024 * 1024));

        Self {
            limits,
            usage_stats: Arc::new(Mutex::new(BufferUsageStats::default())),
            allocation_history: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Validate a buffer allocation before creation
    pub fn validate_allocation(
        &self,
        buffer_type: BufferType,
        requested_size: u64,
        operation_name: &str,
    ) -> RobinResult<()> {
        // Check individual buffer type limits
        let type_limit = match buffer_type {
            BufferType::Vertex => self.limits.max_vertex_buffer_size,
            BufferType::Index => self.limits.max_index_buffer_size,
            BufferType::Uniform => self.limits.max_uniform_buffer_size,
            BufferType::Generic => self.limits.max_vertex_buffer_size, // Use vertex limit as default
        };

        if requested_size > type_limit {
            return Err(RobinError::Buffer {
                message: format!("Buffer size exceeds type limit for {:?}", buffer_type),
                capacity: type_limit as usize,
                attempted: requested_size as usize,
                operation: operation_name.to_string(),
            });
        }

        // Check total memory usage
        let current_total = {
            let stats = self.usage_stats.lock().map_err(|_| {
                RobinError::Buffer {
                    message: "Failed to acquire usage stats lock".to_string(),
                    capacity: 0,
                    attempted: requested_size as usize,
                    operation: operation_name.to_string(),
                }
            })?;
            stats.total_gpu_memory
        };

        if current_total + requested_size > self.limits.max_total_gpu_memory {
            return Err(RobinError::Buffer {
                message: "Total GPU memory limit would be exceeded".to_string(),
                capacity: self.limits.max_total_gpu_memory as usize,
                attempted: (current_total + requested_size) as usize,
                operation: operation_name.to_string(),
            });
        }

        // Check warning threshold
        let usage_percentage = (current_total + requested_size) as f32 / self.limits.max_total_gpu_memory as f32;
        if usage_percentage > self.limits.warning_threshold {
            warn!("⚠️ GPU memory usage approaching limit: {:.1}% ({} / {} MB) for operation '{}'",
                  usage_percentage * 100.0,
                  (current_total + requested_size) / (1024 * 1024),
                  self.limits.max_total_gpu_memory / (1024 * 1024),
                  operation_name);
        }

        debug!("✅ Buffer allocation validated: {} bytes for {:?} operation '{}'",
               requested_size, buffer_type, operation_name);

        Ok(())
    }

    /// Register a successful buffer allocation
    pub fn register_allocation(
        &self,
        buffer_type: BufferType,
        allocated_size: u64,
        operation_name: &str,
    ) -> RobinResult<()> {
        let mut stats = self.usage_stats.lock().map_err(|_| {
            RobinError::Buffer {
                message: "Failed to acquire usage stats lock for registration".to_string(),
                capacity: 0,
                attempted: allocated_size as usize,
                operation: operation_name.to_string(),
            }
        })?;

        // Update specific buffer type usage
        match buffer_type {
            BufferType::Vertex => stats.vertex_buffer_usage += allocated_size,
            BufferType::Index => stats.index_buffer_usage += allocated_size,
            BufferType::Uniform => stats.uniform_buffer_usage += allocated_size,
            BufferType::Generic => {}, // Don't track generic buffers separately
        }

        stats.total_gpu_memory += allocated_size;
        stats.allocation_count += 1;

        if stats.total_gpu_memory > stats.peak_usage {
            stats.peak_usage = stats.total_gpu_memory;
        }

        // Record allocation in history
        if let Ok(mut history) = self.allocation_history.lock() {
            let entry = history.entry(operation_name.to_string()).or_insert_with(Vec::new);
            entry.push(allocated_size);
            // Keep only last 100 allocations per operation
            if entry.len() > 100 {
                entry.remove(0);
            }
        }

        debug!("📊 Buffer allocation registered: {} bytes for {:?} (Total: {} MB)",
               allocated_size, buffer_type, stats.total_gpu_memory / (1024 * 1024));

        Ok(())
    }

    /// Register a buffer deallocation
    pub fn register_deallocation(
        &self,
        buffer_type: BufferType,
        deallocated_size: u64,
    ) -> RobinResult<()> {
        let mut stats = self.usage_stats.lock().map_err(|_| {
            RobinError::Buffer {
                message: "Failed to acquire usage stats lock for deallocation".to_string(),
                capacity: 0,
                attempted: deallocated_size as usize,
                operation: "deallocation".to_string(),
            }
        })?;

        // Update specific buffer type usage
        match buffer_type {
            BufferType::Vertex => stats.vertex_buffer_usage = stats.vertex_buffer_usage.saturating_sub(deallocated_size),
            BufferType::Index => stats.index_buffer_usage = stats.index_buffer_usage.saturating_sub(deallocated_size),
            BufferType::Uniform => stats.uniform_buffer_usage = stats.uniform_buffer_usage.saturating_sub(deallocated_size),
            BufferType::Generic => {}, // Don't track generic buffers separately
        }

        stats.total_gpu_memory = stats.total_gpu_memory.saturating_sub(deallocated_size);

        debug!("📉 Buffer deallocation registered: {} bytes for {:?} (Total: {} MB)",
               deallocated_size, buffer_type, stats.total_gpu_memory / (1024 * 1024));

        Ok(())
    }

    /// Validate data fits within buffer bounds
    pub fn validate_data_write(
        &self,
        buffer_size: u64,
        write_offset: u64,
        data_size: u64,
        operation_name: &str,
    ) -> RobinResult<()> {
        let end_position = write_offset + data_size;

        if end_position > buffer_size {
            return Err(RobinError::Buffer {
                message: format!("Data write would overflow buffer bounds"),
                capacity: buffer_size as usize,
                attempted: end_position as usize,
                operation: operation_name.to_string(),
            });
        }

        if write_offset >= buffer_size {
            return Err(RobinError::Buffer {
                message: format!("Write offset beyond buffer end"),
                capacity: buffer_size as usize,
                attempted: write_offset as usize,
                operation: operation_name.to_string(),
            });
        }

        Ok(())
    }

    /// Validate vertex/index data consistency
    pub fn validate_vertex_index_consistency(
        &self,
        vertex_count: u32,
        indices: &[u16],
        operation_name: &str,
    ) -> RobinResult<()> {
        for (i, &index) in indices.iter().enumerate() {
            if index as u32 >= vertex_count {
                return Err(RobinError::Validation {
                    message: format!("Index {} points to non-existent vertex {}, max vertex index is {}",
                                    i, index, vertex_count.saturating_sub(1)),
                    field: "index_buffer".to_string(),
                    provided_value: index.to_string(),
                    expected_format: Some(format!("0..{}", vertex_count.saturating_sub(1))),
                });
            }
        }

        debug!("✅ Vertex-index consistency validated: {} vertices, {} indices for operation '{}'",
               vertex_count, indices.len(), operation_name);

        Ok(())
    }

    /// Create a safe buffer operation guard
    pub fn create_operation<'a>(
        &'a self,
        buffer_type: BufferType,
        operation_name: impl Into<String>,
    ) -> SafeBufferOperation<'a> {
        SafeBufferOperation {
            validator: self,
            buffer_type,
            operation_name: operation_name.into(),
        }
    }

    /// Get current buffer usage statistics
    pub fn get_usage_stats(&self) -> BufferUsageStats {
        match self.usage_stats.lock() {
            Ok(stats) => stats.clone(),
            Err(_) => BufferUsageStats::default()
        }
    }

    /// Get usage summary for debugging
    pub fn get_usage_summary(&self) -> String {
        let stats = self.get_usage_stats();
        let total_mb = stats.total_gpu_memory as f64 / (1024.0 * 1024.0);
        let peak_mb = stats.peak_usage as f64 / (1024.0 * 1024.0);
        let limit_mb = self.limits.max_total_gpu_memory as f64 / (1024.0 * 1024.0);
        let usage_percent = (stats.total_gpu_memory as f64 / self.limits.max_total_gpu_memory as f64) * 100.0;

        format!(
            "🛡️ Buffer Usage: {:.2} MB / {:.2} MB ({:.1}%) | Peak: {:.2} MB | Allocations: {} | Overflows Prevented: {}",
            total_mb, limit_mb, usage_percent, peak_mb, stats.allocation_count, stats.overflow_prevention_count
        )
    }

    /// Check if system is approaching memory limits
    pub fn is_memory_pressure_high(&self) -> bool {
        if let Ok(stats) = self.usage_stats.lock() {
            let usage_ratio = stats.total_gpu_memory as f32 / self.limits.max_total_gpu_memory as f32;
            usage_ratio > self.limits.warning_threshold
        } else {
            false
        }
    }

    /// Get memory pressure level (0.0 to 1.0)
    pub fn get_memory_pressure(&self) -> f32 {
        if let Ok(stats) = self.usage_stats.lock() {
            (stats.total_gpu_memory as f32 / self.limits.max_total_gpu_memory as f32).min(1.0)
        } else {
            0.0
        }
    }
}

impl<'a> SafeBufferOperation<'a> {
    /// Validate and register an allocation with this operation
    pub fn allocate(&self, size: u64) -> RobinResult<()> {
        self.validator.validate_allocation(self.buffer_type, size, &self.operation_name)?;
        self.validator.register_allocation(self.buffer_type, size, &self.operation_name)?;
        Ok(())
    }

    /// Validate a data write operation
    pub fn validate_write(&self, buffer_size: u64, offset: u64, data_size: u64) -> RobinResult<()> {
        self.validator.validate_data_write(buffer_size, offset, data_size, &self.operation_name)
    }

    /// Get the buffer type for this operation
    pub fn buffer_type(&self) -> BufferType {
        self.buffer_type
    }

    /// Get the operation name
    pub fn operation_name(&self) -> &str {
        &self.operation_name
    }
}

impl Default for BufferValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for safe buffer operations
pub mod safe_operations {
    use super::*;

    /// Safely calculate vertex buffer size with overflow protection
    pub fn calculate_vertex_buffer_size<T>(vertex_count: usize, operation_name: &str) -> RobinResult<u64> {
        let vertex_size = std::mem::size_of::<T>() as u64;
        let total_size = vertex_count as u64 * vertex_size;

        // Check for multiplication overflow
        if vertex_count > 0 && total_size / vertex_count as u64 != vertex_size {
            return Err(RobinError::Buffer {
                message: "Vertex buffer size calculation would overflow".to_string(),
                capacity: usize::MAX,
                attempted: vertex_count,
                operation: operation_name.to_string(),
            });
        }

        Ok(total_size)
    }

    /// Safely calculate index buffer size with overflow protection
    pub fn calculate_index_buffer_size(index_count: usize, operation_name: &str) -> RobinResult<u64> {
        let index_size = std::mem::size_of::<u16>() as u64;
        let total_size = index_count as u64 * index_size;

        // Check for multiplication overflow
        if index_count > 0 && total_size / index_count as u64 != index_size {
            return Err(RobinError::Buffer {
                message: "Index buffer size calculation would overflow".to_string(),
                capacity: usize::MAX,
                attempted: index_count,
                operation: operation_name.to_string(),
            });
        }

        Ok(total_size)
    }

    /// Validate that slice data won't overflow when written to buffer
    pub fn validate_slice_write<T>(
        buffer_size: u64,
        offset: u64,
        data: &[T],
        operation_name: &str,
    ) -> RobinResult<()> {
        let data_size = data.len() as u64 * std::mem::size_of::<T>() as u64;
        let end_position = offset + data_size;

        if end_position > buffer_size {
            return Err(RobinError::Buffer {
                message: "Slice write would overflow buffer".to_string(),
                capacity: buffer_size as usize,
                attempted: end_position as usize,
                operation: operation_name.to_string(),
            });
        }

        Ok(())
    }
}