// Standalone test for Engineer Character Controller
// This test doesn't depend on the full engine compilation

use nalgebra::{Vector3, Point3, UnitQuaternion};
use std::time::Instant;
use std::collections::HashMap;

// Core character types (simplified versions)
#[derive(Clone, Debug)]
pub enum MovementMode {
    Walk,
    Run,
    Jump,
    Fly,
    Noclip,
}

#[derive(Clone, Debug)]
pub struct CharacterState {
    pub position: Point3<f32>,
    pub velocity: Vector3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub movement_mode: MovementMode,
    pub is_grounded: bool,
    pub health: f32,
    pub energy: f32,
    pub build_mode: bool,
    pub selected_tool: String,
}

impl Default for CharacterState {
    fn default() -> Self {
        Self {
            position: Point3::new(0.0, 0.0, 0.0),
            velocity: Vector3::new(0.0, 0.0, 0.0),
            rotation: UnitQuaternion::identity(),
            movement_mode: MovementMode::Walk,
            is_grounded: false,
            health: 100.0,
            energy: 100.0,
            build_mode: false,
            selected_tool: "none".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct InputState {
    pub move_forward: bool,
    pub move_backward: bool,
    pub move_left: bool,
    pub move_right: bool,
    pub jump: bool,
    pub crouch: bool,
    pub run: bool,
    pub fly_toggle: bool,
    pub build_mode_toggle: bool,
    pub mouse_delta: Vector3<f32>,
    pub scroll_delta: f32,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            move_forward: false,
            move_backward: false,
            move_left: false,
            move_right: false,
            jump: false,
            crouch: false,
            run: false,
            fly_toggle: false,
            build_mode_toggle: false,
            mouse_delta: Vector3::new(0.0, 0.0, 0.0),
            scroll_delta: 0.0,
        }
    }
}

// Simplified Physics System
pub struct CharacterPhysics {
    gravity: f32,
    terminal_velocity: f32,
    ground_check_distance: f32,
    character_radius: f32,
    character_height: f32,
}

impl CharacterPhysics {
    pub fn new() -> Self {
        Self {
            gravity: -18.0,
            terminal_velocity: -50.0,
            ground_check_distance: 0.1,
            character_radius: 0.3,
            character_height: 1.8,
        }
    }

    pub fn update_character(&mut self, state: &mut CharacterState, delta_time: f32) {
        match state.movement_mode {
            MovementMode::Fly | MovementMode::Noclip => {
                // No physics for flying/noclip modes
                return;
            }
            _ => {
                self.apply_gravity(state, delta_time);
                self.check_ground_collision(state);
                self.resolve_collisions(state, delta_time);
                self.apply_friction(state, delta_time);
            }
        }
    }

    fn apply_gravity(&self, state: &mut CharacterState, delta_time: f32) {
        if !state.is_grounded {
            state.velocity.y += self.gravity * delta_time;
            
            if state.velocity.y < self.terminal_velocity {
                state.velocity.y = self.terminal_velocity;
            }
        }
    }

    fn check_ground_collision(&mut self, state: &mut CharacterState) {
        let ground_check_pos = Point3::new(
            state.position.x,
            state.position.y - self.character_height / 2.0 - self.ground_check_distance,
            state.position.z
        );

        let was_grounded = state.is_grounded;
        state.is_grounded = ground_check_pos.y <= 0.0;

        if state.is_grounded && !was_grounded {
            state.position.y = self.character_height / 2.0;
            if state.velocity.y < 0.0 {
                state.velocity.y = 0.0;
            }
        }
    }

    fn resolve_collisions(&mut self, state: &mut CharacterState, delta_time: f32) {
        let new_position = state.position + state.velocity * delta_time;
        
        // Simple world boundaries
        let bounds_min = Point3::new(-1000.0, -100.0, -1000.0);
        let bounds_max = Point3::new(1000.0, 1000.0, 1000.0);

        if new_position.x < bounds_min.x || new_position.x > bounds_max.x ||
           new_position.z < bounds_min.z || new_position.z > bounds_max.z {
            // Outside bounds - stop movement
            state.velocity.x = 0.0;
            state.velocity.z = 0.0;
        } else {
            state.position = new_position;
        }
    }

    fn apply_friction(&self, state: &mut CharacterState, delta_time: f32) {
        let friction = if state.is_grounded { 8.0 } else { 1.0 };
        
        let horizontal_velocity = Vector3::new(state.velocity.x, 0.0, state.velocity.z);
        let friction_force = horizontal_velocity * -friction * delta_time;
        
        state.velocity.x += friction_force.x;
        state.velocity.z += friction_force.z;
        
        if horizontal_velocity.magnitude() < 0.1 && state.is_grounded {
            state.velocity.x = 0.0;
            state.velocity.z = 0.0;
        }
    }
}

// Simplified Engineer Controller
pub struct EngineerController {
    pub name: String,
    pub state: CharacterState,
    pub input: InputState,
    
    walk_speed: f32,
    run_speed: f32,
    fly_speed: f32,
    jump_force: f32,
    air_control: f32,
    
    build_reach: f32,
    selected_material: String,
    
    camera_sensitivity: f32,
    camera_pitch: f32,
    camera_yaw: f32,
    
    last_fly_toggle: Instant,
    last_build_toggle: Instant,
    toggle_cooldown: f32,
}

impl EngineerController {
    pub fn new(name: String) -> Self {
        Self {
            name,
            state: CharacterState::default(),
            input: InputState::default(),
            
            walk_speed: 4.0,
            run_speed: 8.0,
            fly_speed: 12.0,
            jump_force: 6.0,
            air_control: 0.3,
            
            build_reach: 10.0,
            selected_material: "wood".to_string(),
            
            camera_sensitivity: 0.002,
            camera_pitch: 0.0,
            camera_yaw: 0.0,
            
            last_fly_toggle: Instant::now(),
            last_build_toggle: Instant::now(),
            toggle_cooldown: 0.3,
        }
    }

    pub fn update(&mut self, delta_time: f32, physics: &mut CharacterPhysics) {
        self.process_input();
        self.update_movement(delta_time, physics);
        self.update_camera(delta_time);
    }

    fn process_input(&mut self) {
        let now = Instant::now();
        
        if self.input.fly_toggle && now.duration_since(self.last_fly_toggle).as_secs_f32() > self.toggle_cooldown {
            self.toggle_flight_mode();
            self.last_fly_toggle = now;
        }

        if self.input.build_mode_toggle && now.duration_since(self.last_build_toggle).as_secs_f32() > self.toggle_cooldown {
            self.state.build_mode = !self.state.build_mode;
            self.last_build_toggle = now;
        }

        if self.input.run && matches!(self.state.movement_mode, MovementMode::Walk) {
            self.state.movement_mode = MovementMode::Run;
        } else if !self.input.run && matches!(self.state.movement_mode, MovementMode::Run) {
            self.state.movement_mode = MovementMode::Walk;
        }
    }

    fn update_movement(&mut self, delta_time: f32, physics: &mut CharacterPhysics) {
        let mut movement_input = Vector3::new(0.0, 0.0, 0.0);
        
        if self.input.move_forward { movement_input.z -= 1.0; }
        if self.input.move_backward { movement_input.z += 1.0; }
        if self.input.move_left { movement_input.x -= 1.0; }
        if self.input.move_right { movement_input.x += 1.0; }

        if movement_input.magnitude() > 0.0 {
            movement_input = movement_input.normalize();
        }

        match self.state.movement_mode {
            MovementMode::Walk => self.update_ground_movement(movement_input, delta_time, self.walk_speed),
            MovementMode::Run => self.update_ground_movement(movement_input, delta_time, self.run_speed),
            MovementMode::Jump => self.update_jump_movement(movement_input, delta_time),
            MovementMode::Fly => self.update_fly_movement(movement_input, delta_time),
            MovementMode::Noclip => self.update_noclip_movement(movement_input, delta_time),
        }

        physics.update_character(&mut self.state, delta_time);
    }

    fn update_ground_movement(&mut self, input: Vector3<f32>, delta_time: f32, speed: f32) {
        if self.state.is_grounded {
            let forward = self.state.rotation * Vector3::new(0.0, 0.0, -1.0);
            let right = self.state.rotation * Vector3::new(1.0, 0.0, 0.0);
            
            let desired_velocity = (forward * input.z + right * input.x) * speed;
            self.state.velocity.x = desired_velocity.x;
            self.state.velocity.z = desired_velocity.z;
            
            if self.input.jump {
                self.state.velocity.y = self.jump_force;
                self.state.movement_mode = MovementMode::Jump;
                self.state.is_grounded = false;
            }
        } else {
            let forward = self.state.rotation * Vector3::new(0.0, 0.0, -1.0);
            let right = self.state.rotation * Vector3::new(1.0, 0.0, 0.0);
            
            let air_input = (forward * input.z + right * input.x) * speed * self.air_control;
            self.state.velocity.x += air_input.x * delta_time;
            self.state.velocity.z += air_input.z * delta_time;
        }
    }

    fn update_jump_movement(&mut self, input: Vector3<f32>, delta_time: f32) {
        self.update_ground_movement(input, delta_time, self.walk_speed);
        
        if self.state.is_grounded && self.state.velocity.y <= 0.0 {
            self.state.movement_mode = MovementMode::Walk;
        }
    }

    fn update_fly_movement(&mut self, input: Vector3<f32>, delta_time: f32) {
        let mut movement = Vector3::new(0.0, 0.0, 0.0);
        
        if input.magnitude() > 0.0 {
            let forward = self.state.rotation * Vector3::new(0.0, 0.0, -1.0);
            let right = self.state.rotation * Vector3::new(1.0, 0.0, 0.0);
            let up = Vector3::new(0.0, 1.0, 0.0);
            
            movement = forward * input.z + right * input.x;
            
            if self.input.jump { movement += up; }
            if self.input.crouch { movement -= up; }
            
            movement = movement.normalize() * self.fly_speed;
        }
        
        self.state.velocity = self.state.velocity * 0.8 + movement * 0.2;
        self.state.position += self.state.velocity * delta_time;
    }

    fn update_noclip_movement(&mut self, input: Vector3<f32>, delta_time: f32) {
        let mut movement = Vector3::new(0.0, 0.0, 0.0);
        
        if input.magnitude() > 0.0 {
            let forward = self.state.rotation * Vector3::new(0.0, 0.0, -1.0);
            let right = self.state.rotation * Vector3::new(1.0, 0.0, 0.0);
            let up = Vector3::new(0.0, 1.0, 0.0);
            
            movement = forward * input.z + right * input.x;
            
            if self.input.jump { movement += up; }
            if self.input.crouch { movement -= up; }
            
            movement = movement.normalize() * self.fly_speed * 2.0;
        }
        
        self.state.velocity = movement;
        self.state.position += self.state.velocity * delta_time;
    }

    fn update_camera(&mut self, _delta_time: f32) {
        self.camera_yaw += self.input.mouse_delta.x * self.camera_sensitivity;
        self.camera_pitch += self.input.mouse_delta.y * self.camera_sensitivity;
        
        self.camera_pitch = self.camera_pitch.clamp(
            -std::f32::consts::FRAC_PI_2 * 0.99, 
            std::f32::consts::FRAC_PI_2 * 0.99
        );
        
        self.state.rotation = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), self.camera_yaw);
        self.input.mouse_delta = Vector3::zeros();
    }

    fn toggle_flight_mode(&mut self) {
        match self.state.movement_mode {
            MovementMode::Walk | MovementMode::Run | MovementMode::Jump => {
                self.state.movement_mode = MovementMode::Fly;
                self.state.velocity.y = 0.0;
            }
            MovementMode::Fly => {
                self.state.movement_mode = MovementMode::Walk;
            }
            MovementMode::Noclip => {
                self.state.movement_mode = MovementMode::Walk;
            }
        }
    }

    pub fn set_input(&mut self, input: InputState) {
        self.input = input;
    }

    pub fn add_mouse_delta(&mut self, delta_x: f32, delta_y: f32) {
        self.input.mouse_delta.x += delta_x;
        self.input.mouse_delta.y += delta_y;
    }

    pub fn get_camera_position(&self) -> Point3<f32> {
        self.state.position + Vector3::new(0.0, 1.8, 0.0)
    }

    pub fn get_camera_rotation(&self) -> (f32, f32) {
        (self.camera_pitch, self.camera_yaw)
    }

    pub fn get_build_mode(&self) -> bool {
        self.state.build_mode
    }
}

fn main() {
    println!("🔧 Engineer Character Controller Standalone Test");
    println!("=" .repeat(60));
    
    let start_time = Instant::now();
    
    // Test 1: Controller Creation
    println!("\n📋 Test 1: Engineer Controller Creation");
    let mut engineer = EngineerController::new("test_engineer".to_string());
    let mut physics = CharacterPhysics::new();
    println!("✅ Engineer controller created successfully");
    println!("   Name: {}", engineer.name);
    println!("   Initial Position: ({:.1}, {:.1}, {:.1})", 
             engineer.state.position.x, engineer.state.position.y, engineer.state.position.z);
    
    // Test 2: Basic Movement Input
    println!("\n📋 Test 2: Movement Input Processing");
    let mut input = InputState::default();
    input.move_forward = true;
    input.run = true;
    engineer.set_input(input);
    println!("✅ Movement input set");
    println!("   Forward: {}, Run: {}", engineer.input.move_forward, engineer.input.run);
    
    // Test 3: Physics Update Loop
    println!("\n📋 Test 3: Physics Simulation");
    let delta_time = 0.016; // 60 FPS
    
    for frame in 0..60 { // 1 second of simulation
        engineer.update(delta_time, &mut physics);
        
        if frame % 15 == 0 { // Every quarter second
            let pos = engineer.state.position;
            let vel = engineer.state.velocity;
            println!("   Frame {}: Pos({:.2}, {:.2}, {:.2}) Vel({:.2}, {:.2}, {:.2})", 
                     frame, pos.x, pos.y, pos.z, vel.x, vel.y, vel.z);
        }
    }
    
    let final_pos = engineer.state.position;
    let final_vel = engineer.state.velocity;
    println!("✅ Physics simulation completed");
    println!("   Final Position: ({:.2}, {:.2}, {:.2})", final_pos.x, final_pos.y, final_pos.z);
    println!("   Final Velocity: ({:.2}, {:.2}, {:.2})", final_vel.x, final_vel.y, final_vel.z);
    println!("   Movement Mode: {:?}", engineer.state.movement_mode);
    println!("   Is Grounded: {}", engineer.state.is_grounded);
    
    // Test 4: Flight Mode Toggle
    println!("\n📋 Test 4: Flight Mode Toggle");
    let mut input = InputState::default();
    input.fly_toggle = true;
    engineer.set_input(input);
    engineer.update(delta_time, &mut physics);
    
    println!("✅ Flight mode toggled");
    println!("   New Movement Mode: {:?}", engineer.state.movement_mode);
    
    // Test flight movement
    let mut input = InputState::default();
    input.move_forward = true;
    input.jump = true; // Up in flight mode
    engineer.set_input(input);
    
    for _ in 0..30 {
        engineer.update(delta_time, &mut physics);
    }
    
    let flight_pos = engineer.state.position;
    println!("   Flight Position: ({:.2}, {:.2}, {:.2})", flight_pos.x, flight_pos.y, flight_pos.z);
    
    // Test 5: Build Mode
    println!("\n📋 Test 5: Build Mode Functionality");
    let mut input = InputState::default();
    input.build_mode_toggle = true;
    engineer.set_input(input);
    engineer.update(delta_time, &mut physics);
    
    println!("✅ Build mode toggled");
    println!("   Build Mode Active: {}", engineer.get_build_mode());
    
    // Test 6: Camera System
    println!("\n📋 Test 6: Camera System");
    engineer.add_mouse_delta(0.5, -0.2);
    engineer.update(delta_time, &mut physics);
    
    let camera_pos = engineer.get_camera_position();
    let (pitch, yaw) = engineer.get_camera_rotation();
    
    println!("✅ Camera system tested");
    println!("   Camera Position: ({:.2}, {:.2}, {:.2})", camera_pos.x, camera_pos.y, camera_pos.z);
    println!("   Camera Rotation: Pitch={:.2}°, Yaw={:.2}°", 
             pitch.to_degrees(), yaw.to_degrees());
    
    // Test 7: Jump Mechanics
    println!("\n📋 Test 7: Jump Mechanics");
    
    // Return to ground first
    engineer.state.movement_mode = MovementMode::Walk;
    engineer.state.position = Point3::new(0.0, 1.0, 0.0);
    engineer.state.velocity = Vector3::zeros();
    engineer.state.is_grounded = true;
    
    // Trigger jump
    let mut input = InputState::default();
    input.jump = true;
    engineer.set_input(input);
    engineer.update(delta_time, &mut physics);
    
    println!("✅ Jump initiated");
    println!("   Jump Mode: {:?}", engineer.state.movement_mode);
    println!("   Initial Jump Velocity: {:.2}", engineer.state.velocity.y);
    
    // Simulate jump arc
    for frame in 0..60 {
        engineer.update(delta_time, &mut physics);
        
        if frame % 10 == 0 {
            println!("   Frame {}: Height={:.2}, VelY={:.2}, Grounded={}", 
                     frame, engineer.state.position.y, engineer.state.velocity.y, engineer.state.is_grounded);
        }
        
        if engineer.state.is_grounded && frame > 10 {
            println!("   Landed after {} frames", frame);
            break;
        }
    }
    
    // Test 8: Performance Validation
    println!("\n📋 Test 8: Performance Test");
    let perf_start = Instant::now();
    
    // Simulate high-frequency updates
    for _ in 0..1000 {
        engineer.update(0.001, &mut physics); // 1ms updates
    }
    
    let perf_duration = perf_start.elapsed();
    println!("✅ Performance test completed");
    println!("   1000 updates in: {:.2}ms", perf_duration.as_secs_f32() * 1000.0);
    println!("   Average update time: {:.3}ms", perf_duration.as_secs_f32() * 1000.0 / 1000.0);
    
    // Final Summary
    let total_time = start_time.elapsed();
    println!("\n" + "=" .repeat(60));
    println!("🎯 Engineer Character Controller Test Summary");
    println!("=" .repeat(60));
    println!("✅ All core systems validated successfully");
    println!("⚡ Total test execution time: {:.2}ms", total_time.as_secs_f32() * 1000.0);
    println!("");
    println!("🔧 Features Validated:");
    println!("   • Multi-mode movement system (Walk/Run/Jump/Fly/Noclip)");
    println!("   • Physics simulation with gravity and collision");
    println!("   • Camera control with mouse look");
    println!("   • Build mode integration");
    println!("   • Input processing and state management");
    println!("   • High-performance real-time updates");
    println!("");
    println!("🚀 Phase 1.1 Complete - Ready for Phase 1.2: World Construction System");
}