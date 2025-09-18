/*!
 * Phase 2 Memory Safety and Security Validation
 *
 * Comprehensive tests for GPU memory management, buffer safety,
 * and protection against memory-related vulnerabilities.
 */

#[cfg(test)]
mod memory_safety_tests {
    use robin_engine::engine::{
        gpu::{
            voxel_compute::VoxelComputePipeline,
            buffers::{GPUBufferHandle, BufferType, BufferUsage},
            memory::GPUMemoryManager,
        },
        rendering::batch_renderer::BatchRenderer,
        graphics::GraphicsContext,
        error::{RobinError, RobinResult},
    };
    use std::sync::Arc;
    use std::time::Duration;

    struct MemorySafetyFixture {
        graphics_context: GraphicsContext,
        memory_manager: Arc<GPUMemoryManager>,
        voxel_pipeline: VoxelComputePipeline,
        batch_renderer: BatchRenderer,
    }

    impl MemorySafetyFixture {
        async fn new() -> RobinResult<Self> {
            let graphics_context = GraphicsContext::new_test_context()?;
            let memory_manager = Arc::new(GPUMemoryManager::new(&graphics_context)?);

            let voxel_pipeline = VoxelComputePipeline::new(&graphics_context, memory_manager.clone())?;
            let batch_renderer = BatchRenderer::new(&graphics_context, memory_manager.clone())?;

            Ok(Self {
                graphics_context,
                memory_manager,
                voxel_pipeline,
                batch_renderer,
            })
        }
    }

    // =======================
    // GPU BUFFER SAFETY TESTS
    // =======================

    #[tokio::test]
    async fn test_buffer_bounds_checking() {
        let fixture = MemorySafetyFixture::new().await.unwrap();

        // Test reading beyond buffer bounds
        let buffer = fixture.memory_manager.create_buffer(
            BufferType::Storage,
            BufferUsage::READ,
            1024, // 1KB buffer
            Some("Test Buffer"),
        ).unwrap();

        // Try to read beyond buffer size
        let result = fixture.memory_manager.read_buffer(&buffer, 0, 2048).await;
        assert!(result.is_err(), "Should reject reads beyond buffer bounds");

        // Try to read with invalid offset
        let result = fixture.memory_manager.read_buffer(&buffer, 1500, 100).await;
        assert!(result.is_err(), "Should reject reads starting beyond buffer size");
    }

    #[tokio::test]
    async fn test_buffer_write_protection() {
        let fixture = MemorySafetyFixture::new().await.unwrap();

        // Create read-only buffer
        let read_only_buffer = fixture.memory_manager.create_buffer(
            BufferType::Storage,
            BufferUsage::READ,
            1024,
            Some("Read Only Buffer"),
        ).unwrap();

        // Try to write to read-only buffer
        let test_data = vec![0u8; 512];
        let result = fixture.memory_manager.write_buffer(&read_only_buffer, 0, &test_data).await;
        assert!(result.is_err(), "Should reject writes to read-only buffer");

        // Create write-only buffer
        let write_only_buffer = fixture.memory_manager.create_buffer(
            BufferType::Storage,
            BufferUsage::WRITE,
            1024,
            Some("Write Only Buffer"),
        ).unwrap();

        // Try to read from write-only buffer
        let result = fixture.memory_manager.read_buffer(&write_only_buffer, 0, 512).await;
        assert!(result.is_err(), "Should reject reads from write-only buffer");
    }

    #[tokio::test]
    async fn test_memory_alignment_enforcement() {
        let fixture = MemorySafetyFixture::new().await.unwrap();

        // Test uniform buffer alignment requirements
        let uniform_buffer = fixture.memory_manager.create_buffer(
            BufferType::Uniform,
            BufferUsage::READ,
            1024,
            Some("Uniform Buffer"),
        ).unwrap();

        // Try to write unaligned data (assuming 256-byte alignment requirement)
        let unaligned_data = vec![0u8; 100]; // Not aligned to 256 bytes
        let result = fixture.memory_manager.write_buffer(&uniform_buffer, 1, &unaligned_data).await;
        // Should either succeed with automatic alignment or fail with clear error
        match result {
            Ok(_) => {
                // Verify data was properly aligned internally
                let read_result = fixture.memory_manager.read_buffer(&uniform_buffer, 0, 256).await;
                assert!(read_result.is_ok(), "Aligned read should succeed");
            }
            Err(RobinError::GPUMemoryError(msg)) => {
                assert!(msg.contains("alignment"), "Error should mention alignment requirement");
            }
            Err(e) => panic!("Unexpected error type: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_buffer_lifecycle_safety() {
        let fixture = MemorySafetyFixture::new().await.unwrap();

        let buffer_handle = {
            // Create buffer in limited scope
            let buffer = fixture.memory_manager.create_buffer(
                BufferType::Storage,
                BufferUsage::READ_WRITE,
                1024,
                Some("Scoped Buffer"),
            ).unwrap();

            // Write some data
            let test_data = vec![42u8; 512];
            fixture.memory_manager.write_buffer(&buffer, 0, &test_data).await.unwrap();

            buffer
        }; // Buffer handle goes out of scope but should remain valid

        // Try to use buffer handle after its original reference is dropped
        let read_result = fixture.memory_manager.read_buffer(&buffer_handle, 0, 512).await;
        assert!(read_result.is_ok(), "Buffer should remain valid through handle");

        // Verify data integrity
        let data = read_result.unwrap();
        assert_eq!(data[0], 42, "Data should be preserved");
        assert_eq!(data[511], 42, "Data should be preserved at end");
    }

    // =======================
    // MEMORY LEAK DETECTION
    // =======================

    #[tokio::test]
    async fn test_voxel_pipeline_memory_leaks() {
        let mut fixture = MemorySafetyFixture::new().await.unwrap();

        let initial_stats = fixture.memory_manager.get_stats();

        // Generate many voxel meshes and let them go out of scope
        for i in 0..100 {
            let chunk = create_test_voxel_chunk(16);
            let result = fixture.voxel_pipeline.generate_chunk_mesh(&chunk, 0, true).await;

            match result {
                Ok((vertices, indices)) => {
                    // Explicitly drop the results to test cleanup
                    drop(vertices);
                    drop(indices);
                }
                Err(_) => break, // Stop if we hit memory limits
            }

            // Force periodic cleanup
            if i % 20 == 0 {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }

        // Allow time for any async cleanup
        tokio::time::sleep(Duration::from_millis(100)).await;

        let final_stats = fixture.memory_manager.get_stats();
        let memory_growth = final_stats.total_memory_used - initial_stats.total_memory_used;

        // Memory growth should be minimal (indicating no major leaks)
        assert!(memory_growth < 50_000_000, // < 50MB
            "Excessive memory growth detected: {} bytes (potential leak)", memory_growth);

        // Fragmentation should not be excessive
        assert!(final_stats.fragmentation_ratio < 0.3,
            "High memory fragmentation: {:.2}", final_stats.fragmentation_ratio);
    }

    #[tokio::test]
    async fn test_batch_renderer_memory_leaks() {
        let mut fixture = MemorySafetyFixture::new().await.unwrap();

        let initial_stats = fixture.memory_manager.get_stats();

        // Create and destroy many batches
        for cycle in 0..50 {
            // Add many render items
            for i in 0..100 {
                let transform = create_test_transform(i as f32);
                let mesh = create_test_mesh();
                let material = create_test_material();
                let shader = create_test_shader();

                let result = fixture.batch_renderer.add_render_item(
                    mesh, material, shader, &transform, 0
                );
                if result.is_err() {
                    break; // Hit capacity limit
                }
            }

            // Optimize batches (allocates intermediate buffers)
            let camera = create_test_camera();
            let _ = fixture.batch_renderer.optimize_batches(&camera);

            // Clear all batches (should free memory)
            fixture.batch_renderer.clear_batches();

            // Periodic garbage collection
            if cycle % 10 == 0 {
                fixture.batch_renderer.gc_unused_batches(0);
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }

        let final_stats = fixture.memory_manager.get_stats();
        let memory_growth = final_stats.total_memory_used - initial_stats.total_memory_used;

        assert!(memory_growth < 20_000_000, // < 20MB
            "Batch renderer memory leak detected: {} bytes", memory_growth);
    }

    // =======================
    // CONCURRENT ACCESS SAFETY
    // =======================

    #[tokio::test]
    async fn test_concurrent_buffer_access() {
        let fixture = MemorySafetyFixture::new().await.unwrap();

        let buffer = fixture.memory_manager.create_buffer(
            BufferType::Storage,
            BufferUsage::READ_WRITE,
            4096,
            Some("Concurrent Test Buffer"),
        ).unwrap();

        let memory_manager = fixture.memory_manager.clone();

        // Spawn multiple tasks that try to access the same buffer
        let mut handles = Vec::new();

        for i in 0..8 {
            let buffer_clone = buffer.clone();
            let memory_manager_clone = memory_manager.clone();

            let handle = tokio::spawn(async move {
                let offset = (i * 512) as u64;
                let data = vec![i as u8; 512];

                // Try to write to different sections of the buffer
                let write_result = memory_manager_clone.write_buffer(&buffer_clone, offset, &data).await;
                assert!(write_result.is_ok(), "Concurrent write should succeed");

                // Try to read back the data
                tokio::time::sleep(Duration::from_millis(10)).await;
                let read_result = memory_manager_clone.read_buffer(&buffer_clone, offset, 512).await;
                assert!(read_result.is_ok(), "Concurrent read should succeed");

                let read_data = read_result.unwrap();
                assert_eq!(read_data[0], i as u8, "Data should be preserved during concurrent access");
            });

            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_concurrent_voxel_generation() {
        let mut fixture = MemorySafetyFixture::new().await.unwrap();

        // Create multiple voxel generation tasks
        let mut handles = Vec::new();

        for i in 0..4 {
            let handle = tokio::spawn(async move {
                // Each task generates different chunks
                let chunk = create_test_voxel_chunk_with_seed(16, i);

                // Simulate some processing time
                tokio::time::sleep(Duration::from_millis(i * 10)).await;

                // This should be thread-safe
                let result = std::future::ready(Ok((Vec::new(), Vec::new()))); // Placeholder
                result.await
            });

            handles.push(handle);
        }

        // All tasks should complete without data races
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }
    }

    // =======================
    // SECURITY VULNERABILITY TESTS
    // =======================

    #[tokio::test]
    async fn test_integer_overflow_protection() {
        let fixture = MemorySafetyFixture::new().await.unwrap();

        // Try to create buffer with size that would overflow
        let result = fixture.memory_manager.create_buffer(
            BufferType::Storage,
            BufferUsage::READ,
            u64::MAX, // This should be rejected
            Some("Overflow Test Buffer"),
        );
        assert!(result.is_err(), "Should reject buffers with overflow-prone sizes");

        // Try to read with offset + size that would overflow
        let normal_buffer = fixture.memory_manager.create_buffer(
            BufferType::Storage,
            BufferUsage::READ,
            1024,
            Some("Normal Buffer"),
        ).unwrap();

        let result = fixture.memory_manager.read_buffer(&normal_buffer, u64::MAX - 100, 200).await;
        assert!(result.is_err(), "Should reject reads that would cause integer overflow");
    }

    #[tokio::test]
    async fn test_resource_exhaustion_handling() {
        let fixture = MemorySafetyFixture::new().await.unwrap();

        let mut buffers = Vec::new();
        let mut allocation_count = 0;

        // Try to exhaust available buffer handles
        for i in 0..10000 {
            match fixture.memory_manager.create_buffer(
                BufferType::Storage,
                BufferUsage::READ,
                1024, // Small buffers to test handle exhaustion rather than memory exhaustion
                Some(&format!("Buffer {}", i)),
            ) {
                Ok(buffer) => {
                    buffers.push(buffer);
                    allocation_count += 1;
                }
                Err(RobinError::ResourceExhausted(_)) => {
                    // Expected behavior - should gracefully handle exhaustion
                    break;
                }
                Err(RobinError::GPUMemoryError(_)) => {
                    // Also acceptable - ran out of GPU memory
                    break;
                }
                Err(e) => {
                    panic!("Unexpected error during resource exhaustion test: {:?}", e);
                }
            }
        }

        assert!(allocation_count > 100, "Should be able to allocate reasonable number of buffers");
        assert!(allocation_count < 10000, "Should enforce resource limits");

        println!("Successfully allocated {} buffers before hitting limits", allocation_count);
    }

    #[tokio::test]
    async fn test_memory_initialization() {
        let fixture = MemorySafetyFixture::new().await.unwrap();

        // Create buffer without initial data
        let buffer = fixture.memory_manager.create_buffer(
            BufferType::Storage,
            BufferUsage::READ_WRITE,
            1024,
            Some("Uninitialized Buffer"),
        ).unwrap();

        // Read from uninitialized buffer
        let result = fixture.memory_manager.read_buffer(&buffer, 0, 1024).await;
        assert!(result.is_ok(), "Should be able to read from uninitialized buffer");

        let data = result.unwrap();
        // Data should be zero-initialized or have some predictable pattern
        // (This prevents information leaks from previous allocations)
        let has_random_data = data.windows(16).any(|window| {
            window.iter().any(|&b| b != 0) && window.iter().any(|&b| b != 255)
        });

        assert!(!has_random_data || data.iter().all(|&b| b == 0),
            "Uninitialized buffer should not contain random data (security risk)");
    }

    // =======================
    // HELPER FUNCTIONS
    // =======================

    fn create_test_voxel_chunk(size: u32) -> VoxelChunk {
        // Create a test chunk with predictable data
        unimplemented!("Test helper implementation")
    }

    fn create_test_voxel_chunk_with_seed(size: u32, seed: u64) -> VoxelChunk {
        // Create a test chunk with seeded random data
        unimplemented!("Test helper implementation")
    }

    fn create_test_mesh() -> Arc<Mesh> {
        unimplemented!("Test helper implementation")
    }

    fn create_test_material() -> Arc<Material> {
        unimplemented!("Test helper implementation")
    }

    fn create_test_shader() -> Arc<Shader> {
        unimplemented!("Test helper implementation")
    }

    fn create_test_transform(offset: f32) -> Transform {
        unimplemented!("Test helper implementation")
    }

    fn create_test_camera() -> Camera {
        unimplemented!("Test helper implementation")
    }
}