use robin::engine::{
    GameBuilder, 
    assets::{AssetType, TextureImportSettings, AudioImportSettings, BuildTarget},
    error::RobinResult,
    input::InputManager,
};
use std::path::PathBuf;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Robin Engine Asset Pipeline Demo");
    println!("===================================");
    println!();

    // Initialize the game with comprehensive asset pipeline
    let mut game = GameBuilder::new();
    
    // Set up logging for the demo
    game.setup_logging(None);
    
    // Demo the asset pipeline workflow
    demonstrate_asset_import(&mut game)?;
    demonstrate_asset_building(&mut game)?;
    demonstrate_pipeline_configuration(&mut game)?;
    demonstrate_platform_builds(&mut game)?;
    demonstrate_asset_validation(&mut game)?;
    demonstrate_cache_management(&mut game)?;
    demonstrate_integration_with_hot_reload(&mut game)?;

    println!("🏁 Asset pipeline demo completed!");
    Ok(())
}

fn demonstrate_asset_import(game: &mut GameBuilder) -> Result<(), Box<dyn std::error::Error>> {
    println!("📥 Asset Import Demonstration");
    println!("=============================");
    
    // Create sample asset directories
    std::fs::create_dir_all("examples/sample_assets/textures")?;
    std::fs::create_dir_all("examples/sample_assets/audio")?;
    std::fs::create_dir_all("examples/sample_assets/configs")?;
    
    // Create some sample files for demonstration
    create_sample_assets()?;
    
    // Import individual assets with specific settings
    println!("Importing individual assets with custom settings...");
    
    // Configure texture importer with high-quality settings
    game.configure_asset_importer(|importer| {
        let mut texture_settings = TextureImportSettings::default();
        texture_settings.generate_mipmaps = true;
        texture_settings.max_size = Some((1024, 1024));
        texture_settings.compression_quality = 0.9;
        texture_settings.format_override = Some("PNG".to_string());
        
        importer.set_texture_import_settings(texture_settings);
    });
    
    // Import a texture
    game.import_asset("examples/sample_assets/textures/player_sprite.png", AssetType::Texture);
    
    // Configure audio importer
    game.configure_asset_importer(|importer| {
        let mut audio_settings = AudioImportSettings::default();
        audio_settings.quality = 0.8;
        audio_settings.sample_rate = Some(44100);
        audio_settings.channels = Some(2);
        audio_settings.normalize_volume = true;
        
        importer.set_audio_import_settings(audio_settings);
    });
    
    // Import audio file
    game.import_asset("examples/sample_assets/audio/background_music.wav", AssetType::Audio);
    
    // Import entire directories
    println!("Importing assets from directories...");
    game.import_assets_directory("examples/sample_assets/textures", true);
    game.import_assets_directory("examples/sample_assets/audio", false);
    
    println!("✅ Asset import completed");
    println!();
    Ok(())
}

fn demonstrate_asset_building(game: &mut GameBuilder) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔨 Asset Building Demonstration");
    println!("===============================");
    
    // Build all assets for desktop platform
    println!("Building all assets for desktop platform...");
    game.build_assets(Some("desktop"));
    
    // Build specific asset types for mobile
    println!("Building textures and audio for mobile platform...");
    game.build_assets_for_platform(&[AssetType::Texture, AssetType::Audio], "mobile");
    
    // Show build statistics
    println!("📊 {}", game.get_build_stats());
    println!("✅ Asset building completed");
    println!();
    Ok(())
}

fn demonstrate_pipeline_configuration(game: &mut GameBuilder) -> Result<(), Box<dyn std::error::Error>> {
    println!("⚙️  Pipeline Configuration Demonstration");
    println!("=======================================");
    
    // Configure the asset pipeline with custom settings
    game.configure_asset_pipeline(|pipeline| {
        // Enable parallel processing
        pipeline.set_parallel_processing(true);
        
        // Configure cache settings
        pipeline.set_cache_size_limit(100 * 1024 * 1024); // 100MB
        pipeline.set_cache_expiry(Duration::from_hours(24));
        
        // Add build targets
        pipeline.add_build_target(BuildTarget {
            name: "web".to_string(),
            texture_format: "WebP".to_string(),
            audio_format: "OGG".to_string(),
            max_texture_size: 512,
            audio_quality: 0.7,
            enable_compression: true,
        });
        
        pipeline.add_build_target(BuildTarget {
            name: "mobile".to_string(), 
            texture_format: "ASTC".to_string(),
            audio_format: "AAC".to_string(),
            max_texture_size: 1024,
            audio_quality: 0.8,
            enable_compression: true,
        });
    });
    
    // Configure importer with different quality presets
    game.configure_asset_importer(|importer| {
        // High-quality preset for hero assets
        let high_quality_texture = TextureImportSettings {
            generate_mipmaps: true,
            max_size: Some((2048, 2048)),
            compression_quality: 1.0,
            format_override: None,
            enable_compression: false,
        };
        
        // Low-quality preset for background assets
        let low_quality_texture = TextureImportSettings {
            generate_mipmaps: false,
            max_size: Some((512, 512)),
            compression_quality: 0.6,
            format_override: Some("JPEG".to_string()),
            enable_compression: true,
        };
        
        importer.set_texture_import_settings(high_quality_texture);
        importer.add_import_preset("low_quality", Box::new(low_quality_texture));
    });
    
    println!("✅ Pipeline configuration completed");
    println!();
    Ok(())
}

fn demonstrate_platform_builds(game: &mut GameBuilder) -> Result<(), Box<dyn std::error::Error>> {
    println!("📱 Platform-Specific Build Demonstration");
    println!("========================================");
    
    let platforms = vec!["desktop", "mobile", "web"];
    
    for platform in platforms {
        println!("Building assets for {} platform...", platform);
        
        // Configure platform-specific settings
        match platform {
            "desktop" => {
                game.configure_asset_pipeline(|pipeline| {
                    pipeline.set_texture_compression(false);
                    pipeline.set_audio_quality(1.0);
                });
            }
            "mobile" => {
                game.configure_asset_pipeline(|pipeline| {
                    pipeline.set_texture_compression(true);
                    pipeline.set_audio_quality(0.8);
                    pipeline.set_max_texture_size(1024);
                });
            }
            "web" => {
                game.configure_asset_pipeline(|pipeline| {
                    pipeline.set_texture_compression(true);
                    pipeline.set_audio_quality(0.7);
                    pipeline.set_max_texture_size(512);
                    pipeline.enable_progressive_loading(true);
                });
            }
            _ => {}
        }
        
        // Build for specific platform
        game.build_assets(Some(platform));
        
        println!("  📊 Platform {}: {}", platform, game.get_build_stats());
    }
    
    println!("✅ Platform builds completed");
    println!();
    Ok(())
}

fn demonstrate_asset_validation(game: &mut GameBuilder) -> Result<(), Box<dyn std::error::Error>> {
    println!("✅ Asset Validation Demonstration");
    println!("=================================");
    
    // Create some invalid assets for testing
    create_test_invalid_assets()?;
    
    // Run comprehensive asset validation
    println!("Running asset validation checks...");
    game.validate_assets();
    
    // Show validation results
    if game.has_error() {
        println!("⚠️  Validation found issues:");
        if let Some(error) = game.get_last_error() {
            println!("  - {}", error);
        }
        game.clear_last_error();
    } else {
        println!("✅ All assets passed validation");
    }
    
    println!();
    Ok(())
}

fn demonstrate_cache_management(game: &mut GameBuilder) -> Result<(), Box<dyn std::error::Error>> {
    println!("🗄️  Cache Management Demonstration");
    println!("=================================");
    
    // Show current cache statistics
    println!("📊 Current build stats: {}", game.get_build_stats());
    
    // Build the same assets again to show cache hits
    println!("Building assets again to demonstrate caching...");
    game.build_assets(Some("desktop"));
    
    println!("📊 After rebuild: {}", game.get_build_stats());
    
    // Clear cache and rebuild to show cache miss
    println!("Clearing cache and rebuilding...");
    game.clear_asset_cache();
    game.build_assets(Some("desktop"));
    
    println!("📊 After cache clear: {}", game.get_build_stats());
    println!("✅ Cache management demonstration completed");
    println!();
    Ok(())
}

fn demonstrate_integration_with_hot_reload(game: &mut GameBuilder) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Hot Reload Integration Demonstration");
    println!("=======================================");
    
    // Enable hot reloading and auto-register assets
    game.enable_hot_reload(true)
        .auto_register_assets();
    
    // Add a callback to rebuild assets when they change
    game.add_reload_callback("player_sprite", |event| {
        println!("🔄 Asset changed, triggering rebuild: {:?}", event);
    });
    
    println!("Hot reload system configured with asset pipeline integration");
    
    // Simulate a game loop with asset updates
    let input = InputManager::new();
    println!("Running simulation with hot reload monitoring...");
    
    for frame in 0..30 {
        game.update(0.016, &input);
        
        // Every 10 frames, show hot reload stats
        if frame % 10 == 9 {
            println!("Frame {}: {}", frame + 1, game.get_hot_reload_stats());
        }
        
        // Simulate frame delay
        std::thread::sleep(Duration::from_millis(20));
    }
    
    println!("✅ Hot reload integration completed");
    println!();
    Ok(())
}

fn create_sample_assets() -> Result<(), Box<dyn std::error::Error>> {
    // Create sample texture file (fake PNG data)
    let png_data = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, // IHDR chunk size
        b'I', b'H', b'D', b'R',  // IHDR
        0x00, 0x00, 0x00, 0x10,  // width: 16
        0x00, 0x00, 0x00, 0x10,  // height: 16
        0x08, 0x06, 0x00, 0x00, 0x00, // bit depth, color type, etc.
        0x1F, 0xF3, 0xFF, 0x61,  // CRC
        0x00, 0x00, 0x00, 0x00,  // Empty IDAT
        b'I', b'E', b'N', b'D',
        0xAE, 0x42, 0x60, 0x82   // IEND
    ];
    
    std::fs::write("examples/sample_assets/textures/player_sprite.png", png_data)?;
    std::fs::write("examples/sample_assets/textures/background.png", &[0xFF; 100])?;
    
    // Create sample audio file (fake WAV data)
    let wav_header = vec![
        b'R', b'I', b'F', b'F',  // RIFF
        0x24, 0x00, 0x00, 0x00,  // File size
        b'W', b'A', b'V', b'E',  // WAVE
        b'f', b'm', b't', b' ',  // fmt chunk
        0x10, 0x00, 0x00, 0x00,  // fmt chunk size
        0x01, 0x00, 0x02, 0x00,  // PCM, stereo
        0x44, 0xAC, 0x00, 0x00,  // 44100 Hz
        0x10, 0xB1, 0x02, 0x00,  // bytes per second
        0x04, 0x00, 0x10, 0x00,  // block align, bits per sample
        b'd', b'a', b't', b'a',  // data chunk
        0x00, 0x00, 0x00, 0x00   // data size (empty for demo)
    ];
    
    std::fs::write("examples/sample_assets/audio/background_music.wav", wav_header)?;
    std::fs::write("examples/sample_assets/audio/jump_sound.wav", &[0x00; 50])?;
    
    // Create sample config file
    let config_data = r#"{
    "version": "1.0",
    "settings": {
        "volume": 0.8,
        "graphics_quality": "high"
    }
}"#;
    
    std::fs::write("examples/sample_assets/configs/game_config.json", config_data)?;
    
    Ok(())
}

fn create_test_invalid_assets() -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all("examples/sample_assets/invalid")?;
    
    // Create corrupted PNG
    std::fs::write("examples/sample_assets/invalid/corrupted.png", &[0xFF, 0xFF, 0xFF])?;
    
    // Create invalid JSON config
    std::fs::write("examples/sample_assets/invalid/bad_config.json", "{ invalid json")?;
    
    // Create empty audio file
    std::fs::write("examples/sample_assets/invalid/empty.wav", &[])?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_pipeline_workflow() {
        let mut game = GameBuilder::new();
        
        // Test basic pipeline configuration
        game.configure_asset_pipeline(|pipeline| {
            pipeline.set_parallel_processing(true);
            pipeline.set_cache_size_limit(50 * 1024 * 1024);
        });
        
        // Test importer configuration
        game.configure_asset_importer(|importer| {
            let texture_settings = TextureImportSettings::default();
            importer.set_texture_import_settings(texture_settings);
        });
        
        // Pipeline should be properly configured
        let stats = game.get_build_stats();
        assert!(stats.contains("assets processed"));
    }

    #[test]
    fn test_cache_management() {
        let mut game = GameBuilder::new();
        
        // Clear cache should work without errors
        game.clear_asset_cache();
        
        // Build stats should be accessible
        let stats = game.get_build_stats();
        assert!(!stats.is_empty());
    }

    #[test]
    fn test_asset_validation() {
        let mut game = GameBuilder::new();
        
        // Validation should run without panicking
        game.validate_assets();
        
        // Should be able to check for errors
        let has_error = game.has_error();
        assert!(!has_error || game.get_last_error().is_some());
    }
}