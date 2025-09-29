//! Dynamic stat monitoring and performance analytics for player attributes
//! Real-time tracking with Apple Silicon Metal compute optimization

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

use crate::engine::error::RobinResult;
use crate::engine::math::Vec3;
use super::player_attributes::{CoreAttributeType, DerivedStatType, PlayerAttributeManager};

#[cfg(target_os = "macos")]
use super::metal_stats_compute::MetalStatsCompute;

/// Real-time monitoring system for player stats and performance analytics
#[derive(Debug)]
pub struct StatMonitoringSystem {
    /// Historical stat tracking
    stat_history: HashMap<CoreAttributeType, StatHistory>,
    derived_stat_history: HashMap<DerivedStatType, StatHistory>,

    /// Performance monitoring
    performance_tracker: PerformanceTracker,

    /// Real-time analytics
    analytics_engine: AnalyticsEngine,

    /// Event-driven monitoring
    event_tracker: StatEventTracker,

    /// Configuration settings
    config: MonitoringConfig,
}

impl StatMonitoringSystem {
    /// Create new monitoring system with Apple Silicon optimizations
    pub fn new() -> Self {
        Self {
            stat_history: HashMap::new(),
            derived_stat_history: HashMap::new(),
            performance_tracker: PerformanceTracker::new(),
            analytics_engine: AnalyticsEngine::new(),
            event_tracker: StatEventTracker::new(),
            config: MonitoringConfig::default(),
        }
    }

    /// Initialize monitoring system with attribute manager integration
    pub fn initialize(&mut self, attribute_manager: &PlayerAttributeManager) -> RobinResult<()> {
        // Initialize tracking for all core attributes
        for attribute in [
            CoreAttribute::Strength,
            CoreAttribute::Dexterity,
            CoreAttribute::Intelligence,
            CoreAttribute::Vitality,
            CoreAttribute::Willpower,
            CoreAttribute::Charisma,
            CoreAttribute::Focus,
            CoreAttribute::Creativity,
            CoreAttribute::Perception,
            CoreAttribute::Endurance,
            CoreAttribute::Luck,
            CoreAttribute::Resonance,
        ] {
            self.stat_history.insert(attribute, StatHistory::new());
        }

        // Initialize tracking for derived stats
        for stat_type in [
            DerivedStatType::MaxHealth,
            DerivedStatType::MaxStamina,
            DerivedStatType::MaxMana,
            DerivedStatType::CarryCapacity,
            DerivedStatType::MovementSpeed,
            DerivedStatType::AttackSpeed,
            DerivedStatType::CastingSpeed,
            DerivedStatType::CriticalChance,
            DerivedStatType::CriticalDamage,
            DerivedStatType::Accuracy,
            DerivedStatType::Evasion,
            DerivedStatType::ManaRegenRate,
            DerivedStatType::HealthRegenRate,
            DerivedStatType::StaminaRegenRate,
            DerivedStatType::MagicResistance,
            DerivedStatType::PhysicalResistance,
            DerivedStatType::ExperienceGain,
            DerivedStatType::ResourceGatheringSpeed,
            DerivedStatType::CraftingSpeed,
            DerivedStatType::BuildingSpeed,
        ] {
            self.derived_stat_history.insert(stat_type, StatHistory::new());
        }

        println!("📊 Stat monitoring system initialized with {} attributes and {} derived stats",
                 self.stat_history.len(), self.derived_stat_history.len());

        Ok(())
    }

    /// Update monitoring system with current attribute values
    pub fn update(&mut self,
                  attribute_manager: &PlayerAttributeManager,
                  delta_time: f32) -> RobinResult<()> {
        let current_time = Instant::now();

        // Record current attribute values
        self.record_current_stats(attribute_manager, current_time)?;

        // Update performance tracking
        self.performance_tracker.update(attribute_manager, delta_time);

        // Process analytics
        self.analytics_engine.update(&self.stat_history, &self.derived_stat_history, delta_time);

        // Update event tracking
        self.event_tracker.update(delta_time);

        // Cleanup old data if needed
        if self.config.auto_cleanup && current_time.duration_since(self.performance_tracker.last_cleanup) > self.config.cleanup_interval {
            self.cleanup_old_data(current_time);
            self.performance_tracker.last_cleanup = current_time;
        }

        Ok(())
    }

    /// Record current stat values to history
    fn record_current_stats(&mut self,
                            attribute_manager: &PlayerAttributeManager,
                            timestamp: Instant) -> RobinResult<()> {
        // Record core attributes
        for (&attribute, history) in self.stat_history.iter_mut() {
            let value = attribute_manager.get_core_attribute_value(attribute) as f32;
            history.record_value(value, timestamp);
        }

        // Record derived stats
        for (&stat_type, history) in self.derived_stat_history.iter_mut() {
            let value = attribute_manager.get_derived_stat_value(stat_type);
            history.record_value(value, timestamp);
        }

        Ok(())
    }

    /// Get performance analytics report
    pub fn get_performance_report(&self) -> PerformanceReport {
        self.performance_tracker.generate_report()
    }

    /// Get stat trend analysis
    pub fn get_stat_trends(&self) -> StatTrends {
        self.analytics_engine.generate_trends(&self.stat_history, &self.derived_stat_history)
    }

    /// Get real-time stat dashboard data
    pub fn get_dashboard_data(&self) -> StatDashboardData {
        StatDashboardData {
            current_values: self.get_current_values(),
            recent_changes: self.get_recent_changes(),
            performance_metrics: self.performance_tracker.get_current_metrics(),
            trend_indicators: self.analytics_engine.get_trend_indicators(),
            alerts: self.event_tracker.get_active_alerts(),
        }
    }

    /// Record stat change event
    pub fn record_stat_event(&mut self, event: StatEvent) {
        self.event_tracker.record_event(event);
    }

    /// Clean up old data based on retention policy
    fn cleanup_old_data(&mut self, current_time: Instant) {
        let retention_period = self.config.data_retention_period;
        let cutoff_time = current_time - retention_period;

        // Clean stat histories
        for history in self.stat_history.values_mut() {
            history.cleanup_before(cutoff_time);
        }

        for history in self.derived_stat_history.values_mut() {
            history.cleanup_before(cutoff_time);
        }

        // Clean event tracking
        self.event_tracker.cleanup_before(cutoff_time);

        println!("🧹 Cleaned up stat monitoring data older than {:?}", retention_period);
    }

    /// Get current stat values as snapshot
    fn get_current_values(&self) -> HashMap<String, f32> {
        let mut values = HashMap::new();

        // Add core attributes
        for (&attribute, history) in &self.stat_history {
            if let Some(latest) = history.get_latest_value() {
                values.insert(format!("{:?}", attribute), latest);
            }
        }

        // Add derived stats
        for (&stat_type, history) in &self.derived_stat_history {
            if let Some(latest) = history.get_latest_value() {
                values.insert(format!("{:?}", stat_type), latest);
            }
        }

        values
    }

    /// Get recent stat changes
    fn get_recent_changes(&self) -> Vec<StatChange> {
        let mut changes = Vec::new();
        let window = Duration::from_secs(30); // Last 30 seconds

        for (&attribute, history) in &self.stat_history {
            if let Some(change) = history.get_change_in_window(window) {
                changes.push(StatChange {
                    stat_name: format!("{:?}", attribute),
                    old_value: change.old_value,
                    new_value: change.new_value,
                    change_amount: change.change_amount,
                    change_percent: change.change_percent,
                    timestamp: change.timestamp,
                });
            }
        }

        changes
    }
}

/// Historical tracking for individual stats
#[derive(Debug)]
pub struct StatHistory {
    /// Ring buffer of stat values with timestamps
    values: VecDeque<(f32, Instant)>,
    /// Maximum number of entries to keep
    max_entries: usize,
    /// Statistical aggregates for quick access
    aggregates: StatAggregates,
}

impl StatHistory {
    pub fn new() -> Self {
        Self {
            values: VecDeque::with_capacity(1000),
            max_entries: 1000,
            aggregates: StatAggregates::default(),
        }
    }

    /// Record a new value
    pub fn record_value(&mut self, value: f32, timestamp: Instant) {
        self.values.push_back((value, timestamp));

        // Maintain size limit
        if self.values.len() > self.max_entries {
            self.values.pop_front();
        }

        // Update aggregates
        self.aggregates.update(value);
    }

    /// Get the most recent value
    pub fn get_latest_value(&self) -> Option<f32> {
        self.values.back().map(|(value, _)| *value)
    }

    /// Get change within a time window
    pub fn get_change_in_window(&self, window: Duration) -> Option<ValueChange> {
        let now = Instant::now();
        let cutoff = now - window;

        let recent_values: Vec<_> = self.values.iter()
            .filter(|(_, timestamp)| *timestamp >= cutoff)
            .collect();

        if recent_values.len() < 2 {
            return None;
        }

        let (old_value, old_time) = recent_values[0];
        let (new_value, new_time) = recent_values[recent_values.len() - 1];

        let change_amount = new_value - old_value;
        let change_percent = if *old_value != 0.0 {
            (change_amount / old_value) * 100.0
        } else {
            0.0
        };

        Some(ValueChange {
            old_value: *old_value,
            new_value: *new_value,
            change_amount,
            change_percent,
            timestamp: *new_time,
        })
    }

    /// Clean up entries before specified time
    pub fn cleanup_before(&mut self, cutoff_time: Instant) {
        while let Some((_, timestamp)) = self.values.front() {
            if *timestamp < cutoff_time {
                self.values.pop_front();
            } else {
                break;
            }
        }

        // Recalculate aggregates
        self.aggregates = StatAggregates::default();
        for (value, _) in &self.values {
            self.aggregates.update(*value);
        }
    }
}

/// Performance tracking with Apple Silicon optimization monitoring
#[derive(Debug)]
pub struct PerformanceTracker {
    /// Metal compute performance metrics
    #[cfg(target_os = "macos")]
    metal_metrics: MetalPerformanceMetrics,

    /// CPU computation timings
    cpu_timings: VecDeque<Duration>,

    /// Memory usage tracking
    memory_usage: VecDeque<MemoryUsage>,

    /// Calculation throughput metrics
    throughput_metrics: ThroughputMetrics,

    /// Last cleanup time
    last_cleanup: Instant,
}

impl PerformanceTracker {
    pub fn new() -> Self {
        Self {
            #[cfg(target_os = "macos")]
            metal_metrics: MetalPerformanceMetrics::new(),
            cpu_timings: VecDeque::with_capacity(100),
            memory_usage: VecDeque::with_capacity(100),
            throughput_metrics: ThroughputMetrics::new(),
            last_cleanup: Instant::now(),
        }
    }

    /// Update performance tracking
    pub fn update(&mut self, attribute_manager: &PlayerAttributeManager, delta_time: f32) {
        let start_time = Instant::now();

        // Simulate stat calculation timing
        self.measure_calculation_performance(attribute_manager);

        // Record memory usage
        self.record_memory_usage();

        // Update throughput metrics
        self.throughput_metrics.update(delta_time);

        let calculation_time = start_time.elapsed();
        self.cpu_timings.push_back(calculation_time);

        // Maintain size limits
        if self.cpu_timings.len() > 100 {
            self.cpu_timings.pop_front();
        }
        if self.memory_usage.len() > 100 {
            self.memory_usage.pop_front();
        }
    }

    /// Measure calculation performance
    fn measure_calculation_performance(&mut self, _attribute_manager: &PlayerAttributeManager) {
        // This would measure actual calculation times
        // For now, we'll simulate the measurement

        #[cfg(target_os = "macos")]
        {
            // Record Metal compute metrics if available
            self.metal_metrics.record_calculation_cycle();
        }
    }

    /// Record current memory usage
    fn record_memory_usage(&mut self) {
        // This would measure actual memory usage
        // For now, we'll simulate the measurement
        let usage = MemoryUsage {
            total_bytes: 1024 * 1024, // 1MB simulated
            attribute_cache_bytes: 256 * 1024, // 256KB
            history_bytes: 512 * 1024, // 512KB
            timestamp: Instant::now(),
        };

        self.memory_usage.push_back(usage);
    }

    /// Generate performance report
    pub fn generate_report(&self) -> PerformanceReport {
        let avg_cpu_time = if !self.cpu_timings.is_empty() {
            self.cpu_timings.iter().sum::<Duration>() / self.cpu_timings.len() as u32
        } else {
            Duration::ZERO
        };

        let latest_memory = self.memory_usage.back().copied();

        PerformanceReport {
            average_calculation_time: avg_cpu_time,
            memory_usage: latest_memory,
            throughput: self.throughput_metrics.get_current_throughput(),
            #[cfg(target_os = "macos")]
            metal_performance: self.metal_metrics.get_summary(),
            optimization_recommendations: self.generate_optimization_recommendations(),
        }
    }

    /// Get current performance metrics
    pub fn get_current_metrics(&self) -> CurrentPerformanceMetrics {
        CurrentPerformanceMetrics {
            calculation_time_ms: self.cpu_timings.back()
                .map(|d| d.as_secs_f32() * 1000.0)
                .unwrap_or(0.0),
            memory_usage_mb: self.memory_usage.back()
                .map(|m| m.total_bytes as f32 / (1024.0 * 1024.0))
                .unwrap_or(0.0),
            calculations_per_second: self.throughput_metrics.get_current_throughput(),
            apple_silicon_utilization: {
                #[cfg(target_os = "macos")]
                {
                    self.metal_metrics.get_utilization_percent()
                }
                #[cfg(not(target_os = "macos"))]
                {
                    0.0
                }
            },
        }
    }

    /// Generate optimization recommendations
    fn generate_optimization_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Check calculation time
        if let Some(avg_time) = self.cpu_timings.iter().sum::<Duration>().checked_div(self.cpu_timings.len() as u32) {
            if avg_time > Duration::from_millis(10) {
                recommendations.push("Consider enabling Metal compute acceleration for stat calculations".to_string());
            }
        }

        // Check memory usage
        if let Some(latest_memory) = self.memory_usage.back() {
            if latest_memory.total_bytes > 10 * 1024 * 1024 { // 10MB
                recommendations.push("Memory usage is high, consider reducing history retention period".to_string());
            }
        }

        #[cfg(target_os = "macos")]
        {
            if self.metal_metrics.get_utilization_percent() < 50.0 {
                recommendations.push("Apple Silicon Metal compute is underutilized, consider batching more calculations".to_string());
            }
        }

        if recommendations.is_empty() {
            recommendations.push("Performance is optimal".to_string());
        }

        recommendations
    }
}

/// Analytics engine for trend analysis and insights
#[derive(Debug)]
pub struct AnalyticsEngine {
    /// Trend calculators
    trend_calculators: HashMap<String, TrendCalculator>,

    /// Pattern recognition
    pattern_detector: PatternDetector,

    /// Insight generator
    insight_generator: InsightGenerator,
}

impl AnalyticsEngine {
    pub fn new() -> Self {
        Self {
            trend_calculators: HashMap::new(),
            pattern_detector: PatternDetector::new(),
            insight_generator: InsightGenerator::new(),
        }
    }

    /// Update analytics processing
    pub fn update(&mut self,
                  stat_history: &HashMap<CoreAttribute, StatHistory>,
                  derived_stat_history: &HashMap<DerivedStatType, StatHistory>,
                  delta_time: f32) {
        // Update trend calculations
        self.update_trends(stat_history, derived_stat_history);

        // Run pattern detection
        self.pattern_detector.update(stat_history, derived_stat_history, delta_time);

        // Generate insights
        self.insight_generator.update(&self.trend_calculators, &self.pattern_detector);
    }

    /// Update trend calculations
    fn update_trends(&mut self,
                     stat_history: &HashMap<CoreAttribute, StatHistory>,
                     derived_stat_history: &HashMap<DerivedStatType, StatHistory>) {
        // Calculate trends for core attributes
        for (attribute, history) in stat_history {
            let key = format!("{:?}", attribute);
            let calculator = self.trend_calculators.entry(key).or_insert_with(TrendCalculator::new);
            calculator.update(history);
        }

        // Calculate trends for derived stats
        for (stat_type, history) in derived_stat_history {
            let key = format!("{:?}", stat_type);
            let calculator = self.trend_calculators.entry(key).or_insert_with(TrendCalculator::new);
            calculator.update(history);
        }
    }

    /// Generate trend analysis
    pub fn generate_trends(&self,
                          stat_history: &HashMap<CoreAttribute, StatHistory>,
                          derived_stat_history: &HashMap<DerivedStatType, StatHistory>) -> StatTrends {
        let mut trends = HashMap::new();

        for (key, calculator) in &self.trend_calculators {
            trends.insert(key.clone(), calculator.get_trend());
        }

        StatTrends {
            trends,
            insights: self.insight_generator.get_current_insights(),
            patterns: self.pattern_detector.get_detected_patterns(),
        }
    }

    /// Get current trend indicators
    pub fn get_trend_indicators(&self) -> Vec<TrendIndicator> {
        self.trend_calculators.iter()
            .map(|(name, calculator)| TrendIndicator {
                stat_name: name.clone(),
                direction: calculator.get_direction(),
                strength: calculator.get_strength(),
                confidence: calculator.get_confidence(),
            })
            .collect()
    }
}

/// Event tracking system for stat changes
#[derive(Debug)]
pub struct StatEventTracker {
    /// Event history
    events: VecDeque<TimestampedStatEvent>,

    /// Active alerts
    active_alerts: Vec<StatAlert>,

    /// Event processing rules
    event_rules: Vec<EventRule>,
}

impl StatEventTracker {
    pub fn new() -> Self {
        Self {
            events: VecDeque::with_capacity(1000),
            active_alerts: Vec::new(),
            event_rules: Self::create_default_rules(),
        }
    }

    /// Record new stat event
    pub fn record_event(&mut self, event: StatEvent) {
        let timestamped_event = TimestampedStatEvent {
            event,
            timestamp: Instant::now(),
        };

        self.events.push_back(timestamped_event);

        // Maintain size limit
        if self.events.len() > 1000 {
            self.events.pop_front();
        }

        // Process event against rules
        self.process_event_rules(&timestamped_event);
    }

    /// Update event tracking
    pub fn update(&mut self, delta_time: f32) {
        // Update active alerts
        self.update_alerts(delta_time);
    }

    /// Get active alerts
    pub fn get_active_alerts(&self) -> Vec<StatAlert> {
        self.active_alerts.clone()
    }

    /// Clean up events before specified time
    pub fn cleanup_before(&mut self, cutoff_time: Instant) {
        while let Some(event) = self.events.front() {
            if event.timestamp < cutoff_time {
                self.events.pop_front();
            } else {
                break;
            }
        }
    }

    /// Create default event processing rules
    fn create_default_rules() -> Vec<EventRule> {
        vec![
            EventRule {
                name: "Rapid stat increase".to_string(),
                condition: EventCondition::RapidIncrease { threshold: 50.0, time_window: Duration::from_secs(10) },
                action: EventAction::CreateAlert {
                    message: "Stat increased rapidly".to_string(),
                    severity: AlertSeverity::Info
                },
            },
            EventRule {
                name: "Stat anomaly detection".to_string(),
                condition: EventCondition::AnomalousValue { deviation_threshold: 3.0 },
                action: EventAction::CreateAlert {
                    message: "Anomalous stat value detected".to_string(),
                    severity: AlertSeverity::Warning
                },
            },
        ]
    }

    /// Process event against rules
    fn process_event_rules(&mut self, event: &TimestampedStatEvent) {
        for rule in &self.event_rules {
            if rule.matches(event, &self.events) {
                match &rule.action {
                    EventAction::CreateAlert { message, severity } => {
                        self.create_alert(message.clone(), *severity);
                    }
                }
            }
        }
    }

    /// Create new alert
    fn create_alert(&mut self, message: String, severity: AlertSeverity) {
        let alert = StatAlert {
            message,
            severity,
            timestamp: Instant::now(),
            duration: Duration::from_secs(30), // 30 second default duration
        };

        self.active_alerts.push(alert);
    }

    /// Update active alerts (remove expired ones)
    fn update_alerts(&mut self, _delta_time: f32) {
        let now = Instant::now();
        self.active_alerts.retain(|alert| {
            now.duration_since(alert.timestamp) < alert.duration
        });
    }
}

// Supporting data structures

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoreAttribute {
    Strength,
    Dexterity,
    Intelligence,
    Vitality,
    Willpower,
    Charisma,
    Focus,
    Creativity,
    Perception,
    Endurance,
    Luck,
    Resonance,
}

// Note: DerivedStatType is imported from super::player_attributes

#[derive(Debug, Default)]
pub struct StatAggregates {
    pub count: usize,
    pub sum: f32,
    pub min: f32,
    pub max: f32,
    pub mean: f32,
}

impl StatAggregates {
    pub fn update(&mut self, value: f32) {
        if self.count == 0 {
            self.min = value;
            self.max = value;
        } else {
            self.min = self.min.min(value);
            self.max = self.max.max(value);
        }

        self.count += 1;
        self.sum += value;
        self.mean = self.sum / self.count as f32;
    }
}

#[derive(Debug, Clone)]
pub struct ValueChange {
    pub old_value: f32,
    pub new_value: f32,
    pub change_amount: f32,
    pub change_percent: f32,
    pub timestamp: Instant,
}

#[cfg(target_os = "macos")]
#[derive(Debug)]
pub struct MetalPerformanceMetrics {
    calculation_cycles: u64,
    total_calculation_time: Duration,
    utilization_samples: VecDeque<f32>,
}

#[cfg(target_os = "macos")]
impl MetalPerformanceMetrics {
    pub fn new() -> Self {
        Self {
            calculation_cycles: 0,
            total_calculation_time: Duration::ZERO,
            utilization_samples: VecDeque::with_capacity(100),
        }
    }

    pub fn record_calculation_cycle(&mut self) {
        self.calculation_cycles += 1;
        // Would record actual Metal performance metrics
        self.utilization_samples.push_back(75.0); // Simulated 75% utilization

        if self.utilization_samples.len() > 100 {
            self.utilization_samples.pop_front();
        }
    }

    pub fn get_utilization_percent(&self) -> f32 {
        if self.utilization_samples.is_empty() {
            0.0
        } else {
            self.utilization_samples.iter().sum::<f32>() / self.utilization_samples.len() as f32
        }
    }

    pub fn get_summary(&self) -> MetalPerformanceSummary {
        MetalPerformanceSummary {
            total_cycles: self.calculation_cycles,
            average_utilization: self.get_utilization_percent(),
            total_computation_time: self.total_calculation_time,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryUsage {
    pub total_bytes: usize,
    pub attribute_cache_bytes: usize,
    pub history_bytes: usize,
    pub timestamp: Instant,
}

#[derive(Debug)]
pub struct ThroughputMetrics {
    calculations_this_second: u32,
    last_reset: Instant,
    running_average: f32,
}

impl ThroughputMetrics {
    pub fn new() -> Self {
        Self {
            calculations_this_second: 0,
            last_reset: Instant::now(),
            running_average: 0.0,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        self.calculations_this_second += 1;

        let now = Instant::now();
        if now.duration_since(self.last_reset) >= Duration::from_secs(1) {
            // Update running average with exponential moving average
            let alpha = 0.1; // Smoothing factor
            self.running_average = alpha * self.calculations_this_second as f32 + (1.0 - alpha) * self.running_average;

            self.calculations_this_second = 0;
            self.last_reset = now;
        }
    }

    pub fn get_current_throughput(&self) -> f32 {
        self.running_average
    }
}

#[derive(Debug)]
pub struct PerformanceReport {
    pub average_calculation_time: Duration,
    pub memory_usage: Option<MemoryUsage>,
    pub throughput: f32,
    #[cfg(target_os = "macos")]
    pub metal_performance: MetalPerformanceSummary,
    pub optimization_recommendations: Vec<String>,
}

#[cfg(target_os = "macos")]
#[derive(Debug)]
pub struct MetalPerformanceSummary {
    pub total_cycles: u64,
    pub average_utilization: f32,
    pub total_computation_time: Duration,
}

#[derive(Debug)]
pub struct CurrentPerformanceMetrics {
    pub calculation_time_ms: f32,
    pub memory_usage_mb: f32,
    pub calculations_per_second: f32,
    pub apple_silicon_utilization: f32,
}

#[derive(Debug)]
pub struct TrendCalculator {
    values: VecDeque<f32>,
    slope: f32,
    confidence: f32,
}

impl TrendCalculator {
    pub fn new() -> Self {
        Self {
            values: VecDeque::with_capacity(50),
            slope: 0.0,
            confidence: 0.0,
        }
    }

    pub fn update(&mut self, history: &StatHistory) {
        if let Some(latest) = history.get_latest_value() {
            self.values.push_back(latest);

            if self.values.len() > 50 {
                self.values.pop_front();
            }

            // Calculate linear regression slope
            self.calculate_trend();
        }
    }

    fn calculate_trend(&mut self) {
        if self.values.len() < 2 {
            return;
        }

        let n = self.values.len() as f32;
        let x_values: Vec<f32> = (0..self.values.len()).map(|i| i as f32).collect();
        let y_values: Vec<f32> = self.values.iter().copied().collect();

        let x_mean = x_values.iter().sum::<f32>() / n;
        let y_mean = y_values.iter().sum::<f32>() / n;

        let numerator: f32 = x_values.iter().zip(y_values.iter())
            .map(|(x, y)| (x - x_mean) * (y - y_mean))
            .sum();

        let denominator: f32 = x_values.iter()
            .map(|x| (x - x_mean).powi(2))
            .sum();

        if denominator != 0.0 {
            self.slope = numerator / denominator;
            // Simple confidence based on R-squared
            self.confidence = (numerator.powi(2) / (denominator * y_values.iter().map(|y| (y - y_mean).powi(2)).sum::<f32>())).min(1.0);
        }
    }

    pub fn get_trend(&self) -> TrendDirection {
        if self.slope.abs() < 0.01 {
            TrendDirection::Stable
        } else if self.slope > 0.0 {
            TrendDirection::Increasing
        } else {
            TrendDirection::Decreasing
        }
    }

    pub fn get_direction(&self) -> TrendDirection {
        self.get_trend()
    }

    pub fn get_strength(&self) -> f32 {
        self.slope.abs()
    }

    pub fn get_confidence(&self) -> f32 {
        self.confidence
    }
}

#[derive(Debug)]
pub struct PatternDetector {
    detected_patterns: Vec<StatPattern>,
}

impl PatternDetector {
    pub fn new() -> Self {
        Self {
            detected_patterns: Vec::new(),
        }
    }

    pub fn update(&mut self,
                  _stat_history: &HashMap<CoreAttribute, StatHistory>,
                  _derived_stat_history: &HashMap<DerivedStatType, StatHistory>,
                  _delta_time: f32) {
        // Pattern detection would be implemented here
        // For now, we'll simulate some detected patterns
        self.detected_patterns.clear();
        self.detected_patterns.push(StatPattern {
            name: "Strength-Endurance Correlation".to_string(),
            description: "Strength and Endurance tend to increase together".to_string(),
            confidence: 0.85,
        });
    }

    pub fn get_detected_patterns(&self) -> Vec<StatPattern> {
        self.detected_patterns.clone()
    }
}

#[derive(Debug)]
pub struct InsightGenerator {
    current_insights: Vec<StatInsight>,
}

impl InsightGenerator {
    pub fn new() -> Self {
        Self {
            current_insights: Vec::new(),
        }
    }

    pub fn update(&mut self,
                  _trend_calculators: &HashMap<String, TrendCalculator>,
                  _pattern_detector: &PatternDetector) {
        // Insight generation would be implemented here
        self.current_insights.clear();
        self.current_insights.push(StatInsight {
            title: "Balanced Growth".to_string(),
            description: "Your attributes are developing in a balanced way".to_string(),
            impact: InsightImpact::Positive,
            recommendations: vec!["Continue your current training regimen".to_string()],
        });
    }

    pub fn get_current_insights(&self) -> Vec<StatInsight> {
        self.current_insights.clone()
    }
}

// Data structures for monitoring output

#[derive(Debug)]
pub struct StatDashboardData {
    pub current_values: HashMap<String, f32>,
    pub recent_changes: Vec<StatChange>,
    pub performance_metrics: CurrentPerformanceMetrics,
    pub trend_indicators: Vec<TrendIndicator>,
    pub alerts: Vec<StatAlert>,
}

#[derive(Debug, Clone)]
pub struct StatChange {
    pub stat_name: String,
    pub old_value: f32,
    pub new_value: f32,
    pub change_amount: f32,
    pub change_percent: f32,
    pub timestamp: Instant,
}

#[derive(Debug)]
pub struct StatTrends {
    pub trends: HashMap<String, TrendDirection>,
    pub insights: Vec<StatInsight>,
    pub patterns: Vec<StatPattern>,
}

#[derive(Debug, Clone, Copy)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
}

#[derive(Debug)]
pub struct TrendIndicator {
    pub stat_name: String,
    pub direction: TrendDirection,
    pub strength: f32,
    pub confidence: f32,
}

#[derive(Debug, Clone)]
pub struct StatPattern {
    pub name: String,
    pub description: String,
    pub confidence: f32,
}

#[derive(Debug, Clone)]
pub struct StatInsight {
    pub title: String,
    pub description: String,
    pub impact: InsightImpact,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum InsightImpact {
    Positive,
    Neutral,
    Negative,
}

// Event system data structures

#[derive(Debug, Clone)]
pub struct StatEvent {
    pub stat_name: String,
    pub event_type: StatEventType,
    pub old_value: f32,
    pub new_value: f32,
    pub source: EventSource,
}

#[derive(Debug, Clone)]
pub enum StatEventType {
    AttributeIncrease,
    AttributeDecrease,
    EquipmentChange,
    BuffApplied,
    BuffRemoved,
    LevelUp,
}

#[derive(Debug, Clone)]
pub enum EventSource {
    Equipment,
    Buff,
    LevelUp,
    Training,
    Other(String),
}

#[derive(Debug)]
pub struct TimestampedStatEvent {
    pub event: StatEvent,
    pub timestamp: Instant,
}

#[derive(Debug, Clone)]
pub struct StatAlert {
    pub message: String,
    pub severity: AlertSeverity,
    pub timestamp: Instant,
    pub duration: Duration,
}

#[derive(Debug, Clone, Copy)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug)]
pub struct EventRule {
    pub name: String,
    pub condition: EventCondition,
    pub action: EventAction,
}

impl EventRule {
    pub fn matches(&self, _event: &TimestampedStatEvent, _history: &VecDeque<TimestampedStatEvent>) -> bool {
        // Rule matching logic would be implemented here
        false // For now, always false
    }
}

#[derive(Debug)]
pub enum EventCondition {
    RapidIncrease { threshold: f32, time_window: Duration },
    AnomalousValue { deviation_threshold: f32 },
}

#[derive(Debug)]
pub enum EventAction {
    CreateAlert { message: String, severity: AlertSeverity },
}

#[derive(Debug)]
pub struct MonitoringConfig {
    pub auto_cleanup: bool,
    pub cleanup_interval: Duration,
    pub data_retention_period: Duration,
    pub performance_tracking_enabled: bool,
    pub analytics_enabled: bool,
    pub event_tracking_enabled: bool,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            auto_cleanup: true,
            cleanup_interval: Duration::from_secs(300), // 5 minutes
            data_retention_period: Duration::from_secs(3600), // 1 hour
            performance_tracking_enabled: true,
            analytics_enabled: true,
            event_tracking_enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stat_monitoring_creation() {
        let monitoring = StatMonitoringSystem::new();
        assert!(monitoring.stat_history.is_empty());
        assert!(monitoring.derived_stat_history.is_empty());
    }

    #[test]
    fn test_stat_history_tracking() {
        let mut history = StatHistory::new();
        let timestamp = Instant::now();

        history.record_value(100.0, timestamp);
        assert_eq!(history.get_latest_value(), Some(100.0));

        history.record_value(110.0, timestamp);
        assert_eq!(history.get_latest_value(), Some(110.0));
    }

    #[test]
    fn test_performance_tracker() {
        let tracker = PerformanceTracker::new();
        assert_eq!(tracker.cpu_timings.len(), 0);
        assert_eq!(tracker.memory_usage.len(), 0);
    }

    #[test]
    fn test_trend_calculator() {
        let mut calculator = TrendCalculator::new();

        // Add increasing values
        for i in 0..10 {
            calculator.values.push_back(i as f32);
        }

        calculator.calculate_trend();
        assert!(matches!(calculator.get_trend(), TrendDirection::Increasing));
    }

    #[test]
    fn test_event_tracking() {
        let mut tracker = StatEventTracker::new();

        let event = StatEvent {
            stat_name: "Strength".to_string(),
            event_type: StatEventType::AttributeIncrease,
            old_value: 10.0,
            new_value: 11.0,
            source: EventSource::Training,
        };

        tracker.record_event(event);
        assert_eq!(tracker.events.len(), 1);
    }
}