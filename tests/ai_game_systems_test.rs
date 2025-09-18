// Comprehensive tests for AI Game Systems
// Tests player analytics, dynamic adaptation, procedural generation, and game balancing

use robin::engine::ai_game::*;
use robin::engine::error::RobinResult;
use std::collections::HashMap;

#[test]
fn test_game_ai_manager_initialization() {
    let manager = GameAIManager::new();
    assert!(manager.is_enabled());

    let status = manager.get_status();
    assert!(status.contains_key("player_analytics"));
    assert!(status.contains_key("dynamic_adaptation"));
    assert!(status.contains_key("procedural_generation"));
    assert!(status.contains_key("game_balancing"));
}

#[test]
fn test_player_session_lifecycle() -> RobinResult<()> {
    let mut manager = GameAIManager::new();

    let test_profile = PlayerProfile {
        player_id: "test_player_001".to_string(),
        skill_level: 0.5,
        engagement_level: 0.7,
        learning_style: "Visual".to_string(),
        preferences: HashMap::new(),
        play_history: Vec::new(),
        performance_metrics: HashMap::new(),
        last_session: std::time::SystemTime::now(),
        total_playtime: std::time::Duration::from_secs(3600),
        achievement_progress: HashMap::new(),
    };

    // Start session
    manager.start_player_session("test_player_001", test_profile.clone())?;

    // Verify session exists
    assert!(manager.get_player_metrics("test_player_001").is_some());

    // Process some interactions
    let interaction = PlayerInteraction {
        interaction_type: "building".to_string(),
        timestamp: std::time::SystemTime::now(),
        duration: std::time::Duration::from_secs(30),
        success_rate: 0.8,
        complexity_level: 0.6,
        context: HashMap::new(),
        performance_data: HashMap::new(),
    };

    let events = manager.process_player_interaction("test_player_001", &interaction)?;
    assert!(!events.is_empty());

    // End session
    let final_profile = manager.end_player_session("test_player_001")?;
    assert_eq!(final_profile.player_id, "test_player_001");

    Ok(())
}

#[test]
fn test_difficulty_adjustment() -> RobinResult<()> {
    let mut manager = GameAIManager::new();

    let test_profile = PlayerProfile {
        player_id: "difficulty_test".to_string(),
        skill_level: 0.3, // Low skill
        engagement_level: 0.9, // High engagement
        learning_style: "Kinesthetic".to_string(),
        preferences: HashMap::new(),
        play_history: Vec::new(),
        performance_metrics: HashMap::new(),
        last_session: std::time::SystemTime::now(),
        total_playtime: std::time::Duration::from_secs(1800),
        achievement_progress: HashMap::new(),
    };

    manager.start_player_session("difficulty_test", test_profile)?;

    // Request difficulty adjustment for flow state
    let new_difficulty = manager.adjust_difficulty("difficulty_test", 0.75)?;

    // Should be easier than default since skill is low
    assert!(new_difficulty < 0.5);

    Ok(())
}

#[test]
fn test_procedural_content_generation() -> RobinResult<()> {
    let mut manager = GameAIManager::new();

    let test_profile = PlayerProfile {
        player_id: "content_test".to_string(),
        skill_level: 0.7,
        engagement_level: 0.6,
        learning_style: "Analytical".to_string(),
        preferences: HashMap::new(),
        play_history: Vec::new(),
        performance_metrics: HashMap::new(),
        last_session: std::time::SystemTime::now(),
        total_playtime: std::time::Duration::from_secs(7200),
        achievement_progress: HashMap::new(),
    };

    manager.start_player_session("content_test", test_profile)?;

    let mut parameters = HashMap::new();
    parameters.insert("complexity".to_string(), 0.7);
    parameters.insert("size".to_string(), 100.0);

    // Test world generation
    let world_content = manager.generate_content("content_test", "world", parameters.clone())?;
    assert!(!world_content.is_empty());

    // Test structure generation
    let structure_content = manager.generate_content("content_test", "structure", parameters.clone())?;
    assert!(!structure_content.is_empty());

    // Test challenge generation
    let challenge_content = manager.generate_content("content_test", "challenge", parameters)?;
    assert!(!challenge_content.is_empty());

    Ok(())
}

#[test]
fn test_ai_recommendations() -> RobinResult<()> {
    let mut manager = GameAIManager::new();

    let test_profile = PlayerProfile {
        player_id: "recommendations_test".to_string(),
        skill_level: 0.8,
        engagement_level: 0.4, // Low engagement - should trigger recommendations
        learning_style: "Social".to_string(),
        preferences: HashMap::new(),
        play_history: Vec::new(),
        performance_metrics: HashMap::new(),
        last_session: std::time::SystemTime::now(),
        total_playtime: std::time::Duration::from_secs(5400),
        achievement_progress: HashMap::new(),
    };

    manager.start_player_session("recommendations_test", test_profile)?;

    let recommendations = manager.get_recommendations("recommendations_test")?;
    assert!(!recommendations.is_empty());

    // Verify recommendation structure
    for rec in &recommendations {
        assert!(!rec.player_id.is_empty());
        assert!(!rec.title.is_empty());
        assert!(!rec.description.is_empty());
        assert!(rec.confidence > 0.0 && rec.confidence <= 1.0);
        assert!(!rec.implementation_steps.is_empty());
    }

    Ok(())
}

#[test]
fn test_behavior_analysis() -> RobinResult<()> {
    let mut manager = GameAIManager::new();

    let test_profile = PlayerProfile {
        player_id: "behavior_test".to_string(),
        skill_level: 0.6,
        engagement_level: 0.7,
        learning_style: "Mixed".to_string(),
        preferences: HashMap::new(),
        play_history: Vec::new(),
        performance_metrics: HashMap::new(),
        last_session: std::time::SystemTime::now(),
        total_playtime: std::time::Duration::from_secs(10800),
        achievement_progress: HashMap::new(),
    };

    manager.start_player_session("behavior_test", test_profile)?;

    // Simulate multiple interactions
    let interactions = vec![
        ("building", 0.9, 0.7),
        ("exploration", 0.6, 0.4),
        ("problem_solving", 0.8, 0.8),
        ("social_interaction", 0.5, 0.3),
    ];

    for (action_type, success_rate, complexity) in interactions {
        let interaction = PlayerInteraction {
            interaction_type: action_type.to_string(),
            timestamp: std::time::SystemTime::now(),
            duration: std::time::Duration::from_secs(60),
            success_rate,
            complexity_level: complexity,
            context: HashMap::new(),
            performance_data: HashMap::new(),
        };

        manager.process_player_interaction("behavior_test", &interaction)?;
    }

    let behavior_analysis = manager.analyze_behavior_patterns("behavior_test")?;
    assert!(!behavior_analysis.is_empty());

    // Should detect building preference
    assert!(behavior_analysis.get("building_preference").unwrap_or(&0.0) > &0.5);

    Ok(())
}

#[test]
fn test_game_balancing_system() -> RobinResult<()> {
    let mut manager = GameAIManager::new();

    // Test system rebalancing
    let focus_areas = vec![
        "difficulty_progression".to_string(),
        "resource_distribution".to_string(),
        "challenge_variety".to_string(),
    ];

    let balance_results = manager.rebalance_systems(focus_areas)?;
    assert!(!balance_results.is_empty());

    // Verify balance metrics are within reasonable ranges
    for (metric, value) in balance_results {
        assert!(value >= 0.0 && value <= 1.0, "Balance metric {} = {} out of range", metric, value);
    }

    Ok(())
}

#[test]
fn test_adaptive_tutorial_creation() -> RobinResult<()> {
    let mut manager = GameAIManager::new();

    let beginner_profile = PlayerProfile {
        player_id: "tutorial_test".to_string(),
        skill_level: 0.1, // Complete beginner
        engagement_level: 0.8,
        learning_style: "Step-by-step".to_string(),
        preferences: HashMap::new(),
        play_history: Vec::new(),
        performance_metrics: HashMap::new(),
        last_session: std::time::SystemTime::now(),
        total_playtime: std::time::Duration::from_secs(300),
        achievement_progress: HashMap::new(),
    };

    manager.start_player_session("tutorial_test", beginner_profile)?;

    let tutorial_steps = manager.create_adaptive_tutorial("tutorial_test", "building_basics")?;
    assert!(!tutorial_steps.is_empty());

    // Beginner should get more detailed steps
    assert!(tutorial_steps.len() >= 5, "Tutorial should have at least 5 steps for beginners");

    Ok(())
}

#[test]
fn test_dynamic_quest_generation() -> RobinResult<()> {
    let mut manager = GameAIManager::new();

    let experienced_profile = PlayerProfile {
        player_id: "quest_test".to_string(),
        skill_level: 0.9, // Very experienced
        engagement_level: 0.6,
        learning_style: "Challenge-seeking".to_string(),
        preferences: HashMap::new(),
        play_history: Vec::new(),
        performance_metrics: HashMap::new(),
        last_session: std::time::SystemTime::now(),
        total_playtime: std::time::Duration::from_secs(36000),
        achievement_progress: HashMap::new(),
    };

    manager.start_player_session("quest_test", experienced_profile)?;

    // Request a challenging quest
    let quest = manager.generate_dynamic_quest("quest_test", 0.8)?;
    assert!(!quest.is_empty());

    // Should contain quest objectives and be appropriately challenging
    assert!(quest.contains("challenge") || quest.contains("advanced") || quest.contains("complex"));

    Ok(())
}

#[test]
fn test_global_analytics() {
    let manager = GameAIManager::new();

    let global_metrics = manager.get_global_metrics();

    // Should have basic metrics even with no players
    assert!(global_metrics.contains_key("total_players"));
    assert!(global_metrics.contains_key("average_session_length"));
    assert!(global_metrics.contains_key("system_performance"));

    // Values should be reasonable
    assert!(*global_metrics.get("total_players").unwrap() >= 0.0);
    assert!(*global_metrics.get("system_performance").unwrap() >= 0.0);
}

#[test]
fn test_ai_system_performance() -> RobinResult<()> {
    let mut manager = GameAIManager::new();

    // Measure update performance
    let start_time = std::time::Instant::now();

    for _ in 0..100 {
        let _events = manager.update(0.016)?; // 60 FPS
    }

    let elapsed = start_time.elapsed();

    // Should be able to handle 100 updates in reasonable time (< 100ms)
    assert!(elapsed.as_millis() < 100, "AI system updates too slow: {}ms", elapsed.as_millis());

    Ok(())
}

#[test]
fn test_data_export_import() -> RobinResult<()> {
    let mut manager = GameAIManager::new();

    let test_profile = PlayerProfile {
        player_id: "export_test".to_string(),
        skill_level: 0.6,
        engagement_level: 0.7,
        learning_style: "Visual".to_string(),
        preferences: HashMap::new(),
        play_history: Vec::new(),
        performance_metrics: HashMap::new(),
        last_session: std::time::SystemTime::now(),
        total_playtime: std::time::Duration::from_secs(3600),
        achievement_progress: HashMap::new(),
    };

    manager.start_player_session("export_test", test_profile)?;

    // Export data
    let exported_data = manager.export_player_data("export_test", false)?;
    assert!(!exported_data.is_empty());

    // Import preferences
    let mut preferences = HashMap::new();
    preferences.insert("difficulty_preference".to_string(), 0.7);
    preferences.insert("tutorial_speed".to_string(), 1.2);

    manager.import_player_preferences("export_test", preferences)?;

    // Verify import worked
    let updated_profile = manager.end_player_session("export_test")?;
    assert!(!updated_profile.preferences.is_empty());

    Ok(())
}