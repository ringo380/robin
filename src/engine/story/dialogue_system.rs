use std::collections::HashMap;
use crate::engine::story::{Character, WorldState};

#[derive(Debug, Clone)]
pub struct DialogueSystem {
    dialogue_trees: HashMap<String, DialogueTree>,
    conversation_states: HashMap<String, ConversationState>,
    character_speech_patterns: HashMap<String, SpeechPattern>,
    dialogue_history: Vec<DialogueExchange>,
    dynamic_dialogue_generator: DynamicDialogueGenerator,
    emotion_system: EmotionSystem,
}

#[derive(Debug, Clone)]
pub struct DialogueTree {
    pub id: String,
    pub name: String,
    pub root_node: DialogueNode,
    pub context_requirements: Vec<ContextRequirement>,
    pub participants: Vec<String>,
    pub tree_type: DialogueTreeType,
}

#[derive(Debug, Clone)]
pub enum DialogueTreeType {
    Linear,
    Branching,
    Dynamic,
    Procedural,
}

#[derive(Debug, Clone)]
pub struct DialogueNode {
    pub id: String,
    pub speaker: String,
    pub content: DialogueContent,
    pub responses: Vec<DialogueResponse>,
    pub conditions: Vec<DialogueCondition>,
    pub effects: Vec<DialogueEffect>,
    pub emotional_context: EmotionalContext,
}

#[derive(Debug, Clone)]
pub enum DialogueContent {
    StaticText(String),
    TemplatedText(String, HashMap<String, String>),
    GeneratedText(GenerationParameters),
    ConditionalText(Vec<ConditionalContent>),
}

#[derive(Debug, Clone)]
pub struct ConditionalContent {
    pub condition: DialogueCondition,
    pub content: String,
    pub priority: f32,
}

#[derive(Debug, Clone)]
pub struct GenerationParameters {
    pub topic: String,
    pub tone: DialogueTone,
    pub length: DialogueLength,
    pub personality_influence: f32,
    pub context_awareness: f32,
}

#[derive(Debug, Clone)]
pub enum DialogueTone {
    Friendly,
    Hostile,
    Formal,
    Casual,
    Mysterious,
    Humorous,
    Romantic,
    Intimidating,
}

#[derive(Debug, Clone)]
pub enum DialogueLength {
    Short,
    Medium,
    Long,
    Variable,
}

#[derive(Debug, Clone)]
pub struct DialogueResponse {
    pub id: String,
    pub text: String,
    pub next_node: Option<String>,
    pub requirements: Vec<ResponseRequirement>,
    pub consequences: Vec<ResponseConsequence>,
    pub speaker_motivation: SpeakerMotivation,
}

#[derive(Debug, Clone)]
pub enum SpeakerMotivation {
    Information,
    Persuasion,
    Deception,
    Comfort,
    Challenge,
    Romance,
    Business,
}

#[derive(Debug, Clone)]
pub struct ResponseRequirement {
    pub requirement_type: RequirementType,
    pub parameter: String,
    pub threshold: f32,
}

#[derive(Debug, Clone)]
pub enum RequirementType {
    CharacterSkill,
    RelationshipLevel,
    QuestProgress,
    ItemPossession,
    WorldState,
    CharacterTrait,
}

#[derive(Debug, Clone)]
pub struct ResponseConsequence {
    pub consequence_type: ConsequenceType,
    pub target: String,
    pub magnitude: f32,
    pub description: String,
}

#[derive(Debug, Clone)]
pub enum ConsequenceType {
    RelationshipChange,
    QuestProgression,
    ItemGain,
    ItemLoss,
    SkillGain,
    WorldStateChange,
    EmotionalImpact,
}

#[derive(Debug, Clone)]
pub struct DialogueCondition {
    pub condition_type: ConditionType,
    pub parameter: String,
    pub comparison: ComparisonOperator,
    pub value: String,
}

#[derive(Debug, Clone)]
pub enum ConditionType {
    WorldVariable,
    CharacterProperty,
    RelationshipStatus,
    QuestState,
    TimeOfDay,
    Location,
    PreviousDialogue,
}

#[derive(Debug, Clone)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    Contains,
    NotContains,
}

#[derive(Debug, Clone)]
pub struct DialogueEffect {
    pub effect_type: EffectType,
    pub target: String,
    pub value: String,
    pub duration: Option<u64>,
}

#[derive(Debug, Clone)]
pub enum EffectType {
    StateChange,
    RelationshipModification,
    QuestTrigger,
    ItemTransfer,
    EmotionModification,
    MemoryCreation,
}

#[derive(Debug, Clone)]
pub struct ConversationState {
    pub participants: Vec<String>,
    pub current_tree: String,
    pub current_node: String,
    pub conversation_context: ConversationContext,
    pub turn_order: Vec<String>,
    pub current_speaker: String,
    pub mood: ConversationMood,
}

#[derive(Debug, Clone)]
pub struct ConversationContext {
    pub topic: String,
    pub location: String,
    pub privacy_level: PrivacyLevel,
    pub urgency: f32,
    pub emotional_tension: f32,
    pub shared_history: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum PrivacyLevel {
    Public,
    SemiPrivate,
    Private,
    Intimate,
}

#[derive(Debug, Clone)]
pub struct ConversationMood {
    pub primary_emotion: String,
    pub intensity: f32,
    pub participant_moods: HashMap<String, ParticipantMood>,
}

#[derive(Debug, Clone)]
pub struct ParticipantMood {
    pub current_emotion: String,
    pub engagement_level: f32,
    pub comfort_level: f32,
    pub trust_level: f32,
}

#[derive(Debug, Clone)]
pub struct SpeechPattern {
    pub character_id: String,
    pub vocabulary_level: VocabularyLevel,
    pub speech_quirks: Vec<SpeechQuirk>,
    pub common_phrases: Vec<String>,
    pub topic_preferences: HashMap<String, f32>,
    pub cultural_background: CulturalBackground,
}

#[derive(Debug, Clone)]
pub enum VocabularyLevel {
    Simple,
    Average,
    Advanced,
    Professional,
    Academic,
}

#[derive(Debug, Clone)]
pub struct SpeechQuirk {
    pub quirk_type: QuirkType,
    pub frequency: f32,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum QuirkType {
    Stutter,
    Repetition,
    Slang,
    FormalSpeech,
    Accent,
    Hesitation,
    Interruption,
}

#[derive(Debug, Clone)]
pub struct CulturalBackground {
    pub region: String,
    pub social_class: String,
    pub education_level: String,
    pub profession: String,
    pub language_influences: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DialogueExchange {
    pub exchange_id: String,
    pub participants: Vec<String>,
    pub timestamp: u64,
    pub location: String,
    pub exchanges: Vec<SingleExchange>,
    pub outcomes: Vec<ExchangeOutcome>,
}

#[derive(Debug, Clone)]
pub struct SingleExchange {
    pub speaker: String,
    pub content: String,
    pub tone: DialogueTone,
    pub emotional_state: String,
    pub response_to: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ExchangeOutcome {
    pub outcome_type: OutcomeType,
    pub affected_character: String,
    pub change_description: String,
    pub magnitude: f32,
}

#[derive(Debug, Clone)]
pub enum OutcomeType {
    RelationshipChange,
    InformationGained,
    QuestAdvancement,
    EmotionalImpact,
    WorldKnowledge,
    SkillDevelopment,
}

#[derive(Debug, Clone)]
pub struct DynamicDialogueGenerator {
    topic_knowledge: HashMap<String, TopicKnowledge>,
    personality_templates: HashMap<String, PersonalityTemplate>,
    context_analyzers: Vec<ContextAnalyzer>,
    generation_rules: Vec<GenerationRule>,
}

#[derive(Debug, Clone)]
pub struct TopicKnowledge {
    pub topic: String,
    pub knowledge_base: Vec<KnowledgePoint>,
    pub common_perspectives: Vec<Perspective>,
    pub related_topics: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct KnowledgePoint {
    pub fact: String,
    pub certainty: f32,
    pub source_credibility: f32,
    pub emotional_weight: f32,
}

#[derive(Debug, Clone)]
pub struct Perspective {
    pub viewpoint: String,
    pub supporting_arguments: Vec<String>,
    pub emotional_association: String,
    pub popularity: f32,
}

#[derive(Debug, Clone)]
pub struct PersonalityTemplate {
    pub personality_type: String,
    pub speech_patterns: Vec<String>,
    pub response_tendencies: HashMap<String, f32>,
    pub emotional_expressions: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ContextAnalyzer {
    pub analyzer_type: AnalyzerType,
    pub weight: f32,
    pub analysis_function: String,
}

#[derive(Debug, Clone)]
pub enum AnalyzerType {
    EmotionalState,
    RelationshipDynamic,
    PowerBalance,
    InformationAsymmetry,
    CulturalContext,
}

#[derive(Debug, Clone)]
pub struct GenerationRule {
    pub rule_name: String,
    pub conditions: Vec<DialogueCondition>,
    pub modifications: Vec<ContentModification>,
    pub priority: f32,
}

#[derive(Debug, Clone)]
pub struct ContentModification {
    pub modification_type: ModificationType,
    pub target_element: String,
    pub modification_value: String,
}

#[derive(Debug, Clone)]
pub enum ModificationType {
    ToneAdjustment,
    LengthAdjustment,
    VocabularyChange,
    TopicShift,
    EmotionalInflection,
}

#[derive(Debug, Clone)]
pub struct EmotionSystem {
    emotional_states: HashMap<String, EmotionalState>,
    emotion_transitions: HashMap<String, Vec<EmotionTransition>>,
    empathy_calculator: EmpathyCalculator,
}

#[derive(Debug, Clone)]
pub struct EmotionalState {
    pub character_id: String,
    pub primary_emotion: String,
    pub secondary_emotions: Vec<String>,
    pub intensity: f32,
    pub stability: f32,
    pub triggers: Vec<EmotionalTrigger>,
}

#[derive(Debug, Clone)]
pub struct EmotionTransition {
    pub from_emotion: String,
    pub to_emotion: String,
    pub trigger_condition: String,
    pub transition_probability: f32,
    pub transition_speed: f32,
}

#[derive(Debug, Clone)]
pub struct EmotionalTrigger {
    pub trigger_type: TriggerType,
    pub intensity_threshold: f32,
    pub resulting_emotion: String,
}

#[derive(Debug, Clone)]
pub enum TriggerType {
    TopicMention,
    ToneDetection,
    RelationshipChange,
    MemoryRecall,
    PhysicalProximity,
}

#[derive(Debug, Clone)]
pub struct EmpathyCalculator {
    empathy_models: HashMap<String, EmpathyModel>,
}

#[derive(Debug, Clone)]
pub struct EmpathyModel {
    pub character_id: String,
    pub empathy_level: f32,
    pub emotional_sensitivity: f32,
    pub response_patterns: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct EmotionalContext {
    pub speaker_emotion: String,
    pub target_emotion: String,
    pub emotional_distance: f32,
    pub empathy_factor: f32,
}

#[derive(Debug, Clone)]
pub struct ContextRequirement {
    pub requirement_type: String,
    pub required_value: String,
    pub importance: f32,
}

impl DialogueSystem {
    pub fn new() -> Self {
        DialogueSystem {
            dialogue_trees: HashMap::new(),
            conversation_states: HashMap::new(),
            character_speech_patterns: HashMap::new(),
            dialogue_history: Vec::new(),
            dynamic_dialogue_generator: DynamicDialogueGenerator {
                topic_knowledge: HashMap::new(),
                personality_templates: HashMap::new(),
                context_analyzers: Vec::new(),
                generation_rules: Vec::new(),
            },
            emotion_system: EmotionSystem {
                emotional_states: HashMap::new(),
                emotion_transitions: HashMap::new(),
                empathy_calculator: EmpathyCalculator {
                    empathy_models: HashMap::new(),
                },
            },
        }
    }

    pub fn start_conversation(&mut self, participants: Vec<String>, tree_id: &str, context: ConversationContext) -> Result<String, String> {
        if !self.dialogue_trees.contains_key(tree_id) {
            return Err(format!("Dialogue tree '{}' not found", tree_id));
        }
        
        let conversation_id = format!("conv_{}_{}", participants.join("_"), tree_id);
        
        let conversation_state = ConversationState {
            participants: participants.clone(),
            current_tree: tree_id.to_string(),
            current_node: "root".to_string(),
            conversation_context: context,
            turn_order: participants.clone(),
            current_speaker: participants[0].clone(),
            mood: ConversationMood {
                primary_emotion: "neutral".to_string(),
                intensity: 0.5,
                participant_moods: HashMap::new(),
            },
        };
        
        self.conversation_states.insert(conversation_id.clone(), conversation_state);
        
        Ok(conversation_id)
    }

    pub fn get_current_dialogue(&self, conversation_id: &str) -> Result<DialoguePresentation, String> {
        let state = self.conversation_states.get(conversation_id)
            .ok_or("Conversation not found")?;
            
        let tree = self.dialogue_trees.get(&state.current_tree)
            .ok_or("Dialogue tree not found")?;
            
        let node = self.find_dialogue_node(&tree.root_node, &state.current_node)
            .ok_or("Dialogue node not found")?;
            
        let content = self.render_dialogue_content(&node.content, &state.participants, &state.conversation_context)?;
        
        let available_responses = self.get_available_responses(node, &state.participants[0]);
        
        Ok(DialoguePresentation {
            speaker: node.speaker.clone(),
            content,
            responses: available_responses,
            emotional_context: node.emotional_context.clone(),
        })
    }

    pub fn choose_response(&mut self, conversation_id: &str, response_id: &str, world_state: &mut WorldState) -> Result<DialogueResult, String> {
        let (current_tree_id, current_node_id) = {
            let state = self.conversation_states.get(conversation_id)
                .ok_or("Conversation not found")?;
            (state.current_tree.clone(), state.current_node.clone())
        };

        let tree = self.dialogue_trees.get(&current_tree_id)
            .ok_or("Dialogue tree not found")?.clone();

        let current_node = Self::find_dialogue_node_static(&tree.root_node, &current_node_id)
            .ok_or("Current dialogue node not found")?;
            
        let response = current_node.responses.iter()
            .find(|r| r.id == response_id)
            .ok_or("Response not found")?;
            
        let consequences = self.apply_response_consequences(&response.consequences, world_state);

        // Get the data we need before borrowing mutably
        let (current_speaker, next_speaker) = {
            let state = self.conversation_states.get(conversation_id)
                .ok_or("Conversation not found")?;
            (state.current_speaker.clone(), self.determine_next_speaker(state))
        };

        // Now get mutable reference to state for updates
        let state = self.conversation_states.get_mut(conversation_id)
            .ok_or("Conversation not found")?;

        if let Some(ref next_node) = response.next_node {
            state.current_node = next_node.clone();
        }

        let exchange = SingleExchange {
            speaker: current_speaker,
            content: response.text.clone(),
            tone: DialogueTone::Casual, // Would be determined dynamically
            emotional_state: "neutral".to_string(),
            response_to: Some(state.current_node.clone()),
        };

        // Drop the mutable borrow before calling other methods
        drop(state);

        self.record_dialogue_exchange(conversation_id, exchange);

        Ok(DialogueResult {
            conversation_continues: response.next_node.is_some(),
            consequences,
            next_speaker,
            emotional_changes: Vec::new(),
        })
    }

    pub fn generate_dynamic_dialogue(&mut self, speaker: &str, topic: &str, context: &ConversationContext) -> Result<String, String> {
        let speech_pattern = self.character_speech_patterns.get(speaker)
            .ok_or("Speech pattern not found for character")?;
            
        let topic_knowledge = self.dynamic_dialogue_generator.topic_knowledge.get(topic);
        
        let generation_params = GenerationParameters {
            topic: topic.to_string(),
            tone: self.determine_appropriate_tone(speaker, context),
            length: DialogueLength::Medium,
            personality_influence: 0.8,
            context_awareness: 0.9,
        };
        
        let content = self.generate_contextual_content(&generation_params, speech_pattern, topic_knowledge, context)?;

        Ok(content)
    }

    pub fn end_conversation(&mut self, conversation_id: &str) -> Result<ConversationSummary, String> {
        let state = self.conversation_states.remove(conversation_id)
            .ok_or("Conversation not found")?;
            
        let summary = ConversationSummary {
            participants: state.participants,
            duration: 0, // Would calculate actual duration
            topics_discussed: vec![state.conversation_context.topic],
            relationship_changes: Vec::new(),
            information_exchanged: Vec::new(),
            emotional_outcomes: HashMap::new(),
        };
        
        Ok(summary)
    }

    pub fn register_dialogue_tree(&mut self, tree: DialogueTree) {
        self.dialogue_trees.insert(tree.id.clone(), tree);
    }

    pub fn register_speech_pattern(&mut self, pattern: SpeechPattern) {
        self.character_speech_patterns.insert(pattern.character_id.clone(), pattern);
    }

    pub fn update_emotional_state(&mut self, character_id: &str, emotion: &str, intensity: f32) {
        self.emotion_system.emotional_states.insert(
            character_id.to_string(),
            EmotionalState {
                character_id: character_id.to_string(),
                primary_emotion: emotion.to_string(),
                secondary_emotions: Vec::new(),
                intensity,
                stability: 0.5,
                triggers: Vec::new(),
            }
        );
    }

    fn find_dialogue_node<'a>(&self, root: &'a DialogueNode, node_id: &str) -> Option<&'a DialogueNode> {
        if root.id == node_id {
            return Some(root);
        }

        // Would implement recursive search through dialogue tree
        Some(root) // Simplified
    }

    // Static version to avoid borrowing conflicts
    fn find_dialogue_node_static<'a>(root: &'a DialogueNode, node_id: &'a str) -> Option<&'a DialogueNode> {
        if root.id == node_id {
            return Some(root);
        }

        // Would implement recursive search through dialogue tree
        Some(root) // Simplified
    }

    fn render_dialogue_content(&self, content: &DialogueContent, participants: &[String], context: &ConversationContext) -> Result<String, String> {
        match content {
            DialogueContent::StaticText(text) => Ok(text.clone()),
            DialogueContent::TemplatedText(template, vars) => {
                let mut rendered = template.clone();
                for (key, value) in vars {
                    rendered = rendered.replace(&format!("{{{}}}", key), value);
                }
                Ok(rendered)
            }
            DialogueContent::GeneratedText(params) => {
                self.generate_contextual_content(params, 
                    self.character_speech_patterns.get(&participants[0]).unwrap(), 
                    self.dynamic_dialogue_generator.topic_knowledge.get(&params.topic), 
                    context)
                    .map_err(|e| format!("Generation error: {}", e))
            }
            _ => Ok("Generated dialogue content".to_string()),
        }
    }

    fn get_available_responses(&self, node: &DialogueNode, speaker: &str) -> Vec<DialogueResponse> {
        node.responses.iter()
            .filter(|response| self.response_available_to_speaker(response, speaker))
            .cloned()
            .collect()
    }

    fn response_available_to_speaker(&self, response: &DialogueResponse, speaker: &str) -> bool {
        // Check if response requirements are met for the speaker
        true // Simplified
    }

    fn apply_response_consequences(&self, consequences: &[ResponseConsequence], world_state: &mut WorldState) -> Vec<String> {
        let mut applied = Vec::new();
        
        for consequence in consequences {
            match consequence.consequence_type {
                ConsequenceType::WorldStateChange => {
                    world_state.variables.insert(consequence.target.clone(), "changed".to_string());
                    applied.push(format!("Changed world state: {}", consequence.target));
                }
                _ => {
                    applied.push(format!("Applied consequence: {}", consequence.description));
                }
            }
        }
        
        applied
    }

    fn determine_next_speaker(&self, state: &ConversationState) -> Option<String> {
        let current_index = state.turn_order.iter()
            .position(|p| p == &state.current_speaker)?;
        let next_index = (current_index + 1) % state.turn_order.len();
        state.turn_order.get(next_index).cloned()
    }

    fn record_dialogue_exchange(&mut self, conversation_id: &str, exchange: SingleExchange) {
        // Implementation for recording dialogue exchanges
    }

    fn determine_appropriate_tone(&self, speaker: &str, context: &ConversationContext) -> DialogueTone {
        DialogueTone::Casual // Simplified
    }

    fn generate_contextual_content(&self, params: &GenerationParameters, pattern: &SpeechPattern, knowledge: Option<&TopicKnowledge>, context: &ConversationContext) -> Result<String, String> {
        Ok("Generated contextual dialogue content".to_string()) // Simplified
    }
}

#[derive(Debug)]
pub struct DialoguePresentation {
    pub speaker: String,
    pub content: String,
    pub responses: Vec<DialogueResponse>,
    pub emotional_context: EmotionalContext,
}

#[derive(Debug)]
pub struct DialogueResult {
    pub conversation_continues: bool,
    pub consequences: Vec<String>,
    pub next_speaker: Option<String>,
    pub emotional_changes: Vec<String>,
}

#[derive(Debug)]
pub struct ConversationSummary {
    pub participants: Vec<String>,
    pub duration: u64,
    pub topics_discussed: Vec<String>,
    pub relationship_changes: Vec<String>,
    pub information_exchanged: Vec<String>,
    pub emotional_outcomes: HashMap<String, String>,
}