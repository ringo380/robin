// Robin Game Engine - Real-time Performance Monitoring and Alerting
// Comprehensive monitoring system for all Phase 3 performance metrics

use crate::engine::{
    assets::{AssetPipeline, DatabasePerformanceMetrics as AssetDatabasePerformanceMetrics},
    error::RobinResult,
};
use std::{
    collections::{HashMap, VecDeque},
    time::{Duration, Instant, SystemTime},
    sync::{Arc, Mutex, atomic::{AtomicU64, AtomicUsize, Ordering}},
    thread,
};
use serde::{Serialize, Deserialize};

/// Real-time performance monitoring system
pub struct PerformanceMonitor {
    config: MonitoringConfig,
    metrics_collectors: HashMap<String, Box<dyn MetricsCollector + Send + Sync>>,
    alert_rules: Vec<AlertRule>,
    metrics_history: Arc<Mutex<MetricsHistory>>,
    alerts_manager: AlertManager,
    sampling_thread: Option<thread::JoinHandle<()>>,
    is_running: Arc<std::sync::atomic::AtomicBool>,
}

impl std::fmt::Debug for PerformanceMonitor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PerformanceMonitor")
            .field("config", &self.config)
            .field("metrics_collectors", &format!("{} collectors", self.metrics_collectors.len()))
            .field("alert_rules", &self.alert_rules)
            .field("alerts_manager", &self.alerts_manager)
            .field("is_running", &self.is_running.load(Ordering::Relaxed))
            .finish()
    }
}

/// Configuration for performance monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub sampling_interval_ms: u64,
    pub history_retention_minutes: u32,
    pub enable_alerting: bool,
    pub enable_real_time_alerts: bool,
    pub alert_throttle_seconds: u32,
    pub metrics_export_interval_seconds: u32,
    pub detailed_logging: bool,
    pub performance_dashboard: bool,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            sampling_interval_ms: 1000, // 1 second
            history_retention_minutes: 60, // 1 hour
            enable_alerting: true,
            enable_real_time_alerts: true,
            alert_throttle_seconds: 30,
            metrics_export_interval_seconds: 300, // 5 minutes
            detailed_logging: false,
            performance_dashboard: true,
        }
    }
}

/// Metrics collection trait
pub trait MetricsCollector {
    fn name(&self) -> &str;
    fn collect(&self) -> RobinResult<SystemMetrics>;
    fn is_healthy(&self) -> bool;
}

/// System performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: SystemTime,
    pub cpu_usage_percent: f32,
    pub memory_usage_mb: f32,
    pub memory_available_mb: f32,
    pub disk_io_read_mb_per_sec: f32,
    pub disk_io_write_mb_per_sec: f32,
    pub network_rx_mb_per_sec: f32,
    pub network_tx_mb_per_sec: f32,
    pub gpu_usage_percent: Option<f32>,
    pub gpu_memory_mb: Option<f32>,
    pub custom_metrics: HashMap<String, f64>,
}

/// Metrics history storage
#[derive(Debug)]
struct MetricsHistory {
    samples: VecDeque<PerformanceSample>,
    max_samples: usize,
}

/// Individual performance sample
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSample {
    pub timestamp: SystemTime,
    pub system_metrics: SystemMetrics,
    pub asset_pipeline_metrics: Option<PipelinePerformanceMetrics>,
    pub ui_metrics: Option<UIPerformanceMetrics>,
    pub database_metrics: Option<DatabasePerformanceMetrics>,
    pub hot_reload_metrics: Option<HotReloadPerformanceMetrics>,
}

/// Asset pipeline performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelinePerformanceMetrics {
    pub processing_time_ms: f32,
    pub assets_processed: u32,
    pub cache_hits: u32,
    pub cache_misses: u32,
    pub memory_usage_mb: f32,
    pub parallel_jobs_active: u32,
    pub throughput_assets_per_sec: f32,
    pub error_count: u32,
}

/// UI rendering performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIPerformanceMetrics {
    pub frame_time_ms: f32,
    pub render_time_ms: f32,
    pub layout_time_ms: f32,
    pub paint_time_ms: f32,
    pub components_rendered: u32,
    pub virtual_scroll_items: u32,
    pub state_updates_per_sec: f32,
    pub memory_usage_mb: f32,
}

/// Database performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabasePerformanceMetrics {
    pub query_time_p50_ms: f32,
    pub query_time_p95_ms: f32,
    pub queries_per_second: f32,
    pub connection_pool_usage: f32,
    pub cache_hit_ratio: f32,
    pub active_connections: u32,
    pub slow_queries_count: u32,
    pub memory_usage_mb: f32,
}

/// Hot reload performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotReloadPerformanceMetrics {
    pub reload_time_ms: f32,
    pub files_watched: u32,
    pub files_changed_per_min: f32,
    pub debounce_efficiency: f32,
    pub incremental_updates: u32,
    pub full_rebuilds: u32,
    pub memory_usage_mb: f32,
}

/// Alert rule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub name: String,
    pub metric_path: String,
    pub threshold: f32,
    pub comparison: ComparisonOperator,
    pub duration_seconds: u32,
    pub severity: AlertSeverity,
    pub enabled: bool,
    pub description: String,
}

/// Alert comparison operators
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComparisonOperator {
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Equal,
    NotEqual,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

/// Alert management system
#[derive(Debug)]
struct AlertManager {
    active_alerts: HashMap<String, ActiveAlert>,
    alert_history: VecDeque<AlertEvent>,
    throttle_tracker: HashMap<String, SystemTime>,
}

/// Active alert tracking
#[derive(Debug, Clone)]
struct ActiveAlert {
    rule: AlertRule,
    triggered_at: SystemTime,
    last_updated: SystemTime,
    current_value: f32,
    notification_count: u32,
}

/// Alert event for history tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEvent {
    pub alert_name: String,
    pub severity: AlertSeverity,
    pub message: String,
    pub triggered_at: SystemTime,
    pub resolved_at: Option<SystemTime>,
    pub current_value: f32,
    pub threshold: f32,
}

impl PerformanceMonitor {
    pub fn new(config: MonitoringConfig) -> RobinResult<Self> {
        let max_samples = (config.history_retention_minutes as u64 * 60 * 1000 / config.sampling_interval_ms) as usize;

        let metrics_history = Arc::new(Mutex::new(MetricsHistory {
            samples: VecDeque::with_capacity(max_samples),
            max_samples,
        }));

        let mut monitor = Self {
            config,
            metrics_collectors: HashMap::new(),
            alert_rules: Vec::new(),
            metrics_history,
            alerts_manager: AlertManager {
                active_alerts: HashMap::new(),
                alert_history: VecDeque::new(),
                throttle_tracker: HashMap::new(),
            },
            sampling_thread: None,
            is_running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        };

        // Register default collectors
        monitor.register_default_collectors()?;
        monitor.setup_default_alerts();

        Ok(monitor)
    }

    /// Register a custom metrics collector
    pub fn register_collector<C>(&mut self, collector: C) -> RobinResult<()>
    where
        C: MetricsCollector + Send + Sync + 'static,
    {
        let name = collector.name().to_string();
        self.metrics_collectors.insert(name.clone(), Box::new(collector));
        log::info!("Registered metrics collector: {}", name);
        Ok(())
    }

    /// Add an alert rule
    pub fn add_alert_rule(&mut self, rule: AlertRule) {
        log::info!("Added alert rule: {} ({})", rule.name, rule.description);
        self.alert_rules.push(rule);
    }

    /// Start monitoring
    pub fn start(&mut self) -> RobinResult<()> {
        if self.is_running.load(Ordering::Relaxed) {
            return Ok(());
        }

        self.is_running.store(true, Ordering::Relaxed);

        let is_running = Arc::clone(&self.is_running);
        let metrics_history = Arc::clone(&self.metrics_history);
        let sampling_interval = Duration::from_millis(self.config.sampling_interval_ms);

        // Start background sampling thread
        let handle = thread::spawn(move || {
            let mut last_sample = Instant::now();

            while is_running.load(Ordering::Relaxed) {
                if last_sample.elapsed() >= sampling_interval {
                    // Collect metrics from all collectors
                    // In a real implementation, this would iterate through collectors

                    let sample = PerformanceSample {
                        timestamp: SystemTime::now(),
                        system_metrics: Self::collect_system_metrics(),
                        asset_pipeline_metrics: None, // Would be filled by collector
                        ui_metrics: None,
                        database_metrics: None,
                        hot_reload_metrics: None,
                    };

                    // Store sample
                    {
                        let mut history = metrics_history.lock().unwrap();
                        history.samples.push_back(sample);

                        if history.samples.len() > history.max_samples {
                            history.samples.pop_front();
                        }
                    }

                    last_sample = Instant::now();
                }

                thread::sleep(Duration::from_millis(100)); // Small sleep to prevent busy waiting
            }
        });

        self.sampling_thread = Some(handle);
        log::info!("Performance monitoring started with {}ms sampling interval", self.config.sampling_interval_ms);

        Ok(())
    }

    /// Stop monitoring
    pub fn stop(&mut self) -> RobinResult<()> {
        if !self.is_running.load(Ordering::Relaxed) {
            return Ok(());
        }

        self.is_running.store(false, Ordering::Relaxed);

        if let Some(handle) = self.sampling_thread.take() {
            handle.join().map_err(|_| {
                crate::engine::error::RobinError::new("Failed to join monitoring thread")
            })?;
        }

        log::info!("Performance monitoring stopped");
        Ok(())
    }

    /// Get current metrics snapshot
    pub fn get_current_metrics(&self) -> RobinResult<PerformanceSample> {
        let history = self.metrics_history.lock().unwrap();

        if let Some(latest) = history.samples.back() {
            Ok(latest.clone())
        } else {
            // Return empty sample if no data available
            Ok(PerformanceSample {
                timestamp: SystemTime::now(),
                system_metrics: Self::collect_system_metrics(),
                asset_pipeline_metrics: None,
                ui_metrics: None,
                database_metrics: None,
                hot_reload_metrics: None,
            })
        }
    }

    /// Get metrics history
    pub fn get_metrics_history(&self, duration_minutes: Option<u32>) -> RobinResult<Vec<PerformanceSample>> {
        let history = self.metrics_history.lock().unwrap();

        let cutoff_time = duration_minutes.map(|minutes| {
            SystemTime::now() - Duration::from_secs(minutes as u64 * 60)
        });

        let filtered_samples: Vec<_> = history.samples
            .iter()
            .filter(|sample| {
                if let Some(cutoff) = cutoff_time {
                    sample.timestamp >= cutoff
                } else {
                    true
                }
            })
            .cloned()
            .collect();

        Ok(filtered_samples)
    }

    /// Check alerts and trigger notifications
    pub fn check_alerts(&mut self) -> RobinResult<Vec<AlertEvent>> {
        if !self.config.enable_alerting {
            return Ok(Vec::new());
        }

        let current_sample = self.get_current_metrics()?;
        let mut new_alerts = Vec::new();

        for rule in &self.alert_rules {
            if !rule.enabled {
                continue;
            }

            let current_value = self.extract_metric_value(&current_sample, &rule.metric_path)?;
            let threshold_exceeded = self.evaluate_threshold(current_value, rule.threshold, &rule.comparison);

            if threshold_exceeded {
                // Check if alert is already active
                if let Some(active_alert) = self.alerts_manager.active_alerts.get_mut(&rule.name) {
                    // Update existing alert
                    active_alert.last_updated = SystemTime::now();
                    active_alert.current_value = current_value;
                } else {
                    // Create new alert
                    let alert_event = AlertEvent {
                        alert_name: rule.name.clone(),
                        severity: rule.severity.clone(),
                        message: format!("{}: {} {} {} (current: {:.2})",
                                       rule.description,
                                       rule.metric_path,
                                       self.comparison_to_string(&rule.comparison),
                                       rule.threshold,
                                       current_value),
                        triggered_at: SystemTime::now(),
                        resolved_at: None,
                        current_value,
                        threshold: rule.threshold,
                    };

                    let active_alert = ActiveAlert {
                        rule: rule.clone(),
                        triggered_at: SystemTime::now(),
                        last_updated: SystemTime::now(),
                        current_value,
                        notification_count: 1,
                    };

                    self.alerts_manager.active_alerts.insert(rule.name.clone(), active_alert);
                    self.alerts_manager.alert_history.push_back(alert_event.clone());
                    new_alerts.push(alert_event);

                    log::warn!("ALERT TRIGGERED: {}", rule.name);
                }
            } else {
                // Check if we need to resolve an active alert
                if let Some(mut active_alert) = self.alerts_manager.active_alerts.remove(&rule.name) {
                    // Mark alert as resolved
                    if let Some(mut event) = self.alerts_manager.alert_history
                        .iter_mut()
                        .rev()
                        .find(|e| e.alert_name == rule.name && e.resolved_at.is_none())
                    {
                        event.resolved_at = Some(SystemTime::now());
                    }

                    log::info!("ALERT RESOLVED: {}", rule.name);
                }
            }
        }

        // Cleanup old alert history
        self.cleanup_alert_history();

        Ok(new_alerts)
    }

    /// Get dashboard data for visualization
    pub fn get_dashboard_data(&self) -> RobinResult<DashboardData> {
        let current_metrics = self.get_current_metrics()?;
        let recent_history = self.get_metrics_history(Some(15))?; // Last 15 minutes

        // Calculate statistics
        let cpu_stats = self.calculate_metric_stats(&recent_history, "cpu_usage_percent");
        let memory_stats = self.calculate_metric_stats(&recent_history, "memory_usage_mb");
        let fps_stats = self.calculate_ui_metric_stats(&recent_history, "frame_time_ms");

        Ok(DashboardData {
            current_sample: current_metrics,
            active_alerts: self.alerts_manager.active_alerts.len(),
            total_collectors: self.metrics_collectors.len(),
            cpu_usage_trend: cpu_stats,
            memory_usage_trend: memory_stats,
            fps_trend: fps_stats,
            performance_score: self.calculate_performance_score(&recent_history),
            recent_samples: recent_history,
        })
    }

    /// Export metrics for external monitoring systems
    pub fn export_metrics(&self, format: ExportFormat) -> RobinResult<String> {
        let history = self.get_metrics_history(Some(60))?; // Last hour

        match format {
            ExportFormat::Json => {
                serde_json::to_string_pretty(&history)
                    .map_err(|e| crate::engine::error::RobinError::new(&format!("JSON export failed: {}", e)))
            }
            ExportFormat::Prometheus => {
                self.export_prometheus_format(&history)
            }
            ExportFormat::InfluxDB => {
                self.export_influxdb_format(&history)
            }
        }
    }

    // Private implementation methods

    fn register_default_collectors(&mut self) -> RobinResult<()> {
        self.register_collector(SystemMetricsCollector::new())?;
        self.register_collector(AssetPipelineCollector::new())?;
        self.register_collector(UIMetricsCollector::new())?;
        self.register_collector(DatabaseCollector::new())?;
        Ok(())
    }

    fn setup_default_alerts(&mut self) {
        // CPU usage alert
        self.add_alert_rule(AlertRule {
            name: "high_cpu_usage".to_string(),
            metric_path: "system.cpu_usage_percent".to_string(),
            threshold: 90.0,
            comparison: ComparisonOperator::GreaterThan,
            duration_seconds: 30,
            severity: AlertSeverity::Warning,
            enabled: true,
            description: "High CPU usage detected".to_string(),
        });

        // Memory usage alert
        self.add_alert_rule(AlertRule {
            name: "high_memory_usage".to_string(),
            metric_path: "system.memory_usage_mb".to_string(),
            threshold: 1800.0, // 1.8GB
            comparison: ComparisonOperator::GreaterThan,
            duration_seconds: 60,
            severity: AlertSeverity::Critical,
            enabled: true,
            description: "High memory usage detected".to_string(),
        });

        // Low FPS alert
        self.add_alert_rule(AlertRule {
            name: "low_fps".to_string(),
            metric_path: "ui.frame_time_ms".to_string(),
            threshold: 20.0, // > 20ms = < 50 FPS
            comparison: ComparisonOperator::GreaterThan,
            duration_seconds: 10,
            severity: AlertSeverity::Warning,
            enabled: true,
            description: "Low FPS performance detected".to_string(),
        });
    }

    fn collect_system_metrics() -> SystemMetrics {
        // Simplified system metrics collection
        // In production, would use system APIs to get real metrics
        SystemMetrics {
            timestamp: SystemTime::now(),
            cpu_usage_percent: 45.0,
            memory_usage_mb: 1024.0,
            memory_available_mb: 2048.0,
            disk_io_read_mb_per_sec: 10.0,
            disk_io_write_mb_per_sec: 5.0,
            network_rx_mb_per_sec: 2.0,
            network_tx_mb_per_sec: 1.0,
            gpu_usage_percent: Some(60.0),
            gpu_memory_mb: Some(512.0),
            custom_metrics: HashMap::new(),
        }
    }

    fn extract_metric_value(&self, sample: &PerformanceSample, metric_path: &str) -> RobinResult<f32> {
        let parts: Vec<&str> = metric_path.split('.').collect();

        match parts.as_slice() {
            ["system", "cpu_usage_percent"] => Ok(sample.system_metrics.cpu_usage_percent),
            ["system", "memory_usage_mb"] => Ok(sample.system_metrics.memory_usage_mb),
            ["ui", "frame_time_ms"] => {
                sample.ui_metrics.as_ref()
                    .map(|m| m.frame_time_ms)
                    .ok_or_else(|| crate::engine::error::RobinError::new("UI metrics not available"))
            }
            _ => Err(crate::engine::error::RobinError::new(&format!("Unknown metric path: {}", metric_path)))
        }
    }

    fn evaluate_threshold(&self, current: f32, threshold: f32, comparison: &ComparisonOperator) -> bool {
        match comparison {
            ComparisonOperator::GreaterThan => current > threshold,
            ComparisonOperator::GreaterThanOrEqual => current >= threshold,
            ComparisonOperator::LessThan => current < threshold,
            ComparisonOperator::LessThanOrEqual => current <= threshold,
            ComparisonOperator::Equal => (current - threshold).abs() < f32::EPSILON,
            ComparisonOperator::NotEqual => (current - threshold).abs() >= f32::EPSILON,
        }
    }

    fn comparison_to_string(&self, comparison: &ComparisonOperator) -> &str {
        match comparison {
            ComparisonOperator::GreaterThan => ">",
            ComparisonOperator::GreaterThanOrEqual => ">=",
            ComparisonOperator::LessThan => "<",
            ComparisonOperator::LessThanOrEqual => "<=",
            ComparisonOperator::Equal => "==",
            ComparisonOperator::NotEqual => "!=",
        }
    }

    fn cleanup_alert_history(&mut self) {
        let cutoff = SystemTime::now() - Duration::from_secs(24 * 3600); // 24 hours

        while let Some(front) = self.alerts_manager.alert_history.front() {
            if front.triggered_at < cutoff {
                self.alerts_manager.alert_history.pop_front();
            } else {
                break;
            }
        }
    }

    fn calculate_metric_stats(&self, history: &[PerformanceSample], metric: &str) -> MetricTrend {
        if history.is_empty() {
            return MetricTrend::default();
        }

        let values: Vec<f32> = history.iter()
            .filter_map(|sample| {
                match metric {
                    "cpu_usage_percent" => Some(sample.system_metrics.cpu_usage_percent),
                    "memory_usage_mb" => Some(sample.system_metrics.memory_usage_mb),
                    _ => None,
                }
            })
            .collect();

        if values.is_empty() {
            return MetricTrend::default();
        }

        let sum: f32 = values.iter().sum();
        let avg = sum / values.len() as f32;
        let min = values.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = values.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

        MetricTrend {
            current: values.last().copied().unwrap_or(0.0),
            average: avg,
            min,
            max,
            trend_direction: if values.len() > 1 {
                let first_half_avg = values[..values.len()/2].iter().sum::<f32>() / (values.len()/2) as f32;
                let second_half_avg = values[values.len()/2..].iter().sum::<f32>() / (values.len() - values.len()/2) as f32;

                if second_half_avg > first_half_avg * 1.05 {
                    TrendDirection::Increasing
                } else if second_half_avg < first_half_avg * 0.95 {
                    TrendDirection::Decreasing
                } else {
                    TrendDirection::Stable
                }
            } else {
                TrendDirection::Stable
            },
        }
    }

    fn calculate_ui_metric_stats(&self, history: &[PerformanceSample], metric: &str) -> MetricTrend {
        // Similar to calculate_metric_stats but for UI metrics
        MetricTrend::default() // Simplified implementation
    }

    fn calculate_performance_score(&self, history: &[PerformanceSample]) -> f32 {
        if history.is_empty() {
            return 0.0;
        }

        // Simple performance scoring (0-100)
        let cpu_score = history.iter()
            .map(|s| (100.0 - s.system_metrics.cpu_usage_percent).max(0.0))
            .sum::<f32>() / history.len() as f32;

        let memory_score = history.iter()
            .map(|s| ((2048.0 - s.system_metrics.memory_usage_mb) / 2048.0 * 100.0).max(0.0))
            .sum::<f32>() / history.len() as f32;

        (cpu_score + memory_score) / 2.0
    }

    fn export_prometheus_format(&self, history: &[PerformanceSample]) -> RobinResult<String> {
        let mut output = String::new();

        if let Some(latest) = history.last() {
            output.push_str(&format!("# HELP robin_cpu_usage_percent CPU usage percentage\n"));
            output.push_str(&format!("# TYPE robin_cpu_usage_percent gauge\n"));
            output.push_str(&format!("robin_cpu_usage_percent {}\n", latest.system_metrics.cpu_usage_percent));

            output.push_str(&format!("# HELP robin_memory_usage_mb Memory usage in MB\n"));
            output.push_str(&format!("# TYPE robin_memory_usage_mb gauge\n"));
            output.push_str(&format!("robin_memory_usage_mb {}\n", latest.system_metrics.memory_usage_mb));
        }

        Ok(output)
    }

    fn export_influxdb_format(&self, history: &[PerformanceSample]) -> RobinResult<String> {
        let mut output = String::new();

        for sample in history {
            let timestamp = sample.timestamp.duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default().as_nanos();

            output.push_str(&format!(
                "robin_metrics cpu_usage={},memory_usage={} {}\n",
                sample.system_metrics.cpu_usage_percent,
                sample.system_metrics.memory_usage_mb,
                timestamp
            ));
        }

        Ok(output)
    }
}

impl Drop for PerformanceMonitor {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

// Supporting types

#[derive(Debug, Clone)]
pub enum ExportFormat {
    Json,
    Prometheus,
    InfluxDB,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    pub current_sample: PerformanceSample,
    pub active_alerts: usize,
    pub total_collectors: usize,
    pub cpu_usage_trend: MetricTrend,
    pub memory_usage_trend: MetricTrend,
    pub fps_trend: MetricTrend,
    pub performance_score: f32,
    pub recent_samples: Vec<PerformanceSample>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetricTrend {
    pub current: f32,
    pub average: f32,
    pub min: f32,
    pub max: f32,
    pub trend_direction: TrendDirection,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum TrendDirection {
    #[default]
    Stable,
    Increasing,
    Decreasing,
}

// Concrete metrics collectors

struct SystemMetricsCollector;

impl SystemMetricsCollector {
    fn new() -> Self {
        Self
    }
}

impl MetricsCollector for SystemMetricsCollector {
    fn name(&self) -> &str {
        "system"
    }

    fn collect(&self) -> RobinResult<SystemMetrics> {
        Ok(PerformanceMonitor::collect_system_metrics())
    }

    fn is_healthy(&self) -> bool {
        true
    }
}

struct AssetPipelineCollector;

impl AssetPipelineCollector {
    fn new() -> Self {
        Self
    }
}

impl MetricsCollector for AssetPipelineCollector {
    fn name(&self) -> &str {
        "asset_pipeline"
    }

    fn collect(&self) -> RobinResult<SystemMetrics> {
        // Would collect from actual asset pipeline
        Ok(SystemMetrics {
            timestamp: SystemTime::now(),
            cpu_usage_percent: 20.0,
            memory_usage_mb: 256.0,
            memory_available_mb: 1792.0,
            disk_io_read_mb_per_sec: 15.0,
            disk_io_write_mb_per_sec: 8.0,
            network_rx_mb_per_sec: 0.0,
            network_tx_mb_per_sec: 0.0,
            gpu_usage_percent: None,
            gpu_memory_mb: None,
            custom_metrics: HashMap::new(),
        })
    }

    fn is_healthy(&self) -> bool {
        true
    }
}

struct UIMetricsCollector;

impl UIMetricsCollector {
    fn new() -> Self {
        Self
    }
}

impl MetricsCollector for UIMetricsCollector {
    fn name(&self) -> &str {
        "ui"
    }

    fn collect(&self) -> RobinResult<SystemMetrics> {
        // Would collect from UI system
        Ok(SystemMetrics {
            timestamp: SystemTime::now(),
            cpu_usage_percent: 10.0,
            memory_usage_mb: 128.0,
            memory_available_mb: 1920.0,
            disk_io_read_mb_per_sec: 0.0,
            disk_io_write_mb_per_sec: 0.0,
            network_rx_mb_per_sec: 0.0,
            network_tx_mb_per_sec: 0.0,
            gpu_usage_percent: Some(30.0),
            gpu_memory_mb: Some(200.0),
            custom_metrics: HashMap::new(),
        })
    }

    fn is_healthy(&self) -> bool {
        true
    }
}

struct DatabaseCollector;

impl DatabaseCollector {
    fn new() -> Self {
        Self
    }
}

impl MetricsCollector for DatabaseCollector {
    fn name(&self) -> &str {
        "database"
    }

    fn collect(&self) -> RobinResult<SystemMetrics> {
        // Would collect from database system
        Ok(SystemMetrics {
            timestamp: SystemTime::now(),
            cpu_usage_percent: 5.0,
            memory_usage_mb: 64.0,
            memory_available_mb: 1984.0,
            disk_io_read_mb_per_sec: 2.0,
            disk_io_write_mb_per_sec: 1.0,
            network_rx_mb_per_sec: 0.0,
            network_tx_mb_per_sec: 0.0,
            gpu_usage_percent: None,
            gpu_memory_mb: None,
            custom_metrics: HashMap::new(),
        })
    }

    fn is_healthy(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_monitor_creation() {
        let config = MonitoringConfig::default();
        let monitor = PerformanceMonitor::new(config);
        assert!(monitor.is_ok());
    }

    #[test]
    fn test_alert_rule_evaluation() {
        let monitor = PerformanceMonitor::new(MonitoringConfig::default()).unwrap();
        assert!(monitor.evaluate_threshold(95.0, 90.0, &ComparisonOperator::GreaterThan));
        assert!(!monitor.evaluate_threshold(85.0, 90.0, &ComparisonOperator::GreaterThan));
    }

    #[test]
    fn test_metric_extraction() {
        let monitor = PerformanceMonitor::new(MonitoringConfig::default()).unwrap();
        let sample = PerformanceSample {
            timestamp: SystemTime::now(),
            system_metrics: SystemMetrics {
                timestamp: SystemTime::now(),
                cpu_usage_percent: 75.0,
                memory_usage_mb: 1024.0,
                memory_available_mb: 1024.0,
                disk_io_read_mb_per_sec: 0.0,
                disk_io_write_mb_per_sec: 0.0,
                network_rx_mb_per_sec: 0.0,
                network_tx_mb_per_sec: 0.0,
                gpu_usage_percent: None,
                gpu_memory_mb: None,
                custom_metrics: HashMap::new(),
            },
            asset_pipeline_metrics: None,
            ui_metrics: None,
            database_metrics: None,
            hot_reload_metrics: None,
        };

        let cpu_value = monitor.extract_metric_value(&sample, "system.cpu_usage_percent").unwrap();
        assert_eq!(cpu_value, 75.0);
    }
}