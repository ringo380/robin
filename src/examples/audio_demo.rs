use crate::engine::{
    GameBuilder,
    input::InputManager,
    math::Vec2,
};
use std::time::Instant;

pub async fn run_audio_demo() {
    env_logger::init();
    log::info!("Starting audio system demo");

    let mut game_builder = GameBuilder::new();
    
    log::info!("=== ROBIN ENGINE AUDIO SYSTEM DEMO ===");
    log::info!("🔊 Spatial audio with distance attenuation");
    log::info!("🎵 Background music management with crossfading");
    log::info!("🎛️  Master, SFX, and music volume controls");
    log::info!("📍 3D positioned sound effects");
    log::info!("🔄 Audio loop management and cleanup");
    log::info!("🎭 Integration with particle effects and animations");
    
    // Set up listener position (usually the player/camera)
    game_builder.set_audio_listener(300.0, 200.0);
    
    // Test volume controls
    game_builder.set_master_volume(0.8)
                .set_music_volume(0.6)
                .set_sfx_volume(0.7);
    
    let start_time = Instant::now();
    let mut frame_count = 0;
    
    // Demo simulation loop
    loop {
        let current_time = Instant::now();
        let delta_time = 0.016; // ~60 FPS
        frame_count += 1;
        
        // Update all game systems including audio
        let input = InputManager::new();
        let _animation_updates = game_builder.update(delta_time, &input);
        
        // Test different audio scenarios
        match frame_count {
            60 => {
                // 1 second in - test basic sound
                log::info!("🔊 Testing basic sound effects...");
                game_builder.play_sound("beep");
            },
            120 => {
                // 2 seconds - test spatial audio
                log::info!("📍 Testing spatial audio...");
                game_builder.play_sound_at("coin", 100.0, 100.0, 1.0);  // Far left
                game_builder.play_sound_at("coin", 500.0, 100.0, 1.0);  // Far right
                game_builder.play_sound_at("coin", 300.0, 50.0, 1.0);   // Close to listener
            },
            180 => {
                // 3 seconds - create explosion with visual and audio
                log::info!("💥 Creating explosion with synchronized audio-visual effects...");
                let explosion_x = 250.0;
                let explosion_y = 150.0;
                
                // This will automatically play explosion sound at the position
                game_builder.create_explosion(explosion_x, explosion_y);
                
                // Add dramatic lighting
                game_builder.add_light(explosion_x, explosion_y, (1.0, 0.5, 0.2), 1.5);
            },
            240 => {
                // 4 seconds - test music
                log::info!("🎵 Testing background music (simulated)...");
                // In a real implementation, we'd load actual music files
                game_builder.load_music_from_bytes("background_music", vec![0; 100000]);
                game_builder.play_music("background_music");
            },
            300 => {
                // 5 seconds - create multiple sound sources
                log::info!("🎪 Creating multiple simultaneous audio sources...");
                for i in 0..5 {
                    let x = 200.0 + (i as f32) * 40.0;
                    let y = 200.0 + (i as f32 * 0.5).sin() * 50.0;
                    game_builder.play_sound_at("jump", x, y, 0.8);
                }
            },
            360 => {
                // 6 seconds - test treasure collection with audio
                log::info!("💰 Testing treasure collection with audio feedback...");
                let treasures = [(180.0, 180.0), (320.0, 220.0), (420.0, 160.0)];
                
                for &(x, y) in &treasures {
                    // This automatically plays coin sound
                    game_builder.create_treasure_pickup(x, y);
                }
            },
            420 => {
                // 7 seconds - test volume controls
                log::info!("🎛️  Testing dynamic volume controls...");
                game_builder.set_sfx_volume(0.3);
                game_builder.play_sound("explosion");  // Should be quieter
            },
            480 => {
                // 8 seconds - restore volume
                game_builder.set_sfx_volume(0.8);
                log::info!("🔊 Restored SFX volume");
            },
            540 => {
                // 9 seconds - test moving audio listener
                log::info!("🚶 Testing moving audio listener...");
                game_builder.set_audio_listener(100.0, 100.0);  // Move listener far away
                game_builder.play_sound_at("beep", 300.0, 200.0, 1.0);  // Sound should be quieter
            },
            600 => {
                // 10 seconds - create campfire scene with ambient audio
                log::info!("🔥 Creating campfire scene with ambient audio...");
                let campfire_effects = game_builder.create_campfire(350.0, 250.0);
                log::info!("Created campfire with {} visual effects", campfire_effects.len());
                
                // Move listener back to campfire
                game_builder.set_audio_listener(350.0, 250.0);
            },
            660 => {
                // 11 seconds - test audio muting
                log::info!("🔇 Testing audio muting...");
                game_builder.audio_manager.set_muted(true);
                game_builder.play_sound("explosion");  // Should be muted
            },
            720 => {
                // 12 seconds - unmute
                game_builder.audio_manager.set_muted(false);
                game_builder.play_sound("coin");  // Should be audible again
                log::info!("🔊 Audio unmuted");
            },
            780 => {
                // 13 seconds - test portal with magical audio
                log::info!("✨ Creating magical portal with audio effects...");
                let portal_effects = game_builder.create_portal(400.0, 300.0);
                log::info!("Created portal with {} effects", portal_effects.len());
                
                // Play mystical sound at portal location
                game_builder.play_sound_at("beep", 400.0, 300.0, 1.2);
            },
            _ => {
                // Random ambient sounds every few seconds
                if frame_count % 300 == 0 && frame_count < 900 {
                    let ambient_sounds = ["beep", "coin", "jump"];
                    let sound = ambient_sounds[frame_count / 300 % ambient_sounds.len()];
                    
                    let x = 200.0 + rand::random::<f32>() * 200.0;
                    let y = 150.0 + rand::random::<f32>() * 100.0;
                    
                    game_builder.play_sound_at(sound, x, y, 0.5);
                    log::debug!("Played ambient sound '{}' at ({:.1}, {:.1})", sound, x, y);
                }
            }
        }
        
        // Performance and statistics logging
        if frame_count % 300 == 0 { // Every 5 seconds
            let elapsed = current_time.duration_since(start_time).as_secs_f32();
            let fps = frame_count as f32 / elapsed;
            
            log::info!("Audio Demo: {:.1}fps | Active Sounds: {} | Music Playing: {} | Master Vol: {:.1}", 
                fps,
                game_builder.audio_manager.active_sound_count(),
                if game_builder.audio_manager.is_music_playing() { "Yes" } else { "No" },
                game_builder.audio_manager.get_master_volume()
            );
        }
        
        // Run demo for 30 seconds
        if current_time.duration_since(start_time).as_secs() >= 30 {
            break;
        }
        
        // Simulate frame timing
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
    
    // Clean up audio
    game_builder.stop_all_sounds();
    
    log::info!("=== AUDIO DEMO COMPLETE ===");
    log::info!("Demonstrated features:");
    log::info!("✅ Basic sound effect playback with volume control");
    log::info!("✅ Spatial audio with distance-based attenuation");
    log::info!("✅ Background music management and controls");
    log::info!("✅ Master, SFX, and music volume separation");
    log::info!("✅ Audio listener position tracking");
    log::info!("✅ Integration with visual effects (explosions, pickups, etc.)");
    log::info!("✅ Real-time volume adjustments and muting");
    log::info!("✅ Multiple simultaneous sound sources");
    log::info!("✅ No-code friendly API for easy integration");
}