// Event System for Robin Engine
// Provides comprehensive trigger-based interactions and event-driven programming

use crate::engine::error::RobinResult;
use std::collections::{HashMap, VecDeque, BTreeMap};
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub name: String,
    pub data: HashMap<String, RuntimeValue>,
    pub timestamp: u64,
    pub priority: EventPriority,
    pub source: String,
    pub propagation_stopped: bool,
}

impl Event {
    pub fn new(name: String, data: HashMap<String, RuntimeValue>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            data,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            priority: EventPriority::Normal,
            source: "unknown".to_string(),
            propagation_stopped: false,
        }
    }

    pub fn with_priority(mut self, priority: EventPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_source(mut self, source: String) -> Self {
        self.source = source;
        self
    }

    pub fn get_data<T>(&self, key: &str) -> Option<T>
    where
        T: for<'a> TryFrom<&'a RuntimeValue>,
    {
        self.data.get(key).and_then(|v| T::try_from(v).ok())
    }

    pub fn set_data(&mut self, key: String, value: RuntimeValue) {
        self.data.insert(key, value);
    }

    pub fn stop_propagation(&mut self) {
        self.propagation_stopped = true;
    }

    pub fn is_propagation_stopped(&self) -> bool {
        self.propagation_stopped
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

    pub fn as_int(&self) -> i32 {
        match self {
            RuntimeValue::Int(i) => *i,
            RuntimeValue::Float(f) => *f as i32,
            RuntimeValue::Bool(b) => if *b { 1 } else { 0 },
            _ => 0,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EventPriority {
    Critical = 0,
    High = 1,
    Normal = 2,
    Low = 3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventCondition {
    Always,
    Never,
    KeyExists(String),
    KeyEquals(String, RuntimeValue),
    KeyGreater(String, f32),
    KeyLess(String, f32),
    And(Box<EventCondition>, Box<EventCondition>),
    Or(Box<EventCondition>, Box<EventCondition>),
    Not(Box<EventCondition>),
    Custom(String), // References a custom condition function
}

impl EventCondition {
    pub fn evaluate(&self, event: &Event, custom_conditions: &HashMap<String, Box<dyn Fn(&Event) -> bool + Send + Sync>>) -> bool {
        match self {
            EventCondition::Always => true,
            EventCondition::Never => false,
            EventCondition::KeyExists(key) => event.data.contains_key(key),
            EventCondition::KeyEquals(key, expected) => {
                event.data.get(key).map_or(false, |v| self.values_equal(v, expected))
            }
            EventCondition::KeyGreater(key, threshold) => {
                event.data.get(key).map_or(false, |v| v.as_float() > *threshold)
            }
            EventCondition::KeyLess(key, threshold) => {
                event.data.get(key).map_or(false, |v| v.as_float() < *threshold)
            }
            EventCondition::And(a, b) => {
                a.evaluate(event, custom_conditions) && b.evaluate(event, custom_conditions)
            }
            EventCondition::Or(a, b) => {
                a.evaluate(event, custom_conditions) || b.evaluate(event, custom_conditions)
            }
            EventCondition::Not(condition) => {
                !condition.evaluate(event, custom_conditions)
            }
            EventCondition::Custom(name) => {
                custom_conditions.get(name).map_or(false, |f| f(event))
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventAction {
    LogMessage(String),
    SetVariable(String, RuntimeValue),
    TriggerEvent(String, HashMap<String, RuntimeValue>),
    CallFunction(String, Vec<RuntimeValue>),
    Sequence(Vec<EventAction>),
    Conditional {
        condition: EventCondition,
        then_action: Box<EventAction>,
        else_action: Option<Box<EventAction>>,
    },
    Delay {
        duration_ms: u64,
        action: Box<EventAction>,
    },
    Custom(String), // References a custom action function
}

impl EventAction {
    pub fn execute(&self, 
                   event: &Event, 
                   context: &mut EventContext,
                   custom_actions: &HashMap<String, Box<dyn Fn(&Event, &mut EventContext) -> RobinResult<()> + Send + Sync>>,
                   custom_conditions: &HashMap<String, Box<dyn Fn(&Event) -> bool + Send + Sync>>) -> RobinResult<()> {
        match self {
            EventAction::LogMessage(message) => {
                println!("Event Log [{}]: {}", event.name, message);
                Ok(())
            }
            EventAction::SetVariable(key, value) => {
                context.variables.insert(key.clone(), value.clone());
                Ok(())
            }
            EventAction::TriggerEvent(event_name, data) => {
                let mut new_event = Event::new(event_name.clone(), data.clone());
                new_event.source = format!("triggered_by_{}", event.id);
                context.triggered_events.push(new_event);
                Ok(())
            }
            EventAction::CallFunction(function_name, args) => {
                // In a real implementation, this would call registered functions
                println!("Calling function: {} with {} args", function_name, args.len());
                Ok(())
            }
            EventAction::Sequence(actions) => {
                for action in actions {
                    action.execute(event, context, custom_actions, custom_conditions)?;
                }
                Ok(())
            }
            EventAction::Conditional { condition, then_action, else_action } => {
                if condition.evaluate(event, custom_conditions) {
                    then_action.execute(event, context, custom_actions, custom_conditions)
                } else if let Some(else_action) = else_action {
                    else_action.execute(event, context, custom_actions, custom_conditions)
                } else {
                    Ok(())
                }
            }
            EventAction::Delay { duration_ms, action } => {
                let delayed_action = DelayedAction {
                    id: Uuid::new_v4().to_string(),
                    execute_at: Instant::now() + Duration::from_millis(*duration_ms),
                    action: action.clone(),
                    event: event.clone(),
                };
                context.delayed_actions.push(delayed_action);
                Ok(())
            }
            EventAction::Custom(name) => {
                if let Some(action_fn) = custom_actions.get(name) {
                    action_fn(event, context)
                } else {
                    Err(crate::engine::error::RobinError::InvalidInput(
                        format!("Custom action '{}' not found", name)
                    ))
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct DelayedAction {
    pub id: String,
    pub execute_at: Instant,
    pub action: Box<EventAction>,
    pub event: Event,
}

#[derive(Debug)]
pub struct EventContext {
    pub variables: HashMap<String, RuntimeValue>,
    pub triggered_events: Vec<Event>,
    pub delayed_actions: Vec<DelayedAction>,
}

impl EventContext {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            triggered_events: Vec::new(),
            delayed_actions: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.triggered_events.clear();
    }
}

#[derive(Debug)]
pub struct EventHandler {
    pub id: String,
    pub name: String,
    pub event_pattern: String, // Regex pattern to match event names
    pub condition: EventCondition,
    pub action: EventAction,
    pub enabled: bool,
    pub execution_count: u64,
    pub last_execution: Option<Instant>,
    pub cooldown_ms: Option<u64>,
}

impl EventHandler {
    pub fn new(name: String, event_pattern: String, condition: EventCondition, action: EventAction) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            event_pattern,
            condition,
            action,
            enabled: true,
            execution_count: 0,
            last_execution: None,
            cooldown_ms: None,
        }
    }

    pub fn with_cooldown(mut self, cooldown_ms: u64) -> Self {
        self.cooldown_ms = Some(cooldown_ms);
        self
    }

    pub fn can_execute(&self) -> bool {
        if !self.enabled {
            return false;
        }

        if let (Some(cooldown), Some(last_exec)) = (self.cooldown_ms, self.last_execution) {
            last_exec.elapsed() >= Duration::from_millis(cooldown)
        } else {
            true
        }
    }

    pub fn matches_event(&self, event: &Event) -> bool {
        // Simple pattern matching - in a full implementation, this would use regex
        if self.event_pattern == "*" {
            return true;
        }
        
        if self.event_pattern.contains('*') {
            let pattern = self.event_pattern.replace('*', "");
            event.name.contains(&pattern)
        } else {
            event.name == self.event_pattern
        }
    }
}

#[derive(Debug)]
pub struct EventTrigger {
    pub id: String,
    pub name: String,
    pub condition: EventCondition,
    pub action: EventAction,
    pub trigger_type: TriggerType,
    pub enabled: bool,
    pub activation_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerType {
    Immediate,
    Delayed(u64), // milliseconds
    Interval(u64), // milliseconds
    Once,
}

impl EventTrigger {
    pub fn new(name: String, condition: EventCondition, action: EventAction) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            condition,
            action,
            trigger_type: TriggerType::Immediate,
            enabled: true,
            activation_count: 0,
        }
    }

    pub fn with_trigger_type(mut self, trigger_type: TriggerType) -> Self {
        self.trigger_type = trigger_type;
        self
    }
}

#[derive(Debug)]
pub struct EventSubscription {
    pub id: String,
    pub handler_id: String,
    pub event_pattern: String,
    pub created_at: Instant,
    pub active: bool,
}

impl EventSubscription {
    pub fn new(handler_id: String, event_pattern: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            handler_id,
            event_pattern,
            created_at: Instant::now(),
            active: true,
        }
    }
}

#[derive(Debug)]
pub struct GlobalEventBus {
    events: Arc<Mutex<VecDeque<Event>>>,
    max_queue_size: usize,
}

impl GlobalEventBus {
    pub fn new(max_queue_size: usize) -> Self {
        Self {
            events: Arc::new(Mutex::new(VecDeque::new())),
            max_queue_size,
        }
    }

    pub fn publish(&self, event: Event) -> RobinResult<()> {
        let mut queue = self.events.lock().map_err(|_| {
            crate::engine::error::RobinError::SystemError("Failed to lock event queue".to_string())
        })?;

        if queue.len() >= self.max_queue_size {
            queue.pop_front(); // Remove oldest event
        }

        queue.push_back(event);
        Ok(())
    }

    pub fn subscribe(&self) -> Arc<Mutex<VecDeque<Event>>> {
        Arc::clone(&self.events)
    }

    pub fn drain_events(&self) -> RobinResult<Vec<Event>> {
        let mut queue = self.events.lock().map_err(|_| {
            crate::engine::error::RobinError::SystemError("Failed to lock event queue".to_string())
        })?;

        let events: Vec<Event> = queue.drain(..).collect();
        Ok(events)
    }

    pub fn get_queue_size(&self) -> usize {
        self.events.lock().map(|q| q.len()).unwrap_or(0)
    }
}

pub struct EventSystem {
    handlers: HashMap<String, EventHandler>,
    triggers: HashMap<String, EventTrigger>,
    subscriptions: Vec<EventSubscription>,
    custom_conditions: HashMap<String, Box<dyn Fn(&Event) -> bool + Send + Sync>>,
    custom_actions: HashMap<String, Box<dyn Fn(&Event, &mut EventContext) -> RobinResult<()> + Send + Sync>>,
    global_bus: GlobalEventBus,
    context: EventContext,
    event_queue: BTreeMap<EventPriority, VecDeque<Event>>,
    stats: EventSystemStats,
    last_stats_update: Instant,
    processing_enabled: bool,
}

impl std::fmt::Debug for EventSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventSystem")
            .field("handlers", &self.handlers)
            .field("triggers", &self.triggers)
            .field("subscriptions", &self.subscriptions)
            .field("custom_conditions", &format!("<{} conditions>", self.custom_conditions.len()))
            .field("custom_actions", &format!("<{} actions>", self.custom_actions.len()))
            .field("global_bus", &self.global_bus)
            .field("context", &self.context)
            .field("event_queue", &self.event_queue)
            .field("stats", &self.stats)
            .field("last_stats_update", &self.last_stats_update)
            .field("processing_enabled", &self.processing_enabled)
            .finish()
    }
}

#[derive(Debug, Default)]
pub struct EventSystemStats {
    pub events_processed_total: u64,
    pub events_processed_per_second: u32,
    pub handlers_executed_total: u64,
    pub triggers_activated_total: u64,
    pub custom_conditions_evaluated: u64,
    pub custom_actions_executed: u64,
    pub average_processing_time_ms: f32,
    pub queue_sizes: HashMap<String, u32>,
}

impl EventSystem {
    pub fn new() -> RobinResult<Self> {
        let mut event_queue = BTreeMap::new();
        event_queue.insert(EventPriority::Critical, VecDeque::new());
        event_queue.insert(EventPriority::High, VecDeque::new());
        event_queue.insert(EventPriority::Normal, VecDeque::new());
        event_queue.insert(EventPriority::Low, VecDeque::new());

        Ok(Self {
            handlers: HashMap::new(),
            triggers: HashMap::new(),
            subscriptions: Vec::new(),
            custom_conditions: HashMap::new(),
            custom_actions: HashMap::new(),
            global_bus: GlobalEventBus::new(10000),
            context: EventContext::new(),
            event_queue,
            stats: EventSystemStats::default(),
            last_stats_update: Instant::now(),
            processing_enabled: true,
        })
    }

    pub fn initialize(&mut self) -> RobinResult<()> {
        println!("Event System initialized:");
        println!("  Global event bus: Enabled");
        println!("  Priority queues: 4 levels (Critical, High, Normal, Low)");
        println!("  Custom conditions and actions: Supported");
        
        self.register_builtin_conditions()?;
        self.register_builtin_actions()?;
        
        Ok(())
    }

    fn register_builtin_conditions(&mut self) -> RobinResult<()> {
        // Building system conditions
        self.register_custom_condition("building.block_placed".to_string(), Box::new(|event: &Event| {
            event.data.contains_key("block_type") && event.data.contains_key("position")
        }))?;

        self.register_custom_condition("player.health_low".to_string(), Box::new(|event: &Event| {
            event.data.get("health").map_or(false, |v| v.as_float() < 25.0)
        }))?;

        self.register_custom_condition("npc.in_range".to_string(), Box::new(|event: &Event| {
            event.data.get("distance").map_or(false, |v| v.as_float() < 10.0)
        }))?;

        Ok(())
    }

    fn register_builtin_actions(&mut self) -> RobinResult<()> {
        // Building system actions
        self.register_custom_action("building.save_structure".to_string(), 
            Box::new(|event: &Event, context: &mut EventContext| {
                if let Some(structure_data) = event.data.get("structure") {
                    context.variables.insert("last_saved_structure".to_string(), structure_data.clone());
                    println!("Structure saved: {}", structure_data.as_string());
                }
                Ok(())
            })
        )?;

        self.register_custom_action("player.show_notification".to_string(),
            Box::new(|event: &Event, _context: &mut EventContext| {
                if let Some(message) = event.data.get("message") {
                    println!("NOTIFICATION: {}", message.as_string());
                }
                Ok(())
            })
        )?;

        self.register_custom_action("ai.update_behavior".to_string(),
            Box::new(|event: &Event, context: &mut EventContext| {
                if let (Some(entity_id), Some(behavior)) = (event.data.get("entity_id"), event.data.get("behavior")) {
                    context.variables.insert(
                        format!("ai_behavior_{}", entity_id.as_string()),
                        behavior.clone()
                    );
                    println!("AI behavior updated for entity: {}", entity_id.as_string());
                }
                Ok(())
            })
        )?;

        Ok(())
    }

    pub fn register_handler(&mut self, event_pattern: String, condition: EventCondition, action: EventAction) -> RobinResult<String> {
        let handler = EventHandler::new(
            format!("handler_{}", self.handlers.len()),
            event_pattern.clone(),
            condition,
            action
        );

        let handler_id = handler.id.clone();
        self.handlers.insert(handler_id.clone(), handler);

        let subscription = EventSubscription::new(handler_id.clone(), event_pattern);
        self.subscriptions.push(subscription);

        Ok(handler_id)
    }

    pub fn create_trigger(&mut self, name: String, condition: EventCondition, action: EventAction) -> RobinResult<String> {
        let trigger = EventTrigger::new(name, condition, action);
        let trigger_id = trigger.id.clone();
        
        self.triggers.insert(trigger_id.clone(), trigger);
        Ok(trigger_id)
    }

    pub fn register_custom_condition<F>(&mut self, name: String, condition: F) -> RobinResult<()>
    where
        F: Fn(&Event) -> bool + Send + Sync + 'static,
    {
        self.custom_conditions.insert(name, Box::new(condition));
        Ok(())
    }

    pub fn register_custom_action<F>(&mut self, name: String, action: F) -> RobinResult<()>
    where
        F: Fn(&Event, &mut EventContext) -> RobinResult<()> + Send + Sync + 'static,
    {
        self.custom_actions.insert(name, Box::new(action));
        Ok(())
    }

    pub fn trigger_event(&mut self, event: Event) -> RobinResult<()> {
        if !self.processing_enabled {
            return Ok(());
        }

        let priority = event.priority;
        if let Some(queue) = self.event_queue.get_mut(&priority) {
            queue.push_back(event.clone());
        }

        self.global_bus.publish(event)?;
        Ok(())
    }

    pub fn update(&mut self, _delta_time: f32) -> RobinResult<()> {
        if !self.processing_enabled {
            return Ok(());
        }

        let processing_start = Instant::now();
        let mut events_processed = 0u32;

        // Process events by priority
        for priority in [EventPriority::Critical, EventPriority::High, EventPriority::Normal, EventPriority::Low] {
            let mut events_to_process = Vec::new();

            // Collect events first to avoid borrowing conflicts
            if let Some(queue) = self.event_queue.get_mut(&priority) {
                while let Some(event) = queue.pop_front() {
                    events_to_process.push(event);

                    // Prevent overlong processing
                    if processing_start.elapsed() > Duration::from_millis(16) {
                        break;
                    }
                }
            }

            // Process collected events
            for event in events_to_process {
                self.process_event(event)?;
                events_processed += 1;

                // Check time again
                if processing_start.elapsed() > Duration::from_millis(16) {
                    break;
                }
            }
        }

        // Process delayed actions
        self.process_delayed_actions()?;

        // Process triggered events from context
        let triggered_events = std::mem::take(&mut self.context.triggered_events);
        for event in triggered_events {
            self.trigger_event(event)?;
        }

        // Update statistics
        if self.last_stats_update.elapsed() >= Duration::from_secs(1) {
            self.update_stats(events_processed, processing_start.elapsed().as_secs_f32() * 1000.0);
            self.last_stats_update = Instant::now();
        }

        Ok(())
    }

    fn process_event(&mut self, event: Event) -> RobinResult<()> {
        if event.is_propagation_stopped() {
            return Ok(());
        }

        // Execute matching handlers
        let handler_ids: Vec<String> = self.handlers
            .iter()
            .filter(|(_, handler)| handler.matches_event(&event) && handler.can_execute())
            .map(|(id, _)| id.clone())
            .collect();

        for handler_id in handler_ids {
            if let Some(handler) = self.handlers.get_mut(&handler_id) {
                if handler.condition.evaluate(&event, &self.custom_conditions) {
                    handler.action.execute(&event, &mut self.context, &self.custom_actions, &self.custom_conditions)?;
                    handler.execution_count += 1;
                    handler.last_execution = Some(Instant::now());
                    self.stats.handlers_executed_total += 1;
                }
            }
        }

        // Check triggers
        let trigger_ids: Vec<String> = self.triggers.keys().cloned().collect();
        for trigger_id in trigger_ids {
            if let Some(trigger) = self.triggers.get_mut(&trigger_id) {
                if trigger.enabled && trigger.condition.evaluate(&event, &self.custom_conditions) {
                    match trigger.trigger_type {
                        TriggerType::Once => {
                            if trigger.activation_count == 0 {
                                trigger.action.execute(&event, &mut self.context, &self.custom_actions, &self.custom_conditions)?;
                                trigger.activation_count += 1;
                                trigger.enabled = false;
                                self.stats.triggers_activated_total += 1;
                            }
                        }
                        TriggerType::Immediate => {
                            trigger.action.execute(&event, &mut self.context, &self.custom_actions, &self.custom_conditions)?;
                            trigger.activation_count += 1;
                            self.stats.triggers_activated_total += 1;
                        }
                        TriggerType::Delayed(_) | TriggerType::Interval(_) => {
                            // These would be handled by a separate timer system
                            trigger.activation_count += 1;
                            self.stats.triggers_activated_total += 1;
                        }
                    }
                }
            }
        }

        self.stats.events_processed_total += 1;
        Ok(())
    }

    fn process_delayed_actions(&mut self) -> RobinResult<()> {
        let now = Instant::now();
        let mut actions_to_execute = Vec::new();

        // Find actions ready to execute
        self.context.delayed_actions.retain(|action| {
            if now >= action.execute_at {
                actions_to_execute.push(action.clone());
                false
            } else {
                true
            }
        });

        // Execute ready actions
        for delayed_action in actions_to_execute {
            delayed_action.action.execute(
                &delayed_action.event, 
                &mut self.context, 
                &self.custom_actions, 
                &self.custom_conditions
            )?;
        }

        Ok(())
    }

    fn update_stats(&mut self, events_processed: u32, processing_time_ms: f32) {
        self.stats.events_processed_per_second = events_processed;
        self.stats.average_processing_time_ms = processing_time_ms;
        
        // Update queue sizes
        self.stats.queue_sizes.clear();
        for (priority, queue) in &self.event_queue {
            let priority_name = match priority {
                EventPriority::Critical => "critical",
                EventPriority::High => "high", 
                EventPriority::Normal => "normal",
                EventPriority::Low => "low",
            };
            self.stats.queue_sizes.insert(priority_name.to_string(), queue.len() as u32);
        }
    }

    pub fn enable_handler(&mut self, handler_id: &str) -> RobinResult<()> {
        if let Some(handler) = self.handlers.get_mut(handler_id) {
            handler.enabled = true;
        }
        Ok(())
    }

    pub fn disable_handler(&mut self, handler_id: &str) -> RobinResult<()> {
        if let Some(handler) = self.handlers.get_mut(handler_id) {
            handler.enabled = false;
        }
        Ok(())
    }

    pub fn remove_handler(&mut self, handler_id: &str) -> RobinResult<()> {
        self.handlers.remove(handler_id);
        self.subscriptions.retain(|sub| sub.handler_id != handler_id);
        Ok(())
    }

    pub fn enable_trigger(&mut self, trigger_id: &str) -> RobinResult<()> {
        if let Some(trigger) = self.triggers.get_mut(trigger_id) {
            trigger.enabled = true;
        }
        Ok(())
    }

    pub fn disable_trigger(&mut self, trigger_id: &str) -> RobinResult<()> {
        if let Some(trigger) = self.triggers.get_mut(trigger_id) {
            trigger.enabled = false;
        }
        Ok(())
    }

    pub fn remove_trigger(&mut self, trigger_id: &str) -> RobinResult<()> {
        self.triggers.remove(trigger_id);
        Ok(())
    }

    pub fn clear_all_events(&mut self) {
        for queue in self.event_queue.values_mut() {
            queue.clear();
        }
        self.context.clear();
    }

    pub fn set_processing_enabled(&mut self, enabled: bool) {
        self.processing_enabled = enabled;
    }

    pub fn get_handler_count(&self) -> usize {
        self.handlers.len()
    }

    pub fn get_trigger_count(&self) -> usize {
        self.triggers.len()
    }

    pub fn get_events_processed_per_second(&self) -> u32 {
        self.stats.events_processed_per_second
    }

    pub fn get_stats(&self) -> &EventSystemStats {
        &self.stats
    }

    pub fn get_total_queue_size(&self) -> usize {
        self.event_queue.values().map(|q| q.len()).sum()
    }

    pub fn shutdown(&mut self) -> RobinResult<()> {
        println!("Event System shutdown:");
        println!("  Total events processed: {}", self.stats.events_processed_total);
        println!("  Total handlers executed: {}", self.stats.handlers_executed_total);
        println!("  Total triggers activated: {}", self.stats.triggers_activated_total);
        println!("  Handlers registered: {}", self.handlers.len());
        println!("  Triggers registered: {}", self.triggers.len());
        println!("  Custom conditions: {}", self.custom_conditions.len());
        println!("  Custom actions: {}", self.custom_actions.len());

        self.handlers.clear();
        self.triggers.clear();
        self.subscriptions.clear();
        self.custom_conditions.clear();
        self.custom_actions.clear();
        self.clear_all_events();

        Ok(())
    }
}

// Convenience builders for common event patterns
pub struct EventBuilder {
    event: Event,
}

impl EventBuilder {
    pub fn new(name: String) -> Self {
        Self {
            event: Event::new(name, HashMap::new()),
        }
    }

    pub fn with_data(mut self, key: String, value: RuntimeValue) -> Self {
        self.event.data.insert(key, value);
        self
    }

    pub fn with_priority(mut self, priority: EventPriority) -> Self {
        self.event.priority = priority;
        self
    }

    pub fn with_source(mut self, source: String) -> Self {
        self.event.source = source;
        self
    }

    pub fn build(self) -> Event {
        self.event
    }
}

pub struct HandlerBuilder {
    name: String,
    event_pattern: String,
    condition: EventCondition,
    action: EventAction,
    cooldown_ms: Option<u64>,
}

impl HandlerBuilder {
    pub fn new(name: String, event_pattern: String) -> Self {
        Self {
            name,
            event_pattern,
            condition: EventCondition::Always,
            action: EventAction::LogMessage("Default handler action".to_string()),
            cooldown_ms: None,
        }
    }

    pub fn with_condition(mut self, condition: EventCondition) -> Self {
        self.condition = condition;
        self
    }

    pub fn with_action(mut self, action: EventAction) -> Self {
        self.action = action;
        self
    }

    pub fn with_cooldown(mut self, cooldown_ms: u64) -> Self {
        self.cooldown_ms = Some(cooldown_ms);
        self
    }

    pub fn build(self) -> EventHandler {
        let mut handler = EventHandler::new(self.name, self.event_pattern, self.condition, self.action);
        if let Some(cooldown) = self.cooldown_ms {
            handler = handler.with_cooldown(cooldown);
        }
        handler
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let mut data = HashMap::new();
        data.insert("test_key".to_string(), RuntimeValue::String("test_value".to_string()));
        
        let event = Event::new("test_event".to_string(), data);
        assert_eq!(event.name, "test_event");
        assert!(event.data.contains_key("test_key"));
        assert!(!event.is_propagation_stopped());
    }

    #[test]
    fn test_event_condition_evaluation() {
        let mut data = HashMap::new();
        data.insert("health".to_string(), RuntimeValue::Int(50));
        data.insert("alive".to_string(), RuntimeValue::Bool(true));
        
        let event = Event::new("player_update".to_string(), data);
        let custom_conditions = HashMap::new();
        
        let condition1 = EventCondition::KeyExists("health".to_string());
        assert!(condition1.evaluate(&event, &custom_conditions));
        
        let condition2 = EventCondition::KeyGreater("health".to_string(), 25.0);
        assert!(condition2.evaluate(&event, &custom_conditions));
        
        let condition3 = EventCondition::And(
            Box::new(EventCondition::KeyExists("alive".to_string())),
            Box::new(EventCondition::KeyEquals("alive".to_string(), RuntimeValue::Bool(true)))
        );
        assert!(condition3.evaluate(&event, &custom_conditions));
    }

    #[test]
    fn test_event_system_basic_operations() {
        let mut system = EventSystem::new().unwrap();
        system.initialize().unwrap();
        
        // Register a handler
        let handler_id = system.register_handler(
            "test_*".to_string(),
            EventCondition::Always,
            EventAction::LogMessage("Test handler executed".to_string())
        ).unwrap();
        
        assert_eq!(system.get_handler_count(), 1);
        
        // Trigger an event
        let event = EventBuilder::new("test_event".to_string())
            .with_data("test_key".to_string(), RuntimeValue::String("test_value".to_string()))
            .build();
        
        system.trigger_event(event).unwrap();
        system.update(0.016).unwrap();
        
        // Clean up
        system.remove_handler(&handler_id).unwrap();
        assert_eq!(system.get_handler_count(), 0);
    }

    #[test]
    fn test_event_builder() {
        let event = EventBuilder::new("player_action".to_string())
            .with_data("player_id".to_string(), RuntimeValue::String("player_123".to_string()))
            .with_data("action".to_string(), RuntimeValue::String("jump".to_string()))
            .with_priority(EventPriority::High)
            .with_source("input_system".to_string())
            .build();
        
        assert_eq!(event.name, "player_action");
        assert_eq!(event.priority, EventPriority::High);
        assert_eq!(event.source, "input_system");
        assert!(event.data.contains_key("player_id"));
        assert!(event.data.contains_key("action"));
    }

    #[test]
    fn test_handler_builder() {
        let handler = HandlerBuilder::new("test_handler".to_string(), "player_*".to_string())
            .with_condition(EventCondition::KeyExists("player_id".to_string()))
            .with_action(EventAction::LogMessage("Player event received".to_string()))
            .with_cooldown(1000)
            .build();
        
        assert_eq!(handler.name, "test_handler");
        assert_eq!(handler.event_pattern, "player_*");
        assert_eq!(handler.cooldown_ms, Some(1000));
    }
}