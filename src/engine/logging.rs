use log::{Level, LevelFilter, Record};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Write, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// Robin Engine logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Global log level
    pub level: String,
    /// Enable console logging
    pub console_enabled: bool,
    /// Enable file logging
    pub file_enabled: bool,
    /// Log file path
    pub file_path: Option<PathBuf>,
    /// Maximum log file size before rotation (bytes)
    pub max_file_size: u64,
    /// Number of rotated log files to keep
    pub max_files: usize,
    /// Enable structured JSON logging
    pub json_format: bool,
    /// Include timestamps in logs
    pub include_timestamp: bool,
    /// Include thread names/IDs
    pub include_thread_info: bool,
    /// Include source location (file:line)
    pub include_location: bool,
    /// Module-specific log levels
    pub module_levels: HashMap<String, String>,
    /// Performance logging settings
    pub performance: PerformanceLoggingConfig,
    /// Enable crash reporting
    pub crash_reporting: CrashReportingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceLoggingConfig {
    pub enabled: bool,
    /// Log frame time warnings above this threshold (ms)
    pub frame_time_threshold: f64,
    /// Log memory usage warnings above this threshold (MB)
    pub memory_threshold: u64,
    /// Sample rate for performance metrics (0.0 to 1.0)
    pub sample_rate: f64,
    /// Log slow operations above this threshold (ms)
    pub slow_operation_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashReportingConfig {
    pub enabled: bool,
    /// Directory to save crash reports
    pub crash_dir: PathBuf,
    /// Include system information in crash reports
    pub include_system_info: bool,
    /// Include engine state in crash reports
    pub include_engine_state: bool,
    /// Maximum number of crash reports to keep
    pub max_reports: usize,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            console_enabled: true,
            file_enabled: true,
            file_path: Some(PathBuf::from("logs/robin.log")),
            max_file_size: 10 * 1024 * 1024, // 10MB
            max_files: 5,
            json_format: false,
            include_timestamp: true,
            include_thread_info: false,
            include_location: false,
            module_levels: HashMap::new(),
            performance: PerformanceLoggingConfig::default(),
            crash_reporting: CrashReportingConfig::default(),
        }
    }
}

impl Default for PerformanceLoggingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            frame_time_threshold: 16.7, // 60 FPS = 16.7ms per frame
            memory_threshold: 100, // 100MB
            sample_rate: 0.1, // 10% sampling
            slow_operation_threshold: 10.0, // 10ms
        }
    }
}

impl Default for CrashReportingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            crash_dir: PathBuf::from("crashes"),
            include_system_info: true,
            include_engine_state: true,
            max_reports: 10,
        }
    }
}

/// Structured log entry for JSON logging
#[derive(Debug, Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub target: String,
    pub message: String,
    pub thread: Option<String>,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub fields: HashMap<String, serde_json::Value>,
}

/// Performance metrics for logging
#[derive(Debug, Clone, Serialize)]
pub struct PerformanceMetrics {
    pub frame_time: f64,
    pub fps: f64,
    pub memory_usage: u64,
    pub draw_calls: u32,
    pub triangles: u32,
    pub active_objects: u32,
    pub audio_sources: u32,
    pub timestamp: DateTime<Utc>,
}

/// System information for crash reports
#[derive(Debug, Serialize)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub cpu_count: usize,
    pub memory_total: u64,
    pub engine_version: String,
    pub build_timestamp: String,
}

/// Crash report data
#[derive(Debug, Serialize)]
pub struct CrashReport {
    pub timestamp: DateTime<Utc>,
    pub error: String,
    pub backtrace: Option<String>,
    pub system_info: SystemInfo,
    pub engine_state: Option<serde_json::Value>,
    pub recent_logs: Vec<String>,
}

/// Main logging system for Robin Engine
pub struct RobinLogger {
    config: LoggingConfig,
    file_writer: Option<Arc<Mutex<BufWriter<File>>>>,
    current_file_size: Arc<Mutex<u64>>,
    performance_samples: Arc<Mutex<Vec<PerformanceMetrics>>>,
    recent_logs: Arc<Mutex<std::collections::VecDeque<String>>>,
}

impl RobinLogger {
    /// Initialize the logging system with configuration
    pub fn init(config: LoggingConfig) -> Result<Self, Box<dyn std::error::Error>> {
        // Parse log level
        let level_filter = match config.level.to_lowercase().as_str() {
            "error" => LevelFilter::Error,
            "warn" => LevelFilter::Warn,
            "info" => LevelFilter::Info,
            "debug" => LevelFilter::Debug,
            "trace" => LevelFilter::Trace,
            "off" => LevelFilter::Off,
            _ => LevelFilter::Info,
        };

        // Setup file writer if enabled
        let file_writer = if config.file_enabled {
            if let Some(ref file_path) = config.file_path {
                // Create directory if it doesn't exist
                if let Some(parent) = file_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }

                let file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(file_path)?;
                
                Some(Arc::new(Mutex::new(BufWriter::new(file))))
            } else {
                None
            }
        } else {
            None
        };

        // Create crash report directory
        if config.crash_reporting.enabled {
            std::fs::create_dir_all(&config.crash_reporting.crash_dir)?;
        }

        let logger = Self {
            config: config.clone(),
            file_writer,
            current_file_size: Arc::new(Mutex::new(0)),
            performance_samples: Arc::new(Mutex::new(Vec::new())),
            recent_logs: Arc::new(Mutex::new(std::collections::VecDeque::with_capacity(1000))),
        };

        // Initialize global logger
        log::set_boxed_logger(Box::new(RobinLoggerImpl {
            inner: Arc::new(Mutex::new(logger.clone())),
        }))?;
        
        log::set_max_level(level_filter);

        log::info!("Robin Engine logging system initialized");
        log::info!("Log level: {}", config.level);
        log::info!("Console logging: {}", config.console_enabled);
        log::info!("File logging: {} ({})", 
            config.file_enabled,
            config.file_path.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "none".to_string())
        );

        Ok(logger)
    }

    /// Log performance metrics
    pub fn log_performance(&self, metrics: PerformanceMetrics) {
        if !self.config.performance.enabled {
            return;
        }

        // Sample based on configured rate
        if rand::random::<f64>() > self.config.performance.sample_rate {
            return;
        }

        // Check thresholds and warn if exceeded
        if metrics.frame_time > self.config.performance.frame_time_threshold {
            log::warn!("Frame time exceeded threshold: {:.2}ms (limit: {:.2}ms)", 
                metrics.frame_time, self.config.performance.frame_time_threshold);
        }

        if metrics.memory_usage > self.config.performance.memory_threshold * 1024 * 1024 {
            log::warn!("Memory usage exceeded threshold: {:.2}MB (limit: {}MB)",
                metrics.memory_usage as f64 / (1024.0 * 1024.0), self.config.performance.memory_threshold);
        }

        // Store sample
        if let Ok(mut samples) = self.performance_samples.lock() {
            samples.push(metrics.clone());
            
            // Keep only last 1000 samples
            if samples.len() > 1000 {
                let len = samples.len();
                samples.drain(0..len - 1000);
            }
        }

        // Log periodically (every 100 samples)
        if let Ok(samples) = self.performance_samples.lock() {
            if samples.len() % 100 == 0 {
                let avg_frame_time = samples.iter()
                    .rev()
                    .take(100)
                    .map(|m| m.frame_time)
                    .sum::<f64>() / 100.0;
                
                let avg_fps = samples.iter()
                    .rev()
                    .take(100)
                    .map(|m| m.fps)
                    .sum::<f64>() / 100.0;

                log::debug!("Performance (100 sample avg): {:.2}ms frame time, {:.1} FPS, {}MB memory",
                    avg_frame_time, avg_fps, metrics.memory_usage / (1024 * 1024));
            }
        }
    }

    /// Generate crash report
    pub fn generate_crash_report(&self, error: &str, backtrace: Option<String>) -> Result<PathBuf, Box<dyn std::error::Error>> {
        if !self.config.crash_reporting.enabled {
            return Err("Crash reporting is disabled".into());
        }

        let timestamp = Utc::now();
        let filename = format!("crash_{}.json", timestamp.format("%Y%m%d_%H%M%S"));
        let crash_path = self.config.crash_reporting.crash_dir.join(filename);

        // Collect system information
        let system_info = SystemInfo {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            cpu_count: num_cpus::get(),
            memory_total: get_memory_info().unwrap_or(0),
            engine_version: env!("CARGO_PKG_VERSION").to_string(),
            build_timestamp: std::env::var("BUILD_TIMESTAMP").unwrap_or_else(|_| "unknown".to_string()),
        };

        // Get recent logs
        let recent_logs = if let Ok(logs) = self.recent_logs.lock() {
            logs.iter().cloned().collect()
        } else {
            Vec::new()
        };

        let crash_report = CrashReport {
            timestamp,
            error: error.to_string(),
            backtrace,
            system_info,
            engine_state: None, // Could be populated with engine-specific state
            recent_logs,
        };

        // Write crash report
        let crash_json = serde_json::to_string_pretty(&crash_report)?;
        std::fs::write(&crash_path, crash_json)?;

        // Cleanup old crash reports
        self.cleanup_old_crash_reports()?;

        log::error!("Crash report generated: {}", crash_path.display());
        Ok(crash_path)
    }

    /// Get performance statistics
    pub fn get_performance_stats(&self) -> Option<PerformanceStats> {
        let samples = self.performance_samples.lock().ok()?;
        
        if samples.is_empty() {
            return None;
        }

        let recent_samples: Vec<_> = samples.iter().rev().take(100).collect();
        
        let avg_frame_time = recent_samples.iter().map(|m| m.frame_time).sum::<f64>() / recent_samples.len() as f64;
        let min_frame_time = recent_samples.iter().map(|m| m.frame_time).fold(f64::INFINITY, f64::min);
        let max_frame_time = recent_samples.iter().map(|m| m.frame_time).fold(0.0, f64::max);
        
        let avg_fps = recent_samples.iter().map(|m| m.fps).sum::<f64>() / recent_samples.len() as f64;
        let current_memory = recent_samples.first().map(|m| m.memory_usage).unwrap_or(0);

        Some(PerformanceStats {
            avg_frame_time,
            min_frame_time,
            max_frame_time,
            avg_fps,
            current_memory_usage: current_memory,
            sample_count: recent_samples.len(),
        })
    }

    fn cleanup_old_crash_reports(&self) -> Result<(), Box<dyn std::error::Error>> {
        let crash_dir = &self.config.crash_reporting.crash_dir;
        if !crash_dir.exists() {
            return Ok(());
        }

        let mut crash_files: Vec<_> = std::fs::read_dir(crash_dir)?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension().and_then(|ext| ext.to_str()) == Some("json") &&
                   path.file_name().and_then(|name| name.to_str())?.starts_with("crash_") {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        crash_files.sort_by_key(|path| {
            std::fs::metadata(path)
                .and_then(|meta| meta.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH)
        });

        // Keep only the most recent crash reports
        if crash_files.len() > self.config.crash_reporting.max_reports {
            let to_remove = crash_files.len() - self.config.crash_reporting.max_reports;
            for file in crash_files.iter().take(to_remove) {
                if let Err(e) = std::fs::remove_file(file) {
                    log::warn!("Failed to remove old crash report {}: {}", file.display(), e);
                }
            }
        }

        Ok(())
    }
}

impl Clone for RobinLogger {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            file_writer: self.file_writer.clone(),
            current_file_size: self.current_file_size.clone(),
            performance_samples: self.performance_samples.clone(),
            recent_logs: self.recent_logs.clone(),
        }
    }
}

/// Performance statistics summary
#[derive(Debug, Clone, Serialize)]
pub struct PerformanceStats {
    pub avg_frame_time: f64,
    pub min_frame_time: f64,
    pub max_frame_time: f64,
    pub avg_fps: f64,
    pub current_memory_usage: u64,
    pub sample_count: usize,
}

/// Implementation of the log::Log trait
struct RobinLoggerImpl {
    inner: Arc<Mutex<RobinLogger>>,
}

impl log::Log for RobinLoggerImpl {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        // Check module-specific levels
        if let Ok(logger) = self.inner.lock() {
            if let Some(module_level) = logger.config.module_levels.get(metadata.target()) {
                let level = match module_level.to_lowercase().as_str() {
                    "error" => Level::Error,
                    "warn" => Level::Warn,
                    "info" => Level::Info,
                    "debug" => Level::Debug,
                    "trace" => Level::Trace,
                    _ => Level::Info,
                };
                return metadata.level() <= level;
            }
        }
        
        true
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        if let Ok(logger) = self.inner.lock() {
            let timestamp = if logger.config.include_timestamp {
                Some(Utc::now().format("%Y-%m-%d %H:%M:%S%.3f UTC").to_string())
            } else {
                None
            };

            let thread_info = if logger.config.include_thread_info {
                std::thread::current().name().map(|name| name.to_string())
            } else {
                None
            };

            let location = if logger.config.include_location {
                record.file().map(|file| format!("{}:{}", file, record.line().unwrap_or(0)))
            } else {
                None
            };

            let log_line = if logger.config.json_format {
                let entry = LogEntry {
                    timestamp: timestamp.unwrap_or_else(|| "".to_string()),
                    level: record.level().to_string(),
                    target: record.target().to_string(),
                    message: record.args().to_string(),
                    thread: thread_info,
                    file: location,
                    line: record.line(),
                    fields: HashMap::new(),
                };
                serde_json::to_string(&entry).unwrap_or_else(|_| record.args().to_string())
            } else {
                let mut parts = Vec::new();
                
                if let Some(ts) = timestamp {
                    parts.push(format!("[{}]", ts));
                }
                
                parts.push(format!("[{}]", record.level()));
                
                if let Some(thread) = thread_info {
                    parts.push(format!("[{}]", thread));
                }
                
                parts.push(format!("[{}]", record.target()));
                
                if let Some(loc) = location {
                    parts.push(format!("[{}]", loc));
                }
                
                parts.push(record.args().to_string());
                
                parts.join(" ")
            };

            // Store in recent logs for crash reporting
            if let Ok(mut recent_logs) = logger.recent_logs.lock() {
                recent_logs.push_back(log_line.clone());
                if recent_logs.len() > 1000 {
                    recent_logs.pop_front();
                }
            }

            // Console output
            if logger.config.console_enabled {
                match record.level() {
                    Level::Error => eprintln!("{}", log_line),
                    _ => println!("{}", log_line),
                }
            }

            // File output
            if let Some(ref file_writer) = logger.file_writer {
                if let Ok(mut writer) = file_writer.lock() {
                    if let Err(e) = writeln!(writer, "{}", log_line) {
                        eprintln!("Failed to write to log file: {}", e);
                    }
                    let _ = writer.flush();
                }
            }
        }
    }

    fn flush(&self) {
        if let Ok(logger) = self.inner.lock() {
            if let Some(ref file_writer) = logger.file_writer {
                if let Ok(mut writer) = file_writer.lock() {
                    let _ = writer.flush();
                }
            }
        }
    }
}

/// Helper function to get system memory information
fn get_memory_info() -> Option<u64> {
    // This is a simplified version - in a real implementation,
    // you'd use platform-specific APIs or a crate like `sysinfo`
    None
}

/// Convenience macros for structured logging
#[macro_export]
macro_rules! log_performance {
    ($frame_time:expr, $fps:expr, $memory:expr) => {
        $crate::engine::logging::log_performance_metrics($frame_time, $fps, $memory, 0, 0, 0, 0);
    };
}

#[macro_export]
macro_rules! log_slow_operation {
    ($operation:expr, $duration:expr) => {
        log::debug!("Slow operation detected: {} took {:.2}ms", $operation, $duration);
    };
}

/// Helper function to create performance metrics
pub fn log_performance_metrics(
    frame_time: f64,
    fps: f64,
    memory_usage: u64,
    draw_calls: u32,
    triangles: u32,
    active_objects: u32,
    audio_sources: u32,
) {
    // This would typically be called through the logger instance
    log::trace!("Performance: {:.2}ms frame, {:.1} FPS, {}MB memory, {} draw calls",
        frame_time, fps, memory_usage / (1024 * 1024), draw_calls);
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_logging_config_default() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, "info");
        assert!(config.console_enabled);
        assert!(config.file_enabled);
    }

    #[test]
    fn test_log_entry_serialization() {
        let entry = LogEntry {
            timestamp: "2024-01-01T00:00:00.000Z".to_string(),
            level: "INFO".to_string(),
            target: "test".to_string(),
            message: "Test message".to_string(),
            thread: None,
            file: None,
            line: None,
            fields: HashMap::new(),
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("Test message"));
        assert!(json.contains("INFO"));
    }

    #[test]
    fn test_performance_stats() {
        let metrics = vec![
            PerformanceMetrics {
                frame_time: 16.0,
                fps: 60.0,
                memory_usage: 100 * 1024 * 1024,
                draw_calls: 50,
                triangles: 1000,
                active_objects: 100,
                audio_sources: 5,
                timestamp: Utc::now(),
            },
            PerformanceMetrics {
                frame_time: 20.0,
                fps: 50.0,
                memory_usage: 120 * 1024 * 1024,
                draw_calls: 60,
                triangles: 1200,
                active_objects: 120,
                audio_sources: 6,
                timestamp: Utc::now(),
            },
        ];

        let avg_frame_time = metrics.iter().map(|m| m.frame_time).sum::<f64>() / metrics.len() as f64;
        assert_eq!(avg_frame_time, 18.0);
    }
}