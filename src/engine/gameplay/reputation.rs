//! Reputation and Social Systems for Robin Engine
//!
//! Multi-faction reputation management, social standing tracking,
//! NPC relationship systems, and community reputation mechanics.
//! Integrates with guild systems and multiplayer collaboration.

use crate::engine::{
    error::{RobinError, RobinResult},
    save_system::PlayerData,
    math::Vec3,
};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use rand;

/// Core reputation system managing faction standing and social relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationManager {
    pub faction_standings: HashMap<FactionId, FactionStanding>,
    pub npc_relationships: HashMap<NpcId, NpcRelationship>,
    pub community_reputation: CommunityReputation,
    pub social_history: Vec<SocialEvent>,
    pub reputation_config: ReputationConfig,
    pub social_analytics: SocialAnalytics,
}

impl ReputationManager {
    pub fn new() -> Self {
        Self {
            faction_standings: HashMap::new(),
            npc_relationships: HashMap::new(),
            community_reputation: CommunityReputation::default(),
            social_history: Vec::new(),
            reputation_config: ReputationConfig::default(),
            social_analytics: SocialAnalytics::default(),
        }
    }

    /// Initialize reputation system with player data
    pub fn initialize(&mut self, player_data: &PlayerData) -> RobinResult<()> {
        // Initialize faction standings
        self.initialize_default_factions()?;

        // Load existing reputation data from player save
        if let Some(reputation_data) = player_data.custom_data.get("reputation") {
            self.load_reputation_data(reputation_data)?;
        }

        // Initialize community standing
        self.community_reputation.initialize_community_standing();

        println!("🤝 ReputationManager initialized with {} factions and {} NPC relationships",
                self.faction_standings.len(), self.npc_relationships.len());
        Ok(())
    }

    /// Update reputation system processing
    pub fn update(&mut self, delta_time: f32, player_data: &mut PlayerData) -> RobinResult<()> {
        // Process reputation decay over time
        self.process_reputation_decay(delta_time)?;

        // Update social analytics
        self.social_analytics.update(delta_time, &self.faction_standings, &self.npc_relationships)?;

        // Process pending social events
        self.process_social_events(player_data)?;

        // Update community reputation based on recent actions
        self.community_reputation.update(delta_time, &self.social_history)?;

        Ok(())
    }

    /// Modify faction standing
    pub fn modify_faction_standing(&mut self,
                                  faction_id: FactionId,
                                  amount: i32,
                                  reason: String,
                                  player_data: &mut PlayerData) -> RobinResult<ReputationChange> {
        let mut standing = self.faction_standings.entry(faction_id.clone())
            .or_insert_with(|| FactionStanding::new(faction_id.clone()));

        let old_value = standing.reputation_value;
        let old_tier = standing.reputation_tier;

        // Apply reputation change with modifiers
        let modified_amount = self.calculate_reputation_modifier(amount, &faction_id, player_data);
        standing.reputation_value = (standing.reputation_value + modified_amount)
            .clamp(-1000, 1000);

        // Update reputation tier
        standing.update_reputation_tier();

        // Record social event
        let event = SocialEvent {
            event_id: format!("rep_{}_{}", chrono::Utc::now().timestamp(), rand::random::<u32>()),
            event_type: SocialEventType::FactionStandingChange,
            timestamp: Utc::now(),
            faction_id: Some(faction_id.clone()),
            npc_id: None,
            value_change: modified_amount,
            reason: reason.clone(),
            context: HashMap::new(),
        };
        self.social_history.push(event);

        // Check for tier changes and notifications
        let tier_changed = old_tier != standing.reputation_tier;
        if tier_changed {
            self.handle_reputation_tier_change(&faction_id, old_tier, standing.reputation_tier, player_data)?;
        }

        // Update player stats
        player_data.stats.custom_stats.insert(
            format!("reputation_{}", faction_id.name.to_lowercase()),
            standing.reputation_value as f32
        );

        Ok(ReputationChange {
            faction_id: faction_id.clone(),
            old_value,
            new_value: standing.reputation_value,
            change_amount: modified_amount,
            old_tier,
            new_tier: standing.reputation_tier,
            tier_changed,
            reason,
        })
    }

    /// Modify NPC relationship
    pub fn modify_npc_relationship(&mut self,
                                  npc_id: NpcId,
                                  amount: i32,
                                  interaction_type: InteractionType,
                                  player_data: &mut PlayerData) -> RobinResult<RelationshipChange> {
        let mut relationship = self.npc_relationships.entry(npc_id.clone())
            .or_insert_with(|| NpcRelationship::new(npc_id.clone()));

        let old_value = relationship.relationship_value;
        let old_status = relationship.relationship_status;

        // Apply relationship change
        relationship.relationship_value = (relationship.relationship_value + amount)
            .clamp(-100, 100);
        relationship.last_interaction = Utc::now();
        relationship.interaction_count += 1;
        relationship.interaction_history.push(InteractionRecord {
            timestamp: Utc::now(),
            interaction_type,
            value_change: amount,
            context: HashMap::new(),
        });

        // Update relationship status
        relationship.update_relationship_status();

        // Record social event
        let event = SocialEvent {
            event_id: format!("rep_{}_{}", chrono::Utc::now().timestamp(), rand::random::<u32>()),
            event_type: SocialEventType::NpcRelationshipChange,
            timestamp: Utc::now(),
            faction_id: None,
            npc_id: Some(npc_id.clone()),
            value_change: amount,
            reason: format!("{:?} interaction", interaction_type),
            context: HashMap::new(),
        };
        self.social_history.push(event);

        // Check for status changes
        let status_changed = old_status != relationship.relationship_status;
        if status_changed {
            self.handle_relationship_status_change(&npc_id, old_status, relationship.relationship_status, player_data)?;
        }

        Ok(RelationshipChange {
            npc_id: npc_id.clone(),
            old_value,
            new_value: relationship.relationship_value,
            change_amount: amount,
            old_status,
            new_status: relationship.relationship_status,
            status_changed,
            interaction_type,
        })
    }

    /// Get faction standing information
    pub fn get_faction_standing(&self, faction_id: &FactionId) -> Option<&FactionStanding> {
        self.faction_standings.get(faction_id)
    }

    /// Get NPC relationship information
    pub fn get_npc_relationship(&self, npc_id: &NpcId) -> Option<&NpcRelationship> {
        self.npc_relationships.get(npc_id)
    }

    /// Get reputation summary for UI display
    pub fn get_reputation_summary(&self) -> ReputationSummary {
        let faction_count = self.faction_standings.len();
        let npc_count = self.npc_relationships.len();

        let average_faction_standing = if faction_count > 0 {
            self.faction_standings.values()
                .map(|s| s.reputation_value)
                .sum::<i32>() as f32 / faction_count as f32
        } else {
            0.0
        };

        let average_npc_relationship = if npc_count > 0 {
            self.npc_relationships.values()
                .map(|r| r.relationship_value)
                .sum::<i32>() as f32 / npc_count as f32
        } else {
            0.0
        };

        ReputationSummary {
            faction_standings: self.faction_standings.clone(),
            npc_relationships: self.npc_relationships.clone(),
            community_reputation: self.community_reputation.clone(),
            average_faction_standing,
            average_npc_relationship,
            total_social_events: self.social_history.len(),
            social_analytics: self.social_analytics.clone(),
        }
    }

    /// Check if player can access faction-specific content
    pub fn can_access_faction_content(&self, faction_id: &FactionId, required_tier: ReputationTier) -> bool {
        if let Some(standing) = self.faction_standings.get(faction_id) {
            standing.reputation_tier as u8 >= required_tier as u8
        } else {
            false
        }
    }

    /// Get reputation-based modifiers for gameplay
    pub fn get_reputation_modifiers(&self, player_data: &PlayerData) -> ReputationModifiers {
        let mut modifiers = ReputationModifiers::default();

        // Calculate faction-based modifiers
        for (faction_id, standing) in &self.faction_standings {
            match standing.reputation_tier {
                ReputationTier::Exalted => {
                    modifiers.trading_discount += 0.15;
                    modifiers.quest_reward_bonus += 0.20;
                }
                ReputationTier::Revered => {
                    modifiers.trading_discount += 0.10;
                    modifiers.quest_reward_bonus += 0.15;
                }
                ReputationTier::Honored => {
                    modifiers.trading_discount += 0.05;
                    modifiers.quest_reward_bonus += 0.10;
                }
                ReputationTier::Hostile => {
                    modifiers.trading_penalty += 0.25;
                    modifiers.quest_availability_penalty += 0.50;
                }
                ReputationTier::Hated => {
                    modifiers.trading_penalty += 0.50;
                    modifiers.quest_availability_penalty += 0.75;
                }
                _ => {}
            }
        }

        // Calculate community-based modifiers
        match self.community_reputation.community_standing {
            CommunityStanding::Legend => {
                modifiers.experience_bonus += 0.25;
                modifiers.resource_find_bonus += 0.20;
            }
            CommunityStanding::Hero => {
                modifiers.experience_bonus += 0.15;
                modifiers.resource_find_bonus += 0.10;
            }
            CommunityStanding::Respected => {
                modifiers.experience_bonus += 0.05;
            }
            CommunityStanding::Outcast => {
                modifiers.experience_penalty += 0.20;
                modifiers.social_penalty += 0.30;
            }
            CommunityStanding::Villain => {
                modifiers.experience_penalty += 0.35;
                modifiers.social_penalty += 0.50;
            }
            _ => {}
        }

        modifiers
    }

    /// Process social events and their consequences
    fn process_social_events(&mut self, player_data: &mut PlayerData) -> RobinResult<()> {
        // Process events from the last update cycle
        let recent_events: Vec<_> = self.social_history.iter()
            .filter(|event| event.timestamp > Utc::now() - Duration::minutes(1))
            .collect();

        for event in recent_events {
            match event.event_type {
                SocialEventType::FactionStandingChange => {
                    // Process faction-related consequences
                    if let Some(faction_id) = &event.faction_id {
                        self.process_faction_consequences(faction_id, event.value_change, player_data)?;
                    }
                }
                SocialEventType::NpcRelationshipChange => {
                    // Process NPC-related consequences
                    if let Some(npc_id) = &event.npc_id {
                        self.process_npc_consequences(npc_id, event.value_change, player_data)?;
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Initialize default game factions
    fn initialize_default_factions(&mut self) -> RobinResult<()> {
        let default_factions = vec![
            FactionId { name: "Builders Guild".to_string(), faction_type: FactionType::Guild },
            FactionId { name: "Miners Union".to_string(), faction_type: FactionType::Guild },
            FactionId { name: "Architects Society".to_string(), faction_type: FactionType::Professional },
            FactionId { name: "Engineers Collective".to_string(), faction_type: FactionType::Professional },
            FactionId { name: "Merchants Alliance".to_string(), faction_type: FactionType::Trade },
            FactionId { name: "Crafters Consortium".to_string(), faction_type: FactionType::Trade },
            FactionId { name: "City Council".to_string(), faction_type: FactionType::Government },
            FactionId { name: "Environmental Coalition".to_string(), faction_type: FactionType::Advocacy },
        ];

        for faction_id in default_factions {
            self.faction_standings.insert(faction_id.clone(), FactionStanding::new(faction_id));
        }

        Ok(())
    }

    /// Calculate reputation modifier based on various factors
    fn calculate_reputation_modifier(&self, base_amount: i32, faction_id: &FactionId, player_data: &PlayerData) -> i32 {
        let mut modifier = 1.0;

        // Charisma bonus from character attributes
        if let Some(charisma) = player_data.stats.custom_stats.get("charisma") {
            modifier += charisma * 0.01; // 1% per charisma point
        }

        // Community reputation modifier
        match self.community_reputation.community_standing {
            CommunityStanding::Legend => modifier += 0.25,
            CommunityStanding::Hero => modifier += 0.15,
            CommunityStanding::Respected => modifier += 0.05,
            CommunityStanding::Outcast => modifier -= 0.15,
            CommunityStanding::Villain => modifier -= 0.30,
            _ => {}
        }

        // Apply faction-specific modifiers
        modifier += self.get_faction_specific_modifier(faction_id);

        (base_amount as f32 * modifier) as i32
    }

    /// Get faction-specific reputation modifiers
    fn get_faction_specific_modifier(&self, faction_id: &FactionId) -> f32 {
        match faction_id.faction_type {
            FactionType::Guild => 0.1,      // Guilds are easier to gain reputation with
            FactionType::Government => -0.1, // Government factions are harder
            FactionType::Trade => 0.05,     // Trade factions moderate
            _ => 0.0,
        }
    }

    /// Handle reputation tier changes and notifications
    fn handle_reputation_tier_change(&mut self,
                                   faction_id: &FactionId,
                                   old_tier: ReputationTier,
                                   new_tier: ReputationTier,
                                   player_data: &mut PlayerData) -> RobinResult<()> {
        println!("🏆 Reputation with {} changed from {:?} to {:?}",
                faction_id.name, old_tier, new_tier);

        // Unlock faction-specific content based on new tier
        match new_tier {
            ReputationTier::Exalted => {
                // Unlock elite content and abilities
                player_data.unlocked_features.insert(format!("{}_exalted_access", faction_id.name));
            }
            ReputationTier::Revered => {
                // Unlock advanced content
                player_data.unlocked_features.insert(format!("{}_advanced_access", faction_id.name));
            }
            ReputationTier::Honored => {
                // Unlock basic faction content
                player_data.unlocked_features.insert(format!("{}_basic_access", faction_id.name));
            }
            _ => {}
        }

        Ok(())
    }

    /// Handle NPC relationship status changes
    fn handle_relationship_status_change(&mut self,
                                       npc_id: &NpcId,
                                       old_status: RelationshipStatus,
                                       new_status: RelationshipStatus,
                                       player_data: &mut PlayerData) -> RobinResult<()> {
        println!("💫 Relationship with {} changed from {:?} to {:?}",
                npc_id.name, old_status, new_status);

        // Process relationship-specific benefits
        match new_status {
            RelationshipStatus::BestFriend => {
                // Unlock special NPC interactions and benefits
                player_data.unlocked_features.insert(format!("{}_best_friend", npc_id.name));
            }
            RelationshipStatus::Friend => {
                // Unlock friendship benefits
                player_data.unlocked_features.insert(format!("{}_friend", npc_id.name));
            }
            RelationshipStatus::Enemy => {
                // Apply enemy penalties
                player_data.unlocked_features.remove(&format!("{}_friend", npc_id.name));
            }
            _ => {}
        }

        Ok(())
    }

    /// Process faction-related consequences
    fn process_faction_consequences(&mut self, faction_id: &FactionId, value_change: i32, player_data: &mut PlayerData) -> RobinResult<()> {
        // Implement faction-specific consequence logic
        // This could include rival faction relationships, quest availability, etc.
        Ok(())
    }

    /// Process NPC-related consequences
    fn process_npc_consequences(&mut self, npc_id: &NpcId, value_change: i32, player_data: &mut PlayerData) -> RobinResult<()> {
        // Implement NPC-specific consequence logic
        // This could include friend/family relationships, recommendation letters, etc.
        Ok(())
    }

    /// Process reputation decay over time
    fn process_reputation_decay(&mut self, delta_time: f32) -> RobinResult<()> {
        let decay_rate = self.reputation_config.reputation_decay_rate * delta_time;

        // Apply decay to extreme reputation values
        for standing in self.faction_standings.values_mut() {
            if standing.reputation_value.abs() > 500 {
                let decay_amount = (standing.reputation_value as f32 * decay_rate) as i32;
                standing.reputation_value -= decay_amount.signum() * decay_amount.abs().min(1);
                standing.update_reputation_tier();
            }
        }

        // Apply decay to NPC relationships
        for relationship in self.npc_relationships.values_mut() {
            if relationship.relationship_value.abs() > 50 {
                let decay_amount = (relationship.relationship_value as f32 * decay_rate * 0.5) as i32;
                relationship.relationship_value -= decay_amount.signum() * decay_amount.abs().min(1);
                relationship.update_relationship_status();
            }
        }

        Ok(())
    }

    /// Load reputation data from save file
    fn load_reputation_data(&mut self, data: &serde_json::Value) -> RobinResult<()> {
        // Implementation for loading saved reputation data
        // This would deserialize faction standings, NPC relationships, etc.
        Ok(())
    }
}

/// Faction identification and categorization
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FactionId {
    pub name: String,
    pub faction_type: FactionType,
}

/// Types of factions in the game world
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FactionType {
    Guild,          // Professional guilds (Builders, Miners, etc.)
    Government,     // City councils, authorities
    Trade,          // Merchant alliances, trading companies
    Professional,   // Architects, Engineers
    Advocacy,       // Environmental, social causes
    Religious,      // Spiritual organizations
    Military,       // Defense forces, security
    Academic,       // Research institutions, schools
}

/// Faction standing tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionStanding {
    pub faction_id: FactionId,
    pub reputation_value: i32,  // -1000 to +1000
    pub reputation_tier: ReputationTier,
    pub first_contact: DateTime<Utc>,
    pub last_interaction: DateTime<Utc>,
    pub total_contributions: u32,
    pub notable_achievements: Vec<String>,
}

impl FactionStanding {
    pub fn new(faction_id: FactionId) -> Self {
        Self {
            faction_id,
            reputation_value: 0,
            reputation_tier: ReputationTier::Neutral,
            first_contact: Utc::now(),
            last_interaction: Utc::now(),
            total_contributions: 0,
            notable_achievements: Vec::new(),
        }
    }

    pub fn update_reputation_tier(&mut self) {
        self.reputation_tier = match self.reputation_value {
            900..=1000 => ReputationTier::Exalted,
            700..=899 => ReputationTier::Revered,
            500..=699 => ReputationTier::Honored,
            200..=499 => ReputationTier::Friendly,
            -199..=199 => ReputationTier::Neutral,
            -499..=-200 => ReputationTier::Unfriendly,
            -699..=-500 => ReputationTier::Hostile,
            -1000..=-700 => ReputationTier::Hated,
            _ => ReputationTier::Neutral,
        };
    }
}

/// Reputation tier levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ReputationTier {
    Hated = 0,
    Hostile = 1,
    Unfriendly = 2,
    Neutral = 3,
    Friendly = 4,
    Honored = 5,
    Revered = 6,
    Exalted = 7,
}

/// NPC identification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NpcId {
    pub name: String,
    pub npc_type: NpcType,
    pub location: Option<Vec3>,
}

/// Types of NPCs for relationship tracking
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NpcType {
    Merchant,
    Craftsman,
    Guard,
    Official,
    Citizen,
    Visitor,
    Expert,
    Leader,
}

/// Individual NPC relationship tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcRelationship {
    pub npc_id: NpcId,
    pub relationship_value: i32,  // -100 to +100
    pub relationship_status: RelationshipStatus,
    pub first_meeting: DateTime<Utc>,
    pub last_interaction: DateTime<Utc>,
    pub interaction_count: u32,
    pub interaction_history: Vec<InteractionRecord>,
    pub personal_notes: String,
}

impl NpcRelationship {
    pub fn new(npc_id: NpcId) -> Self {
        Self {
            npc_id,
            relationship_value: 0,
            relationship_status: RelationshipStatus::Stranger,
            first_meeting: Utc::now(),
            last_interaction: Utc::now(),
            interaction_count: 0,
            interaction_history: Vec::new(),
            personal_notes: String::new(),
        }
    }

    pub fn update_relationship_status(&mut self) {
        self.relationship_status = match self.relationship_value {
            80..=100 => RelationshipStatus::BestFriend,
            50..=79 => RelationshipStatus::Friend,
            20..=49 => RelationshipStatus::Acquaintance,
            -19..=19 => RelationshipStatus::Neutral,
            -49..=-20 => RelationshipStatus::Dislike,
            -79..=-50 => RelationshipStatus::Enemy,
            -100..=-80 => RelationshipStatus::Nemesis,
            _ => RelationshipStatus::Neutral,
        };
    }
}

/// Relationship status levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationshipStatus {
    Nemesis,
    Enemy,
    Dislike,
    Stranger,
    Neutral,
    Acquaintance,
    Friend,
    BestFriend,
}

/// Types of interactions with NPCs
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum InteractionType {
    Trade,
    Quest,
    Conversation,
    Gift,
    Help,
    Conflict,
    Collaboration,
    Teaching,
}

/// Record of NPC interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionRecord {
    pub timestamp: DateTime<Utc>,
    pub interaction_type: InteractionType,
    pub value_change: i32,
    pub context: HashMap<String, String>,
}

/// Community reputation tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityReputation {
    pub community_standing: CommunityStanding,
    pub public_works_score: u32,
    pub environmental_score: u32,
    pub innovation_score: u32,
    pub collaboration_score: u32,
    pub leadership_score: u32,
    pub total_community_contributions: u32,
    pub community_achievements: Vec<String>,
}

impl CommunityReputation {
    pub fn initialize_community_standing(&mut self) {
        self.update_community_standing();
    }

    pub fn update(&mut self, delta_time: f32, social_history: &[SocialEvent]) -> RobinResult<()> {
        // Update community standing based on recent actions
        self.update_community_standing();
        Ok(())
    }

    fn update_community_standing(&mut self) {
        let total_score = self.public_works_score + self.environmental_score +
                         self.innovation_score + self.collaboration_score +
                         self.leadership_score;

        self.community_standing = match total_score {
            500.. => CommunityStanding::Legend,
            300..=499 => CommunityStanding::Hero,
            150..=299 => CommunityStanding::Respected,
            50..=149 => CommunityStanding::Known,
            10..=49 => CommunityStanding::Citizen,
            0..=9 => CommunityStanding::Unknown,
            _ => CommunityStanding::Outcast,
        };
    }
}

impl Default for CommunityReputation {
    fn default() -> Self {
        Self {
            community_standing: CommunityStanding::Unknown,
            public_works_score: 0,
            environmental_score: 0,
            innovation_score: 0,
            collaboration_score: 0,
            leadership_score: 0,
            total_community_contributions: 0,
            community_achievements: Vec::new(),
        }
    }
}

/// Community standing levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommunityStanding {
    Villain,
    Outcast,
    Unknown,
    Citizen,
    Known,
    Respected,
    Hero,
    Legend,
}

/// Social event tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialEvent {
    pub event_id: String,
    pub event_type: SocialEventType,
    pub timestamp: DateTime<Utc>,
    pub faction_id: Option<FactionId>,
    pub npc_id: Option<NpcId>,
    pub value_change: i32,
    pub reason: String,
    pub context: HashMap<String, String>,
}

/// Types of social events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SocialEventType {
    FactionStandingChange,
    NpcRelationshipChange,
    CommunityContribution,
    PublicRecognition,
    SocialConflict,
    CollaborativeProject,
    LeadershipAction,
}

/// Social analytics tracking
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SocialAnalytics {
    pub most_improved_faction: Option<FactionId>,
    pub most_declined_faction: Option<FactionId>,
    pub strongest_npc_relationship: Option<NpcId>,
    pub most_interactions_npc: Option<NpcId>,
    pub social_velocity: f32,  // Rate of reputation change
    pub relationship_diversity: f32,  // How many different factions/NPCs engaged with
    pub conflict_resolution_rate: f32,
    pub collaboration_success_rate: f32,
}

impl SocialAnalytics {
    pub fn update(&mut self,
                 delta_time: f32,
                 faction_standings: &HashMap<FactionId, FactionStanding>,
                 npc_relationships: &HashMap<NpcId, NpcRelationship>) -> RobinResult<()> {
        // Calculate social velocity
        self.calculate_social_velocity(faction_standings, npc_relationships);

        // Update relationship diversity
        self.calculate_relationship_diversity(faction_standings, npc_relationships);

        // Find most improved/declined factions
        self.analyze_faction_trends(faction_standings);

        // Analyze NPC relationship patterns
        self.analyze_npc_patterns(npc_relationships);

        Ok(())
    }

    fn calculate_social_velocity(&mut self,
                               faction_standings: &HashMap<FactionId, FactionStanding>,
                               npc_relationships: &HashMap<NpcId, NpcRelationship>) {
        // Implementation for calculating rate of social change
        let total_interactions = faction_standings.values()
            .map(|f| f.total_contributions)
            .sum::<u32>() as f32;

        let npc_interactions = npc_relationships.values()
            .map(|n| n.interaction_count)
            .sum::<u32>() as f32;

        self.social_velocity = (total_interactions + npc_interactions) / 100.0;
    }

    fn calculate_relationship_diversity(&mut self,
                                      faction_standings: &HashMap<FactionId, FactionStanding>,
                                      npc_relationships: &HashMap<NpcId, NpcRelationship>) {
        let active_factions = faction_standings.values()
            .filter(|f| f.total_contributions > 0)
            .count() as f32;

        let active_npcs = npc_relationships.values()
            .filter(|n| n.interaction_count > 0)
            .count() as f32;

        let total_possible = (faction_standings.len() + npc_relationships.len()) as f32;
        self.relationship_diversity = (active_factions + active_npcs) / total_possible.max(1.0);
    }

    fn analyze_faction_trends(&mut self, faction_standings: &HashMap<FactionId, FactionStanding>) {
        // Find most improved and most declined factions
        // This would track reputation changes over time
    }

    fn analyze_npc_patterns(&mut self, npc_relationships: &HashMap<NpcId, NpcRelationship>) {
        // Find strongest relationships and most interacted NPCs
        self.strongest_npc_relationship = npc_relationships.iter()
            .max_by_key(|(_, rel)| rel.relationship_value)
            .map(|(id, _)| id.clone());

        self.most_interactions_npc = npc_relationships.iter()
            .max_by_key(|(_, rel)| rel.interaction_count)
            .map(|(id, _)| id.clone());
    }
}

/// Configuration for reputation system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationConfig {
    pub reputation_decay_rate: f32,
    pub faction_interaction_cooldown: Duration,
    pub npc_memory_duration: Duration,
    pub community_update_interval: Duration,
    pub max_daily_reputation_gain: i32,
    pub reputation_tier_thresholds: HashMap<ReputationTier, i32>,
}

impl Default for ReputationConfig {
    fn default() -> Self {
        let mut tier_thresholds = HashMap::new();
        tier_thresholds.insert(ReputationTier::Hated, -700);
        tier_thresholds.insert(ReputationTier::Hostile, -500);
        tier_thresholds.insert(ReputationTier::Unfriendly, -200);
        tier_thresholds.insert(ReputationTier::Neutral, 0);
        tier_thresholds.insert(ReputationTier::Friendly, 200);
        tier_thresholds.insert(ReputationTier::Honored, 500);
        tier_thresholds.insert(ReputationTier::Revered, 700);
        tier_thresholds.insert(ReputationTier::Exalted, 900);

        Self {
            reputation_decay_rate: 0.001,  // 0.1% per second for extreme values
            faction_interaction_cooldown: Duration::hours(1),
            npc_memory_duration: Duration::days(30),
            community_update_interval: Duration::hours(6),
            max_daily_reputation_gain: 200,
            reputation_tier_thresholds: tier_thresholds,
        }
    }
}

/// Result types for reputation changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationChange {
    pub faction_id: FactionId,
    pub old_value: i32,
    pub new_value: i32,
    pub change_amount: i32,
    pub old_tier: ReputationTier,
    pub new_tier: ReputationTier,
    pub tier_changed: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipChange {
    pub npc_id: NpcId,
    pub old_value: i32,
    pub new_value: i32,
    pub change_amount: i32,
    pub old_status: RelationshipStatus,
    pub new_status: RelationshipStatus,
    pub status_changed: bool,
    pub interaction_type: InteractionType,
}

/// Reputation-based gameplay modifiers
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReputationModifiers {
    pub trading_discount: f32,
    pub trading_penalty: f32,
    pub quest_reward_bonus: f32,
    pub quest_availability_penalty: f32,
    pub experience_bonus: f32,
    pub experience_penalty: f32,
    pub resource_find_bonus: f32,
    pub social_penalty: f32,
    pub access_bonuses: Vec<String>,
    pub restricted_content: Vec<String>,
}

/// Summary information for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationSummary {
    pub faction_standings: HashMap<FactionId, FactionStanding>,
    pub npc_relationships: HashMap<NpcId, NpcRelationship>,
    pub community_reputation: CommunityReputation,
    pub average_faction_standing: f32,
    pub average_npc_relationship: f32,
    pub total_social_events: usize,
    pub social_analytics: SocialAnalytics,
}

impl Default for ReputationManager {
    fn default() -> Self {
        Self::new()
    }
}