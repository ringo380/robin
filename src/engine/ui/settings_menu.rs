/*!
 * Production Settings Menu System
 *
 * Comprehensive settings interface for Robin Engine with graphics, audio,
 * controls, and accessibility options. Uses modern components and dark theme.
 */

use crate::engine::{
    ui::{
        production_theme_simple::ProductionDarkTheme,
        modern_components::{ModernButton, AccessibilityProps},
        css_in_rust::Style,
        UIBounds, UIState, UIElement,
    },
    input::InputManager,
    error::RobinResult,
};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Settings categories
#[derive(Debug, Clone, PartialEq)]
pub enum SettingsCategory {
    Graphics,
    Audio,
    Controls,
    Gameplay,
    Accessibility,
    System,
}

/// Settings actions sent to the engine
#[derive(Debug, Clone)]
pub enum SettingsAction {
    // Graphics
    SetResolution(u32, u32),
    SetFullscreen(bool),
    SetVSync(bool),
    SetGraphicsQuality(GraphicsQuality),
    SetRenderDistance(u32),
    SetShadowQuality(ShadowQuality),
    SetAntiAliasing(AntiAliasingMode),

    // Audio
    SetMasterVolume(f32),
    SetEffectsVolume(f32),
    SetMusicVolume(f32),
    SetSpatialAudio(bool),
    SetAudioDevice(String),

    // Controls
    SetMouseSensitivity(f32),
    SetInvertMouse(bool),
    SetKeyBinding(String, String), // action, key
    ResetKeyBindings,

    // Gameplay
    SetAutoSave(bool),
    SetAutoSaveInterval(u32),
    SetTutorialsEnabled(bool),
    SetBuildGridSnap(bool),
    SetShowHints(bool),

    // Accessibility
    SetHighContrast(bool),
    SetLargeText(bool),
    SetReducedMotion(bool),
    SetColorBlindMode(ColorBlindMode),
    SetScreenReaderMode(bool),

    // System
    SetLanguage(String),
    SetSaveLocation(String),
    ResetToDefaults,
    ApplySettings,
    CancelSettings,
    Close,
}

/// Graphics quality presets
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GraphicsQuality {
    Low,
    Medium,
    High,
    Ultra,
    Custom,
}

/// Shadow quality levels
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ShadowQuality {
    Off,
    Low,
    Medium,
    High,
}

/// Anti-aliasing modes
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AntiAliasingMode {
    Off,
    FXAA,
    MSAA2x,
    MSAA4x,
    MSAA8x,
}

/// Color blind assistance modes
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ColorBlindMode {
    None,
    Protanopia,    // Red-blind
    Deuteranopia,  // Green-blind
    Tritanopia,    // Blue-blind
}

/// Complete settings configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsConfig {
    // Graphics
    pub resolution: (u32, u32),
    pub fullscreen: bool,
    pub vsync: bool,
    pub graphics_quality: GraphicsQuality,
    pub render_distance: u32,
    pub shadow_quality: ShadowQuality,
    pub anti_aliasing: AntiAliasingMode,
    pub show_fps: bool,

    // Audio
    pub master_volume: f32,
    pub effects_volume: f32,
    pub music_volume: f32,
    pub spatial_audio: bool,
    pub audio_device: String,

    // Controls
    pub mouse_sensitivity: f32,
    pub invert_mouse: bool,
    pub key_bindings: HashMap<String, String>,

    // Gameplay
    pub auto_save: bool,
    pub auto_save_interval: u32,
    pub tutorials_enabled: bool,
    pub build_grid_snap: bool,
    pub show_hints: bool,

    // Accessibility
    pub high_contrast: bool,
    pub large_text: bool,
    pub reduced_motion: bool,
    pub color_blind_mode: ColorBlindMode,
    pub screen_reader_mode: bool,

    // System
    pub language: String,
    pub save_location: String,
}

/// Settings menu system
pub struct SettingsMenuSystem {
    theme: ProductionDarkTheme,
    current_category: SettingsCategory,
    visible: bool,

    // Configuration
    settings: SettingsConfig,
    temp_settings: SettingsConfig, // For preview before applying
    has_changes: bool,

    // UI components
    category_buttons: HashMap<String, ModernButton>,
    setting_controls: HashMap<String, SettingControl>,
    action_buttons: HashMap<String, ModernButton>,

    // State
    selected_control: Option<String>,
    category_order: Vec<SettingsCategory>,

    // Layout
    screen_width: f32,
    screen_height: f32,
    styles: HashMap<String, Style>,
}

/// Individual setting control (slider, toggle, dropdown, etc.)
#[derive(Debug, Clone)]
pub struct SettingControl {
    pub id: String,
    pub label: String,
    pub description: String,
    pub control_type: ControlType,
    pub bounds: UIBounds,
    pub value: SettingValue,
    pub enabled: bool,
}

/// Types of setting controls
#[derive(Debug, Clone)]
pub enum ControlType {
    Toggle,
    Slider { min: f32, max: f32, step: f32 },
    Dropdown { options: Vec<String> },
    KeyBinding,
    TextInput,
    Button,
}

/// Setting values
#[derive(Debug, Clone)]
pub enum SettingValue {
    Bool(bool),
    Float(f32),
    Int(i32),
    String(String),
    Index(usize), // For dropdowns
}

impl SettingsMenuSystem {
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        let theme = ProductionDarkTheme::new();
        let styles = theme.create_main_menu_styles(); // Reuse main menu styles

        let mut system = Self {
            theme,
            current_category: SettingsCategory::Graphics,
            visible: false,
            settings: SettingsConfig::default(),
            temp_settings: SettingsConfig::default(),
            has_changes: false,
            category_buttons: HashMap::new(),
            setting_controls: HashMap::new(),
            action_buttons: HashMap::new(),
            selected_control: None,
            category_order: vec![
                SettingsCategory::Graphics,
                SettingsCategory::Audio,
                SettingsCategory::Controls,
                SettingsCategory::Gameplay,
                SettingsCategory::Accessibility,
                SettingsCategory::System,
            ],
            screen_width,
            screen_height,
            styles,
        };

        system.initialize_ui();
        system
    }

    /// Initialize the settings UI
    fn initialize_ui(&mut self) {
        self.create_category_buttons();
        self.create_settings_controls();
        self.create_action_buttons();
    }

    /// Create category navigation buttons
    fn create_category_buttons(&mut self) {
        let categories = vec![
            (SettingsCategory::Graphics, "🎮 Graphics", "Display and rendering settings"),
            (SettingsCategory::Audio, "🔊 Audio", "Sound and music settings"),
            (SettingsCategory::Controls, "⌨️ Controls", "Input and key bindings"),
            (SettingsCategory::Gameplay, "🎯 Gameplay", "Game behavior settings"),
            (SettingsCategory::Accessibility, "♿ Accessibility", "Accessibility options"),
            (SettingsCategory::System, "⚙️ System", "System and advanced settings"),
        ];

        for (i, (category, text, description)) in categories.iter().enumerate() {
            let button_bounds = UIBounds::new(
                20.0,
                100.0 + i as f32 * 60.0,
                200.0,
                50.0,
            );

            let accessibility = AccessibilityProps {
                aria_label: Some(text.to_string()),
                aria_description: Some(description.to_string()),
                role: "tab".to_string(),
                tab_index: i as i32,
                keyboard_shortcuts: vec![format!("F{}", i + 1)],
                ..Default::default()
            };

            let button = if *category == self.current_category {
                ModernButton::primary()
            } else {
                ModernButton::secondary()
            }.with_text(text.to_string())
             .with_accessibility(accessibility);

            self.category_buttons.insert(format!("{:?}", category), button);
        }
    }

    /// Create setting controls for current category
    fn create_settings_controls(&mut self) {
        self.setting_controls.clear();

        match self.current_category {
            SettingsCategory::Graphics => self.create_graphics_controls(),
            SettingsCategory::Audio => self.create_audio_controls(),
            SettingsCategory::Controls => self.create_controls_controls(),
            SettingsCategory::Gameplay => self.create_gameplay_controls(),
            SettingsCategory::Accessibility => self.create_accessibility_controls(),
            SettingsCategory::System => self.create_system_controls(),
        }
    }

    /// Create graphics setting controls
    fn create_graphics_controls(&mut self) {
        let controls = vec![
            (
                "resolution",
                "Resolution",
                "Display resolution",
                ControlType::Dropdown {
                    options: vec![
                        "1920x1080".to_string(),
                        "2560x1440".to_string(),
                        "3840x2160".to_string(),
                        "Auto".to_string(),
                    ],
                },
                SettingValue::Index(0),
            ),
            (
                "fullscreen",
                "Fullscreen",
                "Run in fullscreen mode",
                ControlType::Toggle,
                SettingValue::Bool(self.temp_settings.fullscreen),
            ),
            (
                "vsync",
                "VSync",
                "Vertical synchronization",
                ControlType::Toggle,
                SettingValue::Bool(self.temp_settings.vsync),
            ),
            (
                "graphics_quality",
                "Graphics Quality",
                "Overall graphics quality preset",
                ControlType::Dropdown {
                    options: vec![
                        "Low".to_string(),
                        "Medium".to_string(),
                        "High".to_string(),
                        "Ultra".to_string(),
                        "Custom".to_string(),
                    ],
                },
                SettingValue::Index(self.temp_settings.graphics_quality as usize),
            ),
            (
                "render_distance",
                "Render Distance",
                "Maximum chunk render distance",
                ControlType::Slider { min: 2.0, max: 32.0, step: 2.0 },
                SettingValue::Float(self.temp_settings.render_distance as f32),
            ),
            (
                "shadow_quality",
                "Shadow Quality",
                "Shadow rendering quality",
                ControlType::Dropdown {
                    options: vec![
                        "Off".to_string(),
                        "Low".to_string(),
                        "Medium".to_string(),
                        "High".to_string(),
                    ],
                },
                SettingValue::Index(self.temp_settings.shadow_quality as usize),
            ),
            (
                "show_fps",
                "Show FPS",
                "Display frame rate counter",
                ControlType::Toggle,
                SettingValue::Bool(self.temp_settings.show_fps),
            ),
        ];

        self.create_controls_from_list(controls);
    }

    /// Create audio setting controls
    fn create_audio_controls(&mut self) {
        let controls = vec![
            (
                "master_volume",
                "Master Volume",
                "Overall audio volume",
                ControlType::Slider { min: 0.0, max: 1.0, step: 0.1 },
                SettingValue::Float(self.temp_settings.master_volume),
            ),
            (
                "effects_volume",
                "Effects Volume",
                "Sound effects volume",
                ControlType::Slider { min: 0.0, max: 1.0, step: 0.1 },
                SettingValue::Float(self.temp_settings.effects_volume),
            ),
            (
                "music_volume",
                "Music Volume",
                "Background music volume",
                ControlType::Slider { min: 0.0, max: 1.0, step: 0.1 },
                SettingValue::Float(self.temp_settings.music_volume),
            ),
            (
                "spatial_audio",
                "Spatial Audio",
                "3D positional audio",
                ControlType::Toggle,
                SettingValue::Bool(self.temp_settings.spatial_audio),
            ),
        ];

        self.create_controls_from_list(controls);
    }

    /// Create controls setting controls
    fn create_controls_controls(&mut self) {
        let controls = vec![
            (
                "mouse_sensitivity",
                "Mouse Sensitivity",
                "Camera rotation speed",
                ControlType::Slider { min: 0.1, max: 3.0, step: 0.1 },
                SettingValue::Float(self.temp_settings.mouse_sensitivity),
            ),
            (
                "invert_mouse",
                "Invert Mouse",
                "Invert Y-axis mouse movement",
                ControlType::Toggle,
                SettingValue::Bool(self.temp_settings.invert_mouse),
            ),
            (
                "move_forward",
                "Move Forward",
                "Key binding for forward movement",
                ControlType::KeyBinding,
                SettingValue::String(
                    self.temp_settings.key_bindings
                        .get("move_forward")
                        .unwrap_or(&"W".to_string())
                        .clone()
                ),
            ),
            (
                "move_backward",
                "Move Backward",
                "Key binding for backward movement",
                ControlType::KeyBinding,
                SettingValue::String(
                    self.temp_settings.key_bindings
                        .get("move_backward")
                        .unwrap_or(&"S".to_string())
                        .clone()
                ),
            ),
            (
                "move_left",
                "Move Left",
                "Key binding for left movement",
                ControlType::KeyBinding,
                SettingValue::String(
                    self.temp_settings.key_bindings
                        .get("move_left")
                        .unwrap_or(&"A".to_string())
                        .clone()
                ),
            ),
            (
                "move_right",
                "Move Right",
                "Key binding for right movement",
                ControlType::KeyBinding,
                SettingValue::String(
                    self.temp_settings.key_bindings
                        .get("move_right")
                        .unwrap_or(&"D".to_string())
                        .clone()
                ),
            ),
        ];

        self.create_controls_from_list(controls);
    }

    /// Create gameplay setting controls
    fn create_gameplay_controls(&mut self) {
        let controls = vec![
            (
                "auto_save",
                "Auto Save",
                "Automatically save progress",
                ControlType::Toggle,
                SettingValue::Bool(self.temp_settings.auto_save),
            ),
            (
                "auto_save_interval",
                "Auto Save Interval",
                "Minutes between auto saves",
                ControlType::Slider { min: 1.0, max: 30.0, step: 1.0 },
                SettingValue::Float(self.temp_settings.auto_save_interval as f32),
            ),
            (
                "tutorials_enabled",
                "Show Tutorials",
                "Enable tutorial hints and guides",
                ControlType::Toggle,
                SettingValue::Bool(self.temp_settings.tutorials_enabled),
            ),
            (
                "build_grid_snap",
                "Grid Snap",
                "Snap building to grid",
                ControlType::Toggle,
                SettingValue::Bool(self.temp_settings.build_grid_snap),
            ),
            (
                "show_hints",
                "Show Hints",
                "Display helpful hints during gameplay",
                ControlType::Toggle,
                SettingValue::Bool(self.temp_settings.show_hints),
            ),
        ];

        self.create_controls_from_list(controls);
    }

    /// Create accessibility setting controls
    fn create_accessibility_controls(&mut self) {
        let controls = vec![
            (
                "high_contrast",
                "High Contrast",
                "Increase visual contrast for better visibility",
                ControlType::Toggle,
                SettingValue::Bool(self.temp_settings.high_contrast),
            ),
            (
                "large_text",
                "Large Text",
                "Increase text size for better readability",
                ControlType::Toggle,
                SettingValue::Bool(self.temp_settings.large_text),
            ),
            (
                "reduced_motion",
                "Reduce Motion",
                "Minimize animations and transitions",
                ControlType::Toggle,
                SettingValue::Bool(self.temp_settings.reduced_motion),
            ),
            (
                "color_blind_mode",
                "Color Blind Mode",
                "Adjust colors for color vision deficiency",
                ControlType::Dropdown {
                    options: vec![
                        "None".to_string(),
                        "Protanopia (Red-blind)".to_string(),
                        "Deuteranopia (Green-blind)".to_string(),
                        "Tritanopia (Blue-blind)".to_string(),
                    ],
                },
                SettingValue::Index(self.temp_settings.color_blind_mode as usize),
            ),
            (
                "screen_reader_mode",
                "Screen Reader Mode",
                "Enhanced screen reader compatibility",
                ControlType::Toggle,
                SettingValue::Bool(self.temp_settings.screen_reader_mode),
            ),
        ];

        self.create_controls_from_list(controls);
    }

    /// Create system setting controls
    fn create_system_controls(&mut self) {
        let controls = vec![
            (
                "language",
                "Language",
                "Interface language",
                ControlType::Dropdown {
                    options: vec![
                        "English".to_string(),
                        "Spanish".to_string(),
                        "French".to_string(),
                        "German".to_string(),
                        "Japanese".to_string(),
                    ],
                },
                SettingValue::Index(0),
            ),
            (
                "save_location",
                "Save Location",
                "Directory for saved worlds",
                ControlType::TextInput,
                SettingValue::String(self.temp_settings.save_location.clone()),
            ),
            (
                "reset_defaults",
                "Reset to Defaults",
                "Restore all settings to default values",
                ControlType::Button,
                SettingValue::String("Reset".to_string()),
            ),
        ];

        self.create_controls_from_list(controls);
    }

    /// Helper to create controls from a list
    fn create_controls_from_list(&mut self, controls: Vec<(&str, &str, &str, ControlType, SettingValue)>) {
        for (i, (id, label, description, control_type, value)) in controls.iter().enumerate() {
            let bounds = UIBounds::new(
                250.0,
                120.0 + i as f32 * 60.0,
                400.0,
                50.0,
            );

            let control = SettingControl {
                id: id.to_string(),
                label: label.to_string(),
                description: description.to_string(),
                control_type: control_type.clone(),
                bounds,
                value: value.clone(),
                enabled: true,
            };

            self.setting_controls.insert(id.to_string(), control);
        }
    }

    /// Create action buttons (Apply, Cancel, etc.)
    fn create_action_buttons(&mut self) {
        let buttons = vec![
            ("apply", "Apply", "Apply all changes"),
            ("cancel", "Cancel", "Discard changes"),
            ("reset", "Reset", "Reset to defaults"),
            ("close", "Close", "Close settings"),
        ];

        for (i, (id, text, description)) in buttons.iter().enumerate() {
            let button_bounds = UIBounds::new(
                250.0 + i as f32 * 100.0,
                self.screen_height - 80.0,
                90.0,
                40.0,
            );

            let accessibility = AccessibilityProps {
                aria_label: Some(text.to_string()),
                aria_description: Some(description.to_string()),
                role: "button".to_string(),
                tab_index: 100 + i as i32,
                ..Default::default()
            };

            let button_style = match *id {
                "apply" => ModernButton::primary(),
                "cancel" | "reset" => ModernButton::secondary(),
                _ => ModernButton::ghost(),
            };

            let button = button_style
                .with_text(text.to_string())
                .with_accessibility(accessibility);

            self.action_buttons.insert(id.to_string(), button);
        }
    }

    /// Update the settings menu
    pub fn update(&mut self, delta_time: f32, input: &InputManager) -> RobinResult<Vec<SettingsAction>> {
        let mut actions = Vec::new();

        if !self.visible {
            return Ok(actions);
        }

        // Handle keyboard navigation
        self.handle_keyboard_navigation(input, &mut actions);

        // Update all buttons
        for button in self.category_buttons.values_mut() {
            button.update(delta_time, input);
        }

        for button in self.action_buttons.values_mut() {
            button.update(delta_time, input);
        }

        // Check for interactions
        self.check_category_interactions(&mut actions);
        self.check_action_interactions(&mut actions);

        Ok(actions)
    }

    /// Handle keyboard navigation
    fn handle_keyboard_navigation(&mut self, input: &InputManager, actions: &mut Vec<SettingsAction>) {
        // Category switching with F1-F6
        let mut category_to_switch = None;
        for (i, category) in self.category_order.iter().enumerate() {
            let key_name = format!("F{}", i + 1);
            if input.is_key_just_pressed(&winit::keyboard::Key::Named(
                match key_name.as_str() {
                    "F1" => winit::keyboard::NamedKey::F1,
                    "F2" => winit::keyboard::NamedKey::F2,
                    "F3" => winit::keyboard::NamedKey::F3,
                    "F4" => winit::keyboard::NamedKey::F4,
                    "F5" => winit::keyboard::NamedKey::F5,
                    "F6" => winit::keyboard::NamedKey::F6,
                    _ => continue,
                }
            )) {
                category_to_switch = Some(category.clone());
                break;
            }
        }

        // Switch category after the borrow ends
        if let Some(category) = category_to_switch {
            self.switch_category(category);
        }

        // Escape to close
        if input.is_named_key_just_pressed(winit::keyboard::NamedKey::Escape) {
            actions.push(SettingsAction::Close);
        }

        // Enter to apply
        if input.is_named_key_just_pressed(winit::keyboard::NamedKey::Enter) {
            actions.push(SettingsAction::ApplySettings);
        }
    }

    /// Check category button interactions
    fn check_category_interactions(&mut self, actions: &mut Vec<SettingsAction>) {
        let mut pending_category = None;
        for (category_str, button) in &mut self.category_buttons {
            if button.get_state() == UIState::Pressed {
                if let Ok(category) = category_str.parse::<SettingsCategory>() {
                    pending_category = Some(category);
                }
                button.set_state(UIState::Normal);
            }
        }

        // Apply category switch after loop
        if let Some(category) = pending_category {
            self.switch_category(category);
        }
    }

    /// Check action button interactions
    fn check_action_interactions(&mut self, actions: &mut Vec<SettingsAction>) {
        for (action_id, button) in &mut self.action_buttons {
            if button.get_state() == UIState::Pressed {
                let action = match action_id.as_str() {
                    "apply" => SettingsAction::ApplySettings,
                    "cancel" => SettingsAction::CancelSettings,
                    "reset" => SettingsAction::ResetToDefaults,
                    "close" => SettingsAction::Close,
                    _ => continue,
                };
                actions.push(action);
                button.set_state(UIState::Normal);
            }
        }
    }

    /// Switch to a different settings category
    fn switch_category(&mut self, category: SettingsCategory) {
        if category != self.current_category {
            // Update button states
            for (category_str, button) in &mut self.category_buttons {
                if category_str == &format!("{:?}", category) {
                    *button = ModernButton::primary().with_text(button.get_text().clone());
                } else {
                    *button = ModernButton::secondary().with_text(button.get_text().clone());
                }
            }

            self.current_category = category;
            self.create_settings_controls();
        }
    }

    /// Show the settings menu
    pub fn show(&mut self) {
        self.visible = true;
        self.temp_settings = self.settings.clone();
        self.has_changes = false;
    }

    /// Hide the settings menu
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Apply settings changes
    pub fn apply_settings(&mut self) {
        self.settings = self.temp_settings.clone();
        self.has_changes = false;
    }

    /// Cancel settings changes
    pub fn cancel_settings(&mut self) {
        self.temp_settings = self.settings.clone();
        self.has_changes = false;
        self.create_settings_controls(); // Refresh UI
    }

    /// Reset to default settings
    pub fn reset_to_defaults(&mut self) {
        self.temp_settings = SettingsConfig::default();
        self.has_changes = true;
        self.create_settings_controls(); // Refresh UI
    }

    /// Get current settings
    pub fn get_settings(&self) -> &SettingsConfig {
        &self.settings
    }

    /// Check if menu is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Check if there are unsaved changes
    pub fn has_unsaved_changes(&self) -> bool {
        self.has_changes
    }

    /// Get current category
    pub fn get_current_category(&self) -> &SettingsCategory {
        &self.current_category
    }

    /// Get category buttons for rendering
    pub fn get_category_buttons(&self) -> &HashMap<String, ModernButton> {
        &self.category_buttons
    }

    /// Get action buttons for rendering
    pub fn get_action_buttons(&self) -> &HashMap<String, ModernButton> {
        &self.action_buttons
    }

    /// Get setting controls for rendering
    pub fn get_setting_controls(&self) -> &HashMap<String, SettingControl> {
        &self.setting_controls
    }

    /// Get theme
    pub fn get_theme(&self) -> &ProductionDarkTheme {
        &self.theme
    }
}

// Default settings configuration
impl Default for SettingsConfig {
    fn default() -> Self {
        let mut key_bindings = HashMap::new();
        key_bindings.insert("move_forward".to_string(), "W".to_string());
        key_bindings.insert("move_backward".to_string(), "S".to_string());
        key_bindings.insert("move_left".to_string(), "A".to_string());
        key_bindings.insert("move_right".to_string(), "D".to_string());
        key_bindings.insert("jump".to_string(), "Space".to_string());
        key_bindings.insert("build_mode".to_string(), "B".to_string());
        key_bindings.insert("save".to_string(), "Ctrl+S".to_string());
        key_bindings.insert("help".to_string(), "H".to_string());

        Self {
            resolution: (1920, 1080),
            fullscreen: false,
            vsync: true,
            graphics_quality: GraphicsQuality::High,
            render_distance: 16,
            shadow_quality: ShadowQuality::Medium,
            anti_aliasing: AntiAliasingMode::FXAA,
            show_fps: false,

            master_volume: 0.8,
            effects_volume: 0.7,
            music_volume: 0.6,
            spatial_audio: true,
            audio_device: "Default".to_string(),

            mouse_sensitivity: 1.0,
            invert_mouse: false,
            key_bindings,

            auto_save: true,
            auto_save_interval: 5,
            tutorials_enabled: true,
            build_grid_snap: true,
            show_hints: true,

            high_contrast: false,
            large_text: false,
            reduced_motion: false,
            color_blind_mode: ColorBlindMode::None,
            screen_reader_mode: false,

            language: "English".to_string(),
            save_location: "saves/".to_string(),
        }
    }
}

// Helper to parse category from string
impl std::str::FromStr for SettingsCategory {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Graphics" => Ok(SettingsCategory::Graphics),
            "Audio" => Ok(SettingsCategory::Audio),
            "Controls" => Ok(SettingsCategory::Controls),
            "Gameplay" => Ok(SettingsCategory::Gameplay),
            "Accessibility" => Ok(SettingsCategory::Accessibility),
            "System" => Ok(SettingsCategory::System),
            _ => Err(()),
        }
    }
}