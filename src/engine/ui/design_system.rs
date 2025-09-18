// Unified Design System for Robin Engine
// Provides consistent visual language and design tokens

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::engine::ui::modern_architecture::{Color, FontStyle, FontWeight, FontVariant, Theme, ThemeColors, ThemeTypography, ThemeSpacing, ThemeBorders, ThemeShadows, ThemeAnimations, ShadowStyle};
use crate::engine::math::Vec2;

// ============================================================================
// Design System Core
// ============================================================================

/// Central design system that manages all design tokens and themes
#[derive(Debug, Clone)]
pub struct DesignSystem {
    /// Available themes
    themes: HashMap<String, Theme>,

    /// Currently active theme
    active_theme: String,

    /// Color palette
    palette: ColorPalette,

    /// Typography scale
    typography_scale: TypographyScale,

    /// Spacing scale
    spacing_scale: SpacingScale,

    /// Breakpoints for responsive design
    breakpoints: Breakpoints,

    /// Animation presets
    animation_presets: AnimationPresets,

    /// Component variants
    component_variants: ComponentVariants,
}

impl DesignSystem {
    /// Create a new design system with default settings
    pub fn new() -> Self {
        let mut system = Self {
            themes: HashMap::new(),
            active_theme: "dark".to_string(),
            palette: ColorPalette::default(),
            typography_scale: TypographyScale::default(),
            spacing_scale: SpacingScale::default(),
            breakpoints: Breakpoints::default(),
            animation_presets: AnimationPresets::default(),
            component_variants: ComponentVariants::default(),
        };

        // Register default themes
        system.register_theme("dark", Self::create_dark_theme());
        system.register_theme("light", Self::create_light_theme());
        system.register_theme("high_contrast", Self::create_high_contrast_theme());

        system
    }

    /// Register a new theme
    pub fn register_theme(&mut self, name: &str, theme: Theme) {
        self.themes.insert(name.to_string(), theme);
    }

    /// Set the active theme
    pub fn set_active_theme(&mut self, name: &str) -> Result<(), String> {
        if self.themes.contains_key(name) {
            self.active_theme = name.to_string();
            Ok(())
        } else {
            Err(format!("Theme '{}' not found", name))
        }
    }

    /// Get the active theme
    pub fn get_active_theme(&self) -> &Theme {
        self.themes.get(&self.active_theme)
            .expect("Active theme should always exist")
    }

    /// Get a specific theme
    pub fn get_theme(&self, name: &str) -> Option<&Theme> {
        self.themes.get(name)
    }

    /// Get the active theme name
    pub fn get_active_theme_name(&self) -> &str {
        &self.active_theme
    }

    /// Get the color palette
    pub fn get_palette(&self) -> &ColorPalette {
        &self.palette
    }

    /// Create the dark theme
    fn create_dark_theme() -> Theme {
        Theme {
            name: "Dark".to_string(),
            colors: ThemeColors {
                primary: Color::from_hex("#007AFF").unwrap(),
                secondary: Color::from_hex("#5AC8FA").unwrap(),
                accent: Color::from_hex("#FF9500").unwrap(),
                background: Color::from_hex("#000000").unwrap(),
                surface: Color::from_hex("#1C1C1E").unwrap(),
                text: Color::from_hex("#FFFFFF").unwrap(),
                text_secondary: Color::from_hex("#8E8E93").unwrap(),
                error: Color::from_hex("#FF3B30").unwrap(),
                warning: Color::from_hex("#FF9500").unwrap(),
                success: Color::from_hex("#34C759").unwrap(),
                info: Color::from_hex("#5AC8FA").unwrap(),
            },
            typography: ThemeTypography {
                font_family: "Inter".to_string(),
                heading1: FontStyle {
                    family: "Inter".to_string(),
                    size: 32.0,
                    weight: FontWeight::Bold,
                    style: FontVariant::Normal,
                },
                heading2: FontStyle {
                    family: "Inter".to_string(),
                    size: 24.0,
                    weight: FontWeight::Bold,
                    style: FontVariant::Normal,
                },
                heading3: FontStyle {
                    family: "Inter".to_string(),
                    size: 20.0,
                    weight: FontWeight::Medium,
                    style: FontVariant::Normal,
                },
                body: FontStyle {
                    family: "Inter".to_string(),
                    size: 16.0,
                    weight: FontWeight::Regular,
                    style: FontVariant::Normal,
                },
                caption: FontStyle {
                    family: "Inter".to_string(),
                    size: 12.0,
                    weight: FontWeight::Regular,
                    style: FontVariant::Normal,
                },
                button: FontStyle {
                    family: "Inter".to_string(),
                    size: 16.0,
                    weight: FontWeight::Medium,
                    style: FontVariant::Normal,
                },
            },
            spacing: ThemeSpacing {
                xs: 4.0,
                sm: 8.0,
                md: 16.0,
                lg: 24.0,
                xl: 32.0,
                xxl: 48.0,
            },
            borders: ThemeBorders {
                radius_sm: 4.0,
                radius_md: 8.0,
                radius_lg: 16.0,
                width_thin: 1.0,
                width_medium: 2.0,
                width_thick: 4.0,
            },
            shadows: ThemeShadows {
                sm: ShadowStyle {
                    offset: Vec2::new(0.0, 1.0),
                    blur: 2.0,
                    spread: 0.0,
                    color: Color::new(0.0, 0.0, 0.0, 0.1),
                },
                md: ShadowStyle {
                    offset: Vec2::new(0.0, 2.0),
                    blur: 4.0,
                    spread: 0.0,
                    color: Color::new(0.0, 0.0, 0.0, 0.15),
                },
                lg: ShadowStyle {
                    offset: Vec2::new(0.0, 4.0),
                    blur: 8.0,
                    spread: 0.0,
                    color: Color::new(0.0, 0.0, 0.0, 0.2),
                },
                xl: ShadowStyle {
                    offset: Vec2::new(0.0, 8.0),
                    blur: 16.0,
                    spread: 0.0,
                    color: Color::new(0.0, 0.0, 0.0, 0.25),
                },
            },
            animations: ThemeAnimations {
                duration_fast: 0.15,
                duration_normal: 0.3,
                duration_slow: 0.5,
                easing_linear: "linear".to_string(),
                easing_ease_in: "ease-in".to_string(),
                easing_ease_out: "ease-out".to_string(),
                easing_ease_in_out: "ease-in-out".to_string(),
                easing_spring: "spring".to_string(),
            },
        }
    }

    /// Create the light theme
    fn create_light_theme() -> Theme {
        Theme {
            name: "Light".to_string(),
            colors: ThemeColors {
                primary: Color::from_hex("#007AFF").unwrap(),
                secondary: Color::from_hex("#5AC8FA").unwrap(),
                accent: Color::from_hex("#FF9500").unwrap(),
                background: Color::from_hex("#FFFFFF").unwrap(),
                surface: Color::from_hex("#F2F2F7").unwrap(),
                text: Color::from_hex("#000000").unwrap(),
                text_secondary: Color::from_hex("#6C6C70").unwrap(),
                error: Color::from_hex("#FF3B30").unwrap(),
                warning: Color::from_hex("#FF9500").unwrap(),
                success: Color::from_hex("#34C759").unwrap(),
                info: Color::from_hex("#5AC8FA").unwrap(),
            },
            typography: ThemeTypography {
                font_family: "Inter".to_string(),
                heading1: FontStyle {
                    family: "Inter".to_string(),
                    size: 32.0,
                    weight: FontWeight::Bold,
                    style: FontVariant::Normal,
                },
                heading2: FontStyle {
                    family: "Inter".to_string(),
                    size: 24.0,
                    weight: FontWeight::Bold,
                    style: FontVariant::Normal,
                },
                heading3: FontStyle {
                    family: "Inter".to_string(),
                    size: 20.0,
                    weight: FontWeight::Medium,
                    style: FontVariant::Normal,
                },
                body: FontStyle {
                    family: "Inter".to_string(),
                    size: 16.0,
                    weight: FontWeight::Regular,
                    style: FontVariant::Normal,
                },
                caption: FontStyle {
                    family: "Inter".to_string(),
                    size: 12.0,
                    weight: FontWeight::Regular,
                    style: FontVariant::Normal,
                },
                button: FontStyle {
                    family: "Inter".to_string(),
                    size: 16.0,
                    weight: FontWeight::Medium,
                    style: FontVariant::Normal,
                },
            },
            spacing: ThemeSpacing {
                xs: 4.0,
                sm: 8.0,
                md: 16.0,
                lg: 24.0,
                xl: 32.0,
                xxl: 48.0,
            },
            borders: ThemeBorders {
                radius_sm: 4.0,
                radius_md: 8.0,
                radius_lg: 16.0,
                width_thin: 1.0,
                width_medium: 2.0,
                width_thick: 4.0,
            },
            shadows: ThemeShadows {
                sm: ShadowStyle {
                    offset: Vec2::new(0.0, 1.0),
                    blur: 2.0,
                    spread: 0.0,
                    color: Color::new(0.0, 0.0, 0.0, 0.05),
                },
                md: ShadowStyle {
                    offset: Vec2::new(0.0, 2.0),
                    blur: 4.0,
                    spread: 0.0,
                    color: Color::new(0.0, 0.0, 0.0, 0.1),
                },
                lg: ShadowStyle {
                    offset: Vec2::new(0.0, 4.0),
                    blur: 8.0,
                    spread: 0.0,
                    color: Color::new(0.0, 0.0, 0.0, 0.15),
                },
                xl: ShadowStyle {
                    offset: Vec2::new(0.0, 8.0),
                    blur: 16.0,
                    spread: 0.0,
                    color: Color::new(0.0, 0.0, 0.0, 0.2),
                },
            },
            animations: ThemeAnimations {
                duration_fast: 0.15,
                duration_normal: 0.3,
                duration_slow: 0.5,
                easing_linear: "linear".to_string(),
                easing_ease_in: "ease-in".to_string(),
                easing_ease_out: "ease-out".to_string(),
                easing_ease_in_out: "ease-in-out".to_string(),
                easing_spring: "spring".to_string(),
            },
        }
    }

    /// Create the high contrast theme for accessibility
    fn create_high_contrast_theme() -> Theme {
        Theme {
            name: "High Contrast".to_string(),
            colors: ThemeColors {
                primary: Color::from_hex("#00FF00").unwrap(),
                secondary: Color::from_hex("#00FFFF").unwrap(),
                accent: Color::from_hex("#FFFF00").unwrap(),
                background: Color::from_hex("#000000").unwrap(),
                surface: Color::from_hex("#1A1A1A").unwrap(),
                text: Color::from_hex("#FFFFFF").unwrap(),
                text_secondary: Color::from_hex("#CCCCCC").unwrap(),
                error: Color::from_hex("#FF0000").unwrap(),
                warning: Color::from_hex("#FFAA00").unwrap(),
                success: Color::from_hex("#00FF00").unwrap(),
                info: Color::from_hex("#00FFFF").unwrap(),
            },
            typography: ThemeTypography {
                font_family: "Inter".to_string(),
                heading1: FontStyle {
                    family: "Inter".to_string(),
                    size: 34.0,
                    weight: FontWeight::Black,
                    style: FontVariant::Normal,
                },
                heading2: FontStyle {
                    family: "Inter".to_string(),
                    size: 26.0,
                    weight: FontWeight::Black,
                    style: FontVariant::Normal,
                },
                heading3: FontStyle {
                    family: "Inter".to_string(),
                    size: 22.0,
                    weight: FontWeight::Bold,
                    style: FontVariant::Normal,
                },
                body: FontStyle {
                    family: "Inter".to_string(),
                    size: 18.0,
                    weight: FontWeight::Medium,
                    style: FontVariant::Normal,
                },
                caption: FontStyle {
                    family: "Inter".to_string(),
                    size: 14.0,
                    weight: FontWeight::Medium,
                    style: FontVariant::Normal,
                },
                button: FontStyle {
                    family: "Inter".to_string(),
                    size: 18.0,
                    weight: FontWeight::Bold,
                    style: FontVariant::Normal,
                },
            },
            spacing: ThemeSpacing {
                xs: 6.0,
                sm: 10.0,
                md: 18.0,
                lg: 26.0,
                xl: 34.0,
                xxl: 50.0,
            },
            borders: ThemeBorders {
                radius_sm: 2.0,
                radius_md: 4.0,
                radius_lg: 8.0,
                width_thin: 2.0,
                width_medium: 3.0,
                width_thick: 5.0,
            },
            shadows: ThemeShadows {
                sm: ShadowStyle {
                    offset: Vec2::new(0.0, 0.0),
                    blur: 0.0,
                    spread: 2.0,
                    color: Color::from_hex("#FFFFFF").unwrap(),
                },
                md: ShadowStyle {
                    offset: Vec2::new(0.0, 0.0),
                    blur: 0.0,
                    spread: 3.0,
                    color: Color::from_hex("#FFFFFF").unwrap(),
                },
                lg: ShadowStyle {
                    offset: Vec2::new(0.0, 0.0),
                    blur: 0.0,
                    spread: 4.0,
                    color: Color::from_hex("#FFFFFF").unwrap(),
                },
                xl: ShadowStyle {
                    offset: Vec2::new(0.0, 0.0),
                    blur: 0.0,
                    spread: 5.0,
                    color: Color::from_hex("#FFFFFF").unwrap(),
                },
            },
            animations: ThemeAnimations {
                duration_fast: 0.1,
                duration_normal: 0.2,
                duration_slow: 0.3,
                easing_linear: "linear".to_string(),
                easing_ease_in: "ease-in".to_string(),
                easing_ease_out: "ease-out".to_string(),
                easing_ease_in_out: "ease-in-out".to_string(),
                easing_spring: "spring".to_string(),
            },
        }
    }
}

// ============================================================================
// Color Palette
// ============================================================================

/// Comprehensive color palette with semantic colors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPalette {
    // Brand colors
    pub brand_primary: Color,
    pub brand_secondary: Color,
    pub brand_tertiary: Color,

    // Neutral colors
    pub gray_50: Color,
    pub gray_100: Color,
    pub gray_200: Color,
    pub gray_300: Color,
    pub gray_400: Color,
    pub gray_500: Color,
    pub gray_600: Color,
    pub gray_700: Color,
    pub gray_800: Color,
    pub gray_900: Color,

    // Semantic colors
    pub success_light: Color,
    pub success_main: Color,
    pub success_dark: Color,

    pub warning_light: Color,
    pub warning_main: Color,
    pub warning_dark: Color,

    pub error_light: Color,
    pub error_main: Color,
    pub error_dark: Color,

    pub info_light: Color,
    pub info_main: Color,
    pub info_dark: Color,
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self {
            brand_primary: Color::from_hex("#007AFF").unwrap(),
            brand_secondary: Color::from_hex("#5AC8FA").unwrap(),
            brand_tertiary: Color::from_hex("#FF9500").unwrap(),

            gray_50: Color::from_hex("#FAFAFA").unwrap(),
            gray_100: Color::from_hex("#F5F5F5").unwrap(),
            gray_200: Color::from_hex("#E5E5E5").unwrap(),
            gray_300: Color::from_hex("#D4D4D4").unwrap(),
            gray_400: Color::from_hex("#A3A3A3").unwrap(),
            gray_500: Color::from_hex("#737373").unwrap(),
            gray_600: Color::from_hex("#525252").unwrap(),
            gray_700: Color::from_hex("#404040").unwrap(),
            gray_800: Color::from_hex("#262626").unwrap(),
            gray_900: Color::from_hex("#171717").unwrap(),

            success_light: Color::from_hex("#86EFAC").unwrap(),
            success_main: Color::from_hex("#34C759").unwrap(),
            success_dark: Color::from_hex("#16A34A").unwrap(),

            warning_light: Color::from_hex("#FED7AA").unwrap(),
            warning_main: Color::from_hex("#FF9500").unwrap(),
            warning_dark: Color::from_hex("#EA580C").unwrap(),

            error_light: Color::from_hex("#FCA5A5").unwrap(),
            error_main: Color::from_hex("#FF3B30").unwrap(),
            error_dark: Color::from_hex("#DC2626").unwrap(),

            info_light: Color::from_hex("#93C5FD").unwrap(),
            info_main: Color::from_hex("#5AC8FA").unwrap(),
            info_dark: Color::from_hex("#2563EB").unwrap(),
        }
    }
}

// ============================================================================
// Typography Scale
// ============================================================================

/// Typography scale system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypographyScale {
    pub scale_ratio: f32,
    pub base_size: f32,
    pub font_families: FontFamilies,
    pub line_heights: LineHeights,
    pub letter_spacings: LetterSpacings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontFamilies {
    pub sans_serif: String,
    pub serif: String,
    pub monospace: String,
    pub display: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineHeights {
    pub tight: f32,
    pub normal: f32,
    pub relaxed: f32,
    pub loose: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LetterSpacings {
    pub tight: f32,
    pub normal: f32,
    pub wide: f32,
}

impl Default for TypographyScale {
    fn default() -> Self {
        Self {
            scale_ratio: 1.25,
            base_size: 16.0,
            font_families: FontFamilies {
                sans_serif: "Inter, system-ui, sans-serif".to_string(),
                serif: "Georgia, serif".to_string(),
                monospace: "JetBrains Mono, monospace".to_string(),
                display: "Inter Display, sans-serif".to_string(),
            },
            line_heights: LineHeights {
                tight: 1.25,
                normal: 1.5,
                relaxed: 1.75,
                loose: 2.0,
            },
            letter_spacings: LetterSpacings {
                tight: -0.02,
                normal: 0.0,
                wide: 0.02,
            },
        }
    }
}

// ============================================================================
// Spacing Scale
// ============================================================================

/// Spacing scale system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpacingScale {
    pub base_unit: f32,
    pub scale: Vec<f32>,
}

impl Default for SpacingScale {
    fn default() -> Self {
        Self {
            base_unit: 8.0,
            scale: vec![
                0.0,   // 0
                0.5,   // 4px
                1.0,   // 8px
                1.5,   // 12px
                2.0,   // 16px
                2.5,   // 20px
                3.0,   // 24px
                4.0,   // 32px
                5.0,   // 40px
                6.0,   // 48px
                8.0,   // 64px
                10.0,  // 80px
                12.0,  // 96px
                16.0,  // 128px
                20.0,  // 160px
                24.0,  // 192px
            ],
        }
    }
}

impl SpacingScale {
    pub fn get(&self, index: usize) -> f32 {
        self.scale.get(index).copied().unwrap_or(0.0) * self.base_unit
    }
}

// ============================================================================
// Breakpoints
// ============================================================================

/// Breakpoints for responsive design
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoints {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub xxl: f32,
}

impl Default for Breakpoints {
    fn default() -> Self {
        Self {
            xs: 0.0,
            sm: 576.0,
            md: 768.0,
            lg: 992.0,
            xl: 1200.0,
            xxl: 1600.0,
        }
    }
}

impl Breakpoints {
    pub fn get_current(&self, width: f32) -> &str {
        if width >= self.xxl {
            "xxl"
        } else if width >= self.xl {
            "xl"
        } else if width >= self.lg {
            "lg"
        } else if width >= self.md {
            "md"
        } else if width >= self.sm {
            "sm"
        } else {
            "xs"
        }
    }
}

// ============================================================================
// Animation Presets
// ============================================================================

/// Predefined animation presets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationPresets {
    pub fade_in: AnimationPreset,
    pub fade_out: AnimationPreset,
    pub slide_in_left: AnimationPreset,
    pub slide_in_right: AnimationPreset,
    pub slide_in_top: AnimationPreset,
    pub slide_in_bottom: AnimationPreset,
    pub scale_in: AnimationPreset,
    pub scale_out: AnimationPreset,
    pub rotate_in: AnimationPreset,
    pub bounce: AnimationPreset,
    pub shake: AnimationPreset,
    pub pulse: AnimationPreset,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationPreset {
    pub duration: f32,
    pub easing: String,
    pub delay: f32,
    pub iterations: u32,
    pub fill_mode: String,
    pub keyframes: Vec<AnimationKeyframe>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationKeyframe {
    pub offset: f32,
    pub properties: HashMap<String, AnimationValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnimationValue {
    Number(f32),
    Color(Color),
    Transform(String),
}

impl Default for AnimationPresets {
    fn default() -> Self {
        Self {
            fade_in: AnimationPreset {
                duration: 0.3,
                easing: "ease-out".to_string(),
                delay: 0.0,
                iterations: 1,
                fill_mode: "forwards".to_string(),
                keyframes: vec![
                    AnimationKeyframe {
                        offset: 0.0,
                        properties: {
                            let mut props = HashMap::new();
                            props.insert("opacity".to_string(), AnimationValue::Number(0.0));
                            props
                        },
                    },
                    AnimationKeyframe {
                        offset: 1.0,
                        properties: {
                            let mut props = HashMap::new();
                            props.insert("opacity".to_string(), AnimationValue::Number(1.0));
                            props
                        },
                    },
                ],
            },
            fade_out: AnimationPreset {
                duration: 0.3,
                easing: "ease-in".to_string(),
                delay: 0.0,
                iterations: 1,
                fill_mode: "forwards".to_string(),
                keyframes: vec![
                    AnimationKeyframe {
                        offset: 0.0,
                        properties: {
                            let mut props = HashMap::new();
                            props.insert("opacity".to_string(), AnimationValue::Number(1.0));
                            props
                        },
                    },
                    AnimationKeyframe {
                        offset: 1.0,
                        properties: {
                            let mut props = HashMap::new();
                            props.insert("opacity".to_string(), AnimationValue::Number(0.0));
                            props
                        },
                    },
                ],
            },
            slide_in_left: AnimationPreset {
                duration: 0.3,
                easing: "ease-out".to_string(),
                delay: 0.0,
                iterations: 1,
                fill_mode: "forwards".to_string(),
                keyframes: vec![
                    AnimationKeyframe {
                        offset: 0.0,
                        properties: {
                            let mut props = HashMap::new();
                            props.insert("transform".to_string(), AnimationValue::Transform("translateX(-100%)".to_string()));
                            props.insert("opacity".to_string(), AnimationValue::Number(0.0));
                            props
                        },
                    },
                    AnimationKeyframe {
                        offset: 1.0,
                        properties: {
                            let mut props = HashMap::new();
                            props.insert("transform".to_string(), AnimationValue::Transform("translateX(0)".to_string()));
                            props.insert("opacity".to_string(), AnimationValue::Number(1.0));
                            props
                        },
                    },
                ],
            },
            slide_in_right: AnimationPreset {
                duration: 0.3,
                easing: "ease-out".to_string(),
                delay: 0.0,
                iterations: 1,
                fill_mode: "forwards".to_string(),
                keyframes: vec![
                    AnimationKeyframe {
                        offset: 0.0,
                        properties: {
                            let mut props = HashMap::new();
                            props.insert("transform".to_string(), AnimationValue::Transform("translateX(100%)".to_string()));
                            props.insert("opacity".to_string(), AnimationValue::Number(0.0));
                            props
                        },
                    },
                    AnimationKeyframe {
                        offset: 1.0,
                        properties: {
                            let mut props = HashMap::new();
                            props.insert("transform".to_string(), AnimationValue::Transform("translateX(0)".to_string()));
                            props.insert("opacity".to_string(), AnimationValue::Number(1.0));
                            props
                        },
                    },
                ],
            },
            slide_in_top: AnimationPreset {
                duration: 0.3,
                easing: "ease-out".to_string(),
                delay: 0.0,
                iterations: 1,
                fill_mode: "forwards".to_string(),
                keyframes: vec![
                    AnimationKeyframe {
                        offset: 0.0,
                        properties: {
                            let mut props = HashMap::new();
                            props.insert("transform".to_string(), AnimationValue::Transform("translateY(-100%)".to_string()));
                            props.insert("opacity".to_string(), AnimationValue::Number(0.0));
                            props
                        },
                    },
                    AnimationKeyframe {
                        offset: 1.0,
                        properties: {
                            let mut props = HashMap::new();
                            props.insert("transform".to_string(), AnimationValue::Transform("translateY(0)".to_string()));
                            props.insert("opacity".to_string(), AnimationValue::Number(1.0));
                            props
                        },
                    },
                ],
            },
            slide_in_bottom: AnimationPreset {
                duration: 0.3,
                easing: "ease-out".to_string(),
                delay: 0.0,
                iterations: 1,
                fill_mode: "forwards".to_string(),
                keyframes: vec![
                    AnimationKeyframe {
                        offset: 0.0,
                        properties: {
                            let mut props = HashMap::new();
                            props.insert("transform".to_string(), AnimationValue::Transform("translateY(100%)".to_string()));
                            props.insert("opacity".to_string(), AnimationValue::Number(0.0));
                            props
                        },
                    },
                    AnimationKeyframe {
                        offset: 1.0,
                        properties: {
                            let mut props = HashMap::new();
                            props.insert("transform".to_string(), AnimationValue::Transform("translateY(0)".to_string()));
                            props.insert("opacity".to_string(), AnimationValue::Number(1.0));
                            props
                        },
                    },
                ],
            },
            scale_in: AnimationPreset {
                duration: 0.3,
                easing: "ease-out".to_string(),
                delay: 0.0,
                iterations: 1,
                fill_mode: "forwards".to_string(),
                keyframes: vec![
                    AnimationKeyframe {
                        offset: 0.0,
                        properties: {
                            let mut props = HashMap::new();
                            props.insert("transform".to_string(), AnimationValue::Transform("scale(0)".to_string()));
                            props.insert("opacity".to_string(), AnimationValue::Number(0.0));
                            props
                        },
                    },
                    AnimationKeyframe {
                        offset: 1.0,
                        properties: {
                            let mut props = HashMap::new();
                            props.insert("transform".to_string(), AnimationValue::Transform("scale(1)".to_string()));
                            props.insert("opacity".to_string(), AnimationValue::Number(1.0));
                            props
                        },
                    },
                ],
            },
            scale_out: AnimationPreset {
                duration: 0.3,
                easing: "ease-in".to_string(),
                delay: 0.0,
                iterations: 1,
                fill_mode: "forwards".to_string(),
                keyframes: vec![
                    AnimationKeyframe {
                        offset: 0.0,
                        properties: {
                            let mut props = HashMap::new();
                            props.insert("transform".to_string(), AnimationValue::Transform("scale(1)".to_string()));
                            props.insert("opacity".to_string(), AnimationValue::Number(1.0));
                            props
                        },
                    },
                    AnimationKeyframe {
                        offset: 1.0,
                        properties: {
                            let mut props = HashMap::new();
                            props.insert("transform".to_string(), AnimationValue::Transform("scale(0)".to_string()));
                            props.insert("opacity".to_string(), AnimationValue::Number(0.0));
                            props
                        },
                    },
                ],
            },
            rotate_in: AnimationPreset {
                duration: 0.5,
                easing: "ease-out".to_string(),
                delay: 0.0,
                iterations: 1,
                fill_mode: "forwards".to_string(),
                keyframes: vec![
                    AnimationKeyframe {
                        offset: 0.0,
                        properties: {
                            let mut props = HashMap::new();
                            props.insert("transform".to_string(), AnimationValue::Transform("rotate(-180deg) scale(0)".to_string()));
                            props.insert("opacity".to_string(), AnimationValue::Number(0.0));
                            props
                        },
                    },
                    AnimationKeyframe {
                        offset: 1.0,
                        properties: {
                            let mut props = HashMap::new();
                            props.insert("transform".to_string(), AnimationValue::Transform("rotate(0) scale(1)".to_string()));
                            props.insert("opacity".to_string(), AnimationValue::Number(1.0));
                            props
                        },
                    },
                ],
            },
            bounce: AnimationPreset {
                duration: 0.6,
                easing: "ease-out".to_string(),
                delay: 0.0,
                iterations: 1,
                fill_mode: "forwards".to_string(),
                keyframes: vec![
                    AnimationKeyframe {
                        offset: 0.0,
                        properties: {
                            let mut props = HashMap::new();
                            props.insert("transform".to_string(), AnimationValue::Transform("scale(1)".to_string()));
                            props
                        },
                    },
                    AnimationKeyframe {
                        offset: 0.25,
                        properties: {
                            let mut props = HashMap::new();
                            props.insert("transform".to_string(), AnimationValue::Transform("scale(0.95)".to_string()));
                            props
                        },
                    },
                    AnimationKeyframe {
                        offset: 0.5,
                        properties: {
                            let mut props = HashMap::new();
                            props.insert("transform".to_string(), AnimationValue::Transform("scale(1.05)".to_string()));
                            props
                        },
                    },
                    AnimationKeyframe {
                        offset: 0.75,
                        properties: {
                            let mut props = HashMap::new();
                            props.insert("transform".to_string(), AnimationValue::Transform("scale(0.98)".to_string()));
                            props
                        },
                    },
                    AnimationKeyframe {
                        offset: 1.0,
                        properties: {
                            let mut props = HashMap::new();
                            props.insert("transform".to_string(), AnimationValue::Transform("scale(1)".to_string()));
                            props
                        },
                    },
                ],
            },
            shake: AnimationPreset {
                duration: 0.5,
                easing: "linear".to_string(),
                delay: 0.0,
                iterations: 1,
                fill_mode: "forwards".to_string(),
                keyframes: vec![
                    AnimationKeyframe {
                        offset: 0.0,
                        properties: {
                            let mut props = HashMap::new();
                            props.insert("transform".to_string(), AnimationValue::Transform("translateX(0)".to_string()));
                            props
                        },
                    },
                    AnimationKeyframe {
                        offset: 0.1,
                        properties: {
                            let mut props = HashMap::new();
                            props.insert("transform".to_string(), AnimationValue::Transform("translateX(-10px)".to_string()));
                            props
                        },
                    },
                    AnimationKeyframe {
                        offset: 0.2,
                        properties: {
                            let mut props = HashMap::new();
                            props.insert("transform".to_string(), AnimationValue::Transform("translateX(10px)".to_string()));
                            props
                        },
                    },
                    AnimationKeyframe {
                        offset: 0.3,
                        properties: {
                            let mut props = HashMap::new();
                            props.insert("transform".to_string(), AnimationValue::Transform("translateX(-10px)".to_string()));
                            props
                        },
                    },
                    AnimationKeyframe {
                        offset: 0.4,
                        properties: {
                            let mut props = HashMap::new();
                            props.insert("transform".to_string(), AnimationValue::Transform("translateX(10px)".to_string()));
                            props
                        },
                    },
                    AnimationKeyframe {
                        offset: 0.5,
                        properties: {
                            let mut props = HashMap::new();
                            props.insert("transform".to_string(), AnimationValue::Transform("translateX(0)".to_string()));
                            props
                        },
                    },
                ],
            },
            pulse: AnimationPreset {
                duration: 1.0,
                easing: "ease-in-out".to_string(),
                delay: 0.0,
                iterations: 0, // Infinite
                fill_mode: "forwards".to_string(),
                keyframes: vec![
                    AnimationKeyframe {
                        offset: 0.0,
                        properties: {
                            let mut props = HashMap::new();
                            props.insert("opacity".to_string(), AnimationValue::Number(1.0));
                            props
                        },
                    },
                    AnimationKeyframe {
                        offset: 0.5,
                        properties: {
                            let mut props = HashMap::new();
                            props.insert("opacity".to_string(), AnimationValue::Number(0.5));
                            props
                        },
                    },
                    AnimationKeyframe {
                        offset: 1.0,
                        properties: {
                            let mut props = HashMap::new();
                            props.insert("opacity".to_string(), AnimationValue::Number(1.0));
                            props
                        },
                    },
                ],
            },
        }
    }
}

// ============================================================================
// Component Variants
// ============================================================================

/// Predefined component variants for consistency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentVariants {
    pub button_variants: ButtonVariants,
    pub card_variants: CardVariants,
    pub input_variants: InputVariants,
    pub badge_variants: BadgeVariants,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonVariants {
    pub primary: String,
    pub secondary: String,
    pub tertiary: String,
    pub danger: String,
    pub success: String,
    pub ghost: String,
    pub link: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardVariants {
    pub default: String,
    pub outlined: String,
    pub elevated: String,
    pub filled: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputVariants {
    pub default: String,
    pub outlined: String,
    pub filled: String,
    pub underlined: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BadgeVariants {
    pub default: String,
    pub primary: String,
    pub secondary: String,
    pub success: String,
    pub warning: String,
    pub error: String,
    pub info: String,
}

impl Default for ComponentVariants {
    fn default() -> Self {
        Self {
            button_variants: ButtonVariants {
                primary: "primary".to_string(),
                secondary: "secondary".to_string(),
                tertiary: "tertiary".to_string(),
                danger: "danger".to_string(),
                success: "success".to_string(),
                ghost: "ghost".to_string(),
                link: "link".to_string(),
            },
            card_variants: CardVariants {
                default: "default".to_string(),
                outlined: "outlined".to_string(),
                elevated: "elevated".to_string(),
                filled: "filled".to_string(),
            },
            input_variants: InputVariants {
                default: "default".to_string(),
                outlined: "outlined".to_string(),
                filled: "filled".to_string(),
                underlined: "underlined".to_string(),
            },
            badge_variants: BadgeVariants {
                default: "default".to_string(),
                primary: "primary".to_string(),
                secondary: "secondary".to_string(),
                success: "success".to_string(),
                warning: "warning".to_string(),
                error: "error".to_string(),
                info: "info".to_string(),
            },
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_design_system_creation() {
        let design_system = DesignSystem::new();
        assert!(design_system.themes.contains_key("dark"));
        assert!(design_system.themes.contains_key("light"));
        assert!(design_system.themes.contains_key("high_contrast"));
    }

    #[test]
    fn test_theme_switching() {
        let mut design_system = DesignSystem::new();
        assert_eq!(design_system.active_theme, "dark");

        design_system.set_active_theme("light").unwrap();
        assert_eq!(design_system.active_theme, "light");

        let result = design_system.set_active_theme("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_breakpoint_detection() {
        let breakpoints = Breakpoints::default();
        assert_eq!(breakpoints.get_current(320.0), "xs");
        assert_eq!(breakpoints.get_current(600.0), "sm");
        assert_eq!(breakpoints.get_current(800.0), "md");
        assert_eq!(breakpoints.get_current(1000.0), "lg");
        assert_eq!(breakpoints.get_current(1300.0), "xl");
        assert_eq!(breakpoints.get_current(1700.0), "xxl");
    }

    #[test]
    fn test_spacing_scale() {
        let spacing = SpacingScale::default();
        assert_eq!(spacing.get(0), 0.0);
        assert_eq!(spacing.get(1), 4.0);
        assert_eq!(spacing.get(2), 8.0);
        assert_eq!(spacing.get(4), 16.0);
    }
}