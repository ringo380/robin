/// Robin Voxel Engine - macOS Native Metal Demo
///
/// A native macOS application demonstrating the voxel engine with:
/// - Metal rendering optimized for Apple Silicon
/// - Native Cocoa windowing
/// - First-person camera controls (WASD + Mouse)
/// - Real-time voxel placement/destruction
/// - Engineer Build Mode with crafting systems
/// - ImGui-based UI for inventory and building tools

mod renderer;
mod window;
mod ui;

use renderer::{MetalRenderer, Camera, Mesh, TextureAtlas};
use window::{NativeWindow, WindowEvent, MouseButton, key_codes};
use game::{VoxelWorld, EngineerBuildSystem, VoxelType, BuildMode, TemplateType};
use ui::simple_ui::{SimpleUISystem, UIAction};

use cgmath::Vector3;
use std::time::Instant;
use std::collections::HashSet;

fn main() {
    env_logger::init();

    match run() {
        Ok(_) => println!("✅ Robin Engine exited successfully"),
        Err(e) => eprintln!("❌ Error: {}", e),
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Robin Voxel Engine - macOS Native Demo");
    println!("🍎 Optimized for Apple Silicon with Metal rendering");

    // Create native macOS window
    let mut window = NativeWindow::new("Robin Engine - Engineer Build Mode", 1200.0, 800.0)?;
    println!("✅ Native macOS window created");

    // Create Metal renderer
    let mut renderer = MetalRenderer::new(&window)?;
    println!("✅ Metal renderer initialized");

    // Create camera
    let window_size = window.get_size();
    let mut camera = Camera::new(window_size.width as f32, window_size.height as f32);

    // Create voxel world
    println!("🌍 Generating voxel world...");
    let mut world = VoxelWorld::new();
    let mut world_mesh = Mesh::new();

    // Generate initial world mesh
    generate_world_mesh(&world, &mut world_mesh, &camera);
    world_mesh.create_buffers(renderer.get_device());
    println!("Generated {} vertices, {} indices", world_mesh.vertex_count, world_mesh.index_count);

    // Create preview mesh for ghost blocks
    let mut preview_mesh = Mesh::new();
    preview_mesh.create_buffers(renderer.get_device());

    // Create Engineer Build System
    let mut build_system = EngineerBuildSystem::new();
    println!("🔧 Engineer Build Mode initialized");

    // Create UI System
    let mut ui_system = SimpleUISystem::new();
    println!("🎨 UI System initialized");

    // Create Time-of-Day System
    let mut time_system = TimeOfDaySystem::new();
    println!("🌅 Time-of-Day System initialized");

    // Upload font texture to Metal and link to ImGui
    if let Some(texture_data) = ui_system.get_font_texture_data() {
        let (width, height) = ui_system.get_font_texture_dimensions();
        match renderer.create_font_texture(texture_data, width, height) {
            Ok(texture_id) => {
                ui_system.set_font_texture_id(imgui::TextureId::from(texture_id as usize));
                println!("🔤 Font texture linked to ImGui context");
            }
            Err(e) => eprintln!("❌ Failed to create font texture: {}", e),
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

    // Game state
    let mut keys_pressed = HashSet::new();
    let mut _mouse_grabbed = false;
    let start_time = Instant::now();
    let mut last_frame = Instant::now();
    let mut _ui_visible = true;

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

        // Handle window events
        let events = window.poll_events();
        for event in events {
            handle_event(&event, &mut camera, &mut world, &mut build_system, &mut world_mesh,
                        renderer.get_device(), &mut keys_pressed, &mut ui_system);
        }

        // Update camera from continuous input
        update_camera_from_input(&window, &mut camera, delta_time);

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
            // Update UI and get draw data
            let window_size = window.get_size();
            let (ui_actions, draw_data) = ui_system.update_and_render(
                window_size,
                &mut build_system,
                &camera,
                delta_time,
                time_system.get_time(),
                time_system.get_speed(),
                time_system.is_paused(),
                &time_system.get_time_string(),
                time_system.get_day_phase(),
            );

            // Render 3D scene with UI overlay
            renderer.render_frame_with_ui(&world_mesh, &preview_mesh, &camera, time_system.get_time(), time_system.get_time_of_day(), draw_data);

            // Handle UI actions
            for action in ui_actions {
                handle_ui_action(action, &mut build_system, &mut world, &mut world_mesh, renderer.get_device(), &mut time_system, &camera);
            }
        }

        // Small sleep to prevent 100% CPU usage
        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    Ok(())
}

fn handle_event(
    event: &WindowEvent,
    camera: &mut Camera,
    world: &mut VoxelWorld,
    build_system: &mut EngineerBuildSystem,
    world_mesh: &mut Mesh,
    device: &metal::Device,
    keys_pressed: &mut HashSet<u16>,
    ui_system: &mut SimpleUISystem,
) {
    match event {
        WindowEvent::KeyPressed(key_code) => {
            keys_pressed.insert(*key_code);

            // Pass key events to UI system first
            ui_system.handle_key_input(*key_code);

            match *key_code {
                key_codes::B => {
                    build_system.cycle_build_mode();
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
                    build_system.select_material_by_index(material_index);
                    print_build_status(build_system);
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
                    // Remove block
                    if let Some(hit_pos) = raycast_world(camera, world) {
                        if build_system.remove_block(world, hit_pos) {
                            regenerate_world_mesh(world, world_mesh, device, camera);
                        }
                    }
                }
                MouseButton::Right => {
                    // Place block or build structure
                    if let Some(hit_pos) = raycast_world(camera, world) {
                        let placed_pos = (hit_pos.0, hit_pos.1 + 1, hit_pos.2); // Place above hit
                        if build_system.build_at_position(world, placed_pos) {
                            regenerate_world_mesh(world, world_mesh, device, camera);
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

fn update_camera_from_input(window: &NativeWindow, camera: &mut Camera, _delta_time: f32) {
    let mut forward = 0.0;
    let mut right = 0.0;
    let mut up = 0.0;

    let speed = if window.is_key_pressed(key_codes::LEFT_SHIFT) { 2.0 } else { 1.0 };

    if window.is_key_pressed(key_codes::W) {
        forward += speed;
    }
    if window.is_key_pressed(key_codes::S) {
        forward -= speed;
    }
    if window.is_key_pressed(key_codes::A) {
        right -= speed;
    }
    if window.is_key_pressed(key_codes::D) {
        right += speed;
    }
    if window.is_key_pressed(key_codes::SPACE) {
        up += speed;
    }
    if window.is_key_pressed(key_codes::LEFT_SHIFT) {
        up -= speed;
    }

    if forward != 0.0 || right != 0.0 || up != 0.0 {
        camera.update_from_input(forward, right, up, 0.0, 0.0);
    }
}

struct TimeOfDaySystem {
    time_of_day: f32,     // Current time in hours (0.0 - 24.0)
    time_speed: f32,      // Speed multiplier (1.0 = normal, 0.0 = paused)
    is_paused: bool,      // Whether time is paused
}

impl TimeOfDaySystem {
    fn new() -> Self {
        Self {
            time_of_day: 12.0, // Start at noon
            time_speed: 1.0,   // Normal speed
            is_paused: false,
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
}

fn raycast_world(camera: &Camera, world: &VoxelWorld) -> Option<(i32, i32, i32)> {
    let forward = camera.get_forward_vector();
    let origin = Vector3::new(camera.eye.x, camera.eye.y, camera.eye.z);
    world.raycast(origin, forward, 10.0)
}

fn regenerate_world_mesh(world: &VoxelWorld, world_mesh: &mut Mesh, device: &metal::Device, camera: &Camera) {
    world_mesh.clear();
    generate_world_mesh(world, world_mesh, camera);
    world_mesh.update_buffers(device);
}

fn generate_preview_mesh(
    world: &VoxelWorld,
    preview_mesh: &mut Mesh,
    build_system: &EngineerBuildSystem,
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
            if world.get_voxel_world(preview_pos.0, preview_pos.1, preview_pos.2) == VoxelType::Air {
                add_ghost_block(preview_mesh, preview_pos, preview_color, &texture_atlas);
            }
        },
        BuildMode::Wall => {
            // 5x3 wall preview
            for x_offset in 0..5 {
                for y_offset in 0..3 {
                    let block_pos = (preview_pos.0 + x_offset, preview_pos.1 + y_offset, preview_pos.2);
                    if world.get_voxel_world(block_pos.0, block_pos.1, block_pos.2) == VoxelType::Air {
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
                    if world.get_voxel_world(block_pos.0, block_pos.1, block_pos.2) == VoxelType::Air {
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
                        if world.get_voxel_world(block_pos.0, block_pos.1, block_pos.2) == VoxelType::Air {
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
                        if world.get_voxel_world(block_pos.0, block_pos.1, block_pos.2) == VoxelType::Air {
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
                            if world.get_voxel_world(block_pos.0, block_pos.1, block_pos.2) == VoxelType::Air {
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
                            if world.get_voxel_world(block_pos.0, block_pos.1, block_pos.2) == VoxelType::Air {
                                add_ghost_block(preview_mesh, block_pos, preview_color, &texture_atlas);
                            }
                        }

                        // Railings (sides of the bridge, height 2)
                        for railing_height in 1..=2 {
                            // Left railing
                            let left_pos = (preview_pos.0 + x_offset, preview_pos.1 + railing_height, preview_pos.2 - 2);
                            if world.get_voxel_world(left_pos.0, left_pos.1, left_pos.2) == VoxelType::Air {
                                add_ghost_block(preview_mesh, left_pos, preview_color, &texture_atlas);
                            }

                            // Right railing
                            let right_pos = (preview_pos.0 + x_offset, preview_pos.1 + railing_height, preview_pos.2 + 2);
                            if world.get_voxel_world(right_pos.0, right_pos.1, right_pos.2) == VoxelType::Air {
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
                            if world.get_voxel_world(block_pos.0, block_pos.1, block_pos.2) == VoxelType::Air {
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
                        if world.get_voxel_world(block_pos.0, block_pos.1, block_pos.2) == VoxelType::Air {
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
                            if world.get_voxel_world(block_pos.0, block_pos.1, block_pos.2) == VoxelType::Air {
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

                                    if world.get_voxel_world(block_pos.0, block_pos.1, block_pos.2) == VoxelType::Air {
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
                            if world.get_voxel_world(block_pos.0, block_pos.1, block_pos.2) == VoxelType::Air {
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
        }
    }
}

fn add_ghost_block(mesh: &mut Mesh, pos: (i32, i32, i32), color: [f32; 3], texture_atlas: &TextureAtlas) {
    let (x, y, z) = (pos.0 as f32, pos.1 as f32, pos.2 as f32);

    // Get UV coordinates for the current material (use stone as default for preview)
    let tile_uv = texture_atlas.get_uv(VoxelType::Stone);

    // Define all 6 faces of a cube
    let faces = [
        // Front (+Z)
        ([[x, y, z + 1.0], [x + 1.0, y, z + 1.0], [x + 1.0, y + 1.0, z + 1.0], [x, y + 1.0, z + 1.0]], [0.0, 0.0, 1.0]),
        // Back (-Z)
        ([[x + 1.0, y, z], [x, y, z], [x, y + 1.0, z], [x + 1.0, y + 1.0, z]], [0.0, 0.0, -1.0]),
        // Right (+X)
        ([[x + 1.0, y, z + 1.0], [x + 1.0, y, z], [x + 1.0, y + 1.0, z], [x + 1.0, y + 1.0, z + 1.0]], [1.0, 0.0, 0.0]),
        // Left (-X)
        ([[x, y, z], [x, y, z + 1.0], [x, y + 1.0, z + 1.0], [x, y + 1.0, z]], [-1.0, 0.0, 0.0]),
        // Top (+Y)
        ([[x, y + 1.0, z + 1.0], [x + 1.0, y + 1.0, z + 1.0], [x + 1.0, y + 1.0, z], [x, y + 1.0, z]], [0.0, 1.0, 0.0]),
        // Bottom (-Y)
        ([[x, y, z], [x + 1.0, y, z], [x + 1.0, y, z + 1.0], [x, y, z + 1.0]], [0.0, -1.0, 0.0]),
    ];

    // Add all faces
    for (vertices, normal) in faces.iter() {
        mesh.add_quad_with_uv(*vertices, color, *normal, tile_uv.coords);
    }
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

fn print_build_status(build_system: &EngineerBuildSystem) {
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

// Import game logic modules
mod game {
    use cgmath::Vector3;
    use std::collections::{HashMap, VecDeque};
    use serde::{Serialize, Deserialize};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum VoxelType {
        Air,
        Stone,
        Dirt,
        Grass,
        Sand,
        Water,
        Wood,
        Leaves,
        Crystal,
        Lava,
    }

    impl VoxelType {
        pub fn get_color(&self) -> [f32; 3] {
            match self {
                VoxelType::Air => [0.0, 0.0, 0.0],
                VoxelType::Stone => [0.6, 0.6, 0.6],
                VoxelType::Dirt => [0.6, 0.4, 0.2],
                VoxelType::Grass => [0.2, 0.8, 0.2],
                VoxelType::Sand => [0.9, 0.8, 0.6],
                VoxelType::Water => [0.2, 0.4, 0.8],
                VoxelType::Wood => [0.6, 0.4, 0.2],
                VoxelType::Leaves => [0.1, 0.6, 0.1],
                VoxelType::Crystal => [0.7, 0.3, 0.9], // Purple
                VoxelType::Lava => [1.0, 0.3, 0.1], // Orange-red
            }
        }
    }

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

    pub struct VoxelWorld {
        pub chunks: HashMap<(i32, i32, i32), Chunk>,
        pub chunk_size: usize,
    }

    impl VoxelWorld {
        pub fn new() -> Self {
            let mut world = Self {
                chunks: HashMap::new(),
                chunk_size: 16,
            };
            world.generate_terrain();
            world
        }

        fn generate_terrain(&mut self) {
            println!("🌍 Starting terrain generation with chunk_size: {}", self.chunk_size);
            let mut total_chunks = 0;
            let mut total_solid_voxels = 0;

            // Generate terrain chunks
            for cx in -2..=2 {
                for cz in -2..=2 {
                    total_chunks += 1;
                    let mut chunk = Chunk::new(self.chunk_size);
                    let mut chunk_solid_voxels = 0;

                    for x in 0..self.chunk_size {
                        for z in 0..self.chunk_size {
                            let world_x = cx * self.chunk_size as i32 + x as i32;
                            let world_z = cz * self.chunk_size as i32 + z as i32;

                            // Enhanced height map
                            let height = 8 + ((world_x as f32 * 0.1).sin() * 4.0
                                        + (world_z as f32 * 0.1).cos() * 4.0
                                        + (world_x as f32 * 0.05 + world_z as f32 * 0.05).sin() * 2.0) as i32;

                            for y in 0..self.chunk_size {
                                let world_y = y as i32;
                                let voxel = if world_y < height - 3 {
                                    VoxelType::Stone
                                } else if world_y < height - 1 {
                                    VoxelType::Dirt
                                } else if world_y < height {
                                    VoxelType::Grass
                                } else {
                                    VoxelType::Air
                                };

                                if voxel != VoxelType::Air {
                                    chunk_solid_voxels += 1;
                                }

                                chunk.set_voxel(x, y, z, voxel);
                            }
                        }
                    }

                    total_solid_voxels += chunk_solid_voxels;
                    println!("Generated chunk ({}, 0, {}) with {} solid voxels", cx, cz, chunk_solid_voxels);
                    self.chunks.insert((cx, 0, cz), chunk);
                }
            }

            println!("Generated {} chunks with {} total solid voxels", total_chunks, total_solid_voxels);

            // Add some features
            self.set_voxel_world(0, 12, 0, VoxelType::Crystal);
            self.set_voxel_world(5, 10, 5, VoxelType::Wood);
            self.set_voxel_world(5, 11, 5, VoxelType::Wood);
            self.set_voxel_world(5, 12, 5, VoxelType::Leaves);
            println!("Added special features");
        }

        pub fn set_voxel_world(&mut self, x: i32, y: i32, z: i32, voxel_type: VoxelType) {
            let chunk_x = x.div_euclid(self.chunk_size as i32);
            let chunk_z = z.div_euclid(self.chunk_size as i32);
            let chunk_y = y.div_euclid(self.chunk_size as i32);

            let local_x = x.rem_euclid(self.chunk_size as i32) as usize;
            let local_y = y.rem_euclid(self.chunk_size as i32) as usize;
            let local_z = z.rem_euclid(self.chunk_size as i32) as usize;

            if let Some(chunk) = self.chunks.get_mut(&(chunk_x, chunk_y, chunk_z)) {
                chunk.set_voxel(local_x, local_y, local_z, voxel_type);
            }
        }

        pub fn get_voxel_world(&self, x: i32, y: i32, z: i32) -> VoxelType {
            let chunk_x = x.div_euclid(self.chunk_size as i32);
            let chunk_z = z.div_euclid(self.chunk_size as i32);
            let chunk_y = y.div_euclid(self.chunk_size as i32);

            let local_x = x.rem_euclid(self.chunk_size as i32) as usize;
            let local_y = y.rem_euclid(self.chunk_size as i32) as usize;
            let local_z = z.rem_euclid(self.chunk_size as i32) as usize;

            if let Some(chunk) = self.chunks.get(&(chunk_x, chunk_y, chunk_z)) {
                chunk.get_voxel(local_x, local_y, local_z)
            } else {
                VoxelType::Air
            }
        }

        pub fn raycast(&self, origin: Vector3<f32>, direction: Vector3<f32>, max_distance: f32) -> Option<(i32, i32, i32)> {
            use cgmath::InnerSpace;
            let mut current = origin;
            let step = direction.normalize() * 0.1;
            let mut distance = 0.0;

            while distance < max_distance {
                let x = current.x.floor() as i32;
                let y = current.y.floor() as i32;
                let z = current.z.floor() as i32;

                if self.get_voxel_world(x, y, z) != VoxelType::Air {
                    return Some((x, y, z));
                }

                current += step;
                distance += 0.1;
            }

            None
        }
    }

    // Build System (simplified for initial implementation)
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum BuildMode {
        Single,
        Wall,
        Floor,
        Roof,
        Template,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum TemplateType {
        Stairs,
        Arch,
        Bridge,
        Tower,
        House,
    }

    pub struct EngineerBuildSystem {
        mode: BuildMode,
        current_material: VoxelType,
        current_template: TemplateType,
        inventory: HashMap<VoxelType, u32>,
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

    impl EngineerBuildSystem {
        pub fn new() -> Self {
            let mut inventory = HashMap::new();

            // Unlimited basic materials
            inventory.insert(VoxelType::Stone, 999);
            inventory.insert(VoxelType::Dirt, 999);
            inventory.insert(VoxelType::Grass, 999);
            inventory.insert(VoxelType::Sand, 999);
            inventory.insert(VoxelType::Water, 999);
            inventory.insert(VoxelType::Wood, 999);
            inventory.insert(VoxelType::Leaves, 999);

            // Limited special materials
            inventory.insert(VoxelType::Crystal, 100);
            inventory.insert(VoxelType::Lava, 50);

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
                BuildMode::Template => BuildMode::Single,
            };
        }

        pub fn cycle_template(&mut self) {
            self.current_template = match self.current_template {
                TemplateType::Stairs => TemplateType::Arch,
                TemplateType::Arch => TemplateType::Bridge,
                TemplateType::Bridge => TemplateType::Tower,
                TemplateType::Tower => TemplateType::House,
                TemplateType::House => TemplateType::Stairs,
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
                VoxelType::Stone, VoxelType::Dirt, VoxelType::Grass, VoxelType::Sand,
                VoxelType::Water, VoxelType::Wood, VoxelType::Crystal, VoxelType::Lava,
            ];

            if index < materials.len() {
                self.current_material = materials[index];
            }
        }

        pub fn remove_block(&mut self, world: &mut VoxelWorld, pos: (i32, i32, i32)) -> bool {
            let old_voxel = world.get_voxel_world(pos.0, pos.1, pos.2);
            if old_voxel != VoxelType::Air {
                world.set_voxel_world(pos.0, pos.1, pos.2, VoxelType::Air);

                // Add to inventory
                if let Some(count) = self.inventory.get_mut(&old_voxel) {
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
            }
        }

        fn place_single_block(&mut self, world: &mut VoxelWorld, pos: (i32, i32, i32)) -> bool {
            if world.get_voxel_world(pos.0, pos.1, pos.2) == VoxelType::Air {
                // Check if we have materials
                if let Some(count) = self.inventory.get_mut(&self.current_material) {
                    if *count > 0 {
                        *count -= 1;
                        world.set_voxel_world(pos.0, pos.1, pos.2, self.current_material);

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
            if let Some(count) = self.inventory.get(&self.current_material) {
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
                    if world.get_voxel_world(block_pos.0, block_pos.1, block_pos.2) == VoxelType::Air {
                        world.set_voxel_world(block_pos.0, block_pos.1, block_pos.2, self.current_material);

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
                if let Some(count) = self.inventory.get_mut(&self.current_material) {
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
            if let Some(count) = self.inventory.get(&self.current_material) {
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
                    if world.get_voxel_world(block_pos.0, block_pos.1, block_pos.2) == VoxelType::Air {
                        world.set_voxel_world(block_pos.0, block_pos.1, block_pos.2, self.current_material);

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
                if let Some(count) = self.inventory.get_mut(&self.current_material) {
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
            if let Some(count) = self.inventory.get(&self.current_material) {
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

                        if world.get_voxel_world(block_pos.0, block_pos.1, block_pos.2) == VoxelType::Air {
                            world.set_voxel_world(block_pos.0, block_pos.1, block_pos.2, self.current_material);

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
                if let Some(count) = self.inventory.get_mut(&self.current_material) {
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
            }
        }

        fn build_stairs(&mut self, world: &mut VoxelWorld, pos: (i32, i32, i32)) -> bool {
            // Build a 4-step staircase going up in the X direction
            let blocks_needed = 4;

            if let Some(count) = self.inventory.get(&self.current_material) {
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

                if world.get_voxel_world(block_pos.0, block_pos.1, block_pos.2) == VoxelType::Air {
                    world.set_voxel_world(block_pos.0, block_pos.1, block_pos.2, self.current_material);

                    operations.push(BuildOperation {
                        position: block_pos,
                        old_voxel: VoxelType::Air,
                        new_voxel: self.current_material,
                    });

                    blocks_placed += 1;
                }
            }

            if blocks_placed > 0 {
                if let Some(count) = self.inventory.get_mut(&self.current_material) {
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
            if let Some(count) = self.inventory.get(&self.current_material) {
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
                    if world.get_voxel_world(block_pos.0, block_pos.1, block_pos.2) == VoxelType::Air {
                        world.set_voxel_world(block_pos.0, block_pos.1, block_pos.2, self.current_material);

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
                if let Some(count) = self.inventory.get_mut(&self.current_material) {
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
            if let Some(count) = self.inventory.get(&self.current_material) {
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

                    if world.get_voxel_world(block_pos.0, block_pos.1, block_pos.2) == VoxelType::Air {
                        world.set_voxel_world(block_pos.0, block_pos.1, block_pos.2, self.current_material);

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
                    if world.get_voxel_world(left_pos.0, left_pos.1, left_pos.2) == VoxelType::Air {
                        world.set_voxel_world(left_pos.0, left_pos.1, left_pos.2, self.current_material);

                        operations.push(BuildOperation {
                            position: left_pos,
                            old_voxel: VoxelType::Air,
                            new_voxel: self.current_material,
                        });

                        blocks_placed += 1;
                    }

                    // Right railing
                    let right_pos = (pos.0 + x_offset, pos.1 + railing_height, pos.2 + 2);
                    if world.get_voxel_world(right_pos.0, right_pos.1, right_pos.2) == VoxelType::Air {
                        world.set_voxel_world(right_pos.0, right_pos.1, right_pos.2, self.current_material);

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
                if let Some(count) = self.inventory.get_mut(&self.current_material) {
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
            if let Some(count) = self.inventory.get(&self.current_material) {
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

                    if world.get_voxel_world(block_pos.0, block_pos.1, block_pos.2) == VoxelType::Air {
                        world.set_voxel_world(block_pos.0, block_pos.1, block_pos.2, self.current_material);

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

                if world.get_voxel_world(block_pos.0, block_pos.1, block_pos.2) == VoxelType::Air {
                    world.set_voxel_world(block_pos.0, block_pos.1, block_pos.2, self.current_material);

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
                if let Some(count) = self.inventory.get_mut(&self.current_material) {
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
            if let Some(count) = self.inventory.get(&self.current_material) {
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

                    if world.get_voxel_world(block_pos.0, block_pos.1, block_pos.2) == VoxelType::Air {
                        world.set_voxel_world(block_pos.0, block_pos.1, block_pos.2, self.current_material);

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

                            if world.get_voxel_world(block_pos.0, block_pos.1, block_pos.2) == VoxelType::Air {
                                world.set_voxel_world(block_pos.0, block_pos.1, block_pos.2, self.current_material);

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

                    if world.get_voxel_world(block_pos.0, block_pos.1, block_pos.2) == VoxelType::Air {
                        world.set_voxel_world(block_pos.0, block_pos.1, block_pos.2, self.current_material);

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
                if let Some(count) = self.inventory.get_mut(&self.current_material) {
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
                    world.set_voxel_world(op.position.0, op.position.1, op.position.2, op.old_voxel);
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
                    world.set_voxel_world(op.position.0, op.position.1, op.position.2, op.old_voxel);
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

        // Getters for UI
        pub fn get_current_mode(&self) -> BuildMode { self.mode }
        pub fn get_current_material(&self) -> VoxelType { self.current_material }
        pub fn is_grid_snap_enabled(&self) -> bool { self.grid_snap }
        pub fn get_inventory(&self) -> &HashMap<VoxelType, u32> { &self.inventory }
    }
}

fn generate_world_mesh(world: &game::VoxelWorld, mesh: &mut Mesh, camera: &Camera) {
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
        for x in 0..chunk.size {
            for y in 0..chunk.size {
                for z in 0..chunk.size {
                    total_voxels += 1;
                    let voxel = chunk.get_voxel(x, y, z);
                    if voxel != game::VoxelType::Air {
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
    world: &game::VoxelWorld,
    mesh: &mut Mesh,
    chunk_pos: (i32, i32, i32),
    chunk: &game::Chunk,
    texture_atlas: &TextureAtlas,
) {
    let (chunk_x, chunk_y, chunk_z) = chunk_pos;
    let chunk_size = chunk.size;

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
    world: &game::VoxelWorld,
    mesh: &mut Mesh,
    chunk_pos: (i32, i32, i32),
    chunk: &game::Chunk,
    texture_atlas: &TextureAtlas,
    direction: FaceDirection,
) {
    let (chunk_x, chunk_y, chunk_z) = chunk_pos;
    let chunk_size = chunk.size;

    // Create a 2D mask for this face direction
    // The mask tells us which positions need a face in this direction
    let mut face_mask: Vec<Vec<Option<game::VoxelType>>> = vec![vec![None; chunk_size]; chunk_size];
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
    world: &game::VoxelWorld,
    chunk_pos: (i32, i32, i32),
    chunk: &game::Chunk,
    face_mask: &mut Vec<Vec<Option<game::VoxelType>>>,
    direction: FaceDirection,
) {
    let (chunk_x, chunk_y, chunk_z) = chunk_pos;
    let chunk_size = chunk.size;

    // For each direction, we scan through the chunk in a different order
    // and check if a face is needed at each position
    match direction {
        FaceDirection::PosX => {
            // For +X faces, scan through YZ plane, check if voxel needs +X face
            for y in 0..chunk_size {
                for z in 0..chunk_size {
                    for x in 0..chunk_size {
                        let voxel = chunk.get_voxel(x, y, z);
                        if voxel != game::VoxelType::Air {
                            // Check if adjacent voxel in +X direction is air
                            let world_x = chunk_x * chunk_size as i32 + x as i32;
                            let world_y = chunk_y * chunk_size as i32 + y as i32;
                            let world_z = chunk_z * chunk_size as i32 + z as i32;

                            let adjacent = world.get_voxel_world(world_x + 1, world_y, world_z);
                            if adjacent == game::VoxelType::Air {
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
                        let voxel = chunk.get_voxel(x, y, z);
                        if voxel != game::VoxelType::Air {
                            let world_x = chunk_x * chunk_size as i32 + x as i32;
                            let world_y = chunk_y * chunk_size as i32 + y as i32;
                            let world_z = chunk_z * chunk_size as i32 + z as i32;

                            let adjacent = world.get_voxel_world(world_x - 1, world_y, world_z);
                            if adjacent == game::VoxelType::Air {
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
                        let voxel = chunk.get_voxel(x, y, z);
                        if voxel != game::VoxelType::Air {
                            let world_x = chunk_x * chunk_size as i32 + x as i32;
                            let world_y = chunk_y * chunk_size as i32 + y as i32;
                            let world_z = chunk_z * chunk_size as i32 + z as i32;

                            let adjacent = world.get_voxel_world(world_x, world_y + 1, world_z);
                            if adjacent == game::VoxelType::Air {
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
                        let voxel = chunk.get_voxel(x, y, z);
                        if voxel != game::VoxelType::Air {
                            let world_x = chunk_x * chunk_size as i32 + x as i32;
                            let world_y = chunk_y * chunk_size as i32 + y as i32;
                            let world_z = chunk_z * chunk_size as i32 + z as i32;

                            let adjacent = world.get_voxel_world(world_x, world_y - 1, world_z);
                            if adjacent == game::VoxelType::Air {
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
                        let voxel = chunk.get_voxel(x, y, z);
                        if voxel != game::VoxelType::Air {
                            let world_x = chunk_x * chunk_size as i32 + x as i32;
                            let world_y = chunk_y * chunk_size as i32 + y as i32;
                            let world_z = chunk_z * chunk_size as i32 + z as i32;

                            let adjacent = world.get_voxel_world(world_x, world_y, world_z + 1);
                            if adjacent == game::VoxelType::Air {
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
                        let voxel = chunk.get_voxel(x, y, z);
                        if voxel != game::VoxelType::Air {
                            let world_x = chunk_x * chunk_size as i32 + x as i32;
                            let world_y = chunk_y * chunk_size as i32 + y as i32;
                            let world_z = chunk_z * chunk_size as i32 + z as i32;

                            let adjacent = world.get_voxel_world(world_x, world_y, world_z - 1);
                            if adjacent == game::VoxelType::Air {
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
    face_mask: &Vec<Vec<Option<game::VoxelType>>>,
    processed: &mut Vec<Vec<bool>>,
    start_u: usize,
    start_v: usize,
    material: game::VoxelType,
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
    chunk: &game::Chunk,
    direction: FaceDirection,
    start_u: usize,
    start_v: usize,
    width: usize,
    height: usize,
    material: game::VoxelType,
    texture_atlas: &TextureAtlas,
) {
    let (chunk_x, chunk_y, chunk_z) = chunk_pos;
    let chunk_size = chunk.size;

    let color = material.get_color();
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
    build_system: &mut EngineerBuildSystem,
    world: &mut VoxelWorld,
    world_mesh: &mut Mesh,
    device: &metal::Device,
    time_system: &mut TimeOfDaySystem,
    camera: &Camera,
) {
    match action {
        UIAction::SelectMaterial(material) => {
            build_system.select_material(material);
            print_build_status(build_system);
        }
        UIAction::ToggleBuildMode => {
            build_system.cycle_build_mode();
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