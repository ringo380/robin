// Behavior Tree System for Robin Engine
// Provides sophisticated AI behavior management for NPCs and game systems

use crate::engine::error::RobinResult;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeStatus {
    Success,
    Failure,
    Running,
    Invalid,
}

// Alias for backward compatibility
pub type NodeState = NodeStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BehaviorNodeType {
    // Composite nodes
    Sequence,
    Selector,
    Parallel,
    
    // Decorator nodes
    Inverter,
    Repeater,
    Timer,
    Condition,
    
    // Action nodes
    Action(String),
    MoveTo,
    Attack,
    Patrol,
    Idle,
    Interact,
    
    // Custom nodes
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeConfig {
    pub max_execution_time_ms: u32,
    pub max_depth: u32,
    pub enable_blackboard_sharing: bool,
    pub enable_debugging: bool,
    pub tick_rate_hz: f32,
    pub memory_limit_mb: u32,
}

impl Default for TreeConfig {
    fn default() -> Self {
        Self {
            max_execution_time_ms: 16,
            max_depth: 20,
            enable_blackboard_sharing: true,
            enable_debugging: true,
            tick_rate_hz: 30.0,
            memory_limit_mb: 64,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeValue {
    Bool(bool),
    Int(i32),
    Float(f32),
    String(String),
    Vector3([f32; 3]),
    Array(Vec<RuntimeValue>),
    Object(HashMap<String, RuntimeValue>),
    None,
}

impl RuntimeValue {
    pub fn as_bool(&self) -> bool {
        match self {
            RuntimeValue::Bool(b) => *b,
            RuntimeValue::Int(i) => *i != 0,
            RuntimeValue::Float(f) => *f != 0.0,
            RuntimeValue::String(s) => !s.is_empty(),
            RuntimeValue::Vector3(_) => true, // Non-zero vector is considered true
            RuntimeValue::Array(a) => !a.is_empty(),
            RuntimeValue::Object(o) => !o.is_empty(),
            RuntimeValue::None => false,
        }
    }

    pub fn as_float(&self) -> f32 {
        match self {
            RuntimeValue::Float(f) => *f,
            RuntimeValue::Int(i) => *i as f32,
            RuntimeValue::Bool(b) => if *b { 1.0 } else { 0.0 },
            _ => 0.0,
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            RuntimeValue::String(s) => s.clone(),
            RuntimeValue::Bool(b) => b.to_string(),
            RuntimeValue::Int(i) => i.to_string(),
            RuntimeValue::Float(f) => f.to_string(),
            RuntimeValue::Vector3(v) => format!("({}, {}, {})", v[0], v[1], v[2]),
            RuntimeValue::Array(_) => "[Array]".to_string(),
            RuntimeValue::Object(_) => "[Object]".to_string(),
            RuntimeValue::None => "None".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blackboard {
    data: HashMap<String, RuntimeValue>,
    shared_data: HashMap<String, RuntimeValue>,
    entity_id: String,
}

impl Blackboard {
    pub fn new(entity_id: String) -> Self {
        Self {
            data: HashMap::new(),
            shared_data: HashMap::new(),
            entity_id,
        }
    }

    pub fn get(&self, key: &str) -> Option<&RuntimeValue> {
        self.data.get(key).or_else(|| self.shared_data.get(key))
    }

    pub fn set(&mut self, key: String, value: RuntimeValue) {
        self.data.insert(key, value);
    }

    pub fn set_shared(&mut self, key: String, value: RuntimeValue) {
        self.shared_data.insert(key, value);
    }

    pub fn remove(&mut self, key: &str) -> Option<RuntimeValue> {
        self.data.remove(key)
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn has_key(&self, key: &str) -> bool {
        self.data.contains_key(key) || self.shared_data.contains_key(key)
    }

    pub fn keys(&self) -> Vec<String> {
        let mut keys: Vec<String> = self.data.keys().cloned().collect();
        keys.extend(self.shared_data.keys().cloned());
        keys.sort();
        keys.dedup();
        keys
    }
}

impl Default for Blackboard {
    fn default() -> Self {
        Self::new("default".to_string())
    }
}

pub trait BehaviorNode: std::fmt::Debug + Send + Sync {
    fn tick(&mut self, blackboard: &mut Blackboard, delta_time: f32) -> NodeStatus;
    fn reset(&mut self);
    fn get_name(&self) -> &str;
    fn get_description(&self) -> &str;
    fn is_composite(&self) -> bool { false }
    fn is_decorator(&self) -> bool { false }
    fn is_leaf(&self) -> bool { true }
}

// Composite Nodes
#[derive(Debug)]
pub struct SequenceNode {
    name: String,
    description: String,
    children: Vec<Box<dyn BehaviorNode>>,
    current_child: usize,
    status: NodeStatus,
}

impl SequenceNode {
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            children: Vec::new(),
            current_child: 0,
            status: NodeStatus::Invalid,
        }
    }

    pub fn add_child(&mut self, child: Box<dyn BehaviorNode>) {
        self.children.push(child);
    }
}

impl BehaviorNode for SequenceNode {
    fn tick(&mut self, blackboard: &mut Blackboard, delta_time: f32) -> NodeStatus {
        if self.children.is_empty() {
            return NodeStatus::Success;
        }

        while self.current_child < self.children.len() {
            let status = self.children[self.current_child].tick(blackboard, delta_time);
            
            match status {
                NodeStatus::Success => {
                    self.current_child += 1;
                }
                NodeStatus::Failure => {
                    self.reset();
                    return NodeStatus::Failure;
                }
                NodeStatus::Running => {
                    return NodeStatus::Running;
                }
                NodeStatus::Invalid => {
                    self.reset();
                    return NodeStatus::Invalid;
                }
            }
        }

        self.reset();
        NodeStatus::Success
    }

    fn reset(&mut self) {
        self.current_child = 0;
        for child in &mut self.children {
            child.reset();
        }
    }

    fn get_name(&self) -> &str { &self.name }
    fn get_description(&self) -> &str { &self.description }
    fn is_composite(&self) -> bool { true }
}

#[derive(Debug)]
pub struct SelectorNode {
    name: String,
    description: String,
    children: Vec<Box<dyn BehaviorNode>>,
    current_child: usize,
}

impl SelectorNode {
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            children: Vec::new(),
            current_child: 0,
        }
    }

    pub fn add_child(&mut self, child: Box<dyn BehaviorNode>) {
        self.children.push(child);
    }
}

impl BehaviorNode for SelectorNode {
    fn tick(&mut self, blackboard: &mut Blackboard, delta_time: f32) -> NodeStatus {
        if self.children.is_empty() {
            return NodeStatus::Failure;
        }

        while self.current_child < self.children.len() {
            let status = self.children[self.current_child].tick(blackboard, delta_time);
            
            match status {
                NodeStatus::Success => {
                    self.reset();
                    return NodeStatus::Success;
                }
                NodeStatus::Failure => {
                    self.current_child += 1;
                }
                NodeStatus::Running => {
                    return NodeStatus::Running;
                }
                NodeStatus::Invalid => {
                    self.current_child += 1;
                }
            }
        }

        self.reset();
        NodeStatus::Failure
    }

    fn reset(&mut self) {
        self.current_child = 0;
        for child in &mut self.children {
            child.reset();
        }
    }

    fn get_name(&self) -> &str { &self.name }
    fn get_description(&self) -> &str { &self.description }
    fn is_composite(&self) -> bool { true }
}

#[derive(Debug)]
pub struct ParallelNode {
    name: String,
    description: String,
    children: Vec<Box<dyn BehaviorNode>>,
    success_policy: ParallelPolicy,
    failure_policy: ParallelPolicy,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ParallelPolicy {
    RequireOne,
    RequireAll,
}

impl ParallelNode {
    pub fn new(name: String, description: String, success_policy: ParallelPolicy, failure_policy: ParallelPolicy) -> Self {
        Self {
            name,
            description,
            children: Vec::new(),
            success_policy,
            failure_policy,
        }
    }

    pub fn add_child(&mut self, child: Box<dyn BehaviorNode>) {
        self.children.push(child);
    }
}

impl BehaviorNode for ParallelNode {
    fn tick(&mut self, blackboard: &mut Blackboard, delta_time: f32) -> NodeStatus {
        if self.children.is_empty() {
            return NodeStatus::Success;
        }

        let mut success_count = 0;
        let mut failure_count = 0;
        let mut running_count = 0;

        for child in &mut self.children {
            match child.tick(blackboard, delta_time) {
                NodeStatus::Success => success_count += 1,
                NodeStatus::Failure => failure_count += 1,
                NodeStatus::Running => running_count += 1,
                NodeStatus::Invalid => failure_count += 1,
            }
        }

        // Check success conditions
        let success_threshold = match self.success_policy {
            ParallelPolicy::RequireOne => 1,
            ParallelPolicy::RequireAll => self.children.len(),
        };

        if success_count >= success_threshold {
            return NodeStatus::Success;
        }

        // Check failure conditions
        let failure_threshold = match self.failure_policy {
            ParallelPolicy::RequireOne => 1,
            ParallelPolicy::RequireAll => self.children.len(),
        };

        if failure_count >= failure_threshold {
            return NodeStatus::Failure;
        }

        // Still running if we have running children
        if running_count > 0 {
            NodeStatus::Running
        } else {
            NodeStatus::Failure
        }
    }

    fn reset(&mut self) {
        for child in &mut self.children {
            child.reset();
        }
    }

    fn get_name(&self) -> &str { &self.name }
    fn get_description(&self) -> &str { &self.description }
    fn is_composite(&self) -> bool { true }
}

// Decorator Nodes
#[derive(Debug)]
pub struct InverterNode {
    name: String,
    description: String,
    child: Option<Box<dyn BehaviorNode>>,
}

impl InverterNode {
    pub fn new(name: String, description: String, child: Option<Box<dyn BehaviorNode>>) -> Self {
        Self {
            name,
            description,
            child,
        }
    }
}

impl BehaviorNode for InverterNode {
    fn tick(&mut self, blackboard: &mut Blackboard, delta_time: f32) -> NodeStatus {
        if let Some(ref mut child) = self.child {
            match child.tick(blackboard, delta_time) {
                NodeStatus::Success => NodeStatus::Failure,
                NodeStatus::Failure => NodeStatus::Success,
                NodeStatus::Running => NodeStatus::Running,
                NodeStatus::Invalid => NodeStatus::Invalid,
            }
        } else {
            NodeStatus::Invalid
        }
    }

    fn reset(&mut self) {
        if let Some(ref mut child) = self.child {
            child.reset();
        }
    }

    fn get_name(&self) -> &str { &self.name }
    fn get_description(&self) -> &str { &self.description }
    fn is_decorator(&self) -> bool { true }
}

#[derive(Debug)]
pub struct RepeaterNode {
    name: String,
    description: String,
    child: Option<Box<dyn BehaviorNode>>,
    repeat_count: Option<u32>,
    current_count: u32,
}

impl RepeaterNode {
    pub fn new(name: String, description: String, child: Option<Box<dyn BehaviorNode>>, repeat_count: Option<u32>) -> Self {
        Self {
            name,
            description,
            child,
            repeat_count,
            current_count: 0,
        }
    }
}

impl BehaviorNode for RepeaterNode {
    fn tick(&mut self, blackboard: &mut Blackboard, delta_time: f32) -> NodeStatus {
        if let Some(ref mut child) = self.child {
            if let Some(max_count) = self.repeat_count {
                if self.current_count >= max_count {
                    return NodeStatus::Success;
                }
            }

            match child.tick(blackboard, delta_time) {
                NodeStatus::Success | NodeStatus::Failure => {
                    self.current_count += 1;
                    child.reset();
                    
                    if let Some(max_count) = self.repeat_count {
                        if self.current_count >= max_count {
                            NodeStatus::Success
                        } else {
                            NodeStatus::Running
                        }
                    } else {
                        NodeStatus::Running
                    }
                }
                NodeStatus::Running => NodeStatus::Running,
                NodeStatus::Invalid => NodeStatus::Invalid,
            }
        } else {
            NodeStatus::Invalid
        }
    }

    fn reset(&mut self) {
        self.current_count = 0;
        if let Some(ref mut child) = self.child {
            child.reset();
        }
    }

    fn get_name(&self) -> &str { &self.name }
    fn get_description(&self) -> &str { &self.description }
    fn is_decorator(&self) -> bool { true }
}

#[derive(Debug)]
pub struct RetryNode {
    name: String,
    description: String,
    child: Option<Box<dyn BehaviorNode>>,
    max_attempts: u32,
    current_attempts: u32,
}

impl RetryNode {
    pub fn new(name: String, description: String, child: Option<Box<dyn BehaviorNode>>, max_attempts: u32) -> Self {
        Self {
            name,
            description,
            child,
            max_attempts,
            current_attempts: 0,
        }
    }
}

impl BehaviorNode for RetryNode {
    fn tick(&mut self, blackboard: &mut Blackboard, delta_time: f32) -> NodeStatus {
        if let Some(ref mut child) = self.child {
            match child.tick(blackboard, delta_time) {
                NodeStatus::Success => {
                    self.reset();
                    NodeStatus::Success
                }
                NodeStatus::Failure => {
                    self.current_attempts += 1;
                    if self.current_attempts >= self.max_attempts {
                        self.reset();
                        NodeStatus::Failure
                    } else {
                        child.reset();
                        NodeStatus::Running
                    }
                }
                NodeStatus::Running => NodeStatus::Running,
                NodeStatus::Invalid => NodeStatus::Invalid,
            }
        } else {
            NodeStatus::Invalid
        }
    }

    fn reset(&mut self) {
        self.current_attempts = 0;
        if let Some(ref mut child) = self.child {
            child.reset();
        }
    }

    fn get_name(&self) -> &str { &self.name }
    fn get_description(&self) -> &str { &self.description }
    fn is_decorator(&self) -> bool { true }
}

// Leaf Nodes (Actions and Conditions)
pub struct ConditionNode {
    name: String,
    description: String,
    condition_fn: Box<dyn Fn(&Blackboard) -> bool + Send + Sync>,
}

impl std::fmt::Debug for ConditionNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConditionNode")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("condition_fn", &"<function>")
            .finish()
    }
}

impl ConditionNode {
    pub fn new<F>(name: String, description: String, condition_fn: F) -> Self 
    where
        F: Fn(&Blackboard) -> bool + Send + Sync + 'static,
    {
        Self {
            name,
            description,
            condition_fn: Box::new(condition_fn),
        }
    }
}

impl BehaviorNode for ConditionNode {
    fn tick(&mut self, blackboard: &mut Blackboard, _delta_time: f32) -> NodeStatus {
        if (self.condition_fn)(blackboard) {
            NodeStatus::Success
        } else {
            NodeStatus::Failure
        }
    }

    fn reset(&mut self) {}

    fn get_name(&self) -> &str { &self.name }
    fn get_description(&self) -> &str { &self.description }
}

pub struct ActionNode {
    name: String,
    description: String,
    action_fn: Box<dyn FnMut(&mut Blackboard, f32) -> NodeStatus + Send + Sync>,
}

impl std::fmt::Debug for ActionNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActionNode")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("action_fn", &"<function>")
            .finish()
    }
}

impl ActionNode {
    pub fn new<F>(name: String, description: String, action_fn: F) -> Self 
    where
        F: FnMut(&mut Blackboard, f32) -> NodeStatus + Send + Sync + 'static,
    {
        Self {
            name,
            description,
            action_fn: Box::new(action_fn),
        }
    }
}

impl BehaviorNode for ActionNode {
    fn tick(&mut self, blackboard: &mut Blackboard, delta_time: f32) -> NodeStatus {
        (self.action_fn)(blackboard, delta_time)
    }

    fn reset(&mut self) {}

    fn get_name(&self) -> &str { &self.name }
    fn get_description(&self) -> &str { &self.description }
}

// Wait Action - useful for timing-based behaviors
#[derive(Debug)]
pub struct WaitNode {
    name: String,
    description: String,
    wait_time: f32,
    elapsed_time: f32,
}

impl WaitNode {
    pub fn new(name: String, description: String, wait_time: f32) -> Self {
        Self {
            name,
            description,
            wait_time,
            elapsed_time: 0.0,
        }
    }
}

impl BehaviorNode for WaitNode {
    fn tick(&mut self, _blackboard: &mut Blackboard, delta_time: f32) -> NodeStatus {
        self.elapsed_time += delta_time;
        if self.elapsed_time >= self.wait_time {
            NodeStatus::Success
        } else {
            NodeStatus::Running
        }
    }

    fn reset(&mut self) {
        self.elapsed_time = 0.0;
    }

    fn get_name(&self) -> &str { &self.name }
    fn get_description(&self) -> &str { &self.description }
}

// Behavior Tree Structure
#[derive(Debug)]
pub struct BehaviorTree {
    id: String,
    name: String,
    root: Option<Box<dyn BehaviorNode>>,
    blackboard: Blackboard,
    is_active: bool,
    last_tick_time: Instant,
    tick_interval: Duration,
}

impl Default for BehaviorTree {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            root: None,
            blackboard: Blackboard::default(),
            is_active: false,
            last_tick_time: Instant::now(),
            tick_interval: Duration::from_millis(100),
        }
    }
}

impl BehaviorTree {
    pub fn new(name: String, entity_id: String, tick_rate_hz: f32) -> Self {
        let tick_interval = Duration::from_secs_f32(1.0 / tick_rate_hz);
        
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            root: None,
            blackboard: Blackboard::new(entity_id),
            is_active: false,
            last_tick_time: Instant::now(),
            tick_interval,
        }
    }

    pub fn set_root(&mut self, root: Box<dyn BehaviorNode>) {
        self.root = Some(root);
    }

    pub fn tick(&mut self, delta_time: f32) -> RobinResult<NodeStatus> {
        if !self.is_active {
            return Ok(NodeStatus::Invalid);
        }

        if self.last_tick_time.elapsed() < self.tick_interval {
            return Ok(NodeStatus::Running);
        }

        self.last_tick_time = Instant::now();

        if let Some(ref mut root) = self.root {
            Ok(root.tick(&mut self.blackboard, delta_time))
        } else {
            Ok(NodeStatus::Invalid)
        }
    }

    pub fn reset(&mut self) {
        if let Some(ref mut root) = self.root {
            root.reset();
        }
    }

    pub fn start(&mut self) {
        self.is_active = true;
        self.reset();
    }

    pub fn stop(&mut self) {
        self.is_active = false;
        self.reset();
    }

    pub fn pause(&mut self) {
        self.is_active = false;
    }

    pub fn resume(&mut self) {
        self.is_active = true;
    }

    pub fn get_id(&self) -> &str { &self.id }
    pub fn get_name(&self) -> &str { &self.name }
    pub fn is_active(&self) -> bool { self.is_active }
    pub fn get_blackboard(&self) -> &Blackboard { &self.blackboard }
    pub fn get_blackboard_mut(&mut self) -> &mut Blackboard { &mut self.blackboard }
}

// Behavior Tree System
#[derive(Debug)]
pub struct BehaviorTreeSystem {
    config: TreeConfig,
    trees: HashMap<String, BehaviorTree>,
    entity_trees: HashMap<String, String>,
    shared_blackboard: HashMap<String, RuntimeValue>,
    active_tree_count: u32,
}

impl BehaviorTreeSystem {
    pub fn new(config: TreeConfig) -> RobinResult<Self> {
        Ok(Self {
            config,
            trees: HashMap::new(),
            entity_trees: HashMap::new(),
            shared_blackboard: HashMap::new(),
            active_tree_count: 0,
        })
    }

    pub fn initialize(&mut self) -> RobinResult<()> {
        println!("Behavior Tree System initialized:");
        println!("  Max execution time: {}ms", self.config.max_execution_time_ms);
        println!("  Max tree depth: {}", self.config.max_depth);
        println!("  Tick rate: {}Hz", self.config.tick_rate_hz);
        println!("  Blackboard sharing: {}", self.config.enable_blackboard_sharing);
        Ok(())
    }

    pub fn create_tree(&mut self, name: String) -> RobinResult<String> {
        let entity_id = format!("entity_{}", Uuid::new_v4());
        let tree = BehaviorTree::new(name.clone(), entity_id, self.config.tick_rate_hz);
        let tree_id = tree.get_id().to_string();
        
        self.trees.insert(tree_id.clone(), tree);
        
        println!("Created behavior tree: {} (ID: {})", name, tree_id);
        Ok(tree_id)
    }

    pub fn assign_tree_to_entity(&mut self, entity_id: &str, tree_id: &str) -> RobinResult<()> {
        if !self.trees.contains_key(tree_id) {
            return Err(crate::engine::error::RobinError::InvalidInput(
                format!("Tree with ID {} not found", tree_id)
            ));
        }

        self.entity_trees.insert(entity_id.to_string(), tree_id.to_string());
        
        if let Some(tree) = self.trees.get_mut(tree_id) {
            tree.start();
            self.active_tree_count += 1;
        }

        Ok(())
    }

    pub fn update(&mut self, delta_time: f32) -> RobinResult<()> {
        let execution_start = Instant::now();

        for tree in self.trees.values_mut() {
            if tree.is_active() {
                tree.tick(delta_time)?;
                
                // Check execution time limit
                if execution_start.elapsed().as_millis() > self.config.max_execution_time_ms as u128 {
                    println!("Warning: Behavior tree execution time exceeded limit");
                    break;
                }
            }
        }

        Ok(())
    }

    pub fn update_blackboard(&mut self, entity_id: &str, key: String, value: RuntimeValue) -> RobinResult<()> {
        if let Some(tree_id) = self.entity_trees.get(entity_id) {
            if let Some(tree) = self.trees.get_mut(tree_id) {
                tree.get_blackboard_mut().set(key, value);
                return Ok(());
            }
        }
        
        Err(crate::engine::error::RobinError::InvalidInput(
            format!("No behavior tree assigned to entity: {}", entity_id)
        ))
    }

    pub fn get_blackboard_value(&self, entity_id: &str, key: &str) -> Option<&RuntimeValue> {
        if let Some(tree_id) = self.entity_trees.get(entity_id) {
            if let Some(tree) = self.trees.get(tree_id) {
                return tree.get_blackboard().get(key);
            }
        }
        None
    }

    pub fn pause_tree(&mut self, tree_id: &str) -> RobinResult<()> {
        if let Some(tree) = self.trees.get_mut(tree_id) {
            if tree.is_active() {
                tree.pause();
                self.active_tree_count = self.active_tree_count.saturating_sub(1);
            }
        }
        Ok(())
    }

    pub fn resume_tree(&mut self, tree_id: &str) -> RobinResult<()> {
        if let Some(tree) = self.trees.get_mut(tree_id) {
            if !tree.is_active() {
                tree.resume();
                self.active_tree_count += 1;
            }
        }
        Ok(())
    }

    pub fn remove_tree(&mut self, tree_id: &str) -> RobinResult<()> {
        if let Some(tree) = self.trees.remove(tree_id) {
            if tree.is_active() {
                self.active_tree_count = self.active_tree_count.saturating_sub(1);
            }

            // Remove entity assignments
            self.entity_trees.retain(|_, v| v != tree_id);
        }
        Ok(())
    }

    pub fn get_tree_status(&self, tree_id: &str) -> Option<bool> {
        self.trees.get(tree_id).map(|tree| tree.is_active())
    }

    pub fn get_active_tree_count(&self) -> u32 {
        self.active_tree_count
    }

    pub fn get_tree_names(&self) -> Vec<String> {
        self.trees.values().map(|tree| tree.get_name().to_string()).collect()
    }

    pub fn auto_save_modified_trees(&mut self) -> RobinResult<()> {
        // In a full implementation, this would serialize and save modified trees
        println!("Auto-saving {} behavior trees", self.trees.len());
        Ok(())
    }

    pub fn shutdown(&mut self) -> RobinResult<()> {
        println!("Behavior Tree System shutdown:");
        println!("  Total trees created: {}", self.trees.len());
        println!("  Active trees at shutdown: {}", self.active_tree_count);
        
        self.trees.clear();
        self.entity_trees.clear();
        self.shared_blackboard.clear();
        self.active_tree_count = 0;
        
        Ok(())
    }
}

// Builder pattern for creating complex behavior trees
pub struct BehaviorTreeBuilder {
    tree: BehaviorTree,
}

impl BehaviorTreeBuilder {
    pub fn new(name: String, entity_id: String, tick_rate_hz: f32) -> Self {
        Self {
            tree: BehaviorTree::new(name, entity_id, tick_rate_hz),
        }
    }

    pub fn with_sequence(mut self, name: String) -> SequenceBuilder {
        SequenceBuilder::new(self, name)
    }

    pub fn with_selector(mut self, name: String) -> SelectorBuilder {
        SelectorBuilder::new(self, name)
    }

    pub fn with_parallel(mut self, name: String, success_policy: ParallelPolicy, failure_policy: ParallelPolicy) -> ParallelBuilder {
        ParallelBuilder::new(self, name, success_policy, failure_policy)
    }

    pub fn build(self) -> BehaviorTree {
        self.tree
    }

    fn set_root(&mut self, root: Box<dyn BehaviorNode>) {
        self.tree.set_root(root);
    }
}

pub struct SequenceBuilder {
    tree_builder: BehaviorTreeBuilder,
    node: SequenceNode,
}

impl SequenceBuilder {
    fn new(tree_builder: BehaviorTreeBuilder, name: String) -> Self {
        Self {
            tree_builder,
            node: SequenceNode::new(name, "Sequence node".to_string()),
        }
    }

    pub fn add_condition<F>(mut self, name: String, condition: F) -> Self 
    where
        F: Fn(&Blackboard) -> bool + Send + Sync + 'static,
    {
        let condition_node = ConditionNode::new(name, "Condition".to_string(), condition);
        self.node.add_child(Box::new(condition_node));
        self
    }

    pub fn add_action<F>(mut self, name: String, action: F) -> Self 
    where
        F: FnMut(&mut Blackboard, f32) -> NodeStatus + Send + Sync + 'static,
    {
        let action_node = ActionNode::new(name, "Action".to_string(), action);
        self.node.add_child(Box::new(action_node));
        self
    }

    pub fn add_wait(mut self, name: String, wait_time: f32) -> Self {
        let wait_node = WaitNode::new(name, format!("Wait {} seconds", wait_time), wait_time);
        self.node.add_child(Box::new(wait_node));
        self
    }

    pub fn build(mut self) -> BehaviorTreeBuilder {
        self.tree_builder.set_root(Box::new(self.node));
        self.tree_builder
    }
}

pub struct SelectorBuilder {
    tree_builder: BehaviorTreeBuilder,
    node: SelectorNode,
}

impl SelectorBuilder {
    fn new(tree_builder: BehaviorTreeBuilder, name: String) -> Self {
        Self {
            tree_builder,
            node: SelectorNode::new(name, "Selector node".to_string()),
        }
    }

    pub fn add_condition<F>(mut self, name: String, condition: F) -> Self 
    where
        F: Fn(&Blackboard) -> bool + Send + Sync + 'static,
    {
        let condition_node = ConditionNode::new(name, "Condition".to_string(), condition);
        self.node.add_child(Box::new(condition_node));
        self
    }

    pub fn add_action<F>(mut self, name: String, action: F) -> Self 
    where
        F: FnMut(&mut Blackboard, f32) -> NodeStatus + Send + Sync + 'static,
    {
        let action_node = ActionNode::new(name, "Action".to_string(), action);
        self.node.add_child(Box::new(action_node));
        self
    }

    pub fn build(mut self) -> BehaviorTreeBuilder {
        self.tree_builder.set_root(Box::new(self.node));
        self.tree_builder
    }
}

pub struct ParallelBuilder {
    tree_builder: BehaviorTreeBuilder,
    node: ParallelNode,
}

impl ParallelBuilder {
    fn new(tree_builder: BehaviorTreeBuilder, name: String, success_policy: ParallelPolicy, failure_policy: ParallelPolicy) -> Self {
        Self {
            tree_builder,
            node: ParallelNode::new(name, "Parallel node".to_string(), success_policy, failure_policy),
        }
    }

    pub fn add_condition<F>(mut self, name: String, condition: F) -> Self 
    where
        F: Fn(&Blackboard) -> bool + Send + Sync + 'static,
    {
        let condition_node = ConditionNode::new(name, "Condition".to_string(), condition);
        self.node.add_child(Box::new(condition_node));
        self
    }

    pub fn add_action<F>(mut self, name: String, action: F) -> Self 
    where
        F: FnMut(&mut Blackboard, f32) -> NodeStatus + Send + Sync + 'static,
    {
        let action_node = ActionNode::new(name, "Action".to_string(), action);
        self.node.add_child(Box::new(action_node));
        self
    }

    pub fn build(mut self) -> BehaviorTreeBuilder {
        self.tree_builder.set_root(Box::new(self.node));
        self.tree_builder
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequence_node() {
        let mut sequence = SequenceNode::new("TestSequence".to_string(), "Test sequence".to_string());
        
        // Add simple condition that always succeeds
        let condition = ConditionNode::new(
            "AlwaysTrue".to_string(),
            "Always returns true".to_string(),
            |_| true
        );
        sequence.add_child(Box::new(condition));

        let mut blackboard = Blackboard::new("test_entity".to_string());
        let status = sequence.tick(&mut blackboard, 0.016);
        assert_eq!(status, NodeStatus::Success);
    }

    #[test]
    fn test_selector_node() {
        let mut selector = SelectorNode::new("TestSelector".to_string(), "Test selector".to_string());
        
        // Add condition that always fails
        let condition1 = ConditionNode::new(
            "AlwaysFalse".to_string(),
            "Always returns false".to_string(),
            |_| false
        );
        selector.add_child(Box::new(condition1));

        // Add condition that always succeeds
        let condition2 = ConditionNode::new(
            "AlwaysTrue".to_string(),
            "Always returns true".to_string(),
            |_| true
        );
        selector.add_child(Box::new(condition2));

        let mut blackboard = Blackboard::new("test_entity".to_string());
        let status = selector.tick(&mut blackboard, 0.016);
        assert_eq!(status, NodeStatus::Success);
    }

    #[test]
    fn test_blackboard_operations() {
        let mut blackboard = Blackboard::new("test_entity".to_string());
        
        blackboard.set("health".to_string(), RuntimeValue::Int(100));
        blackboard.set("position".to_string(), RuntimeValue::Vector3([1.0, 2.0, 3.0]));
        
        assert_eq!(blackboard.get("health").unwrap().as_float(), 100.0);
        assert!(blackboard.has_key("position"));
        assert!(!blackboard.has_key("nonexistent"));
        
        let keys = blackboard.keys();
        assert!(keys.contains(&"health".to_string()));
        assert!(keys.contains(&"position".to_string()));
    }

    #[test]
    fn test_behavior_tree_builder() {
        let tree = BehaviorTreeBuilder::new("TestTree".to_string(), "test_entity".to_string(), 30.0)
            .with_sequence("MainSequence".to_string())
                .add_condition("CheckHealth".to_string(), |blackboard| {
                    blackboard.get("health").map(|v| v.as_float() > 0.0).unwrap_or(false)
                })
                .add_wait("Wait1Second".to_string(), 1.0)
                .add_action("Heal".to_string(), |blackboard, _delta| {
                    blackboard.set("health".to_string(), RuntimeValue::Int(100));
                    NodeStatus::Success
                })
                .build()
            .build();

        assert_eq!(tree.get_name(), "TestTree");
    }
}