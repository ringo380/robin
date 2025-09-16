use std::collections::{HashMap, VecDeque};
use crate::engine::story::{StoryEvent, WorldState, Character};

#[derive(Debug, Clone)]
pub struct EventSystem {
    event_queue: VecDeque<ScheduledEvent>,
    event_handlers: HashMap<String, Vec<EventHandler>>,
    event_history: Vec<ProcessedEvent>,
    event_templates: HashMap<String, EventTemplate>,
    global_modifiers: Vec<GlobalModifier>,
    event_chains: HashMap<String, EventChain>,
}

#[derive(Debug, Clone)]
pub struct ScheduledEvent {
    pub event: StoryEvent,
    pub scheduled_time: u64,
    pub priority: EventPriority,
    pub prerequisites: Vec<Prerequisite>,
    pub expiry_time: Option<u64>,
}

#[derive(Debug, Clone)]
pub enum EventPriority {
    Critical,
    High,
    Normal,
    Low,
    Background,
}

#[derive(Debug, Clone)]
pub struct Prerequisite {
    pub condition_type: PrerequisiteType,
    pub required_value: String,
    pub check_function: String,
}

#[derive(Debug, Clone)]
pub enum PrerequisiteType {
    WorldState,
    CharacterState,
    QuestStatus,
    RelationshipLevel,
    TimeConstraint,
    LocationRequirement,
    ItemPossession,
}

#[derive(Debug, Clone)]
pub struct EventHandler {
    pub handler_id: String,
    pub event_pattern: EventPattern,
    pub handler_function: HandlerFunction,
    pub execution_order: i32,
    pub conditions: Vec<ExecutionCondition>,
}

#[derive(Debug, Clone)]
pub struct EventPattern {
    pub event_type: Option<String>,
    pub source_pattern: Option<String>,
    pub target_pattern: Option<String>,
    pub data_requirements: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum HandlerFunction {
    StateModification(StateModification),
    QuestTrigger(String),
    CharacterAction(CharacterAction),
    WorldChange(WorldChange),
    DialogueTrigger(DialogueTrigger),
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct StateModification {
    pub target: ModificationTarget,
    pub property: String,
    pub operation: ModificationOperation,
    pub value: String,
}

#[derive(Debug, Clone)]
pub enum ModificationTarget {
    World,
    Character(String),
    Quest(String),
    Relationship(String, String),
}

#[derive(Debug, Clone)]
pub enum ModificationOperation {
    Set,
    Add,
    Subtract,
    Multiply,
    Toggle,
    Append,
}

#[derive(Debug, Clone)]
pub struct CharacterAction {
    pub character_id: String,
    pub action_type: ActionType,
    pub parameters: HashMap<String, String>,
    pub duration: Option<u64>,
}

#[derive(Debug, Clone)]
pub enum ActionType {
    Move,
    Interact,
    Dialogue,
    EmotionalResponse,
    StateChange,
    QuestAction,
}

#[derive(Debug, Clone)]
pub struct WorldChange {
    pub change_type: WorldChangeType,
    pub location: Option<String>,
    pub scope: ChangeScope,
    pub parameters: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum WorldChangeType {
    EnvironmentalChange,
    ObjectSpawn,
    ObjectRemoval,
    LocationModification,
    WeatherChange,
    TimeSkip,
}

#[derive(Debug, Clone)]
pub enum ChangeScope {
    Global,
    Regional,
    Local,
    Personal,
}

#[derive(Debug, Clone)]
pub struct DialogueTrigger {
    pub characters: Vec<String>,
    pub dialogue_set: String,
    pub trigger_conditions: Vec<String>,
    pub auto_start: bool,
}

#[derive(Debug, Clone)]
pub struct ExecutionCondition {
    pub condition_type: ConditionType,
    pub parameter: String,
    pub expected_value: String,
    pub comparison: ComparisonOperator,
}

#[derive(Debug, Clone)]
pub enum ConditionType {
    WorldVariable,
    CharacterProperty,
    QuestState,
    TimeOfDay,
    Weather,
    Location,
}

#[derive(Debug, Clone)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterEqual,
    LessEqual,
    Contains,
    NotContains,
}

#[derive(Debug, Clone)]
pub struct ProcessedEvent {
    pub original_event: StoryEvent,
    pub processed_time: u64,
    pub handlers_executed: Vec<String>,
    pub resulting_changes: Vec<EventResult>,
    pub propagated_events: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct EventResult {
    pub result_type: ResultType,
    pub description: String,
    pub affected_entities: Vec<String>,
    pub magnitude: f32,
}

#[derive(Debug, Clone)]
pub enum ResultType {
    StateChange,
    RelationshipChange,
    QuestProgression,
    WorldModification,
    CharacterDevelopment,
    StoryProgression,
}

#[derive(Debug, Clone)]
pub struct EventTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub default_data: HashMap<String, String>,
    pub required_parameters: Vec<String>,
    pub optional_parameters: Vec<String>,
    pub typical_handlers: Vec<String>,
    pub cooldown_period: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct GlobalModifier {
    pub name: String,
    pub modifier_type: ModifierType,
    pub effect: ModifierEffect,
    pub conditions: Vec<String>,
    pub duration: Option<u64>,
}

#[derive(Debug, Clone)]
pub enum ModifierType {
    EventFrequency,
    EventIntensity,
    HandlerEffectiveness,
    ConsequenceMagnitude,
    ChainProbability,
}

#[derive(Debug, Clone)]
pub struct ModifierEffect {
    pub operation: ModificationOperation,
    pub value: f32,
    pub target_pattern: String,
}

#[derive(Debug, Clone)]
pub struct EventChain {
    pub id: String,
    pub name: String,
    pub triggering_event: String,
    pub chain_steps: Vec<ChainStep>,
    pub chain_conditions: Vec<ChainCondition>,
    pub success_probability: f32,
}

#[derive(Debug, Clone)]
pub struct ChainStep {
    pub step_id: String,
    pub delay: u64,
    pub event_template: String,
    pub modifications: HashMap<String, String>,
    pub success_conditions: Vec<String>,
    pub failure_consequences: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ChainCondition {
    pub condition_name: String,
    pub check_time: ChainCheckTime,
    pub condition_function: String,
    pub required_for_continuation: bool,
}

#[derive(Debug, Clone)]
pub enum ChainCheckTime {
    BeforeChain,
    BeforeStep(usize),
    AfterStep(usize),
    AfterChain,
}

impl EventSystem {
    pub fn new() -> Self {
        EventSystem {
            event_queue: VecDeque::new(),
            event_handlers: HashMap::new(),
            event_history: Vec::new(),
            event_templates: HashMap::new(),
            global_modifiers: Vec::new(),
            event_chains: HashMap::new(),
        }
    }

    pub fn schedule_event(&mut self, event: StoryEvent, delay: u64, priority: EventPriority) {
        let scheduled_event = ScheduledEvent {
            event,
            scheduled_time: delay,
            priority,
            prerequisites: Vec::new(),
            expiry_time: None,
        };
        
        self.insert_event_by_priority(scheduled_event);
    }

    pub fn schedule_event_with_prerequisites(&mut self, event: StoryEvent, delay: u64, priority: EventPriority, prerequisites: Vec<Prerequisite>) {
        let scheduled_event = ScheduledEvent {
            event,
            scheduled_time: delay,
            priority,
            prerequisites,
            expiry_time: None,
        };
        
        self.insert_event_by_priority(scheduled_event);
    }

    pub fn register_event_handler(&mut self, handler: EventHandler) {
        let pattern_key = self.create_pattern_key(&handler.event_pattern);
        self.event_handlers
            .entry(pattern_key)
            .or_insert_with(Vec::new)
            .push(handler);
    }

    pub fn process_events(&mut self, current_time: u64, world_state: &mut WorldState) -> Vec<ProcessedEvent> {
        let mut processed_events = Vec::new();
        
        while let Some(scheduled_event) = self.event_queue.front() {
            if scheduled_event.scheduled_time > current_time {
                break;
            }
            
            if let Some(expiry) = scheduled_event.expiry_time {
                if current_time > expiry {
                    self.event_queue.pop_front();
                    continue;
                }
            }
            
            let event = self.event_queue.pop_front().unwrap();
            
            if self.check_prerequisites(&event.prerequisites, world_state) {
                let processed = self.process_single_event(event.event, current_time, world_state);
                processed_events.push(processed);
            }
        }
        
        processed_events
    }

    pub fn trigger_immediate_event(&mut self, event: StoryEvent, world_state: &mut WorldState) -> ProcessedEvent {
        self.process_single_event(event, 0, world_state)
    }

    pub fn add_global_modifier(&mut self, modifier: GlobalModifier) {
        self.global_modifiers.push(modifier);
    }

    pub fn remove_global_modifier(&mut self, modifier_name: &str) {
        self.global_modifiers.retain(|m| m.name != modifier_name);
    }

    pub fn register_event_chain(&mut self, chain: EventChain) {
        self.event_chains.insert(chain.id.clone(), chain);
    }

    pub fn get_event_history(&self, filter: Option<EventFilter>) -> Vec<&ProcessedEvent> {
        match filter {
            Some(f) => self.event_history.iter().filter(|e| self.matches_filter(e, &f)).collect(),
            None => self.event_history.iter().collect(),
        }
    }

    pub fn get_upcoming_events(&self, time_window: u64, current_time: u64) -> Vec<&ScheduledEvent> {
        self.event_queue
            .iter()
            .filter(|e| e.scheduled_time <= current_time + time_window)
            .collect()
    }

    pub fn cancel_events(&mut self, pattern: &EventPattern) {
        let _pattern_key = self.create_pattern_key(pattern);
        // Create a copy of pattern data to avoid borrowing issues
        let pattern_clone = pattern.clone();
        self.event_queue.retain(|event| {
            !Self::event_matches_pattern_static(&event.event, &pattern_clone)
        });
    }

    pub fn modify_scheduled_event(&mut self, event_id: &str, modification: EventModification) {
        let event_id = event_id.to_string(); // Clone to avoid borrowing issues
        let mut found_event = None;

        // Find the event index
        for (i, event) in self.event_queue.iter().enumerate() {
            if event.event.id == event_id {
                found_event = Some(i);
                break;
            }
        }

        // Apply modification if event found
        if let Some(index) = found_event {
            if let Some(event) = self.event_queue.get_mut(index) {
                Self::apply_event_modification_static(event, modification);
            }
        }
    }

    fn insert_event_by_priority(&mut self, event: ScheduledEvent) {
        let position = self.event_queue
            .iter()
            .position(|e| {
                e.scheduled_time > event.scheduled_time || 
                (e.scheduled_time == event.scheduled_time && self.priority_value(&e.priority) < self.priority_value(&event.priority))
            })
            .unwrap_or(self.event_queue.len());
            
        self.event_queue.insert(position, event);
    }

    fn priority_value(&self, priority: &EventPriority) -> u8 {
        match priority {
            EventPriority::Critical => 5,
            EventPriority::High => 4,
            EventPriority::Normal => 3,
            EventPriority::Low => 2,
            EventPriority::Background => 1,
        }
    }

    fn process_single_event(&mut self, event: StoryEvent, current_time: u64, world_state: &mut WorldState) -> ProcessedEvent {
        let mut handlers_executed = Vec::new();
        let mut resulting_changes = Vec::new();
        let mut propagated_events = Vec::new();
        
        let applicable_handlers = self.find_applicable_handlers(&event);
        
        for handler in applicable_handlers {
            if self.check_execution_conditions(&handler.conditions, world_state) {
                let results = self.execute_handler(&handler, &event, world_state);
                handlers_executed.push(handler.handler_id.clone());
                resulting_changes.extend(results.0);
                propagated_events.extend(results.1);
            }
        }
        
        self.check_and_trigger_chains(&event, world_state);
        
        let processed = ProcessedEvent {
            original_event: event,
            processed_time: current_time,
            handlers_executed,
            resulting_changes,
            propagated_events,
        };
        
        self.event_history.push(processed.clone());
        processed
    }

    fn check_prerequisites(&self, prerequisites: &[Prerequisite], world_state: &WorldState) -> bool {
        prerequisites.iter().all(|prereq| self.check_prerequisite(prereq, world_state))
    }

    fn check_prerequisite(&self, prerequisite: &Prerequisite, world_state: &WorldState) -> bool {
        match prerequisite.condition_type {
            PrerequisiteType::WorldState => {
                world_state.variables.get(&prerequisite.check_function) == Some(&prerequisite.required_value)
            }
            PrerequisiteType::CharacterState => {
                world_state.variables.get(&format!("character_{}", prerequisite.check_function)) == Some(&prerequisite.required_value)
            }
            PrerequisiteType::QuestStatus => {
                world_state.completed_storylines.contains(&prerequisite.check_function) == (prerequisite.required_value == "completed")
            }
            _ => true, // Simplified for other types
        }
    }

    fn find_applicable_handlers(&self, event: &StoryEvent) -> Vec<&EventHandler> {
        let mut applicable = Vec::new();
        
        for handlers in self.event_handlers.values() {
            for handler in handlers {
                if self.event_matches_pattern(event, &handler.event_pattern) {
                    applicable.push(handler);
                }
            }
        }
        
        applicable.sort_by_key(|h| h.execution_order);
        applicable
    }

    fn check_execution_conditions(&self, conditions: &[ExecutionCondition], world_state: &WorldState) -> bool {
        conditions.iter().all(|condition| self.evaluate_condition(condition, world_state))
    }

    fn evaluate_condition(&self, condition: &ExecutionCondition, world_state: &WorldState) -> bool {
        let actual_value = match condition.condition_type {
            ConditionType::WorldVariable => {
                world_state.variables.get(&condition.parameter).cloned().unwrap_or_default()
            }
            ConditionType::CharacterProperty => {
                world_state.variables.get(&format!("character_{}", condition.parameter)).cloned().unwrap_or_default()
            }
            _ => String::new(),
        };
        
        self.compare_values(&actual_value, &condition.expected_value, &condition.comparison)
    }

    fn compare_values(&self, actual: &str, expected: &str, op: &ComparisonOperator) -> bool {
        match op {
            ComparisonOperator::Equal => actual == expected,
            ComparisonOperator::NotEqual => actual != expected,
            ComparisonOperator::Contains => actual.contains(expected),
            ComparisonOperator::NotContains => !actual.contains(expected),
            _ => false, // Simplified for numeric comparisons
        }
    }

    fn execute_handler(&self, handler: &EventHandler, event: &StoryEvent, world_state: &mut WorldState) -> (Vec<EventResult>, Vec<String>) {
        let mut results = Vec::new();
        let mut propagated = Vec::new();
        
        match &handler.handler_function {
            HandlerFunction::StateModification(modification) => {
                let result = self.apply_state_modification(modification, world_state);
                results.push(result);
            }
            HandlerFunction::QuestTrigger(quest_id) => {
                propagated.push(format!("quest_trigger_{}", quest_id));
            }
            HandlerFunction::CharacterAction(action) => {
                let result = EventResult {
                    result_type: ResultType::CharacterDevelopment,
                    description: format!("Character {} performed action", action.character_id),
                    affected_entities: vec![action.character_id.clone()],
                    magnitude: 1.0,
                };
                results.push(result);
            }
            _ => {} // Simplified for other handler types
        }
        
        (results, propagated)
    }

    fn apply_state_modification(&self, modification: &StateModification, world_state: &mut WorldState) -> EventResult {
        match &modification.target {
            ModificationTarget::World => {
                match modification.operation {
                    ModificationOperation::Set => {
                        world_state.variables.insert(modification.property.clone(), modification.value.clone());
                    }
                    _ => {} // Other operations
                }
            }
            _ => {} // Other targets
        }
        
        EventResult {
            result_type: ResultType::StateChange,
            description: format!("Modified {} to {}", modification.property, modification.value),
            affected_entities: vec!["world".to_string()],
            magnitude: 1.0,
        }
    }

    fn check_and_trigger_chains(&mut self, event: &StoryEvent, world_state: &WorldState) {
        let chains_to_trigger: Vec<_> = self.event_chains
            .values()
            .filter(|chain| chain.triggering_event == event.event_type)
            .cloned()
            .collect();
            
        for chain in chains_to_trigger {
            if self.evaluate_chain_conditions(&chain.chain_conditions, world_state) {
                self.initiate_event_chain(&chain, world_state);
            }
        }
    }

    fn evaluate_chain_conditions(&self, conditions: &[ChainCondition], world_state: &WorldState) -> bool {
        conditions.iter().all(|condition| {
            match condition.check_time {
                ChainCheckTime::BeforeChain => true, // Simplified
                _ => true,
            }
        })
    }

    fn initiate_event_chain(&mut self, chain: &EventChain, world_state: &WorldState) {
        for (index, step) in chain.chain_steps.iter().enumerate() {
            if let Some(template) = self.event_templates.get(&step.event_template) {
                let mut event_data = template.default_data.clone();
                for (key, value) in &step.modifications {
                    event_data.insert(key.clone(), value.clone());
                }
                
                let chained_event = StoryEvent {
                    id: format!("{}_{}", chain.id, step.step_id),
                    event_type: step.event_template.clone(),
                    source: "event_chain".to_string(),
                    target: String::new(),
                    data: event_data,
                    timestamp: std::time::SystemTime::now(),
                };
                
                self.schedule_event(chained_event, step.delay, EventPriority::Normal);
            }
        }
    }

    fn create_pattern_key(&self, pattern: &EventPattern) -> String {
        format!("{}_{}_{}",
            pattern.event_type.as_ref().unwrap_or(&"*".to_string()),
            pattern.source_pattern.as_ref().unwrap_or(&"*".to_string()),
            pattern.target_pattern.as_ref().unwrap_or(&"*".to_string())
        )
    }

    fn event_matches_pattern(&self, event: &StoryEvent, pattern: &EventPattern) -> bool {
        if let Some(ref event_type) = pattern.event_type {
            if event.event_type != *event_type {
                return false;
            }
        }
        
        if let Some(ref source_pattern) = pattern.source_pattern {
            if !event.source.contains(source_pattern) {
                return false;
            }
        }
        
        if let Some(ref target_pattern) = pattern.target_pattern {
            if !event.target.contains(target_pattern) {
                return false;
            }
        }
        
        true
    }

    fn matches_filter(&self, event: &ProcessedEvent, filter: &EventFilter) -> bool {
        // Implementation for filtering events
        true
    }

    fn apply_event_modification(&mut self, event: &mut ScheduledEvent, modification: EventModification) {
        // Implementation for modifying scheduled events
    }

    // Static helper method to avoid borrowing issues
    fn event_matches_pattern_static(event: &StoryEvent, pattern: &EventPattern) -> bool {
        // Simple pattern matching implementation
        if let Some(ref event_type) = pattern.event_type {
            if &event.event_type != event_type {
                return false;
            }
        }
        // Add more pattern matching logic as needed
        true
    }

    // Static version of apply_event_modification
    fn apply_event_modification_static(event: &mut ScheduledEvent, modification: EventModification) {
        match modification {
            EventModification::RescheduleTime(new_time) => {
                event.scheduled_time = new_time;
            }
            EventModification::ChangePriority(new_priority) => {
                event.priority = new_priority;
            }
            EventModification::ModifyData(new_data) => {
                // Merge or replace data as needed
                for (key, value) in new_data {
                    event.event.data.insert(key, value);
                }
            }
            EventModification::AddPrerequisite(prerequisite) => {
                event.prerequisites.push(prerequisite);
            }
        }
    }
}

#[derive(Debug)]
pub struct EventFilter {
    pub event_type: Option<String>,
    pub time_range: Option<(u64, u64)>,
    pub source_filter: Option<String>,
    pub target_filter: Option<String>,
}

#[derive(Debug)]
pub enum EventModification {
    RescheduleTime(u64),
    ChangePriority(EventPriority),
    ModifyData(HashMap<String, String>),
    AddPrerequisite(Prerequisite),
}