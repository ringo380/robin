use crate::engine::ui::UIState;
use serde::{Serialize, Deserialize};

/// Color representation for UI elements
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const TRANSPARENT: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
    pub const RED: Color = Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const GREEN: Color = Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const BLUE: Color = Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };
    pub const GRAY: Color = Color { r: 0.5, g: 0.5, b: 0.5, a: 1.0 };
    pub const LIGHT_GRAY: Color = Color { r: 0.8, g: 0.8, b: 0.8, a: 1.0 };
    pub const DARK_GRAY: Color = Color { r: 0.3, g: 0.3, b: 0.3, a: 1.0 };

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_hex(hex: u32) -> Self {
        let r = ((hex >> 16) & 0xFF) as f32 / 255.0;
        let g = ((hex >> 8) & 0xFF) as f32 / 255.0;
        let b = (hex & 0xFF) as f32 / 255.0;
        Self { r, g, b, a: 1.0 }
    }

    pub fn lerp(&self, other: &Color, t: f32) -> Color {
        Color {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
            a: self.a + (other.a - self.a) * t,
        }
    }

    pub fn with_alpha(&self, alpha: f32) -> Color {
        Color {
            r: self.r,
            g: self.g,
            b: self.b,
            a: alpha,
        }
    }
}

/// Margin and padding values
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spacing {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Spacing {
    pub const ZERO: Spacing = Spacing { top: 0.0, right: 0.0, bottom: 0.0, left: 0.0 };

    pub fn all(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    pub fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self { top, right, bottom, left }
    }
}

/// Border style types
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BorderStyle {
    None,
    Solid,
    Dashed,
    Dotted,
    Double,
    Groove,
    Ridge,
    Inset,
    Outset,
}

impl Default for BorderStyle {
    fn default() -> Self {
        BorderStyle::Solid
    }
}

/// Border styling
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Border {
    pub width: f32,
    pub color: Color,
    pub radius: f32,
}

impl Border {
    pub const NONE: Border = Border {
        width: 0.0,
        color: Color::TRANSPARENT,
        radius: 0.0,
    };

    pub fn new(width: f32, color: Color) -> Self {
        Self {
            width,
            color,
            radius: 0.0,
        }
    }

    pub fn with_radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }
}

/// Typography settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Typography {
    pub font_size: f32,
    pub color: Color,
    pub font_family: String,
    pub bold: bool,
    pub italic: bool,
    pub line_height: f32,
    pub text_align: TextAlign,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

impl Default for Typography {
    fn default() -> Self {
        Self {
            font_size: 14.0,
            color: Color::BLACK,
            font_family: "Arial".to_string(),
            bold: false,
            italic: false,
            line_height: 1.2,
            text_align: TextAlign::Left,
        }
    }
}

/// Text decoration styles
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TextDecoration {
    None,
    Underline,
    Overline,
    LineThrough,
    Blink,
}

impl Default for TextDecoration {
    fn default() -> Self {
        TextDecoration::None
    }
}

/// Flexbox alignment for cross axis
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AlignItems {
    Stretch,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
}

impl Default for AlignItems {
    fn default() -> Self {
        AlignItems::Stretch
    }
}

/// Flexbox alignment for main axis
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum JustifyContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

impl Default for JustifyContent {
    fn default() -> Self {
        JustifyContent::FlexStart
    }
}

/// Flexbox direction
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FlexDirection {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

impl Default for FlexDirection {
    fn default() -> Self {
        FlexDirection::Row
    }
}

/// Box sizing model
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BoxSizing {
    ContentBox,
    BorderBox,
}

impl Default for BoxSizing {
    fn default() -> Self {
        BoxSizing::ContentBox
    }
}

/// Resize behavior
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Resize {
    None,
    Both,
    Horizontal,
    Vertical,
}

impl Default for Resize {
    fn default() -> Self {
        Resize::None
    }
}

/// Complete UI element style
#[derive(Debug, Clone)]
pub struct UIStyle {
    pub background_color: Color,
    pub border: Border,
    pub margin: Spacing,
    pub padding: Spacing,
    pub typography: Typography,
    pub opacity: f32,
    pub visible: bool,
    pub shadow_offset: (f32, f32),
    pub shadow_color: Color,
    pub shadow_blur: f32,
}

impl Default for UIStyle {
    fn default() -> Self {
        Self {
            background_color: Color::TRANSPARENT,
            border: Border::NONE,
            margin: Spacing::ZERO,
            padding: Spacing::all(4.0),
            typography: Typography::default(),
            opacity: 1.0,
            visible: true,
            shadow_offset: (0.0, 0.0),
            shadow_color: Color::TRANSPARENT,
            shadow_blur: 0.0,
        }
    }
}

/// State-based styling (different styles for different UI states)
#[derive(Debug, Clone)]
pub struct StateStyle {
    pub normal: UIStyle,
    pub hovered: UIStyle,
    pub pressed: UIStyle,
    pub focused: UIStyle,
    pub disabled: UIStyle,
}

impl StateStyle {
    pub fn new(base_style: UIStyle) -> Self {
        Self {
            normal: base_style.clone(),
            hovered: base_style.clone(),
            pressed: base_style.clone(),
            focused: base_style.clone(),
            disabled: base_style,
        }
    }

    pub fn get_style(&self, state: UIState) -> &UIStyle {
        match state {
            UIState::Normal => &self.normal,
            UIState::Hovered => &self.hovered,
            UIState::Pressed => &self.pressed,
            UIState::Focused => &self.focused,
            UIState::Disabled => &self.disabled,
        }
    }

    pub fn get_style_mut(&mut self, state: UIState) -> &mut UIStyle {
        match state {
            UIState::Normal => &mut self.normal,
            UIState::Hovered => &mut self.hovered,
            UIState::Pressed => &mut self.pressed,
            UIState::Focused => &mut self.focused,
            UIState::Disabled => &mut self.disabled,
        }
    }
}

/// Modern design system color palette
pub struct DesignSystem;

impl DesignSystem {
    // Dark theme colors (following user preference for dark mode aesthetic)
    pub const PRIMARY: Color = Color { r: 0.20, g: 0.60, b: 1.00, a: 1.0 };         // #3399FF - Bright blue
    pub const PRIMARY_HOVER: Color = Color { r: 0.35, g: 0.68, b: 1.00, a: 1.0 };    // #59ADFF
    pub const PRIMARY_PRESSED: Color = Color { r: 0.10, g: 0.50, b: 0.90, a: 1.0 };  // #1A80E6

    pub const SECONDARY: Color = Color { r: 0.45, g: 0.25, b: 0.85, a: 1.0 };        // #7340D9 - Purple accent
    pub const SUCCESS: Color = Color { r: 0.20, g: 0.80, b: 0.40, a: 1.0 };          // #33CC66 - Green
    pub const WARNING: Color = Color { r: 1.00, g: 0.60, b: 0.20, a: 1.0 };          // #FF9933 - Orange
    pub const ERROR: Color = Color { r: 0.95, g: 0.25, b: 0.35, a: 1.0 };            // #F24059 - Red

    // Dark theme backgrounds
    pub const SURFACE_PRIMARY: Color = Color { r: 0.08, g: 0.08, b: 0.10, a: 1.0 };  // #141419 - Main background
    pub const SURFACE_SECONDARY: Color = Color { r: 0.12, g: 0.12, b: 0.15, a: 1.0 }; // #1F1F26 - Card background
    pub const SURFACE_TERTIARY: Color = Color { r: 0.16, g: 0.16, b: 0.20, a: 1.0 }; // #292933 - Elevated surfaces
    pub const SURFACE_HOVER: Color = Color { r: 0.20, g: 0.20, b: 0.25, a: 1.0 };    // #333340 - Hover state

    // Text colors
    pub const TEXT_PRIMARY: Color = Color { r: 0.95, g: 0.95, b: 0.97, a: 1.0 };     // #F2F2F7 - Primary text
    pub const TEXT_SECONDARY: Color = Color { r: 0.70, g: 0.70, b: 0.75, a: 1.0 };   // #B3B3BF - Secondary text
    pub const TEXT_TERTIARY: Color = Color { r: 0.50, g: 0.50, b: 0.55, a: 1.0 };    // #80808C - Tertiary text
    pub const TEXT_DISABLED: Color = Color { r: 0.35, g: 0.35, b: 0.40, a: 1.0 };    // #595966 - Disabled text

    // Border colors
    pub const BORDER_PRIMARY: Color = Color { r: 0.25, g: 0.25, b: 0.30, a: 1.0 };   // #40404D - Default borders
    pub const BORDER_SECONDARY: Color = Color { r: 0.18, g: 0.18, b: 0.22, a: 1.0 }; // #2E2E38 - Subtle borders
    pub const BORDER_FOCUS: Color = Color { r: 0.20, g: 0.60, b: 1.00, a: 0.6 };     // Focus ring

    // Shadow colors
    pub const SHADOW_LIGHT: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.10 };       // Light shadow
    pub const SHADOW_MEDIUM: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.20 };      // Medium shadow
    pub const SHADOW_HEAVY: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.40 };       // Heavy shadow

    // Glassmorphism effects
    pub const GLASS_OVERLAY: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 0.05 };      // Glass overlay
    pub const GLASS_BORDER: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 0.10 };       // Glass border

    // Common spacing values
    pub const SPACING_XS: f32 = 4.0;
    pub const SPACING_SM: f32 = 8.0;
    pub const SPACING_MD: f32 = 16.0;
    pub const SPACING_LG: f32 = 24.0;
    pub const SPACING_XL: f32 = 32.0;
    pub const SPACING_XXL: f32 = 48.0;

    // Border radius values
    pub const RADIUS_SM: f32 = 4.0;
    pub const RADIUS_MD: f32 = 8.0;
    pub const RADIUS_LG: f32 = 12.0;
    pub const RADIUS_XL: f32 = 16.0;
    pub const RADIUS_FULL: f32 = 9999.0;

    // Typography scale
    pub const FONT_SIZE_XS: f32 = 10.0;
    pub const FONT_SIZE_SM: f32 = 12.0;
    pub const FONT_SIZE_MD: f32 = 14.0;
    pub const FONT_SIZE_LG: f32 = 16.0;
    pub const FONT_SIZE_XL: f32 = 20.0;
    pub const FONT_SIZE_XXL: f32 = 24.0;
    pub const FONT_SIZE_XXXL: f32 = 32.0;
}

/// Predefined themes for common UI styles
pub struct UITheme;

impl UITheme {
    /// Create a modern primary button style (using new design system)
    pub fn modern_button() -> StateStyle {
        let base = UIStyle {
            background_color: DesignSystem::PRIMARY,
            border: Border::new(0.0, Color::TRANSPARENT).with_radius(DesignSystem::RADIUS_MD),
            padding: Spacing::symmetric(DesignSystem::SPACING_LG, DesignSystem::SPACING_MD),
            typography: Typography {
                color: DesignSystem::TEXT_PRIMARY,
                font_size: DesignSystem::FONT_SIZE_MD,
                bold: true,
                text_align: TextAlign::Center,
                ..Typography::default()
            },
            shadow_offset: (0.0, 2.0),
            shadow_color: DesignSystem::SHADOW_MEDIUM,
            shadow_blur: 8.0,
            ..UIStyle::default()
        };

        let mut style = StateStyle::new(base);

        // Hovered state
        style.hovered.background_color = DesignSystem::PRIMARY_HOVER;
        style.hovered.shadow_offset = (0.0, 4.0);
        style.hovered.shadow_blur = 12.0;

        // Pressed state
        style.pressed.background_color = DesignSystem::PRIMARY_PRESSED;
        style.pressed.shadow_offset = (0.0, 1.0);
        style.pressed.shadow_blur = 4.0;

        // Focused state (accessibility)
        style.focused.background_color = DesignSystem::PRIMARY;
        style.focused.border = Border::new(2.0, DesignSystem::BORDER_FOCUS).with_radius(DesignSystem::RADIUS_MD);

        // Disabled state
        style.disabled.background_color = DesignSystem::SURFACE_TERTIARY;
        style.disabled.typography.color = DesignSystem::TEXT_DISABLED;
        style.disabled.shadow_color = Color::TRANSPARENT;

        style
    }

    /// Create a secondary button style (outline style)
    pub fn secondary_button() -> StateStyle {
        let base = UIStyle {
            background_color: Color::TRANSPARENT,
            border: Border::new(1.0, DesignSystem::PRIMARY).with_radius(DesignSystem::RADIUS_MD),
            padding: Spacing::symmetric(DesignSystem::SPACING_LG, DesignSystem::SPACING_MD),
            typography: Typography {
                color: DesignSystem::PRIMARY,
                font_size: DesignSystem::FONT_SIZE_MD,
                bold: true,
                text_align: TextAlign::Center,
                ..Typography::default()
            },
            ..UIStyle::default()
        };

        let mut style = StateStyle::new(base);

        // Hovered state
        style.hovered.background_color = DesignSystem::PRIMARY.with_alpha(0.1);
        style.hovered.border.color = DesignSystem::PRIMARY_HOVER;
        style.hovered.typography.color = DesignSystem::PRIMARY_HOVER;

        // Pressed state
        style.pressed.background_color = DesignSystem::PRIMARY.with_alpha(0.2);
        style.pressed.border.color = DesignSystem::PRIMARY_PRESSED;
        style.pressed.typography.color = DesignSystem::PRIMARY_PRESSED;

        // Focused state
        style.focused.border = Border::new(2.0, DesignSystem::BORDER_FOCUS).with_radius(DesignSystem::RADIUS_MD);

        // Disabled state
        style.disabled.border.color = DesignSystem::TEXT_DISABLED;
        style.disabled.typography.color = DesignSystem::TEXT_DISABLED;

        style
    }

    /// Create a ghost/flat button style
    pub fn ghost_button() -> StateStyle {
        let base = UIStyle {
            background_color: Color::TRANSPARENT,
            border: Border::NONE,
            padding: Spacing::symmetric(DesignSystem::SPACING_MD, DesignSystem::SPACING_SM),
            typography: Typography {
                color: DesignSystem::TEXT_SECONDARY,
                font_size: DesignSystem::FONT_SIZE_MD,
                text_align: TextAlign::Center,
                ..Typography::default()
            },
            ..UIStyle::default()
        };

        let mut style = StateStyle::new(base);

        style.hovered.background_color = DesignSystem::SURFACE_HOVER;
        style.hovered.typography.color = DesignSystem::TEXT_PRIMARY;
        style.pressed.background_color = DesignSystem::SURFACE_TERTIARY;
        style.disabled.typography.color = DesignSystem::TEXT_DISABLED;

        style
    }

    /// Create a modern card style
    pub fn modern_card() -> StateStyle {
        let base = UIStyle {
            background_color: DesignSystem::SURFACE_SECONDARY,
            border: Border::new(1.0, DesignSystem::BORDER_SECONDARY).with_radius(DesignSystem::RADIUS_LG),
            padding: Spacing::all(DesignSystem::SPACING_LG),
            shadow_offset: (0.0, 4.0),
            shadow_color: DesignSystem::SHADOW_LIGHT,
            shadow_blur: 12.0,
            ..UIStyle::default()
        };

        let mut style = StateStyle::new(base);

        style.hovered.background_color = DesignSystem::SURFACE_TERTIARY;
        style.hovered.shadow_offset = (0.0, 8.0);
        style.hovered.shadow_blur = 24.0;

        style
    }

    /// Create a glass morphism card style
    pub fn glass_card() -> StateStyle {
        let base = UIStyle {
            background_color: DesignSystem::GLASS_OVERLAY,
            border: Border::new(1.0, DesignSystem::GLASS_BORDER).with_radius(DesignSystem::RADIUS_XL),
            padding: Spacing::all(DesignSystem::SPACING_XL),
            shadow_offset: (0.0, 8.0),
            shadow_color: DesignSystem::SHADOW_MEDIUM,
            shadow_blur: 32.0,
            ..UIStyle::default()
        };

        StateStyle::new(base)
    }

    /// Create a modern input field style
    pub fn modern_input() -> StateStyle {
        let base = UIStyle {
            background_color: DesignSystem::SURFACE_TERTIARY,
            border: Border::new(1.0, DesignSystem::BORDER_PRIMARY).with_radius(DesignSystem::RADIUS_MD),
            padding: Spacing::symmetric(DesignSystem::SPACING_MD, DesignSystem::SPACING_MD),
            typography: Typography {
                color: DesignSystem::TEXT_PRIMARY,
                font_size: DesignSystem::FONT_SIZE_MD,
                text_align: TextAlign::Left,
                ..Typography::default()
            },
            ..UIStyle::default()
        };

        let mut style = StateStyle::new(base);

        style.hovered.border.color = DesignSystem::PRIMARY.with_alpha(0.5);
        style.focused.border = Border::new(2.0, DesignSystem::PRIMARY).with_radius(DesignSystem::RADIUS_MD);
        style.disabled.background_color = DesignSystem::SURFACE_SECONDARY;
        style.disabled.typography.color = DesignSystem::TEXT_DISABLED;

        style
    }

    /// Create a modern panel/container style
    pub fn modern_panel() -> StateStyle {
        let base = UIStyle {
            background_color: DesignSystem::SURFACE_PRIMARY,
            border: Border::new(1.0, DesignSystem::BORDER_SECONDARY).with_radius(DesignSystem::RADIUS_LG),
            padding: Spacing::all(DesignSystem::SPACING_XL),
            ..UIStyle::default()
        };

        StateStyle::new(base)
    }

    /// Create a success notification style
    pub fn success_notification() -> StateStyle {
        let base = UIStyle {
            background_color: DesignSystem::SUCCESS.with_alpha(0.1),
            border: Border::new(1.0, DesignSystem::SUCCESS).with_radius(DesignSystem::RADIUS_MD),
            padding: Spacing::all(DesignSystem::SPACING_MD),
            typography: Typography {
                color: DesignSystem::SUCCESS,
                font_size: DesignSystem::FONT_SIZE_MD,
                ..Typography::default()
            },
            ..UIStyle::default()
        };

        StateStyle::new(base)
    }

    /// Create a warning notification style
    pub fn warning_notification() -> StateStyle {
        let base = UIStyle {
            background_color: DesignSystem::WARNING.with_alpha(0.1),
            border: Border::new(1.0, DesignSystem::WARNING).with_radius(DesignSystem::RADIUS_MD),
            padding: Spacing::all(DesignSystem::SPACING_MD),
            typography: Typography {
                color: DesignSystem::WARNING,
                font_size: DesignSystem::FONT_SIZE_MD,
                ..Typography::default()
            },
            ..UIStyle::default()
        };

        StateStyle::new(base)
    }

    /// Create an error notification style
    pub fn error_notification() -> StateStyle {
        let base = UIStyle {
            background_color: DesignSystem::ERROR.with_alpha(0.1),
            border: Border::new(1.0, DesignSystem::ERROR).with_radius(DesignSystem::RADIUS_MD),
            padding: Spacing::all(DesignSystem::SPACING_MD),
            typography: Typography {
                color: DesignSystem::ERROR,
                font_size: DesignSystem::FONT_SIZE_MD,
                ..Typography::default()
            },
            ..UIStyle::default()
        };

        StateStyle::new(base)
    }

    /// Create a card/panel style
    pub fn card() -> UIStyle {
        UIStyle {
            background_color: Color::WHITE,
            border: Border::new(1.0, Color::LIGHT_GRAY).with_radius(8.0),
            padding: Spacing::all(16.0),
            shadow_offset: (0.0, 2.0),
            shadow_color: Color::new(0.0, 0.0, 0.0, 0.1),
            shadow_blur: 8.0,
            ..UIStyle::default()
        }
    }

    /// Create an input field style
    pub fn text_input() -> StateStyle {
        let base = UIStyle {
            background_color: Color::WHITE,
            border: Border::new(2.0, Color::LIGHT_GRAY).with_radius(4.0),
            padding: Spacing::symmetric(12.0, 8.0),
            typography: Typography {
                color: Color::BLACK,
                font_size: 14.0,
                ..Typography::default()
            },
            ..UIStyle::default()
        };

        let mut style = StateStyle::new(base);
        
        style.focused.border.color = Color::from_hex(0x007ACC);
        style.disabled.background_color = Color::LIGHT_GRAY;
        style.disabled.typography.color = Color::GRAY;

        style
    }

    /// Create a dark theme style
    pub fn dark_button() -> StateStyle {
        let base = UIStyle {
            background_color: Color::from_hex(0x2D2D30),
            border: Border::new(1.0, Color::from_hex(0x3E3E42)).with_radius(4.0),
            padding: Spacing::symmetric(16.0, 8.0),
            typography: Typography {
                color: Color::WHITE,
                font_size: 14.0,
                bold: true,
                text_align: TextAlign::Center,
                ..Typography::default()
            },
            ..UIStyle::default()
        };

        let mut style = StateStyle::new(base);
        
        style.hovered.background_color = Color::from_hex(0x3E3E42);
        style.pressed.background_color = Color::from_hex(0x1E1E1E);
        style.disabled.background_color = Color::from_hex(0x1A1A1A);
        style.disabled.typography.color = Color::from_hex(0x808080);

        style
    }
}