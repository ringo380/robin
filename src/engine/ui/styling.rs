use crate::engine::ui::UIState;

/// Color representation for UI elements
#[derive(Debug, Clone, Copy, PartialEq)]
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

/// Border styling
#[derive(Debug, Clone, Copy, PartialEq)]
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
#[derive(Debug, Clone)]
pub struct Typography {
    pub font_size: f32,
    pub color: Color,
    pub font_family: String,
    pub bold: bool,
    pub italic: bool,
    pub line_height: f32,
    pub text_align: TextAlign,
}

#[derive(Debug, Clone, Copy, PartialEq)]
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

/// Predefined themes for common UI styles
pub struct UITheme;

impl UITheme {
    /// Create a modern button style
    pub fn modern_button() -> StateStyle {
        let mut base = UIStyle {
            background_color: Color::from_hex(0x007ACC),
            border: Border::new(2.0, Color::from_hex(0x005A9E)).with_radius(6.0),
            padding: Spacing::symmetric(16.0, 8.0),
            typography: Typography {
                color: Color::WHITE,
                font_size: 14.0,
                bold: true,
                text_align: TextAlign::Center,
                ..Typography::default()
            },
            shadow_offset: (0.0, 2.0),
            shadow_color: Color::new(0.0, 0.0, 0.0, 0.2),
            shadow_blur: 4.0,
            ..UIStyle::default()
        };

        let mut style = StateStyle::new(base.clone());
        
        // Hovered state
        style.hovered.background_color = Color::from_hex(0x1E88E5);
        style.hovered.shadow_offset = (0.0, 4.0);
        style.hovered.shadow_blur = 8.0;

        // Pressed state
        style.pressed.background_color = Color::from_hex(0x0056B3);
        style.pressed.shadow_offset = (0.0, 1.0);
        style.pressed.shadow_blur = 2.0;

        // Disabled state
        style.disabled.background_color = Color::GRAY;
        style.disabled.border.color = Color::DARK_GRAY;
        style.disabled.typography.color = Color::LIGHT_GRAY;
        style.disabled.shadow_color = Color::TRANSPARENT;

        style
    }

    /// Create a flat button style
    pub fn flat_button() -> StateStyle {
        let base = UIStyle {
            background_color: Color::TRANSPARENT,
            border: Border::NONE,
            padding: Spacing::symmetric(12.0, 6.0),
            typography: Typography {
                color: Color::from_hex(0x007ACC),
                font_size: 14.0,
                text_align: TextAlign::Center,
                ..Typography::default()
            },
            ..UIStyle::default()
        };

        let mut style = StateStyle::new(base);
        
        style.hovered.background_color = Color::new(0.0, 0.5, 0.8, 0.1);
        style.pressed.background_color = Color::new(0.0, 0.5, 0.8, 0.2);
        style.disabled.typography.color = Color::GRAY;

        style
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