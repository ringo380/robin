// Robin Game Engine - Game AI & Dynamic Adaptation Systems
// Player analytics, dynamic difficulty, and intelligent gameplay features

use crate::engine::error::RobinResult;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use nalgebra::Vector3;

pub mod player_analytics;
pub mod dynamic_adaptation;
pub mod player_state_analysis;
pub mod procedural_generation;
pub mod game_balancing;

/// Main Game AI coordinator for the Robin Engine
#[derive(Debug)]
pub struct GameAIManager {
    pub player_analytics: player_analytics::PlayerAnalytics,
    pub dynamic_adaptation: dynamic_adaptation::DynamicAdaptation,
    pub player_state: player_state_analysis::PlayerStateAnalysis,
    pub procedural_gen: procedural_generation::ProceduralGeneration,
    pub game_balancing: game_balancing::GameBalancing,
    pub player_profiles: HashMap<String, PlayerProfile>,
    pub game_config: GameAIConfiguration,
    pub performance_metrics: GamePerformanceMetrics,
}

/// Comprehensive player profile for game adaptation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerProfile {
    pub player_id: String,
    pub username: String,
    pub play_style: PlayStyle,
    pub skill_levels: HashMap<String, SkillLevel>,
    pub preferences: GamePreferences,
    pub play_history: PlayHistory,
    pub current_state: PlayerState,
    pub building_style: BuildingStyle,
    pub social_preferences: SocialPreferences,
    pub performance_trends: PerformanceTrends,
}

/// Play styles for different types of players
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayStyle {
    pub creativity: f32,         // 0.0-1.0 preference for creative building
    pub exploration: f32,        // 0.0-1.0 preference for exploring
    pub problem_solving: f32,    // 0.0-1.0 preference for puzzles/challenges
    pub social_building: f32,    // 0.0-1.0 preference for collaborative building
    pub competitive: f32,        // 0.0-1.0 preference for competitive gameplay
    pub efficiency: f32,         // 0.0-1.0 preference for optimized builds
    pub experimentation: f32,    // 0.0-1.0 willingness to try new things
    pub primary_style: PrimaryPlayStyle,
}

/// Primary play style classifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrimaryPlayStyle {
    Builder,       // Focuses on construction and creation
    Explorer,      // Enjoys discovering and exploring
    Engineer,      // Likes technical challenges and optimization
    Artist,        // Creative expression through building
    Collaborator,  // Enjoys working with others
    Competitor,    // Seeks challenges and competition
    Experimenter,  // Enjoys trying new mechanics
}

/// Skill levels in different game areas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillLevel {
    pub current_level: f32,      // 0.0-1.0 current skill
    pub progression_rate: f32,   // Rate of skill improvement
    pub consistency: f32,        // How consistent performance is
    pub peak_performance: f32,   // Best recorded performance
    pub practice_time: f32,      // Time spent practicing this skill
    pub last_assessment: chrono::DateTime<chrono::Utc>,
}

/// Game preferences for adaptation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamePreferences {
    pub difficulty_preference: DifficultyPreference,
    pub session_length: SessionLength,
    pub challenge_type: ChallengeType,
    pub feedback_style: FeedbackStyle,
    pub ui_complexity: UIComplexity,
    pub automation_level: AutomationLevel,
    pub tutorial_preference: TutorialPreference,
}

/// Difficulty preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DifficultyPreference {
    Casual,        // Easy, relaxed gameplay
    Moderate,      // Balanced challenge
    Challenging,   // Higher difficulty
    Hardcore,      // Maximum challenge
    Adaptive,      // AI-determined optimal difficulty
}

/// Session length preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionLength {
    Short,    // 10-30 minutes
    Medium,   // 30-60 minutes
    Long,     // 1-2 hours
    Extended, // 2+ hours
    Variable, // Flexible based on engagement
}

/// Challenge type preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChallengeType {
    Creative,    // Building and creation challenges
    Technical,   // Engineering and optimization
    Speed,       // Time-based challenges
    Puzzle,      // Problem-solving challenges
    Social,      // Collaborative challenges
    Mixed,       // Variety of challenge types
}

/// Feedback style preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeedbackStyle {
    Immediate,   // Instant feedback
    Delayed,     // Feedback after completion
    Detailed,    // Comprehensive feedback
    Minimal,     // Simple success/failure
    Visual,      // Visual indicators
    Numeric,     // Scores and metrics
}

/// UI complexity preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UIComplexity {
    Simple,      // Minimal UI elements
    Standard,    // Balanced interface
    Advanced,    // Full feature set visible
    Expert,      // All advanced tools available
}

/// Automation level preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AutomationLevel {
    Manual,      // Player controls everything
    Assisted,    // Some automated helpers
    Automated,   // High level of automation
    Smart,       // AI-driven automation
}

/// Tutorial preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TutorialPreference {
    Comprehensive, // Full tutorials for everything
    Minimal,       // Basic tutorials only
    ContextOnly,   // Just-in-time tutorials
    None,          // Skip all tutorials
}

/// Play history and statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayHistory {
    pub total_play_time: f32,         // Total hours played
    pub sessions_completed: u32,       // Number of play sessions
    pub projects_built: u32,          // Number of building projects
    pub challenges_completed: u32,     // Number of challenges finished
    pub collaboration_time: f32,      // Time spent in multiplayer
    pub favorite_activities: Vec<String>, // Most engaged activities
    pub achievement_progress: HashMap<String, f32>, // Achievement completion
    pub play_patterns: Vec<PlayPattern>,
}

/// Play patterns for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayPattern {
    pub activity_type: String,
    pub frequency: f32,
    pub average_duration: f32,
    pub success_rate: f32,
    pub engagement_level: f32,
}

/// Current player state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub current_mood: PlayerMood,
    pub engagement_level: f32,        // 0.0-1.0
    pub frustration_level: f32,       // 0.0-1.0
    pub confidence_level: f32,        // 0.0-1.0
    pub energy_level: f32,           // 0.0-1.0
    pub focus_level: f32,            // 0.0-1.0
    pub flow_state: bool,            // Whether in flow state
    pub recent_performance: f32,      // Recent performance score
    pub session_progress: SessionProgress,
}

/// Player mood for gameplay adaptation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlayerMood {
    Excited,      // High energy, ready for challenges
    Focused,      // Concentrated, good for complex tasks
    Relaxed,      // Calm, good for creative activities
    Frustrated,   // Struggling, may need assistance
    Bored,        // Low engagement, needs variety
    Confident,    // High confidence, ready for harder content
    Curious,      // Exploring, good for discovery activities
    Tired,        // Low energy, simple activities preferred
}

/// Session progress tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionProgress {
    pub session_start: chrono::DateTime<chrono::Utc>,
    pub current_activity: String,
    pub goals_completed: u32,
    pub goals_total: u32,
    pub milestones_reached: Vec<String>,
    pub time_in_flow: f32,
    pub breaks_taken: u32,
}

/// Building style preferences and patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingStyle {
    pub architectural_preference: ArchitecturalStyle,
    pub complexity_preference: f32,   // 0.0-1.0 simple to complex
    pub detail_level: f32,           // 0.0-1.0 basic to highly detailed
    pub scale_preference: ScalePreference,
    pub material_preferences: Vec<String>,
    pub color_preferences: Vec<String>,
    pub symmetry_preference: f32,     // 0.0-1.0 asymmetric to symmetric
    pub innovation_level: f32,        // 0.0-1.0 traditional to experimental
}

/// Architectural style preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArchitecturalStyle {
    Modern,
    Classical,
    Fantasy,
    SciFi,
    Medieval,
    Industrial,
    Organic,
    Minimalist,
    Mixed,
}

/// Scale preferences for building
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalePreference {
    Small,       // Small detailed builds
    Medium,      // Medium-sized projects
    Large,       // Large structures
    Massive,     // Mega builds
    Variable,    // Different scales for different projects
}

/// Social preferences for multiplayer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialPreferences {
    pub collaboration_style: CollaborationStyle,
    pub communication_preference: CommunicationStyle,
    pub group_size_preference: GroupSizePreference,
    pub leadership_tendency: f32,     // 0.0-1.0 follower to leader
    pub teaching_willingness: f32,    // 0.0-1.0 willingness to help others
    pub learning_from_others: f32,    // 0.0-1.0 openness to learning
    pub competition_comfort: f32,     // 0.0-1.0 comfort with competition
}

/// Collaboration styles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollaborationStyle {
    Independent,  // Prefers working alone
    Cooperative,  // Enjoys working together
    Competitive,  // Likes friendly competition
    Mentoring,    // Enjoys teaching others
    Learning,     // Focuses on learning from others
}

/// Communication preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommunicationStyle {
    Voice,        // Voice chat preferred
    Text,         // Text chat preferred
    Visual,       // Prefers visual communication
    Minimal,      // Limited communication
    Expressive,   // Frequent communication
}

/// Group size preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GroupSizePreference {
    Solo,         // Single player
    Duo,          // Two players
    SmallGroup,   // 3-4 players
    LargeGroup,   // 5+ players
    Flexible,     // Comfortable with any size
}

/// Performance trends over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrends {
    pub skill_progression: HashMap<String, f32>,
    pub engagement_trends: Vec<EngagementPoint>,
    pub difficulty_adaptation: DifficultyTrend,
    pub session_quality: f32,
    pub retention_indicators: RetentionIndicators,
    pub growth_areas: Vec<String>,
    pub strength_areas: Vec<String>,
}

/// Engagement tracking points
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngagementPoint {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub engagement_level: f32,
    pub activity_type: String,
    pub context: String,
    pub duration: f32,
}

/// Difficulty adaptation trends
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifficultyTrend {
    pub optimal_difficulty: f32,      // Current optimal difficulty level
    pub adaptation_speed: f32,        // How quickly to adjust difficulty
    pub challenge_tolerance: f32,     // Tolerance for difficult content
    pub success_rate_target: f32,     // Target success rate
    pub recent_adjustments: Vec<DifficultyAdjustment>,
}

/// Individual difficulty adjustments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifficultyAdjustment {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub previous_difficulty: f32,
    pub new_difficulty: f32,
    pub reason: String,
    pub effectiveness: Option<f32>,
}

/// Retention indicators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionIndicators {
    pub session_frequency: f32,       // How often player returns
    pub session_duration_trend: f32,  // Trend in session length
    pub engagement_stability: f32,    // Consistency of engagement
    pub goal_completion_rate: f32,    // Rate of completing objectives
    pub social_connection_strength: f32, // Strength of social ties
}

/// Game AI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameAIConfiguration {
    pub adaptation_enabled: bool,
    pub difficulty_auto_adjust: bool,
    pub content_recommendation: bool,
    pub social_matching: bool,
    pub performance_tracking: bool,
    pub privacy_level: PrivacyLevel,
    pub data_retention_days: u32,
    pub adaptation_sensitivity: f32,
}

/// Privacy levels for game AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrivacyLevel {
    Essential,    // Only essential gameplay data
    Standard,     // Standard analytics
    Enhanced,     // Full personalization features
    Analytics,    // Detailed analytics for improvement
}

/// Game performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamePerformanceMetrics {
    pub player_satisfaction: f32,     // Overall satisfaction score
    pub retention_rate: f32,          // Player retention percentage
    pub engagement_score: f32,        // Average engagement level
    pub difficulty_balance: f32,      // How well difficulty is balanced
    pub content_variety_usage: f32,   // Usage of different content types
    pub social_interaction_health: f32, // Quality of social interactions
    pub performance_improvement: f32,  // Rate of player improvement
}

/// Events generated by the game AI system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameAIEvent {
    /// Player profile updated
    ProfileUpdated {
        player_id: String,
        updated_aspects: Vec<String>,
    },
    /// Play style detected or changed
    PlayStyleDetected {
        player_id: String,
        new_style: PrimaryPlayStyle,
        confidence: f32,
    },
    /// Player state changed
    PlayerStateChanged {
        player_id: String,
        new_mood: PlayerMood,
        engagement_change: f32,
    },
    /// Difficulty adjustment recommended
    DifficultyAdjustment {
        player_id: String,
        current_difficulty: f32,
        recommended_difficulty: f32,
        reason: String,
    },
    /// Content recommendation generated
    ContentRecommendation {
        player_id: String,
        content_type: String,
        recommendation: String,
        confidence: f32,
    },
    /// Social match suggestion
    SocialMatchSuggestion {
        player_id: String,
        suggested_partners: Vec<String>,
        match_reason: String,
    },
    /// Flow state detected
    FlowStateDetected {
        player_id: String,
        activity: String,
        duration: f32,
    },
    /// Performance milestone reached
    MilestoneReached {
        player_id: String,
        milestone_type: String,
        achievement: String,
    },
}

/// Game AI recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameAIRecommendation {
    pub recommendation_id: String,
    pub player_id: String,
    pub recommendation_type: RecommendationType,
    pub title: String,
    pub description: String,
    pub rationale: String,
    pub confidence: f32,
    pub priority: Priority,
    pub expected_impact: ExpectedImpact,
    pub implementation_steps: Vec<String>,
}

/// Types of game AI recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    ActivitySuggestion,     // Suggest new activities to try
    DifficultyAdjustment,   // Adjust challenge level
    ContentVariation,       // Try different content types
    SocialActivity,         // Suggest multiplayer activities
    SkillDevelopment,       // Focus on developing specific skills
    CreativeChallenge,      // Suggest creative building projects
    PerformanceOptimization, // Optimize building efficiency
    ExplorationGoal,        // Suggest exploration activities
}

/// Priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

/// Expected impact of recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedImpact {
    pub engagement_improvement: f32,
    pub skill_development: f32,
    pub satisfaction_increase: f32,
    pub retention_improvement: f32,
}

/// Player interaction data for AI processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInteraction {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub interaction_type: InteractionType,
    pub duration: f32,
    pub context: String,
    pub performance_data: PerformanceData,
    pub engagement_indicators: EngagementIndicators,
    pub social_data: Option<SocialInteractionData>,
}

/// Types of player interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InteractionType {
    Building,         // Creating structures
    Exploring,        // Exploring the world
    ProblemSolving,   // Working on challenges
    Collaborating,    // Working with others
    Experimenting,    // Trying new mechanics
    Optimizing,       // Improving existing builds
    Socializing,      // Social interaction focus
    Learning,         // Learning new mechanics
}

/// Performance data from interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceData {
    pub completion_rate: f32,        // Rate of completing objectives
    pub efficiency: f32,             // How efficiently goals are achieved
    pub innovation: f32,             // Creativity and innovation shown
    pub technical_skill: f32,        // Technical proficiency displayed
    pub problem_solving: f32,        // Problem-solving ability shown
    pub collaboration_quality: f32,  // Quality of collaboration
    pub time_management: f32,        // How well time is managed
}

/// Engagement indicators from gameplay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngagementIndicators {
    pub attention_focus: f32,        // How focused the player is
    pub voluntary_actions: f32,      // Frequency of voluntary actions
    pub exploration_behavior: f32,   // Amount of exploration
    pub experimentation: f32,        // Willingness to try new things
    pub persistence: f32,            // Persistence through challenges
    pub creative_expression: f32,    // Level of creative expression
}

/// Social interaction data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialInteractionData {
    pub partner_ids: Vec<String>,
    pub interaction_quality: f32,
    pub communication_frequency: f32,
    pub collaboration_effectiveness: f32,
    pub leadership_shown: f32,
    pub support_given: f32,
    pub support_received: f32,
}

impl GameAIManager {
    /// Create a new game AI manager
    pub fn new() -> Self {
        Self {
            player_analytics: player_analytics::PlayerAnalytics::new(),
            dynamic_adaptation: dynamic_adaptation::DynamicAdaptation::new(),
            player_state: player_state_analysis::PlayerStateAnalysis::new(),
            procedural_gen: procedural_generation::ProceduralGeneration::new(),
            game_balancing: game_balancing::GameBalancing::new(),
            player_profiles: HashMap::new(),
            game_config: GameAIConfiguration {
                adaptation_enabled: true,
                difficulty_auto_adjust: true,
                content_recommendation: true,
                social_matching: true,
                performance_tracking: true,
                privacy_level: PrivacyLevel::Standard,
                data_retention_days: 90,
                adaptation_sensitivity: 0.7,
            },
            performance_metrics: GamePerformanceMetrics {
                player_satisfaction: 0.8,
                retention_rate: 0.75,
                engagement_score: 0.8,
                difficulty_balance: 0.8,
                content_variety_usage: 0.7,
                social_interaction_health: 0.8,
                performance_improvement: 0.1,
            },
        }
    }

    /// Initialize the game AI manager
    pub fn initialize(&mut self) -> RobinResult<()> {
        self.player_analytics.initialize()?;
        self.dynamic_adaptation.initialize()?;
        self.player_state.initialize()?;
        self.procedural_gen.initialize()?;
        self.game_balancing.initialize()?;

        println!("🎮 Game AI Manager initialized successfully");
        Ok(())
    }

    /// Update the game AI systems
    pub fn update(&mut self, delta_time: f32) -> RobinResult<Vec<GameAIEvent>> {
        let mut events = Vec::new();

        // Update all AI subsystems
        events.extend(self.player_analytics.update(delta_time)?);
        events.extend(self.dynamic_adaptation.update(delta_time)?);
        events.extend(self.player_state.update(delta_time)?);
        events.extend(self.procedural_gen.update(delta_time)?);
        events.extend(self.game_balancing.update(delta_time)?);

        // Update player profiles based on new data
        self.update_player_profiles(&events)?;

        // Update performance metrics
        self.update_performance_metrics()?;

        Ok(events)
    }

    /// Update player profiles based on AI events
    fn update_player_profiles(&mut self, events: &[GameAIEvent]) -> RobinResult<()> {
        for event in events {
            match event {
                GameAIEvent::PlayStyleDetected { player_id, new_style, .. } => {
                    if let Some(profile) = self.player_profiles.get_mut(player_id) {
                        profile.play_style.primary_style = new_style.clone();
                    }
                }
                GameAIEvent::PlayerStateChanged { player_id, new_mood, engagement_change } => {
                    if let Some(profile) = self.player_profiles.get_mut(player_id) {
                        profile.current_state.current_mood = new_mood.clone();
                        profile.current_state.engagement_level =
                            (profile.current_state.engagement_level + engagement_change).clamp(0.0, 1.0);
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Update performance metrics
    fn update_performance_metrics(&mut self) -> RobinResult<()> {
        // Calculate metrics based on all player data
        let total_players = self.player_profiles.len() as f32;

        if total_players > 0.0 {
            let avg_engagement: f32 = self.player_profiles.values()
                .map(|p| p.current_state.engagement_level)
                .sum::<f32>() / total_players;

            self.performance_metrics.engagement_score = avg_engagement;

            // Update other metrics based on player data
            self.performance_metrics.player_satisfaction += 0.001; // Gradual improvement
            self.performance_metrics.retention_rate = (self.performance_metrics.retention_rate + 0.001).min(1.0);
        }

        Ok(())
    }

    /// Add a new player profile
    pub fn add_player_profile(&mut self, profile: PlayerProfile) {
        self.player_profiles.insert(profile.player_id.clone(), profile);
    }

    /// Get a player profile
    pub fn get_player_profile(&self, player_id: &str) -> Option<&PlayerProfile> {
        self.player_profiles.get(player_id)
    }

    /// Generate AI recommendations for a player
    pub fn generate_recommendations(&self, player_id: &str) -> RobinResult<Vec<GameAIRecommendation>> {
        if let Some(profile) = self.player_profiles.get(player_id) {
            let mut recommendations = Vec::new();

            // Generate recommendations from different AI systems
            recommendations.extend(self.dynamic_adaptation.generate_recommendations(profile)?);
            recommendations.extend(self.player_analytics.generate_recommendations(profile)?);
            recommendations.extend(self.game_balancing.generate_recommendations(profile)?);

            Ok(recommendations)
        } else {
            Ok(Vec::new())
        }
    }

    /// Process player interaction data
    pub fn process_interaction(&mut self, player_id: &str, interaction: PlayerInteraction) -> RobinResult<Vec<GameAIEvent>> {
        let mut events = Vec::new();

        // Process interaction through various AI systems
        events.extend(self.player_analytics.process_interaction(player_id, &interaction)?);
        events.extend(self.player_state.process_interaction(player_id, &interaction)?);
        events.extend(self.dynamic_adaptation.process_interaction(player_id, &interaction)?);

        Ok(events)
    }

    /// Get optimal difficulty for a player
    pub fn get_optimal_difficulty(&self, player_id: &str) -> f32 {
        if let Some(profile) = self.player_profiles.get(player_id) {
            profile.performance_trends.difficulty_adaptation.optimal_difficulty
        } else {
            0.5 // Default medium difficulty
        }
    }

    /// Get content recommendations for a player
    pub fn get_content_recommendations(&self, player_id: &str) -> Vec<String> {
        if let Some(profile) = self.player_profiles.get(player_id) {
            match profile.play_style.primary_style {
                PrimaryPlayStyle::Builder => vec!["Complex Building Challenge".to_string(), "Architecture Tutorial".to_string()],
                PrimaryPlayStyle::Explorer => vec!["Hidden Area Discovery".to_string(), "Exploration Challenge".to_string()],
                PrimaryPlayStyle::Engineer => vec!["Technical Optimization".to_string(), "System Design Challenge".to_string()],
                PrimaryPlayStyle::Artist => vec!["Creative Building Contest".to_string(), "Artistic Expression Challenge".to_string()],
                PrimaryPlayStyle::Collaborator => vec!["Team Building Project".to_string(), "Collaborative Challenge".to_string()],
                PrimaryPlayStyle::Competitor => vec!["Speed Building Challenge".to_string(), "Competitive Tournament".to_string()],
                PrimaryPlayStyle::Experimenter => vec!["New Mechanics Tutorial".to_string(), "Experimental Features".to_string()],
            }
        } else {
            vec!["Basic Tutorial".to_string(), "Free Build Mode".to_string()]
        }
    }

    /// Check if player is in flow state
    pub fn is_player_in_flow(&self, player_id: &str) -> bool {
        if let Some(profile) = self.player_profiles.get(player_id) {
            profile.current_state.flow_state
        } else {
            false
        }
    }

    /// Get social match suggestions for a player
    pub fn get_social_matches(&self, player_id: &str) -> Vec<String> {
        if let Some(profile) = self.player_profiles.get(player_id) {
            // Find players with compatible play styles and preferences
            self.player_profiles
                .values()
                .filter(|other| {
                    other.player_id != player_id &&
                    self.calculate_compatibility(profile, other) > 0.7
                })
                .map(|p| p.player_id.clone())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Calculate compatibility between two players
    fn calculate_compatibility(&self, player1: &PlayerProfile, player2: &PlayerProfile) -> f32 {
        let style_compatibility = match (&player1.play_style.primary_style, &player2.play_style.primary_style) {
            (PrimaryPlayStyle::Builder, PrimaryPlayStyle::Engineer) => 0.9,
            (PrimaryPlayStyle::Artist, PrimaryPlayStyle::Builder) => 0.8,
            (PrimaryPlayStyle::Collaborator, _) => 0.8,
            (_, PrimaryPlayStyle::Collaborator) => 0.8,
            (a, b) if a == b => 0.7,
            _ => 0.5,
        };

        let social_compatibility = if player1.social_preferences.group_size_preference == player2.social_preferences.group_size_preference {
            0.8
        } else {
            0.5
        };

        (style_compatibility + social_compatibility) / 2.0
    }
}

impl Default for PlayerProfile {
    fn default() -> Self {
        Self {
            player_id: String::new(),
            username: String::new(),
            play_style: PlayStyle {
                creativity: 0.5,
                exploration: 0.5,
                problem_solving: 0.5,
                social_building: 0.5,
                competitive: 0.5,
                efficiency: 0.5,
                experimentation: 0.5,
                primary_style: PrimaryPlayStyle::Builder,
            },
            skill_levels: HashMap::new(),
            preferences: GamePreferences {
                difficulty_preference: DifficultyPreference::Adaptive,
                session_length: SessionLength::Variable,
                challenge_type: ChallengeType::Mixed,
                feedback_style: FeedbackStyle::Visual,
                ui_complexity: UIComplexity::Standard,
                automation_level: AutomationLevel::Assisted,
                tutorial_preference: TutorialPreference::ContextOnly,
            },
            play_history: PlayHistory {
                total_play_time: 0.0,
                sessions_completed: 0,
                projects_built: 0,
                challenges_completed: 0,
                collaboration_time: 0.0,
                favorite_activities: Vec::new(),
                achievement_progress: HashMap::new(),
                play_patterns: Vec::new(),
            },
            current_state: PlayerState {
                current_mood: PlayerMood::Curious,
                engagement_level: 0.7,
                frustration_level: 0.0,
                confidence_level: 0.6,
                energy_level: 0.8,
                focus_level: 0.7,
                flow_state: false,
                recent_performance: 0.6,
                session_progress: SessionProgress {
                    session_start: chrono::Utc::now(),
                    current_activity: String::new(),
                    goals_completed: 0,
                    goals_total: 0,
                    milestones_reached: Vec::new(),
                    time_in_flow: 0.0,
                    breaks_taken: 0,
                },
            },
            building_style: BuildingStyle {
                architectural_preference: ArchitecturalStyle::Mixed,
                complexity_preference: 0.5,
                detail_level: 0.5,
                scale_preference: ScalePreference::Variable,
                material_preferences: Vec::new(),
                color_preferences: Vec::new(),
                symmetry_preference: 0.5,
                innovation_level: 0.6,
            },
            social_preferences: SocialPreferences {
                collaboration_style: CollaborationStyle::Cooperative,
                communication_preference: CommunicationStyle::Visual,
                group_size_preference: GroupSizePreference::Flexible,
                leadership_tendency: 0.5,
                teaching_willingness: 0.6,
                learning_from_others: 0.7,
                competition_comfort: 0.5,
            },
            performance_trends: PerformanceTrends {
                skill_progression: HashMap::new(),
                engagement_trends: Vec::new(),
                difficulty_adaptation: DifficultyTrend {
                    optimal_difficulty: 0.5,
                    adaptation_speed: 0.1,
                    challenge_tolerance: 0.7,
                    success_rate_target: 0.75,
                    recent_adjustments: Vec::new(),
                },
                session_quality: 0.7,
                retention_indicators: RetentionIndicators {
                    session_frequency: 0.7,
                    session_duration_trend: 0.0,
                    engagement_stability: 0.8,
                    goal_completion_rate: 0.6,
                    social_connection_strength: 0.5,
                },
                growth_areas: Vec::new(),
                strength_areas: Vec::new(),
            },
        }
    }
}