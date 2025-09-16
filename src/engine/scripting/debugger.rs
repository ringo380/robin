use crate::engine::error::RobinResult;
use crate::engine::scripting::{ScriptValue, ScriptResult, ScriptError, ScriptRuntime, ExecutionContext, Script};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, VecDeque, HashSet};
use std::time::{Instant, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoint {
    pub id: String,
    pub script_id: String,
    pub node_id: String,
    pub enabled: bool,
    pub condition: Option<String>,
    pub hit_count: u32,
    pub hit_count_condition: Option<HitCountCondition>,
    pub log_message: Option<String>,
    pub created_at: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitCountCondition {
    pub operation: HitCountOperation,
    pub value: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HitCountOperation {
    Equal,
    GreaterThan,
    Multiple,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DebugState {
    Running,
    Paused,
    Stepping,
    StepInto,
    StepOver,
    StepOut,
    Stopped,
}

#[derive(Debug, Clone)]
pub struct DebugFrame {
    pub script_id: String,
    pub node_id: String,
    pub local_variables: HashMap<String, ScriptValue>,
    pub execution_time: Duration,
    pub step_count: u32,
}

#[derive(Debug, Clone)]
pub struct VariableWatch {
    pub id: String,
    pub name: String,
    pub expression: String,
    pub current_value: Option<ScriptValue>,
    pub previous_value: Option<ScriptValue>,
    pub changed: bool,
    pub enabled: bool,
}

#[derive(Debug)]
pub struct ScriptDebugger {
    state: DebugState,
    breakpoints: HashMap<String, Breakpoint>,
    call_stack: Vec<DebugFrame>,
    watches: HashMap<String, VariableWatch>,
    execution_history: VecDeque<ExecutionEvent>,
    step_targets: HashSet<String>,
    current_script: Option<String>,
    current_node: Option<String>,
    debug_stats: DebugStats,
    max_history_size: usize,
}

#[derive(Debug, Clone)]
pub struct ExecutionEvent {
    pub timestamp: f64,
    pub event_type: ExecutionEventType,
    pub script_id: String,
    pub node_id: String,
    pub data: HashMap<String, ScriptValue>,
}

#[derive(Debug, Clone)]
pub enum ExecutionEventType {
    NodeEnter,
    NodeExit,
    VariableChanged,
    BreakpointHit,
    Exception,
    ScriptStart,
    ScriptEnd,
}

#[derive(Debug, Default)]
pub struct DebugStats {
    pub total_breakpoints: u32,
    pub breakpoints_hit: u32,
    pub steps_executed: u32,
    pub scripts_debugged: u32,
    pub total_debug_time_ms: u64,
    pub average_step_time_ms: f32,
}

impl Breakpoint {
    pub fn new(script_id: String, node_id: String) -> Self {
        Self {
            id: format!("bp_{}_{}", script_id, node_id),
            script_id,
            node_id,
            enabled: true,
            condition: None,
            hit_count: 0,
            hit_count_condition: None,
            log_message: None,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
        }
    }

    pub fn with_condition(mut self, condition: String) -> Self {
        self.condition = Some(condition);
        self
    }

    pub fn with_hit_count_condition(mut self, operation: HitCountOperation, value: u32) -> Self {
        self.hit_count_condition = Some(HitCountCondition { operation, value });
        self
    }

    pub fn with_log_message(mut self, message: String) -> Self {
        self.log_message = Some(message);
        self
    }

    pub fn should_break(&mut self, context: &ExecutionContext) -> ScriptResult<bool> {
        if !self.enabled {
            return Ok(false);
        }

        self.hit_count += 1;

        if let Some(ref hit_condition) = self.hit_count_condition {
            let meets_condition = match hit_condition.operation {
                HitCountOperation::Equal => self.hit_count == hit_condition.value,
                HitCountOperation::GreaterThan => self.hit_count > hit_condition.value,
                HitCountOperation::Multiple => self.hit_count % hit_condition.value == 0,
            };

            if !meets_condition {
                return Ok(false);
            }
        }

        if let Some(ref condition_expr) = self.condition {
            return self.evaluate_condition(condition_expr, context);
        }

        Ok(true)
    }

    fn evaluate_condition(&self, condition: &str, context: &ExecutionContext) -> ScriptResult<bool> {
        let parts: Vec<&str> = condition.split_whitespace().collect();
        if parts.len() != 3 {
            return Err(ScriptError::RuntimeError(
                "Invalid condition format. Expected 'variable operator value'".to_string()
            ));
        }

        let variable_name = parts[0];
        let operator = parts[1];
        let expected_value_str = parts[2];

        let current_value = context.get_variable(variable_name)
            .ok_or_else(|| ScriptError::RuntimeError(format!("Variable not found: {}", variable_name)))?;

        let expected_value = self.parse_value(expected_value_str)?;

        match operator {
            "==" | "=" => Ok(self.values_equal(current_value, &expected_value)),
            "!=" => Ok(!self.values_equal(current_value, &expected_value)),
            ">" => Ok(current_value.as_number() > expected_value.as_number()),
            ">=" => Ok(current_value.as_number() >= expected_value.as_number()),
            "<" => Ok(current_value.as_number() < expected_value.as_number()),
            "<=" => Ok(current_value.as_number() <= expected_value.as_number()),
            _ => Err(ScriptError::RuntimeError(format!("Unknown operator: {}", operator))),
        }
    }

    fn parse_value(&self, value_str: &str) -> ScriptResult<ScriptValue> {
        if value_str.starts_with('"') && value_str.ends_with('"') {
            Ok(ScriptValue::String(value_str[1..value_str.len()-1].to_string()))
        } else if value_str == "true" {
            Ok(ScriptValue::Boolean(true))
        } else if value_str == "false" {
            Ok(ScriptValue::Boolean(false))
        } else if let Ok(int_val) = value_str.parse::<i64>() {
            Ok(ScriptValue::Integer(int_val))
        } else if let Ok(float_val) = value_str.parse::<f64>() {
            Ok(ScriptValue::Float(float_val))
        } else {
            Err(ScriptError::RuntimeError(format!("Cannot parse value: {}", value_str)))
        }
    }

    fn values_equal(&self, a: &ScriptValue, b: &ScriptValue) -> bool {
        match (a, b) {
            (ScriptValue::Integer(x), ScriptValue::Float(y)) => *x as f64 == *y,
            (ScriptValue::Float(x), ScriptValue::Integer(y)) => *x == *y as f64,
            _ => a == b,
        }
    }
}

impl VariableWatch {
    pub fn new(name: String, expression: String) -> Self {
        Self {
            id: format!("watch_{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()),
            name,
            expression,
            current_value: None,
            previous_value: None,
            changed: false,
            enabled: true,
        }
    }

    pub fn update(&mut self, context: &ExecutionContext) -> ScriptResult<()> {
        if !self.enabled {
            return Ok(());
        }

        self.previous_value = self.current_value.clone();
        
        let new_value = if self.expression.contains(' ') {
            self.evaluate_expression(&self.expression, context)?
        } else {
            context.get_variable(&self.expression).cloned().unwrap_or(ScriptValue::None)
        };

        self.changed = self.previous_value.as_ref() != Some(&new_value);
        self.current_value = Some(new_value);

        Ok(())
    }

    fn evaluate_expression(&self, expression: &str, context: &ExecutionContext) -> ScriptResult<ScriptValue> {
        let trimmed = expression.trim();
        
        if let Some(var_value) = context.get_variable(trimmed) {
            return Ok(var_value.clone());
        }

        if trimmed.contains('.') {
            let parts: Vec<&str> = trimmed.split('.').collect();
            if parts.len() == 2 {
                if let Ok(entity_id) = parts[0].parse::<u64>() {
                    if let Some(state_value) = context.entity_states.get(&entity_id)
                        .and_then(|states| states.get(parts[1])) {
                        return Ok(state_value.clone());
                    }
                }
            }
        }

        Ok(ScriptValue::String(format!("Error: Cannot evaluate '{}'", expression)))
    }
}

impl ScriptDebugger {
    pub fn new() -> RobinResult<Self> {
        Ok(Self {
            state: DebugState::Stopped,
            breakpoints: HashMap::new(),
            call_stack: Vec::new(),
            watches: HashMap::new(),
            execution_history: VecDeque::new(),
            step_targets: HashSet::new(),
            current_script: None,
            current_node: None,
            debug_stats: DebugStats::default(),
            max_history_size: 1000,
        })
    }

    pub fn add_breakpoint(&mut self, breakpoint: Breakpoint) -> String {
        let id = breakpoint.id.clone();
        self.breakpoints.insert(id.clone(), breakpoint);
        self.debug_stats.total_breakpoints += 1;
        id
    }

    pub fn remove_breakpoint(&mut self, breakpoint_id: &str) -> bool {
        if self.breakpoints.remove(breakpoint_id).is_some() {
            self.debug_stats.total_breakpoints = self.debug_stats.total_breakpoints.saturating_sub(1);
            true
        } else {
            false
        }
    }

    pub fn enable_breakpoint(&mut self, breakpoint_id: &str, enabled: bool) -> bool {
        if let Some(breakpoint) = self.breakpoints.get_mut(breakpoint_id) {
            breakpoint.enabled = enabled;
            true
        } else {
            false
        }
    }

    pub fn add_watch(&mut self, name: String, expression: String) -> String {
        let watch = VariableWatch::new(name, expression);
        let id = watch.id.clone();
        self.watches.insert(id.clone(), watch);
        id
    }

    pub fn remove_watch(&mut self, watch_id: &str) -> bool {
        self.watches.remove(watch_id).is_some()
    }

    pub fn enable_watch(&mut self, watch_id: &str, enabled: bool) -> bool {
        if let Some(watch) = self.watches.get_mut(watch_id) {
            watch.enabled = enabled;
            true
        } else {
            false
        }
    }

    pub fn start_debugging(&mut self, script_id: String) {
        self.state = DebugState::Running;
        self.current_script = Some(script_id.clone());
        self.debug_stats.scripts_debugged += 1;
        
        self.add_execution_event(ExecutionEvent {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
            event_type: ExecutionEventType::ScriptStart,
            script_id,
            node_id: "start".to_string(),
            data: HashMap::new(),
        });
    }

    pub fn stop_debugging(&mut self) {
        if let Some(script_id) = &self.current_script {
            self.add_execution_event(ExecutionEvent {
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64(),
                event_type: ExecutionEventType::ScriptEnd,
                script_id: script_id.clone(),
                node_id: self.current_node.clone().unwrap_or_default(),
                data: HashMap::new(),
            });
        }

        self.state = DebugState::Stopped;
        self.current_script = None;
        self.current_node = None;
        self.call_stack.clear();
        self.step_targets.clear();
    }

    pub fn pause(&mut self) {
        if self.state == DebugState::Running {
            self.state = DebugState::Paused;
        }
    }

    pub fn resume(&mut self) {
        if self.state == DebugState::Paused {
            self.state = DebugState::Running;
        }
    }

    pub fn step_over(&mut self) {
        self.state = DebugState::StepOver;
    }

    pub fn step_into(&mut self) {
        self.state = DebugState::StepInto;
    }

    pub fn step_out(&mut self) {
        self.state = DebugState::StepOut;
        if !self.call_stack.is_empty() {
            self.call_stack.pop();
        }
    }

    pub fn execute_with_debugging(
        &mut self,
        runtime: &mut ScriptRuntime,
        script: &Script,
        context: &mut ExecutionContext,
    ) -> ScriptResult<ScriptValue> {
        let debug_start = Instant::now();
        
        self.start_debugging(script.id.clone());
        
        let result = self.debug_execute_script(runtime, script, context);
        
        self.debug_stats.total_debug_time_ms += debug_start.elapsed().as_millis() as u64;
        self.stop_debugging();
        
        result
    }

    fn debug_execute_script(
        &mut self,
        runtime: &mut ScriptRuntime,
        script: &Script,
        context: &mut ExecutionContext,
    ) -> ScriptResult<ScriptValue> {
        loop {
            match self.state {
                DebugState::Stopped => break,
                DebugState::Running => {
                    return runtime.execute(script, context);
                }
                DebugState::Paused => {
                    std::thread::sleep(Duration::from_millis(10));
                    continue;
                }
                DebugState::Stepping | DebugState::StepInto | DebugState::StepOver => {
                    let step_result = self.execute_single_step(runtime, script, context)?;
                    self.debug_stats.steps_executed += 1;
                    
                    if let Some(value) = step_result {
                        return Ok(value);
                    }
                    
                    self.state = DebugState::Paused;
                }
                DebugState::StepOut => {
                    if self.call_stack.len() <= 1 {
                        self.state = DebugState::Running;
                        return runtime.execute(script, context);
                    } else {
                        self.state = DebugState::Running;
                    }
                }
            }
        }

        Ok(ScriptValue::None)
    }

    fn execute_single_step(
        &mut self,
        runtime: &mut ScriptRuntime,
        script: &Script,
        context: &mut ExecutionContext,
    ) -> ScriptResult<Option<ScriptValue>> {
        self.update_watches(context)?;
        
        let old_step_count = context.step_count;
        context.max_steps = old_step_count + 1;
        
        match runtime.execute(script, context) {
            Ok(value) => {
                if context.step_count > old_step_count {
                    Ok(None)
                } else {
                    Ok(Some(value))
                }
            }
            Err(ScriptError::ExecutionTimeout) => {
                context.max_steps = 10000;
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }

    fn update_watches(&mut self, context: &ExecutionContext) -> ScriptResult<()> {
        for watch in self.watches.values_mut() {
            watch.update(context)?;
            
            if watch.changed {
                let mut event_data = HashMap::new();
                event_data.insert("watch_name".to_string(), ScriptValue::String(watch.name.clone()));
                if let Some(ref current) = watch.current_value {
                    event_data.insert("new_value".to_string(), current.clone());
                }
                if let Some(ref previous) = watch.previous_value {
                    event_data.insert("old_value".to_string(), previous.clone());
                }

                self.add_execution_event(ExecutionEvent {
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs_f64(),
                    event_type: ExecutionEventType::VariableChanged,
                    script_id: self.current_script.clone().unwrap_or_default(),
                    node_id: self.current_node.clone().unwrap_or_default(),
                    data: event_data,
                });
            }
        }
        Ok(())
    }

    fn add_execution_event(&mut self, event: ExecutionEvent) {
        if self.execution_history.len() >= self.max_history_size {
            self.execution_history.pop_front();
        }
        self.execution_history.push_back(event);
    }

    pub fn get_call_stack(&self) -> &[DebugFrame] {
        &self.call_stack
    }

    pub fn get_watches(&self) -> Vec<&VariableWatch> {
        self.watches.values().collect()
    }

    pub fn get_breakpoints(&self) -> Vec<&Breakpoint> {
        self.breakpoints.values().collect()
    }

    pub fn get_execution_history(&self, limit: Option<usize>) -> Vec<&ExecutionEvent> {
        let limit = limit.unwrap_or(self.execution_history.len());
        self.execution_history.iter().rev().take(limit).collect()
    }

    pub fn get_debug_state(&self) -> &DebugState {
        &self.state
    }

    pub fn get_current_location(&self) -> (Option<&String>, Option<&String>) {
        (self.current_script.as_ref(), self.current_node.as_ref())
    }

    pub fn get_debug_stats(&self) -> &DebugStats {
        &self.debug_stats
    }

    pub fn clear_history(&mut self) {
        self.execution_history.clear();
    }

    pub fn export_debug_session(&self) -> ScriptResult<String> {
        let session_data = DebugSessionData {
            breakpoints: self.breakpoints.values().cloned().collect(),
            watches: self.watches.values().cloned().collect(),
            execution_history: self.execution_history.iter().cloned().collect(),
            stats: self.debug_stats.clone(),
        };

        serde_json::to_string_pretty(&session_data)
            .map_err(|e| ScriptError::RuntimeError(format!("Failed to export debug session: {}", e)))
    }

    pub fn update(&mut self, _delta_time: f32) -> RobinResult<()> {
        if self.debug_stats.steps_executed > 0 {
            self.debug_stats.average_step_time_ms = 
                self.debug_stats.total_debug_time_ms as f32 / self.debug_stats.steps_executed as f32;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DebugSessionData {
    breakpoints: Vec<Breakpoint>,
    watches: Vec<VariableWatch>,
    execution_history: Vec<ExecutionEvent>,
    stats: DebugStats,
}

impl Clone for DebugStats {
    fn clone(&self) -> Self {
        Self {
            total_breakpoints: self.total_breakpoints,
            breakpoints_hit: self.breakpoints_hit,
            steps_executed: self.steps_executed,
            scripts_debugged: self.scripts_debugged,
            total_debug_time_ms: self.total_debug_time_ms,
            average_step_time_ms: self.average_step_time_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breakpoint_creation() {
        let breakpoint = Breakpoint::new("script1".to_string(), "node1".to_string());
        
        assert_eq!(breakpoint.script_id, "script1");
        assert_eq!(breakpoint.node_id, "node1");
        assert!(breakpoint.enabled);
        assert_eq!(breakpoint.hit_count, 0);
    }

    #[test]
    fn test_breakpoint_with_condition() {
        let mut context = ExecutionContext::default();
        context.set_variable("x".to_string(), ScriptValue::Integer(10));

        let mut breakpoint = Breakpoint::new("script1".to_string(), "node1".to_string())
            .with_condition("x > 5".to_string());

        let should_break = breakpoint.should_break(&context).unwrap();
        assert!(should_break);
        assert_eq!(breakpoint.hit_count, 1);
    }

    #[test]
    fn test_variable_watch() {
        let mut context = ExecutionContext::default();
        context.set_variable("health".to_string(), ScriptValue::Float(100.0));

        let mut watch = VariableWatch::new("Player Health".to_string(), "health".to_string());
        
        assert!(watch.update(&context).is_ok());
        assert_eq!(watch.current_value, Some(ScriptValue::Float(100.0)));
        assert!(!watch.changed); // First update, no previous value

        context.set_variable("health".to_string(), ScriptValue::Float(80.0));
        assert!(watch.update(&context).is_ok());
        assert_eq!(watch.current_value, Some(ScriptValue::Float(80.0)));
        assert!(watch.changed);
    }

    #[test]
    fn test_debugger_creation() {
        let debugger = ScriptDebugger::new();
        assert!(debugger.is_ok());
        
        let debugger = debugger.unwrap();
        assert_eq!(debugger.state, DebugState::Stopped);
        assert_eq!(debugger.breakpoints.len(), 0);
        assert_eq!(debugger.watches.len(), 0);
    }

    #[test]
    fn test_debugger_breakpoint_management() {
        let mut debugger = ScriptDebugger::new().unwrap();
        
        let breakpoint = Breakpoint::new("script1".to_string(), "node1".to_string());
        let bp_id = debugger.add_breakpoint(breakpoint);
        
        assert_eq!(debugger.breakpoints.len(), 1);
        assert_eq!(debugger.debug_stats.total_breakpoints, 1);
        
        assert!(debugger.enable_breakpoint(&bp_id, false));
        assert!(!debugger.breakpoints.get(&bp_id).unwrap().enabled);
        
        assert!(debugger.remove_breakpoint(&bp_id));
        assert_eq!(debugger.breakpoints.len(), 0);
    }

    #[test]
    fn test_debugger_watch_management() {
        let mut debugger = ScriptDebugger::new().unwrap();
        
        let watch_id = debugger.add_watch("Test Watch".to_string(), "test_var".to_string());
        assert_eq!(debugger.watches.len(), 1);
        
        assert!(debugger.enable_watch(&watch_id, false));
        assert!(!debugger.watches.get(&watch_id).unwrap().enabled);
        
        assert!(debugger.remove_watch(&watch_id));
        assert_eq!(debugger.watches.len(), 0);
    }

    #[test]
    fn test_debug_state_transitions() {
        let mut debugger = ScriptDebugger::new().unwrap();
        
        assert_eq!(debugger.state, DebugState::Stopped);
        
        debugger.start_debugging("script1".to_string());
        assert_eq!(debugger.state, DebugState::Running);
        
        debugger.pause();
        assert_eq!(debugger.state, DebugState::Paused);
        
        debugger.resume();
        assert_eq!(debugger.state, DebugState::Running);
        
        debugger.step_over();
        assert_eq!(debugger.state, DebugState::StepOver);
        
        debugger.stop_debugging();
        assert_eq!(debugger.state, DebugState::Stopped);
    }
}