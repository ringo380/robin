use nalgebra::{Vector3, Point3, UnitQuaternion, Matrix4};
use std::collections::HashMap;
use super::{CharacterState, MovementMode};

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum AnimationState {
    Idle,
    Walking,
    Running,
    Jumping,
    Flying,
    Building,
    Climbing,
    Swimming,
}

#[derive(Clone, Debug)]
pub struct AnimationClip {
    pub name: String,
    pub duration: f32,
    pub looping: bool,
    pub keyframes: Vec<Keyframe>,
    pub bone_tracks: HashMap<String, Vec<BoneKeyframe>>,
}

#[derive(Clone, Debug)]
pub struct Keyframe {
    pub time: f32,
    pub position: Point3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub scale: Vector3<f32>,
}

#[derive(Clone, Debug)]
pub struct BoneKeyframe {
    pub time: f32,
    pub position: Vector3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub scale: Vector3<f32>,
}

#[derive(Clone, Debug)]
pub struct Bone {
    pub name: String,
    pub parent: Option<String>,
    pub bind_transform: Matrix4<f32>,
    pub current_transform: Matrix4<f32>,
    pub local_transform: Matrix4<f32>,
}

#[derive(Clone)]
pub struct AnimationController {
    pub current_state: AnimationState,
    pub current_clip: Option<String>,
    pub animation_time: f32,
    pub transition_time: f32,
    pub transition_duration: f32,
    pub blend_weight: f32,
    pub previous_clip: Option<String>,
    pub state_machine: HashMap<AnimationState, Vec<AnimationTransition>>,
}

#[derive(Clone, Debug)]
pub struct AnimationTransition {
    pub to_state: AnimationState,
    pub condition: TransitionCondition,
    pub duration: f32,
    pub has_exit_time: bool,
    pub exit_time: f32,
}

#[derive(Clone, Debug)]
pub enum TransitionCondition {
    Velocity(f32),
    IsGrounded(bool),
    IsFalling(bool),
    IsBuilding(bool),
    MovementMode(MovementMode),
    Manual,
}

pub struct AnimationSystem {
    clips: HashMap<String, AnimationClip>,
    controllers: HashMap<String, AnimationController>,
    skeleton: HashMap<String, Bone>,
    bone_hierarchy: Vec<String>,
    
    // Procedural animation
    footstep_system: FootstepSystem,
    look_at_system: LookAtSystem,
    ik_system: InverseKinematicsSystem,
}

impl AnimationSystem {
    pub fn new() -> Self {
        let mut system = Self {
            clips: HashMap::new(),
            controllers: HashMap::new(),
            skeleton: HashMap::new(),
            bone_hierarchy: Vec::new(),
            footstep_system: FootstepSystem::new(),
            look_at_system: LookAtSystem::new(),
            ik_system: InverseKinematicsSystem::new(),
        };
        
        system.initialize_default_animations();
        system
    }

    fn initialize_default_animations(&mut self) {
        // Create basic animation clips
        self.create_idle_animation();
        self.create_walk_animation();
        self.create_run_animation();
        self.create_jump_animation();
        self.create_fly_animation();
        self.create_build_animation();
        
        // Setup basic skeleton
        self.create_basic_skeleton();
    }

    fn create_idle_animation(&mut self) {
        let mut keyframes = Vec::new();
        
        // Simple breathing animation
        for i in 0..30 {
            let time = i as f32 / 29.0 * 2.0; // 2 second loop
            let breath_offset = (time * std::f32::consts::PI).sin() * 0.02;
            
            keyframes.push(Keyframe {
                time,
                position: Point3::new(0.0, breath_offset, 0.0),
                rotation: UnitQuaternion::identity(),
                scale: Vector3::new(1.0, 1.0, 1.0),
            });
        }
        
        let clip = AnimationClip {
            name: "idle".to_string(),
            duration: 2.0,
            looping: true,
            keyframes,
            bone_tracks: HashMap::new(),
        };
        
        self.clips.insert("idle".to_string(), clip);
    }

    fn create_walk_animation(&mut self) {
        let mut keyframes = Vec::new();
        let duration = 1.2; // Walk cycle duration
        let steps = 24;
        
        for i in 0..steps {
            let time = i as f32 / (steps - 1) as f32 * duration;
            let cycle = time / duration * std::f32::consts::TAU;
            
            // Bob up and down
            let bob_height = (cycle * 2.0).sin().abs() * 0.03;
            
            keyframes.push(Keyframe {
                time,
                position: Point3::new(0.0, bob_height, 0.0),
                rotation: UnitQuaternion::identity(),
                scale: Vector3::new(1.0, 1.0, 1.0),
            });
        }
        
        let clip = AnimationClip {
            name: "walk".to_string(),
            duration,
            looping: true,
            keyframes,
            bone_tracks: HashMap::new(),
        };
        
        self.clips.insert("walk".to_string(), clip);
    }

    fn create_run_animation(&mut self) {
        let mut keyframes = Vec::new();
        let duration = 0.8; // Faster than walk
        let steps = 16;
        
        for i in 0..steps {
            let time = i as f32 / (steps - 1) as f32 * duration;
            let cycle = time / duration * std::f32::consts::TAU;
            
            // More pronounced bob for running
            let bob_height = (cycle * 2.0).sin().abs() * 0.06;
            
            keyframes.push(Keyframe {
                time,
                position: Point3::new(0.0, bob_height, 0.0),
                rotation: UnitQuaternion::identity(),
                scale: Vector3::new(1.0, 1.0, 1.0),
            });
        }
        
        let clip = AnimationClip {
            name: "run".to_string(),
            duration,
            looping: true,
            keyframes,
            bone_tracks: HashMap::new(),
        };
        
        self.clips.insert("run".to_string(), clip);
    }

    fn create_jump_animation(&mut self) {
        let keyframes = vec![
            Keyframe {
                time: 0.0,
                position: Point3::new(0.0, -0.1, 0.0),
                rotation: UnitQuaternion::identity(),
                scale: Vector3::new(1.0, 0.9, 1.0),
            },
            Keyframe {
                time: 0.2,
                position: Point3::new(0.0, 0.0, 0.0),
                rotation: UnitQuaternion::identity(),
                scale: Vector3::new(1.0, 1.1, 1.0),
            },
            Keyframe {
                time: 1.0,
                position: Point3::new(0.0, 0.0, 0.0),
                rotation: UnitQuaternion::identity(),
                scale: Vector3::new(1.0, 1.0, 1.0),
            },
        ];
        
        let clip = AnimationClip {
            name: "jump".to_string(),
            duration: 1.0,
            looping: false,
            keyframes,
            bone_tracks: HashMap::new(),
        };
        
        self.clips.insert("jump".to_string(), clip);
    }

    fn create_fly_animation(&mut self) {
        let mut keyframes = Vec::new();
        let duration = 3.0;
        let steps = 30;
        
        for i in 0..steps {
            let time = i as f32 / (steps - 1) as f32 * duration;
            let float_cycle = time * std::f32::consts::TAU / duration;
            
            // Gentle floating motion
            let float_offset = float_cycle.sin() * 0.05;
            let sway = (float_cycle * 0.7).sin() * 0.02;
            
            keyframes.push(Keyframe {
                time,
                position: Point3::new(sway, float_offset, 0.0),
                rotation: UnitQuaternion::from_axis_angle(&Vector3::z_axis(), sway * 0.1),
                scale: Vector3::new(1.0, 1.0, 1.0),
            });
        }
        
        let clip = AnimationClip {
            name: "fly".to_string(),
            duration,
            looping: true,
            keyframes,
            bone_tracks: HashMap::new(),
        };
        
        self.clips.insert("fly".to_string(), clip);
    }

    fn create_build_animation(&mut self) {
        let keyframes = vec![
            Keyframe {
                time: 0.0,
                position: Point3::new(0.0, 0.0, 0.0),
                rotation: UnitQuaternion::identity(),
                scale: Vector3::new(1.0, 1.0, 1.0),
            },
            Keyframe {
                time: 0.5,
                position: Point3::new(0.0, -0.05, 0.0),
                rotation: UnitQuaternion::from_axis_angle(&Vector3::x_axis(), -0.1),
                scale: Vector3::new(1.0, 1.0, 1.0),
            },
            Keyframe {
                time: 1.0,
                position: Point3::new(0.0, 0.0, 0.0),
                rotation: UnitQuaternion::identity(),
                scale: Vector3::new(1.0, 1.0, 1.0),
            },
        ];
        
        let clip = AnimationClip {
            name: "build".to_string(),
            duration: 1.0,
            looping: true,
            keyframes,
            bone_tracks: HashMap::new(),
        };
        
        self.clips.insert("build".to_string(), clip);
    }

    fn create_basic_skeleton(&mut self) {
        // Create a simple humanoid skeleton
        let bones = vec![
            ("root", None),
            ("pelvis", Some("root")),
            ("spine", Some("pelvis")),
            ("chest", Some("spine")),
            ("neck", Some("chest")),
            ("head", Some("neck")),
            ("left_shoulder", Some("chest")),
            ("left_arm", Some("left_shoulder")),
            ("left_forearm", Some("left_arm")),
            ("left_hand", Some("left_forearm")),
            ("right_shoulder", Some("chest")),
            ("right_arm", Some("right_shoulder")),
            ("right_forearm", Some("right_arm")),
            ("right_hand", Some("right_forearm")),
            ("left_thigh", Some("pelvis")),
            ("left_leg", Some("left_thigh")),
            ("left_foot", Some("left_leg")),
            ("right_thigh", Some("pelvis")),
            ("right_leg", Some("right_thigh")),
            ("right_foot", Some("right_leg")),
        ];

        for (name, parent) in bones {
            let bone = Bone {
                name: name.to_string(),
                parent: parent.map(|p| p.to_string()),
                bind_transform: Matrix4::identity(),
                current_transform: Matrix4::identity(),
                local_transform: Matrix4::identity(),
            };
            
            self.skeleton.insert(name.to_string(), bone);
            self.bone_hierarchy.push(name.to_string());
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        // Update all animation controllers
        for (_, controller) in &mut self.controllers {
            Self::update_controller_static(controller, &self.clips, delta_time);
        }
        
        // Update procedural systems
        self.footstep_system.update(delta_time);
        self.look_at_system.update(delta_time);
        self.ik_system.update(delta_time);
    }

    fn update_controller_static(controller: &mut AnimationController, clips: &HashMap<String, AnimationClip>, delta_time: f32) {
        controller.animation_time += delta_time;
        
        if controller.transition_time > 0.0 {
            controller.transition_time += delta_time;
            controller.blend_weight = controller.transition_time / controller.transition_duration;
            
            if controller.transition_time >= controller.transition_duration {
                // Transition complete
                controller.transition_time = 0.0;
                controller.previous_clip = None;
                controller.blend_weight = 1.0;
            }
        }
        
        // Handle looping
        if let Some(clip_name) = &controller.current_clip {
            if let Some(clip) = clips.get(clip_name) {
                if clip.looping && controller.animation_time >= clip.duration {
                    controller.animation_time = controller.animation_time % clip.duration;
                }
            }
        }
    }

    pub fn create_controller(&mut self, name: &str) -> &mut AnimationController {
        let mut controller = AnimationController {
            current_state: AnimationState::Idle,
            current_clip: Some("idle".to_string()),
            animation_time: 0.0,
            transition_time: 0.0,
            transition_duration: 0.3,
            blend_weight: 1.0,
            previous_clip: None,
            state_machine: HashMap::new(),
        };
        
        self.setup_state_machine(&mut controller);
        self.controllers.insert(name.to_string(), controller);
        self.controllers.get_mut(name).unwrap()
    }

    fn setup_state_machine(&self, controller: &mut AnimationController) {
        use AnimationState::*;
        
        // Define state transitions
        let transitions = vec![
            (Idle, vec![
                AnimationTransition {
                    to_state: Walking,
                    condition: TransitionCondition::Velocity(0.5),
                    duration: 0.2,
                    has_exit_time: false,
                    exit_time: 0.0,
                },
                AnimationTransition {
                    to_state: Flying,
                    condition: TransitionCondition::MovementMode(MovementMode::Fly),
                    duration: 0.3,
                    has_exit_time: false,
                    exit_time: 0.0,
                },
            ]),
            (Walking, vec![
                AnimationTransition {
                    to_state: Idle,
                    condition: TransitionCondition::Velocity(0.1),
                    duration: 0.2,
                    has_exit_time: false,
                    exit_time: 0.0,
                },
                AnimationTransition {
                    to_state: Running,
                    condition: TransitionCondition::Velocity(6.0),
                    duration: 0.1,
                    has_exit_time: false,
                    exit_time: 0.0,
                },
                AnimationTransition {
                    to_state: Jumping,
                    condition: TransitionCondition::IsGrounded(false),
                    duration: 0.1,
                    has_exit_time: false,
                    exit_time: 0.0,
                },
            ]),
        ];
        
        for (state, state_transitions) in transitions {
            controller.state_machine.insert(state, state_transitions);
        }
    }

    pub fn update_character_animation(&mut self, character_name: &str, state: &CharacterState) {
        if let Some(controller) = self.controllers.get_mut(character_name) {
            let velocity_magnitude = state.velocity.magnitude();
            
            // Determine target animation state
            let target_state = match state.movement_mode {
                MovementMode::Fly => AnimationState::Flying,
                MovementMode::Jump => AnimationState::Jumping,
                _ => {
                    if state.build_mode {
                        AnimationState::Building
                    } else if velocity_magnitude > 6.0 {
                        AnimationState::Running
                    } else if velocity_magnitude > 0.5 {
                        AnimationState::Walking
                    } else {
                        AnimationState::Idle
                    }
                }
            };
            
            // Trigger transition if needed
            if controller.current_state.clone() as u8 != target_state.clone() as u8 {
                Self::transition_to_state_static(controller, target_state);
            }
        }
    }

    fn transition_to_state(&mut self, controller: &mut AnimationController, new_state: AnimationState) {
        // Start transition
        controller.previous_clip = controller.current_clip.clone();
        controller.current_state = new_state.clone();
        controller.transition_time = 0.0;
        controller.blend_weight = 0.0;

        // Set new clip
        controller.current_clip = match new_state {
            AnimationState::Idle => Some("idle".to_string()),
            AnimationState::Walking => Some("walk".to_string()),
            AnimationState::Running => Some("run".to_string()),
            AnimationState::Jumping => Some("jump".to_string()),
            AnimationState::Flying => Some("fly".to_string()),
            AnimationState::Building => Some("build".to_string()),
            _ => Some("idle".to_string()),
        };
        
        controller.animation_time = 0.0;
    }

    fn transition_to_state_static(controller: &mut AnimationController, new_state: AnimationState) {
        // Start transition
        controller.previous_clip = controller.current_clip.clone();
        controller.current_state = new_state.clone();
        controller.transition_time = 0.0;
        controller.blend_weight = 0.0;

        // Set new clip
        controller.current_clip = match new_state {
            AnimationState::Idle => Some("idle".to_string()),
            AnimationState::Walking => Some("walk".to_string()),
            AnimationState::Running => Some("run".to_string()),
            AnimationState::Jumping => Some("jump".to_string()),
            AnimationState::Flying => Some("fly".to_string()),
            AnimationState::Building => Some("build".to_string()),
            _ => Some("idle".to_string()),
        };

        controller.animation_time = 0.0;
    }

    pub fn get_bone_transform(&self, character_name: &str, bone_name: &str) -> Option<Matrix4<f32>> {
        // In a full implementation, this would calculate the final bone transform
        // based on the current animation state and blending
        self.skeleton.get(bone_name).map(|bone| bone.current_transform)
    }
}

// Procedural animation systems
struct FootstepSystem {
    last_step_time: f32,
    step_interval: f32,
}

impl FootstepSystem {
    fn new() -> Self {
        Self {
            last_step_time: 0.0,
            step_interval: 0.5,
        }
    }
    
    fn update(&mut self, delta_time: f32) {
        self.last_step_time += delta_time;
    }
}

struct LookAtSystem {
    target_position: Option<Point3<f32>>,
    blend_speed: f32,
}

impl LookAtSystem {
    fn new() -> Self {
        Self {
            target_position: None,
            blend_speed: 2.0,
        }
    }
    
    fn update(&mut self, _delta_time: f32) {
        // Update look-at blending
    }
}

struct InverseKinematicsSystem {
    ik_chains: HashMap<String, IKChain>,
}

#[derive(Clone)]
struct IKChain {
    bones: Vec<String>,
    target_position: Point3<f32>,
    iterations: u32,
}

impl InverseKinematicsSystem {
    fn new() -> Self {
        Self {
            ik_chains: HashMap::new(),
        }
    }
    
    fn update(&mut self, _delta_time: f32) {
        // Update IK solving
    }
}