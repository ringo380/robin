use crate::engine::math::{Vec3, Point3};
use crate::engine::npc::NPC;
use crate::engine::story::{Objective, ObjectiveType, ObjectiveTarget, CompletionCriterion, Reward, Consequence};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct QuestSystem {
    active_quests: HashMap<String, Quest>,
    quest_templates: HashMap<String, QuestTemplate>,
    quest_chains: HashMap<String, QuestChain>,
    quest_giver_npcs: HashMap<String, QuestGiver>,
    quest_history: Vec<CompletedQuest>,
    dynamic_quest_generator: DynamicQuestGenerator,
}

#[derive(Debug, Clone)]
pub struct Quest {
    pub id: String,
    pub title: String,
    pub description: String,
    pub quest_type: QuestType,
    pub difficulty: QuestDifficulty,
    pub giver: Option<String>, // NPC ID
    pub objectives: Vec<QuestObjective>,
    pub current_objective_index: usize,
    pub rewards: Vec<Reward>,
    pub failure_consequences: Vec<Consequence>,
    pub time_limit: Option<u32>,
    pub prerequisites: Vec<QuestPrerequisite>,
    pub progress: f32,
    pub status: QuestStatus,
    pub creation_time: u64,
    pub completion_time: Option<u64>,
    pub participants: Vec<String>, // NPC IDs involved
    pub locations: Vec<Point3>,
    pub journal_entries: Vec<JournalEntry>,
}

#[derive(Debug, Clone)]
pub enum QuestType {
    MainStory,
    SideQuest,
    Daily,
    Weekly,
    Chain,
    Repeatable,
    OneTime,
    Emergency,
    Community,
    Personal,
    Discovery,
    Construction,
    Social,
    Combat,
    Mystery,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum QuestDifficulty {
    Trivial,
    Easy,
    Normal,
    Hard,
    Expert,
    Legendary,
}

#[derive(Debug, Clone)]
pub struct QuestObjective {
    pub objective: Objective,
    pub status: ObjectiveStatus,
    pub progress: f32,
    pub hints: Vec<String>,
    pub discovered: bool, // Hidden objectives
}

#[derive(Debug, Clone)]
pub enum ObjectiveStatus {
    NotStarted,
    InProgress,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, PartialEq)]
pub enum QuestStatus {
    Available,
    Active,
    Completed,
    Failed,
    Abandoned,
    OnHold,
}

#[derive(Debug, Clone)]
pub enum QuestPrerequisite {
    CompletedQuest(String),
    SkillLevel(String, f32),
    RelationshipLevel(String, f32),
    WorldState(String, f32),
    TimeOfDay(u32, u32),
    LocationVisited(Point3),
    ItemPossession(String),
    NPCAlive(String),
}

#[derive(Debug, Clone)]
pub struct JournalEntry {
    pub timestamp: u64,
    pub entry_type: JournalEntryType,
    pub content: String,
    pub importance: f32,
}

#[derive(Debug, Clone)]
pub enum JournalEntryType {
    ObjectiveUpdate,
    Dialogue,
    Discovery,
    PersonalThought,
    WorldObservation,
    Reminder,
    Clue,
    Decision,
}

#[derive(Debug, Clone)]
pub struct QuestTemplate {
    pub template_id: String,
    pub name: String,
    pub description_template: String,
    pub objective_templates: Vec<ObjectiveTemplate>,
    pub reward_formulas: Vec<RewardFormula>,
    pub prerequisite_patterns: Vec<PrerequisitePattern>,
    pub customization_points: Vec<CustomizationPoint>,
    pub narrative_hooks: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ObjectiveTemplate {
    pub objective_type: ObjectiveType,
    pub description_template: String,
    pub target_patterns: Vec<TargetPattern>,
    pub completion_formulas: Vec<CompletionFormula>,
    pub difficulty_modifiers: HashMap<QuestDifficulty, f32>,
}

#[derive(Debug, Clone)]
pub enum TargetPattern {
    NearestNPC(String), // Role/type
    RandomLocation(f32), // Distance from origin
    SpecificItem(String),
    ConceptualTarget(String),
    PlayerChoice,
}

#[derive(Debug, Clone)]
pub struct CompletionFormula {
    pub criterion_type: String,
    pub base_value: f32,
    pub scaling_factors: HashMap<String, f32>,
    pub randomization_range: f32,
}

#[derive(Debug, Clone)]
pub struct RewardFormula {
    pub reward_type: String,
    pub base_amount: f32,
    pub difficulty_scaling: f32,
    pub completion_bonus: f32,
    pub time_bonus: f32,
}

#[derive(Debug, Clone)]
pub struct PrerequisitePattern {
    pub pattern_type: String,
    pub conditions: Vec<String>,
    pub alternatives: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CustomizationPoint {
    pub point_type: CustomizationType,
    pub options: Vec<String>,
    pub impact_on_rewards: f32,
    pub narrative_weight: f32,
}

#[derive(Debug, Clone)]
pub enum CustomizationType {
    Location,
    NPC,
    Objective,
    Theme,
    Tone,
    Reward,
    Difficulty,
}

#[derive(Debug, Clone)]
pub struct QuestChain {
    pub chain_id: String,
    pub name: String,
    pub description: String,
    pub quests: Vec<String>, // Quest IDs in order
    pub current_quest_index: usize,
    pub chain_type: ChainType,
    pub overall_theme: String,
    pub completion_rewards: Vec<Reward>,
    pub branching_points: Vec<ChainBranchingPoint>,
}

#[derive(Debug, Clone)]
pub enum ChainType {
    Linear,
    Branching,
    Parallel,
    Cyclical,
    ConditionalLoop,
}

#[derive(Debug, Clone)]
pub struct ChainBranchingPoint {
    pub quest_index: usize,
    pub branch_condition: BranchCondition,
    pub branch_paths: Vec<BranchPath>,
}

#[derive(Debug, Clone)]
pub enum BranchCondition {
    QuestOutcome(String),
    PlayerChoice,
    WorldState(String, f32),
    NPCState(String, String),
}

#[derive(Debug, Clone)]
pub struct BranchPath {
    pub path_name: String,
    pub quest_sequence: Vec<String>,
    pub convergence_point: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct QuestGiver {
    pub npc_id: String,
    pub available_quests: Vec<String>,
    pub quest_giving_style: QuestGivingStyle,
    pub relationship_requirements: f32,
    pub specializations: Vec<String>,
    pub quest_cooldowns: HashMap<String, u32>,
}

#[derive(Debug, Clone)]
pub enum QuestGivingStyle {
    Direct,
    Mysterious,
    Urgent,
    Casual,
    Formal,
    Manipulative,
    Desperate,
    Inspiring,
}

#[derive(Debug, Clone)]
pub struct CompletedQuest {
    pub quest: Quest,
    pub completion_rating: QuestRating,
    pub player_choices: Vec<String>,
    pub time_taken: u32,
    pub npcs_helped: Vec<String>,
    pub world_impact: f32,
    pub personal_impact: f32,
    pub lessons_learned: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum QuestRating {
    Failed,
    Barely,
    Adequate,
    Good,
    Excellent,
    Perfect,
}

#[derive(Debug, Clone)]
pub struct DynamicQuestGenerator {
    generation_parameters: GenerationParameters,
    content_library: ContentLibrary,
    narrative_patterns: Vec<NarrativePattern>,
    world_analysis: WorldAnalysis,
}

#[derive(Debug, Clone)]
pub struct GenerationParameters {
    pub quest_frequency: f32,
    pub difficulty_distribution: HashMap<QuestDifficulty, f32>,
    pub type_preferences: HashMap<QuestType, f32>,
    pub narrative_complexity: f32,
    pub player_skill_consideration: f32,
    pub world_state_integration: f32,
}

#[derive(Debug, Clone)]
pub struct ContentLibrary {
    pub name_fragments: HashMap<String, Vec<String>>,
    pub description_templates: HashMap<String, Vec<String>>,
    pub objective_variants: HashMap<ObjectiveType, Vec<String>>,
    pub reward_pools: HashMap<String, Vec<String>>,
    pub location_types: Vec<String>,
    pub npc_roles: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct NarrativePattern {
    pub pattern_name: String,
    pub story_beats: Vec<StoryBeat>,
    pub character_archetypes: Vec<String>,
    pub conflict_types: Vec<String>,
    pub resolution_methods: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct StoryBeat {
    pub beat_type: BeatType,
    pub description: String,
    pub emotional_tone: String,
    pub typical_duration: f32,
}

#[derive(Debug, Clone)]
pub enum BeatType {
    Opening,
    IncitingIncident,
    RisingAction,
    Climax,
    FallingAction,
    Resolution,
    Twist,
    Revelation,
}

#[derive(Debug, Clone)]
pub struct WorldAnalysis {
    pub current_tensions: Vec<String>,
    pub available_npcs: Vec<String>,
    pub notable_locations: Vec<Point3>,
    pub resource_scarcities: Vec<String>,
    pub recent_events: Vec<String>,
    pub player_relationships: HashMap<String, f32>,
}

impl QuestSystem {
    pub fn new() -> Self {
        let mut system = Self {
            active_quests: HashMap::new(),
            quest_templates: HashMap::new(),
            quest_chains: HashMap::new(),
            quest_giver_npcs: HashMap::new(),
            quest_history: Vec::new(),
            dynamic_quest_generator: DynamicQuestGenerator::new(),
        };
        
        system.initialize_quest_templates();
        system.initialize_quest_givers();
        system
    }

    pub fn update_quests(&mut self, delta_time: f32) {
        let quest_ids: Vec<_> = self.active_quests.keys().cloned().collect();
        
        for quest_id in quest_ids {
            if let Some(quest) = self.active_quests.get_mut(&quest_id) {
                if quest.status != QuestStatus::Active {
                    continue;
                }
                
                // Update quest objectives
                let mut quest_completed = true;
                let mut total_progress = 0.0;
                
                for objective in &mut quest.objectives {
                    match objective.status {
                        ObjectiveStatus::InProgress => {
                            // Simplified auto-progression for testing
                            objective.progress += delta_time * 0.02;
                            if objective.progress >= 1.0 {
                                objective.status = ObjectiveStatus::Completed;
                                objective.progress = 1.0;
                            } else {
                                quest_completed = false;
                            }
                        }
                        ObjectiveStatus::NotStarted => {
                            if objective.objective.optional {
                                // Skip optional objectives for now
                            } else {
                                quest_completed = false;
                                // Auto-start the first non-completed objective
                                objective.status = ObjectiveStatus::InProgress;
                            }
                        }
                        ObjectiveStatus::Failed => {
                            if !objective.objective.optional {
                                quest_completed = false;
                            }
                        }
                        _ => {} // Completed or Skipped
                    }
                    
                    total_progress += objective.progress;
                }
                
                // Update overall quest progress
                quest.progress = total_progress / quest.objectives.len() as f32;
                
                // Check if quest should be completed
                if quest_completed {
                    quest.status = QuestStatus::Completed;
                    quest.completion_time = Some(std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs());
                }
            }
        }
    }

    fn initialize_quest_templates(&mut self) {
        // Construction quest template
        let construction_template = QuestTemplate {
            template_id: "build_structure".to_string(),
            name: "Construction Project".to_string(),
            description_template: "Build a {structure_type} at {location} for {reason}".to_string(),
            objective_templates: vec![
                ObjectiveTemplate {
                    objective_type: ObjectiveType::Collect,
                    description_template: "Gather {amount} {material} for construction".to_string(),
                    target_patterns: vec![TargetPattern::SpecificItem("building_materials".to_string())],
                    completion_formulas: vec![CompletionFormula {
                        criterion_type: "quantity".to_string(),
                        base_value: 10.0,
                        scaling_factors: {
                            let mut factors = HashMap::new();
                            factors.insert("difficulty".to_string(), 1.5);
                            factors
                        },
                        randomization_range: 0.2,
                    }],
                    difficulty_modifiers: HashMap::new(),
                },
                ObjectiveTemplate {
                    objective_type: ObjectiveType::Build,
                    description_template: "Construct the {structure_type}".to_string(),
                    target_patterns: vec![TargetPattern::RandomLocation(50.0)],
                    completion_formulas: vec![CompletionFormula {
                        criterion_type: "construction_complete".to_string(),
                        base_value: 1.0,
                        scaling_factors: HashMap::new(),
                        randomization_range: 0.0,
                    }],
                    difficulty_modifiers: HashMap::new(),
                },
            ],
            reward_formulas: vec![
                RewardFormula {
                    reward_type: "skill_construction".to_string(),
                    base_amount: 5.0,
                    difficulty_scaling: 2.0,
                    completion_bonus: 1.0,
                    time_bonus: 0.5,
                },
            ],
            prerequisite_patterns: vec![],
            customization_points: vec![
                CustomizationPoint {
                    point_type: CustomizationType::Location,
                    options: vec!["town_center".to_string(), "outskirts".to_string(), "hillside".to_string()],
                    impact_on_rewards: 0.1,
                    narrative_weight: 0.3,
                },
            ],
            narrative_hooks: vec![
                "The community needs this building".to_string(),
                "A personal project for self-improvement".to_string(),
                "Preparing for future challenges".to_string(),
            ],
        };

        self.quest_templates.insert("build_structure".to_string(), construction_template);

        // Social quest template
        let social_template = QuestTemplate {
            template_id: "help_npc".to_string(),
            name: "Personal Assistance".to_string(),
            description_template: "Help {npc_name} with their {problem_type}".to_string(),
            objective_templates: vec![
                ObjectiveTemplate {
                    objective_type: ObjectiveType::Talk,
                    description_template: "Speak with {npc_name} about their problem".to_string(),
                    target_patterns: vec![TargetPattern::NearestNPC("troubled".to_string())],
                    completion_formulas: vec![CompletionFormula {
                        criterion_type: "conversation_complete".to_string(),
                        base_value: 1.0,
                        scaling_factors: HashMap::new(),
                        randomization_range: 0.0,
                    }],
                    difficulty_modifiers: HashMap::new(),
                },
                ObjectiveTemplate {
                    objective_type: ObjectiveType::Deliver,
                    description_template: "Deliver {item} to resolve the situation".to_string(),
                    target_patterns: vec![TargetPattern::SpecificItem("solution_item".to_string())],
                    completion_formulas: vec![CompletionFormula {
                        criterion_type: "delivery_complete".to_string(),
                        base_value: 1.0,
                        scaling_factors: HashMap::new(),
                        randomization_range: 0.0,
                    }],
                    difficulty_modifiers: HashMap::new(),
                },
            ],
            reward_formulas: vec![
                RewardFormula {
                    reward_type: "relationship".to_string(),
                    base_amount: 10.0,
                    difficulty_scaling: 1.0,
                    completion_bonus: 5.0,
                    time_bonus: 0.0,
                },
            ],
            prerequisite_patterns: vec![],
            customization_points: vec![],
            narrative_hooks: vec![
                "Someone in need of help".to_string(),
                "A chance to make a difference".to_string(),
                "Building community bonds".to_string(),
            ],
        };

        self.quest_templates.insert("help_npc".to_string(), social_template);
    }

    fn initialize_quest_givers(&mut self) {
        // Example quest givers would be initialized here
        // This would typically be done dynamically based on available NPCs
    }

    pub fn update(&mut self, delta_time: f32, npcs: &HashMap<String, NPC>) {
        // Update active quests
        self.update_active_quests(delta_time, npcs);
        
        // Check for quest completion
        self.check_quest_completion(npcs);
        
        // Generate new quests
        self.generate_dynamic_quests(npcs);
        
        // Update quest chains
        self.update_quest_chains();
        
        // Clean up expired quests
        self.cleanup_expired_quests();
    }

    fn update_active_quests(&mut self, delta_time: f32, npcs: &HashMap<String, NPC>) {
        let quest_ids: Vec<_> = self.active_quests.keys().cloned().collect();
        
        for quest_id in quest_ids {
            // Update objective progress by collecting them first
            let mut objectives_to_update = Vec::new();
            if let Some(quest) = self.active_quests.get(&quest_id) {
                // Update time-based elements
                if let Some(_time_limit) = quest.time_limit {
                    // Would decrease time limit
                }

                // Collect objectives for updating
                for (i, objective) in quest.objectives.iter().enumerate() {
                    objectives_to_update.push((i, objective.objective.clone()));
                }
            }

            // Update each objective
            for (objective_index, mut objective) in objectives_to_update {
                self.update_objective_progress(&mut objective, npcs);
                // Update the quest with the modified objective
                if let Some(quest) = self.active_quests.get_mut(&quest_id) {
                    if let Some(quest_objective) = quest.objectives.get_mut(objective_index) {
                        quest_objective.objective = objective;
                    }
                }
            }

            // Update overall quest progress
            if let Some(quest) = self.active_quests.get_mut(&quest_id) {
                let completed_objectives = quest.objectives.iter()
                    .filter(|obj| matches!(obj.status, ObjectiveStatus::Completed))
                    .count();

                quest.progress = if quest.objectives.is_empty() {
                    0.0
                } else {
                    completed_objectives as f32 / quest.objectives.len() as f32
                };
            }
        }
    }

    fn update_objective_progress(&self, objective: &mut Objective, npcs: &HashMap<String, NPC>) {
        // Simplified objective progress tracking
        match objective.objective_type {
            ObjectiveType::Talk => {
                if let ObjectiveTarget::NPC(npc_id) = &objective.target {
                    // Check if conversation has occurred
                    // This would integrate with the dialogue system
                }
            },
            ObjectiveType::Build => {
                // Check construction progress
                // This would integrate with the construction system
            },
            ObjectiveType::Collect => {
                // Check inventory for required items
                // This would integrate with the inventory system
            },
            _ => {
                // Handle other objective types
            }
        }
    }

    fn check_quest_completion(&mut self, npcs: &HashMap<String, NPC>) {
        let mut completed_quests = Vec::new();
        
        for (quest_id, quest) in &self.active_quests {
            if quest.progress >= 1.0 {
                completed_quests.push(quest_id.clone());
            }
        }
        
        for quest_id in completed_quests {
            self.complete_quest(quest_id);
        }
    }

    fn complete_quest(&mut self, quest_id: String) {
        if let Some(mut quest) = self.active_quests.remove(&quest_id) {
            quest.status = QuestStatus::Completed;
            quest.completion_time = Some(std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs());

            // Create journal entry
            let completion_entry = JournalEntry {
                timestamp: quest.completion_time.unwrap(),
                entry_type: JournalEntryType::ObjectiveUpdate,
                content: format!("Completed quest: {}", quest.title),
                importance: 0.8,
            };
            
            // Calculate completion rating
            let rating = self.calculate_quest_rating(&quest);
            
            // Apply rewards
            for reward in &quest.rewards {
                self.apply_quest_reward(reward);
            }
            
            let completed_quest = CompletedQuest {
                quest,
                completion_rating: rating,
                player_choices: vec![], // Would track actual choices
                time_taken: 0, // Would calculate actual time
                npcs_helped: vec![], // Would track NPCs involved
                world_impact: 0.5,
                personal_impact: 0.7,
                lessons_learned: vec!["Persistence pays off".to_string()],
            };

            self.quest_history.push(completed_quest);
        }
    }

    fn calculate_quest_rating(&self, quest: &Quest) -> QuestRating {
        // Simplified rating calculation
        if quest.progress >= 1.0 {
            match quest.difficulty {
                QuestDifficulty::Trivial => QuestRating::Good,
                QuestDifficulty::Easy => QuestRating::Good,
                QuestDifficulty::Normal => QuestRating::Excellent,
                QuestDifficulty::Hard => QuestRating::Excellent,
                QuestDifficulty::Expert => QuestRating::Perfect,
                QuestDifficulty::Legendary => QuestRating::Perfect,
            }
        } else {
            QuestRating::Failed
        }
    }

    fn apply_quest_reward(&self, reward: &Reward) {
        // Apply rewards to the player/world
        // Note: Reward struct is currently empty, so this is a placeholder implementation
        println!("Applied quest reward: {:?}", reward);
    }

    fn generate_dynamic_quests(&mut self, npcs: &HashMap<String, NPC>) {
        // Limit number of active quests
        if self.active_quests.len() >= 10 {
            return;
        }
        
        // Generate quests based on world state and NPC needs
        let quest_opportunities = self.dynamic_quest_generator.identify_quest_opportunities(npcs);
        
        for opportunity in quest_opportunities {
            if self.should_generate_quest(&opportunity) {
                let new_quest = self.generate_quest_from_opportunity(opportunity);
                self.active_quests.insert(new_quest.id.clone(), new_quest);
            }
        }
    }

    fn should_generate_quest(&self, opportunity: &QuestOpportunity) -> bool {
        // Simple generation criteria
        opportunity.urgency > 0.5 && opportunity.feasibility > 0.6
    }

    fn generate_quest_from_opportunity(&self, opportunity: QuestOpportunity) -> Quest {
        let quest_id = format!("dynamic_quest_{}", self.active_quests.len());
        
        Quest {
            id: quest_id,
            title: opportunity.suggested_title,
            description: opportunity.description,
            quest_type: opportunity.quest_type,
            difficulty: QuestDifficulty::Normal,
            giver: opportunity.giver_npc,
            objectives: opportunity.objectives,
            current_objective_index: 0,
            rewards: opportunity.rewards,
            failure_consequences: vec![],
            time_limit: None,
            prerequisites: vec![],
            progress: 0.0,
            status: QuestStatus::Available,
            creation_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            completion_time: None,
            participants: opportunity.involved_npcs,
            locations: vec![opportunity.location],
            journal_entries: vec![],
        }
    }

    fn update_quest_chains(&mut self) {
        // Update quest chain progression
        for (chain_id, chain) in &mut self.quest_chains {
            if chain.current_quest_index < chain.quests.len() {
                let current_quest_id = &chain.quests[chain.current_quest_index];
                
                // Check if current quest in chain is completed
                if self.quest_history.iter().any(|cq| cq.quest.id == *current_quest_id) {
                    chain.current_quest_index += 1;
                    
                    // Start next quest in chain if available
                    if chain.current_quest_index < chain.quests.len() {
                        let next_quest_id = &chain.quests[chain.current_quest_index];
                        // Would activate the next quest
                        println!("Activating next quest in chain {}: {}", chain_id, next_quest_id);
                    } else {
                        // Chain completed
                        println!("Quest chain {} completed!", chain.name);
                    }
                }
            }
        }
    }

    fn cleanup_expired_quests(&mut self) {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let mut expired_quests = Vec::new();
        
        for (quest_id, quest) in &self.active_quests {
            if let Some(time_limit) = quest.time_limit {
                let elapsed_time = current_time - quest.creation_time;
                if elapsed_time > time_limit as u64 {
                    expired_quests.push(quest_id.clone());
                }
            }
        }
        
        for quest_id in expired_quests {
            if let Some(mut quest) = self.active_quests.remove(&quest_id) {
                quest.status = QuestStatus::Failed;
                
                // Apply failure consequences
                for consequence in &quest.failure_consequences {
                    println!("Applying failure consequence: {:?}", consequence);
                }
            }
        }
    }

    // Public interface methods
    pub fn start_quest(&mut self, quest_id: &str) -> bool {
        if let Some(quest) = self.active_quests.get_mut(quest_id) {
            if matches!(quest.status, QuestStatus::Available) {
                quest.status = QuestStatus::Active;
                
                let start_entry = JournalEntry {
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    entry_type: JournalEntryType::ObjectiveUpdate,
                    content: format!("Started quest: {}", quest.title),
                    importance: 0.6,
                };
                quest.journal_entries.push(start_entry);
                
                return true;
            }
        }
        false
    }

    pub fn abandon_quest(&mut self, quest_id: &str) -> bool {
        if let Some(quest) = self.active_quests.get_mut(quest_id) {
            quest.status = QuestStatus::Abandoned;
            
            // Apply abandonment consequences if any
            for consequence in &quest.failure_consequences {
                println!("Applying abandonment consequence: {:?}", consequence);
            }
            
            return true;
        }
        false
    }

    pub fn get_active_quests(&self) -> Vec<&Quest> {
        self.active_quests.values()
            .filter(|quest| matches!(quest.status, QuestStatus::Active | QuestStatus::Available))
            .collect()
    }

    pub fn get_completed_quests(&self) -> &Vec<CompletedQuest> {
        &self.quest_history
    }

    pub fn create_custom_quest(&mut self, title: String, description: String, objectives: Vec<Objective>) -> String {
        let quest_id = format!("custom_{}_{}", title.to_lowercase().replace(' ', "_"), self.active_quests.len());
        
        let quest_objectives: Vec<QuestObjective> = objectives.into_iter()
            .map(|obj| QuestObjective {
                objective: obj,
                status: ObjectiveStatus::NotStarted,
                progress: 0.0,
                hints: vec![],
                discovered: true,
            })
            .collect();

        let quest = Quest {
            id: quest_id.clone(),
            title,
            description,
            quest_type: QuestType::Personal,
            difficulty: QuestDifficulty::Normal,
            giver: None,
            objectives: quest_objectives,
            current_objective_index: 0,
            rewards: vec![],
            failure_consequences: vec![],
            time_limit: None,
            prerequisites: vec![],
            progress: 0.0,
            status: QuestStatus::Available,
            creation_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            completion_time: None,
            participants: vec![],
            locations: vec![],
            journal_entries: vec![],
        };

        self.active_quests.insert(quest_id.clone(), quest);
        quest_id
    }
}

// Supporting types for quest generation
#[derive(Debug, Clone)]
pub struct QuestOpportunity {
    pub suggested_title: String,
    pub description: String,
    pub quest_type: QuestType,
    pub giver_npc: Option<String>,
    pub objectives: Vec<QuestObjective>,
    pub rewards: Vec<Reward>,
    pub involved_npcs: Vec<String>,
    pub location: Point3,
    pub urgency: f32,
    pub feasibility: f32,
}

impl DynamicQuestGenerator {
    pub fn new() -> Self {
        Self {
            generation_parameters: GenerationParameters::default(),
            content_library: ContentLibrary::new(),
            narrative_patterns: Vec::new(),
            world_analysis: WorldAnalysis::default(),
        }
    }

    pub fn identify_quest_opportunities(&self, npcs: &HashMap<String, NPC>) -> Vec<QuestOpportunity> {
        let mut opportunities = Vec::new();
        
        // Analyze NPC needs and generate quest opportunities
        for (npc_id, npc) in npcs {
            // Check if NPC has low mood - potential for help quest
            if npc.mood < 40.0 {
                opportunities.push(QuestOpportunity {
                    suggested_title: format!("Help {}", npc.name),
                    description: format!("{} seems to be having difficulties and could use assistance", npc.name),
                    quest_type: QuestType::Social,
                    giver_npc: Some(npc_id.clone()),
                    objectives: vec![QuestObjective {
                        objective: Objective {
                            id: "talk_to_npc".to_string(),
                            description: format!("Talk to {} about their problems", npc.name),
                            objective_type: ObjectiveType::Talk,
                            target: ObjectiveTarget::NPC(npc_id.clone()),
                            completion_criterion: CompletionCriterion::Condition("conversation_completed".to_string()),
                            completion_criteria: vec![CompletionCriterion::Condition("conversation_completed".to_string())],
                            optional: false,
                            hidden: false,
                            time_limit: None,
                            rewards: vec![],
                            failure_consequences: vec![],
                        },
                        status: ObjectiveStatus::NotStarted,
                        progress: 0.0,
                        hints: vec!["Maybe they just need someone to listen".to_string()],
                        discovered: true,
                    }],
                    rewards: vec![],
                    involved_npcs: vec![npc_id.clone()],
                    location: npc.position,
                    urgency: (100.0 - npc.mood) / 100.0,
                    feasibility: 0.8,
                });
            }
            
            // Check for skill-based opportunities
            if let Some(construction_skill) = npc.skills.get("construction") {
                if *construction_skill > 50.0 {
                    opportunities.push(QuestOpportunity {
                        suggested_title: format!("Learn from {}", npc.name),
                        description: format!("{} is skilled in construction and might be willing to teach", npc.name),
                        quest_type: QuestType::SideQuest,
                        giver_npc: Some(npc_id.clone()),
                        objectives: vec![QuestObjective {
                            objective: Objective {
                                id: "learn_skill".to_string(),
                                description: format!("Learn construction techniques from {}", npc.name),
                                objective_type: ObjectiveType::Learn,
                                target: ObjectiveTarget::NPC(npc_id.clone()),
                                completion_criterion: CompletionCriterion::Skill("construction".to_string(), 10.0),
                                completion_criteria: vec![CompletionCriterion::Skill("construction".to_string(), 10.0)],
                                optional: false,
                                hidden: false,
                                time_limit: None,
                                rewards: vec![],
                                failure_consequences: vec![],
                            },
                            status: ObjectiveStatus::NotStarted,
                            progress: 0.0,
                            hints: vec!["Bring some building materials to practice with".to_string()],
                            discovered: true,
                        }],
                        rewards: vec![],
                        involved_npcs: vec![npc_id.clone()],
                        location: npc.position,
                        urgency: 0.3,
                        feasibility: 0.9,
                    });
                }
            }
        }
        
        opportunities
    }
}

impl ContentLibrary {
    pub fn new() -> Self {
        Self {
            name_fragments: HashMap::new(),
            description_templates: HashMap::new(),
            objective_variants: HashMap::new(),
            reward_pools: HashMap::new(),
            location_types: vec![
                "forest".to_string(),
                "mountain".to_string(),
                "village".to_string(),
                "ruins".to_string(),
            ],
            npc_roles: vec![
                "merchant".to_string(),
                "craftsperson".to_string(),
                "guard".to_string(),
                "farmer".to_string(),
            ],
        }
    }
}

impl Default for GenerationParameters {
    fn default() -> Self {
        Self {
            quest_frequency: 0.5,
            difficulty_distribution: {
                let mut dist = HashMap::new();
                dist.insert(QuestDifficulty::Easy, 0.3);
                dist.insert(QuestDifficulty::Normal, 0.4);
                dist.insert(QuestDifficulty::Hard, 0.2);
                dist.insert(QuestDifficulty::Expert, 0.1);
                dist
            },
            type_preferences: HashMap::new(),
            narrative_complexity: 0.6,
            player_skill_consideration: 0.7,
            world_state_integration: 0.8,
        }
    }
}

impl Default for WorldAnalysis {
    fn default() -> Self {
        Self {
            current_tensions: Vec::new(),
            available_npcs: Vec::new(),
            notable_locations: Vec::new(),
            resource_scarcities: Vec::new(),
            recent_events: Vec::new(),
            player_relationships: HashMap::new(),
        }
    }
}