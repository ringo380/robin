// Unified HUD System - Integrates ImGui and Production UI
// Provides seamless integration between demo ImGui overlays and engine production UI components

use imgui::*;
use core_graphics::geometry::CGSize;
use robin::engine::generation::voxel_system::VoxelType;
use robin::engine::ui::{GameHUDSystem, HUDAction, PerformanceMetrics, BuildState, GameState};
use robin::engine::ui::{MainMenuSystem, MenuAction, SettingsMenuSystem, SettingsAction};
use robin::engine::character::CharacterState;
use crate::renderer::Camera;
use crate::ui::{UIAction, InventoryPanel, BuildToolsPanel, MiniMapPanel, CommunityPanel};

#[derive(Debug, Clone)]
pub enum UnifiedUIAction {
    // ImGui Demo Actions
    Demo(UIAction),
    // Production UI Actions
    HUD(HUDAction),
    Menu(MenuAction),
    Settings(SettingsAction),
    // Unified Actions
    ToggleUI,
    ToggleProductionMode,
    SwitchUISystem,
}

pub struct UnifiedHUDSystem {
    // ImGui context and panels
    imgui_context: Context,
    inventory_panel: InventoryPanel,
    build_tools_panel: BuildToolsPanel,
    mini_map_panel: MiniMapPanel,
    community_panel: CommunityPanel,

    // Production UI systems
    production_hud: GameHUDSystem,
    main_menu: MainMenuSystem,
    settings_menu: SettingsMenuSystem,

    // State management
    use_production_ui: bool,
    show_imgui_overlay: bool,
    show_production_hud: bool,
    show_performance_metrics: bool,
    show_debug_info: bool,

    // Font and rendering
    font_texture_id: Option<TextureId>,
    font_texture_data: Option<Vec<u8>>,
    font_texture_width: u32,
    font_texture_height: u32,

    // UI State
    player_health: f32,
    player_energy: f32,
    fps: f32,
    frame_time: f32,
    voxel_count: u32,
    current_tool: String,
}

impl UnifiedHUDSystem {
    pub fn new() -> Self {
        let mut imgui_context = Context::create();
        imgui_context.set_ini_filename(None);

        // Build font atlas
        let (font_texture_data, font_texture_width, font_texture_height) = Self::build_font_atlas(&mut imgui_context);

        // Configure ImGui style for integration
        Self::setup_unified_style(&mut imgui_context);

        Self {
            imgui_context,
            inventory_panel: InventoryPanel::new(),
            build_tools_panel: BuildToolsPanel::new(),
            mini_map_panel: MiniMapPanel::new(),
            community_panel: CommunityPanel::new(),

            production_hud: GameHUDSystem::new(),
            main_menu: MainMenuSystem::new(),
            settings_menu: SettingsMenuSystem::new(),

            use_production_ui: false, // Start with ImGui for compatibility
            show_imgui_overlay: true,
            show_production_hud: false,
            show_performance_metrics: true,
            show_debug_info: false,

            font_texture_id: None,
            font_texture_data: Some(font_texture_data),
            font_texture_width,
            font_texture_height,

            player_health: 100.0,
            player_energy: 100.0,
            fps: 60.0,
            frame_time: 16.67,
            voxel_count: 0,
            current_tool: "Build".to_string(),
        }
    }

    fn build_font_atlas(context: &mut Context) -> (Vec<u8>, u32, u32) {
        let fonts = context.fonts();
        fonts.add_font(&[FontSource::DefaultFontData { config: None }]);
        let font_atlas = fonts.build_rgba32_texture();
        let texture_data = font_atlas.data.to_vec();
        (texture_data, font_atlas.width, font_atlas.height)
    }

    fn setup_unified_style(context: &mut Context) {
        let style = context.style_mut();

        // Professional dark theme with robin blue accents
        style.colors[StyleColor::Text as usize] = [0.95, 0.95, 0.95, 1.00];
        style.colors[StyleColor::WindowBg as usize] = [0.08, 0.08, 0.10, 0.90];
        style.colors[StyleColor::FrameBg as usize] = [0.15, 0.15, 0.18, 0.90];
        style.colors[StyleColor::FrameBgHovered as usize] = [0.20, 0.35, 0.55, 0.80];
        style.colors[StyleColor::FrameBgActive as usize] = [0.25, 0.40, 0.65, 0.90];
        style.colors[StyleColor::TitleBg as usize] = [0.10, 0.10, 0.12, 1.00];
        style.colors[StyleColor::TitleBgActive as usize] = [0.15, 0.30, 0.50, 1.00];
        style.colors[StyleColor::Button as usize] = [0.20, 0.35, 0.55, 0.80];
        style.colors[StyleColor::ButtonHovered as usize] = [0.25, 0.40, 0.65, 0.90];
        style.colors[StyleColor::ButtonActive as usize] = [0.30, 0.45, 0.70, 1.00];

        // Rounded corners and padding
        style.window_rounding = 8.0;
        style.frame_rounding = 4.0;
        style.grab_rounding = 4.0;
        style.window_padding = [12.0, 8.0];
        style.frame_padding = [8.0, 4.0];
        style.item_spacing = [8.0, 4.0];
    }

    pub fn update_and_render(
        &mut self,
        window_size: CGSize,
        build_system: &mut impl std::fmt::Debug,
        camera: &Camera,
        character_state: &CharacterState,
        delta_time: f32,
        time_of_day: f32,
    ) -> (Vec<UnifiedUIAction>, Option<&imgui::DrawData>) {
        let mut actions = Vec::new();

        // Update performance metrics
        self.fps = 1.0 / delta_time.max(0.001);
        self.frame_time = delta_time * 1000.0;

        // Update player stats from character state
        self.player_health = if character_state.is_grounded { 100.0 } else { 95.0 }; // Example health logic
        self.player_energy = 100.0 - (character_state.velocity.magnitude() * 10.0).min(50.0); // Energy based on movement

        if self.use_production_ui {
            // Use production UI system
            actions.extend(self.render_production_ui(window_size, build_system, camera, character_state, delta_time, time_of_day));
            (actions, None)
        } else {
            // Use ImGui system with unified HUD overlay
            let draw_data = self.render_imgui_ui(window_size, build_system, camera, character_state, delta_time, time_of_day, &mut actions);
            (actions, draw_data)
        }
    }

    fn render_production_ui(
        &mut self,
        window_size: CGSize,
        _build_system: &mut impl std::fmt::Debug,
        camera: &Camera,
        character_state: &CharacterState,
        delta_time: f32,
        _time_of_day: f32,
    ) -> Vec<UnifiedUIAction> {
        let mut actions = Vec::new();

        // Create performance metrics for production UI
        let performance_metrics = PerformanceMetrics {
            fps: self.fps,
            frame_time: self.frame_time,
            memory_usage: 0.0, // Would come from system monitoring
            gpu_usage: 0.0,
            draw_calls: 0,
            vertices: self.voxel_count,
        };

        // Create build state
        let build_state = BuildState {
            current_tool: self.current_tool.clone(),
            material_count: 100,
            available_materials: vec!["Stone".to_string(), "Wood".to_string(), "Metal".to_string()],
            build_mode_active: true,
        };

        // Create game state
        let game_state = GameState {
            player_health: self.player_health,
            player_energy: self.player_energy,
            position: [character_state.position.x, character_state.position.y, character_state.position.z],
            camera_rotation: [camera.yaw, camera.pitch, 0.0],
            time_of_day: 12.0, // Example time
            weather: "Clear".to_string(),
        };

        // Render production HUD
        if let Ok(hud_actions) = self.production_hud.render(
            window_size.width as f32,
            window_size.height as f32,
            delta_time,
            &performance_metrics,
            &build_state,
            &game_state
        ) {
            for action in hud_actions {
                actions.push(UnifiedUIAction::HUD(action));
            }
        }

        actions
    }

    fn render_imgui_ui(
        &mut self,
        window_size: CGSize,
        build_system: &mut impl std::fmt::Debug,
        camera: &Camera,
        character_state: &CharacterState,
        delta_time: f32,
        time_of_day: f32,
        actions: &mut Vec<UnifiedUIAction>,
    ) -> Option<&imgui::DrawData> {
        let io = self.imgui_context.io_mut();
        io.display_size = [window_size.width as f32, window_size.height as f32];
        io.delta_time = delta_time;

        let ui = self.imgui_context.frame();

        if self.show_imgui_overlay {
            // Render unified HUD overlay
            self.render_unified_hud_overlay(&ui, character_state, camera, time_of_day, actions);

            // Render existing ImGui panels
            if self.show_debug_info {
                let demo_actions = self.inventory_panel.render(&ui, &build_system);
                actions.extend(demo_actions.into_iter().map(UnifiedUIAction::Demo));

                let tool_actions = self.build_tools_panel.render(&ui, &build_system);
                actions.extend(tool_actions.into_iter().map(UnifiedUIAction::Demo));

                let map_actions = self.mini_map_panel.render(&ui, camera);
                actions.extend(map_actions.into_iter().map(UnifiedUIAction::Demo));

                let community_actions = self.community_panel.render(&ui);
                actions.extend(community_actions.into_iter().map(|_| UnifiedUIAction::ToggleUI)); // Convert community actions
            }

            // UI mode switcher
            self.render_ui_mode_switcher(&ui, actions);
        }

        ui.render()
    }

    fn render_unified_hud_overlay(
        &self,
        ui: &Ui,
        character_state: &CharacterState,
        camera: &Camera,
        time_of_day: f32,
        actions: &mut Vec<UnifiedUIAction>,
    ) {
        // Top-left: Health and Energy
        ui.window("##health_energy")
            .size([200.0, 80.0], Condition::Always)
            .position([10.0, 10.0], Condition::Always)
            .no_decoration()
            .no_background()
            .build(|| {
                ui.text("❤️ Health");
                ProgressBar::new(self.player_health / 100.0)
                    .size([180.0, 12.0])
                    .overlay_text(&format!("{:.0}/100", self.player_health))
                    .build(ui);

                ui.text("⚡ Energy");
                ProgressBar::new(self.player_energy / 100.0)
                    .size([180.0, 12.0])
                    .overlay_text(&format!("{:.0}/100", self.player_energy))
                    .build(ui);
            });

        // Top-right: Performance metrics
        if self.show_performance_metrics {
            ui.window("##performance")
                .size([150.0, 100.0], Condition::Always)
                .position([ui.io().display_size[0] - 160.0, 10.0], Condition::Always)
                .no_decoration()
                .no_background()
                .build(|| {
                    ui.text(format!("FPS: {:.1}", self.fps));
                    ui.text(format!("Frame: {:.1}ms", self.frame_time));
                    ui.text(format!("Pos: ({:.1}, {:.1}, {:.1})",
                        character_state.position.x,
                        character_state.position.y,
                        character_state.position.z));
                    ui.text(format!("Look: {:.1}°, {:.1}°", camera.yaw.to_degrees(), camera.pitch.to_degrees()));
                });
        }

        // Bottom-center: Tool and time info
        let window_width = ui.io().display_size[0];
        ui.window("##tool_time")
            .size([200.0, 60.0], Condition::Always)
            .position([window_width / 2.0 - 100.0, ui.io().display_size[1] - 70.0], Condition::Always)
            .no_decoration()
            .no_background()
            .build(|| {
                ui.text_centered(&format!("🔧 {}", self.current_tool));
                ui.text_centered(&format!("🌅 Time: {:.1}h", time_of_day));
                if character_state.is_grounded {
                    ui.text_colored([0.0, 1.0, 0.0, 1.0], "● Grounded");
                } else {
                    ui.text_colored([1.0, 1.0, 0.0, 1.0], "● Airborne");
                }
            });

        // Crosshair in center
        let center_x = window_width / 2.0;
        let center_y = ui.io().display_size[1] / 2.0;
        let draw_list = ui.get_window_draw_list();
        draw_list.add_line(
            [center_x - 10.0, center_y],
            [center_x + 10.0, center_y],
            [1.0, 1.0, 1.0, 0.8]
        ).thickness(2.0).build();
        draw_list.add_line(
            [center_x, center_y - 10.0],
            [center_x, center_y + 10.0],
            [1.0, 1.0, 1.0, 0.8]
        ).thickness(2.0).build();
    }

    fn render_ui_mode_switcher(&self, ui: &Ui, actions: &mut Vec<UnifiedUIAction>) {
        ui.window("##ui_switcher")
            .size([200.0, 120.0], Condition::Always)
            .position([10.0, ui.io().display_size[1] - 130.0], Condition::Always)
            .title_bar(false)
            .resizable(false)
            .build(|| {
                ui.text("UI System:");

                if ui.radio_button("ImGui Demo", !self.use_production_ui) {
                    actions.push(UnifiedUIAction::SwitchUISystem);
                }
                if ui.radio_button("Production UI", self.use_production_ui) {
                    actions.push(UnifiedUIAction::SwitchUISystem);
                }

                ui.separator();

                if ui.button("Toggle Debug") {
                    actions.push(UnifiedUIAction::ToggleUI);
                }

                if ui.button("Performance") {
                    actions.push(UnifiedUIAction::ToggleProductionMode);
                }
            });
    }

    pub fn handle_action(&mut self, action: UnifiedUIAction) {
        match action {
            UnifiedUIAction::ToggleUI => {
                self.show_debug_info = !self.show_debug_info;
            }
            UnifiedUIAction::ToggleProductionMode => {
                self.show_performance_metrics = !self.show_performance_metrics;
            }
            UnifiedUIAction::SwitchUISystem => {
                self.use_production_ui = !self.use_production_ui;
                println!("🔄 Switched to {} UI system",
                    if self.use_production_ui { "Production" } else { "ImGui Demo" });
            }
            UnifiedUIAction::Demo(demo_action) => {
                match demo_action {
                    UIAction::ToggleBuildMode => {
                        self.current_tool = if self.current_tool == "Build" { "Mine" } else { "Build" }.to_string();
                    }
                    _ => {} // Handle other demo actions as needed
                }
            }
            UnifiedUIAction::HUD(_) | UnifiedUIAction::Menu(_) | UnifiedUIAction::Settings(_) => {
                // Production UI actions would be handled by their respective systems
            }
        }
    }

    // Font texture management
    pub fn get_font_texture_data(&self) -> Option<&Vec<u8>> {
        self.font_texture_data.as_ref()
    }

    pub fn get_font_texture_dimensions(&self) -> (u32, u32) {
        (self.font_texture_width, self.font_texture_height)
    }

    pub fn set_font_texture_id(&mut self, texture_id: TextureId) {
        self.font_texture_id = Some(texture_id);
        self.imgui_context.fonts().tex_id = texture_id;
    }

    // Utility methods
    pub fn is_using_production_ui(&self) -> bool {
        self.use_production_ui
    }

    pub fn update_voxel_count(&mut self, count: u32) {
        self.voxel_count = count;
    }

    pub fn set_current_tool(&mut self, tool: String) {
        self.current_tool = tool;
    }
}

impl Default for UnifiedHUDSystem {
    fn default() -> Self {
        Self::new()
    }
}