// Script Debugging and Visualization Tools for Robin Engine
// Provides comprehensive debugging support for visual scripts and behavior trees

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use crate::engine::error::RobinResult;
use super::ScriptValue;

/// Script debugging system with breakpoints, step-through, and visualization
#[derive(Debug)]
pub struct ScriptDebugger {
    /// Active debug sessions
    debug_sessions: HashMap<String, DebugSession>,
    
    /// Global breakpoints
    breakpoints: HashMap<usize, Breakpoint>,
    
    /// Execution tracer for recording execution flow
    tracer: ExecutionTracer,
    
    /// Variable inspector
    variable_inspector: VariableInspector,
    
    /// Performance profiler
    performance_profiler: PerformanceProfiler,
    
    /// Debug UI state
    ui_state: DebugUIState,
    
    /// Debug settings
    settings: DebugSettings,
}

impl ScriptDebugger {
    pub fn new() -> Self {
        Self {
            debug_sessions: HashMap::new(),
            breakpoints: HashMap::new(),
            tracer: ExecutionTracer::new(),
            variable_inspector: VariableInspector::new(),
            performance_profiler: PerformanceProfiler::new(),
            ui_state: DebugUIState::default(),
            settings: DebugSettings::default(),
        }
    }
    
    /// Start a new debug session
    pub fn start_debug_session(&mut self, execution_id: String, script_name: String) -> RobinResult<()> {
        let session = DebugSession {
            execution_id: execution_id.clone(),
            script_name,
            state: DebugState::Running,
            breakpoints: Vec::new(),
            step_mode: false,
            call_stack: Vec::new(),
            local_variables: HashMap::new(),
            execution_trace: Vec::new(),
            started_at: chrono::Utc::now(),
            paused_at: None,
        };
        
        self.debug_sessions.insert(execution_id.clone(), session);
        self.tracer.start_trace(execution_id.clone());
        
        println!("Started debug session: {}", execution_id);
        Ok(())
    }
    
    /// End debug session
    pub fn end_debug_session(&mut self, execution_id: &str) -> RobinResult<()> {
        self.debug_sessions.remove(execution_id);
        self.tracer.end_trace(execution_id);
        
        println!("Ended debug session: {}", execution_id);
        Ok(())
    }
    
    /// Check if debugging is enabled for an execution
    pub fn is_debugging_enabled(&self, script_id: &str) -> bool {
        self.debug_sessions.contains_key(script_id) || self.settings.global_debug_mode
    }
    
    /// Add a breakpoint at specific instruction
    pub fn add_breakpoint(&mut self, instruction_address: usize, condition: Option<BreakpointCondition>) -> String {
        let breakpoint_id = uuid::Uuid::new_v4().to_string();
        let breakpoint = Breakpoint {
            id: breakpoint_id.clone(),
            instruction_address,
            condition,
            enabled: true,
            hit_count: 0,
            created_at: chrono::Utc::now(),
        };
        
        self.breakpoints.insert(instruction_address, breakpoint);
        println!("Added breakpoint at instruction {}", instruction_address);
        
        breakpoint_id
    }
    
    /// Remove breakpoint
    pub fn remove_breakpoint(&mut self, instruction_address: usize) -> bool {
        if self.breakpoints.remove(&instruction_address).is_some() {
            println!("Removed breakpoint at instruction {}", instruction_address);
            true
        } else {
            false
        }
    }
    
    /// Check if there's a breakpoint at given address
    pub fn has_breakpoint(&self, instruction_address: usize) -> bool {
        self.breakpoints.get(&instruction_address)
            .map(|bp| bp.enabled)
            .unwrap_or(false)
    }
    
    /// Handle breakpoint hit
    pub fn hit_breakpoint(&mut self, execution_id: &str, instruction_address: usize, stack: &[ScriptValue]) -> RobinResult<()> {
        if let Some(breakpoint) = self.breakpoints.get_mut(&instruction_address) {
            breakpoint.hit_count += 1;
            
            // Check condition if present
            if let Some(ref condition) = breakpoint.condition {
                if !self.evaluate_breakpoint_condition(condition, stack)? {
                    return Ok(());
                }
            }
            
            // Pause execution
            if let Some(session) = self.debug_sessions.get_mut(execution_id) {
                session.state = DebugState::Paused;
                session.paused_at = Some(chrono::Utc::now());
                
                println!("Hit breakpoint at instruction {} (hit count: {})", 
                        instruction_address, breakpoint.hit_count);
                
                // Capture current state
                self.capture_execution_state(execution_id, instruction_address, stack)?;
            }
        }
        
        Ok(())
    }
    
    /// Handle step execution
    pub fn hit_step(&mut self, execution_id: &str, instruction_address: usize, stack: &[ScriptValue]) -> RobinResult<()> {
        if let Some(session) = self.debug_sessions.get_mut(execution_id) {
            if session.step_mode {
                session.state = DebugState::Paused;
                session.paused_at = Some(chrono::Utc::now());
                
                println!("Step pause at instruction {}", instruction_address);
                self.capture_execution_state(execution_id, instruction_address, stack)?;
            }
        }
        
        Ok(())
    }
    
    /// Record execution step for tracing
    pub fn record_execution_step(&mut self, execution_id: &str, instruction_address: usize, opcode: u8, stack: &[ScriptValue]) -> RobinResult<()> {
        let step = ExecutionStep {
            instruction_address,
            opcode,
            stack_snapshot: stack.to_vec(),
            timestamp: chrono::Utc::now(),
        };
        
        // Add to session trace
        if let Some(session) = self.debug_sessions.get_mut(execution_id) {
            session.execution_trace.push(step.clone());
            
            // Limit trace size
            if session.execution_trace.len() > 1000 {
                session.execution_trace.remove(0);
            }
        }
        
        // Add to global tracer
        self.tracer.record_step(execution_id.to_string(), step);
        
        Ok(())
    }
    
    /// Wait for debugger command (in real implementation this would be async)
    pub fn wait_for_command(&self) -> RobinResult<DebugCommand> {
        // In a real implementation, this would wait for input from debug UI
        // For now, just return continue to keep execution flowing
        Ok(DebugCommand::Continue)
    }
    
    /// Set step mode
    pub fn set_step_mode(&mut self, enabled: bool) {
        self.ui_state.step_mode = enabled;
    }
    
    /// Check if in step mode
    pub fn is_step_mode(&self) -> bool {
        self.ui_state.step_mode
    }
    
    /// Capture current execution state
    fn capture_execution_state(&mut self, execution_id: &str, instruction_address: usize, stack: &[ScriptValue]) -> RobinResult<()> {
        let state_snapshot = ExecutionStateSnapshot {
            execution_id: execution_id.to_string(),
            instruction_address,
            stack_values: stack.to_vec(),
            local_variables: HashMap::new(), // Would be populated from execution context
            global_variables: HashMap::new(), // Would be populated from VM
            call_stack_depth: 0, // Would be actual call stack depth
            timestamp: chrono::Utc::now(),
        };
        
        self.variable_inspector.add_snapshot(state_snapshot);
        Ok(())
    }
    
    /// Evaluate breakpoint condition
    fn evaluate_breakpoint_condition(&self, condition: &BreakpointCondition, stack: &[ScriptValue]) -> RobinResult<bool> {
        match condition {
            BreakpointCondition::StackNotEmpty => Ok(!stack.is_empty()),
            BreakpointCondition::StackSize(size) => Ok(stack.len() == *size),
            BreakpointCondition::StackContains(value) => Ok(stack.contains(value)),
            BreakpointCondition::VariableEquals { name: _name, value: _value } => {
                // Would check variable value in execution context
                Ok(true)
            }
            BreakpointCondition::HitCount(count) => {
                // Would track hit count per breakpoint
                Ok(*count > 0)
            }
            BreakpointCondition::Custom { predicate } => predicate(),
        }
    }
    
    /// Update debugger (call every frame)
    pub fn update(&mut self, delta_time: f32) -> RobinResult<()> {
        // Update performance profiler
        self.performance_profiler.update(delta_time)?;
        
        // Update execution tracer
        self.tracer.update(delta_time)?;
        
        // Update variable inspector
        self.variable_inspector.update(delta_time)?;
        
        // Clean up old debug sessions
        let cutoff_time = chrono::Utc::now() - chrono::Duration::hours(1);
        self.debug_sessions.retain(|_id, session| {
            session.started_at > cutoff_time
        });
        
        Ok(())
    }
    
    /// Get debug session info
    pub fn get_debug_session(&self, execution_id: &str) -> Option<&DebugSession> {
        self.debug_sessions.get(execution_id)
    }
    
    /// List all active debug sessions
    pub fn list_debug_sessions(&self) -> Vec<DebugSessionSummary> {
        self.debug_sessions.values().map(|session| {
            DebugSessionSummary {
                execution_id: session.execution_id.clone(),
                script_name: session.script_name.clone(),
                state: session.state,
                started_at: session.started_at,
                paused_at: session.paused_at,
                execution_steps: session.execution_trace.len(),
            }
        }).collect()
    }
    
    /// Get execution trace for visualization
    pub fn get_execution_trace(&self, execution_id: &str) -> Option<&Vec<ExecutionStep>> {
        self.debug_sessions.get(execution_id)
            .map(|session| &session.execution_trace)
    }
    
    /// Get variable inspection data
    pub fn get_variable_inspector(&self) -> &VariableInspector {
        &self.variable_inspector
    }
    
    /// Get performance profiling data
    pub fn get_performance_profile(&self) -> &PerformanceProfiler {
        &self.performance_profiler
    }
    
    /// Export debug session data
    pub fn export_debug_session(&self, execution_id: &str) -> RobinResult<DebugSessionExport> {
        if let Some(session) = self.debug_sessions.get(execution_id) {
            Ok(DebugSessionExport {
                session: session.clone(),
                breakpoints: self.breakpoints.values().cloned().collect(),
                trace_data: self.tracer.get_trace_data(execution_id).cloned().unwrap_or_default(),
                variable_snapshots: self.variable_inspector.get_snapshots_for_session(execution_id),
                performance_data: self.performance_profiler.get_session_data(execution_id),
            })
        } else {
            Err(format!("Debug session not found: {}", execution_id).into())
        }
    }
    
    /// Set debug settings
    pub fn configure_debugging(&mut self, settings: DebugSettings) {
        self.settings = settings;
        println!("Updated debug settings");
    }
    
    /// Create visualization data for UI
    pub fn create_visualization_data(&self, execution_id: &str) -> Option<DebugVisualization> {
        let session = self.debug_sessions.get(execution_id)?;
        
        Some(DebugVisualization {
            execution_flow: self.create_execution_flow_viz(session),
            variable_timeline: self.variable_inspector.create_variable_timeline(execution_id),
            performance_chart: self.performance_profiler.create_performance_chart(execution_id),
            call_stack_viz: self.create_call_stack_visualization(session),
            memory_usage: self.create_memory_usage_viz(execution_id),
        })
    }
    
    fn create_execution_flow_viz(&self, session: &DebugSession) -> ExecutionFlowVisualization {
        ExecutionFlowVisualization {
            nodes: session.execution_trace.iter().enumerate().map(|(i, step)| {
                FlowNode {
                    id: i,
                    instruction_address: step.instruction_address,
                    opcode: step.opcode,
                    execution_time: 0.0, // Would be calculated from timestamps
                    stack_size: step.stack_snapshot.len(),
                }
            }).collect(),
            connections: Vec::new(), // Would show execution flow connections
        }
    }
    
    fn create_call_stack_visualization(&self, session: &DebugSession) -> CallStackVisualization {
        CallStackVisualization {
            frames: session.call_stack.iter().enumerate().map(|(i, frame)| {
                CallStackFrame {
                    depth: i,
                    function_name: frame.function_name.clone(),
                    instruction_address: frame.return_address,
                    local_variables: frame.local_variables.clone(),
                }
            }).collect(),
        }
    }
    
    fn create_memory_usage_viz(&self, _execution_id: &str) -> MemoryUsageVisualization {
        MemoryUsageVisualization {
            timeline: Vec::new(), // Would be populated with actual memory usage data
            allocations: Vec::new(), // Would show memory allocations over time
            peak_usage: 0,
            current_usage: 0,
        }
    }
}

/// Individual debug session
#[derive(Debug, Clone)]
pub struct DebugSession {
    pub execution_id: String,
    pub script_name: String,
    pub state: DebugState,
    pub breakpoints: Vec<String>,
    pub step_mode: bool,
    pub call_stack: Vec<CallFrame>,
    pub local_variables: HashMap<String, ScriptValue>,
    pub execution_trace: Vec<ExecutionStep>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub paused_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Debug state for sessions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugState {
    Running,
    Paused,
    Stopped,
    Error,
}

/// Breakpoint definition
#[derive(Debug, Clone)]
pub struct Breakpoint {
    pub id: String,
    pub instruction_address: usize,
    pub condition: Option<BreakpointCondition>,
    pub enabled: bool,
    pub hit_count: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Conditions for breakpoint activation
#[derive(Debug, Clone)]
pub enum BreakpointCondition {
    /// Break when stack is not empty
    StackNotEmpty,
    
    /// Break when stack has specific size
    StackSize(usize),
    
    /// Break when stack contains specific value
    StackContains(ScriptValue),
    
    /// Break when variable equals specific value
    VariableEquals { name: String, value: ScriptValue },
    
    /// Break after specific number of hits
    HitCount(u32),
    
    /// Custom condition function
    Custom { predicate: Arc<dyn Fn() -> bool + Send + Sync> },
}

/// Debug commands from UI
#[derive(Debug, Clone)]
pub enum DebugCommand {
    Continue,
    StepOver,
    StepInto,
    StepOut,
    Stop,
    Restart,
    SetBreakpoint(usize),
    RemoveBreakpoint(usize),
    EvaluateExpression(String),
}

/// Execution step for tracing
#[derive(Debug, Clone)]
pub struct ExecutionStep {
    pub instruction_address: usize,
    pub opcode: u8,
    pub stack_snapshot: Vec<ScriptValue>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Call frame for stack tracing
#[derive(Debug, Clone)]
pub struct CallFrame {
    pub function_name: String,
    pub return_address: usize,
    pub local_variables: HashMap<String, ScriptValue>,
}

/// Execution tracer for recording script execution
#[derive(Debug)]
pub struct ExecutionTracer {
    traces: HashMap<String, ExecutionTrace>,
    max_trace_length: usize,
    enabled: bool,
}

impl ExecutionTracer {
    pub fn new() -> Self {
        Self {
            traces: HashMap::new(),
            max_trace_length: 10000,
            enabled: true,
        }
    }
    
    pub fn start_trace(&mut self, execution_id: String) {
        if self.enabled {
            let trace = ExecutionTrace {
                execution_id: execution_id.clone(),
                steps: Vec::new(),
                started_at: chrono::Utc::now(),
            };
            self.traces.insert(execution_id, trace);
        }
    }
    
    pub fn end_trace(&mut self, execution_id: &str) {
        self.traces.remove(execution_id);
    }
    
    pub fn record_step(&mut self, execution_id: String, step: ExecutionStep) {
        if let Some(trace) = self.traces.get_mut(&execution_id) {
            trace.steps.push(step);
            
            // Limit trace size
            if trace.steps.len() > self.max_trace_length {
                trace.steps.remove(0);
            }
        }
    }
    
    pub fn get_trace_data(&self, execution_id: &str) -> Option<&ExecutionTrace> {
        self.traces.get(execution_id)
    }
    
    pub fn update(&mut self, _delta_time: f32) -> RobinResult<()> {
        // Clean up old traces
        let cutoff_time = chrono::Utc::now() - chrono::Duration::minutes(30);
        self.traces.retain(|_id, trace| trace.started_at > cutoff_time);
        Ok(())
    }
    
    pub fn enable(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionTrace {
    pub execution_id: String,
    pub steps: Vec<ExecutionStep>,
    pub started_at: chrono::DateTime<chrono::Utc>,
}

/// Variable inspector for tracking variable changes
#[derive(Debug)]
pub struct VariableInspector {
    snapshots: Vec<ExecutionStateSnapshot>,
    variable_history: HashMap<String, VariableHistory>,
    watch_list: Vec<WatchExpression>,
}

impl VariableInspector {
    pub fn new() -> Self {
        Self {
            snapshots: Vec::new(),
            variable_history: HashMap::new(),
            watch_list: Vec::new(),
        }
    }
    
    pub fn add_snapshot(&mut self, snapshot: ExecutionStateSnapshot) {
        // Update variable history
        for (var_name, value) in &snapshot.local_variables {
            let history = self.variable_history.entry(var_name.clone()).or_insert_with(|| {
                VariableHistory::new(var_name.clone())
            });
            history.add_value_change(value.clone(), snapshot.timestamp);
        }
        
        self.snapshots.push(snapshot);
        
        // Limit snapshot history
        if self.snapshots.len() > 1000 {
            self.snapshots.remove(0);
        }
    }
    
    pub fn add_watch(&mut self, expression: String) -> String {
        let watch_id = uuid::Uuid::new_v4().to_string();
        let watch = WatchExpression {
            id: watch_id.clone(),
            expression,
            last_value: None,
            enabled: true,
        };
        self.watch_list.push(watch);
        watch_id
    }
    
    pub fn remove_watch(&mut self, watch_id: &str) -> bool {
        let initial_len = self.watch_list.len();
        self.watch_list.retain(|watch| watch.id != watch_id);
        self.watch_list.len() != initial_len
    }
    
    pub fn update(&mut self, _delta_time: f32) -> RobinResult<()> {
        // Update watch expressions
        for watch in &mut self.watch_list {
            if watch.enabled {
                // In real implementation, would evaluate expression
                // watch.last_value = Some(evaluate_expression(&watch.expression)?);
            }
        }
        Ok(())
    }
    
    pub fn get_snapshots_for_session(&self, execution_id: &str) -> Vec<ExecutionStateSnapshot> {
        self.snapshots.iter()
            .filter(|snapshot| snapshot.execution_id == execution_id)
            .cloned()
            .collect()
    }
    
    pub fn create_variable_timeline(&self, execution_id: &str) -> VariableTimeline {
        let session_snapshots = self.get_snapshots_for_session(execution_id);
        
        VariableTimeline {
            variables: self.variable_history.values().map(|history| {
                VariableTimelineEntry {
                    name: history.name.clone(),
                    changes: history.changes.iter().map(|change| {
                        VariableChange {
                            value: change.value.clone(),
                            timestamp: change.timestamp,
                        }
                    }).collect(),
                }
            }).collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionStateSnapshot {
    pub execution_id: String,
    pub instruction_address: usize,
    pub stack_values: Vec<ScriptValue>,
    pub local_variables: HashMap<String, ScriptValue>,
    pub global_variables: HashMap<String, ScriptValue>,
    pub call_stack_depth: usize,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug)]
pub struct VariableHistory {
    pub name: String,
    pub changes: Vec<VariableValueChange>,
}

impl VariableHistory {
    pub fn new(name: String) -> Self {
        Self {
            name,
            changes: Vec::new(),
        }
    }
    
    pub fn add_value_change(&mut self, value: ScriptValue, timestamp: chrono::DateTime<chrono::Utc>) {
        self.changes.push(VariableValueChange {
            value,
            timestamp,
        });
        
        // Limit history size
        if self.changes.len() > 100 {
            self.changes.remove(0);
        }
    }
}

#[derive(Debug)]
pub struct VariableValueChange {
    pub value: ScriptValue,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug)]
pub struct WatchExpression {
    pub id: String,
    pub expression: String,
    pub last_value: Option<ScriptValue>,
    pub enabled: bool,
}

/// Performance profiler for execution analysis
#[derive(Debug)]
pub struct PerformanceProfiler {
    session_profiles: HashMap<String, SessionProfile>,
    global_stats: GlobalPerformanceStats,
    enabled: bool,
}

impl PerformanceProfiler {
    pub fn new() -> Self {
        Self {
            session_profiles: HashMap::new(),
            global_stats: GlobalPerformanceStats::default(),
            enabled: true,
        }
    }
    
    pub fn start_profiling(&mut self, execution_id: String, script_name: String) {
        if self.enabled {
            let profile = SessionProfile {
                execution_id: execution_id.clone(),
                script_name,
                start_time: std::time::Instant::now(),
                instruction_counts: HashMap::new(),
                memory_usage: Vec::new(),
                function_calls: Vec::new(),
            };
            self.session_profiles.insert(execution_id, profile);
        }
    }
    
    pub fn record_instruction(&mut self, execution_id: &str, opcode: u8) {
        if let Some(profile) = self.session_profiles.get_mut(execution_id) {
            *profile.instruction_counts.entry(opcode).or_insert(0) += 1;
        }
    }
    
    pub fn record_function_call(&mut self, execution_id: &str, function_name: String, duration: std::time::Duration) {
        if let Some(profile) = self.session_profiles.get_mut(execution_id) {
            profile.function_calls.push(FunctionCallRecord {
                function_name,
                duration,
                timestamp: std::time::Instant::now(),
            });
        }
    }
    
    pub fn update(&mut self, _delta_time: f32) -> RobinResult<()> {
        // Update global statistics
        self.global_stats.total_sessions = self.session_profiles.len() as u64;
        Ok(())
    }
    
    pub fn get_session_data(&self, execution_id: &str) -> Option<&SessionProfile> {
        self.session_profiles.get(execution_id)
    }
    
    pub fn create_performance_chart(&self, execution_id: &str) -> PerformanceChart {
        if let Some(profile) = self.session_profiles.get(execution_id) {
            PerformanceChart {
                instruction_histogram: profile.instruction_counts.clone(),
                memory_timeline: profile.memory_usage.clone(),
                function_call_timeline: profile.function_calls.iter().map(|call| {
                    FunctionCallPoint {
                        name: call.function_name.clone(),
                        duration_ms: call.duration.as_secs_f64() * 1000.0,
                        timestamp_ms: call.timestamp.elapsed().as_secs_f64() * 1000.0,
                    }
                }).collect(),
            }
        } else {
            PerformanceChart {
                instruction_histogram: HashMap::new(),
                memory_timeline: Vec::new(),
                function_call_timeline: Vec::new(),
            }
        }
    }
}

#[derive(Debug)]
pub struct SessionProfile {
    pub execution_id: String,
    pub script_name: String,
    pub start_time: std::time::Instant,
    pub instruction_counts: HashMap<u8, u64>,
    pub memory_usage: Vec<MemoryUsagePoint>,
    pub function_calls: Vec<FunctionCallRecord>,
}

#[derive(Debug)]
pub struct FunctionCallRecord {
    pub function_name: String,
    pub duration: std::time::Duration,
    pub timestamp: std::time::Instant,
}

#[derive(Debug, Default)]
pub struct GlobalPerformanceStats {
    pub total_sessions: u64,
    pub average_execution_time: std::time::Duration,
    pub total_instructions_executed: u64,
    pub most_used_instructions: HashMap<u8, u64>,
}

// Debug UI and visualization types

#[derive(Debug, Default)]
pub struct DebugUIState {
    pub step_mode: bool,
    pub selected_session: Option<String>,
    pub show_variable_inspector: bool,
    pub show_performance_profiler: bool,
    pub show_execution_trace: bool,
    pub breakpoint_panel_open: bool,
}

#[derive(Debug)]
pub struct DebugSettings {
    pub global_debug_mode: bool,
    pub auto_break_on_error: bool,
    pub max_trace_length: usize,
    pub enable_performance_profiling: bool,
    pub enable_variable_tracking: bool,
    pub trace_all_instructions: bool,
}

impl Default for DebugSettings {
    fn default() -> Self {
        Self {
            global_debug_mode: false,
            auto_break_on_error: true,
            max_trace_length: 1000,
            enable_performance_profiling: true,
            enable_variable_tracking: true,
            trace_all_instructions: false,
        }
    }
}

#[derive(Debug)]
pub struct DebugSessionSummary {
    pub execution_id: String,
    pub script_name: String,
    pub state: DebugState,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub paused_at: Option<chrono::DateTime<chrono::Utc>>,
    pub execution_steps: usize,
}

#[derive(Debug)]
pub struct DebugSessionExport {
    pub session: DebugSession,
    pub breakpoints: Vec<Breakpoint>,
    pub trace_data: ExecutionTrace,
    pub variable_snapshots: Vec<ExecutionStateSnapshot>,
    pub performance_data: Option<SessionProfile>,
}

// Visualization data structures

#[derive(Debug)]
pub struct DebugVisualization {
    pub execution_flow: ExecutionFlowVisualization,
    pub variable_timeline: VariableTimeline,
    pub performance_chart: PerformanceChart,
    pub call_stack_viz: CallStackVisualization,
    pub memory_usage: MemoryUsageVisualization,
}

#[derive(Debug)]
pub struct ExecutionFlowVisualization {
    pub nodes: Vec<FlowNode>,
    pub connections: Vec<FlowConnection>,
}

#[derive(Debug)]
pub struct FlowNode {
    pub id: usize,
    pub instruction_address: usize,
    pub opcode: u8,
    pub execution_time: f64,
    pub stack_size: usize,
}

#[derive(Debug)]
pub struct FlowConnection {
    pub from: usize,
    pub to: usize,
    pub condition: Option<String>,
}

#[derive(Debug)]
pub struct VariableTimeline {
    pub variables: Vec<VariableTimelineEntry>,
}

#[derive(Debug)]
pub struct VariableTimelineEntry {
    pub name: String,
    pub changes: Vec<VariableChange>,
}

#[derive(Debug)]
pub struct VariableChange {
    pub value: ScriptValue,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug)]
pub struct PerformanceChart {
    pub instruction_histogram: HashMap<u8, u64>,
    pub memory_timeline: Vec<MemoryUsagePoint>,
    pub function_call_timeline: Vec<FunctionCallPoint>,
}

#[derive(Debug)]
pub struct MemoryUsagePoint {
    pub timestamp: std::time::Instant,
    pub bytes_allocated: usize,
}

#[derive(Debug)]
pub struct FunctionCallPoint {
    pub name: String,
    pub duration_ms: f64,
    pub timestamp_ms: f64,
}

#[derive(Debug)]
pub struct CallStackVisualization {
    pub frames: Vec<CallStackFrame>,
}

#[derive(Debug)]
pub struct CallStackFrame {
    pub depth: usize,
    pub function_name: String,
    pub instruction_address: usize,
    pub local_variables: HashMap<String, ScriptValue>,
}

#[derive(Debug)]
pub struct MemoryUsageVisualization {
    pub timeline: Vec<MemoryUsagePoint>,
    pub allocations: Vec<MemoryAllocation>,
    pub peak_usage: usize,
    pub current_usage: usize,
}

#[derive(Debug)]
pub struct MemoryAllocation {
    pub address: usize,
    pub size: usize,
    pub allocated_at: std::time::Instant,
    pub freed_at: Option<std::time::Instant>,
}