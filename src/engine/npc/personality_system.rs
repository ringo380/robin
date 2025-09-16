use crate::engine::npc::{NPC, Personality};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PersonalitySystem {
    personality_models: HashMap<String, PersonalityModel>,
    trait_interactions: TraitInteractionMatrix,
    personality_development: PersonalityDevelopment,
    cultural_influences: CulturalInfluences,
    archetype_templates: HashMap<String, PersonalityArchetype>,
}

#[derive(Debug, Clone)]
pub struct PersonalityModel {
    pub model_name: String,
    pub traits: HashMap<String, TraitDefinition>,
    pub trait_ranges: HashMap<String, (f32, f32)>, // min, max values
    pub compatibility_matrix: HashMap<String, HashMap<String, f32>>,
    pub behavioral_mappings: HashMap<String, BehaviorMapping>,
}

#[derive(Debug, Clone)]
pub struct TraitDefinition {
    pub trait_name: String,
    pub description: String,
    pub behavioral_indicators: Vec<String>,
    pub development_factors: Vec<String>,
    pub interaction_modifiers: HashMap<String, f32>, // How this trait affects interactions with other traits
    pub stability: f32, // How resistant to change this trait is (0.0 = very malleable, 1.0 = fixed)
}

#[derive(Debug, Clone)]
pub struct BehaviorMapping {
    pub trait_name: String,
    pub behavior_modifications: HashMap<String, f32>, // behavior_type -> modifier strength
    pub decision_influences: HashMap<String, f32>,   // decision_type -> influence weight
    pub social_effects: HashMap<String, f32>,        // social_context -> effect modifier
}

#[derive(Debug, Clone)]
pub struct TraitInteractionMatrix {
    pub synergies: HashMap<String, Vec<TraitSynergy>>, // trait -> synergies
    pub conflicts: HashMap<String, Vec<TraitConflict>>, // trait -> conflicts
    pub equilibrium_points: HashMap<String, f32>,      // balanced points for traits
}

#[derive(Debug, Clone)]
pub struct TraitSynergy {
    pub partner_trait: String,
    pub synergy_type: SynergyType,
    pub strength: f32,
    pub manifestations: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum SynergyType {
    Reinforcing,    // Traits strengthen each other
    Complementary,  // Traits complement each other
    Amplifying,     // One trait amplifies the other
    Balancing,      // Traits provide balance
}

#[derive(Debug, Clone)]
pub struct TraitConflict {
    pub conflicting_trait: String,
    pub conflict_type: ConflictType,
    pub intensity: f32,
    pub resolution_strategies: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum ConflictType {
    DirectOpposition,  // Traits are directly opposite
    Tension,          // Traits create internal tension
    Inconsistency,    // Traits lead to inconsistent behavior
    Competition,      // Traits compete for expression
}

#[derive(Debug, Clone)]
pub struct PersonalityDevelopment {
    pub development_stages: Vec<DevelopmentStage>,
    pub life_events: HashMap<String, LifeEvent>,
    pub adaptation_mechanisms: Vec<AdaptationMechanism>,
    pub growth_trajectories: HashMap<String, GrowthTrajectory>,
}

#[derive(Debug, Clone)]
pub struct DevelopmentStage {
    pub stage_name: String,
    pub age_range: (u32, u32), // min, max age
    pub key_developments: Vec<String>,
    pub trait_modifications: HashMap<String, f32>,
    pub vulnerability_factors: Vec<String>,
    pub growth_opportunities: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LifeEvent {
    pub event_type: String,
    pub impact_magnitude: f32,
    pub trait_effects: HashMap<String, f32>, // trait -> change amount
    pub duration_effects: Option<u32>,       // How long effects last (in days)
    pub prerequisite_traits: HashMap<String, f32>, // Required trait levels for event to occur
}

#[derive(Debug, Clone)]
pub struct AdaptationMechanism {
    pub mechanism_name: String,
    pub trigger_conditions: Vec<String>,
    pub adaptation_type: AdaptationType,
    pub effectiveness: f32,
    pub trait_requirements: HashMap<String, f32>,
}

#[derive(Debug, Clone)]
pub enum AdaptationType {
    Coping,          // Dealing with stress/challenges
    Learning,        // Adapting through experience
    Social,          // Adapting to social pressures
    Environmental,   // Adapting to environmental changes
    Growth,          // Personal growth and development
}

#[derive(Debug, Clone)]
pub struct GrowthTrajectory {
    pub trajectory_name: String,
    pub target_traits: HashMap<String, f32>, // Target trait levels
    pub growth_rate: f32,
    pub prerequisites: Vec<String>,
    pub milestones: Vec<GrowthMilestone>,
}

#[derive(Debug, Clone)]
pub struct GrowthMilestone {
    pub milestone_name: String,
    pub required_progress: f32,
    pub rewards: Vec<String>,
    pub unlock_conditions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CulturalInfluences {
    pub cultural_norms: HashMap<String, f32>,    // norm -> strength
    pub social_expectations: HashMap<String, f32>, // expectation -> pressure
    pub value_systems: Vec<ValueSystem>,
    pub cultural_events: Vec<CulturalEvent>,
}

#[derive(Debug, Clone)]
pub struct ValueSystem {
    pub system_name: String,
    pub core_values: HashMap<String, f32>,
    pub behavioral_expectations: Vec<String>,
    pub trait_biases: HashMap<String, f32>, // Tendency to develop certain traits
}

#[derive(Debug, Clone)]
pub struct CulturalEvent {
    pub event_name: String,
    pub frequency: EventFrequency,
    pub participant_effects: HashMap<String, f32>, // trait -> modification
    pub community_impact: f32,
    pub participation_requirements: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum EventFrequency {
    Daily,
    Weekly,
    Monthly,
    Seasonal,
    Annual,
    Rare,
}

#[derive(Debug, Clone)]
pub struct PersonalityArchetype {
    pub archetype_name: String,
    pub trait_profile: HashMap<String, f32>,
    pub typical_behaviors: Vec<String>,
    pub strengths: Vec<String>,
    pub weaknesses: Vec<String>,
    pub development_paths: Vec<String>,
    pub compatibility: HashMap<String, f32>, // archetype -> compatibility score
}

#[derive(Debug, Clone)]
pub struct PersonalityAssessment {
    pub npc_id: String,
    pub trait_scores: HashMap<String, f32>,
    pub dominant_traits: Vec<String>,
    pub archetype_matches: Vec<(String, f32)>, // archetype -> match strength
    pub development_recommendations: Vec<String>,
    pub interaction_preferences: HashMap<String, f32>,
}

#[derive(Debug, Clone)]
pub struct PersonalityChange {
    pub change_id: String,
    pub npc_id: String,
    pub change_type: ChangeType,
    pub affected_traits: HashMap<String, f32>,
    pub trigger_event: String,
    pub change_magnitude: f32,
    pub duration: Option<u32>, // Days, None for permanent
    pub resistance_factors: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum ChangeType {
    Gradual,     // Slow, steady change
    Sudden,      // Rapid change due to event
    Temporary,   // Short-term change
    Cyclical,    // Recurring change pattern
    Progressive, // Building change over time
}

impl PersonalitySystem {
    pub fn new() -> Self {
        let mut system = Self {
            personality_models: HashMap::new(),
            trait_interactions: TraitInteractionMatrix::new(),
            personality_development: PersonalityDevelopment::new(),
            cultural_influences: CulturalInfluences::new(),
            archetype_templates: HashMap::new(),
        };
        
        system.initialize_big_five_model();
        system.initialize_archetypes();
        system.initialize_trait_interactions();
        system
    }

    fn initialize_big_five_model(&mut self) {
        let mut traits = HashMap::new();
        
        // Openness to Experience
        traits.insert("openness".to_string(), TraitDefinition {
            trait_name: "openness".to_string(),
            description: "Appreciation for art, emotion, adventure, unusual ideas, curiosity, and variety".to_string(),
            behavioral_indicators: vec![
                "explores_new_places".to_string(),
                "tries_new_activities".to_string(),
                "asks_questions".to_string(),
                "appreciates_creativity".to_string(),
            ],
            development_factors: vec![
                "education".to_string(),
                "travel".to_string(),
                "diverse_experiences".to_string(),
            ],
            interaction_modifiers: HashMap::new(),
            stability: 0.3, // Moderately malleable
        });

        // Conscientiousness
        traits.insert("conscientiousness".to_string(), TraitDefinition {
            trait_name: "conscientiousness".to_string(),
            description: "Tendency to be organized, dependable, and show self-discipline".to_string(),
            behavioral_indicators: vec![
                "completes_tasks".to_string(),
                "follows_schedules".to_string(),
                "maintains_order".to_string(),
                "sets_goals".to_string(),
            ],
            development_factors: vec![
                "responsibility".to_string(),
                "structure".to_string(),
                "accountability".to_string(),
            ],
            interaction_modifiers: HashMap::new(),
            stability: 0.6, // Relatively stable
        });

        // Extroversion
        traits.insert("extroversion".to_string(), TraitDefinition {
            trait_name: "extroversion".to_string(),
            description: "Tendency to seek stimulation in the company of others".to_string(),
            behavioral_indicators: vec![
                "initiates_conversations".to_string(),
                "seeks_social_events".to_string(),
                "enjoys_groups".to_string(),
                "expresses_emotions".to_string(),
            ],
            development_factors: vec![
                "social_opportunities".to_string(),
                "positive_interactions".to_string(),
                "leadership_roles".to_string(),
            ],
            interaction_modifiers: HashMap::new(),
            stability: 0.7, // Fairly stable
        });

        // Agreeableness
        traits.insert("agreeableness".to_string(), TraitDefinition {
            trait_name: "agreeableness".to_string(),
            description: "Tendency to be compassionate and cooperative rather than suspicious and antagonistic".to_string(),
            behavioral_indicators: vec![
                "helps_others".to_string(),
                "avoids_conflict".to_string(),
                "shows_empathy".to_string(),
                "cooperates_willingly".to_string(),
            ],
            development_factors: vec![
                "positive_relationships".to_string(),
                "community_involvement".to_string(),
                "moral_education".to_string(),
            ],
            interaction_modifiers: HashMap::new(),
            stability: 0.5, // Moderately stable
        });

        // Neuroticism
        traits.insert("neuroticism".to_string(), TraitDefinition {
            trait_name: "neuroticism".to_string(),
            description: "Tendency to experience unpleasant emotions easily, such as anger, anxiety, or depression".to_string(),
            behavioral_indicators: vec![
                "shows_anxiety".to_string(),
                "emotional_volatility".to_string(),
                "stress_sensitivity".to_string(),
                "negative_emotions".to_string(),
            ],
            development_factors: vec![
                "stress_exposure".to_string(),
                "trauma_history".to_string(),
                "coping_skills".to_string(),
            ],
            interaction_modifiers: HashMap::new(),
            stability: 0.4, // Somewhat malleable
        });

        let big_five_model = PersonalityModel {
            model_name: "Big Five".to_string(),
            traits,
            trait_ranges: {
                let mut ranges = HashMap::new();
                ranges.insert("openness".to_string(), (0.0, 1.0));
                ranges.insert("conscientiousness".to_string(), (0.0, 1.0));
                ranges.insert("extroversion".to_string(), (0.0, 1.0));
                ranges.insert("agreeableness".to_string(), (0.0, 1.0));
                ranges.insert("neuroticism".to_string(), (0.0, 1.0));
                ranges
            },
            compatibility_matrix: HashMap::new(),
            behavioral_mappings: HashMap::new(),
        };

        self.personality_models.insert("big_five".to_string(), big_five_model);
    }

    fn initialize_archetypes(&mut self) {
        // The Explorer
        self.archetype_templates.insert("explorer".to_string(), PersonalityArchetype {
            archetype_name: "The Explorer".to_string(),
            trait_profile: {
                let mut profile = HashMap::new();
                profile.insert("openness".to_string(), 0.9);
                profile.insert("conscientiousness".to_string(), 0.4);
                profile.insert("extroversion".to_string(), 0.6);
                profile.insert("agreeableness".to_string(), 0.6);
                profile.insert("neuroticism".to_string(), 0.3);
                profile
            },
            typical_behaviors: vec![
                "seeks_new_experiences".to_string(),
                "takes_calculated_risks".to_string(),
                "questions_conventions".to_string(),
                "adapts_quickly".to_string(),
            ],
            strengths: vec![
                "innovation".to_string(),
                "adaptability".to_string(),
                "curiosity".to_string(),
                "creativity".to_string(),
            ],
            weaknesses: vec![
                "inconsistency".to_string(),
                "impulsiveness".to_string(),
                "difficulty_with_routine".to_string(),
            ],
            development_paths: vec![
                "focus_development".to_string(),
                "leadership_skills".to_string(),
                "patience_building".to_string(),
            ],
            compatibility: HashMap::new(),
        });

        // The Guardian
        self.archetype_templates.insert("guardian".to_string(), PersonalityArchetype {
            archetype_name: "The Guardian".to_string(),
            trait_profile: {
                let mut profile = HashMap::new();
                profile.insert("openness".to_string(), 0.3);
                profile.insert("conscientiousness".to_string(), 0.9);
                profile.insert("extroversion".to_string(), 0.5);
                profile.insert("agreeableness".to_string(), 0.8);
                profile.insert("neuroticism".to_string(), 0.2);
                profile
            },
            typical_behaviors: vec![
                "protects_others".to_string(),
                "maintains_traditions".to_string(),
                "follows_rules".to_string(),
                "provides_stability".to_string(),
            ],
            strengths: vec![
                "reliability".to_string(),
                "loyalty".to_string(),
                "organization".to_string(),
                "responsibility".to_string(),
            ],
            weaknesses: vec![
                "inflexibility".to_string(),
                "resistance_to_change".to_string(),
                "over_protection".to_string(),
            ],
            development_paths: vec![
                "flexibility_training".to_string(),
                "innovation_appreciation".to_string(),
                "delegation_skills".to_string(),
            ],
            compatibility: HashMap::new(),
        });

        // The Socializer
        self.archetype_templates.insert("socializer".to_string(), PersonalityArchetype {
            archetype_name: "The Socializer".to_string(),
            trait_profile: {
                let mut profile = HashMap::new();
                profile.insert("openness".to_string(), 0.6);
                profile.insert("conscientiousness".to_string(), 0.5);
                profile.insert("extroversion".to_string(), 0.9);
                profile.insert("agreeableness".to_string(), 0.8);
                profile.insert("neuroticism".to_string(), 0.3);
                profile
            },
            typical_behaviors: vec![
                "initiates_social_events".to_string(),
                "mediates_conflicts".to_string(),
                "builds_networks".to_string(),
                "shares_information".to_string(),
            ],
            strengths: vec![
                "communication".to_string(),
                "networking".to_string(),
                "empathy".to_string(),
                "enthusiasm".to_string(),
            ],
            weaknesses: vec![
                "over_socializing".to_string(),
                "difficulty_being_alone".to_string(),
                "people_pleasing".to_string(),
            ],
            development_paths: vec![
                "independence_building".to_string(),
                "analytical_skills".to_string(),
                "boundary_setting".to_string(),
            ],
            compatibility: HashMap::new(),
        });
    }

    fn initialize_trait_interactions(&mut self) {
        // Openness synergies
        let openness_synergies = vec![
            TraitSynergy {
                partner_trait: "extroversion".to_string(),
                synergy_type: SynergyType::Amplifying,
                strength: 0.6,
                manifestations: vec!["social_exploration".to_string(), "adventurous_socializing".to_string()],
            },
            TraitSynergy {
                partner_trait: "conscientiousness".to_string(),
                synergy_type: SynergyType::Balancing,
                strength: 0.4,
                manifestations: vec!["structured_creativity".to_string(), "planned_exploration".to_string()],
            },
        ];

        self.trait_interactions.synergies.insert("openness".to_string(), openness_synergies);

        // Conscientiousness conflicts
        let conscientiousness_conflicts = vec![
            TraitConflict {
                conflicting_trait: "openness".to_string(),
                conflict_type: ConflictType::Tension,
                intensity: 0.3,
                resolution_strategies: vec![
                    "scheduled_exploration".to_string(),
                    "structured_creativity".to_string(),
                ],
            },
        ];

        self.trait_interactions.conflicts.insert("conscientiousness".to_string(), conscientiousness_conflicts);

        // Set equilibrium points
        self.trait_interactions.equilibrium_points.insert("openness".to_string(), 0.5);
        self.trait_interactions.equilibrium_points.insert("conscientiousness".to_string(), 0.6);
        self.trait_interactions.equilibrium_points.insert("extroversion".to_string(), 0.5);
        self.trait_interactions.equilibrium_points.insert("agreeableness".to_string(), 0.7);
        self.trait_interactions.equilibrium_points.insert("neuroticism".to_string(), 0.3);
    }

    pub fn update(&mut self, npcs: &mut HashMap<String, NPC>, delta_time: f32, life_events: &[String]) {
        for (npc_id, npc) in npcs.iter_mut() {
            // Apply gradual personality development
            self.apply_gradual_development(npc, delta_time);
            
            // Process life events
            for event in life_events {
                if self.is_event_relevant_to_npc(event, npc) {
                    self.apply_life_event_effects(event, npc);
                }
            }
            
            // Apply cultural influences
            self.apply_cultural_influences(npc, delta_time);
            
            // Resolve trait conflicts
            self.resolve_trait_conflicts(npc);
            
            // Update behavioral modifiers
            self.update_behavioral_modifiers(npc);
        }
    }

    fn apply_gradual_development(&self, npc: &mut NPC, delta_time: f32) {
        // Gradual trait development based on recent experiences
        for (trait_name, current_value) in &mut npc.personality.traits {
            if let Some(equilibrium) = self.trait_interactions.equilibrium_points.get(trait_name) {
                // Tendency to move towards equilibrium
                let drift_towards_equilibrium = (*equilibrium - *current_value) * 0.001 * delta_time;
                *current_value += drift_towards_equilibrium;
            }
            
            // Apply experience-based changes (simplified)
            match npc.state {
                crate::engine::npc::NPCState::Socializing => {
                    if trait_name == "extroversion" {
                        *current_value += 0.0001 * delta_time; // Very small increase
                    }
                },
                crate::engine::npc::NPCState::Working => {
                    if trait_name == "conscientiousness" {
                        *current_value += 0.0001 * delta_time;
                    }
                },
                _ => {},
            }
            
            // Ensure values stay within bounds
            *current_value = current_value.clamp(0.0, 1.0);
        }
    }

    fn is_event_relevant_to_npc(&self, event: &str, npc: &NPC) -> bool {
        // Check if the life event should affect this NPC
        // This is a simplified check - in reality, would be more sophisticated
        match event {
            "community_celebration" => true,  // Affects everyone
            "natural_disaster" => true,       // Affects everyone
            "personal_achievement" => {
                // Only affects NPCs with high conscientiousness or relevant skills
                npc.personality.traits.get("conscientiousness").unwrap_or(&0.5) > &0.7
            },
            "social_conflict" => {
                // More likely to affect extroverted or agreeable NPCs
                let extroversion = npc.personality.traits.get("extroversion").unwrap_or(&0.5);
                let agreeableness = npc.personality.traits.get("agreeableness").unwrap_or(&0.5);
                extroversion > &0.6 || agreeableness > &0.7
            },
            _ => false,
        }
    }

    fn apply_life_event_effects(&self, event: &str, npc: &mut NPC) {
        // Apply personality changes based on life events
        match event {
            "community_celebration" => {
                // Positive event - increases mood-related traits
                if let Some(extroversion) = npc.personality.traits.get_mut("extroversion") {
                    *extroversion = (*extroversion + 0.05).min(1.0);
                }
                if let Some(agreeableness) = npc.personality.traits.get_mut("agreeableness") {
                    *agreeableness = (*agreeableness + 0.03).min(1.0);
                }
                npc.mood += 15.0;
                npc.stress -= 10.0;
            },
            
            "natural_disaster" => {
                // Stressful event - increases neuroticism, may decrease openness
                if let Some(neuroticism) = npc.personality.traits.get_mut("neuroticism") {
                    *neuroticism = (*neuroticism + 0.1).min(1.0);
                }
                if let Some(openness) = npc.personality.traits.get_mut("openness") {
                    *openness = (*openness - 0.05).max(0.0);
                }
                npc.mood -= 20.0;
                npc.stress += 30.0;
            },
            
            "personal_achievement" => {
                // Achievement - increases conscientiousness and reduces neuroticism
                if let Some(conscientiousness) = npc.personality.traits.get_mut("conscientiousness") {
                    *conscientiousness = (*conscientiousness + 0.08).min(1.0);
                }
                if let Some(neuroticism) = npc.personality.traits.get_mut("neuroticism") {
                    *neuroticism = (*neuroticism - 0.05).max(0.0);
                }
                npc.mood += 25.0;
                npc.stress -= 15.0;
            },
            
            "social_conflict" => {
                // Conflict - may decrease agreeableness, increase neuroticism
                if let Some(agreeableness) = npc.personality.traits.get_mut("agreeableness") {
                    *agreeableness = (*agreeableness - 0.06).max(0.0);
                }
                if let Some(neuroticism) = npc.personality.traits.get_mut("neuroticism") {
                    *neuroticism = (*neuroticism + 0.07).min(1.0);
                }
                npc.mood -= 15.0;
                npc.stress += 20.0;
            },
            
            _ => {},
        }
        
        // Clamp all values to valid ranges
        npc.mood = npc.mood.clamp(0.0, 100.0);
        npc.stress = npc.stress.clamp(0.0, 100.0);
    }

    fn apply_cultural_influences(&self, npc: &mut NPC, delta_time: f32) {
        // Apply subtle cultural pressure towards certain personality norms
        for (norm, strength) in &self.cultural_influences.cultural_norms {
            match norm.as_str() {
                "cooperation" => {
                    if let Some(agreeableness) = npc.personality.traits.get_mut("agreeableness") {
                        let pressure = (0.7 - *agreeableness) * strength * 0.0001 * delta_time;
                        *agreeableness += pressure;
                    }
                },
                "hard_work" => {
                    if let Some(conscientiousness) = npc.personality.traits.get_mut("conscientiousness") {
                        let pressure = (0.6 - *conscientiousness) * strength * 0.0001 * delta_time;
                        *conscientiousness += pressure;
                    }
                },
                "innovation" => {
                    if let Some(openness) = npc.personality.traits.get_mut("openness") {
                        let pressure = (0.5 - *openness) * strength * 0.0001 * delta_time;
                        *openness += pressure;
                    }
                },
                _ => {},
            }
        }
    }

    fn resolve_trait_conflicts(&self, npc: &mut NPC) {
        // Handle internal personality conflicts
        for (trait_name, conflicts) in &self.trait_interactions.conflicts {
            if let Some(trait_value) = npc.personality.traits.get(trait_name) {
                for conflict in conflicts {
                    if let Some(conflicting_value) = npc.personality.traits.get(&conflict.conflicting_trait) {
                        // Check if traits are in conflict
                        let conflict_strength = match conflict.conflict_type {
                            ConflictType::DirectOpposition => {
                                // High values in both opposing traits create stress
                                if *trait_value > 0.7 && *conflicting_value > 0.7 {
                                    conflict.intensity * 0.5
                                } else {
                                    0.0
                                }
                            },
                            ConflictType::Tension => {
                                // Moderate values in conflicting traits create mild tension
                                (trait_value * conflicting_value) * conflict.intensity * 0.3
                            },
                            _ => 0.0, // Simplified for other conflict types
                        };
                        
                        if conflict_strength > 0.1 {
                            // Apply stress due to personality conflict
                            npc.stress += conflict_strength * 2.0;
                            npc.mood -= conflict_strength * 1.0;
                        }
                    }
                }
            }
        }
    }

    fn update_behavioral_modifiers(&self, npc: &mut NPC) {
        // Update behavioral modifiers based on current personality
        npc.personality.behavioral_modifiers.clear();
        
        for (trait_name, trait_value) in &npc.personality.traits {
            match trait_name.as_str() {
                "extroversion" => {
                    npc.personality.behavioral_modifiers.insert("social_frequency".to_string(), *trait_value);
                    npc.personality.behavioral_modifiers.insert("group_preference".to_string(), *trait_value);
                },
                "conscientiousness" => {
                    npc.personality.behavioral_modifiers.insert("work_dedication".to_string(), *trait_value);
                    npc.personality.behavioral_modifiers.insert("routine_adherence".to_string(), *trait_value);
                },
                "openness" => {
                    npc.personality.behavioral_modifiers.insert("exploration_tendency".to_string(), *trait_value);
                    npc.personality.behavioral_modifiers.insert("change_acceptance".to_string(), *trait_value);
                },
                "agreeableness" => {
                    npc.personality.behavioral_modifiers.insert("cooperation_willingness".to_string(), *trait_value);
                    npc.personality.behavioral_modifiers.insert("conflict_avoidance".to_string(), *trait_value);
                },
                "neuroticism" => {
                    npc.personality.behavioral_modifiers.insert("stress_sensitivity".to_string(), *trait_value);
                    npc.personality.behavioral_modifiers.insert("emotional_volatility".to_string(), *trait_value);
                },
                _ => {},
            }
        }
    }

    pub fn assess_personality(&self, npc: &NPC) -> PersonalityAssessment {
        let mut dominant_traits = Vec::new();
        let mut archetype_matches = Vec::new();
        
        // Find dominant traits (above 0.7)
        for (trait_name, value) in &npc.personality.traits {
            if *value > 0.7 {
                dominant_traits.push(trait_name.clone());
            }
        }
        
        // Calculate archetype matches
        for (archetype_name, archetype) in &self.archetype_templates {
            let mut match_score = 0.0;
            let mut trait_count = 0;
            
            for (trait_name, archetype_value) in &archetype.trait_profile {
                if let Some(npc_value) = npc.personality.traits.get(trait_name) {
                    let difference = (npc_value - archetype_value).abs();
                    match_score += 1.0 - difference; // Closer values = higher score
                    trait_count += 1;
                }
            }
            
            if trait_count > 0 {
                match_score /= trait_count as f32;
                archetype_matches.push((archetype_name.clone(), match_score));
            }
        }
        
        // Sort archetype matches by score
        archetype_matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Generate development recommendations
        let development_recommendations = self.generate_development_recommendations(npc);
        
        PersonalityAssessment {
            npc_id: npc.id.clone(),
            trait_scores: npc.personality.traits.clone(),
            dominant_traits,
            archetype_matches,
            development_recommendations,
            interaction_preferences: self.calculate_interaction_preferences(npc),
        }
    }

    fn generate_development_recommendations(&self, npc: &NPC) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        // Check for extreme trait values that might benefit from balancing
        for (trait_name, value) in &npc.personality.traits {
            if *value > 0.9 {
                recommendations.push(format!("Consider developing balancing traits to complement high {}", trait_name));
            } else if *value < 0.1 {
                recommendations.push(format!("Consider activities that could develop {}", trait_name));
            }
        }
        
        // Check for trait conflicts
        for (trait_name, conflicts) in &self.trait_interactions.conflicts {
            if let Some(trait_value) = npc.personality.traits.get(trait_name) {
                for conflict in conflicts {
                    if let Some(conflicting_value) = npc.personality.traits.get(&conflict.conflicting_trait) {
                        if *trait_value > 0.7 && *conflicting_value > 0.7 {
                            recommendations.push(format!("Work on resolving internal conflict between {} and {}", trait_name, conflict.conflicting_trait));
                        }
                    }
                }
            }
        }
        
        recommendations
    }

    fn calculate_interaction_preferences(&self, npc: &NPC) -> HashMap<String, f32> {
        let mut preferences = HashMap::new();
        
        let extroversion = npc.personality.traits.get("extroversion").unwrap_or(&0.5);
        let agreeableness = npc.personality.traits.get("agreeableness").unwrap_or(&0.5);
        let openness = npc.personality.traits.get("openness").unwrap_or(&0.5);
        
        preferences.insert("group_activities".to_string(), *extroversion);
        preferences.insert("one_on_one".to_string(), 1.0 - *extroversion + 0.3); // Introverts prefer but extroverts also okay
        preferences.insert("cooperative_tasks".to_string(), *agreeableness);
        preferences.insert("competitive_activities".to_string(), 1.0 - *agreeableness);
        preferences.insert("novel_experiences".to_string(), *openness);
        preferences.insert("routine_activities".to_string(), 1.0 - *openness + 0.2);
        
        preferences
    }

    pub fn predict_compatibility(&self, npc1: &NPC, npc2: &NPC) -> f32 {
        let mut compatibility_score = 0.0;
        let mut factor_count = 0;
        
        // Compare trait compatibility
        for (trait_name, value1) in &npc1.personality.traits {
            if let Some(value2) = npc2.personality.traits.get(trait_name) {
                let similarity = 1.0 - (value1 - value2).abs();
                
                // Some traits are better when similar, others when complementary
                let adjusted_similarity = match trait_name.as_str() {
                    "extroversion" => {
                        // Moderate difference can be good (extrovert + introvert balance)
                        if (value1 - value2).abs() < 0.4 && (value1 - value2).abs() > 0.1 {
                            0.8
                        } else {
                            similarity
                        }
                    },
                    "agreeableness" => similarity * 1.2, // High similarity is very good
                    "conscientiousness" => similarity,   // Similar levels work well
                    "neuroticism" => {
                        // Lower neuroticism is generally better for relationships
                        let avg_neuroticism = (value1 + value2) / 2.0;
                        similarity * (1.0 - avg_neuroticism * 0.5)
                    },
                    "openness" => {
                        // Some difference can be stimulating
                        if (value1 - value2).abs() < 0.3 {
                            similarity
                        } else {
                            similarity * 0.8
                        }
                    },
                    _ => similarity,
                };
                
                compatibility_score += adjusted_similarity;
                factor_count += 1;
            }
        }
        
        if factor_count > 0 {
            compatibility_score /= factor_count as f32;
        }
        
        compatibility_score.clamp(0.0, 1.0)
    }
}

impl TraitInteractionMatrix {
    pub fn new() -> Self {
        Self {
            synergies: HashMap::new(),
            conflicts: HashMap::new(),
            equilibrium_points: HashMap::new(),
        }
    }
}

impl PersonalityDevelopment {
    pub fn new() -> Self {
        Self {
            development_stages: Vec::new(),
            life_events: HashMap::new(),
            adaptation_mechanisms: Vec::new(),
            growth_trajectories: HashMap::new(),
        }
    }
}

impl CulturalInfluences {
    pub fn new() -> Self {
        Self {
            cultural_norms: {
                let mut norms = HashMap::new();
                norms.insert("cooperation".to_string(), 0.7);
                norms.insert("hard_work".to_string(), 0.8);
                norms.insert("innovation".to_string(), 0.5);
                norms.insert("respect_for_authority".to_string(), 0.6);
                norms
            },
            social_expectations: HashMap::new(),
            value_systems: Vec::new(),
            cultural_events: Vec::new(),
        }
    }
}