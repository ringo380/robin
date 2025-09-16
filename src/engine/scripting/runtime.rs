use crate::engine::error::RobinResult;
use super::{ScriptNode, NodeConnection, NodeType};
use crate::engine::scripting::ScriptingSystemConfig;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, VecDeque, HashSet};
use std::time::{Instant, Duration};

// Core scripting value types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScriptValue {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Array(Vec<ScriptValue>),
    Object(HashMap<String, ScriptValue>),
}

impl ScriptValue {
    /// Get the type name of this script value
    pub fn type_name(&self) -> &'static str {
        match self {
            ScriptValue::Null => "null",
            ScriptValue::Boolean(_) => "boolean",
            ScriptValue::Integer(_) => "integer",
            ScriptValue::Float(_) => "float",
            ScriptValue::String(_) => "string",
            ScriptValue::Array(_) => "array",
            ScriptValue::Object(_) => "object",
        }
    }

    /// Convert this value to a number if possible
    pub fn as_number(&self) -> Option<f64> {
        match self {
            ScriptValue::Integer(i) => Some(*i as f64),
            ScriptValue::Float(f) => Some(*f),
            ScriptValue::Boolean(b) => Some(if *b { 1.0 } else { 0.0 }),
            ScriptValue::String(s) => s.parse().ok(),
            _ => None,
        }
    }

    /// Convert this value to a string
    pub fn as_string(&self) -> String {
        match self {
            ScriptValue::Null => "null".to_string(),
            ScriptValue::Boolean(b) => b.to_string(),
            ScriptValue::Integer(i) => i.to_string(),
            ScriptValue::Float(f) => f.to_string(),
            ScriptValue::String(s) => s.clone(),
            ScriptValue::Array(_) => "[array]".to_string(),
            ScriptValue::Object(_) => "[object]".to_string(),
        }
    }

    /// None variant equivalent (maps to Null for compatibility)
    pub const None: ScriptValue = ScriptValue::Null;
}

// Script execution result
pub type ScriptResult<T> = Result<T, ScriptError>;

// Script error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScriptError {
    RuntimeError(String),
    TypeError(String),
    IndexOutOfBounds(usize),
    VariableNotFound(String),
    FunctionNotFound(String),
    ExecutionTimeout,
    MemoryLimitExceeded,
    StackOverflow,
    MissingConnection(String),
}

impl std::fmt::Display for ScriptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScriptError::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
            ScriptError::TypeError(msg) => write!(f, "Type error: {}", msg),
            ScriptError::IndexOutOfBounds(index) => write!(f, "Index out of bounds: {}", index),
            ScriptError::VariableNotFound(name) => write!(f, "Variable not found: {}", name),
            ScriptError::FunctionNotFound(name) => write!(f, "Function not found: {}", name),
            ScriptError::ExecutionTimeout => write!(f, "Execution timeout"),
            ScriptError::MemoryLimitExceeded => write!(f, "Memory limit exceeded"),
            ScriptError::StackOverflow => write!(f, "Stack overflow"),
            ScriptError::MissingConnection(details) => write!(f, "Missing connection: {}", details),
        }
    }
}

impl std::error::Error for ScriptError {}

// Script definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    pub id: String,
    pub name: String,
    pub nodes: Vec<ScriptNode>,
    pub connections: Vec<NodeConnection>,
    pub entry_point: Option<String>,
    pub variables: HashMap<String, ScriptValue>,
}

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub variables: HashMap<String, ScriptValue>,
    pub entity_states: HashMap<u64, HashMap<String, ScriptValue>>,
    pub world_state: HashMap<String, ScriptValue>,
    pub call_stack: Vec<CallFrame>,
    pub execution_time: Duration,
    pub step_count: u32,
    pub max_steps: u32,
    pub random_seed: u64,
}

#[derive(Debug, Clone)]
pub struct CallFrame {
    pub script_id: String,
    pub node_id: String,
    pub local_variables: HashMap<String, ScriptValue>,
    pub entry_time: Instant,
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self {
            variables: HashMap::new(),
            entity_states: HashMap::new(),
            world_state: HashMap::new(),
            call_stack: Vec::new(),
            execution_time: Duration::new(0, 0),
            step_count: 0,
            max_steps: 10000,
            random_seed: 42,
        }
    }
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_with_seed(seed: u64) -> Self {
        Self {
            random_seed: seed,
            ..Default::default()
        }
    }

    pub fn get_variable(&self, name: &str) -> Option<&ScriptValue> {
        if let Some(frame) = self.call_stack.last() {
            if let Some(value) = frame.local_variables.get(name) {
                return Some(value);
            }
        }
        self.variables.get(name)
    }

    pub fn set_variable(&mut self, name: String, value: ScriptValue) {
        if let Some(frame) = self.call_stack.last_mut() {
            frame.local_variables.insert(name, value);
        } else {
            self.variables.insert(name, value);
        }
    }

    pub fn set_global_variable(&mut self, name: String, value: ScriptValue) {
        self.variables.insert(name, value);
    }

    pub fn push_call_frame(&mut self, script_id: String, node_id: String) -> ScriptResult<()> {
        if self.call_stack.len() >= 50 {
            return Err(ScriptError::StackOverflow);
        }

        self.call_stack.push(CallFrame {
            script_id,
            node_id,
            local_variables: HashMap::new(),
            entry_time: Instant::now(),
        });
        
        Ok(())
    }

    pub fn pop_call_frame(&mut self) -> Option<CallFrame> {
        self.call_stack.pop()
    }

    pub fn increment_step(&mut self) -> ScriptResult<()> {
        self.step_count += 1;
        if self.step_count > self.max_steps {
            return Err(ScriptError::ExecutionTimeout);
        }
        Ok(())
    }

    pub fn reset_execution(&mut self) {
        self.call_stack.clear();
        self.execution_time = Duration::new(0, 0);
        self.step_count = 0;
    }
}

#[derive(Debug)]
pub struct ScriptRuntime {
    config: ScriptingSystemConfig,
    execution_cache: HashMap<String, CachedExecution>,
    instruction_set: InstructionSet,
}

#[derive(Debug, Clone)]
struct CachedExecution {
    instructions: Vec<Instruction>,
    execution_order: Vec<String>,
    last_compiled: Instant,
    compile_time_ms: u64,
}

#[derive(Debug, Clone)]
pub enum Instruction {
    LoadConstant { target: String, value: ScriptValue },
    LoadVariable { target: String, variable: String },
    StoreVariable { variable: String, source: String },
    Add { target: String, left: String, right: String },
    Subtract { target: String, left: String, right: String },
    Multiply { target: String, left: String, right: String },
    Divide { target: String, left: String, right: String },
    Compare { target: String, left: String, right: String, operation: CompareOp },
    Jump { target_node: String },
    ConditionalJump { condition: String, target_node: String, else_node: Option<String> },
    CallFunction { target: Option<String>, function: String, args: Vec<String> },
    Return { value: Option<String> },
    Log { message: String },
    SetEntityState { entity: String, property: String, value: String },
    GetEntityState { target: String, entity: String, property: String },
    EmitEvent { event_type: String, data: Vec<(String, String)> },
    WaitSeconds { duration: f32 },
    Nop,
}

#[derive(Debug, Clone)]
pub enum CompareOp {
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

#[derive(Debug)]
struct InstructionSet {
    builtin_functions: HashMap<String, fn(&[ScriptValue]) -> ScriptResult<ScriptValue>>,
}

impl InstructionSet {
    fn new() -> Self {
        let mut functions: HashMap<String, fn(&[ScriptValue]) -> ScriptResult<ScriptValue>> = HashMap::new();
        
        functions.insert("abs".to_string(), |args| {
            if args.len() != 1 {
                return Err(ScriptError::RuntimeError("abs() takes exactly 1 argument".to_string()));
            }
            match &args[0] {
                ScriptValue::Float(f) => Ok(ScriptValue::Float(f.abs())),
                ScriptValue::Integer(i) => Ok(ScriptValue::Integer(i.abs())),
                _ => Err(ScriptError::TypeError(
                    format!("Expected number, found {}", args[0].type_name())
                ))
            }
        });

        functions.insert("sqrt".to_string(), |args| {
            if args.len() != 1 {
                return Err(ScriptError::RuntimeError("sqrt() takes exactly 1 argument".to_string()));
            }
            let num = args[0].as_number().unwrap_or(0.0);
            if num < 0.0 {
                return Err(ScriptError::RuntimeError("sqrt() requires non-negative number".to_string()));
            }
            Ok(ScriptValue::Float(num.sqrt()))
        });

        functions.insert("sin".to_string(), |args| {
            if args.len() != 1 {
                return Err(ScriptError::RuntimeError("sin() takes exactly 1 argument".to_string()));
            }
            Ok(ScriptValue::Float(args[0].as_number().unwrap_or(0.0).sin()))
        });

        functions.insert("cos".to_string(), |args| {
            if args.len() != 1 {
                return Err(ScriptError::RuntimeError("cos() takes exactly 1 argument".to_string()));
            }
            Ok(ScriptValue::Float(args[0].as_number().unwrap_or(0.0).cos()))
        });

        functions.insert("random".to_string(), |_args| {
            Ok(ScriptValue::Float(fastrand::f64()))
        });

        functions.insert("string_length".to_string(), |args| {
            if args.len() != 1 {
                return Err(ScriptError::RuntimeError("string_length() takes exactly 1 argument".to_string()));
            }
            Ok(ScriptValue::Integer(args[0].as_string().len() as i64))
        });

        Self { builtin_functions: functions }
    }

    fn call_function(&self, name: &str, args: &[ScriptValue]) -> ScriptResult<ScriptValue> {
        if let Some(func) = self.builtin_functions.get(name) {
            func(args)
        } else {
            Err(ScriptError::RuntimeError(format!("Unknown function: {}", name)))
        }
    }
}

impl ScriptRuntime {
    pub fn new(config: ScriptingSystemConfig) -> RobinResult<Self> {
        Ok(Self {
            config,
            execution_cache: HashMap::new(),
            instruction_set: InstructionSet::new(),
        })
    }

    pub fn execute(&mut self, script: &Script, context: &mut ExecutionContext) -> ScriptResult<ScriptValue> {
        let start_time = Instant::now();
        context.reset_execution();

        let instructions = {
            let cached_execution = self.get_or_compile_script(script)?;
            cached_execution.instructions.clone()
        };

        let mut execution_state = ExecutionState {
            current_node: 0,
            registers: HashMap::new(),
            suspended_until: None,
        };

        let result = self.execute_instructions(&instructions, context, &mut execution_state)?;
        
        context.execution_time = start_time.elapsed();
        
        if context.execution_time.as_millis() as u64 > self.config.execution_time_limit_ms as u64 {
            return Err(ScriptError::ExecutionTimeout);
        }

        Ok(result)
    }

    pub fn validate_script(&self, script: &Script) -> Vec<ScriptError> {
        let mut errors = Vec::new();

        if script.nodes.is_empty() {
            errors.push(ScriptError::RuntimeError("Script contains no nodes".to_string()));
            return errors;
        }

        let node_ids: HashSet<_> = script.nodes.iter().map(|n| &n.id).collect();
        
        for connection in &script.connections {
            if !node_ids.contains(&connection.from_node_id) {
                errors.push(ScriptError::MissingConnection(format!(
                    "Connection references non-existent from_node: {}", connection.from_node_id
                )));
            }
            if !node_ids.contains(&connection.to_node_id) {
                errors.push(ScriptError::MissingConnection(format!(
                    "Connection references non-existent to_node: {}", connection.to_node_id
                )));
            }
        }

        if let Err(compile_error) = self.compile_to_instructions(script) {
            errors.push(compile_error);
        }

        errors
    }

    fn get_or_compile_script(&mut self, script: &Script) -> ScriptResult<&CachedExecution> {
        let cache_key = script.id.clone();

        // Check if we have a valid cached version
        let needs_recompilation = match self.execution_cache.get(&cache_key) {
            Some(cached) => cached.last_compiled.elapsed().as_secs() >= 60,
            None => true,
        };

        if needs_recompilation {
            let compile_start = Instant::now();
            let instructions = self.compile_to_instructions(script)?;
            let execution_order = self.determine_execution_order(script)?;
            let compile_time = compile_start.elapsed().as_millis() as u64;

            let cached_execution = CachedExecution {
                instructions,
                execution_order,
                last_compiled: Instant::now(),
                compile_time_ms: compile_time,
            };

            self.execution_cache.insert(cache_key.clone(), cached_execution);
        }

        Ok(self.execution_cache.get(&cache_key).unwrap())
    }

    fn compile_to_instructions(&self, script: &Script) -> ScriptResult<Vec<Instruction>> {
        let mut instructions = Vec::new();
        let execution_order = self.determine_execution_order(script)?;

        for node_id in execution_order {
            if let Some(node) = script.nodes.iter().find(|n| n.id == node_id) {
                instructions.extend(self.compile_node_to_instructions(node)?);
            }
        }

        Ok(instructions)
    }

    fn compile_node_to_instructions(&self, node: &ScriptNode) -> ScriptResult<Vec<Instruction>> {
        let mut instructions = Vec::new();

        match &node.node_type {
            NodeType::Start => {
                // Start nodes don't generate instructions directly
            }
            NodeType::End => {
                if let Some(return_value) = node.properties.get("return_value") {
                    let temp_reg = format!("temp_return_{}", node.id);
                    instructions.push(Instruction::LoadConstant { 
                        target: temp_reg.clone(), 
                        value: ScriptValue::String(return_value.clone()) 
                    });
                    instructions.push(Instruction::Return { value: Some(temp_reg) });
                } else {
                    instructions.push(Instruction::Return { value: None });
                }
            }
            NodeType::Constant => {
                if let Some(value) = node.properties.get("value") {
                    let output_reg = format!("const_out_{}", node.id);
                    instructions.push(Instruction::LoadConstant { 
                        target: output_reg, 
                        value: ScriptValue::String(value.clone()) 
                    });
                }
            }
            NodeType::Math => {
                if let Some(op) = node.properties.get("operation") {
                    let operation_str = op.as_str();
                    let left_reg = format!("left_{}", node.id);
                    let right_reg = format!("right_{}", node.id);
                    let output_reg = format!("math_out_{}", node.id);

                    match operation_str {
                        "add" => instructions.push(Instruction::Add { 
                            target: output_reg, left: left_reg, right: right_reg 
                        }),
                        "subtract" => instructions.push(Instruction::Subtract { 
                            target: output_reg, left: left_reg, right: right_reg 
                        }),
                        "multiply" => instructions.push(Instruction::Multiply { 
                            target: output_reg, left: left_reg, right: right_reg 
                        }),
                        "divide" => instructions.push(Instruction::Divide { 
                            target: output_reg, left: left_reg, right: right_reg 
                        }),
                        _ => return Err(ScriptError::RuntimeError(format!("Unknown math operation: {}", operation_str)))
                    }
                }
            }
            NodeType::Logic => {
                if let Some(op) = node.properties.get("operation") {
                    let operation_str = op.as_str();
                    let left_reg = format!("left_{}", node.id);
                    let right_reg = format!("right_{}", node.id);
                    let output_reg = format!("logic_out_{}", node.id);

                    let compare_op = match operation_str {
                        "equals" => CompareOp::Equal,
                        "not_equals" => CompareOp::NotEqual,
                        "less_than" => CompareOp::Less,
                        "less_equal" => CompareOp::LessEqual,
                        "greater_than" => CompareOp::Greater,
                        "greater_equal" => CompareOp::GreaterEqual,
                        _ => return Err(ScriptError::RuntimeError(format!("Unknown logic operation: {}", operation_str)))
                    };

                    instructions.push(Instruction::Compare { 
                        target: output_reg, 
                        left: left_reg, 
                        right: right_reg, 
                        operation: compare_op 
                    });
                }
            }
            NodeType::Condition => {
                let condition_reg = format!("condition_{}", node.id);
                let true_target = node.properties.get("true_node")
                    .map(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                let false_target = node.properties.get("false_node")
                    .map(|v| v.to_string());

                instructions.push(Instruction::ConditionalJump {
                    condition: condition_reg,
                    target_node: true_target,
                    else_node: false_target
                });
            }
            NodeType::Control => {
                if let Some(control_type) = node.properties.get("type") {
                    match control_type.as_str() {
                        "wait" => {
                            if let Some(duration) = node.properties.get("duration") {
                                let duration_value = duration.parse::<f32>().unwrap_or(1.0);
                                instructions.push(Instruction::WaitSeconds { duration: duration_value });
                            }
                        }
                        "log" => {
                            if let Some(message) = node.properties.get("message") {
                                instructions.push(Instruction::Log { message: message.to_string() });
                            }
                        }
                        _ => {
                            instructions.push(Instruction::Nop);
                        }
                    }
                }
            }
            // Handle all other NodeType variants with default behavior
            _ => {
                // Default: Generate a no-op instruction for unsupported node types
                instructions.push(Instruction::Nop);
            }
        }

        Ok(instructions)
    }

    fn determine_execution_order(&self, script: &Script) -> ScriptResult<Vec<String>> {
        let mut visited = HashSet::new();
        let mut execution_order = Vec::new();
        let mut connection_map: HashMap<String, Vec<String>> = HashMap::new();

        for connection in &script.connections {
            connection_map.entry(connection.from_node_id.clone())
                .or_insert_with(Vec::new)
                .push(connection.to_node_id.clone());
        }

        let start_nodes: Vec<_> = script.nodes.iter()
            .filter(|node| matches!(node.node_type, NodeType::Start))
            .map(|node| node.id.clone())
            .collect();

        if start_nodes.is_empty() {
            return Err(ScriptError::RuntimeError("No start node found".to_string()));
        }

        for start_node in start_nodes {
            self.depth_first_traverse(&start_node, &connection_map, &mut visited, &mut execution_order)?;
        }

        Ok(execution_order)
    }

    fn depth_first_traverse(
        &self,
        node_id: &str,
        connection_map: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        execution_order: &mut Vec<String>,
    ) -> ScriptResult<()> {
        if visited.contains(node_id) {
            return Ok(());
        }

        visited.insert(node_id.to_string());
        execution_order.push(node_id.to_string());

        if let Some(connected_nodes) = connection_map.get(node_id) {
            for connected_node in connected_nodes {
                self.depth_first_traverse(connected_node, connection_map, visited, execution_order)?;
            }
        }

        Ok(())
    }

    fn execute_instructions(
        &mut self,
        instructions: &[Instruction],
        context: &mut ExecutionContext,
        state: &mut ExecutionState,
    ) -> ScriptResult<ScriptValue> {
        while state.current_node < instructions.len() {
            context.increment_step()?;

            if let Some(suspend_time) = state.suspended_until {
                if Instant::now() < suspend_time {
                    continue;
                }
                state.suspended_until = None;
            }

            let instruction = &instructions[state.current_node];
            
            match self.execute_instruction(instruction, context, state)? {
                InstructionResult::Continue => {
                    state.current_node += 1;
                }
                InstructionResult::Jump(target) => {
                    state.current_node = target;
                }
                InstructionResult::Return(value) => {
                    return Ok(value);
                }
                InstructionResult::Suspend(duration) => {
                    state.suspended_until = Some(Instant::now() + Duration::from_secs_f32(duration));
                    state.current_node += 1;
                }
            }
        }

        Ok(ScriptValue::None)
    }

    fn execute_instruction(
        &mut self,
        instruction: &Instruction,
        context: &mut ExecutionContext,
        state: &mut ExecutionState,
    ) -> ScriptResult<InstructionResult> {
        match instruction {
            Instruction::LoadConstant { target, value } => {
                state.registers.insert(target.clone(), value.clone());
                Ok(InstructionResult::Continue)
            }
            Instruction::LoadVariable { target, variable } => {
                let value = context.get_variable(variable).unwrap_or(&ScriptValue::None).clone();
                state.registers.insert(target.clone(), value);
                Ok(InstructionResult::Continue)
            }
            Instruction::StoreVariable { variable, source } => {
                if let Some(value) = state.registers.get(source) {
                    context.set_variable(variable.clone(), value.clone());
                }
                Ok(InstructionResult::Continue)
            }
            Instruction::Add { target, left, right } => {
                let left_val = state.registers.get(left).unwrap_or(&ScriptValue::Integer(0));
                let right_val = state.registers.get(right).unwrap_or(&ScriptValue::Integer(0));
                
                let result = match (left_val, right_val) {
                    (ScriptValue::Integer(a), ScriptValue::Integer(b)) => ScriptValue::Integer(a + b),
                    (ScriptValue::Float(a), ScriptValue::Float(b)) => ScriptValue::Float(a + b),
                    (ScriptValue::Integer(a), ScriptValue::Float(b)) => ScriptValue::Float(*a as f64 + b),
                    (ScriptValue::Float(a), ScriptValue::Integer(b)) => ScriptValue::Float(a + *b as f64),
                    (ScriptValue::String(a), ScriptValue::String(b)) => ScriptValue::String(format!("{}{}", a, b)),
                    _ => return Err(ScriptError::TypeError(
                        format!("Expected number or string for addition, found {} + {}", left_val.type_name(), right_val.type_name())
                    ))
                };
                
                state.registers.insert(target.clone(), result);
                Ok(InstructionResult::Continue)
            }
            Instruction::Subtract { target, left, right } => {
                let left_val = state.registers.get(left).unwrap_or(&ScriptValue::Integer(0));
                let right_val = state.registers.get(right).unwrap_or(&ScriptValue::Integer(0));
                
                let result = match (left_val, right_val) {
                    (ScriptValue::Integer(a), ScriptValue::Integer(b)) => ScriptValue::Integer(a - b),
                    (ScriptValue::Float(a), ScriptValue::Float(b)) => ScriptValue::Float(a - b),
                    (ScriptValue::Integer(a), ScriptValue::Float(b)) => ScriptValue::Float(*a as f64 - b),
                    (ScriptValue::Float(a), ScriptValue::Integer(b)) => ScriptValue::Float(a - *b as f64),
                    _ => return Err(ScriptError::TypeError(
                        format!("Expected numbers for subtraction, found {} - {}", left_val.type_name(), right_val.type_name())
                    ))
                };
                
                state.registers.insert(target.clone(), result);
                Ok(InstructionResult::Continue)
            }
            Instruction::Multiply { target, left, right } => {
                let left_val = state.registers.get(left).unwrap_or(&ScriptValue::Integer(1));
                let right_val = state.registers.get(right).unwrap_or(&ScriptValue::Integer(1));
                
                let result = match (left_val, right_val) {
                    (ScriptValue::Integer(a), ScriptValue::Integer(b)) => ScriptValue::Integer(a * b),
                    (ScriptValue::Float(a), ScriptValue::Float(b)) => ScriptValue::Float(a * b),
                    (ScriptValue::Integer(a), ScriptValue::Float(b)) => ScriptValue::Float(*a as f64 * b),
                    (ScriptValue::Float(a), ScriptValue::Integer(b)) => ScriptValue::Float(a * *b as f64),
                    _ => return Err(ScriptError::TypeError(
                        format!("Expected numbers for multiplication, found {} * {}", left_val.type_name(), right_val.type_name())
                    ))
                };
                
                state.registers.insert(target.clone(), result);
                Ok(InstructionResult::Continue)
            }
            Instruction::Divide { target, left, right } => {
                let left_val = state.registers.get(left).unwrap_or(&ScriptValue::Integer(1));
                let right_val = state.registers.get(right).unwrap_or(&ScriptValue::Integer(1));
                
                let result = match (left_val, right_val) {
                    (ScriptValue::Integer(a), ScriptValue::Integer(b)) => {
                        if *b == 0 {
                            return Err(ScriptError::RuntimeError("Division by zero".to_string()));
                        }
                        ScriptValue::Integer(a / b)
                    }
                    (ScriptValue::Float(a), ScriptValue::Float(b)) => {
                        if *b == 0.0 {
                            return Err(ScriptError::RuntimeError("Division by zero".to_string()));
                        }
                        ScriptValue::Float(a / b)
                    }
                    (ScriptValue::Integer(a), ScriptValue::Float(b)) => {
                        if *b == 0.0 {
                            return Err(ScriptError::RuntimeError("Division by zero".to_string()));
                        }
                        ScriptValue::Float(*a as f64 / b)
                    }
                    (ScriptValue::Float(a), ScriptValue::Integer(b)) => {
                        if *b == 0 {
                            return Err(ScriptError::RuntimeError("Division by zero".to_string()));
                        }
                        ScriptValue::Float(a / *b as f64)
                    }
                    _ => return Err(ScriptError::TypeError(
                        format!("Expected numbers for division, found {} / {}", left_val.type_name(), right_val.type_name())
                    ))
                };
                
                state.registers.insert(target.clone(), result);
                Ok(InstructionResult::Continue)
            }
            Instruction::Compare { target, left, right, operation } => {
                let left_val = state.registers.get(left).unwrap_or(&ScriptValue::None);
                let right_val = state.registers.get(right).unwrap_or(&ScriptValue::None);
                
                let result = match operation {
                    CompareOp::Equal => left_val == right_val,
                    CompareOp::NotEqual => left_val != right_val,
                    CompareOp::Less => left_val.as_number() < right_val.as_number(),
                    CompareOp::LessEqual => left_val.as_number() <= right_val.as_number(),
                    CompareOp::Greater => left_val.as_number() > right_val.as_number(),
                    CompareOp::GreaterEqual => left_val.as_number() >= right_val.as_number(),
                };
                
                state.registers.insert(target.clone(), ScriptValue::Boolean(result));
                Ok(InstructionResult::Continue)
            }
            Instruction::Log { message } => {
                println!("[Script Log] {}", message);
                Ok(InstructionResult::Continue)
            }
            Instruction::CallFunction { target, function, args } => {
                let arg_values: Vec<_> = args.iter()
                    .map(|arg| state.registers.get(arg).unwrap_or(&ScriptValue::None).clone())
                    .collect();
                
                let result = self.instruction_set.call_function(function, &arg_values)?;
                
                if let Some(target_reg) = target {
                    state.registers.insert(target_reg.clone(), result);
                }
                
                Ok(InstructionResult::Continue)
            }
            Instruction::Return { value } => {
                let return_value = if let Some(val_reg) = value {
                    state.registers.get(val_reg).unwrap_or(&ScriptValue::None).clone()
                } else {
                    ScriptValue::None
                };
                Ok(InstructionResult::Return(return_value))
            }
            Instruction::WaitSeconds { duration } => {
                Ok(InstructionResult::Suspend(*duration))
            }
            Instruction::Nop => Ok(InstructionResult::Continue),
            _ => {
                println!("Warning: Unimplemented instruction: {:?}", instruction);
                Ok(InstructionResult::Continue)
            }
        }
    }

    pub fn get_execution_stats(&self) -> RuntimeStats {
        RuntimeStats {
            cached_scripts: self.execution_cache.len(),
            total_compile_time_ms: self.execution_cache.values()
                .map(|cache| cache.compile_time_ms)
                .sum(),
            average_compile_time_ms: if !self.execution_cache.is_empty() {
                self.execution_cache.values()
                    .map(|cache| cache.compile_time_ms)
                    .sum::<u64>() as f32 / self.execution_cache.len() as f32
            } else {
                0.0
            },
            builtin_functions: self.instruction_set.builtin_functions.len(),
        }
    }

    /// Initialize the script runtime system
    pub fn initialize(&mut self) -> RobinResult<()> {
        // Clear any existing cache
        self.execution_cache.clear();
        log::info!("Script runtime initialized");
        Ok(())
    }

    /// Update the script runtime (called each frame)
    pub fn update(&mut self, _delta_time: f32) -> RobinResult<()> {
        // Clean up old cache entries
        let current_time = Instant::now();
        self.execution_cache.retain(|_, cached| {
            current_time.duration_since(cached.last_compiled).as_secs() < 300 // 5 minutes
        });
        Ok(())
    }

    /// Execute a script by ID
    pub fn execute_script(&mut self, script_id: &str, context: &mut ExecutionContext) -> ScriptResult<ScriptValue> {
        // For now, return a default value since we need the actual script
        // This would normally look up the script by ID and execute it
        log::warn!("execute_script called with script_id: {}", script_id);
        Ok(ScriptValue::Null)
    }

    /// Pause a running script
    pub fn pause_script(&mut self, script_id: &str) -> RobinResult<()> {
        log::info!("Pausing script: {}", script_id);
        // Script pausing logic would go here
        Ok(())
    }

    /// Resume a paused script
    pub fn resume_script(&mut self, script_id: &str) -> RobinResult<()> {
        log::info!("Resuming script: {}", script_id);
        // Script resuming logic would go here
        Ok(())
    }

    /// Stop a running script
    pub fn stop_script(&mut self, script_id: &str) -> RobinResult<()> {
        log::info!("Stopping script: {}", script_id);
        // Script stopping logic would go here
        Ok(())
    }

    /// Reload scripts that have been modified
    pub fn reload_modified_scripts(&mut self) -> RobinResult<()> {
        // Clear cache to force recompilation
        self.execution_cache.clear();
        log::info!("Reloaded modified scripts");
        Ok(())
    }

    /// Optimize a script for better performance
    pub fn optimize_script(&self, script_id: &str) -> RobinResult<()> {
        log::info!("Optimizing script: {}", script_id);
        // Script optimization logic would go here
        Ok(())
    }

    /// Get the number of currently active scripts
    pub fn get_active_script_count(&self) -> usize {
        self.execution_cache.len()
    }

    /// Get the number of nodes executed this frame
    pub fn get_nodes_executed_this_frame(&self) -> u64 {
        // This would track actual execution statistics
        0
    }

    /// Get current memory usage in MB
    pub fn get_memory_usage_mb(&self) -> f64 {
        // This would calculate actual memory usage
        (self.execution_cache.len() * 1024) as f64 / (1024.0 * 1024.0) // Rough estimate
    }

    /// Shutdown the script runtime
    pub fn shutdown(&mut self) -> RobinResult<()> {
        self.execution_cache.clear();
        log::info!("Script runtime shutdown");
        Ok(())
    }
}

#[derive(Debug)]
struct ExecutionState {
    current_node: usize,
    registers: HashMap<String, ScriptValue>,
    suspended_until: Option<Instant>,
}

#[derive(Debug)]
enum InstructionResult {
    Continue,
    Jump(usize),
    Return(ScriptValue),
    Suspend(f32),
}

#[derive(Debug)]
pub struct RuntimeStats {
    pub cached_scripts: usize,
    pub total_compile_time_ms: u64,
    pub average_compile_time_ms: f32,
    pub builtin_functions: usize,
}

pub trait ScriptExecutor {
    fn execute_script(&mut self, script_id: &str, context: &mut ExecutionContext) -> ScriptResult<ScriptValue>;
    fn validate_script(&self, script_id: &str) -> Vec<ScriptError>;
    fn get_execution_info(&self, script_id: &str) -> Option<ExecutionInfo>;
}

#[derive(Debug, Clone)]
pub struct ExecutionInfo {
    pub script_id: String,
    pub last_execution: Instant,
    pub execution_count: u64,
    pub total_execution_time: Duration,
    pub average_execution_time: Duration,
    pub last_result: Option<ScriptValue>,
    pub error_count: u64,
}

// Add fastrand dependency for random number generation

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::scripting::*;

    #[test]
    fn test_execution_context() {
        let mut context = ExecutionContext::default();
        
        context.set_variable("test_var".to_string(), ScriptValue::Integer(42));
        assert_eq!(context.get_variable("test_var"), Some(&ScriptValue::Integer(42)));
        
        assert!(context.push_call_frame("script1".to_string(), "node1".to_string()).is_ok());
        context.set_variable("local_var".to_string(), ScriptValue::String("test".to_string()));
        assert_eq!(context.get_variable("local_var"), Some(&ScriptValue::String("test".to_string())));
        
        context.pop_call_frame();
        assert_eq!(context.get_variable("local_var"), None);
    }

    #[test]
    fn test_instruction_set() {
        let instruction_set = InstructionSet::new();
        
        let result = instruction_set.call_function("abs", &[ScriptValue::Integer(-5)]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ScriptValue::Integer(5));
        
        let result = instruction_set.call_function("sqrt", &[ScriptValue::Float(16.0)]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ScriptValue::Float(4.0));
        
        let result = instruction_set.call_function("string_length", &[ScriptValue::String("hello".to_string())]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ScriptValue::Integer(5));
    }

    #[test]
    fn test_script_runtime_creation() {
        let config = ScriptingSystemConfig::default();
        let runtime = ScriptRuntime::new(config);
        assert!(runtime.is_ok());
    }

    #[test]
    fn test_compare_operations() {
        let left = ScriptValue::Integer(5);
        let right = ScriptValue::Integer(3);
        
        assert_eq!(left.as_number() > right.as_number(), true);
        assert_eq!(left.as_number() < right.as_number(), false);
        assert_eq!(left.as_number() == right.as_number(), false);
    }
}