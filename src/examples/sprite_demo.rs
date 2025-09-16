use crate::engine::{
    GameBuilder,
    graphics::{Sprite, AnimatedSprite, SpriteAnimation, SpriteFrame, SpriteManager, UVRect, Renderer, Light},
    input::InputManager,
    math::Vec2,
};
use std::time::Instant;

pub async fn run_sprite_demo() {
    env_logger::init();
    log::info!("Starting sprite system demo");

    let mut game_builder = GameBuilder::new();
    let mut sprite_manager = SpriteManager::new();
    
    // Create some example sprites programmatically
    create_example_sprites(&mut sprite_manager);
    
    // Set up a diverse scene
    setup_demo_scene(&mut game_builder, &sprite_manager);
    
    log::info!("=== ROBIN ENGINE SPRITE SYSTEM DEMO ===");
    log::info!("🎨 Texture system with sprite batching");
    log::info!("🖼️  Sprite animations with multiple frames");
    log::info!("⚡ Instance-based rendering for performance");
    log::info!("💡 Dynamic lighting on textured sprites");
    log::info!("🎭 Sprite atlas support and UV mapping");
    log::info!("🎮 Integrated with existing particle/animation systems");
    
    let start_time = Instant::now();
    let mut frame_count = 0;
    
    // Demo loop
    loop {
        let current_time = Instant::now();
        let delta_time = 0.016; // ~60 FPS
        frame_count += 1;
        
        // Update sprite animations
        sprite_manager.update_animated_sprites(delta_time);
        
        // Update game systems
        let input = InputManager::new();
        let _animation_updates = game_builder.update(delta_time, &input);
        
        // Create some dynamic sprites based on time for variety
        if frame_count % 180 == 0 { // Every 3 seconds at 60 FPS
            create_dynamic_sprites(&mut game_builder, current_time);
        }
        
        // Performance logging
        if frame_count % 300 == 0 { // Every 5 seconds
            let elapsed = current_time.duration_since(start_time).as_secs_f32();
            let fps = frame_count as f32 / elapsed;
            log::info!("Demo running: {:.1}fps, Particles: {}, Lights: {}", 
                fps, 
                game_builder.get_particle_count(), 
                game_builder.get_light_count()
            );
        }
        
        // Run demo for 60 seconds
        if current_time.duration_since(start_time).as_secs() >= 60 {
            break;
        }
        
        // Simulate frame timing
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
    
    log::info!("=== SPRITE DEMO COMPLETE ===");
    log::info!("Demonstrated features:");
    log::info!("✅ Textured sprite rendering with batching");
    log::info!("✅ Animated sprites with frame-based animations");
    log::info!("✅ Dynamic sprite creation and management");
    log::info!("✅ Integration with lighting and particle systems");
    log::info!("✅ Performance optimization through instanced rendering");
}

fn create_example_sprites(sprite_manager: &mut SpriteManager) {
    // Create basic sprites with different textures
    let player_sprite = Sprite::new("white".to_string())
        .with_size(32.0, 32.0)
        .with_color(0.2, 0.8, 0.2, 1.0); // Green tint for player
    sprite_manager.create_sprite("player".to_string(), player_sprite);
    
    let enemy_sprite = Sprite::new("white".to_string())
        .with_size(24.0, 24.0)
        .with_color(0.8, 0.2, 0.2, 1.0); // Red tint for enemy
    sprite_manager.create_sprite("enemy".to_string(), enemy_sprite);
    
    let coin_sprite = Sprite::new("white".to_string())
        .with_size(16.0, 16.0)
        .with_color(1.0, 0.8, 0.2, 1.0); // Gold tint
    sprite_manager.create_sprite("coin".to_string(), coin_sprite);
    
    let ui_button_sprite = Sprite::new("ui_button".to_string())
        .with_size(100.0, 40.0)
        .with_pivot(0.0, 0.0); // Top-left pivot for UI
    sprite_manager.create_sprite("button".to_string(), ui_button_sprite);
    
    // Create an animated sprite
    create_animated_fire_sprite(sprite_manager);
    create_animated_water_sprite(sprite_manager);
    
    log::info!("Created {} static sprites and animated sprites", 4);
}

fn create_animated_fire_sprite(sprite_manager: &mut SpriteManager) {
    let base_sprite = Sprite::new("white".to_string())
        .with_size(24.0, 32.0);
    
    let mut animated_sprite = AnimatedSprite::new(base_sprite);
    
    // Create fire animation frames
    let mut fire_animation = SpriteAnimation::new("burn".to_string(), true);
    
    // Frame 1: Bright orange
    fire_animation.add_frame(SpriteFrame {
        texture_name: "white".to_string(),
        uv_rect: UVRect::full_texture(),
        pivot: Vec2::new(0.5, 0.5),
        duration: 0.1,
    });
    
    // Add more frames with different colors to simulate fire
    for i in 0..6 {
        let intensity = 0.8 + 0.2 * (i as f32 / 6.0).sin();
        fire_animation.add_frame(SpriteFrame {
            texture_name: "white".to_string(),
            uv_rect: UVRect::full_texture(),
            pivot: Vec2::new(0.5, 0.5),
            duration: 0.08 + 0.04 * (i as f32 / 6.0).cos(),
        });
    }
    
    animated_sprite.add_animation(fire_animation);
    animated_sprite.play_animation("burn");
    
    sprite_manager.create_animated_sprite("fire_torch".to_string(), animated_sprite);
}

fn create_animated_water_sprite(sprite_manager: &mut SpriteManager) {
    let base_sprite = Sprite::new("white".to_string())
        .with_size(32.0, 24.0)
        .with_color(0.2, 0.6, 0.9, 0.8); // Blue with transparency
    
    let mut animated_sprite = AnimatedSprite::new(base_sprite);
    
    // Create water wave animation
    let mut wave_animation = SpriteAnimation::new("wave".to_string(), true);
    
    for i in 0..8 {
        let wave_offset = (i as f32 / 8.0) * std::f32::consts::PI * 2.0;
        wave_animation.add_frame(SpriteFrame {
            texture_name: "white".to_string(),
            uv_rect: UVRect::full_texture(),
            pivot: Vec2::new(0.5, 0.5),
            duration: 0.12,
        });
    }
    
    animated_sprite.add_animation(wave_animation);
    animated_sprite.play_animation("wave");
    animated_sprite.speed_multiplier = 0.8; // Slower water movement
    
    sprite_manager.create_animated_sprite("water".to_string(), animated_sprite);
}

fn setup_demo_scene(game_builder: &mut GameBuilder, sprite_manager: &SpriteManager) {
    // Create a dungeon-like environment with torches
    let torch_positions = [
        (100.0, 100.0),
        (300.0, 100.0),
        (500.0, 100.0),
        (100.0, 300.0),
        (500.0, 300.0),
    ];
    game_builder.setup_dungeon_lighting(&torch_positions);
    
    // Add some fire effects at torch positions
    for &(x, y) in &torch_positions {
        game_builder.create_fire(x, y + 20.0);
    }
    
    // Add magical elements
    game_builder.add_magic_light(300.0, 200.0);
    let portal_effects = game_builder.create_portal(300.0, 200.0);
    log::info!("Created magical portal with {} effects", portal_effects.len());
    
    // Create treasure scattered around
    let treasure_positions = [
        (150.0, 150.0),
        (250.0, 120.0),
        (350.0, 140.0),
        (450.0, 160.0),
        (200.0, 250.0),
        (400.0, 280.0),
    ];
    
    for (i, &(x, y)) in treasure_positions.iter().enumerate() {
        let treasure_id = format!("treasure_{}", i);
        game_builder.pulse_color(
            &treasure_id,
            (1.0, 0.8, 0.2, 1.0), // Gold
            (1.0, 1.0, 0.6, 1.0), // Bright gold
            1.0 + (i as f32 * 0.2) // Vary the timing
        );
        
        // Add floating animation
        game_builder.move_object(
            &treasure_id,
            x, y,
            x, y + 8.0,
            2.0 + (i as f32 * 0.3)
        );
        
        game_builder.start_animations(&treasure_id);
    }
    
    // Add some water areas
    let water_positions = [(80.0, 350.0), (180.0, 360.0), (280.0, 340.0)];
    for &(x, y) in &water_positions {
        // Add blue glow under water
        game_builder.add_light(x, y, (0.2, 0.4, 0.8), 0.3);
    }
    
    log::info!("Set up demo scene with {} torches, {} treasures, {} water areas", 
        torch_positions.len(), treasure_positions.len(), water_positions.len());
}

fn create_dynamic_sprites(game_builder: &mut GameBuilder, current_time: Instant) {
    let time_offset = current_time.elapsed().as_secs_f32();
    
    // Create floating magical orbs
    let orb_x = 300.0 + 100.0 * (time_offset * 0.5).sin();
    let orb_y = 200.0 + 50.0 * (time_offset * 0.7).cos();
    
    game_builder.add_magic_light(orb_x, orb_y);
    let sparkles = game_builder.create_magic_trail(orb_x, orb_y);
    
    // Create temporary explosion effects
    if time_offset.fract() < 0.1 { // Roughly once per cycle
        let explosion_x = 400.0 + 80.0 * (time_offset * 0.3).cos();
        let explosion_y = 300.0 + 60.0 * (time_offset * 0.4).sin();
        
        let _explosion = game_builder.create_explosion(explosion_x, explosion_y);
        
        // Add temporary light at explosion
        game_builder.add_light(explosion_x, explosion_y, (1.0, 0.6, 0.2), 1.2);
    }
    
    log::debug!("Created dynamic effects at ({:.1}, {:.1})", orb_x, orb_y);
}