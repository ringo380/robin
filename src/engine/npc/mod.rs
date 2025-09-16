use crate::engine::math::{Vec3, Point3};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub mod behavior_system;
pub mod social_system;
pub mod routine_system;
pub mod ai_decision;
pub mod personality_system;

pub use behavior_system::BehaviorSystem;
pub use social_system::SocialSystem;
pub use routine_system::RoutineSystem;
pub use ai_decision::AIDecisionEngine;
pub use personality_system::PersonalitySystem;

#[derive(Debug, Clone)]
pub struct NPC {
    pub id: String,
    pub name: String,
    pub position: Point3,
    pub rotation: Vec3,
    pub state: NPCState,
    
    // Physical properties
    pub health: f32,
    pub energy: f32,
    pub hunger: f32,
    pub mood: f32,
    pub stress: f32,
    
    // Core attributes
    pub attributes: NPCAttributes,
    pub personality: Personality,
    pub relationships: HashMap<String, Relationship>,
    pub skills: HashMap<String, f32>,
    
    // Current behavior
    pub current_behavior: Option<Behavior>,
    pub behavior_queue: Vec<Behavior>,
    pub daily_routine: DailyRoutine,
    
    // Memory and awareness
    pub memory: NPCMemory,
    pub awareness_radius: f32,
    pub visible_npcs: Vec<String>,
    pub visible_objects: Vec<String>,
    
    // Job and role
    pub occupation: Occupation,
    pub workplace: Option<Point3>,
    pub home: Option<Point3>,
    pub favorite_locations: Vec<Point3>,
    
    // AI state
    pub decision_cooldown: f32,
    pub last_decision_time: u64,
    pub priority_goals: Vec<Goal>,
}

#[derive(Debug, Clone)]
pub enum NPCState {
    Idle,
    Walking,
    Working,
    Socializing,
    Eating,
    Sleeping,
    Shopping,
    Entertaining,
    Panicking,
    Investigating,
    Following,
    Fleeing,
}

#[derive(Debug, Clone)]
pub struct NPCAttributes {
    pub intelligence: f32,
    pub creativity: f32,
    pub strength: f32,
    pub agility: f32,
    pub charisma: f32,
    pub patience: f32,
    pub curiosity: f32,
    pub bravery: f32,
}

#[derive(Debug, Clone)]
pub struct Personality {
    pub traits: HashMap<String, f32>,
    pub dominant_traits: Vec<String>,
    pub behavioral_modifiers: HashMap<String, f32>,
}

#[derive(Debug, Clone)]
pub struct Relationship {
    pub npc_id: String,
    pub relationship_type: RelationshipType,
    pub affection: f32,
    pub trust: f32,
    pub respect: f32,
    pub familiarity: f32,
    pub last_interaction: u64,
    pub shared_experiences: Vec<SharedExperience>,
}

#[derive(Debug, Clone)]
pub enum RelationshipType {
    Family,
    Friend,
    Romantic,
    Colleague,
    Neighbor,
    Rival,
    Enemy,
    Stranger,
    Acquaintance,
}

#[derive(Debug, Clone)]
pub struct SharedExperience {
    pub experience_type: ExperienceType,
    pub timestamp: u64,
    pub emotional_impact: f32,
    pub location: Point3,
    pub other_participants: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum ExperienceType {
    Conversation,
    WorkTogether,
    Conflict,
    Celebration,
    Crisis,
    Discovery,
    Competition,
    Cooperation,
}

#[derive(Debug, Clone)]
pub struct Behavior {
    pub id: String,
    pub behavior_type: BehaviorType,
    pub priority: f32,
    pub target_location: Option<Point3>,
    pub target_npc: Option<String>,
    pub target_object: Option<String>,
    pub duration: f32,
    pub progress: f32,
    pub conditions: Vec<BehaviorCondition>,
    pub effects: Vec<BehaviorEffect>,
}

#[derive(Debug, Clone)]
pub enum BehaviorType {
    // Basic behaviors
    MoveTo,
    LookAt,
    Wait,
    Rest,
    
    // Work behaviors
    Work,
    Build,
    Craft,
    Gather,
    
    // Social behaviors
    Talk,
    Greet,
    Argue,
    Comfort,
    Celebrate,
    
    // Maintenance behaviors
    Eat,
    Sleep,
    Exercise,
    Clean,
    
    // Entertainment behaviors
    Play,
    Dance,
    Sing,
    Watch,
    
    // Emergency behaviors
    Flee,
    Hide,
    Investigate,
    Alert,
}

#[derive(Debug, Clone)]
pub enum BehaviorCondition {
    EnergyAbove(f32),
    EnergyBelow(f32),
    MoodAbove(f32),
    MoodBelow(f32),
    TimeOfDay(u32, u32), // start hour, end hour
    NPCNearby(String, f32), // npc_id, distance
    LocationReached(Point3, f32), // location, tolerance
    ObjectAvailable(String),
    SkillLevel(String, f32),
}

#[derive(Debug, Clone)]
pub enum BehaviorEffect {
    ChangeEnergy(f32),
    ChangeMood(f32),
    ChangeHunger(f32),
    ChangeStress(f32),
    LearnSkill(String, f32),
    ModifyRelationship(String, f32, f32), // npc_id, affection_change, trust_change
    CreateMemory(String, f32), // memory_content, importance
    TriggerEvent(String),
}

#[derive(Debug, Clone)]
pub struct DailyRoutine {
    pub schedule: Vec<ScheduleEntry>,
    pub flexibility: f32,
    pub routine_adherence: f32,
    pub weekend_variation: bool,
}

#[derive(Debug, Clone)]
pub struct ScheduleEntry {
    pub start_time: u32, // Hour of day (0-23)
    pub duration: u32,   // Duration in minutes
    pub activity: String,
    pub location: Option<Point3>,
    pub required_npcs: Vec<String>,
    pub priority: f32,
    pub conditions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct NPCMemory {
    pub short_term: Vec<MemoryEntry>,
    pub long_term: Vec<MemoryEntry>,
    pub episodic: Vec<EpisodicMemory>,
    pub procedural: HashMap<String, f32>,
    pub capacity: usize,
}

#[derive(Debug, Clone)]
pub struct MemoryEntry {
    pub content: String,
    pub timestamp: u64,
    pub importance: f32,
    pub emotional_charge: f32,
    pub associated_npcs: Vec<String>,
    pub location: Point3,
    pub decay_rate: f32,
}

#[derive(Debug, Clone)]
pub struct EpisodicMemory {
    pub event_type: String,
    pub participants: Vec<String>,
    pub location: Point3,
    pub timestamp: u64,
    pub emotional_impact: f32,
    pub consequences: Vec<String>,
    pub vividness: f32,
}

#[derive(Debug, Clone)]
pub enum Occupation {
    Builder,
    Craftsperson,
    Merchant,
    Guard,
    Farmer,
    Cook,
    Entertainer,
    Teacher,
    Healer,
    Explorer,
    Artist,
    Leader,
    Unemployed,
}

#[derive(Debug, Clone)]
pub struct Goal {
    pub id: String,
    pub goal_type: GoalType,
    pub description: String,
    pub priority: f32,
    pub progress: f32,
    pub target_value: f32,
    pub deadline: Option<u64>,
    pub sub_goals: Vec<String>,
    pub conditions: Vec<GoalCondition>,
}

#[derive(Debug, Clone)]
pub enum GoalType {
    Survival,      // Basic needs
    Achievement,   // Skill or status goals
    Social,        // Relationship goals
    Creative,      // Building or crafting goals
    Exploration,   // Discovery goals
    Maintenance,   // Upkeep goals
}

#[derive(Debug, Clone)]
pub enum GoalCondition {
    ResourceAvailable(String, u32),
    SkillReached(String, f32),
    LocationAccessible(Point3),
    NPCAvailable(String),
    TimeAvailable(u32), // minutes
}

impl Default for NPCAttributes {
    fn default() -> Self {
        Self {
            intelligence: 0.5,
            creativity: 0.5,
            strength: 0.5,
            agility: 0.5,
            charisma: 0.5,
            patience: 0.5,
            curiosity: 0.5,
            bravery: 0.5,
        }
    }
}

impl Default for Personality {
    fn default() -> Self {
        let mut traits = HashMap::new();
        traits.insert("extroversion".to_string(), 0.5);
        traits.insert("agreeableness".to_string(), 0.5);
        traits.insert("conscientiousness".to_string(), 0.5);
        traits.insert("neuroticism".to_string(), 0.5);
        traits.insert("openness".to_string(), 0.5);
        
        Self {
            traits,
            dominant_traits: vec!["balanced".to_string()],
            behavioral_modifiers: HashMap::new(),
        }
    }
}

impl Default for DailyRoutine {
    fn default() -> Self {
        Self {
            schedule: Vec::new(),
            flexibility: 0.3,
            routine_adherence: 0.8,
            weekend_variation: true,
        }
    }
}

impl Default for NPCMemory {
    fn default() -> Self {
        Self {
            short_term: Vec::new(),
            long_term: Vec::new(),
            episodic: Vec::new(),
            procedural: HashMap::new(),
            capacity: 100,
        }
    }
}

impl NPC {
    pub fn new(id: String, name: String, position: Point3) -> Self {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id,
            name,
            position,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            state: NPCState::Idle,
            
            health: 100.0,
            energy: 80.0,
            hunger: 30.0,
            mood: 70.0,
            stress: 20.0,
            
            attributes: NPCAttributes::default(),
            personality: Personality::default(),
            relationships: HashMap::new(),
            skills: HashMap::new(),
            
            current_behavior: None,
            behavior_queue: Vec::new(),
            daily_routine: DailyRoutine::default(),
            
            memory: NPCMemory::default(),
            awareness_radius: 10.0,
            visible_npcs: Vec::new(),
            visible_objects: Vec::new(),
            
            occupation: Occupation::Unemployed,
            workplace: None,
            home: None,
            favorite_locations: Vec::new(),
            
            decision_cooldown: 0.0,
            last_decision_time: current_time,
            priority_goals: Vec::new(),
        }
    }

    pub fn update(&mut self, delta_time: f32, current_time: u64, world_time: u32) {
        // Update basic needs
        self.update_needs(delta_time);
        
        // Update current behavior
        self.update_behavior(delta_time);
        
        // Update decision making
        self.update_decisions(delta_time, current_time, world_time);
        
        // Update memory decay
        self.update_memory(delta_time);
        
        // Update relationships
        self.update_relationships(delta_time);
    }

    fn update_needs(&mut self, delta_time: f32) {
        // Energy decreases over time, faster when active
        let energy_drain = match self.state {
            NPCState::Working | NPCState::Walking => 3.0,
            NPCState::Socializing | NPCState::Entertaining => 2.0,
            NPCState::Sleeping => -5.0, // Restore energy when sleeping
            _ => 1.0,
        };
        
        self.energy = (self.energy - energy_drain * delta_time).clamp(0.0, 100.0);
        
        // Hunger increases over time
        self.hunger = (self.hunger + 0.5 * delta_time).clamp(0.0, 100.0);
        
        // Mood affected by needs and activities
        let mood_change = match self.state {
            NPCState::Socializing | NPCState::Entertaining => 0.2,
            NPCState::Working => if self.energy > 50.0 { 0.1 } else { -0.1 },
            NPCState::Panicking | NPCState::Fleeing => -0.5,
            _ => -0.05,
        };
        
        // Adjust mood based on needs
        let needs_mood_modifier = if self.energy < 20.0 || self.hunger > 80.0 {
            -0.3
        } else if self.energy > 80.0 && self.hunger < 20.0 {
            0.2
        } else {
            0.0
        };
        
        self.mood = (self.mood + (mood_change + needs_mood_modifier) * delta_time).clamp(0.0, 100.0);
        
        // Stress affected by mood and circumstances
        let stress_change = if self.mood < 30.0 {
            0.2
        } else if self.mood > 70.0 {
            -0.1
        } else {
            0.0
        };
        
        self.stress = (self.stress + stress_change * delta_time).clamp(0.0, 100.0);
    }

    fn update_behavior(&mut self, delta_time: f32) {
        if let Some(ref mut behavior) = self.current_behavior {
            behavior.progress += delta_time / behavior.duration;
            
            // Apply behavior effects gradually
            for effect in &behavior.effects {
                match effect {
                    BehaviorEffect::ChangeEnergy(amount) => {
                        self.energy = (self.energy + amount * delta_time).clamp(0.0, 100.0);
                    },
                    BehaviorEffect::ChangeMood(amount) => {
                        self.mood = (self.mood + amount * delta_time).clamp(0.0, 100.0);
                    },
                    BehaviorEffect::ChangeHunger(amount) => {
                        self.hunger = (self.hunger + amount * delta_time).clamp(0.0, 100.0);
                    },
                    BehaviorEffect::ChangeStress(amount) => {
                        self.stress = (self.stress + amount * delta_time).clamp(0.0, 100.0);
                    },
                    _ => {}, // Handle other effects elsewhere
                }
            }
            
            // Check if behavior is complete
            if behavior.progress >= 1.0 {
                self.complete_current_behavior();
            }
        }
        
        // Start next behavior if none active
        if self.current_behavior.is_none() && !self.behavior_queue.is_empty() {
            self.start_next_behavior();
        }
    }

    fn update_decisions(&mut self, delta_time: f32, current_time: u64, world_time: u32) {
        self.decision_cooldown -= delta_time;
        
        if self.decision_cooldown <= 0.0 {
            // Make decisions based on current state and needs
            self.make_decisions(current_time, world_time);
            self.decision_cooldown = 2.0; // Decision every 2 seconds
        }
    }

    fn update_memory(&mut self, delta_time: f32) {
        // Decay short-term memories
        for memory in &mut self.memory.short_term {
            memory.importance *= 1.0 - memory.decay_rate * delta_time;
        }
        
        // Move important short-term memories to long-term
        let mut to_promote = Vec::new();
        for (i, memory) in self.memory.short_term.iter().enumerate() {
            if memory.importance > 0.8 || memory.emotional_charge.abs() > 0.7 {
                to_promote.push(i);
            }
        }
        
        // Promote memories (in reverse order to maintain indices)
        for &index in to_promote.iter().rev() {
            if index < self.memory.short_term.len() {
                let memory = self.memory.short_term.remove(index);
                self.memory.long_term.push(memory);
            }
        }
        
        // Remove very faded memories
        self.memory.short_term.retain(|m| m.importance > 0.1);
        
        // Limit memory capacity
        if self.memory.short_term.len() > self.memory.capacity {
            self.memory.short_term.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap());
            self.memory.short_term.truncate(self.memory.capacity);
        }
    }

    fn update_relationships(&mut self, _delta_time: f32) {
        // Relationship decay over time if no interaction
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        for relationship in self.relationships.values_mut() {
            let time_since_interaction = current_time - relationship.last_interaction;
            let days_since = time_since_interaction / (24 * 60 * 60);
            
            if days_since > 7 {
                // Relationships decay if no interaction for a week
                let decay_factor = 0.99_f32.powi(days_since as i32);
                relationship.familiarity *= decay_factor;
                
                // Affection decays slower for close relationships
                if relationship.affection > 0.5 {
                    relationship.affection *= decay_factor.sqrt();
                }
            }
        }
    }

    fn make_decisions(&mut self, current_time: u64, world_time: u32) {
        // Priority-based decision making
        let mut potential_behaviors = Vec::new();
        
        // Check immediate needs
        if self.energy < 30.0 {
            potential_behaviors.push(self.create_rest_behavior());
        }
        
        if self.hunger > 70.0 {
            potential_behaviors.push(self.create_eat_behavior());
        }
        
        if self.stress > 80.0 {
            potential_behaviors.push(self.create_stress_relief_behavior());
        }
        
        // Check routine schedule
        if let Some(scheduled_behavior) = self.get_scheduled_behavior(world_time) {
            potential_behaviors.push(scheduled_behavior);
        }
        
        // Social behaviors
        if !self.visible_npcs.is_empty() && self.mood > 40.0 {
            potential_behaviors.push(self.create_social_behavior());
        }
        
        // Work behaviors (if at workplace)
        if let Some(workplace) = self.workplace {
            let distance_to_work = self.distance_to(workplace);
            if distance_to_work < 5.0 && self.is_work_time(world_time) {
                potential_behaviors.push(self.create_work_behavior());
            }
        }
        
        // Select highest priority behavior
        if let Some(best_behavior) = potential_behaviors.into_iter()
            .max_by(|a, b| a.priority.partial_cmp(&b.priority).unwrap()) {
            
            if self.current_behavior.is_none() || 
               best_behavior.priority > self.current_behavior.as_ref().unwrap().priority {
                self.queue_behavior(best_behavior);
            }
        }
        
        self.last_decision_time = current_time;
    }

    fn complete_current_behavior(&mut self) {
        if let Some(behavior) = self.current_behavior.take() {
            // Apply completion effects
            for effect in &behavior.effects {
                match effect {
                    BehaviorEffect::LearnSkill(skill, amount) => {
                        let current_skill = self.skills.get(skill).unwrap_or(&0.0);
                        self.skills.insert(skill.clone(), current_skill + amount);
                    },
                    BehaviorEffect::CreateMemory(content, importance) => {
                        self.add_memory(content.clone(), *importance);
                    },
                    _ => {}, // Other effects already applied during update
                }
            }
            
            // Update state based on completed behavior
            match behavior.behavior_type {
                BehaviorType::Sleep => {
                    self.energy = 100.0;
                    self.state = NPCState::Idle;
                },
                BehaviorType::Eat => {
                    self.hunger = 0.0;
                    self.state = NPCState::Idle;
                },
                BehaviorType::Work => {
                    self.state = NPCState::Idle;
                },
                _ => {
                    self.state = NPCState::Idle;
                },
            }
        }
    }

    fn start_next_behavior(&mut self) {
        if let Some(behavior) = self.behavior_queue.first() {
            self.current_behavior = Some(behavior.clone());
            self.behavior_queue.remove(0);
            
            // Update state based on new behavior
            match self.current_behavior.as_ref().unwrap().behavior_type {
                BehaviorType::Sleep => self.state = NPCState::Sleeping,
                BehaviorType::Eat => self.state = NPCState::Eating,
                BehaviorType::Work => self.state = NPCState::Working,
                BehaviorType::Talk | BehaviorType::Greet => self.state = NPCState::Socializing,
                BehaviorType::MoveTo => self.state = NPCState::Walking,
                _ => self.state = NPCState::Idle,
            }
        }
    }

    // Helper methods for behavior creation
    fn create_rest_behavior(&self) -> Behavior {
        Behavior {
            id: "rest".to_string(),
            behavior_type: BehaviorType::Sleep,
            priority: 90.0,
            target_location: self.home,
            target_npc: None,
            target_object: None,
            duration: 300.0, // 5 minutes
            progress: 0.0,
            conditions: vec![],
            effects: vec![
                BehaviorEffect::ChangeEnergy(50.0),
                BehaviorEffect::ChangeStress(-20.0),
            ],
        }
    }

    fn create_eat_behavior(&self) -> Behavior {
        Behavior {
            id: "eat".to_string(),
            behavior_type: BehaviorType::Eat,
            priority: 85.0,
            target_location: self.home,
            target_npc: None,
            target_object: Some("food".to_string()),
            duration: 60.0, // 1 minute
            progress: 0.0,
            conditions: vec![],
            effects: vec![
                BehaviorEffect::ChangeHunger(-70.0),
                BehaviorEffect::ChangeMood(10.0),
            ],
        }
    }

    fn create_stress_relief_behavior(&self) -> Behavior {
        Behavior {
            id: "relax".to_string(),
            behavior_type: BehaviorType::Rest,
            priority: 75.0,
            target_location: None,
            target_npc: None,
            target_object: None,
            duration: 120.0, // 2 minutes
            progress: 0.0,
            conditions: vec![],
            effects: vec![
                BehaviorEffect::ChangeStress(-30.0),
                BehaviorEffect::ChangeMood(15.0),
            ],
        }
    }

    fn create_social_behavior(&self) -> Behavior {
        let target_npc = self.visible_npcs.first().cloned();
        
        Behavior {
            id: "socialize".to_string(),
            behavior_type: BehaviorType::Talk,
            priority: 50.0,
            target_location: None,
            target_npc,
            target_object: None,
            duration: 90.0, // 1.5 minutes
            progress: 0.0,
            conditions: vec![],
            effects: vec![
                BehaviorEffect::ChangeMood(20.0),
                BehaviorEffect::ChangeStress(-10.0),
            ],
        }
    }

    fn create_work_behavior(&self) -> Behavior {
        Behavior {
            id: "work".to_string(),
            behavior_type: BehaviorType::Work,
            priority: 60.0,
            target_location: self.workplace,
            target_npc: None,
            target_object: None,
            duration: 240.0, // 4 minutes
            progress: 0.0,
            conditions: vec![],
            effects: vec![
                BehaviorEffect::ChangeEnergy(-15.0),
                BehaviorEffect::LearnSkill("work_skill".to_string(), 0.1),
                BehaviorEffect::ChangeMood(5.0),
            ],
        }
    }

    fn get_scheduled_behavior(&self, world_time: u32) -> Option<Behavior> {
        let hour = world_time / 60; // Convert minutes to hours
        
        for entry in &self.daily_routine.schedule {
            if hour >= entry.start_time && hour < (entry.start_time + entry.duration / 60) {
                // Create behavior based on activity
                match entry.activity.as_str() {
                    "sleep" => return Some(self.create_rest_behavior()),
                    "eat" => return Some(self.create_eat_behavior()),
                    "work" => return Some(self.create_work_behavior()),
                    _ => {},
                }
            }
        }
        
        None
    }

    fn is_work_time(&self, world_time: u32) -> bool {
        let hour = world_time / 60;
        hour >= 8 && hour <= 17 // 8 AM to 5 PM
    }

    fn distance_to(&self, point: Point3) -> f32 {
        let dx = self.position.x - point.x;
        let dy = self.position.y - point.y;
        let dz = self.position.z - point.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    pub fn queue_behavior(&mut self, behavior: Behavior) {
        // Insert behavior based on priority
        let insert_pos = self.behavior_queue
            .iter()
            .position(|b| b.priority < behavior.priority)
            .unwrap_or(self.behavior_queue.len());
        
        self.behavior_queue.insert(insert_pos, behavior);
    }

    pub fn add_memory(&mut self, content: String, importance: f32) {
        let memory = MemoryEntry {
            content,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            importance,
            emotional_charge: 0.0,
            associated_npcs: Vec::new(),
            location: self.position,
            decay_rate: 0.01,
        };
        
        self.memory.short_term.push(memory);
    }

    pub fn interact_with(&mut self, other_npc_id: String, interaction_type: ExperienceType) {
        // Update relationship
        let relationship = self.relationships.entry(other_npc_id.clone())
            .or_insert(Relationship {
                npc_id: other_npc_id.clone(),
                relationship_type: RelationshipType::Acquaintance,
                affection: 50.0,
                trust: 50.0,
                respect: 50.0,
                familiarity: 0.0,
                last_interaction: 0,
                shared_experiences: Vec::new(),
            });
        
        relationship.last_interaction = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        relationship.familiarity += 1.0;
        
        // Adjust relationship based on interaction type
        match interaction_type {
            ExperienceType::Conversation => {
                relationship.familiarity += 2.0;
                relationship.affection += 1.0;
            },
            ExperienceType::Cooperation => {
                relationship.trust += 3.0;
                relationship.respect += 2.0;
                relationship.affection += 1.0;
            },
            ExperienceType::Conflict => {
                relationship.trust -= 2.0;
                relationship.affection -= 3.0;
            },
            ExperienceType::Celebration => {
                relationship.affection += 4.0;
                relationship.familiarity += 2.0;
            },
            _ => {},
        }
        
        // Clamp relationship values
        relationship.affection = relationship.affection.clamp(0.0, 100.0);
        relationship.trust = relationship.trust.clamp(0.0, 100.0);
        relationship.respect = relationship.respect.clamp(0.0, 100.0);
        
        // Add shared experience
        let experience = SharedExperience {
            experience_type: interaction_type,
            timestamp: relationship.last_interaction,
            emotional_impact: 1.0,
            location: self.position,
            other_participants: vec![other_npc_id],
        };
        
        relationship.shared_experiences.push(experience);
    }

    // Getters
    pub fn get_state(&self) -> &NPCState {
        &self.state
    }

    pub fn get_current_behavior(&self) -> Option<&Behavior> {
        self.current_behavior.as_ref()
    }

    pub fn get_mood_state(&self) -> String {
        match self.mood {
            m if m >= 80.0 => "Joyful".to_string(),
            m if m >= 60.0 => "Happy".to_string(),
            m if m >= 40.0 => "Content".to_string(),
            m if m >= 20.0 => "Sad".to_string(),
            _ => "Depressed".to_string(),
        }
    }

    pub fn get_energy_state(&self) -> String {
        match self.energy {
            e if e >= 80.0 => "Energetic".to_string(),
            e if e >= 60.0 => "Active".to_string(),
            e if e >= 40.0 => "Normal".to_string(),
            e if e >= 20.0 => "Tired".to_string(),
            _ => "Exhausted".to_string(),
        }
    }

    pub fn get_relationship_summary(&self) -> HashMap<String, String> {
        let mut summary = HashMap::new();
        
        for (npc_id, relationship) in &self.relationships {
            let status = match relationship.affection {
                a if a >= 80.0 => "Best Friend",
                a if a >= 60.0 => "Good Friend",
                a if a >= 40.0 => "Friend",
                a if a >= 20.0 => "Acquaintance",
                _ => "Dislike",
            };
            
            summary.insert(npc_id.clone(), status.to_string());
        }
        
        summary
    }
}