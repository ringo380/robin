/*!
 * Robin Engine GPU Buffer Management
 * 
 * Efficient GPU buffer allocation, pooling, and memory management system
 * with automatic lifetime tracking and defragmentation.
 */

use crate::engine::{
    graphics::GraphicsContext,
    error::{RobinError, RobinResult},
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

/// GPU buffer handle for safe resource access
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GPUBufferHandle(pub u32);

/// GPU texture handle for safe resource access
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GPUTextureHandle(pub u32);

/// Buffer pool for efficient GPU buffer management
#[derive(Debug)]
pub struct BufferPool {
    config: BufferPoolConfig,
    /// Active buffers currently in use
    active_buffers: HashMap<GPUBufferHandle, BufferEntry>,
    /// Free buffers available for reuse
    free_buffers: HashMap<BufferKey, VecDeque<PooledBuffer>>,
    /// Buffer allocator for new allocations
    allocator: BufferAllocator,
    /// Next available handle ID
    next_handle: u32,
    /// Frame-based garbage collection
    gc_queue: VecDeque<(GPUBufferHandle, u32)>, // (handle, frame_to_free)
    current_frame: u32,
}

impl BufferPool {
    pub fn new(config: BufferPoolConfig) -> Self {
        Self {
            allocator: BufferAllocator::new(config.allocator_config.clone()),
            free_buffers: HashMap::new(),
            active_buffers: HashMap::new(),
            next_handle: 1,
            gc_queue: VecDeque::new(),
            current_frame: 0,
            config,
        }
    }

    /// Create a new GPU buffer
    pub fn create_buffer(&mut self, graphics_context: &GraphicsContext, desc: BufferDescriptor) -> RobinResult<GPUBufferHandle> {
        let handle = GPUBufferHandle(self.next_handle);
        self.next_handle += 1;

        // Try to find a suitable buffer from the pool
        let buffer_key = BufferKey::from_descriptor(&desc);
        if let Some(pooled_buffer) = self.get_pooled_buffer(&buffer_key) {
            let entry = BufferEntry {
                buffer: pooled_buffer.buffer,
                descriptor: desc,
                allocated_frame: self.current_frame,
                last_used_frame: self.current_frame,
                mapped_data: None,
                usage_count: 1,
            };

            self.active_buffers.insert(handle, entry);
            return Ok(handle);
        }

        // Allocate new buffer
        let buffer = self.allocator.allocate_buffer(graphics_context, &desc)?;

        let entry = BufferEntry {
            buffer,
            descriptor: desc,
            allocated_frame: self.current_frame,
            last_used_frame: self.current_frame,
            mapped_data: None,
            usage_count: 1,
        };

        self.active_buffers.insert(handle, entry);
        Ok(handle)
    }

    /// Release a buffer back to the pool
    pub fn release_buffer(&mut self, handle: GPUBufferHandle) {
        if let Some(mut entry) = self.active_buffers.remove(&handle) {
            // Reset buffer if needed
            if self.config.clear_on_release {
                // Would clear buffer contents in real implementation
            }

            let buffer_key = BufferKey::from_descriptor(&entry.descriptor);
            let pooled_buffer = PooledBuffer {
                buffer: entry.buffer,
                last_used_frame: self.current_frame,
                reuse_count: 0,
            };

            self.free_buffers.entry(buffer_key)
                .or_insert_with(VecDeque::new)
                .push_back(pooled_buffer);
        }
    }

    /// Map buffer for CPU access
    pub fn map_buffer<T>(&mut self, graphics_context: &GraphicsContext, handle: GPUBufferHandle) -> RobinResult<BufferMapping<T>> {
        let entry = self.active_buffers.get_mut(&handle)
            .ok_or_else(|| RobinError::BufferError(format!("Invalid buffer handle: {:?}", handle)))?;

        if entry.mapped_data.is_some() {
            return Err(RobinError::BufferError("Buffer is already mapped".to_string()));
        }

        // Map the buffer memory
        let mapped_ptr = self.allocator.map_buffer(graphics_context, &entry.buffer)?;
        let mapped_data = MappedBufferData {
            ptr: mapped_ptr,
            size: entry.descriptor.size,
        };

        entry.mapped_data = Some(mapped_data);
        entry.last_used_frame = self.current_frame;

        Ok(BufferMapping {
            handle,
            data: unsafe { std::slice::from_raw_parts_mut(mapped_ptr as *mut T, entry.descriptor.size / std::mem::size_of::<T>()) },
            _phantom: std::marker::PhantomData,
        })
    }

    /// Unmap buffer
    pub fn unmap_buffer(&mut self, graphics_context: &GraphicsContext, handle: GPUBufferHandle) -> RobinResult<()> {
        let entry = self.active_buffers.get_mut(&handle)
            .ok_or_else(|| RobinError::BufferError(format!("Invalid buffer handle: {:?}", handle)))?;

        if entry.mapped_data.is_none() {
            return Err(RobinError::BufferError("Buffer is not mapped".to_string()));
        }

        self.allocator.unmap_buffer(graphics_context, &entry.buffer)?;
        entry.mapped_data = None;

        Ok(())
    }

    /// Upload data to buffer
    pub fn upload_data<T>(&mut self, graphics_context: &GraphicsContext, handle: GPUBufferHandle, data: &[T]) -> RobinResult<()> {
        let entry = self.active_buffers.get_mut(&handle)
            .ok_or_else(|| RobinError::BufferError(format!("Invalid buffer handle: {:?}", handle)))?;

        let data_size = data.len() * std::mem::size_of::<T>();
        if data_size > entry.descriptor.size {
            return Err(RobinError::BufferError(format!("Data size {} exceeds buffer size {}", data_size, entry.descriptor.size)));
        }

        self.allocator.upload_data(graphics_context, &entry.buffer, data)?;
        entry.last_used_frame = self.current_frame;
        entry.usage_count += 1;

        Ok(())
    }

    /// Download data from buffer
    pub fn download_data<T>(&mut self, graphics_context: &GraphicsContext, handle: GPUBufferHandle, data: &mut [T]) -> RobinResult<()> {
        let entry = self.active_buffers.get_mut(&handle)
            .ok_or_else(|| RobinError::BufferError(format!("Invalid buffer handle: {:?}", handle)))?;

        let data_size = data.len() * std::mem::size_of::<T>();
        if data_size > entry.descriptor.size {
            return Err(RobinError::BufferError(format!("Data size {} exceeds buffer size {}", data_size, entry.descriptor.size)));
        }

        self.allocator.download_data(graphics_context, &entry.buffer, data)?;
        entry.last_used_frame = self.current_frame;

        Ok(())
    }

    /// Begin new frame
    pub fn begin_frame(&mut self) {
        self.current_frame += 1;
    }

    /// End frame and perform cleanup
    pub fn end_frame(&mut self, graphics_context: &GraphicsContext) -> RobinResult<()> {
        // Process garbage collection queue
        self.process_gc_queue(graphics_context)?;

        // Clean up old pooled buffers
        self.cleanup_old_buffers(graphics_context)?;

        // Defragment if needed
        if self.current_frame % self.config.defrag_frequency == 0 {
            self.defragment_memory(graphics_context)?;
        }

        Ok(())
    }

    /// Get buffer pool statistics
    pub fn get_stats(&self) -> BufferPoolStats {
        let total_active = self.active_buffers.len();
        let total_free: usize = self.free_buffers.values().map(|queue| queue.len()).sum();
        let total_active_memory: usize = self.active_buffers.values().map(|entry| entry.descriptor.size).sum();
        let total_free_memory: usize = self.free_buffers.values()
            .flat_map(|queue| queue.iter())
            .map(|buffer| buffer.buffer.size)
            .sum();

        BufferPoolStats {
            active_buffers: total_active,
            free_buffers: total_free,
            total_memory_active: total_active_memory,
            total_memory_free: total_free_memory,
            fragmentation_ratio: self.calculate_fragmentation_ratio(),
            gc_queue_size: self.gc_queue.len(),
        }
    }

    fn get_pooled_buffer(&mut self, key: &BufferKey) -> Option<PooledBuffer> {
        if let Some(queue) = self.free_buffers.get_mut(key) {
            if let Some(mut buffer) = queue.pop_front() {
                buffer.reuse_count += 1;
                buffer.last_used_frame = self.current_frame;
                return Some(buffer);
            }
        }
        None
    }

    fn process_gc_queue(&mut self, graphics_context: &GraphicsContext) -> RobinResult<()> {
        while let Some((handle, frame_to_free)) = self.gc_queue.front() {
            if self.current_frame >= *frame_to_free {
                let (handle, _) = self.gc_queue.pop_front().unwrap();
                if let Some(entry) = self.active_buffers.remove(&handle) {
                    self.allocator.deallocate_buffer(graphics_context, entry.buffer)?;
                }
            } else {
                break;
            }
        }
        Ok(())
    }

    fn cleanup_old_buffers(&mut self, graphics_context: &GraphicsContext) -> RobinResult<()> {
        let max_age = self.config.max_unused_frames;
        let current_frame = self.current_frame;

        for queue in self.free_buffers.values_mut() {
            queue.retain(|buffer| {
                let age = current_frame.saturating_sub(buffer.last_used_frame);
                if age > max_age {
                    // Would deallocate buffer in real implementation
                    false
                } else {
                    true
                }
            });
        }

        // Remove empty queues
        self.free_buffers.retain(|_, queue| !queue.is_empty());

        Ok(())
    }

    fn defragment_memory(&mut self, graphics_context: &GraphicsContext) -> RobinResult<()> {
        // Memory defragmentation would be implemented here
        // This is a complex operation that would reorganize buffer memory
        Ok(())
    }

    fn calculate_fragmentation_ratio(&self) -> f32 {
        // Calculate memory fragmentation ratio
        // This would analyze the distribution of free memory blocks
        0.1 // Placeholder
    }
}

/// Buffer allocator for managing GPU memory
#[derive(Debug)]
pub struct BufferAllocator {
    config: BufferAllocatorConfig,
    /// Memory heaps for different buffer types
    heaps: HashMap<MemoryType, MemoryHeap>,
    /// Allocation statistics
    stats: AllocationStats,
}

impl BufferAllocator {
    fn new(config: BufferAllocatorConfig) -> Self {
        let mut heaps = HashMap::new();
        
        // Initialize memory heaps
        heaps.insert(MemoryType::DeviceLocal, MemoryHeap::new(MemoryType::DeviceLocal, config.device_heap_size));
        heaps.insert(MemoryType::HostVisible, MemoryHeap::new(MemoryType::HostVisible, config.host_heap_size));
        heaps.insert(MemoryType::HostCoherent, MemoryHeap::new(MemoryType::HostCoherent, config.host_heap_size));

        Self {
            config,
            heaps,
            stats: AllocationStats::default(),
        }
    }

    fn allocate_buffer(&mut self, graphics_context: &GraphicsContext, desc: &BufferDescriptor) -> RobinResult<AllocatedBuffer> {
        let heap = self.heaps.get_mut(&desc.memory_type)
            .ok_or_else(|| RobinError::BufferError(format!("Unsupported memory type: {:?}", desc.memory_type)))?;

        let allocation = heap.allocate(desc.size, self.config.alignment)?;
        
        // Create the actual buffer object
        // In a real implementation, this would call graphics API
        let buffer = AllocatedBuffer {
            memory_type: desc.memory_type,
            size: desc.size,
            offset: allocation.offset,
            allocation_id: allocation.id,
        };

        self.stats.total_allocations += 1;
        self.stats.total_allocated_bytes += desc.size;

        Ok(buffer)
    }

    fn deallocate_buffer(&mut self, graphics_context: &GraphicsContext, buffer: AllocatedBuffer) -> RobinResult<()> {
        let heap = self.heaps.get_mut(&buffer.memory_type)
            .ok_or_else(|| RobinError::BufferError("Invalid memory type".to_string()))?;

        heap.deallocate(buffer.allocation_id)?;
        
        self.stats.total_deallocations += 1;
        self.stats.total_deallocated_bytes += buffer.size;

        Ok(())
    }

    fn map_buffer(&self, graphics_context: &GraphicsContext, buffer: &AllocatedBuffer) -> RobinResult<*mut u8> {
        // Map buffer memory for CPU access
        // In a real implementation, this would call graphics API
        Ok(std::ptr::null_mut()) // Placeholder
    }

    fn unmap_buffer(&self, graphics_context: &GraphicsContext, buffer: &AllocatedBuffer) -> RobinResult<()> {
        // Unmap buffer memory
        // In a real implementation, this would call graphics API
        Ok(())
    }

    fn upload_data<T>(&self, graphics_context: &GraphicsContext, buffer: &AllocatedBuffer, data: &[T]) -> RobinResult<()> {
        // Upload data to buffer
        // In a real implementation, this would call graphics API
        Ok(())
    }

    fn download_data<T>(&self, graphics_context: &GraphicsContext, buffer: &AllocatedBuffer, data: &mut [T]) -> RobinResult<()> {
        // Download data from buffer
        // In a real implementation, this would call graphics API
        Ok(())
    }
}

/// Memory heap for managing allocations
#[derive(Debug)]
pub struct MemoryHeap {
    memory_type: MemoryType,
    total_size: usize,
    used_size: usize,
    /// Free blocks available for allocation
    free_blocks: Vec<MemoryBlock>,
    /// Allocated blocks currently in use
    allocated_blocks: HashMap<AllocationId, MemoryBlock>,
    next_allocation_id: u32,
}

impl MemoryHeap {
    fn new(memory_type: MemoryType, total_size: usize) -> Self {
        let mut heap = Self {
            memory_type,
            total_size,
            used_size: 0,
            free_blocks: Vec::new(),
            allocated_blocks: HashMap::new(),
            next_allocation_id: 1,
        };

        // Initialize with one large free block
        heap.free_blocks.push(MemoryBlock {
            offset: 0,
            size: total_size,
        });

        heap
    }

    fn allocate(&mut self, size: usize, alignment: usize) -> RobinResult<MemoryAllocation> {
        let aligned_size = self.align_size(size, alignment);
        
        // Find suitable free block
        for i in 0..self.free_blocks.len() {
            let block = &self.free_blocks[i];
            
            if block.size >= aligned_size {
                let allocation_id = AllocationId(self.next_allocation_id);
                self.next_allocation_id += 1;

                let allocated_block = MemoryBlock {
                    offset: block.offset,
                    size: aligned_size,
                };

                // Remove or split the free block
                if block.size == aligned_size {
                    self.free_blocks.remove(i);
                } else {
                    self.free_blocks[i] = MemoryBlock {
                        offset: block.offset + aligned_size,
                        size: block.size - aligned_size,
                    };
                }

                // Extract offset before moving allocated_block
                let block_offset = allocated_block.offset;
                
                self.allocated_blocks.insert(allocation_id, allocated_block);
                self.used_size += aligned_size;

                return Ok(MemoryAllocation {
                    id: allocation_id,
                    offset: block_offset,
                    size: aligned_size,
                });
            }
        }

        Err(RobinError::GPUMemoryError(format!("Cannot allocate {} bytes", size)))
    }

    fn deallocate(&mut self, allocation_id: AllocationId) -> RobinResult<()> {
        let block = self.allocated_blocks.remove(&allocation_id)
            .ok_or_else(|| RobinError::BufferError("Invalid allocation ID".to_string()))?;

        self.used_size -= block.size;

        // Add back to free blocks and coalesce
        self.free_blocks.push(block);
        self.coalesce_free_blocks();

        Ok(())
    }

    fn align_size(&self, size: usize, alignment: usize) -> usize {
        (size + alignment - 1) & !(alignment - 1)
    }

    fn coalesce_free_blocks(&mut self) {
        // Sort by offset
        self.free_blocks.sort_by_key(|block| block.offset);

        let mut i = 0;
        while i + 1 < self.free_blocks.len() {
            let current = &self.free_blocks[i];
            let next = &self.free_blocks[i + 1];

            // Check if blocks are adjacent
            if current.offset + current.size == next.offset {
                // Coalesce blocks
                self.free_blocks[i].size += next.size;
                self.free_blocks.remove(i + 1);
            } else {
                i += 1;
            }
        }
    }
}

// Data structures and types

/// Buffer descriptor for creation
#[derive(Debug, Clone)]
pub struct BufferDescriptor {
    pub size: usize,
    pub usage: BufferUsage,
    pub memory_type: MemoryType,
    pub initial_data: Option<Vec<u8>>,
}

/// Texture descriptor for creation
#[derive(Debug, Clone)]
pub struct TextureDescriptor {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub format: TextureFormat,
    pub usage: TextureUsage,
    pub memory_type: MemoryType,
}

/// Buffer usage flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BufferUsage {
    Storage,
    Uniform,
    Vertex,
    Index,
    Transfer,
}

/// Texture usage flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureUsage {
    Storage,
    Sampled,
    RenderTarget,
    DepthStencil,
}

/// Memory types for buffers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryType {
    /// GPU-only memory (fastest)
    DeviceLocal,
    /// CPU-visible GPU memory (slower)
    HostVisible,
    /// CPU-visible coherent memory (slowest, but no sync needed)
    HostCoherent,
}

/// Texture formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureFormat {
    R8,
    RG8,
    RGBA8,
    R16F,
    RG16F,
    RGBA16F,
    R32F,
    RG32F,
    RGBA32F,
}

/// Buffer pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferPoolConfig {
    pub max_buffers: usize,
    pub max_unused_frames: u32,
    pub clear_on_release: bool,
    pub defrag_frequency: u32,
    pub allocator_config: BufferAllocatorConfig,
}

impl Default for BufferPoolConfig {
    fn default() -> Self {
        Self {
            max_buffers: 10000,
            max_unused_frames: 60, // 1 second at 60fps
            clear_on_release: true,
            defrag_frequency: 300, // Every 5 seconds at 60fps
            allocator_config: BufferAllocatorConfig::default(),
        }
    }
}

/// Buffer allocator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferAllocatorConfig {
    pub device_heap_size: usize,
    pub host_heap_size: usize,
    pub alignment: usize,
}

impl Default for BufferAllocatorConfig {
    fn default() -> Self {
        Self {
            device_heap_size: 1024 * 1024 * 1024, // 1GB
            host_heap_size: 512 * 1024 * 1024,    // 512MB
            alignment: 256, // Common GPU alignment
        }
    }
}

/// Active buffer entry in the pool
#[derive(Debug)]
struct BufferEntry {
    buffer: AllocatedBuffer,
    descriptor: BufferDescriptor,
    allocated_frame: u32,
    last_used_frame: u32,
    mapped_data: Option<MappedBufferData>,
    usage_count: u32,
}

/// Pooled buffer available for reuse
#[derive(Debug)]
struct PooledBuffer {
    buffer: AllocatedBuffer,
    last_used_frame: u32,
    reuse_count: u32,
}

/// Key for buffer pool lookup
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct BufferKey {
    size_class: usize, // Rounded size for pooling
    usage: BufferUsage,
    memory_type: MemoryType,
}

impl BufferKey {
    fn from_descriptor(desc: &BufferDescriptor) -> Self {
        Self {
            size_class: Self::size_to_class(desc.size),
            usage: desc.usage,
            memory_type: desc.memory_type,
        }
    }

    fn size_to_class(size: usize) -> usize {
        // Round up to next power of 2 for better pooling
        if size <= 1024 {
            size.next_power_of_two()
        } else {
            // For larger sizes, round to nearest 4KB
            ((size + 4095) / 4096) * 4096
        }
    }
}

/// Allocated buffer on GPU
#[derive(Debug, Clone)]
pub struct AllocatedBuffer {
    pub memory_type: MemoryType,
    pub size: usize,
    pub offset: usize,
    pub allocation_id: AllocationId,
}

/// Memory allocation tracking
#[derive(Debug, Clone)]
struct MemoryAllocation {
    id: AllocationId,
    offset: usize,
    size: usize,
}

/// Unique allocation identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct AllocationId(u32);

/// Memory block in heap
#[derive(Debug, Clone)]
struct MemoryBlock {
    offset: usize,
    size: usize,
}

/// Mapped buffer data for CPU access
#[derive(Debug)]
struct MappedBufferData {
    ptr: *mut u8,
    size: usize,
}

/// Buffer mapping for typed access
pub struct BufferMapping<'a, T> {
    handle: GPUBufferHandle,
    data: &'a mut [T],
    _phantom: std::marker::PhantomData<T>,
}

impl<'a, T> std::ops::Deref for BufferMapping<'a, T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<'a, T> std::ops::DerefMut for BufferMapping<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data
    }
}

/// Buffer pool statistics
#[derive(Debug, Clone)]
pub struct BufferPoolStats {
    pub active_buffers: usize,
    pub free_buffers: usize,
    pub total_memory_active: usize,
    pub total_memory_free: usize,
    pub fragmentation_ratio: f32,
    pub gc_queue_size: usize,
}

/// Allocation statistics
#[derive(Debug, Default)]
struct AllocationStats {
    total_allocations: u64,
    total_deallocations: u64,
    total_allocated_bytes: usize,
    total_deallocated_bytes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::graphics::MockGraphicsContext;

    #[test]
    fn test_buffer_pool_creation() {
        let config = BufferPoolConfig::default();
        let pool = BufferPool::new(config);
        
        let stats = pool.get_stats();
        assert_eq!(stats.active_buffers, 0);
        assert_eq!(stats.free_buffers, 0);
    }

    #[test]
    fn test_buffer_key_size_classes() {
        // Test small sizes round to powers of 2
        assert_eq!(BufferKey::size_to_class(100), 128);
        assert_eq!(BufferKey::size_to_class(1000), 1024);
        
        // Test large sizes round to 4KB boundaries
        assert_eq!(BufferKey::size_to_class(5000), 8192);
        assert_eq!(BufferKey::size_to_class(10000), 12288);
    }

    #[test]
    fn test_memory_heap_allocation() {
        let mut heap = MemoryHeap::new(MemoryType::DeviceLocal, 1024 * 1024);
        
        // Test allocation
        let alloc1 = heap.allocate(1024, 256).unwrap();
        assert_eq!(alloc1.size, 1024);
        assert_eq!(heap.used_size, 1024);
        
        // Test another allocation
        let alloc2 = heap.allocate(2048, 256).unwrap();
        assert_eq!(alloc2.size, 2048);
        assert_eq!(heap.used_size, 3072);
        
        // Test deallocation
        heap.deallocate(alloc1.id).unwrap();
        assert_eq!(heap.used_size, 2048);
    }

    #[test]
    fn test_memory_heap_coalescing() {
        let mut heap = MemoryHeap::new(MemoryType::DeviceLocal, 1024);
        
        // Allocate all memory in small chunks
        let alloc1 = heap.allocate(256, 1).unwrap();
        let alloc2 = heap.allocate(256, 1).unwrap();
        let alloc3 = heap.allocate(256, 1).unwrap();
        let alloc4 = heap.allocate(256, 1).unwrap();
        
        assert_eq!(heap.used_size, 1024);
        assert_eq!(heap.free_blocks.len(), 0);
        
        // Deallocate middle chunks
        heap.deallocate(alloc2.id).unwrap();
        heap.deallocate(alloc3.id).unwrap();
        
        // Should have coalesced into one free block
        assert_eq!(heap.free_blocks.len(), 1);
        assert_eq!(heap.free_blocks[0].size, 512);
    }

    #[test]
    fn test_buffer_descriptor_to_key() {
        let desc = BufferDescriptor {
            size: 1500,
            usage: BufferUsage::Storage,
            memory_type: MemoryType::DeviceLocal,
            initial_data: None,
        };
        
        let key = BufferKey::from_descriptor(&desc);
        assert_eq!(key.size_class, 2048); // Next power of 2
        assert_eq!(key.usage, BufferUsage::Storage);
        assert_eq!(key.memory_type, MemoryType::DeviceLocal);
    }
}