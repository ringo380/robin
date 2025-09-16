use crate::engine::{
    GameBuilder,
    input::InputManager,
    math::Vec2,
};
use std::time::Instant;

pub async fn run_physics_demo() {
    env_logger::init();
    log::info!("Starting physics system demo");

    let mut game_builder = GameBuilder::new();
    
    // Set up physics world with custom gravity
    game_builder.physics_world.gravity = Vec2::new(0.0, -200.0); // Downward gravity
    
    // Create physics playground with boundaries and platforms
    log::info!("=== ROBIN ENGINE PHYSICS SYSTEM DEMO ===");
    log::info!("🏀 Rigid body dynamics with collision detection");
    log::info!("🎱 Circle and rectangle collider shapes");
    log::info!("⚡ Real-time collision resolution with physics response");
    log::info!("🏗️  Static and dynamic bodies with mass/force simulation");
    log::info!("🎯 Spatial grid optimization for collision detection");
    log::info!("💫 Integration with particle effects and lighting");
    
    // Create the physics playground
    let playground_bounds = (50.0, 550.0, 50.0, 400.0); // left, right, top, bottom
    let physics_objects = game_builder.create_physics_playground(
        playground_bounds.0, playground_bounds.1, playground_bounds.2, playground_bounds.3
    );
    
    log::info!("Created physics playground with {} objects", physics_objects.len());
    
    // Add some additional interactive elements
    let player_ball = game_builder.create_bouncing_ball(300.0, 100.0, 20.0);
    game_builder.apply_impulse(player_ball, 50.0, 100.0); // Give it initial momentum
    
    // Create some collectible items (static circles)
    let collectibles = [
        (150.0, 200.0),
        (300.0, 250.0), 
        (450.0, 180.0),
    ];
    
    let mut collectible_ids = Vec::new();
    for (i, &(x, y)) in collectibles.iter().enumerate() {
        let collectible_id = game_builder.create_physics_circle(x, y, 12.0, true);
        collectible_ids.push(collectible_id);
        
        // Add golden glow effect to collectibles
        game_builder.add_light(x, y, (1.0, 0.8, 0.2), 0.6);
        
        // Add sparkle effect
        let sparkle_name = format!("collectible_sparkles_{}", i);
        game_builder.particle_system.create_magic_sparkles(Vec2::new(x, y), sparkle_name);
        
        // Pulse animation
        let collectible_name = format!("collectible_{}", i);
        game_builder.pulse_color(
            &collectible_name,
            (1.0, 0.8, 0.2, 1.0), // Gold
            (1.0, 1.0, 0.6, 1.0), // Bright gold
            1.0 + (i as f32 * 0.3) // Stagger timing
        );
    }
    
    // Add some moving obstacles
    let mut moving_platforms = Vec::new();
    for i in 0..2 {
        let x = 200.0 + (i as f32) * 200.0;
        let y = 300.0;
        let platform_id = game_builder.create_platform(x, y, 80.0, 15.0);
        moving_platforms.push(platform_id);
        
        // Make platforms move back and forth
        let platform_name = format!("moving_platform_{}", i);
        game_builder.move_object(&platform_name, x, y, x + 100.0, y, 2.0);
        game_builder.start_animations(&platform_name);
    }
    
    // Add atmospheric effects
    game_builder.create_fog(300.0, 150.0);
    game_builder.add_magic_light(100.0, 350.0);
    game_builder.add_magic_light(500.0, 350.0);
    
    let start_time = Instant::now();
    let mut frame_count = 0;
    let mut collected_items = 0;
    
    // Demo simulation loop
    loop {
        let current_time = Instant::now();
        let delta_time = 0.016; // ~60 FPS
        frame_count += 1;
        
        // Update all game systems including physics
        let input = InputManager::new();
        let _animation_updates = game_builder.update(delta_time, &input);
        
        // Check for collisions between player ball and collectibles
        for (i, &collectible_id) in collectible_ids.iter().enumerate() {
            if game_builder.are_colliding(player_ball, collectible_id) {
                log::info!("Player collected item {}! Creating collection effect...", i + 1);
                
                // Get collectible position for effects
                if let Some((x, y)) = game_builder.get_physics_position(collectible_id) {
                    // Create collection effect
                    let collection_effects = game_builder.create_treasure_pickup(x, y);
                    log::debug!("Created {} collection effects", collection_effects.len());
                    collected_items += 1;
                }
                
                // Remove collectible (in real game, we'd remove from physics world)
                // For demo purposes, we'll just move it far away
                game_builder.physics_world.get_body_mut(collectible_id)
                    .map(|body| body.position = Vec2::new(-1000.0, -1000.0));
            }
        }
        
        // Apply random forces for interesting interactions
        if frame_count % 300 == 0 { // Every 5 seconds
            // Apply upward force to some balls
            for &object_id in &physics_objects[8..] { // Skip boundaries and platforms
                if game_builder.physics_world.get_body(object_id)
                    .map(|body| body.body_type != crate::engine::physics::BodyType::Static)
                    .unwrap_or(false) 
                {
                    let force_x = (rand::random::<f32>() - 0.5) * 200.0;
                    let force_y = rand::random::<f32>() * 100.0 + 50.0;
                    game_builder.apply_force(object_id, force_x, force_y);
                }
            }
            log::debug!("Applied random forces to dynamic objects");
        }
        
        // Create occasional fireworks
        if frame_count % 240 == 0 { // Every 4 seconds
            let x = 200.0 + rand::random::<f32>() * 200.0;
            let y = 100.0 + rand::random::<f32>() * 50.0;
            let colors = [
                (1.0, 0.2, 0.2), // Red
                (0.2, 1.0, 0.2), // Green  
                (0.2, 0.2, 1.0), // Blue
                (1.0, 0.8, 0.2), // Gold
                (1.0, 0.2, 1.0), // Magenta
            ];
            let color = colors[frame_count % colors.len()];
            game_builder.create_fireworks(x, y, color);
        }
        
        // Performance and statistics logging
        if frame_count % 300 == 0 { // Every 5 seconds
            let elapsed = current_time.duration_since(start_time).as_secs_f32();
            let fps = frame_count as f32 / elapsed;
            let collisions = game_builder.get_collision_events();
            
            log::info!("Physics Demo: {:.1}fps | Bodies: {} | Particles: {} | Lights: {} | Collisions: {} | Collected: {}", 
                fps,
                game_builder.get_physics_body_count(),
                game_builder.get_particle_count(), 
                game_builder.get_light_count(),
                collisions.len(),
                collected_items
            );
            
            // Print some physics object positions for verification
            if let Some((x, y)) = game_builder.get_physics_position(player_ball) {
                log::debug!("Player ball position: ({:.1}, {:.1})", x, y);
            }
        }
        
        // Run demo for 45 seconds
        if current_time.duration_since(start_time).as_secs() >= 45 {
            break;
        }
        
        // Simulate frame timing
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
    
    log::info!("=== PHYSICS DEMO COMPLETE ===");
    log::info!("Demonstrated features:");
    log::info!("✅ 2D rigid body physics with gravity simulation");
    log::info!("✅ Collision detection between circles and rectangles");
    log::info!("✅ Physics boundaries and static/dynamic object interaction");
    log::info!("✅ Force and impulse application for realistic movement");
    log::info!("✅ Real-time collision resolution with bouncing and friction");
    log::info!("✅ Integration with particle effects, lighting, and animations");
    log::info!("✅ Performance optimization with spatial grid broad-phase");
    log::info!("✅ No-code friendly API for easy physics setup");
    log::info!("Collected {} items during demo", collected_items);
}