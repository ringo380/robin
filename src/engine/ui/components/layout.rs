// Robin Game Engine - Layout Components
// Grid, flex, containers, and layout utilities

use super::*;
use crate::engine::error::RobinResult;
use crate::engine::generation::templates::UITheme;
use crate::engine::ui::modern_architecture::{ComponentContext, Bounds, RenderPrimitive};
use serde::{Serialize, Deserialize};

/// Grid layout component
#[derive(Debug)]
pub struct Grid {
    id: ComponentId,
    props: GridProps,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridProps {
    pub base: ComponentProps,
    pub columns: GridColumns,
    pub gap: f32,
    pub align_items: AlignItems,
    pub justify_content: JustifyContent,
    pub responsive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GridColumns {
    Fixed(u32),           // Fixed number of columns
    Auto,                 // Auto-fit columns
    Responsive(Vec<u32>), // Responsive columns [xs, sm, md, lg, xl]
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AlignItems {
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum JustifyContent {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

impl Grid {
    pub fn new(props: GridProps) -> Self {
        let id = utils::generate_component_id("grid");
        Self { id, props }
    }
}

impl Component for Grid {
    fn id(&self) -> ComponentId { self.id.clone() }
    fn render(&self, _ctx: &RenderContext) -> RobinResult<RenderOutput> {
        Ok(RenderOutput::with_legacy_fields(
            "div".to_string(),
            "Grid Layout".to_string(),
            Style {
                display: Some(super::super::css_in_rust::Display::Grid),
                gap: Some(self.props.gap),
                ..Default::default()
            },
            HashMap::new(),
            Vec::new(),
        ))
    }

    fn type_name(&self) -> &'static str {
        "Grid"
    }

    fn init(&mut self, _ctx: &mut ComponentContext) -> RobinResult<()> {
        Ok(())
    }

    fn update(&mut self, _ctx: &mut ComponentContext, _delta_time: f32) -> RobinResult<()> {
        Ok(())
    }
}

impl UIComponent for Grid {
    fn component_type(&self) -> &'static str { "grid" }
    fn props(&self) -> &ComponentProps { &self.props.base }
    fn set_props(&mut self, props: ComponentProps) -> RobinResult<()> {
        self.props.base = props;
        Ok(())
    }
    fn handle_event(&mut self, _event: UIEvent) -> RobinResult<Vec<UIEvent>> { Ok(Vec::new()) }
    fn accessibility_info(&self) -> AccessibilityInfo { AccessibilityInfo::default() }
    fn apply_theme(&mut self, _theme: &UITheme) -> RobinResult<()> { Ok(()) }
}

/// Flex layout component
#[derive(Debug)]
pub struct Flex {
    id: ComponentId,
    props: FlexProps,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlexProps {
    pub base: ComponentProps,
    pub direction: FlexDirection,
    pub wrap: FlexWrap,
    pub gap: f32,
    pub align_items: AlignItems,
    pub justify_content: JustifyContent,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FlexDirection {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}

impl Flex {
    pub fn new(props: FlexProps) -> Self {
        let id = utils::generate_component_id("flex");
        Self { id, props }
    }
}

impl Component for Flex {
    fn id(&self) -> ComponentId { self.id.clone() }
    fn render(&self, _ctx: &RenderContext) -> RobinResult<RenderOutput> {
        Ok(RenderOutput {
            primitives: Vec::new(),
            bounds: Bounds::new(0.0, 0.0, 100.0, 100.0),
            interaction_bounds: None,
            element_type: Some("div".to_string()),
            content: Some("Flex Layout".to_string()),
            style: Some(Style {
                display: Some(super::super::css_in_rust::Display::Flex),
                gap: Some(self.props.gap),
                ..Default::default()
            }),
            attributes: Some(HashMap::new()),
            children: Some(Vec::new()),
        })
    }

    fn type_name(&self) -> &'static str {
        "Flex"
    }

    fn init(&mut self, _ctx: &mut ComponentContext) -> RobinResult<()> {
        Ok(())
    }

    fn update(&mut self, _ctx: &mut ComponentContext, _delta_time: f32) -> RobinResult<()> {
        Ok(())
    }
}

impl UIComponent for Flex {
    fn component_type(&self) -> &'static str { "flex" }
    fn props(&self) -> &ComponentProps { &self.props.base }
    fn set_props(&mut self, props: ComponentProps) -> RobinResult<()> {
        self.props.base = props;
        Ok(())
    }
    fn handle_event(&mut self, _event: UIEvent) -> RobinResult<Vec<UIEvent>> { Ok(Vec::new()) }
    fn accessibility_info(&self) -> AccessibilityInfo { AccessibilityInfo::default() }
    fn apply_theme(&mut self, _theme: &UITheme) -> RobinResult<()> { Ok(()) }
}

/// Container component for responsive layouts
#[derive(Debug)]
pub struct Container {
    id: ComponentId,
    props: ContainerProps,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerProps {
    pub base: ComponentProps,
    pub max_width: ContainerSize,
    pub padding: f32,
    pub centered: bool,
    pub fluid: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ContainerSize {
    Small,   // 640px
    Medium,  // 768px
    Large,   // 1024px
    XLarge,  // 1280px
    Full,    // 100%
}

impl Container {
    pub fn new(props: ContainerProps) -> Self {
        let id = utils::generate_component_id("container");
        Self { id, props }
    }
}

impl Component for Container {
    fn id(&self) -> ComponentId { self.id.clone() }
    fn render(&self, _ctx: &RenderContext) -> RobinResult<RenderOutput> {
        Ok(RenderOutput {
            primitives: Vec::new(),
            bounds: Bounds::new(0.0, 0.0, 100.0, 100.0),
            interaction_bounds: None,
            element_type: Some("div".to_string()),
            content: Some("Container".to_string()),
            style: Some(Style::default()),
            attributes: Some(HashMap::new()),
            children: Some(Vec::new()),
        })
    }

    fn type_name(&self) -> &'static str {
        "Container"
    }

    fn init(&mut self, _ctx: &mut ComponentContext) -> RobinResult<()> {
        Ok(())
    }

    fn update(&mut self, _ctx: &mut ComponentContext, _delta_time: f32) -> RobinResult<()> {
        Ok(())
    }
}

impl UIComponent for Container {
    fn component_type(&self) -> &'static str { "container" }
    fn props(&self) -> &ComponentProps { &self.props.base }
    fn set_props(&mut self, props: ComponentProps) -> RobinResult<()> {
        self.props.base = props;
        Ok(())
    }
    fn handle_event(&mut self, _event: UIEvent) -> RobinResult<Vec<UIEvent>> { Ok(Vec::new()) }
    fn accessibility_info(&self) -> AccessibilityInfo { AccessibilityInfo::default() }
    fn apply_theme(&mut self, _theme: &UITheme) -> RobinResult<()> { Ok(()) }
}