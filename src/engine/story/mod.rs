use crate::engine::math::{Vec3, Point3};
use crate::engine::npc::NPC;
use std::collections::HashMap;

pub mod narrative_engine;
pub mod quest_system;
pub mod story_generator;
pub mod event_system;
pub mod dialogue_system;
// Temporarily disabled due to syntax error: pub mod mod_old;

// Placeholder types to resolve imports (will need proper implementation later)
#[derive(Debug, Clone)]
pub struct StoryEvent {
    pub id: String,
    pub event_type: String,
    pub source: String,
    pub target: String,
    pub data: std::collections::HashMap<String, String>,
    pub timestamp: std::time::SystemTime,
}
#[derive(Debug, Clone)]
pub struct Character;
#[derive(Debug, Clone)]
pub struct Objective {
    pub id: String,
    pub description: String,
    pub objective_type: ObjectiveType,
    pub target: ObjectiveTarget,
    pub completion_criterion: CompletionCriterion,
    pub completion_criteria: Vec<CompletionCriterion>,
    pub optional: bool,
    pub hidden: bool,
    pub time_limit: Option<f32>,
    pub rewards: Vec<Reward>,
    pub failure_consequences: Vec<Consequence>,
}
#[derive(Debug, Clone)]
pub enum ObjectiveType { Build, Collect, Interact, Talk, Deliver, Learn }
#[derive(Debug, Clone)]
pub enum ObjectiveTarget { NPC(String), Building(String), Item(String) }
#[derive(Debug, Clone)]
pub enum CompletionCriterion {
    Quantity(u32),
    Time(f32),
    Condition(String),
    Skill(String, f32)
}
#[derive(Debug, Clone)]
pub struct Reward;
#[derive(Debug, Clone)]
pub struct Consequence;
#[derive(Debug, Clone)]
pub struct Quest;

use crate::engine::error::{RobinResult, RobinError};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryManager {
    pub active_storylines: Vec<Storyline>,
    pub completed_storylines: Vec<Storyline>,
    pub world_state: WorldState,
    pub character_relationships: HashMap<String, CharacterRelationship>,
    pub story_settings: StorySettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Storyline {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: StorylineStatus,
    pub chapters: Vec<Chapter>,
    pub current_chapter: usize,
    pub participants: Vec<String>,
    pub themes: Vec<StoryTheme>,
    pub emotional_arc: EmotionalArc,
    pub branching_points: Vec<BranchingPoint>,
    pub prerequisites: Vec<String>,
    pub consequences: Vec<StoryConsequence>,
    pub estimated_duration: f32,
    pub difficulty_level: f32,
    pub moral_complexity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StorylineStatus {
    Inactive,
    Active,
    Paused,
    Completed,
    Abandoned,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub scenes: Vec<Scene>,
    pub objectives: Vec<ChapterObjective>,
    pub character_development: Vec<CharacterDevelopment>,
    pub world_impact: WorldImpact,
    pub emotional_beats: Vec<EmotionalBeat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    pub id: String,
    pub location: Point3,
    pub participants: Vec<String>,
    pub dialogue: Vec<DialogueLine>,
    pub actions: Vec<SceneAction>,
    pub mood: SceneMood,
    pub duration: f32,
    pub importance: SceneImportance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueLine {
    pub speaker: String,
    pub text: String,
    pub emotion: Emotion,
    pub intent: DialogueIntent,
    pub responses: Vec<DialogueResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueResponse {
    pub text: String,
    pub consequences: Vec<StoryConsequence>,
    pub requirements: Vec<String>,
    pub emotional_impact: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DialogueIntent {
    Inform,
    Persuade,
    Threaten,
    Comfort,
    Question,
    Confess,
    Lie,
    Joke,
    Flirt,
    Insult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Emotion {
    Joy,
    Sadness,
    Anger,
    Fear,
    Surprise,
    Disgust,
    Trust,
    Anticipation,
    Contempt,
    Pride,
    Shame,
    Guilt,
    Envy,
    Love,
    Hate,
    Hope,
    Despair,
    Neutral,
    Excitement,
    Satisfaction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneAction {
    pub actor: String,
    pub action_type: ActionType,
    pub target: Option<String>,
    pub description: String,
    pub consequences: Vec<StoryConsequence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    Move,
    Interact,
    Combat,
    Skill,
    Magic,
    Social,
    Exploration,
    Puzzle,
    Stealth,
    Crafting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SceneMood {
    Tense,
    Peaceful,
    Exciting,
    Mysterious,
    Romantic,
    Comedic,
    Tragic,
    Suspenseful,
    Inspiring,
    Melancholic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SceneImportance {
    Critical,
    Major,
    Minor,
    Flavor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterObjective {
    pub description: String,
    pub is_completed: bool,
    pub is_optional: bool,
    pub completion_requirements: Vec<String>,
    pub failure_conditions: Vec<String>,
    pub rewards: Vec<ObjectiveReward>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveReward {
    pub reward_type: RewardType,
    pub amount: f32,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RewardType {
    Experience,
    Reputation,
    Items,
    Knowledge,
    Relationships,
    WorldState,
    Skill,
    Relationship,
    WorldInfluence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterDevelopment {
    pub character_id: String,
    pub development_type: DevelopmentType,
    pub description: String,
    pub impact_level: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DevelopmentType {
    SkillGrowth,
    PersonalityChange,
    RelationshipEvolution,
    GoalShift,
    TraumaResolution,
    MoralDevelopment,
    EmotionalMaturity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldImpact {
    pub political_changes: Vec<PoliticalChange>,
    pub economic_effects: Vec<EconomicEffect>,
    pub social_shifts: Vec<SocialShift>,
    pub environmental_changes: Vec<EnvironmentalChange>,
    pub technological_progress: Vec<TechnologicalAdvancement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoliticalChange {
    pub faction: String,
    pub change_type: PoliticalChangeType,
    pub magnitude: f32,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PoliticalChangeType {
    PowerShift,
    AllianceChange,
    TerritoryControl,
    PolicyChange,
    LeadershipChange,
    Revolution,
    War,
    Peace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicEffect {
    pub sector: String,
    pub effect_type: EconomicEffectType,
    pub magnitude: f32,
    pub duration: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EconomicEffectType {
    Growth,
    Recession,
    Inflation,
    TradeChange,
    ResourceDiscovery,
    ResourceDepletion,
    MarketDisruption,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialShift {
    pub community: String,
    pub shift_type: SocialShiftType,
    pub magnitude: f32,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SocialShiftType {
    CulturalChange,
    ReligiousShift,
    ClassMobility,
    PopulationMovement,
    SocialMovement,
    GenerationalChange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentalChange {
    pub region: String,
    pub change_type: EnvironmentalChangeType,
    pub severity: f32,
    pub recovery_time: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnvironmentalChangeType {
    ClimateShift,
    Disaster,
    Restoration,
    Pollution,
    Deforestation,
    Conservation,
    SpeciesChange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnologicalAdvancement {
    pub field: String,
    pub advancement_type: TechAdvancementType,
    pub impact_level: f32,
    pub adoption_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TechAdvancementType {
    Discovery,
    Innovation,
    Implementation,
    Obsolescence,
    Revolution,
    Evolution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalBeat {
    pub character: String,
    pub emotion: Emotion,
    pub intensity: f32,
    pub trigger: String,
    pub resolution: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalArc {
    pub starting_emotion: Emotion,
    pub emotional_journey: Vec<EmotionalTransition>,
    pub climax_emotion: Emotion,
    pub resolution_emotion: Emotion,
    pub character_growth: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalTransition {
    pub from_emotion: Emotion,
    pub to_emotion: Emotion,
    pub trigger_event: String,
    pub intensity_change: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchingPoint {
    pub id: String,
    pub description: String,
    pub decision_point: String,
    pub choices: Vec<StoryChoice>,
    pub consequences: Vec<StoryConsequence>,
    pub is_major_branch: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryChoice {
    pub text: String,
    pub requirements: Vec<String>,
    pub immediate_consequences: Vec<StoryConsequence>,
    pub long_term_effects: Vec<StoryConsequence>,
    pub moral_weight: MoralAlignment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MoralAlignment {
    Good,
    Neutral,
    Evil,
    Lawful,
    Chaotic,
    Complex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryConsequence {
    pub consequence_type: ConsequenceType,
    pub description: String,
    pub magnitude: f32,
    pub delay: f32,
    pub affected_entities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsequenceType {
    CharacterChange,
    RelationshipChange,
    WorldStateChange,
    ItemGain,
    ItemLoss,
    LocationUnlock,
    LocationLock,
    SkillGain,
    QuestStart,
    QuestEnd,
    FactionStandingChange,
    ReputationChange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StoryTheme {
    Redemption,
    Revenge,
    Love,
    Loss,
    Growth,
    Sacrifice,
    Discovery,
    Betrayal,
    Loyalty,
    Justice,
    Freedom,
    Power,
    Corruption,
    Hope,
    Survival,
    Identity,
    Family,
    Friendship,
    Honor,
    Duty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterRelationship {
    pub character_a: String,
    pub character_b: String,
    pub relationship_type: RelationshipType,
    pub affection_level: f32,
    pub trust_level: f32,
    pub respect_level: f32,
    pub history: Vec<RelationshipEvent>,
    pub current_status: RelationshipStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipType {
    Romantic,
    Platonic,
    Family,
    Professional,
    Rivalry,
    Mentorship,
    Alliance,
    Neutral,
    Hostile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipStatus {
    Developing,
    Stable,
    Strained,
    Improving,
    Deteriorating,
    Broken,
    Reconnecting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipEvent {
    pub event_type: RelationshipEventType,
    pub description: String,
    pub impact: f32,
    pub timestamp: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipEventType {
    FirstMeeting,
    PositiveInteraction,
    NegativeInteraction,
    Conflict,
    Reconciliation,
    SharedExperience,
    Betrayal,
    ActOfKindness,
    Misunderstanding,
    DeepConversation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    pub global_variables: HashMap<String, f32>,
    pub variables: HashMap<String, String>,
    pub faction_standings: HashMap<String, f32>,
    pub world_reputation: HashMap<String, f32>,
    pub completed_storylines: Vec<String>,
    pub active_conflicts: Vec<WorldConflict>,
    pub historical_events: Vec<HistoricalEvent>,
    pub known_secrets: Vec<WorldSecret>,
    pub world_mood: f32,
    pub technological_progress: f32,
    pub environmental_state: HashMap<String, f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldConflict {
    pub id: String,
    pub name: String,
    pub participants: Vec<String>,
    pub conflict_type: ConflictType,
    pub intensity: f32,
    pub duration: f32,
    pub resolution_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictType {
    War,
    Trade,
    Territorial,
    Ideological,
    Resource,
    Succession,
    Religious,
    Personal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalEvent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub participants: Vec<String>,
    pub timestamp: f32,
    pub impact_level: f32,
    pub event_type: HistoricalEventType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HistoricalEventType {
    Battle,
    Treaty,
    Discovery,
    Disaster,
    Celebration,
    Revolution,
    Invention,
    Death,
    Birth,
    Marriage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSecret {
    pub id: String,
    pub description: String,
    pub known_by: Vec<String>,
    pub importance: f32,
    pub revelation_consequences: Vec<StoryConsequence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorySettings {
    pub narrative_complexity: f32,
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

impl Default for StoryManager {
    fn default() -> Self {
        Self::new()
    }
}

impl StoryManager {
    pub fn new() -> Self {
        Self {
            active_storylines: Vec::new(),
            completed_storylines: Vec::new(),
            world_state: WorldState::default(),
            character_relationships: HashMap::new(),
            story_settings: StorySettings::default(),
        }
    }

    pub fn create_storyline(&mut self, title: String, description: String, themes: Vec<StoryTheme>) -> RobinResult<String> {
        let storyline_id = format!("story_{}", self.active_storylines.len());

        let storyline = Storyline {
            id: storyline_id.clone(),
            title,
            description,
            status: StorylineStatus::Inactive,
            chapters: Vec::new(),
            current_chapter: 0,
            participants: Vec::new(),
            themes,
            emotional_arc: EmotionalArc {
                starting_emotion: Emotion::Neutral,
                emotional_journey: Vec::new(),
                climax_emotion: Emotion::Excitement,
                resolution_emotion: Emotion::Satisfaction,
                character_growth: 0.0,
            },
            branching_points: Vec::new(),
            prerequisites: Vec::new(),
            consequences: Vec::new(),
            estimated_duration: 60.0,
            difficulty_level: 0.5,
            moral_complexity: 0.5,
        };

        self.active_storylines.push(storyline);
        Ok(storyline_id)
    }

    pub fn start_storyline(&mut self, storyline_id: &str) -> RobinResult<()> {
        if let Some(storyline) = self.active_storylines.iter_mut().find(|s| s.id == storyline_id) {
            storyline.status = StorylineStatus::Active;
            Ok(())
        } else {
            Err(RobinError::Story(format!("Storyline not found: {}", storyline_id)))
        }
    }

    pub fn update_world_state(&mut self, variable: String, value: f32) {
        self.world_state.global_variables.insert(variable, value);
    }

    pub fn get_active_storylines(&self) -> Vec<&Storyline> {
        self.active_storylines.iter().filter(|s| s.status == StorylineStatus::Active).collect()
    }

    pub fn progress_story(&mut self, delta_time: f32) -> RobinResult<()> {
        for storyline in &mut self.active_storylines {
            if storyline.status == StorylineStatus::Active {
                // Basic story progression logic
                // This is a placeholder for more complex narrative advancement
                storyline.estimated_duration -= delta_time;

                if storyline.estimated_duration <= 0.0 {
                    storyline.status = StorylineStatus::Completed;
                }
            }
        }
        Ok(())
    }

    pub fn get_world_summary(&self) -> WorldStateSummary {
        WorldStateSummary {
            active_storylines: self.active_storylines.iter().filter(|s| s.status == StorylineStatus::Active).count(),
            completed_storylines: self.completed_storylines.len(),
            active_conflicts: self.world_state.active_conflicts.len(),
            world_mood: self.world_state.world_mood,
            major_events: self.world_state.historical_events.iter().filter(|e| e.impact_level > 0.7).count(),
        }
    }
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
        let mut global_vars = HashMap::new();
        global_vars.insert("tension".to_string(), 0.3);
        global_vars.insert("prosperity".to_string(), 0.6);
        global_vars.insert("tech_level".to_string(), 0.5);

        Self {
            global_variables: global_vars,
            variables: HashMap::new(),
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