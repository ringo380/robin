// Robin Game Engine - Feedback Components
// Notifications, toasts, alerts, and progress indicators

use super::*;
use crate::engine::error::RobinResult;
use crate::engine::generation::templates::UITheme;
use crate::engine::ui::modern_architecture::ComponentContext;
use serde::{Serialize, Deserialize};

/// Toast notification component
#[derive(Debug)]
pub struct Toast {
    id: ComponentId,
    props: ToastProps,
    state: ToastState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToastProps {
    pub base: ComponentProps,
    pub title: Option<String>,
    pub message: String,
    pub toast_type: ToastType,
    pub duration: Option<f32>, // Auto-hide duration in seconds
    pub closable: bool,
    pub action: Option<ToastAction>,
    pub position: ToastPosition,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ToastType {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ToastPosition {
    TopLeft,
    TopCenter,
    TopRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToastAction {
    pub label: String,
    pub action: String,
}

#[derive(Debug, Clone)]
struct ToastState {
    time_remaining: Option<f32>,
    visible: bool,
}

impl Toast {
    pub fn new(props: ToastProps) -> Self {
        let id = utils::generate_component_id("toast");
        let state = ToastState {
            time_remaining: props.duration,
            visible: true,
        };
        Self { id, props, state }
    }
}

impl Component for Toast {
    fn id(&self) -> ComponentId { self.id.clone() }
    fn render(&self, _ctx: &RenderContext) -> RobinResult<RenderOutput> {
        Ok(RenderOutput::with_legacy_fields(
            "div".to_string(),
            self.props.message.clone(),
            Style::default(),
            HashMap::new(),
            Vec::new(),
        ))
    }

    fn type_name(&self) -> &'static str {
        "Toast"
    }

    fn init(&mut self, _ctx: &mut ComponentContext) -> RobinResult<()> {
        Ok(())
    }

    fn update(&mut self, _ctx: &mut ComponentContext, _delta_time: f32) -> RobinResult<()> {
        Ok(())
    }
}

impl UIComponent for Toast {
    fn component_type(&self) -> &'static str { "toast" }
    fn props(&self) -> &ComponentProps { &self.props.base }
    fn set_props(&mut self, props: ComponentProps) -> RobinResult<()> {
        self.props.base = props;
        Ok(())
    }
    fn handle_event(&mut self, _event: UIEvent) -> RobinResult<Vec<UIEvent>> { Ok(Vec::new()) }
    fn accessibility_info(&self) -> AccessibilityInfo { AccessibilityInfo::default() }
    fn apply_theme(&mut self, _theme: &UITheme) -> RobinResult<()> { Ok(()) }
}

/// Progress bar component
#[derive(Debug)]
pub struct ProgressBar {
    id: ComponentId,
    props: ProgressProps,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressProps {
    pub base: ComponentProps,
    pub value: f32, // 0.0 to 1.0
    pub label: Option<String>,
    pub show_percentage: bool,
    pub size: Size,
    pub variant: ProgressVariant,
    pub animated: bool,
    pub indeterminate: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ProgressVariant {
    Linear,
    Circular,
}

impl ProgressBar {
    pub fn new(props: ProgressProps) -> Self {
        let id = utils::generate_component_id("progress");
        Self { id, props }
    }
}

impl Component for ProgressBar {
    fn id(&self) -> ComponentId { self.id.clone() }
    fn render(&self, _ctx: &RenderContext) -> RobinResult<RenderOutput> {
        Ok(RenderOutput::with_legacy_fields(
            "div".to_string(),
            format!("Progress: {:.0}%", self.props.value * 100.0),
            Style::default(),
            HashMap::new(),
            Vec::new(),
        ))
    }

    fn type_name(&self) -> &'static str {
        "ProgressBar"
    }

    fn init(&mut self, _ctx: &mut ComponentContext) -> RobinResult<()> {
        Ok(())
    }

    fn update(&mut self, _ctx: &mut ComponentContext, _delta_time: f32) -> RobinResult<()> {
        Ok(())
    }
}

impl UIComponent for ProgressBar {
    fn component_type(&self) -> &'static str { "progress" }
    fn props(&self) -> &ComponentProps { &self.props.base }
    fn set_props(&mut self, props: ComponentProps) -> RobinResult<()> {
        self.props.base = props;
        Ok(())
    }
    fn handle_event(&mut self, _event: UIEvent) -> RobinResult<Vec<UIEvent>> { Ok(Vec::new()) }
    fn accessibility_info(&self) -> AccessibilityInfo { AccessibilityInfo::default() }
    fn apply_theme(&mut self, _theme: &UITheme) -> RobinResult<()> { Ok(()) }
}