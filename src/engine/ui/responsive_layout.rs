use crate::engine::{
    error::RobinResult,
    input::InputManager,
    math::{Vec2, Vec3},
    ui::modern_interface::{ModernUISystem, UITheme, Color, Rectangle, TextStyle},
};
use std::collections::HashMap;

pub struct ResponsiveLayoutSystem {
    modern_ui: ModernUISystem,
    current_breakpoint: Breakpoint,
    screen_size: Vec2,
    layout_containers: Vec<LayoutContainer>,
    breakpoints: BreakpointConfiguration,
    scaling_system: UIScalingSystem,
    adaptive_components: Vec<AdaptiveComponent>,
    orientation: ScreenOrientation,
    dpi_scale: f32,
    layout_cache: LayoutCache,
    animation_controller: LayoutAnimationController,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Breakpoint {
    Mobile,       // < 768px
    Tablet,       // 768px - 1024px
    Desktop,      // 1024px - 1440px
    Large,        // 1440px - 1920px
    UltraWide,    // > 1920px
}

#[derive(Debug, Clone)]
pub struct BreakpointConfiguration {
    pub mobile_max: f32,
    pub tablet_max: f32,
    pub desktop_max: f32,
    pub large_max: f32,
    pub custom_breakpoints: HashMap<String, f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenOrientation {
    Portrait,
    Landscape,
    Square,
}

pub struct LayoutContainer {
    pub id: String,
    pub layout_type: LayoutType,
    pub bounds: Rectangle,
    pub children: Vec<String>,
    pub parent: Option<String>,
    pub responsive_rules: ResponsiveRules,
    pub visibility_rules: VisibilityRules,
    pub z_index: i32,
    pub overflow: OverflowBehavior,
}

#[derive(Debug, Clone)]
pub enum LayoutType {
    Flexbox(FlexboxLayout),
    Grid(GridLayout),
    Absolute(AbsoluteLayout),
    Flow(FlowLayout),
    Stack(StackLayout),
    Adaptive(AdaptiveLayout),
}

#[derive(Debug, Clone)]
pub struct FlexboxLayout {
    pub direction: FlexDirection,
    pub wrap: FlexWrap,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub align_content: AlignContent,
    pub gap: f32,
    pub responsive_direction: HashMap<Breakpoint, FlexDirection>,
}

#[derive(Debug, Clone)]
pub struct GridLayout {
    pub template_columns: Vec<GridTrack>,
    pub template_rows: Vec<GridTrack>,
    pub gap: Vec2,
    pub auto_flow: GridAutoFlow,
    pub responsive_columns: HashMap<Breakpoint, Vec<GridTrack>>,
    pub responsive_rows: HashMap<Breakpoint, Vec<GridTrack>>,
}

#[derive(Debug, Clone)]
pub struct AbsoluteLayout {
    pub positioning: PositioningMode,
    pub anchor_points: AnchorConfiguration,
    pub responsive_positioning: HashMap<Breakpoint, PositioningMode>,
}

#[derive(Debug, Clone)]
pub struct FlowLayout {
    pub direction: FlowDirection,
    pub line_height: f32,
    pub word_wrap: bool,
    pub text_align: TextAlign,
}

#[derive(Debug, Clone)]
pub struct StackLayout {
    pub direction: StackDirection,
    pub spacing: f32,
    pub alignment: StackAlignment,
    pub distribution: StackDistribution,
}

#[derive(Debug, Clone)]
pub struct AdaptiveLayout {
    pub breakpoint_layouts: HashMap<Breakpoint, LayoutType>,
    pub transition_animations: HashMap<Breakpoint, TransitionAnimation>,
}

#[derive(Debug, Clone)]
pub enum FlexDirection {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

#[derive(Debug, Clone)]
pub enum FlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}

#[derive(Debug, Clone)]
pub enum JustifyContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone)]
pub enum AlignItems {
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
}

#[derive(Debug, Clone)]
pub enum AlignContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    Stretch,
}

#[derive(Debug, Clone)]
pub enum GridTrack {
    Fixed(f32),
    Fraction(f32),
    MinContent,
    MaxContent,
    Auto,
    MinMax { min: Box<GridTrack>, max: Box<GridTrack> },
    Repeat { count: u32, tracks: Vec<GridTrack> },
}

#[derive(Debug, Clone)]
pub enum GridAutoFlow {
    Row,
    Column,
    RowDense,
    ColumnDense,
}

#[derive(Debug, Clone)]
pub enum PositioningMode {
    Static,
    Relative { offset: Vec2 },
    Absolute { position: Vec2 },
    Fixed { position: Vec2 },
    Sticky { offset: Vec2, threshold: f32 },
}

#[derive(Debug, Clone)]
pub struct AnchorConfiguration {
    pub horizontal: HorizontalAnchor,
    pub vertical: VerticalAnchor,
    pub margins: EdgeInsets,
}

#[derive(Debug, Clone)]
pub enum HorizontalAnchor {
    Left(f32),
    Right(f32),
    Center,
    LeftRight { left: f32, right: f32 },
    Percentage(f32),
}

#[derive(Debug, Clone)]
pub enum VerticalAnchor {
    Top(f32),
    Bottom(f32),
    Center,
    TopBottom { top: f32, bottom: f32 },
    Percentage(f32),
}

#[derive(Debug, Clone)]
pub struct EdgeInsets {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

#[derive(Debug, Clone)]
pub enum FlowDirection {
    LeftToRight,
    RightToLeft,
    TopToBottom,
}

#[derive(Debug, Clone)]
pub enum TextAlign {
    Left,
    Right,
    Center,
    Justify,
}

#[derive(Debug, Clone)]
pub enum StackDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone)]
pub enum StackAlignment {
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Debug, Clone)]
pub enum StackDistribution {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
    Fill,
}

#[derive(Debug, Clone)]
pub struct ResponsiveRules {
    pub visibility: HashMap<Breakpoint, bool>,
    pub sizes: HashMap<Breakpoint, ResponsiveSize>,
    pub margins: HashMap<Breakpoint, EdgeInsets>,
    pub padding: HashMap<Breakpoint, EdgeInsets>,
    pub typography: HashMap<Breakpoint, ResponsiveTypography>,
    pub colors: HashMap<Breakpoint, ResponsiveColors>,
}

#[derive(Debug, Clone)]
pub struct ResponsiveSize {
    pub width: SizeConstraint,
    pub height: SizeConstraint,
    pub min_width: Option<f32>,
    pub max_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_height: Option<f32>,
    pub aspect_ratio: Option<f32>,
}

#[derive(Debug, Clone)]
pub enum SizeConstraint {
    Fixed(f32),
    Percentage(f32),
    Auto,
    FitContent,
    MinContent,
    MaxContent,
    Viewport { width: Option<f32>, height: Option<f32> },
}

#[derive(Debug, Clone)]
pub struct ResponsiveTypography {
    pub font_size: Option<f32>,
    pub font_weight: Option<u16>,
    pub line_height: Option<f32>,
    pub letter_spacing: Option<f32>,
    pub text_transform: Option<TextTransform>,
}

#[derive(Debug, Clone)]
pub enum TextTransform {
    None,
    Uppercase,
    Lowercase,
    Capitalize,
}

#[derive(Debug, Clone)]
pub struct ResponsiveColors {
    pub background: Option<Color>,
    pub text: Option<Color>,
    pub border: Option<Color>,
    pub accent: Option<Color>,
}

#[derive(Debug, Clone)]
pub struct VisibilityRules {
    pub breakpoint_visibility: HashMap<Breakpoint, bool>,
    pub orientation_visibility: HashMap<ScreenOrientation, bool>,
    pub conditional_visibility: Vec<VisibilityCondition>,
}

#[derive(Debug, Clone)]
pub struct VisibilityCondition {
    pub condition: VisibilityConditionType,
    pub visible: bool,
}

#[derive(Debug, Clone)]
pub enum VisibilityConditionType {
    ScreenWidthRange { min: f32, max: f32 },
    ScreenHeightRange { min: f32, max: f32 },
    AspectRatioRange { min: f32, max: f32 },
    DPIRange { min: f32, max: f32 },
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum OverflowBehavior {
    Visible,
    Hidden,
    Scroll,
    Auto,
}

pub struct UIScalingSystem {
    base_scale: f32,
    dpi_aware: bool,
    scaling_mode: ScalingMode,
    custom_scale_factors: HashMap<Breakpoint, f32>,
    font_scaling: FontScalingConfiguration,
    icon_scaling: IconScalingConfiguration,
    spacing_scaling: SpacingScalingConfiguration,
}

#[derive(Debug, Clone)]
pub enum ScalingMode {
    None,
    Uniform,
    NonUniform { x: f32, y: f32 },
    DPIAware,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct FontScalingConfiguration {
    pub base_size: f32,
    pub scale_factors: HashMap<Breakpoint, f32>,
    pub min_size: f32,
    pub max_size: f32,
    pub line_height_adjustment: f32,
}

#[derive(Debug, Clone)]
pub struct IconScalingConfiguration {
    pub base_size: f32,
    pub scale_factors: HashMap<Breakpoint, f32>,
    pub crisp_edges: bool,
    pub vector_scaling: bool,
}

#[derive(Debug, Clone)]
pub struct SpacingScalingConfiguration {
    pub base_unit: f32,
    pub scale_factors: HashMap<Breakpoint, f32>,
    pub maintain_proportions: bool,
}

pub struct AdaptiveComponent {
    pub id: String,
    pub component_type: ComponentType,
    pub adaptive_behaviors: Vec<AdaptiveBehavior>,
    pub state: ComponentState,
    pub priority: i32,
}

#[derive(Debug, Clone)]
pub enum ComponentType {
    Navigation,
    Toolbar,
    Sidebar,
    Panel,
    Modal,
    Tooltip,
    Menu,
    Button,
    Input,
    Text,
    Image,
    Video,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct AdaptiveBehavior {
    pub trigger: AdaptiveTrigger,
    pub action: AdaptiveAction,
    pub priority: i32,
    pub conditions: Vec<AdaptiveCondition>,
}

#[derive(Debug, Clone)]
pub enum AdaptiveTrigger {
    BreakpointChange(Breakpoint),
    OrientationChange(ScreenOrientation),
    SizeChange { width_threshold: Option<f32>, height_threshold: Option<f32> },
    DPIChange(f32),
    UserInteraction(InteractionType),
    ContentOverflow,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum AdaptiveAction {
    ChangeLayout(LayoutType),
    ToggleVisibility(bool),
    Resize { width: Option<f32>, height: Option<f32> },
    Reposition(Vec2),
    ChangeStyle(StyleUpdate),
    Reorganize(ReorganizationStrategy),
    Custom(String, HashMap<String, String>),
}

#[derive(Debug, Clone)]
pub enum InteractionType {
    Hover,
    Click,
    Touch,
    Drag,
    Scroll,
    Pinch,
    Keyboard,
}

#[derive(Debug, Clone)]
pub struct StyleUpdate {
    pub colors: Option<ResponsiveColors>,
    pub typography: Option<ResponsiveTypography>,
    pub spacing: Option<EdgeInsets>,
    pub borders: Option<BorderStyle>,
    pub shadows: Option<ShadowStyle>,
}

#[derive(Debug, Clone)]
pub struct BorderStyle {
    pub width: f32,
    pub color: Color,
    pub style: BorderLineStyle,
    pub radius: BorderRadius,
}

#[derive(Debug, Clone)]
pub enum BorderLineStyle {
    Solid,
    Dashed,
    Dotted,
    Double,
    Groove,
    Ridge,
    Inset,
    Outset,
}

#[derive(Debug, Clone)]
pub struct BorderRadius {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_left: f32,
    pub bottom_right: f32,
}

#[derive(Debug, Clone)]
pub struct ShadowStyle {
    pub offset: Vec2,
    pub blur_radius: f32,
    pub spread_radius: f32,
    pub color: Color,
    pub inset: bool,
}

#[derive(Debug, Clone)]
pub enum ReorganizationStrategy {
    Stack,
    Wrap,
    Collapse,
    Prioritize(Vec<String>),
    Merge(Vec<String>),
    Split(String, Vec<String>),
}

#[derive(Debug, Clone)]
pub struct AdaptiveCondition {
    pub condition_type: ConditionType,
    pub operator: ComparisonOperator,
    pub value: ConditionValue,
}

#[derive(Debug, Clone)]
pub enum ConditionType {
    ScreenWidth,
    ScreenHeight,
    AspectRatio,
    DPI,
    AvailableSpace,
    ContentSize,
    UserPreference(String),
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Between,
    NotBetween,
}

#[derive(Debug, Clone)]
pub enum ConditionValue {
    Number(f32),
    Range(f32, f32),
    String(String),
    Boolean(bool),
}

#[derive(Debug, Clone)]
pub struct ComponentState {
    pub visible: bool,
    pub bounds: Rectangle,
    pub current_layout: LayoutType,
    pub animation_state: ComponentAnimationState,
    pub user_state: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ComponentAnimationState {
    pub transition_progress: f32,
    pub target_bounds: Option<Rectangle>,
    pub target_layout: Option<LayoutType>,
    pub animation_duration: f32,
    pub easing_function: EasingFunction,
}

#[derive(Debug, Clone)]
pub enum EasingFunction {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Cubic,
    Bounce,
    Elastic,
}

pub struct LayoutCache {
    cached_layouts: HashMap<String, CachedLayout>,
    cache_size_limit: usize,
    cache_lifetime: f32,
}

#[derive(Debug, Clone)]
pub struct CachedLayout {
    pub breakpoint: Breakpoint,
    pub screen_size: Vec2,
    pub layout_result: LayoutResult,
    pub timestamp: f32,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LayoutResult {
    pub bounds: Rectangle,
    pub children_bounds: HashMap<String, Rectangle>,
    pub overflow_info: OverflowInfo,
}

#[derive(Debug, Clone)]
pub struct OverflowInfo {
    pub has_overflow: bool,
    pub overflow_direction: Vec2,
    pub content_size: Vec2,
    pub visible_area: Rectangle,
}

pub struct LayoutAnimationController {
    active_animations: HashMap<String, LayoutAnimation>,
    default_duration: f32,
    default_easing: EasingFunction,
    stagger_delay: f32,
}

#[derive(Debug, Clone)]
pub struct LayoutAnimation {
    pub target_id: String,
    pub start_bounds: Rectangle,
    pub end_bounds: Rectangle,
    pub start_time: f32,
    pub duration: f32,
    pub easing: EasingFunction,
    pub progress: f32,
}

#[derive(Debug, Clone)]
pub struct TransitionAnimation {
    pub duration: f32,
    pub easing: EasingFunction,
    pub properties: Vec<AnimatedProperty>,
}

#[derive(Debug, Clone)]
pub enum AnimatedProperty {
    Position,
    Size,
    Opacity,
    Scale,
    Rotation,
    Color,
    Custom(String),
}

impl ResponsiveLayoutSystem {
    pub fn new(modern_ui: ModernUISystem, initial_screen_size: Vec2) -> Self {
        let mut system = Self {
            modern_ui,
            current_breakpoint: Self::calculate_breakpoint(initial_screen_size.x),
            screen_size: initial_screen_size,
            layout_containers: Vec::new(),
            breakpoints: BreakpointConfiguration::default(),
            scaling_system: UIScalingSystem::new(),
            adaptive_components: Vec::new(),
            orientation: Self::calculate_orientation(initial_screen_size),
            dpi_scale: 1.0,
            layout_cache: LayoutCache::new(),
            animation_controller: LayoutAnimationController::new(),
        };

        system.create_default_containers();
        system
    }

    fn calculate_breakpoint(width: f32) -> Breakpoint {
        if width < 768.0 {
            Breakpoint::Mobile
        } else if width < 1024.0 {
            Breakpoint::Tablet
        } else if width < 1440.0 {
            Breakpoint::Desktop
        } else if width < 1920.0 {
            Breakpoint::Large
        } else {
            Breakpoint::UltraWide
        }
    }

    fn calculate_orientation(size: Vec2) -> ScreenOrientation {
        let ratio = size.x / size.y;
        if ratio > 1.2 {
            ScreenOrientation::Landscape
        } else if ratio < 0.8 {
            ScreenOrientation::Portrait
        } else {
            ScreenOrientation::Square
        }
    }

    fn create_default_containers(&mut self) {
        // Main viewport container
        self.layout_containers.push(LayoutContainer {
            id: "main_viewport".to_string(),
            layout_type: LayoutType::Adaptive(AdaptiveLayout {
                breakpoint_layouts: {
                    let mut layouts = HashMap::new();
                    layouts.insert(Breakpoint::Mobile, LayoutType::Stack(StackLayout {
                        direction: StackDirection::Vertical,
                        spacing: 8.0,
                        alignment: StackAlignment::Stretch,
                        distribution: StackDistribution::Fill,
                    }));
                    layouts.insert(Breakpoint::Desktop, LayoutType::Grid(GridLayout {
                        template_columns: vec![
                            GridTrack::Fixed(280.0), // Tool palette
                            GridTrack::Fraction(1.0), // Main content
                            GridTrack::Fixed(320.0), // Properties panel
                        ],
                        template_rows: vec![
                            GridTrack::Fixed(60.0), // Top toolbar
                            GridTrack::Fraction(1.0), // Main area
                            GridTrack::Fixed(30.0), // Status bar
                        ],
                        gap: Vec2::new(8.0, 8.0),
                        auto_flow: GridAutoFlow::Row,
                        responsive_columns: HashMap::new(),
                        responsive_rows: HashMap::new(),
                    }));
                    layouts
                },
                transition_animations: HashMap::new(),
            }),
            bounds: Rectangle {
                x: 0.0,
                y: 0.0,
                width: self.screen_size.x,
                height: self.screen_size.y,
            },
            children: vec![
                "top_toolbar".to_string(),
                "tool_palette".to_string(),
                "main_canvas".to_string(),
                "properties_panel".to_string(),
                "status_bar".to_string(),
            ],
            parent: None,
            responsive_rules: ResponsiveRules::default(),
            visibility_rules: VisibilityRules::default(),
            z_index: 0,
            overflow: OverflowBehavior::Hidden,
        });

        // Tool palette container
        self.layout_containers.push(LayoutContainer {
            id: "tool_palette".to_string(),
            layout_type: LayoutType::Flexbox(FlexboxLayout {
                direction: FlexDirection::Column,
                wrap: FlexWrap::NoWrap,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Stretch,
                align_content: AlignContent::FlexStart,
                gap: 4.0,
                responsive_direction: {
                    let mut dirs = HashMap::new();
                    dirs.insert(Breakpoint::Mobile, FlexDirection::Row);
                    dirs
                },
            }),
            bounds: Rectangle::default(),
            children: vec!["tool_groups".to_string(), "favorites".to_string()],
            parent: Some("main_viewport".to_string()),
            responsive_rules: ResponsiveRules {
                visibility: {
                    let mut vis = HashMap::new();
                    vis.insert(Breakpoint::Mobile, false); // Hidden on mobile by default
                    vis.insert(Breakpoint::Tablet, true);
                    vis.insert(Breakpoint::Desktop, true);
                    vis
                },
                sizes: {
                    let mut sizes = HashMap::new();
                    sizes.insert(Breakpoint::Mobile, ResponsiveSize {
                        width: SizeConstraint::Percentage(100.0),
                        height: SizeConstraint::Fixed(60.0),
                        min_width: None,
                        max_width: None,
                        min_height: None,
                        max_height: None,
                        aspect_ratio: None,
                    });
                    sizes.insert(Breakpoint::Desktop, ResponsiveSize {
                        width: SizeConstraint::Fixed(280.0),
                        height: SizeConstraint::Percentage(100.0),
                        min_width: Some(240.0),
                        max_width: Some(320.0),
                        min_height: None,
                        max_height: None,
                        aspect_ratio: None,
                    });
                    sizes
                },
                margins: HashMap::new(),
                padding: HashMap::new(),
                typography: HashMap::new(),
                colors: HashMap::new(),
            },
            visibility_rules: VisibilityRules::default(),
            z_index: 10,
            overflow: OverflowBehavior::Auto,
        });

        // Properties panel container
        self.layout_containers.push(LayoutContainer {
            id: "properties_panel".to_string(),
            layout_type: LayoutType::Flexbox(FlexboxLayout {
                direction: FlexDirection::Column,
                wrap: FlexWrap::NoWrap,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Stretch,
                align_content: AlignContent::FlexStart,
                gap: 8.0,
                responsive_direction: HashMap::new(),
            }),
            bounds: Rectangle::default(),
            children: vec!["element_properties".to_string(), "scene_hierarchy".to_string()],
            parent: Some("main_viewport".to_string()),
            responsive_rules: ResponsiveRules {
                visibility: {
                    let mut vis = HashMap::new();
                    vis.insert(Breakpoint::Mobile, false);
                    vis.insert(Breakpoint::Tablet, false);
                    vis.insert(Breakpoint::Desktop, true);
                    vis
                },
                sizes: {
                    let mut sizes = HashMap::new();
                    sizes.insert(Breakpoint::Desktop, ResponsiveSize {
                        width: SizeConstraint::Fixed(320.0),
                        height: SizeConstraint::Percentage(100.0),
                        min_width: Some(280.0),
                        max_width: Some(400.0),
                        min_height: None,
                        max_height: None,
                        aspect_ratio: None,
                    });
                    sizes
                },
                margins: HashMap::new(),
                padding: HashMap::new(),
                typography: HashMap::new(),
                colors: HashMap::new(),
            },
            visibility_rules: VisibilityRules::default(),
            z_index: 10,
            overflow: OverflowBehavior::Auto,
        });
    }

    pub fn update_screen_size(&mut self, new_size: Vec2, dpi_scale: f32) -> RobinResult<()> {
        let old_breakpoint = self.current_breakpoint;
        let old_orientation = self.orientation;

        self.screen_size = new_size;
        self.dpi_scale = dpi_scale;
        self.current_breakpoint = Self::calculate_breakpoint(new_size.x);
        self.orientation = Self::calculate_orientation(new_size);

        // Update main viewport bounds
        if let Some(main_container) = self.layout_containers.iter_mut().find(|c| c.id == "main_viewport") {
            main_container.bounds.width = new_size.x;
            main_container.bounds.height = new_size.y;
        }

        // Trigger layout recalculation if breakpoint or orientation changed
        if old_breakpoint != self.current_breakpoint || old_orientation != self.orientation {
            self.invalidate_layout_cache();
            self.trigger_adaptive_behaviors();
            self.recalculate_all_layouts()?;
        }

        Ok(())
    }

    pub fn add_container(&mut self, container: LayoutContainer) {
        self.layout_containers.push(container);
        self.invalidate_layout_cache();
    }

    pub fn add_adaptive_component(&mut self, component: AdaptiveComponent) {
        self.adaptive_components.push(component);
    }

    pub fn update(&mut self, delta_time: f32, input: &InputManager) -> RobinResult<()> {
        // Update animations
        self.animation_controller.update(delta_time);

        // Update adaptive components
        for component in &mut self.adaptive_components {
            self.update_adaptive_component(component, delta_time, input)?;
        }

        // Update layout cache lifetime
        self.layout_cache.update(delta_time);

        Ok(())
    }

    fn update_adaptive_component(&mut self, component: &mut AdaptiveComponent, delta_time: f32, input: &InputManager) -> RobinResult<()> {
        for behavior in &component.adaptive_behaviors {
            if self.should_trigger_behavior(behavior, input) {
                self.execute_adaptive_action(&behavior.action, &component.id)?;
            }
        }

        // Update component animation state
        if component.state.animation_state.transition_progress < 1.0 {
            component.state.animation_state.transition_progress += delta_time / component.state.animation_state.animation_duration;
            component.state.animation_state.transition_progress = component.state.animation_state.transition_progress.min(1.0);

            // Apply eased progress to bounds
            if let (Some(target_bounds), progress) = (&component.state.animation_state.target_bounds, component.state.animation_state.transition_progress) {
                let eased_progress = self.apply_easing(progress, &component.state.animation_state.easing_function);
                component.state.bounds = self.interpolate_rectangles(&component.state.bounds, target_bounds, eased_progress);
            }
        }

        Ok(())
    }

    fn should_trigger_behavior(&self, behavior: &AdaptiveBehavior, input: &InputManager) -> bool {
        // Check conditions
        for condition in &behavior.conditions {
            if !self.evaluate_condition(condition) {
                return false;
            }
        }

        // Check trigger
        match &behavior.trigger {
            AdaptiveTrigger::BreakpointChange(breakpoint) => {
                self.current_breakpoint == *breakpoint
            }
            AdaptiveTrigger::OrientationChange(orientation) => {
                self.orientation == *orientation
            }
            AdaptiveTrigger::SizeChange { width_threshold, height_threshold } => {
                if let Some(width_threshold) = width_threshold {
                    if self.screen_size.x < *width_threshold {
                        return true;
                    }
                }
                if let Some(height_threshold) = height_threshold {
                    if self.screen_size.y < *height_threshold {
                        return true;
                    }
                }
                false
            }
            AdaptiveTrigger::UserInteraction(interaction_type) => {
                match interaction_type {
                    InteractionType::Hover => {
                        // Would check for hover state
                        false
                    }
                    InteractionType::Click => {
                        input.is_mouse_button_just_pressed(winit::event::MouseButton::Left)
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    fn evaluate_condition(&self, condition: &AdaptiveCondition) -> bool {
        let actual_value = match &condition.condition_type {
            ConditionType::ScreenWidth => self.screen_size.x,
            ConditionType::ScreenHeight => self.screen_size.y,
            ConditionType::AspectRatio => self.screen_size.x / self.screen_size.y,
            ConditionType::DPI => self.dpi_scale,
            _ => return true, // Unknown conditions default to true
        };

        match (&condition.operator, &condition.value) {
            (ComparisonOperator::Greater, ConditionValue::Number(threshold)) => actual_value > *threshold,
            (ComparisonOperator::Less, ConditionValue::Number(threshold)) => actual_value < *threshold,
            (ComparisonOperator::Equal, ConditionValue::Number(target)) => (actual_value - target).abs() < 0.001,
            (ComparisonOperator::Between, ConditionValue::Range(min, max)) => actual_value >= *min && actual_value <= *max,
            _ => true,
        }
    }

    fn execute_adaptive_action(&mut self, action: &AdaptiveAction, component_id: &str) -> RobinResult<()> {
        match action {
            AdaptiveAction::ChangeLayout(new_layout) => {
                if let Some(container) = self.layout_containers.iter_mut().find(|c| c.id == component_id) {
                    container.layout_type = new_layout.clone();
                    self.invalidate_layout_cache();
                }
            }
            AdaptiveAction::ToggleVisibility(visible) => {
                if let Some(container) = self.layout_containers.iter_mut().find(|c| c.id == component_id) {
                    container.visibility_rules.breakpoint_visibility.insert(self.current_breakpoint, *visible);
                }
                if let Some(component) = self.adaptive_components.iter_mut().find(|c| c.id == component_id) {
                    component.state.visible = *visible;
                }
            }
            AdaptiveAction::Resize { width, height } => {
                if let Some(component) = self.adaptive_components.iter_mut().find(|c| c.id == component_id) {
                    let mut new_bounds = component.state.bounds;
                    if let Some(width) = width {
                        new_bounds.width = *width;
                    }
                    if let Some(height) = height {
                        new_bounds.height = *height;
                    }

                    // Start animation to new bounds
                    component.state.animation_state.target_bounds = Some(new_bounds);
                    component.state.animation_state.transition_progress = 0.0;
                }
            }
            AdaptiveAction::Reposition(new_position) => {
                if let Some(component) = self.adaptive_components.iter_mut().find(|c| c.id == component_id) {
                    let mut new_bounds = component.state.bounds;
                    new_bounds.x = new_position.x;
                    new_bounds.y = new_position.y;

                    component.state.animation_state.target_bounds = Some(new_bounds);
                    component.state.animation_state.transition_progress = 0.0;
                }
            }
            _ => {
                println!("Adaptive action not implemented: {:?}", action);
            }
        }

        Ok(())
    }

    fn trigger_adaptive_behaviors(&mut self) {
        // Create a list of behaviors to trigger based on current state
        let mut behaviors_to_trigger = Vec::new();

        for component in &self.adaptive_components {
            for behavior in &component.adaptive_behaviors {
                match &behavior.trigger {
                    AdaptiveTrigger::BreakpointChange(breakpoint) => {
                        if self.current_breakpoint == *breakpoint {
                            behaviors_to_trigger.push((component.id.clone(), behavior.clone()));
                        }
                    }
                    AdaptiveTrigger::OrientationChange(orientation) => {
                        if self.orientation == *orientation {
                            behaviors_to_trigger.push((component.id.clone(), behavior.clone()));
                        }
                    }
                    _ => {}
                }
            }
        }

        // Execute behaviors
        for (component_id, behavior) in behaviors_to_trigger {
            if let Err(e) = self.execute_adaptive_action(&behavior.action, &component_id) {
                println!("Failed to execute adaptive action: {:?}", e);
            }
        }
    }

    fn recalculate_all_layouts(&mut self) -> RobinResult<()> {
        // Sort containers by dependency order (parents before children)
        let mut sorted_containers = self.layout_containers.clone();
        sorted_containers.sort_by(|a, b| {
            if a.parent.is_none() && b.parent.is_some() {
                std::cmp::Ordering::Less
            } else if a.parent.is_some() && b.parent.is_none() {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        });

        // Calculate layouts
        for container in &mut sorted_containers {
            self.calculate_container_layout(container)?;
        }

        // Update the original containers
        for (i, container) in self.layout_containers.iter_mut().enumerate() {
            if let Some(updated_container) = sorted_containers.get(i) {
                container.bounds = updated_container.bounds;
            }
        }

        Ok(())
    }

    fn calculate_container_layout(&mut self, container: &mut LayoutContainer) -> RobinResult<()> {
        // Check cache first
        let cache_key = format!("{}_{:?}_{:?}", container.id, self.current_breakpoint, self.screen_size);
        if let Some(cached_result) = self.layout_cache.get(&cache_key) {
            container.bounds = cached_result.layout_result.bounds;
            return Ok(());
        }

        // Get responsive properties for current breakpoint
        let responsive_size = container.responsive_rules.sizes.get(&self.current_breakpoint);
        let responsive_margins = container.responsive_rules.margins.get(&self.current_breakpoint);
        let responsive_padding = container.responsive_rules.padding.get(&self.current_breakpoint);

        // Calculate base bounds
        let mut bounds = container.bounds;

        if let Some(size) = responsive_size {
            bounds.width = self.calculate_size_constraint(&size.width, bounds.width, true);
            bounds.height = self.calculate_size_constraint(&size.height, bounds.height, false);

            // Apply size constraints
            if let Some(min_width) = size.min_width {
                bounds.width = bounds.width.max(min_width);
            }
            if let Some(max_width) = size.max_width {
                bounds.width = bounds.width.min(max_width);
            }
            if let Some(min_height) = size.min_height {
                bounds.height = bounds.height.max(min_height);
            }
            if let Some(max_height) = size.max_height {
                bounds.height = bounds.height.min(max_height);
            }

            // Apply aspect ratio
            if let Some(aspect_ratio) = size.aspect_ratio {
                let target_height = bounds.width / aspect_ratio;
                if target_height != bounds.height {
                    // Decide whether to constrain width or height based on which changes less
                    let width_change = (bounds.height * aspect_ratio - bounds.width).abs();
                    let height_change = (target_height - bounds.height).abs();

                    if width_change < height_change {
                        bounds.width = bounds.height * aspect_ratio;
                    } else {
                        bounds.height = target_height;
                    }
                }
            }
        }

        // Apply margins
        if let Some(margins) = responsive_margins {
            bounds.x += margins.left;
            bounds.y += margins.top;
            bounds.width -= margins.left + margins.right;
            bounds.height -= margins.top + margins.bottom;
        }

        // Calculate layout specific to container type
        match &container.layout_type {
            LayoutType::Flexbox(flex_layout) => {
                self.calculate_flexbox_layout(container, flex_layout, &mut bounds)?;
            }
            LayoutType::Grid(grid_layout) => {
                self.calculate_grid_layout(container, grid_layout, &mut bounds)?;
            }
            LayoutType::Absolute(absolute_layout) => {
                self.calculate_absolute_layout(container, absolute_layout, &mut bounds)?;
            }
            LayoutType::Adaptive(adaptive_layout) => {
                if let Some(layout_for_breakpoint) = adaptive_layout.breakpoint_layouts.get(&self.current_breakpoint) {
                    let mut temp_container = container.clone();
                    temp_container.layout_type = layout_for_breakpoint.clone();
                    self.calculate_container_layout(&mut temp_container)?;
                    bounds = temp_container.bounds;
                }
            }
            _ => {
                // Default layout calculation
            }
        }

        container.bounds = bounds;

        // Cache the result
        let layout_result = LayoutResult {
            bounds,
            children_bounds: HashMap::new(), // Would be populated in real implementation
            overflow_info: OverflowInfo {
                has_overflow: false,
                overflow_direction: Vec2::new(0.0, 0.0),
                content_size: Vec2::new(bounds.width, bounds.height),
                visible_area: bounds,
            },
        };

        self.layout_cache.set(cache_key, CachedLayout {
            breakpoint: self.current_breakpoint,
            screen_size: self.screen_size,
            layout_result,
            timestamp: 0.0, // Would use actual time
            dependencies: vec![container.id.clone()],
        });

        Ok(())
    }

    fn calculate_size_constraint(&self, constraint: &SizeConstraint, current_size: f32, is_width: bool) -> f32 {
        match constraint {
            SizeConstraint::Fixed(size) => *size * self.scaling_system.get_scale_factor(self.current_breakpoint),
            SizeConstraint::Percentage(percent) => {
                let reference_size = if is_width { self.screen_size.x } else { self.screen_size.y };
                reference_size * (percent / 100.0)
            }
            SizeConstraint::Auto => current_size,
            SizeConstraint::FitContent => current_size, // Would calculate based on content
            SizeConstraint::MinContent => 50.0, // Minimum reasonable size
            SizeConstraint::MaxContent => {
                let reference_size = if is_width { self.screen_size.x } else { self.screen_size.y };
                reference_size * 0.8 // Maximum 80% of screen
            }
            SizeConstraint::Viewport { width, height } => {
                if is_width {
                    width.unwrap_or(1.0) * self.screen_size.x
                } else {
                    height.unwrap_or(1.0) * self.screen_size.y
                }
            }
        }
    }

    fn calculate_flexbox_layout(&mut self, container: &LayoutContainer, flex_layout: &FlexboxLayout, bounds: &mut Rectangle) -> RobinResult<()> {
        // Get effective direction for current breakpoint
        let direction = flex_layout.responsive_direction.get(&self.current_breakpoint)
            .unwrap_or(&flex_layout.direction);

        // This would contain the full flexbox layout algorithm
        // For now, just apply basic spacing
        let gap = flex_layout.gap * self.scaling_system.get_scale_factor(self.current_breakpoint);

        match direction {
            FlexDirection::Column => {
                // Arrange children vertically
                let child_count = container.children.len() as f32;
                if child_count > 0.0 {
                    let available_height = bounds.height - gap * (child_count - 1.0);
                    let child_height = available_height / child_count;

                    // This would set child bounds in a real implementation
                    println!("Flexbox column layout: {} children, {}px each", child_count, child_height);
                }
            }
            FlexDirection::Row => {
                // Arrange children horizontally
                let child_count = container.children.len() as f32;
                if child_count > 0.0 {
                    let available_width = bounds.width - gap * (child_count - 1.0);
                    let child_width = available_width / child_count;

                    println!("Flexbox row layout: {} children, {}px each", child_count, child_width);
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn calculate_grid_layout(&mut self, container: &LayoutContainer, grid_layout: &GridLayout, bounds: &mut Rectangle) -> RobinResult<()> {
        // Get responsive grid configuration
        let columns = grid_layout.responsive_columns.get(&self.current_breakpoint)
            .unwrap_or(&grid_layout.template_columns);
        let rows = grid_layout.responsive_rows.get(&self.current_breakpoint)
            .unwrap_or(&grid_layout.template_rows);

        let gap = grid_layout.gap * self.scaling_system.get_scale_factor(self.current_breakpoint);

        // Calculate column sizes
        let total_gap_width = gap.x * (columns.len() as f32 - 1.0);
        let available_width = bounds.width - total_gap_width;

        let mut column_sizes = Vec::new();
        let mut total_fractions = 0.0;
        let mut fixed_width = 0.0;

        // First pass: calculate fixed sizes and count fractions
        for track in columns {
            match track {
                GridTrack::Fixed(size) => {
                    let scaled_size = size * self.scaling_system.get_scale_factor(self.current_breakpoint);
                    column_sizes.push(scaled_size);
                    fixed_width += scaled_size;
                }
                GridTrack::Fraction(fraction) => {
                    total_fractions += fraction;
                    column_sizes.push(0.0); // Placeholder
                }
                _ => {
                    column_sizes.push(100.0); // Default size
                    fixed_width += 100.0;
                }
            }
        }

        // Second pass: calculate fractional sizes
        let available_for_fractions = available_width - fixed_width;
        let mut column_index = 0;
        for track in columns {
            if let GridTrack::Fraction(fraction) = track {
                if total_fractions > 0.0 {
                    column_sizes[column_index] = available_for_fractions * (fraction / total_fractions);
                }
            }
            column_index += 1;
        }

        // Similar calculation for rows...
        let total_gap_height = gap.y * (rows.len() as f32 - 1.0);
        let available_height = bounds.height - total_gap_height;

        println!("Grid layout: {}x{} with gaps {:?}", columns.len(), rows.len(), gap);

        Ok(())
    }

    fn calculate_absolute_layout(&mut self, container: &LayoutContainer, absolute_layout: &AbsoluteLayout, bounds: &mut Rectangle) -> RobinResult<()> {
        // Get responsive positioning
        let positioning = absolute_layout.responsive_positioning.get(&self.current_breakpoint)
            .unwrap_or(&absolute_layout.positioning);

        match positioning {
            PositioningMode::Absolute { position } => {
                bounds.x = position.x;
                bounds.y = position.y;
            }
            PositioningMode::Relative { offset } => {
                bounds.x += offset.x;
                bounds.y += offset.y;
            }
            PositioningMode::Fixed { position } => {
                bounds.x = position.x;
                bounds.y = position.y;
            }
            _ => {}
        }

        // Apply anchor points
        self.apply_anchor_configuration(&absolute_layout.anchor_points, bounds);

        Ok(())
    }

    fn apply_anchor_configuration(&self, anchors: &AnchorConfiguration, bounds: &mut Rectangle) {
        // Apply horizontal anchoring
        match &anchors.horizontal {
            HorizontalAnchor::Left(offset) => {
                bounds.x = *offset + anchors.margins.left;
            }
            HorizontalAnchor::Right(offset) => {
                bounds.x = self.screen_size.x - bounds.width - *offset - anchors.margins.right;
            }
            HorizontalAnchor::Center => {
                bounds.x = (self.screen_size.x - bounds.width) * 0.5;
            }
            HorizontalAnchor::LeftRight { left, right } => {
                bounds.x = *left + anchors.margins.left;
                bounds.width = self.screen_size.x - *left - *right - anchors.margins.left - anchors.margins.right;
            }
            HorizontalAnchor::Percentage(percent) => {
                bounds.x = self.screen_size.x * (percent / 100.0) - bounds.width * 0.5;
            }
        }

        // Apply vertical anchoring
        match &anchors.vertical {
            VerticalAnchor::Top(offset) => {
                bounds.y = *offset + anchors.margins.top;
            }
            VerticalAnchor::Bottom(offset) => {
                bounds.y = self.screen_size.y - bounds.height - *offset - anchors.margins.bottom;
            }
            VerticalAnchor::Center => {
                bounds.y = (self.screen_size.y - bounds.height) * 0.5;
            }
            VerticalAnchor::TopBottom { top, bottom } => {
                bounds.y = *top + anchors.margins.top;
                bounds.height = self.screen_size.y - *top - *bottom - anchors.margins.top - anchors.margins.bottom;
            }
            VerticalAnchor::Percentage(percent) => {
                bounds.y = self.screen_size.y * (percent / 100.0) - bounds.height * 0.5;
            }
        }
    }

    fn invalidate_layout_cache(&mut self) {
        self.layout_cache.clear();
    }

    fn apply_easing(&self, t: f32, easing: &EasingFunction) -> f32 {
        match easing {
            EasingFunction::Linear => t,
            EasingFunction::EaseOut => 1.0 - (1.0 - t).powi(3),
            EasingFunction::EaseIn => t.powi(3),
            EasingFunction::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
            _ => t,
        }
    }

    fn interpolate_rectangles(&self, start: &Rectangle, end: &Rectangle, progress: f32) -> Rectangle {
        Rectangle {
            x: start.x + (end.x - start.x) * progress,
            y: start.y + (end.y - start.y) * progress,
            width: start.width + (end.width - start.width) * progress,
            height: start.height + (end.height - start.height) * progress,
        }
    }

    pub fn get_current_breakpoint(&self) -> Breakpoint {
        self.current_breakpoint
    }

    pub fn get_container_bounds(&self, container_id: &str) -> Option<Rectangle> {
        self.layout_containers.iter()
            .find(|c| c.id == container_id)
            .map(|c| c.bounds)
    }

    pub fn is_container_visible(&self, container_id: &str) -> bool {
        if let Some(container) = self.layout_containers.iter().find(|c| c.id == container_id) {
            container.visibility_rules.breakpoint_visibility
                .get(&self.current_breakpoint)
                .copied()
                .unwrap_or(true)
        } else {
            false
        }
    }

    pub fn render(&self, renderer: &mut dyn Renderer) -> RobinResult<()> {
        // Render containers in z-index order
        let mut sorted_containers = self.layout_containers.clone();
        sorted_containers.sort_by_key(|c| c.z_index);

        for container in &sorted_containers {
            if self.is_container_visible(&container.id) {
                self.render_container(container, renderer)?;
            }
        }

        Ok(())
    }

    fn render_container(&self, container: &LayoutContainer, renderer: &mut dyn Renderer) -> RobinResult<()> {
        // Render container background if needed
        let theme = self.modern_ui.get_theme();

        match &container.layout_type {
            LayoutType::Grid(_) => {
                // Render grid lines in debug mode
                if self.should_show_debug_info() {
                    let grid_color = Color::new(0.3, 0.3, 0.3, 0.5);
                    renderer.stroke_rect(&container.bounds, &grid_color, 1.0)?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn should_show_debug_info(&self) -> bool {
        // Would be configurable
        false
    }
}

// Implementation of helper structures
impl Default for BreakpointConfiguration {
    fn default() -> Self {
        Self {
            mobile_max: 768.0,
            tablet_max: 1024.0,
            desktop_max: 1440.0,
            large_max: 1920.0,
            custom_breakpoints: HashMap::new(),
        }
    }
}

impl Default for ResponsiveRules {
    fn default() -> Self {
        Self {
            visibility: HashMap::new(),
            sizes: HashMap::new(),
            margins: HashMap::new(),
            padding: HashMap::new(),
            typography: HashMap::new(),
            colors: HashMap::new(),
        }
    }
}

impl Default for VisibilityRules {
    fn default() -> Self {
        Self {
            breakpoint_visibility: HashMap::new(),
            orientation_visibility: HashMap::new(),
            conditional_visibility: Vec::new(),
        }
    }
}

impl Default for Rectangle {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }
}

impl UIScalingSystem {
    fn new() -> Self {
        Self {
            base_scale: 1.0,
            dpi_aware: true,
            scaling_mode: ScalingMode::DPIAware,
            custom_scale_factors: {
                let mut factors = HashMap::new();
                factors.insert(Breakpoint::Mobile, 0.9);
                factors.insert(Breakpoint::Tablet, 1.0);
                factors.insert(Breakpoint::Desktop, 1.0);
                factors.insert(Breakpoint::Large, 1.1);
                factors.insert(Breakpoint::UltraWide, 1.2);
                factors
            },
            font_scaling: FontScalingConfiguration {
                base_size: 14.0,
                scale_factors: HashMap::new(),
                min_size: 10.0,
                max_size: 24.0,
                line_height_adjustment: 1.4,
            },
            icon_scaling: IconScalingConfiguration {
                base_size: 16.0,
                scale_factors: HashMap::new(),
                crisp_edges: true,
                vector_scaling: true,
            },
            spacing_scaling: SpacingScalingConfiguration {
                base_unit: 8.0,
                scale_factors: HashMap::new(),
                maintain_proportions: true,
            },
        }
    }

    fn get_scale_factor(&self, breakpoint: Breakpoint) -> f32 {
        self.custom_scale_factors.get(&breakpoint).copied().unwrap_or(1.0) * self.base_scale
    }
}

impl LayoutCache {
    fn new() -> Self {
        Self {
            cached_layouts: HashMap::new(),
            cache_size_limit: 100,
            cache_lifetime: 300.0, // 5 minutes
        }
    }

    fn get(&self, key: &str) -> Option<&CachedLayout> {
        self.cached_layouts.get(key)
    }

    fn set(&mut self, key: String, layout: CachedLayout) {
        // Remove oldest entries if cache is full
        if self.cached_layouts.len() >= self.cache_size_limit {
            let oldest_key = self.cached_layouts.iter()
                .min_by(|a, b| a.1.timestamp.partial_cmp(&b.1.timestamp).unwrap())
                .map(|(k, _)| k.clone());

            if let Some(key_to_remove) = oldest_key {
                self.cached_layouts.remove(&key_to_remove);
            }
        }

        self.cached_layouts.insert(key, layout);
    }

    fn clear(&mut self) {
        self.cached_layouts.clear();
    }

    fn update(&mut self, delta_time: f32) {
        // Remove expired entries
        let current_time = 0.0; // Would use actual time
        self.cached_layouts.retain(|_, layout| {
            current_time - layout.timestamp < self.cache_lifetime
        });
    }
}

impl LayoutAnimationController {
    fn new() -> Self {
        Self {
            active_animations: HashMap::new(),
            default_duration: 0.3,
            default_easing: EasingFunction::EaseOut,
            stagger_delay: 0.05,
        }
    }

    fn update(&mut self, delta_time: f32) {
        // Update active animations
        let mut completed_animations = Vec::new();

        for (id, animation) in &mut self.active_animations {
            animation.progress += delta_time / animation.duration;
            if animation.progress >= 1.0 {
                animation.progress = 1.0;
                completed_animations.push(id.clone());
            }
        }

        // Remove completed animations
        for id in completed_animations {
            self.active_animations.remove(&id);
        }
    }
}

// Placeholder trait for renderer
pub trait Renderer {
    fn stroke_rect(&mut self, rect: &Rectangle, color: &Color, width: f32) -> RobinResult<()>;
    fn fill_rect(&mut self, rect: &Rectangle, color: &Color) -> RobinResult<()>;
    fn render_text(&mut self, text: &str, rect: &Rectangle, style: &TextStyle) -> RobinResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breakpoint_calculation() {
        assert_eq!(ResponsiveLayoutSystem::calculate_breakpoint(500.0), Breakpoint::Mobile);
        assert_eq!(ResponsiveLayoutSystem::calculate_breakpoint(800.0), Breakpoint::Tablet);
        assert_eq!(ResponsiveLayoutSystem::calculate_breakpoint(1200.0), Breakpoint::Desktop);
        assert_eq!(ResponsiveLayoutSystem::calculate_breakpoint(1600.0), Breakpoint::Large);
        assert_eq!(ResponsiveLayoutSystem::calculate_breakpoint(2000.0), Breakpoint::UltraWide);
    }

    #[test]
    fn test_orientation_calculation() {
        assert_eq!(ResponsiveLayoutSystem::calculate_orientation(Vec2::new(1920.0, 1080.0)), ScreenOrientation::Landscape);
        assert_eq!(ResponsiveLayoutSystem::calculate_orientation(Vec2::new(768.0, 1024.0)), ScreenOrientation::Portrait);
        assert_eq!(ResponsiveLayoutSystem::calculate_orientation(Vec2::new(1000.0, 1000.0)), ScreenOrientation::Square);
    }

    #[test]
    fn test_responsive_layout_system_creation() {
        let modern_ui = ModernUISystem::new();
        let screen_size = Vec2::new(1920.0, 1080.0);
        let layout_system = ResponsiveLayoutSystem::new(modern_ui, screen_size);

        assert_eq!(layout_system.current_breakpoint, Breakpoint::UltraWide);
        assert_eq!(layout_system.orientation, ScreenOrientation::Landscape);
        assert!(!layout_system.layout_containers.is_empty());
    }

    #[test]
    fn test_layout_container_visibility() {
        let modern_ui = ModernUISystem::new();
        let screen_size = Vec2::new(1920.0, 1080.0);
        let layout_system = ResponsiveLayoutSystem::new(modern_ui, screen_size);

        // Test that tool palette is visible on desktop
        assert!(layout_system.is_container_visible("tool_palette"));

        // Test that properties panel is visible on desktop
        assert!(layout_system.is_container_visible("properties_panel"));
    }

    #[test]
    fn test_size_constraint_calculation() {
        let modern_ui = ModernUISystem::new();
        let screen_size = Vec2::new(1920.0, 1080.0);
        let layout_system = ResponsiveLayoutSystem::new(modern_ui, screen_size);

        // Test fixed size
        let fixed_size = layout_system.calculate_size_constraint(&SizeConstraint::Fixed(100.0), 0.0, true);
        assert_eq!(fixed_size, 100.0);

        // Test percentage size
        let percentage_size = layout_system.calculate_size_constraint(&SizeConstraint::Percentage(50.0), 0.0, true);
        assert_eq!(percentage_size, 960.0); // 50% of 1920

        // Test viewport size
        let viewport_size = layout_system.calculate_size_constraint(
            &SizeConstraint::Viewport { width: Some(0.8), height: None },
            0.0,
            true
        );
        assert_eq!(viewport_size, 1536.0); // 80% of 1920
    }
}