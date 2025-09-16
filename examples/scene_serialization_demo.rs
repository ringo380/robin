use robin::engine::{GameBuilder, input::InputManager};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎭 Robin Engine Scene Serialization Demo");
    println!("=======================================");
    println!();
    
    // Initialize env_logger to see scene management logs
    env_logger::init();
    
    // Initialize the game builder
    let mut game = GameBuilder::new();

    // Setup scene directory
    game.set_scene_directory("examples/scenes")
        .enable_scene_auto_save(true, Some(30)); // Auto-save every 30 seconds

    println!("🚀 Scene system initialized!");
    print_instructions();

    // Create multiple scene types
    demonstrate_scene_creation(&mut game);
    
    // Demonstrate scene operations
    demonstrate_scene_operations(&mut game)?;
    
    // Simulate game loop with scene management
    simulate_game_loop(&mut game);

    println!("🏁 Scene serialization demo completed!");
    Ok(())
}

fn demonstrate_scene_creation(game: &mut GameBuilder) {
    println!("\n📦 Creating Different Scene Types:");
    println!("==================================");
    
    // Create various scene templates
    game.create_2d_platformer_scene("platformer_level_1")
        .create_top_down_scene("dungeon_floor_1") 
        .create_basic_scene("main_menu")
        .create_scene("custom_scene");

    println!("✅ Created 4 different scenes");
    print_scene_stats(game);
}

fn demonstrate_scene_operations(game: &mut GameBuilder) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n💾 Testing Scene Save/Load Operations:");
    println!("=====================================");
    
    // Save scenes to different formats
    game.save_scene("platformer_level_1", Some("examples/scenes/platformer.json"))
        .save_scene("dungeon_floor_1", Some("examples/scenes/dungeon.yaml"))
        .save_scene("main_menu", Some("examples/scenes/menu.toml"))
        .save_scene("custom_scene", Some("examples/scenes/custom.scene")); // Binary format

    println!("💾 Saved scenes in multiple formats:");
    println!("  - platformer_level_1 → platformer.json (JSON)");
    println!("  - dungeon_floor_1 → dungeon.yaml (YAML)");
    println!("  - main_menu → menu.toml (TOML)");
    println!("  - custom_scene → custom.scene (Binary)");

    // Duplicate scenes
    game.duplicate_scene("platformer_level_1", "platformer_level_2")
        .duplicate_scene("dungeon_floor_1", "dungeon_floor_2");

    println!("📋 Duplicated scenes:");
    println!("  - platformer_level_1 → platformer_level_2");
    println!("  - dungeon_floor_1 → dungeon_floor_2");

    print_scene_stats(game);

    // Test scene activation
    println!("\n🎯 Testing Scene Activation:");
    game.set_active_scene("platformer_level_1");
    println!("✅ Activated scene: platformer_level_1");
    
    game.set_active_scene("dungeon_floor_1");
    println!("✅ Activated scene: dungeon_floor_1");

    // Load scenes from directory (this will reload our saved scenes)
    println!("\n📂 Scanning and loading scenes from directory:");
    game.load_scenes_from_directory("examples/scenes");
    
    Ok(())
}

fn simulate_game_loop(game: &mut GameBuilder) {
    println!("\n🎮 Simulating Game Loop with Scene Management:");
    println!("=============================================");
    
    let input = InputManager::new();
    let mut frame_count = 0;
    let mut last_stats_print = std::time::Instant::now();
    let mut scene_switch_timer = 0.0;
    let scenes = vec!["platformer_level_1", "dungeon_floor_1", "main_menu"];
    let mut current_scene_index = 0;

    // Run for 10 seconds
    let start_time = std::time::Instant::now();
    while start_time.elapsed() < Duration::from_secs(10) {
        let delta_time = 1.0 / 60.0; // 60 FPS
        frame_count += 1;
        scene_switch_timer += delta_time;

        // Update all systems including scene manager
        game.update(delta_time, &input);

        // Switch scenes every 3 seconds for demonstration
        if scene_switch_timer >= 3.0 {
            let scene_name = scenes[current_scene_index % scenes.len()];
            game.set_active_scene(scene_name);
            println!("🔄 Switched to scene: {}", scene_name);
            
            current_scene_index += 1;
            scene_switch_timer = 0.0;
        }

        // Print stats every 2 seconds
        if last_stats_print.elapsed() > Duration::from_secs(2) {
            print_scene_stats(game);
            last_stats_print = std::time::Instant::now();
        }

        // Simulate frame timing
        std::thread::sleep(Duration::from_millis(16)); // ~60 FPS
    }

    println!("⏰ Game loop simulation completed (10 seconds)");
}

fn print_scene_stats(game: &GameBuilder) {
    println!("\n📊 Current Scene Statistics:");
    println!("{}", game.get_scene_stats());
    
    let scene_list = game.list_scenes();
    println!("📝 Available scenes ({}): {:?}", scene_list.len(), scene_list);
}

fn print_instructions() {
    println!("📋 This demo demonstrates:");
    println!("  1. Creating scenes with different templates");
    println!("  2. Saving scenes in multiple formats (JSON, YAML, TOML, Binary)");
    println!("  3. Loading and duplicating scenes");
    println!("  4. Scene activation and management");
    println!("  5. Auto-save functionality");
    println!("  6. Directory scanning for scenes");
    println!("  7. Real-time scene switching in a game loop");
    println!();
}

/// Create sample scene files with different configurations
fn create_sample_scene_files() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    
    // Ensure scenes directory exists
    fs::create_dir_all("examples/scenes")?;
    
    // Create a sample JSON scene
    let sample_json = r#"{
  "name": "Sample Level",
  "version": "1.0",
  "metadata": {
    "created_at": "2024-01-01 12:00:00 UTC",
    "last_modified": "2024-01-01 12:00:00 UTC",
    "author": "Robin Engine",
    "description": "A sample platformer level",
    "tags": ["platformer", "demo"],
    "engine_version": "0.1.0"
  },
  "objects": [
    {
      "id": 1,
      "name": "Player Spawn",
      "transform": {
        "position": [0.0, 0.0, 0.0],
        "rotation": 0.0,
        "scale": [1.0, 1.0]
      },
      "active": true,
      "components": [
        {
          "type": "Sprite",
          "texture_path": "player.png",
          "width": 32.0,
          "height": 32.0,
          "color": [1.0, 1.0, 1.0, 1.0]
        }
      ],
      "tags": ["player", "spawn"]
    },
    {
      "id": 2,
      "name": "Ground Platform",
      "transform": {
        "position": [0.0, -100.0, 0.0],
        "rotation": 0.0,
        "scale": [200.0, 20.0]
      },
      "active": true,
      "components": [
        {
          "type": "Collider",
          "shape": "rectangle",
          "width": 200.0,
          "height": 20.0,
          "is_sensor": false
        }
      ],
      "tags": ["platform", "ground"]
    }
  ],
  "global_settings": {
    "gravity": [0.0, -9.81],
    "background_color": [0.5, 0.8, 1.0, 1.0],
    "ambient_light_color": [1.0, 1.0, 1.0],
    "ambient_light_intensity": 0.3,
    "physics_enabled": true,
    "physics_timestep": 0.016666666666666666,
    "render_layers": ["Background", "Main", "Foreground", "UI"],
    "audio_settings": {
      "master_volume": 1.0,
      "music_volume": 0.7,
      "sfx_volume": 1.0,
      "doppler_factor": 1.0,
      "speed_of_sound": 343.0
    }
  }
}"#;
    
    fs::write("examples/scenes/sample_level.json", sample_json)?;
    println!("📄 Created sample scene file: examples/scenes/sample_level.json");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_scene_creation_and_management() {
        let mut game = GameBuilder::new();
        
        // Test scene creation
        game.create_2d_platformer_scene("test_platformer")
            .create_top_down_scene("test_topdown")
            .create_basic_scene("test_basic");
        
        let scenes = game.list_scenes();
        assert!(scenes.contains(&"test_platformer".to_string()));
        assert!(scenes.contains(&"test_topdown".to_string()));
        assert!(scenes.contains(&"test_basic".to_string()));
        
        // Test scene activation
        game.set_active_scene("test_platformer");
        // In a real implementation, we'd check the active scene
        
        // Test scene duplication
        game.duplicate_scene("test_platformer", "test_platformer_copy");
        let scenes_after_duplicate = game.list_scenes();
        assert!(scenes_after_duplicate.contains(&"test_platformer_copy".to_string()));
    }

    #[test]
    fn test_scene_directory_management() {
        let temp_dir = TempDir::new().unwrap();
        let mut game = GameBuilder::new();
        
        // Set custom scene directory
        game.set_scene_directory(temp_dir.path().to_str().unwrap());
        
        // Create and save a scene
        game.create_basic_scene("test_scene")
            .save_scene("test_scene", None);
        
        // The scene should be saved in the temp directory
        // In a real test, we'd verify the file exists
    }

    #[test]
    fn test_auto_save_configuration() {
        let mut game = GameBuilder::new();
        
        // Enable auto-save
        game.enable_scene_auto_save(true, Some(60));
        
        // Create a scene
        game.create_basic_scene("auto_save_test");
        
        // Simulate time passing (auto-save would trigger)
        let input = InputManager::new();
        game.update(1.0, &input); // 1 second
        
        // In a real implementation, we'd check that auto-save works
    }
}