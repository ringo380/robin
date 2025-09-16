use crate::engine::math::{Vec3, Point3};
use crate::engine::npc::{NPC, Relationship, RelationshipType, SharedExperience, ExperienceType};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SocialSystem {
    social_networks: HashMap<String, SocialNetwork>,
    conversation_engine: ConversationEngine,
    reputation_system: ReputationSystem,
    social_events: Vec<SocialEvent>,
    group_dynamics: HashMap<String, GroupDynamic>,
}

#[derive(Debug, Clone)]
pub struct SocialNetwork {
    pub network_id: String,
    pub members: Vec<String>,
    pub network_type: NetworkType,
    pub influence_map: HashMap<String, f32>, // NPC ID -> influence score
    pub shared_interests: Vec<String>,
    pub meeting_locations: Vec<Point3>,
    pub activity_schedule: Vec<GroupActivity>,
}

#[derive(Debug, Clone)]
pub enum NetworkType {
    Family,
    WorkColleagues,
    Friends,
    Neighbors,
    HobbyGroup,
    Professional,
    Community,
}

#[derive(Debug, Clone)]
pub struct GroupActivity {
    pub activity_name: String,
    pub scheduled_time: u32, // World time
    pub duration: u32,       // Minutes
    pub location: Point3,
    pub required_participants: Vec<String>,
    pub optional_participants: Vec<String>,
    pub activity_type: ActivityType,
}

#[derive(Debug, Clone)]
pub enum ActivityType {
    Work,
    Celebration,
    Meeting,
    Recreation,
    Competition,
    Cooperation,
    Learning,
    Crisis,
}

#[derive(Debug, Clone)]
pub struct ConversationEngine {
    pub conversation_topics: HashMap<String, ConversationTopic>,
    pub active_conversations: HashMap<String, ActiveConversation>,
    pub dialogue_templates: HashMap<String, DialogueTemplate>,
    pub personality_responses: HashMap<String, Vec<ResponsePattern>>,
}

#[derive(Debug, Clone)]
pub struct ConversationTopic {
    pub topic_id: String,
    pub name: String,
    pub interest_level: HashMap<String, f32>, // Personality trait -> interest modifier
    pub emotional_impact: f32,
    pub complexity: f32,
    pub prerequisites: Vec<String>, // Required knowledge or relationships
    pub outcomes: Vec<ConversationOutcome>,
}

#[derive(Debug, Clone)]
pub struct ActiveConversation {
    pub participants: Vec<String>,
    pub current_topic: String,
    pub mood: ConversationMood,
    pub duration: f32,
    pub topic_history: Vec<String>,
    pub relationship_changes: HashMap<String, RelationshipChange>,
}

#[derive(Debug, Clone)]
pub enum ConversationMood {
    Friendly,
    Tense,
    Excited,
    Serious,
    Casual,
    Intimate,
    Formal,
    Heated,
}

#[derive(Debug, Clone)]
pub struct RelationshipChange {
    pub affection_delta: f32,
    pub trust_delta: f32,
    pub respect_delta: f32,
    pub familiarity_delta: f32,
}

#[derive(Debug, Clone)]
pub struct DialogueTemplate {
    pub template_id: String,
    pub situation: String,
    pub personality_variants: HashMap<String, Vec<String>>, // Personality type -> possible responses
    pub emotional_variants: HashMap<String, Vec<String>>,   // Emotional state -> responses
    pub relationship_variants: HashMap<String, Vec<String>>, // Relationship level -> responses
}

#[derive(Debug, Clone)]
pub struct ResponsePattern {
    pub trigger: ResponseTrigger,
    pub response_templates: Vec<String>,
    pub probability_modifiers: HashMap<String, f32>,
}

#[derive(Debug, Clone)]
pub enum ResponseTrigger {
    TopicMentioned(String),
    EmotionalState(String),
    RelationshipLevel(f32),
    PersonalityTrait(String, f32),
    RecentMemory(String),
}

#[derive(Debug, Clone)]
pub enum ConversationOutcome {
    RelationshipImprove(f32),
    RelationshipWorsen(f32),
    MemoryCreated(String, f32), // Memory content, importance
    SkillLearned(String, f32),
    InformationShared(String),
    ConflictEscalated,
    ConflictResolved,
    PlanMade(String), // Plan description
}

#[derive(Debug, Clone)]
pub struct ReputationSystem {
    pub reputation_scores: HashMap<String, NPCReputation>,
    pub reputation_events: Vec<ReputationEvent>,
    pub community_opinions: HashMap<String, f32>, // Topic -> community sentiment
}

#[derive(Debug, Clone)]
pub struct NPCReputation {
    pub npc_id: String,
    pub overall_reputation: f32,
    pub trait_reputations: HashMap<String, f32>, // "honest", "reliable", "skilled", etc.
    pub occupation_reputation: f32,
    pub social_standing: f32,
    pub recent_actions: Vec<ReputationAction>,
}

#[derive(Debug, Clone)]
pub struct ReputationAction {
    pub action: String,
    pub witnesses: Vec<String>,
    pub impact: f32,
    pub timestamp: u64,
    pub context: String,
}

#[derive(Debug, Clone)]
pub struct ReputationEvent {
    pub event_type: String,
    pub involved_npcs: Vec<String>,
    pub impact_magnitude: f32,
    pub spread_rate: f32,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct SocialEvent {
    pub event_id: String,
    pub event_type: SocialEventType,
    pub organizer: Option<String>,
    pub participants: Vec<String>,
    pub location: Point3,
    pub start_time: u32,
    pub duration: u32,
    pub requirements: Vec<String>,
    pub outcomes: Vec<SocialEventOutcome>,
}

#[derive(Debug, Clone)]
pub enum SocialEventType {
    Party,
    Meeting,
    Festival,
    Competition,
    Ceremony,
    Crisis,
    Market,
    Performance,
}

#[derive(Debug, Clone)]
pub enum SocialEventOutcome {
    RelationshipChanges(HashMap<String, RelationshipChange>),
    ReputationChanges(HashMap<String, f32>),
    MemoriesCreated(Vec<(String, String, f32)>), // NPC ID, memory, importance
    SkillsLearned(HashMap<String, HashMap<String, f32>>), // NPC ID -> skill -> amount
    NetworkFormed(String, Vec<String>), // Network type, members
    ConflictStarted(Vec<String>),
    AllianceFormed(Vec<String>),
}

#[derive(Debug, Clone)]
pub struct GroupDynamic {
    pub group_id: String,
    pub members: Vec<String>,
    pub leader: Option<String>,
    pub cohesion: f32,
    pub cooperation_level: f32,
    pub shared_goals: Vec<String>,
    pub internal_conflicts: Vec<Conflict>,
    pub group_mood: f32,
}

#[derive(Debug, Clone)]
pub struct Conflict {
    pub participants: Vec<String>,
    pub cause: String,
    pub intensity: f32,
    pub duration: u32,
    pub resolution_attempts: Vec<String>,
}

impl SocialSystem {
    pub fn new() -> Self {
        let mut system = Self {
            social_networks: HashMap::new(),
            conversation_engine: ConversationEngine::new(),
            reputation_system: ReputationSystem::new(),
            social_events: Vec::new(),
            group_dynamics: HashMap::new(),
        };

        system.initialize_default_topics();
        system.initialize_dialogue_templates();
        system
    }

    pub fn update(&mut self, npcs: &mut HashMap<String, NPC>, delta_time: f32, world_time: u32) {
        // Update active conversations
        self.update_conversations(npcs, delta_time, world_time);
        
        // Process social events
        self.process_social_events(npcs, world_time);
        
        // Update reputation propagation
        self.update_reputation_propagation(npcs, delta_time);
        
        // Update group dynamics
        self.update_group_dynamics(npcs, delta_time);
        
        // Generate spontaneous social interactions
        self.generate_social_interactions(npcs, world_time);
    }

    fn update_conversations(&mut self, npcs: &mut HashMap<String, NPC>, delta_time: f32, _world_time: u32) {
        let mut completed_conversations = Vec::new();
        
        for (conv_id, conversation) in &mut self.conversation_engine.active_conversations {
            conversation.duration += delta_time;
            
            // Update participants' mood and energy based on conversation
            for participant_id in &conversation.participants {
                if let Some(npc) = npcs.get_mut(participant_id) {
                    match conversation.mood {
                        ConversationMood::Friendly | ConversationMood::Excited => {
                            npc.mood += 0.1 * delta_time;
                            npc.stress -= 0.05 * delta_time;
                        },
                        ConversationMood::Tense | ConversationMood::Heated => {
                            npc.stress += 0.2 * delta_time;
                            npc.mood -= 0.05 * delta_time;
                        },
                        ConversationMood::Intimate => {
                            npc.mood += 0.15 * delta_time;
                            npc.stress -= 0.1 * delta_time;
                        },
                        _ => {},
                    }
                    
                    npc.energy -= 0.5 * delta_time; // Conversations use energy
                }
            }
            
            // Check if conversation should end
            if conversation.duration > 300.0 || // 5 minutes max
               conversation.participants.iter().any(|id| {
                   npcs.get(id).map(|npc| npc.energy < 20.0 || npc.stress > 80.0).unwrap_or(false)
               }) {
                completed_conversations.push(conv_id.clone());
            }
        }
        
        // Complete conversations and apply outcomes
        for conv_id in completed_conversations {
            if let Some(conversation) = self.conversation_engine.active_conversations.remove(&conv_id) {
                self.complete_conversation(conversation, npcs);
            }
        }
    }

    fn complete_conversation(&self, conversation: ActiveConversation, npcs: &mut HashMap<String, NPC>) {
        // Apply relationship changes
        for participant_id in &conversation.participants {
            if let Some(npc) = npcs.get_mut(participant_id) {
                for other_id in &conversation.participants {
                    if other_id != participant_id {
                        if let Some(change) = conversation.relationship_changes.get(other_id) {
                            // Apply relationship changes
                            let relationship = npc.relationships.entry(other_id.clone())
                                .or_insert(Relationship {
                                    npc_id: other_id.clone(),
                                    relationship_type: RelationshipType::Acquaintance,
                                    affection: 50.0,
                                    trust: 50.0,
                                    respect: 50.0,
                                    familiarity: 0.0,
                                    last_interaction: 0,
                                    shared_experiences: Vec::new(),
                                });

                            relationship.affection = (relationship.affection + change.affection_delta).clamp(0.0, 100.0);
                            relationship.trust = (relationship.trust + change.trust_delta).clamp(0.0, 100.0);
                            relationship.respect = (relationship.respect + change.respect_delta).clamp(0.0, 100.0);
                            relationship.familiarity += change.familiarity_delta;

                            // Add shared experience
                            relationship.shared_experiences.push(SharedExperience {
                                experience_type: ExperienceType::Conversation,
                                timestamp: std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs(),
                                emotional_impact: 1.0,
                                location: npc.position,
                                other_participants: conversation.participants.clone(),
                            });
                        }
                    }
                }
                
                // Add memory of conversation
                npc.add_memory(
                    format!("Had a conversation about {}", conversation.current_topic),
                    0.4
                );
            }
        }
    }

    fn process_social_events(&mut self, npcs: &mut HashMap<String, NPC>, world_time: u32) {
        let mut completed_events = Vec::new();
        
        for (i, event) in self.social_events.iter().enumerate() {
            if world_time >= event.start_time && world_time <= event.start_time + event.duration {
                // Event is active
                self.execute_social_event(event, npcs);
            } else if world_time > event.start_time + event.duration {
                // Event is completed
                completed_events.push(i);
            }
        }
        
        // Remove completed events
        for &index in completed_events.iter().rev() {
            let event = self.social_events.remove(index);
            self.complete_social_event(event, npcs);
        }
    }

    fn execute_social_event(&self, event: &SocialEvent, npcs: &mut HashMap<String, NPC>) {
        // Move participants toward event location
        for participant_id in &event.participants {
            if let Some(npc) = npcs.get_mut(participant_id) {
                let distance = npc.distance_to(event.location);
                if distance > 5.0 {
                    // Move toward event location (simplified)
                    let direction = Vec3::new(
                        event.location.x - npc.position.x,
                        event.location.y - npc.position.y,
                        event.location.z - npc.position.z,
                    );
                    let normalized_direction = direction; // Simplified normalization
                    
                    npc.position.x += normalized_direction.x * 0.1;
                    npc.position.y += normalized_direction.y * 0.1;
                    npc.position.z += normalized_direction.z * 0.1;
                }
                
                // Update NPC state based on event type
                match event.event_type {
                    SocialEventType::Party => {
                        npc.mood += 0.2;
                        npc.stress -= 0.1;
                        npc.energy -= 0.1;
                    },
                    SocialEventType::Meeting => {
                        npc.stress += 0.05;
                        npc.energy -= 0.05;
                    },
                    SocialEventType::Competition => {
                        npc.stress += 0.1;
                        npc.energy -= 0.2;
                        npc.mood += 0.1;
                    },
                    _ => {},
                }
            }
        }
    }

    fn complete_social_event(&self, event: SocialEvent, npcs: &mut HashMap<String, NPC>) {
        // Apply event outcomes
        for outcome in &event.outcomes {
            match outcome {
                SocialEventOutcome::RelationshipChanges(changes) => {
                    for (npc_id, relationship_changes) in changes {
                        if let Some(npc) = npcs.get_mut(npc_id) {
                            for other_id in &event.participants {
                                if other_id != npc_id {
                                    if let Some(relationship) = npc.relationships.get_mut(other_id) {
                                        relationship.affection = (relationship.affection + relationship_changes.affection_delta).clamp(0.0, 100.0);
                                        relationship.trust = (relationship.trust + relationship_changes.trust_delta).clamp(0.0, 100.0);
                                        relationship.respect = (relationship.respect + relationship_changes.respect_delta).clamp(0.0, 100.0);
                                        relationship.familiarity += relationship_changes.familiarity_delta;
                                    }
                                }
                            }
                        }
                    }
                },
                SocialEventOutcome::MemoriesCreated(memories) => {
                    for (npc_id, memory_content, importance) in memories {
                        if let Some(npc) = npcs.get_mut(npc_id) {
                            npc.add_memory(memory_content.clone(), *importance);
                        }
                    }
                },
                _ => {}, // Handle other outcomes
            }
        }
    }

    fn update_reputation_propagation(&mut self, npcs: &HashMap<String, NPC>, delta_time: f32) {
        // Simulate reputation spreading through social networks
        for (network_id, network) in &self.social_networks {
            for member_id in &network.members {
                if let Some(reputation) = self.reputation_system.reputation_scores.get(member_id) {
                    // Spread reputation information to other network members
                    for other_member in &network.members {
                        if other_member != member_id {
                            // Simplified reputation propagation
                            // In a full implementation, this would be more sophisticated
                        }
                    }
                }
            }
        }
    }

    fn update_group_dynamics(&mut self, npcs: &HashMap<String, NPC>, delta_time: f32) {
        for (group_id, dynamic) in &mut self.group_dynamics {
            // Calculate group cohesion based on member relationships
            let mut total_relationship_strength = 0.0;
            let mut relationship_count = 0;
            
            for member_id in &dynamic.members {
                if let Some(npc) = npcs.get(member_id) {
                    for other_member in &dynamic.members {
                        if other_member != member_id {
                            if let Some(relationship) = npc.relationships.get(other_member) {
                                total_relationship_strength += relationship.affection + relationship.trust;
                                relationship_count += 2;
                            }
                        }
                    }
                }
            }
            
            if relationship_count > 0 {
                dynamic.cohesion = (total_relationship_strength / relationship_count as f32) / 100.0;
            }
            
            // Update cooperation level based on shared goals and conflicts
            let conflict_penalty = dynamic.internal_conflicts.len() as f32 * 0.1;
            dynamic.cooperation_level = (dynamic.cohesion - conflict_penalty).clamp(0.0, 1.0);
            
            // Update group mood as average of member moods
            let mut total_mood = 0.0;
            let mut mood_count = 0;
            
            for member_id in &dynamic.members {
                if let Some(npc) = npcs.get(member_id) {
                    total_mood += npc.mood;
                    mood_count += 1;
                }
            }
            
            if mood_count > 0 {
                dynamic.group_mood = total_mood / mood_count as f32;
            }
        }
    }

    fn generate_social_interactions(&mut self, npcs: &mut HashMap<String, NPC>, world_time: u32) {
        // Find NPCs that are near each other and might interact
        let mut potential_interactions = Vec::new();
        
        for (npc_id, npc) in npcs.iter() {
            for other_id in &npc.visible_npcs {
                if npc_id < other_id { // Avoid duplicate pairs
                    potential_interactions.push((npc_id.clone(), other_id.clone()));
                }
            }
        }
        
        // Process potential interactions
        for (npc1_id, npc2_id) in potential_interactions {
            if let (Some(npc1), Some(npc2)) = (npcs.get(&npc1_id), npcs.get(&npc2_id)) {
                // Check if they should start interacting
                if self.should_interact(npc1, npc2, world_time) {
                    self.start_interaction(npc1_id, npc2_id, npcs);
                }
            }
        }
    }

    fn should_interact(&self, npc1: &NPC, npc2: &NPC, _world_time: u32) -> bool {
        // Basic criteria for starting an interaction
        if npc1.current_behavior.is_some() || npc2.current_behavior.is_some() {
            return false; // Don't interrupt current behaviors
        }
        
        if npc1.energy < 30.0 || npc2.energy < 30.0 {
            return false; // Too tired to socialize
        }
        
        // Check relationship compatibility
        if let Some(relationship) = npc1.relationships.get(&npc2.id) {
            if relationship.affection < 20.0 {
                return false; // Don't interact with disliked NPCs
            }
        }
        
        // Personality compatibility
        let extroversion1 = npc1.personality.traits.get("extroversion").unwrap_or(&0.5);
        let extroversion2 = npc2.personality.traits.get("extroversion").unwrap_or(&0.5);
        
        let interaction_chance = (extroversion1 + extroversion2) / 2.0;
        
        // Simple random check (would use proper RNG in real implementation)
        interaction_chance > 0.6
    }

    fn start_interaction(&mut self, npc1_id: String, npc2_id: String, npcs: &mut HashMap<String, NPC>) {
        let conversation_id = format!("conv_{}_{}", npc1_id, npc2_id);
        
        // Select appropriate conversation topic
        let topic = self.select_conversation_topic(&npc1_id, &npc2_id, npcs);
        
        let conversation = ActiveConversation {
            participants: vec![npc1_id.clone(), npc2_id.clone()],
            current_topic: topic,
            mood: ConversationMood::Friendly,
            duration: 0.0,
            topic_history: Vec::new(),
            relationship_changes: HashMap::new(),
        };
        
        self.conversation_engine.active_conversations.insert(conversation_id, conversation);
        
        // Set NPCs to socializing state
        if let Some(npc1) = npcs.get_mut(&npc1_id) {
            npc1.state = crate::engine::npc::NPCState::Socializing;
        }
        if let Some(npc2) = npcs.get_mut(&npc2_id) {
            npc2.state = crate::engine::npc::NPCState::Socializing;
        }
    }

    fn select_conversation_topic(&self, npc1_id: &str, npc2_id: &str, npcs: &HashMap<String, NPC>) -> String {
        // Simple topic selection based on shared interests or circumstances
        let default_topics = vec![
            "weather".to_string(),
            "work".to_string(),
            "local_events".to_string(),
            "hobbies".to_string(),
        ];
        
        // In a full implementation, this would be much more sophisticated
        default_topics.get(0).unwrap_or(&"general".to_string()).clone()
    }

    fn initialize_default_topics(&mut self) {
        let topics = vec![
            ("weather", 0.1, 0.2),
            ("work", 0.3, 0.4),
            ("hobbies", 0.4, 0.3),
            ("local_events", 0.5, 0.4),
            ("relationships", 0.7, 0.6),
            ("personal_problems", 0.8, 0.7),
        ];

        for (topic_name, emotional_impact, complexity) in topics {
            let topic = ConversationTopic {
                topic_id: topic_name.to_string(),
                name: topic_name.to_string(),
                interest_level: HashMap::new(),
                emotional_impact,
                complexity,
                prerequisites: Vec::new(),
                outcomes: vec![ConversationOutcome::RelationshipImprove(1.0)],
            };
            
            self.conversation_engine.conversation_topics.insert(topic_name.to_string(), topic);
        }
    }

    fn initialize_dialogue_templates(&mut self) {
        // Initialize basic dialogue templates for different personality types and situations
        let greeting_template = DialogueTemplate {
            template_id: "greeting".to_string(),
            situation: "meeting_someone".to_string(),
            personality_variants: {
                let mut variants = HashMap::new();
                variants.insert("extroversion_high".to_string(), vec![
                    "Hello there! Great to see you!".to_string(),
                    "Hey! How's it going?".to_string(),
                ]);
                variants.insert("extroversion_low".to_string(), vec![
                    "Hi...".to_string(),
                    "Hello.".to_string(),
                ]);
                variants
            },
            emotional_variants: HashMap::new(),
            relationship_variants: HashMap::new(),
        };

        self.conversation_engine.dialogue_templates.insert("greeting".to_string(), greeting_template);
    }

    // Public interface methods
    pub fn create_social_network(&mut self, network_type: NetworkType, members: Vec<String>) -> String {
        let network_id = format!("network_{}", self.social_networks.len());
        
        let network = SocialNetwork {
            network_id: network_id.clone(),
            members,
            network_type,
            influence_map: HashMap::new(),
            shared_interests: Vec::new(),
            meeting_locations: Vec::new(),
            activity_schedule: Vec::new(),
        };
        
        self.social_networks.insert(network_id.clone(), network);
        network_id
    }

    pub fn schedule_social_event(&mut self, event_type: SocialEventType, participants: Vec<String>, location: Point3, start_time: u32, duration: u32) -> String {
        let event_id = format!("event_{}", self.social_events.len());
        
        let event = SocialEvent {
            event_id: event_id.clone(),
            event_type,
            organizer: participants.first().cloned(),
            participants,
            location,
            start_time,
            duration,
            requirements: Vec::new(),
            outcomes: Vec::new(),
        };
        
        self.social_events.push(event);
        event_id
    }

    pub fn get_npc_social_status(&self, npc_id: &str) -> Option<SocialStatus> {
        if let Some(reputation) = self.reputation_system.reputation_scores.get(npc_id) {
            Some(SocialStatus {
                reputation: reputation.overall_reputation,
                social_standing: reputation.social_standing,
                active_relationships: 0, // Would count from relationships
                network_memberships: self.get_network_memberships(npc_id),
            })
        } else {
            None
        }
    }

    fn get_network_memberships(&self, npc_id: &str) -> Vec<String> {
        let mut memberships = Vec::new();
        
        for (network_id, network) in &self.social_networks {
            if network.members.contains(&npc_id.to_string()) {
                memberships.push(network_id.clone());
            }
        }
        
        memberships
    }
}

#[derive(Debug, Clone)]
pub struct SocialStatus {
    pub reputation: f32,
    pub social_standing: f32,
    pub active_relationships: usize,
    pub network_memberships: Vec<String>,
}

impl ConversationEngine {
    pub fn new() -> Self {
        Self {
            conversation_topics: HashMap::new(),
            active_conversations: HashMap::new(),
            dialogue_templates: HashMap::new(),
            personality_responses: HashMap::new(),
        }
    }
}

impl ReputationSystem {
    pub fn new() -> Self {
        Self {
            reputation_scores: HashMap::new(),
            reputation_events: Vec::new(),
            community_opinions: HashMap::new(),
        }
    }
}