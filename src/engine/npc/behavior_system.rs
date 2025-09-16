use crate::engine::math::{Vec3, Point3};
use crate::engine::npc::{NPC, Behavior, BehaviorType, BehaviorCondition, BehaviorEffect};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct BehaviorSystem {
    behavior_templates: HashMap<String, BehaviorTemplate>,
    behavior_trees: HashMap<String, BehaviorTree>,
    global_behavior_modifiers: HashMap<String, f32>,
    environmental_factors: EnvironmentalFactors,
}

#[derive(Debug, Clone)]
pub struct BehaviorTemplate {
    pub name: String,
    pub base_priority: f32,
    pub duration_range: (f32, f32),
    pub energy_cost: f32,
    pub mood_requirements: Option<(f32, f32)>,
    pub skill_requirements: HashMap<String, f32>,
    pub personality_influences: HashMap<String, f32>,
    pub conditions: Vec<BehaviorCondition>,
    pub effects: Vec<BehaviorEffect>,
    pub compatible_occupations: Vec<String>,
    pub time_of_day_preference: Option<(u32, u32)>,
}

#[derive(Debug, Clone)]
pub struct BehaviorTree {
    pub root: BehaviorNode,
    pub variables: HashMap<String, f32>,
}

#[derive(Debug, Clone)]
pub enum BehaviorNode {
    Sequence(Vec<BehaviorNode>),
    Selector(Vec<BehaviorNode>),
    Parallel(Vec<BehaviorNode>),
    Decorator(DecoratorType, Box<BehaviorNode>),
    Action(String), // References behavior template
    Condition(ConditionType),
}

#[derive(Debug, Clone)]
pub enum DecoratorType {
    Repeat(u32),
    UntilSuccess,
    UntilFailure,
    Invert,
    Cooldown(f32),
    Probability(f32),
}

#[derive(Debug, Clone)]
pub enum ConditionType {
    EnergyLevel(f32, f32), // min, max
    MoodLevel(f32, f32),
    TimeOfDay(u32, u32),
    NPCNearby(f32), // radius
    HasSkill(String, f32),
    LocationDistance(Point3, f32),
    RelationshipLevel(String, f32),
}

#[derive(Debug, Clone)]
pub struct EnvironmentalFactors {
    pub weather: Weather,
    pub time_of_day: u32,
    pub season: Season,
    pub population_density: f32,
    pub noise_level: f32,
    pub safety_level: f32,
    pub resource_availability: HashMap<String, f32>,
}

#[derive(Debug, Clone)]
pub enum Weather {
    Sunny,
    Cloudy,
    Rainy,
    Stormy,
    Snowy,
    Foggy,
}

#[derive(Debug, Clone)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

#[derive(Debug, Clone)]
pub struct BehaviorExecutionContext {
    pub npc_id: String,
    pub current_behavior: Option<String>,
    pub execution_time: f32,
    pub interruption_count: u32,
    pub success_rate: f32,
    pub adaptation_factors: HashMap<String, f32>,
}

impl BehaviorSystem {
    pub fn new() -> Self {
        let mut system = Self {
            behavior_templates: HashMap::new(),
            behavior_trees: HashMap::new(),
            global_behavior_modifiers: HashMap::new(),
            environmental_factors: EnvironmentalFactors::default(),
        };
        
        system.initialize_default_behaviors();
        system.initialize_behavior_trees();
        system
    }

    fn initialize_default_behaviors(&mut self) {
        // Basic survival behaviors
        self.add_behavior_template(BehaviorTemplate {
            name: "sleep".to_string(),
            base_priority: 80.0,
            duration_range: (300.0, 480.0), // 5-8 minutes
            energy_cost: -50.0, // Restores energy
            mood_requirements: None,
            skill_requirements: HashMap::new(),
            personality_influences: {
                let mut influences = HashMap::new();
                influences.insert("conscientiousness".to_string(), 0.2);
                influences
            },
            conditions: vec![
                BehaviorCondition::EnergyBelow(30.0),
                BehaviorCondition::TimeOfDay(22, 6), // 10 PM to 6 AM
            ],
            effects: vec![
                BehaviorEffect::ChangeEnergy(60.0),
                BehaviorEffect::ChangeStress(-20.0),
                BehaviorEffect::ChangeMood(10.0),
            ],
            compatible_occupations: vec![], // All occupations need sleep
            time_of_day_preference: Some((22, 6)),
        });

        self.add_behavior_template(BehaviorTemplate {
            name: "eat".to_string(),
            base_priority: 75.0,
            duration_range: (30.0, 90.0), // 30 seconds to 1.5 minutes
            energy_cost: 5.0,
            mood_requirements: None,
            skill_requirements: HashMap::new(),
            personality_influences: HashMap::new(),
            conditions: vec![
                BehaviorCondition::EnergyBelow(80.0), // Don't eat when full of energy
            ],
            effects: vec![
                BehaviorEffect::ChangeHunger(-60.0),
                BehaviorEffect::ChangeMood(15.0),
                BehaviorEffect::ChangeEnergy(10.0),
            ],
            compatible_occupations: vec![],
            time_of_day_preference: Some((7, 19)), // 7 AM to 7 PM
        });

        // Work behaviors
        self.add_behavior_template(BehaviorTemplate {
            name: "build".to_string(),
            base_priority: 60.0,
            duration_range: (120.0, 300.0), // 2-5 minutes
            energy_cost: 20.0,
            mood_requirements: Some((20.0, 100.0)),
            skill_requirements: {
                let mut skills = HashMap::new();
                skills.insert("construction".to_string(), 10.0);
                skills
            },
            personality_influences: {
                let mut influences = HashMap::new();
                influences.insert("conscientiousness".to_string(), 0.3);
                influences.insert("creativity".to_string(), 0.2);
                influences
            },
            conditions: vec![
                BehaviorCondition::EnergyAbove(30.0),
                BehaviorCondition::TimeOfDay(8, 18), // 8 AM to 6 PM
            ],
            effects: vec![
                BehaviorEffect::ChangeEnergy(-15.0),
                BehaviorEffect::LearnSkill("construction".to_string(), 1.0),
                BehaviorEffect::ChangeMood(5.0),
                BehaviorEffect::CreateMemory("Built something".to_string(), 0.3),
            ],
            compatible_occupations: vec!["Builder".to_string(), "Craftsperson".to_string()],
            time_of_day_preference: Some((8, 18)),
        });

        // Social behaviors
        self.add_behavior_template(BehaviorTemplate {
            name: "socialize".to_string(),
            base_priority: 45.0,
            duration_range: (60.0, 180.0), // 1-3 minutes
            energy_cost: 10.0,
            mood_requirements: Some((30.0, 100.0)),
            skill_requirements: HashMap::new(),
            personality_influences: {
                let mut influences = HashMap::new();
                influences.insert("extroversion".to_string(), 0.4);
                influences.insert("agreeableness".to_string(), 0.2);
                influences
            },
            conditions: vec![
                BehaviorCondition::NPCNearby("any".to_string(), 10.0),
                BehaviorCondition::EnergyAbove(20.0),
            ],
            effects: vec![
                BehaviorEffect::ChangeMood(20.0),
                BehaviorEffect::ChangeStress(-15.0),
                BehaviorEffect::LearnSkill("social".to_string(), 0.5),
            ],
            compatible_occupations: vec![],
            time_of_day_preference: Some((10, 22)),
        });

        // Entertainment behaviors
        self.add_behavior_template(BehaviorTemplate {
            name: "explore".to_string(),
            base_priority: 35.0,
            duration_range: (180.0, 600.0), // 3-10 minutes
            energy_cost: 25.0,
            mood_requirements: Some((40.0, 100.0)),
            skill_requirements: HashMap::new(),
            personality_influences: {
                let mut influences = HashMap::new();
                influences.insert("openness".to_string(), 0.5);
                influences.insert("curiosity".to_string(), 0.3);
                influences
            },
            conditions: vec![
                BehaviorCondition::EnergyAbove(40.0),
                BehaviorCondition::MoodAbove(40.0),
            ],
            effects: vec![
                BehaviorEffect::ChangeMood(10.0),
                BehaviorEffect::LearnSkill("exploration".to_string(), 1.5),
                BehaviorEffect::CreateMemory("Discovered something new".to_string(), 0.6),
                BehaviorEffect::ChangeStress(-10.0),
            ],
            compatible_occupations: vec!["Explorer".to_string()],
            time_of_day_preference: Some((9, 17)),
        });
    }

    fn initialize_behavior_trees(&mut self) {
        // Create a basic daily routine behavior tree
        let daily_routine_tree = BehaviorTree {
            root: BehaviorNode::Selector(vec![
                // Emergency needs (highest priority)
                BehaviorNode::Sequence(vec![
                    BehaviorNode::Condition(ConditionType::EnergyLevel(0.0, 20.0)),
                    BehaviorNode::Action("sleep".to_string()),
                ]),
                
                // Basic needs
                BehaviorNode::Sequence(vec![
                    BehaviorNode::Condition(ConditionType::EnergyLevel(0.0, 40.0)),
                    BehaviorNode::Condition(ConditionType::TimeOfDay(7, 22)),
                    BehaviorNode::Action("eat".to_string()),
                ]),
                
                // Work during work hours
                BehaviorNode::Sequence(vec![
                    BehaviorNode::Condition(ConditionType::TimeOfDay(8, 17)),
                    BehaviorNode::Condition(ConditionType::EnergyLevel(30.0, 100.0)),
                    BehaviorNode::Selector(vec![
                        BehaviorNode::Action("build".to_string()),
                        BehaviorNode::Action("work".to_string()),
                    ]),
                ]),
                
                // Social activities
                BehaviorNode::Sequence(vec![
                    BehaviorNode::Condition(ConditionType::NPCNearby(15.0)),
                    BehaviorNode::Condition(ConditionType::MoodLevel(30.0, 100.0)),
                    BehaviorNode::Decorator(
                        DecoratorType::Probability(0.7),
                        Box::new(BehaviorNode::Action("socialize".to_string()))
                    ),
                ]),
                
                // Exploration and leisure
                BehaviorNode::Sequence(vec![
                    BehaviorNode::Condition(ConditionType::EnergyLevel(40.0, 100.0)),
                    BehaviorNode::Condition(ConditionType::TimeOfDay(10, 20)),
                    BehaviorNode::Action("explore".to_string()),
                ]),
                
                // Default idle behavior
                BehaviorNode::Action("idle".to_string()),
            ]),
            variables: HashMap::new(),
        };

        self.behavior_trees.insert("daily_routine".to_string(), daily_routine_tree);

        // Create specialized behavior trees for different occupations
        self.create_occupation_behavior_trees();
    }

    fn create_occupation_behavior_trees(&mut self) {
        // Builder behavior tree
        let builder_tree = BehaviorTree {
            root: BehaviorNode::Selector(vec![
                // Emergency needs
                BehaviorNode::Sequence(vec![
                    BehaviorNode::Condition(ConditionType::EnergyLevel(0.0, 25.0)),
                    BehaviorNode::Action("sleep".to_string()),
                ]),
                
                // Work focus during work hours
                BehaviorNode::Sequence(vec![
                    BehaviorNode::Condition(ConditionType::TimeOfDay(8, 17)),
                    BehaviorNode::Condition(ConditionType::EnergyLevel(30.0, 100.0)),
                    BehaviorNode::Decorator(
                        DecoratorType::Repeat(3),
                        Box::new(BehaviorNode::Action("build".to_string()))
                    ),
                ]),
                
                // Skill improvement during off hours
                BehaviorNode::Sequence(vec![
                    BehaviorNode::Condition(ConditionType::TimeOfDay(18, 22)),
                    BehaviorNode::Action("practice_building".to_string()),
                ]),
                
                // Default to daily routine
                BehaviorNode::Action("daily_routine".to_string()),
            ]),
            variables: HashMap::new(),
        };

        self.behavior_trees.insert("builder".to_string(), builder_tree);

        // Explorer behavior tree
        let explorer_tree = BehaviorTree {
            root: BehaviorNode::Selector(vec![
                // Basic needs
                BehaviorNode::Sequence(vec![
                    BehaviorNode::Condition(ConditionType::EnergyLevel(0.0, 30.0)),
                    BehaviorNode::Action("sleep".to_string()),
                ]),
                
                // Primary exploration behavior
                BehaviorNode::Sequence(vec![
                    BehaviorNode::Condition(ConditionType::EnergyLevel(40.0, 100.0)),
                    BehaviorNode::Condition(ConditionType::TimeOfDay(6, 20)),
                    BehaviorNode::Parallel(vec![
                        BehaviorNode::Action("explore".to_string()),
                        BehaviorNode::Decorator(
                            DecoratorType::Probability(0.3),
                            Box::new(BehaviorNode::Action("document_findings".to_string()))
                        ),
                    ]),
                ]),
                
                // Social sharing of discoveries
                BehaviorNode::Sequence(vec![
                    BehaviorNode::Condition(ConditionType::NPCNearby(10.0)),
                    BehaviorNode::Action("share_discoveries".to_string()),
                ]),
                
                BehaviorNode::Action("daily_routine".to_string()),
            ]),
            variables: HashMap::new(),
        };

        self.behavior_trees.insert("explorer".to_string(), explorer_tree);
    }

    pub fn update(&mut self, npcs: &mut HashMap<String, NPC>, delta_time: f32, world_time: u32) {
        // Update environmental factors
        self.update_environmental_factors(world_time);
        
        // Process behavior trees for each NPC
        for (npc_id, npc) in npcs.iter_mut() {
            self.process_npc_behavior_tree(npc, delta_time, world_time);
        }
        
        // Update global behavior modifiers
        self.update_global_modifiers(delta_time);
    }

    fn update_environmental_factors(&mut self, world_time: u32) {
        self.environmental_factors.time_of_day = world_time;
        
        // Simple weather simulation
        if world_time % 720 == 0 { // Every 12 hours, potentially change weather
            self.environmental_factors.weather = match rand::random::<f32>() {
                x if x < 0.4 => Weather::Sunny,
                x if x < 0.6 => Weather::Cloudy,
                x if x < 0.8 => Weather::Rainy,
                x if x < 0.9 => Weather::Stormy,
                _ => Weather::Foggy,
            };
        }
        
        // Adjust other factors based on weather
        match self.environmental_factors.weather {
            Weather::Rainy | Weather::Stormy => {
                self.global_behavior_modifiers.insert("outdoor_activity".to_string(), 0.3);
                self.global_behavior_modifiers.insert("social_activity".to_string(), 0.7);
            },
            Weather::Sunny => {
                self.global_behavior_modifiers.insert("outdoor_activity".to_string(), 1.2);
                self.global_behavior_modifiers.insert("exploration".to_string(), 1.3);
            },
            _ => {
                // Reset modifiers for neutral weather
                self.global_behavior_modifiers.insert("outdoor_activity".to_string(), 1.0);
                self.global_behavior_modifiers.insert("social_activity".to_string(), 1.0);
            },
        }
    }

    fn process_npc_behavior_tree(&self, npc: &mut NPC, delta_time: f32, world_time: u32) {
        // Determine which behavior tree to use
        let tree_name = match npc.occupation {
            crate::engine::npc::Occupation::Builder => "builder",
            crate::engine::npc::Occupation::Explorer => "explorer",
            _ => "daily_routine",
        };

        if let Some(tree) = self.behavior_trees.get(tree_name) {
            if let Some(behavior_name) = self.evaluate_behavior_tree(&tree.root, npc, world_time) {
                if let Some(template) = self.behavior_templates.get(&behavior_name) {
                    let behavior = self.create_behavior_from_template(template, npc, world_time);
                    
                    // Only queue if priority is high enough or no current behavior
                    if npc.current_behavior.is_none() || 
                       behavior.priority > npc.current_behavior.as_ref().unwrap().priority {
                        npc.queue_behavior(behavior);
                    }
                }
            }
        }
    }

    fn evaluate_behavior_tree(&self, node: &BehaviorNode, npc: &NPC, world_time: u32) -> Option<String> {
        match node {
            BehaviorNode::Sequence(children) => {
                for child in children {
                    match self.evaluate_behavior_tree(child, npc, world_time) {
                        Some(result) => return Some(result),
                        None => return None, // Sequence fails if any child fails
                    }
                }
                None
            },
            
            BehaviorNode::Selector(children) => {
                for child in children {
                    if let Some(result) = self.evaluate_behavior_tree(child, npc, world_time) {
                        return Some(result);
                    }
                }
                None
            },
            
            BehaviorNode::Parallel(children) => {
                // Return first successful behavior
                for child in children {
                    if let Some(result) = self.evaluate_behavior_tree(child, npc, world_time) {
                        return Some(result);
                    }
                }
                None
            },
            
            BehaviorNode::Decorator(decorator_type, child) => {
                match decorator_type {
                    DecoratorType::Probability(prob) => {
                        if rand::random::<f32>() < *prob {
                            self.evaluate_behavior_tree(child, npc, world_time)
                        } else {
                            None
                        }
                    },
                    DecoratorType::Invert => {
                        match self.evaluate_behavior_tree(child, npc, world_time) {
                            Some(_) => None,
                            None => Some("invert_success".to_string()),
                        }
                    },
                    _ => self.evaluate_behavior_tree(child, npc, world_time),
                }
            },
            
            BehaviorNode::Action(behavior_name) => {
                Some(behavior_name.clone())
            },
            
            BehaviorNode::Condition(condition) => {
                if self.evaluate_condition(condition, npc, world_time) {
                    Some("condition_met".to_string())
                } else {
                    None
                }
            },
        }
    }

    fn evaluate_condition(&self, condition: &ConditionType, npc: &NPC, world_time: u32) -> bool {
        match condition {
            ConditionType::EnergyLevel(min, max) => {
                npc.energy >= *min && npc.energy <= *max
            },
            ConditionType::MoodLevel(min, max) => {
                npc.mood >= *min && npc.mood <= *max
            },
            ConditionType::TimeOfDay(start, end) => {
                let hour = world_time / 60;
                if start <= end {
                    hour >= *start && hour <= *end
                } else {
                    // Handles overnight periods like 22-6
                    hour >= *start || hour <= *end
                }
            },
            ConditionType::NPCNearby(radius) => {
                !npc.visible_npcs.is_empty() // Simplified - would check actual distances
            },
            ConditionType::HasSkill(skill, level) => {
                npc.skills.get(skill).unwrap_or(&0.0) >= level
            },
            ConditionType::LocationDistance(target, max_distance) => {
                let distance = npc.distance_to(*target);
                distance <= *max_distance
            },
            ConditionType::RelationshipLevel(target_npc, min_affection) => {
                npc.relationships.get(target_npc)
                    .map(|rel| rel.affection >= *min_affection)
                    .unwrap_or(false)
            },
        }
    }

    fn create_behavior_from_template(&self, template: &BehaviorTemplate, npc: &NPC, world_time: u32) -> Behavior {
        // Calculate priority based on personality and current state
        let mut priority = template.base_priority;
        
        // Adjust priority based on personality traits
        for (trait_name, influence) in &template.personality_influences {
            if let Some(trait_value) = npc.personality.traits.get(trait_name) {
                priority += (trait_value - 0.5) * influence * 20.0; // Scale influence
            }
        }
        
        // Adjust priority based on current needs
        match template.name.as_str() {
            "sleep" => {
                priority += (30.0 - npc.energy) * 2.0; // More urgent when energy is low
            },
            "eat" => {
                priority += npc.hunger * 0.8;
            },
            "socialize" => {
                priority += (100.0 - npc.mood) * 0.3; // More urgent when mood is low
                priority -= npc.stress * 0.2; // Less likely when stressed
            },
            _ => {},
        }
        
        // Apply environmental modifiers
        if let Some(modifier) = self.global_behavior_modifiers.get(&template.name) {
            priority *= modifier;
        }
        
        // Random duration within range
        let duration = template.duration_range.0 + 
            (template.duration_range.1 - template.duration_range.0) * rand::random::<f32>();

        Behavior {
            id: format!("{}_{}", template.name, world_time),
            behavior_type: self.string_to_behavior_type(&template.name),
            priority,
            target_location: None, // Would be determined by specific behavior logic
            target_npc: None,
            target_object: None,
            duration,
            progress: 0.0,
            conditions: template.conditions.clone(),
            effects: template.effects.clone(),
        }
    }

    fn string_to_behavior_type(&self, name: &str) -> BehaviorType {
        match name {
            "sleep" => BehaviorType::Sleep,
            "eat" => BehaviorType::Eat,
            "build" => BehaviorType::Build,
            "socialize" => BehaviorType::Talk,
            "explore" => BehaviorType::MoveTo, // Simplified
            "work" => BehaviorType::Work,
            _ => BehaviorType::Wait,
        }
    }

    fn update_global_modifiers(&mut self, _delta_time: f32) {
        // Seasonal adjustments
        match self.environmental_factors.season {
            Season::Winter => {
                self.global_behavior_modifiers.insert("outdoor_activity".to_string(), 0.6);
                self.global_behavior_modifiers.insert("social_activity".to_string(), 1.2);
            },
            Season::Summer => {
                self.global_behavior_modifiers.insert("outdoor_activity".to_string(), 1.4);
                self.global_behavior_modifiers.insert("work".to_string(), 0.9);
            },
            _ => {},
        }
        
        // Population density effects
        if self.environmental_factors.population_density > 0.8 {
            self.global_behavior_modifiers.insert("exploration".to_string(), 0.7);
            self.global_behavior_modifiers.insert("social_activity".to_string(), 1.3);
        }
    }

    pub fn add_behavior_template(&mut self, template: BehaviorTemplate) {
        self.behavior_templates.insert(template.name.clone(), template);
    }

    pub fn get_behavior_suggestions(&self, npc: &NPC, world_time: u32) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        for (name, template) in &self.behavior_templates {
            // Check if behavior is applicable
            let mut applicable = true;
            
            for condition in &template.conditions {
                if !self.evaluate_behavior_condition(condition, npc, world_time) {
                    applicable = false;
                    break;
                }
            }
            
            if applicable {
                suggestions.push(name.clone());
            }
        }
        
        suggestions.sort_by(|a, b| {
            let priority_a = self.behavior_templates.get(a).unwrap().base_priority;
            let priority_b = self.behavior_templates.get(b).unwrap().base_priority;
            priority_b.partial_cmp(&priority_a).unwrap()
        });
        
        suggestions
    }

    fn evaluate_behavior_condition(&self, condition: &BehaviorCondition, npc: &NPC, world_time: u32) -> bool {
        match condition {
            BehaviorCondition::EnergyAbove(threshold) => npc.energy > *threshold,
            BehaviorCondition::EnergyBelow(threshold) => npc.energy < *threshold,
            BehaviorCondition::MoodAbove(threshold) => npc.mood > *threshold,
            BehaviorCondition::MoodBelow(threshold) => npc.mood < *threshold,
            BehaviorCondition::TimeOfDay(start, end) => {
                let hour = world_time / 60;
                if start <= end {
                    hour >= *start && hour <= *end
                } else {
                    hour >= *start || hour <= *end
                }
            },
            BehaviorCondition::NPCNearby(npc_id, distance) => {
                if npc_id == "any" {
                    !npc.visible_npcs.is_empty()
                } else {
                    npc.visible_npcs.contains(npc_id)
                }
            },
            _ => true, // Simplified for other conditions
        }
    }
}

impl Default for EnvironmentalFactors {
    fn default() -> Self {
        Self {
            weather: Weather::Sunny,
            time_of_day: 720, // 12:00 PM
            season: Season::Spring,
            population_density: 0.5,
            noise_level: 0.3,
            safety_level: 0.8,
            resource_availability: HashMap::new(),
        }
    }
}

// Simple random number generator for demo purposes
mod rand {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn random<T>() -> T 
    where 
        T: From<f32>
    {
        let mut hasher = DefaultHasher::new();
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().hash(&mut hasher);
        let hash_value = hasher.finish();
        let normalized = (hash_value as f64 / u64::MAX as f64) as f32;
        T::from(normalized)
    }
}