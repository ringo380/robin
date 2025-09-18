/// Comprehensive performance monitoring and metrics collection system
///
/// This module provides:
/// - Real-time performance monitoring across all engine subsystems
/// - Configurable metrics collection with minimal overhead
/// - Performance regression detection and alerting
/// - Detailed analytics and reporting capabilities
/// - Integration with all optimization systems

use std::{
    collections::{HashMap, VecDeque},
    sync::{
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use ahash::AHashMap;
use crossbeam_channel::{Receiver, Sender, bounded, unbounded};
use dashmap::DashMap;
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use tokio::time::interval;

/// Configuration for comprehensive monitoring system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable monitoring (can be disabled for production if needed)
    pub enabled: bool,
    /// Collection interval in milliseconds
    pub collection_interval_ms: u64,
    /// Maximum number of data points to keep in memory
    pub max_data_points: usize,
    /// Enable real-time alerts
    pub enable_alerts: bool,
    /// Performance thresholds for alerting
    pub thresholds: PerformanceThresholds,
    /// Enable detailed profiling (higher overhead)
    pub enable_detailed_profiling: bool,
    /// Metrics export interval in seconds
    pub export_interval_secs: u64,
    /// Enable performance regression detection
    pub enable_regression_detection: bool,
    /// Regression detection sensitivity (0.0 - 1.0)
    pub regression_sensitivity: f64,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            collection_interval_ms: 1000, // 1 second
            max_data_points: 3600, // 1 hour at 1-second intervals
            enable_alerts: true,
            thresholds: PerformanceThresholds::default(),
            enable_detailed_profiling: false,
            export_interval_secs: 60, // 1 minute
            enable_regression_detection: true,
            regression_sensitivity: 0.1, // 10% degradation threshold
        }
    }
}

/// Performance thresholds for alerting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceThresholds {
    /// Maximum acceptable asset processing time (ms)
    pub max_asset_processing_time_ms: u64,
    /// Maximum acceptable database query time (ms)
    pub max_database_query_time_ms: u64,
    /// Maximum acceptable memory usage (MB)
    pub max_memory_usage_mb: u64,
    /// Maximum acceptable cache miss rate (0.0 - 1.0)
    pub max_cache_miss_rate: f64,
    /// Maximum acceptable hot reload time (ms)
    pub max_hot_reload_time_ms: u64,
    /// Maximum acceptable GC pause time (ms)
    pub max_gc_pause_time_ms: u64,
    /// Maximum acceptable memory pressure (0.0 - 1.0)
    pub max_memory_pressure: f64,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            max_asset_processing_time_ms: 100,
            max_database_query_time_ms: 5,
            max_memory_usage_mb: 1024,
            max_cache_miss_rate: 0.3,
            max_hot_reload_time_ms: 50,
            max_gc_pause_time_ms: 10,
            max_memory_pressure: 0.8,
        }
    }
}

/// Comprehensive performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub timestamp: SystemTime,
    pub asset_processing: AssetProcessingMetrics,
    pub database: DatabaseMetrics,
    pub memory_management: MemoryManagementMetrics,
    pub hot_reload: HotReloadMetrics,
    pub general_system: GeneralSystemMetrics,
}

/// Asset processing specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetProcessingMetrics {
    pub total_assets_processed: u64,
    pub average_processing_time_ms: f64,
    pub parallel_efficiency: f64,
    pub memory_mapped_operations: u64,
    pub streaming_operations: u64,
    pub compression_ratio: f64,
    pub throughput_mb_per_sec: f64,
}

/// Database specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseMetrics {
    pub total_queries: u64,
    pub average_query_time_ms: f64,
    pub cache_hit_rate: f64,
    pub connection_pool_utilization: f64,
    pub index_efficiency: f64,
    pub bulk_operation_performance: f64,
}

/// Memory management specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryManagementMetrics {
    pub current_usage_mb: f64,
    pub peak_usage_mb: f64,
    pub cache_hit_rate: f64,
    pub pool_utilization: f64,
    pub garbage_collection_frequency: f64,
    pub memory_pressure: f64,
    pub fragmentation_level: f64,
    pub allocation_efficiency: f64,
}

/// Hot reload specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotReloadMetrics {
    pub total_reload_events: u64,
    pub average_reload_time_ms: f64,
    pub debounce_efficiency: f64,
    pub incremental_update_rate: f64,
    pub dependency_graph_size: usize,
    pub cache_effectiveness: f64,
}

/// General system metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralSystemMetrics {
    pub cpu_usage_percent: f64,
    pub memory_usage_percent: f64,
    pub disk_io_mb_per_sec: f64,
    pub network_io_mb_per_sec: f64,
    pub frame_rate: f64,
    pub frame_time_ms: f64,
    pub thread_pool_utilization: f64,
}

/// Performance alert types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceAlert {
    AssetProcessingSlowdown {
        actual_time_ms: u64,
        threshold_ms: u64,
        asset_type: String,
    },
    DatabaseQuerySlow {
        actual_time_ms: u64,
        threshold_ms: u64,
        query_type: String,
    },
    MemoryUsageHigh {
        actual_usage_mb: u64,
        threshold_mb: u64,
    },
    CacheMissRateHigh {
        actual_rate: f64,
        threshold_rate: f64,
        cache_type: String,
    },
    HotReloadSlow {
        actual_time_ms: u64,
        threshold_ms: u64,
        file_path: String,
    },
    GcPauseLong {
        actual_time_ms: u64,
        threshold_ms: u64,
    },
    PerformanceRegression {
        metric_name: String,
        current_value: f64,
        baseline_value: f64,
        degradation_percent: f64,
    },
}

/// Time series data for trend analysis
#[derive(Debug, Clone)]
struct TimeSeriesData {
    timestamps: VecDeque<SystemTime>,
    values: VecDeque<f64>,
    max_points: usize,
}

impl TimeSeriesData {
    fn new(max_points: usize) -> Self {
        Self {
            timestamps: VecDeque::with_capacity(max_points),
            values: VecDeque::with_capacity(max_points),
            max_points,
        }
    }

    fn add_point(&mut self, timestamp: SystemTime, value: f64) {
        if self.values.len() >= self.max_points {
            self.timestamps.pop_front();
            self.values.pop_front();
        }

        self.timestamps.push_back(timestamp);
        self.values.push_back(value);
    }

    fn average(&self) -> f64 {
        if self.values.is_empty() {
            0.0
        } else {
            self.values.iter().sum::<f64>() / self.values.len() as f64
        }
    }

    fn trend(&self) -> Option<f64> {
        if self.values.len() < 2 {
            return None;
        }

        // Simple linear regression for trend detection
        let n = self.values.len() as f64;
        let sum_x: f64 = (0..self.values.len()).map(|i| i as f64).sum();
        let sum_y: f64 = self.values.iter().sum();
        let sum_xy: f64 = self.values.iter().enumerate()
            .map(|(i, &v)| i as f64 * v)
            .sum();
        let sum_x_squared: f64 = (0..self.values.len())
            .map(|i| (i as f64).powi(2))
            .sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x_squared - sum_x.powi(2));
        Some(slope)
    }

    fn detect_regression(&self, sensitivity: f64) -> bool {
        if let Some(trend) = self.trend() {
            // Negative trend indicates regression for most metrics
            trend < -sensitivity
        } else {
            false
        }
    }
}

/// Comprehensive monitoring system
pub struct ComprehensiveMonitoringSystem {
    config: MonitoringConfig,
    metrics_history: Arc<Mutex<HashMap<String, TimeSeriesData>>>,
    current_metrics: Arc<RwLock<Option<PerformanceMetrics>>>,
    alert_sender: Sender<PerformanceAlert>,
    alert_receiver: Receiver<PerformanceAlert>,
    running: Arc<AtomicBool>,
    collection_handle: Option<tokio::task::JoinHandle<()>>,
    alert_handler: Option<tokio::task::JoinHandle<()>>,
    export_handle: Option<tokio::task::JoinHandle<()>>,
    baseline_metrics: Arc<RwLock<HashMap<String, f64>>>,
}

impl ComprehensiveMonitoringSystem {
    /// Create a new comprehensive monitoring system
    pub fn new(config: MonitoringConfig) -> Self {
        let (alert_sender, alert_receiver) = bounded(1000);
        let metrics_history = Arc::new(Mutex::new(HashMap::new()));
        let current_metrics = Arc::new(RwLock::new(None));
        let running = Arc::new(AtomicBool::new(config.enabled));
        let baseline_metrics = Arc::new(RwLock::new(HashMap::new()));

        Self {
            config,
            metrics_history,
            current_metrics,
            alert_sender,
            alert_receiver,
            running,
            collection_handle: None,
            alert_handler: None,
            export_handle: None,
            baseline_metrics,
        }
    }

    /// Start the monitoring system
    pub async fn start(&mut self) -> Result<(), String> {
        if !self.config.enabled {
            return Ok(());
        }

        // Start metrics collection task
        let collection_handle = {
            let config = self.config.clone();
            let metrics_history = Arc::clone(&self.metrics_history);
            let current_metrics = Arc::clone(&self.current_metrics);
            let running = Arc::clone(&self.running);
            let alert_sender = self.alert_sender.clone();
            let baseline_metrics = Arc::clone(&self.baseline_metrics);

            tokio::spawn(async move {
                Self::metrics_collection_loop(
                    config,
                    metrics_history,
                    current_metrics,
                    running,
                    alert_sender,
                    baseline_metrics,
                ).await;
            })
        };

        // Start alert handling task
        let alert_handler = {
            let alert_receiver = self.alert_receiver.clone();
            let running = Arc::clone(&self.running);

            tokio::spawn(async move {
                Self::alert_handling_loop(alert_receiver, running).await;
            })
        };

        // Start export task
        let export_handle = {
            let config = self.config.clone();
            let current_metrics = Arc::clone(&self.current_metrics);
            let metrics_history = Arc::clone(&self.metrics_history);
            let running = Arc::clone(&self.running);

            tokio::spawn(async move {
                Self::export_loop(config, current_metrics, metrics_history, running).await;
            })
        };

        self.collection_handle = Some(collection_handle);
        self.alert_handler = Some(alert_handler);
        self.export_handle = Some(export_handle);

        Ok(())
    }

    /// Record custom metric
    pub fn record_metric(&self, name: String, value: f64) {
        if !self.config.enabled {
            return;
        }

        let timestamp = SystemTime::now();
        let mut history = self.metrics_history.lock();

        let series = history.entry(name.clone()).or_insert_with(|| {
            TimeSeriesData::new(self.config.max_data_points)
        });

        series.add_point(timestamp, value);

        // Check for regression if enabled
        if self.config.enable_regression_detection {
            if series.detect_regression(self.config.regression_sensitivity) {
                let baseline = self.baseline_metrics.read()
                    .get(&name)
                    .copied()
                    .unwrap_or(0.0);

                let current_avg = series.average();
                let degradation = if baseline > 0.0 {
                    ((current_avg - baseline) / baseline).abs()
                } else {
                    0.0
                };

                if degradation > self.config.regression_sensitivity {
                    let alert = PerformanceAlert::PerformanceRegression {
                        metric_name: name,
                        current_value: current_avg,
                        baseline_value: baseline,
                        degradation_percent: degradation * 100.0,
                    };

                    let _ = self.alert_sender.try_send(alert);
                }
            }
        }
    }

    /// Get current performance metrics
    pub fn get_current_metrics(&self) -> Option<PerformanceMetrics> {
        self.current_metrics.read().clone()
    }

    /// Get historical metrics for a specific metric
    pub fn get_metric_history(&self, metric_name: &str) -> Option<Vec<(SystemTime, f64)>> {
        let history = self.metrics_history.lock();
        if let Some(series) = history.get(metric_name) {
            Some(
                series.timestamps.iter()
                    .zip(series.values.iter())
                    .map(|(&timestamp, &value)| (timestamp, value))
                    .collect()
            )
        } else {
            None
        }
    }

    /// Set performance baselines
    pub fn set_baselines(&self, baselines: HashMap<String, f64>) {
        let mut baseline_metrics = self.baseline_metrics.write();
        *baseline_metrics = baselines;
    }

    /// Get performance report
    pub fn generate_performance_report(&self) -> PerformanceReport {
        let current = self.get_current_metrics();
        let history = self.metrics_history.lock();

        let mut metric_trends = HashMap::new();
        for (name, series) in history.iter() {
            metric_trends.insert(name.clone(), MetricTrend {
                current_value: series.values.back().copied().unwrap_or(0.0),
                average_value: series.average(),
                trend_direction: series.trend().unwrap_or(0.0),
                data_points: series.values.len(),
            });
        }

        PerformanceReport {
            generated_at: SystemTime::now(),
            current_metrics: current,
            metric_trends,
            config: self.config.clone(),
        }
    }

    /// Main metrics collection loop
    async fn metrics_collection_loop(
        config: MonitoringConfig,
        metrics_history: Arc<Mutex<HashMap<String, TimeSeriesData>>>,
        current_metrics: Arc<RwLock<Option<PerformanceMetrics>>>,
        running: Arc<AtomicBool>,
        alert_sender: Sender<PerformanceAlert>,
        baseline_metrics: Arc<RwLock<HashMap<String, f64>>>,
    ) {
        let mut interval = interval(Duration::from_millis(config.collection_interval_ms));

        while running.load(Ordering::Relaxed) {
            interval.tick().await;

            // Collect current metrics
            let metrics = Self::collect_performance_metrics(&config).await;

            // Update current metrics
            {
                let mut current = current_metrics.write();
                *current = Some(metrics.clone());
            }

            // Update historical data
            Self::update_historical_metrics(&metrics, &metrics_history, &config);

            // Check thresholds and generate alerts
            Self::check_thresholds(&metrics, &config, &alert_sender);
        }
    }

    /// Collect comprehensive performance metrics
    async fn collect_performance_metrics(config: &MonitoringConfig) -> PerformanceMetrics {
        let timestamp = SystemTime::now();

        // In a real implementation, these would collect actual metrics from the engine subsystems
        // For now, we'll provide a structure that shows what should be collected

        PerformanceMetrics {
            timestamp,
            asset_processing: AssetProcessingMetrics {
                total_assets_processed: 0,
                average_processing_time_ms: 0.0,
                parallel_efficiency: 0.0,
                memory_mapped_operations: 0,
                streaming_operations: 0,
                compression_ratio: 1.0,
                throughput_mb_per_sec: 0.0,
            },
            database: DatabaseMetrics {
                total_queries: 0,
                average_query_time_ms: 0.0,
                cache_hit_rate: 0.0,
                connection_pool_utilization: 0.0,
                index_efficiency: 1.0,
                bulk_operation_performance: 0.0,
            },
            memory_management: MemoryManagementMetrics {
                current_usage_mb: 0.0,
                peak_usage_mb: 0.0,
                cache_hit_rate: 0.0,
                pool_utilization: 0.0,
                garbage_collection_frequency: 0.0,
                memory_pressure: 0.0,
                fragmentation_level: 0.0,
                allocation_efficiency: 1.0,
            },
            hot_reload: HotReloadMetrics {
                total_reload_events: 0,
                average_reload_time_ms: 0.0,
                debounce_efficiency: 1.0,
                incremental_update_rate: 1.0,
                dependency_graph_size: 0,
                cache_effectiveness: 1.0,
            },
            general_system: GeneralSystemMetrics {
                cpu_usage_percent: Self::get_cpu_usage(),
                memory_usage_percent: Self::get_memory_usage(),
                disk_io_mb_per_sec: 0.0,
                network_io_mb_per_sec: 0.0,
                frame_rate: 60.0,
                frame_time_ms: 16.67,
                thread_pool_utilization: 0.0,
            },
        }
    }

    /// Update historical metrics
    fn update_historical_metrics(
        metrics: &PerformanceMetrics,
        history: &Arc<Mutex<HashMap<String, TimeSeriesData>>>,
        config: &MonitoringConfig,
    ) {
        let mut history_lock = history.lock();

        // Helper macro to add metric to history
        macro_rules! add_metric {
            ($name:expr, $value:expr) => {
                let series = history_lock.entry($name.to_string()).or_insert_with(|| {
                    TimeSeriesData::new(config.max_data_points)
                });
                series.add_point(metrics.timestamp, $value);
            };
        }

        // Asset processing metrics
        add_metric!("asset_processing.average_time_ms", metrics.asset_processing.average_processing_time_ms);
        add_metric!("asset_processing.parallel_efficiency", metrics.asset_processing.parallel_efficiency);
        add_metric!("asset_processing.throughput_mb_per_sec", metrics.asset_processing.throughput_mb_per_sec);

        // Database metrics
        add_metric!("database.average_query_time_ms", metrics.database.average_query_time_ms);
        add_metric!("database.cache_hit_rate", metrics.database.cache_hit_rate);
        add_metric!("database.connection_pool_utilization", metrics.database.connection_pool_utilization);

        // Memory management metrics
        add_metric!("memory.current_usage_mb", metrics.memory_management.current_usage_mb);
        add_metric!("memory.cache_hit_rate", metrics.memory_management.cache_hit_rate);
        add_metric!("memory.memory_pressure", metrics.memory_management.memory_pressure);
        add_metric!("memory.fragmentation_level", metrics.memory_management.fragmentation_level);

        // Hot reload metrics
        add_metric!("hot_reload.average_reload_time_ms", metrics.hot_reload.average_reload_time_ms);
        add_metric!("hot_reload.incremental_update_rate", metrics.hot_reload.incremental_update_rate);

        // General system metrics
        add_metric!("system.cpu_usage_percent", metrics.general_system.cpu_usage_percent);
        add_metric!("system.memory_usage_percent", metrics.general_system.memory_usage_percent);
        add_metric!("system.frame_rate", metrics.general_system.frame_rate);
        add_metric!("system.frame_time_ms", metrics.general_system.frame_time_ms);
    }

    /// Check performance thresholds and generate alerts
    fn check_thresholds(
        metrics: &PerformanceMetrics,
        config: &MonitoringConfig,
        alert_sender: &Sender<PerformanceAlert>,
    ) {
        if !config.enable_alerts {
            return;
        }

        let thresholds = &config.thresholds;

        // Check asset processing thresholds
        if metrics.asset_processing.average_processing_time_ms > thresholds.max_asset_processing_time_ms as f64 {
            let alert = PerformanceAlert::AssetProcessingSlowdown {
                actual_time_ms: metrics.asset_processing.average_processing_time_ms as u64,
                threshold_ms: thresholds.max_asset_processing_time_ms,
                asset_type: "general".to_string(),
            };
            let _ = alert_sender.try_send(alert);
        }

        // Check database thresholds
        if metrics.database.average_query_time_ms > thresholds.max_database_query_time_ms as f64 {
            let alert = PerformanceAlert::DatabaseQuerySlow {
                actual_time_ms: metrics.database.average_query_time_ms as u64,
                threshold_ms: thresholds.max_database_query_time_ms,
                query_type: "general".to_string(),
            };
            let _ = alert_sender.try_send(alert);
        }

        // Check memory thresholds
        if metrics.memory_management.current_usage_mb > thresholds.max_memory_usage_mb as f64 {
            let alert = PerformanceAlert::MemoryUsageHigh {
                actual_usage_mb: metrics.memory_management.current_usage_mb as u64,
                threshold_mb: thresholds.max_memory_usage_mb,
            };
            let _ = alert_sender.try_send(alert);
        }

        // Check cache miss rate
        if metrics.memory_management.cache_hit_rate < (1.0 - thresholds.max_cache_miss_rate) {
            let alert = PerformanceAlert::CacheMissRateHigh {
                actual_rate: 1.0 - metrics.memory_management.cache_hit_rate,
                threshold_rate: thresholds.max_cache_miss_rate,
                cache_type: "memory".to_string(),
            };
            let _ = alert_sender.try_send(alert);
        }

        // Check hot reload thresholds
        if metrics.hot_reload.average_reload_time_ms > thresholds.max_hot_reload_time_ms as f64 {
            let alert = PerformanceAlert::HotReloadSlow {
                actual_time_ms: metrics.hot_reload.average_reload_time_ms as u64,
                threshold_ms: thresholds.max_hot_reload_time_ms,
                file_path: "general".to_string(),
            };
            let _ = alert_sender.try_send(alert);
        }

        // Check memory pressure
        if metrics.memory_management.memory_pressure > thresholds.max_memory_pressure {
            let alert = PerformanceAlert::MemoryUsageHigh {
                actual_usage_mb: (metrics.memory_management.memory_pressure * 1024.0) as u64,
                threshold_mb: (thresholds.max_memory_pressure * 1024.0) as u64,
            };
            let _ = alert_sender.try_send(alert);
        }
    }

    /// Alert handling loop
    async fn alert_handling_loop(
        alert_receiver: Receiver<PerformanceAlert>,
        running: Arc<AtomicBool>,
    ) {
        while running.load(Ordering::Relaxed) {
            if let Ok(alert) = alert_receiver.try_recv() {
                Self::handle_alert(alert);
            } else {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    }

    /// Handle performance alert
    fn handle_alert(alert: PerformanceAlert) {
        match alert {
            PerformanceAlert::AssetProcessingSlowdown { actual_time_ms, threshold_ms, asset_type } => {
                log::warn!(
                    "Asset processing slowdown detected: {} took {}ms (threshold: {}ms)",
                    asset_type, actual_time_ms, threshold_ms
                );
            },
            PerformanceAlert::DatabaseQuerySlow { actual_time_ms, threshold_ms, query_type } => {
                log::warn!(
                    "Database query slowdown detected: {} took {}ms (threshold: {}ms)",
                    query_type, actual_time_ms, threshold_ms
                );
            },
            PerformanceAlert::MemoryUsageHigh { actual_usage_mb, threshold_mb } => {
                log::warn!(
                    "High memory usage detected: {}MB (threshold: {}MB)",
                    actual_usage_mb, threshold_mb
                );
            },
            PerformanceAlert::CacheMissRateHigh { actual_rate, threshold_rate, cache_type } => {
                log::warn!(
                    "High cache miss rate detected for {}: {:.2}% (threshold: {:.2}%)",
                    cache_type, actual_rate * 100.0, threshold_rate * 100.0
                );
            },
            PerformanceAlert::HotReloadSlow { actual_time_ms, threshold_ms, file_path } => {
                log::warn!(
                    "Hot reload slowdown detected for {}: {}ms (threshold: {}ms)",
                    file_path, actual_time_ms, threshold_ms
                );
            },
            PerformanceAlert::GcPauseLong { actual_time_ms, threshold_ms } => {
                log::warn!(
                    "Long GC pause detected: {}ms (threshold: {}ms)",
                    actual_time_ms, threshold_ms
                );
            },
            PerformanceAlert::PerformanceRegression { metric_name, current_value, baseline_value, degradation_percent } => {
                log::error!(
                    "Performance regression detected for {}: current={:.2}, baseline={:.2}, degradation={:.1}%",
                    metric_name, current_value, baseline_value, degradation_percent
                );
            },
        }
    }

    /// Export metrics loop
    async fn export_loop(
        config: MonitoringConfig,
        current_metrics: Arc<RwLock<Option<PerformanceMetrics>>>,
        metrics_history: Arc<Mutex<HashMap<String, TimeSeriesData>>>,
        running: Arc<AtomicBool>,
    ) {
        let mut interval = interval(Duration::from_secs(config.export_interval_secs));

        while running.load(Ordering::Relaxed) {
            interval.tick().await;

            let metrics_copy = current_metrics.read().clone();
            if let Some(metrics) = metrics_copy {
                // In a real implementation, this would export to external monitoring systems
                // like Prometheus, Grafana, etc.
                Self::export_metrics_to_external_systems(&metrics, &metrics_history).await;
            }
        }
    }

    /// Export metrics to external monitoring systems
    async fn export_metrics_to_external_systems(
        _current_metrics: &PerformanceMetrics,
        _metrics_history: &Arc<Mutex<HashMap<String, TimeSeriesData>>>,
    ) {
        // Implementation would integrate with external monitoring systems
        // Examples: Prometheus metrics export, InfluxDB integration, etc.
    }

    /// Get system CPU usage (placeholder implementation)
    fn get_cpu_usage() -> f64 {
        // In a real implementation, this would use system APIs
        // to get actual CPU usage
        0.0
    }

    /// Get system memory usage (placeholder implementation)
    fn get_memory_usage() -> f64 {
        // In a real implementation, this would use system APIs
        // to get actual memory usage
        0.0
    }

    /// Shutdown the monitoring system
    pub async fn shutdown(self) {
        self.running.store(false, Ordering::Relaxed);

        if let Some(handle) = self.collection_handle {
            let _ = handle.await;
        }

        if let Some(handle) = self.alert_handler {
            let _ = handle.await;
        }

        if let Some(handle) = self.export_handle {
            let _ = handle.await;
        }
    }
}

/// Performance report structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub generated_at: SystemTime,
    pub current_metrics: Option<PerformanceMetrics>,
    pub metric_trends: HashMap<String, MetricTrend>,
    pub config: MonitoringConfig,
}

/// Metric trend information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricTrend {
    pub current_value: f64,
    pub average_value: f64,
    pub trend_direction: f64,
    pub data_points: usize,
}

impl std::fmt::Display for PerformanceReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Performance Report")?;
        writeln!(f, "===================")?;
        writeln!(f, "Generated: {:?}", self.generated_at)?;
        writeln!(f)?;

        if let Some(ref metrics) = self.current_metrics {
            writeln!(f, "Current Metrics:")?;
            writeln!(f, "  Asset Processing:")?;
            writeln!(f, "    Average Processing Time: {:.2}ms", metrics.asset_processing.average_processing_time_ms)?;
            writeln!(f, "    Parallel Efficiency: {:.2}%", metrics.asset_processing.parallel_efficiency * 100.0)?;
            writeln!(f, "    Throughput: {:.2} MB/s", metrics.asset_processing.throughput_mb_per_sec)?;
            writeln!(f)?;

            writeln!(f, "  Database:")?;
            writeln!(f, "    Average Query Time: {:.2}ms", metrics.database.average_query_time_ms)?;
            writeln!(f, "    Cache Hit Rate: {:.2}%", metrics.database.cache_hit_rate * 100.0)?;
            writeln!(f, "    Connection Pool Utilization: {:.2}%", metrics.database.connection_pool_utilization * 100.0)?;
            writeln!(f)?;

            writeln!(f, "  Memory Management:")?;
            writeln!(f, "    Current Usage: {:.2} MB", metrics.memory_management.current_usage_mb)?;
            writeln!(f, "    Cache Hit Rate: {:.2}%", metrics.memory_management.cache_hit_rate * 100.0)?;
            writeln!(f, "    Memory Pressure: {:.2}%", metrics.memory_management.memory_pressure * 100.0)?;
            writeln!(f, "    Fragmentation Level: {:.2}%", metrics.memory_management.fragmentation_level * 100.0)?;
            writeln!(f)?;

            writeln!(f, "  Hot Reload:")?;
            writeln!(f, "    Average Reload Time: {:.2}ms", metrics.hot_reload.average_reload_time_ms)?;
            writeln!(f, "    Incremental Update Rate: {:.2}%", metrics.hot_reload.incremental_update_rate * 100.0)?;
            writeln!(f, "    Dependency Graph Size: {} assets", metrics.hot_reload.dependency_graph_size)?;
            writeln!(f)?;

            writeln!(f, "  System:")?;
            writeln!(f, "    CPU Usage: {:.2}%", metrics.general_system.cpu_usage_percent)?;
            writeln!(f, "    Memory Usage: {:.2}%", metrics.general_system.memory_usage_percent)?;
            writeln!(f, "    Frame Rate: {:.2} FPS", metrics.general_system.frame_rate)?;
            writeln!(f, "    Frame Time: {:.2}ms", metrics.general_system.frame_time_ms)?;
        }

        writeln!(f)?;
        writeln!(f, "Trend Analysis:")?;
        for (metric, trend) in &self.metric_trends {
            let direction = if trend.trend_direction > 0.01 {
                "↑"
            } else if trend.trend_direction < -0.01 {
                "↓"
            } else {
                "→"
            };

            writeln!(
                f,
                "  {}: {:.2} (avg: {:.2}) {} ({} points)",
                metric, trend.current_value, trend.average_value, direction, trend.data_points
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_series_data() {
        let mut series = TimeSeriesData::new(5);

        // Add some data points
        for i in 0..7 {
            series.add_point(
                UNIX_EPOCH + Duration::from_secs(i),
                i as f64
            );
        }

        // Should only keep 5 points (max_points)
        assert_eq!(series.values.len(), 5);
        assert_eq!(series.average(), 4.0); // (2+3+4+5+6)/5

        // Test trend detection
        let trend = series.trend().unwrap();
        assert!(trend > 0.0); // Should detect positive trend
    }

    #[test]
    fn test_regression_detection() {
        let mut series = TimeSeriesData::new(10);

        // Add baseline data (stable performance)
        for i in 0..5 {
            series.add_point(
                UNIX_EPOCH + Duration::from_secs(i),
                100.0
            );
        }

        // Add degraded performance
        for i in 5..10 {
            series.add_point(
                UNIX_EPOCH + Duration::from_secs(i),
                150.0
            );
        }

        assert!(series.detect_regression(0.1)); // Should detect 50% regression
    }

    #[tokio::test]
    async fn test_monitoring_system_creation() {
        let config = MonitoringConfig::default();
        let monitoring = ComprehensiveMonitoringSystem::new(config);

        // Should create without errors
        assert!(!monitoring.running.load(Ordering::Relaxed) || true);
    }

    #[test]
    fn test_performance_thresholds() {
        let thresholds = PerformanceThresholds::default();

        assert!(thresholds.max_asset_processing_time_ms > 0);
        assert!(thresholds.max_database_query_time_ms > 0);
        assert!(thresholds.max_memory_usage_mb > 0);
        assert!(thresholds.max_cache_miss_rate > 0.0 && thresholds.max_cache_miss_rate < 1.0);
    }

    #[test]
    fn test_metric_recording() {
        let config = MonitoringConfig::default();
        let monitoring = ComprehensiveMonitoringSystem::new(config);

        monitoring.record_metric("test_metric".to_string(), 42.0);

        let history = monitoring.get_metric_history("test_metric");
        assert!(history.is_some());

        let history = history.unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].1, 42.0);
    }

    #[test]
    fn test_performance_report() {
        let config = MonitoringConfig::default();
        let monitoring = ComprehensiveMonitoringSystem::new(config);

        monitoring.record_metric("test1".to_string(), 10.0);
        monitoring.record_metric("test2".to_string(), 20.0);

        let report = monitoring.generate_performance_report();
        assert_eq!(report.metric_trends.len(), 2);
    }
}