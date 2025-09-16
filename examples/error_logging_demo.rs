use robin::engine::{
    GameBuilder, 
    error::{RobinError, RobinResult},
    logging::{LoggingConfig, PerformanceLoggingConfig, CrashReportingConfig},
    diagnostics::DiagnosticsConfig,
    input::InputManager,
};
use std::time::Duration;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Robin Engine Error Handling & Logging Demo");
    println!("============================================");
    println!();

    // Demo different aspects of error handling and logging
    demonstrate_logging_setup()?;
    demonstrate_error_handling()?;
    demonstrate_performance_monitoring()?;
    demonstrate_diagnostics()?;
    simulate_error_scenarios()?;

    println!("🏁 Error handling and logging demo completed!");
    Ok(())
}

fn demonstrate_logging_setup() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Setting Up Comprehensive Logging");
    println!("===================================");

    // Create custom logging configuration
    let logging_config = LoggingConfig {
        level: "debug".to_string(),
        console_enabled: true,
        file_enabled: true,
        file_path: Some(PathBuf::from("examples/logs/robin_demo.log")),
        max_file_size: 5 * 1024 * 1024, // 5MB
        max_files: 3,
        json_format: false,
        include_timestamp: true,
        include_thread_info: true,
        include_location: true,
        module_levels: [
            ("robin::engine::physics".to_string(), "info".to_string()),
            ("robin::engine::audio".to_string(), "warn".to_string()),
        ].into_iter().collect(),
        performance: PerformanceLoggingConfig {
            enabled: true,
            frame_time_threshold: 20.0, // Warn if frame takes >20ms
            memory_threshold: 50, // Warn if memory usage >50MB
            sample_rate: 1.0, // Log all performance samples
            slow_operation_threshold: 5.0, // Warn if operation takes >5ms
        },
        crash_reporting: CrashReportingConfig {
            enabled: true,
            crash_dir: PathBuf::from("examples/crashes"),
            include_system_info: true,
            include_engine_state: true,
            max_reports: 5,
        },
    };

    // Initialize game with custom logging
    let mut game = GameBuilder::new()
        .setup_logging(Some(logging_config));

    println!("✅ Logging system configured:");
    println!("  - Console output: enabled");
    println!("  - File logging: examples/logs/robin_demo.log");
    println!("  - Performance monitoring: enabled");
    println!("  - Crash reporting: examples/crashes/");
    println!();

    // Test different log levels
    log::trace!("This is a TRACE message - very detailed debugging info");
    log::debug!("This is a DEBUG message - general debugging info");
    log::info!("This is an INFO message - general information");
    log::warn!("This is a WARN message - something might be wrong");
    log::error!("This is an ERROR message - something is definitely wrong");

    Ok(())
}

fn demonstrate_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚠️  Error Handling Mechanisms");
    println!("============================");

    let mut game = GameBuilder::new()
        .enable_error_recovery(true);

    // Demonstrate different error types
    let errors = vec![
        RobinError::AssetNotFound {
            asset_id: "missing_texture".to_string(),
            asset_type: "Texture".to_string(),
            searched_paths: vec![
                PathBuf::from("assets/textures/missing.png"),
                PathBuf::from("textures/missing.png"),
            ],
        },
        RobinError::AudioPlaybackError {
            sound_id: "background_music".to_string(),
            reason: "Audio device not available".to_string(),
        },
        RobinError::ValidationError {
            field: "volume".to_string(),
            value: "150".to_string(),
            constraint: "Must be between 0 and 100".to_string(),
        },
        RobinError::ComponentError {
            object_id: 42,
            component_type: "Transform".to_string(),
            operation: "update".to_string(),
            reason: "Invalid rotation value".to_string(),
        },
    ];

    for (i, error) in errors.into_iter().enumerate() {
        println!("🔥 Handling error {} of 4:", i + 1);
        
        // Handle the error with context
        game.handle_error(error.clone(), Some(&format!("Demo error scenario {}", i + 1)));
        
        // Check error state
        if game.has_error() {
            println!("  ❌ Engine is in error state");
            if let Some(last_error) = game.get_last_error() {
                println!("  📋 Last error: {}", last_error);
            }
        }
        
        // Demonstrate error recovery
        println!("  🔧 Attempting error recovery...");
        game.clear_last_error();
        
        if !game.has_error() {
            println!("  ✅ Error cleared successfully");
        }
        
        println!();
    }

    println!("Error handling demonstration completed!");
    println!();
    Ok(())
}

fn demonstrate_performance_monitoring() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Performance Monitoring");
    println!("========================");

    let mut game = GameBuilder::new()
        .setup_diagnostics(Some(DiagnosticsConfig {
            enabled: true,
            max_frame_samples: 100,
            performance_sampling_rate: 1.0,
            detailed_profiling: true,
            ..Default::default()
        }));

    let input = InputManager::new();

    println!("Running performance monitoring simulation...");
    println!();

    // Simulate game loop with varying performance
    for frame in 0..60 {
        // Simulate different frame times
        let delta_time = match frame % 20 {
            0..=5 => 0.016,    // Good performance: 60 FPS
            6..=10 => 0.033,   // Medium performance: 30 FPS
            11..=15 => 0.050,  // Poor performance: 20 FPS
            _ => 0.100,        // Very poor performance: 10 FPS
        };

        // Update the game (this records performance metrics)
        game.update(delta_time, &input);

        // Log custom performance counters
        game.log_counter("triangles_rendered", (1000 + frame * 10) as f64)
            .log_counter("draw_calls", (50 + frame / 2) as f64)
            .log_counter("audio_sources", (5 + frame / 10) as f64);

        // Periodic performance reports
        if frame % 20 == 19 {
            println!("Frame {}: {}", frame + 1, game.get_performance_metrics());
            
            // Check engine health
            let health = game.get_health_status();
            println!("Engine Health: {:?} - {}", health.overall, health.details);
            println!();
        }

        // Simulate frame delay
        std::thread::sleep(Duration::from_millis(10));
    }

    // Take a memory snapshot
    game.snapshot_memory();
    
    println!("Performance monitoring completed!");
    println!();
    Ok(())
}

fn demonstrate_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Comprehensive Diagnostics");
    println!("============================");

    let mut game = GameBuilder::new();
    let input = InputManager::new();

    // Run a simulation to generate diagnostic data
    println!("Generating diagnostic data...");
    
    // Simulate some operations
    for i in 0..30 {
        game.update(0.016, &input);
        
        // Simulate some errors occasionally
        if i % 10 == 9 {
            let error = RobinError::RenderingError(
                format!("Simulated rendering issue #{}", i / 10 + 1)
            );
            game.handle_error(error, Some("Diagnostic simulation"));
            game.clear_last_error(); // Clear for next iteration
        }
        
        // Log some metrics
        game.log_counter("simulation_step", i as f64);
        
        std::thread::sleep(Duration::from_millis(5));
    }

    // Generate comprehensive diagnostics report
    println!("\n📋 Generating Diagnostics Report:");
    println!("=================================");
    
    let report = game.generate_diagnostics_report();
    
    // Save report to file
    std::fs::create_dir_all("examples/diagnostics")?;
    let report_path = "examples/diagnostics/demo_report.json";
    std::fs::write(report_path, &report)?;
    
    println!("Full diagnostics report saved to: {}", report_path);
    
    // Show summary of the report
    println!("\n📊 Diagnostics Summary:");
    println!("{}", game.get_performance_metrics());
    
    let health = game.get_health_status();
    println!("Overall Health: {:?}", health.overall);
    println!("Performance: {:?}", health.performance);
    println!("Memory: {:?}", health.memory);
    println!("Errors: {:?}", health.errors);
    println!();

    Ok(())
}

fn simulate_error_scenarios() -> Result<(), Box<dyn std::error::Error>> {
    println!("💥 Error Scenario Simulation");
    println!("============================");

    let mut game = GameBuilder::new();

    // Scenario 1: Asset loading failures
    println!("Scenario 1: Asset Loading Failures");
    for i in 0..3 {
        let error = RobinError::AssetLoadError {
            asset_id: format!("test_asset_{}", i),
            path: PathBuf::from(format!("assets/missing_{}.png", i)),
            reason: "File corrupted or format not supported".to_string(),
        };
        game.handle_error(error, Some("Asset loading test"));
        game.clear_last_error();
    }

    // Scenario 2: System initialization failures
    println!("Scenario 2: System Initialization Issues");
    let init_error = RobinError::InitializationError {
        subsystem: "Audio System".to_string(),
        reason: "No audio devices found".to_string(),
    };
    game.handle_error(init_error, Some("System initialization"));
    game.clear_last_error();

    // Scenario 3: Runtime validation errors
    println!("Scenario 3: Runtime Validation Errors");
    let validation_errors = vec![
        RobinError::RangeError {
            parameter: "volume".to_string(),
            value: 150.0,
            min: 0.0,
            max: 100.0,
        },
        RobinError::ValidationError {
            field: "texture_size".to_string(),
            value: "8192x8192".to_string(),
            constraint: "Maximum texture size is 4096x4096".to_string(),
        },
    ];

    for error in validation_errors {
        game.handle_error(error, Some("Input validation"));
        game.clear_last_error();
    }

    // Scenario 4: Resource exhaustion
    println!("Scenario 4: Resource Exhaustion");
    let resource_error = RobinError::ResourceExhausted {
        resource_type: "Texture Memory".to_string(),
        limit: 256,
        requested: 512,
    };
    game.handle_error(resource_error, Some("Resource management"));

    // Check final health status after all errors
    let final_health = game.get_health_status();
    println!("\n🏥 Final Engine Health After Error Scenarios:");
    println!("Overall: {:?}", final_health.overall);
    println!("Details: {}", final_health.details);

    if game.has_error() {
        println!("⚠️  Engine has unresolved errors");
    } else {
        println!("✅ All errors have been handled successfully");
    }

    println!();
    Ok(())
}

fn demonstrate_crash_reporting() -> Result<(), Box<dyn std::error::Error>> {
    println!("💀 Crash Reporting Simulation");
    println!("=============================");

    // This would typically be called when a panic or critical error occurs
    let game = GameBuilder::new();
    
    // Simulate a crash scenario (but don't actually crash)
    let critical_error = RobinError::InternalError(
        "Simulated critical engine failure for demo purposes".to_string()
    );

    println!("Simulating critical error: {}", critical_error);
    
    // In a real scenario, this would be called from a panic handler
    // or when a critical error is detected
    println!("✅ Crash reporting system would generate detailed report");
    println!("  - System information");
    println!("  - Engine state snapshot");
    println!("  - Recent log entries");
    println!("  - Stack trace (if available)");
    println!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_handling_flow() {
        let mut game = GameBuilder::new();
        
        // Test error handling
        let error = RobinError::ValidationError {
            field: "test".to_string(),
            value: "invalid".to_string(),
            constraint: "must be valid".to_string(),
        };
        
        assert!(!game.has_error());
        
        game.handle_error(error, Some("test"));
        assert!(game.has_error());
        
        game.clear_last_error();
        assert!(!game.has_error());
    }

    #[test]
    fn test_performance_monitoring() {
        let mut game = GameBuilder::new();
        let input = InputManager::new();
        
        // Run a few frames
        for _ in 0..5 {
            game.update(0.016, &input);
            game.log_counter("test_counter", 42.0);
        }
        
        let metrics = game.get_performance_metrics();
        assert!(metrics.contains("FPS"));
        assert!(metrics.contains("Frame Time"));
    }

    #[test]
    fn test_health_status() {
        let game = GameBuilder::new();
        let health = game.get_health_status();
        
        // Fresh engine should be in good health
        assert_eq!(health.overall, robin::engine::game_builder::HealthLevel::Good);
        assert_eq!(health.errors, robin::engine::game_builder::HealthLevel::Good);
    }

    #[test]
    fn test_diagnostics_report() {
        let game = GameBuilder::new();
        let report = game.generate_diagnostics_report();
        
        // Should be valid JSON
        assert!(serde_json::from_str::<serde_json::Value>(&report).is_ok());
    }
}