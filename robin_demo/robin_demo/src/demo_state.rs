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
use std::time::Instant;

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

/// Coordinates all demo systems and UI layers
pub struct DemoStateManager {
    /// Current demo mode
    current_mode: DemoMode,
    /// Existing ImGui UI system (proven and working)
    imgui_system: SimpleUISystem,
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
}

impl DemoStateManager {
    pub fn new() -> Self {
        let now = Instant::now();

        Self {
            current_mode: DemoMode::InteractivePlayground,
            imgui_system: SimpleUISystem::new(),
            production_ui: UIManager::new(1920.0, 1080.0), // Default screen size
            gameplay: GameplayManager::new(),
            input_manager: InputManager::new(), // Input manager doesn't need screen size
            session_start: now,
            mode_switch_time: now,
            show_production_ui: true,
            show_imgui_ui: true,
        }
    }

    /// Get current demo mode
    pub fn get_current_mode(&self) -> &DemoMode {
        &self.current_mode
    }

    /// Switch to a different demo mode
    pub fn switch_mode(&mut self, new_mode: DemoMode) {
        if new_mode != self.current_mode {
            self.current_mode = new_mode;
            self.mode_switch_time = Instant::now();

            // Configure UI visibility based on mode
            match &self.current_mode {
                DemoMode::InteractivePlayground => {
                    self.show_production_ui = true;
                    self.show_imgui_ui = true;
                    self.production_ui.set_ui_mode(UIMode::InGame);
                }
                DemoMode::EngineerBuildShowcase => {
                    self.show_production_ui = true;
                    self.show_imgui_ui = true;
                    self.production_ui.set_ui_mode(UIMode::InGame);
                }
                DemoMode::GameplaySystemsDemo => {
                    self.show_production_ui = true;
                    self.show_imgui_ui = false; // Focus on production UI
                    self.production_ui.set_ui_mode(UIMode::InGame);
                }
                DemoMode::CollaborationPreview => {
                    self.show_production_ui = true;
                    self.show_imgui_ui = false;
                    self.production_ui.set_ui_mode(UIMode::InGame);
                }
                DemoMode::PerformanceBenchmarks => {
                    self.show_production_ui = true;
                    self.show_imgui_ui = true;
                    self.production_ui.set_ui_mode(UIMode::InGame);
                }
                DemoMode::VisualShowcase => {
                    self.show_production_ui = false; // Minimal UI for visual focus
                    self.show_imgui_ui = false;
                    self.production_ui.set_ui_mode(UIMode::InGame);
                }
            }
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
    ) -> RobinResult<(Vec<DemoUIAction>, Vec<EngineUIAction>, Option<imgui::DrawData>)> {

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
            use crate::renderer::CGSize;
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

    /// Get access to production UI for external operations
    pub fn get_production_ui(&self) -> &UIManager {
        &self.production_ui
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