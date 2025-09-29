/// Robin Engine - Unified Interactive Demo
///
/// The flagship demonstration of Robin Engine capabilities featuring:
/// - Metal rendering optimized for Apple Silicon
/// - Voxel world construction with frustum culling and greedy meshing
/// - Engineer Build Mode with advanced building tools
/// - Real-time physics and dynamic lighting
/// - Save/load system integration
/// - Modular feature architecture for progressive enhancement

mod renderer;
mod window;
mod ui;
mod ui_integration;  // New production UI integration
mod demo_state;
mod player_representation;
mod config;
mod logging;
mod culling;
mod lod_system;
mod greedy_meshing;
mod material_batching;
mod performance_monitor;
mod dynamic_texture_atlas;
mod pbr_lighting;
mod chunk_streaming;
mod gameplay_systems;
mod multiplayer_collaboration;
mod voxel_physics_system;
mod integration_test;

use renderer::{MetalRenderer, Camera, Mesh, TextureAtlas};
use window::{NativeWindow, WindowEvent, MouseButton, key_codes};
use robin::engine::generation::voxel_system::{VoxelWorld, VoxelType};
use robin::engine::build_mode::{BuildMode, TemplateType};

// Simple raycast function for the demo
fn raycast_voxel_world(world: &VoxelWorld, origin: Vector3<f32>, direction: Vector3<f32>, max_distance: f32) -> Option<(i32, i32, i32)> {
    use cgmath::InnerSpace;
    let mut current = origin;
    let step = direction.normalize() * 0.1;
    let mut distance = 0.0;

    while distance < max_distance {
        let x = current.x.floor() as i32;
        let y = current.y.floor() as i32;
        let z = current.z.floor() as i32;

        if let Some(voxel_type) = world.get_voxel(Vector3::new(x as f32, y as f32, z as f32)) {
            if voxel_type != VoxelType::Air {
                return Some((x, y, z));
            }
        }

        current += step;
        distance += 0.1;
    }

    None
}

// Helper function for VoxelType colors
fn get_voxel_color(voxel_type: &VoxelType) -> [f32; 3] {
    match voxel_type {
        VoxelType::Air => [0.0, 0.0, 0.0],
        VoxelType::Stone => [0.6, 0.6, 0.6],
        VoxelType::Wood => [0.6, 0.4, 0.2],
        VoxelType::Glass => [0.8, 0.9, 1.0],
        VoxelType::Metal => [0.7, 0.7, 0.8],
        VoxelType::Brick => [0.8, 0.4, 0.3],
        VoxelType::Concrete => [0.5, 0.5, 0.5],
        VoxelType::Solid => [0.4, 0.4, 0.4],
        VoxelType::Liquid => [0.2, 0.4, 0.8],
        VoxelType::Gas => [0.9, 0.9, 0.9],
        VoxelType::Light => [1.0, 1.0, 0.8],
        VoxelType::Custom(_) => [0.5, 0.5, 0.5],
    }
}

// Convert between robin::world::VoxelType and robin::generation::voxel_system::VoxelType
fn convert_world_voxel_to_generation(world_voxel: robin::world::VoxelType) -> robin::engine::generation::voxel_system::VoxelType {
    use robin::world::VoxelType as WorldVoxel;
    use robin::engine::generation::voxel_system::VoxelType as GenVoxel;

    match world_voxel {
        WorldVoxel::Air => GenVoxel::Air,
        WorldVoxel::Stone => GenVoxel::Stone,
        WorldVoxel::Wood => GenVoxel::Wood,
        WorldVoxel::Metal => GenVoxel::Metal,
        WorldVoxel::Glass => GenVoxel::Glass,
        WorldVoxel::Brick => GenVoxel::Brick,
        // Map additional world variants to available generation variants
        WorldVoxel::Dirt => GenVoxel::Solid,     // Dirt maps to Solid
        WorldVoxel::Grass => GenVoxel::Solid,    // Grass maps to Solid
        WorldVoxel::Sand => GenVoxel::Solid,     // Sand maps to Solid
        WorldVoxel::Water => GenVoxel::Liquid,   // Water maps to Liquid
        WorldVoxel::Leaves => GenVoxel::Solid,   // Leaves map to Solid
        WorldVoxel::Crystal => GenVoxel::Light,  // Crystal maps to Light (emissive)
        WorldVoxel::Lava => GenVoxel::Liquid,    // Lava maps to Liquid (hot liquid)
        WorldVoxel::Ice => GenVoxel::Solid,      // Ice maps to Solid
        WorldVoxel::Obsidian => GenVoxel::Stone, // Obsidian maps to Stone (hard material)
    }
}

use ui::simple_ui::{SimpleUISystem, UIAction};
use demo_state::{DemoStateManager, DemoMode};
use logging::{LogCategory, init_logging, log_info, log_warn, log_error, log_debug,
             log_startup_message, log_initialization_step, log_initialization_complete,
             log_user_action, PerformanceLogger};

use cgmath::Vector3;
use std::time::Instant;
use std::collections::HashSet;

fn main() {
    init_logging();

    // Run integration tests for performance dashboard
    log_info!(LogCategory::Engine, "Running integration tests...");
    integration_test::run_integration_tests();

    match run() {
        Ok(_) => log_info!(LogCategory::Engine, "Robin Engine exited successfully"),
        Err(e) => log_error!(LogCategory::Engine, "Application error: {}", e),
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    log_startup_message();

    // Create native macOS window
    let window_perf = PerformanceLogger::new(LogCategory::Window, "Window creation");
    let mut window = NativeWindow::new("Robin Engine - Engineer Build Mode", 1200.0, 800.0)?;
    window_perf.finish();
    log_initialization_complete(LogCategory::Window, "Native macOS window");

    // Create Metal renderer
    let renderer_perf = PerformanceLogger::new(LogCategory::Renderer, "Metal renderer initialization");
    let mut renderer = MetalRenderer::new(&window)?;
    renderer_perf.finish();
    log_initialization_complete(LogCategory::Renderer, "Metal renderer");

    // Create camera
    let window_size = window.get_size();
    let mut camera = Camera::new(window_size.width as f32, window_size.height as f32);

    // Create player representation
    let mut player_renderer = player_representation::PlayerBodyRenderer::new();

    // Initialize character physics system with Robin engine integration
    let mut character_state = robin::engine::character::CharacterState::default();
    let mut character_physics = robin::engine::character::CharacterPhysics::new();

    // Set initial player position to match camera
    character_state.position = nalgebra::Point3::new(camera.eye.x, camera.eye.y, camera.eye.z);

    // Initialize physics world and create character body
    if let Err(e) = character_physics.initialize_physics_world() {
        eprintln!("❌ Failed to initialize physics world: {}", e);
    } else {
        println!("⚡ Physics world initialized successfully");
    }

    if let Err(e) = character_physics.create_character_body(cgmath::Point3::new(camera.eye.x, camera.eye.y, camera.eye.z)) {
        eprintln!("❌ Failed to create character physics body: {}", e);
    } else {
        println!("🧍 Character physics body created successfully");
    }

    log_initialization_complete(LogCategory::Engine, "Player representation and physics");

    // Create voxel world
    let world_perf = PerformanceLogger::new(LogCategory::World, "Voxel world generation");
    log_initialization_step(LogCategory::World, "voxel world");
    let mut world = VoxelWorld::new("Demo World".to_string(), (100, 100, 100));

    // Generate some initial terrain
    log_info!(LogCategory::World, "Generating initial terrain...");
    for x in -20..20 {
        for z in -20..20 {
            let height = 8 + ((x as f32 * 0.1).sin() * 4.0 + (z as f32 * 0.1).cos() * 4.0) as i32;
            for y in 0..height {
                let voxel_type = if y < height - 3 {
                    VoxelType::Stone
                } else if y < height - 1 {
                    VoxelType::Solid
                } else {
                    VoxelType::Wood
                };
                world.set_voxel(Vector3::new(x as f32, y as f32, z as f32), voxel_type);
            }
        }
    }

    // Add some special features
    world.set_voxel(Vector3::new(0.0, 12.0, 0.0), VoxelType::Glass);
    world.set_voxel(Vector3::new(5.0, 10.0, 5.0), VoxelType::Wood);
    world.set_voxel(Vector3::new(5.0, 11.0, 5.0), VoxelType::Wood);
    world.set_voxel(Vector3::new(5.0, 12.0, 5.0), VoxelType::Metal);
    let voxel_count = world.count_active_voxels();
    log_info!(LogCategory::World, "Generated {} active voxels", voxel_count);
    world_perf.finish();
    let mut world_mesh = Mesh::new();

    // Generate initial world mesh
    let mesh_perf = PerformanceLogger::new(LogCategory::Renderer, "World mesh generation");
    generate_world_mesh(&world, &mut world_mesh, &camera);
    world_mesh.create_buffers(renderer.get_device());
    mesh_perf.finish();
    log_info!(LogCategory::Renderer, "Generated {} vertices, {} indices", world_mesh.vertex_count, world_mesh.index_count);

    // Create preview mesh for ghost blocks
    let mut preview_mesh = Mesh::new();
    preview_mesh.create_buffers(renderer.get_device());

    // Create Engineer Build System
    let mut build_system = VoxelBuildSystem::new();
    log_initialization_complete(LogCategory::Build, "Engineer Build Mode");

    // Initialize production UI system
    let ui_perf = PerformanceLogger::new(LogCategory::UI, "Production UI initialization");
    let mut integrated_ui = ui_integration::IntegratedUI::new((window_size.width as f32, window_size.height as f32));
    integrated_ui.handle_startup();
    ui_perf.finish();
    log_initialization_complete(LogCategory::UI, "Production UI system");

    // Create Production Demo State Manager (integrates ImGui + Production UI)
    let mut demo_state = DemoStateManager::new();
    log_initialization_complete(LogCategory::UI, "Production Demo State Manager");

    // Initialize physics system for voxel interactions
    demo_state.initialize_physics(renderer.get_device());
    log_initialization_complete(LogCategory::Engine, "Voxel Physics System with rapier3d");

    // Create Time-of-Day System
    let mut time_system = TimeOfDaySystem::new();
    log_initialization_complete(LogCategory::Engine, "Time-of-Day System");

    // Upload font texture to Metal and link to ImGui systems
    {
        // Link font texture for legacy ImGui system
        let imgui_system = demo_state.get_imgui_system_mut();
        if let Some(texture_data) = imgui_system.get_font_texture_data() {
            let (width, height) = imgui_system.get_font_texture_dimensions();
            match renderer.create_font_texture(texture_data, width, height) {
                Ok(texture_id) => {
                    imgui_system.set_font_texture_id(imgui::TextureId::from(texture_id as usize));
                    println!("🔤 Font texture linked to ImGui context via Demo State");
                }
                Err(e) => eprintln!("❌ Failed to create font texture: {}", e),
            }
        }

        // Link font texture for unified HUD system
        let unified_hud = demo_state.get_unified_hud_mut();
        if let Some(texture_data) = unified_hud.get_font_texture_data() {
            let (width, height) = unified_hud.get_font_texture_dimensions();
            match renderer.create_font_texture(texture_data, width, height) {
                Ok(texture_id) => {
                    unified_hud.set_font_texture_id(imgui::TextureId::from(texture_id as usize));
                    println!("🎯 Font texture linked to Unified HUD system");
                }
                Err(e) => eprintln!("❌ Failed to create unified HUD font texture: {}", e),
            }
        }
    }

    // Initialize UI rendering system
    if let Err(e) = renderer.initialize_ui() {
        eprintln!("❌ Failed to initialize UI rendering: {}", e);
    }

    // Initialize texture atlas
    {
        let atlas_data = TextureAtlas::generate_atlas_data();
        if let Err(e) = renderer.create_atlas_texture(&atlas_data, crate::renderer::texture_atlas::ATLAS_SIZE, crate::renderer::texture_atlas::ATLAS_SIZE) {
            eprintln!("❌ Failed to create atlas texture: {}", e);
        } else {
            println!("🎨 Texture atlas initialized successfully");
        }
    }

    // Initialize player representation renderer
    player_renderer.initialize(renderer.get_device());
    println!("🧑 Player representation initialized successfully");

    // Game state
    let mut keys_pressed = HashSet::new();
    let mut _mouse_grabbed = false;
    let start_time = Instant::now();
    let mut last_frame = Instant::now();
    let mut _ui_visible = true;

    // Initialize gameplay systems
    use crate::gameplay_systems::GameplayManager;
    let mut gameplay_manager = GameplayManager::new();
    log_initialization_complete(LogCategory::Engine, "Gameplay progression systems");

    // Initialize multiplayer collaboration system
    use crate::multiplayer_collaboration::{MultiplayerCollaboration, CollaborationEvent};
    use uuid::Uuid;
    let local_player_id = Uuid::new_v4();
    let mut multiplayer_system = MultiplayerCollaboration::new(local_player_id);
    log_initialization_complete(LogCategory::Engine, "Multiplayer collaboration framework");

    println!("✅ Initialization complete!");
    print_controls();

    // Main game loop
    while !window.should_close() {
        let current_time = Instant::now();
        let delta_time = current_time.duration_since(last_frame).as_secs_f32();
        let _elapsed_time = current_time.duration_since(start_time).as_secs_f32();
        last_frame = current_time;

        // Update time-of-day system
        time_system.update(delta_time);

        // Update gameplay systems
        gameplay_manager.update(delta_time);

        // Update multiplayer collaboration system
        if let Ok(collaboration_events) = pollster::block_on(multiplayer_system.update(delta_time)) {
            // Process collaboration events
            if let Ok(events) = pollster::block_on(multiplayer_system.process_messages()) {
                for event in events {
                    match event {
                        CollaborationEvent::RemoteBlockPlaced { position, material, player_id } => {
                            // Apply remote block placement to the world
                            log::info!("🔗 Remote block placed at {:?} by player {:?}", position, player_id);
                            // Here you would apply the block change to the voxel world
                        },
                        CollaborationEvent::RemoteBlockRemoved { position, player_id } => {
                            log::info!("🔗 Remote block removed at {:?} by player {:?}", position, player_id);
                        },
                        CollaborationEvent::PlayerJoined { player_id, username } => {
                            log::info!("🤝 Player {} joined the session", username);
                        },
                        CollaborationEvent::PlayerLeft { player_id } => {
                            log::info!("👋 Player {:?} left the session", player_id);
                        },
                        CollaborationEvent::PlayerCursorUpdate { player_id, position, selection } => {
                            // Update visual representation of other players' cursors
                        },
                        CollaborationEvent::ChatMessage { player_id, message, timestamp } => {
                            log::info!("💬 Chat from {:?}: {}", player_id, message);
                        },
                        _ => {}
                    }
                }
            }
        }

        // Display progress every 10 seconds
        static mut LAST_PROGRESS_LOG: f32 = 0.0;
        static mut CURRENT_TIME: f32 = 0.0;
        unsafe {
            CURRENT_TIME += delta_time;
            if CURRENT_TIME - LAST_PROGRESS_LOG > 10.0 {
                let summary = gameplay_manager.get_session_summary();
                println!("📊 {}", summary);
                LAST_PROGRESS_LOG = CURRENT_TIME;
            }
        }

        // Handle window events
        let events = window.poll_events();
        for event in events {
            // Pass events to production UI first
            handle_ui_event(&event, &mut integrated_ui);

            // Then handle regular events
            handle_event(&event, &mut camera, &mut world, &mut build_system, &mut world_mesh,
                        renderer.get_device(), &mut keys_pressed, &mut demo_state, &mut gameplay_manager, &mut multiplayer_system);
        }

        // Update production UI
        integrated_ui.update(delta_time);

        // Update character physics and camera from continuous input
        update_character_and_camera(&window, &mut camera, &mut character_state, &mut character_physics, &world, delta_time);

        // Update voxel physics system with rapier3d integration
        if let Err(e) = demo_state.update_physics(delta_time, &mut world) {
            log_error!(LogCategory::Engine, "Physics update error: {}", e);
        }

        // Update performance metrics with estimated chunk rendering statistics
        // In a real implementation, these would come from the actual rendering system
        let estimated_chunks_rendered = 64; // Estimated visible chunks
        let estimated_chunks_culled = 156;  // Estimated culled chunks (matches 92% culling efficiency)
        demo_state.update_performance_metrics(delta_time, estimated_chunks_rendered, estimated_chunks_culled);

        // Update world mesh if needed
        if world_mesh.vertices.is_empty() {
            generate_world_mesh(&world, &mut world_mesh, &camera);
            world_mesh.update_buffers(renderer.get_device());
        }

        // Update preview mesh based on cursor position
        if let Some(hit_pos) = raycast_world(&camera, &world) {
            let preview_pos = (hit_pos.0, hit_pos.1 + 1, hit_pos.2); // Place above hit
            let is_valid = true; // For now, assume all placements are valid
            generate_preview_mesh(&world, &mut preview_mesh, &build_system, preview_pos, is_valid);
            preview_mesh.update_buffers(renderer.get_device());
        } else {
            // Clear preview when not pointing at anything
            preview_mesh.clear();
            preview_mesh.update_buffers(renderer.get_device());
        }

        // Render frame
        if renderer.begin_frame() {
            // Update Unified HUD System (combines ImGui + Production UI)
            let window_size = window.get_size();
            let cgsize = core_graphics::geometry::CGSize {
                width: window_size.width as f64,
                height: window_size.height as f64,
            };

            let unified_hud = demo_state.get_unified_hud_mut();
            let (unified_actions, imgui_draw_data) = unified_hud.update_and_render(
                cgsize,
                &mut build_system,
                &camera,
                &character_state,
                delta_time,
                time_system.get_time_of_day(),
            );

            // Render 3D scene with UI overlay
            if let Some(ref draw_data) = imgui_draw_data {
                // Custom rendering with player representation
                if let Some(encoder) = renderer.begin_render_pass() {
                    // Render world
                    renderer.render_mesh(&encoder, &world_mesh);
                    renderer.render_mesh(&encoder, &preview_mesh);

                    // Render player body
                    player_renderer.render_player_body(
                        &encoder,
                        &renderer,
                        cgmath::Point3::new(camera.eye.x, camera.eye.y, camera.eye.z),
                        character_state.position
                    );

                    // Render legacy ImGui UI if welcome flow is complete
                    if integrated_ui.is_welcome_complete() {
                        renderer.render_ui(&encoder, draw_data);
                    }

                    // Render production UI overlay
                    integrated_ui.render(&mut renderer);

                    renderer.end_render_pass();
                } else {
                    // Fallback to original rendering
                    renderer.render_frame_with_ui(&world_mesh, &preview_mesh, &camera, time_system.get_time(), time_system.get_time_of_day(), Some(draw_data));
                }
            } else {
                // Render without UI but with player representation
                if let Some(encoder) = renderer.begin_render_pass() {
                    // Render world
                    renderer.render_mesh(&encoder, &world_mesh);
                    renderer.render_mesh(&encoder, &preview_mesh);

                    // Render player body
                    player_renderer.render_player_body(
                        &encoder,
                        &renderer,
                        cgmath::Point3::new(camera.eye.x, camera.eye.y, camera.eye.z),
                        character_state.position
                    );

                    // Always render production UI
                    integrated_ui.render(&mut renderer);

                    renderer.end_render_pass();
                } else {
                    // Fallback to original rendering
                    renderer.render_frame(&world_mesh, &camera, time_system.get_time(), time_system.get_time_of_day());
                }
            }

            // Handle Unified UI actions (from Unified HUD system)
            for action in unified_actions {
                handle_unified_ui_action(action, &mut build_system, &mut world, &mut world_mesh, renderer.get_device(), &mut time_system, &camera, &mut demo_state);
            }
        }

        // Small sleep to prevent 100% CPU usage
        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    Ok(())
}

// Helper functions for gameplay integration
fn voxel_type_to_material_type(voxel_type: VoxelType) -> crate::material_batching::MaterialType {
    use crate::material_batching::MaterialType;
    match voxel_type {
        VoxelType::Solid => MaterialType::Earth,
        VoxelType::Stone => MaterialType::Stone,
        VoxelType::Wood => MaterialType::Wood,
        VoxelType::Liquid => MaterialType::Water,
        VoxelType::Glass => MaterialType::Crystal,
        VoxelType::Metal => MaterialType::Stone, // Metal acts like stone for now
        VoxelType::Air => MaterialType::Air,
        _ => MaterialType::Earth, // Default fallback
    }
}

fn get_block_xp_value(material_type: crate::material_batching::MaterialType) -> u64 {
    use crate::material_batching::MaterialType;
    match material_type {
        MaterialType::Stone => 2,
        MaterialType::Wood => 3,
        MaterialType::Crystal => 10,
        MaterialType::Lava => 15,
        MaterialType::Water => 5,
        _ => 1,
    }
}

fn handle_ui_event(event: &WindowEvent, ui: &mut ui_integration::IntegratedUI) {
    match event {
        WindowEvent::MouseMove { x, y } => {
            ui.handle_mouse_move(*x as f32, *y as f32);
        }
        WindowEvent::MouseClick { button, x, y } => {
            let button_id = match button {
                MouseButton::Left => 0,
                MouseButton::Right => 1,
                MouseButton::Middle => 2,
            };
            ui.handle_mouse_click(*x as f32, *y as f32, button_id);
        }
        WindowEvent::KeyPress { key } => {
            ui.handle_key(*key, true);
        }
        WindowEvent::KeyRelease { key } => {
            ui.handle_key(*key, false);
        }
        WindowEvent::Resize { width, height } => {
            ui.resize(*width as f32, *height as f32);
        }
        _ => {}
    }
}

fn handle_event(
    event: &WindowEvent,
    camera: &mut Camera,
    world: &mut VoxelWorld,
    build_system: &mut VoxelBuildSystem,
    world_mesh: &mut Mesh,
    device: &metal::Device,
    keys_pressed: &mut HashSet<u16>,
    demo_state: &mut demo_state::DemoStateManager,
    gameplay_manager: &mut crate::gameplay_systems::GameplayManager,
    multiplayer_system: &mut crate::multiplayer_collaboration::MultiplayerCollaboration,
) {
    match event {
        WindowEvent::KeyPressed(key_code) => {
            keys_pressed.insert(*key_code);

            // UI events are handled separately at the top level

            match *key_code {
                key_codes::B => {
                    build_system.cycle_mode();
                    print_build_status(build_system);
                }
                key_codes::T => {
                    build_system.cycle_template();
                    print_build_status(build_system);
                }
                key_codes::G => {
                    build_system.toggle_grid_snap();
                    print_build_status(build_system);
                }
                key_codes::Z => {
                    if build_system.undo(world) {
                        regenerate_world_mesh(world, world_mesh, device, camera);
                    }
                }
                key_codes::Y => {
                    if build_system.redo(world) {
                        regenerate_world_mesh(world, world_mesh, device, camera);
                    }
                }
                key_codes::KEY_1..=key_codes::KEY_9 => {
                    let material_index = (*key_code - key_codes::KEY_1) as usize;
                    // Map material index to VoxelType
                    let materials = [
                        VoxelType::Stone,
                        VoxelType::Wood,
                        VoxelType::Solid,
                        VoxelType::Glass,
                        VoxelType::Metal,
                        VoxelType::Glass,
                        VoxelType::Brick,
                        VoxelType::Concrete,
                    ];
                    if let Some(material) = materials.get(material_index) {
                        build_system.set_material(*material);
                    }
                    print_build_status(build_system);
                }
                // Demo mode switching with F1-F6 keys
                key_codes::F1 => {
                    demo_state.switch_mode(DemoMode::InteractivePlayground);
                    println!("🎮 Switched to Interactive Playground mode");
                }
                key_codes::F2 => {
                    demo_state.switch_mode(DemoMode::EngineerBuildShowcase);
                    println!("🔧 Switched to Engineer Build Showcase mode");
                }
                key_codes::F3 => {
                    demo_state.switch_mode(DemoMode::GameplaySystemsDemo);
                    println!("🎯 Switched to Gameplay Systems Demo mode");
                }
                key_codes::F4 => {
                    demo_state.switch_mode(DemoMode::CollaborationPreview);
                    println!("🤝 Switched to Collaboration Preview mode");
                }
                key_codes::F5 => {
                    demo_state.switch_mode(DemoMode::PerformanceBenchmarks);
                    println!("⚡ Switched to Performance Benchmarks mode");
                }
                key_codes::F6 => {
                    demo_state.switch_mode(DemoMode::VisualShowcase);
                    println!("🎨 Switched to Visual Showcase mode");
                }
                key_codes::P => {
                    demo_state.toggle_physics_demo();
                    if demo_state.is_physics_demo_active() {
                        println!("🔬 Physics Demo ENABLED - floating blocks will become dynamic!");
                    } else {
                        println!("🚫 Physics Demo DISABLED - standard block placement");
                    }
                }
                key_codes::F7 => {
                    ui_system.toggle_performance_dashboard();
                    println!("📊 Performance Dashboard toggled - Press F7 to toggle");
                }
                _ => {}
            }
        }
        WindowEvent::KeyReleased(key_code) => {
            keys_pressed.remove(key_code);
        }
        WindowEvent::MousePressed(button) => {
            match button {
                MouseButton::Left => {
                    // Remove block with collaborative tracking
                    if let Some(hit_pos) = raycast_world(camera, world) {
                        // Get the voxel type before removal for resource tracking
                        let voxel_type = world.get_voxel(Vector3::new(hit_pos.0 as f32, hit_pos.1 as f32, hit_pos.2 as f32)).unwrap_or(VoxelType::Air);

                        // Use collaborative removal system
                        let position = [hit_pos.0, hit_pos.1, hit_pos.2];
                        if let Ok(_) = pollster::block_on(multiplayer_system.remove_block_collaborative(position)) {
                            if build_system.remove_block(world, hit_pos) {
                                regenerate_world_mesh(world, world_mesh, device, camera);

                                // Create physics debris for visual feedback
                                let debris_position = cgmath::Vector3::new(hit_pos.0 as f32, hit_pos.1 as f32, hit_pos.2 as f32);
                                let impact_velocity = camera.get_forward_vector() * 3.0; // Direction from camera
                                if let Err(e) = demo_state.create_physics_debris(voxel_type, debris_position, impact_velocity) {
                                    println!("⚠️ Failed to create debris: {}", e);
                                }

                                // Track the mining action for progression
                                gameplay_manager.on_block_destroyed();
                                println!("🗑️ Mined {:?} at {:?} - gained {} XP! (Collaborative + Physics debris)", voxel_type, hit_pos, 1);
                            }
                        }
                    }
                }
                MouseButton::Right => {
                    // Place block with physics integration and collaborative tracking
                    if let Some(hit_pos) = raycast_world(camera, world) {
                        let placed_pos = (hit_pos.0, hit_pos.1 + 1, hit_pos.2); // Place above hit
                        let voxel_type = build_system.get_current_material();

                        // Use collaborative placement system
                        let position = [placed_pos.0, placed_pos.1, placed_pos.2];
                        let material_type = voxel_type_to_material_type(voxel_type);

                        if let Ok(_) = pollster::block_on(multiplayer_system.place_block_collaborative(position, material_type)) {
                            // Check if block should be physics-enabled (floating blocks)
                            let block_below = world.get_voxel((placed_pos.0, placed_pos.1 - 1, placed_pos.2));
                            let is_floating = block_below == robin::engine::generation::voxel_system::VoxelType::Air;

                            if is_floating && demo_state.is_physics_demo_active() {
                                // Create dynamic physics block for floating placement
                                let block_position = cgmath::Vector3::new(
                                    placed_pos.0 as f32 + 0.5,  // Center of voxel
                                    placed_pos.1 as f32 + 0.5,
                                    placed_pos.2 as f32 + 0.5
                                );
                                let initial_velocity = cgmath::Vector3::new(0.0, 0.0, 0.0); // Start at rest

                                if let Err(e) = demo_state.create_dynamic_voxel_block(voxel_type, block_position, initial_velocity) {
                                    println!("⚠️ Failed to create dynamic block, placing static: {}", e);
                                    // Fallback to static placement
                                    if build_system.build_at_position(world, placed_pos) {
                                        regenerate_world_mesh(world, world_mesh, device, camera);
                                    }
                                } else {
                                    println!("🎯 Created dynamic physics block {:?} at {:?}", voxel_type, block_position);
                                }
                            } else {
                                // Standard static block placement
                                if build_system.build_at_position(world, placed_pos) {
                                    regenerate_world_mesh(world, world_mesh, device, camera);

                                    // Create placement particles for visual feedback
                                    let placement_position = cgmath::Vector3::new(placed_pos.0 as f32, placed_pos.1 as f32, placed_pos.2 as f32);
                                    let upward_velocity = cgmath::Vector3::new(0.0, 2.0, 0.0); // Gentle upward motion
                                    if let Err(e) = demo_state.create_physics_debris(voxel_type, placement_position, upward_velocity) {
                                        println!("⚠️ Failed to create placement particles: {}", e);
                                    }
                                }
                            }

                            // Track the building action for progression
                            gameplay_manager.on_block_placed(material_type);
                            let xp = get_block_xp_value(material_type);
                            println!("🏗️ Placed {:?} at {:?} - gained {} XP! (Collaborative + Physics)", voxel_type, placed_pos, xp);
                        }
                    }
                }
                MouseButton::Middle => {
                    println!("🖱️  Mouse grab toggled");
                }
            }
        }
        WindowEvent::MouseMoved(delta) => {
            if keys_pressed.contains(&key_codes::ESCAPE) == false {
                // Only apply mouse look if escape isn't pressed
                camera.update_from_input(0.0, 0.0, 0.0, delta.x as f32, delta.y as f32);
            }
        }
        _ => {}
    }
}

fn update_character_and_camera(
    window: &NativeWindow,
    camera: &mut Camera,
    character_state: &mut robin::engine::character::CharacterState,
    character_physics: &mut robin::engine::character::CharacterPhysics,
    world: &VoxelWorld,
    delta_time: f32
) {
    use cgmath::{InnerSpace, Vector3};
    use nalgebra::{Vector3 as NaVector3, Point3 as NaPoint3};

    // Gather input for movement
    let mut movement_input = NaVector3::new(0.0, 0.0, 0.0);
    let speed_multiplier = if window.is_key_pressed(key_codes::LEFT_SHIFT) { 2.0 } else { 1.0 };
    let base_speed = 5.0 * speed_multiplier;

    // Get camera forward and right vectors for movement direction
    let forward = camera.get_forward_vector();
    let right = camera.get_right_vector();

    if window.is_key_pressed(key_codes::W) {
        movement_input += NaVector3::new(forward.x, 0.0, forward.z).normalize() * base_speed;
    }
    if window.is_key_pressed(key_codes::S) {
        movement_input -= NaVector3::new(forward.x, 0.0, forward.z).normalize() * base_speed;
    }
    if window.is_key_pressed(key_codes::A) {
        movement_input -= NaVector3::new(right.x, 0.0, right.z).normalize() * base_speed;
    }
    if window.is_key_pressed(key_codes::D) {
        movement_input += NaVector3::new(right.x, 0.0, right.z).normalize() * base_speed;
    }

    // Handle jumping (space key)
    let jump_requested = window.is_key_pressed(key_codes::SPACE);

    // Update character physics with movement and world collision
    character_physics.update_movement(character_state, movement_input, jump_requested, world, delta_time);

    // Update character state from physics
    character_physics.update_character_state(character_state, delta_time);

    // Sync camera position with character (first-person view)
    // Position camera at eye height above character position
    let eye_height = character_physics.get_eye_height();
    let character_pos = character_state.position;
    camera.eye = cgmath::Point3::new(
        character_pos.x,
        character_pos.y + eye_height,
        character_pos.z,
    );

    // Handle enhanced camera movement for looking around
    // Check for mouse movement in window
    let mouse_delta = window.get_mouse_delta();
    if mouse_delta.x != 0.0 || mouse_delta.y != 0.0 {
        // Use enhanced camera movement for smooth look
        camera.update_from_input_with_delta(0.0, 0.0, 0.0, mouse_delta.x as f32, mouse_delta.y as f32, delta_time);
    }

    // Update camera target to maintain current view direction
    let forward_vector = camera.get_forward_vector();
    camera.target = camera.eye + forward_vector;
}

struct TimeOfDaySystem {
    time_of_day: f32,     // Current time in hours (0.0 - 24.0)
    time_speed: f32,      // Speed multiplier (1.0 = normal, 0.0 = paused)
    is_paused: bool,      // Whether time is paused
    // Enhanced weather system from archived demos
    weather_type: WeatherType,
    weather_intensity: f32,  // 0.0 to 1.0
    fog_density: f32,        // Enhanced atmospheric effects
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum WeatherType {
    Clear,
    Cloudy,
    Rain,
    Snow,
    Storm,
    Fog,
}

impl TimeOfDaySystem {
    fn new() -> Self {
        Self {
            time_of_day: 12.0, // Start at noon
            time_speed: 1.0,   // Normal speed
            is_paused: false,
            // Initialize with clear weather
            weather_type: WeatherType::Clear,
            weather_intensity: 0.5,
            fog_density: 0.1,
        }
    }

    fn update(&mut self, delta_time: f32) {
        if !self.is_paused {
            // Convert real seconds to game hours
            // 1 real second = 0.1 game hours (so 10 seconds = 1 game hour)
            self.time_of_day += delta_time * self.time_speed * 0.1;

            // Wrap around 24 hours
            if self.time_of_day >= 24.0 {
                self.time_of_day -= 24.0;
            } else if self.time_of_day < 0.0 {
                self.time_of_day += 24.0;
            }
        }
    }

    fn set_time(&mut self, hours: f32) {
        self.time_of_day = hours.max(0.0).min(24.0);
    }

    fn set_speed(&mut self, speed: f32) {
        self.time_speed = speed.max(0.0).min(10.0); // Cap at 10x speed
    }

    fn toggle_pause(&mut self) {
        self.is_paused = !self.is_paused;
    }

    fn get_time(&self) -> f32 {
        self.time_of_day
    }

    fn get_time_of_day(&self) -> f32 {
        self.time_of_day
    }

    fn get_time_string(&self) -> String {
        let hours = self.time_of_day.floor() as u32;
        let minutes = ((self.time_of_day.fract() * 60.0).floor()) as u32;
        let period = if hours < 12 { "AM" } else { "PM" };
        let display_hours = if hours == 0 { 12 } else if hours > 12 { hours - 12 } else { hours };
        format!("{:02}:{:02} {}", display_hours, minutes, period)
    }

    fn get_day_phase(&self) -> &'static str {
        match self.time_of_day {
            t if t >= 6.0 && t < 12.0 => "Morning",
            t if t >= 12.0 && t < 18.0 => "Afternoon",
            t if t >= 18.0 && t < 21.0 => "Evening",
            _ => "Night",
        }
    }

    fn is_paused(&self) -> bool {
        self.is_paused
    }

    fn get_speed(&self) -> f32 {
        self.time_speed
    }

    // Enhanced weather methods from archived demos
    fn get_weather(&self) -> (WeatherType, f32) {
        (self.weather_type, self.weather_intensity)
    }

    fn set_weather(&mut self, weather: WeatherType, intensity: f32) {
        self.weather_type = weather;
        self.weather_intensity = intensity.clamp(0.0, 1.0);

        // Adjust fog based on weather
        self.fog_density = match weather {
            WeatherType::Clear => 0.05,
            WeatherType::Cloudy => 0.15,
            WeatherType::Rain => 0.25,
            WeatherType::Snow => 0.35,
            WeatherType::Storm => 0.45,
            WeatherType::Fog => 0.8,
        };
    }

    fn get_weather_description(&self) -> String {
        let base = match self.weather_type {
            WeatherType::Clear => "Clear skies",
            WeatherType::Cloudy => "Cloudy",
            WeatherType::Rain => "Rainy",
            WeatherType::Snow => "Snowy",
            WeatherType::Storm => "Stormy",
            WeatherType::Fog => "Foggy",
        };

        let intensity = match (self.weather_intensity * 10.0) as u32 {
            0..=3 => "Light",
            4..=6 => "Moderate",
            7..=8 => "Heavy",
            _ => "Extreme",
        };

        if self.weather_type == WeatherType::Clear {
            base.to_string()
        } else {
            format!("{} {}", intensity, base.to_lowercase())
        }
    }

    fn get_fog_density(&self) -> f32 {
        self.fog_density
    }
}

fn raycast_world(camera: &Camera, world: &VoxelWorld) -> Option<(i32, i32, i32)> {
    let forward = camera.get_forward_vector();
    let origin = Vector3::new(camera.eye.x, camera.eye.y, camera.eye.z);
    raycast_voxel_world(world, origin, forward, 10.0)
}

fn regenerate_world_mesh(world: &VoxelWorld, world_mesh: &mut Mesh, device: &metal::Device, camera: &Camera) {
    world_mesh.clear();
    generate_world_mesh(world, world_mesh, camera);
    world_mesh.update_buffers(device);
}

fn generate_preview_mesh(
    world: &VoxelWorld,
    preview_mesh: &mut Mesh,
    build_system: &VoxelBuildSystem,
    preview_pos: (i32, i32, i32),
    is_valid: bool,
) {
    preview_mesh.clear();
    let texture_atlas = TextureAtlas::new();

    // Choose preview color: green for valid, red for invalid placement
    let preview_color = if is_valid { [0.0, 1.0, 0.0] } else { [1.0, 0.0, 0.0] };

    match build_system.get_mode() {
        BuildMode::Single => {
            // Single block preview
            if world.get_voxel(Vector3::new(preview_pos.0 as f32, preview_pos.1 as f32, preview_pos.2 as f32)).unwrap_or(VoxelType::Air) == VoxelType::Air {
                add_ghost_block(preview_mesh, preview_pos, preview_color, &texture_atlas);
            }
        },
        BuildMode::Wall => {
            // 5x3 wall preview
            for x_offset in 0..5 {
                for y_offset in 0..3 {
                    let block_pos = (preview_pos.0 + x_offset, preview_pos.1 + y_offset, preview_pos.2);
                    if world.get_voxel(Vector3::new(block_pos.0 as f32, block_pos.1 as f32, block_pos.2 as f32)).unwrap_or(VoxelType::Air) == VoxelType::Air {
                        add_ghost_block(preview_mesh, block_pos, preview_color, &texture_atlas);
                    }
                }
            }
        },
        BuildMode::Floor => {
            // 5x5 floor preview
            for x_offset in -2..=2 {
                for z_offset in -2..=2 {
                    let block_pos = (preview_pos.0 + x_offset, preview_pos.1, preview_pos.2 + z_offset);
                    if world.get_voxel(Vector3::new(block_pos.0 as f32, block_pos.1 as f32, block_pos.2 as f32)).unwrap_or(VoxelType::Air) == VoxelType::Air {
                        add_ghost_block(preview_mesh, block_pos, preview_color, &texture_atlas);
                    }
                }
            }
        },
        BuildMode::Roof => {
            // Pyramid roof preview
            let layers = vec![
                (3, -1, 1), // 3x3 base, offset -1 from center, Y+1
                (2, 0, 2),  // 2x2 middle, no offset, Y+2
                (1, 0, 3),  // 1x1 top, no offset, Y+3
            ];

            for (size, offset, y_level) in layers {
                let half_size = size / 2;
                for x_offset in -half_size..=half_size {
                    for z_offset in -half_size..=half_size {
                        let block_pos = (preview_pos.0 + x_offset + offset, preview_pos.1 + y_level, preview_pos.2 + z_offset + offset);
                        if world.get_voxel(Vector3::new(block_pos.0 as f32, block_pos.1 as f32, block_pos.2 as f32)).unwrap_or(VoxelType::Air) == VoxelType::Air {
                            add_ghost_block(preview_mesh, block_pos, preview_color, &texture_atlas);
                        }
                    }
                }
            }
        },
        BuildMode::Template => {
            // Template-specific previews
            match build_system.get_current_template() {
                TemplateType::Stairs => {
                    // 4-step staircase preview
                    for step in 0..4 {
                        let block_pos = (preview_pos.0 + step, preview_pos.1 + step, preview_pos.2);
                        if world.get_voxel(Vector3::new(block_pos.0 as f32, block_pos.1 as f32, block_pos.2 as f32)).unwrap_or(VoxelType::Air) == VoxelType::Air {
                            add_ghost_block(preview_mesh, block_pos, preview_color, &texture_atlas);
                        }
                    }
                },
                TemplateType::Arch => {
                    // 5×4 arch doorway preview
                    let arch_pattern = [
                        // Base level (Y+0) - pillars only
                        (-2, 0, true), (-1, 0, false), (0, 0, false), (1, 0, false), (2, 0, true),
                        // Lower sides (Y+1) - pillars only
                        (-2, 1, true), (-1, 1, false), (0, 1, false), (1, 1, false), (2, 1, true),
                        // Upper sides (Y+2) - pillars only
                        (-2, 2, true), (-1, 2, false), (0, 2, false), (1, 2, false), (2, 2, true),
                        // Top (Y+3) - full arch span
                        (-2, 3, true), (-1, 3, true), (0, 3, true), (1, 3, true), (2, 3, true),
                    ];

                    for (x_offset, y_offset, should_place) in arch_pattern.iter() {
                        if *should_place {
                            let block_pos = (preview_pos.0 + x_offset, preview_pos.1 + y_offset, preview_pos.2);
                            if world.get_voxel(Vector3::new(block_pos.0 as f32, block_pos.1 as f32, block_pos.2 as f32)).unwrap_or(VoxelType::Air) == VoxelType::Air {
                                add_ghost_block(preview_mesh, block_pos, preview_color, &texture_atlas);
                            }
                        }
                    }
                },
                TemplateType::Bridge => {
                    // 7×3 walkway with railings preview
                    for x_offset in 0..7 {
                        // Walkway floor (3-wide in center)
                        for z_offset in -1..=1 {
                            let block_pos = (preview_pos.0 + x_offset, preview_pos.1, preview_pos.2 + z_offset);
                            if world.get_voxel(Vector3::new(block_pos.0 as f32, block_pos.1 as f32, block_pos.2 as f32)).unwrap_or(VoxelType::Air) == VoxelType::Air {
                                add_ghost_block(preview_mesh, block_pos, preview_color, &texture_atlas);
                            }
                        }

                        // Railings (sides of the bridge, height 2)
                        for railing_height in 1..=2 {
                            // Left railing
                            let left_pos = (preview_pos.0 + x_offset, preview_pos.1 + railing_height, preview_pos.2 - 2);
                            if world.get_voxel(Vector3::new(left_pos.0 as f32, left_pos.1 as f32, left_pos.2 as f32)).unwrap_or(VoxelType::Air) == VoxelType::Air {
                                add_ghost_block(preview_mesh, left_pos, preview_color, &texture_atlas);
                            }

                            // Right railing
                            let right_pos = (preview_pos.0 + x_offset, preview_pos.1 + railing_height, preview_pos.2 + 2);
                            if world.get_voxel(Vector3::new(right_pos.0 as f32, right_pos.1 as f32, right_pos.2 as f32)).unwrap_or(VoxelType::Air) == VoxelType::Air {
                                add_ghost_block(preview_mesh, right_pos, preview_color, &texture_atlas);
                            }
                        }
                    }
                },
                TemplateType::Tower => {
                    // 3×3×8 cylindrical tower with battlements preview
                    // Tower walls (outer ring of 3×3, excluding corners for cylinder effect)
                    let wall_positions = [
                        (0, 1), (1, 0), (1, 2), (2, 1), // Cross pattern for cylinder
                    ];

                    // Main tower structure (8 blocks high)
                    for height in 0..8 {
                        for (x_offset, z_offset) in wall_positions.iter() {
                            let block_pos = (preview_pos.0 + x_offset - 1, preview_pos.1 + height, preview_pos.2 + z_offset - 1);
                            if world.get_voxel(Vector3::new(block_pos.0 as f32, block_pos.1 as f32, block_pos.2 as f32)).unwrap_or(VoxelType::Air) == VoxelType::Air {
                                add_ghost_block(preview_mesh, block_pos, preview_color, &texture_atlas);
                            }
                        }
                    }

                    // Battlements (crenellated top at height 8)
                    let battlement_positions = [
                        (0, 1), (1, 2), (2, 1), (1, 0), // Alternating pattern
                    ];
                    for (x_offset, z_offset) in battlement_positions.iter() {
                        let block_pos = (preview_pos.0 + x_offset - 1, preview_pos.1 + 8, preview_pos.2 + z_offset - 1);
                        if world.get_voxel(Vector3::new(block_pos.0 as f32, block_pos.1 as f32, block_pos.2 as f32)).unwrap_or(VoxelType::Air) == VoxelType::Air {
                            add_ghost_block(preview_mesh, block_pos, preview_color, &texture_atlas);
                        }
                    }
                },
                TemplateType::House => {
                    // 5×5×4 house with door and windows preview
                    // Foundation (5×5 floor)
                    for x_offset in 0..5 {
                        for z_offset in 0..5 {
                            let block_pos = (preview_pos.0 + x_offset - 2, preview_pos.1, preview_pos.2 + z_offset - 2);
                            if world.get_voxel(Vector3::new(block_pos.0 as f32, block_pos.1 as f32, block_pos.2 as f32)).unwrap_or(VoxelType::Air) == VoxelType::Air {
                                add_ghost_block(preview_mesh, block_pos, preview_color, &texture_atlas);
                            }
                        }
                    }

                    // Walls (levels 1-3) with door and window openings
                    for height in 1..=3 {
                        for x_offset in 0..5 {
                            for z_offset in 0..5 {
                                // Only place blocks on the perimeter
                                if x_offset == 0 || x_offset == 4 || z_offset == 0 || z_offset == 4 {
                                    let block_pos = (preview_pos.0 + x_offset - 2, preview_pos.1 + height, preview_pos.2 + z_offset - 2);

                                    // Skip door opening (front wall, center, levels 1-2)
                                    if z_offset == 0 && x_offset == 2 && height <= 2 {
                                        continue;
                                    }

                                    // Skip window openings (side walls, center, level 2)
                                    if height == 2 && ((x_offset == 0 && z_offset == 2) || (x_offset == 4 && z_offset == 2)) {
                                        continue;
                                    }

                                    if world.get_voxel(Vector3::new(block_pos.0 as f32, block_pos.1 as f32, block_pos.2 as f32)).unwrap_or(VoxelType::Air) == VoxelType::Air {
                                        add_ghost_block(preview_mesh, block_pos, preview_color, &texture_atlas);
                                    }
                                }
                            }
                        }
                    }

                    // Roof (5×5 at level 4)
                    for x_offset in 0..5 {
                        for z_offset in 0..5 {
                            let block_pos = (preview_pos.0 + x_offset - 2, preview_pos.1 + 4, preview_pos.2 + z_offset - 2);
                            if world.get_voxel(Vector3::new(block_pos.0 as f32, block_pos.1 as f32, block_pos.2 as f32)).unwrap_or(VoxelType::Air) == VoxelType::Air {
                                add_ghost_block(preview_mesh, block_pos, preview_color, &texture_atlas);
                            }
                        }
                    }
                },
                _ => {
                    // Other templates - just show single block for now
                    add_ghost_block(preview_mesh, preview_pos, preview_color, &texture_atlas);
                }
            }
        },
        // Enhanced build modes - basic implementations for now
        BuildMode::Circle => {
            // Simple circle preview
            add_ghost_block(preview_mesh, preview_pos, preview_color, &texture_atlas);
        },
        BuildMode::Sphere => {
            // Simple sphere preview
            add_ghost_block(preview_mesh, preview_pos, preview_color, &texture_atlas);
        },
        BuildMode::Terrain => {
            // Terrain sculpting preview
            add_ghost_block(preview_mesh, preview_pos, preview_color, &texture_atlas);
        },
        BuildMode::Copy => {
            // Copy mode preview
            add_ghost_block(preview_mesh, preview_pos, preview_color, &texture_atlas);
        },
        BuildMode::Paste => {
            // Paste mode preview
            add_ghost_block(preview_mesh, preview_pos, preview_color, &texture_atlas);
        },
    }
}

/// Enhanced ghost block with material-specific textures and transparency effects
fn add_ghost_block(mesh: &mut Mesh, pos: (i32, i32, i32), color: [f32; 3], texture_atlas: &TextureAtlas) {
    add_enhanced_ghost_block(mesh, pos, color, texture_atlas, VoxelType::Stone, 0.6);
}

/// Enhanced ghost block with material preview and transparency
fn add_enhanced_ghost_block(
    mesh: &mut Mesh,
    pos: (i32, i32, i32),
    base_color: [f32; 3],
    texture_atlas: &TextureAtlas,
    material: VoxelType,
    alpha: f32
) {
    let (x, y, z) = (pos.0 as f32, pos.1 as f32, pos.2 as f32);

    // Get UV coordinates for the specific material being placed
    let tile_uv = texture_atlas.get_uv(material);

    // Create material-specific color tinting with transparency
    let material_color = get_material_preview_color(material, base_color, alpha);

    // Define all 6 faces of a cube with slight inset for ghost effect
    let inset = 0.02; // Small inset to create visual distinction
    let faces = [
        // Front (+Z) - slightly inset
        ([[x + inset, y + inset, z + 1.0 - inset],
          [x + 1.0 - inset, y + inset, z + 1.0 - inset],
          [x + 1.0 - inset, y + 1.0 - inset, z + 1.0 - inset],
          [x + inset, y + 1.0 - inset, z + 1.0 - inset]], [0.0, 0.0, 1.0]),
        // Back (-Z)
        ([[x + 1.0 - inset, y + inset, z + inset],
          [x + inset, y + inset, z + inset],
          [x + inset, y + 1.0 - inset, z + inset],
          [x + 1.0 - inset, y + 1.0 - inset, z + inset]], [0.0, 0.0, -1.0]),
        // Right (+X)
        ([[x + 1.0 - inset, y + inset, z + 1.0 - inset],
          [x + 1.0 - inset, y + inset, z + inset],
          [x + 1.0 - inset, y + 1.0 - inset, z + inset],
          [x + 1.0 - inset, y + 1.0 - inset, z + 1.0 - inset]], [1.0, 0.0, 0.0]),
        // Left (-X)
        ([[x + inset, y + inset, z + inset],
          [x + inset, y + inset, z + 1.0 - inset],
          [x + inset, y + 1.0 - inset, z + 1.0 - inset],
          [x + inset, y + 1.0 - inset, z + inset]], [-1.0, 0.0, 0.0]),
        // Top (+Y)
        ([[x + inset, y + 1.0 - inset, z + 1.0 - inset],
          [x + 1.0 - inset, y + 1.0 - inset, z + 1.0 - inset],
          [x + 1.0 - inset, y + 1.0 - inset, z + inset],
          [x + inset, y + 1.0 - inset, z + inset]], [0.0, 1.0, 0.0]),
        // Bottom (-Y)
        ([[x + inset, y + inset, z + inset],
          [x + 1.0 - inset, y + inset, z + inset],
          [x + 1.0 - inset, y + inset, z + 1.0 - inset],
          [x + inset, y + inset, z + 1.0 - inset]], [0.0, -1.0, 0.0]),
    ];

    // Add all faces with material-specific appearance
    for (vertices, normal) in faces.iter() {
        mesh.add_quad_with_uv(*vertices, material_color, *normal, tile_uv.coords);
    }
}

/// Get material-specific preview color with transparency
fn get_material_preview_color(material: VoxelType, base_color: [f32; 3], alpha: f32) -> [f32; 3] {
    let material_tint = match material {
        VoxelType::Stone => [0.5, 0.5, 0.5],    // Gray tint
        VoxelType::Earth => [0.6, 0.4, 0.2],    // Brown tint
        VoxelType::Water => [0.2, 0.4, 0.8],    // Blue tint
        VoxelType::Grass => [0.2, 0.8, 0.2],    // Green tint
        VoxelType::Sand => [0.9, 0.8, 0.5],     // Yellow tint
        VoxelType::Wood => [0.6, 0.3, 0.1],     // Dark brown tint
        VoxelType::Metal => [0.7, 0.7, 0.8],    // Metallic tint
        VoxelType::Air => [1.0, 1.0, 1.0],      // White (shouldn't be used)
    };

    // Blend base color with material tint and apply transparency effect
    [
        (base_color[0] * 0.3 + material_tint[0] * 0.7) * alpha + 0.3 * (1.0 - alpha),
        (base_color[1] * 0.3 + material_tint[1] * 0.7) * alpha + 0.3 * (1.0 - alpha),
        (base_color[2] * 0.3 + material_tint[2] * 0.7) * alpha + 0.3 * (1.0 - alpha),
    ]
}

fn print_controls() {
    println!();
    println!("═══════════════════════════════════════════════════════════");
    println!("        🏗️  ROBIN ENGINEER BUILD MODE - METAL DEMO  🏗️        ");
    println!("═══════════════════════════════════════════════════════════");
    println!("🎮 Movement Controls:");
    println!("   WASD        - Move around");
    println!("   Mouse       - Look around");
    println!("   Space/Shift - Move up/down");
    println!();
    println!("🔧 Engineer Build Controls:");
    println!("   Left Click  - Remove blocks");
    println!("   Right Click - Place blocks / Build shapes");
    println!("   B           - Cycle build modes (Single→Wall→Floor→Roof→Template)");
    println!("   T           - Change template");
    println!("   G           - Toggle grid snap alignment");
    println!("   Z           - Undo last action");
    println!("   Y           - Redo last action");
    println!("   Tab         - Toggle UI overlay");
    println!();
    println!("📦 Material Selection (1-8):");
    println!("   1 - Stone    5 - Water");
    println!("   2 - Dirt     6 - Wood");
    println!("   3 - Grass    7 - Crystal (emissive)");
    println!("   4 - Sand     8 - Lava (emissive)");
    println!("═══════════════════════════════════════════════════════════");
    println!("🎯 Start building with native Metal performance!");
    println!("═══════════════════════════════════════════════════════════");
    println!();
}

fn print_build_status(build_system: &VoxelBuildSystem) {
    println!();
    println!("═══════════════ ENGINEER BUILD MODE ═══════════════");
    println!("🔧 Current Mode: {:?}", build_system.get_current_mode());
    println!("📦 Current Material: {:?}", build_system.get_current_material());
    println!("⚡ Grid Snap: {}", if build_system.is_grid_snap_enabled() { "ON" } else { "OFF" });

    println!();
    println!("📊 Material Inventory:");
    for (material, count) in build_system.get_inventory() {
        println!("   {:?}: {}", material, count);
    }
    println!("══════════════════════════════════════════════════");
    println!();
}

// VoxelType is already imported above in the working imports

// For backward compatibility, re-export types in a game module
mod game {
    use cgmath::Vector3;
    use std::collections::VecDeque;
    use serde::{Serialize, Deserialize};
    use super::{VoxelType, VoxelWorld, BuildMode, TemplateType};

    // Re-export VoxelType to avoid conflicts
    pub type GameVoxelType = super::VoxelType;


    pub struct Chunk {
        pub voxels: Vec<Vec<Vec<VoxelType>>>,
        pub needs_rebuild: bool,
        pub size: usize,
    }

    impl Chunk {
        pub fn new(size: usize) -> Self {
            Self {
                voxels: vec![vec![vec![VoxelType::Air; size]; size]; size],
                needs_rebuild: true,
                size,
            }
        }

        pub fn get_voxel(&self, x: usize, y: usize, z: usize) -> VoxelType {
            if x < self.size && y < self.size && z < self.size {
                self.voxels[x][y][z]
            } else {
                VoxelType::Air
            }
        }

        pub fn set_voxel(&mut self, x: usize, y: usize, z: usize, voxel_type: VoxelType) {
            if x < self.size && y < self.size && z < self.size {
                self.voxels[x][y][z] = voxel_type;
                self.needs_rebuild = true;
            }
        }
    }

    // Using Robin engine's VoxelWorld directly

    // Using Robin engine's BuildMode and TemplateType (imported at top)

    #[derive(Debug)]
    pub struct VoxelBuildSystem {
        mode: BuildMode,
        current_material: VoxelType,
        current_template: TemplateType,
        inventory: Vec<(VoxelType, u32)>, // Using Vec instead of HashMap to avoid Hash trait requirement
        grid_snap: bool,
        undo_stack: VecDeque<BuildAction>,
        redo_stack: VecDeque<BuildAction>,
    }

    #[derive(Debug, Clone)]
    struct BuildAction {
        operations: Vec<BuildOperation>,
        description: String,
    }

    #[derive(Debug, Clone)]
    struct BuildOperation {
        position: (i32, i32, i32),
        old_voxel: VoxelType,
        new_voxel: VoxelType,
    }

    impl VoxelBuildSystem {
        pub fn new() -> Self {
            let inventory = vec![
                (VoxelType::Stone, 999),
                (VoxelType::Wood, 999),
                (VoxelType::Solid, 999),
                (VoxelType::Glass, 250),
                (VoxelType::Metal, 500),
                (VoxelType::Brick, 999),
                (VoxelType::Concrete, 999),
                (VoxelType::Liquid, 100),
            ];

            Self {
                mode: BuildMode::Single,
                current_material: VoxelType::Stone,
                current_template: TemplateType::Stairs,
                inventory,
                grid_snap: true,
                undo_stack: VecDeque::new(),
                redo_stack: VecDeque::new(),
            }
        }

        pub fn cycle_build_mode(&mut self) {
            self.mode = match self.mode {
                BuildMode::Single => BuildMode::Wall,
                BuildMode::Wall => BuildMode::Floor,
                BuildMode::Floor => BuildMode::Roof,
                BuildMode::Roof => BuildMode::Template,
                BuildMode::Template => BuildMode::Circle,
                BuildMode::Circle => BuildMode::Sphere,
                BuildMode::Sphere => BuildMode::Terrain,
                BuildMode::Terrain => BuildMode::Copy,
                BuildMode::Copy => BuildMode::Paste,
                BuildMode::Paste => BuildMode::Single,
            };
        }

        pub fn cycle_template(&mut self) {
            self.current_template = match self.current_template {
                TemplateType::Stairs => TemplateType::Arch,
                TemplateType::Arch => TemplateType::Bridge,
                TemplateType::Bridge => TemplateType::Tower,
                TemplateType::Tower => TemplateType::House,
                TemplateType::House => TemplateType::Castle,
                TemplateType::Castle => TemplateType::Garden,
                TemplateType::Garden => TemplateType::Workshop,
                TemplateType::Workshop => TemplateType::Fortress,
                TemplateType::Fortress => TemplateType::Lighthouse,
                TemplateType::Lighthouse => TemplateType::Windmill,
                TemplateType::Windmill => TemplateType::Stairs,
            };
            println!("🏗️ Template changed to: {:?}", self.current_template);
        }

        pub fn toggle_grid_snap(&mut self) {
            self.grid_snap = !self.grid_snap;
        }

        pub fn get_current_template(&self) -> TemplateType {
            self.current_template
        }

        pub fn get_mode(&self) -> BuildMode {
            self.mode
        }

        pub fn select_material_by_index(&mut self, index: usize) {
            let materials = [
                VoxelType::Stone, VoxelType::Wood, VoxelType::Solid, VoxelType::Glass,
                VoxelType::Metal, VoxelType::Brick, VoxelType::Concrete, VoxelType::Liquid,
            ];

            if index < materials.len() {
                self.current_material = materials[index];
            }
        }

        pub fn remove_block(&mut self, world: &mut VoxelWorld, pos: (i32, i32, i32)) -> bool {
            let world_pos = Vector3::new(pos.0 as f32, pos.1 as f32, pos.2 as f32);
            let old_voxel = world.get_voxel(world_pos).unwrap_or(VoxelType::Air);
            if old_voxel != VoxelType::Air {
                world.set_voxel(world_pos, VoxelType::Air);

                // Add to inventory
                if let Some((_, count)) = self.inventory.iter_mut().find(|(mat, _)| *mat == old_voxel) {
                    *count += 1;
                }

                // Add to undo stack
                let action = BuildAction {
                    operations: vec![BuildOperation {
                        position: pos,
                        old_voxel,
                        new_voxel: VoxelType::Air,
                    }],
                    description: "Remove block".to_string(),
                };
                self.add_action(action);

                println!("⛏️  *THUD* Block removed");
                true
            } else {
                false
            }
        }

        pub fn build_at_position(&mut self, world: &mut VoxelWorld, pos: (i32, i32, i32)) -> bool {
            match self.mode {
                BuildMode::Single => self.place_single_block(world, pos),
                BuildMode::Wall => self.build_wall(world, pos),
                BuildMode::Floor => self.build_floor(world, pos),
                BuildMode::Roof => self.build_roof(world, pos),
                BuildMode::Template => self.build_template(world, pos),
                // Enhanced build modes - basic implementations for now
                BuildMode::Circle => self.place_single_block(world, pos),
                BuildMode::Sphere => self.place_single_block(world, pos),
                BuildMode::Terrain => self.place_single_block(world, pos),
                BuildMode::Copy => self.place_single_block(world, pos),
                BuildMode::Paste => self.place_single_block(world, pos),
            }
        }

        fn place_single_block(&mut self, world: &mut VoxelWorld, pos: (i32, i32, i32)) -> bool {
            let world_pos = Vector3::new(pos.0 as f32, pos.1 as f32, pos.2 as f32);
            if world.get_voxel(world_pos).unwrap_or(VoxelType::Air) == VoxelType::Air {
                // Check if we have materials
                if let Some((_, count)) = self.inventory.iter_mut().find(|(mat, _)| *mat == self.current_material) {
                    if *count > 0 {
                        *count -= 1;
                        world.set_voxel(world_pos, self.current_material);

                        let action = BuildAction {
                            operations: vec![BuildOperation {
                                position: pos,
                                old_voxel: VoxelType::Air,
                                new_voxel: self.current_material,
                            }],
                            description: "Place block".to_string(),
                        };
                        self.add_action(action);

                        println!("🔨 *CLINK* Block placed");
                        return true;
                    }
                }
            }
            false
        }

        fn build_wall(&mut self, world: &mut VoxelWorld, pos: (i32, i32, i32)) -> bool {
            // Build a 5-wide x 3-high wall extending in the X direction from the clicked position
            let blocks_needed = 5 * 3; // 15 blocks total

            // Check if we have enough materials
            if let Some((_, count)) = self.inventory.iter().find(|(mat, _)| *mat == self.current_material) {
                if *count < blocks_needed {
                    println!("❌ Not enough materials! Need {} blocks, have {}", blocks_needed, count);
                    return false;
                }
            } else {
                println!("❌ No {} blocks in inventory!", format!("{:?}", self.current_material));
                return false;
            }

            let mut operations = Vec::new();
            let mut blocks_placed = 0;

            // Place blocks in a 5x3 pattern (width x height)
            for x_offset in 0..5 {
                for y_offset in 0..3 {
                    let block_pos = (pos.0 + x_offset, pos.1 + y_offset, pos.2);

                    // Only place if the position is empty
                    if world.get_voxel(Vector3::new(block_pos.0 as f32, block_pos.1 as f32, block_pos.2 as f32)).unwrap_or(VoxelType::Air) == VoxelType::Air {
                        world.set_voxel(Vector3::new(block_pos.0 as f32, block_pos.1 as f32, block_pos.2 as f32), self.current_material);

                        operations.push(BuildOperation {
                            position: block_pos,
                            old_voxel: VoxelType::Air,
                            new_voxel: self.current_material,
                        });

                        blocks_placed += 1;
                    }
                }
            }

            if blocks_placed > 0 {
                // Deduct materials from inventory
                if let Some((_, count)) = self.inventory.iter_mut().find(|(mat, _)| *mat == self.current_material) {
                    *count = count.saturating_sub(blocks_placed);
                }

                let action = BuildAction {
                    operations,
                    description: format!("Build wall ({} blocks)", blocks_placed),
                };
                self.add_action(action);

                println!("🧱 Wall built with {} blocks", blocks_placed);
                true
            } else {
                println!("❌ No valid positions to place wall blocks");
                false
            }
        }

        fn build_floor(&mut self, world: &mut VoxelWorld, pos: (i32, i32, i32)) -> bool {
            // Build a 5x5 floor platform at the clicked Y level
            let blocks_needed = 5 * 5; // 25 blocks total

            // Check if we have enough materials
            if let Some((_, count)) = self.inventory.iter().find(|(mat, _)| *mat == self.current_material) {
                if *count < blocks_needed {
                    println!("❌ Not enough materials! Need {} blocks, have {}", blocks_needed, count);
                    return false;
                }
            } else {
                return false;
            }

            let mut operations = Vec::new();
            let mut blocks_placed = 0;

            // Place blocks in a 5x5 pattern (width x depth)
            for x_offset in -2..=2 {
                for z_offset in -2..=2 {
                    let block_pos = (pos.0 + x_offset, pos.1, pos.2 + z_offset);

                    // Only place if the position is empty
                    if world.get_voxel(Vector3::new(block_pos.0 as f32, block_pos.1 as f32, block_pos.2 as f32)).unwrap_or(VoxelType::Air) == VoxelType::Air {
                        world.set_voxel(Vector3::new(block_pos.0 as f32, block_pos.1 as f32, block_pos.2 as f32), self.current_material);

                        operations.push(BuildOperation {
                            position: block_pos,
                            old_voxel: VoxelType::Air,
                            new_voxel: self.current_material,
                        });

                        blocks_placed += 1;
                    }
                }
            }

            if blocks_placed > 0 {
                // Deduct materials from inventory
                if let Some((_, count)) = self.inventory.iter_mut().find(|(mat, _)| *mat == self.current_material) {
                    *count = count.saturating_sub(blocks_placed);
                }

                let action = BuildAction {
                    operations,
                    description: format!("Build floor ({} blocks)", blocks_placed),
                };
                self.add_action(action);

                println!("📦 Floor built with {} blocks", blocks_placed);
                true
            } else {
                false
            }
        }

        fn build_roof(&mut self, world: &mut VoxelWorld, pos: (i32, i32, i32)) -> bool {
            // Build a simple pyramid roof (3x3 base tapering to 1x1 top)
            let blocks_needed = 9 + 4 + 1; // 14 blocks total (3x3 + 2x2 + 1x1)

            // Check if we have enough materials
            if let Some((_, count)) = self.inventory.iter().find(|(mat, _)| *mat == self.current_material) {
                if *count < blocks_needed {
                    println!("❌ Not enough materials! Need {} blocks, have {}", blocks_needed, count);
                    return false;
                }
            } else {
                return false;
            }

            let mut operations = Vec::new();
            let mut blocks_placed = 0;

            // Build pyramid layers
            let layers = vec![
                (3, -1, 1), // 3x3 base, offset -1 from center, Y+1
                (2, 0, 2),  // 2x2 middle, no offset, Y+2
                (1, 0, 3),  // 1x1 top, no offset, Y+3
            ];

            for (size, offset, y_level) in layers {
                let half_size = size / 2;
                for x_offset in -half_size..=half_size {
                    for z_offset in -half_size..=half_size {
                        let block_pos = (pos.0 + x_offset + offset, pos.1 + y_level, pos.2 + z_offset + offset);

                        if world.get_voxel(Vector3::new(block_pos.0 as f32, block_pos.1 as f32, block_pos.2 as f32)).unwrap_or(VoxelType::Air) == VoxelType::Air {
                            world.set_voxel(Vector3::new(block_pos.0 as f32, block_pos.1 as f32, block_pos.2 as f32), self.current_material);

                            operations.push(BuildOperation {
                                position: block_pos,
                                old_voxel: VoxelType::Air,
                                new_voxel: self.current_material,
                            });

                            blocks_placed += 1;
                        }
                    }
                }
            }

            if blocks_placed > 0 {
                // Deduct materials from inventory
                if let Some((_, count)) = self.inventory.iter_mut().find(|(mat, _)| *mat == self.current_material) {
                    *count = count.saturating_sub(blocks_placed);
                }

                let action = BuildAction {
                    operations,
                    description: format!("Build roof ({} blocks)", blocks_placed),
                };
                self.add_action(action);

                println!("🏠 Roof built with {} blocks", blocks_placed);
                true
            } else {
                false
            }
        }

        fn build_template(&mut self, world: &mut VoxelWorld, pos: (i32, i32, i32)) -> bool {
            match self.current_template {
                TemplateType::Stairs => self.build_stairs(world, pos),
                TemplateType::Arch => self.build_arch(world, pos),
                TemplateType::Bridge => self.build_bridge(world, pos),
                TemplateType::Tower => self.build_tower(world, pos),
                TemplateType::House => self.build_house(world, pos),
                // Enhanced templates - basic implementations for now
                TemplateType::Castle => self.build_house(world, pos),     // Use house as base
                TemplateType::Garden => self.build_stairs(world, pos),   // Use stairs as base
                TemplateType::Workshop => self.build_house(world, pos),  // Use house as base
                TemplateType::Fortress => self.build_tower(world, pos),  // Use tower as base
                TemplateType::Lighthouse => self.build_tower(world, pos), // Use tower as base
                TemplateType::Windmill => self.build_tower(world, pos),  // Use tower as base
            }
        }

        fn build_stairs(&mut self, world: &mut VoxelWorld, pos: (i32, i32, i32)) -> bool {
            // Build a 4-step staircase going up in the X direction
            let blocks_needed = 4;

            if let Some((_, count)) = self.inventory.iter().find(|(mat, _)| *mat == self.current_material) {
                if *count < blocks_needed {
                    println!("❌ Not enough materials! Need {} blocks, have {}", blocks_needed, count);
                    return false;
                }
            } else {
                return false;
            }

            let mut operations = Vec::new();
            let mut blocks_placed = 0;

            // Place 4 steps, each one block higher than the last
            for step in 0..4 {
                let block_pos = (pos.0 + step, pos.1 + step, pos.2);

                if world.get_voxel(Vector3::new(block_pos.0 as f32, block_pos.1 as f32, block_pos.2 as f32)).unwrap_or(VoxelType::Air) == VoxelType::Air {
                    world.set_voxel(Vector3::new(block_pos.0 as f32, block_pos.1 as f32, block_pos.2 as f32), self.current_material);

                    operations.push(BuildOperation {
                        position: block_pos,
                        old_voxel: VoxelType::Air,
                        new_voxel: self.current_material,
                    });

                    blocks_placed += 1;
                }
            }

            if blocks_placed > 0 {
                if let Some((_, count)) = self.inventory.iter_mut().find(|(mat, _)| *mat == self.current_material) {
                    *count = count.saturating_sub(blocks_placed);
                }

                let action = BuildAction {
                    operations,
                    description: format!("Build stairs ({} blocks)", blocks_placed),
                };
                self.add_action(action);

                println!("🪜 Stairs built with {} blocks", blocks_placed);
                true
            } else {
                false
            }
        }

        fn build_arch(&mut self, world: &mut VoxelWorld, pos: (i32, i32, i32)) -> bool {
            // Build a 5-wide x 4-high arch doorway
            // Pattern:
            //   ###   (top)
            //   # #   (upper sides)
            //   # #   (lower sides)
            //   # #   (base)

            let blocks_needed = 14; // 3 + 2 + 2 + 2 + 2 + 3 = 14 blocks total

            // Check if we have enough materials
            if let Some((_, count)) = self.inventory.iter().find(|(mat, _)| *mat == self.current_material) {
                if *count < blocks_needed {
                    println!("❌ Not enough materials! Need {} blocks, have {}", blocks_needed, count);
                    return false;
                }
            } else {
                return false;
            }

            let mut operations = Vec::new();
            let mut blocks_placed = 0;

            // Define arch pattern: (x_offset, y_offset, place_block)
            let arch_pattern = [
                // Base level (Y+0) - pillars only
                (-2, 0, true), (-1, 0, false), (0, 0, false), (1, 0, false), (2, 0, true),
                // Lower sides (Y+1) - pillars only
                (-2, 1, true), (-1, 1, false), (0, 1, false), (1, 1, false), (2, 1, true),
                // Upper sides (Y+2) - pillars only
                (-2, 2, true), (-1, 2, false), (0, 2, false), (1, 2, false), (2, 2, true),
                // Top (Y+3) - full arch span
                (-2, 3, true), (-1, 3, true), (0, 3, true), (1, 3, true), (2, 3, true),
            ];

            // Place blocks according to pattern
            for (x_offset, y_offset, should_place) in arch_pattern.iter() {
                if *should_place {
                    let block_pos = (pos.0 + x_offset, pos.1 + y_offset, pos.2);

                    // Only place if the position is empty
                    if world.get_voxel(Vector3::new(block_pos.0 as f32, block_pos.1 as f32, block_pos.2 as f32)).unwrap_or(VoxelType::Air) == VoxelType::Air {
                        world.set_voxel(Vector3::new(block_pos.0 as f32, block_pos.1 as f32, block_pos.2 as f32), self.current_material);

                        operations.push(BuildOperation {
                            position: block_pos,
                            old_voxel: VoxelType::Air,
                            new_voxel: self.current_material,
                        });

                        blocks_placed += 1;
                    }
                }
            }

            if blocks_placed > 0 {
                // Deduct materials from inventory
                if let Some((_, count)) = self.inventory.iter_mut().find(|(mat, _)| *mat == self.current_material) {
                    *count = count.saturating_sub(blocks_placed);
                }

                let action = BuildAction {
                    operations,
                    description: format!("Build arch ({} blocks)", blocks_placed),
                };
                self.add_action(action);

                println!("🏗️ Arch built with {} blocks", blocks_placed);
                true
            } else {
                false
            }
        }

        fn build_bridge(&mut self, world: &mut VoxelWorld, pos: (i32, i32, i32)) -> bool {
            // Build a 7-long x 3-wide suspended walkway with railings
            // Total blocks: (7×3 walkway) + (7×2×2 railings) = 21 + 28 = 49 blocks
            let blocks_needed = 49;

            // Check if we have enough materials
            if let Some((_, count)) = self.inventory.iter().find(|(mat, _)| *mat == self.current_material) {
                if *count < blocks_needed {
                    println!("❌ Not enough materials! Need {} blocks, have {}", blocks_needed, count);
                    return false;
                }
            } else {
                println!("❌ No {} blocks in inventory!", format!("{:?}", self.current_material));
                return false;
            }

            let mut operations = Vec::new();
            let mut blocks_placed = 0;

            // Build walkway floor (7×3)
            for x_offset in 0..7 {
                for z_offset in -1..=1 {
                    let block_pos = (pos.0 + x_offset, pos.1, pos.2 + z_offset);

                    if world.get_voxel(Vector3::new(block_pos.0 as f32, block_pos.1 as f32, block_pos.2 as f32)).unwrap_or(VoxelType::Air) == VoxelType::Air {
                        world.set_voxel(Vector3::new(block_pos.0 as f32, block_pos.1 as f32, block_pos.2 as f32), self.current_material);

                        operations.push(BuildOperation {
                            position: block_pos,
                            old_voxel: VoxelType::Air,
                            new_voxel: self.current_material,
                        });

                        blocks_placed += 1;
                    }
                }
            }

            // Build railings on both sides (7×2×2)
            for x_offset in 0..7 {
                for railing_height in 1..=2 {
                    // Left railing
                    let left_pos = (pos.0 + x_offset, pos.1 + railing_height, pos.2 - 2);
                    if world.get_voxel(Vector3::new(left_pos.0 as f32, left_pos.1 as f32, left_pos.2 as f32)).unwrap_or(VoxelType::Air) == VoxelType::Air {
                        world.set_voxel(Vector3::new(left_pos.0 as f32, left_pos.1 as f32, left_pos.2 as f32), self.current_material);

                        operations.push(BuildOperation {
                            position: left_pos,
                            old_voxel: VoxelType::Air,
                            new_voxel: self.current_material,
                        });

                        blocks_placed += 1;
                    }

                    // Right railing
                    let right_pos = (pos.0 + x_offset, pos.1 + railing_height, pos.2 + 2);
                    if world.get_voxel(Vector3::new(right_pos.0 as f32, right_pos.1 as f32, right_pos.2 as f32)).unwrap_or(VoxelType::Air) == VoxelType::Air {
                        world.set_voxel(Vector3::new(right_pos.0 as f32, right_pos.1 as f32, right_pos.2 as f32), self.current_material);

                        operations.push(BuildOperation {
                            position: right_pos,
                            old_voxel: VoxelType::Air,
                            new_voxel: self.current_material,
                        });

                        blocks_placed += 1;
                    }
                }
            }

            if blocks_placed > 0 {
                // Deduct materials from inventory
                if let Some((_, count)) = self.inventory.iter_mut().find(|(mat, _)| *mat == self.current_material) {
                    *count = count.saturating_sub(blocks_placed);
                }

                let action = BuildAction {
                    operations,
                    description: format!("Build bridge ({} blocks)", blocks_placed),
                };
                self.add_action(action);

                println!("🌉 Bridge built with {} blocks", blocks_placed);
                true
            } else {
                println!("❌ No blocks placed (positions already occupied)");
                false
            }
        }

        fn build_tower(&mut self, world: &mut VoxelWorld, pos: (i32, i32, i32)) -> bool {
            // Build a 3×3×8 cylindrical tower with battlements
            // Main structure: 4 wall blocks × 8 levels = 32 blocks
            // Battlements: 4 additional blocks = 4 blocks
            // Total: 36 blocks
            let blocks_needed = 36;

            // Check if we have enough materials
            if let Some((_, count)) = self.inventory.iter().find(|(mat, _)| *mat == self.current_material) {
                if *count < blocks_needed {
                    println!("❌ Not enough materials! Need {} blocks, have {}", blocks_needed, count);
                    return false;
                }
            } else {
                println!("❌ No {} blocks in inventory!", format!("{:?}", self.current_material));
                return false;
            }

            let mut operations = Vec::new();
            let mut blocks_placed = 0;

            // Tower walls (cross pattern for cylindrical effect)
            let wall_positions = [
                (0, 1), (1, 0), (1, 2), (2, 1), // Cross pattern avoiding corners
            ];

            // Build main tower structure (8 blocks high)
            for height in 0..8 {
                for (x_offset, z_offset) in wall_positions.iter() {
                    let block_pos = (pos.0 + x_offset - 1, pos.1 + height, pos.2 + z_offset - 1);

                    if world.get_voxel(Vector3::new(block_pos.0 as f32, block_pos.1 as f32, block_pos.2 as f32)).unwrap_or(VoxelType::Air) == VoxelType::Air {
                        world.set_voxel(Vector3::new(block_pos.0 as f32, block_pos.1 as f32, block_pos.2 as f32), self.current_material);

                        operations.push(BuildOperation {
                            position: block_pos,
                            old_voxel: VoxelType::Air,
                            new_voxel: self.current_material,
                        });

                        blocks_placed += 1;
                    }
                }
            }

            // Build battlements (crenellated top at height 8)
            let battlement_positions = [
                (0, 1), (1, 2), (2, 1), (1, 0), // Alternating pattern for medieval look
            ];
            for (x_offset, z_offset) in battlement_positions.iter() {
                let block_pos = (pos.0 + x_offset - 1, pos.1 + 8, pos.2 + z_offset - 1);

                if world.get_voxel(Vector3::new(block_pos.0 as f32, block_pos.1 as f32, block_pos.2 as f32)).unwrap_or(VoxelType::Air) == VoxelType::Air {
                    world.set_voxel(Vector3::new(block_pos.0 as f32, block_pos.1 as f32, block_pos.2 as f32), self.current_material);

                    operations.push(BuildOperation {
                        position: block_pos,
                        old_voxel: VoxelType::Air,
                        new_voxel: self.current_material,
                    });

                    blocks_placed += 1;
                }
            }

            if blocks_placed > 0 {
                // Deduct materials from inventory
                if let Some((_, count)) = self.inventory.iter_mut().find(|(mat, _)| *mat == self.current_material) {
                    *count = count.saturating_sub(blocks_placed);
                }

                let action = BuildAction {
                    operations,
                    description: format!("Build tower ({} blocks)", blocks_placed),
                };
                self.add_action(action);

                println!("🗼 Tower built with {} blocks", blocks_placed);
                true
            } else {
                println!("❌ No blocks placed (positions already occupied)");
                false
            }
        }

        fn build_house(&mut self, world: &mut VoxelWorld, pos: (i32, i32, i32)) -> bool {
            // Build a 5×5×4 house with door and windows
            // Foundation: 25 blocks, Walls: ~45 blocks (with openings), Roof: 25 blocks
            // Total estimate: ~95 blocks
            let blocks_needed = 95;

            // Check if we have enough materials
            if let Some((_, count)) = self.inventory.iter().find(|(mat, _)| *mat == self.current_material) {
                if *count < blocks_needed {
                    println!("❌ Not enough materials! Need {} blocks, have {}", blocks_needed, count);
                    return false;
                }
            } else {
                println!("❌ No {} blocks in inventory!", format!("{:?}", self.current_material));
                return false;
            }

            let mut operations = Vec::new();
            let mut blocks_placed = 0;

            // Build foundation (5×5 floor)
            for x_offset in 0..5 {
                for z_offset in 0..5 {
                    let block_pos = (pos.0 + x_offset - 2, pos.1, pos.2 + z_offset - 2);

                    if world.get_voxel(Vector3::new(block_pos.0 as f32, block_pos.1 as f32, block_pos.2 as f32)).unwrap_or(VoxelType::Air) == VoxelType::Air {
                        world.set_voxel(Vector3::new(block_pos.0 as f32, block_pos.1 as f32, block_pos.2 as f32), self.current_material);

                        operations.push(BuildOperation {
                            position: block_pos,
                            old_voxel: VoxelType::Air,
                            new_voxel: self.current_material,
                        });

                        blocks_placed += 1;
                    }
                }
            }

            // Build walls (levels 1-3) with door and window openings
            for height in 1..=3 {
                for x_offset in 0..5 {
                    for z_offset in 0..5 {
                        // Only place blocks on the perimeter (walls)
                        if x_offset == 0 || x_offset == 4 || z_offset == 0 || z_offset == 4 {
                            let block_pos = (pos.0 + x_offset - 2, pos.1 + height, pos.2 + z_offset - 2);

                            // Skip door opening (front wall, center, levels 1-2)
                            if z_offset == 0 && x_offset == 2 && height <= 2 {
                                continue;
                            }

                            // Skip window openings (side walls, center, level 2)
                            if height == 2 && ((x_offset == 0 && z_offset == 2) || (x_offset == 4 && z_offset == 2)) {
                                continue;
                            }

                            if world.get_voxel(Vector3::new(block_pos.0 as f32, block_pos.1 as f32, block_pos.2 as f32)).unwrap_or(VoxelType::Air) == VoxelType::Air {
                                world.set_voxel(Vector3::new(block_pos.0 as f32, block_pos.1 as f32, block_pos.2 as f32), self.current_material);

                                operations.push(BuildOperation {
                                    position: block_pos,
                                    old_voxel: VoxelType::Air,
                                    new_voxel: self.current_material,
                                });

                                blocks_placed += 1;
                            }
                        }
                    }
                }
            }

            // Build roof (5×5 at level 4)
            for x_offset in 0..5 {
                for z_offset in 0..5 {
                    let block_pos = (pos.0 + x_offset - 2, pos.1 + 4, pos.2 + z_offset - 2);

                    if world.get_voxel(Vector3::new(block_pos.0 as f32, block_pos.1 as f32, block_pos.2 as f32)).unwrap_or(VoxelType::Air) == VoxelType::Air {
                        world.set_voxel(Vector3::new(block_pos.0 as f32, block_pos.1 as f32, block_pos.2 as f32), self.current_material);

                        operations.push(BuildOperation {
                            position: block_pos,
                            old_voxel: VoxelType::Air,
                            new_voxel: self.current_material,
                        });

                        blocks_placed += 1;
                    }
                }
            }

            if blocks_placed > 0 {
                // Deduct materials from inventory
                if let Some((_, count)) = self.inventory.iter_mut().find(|(mat, _)| *mat == self.current_material) {
                    *count = count.saturating_sub(blocks_placed);
                }

                let action = BuildAction {
                    operations,
                    description: format!("Build house ({} blocks)", blocks_placed),
                };
                self.add_action(action);

                println!("🏠 House built with {} blocks", blocks_placed);
                true
            } else {
                println!("❌ No blocks placed (positions already occupied)");
                false
            }
        }

        fn add_action(&mut self, action: BuildAction) {
            self.redo_stack.clear();
            self.undo_stack.push_back(action);

            if self.undo_stack.len() > 50 {
                self.undo_stack.pop_front();
            }
        }

        pub fn undo(&mut self, world: &mut VoxelWorld) -> bool {
            if let Some(action) = self.undo_stack.pop_back() {
                let mut redo_action = BuildAction {
                    operations: Vec::new(),
                    description: format!("Redo {}", action.description),
                };

                for op in action.operations.iter().rev() {
                    redo_action.operations.push(BuildOperation {
                        position: op.position,
                        old_voxel: op.new_voxel,
                        new_voxel: op.old_voxel,
                    });
                    world.set_voxel(Vector3::new(op.position.0 as f32, op.position.1 as f32, op.position.2 as f32), op.old_voxel);
                }

                self.redo_stack.push_back(redo_action);
                println!("🔄 Undid: {}", action.description);
                true
            } else {
                false
            }
        }

        pub fn redo(&mut self, world: &mut VoxelWorld) -> bool {
            if let Some(action) = self.redo_stack.pop_back() {
                let mut undo_action = BuildAction {
                    operations: Vec::new(),
                    description: format!("Undo {}", action.description),
                };

                for op in action.operations.iter().rev() {
                    undo_action.operations.push(BuildOperation {
                        position: op.position,
                        old_voxel: op.new_voxel,
                        new_voxel: op.old_voxel,
                    });
                    world.set_voxel(Vector3::new(op.position.0 as f32, op.position.1 as f32, op.position.2 as f32), op.old_voxel);
                }

                self.undo_stack.push_back(undo_action);
                println!("🔄 Redid: {}", action.description);
                true
            } else {
                false
            }
        }

        // Setters for UI
        pub fn select_material(&mut self, material: VoxelType) {
            self.current_material = material;
        }

        pub fn set_build_mode(&mut self, mode: BuildMode) {
            self.mode = mode;
        }

        // Missing methods needed by UI
        pub fn cycle_mode(&mut self) {
            self.cycle_build_mode();
        }

        pub fn set_material(&mut self, material: VoxelType) {
            self.current_material = material;
        }

        // Getters for UI
        pub fn get_current_mode(&self) -> BuildMode { self.mode }
        pub fn get_current_material(&self) -> VoxelType { self.current_material }
        pub fn is_grid_snap_enabled(&self) -> bool { self.grid_snap }
        pub fn get_inventory(&self) -> &Vec<(VoxelType, u32)> { &self.inventory }
    }
}

// Re-export types from game module for external use
use game::VoxelBuildSystem;

fn generate_world_mesh(world: &VoxelWorld, mesh: &mut Mesh, camera: &Camera) {
    let mut total_voxels = 0;
    let mut solid_voxels = 0;
    let mut visible_chunks = 0;
    let mut culled_chunks = 0;

    // Create texture atlas instance for UV coordinate mapping
    let texture_atlas = TextureAtlas::new();

    // Extract camera frustum for culling
    let frustum = camera.get_frustum();

    // Generate mesh for visible chunks only
    for ((cx, cy, cz), chunk) in &world.chunks {
        // Calculate chunk bounding box
        let chunk_aabb = renderer::AABB::from_chunk_coords(*cx, *cy, *cz, world.chunk_size);

        // Test if chunk is visible in camera frustum
        if !frustum.intersects_aabb(&chunk_aabb) {
            culled_chunks += 1;
            continue; // Skip this chunk - it's not visible
        }

        visible_chunks += 1;
        println!("Processing visible chunk ({}, {}, {})", cx, cy, cz);
        // Use greedy meshing for this chunk instead of naive face-by-face
        greedy_mesh_chunk(world, mesh, (*cx, *cy, *cz), chunk, &texture_atlas);

        // Count voxels for statistics
        for x in 0..chunk.grid.size.0 {
            for y in 0..chunk.grid.size.0 {
                for z in 0..chunk.grid.size.0 {
                    total_voxels += 1;
                    let voxel = chunk.grid.get_voxel_type(x, y, z).unwrap_or(VoxelType::Air);
                    if voxel != VoxelType::Air {
                        solid_voxels += 1;
                    }
                }
            }
        }
    }

    println!("Frustum culling: {} visible chunks, {} culled chunks ({:.1}% culled)",
             visible_chunks, culled_chunks,
             (culled_chunks as f32 / (visible_chunks + culled_chunks) as f32) * 100.0);
    println!("Processed {} total voxels, {} solid voxels", total_voxels, solid_voxels);
    println!("Generated {} vertices from {} faces", mesh.vertices.len(), mesh.vertices.len() / 4);
}

fn greedy_mesh_chunk(
    world: &VoxelWorld,
    mesh: &mut Mesh,
    chunk_pos: (i32, i32, i32),
    chunk: &robin::engine::generation::voxel_system::VoxelChunk,
    texture_atlas: &TextureAtlas,
) {
    let (chunk_x, chunk_y, chunk_z) = chunk_pos;
    let chunk_size = chunk.grid.size.0;

    // Process each face direction separately for greedy meshing
    // This is the core of the algorithm: instead of processing voxel-by-voxel,
    // we process face-by-face to find opportunities for merging

    // +X faces (right side of voxels)
    greedy_mesh_direction(world, mesh, chunk_pos, chunk, texture_atlas,
                         FaceDirection::PosX);

    // -X faces (left side of voxels)
    greedy_mesh_direction(world, mesh, chunk_pos, chunk, texture_atlas,
                         FaceDirection::NegX);

    // +Y faces (top side of voxels)
    greedy_mesh_direction(world, mesh, chunk_pos, chunk, texture_atlas,
                         FaceDirection::PosY);

    // -Y faces (bottom side of voxels)
    greedy_mesh_direction(world, mesh, chunk_pos, chunk, texture_atlas,
                         FaceDirection::NegY);

    // +Z faces (front side of voxels)
    greedy_mesh_direction(world, mesh, chunk_pos, chunk, texture_atlas,
                         FaceDirection::PosZ);

    // -Z faces (back side of voxels)
    greedy_mesh_direction(world, mesh, chunk_pos, chunk, texture_atlas,
                         FaceDirection::NegZ);
}

#[derive(Debug, Clone, Copy)]
enum FaceDirection {
    PosX, NegX, PosY, NegY, PosZ, NegZ,
}

fn greedy_mesh_direction(
    world: &VoxelWorld,
    mesh: &mut Mesh,
    chunk_pos: (i32, i32, i32),
    chunk: &robin::engine::generation::voxel_system::VoxelChunk,
    texture_atlas: &TextureAtlas,
    direction: FaceDirection,
) {
    let (chunk_x, chunk_y, chunk_z) = chunk_pos;
    let chunk_size = chunk.grid.size.0;

    // Create a 2D mask for this face direction
    // The mask tells us which positions need a face in this direction
    let mut face_mask: Vec<Vec<Option<VoxelType>>> = vec![vec![None; chunk_size]; chunk_size];
    let mut processed: Vec<Vec<bool>> = vec![vec![false; chunk_size]; chunk_size];

    // Fill the face mask by checking each position in the 2D plane
    fill_face_mask(world, chunk_pos, chunk, &mut face_mask, direction);

    // Now use greedy algorithm to find rectangular regions and merge them
    for u in 0..chunk_size {
        for v in 0..chunk_size {
            if processed[u][v] || face_mask[u][v].is_none() {
                continue; // Skip if already processed or no face needed
            }

            let material = face_mask[u][v].unwrap();

            // Find the largest rectangle starting at (u, v) with this material
            let (width, height) = find_largest_rectangle(&face_mask, &mut processed, u, v, material);

            // Generate one large quad for this merged rectangle
            generate_merged_quad(mesh, chunk_pos, chunk, direction, u, v, width, height, material, texture_atlas);
        }
    }
}

fn fill_face_mask(
    world: &VoxelWorld,
    chunk_pos: (i32, i32, i32),
    chunk: &robin::engine::generation::voxel_system::VoxelChunk,
    face_mask: &mut Vec<Vec<Option<VoxelType>>>,
    direction: FaceDirection,
) {
    let (chunk_x, chunk_y, chunk_z) = chunk_pos;
    let chunk_size = chunk.grid.size.0;

    // For each direction, we scan through the chunk in a different order
    // and check if a face is needed at each position
    match direction {
        FaceDirection::PosX => {
            // For +X faces, scan through YZ plane, check if voxel needs +X face
            for y in 0..chunk_size {
                for z in 0..chunk_size {
                    for x in 0..chunk_size {
                        let voxel = chunk.grid.get_voxel_type(x, y, z).unwrap_or(VoxelType::Air);
                        if voxel != VoxelType::Air {
                            // Check if adjacent voxel in +X direction is air
                            let world_x = chunk_x * chunk_size as i32 + x as i32;
                            let world_y = chunk_y * chunk_size as i32 + y as i32;
                            let world_z = chunk_z * chunk_size as i32 + z as i32;

                            let adjacent = world.get_voxel(Vector3::new((world_x + 1) as f32, world_y as f32, world_z as f32)).unwrap_or(VoxelType::Air);
                            if adjacent == VoxelType::Air {
                                face_mask[y][z] = Some(voxel);
                                break; // Only need one face per YZ position
                            }
                        }
                    }
                }
            }
        },
        FaceDirection::NegX => {
            for y in 0..chunk_size {
                for z in 0..chunk_size {
                    for x in (0..chunk_size).rev() {
                        let voxel = chunk.grid.get_voxel_type(x, y, z).unwrap_or(VoxelType::Air);
                        if voxel != VoxelType::Air {
                            let world_x = chunk_x * chunk_size as i32 + x as i32;
                            let world_y = chunk_y * chunk_size as i32 + y as i32;
                            let world_z = chunk_z * chunk_size as i32 + z as i32;

                            let adjacent = world.get_voxel(Vector3::new((world_x - 1) as f32, world_y as f32, world_z as f32)).unwrap_or(VoxelType::Air);
                            if adjacent == VoxelType::Air {
                                face_mask[y][z] = Some(voxel);
                                break;
                            }
                        }
                    }
                }
            }
        },
        FaceDirection::PosY => {
            for x in 0..chunk_size {
                for z in 0..chunk_size {
                    for y in 0..chunk_size {
                        let voxel = chunk.grid.get_voxel_type(x, y, z).unwrap_or(VoxelType::Air);
                        if voxel != VoxelType::Air {
                            let world_x = chunk_x * chunk_size as i32 + x as i32;
                            let world_y = chunk_y * chunk_size as i32 + y as i32;
                            let world_z = chunk_z * chunk_size as i32 + z as i32;

                            let adjacent = world.get_voxel(Vector3::new(world_x as f32, (world_y + 1) as f32, world_z as f32)).unwrap_or(VoxelType::Air);
                            if adjacent == VoxelType::Air {
                                face_mask[x][z] = Some(voxel);
                                break;
                            }
                        }
                    }
                }
            }
        },
        FaceDirection::NegY => {
            for x in 0..chunk_size {
                for z in 0..chunk_size {
                    for y in (0..chunk_size).rev() {
                        let voxel = chunk.grid.get_voxel_type(x, y, z).unwrap_or(VoxelType::Air);
                        if voxel != VoxelType::Air {
                            let world_x = chunk_x * chunk_size as i32 + x as i32;
                            let world_y = chunk_y * chunk_size as i32 + y as i32;
                            let world_z = chunk_z * chunk_size as i32 + z as i32;

                            let adjacent = world.get_voxel(Vector3::new(world_x as f32, (world_y - 1) as f32, world_z as f32)).unwrap_or(VoxelType::Air);
                            if adjacent == VoxelType::Air {
                                face_mask[x][z] = Some(voxel);
                                break;
                            }
                        }
                    }
                }
            }
        },
        FaceDirection::PosZ => {
            for x in 0..chunk_size {
                for y in 0..chunk_size {
                    for z in 0..chunk_size {
                        let voxel = chunk.grid.get_voxel_type(x, y, z).unwrap_or(VoxelType::Air);
                        if voxel != VoxelType::Air {
                            let world_x = chunk_x * chunk_size as i32 + x as i32;
                            let world_y = chunk_y * chunk_size as i32 + y as i32;
                            let world_z = chunk_z * chunk_size as i32 + z as i32;

                            let adjacent = world.get_voxel(Vector3::new(world_x as f32, world_y as f32, (world_z + 1) as f32)).unwrap_or(VoxelType::Air);
                            if adjacent == VoxelType::Air {
                                face_mask[x][y] = Some(voxel);
                                break;
                            }
                        }
                    }
                }
            }
        },
        FaceDirection::NegZ => {
            for x in 0..chunk_size {
                for y in 0..chunk_size {
                    for z in (0..chunk_size).rev() {
                        let voxel = chunk.grid.get_voxel_type(x, y, z).unwrap_or(VoxelType::Air);
                        if voxel != VoxelType::Air {
                            let world_x = chunk_x * chunk_size as i32 + x as i32;
                            let world_y = chunk_y * chunk_size as i32 + y as i32;
                            let world_z = chunk_z * chunk_size as i32 + z as i32;

                            let adjacent = world.get_voxel(Vector3::new(world_x as f32, world_y as f32, (world_z - 1) as f32)).unwrap_or(VoxelType::Air);
                            if adjacent == VoxelType::Air {
                                face_mask[x][y] = Some(voxel);
                                break;
                            }
                        }
                    }
                }
            }
        },
    }
}

fn find_largest_rectangle(
    face_mask: &Vec<Vec<Option<VoxelType>>>,
    processed: &mut Vec<Vec<bool>>,
    start_u: usize,
    start_v: usize,
    material: VoxelType,
) -> (usize, usize) {
    let mask_size = face_mask.len();

    // Find maximum width (expand horizontally first)
    let mut width = 0;
    for u in start_u..mask_size {
        if processed[u][start_v] ||
           face_mask[u][start_v] != Some(material) {
            break;
        }
        width += 1;
    }

    // Find maximum height (expand vertically, maintaining the width)
    let mut height = 0;
    'height_loop: for v in start_v..mask_size {
        // Check if entire row can be added
        for u in start_u..start_u + width {
            if processed[u][v] ||
               face_mask[u][v] != Some(material) {
                break 'height_loop;
            }
        }
        height += 1;
    }

    // Mark the entire rectangle as processed
    for u in start_u..start_u + width {
        for v in start_v..start_v + height {
            processed[u][v] = true;
        }
    }

    (width, height)
}

fn generate_merged_quad(
    mesh: &mut Mesh,
    chunk_pos: (i32, i32, i32),
    chunk: &robin::engine::generation::voxel_system::VoxelChunk,
    direction: FaceDirection,
    start_u: usize,
    start_v: usize,
    width: usize,
    height: usize,
    material: VoxelType,
    texture_atlas: &TextureAtlas,
) {
    let (chunk_x, chunk_y, chunk_z) = chunk_pos;
    let chunk_size = chunk.grid.size.0;

    let color = get_voxel_color(&material);
    let tile_uv = texture_atlas.get_uv(material);

    // Calculate world coordinates for the merged quad
    // This is the complex part - converting from 2D mask coordinates back to 3D world coordinates
    let (positions, normal) = match direction {
        FaceDirection::PosX => {
            // +X face: YZ plane projected, u=Y, v=Z
            let world_x = (chunk_x * chunk_size as i32 + chunk_size as i32) as f32;
            let start_y = (chunk_y * chunk_size as i32 + start_u as i32) as f32;
            let start_z = (chunk_z * chunk_size as i32 + start_v as i32) as f32;
            let end_y = start_y + width as f32;
            let end_z = start_z + height as f32;

            ([
                [world_x, start_y, end_z],
                [world_x, end_y, end_z],
                [world_x, end_y, start_z],
                [world_x, start_y, start_z],
            ], [1.0, 0.0, 0.0])
        },
        FaceDirection::NegX => {
            let world_x = (chunk_x * chunk_size as i32) as f32;
            let start_y = (chunk_y * chunk_size as i32 + start_u as i32) as f32;
            let start_z = (chunk_z * chunk_size as i32 + start_v as i32) as f32;
            let end_y = start_y + width as f32;
            let end_z = start_z + height as f32;

            ([
                [world_x, start_y, start_z],
                [world_x, end_y, start_z],
                [world_x, end_y, end_z],
                [world_x, start_y, end_z],
            ], [-1.0, 0.0, 0.0])
        },
        FaceDirection::PosY => {
            // +Y face: XZ plane projected, u=X, v=Z
            let world_y = (chunk_y * chunk_size as i32 + chunk_size as i32) as f32;
            let start_x = (chunk_x * chunk_size as i32 + start_u as i32) as f32;
            let start_z = (chunk_z * chunk_size as i32 + start_v as i32) as f32;
            let end_x = start_x + width as f32;
            let end_z = start_z + height as f32;

            ([
                [start_x, world_y, end_z],
                [end_x, world_y, end_z],
                [end_x, world_y, start_z],
                [start_x, world_y, start_z],
            ], [0.0, 1.0, 0.0])
        },
        FaceDirection::NegY => {
            let world_y = (chunk_y * chunk_size as i32) as f32;
            let start_x = (chunk_x * chunk_size as i32 + start_u as i32) as f32;
            let start_z = (chunk_z * chunk_size as i32 + start_v as i32) as f32;
            let end_x = start_x + width as f32;
            let end_z = start_z + height as f32;

            ([
                [start_x, world_y, start_z],
                [end_x, world_y, start_z],
                [end_x, world_y, end_z],
                [start_x, world_y, end_z],
            ], [0.0, -1.0, 0.0])
        },
        FaceDirection::PosZ => {
            // +Z face: XY plane projected, u=X, v=Y
            let world_z = (chunk_z * chunk_size as i32 + chunk_size as i32) as f32;
            let start_x = (chunk_x * chunk_size as i32 + start_u as i32) as f32;
            let start_y = (chunk_y * chunk_size as i32 + start_v as i32) as f32;
            let end_x = start_x + width as f32;
            let end_y = start_y + height as f32;

            ([
                [start_x, start_y, world_z],
                [end_x, start_y, world_z],
                [end_x, end_y, world_z],
                [start_x, end_y, world_z],
            ], [0.0, 0.0, 1.0])
        },
        FaceDirection::NegZ => {
            let world_z = (chunk_z * chunk_size as i32) as f32;
            let start_x = (chunk_x * chunk_size as i32 + start_u as i32) as f32;
            let start_y = (chunk_y * chunk_size as i32 + start_v as i32) as f32;
            let end_x = start_x + width as f32;
            let end_y = start_y + height as f32;

            ([
                [end_x, start_y, world_z],
                [start_x, start_y, world_z],
                [start_x, end_y, world_z],
                [end_x, end_y, world_z],
            ], [0.0, 0.0, -1.0])
        },
    };

    // Calculate UV coordinates for the merged quad
    // Scale the texture coordinates based on the size of the merged quad
    let u_scale = width as f32;
    let v_scale = height as f32;

    // Extract UV bounds from TileUV coords array
    // coords[0] = bottom-left, coords[2] = top-right
    let u_min = tile_uv.coords[0][0];
    let v_min = tile_uv.coords[0][1];
    let u_max = tile_uv.coords[2][0];
    let v_max = tile_uv.coords[2][1];

    let uv_coords = [
        [u_min, v_min],
        [u_min + (u_max - u_min) * u_scale, v_min],
        [u_min + (u_max - u_min) * u_scale, v_min + (v_max - v_min) * v_scale],
        [u_min, v_min + (v_max - v_min) * v_scale],
    ];

    // Add the optimized quad to the mesh
    mesh.add_quad_with_uv(positions, color, normal, uv_coords);
}


fn handle_ui_action(
    action: UIAction,
    build_system: &mut VoxelBuildSystem,
    world: &mut VoxelWorld,
    world_mesh: &mut Mesh,
    device: &metal::Device,
    time_system: &mut TimeOfDaySystem,
    camera: &Camera,
) {
    match action {
        UIAction::SelectMaterial(material) => {
            build_system.set_material(material);
            print_build_status(build_system);
        }
        UIAction::ToggleBuildMode => {
            build_system.cycle_mode();
            println!("🔨 Build mode changed to: {:?}", build_system.get_mode());
        }
        UIAction::Undo => {
            if build_system.undo(world) {
                regenerate_world_mesh(world, world_mesh, device, camera);
            }
        }
        UIAction::Redo => {
            if build_system.redo(world) {
                regenerate_world_mesh(world, world_mesh, device, camera);
            }
        }
        UIAction::SetTimeSpeed(speed) => {
            time_system.set_speed(speed);
            println!("⏰ Time speed set to: {:.1}x", speed);
        }
        UIAction::SetTimeOfDay(hours) => {
            time_system.set_time(hours);
            println!("🕐 Time set to: {}", time_system.get_time_string());
        }
        UIAction::ToggleTimePause => {
            time_system.toggle_pause();
            let status = if time_system.is_paused() { "paused" } else { "resumed" };
            println!("⏸️ Time {}", status);
        }
    }
}

fn handle_production_ui_action(
    action: robin::engine::ui::UIAction,
    demo_state: &mut demo_state::DemoStateManager,
    world: &mut VoxelWorld,
    build_system: &mut VoxelBuildSystem,
    time_system: &TimeOfDaySystem,
) {
    // Convert robin::engine::ui::UIAction to local UIAction if needed

    match action {
        robin::engine::ui::UIAction::StartGame => {
            println!("🎮 Starting game from production UI");
            demo_state.switch_mode(demo_state::DemoMode::InteractivePlayground);
        }
        robin::engine::ui::UIAction::StartCreativeMode => {
            println!("🎨 Starting creative mode from production UI");
            demo_state.switch_mode(demo_state::DemoMode::EngineerBuildShowcase);
        }
        robin::engine::ui::UIAction::QuitGame => {
            println!("👋 Quit game requested from production UI");
            // Note: Actual quit handled by main window close
        }
        robin::engine::ui::UIAction::ToggleBuildMode => {
            build_system.cycle_mode();
            println!("🔨 Production UI: Build mode changed to: {:?}", build_system.get_mode());
        }
        robin::engine::ui::UIAction::SelectMaterial(material) => {
            let converted_material = convert_world_voxel_to_generation(material);
            build_system.set_material(converted_material);
            println!("🧱 Production UI: Material selected: {:?}", material);
        }
        robin::engine::ui::UIAction::SettingsAction(settings_action) => {
            println!("⚙️ Production UI: Settings action: {:?}", settings_action);
            // Settings actions handled by the settings menu system
        }
        robin::engine::ui::UIAction::SaveGame => {
            println!("💾 Production UI: Save game requested");
            // TODO: Implement save system integration
        }
        robin::engine::ui::UIAction::LoadGame => {
            println!("📁 Production UI: Load game requested");
            // TODO: Implement load system integration
        }
        robin::engine::ui::UIAction::PauseGame => {
            println!("⏸️ Production UI: Game pause requested");
            // TODO: Implement game state pause
        }
        robin::engine::ui::UIAction::ResumeGame => {
            println!("▶️ Production UI: Game resume requested");
            // TODO: Implement game state resume
        }
    }
}

fn handle_unified_ui_action(
    action: crate::ui::unified_hud::UnifiedUIAction,
    build_system: &mut VoxelBuildSystem,
    world: &mut VoxelWorld,
    world_mesh: &mut Mesh,
    device: &metal::Device,
    time_system: &mut TimeOfDaySystem,
    camera: &Camera,
    demo_state: &mut demo_state::DemoStateManager,
) {
    use crate::ui::unified_hud::UnifiedUIAction;
    use crate::ui::UIAction;

    match action {
        UnifiedUIAction::Demo(demo_action) => {
            // Handle demo UI actions through the existing handler
            handle_ui_action(demo_action, build_system, world, world_mesh, device, time_system, camera);
        }
        UnifiedUIAction::HUD(hud_action) => {
            // Handle production HUD actions
            println!("🎯 Unified HUD Action: {:?}", hud_action);
            // Convert HUD actions to appropriate game actions
            match hud_action {
                robin::engine::ui::HUDAction::ToggleBuildMode => {
                    build_system.cycle_mode();
                    println!("🔨 Unified HUD: Build mode changed to: {:?}", build_system.get_mode());
                }
                robin::engine::ui::HUDAction::SelectMaterial(material) => {
                    let converted_material = convert_world_voxel_to_generation(material);
                    build_system.set_material(converted_material);
                    println!("🧱 Unified HUD: Material selected: {:?}", material);
                }
                robin::engine::ui::HUDAction::UpdateVoxelCount(count) => {
                    println!("📊 Updating voxel count: {}", count);
                }
                _ => {
                    println!("🔧 Unhandled HUD action: {:?}", hud_action);
                }
            }
        }
        UnifiedUIAction::Menu(menu_action) => {
            // Handle menu actions
            println!("📋 Unified Menu Action: {:?}", menu_action);
            match menu_action {
                robin::engine::ui::MenuAction::StartGame => {
                    demo_state.switch_mode(demo_state::DemoMode::InteractivePlayground);
                }
                robin::engine::ui::MenuAction::StartCreativeMode => {
                    demo_state.switch_mode(demo_state::DemoMode::EngineerBuildShowcase);
                }
                robin::engine::ui::MenuAction::QuitGame => {
                    println!("👋 Quit game requested from unified menu");
                    // Note: Actual quit handled by main window close
                }
                _ => {
                    println!("🔧 Unhandled menu action: {:?}", menu_action);
                }
            }
        }
        UnifiedUIAction::Settings(settings_action) => {
            println!("⚙️ Unified Settings Action: {:?}", settings_action);
            // Settings actions handled by the settings menu system
        }
        UnifiedUIAction::ToggleUI => {
            println!("🎛️ UI debug toggled");
            // Note: Actual toggle handled by the unified HUD system itself
        }
        UnifiedUIAction::ToggleProductionMode => {
            println!("📊 Performance metrics toggled");
            // Note: Actual toggle handled by the unified HUD system itself
        }
        UnifiedUIAction::SwitchUISystem => {
            println!("🔄 Switching UI system");
            // Note: Actual switch handled by the unified HUD system itself
        }
    }
}