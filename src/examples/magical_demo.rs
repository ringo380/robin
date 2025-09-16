use crate::engine::{GameBuilder, input::InputManager};
use std::time::Instant;

pub async fn run_magical_demo() {
    println!("DEBUG: Starting run_magical_demo");
    env_logger::init();
    println!("DEBUG: env_logger initialized");
    log::info!("Starting magical demo with particles, lighting, and animations");
    println!("DEBUG: About to create GameBuilder");

    let mut game_builder = GameBuilder::new();
    println!("DEBUG: GameBuilder created successfully");
    
    // === SETUP MAGICAL SCENE ===
    
    // Create a magical forest environment
    let fairy_positions = [
        (100.0, 150.0),
        (300.0, 200.0),
        (500.0, 120.0),
        (200.0, 350.0),
    ];
    game_builder.setup_magical_forest(&fairy_positions);
    
    // Add a campfire scene
    let campfire_effects = game_builder.create_campfire(400.0, 300.0);
    log::info!("Created campfire with {} effects", campfire_effects.len());
    
    // Create a magical portal
    let portal_effects = game_builder.create_portal(600.0, 200.0);
    log::info!("Created portal with {} effects", portal_effects.len());
    
    // === SETUP INTERACTIVE ELEMENTS ===
    
    // Create some treasure that will have pickup effects
    let treasure_positions = [
        (150.0, 100.0),
        (350.0, 150.0),
        (550.0, 300.0),
    ];
    
    for (i, &(x, y)) in treasure_positions.iter().enumerate() {
        let treasure_id = format!("treasure_{}", i);
        
        // Add pulsing glow animation to treasures
        game_builder.pulse_color(
            &treasure_id,
            (1.0, 0.8, 0.2, 1.0), // Gold
            (1.0, 1.0, 0.6, 0.8), // Bright gold
            1.5
        );
        
        // Add gentle floating animation
        game_builder.move_object(&treasure_id, x, y, x, y + 10.0, 2.0);
        
        // Start animations
        game_builder.start_animations(&treasure_id);
    }
    
    // === DEMO LOOP ===
    
    let start_time = Instant::now();
    let mut last_explosion = Instant::now();
    let mut explosion_count = 0;
    
    log::info!("=== ROBIN ENGINE MAGICAL DEMO ===");
    log::info!("Features demonstrated:");
    log::info!("✨ Dynamic lighting system with {} lights", game_builder.get_light_count());
    log::info!("🎆 Particle effects (explosions, magic, fire, fog)");
    log::info!("🌟 Smooth animations with easing");
    log::info!("🔮 No-code friendly API");
    log::info!("🎮 Real-time visual effects");
    
    // Simulate a game loop for 30 seconds
    loop {
        let current_time = Instant::now();
        let delta_time = 0.016; // ~60 FPS
        
        // Update all systems
        let input = InputManager::new();
        let animation_updates = game_builder.update(delta_time, &input);
        
        // Apply animation updates to objects (in a real game, this would update actual game objects)
        for (object_id, values) in animation_updates {
            log::debug!("Updating object {} with {} animation values", object_id, values.len());
        }
        
        // Trigger periodic explosions for demonstration
        if current_time.duration_since(last_explosion).as_secs() >= 3 && explosion_count < 5 {
            let explosion_x = 100.0 + (explosion_count as f32 * 100.0);
            let explosion_y = 400.0;
            
            let explosion_id = game_builder.create_explosion(explosion_x, explosion_y);
            log::info!("💥 Explosion {} created at ({}, {})", explosion_count + 1, explosion_x, explosion_y);
            
            // Create some fireworks too
            let colors = [
                (1.0, 0.2, 0.2), // Red
                (0.2, 1.0, 0.2), // Green  
                (0.2, 0.2, 1.0), // Blue
                (1.0, 1.0, 0.2), // Yellow
                (1.0, 0.2, 1.0), // Magenta
            ];
            
            let firework_color = colors[explosion_count % colors.len()];
            let firework_id = game_builder.create_fireworks(explosion_x, explosion_y + 50.0, firework_color);
            log::info!("🎆 Fireworks created with color {:?}", firework_color);
            
            last_explosion = current_time;
            explosion_count += 1;
        }
        
        // Performance monitoring
        let particle_count = game_builder.get_particle_count();
        if particle_count > 0 {
            log::debug!("Active particles: {}, Lights: {}", 
                particle_count, game_builder.get_light_count());
        }
        
        // Run demo for 30 seconds
        if current_time.duration_since(start_time).as_secs() >= 30 {
            break;
        }
        
        // Simulate frame timing
        std::thread::sleep(std::time::Duration::from_millis(16)); // ~60 FPS
    }
    
    log::info!("=== DEMO COMPLETE ===");
    log::info!("Final stats:");
    log::info!("- Particles active: {}", game_builder.get_particle_count());
    log::info!("- Lights active: {}", game_builder.get_light_count());
    log::info!("- Effects created: {}", explosion_count + campfire_effects.len() + portal_effects.len());
    
    // Cleanup demonstration
    log::info!("Cleaning up effects...");
    game_builder.clear_all_effects();
    log::info!("Cleanup complete. Particles: {}, Lights: {}", 
        game_builder.get_particle_count(), game_builder.get_light_count());
}