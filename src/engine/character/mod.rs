use nalgebra::{Vector3, Point3, Matrix4, Quaternion, UnitQuaternion};
use std::collections::HashMap;

pub mod engineer_controller;
pub mod character_physics;
pub mod animation_system;
pub mod camera_controller;

pub use engineer_controller::EngineerController;
pub use character_physics::CharacterPhysics;
pub use animation_system::AnimationSystem;
pub use camera_controller::CameraController;

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

pub struct CharacterSystem {
    pub engineers: HashMap<String, EngineerController>,
    pub physics: CharacterPhysics,
    pub animation: AnimationSystem,
    pub camera: CameraController,
}

impl CharacterSystem {
    pub fn new() -> Self {
        Self {
            engineers: HashMap::new(),
            physics: CharacterPhysics::new(),
            animation: AnimationSystem::new(),
            camera: CameraController::new(),
        }
    }

    pub fn create_engineer(&mut self, name: &str) -> Result<(), String> {
        if self.engineers.contains_key(name) {
            return Err(format!("Engineer '{}' already exists", name));
        }

        let engineer = EngineerController::new(name.to_string());
        self.engineers.insert(name.to_string(), engineer);
        Ok(())
    }

    pub fn update(&mut self, delta_time: f32) {
        for (_, engineer) in &mut self.engineers {
            engineer.update(delta_time, &mut self.physics);
        }
        
        self.animation.update(delta_time);
        self.camera.update(delta_time);
    }

    pub fn get_engineer(&self, name: &str) -> Option<&EngineerController> {
        self.engineers.get(name)
    }

    pub fn get_engineer_mut(&mut self, name: &str) -> Option<&mut EngineerController> {
        self.engineers.get_mut(name)
    }
}