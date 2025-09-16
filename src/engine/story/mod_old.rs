use crate::engine::math::{Vec3, Point3};
use crate::engine::npc::NPC;
use std::collections::HashMap;

pub mod narrative_engine;
pub mod quest_system;
pub mod story_generator;
pub mod event_system;
pub mod dialogue_system;

pub use narrative_engine::NarrativeEngine;
pub use quest_system::QuestSystem;
pub use story_generator::StoryGenerator;
pub use event_system::EventSystem;
pub use dialogue_system::DialogueSystem;

// Additional types needed by the story system components
#[derive(Debug, Clone)]
pub struct StoryEvent {
    pub id: String,
    pub event_type: String,
    pub source: String,
    pub target: String,
    pub data: HashMap<String, String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct WorldState {
    pub variables: HashMap<String, String>,
    pub character_states: HashMap<String, String>,
    pub quest_states: HashMap<String, String>,
    pub current_time: u64,
    pub occurred_events: Vec<String>,
    pub completed_quests: Vec<String>,
    pub visited_locations: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Character {
    pub id: String,
    pub name: String,
    pub position: Point3,
    pub personality: HashMap<String, f32>,
    pub relationships: HashMap<String, f32>,
}

#[derive(Debug, Clone)]
pub struct StoryManager {
    pub narrative_engine: NarrativeEngine,
    pub quest_system: QuestSystem,
    pub story_generator: StoryGenerator,
    pub event_system: EventSystem,
    pub dialogue_system: DialogueSystem,
    
    // Active story elements
    pub active_storylines: HashMap<String, Storyline>,
    pub world_state: WorldState,
    pub story_history: Vec<StoryEvent>,
    
    // Configuration
    pub story_settings: StorySettings,
    pub narrative_style: NarrativeStyle,
}

#[derive(Debug, Clone)]
pub struct Storyline {
    pub id: String,
    pub title: String,
    pub description: String,
    pub storyline_type: StorylineType,
    pub current_act: u32,
    pub acts: Vec<StoryAct>,
    pub participants: Vec<String>, // NPC IDs
    pub locations: Vec<Point3>,
    pub themes: Vec<String>,
    pub tone: StoryTone,
    pub progression: f32, // 0.0 to 1.0
    pub branching_paths: Vec<BranchingPath>,
    pub dependencies: Vec<String>, // Other storyline IDs
    pub consequences: Vec<Consequence>,
}

#[derive(Debug, Clone)]
pub enum StorylineType {
    MainQuest,
    SideQuest,
    PersonalStory,
    CommunityEvent,
    Mystery,
    Romance,
    Adventure,
    Tragedy,
    Comedy,
    Epic,
    SliceOfLife,
}

#[derive(Debug, Clone)]
pub enum StoryTone {
    Heroic,
    Dark,
    Lighthearted,
    Mysterious,
    Dramatic,
    Comedic,
    Romantic,
    Suspenseful,
    Melancholic,
    Inspirational,
}

#[derive(Debug, Clone)]
pub struct StoryAct {
    pub act_number: u32,
    pub title: String,
    pub objectives: Vec<Objective>,
    pub scenes: Vec<Scene>,
    pub climax_threshold: f32,
    pub resolution_conditions: Vec<String>,
    pub character_development: HashMap<String, CharacterArc>,
}

#[derive(Debug, Clone)]
pub struct Objective {
    pub id: String,
    pub description: String,
    pub objective_type: ObjectiveType,
    pub target: ObjectiveTarget,
    pub completion_criteria: Vec<CompletionCriterion>,
    pub optional: bool,
    pub hidden: bool,
    pub time_limit: Option<u32>,
    pub rewards: Vec<Reward>,
    pub failure_consequences: Vec<Consequence>,
}

#[derive(Debug, Clone)]
pub enum ObjectiveType {
    Kill,
    Collect,
    Deliver,
    Build,
    Explore,
    Talk,
    Escort,
    Defend,
    Solve,
    Learn,
    Survive,
    Unite,
    Discover,
}

#[derive(Debug, Clone)]
pub enum ObjectiveTarget {
    NPC(String),
    Location(Point3),
    Object(String),
    Concept(String),
    Multiple(Vec<ObjectiveTarget>),
}

#[derive(Debug, Clone)]
pub enum CompletionCriterion {
    Quantity(u32),
    Quality(f32),
    TimeLimit(u32),
    Condition(String),
    Relationship(String, f32), // NPC ID, required relationship level
    Skill(String, f32), // Skill name, required level
}

#[derive(Debug, Clone)]
pub struct Scene {
    pub id: String,
    pub title: String,
    pub setting: SceneSetting,
    pub participants: Vec<String>,
    pub dialogue_sequences: Vec<String>, // Dialogue IDs
    pub actions: Vec<SceneAction>,
    pub emotional_beats: Vec<EmotionalBeat>,
    pub plot_reveals: Vec<PlotReveal>,
    pub branching_points: Vec<BranchingPoint>,
}

#[derive(Debug, Clone)]
pub struct SceneSetting {
    pub location: Point3,
    pub time_of_day: u32,
    pub weather: String,
    pub atmosphere: String,
    pub props: Vec<String>,
    pub lighting: String,
    pub ambient_sounds: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum SceneAction {
    Movement(String, Point3), // Character ID, destination
    Interaction(String, String), // Character ID, object/character
    Animation(String, String), // Character ID, animation
    Effect(String, Point3), // Effect type, location
    CameraMovement(Point3, Point3), // From, to
    Dialogue(String, String), // Speaker ID, dialogue line
    Choice(Vec<String>), // Available choices
}

#[derive(Debug, Clone)]
pub struct EmotionalBeat {
    pub character_id: String,
    pub emotion: String,
    pub intensity: f32,
    pub duration: f32,
    pub triggers: Vec<String>,
    pub expression_methods: Vec<String>, // dialogue, animation, etc.
}

#[derive(Debug, Clone)]
pub struct PlotReveal {
    pub reveal_type: RevealType,
    pub information: String,
    pub impact_level: f32,
    pub affected_characters: Vec<String>,
    pub consequences: Vec<Consequence>,
}

#[derive(Debug, Clone)]
pub enum RevealType {
    CharacterSecret,
    WorldTruth,
    HistoricalEvent,
    RelationshipTruth,
    MacGuffin,
    RedHerring,
    PlotTwist,
    Foreshadowing,
}

#[derive(Debug, Clone)]
pub struct BranchingPoint {
    pub id: String,
    pub description: String,
    pub choices: Vec<StoryChoice>,
    pub trigger_conditions: Vec<String>,
    pub immediate_consequences: Vec<Consequence>,
}

#[derive(Debug, Clone)]
pub struct StoryChoice {
    pub id: String,
    pub text: String,
    pub requirements: Vec<ChoiceRequirement>,
    pub immediate_effects: Vec<Effect>,
    pub long_term_consequences: Vec<Consequence>,
    pub personality_alignment: HashMap<String, f32>, // Personality trait -> alignment strength
}

#[derive(Debug, Clone)]
pub enum ChoiceRequirement {
    SkillLevel(String, f32),
    RelationshipLevel(String, f32),
    PersonalityTrait(String, f32),
    ItemPossession(String),
    KnowledgeOfFact(String),
    PreviousChoice(String),
}

#[derive(Debug, Clone)]
pub struct BranchingPath {
    pub path_id: String,
    pub trigger_choice: String,
    pub alternative_scenes: Vec<Scene>,
    pub path_convergence: Option<String>, // Scene ID where paths reconverge
    pub exclusive_consequences: Vec<Consequence>,
}

#[derive(Debug, Clone)]
pub struct CharacterArc {
    pub character_id: String,
    pub arc_type: ArcType,
    pub starting_state: CharacterState,
    pub target_state: CharacterState,
    pub development_milestones: Vec<DevelopmentMilestone>,
    pub internal_conflict: Option<String>,
    pub relationships_affected: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum ArcType {
    Growth,
    Fall,
    Redemption,
    Corruption,
    Discovery,
    Transformation,
    Steadfast,
    Awakening,
}

#[derive(Debug, Clone)]
pub struct CharacterState {
    pub beliefs: Vec<String>,
    pub goals: Vec<String>,
    pub fears: Vec<String>,
    pub strengths: Vec<String>,
    pub flaws: Vec<String>,
    pub relationships: HashMap<String, f32>,
    pub emotional_state: String,
}

#[derive(Debug, Clone)]
pub struct DevelopmentMilestone {
    pub milestone_id: String,
    pub description: String,
    pub trigger_events: Vec<String>,
    pub character_changes: Vec<CharacterChange>,
    pub required_progress: f32,
}

#[derive(Debug, Clone)]
pub enum CharacterChange {
    BeliefShift(String, String), // From belief, to belief
    GoalChange(String, String), // From goal, to goal
    PersonalityAdjustment(String, f32), // Trait, change amount
    RelationshipChange(String, f32), // Character ID, change amount
    SkillGain(String, f32),
    MemoryImplant(String),
    EmotionalTransformation(String, String), // From state, to state
}

#[derive(Debug, Clone)]
pub struct StoryEvent {
    pub event_id: String,
    pub event_type: StoryEventType,
    pub timestamp: u64,
    pub location: Point3,
    pub participants: Vec<String>,
    pub description: String,
    pub impact_scope: ImpactScope,
    pub emotional_resonance: f32,
    pub world_state_changes: Vec<WorldStateChange>,
}

#[derive(Debug, Clone)]
pub enum StoryEventType {
    QuestBegin,
    QuestComplete,
    QuestFail,
    CharacterMeeting,
    CharacterDeath,
    RomanceBegin,
    RomanceEnd,
    Betrayal,
    Discovery,
    Victory,
    Defeat,
    Sacrifice,
    Revelation,
    Transformation,
    CommunityEvent,
}

#[derive(Debug, Clone)]
pub enum ImpactScope {
    Personal,
    Local,
    Regional,
    Global,
    Cosmic,
}

#[derive(Debug, Clone)]
pub struct WorldState {
    pub global_variables: HashMap<String, f32>,
    pub faction_standings: HashMap<String, f32>,
    pub world_reputation: HashMap<String, f32>,
    pub completed_storylines: Vec<String>,
    pub active_conflicts: Vec<Conflict>,
    pub historical_events: Vec<HistoricalEvent>,
    pub known_secrets: Vec<String>,
    pub world_mood: f32,
    pub technological_progress: f32,
    pub environmental_state: HashMap<String, f32>,
}

#[derive(Debug, Clone)]
pub struct Conflict {
    pub conflict_id: String,
    pub conflict_type: ConflictType,
    pub involved_parties: Vec<String>,
    pub cause: String,
    pub intensity: f32,
    pub resolution_paths: Vec<ResolutionPath>,
    pub escalation_triggers: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum ConflictType {
    Personal,
    Ideological,
    Resource,
    Territorial,
    Cultural,
    Religious,
    Political,
    Economic,
}

#[derive(Debug, Clone)]
pub struct ResolutionPath {
    pub path_name: String,
    pub requirements: Vec<String>,
    pub mediators: Vec<String>,
    pub outcomes: Vec<Consequence>,
    pub likelihood: f32,
}

#[derive(Debug, Clone)]
pub struct HistoricalEvent {
    pub event_id: String,
    pub name: String,
    pub date: u64,
    pub description: String,
    pub participants: Vec<String>,
    pub consequences: Vec<Consequence>,
    pub cultural_impact: f32,
    pub remembrance_level: f32, // How well it's remembered
}

#[derive(Debug, Clone)]
pub struct Consequence {
    pub consequence_id: String,
    pub consequence_type: ConsequenceType,
    pub target: ConsequenceTarget,
    pub magnitude: f32,
    pub duration: ConsequenceDuration,
    pub description: String,
    pub trigger_delay: u32, // Time before consequence takes effect
}

#[derive(Debug, Clone)]
pub enum ConsequenceType {
    RelationshipChange,
    ReputationChange,
    WorldStateChange,
    CharacterDevelopment,
    StorylineUnlock,
    StorylineBlock,
    ItemGain,
    ItemLoss,
    SkillGain,
    MemoryImplant,
    LocationUnlock,
    NPCBehaviorChange,
}

#[derive(Debug, Clone)]
pub enum ConsequenceTarget {
    Player,
    NPC(String),
    Community(String),
    World,
    Storyline(String),
}

#[derive(Debug, Clone)]
pub enum ConsequenceDuration {
    Instant,
    Temporary(u32), // Duration in game time
    Permanent,
    UntilCondition(String),
}

#[derive(Debug, Clone)]
pub struct Reward {
    pub reward_type: RewardType,
    pub value: f32,
    pub description: String,
    pub rarity: RewardRarity,
}

#[derive(Debug, Clone)]
pub enum RewardType {
    Experience,
    Skill(String),
    Item(String),
    Relationship(String),
    Reputation(String),
    WorldInfluence,
    Knowledge(String),
    Access(String),
}

#[derive(Debug, Clone)]
pub enum RewardRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
    Unique,
}

#[derive(Debug, Clone)]
pub enum Effect {
    Immediate(String),
    Delayed(String, u32),
    Conditional(String, String), // Effect, condition
    Cumulative(String, f32), // Effect, accumulation rate
}

#[derive(Debug, Clone)]
pub enum WorldStateChange {
    VariableChange(String, f32), // Variable name, change amount
    FactionStandingChange(String, f32),
    ReputationChange(String, f32),
    ConflictStateChange(String, f32),
    EnvironmentalChange(String, f32),
    TechnologicalAdvancement(String, f32),
}

#[derive(Debug, Clone)]
pub struct StorySettings {
    pub narrative_complexity: f32, // 0.0 to 1.0
    pub branching_frequency: f32,
    pub character_development_rate: f32,
    pub emotional_intensity: f32,
    pub moral_ambiguity: f32,
    pub comedy_level: f32,
    pub romance_likelihood: f32,
    pub tragedy_tolerance: f32,
    pub mystery_elements: f32,
    pub epic_scale_preference: f32,
}

#[derive(Debug, Clone)]
pub enum NarrativeStyle {
    Linear,
    Branching,
    OpenWorld,
    Episodic,
    Emergent,
    PlayerDriven,
    CharacterDriven,
    EventDriven,
    ThemeDriven,
}

impl StoryManager {
    pub fn new() -> Self {
        Self {
            narrative_engine: NarrativeEngine::new(),
            quest_system: QuestSystem::new(),
            story_generator: StoryGenerator::new(),
            event_system: EventSystem::new(),
            dialogue_system: DialogueSystem::new(),
            
            active_storylines: HashMap::new(),
            world_state: WorldState::default(),
            story_history: Vec::new(),
            
            story_settings: StorySettings::default(),
            narrative_style: NarrativeStyle::Emergent,
        }
    }

    pub fn update(&mut self, delta_time: f32, npcs: &HashMap<String, NPC>, world_time: u32) {
        // Update narrative engine
        self.narrative_engine.apply_narrative_rules();
        
        // Update quest system
        self.quest_system.update_quests(delta_time);
        
        // Process story events
        let events = self.event_system.process_events(world_time as u64, &mut self.world_state);
        for event in events {
            self.story_history.push(StoryEvent {
                id: event.original_event.id,
                event_type: event.original_event.event_type,
                source: event.original_event.source,
                target: event.original_event.target,
                data: event.original_event.data,
                timestamp: event.processed_time,
            });
        }
        
        // Process active storylines
        self.update_storylines(delta_time, npcs);
        
        // Generate new story content
        self.generate_emergent_stories(npcs, world_time);
        
        // Update world state based on story events
        self.update_world_state(delta_time);
    }

    fn update_storylines(&mut self, delta_time: f32, npcs: &HashMap<String, NPC>) {
        let storyline_ids: Vec<_> = self.active_storylines.keys().cloned().collect();
        
        for storyline_id in storyline_ids {
            if let Some(storyline) = self.active_storylines.get_mut(&storyline_id) {
                // Update storyline progression
                self.advance_storyline(storyline, delta_time, npcs);
                
                // Check for completion
                if storyline.progression >= 1.0 {
                    self.complete_storyline(storyline_id.clone());
                }
            }
        }
    }

    fn advance_storyline(&mut self, storyline: &mut Storyline, delta_time: f32, npcs: &HashMap<String, NPC>) {
        let current_act_index = (storyline.current_act - 1) as usize;
        
        if let Some(current_act) = storyline.acts.get_mut(current_act_index) {
            // Check objective completion
            let mut objectives_completed = 0;
            let total_objectives = current_act.objectives.len();
            
            for objective in &mut current_act.objectives {
                if self.is_objective_completed(objective, npcs) {
                    objectives_completed += 1;
                }
            }
            
            // Calculate act progression
            let act_progress = if total_objectives > 0 {
                objectives_completed as f32 / total_objectives as f32
            } else {
                1.0
            };
            
            // Update character arcs
            for (character_id, arc) in &mut current_act.character_development {
                self.update_character_arc(arc, delta_time, npcs);
            }
            
            // Check if act is complete
            if act_progress >= 1.0 {
                if storyline.current_act < storyline.acts.len() as u32 {
                    storyline.current_act += 1;
                    self.trigger_act_transition(storyline);
                } else {
                    storyline.progression = 1.0;
                }
            } else {
                // Update overall storyline progression
                let acts_progress = (storyline.current_act - 1) as f32 + act_progress;
                storyline.progression = acts_progress / storyline.acts.len() as f32;
            }
        }
    }

    fn is_objective_completed(&self, objective: &Objective, npcs: &HashMap<String, NPC>) -> bool {
        // Simplified objective completion check
        match objective.objective_type {
            ObjectiveType::Talk => {
                // Check if required conversations have happened
                if let ObjectiveTarget::NPC(npc_id) = &objective.target {
                    // Would check dialogue system for completed conversations
                    return self.has_conversation_occurred("player", npc_id);
                }
            },
            ObjectiveType::Build => {
                // Check if construction is complete
                // Would integrate with construction system
                return true; // Simplified
            },
            ObjectiveType::Explore => {
                // Check if location has been visited
                if let ObjectiveTarget::Location(location) = &objective.target {
                    // Would check player's exploration history
                    return true; // Simplified
                }
            },
            _ => {
                // Other objective types would be checked similarly
                return false;
            }
        }
        false
    }

    fn update_character_arc(&self, arc: &mut CharacterArc, delta_time: f32, npcs: &HashMap<String, NPC>) {
        if let Some(npc) = npcs.get(&arc.character_id) {
            // Check for milestone triggers
            for milestone in &mut arc.development_milestones {
                if milestone.required_progress <= arc.starting_state.emotional_state.len() as f32 {
                    // Apply character changes
                    for change in &milestone.character_changes {
                        self.apply_character_change(change, &arc.character_id, npcs);
                    }
                }
            }
        }
    }

    fn apply_character_change(&self, change: &CharacterChange, character_id: &str, npcs: &HashMap<String, NPC>) {
        // Apply character development changes
        match change {
            CharacterChange::PersonalityAdjustment(trait_name, adjustment) => {
                // Would modify NPC personality traits
                println!("Applying personality adjustment to {}: {} by {}", character_id, trait_name, adjustment);
            },
            CharacterChange::RelationshipChange(other_id, change_amount) => {
                // Would modify NPC relationships
                println!("Changing relationship between {} and {} by {}", character_id, other_id, change_amount);
            },
            CharacterChange::SkillGain(skill, amount) => {
                // Would add skills to NPC
                println!("Adding {} skill to {} (+{})", skill, character_id, amount);
            },
            _ => {
                // Handle other character changes
            }
        }
    }

    fn trigger_act_transition(&mut self, storyline: &mut Storyline) {
        // Create transition event
        let transition_event = StoryEvent {
            event_id: format!("{}_act_{}_transition", storyline.id, storyline.current_act),
            event_type: StoryEventType::Discovery,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            location: Point3::new(0.0, 0.0, 0.0), // Would use appropriate location
            participants: storyline.participants.clone(),
            description: format!("Transitioning to Act {} of {}", storyline.current_act, storyline.title),
            impact_scope: ImpactScope::Local,
            emotional_resonance: 0.7,
            world_state_changes: vec![],
        };

        self.story_history.push(transition_event);
    }

    fn complete_storyline(&mut self, storyline_id: String) {
        if let Some(storyline) = self.active_storylines.remove(&storyline_id) {
            // Add to completed storylines
            self.world_state.completed_storylines.push(storyline_id.clone());
            
            // Apply final consequences
            for consequence in &storyline.consequences {
                self.apply_consequence(consequence);
            }
            
            // Create completion event
            let completion_event = StoryEvent {
                event_id: format!("{}_complete", storyline_id),
                event_type: StoryEventType::QuestComplete,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                location: Point3::new(0.0, 0.0, 0.0),
                participants: storyline.participants,
                description: format!("Completed storyline: {}", storyline.title),
                impact_scope: match storyline.storyline_type {
                    StorylineType::MainQuest => ImpactScope::Global,
                    StorylineType::CommunityEvent => ImpactScope::Regional,
                    _ => ImpactScope::Local,
                },
                emotional_resonance: 0.9,
                world_state_changes: vec![],
            };

            self.story_history.push(completion_event);
        }
    }

    fn apply_consequence(&mut self, consequence: &Consequence) {
        match consequence.consequence_type {
            ConsequenceType::WorldStateChange => {
                // Apply world state changes
                self.world_state.world_mood += consequence.magnitude * 0.1;
            },
            ConsequenceType::ReputationChange => {
                if let ConsequenceTarget::Community(community) = &consequence.target {
                    let current_rep = self.world_state.world_reputation
                        .get(community).unwrap_or(&0.0);
                    self.world_state.world_reputation
                        .insert(community.clone(), current_rep + consequence.magnitude);
                }
            },
            ConsequenceType::StorylineUnlock => {
                // Would unlock new storylines
                println!("Unlocking new storyline due to consequence: {}", consequence.description);
            },
            _ => {
                // Handle other consequence types
            }
        }
    }

    fn generate_emergent_stories(&mut self, npcs: &HashMap<String, NPC>, world_time: u32) {
        // Generate story content based on NPC interactions and world state
        let story_opportunities = self.identify_story_opportunities(npcs);
        
        for opportunity in story_opportunities {
            if self.should_create_story_from_opportunity(&opportunity) {
                let new_storyline = self.create_storyline_from_opportunity(opportunity);
                self.active_storylines.insert(new_storyline.id.clone(), new_storyline);
            }
        }
    }

    fn should_create_story_from_opportunity(&self, opportunity: &StoryOpportunity) -> bool {
        // Limit number of active storylines
        if self.active_storylines.len() >= 5 {
            return false;
        }
        
        // Check story generation settings
        if opportunity.story_potential < self.story_settings.narrative_complexity {
            return false;
        }
        
        true
    }

    fn update_world_state(&mut self, delta_time: f32) {
        // Natural world state evolution
        for (variable, value) in &mut self.world_state.global_variables {
            match variable.as_str() {
                "tension" => {
                    // Tension naturally decreases over time
                    *value *= 0.999;
                },
                "prosperity" => {
                    // Prosperity slowly increases
                    *value += 0.001 * delta_time;
                },
                _ => {},
            }
            
            *value = value.clamp(0.0, 1.0);
        }
        
        // Update conflicts
        for conflict in &mut self.world_state.active_conflicts {
            conflict.intensity *= 0.995; // Conflicts naturally de-escalate
        }
        
        // Remove resolved conflicts
        self.world_state.active_conflicts.retain(|conflict| conflict.intensity > 0.1);
    }

    // Public interface methods
    pub fn create_custom_storyline(&mut self, title: String, storyline_type: StorylineType, participants: Vec<String>) -> String {
        let storyline_id = format!("custom_{}_{}", title.to_lowercase().replace(' ', '_"), self.active_storylines.len());
        
        let storyline = Storyline {
            id: storyline_id.clone(),
            title,
            description: format!("Custom storyline created by the {}", "engineer"),
            storyline_type,
            current_act: 1,
            acts: vec![StoryAct {
                act_number: 1,
                title: "Beginning".to_string(),
                objectives: vec![],
                scenes: vec![],
                climax_threshold: 0.8,
                resolution_conditions: vec![],
                character_development: HashMap::new(),
            }],
            participants,
            locations: vec![],
            themes: vec!["engineering".to_string(), "creativity".to_string()],
            tone: StoryTone::Inspirational,
            progression: 0.0,
            branching_paths: vec![],
            dependencies: vec![],
            consequences: vec![],
        };

        self.active_storylines.insert(storyline_id.clone(), storyline);
        storyline_id
    }

    pub fn add_storyline_objective(&mut self, storyline_id: &str, objective: Objective) -> bool {
        if let Some(storyline) = self.active_storylines.get_mut(storyline_id) {
            let current_act_index = (storyline.current_act - 1) as usize;
            if let Some(current_act) = storyline.acts.get_mut(current_act_index) {
                current_act.objectives.push(objective);
                return true;
            }
        }
        false
    }

    pub fn get_active_storylines(&self) -> Vec<&Storyline> {
        self.active_storylines.values().collect()
    }

    pub fn get_storyline_progress(&self, storyline_id: &str) -> Option<f32> {
        self.active_storylines.get(storyline_id).map(|s| s.progression)
    }

    pub fn get_world_state_summary(&self) -> WorldStateSummary {
        WorldStateSummary {
            active_storylines: self.active_storylines.len(),
            completed_storylines: self.world_state.completed_storylines.len(),
            active_conflicts: self.world_state.active_conflicts.len(),
            world_mood: self.world_state.world_mood,
            major_events: self.story_history.len(),
        }
    }

    // Helper methods for story system integration
    fn has_conversation_occurred(&self, participant1: &str, participant2: &str) -> bool {
        // Check if a conversation has occurred between two participants
        self.dialogue_system.dialogue_history
            .iter()
            .any(|exchange| {
                exchange.participants.contains(&participant1.to_string()) &&
                exchange.participants.contains(&participant2.to_string())
            })
    }

    fn identify_story_opportunities(&self, npcs: &HashMap<String, NPC>) -> Vec<StoryOpportunity> {
        let mut opportunities = Vec::new();
        
        // Look for NPCs with high relationship changes
        for (npc_id, npc) in npcs {
            if npc.relationships.values().any(|r| r.affection > 0.8) {
                opportunities.push(StoryOpportunity {
                    opportunity_type: "romance".to_string(),
                    involved_npcs: vec![npc_id.clone()],
                    location: npc.position,
                    story_potential: 0.7,
                    suggested_themes: vec!["love".to_string(), "relationships".to_string()],
                    trigger_event: "high_affection".to_string(),
                });
            }
            
            if npc.relationships.values().any(|r| r.trust < 0.2) {
                opportunities.push(StoryOpportunity {
                    opportunity_type: "conflict".to_string(),
                    involved_npcs: vec![npc_id.clone()],
                    location: npc.position,
                    story_potential: 0.8,
                    suggested_themes: vec!["betrayal".to_string(), "conflict".to_string()],
                    trigger_event: "low_trust".to_string(),
                });
            }
        }
        
        opportunities
    }

    fn create_storyline_from_opportunity(&self, opportunity: StoryOpportunity) -> Storyline {
        let npc_string = opportunity.involved_npcs.join("-");
        let storyline_id = format!("emergent-{}-{}", opportunity.opportunity_type, npc_string);
        
        Storyline {
            id: storyline_id,
            title: format!("The {}", opportunity.opportunity_type.to_uppercase()),
            description: format!("An emergent story involving {}", opportunity.involved_npcs.join(", ")),
            storyline_type: match opportunity.opportunity_type.as_str() {
                "romance" => StorylineType::Romance,
                "conflict" => StorylineType::Tragedy,
                _ => StorylineType::PersonalStory,
            },
            current_act: 1,
            acts: vec![StoryAct {
                act_number: 1,
                title: "Development".to_string(),
                objectives: vec![],
                scenes: vec![],
                climax_threshold: 0.8,
                resolution_conditions: vec![],
                character_development: HashMap::new(),
            }],
            participants: opportunity.involved_npcs,
            locations: vec![opportunity.location],
            themes: opportunity.suggested_themes,
            tone: StoryTone::Dramatic,
            progression: 0.0,
            branching_paths: vec![],
            dependencies: vec![],
            consequences: vec![],
        }
    }
}

// Additional supporting types
#[derive(Debug, Clone)]
pub struct StoryOpportunity {
    pub opportunity_type: String,
    pub involved_npcs: Vec<String>,
    pub location: Point3,
    pub story_potential: f32,
    pub suggested_themes: Vec<String>,
    pub trigger_event: String,
}

#[derive(Debug)]
pub struct WorldStateSummary {
    pub active_storylines: usize,
    pub completed_storylines: usize,
    pub active_conflicts: usize,
    pub world_mood: f32,
    pub major_events: usize,
}

impl Default for WorldState {
    fn default() -> Self {
        Self {
            global_variables: HashMap::new(),
            faction_standings: HashMap::new(),
            world_reputation: HashMap::new(),
            completed_storylines: Vec::new(),
            active_conflicts: Vec::new(),
            historical_events: Vec::new(),
            known_secrets: Vec::new(),
            world_mood: 0.6,
            technological_progress: 0.5,
            environmental_state: HashMap::new(),
        }
    }
}

impl Default for StorySettings {
    fn default() -> Self {
        Self {
            narrative_complexity: 0.7,
            branching_frequency: 0.5,
            character_development_rate: 0.6,
            emotional_intensity: 0.8,
            moral_ambiguity: 0.4,
            comedy_level: 0.3,
            romance_likelihood: 0.2,
            tragedy_tolerance: 0.5,
            mystery_elements: 0.6,
            epic_scale_preference: 0.7,
        }
    }
}