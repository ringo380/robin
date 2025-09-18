use std::collections::HashMap;
use crate::engine::ui::styling::{Color, Spacing, Border, Typography, TextDecoration, AlignItems, JustifyContent, FlexDirection, BoxSizing, Resize};
use serde::{Serialize, Deserialize};

/// Advanced CSS-in-Rust styling system for modern UI components
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StyleSheet {
    pub styles: HashMap<String, Style>,
    pub media_queries: Vec<MediaQuery>,
    pub keyframes: HashMap<String, KeyframeAnimation>,
    pub variables: HashMap<String, StyleVariable>,
}

/// CSS-like style properties
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Style {
    // Layout
    pub display: Option<Display>,
    pub position: Option<Position>,
    pub flex: Option<FlexStyle>,
    pub grid: Option<GridStyle>,
    pub width: Option<Dimension>,
    pub height: Option<Dimension>,
    pub min_width: Option<Dimension>,
    pub min_height: Option<Dimension>,
    pub max_width: Option<Dimension>,
    pub max_height: Option<Dimension>,

    // Spacing
    pub margin: Option<BoxSpacing>,
    pub padding: Option<BoxSpacing>,
    pub gap: Option<f32>,

    // Positioning
    pub top: Option<Dimension>,
    pub right: Option<Dimension>,
    pub bottom: Option<Dimension>,
    pub left: Option<Dimension>,
    pub z_index: Option<i32>,

    // Typography
    pub font_family: Option<String>,
    pub font_size: Option<f32>,
    pub font_weight: Option<FontWeight>,
    pub line_height: Option<f32>,
    pub text_align: Option<TextAlign>,
    pub text_transform: Option<TextTransform>,
    pub letter_spacing: Option<f32>,

    // Colors and backgrounds
    pub color: Option<Color>,
    pub background: Option<Background>,
    pub background_color: Option<Color>,
    pub border_color: Option<Color>,

    // Borders
    pub border_width: Option<f32>,
    pub border_style: Option<BorderStyle>,
    pub border_radius: Option<BorderRadius>,
    pub border: Option<Border>,
    pub border_bottom: Option<Border>,
    pub border_left: Option<Border>,
    pub outline: Option<Border>,
    pub outline_offset: Option<f32>,

    // Typography extended
    pub typography: Option<Typography>,
    pub text_decoration: Option<TextDecoration>,

    // Layout extended
    pub align_items: Option<AlignItems>,
    pub justify_content: Option<JustifyContent>,
    pub flex_direction: Option<FlexDirection>,
    pub box_sizing: Option<BoxSizing>,
    pub resize: Option<Resize>,

    // Effects
    pub opacity: Option<f32>,
    pub box_shadow: Option<Vec<BoxShadow>>,
    pub filter: Option<Vec<Filter>>,
    pub backdrop_filter: Option<Vec<Filter>>,

    // Transforms
    pub transform: Option<Vec<Transform>>,
    pub transform_origin: Option<TransformOrigin>,

    // Transitions
    pub transition: Option<Vec<Transition>>,

    // Animations
    pub animation: Option<Vec<Animation>>,

    // Interactivity
    pub cursor: Option<Cursor>,
    pub pointer_events: Option<PointerEvents>,
    pub user_select: Option<UserSelect>,

    // Overflow
    pub overflow: Option<Overflow>,
    pub overflow_x: Option<Overflow>,
    pub overflow_y: Option<Overflow>,

    // Pseudo-classes
    pub hover: Option<Box<Style>>,
    pub active: Option<Box<Style>>,
    pub focus: Option<Box<Style>>,
    pub disabled: Option<Box<Style>>,
    pub before: Option<Box<Style>>,
    pub after: Option<Box<Style>>,
}

// Display types
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Display {
    None,
    Block,
    Inline,
    InlineBlock,
    Flex,
    InlineFlex,
    Grid,
    InlineGrid,
}

// Position types
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Position {
    Static,
    Relative,
    Absolute,
    Fixed,
    Sticky,
}

// Dimension units
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Dimension {
    Px(f32),
    Percent(f32),
    Vw(f32),
    Vh(f32),
    Em(f32),
    Rem(f32),
    Auto,
    FitContent,
    MinContent,
    MaxContent,
    Calc(String),
}

// Flex properties
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FlexStyle {
    pub direction: Option<FlexDirection>,
    pub wrap: Option<FlexWrap>,
    pub justify: Option<JustifyContent>,
    pub align: Option<AlignItems>,
    pub align_content: Option<AlignContent>,
    pub grow: Option<f32>,
    pub shrink: Option<f32>,
    pub basis: Option<Dimension>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AlignContent {
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
    Stretch,
}

// Grid properties
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GridStyle {
    pub template_columns: Option<Vec<GridTrack>>,
    pub template_rows: Option<Vec<GridTrack>>,
    pub gap: Option<f32>,
    pub column_gap: Option<f32>,
    pub row_gap: Option<f32>,
    pub auto_flow: Option<GridAutoFlow>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GridTrack {
    Fixed(Dimension),
    MinMax(Dimension, Dimension),
    Repeat(u32, Box<GridTrack>),
    Fr(f32),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GridAutoFlow {
    Row,
    Column,
    RowDense,
    ColumnDense,
}

// Background
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Background {
    Color(Color),
    Gradient(Gradient),
    Image(String),
    Multiple(Vec<Background>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Gradient {
    pub gradient_type: GradientType,
    pub stops: Vec<ColorStop>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GradientType {
    Linear(f32), // angle in degrees
    Radial,
    Conic(f32), // angle in degrees
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorStop {
    pub color: Color,
    pub position: f32, // 0.0 to 1.0
}

// Box spacing
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BoxSpacing {
    pub top: Option<f32>,
    pub right: Option<f32>,
    pub bottom: Option<f32>,
    pub left: Option<f32>,
}

impl BoxSpacing {
    pub fn all(value: f32) -> Self {
        Self {
            top: Some(value),
            right: Some(value),
            bottom: Some(value),
            left: Some(value),
        }
    }

    pub fn xy(x: f32, y: f32) -> Self {
        Self {
            top: Some(y),
            right: Some(x),
            bottom: Some(y),
            left: Some(x),
        }
    }
}

// Typography
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FontWeight {
    Thin,
    Light,
    Normal,
    Medium,
    Bold,
    Black,
    Weight(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TextAlign {
    Left,
    Right,
    Center,
    Justify,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TextTransform {
    None,
    Uppercase,
    Lowercase,
    Capitalize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TypographyStyle {
    pub font_family: Option<String>,
    pub font_size: Option<f32>,
    pub font_weight: Option<FontWeight>,
    pub line_height: Option<f32>,
    pub text_align: Option<TextAlign>,
    pub text_transform: Option<TextTransform>,
    pub letter_spacing: Option<f32>,
    pub color: Option<Color>,
}

// Borders
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BorderStyle {
    None,
    Solid,
    Dashed,
    Dotted,
    Double,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BorderRadius {
    pub top_left: Option<f32>,
    pub top_right: Option<f32>,
    pub bottom_right: Option<f32>,
    pub bottom_left: Option<f32>,
}

impl BorderRadius {
    pub fn all(radius: f32) -> Self {
        Self {
            top_left: Some(radius),
            top_right: Some(radius),
            bottom_right: Some(radius),
            bottom_left: Some(radius),
        }
    }
}

// Shadows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoxShadow {
    pub x: f32,
    pub y: f32,
    pub blur: f32,
    pub spread: f32,
    pub color: Color,
    pub inset: bool,
}

// Filters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Filter {
    Blur(f32),
    Brightness(f32),
    Contrast(f32),
    Grayscale(f32),
    HueRotate(f32),
    Invert(f32),
    Opacity(f32),
    Saturate(f32),
    Sepia(f32),
}

// Transforms
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Transform {
    Translate(f32, f32),
    Translate3d(f32, f32, f32),
    Scale(f32, f32),
    Scale3d(f32, f32, f32),
    Rotate(f32),
    Rotate3d(f32, f32, f32, f32),
    Skew(f32, f32),
    Matrix(Vec<f32>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransformOrigin {
    pub x: Dimension,
    pub y: Dimension,
    pub z: Option<f32>,
}

// Transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    pub property: String,
    pub duration: f32,
    pub timing: TimingFunction,
    pub delay: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TimingFunction {
    Linear,
    Ease,
    EaseIn,
    EaseOut,
    EaseInOut,
    CubicBezier(f32, f32, f32, f32),
}

// Animations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Animation {
    pub name: String,
    pub duration: f32,
    pub timing: TimingFunction,
    pub delay: f32,
    pub iterations: AnimationIterations,
    pub direction: AnimationDirection,
    pub fill_mode: AnimationFillMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AnimationIterations {
    Count(u32),
    Infinite,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AnimationDirection {
    Normal,
    Reverse,
    Alternate,
    AlternateReverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AnimationFillMode {
    None,
    Forwards,
    Backwards,
    Both,
}

// Keyframes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyframeAnimation {
    pub name: String,
    pub keyframes: Vec<Keyframe>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyframe {
    pub offset: f32, // 0.0 to 1.0
    pub style: Style,
}

// Cursor types
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Cursor {
    Auto,
    Default,
    Pointer,
    Move,
    Text,
    Wait,
    Help,
    Progress,
    NotAllowed,
    Grab,
    Grabbing,
    Crosshair,
    ZoomIn,
    ZoomOut,
    Resize(ResizeDirection),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ResizeDirection {
    N, E, S, W, NE, NW, SE, SW, EW, NS, NESW, NWSE,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PointerEvents {
    Auto,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum UserSelect {
    Auto,
    None,
    Text,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Overflow {
    Visible,
    Hidden,
    Scroll,
    Auto,
}

// Media queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaQuery {
    pub condition: MediaCondition,
    pub styles: HashMap<String, Style>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MediaCondition {
    MinWidth(f32),
    MaxWidth(f32),
    MinHeight(f32),
    MaxHeight(f32),
    Orientation(Orientation),
    PrefersDark,
    PrefersLight,
    PrefersReducedMotion,
    And(Box<MediaCondition>, Box<MediaCondition>),
    Or(Box<MediaCondition>, Box<MediaCondition>),
    Not(Box<MediaCondition>),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Orientation {
    Portrait,
    Landscape,
}

// Style variables (CSS custom properties)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StyleVariable {
    Color(Color),
    Number(f32),
    String(String),
    Dimension(Dimension),
}

/// Styled component macro-like builder
pub struct Styled;

impl Styled {
    pub fn div() -> StyledBuilder {
        StyledBuilder::new("div")
    }

    pub fn button() -> StyledBuilder {
        StyledBuilder::new("button")
    }

    pub fn input() -> StyledBuilder {
        StyledBuilder::new("input")
    }

    pub fn text() -> StyledBuilder {
        StyledBuilder::new("text")
    }

    pub fn image() -> StyledBuilder {
        StyledBuilder::new("image")
    }
}

/// Builder for styled components
pub struct StyledBuilder {
    component_type: String,
    style: Style,
    class_names: Vec<String>,
    id: Option<String>,
}

impl StyledBuilder {
    pub fn new(component_type: &str) -> Self {
        Self {
            component_type: component_type.to_string(),
            style: Style::default(),
            class_names: Vec::new(),
            id: None,
        }
    }

    // Layout methods
    pub fn display(mut self, display: Display) -> Self {
        self.style.display = Some(display);
        self
    }

    pub fn flex_row(mut self) -> Self {
        self.style.display = Some(Display::Flex);
        self.style.flex = Some(FlexStyle {
            direction: Some(FlexDirection::Row),
            ..Default::default()
        });
        self
    }

    pub fn flex_column(mut self) -> Self {
        self.style.display = Some(Display::Flex);
        self.style.flex = Some(FlexStyle {
            direction: Some(FlexDirection::Column),
            ..Default::default()
        });
        self
    }

    pub fn center(mut self) -> Self {
        if let Some(ref mut flex) = self.style.flex {
            flex.justify = Some(JustifyContent::Center);
            flex.align = Some(AlignItems::Center);
        } else {
            self.style.flex = Some(FlexStyle {
                justify: Some(JustifyContent::Center),
                align: Some(AlignItems::Center),
                ..Default::default()
            });
        }
        self
    }

    // Dimensions
    pub fn width(mut self, width: Dimension) -> Self {
        self.style.width = Some(width);
        self
    }

    pub fn height(mut self, height: Dimension) -> Self {
        self.style.height = Some(height);
        self
    }

    pub fn size(mut self, width: Dimension, height: Dimension) -> Self {
        self.style.width = Some(width);
        self.style.height = Some(height);
        self
    }

    // Spacing
    pub fn margin(mut self, margin: BoxSpacing) -> Self {
        self.style.margin = Some(margin);
        self
    }

    pub fn padding(mut self, padding: BoxSpacing) -> Self {
        self.style.padding = Some(padding);
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.style.gap = Some(gap);
        self
    }

    // Colors
    pub fn bg(mut self, background: Background) -> Self {
        self.style.background = Some(background);
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.style.color = Some(color);
        self
    }

    // Borders
    pub fn border(mut self, width: f32, style: BorderStyle, color: Color) -> Self {
        self.style.border_width = Some(width);
        self.style.border_style = Some(style);
        self.style.border_color = Some(color);
        self
    }

    pub fn rounded(mut self, radius: BorderRadius) -> Self {
        self.style.border_radius = Some(radius);
        self
    }

    // Effects
    pub fn shadow(mut self, shadow: BoxShadow) -> Self {
        if self.style.box_shadow.is_none() {
            self.style.box_shadow = Some(Vec::new());
        }
        self.style.box_shadow.as_mut().unwrap().push(shadow);
        self
    }

    pub fn opacity(mut self, opacity: f32) -> Self {
        self.style.opacity = Some(opacity);
        self
    }

    // Typography
    pub fn font_size(mut self, size: f32) -> Self {
        self.style.font_size = Some(size);
        self
    }

    pub fn font_weight(mut self, weight: FontWeight) -> Self {
        self.style.font_weight = Some(weight);
        self
    }

    pub fn text_align(mut self, align: TextAlign) -> Self {
        self.style.text_align = Some(align);
        self
    }

    // Pseudo-classes
    pub fn hover(mut self, hover_style: Style) -> Self {
        self.style.hover = Some(Box::new(hover_style));
        self
    }

    pub fn active(mut self, active_style: Style) -> Self {
        self.style.active = Some(Box::new(active_style));
        self
    }

    pub fn focus(mut self, focus_style: Style) -> Self {
        self.style.focus = Some(Box::new(focus_style));
        self
    }

    // Transitions
    pub fn transition(mut self, transition: Transition) -> Self {
        if self.style.transition.is_none() {
            self.style.transition = Some(Vec::new());
        }
        self.style.transition.as_mut().unwrap().push(transition);
        self
    }

    // Animations
    pub fn animation(mut self, animation: Animation) -> Self {
        if self.style.animation.is_none() {
            self.style.animation = Some(Vec::new());
        }
        self.style.animation.as_mut().unwrap().push(animation);
        self
    }

    // Utility
    pub fn class(mut self, class_name: &str) -> Self {
        self.class_names.push(class_name.to_string());
        self
    }

    pub fn id(mut self, id: &str) -> Self {
        self.id = Some(id.to_string());
        self
    }

    pub fn build(self) -> StyledComponent {
        StyledComponent {
            component_type: self.component_type,
            style: self.style,
            class_names: self.class_names,
            id: self.id,
        }
    }
}

/// A styled component with all its properties
#[derive(Debug, Clone)]
pub struct StyledComponent {
    pub component_type: String,
    pub style: Style,
    pub class_names: Vec<String>,
    pub id: Option<String>,
}

/// Utility functions for common patterns
pub mod css {
    use super::*;

    /// Create a flexbox container
    pub fn flex(direction: FlexDirection, justify: JustifyContent, align: AlignItems) -> Style {
        Style {
            display: Some(Display::Flex),
            flex: Some(FlexStyle {
                direction: Some(direction),
                justify: Some(justify),
                align: Some(align),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    /// Create a centered container
    pub fn center() -> Style {
        flex(FlexDirection::Column, JustifyContent::Center, AlignItems::Center)
    }

    /// Create a grid container
    pub fn grid(columns: Vec<GridTrack>, rows: Vec<GridTrack>) -> Style {
        Style {
            display: Some(Display::Grid),
            grid: Some(GridStyle {
                template_columns: Some(columns),
                template_rows: Some(rows),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    /// Create a card style
    pub fn card(bg: Color, radius: f32) -> Style {
        Style {
            background: Some(Background::Color(bg)),
            border_radius: Some(BorderRadius::all(radius)),
            box_shadow: Some(vec![BoxShadow {
                x: 0.0,
                y: 2.0,
                blur: 8.0,
                spread: 0.0,
                color: Color::new(0.0, 0.0, 0.0, 0.1),
                inset: false,
            }]),
            padding: Some(BoxSpacing::all(16.0)),
            ..Default::default()
        }
    }

    /// Create a button style
    pub fn button(bg: Color, text: Color) -> Style {
        Style {
            display: Some(Display::InlineBlock),
            background: Some(Background::Color(bg)),
            color: Some(text),
            padding: Some(BoxSpacing::xy(16.0, 8.0)),
            border_radius: Some(BorderRadius::all(4.0)),
            cursor: Some(Cursor::Pointer),
            transition: Some(vec![Transition {
                property: "all".to_string(),
                duration: 0.2,
                timing: TimingFunction::EaseInOut,
                delay: 0.0,
            }]),
            ..Default::default()
        }
    }

    /// Create responsive styles
    pub fn responsive(base: Style, breakpoints: Vec<(f32, Style)>) -> StyleSheet {
        let mut sheet = StyleSheet::default();
        sheet.styles.insert("base".to_string(), base);

        for (width, style) in breakpoints {
            sheet.media_queries.push(MediaQuery {
                condition: MediaCondition::MinWidth(width),
                styles: {
                    let mut map = HashMap::new();
                    map.insert("responsive".to_string(), style);
                    map
                },
            });
        }

        sheet
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_styled_builder() {
        let component = Styled::div()
            .display(Display::Flex)
            .flex_row()
            .center()
            .width(Dimension::Px(100.0))
            .height(Dimension::Percent(50.0))
            .bg(Background::Color(Color::WHITE))
            .padding(BoxSpacing::all(16.0))
            .rounded(BorderRadius::all(8.0))
            .build();

        assert_eq!(component.component_type, "div");
        assert_eq!(component.style.display, Some(Display::Flex));
        assert_eq!(component.style.width, Some(Dimension::Px(100.0)));
    }

    #[test]
    fn test_media_queries() {
        let mobile = MediaCondition::MaxWidth(768.0);
        let tablet = MediaCondition::And(
            Box::new(MediaCondition::MinWidth(768.0)),
            Box::new(MediaCondition::MaxWidth(1024.0)),
        );
        let desktop = MediaCondition::MinWidth(1024.0);

        // Ensure conditions can be created and are distinct
        assert_ne!(
            serde_json::to_string(&mobile).unwrap(),
            serde_json::to_string(&tablet).unwrap()
        );
    }

    #[test]
    fn test_css_utilities() {
        let centered = css::center();
        assert_eq!(centered.display, Some(Display::Flex));

        let card = css::card(Color::WHITE, 8.0);
        assert!(card.box_shadow.is_some());
        assert!(card.padding.is_some());
    }

    #[test]
    fn test_gradient_background() {
        let gradient = Background::Gradient(Gradient {
            gradient_type: GradientType::Linear(45.0),
            stops: vec![
                ColorStop { color: Color::RED, position: 0.0 },
                ColorStop { color: Color::BLUE, position: 1.0 },
            ],
        });

        if let Background::Gradient(g) = gradient {
            assert_eq!(g.stops.len(), 2);
            if let GradientType::Linear(angle) = g.gradient_type {
                assert_eq!(angle, 45.0);
            }
        }
    }
}