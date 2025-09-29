/*!
 * Production Dark Theme System
 *
 * Modern dark theme implementation for Robin Engine production release.
 * Integrates with existing CSS-in-Rust and modern components systems.
 */

use crate::engine::ui::{
    css_in_rust::{Style, StyleSheet, BoxShadow, Transition},
    styling::Color,
};
use std::collections::HashMap;

/// Production dark theme configuration
#[derive(Debug, Clone)]
pub struct ProductionDarkTheme {
    pub colors: DarkColorPalette,
    pub typography: DarkTypography,
    pub spacing: SpacingSystem,
    pub animations: AnimationSystem,
    pub components: ComponentStyles,
}

/// Dark theme color palette optimized for 3D voxel editing
#[derive(Debug, Clone)]
pub struct DarkColorPalette {
    // Primary colors
    pub primary: Color,
    pub primary_hover: Color,
    pub primary_active: Color,
    pub primary_disabled: Color,

    // Secondary colors
    pub secondary: Color,
    pub secondary_hover: Color,
    pub secondary_active: Color,

    // Background layers
    pub background_primary: Color,    // Main background
    pub background_secondary: Color,  // Cards, panels
    pub background_tertiary: Color,   // Input fields, dropdowns
    pub background_overlay: Color,    // Modals, tooltips

    // Text colors
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_disabled: Color,
    pub text_inverse: Color,

    // Border colors
    pub border_primary: Color,
    pub border_secondary: Color,
    pub border_focus: Color,
    pub border_error: Color,

    // Status colors
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,

    // Voxel/game specific
    pub build_mode_active: Color,
    pub material_preview: Color,
    pub grid_lines: Color,
    pub selection_highlight: Color,
}

/// Typography system for dark theme
#[derive(Debug, Clone)]
pub struct DarkTypography {
    pub font_family_primary: String,
    pub font_family_mono: String,

    // Font sizes (rem scale)
    pub size_xs: f32,
    pub size_sm: f32,
    pub size_base: f32,
    pub size_lg: f32,
    pub size_xl: f32,
    pub size_2xl: f32,
    pub size_3xl: f32,

    // Font weights
    pub weight_light: u32,
    pub weight_normal: u32,
    pub weight_medium: u32,
    pub weight_semibold: u32,
    pub weight_bold: u32,

    // Line heights
    pub line_height_tight: f32,
    pub line_height_normal: f32,
    pub line_height_relaxed: f32,
}

/// Consistent spacing system
#[derive(Debug, Clone)]
pub struct SpacingSystem {
    pub xs: f32,    // 4px
    pub sm: f32,    // 8px
    pub md: f32,    // 16px
    pub lg: f32,    // 24px
    pub xl: f32,    // 32px
    pub xxl: f32,   // 48px
    pub xxxl: f32,  // 64px
}

/// Animation and transition system
#[derive(Debug, Clone)]
pub struct AnimationSystem {
    pub duration_fast: f32,     // 150ms
    pub duration_normal: f32,   // 250ms
    pub duration_slow: f32,     // 350ms

    pub easing_ease_out: String,
    pub easing_ease_in_out: String,
    pub easing_bounce: String,
}

/// Component-specific styles
#[derive(Debug, Clone)]
pub struct ComponentStyles {
    pub button_primary: Style,
    pub button_secondary: Style,
    pub button_ghost: Style,
    pub card: Style,
    pub input: Style,
    pub modal: Style,
    pub tooltip: Style,
    pub menu: Style,
}

impl ProductionDarkTheme {
    pub fn new() -> Self {
        Self {
            colors: DarkColorPalette::new(),
            typography: DarkTypography::new(),
            spacing: SpacingSystem::new(),
            animations: AnimationSystem::new(),
            components: ComponentStyles::new(),
        }
    }

    /// Generate complete stylesheet for the dark theme
    pub fn generate_stylesheet(&self) -> StyleSheet {
        let mut stylesheet = StyleSheet::default();

        // Define CSS variables for the theme
        stylesheet.variables.insert("--color-primary".to_string(),
            self.colors.primary.to_css_variable());
        stylesheet.variables.insert("--color-bg-primary".to_string(),
            self.colors.background_primary.to_css_variable());
        stylesheet.variables.insert("--spacing-md".to_string(),
            format!("{}px", self.spacing.md).into());

        // Component styles
        stylesheet.styles.insert("button-primary".to_string(), self.components.button_primary.clone());
        stylesheet.styles.insert("button-secondary".to_string(), self.components.button_secondary.clone());
        stylesheet.styles.insert("card".to_string(), self.components.card.clone());
        stylesheet.styles.insert("modal".to_string(), self.components.modal.clone());

        stylesheet
    }

    /// Create main menu styles
    pub fn create_main_menu_styles(&self) -> HashMap<String, Style> {
        let mut styles = HashMap::new();

        // Main menu container
        styles.insert("main-menu".to_string(), Style {
            background_color: Some(self.colors.background_primary),
            width: Some("100vw".into()),
            height: Some("100vh".into()),
            display: Some("flex".into()),
            flex_direction: Some("column".into()),
            align_items: Some("center".into()),
            justify_content: Some("center".into()),
            ..Default::default()
        });

        // Menu title
        styles.insert("menu-title".to_string(), Style {
            font_size: Some(self.typography.size_3xl),
            font_weight: Some(self.typography.weight_bold.into()),
            color: Some(self.colors.text_primary),
            margin: Some(format!("0 0 {}px 0", self.spacing.xl).into()),
            text_align: Some("center".into()),
            ..Default::default()
        });

        // Menu buttons container
        styles.insert("menu-buttons".to_string(), Style {
            display: Some("flex".into()),
            flex_direction: Some("column".into()),
            gap: Some(self.spacing.md),
            width: Some("300px".into()),
            ..Default::default()
        });

        styles
    }

    /// Create in-game HUD styles
    pub fn create_hud_styles(&self) -> HashMap<String, Style> {
        let mut styles = HashMap::new();

        // HUD container
        styles.insert("hud".to_string(), Style {
            position: Some("fixed".into()),
            top: Some("0".into()),
            left: Some("0".into()),
            width: Some("100%".into()),
            height: Some("100%".into()),
            pointer_events: Some("none".into()),
            z_index: Some(1000),
            ..Default::default()
        });

        // Performance indicators
        styles.insert("performance-panel".to_string(), Style {
            position: Some("absolute".into()),
            top: Some(format!("{}px", self.spacing.md).into()),
            right: Some(format!("{}px", self.spacing.md).into()),
            background_color: Some(self.colors.background_overlay),
            border_radius: Some(self.spacing.sm.to_string().into()),
            padding: Some(format!("{}px", self.spacing.sm).into()),
            pointer_events: Some("auto".into()),
            box_shadow: Some(vec![BoxShadow {
                x: 0.0,
                y: 4.0,
                blur: 12.0,
                spread: 0.0,
                color: Color::rgba_u8(0, 0, 0, 0.3),
                inset: false,
            }]),
            ..Default::default()
        });

        // Build tools panel
        styles.insert("build-panel".to_string(), Style {
            position: Some("absolute".into()),
            left: Some(format!("{}px", self.spacing.md).into()),
            top: Some("50%".into()),
            transform: Some(vec!["translateY(-50%)".into()]),
            background_color: Some(self.colors.background_secondary),
            border_radius: Some(self.spacing.sm.to_string().into()),
            padding: Some(format!("{}px", self.spacing.md).into()),
            pointer_events: Some("auto".into()),
            width: Some("280px".into()),
            ..Default::default()
        });

        styles
    }
}

impl DarkColorPalette {
    pub fn new() -> Self {
        Self {
            // Primary colors - Electric blue for tech feel
            primary: Color::hex("#2563eb"),
            primary_hover: Color::hex("#1d4ed8"),
            primary_active: Color::hex("#1e40af"),
            primary_disabled: Color::hex("#64748b"),

            // Secondary colors - Neutral with slight blue tint
            secondary: Color::hex("#475569"),
            secondary_hover: Color::hex("#334155"),
            secondary_active: Color::hex("#1e293b"),

            // Background layers - Deep dark with subtle variations
            background_primary: Color::hex("#0f172a"),    // Very dark blue-gray
            background_secondary: Color::hex("#1e293b"),  // Slightly lighter
            background_tertiary: Color::hex("#334155"),   // Card backgrounds
            background_overlay: Color::rgba_u8(15, 23, 42, 0.95), // Semi-transparent

            // Text colors - High contrast for readability
            text_primary: Color::hex("#f8fafc"),
            text_secondary: Color::hex("#cbd5e1"),
            text_disabled: Color::hex("#64748b"),
            text_inverse: Color::hex("#0f172a"),

            // Border colors
            border_primary: Color::hex("#334155"),
            border_secondary: Color::hex("#475569"),
            border_focus: Color::hex("#3b82f6"),
            border_error: Color::hex("#ef4444"),

            // Status colors
            success: Color::hex("#10b981"),
            warning: Color::hex("#f59e0b"),
            error: Color::hex("#ef4444"),
            info: Color::hex("#3b82f6"),

            // Game-specific colors
            build_mode_active: Color::hex("#06d6a0"),
            material_preview: Color::rgba_u8(37, 99, 235, 0.3),
            grid_lines: Color::rgba_u8(203, 213, 225, 0.1),
            selection_highlight: Color::hex("#fbbf24"),
        }
    }
}

impl DarkTypography {
    pub fn new() -> Self {
        Self {
            font_family_primary: "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif".to_string(),
            font_family_mono: "'JetBrains Mono', 'SF Mono', Monaco, Consolas, monospace".to_string(),

            size_xs: 0.75,    // 12px
            size_sm: 0.875,   // 14px
            size_base: 1.0,   // 16px
            size_lg: 1.125,   // 18px
            size_xl: 1.25,    // 20px
            size_2xl: 1.5,    // 24px
            size_3xl: 2.0,    // 32px

            weight_light: 300,
            weight_normal: 400,
            weight_medium: 500,
            weight_semibold: 600,
            weight_bold: 700,

            line_height_tight: 1.25,
            line_height_normal: 1.5,
            line_height_relaxed: 1.75,
        }
    }
}

impl SpacingSystem {
    pub fn new() -> Self {
        Self {
            xs: 4.0,
            sm: 8.0,
            md: 16.0,
            lg: 24.0,
            xl: 32.0,
            xxl: 48.0,
            xxxl: 64.0,
        }
    }
}

impl AnimationSystem {
    pub fn new() -> Self {
        Self {
            duration_fast: 0.15,
            duration_normal: 0.25,
            duration_slow: 0.35,

            easing_ease_out: "cubic-bezier(0, 0, 0.2, 1)".to_string(),
            easing_ease_in_out: "cubic-bezier(0.4, 0, 0.2, 1)".to_string(),
            easing_bounce: "cubic-bezier(0.68, -0.55, 0.265, 1.55)".to_string(),
        }
    }
}

impl ComponentStyles {
    pub fn new() -> Self {
        let colors = DarkColorPalette::new();
        let spacing = SpacingSystem::new();
        let typography = DarkTypography::new();

        Self {
            button_primary: Style {
                background_color: Some(colors.primary),
                color: Some(colors.text_inverse),
                border: Some("none".into()),
                border_radius: Some(spacing.sm.to_string().into()),
                padding: Some(format!("{}px {}px", spacing.sm, spacing.md).into()),
                font_weight: Some(typography.weight_medium.into()),
                cursor: Some("pointer".into()),
                transition: Some(vec![Transition::new("all", 0.2, "ease-out")]),
                ..Default::default()
            },

            button_secondary: Style {
                background_color: Some(colors.secondary),
                color: Some(colors.text_primary),
                border: Some(format!("1px solid {}", colors.border_primary.to_string()).into()),
                border_radius: Some(spacing.sm.to_string().into()),
                padding: Some(format!("{}px {}px", spacing.sm, spacing.md).into()),
                font_weight: Some(typography.weight_medium.into()),
                cursor: Some("pointer".into()),
                transition: Some(vec![Transition::new("all", 0.2, "ease-out")]),
                ..Default::default()
            },

            button_ghost: Style {
                background_color: Some(Color::transparent()),
                color: Some(colors.text_secondary),
                border: Some("none".into()),
                border_radius: Some(spacing.sm.to_string().into()),
                padding: Some(format!("{}px {}px", spacing.sm, spacing.md).into()),
                font_weight: Some(typography.weight_normal.into()),
                cursor: Some("pointer".into()),
                transition: Some(vec![Transition::new("all", 0.2, "ease-out")]),
                ..Default::default()
            },

            card: Style {
                background_color: Some(colors.background_secondary),
                border: Some(format!("1px solid {}", colors.border_primary.to_string()).into()),
                border_radius: Some(spacing.md.to_string().into()),
                padding: Some(format!("{}px", spacing.lg).into()),
                box_shadow: Some(vec![BoxShadow {
                    x: 0.0,
                    y: 4.0,
                    blur: 6.0,
                    spread: -1.0,
                    color: Color::rgba_u8(0, 0, 0, 0.1),
                    inset: false,
                }]),
                ..Default::default()
            },

            input: Style {
                background_color: Some(colors.background_tertiary),
                border: Some(format!("1px solid {}", colors.border_primary.to_string()).into()),
                border_radius: Some(spacing.sm.to_string().into()),
                padding: Some(format!("{}px", spacing.sm).into()),
                color: Some(colors.text_primary),
                font_size: Some(typography.size_base),
                transition: Some(vec![Transition::new("border-color", 0.2, "ease-out")]),
                ..Default::default()
            },

            modal: Style {
                position: Some("fixed".into()),
                top: Some("0".into()),
                left: Some("0".into()),
                width: Some("100%".into()),
                height: Some("100%".into()),
                background_color: Some(colors.background_overlay),
                display: Some("flex".into()),
                align_items: Some("center".into()),
                justify_content: Some("center".into()),
                z_index: Some(9999),
                ..Default::default()
            },

            tooltip: Style {
                background_color: Some(colors.background_overlay),
                color: Some(colors.text_primary),
                border_radius: Some(spacing.xs.to_string().into()),
                padding: Some(format!("{}px {}px", spacing.xs, spacing.sm).into()),
                font_size: Some(typography.size_sm),
                box_shadow: Some(vec![BoxShadow {
                    x: 0.0,
                    y: 2.0,
                    blur: 8.0,
                    spread: 0.0,
                    color: Color::rgba_u8(0, 0, 0, 0.3),
                    inset: false,
                }]),
                z_index: Some(10000),
                ..Default::default()
            },

            menu: Style {
                background_color: Some(colors.background_secondary),
                border: Some(format!("1px solid {}", colors.border_primary.to_string()).into()),
                border_radius: Some(spacing.sm.to_string().into()),
                padding: Some(format!("{}px 0", spacing.xs).into()),
                box_shadow: Some(vec![BoxShadow {
                    x: 0.0,
                    y: 8.0,
                    blur: 16.0,
                    spread: 0.0,
                    color: Color::rgba_u8(0, 0, 0, 0.2),
                    inset: false,
                }]),
                ..Default::default()
            },
        }
    }
}

// Helper trait extensions
impl Color {
    pub fn hex(hex: &str) -> Self {
        // Parse hex color string to Color
        let hex = hex.trim_start_matches('#');
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f32 / 255.0;
        Color::new(r, g, b, 1.0)
    }

    pub fn rgba_u8(r: u8, g: u8, b: u8, a: f32) -> Self {
        Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a)
    }

    pub fn transparent() -> Self {
        Color::new(0.0, 0.0, 0.0, 0.0)
    }

    pub fn to_css_variable(&self) -> String {
        format!("{}, {}, {}, {}", self.r, self.g, self.b, self.a)
    }
}

// Extension for easier transition creation
impl Transition {
    pub fn new(property: &str, duration: f32, timing: &str) -> Self {
        // This would be implemented based on the actual Transition struct
        // For now, this is a placeholder
        Default::default()
    }
}

// Extension for easier dimension creation
impl From<&str> for crate::engine::ui::css_in_rust::Dimension {
    fn from(value: &str) -> Self {
        // Parse dimension string like "100px", "50%", "100vw"
        // Implementation would depend on the actual Dimension enum
        Default::default()
    }
}

impl From<f32> for crate::engine::ui::css_in_rust::Dimension {
    fn from(value: f32) -> Self {
        // Convert float to pixel dimension
        Default::default()
    }
}

impl From<String> for crate::engine::ui::css_in_rust::Dimension {
    fn from(value: String) -> Self {
        value.as_str().into()
    }
}