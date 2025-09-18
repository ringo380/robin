// Performance benchmarks for AI Game Systems
// Measures throughput, latency, and resource usage of AI components

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use robin::engine::ai_game::*;
use std::collections::HashMap;
use std::time::Duration;

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
        total_playtime: Duration::from_secs(3600),
        achievement_progress: HashMap::new(),
    }
}

fn create_test_interaction() -> PlayerInteraction {
    PlayerInteraction {
        interaction_type: "building".to_string(),
        timestamp: std::time::SystemTime::now(),
        duration: Duration::from_secs(30),
        success_rate: 0.8,
        complexity_level: 0.6,
        context: HashMap::new(),
        performance_data: HashMap::new(),
    }
}

fn bench_ai_manager_initialization(c: &mut Criterion) {
    c.bench_function("ai_manager_init", |b| {
        b.iter(|| {
            let manager = black_box(GameAIManager::new());
            drop(manager);
        })
    });
}

fn bench_player_session_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("player_session");

    for session_count in [1, 10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("create_sessions", session_count),
            session_count,
            |b, &session_count| {
                b.iter(|| {
                    let mut manager = GameAIManager::new();
                    for i in 0..session_count {
                        let profile = create_test_profile(&format!("player_{}", i));
                        let _ = manager.start_player_session(&format!("player_{}", i), profile);
                    }
                    black_box(manager);
                })
            },
        );
    }
    group.finish();
}

fn bench_interaction_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("interaction_processing");

    for interaction_count in [1, 10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("process_interactions", interaction_count),
            interaction_count,
            |b, &interaction_count| {
                b.iter_setup(
                    || {
                        let mut manager = GameAIManager::new();
                        let profile = create_test_profile("benchmark_player");
                        let _ = manager.start_player_session("benchmark_player", profile);
                        manager
                    },
                    |mut manager| {
                        for _ in 0..interaction_count {
                            let interaction = create_test_interaction();
                            let _ = manager.process_player_interaction("benchmark_player", &interaction);
                        }
                        black_box(manager);
                    },
                )
            },
        );
    }
    group.finish();
}

fn bench_difficulty_adjustment(c: &mut Criterion) {
    c.bench_function("difficulty_adjustment", |b| {
        b.iter_setup(
            || {
                let mut manager = GameAIManager::new();
                let profile = create_test_profile("difficulty_player");
                let _ = manager.start_player_session("difficulty_player", profile);
                manager
            },
            |mut manager| {
                let result = manager.adjust_difficulty("difficulty_player", black_box(0.75));
                black_box(result);
            },
        )
    });
}

fn bench_content_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("content_generation");

    let content_types = ["world", "structure", "challenge", "tool"];

    for content_type in content_types.iter() {
        group.bench_with_input(
            BenchmarkId::new("generate_content", content_type),
            content_type,
            |b, &content_type| {
                b.iter_setup(
                    || {
                        let mut manager = GameAIManager::new();
                        let profile = create_test_profile("content_player");
                        let _ = manager.start_player_session("content_player", profile);

                        let mut parameters = HashMap::new();
                        parameters.insert("complexity".to_string(), 0.5);
                        parameters.insert("size".to_string(), 50.0);

                        (manager, parameters)
                    },
                    |(mut manager, parameters)| {
                        let result = manager.generate_content("content_player", content_type, parameters);
                        black_box(result);
                    },
                )
            },
        );
    }
    group.finish();
}

fn bench_recommendation_generation(c: &mut Criterion) {
    c.bench_function("recommendation_generation", |b| {
        b.iter_setup(
            || {
                let mut manager = GameAIManager::new();
                let profile = create_test_profile("rec_player");
                let _ = manager.start_player_session("rec_player", profile);

                // Add some interaction history for better recommendations
                for _ in 0..10 {
                    let interaction = create_test_interaction();
                    let _ = manager.process_player_interaction("rec_player", &interaction);
                }

                manager
            },
            |manager| {
                let result = manager.get_recommendations("rec_player");
                black_box(result);
            },
        )
    });
}

fn bench_behavior_analysis(c: &mut Criterion) {
    c.bench_function("behavior_analysis", |b| {
        b.iter_setup(
            || {
                let mut manager = GameAIManager::new();
                let profile = create_test_profile("analysis_player");
                let _ = manager.start_player_session("analysis_player", profile);

                // Create diverse interaction history
                let interaction_types = ["building", "exploration", "combat", "puzzle", "social"];
                for interaction_type in interaction_types.iter() {
                    for _ in 0..5 {
                        let mut interaction = create_test_interaction();
                        interaction.interaction_type = interaction_type.to_string();
                        let _ = manager.process_player_interaction("analysis_player", &interaction);
                    }
                }

                manager
            },
            |manager| {
                let result = manager.analyze_behavior_patterns("analysis_player");
                black_box(result);
            },
        )
    });
}

fn bench_system_updates(c: &mut Criterion) {
    let mut group = c.benchmark_group("system_updates");

    for player_count in [1, 5, 10, 25].iter() {
        group.bench_with_input(
            BenchmarkId::new("update_with_players", player_count),
            player_count,
            |b, &player_count| {
                b.iter_setup(
                    || {
                        let mut manager = GameAIManager::new();
                        for i in 0..player_count {
                            let profile = create_test_profile(&format!("update_player_{}", i));
                            let _ = manager.start_player_session(&format!("update_player_{}", i), profile);
                        }
                        manager
                    },
                    |mut manager| {
                        let result = manager.update(black_box(0.016)); // 60 FPS
                        black_box(result);
                    },
                )
            },
        );
    }
    group.finish();
}

fn bench_game_balancing(c: &mut Criterion) {
    c.bench_function("game_balancing", |b| {
        b.iter_setup(
            || {
                let mut manager = GameAIManager::new();

                // Add several players with different profiles
                for i in 0..10 {
                    let mut profile = create_test_profile(&format!("balance_player_{}", i));
                    profile.skill_level = (i as f32) / 10.0; // Varied skill levels
                    let _ = manager.start_player_session(&format!("balance_player_{}", i), profile);
                }

                manager
            },
            |mut manager| {
                let focus_areas = vec![
                    "difficulty_progression".to_string(),
                    "resource_distribution".to_string(),
                    "challenge_variety".to_string(),
                ];
                let result = manager.rebalance_systems(focus_areas);
                black_box(result);
            },
        )
    });
}

fn bench_concurrent_operations(c: &mut Criterion) {
    c.bench_function("concurrent_ai_operations", |b| {
        b.iter_setup(
            || {
                let mut manager = GameAIManager::new();
                let profile = create_test_profile("concurrent_player");
                let _ = manager.start_player_session("concurrent_player", profile);
                manager
            },
            |mut manager| {
                // Simulate multiple operations happening in a single frame
                let interaction = create_test_interaction();
                let _ = manager.process_player_interaction("concurrent_player", &interaction);

                let _ = manager.adjust_difficulty("concurrent_player", 0.75);

                let mut parameters = HashMap::new();
                parameters.insert("complexity".to_string(), 0.6);
                let _ = manager.generate_content("concurrent_player", "structure", parameters);

                let _ = manager.get_recommendations("concurrent_player");

                let _ = manager.update(0.016);

                black_box(manager);
            },
        )
    });
}

fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");

    for data_points in [100, 500, 1000, 5000].iter() {
        group.bench_with_input(
            BenchmarkId::new("interaction_history", data_points),
            data_points,
            |b, &data_points| {
                b.iter_setup(
                    || {
                        let mut manager = GameAIManager::new();
                        let profile = create_test_profile("memory_player");
                        let _ = manager.start_player_session("memory_player", profile);
                        manager
                    },
                    |mut manager| {
                        // Add many interactions to test memory handling
                        for _ in 0..data_points {
                            let interaction = create_test_interaction();
                            let _ = manager.process_player_interaction("memory_player", &interaction);
                        }

                        // Perform operations that use the accumulated data
                        let _ = manager.analyze_behavior_patterns("memory_player");
                        let _ = manager.get_recommendations("memory_player");

                        black_box(manager);
                    },
                )
            },
        );
    }
    group.finish();
}

fn bench_adaptive_features(c: &mut Criterion) {
    let mut group = c.benchmark_group("adaptive_features");

    group.bench_function("tutorial_creation", |b| {
        b.iter_setup(
            || {
                let mut manager = GameAIManager::new();
                let profile = create_test_profile("tutorial_player");
                let _ = manager.start_player_session("tutorial_player", profile);
                manager
            },
            |mut manager| {
                let result = manager.create_adaptive_tutorial("tutorial_player", "building_basics");
                black_box(result);
            },
        )
    });

    group.bench_function("quest_generation", |b| {
        b.iter_setup(
            || {
                let mut manager = GameAIManager::new();
                let profile = create_test_profile("quest_player");
                let _ = manager.start_player_session("quest_player", profile);
                manager
            },
            |mut manager| {
                let result = manager.generate_dynamic_quest("quest_player", black_box(0.7));
                black_box(result);
            },
        )
    });

    group.finish();
}

criterion_group!(
    ai_benchmarks,
    bench_ai_manager_initialization,
    bench_player_session_creation,
    bench_interaction_processing,
    bench_difficulty_adjustment,
    bench_content_generation,
    bench_recommendation_generation,
    bench_behavior_analysis,
    bench_system_updates,
    bench_game_balancing,
    bench_concurrent_operations,
    bench_memory_usage,
    bench_adaptive_features
);

criterion_main!(ai_benchmarks);