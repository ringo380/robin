// Modern Component Library for Robin Engine
// Production-ready, accessible UI components with beautiful designs

use crate::engine::{
    error::{RobinResult, RobinError},
    math::Vec2,
};
use super::modern_ui_framework::{
    Component as ModernComponent, VNode, Props, PropValue, RenderContext,
    Color, UIInputEvent, ComponentId, Theme, NodeId as FrameworkNodeId
};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

/// Modern Button Component - Primary UI action element
pub struct ModernButton {
    /// Button text
    text: String,
    /// Button variant
    variant: ButtonVariant,
    /// Button size
    size: ButtonSize,
    /// Disabled state
    disabled: bool,
    /// Loading state
    loading: bool,
    /// Click handler
    on_click: Option<Arc<dyn Fn() -> RobinResult<()> + Send + Sync>>,
    /// Icon (optional)
    icon: Option<String>,
    /// Accessibility label
    aria_label: Option<String>,
    /// Full width
    full_width: bool,
    /// Animation state
    animation_state: ButtonAnimationState,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Success,
    Warning,
    Danger,
    Ghost,
    Link,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonSize {
    Small,
    Medium,
    Large,
    ExtraLarge,
}

#[derive(Debug, Clone)]
struct ButtonAnimationState {
    hover_progress: f32,
    press_progress: f32,
    focus_progress: f32,
    ripple_effects: Vec<RippleEffect>,
}

#[derive(Debug, Clone)]
struct RippleEffect {
    position: Vec2,
    radius: f32,
    opacity: f32,
    start_time: Instant,
}

impl ModernButton {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            variant: ButtonVariant::Primary,
            size: ButtonSize::Medium,
            disabled: false,
            loading: false,
            on_click: None,
            icon: None,
            aria_label: None,
            full_width: false,
            animation_state: ButtonAnimationState {
                hover_progress: 0.0,
                press_progress: 0.0,
                focus_progress: 0.0,
                ripple_effects: Vec::new(),
            },
        }
    }

    /// Set button variant
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Set button size
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    /// Set disabled state
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set loading state
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    /// Set click handler
    pub fn on_click<F>(mut self, handler: F) -> Self
    where
        F: Fn() -> RobinResult<()> + Send + Sync + 'static,
    {
        self.on_click = Some(Arc::new(handler));
        self
    }

    /// Set icon
    pub fn icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());
        self
    }

    /// Set full width
    pub fn full_width(mut self, full_width: bool) -> Self {
        self.full_width = full_width;
        self
    }

    /// Get button colors for current variant
    fn get_colors(&self, theme: &Theme) -> ButtonColors {
        match self.variant {
            ButtonVariant::Primary => ButtonColors {
                background: Color::new(0.0, 0.48, 1.0, 1.0), // Blue
                hover: Color::new(0.0, 0.42, 0.9, 1.0),
                active: Color::new(0.0, 0.36, 0.8, 1.0),
                text: Color::new(1.0, 1.0, 1.0, 1.0),
            },
            ButtonVariant::Secondary => ButtonColors {
                background: Color::new(0.24, 0.24, 0.26, 1.0), // Dark gray
                hover: Color::new(0.28, 0.28, 0.30, 1.0),
                active: Color::new(0.32, 0.32, 0.34, 1.0),
                text: Color::new(1.0, 1.0, 1.0, 1.0),
            },
            ButtonVariant::Success => ButtonColors {
                background: Color::new(0.20, 0.78, 0.35, 1.0), // Green
                hover: Color::new(0.18, 0.70, 0.32, 1.0),
                active: Color::new(0.16, 0.62, 0.28, 1.0),
                text: Color::new(1.0, 1.0, 1.0, 1.0),
            },
            ButtonVariant::Warning => ButtonColors {
                background: Color::new(1.0, 0.58, 0.0, 1.0), // Orange
                hover: Color::new(0.9, 0.52, 0.0, 1.0),
                active: Color::new(0.8, 0.46, 0.0, 1.0),
                text: Color::new(1.0, 1.0, 1.0, 1.0),
            },
            ButtonVariant::Danger => ButtonColors {
                background: Color::new(1.0, 0.23, 0.19, 1.0), // Red
                hover: Color::new(0.9, 0.21, 0.17, 1.0),
                active: Color::new(0.8, 0.18, 0.15, 1.0),
                text: Color::new(1.0, 1.0, 1.0, 1.0),
            },
            ButtonVariant::Ghost => ButtonColors {
                background: Color::new(0.0, 0.0, 0.0, 0.0), // Transparent
                hover: Color::new(0.0, 0.48, 1.0, 0.1),
                active: Color::new(0.0, 0.48, 1.0, 0.2),
                text: Color::new(0.0, 0.48, 1.0, 1.0),
            },
            ButtonVariant::Link => ButtonColors {
                background: Color::new(0.0, 0.0, 0.0, 0.0),
                hover: Color::new(0.0, 0.0, 0.0, 0.0),
                active: Color::new(0.0, 0.0, 0.0, 0.0),
                text: Color::new(0.0, 0.48, 1.0, 1.0),
            },
        }
    }

    /// Get button dimensions for current size
    fn get_dimensions(&self) -> ButtonDimensions {
        match self.size {
            ButtonSize::Small => ButtonDimensions {
                height: 32.0,
                padding_x: 12.0,
                padding_y: 6.0,
                font_size: 14.0,
                border_radius: 6.0,
            },
            ButtonSize::Medium => ButtonDimensions {
                height: 40.0,
                padding_x: 16.0,
                padding_y: 8.0,
                font_size: 16.0,
                border_radius: 8.0,
            },
            ButtonSize::Large => ButtonDimensions {
                height: 48.0,
                padding_x: 20.0,
                padding_y: 12.0,
                font_size: 18.0,
                border_radius: 10.0,
            },
            ButtonSize::ExtraLarge => ButtonDimensions {
                height: 56.0,
                padding_x: 24.0,
                padding_y: 16.0,
                font_size: 20.0,
                border_radius: 12.0,
            },
        }
    }
}

impl ModernComponent for ModernButton {
    fn render(&self, ctx: &mut RenderContext) -> RobinResult<VNode> {
        let colors = self.get_colors(&Theme::Dark); // Temporary fix
        let dimensions = self.get_dimensions();

        // Create button node with computed styles
        let node = VNode {
            id: FrameworkNodeId::new(),
            node_type: "button".to_string(),
            props: self.create_props(colors, dimensions)?,
            children: self.create_children()?,
            state: None,
        };

        Ok(node)
    }
}

impl ModernButton {
    fn create_props(&self, colors: ButtonColors, dimensions: ButtonDimensions) -> RobinResult<Props> {
        let mut props = HashMap::new();

        // Styling
        props.insert("background-color".to_string(), PropValue::Color(colors.background));
        props.insert("color".to_string(), PropValue::Color(colors.text));
        props.insert("height".to_string(), PropValue::Number(dimensions.height as f64));
        props.insert("padding-x".to_string(), PropValue::Number(dimensions.padding_x as f64));
        props.insert("padding-y".to_string(), PropValue::Number(dimensions.padding_y as f64));
        props.insert("font-size".to_string(), PropValue::Number(dimensions.font_size as f64));
        props.insert("border-radius".to_string(), PropValue::Number(dimensions.border_radius as f64));

        // State
        props.insert("disabled".to_string(), PropValue::Bool(self.disabled));
        props.insert("loading".to_string(), PropValue::Bool(self.loading));
        props.insert("full-width".to_string(), PropValue::Bool(self.full_width));

        // Accessibility
        if let Some(ref label) = self.aria_label {
            props.insert("aria-label".to_string(), PropValue::String(label.clone()));
        }
        props.insert("role".to_string(), PropValue::String("button".to_string()));
        props.insert("tabindex".to_string(), PropValue::Number(if self.disabled { -1.0 } else { 0.0 }));

        // Animation
        props.insert("transition".to_string(), PropValue::String("all 0.2s cubic-bezier(0.4, 0, 0.2, 1)".to_string()));

        Ok(Props { values: props })
    }

    fn create_children(&self) -> RobinResult<Vec<VNode>> {
        let mut children = Vec::new();

        // Loading spinner
        if self.loading {
            children.push(VNode {
                id: FrameworkNodeId::new(),
                node_type: "spinner".to_string(),
                props: Props { values: HashMap::new() },
                children: Vec::new(),
                state: None,
            });
        }

        // Icon
        if let Some(ref icon) = self.icon {
            let mut icon_props = HashMap::new();
            icon_props.insert("name".to_string(), PropValue::String(icon.clone()));
            children.push(VNode {
                id: FrameworkNodeId::new(),
                node_type: "icon".to_string(),
                props: Props { values: icon_props },
                children: Vec::new(),
                state: None,
            });
        }

        // Text
        let mut text_props = HashMap::new();
        text_props.insert("content".to_string(), PropValue::String(self.text.clone()));
        children.push(VNode {
            id: FrameworkNodeId::new(),
            node_type: "text".to_string(),
            props: Props { values: text_props },
            children: Vec::new(),
            state: None,
        });

        Ok(children)
    }
}

/// Modern Card Component - Content container with elevation
pub struct ModernCard {
    /// Card title
    title: Option<String>,
    /// Card subtitle
    subtitle: Option<String>,
    /// Card content
    content: Vec<VNode>,
    /// Card actions
    actions: Vec<VNode>,
    /// Elevation level (0-24)
    elevation: u8,
    /// Variant
    variant: CardVariant,
    /// Clickable
    clickable: bool,
    /// Click handler
    on_click: Option<Arc<dyn Fn() -> RobinResult<()> + Send + Sync>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CardVariant {
    Elevated,
    Outlined,
    Filled,
}

impl ModernCard {
    pub fn new() -> Self {
        Self {
            title: None,
            subtitle: None,
            content: Vec::new(),
            actions: Vec::new(),
            elevation: 2,
            variant: CardVariant::Elevated,
            clickable: false,
            on_click: None,
        }
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn subtitle(mut self, subtitle: &str) -> Self {
        self.subtitle = Some(subtitle.to_string());
        self
    }

    pub fn elevation(mut self, elevation: u8) -> Self {
        self.elevation = elevation.min(24);
        self
    }

    pub fn variant(mut self, variant: CardVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn clickable(mut self, clickable: bool) -> Self {
        self.clickable = clickable;
        self
    }

    pub fn add_content(mut self, content: VNode) -> Self {
        self.content.push(content);
        self
    }

    pub fn add_action(mut self, action: VNode) -> Self {
        self.actions.push(action);
        self
    }
}

impl ModernComponent for ModernCard {
    fn render(&self, _ctx: &mut RenderContext) -> RobinResult<VNode> {
        let shadow = self.get_elevation_shadow();

        let mut props = HashMap::new();
        props.insert("box-shadow".to_string(), PropValue::String(shadow));
        props.insert("border-radius".to_string(), PropValue::Number(12.0));
        props.insert("padding".to_string(), PropValue::Number(16.0));
        props.insert("background-color".to_string(), PropValue::Color(Color::new(0.11, 0.11, 0.12, 1.0)));

        if self.clickable {
            props.insert("cursor".to_string(), PropValue::String("pointer".to_string()));
            props.insert("transition".to_string(), PropValue::String("all 0.2s ease".to_string()));
        }

        let children = self.build_card_structure()?;

        Ok(VNode {
            id: FrameworkNodeId::new(),
            node_type: "card".to_string(),
            props: Props { values: props },
            children,
            state: None,
        })
    }
}

impl ModernCard {
    fn get_elevation_shadow(&self) -> String {
        match self.elevation {
            0 => "none".to_string(),
            1 => "0 1px 3px rgba(0,0,0,0.12), 0 1px 2px rgba(0,0,0,0.24)".to_string(),
            2 => "0 3px 6px rgba(0,0,0,0.16), 0 3px 6px rgba(0,0,0,0.23)".to_string(),
            3 => "0 10px 20px rgba(0,0,0,0.19), 0 6px 6px rgba(0,0,0,0.23)".to_string(),
            4 => "0 14px 28px rgba(0,0,0,0.25), 0 10px 10px rgba(0,0,0,0.22)".to_string(),
            5 => "0 19px 38px rgba(0,0,0,0.30), 0 15px 12px rgba(0,0,0,0.22)".to_string(),
            _ => format!("0 {}px {}px rgba(0,0,0,0.3)", self.elevation * 2, self.elevation * 4),
        }
    }

    fn build_card_structure(&self) -> RobinResult<Vec<VNode>> {
        let mut children = Vec::new();

        // Header section (title + subtitle)
        if self.title.is_some() || self.subtitle.is_some() {
            let mut header_children = Vec::new();

            if let Some(ref title) = self.title {
                let mut title_props = HashMap::new();
                title_props.insert("content".to_string(), PropValue::String(title.clone()));
                title_props.insert("font-size".to_string(), PropValue::Number(20.0));
                title_props.insert("font-weight".to_string(), PropValue::String("600".to_string()));
                title_props.insert("color".to_string(), PropValue::Color(Color::new(1.0, 1.0, 1.0, 1.0)));

                header_children.push(VNode {
                    id: FrameworkNodeId::new(),
                    node_type: "text".to_string(),
                    props: Props { values: title_props },
                    children: Vec::new(),
                    state: None,
                });
            }

            if let Some(ref subtitle) = self.subtitle {
                let mut subtitle_props = HashMap::new();
                subtitle_props.insert("content".to_string(), PropValue::String(subtitle.clone()));
                subtitle_props.insert("font-size".to_string(), PropValue::Number(14.0));
                subtitle_props.insert("color".to_string(), PropValue::Color(Color::new(0.7, 0.7, 0.7, 1.0)));

                header_children.push(VNode {
                    id: FrameworkNodeId::new(),
                    node_type: "text".to_string(),
                    props: Props { values: subtitle_props },
                    children: Vec::new(),
                    state: None,
                });
            }

            children.push(VNode {
                id: FrameworkNodeId::new(),
                node_type: "header".to_string(),
                props: Props { values: HashMap::new() },
                children: header_children,
                state: None,
            });
        }

        // Content section
        if !self.content.is_empty() {
            children.push(VNode {
                id: FrameworkNodeId::new(),
                node_type: "content".to_string(),
                props: Props { values: HashMap::new() },
                children: self.content.clone(),
                state: None,
            });
        }

        // Actions section
        if !self.actions.is_empty() {
            children.push(VNode {
                id: FrameworkNodeId::new(),
                node_type: "actions".to_string(),
                props: Props { values: HashMap::new() },
                children: self.actions.clone(),
                state: None,
            });
        }

        Ok(children)
    }
}

/// Modern Input Component - Text input with validation and styling
pub struct ModernInput {
    /// Input value
    value: String,
    /// Placeholder text
    placeholder: Option<String>,
    /// Input type
    input_type: InputType,
    /// Label
    label: Option<String>,
    /// Helper text
    helper_text: Option<String>,
    /// Error message
    error: Option<String>,
    /// Disabled state
    disabled: bool,
    /// Required field
    required: bool,
    /// Change handler
    on_change: Option<Arc<dyn Fn(String) -> RobinResult<()> + Send + Sync>>,
    /// Validation function
    validator: Option<Arc<dyn Fn(&str) -> RobinResult<()> + Send + Sync>>,
    /// Input size
    size: InputSize,
    /// Full width
    full_width: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputType {
    Text,
    Password,
    Email,
    Number,
    Search,
    Url,
    Tel,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputSize {
    Small,
    Medium,
    Large,
}

impl ModernInput {
    pub fn new() -> Self {
        Self {
            value: String::new(),
            placeholder: None,
            input_type: InputType::Text,
            label: None,
            helper_text: None,
            error: None,
            disabled: false,
            required: false,
            on_change: None,
            validator: None,
            size: InputSize::Medium,
            full_width: false,
        }
    }

    pub fn value(mut self, value: &str) -> Self {
        self.value = value.to_string();
        self
    }

    pub fn placeholder(mut self, placeholder: &str) -> Self {
        self.placeholder = Some(placeholder.to_string());
        self
    }

    pub fn input_type(mut self, input_type: InputType) -> Self {
        self.input_type = input_type;
        self
    }

    pub fn label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    pub fn helper_text(mut self, text: &str) -> Self {
        self.helper_text = Some(text.to_string());
        self
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn full_width(mut self, full_width: bool) -> Self {
        self.full_width = full_width;
        self
    }

    pub fn on_change<F>(mut self, handler: F) -> Self
    where
        F: Fn(String) -> RobinResult<()> + Send + Sync + 'static,
    {
        self.on_change = Some(Arc::new(handler));
        self
    }
}

impl ModernComponent for ModernInput {
    fn render(&self, _ctx: &mut RenderContext) -> RobinResult<VNode> {
        let dimensions = self.get_input_dimensions();
        let has_error = self.error.is_some();

        let mut container_children = Vec::new();

        // Label
        if let Some(ref label) = self.label {
            let mut label_props = HashMap::new();
            label_props.insert("content".to_string(), PropValue::String(
                if self.required {
                    format!("{} *", label)
                } else {
                    label.clone()
                }
            ));
            label_props.insert("font-size".to_string(), PropValue::Number(14.0));
            label_props.insert("color".to_string(), PropValue::Color(
                if has_error {
                    Color::new(1.0, 0.23, 0.19, 1.0) // Error red
                } else {
                    Color::new(0.9, 0.9, 0.9, 1.0) // Normal gray
                }
            ));
            label_props.insert("margin-bottom".to_string(), PropValue::Number(4.0));

            container_children.push(VNode {
                id: FrameworkNodeId::new(),
                node_type: "label".to_string(),
                props: Props { values: label_props },
                children: Vec::new(),
                state: None,
            });
        }

        // Input field
        let mut input_props = HashMap::new();
        input_props.insert("value".to_string(), PropValue::String(self.value.clone()));
        if let Some(ref placeholder) = self.placeholder {
            input_props.insert("placeholder".to_string(), PropValue::String(placeholder.clone()));
        }
        input_props.insert("type".to_string(), PropValue::String(self.input_type_to_string()));
        input_props.insert("disabled".to_string(), PropValue::Bool(self.disabled));
        input_props.insert("required".to_string(), PropValue::Bool(self.required));

        // Styling
        input_props.insert("height".to_string(), PropValue::Number(dimensions.height as f64));
        input_props.insert("padding".to_string(), PropValue::Number(dimensions.padding as f64));
        input_props.insert("font-size".to_string(), PropValue::Number(dimensions.font_size as f64));
        input_props.insert("border-radius".to_string(), PropValue::Number(8.0));
        input_props.insert("background-color".to_string(), PropValue::Color(Color::new(0.16, 0.16, 0.18, 1.0)));
        input_props.insert("border".to_string(), PropValue::String(
            if has_error {
                "2px solid #FF3B30".to_string()
            } else {
                "2px solid transparent".to_string()
            }
        ));
        input_props.insert("color".to_string(), PropValue::Color(Color::new(1.0, 1.0, 1.0, 1.0)));
        input_props.insert("transition".to_string(), PropValue::String("all 0.2s ease".to_string()));

        if self.full_width {
            input_props.insert("width".to_string(), PropValue::String("100%".to_string()));
        }

        container_children.push(VNode {
            id: FrameworkNodeId::new(),
            node_type: "input".to_string(),
            props: Props { values: input_props },
            children: Vec::new(),
            state: None,
        });

        // Helper text or error message
        if let Some(ref error) = self.error {
            let mut error_props = HashMap::new();
            error_props.insert("content".to_string(), PropValue::String(error.clone()));
            error_props.insert("font-size".to_string(), PropValue::Number(12.0));
            error_props.insert("color".to_string(), PropValue::Color(Color::new(1.0, 0.23, 0.19, 1.0)));
            error_props.insert("margin-top".to_string(), PropValue::Number(4.0));

            container_children.push(VNode {
                id: FrameworkNodeId::new(),
                node_type: "text".to_string(),
                props: Props { values: error_props },
                children: Vec::new(),
                state: None,
            });
        } else if let Some(ref helper) = self.helper_text {
            let mut helper_props = HashMap::new();
            helper_props.insert("content".to_string(), PropValue::String(helper.clone()));
            helper_props.insert("font-size".to_string(), PropValue::Number(12.0));
            helper_props.insert("color".to_string(), PropValue::Color(Color::new(0.7, 0.7, 0.7, 1.0)));
            helper_props.insert("margin-top".to_string(), PropValue::Number(4.0));

            container_children.push(VNode {
                id: FrameworkNodeId::new(),
                node_type: "text".to_string(),
                props: Props { values: helper_props },
                children: Vec::new(),
                state: None,
            });
        }

        Ok(VNode {
            id: FrameworkNodeId::new(),
            node_type: "input-container".to_string(),
            props: Props { values: HashMap::new() },
            children: container_children,
            state: None,
        })
    }
}

impl ModernInput {
    fn get_input_dimensions(&self) -> InputDimensions {
        match self.size {
            InputSize::Small => InputDimensions {
                height: 32.0,
                padding: 8.0,
                font_size: 14.0,
            },
            InputSize::Medium => InputDimensions {
                height: 40.0,
                padding: 12.0,
                font_size: 16.0,
            },
            InputSize::Large => InputDimensions {
                height: 48.0,
                padding: 16.0,
                font_size: 18.0,
            },
        }
    }

    fn input_type_to_string(&self) -> String {
        match self.input_type {
            InputType::Text => "text".to_string(),
            InputType::Password => "password".to_string(),
            InputType::Email => "email".to_string(),
            InputType::Number => "number".to_string(),
            InputType::Search => "search".to_string(),
            InputType::Url => "url".to_string(),
            InputType::Tel => "tel".to_string(),
        }
    }
}

/// Modern Modal Component - Overlay dialog
pub struct ModernModal {
    /// Modal title
    title: Option<String>,
    /// Modal content
    content: Vec<VNode>,
    /// Modal size
    size: ModalSize,
    /// Closable by clicking backdrop
    closable: bool,
    /// Close handler
    on_close: Option<Arc<dyn Fn() -> RobinResult<()> + Send + Sync>>,
    /// Show close button
    show_close_button: bool,
    /// Centered
    centered: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModalSize {
    Small,
    Medium,
    Large,
    ExtraLarge,
    FullScreen,
}

impl ModernModal {
    pub fn new() -> Self {
        Self {
            title: None,
            content: Vec::new(),
            size: ModalSize::Medium,
            closable: true,
            on_close: None,
            show_close_button: true,
            centered: true,
        }
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn size(mut self, size: ModalSize) -> Self {
        self.size = size;
        self
    }

    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    pub fn add_content(mut self, content: VNode) -> Self {
        self.content.push(content);
        self
    }

    fn get_modal_dimensions(&self) -> ModalDimensions {
        match self.size {
            ModalSize::Small => ModalDimensions { width: 400.0, height: 300.0 },
            ModalSize::Medium => ModalDimensions { width: 600.0, height: 400.0 },
            ModalSize::Large => ModalDimensions { width: 800.0, height: 600.0 },
            ModalSize::ExtraLarge => ModalDimensions { width: 1200.0, height: 800.0 },
            ModalSize::FullScreen => ModalDimensions { width: 0.0, height: 0.0 }, // Special case
        }
    }
}

impl ModernComponent for ModernModal {
    fn render(&self, _ctx: &mut RenderContext) -> RobinResult<VNode> {
        let dimensions = self.get_modal_dimensions();

        // Backdrop
        let mut backdrop_props = HashMap::new();
        backdrop_props.insert("position".to_string(), PropValue::String("fixed".to_string()));
        backdrop_props.insert("top".to_string(), PropValue::Number(0.0));
        backdrop_props.insert("left".to_string(), PropValue::Number(0.0));
        backdrop_props.insert("width".to_string(), PropValue::String("100%".to_string()));
        backdrop_props.insert("height".to_string(), PropValue::String("100%".to_string()));
        backdrop_props.insert("background-color".to_string(), PropValue::Color(Color::new(0.0, 0.0, 0.0, 0.5)));
        backdrop_props.insert("z-index".to_string(), PropValue::Number(1000.0));

        // Modal container
        let mut modal_props = HashMap::new();
        modal_props.insert("position".to_string(), PropValue::String("relative".to_string()));
        modal_props.insert("background-color".to_string(), PropValue::Color(Color::new(0.11, 0.11, 0.12, 1.0)));
        modal_props.insert("border-radius".to_string(), PropValue::Number(12.0));
        modal_props.insert("padding".to_string(), PropValue::Number(24.0));
        modal_props.insert("box-shadow".to_string(), PropValue::String("0 20px 40px rgba(0,0,0,0.4)".to_string()));

        if self.size != ModalSize::FullScreen {
            modal_props.insert("width".to_string(), PropValue::Number(dimensions.width as f64));
            modal_props.insert("max-height".to_string(), PropValue::Number(dimensions.height as f64));
        }

        let modal_content = self.build_modal_content()?;

        let modal_node = VNode {
            id: FrameworkNodeId::new(),
            node_type: "modal".to_string(),
            props: Props { values: modal_props },
            children: modal_content,
            state: None,
        };

        Ok(VNode {
            id: FrameworkNodeId::new(),
            node_type: "modal-backdrop".to_string(),
            props: Props { values: backdrop_props },
            children: vec![modal_node],
            state: None,
        })
    }
}

impl ModernModal {
    fn build_modal_content(&self) -> RobinResult<Vec<VNode>> {
        let mut children = Vec::new();

        // Header with title and close button
        if self.title.is_some() || self.show_close_button {
            let mut header_children = Vec::new();

            if let Some(ref title) = self.title {
                let mut title_props = HashMap::new();
                title_props.insert("content".to_string(), PropValue::String(title.clone()));
                title_props.insert("font-size".to_string(), PropValue::Number(24.0));
                title_props.insert("font-weight".to_string(), PropValue::String("600".to_string()));
                title_props.insert("color".to_string(), PropValue::Color(Color::new(1.0, 1.0, 1.0, 1.0)));

                header_children.push(VNode {
                    id: FrameworkNodeId::new(),
                    node_type: "text".to_string(),
                    props: Props { values: title_props },
                    children: Vec::new(),
                    state: None,
                });
            }

            if self.show_close_button {
                let mut close_props = HashMap::new();
                close_props.insert("content".to_string(), PropValue::String("×".to_string()));
                close_props.insert("font-size".to_string(), PropValue::Number(32.0));
                close_props.insert("color".to_string(), PropValue::Color(Color::new(0.7, 0.7, 0.7, 1.0)));
                close_props.insert("cursor".to_string(), PropValue::String("pointer".to_string()));
                close_props.insert("position".to_string(), PropValue::String("absolute".to_string()));
                close_props.insert("top".to_string(), PropValue::Number(16.0));
                close_props.insert("right".to_string(), PropValue::Number(16.0));

                header_children.push(VNode {
                    id: FrameworkNodeId::new(),
                    node_type: "button".to_string(),
                    props: Props { values: close_props },
                    children: Vec::new(),
                    state: None,
                });
            }

            children.push(VNode {
                id: FrameworkNodeId::new(),
                node_type: "modal-header".to_string(),
                props: Props { values: HashMap::new() },
                children: header_children,
                state: None,
            });
        }

        // Content
        children.push(VNode {
            id: FrameworkNodeId::new(),
            node_type: "modal-content".to_string(),
            props: Props { values: HashMap::new() },
            children: self.content.clone(),
            state: None,
        });

        Ok(children)
    }
}

// Supporting structures
#[derive(Debug)]
struct ButtonColors {
    background: Color,
    hover: Color,
    active: Color,
    text: Color,
}

#[derive(Debug)]
struct ButtonDimensions {
    height: f32,
    padding_x: f32,
    padding_y: f32,
    font_size: f32,
    border_radius: f32,
}

#[derive(Debug)]
struct InputDimensions {
    height: f32,
    padding: f32,
    font_size: f32,
}

#[derive(Debug)]
struct ModalDimensions {
    width: f32,
    height: f32,
}

// Helper to create unique node IDs
pub struct NodeId(u64);
impl NodeId {
    fn new() -> Self {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        Self(COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst))
    }
}

impl From<NodeId> for super::modern_ui_framework::NodeId {
    fn from(id: NodeId) -> Self {
        super::modern_ui_framework::NodeId(id.0)
    }
}

/// Component library exports
pub struct ComponentLibrary;

impl ComponentLibrary {
    /// Create a new modern button
    pub fn button(text: &str) -> ModernButton {
        ModernButton::new(text)
    }

    /// Create a new modern card
    pub fn card() -> ModernCard {
        ModernCard::new()
    }

    /// Create a new modern input
    pub fn input() -> ModernInput {
        ModernInput::new()
    }

    /// Create a new modern modal
    pub fn modal() -> ModernModal {
        ModernModal::new()
    }
}

/// Toast notification system
pub struct ToastManager {
    toasts: Vec<Toast>,
    max_toasts: usize,
}

pub struct Toast {
    id: u64,
    message: String,
    toast_type: ToastType,
    duration: Duration,
    created_at: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToastType {
    Success,
    Warning,
    Error,
    Info,
}

impl ToastManager {
    pub fn new() -> Self {
        Self {
            toasts: Vec::new(),
            max_toasts: 5,
        }
    }

    pub fn show(&mut self, message: &str, toast_type: ToastType, duration: Duration) {
        let toast = Toast {
            id: Self::generate_id(),
            message: message.to_string(),
            toast_type,
            duration,
            created_at: Instant::now(),
        };

        self.toasts.push(toast);

        // Remove excess toasts
        while self.toasts.len() > self.max_toasts {
            self.toasts.remove(0);
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        self.toasts.retain(|toast| {
            now.duration_since(toast.created_at) < toast.duration
        });
    }

    fn generate_id() -> u64 {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }
}

/// Dropdown component
pub struct ModernDropdown {
    items: Vec<DropdownItem>,
    selected_index: Option<usize>,
    placeholder: Option<String>,
    disabled: bool,
    on_select: Option<Arc<dyn Fn(usize) -> RobinResult<()> + Send + Sync>>,
}

pub struct DropdownItem {
    pub label: String,
    pub value: String,
    pub disabled: bool,
}

impl ModernDropdown {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected_index: None,
            placeholder: None,
            disabled: false,
            on_select: None,
        }
    }

    pub fn add_item(mut self, label: &str, value: &str) -> Self {
        self.items.push(DropdownItem {
            label: label.to_string(),
            value: value.to_string(),
            disabled: false,
        });
        self
    }

    pub fn placeholder(mut self, placeholder: &str) -> Self {
        self.placeholder = Some(placeholder.to_string());
        self
    }
}

impl ModernComponent for ModernDropdown {
    fn render(&self, _ctx: &mut RenderContext) -> RobinResult<VNode> {
        // Implementation would create dropdown structure
        Ok(VNode {
            id: FrameworkNodeId::new().into(),
            node_type: "dropdown".to_string(),
            props: Props { values: HashMap::new() },
            children: Vec::new(),
            state: None,
        })
    }
}