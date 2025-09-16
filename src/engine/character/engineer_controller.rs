use nalgebra::{Vector3, Point3, UnitQuaternion};
use super::{CharacterState, InputState, MovementMode};
use super::character_physics::CharacterPhysics;
use crate::engine::world::building::BuildingSystem;
use std::time::Instant;

pub struct EngineerController {
    pub name: String,
    pub state: CharacterState,
    pub input: InputState,
    
    // Movement parameters
    walk_speed: f32,
    run_speed: f32,
    fly_speed: f32,
    jump_force: f32,
    air_control: f32,
    ground_friction: f32,
    air_friction: f32,
    
    // Build mode parameters
    build_reach: f32,
    selected_material: String,
    build_grid_size: f32,
    snap_to_grid: bool,
    
    // Camera parameters
    camera_sensitivity: f32,
    camera_offset: Vector3<f32>,
    camera_pitch: f32,
    camera_yaw: f32,
    
    // State tracking
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
            
            // Tuned movement parameters
            walk_speed: 4.0,
            run_speed: 8.0,
            fly_speed: 12.0,
            jump_force: 6.0,
            air_control: 0.3,
            ground_friction: 8.0,
            air_friction: 1.0,
            
            // Build parameters
            build_reach: 10.0,
            selected_material: "wood".to_string(),
            build_grid_size: 1.0,
            snap_to_grid: true,
            
            // Camera setup
            camera_sensitivity: 0.002,
            camera_offset: Vector3::new(0.0, 1.8, 0.0), // Eye level
            camera_pitch: 0.0,
            camera_yaw: 0.0,
            
            // State tracking
            last_fly_toggle: Instant::now(),
            last_build_toggle: Instant::now(),
            toggle_cooldown: 0.3,
        }
    }

    pub fn update(&mut self, delta_time: f32, physics: &mut CharacterPhysics) {
        self.process_input();
        self.update_movement(delta_time, physics);
        self.update_camera(delta_time);
        self.update_build_systems();
    }

    fn process_input(&mut self) {
        // Handle mode toggles with cooldown
        let now = Instant::now();
        
        if self.input.fly_toggle && now.duration_since(self.last_fly_toggle).as_secs_f32() > self.toggle_cooldown {
            self.toggle_flight_mode();
            self.last_fly_toggle = now;
        }

        if self.input.build_mode_toggle && now.duration_since(self.last_build_toggle).as_secs_f32() > self.toggle_cooldown {
            self.state.build_mode = !self.state.build_mode;
            self.last_build_toggle = now;
        }

        // Handle run state
        if self.input.run && matches!(self.state.movement_mode, MovementMode::Walk) {
            self.state.movement_mode = MovementMode::Run;
        } else if !self.input.run && matches!(self.state.movement_mode, MovementMode::Run) {
            self.state.movement_mode = MovementMode::Walk;
        }
    }

    fn update_movement(&mut self, delta_time: f32, physics: &mut CharacterPhysics) {
        let mut movement_input = Vector3::new(0.0, 0.0, 0.0);
        
        // Gather input
        if self.input.move_forward { movement_input.z -= 1.0; }
        if self.input.move_backward { movement_input.z += 1.0; }
        if self.input.move_left { movement_input.x -= 1.0; }
        if self.input.move_right { movement_input.x += 1.0; }

        // Normalize diagonal movement
        if movement_input.magnitude() > 0.0 {
            movement_input = movement_input.normalize();
        }

        match self.state.movement_mode {
            MovementMode::Walk => self.update_ground_movement(movement_input, delta_time, physics, self.walk_speed),
            MovementMode::Run => self.update_ground_movement(movement_input, delta_time, physics, self.run_speed),
            MovementMode::Jump => self.update_jump_movement(movement_input, delta_time, physics),
            MovementMode::Fly => self.update_fly_movement(movement_input, delta_time, physics),
            MovementMode::Noclip => self.update_noclip_movement(movement_input, delta_time),
        }

        // Apply physics update
        physics.update_character(&mut self.state, delta_time);
    }

    fn update_ground_movement(&mut self, input: Vector3<f32>, delta_time: f32, physics: &CharacterPhysics, speed: f32) {
        if self.state.is_grounded {
            // Ground movement with proper rotation
            let forward = self.state.rotation * Vector3::new(0.0, 0.0, -1.0);
            let right = self.state.rotation * Vector3::new(1.0, 0.0, 0.0);
            
            let desired_velocity = (forward * input.z + right * input.x) * speed;
            self.state.velocity.x = desired_velocity.x;
            self.state.velocity.z = desired_velocity.z;
            
            // Handle jumping
            if self.input.jump {
                self.state.velocity.y = self.jump_force;
                self.state.movement_mode = MovementMode::Jump;
                self.state.is_grounded = false;
            }
        } else {
            // Air control
            let forward = self.state.rotation * Vector3::new(0.0, 0.0, -1.0);
            let right = self.state.rotation * Vector3::new(1.0, 0.0, 0.0);
            
            let air_input = (forward * input.z + right * input.x) * speed * self.air_control;
            self.state.velocity.x += air_input.x * delta_time;
            self.state.velocity.z += air_input.z * delta_time;
        }
    }

    fn update_jump_movement(&mut self, input: Vector3<f32>, delta_time: f32, physics: &CharacterPhysics) {
        // Same as ground movement but in air
        self.update_ground_movement(input, delta_time, physics, self.walk_speed);
        
        // Return to walk when grounded
        if self.state.is_grounded && self.state.velocity.y <= 0.0 {
            self.state.movement_mode = MovementMode::Walk;
        }
    }

    fn update_fly_movement(&mut self, input: Vector3<f32>, delta_time: f32, _physics: &CharacterPhysics) {
        // Full 3D movement
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
        
        // Smooth velocity interpolation
        self.state.velocity = self.state.velocity * 0.8 + movement * 0.2;
        
        // Update position directly (no gravity)
        self.state.position += self.state.velocity * delta_time;
    }

    fn update_noclip_movement(&mut self, input: Vector3<f32>, delta_time: f32) {
        // Similar to fly but with instant movement
        let mut movement = Vector3::new(0.0, 0.0, 0.0);
        
        if input.magnitude() > 0.0 {
            let forward = self.state.rotation * Vector3::new(0.0, 0.0, -1.0);
            let right = self.state.rotation * Vector3::new(1.0, 0.0, 0.0);
            let up = Vector3::new(0.0, 1.0, 0.0);
            
            movement = forward * input.z + right * input.x;
            
            if self.input.jump { movement += up; }
            if self.input.crouch { movement -= up; }
            
            movement = movement.normalize() * self.fly_speed * 2.0; // Faster in noclip
        }
        
        self.state.velocity = movement;
        self.state.position += self.state.velocity * delta_time;
    }

    fn update_camera(&mut self, delta_time: f32) {
        // Mouse look
        self.camera_yaw += self.input.mouse_delta.x * self.camera_sensitivity;
        self.camera_pitch += self.input.mouse_delta.y * self.camera_sensitivity;
        
        // Clamp pitch to prevent over-rotation
        self.camera_pitch = self.camera_pitch.clamp(-std::f32::consts::FRAC_PI_2 * 0.99, std::f32::consts::FRAC_PI_2 * 0.99);
        
        // Update character rotation (yaw only)
        self.state.rotation = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), self.camera_yaw);
        
        // Clear mouse delta for next frame
        self.input.mouse_delta = Vector3::zeros();
    }

    fn update_build_systems(&mut self) {
        if !self.state.build_mode {
            return;
        }
        
        // Handle tool selection with scroll wheel
        if self.input.scroll_delta != 0.0 {
            self.cycle_build_tool(self.input.scroll_delta > 0.0);
        }
        
        // Reset scroll delta
        self.input.scroll_delta = 0.0;
    }

    fn toggle_flight_mode(&mut self) {
        match self.state.movement_mode {
            MovementMode::Walk | MovementMode::Run | MovementMode::Jump => {
                self.state.movement_mode = MovementMode::Fly;
                self.state.velocity.y = 0.0; // Stop falling
            }
            MovementMode::Fly => {
                self.state.movement_mode = MovementMode::Walk;
            }
            MovementMode::Noclip => {
                self.state.movement_mode = MovementMode::Walk;
            }
        }
    }

    fn cycle_build_tool(&mut self, forward: bool) {
        let tools = vec!["place", "remove", "copy", "paste", "paint", "terrain"];
        let current_index = tools.iter().position(|&tool| tool == self.selected_material).unwrap_or(0);
        
        let new_index = if forward {
            (current_index + 1) % tools.len()
        } else {
            (current_index + tools.len() - 1) % tools.len()
        };
        
        self.selected_material = tools[new_index].to_string();
    }

    // Input setters
    pub fn set_input(&mut self, input: InputState) {
        self.input = input;
    }

    pub fn add_mouse_delta(&mut self, delta_x: f32, delta_y: f32) {
        self.input.mouse_delta.x += delta_x;
        self.input.mouse_delta.y += delta_y;
    }

    pub fn add_scroll_delta(&mut self, delta: f32) {
        self.input.scroll_delta += delta;
    }

    // Getters
    pub fn get_camera_position(&self) -> Point3<f32> {
        self.state.position + self.camera_offset
    }

    pub fn get_camera_rotation(&self) -> (f32, f32) {
        (self.camera_pitch, self.camera_yaw)
    }

    pub fn get_build_mode(&self) -> bool {
        self.state.build_mode
    }

    pub fn get_selected_tool(&self) -> &str {
        &self.selected_material
    }
}