/*!
 * Interactive Components - Building Blocks for Game Mechanics
 *
 * This module provides a library of interactive elements that users can
 * place in their world to create game mechanics. Each component has
 * properties that can be configured and logic connections for behavior.
 */

use crate::engine::{
    math::{Vec3, Vec2},
    graphics::{Color, Mesh},
    input::InputManager,
    error::{RobinResult, RobinError},
};
use cgmath::InnerSpace;
use super::{LogicValue, LogicNode, LogicNodeType};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Library of all available interactive components
#[derive(Debug)]
pub struct ComponentLibrary {
    /// Available component types
    components: Vec<ComponentType>,

    /// Active components in the world
    active_components: HashMap<u32, InteractiveComponent>,

    /// Next component ID
    next_component_id: u32,
}

impl ComponentLibrary {
    pub fn new() -> Self {
        let components = vec![
            ComponentType::Mechanical(MechanicalType::Door),
            ComponentType::Mechanical(MechanicalType::Platform),
            ComponentType::Mechanical(MechanicalType::Elevator),
            ComponentType::Mechanical(MechanicalType::ConveyorBelt),
            ComponentType::Trigger(TriggerType::PressurePlate),
            ComponentType::Trigger(TriggerType::LaserTripwire),
            ComponentType::Trigger(TriggerType::MotionDetector),
            ComponentType::Trigger(TriggerType::SoundDetector),
            ComponentType::GameLogic(GameLogicType::SpawnPoint),
            ComponentType::GameLogic(GameLogicType::Checkpoint),
            ComponentType::GameLogic(GameLogicType::Collectible),
            ComponentType::GameLogic(GameLogicType::Hazard),
            ComponentType::Utility(UtilityType::Light),
            ComponentType::Utility(UtilityType::Camera),
            ComponentType::Utility(UtilityType::AudioSource),
        ];

        Self {
            components,
            active_components: HashMap::new(),
            next_component_id: 1,
        }
    }

    /// Get all available component types
    pub fn get_available_components(&self) -> &[ComponentType] {
        &self.components
    }

    /// Create a new component instance
    pub fn create_component(
        &mut self,
        component_type: ComponentType,
        position: Vec3,
    ) -> RobinResult<u32> {
        let component_id = self.next_component_id;
        self.next_component_id += 1;

        let component = InteractiveComponent::new(component_id, component_type, position)?;
        self.active_components.insert(component_id, component);

        log::debug!("Created component {} at {:?}", component_id, position);
        Ok(component_id)
    }

    /// Update all active components
    pub fn update(&mut self, delta_time: f32, input: &InputManager) -> RobinResult<()> {
        for component in self.active_components.values_mut() {
            component.update(delta_time, input)?;
        }
        Ok(())
    }

    /// Get a component by ID
    pub fn get_component(&self, component_id: u32) -> Option<&InteractiveComponent> {
        self.active_components.get(&component_id)
    }

    /// Get a mutable component by ID
    pub fn get_component_mut(&mut self, component_id: u32) -> Option<&mut InteractiveComponent> {
        self.active_components.get_mut(&component_id)
    }

    /// Remove a component
    pub fn remove_component(&mut self, component_id: u32) {
        self.active_components.remove(&component_id);
        log::debug!("Removed component {}", component_id);
    }

    /// Get all active components
    pub fn get_active_components(&self) -> &HashMap<u32, InteractiveComponent> {
        &self.active_components
    }
}

/// Categories of interactive components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentType {
    /// Mechanical components that move or operate
    Mechanical(MechanicalType),
    /// Triggers that detect events
    Trigger(TriggerType),
    /// Game logic components
    GameLogic(GameLogicType),
    /// Utility components
    Utility(UtilityType),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MechanicalType {
    Door,
    Platform,
    Elevator,
    ConveyorBelt,
    Piston,
    Crusher,
    RotatingPlatform,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerType {
    PressurePlate,
    LaserTripwire,
    MotionDetector,
    SoundDetector,
    LightSensor,
    ProximityTrigger,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameLogicType {
    SpawnPoint,
    Checkpoint,
    Collectible,
    Hazard,
    GoalZone,
    Teleporter,
    InventoryChest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UtilityType {
    Light,
    Camera,
    AudioSource,
    ParticleEmitter,
    InfoDisplay,
}

/// A single interactive component instance
#[derive(Debug, Clone)]
pub struct InteractiveComponent {
    pub id: u32,
    pub component_type: ComponentType,
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
    pub properties: ComponentProperties,
    pub state: ComponentState,
    pub logic_connections: Vec<u32>, // Connected logic node IDs
}

impl InteractiveComponent {
    pub fn new(id: u32, component_type: ComponentType, position: Vec3) -> RobinResult<Self> {
        let properties = ComponentProperties::default_for_type(&component_type);

        Ok(Self {
            id,
            component_type,
            position,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            scale: Vec3::new(1.0, 1.0, 1.0),
            properties,
            state: ComponentState::Idle,
            logic_connections: Vec::new(),
        })
    }

    pub fn update(&mut self, delta_time: f32, _input: &InputManager) -> RobinResult<()> {
        let component_type = self.component_type.clone();
        match component_type {
            ComponentType::Mechanical(mech_type) => {
                self.update_mechanical_component(&mech_type, delta_time)?;
            }
            ComponentType::Trigger(trigger_type) => {
                self.update_trigger_component(&trigger_type, delta_time)?;
            }
            ComponentType::GameLogic(logic_type) => {
                self.update_game_logic_component(&logic_type, delta_time)?;
            }
            ComponentType::Utility(utility_type) => {
                self.update_utility_component(&utility_type, delta_time)?;
            }
        }

        Ok(())
    }

    fn update_mechanical_component(&mut self, mech_type: &MechanicalType, delta_time: f32) -> RobinResult<()> {
        match mech_type {
            MechanicalType::Door => {
                if let ComponentState::Moving { target_position, speed } = &self.state {
                    let current_pos = self.properties.get_vector3("current_position")
                        .unwrap_or([self.position.x, self.position.y, self.position.z]);
                    let target = *target_position;
                    let movement_speed = *speed;

                    // Move towards target
                    let direction = Vec3::new(target[0] - current_pos[0], target[1] - current_pos[1], target[2] - current_pos[2]);
                    let distance = direction.magnitude();

                    if distance < 0.1 {
                        // Reached target
                        self.properties.set_vector3("current_position", target);
                        self.state = ComponentState::Idle;
                        log::debug!("Door {} reached target position", self.id);
                    } else {
                        // Continue moving
                        let normalized_direction = direction.normalize();
                        let movement = normalized_direction * movement_speed * delta_time;
                        let new_position = [
                            current_pos[0] + movement.x,
                            current_pos[1] + movement.y,
                            current_pos[2] + movement.z,
                        ];
                        self.properties.set_vector3("current_position", new_position);
                    }
                }
            }
            MechanicalType::Platform => {
                // TODO: Implement platform movement logic
            }
            MechanicalType::Elevator => {
                // TODO: Implement elevator logic
            }
            MechanicalType::ConveyorBelt => {
                // TODO: Implement conveyor belt logic
            }
            _ => {}
        }

        Ok(())
    }

    fn update_trigger_component(&mut self, trigger_type: &TriggerType, delta_time: f32) -> RobinResult<()> {
        match trigger_type {
            TriggerType::PressurePlate => {
                // TODO: Check for objects on pressure plate
                let was_triggered = self.properties.get_bool("triggered").unwrap_or(false);
                let is_triggered = false; // TODO: Implement actual detection

                if is_triggered != was_triggered {
                    self.properties.set_bool("triggered", is_triggered);
                    self.state = if is_triggered { ComponentState::Triggered } else { ComponentState::Idle };
                    log::debug!("Pressure plate {} {}", self.id, if is_triggered { "activated" } else { "deactivated" });
                }
            }
            TriggerType::LaserTripwire => {
                // TODO: Check for beam interruption
            }
            TriggerType::MotionDetector => {
                // TODO: Check for movement in range
            }
            TriggerType::SoundDetector => {
                // TODO: Check for sound levels
            }
            _ => {}
        }

        Ok(())
    }

    fn update_game_logic_component(&mut self, logic_type: &GameLogicType, delta_time: f32) -> RobinResult<()> {
        match logic_type {
            GameLogicType::SpawnPoint => {
                // Spawn point logic
                if let ComponentState::Spawning { timer } = &mut self.state {
                    *timer -= delta_time;
                    if *timer <= 0.0 {
                        // Spawn object
                        log::debug!("Spawn point {} spawning object", self.id);
                        self.state = ComponentState::Idle;
                        // TODO: Actually spawn the object
                    }
                }
            }
            GameLogicType::Collectible => {
                // Collectible auto-rotation for visual appeal
                if self.state == ComponentState::Idle {
                    let current_rotation_y = self.rotation.y;
                    self.rotation.y = (current_rotation_y + delta_time * 2.0) % (2.0 * std::f32::consts::PI);
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn update_utility_component(&mut self, utility_type: &UtilityType, delta_time: f32) -> RobinResult<()> {
        match utility_type {
            UtilityType::Light => {
                // Handle light flickering, dimming, etc.
                if let Some(flicker_rate) = self.properties.get_float("flicker_rate") {
                    if flicker_rate > 0.0 {
                        let flicker_time = self.properties.get_float("flicker_time").unwrap_or(0.0) + delta_time;
                        self.properties.set_float("flicker_time", flicker_time);

                        let intensity = 0.5 + 0.5 * (flicker_time * flicker_rate * 2.0 * std::f32::consts::PI).sin();
                        self.properties.set_float("current_intensity", intensity);
                    }
                }
            }
            UtilityType::AudioSource => {
                // Handle audio playback state
            }
            UtilityType::ParticleEmitter => {
                // Update particle emission
            }
            _ => {}
        }

        Ok(())
    }

    /// Trigger this component (called by logic system)
    pub fn trigger(&mut self, signal: LogicValue) -> RobinResult<()> {
        match &self.component_type {
            ComponentType::Mechanical(MechanicalType::Door) => {
                let open = signal.as_bool();
                if open {
                    self.open_door()?;
                } else {
                    self.close_door()?;
                }
            }
            ComponentType::GameLogic(GameLogicType::SpawnPoint) => {
                if signal.as_bool() {
                    self.start_spawning()?;
                }
            }
            _ => {
                log::debug!("Component {} triggered with signal: {:?}", self.id, signal);
            }
        }

        Ok(())
    }

    fn open_door(&mut self) -> RobinResult<()> {
        let open_position = self.properties.get_vector3("open_position")
            .unwrap_or([self.position.x, self.position.y + 3.0, self.position.z]);
        let movement_speed = self.properties.get_float("movement_speed").unwrap_or(2.0);

        self.state = ComponentState::Moving {
            target_position: open_position,
            speed: movement_speed,
        };

        log::debug!("Opening door {}", self.id);
        Ok(())
    }

    fn close_door(&mut self) -> RobinResult<()> {
        let closed_position = self.properties.get_vector3("closed_position")
            .unwrap_or([self.position.x, self.position.y, self.position.z]);
        let movement_speed = self.properties.get_float("movement_speed").unwrap_or(2.0);

        self.state = ComponentState::Moving {
            target_position: closed_position,
            speed: movement_speed,
        };

        log::debug!("Closing door {}", self.id);
        Ok(())
    }

    fn start_spawning(&mut self) -> RobinResult<()> {
        let spawn_delay = self.properties.get_float("spawn_delay").unwrap_or(1.0);
        self.state = ComponentState::Spawning { timer: spawn_delay };
        log::debug!("Spawn point {} starting spawn sequence", self.id);
        Ok(())
    }

    /// Connect this component to a logic node
    pub fn connect_to_logic(&mut self, node_id: u32) {
        if !self.logic_connections.contains(&node_id) {
            self.logic_connections.push(node_id);
            log::debug!("Connected component {} to logic node {}", self.id, node_id);
        }
    }

    /// Disconnect from a logic node
    pub fn disconnect_from_logic(&mut self, node_id: u32) {
        self.logic_connections.retain(|&id| id != node_id);
        log::debug!("Disconnected component {} from logic node {}", self.id, node_id);
    }

    /// Get component display name
    pub fn get_display_name(&self) -> String {
        match &self.component_type {
            ComponentType::Mechanical(mech_type) => {
                match mech_type {
                    MechanicalType::Door => "Door".to_string(),
                    MechanicalType::Platform => "Moving Platform".to_string(),
                    MechanicalType::Elevator => "Elevator".to_string(),
                    MechanicalType::ConveyorBelt => "Conveyor Belt".to_string(),
                    MechanicalType::Piston => "Piston".to_string(),
                    MechanicalType::Crusher => "Crusher".to_string(),
                    MechanicalType::RotatingPlatform => "Rotating Platform".to_string(),
                }
            }
            ComponentType::Trigger(trigger_type) => {
                match trigger_type {
                    TriggerType::PressurePlate => "Pressure Plate".to_string(),
                    TriggerType::LaserTripwire => "Laser Tripwire".to_string(),
                    TriggerType::MotionDetector => "Motion Detector".to_string(),
                    TriggerType::SoundDetector => "Sound Detector".to_string(),
                    TriggerType::LightSensor => "Light Sensor".to_string(),
                    TriggerType::ProximityTrigger => "Proximity Trigger".to_string(),
                }
            }
            ComponentType::GameLogic(logic_type) => {
                match logic_type {
                    GameLogicType::SpawnPoint => "Spawn Point".to_string(),
                    GameLogicType::Checkpoint => "Checkpoint".to_string(),
                    GameLogicType::Collectible => "Collectible".to_string(),
                    GameLogicType::Hazard => "Hazard".to_string(),
                    GameLogicType::GoalZone => "Goal Zone".to_string(),
                    GameLogicType::Teleporter => "Teleporter".to_string(),
                    GameLogicType::InventoryChest => "Inventory Chest".to_string(),
                }
            }
            ComponentType::Utility(utility_type) => {
                match utility_type {
                    UtilityType::Light => "Light".to_string(),
                    UtilityType::Camera => "Camera".to_string(),
                    UtilityType::AudioSource => "Audio Source".to_string(),
                    UtilityType::ParticleEmitter => "Particle Emitter".to_string(),
                    UtilityType::InfoDisplay => "Info Display".to_string(),
                }
            }
        }
    }
}

/// Properties that can be configured on components
#[derive(Debug, Clone)]
pub struct ComponentProperties {
    properties: HashMap<String, LogicValue>,
}

impl ComponentProperties {
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
        }
    }

    pub fn default_for_type(component_type: &ComponentType) -> Self {
        let mut properties = Self::new();

        match component_type {
            ComponentType::Mechanical(MechanicalType::Door) => {
                properties.set_float("movement_speed", 2.0);
                properties.set_vector3("closed_position", [0.0, 0.0, 0.0]);
                properties.set_vector3("open_position", [0.0, 3.0, 0.0]);
                properties.set_vector3("current_position", [0.0, 0.0, 0.0]);
                properties.set_bool("locked", false);
            }
            ComponentType::Trigger(TriggerType::PressurePlate) => {
                properties.set_float("sensitivity", 1.0);
                properties.set_bool("triggered", false);
                properties.set_bool("one_time_use", false);
            }
            ComponentType::GameLogic(GameLogicType::SpawnPoint) => {
                properties.set_string("spawn_object", "enemy".to_string());
                properties.set_float("spawn_delay", 1.0);
                properties.set_int("max_spawns", -1); // -1 = unlimited
                properties.set_int("current_spawns", 0);
            }
            ComponentType::GameLogic(GameLogicType::Collectible) => {
                properties.set_int("points", 100);
                properties.set_string("item_type", "coin".to_string());
                properties.set_bool("collected", false);
            }
            ComponentType::Utility(UtilityType::Light) => {
                properties.set_float("intensity", 1.0);
                properties.set_float("range", 10.0);
                properties.set_vector3("color", [1.0, 1.0, 1.0]);
                properties.set_float("flicker_rate", 0.0);
                properties.set_float("flicker_time", 0.0);
                properties.set_float("current_intensity", 1.0);
            }
            ComponentType::Utility(UtilityType::AudioSource) => {
                properties.set_string("audio_file", "".to_string());
                properties.set_float("volume", 1.0);
                properties.set_float("pitch", 1.0);
                properties.set_bool("loop", false);
                properties.set_bool("3d_sound", true);
            }
            _ => {
                // Default properties for other types
                properties.set_bool("enabled", true);
            }
        }

        properties
    }

    // Convenience methods for common property types
    pub fn set_bool(&mut self, key: &str, value: bool) {
        self.properties.insert(key.to_string(), LogicValue::Bool(value));
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.properties.get(key).map(|v| v.as_bool())
    }

    pub fn set_int(&mut self, key: &str, value: i32) {
        self.properties.insert(key.to_string(), LogicValue::Int(value));
    }

    pub fn get_int(&self, key: &str) -> Option<i32> {
        self.properties.get(key).map(|v| v.as_int())
    }

    pub fn set_float(&mut self, key: &str, value: f32) {
        self.properties.insert(key.to_string(), LogicValue::Float(value));
    }

    pub fn get_float(&self, key: &str) -> Option<f32> {
        self.properties.get(key).map(|v| v.as_float())
    }

    pub fn set_string(&mut self, key: &str, value: String) {
        self.properties.insert(key.to_string(), LogicValue::String(value));
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        self.properties.get(key).map(|v| v.as_string())
    }

    pub fn set_vector3(&mut self, key: &str, value: [f32; 3]) {
        self.properties.insert(key.to_string(), LogicValue::Vector3(value));
    }

    pub fn get_vector3(&self, key: &str) -> Option<[f32; 3]> {
        if let Some(LogicValue::Vector3(v)) = self.properties.get(key) {
            Some(*v)
        } else {
            None
        }
    }
}

/// Current state of a component
#[derive(Debug, Clone, PartialEq)]
pub enum ComponentState {
    Idle,
    Triggered,
    Moving { target_position: [f32; 3], speed: f32 },
    Spawning { timer: f32 },
    Collecting,
    Error(String),
}

impl Default for ComponentLibrary {
    fn default() -> Self {
        Self::new()
    }
}