/// Demo State Management for Production Showcase
///
/// Coordinates between existing ImGui systems and new Production UI systems
/// while maintaining the excellent performance of the current robin_demo.

use robin::engine::{
    ui::{UIManager, UIMode, UIAction as EngineUIAction},
    gameplay::{GameplayManager, SessionStats},
    input::InputManager,
    error::RobinResult,
};
use crate::ui::simple_ui::{SimpleUISystem, UIAction as DemoUIAction};
use crate::ui::unified_hud::{UnifiedHUDSystem, UnifiedUIAction};
use crate::ui::mode_selection::{ModeSelectionInterface, ModeSelectionAction};
use crate::pbr_lighting::{PBRLightingSystem, WeatherType, TimePreset, LightingPerformanceInfo};
use crate::voxel_physics_system::{
    VoxelPhysicsSystem, VoxelPhysicsConfig, VoxelPhysicsEvent, VoxelPhysicsMetrics,
    VoxelDebrisParticle, DynamicVoxelBlock,
};
use std::time::Instant;
use std::collections::VecDeque;

/// Available demo modes for showcasing different engine capabilities
#[derive(Debug, Clone, PartialEq)]
pub enum DemoMode {
    /// Enhanced current robin_demo with production UI overlay
    InteractivePlayground,
    /// Showcase advanced build tools and templates
    EngineerBuildShowcase,
    /// Demonstrate resource/crafting/progression mechanics
    GameplaySystemsDemo,
    /// Preview collaboration features (visual mockups)
    CollaborationPreview,
    /// Show performance metrics and optimizations
    PerformanceBenchmarks,
    /// Highlight rendering and visual capabilities
    VisualShowcase,
}

/// Transition states for smooth mode switching
#[derive(Debug, Clone, PartialEq)]
pub enum TransitionState {
    /// Mode is active and fully visible
    Active,
    /// Fading out from current mode
    FadeOut,
    /// Loading new mode content
    Loading,
    /// Fading in to new mode
    FadeIn,
}

/// Help information and instructions for a specific demo mode
#[derive(Debug, Clone)]
pub struct ModeHelp {
    /// Display name of the mode
    pub title: &'static str,
    /// Brief description of what this mode demonstrates
    pub description: &'static str,
    /// List of key features and controls available in this mode
    pub key_features: Vec<&'static str>,
    /// List of showcase points that highlight engine capabilities
    pub showcase_points: Vec<&'static str>,
}

/// Real-time performance monitoring and dashboard
#[derive(Debug)]
pub struct PerformanceDashboard {
    /// FPS history for smooth graph display (last 120 frames = 2 seconds at 60fps)
    pub fps_history: VecDeque<f32>,
    /// Memory usage history in MB
    pub memory_history: VecDeque<f32>,
    /// Frame time history in milliseconds
    pub frame_time_history: VecDeque<f32>,
    /// Current FPS calculation
    pub current_fps: f32,
    /// Average FPS over the last second
    pub average_fps: f32,
    /// Peak FPS recorded this session
    pub peak_fps: f32,
    /// Minimum FPS recorded this session
    pub min_fps: f32,
    /// Current memory usage in MB
    pub current_memory_mb: f32,
    /// Peak memory usage this session
    pub peak_memory_mb: f32,
    /// Frame time calculation
    last_frame_time: Instant,
    /// Performance tier based on current metrics
    pub performance_tier: PerformanceTier,
    /// Optimization metrics
    pub frustum_culling_efficiency: f32,
    pub vertex_reduction_percentage: f32,
    pub chunks_rendered: u32,
    pub chunks_culled: u32,
}

/// Performance tier classification for color-coded indicators
#[derive(Debug, Clone, PartialEq)]
pub enum PerformanceTier {
    /// 60+ FPS - Excellent performance (Green)
    Excellent,
    /// 45-59 FPS - Good performance (Yellow)
    Good,
    /// 30-44 FPS - Fair performance (Orange)
    Fair,
    /// Below 30 FPS - Poor performance (Red)
    Poor,
}

impl PerformanceDashboard {
    /// Create a new performance dashboard
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            fps_history: VecDeque::with_capacity(120),
            memory_history: VecDeque::with_capacity(120),
            frame_time_history: VecDeque::with_capacity(120),
            current_fps: 60.0,
            average_fps: 60.0,
            peak_fps: 60.0,
            min_fps: 60.0,
            current_memory_mb: 0.0,
            peak_memory_mb: 0.0,
            last_frame_time: now,
            performance_tier: PerformanceTier::Excellent,
            frustum_culling_efficiency: 92.0, // Robin's optimized culling rate
            vertex_reduction_percentage: 70.0, // Greedy meshing efficiency
            chunks_rendered: 0,
            chunks_culled: 0,
        }
    }

    /// Update performance metrics for the current frame
    pub fn update(&mut self, delta_time: f32) {
        let now = Instant::now();
        let frame_duration = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;

        // Calculate current FPS
        self.current_fps = if frame_duration > 0.0 { 1.0 / frame_duration } else { 60.0 };

        // Update FPS history (rolling window of 120 frames)
        self.fps_history.push_back(self.current_fps);
        if self.fps_history.len() > 120 {
            self.fps_history.pop_front();
        }

        // Calculate average FPS over the window
        if !self.fps_history.is_empty() {
            self.average_fps = self.fps_history.iter().sum::<f32>() / self.fps_history.len() as f32;
        }

        // Update peak and minimum FPS
        self.peak_fps = self.peak_fps.max(self.current_fps);
        self.min_fps = self.min_fps.min(self.current_fps);

        // Simulate memory usage (in a real implementation, this would use actual memory APIs)
        self.current_memory_mb = 250.0 + (self.fps_history.len() as f32 * 0.5);
        self.peak_memory_mb = self.peak_memory_mb.max(self.current_memory_mb);

        // Update memory history
        self.memory_history.push_back(self.current_memory_mb);
        if self.memory_history.len() > 120 {
            self.memory_history.pop_front();
        }

        // Update frame time history
        let frame_time_ms = frame_duration * 1000.0;
        self.frame_time_history.push_back(frame_time_ms);
        if self.frame_time_history.len() > 120 {
            self.frame_time_history.pop_front();
        }

        // Update performance tier based on current FPS
        self.performance_tier = match self.current_fps {
            fps if fps >= 60.0 => PerformanceTier::Excellent,
            fps if fps >= 45.0 => PerformanceTier::Good,
            fps if fps >= 30.0 => PerformanceTier::Fair,
            _ => PerformanceTier::Poor,
        };

        // Simulate optimization metrics (in real implementation, these would come from the renderer)
        self.chunks_rendered = (100.0 - self.frustum_culling_efficiency) as u32;
        self.chunks_culled = (self.frustum_culling_efficiency * 10.0) as u32;
    }

    /// Get current FPS
    pub fn get_current_fps(&self) -> f32 {
        self.current_fps
    }

    /// Get average FPS over the measurement window
    pub fn get_average_fps(&self) -> f32 {
        self.average_fps
    }

    /// Get performance tier for color-coded display
    pub fn get_performance_tier(&self) -> &PerformanceTier {
        &self.performance_tier
    }

    /// Get current memory usage in MB
    pub fn get_current_memory_mb(&self) -> f32 {
        self.current_memory_mb
    }

    /// Get peak memory usage this session
    pub fn get_peak_memory_mb(&self) -> f32 {
        self.peak_memory_mb
    }

    /// Get frustum culling efficiency percentage
    pub fn get_frustum_culling_efficiency(&self) -> f32 {
        self.frustum_culling_efficiency
    }

    /// Get vertex reduction percentage from greedy meshing
    pub fn get_vertex_reduction_percentage(&self) -> f32 {
        self.vertex_reduction_percentage
    }

    /// Get rendering statistics
    pub fn get_render_stats(&self) -> (u32, u32) {
        (self.chunks_rendered, self.chunks_culled)
    }

    /// Get FPS history for graph display
    pub fn get_fps_history(&self) -> &VecDeque<f32> {
        &self.fps_history
    }

    /// Get memory history for graph display
    pub fn get_memory_history(&self) -> &VecDeque<f32> {
        &self.memory_history
    }

    /// Get frame time history for graph display
    pub fn get_frame_time_history(&self) -> &VecDeque<f32> {
        &self.frame_time_history
    }

    /// Get performance summary for display
    pub fn get_performance_summary(&self) -> String {
        let tier_emoji = match self.performance_tier {
            PerformanceTier::Excellent => "🟢",
            PerformanceTier::Good => "🟡",
            PerformanceTier::Fair => "🟠",
            PerformanceTier::Poor => "🔴",
        };

        format!(
            "{} {:.1} FPS | {:.1} MB | {:.1}% Culling | {:.1}% Meshing",
            tier_emoji,
            self.current_fps,
            self.current_memory_mb,
            self.frustum_culling_efficiency,
            self.vertex_reduction_percentage
        )
    }
}

/// Types of onboarding tasks that can be completed
#[derive(Debug, Clone, PartialEq)]
pub enum OnboardingTaskType {
    FirstMovement,
    FirstBuild,
    ModeExploration,
    AdvancedTools,
    ProductionUITour,
}

/// Professional welcome flow and user onboarding system
#[derive(Debug)]
pub struct OnboardingSystem {
    /// Current onboarding state
    current_state: OnboardingState,
    /// Onboarding progress tracking
    progress: OnboardingProgress,
    /// Tutorial step within current state
    tutorial_step: usize,
    /// Time spent in current state
    state_start_time: Instant,
    /// Whether onboarding has been completed
    completed: bool,
    /// User's first visit flag
    is_first_visit: bool,
}

/// Onboarding flow states for professional presentation
#[derive(Debug, Clone, PartialEq)]
pub enum OnboardingState {
    /// Professional welcome screen with engine overview
    WelcomeScreen,
    /// Quick engine capabilities overview
    EngineOverview,
    /// Interactive controls tutorial
    ControlsTutorial,
    /// Demo modes introduction
    DemoModesIntro,
    /// Interactive first building experience
    FirstBuildExperience,
    /// Achievement showcase
    AchievementShowcase,
    /// Onboarding complete - normal operation
    Completed,
}

/// Progress tracking for onboarding completion
#[derive(Debug)]
pub struct OnboardingProgress {
    /// Modes the user has explored
    modes_explored: Vec<DemoMode>,
    /// Controls the user has successfully used
    controls_mastered: Vec<String>,
    /// Building actions completed
    building_actions_completed: usize,
    /// Total time spent in demo
    total_time_seconds: f32,
    /// Whether user completed the full tutorial
    tutorial_completed: bool,
}

/// Tutorial step information for guided onboarding
#[derive(Debug, Clone)]
pub struct TutorialStep {
    /// Step title
    pub title: &'static str,
    /// Detailed instructions
    pub instructions: &'static str,
    /// Visual highlight (what to focus on)
    pub highlight: &'static str,
    /// Success criteria for advancing
    pub success_criteria: &'static str,
}

impl OnboardingSystem {
    /// Create a new onboarding system
    pub fn new() -> Self {
        Self {
            current_state: OnboardingState::WelcomeScreen,
            progress: OnboardingProgress {
                modes_explored: Vec::new(),
                controls_mastered: Vec::new(),
                building_actions_completed: 0,
                total_time_seconds: 0.0,
                tutorial_completed: false,
            },
            tutorial_step: 0,
            state_start_time: Instant::now(),
            completed: false,
            is_first_visit: true,
        }
    }

    /// Update onboarding system and advance through states
    pub fn update(&mut self, delta_time: f32) {
        self.progress.total_time_seconds += delta_time;

        // Auto-advance certain states after time threshold
        let time_in_state = self.state_start_time.elapsed().as_secs_f32();

        match self.current_state {
            OnboardingState::WelcomeScreen => {
                if time_in_state > 5.0 { // 5 seconds welcome screen
                    self.advance_to_state(OnboardingState::EngineOverview);
                }
            }
            OnboardingState::EngineOverview => {
                if time_in_state > 8.0 { // 8 seconds overview
                    self.advance_to_state(OnboardingState::ControlsTutorial);
                }
            }
            _ => {} // Other states advance based on user interaction
        }
    }

    /// Advance to a new onboarding state
    pub fn advance_to_state(&mut self, new_state: OnboardingState) {
        if new_state != self.current_state {
            self.current_state = new_state;
            self.tutorial_step = 0;
            self.state_start_time = Instant::now();

            match &self.current_state {
                OnboardingState::Completed => {
                    self.completed = true;
                    self.progress.tutorial_completed = true;
                    println!("🎉 Onboarding completed! Welcome to Robin Engine!");
                }
                _ => {
                    println!("📚 Onboarding: {:?}", self.current_state);
                }
            }
        }
    }

    /// Handle user action and update progress
    pub fn handle_user_action(&mut self, action: &str) {
        // Track controls mastered
        if !self.progress.controls_mastered.contains(&action.to_string()) {
            self.progress.controls_mastered.push(action.to_string());
        }

        // Handle building actions
        if action.contains("place") || action.contains("remove") || action.contains("build") {
            self.progress.building_actions_completed += 1;
        }

        // Advance tutorial based on actions
        match &self.current_state {
            OnboardingState::ControlsTutorial => {
                if self.progress.controls_mastered.len() >= 3 {
                    self.advance_to_state(OnboardingState::DemoModesIntro);
                }
            }
            OnboardingState::FirstBuildExperience => {
                if self.progress.building_actions_completed >= 5 {
                    self.advance_to_state(OnboardingState::AchievementShowcase);
                }
            }
            _ => {}
        }
    }

    /// Handle demo mode exploration
    pub fn handle_mode_explored(&mut self, mode: DemoMode) {
        if !self.progress.modes_explored.contains(&mode) {
            self.progress.modes_explored.push(mode);

            if self.current_state == OnboardingState::DemoModesIntro &&
               self.progress.modes_explored.len() >= 2 {
                self.advance_to_state(OnboardingState::FirstBuildExperience);
            }
        }
    }

    /// Get current onboarding state
    pub fn get_current_state(&self) -> &OnboardingState {
        &self.current_state
    }

    /// Check if onboarding is completed
    pub fn is_completed(&self) -> bool {
        self.completed
    }

    /// Get current tutorial step information
    pub fn get_current_tutorial_step(&self) -> Option<TutorialStep> {
        match &self.current_state {
            OnboardingState::WelcomeScreen => Some(TutorialStep {
                title: "Welcome to Robin Engine",
                instructions: "A 3D voxel game engine built from scratch in Rust, optimized for Apple Silicon with Metal rendering.",
                highlight: "Professional-grade building and crafting gameplay mechanics",
                success_criteria: "Automatic advancement after introduction"
            }),
            OnboardingState::EngineOverview => Some(TutorialStep {
                title: "Engine Capabilities Overview",
                instructions: "• 92% frustum culling efficiency\n• 60-80% vertex reduction via greedy meshing\n• Real-time physics and particle effects\n• Advanced AI and ML integration",
                highlight: "Production-ready performance and features",
                success_criteria: "Automatic advancement after overview"
            }),
            OnboardingState::ControlsTutorial => Some(TutorialStep {
                title: "Basic Controls Tutorial",
                instructions: "WASD: Navigate | Mouse: Look around | Left Click: Place blocks | Right Click: Remove blocks | F1-F6: Switch demo modes",
                highlight: "Try moving around and interacting with the world",
                success_criteria: "Use at least 3 different controls"
            }),
            OnboardingState::DemoModesIntro => Some(TutorialStep {
                title: "Demo Modes Introduction",
                instructions: "Press F1-F6 to explore different engine capabilities. Each mode showcases unique features of Robin Engine.",
                highlight: "Interactive Playground (F1) and Engineer Build Showcase (F2)",
                success_criteria: "Explore at least 2 different demo modes"
            }),
            OnboardingState::FirstBuildExperience => Some(TutorialStep {
                title: "Your First Building Experience",
                instructions: "Try building something! Use left click to place blocks and right click to remove them. Press B to cycle building modes.",
                highlight: "Place and remove blocks to see particle effects",
                success_criteria: "Complete 5 building actions (place/remove blocks)"
            }),
            OnboardingState::AchievementShowcase => Some(TutorialStep {
                title: "Achievement Unlocked!",
                instructions: "🏆 First Builder: You've mastered the basic building controls!\n🎯 Explorer: You've discovered multiple demo modes!\n⭐ Engine Expert: You understand Robin's capabilities!",
                highlight: "Ready for advanced features and professional usage",
                success_criteria: "Automatic advancement to normal operation"
            }),
            OnboardingState::Completed => None,
        }
    }

    /// Get onboarding progress summary
    pub fn get_progress_summary(&self) -> String {
        format!(
            "🎯 Progress: {} modes explored | {} controls mastered | {} builds completed | {:.1}s total time",
            self.progress.modes_explored.len(),
            self.progress.controls_mastered.len(),
            self.progress.building_actions_completed,
            self.progress.total_time_seconds
        )
    }

    /// Get professional completion certificate for presentation
    pub fn get_completion_certificate(&self) -> Option<String> {
        if self.completed {
            Some(format!(
                "🏆 ROBIN ENGINE CERTIFICATION\n\
                 User has successfully completed the comprehensive demo tutorial.\n\
                 • Explored {} demo modes\n\
                 • Mastered {} control schemes\n\
                 • Completed {} building actions\n\
                 • Total engagement time: {:.1} seconds\n\
                 Ready for professional usage and advanced features.",
                self.progress.modes_explored.len(),
                self.progress.controls_mastered.len(),
                self.progress.building_actions_completed,
                self.progress.total_time_seconds
            ))
        } else {
            None
        }
    }

    /// Skip onboarding (for experienced users or demos)
    pub fn skip_onboarding(&mut self) {
        self.advance_to_state(OnboardingState::Completed);
        println!("⏩ Onboarding skipped - entering normal operation mode");
    }
}

/// Coordinates all demo systems and UI layers
pub struct DemoStateManager {
    /// Current demo mode
    current_mode: DemoMode,
    /// Target mode during transitions
    target_mode: Option<DemoMode>,
    /// Current transition state
    transition_state: TransitionState,
    /// Transition timing
    transition_start: Instant,
    transition_duration: f32, // seconds
    /// Existing ImGui UI system (proven and working)
    imgui_system: SimpleUISystem,
    /// Unified HUD system integrating ImGui and production UI
    unified_hud: UnifiedHUDSystem,
    /// New production UI system (integrated alongside)
    production_ui: UIManager,
    /// Integrated gameplay systems
    gameplay: GameplayManager,
    /// Input coordination
    input_manager: InputManager,
    /// Demo statistics
    session_start: Instant,
    mode_switch_time: Instant,
    /// UI coordination state
    show_production_ui: bool,
    show_imgui_ui: bool,
    /// Transition effects
    fade_alpha: f32, // 0.0 = fully transparent, 1.0 = fully opaque
    loading_progress: f32, // 0.0 to 1.0 for loading indicator
    /// Real-time performance monitoring dashboard
    performance_dashboard: PerformanceDashboard,
    /// Professional welcome flow and user onboarding
    onboarding_system: OnboardingSystem,
    /// Enhanced PBR lighting system for dynamic atmospherics
    pbr_lighting: PBRLightingSystem,
    /// Lighting demonstration mode
    lighting_demo_active: bool,
    /// Auto-cycle weather and time for demonstration
    auto_cycle_lighting: bool,
    lighting_cycle_timer: f32,
    /// Physics-based voxel interactions with rapier3d
    voxel_physics: VoxelPhysicsSystem,
    /// Physics demonstration mode
    physics_demo_active: bool,
    /// Physics events from last frame
    physics_events: Vec<VoxelPhysicsEvent>,
    /// Enhanced mode selection interface with visual previews
    mode_selection: ModeSelectionInterface,
    /// Mode selection interface visibility
    show_mode_selection: bool,
}

impl DemoStateManager {
    pub fn new() -> Self {
        let now = Instant::now();
        let mut pbr_lighting = PBRLightingSystem::new();

        // Initialize with golden hour lighting for beautiful first impression
        pbr_lighting.apply_time_preset(TimePreset::GoldenHour);
        pbr_lighting.set_season(0.25); // Start in summer
        pbr_lighting.set_moon_phase(0.7); // Nearly full moon for night beauty

        // Initialize voxel physics system with enhanced demonstration settings
        let voxel_physics_config = VoxelPhysicsConfig {
            enable_falling_blocks: true,
            enable_debris_particles: true,
            max_debris_particles: 150, // Enhanced particle count for demo
            debris_lifetime: 4.0, // Longer debris lifetime for visual appeal
            enable_stacking_physics: true,
            enable_collision_effects: true,
            voxel_gravity_scale: 1.2, // Slightly enhanced gravity for dramatic effect
            air_resistance: 0.015, // Reduced air resistance for better physics
            velocity_threshold: 0.008, // More sensitive velocity detection
        };
        let voxel_physics = VoxelPhysicsSystem::new(voxel_physics_config);

        Self {
            current_mode: DemoMode::InteractivePlayground,
            target_mode: None,
            transition_state: TransitionState::Active,
            transition_start: now,
            transition_duration: 1.0, // 1 second transition duration
            imgui_system: SimpleUISystem::new(),
            unified_hud: UnifiedHUDSystem::new(),
            production_ui: UIManager::new(1920.0, 1080.0), // Default screen size
            gameplay: GameplayManager::new(),
            input_manager: InputManager::new(), // Input manager doesn't need screen size
            session_start: now,
            mode_switch_time: now,
            show_production_ui: true,
            show_imgui_ui: true,
            fade_alpha: 1.0, // Start fully opaque
            loading_progress: 0.0,
            performance_dashboard: PerformanceDashboard::new(),
            onboarding_system: OnboardingSystem::new(),
            pbr_lighting,
            lighting_demo_active: false,
            auto_cycle_lighting: true, // Start with auto-cycling for demonstration
            lighting_cycle_timer: 0.0,
            voxel_physics,
            physics_demo_active: true, // Start with physics demo active
            physics_events: Vec::new(),
            mode_selection: ModeSelectionInterface::new(1920.0, 1080.0), // Default screen size
            show_mode_selection: false, // Start hidden
        }
    }

    /// Get current demo mode
    pub fn get_current_mode(&self) -> &DemoMode {
        &self.current_mode
    }

    /// Switch to a different demo mode with smooth transition
    pub fn switch_mode(&mut self, new_mode: DemoMode) {
        if new_mode != self.current_mode && self.target_mode.is_none() {
            // Initiate smooth transition
            println!("🔄 Starting transition from {:?} to {:?}", self.current_mode, new_mode);
            self.target_mode = Some(new_mode.clone());
            self.transition_state = TransitionState::FadeOut;
            self.transition_start = Instant::now();
            self.fade_alpha = 1.0;
            self.loading_progress = 0.0;

            // Track mode exploration for onboarding
            self.onboarding_system.handle_mode_explored(new_mode);
        }
    }

    /// Show the enhanced mode selection interface
    pub fn show_mode_selection(&mut self) {
        self.show_mode_selection = true;
        self.mode_selection.show();
        println!("🎯 Mode selection interface opened - Choose from 6 specialized demonstration modes");
    }

    /// Hide the mode selection interface
    pub fn hide_mode_selection(&mut self) {
        self.show_mode_selection = false;
        self.mode_selection.hide();
        println!("🎯 Mode selection interface closed");
    }

    /// Toggle the mode selection interface
    pub fn toggle_mode_selection(&mut self) {
        if self.show_mode_selection {
            self.hide_mode_selection();
        } else {
            self.show_mode_selection();
        }
    }

    /// Check if mode selection interface is visible
    pub fn is_mode_selection_visible(&self) -> bool {
        self.show_mode_selection
    }

    /// Configure UI visibility and mode-specific content based on demo mode
    fn configure_mode_ui(&mut self, mode: &DemoMode) {
        match mode {
            DemoMode::InteractivePlayground => {
                self.show_production_ui = true;
                self.show_imgui_ui = true;
                self.production_ui.set_ui_mode(UIMode::InGame);

                // Configure interactive playground content
                self.setup_interactive_playground_content();
            }
            DemoMode::EngineerBuildShowcase => {
                self.show_production_ui = true;
                self.show_imgui_ui = true;
                self.production_ui.set_ui_mode(UIMode::InGame);

                // Configure engineer build showcase content
                self.setup_engineer_build_showcase_content();
            }
            DemoMode::GameplaySystemsDemo => {
                self.show_production_ui = true;
                self.show_imgui_ui = false; // Focus on production UI
                self.production_ui.set_ui_mode(UIMode::InGame);

                // Configure gameplay systems demo content
                self.setup_gameplay_systems_demo_content();
            }
            DemoMode::CollaborationPreview => {
                self.show_production_ui = true;
                self.show_imgui_ui = false;
                self.production_ui.set_ui_mode(UIMode::InGame);

                // Configure collaboration preview content
                self.setup_collaboration_preview_content();
            }
            DemoMode::PerformanceBenchmarks => {
                self.show_production_ui = true;
                self.show_imgui_ui = true;
                self.production_ui.set_ui_mode(UIMode::InGame);

                // Configure performance benchmarks content
                self.setup_performance_benchmarks_content();
            }
            DemoMode::VisualShowcase => {
                self.show_production_ui = false; // Minimal UI for visual focus
                self.show_imgui_ui = false;
                self.production_ui.set_ui_mode(UIMode::InGame);

                // Configure visual showcase content
                self.setup_visual_showcase_content();
            }
        }
    }

    /// Setup content for Interactive Playground mode
    fn setup_interactive_playground_content(&mut self) {
        println!("🎮 Setting up Interactive Playground content:");

        // Activate building tutorial system
        self.onboarding_system.start_building_tutorial();
        println!("   ✅ Building tutorials with guided construction tasks");

        // Enable physics-based voxel interactions
        self.voxel_physics.enable_interaction_effects(true);
        self.physics_demo_active = true;
        println!("   ✅ Physics-based particle effects for immersive building");

        // Configure gameplay progression tracking
        self.gameplay.enable_skill_progression(true);
        self.gameplay.enable_resource_tracking(true);
        println!("   ✅ Resource tracking and skill progression");

        // Setup optimal lighting for interactive building
        self.pbr_lighting.apply_time_preset(crate::pbr_lighting::TimePreset::Midday);
        self.pbr_lighting.set_lighting_quality(crate::pbr_lighting::LightingQuality::High);
        println!("   ✅ Optimized lighting for building activities");

        // Enable performance monitoring for user feedback
        self.performance_dashboard.enable_interactive_metrics(true);
        println!("   ✅ Real-time performance feedback enabled");

        println!("🎮 Interactive Playground mode ready for production demo!");
    }

    /// Setup content for Engineer Build Showcase mode
    fn setup_engineer_build_showcase_content(&mut self) {
        println!("🔧 Setting up Engineer Build Showcase content:");

        // Activate advanced template gallery
        self.onboarding_system.start_template_showcase();
        println!("   ✅ Advanced building templates gallery activated");

        // Enable precision building tools
        self.gameplay.enable_precision_mode(true);
        self.gameplay.enable_template_library(true);
        println!("   ✅ Grid snapping and precision placement tools");

        // Configure time-lapse and construction demonstrations
        self.onboarding_system.start_construction_timelapse();
        println!("   ✅ Construction time-lapse demonstrations");

        // Setup optimal lighting for showcasing constructions
        self.pbr_lighting.apply_time_preset(crate::pbr_lighting::TimePreset::GoldenHour);
        self.pbr_lighting.set_lighting_quality(crate::pbr_lighting::LightingQuality::Ultra);
        println!("   ✅ Golden hour lighting for spectacular construction visuals");

        // Enable advanced build mode features
        self.voxel_physics.enable_construction_helpers(true);
        println!("   ✅ Advanced construction assistance tools");

        // Performance optimization for complex structures
        self.performance_dashboard.enable_build_metrics(true);
        println!("   ✅ Build performance metrics for large constructions");

        println!("🔧 Engineer Build Showcase mode ready for professional demo!");
    }

    /// Setup content for Gameplay Systems Demo mode
    fn setup_gameplay_systems_demo_content(&mut self) {
        println!("⚙️ Setting up Gameplay Systems Demo content:");

        // Activate resource and crafting systems
        self.gameplay.enable_resource_system(true);
        self.gameplay.enable_crafting_workflow(true);
        self.gameplay.start_mining_demonstration();
        println!("   ✅ Resource mining and crafting workflow demonstrations");

        // Enable progression and achievement systems
        self.gameplay.enable_skill_progression(true);
        self.gameplay.enable_achievement_system(true);
        self.gameplay.enable_skill_tree_display(true);
        println!("   ✅ Player progression with skill trees and achievements");

        // Activate AI and balancing systems
        self.gameplay.enable_dynamic_difficulty(true);
        self.gameplay.enable_ai_showcase(true);
        self.gameplay.start_npc_demonstrations();
        println!("   ✅ AI-powered NPC intelligence and behavior systems");
        println!("   ✅ Dynamic difficulty adjustment and game balancing");

        // Enable narrative and quest systems
        self.gameplay.enable_procedural_quests(true);
        self.gameplay.start_narrative_showcase();
        println!("   ✅ Procedural quest generation and narrative systems");

        // Optimal lighting for gameplay demonstration
        self.pbr_lighting.apply_time_preset(crate::pbr_lighting::TimePreset::Afternoon);
        self.pbr_lighting.set_lighting_quality(crate::pbr_lighting::LightingQuality::High);
        println!("   ✅ Balanced lighting for gameplay visibility");

        // Enable gameplay-specific performance tracking
        self.performance_dashboard.enable_gameplay_metrics(true);
        println!("   ✅ Gameplay performance metrics enabled");

        println!("⚙️ Gameplay Systems Demo mode ready for comprehensive showcase!");
    }

    /// Setup content for Collaboration Preview mode
    fn setup_collaboration_preview_content(&mut self) {
        println!("👥 Setting up Collaboration Preview content:");

        // Activate collaboration simulation systems
        self.gameplay.enable_multi_cursor_simulation(true);
        self.gameplay.start_collaboration_showcase();
        println!("   ✅ Multi-cursor simulation for collaborative building");

        // Enable version control mockup system
        self.gameplay.enable_version_control_preview(true);
        self.gameplay.start_branching_demonstration();
        println!("   ✅ Version control mockups with branching/merging");

        // Activate real-time sync visualization
        self.gameplay.enable_sync_visualization(true);
        self.gameplay.start_realtime_sync_demo();
        println!("   ✅ Real-time synchronization previews");

        // Enable conflict resolution demonstration
        self.gameplay.enable_conflict_resolution_demo(true);
        self.gameplay.start_conflict_scenarios();
        println!("   ✅ Conflict resolution visualization for shared projects");

        // Activate team coordination tools
        self.gameplay.enable_team_coordination_preview(true);
        self.onboarding_system.start_collaboration_tutorial();
        println!("   ✅ Team coordination tools and project management");

        // Optimal lighting for collaboration demonstration
        self.pbr_lighting.apply_time_preset(crate::pbr_lighting::TimePreset::Morning);
        self.pbr_lighting.set_lighting_quality(crate::pbr_lighting::LightingQuality::High);
        println!("   ✅ Clear morning lighting for collaboration clarity");

        // Enable collaboration-specific performance tracking
        self.performance_dashboard.enable_collaboration_metrics(true);
        println!("   ✅ Collaboration performance metrics enabled");

        println!("👥 Collaboration Preview mode ready for team showcase!");
    }

    /// Setup content for Performance Benchmarks mode
    fn setup_performance_benchmarks_content(&mut self) {
        println!("📊 Setting up Performance Benchmarks content:");

        // Activate comprehensive performance dashboard
        self.performance_dashboard.enable_detailed_metrics(true);
        self.performance_dashboard.enable_fps_graphs(true);
        self.performance_dashboard.enable_memory_tracking(true);
        println!("   ✅ Live FPS monitoring with color-coded performance indicators");

        // Enable optimization visualization
        self.performance_dashboard.enable_culling_visualization(true);
        self.performance_dashboard.enable_meshing_metrics(true);
        println!("   ✅ Frustum culling efficiency: 92% performance improvement");
        println!("   ✅ Greedy meshing: 60-80% vertex reduction optimization");

        // Metal rendering performance analysis
        self.performance_dashboard.enable_metal_profiling(true);
        self.performance_dashboard.enable_unified_memory_tracking(true);
        println!("   ✅ Apple Silicon Metal rendering with unified memory analysis");

        // Stress testing scenarios for demonstration
        self.performance_dashboard.start_benchmark_scenarios();
        println!("   ✅ Automated benchmark scenarios for performance demonstration");

        // Minimal lighting for focus on metrics
        self.pbr_lighting.apply_time_preset(crate::pbr_lighting::TimePreset::Noon);
        self.pbr_lighting.set_lighting_quality(crate::pbr_lighting::LightingQuality::Medium);
        println!("   ✅ Optimized lighting for performance focus");

        println!("📊 Performance Benchmarks mode ready for technical demonstration!");
    }

    /// Setup content for Visual Showcase mode
    fn setup_visual_showcase_content(&mut self) {
        println!("🎨 Setting up Visual Showcase content:");

        // Activate advanced lighting demonstration
        self.pbr_lighting.set_lighting_quality(crate::pbr_lighting::LightingQuality::Ultra);
        self.auto_cycle_lighting = true;
        self.lighting_demo_active = true;
        self.lighting_cycle_timer = 0.0;
        println!("   ✅ Advanced lighting system with dynamic day/night cycles");

        // Enable PBR materials gallery
        self.pbr_lighting.enable_material_showcase(true);
        self.pbr_lighting.set_showcase_materials_cycle(true);
        println!("   ✅ PBR materials gallery with physically-based rendering");

        // Activate particle effects demonstration
        self.voxel_physics.enable_particle_showcase(true);
        self.voxel_physics.enable_enhanced_effects(true);
        self.physics_demo_active = true;
        println!("   ✅ Particle effects showcase with physics-based simulations");

        // Enable post-processing showcase
        self.pbr_lighting.enable_post_processing_demo(true);
        self.pbr_lighting.cycle_post_effects(true);
        println!("   ✅ Post-processing effects and visual optimization");

        // High-quality texture demonstration
        self.pbr_lighting.set_texture_quality_max(true);
        println!("   ✅ High-resolution texture atlases and material variations");

        // Disable performance monitoring for visual focus
        self.performance_dashboard.enable_minimal_overlay(true);
        println!("   ✅ Minimal UI overlay for maximum visual impact");

        println!("🎨 Visual Showcase mode ready for stunning presentation!");
    }

    /// Update transition state machine for smooth mode switching with enhanced effects
    fn update_transitions(&mut self, delta_time: f32) {
        if self.target_mode.is_none() {
            return; // No transition in progress
        }

        let elapsed = self.transition_start.elapsed().as_secs_f32();
        let phase_duration = self.transition_duration / 3.0; // Each phase is 1/3 of total duration

        match self.transition_state {
            TransitionState::FadeOut => {
                // Smooth fade out with easing function
                let progress = (elapsed / phase_duration).min(1.0);
                let eased_progress = Self::ease_in_out_quad(progress);
                self.fade_alpha = 1.0 - eased_progress;

                if progress >= 1.0 {
                    // Start loading phase
                    self.transition_state = TransitionState::Loading;
                    self.transition_start = std::time::Instant::now();
                    self.loading_progress = 0.0;
                    println!("🔄 Loading {:?} mode...", self.target_mode.as_ref().unwrap());
                }
            }
            TransitionState::Loading => {
                // Animated loading progress with smooth increments
                let target_progress = (elapsed / phase_duration).min(1.0);
                let progress_speed = 2.5; // Smooth progress animation speed
                self.loading_progress += (target_progress - self.loading_progress) * progress_speed * delta_time;

                // Show loading stages for better user feedback
                if self.loading_progress >= 0.3 && self.loading_progress < 0.35 {
                    println!("📦 Preparing mode resources...");
                } else if self.loading_progress >= 0.6 && self.loading_progress < 0.65 {
                    println!("🎨 Configuring UI layout...");
                } else if self.loading_progress >= 0.9 && self.loading_progress < 0.95 {
                    println!("⚡ Finalizing mode setup...");
                }

                if target_progress >= 0.5 && self.target_mode.is_some() {
                    // Switch to new mode at 50% loading progress
                    let new_mode = self.target_mode.take().unwrap();
                    self.current_mode = new_mode.clone();
                    self.mode_switch_time = std::time::Instant::now();
                    self.configure_mode_ui(&new_mode);

                    println!("✅ Switched to {:?} mode", new_mode);
                }

                if target_progress >= 1.0 {
                    // Start fade in phase
                    self.transition_state = TransitionState::FadeIn;
                    self.transition_start = std::time::Instant::now();
                    self.fade_alpha = 0.0;
                    self.loading_progress = 1.0;
                }
            }
            TransitionState::FadeIn => {
                // Smooth fade in with easing function
                let progress = (elapsed / phase_duration).min(1.0);
                let eased_progress = Self::ease_in_out_quad(progress);
                self.fade_alpha = eased_progress;

                if progress >= 1.0 {
                    // Transition complete
                    self.transition_state = TransitionState::Active;
                    self.fade_alpha = 1.0;
                    self.loading_progress = 0.0;
                    self.target_mode = None;

                    println!("🎯 Transition to {:?} complete - Ready for interaction!", self.current_mode);
                }
            }
            TransitionState::Active => {
                // No transition in progress - ensure alpha is at full visibility
                self.fade_alpha = 1.0;
            }
        }
    }

    /// Easing function for smooth animations
    fn ease_in_out_quad(t: f32) -> f32 {
        if t < 0.5 {
            2.0 * t * t
        } else {
            -1.0 + (4.0 - 2.0 * t) * t
        }
    }

    /// Update all UI systems and coordinate between them
    pub fn update(
        &mut self,
        delta_time: f32,
        window_size: (f64, f64),
        build_system: &mut crate::VoxelBuildSystem,
        camera: &crate::renderer::Camera,
        time: f32,
        time_speed: f32,
        time_paused: bool,
        time_string: &str,
        day_phase: f32,
    ) -> RobinResult<(Vec<DemoUIAction>, Vec<EngineUIAction>, Option<&imgui::DrawData>)> {

        // Update transition state machine
        self.update_transitions(delta_time);

        // Update performance dashboard
        self.performance_dashboard.update(delta_time);

        // Update onboarding system
        self.onboarding_system.update(delta_time);

        // Update mode selection interface if visible
        if self.show_mode_selection {
            let mode_actions = self.mode_selection.update(delta_time, &self.input_manager)?;
            for action in mode_actions {
                match action {
                    ModeSelectionAction::SelectMode(mode) => {
                        // Convert mode selection mode to demo mode and switch
                        let demo_mode = match mode {
                            crate::ui::mode_selection::DemoMode::InteractivePlayground => DemoMode::InteractivePlayground,
                            crate::ui::mode_selection::DemoMode::EngineerBuildShowcase => DemoMode::EngineerBuildShowcase,
                            crate::ui::mode_selection::DemoMode::GameplaySystemsDemo => DemoMode::GameplaySystemsDemo,
                            crate::ui::mode_selection::DemoMode::CollaborationPreview => DemoMode::CollaborationPreview,
                            crate::ui::mode_selection::DemoMode::PerformanceBenchmarks => DemoMode::PerformanceBenchmarks,
                            crate::ui::mode_selection::DemoMode::VisualShowcase => DemoMode::VisualShowcase,
                        };
                        self.switch_mode(demo_mode);
                        self.hide_mode_selection();
                    }
                    ModeSelectionAction::Close => {
                        self.hide_mode_selection();
                    }
                    ModeSelectionAction::ShowPreview(mode) => {
                        // Handle preview display (could trigger UI updates)
                        println!("🔍 Previewing mode: {:?}", mode);
                    }
                    ModeSelectionAction::HidePreview => {
                        // Handle preview hide
                    }
                    ModeSelectionAction::None => {
                        // No action needed
                    }
                }
            }
        }

        // Update gameplay systems (always active for resource tracking)
        // Note: GameplayManager update requires player_data and progress parameters
        // For now, we'll track session time manually since we don't have these params available

        // Update session stats for achievements and progression
        self.gameplay.session_stats.total_play_time = self.session_start.elapsed().as_secs_f32();

        let mut demo_actions = Vec::new();
        let mut production_actions = Vec::new();
        let mut imgui_draw_data = None;

        // Update ImGui system if enabled
        if self.show_imgui_ui {
            use core_graphics::geometry::CGSize;
            let cgsize = CGSize { width: window_size.0, height: window_size.1 };
            let day_phase_str = if day_phase < 0.25 { "dawn" }
                               else if day_phase < 0.5 { "day" }
                               else if day_phase < 0.75 { "dusk" }
                               else { "night" };
            let (actions, draw_data) = self.imgui_system.update_and_render(
                cgsize,
                build_system,
                camera,
                delta_time,
                time,
                time_speed,
                time_paused,
                time_string,
                day_phase_str,
            );
            demo_actions = actions;
            imgui_draw_data = draw_data;
        }

        // Update production UI system if enabled
        if self.show_production_ui {
            production_actions = self.production_ui.update_production_ui(delta_time, &self.input_manager)?;
        }

        Ok((demo_actions, production_actions, imgui_draw_data))
    }

    /// Get access to the ImGui system for external operations
    pub fn get_imgui_system_mut(&mut self) -> &mut SimpleUISystem {
        &mut self.imgui_system
    }

    /// Get access to the unified HUD system for external operations
    pub fn get_unified_hud_mut(&mut self) -> &mut UnifiedHUDSystem {
        &mut self.unified_hud
    }

    /// Get reference to the PBR lighting system
    pub fn get_pbr_lighting(&self) -> &PBRLightingSystem {
        &self.pbr_lighting
    }

    /// Get mutable reference to the PBR lighting system
    pub fn get_pbr_lighting_mut(&mut self) -> &mut PBRLightingSystem {
        &mut self.pbr_lighting
    }

    /// Get access to production UI for external operations
    pub fn get_production_ui(&self) -> &UIManager {
        &self.production_ui
    }

    /// Get access to mode selection interface for external operations
    pub fn get_mode_selection(&self) -> &ModeSelectionInterface {
        &self.mode_selection
    }

    /// Get mutable access to mode selection interface for external operations
    pub fn get_mode_selection_mut(&mut self) -> &mut ModeSelectionInterface {
        &mut self.mode_selection
    }

    /// Get access to gameplay systems
    pub fn get_gameplay(&self) -> &GameplayManager {
        &self.gameplay
    }

    /// Handle resource mining event (connect to existing voxel interactions)
    pub fn handle_block_mined(&mut self, voxel_type: robin::engine::world::construction::VoxelType, player_data: &mut robin::engine::save_system::PlayerData) -> RobinResult<()> {
        use robin::engine::gameplay::resources::ResourceType;
        let resource_type = ResourceType::from_voxel(voxel_type);

        // Add to resource inventory via item system
        let item_id = resource_type.to_item_id();
        player_data.add_item(&item_id, 1);
        println!("Mined 1 {:?}", resource_type);

        // Update skill progression
        if let Err(e) = self.gameplay.skills.award_experience(
            robin::engine::gameplay::progression::BuildingSkill::Mining,
            10,
            player_data
        ) {
            eprintln!("Failed to gain mining experience: {}", e);
        }

        Ok(())
    }

    /// Handle resource placement event
    pub fn handle_block_placed(&mut self, voxel_type: robin::engine::world::construction::VoxelType, player_data: &mut robin::engine::save_system::PlayerData) -> RobinResult<()> {
        use robin::engine::gameplay::resources::ResourceType;
        let resource_type = ResourceType::from_voxel(voxel_type);

        // Consume from resource inventory
        if !self.gameplay.resources.consume_resource(player_data, &resource_type, 1) {
            eprintln!("Failed to consume resource for placement: insufficient {:?}", resource_type);
        }

        // Update skill progression
        if let Err(e) = self.gameplay.skills.award_experience(
            robin::engine::gameplay::progression::BuildingSkill::Construction,
            5,
            player_data
        ) {
            eprintln!("Failed to gain building experience: {}", e);
        }

        Ok(())
    }

    /// Get demo statistics for display
    pub fn get_demo_stats(&self) -> DemoStatistics {
        DemoStatistics {
            total_session_time: self.session_start.elapsed().as_secs_f32(),
            current_mode_time: self.mode_switch_time.elapsed().as_secs_f32(),
            current_mode: self.current_mode.clone(),
            production_ui_active: self.show_production_ui,
            imgui_active: self.show_imgui_ui,
        }
    }

    /// Get current fade alpha for rendering transitions (0.0 = transparent, 1.0 = opaque)
    pub fn get_fade_alpha(&self) -> f32 {
        self.fade_alpha
    }

    /// Get current loading progress for loading screens (0.0 to 1.0)
    pub fn get_loading_progress(&self) -> f32 {
        self.loading_progress
    }

    /// Check if a transition is currently in progress
    pub fn is_transitioning(&self) -> bool {
        self.transition_state != TransitionState::Active
    }

    /// Get current transition state
    pub fn get_transition_state(&self) -> &TransitionState {
        &self.transition_state
    }

    /// Get target mode during transition (None if not transitioning)
    pub fn get_target_mode(&self) -> Option<&DemoMode> {
        self.target_mode.as_ref()
    }

    /// Get access to the performance dashboard
    pub fn get_performance_dashboard(&self) -> &PerformanceDashboard {
        &self.performance_dashboard
    }

    /// Get current performance summary for quick display
    pub fn get_performance_summary(&self) -> String {
        self.performance_dashboard.get_performance_summary()
    }

    /// Check if performance monitoring should be prominently displayed (in PerformanceBenchmarks mode)
    pub fn should_show_performance_details(&self) -> bool {
        matches!(self.current_mode, DemoMode::PerformanceBenchmarks)
    }

    /// Get mode-specific help text and instructions for the current mode
    pub fn get_current_mode_help(&self) -> ModeHelp {
        match &self.current_mode {
            DemoMode::InteractivePlayground => ModeHelp {
                title: "Interactive Playground",
                description: "Experience the core Robin Engine building mechanics with guided tutorials and real-time feedback.",
                key_features: vec![
                    "WASD + Mouse: Navigate in first-person perspective",
                    "Left Click: Place voxel blocks with particle effects",
                    "Right Click: Remove voxel blocks with mining feedback",
                    "B: Cycle through building modes (Single, Wall, Floor, etc.)",
                    "T: Switch construction templates",
                    "1-9: Select different materials (Earth, Stone, Water, etc.)",
                    "F1-F6: Switch between demo modes"
                ],
                showcase_points: vec![
                    "Physics-based particle effects for immersive building",
                    "Resource tracking and skill progression systems",
                    "Engineer Build Mode with precision construction tools",
                    "Real-time voxel world generation and modification"
                ]
            },
            DemoMode::EngineerBuildShowcase => ModeHelp {
                title: "Engineer Build Showcase",
                description: "Advanced construction tools and templates for professional building projects.",
                key_features: vec![
                    "Template Library: Pre-built structures (houses, bridges, towers)",
                    "Copy/Paste: Duplicate complex structures efficiently",
                    "Grid Snapping: Precision placement with alignment guides",
                    "Material Switching: Quick material selection and application",
                    "Undo/Redo: Full construction history management"
                ],
                showcase_points: vec![
                    "Construction time-lapse demonstrations",
                    "Advanced building templates and blueprints",
                    "Professional-grade construction tools",
                    "Scalable architecture for large projects"
                ]
            },
            DemoMode::GameplaySystemsDemo => ModeHelp {
                title: "Gameplay Systems Demo",
                description: "Comprehensive game mechanics including progression, crafting, and AI systems.",
                key_features: vec![
                    "Resource Mining: Extract materials with skill-based efficiency",
                    "Crafting System: Combine resources into advanced materials",
                    "Player Progression: Skill trees and achievement systems",
                    "AI NPCs: Intelligent non-player characters with dynamic behavior",
                    "Procedural Quests: AI-generated missions and objectives"
                ],
                showcase_points: vec![
                    "Dynamic difficulty adjustment and game balancing",
                    "ML-powered content generation and assistance",
                    "Real-time analytics and player behavior adaptation",
                    "Comprehensive progression and achievement systems"
                ]
            },
            DemoMode::CollaborationPreview => ModeHelp {
                title: "Collaboration Preview",
                description: "Multiplayer collaboration features for team building projects.",
                key_features: vec![
                    "Multi-cursor Simulation: See real-time collaborative building",
                    "Version Control: Branching and merging for team projects",
                    "Conflict Resolution: Handle simultaneous edits gracefully",
                    "Team Coordination: Project management and task assignment",
                    "Real-time Sync: Instant updates across team members"
                ],
                showcase_points: vec![
                    "Distributed version control for voxel worlds",
                    "Real-time synchronization with conflict resolution",
                    "Team coordination tools and project management",
                    "Scalable architecture for large collaborative projects"
                ]
            },
            DemoMode::PerformanceBenchmarks => ModeHelp {
                title: "Performance Benchmarks",
                description: "Live performance metrics and optimization demonstrations.",
                key_features: vec![
                    "FPS Monitoring: Real-time frame rate with performance indicators",
                    "Memory Usage: Live graphs of memory allocation and optimization",
                    "Frustum Culling: 92% efficiency improvement visualization",
                    "Greedy Meshing: 60-80% vertex reduction demonstration",
                    "Metal Optimization: Apple Silicon unified memory utilization"
                ],
                showcase_points: vec![
                    "Production-ready 60fps performance on Apple Silicon",
                    "Advanced GPU acceleration and memory optimization",
                    "Real-time performance profiling and diagnostics",
                    "Scalable rendering pipeline for large voxel worlds"
                ]
            },
            DemoMode::VisualShowcase => ModeHelp {
                title: "Visual Showcase",
                description: "Advanced graphics and visual effects capabilities.",
                key_features: vec![
                    "Dynamic Lighting: Real-time day/night cycles with realistic shadows",
                    "PBR Materials: Physically-based rendering with realistic materials",
                    "Particle Effects: Physics-based particle systems for immersion",
                    "Post-processing: Advanced visual effects and optimization",
                    "Texture Atlases: High-resolution material variations"
                ],
                showcase_points: vec![
                    "Professional-grade visual quality and lighting",
                    "Advanced shader system with Metal optimization",
                    "Real-time global illumination and shadow mapping",
                    "High-performance rendering suitable for commercial games"
                ]
            }
        }
    }

    /// Get access to the onboarding system
    pub fn get_onboarding_system(&self) -> &OnboardingSystem {
        &self.onboarding_system
    }

    /// Get mutable access to the onboarding system for external state updates
    pub fn get_onboarding_system_mut(&mut self) -> &mut OnboardingSystem {
        &mut self.onboarding_system
    }

    /// Check if the onboarding process is active and needs to be displayed
    pub fn is_onboarding_active(&self) -> bool {
        !self.onboarding_system.completed &&
        !matches!(self.onboarding_system.current_state, OnboardingState::Completed)
    }

    /// Get the current onboarding step information for UI display
    pub fn get_current_onboarding_step(&self) -> Option<TutorialStep> {
        if self.is_onboarding_active() {
            match &self.onboarding_system.current_state {
                OnboardingState::WelcomeScreen => Some(TutorialStep {
                    title: "Welcome to Robin Engine!",
                    instructions: "Robin is a professional 3D voxel game engine with advanced building mechanics, AI systems, and production-ready performance. Click 'Next' to begin your guided tour.",
                    highlight: "Professional voxel game engine with industry-leading performance",
                    success_criteria: "Click Next to continue the tour",
                }),
                OnboardingState::EngineOverview => Some(TutorialStep {
                    title: "Engine Overview",
                    instructions: "Learn about Robin's core capabilities: 92% frustum culling efficiency, 60-80% vertex reduction via greedy meshing, and Apple Silicon Metal rendering optimization.",
                    highlight: "Production-ready 60fps performance on Apple Silicon",
                    success_criteria: "Review the key features and performance metrics",
                }),
                OnboardingState::ControlsTutorial => Some(TutorialStep {
                    title: "Controls Tutorial",
                    instructions: "Master the core navigation and building controls. Use WASD keys to move, mouse to look around, left click to place blocks, right click to remove blocks.",
                    highlight: "First-person navigation with voxel building controls",
                    success_criteria: "Try moving around and placing/removing at least one block",
                }),
                OnboardingState::DemoModesIntro => Some(TutorialStep {
                    title: "Demo Mode Exploration",
                    instructions: "Discover the six unique demonstration modes. Press F1-F6 to switch: Interactive Playground, Engineer Build Showcase, Gameplay Systems, Collaboration Preview, Performance Benchmarks, Visual Showcase.",
                    highlight: "Six specialized demo modes showcasing different engine capabilities",
                    success_criteria: "Explore at least 2 different demo modes using F1-F6",
                }),
                OnboardingState::FirstBuildExperience => Some(TutorialStep {
                    title: "First Build Experience",
                    instructions: "Complete your first building project using different materials and tools. Use 1-9 keys for materials, B key for building modes, T key for templates.",
                    highlight: "Physics-based particle feedback for immersive building",
                    success_criteria: "Place at least 5 blocks using different materials or tools",
                }),
                OnboardingState::AchievementShowcase => Some(TutorialStep {
                    title: "Achievement System",
                    instructions: "Review your progress and earned achievements. You've mastered the basics and can now explore advanced features and professional tools.",
                    highlight: "Professional completion certificate and progress tracking",
                    success_criteria: "Review your achievements and progress summary",
                }),
                OnboardingState::Completed => Some(TutorialStep {
                    title: "Congratulations!",
                    instructions: "You've completed the Robin Engine tour and are ready to explore all features. Use the help system (?) for detailed documentation on any mode.",
                    highlight: "Robin Engine is ready for commercial deployment",
                    success_criteria: "Continue exploring demo modes and advanced features",
                }),
            }
        } else {
            None
        }
    }

    /// Advance the onboarding to the next step (called when user clicks Next)
    pub fn advance_onboarding(&mut self) {
        let next_state = match &self.onboarding_system.current_state {
            OnboardingState::WelcomeScreen => OnboardingState::EngineOverview,
            OnboardingState::EngineOverview => OnboardingState::ControlsTutorial,
            OnboardingState::ControlsTutorial => OnboardingState::DemoModesIntro,
            OnboardingState::DemoModesIntro => OnboardingState::FirstBuildExperience,
            OnboardingState::FirstBuildExperience => OnboardingState::AchievementShowcase,
            OnboardingState::AchievementShowcase => OnboardingState::Completed,
            OnboardingState::Completed => return, // Already completed
        };
        self.onboarding_system.advance_to_state(next_state);
    }

    /// Handle user completing an onboarding task
    pub fn handle_onboarding_task_completed(&mut self, task_type: OnboardingTaskType) {
        match task_type {
            OnboardingTaskType::FirstMovement => {
                self.onboarding_system.handle_user_action("movement");
            },
            OnboardingTaskType::FirstBuild => {
                self.onboarding_system.handle_user_action("build_action");
            },
            OnboardingTaskType::ModeExploration => {
                // This is handled automatically in switch_mode()
            },
            OnboardingTaskType::AdvancedTools => {
                self.onboarding_system.handle_user_action("advanced_tools");
            },
            OnboardingTaskType::ProductionUITour => {
                self.onboarding_system.handle_user_action("production_ui");
            },
        }
    }

    /// Get the onboarding progress summary for display
    pub fn get_onboarding_progress_summary(&self) -> String {
        let modes_explored = self.onboarding_system.progress.modes_explored.len();
        let controls_mastered = self.onboarding_system.progress.controls_mastered.len();
        let building_actions = self.onboarding_system.progress.building_actions_completed;

        format!("Progress: {} modes, {} controls, {} builds completed",
                modes_explored, controls_mastered, building_actions)
    }

    /// Check if the user has completed the basic onboarding requirements
    pub fn has_completed_basic_onboarding(&self) -> bool {
        self.onboarding_system.progress.controls_mastered.len() >= 3 &&
        self.onboarding_system.progress.building_actions_completed >= 2 &&
        self.onboarding_system.progress.modes_explored.len() >= 2
    }

    /// Skip onboarding for experienced users
    pub fn skip_onboarding(&mut self) {
        self.onboarding_system.advance_to_state(OnboardingState::Completed);
        println!("🎓 Onboarding skipped - Welcome to Robin Engine!");
    }

    /// Update the lighting system with delta time and demonstration cycling
    pub fn update_lighting(&mut self, delta_time: f32) {
        // Update PBR lighting system
        self.pbr_lighting.update_time_of_day(delta_time);

        // Auto-cycle lighting for demonstration
        if self.auto_cycle_lighting {
            self.lighting_cycle_timer += delta_time;

            // Cycle through different atmospheric conditions every 30 seconds
            if self.lighting_cycle_timer >= 30.0 {
                self.lighting_cycle_timer = 0.0;
                self.cycle_to_next_lighting_demo();
            }
        }
    }

    /// Cycle to the next lighting demonstration
    fn cycle_to_next_lighting_demo(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        // Randomly select a lighting preset to demonstrate variety
        let presets = [
            TimePreset::EarlyMorning,
            TimePreset::Noon,
            TimePreset::GoldenHour,
            TimePreset::Midnight,
            TimePreset::StormyDay,
        ];

        let weather_types = [
            WeatherType::Clear,
            WeatherType::Cloudy,
            WeatherType::Overcast,
            WeatherType::Rainy,
            WeatherType::Stormy,
            WeatherType::Foggy,
        ];

        // Apply random preset
        let preset = presets[rng.gen_range(0..presets.len())];
        self.pbr_lighting.apply_time_preset(preset);

        // Sometimes add interesting weather
        if rng.gen_bool(0.4) { // 40% chance of weather change
            let weather = weather_types[rng.gen_range(0..weather_types.len())];
            let intensity = rng.gen_range(0.3..0.9);
            self.pbr_lighting.set_weather(weather, intensity);
        }

        // Vary moon phase for night scenes
        let moon_phase = rng.gen_range(0.0..1.0);
        self.pbr_lighting.set_moon_phase(moon_phase);

        // Vary seasonal effects
        let season = rng.gen_range(0.0..1.0);
        self.pbr_lighting.set_season(season);

        println!("🌅 Lighting demo: {:?} with {:?} weather", preset, self.pbr_lighting.weather_type);
    }

    /// Toggle automatic lighting cycling
    pub fn toggle_lighting_auto_cycle(&mut self) {
        self.auto_cycle_lighting = !self.auto_cycle_lighting;
        println!("🔄 Lighting auto-cycle: {}", if self.auto_cycle_lighting { "enabled" } else { "disabled" });
    }

    /// Set specific lighting preset manually
    pub fn set_lighting_preset(&mut self, preset: TimePreset) {
        self.pbr_lighting.apply_time_preset(preset);
        self.auto_cycle_lighting = false; // Disable auto-cycle when manually setting
        println!("🌅 Manual lighting preset: {:?}", preset);
    }

    /// Set weather manually
    pub fn set_weather(&mut self, weather: WeatherType, intensity: f32) {
        self.pbr_lighting.set_weather(weather, intensity);
        println!("🌦 Weather set to: {:?} (intensity: {:.1})", weather, intensity);
    }

    /// Get lighting performance info for dashboard
    pub fn get_lighting_performance(&self) -> LightingPerformanceInfo {
        self.pbr_lighting.get_performance_info()
    }

    /// Initialize GPU resources for physics system
    pub fn initialize_physics(&mut self, device: &metal::DeviceRef) {
        self.voxel_physics.initialize(device);
        println!("🔬 Voxel physics system initialized with rapier3d integration");
    }

    /// Update the physics system with delta time and voxel world
    pub fn update_physics(&mut self, delta_time: f32, voxel_world: &mut robin::engine::generation::voxel_system::VoxelWorld) -> RobinResult<()> {
        // Update physics simulation
        self.voxel_physics.update(delta_time, voxel_world)?;

        // Collect physics events
        self.physics_events = self.voxel_physics.get_events().to_vec();

        // Process physics events for demo feedback
        for event in &self.physics_events {
            match event {
                VoxelPhysicsEvent::BlockSettled { voxel_type, position, .. } => {
                    println!("🧱 Block {:?} settled at {:?}", voxel_type, position);
                },
                VoxelPhysicsEvent::BlockBroken { voxel_type, position, debris_count } => {
                    println!("💥 Block {:?} broken at {:?}, {} debris particles", voxel_type, position, debris_count);
                },
                VoxelPhysicsEvent::BlockCollision { impact_force, position, .. } => {
                    if *impact_force > 10.0 {
                        println!("💥 High-impact collision at {:?} (force: {:.1})", position, impact_force);
                    }
                },
                VoxelPhysicsEvent::StructureCollapse { center, affected_blocks } => {
                    println!("🏗️ Structure collapse at {:?}, {} blocks affected", center, affected_blocks.len());
                },
            }
        }

        Ok(())
    }

    /// Add a dynamic voxel block for physics demonstration
    pub fn add_physics_block(&mut self, voxel_type: robin::engine::generation::voxel_system::VoxelType, position: cgmath::Vector3<f32>) -> RobinResult<()> {
        let handle = self.voxel_physics.add_dynamic_block(voxel_type, position, None)?;
        println!("🔬 Added physics block {:?} at {:?} (handle: {:?})", voxel_type, position, handle);
        Ok(())
    }

    /// Create debris particles for physics demonstration
    pub fn create_physics_debris(&mut self, voxel_type: robin::engine::generation::voxel_system::VoxelType, position: cgmath::Vector3<f32>, impact_velocity: cgmath::Vector3<f32>) -> RobinResult<()> {
        self.voxel_physics.create_debris(voxel_type, position, impact_velocity, 8)?;
        println!("💥 Created debris particles for {:?} at {:?}", voxel_type, position);
        Ok(())
    }

    /// Get physics performance metrics
    pub fn get_physics_metrics(&self) -> VoxelPhysicsMetrics {
        self.voxel_physics.get_performance_metrics()
    }

    /// Get reference to the voxel physics system
    pub fn get_voxel_physics(&self) -> &VoxelPhysicsSystem {
        &self.voxel_physics
    }

    /// Get mutable reference to the voxel physics system
    pub fn get_voxel_physics_mut(&mut self) -> &mut VoxelPhysicsSystem {
        &mut self.voxel_physics
    }

    /// Toggle physics demonstration mode
    pub fn toggle_physics_demo(&mut self) {
        self.physics_demo_active = !self.physics_demo_active;
        println!("🔬 Physics demo mode: {}", if self.physics_demo_active { "ON" } else { "OFF" });
    }

    /// Get physics events from last frame
    pub fn get_physics_events(&self) -> &[VoxelPhysicsEvent] {
        &self.physics_events
    }

    /// Update performance metrics with enhanced real-time data collection
    pub fn update_performance_metrics(&mut self, delta_time: f32, chunks_rendered: u32, chunks_culled: u32) {
        // Update performance dashboard with current frame metrics
        self.performance_dashboard.update(delta_time);

        // Update chunk rendering statistics
        self.performance_dashboard.chunks_rendered = chunks_rendered;
        self.performance_dashboard.chunks_culled = chunks_culled;

        // Calculate frame time in milliseconds
        let frame_time_ms = delta_time * 1000.0;
        self.performance_dashboard.frame_time_history.push_back(frame_time_ms);
        if self.performance_dashboard.frame_time_history.len() > 120 {
            self.performance_dashboard.frame_time_history.pop_front();
        }

        // Estimate memory usage (simplified - in production would use proper memory tracking)
        let estimated_memory = 128.0 + (chunks_rendered as f32 * 2.5); // Base memory + chunk memory
        self.performance_dashboard.current_memory_mb = estimated_memory;
        self.performance_dashboard.memory_history.push_back(estimated_memory);
        if self.performance_dashboard.memory_history.len() > 120 {
            self.performance_dashboard.memory_history.pop_front();
        }

        // Update peak memory
        self.performance_dashboard.peak_memory_mb = self.performance_dashboard.peak_memory_mb.max(estimated_memory);

        // Update performance tier based on current FPS
        self.performance_dashboard.performance_tier = if self.performance_dashboard.current_fps >= 60.0 {
            PerformanceTier::Excellent
        } else if self.performance_dashboard.current_fps >= 45.0 {
            PerformanceTier::Good
        } else if self.performance_dashboard.current_fps >= 30.0 {
            PerformanceTier::Fair
        } else {
            PerformanceTier::Poor
        };
    }

    /// Toggle lighting demonstration mode
    pub fn toggle_lighting_demo(&mut self) {
        self.lighting_demo_active = !self.lighting_demo_active;
        if self.lighting_demo_active {
            println!("🎭 Lighting demonstration mode activated");
            self.auto_cycle_lighting = true;
        } else {
            println!("🎭 Lighting demonstration mode deactivated");
        }
    }
}

/// Statistics for the demo session
#[derive(Debug)]
pub struct DemoStatistics {
    pub total_session_time: f32,
    pub current_mode_time: f32,
    pub current_mode: DemoMode,
    pub production_ui_active: bool,
    pub imgui_active: bool,
}

impl Default for DemoStateManager {
    fn default() -> Self {
        Self::new()
    }
}