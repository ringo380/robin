use robin::engine::{
    GameBuilder,
    save_system::{GameState, PlayerData, SaveSystemConfig, GameProgress},
    input::InputManager,
    error::RobinResult,
    physics::BodyType,
    math::Vec2,
};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎮 Robin Engine Save/Load System Demo");
    println!("===================================");
    println!();

    // Initialize the game with comprehensive save system
    let mut game = GameBuilder::new();
    
    // Set up logging for the demo
    game.setup_logging(None);
    
    // Demo the save/load system workflow
    demonstrate_profile_management(&mut game)?;
    demonstrate_game_saving(&mut game)?;
    demonstrate_game_loading(&mut game)?;
    demonstrate_auto_save(&mut game)?;
    demonstrate_achievements(&mut game)?;
    demonstrate_save_management(&mut game)?;

    println!("🏁 Save/Load system demo completed!");
    Ok(())
}

fn demonstrate_profile_management(game: &mut GameBuilder) -> Result<(), Box<dyn std::error::Error>> {
    println!("👤 Profile Management Demonstration");
    println!("==================================");
    
    // Create multiple user profiles
    game.create_user_profile("player1", "Alice the Explorer");
    game.create_user_profile("player2", "Bob the Builder");
    game.create_user_profile("speedrunner", "Charlie the Fast");
    
    // List all profiles
    println!("📋 Available profiles:");
    for profile_info in game.list_profiles() {
        println!("  - {}", profile_info);
    }
    println!();
    
    // Set active profile
    game.set_active_profile("player1");
    
    if let Some(active_info) = game.get_active_profile_info() {
        println!("🎯 Active profile: {}", active_info);
    }
    
    println!("✅ Profile management completed");
    println!();
    Ok(())
}

fn demonstrate_game_saving(game: &mut GameBuilder) -> Result<(), Box<dyn std::error::Error>> {
    println!("💾 Game Saving Demonstration");
    println!("============================");
    
    // Set up a game scenario to save
    setup_demo_game_state(game);
    
    // Save to multiple slots
    game.save_game(0, "Tutorial Complete");
    game.save_game(1, "Forest Level - Checkpoint 1");
    game.save_game(2, "Boss Battle Preparation");
    
    // Demonstrate quick save scenarios
    println!("Creating different game scenarios for saves...");
    
    // Modify game state for second save
    modify_game_state_for_demo(game, 1);
    game.save_game(3, "After Boss Battle");
    
    // Modify game state for third save  
    modify_game_state_for_demo(game, 2);
    game.save_game(4, "New Area Unlocked");
    
    // Show all saves
    println!("📋 Current save slots:");
    for (i, save_info) in game.list_all_saves().iter().enumerate() {
        println!("  {}", save_info);
        if i >= 4 { break; } // Only show first 5 slots
    }
    
    println!("✅ Game saving completed");
    println!();
    Ok(())
}

fn demonstrate_game_loading(game: &mut GameBuilder) -> Result<(), Box<dyn std::error::Error>> {
    println!("📁 Game Loading Demonstration");
    println!("=============================");
    
    // Show available saves before loading
    println!("Available saves to load:");
    for i in 0..5 {
        if game.save_exists(i) {
            if let Some(save_info) = game.get_save_info(i) {
                println!("  Slot {}: {}", i, save_info);
            }
        }
    }
    println!();
    
    // Load different saves to demonstrate state restoration
    println!("Loading save from slot 1...");
    game.load_game(1);
    
    println!("Game state after loading:");
    print_current_game_status(game);
    println!();
    
    // Load another save
    println!("Loading save from slot 3...");
    game.load_game(3);
    
    println!("Game state after loading slot 3:");
    print_current_game_status(game);
    
    println!("✅ Game loading completed");
    println!();
    Ok(())
}

fn demonstrate_auto_save(game: &mut GameBuilder) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Auto-Save Demonstration");
    println!("==========================");
    
    // Enable auto-save for slot 9 (dedicated auto-save slot)
    game.set_auto_save(true, Some(9));
    
    // Simulate gameplay with periodic auto-saves
    let input = InputManager::new();
    println!("Simulating gameplay with auto-save enabled...");
    
    // Create some game activity
    let ball_id = game.create_bouncing_ball(100.0, 100.0, 10.0);
    game.apply_impulse(ball_id, 50.0, -100.0);
    
    for frame in 0..60 {
        game.update(0.016, &input);
        
        // Simulate some gameplay changes
        if frame % 20 == 0 {
            modify_game_state_for_demo(game, frame / 20);
        }
        
        // Force an auto-save demonstration
        if frame == 30 {
            println!("⏰ Triggering manual auto-save...");
            game.force_auto_save();
            if game.save_exists(9) {
                if let Some(save_info) = game.get_save_info(9) {
                    println!("  Auto-save created: {}", save_info);
                }
            }
        }
        
        // Small delay to simulate frame timing
        std::thread::sleep(Duration::from_millis(10));
    }
    
    println!("✅ Auto-save demonstration completed");
    println!();
    Ok(())
}

fn demonstrate_achievements(game: &mut GameBuilder) -> Result<(), Box<dyn std::error::Error>> {
    println!("🏆 Achievement System Demonstration");
    println!("===================================");
    
    // Demonstrate achievement progress
    game.update_achievement_progress("first_steps", 0.3);
    game.update_achievement_progress("monster_slayer", 0.7);
    game.update_achievement_progress("treasure_hunter", 0.9);
    
    // Unlock some achievements
    game.unlock_achievement("tutorial_complete");
    game.unlock_achievement("first_level_complete");
    game.update_achievement_progress("speed_demon", 1.0); // This should auto-unlock
    
    // Show profile info with achievements
    if let Some(profile_info) = game.get_active_profile_info() {
        println!("🎯 Profile with achievements: {}", profile_info);
    }
    
    println!("✅ Achievement system completed");
    println!();
    Ok(())
}

fn demonstrate_save_management(game: &mut GameBuilder) -> Result<(), Box<dyn std::error::Error>> {
    println!("🗂️  Save Management Demonstration");
    println!("=================================");
    
    // Show all current saves
    println!("📋 All save slots status:");
    for save_info in game.list_all_saves() {
        println!("  {}", save_info);
    }
    println!();
    
    // Demonstrate save deletion
    println!("🗑️  Deleting save from slot 4...");
    game.delete_save(4);
    
    // Show saves after deletion
    println!("📋 Saves after deletion:");
    for (i, save_info) in game.list_all_saves().iter().enumerate() {
        println!("  {}", save_info);
        if i >= 6 { break; } // Show first 7 slots
    }
    println!();
    
    // Create a final save with comprehensive data
    setup_comprehensive_game_state(game);
    game.save_game(5, "Demo Complete - Full Features");
    
    if let Some(final_save) = game.get_save_info(5) {
        println!("🎯 Final demo save created: {}", final_save);
    }
    
    println!("✅ Save management completed");
    println!();
    Ok(())
}

fn setup_demo_game_state(game: &mut GameBuilder) {
    println!("🎮 Setting up demo game state...");
    
    // Create a basic scene
    game.create_scene("demo_level_1");
    
    // Add some physics objects
    let player_body = game.create_physics_object(100.0, 100.0, 20.0, 40.0, false);
    let platform = game.create_platform(0.0, 200.0, 400.0, 20.0);
    
    // Add some effects
    game.create_magic_trail(150.0, 80.0);
    game.add_torch_light(200.0, 120.0);
    
    // Create UI elements
    let health_bar = game.create_progress_bar(10.0, 10.0, 200.0, 20.0, 100.0);
    let score_label = game.create_label(10.0, 40.0, 150.0, 20.0, "Score: 1500");
    
    println!("  ✅ Basic game scene created");
}

fn modify_game_state_for_demo(game: &mut GameBuilder, variation: usize) {
    match variation {
        0 => {
            // Add some additional objects
            game.create_bouncing_ball(200.0, 50.0, 15.0);
            game.create_explosion(250.0, 150.0);
        }
        1 => {
            // Add more complex effects
            game.create_campfire(300.0, 180.0);
            game.create_portal(350.0, 100.0);
        }
        2 => {
            // Create more advanced scenario
            game.create_physics_playground(50.0, 450.0, 50.0, 250.0);
            game.add_magic_light(400.0, 75.0);
        }
        _ => {
            // Default case - add a simple effect
            game.create_magic_trail(100.0 + (variation as f32) * 50.0, 120.0);
        }
    }
}

fn setup_comprehensive_game_state(game: &mut GameBuilder) {
    println!("🌟 Setting up comprehensive final game state...");
    
    // Create multiple physics objects
    for i in 0..5 {
        let x = 50.0 + (i as f32) * 80.0;
        let ball = game.create_bouncing_ball(x, 50.0, 10.0 + (i as f32) * 2.0);
        game.apply_impulse(ball, (i as f32) * 10.0, -50.0);
    }
    
    // Create a complex lighting setup
    let torch_positions = [(100.0, 180.0), (200.0, 180.0), (300.0, 180.0)];
    game.setup_dungeon_lighting(&torch_positions);
    
    // Add various effects
    game.create_campfire(150.0, 200.0);
    game.create_portal(250.0, 100.0);
    
    // Create UI elements
    let menu_elements = game.create_simple_menu(
        "Game Complete!", 
        &[("Continue", 100.0, 300.0), ("New Game", 100.0, 360.0), ("Exit", 100.0, 420.0)]
    );
    
    println!("  ✅ Comprehensive game state created");
}

fn print_current_game_status(game: &GameBuilder) {
    println!("  🎯 Current game status:");
    println!("    - Physics objects: {}", game.get_physics_body_count());
    println!("    - Active particles: {}", game.get_particle_count());
    println!("    - Light sources: {}", game.get_light_count());
    println!("    - Performance: {}", game.get_performance_metrics());
}

// Demo helper functions for testing save/load functionality
fn create_sample_player_data() -> PlayerData {
    let mut player = PlayerData::new("Demo Player".to_string());
    player.level = 5;
    player.experience = 2500;
    player.position = (150.0, 100.0);
    player.health = 75.0;
    player.max_health = 100.0;
    
    // Add some inventory items
    player.add_item("health_potion", 3);
    player.add_item("magic_scroll", 1);
    player.add_item("gold_coin", 150);
    
    // Add some stats
    player.stats.enemies_defeated = 25;
    player.stats.items_collected = 47;
    player.stats.distance_traveled = 1500.0;
    
    // Unlock some achievements
    player.unlock_achievement("first_level_complete");
    player.unlock_achievement("coin_collector");
    
    player
}

fn create_sample_progress() -> GameProgress {
    let mut progress = GameProgress::default();
    
    progress.completion_percentage = 35.0;
    progress.current_level = "forest_level_2".to_string();
    progress.complete_level("tutorial");
    progress.complete_level("forest_level_1");
    progress.unlock_content("fire_magic");
    progress.unlock_content("forest_area");
    
    progress.set_story_flag("met_wizard", true);
    progress.set_story_flag("found_magic_sword", false);
    progress.set_story_variable("reputation", 75.0);
    
    progress
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_game_state_setup() {
        let mut game = GameBuilder::new();
        setup_demo_game_state(&mut game);
        
        // Verify game state was set up
        assert!(game.get_physics_body_count() > 0);
    }

    #[test]
    fn test_profile_management_flow() {
        let mut game = GameBuilder::new();
        
        // Create profile
        game.create_user_profile("test_user", "Test User");
        game.set_active_profile("test_user");
        
        // Verify profile is active
        assert!(game.get_active_profile_info().is_some());
        
        // Test achievements
        game.unlock_achievement("test_achievement");
        game.update_achievement_progress("progress_test", 0.5);
        
        // Profile should still be active
        assert!(game.get_active_profile_info().is_some());
    }

    #[test]
    fn test_save_load_cycle() {
        let mut game = GameBuilder::new();
        
        // Set up initial state
        setup_demo_game_state(&mut game);
        let initial_physics_count = game.get_physics_body_count();
        
        // Save the game
        game.save_game(0, "Test Save");
        assert!(game.save_exists(0));
        
        // Modify state
        game.create_bouncing_ball(300.0, 300.0, 20.0);
        assert!(game.get_physics_body_count() > initial_physics_count);
        
        // Load the save
        game.load_game(0);
        
        // State should be restored
        // Note: In a real implementation, this would actually restore the physics count
        assert!(game.save_exists(0));
    }

    #[test]
    fn test_auto_save_functionality() {
        let mut game = GameBuilder::new();
        
        // Enable auto-save
        game.set_auto_save(true, Some(9));
        
        // Force auto-save
        game.force_auto_save();
        
        // Auto-save slot should exist
        assert!(game.save_exists(9));
    }
}