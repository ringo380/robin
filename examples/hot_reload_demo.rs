use robin::engine::{GameBuilder, assets::{AssetType, HotReloadEvent}};
use std::time::Duration;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔥 Robin Engine Hot Reload Demo");
    println!("================================");
    println!();
    
    // Initialize env_logger to see hot reload events
    env_logger::init();
    
    // Initialize the game builder with hot reload enabled
    let mut game = GameBuilder::new();

    // Create demo assets directory structure if it doesn't exist
    setup_demo_assets()?;

    // Enable hot reload and register assets
    game.enable_hot_reload(true)
        .auto_register_assets()
        .register_asset("player_sprite", "examples/assets/textures/player.png")
        .register_asset("background_music", "examples/assets/audio/bgm.ogg")
        .register_asset("game_config", "examples/assets/config/settings.json");

    // Add a custom reload callback for textures
    game.add_reload_callback("player_sprite", |event| {
        match event {
            HotReloadEvent::AssetModified { asset_id, asset_type, .. } => {
                if *asset_type == AssetType::Texture {
                    println!("🎨 Texture '{}' was reloaded! Visual changes should be visible.", asset_id);
                }
            }
            HotReloadEvent::ReloadComplete { asset_id, reload_time } => {
                println!("✅ '{}' reload completed in {:?}", asset_id, reload_time);
            }
            HotReloadEvent::ReloadFailed { asset_id, error } => {
                println!("❌ '{}' reload failed: {}", asset_id, error);
            }
            _ => {}
        }
    });

    // Start hot reloading system
    game.start_hot_reload();

    println!("🚀 Hot reload system started!");
    println!();
    print_instructions();

    // Demo state
    let mut frame_count = 0;
    let mut last_stats_print = std::time::Instant::now();
    let mut should_quit = false;

    // Main game loop
    while !should_quit {
        frame_count += 1;

        // Simple input manager for update call
        let input = robin::engine::input::InputManager::new();

        // Update hot reload system (this processes file changes)
        game.update(1.0/60.0, &input);

        // Print stats every 5 seconds
        if last_stats_print.elapsed() > Duration::from_secs(5) {
            print_hot_reload_stats(&game);
            last_stats_print = std::time::Instant::now();
        }

        // Simulate some game rendering
        if frame_count % 3600 == 0 { // Every minute at 60 FPS
            println!("🎮 Game running... Frame: {}", frame_count);
        }

        // Check for manual reload requests
        should_quit = handle_user_input(&mut game);

        // Sleep to maintain 60 FPS
        std::thread::sleep(Duration::from_millis(16));
    }

    println!("🏁 Demo completed!");
    Ok(())
}

fn setup_demo_assets() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;

    // Create directory structure
    fs::create_dir_all("examples/assets/textures")?;
    fs::create_dir_all("examples/assets/audio")?;
    fs::create_dir_all("examples/assets/config")?;

    // Create dummy texture file (simple PNG-like header)
    if !PathBuf::from("examples/assets/textures/player.png").exists() {
        let dummy_png = create_dummy_png_data();
        fs::write("examples/assets/textures/player.png", dummy_png)?;
        println!("📁 Created dummy player.png texture");
    }

    // Create dummy audio file
    if !PathBuf::from("examples/assets/audio/bgm.ogg").exists() {
        let dummy_audio = b"OggS\x00\x02\x00\x00\x00\x00\x00\x00\x00\x00"; // Dummy OGG header
        fs::write("examples/assets/audio/bgm.ogg", dummy_audio)?;
        println!("📁 Created dummy bgm.ogg audio file");
    }

    // Create config file
    let config_content = r#"{
    "player_speed": 5.0,
    "jump_height": 10.0,
    "gravity": 9.8,
    "version": "1.0.0",
    "debug_mode": false
}"#;
    fs::write("examples/assets/config/settings.json", config_content)?;
    println!("📁 Created game settings.json config");

    Ok(())
}

fn create_dummy_png_data() -> Vec<u8> {
    // Simple 1x1 red pixel PNG
    vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk start
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1 dimensions
        0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, 0xDE, // IHDR chunk end
        0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, // IDAT chunk start
        0x08, 0x99, 0x01, 0x01, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01, // Red pixel data
        0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82, // IEND chunk
    ]
}

fn print_instructions() {
    println!("📋 Instructions:");
    println!("  1. Edit files in examples/assets/ while this demo runs");
    println!("  2. Try modifying examples/assets/config/settings.json");
    println!("  3. Replace examples/assets/textures/player.png with a different image");
    println!("  4. Press 'r' + Enter to force reload player_sprite");
    println!("  5. Press 's' + Enter to show current stats");
    println!("  6. Press 'q' + Enter to quit");
    println!("  7. Watch the console for hot reload events!");
    println!();
}

fn print_hot_reload_stats(game: &GameBuilder) {
    println!("📊 Hot Reload Statistics:");
    println!("{}", game.get_hot_reload_stats());
    println!();
}

fn handle_user_input(game: &mut GameBuilder) -> bool {
    use std::io::{self, BufRead, BufReader};
    use std::sync::mpsc;
    use std::thread;
    
    // Simple non-blocking input check using available_input
    static mut INPUT_RECEIVER: Option<std::sync::mpsc::Receiver<String>> = None;
    static mut INPUT_STARTED: bool = false;
    
    unsafe {
        if !INPUT_STARTED {
            let (tx, rx) = mpsc::channel();
            INPUT_RECEIVER = Some(rx);
            INPUT_STARTED = true;
            
            thread::spawn(move || {
                let stdin = io::stdin();
                for line in stdin.lock().lines() {
                    if let Ok(line) = line {
                        let _ = tx.send(line);
                    }
                }
            });
        }
        
        if let Some(ref receiver) = INPUT_RECEIVER {
            if let Ok(line) = receiver.try_recv() {
                let input = line.trim().to_lowercase();
                
                match input.as_str() {
                    "r" => {
                        println!("🔄 Forcing reload of player_sprite...");
                        game.force_reload_asset("player_sprite");
                    }
                    "s" => {
                        print_hot_reload_stats(game);
                    }
                    "q" => {
                        println!("👋 Quitting demo...");
                        return true;
                    }
                    _ => {
                        if !input.is_empty() {
                            println!("❓ Unknown command: '{}'. Try 'r', 's', or 'q'", input);
                        }
                    }
                }
            }
        }
    }
    
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_asset_creation() {
        // Test that we can create the demo asset structure
        let temp_dir = std::env::temp_dir().join("robin_hot_reload_test");
        std::env::set_var("ROBIN_ASSETS_PATH", temp_dir.to_str().unwrap());
        
        // This would test the asset creation in a temp directory
        assert!(true); // Placeholder for now
    }

    #[test]
    fn test_dummy_png_creation() {
        let png_data = create_dummy_png_data();
        assert!(png_data.starts_with(&[0x89, 0x50, 0x4E, 0x47])); // PNG signature
        assert!(png_data.len() > 20); // Should have reasonable size
    }
}