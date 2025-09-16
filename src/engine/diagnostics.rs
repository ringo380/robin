use crate::engine::{
    error::{RobinError, RobinResult},
    logging::{PerformanceMetrics, PerformanceStats},
};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// Comprehensive diagnostics system for Robin Engine
pub struct DiagnosticsManager {
    frame_timings: Arc<Mutex<VecDeque<FrameTiming>>>,
    performance_counters: Arc<Mutex<HashMap<String, PerformanceCounter>>>,
    memory_snapshots: Arc<Mutex<VecDeque<MemorySnapshot>>>,
    error_log: Arc<Mutex<VecDeque<ErrorRecord>>>,
    profiler: ProfilerData,
    config: DiagnosticsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsConfig {
    pub enabled: bool,
    pub max_frame_samples: usize,
    pub max_error_records: usize,
    pub max_memory_snapshots: usize,
    pub performance_sampling_rate: f64,
    pub auto_generate_reports: bool,
    pub report_interval: Duration,
    pub memory_tracking: bool,
    pub detailed_profiling: bool,
}

impl Default for DiagnosticsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_frame_samples: 1000,
            max_error_records: 500,
            max_memory_snapshots: 100,
            performance_sampling_rate: 1.0,
            auto_generate_reports: true,
            report_interval: Duration::from_secs(300), // 5 minutes
            memory_tracking: true,
            detailed_profiling: false,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FrameTiming {
    pub timestamp: DateTime<Utc>,
    pub frame_duration: Duration,
    pub update_duration: Duration,
    pub render_duration: Duration,
    pub physics_duration: Duration,
    pub audio_duration: Duration,
    pub ui_duration: Duration,
    pub other_duration: Duration,
    pub fps: f64,
    pub frame_number: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PerformanceCounter {
    pub name: String,
    pub value: f64,
    pub last_updated: DateTime<Utc>,
    pub history: VecDeque<(DateTime<Utc>, f64)>,
    pub min_value: f64,
    pub max_value: f64,
    pub average: f64,
    pub samples: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemorySnapshot {
    pub timestamp: DateTime<Utc>,
    pub total_allocated: usize,
    pub texture_memory: usize,
    pub audio_memory: usize,
    pub mesh_memory: usize,
    pub scene_memory: usize,
    pub ui_memory: usize,
    pub other_memory: usize,
    pub gc_collections: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorRecord {
    pub timestamp: DateTime<Utc>,
    pub error: String,
    pub error_type: String,
    pub severity: String,
    pub context: Option<String>,
    pub recovery_attempted: bool,
    pub recovery_successful: Option<bool>,
}

#[derive(Debug)]
struct ProfilerData {
    active_scopes: Arc<Mutex<Vec<ProfileScope>>>,
    completed_scopes: Arc<Mutex<Vec<CompletedScope>>>,
}

#[derive(Debug, Clone)]
struct ProfileScope {
    name: String,
    start_time: Instant,
    parent: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompletedScope {
    pub name: String,
    pub duration: Duration,
    pub start_time: DateTime<Utc>,
    pub parent: Option<String>,
    pub thread_id: String,
}

impl DiagnosticsManager {
    pub fn new(config: DiagnosticsConfig) -> Self {
        Self {
            frame_timings: Arc::new(Mutex::new(VecDeque::with_capacity(config.max_frame_samples))),
            performance_counters: Arc::new(Mutex::new(HashMap::new())),
            memory_snapshots: Arc::new(Mutex::new(VecDeque::with_capacity(config.max_memory_snapshots))),
            error_log: Arc::new(Mutex::new(VecDeque::with_capacity(config.max_error_records))),
            profiler: ProfilerData {
                active_scopes: Arc::new(Mutex::new(Vec::new())),
                completed_scopes: Arc::new(Mutex::new(Vec::new())),
            },
            config,
        }
    }

    /// Start a new frame timing measurement
    pub fn begin_frame(&self, frame_number: u64) -> FrameTimer {
        if !self.config.enabled {
            return FrameTimer::disabled();
        }

        FrameTimer::new(frame_number, self.frame_timings.clone(), self.config.max_frame_samples)
    }

    /// Record a performance counter value
    pub fn record_counter(&self, name: &str, value: f64) {
        if !self.config.enabled {
            return;
        }

        let mut counters = self.performance_counters.lock().unwrap();
        let counter = counters.entry(name.to_string()).or_insert_with(|| {
            PerformanceCounter {
                name: name.to_string(),
                value: 0.0,
                last_updated: Utc::now(),
                history: VecDeque::with_capacity(100),
                min_value: f64::INFINITY,
                max_value: f64::NEG_INFINITY,
                average: 0.0,
                samples: 0,
            }
        });

        let timestamp = Utc::now();
        counter.value = value;
        counter.last_updated = timestamp;
        counter.history.push_back((timestamp, value));

        // Maintain history size
        if counter.history.len() > 100 {
            counter.history.pop_front();
        }

        // Update statistics
        counter.min_value = counter.min_value.min(value);
        counter.max_value = counter.max_value.max(value);
        counter.samples += 1;
        
        // Compute rolling average
        let recent_values: Vec<f64> = counter.history.iter().map(|(_, v)| *v).collect();
        counter.average = recent_values.iter().sum::<f64>() / recent_values.len() as f64;
    }

    /// Take a memory snapshot
    pub fn snapshot_memory(&self) {
        if !self.config.enabled || !self.config.memory_tracking {
            return;
        }

        let snapshot = MemorySnapshot {
            timestamp: Utc::now(),
            total_allocated: get_total_memory_usage(),
            texture_memory: get_texture_memory_usage(),
            audio_memory: get_audio_memory_usage(),
            mesh_memory: get_mesh_memory_usage(),
            scene_memory: get_scene_memory_usage(),
            ui_memory: get_ui_memory_usage(),
            other_memory: get_other_memory_usage(),
            gc_collections: get_gc_collections(),
        };

        let mut snapshots = self.memory_snapshots.lock().unwrap();
        snapshots.push_back(snapshot);

        // Maintain size limit
        if snapshots.len() > self.config.max_memory_snapshots {
            snapshots.pop_front();
        }
    }

    /// Log an error with context
    pub fn log_error(&self, error: &RobinError, context: Option<String>) {
        if !self.config.enabled {
            return;
        }

        let error_record = ErrorRecord {
            timestamp: Utc::now(),
            error: error.to_string(),
            error_type: format!("{:?}", error).split('(').next().unwrap_or("Unknown").to_string(),
            severity: classify_error_severity(error),
            context,
            recovery_attempted: false,
            recovery_successful: None,
        };

        let mut error_log = self.error_log.lock().unwrap();
        error_log.push_back(error_record);

        // Maintain size limit
        if error_log.len() > self.config.max_error_records {
            error_log.pop_front();
        }
    }

    /// Start profiling a scope
    pub fn begin_scope(&self, name: &str) -> ProfileGuard {
        if !self.config.enabled || !self.config.detailed_profiling {
            return ProfileGuard::disabled();
        }

        let scope = ProfileScope {
            name: name.to_string(),
            start_time: Instant::now(),
            parent: get_current_scope_name(&self.profiler.active_scopes),
        };

        {
            let mut active_scopes = self.profiler.active_scopes.lock().unwrap();
            active_scopes.push(scope.clone());
        }

        ProfileGuard::new(scope, self.profiler.active_scopes.clone(), self.profiler.completed_scopes.clone())
    }

    /// Generate a comprehensive diagnostics report
    pub fn generate_report(&self) -> DiagnosticsReport {
        let frame_timings = self.frame_timings.lock().unwrap().clone();
        let performance_counters = self.performance_counters.lock().unwrap().clone();
        let memory_snapshots = self.memory_snapshots.lock().unwrap().clone();
        let error_log = self.error_log.lock().unwrap().clone();
        let completed_scopes = self.profiler.completed_scopes.lock().unwrap().clone();

        // Calculate frame statistics
        let frame_stats = if frame_timings.is_empty() {
            None
        } else {
            let durations: Vec<f64> = frame_timings.iter()
                .map(|ft| ft.frame_duration.as_secs_f64() * 1000.0)
                .collect();

            let avg_frame_time = durations.iter().sum::<f64>() / durations.len() as f64;
            let min_frame_time = durations.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max_frame_time = durations.iter().fold(0.0f64, |a, &b| a.max(b));
            let avg_fps = if avg_frame_time > 0.0 { 1000.0 / avg_frame_time } else { 0.0 };

            Some(FrameStats {
                sample_count: durations.len(),
                avg_frame_time,
                min_frame_time,
                max_frame_time,
                avg_fps,
                frame_time_variance: calculate_variance(&durations),
            })
        };

        // Calculate memory statistics
        let memory_stats = if memory_snapshots.is_empty() {
            None
        } else {
            let current_memory = memory_snapshots.back().cloned();
            let peak_memory = memory_snapshots.iter()
                .max_by_key(|m| m.total_allocated)
                .cloned();

            Some(MemoryStats {
                current_memory,
                peak_memory,
                sample_count: memory_snapshots.len(),
            })
        };

        // Categorize errors
        let error_summary = categorize_errors(&error_log);

        DiagnosticsReport {
            timestamp: Utc::now(),
            frame_stats,
            memory_stats,
            performance_counters,
            error_summary,
            profiling_data: completed_scopes,
            system_info: collect_system_info(),
        }
    }

    /// Get current performance metrics for real-time monitoring
    pub fn get_current_metrics(&self) -> CurrentMetrics {
        let frame_timings = self.frame_timings.lock().unwrap();
        let performance_counters = self.performance_counters.lock().unwrap();
        let memory_snapshots = self.memory_snapshots.lock().unwrap();

        let current_fps = frame_timings.back()
            .map(|ft| ft.fps)
            .unwrap_or(0.0);

        let current_frame_time = frame_timings.back()
            .map(|ft| ft.frame_duration.as_secs_f64() * 1000.0)
            .unwrap_or(0.0);

        let current_memory = memory_snapshots.back()
            .map(|ms| ms.total_allocated)
            .unwrap_or(0);

        let active_errors = self.error_log.lock().unwrap().len();

        CurrentMetrics {
            fps: current_fps,
            frame_time: current_frame_time,
            memory_usage: current_memory,
            active_errors,
            counter_values: performance_counters.iter()
                .map(|(name, counter)| (name.clone(), counter.value))
                .collect(),
        }
    }
}

/// Frame timing helper
pub struct FrameTimer {
    frame_number: u64,
    start_time: Instant,
    frame_timings: Option<Arc<Mutex<VecDeque<FrameTiming>>>>,
    max_samples: usize,
    update_start: Option<Instant>,
    render_start: Option<Instant>,
    physics_start: Option<Instant>,
    audio_start: Option<Instant>,
    ui_start: Option<Instant>,
    durations: HashMap<String, Duration>,
}

impl FrameTimer {
    fn new(frame_number: u64, frame_timings: Arc<Mutex<VecDeque<FrameTiming>>>, max_samples: usize) -> Self {
        Self {
            frame_number,
            start_time: Instant::now(),
            frame_timings: Some(frame_timings),
            max_samples,
            update_start: None,
            render_start: None,
            physics_start: None,
            audio_start: None,
            ui_start: None,
            durations: HashMap::new(),
        }
    }

    fn disabled() -> Self {
        Self {
            frame_number: 0,
            start_time: Instant::now(),
            frame_timings: None,
            max_samples: 0,
            update_start: None,
            render_start: None,
            physics_start: None,
            audio_start: None,
            ui_start: None,
            durations: HashMap::new(),
        }
    }

    pub fn begin_update(&mut self) {
        self.update_start = Some(Instant::now());
    }

    pub fn end_update(&mut self) {
        if let Some(start) = self.update_start.take() {
            self.durations.insert("update".to_string(), start.elapsed());
        }
    }

    pub fn begin_render(&mut self) {
        self.render_start = Some(Instant::now());
    }

    pub fn end_render(&mut self) {
        if let Some(start) = self.render_start.take() {
            self.durations.insert("render".to_string(), start.elapsed());
        }
    }

    pub fn begin_physics(&mut self) {
        self.physics_start = Some(Instant::now());
    }

    pub fn end_physics(&mut self) {
        if let Some(start) = self.physics_start.take() {
            self.durations.insert("physics".to_string(), start.elapsed());
        }
    }
}

impl Drop for FrameTimer {
    fn drop(&mut self) {
        if let Some(ref frame_timings) = self.frame_timings {
            let total_duration = self.start_time.elapsed();
            let fps = if total_duration.as_secs_f64() > 0.0 {
                1.0 / total_duration.as_secs_f64()
            } else {
                0.0
            };

            let timing = FrameTiming {
                timestamp: Utc::now(),
                frame_duration: total_duration,
                update_duration: self.durations.get("update").copied().unwrap_or_default(),
                render_duration: self.durations.get("render").copied().unwrap_or_default(),
                physics_duration: self.durations.get("physics").copied().unwrap_or_default(),
                audio_duration: self.durations.get("audio").copied().unwrap_or_default(),
                ui_duration: self.durations.get("ui").copied().unwrap_or_default(),
                other_duration: Duration::default(),
                fps,
                frame_number: self.frame_number,
            };

            let mut timings = frame_timings.lock().unwrap();
            timings.push_back(timing);

            if timings.len() > self.max_samples {
                timings.pop_front();
            }
        }
    }
}

/// Profiling scope guard
pub struct ProfileGuard {
    scope: Option<ProfileScope>,
    active_scopes: Option<Arc<Mutex<Vec<ProfileScope>>>>,
    completed_scopes: Option<Arc<Mutex<Vec<CompletedScope>>>>,
}

impl ProfileGuard {
    fn new(
        scope: ProfileScope,
        active_scopes: Arc<Mutex<Vec<ProfileScope>>>,
        completed_scopes: Arc<Mutex<Vec<CompletedScope>>>,
    ) -> Self {
        Self {
            scope: Some(scope),
            active_scopes: Some(active_scopes),
            completed_scopes: Some(completed_scopes),
        }
    }

    fn disabled() -> Self {
        Self {
            scope: None,
            active_scopes: None,
            completed_scopes: None,
        }
    }
}

impl Drop for ProfileGuard {
    fn drop(&mut self) {
        if let (Some(scope), Some(active_scopes), Some(completed_scopes)) = (
            self.scope.take(),
            &self.active_scopes,
            &self.completed_scopes,
        ) {
            let duration = scope.start_time.elapsed();
            
            // Remove from active scopes
            if let Ok(mut active) = active_scopes.lock() {
                if let Some(pos) = active.iter().position(|s| s.name == scope.name) {
                    active.remove(pos);
                }
            }

            // Add to completed scopes
            let completed_scope = CompletedScope {
                name: scope.name,
                duration,
                start_time: Utc::now() - chrono::Duration::from_std(duration).unwrap_or_default(),
                parent: scope.parent,
                thread_id: format!("{:?}", std::thread::current().id()),
            };

            if let Ok(mut completed) = completed_scopes.lock() {
                completed.push(completed_scope);
                
                // Keep only the last 1000 scopes
                if completed.len() > 1000 {
                    let len = completed.len();
                    completed.drain(0..len - 1000);
                }
            }
        }
    }
}

// Report structures
#[derive(Debug, Serialize)]
pub struct DiagnosticsReport {
    pub timestamp: DateTime<Utc>,
    pub frame_stats: Option<FrameStats>,
    pub memory_stats: Option<MemoryStats>,
    pub performance_counters: HashMap<String, PerformanceCounter>,
    pub error_summary: ErrorSummary,
    pub profiling_data: Vec<CompletedScope>,
    pub system_info: SystemInfo,
}

#[derive(Debug, Serialize)]
pub struct FrameStats {
    pub sample_count: usize,
    pub avg_frame_time: f64,
    pub min_frame_time: f64,
    pub max_frame_time: f64,
    pub avg_fps: f64,
    pub frame_time_variance: f64,
}

#[derive(Debug, Serialize)]
pub struct MemoryStats {
    pub current_memory: Option<MemorySnapshot>,
    pub peak_memory: Option<MemorySnapshot>,
    pub sample_count: usize,
}

#[derive(Debug, Serialize)]
pub struct ErrorSummary {
    pub total_errors: usize,
    pub error_types: HashMap<String, usize>,
    pub recent_errors: Vec<ErrorRecord>,
}

#[derive(Debug, Serialize)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub cpu_count: usize,
    pub engine_version: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CurrentMetrics {
    pub fps: f64,
    pub frame_time: f64,
    pub memory_usage: usize,
    pub active_errors: usize,
    pub counter_values: HashMap<String, f64>,
}

// Helper functions
fn classify_error_severity(error: &RobinError) -> String {
    match error {
        RobinError::InternalError(_) | RobinError::InitializationError { .. } => "Critical".to_string(),
        RobinError::GraphicsInitError(_) | RobinError::AudioInitError(_) => "High".to_string(),
        RobinError::AssetNotFound { .. } | RobinError::SceneNotFound(_) => "Medium".to_string(),
        _ => "Low".to_string(),
    }
}

fn get_current_scope_name(active_scopes: &Arc<Mutex<Vec<ProfileScope>>>) -> Option<String> {
    active_scopes.lock().ok()?.last().map(|s| s.name.clone())
}

fn categorize_errors(error_log: &VecDeque<ErrorRecord>) -> ErrorSummary {
    let mut error_types = HashMap::new();
    
    for error in error_log {
        *error_types.entry(error.error_type.clone()).or_insert(0) += 1;
    }

    let recent_errors = error_log.iter()
        .rev()
        .take(10)
        .cloned()
        .collect();

    ErrorSummary {
        total_errors: error_log.len(),
        error_types,
        recent_errors,
    }
}

fn calculate_variance(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }

    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values.iter()
        .map(|v| (v - mean).powi(2))
        .sum::<f64>() / values.len() as f64;
    
    variance
}

fn collect_system_info() -> SystemInfo {
    SystemInfo {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        cpu_count: num_cpus::get(),
        engine_version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

// Placeholder functions for memory tracking
fn get_total_memory_usage() -> usize { 0 }
fn get_texture_memory_usage() -> usize { 0 }
fn get_audio_memory_usage() -> usize { 0 }
fn get_mesh_memory_usage() -> usize { 0 }
fn get_scene_memory_usage() -> usize { 0 }
fn get_ui_memory_usage() -> usize { 0 }
fn get_other_memory_usage() -> usize { 0 }
fn get_gc_collections() -> u32 { 0 }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostics_creation() {
        let config = DiagnosticsConfig::default();
        let diagnostics = DiagnosticsManager::new(config);
        
        let metrics = diagnostics.get_current_metrics();
        assert_eq!(metrics.fps, 0.0);
    }

    #[test]
    fn test_frame_timer() {
        let config = DiagnosticsConfig::default();
        let diagnostics = DiagnosticsManager::new(config);
        
        let mut timer = diagnostics.begin_frame(1);
        timer.begin_update();
        std::thread::sleep(Duration::from_millis(1));
        timer.end_update();
        
        // Timer automatically completes when dropped
        drop(timer);
        
        let metrics = diagnostics.get_current_metrics();
        assert!(metrics.frame_time > 0.0);
    }

    #[test]
    fn test_performance_counter() {
        let config = DiagnosticsConfig::default();
        let diagnostics = DiagnosticsManager::new(config);
        
        diagnostics.record_counter("test_counter", 42.0);
        diagnostics.record_counter("test_counter", 43.0);
        
        let metrics = diagnostics.get_current_metrics();
        assert_eq!(metrics.counter_values.get("test_counter"), Some(&43.0));
    }
}