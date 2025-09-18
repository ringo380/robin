// Stress tests for AI Game Systems
// Tests system behavior under heavy load and edge conditions

use robin::engine::ai_game::*;
use robin::engine::GameBuilder;
use robin::engine::error::RobinResult;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

fn create_random_profile(player_id: &str, seed: u64) -> PlayerProfile {
    // Simple deterministic "random" based on seed
    let skill = (seed % 100) as f32 / 100.0;
    let engagement = ((seed * 17) % 100) as f32 / 100.0;
    let playtime_secs = (seed % 36000) + 300; // 5 min to 10 hours

    PlayerProfile {
        player_id: player_id.to_string(),
        skill_level: skill,
        engagement_level: engagement,
        learning_style: if seed % 2 == 0 { "Visual" } else { "Kinesthetic" }.to_string(),
        preferences: HashMap::new(),
        play_history: Vec::new(),
        performance_metrics: HashMap::new(),
        last_session: std::time::SystemTime::now(),
        total_playtime: Duration::from_secs(playtime_secs),
        achievement_progress: HashMap::new(),
    }
}

fn create_random_interaction(seed: u64) -> PlayerInteraction {
    let interaction_types = ["building", "exploration", "combat", "puzzle", "social", "resource_gathering"];
    let interaction_type = interaction_types[seed as usize % interaction_types.len()];

    PlayerInteraction {
        interaction_type: interaction_type.to_string(),
        timestamp: std::time::SystemTime::now(),
        duration: Duration::from_secs(10 + (seed % 300)), // 10s to 5min
        success_rate: (seed % 100) as f32 / 100.0,
        complexity_level: ((seed * 7) % 100) as f32 / 100.0,
        context: HashMap::new(),
        performance_data: HashMap::new(),
    }
}

#[test]
fn test_high_player_count() -> RobinResult<()> {
    let mut manager = GameAIManager::new();
    const PLAYER_COUNT: usize = 1000;

    let start_time = Instant::now();

    // Create many players
    for i in 0..PLAYER_COUNT {
        let profile = create_random_profile(&format!("stress_player_{}", i), i as u64);
        manager.start_player_session(&format!("stress_player_{}", i), profile)?;
    }

    let creation_time = start_time.elapsed();
    println!("Created {} players in {:?}", PLAYER_COUNT, creation_time);

    // Should handle 1000 players reasonably quickly (< 5 seconds)
    assert!(creation_time.as_secs() < 5, "Player creation too slow: {:?}", creation_time);

    // Test getting metrics for all players
    let metrics_start = Instant::now();
    let mut metrics_count = 0;

    for i in 0..PLAYER_COUNT {
        if manager.get_player_metrics(&format!("stress_player_{}", i)).is_some() {
            metrics_count += 1;
        }
    }

    let metrics_time = metrics_start.elapsed();
    assert_eq!(metrics_count, PLAYER_COUNT);
    assert!(metrics_time.as_millis() < 1000, "Metrics retrieval too slow: {:?}", metrics_time);

    Ok(())
}

#[test]
fn test_high_interaction_rate() -> RobinResult<()> {
    let mut manager = GameAIManager::new();

    let profile = create_random_profile("high_rate_player", 42);
    manager.start_player_session("high_rate_player", profile)?;

    const INTERACTION_COUNT: usize = 10000;
    let start_time = Instant::now();

    // Process many interactions rapidly
    for i in 0..INTERACTION_COUNT {
        let interaction = create_random_interaction(i as u64);
        manager.process_player_interaction("high_rate_player", &interaction)?;
    }

    let processing_time = start_time.elapsed();
    println!("Processed {} interactions in {:?}", INTERACTION_COUNT, processing_time);

    // Should handle 10k interactions in reasonable time (< 10 seconds)
    assert!(processing_time.as_secs() < 10, "Interaction processing too slow: {:?}", processing_time);

    // Verify data integrity
    let metrics = manager.get_player_metrics("high_rate_player");
    assert!(metrics.is_some());

    Ok(())
}

#[test]
fn test_memory_stability_under_load() -> RobinResult<()> {
    let mut manager = GameAIManager::new();

    // Create players and run for extended period
    const LONG_RUNNING_PLAYERS: usize = 100;
    const SIMULATION_CYCLES: usize = 1000;

    // Create players
    for i in 0..LONG_RUNNING_PLAYERS {
        let profile = create_random_profile(&format!("long_player_{}", i), i as u64);
        manager.start_player_session(&format!("long_player_{}", i), profile)?;
    }

    let start_time = Instant::now();

    // Simulate extended gameplay
    for cycle in 0..SIMULATION_CYCLES {
        // Update AI systems
        manager.update(0.016)?;

        // Random interactions from random players
        let player_id = format!("long_player_{}", cycle % LONG_RUNNING_PLAYERS);
        let interaction = create_random_interaction(cycle as u64);
        manager.process_player_interaction(&player_id, &interaction)?;

        // Periodic operations
        if cycle % 100 == 0 {
            let _ = manager.get_recommendations(&player_id);
            let _ = manager.adjust_difficulty(&player_id, 0.7);
        }

        // Simulate content generation periodically
        if cycle % 200 == 0 {
            let mut params = HashMap::new();
            params.insert("complexity".to_string(), 0.5);
            let _ = manager.generate_content(&player_id, "structure", params);
        }
    }

    let total_time = start_time.elapsed();
    println!("Ran {} cycles with {} players in {:?}", SIMULATION_CYCLES, LONG_RUNNING_PLAYERS, total_time);

    // Should complete in reasonable time
    assert!(total_time.as_secs() < 30, "Long-running simulation too slow: {:?}", total_time);

    // Verify all players still have valid data
    for i in 0..LONG_RUNNING_PLAYERS {
        let metrics = manager.get_player_metrics(&format!("long_player_{}", i));
        assert!(metrics.is_some(), "Player {} lost data", i);
    }

    Ok(())
}

#[test]
fn test_concurrent_access_simulation() -> RobinResult<()> {
    let manager = Arc::new(Mutex::new(GameAIManager::new()));
    const THREAD_COUNT: usize = 10;
    const OPERATIONS_PER_THREAD: usize = 100;

    // Create initial players
    {
        let mut mgr = manager.lock().unwrap();
        for i in 0..THREAD_COUNT {
            let profile = create_random_profile(&format!("concurrent_player_{}", i), i as u64);
            mgr.start_player_session(&format!("concurrent_player_{}", i), profile)?;
        }
    }

    let mut handles = Vec::new();

    // Spawn multiple threads doing concurrent operations
    for thread_id in 0..THREAD_COUNT {
        let manager_clone = Arc::clone(&manager);

        let handle = thread::spawn(move || {
            let player_id = format!("concurrent_player_{}", thread_id);

            for op in 0..OPERATIONS_PER_THREAD {
                let mut mgr = manager_clone.lock().unwrap();

                match op % 4 {
                    0 => {
                        // Process interaction
                        let interaction = create_random_interaction((thread_id * 1000 + op) as u64);
                        let _ = mgr.process_player_interaction(&player_id, &interaction);
                    },
                    1 => {
                        // Get recommendations
                        let _ = mgr.get_recommendations(&player_id);
                    },
                    2 => {
                        // Adjust difficulty
                        let _ = mgr.adjust_difficulty(&player_id, 0.5 + (op as f32) * 0.01);
                    },
                    3 => {
                        // Update systems
                        let _ = mgr.update(0.016);
                    },
                    _ => unreachable!(),
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // Verify system is still functional
    {
        let mgr = manager.lock().unwrap();
        for i in 0..THREAD_COUNT {
            let metrics = mgr.get_player_metrics(&format!("concurrent_player_{}", i));
            assert!(metrics.is_some(), "Player {} lost data during concurrent access", i);
        }
    }

    Ok(())
}

#[test]
fn test_rapid_session_turnover() -> RobinResult<()> {
    let mut manager = GameAIManager::new();
    const TURNOVER_CYCLES: usize = 500;

    let start_time = Instant::now();

    for i in 0..TURNOVER_CYCLES {
        let player_id = format!("turnover_player_{}", i);
        let profile = create_random_profile(&player_id, i as u64);

        // Start session
        manager.start_player_session(&player_id, profile)?;

        // Do some quick interactions
        for j in 0..5 {
            let interaction = create_random_interaction((i * 10 + j) as u64);
            manager.process_player_interaction(&player_id, &interaction)?;
        }

        // End session immediately
        let _final_profile = manager.end_player_session(&player_id)?;

        // Verify session is gone
        assert!(manager.get_player_metrics(&player_id).is_none());
    }

    let total_time = start_time.elapsed();
    println!("Completed {} session turnovers in {:?}", TURNOVER_CYCLES, total_time);

    // Should handle rapid turnover efficiently
    assert!(total_time.as_secs() < 15, "Session turnover too slow: {:?}", total_time);

    Ok(())
}

#[test]
fn test_extreme_difficulty_adjustments() -> RobinResult<()> {
    let mut manager = GameAIManager::new();

    let profile = create_random_profile("extreme_test_player", 123);
    manager.start_player_session("extreme_test_player", profile)?;

    // Test extreme difficulty values
    let extreme_values = vec![0.0, 1.0, 0.001, 0.999, 0.5];

    for target_difficulty in extreme_values {
        let result = manager.adjust_difficulty("extreme_test_player", target_difficulty)?;

        // Should return valid difficulty values
        assert!(result >= 0.0 && result <= 1.0, "Invalid difficulty returned: {}", result);
    }

    Ok(())
}

#[test]
fn test_massive_content_generation() -> RobinResult<()> {
    let mut manager = GameAIManager::new();

    let profile = create_random_profile("content_stress_player", 456);
    manager.start_player_session("content_stress_player", profile)?;

    const CONTENT_REQUESTS: usize = 200;
    let content_types = ["world", "structure", "challenge", "tool", "material", "blueprint"];

    let start_time = Instant::now();

    for i in 0..CONTENT_REQUESTS {
        let content_type = content_types[i % content_types.len()];
        let mut params = HashMap::new();
        params.insert("complexity".to_string(), (i as f32) / (CONTENT_REQUESTS as f32));
        params.insert("variation".to_string(), ((i * 7) % 100) as f32 / 100.0);

        let content = manager.generate_content("content_stress_player", content_type, params)?;
        assert!(!content.is_empty(), "Empty content generated for type: {}", content_type);
    }

    let generation_time = start_time.elapsed();
    println!("Generated {} content items in {:?}", CONTENT_REQUESTS, generation_time);

    // Should generate content efficiently
    assert!(generation_time.as_secs() < 20, "Content generation too slow: {:?}", generation_time);

    Ok(())
}

#[test]
fn test_gamebuilder_stress_integration() -> RobinResult<()> {
    let mut game_builder = GameBuilder::new();

    const STRESS_PLAYERS: usize = 50;
    const STRESS_OPERATIONS: usize = 1000;

    // Enable all AI features
    game_builder.enable_ai_features(vec!["analytics", "adaptation", "generation", "balancing"]);

    let start_time = Instant::now();

    // Create stress players
    for i in 0..STRESS_PLAYERS {
        let profile = create_random_profile(&format!("stress_gb_player_{}", i), i as u64);
        game_builder.start_player_session(&format!("stress_gb_player_{}", i), profile)?;
    }

    // Perform many operations
    for op in 0..STRESS_OPERATIONS {
        let player_idx = op % STRESS_PLAYERS;
        let player_id = format!("stress_gb_player_{}", player_idx);

        match op % 8 {
            0 => {
                // Record interaction
                let interaction = create_random_interaction(op as u64);
                game_builder.record_player_interaction(&player_id, interaction)?;
            },
            1 => {
                // Get recommendations
                let _ = game_builder.get_ai_recommendations(&player_id)?;
            },
            2 => {
                // Adjust difficulty
                let _ = game_builder.adjust_difficulty(&player_id, 0.5)?;
            },
            3 => {
                // Generate content
                let mut params = HashMap::new();
                params.insert("seed".to_string(), op as f32);
                let _ = game_builder.generate_procedural_content(&player_id, "structure", params)?;
            },
            4 => {
                // Update AI systems
                let _ = game_builder.update_ai_systems(0.016)?;
            },
            5 => {
                // Analyze behavior
                let _ = game_builder.analyze_player_behavior(&player_id)?;
            },
            6 => {
                // Create tutorial
                let _ = game_builder.create_ai_tutorial(&player_id, "advanced_building")?;
            },
            7 => {
                // Generate quest
                let _ = game_builder.generate_dynamic_quest(&player_id, 0.6)?;
            },
            _ => unreachable!(),
        }
    }

    let total_time = start_time.elapsed();
    println!("Completed {} GameBuilder stress operations with {} players in {:?}",
             STRESS_OPERATIONS, STRESS_PLAYERS, total_time);

    // Should handle stress operations efficiently
    assert!(total_time.as_secs() < 30, "GameBuilder stress test too slow: {:?}", total_time);

    // Verify system stability
    let global_analytics = game_builder.get_global_analytics();
    assert!(!global_analytics.is_empty());
    assert!(*global_analytics.get("total_players").unwrap() >= STRESS_PLAYERS as f32);

    Ok(())
}

#[test]
fn test_edge_case_robustness() -> RobinResult<()> {
    let mut manager = GameAIManager::new();

    // Test with extreme profile values
    let extreme_profile = PlayerProfile {
        player_id: "extreme_edge_case".to_string(),
        skill_level: f32::NAN, // Invalid skill level
        engagement_level: -1.0, // Invalid engagement
        learning_style: "".to_string(), // Empty learning style
        preferences: HashMap::new(),
        play_history: Vec::new(),
        performance_metrics: HashMap::new(),
        last_session: std::time::SystemTime::now(),
        total_playtime: Duration::from_secs(0), // Zero playtime
        achievement_progress: HashMap::new(),
    };

    // System should handle invalid data gracefully
    let result = manager.start_player_session("extreme_edge_case", extreme_profile);
    // Either succeeds (by normalizing data) or fails gracefully
    if result.is_ok() {
        let metrics = manager.get_player_metrics("extreme_edge_case");
        assert!(metrics.is_some());
    }

    Ok(())
}

#[test]
fn test_resource_cleanup() -> RobinResult<()> {
    let mut manager = GameAIManager::new();

    // Create and destroy many sessions to test cleanup
    for batch in 0..10 {
        // Create batch of players
        for i in 0..50 {
            let player_id = format!("cleanup_batch_{}_player_{}", batch, i);
            let profile = create_random_profile(&player_id, (batch * 50 + i) as u64);
            manager.start_player_session(&player_id, profile)?;
        }

        // Process some interactions
        for i in 0..50 {
            let player_id = format!("cleanup_batch_{}_player_{}", batch, i);
            let interaction = create_random_interaction((batch * 50 + i) as u64);
            manager.process_player_interaction(&player_id, &interaction)?;
        }

        // Clean up batch
        for i in 0..50 {
            let player_id = format!("cleanup_batch_{}_player_{}", batch, i);
            let _ = manager.end_player_session(&player_id)?;
        }

        // Verify cleanup
        for i in 0..50 {
            let player_id = format!("cleanup_batch_{}_player_{}", batch, i);
            assert!(manager.get_player_metrics(&player_id).is_none());
        }
    }

    // System should be clean after all batches
    let global_metrics = manager.get_global_metrics();
    // Should have low active player count
    assert!(*global_metrics.get("active_sessions").unwrap_or(&0.0) < 5.0);

    Ok(())
}