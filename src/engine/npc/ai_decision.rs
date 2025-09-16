use crate::engine::math::{Vec3, Point3};
use crate::engine::npc::{NPC, Goal, GoalType, GoalCondition, Behavior, BehaviorType};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct AIDecisionEngine {
    decision_trees: HashMap<String, DecisionTree>,
    goal_planner: GoalPlanner,
    utility_calculator: UtilityCalculator,
    knowledge_base: KnowledgeBase,
    learning_system: LearningSystem,
}

#[derive(Debug, Clone)]
pub struct DecisionTree {
    pub root: DecisionNode,
    pub context: DecisionContext,
    pub success_rate: f32,
    pub usage_count: u32,
}

#[derive(Debug, Clone)]
pub enum DecisionNode {
    Condition {
        condition: DecisionCondition,
        true_branch: Box<DecisionNode>,
        false_branch: Box<DecisionNode>,
    },
    Action {
        action: DecisionAction,
        confidence: f32,
    },
    RandomChoice {
        choices: Vec<(DecisionNode, f32)>, // (node, probability)
    },
    Sequence {
        steps: Vec<DecisionNode>,
    },
}

#[derive(Debug, Clone)]
pub enum DecisionCondition {
    NeedLevel(String, f32, ComparisonType), // need_name, threshold, comparison
    GoalPriority(f32),
    ResourceAvailable(String, u32),
    TimeOfDay(u32, u32),
    WeatherCondition(String),
    SocialContext(SocialContextType),
    MemoryExists(String),
    SkillLevel(String, f32),
    RelationshipQuality(String, f32),
    LocationProximity(Point3, f32),
    EmotionalState(EmotionalCondition),
}

#[derive(Debug, Clone)]
pub enum ComparisonType {
    Greater,
    Less,
    Equal,
    GreaterOrEqual,
    LessOrEqual,
}

#[derive(Debug, Clone)]
pub enum SocialContextType {
    NPCNearby(f32), // radius
    InConversation,
    GroupActivity,
    Alone,
    WithFriend,
    AtWork,
}

#[derive(Debug, Clone)]
pub enum EmotionalCondition {
    MoodAbove(f32),
    MoodBelow(f32),
    StressAbove(f32),
    StressBelow(f32),
    RecentEmotionalEvent(String),
}

#[derive(Debug, Clone)]
pub enum DecisionAction {
    SetGoal(Goal),
    ModifyGoal(String, GoalModification),
    ExecuteBehavior(BehaviorType),
    ChangeState(String),
    SendMessage(String, String), // target_npc, message
    UpdateMemory(String, f32), // content, importance
    LearnSkill(String, f32),
    ModifyRelationship(String, f32, f32), // npc_id, affection_change, trust_change
    Wait(f32),
    Explore(Point3),
}

#[derive(Debug, Clone)]
pub enum GoalModification {
    IncreasePriority(f32),
    DecreasePriority(f32),
    AddSubGoal(Goal),
    MarkComplete,
    Abandon,
}

#[derive(Debug, Clone)]
pub enum DecisionContext {
    Emergency,
    Work,
    Social,
    Leisure,
    Maintenance,
    Crisis,
    Opportunity,
}

#[derive(Debug, Clone)]
pub struct GoalPlanner {
    pub active_planners: HashMap<String, NPCGoalPlanner>, // npc_id -> planner
    pub goal_templates: HashMap<String, GoalTemplate>,
    pub planning_algorithms: PlanningAlgorithms,
}

#[derive(Debug, Clone)]
pub struct NPCGoalPlanner {
    pub npc_id: String,
    pub current_plan: Option<Plan>,
    pub goal_stack: Vec<Goal>,
    pub completed_goals: Vec<CompletedGoal>,
    pub failed_goals: Vec<FailedGoal>,
    pub planning_style: PlanningStyle,
    pub risk_tolerance: f32,
}

#[derive(Debug, Clone)]
pub struct Plan {
    pub plan_id: String,
    pub main_goal: Goal,
    pub steps: Vec<PlanStep>,
    pub estimated_duration: f32,
    pub success_probability: f32,
    pub required_resources: HashMap<String, u32>,
    pub contingencies: Vec<Contingency>,
}

#[derive(Debug, Clone)]
pub struct PlanStep {
    pub step_id: String,
    pub action: PlanAction,
    pub prerequisites: Vec<String>,
    pub expected_outcomes: Vec<String>,
    pub duration: f32,
    pub failure_conditions: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum PlanAction {
    MoveTo(Point3),
    InteractWith(String), // NPC ID
    Use(String), // Object/Tool ID
    Wait(f32),
    Learn(String), // Skill name
    Build(String), // Structure type
    Gather(String, u32), // Resource type, amount
    Communicate(String, String), // Target NPC, message intent
}

#[derive(Debug, Clone)]
pub struct Contingency {
    pub trigger: ContingencyTrigger,
    pub response: ContingencyResponse,
    pub priority: f32,
}

#[derive(Debug, Clone)]
pub enum ContingencyTrigger {
    StepFailed(String),
    ResourceUnavailable(String),
    NPCUnavailable(String),
    TimeConstraintViolated,
    EmergencyEvent,
    OpportunityDetected(String),
}

#[derive(Debug, Clone)]
pub enum ContingencyResponse {
    RetryStep(String),
    SkipStep(String),
    FindAlternative(String),
    ReplanFromCurrent,
    AbandonPlan,
    SeekHelp(String), // NPC ID to ask for help
}

#[derive(Debug, Clone)]
pub struct CompletedGoal {
    pub goal: Goal,
    pub completion_time: u64,
    pub actual_duration: f32,
    pub success_factors: Vec<String>,
    pub lessons_learned: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FailedGoal {
    pub goal: Goal,
    pub failure_time: u64,
    pub failure_reasons: Vec<String>,
    pub attempted_solutions: Vec<String>,
    pub lessons_learned: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum PlanningStyle {
    Methodical,     // Careful, detailed planning
    Opportunistic,  // Adapt to circumstances
    Aggressive,     // Take risks for faster results
    Collaborative,  // Involve others in planning
    Reactive,       // Minimal forward planning
}

#[derive(Debug, Clone)]
pub struct GoalTemplate {
    pub template_id: String,
    pub goal_type: GoalType,
    pub typical_priority: f32,
    pub average_duration: f32,
    pub required_skills: HashMap<String, f32>,
    pub common_obstacles: Vec<String>,
    pub success_strategies: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PlanningAlgorithms {
    pub default_algorithm: String,
    pub specialized_algorithms: HashMap<GoalType, String>,
}

#[derive(Debug, Clone)]
pub struct UtilityCalculator {
    pub utility_functions: HashMap<String, UtilityFunction>,
    pub weighting_schemes: HashMap<String, WeightingScheme>,
    pub context_modifiers: HashMap<String, f32>,
}

#[derive(Debug, Clone)]
pub struct UtilityFunction {
    pub function_name: String,
    pub parameters: HashMap<String, f32>,
    pub curve_type: UtilityCurveType,
}

#[derive(Debug, Clone)]
pub enum UtilityCurveType {
    Linear,
    Exponential,
    Logarithmic,
    Sigmoid,
    Custom(Vec<(f32, f32)>), // (input, output) pairs
}

#[derive(Debug, Clone)]
pub struct WeightingScheme {
    pub scheme_name: String,
    pub factor_weights: HashMap<String, f32>,
    pub personality_influence: HashMap<String, f32>,
}

#[derive(Debug, Clone)]
pub struct KnowledgeBase {
    pub facts: HashMap<String, Fact>,
    pub rules: Vec<Rule>,
    pub patterns: Vec<Pattern>,
    pub world_model: WorldModel,
}

#[derive(Debug, Clone)]
pub struct Fact {
    pub fact_id: String,
    pub content: String,
    pub confidence: f32,
    pub source: String,
    pub timestamp: u64,
    pub relevance_contexts: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub rule_id: String,
    pub condition: RuleCondition,
    pub conclusion: RuleConclusion,
    pub confidence: f32,
    pub usage_count: u32,
}

#[derive(Debug, Clone)]
pub enum RuleCondition {
    FactExists(String),
    FactValue(String, ComparisonType, f32),
    MultipleConditions(Vec<RuleCondition>, LogicalOperator),
}

#[derive(Debug, Clone)]
pub enum LogicalOperator {
    And,
    Or,
    Not,
}

#[derive(Debug, Clone)]
pub enum RuleConclusion {
    AddFact(Fact),
    ModifyFact(String, f32), // fact_id, new_confidence
    TriggerAction(DecisionAction),
    SetGoalPriority(String, f32),
}

#[derive(Debug, Clone)]
pub struct Pattern {
    pub pattern_id: String,
    pub pattern_type: PatternType,
    pub conditions: Vec<String>,
    pub typical_outcomes: Vec<String>,
    pub confidence: f32,
}

#[derive(Debug, Clone)]
pub enum PatternType {
    Behavioral,
    Social,
    Environmental,
    Temporal,
    Causal,
}

#[derive(Debug, Clone)]
pub struct WorldModel {
    pub locations: HashMap<String, LocationInfo>,
    pub npcs: HashMap<String, NPCInfo>,
    pub objects: HashMap<String, ObjectInfo>,
    pub relationships: HashMap<String, Vec<String>>, // relationship_type -> npc_ids
    pub temporal_patterns: Vec<TemporalPattern>,
}

#[derive(Debug, Clone)]
pub struct LocationInfo {
    pub location_type: String,
    pub accessibility: f32,
    pub safety_level: f32,
    pub resources: Vec<String>,
    pub typical_occupants: Vec<String>,
    pub activities: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct NPCInfo {
    pub npc_id: String,
    pub typical_behavior: Vec<String>,
    pub skills: HashMap<String, f32>,
    pub preferences: HashMap<String, f32>,
    pub reliability: f32,
    pub interaction_history: Vec<InteractionRecord>,
}

#[derive(Debug, Clone)]
pub struct ObjectInfo {
    pub object_id: String,
    pub object_type: String,
    pub functionality: Vec<String>,
    pub requirements: Vec<String>,
    pub availability: f32,
}

#[derive(Debug, Clone)]
pub struct InteractionRecord {
    pub timestamp: u64,
    pub interaction_type: String,
    pub outcome: String,
    pub satisfaction: f32,
}

#[derive(Debug, Clone)]
pub struct TemporalPattern {
    pub pattern_name: String,
    pub time_conditions: Vec<String>,
    pub typical_events: Vec<String>,
    pub probability: f32,
}

#[derive(Debug, Clone)]
pub struct LearningSystem {
    pub learning_algorithms: HashMap<String, LearningAlgorithm>,
    pub experience_buffer: HashMap<String, Vec<Experience>>, // npc_id -> experiences
    pub adaptation_rates: HashMap<String, f32>,
    pub learning_goals: Vec<LearningGoal>,
}

#[derive(Debug, Clone)]
pub struct LearningAlgorithm {
    pub algorithm_name: String,
    pub learning_rate: f32,
    pub memory_decay: f32,
    pub generalization_factor: f32,
}

#[derive(Debug, Clone)]
pub struct Experience {
    pub experience_id: String,
    pub context: ExperienceContext,
    pub actions_taken: Vec<String>,
    pub outcomes: Vec<String>,
    pub success_rating: f32,
    pub timestamp: u64,
    pub lessons: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ExperienceContext {
    pub situation_type: String,
    pub participants: Vec<String>,
    pub location: Point3,
    pub initial_conditions: HashMap<String, f32>,
    pub goals_at_time: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LearningGoal {
    pub goal_id: String,
    pub skill_to_improve: String,
    pub target_proficiency: f32,
    pub learning_method: LearningMethod,
    pub progress: f32,
}

#[derive(Debug, Clone)]
pub enum LearningMethod {
    Practice,
    Observation,
    Teaching,
    Experimentation,
    Collaboration,
}

impl AIDecisionEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            decision_trees: HashMap::new(),
            goal_planner: GoalPlanner::new(),
            utility_calculator: UtilityCalculator::new(),
            knowledge_base: KnowledgeBase::new(),
            learning_system: LearningSystem::new(),
        };
        
        engine.initialize_decision_trees();
        engine.initialize_goal_templates();
        engine
    }

    fn initialize_decision_trees(&mut self) {
        // Survival decision tree
        let survival_tree = DecisionTree {
            root: DecisionNode::Condition {
                condition: DecisionCondition::NeedLevel("energy".to_string(), 20.0, ComparisonType::Less),
                true_branch: Box::new(DecisionNode::Action {
                    action: DecisionAction::ExecuteBehavior(BehaviorType::Sleep),
                    confidence: 0.9,
                }),
                false_branch: Box::new(DecisionNode::Condition {
                    condition: DecisionCondition::NeedLevel("hunger".to_string(), 80.0, ComparisonType::Greater),
                    true_branch: Box::new(DecisionNode::Action {
                        action: DecisionAction::ExecuteBehavior(BehaviorType::Eat),
                        confidence: 0.85,
                    }),
                    false_branch: Box::new(DecisionNode::Action {
                        action: DecisionAction::Wait(1.0),
                        confidence: 0.3,
                    }),
                }),
            },
            context: DecisionContext::Emergency,
            success_rate: 0.8,
            usage_count: 0,
        };

        self.decision_trees.insert("survival".to_string(), survival_tree);

        // Work decision tree
        let work_tree = DecisionTree {
            root: DecisionNode::Condition {
                condition: DecisionCondition::TimeOfDay(8, 17), // Work hours
                true_branch: Box::new(DecisionNode::Condition {
                    condition: DecisionCondition::NeedLevel("energy".to_string(), 30.0, ComparisonType::Greater),
                    true_branch: Box::new(DecisionNode::Action {
                        action: DecisionAction::ExecuteBehavior(BehaviorType::Work),
                        confidence: 0.8,
                    }),
                    false_branch: Box::new(DecisionNode::Action {
                        action: DecisionAction::ExecuteBehavior(BehaviorType::Rest),
                        confidence: 0.6,
                    }),
                }),
                false_branch: Box::new(DecisionNode::Action {
                    action: DecisionAction::Wait(1.0),
                    confidence: 0.4,
                }),
            },
            context: DecisionContext::Work,
            success_rate: 0.7,
            usage_count: 0,
        };

        self.decision_trees.insert("work".to_string(), work_tree);

        // Social decision tree
        let social_tree = DecisionTree {
            root: DecisionNode::Condition {
                condition: DecisionCondition::SocialContext(SocialContextType::NPCNearby(10.0)),
                true_branch: Box::new(DecisionNode::Condition {
                    condition: DecisionCondition::EmotionalState(EmotionalCondition::MoodAbove(40.0)),
                    true_branch: Box::new(DecisionNode::RandomChoice {
                        choices: vec![
                            (DecisionNode::Action {
                                action: DecisionAction::ExecuteBehavior(BehaviorType::Talk),
                                confidence: 0.7,
                            }, 0.6),
                            (DecisionNode::Action {
                                action: DecisionAction::ExecuteBehavior(BehaviorType::Greet),
                                confidence: 0.8,
                            }, 0.4),
                        ],
                    }),
                    false_branch: Box::new(DecisionNode::Action {
                        action: DecisionAction::Wait(0.5),
                        confidence: 0.5,
                    }),
                }),
                false_branch: Box::new(DecisionNode::Action {
                    action: DecisionAction::Explore(Point3::new(0.0, 0.0, 0.0)), // Random exploration
                    confidence: 0.3,
                }),
            },
            context: DecisionContext::Social,
            success_rate: 0.6,
            usage_count: 0,
        };

        self.decision_trees.insert("social".to_string(), social_tree);
    }

    fn initialize_goal_templates(&mut self) {
        let templates = vec![
            GoalTemplate {
                template_id: "build_shelter".to_string(),
                goal_type: GoalType::Survival,
                typical_priority: 8.0,
                average_duration: 240.0, // 4 hours
                required_skills: {
                    let mut skills = HashMap::new();
                    skills.insert("construction".to_string(), 20.0);
                    skills
                },
                common_obstacles: vec!["lack_materials".to_string(), "bad_weather".to_string()],
                success_strategies: vec!["gather_materials_first".to_string(), "find_helpers".to_string()],
            },
            GoalTemplate {
                template_id: "make_friend".to_string(),
                goal_type: GoalType::Social,
                typical_priority: 5.0,
                average_duration: 1440.0, // 1 day
                required_skills: {
                    let mut skills = HashMap::new();
                    skills.insert("social".to_string(), 15.0);
                    skills
                },
                common_obstacles: vec!["shyness".to_string(), "language_barrier".to_string()],
                success_strategies: vec!["find_common_interests".to_string(), "be_helpful".to_string()],
            },
        ];

        for template in templates {
            self.goal_planner.goal_templates.insert(template.template_id.clone(), template);
        }
    }

    pub fn make_decision(&mut self, npc: &NPC, context: DecisionContext, world_time: u32) -> Option<DecisionAction> {
        // Select appropriate decision tree based on context
        let tree_name = match context {
            DecisionContext::Emergency => "survival",
            DecisionContext::Work => "work",
            DecisionContext::Social => "social",
            _ => "survival", // Default fallback
        };

        if let Some(tree) = self.decision_trees.get_mut(tree_name) {
            tree.usage_count += 1;
            let root_node = tree.root.clone();
            drop(tree); // Release mutable borrow

            let action = self.evaluate_decision_node(&root_node, npc, world_time);

            // Learn from the decision
            if let Some(ref action) = action {
                self.record_decision_experience(npc.id.clone(), action.clone(), &context);
            }

            action
        } else {
            None
        }
    }

    fn evaluate_decision_node(&self, node: &DecisionNode, npc: &NPC, world_time: u32) -> Option<DecisionAction> {
        match node {
            DecisionNode::Condition { condition, true_branch, false_branch } => {
                if self.evaluate_condition(condition, npc, world_time) {
                    self.evaluate_decision_node(true_branch, npc, world_time)
                } else {
                    self.evaluate_decision_node(false_branch, npc, world_time)
                }
            },
            
            DecisionNode::Action { action, confidence: _ } => {
                Some(action.clone())
            },
            
            DecisionNode::RandomChoice { choices } => {
                // Simple random selection (would use proper weighted random in real implementation)
                if let Some((node, _)) = choices.first() {
                    self.evaluate_decision_node(node, npc, world_time)
                } else {
                    None
                }
            },
            
            DecisionNode::Sequence { steps } => {
                // Return the first step in the sequence
                if let Some(first_step) = steps.first() {
                    self.evaluate_decision_node(first_step, npc, world_time)
                } else {
                    None
                }
            },
        }
    }

    fn evaluate_condition(&self, condition: &DecisionCondition, npc: &NPC, world_time: u32) -> bool {
        match condition {
            DecisionCondition::NeedLevel(need, threshold, comparison) => {
                let value = match need.as_str() {
                    "energy" => npc.energy,
                    "hunger" => npc.hunger,
                    "mood" => npc.mood,
                    "stress" => npc.stress,
                    _ => 50.0, // Default
                };
                
                match comparison {
                    ComparisonType::Greater => value > *threshold,
                    ComparisonType::Less => value < *threshold,
                    ComparisonType::Equal => (value - threshold).abs() < 1.0,
                    ComparisonType::GreaterOrEqual => value >= *threshold,
                    ComparisonType::LessOrEqual => value <= *threshold,
                }
            },
            
            DecisionCondition::TimeOfDay(start, end) => {
                let hour = (world_time / 60) % 24;
                if start <= end {
                    hour >= *start && hour <= *end
                } else {
                    // Handle overnight periods
                    hour >= *start || hour <= *end
                }
            },
            
            DecisionCondition::SocialContext(context_type) => {
                match context_type {
                    SocialContextType::NPCNearby(_radius) => !npc.visible_npcs.is_empty(),
                    SocialContextType::InConversation => matches!(npc.state, crate::engine::npc::NPCState::Socializing),
                    SocialContextType::Alone => npc.visible_npcs.is_empty(),
                    _ => false, // Simplified
                }
            },
            
            DecisionCondition::EmotionalState(emotion_condition) => {
                match emotion_condition {
                    EmotionalCondition::MoodAbove(threshold) => npc.mood > *threshold,
                    EmotionalCondition::MoodBelow(threshold) => npc.mood < *threshold,
                    EmotionalCondition::StressAbove(threshold) => npc.stress > *threshold,
                    EmotionalCondition::StressBelow(threshold) => npc.stress < *threshold,
                    _ => false,
                }
            },
            
            DecisionCondition::SkillLevel(skill, min_level) => {
                npc.skills.get(skill).unwrap_or(&0.0) >= min_level
            },
            
            _ => false, // Simplified for other conditions
        }
    }

    fn record_decision_experience(&mut self, npc_id: String, action: DecisionAction, context: &DecisionContext) {
        let experience = Experience {
            experience_id: format!("exp_{}_{}", npc_id, std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()),
            context: ExperienceContext {
                situation_type: format!("{:?}", context),
                participants: vec![npc_id.clone()],
                location: Point3::new(0.0, 0.0, 0.0), // Would get actual location
                initial_conditions: HashMap::new(),
                goals_at_time: Vec::new(),
            },
            actions_taken: vec![format!("{:?}", action)],
            outcomes: Vec::new(), // Would be filled later based on results
            success_rating: 0.5, // Placeholder
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            lessons: Vec::new(),
        };

        self.learning_system.experience_buffer
            .entry(npc_id)
            .or_insert_with(Vec::new)
            .push(experience);
    }

    pub fn plan_goal_achievement(&mut self, npc: &NPC, goal: Goal) -> Option<Plan> {
        let npc_planner = self.goal_planner.active_planners
            .entry(npc.id.clone())
            .or_insert_with(|| NPCGoalPlanner::new(npc.id.clone()));

        // Simple planning algorithm
        let mut steps = Vec::new();
        
        match goal.goal_type {
            GoalType::Survival => {
                if goal.description.contains("shelter") {
                    steps.push(PlanStep {
                        step_id: "gather_materials".to_string(),
                        action: PlanAction::Gather("wood".to_string(), 10),
                        prerequisites: Vec::new(),
                        expected_outcomes: vec!["have_building_materials".to_string()],
                        duration: 60.0,
                        failure_conditions: vec!["no_wood_available".to_string()],
                    });
                    
                    steps.push(PlanStep {
                        step_id: "find_location".to_string(),
                        action: PlanAction::MoveTo(Point3::new(50.0, 0.0, 50.0)), // Suitable building location
                        prerequisites: vec!["gather_materials".to_string()],
                        expected_outcomes: vec!["at_building_site".to_string()],
                        duration: 30.0,
                        failure_conditions: vec!["location_unavailable".to_string()],
                    });
                    
                    steps.push(PlanStep {
                        step_id: "build_shelter".to_string(),
                        action: PlanAction::Build("simple_shelter".to_string()),
                        prerequisites: vec!["gather_materials".to_string(), "find_location".to_string()],
                        expected_outcomes: vec!["shelter_completed".to_string()],
                        duration: 120.0,
                        failure_conditions: vec!["insufficient_skill".to_string(), "bad_weather".to_string()],
                    });
                }
            },
            
            GoalType::Social => {
                if goal.description.contains("friend") {
                    steps.push(PlanStep {
                        step_id: "find_social_opportunity".to_string(),
                        action: PlanAction::MoveTo(Point3::new(75.0, 0.0, 75.0)), // Social area
                        prerequisites: Vec::new(),
                        expected_outcomes: vec!["in_social_area".to_string()],
                        duration: 15.0,
                        failure_conditions: vec!["no_people_around".to_string()],
                    });
                    
                    steps.push(PlanStep {
                        step_id: "initiate_conversation".to_string(),
                        action: PlanAction::Communicate("nearby_npc".to_string(), "friendly_greeting".to_string()),
                        prerequisites: vec!["find_social_opportunity".to_string()],
                        expected_outcomes: vec!["conversation_started".to_string()],
                        duration: 30.0,
                        failure_conditions: vec!["npc_unavailable".to_string(), "too_shy".to_string()],
                    });
                    
                    steps.push(PlanStep {
                        step_id: "build_rapport".to_string(),
                        action: PlanAction::Communicate("target_npc".to_string(), "share_interests".to_string()),
                        prerequisites: vec!["initiate_conversation".to_string()],
                        expected_outcomes: vec!["friendship_formed".to_string()],
                        duration: 180.0,
                        failure_conditions: vec!["incompatible_personalities".to_string()],
                    });
                }
            },
            
            _ => {
                // Generic goal planning
                steps.push(PlanStep {
                    step_id: "generic_action".to_string(),
                    action: PlanAction::Wait(60.0),
                    prerequisites: Vec::new(),
                    expected_outcomes: vec!["time_passed".to_string()],
                    duration: 60.0,
                    failure_conditions: Vec::new(),
                });
            },
        }

        if steps.is_empty() {
            return None;
        }

        let total_duration: f32 = steps.iter().map(|s| s.duration).sum();
        
        let plan = Plan {
            plan_id: format!("plan_{}_{}", npc.id, goal.id),
            main_goal: goal,
            steps,
            estimated_duration: total_duration,
            success_probability: 0.7, // Would calculate based on NPC skills, etc.
            required_resources: HashMap::new(),
            contingencies: Vec::new(),
        };

        npc_planner.current_plan = Some(plan.clone());
        Some(plan)
    }

    pub fn calculate_utility(&self, action: &DecisionAction, npc: &NPC) -> f32 {
        // Simple utility calculation
        match action {
            DecisionAction::ExecuteBehavior(behavior_type) => {
                match behavior_type {
                    BehaviorType::Sleep if npc.energy < 30.0 => 0.9,
                    BehaviorType::Eat if npc.hunger > 70.0 => 0.8,
                    BehaviorType::Talk if !npc.visible_npcs.is_empty() => 0.6,
                    BehaviorType::Work if npc.energy > 40.0 => 0.7,
                    _ => 0.3,
                }
            },
            DecisionAction::Wait(_) => 0.1,
            DecisionAction::Explore(_) => 0.4,
            _ => 0.5,
        }
    }

    pub fn update_learning(&mut self, npc_id: &str, success: bool, action: &DecisionAction) {
        if let Some(experiences) = self.learning_system.experience_buffer.get_mut(npc_id) {
            if let Some(last_experience) = experiences.last_mut() {
                last_experience.success_rating = if success { 0.8 } else { 0.2 };
                last_experience.outcomes.push(if success {
                    "success".to_string()
                } else {
                    "failure".to_string()
                });
                
                // Simple learning: adjust decision tree success rates
                for tree in self.decision_trees.values_mut() {
                    if success {
                        tree.success_rate = (tree.success_rate * 0.9 + 0.1).min(0.95);
                    } else {
                        tree.success_rate = (tree.success_rate * 0.9).max(0.1);
                    }
                }
            }
        }
    }

    pub fn get_current_goals(&self, npc_id: &str) -> Vec<Goal> {
        if let Some(planner) = self.goal_planner.active_planners.get(npc_id) {
            planner.goal_stack.clone()
        } else {
            Vec::new()
        }
    }
}

impl GoalPlanner {
    pub fn new() -> Self {
        Self {
            active_planners: HashMap::new(),
            goal_templates: HashMap::new(),
            planning_algorithms: PlanningAlgorithms {
                default_algorithm: "simple_forward".to_string(),
                specialized_algorithms: HashMap::new(),
            },
        }
    }
}

impl NPCGoalPlanner {
    pub fn new(npc_id: String) -> Self {
        Self {
            npc_id,
            current_plan: None,
            goal_stack: Vec::new(),
            completed_goals: Vec::new(),
            failed_goals: Vec::new(),
            planning_style: PlanningStyle::Methodical,
            risk_tolerance: 0.5,
        }
    }
}

impl UtilityCalculator {
    pub fn new() -> Self {
        Self {
            utility_functions: HashMap::new(),
            weighting_schemes: HashMap::new(),
            context_modifiers: HashMap::new(),
        }
    }
}

impl KnowledgeBase {
    pub fn new() -> Self {
        Self {
            facts: HashMap::new(),
            rules: Vec::new(),
            patterns: Vec::new(),
            world_model: WorldModel {
                locations: HashMap::new(),
                npcs: HashMap::new(),
                objects: HashMap::new(),
                relationships: HashMap::new(),
                temporal_patterns: Vec::new(),
            },
        }
    }
}

impl LearningSystem {
    pub fn new() -> Self {
        Self {
            learning_algorithms: HashMap::new(),
            experience_buffer: HashMap::new(),
            adaptation_rates: HashMap::new(),
            learning_goals: Vec::new(),
        }
    }
}