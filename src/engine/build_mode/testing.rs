/*!
 * Testing & Debug System - Seamless Development Workflow
 *
 * This module provides tools for testing games while building them,
 * including debug visualization, performance monitoring, and automated testing.
 */

use crate::engine::{
    math::{Vec3, Vec2},
    graphics::{Color},
    input::InputManager,
    error::{RobinResult, RobinError},
};
use cgmath::InnerSpace;
use super::{LogicSystem, ComponentLibrary, BuildModeState};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// The testing system manages debug tools and test automation
#[derive(Debug)]
pub struct TestingSystem {
    /// Current test mode
    mode: TestMode,

    /// Debug visualization settings
    debug_settings: DebugSettings,

    /// Performance monitor
    performance_monitor: PerformanceMonitor,

    /// Automated test bots
    test_bots: Vec<TestBot>,

    /// Test scenarios and validations
    test_scenarios: Vec<TestScenario>,

    /// Debug overlay data
    debug_overlay: DebugOverlay,
}

impl TestingSystem {
    pub fn new() -> Self {
        Self {
            mode: TestMode::Off,
            debug_settings: DebugSettings::new(),
            performance_monitor: PerformanceMonitor::new(),
            test_bots: Vec::new(),
            test_scenarios: Vec::new(),
            debug_overlay: DebugOverlay::new(),
        }
    }

    /// Update the testing system
    pub fn update(
        &mut self,
        delta_time: f32,
        input: &InputManager,
        logic_system: &LogicSystem,
        component_library: &ComponentLibrary,
        build_mode: BuildModeState,
    ) -> RobinResult<()> {
        // Handle debug mode toggle (F1 key)
        if input.is_key_just_pressed(&winit::keyboard::Key::Named(winit::keyboard::NamedKey::F1)) {
            self.toggle_debug_mode();
        }

        // Handle test bot spawning (F2 key)
        if input.is_key_just_pressed(&winit::keyboard::Key::Named(winit::keyboard::NamedKey::F2)) {
            self.spawn_test_bot()?;
        }

        // Update performance monitoring
        self.performance_monitor.update(delta_time)?;

        // Update test bots
        for bot in &mut self.test_bots {
            bot.update(delta_time, logic_system, component_library)?;
        }

        // Update debug overlay based on current mode
        match build_mode {
            BuildModeState::Test => {
                self.update_debug_overlay(logic_system, component_library)?;
            }
            BuildModeState::Build => {
                if self.debug_settings.show_in_build_mode {
                    self.update_debug_overlay(logic_system, component_library)?;
                }
            }
            BuildModeState::Play => {
                // Minimal debug in play mode
                if self.debug_settings.show_performance_in_play {
                    self.debug_overlay.update_performance_info(&self.performance_monitor);
                }
            }
        }

        Ok(())
    }

    /// Toggle debug visualization
    pub fn toggle_debug_mode(&mut self) {
        self.mode = match self.mode {
            TestMode::Off => TestMode::Debug,
            TestMode::Debug => TestMode::Performance,
            TestMode::Performance => TestMode::Full,
            TestMode::Full => TestMode::Off,
        };

        log::info!("Debug mode: {:?}", self.mode);
        self.update_debug_settings();
    }

    fn update_debug_settings(&mut self) {
        match self.mode {
            TestMode::Off => {
                self.debug_settings.show_logic_flow = false;
                self.debug_settings.show_component_states = false;
                self.debug_settings.show_performance = false;
                self.debug_settings.show_collision_shapes = false;
            }
            TestMode::Debug => {
                self.debug_settings.show_logic_flow = true;
                self.debug_settings.show_component_states = true;
                self.debug_settings.show_performance = false;
                self.debug_settings.show_collision_shapes = true;
            }
            TestMode::Performance => {
                self.debug_settings.show_logic_flow = false;
                self.debug_settings.show_component_states = false;
                self.debug_settings.show_performance = true;
                self.debug_settings.show_collision_shapes = false;
            }
            TestMode::Full => {
                self.debug_settings.show_logic_flow = true;
                self.debug_settings.show_component_states = true;
                self.debug_settings.show_performance = true;
                self.debug_settings.show_collision_shapes = true;
            }
        }
    }

    /// Spawn a test bot to automatically play the game
    pub fn spawn_test_bot(&mut self) -> RobinResult<()> {
        let bot_id = self.test_bots.len() as u32;
        let spawn_position = Vec3::new(0.0, 1.0, 0.0); // TODO: Find actual spawn point

        let bot = TestBot::new(bot_id, spawn_position, BotBehavior::Explorer);
        self.test_bots.push(bot);

        log::info!("Spawned test bot {} at {:?}", bot_id, spawn_position);
        Ok(())
    }

    /// Update debug overlay with current system state
    fn update_debug_overlay(
        &mut self,
        logic_system: &LogicSystem,
        component_library: &ComponentLibrary,
    ) -> RobinResult<()> {
        // Update logic flow visualization
        if self.debug_settings.show_logic_flow {
            self.debug_overlay.update_logic_flow(logic_system);
        }

        // Update component state visualization
        if self.debug_settings.show_component_states {
            self.debug_overlay.update_component_states(component_library);
        }

        // Update performance info
        if self.debug_settings.show_performance {
            self.debug_overlay.update_performance_info(&self.performance_monitor);
        }

        Ok(())
    }

    /// Get debug overlay for rendering
    pub fn get_debug_overlay(&self) -> &DebugOverlay {
        &self.debug_overlay
    }

    /// Get performance monitor
    pub fn get_performance_monitor(&self) -> &PerformanceMonitor {
        &self.performance_monitor
    }

    /// Get test bots
    pub fn get_test_bots(&self) -> &[TestBot] {
        &self.test_bots
    }

    /// Clear all test bots
    pub fn clear_test_bots(&mut self) {
        self.test_bots.clear();
        log::info!("Cleared all test bots");
    }

    /// Run automated test scenario
    pub fn run_test_scenario(&mut self, scenario: TestScenario) -> RobinResult<()> {
        log::info!("Running test scenario: {}", scenario.name);
        self.test_scenarios.push(scenario);
        Ok(())
    }
}

/// Debug mode settings
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TestMode {
    Off,
    Debug,
    Performance,
    Full,
}

/// Debug visualization settings
#[derive(Debug, Clone)]
pub struct DebugSettings {
    pub show_logic_flow: bool,
    pub show_component_states: bool,
    pub show_performance: bool,
    pub show_collision_shapes: bool,
    pub show_ai_paths: bool,
    pub show_in_build_mode: bool,
    pub show_performance_in_play: bool,
}

impl DebugSettings {
    pub fn new() -> Self {
        Self {
            show_logic_flow: false,
            show_component_states: false,
            show_performance: false,
            show_collision_shapes: false,
            show_ai_paths: false,
            show_in_build_mode: true,
            show_performance_in_play: false,
        }
    }
}

/// Performance monitoring system
#[derive(Debug)]
pub struct PerformanceMonitor {
    /// Frame time tracking
    frame_times: VecDeque<f32>,
    max_frame_samples: usize,

    /// FPS calculation
    fps: f32,
    frame_count: u32,
    fps_timer: f32,

    /// Memory usage tracking
    memory_usage: MemoryUsage,

    /// Logic system performance
    logic_execution_time: f32,
    logic_node_count: usize,

    /// Rendering performance
    draw_calls: u32,
    vertices_rendered: u32,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            frame_times: VecDeque::with_capacity(120), // 2 seconds at 60fps
            max_frame_samples: 120,
            fps: 0.0,
            frame_count: 0,
            fps_timer: 0.0,
            memory_usage: MemoryUsage::new(),
            logic_execution_time: 0.0,
            logic_node_count: 0,
            draw_calls: 0,
            vertices_rendered: 0,
        }
    }

    pub fn update(&mut self, delta_time: f32) -> RobinResult<()> {
        // Track frame time
        self.frame_times.push_back(delta_time);
        if self.frame_times.len() > self.max_frame_samples {
            self.frame_times.pop_front();
        }

        // Calculate FPS
        self.frame_count += 1;
        self.fps_timer += delta_time;

        if self.fps_timer >= 1.0 {
            self.fps = self.frame_count as f32 / self.fps_timer;
            self.frame_count = 0;
            self.fps_timer = 0.0;
        }

        // Update memory usage (simplified)
        self.memory_usage.update();

        Ok(())
    }

    pub fn record_logic_performance(&mut self, execution_time: f32, node_count: usize) {
        self.logic_execution_time = execution_time;
        self.logic_node_count = node_count;
    }

    pub fn record_render_performance(&mut self, draw_calls: u32, vertices: u32) {
        self.draw_calls = draw_calls;
        self.vertices_rendered = vertices;
    }

    pub fn get_fps(&self) -> f32 {
        self.fps
    }

    pub fn get_average_frame_time(&self) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32
    }

    pub fn get_memory_usage(&self) -> &MemoryUsage {
        &self.memory_usage
    }
}

#[derive(Debug, Clone)]
pub struct MemoryUsage {
    pub total_mb: f32,
    pub used_mb: f32,
    pub logic_system_mb: f32,
    pub graphics_mb: f32,
}

impl MemoryUsage {
    pub fn new() -> Self {
        Self {
            total_mb: 0.0,
            used_mb: 0.0,
            logic_system_mb: 0.0,
            graphics_mb: 0.0,
        }
    }

    pub fn update(&mut self) {
        // TODO: Implement actual memory usage tracking
        // For now, use placeholder values
        self.total_mb = 512.0;
        self.used_mb = 128.0;
        self.logic_system_mb = 32.0;
        self.graphics_mb = 64.0;
    }
}

/// Automated test bot
#[derive(Debug, Clone)]
pub struct TestBot {
    pub id: u32,
    pub position: Vec3,
    pub velocity: Vec3,
    pub behavior: BotBehavior,
    pub state: BotState,
    pub path: Vec<Vec3>,
    pub target_index: usize,
    pub stuck_timer: f32,
    pub test_results: TestResults,
}

impl TestBot {
    pub fn new(id: u32, position: Vec3, behavior: BotBehavior) -> Self {
        Self {
            id,
            position,
            velocity: Vec3::new(0.0, 0.0, 0.0),
            behavior,
            state: BotState::Idle,
            path: Vec::new(),
            target_index: 0,
            stuck_timer: 0.0,
            test_results: TestResults::new(),
        }
    }

    pub fn update(
        &mut self,
        delta_time: f32,
        logic_system: &LogicSystem,
        component_library: &ComponentLibrary,
    ) -> RobinResult<()> {
        match self.behavior {
            BotBehavior::Explorer => {
                self.update_explorer_behavior(delta_time)?;
            }
            BotBehavior::Tester => {
                self.update_tester_behavior(delta_time, logic_system, component_library)?;
            }
            BotBehavior::Speedrunner => {
                self.update_speedrunner_behavior(delta_time)?;
            }
        }

        // Update position
        self.position += self.velocity * delta_time;

        // Check if stuck
        if self.velocity.magnitude() < 0.1 {
            self.stuck_timer += delta_time;
            if self.stuck_timer > 5.0 {
                log::warn!("Test bot {} appears to be stuck", self.id);
                self.test_results.record_issue("Bot stuck".to_string());
                self.stuck_timer = 0.0;
            }
        } else {
            self.stuck_timer = 0.0;
        }

        Ok(())
    }

    fn update_explorer_behavior(&mut self, delta_time: f32) -> RobinResult<()> {
        match self.state {
            BotState::Idle => {
                // Pick a random direction to explore
                self.generate_exploration_path();
                self.state = BotState::Moving;
            }
            BotState::Moving => {
                if !self.path.is_empty() && self.target_index < self.path.len() {
                    let target = self.path[self.target_index];
                    let direction = target - self.position;

                    if direction.magnitude() < 1.0 {
                        self.target_index += 1;
                        if self.target_index >= self.path.len() {
                            self.state = BotState::Idle;
                            self.path.clear();
                            self.target_index = 0;
                        }
                    } else {
                        self.velocity = direction.normalize() * 5.0; // Move at 5 units/sec
                    }
                } else {
                    self.state = BotState::Idle;
                }
            }
            BotState::Interacting => {
                // TODO: Implement interaction behavior
                self.state = BotState::Idle;
            }
        }

        Ok(())
    }

    fn update_tester_behavior(
        &mut self,
        delta_time: f32,
        logic_system: &LogicSystem,
        component_library: &ComponentLibrary,
    ) -> RobinResult<()> {
        // Systematically test interactive components
        for (component_id, component) in component_library.get_active_components() {
            // Test proximity triggers
            let distance = (component.position - self.position).magnitude();
            if distance < 2.0 {
                self.test_results.record_interaction(*component_id, "proximity_test".to_string());
            }
        }

        // Continue with explorer behavior as base
        self.update_explorer_behavior(delta_time)?;
        Ok(())
    }

    fn update_speedrunner_behavior(&mut self, delta_time: f32) -> RobinResult<()> {
        // TODO: Implement speedrunner AI that tries to complete the game as fast as possible
        self.update_explorer_behavior(delta_time)?;
        Ok(())
    }

    fn generate_exploration_path(&mut self) {
        // Generate a simple random path
        self.path.clear();
        self.target_index = 0;

        let num_waypoints = 3;
        for _ in 0..num_waypoints {
            let random_offset = Vec3::new(
                (rand::random::<f32>() - 0.5) * 20.0,
                0.0,
                (rand::random::<f32>() - 0.5) * 20.0,
            );
            self.path.push(self.position + random_offset);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BotBehavior {
    Explorer,  // Random exploration
    Tester,    // Systematic component testing
    Speedrunner, // Try to complete objectives quickly
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BotState {
    Idle,
    Moving,
    Interacting,
}

#[derive(Debug, Clone)]
pub struct TestResults {
    pub interactions: Vec<(u32, String)>, // (component_id, interaction_type)
    pub issues: Vec<String>,
    pub completion_time: Option<f32>,
}

impl TestResults {
    pub fn new() -> Self {
        Self {
            interactions: Vec::new(),
            issues: Vec::new(),
            completion_time: None,
        }
    }

    pub fn record_interaction(&mut self, component_id: u32, interaction_type: String) {
        self.interactions.push((component_id, interaction_type));
    }

    pub fn record_issue(&mut self, issue: String) {
        self.issues.push(issue);
    }
}

/// Test scenario definition
#[derive(Debug, Clone)]
pub struct TestScenario {
    pub name: String,
    pub description: String,
    pub objectives: Vec<TestObjective>,
    pub timeout: Duration,
    pub bot_count: u32,
}

#[derive(Debug, Clone)]
pub struct TestObjective {
    pub name: String,
    pub condition: String, // TODO: Implement proper condition system
    pub completed: bool,
}

/// Debug overlay data for rendering
#[derive(Debug)]
pub struct DebugOverlay {
    pub logic_flow_lines: Vec<DebugLine>,
    pub component_info: Vec<DebugText>,
    pub performance_text: Vec<DebugText>,
    pub collision_shapes: Vec<DebugShape>,
}

impl DebugOverlay {
    pub fn new() -> Self {
        Self {
            logic_flow_lines: Vec::new(),
            component_info: Vec::new(),
            performance_text: Vec::new(),
            collision_shapes: Vec::new(),
        }
    }

    pub fn update_logic_flow(&mut self, logic_system: &LogicSystem) {
        self.logic_flow_lines.clear();

        // Create lines showing data flow between nodes
        for connection in logic_system.get_connections() {
            if let (Some(from_node), Some(to_node)) = (
                logic_system.get_node(connection.from_node),
                logic_system.get_node(connection.to_node)
            ) {
                let line = DebugLine {
                    start: from_node.position,
                    end: to_node.position,
                    color: connection.wire_type.get_color(),
                    thickness: 2.0,
                };
                self.logic_flow_lines.push(line);
            }
        }
    }

    pub fn update_component_states(&mut self, component_library: &ComponentLibrary) {
        self.component_info.clear();

        // Show component states
        for (component_id, component) in component_library.get_active_components() {
            let state_text = format!("{}: {:?}", component.get_display_name(), component.state);
            let info = DebugText {
                position: component.position + Vec3::new(0.0, 2.0, 0.0),
                text: state_text,
                color: Color::new(1.0, 1.0, 1.0, 0.8),
                size: 12.0,
            };
            self.component_info.push(info);
        }
    }

    pub fn update_performance_info(&mut self, monitor: &PerformanceMonitor) {
        self.performance_text.clear();

        let fps_text = format!("FPS: {:.1}", monitor.get_fps());
        let frame_time_text = format!("Frame: {:.2}ms", monitor.get_average_frame_time() * 1000.0);
        let memory_text = format!("Memory: {:.1}MB", monitor.get_memory_usage().used_mb);

        self.performance_text.push(DebugText {
            position: Vec3::new(-10.0, 8.0, 0.0), // Top-left of screen
            text: fps_text,
            color: Color::new(0.0, 1.0, 0.0, 1.0),
            size: 16.0,
        });

        self.performance_text.push(DebugText {
            position: Vec3::new(-10.0, 7.0, 0.0),
            text: frame_time_text,
            color: Color::new(0.0, 1.0, 0.0, 1.0),
            size: 16.0,
        });

        self.performance_text.push(DebugText {
            position: Vec3::new(-10.0, 6.0, 0.0),
            text: memory_text,
            color: Color::new(0.0, 1.0, 0.0, 1.0),
            size: 16.0,
        });
    }
}

#[derive(Debug, Clone)]
pub struct DebugLine {
    pub start: Vec3,
    pub end: Vec3,
    pub color: Color,
    pub thickness: f32,
}

#[derive(Debug, Clone)]
pub struct DebugText {
    pub position: Vec3,
    pub text: String,
    pub color: Color,
    pub size: f32,
}

#[derive(Debug, Clone)]
pub struct DebugShape {
    pub position: Vec3,
    pub shape_type: DebugShapeType,
    pub color: Color,
}

#[derive(Debug, Clone)]
pub enum DebugShapeType {
    Box { size: Vec3 },
    Sphere { radius: f32 },
    Cylinder { radius: f32, height: f32 },
}

impl Default for TestingSystem {
    fn default() -> Self {
        Self::new()
    }
}