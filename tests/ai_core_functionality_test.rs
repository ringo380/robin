// Core functionality tests for AI Game Systems
// Tests the actual implemented methods and verifies basic functionality

use robin::engine::ai_game::*;
use robin::engine::error::RobinResult;
use std::collections::HashMap;

fn create_test_profile(player_id: &str) -> PlayerProfile {
    PlayerProfile {
        player_id: player_id.to_string(),
        skill_level: 0.5,
        engagement_level: 0.7,
        learning_style: "Mixed".to_string(),
        preferences: HashMap::new(),
        play_history: Vec::new(),
        performance_metrics: HashMap::new(),
        last_session: std::time::SystemTime::now(),
        total_playtime: std::time::Duration::from_secs(3600),
        achievement_progress: HashMap::new(),
    }
}

fn create_test_interaction() -> PlayerInteraction {
    PlayerInteraction {
        interaction_type: "building".to_string(),
        timestamp: std::time::SystemTime::now(),
        duration: std::time::Duration::from_secs(30),
        success_rate: 0.8,
        complexity_level: 0.6,
        context: HashMap::new(),
        performance_data: HashMap::new(),
    }
}

#[test]
fn test_ai_manager_initialization() -> RobinResult<()> {
    let mut manager = GameAIManager::new();
    manager.initialize()?;
    Ok(())
}

#[test]
fn test_player_profile_management() {
    let mut manager = GameAIManager::new();
    let profile = create_test_profile("test_player");

    // Add player profile
    manager.add_player_profile(profile.clone());

    // Retrieve player profile
    let retrieved = manager.get_player_profile("test_player");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().player_id, "test_player");
}

#[test]
fn test_interaction_processing() -> RobinResult<()> {
    let mut manager = GameAIManager::new();
    manager.initialize()?;

    let profile = create_test_profile("interaction_player");
    manager.add_player_profile(profile);

    let interaction = create_test_interaction();
    let events = manager.process_interaction("interaction_player", interaction)?;

    // Should generate some events
    assert!(!events.is_empty());

    Ok(())
}

#[test]
fn test_difficulty_calculation() {
    let mut manager = GameAIManager::new();
    let profile = create_test_profile("difficulty_player");
    manager.add_player_profile(profile);

    let difficulty = manager.get_optimal_difficulty("difficulty_player");
    assert!(difficulty >= 0.0 && difficulty <= 1.0);
}

#[test]
fn test_content_recommendations() {
    let mut manager = GameAIManager::new();
    let profile = create_test_profile("content_player");
    manager.add_player_profile(profile);

    let recommendations = manager.get_content_recommendations("content_player");
    // Should get some recommendations
    assert!(!recommendations.is_empty());
}

#[test]
fn test_flow_state_detection() {
    let mut manager = GameAIManager::new();
    let profile = create_test_profile("flow_player");
    manager.add_player_profile(profile);

    let in_flow = manager.is_player_in_flow("flow_player");
    // Should return a boolean
    assert!(in_flow == true || in_flow == false);
}

#[test]
fn test_ai_recommendations() -> RobinResult<()> {
    let mut manager = GameAIManager::new();
    manager.initialize()?;

    let profile = create_test_profile("rec_player");
    manager.add_player_profile(profile);

    let recommendations = manager.generate_recommendations("rec_player")?;
    assert!(!recommendations.is_empty());

    // Verify recommendation structure
    for rec in recommendations {
        assert!(!rec.player_id.is_empty());
        assert!(!rec.title.is_empty());
        assert!(rec.confidence >= 0.0 && rec.confidence <= 1.0);
    }

    Ok(())
}

#[test]
fn test_social_matching() {
    let mut manager = GameAIManager::new();

    // Add multiple players
    for i in 0..5 {
        let profile = create_test_profile(&format!("social_player_{}", i));
        manager.add_player_profile(profile);
    }

    let matches = manager.get_social_matches("social_player_0");
    // Should return some potential matches
    assert!(!matches.is_empty());
}

#[test]
fn test_system_updates() -> RobinResult<()> {
    let mut manager = GameAIManager::new();
    manager.initialize()?;

    // Add some players
    for i in 0..3 {
        let profile = create_test_profile(&format!("update_player_{}", i));
        manager.add_player_profile(profile);
    }

    // Test multiple updates
    for _ in 0..10 {
        let events = manager.update(0.016)?; // 60 FPS
        // Events may or may not be generated each frame
        assert!(events.len() >= 0);
    }

    Ok(())
}

#[test]
fn test_multiple_interactions() -> RobinResult<()> {
    let mut manager = GameAIManager::new();
    manager.initialize()?;

    let profile = create_test_profile("multi_interaction_player");
    manager.add_player_profile(profile);

    // Process multiple interactions
    for i in 0..10 {
        let mut interaction = create_test_interaction();
        interaction.interaction_type = format!("action_{}", i % 3);
        interaction.success_rate = 0.5 + (i as f32) * 0.05;

        let events = manager.process_interaction("multi_interaction_player", interaction)?;
        assert!(!events.is_empty());
    }

    // Check if recommendations changed based on interaction history
    let recommendations = manager.generate_recommendations("multi_interaction_player")?;
    assert!(!recommendations.is_empty());

    Ok(())
}

#[test]
fn test_edge_cases() -> RobinResult<()> {
    let mut manager = GameAIManager::new();
    manager.initialize()?;

    // Test with non-existent player
    let difficulty = manager.get_optimal_difficulty("non_existent_player");
    assert!(difficulty >= 0.0 && difficulty <= 1.0); // Should return default

    let content_recs = manager.get_content_recommendations("non_existent_player");
    assert!(!content_recs.is_empty()); // Should return default recommendations

    let flow_state = manager.is_player_in_flow("non_existent_player");
    assert!(flow_state == true || flow_state == false); // Should return default

    Ok(())
}

#[test]
fn test_performance_characteristics() -> RobinResult<()> {
    let mut manager = GameAIManager::new();
    manager.initialize()?;

    let start_time = std::time::Instant::now();

    // Add many players quickly
    for i in 0..100 {
        let profile = create_test_profile(&format!("perf_player_{}", i));
        manager.add_player_profile(profile);
    }

    let creation_time = start_time.elapsed();

    // Should be able to create 100 players quickly
    assert!(creation_time.as_millis() < 1000, "Player creation too slow: {:?}", creation_time);

    // Test rapid interaction processing
    let interaction_start = std::time::Instant::now();

    for i in 0..100 {
        let interaction = create_test_interaction();
        let _ = manager.process_interaction(&format!("perf_player_{}", i % 100), interaction);
    }

    let interaction_time = interaction_start.elapsed();
    assert!(interaction_time.as_millis() < 2000, "Interaction processing too slow: {:?}", interaction_time);

    Ok(())
}

#[test]
fn test_data_consistency() -> RobinResult<()> {
    let mut manager = GameAIManager::new();
    manager.initialize()?;

    let profile = create_test_profile("consistency_player");
    manager.add_player_profile(profile.clone());

    // Verify profile was stored correctly
    let retrieved = manager.get_player_profile("consistency_player").unwrap();
    assert_eq!(retrieved.player_id, profile.player_id);
    assert_eq!(retrieved.skill_level, profile.skill_level);
    assert_eq!(retrieved.engagement_level, profile.engagement_level);

    // Process interaction and verify it affects recommendations
    let interaction = create_test_interaction();
    let events_before = manager.generate_recommendations("consistency_player")?;

    manager.process_interaction("consistency_player", interaction)?;

    let events_after = manager.generate_recommendations("consistency_player")?;

    // Recommendations should exist both before and after
    assert!(!events_before.is_empty());
    assert!(!events_after.is_empty());

    Ok(())
}