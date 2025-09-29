/*!
 * Robin Engine - Comprehensive Error Handling System
 *
 * Provides centralized error management with:
 * - Structured error types for all subsystems
 * - Graceful error recovery mechanisms
 * - User-friendly error messages
 * - Error telemetry and logging
 * - Recovery strategies for common failures
 */

use thiserror::Error;
use std::fmt;

/// Application-wide result type for consistent error handling
pub type RobinResult<T> = Result<T, RobinError>;

/// Centralized error type covering all subsystems
#[derive(Error, Debug, Clone)]
pub enum RobinError {
    /// Graphics and rendering system errors
    #[error("Graphics system error: {message}")]
    Graphics {
        message: String,
        recoverable: bool,
        suggested_action: Option<String>,
    },

    /// Buffer management and memory errors
    #[error("Buffer error: {message} (capacity: {capacity}, attempted: {attempted})")]
    Buffer {
        message: String,
        capacity: usize,
        attempted: usize,
        operation: String,
    },

    /// Settings and configuration errors
    #[error("Settings error: {message}")]
    Settings {
        message: String,
        field: Option<String>,
        current_value: Option<String>,
    },

    /// Window and surface management errors
    #[error("Window system error: {message}")]
    Window {
        message: String,
        window_state: Option<String>,
        recoverable: bool,
    },

    /// File I/O and persistence errors
    #[error("File system error: {message}")]
    FileSystem {
        message: String,
        path: Option<String>,
        operation: String,
    },

    /// GPU resource management errors
    #[error("GPU resource error: {message}")]
    GpuResource {
        message: String,
        resource_type: String,
        available_memory: Option<u64>,
        requested_memory: Option<u64>,
    },

    /// User interface and interaction errors
    #[error("UI system error: {message}")]
    UserInterface {
        message: String,
        component: Option<String>,
        user_action: Option<String>,
    },

    /// Application state and lifecycle errors
    #[error("Application state error: {message}")]
    ApplicationState {
        message: String,
        current_state: String,
        attempted_transition: Option<String>,
    },

    /// Validation and input errors
    #[error("Validation error: {message}")]
    Validation {
        message: String,
        field: String,
        provided_value: String,
        expected_format: Option<String>,
    },

    /// External dependency and system errors
    #[error("External system error: {message}")]
    External {
        message: String,
    },
}

/// Error severity levels for logging and user notification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// Critical error requiring immediate application shutdown
    Critical,
    /// Major error affecting core functionality
    Major,
    /// Minor error with available workarounds
    Minor,
    /// Warning that doesn't affect functionality
    Warning,
}

/// Error recovery strategies
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// Retry the operation after a delay
    Retry { max_attempts: u32, delay_ms: u64 },
    /// Fall back to a default or simplified mode
    Fallback { description: String },
    /// Reset to a known good state
    Reset { preserve_user_data: bool },
    /// Continue with degraded functionality
    Degrade { limitations: Vec<String> },
    /// Require user intervention
    UserIntervention { suggested_actions: Vec<String> },
    /// Unrecoverable - application must exit
    Fatal,
}

/// Comprehensive error context for debugging and user support
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub error: RobinError,
    pub severity: ErrorSeverity,
    pub recovery_strategy: RecoveryStrategy,
    pub timestamp: std::time::SystemTime,
    pub user_facing_message: String,
    pub technical_details: Option<String>,
    pub error_id: String,
}

impl RobinError {
    /// Determine the severity of an error
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            RobinError::Graphics { recoverable: false, .. } => ErrorSeverity::Critical,
            RobinError::Buffer { .. } => ErrorSeverity::Major,
            RobinError::Window { recoverable: false, .. } => ErrorSeverity::Critical,
            RobinError::Window { recoverable: true, .. } => ErrorSeverity::Major,
            RobinError::ApplicationState { .. } => ErrorSeverity::Major,
            RobinError::GpuResource { .. } => ErrorSeverity::Major,
            RobinError::Settings { .. } => ErrorSeverity::Minor,
            RobinError::FileSystem { .. } => ErrorSeverity::Minor,
            RobinError::UserInterface { .. } => ErrorSeverity::Minor,
            RobinError::Validation { .. } => ErrorSeverity::Warning,
            RobinError::Graphics { recoverable: true, .. } => ErrorSeverity::Minor,
            RobinError::External { .. } => ErrorSeverity::Major,
        }
    }

    /// Get a user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            RobinError::Graphics { message, .. } =>
                format!("Graphics system issue: {}. Try restarting the application or updating your graphics drivers.", message),

            RobinError::Buffer { operation, capacity, attempted, .. } =>
                format!("Memory buffer overflow detected. The application tried to use {} bytes but only {} bytes were available during {}. This has been prevented for safety.",
                    attempted, capacity, operation),

            RobinError::Settings { message, field, .. } =>
                format!("Settings error: {}. {}", message,
                    field.as_ref().map_or(String::new(), |f| format!(" (Field: {})", f))),

            RobinError::Window { message, .. } =>
                format!("Window system issue: {}. Try resizing the window or changing display settings.", message),

            RobinError::FileSystem { message, path, operation } =>
                format!("File system error during {}: {}. {}", operation, message,
                    path.as_ref().map_or(String::new(), |p| format!(" (Path: {})", p))),

            RobinError::GpuResource { message, resource_type, .. } =>
                format!("GPU resource issue with {}: {}. Try reducing graphics quality or restarting the application.", resource_type, message),

            RobinError::UserInterface { message, component, .. } =>
                format!("Interface issue: {}. {}", message,
                    component.as_ref().map_or(String::new(), |c| format!(" (Component: {})", c))),

            RobinError::ApplicationState { message, current_state, .. } =>
                format!("Application state error: {} (Current state: {}). Try restarting the application.", message, current_state),

            RobinError::Validation { message, field, provided_value, .. } =>
                format!("Invalid input for {}: {}. Provided value: '{}'", field, message, provided_value),

            RobinError::External { message } =>
                format!("External system error: {}. This may be a temporary issue.", message),
        }
    }

    /// Get suggested recovery strategy
    pub fn recovery_strategy(&self) -> RecoveryStrategy {
        match self {
            RobinError::Graphics { recoverable: false, .. } => RecoveryStrategy::Fatal,
            RobinError::Graphics { recoverable: true, .. } => RecoveryStrategy::Fallback {
                description: "Switch to basic rendering mode".to_string()
            },

            RobinError::Buffer { .. } => RecoveryStrategy::Reset { preserve_user_data: true },

            RobinError::Window { recoverable: false, .. } => RecoveryStrategy::Fatal,
            RobinError::Window { recoverable: true, .. } => RecoveryStrategy::Retry {
                max_attempts: 3, delay_ms: 1000
            },

            RobinError::Settings { .. } => RecoveryStrategy::Fallback {
                description: "Use default settings".to_string()
            },

            RobinError::FileSystem { .. } => RecoveryStrategy::Retry {
                max_attempts: 3, delay_ms: 500
            },

            RobinError::GpuResource { .. } => RecoveryStrategy::Degrade {
                limitations: vec!["Reduced graphics quality".to_string(), "Limited effects".to_string()]
            },

            RobinError::UserInterface { .. } => RecoveryStrategy::Fallback {
                description: "Use simplified interface".to_string()
            },

            RobinError::ApplicationState { .. } => RecoveryStrategy::Reset { preserve_user_data: true },

            RobinError::Validation { .. } => RecoveryStrategy::UserIntervention {
                suggested_actions: vec!["Check input format".to_string(), "Use default values".to_string()]
            },

            RobinError::External { .. } => RecoveryStrategy::Retry {
                max_attempts: 2, delay_ms: 2000
            },
        }
    }

    /// Generate a unique error ID for tracking and support
    pub fn generate_error_id(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        format!("{:?}", self).hash(&mut hasher);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        format!("RBN-{:08X}-{:08X}", hasher.finish(), timestamp)
    }
}

impl ErrorContext {
    /// Create a new error context with full details
    pub fn new(error: RobinError) -> Self {
        let severity = error.severity();
        let recovery_strategy = error.recovery_strategy();
        let user_facing_message = error.user_message();
        let error_id = error.generate_error_id();

        Self {
            error,
            severity,
            recovery_strategy,
            timestamp: std::time::SystemTime::now(),
            user_facing_message,
            technical_details: None,
            error_id,
        }
    }

    /// Add technical details for debugging
    pub fn with_technical_details(mut self, details: String) -> Self {
        self.technical_details = Some(details);
        self
    }

    /// Check if the error is recoverable
    pub fn is_recoverable(&self) -> bool {
        !matches!(self.recovery_strategy, RecoveryStrategy::Fatal)
    }

    /// Get formatted error for logging
    pub fn log_message(&self) -> String {
        format!(
            "[{}] {} | Severity: {:?} | Recovery: {:?} | ID: {} | Technical: {}",
            self.timestamp.duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default().as_secs(),
            self.error,
            self.severity,
            self.recovery_strategy,
            self.error_id,
            self.technical_details.as_deref().unwrap_or("None")
        )
    }
}

/// Convenience functions for common error types
impl RobinError {
    pub fn graphics_error(message: impl Into<String>, recoverable: bool) -> Self {
        RobinError::Graphics {
            message: message.into(),
            recoverable,
            suggested_action: None,
        }
    }

    pub fn buffer_overflow(operation: impl Into<String>, capacity: usize, attempted: usize) -> Self {
        RobinError::Buffer {
            message: "Buffer overflow prevented".to_string(),
            capacity,
            attempted,
            operation: operation.into(),
        }
    }

    pub fn settings_error(message: impl Into<String>, field: Option<String>) -> Self {
        RobinError::Settings {
            message: message.into(),
            field,
            current_value: None,
        }
    }

    pub fn window_error(message: impl Into<String>, recoverable: bool) -> Self {
        RobinError::Window {
            message: message.into(),
            window_state: None,
            recoverable,
        }
    }

    pub fn gpu_resource_error(message: impl Into<String>, resource_type: impl Into<String>) -> Self {
        RobinError::GpuResource {
            message: message.into(),
            resource_type: resource_type.into(),
            available_memory: None,
            requested_memory: None,
        }
    }
}

/// Helper trait for converting standard library errors to RobinError
pub trait IntoRobinError<T> {
    fn into_robin_error(self) -> RobinResult<T>;
    fn with_context(self, context: &str) -> RobinResult<T>;
}

impl<T, E> IntoRobinError<T> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn into_robin_error(self) -> RobinResult<T> {
        self.map_err(|e| RobinError::External { message: e.to_string() })
    }

    fn with_context(self, context: &str) -> RobinResult<T> {
        self.map_err(|e| RobinError::External {
            message: format!("{}: {}", context, e)
        })
    }
}