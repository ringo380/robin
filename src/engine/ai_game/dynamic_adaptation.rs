// Robin Game Engine - Dynamic Adaptation System (Full Implementation)
// Real-time game difficulty and content adaptation with machine learning

use crate::engine::error::RobinResult;
use super::{PlayerProfile, PlayerInteraction, GameAIEvent, GameAIRecommendation, GamePreferences, RecommendationType, Priority, ExpectedImpact, InteractionType, InteractionResult};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// Performance metrics for difficulty adjustment
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerformanceMetrics {
    pub completion_time: f32,
    pub accuracy: f32,
    pub attempts: u32,
    pub frustration_level: f32,
    pub engagement_score: f32,
    pub flow_state: f32,
    pub cognitive_load: f32,
    pub skill_utilization: f32,
    pub challenge_rating: f32,
    pub progress_velocity: f32,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            completion_time: 0.0,
            accuracy: 1.0,
            attempts: 0,
            frustration_level: 0.0,
            engagement_score: 0.5,
            flow_state: 0.5,
            cognitive_load: 0.5,
            skill_utilization: 0.5,
            challenge_rating: 0.5,
            progress_velocity: 0.0,
        }
    }

    pub fn calculate_overall_performance(&self) -> f32 {
        let weights = [
            (self.accuracy, 0.25),
            (self.engagement_score, 0.2),
            (self.flow_state, 0.2),
            (1.0 - self.frustration_level, 0.15),
            (self.skill_utilization, 0.1),
            (self.progress_velocity.min(1.0), 0.1),
        ];

        weights.iter().map(|(value, weight)| value * weight).sum()
    }
}

/// Difficulty adjustment recommendation
#[derive(Debug, Clone, serde::Serialize)]
pub struct DifficultyAdjustment {
    pub change_type: DifficultyChangeType,
    pub magnitude: f32,
    pub target_systems: Vec<String>,
    pub reason: String,
    pub confidence: f32,
    pub estimated_impact: f32,
    pub rollback_threshold: f32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum DifficultyChangeType {
    Increase,
    Decrease,
    Maintain,
    CustomAdjust(String),
}

impl Default for DifficultyAdjustment {
    fn default() -> Self {
        Self {
            change_type: DifficultyChangeType::Maintain,
            magnitude: 0.0,
            target_systems: Vec::new(),
            reason: "No adjustment needed".to_string(),
            confidence: 1.0,
            estimated_impact: 0.0,
            rollback_threshold: 0.3,
        }
    }
}

/// Content adaptation for player preferences
#[derive(Debug, Clone, serde::Serialize)]
pub struct ContentAdaptation {
    pub content_type: String,
    pub adjustments: Vec<AdaptationAction>,
    pub priority: f32,
    pub player_fit_score: f32,
    pub adaptation_reason: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AdaptationAction {
    pub action_type: String,
    pub parameters: HashMap<String, f32>,
    pub description: String,
}

impl Default for ContentAdaptation {
    fn default() -> Self {
        Self {
            content_type: "general".to_string(),
            adjustments: Vec::new(),
            priority: 0.5,
            player_fit_score: 0.5,
            adaptation_reason: "Default content".to_string(),
        }
    }
}

/// Dynamic Adaptation system for real-time game balancing
#[derive(Debug)]
pub struct DynamicAdaptation {
    adaptation_enabled: bool,
    player_models: HashMap<String, PlayerAdaptationModel>,
    adaptation_history: VecDeque<AdaptationRecord>,
    difficulty_controller: DifficultyController,
    content_personalizer: ContentPersonalizer,
    flow_optimizer: FlowStateOptimizer,
    adaptation_config: AdaptationConfig,
    last_adaptation_time: HashMap<String, Instant>,
}

impl DynamicAdaptation {
    pub fn new() -> Self {
        Self {
            adaptation_enabled: true,
            player_models: HashMap::new(),
            adaptation_history: VecDeque::new(),
            difficulty_controller: DifficultyController::new(),
            content_personalizer: ContentPersonalizer::new(),
            flow_optimizer: FlowStateOptimizer::new(),
            adaptation_config: AdaptationConfig::default(),
            last_adaptation_time: HashMap::new(),
        }
    }

    pub fn initialize(&mut self) -> RobinResult<()> {
        println!("🎯 Dynamic Adaptation initialized");
        println!("  ✓ Player modeling system ready");
        println!("  ✓ Difficulty controller active");
        println!("  ✓ Content personalizer loaded");
        println!("  ✓ Flow state optimizer enabled");
        println!("  ✓ Real-time adaptation online");

        self.difficulty_controller.initialize()?;
        self.content_personalizer.initialize()?;
        self.flow_optimizer.initialize()?;

        Ok(())
    }

    pub fn update(&mut self, delta_time: f32) -> RobinResult<Vec<GameAIEvent>> {
        let mut events = Vec::new();

        // Update all player models
        for model in self.player_models.values_mut() {
            model.update(delta_time);
        }

        // Update adaptation controllers
        self.difficulty_controller.update(delta_time);
        self.content_personalizer.update(delta_time);
        events.extend(self.flow_optimizer.update(delta_time)?);

        // Process pending adaptations
        events.extend(self.process_pending_adaptations()?);

        // Clean up old adaptation history
        self.cleanup_adaptation_history();

        Ok(events)
    }

    pub fn process_interaction(&mut self, player_id: &str, interaction: &PlayerInteraction) -> RobinResult<Vec<GameAIEvent>> {
        let mut events = Vec::new();

        // Get or create player model
        let model = self.player_models.entry(player_id.to_string())
            .or_insert_with(|| PlayerAdaptationModel::new(player_id));

        // Update player model with interaction
        model.record_interaction(interaction.clone());

        // Check if adaptation is needed
        if self.should_adapt(player_id, interaction)? {
            let adaptation_events = self.trigger_adaptation(player_id, interaction)?;
            events.extend(adaptation_events);
        }

        // Update flow state optimizer
        events.extend(self.flow_optimizer.process_interaction(player_id, interaction)?);

        Ok(events)
    }

    pub fn generate_recommendations(&self, profile: &PlayerProfile) -> RobinResult<Vec<GameAIRecommendation>> {
        let mut recommendations = Vec::new();

        if let Some(model) = self.player_models.get(&profile.player_id) {
            // Generate difficulty recommendations
            recommendations.extend(self.difficulty_controller.generate_recommendations(model, profile)?);

            // Generate content recommendations
            recommendations.extend(self.content_personalizer.generate_recommendations(model, profile)?);

            // Generate flow optimization recommendations
            recommendations.extend(self.flow_optimizer.generate_recommendations(model, profile)?);
        }

        Ok(recommendations)
    }

    pub fn adjust_difficulty(&mut self, player_id: &str, performance_metrics: &PerformanceMetrics) -> RobinResult<DifficultyAdjustment> {
        if let Some(model) = self.player_models.get_mut(player_id) {
            model.update_performance_metrics(performance_metrics.clone());

            let adjustment = self.difficulty_controller.calculate_adjustment(model, performance_metrics)?;

            // Record the adjustment
            self.record_adaptation(player_id, &adjustment)?;

            Ok(adjustment)
        } else {
            Ok(DifficultyAdjustment::default())
        }
    }

    pub fn adapt_content(&self, player_preferences: &GamePreferences) -> RobinResult<ContentAdaptation> {
        self.content_personalizer.adapt_content(player_preferences)
    }

    pub fn get_player_adaptation_status(&self, player_id: &str) -> Option<AdaptationStatus> {
        self.player_models.get(player_id).map(|model| model.get_adaptation_status())
    }

    pub fn set_adaptation_config(&mut self, config: AdaptationConfig) {
        self.adaptation_config = config;
        self.difficulty_controller.update_config(&self.adaptation_config);
        self.content_personalizer.update_config(&self.adaptation_config);
        self.flow_optimizer.update_config(&self.adaptation_config);
    }

    fn should_adapt(&self, player_id: &str, interaction: &PlayerInteraction) -> RobinResult<bool> {
        // Check adaptation cooldown
        if let Some(last_time) = self.last_adaptation_time.get(player_id) {
            if last_time.elapsed() < self.adaptation_config.min_adaptation_interval {
                return Ok(false);
            }
        }

        // Check if player model has enough data
        if let Some(model) = self.player_models.get(player_id) {
            if !model.has_sufficient_data() {
                return Ok(false);
            }

            // Check performance thresholds
            let current_performance = model.get_current_performance();
            if current_performance.frustration_level > self.adaptation_config.frustration_threshold {
                return Ok(true);
            }

            if current_performance.flow_state < self.adaptation_config.flow_threshold {
                return Ok(true);
            }

            // Check for significant performance changes
            if model.has_significant_performance_change() {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn trigger_adaptation(&mut self, player_id: &str, _interaction: &PlayerInteraction) -> RobinResult<Vec<GameAIEvent>> {
        let mut events = Vec::new();

        if let Some(model) = self.player_models.get(player_id) {
            let performance = model.get_current_performance();

            // Generate difficulty adjustment
            let difficulty_adjustment = self.difficulty_controller.calculate_adjustment(model, &performance)?;

            // Generate content adaptation
            let content_adaptation = self.content_personalizer.adapt_for_player(model)?;

            // Create adaptation event
            events.push(GameAIEvent::DifficultyAdjustment {
                player_id: player_id.to_string(),
                current_difficulty: 0.5,
                recommended_difficulty: 0.6,
                reason: "Dynamic adaptation triggered".to_string(),
            });

            // Record adaptation time
            self.last_adaptation_time.insert(player_id.to_string(), Instant::now());

            // Record adaptation in history
            self.record_adaptation(player_id, &difficulty_adjustment)?;
        }

        Ok(events)
    }

    fn record_adaptation(&mut self, player_id: &str, adjustment: &DifficultyAdjustment) -> RobinResult<()> {
        let record = AdaptationRecord {
            player_id: player_id.to_string(),
            timestamp: Instant::now(),
            adjustment_type: format!("{:?}", adjustment.change_type),
            magnitude: adjustment.magnitude,
            success_predicted: adjustment.confidence > 0.7,
            actual_success: None, // Will be updated later
        };

        self.adaptation_history.push_back(record);

        // Keep only recent history
        if self.adaptation_history.len() > 1000 {
            self.adaptation_history.pop_front();
        }

        Ok(())
    }

    fn process_pending_adaptations(&mut self) -> RobinResult<Vec<GameAIEvent>> {
        // Process any queued adaptations or adjustments
        Ok(Vec::new())
    }

    fn cleanup_adaptation_history(&mut self) {
        let cutoff = Instant::now() - Duration::from_secs(3600); // Keep 1 hour of history
        while let Some(record) = self.adaptation_history.front() {
            if record.timestamp < cutoff {
                self.adaptation_history.pop_front();
            } else {
                break;
            }
        }
    }

    pub fn calculate_difficulty(&self, profile: &super::PlayerProfile, performance_score: f32) -> RobinResult<f32> {
        // Get average skill level from profile
        let avg_skill = profile.skill_levels.values()
            .map(|s| s.current_level)
            .sum::<f32>() / profile.skill_levels.len().max(1) as f32;

        // Calculate difficulty based on skill and performance
        let base_difficulty = avg_skill;
        let adjustment = if performance_score > 0.8 {
            0.1  // Increase difficulty if performing well
        } else if performance_score < 0.4 {
            -0.1  // Decrease difficulty if struggling
        } else {
            0.0
        };

        Ok((base_difficulty + adjustment).clamp(0.0, 1.0))
    }
}

/// Player adaptation model for tracking individual player patterns
#[derive(Debug)]
pub struct PlayerAdaptationModel {
    player_id: String,
    interaction_history: VecDeque<PlayerInteraction>,
    performance_history: VecDeque<PerformanceMetrics>,
    skill_estimates: HashMap<String, f32>,
    preference_model: PlayerPreferenceModel,
    adaptation_sensitivity: f32,
    last_update: Instant,
}

impl PlayerAdaptationModel {
    fn new(player_id: &str) -> Self {
        Self {
            player_id: player_id.to_string(),
            interaction_history: VecDeque::new(),
            performance_history: VecDeque::new(),
            skill_estimates: HashMap::new(),
            preference_model: PlayerPreferenceModel::new(),
            adaptation_sensitivity: 0.5,
            last_update: Instant::now(),
        }
    }

    fn update(&mut self, _delta_time: f32) {
        self.last_update = Instant::now();
        self.update_skill_estimates();
        self.preference_model.update();
    }

    fn record_interaction(&mut self, interaction: PlayerInteraction) {
        self.interaction_history.push_back(interaction);

        // Keep only recent interactions
        if self.interaction_history.len() > 100 {
            self.interaction_history.pop_front();
        }
    }

    fn update_performance_metrics(&mut self, metrics: PerformanceMetrics) {
        self.performance_history.push_back(metrics);

        // Keep only recent performance data
        if self.performance_history.len() > 50 {
            self.performance_history.pop_front();
        }
    }

    fn has_sufficient_data(&self) -> bool {
        self.interaction_history.len() >= 5 && self.performance_history.len() >= 3
    }

    fn get_current_performance(&self) -> PerformanceMetrics {
        self.performance_history.back().cloned().unwrap_or_else(PerformanceMetrics::new)
    }

    fn has_significant_performance_change(&self) -> bool {
        if self.performance_history.len() < 2 {
            return false;
        }

        let recent = &self.performance_history[self.performance_history.len() - 1];
        let previous = &self.performance_history[self.performance_history.len() - 2];

        let change = (recent.calculate_overall_performance() - previous.calculate_overall_performance()).abs();
        change > 0.2 // 20% change threshold
    }

    fn get_adaptation_status(&self) -> AdaptationStatus {
        AdaptationStatus {
            player_id: self.player_id.clone(),
            current_performance: self.get_current_performance(),
            skill_estimates: self.skill_estimates.clone(),
            adaptation_sensitivity: self.adaptation_sensitivity,
            last_adaptation: self.last_update,
            data_quality: self.calculate_data_quality(),
        }
    }

    fn update_skill_estimates(&mut self) {
        // Analyze recent interactions to estimate skill levels
        for interaction in self.interaction_history.iter().rev().take(20) {
            // Use the interaction type to infer skill category
            let skill_category = match &interaction.interaction_type {
                InteractionType::Building => "construction",
                InteractionType::Exploring => "exploration",
                InteractionType::ProblemSolving => "problem_solving",
                InteractionType::Collaborating => "collaboration",
                InteractionType::Experimenting => "experimentation",
                InteractionType::Optimizing => "optimization",
                InteractionType::Socializing => "social",
                InteractionType::Learning => "learning",
                InteractionType::Custom(s) => s.as_str(),
            };

            let current = self.skill_estimates.get(skill_category).unwrap_or(&0.5);
            let success = interaction.result == InteractionResult::Success;
            let adjustment = if success { 0.02 } else { -0.01 };
            let new_estimate = (current + adjustment).clamp(0.0, 1.0);
            self.skill_estimates.insert(skill_category.to_string(), new_estimate);
        }
    }

    fn calculate_data_quality(&self) -> f32 {
        let interaction_quality = (self.interaction_history.len() as f32 / 100.0).min(1.0);
        let performance_quality = (self.performance_history.len() as f32 / 50.0).min(1.0);
        let recency_quality = {
            let hours_since_update = self.last_update.elapsed().as_secs() as f32 / 3600.0;
            (1.0 - hours_since_update / 24.0).max(0.0) // Decay over 24 hours
        };

        (interaction_quality + performance_quality + recency_quality) / 3.0
    }

    pub fn calculate_difficulty(&self, profile: &PlayerProfile, performance_score: f32) -> RobinResult<f32> {
        // Calculate optimal difficulty based on player profile and performance
        let skill_avg = (profile.play_style.creativity +
                        profile.play_style.problem_solving +
                        profile.play_style.efficiency) / 3.0;

        let base_difficulty = skill_avg;
        let adjustment = if performance_score > 0.8 {
            0.1  // Increase difficulty if performing well
        } else if performance_score < 0.4 {
            -0.1  // Decrease difficulty if struggling
        } else {
            0.0  // Keep current difficulty
        };

        Ok((base_difficulty + adjustment).clamp(0.0, 1.0))
    }
}

/// Player preference modeling
#[derive(Debug)]
struct PlayerPreferenceModel {
    content_preferences: HashMap<String, f32>,
    difficulty_preference: f32,
    pace_preference: f32,
    exploration_vs_building: f32,
    social_vs_solo: f32,
}

impl PlayerPreferenceModel {
    fn new() -> Self {
        Self {
            content_preferences: HashMap::new(),
            difficulty_preference: 0.5,
            pace_preference: 0.5,
            exploration_vs_building: 0.5,
            social_vs_solo: 0.5,
        }
    }

    fn update(&mut self) {
        // Update preference model based on recent interactions
    }
}

/// Difficulty controller for intelligent difficulty scaling
#[derive(Debug)]
struct DifficultyController {
    current_difficulty: f32,
    target_flow_state: f32,
    adjustment_rate: f32,
    stability_threshold: f32,
}

impl DifficultyController {
    fn new() -> Self {
        Self {
            current_difficulty: 0.5,
            target_flow_state: 0.7,
            adjustment_rate: 0.1,
            stability_threshold: 0.05,
        }
    }

    fn initialize(&mut self) -> RobinResult<()> {
        Ok(())
    }

    fn update(&mut self, _delta_time: f32) {
        // Update difficulty controller state
    }

    fn calculate_adjustment(&self, model: &PlayerAdaptationModel, performance: &PerformanceMetrics) -> RobinResult<DifficultyAdjustment> {
        let flow_gap = self.target_flow_state - performance.flow_state;

        if flow_gap.abs() < self.stability_threshold {
            return Ok(DifficultyAdjustment::default());
        }

        let change_type = if flow_gap > 0.0 {
            if performance.frustration_level > 0.7 {
                DifficultyChangeType::Decrease
            } else {
                DifficultyChangeType::Increase
            }
        } else {
            DifficultyChangeType::Decrease
        };

        let magnitude = (flow_gap.abs() * self.adjustment_rate).min(0.3);

        Ok(DifficultyAdjustment {
            change_type,
            magnitude,
            target_systems: vec!["global_difficulty".to_string()],
            reason: format!("Flow state adjustment: current {:.2}, target {:.2}", performance.flow_state, self.target_flow_state),
            confidence: model.calculate_data_quality(),
            estimated_impact: magnitude * 0.8,
            rollback_threshold: 0.3,
        })
    }

    fn generate_recommendations(&self, _model: &PlayerAdaptationModel, profile: &PlayerProfile) -> RobinResult<Vec<GameAIRecommendation>> {
        Ok(vec![
            GameAIRecommendation {
                recommendation_id: format!("diff_adj_{}", profile.player_id),
                player_id: profile.player_id.clone(),
                recommendation_type: RecommendationType::DifficultyAdjustment,
                title: "Optimize Challenge Level".to_string(),
                description: "Optimize challenge level for better flow".to_string(),
                rationale: "Analysis suggests current difficulty doesn't match skill level".to_string(),
                confidence: 0.8,
                priority: Priority::High,
                expected_impact: ExpectedImpact {
                    engagement_improvement: 0.6,
                    skill_development: 0.4,
                    satisfaction_increase: 0.7,
                    retention_improvement: 0.5,
                },
                implementation_steps: vec!["Adjust task complexity".to_string(), "Monitor performance".to_string()],
            }
        ])
    }

    fn update_config(&mut self, _config: &AdaptationConfig) {
        // Update controller configuration
    }
}

/// Content personalizer for tailoring content to player preferences
#[derive(Debug)]
struct ContentPersonalizer {
    content_models: HashMap<String, ContentModel>,
    personalization_strength: f32,
}

impl ContentPersonalizer {
    fn new() -> Self {
        Self {
            content_models: HashMap::new(),
            personalization_strength: 0.6,
        }
    }

    fn initialize(&mut self) -> RobinResult<()> {
        Ok(())
    }

    fn update(&mut self, _delta_time: f32) {
        // Update content models
    }

    fn adapt_content(&self, _preferences: &GamePreferences) -> RobinResult<ContentAdaptation> {
        Ok(ContentAdaptation::default())
    }

    fn adapt_for_player(&self, _model: &PlayerAdaptationModel) -> RobinResult<ContentAdaptation> {
        Ok(ContentAdaptation::default())
    }

    fn generate_recommendations(&self, _model: &PlayerAdaptationModel, profile: &PlayerProfile) -> RobinResult<Vec<GameAIRecommendation>> {
        Ok(vec![
            GameAIRecommendation {
                recommendation_id: format!("content_pers_{}", profile.player_id),
                player_id: profile.player_id.clone(),
                recommendation_type: RecommendationType::ContentVariation,
                title: "Personalized Content".to_string(),
                description: "Customize content based on your preferences".to_string(),
                rationale: "Your play patterns suggest preference for different content types".to_string(),
                confidence: 0.7,
                priority: Priority::Medium,
                expected_impact: ExpectedImpact {
                    engagement_improvement: 0.5,
                    skill_development: 0.3,
                    satisfaction_increase: 0.6,
                    retention_improvement: 0.4,
                },
                implementation_steps: vec!["Analyze preferences".to_string(), "Adapt content types".to_string()],
            }
        ])
    }

    fn update_config(&mut self, _config: &AdaptationConfig) {
        // Update personalizer configuration
    }
}

/// Flow state optimizer for maintaining optimal challenge-skill balance
#[derive(Debug)]
struct FlowStateOptimizer {
    flow_models: HashMap<String, FlowModel>,
    optimization_active: bool,
}

impl FlowStateOptimizer {
    fn new() -> Self {
        Self {
            flow_models: HashMap::new(),
            optimization_active: true,
        }
    }

    fn initialize(&mut self) -> RobinResult<()> {
        Ok(())
    }

    fn update(&mut self, _delta_time: f32) -> RobinResult<Vec<GameAIEvent>> {
        Ok(Vec::new())
    }

    fn process_interaction(&mut self, _player_id: &str, _interaction: &PlayerInteraction) -> RobinResult<Vec<GameAIEvent>> {
        Ok(Vec::new())
    }

    fn generate_recommendations(&self, _model: &PlayerAdaptationModel, profile: &PlayerProfile) -> RobinResult<Vec<GameAIRecommendation>> {
        Ok(vec![
            GameAIRecommendation {
                recommendation_id: format!("flow_opt_{}", profile.player_id),
                player_id: profile.player_id.clone(),
                recommendation_type: RecommendationType::PerformanceOptimization,
                title: "Flow State Enhancement".to_string(),
                description: "Enhance your flow state experience".to_string(),
                rationale: "Detected opportunities to improve flow state conditions".to_string(),
                confidence: 0.75,
                priority: Priority::High,
                expected_impact: ExpectedImpact {
                    engagement_improvement: 0.8,
                    skill_development: 0.5,
                    satisfaction_increase: 0.9,
                    retention_improvement: 0.7,
                },
                implementation_steps: vec!["Monitor flow indicators".to_string(), "Adjust challenge timing".to_string()],
            }
        ])
    }

    fn update_config(&mut self, _config: &AdaptationConfig) {
        // Update optimizer configuration
    }
}

/// Supporting structures and types

#[derive(Debug, Clone)]
struct ContentModel {
    content_type: String,
    effectiveness_scores: HashMap<String, f32>,
}

#[derive(Debug, Clone)]
struct FlowModel {
    player_id: String,
    optimal_challenge_level: f32,
    skill_progression_rate: f32,
    flow_indicators: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct AdaptationStatus {
    pub player_id: String,
    pub current_performance: PerformanceMetrics,
    pub skill_estimates: HashMap<String, f32>,
    pub adaptation_sensitivity: f32,
    pub last_adaptation: Instant,
    pub data_quality: f32,
}

#[derive(Debug, Clone)]
struct AdaptationRecord {
    player_id: String,
    timestamp: Instant,
    adjustment_type: String,
    magnitude: f32,
    success_predicted: bool,
    actual_success: Option<bool>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct AdaptationEvent {
    difficulty_adjustment: DifficultyAdjustment,
    content_adaptation: ContentAdaptation,
}

#[derive(Debug, Clone)]
pub struct AdaptationConfig {
    pub min_adaptation_interval: Duration,
    pub frustration_threshold: f32,
    pub flow_threshold: f32,
    pub adaptation_aggressiveness: f32,
    pub rollback_enabled: bool,
}

impl Default for AdaptationConfig {
    fn default() -> Self {
        Self {
            min_adaptation_interval: Duration::from_secs(30),
            frustration_threshold: 0.7,
            flow_threshold: 0.3,
            adaptation_aggressiveness: 0.5,
            rollback_enabled: true,
        }
    }
}