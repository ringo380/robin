// Robin Game Engine - Player Analytics Extensions
// Supporting structures and implementations for comprehensive analytics

use super::{PlayerProfile, PlayerInteraction};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// Player session data tracking
#[derive(Debug)]
pub struct PlayerSession {
    pub player_id: String,
    pub start_time: Instant,
    pub last_activity: Instant,
    pub interactions: VecDeque<PlayerInteraction>,
    pub metrics: PlayerMetrics,
    pub initial_profile: Option<PlayerProfile>,
    pub session_active: bool,
}

impl PlayerSession {
    pub fn new(player_id: &str) -> Self {
        let now = Instant::now();
        Self {
            player_id: player_id.to_string(),
            start_time: now,
            last_activity: now,
            interactions: VecDeque::new(),
            metrics: PlayerMetrics::new(),
            initial_profile: None,
            session_active: true,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        if self.session_active {
            self.metrics.update(delta_time);

            // Check for inactivity
            if self.last_activity.elapsed() > Duration::from_secs(300) {
                self.session_active = false;
            }
        }
    }

    pub fn record_interaction(&mut self, interaction: PlayerInteraction) {
        self.last_activity = Instant::now();
        self.interactions.push_back(interaction.clone());
        self.metrics.record_interaction(&interaction);

        // Keep only recent interactions (last 1000)
        if self.interactions.len() > 1000 {
            self.interactions.pop_front();
        }
    }

    pub fn set_initial_profile(&mut self, profile: PlayerProfile) {
        self.initial_profile = Some(profile);
    }

    pub fn end_session(&mut self) {
        self.session_active = false;
        self.metrics.end_session();
    }

    pub fn get_metrics(&self) -> PlayerMetrics {
        self.metrics.clone()
    }

    pub fn interaction_count(&self) -> usize {
        self.interactions.len()
    }

    pub fn get_recent_interactions(&self, count: usize) -> Vec<PlayerInteraction> {
        self.interactions.iter().rev().take(count).cloned().collect()
    }

    pub fn detect_patterns(&self) -> Option<GameplayPattern> {
        if self.interactions.len() < 10 {
            return None;
        }

        // Analyze interaction patterns
        let recent = self.get_recent_interactions(20);

        // Detect build-heavy sessions
        let build_actions = recent.iter().filter(|i| i.action_type.contains("build")).count();
        if build_actions > 15 {
            return Some(GameplayPattern {
                pattern_type: "build_focused".to_string(),
                confidence: (build_actions as f32 / recent.len() as f32).min(1.0),
                frequency: build_actions as f32 / self.session_duration().as_secs() as f32,
                description: "Player shows strong building focus".to_string(),
            });
        }

        // Detect exploration patterns
        let move_actions = recent.iter().filter(|i| i.action_type.contains("move")).count();
        if move_actions > 12 {
            return Some(GameplayPattern {
                pattern_type: "exploration_focused".to_string(),
                confidence: (move_actions as f32 / recent.len() as f32).min(1.0),
                frequency: move_actions as f32 / self.session_duration().as_secs() as f32,
                description: "Player shows strong exploration behavior".to_string(),
            });
        }

        None
    }

    pub fn session_duration(&self) -> Duration {
        if self.session_active {
            self.start_time.elapsed()
        } else {
            self.last_activity.duration_since(self.start_time)
        }
    }

    pub fn generate_summary(&self) -> SessionSummary {
        SessionSummary {
            player_id: self.player_id.clone(),
            duration: self.session_duration(),
            total_interactions: self.interactions.len(),
            metrics: self.metrics.clone(),
            patterns: self.detect_patterns(),
            completion_rate: self.calculate_completion_rate(),
            engagement_score: self.calculate_engagement_score(),
        }
    }

    fn calculate_completion_rate(&self) -> f32 {
        // Calculate based on successful actions vs attempts
        let successful = self.interactions.iter().filter(|i| i.success).count();
        if self.interactions.is_empty() {
            0.0
        } else {
            successful as f32 / self.interactions.len() as f32
        }
    }

    fn calculate_engagement_score(&self) -> f32 {
        let duration_minutes = self.session_duration().as_secs() as f32 / 60.0;
        let interaction_rate = self.interactions.len() as f32 / duration_minutes.max(1.0);

        // Normalize to 0-1 range
        (interaction_rate / 10.0).min(1.0)
    }
}

/// Comprehensive player metrics
#[derive(Debug, Clone)]
pub struct PlayerMetrics {
    pub total_playtime: Duration,
    pub actions_per_minute: f32,
    pub success_rate: f32,
    pub tool_usage: HashMap<String, u32>,
    pub building_efficiency: f32,
    pub exploration_coverage: f32,
    pub social_interactions: u32,
    pub skill_progression: HashMap<String, f32>,
    pub error_patterns: Vec<String>,
    pub peak_performance_time: Option<Duration>,
}

impl PlayerMetrics {
    pub fn new() -> Self {
        Self {
            total_playtime: Duration::new(0, 0),
            actions_per_minute: 0.0,
            success_rate: 0.0,
            tool_usage: HashMap::new(),
            building_efficiency: 0.0,
            exploration_coverage: 0.0,
            social_interactions: 0,
            skill_progression: HashMap::new(),
            error_patterns: Vec::new(),
            peak_performance_time: None,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        self.total_playtime += Duration::from_secs_f32(delta_time);
    }

    pub fn record_interaction(&mut self, interaction: &PlayerInteraction) {
        // Update tool usage
        if let Some(tool) = &interaction.tool_used {
            *self.tool_usage.entry(tool.clone()).or_insert(0) += 1;
        }

        // Track errors
        if !interaction.success {
            if let Some(error) = &interaction.error_type {
                self.error_patterns.push(error.clone());
            }
        }

        // Update skill progression
        if let Some(skill) = &interaction.skill_category {
            let current = self.skill_progression.get(skill).unwrap_or(&0.0);
            let increment = if interaction.success { 0.1 } else { 0.05 };
            self.skill_progression.insert(skill.clone(), current + increment);
        }
    }

    pub fn end_session(&mut self) {
        // Finalize metrics calculations
        self.calculate_final_metrics();
    }

    fn calculate_final_metrics(&mut self) {
        // Calculate actions per minute
        let minutes = self.total_playtime.as_secs() as f32 / 60.0;
        if minutes > 0.0 {
            // This would be calculated from actual interaction count
            self.actions_per_minute = 0.0; // Placeholder
        }

        // Calculate success rate from error patterns
        // Implementation would analyze error_patterns vs total actions
    }
}

/// Global analytics metrics
#[derive(Debug, Clone, serde::Serialize)]
pub struct GlobalMetrics {
    pub total_players: u32,
    pub active_sessions: u32,
    pub total_interactions: u64,
    pub average_session_duration: Duration,
    pub popular_tools: HashMap<String, u32>,
    pub performance_trends: Vec<PerformanceTrend>,
    pub engagement_distribution: [u32; 5], // Very Low, Low, Moderate, High, Very High
}

impl GlobalMetrics {
    pub fn new() -> Self {
        Self {
            total_players: 0,
            active_sessions: 0,
            total_interactions: 0,
            average_session_duration: Duration::new(0, 0),
            popular_tools: HashMap::new(),
            performance_trends: Vec::new(),
            engagement_distribution: [0; 5],
        }
    }

    pub fn update(&mut self, _delta_time: f32) {
        // Update trends and rolling averages
        self.update_performance_trends();
    }

    pub fn record_interaction(&mut self) {
        self.total_interactions += 1;
    }

    pub fn record_session_start(&mut self) {
        self.active_sessions += 1;
        self.total_players += 1;
    }

    pub fn record_session_end(&mut self) {
        self.active_sessions = self.active_sessions.saturating_sub(1);
    }

    fn update_performance_trends(&mut self) {
        // Add performance trend data points
        let trend = PerformanceTrend {
            timestamp: std::time::SystemTime::now(),
            active_players: self.active_sessions,
            interactions_per_second: 0.0, // Would be calculated from recent data
            average_engagement: 0.5, // Would be calculated from player data
        };

        self.performance_trends.push(trend);

        // Keep only recent trends (last 24 hours worth)
        if self.performance_trends.len() > 1440 {
            self.performance_trends.remove(0);
        }
    }
}

/// Performance trend data point
#[derive(Debug, Clone, serde::Serialize)]
pub struct PerformanceTrend {
    pub timestamp: std::time::SystemTime,
    pub active_players: u32,
    pub interactions_per_second: f32,
    pub average_engagement: f32,
}

/// Metric calculation utilities
#[derive(Debug)]
pub struct MetricCalculators {
    pub player_calculators: HashMap<String, PlayerMetricCalculator>,
}

impl MetricCalculators {
    pub fn new() -> Self {
        Self {
            player_calculators: HashMap::new(),
        }
    }

    pub fn process_interaction(&mut self, player_id: &str, interaction: &PlayerInteraction) {
        let calculator = self.player_calculators.entry(player_id.to_string())
            .or_insert_with(PlayerMetricCalculator::new);
        calculator.process_interaction(interaction);
    }
}

#[derive(Debug)]
pub struct PlayerMetricCalculator {
    pub recent_interactions: VecDeque<PlayerInteraction>,
    pub skill_tracker: HashMap<String, f32>,
    pub efficiency_tracker: EfficiencyTracker,
}

impl PlayerMetricCalculator {
    pub fn new() -> Self {
        Self {
            recent_interactions: VecDeque::new(),
            skill_tracker: HashMap::new(),
            efficiency_tracker: EfficiencyTracker::new(),
        }
    }

    pub fn process_interaction(&mut self, interaction: &PlayerInteraction) {
        self.recent_interactions.push_back(interaction.clone());

        // Keep only recent interactions for efficiency
        if self.recent_interactions.len() > 50 {
            self.recent_interactions.pop_front();
        }

        self.efficiency_tracker.record_interaction(interaction);
    }
}

#[derive(Debug)]
pub struct EfficiencyTracker {
    pub successful_actions: u32,
    pub total_actions: u32,
    pub time_to_completion: Vec<Duration>,
}

impl EfficiencyTracker {
    pub fn new() -> Self {
        Self {
            successful_actions: 0,
            total_actions: 0,
            time_to_completion: Vec::new(),
        }
    }

    pub fn record_interaction(&mut self, interaction: &PlayerInteraction) {
        self.total_actions += 1;
        if interaction.success {
            self.successful_actions += 1;
        }

        if let Some(duration) = interaction.duration {
            self.time_to_completion.push(duration);
        }
    }

    pub fn calculate_efficiency(&self) -> f32 {
        if self.total_actions == 0 {
            0.0
        } else {
            self.successful_actions as f32 / self.total_actions as f32
        }
    }
}

/// Analytics event for batch processing
#[derive(Debug, Clone)]
pub struct AnalyticsEvent {
    pub player_id: String,
    pub timestamp: Instant,
    pub event_type: AnalyticsEventType,
}

#[derive(Debug, Clone)]
pub enum AnalyticsEventType {
    Interaction(PlayerInteraction),
    SessionStart,
    SessionEnd,
}

/// Detected gameplay pattern
#[derive(Debug, Clone, serde::Serialize)]
pub struct GameplayPattern {
    pub pattern_type: String,
    pub confidence: f32,
    pub frequency: f32,
    pub description: String,
}

/// Session summary report
#[derive(Debug, Clone)]
pub struct SessionSummary {
    pub player_id: String,
    pub duration: Duration,
    pub total_interactions: usize,
    pub metrics: PlayerMetrics,
    pub patterns: Option<GameplayPattern>,
    pub completion_rate: f32,
    pub engagement_score: f32,
}