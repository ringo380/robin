// Robin Engine 3D Demo - First-Person Engineer Build Mode
// This demonstrates the integration of graphics, physics, and input systems

use std::time::{Duration, Instant};

// Import our engine systems (these would normally be proper imports)
use robin::graphics::{GraphicsEngine, Scene, RenderObject, Transform, Light, CameraState, UIElement};
use robin::physics::{PhysicsEngine, RigidBody, Collider, BodyId};
use robin::input::{InputSystem, MovementInput, CameraInput, ActionInput, UIInput, Key, MouseButton};
use robin::engine::{EngineerCharacter, WorldConstructionSystem, AdvancedToolsSuite};

fn main() {
    println!("🎮 Robin Engine 3D Demo - Engineer Build Mode");
    println!("==============================================");
    
    // Initialize core systems
    let mut demo = Demo::new();
    
    if let Err(e) = demo.initialize() {
        eprintln!("Failed to initialize demo: {}", e);
        return;
    }
    
    println!("🔧 All systems initialized successfully!");
    println!("\n📋 Controls:");
    println!("  WASD - Move around");
    println!("  Mouse - Look around");
    println!("  Space - Move up / Jump");
    println!("  Shift - Move down / Crouch");
    println!("  E - Interact with objects");
    println!("  B - Build mode");
    println!("  X - Destroy mode");
    println!("  Tab - Toggle inventory");
    println!("  Escape - Menu");
    println!("  F11 - Fullscreen");
    println!("  F12 - Screenshot");
    
    // Run the main game loop
    demo.run();
}

struct Demo {
    // Core engine systems
    graphics: GraphicsEngine,
    physics: PhysicsEngine,
    input: InputSystem,
    
    // Game systems
    engineer: EngineerCharacter,
    world: WorldConstructionSystem,
    tools: AdvancedToolsSuite,
    
    // Demo state
    running: bool,
    camera_position: [f32; 3],
    camera_rotation: [f32; 3],
    
    // 3D world objects
    world_objects: Vec<WorldObject>,
    physics_bodies: Vec<BodyId>,
    
    // Performance tracking
    frame_count: u64,
    last_fps_update: Instant,
    fps: f32,
    
    // Build mode state
    build_mode: bool,
    selected_tool: u8,
    target_position: Option<[f32; 3]>,
}

impl Demo {
    fn new() -> Self {
        Self {
            graphics: GraphicsEngine::new(),
            physics: PhysicsEngine::new(),
            input: InputSystem::new(),
            engineer: EngineerCharacter::new("Alex Builder"),
            world: WorldConstructionSystem::new(),
            tools: AdvancedToolsSuite::new(),
            running: true,
            camera_position: [0.0, 1.8, 5.0], // Start at eye level, back from origin
            camera_rotation: [0.0, 0.0, 0.0],
            world_objects: Vec::new(),
            physics_bodies: Vec::new(),
            frame_count: 0,
            last_fps_update: Instant::now(),
            fps: 0.0,
            build_mode: false,
            selected_tool: 0,
            target_position: None,
        }
    }
    
    fn initialize(&mut self) -> Result<(), String> {
        println!("🔧 Initializing graphics system...");
        self.graphics.initialize()?;
        
        println!("⚡ Initializing physics system...");
        self.physics.initialize()?;
        
        println!("🎮 Setting up input system...");
        // Input system doesn't need initialization in this demo
        
        println!("👨‍🔧 Setting up engineer character...");
        self.engineer.initialize();
        
        println!("🌍 Creating demo world...");
        self.create_demo_world()?;
        
        println!("🏗️ Setting up building tools...");
        self.tools.initialize();
        
        Ok(())
    }
    
    fn create_demo_world(&mut self) -> Result<(), String> {
        // Create ground plane
        let ground_body = RigidBody::new_static(Collider::Box { 
            half_extents: [50.0, 0.5, 50.0] 
        });
        let ground_id = self.physics.add_rigid_body(ground_body);
        self.physics.set_body_position(ground_id, [0.0, -0.5, 0.0]);
        
        let ground_object = WorldObject {
            name: "Ground".to_string(),
            mesh_id: "plane".to_string(),
            material_id: "concrete".to_string(),
            physics_body: Some(ground_id),
            transform: Transform {
                position: [0.0, -0.5, 0.0],
                rotation: [0.0, 0.0, 0.0],
                scale: [100.0, 1.0, 100.0],
            },
        };
        
        self.world_objects.push(ground_object);
        self.physics_bodies.push(ground_id);
        
        // Create some demo structures
        self.create_demo_structures()?;
        
        // Add some physics objects to interact with
        self.create_physics_objects()?;
        
        Ok(())
    }
    
    fn create_demo_structures(&mut self) -> Result<(), String> {
        // Create a simple building
        let building_positions = vec![
            [10.0, 1.0, 10.0],   // Corner 1
            [15.0, 1.0, 10.0],   // Corner 2  
            [15.0, 1.0, 15.0],   // Corner 3
            [10.0, 1.0, 15.0],   // Corner 4
            [12.5, 3.0, 12.5],   // Roof center
        ];
        
        for (i, position) in building_positions.iter().enumerate() {
            let body = RigidBody::new_static(Collider::Box { 
                half_extents: [1.0, 1.0, 1.0] 
            });
            let body_id = self.physics.add_rigid_body(body);
            self.physics.set_body_position(body_id, *position);
            
            let object = WorldObject {
                name: format!("Building_Block_{}", i),
                mesh_id: "cube".to_string(),
                material_id: if i == 4 { "steel" } else { "concrete" },
                physics_body: Some(body_id),
                transform: Transform {
                    position: *position,
                    rotation: [0.0, 0.0, 0.0],
                    scale: [2.0, 2.0, 2.0],
                },
            };
            
            self.world_objects.push(object);
            self.physics_bodies.push(body_id);
        }
        
        Ok(())
    }
    
    fn create_physics_objects(&mut self) -> Result<(), String> {
        // Create some spheres to demonstrate physics
        for i in 0..5 {
            let x = -10.0 + i as f32 * 2.0;
            let y = 5.0 + i as f32 * 2.0;
            let z = 0.0;
            
            let body = RigidBody::new_dynamic(1.0, Collider::Sphere { radius: 0.5 });
            let body_id = self.physics.add_rigid_body(body);
            self.physics.set_body_position(body_id, [x, y, z]);
            
            let object = WorldObject {
                name: format!("Sphere_{}", i),
                mesh_id: "sphere".to_string(),
                material_id: "steel".to_string(),
                physics_body: Some(body_id),
                transform: Transform {
                    position: [x, y, z],
                    rotation: [0.0, 0.0, 0.0],
                    scale: [1.0, 1.0, 1.0],
                },
            };
            
            self.world_objects.push(object);
            self.physics_bodies.push(body_id);
        }
        
        Ok(())
    }
    
    fn run(&mut self) {
        let target_fps = 60.0;
        let target_frame_time = Duration::from_secs_f32(1.0 / target_fps);
        let mut last_frame_time = Instant::now();
        
        println!("\n🚀 Starting main game loop...");
        println!("Press Escape to exit\n");
        
        while self.running {
            let current_time = Instant::now();
            let delta_time = current_time.duration_since(last_frame_time).as_secs_f32();
            last_frame_time = current_time;
            
            // Update all systems
            self.update(delta_time);
            
            // Render frame
            if let Err(e) = self.render() {
                eprintln!("Render error: {}", e);
                break;
            }
            
            // Update performance metrics
            self.update_performance_metrics();
            
            // Simple frame rate limiting (in a real engine, this would be more sophisticated)
            let frame_time = current_time.elapsed();
            if frame_time < target_frame_time {
                std::thread::sleep(target_frame_time - frame_time);
            }
        }
        
        println!("\n👋 Demo finished. Thanks for trying Robin Engine!");
    }
    
    fn update(&mut self, delta_time: f32) {
        // Update input system
        self.update_input(delta_time);
        
        // Update physics simulation
        if let Err(e) = self.physics.step_simulation(delta_time) {
            eprintln!("Physics error: {}", e);
        }
        
        // Update engineer character
        self.update_engineer(delta_time);
        
        // Update camera
        self.update_camera(delta_time);
        
        // Update world objects from physics
        self.update_world_objects();
        
        // Update building system
        if self.build_mode {
            self.update_building_mode(delta_time);
        }
        
        // Check for exit condition
        let ui_input = self.input.get_ui_input();
        if ui_input.menu_toggle {
            self.running = false;
        }
    }
    
    fn update_input(&mut self, delta_time: f32) {
        // In a real implementation, this would receive events from the windowing system
        // For this demo, we'll simulate some basic input
        self.input.update(delta_time);
        
        // Simulate basic movement input (in a real app, this comes from events)
        // This is just for demonstration - real input would come from window events
    }
    
    fn update_engineer(&mut self, delta_time: f32) {
        let movement_input = self.input.get_movement_input();
        let action_input = self.input.get_action_input();
        
        // Update engineer position based on movement
        let speed = if movement_input.sprint { 10.0 } else { 5.0 };
        let movement_speed = speed * delta_time;
        
        // Calculate movement vector in world space
        let forward = [
            -self.camera_rotation[1].to_radians().sin(),
            0.0,
            -self.camera_rotation[1].to_radians().cos(),
        ];
        
        let right = [
            self.camera_rotation[1].to_radians().cos(),
            0.0,
            -self.camera_rotation[1].to_radians().sin(),
        ];
        
        if movement_input.forward {
            self.camera_position[0] += forward[0] * movement_speed;
            self.camera_position[2] += forward[2] * movement_speed;
        }
        if movement_input.backward {
            self.camera_position[0] -= forward[0] * movement_speed;
            self.camera_position[2] -= forward[2] * movement_speed;
        }
        if movement_input.right {
            self.camera_position[0] += right[0] * movement_speed;
            self.camera_position[2] += right[2] * movement_speed;
        }
        if movement_input.left {
            self.camera_position[0] -= right[0] * movement_speed;
            self.camera_position[2] -= right[2] * movement_speed;
        }
        if movement_input.up {
            self.camera_position[1] += movement_speed;
        }
        if movement_input.down {
            self.camera_position[1] -= movement_speed;
        }
        
        // Handle building actions
        if action_input.build {
            self.build_mode = !self.build_mode;
            if self.build_mode {
                println!("🏗️ Build mode activated!");
            } else {
                println!("🚶 Build mode deactivated");
            }
        }
        
        if let Some(tool) = action_input.tool_select {
            self.selected_tool = tool;
            println!("🔧 Selected tool: {}", tool);
        }
        
        if action_input.interact {
            self.handle_interaction();
        }
    }
    
    fn update_camera(&mut self, delta_time: f32) {
        let camera_input = self.input.get_camera_input();
        
        // Update camera rotation from mouse input
        self.camera_rotation[1] += camera_input.mouse_delta[0] * 0.1; // Yaw
        self.camera_rotation[0] += camera_input.mouse_delta[1] * 0.1; // Pitch
        
        // Clamp pitch to prevent over-rotation
        self.camera_rotation[0] = self.camera_rotation[0].clamp(-89.0, 89.0);
        
        if camera_input.reset {
            self.camera_rotation = [0.0, 0.0, 0.0];
            println!("📷 Camera reset");
        }
    }
    
    fn update_world_objects(&mut self) {
        // Update object transforms from physics bodies
        for object in &mut self.world_objects {
            if let Some(body_id) = object.physics_body {
                if let Some(body) = self.physics.world.get_body(body_id) {
                    object.transform.position = body.position;
                    // Convert quaternion to euler angles (simplified)
                    // In a real engine, this would be a proper quaternion to euler conversion
                }
            }
        }
    }
    
    fn update_building_mode(&mut self, _delta_time: f32) {
        // Raycast from camera to find build target
        let ray_origin = self.camera_position;
        let ray_direction = self.get_camera_forward_vector();
        
        if let Some(hit) = self.physics.raycast(ray_origin, ray_direction, 100.0) {
            self.target_position = Some(hit.point);
            
            // Handle primary action (build)
            let action_input = self.input.get_action_input();
            if action_input.primary_action {
                self.place_block_at_target();
            }
            if action_input.secondary_action {
                self.remove_block_at_target();
            }
        } else {
            self.target_position = None;
        }
    }
    
    fn handle_interaction(&mut self) {
        let ray_origin = self.camera_position;
        let ray_direction = self.get_camera_forward_vector();
        
        if let Some(hit) = self.physics.raycast(ray_origin, ray_direction, 5.0) {
            // Find the object we hit
            for object in &self.world_objects {
                if object.physics_body == Some(hit.body_id) {
                    println!("🔍 Interacting with: {}", object.name);
                    break;
                }
            }
        }
    }
    
    fn place_block_at_target(&mut self) {
        if let Some(target) = self.target_position {
            // Snap to grid
            let grid_size = 2.0;
            let snapped_position = [
                (target[0] / grid_size).round() * grid_size,
                (target[1] / grid_size).round() * grid_size + 1.0, // Offset up by 1 unit
                (target[2] / grid_size).round() * grid_size,
            ];
            
            // Create new block
            let body = RigidBody::new_static(Collider::Box { 
                half_extents: [1.0, 1.0, 1.0] 
            });
            let body_id = self.physics.add_rigid_body(body);
            self.physics.set_body_position(body_id, snapped_position);
            
            let object = WorldObject {
                name: format!("Player_Block_{}", self.world_objects.len()),
                mesh_id: "cube".to_string(),
                material_id: match self.selected_tool {
                    0 => "concrete",
                    1 => "steel", 
                    _ => "concrete",
                },
                physics_body: Some(body_id),
                transform: Transform {
                    position: snapped_position,
                    rotation: [0.0, 0.0, 0.0],
                    scale: [2.0, 2.0, 2.0],
                },
            };
            
            self.world_objects.push(object);
            self.physics_bodies.push(body_id);
            
            println!("🧱 Placed block at ({:.1}, {:.1}, {:.1})", 
                     snapped_position[0], snapped_position[1], snapped_position[2]);
        }
    }
    
    fn remove_block_at_target(&mut self) {
        let ray_origin = self.camera_position;
        let ray_direction = self.get_camera_forward_vector();
        
        if let Some(hit) = self.physics.raycast(ray_origin, ray_direction, 100.0) {
            // Find and remove the object
            if let Some(index) = self.world_objects.iter().position(|obj| obj.physics_body == Some(hit.body_id)) {
                let object = self.world_objects.remove(index);
                if let Some(body_id) = object.physics_body {
                    self.physics.remove_rigid_body(body_id);
                    println!("💥 Removed: {}", object.name);
                }
            }
        }
    }
    
    fn get_camera_forward_vector(&self) -> [f32; 3] {
        let pitch = self.camera_rotation[0].to_radians();
        let yaw = self.camera_rotation[1].to_radians();
        
        [
            -yaw.sin() * pitch.cos(),
            -pitch.sin(),
            -yaw.cos() * pitch.cos(),
        ]
    }
    
    fn render(&mut self) -> Result<(), String> {
        // Create scene from world objects
        let mut scene = Scene {
            objects: Vec::new(),
            lights: vec![
                Light::Directional(robin::graphics::DirectionalLight {
                    direction: [-0.3, -0.8, -0.5],
                    color: [1.0, 0.95, 0.8],
                    intensity: 1.0,
                }),
            ],
            camera_state: CameraState {
                position: self.camera_position,
                rotation: self.camera_rotation,
            },
            ui_elements: Vec::new(),
        };
        
        // Add all world objects to scene
        for object in &self.world_objects {
            scene.objects.push(RenderObject {
                mesh_id: object.mesh_id.clone(),
                material_id: object.material_id.clone(),
                transform: object.transform.clone(),
                visible: true,
            });
        }
        
        // Add build mode cursor
        if self.build_mode {
            if let Some(target) = self.target_position {
                let cursor_position = [
                    (target[0] / 2.0).round() * 2.0,
                    (target[1] / 2.0).round() * 2.0 + 1.0,
                    (target[2] / 2.0).round() * 2.0,
                ];
                
                scene.objects.push(RenderObject {
                    mesh_id: "cube".to_string(),
                    material_id: "steel".to_string(),
                    transform: Transform {
                        position: cursor_position,
                        rotation: [0.0, 0.0, 0.0],
                        scale: [2.1, 2.1, 2.1], // Slightly larger for visibility
                    },
                    visible: true,
                });
            }
        }
        
        // Add UI elements
        scene.ui_elements.push(UIElement {
            text: format!("FPS: {:.1}", self.fps),
            position: [10.0, 10.0],
            color: [1.0, 1.0, 1.0, 1.0],
        });
        
        scene.ui_elements.push(UIElement {
            text: format!("Position: ({:.1}, {:.1}, {:.1})", 
                         self.camera_position[0], self.camera_position[1], self.camera_position[2]),
            position: [10.0, 30.0],
            color: [1.0, 1.0, 1.0, 1.0],
        });
        
        if self.build_mode {
            scene.ui_elements.push(UIElement {
                text: format!("BUILD MODE - Tool: {}", self.selected_tool),
                position: [10.0, 50.0],
                color: [0.0, 1.0, 0.0, 1.0],
            });
        }
        
        scene.ui_elements.push(UIElement {
            text: format!("Objects: {}", self.world_objects.len()),
            position: [10.0, 70.0],
            color: [1.0, 1.0, 1.0, 1.0],
        });
        
        // Render the scene
        self.graphics.render_frame(&scene)?;
        
        Ok(())
    }
    
    fn update_performance_metrics(&mut self) {
        self.frame_count += 1;
        
        let now = Instant::now();
        if now.duration_since(self.last_fps_update) >= Duration::from_secs(1) {
            self.fps = self.frame_count as f32 / now.duration_since(self.last_fps_update).as_secs_f32();
            self.frame_count = 0;
            self.last_fps_update = now;
        }
    }
}

#[derive(Debug, Clone)]
struct WorldObject {
    name: String,
    mesh_id: String,
    material_id: String,
    physics_body: Option<BodyId>,
    transform: Transform,
}

// Mock imports - in a real implementation these would be actual imports
mod robin {
    pub mod graphics {
        pub use crate::GraphicsEngine;
        pub use crate::{Scene, RenderObject, Transform, Light, DirectionalLight, CameraState, UIElement};
    }
    
    pub mod physics {
        pub use crate::{PhysicsEngine, RigidBody, Collider, BodyId};
    }
    
    pub mod input {
        pub use crate::{InputSystem, MovementInput, CameraInput, ActionInput, UIInput, Key, MouseButton};
    }
    
    pub mod engine {
        pub use crate::{EngineerCharacter, WorldConstructionSystem, AdvancedToolsSuite};
    }
}

// Mock implementations for the demo (these would be real implementations)
include!("../graphics/mod.rs");
include!("../physics/mod.rs");  
include!("../input/mod.rs");

#[derive(Debug, Clone)]
struct EngineerCharacter {
    name: String,
    position: [f32; 3],
    tools: Vec<String>,
}

impl EngineerCharacter {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            position: [0.0, 0.0, 0.0],
            tools: vec!["Builder".to_string(), "Wrench".to_string()],
        }
    }
    
    fn initialize(&mut self) {
        println!("👨‍🔧 Engineer {} ready for action!", self.name);
    }
}

#[derive(Debug, Clone)]
struct WorldConstructionSystem {
    structures: Vec<String>,
}

impl WorldConstructionSystem {
    fn new() -> Self {
        Self {
            structures: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
struct AdvancedToolsSuite {
    tools: Vec<String>,
}

impl AdvancedToolsSuite {
    fn new() -> Self {
        Self {
            tools: vec!["Precision Builder".to_string(), "Material Analyzer".to_string()],
        }
    }
    
    fn initialize(&mut self) {
        println!("🔧 Advanced tools ready: {}", self.tools.join(", "));
    }
}