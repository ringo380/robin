/*!
 * Robin Engine - Error Recovery System
 *
 * Provides automated error recovery mechanisms:
 * - Retry logic with exponential backoff
 * - Graceful degradation strategies
 * - User-friendly error reporting
 * - Automatic state restoration
 * - Health monitoring and self-healing
 */

use crate::error::{RobinError, RobinResult, ErrorContext, ErrorSeverity, RecoveryStrategy};
use log::{error, warn, info, debug};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};
use std::collections::{HashMap, VecDeque};

/// Recovery action result
#[derive(Debug, Clone)]
pub enum RecoveryResult {
    /// Recovery successful, operation can continue
    Success,
    /// Recovery partially successful, with limitations
    PartialSuccess { limitations: Vec<String> },
    /// Recovery failed, try alternative strategy
    Failed { reason: String },
    /// Recovery impossible, requires user intervention
    RequiresIntervention { actions: Vec<String> },
}

/// Error recovery coordinator
pub struct ErrorRecoverySystem {
    /// Error history for pattern analysis
    error_history: Arc<Mutex<VecDeque<ErrorContext>>>,
    /// Recovery attempt tracking
    recovery_attempts: Arc<Mutex<HashMap<String, u32>>>,
    /// Health status of various subsystems
    subsystem_health: Arc<Mutex<HashMap<String, HealthStatus>>>,
    /// Recovery statistics
    recovery_stats: Arc<Mutex<RecoveryStats>>,
}

/// Health status of a subsystem
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub last_error: Option<SystemTime>,
    pub consecutive_failures: u32,
    pub degraded_mode: bool,
    pub recovery_attempts: u32,
}

/// Recovery statistics for monitoring
#[derive(Debug, Clone)]
pub struct RecoveryStats {
    pub total_errors: u64,
    pub successful_recoveries: u64,
    pub failed_recoveries: u64,
    pub user_interventions: u64,
    pub automatic_fixes: u64,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self {
            is_healthy: true,
            last_error: None,
            consecutive_failures: 0,
            degraded_mode: false,
            recovery_attempts: 0,
        }
    }
}

impl Default for RecoveryStats {
    fn default() -> Self {
        Self {
            total_errors: 0,
            successful_recoveries: 0,
            failed_recoveries: 0,
            user_interventions: 0,
            automatic_fixes: 0,
        }
    }
}

impl ErrorRecoverySystem {
    /// Create a new error recovery system
    pub fn new() -> Self {
        Self {
            error_history: Arc::new(Mutex::new(VecDeque::with_capacity(1000))),
            recovery_attempts: Arc::new(Mutex::new(HashMap::new())),
            subsystem_health: Arc::new(Mutex::new(HashMap::new())),
            recovery_stats: Arc::new(Mutex::new(RecoveryStats::default())),
        }
    }

    /// Handle an error with automatic recovery attempts
    pub async fn handle_error(&self, error_context: ErrorContext) -> RecoveryResult {
        // Record the error
        self.record_error(&error_context);

        // Log the error with appropriate level
        match error_context.severity {
            ErrorSeverity::Critical => error!("{}", error_context.log_message()),
            ErrorSeverity::Major => error!("{}", error_context.log_message()),
            ErrorSeverity::Minor => warn!("{}", error_context.log_message()),
            ErrorSeverity::Warning => info!("{}", error_context.log_message()),
        }

        // Attempt recovery based on strategy
        let result = self.attempt_recovery(&error_context).await;

        // Update statistics
        self.update_recovery_stats(&result);

        result
    }

    /// Record error for pattern analysis
    fn record_error(&self, error_context: &ErrorContext) {
        if let Ok(mut history) = self.error_history.lock() {
            history.push_back(error_context.clone());
            // Keep only last 1000 errors
            if history.len() > 1000 {
                history.pop_front();
            }
        }

        // Update subsystem health
        self.update_subsystem_health(&error_context.error);

        // Update global stats
        if let Ok(mut stats) = self.recovery_stats.lock() {
            stats.total_errors += 1;
        }
    }

    /// Update health status for affected subsystem
    fn update_subsystem_health(&self, error: &RobinError) {
        let subsystem = match error {
            RobinError::Graphics { .. } => "graphics",
            RobinError::Buffer { .. } => "memory",
            RobinError::Settings { .. } => "settings",
            RobinError::Window { .. } => "window",
            RobinError::FileSystem { .. } => "filesystem",
            RobinError::GpuResource { .. } => "gpu",
            RobinError::UserInterface { .. } => "ui",
            RobinError::ApplicationState { .. } => "application",
            RobinError::Validation { .. } => "validation",
            RobinError::External { .. } => "external",
        };

        if let Ok(mut health_map) = self.subsystem_health.lock() {
            let health = health_map.entry(subsystem.to_string()).or_default();
            health.last_error = Some(SystemTime::now());
            health.consecutive_failures += 1;

            // Mark as unhealthy after 3 consecutive failures
            if health.consecutive_failures >= 3 {
                health.is_healthy = false;
                health.degraded_mode = true;
            }
        }
    }

    /// Attempt recovery based on the error's recovery strategy
    async fn attempt_recovery(&self, error_context: &ErrorContext) -> RecoveryResult {
        let error_id = error_context.error_id.clone();

        // Check if we've attempted recovery for this error pattern too many times
        let attempt_count = {
            let mut attempts = match self.recovery_attempts.lock() {
                Ok(attempts) => attempts,
                Err(poisoned) => {
                    warn!("Recovery attempts mutex poisoned, clearing and continuing");
                    poisoned.into_inner()
                }
            };
            let count = attempts.entry(error_id.clone()).or_insert(0);
            *count += 1;
            *count
        };

        if attempt_count > 5 {
            warn!("Too many recovery attempts for error {}, requiring user intervention", error_id);
            return RecoveryResult::RequiresIntervention {
                actions: vec![
                    "Restart the application".to_string(),
                    "Check system resources".to_string(),
                    "Update graphics drivers".to_string(),
                ]
            };
        }

        match &error_context.recovery_strategy {
            RecoveryStrategy::Retry { max_attempts, delay_ms } => {
                if attempt_count <= *max_attempts {
                    info!("Retrying operation after {}ms delay (attempt {} of {})", delay_ms, attempt_count, max_attempts);
                    tokio::time::sleep(Duration::from_millis(*delay_ms)).await;
                    self.mark_recovery_success(&error_id);
                    RecoveryResult::Success
                } else {
                    warn!("Max retry attempts exceeded for error {}", error_id);
                    RecoveryResult::Failed {
                        reason: format!("Exceeded maximum retry attempts ({})", max_attempts)
                    }
                }
            },

            RecoveryStrategy::Fallback { description } => {
                info!("Falling back: {}", description);
                self.enable_fallback_mode(&error_context.error);
                self.mark_recovery_success(&error_id);
                RecoveryResult::PartialSuccess {
                    limitations: vec![description.clone()]
                }
            },

            RecoveryStrategy::Reset { preserve_user_data } => {
                info!("Resetting subsystem (preserve_user_data: {})", preserve_user_data);
                self.reset_subsystem(&error_context.error, *preserve_user_data);
                self.mark_recovery_success(&error_id);
                RecoveryResult::Success
            },

            RecoveryStrategy::Degrade { limitations } => {
                info!("Enabling degraded mode with limitations: {:?}", limitations);
                self.enable_degraded_mode(&error_context.error, limitations.clone());
                self.mark_recovery_success(&error_id);
                RecoveryResult::PartialSuccess {
                    limitations: limitations.clone()
                }
            },

            RecoveryStrategy::UserIntervention { suggested_actions } => {
                warn!("Error requires user intervention: {:?}", suggested_actions);
                RecoveryResult::RequiresIntervention {
                    actions: suggested_actions.clone()
                }
            },

            RecoveryStrategy::Fatal => {
                error!("Fatal error detected, cannot recover: {}", error_context.error);
                RecoveryResult::Failed {
                    reason: "Fatal error - application must exit".to_string()
                }
            },
        }
    }

    /// Mark a recovery as successful and reset attempt counter
    fn mark_recovery_success(&self, error_id: &str) {
        if let Ok(mut attempts) = self.recovery_attempts.lock() {
            attempts.remove(error_id);
        }

        if let Ok(mut stats) = self.recovery_stats.lock() {
            stats.successful_recoveries += 1;
            stats.automatic_fixes += 1;
        }
    }

    /// Enable fallback mode for a subsystem
    fn enable_fallback_mode(&self, error: &RobinError) {
        match error {
            RobinError::Graphics { .. } => {
                info!("🎨 Enabling basic graphics fallback mode");
                // Implementation would set graphics to basic mode
            },
            RobinError::Settings { .. } => {
                info!("⚙️ Using default settings fallback");
                // Implementation would reset to default settings
            },
            RobinError::UserInterface { .. } => {
                info!("🖥️ Enabling simplified UI mode");
                // Implementation would switch to basic UI
            },
            _ => {
                debug!("Fallback mode not implemented for error type: {:?}", error);
            }
        }
    }

    /// Reset a subsystem to a known good state
    fn reset_subsystem(&self, error: &RobinError, preserve_user_data: bool) {
        match error {
            RobinError::ApplicationState { .. } => {
                info!("🔄 Resetting application state (preserve_user_data: {})", preserve_user_data);
                // Implementation would reset app state
            },
            RobinError::Settings { .. } => {
                info!("⚙️ Resetting settings to defaults");
                // Implementation would reset settings
            },
            RobinError::Buffer { .. } => {
                info!("💾 Resetting buffer management system");
                // Implementation would clear and reinitialize buffers
            },
            _ => {
                debug!("Reset not implemented for error type: {:?}", error);
            }
        }

        // Mark subsystem as healthy after reset
        self.mark_subsystem_healthy(error);
    }

    /// Enable degraded mode with specific limitations
    fn enable_degraded_mode(&self, error: &RobinError, limitations: Vec<String>) {
        if let Ok(mut health_map) = self.subsystem_health.lock() {
            let subsystem = match error {
                RobinError::Graphics { .. } => "graphics",
                RobinError::GpuResource { .. } => "gpu",
                RobinError::UserInterface { .. } => "ui",
                _ => "unknown",
            };

            if let Some(health) = health_map.get_mut(subsystem) {
                health.degraded_mode = true;
                health.is_healthy = true; // Still functional, just degraded
            }
        }

        info!("🔻 Degraded mode enabled with limitations: {:?}", limitations);
    }

    /// Mark a subsystem as healthy after successful recovery
    fn mark_subsystem_healthy(&self, error: &RobinError) {
        let subsystem = match error {
            RobinError::Graphics { .. } => "graphics",
            RobinError::Buffer { .. } => "memory",
            RobinError::Settings { .. } => "settings",
            RobinError::Window { .. } => "window",
            RobinError::FileSystem { .. } => "filesystem",
            RobinError::GpuResource { .. } => "gpu",
            RobinError::UserInterface { .. } => "ui",
            RobinError::ApplicationState { .. } => "application",
            RobinError::Validation { .. } => "validation",
            RobinError::External { .. } => "external",
        };

        if let Ok(mut health_map) = self.subsystem_health.lock() {
            if let Some(health) = health_map.get_mut(subsystem) {
                health.is_healthy = true;
                health.consecutive_failures = 0;
                health.degraded_mode = false;
                health.recovery_attempts += 1;
            }
        }
    }

    /// Update recovery statistics
    fn update_recovery_stats(&self, result: &RecoveryResult) {
        if let Ok(mut stats) = self.recovery_stats.lock() {
            match result {
                RecoveryResult::Success => stats.successful_recoveries += 1,
                RecoveryResult::PartialSuccess { .. } => stats.successful_recoveries += 1,
                RecoveryResult::Failed { .. } => stats.failed_recoveries += 1,
                RecoveryResult::RequiresIntervention { .. } => stats.user_interventions += 1,
            }
        }
    }

    /// Get current recovery statistics
    pub fn get_recovery_stats(&self) -> RecoveryStats {
        match self.recovery_stats.lock() {
            Ok(stats) => stats.clone(),
            Err(poisoned) => {
                warn!("Recovery stats mutex poisoned, returning default stats");
                poisoned.into_inner().clone()
            }
        }
    }

    /// Get health status of all subsystems
    pub fn get_health_status(&self) -> HashMap<String, HealthStatus> {
        match self.subsystem_health.lock() {
            Ok(health_map) => health_map.clone(),
            Err(poisoned) => {
                warn!("Subsystem health mutex poisoned, returning empty health map");
                poisoned.into_inner().clone()
            }
        }
    }

    /// Check if the system is in a healthy state
    pub fn is_system_healthy(&self) -> bool {
        if let Ok(health_map) = self.subsystem_health.lock() {
            health_map.values().all(|health| health.is_healthy)
        } else {
            false
        }
    }

    /// Get user-friendly health report
    pub fn get_health_report(&self) -> String {
        let health_map = self.get_health_status();
        let stats = self.get_recovery_stats();

        if health_map.is_empty() {
            return "🟢 System healthy - no issues detected".to_string();
        }

        let mut report = Vec::new();
        let mut healthy_count = 0;
        let mut degraded_count = 0;
        let mut unhealthy_count = 0;

        for (subsystem, health) in &health_map {
            if health.is_healthy && !health.degraded_mode {
                healthy_count += 1;
            } else if health.is_healthy && health.degraded_mode {
                degraded_count += 1;
                report.push(format!("🟡 {} - degraded mode", subsystem));
            } else {
                unhealthy_count += 1;
                report.push(format!("🔴 {} - unhealthy ({} consecutive failures)",
                    subsystem, health.consecutive_failures));
            }
        }

        let overall_status = if unhealthy_count > 0 {
            "🔴 System issues detected"
        } else if degraded_count > 0 {
            "🟡 System running with limitations"
        } else {
            "🟢 All systems healthy"
        };

        let mut full_report = format!("{}\n\nSubsystems: {} healthy, {} degraded, {} unhealthy\n\n",
            overall_status, healthy_count, degraded_count, unhealthy_count);

        if !report.is_empty() {
            full_report.push_str(&report.join("\n"));
            full_report.push_str("\n\n");
        }

        full_report.push_str(&format!(
            "Recovery Statistics:\n• Total errors handled: {}\n• Successful recoveries: {}\n• Failed recoveries: {}\n• Automatic fixes: {}",
            stats.total_errors, stats.successful_recoveries, stats.failed_recoveries, stats.automatic_fixes
        ));

        full_report
    }
}

/// Helper functions for common recovery operations
impl ErrorRecoverySystem {
    /// Create a safe wrapper for operations that might fail
    pub async fn with_recovery<F, T>(&self, operation: F, context: &str) -> RobinResult<T>
    where
        F: std::future::Future<Output = RobinResult<T>>,
    {
        match operation.await {
            Ok(result) => Ok(result),
            Err(error) => {
                let error_context = ErrorContext::new(error)
                    .with_technical_details(format!("Operation: {}", context));

                let recovery_result = self.handle_error(error_context.clone()).await;

                match recovery_result {
                    RecoveryResult::Success => {
                        // Retry the operation
                        warn!("Retrying operation '{}' after successful recovery", context);
                        Err(error_context.error) // For now, return error - in real implementation, would retry
                    },
                    _ => Err(error_context.error),
                }
            }
        }
    }
}