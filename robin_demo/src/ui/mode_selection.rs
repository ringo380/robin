/*!
 * Enhanced Mode Selection Interface
 *
 * Visual mode selection with previews for Robin Engine Demo.
 * Part of Phase 4 Milestone 2: User Experience Excellence.
 */

use robin::engine::ui::{
    ModernCard, ModernButton, AccessibilityProps, UIBounds, UIState, ElementId,
    production_theme_simple::ProductionDarkTheme,
    css_in_rust::{Style, StyleSheet, Display, Position},
    styling::{UIColor, Spacing, Border},
    UIElement,
};
use robin::engine::{
    input::InputManager,
    error::RobinResult,
    math::Vec2,
};
use std::collections::HashMap;
use std::time::Instant;

/// Mode selection actions
#[derive(Debug, Clone)]
pub enum ModeSelectionAction {
    SelectMode(DemoMode),
    ShowPreview(DemoMode),
    HidePreview,
    Close,
    None,
}

/// Demo modes with enhanced metadata
#[derive(Debug, Clone, PartialEq)]
pub enum DemoMode {
    InteractivePlayground,
    EngineerBuildShowcase,
    GameplaySystemsDemo,
    CollaborationPreview,
    PerformanceBenchmarks,
    VisualShowcase,
}

impl DemoMode {
    /// Get display information for mode
    pub fn get_info(&self) -> ModeInfo {
        match self {
            DemoMode::InteractivePlayground => ModeInfo {
                title: "Interactive Playground".to_string(),
                description: "Hands-on building experience with real-time physics and particle effects. Perfect for exploring Robin's creative potential.".to_string(),
                features: vec![
                    "Physics-based voxel building".to_string(),
                    "Real-time particle effects".to_string(),
                    "Interactive tutorials".to_string(),
                    "Skill progression tracking".to_string(),
                ],
                thumbnail_color: UIColor::rgb(76, 175, 80), // Green
                icon: "🎮".to_string(),
                difficulty: "Beginner".to_string(),
                estimated_time: "5-15 minutes".to_string(),
            },
            DemoMode::EngineerBuildShowcase => ModeInfo {
                title: "Engineer Build Showcase".to_string(),
                description: "Advanced construction tools and architectural templates. Demonstrates precision building capabilities for complex structures.".to_string(),
                features: vec![
                    "Advanced building templates".to_string(),
                    "Precision construction tools".to_string(),
                    "Golden hour lighting".to_string(),
                    "Template gallery system".to_string(),
                ],
                thumbnail_color: UIColor::rgb(255, 152, 0), // Orange
                icon: "🔧".to_string(),
                difficulty: "Intermediate".to_string(),
                estimated_time: "10-20 minutes".to_string(),
            },
            DemoMode::GameplaySystemsDemo => ModeInfo {
                title: "Gameplay Systems Demo".to_string(),
                description: "Complete game mechanics including resource management, crafting workflows, and AI systems working together.".to_string(),
                features: vec![
                    "Resource management".to_string(),
                    "Crafting workflows".to_string(),
                    "AI assistance systems".to_string(),
                    "Player progression".to_string(),
                ],
                thumbnail_color: UIColor::rgb(156, 39, 176), // Purple
                icon: "🎯".to_string(),
                difficulty: "Intermediate".to_string(),
                estimated_time: "15-25 minutes".to_string(),
            },
            DemoMode::CollaborationPreview => ModeInfo {
                title: "Collaboration Preview".to_string(),
                description: "Multiplayer building and communication systems. Shows how teams can work together on large construction projects.".to_string(),
                features: vec![
                    "Multiplayer synchronization".to_string(),
                    "Team communication tools".to_string(),
                    "Shared project management".to_string(),
                    "Real-time collaboration".to_string(),
                ],
                thumbnail_color: UIColor::rgb(33, 150, 243), // Blue
                icon: "🤝".to_string(),
                difficulty: "Advanced".to_string(),
                estimated_time: "20-30 minutes".to_string(),
            },
            DemoMode::PerformanceBenchmarks => ModeInfo {
                title: "Performance Benchmarks".to_string(),
                description: "Technical showcase of engine optimization including frustum culling, greedy meshing, and Metal rendering performance.".to_string(),
                features: vec![
                    "92% frustum culling efficiency".to_string(),
                    "60-80% vertex reduction".to_string(),
                    "Metal rendering optimization".to_string(),
                    "Real-time performance metrics".to_string(),
                ],
                thumbnail_color: UIColor::rgb(244, 67, 54), // Red
                icon: "⚡".to_string(),
                difficulty: "Technical".to_string(),
                estimated_time: "10-15 minutes".to_string(),
            },
            DemoMode::VisualShowcase => ModeInfo {
                title: "Visual Showcase".to_string(),
                description: "Stunning visual effects including PBR materials, dynamic lighting, and advanced particle systems in cinematic presentation.".to_string(),
                features: vec![
                    "PBR material showcase".to_string(),
                    "Dynamic day/night cycles".to_string(),
                    "Advanced particle effects".to_string(),
                    "Cinematic camera work".to_string(),
                ],
                thumbnail_color: UIColor::rgb(233, 30, 99), // Pink
                icon: "🎨".to_string(),
                difficulty: "Showcase".to_string(),
                estimated_time: "8-12 minutes".to_string(),
            },
        }
    }

    /// Get all available modes
    pub fn all() -> Vec<DemoMode> {
        vec![
            DemoMode::InteractivePlayground,
            DemoMode::EngineerBuildShowcase,
            DemoMode::GameplaySystemsDemo,
            DemoMode::CollaborationPreview,
            DemoMode::PerformanceBenchmarks,
            DemoMode::VisualShowcase,
        ]
    }
}

/// Mode information structure
#[derive(Debug, Clone)]
pub struct ModeInfo {
    pub title: String,
    pub description: String,
    pub features: Vec<String>,
    pub thumbnail_color: UIColor,
    pub icon: String,
    pub difficulty: String,
    pub estimated_time: String,
}

/// Enhanced mode selection interface with visual previews
pub struct ModeSelectionInterface {
    theme: ProductionDarkTheme,
    visible: bool,
    selected_mode: Option<DemoMode>,
    hovered_mode: Option<DemoMode>,

    // UI components
    mode_cards: HashMap<DemoMode, ModernCard>,
    preview_panel: Option<ModernCard>,
    action_buttons: HashMap<String, ModernButton>,

    // Layout
    grid_layout: GridLayout,
    animation_progress: f32,
    fade_alpha: f32,

    // State
    last_update: Instant,
    actions: Vec<ModeSelectionAction>,
}

#[derive(Debug, Clone)]
struct GridLayout {
    columns: usize,
    rows: usize,
    card_width: f32,
    card_height: f32,
    spacing: f32,
    start_x: f32,
    start_y: f32,
}

impl ModeSelectionInterface {
    /// Create new mode selection interface
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        let theme = ProductionDarkTheme::new();

        // Calculate grid layout for 6 modes (2x3 or 3x2)
        let grid_layout = GridLayout {
            columns: 3,
            rows: 2,
            card_width: 280.0,
            card_height: 200.0,
            spacing: 20.0,
            start_x: (screen_width - (3.0 * 280.0 + 2.0 * 20.0)) / 2.0,
            start_y: (screen_height - (2.0 * 200.0 + 1.0 * 20.0)) / 2.0 - 50.0,
        };

        let mut interface = Self {
            theme,
            visible: false,
            selected_mode: None,
            hovered_mode: None,
            mode_cards: HashMap::new(),
            preview_panel: None,
            action_buttons: HashMap::new(),
            grid_layout,
            animation_progress: 0.0,
            fade_alpha: 0.0,
            last_update: Instant::now(),
            actions: Vec::new(),
        };

        interface.create_mode_cards();
        interface.create_action_buttons(screen_width, screen_height);

        interface
    }

    /// Create mode selection cards
    fn create_mode_cards(&mut self) {
        let modes = DemoMode::all();

        for (index, mode) in modes.iter().enumerate() {
            let row = index / self.grid_layout.columns;
            let col = index % self.grid_layout.columns;

            let x = self.grid_layout.start_x + col as f32 * (self.grid_layout.card_width + self.grid_layout.spacing);
            let y = self.grid_layout.start_y + row as f32 * (self.grid_layout.card_height + self.grid_layout.spacing);

            let bounds = UIBounds::new(x, y, self.grid_layout.card_width, self.grid_layout.card_height);
            let mode_info = mode.get_info();

            let mut card = ModernCard::new(index as ElementId + 1000, bounds)
                .with_title(format!("{} {}", mode_info.icon, mode_info.title))
                .with_content(vec![
                    mode_info.description.clone(),
                    format!("Difficulty: {}", mode_info.difficulty),
                    format!("Time: {}", mode_info.estimated_time),
                ]);

            // Add interactive features
            card = card.with_clickable(true)
                .with_hover_effects(true);

            self.mode_cards.insert(mode.clone(), card);
        }
    }

    /// Create action buttons
    fn create_action_buttons(&mut self, screen_width: f32, screen_height: f32) {
        // Start button
        let start_bounds = UIBounds::new(
            screen_width - 200.0,
            screen_height - 80.0,
            150.0,
            50.0
        );

        let start_button = ModernButton::primary()
            .with_text("Start Mode".to_string())
            .with_bounds(start_bounds)
            .with_accessibility(AccessibilityProps {
                aria_label: Some("Start selected mode".to_string()),
                role: "button".to_string(),
                tab_index: 1,
                keyboard_shortcuts: vec!["Enter".to_string()],
                ..AccessibilityProps::default()
            });

        self.action_buttons.insert("start".to_string(), start_button);

        // Cancel button
        let cancel_bounds = UIBounds::new(
            50.0,
            screen_height - 80.0,
            100.0,
            50.0
        );

        let cancel_button = ModernButton::secondary()
            .with_text("Cancel".to_string())
            .with_bounds(cancel_bounds)
            .with_accessibility(AccessibilityProps {
                aria_label: Some("Cancel mode selection".to_string()),
                role: "button".to_string(),
                tab_index: 2,
                keyboard_shortcuts: vec!["Escape".to_string()],
                ..AccessibilityProps::default()
            });

        self.action_buttons.insert("cancel".to_string(), cancel_button);
    }

    /// Show the mode selection interface
    pub fn show(&mut self) {
        self.visible = true;
        self.animation_progress = 0.0;
        self.fade_alpha = 0.0;
        self.last_update = Instant::now();
    }

    /// Hide the mode selection interface
    pub fn hide(&mut self) {
        self.visible = false;
        self.selected_mode = None;
        self.hovered_mode = None;
        self.actions.clear();
    }

    /// Update the interface
    pub fn update(&mut self, delta_time: f32, input: &InputManager) -> RobinResult<Vec<ModeSelectionAction>> {
        if !self.visible {
            return Ok(Vec::new());
        }

        // Update animations
        self.update_animations(delta_time);

        // Update mode cards
        self.update_mode_cards(input);

        // Update action buttons
        self.update_action_buttons(input);

        // Handle keyboard input
        self.handle_keyboard_input(input);

        // Return accumulated actions
        let actions = self.actions.clone();
        self.actions.clear();
        Ok(actions)
    }

    /// Update animations
    fn update_animations(&mut self, delta_time: f32) {
        if self.visible {
            self.animation_progress = (self.animation_progress + delta_time * 2.0).min(1.0);
            self.fade_alpha = self.ease_in_out(self.animation_progress);
        }
    }

    /// Update mode cards for interaction
    fn update_mode_cards(&mut self, input: &InputManager) {
        let mouse_pos = input.get_mouse_position();
        let mouse_clicked = input.is_mouse_button_pressed(robin::engine::input::MouseButton::Left);

        for (mode, card) in &mut self.mode_cards {
            // Check hover
            if card.get_bounds().contains_point(mouse_pos) {
                if self.hovered_mode.as_ref() != Some(mode) {
                    self.hovered_mode = Some(mode.clone());
                    self.actions.push(ModeSelectionAction::ShowPreview(mode.clone()));
                }

                // Check click
                if mouse_clicked {
                    self.selected_mode = Some(mode.clone());
                    self.actions.push(ModeSelectionAction::SelectMode(mode.clone()));
                }
            }
        }

        // Hide preview if not hovering
        if let Some(ref hovered) = self.hovered_mode.clone() {
            let still_hovering = self.mode_cards.get(hovered)
                .map(|card| card.get_bounds().contains_point(mouse_pos))
                .unwrap_or(false);

            if !still_hovering {
                self.hovered_mode = None;
                self.actions.push(ModeSelectionAction::HidePreview);
            }
        }
    }

    /// Update action buttons
    fn update_action_buttons(&mut self, input: &InputManager) {
        let mouse_pos = input.get_mouse_position();
        let mouse_clicked = input.is_mouse_button_pressed(robin::engine::input::MouseButton::Left);

        // Start button
        if let Some(start_button) = self.action_buttons.get("start") {
            if start_button.get_bounds().contains_point(mouse_pos) && mouse_clicked {
                if let Some(ref selected) = self.selected_mode {
                    self.actions.push(ModeSelectionAction::SelectMode(selected.clone()));
                }
            }
        }

        // Cancel button
        if let Some(cancel_button) = self.action_buttons.get("cancel") {
            if cancel_button.get_bounds().contains_point(mouse_pos) && mouse_clicked {
                self.actions.push(ModeSelectionAction::Close);
            }
        }
    }

    /// Handle keyboard input
    fn handle_keyboard_input(&mut self, input: &InputManager) {
        // F1-F6 hotkeys for direct mode selection
        if input.is_key_pressed("F1") {
            self.actions.push(ModeSelectionAction::SelectMode(DemoMode::InteractivePlayground));
        } else if input.is_key_pressed("F2") {
            self.actions.push(ModeSelectionAction::SelectMode(DemoMode::EngineerBuildShowcase));
        } else if input.is_key_pressed("F3") {
            self.actions.push(ModeSelectionAction::SelectMode(DemoMode::GameplaySystemsDemo));
        } else if input.is_key_pressed("F4") {
            self.actions.push(ModeSelectionAction::SelectMode(DemoMode::CollaborationPreview));
        } else if input.is_key_pressed("F5") {
            self.actions.push(ModeSelectionAction::SelectMode(DemoMode::PerformanceBenchmarks));
        } else if input.is_key_pressed("F6") {
            self.actions.push(ModeSelectionAction::SelectMode(DemoMode::VisualShowcase));
        }

        // Escape to close
        if input.is_key_pressed("Escape") {
            self.actions.push(ModeSelectionAction::Close);
        }

        // Enter to start selected mode
        if input.is_key_pressed("Enter") || input.is_key_pressed("Return") {
            if let Some(ref selected) = self.selected_mode {
                self.actions.push(ModeSelectionAction::SelectMode(selected.clone()));
            }
        }
    }

    /// Easing function for smooth animations
    fn ease_in_out(&self, t: f32) -> f32 {
        if t < 0.5 {
            2.0 * t * t
        } else {
            -1.0 + (4.0 - 2.0 * t) * t
        }
    }

    /// Check if interface is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Get selected mode
    pub fn get_selected_mode(&self) -> Option<&DemoMode> {
        self.selected_mode.as_ref()
    }

    /// Get hovered mode for preview
    pub fn get_hovered_mode(&self) -> Option<&DemoMode> {
        self.hovered_mode.as_ref()
    }

    /// Get current fade alpha for rendering
    pub fn get_fade_alpha(&self) -> f32 {
        self.fade_alpha
    }

    /// Get mode cards for rendering
    pub fn get_mode_cards(&self) -> &HashMap<DemoMode, ModernCard> {
        &self.mode_cards
    }

    /// Get action buttons for rendering
    pub fn get_action_buttons(&self) -> &HashMap<String, ModernButton> {
        &self.action_buttons
    }
}

impl UIElement for ModeSelectionInterface {
    fn get_id(&self) -> ElementId {
        9999 // High ID for mode selection overlay
    }

    fn get_bounds(&self) -> UIBounds {
        UIBounds::new(0.0, 0.0, 1920.0, 1080.0) // Full screen overlay
    }

    fn get_state(&self) -> UIState {
        if self.visible {
            UIState::Normal
        } else {
            UIState::Disabled
        }
    }

    fn set_state(&mut self, state: UIState) {
        match state {
            UIState::Normal => self.show(),
            UIState::Disabled => self.hide(),
            _ => {}
        }
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        if visible {
            self.show();
        } else {
            self.hide();
        }
    }

    fn update(&mut self, delta_time: f32, input: &InputManager) {
        let _ = self.update(delta_time, input);
    }

    fn handle_event(&mut self, _event: robin::engine::ui::UIEvent) -> bool {
        // Event handling for accessibility
        false
    }
}

impl Default for ModeSelectionInterface {
    fn default() -> Self {
        Self::new(1920.0, 1080.0)
    }
}