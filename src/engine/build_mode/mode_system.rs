use cgmath::{Vector3, Vector2, Quaternion, InnerSpace, Zero, One};
use crate::engine::{
    math::{Vec3, Vec2},
    input::InputManager,
    error::RobinResult,
};
use winit::event::MouseButton;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use super::{
    BuildModeState, InteractiveElementsSystem, VisualLogicSystem,
    GridSystem, SelectionManager, BuildViewport,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeSystemState {
    pub current_mode: BuildModeState,
    pub previous_mode: Option<BuildModeState>,
    pub mode_transition_time: f32,
    pub transition_duration: f32,
    pub auto_save_on_mode_switch: bool,
    pub preserve_camera_state: bool,
    pub mode_specific_settings: HashMap<String, ModeSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeSettings {
    pub enabled_tools: Vec<String>,
    pub ui_elements: Vec<String>,
    pub input_bindings: HashMap<String, String>,
    pub camera_mode: CameraMode,
    pub grid_settings: GridConfig,
    pub debug_overlays: Vec<DebugOverlay>,
    pub performance_profile: PerformanceProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CameraMode {
    FreeFly,
    FirstPerson,
    ThirdPerson,
    Orbital,
    Fixed { position: Vector3<f32>, target: Vector3<f32> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridConfig {
    pub enabled: bool,
    pub visible: bool,
    pub size: f32,
    pub snap_enabled: bool,
    pub subdivisions: u32,
    pub color: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebugOverlay {
    FPSCounter,
    MemoryUsage,
    EntityCount,
    PhysicsDebug,
    RenderStats,
    InputDebug,
    AIDebug,
    LogicConnections,
    CollisionBounds,
    NavigationMesh,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceProfile {
    Development,  // Full debugging, no optimizations
    Testing,      // Some optimizations, key debugging
    Production,   // Full optimizations, minimal debugging
    Custom(PerformanceSettings),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSettings {
    pub target_fps: u32,
    pub render_quality: f32,
    pub physics_substeps: u32,
    pub ai_update_frequency: f32,
    pub audio_quality: AudioQuality,
    pub memory_pool_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioQuality {
    Low,
    Medium,
    High,
    Ultra,
}

pub struct ModeSystem {
    state: ModeSystemState,
    mode_transition_effects: ModeTransitionEffects,
    saved_states: HashMap<BuildModeState, SavedModeState>,
    ui_manager: ModeUIManager,
    input_handler: ModeInputHandler,
    performance_monitor: PerformanceMonitor,
}

#[derive(Debug, Clone)]
pub struct ModeTransitionEffects {
    pub fade_duration: f32,
    pub current_fade: f32,
    pub fade_direction: FadeDirection,
    pub transition_sound: Option<String>,
    pub visual_effects: Vec<TransitionEffect>,
}

#[derive(Debug, Clone)]
pub enum FadeDirection {
    FadeIn,
    FadeOut,
    None,
}

#[derive(Debug, Clone)]
pub enum TransitionEffect {
    ScreenFade { color: [f32; 4], intensity: f32 },
    Blur { strength: f32 },
    GridOverlay { intensity: f32 },
    ParticleSystem { effect_type: String },
}

#[derive(Debug, Clone)]
pub struct SavedModeState {
    pub camera_position: Vector3<f32>,
    pub camera_rotation: Vector2<f32>,
    pub selected_objects: Vec<u32>,
    pub active_tool: Option<String>,
    pub ui_state: HashMap<String, String>,
    pub timestamp: f32,
}

pub struct ModeUIManager {
    pub build_ui: BuildModeUI,
    pub test_ui: TestModeUI,
    pub play_ui: PlayModeUI,
    pub transition_ui: TransitionUI,
    pub current_ui_state: UIState,
}

#[derive(Debug, Clone)]
pub struct BuildModeUI {
    pub tool_palette_visible: bool,
    pub properties_panel_visible: bool,
    pub hierarchy_panel_visible: bool,
    pub asset_browser_visible: bool,
    pub debug_console_visible: bool,
    pub grid_controls_visible: bool,
    pub quick_actions_visible: bool,
}

#[derive(Debug, Clone)]
pub struct TestModeUI {
    pub debug_overlay_visible: bool,
    pub performance_metrics_visible: bool,
    pub entity_inspector_visible: bool,
    pub console_visible: bool,
    pub quick_edit_enabled: bool,
    pub pause_button_visible: bool,
    pub reset_button_visible: bool,
}

#[derive(Debug, Clone)]
pub struct PlayModeUI {
    pub game_ui_visible: bool,
    pub settings_accessible: bool,
    pub pause_menu_enabled: bool,
    pub developer_console_hidden: bool,
    pub immersive_mode: bool,
}

#[derive(Debug, Clone)]
pub struct TransitionUI {
    pub progress_bar_visible: bool,
    pub mode_name_display: bool,
    pub instruction_text_visible: bool,
    pub loading_animation_type: LoadingAnimation,
}

#[derive(Debug, Clone)]
pub enum LoadingAnimation {
    Spinner,
    ProgressBar,
    Dots,
    Fade,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum UIState {
    Hidden,
    Visible,
    Transitioning,
    Minimized,
}

pub struct ModeInputHandler {
    pub build_mode_bindings: HashMap<String, InputBinding>,
    pub test_mode_bindings: HashMap<String, InputBinding>,
    pub play_mode_bindings: HashMap<String, InputBinding>,
    pub global_bindings: HashMap<String, InputBinding>,
    pub input_context_stack: Vec<InputContext>,
}

#[derive(Debug, Clone)]
pub struct InputBinding {
    pub action: String,
    pub keys: Vec<InputKey>,
    pub modifiers: Vec<InputModifier>,
    pub description: String,
    pub category: String,
}

#[derive(Debug, Clone)]
pub enum InputKey {
    Keyboard(winit::keyboard::Key),
    Mouse(MouseButton),
    Gamepad(GamepadButton),
}

#[derive(Debug, Clone)]
pub enum InputModifier {
    Ctrl,
    Shift,
    Alt,
    Super,
}

#[derive(Debug, Clone)]
pub enum GamepadButton {
    A, B, X, Y,
    LeftShoulder, RightShoulder,
    LeftStick, RightStick,
    DPadUp, DPadDown, DPadLeft, DPadRight,
}

#[derive(Debug, Clone)]
pub struct InputContext {
    pub name: String,
    pub bindings: HashMap<String, InputBinding>,
    pub priority: u32,
    pub exclusive: bool,
}

pub struct PerformanceMonitor {
    pub frame_times: Vec<f32>,
    pub memory_usage: Vec<f32>,
    pub entity_counts: Vec<u32>,
    pub mode_switch_times: HashMap<BuildModeState, f32>,
    pub optimization_suggestions: Vec<OptimizationSuggestion>,
}

#[derive(Debug, Clone)]
pub struct OptimizationSuggestion {
    pub severity: SeverityLevel,
    pub category: OptimizationCategory,
    pub description: String,
    pub suggested_action: String,
    pub impact_estimate: ImpactEstimate,
}

#[derive(Debug, Clone)]
pub enum SeverityLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub enum OptimizationCategory {
    Rendering,
    Physics,
    Memory,
    CPU,
    Disk,
    Network,
    AI,
}

#[derive(Debug, Clone)]
pub enum ImpactEstimate {
    Minor,
    Moderate,
    Significant,
    Major,
}

impl ModeSystem {
    pub fn new() -> Self {
        Self {
            state: ModeSystemState::default(),
            mode_transition_effects: ModeTransitionEffects::default(),
            saved_states: HashMap::new(),
            ui_manager: ModeUIManager::new(),
            input_handler: ModeInputHandler::new(),
            performance_monitor: PerformanceMonitor::new(),
        }
    }

    pub fn update(&mut self, delta_time: f32, input: &InputManager) -> RobinResult<()> {
        // Update transition effects
        self.update_transitions(delta_time)?;

        // Handle mode switching input
        self.handle_mode_switching_input(input)?;

        // Update UI for current mode
        self.ui_manager.update(delta_time, &self.state.current_mode)?;

        // Update input handling for current mode
        self.input_handler.update(input, &self.state.current_mode)?;

        // Update performance monitoring
        self.performance_monitor.update(delta_time, &self.state.current_mode)?;

        // Auto-save if configured
        if self.state.auto_save_on_mode_switch && self.is_mode_transition_complete() {
            self.auto_save_mode_state()?;
        }

        Ok(())
    }

    pub fn switch_mode(&mut self, new_mode: BuildModeState) -> RobinResult<()> {
        if self.state.current_mode == new_mode {
            return Ok(());
        }

        // Save current state
        self.save_current_mode_state()?;

        // Begin transition
        self.begin_mode_transition(new_mode)?;

        log::info!("Switching from {:?} to {:?} mode", self.state.current_mode, new_mode);

        Ok(())
    }

    pub fn cycle_mode(&mut self) -> RobinResult<()> {
        let next_mode = match self.state.current_mode {
            BuildModeState::Build => BuildModeState::Test,
            BuildModeState::Test => BuildModeState::Play,
            BuildModeState::Play => BuildModeState::Build,
        };

        self.switch_mode(next_mode)
    }

    pub fn get_current_mode(&self) -> BuildModeState {
        self.state.current_mode
    }

    pub fn is_transitioning(&self) -> bool {
        self.state.mode_transition_time > 0.0
    }

    pub fn get_transition_progress(&self) -> f32 {
        if self.state.transition_duration <= 0.0 {
            return 1.0;
        }
        1.0 - (self.state.mode_transition_time / self.state.transition_duration)
    }

    pub fn get_enabled_tools(&self) -> Vec<String> {
        self.get_mode_settings(&self.state.current_mode)
            .map(|settings| settings.enabled_tools.clone())
            .unwrap_or_default()
    }

    pub fn get_debug_overlays(&self) -> Vec<DebugOverlay> {
        self.get_mode_settings(&self.state.current_mode)
            .map(|settings| settings.debug_overlays.clone())
            .unwrap_or_default()
    }

    pub fn is_tool_enabled(&self, tool_name: &str) -> bool {
        self.get_enabled_tools().contains(&tool_name.to_string())
    }

    pub fn get_camera_mode(&self) -> CameraMode {
        self.get_mode_settings(&self.state.current_mode)
            .map(|settings| settings.camera_mode.clone())
            .unwrap_or(CameraMode::FreeFly)
    }

    pub fn apply_performance_profile(&mut self, profile: PerformanceProfile) -> RobinResult<()> {
        // Update current mode settings
        let current_mode = self.state.current_mode;
        if let Some(settings) = self.get_mode_settings_mut(&current_mode) {
            settings.performance_profile = profile;
        }

        // Apply settings immediately
        self.apply_current_performance_settings()?;

        Ok(())
    }

    fn update_transitions(&mut self, delta_time: f32) -> RobinResult<()> {
        if self.state.mode_transition_time > 0.0 {
            self.state.mode_transition_time -= delta_time;

            // Update transition effects
            self.mode_transition_effects.update(delta_time, self.get_transition_progress())?;

            // Complete transition if time is up
            if self.state.mode_transition_time <= 0.0 {
                self.complete_mode_transition()?;
            }
        }

        Ok(())
    }

    fn handle_mode_switching_input(&mut self, input: &InputManager) -> RobinResult<()> {
        // Tab key cycles modes
        if input.is_named_key_just_pressed(winit::keyboard::NamedKey::Tab) {
            self.cycle_mode()?;
        }

        // F1-F3 for direct mode switching
        if input.is_named_key_just_pressed(winit::keyboard::NamedKey::F1) {
            self.switch_mode(BuildModeState::Build)?;
        }
        if input.is_named_key_just_pressed(winit::keyboard::NamedKey::F2) {
            self.switch_mode(BuildModeState::Test)?;
        }
        if input.is_named_key_just_pressed(winit::keyboard::NamedKey::F3) {
            self.switch_mode(BuildModeState::Play)?;
        }

        Ok(())
    }

    fn begin_mode_transition(&mut self, new_mode: BuildModeState) -> RobinResult<()> {
        self.state.previous_mode = Some(self.state.current_mode);
        self.state.current_mode = new_mode;
        self.state.mode_transition_time = self.state.transition_duration;

        // Start transition effects
        self.mode_transition_effects.begin_transition()?;

        // Prepare UI for new mode
        self.ui_manager.prepare_mode_transition(new_mode)?;

        Ok(())
    }

    fn complete_mode_transition(&mut self) -> RobinResult<()> {
        // Apply new mode settings
        self.apply_mode_settings()?;

        // Restore saved state if available
        self.restore_mode_state()?;

        // Finalize UI transition
        self.ui_manager.complete_mode_transition()?;

        // End transition effects
        self.mode_transition_effects.end_transition()?;

        log::info!("Mode transition to {:?} completed", self.state.current_mode);

        Ok(())
    }

    fn save_current_mode_state(&mut self) -> RobinResult<()> {
        let saved_state = SavedModeState {
            camera_position: Vector3::new(0.0, 0.0, 0.0), // TODO: Get from actual camera
            camera_rotation: Vector2::new(0.0, 0.0),       // TODO: Get from actual camera
            selected_objects: Vec::new(),                    // TODO: Get from selection manager
            active_tool: None,                               // TODO: Get from tool system
            ui_state: HashMap::new(),                        // TODO: Get from UI manager
            timestamp: 0.0,                                  // TODO: Get actual time
        };

        self.saved_states.insert(self.state.current_mode, saved_state);

        Ok(())
    }

    fn restore_mode_state(&mut self) -> RobinResult<()> {
        if let Some(saved_state) = self.saved_states.get(&self.state.current_mode) {
            if self.state.preserve_camera_state {
                // TODO: Restore camera state
                log::debug!("Restoring camera state for {:?} mode", self.state.current_mode);
            }

            // TODO: Restore other state elements
            log::debug!("Restored mode state for {:?}", self.state.current_mode);
        }

        Ok(())
    }

    fn apply_mode_settings(&mut self) -> RobinResult<()> {
        let current_mode = self.state.current_mode;
        if let Some(settings) = self.get_mode_settings(&current_mode).cloned() {
            // Apply camera mode
            self.apply_camera_mode(&settings.camera_mode)?;

            // Apply grid settings
            self.apply_grid_settings(&settings.grid_settings)?;

            // Apply performance profile
            self.apply_performance_profile_settings(&settings.performance_profile)?;

            log::debug!("Applied settings for {:?} mode", current_mode);
        }

        Ok(())
    }

    fn apply_camera_mode(&self, camera_mode: &CameraMode) -> RobinResult<()> {
        match camera_mode {
            CameraMode::FreeFly => {
                log::debug!("Applied free-fly camera mode");
            }
            CameraMode::FirstPerson => {
                log::debug!("Applied first-person camera mode");
            }
            CameraMode::ThirdPerson => {
                log::debug!("Applied third-person camera mode");
            }
            CameraMode::Orbital => {
                log::debug!("Applied orbital camera mode");
            }
            CameraMode::Fixed { position, target } => {
                log::debug!("Applied fixed camera mode at {:?} looking at {:?}", position, target);
            }
        }

        Ok(())
    }

    fn apply_grid_settings(&self, grid_settings: &GridConfig) -> RobinResult<()> {
        log::debug!("Applied grid settings: enabled={}, size={}",
                   grid_settings.enabled, grid_settings.size);
        Ok(())
    }

    fn apply_performance_profile_settings(&self, profile: &PerformanceProfile) -> RobinResult<()> {
        match profile {
            PerformanceProfile::Development => {
                log::debug!("Applied development performance profile");
            }
            PerformanceProfile::Testing => {
                log::debug!("Applied testing performance profile");
            }
            PerformanceProfile::Production => {
                log::debug!("Applied production performance profile");
            }
            PerformanceProfile::Custom(settings) => {
                log::debug!("Applied custom performance profile: target_fps={}", settings.target_fps);
            }
        }

        Ok(())
    }

    fn apply_current_performance_settings(&self) -> RobinResult<()> {
        // TODO: Apply performance settings to engine systems
        Ok(())
    }

    fn auto_save_mode_state(&self) -> RobinResult<()> {
        log::debug!("Auto-saving mode state for {:?}", self.state.current_mode);
        // TODO: Implement auto-save functionality
        Ok(())
    }

    fn is_mode_transition_complete(&self) -> bool {
        self.state.mode_transition_time <= 0.0
    }

    fn get_mode_settings(&self, mode: &BuildModeState) -> Option<&ModeSettings> {
        let mode_key = match mode {
            BuildModeState::Build => "build",
            BuildModeState::Test => "test",
            BuildModeState::Play => "play",
        };
        self.state.mode_specific_settings.get(mode_key)
    }

    fn get_mode_settings_mut(&mut self, mode: &BuildModeState) -> Option<&mut ModeSettings> {
        let mode_key = match mode {
            BuildModeState::Build => "build",
            BuildModeState::Test => "test",
            BuildModeState::Play => "play",
        };
        self.state.mode_specific_settings.get_mut(mode_key)
    }

    pub fn get_performance_stats(&self) -> PerformanceStats {
        self.performance_monitor.get_current_stats()
    }

    pub fn get_optimization_suggestions(&self) -> &[OptimizationSuggestion] {
        &self.performance_monitor.optimization_suggestions
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub avg_frame_time: f32,
    pub min_frame_time: f32,
    pub max_frame_time: f32,
    pub memory_usage_mb: f32,
    pub entity_count: u32,
    pub mode_switch_overhead: f32,
}

impl Default for ModeSystemState {
    fn default() -> Self {
        let mut mode_settings = HashMap::new();

        // Build mode settings
        mode_settings.insert("build".to_string(), ModeSettings {
            enabled_tools: vec![
                "voxel_brush".to_string(),
                "element_placement".to_string(),
                "logic_connector".to_string(),
                "terrain_sculptor".to_string(),
                "material_painter".to_string(),
            ],
            ui_elements: vec![
                "tool_palette".to_string(),
                "properties_panel".to_string(),
                "hierarchy".to_string(),
                "asset_browser".to_string(),
            ],
            input_bindings: HashMap::new(),
            camera_mode: CameraMode::FreeFly,
            grid_settings: GridConfig {
                enabled: true,
                visible: true,
                size: 1.0,
                snap_enabled: true,
                subdivisions: 10,
                color: [0.5, 0.5, 0.5, 0.3],
            },
            debug_overlays: vec![
                DebugOverlay::FPSCounter,
                DebugOverlay::MemoryUsage,
                DebugOverlay::EntityCount,
            ],
            performance_profile: PerformanceProfile::Development,
        });

        // Test mode settings
        mode_settings.insert("test".to_string(), ModeSettings {
            enabled_tools: vec![
                "quick_edit".to_string(),
                "debug_inspector".to_string(),
            ],
            ui_elements: vec![
                "debug_overlay".to_string(),
                "performance_metrics".to_string(),
                "console".to_string(),
            ],
            input_bindings: HashMap::new(),
            camera_mode: CameraMode::FirstPerson,
            grid_settings: GridConfig {
                enabled: false,
                visible: false,
                size: 1.0,
                snap_enabled: false,
                subdivisions: 10,
                color: [0.3, 0.3, 0.3, 0.1],
            },
            debug_overlays: vec![
                DebugOverlay::FPSCounter,
                DebugOverlay::PhysicsDebug,
                DebugOverlay::LogicConnections,
                DebugOverlay::CollisionBounds,
            ],
            performance_profile: PerformanceProfile::Testing,
        });

        // Play mode settings
        mode_settings.insert("play".to_string(), ModeSettings {
            enabled_tools: vec![],
            ui_elements: vec!["game_ui".to_string()],
            input_bindings: HashMap::new(),
            camera_mode: CameraMode::FirstPerson,
            grid_settings: GridConfig {
                enabled: false,
                visible: false,
                size: 1.0,
                snap_enabled: false,
                subdivisions: 10,
                color: [0.0, 0.0, 0.0, 0.0],
            },
            debug_overlays: vec![],
            performance_profile: PerformanceProfile::Production,
        });

        Self {
            current_mode: BuildModeState::Build,
            previous_mode: None,
            mode_transition_time: 0.0,
            transition_duration: 0.5,
            auto_save_on_mode_switch: true,
            preserve_camera_state: true,
            mode_specific_settings: mode_settings,
        }
    }
}

impl Default for ModeTransitionEffects {
    fn default() -> Self {
        Self {
            fade_duration: 0.3,
            current_fade: 0.0,
            fade_direction: FadeDirection::None,
            transition_sound: Some("mode_switch.wav".to_string()),
            visual_effects: vec![
                TransitionEffect::ScreenFade {
                    color: [0.0, 0.0, 0.0, 1.0],
                    intensity: 0.5,
                },
            ],
        }
    }
}

impl ModeTransitionEffects {
    pub fn begin_transition(&mut self) -> RobinResult<()> {
        self.current_fade = 0.0;
        self.fade_direction = FadeDirection::FadeOut;
        log::debug!("Began mode transition effects");
        Ok(())
    }

    pub fn end_transition(&mut self) -> RobinResult<()> {
        self.fade_direction = FadeDirection::FadeIn;
        log::debug!("Ending mode transition effects");
        Ok(())
    }

    pub fn update(&mut self, delta_time: f32, transition_progress: f32) -> RobinResult<()> {
        match self.fade_direction {
            FadeDirection::FadeOut => {
                self.current_fade += delta_time / self.fade_duration;
                self.current_fade = self.current_fade.min(1.0);
            }
            FadeDirection::FadeIn => {
                self.current_fade -= delta_time / self.fade_duration;
                self.current_fade = self.current_fade.max(0.0);

                if self.current_fade <= 0.0 {
                    self.fade_direction = FadeDirection::None;
                }
            }
            FadeDirection::None => {}
        }

        Ok(())
    }

    pub fn get_fade_intensity(&self) -> f32 {
        self.current_fade
    }
}

impl ModeUIManager {
    pub fn new() -> Self {
        Self {
            build_ui: BuildModeUI::default(),
            test_ui: TestModeUI::default(),
            play_ui: PlayModeUI::default(),
            transition_ui: TransitionUI::default(),
            current_ui_state: UIState::Visible,
        }
    }

    pub fn update(&mut self, delta_time: f32, current_mode: &BuildModeState) -> RobinResult<()> {
        match current_mode {
            BuildModeState::Build => self.build_ui.update(delta_time)?,
            BuildModeState::Test => self.test_ui.update(delta_time)?,
            BuildModeState::Play => self.play_ui.update(delta_time)?,
        }
        Ok(())
    }

    pub fn prepare_mode_transition(&mut self, new_mode: BuildModeState) -> RobinResult<()> {
        self.current_ui_state = UIState::Transitioning;
        self.transition_ui.prepare_for_mode(new_mode)?;
        log::debug!("Prepared UI for transition to {:?}", new_mode);
        Ok(())
    }

    pub fn complete_mode_transition(&mut self) -> RobinResult<()> {
        self.current_ui_state = UIState::Visible;
        log::debug!("Completed UI mode transition");
        Ok(())
    }
}

impl Default for BuildModeUI {
    fn default() -> Self {
        Self {
            tool_palette_visible: true,
            properties_panel_visible: true,
            hierarchy_panel_visible: true,
            asset_browser_visible: false,
            debug_console_visible: false,
            grid_controls_visible: true,
            quick_actions_visible: true,
        }
    }
}

impl BuildModeUI {
    pub fn update(&mut self, delta_time: f32) -> RobinResult<()> {
        // TODO: Update build mode UI elements
        Ok(())
    }
}

impl Default for TestModeUI {
    fn default() -> Self {
        Self {
            debug_overlay_visible: true,
            performance_metrics_visible: true,
            entity_inspector_visible: false,
            console_visible: true,
            quick_edit_enabled: true,
            pause_button_visible: true,
            reset_button_visible: true,
        }
    }
}

impl TestModeUI {
    pub fn update(&mut self, delta_time: f32) -> RobinResult<()> {
        // TODO: Update test mode UI elements
        Ok(())
    }
}

impl Default for PlayModeUI {
    fn default() -> Self {
        Self {
            game_ui_visible: true,
            settings_accessible: true,
            pause_menu_enabled: true,
            developer_console_hidden: true,
            immersive_mode: false,
        }
    }
}

impl PlayModeUI {
    pub fn update(&mut self, delta_time: f32) -> RobinResult<()> {
        // TODO: Update play mode UI elements
        Ok(())
    }
}

impl Default for TransitionUI {
    fn default() -> Self {
        Self {
            progress_bar_visible: true,
            mode_name_display: true,
            instruction_text_visible: true,
            loading_animation_type: LoadingAnimation::Fade,
        }
    }
}

impl TransitionUI {
    pub fn prepare_for_mode(&mut self, mode: BuildModeState) -> RobinResult<()> {
        log::debug!("Preparing transition UI for {:?} mode", mode);
        Ok(())
    }
}

impl ModeInputHandler {
    pub fn new() -> Self {
        Self {
            build_mode_bindings: HashMap::new(),
            test_mode_bindings: HashMap::new(),
            play_mode_bindings: HashMap::new(),
            global_bindings: HashMap::new(),
            input_context_stack: Vec::new(),
        }
    }

    pub fn update(&mut self, input: &InputManager, current_mode: &BuildModeState) -> RobinResult<()> {
        // Process input based on current mode and context stack
        match current_mode {
            BuildModeState::Build => self.process_build_mode_input(input)?,
            BuildModeState::Test => self.process_test_mode_input(input)?,
            BuildModeState::Play => self.process_play_mode_input(input)?,
        }

        // Always process global bindings
        self.process_global_input(input)?;

        Ok(())
    }

    fn process_build_mode_input(&self, input: &InputManager) -> RobinResult<()> {
        // TODO: Process build mode specific input
        Ok(())
    }

    fn process_test_mode_input(&self, input: &InputManager) -> RobinResult<()> {
        // TODO: Process test mode specific input
        Ok(())
    }

    fn process_play_mode_input(&self, input: &InputManager) -> RobinResult<()> {
        // TODO: Process play mode specific input
        Ok(())
    }

    fn process_global_input(&self, input: &InputManager) -> RobinResult<()> {
        // TODO: Process global input bindings
        Ok(())
    }
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            frame_times: Vec::with_capacity(120), // 2 seconds at 60fps
            memory_usage: Vec::with_capacity(60), // 1 minute at 1Hz
            entity_counts: Vec::with_capacity(60),
            mode_switch_times: HashMap::new(),
            optimization_suggestions: Vec::new(),
        }
    }

    pub fn update(&mut self, delta_time: f32, current_mode: &BuildModeState) -> RobinResult<()> {
        // Record frame time
        self.frame_times.push(delta_time);
        if self.frame_times.len() > 120 {
            self.frame_times.remove(0);
        }

        // TODO: Record other performance metrics
        // TODO: Generate optimization suggestions based on performance data

        Ok(())
    }

    pub fn get_current_stats(&self) -> PerformanceStats {
        let avg_frame_time = if !self.frame_times.is_empty() {
            self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32
        } else {
            0.016 // 60fps default
        };

        let min_frame_time = self.frame_times.iter().cloned().fold(f32::INFINITY, f32::min);
        let max_frame_time = self.frame_times.iter().cloned().fold(0.0, f32::max);

        PerformanceStats {
            avg_frame_time,
            min_frame_time: if min_frame_time == f32::INFINITY { 0.0 } else { min_frame_time },
            max_frame_time,
            memory_usage_mb: 0.0, // TODO: Get actual memory usage
            entity_count: 0,      // TODO: Get actual entity count
            mode_switch_overhead: 0.0, // TODO: Calculate mode switch overhead
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_system_creation() {
        let mode_system = ModeSystem::new();
        assert_eq!(mode_system.get_current_mode(), BuildModeState::Build);
        assert!(!mode_system.is_transitioning());
    }

    #[test]
    fn test_mode_cycling() {
        let mut mode_system = ModeSystem::new();

        mode_system.cycle_mode().unwrap();
        assert_eq!(mode_system.get_current_mode(), BuildModeState::Test);

        mode_system.cycle_mode().unwrap();
        assert_eq!(mode_system.get_current_mode(), BuildModeState::Play);

        mode_system.cycle_mode().unwrap();
        assert_eq!(mode_system.get_current_mode(), BuildModeState::Build);
    }

    #[test]
    fn test_tool_availability_by_mode() {
        let mode_system = ModeSystem::new();

        // In build mode, should have building tools
        assert!(mode_system.is_tool_enabled("voxel_brush"));
        assert!(mode_system.is_tool_enabled("element_placement"));

        // Create a test mode system
        let mut test_mode_system = ModeSystem::new();
        test_mode_system.switch_mode(BuildModeState::Play).unwrap();

        // In play mode, should have no tools enabled by default
        assert!(test_mode_system.get_enabled_tools().is_empty());
    }
}