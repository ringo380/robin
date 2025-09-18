/// Controller/Gamepad Support for Robin Game Engine
///
/// Provides unified controller input across different platforms and APIs

use crate::engine::core::RobinResult;
use std::collections::HashMap;
use cgmath::Vector2;
use serde::{Serialize, Deserialize};

/// Controller manager - handles all connected controllers
pub struct ControllerManager {
    controllers: HashMap<u32, Controller>,
    next_controller_id: u32,
    vibration_enabled: bool,
    dead_zone: f32,
}

impl ControllerManager {
    /// Create new controller manager
    pub fn new() -> Self {
        Self {
            controllers: HashMap::new(),
            next_controller_id: 0,
            vibration_enabled: true,
            dead_zone: 0.15, // 15% dead zone by default
        }
    }

    /// Initialize controller system
    pub fn initialize(&mut self) -> RobinResult<()> {
        println!("🎮 Initializing controller support");

        // In production, this would initialize platform-specific controller APIs
        // For now, simulate connecting a controller
        self.simulate_controller_connection()?;

        Ok(())
    }

    /// Simulate controller connection for testing
    fn simulate_controller_connection(&mut self) -> RobinResult<()> {
        let controller = Controller {
            id: self.next_controller_id,
            controller_type: ControllerType::XboxOne,
            connected: true,
            battery_level: Some(BatteryLevel::Full),
            state: ControllerState::default(),
            capabilities: ControllerCapabilities {
                has_vibration: true,
                has_gyroscope: false,
                has_accelerometer: false,
                has_touchpad: false,
                has_light_bar: false,
                button_count: 14,
                axis_count: 6,
            },
        };

        self.controllers.insert(controller.id, controller);
        self.next_controller_id += 1;

        println!("✅ Controller 0 connected (Xbox One)");
        Ok(())
    }

    /// Update controller states
    pub fn update(&mut self, delta_time: f32) -> RobinResult<()> {
        // Update all connected controllers
        for controller in self.controllers.values_mut() {
            if controller.connected {
                controller.update(delta_time, self.dead_zone)?;
            }
        }

        // Check for new connections/disconnections
        self.check_connections()?;

        Ok(())
    }

    /// Check for controller connections/disconnections
    fn check_connections(&mut self) -> RobinResult<()> {
        // In production, would check platform APIs for changes
        Ok(())
    }

    /// Get controller by ID
    pub fn get_controller(&self, id: u32) -> Option<&Controller> {
        self.controllers.get(&id)
    }

    /// Get mutable controller by ID
    pub fn get_controller_mut(&mut self, id: u32) -> Option<&mut Controller> {
        self.controllers.get_mut(&id)
    }

    /// Get first connected controller
    pub fn get_primary_controller(&self) -> Option<&Controller> {
        self.controllers.values()
            .find(|c| c.connected)
    }

    /// Get all connected controllers
    pub fn get_connected_controllers(&self) -> Vec<&Controller> {
        self.controllers.values()
            .filter(|c| c.connected)
            .collect()
    }

    /// Set controller vibration
    pub fn set_vibration(&mut self, controller_id: u32, left: f32, right: f32) -> RobinResult<()> {
        if !self.vibration_enabled {
            return Ok(());
        }

        if let Some(controller) = self.controllers.get_mut(&controller_id) {
            controller.set_vibration(left, right)?;
        }

        Ok(())
    }

    /// Enable/disable vibration globally
    pub fn set_vibration_enabled(&mut self, enabled: bool) {
        self.vibration_enabled = enabled;

        if !enabled {
            // Stop all vibrations
            for controller in self.controllers.values_mut() {
                let _ = controller.set_vibration(0.0, 0.0);
            }
        }
    }

    /// Set dead zone for analog sticks
    pub fn set_dead_zone(&mut self, dead_zone: f32) {
        self.dead_zone = dead_zone.clamp(0.0, 0.5);
    }

    /// Shutdown controller system
    pub fn shutdown(&mut self) -> RobinResult<()> {
        println!("🛑 Shutting down controller support");

        // Stop all vibrations
        for controller in self.controllers.values_mut() {
            let _ = controller.set_vibration(0.0, 0.0);
        }

        self.controllers.clear();
        Ok(())
    }
}

/// Individual controller
pub struct Controller {
    pub id: u32,
    pub controller_type: ControllerType,
    pub connected: bool,
    pub battery_level: Option<BatteryLevel>,
    pub state: ControllerState,
    pub capabilities: ControllerCapabilities,
}

impl Controller {
    /// Update controller state
    fn update(&mut self, _delta_time: f32, dead_zone: f32) -> RobinResult<()> {
        // In production, read actual controller state
        // For now, simulate some input for testing

        // Apply dead zone to analog sticks
        self.state.left_stick = Self::apply_dead_zone(self.state.left_stick, dead_zone);
        self.state.right_stick = Self::apply_dead_zone(self.state.right_stick, dead_zone);

        Ok(())
    }

    /// Apply dead zone to analog stick
    fn apply_dead_zone(stick: Vector2<f32>, dead_zone: f32) -> Vector2<f32> {
        let magnitude = stick.magnitude();
        if magnitude < dead_zone {
            Vector2::new(0.0, 0.0)
        } else {
            let normalized = stick / magnitude;
            let adjusted_magnitude = (magnitude - dead_zone) / (1.0 - dead_zone);
            normalized * adjusted_magnitude
        }
    }

    /// Set controller vibration
    fn set_vibration(&mut self, left: f32, right: f32) -> RobinResult<()> {
        if !self.capabilities.has_vibration {
            return Ok(());
        }

        let left = left.clamp(0.0, 1.0);
        let right = right.clamp(0.0, 1.0);

        // In production, call platform API
        println!("🎮 Controller {} vibration: L={:.2} R={:.2}", self.id, left, right);

        Ok(())
    }

    /// Check if button is pressed
    pub fn is_button_pressed(&self, button: Button) -> bool {
        self.state.buttons.contains(&button)
    }

    /// Get left analog stick
    pub fn get_left_stick(&self) -> Vector2<f32> {
        self.state.left_stick
    }

    /// Get right analog stick
    pub fn get_right_stick(&self) -> Vector2<f32> {
        self.state.right_stick
    }

    /// Get left trigger value (0.0 - 1.0)
    pub fn get_left_trigger(&self) -> f32 {
        self.state.left_trigger
    }

    /// Get right trigger value (0.0 - 1.0)
    pub fn get_right_trigger(&self) -> f32 {
        self.state.right_trigger
    }
}

/// Controller types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControllerType {
    XboxOne,
    Xbox360,
    PlayStation4,
    PlayStation5,
    SwitchPro,
    SteamDeck,
    Generic,
}

/// Controller state
#[derive(Debug, Clone)]
pub struct ControllerState {
    pub buttons: Vec<Button>,
    pub left_stick: Vector2<f32>,
    pub right_stick: Vector2<f32>,
    pub left_trigger: f32,
    pub right_trigger: f32,
    pub gyroscope: Option<Vector3<f32>>,
    pub accelerometer: Option<Vector3<f32>>,
}

impl Default for ControllerState {
    fn default() -> Self {
        Self {
            buttons: Vec::new(),
            left_stick: Vector2::new(0.0, 0.0),
            right_stick: Vector2::new(0.0, 0.0),
            left_trigger: 0.0,
            right_trigger: 0.0,
            gyroscope: None,
            accelerometer: None,
        }
    }
}

use cgmath::Vector3;

/// Controller buttons
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Button {
    // Face buttons
    A, // Cross on PlayStation
    B, // Circle on PlayStation
    X, // Square on PlayStation
    Y, // Triangle on PlayStation

    // D-Pad
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,

    // Shoulder buttons
    LeftBumper,  // L1
    RightBumper, // R1

    // Analog stick buttons
    LeftStick,  // L3
    RightStick, // R3

    // Center buttons
    Start,  // Options on PlayStation
    Select, // Share on PlayStation
    Guide,  // PlayStation/Xbox button

    // Additional buttons
    TouchPad,     // PS4/PS5 only
    Capture,      // Switch/Xbox Series only
    Microphone,   // PS5 only
}

/// Controller capabilities
#[derive(Debug, Clone)]
pub struct ControllerCapabilities {
    pub has_vibration: bool,
    pub has_gyroscope: bool,
    pub has_accelerometer: bool,
    pub has_touchpad: bool,
    pub has_light_bar: bool,
    pub button_count: u32,
    pub axis_count: u32,
}

/// Battery level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatteryLevel {
    Empty,
    Low,
    Medium,
    Full,
    Charging,
    Wired,
}

/// Input mapping profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputMapping {
    pub name: String,
    pub button_mappings: HashMap<Button, String>,
    pub axis_mappings: HashMap<String, AxisMapping>,
}

/// Axis mapping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisMapping {
    pub axis_name: String,
    pub invert: bool,
    pub sensitivity: f32,
    pub dead_zone: f32,
}

impl InputMapping {
    /// Create default mapping for game
    pub fn default_game_mapping() -> Self {
        let mut button_mappings = HashMap::new();
        button_mappings.insert(Button::A, "jump".to_string());
        button_mappings.insert(Button::B, "crouch".to_string());
        button_mappings.insert(Button::X, "interact".to_string());
        button_mappings.insert(Button::Y, "inventory".to_string());
        button_mappings.insert(Button::LeftBumper, "previous_tool".to_string());
        button_mappings.insert(Button::RightBumper, "next_tool".to_string());
        button_mappings.insert(Button::Start, "pause".to_string());
        button_mappings.insert(Button::Select, "map".to_string());

        let mut axis_mappings = HashMap::new();
        axis_mappings.insert("move_horizontal".to_string(), AxisMapping {
            axis_name: "left_stick_x".to_string(),
            invert: false,
            sensitivity: 1.0,
            dead_zone: 0.15,
        });
        axis_mappings.insert("move_vertical".to_string(), AxisMapping {
            axis_name: "left_stick_y".to_string(),
            invert: false,
            sensitivity: 1.0,
            dead_zone: 0.15,
        });
        axis_mappings.insert("look_horizontal".to_string(), AxisMapping {
            axis_name: "right_stick_x".to_string(),
            invert: false,
            sensitivity: 2.0,
            dead_zone: 0.15,
        });
        axis_mappings.insert("look_vertical".to_string(), AxisMapping {
            axis_name: "right_stick_y".to_string(),
            invert: true,
            sensitivity: 2.0,
            dead_zone: 0.15,
        });

        Self {
            name: "Default Game".to_string(),
            button_mappings,
            axis_mappings,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_controller_manager() {
        let mut manager = ControllerManager::new();
        assert!(manager.initialize().is_ok());

        // Should have simulated controller
        let controllers = manager.get_connected_controllers();
        assert_eq!(controllers.len(), 1);

        // Get primary controller
        let primary = manager.get_primary_controller();
        assert!(primary.is_some());
        assert_eq!(primary.unwrap().controller_type, ControllerType::XboxOne);
    }

    #[test]
    fn test_dead_zone() {
        let stick = Vector2::new(0.1, 0.1);
        let with_dead_zone = Controller::apply_dead_zone(stick, 0.15);
        assert_eq!(with_dead_zone, Vector2::new(0.0, 0.0));

        let stick = Vector2::new(0.5, 0.5);
        let with_dead_zone = Controller::apply_dead_zone(stick, 0.15);
        assert!(with_dead_zone.magnitude() > 0.0);
    }

    #[test]
    fn test_input_mapping() {
        let mapping = InputMapping::default_game_mapping();
        assert_eq!(mapping.button_mappings.get(&Button::A), Some(&"jump".to_string()));
        assert!(mapping.axis_mappings.contains_key("move_horizontal"));
    }

    #[test]
    fn test_vibration() {
        let mut manager = ControllerManager::new();
        manager.initialize().unwrap();

        // Test vibration
        assert!(manager.set_vibration(0, 0.5, 0.5).is_ok());

        // Disable vibration
        manager.set_vibration_enabled(false);
        assert!(manager.set_vibration(0, 1.0, 1.0).is_ok()); // Should not error but won't vibrate
    }
}