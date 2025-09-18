/// Comprehensive performance benchmarks for Robin Game Engine Phase 3 optimizations
///
/// This benchmark suite tests:
/// - Asset pipeline performance (parallel processing, memory-mapped I/O, streaming)
/// - Database performance (connection pooling, query optimization, caching)
/// - Memory management (LRU cache, memory pools, garbage collection)
/// - Hot reload performance (debouncing, incremental updates, dependency tracking)

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};
use tempfile::TempDir;
use tokio::runtime::Runtime;

// Import the performance modules we want to benchmark
use robin::engine::{
    assets::{
        parallel_processor::*,
        optimized_database::*,
        optimized_hot_reload::*,
    },
    performance::{
        enhanced_memory_management::*,
    },
};

/// Benchmark configuration
const SMALL_ASSET_SIZE: usize = 1024; // 1KB
const MEDIUM_ASSET_SIZE: usize = 1024 * 1024; // 1MB
const LARGE_ASSET_SIZE: usize = 10 * 1024 * 1024; // 10MB
const BATCH_SIZES: &[usize] = &[1, 10, 50, 100, 500];
const CACHE_SIZES: &[usize] = &[100, 1000, 10000];

/// Generate test data of specified size
fn generate_test_data(size: usize) -> Vec<u8> {
    (0..size).map(|i| (i % 256) as u8).collect()
}

/// Create temporary test files
fn create_test_files(temp_dir: &TempDir, count: usize, size: usize) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let data = generate_test_data(size);

    for i in 0..count {
        let file_path = temp_dir.path().join(format!("test_asset_{}.dat", i));
        std::fs::write(&file_path, &data).unwrap();
        files.push(file_path);
    }

    files
}

/// Benchmark parallel asset processing
fn bench_parallel_asset_processing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("parallel_asset_processing");

    for &batch_size in BATCH_SIZES {
        for &asset_size in &[SMALL_ASSET_SIZE, MEDIUM_ASSET_SIZE] {
            let bench_id = BenchmarkId::from_parameter(format!("batch_{}_size_{}", batch_size, asset_size));

            group.throughput(Throughput::Bytes((batch_size * asset_size) as u64));

            group.bench_with_input(bench_id, &(batch_size, asset_size), |b, &(batch_size, asset_size)| {
                let temp_dir = TempDir::new().unwrap();
                let test_files = create_test_files(&temp_dir, batch_size, asset_size);

                b.to_async(&rt).iter(|| async {
                    let config = ParallelProcessorConfig::default();
                    let processor = ParallelAssetProcessor::new(config).unwrap();

                    let start = Instant::now();

                    let results = processor.process_batch_parallel(
                        &test_files,
                        robin::engine::assets::AssetType::Data,
                    ).await.unwrap();

                    black_box(results);
                    start.elapsed()
                });
            });
        }
    }

    group.finish();
}

/// Benchmark memory-mapped file I/O
fn bench_memory_mapped_io(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_mapped_io");

    for &size in &[MEDIUM_ASSET_SIZE, LARGE_ASSET_SIZE] {
        let bench_id = BenchmarkId::from_parameter(format!("size_{}", size));
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(bench_id, &size, |b, &size| {
            let temp_dir = TempDir::new().unwrap();
            let test_files = create_test_files(&temp_dir, 1, size);
            let file_path = &test_files[0];

            b.iter(|| {
                let mapped_file = MappedFile::new(file_path).unwrap();
                let data = mapped_file.data();
                black_box(data.len());
            });
        });
    }

    group.finish();
}

/// Benchmark database operations
fn bench_database_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("database_operations");

    // Benchmark bulk insert operations
    for &batch_size in BATCH_SIZES {
        let bench_id = BenchmarkId::from_parameter(format!("bulk_insert_{}", batch_size));

        group.bench_with_input(bench_id, &batch_size, |b, &batch_size| {
            b.to_async(&rt).iter(|| async {
                let temp_dir = TempDir::new().unwrap();
                let config = OptimizedDatabaseConfig {
                    database_path: temp_dir.path().join("test.db"),
                    ..Default::default()
                };

                let db = OptimizedAssetDatabase::new(config).await.unwrap();

                let assets: Vec<_> = (0..batch_size)
                    .map(|i| {
                        robin::engine::assets::AssetMetadata::new(
                            format!("asset_{}", i),
                            robin::engine::assets::AssetType::Data,
                            PathBuf::from(format!("test_{}.dat", i)),
                        )
                    })
                    .collect();

                let start = Instant::now();
                let _ = db.bulk_insert_assets(&assets).await;
                start.elapsed()
            });
        });
    }

    // Benchmark search operations
    group.bench_function("search_operations", |b| {
        b.to_async(&rt).iter(|| async {
            let temp_dir = TempDir::new().unwrap();
            let config = OptimizedDatabaseConfig {
                database_path: temp_dir.path().join("test.db"),
                ..Default::default()
            };

            let db = OptimizedAssetDatabase::new(config).await.unwrap();

            // Insert test data
            let assets: Vec<_> = (0..1000)
                .map(|i| {
                    robin::engine::assets::AssetMetadata::new(
                        format!("asset_{}", i),
                        if i % 2 == 0 { robin::engine::assets::AssetType::Texture } else { robin::engine::assets::AssetType::Audio },
                        PathBuf::from(format!("test_{}.dat", i)),
                    )
                })
                .collect();

            let _ = db.bulk_insert_assets(&assets).await;

            let start = Instant::now();
            let results = db.search_assets(
                Some(robin::engine::assets::AssetType::Texture),
                Some("test"),
                None,
                Some(100),
            ).await.unwrap();

            black_box(results);
            start.elapsed()
        });
    });

    group.finish();
}

/// Benchmark enhanced memory management
fn bench_memory_management(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_management");

    // Benchmark multi-level cache operations
    for &cache_size in CACHE_SIZES {
        let bench_id = BenchmarkId::from_parameter(format!("cache_size_{}", cache_size));

        group.bench_with_input(bench_id, &cache_size, |b, &cache_size| {
            b.iter(|| {
                let stats = Arc::new(EnhancedMemoryStats::default());
                let config = EnhancedMemoryConfig::default();

                let cache = EnhancedMultiLevelCache::new(
                    vec![(cache_size / 3, 1024), (cache_size / 3, 2048), (cache_size / 3, 4096)],
                    stats,
                    config,
                );

                // Insert data
                let start = Instant::now();
                for i in 0..cache_size {
                    let key = format!("key_{}", i);
                    let value = generate_test_data(1024);
                    cache.insert(
                        key,
                        value,
                        1024,
                        1.0,
                        robin::engine::performance::memory_management::ResourceType::Texture,
                    );
                }

                // Read data
                for i in 0..cache_size / 2 {
                    let key = format!("key_{}", i);
                    black_box(cache.get(&key));
                }

                start.elapsed()
            });
        });
    }

    // Benchmark memory pool allocations
    group.bench_function("memory_pool_allocations", |b| {
        b.iter(|| {
            let stats = Arc::new(EnhancedMemoryStats::default());
            let pool = EnhancedMemoryPool::new(1024, 1000, stats);

            let start = Instant::now();
            let mut allocations = Vec::new();

            // Allocate many chunks
            for _ in 0..500 {
                if let Some(memory) = pool.allocate() {
                    allocations.push(memory);
                }
            }

            // Deallocate by dropping
            drop(allocations);

            start.elapsed()
        });
    });

    // Benchmark bump allocator
    group.bench_function("bump_allocator", |b| {
        b.iter(|| {
            let stats = Arc::new(EnhancedMemoryStats::default());
            let bump_pool = BumpAllocatorPool::new(64 * 1024 * 1024, stats);

            let start = Instant::now();

            bump_pool.with_allocator(|bump| {
                for _ in 0..1000 {
                    let _data: &mut [u8] = bump.alloc_slice_fill_default(1024);
                    black_box(_data);
                }
            });

            start.elapsed()
        });
    });

    group.finish();
}

/// Benchmark hot reload system
fn bench_hot_reload_system(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("hot_reload_system");

    // Benchmark dependency graph operations
    for &asset_count in &[100, 1000, 5000] {
        let bench_id = BenchmarkId::from_parameter(format!("dependency_graph_{}", asset_count));

        group.bench_with_input(bench_id, &asset_count, |b, &asset_count| {
            b.iter(|| {
                let config = OptimizedHotReloadConfig::default();
                let system = OptimizedHotReloadSystem::new(config).unwrap();

                let start = Instant::now();

                // Create dependency relationships
                for i in 0..asset_count {
                    let asset = PathBuf::from(format!("asset_{}.txt", i));
                    if i > 0 {
                        let dependency = PathBuf::from(format!("asset_{}.txt", i - 1));
                        system.add_dependency(&asset, &dependency);
                    }
                }

                // Force some reloads
                for i in 0..asset_count / 10 {
                    let asset = PathBuf::from(format!("asset_{}.txt", i));
                    system.force_reload(&asset);
                }

                start.elapsed()
            });
        });
    }

    // Benchmark content hash caching
    group.bench_function("content_hash_caching", |b| {
        b.iter(|| {
            let temp_dir = TempDir::new().unwrap();
            let test_files = create_test_files(&temp_dir, 100, SMALL_ASSET_SIZE);

            let cache = ContentHashCache::new(1000);

            let start = Instant::now();

            // First pass - cache misses
            for file in &test_files {
                black_box(cache.get_hash(file));
            }

            // Second pass - cache hits
            for file in &test_files {
                black_box(cache.get_hash(file));
            }

            start.elapsed()
        });
    });

    group.finish();
}

/// Benchmark complete asset pipeline end-to-end
fn bench_asset_pipeline_end_to_end(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("asset_pipeline_end_to_end");

    for &asset_count in &[10, 50, 100] {
        let bench_id = BenchmarkId::from_parameter(format!("assets_{}", asset_count));
        group.throughput(Throughput::Elements(asset_count as u64));

        group.bench_with_input(bench_id, &asset_count, |b, &asset_count| {
            b.to_async(&rt).iter(|| async {
                let temp_dir = TempDir::new().unwrap();

                // Setup complete pipeline
                let processor_config = ParallelProcessorConfig::default();
                let processor = ParallelAssetProcessor::new(processor_config).unwrap();

                let db_config = OptimizedDatabaseConfig {
                    database_path: temp_dir.path().join("pipeline.db"),
                    ..Default::default()
                };
                let database = OptimizedAssetDatabase::new(db_config).await.unwrap();

                let memory_config = EnhancedMemoryConfig::default();
                let memory_manager = EnhancedMemoryManager::new(memory_config);

                // Create test assets
                let test_files = create_test_files(&temp_dir, asset_count, MEDIUM_ASSET_SIZE);

                let start = Instant::now();

                // Process assets through complete pipeline
                let processing_results = processor.process_batch_parallel(
                    &test_files,
                    robin::engine::assets::AssetType::Data,
                ).await.unwrap();

                // Store metadata in database
                let metadata: Vec<_> = test_files.iter().enumerate().map(|(i, path)| {
                    robin::engine::assets::AssetMetadata::new(
                        format!("pipeline_asset_{}", i),
                        robin::engine::assets::AssetType::Data,
                        path.clone(),
                    )
                }).collect();

                let _ = database.bulk_insert_assets(&metadata).await;

                // Cache processed data in memory manager
                for (i, result) in processing_results.iter().enumerate() {
                    if let Some(ref data) = result.data {
                        memory_manager.cache_asset(
                            format!("pipeline_asset_{}", i),
                            data.clone(),
                            1.0,
                            robin::engine::performance::memory_management::ResourceType::Data,
                        );
                    }
                }

                // Search and retrieve
                let search_results = database.search_assets(
                    Some(robin::engine::assets::AssetType::Data),
                    Some("pipeline"),
                    None,
                    None,
                ).await.unwrap();

                black_box(search_results);
                start.elapsed()
            });
        });
    }

    group.finish();
}

/// Benchmark memory pressure and garbage collection
fn bench_memory_pressure_and_gc(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_pressure_and_gc");

    group.bench_function("garbage_collection_cycle", |b| {
        b.iter(|| {
            let config = EnhancedMemoryConfig {
                max_cache_size: 1024 * 1024, // 1MB limit for faster GC
                ..Default::default()
            };
            let memory_manager = EnhancedMemoryManager::new(config);

            let start = Instant::now();

            // Fill memory beyond capacity to trigger GC
            for i in 0..2000 {
                let data = generate_test_data(1024);
                memory_manager.cache_asset(
                    format!("gc_test_{}", i),
                    data,
                    1.0,
                    robin::engine::performance::memory_management::ResourceType::Data,
                );
            }

            // Force garbage collection
            memory_manager.force_gc();

            // Access some cached items
            for i in 1900..2000 {
                black_box(memory_manager.get_asset(&format!("gc_test_{}", i)));
            }

            start.elapsed()
        });
    });

    group.finish();
}

/// Benchmark database connection pooling under load
fn bench_database_connection_pooling(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("database_connection_pooling");

    for &concurrent_connections in &[1, 5, 10, 20] {
        let bench_id = BenchmarkId::from_parameter(format!("connections_{}", concurrent_connections));

        group.bench_with_input(bench_id, &concurrent_connections, |b, &concurrent_connections| {
            b.to_async(&rt).iter(|| async {
                let temp_dir = TempDir::new().unwrap();
                let config = OptimizedDatabaseConfig {
                    database_path: temp_dir.path().join("pool_test.db"),
                    max_connections: concurrent_connections,
                    ..Default::default()
                };

                let database = OptimizedAssetDatabase::new(config).await.unwrap();

                // Insert initial data
                let assets: Vec<_> = (0..100)
                    .map(|i| {
                        robin::engine::assets::AssetMetadata::new(
                            format!("pool_asset_{}", i),
                            robin::engine::assets::AssetType::Data,
                            PathBuf::from(format!("pool_test_{}.dat", i)),
                        )
                    })
                    .collect();

                let _ = database.bulk_insert_assets(&assets).await;

                let start = Instant::now();

                // Simulate concurrent database operations
                let mut handles = Vec::new();

                for _ in 0..concurrent_connections {
                    let db_ref = &database;
                    let handle = tokio::spawn(async move {
                        for _ in 0..10 {
                            let _ = db_ref.search_assets(
                                Some(robin::engine::assets::AssetType::Data),
                                Some("pool"),
                                None,
                                Some(10),
                            ).await;
                        }
                    });
                    handles.push(handle);
                }

                // Wait for all operations to complete
                for handle in handles {
                    let _ = handle.await;
                }

                start.elapsed()
            });
        });
    }

    group.finish();
}

/// Regression test benchmarks for performance monitoring
fn bench_regression_tests(c: &mut Criterion) {
    let mut group = c.benchmark_group("regression_tests");

    // Asset processing regression test
    group.bench_function("asset_processing_baseline", |b| {
        let rt = Runtime::new().unwrap();
        b.to_async(&rt).iter(|| async {
            let temp_dir = TempDir::new().unwrap();
            let test_files = create_test_files(&temp_dir, 50, MEDIUM_ASSET_SIZE);

            let config = ParallelProcessorConfig::default();
            let processor = ParallelAssetProcessor::new(config).unwrap();

            let start = Instant::now();
            let results = processor.process_batch_parallel(
                &test_files,
                robin::engine::assets::AssetType::Data,
            ).await.unwrap();

            black_box(results);
            start.elapsed()
        });
    });

    // Database performance regression test
    group.bench_function("database_performance_baseline", |b| {
        let rt = Runtime::new().unwrap();
        b.to_async(&rt).iter(|| async {
            let temp_dir = TempDir::new().unwrap();
            let config = OptimizedDatabaseConfig {
                database_path: temp_dir.path().join("regression.db"),
                ..Default::default()
            };

            let db = OptimizedAssetDatabase::new(config).await.unwrap();

            let assets: Vec<_> = (0..1000)
                .map(|i| {
                    robin::engine::assets::AssetMetadata::new(
                        format!("regression_asset_{}", i),
                        robin::engine::assets::AssetType::Data,
                        PathBuf::from(format!("regression_{}.dat", i)),
                    )
                })
                .collect();

            let start = Instant::now();
            let _ = db.bulk_insert_assets(&assets).await;

            let _ = db.search_assets(
                None,
                Some("regression"),
                None,
                Some(100),
            ).await;

            start.elapsed()
        });
    });

    // Memory management regression test
    group.bench_function("memory_management_baseline", |b| {
        b.iter(|| {
            let config = EnhancedMemoryConfig::default();
            let memory_manager = EnhancedMemoryManager::new(config);

            let start = Instant::now();

            // Cache 1000 small assets
            for i in 0..1000 {
                let data = generate_test_data(1024);
                memory_manager.cache_asset(
                    format!("regression_mem_{}", i),
                    data,
                    1.0,
                    robin::engine::performance::memory_management::ResourceType::Data,
                );
            }

            // Access cached data
            for i in 0..500 {
                black_box(memory_manager.get_asset(&format!("regression_mem_{}", i)));
            }

            start.elapsed()
        });
    });

    group.finish();
}

// Group all benchmarks
criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(Duration::from_secs(30))
        .warm_up_time(Duration::from_secs(5));
    targets =
        bench_parallel_asset_processing,
        bench_memory_mapped_io,
        bench_database_operations,
        bench_memory_management,
        bench_hot_reload_system,
        bench_asset_pipeline_end_to_end,
        bench_memory_pressure_and_gc,
        bench_database_connection_pooling,
        bench_regression_tests
);

criterion_main!(benches);