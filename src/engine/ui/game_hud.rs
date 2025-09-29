/*!
 * Production Game HUD System
 *
 * Comprehensive heads-up display for Robin Engine's 3D voxel building experience.
 * Includes performance monitoring, build tools, and accessibility features.
 */

use crate::engine::{
    ui::{
        production_theme_simple::ProductionDarkTheme,
        modern_components::{ModernButton, ModernCard, AccessibilityProps},
        css_in_rust::Style,
        UIBounds, UIState, UIElement,
    },
    input::InputManager,
    math::Vec3,
    error::RobinResult,
    build_mode::{BuildMode, TemplateType},
    world::VoxelType,
};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// HUD panel visibility states
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HUDPanel {
    PerformanceStats,
    BuildTools,
    MaterialPalette,
    QuickActions,
    Navigation,
    Help,
}

/// HUD action messages sent to the game engine
#[derive(Debug, Clone)]
pub enum HUDAction {
    // Build actions
    SelectMaterial(VoxelType),
    SetBuildMode(BuildMode),
    SelectTemplate(TemplateType),
    RotateTemplate,

    // Quick actions
    Save,
    Load,
    Undo,
    Redo,

    // UI actions
    TogglePanel(HUDPanel),
    ShowSettings,
    ShowHelp,
    ToggleFullscreen,

    // Game state
    Pause,
    PauseGame,
    Resume,
    ToggleBuildMode,
    Quit,
}

/// Performance metrics for display
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub fps: f32,
    pub frame_time_ms: f32,
    pub vertex_count: u32,
    pub draw_calls: u32,
    pub memory_usage_mb: f32,
    pub gpu_usage: f32,
    pub world_chunks_loaded: u32,
    pub particles_active: u32,
}

/// Build state information
#[derive(Debug, Clone)]
pub struct BuildState {
    pub current_mode: BuildMode,
    pub selected_material: VoxelType,
    pub selected_template: Option<TemplateType>,
    pub template_rotation: u32,
    pub can_undo: bool,
    pub can_redo: bool,
    pub grid_snap_enabled: bool,
}

/// Game state information
#[derive(Debug, Clone)]
pub struct GameState {
    pub player_position: Vec3,
    pub camera_direction: Vec3,
    pub world_time: f32,
    pub day_night_cycle: String,
    pub paused: bool,
    pub save_available: bool,
}

/// Main HUD system manager
pub struct GameHUDSystem {
    theme: ProductionDarkTheme,
    visible_panels: HashMap<HUDPanel, bool>,

    // UI components
    buttons: HashMap<String, ModernButton>,
    cards: HashMap<String, ModernCard>,

    // State
    performance: PerformanceMetrics,
    build_state: BuildState,
    game_state: GameState,

    // Interaction
    selected_button: Option<String>,
    material_grid_selection: (usize, usize),

    // Animation and timing
    animation_time: f32,
    last_performance_update: Instant,

    // Layout
    screen_width: f32,
    screen_height: f32,
    styles: HashMap<String, Style>,
}

impl GameHUDSystem {
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        let theme = ProductionDarkTheme::new();
        let styles = theme.create_hud_styles();

        let mut system = Self {
            theme,
            visible_panels: HashMap::new(),
            buttons: HashMap::new(),
            cards: HashMap::new(),
            performance: PerformanceMetrics::default(),
            build_state: BuildState::default(),
            game_state: GameState::default(),
            selected_button: None,
            material_grid_selection: (0, 0),
            animation_time: 0.0,
            last_performance_update: Instant::now(),
            screen_width,
            screen_height,
            styles,
        };

        system.initialize_hud();
        system
    }

    /// Initialize all HUD components
    fn initialize_hud(&mut self) {
        // Set default panel visibility
        self.visible_panels.insert(HUDPanel::PerformanceStats, true);
        self.visible_panels.insert(HUDPanel::BuildTools, true);
        self.visible_panels.insert(HUDPanel::MaterialPalette, true);
        self.visible_panels.insert(HUDPanel::QuickActions, true);
        self.visible_panels.insert(HUDPanel::Navigation, false);
        self.visible_panels.insert(HUDPanel::Help, false);

        self.create_performance_panel();
        self.create_build_tools_panel();
        self.create_material_palette();
        self.create_quick_actions_panel();
        self.create_navigation_panel();
    }

    /// Create performance monitoring panel
    fn create_performance_panel(&mut self) {
        let panel_bounds = UIBounds::new(
            self.screen_width - 220.0,  // Top-right corner
            10.0,
            200.0,
            140.0,
        );

        let performance_card = ModernCard::new(1001, panel_bounds)
            .with_title("Performance".to_string());

        self.cards.insert("performance_panel".to_string(), performance_card);

        // Toggle button for performance panel
        let toggle_bounds = UIBounds::new(self.screen_width - 40.0, 10.0, 30.0, 30.0);
        let toggle_button = ModernButton::ghost()
            .with_text("📊".to_string())
            .with_accessibility(AccessibilityProps {
                aria_label: Some("Toggle Performance Panel".to_string()),
                role: "button".to_string(),
                tab_index: 1,
                keyboard_shortcuts: vec!["F3".to_string()],
                ..Default::default()
            });

        self.buttons.insert("toggle_performance".to_string(), toggle_button);
    }

    /// Create build tools panel
    fn create_build_tools_panel(&mut self) {
        let panel_bounds = UIBounds::new(
            10.0,  // Left side
            self.screen_height / 2.0 - 150.0,  // Centered vertically
            280.0,
            300.0,
        );

        let build_tools_card = ModernCard::new(1002, panel_bounds)
            .with_title("Build Tools".to_string());

        self.cards.insert("build_tools_panel".to_string(), build_tools_card);

        // Build mode buttons
        let build_modes = vec![
            ("single", "Single", BuildMode::Single, "Place individual blocks"),
            ("wall", "Wall", BuildMode::Wall, "Build walls quickly"),
            ("floor", "Floor", BuildMode::Floor, "Create floor surfaces"),
            ("template", "Template", BuildMode::Template, "Use building templates"),
        ];

        for (i, (id, text, mode, description)) in build_modes.iter().enumerate() {
            let button_bounds = UIBounds::new(
                30.0,
                self.screen_height / 2.0 - 120.0 + i as f32 * 40.0,
                100.0,
                35.0,
            );

            let accessibility = AccessibilityProps {
                aria_label: Some(format!("{} Build Mode", text)),
                aria_description: Some(description.to_string()),
                role: "button".to_string(),
                tab_index: 10 + i as i32,
                keyboard_shortcuts: vec![format!("{}", i + 1)],
                ..Default::default()
            };

            let button = if *mode == self.build_state.current_mode {
                ModernButton::primary()
            } else {
                ModernButton::secondary()
            }.with_text(text.to_string())
             .with_accessibility(accessibility);

            self.buttons.insert(format!("build_mode_{}", id), button);
        }
    }

    /// Create material selection palette
    fn create_material_palette(&mut self) {
        let materials = vec![
            (VoxelType::Stone, "🪨", "Stone"),
            (VoxelType::Dirt, "🟤", "Dirt"),
            (VoxelType::Grass, "🟢", "Grass"),
            (VoxelType::Wood, "🟫", "Wood"),
            (VoxelType::Glass, "🔷", "Glass"),
            (VoxelType::Metal, "⚪", "Metal"),
            (VoxelType::Water, "🟦", "Water"),
            (VoxelType::Sand, "🟨", "Sand"),
        ];

        for (i, (material, icon, name)) in materials.iter().enumerate() {
            let row = i / 4;
            let col = i % 4;

            let button_bounds = UIBounds::new(
                30.0 + col as f32 * 60.0,
                self.screen_height / 2.0 + 50.0 + row as f32 * 60.0,
                50.0,
                50.0,
            );

            let accessibility = AccessibilityProps {
                aria_label: Some(format!("{} Material", name)),
                role: "button".to_string(),
                tab_index: 20 + i as i32,
                keyboard_shortcuts: vec![format!("{}", i + 1)],
                ..Default::default()
            };

            let button = if *material == self.build_state.selected_material {
                ModernButton::primary()
            } else {
                ModernButton::secondary()
            }.with_text(format!("{}", icon))
             .with_accessibility(accessibility);

            self.buttons.insert(format!("material_{:?}", material), button);
        }
    }

    /// Create quick actions panel
    fn create_quick_actions_panel(&mut self) {
        let actions = vec![
            ("save", "💾", "Save", "Ctrl+S"),
            ("undo", "↶", "Undo", "Ctrl+Z"),
            ("redo", "↷", "Redo", "Ctrl+Y"),
            ("help", "❓", "Help", "H"),
        ];

        for (i, (id, icon, name, shortcut)) in actions.iter().enumerate() {
            let button_bounds = UIBounds::new(
                10.0 + i as f32 * 50.0,
                10.0,
                40.0,
                40.0,
            );

            let accessibility = AccessibilityProps {
                aria_label: Some(format!("{} ({})", name, shortcut)),
                role: "button".to_string(),
                tab_index: 30 + i as i32,
                keyboard_shortcuts: vec![shortcut.to_string()],
                ..Default::default()
            };

            let mut button = ModernButton::ghost()
                .with_text(icon.to_string())
                .with_accessibility(accessibility);

            // Disable undo/redo based on state
            if *id == "undo" && !self.build_state.can_undo {
                button.set_enabled(false);
            } else if *id == "redo" && !self.build_state.can_redo {
                button.set_enabled(false);
            }

            self.buttons.insert(format!("action_{}", id), button);
        }
    }

    /// Create navigation panel (coordinates, minimap)
    fn create_navigation_panel(&mut self) {
        let panel_bounds = UIBounds::new(
            self.screen_width - 220.0,
            self.screen_height - 160.0,
            200.0,
            140.0,
        );

        let nav_card = ModernCard::new(1003, panel_bounds)
            .with_title("Navigation".to_string());

        self.cards.insert("navigation_panel".to_string(), nav_card);
    }

    /// Update HUD system
    pub fn update(&mut self, delta_time: f32, input: &InputManager) -> RobinResult<Vec<HUDAction>> {
        let mut actions = Vec::new();

        self.animation_time += delta_time;

        // Update performance metrics periodically
        if self.last_performance_update.elapsed() > Duration::from_millis(500) {
            self.update_performance_metrics();
            self.last_performance_update = Instant::now();
        }

        // Handle keyboard shortcuts
        self.handle_keyboard_shortcuts(input, &mut actions);

        // Update all UI components
        for button in self.buttons.values_mut() {
            button.update(delta_time, input);
        }

        // Check for button interactions
        self.check_button_interactions(&mut actions);

        Ok(actions)
    }

    /// Handle global keyboard shortcuts
    fn handle_keyboard_shortcuts(&mut self, input: &InputManager, actions: &mut Vec<HUDAction>) {
        // Toggle panels
        if input.is_named_key_just_pressed(winit::keyboard::NamedKey::F3) {
            self.toggle_panel(HUDPanel::PerformanceStats);
        }

        if input.is_key_just_pressed(&winit::keyboard::Key::Character("h".into())) {
            actions.push(HUDAction::ShowHelp);
        }

        // Build mode shortcuts (1-4)
        for i in 1..=4 {
            if input.is_key_just_pressed(&winit::keyboard::Key::Character(i.to_string().into())) {
                let mode = match i {
                    1 => BuildMode::Single,
                    2 => BuildMode::Wall,
                    3 => BuildMode::Floor,
                    4 => BuildMode::Template,
                    _ => BuildMode::Single,
                };
                actions.push(HUDAction::SetBuildMode(mode));
            }
        }

        // Material shortcuts (Q, W, E, R, T, Y, U, I)
        let material_keys = ["q", "w", "e", "r", "t", "y", "u", "i"];
        let materials = [
            VoxelType::Stone, VoxelType::Dirt, VoxelType::Grass, VoxelType::Wood,
            VoxelType::Glass, VoxelType::Metal, VoxelType::Water, VoxelType::Sand,
        ];

        for (i, key) in material_keys.iter().enumerate() {
            if input.is_key_just_pressed(&winit::keyboard::Key::Character((*key).into())) {
                if let Some(material) = materials.get(i) {
                    actions.push(HUDAction::SelectMaterial(*material));
                }
            }
        }

        // Quick actions
        if input.is_ctrl_pressed() {
            if input.is_key_just_pressed(&winit::keyboard::Key::Character("s".into())) {
                actions.push(HUDAction::Save);
            } else if input.is_key_just_pressed(&winit::keyboard::Key::Character("z".into())) {
                actions.push(HUDAction::Undo);
            } else if input.is_key_just_pressed(&winit::keyboard::Key::Character("y".into())) {
                actions.push(HUDAction::Redo);
            }
        }
    }

    /// Check for button interactions and generate actions
    fn check_button_interactions(&mut self, actions: &mut Vec<HUDAction>) {
        for (button_id, button) in &mut self.buttons {
            if button.get_state() == UIState::Pressed {
                // Parse button action from ID
                if button_id.starts_with("build_mode_") {
                    let mode_str = button_id.strip_prefix("build_mode_").unwrap();
                    let mode = match mode_str {
                        "single" => BuildMode::Single,
                        "wall" => BuildMode::Wall,
                        "floor" => BuildMode::Floor,
                        "template" => BuildMode::Template,
                        _ => BuildMode::Single,
                    };
                    actions.push(HUDAction::SetBuildMode(mode));
                }
                else if button_id.starts_with("material_") {
                    if let Some(material_str) = button_id.strip_prefix("material_") {
                        // Parse VoxelType from string
                        let material = match material_str {
                            "Stone" => VoxelType::Stone,
                            "Dirt" => VoxelType::Dirt,
                            "Grass" => VoxelType::Grass,
                            "Wood" => VoxelType::Wood,
                            "Glass" => VoxelType::Glass,
                            "Metal" => VoxelType::Metal,
                            "Water" => VoxelType::Water,
                            "Sand" => VoxelType::Sand,
                            _ => VoxelType::Stone,
                        };
                        actions.push(HUDAction::SelectMaterial(material));
                    }
                }
                else if button_id.starts_with("action_") {
                    let action_str = button_id.strip_prefix("action_").unwrap();
                    let action = match action_str {
                        "save" => HUDAction::Save,
                        "undo" => HUDAction::Undo,
                        "redo" => HUDAction::Redo,
                        "help" => HUDAction::ShowHelp,
                        _ => continue,
                    };
                    actions.push(action);
                }
                else if button_id == "toggle_performance" {
                    // Will handle this after the loop to avoid borrowing issues
                    actions.push(HUDAction::TogglePanel(HUDPanel::PerformanceStats));
                }

                // Reset button state
                button.set_state(UIState::Normal);
            }
        }
    }

    /// Toggle panel visibility
    pub fn toggle_panel(&mut self, panel: HUDPanel) {
        let current = self.visible_panels.get(&panel).copied().unwrap_or(false);
        self.visible_panels.insert(panel, !current);
    }

    /// Update performance metrics (called from game engine)
    fn update_performance_metrics(&mut self) {
        // This would be updated by the game engine with real metrics
        // For now, simulate some values
        self.performance.fps = 60.0 + (self.animation_time.sin() * 5.0);
        self.performance.frame_time_ms = 1000.0 / self.performance.fps;
    }

    /// Update build state (called from game engine)
    pub fn update_build_state(&mut self, build_state: BuildState) {
        let old_mode = self.build_state.current_mode;
        let old_material = self.build_state.selected_material;

        self.build_state = build_state;

        // Update button states if mode or material changed
        if old_mode != self.build_state.current_mode {
            self.update_build_mode_buttons();
        }

        if old_material != self.build_state.selected_material {
            self.update_material_buttons();
        }
    }

    /// Update build mode button visual states
    fn update_build_mode_buttons(&mut self) {
        let modes = [
            ("single", BuildMode::Single),
            ("wall", BuildMode::Wall),
            ("floor", BuildMode::Floor),
            ("template", BuildMode::Template),
        ];

        for (id, mode) in modes {
            if let Some(button) = self.buttons.get_mut(&format!("build_mode_{}", id)) {
                if mode == self.build_state.current_mode {
                    // Make this button primary (selected)
                    *button = ModernButton::primary().with_text(button.get_text().clone());
                } else {
                    // Make this button secondary (unselected)
                    *button = ModernButton::secondary().with_text(button.get_text().clone());
                }
            }
        }
    }

    /// Update material button visual states
    fn update_material_buttons(&mut self) {
        let materials = [
            VoxelType::Stone, VoxelType::Dirt, VoxelType::Grass, VoxelType::Wood,
            VoxelType::Glass, VoxelType::Metal, VoxelType::Water, VoxelType::Sand,
        ];

        for material in materials {
            if let Some(button) = self.buttons.get_mut(&format!("material_{:?}", material)) {
                if material == self.build_state.selected_material {
                    *button = ModernButton::primary().with_text(button.get_text().clone());
                } else {
                    *button = ModernButton::secondary().with_text(button.get_text().clone());
                }
            }
        }
    }

    /// Update game state (called from game engine)
    pub fn update_game_state(&mut self, game_state: GameState) {
        self.game_state = game_state;
    }

    /// Update performance data (called from game engine)
    pub fn update_performance(&mut self, performance: PerformanceMetrics) {
        self.performance = performance;
    }

    /// Check if a panel is visible
    pub fn is_panel_visible(&self, panel: &HUDPanel) -> bool {
        self.visible_panels.get(panel).copied().unwrap_or(false)
    }

    /// Get all buttons for rendering
    pub fn get_buttons(&self) -> &HashMap<String, ModernButton> {
        &self.buttons
    }

    /// Get all cards for rendering
    pub fn get_cards(&self) -> &HashMap<String, ModernCard> {
        &self.cards
    }

    /// Get performance metrics for display
    pub fn get_performance_metrics(&self) -> &PerformanceMetrics {
        &self.performance
    }

    /// Get current build state
    pub fn get_build_state(&self) -> &BuildState {
        &self.build_state
    }

    /// Get current game state
    pub fn get_game_state(&self) -> &GameState {
        &self.game_state
    }

    /// Get HUD styles
    pub fn get_styles(&self) -> &HashMap<String, Style> {
        &self.styles
    }

    /// Get the theme
    pub fn get_theme(&self) -> &ProductionDarkTheme {
        &self.theme
    }

    /// Resize HUD (called when window resizes)
    pub fn resize(&mut self, new_width: f32, new_height: f32) {
        self.screen_width = new_width;
        self.screen_height = new_height;

        // Reinitialize layout with new dimensions
        self.initialize_hud();
    }

    /// Hide the HUD
    pub fn hide(&mut self) {
        for panel in self.visible_panels.values_mut() {
            *panel = false;
        }
    }

    /// Show the HUD
    pub fn show(&mut self) {
        self.visible_panels.insert(HUDPanel::PerformanceStats, true);
        self.visible_panels.insert(HUDPanel::BuildTools, true);
        self.visible_panels.insert(HUDPanel::MaterialPalette, true);
        self.visible_panels.insert(HUDPanel::QuickActions, true);
    }
}

// Default implementations
impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            fps: 60.0,
            frame_time_ms: 16.67,
            vertex_count: 0,
            draw_calls: 0,
            memory_usage_mb: 0.0,
            gpu_usage: 0.0,
            world_chunks_loaded: 0,
            particles_active: 0,
        }
    }
}

impl Default for BuildState {
    fn default() -> Self {
        Self {
            current_mode: BuildMode::Single,
            selected_material: VoxelType::Stone,
            selected_template: None,
            template_rotation: 0,
            can_undo: false,
            can_redo: false,
            grid_snap_enabled: true,
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            player_position: Vec3::new(0.0, 0.0, 0.0),
            camera_direction: Vec3::new(0.0, 0.0, -1.0),
            world_time: 12.0, // Noon
            day_night_cycle: "Day".to_string(),
            paused: false,
            save_available: true,
        }
    }
}

/// Extension trait for InputManager to check ctrl modifier
trait InputManagerExt {
    fn is_ctrl_pressed(&self) -> bool;
}

impl InputManagerExt for InputManager {
    fn is_ctrl_pressed(&self) -> bool {
        // This would need to be implemented in the actual InputManager
        // For now, return false as a placeholder
        false
    }
}