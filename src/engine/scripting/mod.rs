// Advanced Scripting and Logic Systems for Robin Engine
// Provides visual scripting, behavior trees, event systems, and debugging tools for Engineer Build Mode

pub mod visual_editor;
pub mod behavior_trees;
pub mod event_system;
pub mod templates;
pub mod debug_tools;
pub mod runtime;

pub use visual_editor::{
    VisualScriptEditor, NodeGraph, ScriptNode, NodeConnection, NodePin,
    NodeType, PinType, NodeCategory, EditorConfig, NodeLibrary
};

pub use behavior_trees::{
    BehaviorTreeSystem, BehaviorTree, BehaviorNode, NodeStatus, 
    Blackboard, TreeConfig, RuntimeValue as BehaviorRuntimeValue
};

pub use event_system::{
    EventSystem, Event, EventHandler, EventTrigger, EventCondition,
    EventAction, GlobalEventBus, EventSubscription, EventPriority,
    RuntimeValue as EventRuntimeValue
};

pub use templates::{
    ScriptTemplateManager, ScriptTemplate, TemplateCategory, TemplateParameter,
    BuiltinTemplates, CustomTemplate, ParameterType, TemplateLibrary,
    RuntimeValue as TemplateRuntimeValue
};

pub use debug_tools::{
    ScriptDebugger, DebugSession, BreakPoint, DebugInfo, ExecutionTrace,
    VariableWatcher, PerformanceProfiler, DebugVisualization,
    RuntimeValue as DebugRuntimeValue
};

pub use runtime::{
    ScriptRuntime, ExecutionContext, RuntimeStats,
    ScriptValue, ScriptResult, ScriptError, Script
};

use crate::engine::error::RobinResult;
use crate::engine::ai::narrative_ai_systems::RuntimeValue as NarrativeRuntimeValue;
use std::time::Instant;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptingSystemConfig {
    pub max_concurrent_scripts: u32,
    pub execution_time_limit_ms: u32,
    pub enable_visual_debugging: bool,
    pub enable_performance_profiling: bool,
    pub enable_hot_reload: bool,
    pub auto_save_interval: f32,
    pub node_execution_limit: u32,
    pub memory_limit_mb: u32,
}

impl Default for ScriptingSystemConfig {
    fn default() -> Self {
        Self {
            max_concurrent_scripts: 100,
            execution_time_limit_ms: 16, // 60fps budget
            enable_visual_debugging: true,
            enable_performance_profiling: true,
            enable_hot_reload: true,
            auto_save_interval: 30.0,
            node_execution_limit: 10000,
            memory_limit_mb: 256,
        }
    }
}

#[derive(Debug, Default)]
pub struct ScriptingStats {
    pub active_scripts: u32,
    pub nodes_executed_per_frame: u32,
    pub avg_execution_time_ms: f32,
    pub memory_usage_mb: f32,
    pub cpu_usage_percent: f32,
    pub events_processed_per_second: u32,
    pub behavior_trees_active: u32,
    pub template_instances: u32,
}

#[derive(Debug)]
pub struct ScriptingSystem {
    config: ScriptingSystemConfig,
    visual_editor: VisualScriptEditor,
    behavior_tree_system: BehaviorTreeSystem,
    event_system: EventSystem,
    template_manager: ScriptTemplateManager,
    debugger: ScriptDebugger,
    runtime: ScriptRuntime,
    stats: ScriptingStats,
    frame_counter: u64,
    last_stats_update: Instant,
    auto_save_timer: f32,
}

impl ScriptingSystem {
    pub fn new(config: ScriptingSystemConfig) -> RobinResult<Self> {
        let visual_editor = VisualScriptEditor::new(EditorConfig::default())?;
        let behavior_tree_system = BehaviorTreeSystem::new(TreeConfig::default())?;
        let event_system = EventSystem::new()?;
        let template_manager = ScriptTemplateManager::new()?;
        let debugger = ScriptDebugger::new(config.enable_visual_debugging)?;
        let runtime = ScriptRuntime::new(config.clone())?;

        Ok(Self {
            config,
            visual_editor,
            behavior_tree_system,
            event_system,
            template_manager,
            debugger,
            runtime,
            stats: ScriptingStats::default(),
            frame_counter: 0,
            last_stats_update: Instant::now(),
            auto_save_timer: 0.0,
        })
    }

    pub fn initialize(&mut self) -> RobinResult<()> {
        // Initialize all subsystems
        self.visual_editor.initialize()?;
        self.behavior_tree_system.initialize()?;
        self.event_system.initialize()?;
        self.template_manager.load_builtin_templates()?;
        self.debugger.initialize()?;
        self.runtime.initialize()?;

        println!("Advanced Scripting System initialized:");
        println!("  Visual Editor: Enabled");
        println!("  Behavior Trees: Enabled");
        println!("  Event System: Enabled");
        println!("  Script Templates: {} loaded", self.template_manager.get_template_count());
        println!("  Visual Debugging: {}", self.config.enable_visual_debugging);
        println!("  Hot Reload: {}", self.config.enable_hot_reload);
        println!("  Max Concurrent Scripts: {}", self.config.max_concurrent_scripts);

        Ok(())
    }

    pub fn update(&mut self, delta_time: f32) -> RobinResult<()> {
        self.frame_counter += 1;
        self.auto_save_timer += delta_time;

        let frame_start = Instant::now();

        // Update all scripting subsystems
        self.runtime.update(delta_time)?;
        self.visual_editor.update(delta_time)?;
        self.behavior_tree_system.update(delta_time)?;
        self.event_system.update(delta_time)?;
        
        if self.config.enable_visual_debugging {
            self.debugger.update(delta_time)?;
        }

        // Auto-save check
        if self.auto_save_timer >= self.config.auto_save_interval {
            self.auto_save_scripts()?;
            self.auto_save_timer = 0.0;
        }

        // Update statistics
        if self.last_stats_update.elapsed().as_secs_f32() > 1.0 {
            self.update_scripting_stats(frame_start.elapsed().as_secs_f32() * 1000.0);
            self.last_stats_update = Instant::now();
        }

        Ok(())
    }

    // Visual Editor Operations
    pub fn create_node_graph(&mut self, name: String) -> RobinResult<String> {
        self.visual_editor.create_new_graph(name)
    }

    pub fn add_node_to_graph(&mut self, graph_id: &str, node_type: NodeType, position: (f32, f32)) -> RobinResult<String> {
        self.visual_editor.add_node(graph_id, node_type, position)
    }

    pub fn connect_nodes(&mut self, graph_id: &str, from_node: &str, from_pin: &str, to_node: &str, to_pin: &str) -> RobinResult<()> {
        self.visual_editor.create_connection(graph_id, from_node, from_pin, to_node, to_pin)
    }

    pub fn compile_node_graph(&mut self, graph_id: &str) -> RobinResult<String> {
        self.visual_editor.compile_graph(graph_id)
    }

    // Behavior Tree Operations
    pub fn create_behavior_tree(&mut self, name: String) -> RobinResult<String> {
        self.behavior_tree_system.create_tree(name)
    }

    pub fn assign_behavior_tree(&mut self, entity_id: &str, tree_id: &str) -> RobinResult<()> {
        self.behavior_tree_system.assign_tree_to_entity(entity_id, tree_id)
    }

    pub fn update_blackboard(&mut self, entity_id: &str, key: String, value: BehaviorRuntimeValue) -> RobinResult<()> {
        self.behavior_tree_system.update_blackboard(entity_id, key, value)
    }

    // Event System Operations
    pub fn register_event_handler(&mut self, event_name: String, handler: EventHandler) -> RobinResult<String> {
        self.event_system.register_handler(event_name, handler.condition, handler.action)
    }

    pub fn trigger_event(&mut self, event: Event) -> RobinResult<()> {
        self.event_system.trigger_event(event)
    }

    pub fn create_event_trigger(&mut self, name: String, condition: EventCondition, action: EventAction) -> RobinResult<String> {
        self.event_system.create_trigger(name, condition, action)
    }

    // Template System Operations
    pub fn instantiate_template(&mut self, template_name: &str, parameters: std::collections::HashMap<String, TemplateRuntimeValue>) -> RobinResult<String> {
        self.template_manager.instantiate_template(template_name, parameters)
    }

    pub fn create_custom_template(&mut self, name: String, template: CustomTemplate) -> RobinResult<()> {
        self.template_manager.add_custom_template(name, template)
    }

    pub fn get_available_templates(&self) -> Vec<String> {
        self.template_manager.get_template_names()
    }

    // Debugging Operations
    pub fn start_debug_session(&mut self, script_id: &str) -> RobinResult<String> {
        self.debugger.start_debug_session(script_id)
    }

    pub fn add_breakpoint(&mut self, script_id: &str, line: u32) -> RobinResult<()> {
        self.debugger.add_breakpoint(script_id, line)
    }

    pub fn step_execution(&mut self, session_id: &str) -> RobinResult<()> {
        self.debugger.step_execution(session_id)
    }

    pub fn get_variable_value(&self, session_id: &str, variable_name: &str) -> RobinResult<DebugRuntimeValue> {
        self.debugger.get_variable_value(session_id, variable_name)
    }

    // Runtime Operations  
    pub fn execute_script(&mut self, script_id: &str) -> RobinResult<ScriptValue> {
        let mut context = ExecutionContext::new();
        match self.runtime.execute_script(script_id, &mut context) {
            Ok(value) => Ok(value),
            Err(script_error) => Err(crate::engine::error::RobinError::new(&format!("Script execution failed: {:?}", script_error))),
        }
    }

    pub fn pause_script(&mut self, script_id: &str) -> RobinResult<()> {
        self.runtime.pause_script(script_id)
    }

    pub fn resume_script(&mut self, script_id: &str) -> RobinResult<()> {
        self.runtime.resume_script(script_id)
    }

    pub fn stop_script(&mut self, script_id: &str) -> RobinResult<()> {
        self.runtime.stop_script(script_id)
    }

    // System Management
    pub fn hot_reload_scripts(&mut self) -> RobinResult<()> {
        if self.config.enable_hot_reload {
            self.runtime.reload_modified_scripts()?;
        }
        Ok(())
    }

    pub fn get_scripting_stats(&self) -> &ScriptingStats {
        &self.stats
    }

    pub fn export_node_graph(&self, graph_id: &str) -> RobinResult<String> {
        self.visual_editor.export_graph(graph_id)
    }

    pub fn import_node_graph(&mut self, graph_data: &str) -> RobinResult<String> {
        self.visual_editor.import_graph(graph_data)
    }

    pub fn validate_script(&self, script_id: &str) -> RobinResult<Vec<String>> {
        // For now, return a stub implementation since we need to first get the script by ID
        // TODO: Implement proper script lookup by ID and validation
        Ok(vec!["Script validation not yet implemented".to_string()])
    }

    pub fn optimize_script(&mut self, script_id: &str) -> RobinResult<()> {
        self.runtime.optimize_script(script_id)
    }

    // Integration with Engineer Build Mode
    pub fn on_building_action(&mut self, action: &str, parameters: std::collections::HashMap<String, NarrativeRuntimeValue>) -> RobinResult<()> {
        // Trigger building-related events
        // TODO: Convert NarrativeRuntimeValue to event_system::RuntimeValue
        let event_params = std::collections::HashMap::new(); // Stub for now
        let event = Event::new(format!("building.{}", action), event_params);
        self.trigger_event(event)?;
        Ok(())
    }

    pub fn on_player_action(&mut self, player_id: &str, action: &str, context: std::collections::HashMap<String, NarrativeRuntimeValue>) -> RobinResult<()> {
        // Update behavior tree blackboards and trigger events
        if let Err(_) = self.update_blackboard(player_id, "last_action".to_string(), BehaviorRuntimeValue::String(action.to_string())) {
            // Player might not have a behavior tree assigned, which is fine
        }

        // TODO: Convert NarrativeRuntimeValue context to event_system::RuntimeValue
        let mut event_data = std::collections::HashMap::new();
        event_data.insert("player_id".to_string(), EventRuntimeValue::String(player_id.to_string()));
        event_data.insert("action".to_string(), EventRuntimeValue::String(action.to_string()));

        let event = Event::new(format!("player.{}", action), event_data);
        self.trigger_event(event)?;
        Ok(())
    }

    pub fn register_building_script(&mut self, building_type: &str, script_graph_id: String) -> RobinResult<()> {
        // Register a script to be executed when buildings of this type are interacted with
        let handler = EventHandler::new(
            format!("building_script_{}", building_type),
            format!("building_interaction_{}", building_type),
            EventCondition::Always,
            EventAction::CallFunction(format!("building_script_{}", building_type), vec![])
        );

        self.register_event_handler(format!("building.interact.{}", building_type), handler)?;
        Ok(())
    }

    fn update_scripting_stats(&mut self, frame_time_ms: f32) {
        self.stats.active_scripts = self.runtime.get_active_script_count() as u32;
        self.stats.nodes_executed_per_frame = self.runtime.get_nodes_executed_this_frame() as u32;
        self.stats.avg_execution_time_ms = frame_time_ms;
        self.stats.memory_usage_mb = self.runtime.get_memory_usage_mb() as f32;
        self.stats.cpu_usage_percent = (frame_time_ms / 16.67 * 100.0).min(100.0); // Assuming 60fps target
        self.stats.events_processed_per_second = self.event_system.get_events_processed_per_second();
        self.stats.behavior_trees_active = self.behavior_tree_system.get_active_tree_count();
        self.stats.template_instances = self.template_manager.get_active_instance_count();
    }

    fn auto_save_scripts(&mut self) -> RobinResult<()> {
        // Auto-save all modified scripts and graphs
        self.visual_editor.auto_save_modified_graphs()?;
        self.behavior_tree_system.auto_save_modified_trees()?;
        Ok(())
    }

    pub fn shutdown(&mut self) -> RobinResult<()> {
        println!("Advanced Scripting System shutdown:");
        println!("  Scripts executed: {}", self.frame_counter);
        println!("  Peak active scripts: {}", self.stats.active_scripts);
        println!("  Peak nodes per frame: {}", self.stats.nodes_executed_per_frame);
        println!("  Average execution time: {:.2}ms", self.stats.avg_execution_time_ms);
        println!("  Peak memory usage: {:.1}MB", self.stats.memory_usage_mb);

        self.runtime.shutdown()?;
        self.visual_editor.shutdown()?;
        self.behavior_tree_system.shutdown()?;
        self.event_system.shutdown()?;
        self.template_manager.shutdown()?;
        self.debugger.shutdown()?;

        Ok(())
    }
}