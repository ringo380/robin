// Enhanced logging system with categories and levels for Robin Engine Demo
use env_logger::Env;
use log::LevelFilter;
use std::sync::Once;

/// Logging categories for structured logging
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogCategory {
    Engine,       // Core engine operations
    Renderer,     // Graphics and Metal rendering
    Input,        // User input and controls
    Build,        // Engineer Build Mode operations
    World,        // Voxel world and terrain
    UI,           // User interface operations
    Performance,  // Performance monitoring and metrics
    Config,       // Configuration system
    Window,       // Window management
    Audio,        // Audio systems
    Network,      // Networking (future)
    AI,           // AI systems (future)
}

impl LogCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogCategory::Engine => "ENGINE",
            LogCategory::Renderer => "RENDER",
            LogCategory::Input => "INPUT",
            LogCategory::Build => "BUILD",
            LogCategory::World => "WORLD",
            LogCategory::UI => "UI",
            LogCategory::Performance => "PERF",
            LogCategory::Config => "CONFIG",
            LogCategory::Window => "WINDOW",
            LogCategory::Audio => "AUDIO",
            LogCategory::Network => "NET",
            LogCategory::AI => "AI",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            LogCategory::Engine => "🚀",
            LogCategory::Renderer => "🎨",
            LogCategory::Input => "⌨️",
            LogCategory::Build => "🔧",
            LogCategory::World => "🌍",
            LogCategory::UI => "🖼️",
            LogCategory::Performance => "⚡",
            LogCategory::Config => "⚙️",
            LogCategory::Window => "🪟",
            LogCategory::Audio => "🔊",
            LogCategory::Network => "🌐",
            LogCategory::AI => "🤖",
        }
    }
}

/// Logging configuration with levels per category
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub global_level: LevelFilter,
    pub category_levels: std::collections::HashMap<LogCategory, LevelFilter>,
    pub show_module_path: bool,
    pub show_thread_names: bool,
    pub use_color: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        let mut category_levels = std::collections::HashMap::new();

        // Set default levels for different categories
        category_levels.insert(LogCategory::Engine, LevelFilter::Info);
        category_levels.insert(LogCategory::Renderer, LevelFilter::Info);
        category_levels.insert(LogCategory::Input, LevelFilter::Warn);
        category_levels.insert(LogCategory::Build, LevelFilter::Info);
        category_levels.insert(LogCategory::World, LevelFilter::Info);
        category_levels.insert(LogCategory::UI, LevelFilter::Warn);
        category_levels.insert(LogCategory::Performance, LevelFilter::Info);
        category_levels.insert(LogCategory::Config, LevelFilter::Info);
        category_levels.insert(LogCategory::Window, LevelFilter::Info);
        category_levels.insert(LogCategory::Audio, LevelFilter::Info);
        category_levels.insert(LogCategory::Network, LevelFilter::Debug);
        category_levels.insert(LogCategory::AI, LevelFilter::Debug);

        Self {
            global_level: LevelFilter::Info,
            category_levels,
            show_module_path: false,
            show_thread_names: false,
            use_color: true,
        }
    }
}

/// Initialize the enhanced logging system
static INIT: Once = Once::new();

pub fn init_logging() {
    INIT.call_once(|| {
        let env = Env::default()
            .filter_or("RUST_LOG", "info")
            .write_style_or("RUST_LOG_STYLE", "always");

        env_logger::Builder::from_env(env)
            .format_timestamp_secs()
            .init();
    });
}

pub fn init_logging_with_config(config: &LoggingConfig) {
    INIT.call_once(|| {
        let mut builder = env_logger::Builder::new();

        builder.filter_level(config.global_level);

        if config.use_color {
            builder.write_style(env_logger::WriteStyle::Always);
        } else {
            builder.write_style(env_logger::WriteStyle::Never);
        }

        builder.format_timestamp_secs();
        builder.init();
    });
}

/// Structured logging macros with categories
macro_rules! log_trace {
    ($category:expr, $($arg:tt)*) => {
        log::trace!("{} [{}] {}", $category.icon(), $category.as_str(), format_args!($($arg)*))
    };
}

macro_rules! log_debug {
    ($category:expr, $($arg:tt)*) => {
        log::debug!("{} [{}] {}", $category.icon(), $category.as_str(), format_args!($($arg)*))
    };
}

macro_rules! log_info {
    ($category:expr, $($arg:tt)*) => {
        log::info!("{} [{}] {}", $category.icon(), $category.as_str(), format_args!($($arg)*))
    };
}

macro_rules! log_warn {
    ($category:expr, $($arg:tt)*) => {
        log::warn!("{} [{}] {}", $category.icon(), $category.as_str(), format_args!($($arg)*))
    };
}

macro_rules! log_error {
    ($category:expr, $($arg:tt)*) => {
        log::error!("{} [{}] {}", $category.icon(), $category.as_str(), format_args!($($arg)*))
    };
}

// Export the macros for use in other modules
pub(crate) use log_trace;
pub(crate) use log_debug;
pub(crate) use log_info;
pub(crate) use log_warn;
pub(crate) use log_error;

/// Performance logging helpers
pub struct PerformanceLogger {
    category: LogCategory,
    operation: String,
    start_time: std::time::Instant,
}

impl PerformanceLogger {
    pub fn new(category: LogCategory, operation: &str) -> Self {
        log_debug!(category, "Starting: {}", operation);
        Self {
            category,
            operation: operation.to_string(),
            start_time: std::time::Instant::now(),
        }
    }

    pub fn checkpoint(&self, checkpoint: &str) {
        let elapsed = self.start_time.elapsed();
        log_debug!(self.category, "{} - {}: {:.2}ms", self.operation, checkpoint, elapsed.as_secs_f64() * 1000.0);
    }

    pub fn finish(self) {
        let elapsed = self.start_time.elapsed();
        log_info!(self.category, "{} completed in {:.2}ms", self.operation, elapsed.as_secs_f64() * 1000.0);
    }

    pub fn finish_with_result<T>(self, result: &Result<T, impl std::fmt::Display>) {
        let elapsed = self.start_time.elapsed();
        match result {
            Ok(_) => log_info!(self.category, "{} completed successfully in {:.2}ms", self.operation, elapsed.as_secs_f64() * 1000.0),
            Err(e) => log_error!(self.category, "{} failed after {:.2}ms: {}", self.operation, elapsed.as_secs_f64() * 1000.0, e),
        }
    }
}

/// Convenience functions for common logging patterns
pub fn log_startup_message() {
    log_info!(LogCategory::Engine, "Starting Robin Voxel Engine - macOS Native Demo");
    log_info!(LogCategory::Engine, "Optimized for Apple Silicon with Metal rendering");
}

pub fn log_initialization_step(category: LogCategory, step: &str) {
    log_info!(category, "Initializing {}", step);
}

pub fn log_initialization_complete(category: LogCategory, step: &str) {
    log_info!(category, "{} initialized successfully", step);
}

pub fn log_user_action(category: LogCategory, action: &str) {
    log_info!(category, "User action: {}", action);
}

pub fn log_performance_metric(metric_name: &str, value: f32, unit: &str) {
    log_info!(LogCategory::Performance, "{}: {:.2} {}", metric_name, value, unit);
}

pub fn log_resource_usage(resource_type: &str, amount: usize, unit: &str) {
    log_debug!(LogCategory::Performance, "Resource usage - {}: {} {}", resource_type, amount, unit);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_category_strings() {
        assert_eq!(LogCategory::Engine.as_str(), "ENGINE");
        assert_eq!(LogCategory::Renderer.as_str(), "RENDER");
        assert_eq!(LogCategory::Input.as_str(), "INPUT");
    }

    #[test]
    fn test_log_category_icons() {
        assert_eq!(LogCategory::Engine.icon(), "🚀");
        assert_eq!(LogCategory::Renderer.icon(), "🎨");
        assert_eq!(LogCategory::Build.icon(), "🔧");
    }

    #[test]
    fn test_default_logging_config() {
        let config = LoggingConfig::default();
        assert_eq!(config.global_level, LevelFilter::Info);
        assert!(config.use_color);
        assert!(!config.show_module_path);
    }
}