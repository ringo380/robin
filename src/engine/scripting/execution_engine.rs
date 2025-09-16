// Script Execution Engine for Robin Engine
// Compiles and executes visual scripts with real-time debugging support

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use crate::engine::error::RobinResult;
use super::{ScriptValue, visual_editor::NodeGraph, CompiledScript, ScriptMetadata};

/// Script execution engine with compilation and runtime support
#[derive(Debug)]
pub struct ScriptExecutor {
    /// Bytecode compiler
    compiler: ScriptCompiler,
    
    /// Virtual machine for execution
    virtual_machine: ScriptVM,
    
    /// Active execution contexts
    active_contexts: HashMap<String, ExecutionContext>,
    
    /// Execution queue for scheduled scripts
    execution_queue: VecDeque<QueuedExecution>,
    
    /// Performance profiler
    profiler: ExecutionProfiler,
    
    /// Memory manager for script data
    memory_manager: ScriptMemoryManager,
}

impl ScriptExecutor {
    pub fn new() -> Self {
        Self {
            compiler: ScriptCompiler::new(),
            virtual_machine: ScriptVM::new(),
            active_contexts: HashMap::new(),
            execution_queue: VecDeque::new(),
            profiler: ExecutionProfiler::new(),
            memory_manager: ScriptMemoryManager::new(),
        }
    }
    
    /// Initialize the execution engine
    pub fn initialize(&mut self) -> RobinResult<()> {
        // Setup standard library functions
        self.virtual_machine.register_standard_functions()?;
        
        // Initialize memory pools
        self.memory_manager.initialize()?;
        
        // Setup profiling
        self.profiler.enable_profiling(true);
        
        println!("Script execution engine initialized");
        Ok(())
    }
    
    /// Compile a visual script graph into bytecode
    pub fn compile_graph(&mut self, graph: &NodeGraph) -> RobinResult<CompiledScript> {
        let start_time = std::time::Instant::now();
        
        // Validate graph first
        let validation = graph.validate()?;
        if !validation.is_valid {
            return Err(format!("Graph validation failed: {:?}", validation.errors).into());
        }
        
        // Compile to intermediate representation
        let intermediate = self.compiler.compile_to_intermediate(graph)?;
        
        // Optimize intermediate code
        let optimized = self.compiler.optimize_intermediate(intermediate)?;
        
        // Generate final bytecode
        let bytecode = self.compiler.generate_bytecode(optimized)?;
        
        let compile_time = start_time.elapsed();
        
        let compiled_script = CompiledScript {
            bytecode,
            metadata: ScriptMetadata {
                name: graph.metadata.name.clone(),
                version: "1.0.0".to_string(),
                author: "Robin Engine".to_string(),
                description: graph.metadata.description.clone(),
                tags: Vec::new(),
            },
        };
        
        self.profiler.record_compilation(graph.id.clone(), compile_time);
        
        println!("Compiled script '{}' in {:.3}ms", graph.metadata.name, compile_time.as_secs_f64() * 1000.0);
        Ok(compiled_script)
    }
    
    /// Execute compiled script
    pub fn execute(&mut self, script: &CompiledScript, context: ExecutionContext) -> RobinResult<super::ScriptResult> {
        let execution_id = uuid::Uuid::new_v4().to_string();
        let start_time = std::time::Instant::now();
        
        // Setup execution context
        self.active_contexts.insert(execution_id.clone(), context);
        
        // Execute on virtual machine
        let result = self.virtual_machine.execute(&script.bytecode, &execution_id)?;
        
        // Cleanup context
        self.active_contexts.remove(&execution_id);
        
        let execution_time = start_time.elapsed();
        self.profiler.record_execution(script.metadata.name.clone(), execution_time, result.clone());
        
        Ok(result)
    }
    
    /// Execute with debugging support
    pub fn execute_with_debugging(&mut self, script: &CompiledScript, context: ExecutionContext, debugger: &mut super::ScriptDebugger) -> RobinResult<super::ScriptResult> {
        let execution_id = uuid::Uuid::new_v4().to_string();
        
        // Setup debug session
        debugger.start_debug_session(execution_id.clone(), script.metadata.name.clone())?;
        
        // Setup execution context with debugging
        let mut debug_context = context;
        debug_context.debug_mode = true;
        debug_context.execution_id = Some(execution_id.clone());
        self.active_contexts.insert(execution_id.clone(), debug_context);
        
        // Execute with debug hooks
        let result = self.virtual_machine.execute_with_debug(&script.bytecode, &execution_id, debugger)?;
        
        // End debug session
        debugger.end_debug_session(&execution_id)?;
        
        // Cleanup
        self.active_contexts.remove(&execution_id);
        
        Ok(result)
    }
    
    /// Update execution engine (call every frame)
    pub fn update(&mut self, delta_time: f32) -> RobinResult<()> {
        // Process execution queue
        self.process_execution_queue(delta_time)?;
        
        // Update active contexts
        self.update_active_contexts(delta_time)?;
        
        // Update profiler
        self.profiler.update(delta_time)?;
        
        // Cleanup memory
        self.memory_manager.cleanup()?;
        
        Ok(())
    }
    
    /// Schedule script for execution
    pub fn schedule_execution(&mut self, script: CompiledScript, context: ExecutionContext, delay: f32) {
        let queued = QueuedExecution {
            script,
            context,
            delay,
            scheduled_time: std::time::Instant::now() + std::time::Duration::from_secs_f32(delay),
        };
        
        self.execution_queue.push_back(queued);
    }
    
    /// Process queued executions
    fn process_execution_queue(&mut self, _delta_time: f32) -> RobinResult<()> {
        let now = std::time::Instant::now();
        
        while let Some(queued) = self.execution_queue.front() {
            if queued.scheduled_time <= now {
                let queued = self.execution_queue.pop_front().unwrap();
                self.execute(&queued.script, queued.context)?;
            } else {
                break;
            }
        }
        
        Ok(())
    }
    
    /// Update active execution contexts
    fn update_active_contexts(&mut self, delta_time: f32) -> RobinResult<()> {
        for context in self.active_contexts.values_mut() {
            context.elapsed_time += delta_time;
            context.frame_count += 1;
        }
        Ok(())
    }
    
    /// Get execution context
    pub fn get_execution_context(&self, execution_id: &str) -> Option<&ExecutionContext> {
        self.active_contexts.get(execution_id)
    }
    
    /// Get profiling data
    pub fn get_profiling_data(&self) -> &ExecutionProfiler {
        &self.profiler
    }
    
    /// Get memory usage statistics
    pub fn get_memory_stats(&self) -> MemoryStats {
        self.memory_manager.get_stats()
    }
}

/// Script compiler that converts visual graphs to bytecode
#[derive(Debug)]
pub struct ScriptCompiler {
    /// Optimization level
    optimization_level: OptimizationLevel,
    
    /// Compilation cache
    compilation_cache: HashMap<String, CachedCompilation>,
    
    /// Symbol table for variables and functions
    symbol_table: SymbolTable,
}

impl ScriptCompiler {
    pub fn new() -> Self {
        Self {
            optimization_level: OptimizationLevel::Standard,
            compilation_cache: HashMap::new(),
            symbol_table: SymbolTable::new(),
        }
    }
    
    /// Compile graph to intermediate representation
    pub fn compile_to_intermediate(&mut self, graph: &NodeGraph) -> RobinResult<IntermediateCode> {
        let mut code = IntermediateCode::new();
        
        // Find entry points (nodes with no required inputs)
        let entry_points = self.find_entry_points(graph)?;
        
        if entry_points.is_empty() {
            return Err("No entry points found in graph".into());
        }
        
        // Compile each execution path
        for entry_point in entry_points {
            self.compile_node_tree(graph, &entry_point, &mut code)?;
        }
        
        Ok(code)
    }
    
    /// Find entry point nodes
    fn find_entry_points(&self, graph: &NodeGraph) -> RobinResult<Vec<String>> {
        let mut entry_points = Vec::new();
        
        for (node_id, node) in &graph.nodes {
            // Check if node has execution inputs
            let has_exec_input = node.inputs.iter()
                .any(|input| matches!(input.data_type, super::visual_editor::DataType::Exec));
            
            if !has_exec_input || node.node_type == "Start" || node.node_type.starts_with("Event") {
                entry_points.push(node_id.clone());
            }
        }
        
        Ok(entry_points)
    }
    
    /// Compile a node tree starting from a specific node
    fn compile_node_tree(&mut self, graph: &NodeGraph, node_id: &str, code: &mut IntermediateCode) -> RobinResult<()> {
        let node = graph.nodes.get(node_id)
            .ok_or_else(|| format!("Node not found: {}", node_id))?;
        
        // Generate instruction based on node type
        match node.node_type.as_str() {
            "Start" => {
                code.add_instruction(IRInstruction::StartExecution);
            }
            "PlaceBlock" => {
                // Load parameters
                if let Some(ScriptValue::Vector3(pos)) = node.properties.get("position") {
                    code.add_instruction(IRInstruction::LoadConstant(ScriptValue::Vector3(*pos)));
                }
                if let Some(ScriptValue::String(block_type)) = node.properties.get("block_type") {
                    code.add_instruction(IRInstruction::LoadConstant(ScriptValue::String(block_type.clone())));
                }
                
                // Call place block function
                code.add_instruction(IRInstruction::CallFunction("place_block".to_string(), 2));
            }
            "Add" => {
                // Load operands (would be from connected inputs in real implementation)
                code.add_instruction(IRInstruction::LoadConstant(ScriptValue::Float(0.0))); // Placeholder
                code.add_instruction(IRInstruction::LoadConstant(ScriptValue::Float(0.0))); // Placeholder
                code.add_instruction(IRInstruction::Add);
            }
            "WaitTime" => {
                if let Some(ScriptValue::Float(duration)) = node.properties.get("duration") {
                    code.add_instruction(IRInstruction::LoadConstant(ScriptValue::Float(*duration)));
                    code.add_instruction(IRInstruction::Wait);
                }
            }
            _ => {
                // Generic node handling
                code.add_instruction(IRInstruction::NoOp);
            }
        }
        
        // Find connected output nodes and compile them
        for connection in &graph.connections {
            if connection.from_node == *node_id {
                self.compile_node_tree(graph, &connection.to_node, code)?;
            }
        }
        
        Ok(())
    }
    
    /// Optimize intermediate code
    pub fn optimize_intermediate(&self, mut code: IntermediateCode) -> RobinResult<IntermediateCode> {
        match self.optimization_level {
            OptimizationLevel::None => Ok(code),
            OptimizationLevel::Basic => {
                self.apply_basic_optimizations(&mut code)?;
                Ok(code)
            }
            OptimizationLevel::Standard => {
                self.apply_basic_optimizations(&mut code)?;
                self.apply_standard_optimizations(&mut code)?;
                Ok(code)
            }
            OptimizationLevel::Aggressive => {
                self.apply_basic_optimizations(&mut code)?;
                self.apply_standard_optimizations(&mut code)?;
                self.apply_aggressive_optimizations(&mut code)?;
                Ok(code)
            }
        }
    }
    
    fn apply_basic_optimizations(&self, code: &mut IntermediateCode) -> RobinResult<()> {
        // Remove consecutive NoOp instructions
        code.instructions.retain(|inst| !matches!(inst, IRInstruction::NoOp));
        Ok(())
    }
    
    fn apply_standard_optimizations(&self, _code: &mut IntermediateCode) -> RobinResult<()> {
        // Constant folding, dead code elimination, etc.
        Ok(())
    }
    
    fn apply_aggressive_optimizations(&self, _code: &mut IntermediateCode) -> RobinResult<()> {
        // Function inlining, loop unrolling, etc.
        Ok(())
    }
    
    /// Generate final bytecode from intermediate representation
    pub fn generate_bytecode(&self, code: IntermediateCode) -> RobinResult<Vec<u8>> {
        let mut bytecode = Vec::new();
        
        for instruction in code.instructions {
            self.encode_instruction(instruction, &mut bytecode)?;
        }
        
        Ok(bytecode)
    }
    
    /// Encode a single instruction to bytecode
    fn encode_instruction(&self, instruction: IRInstruction, bytecode: &mut Vec<u8>) -> RobinResult<()> {
        match instruction {
            IRInstruction::NoOp => bytecode.push(0x00),
            IRInstruction::StartExecution => bytecode.push(0x01),
            IRInstruction::LoadConstant(value) => {
                bytecode.push(0x02);
                self.encode_value(value, bytecode)?;
            }
            IRInstruction::Add => bytecode.push(0x10),
            IRInstruction::Subtract => bytecode.push(0x11),
            IRInstruction::Multiply => bytecode.push(0x12),
            IRInstruction::Divide => bytecode.push(0x13),
            IRInstruction::CallFunction(name, arg_count) => {
                bytecode.push(0x20);
                self.encode_string(name, bytecode)?;
                bytecode.push(arg_count);
            }
            IRInstruction::Wait => bytecode.push(0x30),
            IRInstruction::Jump(offset) => {
                bytecode.push(0x40);
                bytecode.extend_from_slice(&(offset as u32).to_le_bytes());
            }
            IRInstruction::JumpIf(offset) => {
                bytecode.push(0x41);
                bytecode.extend_from_slice(&(offset as u32).to_le_bytes());
            }
        }
        
        Ok(())
    }
    
    fn encode_value(&self, value: ScriptValue, bytecode: &mut Vec<u8>) -> RobinResult<()> {
        match value {
            ScriptValue::None => bytecode.push(0x00),
            ScriptValue::Bool(b) => {
                bytecode.push(0x01);
                bytecode.push(if b { 1 } else { 0 });
            }
            ScriptValue::Int(i) => {
                bytecode.push(0x02);
                bytecode.extend_from_slice(&i.to_le_bytes());
            }
            ScriptValue::Float(f) => {
                bytecode.push(0x03);
                bytecode.extend_from_slice(&f.to_le_bytes());
            }
            ScriptValue::String(s) => {
                bytecode.push(0x04);
                self.encode_string(s, bytecode)?;
            }
            ScriptValue::Vector3(v) => {
                bytecode.push(0x05);
                for component in v.iter() {
                    bytecode.extend_from_slice(&component.to_le_bytes());
                }
            }
            ScriptValue::Color(c) => {
                bytecode.push(0x06);
                for component in c.iter() {
                    bytecode.extend_from_slice(&component.to_le_bytes());
                }
            }
            ScriptValue::Object(_) => {
                bytecode.push(0x07);
                // Object encoding would be more complex
            }
            ScriptValue::Array(_) => {
                bytecode.push(0x08);
                // Array encoding would be more complex
            }
        }
        Ok(())
    }
    
    fn encode_string(&self, s: String, bytecode: &mut Vec<u8>) -> RobinResult<()> {
        let bytes = s.as_bytes();
        bytecode.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        bytecode.extend_from_slice(bytes);
        Ok(())
    }
}

/// Virtual machine for executing compiled scripts
#[derive(Debug)]
pub struct ScriptVM {
    /// Instruction pointer
    instruction_pointer: usize,
    
    /// Value stack
    stack: Vec<ScriptValue>,
    
    /// Call stack
    call_stack: Vec<CallFrame>,
    
    /// Global variables
    globals: HashMap<String, ScriptValue>,
    
    /// Built-in function registry
    builtin_functions: HashMap<String, BuiltinFunction>,
    
    /// Execution state
    state: VMState,
}

impl ScriptVM {
    pub fn new() -> Self {
        Self {
            instruction_pointer: 0,
            stack: Vec::new(),
            call_stack: Vec::new(),
            globals: HashMap::new(),
            builtin_functions: HashMap::new(),
            state: VMState::Ready,
        }
    }
    
    /// Register standard library functions
    pub fn register_standard_functions(&mut self) -> RobinResult<()> {
        // Math functions
        self.register_builtin("add", BuiltinFunction::Add);
        self.register_builtin("subtract", BuiltinFunction::Subtract);
        self.register_builtin("multiply", BuiltinFunction::Multiply);
        self.register_builtin("divide", BuiltinFunction::Divide);
        
        // World interaction functions
        self.register_builtin("place_block", BuiltinFunction::PlaceBlock);
        self.register_builtin("remove_block", BuiltinFunction::RemoveBlock);
        self.register_builtin("get_block", BuiltinFunction::GetBlock);
        
        // NPC functions
        self.register_builtin("move_npc", BuiltinFunction::MoveNPC);
        self.register_builtin("npc_say", BuiltinFunction::NPCSay);
        
        // Utility functions
        self.register_builtin("wait", BuiltinFunction::Wait);
        self.register_builtin("log", BuiltinFunction::Log);
        
        println!("Registered {} standard functions", self.builtin_functions.len());
        Ok(())
    }
    
    fn register_builtin(&mut self, name: &str, function: BuiltinFunction) {
        self.builtin_functions.insert(name.to_string(), function);
    }
    
    /// Execute bytecode
    pub fn execute(&mut self, bytecode: &[u8], execution_id: &str) -> RobinResult<super::ScriptResult> {
        self.reset_state();
        self.instruction_pointer = 0;
        self.state = VMState::Running;
        
        while self.instruction_pointer < bytecode.len() && self.state == VMState::Running {
            let opcode = bytecode[self.instruction_pointer];
            self.instruction_pointer += 1;
            
            self.execute_instruction(opcode, bytecode, execution_id)?;
        }
        
        match self.state {
            VMState::Completed => {
                let result = self.stack.pop().unwrap_or(ScriptValue::None);
                Ok(super::ScriptResult::Success(result))
            }
            VMState::Error(ref msg) => {
                Ok(super::ScriptResult::Error(msg.clone()))
            }
            VMState::Yielded => {
                let result = self.stack.pop().unwrap_or(ScriptValue::None);
                Ok(super::ScriptResult::Yield(result))
            }
            _ => Ok(super::ScriptResult::Success(ScriptValue::None))
        }
    }
    
    /// Execute with debugging hooks
    pub fn execute_with_debug(&mut self, bytecode: &[u8], execution_id: &str, debugger: &mut super::ScriptDebugger) -> RobinResult<super::ScriptResult> {
        self.reset_state();
        self.instruction_pointer = 0;
        self.state = VMState::Running;
        
        while self.instruction_pointer < bytecode.len() && self.state == VMState::Running {
            // Debug hook - check breakpoints
            if debugger.has_breakpoint(self.instruction_pointer) {
                debugger.hit_breakpoint(execution_id, self.instruction_pointer, &self.stack)?;
                
                // Wait for debugger command
                match debugger.wait_for_command()? {
                    super::debugging::DebugCommand::Continue => {}
                    super::debugging::DebugCommand::StepOver => {
                        debugger.set_step_mode(true);
                    }
                    super::debugging::DebugCommand::StepInto => {
                        debugger.set_step_mode(true);
                    }
                    super::debugging::DebugCommand::Stop => {
                        self.state = VMState::Stopped;
                        break;
                    }
                }
            }
            
            let opcode = bytecode[self.instruction_pointer];
            self.instruction_pointer += 1;
            
            // Record execution trace
            debugger.record_execution_step(execution_id, self.instruction_pointer - 1, opcode, &self.stack)?;
            
            self.execute_instruction(opcode, bytecode, execution_id)?;
            
            // Check for step mode
            if debugger.is_step_mode() {
                debugger.hit_step(execution_id, self.instruction_pointer, &self.stack)?;
                debugger.wait_for_command()?;
            }
        }
        
        match self.state {
            VMState::Completed => {
                let result = self.stack.pop().unwrap_or(ScriptValue::None);
                Ok(super::ScriptResult::Success(result))
            }
            VMState::Error(ref msg) => {
                Ok(super::ScriptResult::Error(msg.clone()))
            }
            VMState::Yielded => {
                let result = self.stack.pop().unwrap_or(ScriptValue::None);
                Ok(super::ScriptResult::Yield(result))
            }
            _ => Ok(super::ScriptResult::Success(ScriptValue::None))
        }
    }
    
    /// Execute a single instruction
    fn execute_instruction(&mut self, opcode: u8, bytecode: &[u8], _execution_id: &str) -> RobinResult<()> {
        match opcode {
            0x00 => {} // NoOp
            0x01 => {  // StartExecution
                self.stack.push(ScriptValue::Bool(true));
            }
            0x02 => {  // LoadConstant
                let value = self.decode_value(bytecode)?;
                self.stack.push(value);
            }
            0x10 => {  // Add
                let b = self.stack.pop().ok_or("Stack underflow")?;
                let a = self.stack.pop().ok_or("Stack underflow")?;
                let result = self.perform_add(a, b)?;
                self.stack.push(result);
            }
            0x11 => {  // Subtract
                let b = self.stack.pop().ok_or("Stack underflow")?;
                let a = self.stack.pop().ok_or("Stack underflow")?;
                let result = self.perform_subtract(a, b)?;
                self.stack.push(result);
            }
            0x20 => {  // CallFunction
                let func_name = self.decode_string(bytecode)?;
                let arg_count = bytecode[self.instruction_pointer] as usize;
                self.instruction_pointer += 1;
                
                self.call_builtin_function(&func_name, arg_count)?;
            }
            0x30 => {  // Wait
                let duration = self.stack.pop().ok_or("Stack underflow")?;
                if let ScriptValue::Float(time) = duration {
                    // In a real implementation, this would yield execution and resume after the delay
                    println!("Waiting for {} seconds", time);
                    self.stack.push(ScriptValue::Bool(true));
                }
            }
            _ => {
                return Err(format!("Unknown opcode: 0x{:02X}", opcode).into());
            }
        }
        
        Ok(())
    }
    
    fn decode_value(&mut self, bytecode: &[u8]) -> RobinResult<ScriptValue> {
        if self.instruction_pointer >= bytecode.len() {
            return Err("Unexpected end of bytecode".into());
        }
        
        let value_type = bytecode[self.instruction_pointer];
        self.instruction_pointer += 1;
        
        match value_type {
            0x00 => Ok(ScriptValue::None),
            0x01 => {
                let b = bytecode[self.instruction_pointer] != 0;
                self.instruction_pointer += 1;
                Ok(ScriptValue::Bool(b))
            }
            0x02 => {
                let bytes = &bytecode[self.instruction_pointer..self.instruction_pointer + 8];
                self.instruction_pointer += 8;
                let i = i64::from_le_bytes(bytes.try_into().unwrap());
                Ok(ScriptValue::Int(i))
            }
            0x03 => {
                let bytes = &bytecode[self.instruction_pointer..self.instruction_pointer + 8];
                self.instruction_pointer += 8;
                let f = f64::from_le_bytes(bytes.try_into().unwrap());
                Ok(ScriptValue::Float(f))
            }
            0x04 => {
                let s = self.decode_string(bytecode)?;
                Ok(ScriptValue::String(s))
            }
            0x05 => {
                let mut v = [0.0f32; 3];
                for i in 0..3 {
                    let bytes = &bytecode[self.instruction_pointer..self.instruction_pointer + 4];
                    self.instruction_pointer += 4;
                    v[i] = f32::from_le_bytes(bytes.try_into().unwrap());
                }
                Ok(ScriptValue::Vector3(v))
            }
            _ => Err(format!("Unknown value type: 0x{:02X}", value_type).into())
        }
    }
    
    fn decode_string(&mut self, bytecode: &[u8]) -> RobinResult<String> {
        let len_bytes = &bytecode[self.instruction_pointer..self.instruction_pointer + 4];
        self.instruction_pointer += 4;
        let len = u32::from_le_bytes(len_bytes.try_into().unwrap()) as usize;
        
        let string_bytes = &bytecode[self.instruction_pointer..self.instruction_pointer + len];
        self.instruction_pointer += len;
        
        String::from_utf8(string_bytes.to_vec()).map_err(|e| format!("Invalid UTF-8: {}", e).into())
    }
    
    fn perform_add(&self, a: ScriptValue, b: ScriptValue) -> RobinResult<ScriptValue> {
        match (a, b) {
            (ScriptValue::Int(a), ScriptValue::Int(b)) => Ok(ScriptValue::Int(a + b)),
            (ScriptValue::Float(a), ScriptValue::Float(b)) => Ok(ScriptValue::Float(a + b)),
            (ScriptValue::Int(a), ScriptValue::Float(b)) => Ok(ScriptValue::Float(a as f64 + b)),
            (ScriptValue::Float(a), ScriptValue::Int(b)) => Ok(ScriptValue::Float(a + b as f64)),
            (ScriptValue::String(a), ScriptValue::String(b)) => Ok(ScriptValue::String(a + &b)),
            _ => Err("Incompatible types for addition".into())
        }
    }
    
    fn perform_subtract(&self, a: ScriptValue, b: ScriptValue) -> RobinResult<ScriptValue> {
        match (a, b) {
            (ScriptValue::Int(a), ScriptValue::Int(b)) => Ok(ScriptValue::Int(a - b)),
            (ScriptValue::Float(a), ScriptValue::Float(b)) => Ok(ScriptValue::Float(a - b)),
            (ScriptValue::Int(a), ScriptValue::Float(b)) => Ok(ScriptValue::Float(a as f64 - b)),
            (ScriptValue::Float(a), ScriptValue::Int(b)) => Ok(ScriptValue::Float(a - b as f64)),
            _ => Err("Incompatible types for subtraction".into())
        }
    }
    
    fn call_builtin_function(&mut self, func_name: &str, arg_count: usize) -> RobinResult<()> {
        if let Some(function) = self.builtin_functions.get(func_name).cloned() {
            // Pop arguments from stack
            let mut args = Vec::new();
            for _ in 0..arg_count {
                args.push(self.stack.pop().ok_or("Stack underflow")?);
            }
            args.reverse(); // Arguments were pushed in reverse order
            
            // Execute function
            let result = self.execute_builtin_function(function, args)?;
            
            // Push result
            self.stack.push(result);
        } else {
            return Err(format!("Unknown function: {}", func_name).into());
        }
        
        Ok(())
    }
    
    fn execute_builtin_function(&mut self, function: BuiltinFunction, args: Vec<ScriptValue>) -> RobinResult<ScriptValue> {
        match function {
            BuiltinFunction::Add => {
                if args.len() != 2 {
                    return Err("Add function requires 2 arguments".into());
                }
                self.perform_add(args[0].clone(), args[1].clone())
            }
            BuiltinFunction::PlaceBlock => {
                if args.len() != 2 {
                    return Err("PlaceBlock function requires 2 arguments".into());
                }
                
                if let (ScriptValue::Vector3(pos), ScriptValue::String(block_type)) = (&args[0], &args[1]) {
                    println!("Placing {} block at {:?}", block_type, pos);
                    // In real implementation, this would interface with the world construction system
                    Ok(ScriptValue::Bool(true))
                } else {
                    Err("Invalid arguments for PlaceBlock".into())
                }
            }
            BuiltinFunction::Log => {
                if !args.is_empty() {
                    println!("Script Log: {:?}", args[0]);
                }
                Ok(ScriptValue::None)
            }
            BuiltinFunction::Wait => {
                if let Some(ScriptValue::Float(duration)) = args.first() {
                    println!("Waiting for {} seconds", duration);
                    // In real implementation, this would pause execution
                    Ok(ScriptValue::Bool(true))
                } else {
                    Err("Wait function requires a numeric duration".into())
                }
            }
            _ => {
                println!("Builtin function {:?} not yet implemented", function);
                Ok(ScriptValue::None)
            }
        }
    }
    
    fn reset_state(&mut self) {
        self.instruction_pointer = 0;
        self.stack.clear();
        self.call_stack.clear();
        self.state = VMState::Ready;
    }
}

/// Execution context for script runs
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Context identifier
    pub context_id: String,
    
    /// Local variables
    pub locals: HashMap<String, ScriptValue>,
    
    /// Execution environment data
    pub environment: HashMap<String, ScriptValue>,
    
    /// Time since execution started
    pub elapsed_time: f32,
    
    /// Frame count
    pub frame_count: u64,
    
    /// Debug mode flag
    pub debug_mode: bool,
    
    /// Execution ID for debugging
    pub execution_id: Option<String>,
    
    /// Maximum execution time before timeout
    pub timeout: Option<f32>,
    
    /// Priority level
    pub priority: ExecutionPriority,
}

impl ExecutionContext {
    pub fn new(context_id: String) -> Self {
        Self {
            context_id,
            locals: HashMap::new(),
            environment: HashMap::new(),
            elapsed_time: 0.0,
            frame_count: 0,
            debug_mode: false,
            execution_id: None,
            timeout: Some(10.0), // 10 second default timeout
            priority: ExecutionPriority::Normal,
        }
    }
    
    pub fn with_environment(mut self, environment: HashMap<String, ScriptValue>) -> Self {
        self.environment = environment;
        self
    }
    
    pub fn with_timeout(mut self, timeout: f32) -> Self {
        self.timeout = Some(timeout);
        self
    }
    
    pub fn with_priority(mut self, priority: ExecutionPriority) -> Self {
        self.priority = priority;
        self
    }
}

// Supporting types and enums

#[derive(Debug, Clone, Copy)]
pub enum ExecutionPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

#[derive(Debug, Clone, Copy)]
pub enum OptimizationLevel {
    None,
    Basic,
    Standard,
    Aggressive,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VMState {
    Ready,
    Running,
    Paused,
    Completed,
    Error(String),
    Yielded,
    Stopped,
}

#[derive(Debug)]
pub struct QueuedExecution {
    pub script: CompiledScript,
    pub context: ExecutionContext,
    pub delay: f32,
    pub scheduled_time: std::time::Instant,
}

#[derive(Debug)]
pub struct IntermediateCode {
    pub instructions: Vec<IRInstruction>,
}

impl IntermediateCode {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
        }
    }
    
    pub fn add_instruction(&mut self, instruction: IRInstruction) {
        self.instructions.push(instruction);
    }
}

#[derive(Debug, Clone)]
pub enum IRInstruction {
    NoOp,
    StartExecution,
    LoadConstant(ScriptValue),
    Add,
    Subtract,
    Multiply,
    Divide,
    CallFunction(String, u8), // function name, argument count
    Wait,
    Jump(i32),    // relative offset
    JumpIf(i32),  // conditional jump
}

#[derive(Debug, Clone)]
pub enum BuiltinFunction {
    // Math
    Add,
    Subtract,
    Multiply,
    Divide,
    
    // World interaction
    PlaceBlock,
    RemoveBlock,
    GetBlock,
    
    // NPC functions
    MoveNPC,
    NPCSay,
    
    // Utility
    Wait,
    Log,
    Random,
}

#[derive(Debug)]
pub struct CallFrame {
    pub return_address: usize,
    pub locals: HashMap<String, ScriptValue>,
}

#[derive(Debug)]
pub struct SymbolTable {
    symbols: HashMap<String, Symbol>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct Symbol {
    pub name: String,
    pub symbol_type: SymbolType,
    pub address: usize,
}

#[derive(Debug)]
pub enum SymbolType {
    Variable,
    Function,
    Constant,
}

#[derive(Debug)]
pub struct CachedCompilation {
    pub graph_hash: u64,
    pub compiled_script: CompiledScript,
    pub compile_time: std::time::Instant,
}

#[derive(Debug)]
pub struct ExecutionProfiler {
    pub enabled: bool,
    pub compilation_stats: HashMap<String, CompilationStats>,
    pub execution_stats: HashMap<String, ExecutionStats>,
    pub total_executions: u64,
    pub total_compilation_time: std::time::Duration,
    pub total_execution_time: std::time::Duration,
}

impl ExecutionProfiler {
    pub fn new() -> Self {
        Self {
            enabled: false,
            compilation_stats: HashMap::new(),
            execution_stats: HashMap::new(),
            total_executions: 0,
            total_compilation_time: std::time::Duration::ZERO,
            total_execution_time: std::time::Duration::ZERO,
        }
    }
    
    pub fn enable_profiling(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    pub fn record_compilation(&mut self, script_name: String, compile_time: std::time::Duration) {
        if !self.enabled { return; }
        
        let stats = self.compilation_stats.entry(script_name).or_insert(CompilationStats::default());
        stats.compile_count += 1;
        stats.total_time += compile_time;
        stats.average_time = stats.total_time / stats.compile_count;
        
        self.total_compilation_time += compile_time;
    }
    
    pub fn record_execution(&mut self, script_name: String, execution_time: std::time::Duration, result: super::ScriptResult) {
        if !self.enabled { return; }
        
        let stats = self.execution_stats.entry(script_name).or_insert(ExecutionStats::default());
        stats.execution_count += 1;
        stats.total_time += execution_time;
        stats.average_time = stats.total_time / stats.execution_count;
        
        match result {
            super::ScriptResult::Success(_) => stats.success_count += 1,
            super::ScriptResult::Error(_) => stats.error_count += 1,
            super::ScriptResult::Yield(_) => stats.yield_count += 1,
        }
        
        self.total_executions += 1;
        self.total_execution_time += execution_time;
    }
    
    pub fn update(&mut self, _delta_time: f32) -> RobinResult<()> {
        // Update performance metrics
        Ok(())
    }
    
    pub fn get_summary(&self) -> ProfileSummary {
        ProfileSummary {
            total_executions: self.total_executions,
            total_compilation_time: self.total_compilation_time,
            total_execution_time: self.total_execution_time,
            average_compilation_time: if self.compilation_stats.is_empty() {
                std::time::Duration::ZERO
            } else {
                self.total_compilation_time / self.compilation_stats.len() as u32
            },
            average_execution_time: if self.total_executions > 0 {
                self.total_execution_time / self.total_executions as u32
            } else {
                std::time::Duration::ZERO
            },
        }
    }
}

#[derive(Debug, Default)]
pub struct CompilationStats {
    pub compile_count: u32,
    pub total_time: std::time::Duration,
    pub average_time: std::time::Duration,
}

#[derive(Debug, Default)]
pub struct ExecutionStats {
    pub execution_count: u32,
    pub success_count: u32,
    pub error_count: u32,
    pub yield_count: u32,
    pub total_time: std::time::Duration,
    pub average_time: std::time::Duration,
}

#[derive(Debug)]
pub struct ProfileSummary {
    pub total_executions: u64,
    pub total_compilation_time: std::time::Duration,
    pub total_execution_time: std::time::Duration,
    pub average_compilation_time: std::time::Duration,
    pub average_execution_time: std::time::Duration,
}

#[derive(Debug)]
pub struct ScriptMemoryManager {
    pub allocated_bytes: usize,
    pub max_memory: usize,
    pub gc_threshold: usize,
    pub allocations: HashMap<String, AllocationInfo>,
}

impl ScriptMemoryManager {
    pub fn new() -> Self {
        Self {
            allocated_bytes: 0,
            max_memory: 64 * 1024 * 1024, // 64 MB default limit
            gc_threshold: 32 * 1024 * 1024, // GC at 32 MB
            allocations: HashMap::new(),
        }
    }
    
    pub fn initialize(&mut self) -> RobinResult<()> {
        println!("Script memory manager initialized with {} MB limit", self.max_memory / 1024 / 1024);
        Ok(())
    }
    
    pub fn cleanup(&mut self) -> RobinResult<()> {
        if self.allocated_bytes > self.gc_threshold {
            self.run_garbage_collection()?;
        }
        Ok(())
    }
    
    fn run_garbage_collection(&mut self) -> RobinResult<()> {
        let before = self.allocated_bytes;
        
        // Simple mark-and-sweep would go here
        // For now, just clear unused allocations
        self.allocations.retain(|_, info| info.reference_count > 0);
        
        // Recalculate allocated bytes
        self.allocated_bytes = self.allocations.values().map(|info| info.size).sum();
        
        let freed = before - self.allocated_bytes;
        if freed > 0 {
            println!("Garbage collection freed {} bytes", freed);
        }
        
        Ok(())
    }
    
    pub fn get_stats(&self) -> MemoryStats {
        MemoryStats {
            allocated_bytes: self.allocated_bytes,
            max_memory: self.max_memory,
            allocation_count: self.allocations.len(),
            fragmentation_ratio: 0.0, // Would be calculated in real implementation
        }
    }
}

#[derive(Debug)]
pub struct AllocationInfo {
    pub size: usize,
    pub reference_count: u32,
    pub allocated_at: std::time::Instant,
}

#[derive(Debug)]
pub struct MemoryStats {
    pub allocated_bytes: usize,
    pub max_memory: usize,
    pub allocation_count: usize,
    pub fragmentation_ratio: f32,
}