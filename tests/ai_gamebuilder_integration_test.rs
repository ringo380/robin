// Integration tests for AI systems with GameBuilder API
// Tests the complete flow from GameBuilder to AI systems and back

use robin::engine::GameBuilder;
use robin::engine::ai_game::*;
use robin::engine::error::RobinResult;
use std::collections::HashMap;

#[test]
fn test_gamebuilder_ai_initialization() {
    let game_builder = GameBuilder::new();

    // Verify AI system is initialized
    let ai_status = game_builder.get_ai_status();
    assert!(!ai_status.is_empty());
    assert!(ai_status.contains_key("player_analytics"));
}

#[test]
fn test_gamebuilder_player_session_flow() -> RobinResult<()> {
    let mut game_builder = GameBuilder::new();

    let test_profile = PlayerProfile {
        player_id: "integration_player".to_string(),
        skill_level: 0.6,
        engagement_level: 0.8,
        learning_style: "Hands-on".to_string(),
        preferences: HashMap::new(),
        play_history: Vec::new(),
        performance_metrics: HashMap::new(),
        last_session: std::time::SystemTime::now(),
        total_playtime: std::time::Duration::from_secs(1800),
        achievement_progress: HashMap::new(),
    };

    // Start player session through GameBuilder API
    game_builder.start_player_session("integration_player", test_profile.clone())?;

    // Verify session was created
    let analytics = game_builder.get_player_analytics("integration_player");
    assert!(analytics.is_some());

    // Process some interactions
    let interaction = PlayerInteraction {
        interaction_type: "construction".to_string(),
        timestamp: std::time::SystemTime::now(),
        duration: std::time::Duration::from_secs(45),
        success_rate: 0.85,
        complexity_level: 0.7,
        context: HashMap::new(),
        performance_data: HashMap::new(),
    };

    let events = game_builder.record_player_interaction("integration_player", interaction)?;
    assert!(!events.is_empty());

    // Get AI recommendations
    let recommendations = game_builder.get_ai_recommendations("integration_player")?;
    assert!(!recommendations.is_empty());

    // End session
    let final_profile = game_builder.end_player_session("integration_player")?;
    assert_eq!(final_profile.player_id, "integration_player");

    Ok(())
}

#[test]
fn test_gamebuilder_difficulty_adaptation() -> RobinResult<()> {
    let mut game_builder = GameBuilder::new();

    // Enable AI features
    game_builder.enable_ai_features(vec!["analytics", "adaptation"]);

    let struggling_player = PlayerProfile {
        player_id: "struggling_player".to_string(),
        skill_level: 0.2, // Low skill
        engagement_level: 0.3, // Low engagement - struggling
        learning_style: "Visual".to_string(),
        preferences: HashMap::new(),
        play_history: Vec::new(),
        performance_metrics: HashMap::new(),
        last_session: std::time::SystemTime::now(),
        total_playtime: std::time::Duration::from_secs(600),
        achievement_progress: HashMap::new(),
    };

    game_builder.start_player_session("struggling_player", struggling_player)?;

    // Simulate failed interactions
    for _ in 0..5 {
        let failed_interaction = PlayerInteraction {
            interaction_type: "puzzle_solving".to_string(),
            timestamp: std::time::SystemTime::now(),
            duration: std::time::Duration::from_secs(120),
            success_rate: 0.2, // Low success
            complexity_level: 0.8, // Too complex
            context: HashMap::new(),
            performance_data: HashMap::new(),
        };

        game_builder.record_player_interaction("struggling_player", failed_interaction)?;
    }

    // AI should recommend easier difficulty
    let new_difficulty = game_builder.adjust_difficulty("struggling_player", 0.6)?;
    assert!(new_difficulty < 0.5, "Difficulty should be reduced for struggling player");

    Ok(())
}

#[test]
fn test_gamebuilder_content_generation_flow() -> RobinResult<()> {
    let mut game_builder = GameBuilder::new();

    let creative_player = PlayerProfile {
        player_id: "creative_player".to_string(),
        skill_level: 0.8,
        engagement_level: 0.9,
        learning_style: "Experimental".to_string(),
        preferences: HashMap::new(),
        play_history: Vec::new(),
        performance_metrics: HashMap::new(),
        last_session: std::time::SystemTime::now(),
        total_playtime: std::time::Duration::from_secs(7200),
        achievement_progress: HashMap::new(),
    };

    game_builder.start_player_session("creative_player", creative_player)?;

    // Generate various types of content
    let mut world_params = HashMap::new();
    world_params.insert("biome_variety".to_string(), 0.9);
    world_params.insert("terrain_complexity".to_string(), 0.8);

    let world_content = game_builder.generate_procedural_content(
        "creative_player",
        "world",
        world_params
    )?;
    assert!(!world_content.is_empty());

    let mut structure_params = HashMap::new();
    structure_params.insert("architectural_style".to_string(), 0.7);
    structure_params.insert("functional_complexity".to_string(), 0.6);

    let structure_content = game_builder.generate_procedural_content(
        "creative_player",
        "structure",
        structure_params
    )?;
    assert!(!structure_content.is_empty());

    Ok(())
}

#[test]
fn test_gamebuilder_personalized_experience() -> RobinResult<()> {
    let mut game_builder = GameBuilder::new();

    let explorer_player = PlayerProfile {
        player_id: "explorer_player".to_string(),
        skill_level: 0.7,
        engagement_level: 0.8,
        learning_style: "Discovery-based".to_string(),
        preferences: HashMap::new(),
        play_history: Vec::new(),
        performance_metrics: HashMap::new(),
        last_session: std::time::SystemTime::now(),
        total_playtime: std::time::Duration::from_secs(5400),
        achievement_progress: HashMap::new(),
    };

    game_builder.start_player_session("explorer_player", explorer_player)?;

    // Simulate exploration-heavy interactions
    let exploration_interactions = vec![
        "terrain_discovery",
        "resource_scouting",
        "area_mapping",
        "secret_finding",
    ];

    for interaction_type in exploration_interactions {
        let interaction = PlayerInteraction {
            interaction_type: interaction_type.to_string(),
            timestamp: std::time::SystemTime::now(),
            duration: std::time::Duration::from_secs(90),
            success_rate: 0.75,
            complexity_level: 0.5,
            context: HashMap::new(),
            performance_data: HashMap::new(),
        };

        game_builder.record_player_interaction("explorer_player", interaction)?;
    }

    // Create personalized experience
    let experience = game_builder.create_personalized_experience(
        "explorer_player",
        "exploration_adventure"
    )?;
    assert!(!experience.is_empty());
    assert!(experience.contains("exploration") || experience.contains("discovery"));

    Ok(())
}

#[test]
fn test_gamebuilder_adaptive_tutorials() -> RobinResult<()> {
    let mut game_builder = GameBuilder::new();

    // Test for different skill levels
    let beginner_profile = PlayerProfile {
        player_id: "beginner".to_string(),
        skill_level: 0.1,
        engagement_level: 0.9,
        learning_style: "Step-by-step".to_string(),
        preferences: HashMap::new(),
        play_history: Vec::new(),
        performance_metrics: HashMap::new(),
        last_session: std::time::SystemTime::now(),
        total_playtime: std::time::Duration::from_secs(300),
        achievement_progress: HashMap::new(),
    };

    let expert_profile = PlayerProfile {
        player_id: "expert".to_string(),
        skill_level: 0.95,
        engagement_level: 0.7,
        learning_style: "Advanced".to_string(),
        preferences: HashMap::new(),
        play_history: Vec::new(),
        performance_metrics: HashMap::new(),
        last_session: std::time::SystemTime::now(),
        total_playtime: std::time::Duration::from_secs(36000),
        achievement_progress: HashMap::new(),
    };

    game_builder.start_player_session("beginner", beginner_profile)?;
    game_builder.start_player_session("expert", expert_profile)?;

    let beginner_tutorial = game_builder.create_ai_tutorial("beginner", "basic_building")?;
    let expert_tutorial = game_builder.create_ai_tutorial("expert", "basic_building")?;

    // Beginner should get more detailed tutorial
    assert!(beginner_tutorial.len() > expert_tutorial.len());

    Ok(())
}

#[test]
fn test_gamebuilder_quest_generation() -> RobinResult<()> {
    let mut game_builder = GameBuilder::new();

    let challenge_seeker = PlayerProfile {
        player_id: "challenge_seeker".to_string(),
        skill_level: 0.85,
        engagement_level: 0.6, // Needs more challenge
        learning_style: "Challenge-driven".to_string(),
        preferences: HashMap::new(),
        play_history: Vec::new(),
        performance_metrics: HashMap::new(),
        last_session: std::time::SystemTime::now(),
        total_playtime: std::time::Duration::from_secs(14400),
        achievement_progress: HashMap::new(),
    };

    game_builder.start_player_session("challenge_seeker", challenge_seeker)?;

    // Generate quests of different difficulties
    let easy_quest = game_builder.generate_dynamic_quest("challenge_seeker", 0.3)?;
    let hard_quest = game_builder.generate_dynamic_quest("challenge_seeker", 0.9)?;

    assert!(!easy_quest.is_empty());
    assert!(!hard_quest.is_empty());
    assert_ne!(easy_quest, hard_quest);

    Ok(())
}

#[test]
fn test_gamebuilder_behavior_analysis_integration() -> RobinResult<()> {
    let mut game_builder = GameBuilder::new();

    let analytical_player = PlayerProfile {
        player_id: "analytical_player".to_string(),
        skill_level: 0.7,
        engagement_level: 0.8,
        learning_style: "Analytical".to_string(),
        preferences: HashMap::new(),
        play_history: Vec::new(),
        performance_metrics: HashMap::new(),
        last_session: std::time::SystemTime::now(),
        total_playtime: std::time::Duration::from_secs(9000),
        achievement_progress: HashMap::new(),
    };

    game_builder.start_player_session("analytical_player", analytical_player)?;

    // Create diverse behavior pattern
    let behavior_patterns = vec![
        ("planning", 0.9, 0.8, 300),
        ("optimization", 0.85, 0.9, 450),
        ("experimentation", 0.6, 0.7, 200),
        ("analysis", 0.95, 0.9, 600),
    ];

    for (action_type, success_rate, complexity, duration) in behavior_patterns {
        let interaction = PlayerInteraction {
            interaction_type: action_type.to_string(),
            timestamp: std::time::SystemTime::now(),
            duration: std::time::Duration::from_secs(duration),
            success_rate,
            complexity_level: complexity,
            context: HashMap::new(),
            performance_data: HashMap::new(),
        };

        game_builder.record_player_interaction("analytical_player", interaction)?;
    }

    let behavior_analysis = game_builder.analyze_player_behavior("analytical_player")?;
    assert!(!behavior_analysis.is_empty());

    // Should detect analytical preferences
    assert!(behavior_analysis.get("analytical_preference").unwrap_or(&0.0) > &0.5);

    Ok(())
}

#[test]
fn test_gamebuilder_game_balancing_integration() -> RobinResult<()> {
    let mut game_builder = GameBuilder::new();

    // Enable all AI features
    game_builder.enable_ai_features(vec!["analytics", "adaptation", "generation", "balancing"]);

    // Create multiple players with different profiles
    let player_profiles = vec![
        ("casual_player", 0.3, 0.8),
        ("hardcore_player", 0.9, 0.7),
        ("learning_player", 0.5, 0.9),
        ("expert_player", 0.95, 0.6),
    ];

    for (player_id, skill, engagement) in player_profiles {
        let profile = PlayerProfile {
            player_id: player_id.to_string(),
            skill_level: skill,
            engagement_level: engagement,
            learning_style: "Mixed".to_string(),
            preferences: HashMap::new(),
            play_history: Vec::new(),
            performance_metrics: HashMap::new(),
            last_session: std::time::SystemTime::now(),
            total_playtime: std::time::Duration::from_secs(3600),
            achievement_progress: HashMap::new(),
        };

        game_builder.start_player_session(player_id, profile)?;
    }

    // Trigger rebalancing
    let focus_areas = vec![
        "difficulty_progression".to_string(),
        "content_variety".to_string(),
        "engagement_optimization".to_string(),
    ];

    let balance_results = game_builder.rebalance_game_systems(focus_areas)?;
    assert!(!balance_results.is_empty());

    // Verify reasonable balance values
    for (metric, value) in balance_results {
        assert!(
            value >= 0.0 && value <= 1.0,
            "Balance metric {} = {} out of range",
            metric,
            value
        );
    }

    Ok(())
}

#[test]
fn test_gamebuilder_ai_system_updates() -> RobinResult<()> {
    let mut game_builder = GameBuilder::new();

    let test_profile = PlayerProfile {
        player_id: "update_test_player".to_string(),
        skill_level: 0.6,
        engagement_level: 0.7,
        learning_style: "Adaptive".to_string(),
        preferences: HashMap::new(),
        play_history: Vec::new(),
        performance_metrics: HashMap::new(),
        last_session: std::time::SystemTime::now(),
        total_playtime: std::time::Duration::from_secs(2700),
        achievement_progress: HashMap::new(),
    };

    game_builder.start_player_session("update_test_player", test_profile)?;

    // Simulate game loop updates
    for frame in 0..100 {
        let delta_time = 0.016; // 60 FPS

        let ai_events = game_builder.update_ai_systems(delta_time)?;

        // AI should generate events periodically
        if frame % 60 == 0 {
            // Once per second
            assert!(!ai_events.is_empty() || frame == 0);
        }
    }

    Ok(())
}

#[test]
fn test_gamebuilder_data_export_import() -> RobinResult<()> {
    let mut game_builder = GameBuilder::new();

    let test_profile = PlayerProfile {
        player_id: "data_test_player".to_string(),
        skill_level: 0.7,
        engagement_level: 0.8,
        learning_style: "Data-driven".to_string(),
        preferences: HashMap::new(),
        play_history: Vec::new(),
        performance_metrics: HashMap::new(),
        last_session: std::time::SystemTime::now(),
        total_playtime: std::time::Duration::from_secs(5400),
        achievement_progress: HashMap::new(),
    };

    game_builder.start_player_session("data_test_player", test_profile)?;

    // Generate some activity
    for i in 0..10 {
        let interaction = PlayerInteraction {
            interaction_type: format!("action_{}", i % 3),
            timestamp: std::time::SystemTime::now(),
            duration: std::time::Duration::from_secs(30 + i * 5),
            success_rate: 0.7 + (i as f32) * 0.02,
            complexity_level: 0.5 + (i as f32) * 0.03,
            context: HashMap::new(),
            performance_data: HashMap::new(),
        };

        game_builder.record_player_interaction("data_test_player", interaction)?;
    }

    // Export data
    let exported_data = game_builder.export_player_data("data_test_player", false)?;
    assert!(!exported_data.is_empty());

    // Import preferences
    let mut preferences = HashMap::new();
    preferences.insert("preferred_complexity".to_string(), 0.8);
    preferences.insert("preferred_session_length".to_string(), 45.0);

    game_builder.import_player_preferences("data_test_player", preferences)?;

    // Verify preferences were imported
    let analytics = game_builder.get_player_analytics("data_test_player");
    assert!(analytics.is_some());

    Ok(())
}

#[test]
fn test_gamebuilder_global_analytics() {
    let mut game_builder = GameBuilder::new();

    // Add multiple players
    for i in 0..5 {
        let profile = PlayerProfile {
            player_id: format!("global_player_{}", i),
            skill_level: (i as f32) / 5.0,
            engagement_level: 0.5 + (i as f32) * 0.1,
            learning_style: "Mixed".to_string(),
            preferences: HashMap::new(),
            play_history: Vec::new(),
            performance_metrics: HashMap::new(),
            last_session: std::time::SystemTime::now(),
            total_playtime: std::time::Duration::from_secs(1800 * (i + 1) as u64),
            achievement_progress: HashMap::new(),
        };

        let _ = game_builder.start_player_session(&format!("global_player_{}", i), profile);
    }

    let global_analytics = game_builder.get_global_analytics();
    assert!(!global_analytics.is_empty());

    // Should track total players
    assert!(*global_analytics.get("total_players").unwrap() >= 5.0);
}