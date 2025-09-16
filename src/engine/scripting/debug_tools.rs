// Real-time Debugging Tools for Robin Engine Script System
// Provides visual debugging, performance profiling, and execution tracing

use crate::engine::error::RobinResult;
use std::collections::{HashMap, VecDeque, BTreeMap};
use std::time::{Duration, SystemTime};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

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
    pub fn as_string(&self) -> String {
        match self {
            RuntimeValue::String(s) => s.clone(),
            RuntimeValue::Bool(b) => b.to_string(),
            RuntimeValue::Int(i) => i.to_string(),
            RuntimeValue::Float(f) => f.to_string(),
            RuntimeValue::Vector3(v) => format!("({}, {}, {})", v[0], v[1], v[2]),
            RuntimeValue::Array(a) => format!("[{}]", a.len()),
            RuntimeValue::Object(o) => format!("{{{}}}", o.len()),
            RuntimeValue::None => "None".to_string(),
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            RuntimeValue::Bool(_) => "Bool",
            RuntimeValue::Int(_) => "Int",
            RuntimeValue::Float(_) => "Float",
            RuntimeValue::String(_) => "String",
            RuntimeValue::Vector3(_) => "Vector3",
            RuntimeValue::Array(_) => "Array",
            RuntimeValue::Object(_) => "Object",
            RuntimeValue::None => "None",
        }
    }

    pub fn size_bytes(&self) -> usize {
        match self {
            RuntimeValue::Bool(_) => 1,
            RuntimeValue::Int(_) => 4,
            RuntimeValue::Float(_) => 4,
            RuntimeValue::String(s) => s.len(),
            RuntimeValue::Vector3(_) => 12,
            RuntimeValue::Array(a) => a.iter().map(|v| v.size_bytes()).sum::<usize>() + 8,
            RuntimeValue::Object(o) => o.iter().map(|(k, v)| k.len() + v.size_bytes()).sum::<usize>() + 8,
            RuntimeValue::None => 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakPoint {
    pub id: String,
    pub script_id: String,
    pub line: u32,
    pub column: Option<u32>,
    pub enabled: bool,
    pub condition: Option<String>,
    pub hit_count: u32,
    pub hit_count_condition: BreakPointCondition,
    pub created_at: SystemTime,
    pub last_hit: Option<SystemTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BreakPointCondition {
    Always,
    HitCountEquals(u32),
    HitCountGreater(u32),
    HitCountMultiple(u32),
}

impl BreakPoint {
    pub fn new(script_id: String, line: u32) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            script_id,
            line,
            column: None,
            enabled: true,
            condition: None,
            hit_count: 0,
            hit_count_condition: BreakPointCondition::Always,
            created_at: SystemTime::now(),
            last_hit: None,
        }
    }

    pub fn with_condition(mut self, condition: String) -> Self {
        self.condition = Some(condition);
        self
    }

    pub fn with_hit_count_condition(mut self, condition: BreakPointCondition) -> Self {
        self.hit_count_condition = condition;
        self
    }

    pub fn should_break(&self) -> bool {
        if !self.enabled {
            return false;
        }

        match &self.hit_count_condition {
            BreakPointCondition::Always => true,
            BreakPointCondition::HitCountEquals(count) => self.hit_count == *count,
            BreakPointCondition::HitCountGreater(count) => self.hit_count > *count,
            BreakPointCondition::HitCountMultiple(multiple) => {
                *multiple > 0 && self.hit_count % multiple == 0
            }
        }
    }

    pub fn hit(&mut self) {
        self.hit_count += 1;
        self.last_hit = Some(SystemTime::now());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrace {
    pub id: String,
    pub script_id: String,
    pub timestamp: SystemTime,
    pub line: u32,
    pub function_name: Option<String>,
    pub instruction: String,
    pub variables: HashMap<String, RuntimeValue>,
    pub stack_depth: u32,
    pub execution_time_ms: f32,
}

impl ExecutionTrace {
    pub fn new(script_id: String, line: u32, instruction: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            script_id,
            timestamp: SystemTime::now(),
            line,
            function_name: None,
            instruction,
            variables: HashMap::new(),
            stack_depth: 0,
            execution_time_ms: 0.0,
        }
    }

    pub fn with_function(mut self, function_name: String) -> Self {
        self.function_name = Some(function_name);
        self
    }

    pub fn with_variables(mut self, variables: HashMap<String, RuntimeValue>) -> Self {
        self.variables = variables;
        self
    }

    pub fn with_stack_depth(mut self, depth: u32) -> Self {
        self.stack_depth = depth;
        self
    }

    pub fn with_execution_time(mut self, time_ms: f32) -> Self {
        self.execution_time_ms = time_ms;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableWatcher {
    pub id: String,
    pub name: String,
    pub variable_path: String, // e.g., "player.health" or "enemies[0].position.x"
    pub script_id: Option<String>, // None for global variables
    pub condition: WatchCondition,
    pub enabled: bool,
    pub value_history: VecDeque<(SystemTime, RuntimeValue)>,
    pub max_history: usize,
    pub last_triggered: Option<SystemTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WatchCondition {
    OnChange,
    OnValue(RuntimeValue),
    OnGreater(f32),
    OnLess(f32),
    OnContains(String),
    Custom(String), // Custom expression
}

impl VariableWatcher {
    pub fn new(name: String, variable_path: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            variable_path,
            script_id: None,
            condition: WatchCondition::OnChange,
            enabled: true,
            value_history: VecDeque::new(),
            max_history: 100,
            last_triggered: None,
        }
    }

    pub fn with_script(mut self, script_id: String) -> Self {
        self.script_id = Some(script_id);
        self
    }

    pub fn with_condition(mut self, condition: WatchCondition) -> Self {
        self.condition = condition;
        self
    }

    pub fn with_max_history(mut self, max: usize) -> Self {
        self.max_history = max;
        self
    }

    pub fn add_value(&mut self, value: RuntimeValue) -> bool {
        let now = SystemTime::now();
        let should_trigger = self.should_trigger(&value);

        // Add to history
        self.value_history.push_back((now, value));
        if self.value_history.len() > self.max_history {
            self.value_history.pop_front();
        }

        if should_trigger {
            self.last_triggered = Some(now);
        }

        should_trigger
    }

    fn should_trigger(&self, new_value: &RuntimeValue) -> bool {
        if !self.enabled {
            return false;
        }

        match &self.condition {
            WatchCondition::OnChange => {
                if let Some((_, last_value)) = self.value_history.back() {
                    !self.values_equal(last_value, new_value)
                } else {
                    true // First value is always a change
                }
            }
            WatchCondition::OnValue(target) => self.values_equal(new_value, target),
            WatchCondition::OnGreater(threshold) => {
                self.get_numeric_value(new_value) > *threshold
            }
            WatchCondition::OnLess(threshold) => {
                self.get_numeric_value(new_value) < *threshold
            }
            WatchCondition::OnContains(substring) => {
                new_value.as_string().contains(substring)
            }
            WatchCondition::Custom(_) => {
                // In a full implementation, this would evaluate the custom expression
                false
            }
        }
    }

    fn values_equal(&self, a: &RuntimeValue, b: &RuntimeValue) -> bool {
        match (a, b) {
            (RuntimeValue::Bool(a), RuntimeValue::Bool(b)) => a == b,
            (RuntimeValue::Int(a), RuntimeValue::Int(b)) => a == b,
            (RuntimeValue::Float(a), RuntimeValue::Float(b)) => (a - b).abs() < f32::EPSILON,
            (RuntimeValue::String(a), RuntimeValue::String(b)) => a == b,
            (RuntimeValue::Vector3(a), RuntimeValue::Vector3(b)) => {
                (a[0] - b[0]).abs() < f32::EPSILON &&
                (a[1] - b[1]).abs() < f32::EPSILON &&
                (a[2] - b[2]).abs() < f32::EPSILON
            }
            (RuntimeValue::None, RuntimeValue::None) => true,
            _ => false,
        }
    }

    fn get_numeric_value(&self, value: &RuntimeValue) -> f32 {
        match value {
            RuntimeValue::Int(i) => *i as f32,
            RuntimeValue::Float(f) => *f,
            RuntimeValue::Bool(b) => if *b { 1.0 } else { 0.0 },
            _ => 0.0,
        }
    }

    pub fn get_current_value(&self) -> Option<&RuntimeValue> {
        self.value_history.back().map(|(_, value)| value)
    }

    pub fn get_value_at(&self, index: usize) -> Option<&RuntimeValue> {
        self.value_history.get(index).map(|(_, value)| value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceProfiler {
    pub script_profiles: HashMap<String, ScriptProfile>,
    pub function_profiles: HashMap<String, FunctionProfile>,
    pub total_execution_time: Duration,
    pub total_instructions: u64,
    pub memory_usage_samples: VecDeque<(SystemTime, usize)>,
    pub enabled: bool,
    pub sample_rate_hz: f32,
    pub max_samples: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptProfile {
    pub script_id: String,
    pub total_execution_time: Duration,
    pub instruction_count: u64,
    pub call_count: u64,
    pub average_execution_time: Duration,
    pub peak_memory_usage: usize,
    pub last_execution: Option<SystemTime>,
    pub hotspots: Vec<LineProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionProfile {
    pub function_name: String,
    pub script_id: String,
    pub call_count: u64,
    pub total_time: Duration,
    pub average_time: Duration,
    pub min_time: Duration,
    pub max_time: Duration,
    pub memory_allocations: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineProfile {
    pub line_number: u32,
    pub execution_count: u64,
    pub total_time: Duration,
    pub average_time: Duration,
}

impl PerformanceProfiler {
    pub fn new(sample_rate_hz: f32, max_samples: usize) -> Self {
        Self {
            script_profiles: HashMap::new(),
            function_profiles: HashMap::new(),
            total_execution_time: Duration::new(0, 0),
            total_instructions: 0,
            memory_usage_samples: VecDeque::new(),
            enabled: true,
            sample_rate_hz,
            max_samples,
        }
    }

    pub fn start_script_execution(&mut self, script_id: &str) {
        if !self.enabled {
            return;
        }

        let profile = self.script_profiles.entry(script_id.to_string())
            .or_insert_with(|| ScriptProfile {
                script_id: script_id.to_string(),
                total_execution_time: Duration::new(0, 0),
                instruction_count: 0,
                call_count: 0,
                average_execution_time: Duration::new(0, 0),
                peak_memory_usage: 0,
                last_execution: None,
                hotspots: Vec::new(),
            });

        profile.call_count += 1;
        profile.last_execution = Some(SystemTime::now());
    }

    pub fn end_script_execution(&mut self, script_id: &str, execution_time: Duration, instructions: u64, memory_used: usize) {
        if !self.enabled {
            return;
        }

        if let Some(profile) = self.script_profiles.get_mut(script_id) {
            profile.total_execution_time += execution_time;
            profile.instruction_count += instructions;
            profile.average_execution_time = profile.total_execution_time / profile.call_count as u32;
            
            if memory_used > profile.peak_memory_usage {
                profile.peak_memory_usage = memory_used;
            }
        }

        self.total_execution_time += execution_time;
        self.total_instructions += instructions;
    }

    pub fn record_function_call(&mut self, function_name: &str, script_id: &str, execution_time: Duration, memory_allocations: u64) {
        if !self.enabled {
            return;
        }

        let key = format!("{}::{}", script_id, function_name);
        let profile = self.function_profiles.entry(key)
            .or_insert_with(|| FunctionProfile {
                function_name: function_name.to_string(),
                script_id: script_id.to_string(),
                call_count: 0,
                total_time: Duration::new(0, 0),
                average_time: Duration::new(0, 0),
                min_time: Duration::from_secs(u64::MAX),
                max_time: Duration::new(0, 0),
                memory_allocations: 0,
            });

        profile.call_count += 1;
        profile.total_time += execution_time;
        profile.average_time = profile.total_time / profile.call_count as u32;
        profile.memory_allocations += memory_allocations;

        if execution_time < profile.min_time {
            profile.min_time = execution_time;
        }
        if execution_time > profile.max_time {
            profile.max_time = execution_time;
        }
    }

    pub fn record_line_execution(&mut self, script_id: &str, line: u32, execution_time: Duration) {
        if !self.enabled {
            return;
        }

        if let Some(script_profile) = self.script_profiles.get_mut(script_id) {
            if let Some(line_profile) = script_profile.hotspots.iter_mut().find(|lp| lp.line_number == line) {
                line_profile.execution_count += 1;
                line_profile.total_time += execution_time;
                line_profile.average_time = line_profile.total_time / line_profile.execution_count as u32;
            } else {
                script_profile.hotspots.push(LineProfile {
                    line_number: line,
                    execution_count: 1,
                    total_time: execution_time,
                    average_time: execution_time,
                });
            }
        }
    }

    pub fn sample_memory_usage(&mut self, memory_bytes: usize) {
        if !self.enabled {
            return;
        }

        let now = SystemTime::now();
        self.memory_usage_samples.push_back((now, memory_bytes));
        
        if self.memory_usage_samples.len() > self.max_samples {
            self.memory_usage_samples.pop_front();
        }
    }

    pub fn get_top_scripts(&self, limit: usize) -> Vec<ScriptProfile> {
        let mut scripts: Vec<_> = self.script_profiles.values().cloned().collect();
        scripts.sort_by(|a, b| b.total_execution_time.cmp(&a.total_execution_time));
        scripts.into_iter().take(limit).collect()
    }

    pub fn get_top_functions(&self, limit: usize) -> Vec<FunctionProfile> {
        let mut functions: Vec<_> = self.function_profiles.values().cloned().collect();
        functions.sort_by(|a, b| b.total_time.cmp(&a.total_time));
        functions.into_iter().take(limit).collect()
    }

    pub fn get_hotspots(&self, script_id: &str, limit: usize) -> Vec<&LineProfile> {
        if let Some(profile) = self.script_profiles.get(script_id) {
            let mut hotspots: Vec<_> = profile.hotspots.iter().collect();
            hotspots.sort_by(|a, b| b.total_time.cmp(&a.total_time));
            hotspots.into_iter().take(limit).collect()
        } else {
            Vec::new()
        }
    }

    pub fn clear(&mut self) {
        self.script_profiles.clear();
        self.function_profiles.clear();
        self.memory_usage_samples.clear();
        self.total_execution_time = Duration::new(0, 0);
        self.total_instructions = 0;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugVisualization {
    pub visualization_type: VisualizationType,
    pub data: VisualizationData,
    pub position: (f32, f32, f32),
    pub duration: Option<Duration>,
    pub created_at: SystemTime,
    pub visible: bool,
    pub color: (f32, f32, f32, f32),
    pub scale: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VisualizationType {
    Point,
    Line,
    Sphere,
    Box,
    Arrow,
    Text,
    Graph,
    Heatmap,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VisualizationData {
    Point,
    Line { end_position: (f32, f32, f32) },
    Sphere { radius: f32 },
    Box { size: (f32, f32, f32) },
    Arrow { direction: (f32, f32, f32), length: f32 },
    Text { content: String, size: f32 },
    Graph { values: Vec<f32>, range: (f32, f32) },
    Heatmap { grid: Vec<Vec<f32>>, size: (u32, u32) },
}

impl DebugVisualization {
    pub fn point(position: (f32, f32, f32)) -> Self {
        Self {
            visualization_type: VisualizationType::Point,
            data: VisualizationData::Point,
            position,
            duration: Some(Duration::from_secs(5)),
            created_at: SystemTime::now(),
            visible: true,
            color: (1.0, 0.0, 0.0, 1.0),
            scale: 1.0,
        }
    }

    pub fn line(start: (f32, f32, f32), end: (f32, f32, f32)) -> Self {
        Self {
            visualization_type: VisualizationType::Line,
            data: VisualizationData::Line { end_position: end },
            position: start,
            duration: Some(Duration::from_secs(5)),
            created_at: SystemTime::now(),
            visible: true,
            color: (0.0, 1.0, 0.0, 1.0),
            scale: 1.0,
        }
    }

    pub fn sphere(position: (f32, f32, f32), radius: f32) -> Self {
        Self {
            visualization_type: VisualizationType::Sphere,
            data: VisualizationData::Sphere { radius },
            position,
            duration: Some(Duration::from_secs(10)),
            created_at: SystemTime::now(),
            visible: true,
            color: (0.0, 0.0, 1.0, 0.5),
            scale: 1.0,
        }
    }

    pub fn text(position: (f32, f32, f32), content: String) -> Self {
        Self {
            visualization_type: VisualizationType::Text,
            data: VisualizationData::Text { content, size: 12.0 },
            position,
            duration: Some(Duration::from_secs(3)),
            created_at: SystemTime::now(),
            visible: true,
            color: (1.0, 1.0, 1.0, 1.0),
            scale: 1.0,
        }
    }

    pub fn with_color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.color = (r, g, b, a);
        self
    }

    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    pub fn permanent(mut self) -> Self {
        self.duration = None;
        self
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    pub fn is_expired(&self) -> bool {
        if let Some(duration) = self.duration {
            self.created_at.elapsed().unwrap_or_default() >= duration
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebugSessionState {
    Stopped,
    Running,
    Paused,
    SteppingOver,
    SteppingInto,
    SteppingOut,
}

#[derive(Debug)]
pub struct DebugSession {
    pub id: String,
    pub script_id: String,
    pub state: DebugSessionState,
    pub current_line: Option<u32>,
    pub call_stack: Vec<StackFrame>,
    pub variables: HashMap<String, RuntimeValue>,
    pub breakpoints: HashMap<String, BreakPoint>,
    pub execution_trace: VecDeque<ExecutionTrace>,
    pub max_trace_entries: usize,
    pub created_at: SystemTime,
    pub last_step: Option<SystemTime>,
    pub step_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    pub function_name: String,
    pub script_id: String,
    pub line: u32,
    pub local_variables: HashMap<String, RuntimeValue>,
}

impl DebugSession {
    pub fn new(script_id: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            script_id,
            state: DebugSessionState::Stopped,
            current_line: None,
            call_stack: Vec::new(),
            variables: HashMap::new(),
            breakpoints: HashMap::new(),
            execution_trace: VecDeque::new(),
            max_trace_entries: 1000,
            created_at: SystemTime::now(),
            last_step: None,
            step_count: 0,
        }
    }

    pub fn start(&mut self) {
        self.state = DebugSessionState::Running;
    }

    pub fn pause(&mut self) {
        if matches!(self.state, DebugSessionState::Running) {
            self.state = DebugSessionState::Paused;
        }
    }

    pub fn resume(&mut self) {
        if matches!(self.state, DebugSessionState::Paused) {
            self.state = DebugSessionState::Running;
        }
    }

    pub fn stop(&mut self) {
        self.state = DebugSessionState::Stopped;
        self.current_line = None;
        self.call_stack.clear();
        self.variables.clear();
        self.execution_trace.clear();
    }

    pub fn step_over(&mut self) {
        self.state = DebugSessionState::SteppingOver;
        self.step_count += 1;
        self.last_step = Some(SystemTime::now());
    }

    pub fn step_into(&mut self) {
        self.state = DebugSessionState::SteppingInto;
        self.step_count += 1;
        self.last_step = Some(SystemTime::now());
    }

    pub fn step_out(&mut self) {
        self.state = DebugSessionState::SteppingOut;
        self.step_count += 1;
        self.last_step = Some(SystemTime::now());
    }

    pub fn add_breakpoint(&mut self, breakpoint: BreakPoint) {
        self.breakpoints.insert(breakpoint.id.clone(), breakpoint);
    }

    pub fn remove_breakpoint(&mut self, breakpoint_id: &str) -> Option<BreakPoint> {
        self.breakpoints.remove(breakpoint_id)
    }

    pub fn set_current_line(&mut self, line: u32) {
        self.current_line = Some(line);
    }

    pub fn push_stack_frame(&mut self, frame: StackFrame) {
        self.call_stack.push(frame);
    }

    pub fn pop_stack_frame(&mut self) -> Option<StackFrame> {
        self.call_stack.pop()
    }

    pub fn add_trace_entry(&mut self, trace: ExecutionTrace) {
        self.execution_trace.push_back(trace);
        if self.execution_trace.len() > self.max_trace_entries {
            self.execution_trace.pop_front();
        }
    }

    pub fn update_variable(&mut self, name: String, value: RuntimeValue) {
        self.variables.insert(name, value);
    }

    pub fn get_variable(&self, name: &str) -> Option<&RuntimeValue> {
        self.variables.get(name)
    }

    pub fn should_break_at_line(&mut self, line: u32) -> bool {
        // Check if any breakpoint matches this line
        for breakpoint in self.breakpoints.values_mut() {
            if breakpoint.line == line && breakpoint.enabled {
                breakpoint.hit();
                if breakpoint.should_break() {
                    return true;
                }
            }
        }
        false
    }

    pub fn is_running(&self) -> bool {
        !matches!(self.state, DebugSessionState::Stopped)
    }

    pub fn is_paused(&self) -> bool {
        matches!(self.state, DebugSessionState::Paused)
    }

    pub fn is_stepping(&self) -> bool {
        matches!(self.state, 
            DebugSessionState::SteppingOver | 
            DebugSessionState::SteppingInto | 
            DebugSessionState::SteppingOut
        )
    }
}

#[derive(Debug)]
pub struct DebugInfo {
    pub script_performances: HashMap<String, ScriptPerformanceInfo>,
    pub total_memory_usage: usize,
    pub active_sessions: u32,
    pub total_breakpoints: u32,
    pub total_watchers: u32,
    pub debug_visualizations: u32,
    pub uptime: Duration,
    pub last_update: SystemTime,
}

#[derive(Debug, Clone)]
pub struct ScriptPerformanceInfo {
    pub script_id: String,
    pub execution_time_ms: f32,
    pub memory_usage_bytes: usize,
    pub instruction_count: u64,
    pub fps_impact: f32,
    pub status: String,
}

impl DebugInfo {
    pub fn new() -> Self {
        Self {
            script_performances: HashMap::new(),
            total_memory_usage: 0,
            active_sessions: 0,
            total_breakpoints: 0,
            total_watchers: 0,
            debug_visualizations: 0,
            uptime: Duration::new(0, 0),
            last_update: SystemTime::now(),
        }
    }

    pub fn update(&mut self, 
                  profiler: &PerformanceProfiler,
                  active_sessions: u32,
                  total_breakpoints: u32,
                  total_watchers: u32,
                  visualizations: u32) {
        
        let now = SystemTime::now();
        if let Ok(duration) = now.duration_since(self.last_update) {
            self.uptime += duration;
        }
        self.last_update = now;

        // Update script performance info
        self.script_performances.clear();
        for (script_id, profile) in &profiler.script_profiles {
            self.script_performances.insert(script_id.clone(), ScriptPerformanceInfo {
                script_id: script_id.clone(),
                execution_time_ms: profile.average_execution_time.as_secs_f32() * 1000.0,
                memory_usage_bytes: profile.peak_memory_usage,
                instruction_count: profile.instruction_count,
                fps_impact: self.calculate_fps_impact(profile.average_execution_time),
                status: if profile.last_execution.map_or(true, |t| t.elapsed().unwrap_or_default().as_secs() > 5) {
                    "Idle".to_string()
                } else {
                    "Active".to_string()
                },
            });
        }

        self.total_memory_usage = profiler.memory_usage_samples
            .back()
            .map(|(_, usage)| *usage)
            .unwrap_or(0);

        self.active_sessions = active_sessions;
        self.total_breakpoints = total_breakpoints;
        self.total_watchers = total_watchers;
        self.debug_visualizations = visualizations;
    }

    fn calculate_fps_impact(&self, execution_time: Duration) -> f32 {
        let frame_budget_ms = 16.67; // 60 FPS
        let execution_time_ms = execution_time.as_secs_f32() * 1000.0;
        (execution_time_ms / frame_budget_ms * 100.0).min(100.0)
    }
}

#[derive(Debug)]
pub struct ScriptDebugger {
    sessions: HashMap<String, DebugSession>,
    global_breakpoints: HashMap<String, BreakPoint>,
    watchers: HashMap<String, VariableWatcher>,
    profiler: PerformanceProfiler,
    visualizations: HashMap<String, DebugVisualization>,
    debug_info: DebugInfo,
    enabled: bool,
    max_sessions: u32,
    auto_cleanup_interval: Duration,
    last_cleanup: SystemTime,
}

impl ScriptDebugger {
    pub fn new(enabled: bool) -> RobinResult<Self> {
        Ok(Self {
            sessions: HashMap::new(),
            global_breakpoints: HashMap::new(),
            watchers: HashMap::new(),
            profiler: PerformanceProfiler::new(30.0, 1000),
            visualizations: HashMap::new(),
            debug_info: DebugInfo::new(),
            enabled,
            max_sessions: 10,
            auto_cleanup_interval: Duration::from_secs(60),
            last_cleanup: SystemTime::now(),
        })
    }

    pub fn initialize(&mut self) -> RobinResult<()> {
        println!("Script Debugger initialized:");
        println!("  Visual debugging: {}", self.enabled);
        println!("  Performance profiling: {}", self.profiler.enabled);
        println!("  Max debug sessions: {}", self.max_sessions);
        println!("  Auto cleanup interval: {}s", self.auto_cleanup_interval.as_secs());
        Ok(())
    }

    pub fn start_debug_session(&mut self, script_id: &str) -> RobinResult<String> {
        if !self.enabled {
            return Err(crate::engine::error::RobinError::InvalidOperation {
                operation: "start_debug_session".to_string(),
                context: "debugger".to_string(),
                reason: "Debugger is disabled".to_string()
            });
        }

        if self.sessions.len() >= self.max_sessions as usize {
            return Err(crate::engine::error::RobinError::ResourceLimit(
                format!("Maximum debug sessions ({}) reached", self.max_sessions)
            ));
        }

        let session = DebugSession::new(script_id.to_string());
        let session_id = session.id.clone();
        
        self.sessions.insert(session_id.clone(), session);
        
        println!("Started debug session: {} for script: {}", session_id, script_id);
        Ok(session_id)
    }

    pub fn stop_debug_session(&mut self, session_id: &str) -> RobinResult<()> {
        if let Some(mut session) = self.sessions.remove(session_id) {
            session.stop();
            println!("Stopped debug session: {}", session_id);
        }
        Ok(())
    }

    pub fn pause_session(&mut self, session_id: &str) -> RobinResult<()> {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.pause();
        }
        Ok(())
    }

    pub fn resume_session(&mut self, session_id: &str) -> RobinResult<()> {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.resume();
        }
        Ok(())
    }

    pub fn step_execution(&mut self, session_id: &str) -> RobinResult<()> {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.step_over();
        }
        Ok(())
    }

    pub fn step_into(&mut self, session_id: &str) -> RobinResult<()> {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.step_into();
        }
        Ok(())
    }

    pub fn step_out(&mut self, session_id: &str) -> RobinResult<()> {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.step_out();
        }
        Ok(())
    }

    pub fn add_breakpoint(&mut self, script_id: &str, line: u32) -> RobinResult<()> {
        let breakpoint = BreakPoint::new(script_id.to_string(), line);
        let breakpoint_id = breakpoint.id.clone();
        
        // Add to global breakpoints
        self.global_breakpoints.insert(breakpoint_id.clone(), breakpoint.clone());
        
        // Add to relevant sessions
        for session in self.sessions.values_mut() {
            if session.script_id == script_id {
                session.add_breakpoint(breakpoint.clone());
            }
        }
        
        println!("Added breakpoint: {} at line {} in script {}", breakpoint_id, line, script_id);
        Ok(())
    }

    pub fn remove_breakpoint(&mut self, breakpoint_id: &str) -> RobinResult<()> {
        if let Some(_) = self.global_breakpoints.remove(breakpoint_id) {
            // Remove from all sessions
            for session in self.sessions.values_mut() {
                session.remove_breakpoint(breakpoint_id);
            }
            println!("Removed breakpoint: {}", breakpoint_id);
        }
        Ok(())
    }

    pub fn add_variable_watcher(&mut self, name: String, variable_path: String, script_id: Option<String>) -> RobinResult<String> {
        let mut watcher = VariableWatcher::new(name, variable_path);
        if let Some(script_id) = script_id {
            watcher = watcher.with_script(script_id);
        }
        
        let watcher_id = watcher.id.clone();
        self.watchers.insert(watcher_id.clone(), watcher);
        
        println!("Added variable watcher: {}", watcher_id);
        Ok(watcher_id)
    }

    pub fn remove_variable_watcher(&mut self, watcher_id: &str) -> RobinResult<()> {
        if let Some(_) = self.watchers.remove(watcher_id) {
            println!("Removed variable watcher: {}", watcher_id);
        }
        Ok(())
    }

    pub fn get_variable_value(&self, session_id: &str, variable_name: &str) -> RobinResult<RuntimeValue> {
        if let Some(session) = self.sessions.get(session_id) {
            if let Some(value) = session.get_variable(variable_name) {
                Ok(value.clone())
            } else {
                Err(crate::engine::error::RobinError::InvalidInput(
                    format!("Variable '{}' not found in session", variable_name)
                ))
            }
        } else {
            Err(crate::engine::error::RobinError::InvalidInput(
                format!("Debug session '{}' not found", session_id)
            ))
        }
    }

    pub fn add_debug_visualization(&mut self, visualization: DebugVisualization) -> String {
        let viz_id = Uuid::new_v4().to_string();
        self.visualizations.insert(viz_id.clone(), visualization);
        viz_id
    }

    pub fn remove_debug_visualization(&mut self, viz_id: &str) -> RobinResult<()> {
        self.visualizations.remove(viz_id);
        Ok(())
    }

    pub fn clear_debug_visualizations(&mut self) {
        self.visualizations.clear();
    }

    pub fn record_script_execution(&mut self, script_id: &str, execution_time: Duration, instructions: u64, memory_used: usize) {
        self.profiler.end_script_execution(script_id, execution_time, instructions, memory_used);
    }

    pub fn record_function_call(&mut self, function_name: &str, script_id: &str, execution_time: Duration) {
        self.profiler.record_function_call(function_name, script_id, execution_time, 0);
    }

    pub fn record_line_execution(&mut self, script_id: &str, line: u32, execution_time: Duration) {
        self.profiler.record_line_execution(script_id, line, execution_time);
    }

    pub fn update(&mut self, _delta_time: f32) -> RobinResult<()> {
        if !self.enabled {
            return Ok(());
        }

        // Update profiler memory samples
        let total_memory = self.calculate_total_memory_usage();
        self.profiler.sample_memory_usage(total_memory);

        // Clean up expired visualizations
        self.visualizations.retain(|_, viz| !viz.is_expired());

        // Update watchers
        for watcher in self.watchers.values_mut() {
            // In a real implementation, this would check the actual variable values
            // For now, we'll simulate some activity
            if watcher.enabled && rand::random::<f32>() < 0.01 {
                let dummy_value = RuntimeValue::Float(rand::random::<f32>() * 100.0);
                watcher.add_value(dummy_value);
            }
        }

        // Update debug info
        self.debug_info.update(
            &self.profiler,
            self.sessions.len() as u32,
            self.global_breakpoints.len() as u32,
            self.watchers.len() as u32,
            self.visualizations.len() as u32,
        );

        // Periodic cleanup
        if self.last_cleanup.elapsed().unwrap_or_default() >= self.auto_cleanup_interval {
            self.cleanup_inactive_sessions();
            self.last_cleanup = SystemTime::now();
        }

        Ok(())
    }

    fn calculate_total_memory_usage(&self) -> usize {
        let mut total = 0;
        
        // Session memory
        for session in self.sessions.values() {
            for value in session.variables.values() {
                total += value.size_bytes();
            }
            total += session.execution_trace.len() * 256; // Approximate trace entry size
        }
        
        // Watcher memory
        for watcher in self.watchers.values() {
            for (_, value) in &watcher.value_history {
                total += value.size_bytes();
            }
        }
        
        // Profiler memory
        total += self.profiler.memory_usage_samples.len() * 16; // (SystemTime, usize)
        
        total
    }

    fn cleanup_inactive_sessions(&mut self) {
        let inactive_threshold = Duration::from_secs(300); // 5 minutes
        
        let inactive_sessions: Vec<String> = self.sessions
            .iter()
            .filter(|(_, session)| {
                matches!(session.state, DebugSessionState::Stopped) ||
                session.last_step.map_or(true, |t| t.elapsed().unwrap_or_default() > inactive_threshold)
            })
            .map(|(id, _)| id.clone())
            .collect();

        for session_id in inactive_sessions {
            self.sessions.remove(&session_id);
            println!("Cleaned up inactive debug session: {}", session_id);
        }
    }

    pub fn get_performance_report(&self) -> PerformanceReport {
        PerformanceReport {
            top_scripts: self.profiler.get_top_scripts(10),
            top_functions: self.profiler.get_top_functions(10),
            total_execution_time: self.profiler.total_execution_time,
            total_instructions: self.profiler.total_instructions,
            memory_usage: self.debug_info.total_memory_usage,
            active_sessions: self.sessions.len(),
            active_visualizations: self.visualizations.len(),
        }
    }

    pub fn get_debug_info(&self) -> &DebugInfo {
        &self.debug_info
    }

    pub fn get_session_info(&self, session_id: &str) -> Option<SessionInfo> {
        self.sessions.get(session_id).map(|session| SessionInfo {
            session_id: session.id.clone(),
            script_id: session.script_id.clone(),
            state: session.state.clone(),
            current_line: session.current_line,
            call_stack_depth: session.call_stack.len(),
            variable_count: session.variables.len(),
            breakpoint_count: session.breakpoints.len(),
            step_count: session.step_count,
        })
    }

    pub fn enable(&mut self) {
        self.enabled = true;
        self.profiler.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
        self.profiler.enabled = false;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn shutdown(&mut self) -> RobinResult<()> {
        println!("Script Debugger shutdown:");
        println!("  Total debug sessions: {}", self.sessions.len());
        println!("  Total breakpoints: {}", self.global_breakpoints.len());
        println!("  Total watchers: {}", self.watchers.len());
        println!("  Total visualizations: {}", self.visualizations.len());
        println!("  Total execution time: {:.2}s", self.profiler.total_execution_time.as_secs_f32());
        println!("  Total instructions: {}", self.profiler.total_instructions);
        
        self.sessions.clear();
        self.global_breakpoints.clear();
        self.watchers.clear();
        self.visualizations.clear();
        self.profiler.clear();
        
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceReport {
    pub top_scripts: Vec<ScriptProfile>,
    pub top_functions: Vec<FunctionProfile>,
    pub total_execution_time: Duration,
    pub total_instructions: u64,
    pub memory_usage: usize,
    pub active_sessions: usize,
    pub active_visualizations: usize,
}

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub session_id: String,
    pub script_id: String,
    pub state: DebugSessionState,
    pub current_line: Option<u32>,
    pub call_stack_depth: usize,
    pub variable_count: usize,
    pub breakpoint_count: usize,
    pub step_count: u64,
}

// Convenience functions for creating debug visualizations
pub fn debug_point(x: f32, y: f32, z: f32) -> DebugVisualization {
    DebugVisualization::point((x, y, z))
}

pub fn debug_line(start: (f32, f32, f32), end: (f32, f32, f32)) -> DebugVisualization {
    DebugVisualization::line(start, end)
}

pub fn debug_sphere(x: f32, y: f32, z: f32, radius: f32) -> DebugVisualization {
    DebugVisualization::sphere((x, y, z), radius)
}

pub fn debug_text(x: f32, y: f32, z: f32, text: &str) -> DebugVisualization {
    DebugVisualization::text((x, y, z), text.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breakpoint_creation() {
        let breakpoint = BreakPoint::new("script1".to_string(), 42)
            .with_condition("health < 10".to_string())
            .with_hit_count_condition(BreakPointCondition::HitCountEquals(5));
        
        assert_eq!(breakpoint.script_id, "script1");
        assert_eq!(breakpoint.line, 42);
        assert!(breakpoint.enabled);
        assert_eq!(breakpoint.condition, Some("health < 10".to_string()));
        assert_eq!(breakpoint.hit_count, 0);
    }

    #[test]
    fn test_breakpoint_hit_conditions() {
        let mut breakpoint = BreakPoint::new("script1".to_string(), 1)
            .with_hit_count_condition(BreakPointCondition::HitCountEquals(3));
        
        assert!(!breakpoint.should_break()); // hit_count = 0
        breakpoint.hit();
        assert!(!breakpoint.should_break()); // hit_count = 1
        breakpoint.hit();
        assert!(!breakpoint.should_break()); // hit_count = 2
        breakpoint.hit();
        assert!(breakpoint.should_break());  // hit_count = 3
    }

    #[test]
    fn test_variable_watcher() {
        let mut watcher = VariableWatcher::new(
            "Health Monitor".to_string(),
            "player.health".to_string()
        ).with_condition(WatchCondition::OnLess(25.0));

        let triggered1 = watcher.add_value(RuntimeValue::Float(50.0));
        assert!(!triggered1);

        let triggered2 = watcher.add_value(RuntimeValue::Float(20.0));
        assert!(triggered2);

        assert_eq!(watcher.value_history.len(), 2);
        assert_eq!(watcher.get_current_value().unwrap().as_string(), "20");
    }

    #[test]
    fn test_execution_trace() {
        let trace = ExecutionTrace::new(
            "script1".to_string(),
            10,
            "LOAD_VARIABLE health".to_string()
        )
        .with_function("update_player".to_string())
        .with_stack_depth(2)
        .with_execution_time(0.5);

        assert_eq!(trace.script_id, "script1");
        assert_eq!(trace.line, 10);
        assert_eq!(trace.instruction, "LOAD_VARIABLE health");
        assert_eq!(trace.function_name, Some("update_player".to_string()));
        assert_eq!(trace.stack_depth, 2);
        assert_eq!(trace.execution_time_ms, 0.5);
    }

    #[test]
    fn test_debug_session_workflow() {
        let mut session = DebugSession::new("test_script".to_string());
        
        assert_eq!(session.script_id, "test_script");
        assert!(matches!(session.state, DebugSessionState::Stopped));
        
        session.start();
        assert!(matches!(session.state, DebugSessionState::Running));
        assert!(session.is_running());
        
        session.pause();
        assert!(matches!(session.state, DebugSessionState::Paused));
        assert!(session.is_paused());
        
        session.step_over();
        assert!(session.is_stepping());
        assert_eq!(session.step_count, 1);
    }

    #[test]
    fn test_performance_profiler() {
        let mut profiler = PerformanceProfiler::new(30.0, 100);
        
        profiler.start_script_execution("script1");
        profiler.end_script_execution("script1", Duration::from_millis(10), 100, 1024);
        
        profiler.record_function_call("main", "script1", Duration::from_millis(5), 0);
        profiler.record_line_execution("script1", 5, Duration::from_millis(2));
        
        let top_scripts = profiler.get_top_scripts(5);
        assert_eq!(top_scripts.len(), 1);
        assert_eq!(top_scripts[0].script_id, "script1");
        
        let top_functions = profiler.get_top_functions(5);
        assert_eq!(top_functions.len(), 1);
        assert_eq!(top_functions[0].function_name, "main");
    }

    #[test]
    fn test_debug_visualizations() {
        let point = debug_point(1.0, 2.0, 3.0)
            .with_color(1.0, 0.0, 0.0, 1.0)
            .with_duration(Duration::from_secs(10));
        
        assert!(matches!(point.visualization_type, VisualizationType::Point));
        assert_eq!(point.position, (1.0, 2.0, 3.0));
        assert_eq!(point.color, (1.0, 0.0, 0.0, 1.0));
        assert!(!point.is_expired());
        
        let line = debug_line((0.0, 0.0, 0.0), (5.0, 5.0, 5.0));
        assert!(matches!(line.visualization_type, VisualizationType::Line));
        
        let sphere = debug_sphere(0.0, 0.0, 0.0, 2.5);
        assert!(matches!(sphere.visualization_type, VisualizationType::Sphere));
        
        let text = debug_text(1.0, 1.0, 1.0, "Debug Info");
        assert!(matches!(text.visualization_type, VisualizationType::Text));
    }

    #[test]
    fn test_debugger_integration() {
        let mut debugger = ScriptDebugger::new(true).unwrap();
        debugger.initialize().unwrap();
        
        // Start debug session
        let session_id = debugger.start_debug_session("test_script").unwrap();
        assert!(!session_id.is_empty());
        
        // Add breakpoint
        debugger.add_breakpoint("test_script", 10).unwrap();
        
        // Add variable watcher
        let watcher_id = debugger.add_variable_watcher(
            "Health".to_string(),
            "player.health".to_string(),
            Some("test_script".to_string())
        ).unwrap();
        assert!(!watcher_id.is_empty());
        
        // Add debug visualization
        let viz = debug_point(1.0, 2.0, 3.0);
        let viz_id = debugger.add_debug_visualization(viz);
        assert!(!viz_id.is_empty());
        
        // Update debugger
        debugger.update(0.016).unwrap();
        
        // Check debug info
        let debug_info = debugger.get_debug_info();
        assert_eq!(debug_info.active_sessions, 1);
        assert_eq!(debug_info.total_breakpoints, 1);
        assert_eq!(debug_info.total_watchers, 1);
        assert_eq!(debug_info.debug_visualizations, 1);
    }
}