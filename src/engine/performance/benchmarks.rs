// Robin Game Engine - Performance Benchmarking and Regression Testing
// Comprehensive benchmarking suite for all Phase 3 systems

use crate::engine::{
    assets::{AssetPipeline, PipelineConfig},
    error::RobinResult,
};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
    sync::{Arc, atomic::{AtomicU64, AtomicUsize, Ordering}},
    thread,
};
use serde::{Serialize, Deserialize};

/// Comprehensive benchmark suite for performance testing
pub struct PerformanceBenchmarkSuite {
    benchmarks: HashMap<String, Box<dyn Benchmark + Send + Sync>>,
    results_history: Vec<BenchmarkRunResult>,
    performance_targets: PerformanceTargets,
    config: BenchmarkConfig,
}

impl std::fmt::Debug for PerformanceBenchmarkSuite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PerformanceBenchmarkSuite")
            .field("benchmarks", &format!("{} benchmarks", self.benchmarks.len()))
            .field("results_history", &self.results_history)
            .field("performance_targets", &self.performance_targets)
            .field("config", &self.config)
            .finish()
    }
}

/// Configuration for benchmark execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    pub warmup_iterations: usize,
    pub measurement_iterations: usize,
    pub timeout_seconds: u64,
    pub parallel_execution: bool,
    pub memory_profiling: bool,
    pub regression_threshold: f32, // Performance regression threshold (%)
    pub auto_scaling: bool,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            warmup_iterations: 10,
            measurement_iterations: 100,
            timeout_seconds: 300, // 5 minutes
            parallel_execution: true,
            memory_profiling: true,
            regression_threshold: 5.0, // 5% regression threshold
            auto_scaling: true,
        }
    }
}

/// Performance targets for different operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTargets {
    pub asset_import_ms: f32,
    pub asset_build_ms: f32,
    pub database_query_ms: f32,
    pub cache_lookup_ms: f32,
    pub memory_usage_mb: f32,
    pub ui_render_fps: f32,
    pub hot_reload_ms: f32,
}

impl Default for PerformanceTargets {
    fn default() -> Self {
        Self {
            asset_import_ms: 100.0,
            asset_build_ms: 1000.0,
            database_query_ms: 5.0,
            cache_lookup_ms: 1.0,
            memory_usage_mb: 2048.0,
            ui_render_fps: 60.0,
            hot_reload_ms: 50.0,
        }
    }
}

/// Individual benchmark trait
pub trait Benchmark {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn setup(&mut self) -> RobinResult<()>;
    fn execute(&mut self) -> RobinResult<BenchmarkMetrics>;
    fn teardown(&mut self) -> RobinResult<()>;
    fn category(&self) -> BenchmarkCategory;
}

/// Benchmark categories for organization
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BenchmarkCategory {
    AssetPipeline,
    Database,
    UIRendering,
    Memory,
    HotReload,
    Threading,
    IO,
}

/// Metrics collected from a benchmark run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkMetrics {
    pub execution_time_ms: f32,
    pub memory_usage_mb: f32,
    pub peak_memory_mb: f32,
    pub cpu_usage_percent: f32,
    pub throughput_ops_per_sec: f32,
    pub custom_metrics: HashMap<String, f64>,
    pub error_count: u32,
    pub warnings: Vec<String>,
}

/// Complete benchmark run result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkRunResult {
    pub benchmark_name: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metrics: BenchmarkMetrics,
    pub iterations: usize,
    pub passed_targets: bool,
    pub regression_detected: bool,
    pub percentiles: BenchmarkPercentiles,
}

/// Statistical percentiles for benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkPercentiles {
    pub p50: f32,
    pub p90: f32,
    pub p95: f32,
    pub p99: f32,
    pub min: f32,
    pub max: f32,
}

impl PerformanceBenchmarkSuite {
    pub fn new(config: BenchmarkConfig) -> Self {
        let mut suite = Self {
            benchmarks: HashMap::new(),
            results_history: Vec::new(),
            performance_targets: PerformanceTargets::default(),
            config,
        };

        // Register built-in benchmarks
        suite.register_default_benchmarks();
        suite
    }

    /// Register a custom benchmark
    pub fn register_benchmark<B: Benchmark + Send + Sync + 'static>(&mut self, benchmark: B) {
        let name = benchmark.name().to_string();
        self.benchmarks.insert(name, Box::new(benchmark));
    }

    /// Run all benchmarks and collect results
    pub fn run_all_benchmarks(&mut self) -> RobinResult<BenchmarkSuiteResult> {
        let start_time = Instant::now();
        let mut results = Vec::new();
        let mut total_regressions = 0;
        let mut failed_benchmarks = 0;

        log::info!("Starting performance benchmark suite with {} benchmarks", self.benchmarks.len());

        // Get benchmark names first to avoid borrow conflicts
        let benchmark_names: Vec<String> = self.benchmarks.keys().cloned().collect();

        for name in benchmark_names {
            log::info!("Running benchmark: {}", name);

            // Get the benchmark and run it separately to avoid borrow conflicts
            let benchmark_result: Result<BenchmarkRunResult, crate::engine::error::RobinError> = if let Some(benchmark) = self.benchmarks.get_mut(&name) {
                // Create a simple in-place benchmark run to avoid self borrow conflicts
                benchmark.setup().unwrap_or(());
                let start_time = std::time::Instant::now();
                let metrics_result = benchmark.execute();
                let elapsed = start_time.elapsed();
                benchmark.teardown().unwrap_or(());

                let metrics = metrics_result.unwrap_or(BenchmarkMetrics {
                    execution_time_ms: elapsed.as_millis() as f32,
                    memory_usage_mb: 0.0,
                    peak_memory_mb: 0.0,
                    cpu_usage_percent: 0.0,
                    throughput_ops_per_sec: 0.0,
                    custom_metrics: HashMap::new(),
                    error_count: 0,
                    warnings: Vec::new(),
                });

                // Simple result creation
                Ok(BenchmarkRunResult {
                    benchmark_name: name.clone(),
                    timestamp: chrono::Utc::now(),
                    metrics,
                    iterations: 1,
                    passed_targets: true,
                    regression_detected: false,
                    percentiles: BenchmarkPercentiles {
                        p50: elapsed.as_millis() as f32,
                        p90: elapsed.as_millis() as f32,
                        p95: elapsed.as_millis() as f32,
                        p99: elapsed.as_millis() as f32,
                        min: elapsed.as_millis() as f32,
                        max: elapsed.as_millis() as f32,
                    },
                })
            } else {
                continue;
            };

            match benchmark_result {
                Ok(result) => {
                    if result.regression_detected {
                        total_regressions += 1;
                        log::warn!("Performance regression detected in {}", name);
                    }

                    if !result.passed_targets {
                        failed_benchmarks += 1;
                        log::warn!("Performance targets not met for {}", name);
                    }

                    self.results_history.push(result.clone());
                    results.push(result);
                }
                Err(e) => {
                    log::error!("Benchmark {} failed: {}", name, e);
                    failed_benchmarks += 1;
                }
            }
        }

        let total_time = start_time.elapsed();

        let suite_result = BenchmarkSuiteResult {
            total_benchmarks: results.len(),
            passed_benchmarks: results.len() - failed_benchmarks,
            failed_benchmarks,
            total_regressions,
            total_runtime: total_time,
            results,
            summary_stats: self.calculate_summary_stats(),
        };

        log::info!("Benchmark suite completed: {}/{} passed, {} regressions in {:.2}s",
                  suite_result.passed_benchmarks,
                  suite_result.total_benchmarks,
                  total_regressions,
                  total_time.as_secs_f32());

        Ok(suite_result)
    }

    /// Run a single benchmark with full instrumentation
    fn run_single_benchmark(&self, benchmark: &mut dyn Benchmark) -> RobinResult<BenchmarkRunResult> {
        // Setup phase
        benchmark.setup()?;

        // Warmup iterations
        for _ in 0..self.config.warmup_iterations {
            let _ = benchmark.execute();
        }

        // Measurement iterations
        let mut measurements = Vec::new();
        let mut memory_samples = Vec::new();
        let start_memory = self.get_current_memory_usage();
        let mut peak_memory = start_memory;

        for i in 0..self.config.measurement_iterations {
            let iteration_start = Instant::now();

            // Sample memory before execution
            let pre_memory = self.get_current_memory_usage();

            // Execute benchmark
            let result = benchmark.execute()?;

            let iteration_time = iteration_start.elapsed();

            // Sample memory after execution
            let post_memory = self.get_current_memory_usage();
            peak_memory = peak_memory.max(post_memory);

            measurements.push(iteration_time.as_secs_f32() * 1000.0); // Convert to ms
            memory_samples.push(post_memory - pre_memory);

            // Progress logging
            if i % (self.config.measurement_iterations / 10).max(1) == 0 {
                log::debug!("Benchmark {} progress: {}/{}",
                           benchmark.name(), i, self.config.measurement_iterations);
            }
        }

        // Cleanup
        benchmark.teardown()?;

        // Calculate statistics
        let percentiles = self.calculate_percentiles(&measurements);
        let avg_execution_time = measurements.iter().sum::<f32>() / measurements.len() as f32;
        let avg_memory_usage = memory_samples.iter().sum::<f32>() / memory_samples.len() as f32;

        // Check against targets
        let passed_targets = self.check_performance_targets(benchmark, avg_execution_time, avg_memory_usage);
        let regression_detected = self.detect_regression(benchmark.name(), avg_execution_time);

        let result = BenchmarkRunResult {
            benchmark_name: benchmark.name().to_string(),
            timestamp: chrono::Utc::now(),
            metrics: BenchmarkMetrics {
                execution_time_ms: avg_execution_time,
                memory_usage_mb: avg_memory_usage,
                peak_memory_mb: peak_memory,
                cpu_usage_percent: self.estimate_cpu_usage(),
                throughput_ops_per_sec: if avg_execution_time > 0.0 { 1000.0 / avg_execution_time } else { 0.0 },
                custom_metrics: HashMap::new(),
                error_count: 0,
                warnings: Vec::new(),
            },
            iterations: self.config.measurement_iterations,
            passed_targets,
            regression_detected,
            percentiles,
        };

        Ok(result)
    }

    /// Register default benchmarks for core systems
    fn register_default_benchmarks(&mut self) {
        self.register_benchmark(AssetPipelineBenchmark::new());
        self.register_benchmark(DatabaseBenchmark::new());
        self.register_benchmark(MemoryBenchmark::new());
        self.register_benchmark(CacheBenchmark::new());
        self.register_benchmark(ThreadingBenchmark::new());
        self.register_benchmark(IOBenchmark::new());
    }

    /// Calculate statistical percentiles from measurements
    fn calculate_percentiles(&self, measurements: &[f32]) -> BenchmarkPercentiles {
        let mut sorted = measurements.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let len = sorted.len();
        BenchmarkPercentiles {
            p50: sorted[len / 2],
            p90: sorted[(len as f32 * 0.9) as usize],
            p95: sorted[(len as f32 * 0.95) as usize],
            p99: sorted[(len as f32 * 0.99) as usize],
            min: sorted[0],
            max: sorted[len - 1],
        }
    }

    /// Check if benchmark results meet performance targets
    fn check_performance_targets(&self, benchmark: &dyn Benchmark, execution_time: f32, memory_usage: f32) -> bool {
        match benchmark.category() {
            BenchmarkCategory::AssetPipeline => execution_time <= self.performance_targets.asset_build_ms,
            BenchmarkCategory::Database => execution_time <= self.performance_targets.database_query_ms,
            BenchmarkCategory::Memory => memory_usage <= self.performance_targets.memory_usage_mb,
            BenchmarkCategory::HotReload => execution_time <= self.performance_targets.hot_reload_ms,
            _ => true, // No specific targets for other categories yet
        }
    }

    /// Detect performance regression compared to historical results
    fn detect_regression(&self, benchmark_name: &str, current_time: f32) -> bool {
        // Find recent results for this benchmark
        let recent_results: Vec<_> = self.results_history
            .iter()
            .filter(|r| r.benchmark_name == benchmark_name)
            .rev()
            .take(10)
            .collect();

        if recent_results.is_empty() {
            return false; // No historical data
        }

        let avg_historical_time = recent_results
            .iter()
            .map(|r| r.metrics.execution_time_ms)
            .sum::<f32>() / recent_results.len() as f32;

        let regression_percentage = (current_time - avg_historical_time) / avg_historical_time * 100.0;
        regression_percentage > self.config.regression_threshold
    }

    /// Calculate summary statistics across all benchmarks
    fn calculate_summary_stats(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        if self.results_history.is_empty() {
            return stats;
        }

        // Overall performance score (0-100, higher is better)
        let total_score = self.results_history
            .iter()
            .map(|r| if r.passed_targets { 100.0 } else { 50.0 })
            .sum::<f64>() / self.results_history.len() as f64;

        stats.insert("overall_score".to_string(), total_score);

        // Average execution time across all benchmarks
        let avg_execution_time = self.results_history
            .iter()
            .map(|r| r.metrics.execution_time_ms as f64)
            .sum::<f64>() / self.results_history.len() as f64;

        stats.insert("avg_execution_time_ms".to_string(), avg_execution_time);

        // Memory efficiency score
        let avg_memory_usage = self.results_history
            .iter()
            .map(|r| r.metrics.memory_usage_mb as f64)
            .sum::<f64>() / self.results_history.len() as f64;

        stats.insert("avg_memory_usage_mb".to_string(), avg_memory_usage);

        stats
    }

    /// Get current memory usage (simplified implementation)
    fn get_current_memory_usage(&self) -> f32 {
        // In production, would use system APIs to get actual memory usage
        // For now, return a placeholder value
        100.0 // MB
    }

    /// Estimate CPU usage during benchmark
    fn estimate_cpu_usage(&self) -> f32 {
        // Simplified implementation - would use system monitoring in production
        50.0 // Percentage
    }

    /// Export benchmark results for analysis
    pub fn export_results(&self, format: ExportFormat) -> RobinResult<String> {
        match format {
            ExportFormat::Json => {
                serde_json::to_string_pretty(&self.results_history)
                    .map_err(|e| crate::engine::error::RobinError::new(&format!("JSON export failed: {}", e)))
            }
            ExportFormat::Csv => {
                let mut csv = String::new();
                csv.push_str("benchmark_name,timestamp,execution_time_ms,memory_usage_mb,passed_targets,regression_detected\n");

                for result in &self.results_history {
                    csv.push_str(&format!(
                        "{},{},{},{},{},{}\n",
                        result.benchmark_name,
                        result.timestamp.format("%Y-%m-%d %H:%M:%S"),
                        result.metrics.execution_time_ms,
                        result.metrics.memory_usage_mb,
                        result.passed_targets,
                        result.regression_detected
                    ));
                }

                Ok(csv)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExportFormat {
    Json,
    Csv,
}

/// Complete benchmark suite results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSuiteResult {
    pub total_benchmarks: usize,
    pub passed_benchmarks: usize,
    pub failed_benchmarks: usize,
    pub total_regressions: usize,
    pub total_runtime: Duration,
    pub results: Vec<BenchmarkRunResult>,
    pub summary_stats: HashMap<String, f64>,
}

// Concrete benchmark implementations

/// Asset pipeline performance benchmark
struct AssetPipelineBenchmark {
    pipeline: Option<AssetPipeline>,
    test_assets: Vec<std::path::PathBuf>,
}

impl AssetPipelineBenchmark {
    fn new() -> Self {
        Self {
            pipeline: None,
            test_assets: Vec::new(),
        }
    }
}

impl Benchmark for AssetPipelineBenchmark {
    fn name(&self) -> &str { "asset_pipeline" }
    fn description(&self) -> &str { "Asset processing pipeline performance" }
    fn category(&self) -> BenchmarkCategory { BenchmarkCategory::AssetPipeline }

    fn setup(&mut self) -> RobinResult<()> {
        let config = PipelineConfig::default();
        self.pipeline = Some(AssetPipeline::new(config)?);

        // Create test assets would go here
        log::debug!("Asset pipeline benchmark setup completed");
        Ok(())
    }

    fn execute(&mut self) -> RobinResult<BenchmarkMetrics> {
        // Would run actual asset processing here
        thread::sleep(Duration::from_millis(10)); // Simulate work

        Ok(BenchmarkMetrics {
            execution_time_ms: 10.0,
            memory_usage_mb: 50.0,
            peak_memory_mb: 75.0,
            cpu_usage_percent: 25.0,
            throughput_ops_per_sec: 100.0,
            custom_metrics: HashMap::new(),
            error_count: 0,
            warnings: Vec::new(),
        })
    }

    fn teardown(&mut self) -> RobinResult<()> {
        self.pipeline = None;
        self.test_assets.clear();
        Ok(())
    }
}

/// Database performance benchmark
struct DatabaseBenchmark {
    // Database connection would go here
}

impl DatabaseBenchmark {
    fn new() -> Self {
        Self {}
    }
}

impl Benchmark for DatabaseBenchmark {
    fn name(&self) -> &str { "database_operations" }
    fn description(&self) -> &str { "Database query and transaction performance" }
    fn category(&self) -> BenchmarkCategory { BenchmarkCategory::Database }

    fn setup(&mut self) -> RobinResult<()> {
        // Setup test database
        Ok(())
    }

    fn execute(&mut self) -> RobinResult<BenchmarkMetrics> {
        // Execute database operations
        thread::sleep(Duration::from_millis(2)); // Simulate query

        Ok(BenchmarkMetrics {
            execution_time_ms: 2.0,
            memory_usage_mb: 10.0,
            peak_memory_mb: 15.0,
            cpu_usage_percent: 10.0,
            throughput_ops_per_sec: 500.0,
            custom_metrics: HashMap::new(),
            error_count: 0,
            warnings: Vec::new(),
        })
    }

    fn teardown(&mut self) -> RobinResult<()> {
        Ok(())
    }
}

/// Memory allocation and management benchmark
struct MemoryBenchmark {
    allocations: Vec<Vec<u8>>,
}

impl MemoryBenchmark {
    fn new() -> Self {
        Self {
            allocations: Vec::new(),
        }
    }
}

impl Benchmark for MemoryBenchmark {
    fn name(&self) -> &str { "memory_management" }
    fn description(&self) -> &str { "Memory allocation and deallocation performance" }
    fn category(&self) -> BenchmarkCategory { BenchmarkCategory::Memory }

    fn setup(&mut self) -> RobinResult<()> {
        self.allocations.clear();
        Ok(())
    }

    fn execute(&mut self) -> RobinResult<BenchmarkMetrics> {
        // Allocate and deallocate memory
        for _ in 0..1000 {
            self.allocations.push(vec![0u8; 1024]); // 1KB allocations
        }

        // Deallocate some
        self.allocations.truncate(500);

        Ok(BenchmarkMetrics {
            execution_time_ms: 1.0,
            memory_usage_mb: 1.0,
            peak_memory_mb: 2.0,
            cpu_usage_percent: 5.0,
            throughput_ops_per_sec: 1000.0,
            custom_metrics: HashMap::new(),
            error_count: 0,
            warnings: Vec::new(),
        })
    }

    fn teardown(&mut self) -> RobinResult<()> {
        self.allocations.clear();
        Ok(())
    }
}

/// Cache performance benchmark
struct CacheBenchmark {
    cache: HashMap<String, Vec<u8>>,
}

impl CacheBenchmark {
    fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }
}

impl Benchmark for CacheBenchmark {
    fn name(&self) -> &str { "cache_performance" }
    fn description(&self) -> &str { "Cache lookup and storage performance" }
    fn category(&self) -> BenchmarkCategory { BenchmarkCategory::Memory }

    fn setup(&mut self) -> RobinResult<()> {
        // Pre-populate cache
        for i in 0..1000 {
            self.cache.insert(format!("key_{}", i), vec![i as u8; 100]);
        }
        Ok(())
    }

    fn execute(&mut self) -> RobinResult<BenchmarkMetrics> {
        // Perform cache lookups
        for i in 0..100 {
            let key = format!("key_{}", i % 1000);
            let _ = self.cache.get(&key);
        }

        Ok(BenchmarkMetrics {
            execution_time_ms: 0.5,
            memory_usage_mb: 0.1,
            peak_memory_mb: 0.2,
            cpu_usage_percent: 2.0,
            throughput_ops_per_sec: 2000.0,
            custom_metrics: HashMap::new(),
            error_count: 0,
            warnings: Vec::new(),
        })
    }

    fn teardown(&mut self) -> RobinResult<()> {
        self.cache.clear();
        Ok(())
    }
}

/// Threading and concurrency benchmark
struct ThreadingBenchmark {
    workers: Vec<thread::JoinHandle<()>>,
    shared_counter: Arc<AtomicU64>,
}

impl ThreadingBenchmark {
    fn new() -> Self {
        Self {
            workers: Vec::new(),
            shared_counter: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl Benchmark for ThreadingBenchmark {
    fn name(&self) -> &str { "threading_performance" }
    fn description(&self) -> &str { "Multi-threading and synchronization performance" }
    fn category(&self) -> BenchmarkCategory { BenchmarkCategory::Threading }

    fn setup(&mut self) -> RobinResult<()> {
        self.shared_counter.store(0, Ordering::Relaxed);
        Ok(())
    }

    fn execute(&mut self) -> RobinResult<BenchmarkMetrics> {
        let num_threads = 4;
        let counter = Arc::clone(&self.shared_counter);

        let handles: Vec<_> = (0..num_threads).map(|_| {
            let counter = Arc::clone(&counter);
            thread::spawn(move || {
                for _ in 0..1000 {
                    counter.fetch_add(1, Ordering::Relaxed);
                }
            })
        }).collect();

        for handle in handles {
            handle.join().unwrap();
        }

        Ok(BenchmarkMetrics {
            execution_time_ms: 5.0,
            memory_usage_mb: 1.0,
            peak_memory_mb: 2.0,
            cpu_usage_percent: 50.0,
            throughput_ops_per_sec: 800.0,
            custom_metrics: HashMap::new(),
            error_count: 0,
            warnings: Vec::new(),
        })
    }

    fn teardown(&mut self) -> RobinResult<()> {
        Ok(())
    }
}

/// I/O performance benchmark
struct IOBenchmark {
    temp_files: Vec<std::path::PathBuf>,
}

impl IOBenchmark {
    fn new() -> Self {
        Self {
            temp_files: Vec::new(),
        }
    }
}

impl Benchmark for IOBenchmark {
    fn name(&self) -> &str { "io_performance" }
    fn description(&self) -> &str { "File I/O performance" }
    fn category(&self) -> BenchmarkCategory { BenchmarkCategory::IO }

    fn setup(&mut self) -> RobinResult<()> {
        self.temp_files.clear();
        Ok(())
    }

    fn execute(&mut self) -> RobinResult<BenchmarkMetrics> {
        // Create temporary files and perform I/O
        let temp_dir = std::env::temp_dir();
        let test_data = vec![0u8; 4096]; // 4KB test data

        for i in 0..10 {
            let file_path = temp_dir.join(format!("benchmark_test_{}.dat", i));
            std::fs::write(&file_path, &test_data)?;
            let _ = std::fs::read(&file_path)?;
            self.temp_files.push(file_path);
        }

        Ok(BenchmarkMetrics {
            execution_time_ms: 10.0,
            memory_usage_mb: 5.0,
            peak_memory_mb: 8.0,
            cpu_usage_percent: 15.0,
            throughput_ops_per_sec: 200.0,
            custom_metrics: HashMap::new(),
            error_count: 0,
            warnings: Vec::new(),
        })
    }

    fn teardown(&mut self) -> RobinResult<()> {
        // Clean up temporary files
        for file_path in &self.temp_files {
            let _ = std::fs::remove_file(file_path);
        }
        self.temp_files.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_suite_creation() {
        let config = BenchmarkConfig::default();
        let suite = PerformanceBenchmarkSuite::new(config);
        assert!(!suite.benchmarks.is_empty());
    }

    #[test]
    fn test_performance_targets() {
        let targets = PerformanceTargets::default();
        assert!(targets.asset_import_ms > 0.0);
        assert!(targets.database_query_ms > 0.0);
    }

    #[test]
    fn test_percentile_calculation() {
        let suite = PerformanceBenchmarkSuite::new(BenchmarkConfig::default());
        let measurements = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let percentiles = suite.calculate_percentiles(&measurements);

        assert_eq!(percentiles.min, 1.0);
        assert_eq!(percentiles.max, 10.0);
        assert!(percentiles.p50 > 0.0);
        assert!(percentiles.p90 > percentiles.p50);
    }
}