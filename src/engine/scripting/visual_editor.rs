// Visual Script Editor for Robin Engine
// Provides drag-and-drop node-based programming interface for Engineer Build Mode

use crate::engine::error::RobinResult;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    pub grid_size: f32,
    pub snap_to_grid: bool,
    pub auto_arrange: bool,
    pub max_nodes_per_graph: u32,
    pub max_connections_per_pin: u32,
    pub enable_validation: bool,
    pub show_data_types: bool,
    pub enable_minimap: bool,
    pub auto_save_interval: f32,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            grid_size: 20.0,
            snap_to_grid: true,
            auto_arrange: false,
            max_nodes_per_graph: 1000,
            max_connections_per_pin: 100,
            enable_validation: true,
            show_data_types: true,
            enable_minimap: true,
            auto_save_interval: 30.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum NodeType {
    // Flow Control
    Start,
    End,
    Branch,
    Loop,
    ForEach,
    Switch,
    Delay,
    Sequence,

    // Category-level nodes for generic operations
    Math,       // Generic math operation (operation specified in properties)
    Logic,      // Generic logic operation (operation specified in properties)
    Condition,  // Generic condition node (condition specified in properties)
    Control,    // Generic control flow node (control type specified in properties)

    // Specific Logic operations
    And,
    Or,
    Not,
    Equal,
    NotEqual,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,

    // Specific Math operations
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,
    SquareRoot,
    Absolute,
    Random,
    Clamp,
    Lerp,
    
    // Data
    Variable,
    Constant,
    Array,
    GetProperty,
    SetProperty,
    ToString,
    ToNumber,
    ToBool,
    
    // Building & Construction
    PlaceBlock,
    RemoveBlock,
    GetBlockType,
    SetBlockType,
    GetBlockProperty,
    SetBlockProperty,
    FindBlocks,
    CountBlocks,
    
    // Player Interaction
    GetPlayerPosition,
    SetPlayerPosition,
    GetPlayerInput,
    SendMessage,
    ShowUI,
    HideUI,
    PlaySound,
    
    // AI & NPCs
    MoveNPC,
    SetNPCBehavior,
    GetNPCState,
    NPCSpeak,
    NPCFollow,
    NPCPatrol,
    
    // Events
    OnTrigger,
    OnCollision,
    OnTimer,
    OnInput,
    OnBuildingComplete,
    OnPlayerAction,
    TriggerEvent,
    
    // Utilities
    Debug,
    Comment,
    Reroute,
    SubGraph,
    
    // Custom
    Custom { 
        name: String,
        category: NodeCategory,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum NodeCategory {
    FlowControl,
    Logic,
    Math,
    Data,
    Building,
    Player,
    AI,
    Events,
    Utilities,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum PinType {
    Execution,
    Boolean,
    Integer,
    Float,
    String,
    Vector3,
    Color,
    Object,
    Array,
    Any,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePin {
    pub id: String,
    pub name: String,
    pub pin_type: PinType,
    pub is_input: bool,
    pub default_value: Option<String>,
    pub description: String,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptNode {
    pub id: String,
    pub node_type: NodeType,
    pub name: String,
    pub description: String,
    pub position: (f32, f32),
    pub input_pins: Vec<NodePin>,
    pub output_pins: Vec<NodePin>,
    pub properties: HashMap<String, String>,
    pub category: NodeCategory,
    pub color: String,
    pub size: (f32, f32),
    pub collapsed: bool,
    pub enabled: bool,
    pub breakpoint: bool,
}

impl ScriptNode {
    pub fn new(id: String, node_type: NodeType, position: (f32, f32)) -> Self {
        let (name, description, input_pins, output_pins, category, color) = Self::get_node_info(&node_type);
        
        Self {
            id,
            node_type,
            name,
            description,
            position,
            input_pins,
            output_pins,
            properties: HashMap::new(),
            category,
            color,
            size: (120.0, 80.0),
            collapsed: false,
            enabled: true,
            breakpoint: false,
        }
    }

    fn get_node_info(node_type: &NodeType) -> (String, String, Vec<NodePin>, Vec<NodePin>, NodeCategory, String) {
        match node_type {
            NodeType::Start => (
                "Start".to_string(),
                "Entry point for script execution".to_string(),
                vec![],
                vec![NodePin {
                    id: "exec_out".to_string(),
                    name: "".to_string(),
                    pin_type: PinType::Execution,
                    is_input: false,
                    default_value: None,
                    description: "Execution output".to_string(),
                    required: false,
                }],
                NodeCategory::FlowControl,
                "#4CAF50".to_string(),
            ),
            NodeType::End => (
                "End".to_string(),
                "Exit point for script execution".to_string(),
                vec![NodePin {
                    id: "exec_in".to_string(),
                    name: "".to_string(),
                    pin_type: PinType::Execution,
                    is_input: true,
                    default_value: None,
                    description: "Execution input".to_string(),
                    required: true,
                }],
                vec![],
                NodeCategory::FlowControl,
                "#F44336".to_string(),
            ),
            NodeType::Branch => (
                "Branch".to_string(),
                "Conditional execution based on boolean input".to_string(),
                vec![
                    NodePin {
                        id: "exec_in".to_string(),
                        name: "".to_string(),
                        pin_type: PinType::Execution,
                        is_input: true,
                        default_value: None,
                        description: "Execution input".to_string(),
                        required: true,
                    },
                    NodePin {
                        id: "condition".to_string(),
                        name: "Condition".to_string(),
                        pin_type: PinType::Boolean,
                        is_input: true,
                        default_value: Some("false".to_string()),
                        description: "Boolean condition to evaluate".to_string(),
                        required: true,
                    },
                ],
                vec![
                    NodePin {
                        id: "true_exec".to_string(),
                        name: "True".to_string(),
                        pin_type: PinType::Execution,
                        is_input: false,
                        default_value: None,
                        description: "Execute if condition is true".to_string(),
                        required: false,
                    },
                    NodePin {
                        id: "false_exec".to_string(),
                        name: "False".to_string(),
                        pin_type: PinType::Execution,
                        is_input: false,
                        default_value: None,
                        description: "Execute if condition is false".to_string(),
                        required: false,
                    },
                ],
                NodeCategory::FlowControl,
                "#FF9800".to_string(),
            ),
            NodeType::Add => (
                "Add".to_string(),
                "Add two numbers together".to_string(),
                vec![
                    NodePin {
                        id: "a".to_string(),
                        name: "A".to_string(),
                        pin_type: PinType::Float,
                        is_input: true,
                        default_value: Some("0".to_string()),
                        description: "First number".to_string(),
                        required: true,
                    },
                    NodePin {
                        id: "b".to_string(),
                        name: "B".to_string(),
                        pin_type: PinType::Float,
                        is_input: true,
                        default_value: Some("0".to_string()),
                        description: "Second number".to_string(),
                        required: true,
                    },
                ],
                vec![
                    NodePin {
                        id: "result".to_string(),
                        name: "Result".to_string(),
                        pin_type: PinType::Float,
                        is_input: false,
                        default_value: None,
                        description: "Sum of A and B".to_string(),
                        required: false,
                    },
                ],
                NodeCategory::Math,
                "#2196F3".to_string(),
            ),
            NodeType::PlaceBlock => (
                "Place Block".to_string(),
                "Place a block at the specified position".to_string(),
                vec![
                    NodePin {
                        id: "exec_in".to_string(),
                        name: "".to_string(),
                        pin_type: PinType::Execution,
                        is_input: true,
                        default_value: None,
                        description: "Execution input".to_string(),
                        required: true,
                    },
                    NodePin {
                        id: "position".to_string(),
                        name: "Position".to_string(),
                        pin_type: PinType::Vector3,
                        is_input: true,
                        default_value: Some("0,0,0".to_string()),
                        description: "World position to place block".to_string(),
                        required: true,
                    },
                    NodePin {
                        id: "block_type".to_string(),
                        name: "Block Type".to_string(),
                        pin_type: PinType::String,
                        is_input: true,
                        default_value: Some("stone".to_string()),
                        description: "Type of block to place".to_string(),
                        required: true,
                    },
                ],
                vec![
                    NodePin {
                        id: "exec_out".to_string(),
                        name: "".to_string(),
                        pin_type: PinType::Execution,
                        is_input: false,
                        default_value: None,
                        description: "Execution output".to_string(),
                        required: false,
                    },
                    NodePin {
                        id: "success".to_string(),
                        name: "Success".to_string(),
                        pin_type: PinType::Boolean,
                        is_input: false,
                        default_value: None,
                        description: "True if block was placed successfully".to_string(),
                        required: false,
                    },
                ],
                NodeCategory::Building,
                "#8BC34A".to_string(),
            ),
            NodeType::GetPlayerPosition => (
                "Get Player Position".to_string(),
                "Get the current player's world position".to_string(),
                vec![],
                vec![
                    NodePin {
                        id: "position".to_string(),
                        name: "Position".to_string(),
                        pin_type: PinType::Vector3,
                        is_input: false,
                        default_value: None,
                        description: "Player's current position".to_string(),
                        required: false,
                    },
                ],
                NodeCategory::Player,
                "#9C27B0".to_string(),
            ),
            NodeType::OnTrigger => (
                "On Trigger".to_string(),
                "Execute when a trigger event occurs".to_string(),
                vec![
                    NodePin {
                        id: "trigger_name".to_string(),
                        name: "Trigger".to_string(),
                        pin_type: PinType::String,
                        is_input: true,
                        default_value: Some("trigger".to_string()),
                        description: "Name of the trigger to listen for".to_string(),
                        required: true,
                    },
                ],
                vec![
                    NodePin {
                        id: "exec_out".to_string(),
                        name: "".to_string(),
                        pin_type: PinType::Execution,
                        is_input: false,
                        default_value: None,
                        description: "Execution output when triggered".to_string(),
                        required: false,
                    },
                    NodePin {
                        id: "trigger_data".to_string(),
                        name: "Data".to_string(),
                        pin_type: PinType::Object,
                        is_input: false,
                        default_value: None,
                        description: "Data passed with the trigger".to_string(),
                        required: false,
                    },
                ],
                NodeCategory::Events,
                "#FF5722".to_string(),
            ),
            NodeType::Debug => (
                "Debug".to_string(),
                "Output debug information to console".to_string(),
                vec![
                    NodePin {
                        id: "exec_in".to_string(),
                        name: "".to_string(),
                        pin_type: PinType::Execution,
                        is_input: true,
                        default_value: None,
                        description: "Execution input".to_string(),
                        required: true,
                    },
                    NodePin {
                        id: "message".to_string(),
                        name: "Message".to_string(),
                        pin_type: PinType::String,
                        is_input: true,
                        default_value: Some("Debug message".to_string()),
                        description: "Message to output".to_string(),
                        required: true,
                    },
                ],
                vec![
                    NodePin {
                        id: "exec_out".to_string(),
                        name: "".to_string(),
                        pin_type: PinType::Execution,
                        is_input: false,
                        default_value: None,
                        description: "Execution output".to_string(),
                        required: false,
                    },
                ],
                NodeCategory::Utilities,
                "#607D8B".to_string(),
            ),
            _ => (
                "Unknown".to_string(),
                "Unknown node type".to_string(),
                vec![],
                vec![],
                NodeCategory::Custom,
                "#9E9E9E".to_string(),
            ),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConnection {
    pub id: String,
    pub from_node_id: String,
    pub from_pin_id: String,
    pub to_node_id: String,
    pub to_pin_id: String,
    pub connection_type: PinType,
    pub valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeGraph {
    pub id: String,
    pub name: String,
    pub description: String,
    pub nodes: HashMap<String, ScriptNode>,
    pub connections: HashMap<String, NodeConnection>,
    pub variables: HashMap<String, GraphVariable>,
    pub created_at: SystemTime,
    pub modified_at: SystemTime,
    pub author: String,
    pub version: u32,
    pub tags: Vec<String>,
    pub graph_properties: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphVariable {
    pub name: String,
    pub var_type: PinType,
    pub default_value: String,
    pub description: String,
    pub is_input: bool,
    pub is_output: bool,
}

impl NodeGraph {
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            description: String::new(),
            nodes: HashMap::new(),
            connections: HashMap::new(),
            variables: HashMap::new(),
            created_at: SystemTime::now(),
            modified_at: SystemTime::now(),
            author: "Engineer".to_string(),
            version: 1,
            tags: Vec::new(),
            graph_properties: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: ScriptNode) {
        self.nodes.insert(node.id.clone(), node);
        self.modified_at = SystemTime::now();
        self.version += 1;
    }

    pub fn remove_node(&mut self, node_id: &str) -> bool {
        if self.nodes.remove(node_id).is_some() {
            // Remove all connections to/from this node
            let connections_to_remove: Vec<String> = self.connections
                .iter()
                .filter(|(_, conn)| conn.from_node_id == node_id || conn.to_node_id == node_id)
                .map(|(id, _)| id.clone())
                .collect();

            for conn_id in connections_to_remove {
                self.connections.remove(&conn_id);
            }

            self.modified_at = SystemTime::now();
            self.version += 1;
            true
        } else {
            false
        }
    }

    pub fn add_connection(&mut self, connection: NodeConnection) -> bool {
        if self.validate_connection(&connection) {
            self.connections.insert(connection.id.clone(), connection);
            self.modified_at = SystemTime::now();
            self.version += 1;
            true
        } else {
            false
        }
    }

    pub fn validate_connection(&self, connection: &NodeConnection) -> bool {
        // Check if nodes exist
        let from_node = match self.nodes.get(&connection.from_node_id) {
            Some(node) => node,
            None => return false,
        };

        let to_node = match self.nodes.get(&connection.to_node_id) {
            Some(node) => node,
            None => return false,
        };

        // Check if pins exist
        let from_pin = from_node.output_pins.iter().find(|p| p.id == connection.from_pin_id);
        let to_pin = to_node.input_pins.iter().find(|p| p.id == connection.to_pin_id);

        if from_pin.is_none() || to_pin.is_none() {
            return false;
        }

        let from_pin = from_pin.unwrap();
        let to_pin = to_pin.unwrap();

        // Check type compatibility
        self.are_types_compatible(&from_pin.pin_type, &to_pin.pin_type)
    }

    fn are_types_compatible(&self, from_type: &PinType, to_type: &PinType) -> bool {
        match (from_type, to_type) {
            (PinType::Any, _) | (_, PinType::Any) => true,
            (a, b) if a == b => true,
            (PinType::Integer, PinType::Float) => true,
            (PinType::Float, PinType::Integer) => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct NodeLibrary {
    pub categories: HashMap<NodeCategory, Vec<NodeType>>,
    pub custom_nodes: HashMap<String, NodeType>,
    pub favorites: Vec<NodeType>,
    pub recent: Vec<NodeType>,
}

impl NodeLibrary {
    pub fn new() -> Self {
        let mut library = Self {
            categories: HashMap::new(),
            custom_nodes: HashMap::new(),
            favorites: Vec::new(),
            recent: Vec::new(),
        };

        library.initialize_default_nodes();
        library
    }

    fn initialize_default_nodes(&mut self) {
        // Flow Control
        self.categories.insert(NodeCategory::FlowControl, vec![
            NodeType::Start, NodeType::End, NodeType::Branch, NodeType::Loop,
            NodeType::ForEach, NodeType::Switch, NodeType::Delay, NodeType::Sequence,
        ]);

        // Logic
        self.categories.insert(NodeCategory::Logic, vec![
            NodeType::And, NodeType::Or, NodeType::Not, NodeType::Equal,
            NodeType::NotEqual, NodeType::Greater, NodeType::Less,
            NodeType::GreaterEqual, NodeType::LessEqual,
        ]);

        // Math
        self.categories.insert(NodeCategory::Math, vec![
            NodeType::Add, NodeType::Subtract, NodeType::Multiply, NodeType::Divide,
            NodeType::Modulo, NodeType::Power, NodeType::SquareRoot, NodeType::Absolute,
            NodeType::Random, NodeType::Clamp, NodeType::Lerp,
        ]);

        // Data
        self.categories.insert(NodeCategory::Data, vec![
            NodeType::Variable, NodeType::Constant, NodeType::Array,
            NodeType::GetProperty, NodeType::SetProperty, NodeType::ToString,
            NodeType::ToNumber, NodeType::ToBool,
        ]);

        // Building
        self.categories.insert(NodeCategory::Building, vec![
            NodeType::PlaceBlock, NodeType::RemoveBlock, NodeType::GetBlockType,
            NodeType::SetBlockType, NodeType::GetBlockProperty, NodeType::SetBlockProperty,
            NodeType::FindBlocks, NodeType::CountBlocks,
        ]);

        // Player
        self.categories.insert(NodeCategory::Player, vec![
            NodeType::GetPlayerPosition, NodeType::SetPlayerPosition, NodeType::GetPlayerInput,
            NodeType::SendMessage, NodeType::ShowUI, NodeType::HideUI, NodeType::PlaySound,
        ]);

        // AI
        self.categories.insert(NodeCategory::AI, vec![
            NodeType::MoveNPC, NodeType::SetNPCBehavior, NodeType::GetNPCState,
            NodeType::NPCSpeak, NodeType::NPCFollow, NodeType::NPCPatrol,
        ]);

        // Events
        self.categories.insert(NodeCategory::Events, vec![
            NodeType::OnTrigger, NodeType::OnCollision, NodeType::OnTimer,
            NodeType::OnInput, NodeType::OnBuildingComplete, NodeType::OnPlayerAction,
            NodeType::TriggerEvent,
        ]);

        // Utilities
        self.categories.insert(NodeCategory::Utilities, vec![
            NodeType::Debug, NodeType::Comment, NodeType::Reroute, NodeType::SubGraph,
        ]);
    }

    pub fn get_nodes_by_category(&self, category: &NodeCategory) -> Vec<&NodeType> {
        self.categories.get(category).map(|nodes| nodes.iter().collect()).unwrap_or_default()
    }

    pub fn search_nodes(&self, query: &str) -> Vec<&NodeType> {
        let query = query.to_lowercase();
        let mut results = Vec::new();

        for nodes in self.categories.values() {
            for node_type in nodes {
                if format!("{:?}", node_type).to_lowercase().contains(&query) {
                    results.push(node_type);
                }
            }
        }

        results
    }

    pub fn add_to_recent(&mut self, node_type: NodeType) {
        // Remove if already in recent
        self.recent.retain(|n| n != &node_type);
        
        // Add to front
        self.recent.insert(0, node_type);
        
        // Keep only last 10
        if self.recent.len() > 10 {
            self.recent.truncate(10);
        }
    }
}

#[derive(Debug)]
pub struct VisualScriptEditor {
    config: EditorConfig,
    graphs: HashMap<String, NodeGraph>,
    active_graph_id: Option<String>,
    node_library: NodeLibrary,
    next_node_id: u32,
    next_connection_id: u32,
    next_graph_id: u32,
    clipboard: Option<Vec<ScriptNode>>,
    undo_stack: Vec<EditorAction>,
    redo_stack: Vec<EditorAction>,
    selection: Vec<String>,
    last_auto_save: SystemTime,
}

#[derive(Debug, Clone)]
enum EditorAction {
    AddNode(ScriptNode),
    RemoveNode(String),
    MoveNode(String, (f32, f32), (f32, f32)),
    AddConnection(NodeConnection),
    RemoveConnection(String),
    ModifyNodeProperties(String, HashMap<String, String>),
}

impl VisualScriptEditor {
    pub fn new(config: EditorConfig) -> RobinResult<Self> {
        Ok(Self {
            config,
            graphs: HashMap::new(),
            active_graph_id: None,
            node_library: NodeLibrary::new(),
            next_node_id: 1,
            next_connection_id: 1,
            next_graph_id: 1,
            clipboard: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            selection: Vec::new(),
            last_auto_save: SystemTime::now(),
        })
    }

    pub fn initialize(&mut self) -> RobinResult<()> {
        println!("Visual Script Editor initialized:");
        println!("  Node categories: {}", self.node_library.categories.len());
        println!("  Grid size: {}", self.config.grid_size);
        println!("  Snap to grid: {}", self.config.snap_to_grid);
        println!("  Auto arrange: {}", self.config.auto_arrange);
        Ok(())
    }

    pub fn update(&mut self, delta_time: f32) -> RobinResult<()> {
        // Auto-save check
        if self.last_auto_save.elapsed().unwrap_or_default().as_secs_f32() >= self.config.auto_save_interval {
            self.auto_save_modified_graphs()?;
            self.last_auto_save = SystemTime::now();
        }
        Ok(())
    }

    pub fn create_new_graph(&mut self, name: String) -> RobinResult<String> {
        let graph_id = format!("graph_{}", self.next_graph_id);
        self.next_graph_id += 1;

        let mut graph = NodeGraph::new(graph_id.clone(), name);
        
        // Add default Start node
        let start_node = ScriptNode::new(
            format!("node_{}", self.next_node_id),
            NodeType::Start,
            (100.0, 100.0)
        );
        self.next_node_id += 1;
        
        graph.add_node(start_node);
        
        self.graphs.insert(graph_id.clone(), graph);
        self.active_graph_id = Some(graph_id.clone());

        Ok(graph_id)
    }

    pub fn add_node(&mut self, graph_id: &str, node_type: NodeType, position: (f32, f32)) -> RobinResult<String> {
        let graph = self.graphs.get_mut(graph_id)
            .ok_or_else(|| crate::engine::error::RobinError::new(&format!("Graph '{}' not found", graph_id)))?;

        if graph.nodes.len() >= self.config.max_nodes_per_graph as usize {
            return Err(crate::engine::error::RobinError::new("Maximum nodes per graph exceeded"));
        }

        let node_id = format!("node_{}", self.next_node_id);
        self.next_node_id += 1;

        let adjusted_position = if self.config.snap_to_grid {
            (
                (position.0 / self.config.grid_size).round() * self.config.grid_size,
                (position.1 / self.config.grid_size).round() * self.config.grid_size,
            )
        } else {
            position
        };

        let node = ScriptNode::new(node_id.clone(), node_type.clone(), adjusted_position);
        
        // Record action for undo
        self.undo_stack.push(EditorAction::AddNode(node.clone()));
        self.redo_stack.clear();

        graph.add_node(node);
        
        // Add to recent nodes
        self.node_library.add_to_recent(node_type);

        Ok(node_id)
    }

    pub fn remove_node(&mut self, graph_id: &str, node_id: &str) -> RobinResult<()> {
        let graph = self.graphs.get_mut(graph_id)
            .ok_or_else(|| crate::engine::error::RobinError::new(&format!("Graph '{}' not found", graph_id)))?;

        if let Some(node) = graph.nodes.get(node_id) {
            // Record action for undo
            self.undo_stack.push(EditorAction::RemoveNode(node_id.to_string()));
            self.redo_stack.clear();
        }

        graph.remove_node(node_id);
        Ok(())
    }

    pub fn create_connection(&mut self, graph_id: &str, from_node: &str, from_pin: &str, to_node: &str, to_pin: &str) -> RobinResult<()> {
        let graph = self.graphs.get_mut(graph_id)
            .ok_or_else(|| crate::engine::error::RobinError::new(&format!("Graph '{}' not found", graph_id)))?;

        let connection_id = format!("connection_{}", self.next_connection_id);
        self.next_connection_id += 1;

        // Determine connection type from the output pin
        let connection_type = if let Some(from_node_obj) = graph.nodes.get(from_node) {
            from_node_obj.output_pins
                .iter()
                .find(|p| p.id == from_pin)
                .map(|p| p.pin_type.clone())
                .unwrap_or(PinType::Any)
        } else {
            PinType::Any
        };

        let connection = NodeConnection {
            id: connection_id,
            from_node_id: from_node.to_string(),
            from_pin_id: from_pin.to_string(),
            to_node_id: to_node.to_string(),
            to_pin_id: to_pin.to_string(),
            connection_type,
            valid: true,
        };

        if graph.add_connection(connection.clone()) {
            // Record action for undo
            self.undo_stack.push(EditorAction::AddConnection(connection));
            self.redo_stack.clear();
            Ok(())
        } else {
            Err(crate::engine::error::RobinError::new("Invalid connection"))
        }
    }

    pub fn compile_graph(&self, graph_id: &str) -> RobinResult<String> {
        let graph = self.graphs.get(graph_id)
            .ok_or_else(|| crate::engine::error::RobinError::new(&format!("Graph '{}' not found", graph_id)))?;

        if self.config.enable_validation {
            self.validate_graph(graph)?;
        }

        // Generate executable script from the node graph
        let mut script = String::new();
        script.push_str(&format!("// Generated script for graph: {}\n", graph.name));
        script.push_str("// Generated by Robin Engine Visual Script Editor\n\n");

        // Find start nodes
        let start_nodes: Vec<_> = graph.nodes.values()
            .filter(|node| matches!(node.node_type, NodeType::Start))
            .collect();

        if start_nodes.is_empty() {
            return Err(crate::engine::error::RobinError::new("No start node found in graph"));
        }

        for start_node in start_nodes {
            script.push_str(&self.compile_execution_path(graph, &start_node.id)?);
        }

        Ok(script)
    }

    fn compile_execution_path(&self, graph: &NodeGraph, current_node_id: &str) -> RobinResult<String> {
        let mut code = String::new();
        let mut visited = std::collections::HashSet::new();
        let mut to_visit = vec![current_node_id.to_string()];

        while let Some(node_id) = to_visit.pop() {
            if visited.contains(&node_id) {
                continue;
            }
            visited.insert(node_id.clone());

            if let Some(node) = graph.nodes.get(&node_id) {
                // Generate code for this node
                match node.node_type {
                    NodeType::Debug => {
                        let default_message = "Debug".to_string();
                        let message = node.properties.get("message").unwrap_or(&default_message);
                        code.push_str(&format!("console.log({:?});\n", message));
                    },
                    NodeType::PlaceBlock => {
                        code.push_str("// Place block logic\n");
                        code.push_str("world.place_block(position, block_type);\n");
                    },
                    NodeType::Branch => {
                        code.push_str("if (condition) {\n");
                        // Find true path
                        code.push_str("  // True branch\n");
                        code.push_str("} else {\n");
                        // Find false path
                        code.push_str("  // False branch\n");
                        code.push_str("}\n");
                    },
                    _ => {
                        code.push_str(&format!("// Node: {} ({})\n", node.name, node.id));
                    }
                }

                // Find next nodes to visit
                let next_nodes: Vec<String> = graph.connections.values()
                    .filter(|conn| conn.from_node_id == node_id && conn.connection_type == PinType::Execution)
                    .map(|conn| conn.to_node_id.clone())
                    .collect();

                to_visit.extend(next_nodes);
            }
        }

        Ok(code)
    }

    fn validate_graph(&self, graph: &NodeGraph) -> RobinResult<()> {
        let mut errors = Vec::new();

        // Check for disconnected execution nodes
        for node in graph.nodes.values() {
            if !matches!(node.node_type, NodeType::Start | NodeType::OnTrigger | NodeType::OnTimer | NodeType::OnInput) {
                let has_exec_input = node.input_pins.iter().any(|pin| pin.pin_type == PinType::Execution);
                if has_exec_input {
                    let is_connected = graph.connections.values()
                        .any(|conn| conn.to_node_id == node.id && conn.connection_type == PinType::Execution);
                    
                    if !is_connected {
                        errors.push(format!("Node '{}' has no execution input connection", node.name));
                    }
                }
            }
        }

        // Check for required inputs
        for node in graph.nodes.values() {
            for pin in &node.input_pins {
                if pin.required && pin.default_value.is_none() {
                    let is_connected = graph.connections.values()
                        .any(|conn| conn.to_node_id == node.id && conn.to_pin_id == pin.id);
                    
                    if !is_connected {
                        errors.push(format!("Required input '{}' on node '{}' is not connected", pin.name, node.name));
                    }
                }
            }
        }

        if !errors.is_empty() {
            return Err(crate::engine::error::RobinError::new(&format!("Graph validation errors: {}", errors.join(", "))));
        }

        Ok(())
    }

    pub fn export_graph(&self, graph_id: &str) -> RobinResult<String> {
        let graph = self.graphs.get(graph_id)
            .ok_or_else(|| crate::engine::error::RobinError::new(&format!("Graph '{}' not found", graph_id)))?;

        serde_json::to_string_pretty(graph)
            .map_err(|e| crate::engine::error::RobinError::new(&format!("Serialization error: {}", e)))
    }

    pub fn import_graph(&mut self, graph_data: &str) -> RobinResult<String> {
        let graph: NodeGraph = serde_json::from_str(graph_data)
            .map_err(|e| crate::engine::error::RobinError::new(&format!("Deserialization error: {}", e)))?;

        let graph_id = graph.id.clone();
        self.graphs.insert(graph_id.clone(), graph);

        Ok(graph_id)
    }

    pub fn copy_selected_nodes(&mut self, graph_id: &str) -> RobinResult<()> {
        let graph = self.graphs.get(graph_id)
            .ok_or_else(|| crate::engine::error::RobinError::new(&format!("Graph '{}' not found", graph_id)))?;

        let selected_nodes: Vec<ScriptNode> = self.selection
            .iter()
            .filter_map(|id| graph.nodes.get(id))
            .cloned()
            .collect();

        self.clipboard = Some(selected_nodes);
        Ok(())
    }

    pub fn paste_nodes(&mut self, graph_id: &str, offset: (f32, f32)) -> RobinResult<Vec<String>> {
        if let Some(clipboard_nodes) = self.clipboard.clone() {
            let mut new_node_ids = Vec::new();

            for node in &clipboard_nodes {
                let new_position = (node.position.0 + offset.0, node.position.1 + offset.1);
                let new_id = self.add_node(graph_id, node.node_type.clone(), new_position)?;
                new_node_ids.push(new_id);
            }

            Ok(new_node_ids)
        } else {
            Ok(Vec::new())
        }
    }

    pub fn auto_arrange_graph(&mut self, graph_id: &str) -> RobinResult<()> {
        let graph = self.graphs.get_mut(graph_id)
            .ok_or_else(|| crate::engine::error::RobinError::new(&format!("Graph '{}' not found", graph_id)))?;

        // Simple auto-arrangement: arrange nodes in a grid based on execution flow
        let start_nodes: Vec<String> = graph.nodes.values()
            .filter(|node| matches!(node.node_type, NodeType::Start))
            .map(|node| node.id.clone())
            .collect();

        let mut x_offset = 100.0;
        let y_spacing = 150.0;

        for start_node_id in start_nodes {
            let mut current_y = 100.0;
            let mut visited = std::collections::HashSet::new();
            let mut to_arrange = vec![start_node_id];

            while let Some(node_id) = to_arrange.pop() {
                if visited.contains(&node_id) {
                    continue;
                }
                visited.insert(node_id.clone());

                if let Some(node) = graph.nodes.get_mut(&node_id) {
                    node.position = (x_offset, current_y);
                    current_y += y_spacing;

                    // Find next nodes
                    let next_nodes: Vec<String> = graph.connections.values()
                        .filter(|conn| conn.from_node_id == node_id && conn.connection_type == PinType::Execution)
                        .map(|conn| conn.to_node_id.clone())
                        .collect();

                    to_arrange.extend(next_nodes);
                }
            }

            x_offset += 300.0;
        }

        Ok(())
    }

    pub fn undo(&mut self) -> RobinResult<()> {
        if let Some(action) = self.undo_stack.pop() {
            match action {
                EditorAction::AddNode(node) => {
                    if let Some(graph_id) = &self.active_graph_id {
                        if let Some(graph) = self.graphs.get_mut(graph_id) {
                            graph.remove_node(&node.id);
                            self.redo_stack.push(EditorAction::RemoveNode(node.id));
                        }
                    }
                },
                EditorAction::RemoveNode(node_id) => {
                    // This would need to restore the node, which requires storing more state
                    // For now, just add to redo stack
                    self.redo_stack.push(EditorAction::AddNode(ScriptNode::new(node_id, NodeType::Debug, (0.0, 0.0))));
                },
                _ => {
                    // Handle other actions
                }
            }
        }
        Ok(())
    }

    pub fn redo(&mut self) -> RobinResult<()> {
        // Similar implementation to undo but in reverse
        Ok(())
    }

    pub fn auto_save_modified_graphs(&mut self) -> RobinResult<()> {
        // In a real implementation, this would save to disk
        println!("Auto-saving {} modified graphs", self.graphs.len());
        Ok(())
    }

    pub fn shutdown(&mut self) -> RobinResult<()> {
        println!("Visual Script Editor shutdown:");
        println!("  Graphs created: {}", self.graphs.len());
        println!("  Nodes created: {}", self.next_node_id - 1);
        println!("  Connections created: {}", self.next_connection_id - 1);
        
        Ok(())
    }
}