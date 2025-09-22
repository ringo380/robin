/*!
 * Logic System - Visual Programming in 3D Space
 *
 * This module implements the logic node system that allows users to create
 * game mechanics by placing and connecting physical logic objects in 3D space.
 * Think of it as visual programming made tangible.
 */

use crate::engine::{
    math::{Vec3, Vec2},
    graphics::{Color, Mesh},
    input::InputManager,
    error::{RobinResult, RobinError},
};
use std::collections::{HashMap, VecDeque};
use serde::{Serialize, Deserialize};
use winit::keyboard::{Key, KeyCode, NamedKey};
use rand::Rng;
use cgmath::InnerSpace;

// Helper function to convert KeyCode to Key
fn keycode_to_key(keycode: KeyCode) -> Key {
    // In winit 0.29, we map KeyCode to appropriate Key variants
    match keycode {
        KeyCode::KeyA => Key::Character("a".into()),
        KeyCode::KeyB => Key::Character("b".into()),
        KeyCode::KeyC => Key::Character("c".into()),
        KeyCode::KeyD => Key::Character("d".into()),
        KeyCode::KeyE => Key::Character("e".into()),
        KeyCode::KeyF => Key::Character("f".into()),
        KeyCode::KeyG => Key::Character("g".into()),
        KeyCode::KeyH => Key::Character("h".into()),
        KeyCode::KeyI => Key::Character("i".into()),
        KeyCode::KeyJ => Key::Character("j".into()),
        KeyCode::KeyK => Key::Character("k".into()),
        KeyCode::KeyL => Key::Character("l".into()),
        KeyCode::KeyM => Key::Character("m".into()),
        KeyCode::KeyN => Key::Character("n".into()),
        KeyCode::KeyO => Key::Character("o".into()),
        KeyCode::KeyP => Key::Character("p".into()),
        KeyCode::KeyQ => Key::Character("q".into()),
        KeyCode::KeyR => Key::Character("r".into()),
        KeyCode::KeyS => Key::Character("s".into()),
        KeyCode::KeyT => Key::Character("t".into()),
        KeyCode::KeyU => Key::Character("u".into()),
        KeyCode::KeyV => Key::Character("v".into()),
        KeyCode::KeyW => Key::Character("w".into()),
        KeyCode::KeyX => Key::Character("x".into()),
        KeyCode::KeyY => Key::Character("y".into()),
        KeyCode::KeyZ => Key::Character("z".into()),
        KeyCode::Digit1 => Key::Character("1".into()),
        KeyCode::Digit2 => Key::Character("2".into()),
        KeyCode::Digit3 => Key::Character("3".into()),
        KeyCode::Digit4 => Key::Character("4".into()),
        KeyCode::Digit5 => Key::Character("5".into()),
        KeyCode::Digit6 => Key::Character("6".into()),
        KeyCode::Digit7 => Key::Character("7".into()),
        KeyCode::Digit8 => Key::Character("8".into()),
        KeyCode::Digit9 => Key::Character("9".into()),
        KeyCode::Digit0 => Key::Character("0".into()),
        KeyCode::Space => Key::Named(NamedKey::Space),
        KeyCode::Enter => Key::Named(NamedKey::Enter),
        KeyCode::Escape => Key::Named(NamedKey::Escape),
        KeyCode::ShiftLeft | KeyCode::ShiftRight => Key::Named(NamedKey::Shift),
        KeyCode::ControlLeft | KeyCode::ControlRight => Key::Named(NamedKey::Control),
        KeyCode::AltLeft | KeyCode::AltRight => Key::Named(NamedKey::Alt),
        _ => Key::Unidentified(winit::keyboard::NativeKeyCode::Unidentified.into()),
    }
}

/// The logic system manages all logic nodes and their connections
#[derive(Debug)]
pub struct LogicSystem {
    /// All logic nodes in the world
    nodes: HashMap<u32, LogicNode>,

    /// Connections between nodes
    connections: Vec<LogicConnection>,

    /// Next available node ID
    next_node_id: u32,

    /// Execution queue for processing logic
    execution_queue: VecDeque<u32>,

    /// Whether the logic system is running
    running: bool,

    /// Debug visualization settings
    debug_mode: bool,
}

impl LogicSystem {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            connections: Vec::new(),
            next_node_id: 1,
            execution_queue: VecDeque::new(),
            running: false,
            debug_mode: true,
        }
    }

    /// Update the logic system
    pub fn update(&mut self, delta_time: f32, input: &InputManager) -> RobinResult<()> {
        // Toggle logic execution with Space
        if input.is_key_just_pressed(&keycode_to_key(winit::keyboard::KeyCode::Space)) {
            self.running = !self.running;
            log::info!("Logic system {}", if self.running { "started" } else { "stopped" });
        }

        // Toggle debug mode with F3
        if input.is_key_just_pressed(&keycode_to_key(winit::keyboard::KeyCode::F3)) {
            self.debug_mode = !self.debug_mode;
            log::info!("Logic debug mode {}", if self.debug_mode { "enabled" } else { "disabled" });
        }

        if self.running {
            self.execute_logic(delta_time)?;
        }

        // Update individual nodes
        for node in self.nodes.values_mut() {
            node.update(delta_time)?;
        }

        Ok(())
    }

    /// Create a new logic node at the specified position
    pub fn create_node(&mut self, node_type: LogicNodeType, position: Vec3) -> u32 {
        let node_id = self.next_node_id;
        self.next_node_id += 1;

        let node = LogicNode::new(node_id, node_type.clone(), position);
        self.nodes.insert(node_id, node);

        log::debug!("Created {:?} node {} at {:?}", node_type, node_id, position);
        node_id
    }

    /// Connect two nodes
    pub fn connect_nodes(
        &mut self,
        from_node: u32,
        from_output: String,
        to_node: u32,
        to_input: String,
    ) -> RobinResult<()> {
        // Validate nodes exist
        if !self.nodes.contains_key(&from_node) || !self.nodes.contains_key(&to_node) {
            return Err(RobinError::InvalidOperation {
                operation: "connect_nodes".to_string(),
                context: "Logic system".to_string(),
                reason: "One or both nodes not found".to_string(),
            });
        }

        // Check for cycles (prevent infinite loops)
        if self.would_create_cycle(from_node, to_node)? {
            return Err(RobinError::InvalidOperation {
                operation: "connect_nodes".to_string(),
                context: "Logic system".to_string(),
                reason: "Connection would create a cycle".to_string(),
            });
        }

        let connection = LogicConnection {
            from_node,
            from_output,
            to_node,
            to_input,
            wire_type: WireType::Data,
            active: true,
        };

        self.connections.push(connection);
        log::debug!("Connected node {} to node {}", from_node, to_node);

        Ok(())
    }

    /// Remove a connection between nodes
    pub fn disconnect_nodes(&mut self, from_node: u32, to_node: u32) {
        self.connections.retain(|conn| !(conn.from_node == from_node && conn.to_node == to_node));
        log::debug!("Disconnected node {} from node {}", from_node, to_node);
    }

    /// Execute logic for one frame
    fn execute_logic(&mut self, delta_time: f32) -> RobinResult<()> {
        // Find all nodes that should execute this frame
        let mut nodes_to_execute = Vec::new();

        for (&node_id, node) in &self.nodes {
            if node.should_execute() {
                nodes_to_execute.push(node_id);
            }
        }

        // Execute nodes in dependency order
        for node_id in nodes_to_execute {
            self.execute_node(node_id, delta_time)?;
        }

        Ok(())
    }

    /// Execute a specific node
    fn execute_node(&mut self, node_id: u32, delta_time: f32) -> RobinResult<()> {
        // Get input values from connected nodes
        let mut inputs = HashMap::new();
        for connection in &self.connections {
            if connection.to_node == node_id && connection.active {
                if let Some(source_node) = self.nodes.get(&connection.from_node) {
                    if let Some(output_value) = source_node.get_output(&connection.from_output) {
                        inputs.insert(connection.to_input.clone(), output_value.clone());
                    }
                }
            }
        }

        // Execute the node with inputs
        if let Some(node) = self.nodes.get_mut(&node_id) {
            node.execute(inputs, delta_time)?;
        }

        Ok(())
    }

    /// Check if connecting two nodes would create a cycle
    fn would_create_cycle(&self, from_node: u32, to_node: u32) -> RobinResult<bool> {
        // Use depth-first search to detect cycles
        let mut visited = std::collections::HashSet::new();
        let mut rec_stack = std::collections::HashSet::new();

        fn dfs(
            node: u32,
            target: u32,
            connections: &[LogicConnection],
            visited: &mut std::collections::HashSet<u32>,
            rec_stack: &mut std::collections::HashSet<u32>,
        ) -> bool {
            visited.insert(node);
            rec_stack.insert(node);

            // Check all outgoing connections
            for connection in connections {
                if connection.from_node == node {
                    let next_node = connection.to_node;

                    if next_node == target {
                        return true; // Found path back to target
                    }

                    if !visited.contains(&next_node) {
                        if dfs(next_node, target, connections, visited, rec_stack) {
                            return true;
                        }
                    } else if rec_stack.contains(&next_node) {
                        return true; // Found cycle
                    }
                }
            }

            rec_stack.remove(&node);
            false
        }

        Ok(dfs(to_node, from_node, &self.connections, &mut visited, &mut rec_stack))
    }

    /// Get all nodes
    pub fn get_nodes(&self) -> &HashMap<u32, LogicNode> {
        &self.nodes
    }

    /// Get all connections
    pub fn get_connections(&self) -> &[LogicConnection] {
        &self.connections
    }

    /// Get a specific node
    pub fn get_node(&self, node_id: u32) -> Option<&LogicNode> {
        self.nodes.get(&node_id)
    }

    /// Get a mutable reference to a specific node
    pub fn get_node_mut(&mut self, node_id: u32) -> Option<&mut LogicNode> {
        self.nodes.get_mut(&node_id)
    }

    /// Remove a node and all its connections
    pub fn remove_node(&mut self, node_id: u32) {
        self.nodes.remove(&node_id);
        self.connections.retain(|conn| conn.from_node != node_id && conn.to_node != node_id);
        log::debug!("Removed node {} and its connections", node_id);
    }

    /// Clear all nodes and connections
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.connections.clear();
        self.execution_queue.clear();
        log::info!("Cleared all logic nodes and connections");
    }
}

/// A single logic node in the system
#[derive(Debug, Clone)]
pub struct LogicNode {
    pub id: u32,
    pub node_type: LogicNodeType,
    pub position: Vec3,
    pub inputs: HashMap<String, LogicValue>,
    pub outputs: HashMap<String, LogicValue>,
    pub properties: HashMap<String, LogicValue>,
    pub state: NodeState,
    pub last_execution_time: f32,
    pub execution_interval: f32,
}

impl LogicNode {
    pub fn new(id: u32, node_type: LogicNodeType, position: Vec3) -> Self {
        let mut node = Self {
            id,
            node_type: node_type.clone(),
            position,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            properties: HashMap::new(),
            state: NodeState::Idle,
            last_execution_time: 0.0,
            execution_interval: 0.0,
        };

        // Initialize inputs/outputs based on node type
        node.initialize_ports(&node_type);
        node
    }

    fn initialize_ports(&mut self, node_type: &LogicNodeType) {
        match node_type {
            LogicNodeType::Sensor(sensor_type) => {
                match sensor_type {
                    SensorType::Proximity => {
                        self.properties.insert("range".to_string(), LogicValue::Float(5.0));
                        self.outputs.insert("triggered".to_string(), LogicValue::Bool(false));
                        self.outputs.insert("distance".to_string(), LogicValue::Float(0.0));
                    }
                    SensorType::LineOfSight => {
                        self.properties.insert("range".to_string(), LogicValue::Float(10.0));
                        self.outputs.insert("can_see".to_string(), LogicValue::Bool(false));
                    }
                    SensorType::Interaction => {
                        self.outputs.insert("interacted".to_string(), LogicValue::Bool(false));
                    }
                }
            }
            LogicNodeType::Condition(condition_type) => {
                match condition_type {
                    ConditionType::IfThen => {
                        self.inputs.insert("condition".to_string(), LogicValue::Bool(false));
                        self.outputs.insert("true_output".to_string(), LogicValue::Bool(false));
                        self.outputs.insert("false_output".to_string(), LogicValue::Bool(false));
                    }
                    ConditionType::Compare => {
                        self.inputs.insert("value_a".to_string(), LogicValue::Float(0.0));
                        self.inputs.insert("value_b".to_string(), LogicValue::Float(0.0));
                        self.properties.insert("operator".to_string(), LogicValue::String("==".to_string()));
                        self.outputs.insert("result".to_string(), LogicValue::Bool(false));
                    }
                    ConditionType::And => {
                        self.inputs.insert("input_a".to_string(), LogicValue::Bool(false));
                        self.inputs.insert("input_b".to_string(), LogicValue::Bool(false));
                        self.outputs.insert("output".to_string(), LogicValue::Bool(false));
                    }
                    ConditionType::Or => {
                        self.inputs.insert("input_a".to_string(), LogicValue::Bool(false));
                        self.inputs.insert("input_b".to_string(), LogicValue::Bool(false));
                        self.outputs.insert("output".to_string(), LogicValue::Bool(false));
                    }
                    ConditionType::Not => {
                        self.inputs.insert("input".to_string(), LogicValue::Bool(false));
                        self.outputs.insert("output".to_string(), LogicValue::Bool(false));
                    }
                }
            }
            LogicNodeType::Action(action_type) => {
                match action_type {
                    ActionType::OpenDoor => {
                        self.inputs.insert("trigger".to_string(), LogicValue::Bool(false));
                        self.properties.insert("door_id".to_string(), LogicValue::Int(0));
                    }
                    ActionType::SpawnObject => {
                        self.inputs.insert("trigger".to_string(), LogicValue::Bool(false));
                        self.properties.insert("object_type".to_string(), LogicValue::String("enemy".to_string()));
                        self.properties.insert("spawn_position".to_string(), LogicValue::Vector3([0.0, 0.0, 0.0]));
                    }
                    ActionType::SetVariable => {
                        self.inputs.insert("trigger".to_string(), LogicValue::Bool(false));
                        self.inputs.insert("value".to_string(), LogicValue::Float(0.0));
                        self.properties.insert("variable_name".to_string(), LogicValue::String("score".to_string()));
                    }
                    ActionType::PlaySound => {
                        self.inputs.insert("trigger".to_string(), LogicValue::Bool(false));
                        self.properties.insert("sound_id".to_string(), LogicValue::String("beep".to_string()));
                    }
                }
            }
            LogicNodeType::Variable(var_type) => {
                match var_type {
                    VariableType::Counter => {
                        self.inputs.insert("increment".to_string(), LogicValue::Bool(false));
                        self.inputs.insert("decrement".to_string(), LogicValue::Bool(false));
                        self.inputs.insert("reset".to_string(), LogicValue::Bool(false));
                        self.properties.insert("value".to_string(), LogicValue::Int(0));
                        self.outputs.insert("current_value".to_string(), LogicValue::Int(0));
                    }
                    VariableType::Timer => {
                        self.inputs.insert("start".to_string(), LogicValue::Bool(false));
                        self.inputs.insert("stop".to_string(), LogicValue::Bool(false));
                        self.inputs.insert("reset".to_string(), LogicValue::Bool(false));
                        self.properties.insert("duration".to_string(), LogicValue::Float(5.0));
                        self.outputs.insert("elapsed".to_string(), LogicValue::Float(0.0));
                        self.outputs.insert("finished".to_string(), LogicValue::Bool(false));
                    }
                    VariableType::Storage => {
                        self.inputs.insert("store".to_string(), LogicValue::Bool(false));
                        self.inputs.insert("value".to_string(), LogicValue::Float(0.0));
                        self.outputs.insert("stored_value".to_string(), LogicValue::Float(0.0));
                    }
                }
            }
            LogicNodeType::Math(math_type) => {
                match math_type {
                    MathType::Add => {
                        self.inputs.insert("a".to_string(), LogicValue::Float(0.0));
                        self.inputs.insert("b".to_string(), LogicValue::Float(0.0));
                        self.outputs.insert("result".to_string(), LogicValue::Float(0.0));
                    }
                    MathType::Subtract => {
                        self.inputs.insert("a".to_string(), LogicValue::Float(0.0));
                        self.inputs.insert("b".to_string(), LogicValue::Float(0.0));
                        self.outputs.insert("result".to_string(), LogicValue::Float(0.0));
                    }
                    MathType::Multiply => {
                        self.inputs.insert("a".to_string(), LogicValue::Float(0.0));
                        self.inputs.insert("b".to_string(), LogicValue::Float(0.0));
                        self.outputs.insert("result".to_string(), LogicValue::Float(0.0));
                    }
                    MathType::Divide => {
                        self.inputs.insert("a".to_string(), LogicValue::Float(0.0));
                        self.inputs.insert("b".to_string(), LogicValue::Float(1.0));
                        self.outputs.insert("result".to_string(), LogicValue::Float(0.0));
                    }
                    MathType::Random => {
                        self.inputs.insert("min".to_string(), LogicValue::Float(0.0));
                        self.inputs.insert("max".to_string(), LogicValue::Float(1.0));
                        self.inputs.insert("generate".to_string(), LogicValue::Bool(false));
                        self.outputs.insert("value".to_string(), LogicValue::Float(0.0));
                    }
                }
            }
        }
    }

    pub fn update(&mut self, delta_time: f32) -> RobinResult<()> {
        self.last_execution_time += delta_time;

        // Update timers and other time-based nodes
        if let LogicNodeType::Variable(VariableType::Timer) = &self.node_type {
            if self.state == NodeState::Running {
                let elapsed = self.outputs.get("elapsed").unwrap().as_float() + delta_time;
                let duration = self.properties.get("duration").unwrap().as_float();

                self.outputs.insert("elapsed".to_string(), LogicValue::Float(elapsed));

                if elapsed >= duration {
                    self.outputs.insert("finished".to_string(), LogicValue::Bool(true));
                    self.state = NodeState::Finished;
                }
            }
        }

        Ok(())
    }

    pub fn execute(&mut self, inputs: HashMap<String, LogicValue>, _delta_time: f32) -> RobinResult<()> {
        // Update input values
        for (key, value) in inputs {
            self.inputs.insert(key, value);
        }

        // Execute based on node type
        match &self.node_type {
            LogicNodeType::Condition(ConditionType::IfThen) => {
                let condition = self.inputs.get("condition").unwrap_or(&LogicValue::Bool(false)).as_bool();
                self.outputs.insert("true_output".to_string(), LogicValue::Bool(condition));
                self.outputs.insert("false_output".to_string(), LogicValue::Bool(!condition));
            }
            LogicNodeType::Condition(ConditionType::And) => {
                let a = self.inputs.get("input_a").unwrap_or(&LogicValue::Bool(false)).as_bool();
                let b = self.inputs.get("input_b").unwrap_or(&LogicValue::Bool(false)).as_bool();
                self.outputs.insert("output".to_string(), LogicValue::Bool(a && b));
            }
            LogicNodeType::Condition(ConditionType::Or) => {
                let a = self.inputs.get("input_a").unwrap_or(&LogicValue::Bool(false)).as_bool();
                let b = self.inputs.get("input_b").unwrap_or(&LogicValue::Bool(false)).as_bool();
                self.outputs.insert("output".to_string(), LogicValue::Bool(a || b));
            }
            LogicNodeType::Condition(ConditionType::Not) => {
                let input = self.inputs.get("input").unwrap_or(&LogicValue::Bool(false)).as_bool();
                self.outputs.insert("output".to_string(), LogicValue::Bool(!input));
            }
            LogicNodeType::Math(MathType::Add) => {
                let a = self.inputs.get("a").unwrap_or(&LogicValue::Float(0.0)).as_float();
                let b = self.inputs.get("b").unwrap_or(&LogicValue::Float(0.0)).as_float();
                self.outputs.insert("result".to_string(), LogicValue::Float(a + b));
            }
            LogicNodeType::Math(MathType::Subtract) => {
                let a = self.inputs.get("a").unwrap_or(&LogicValue::Float(0.0)).as_float();
                let b = self.inputs.get("b").unwrap_or(&LogicValue::Float(0.0)).as_float();
                self.outputs.insert("result".to_string(), LogicValue::Float(a - b));
            }
            LogicNodeType::Math(MathType::Multiply) => {
                let a = self.inputs.get("a").unwrap_or(&LogicValue::Float(0.0)).as_float();
                let b = self.inputs.get("b").unwrap_or(&LogicValue::Float(0.0)).as_float();
                self.outputs.insert("result".to_string(), LogicValue::Float(a * b));
            }
            LogicNodeType::Math(MathType::Divide) => {
                let a = self.inputs.get("a").unwrap_or(&LogicValue::Float(0.0)).as_float();
                let b = self.inputs.get("b").unwrap_or(&LogicValue::Float(1.0)).as_float();
                let result = if b != 0.0 { a / b } else { 0.0 };
                self.outputs.insert("result".to_string(), LogicValue::Float(result));
            }
            LogicNodeType::Variable(VariableType::Counter) => {
                let increment = self.inputs.get("increment").unwrap_or(&LogicValue::Bool(false)).as_bool();
                let decrement = self.inputs.get("decrement").unwrap_or(&LogicValue::Bool(false)).as_bool();
                let reset = self.inputs.get("reset").unwrap_or(&LogicValue::Bool(false)).as_bool();

                let mut value = self.properties.get("value").unwrap_or(&LogicValue::Int(0)).as_int();

                if reset {
                    value = 0;
                } else if increment {
                    value += 1;
                } else if decrement {
                    value -= 1;
                }

                self.properties.insert("value".to_string(), LogicValue::Int(value));
                self.outputs.insert("current_value".to_string(), LogicValue::Int(value));
            }
            LogicNodeType::Variable(VariableType::Timer) => {
                let start = self.inputs.get("start").unwrap_or(&LogicValue::Bool(false)).as_bool();
                let stop = self.inputs.get("stop").unwrap_or(&LogicValue::Bool(false)).as_bool();
                let reset = self.inputs.get("reset").unwrap_or(&LogicValue::Bool(false)).as_bool();

                if reset {
                    self.outputs.insert("elapsed".to_string(), LogicValue::Float(0.0));
                    self.outputs.insert("finished".to_string(), LogicValue::Bool(false));
                    self.state = NodeState::Idle;
                } else if start && self.state == NodeState::Idle {
                    self.state = NodeState::Running;
                } else if stop {
                    self.state = NodeState::Idle;
                }
            }
            LogicNodeType::Math(MathType::Random) => {
                let generate = self.inputs.get("generate").unwrap_or(&LogicValue::Bool(false)).as_bool();
                if generate {
                    let min = self.inputs.get("min").unwrap_or(&LogicValue::Float(0.0)).as_float();
                    let max = self.inputs.get("max").unwrap_or(&LogicValue::Float(1.0)).as_float();
                    let value = min + rand::thread_rng().gen::<f32>() * (max - min);
                    self.outputs.insert("value".to_string(), LogicValue::Float(value));
                }
            }
            LogicNodeType::Sensor(sensor_type) => {
                match sensor_type {
                    SensorType::Proximity => {
                        let range = self.properties.get("range").unwrap_or(&LogicValue::Float(5.0)).as_float();
                        // TODO: Implement actual proximity detection
                        let triggered = rand::thread_rng().gen::<f32>() < 0.1;
                        let distance = if triggered { rand::thread_rng().gen::<f32>() * range } else { range + 1.0 };

                        self.outputs.insert("triggered".to_string(), LogicValue::Bool(triggered));
                        self.outputs.insert("distance".to_string(), LogicValue::Float(distance));
                    }
                    SensorType::LineOfSight => {
                        let can_see = rand::thread_rng().gen::<f32>() < 0.3;
                        self.outputs.insert("can_see".to_string(), LogicValue::Bool(can_see));
                    }
                    SensorType::Interaction => {
                        let interacted = rand::thread_rng().gen::<f32>() < 0.05;
                        self.outputs.insert("interacted".to_string(), LogicValue::Bool(interacted));
                    }
                }
            }
            LogicNodeType::Action(action_type) => {
                let trigger = self.inputs.get("trigger").unwrap_or(&LogicValue::Bool(false)).as_bool();
                if trigger {
                    match action_type {
                        ActionType::OpenDoor => {
                            let door_id = self.properties.get("door_id").unwrap_or(&LogicValue::Int(0)).as_int();
                            log::info!("Opening door with ID: {}", door_id);
                        }
                        ActionType::SpawnObject => {
                            let object_type = self.properties.get("object_type").unwrap_or(&LogicValue::String("enemy".to_string())).as_string();
                            log::info!("Spawning object: {}", object_type);
                        }
                        ActionType::SetVariable => {
                            let value = self.inputs.get("value").unwrap_or(&LogicValue::Float(0.0));
                            let var_name = self.properties.get("variable_name").unwrap_or(&LogicValue::String("score".to_string())).as_string();
                            log::info!("Setting variable {} to {:?}", var_name, value);
                        }
                        ActionType::PlaySound => {
                            let sound_id = self.properties.get("sound_id").unwrap_or(&LogicValue::String("beep".to_string())).as_string();
                            log::info!("Playing sound: {}", sound_id);
                        }
                    }
                }
            }
            _ => {
                // Other node types already implemented above
            }
        }

        Ok(())
    }

    pub fn should_execute(&self) -> bool {
        match self.state {
            NodeState::Running => true,
            NodeState::Triggered => true,
            _ => false,
        }
    }

    pub fn get_output(&self, output_name: &str) -> Option<&LogicValue> {
        self.outputs.get(output_name)
    }

    pub fn set_input(&mut self, input_name: String, value: LogicValue) {
        self.inputs.insert(input_name, value);
    }

    pub fn get_property(&self, property_name: &str) -> Option<&LogicValue> {
        self.properties.get(property_name)
    }

    pub fn set_property(&mut self, property_name: String, value: LogicValue) {
        self.properties.insert(property_name, value);
    }
}

/// Types of logic nodes available
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogicNodeType {
    /// Sensor nodes detect world state
    Sensor(SensorType),
    /// Condition nodes provide boolean logic
    Condition(ConditionType),
    /// Action nodes modify world state
    Action(ActionType),
    /// Variable nodes store and manipulate data
    Variable(VariableType),
    /// Math nodes perform calculations
    Math(MathType),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SensorType {
    Proximity,
    LineOfSight,
    Interaction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionType {
    IfThen,
    Compare,
    And,
    Or,
    Not,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    OpenDoor,
    SpawnObject,
    SetVariable,
    PlaySound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VariableType {
    Counter,
    Timer,
    Storage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MathType {
    Add,
    Subtract,
    Multiply,
    Divide,
    Random,
}

/// Values that can flow through logic connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogicValue {
    Bool(bool),
    Int(i32),
    Float(f32),
    String(String),
    Vector3([f32; 3]),
    Object(u32), // Object ID reference
}

impl LogicValue {
    pub fn as_bool(&self) -> bool {
        match self {
            LogicValue::Bool(b) => *b,
            LogicValue::Int(i) => *i != 0,
            LogicValue::Float(f) => *f != 0.0,
            LogicValue::String(s) => !s.is_empty(),
            _ => false,
        }
    }

    pub fn as_int(&self) -> i32 {
        match self {
            LogicValue::Int(i) => *i,
            LogicValue::Float(f) => *f as i32,
            LogicValue::Bool(b) => if *b { 1 } else { 0 },
            _ => 0,
        }
    }

    pub fn as_float(&self) -> f32 {
        match self {
            LogicValue::Float(f) => *f,
            LogicValue::Int(i) => *i as f32,
            LogicValue::Bool(b) => if *b { 1.0 } else { 0.0 },
            _ => 0.0,
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            LogicValue::String(s) => s.clone(),
            LogicValue::Bool(b) => b.to_string(),
            LogicValue::Int(i) => i.to_string(),
            LogicValue::Float(f) => f.to_string(),
            LogicValue::Vector3(v) => format!("({}, {}, {})", v[0], v[1], v[2]),
            LogicValue::Object(id) => format!("Object#{}", id),
        }
    }
}

/// Connection between two logic nodes
#[derive(Debug, Clone)]
pub struct LogicConnection {
    pub from_node: u32,
    pub from_output: String,
    pub to_node: u32,
    pub to_input: String,
    pub wire_type: WireType,
    pub active: bool,
}

/// Types of connections/wires
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WireType {
    /// General data flow (blue)
    Data,
    /// Event signals (yellow)
    Signal,
    /// Power/enable lines (red)
    Power,
}

impl WireType {
    pub fn get_color(&self) -> Color {
        match self {
            WireType::Data => Color::new(0.2, 0.6, 1.0, 1.0),    // Blue
            WireType::Signal => Color::new(1.0, 0.9, 0.2, 1.0),   // Yellow
            WireType::Power => Color::new(1.0, 0.3, 0.2, 1.0),    // Red
        }
    }
}

/// State of a logic node
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NodeState {
    Idle,
    Running,
    Triggered,
    Finished,
    Error,
}

impl Default for LogicSystem {
    fn default() -> Self {
        Self::new()
    }
}

// Enhanced 3D visual connection system

/// Visual representation of a wire in 3D space
#[derive(Debug, Clone)]
pub struct VisualWire {
    pub connection: LogicConnection,
    pub control_points: Vec<Vec3>,
    pub curve_segments: u32,
    pub animation_phase: f32,
    pub data_flow_particles: Vec<DataFlowParticle>,
    pub thickness: f32,
    pub glow_intensity: f32,
}

impl VisualWire {
    pub fn new(connection: LogicConnection, start_pos: Vec3, end_pos: Vec3) -> Self {
        let control_points = Self::generate_curve_points(start_pos, end_pos);

        Self {
            connection,
            control_points,
            curve_segments: 32,
            animation_phase: 0.0,
            data_flow_particles: Vec::new(),
            thickness: 0.05,
            glow_intensity: 1.0,
        }
    }

    /// Generate Bezier curve control points for natural wire sag
    fn generate_curve_points(start: Vec3, end: Vec3) -> Vec<Vec3> {
        let direction = end - start;
        let distance = direction.magnitude();
        let midpoint = start + direction * 0.5;

        // Add natural sag based on distance
        let sag_amount = (distance * 0.1).min(2.0).max(0.2);
        let control1 = start + Vec3::new(direction.x * 0.3, -sag_amount, direction.z * 0.3);
        let control2 = end + Vec3::new(-direction.x * 0.3, -sag_amount, -direction.z * 0.3);

        vec![start, control1, control2, end]
    }

    /// Update wire animation and data flow particles
    pub fn update(&mut self, delta_time: f32, data_flowing: bool) {
        self.animation_phase += delta_time * 2.0;
        if self.animation_phase > std::f32::consts::PI * 2.0 {
            self.animation_phase -= std::f32::consts::PI * 2.0;
        }

        // Update glow based on activity
        let target_glow = if data_flowing { 1.5 } else { 0.8 };
        self.glow_intensity = self.glow_intensity * 0.95 + target_glow * 0.05;

        // Update data flow particles
        self.update_data_particles(delta_time, data_flowing);
    }

    fn update_data_particles(&mut self, delta_time: f32, data_flowing: bool) {
        // Remove expired particles
        self.data_flow_particles.retain(|p| p.life > 0.0);

        // Update existing particles
        for particle in &mut self.data_flow_particles {
            particle.update(delta_time);
        }

        // Spawn new particles if data is flowing
        if data_flowing && rand::thread_rng().gen::<f32>() < 0.1 {
            self.spawn_data_particle();
        }
    }

    fn spawn_data_particle(&mut self) {
        let particle = DataFlowParticle {
            position: self.control_points[0],
            curve_progress: 0.0,
            speed: 2.0 + rand::thread_rng().gen::<f32>(),
            life: 3.0,
            max_life: 3.0,
            color: self.connection.wire_type.get_color(),
        };
        self.data_flow_particles.push(particle);
    }

    /// Get position along the curve at normalized t (0.0 to 1.0)
    pub fn get_curve_position(&self, t: f32) -> Vec3 {
        let t = t.clamp(0.0, 1.0);

        // Cubic Bezier curve calculation
        let p0 = self.control_points[0];
        let p1 = self.control_points[1];
        let p2 = self.control_points[2];
        let p3 = self.control_points[3];

        let inv_t = 1.0 - t;
        let inv_t2 = inv_t * inv_t;
        let inv_t3 = inv_t2 * inv_t;
        let t2 = t * t;
        let t3 = t2 * t;

        p0 * inv_t3 + p1 * (3.0 * inv_t2 * t) + p2 * (3.0 * inv_t * t2) + p3 * t3
    }
}

/// Data flow particle for visualizing information flow
#[derive(Debug, Clone)]
pub struct DataFlowParticle {
    pub position: Vec3,
    pub curve_progress: f32,
    pub speed: f32,
    pub life: f32,
    pub max_life: f32,
    pub color: Color,
}

impl DataFlowParticle {
    pub fn update(&mut self, delta_time: f32) {
        self.curve_progress += self.speed * delta_time;
        self.life -= delta_time;

        // Fade out over time
        let life_ratio = (self.life / self.max_life).max(0.0);
        self.color.a = life_ratio;
    }
}

/// 3D node visualization and interaction
#[derive(Debug, Clone)]
pub struct NodeVisual {
    pub node_id: u32,
    pub position: Vec3,
    pub scale: f32,
    pub rotation: f32,
    pub color: Color,
    pub glow_intensity: f32,
    pub input_ports: Vec<NodePort>,
    pub output_ports: Vec<NodePort>,
    pub hover_state: f32,
    pub selection_state: f32,
    pub animation_phase: f32,
}

#[derive(Debug, Clone)]
pub struct NodePort {
    pub name: String,
    pub relative_position: Vec3,
    pub port_type: PortType,
    pub connected: bool,
    pub value_preview: Option<LogicValue>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PortType {
    Input,
    Output,
}

impl NodeVisual {
    pub fn new(node: &LogicNode) -> Self {
        let mut visual = Self {
            node_id: node.id,
            position: node.position,
            scale: 1.0,
            rotation: 0.0,
            color: Self::get_node_color(&node.node_type),
            glow_intensity: 0.5,
            input_ports: Vec::new(),
            output_ports: Vec::new(),
            hover_state: 0.0,
            selection_state: 0.0,
            animation_phase: 0.0,
        };

        visual.setup_ports(node);
        visual
    }

    fn get_node_color(node_type: &LogicNodeType) -> Color {
        match node_type {
            LogicNodeType::Sensor(_) => Color::new(0.3, 0.8, 0.3, 1.0),      // Green
            LogicNodeType::Condition(_) => Color::new(0.8, 0.6, 0.2, 1.0),   // Orange
            LogicNodeType::Action(_) => Color::new(0.8, 0.3, 0.3, 1.0),      // Red
            LogicNodeType::Variable(_) => Color::new(0.3, 0.3, 0.8, 1.0),    // Blue
            LogicNodeType::Math(_) => Color::new(0.6, 0.3, 0.8, 1.0),        // Purple
        }
    }

    fn setup_ports(&mut self, node: &LogicNode) {
        // Create input ports
        let input_count = node.inputs.len();
        for (i, (name, _)) in node.inputs.iter().enumerate() {
            let angle = if input_count > 1 {
                -std::f32::consts::PI * 0.5 + (i as f32 / (input_count - 1) as f32) * std::f32::consts::PI
            } else {
                -std::f32::consts::PI * 0.5
            };

            let port = NodePort {
                name: name.clone(),
                relative_position: Vec3::new(angle.cos(), 0.0, angle.sin()) * 0.6,
                port_type: PortType::Input,
                connected: false,
                value_preview: None,
            };
            self.input_ports.push(port);
        }

        // Create output ports
        let output_count = node.outputs.len();
        for (i, (name, _)) in node.outputs.iter().enumerate() {
            let angle = if output_count > 1 {
                std::f32::consts::PI * 0.5 + (i as f32 / (output_count - 1) as f32) * std::f32::consts::PI
            } else {
                std::f32::consts::PI * 0.5
            };

            let port = NodePort {
                name: name.clone(),
                relative_position: Vec3::new(angle.cos(), 0.0, angle.sin()) * 0.6,
                port_type: PortType::Output,
                connected: false,
                value_preview: None,
            };
            self.output_ports.push(port);
        }
    }

    pub fn update(&mut self, delta_time: f32, is_hovered: bool, is_selected: bool) {
        // Update animation phase
        self.animation_phase += delta_time;

        // Update hover state
        let target_hover = if is_hovered { 1.0 } else { 0.0 };
        self.hover_state = self.hover_state * 0.9 + target_hover * 0.1;

        // Update selection state
        let target_selection = if is_selected { 1.0 } else { 0.0 };
        self.selection_state = self.selection_state * 0.9 + target_selection * 0.1;

        // Update glow based on state
        self.glow_intensity = 0.5 + self.hover_state * 0.3 + self.selection_state * 0.5;

        // Animate scale for feedback
        self.scale = 1.0 + self.hover_state * 0.1 + self.selection_state * 0.05;
    }

    pub fn get_port_world_position(&self, port_name: &str, port_type: PortType) -> Option<Vec3> {
        let ports = match port_type {
            PortType::Input => &self.input_ports,
            PortType::Output => &self.output_ports,
        };

        ports.iter()
            .find(|p| p.name == port_name)
            .map(|port| self.position + port.relative_position * self.scale)
    }
}

/// Enhanced logic system with 3D visualization
pub struct VisualLogicSystem {
    pub logic_system: LogicSystem,
    pub node_visuals: HashMap<u32, NodeVisual>,
    pub visual_wires: Vec<VisualWire>,
    pub selected_nodes: Vec<u32>,
    pub hovered_node: Option<u32>,
    pub active_connection: Option<ActiveLogicConnection>,
    pub debug_info_visible: bool,
}

#[derive(Debug, Clone)]
pub struct ActiveLogicConnection {
    pub start_node: u32,
    pub start_port: String,
    pub current_position: Vec3,
    pub wire_type: WireType,
}

impl VisualLogicSystem {
    pub fn new() -> Self {
        Self {
            logic_system: LogicSystem::new(),
            node_visuals: HashMap::new(),
            visual_wires: Vec::new(),
            selected_nodes: Vec::new(),
            hovered_node: None,
            active_connection: None,
            debug_info_visible: true,
        }
    }

    pub fn update(&mut self, delta_time: f32, input: &InputManager) -> RobinResult<()> {
        // Update base logic system
        self.logic_system.update(delta_time, input)?;

        // Update visual elements
        self.update_node_visuals(delta_time);
        self.update_visual_wires(delta_time);

        Ok(())
    }

    fn update_node_visuals(&mut self, delta_time: f32) {
        for (&node_id, visual) in &mut self.node_visuals {
            let is_hovered = self.hovered_node == Some(node_id);
            let is_selected = self.selected_nodes.contains(&node_id);
            visual.update(delta_time, is_hovered, is_selected);
        }
    }

    fn update_visual_wires(&mut self, delta_time: f32) {
        // First, collect the data flow status for each wire to avoid borrow conflicts
        let wire_data_flows: Vec<bool> = self.visual_wires
            .iter()
            .map(|wire| self.is_connection_active(&wire.connection))
            .collect();

        // Then update each wire with its corresponding data flow status
        for (wire, data_flowing) in self.visual_wires.iter_mut().zip(wire_data_flows.iter()) {
            wire.update(delta_time, *data_flowing);
        }
    }

    fn is_connection_active(&self, connection: &LogicConnection) -> bool {
        // Check if the source node has active output
        if let Some(source_node) = self.logic_system.get_node(connection.from_node) {
            if let Some(output_value) = source_node.get_output(&connection.from_output) {
                return output_value.as_bool();
            }
        }
        false
    }

    pub fn create_node(&mut self, node_type: LogicNodeType, position: Vec3) -> u32 {
        let node_id = self.logic_system.create_node(node_type, position);

        // Create visual representation
        if let Some(node) = self.logic_system.get_node(node_id) {
            let visual = NodeVisual::new(node);
            self.node_visuals.insert(node_id, visual);
        }

        node_id
    }

    pub fn connect_nodes_visual(
        &mut self,
        from_node: u32,
        from_output: String,
        to_node: u32,
        to_input: String,
    ) -> RobinResult<()> {
        // Create logical connection
        self.logic_system.connect_nodes(from_node, from_output.clone(), to_node, to_input.clone())?;

        // Create visual wire
        if let (Some(from_visual), Some(to_visual)) =
            (self.node_visuals.get(&from_node), self.node_visuals.get(&to_node)) {

            let start_pos = from_visual.get_port_world_position(&from_output, PortType::Output)
                .unwrap_or(from_visual.position);
            let end_pos = to_visual.get_port_world_position(&to_input, PortType::Input)
                .unwrap_or(to_visual.position);

            let connection = LogicConnection {
                from_node,
                from_output,
                to_node,
                to_input,
                wire_type: WireType::Data,
                active: true,
            };

            let visual_wire = VisualWire::new(connection, start_pos, end_pos);
            self.visual_wires.push(visual_wire);
        }

        Ok(())
    }

    pub fn remove_node(&mut self, node_id: u32) {
        self.logic_system.remove_node(node_id);
        self.node_visuals.remove(&node_id);
        self.visual_wires.retain(|wire|
            wire.connection.from_node != node_id && wire.connection.to_node != node_id
        );
        self.selected_nodes.retain(|&id| id != node_id);
    }
}