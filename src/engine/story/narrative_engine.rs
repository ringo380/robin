use std::collections::HashMap;
use crate::engine::story::{StoryEvent, WorldState, Character};

#[derive(Debug, Clone)]
pub struct NarrativeEngine {
    plot_threads: HashMap<String, PlotThread>,
    narrative_state: NarrativeState,
    story_beats: Vec<StoryBeat>,
    character_arcs: HashMap<String, CharacterArc>,
    narrative_rules: Vec<NarrativeRule>,
}

#[derive(Debug, Clone)]
pub struct PlotThread {
    pub id: String,
    pub name: String,
    pub current_state: PlotState,
    pub beats: Vec<PlotBeat>,
    pub importance: f32,
    pub tension_level: f32,
    pub active: bool,
}

#[derive(Debug, Clone)]
pub struct PlotBeat {
    pub id: String,
    pub name: String,
    pub description: String,
    pub triggers: Vec<Trigger>,
    pub consequences: Vec<Consequence>,
    pub prerequisites: Vec<String>,
    pub character_involvement: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum PlotState {
    Setup,
    Rising,
    Climax,
    Falling,
    Resolution,
}

#[derive(Debug, Clone)]
pub struct NarrativeState {
    pub overall_tension: f32,
    pub pacing: Pacing,
    pub active_themes: Vec<Theme>,
    pub narrative_momentum: f32,
    pub last_major_event: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Pacing {
    Slow,
    Steady,
    Fast,
    Frantic,
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub weight: f32,
    pub manifestations: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CharacterArc {
    pub character_id: String,
    pub arc_type: ArcType,
    pub current_stage: ArcStage,
    pub key_moments: Vec<String>,
    pub development_goals: Vec<String>,
    pub internal_conflict: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ArcType {
    Hero,
    Antihero,
    Mentor,
    Ally,
    Antagonist,
    Tragic,
    Redemption,
    Growth,
}

#[derive(Debug, Clone)]
pub enum ArcStage {
    Introduction,
    Call,
    Refusal,
    Mentor,
    Crossing,
    Tests,
    Approach,
    Ordeal,
    Reward,
    Road,
    Resurrection,
    Return,
}

#[derive(Debug, Clone)]
pub struct StoryBeat {
    pub timestamp: u64,
    pub beat_type: BeatType,
    pub intensity: f32,
    pub participants: Vec<String>,
    pub location: String,
    pub emotional_impact: EmotionalImpact,
}

#[derive(Debug, Clone)]
pub enum BeatType {
    Dialogue,
    Action,
    Revelation,
    Conflict,
    Resolution,
    Transition,
    Flashback,
    Foreshadowing,
}

#[derive(Debug, Clone)]
pub struct EmotionalImpact {
    pub primary_emotion: Emotion,
    pub intensity: f32,
    pub affected_characters: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum Emotion {
    Joy,
    Sadness,
    Fear,
    Anger,
    Surprise,
    Disgust,
    Anticipation,
    Trust,
}

#[derive(Debug, Clone)]
pub struct Trigger {
    pub condition: TriggerCondition,
    pub probability: f32,
}

#[derive(Debug, Clone)]
pub enum TriggerCondition {
    WorldState(String, String),
    CharacterState(String, String),
    TimeElapsed(u64),
    EventOccurred(String),
    QuestCompleted(String),
    LocationVisited(String),
}

#[derive(Debug, Clone)]
pub struct Consequence {
    pub effect_type: EffectType,
    pub target: String,
    pub magnitude: f32,
}

#[derive(Debug, Clone)]
pub enum EffectType {
    StateChange,
    CharacterDevelopment,
    RelationshipChange,
    WorldModification,
    QuestTrigger,
    DialogueUnlock,
}

#[derive(Debug, Clone)]
pub struct NarrativeRule {
    pub name: String,
    pub condition: RuleCondition,
    pub action: RuleAction,
    pub priority: f32,
}

#[derive(Debug, Clone)]
pub enum RuleCondition {
    TensionThreshold(f32),
    PacingCheck(Pacing),
    CharacterArcStage(String, ArcStage),
    PlotThreadState(String, PlotState),
    TimeSinceLastEvent(u64),
}

#[derive(Debug, Clone)]
pub enum RuleAction {
    AdjustPacing(Pacing),
    TriggerEvent(String),
    ModifyTension(f32),
    AdvanceArc(String),
    IntroducePlotThread(String),
}

impl NarrativeEngine {
    pub fn new() -> Self {
        NarrativeEngine {
            plot_threads: HashMap::new(),
            narrative_state: NarrativeState {
                overall_tension: 0.5,
                pacing: Pacing::Steady,
                active_themes: Vec::new(),
                narrative_momentum: 0.0,
                last_major_event: None,
            },
            story_beats: Vec::new(),
            character_arcs: HashMap::new(),
            narrative_rules: Vec::new(),
        }
    }

    pub fn add_plot_thread(&mut self, thread: PlotThread) {
        self.plot_threads.insert(thread.id.clone(), thread);
    }

    pub fn add_character_arc(&mut self, arc: CharacterArc) {
        self.character_arcs.insert(arc.character_id.clone(), arc);
    }

    pub fn process_story_beat(&mut self, beat: StoryBeat) {
        self.update_narrative_momentum(&beat);
        self.update_tension(&beat);
        self.check_arc_progression(&beat);
        self.story_beats.push(beat);
    }

    pub fn evaluate_triggers(&self, world_state: &WorldState) -> Vec<String> {
        let mut triggered_events = Vec::new();
        
        for thread in self.plot_threads.values() {
            if !thread.active {
                continue;
            }
            
            for beat in &thread.beats {
                for trigger in &beat.triggers {
                    if self.check_trigger(trigger, world_state) {
                        triggered_events.push(beat.id.clone());
                    }
                }
            }
        }
        
        triggered_events
    }

    pub fn apply_narrative_rules(&mut self) {
        let mut actions_to_apply = Vec::new();
        
        for rule in &self.narrative_rules {
            if self.check_rule_condition(&rule.condition) {
                actions_to_apply.push((rule.action.clone(), rule.priority));
            }
        }
        
        actions_to_apply.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        for (action, _) in actions_to_apply {
            self.apply_rule_action(action);
        }
    }

    pub fn get_current_narrative_focus(&self) -> Vec<String> {
        self.plot_threads
            .values()
            .filter(|thread| thread.active && thread.tension_level > 0.7)
            .map(|thread| thread.id.clone())
            .collect()
    }

    pub fn suggest_story_developments(&self, world_state: &WorldState) -> Vec<StoryDevelopment> {
        let mut suggestions = Vec::new();
        
        for thread in self.plot_threads.values() {
            if thread.active && thread.tension_level < 0.3 {
                suggestions.push(StoryDevelopment {
                    thread_id: thread.id.clone(),
                    development_type: DevelopmentType::TensionIncrease,
                    urgency: 0.8,
                    description: format!("Increase tension in plot thread: {}", thread.name),
                });
            }
        }
        
        if self.narrative_state.overall_tension > 0.9 {
            suggestions.push(StoryDevelopment {
                thread_id: String::new(),
                development_type: DevelopmentType::Resolution,
                urgency: 0.9,
                description: "Consider resolving some plot threads to reduce tension".to_string(),
            });
        }
        
        suggestions
    }

    fn update_narrative_momentum(&mut self, beat: &StoryBeat) {
        match beat.beat_type {
            BeatType::Action | BeatType::Conflict => {
                self.narrative_state.narrative_momentum += beat.intensity * 0.3;
            }
            BeatType::Resolution => {
                self.narrative_state.narrative_momentum -= beat.intensity * 0.2;
            }
            _ => {
                self.narrative_state.narrative_momentum += beat.intensity * 0.1;
            }
        }
        
        self.narrative_state.narrative_momentum = self.narrative_state.narrative_momentum.clamp(-1.0, 1.0);
    }

    fn update_tension(&mut self, beat: &StoryBeat) {
        match beat.beat_type {
            BeatType::Conflict | BeatType::Revelation => {
                self.narrative_state.overall_tension += beat.intensity * 0.2;
            }
            BeatType::Resolution => {
                self.narrative_state.overall_tension -= beat.intensity * 0.3;
            }
            _ => {}
        }
        
        self.narrative_state.overall_tension = self.narrative_state.overall_tension.clamp(0.0, 1.0);
    }

    fn check_arc_progression(&mut self, beat: &StoryBeat) {
        for participant in &beat.participants {
            if let Some(arc) = self.character_arcs.get_mut(participant) {
                if beat.intensity > 0.7 {
                    arc.key_moments.push(format!("Beat_{}", beat.timestamp));
                }
            }
        }
    }

    fn check_trigger(&self, trigger: &Trigger, world_state: &WorldState) -> bool {
        match &trigger.condition {
            TriggerCondition::WorldState(key, value) => {
                world_state.variables.get(key) == Some(value)
            }
            TriggerCondition::CharacterState(char_id, state) => {
                world_state.variables.get(&format!("character_{}", char_id)) == Some(state)
            }
            TriggerCondition::TimeElapsed(duration) => {
                world_state.global_variables.get("current_time").unwrap_or(&0.0) >= &(*duration as f32)
            }
            TriggerCondition::EventOccurred(event_id) => {
                world_state.historical_events.iter().any(|e| &e.id == event_id)
            }
            TriggerCondition::QuestCompleted(quest_id) => {
                world_state.completed_storylines.contains(quest_id)
            }
            TriggerCondition::LocationVisited(location) => {
                world_state.variables.get(&format!("visited_{}", location)).is_some()
            }
        }
    }

    fn check_rule_condition(&self, condition: &RuleCondition) -> bool {
        match condition {
            RuleCondition::TensionThreshold(threshold) => {
                self.narrative_state.overall_tension >= *threshold
            }
            RuleCondition::PacingCheck(expected_pacing) => {
                std::mem::discriminant(&self.narrative_state.pacing) == std::mem::discriminant(expected_pacing)
            }
            RuleCondition::CharacterArcStage(char_id, stage) => {
                if let Some(arc) = self.character_arcs.get(char_id) {
                    std::mem::discriminant(&arc.current_stage) == std::mem::discriminant(stage)
                } else {
                    false
                }
            }
            RuleCondition::PlotThreadState(thread_id, state) => {
                if let Some(thread) = self.plot_threads.get(thread_id) {
                    std::mem::discriminant(&thread.current_state) == std::mem::discriminant(state)
                } else {
                    false
                }
            }
            RuleCondition::TimeSinceLastEvent(duration) => {
                true
            }
        }
    }

    fn apply_rule_action(&mut self, action: RuleAction) {
        match action {
            RuleAction::AdjustPacing(new_pacing) => {
                self.narrative_state.pacing = new_pacing;
            }
            RuleAction::TriggerEvent(event_id) => {
                // Would trigger event in event system
            }
            RuleAction::ModifyTension(delta) => {
                self.narrative_state.overall_tension += delta;
                self.narrative_state.overall_tension = self.narrative_state.overall_tension.clamp(0.0, 1.0);
            }
            RuleAction::AdvanceArc(char_id) => {
                if let Some(arc) = self.character_arcs.get_mut(&char_id) {
                    // Advance arc stage logic
                }
            }
            RuleAction::IntroducePlotThread(thread_id) => {
                if let Some(thread) = self.plot_threads.get_mut(&thread_id) {
                    thread.active = true;
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct StoryDevelopment {
    pub thread_id: String,
    pub development_type: DevelopmentType,
    pub urgency: f32,
    pub description: String,
}

#[derive(Debug, Clone)]
pub enum DevelopmentType {
    TensionIncrease,
    TensionDecrease,
    CharacterIntroduction,
    PlotTwist,
    Resolution,
    Complication,
    Revelation,
}