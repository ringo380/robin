/// Error Handling Module for Robin Engine
///
/// Comprehensive error management, recovery, and robustness system

pub mod robust_error_system;
pub mod error_recovery;
pub mod monitoring;

pub use robust_error_system::{
    RobustErrorSystem, RobinError, RobinErrorType, ErrorSeverity,
    ErrorContext, RecoveryAction, ErrorSystemConfig, ErrorStatistics
};

pub use error_recovery::{ErrorRecoveryService, RecoveryConfig, RecoveryStatistics};
pub use monitoring::{SystemHealthMonitor, HealthReport, HealthMetrics, HealthAlert};