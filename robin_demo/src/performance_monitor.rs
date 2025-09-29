// Real-time performance monitoring system for Robin Engine
// Tracks material batching statistics, frame rates, and optimization metrics

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use crate::material_batching::{BatchingStats, MaterialType};

/// Real-time performance metrics for the engine
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub frame_time: Duration,
    pub fps: f32,
    pub vertex_count: usize,
    pub triangle_count: usize,
    pub draw_calls: usize,
    pub batching_efficiency: f32,
    pub memory_usage_mb: f32,
    pub gpu_usage_percent: f32,
}

/// Performance monitoring configuration
#[derive(Debug, Clone)]
pub struct MonitorConfig {
    pub enabled: bool,
    pub sample_window_frames: usize,
    pub log_interval_frames: usize,
    pub performance_targets: PerformanceTargets,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sample_window_frames: 60, // 1 second at 60fps
            log_interval_frames: 300, // 5 seconds at 60fps
            performance_targets: PerformanceTargets::default(),
        }
    }
}

/// Performance targets for optimization alerts
#[derive(Debug, Clone)]
pub struct PerformanceTargets {
    pub min_fps: f32,
    pub max_frame_time_ms: f32,
    pub max_draw_calls: usize,
    pub min_batching_efficiency: f32,
    pub max_memory_usage_mb: f32,
}

impl Default for PerformanceTargets {
    fn default() -> Self {
        Self {
            min_fps: 60.0,
            max_frame_time_ms: 16.67, // 60fps target
            max_draw_calls: 100,
            min_batching_efficiency: 75.0,
            max_memory_usage_mb: 512.0,
        }
    }
}

/// Batch performance statistics
#[derive(Debug, Clone)]
pub struct BatchPerformanceStats {
    pub material_distributions: HashMap<MaterialType, usize>,
    pub batch_sizes: Vec<usize>,
    pub average_batch_size: f32,
    pub total_vertices_saved: usize,
    pub total_draw_calls_saved: usize,
    pub batching_overhead_ms: f32,
}

/// Alert types for performance issues
#[derive(Debug, Clone)]
pub enum PerformanceAlert {
    LowFrameRate { current_fps: f32, target_fps: f32 },
    HighFrameTime { current_ms: f32, target_ms: f32 },
    TooManyDrawCalls { current: usize, target: usize },
    LowBatchingEfficiency { current: f32, target: f32 },
    HighMemoryUsage { current_mb: f32, target_mb: f32 },
    OptimizationOpportunity { description: String },
}

/// Main performance monitoring system
pub struct PerformanceMonitor {
    config: MonitorConfig,
    frame_times: VecDeque<Duration>,
    metrics_history: VecDeque<PerformanceMetrics>,
    batch_stats_history: VecDeque<BatchPerformanceStats>,
    current_frame: u64,
    last_log_frame: u64,
    alerts: Vec<PerformanceAlert>,
    start_time: Instant,
}

impl PerformanceMonitor {
    pub fn new(config: MonitorConfig) -> Self {
        Self {
            config,
            frame_times: VecDeque::new(),
            metrics_history: VecDeque::new(),
            batch_stats_history: VecDeque::new(),
            current_frame: 0,
            last_log_frame: 0,
            alerts: Vec::new(),
            start_time: Instant::now(),
        }
    }

    /// Record a frame's performance metrics
    pub fn record_frame(&mut self, metrics: PerformanceMetrics) {
        self.current_frame += 1;

        if !self.config.enabled {
            return;
        }

        // Add frame time to rolling window
        self.frame_times.push_back(metrics.frame_time);
        if self.frame_times.len() > self.config.sample_window_frames {
            self.frame_times.pop_front();
        }

        // Add metrics to history
        self.metrics_history.push_back(metrics.clone());
        if self.metrics_history.len() > self.config.sample_window_frames {
            self.metrics_history.pop_front();
        }

        // Check for performance alerts
        self.check_performance_alerts(&metrics);

        // Log performance summary at intervals
        if self.current_frame - self.last_log_frame >= self.config.log_interval_frames as u64 {
            self.log_performance_summary();
            self.last_log_frame = self.current_frame;
        }
    }

    /// Record material batching statistics
    pub fn record_batching_stats(&mut self, stats: &BatchingStats, batch_perf: BatchPerformanceStats) {
        if !self.config.enabled {
            return;
        }

        self.batch_stats_history.push_back(batch_perf);
        if self.batch_stats_history.len() > self.config.sample_window_frames {
            self.batch_stats_history.pop_front();
        }

        // Log batching performance
        if self.current_frame % 120 == 0 { // Every 2 seconds at 60fps
            log::info!("🎨 Batching: {} batches, {:.1}% efficiency, {} draw calls saved",
                      stats.batches_created,
                      stats.efficiency_percentage(),
                      stats.draw_calls_saved);
        }
    }

    /// Get current average FPS
    pub fn get_average_fps(&self) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }

        let total_time: Duration = self.frame_times.iter().sum();
        let average_frame_time = total_time.as_secs_f32() / self.frame_times.len() as f32;

        if average_frame_time > 0.0 {
            1.0 / average_frame_time
        } else {
            0.0
        }
    }

    /// Get current frame time statistics
    pub fn get_frame_time_stats(&self) -> (Duration, Duration, Duration) {
        if self.frame_times.is_empty() {
            return (Duration::ZERO, Duration::ZERO, Duration::ZERO);
        }

        let mut sorted_times: Vec<Duration> = self.frame_times.iter().cloned().collect();
        sorted_times.sort();

        let min = sorted_times[0];
        let max = sorted_times[sorted_times.len() - 1];
        let total: Duration = sorted_times.iter().sum();
        let avg = total / sorted_times.len() as u32;

        (min, avg, max)
    }

    /// Get recent performance trends
    pub fn get_performance_trend(&self) -> PerformanceTrend {
        if self.metrics_history.len() < 2 {
            return PerformanceTrend::Stable;
        }

        let recent_window = self.metrics_history.len().min(30); // Last 30 frames
        let recent_metrics: Vec<&PerformanceMetrics> = self.metrics_history
            .iter()
            .rev()
            .take(recent_window)
            .collect();

        if recent_metrics.len() < 2 {
            return PerformanceTrend::Stable;
        }

        // Calculate trend based on FPS
        let first_half_fps: f32 = recent_metrics[recent_window/2..].iter().map(|m| m.fps).sum::<f32>() / (recent_window/2) as f32;
        let second_half_fps: f32 = recent_metrics[..recent_window/2].iter().map(|m| m.fps).sum::<f32>() / (recent_window/2) as f32;

        let fps_change = second_half_fps - first_half_fps;

        if fps_change > 5.0 {
            PerformanceTrend::Improving
        } else if fps_change < -5.0 {
            PerformanceTrend::Degrading
        } else {
            PerformanceTrend::Stable
        }
    }

    /// Check for performance alerts
    fn check_performance_alerts(&mut self, metrics: &PerformanceMetrics) {
        self.alerts.clear();

        let targets = &self.config.performance_targets;

        // Check FPS
        if metrics.fps < targets.min_fps {
            self.alerts.push(PerformanceAlert::LowFrameRate {
                current_fps: metrics.fps,
                target_fps: targets.min_fps,
            });
        }

        // Check frame time
        let frame_time_ms = metrics.frame_time.as_secs_f32() * 1000.0;
        if frame_time_ms > targets.max_frame_time_ms {
            self.alerts.push(PerformanceAlert::HighFrameTime {
                current_ms: frame_time_ms,
                target_ms: targets.max_frame_time_ms,
            });
        }

        // Check draw calls
        if metrics.draw_calls > targets.max_draw_calls {
            self.alerts.push(PerformanceAlert::TooManyDrawCalls {
                current: metrics.draw_calls,
                target: targets.max_draw_calls,
            });
        }

        // Check batching efficiency
        if metrics.batching_efficiency < targets.min_batching_efficiency {
            self.alerts.push(PerformanceAlert::LowBatchingEfficiency {
                current: metrics.batching_efficiency,
                target: targets.min_batching_efficiency,
            });
        }

        // Check memory usage
        if metrics.memory_usage_mb > targets.max_memory_usage_mb {
            self.alerts.push(PerformanceAlert::HighMemoryUsage {
                current_mb: metrics.memory_usage_mb,
                target_mb: targets.max_memory_usage_mb,
            });
        }

        // Suggest optimizations
        if metrics.draw_calls > 50 && metrics.batching_efficiency < 80.0 {
            self.alerts.push(PerformanceAlert::OptimizationOpportunity {
                description: "Consider improving material batching to reduce draw calls".to_string(),
            });
        }
    }

    /// Log performance summary
    fn log_performance_summary(&self) {
        let avg_fps = self.get_average_fps();
        let (min_frame, avg_frame, max_frame) = self.get_frame_time_stats();
        let trend = self.get_performance_trend();

        log::info!("📊 Performance: {:.1} FPS, {:.2}ms frame (min: {:.2}ms, max: {:.2}ms) - {:?}",
                   avg_fps,
                   avg_frame.as_secs_f32() * 1000.0,
                   min_frame.as_secs_f32() * 1000.0,
                   max_frame.as_secs_f32() * 1000.0,
                   trend);

        // Log any active alerts
        for alert in &self.alerts {
            match alert {
                PerformanceAlert::LowFrameRate { current_fps, target_fps } => {
                    log::warn!("⚠️  Low FPS: {:.1} (target: {:.1})", current_fps, target_fps);
                }
                PerformanceAlert::TooManyDrawCalls { current, target } => {
                    log::warn!("⚠️  High draw calls: {} (target: {})", current, target);
                }
                PerformanceAlert::LowBatchingEfficiency { current, target } => {
                    log::warn!("⚠️  Low batching efficiency: {:.1}% (target: {:.1}%)", current, target);
                }
                PerformanceAlert::OptimizationOpportunity { description } => {
                    log::info!("💡 Optimization: {}", description);
                }
                _ => {}
            }
        }
    }

    /// Get current alerts
    pub fn get_alerts(&self) -> &[PerformanceAlert] {
        &self.alerts
    }

    /// Get uptime
    pub fn get_uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get current configuration
    pub fn get_config(&self) -> &MonitorConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: MonitorConfig) {
        self.config = config;
    }

    /// Enable or disable monitoring
    pub fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
        if !enabled {
            self.alerts.clear();
        }
    }

    /// Get batching statistics summary
    pub fn get_batching_summary(&self) -> Option<BatchingSummary> {
        if self.batch_stats_history.is_empty() {
            return None;
        }

        let recent_stats = &self.batch_stats_history[self.batch_stats_history.len() - 1];
        Some(BatchingSummary {
            total_batches: recent_stats.material_distributions.len(),
            average_batch_size: recent_stats.average_batch_size,
            total_vertices_saved: recent_stats.total_vertices_saved,
            total_draw_calls_saved: recent_stats.total_draw_calls_saved,
            overhead_ms: recent_stats.batching_overhead_ms,
            material_distribution: recent_stats.material_distributions.clone(),
        })
    }
}

/// Performance trend analysis
#[derive(Debug, Clone, Copy)]
pub enum PerformanceTrend {
    Improving,
    Stable,
    Degrading,
}

/// Batching performance summary
#[derive(Debug, Clone)]
pub struct BatchingSummary {
    pub total_batches: usize,
    pub average_batch_size: f32,
    pub total_vertices_saved: usize,
    pub total_draw_calls_saved: usize,
    pub overhead_ms: f32,
    pub material_distribution: HashMap<MaterialType, usize>,
}

impl BatchingSummary {
    /// Get efficiency percentage
    pub fn efficiency_percentage(&self) -> f32 {
        if self.total_batches == 0 {
            return 0.0;
        }
        (self.total_draw_calls_saved as f32 / self.total_batches as f32) * 100.0
    }

    /// Format as human-readable string
    pub fn format_summary(&self) -> String {
        format!(
            "Batching: {} batches, avg size {:.1}, {:.1}% efficiency, {} vertices saved, {:.2}ms overhead",
            self.total_batches,
            self.average_batch_size,
            self.efficiency_percentage(),
            self.total_vertices_saved,
            self.overhead_ms
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_monitor_creation() {
        let config = MonitorConfig::default();
        let monitor = PerformanceMonitor::new(config);

        assert_eq!(monitor.get_average_fps(), 0.0);
        assert!(monitor.get_alerts().is_empty());
    }

    #[test]
    fn test_fps_calculation() {
        let mut monitor = PerformanceMonitor::new(MonitorConfig::default());

        // Add some frame times (60 FPS = 16.67ms per frame)
        for _ in 0..60 {
            let metrics = PerformanceMetrics {
                frame_time: Duration::from_millis(16),
                fps: 60.0,
                vertex_count: 1000,
                triangle_count: 500,
                draw_calls: 10,
                batching_efficiency: 80.0,
                memory_usage_mb: 100.0,
                gpu_usage_percent: 50.0,
            };
            monitor.record_frame(metrics);
        }

        let avg_fps = monitor.get_average_fps();
        assert!(avg_fps > 55.0 && avg_fps < 65.0); // Should be around 60 FPS
    }

    #[test]
    fn test_performance_alerts() {
        let mut config = MonitorConfig::default();
        config.performance_targets.min_fps = 60.0;

        let mut monitor = PerformanceMonitor::new(config);

        // Record low FPS
        let low_fps_metrics = PerformanceMetrics {
            frame_time: Duration::from_millis(50), // 20 FPS
            fps: 20.0,
            vertex_count: 1000,
            triangle_count: 500,
            draw_calls: 10,
            batching_efficiency: 80.0,
            memory_usage_mb: 100.0,
            gpu_usage_percent: 50.0,
        };

        monitor.record_frame(low_fps_metrics);

        let alerts = monitor.get_alerts();
        assert!(!alerts.is_empty());

        // Should have low FPS alert
        let has_low_fps_alert = alerts.iter().any(|alert| {
            matches!(alert, PerformanceAlert::LowFrameRate { .. })
        });
        assert!(has_low_fps_alert);
    }

    #[test]
    fn test_batching_summary() {
        let mut material_distribution = HashMap::new();
        material_distribution.insert(MaterialType::Stone, 100);
        material_distribution.insert(MaterialType::Dirt, 75);

        let summary = BatchingSummary {
            total_batches: 10,
            average_batch_size: 17.5,
            total_vertices_saved: 1000,
            total_draw_calls_saved: 8,
            overhead_ms: 0.5,
            material_distribution,
        };

        assert_eq!(summary.efficiency_percentage(), 80.0);
        assert!(summary.format_summary().contains("80.0% efficiency"));
    }
}