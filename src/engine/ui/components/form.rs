// Robin Game Engine - Form Components
// Comprehensive form input components with validation and accessibility

use super::*;
use crate::engine::error::RobinResult;
use crate::engine::ui::modern_architecture::{ComponentContext, RenderPrimitive, Bounds};
use crate::engine::ui::css_in_rust::{
    Display, Transition, TimingFunction,
    TextAlign, TextTransform, FontWeight, Dimension,
    Cursor, BoxSpacing, BorderStyle, BorderRadius
};
use crate::engine::ui::styling::{
    Resize, AlignItems, BoxSizing, Color as StylingColor
};
use crate::engine::generation::templates::UITheme;
use crate::engine::math::Vec2;
use serde::{Serialize, Deserialize};
use std::fmt::Debug;
use std::collections::HashMap;

/// Input field component
#[derive(Debug)]
pub struct Input {
    id: ComponentId,
    props: InputProps,
    state: InputState,
    style: Style,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputProps {
    pub base: ComponentProps,
    pub input_type: InputType,
    pub value: String,
    pub placeholder: Option<String>,
    pub label: Option<String>,
    pub help_text: Option<String>,
    pub error_message: Option<String>,
    pub required: bool,
    pub readonly: bool,
    pub size: Size,
    pub variant: InputVariant,
    pub icon: Option<String>,
    pub icon_position: IconPosition,
    pub max_length: Option<usize>,
    pub min_length: Option<usize>,
    pub pattern: Option<String>,
    pub validation: Vec<ValidationRule>,
    pub on_change: Option<String>,
    pub on_focus: Option<String>,
    pub on_blur: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum InputType {
    Text,
    Password,
    Email,
    Number,
    Tel,
    Url,
    Search,
    TextArea,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum InputVariant {
    Outlined,
    Filled,
    Underlined,
    Borderless,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum IconPosition {
    Left,
    Right,
    Only,
}

impl Default for IconPosition {
    fn default() -> Self {
        IconPosition::Left
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationRule {
    Required,
    MinLength(usize),
    MaxLength(usize),
    Pattern(String),
    Email,
    Custom(String), // Custom validation function name
}

#[derive(Debug, Clone)]
struct InputState {
    focused: bool,
    invalid: bool,
    validation_errors: Vec<String>,
    cursor_position: usize,
}

impl Default for InputProps {
    fn default() -> Self {
        Self {
            base: ComponentProps::default(),
            input_type: InputType::Text,
            value: String::new(),
            placeholder: None,
            label: None,
            help_text: None,
            error_message: None,
            required: false,
            readonly: false,
            size: Size::Medium,
            variant: InputVariant::Outlined,
            icon: None,
            icon_position: IconPosition::Left,
            max_length: None,
            min_length: None,
            pattern: None,
            validation: Vec::new(),
            on_change: None,
            on_focus: None,
            on_blur: None,
        }
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            focused: false,
            invalid: false,
            validation_errors: Vec::new(),
            cursor_position: 0,
        }
    }
}

impl Input {
    pub fn new(props: InputProps) -> Self {
        let id = utils::generate_component_id("input");
        let mut input = Self {
            id,
            props,
            state: InputState::default(),
            style: Style::default(),
        };
        input.update_style();
        input
    }

    pub fn set_value<S: Into<String>>(&mut self, value: S) -> RobinResult<()> {
        self.props.value = value.into();
        self.validate()?;
        self.update_style();
        Ok(())
    }

    pub fn get_value(&self) -> &str {
        &self.props.value
    }

    pub fn is_valid(&self) -> bool {
        !self.state.invalid
    }

    pub fn get_validation_errors(&self) -> &[String] {
        &self.state.validation_errors
    }

    fn validate(&mut self) -> RobinResult<()> {
        self.state.validation_errors.clear();
        self.state.invalid = false;

        for rule in &self.props.validation {
            match rule {
                ValidationRule::Required => {
                    if self.props.value.is_empty() {
                        self.state.validation_errors.push("This field is required".to_string());
                        self.state.invalid = true;
                    }
                },
                ValidationRule::MinLength(min) => {
                    if self.props.value.len() < *min {
                        self.state.validation_errors.push(format!("Minimum {} characters required", min));
                        self.state.invalid = true;
                    }
                },
                ValidationRule::MaxLength(max) => {
                    if self.props.value.len() > *max {
                        self.state.validation_errors.push(format!("Maximum {} characters allowed", max));
                        self.state.invalid = true;
                    }
                },
                ValidationRule::Email => {
                    if !self.props.value.is_empty() && !self.is_valid_email(&self.props.value) {
                        self.state.validation_errors.push("Please enter a valid email address".to_string());
                        self.state.invalid = true;
                    }
                },
                ValidationRule::Pattern(pattern) => {
                    // Simple pattern matching (would use regex in real implementation)
                    if !self.props.value.is_empty() && !self.matches_pattern(&self.props.value, pattern) {
                        self.state.validation_errors.push("Invalid format".to_string());
                        self.state.invalid = true;
                    }
                },
                ValidationRule::Custom(_) => {
                    // Custom validation would be handled by external validator
                }
            }
        }

        Ok(())
    }

    fn is_valid_email(&self, email: &str) -> bool {
        // Simple email validation
        email.contains('@') && email.contains('.')
    }

    fn matches_pattern(&self, value: &str, _pattern: &str) -> bool {
        // Simple pattern matching - would use regex in real implementation
        !value.is_empty()
    }

    fn update_style(&mut self) {
        self.style = self.generate_base_style();
        self.apply_variant_style();
        self.apply_size_style();
        self.apply_state_style();
    }

    fn generate_base_style(&self) -> Style {
        let mut style = Style::default();

        style.display = Some(Display::Flex);
        style.align_items = Some(AlignItems::Center);
        style.box_sizing = Some(BoxSizing::BorderBox);
        style.transition = Some(vec![Transition {
            property: "all".to_string(),
            duration: 0.2,
            timing: TimingFunction::EaseInOut,
            delay: 0.0,
        }]);

        style.font_family = Some("system-ui, -apple-system, sans-serif".to_string());
        style.font_size = Some(16.0);
        style.line_height = Some(1.5);
        style.color = Some(StylingColor::from_hex(0x333333));
        style.text_align = Some(TextAlign::Left);
        style.text_transform = Some(TextTransform::None);
        style.font_weight = Some(FontWeight::Normal);
        style.letter_spacing = Some(0.0);

        if self.props.base.disabled {
            style.opacity = Some(0.6);
            style.cursor = Some(Cursor::NotAllowed);
        } else {
            style.cursor = Some(Cursor::Text);
        }

        style
    }

    fn apply_variant_style(&mut self) {
        match self.props.variant {
            InputVariant::Outlined => {
                self.style.border_width = Some(1.0);
                self.style.border_style = Some(BorderStyle::Solid);
                self.style.border_color = Some(StylingColor::from_hex(0xd1d5db));
                self.style.border_radius = Some(BorderRadius::all(6.0));
                self.style.background_color = Some(StylingColor::from_hex(0xffffff));
            },
            InputVariant::Filled => {
                self.style.border = None;
                self.style.border_radius = Some(BorderRadius::all(6.0));
                self.style.background_color = Some(StylingColor::from_hex(0xf9fafb));
                self.style.border_width = Some(0.0);
                self.style.border_style = Some(BorderStyle::None);
                // Note: CSS border-bottom would need separate implementation
            },
            InputVariant::Underlined => {
                self.style.border = None;
                self.style.border_radius = Some(BorderRadius::all(0.0));
                self.style.background_color = None;
                self.style.border_width = Some(0.0);
                self.style.border_style = Some(BorderStyle::None);
                // Note: CSS border-bottom would need separate implementation
            },
            InputVariant::Borderless => {
                self.style.border = None;
                self.style.border_radius = Some(BorderRadius::all(0.0));
                self.style.background_color = None;
            },
        }
    }

    fn apply_size_style(&mut self) {
        let (height, padding, font_size) = match self.props.size {
            Size::XSmall => (28.0, 6.0, 12.0),
            Size::Small => (32.0, 8.0, 14.0),
            Size::Medium => (40.0, 12.0, 16.0),
            Size::Large => (48.0, 16.0, 18.0),
            Size::XLarge => (56.0, 20.0, 20.0),
        };

        self.style.height = Some(Dimension::Px(height));
        self.style.padding = Some(BoxSpacing::all(padding));

        self.style.font_size = Some(font_size);

        // Adjust for textarea
        if matches!(self.props.input_type, InputType::TextArea) {
            self.style.height = Some(Dimension::Px(height * 2.5));
            self.style.resize = Some(Resize::Vertical);
        }
    }

    fn apply_state_style(&mut self) {
        if self.state.focused {
            match self.props.variant {
                InputVariant::Outlined => {
                    self.style.border_width = Some(2.0);
                    self.style.border_style = Some(BorderStyle::Solid);
                    self.style.border_color = Some(StylingColor::from_hex(0x007bff));
                    self.style.outline = None;
                },
                InputVariant::Filled => {
                    // Focus state - would need separate border-bottom implementation
                    self.style.border_color = Some(StylingColor::from_hex(0x007bff));
                },
                InputVariant::Underlined => {
                    // Focus state - would need separate border-bottom implementation
                    self.style.border_color = Some(StylingColor::from_hex(0x007bff));
                },
                InputVariant::Borderless => {
                    // Focus outline - would need separate outline implementation
                    self.style.border_color = Some(StylingColor::from_hex(0x007bff));
                },
            }
        }

        if self.state.invalid {
            let error_color = StylingColor::from_hex(0xdc3545);
            match self.props.variant {
                InputVariant::Outlined => {
                    self.style.border_width = Some(1.0);
                    self.style.border_style = Some(BorderStyle::Solid);
                    self.style.border_color = Some(error_color);
                },
                InputVariant::Filled | InputVariant::Underlined => {
                    // Error state - would need separate border-bottom implementation
                    self.style.border_color = Some(error_color);
                },
                InputVariant::Borderless => {
                    // Error outline - would need separate outline implementation
                    self.style.border_color = Some(error_color);
                },
            }
        }
    }
}

impl Component for Input {
    fn id(&self) -> ComponentId {
        self.id.clone()
    }

    fn type_name(&self) -> &'static str {
        "input"
    }

    fn init(&mut self, _ctx: &mut ComponentContext) -> RobinResult<()> {
        Ok(())
    }

    fn update(&mut self, _ctx: &mut ComponentContext, _delta_time: f32) -> RobinResult<()> {
        Ok(())
    }

    fn render(&self, _ctx: &RenderContext) -> RobinResult<RenderOutput> {
        let bounds = Bounds {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 40.0,
        };

        let mut primitives = vec![
            RenderPrimitive::Rectangle {
                bounds,
                fill: Some(crate::engine::ui::modern_architecture::Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }),
                stroke: Some((crate::engine::ui::modern_architecture::Color { r: 0.8, g: 0.8, b: 0.8, a: 1.0 }, 1.0)),
                border_radius: 6.0,
            }
        ];

        if !self.props.value.is_empty() || self.props.placeholder.is_some() {
            let text_content = if self.props.value.is_empty() {
                self.props.placeholder.as_ref().unwrap_or(&String::new()).clone()
            } else {
                self.props.value.clone()
            };

            primitives.push(RenderPrimitive::Text {
                content: text_content,
                position: Vec2::new(bounds.x + 12.0, bounds.y + bounds.height / 2.0),
                font: crate::engine::ui::modern_architecture::FontStyle {
                    family: "system-ui".to_string(),
                    size: 16.0,
                    weight: crate::engine::ui::modern_architecture::FontWeight::Regular,
                    style: crate::engine::ui::modern_architecture::FontVariant::Normal,
                },
                color: if self.props.value.is_empty() {
                    crate::engine::ui::modern_architecture::Color { r: 0.6, g: 0.6, b: 0.6, a: 1.0 }
                } else {
                    crate::engine::ui::modern_architecture::Color { r: 0.2, g: 0.2, b: 0.2, a: 1.0 }
                },
                align: crate::engine::ui::modern_architecture::TextAlign::Left,
            });
        }

        Ok(RenderOutput {
            primitives,
            bounds,
            interaction_bounds: Some(bounds),
            element_type: None,
            content: None,
            style: None,
            attributes: None,
            children: None,
        })
    }
}

impl UIComponent for Input {
    fn component_type(&self) -> &'static str {
        "input"
    }

    fn props(&self) -> &ComponentProps {
        &self.props.base
    }

    fn set_props(&mut self, props: ComponentProps) -> RobinResult<()> {
        self.props.base = props;
        self.update_style();
        Ok(())
    }

    fn handle_event(&mut self, event: UIEvent) -> RobinResult<Vec<UIEvent>> {
        match event {
            UIEvent::Input { component_id, value } if component_id == self.id => {
                self.props.value = value.clone();
                self.validate()?;
                self.update_style();

                return Ok(vec![UIEvent::Change {
                    component_id: self.id.to_string(),
                    value,
                }]);
            },
            UIEvent::Focus { component_id, focused } if component_id == self.id => {
                self.state.focused = focused;
                self.update_style();

                if focused {
                    return Ok(vec![UIEvent::Custom {
                        event_type: "input_focus".to_string(),
                        component_id: self.id.to_string(),
                        data: HashMap::new(),
                    }]);
                } else {
                    // Trigger blur validation
                    self.validate()?;
                    return Ok(vec![UIEvent::Custom {
                        event_type: "input_blur".to_string(),
                        component_id: self.id.to_string(),
                        data: [("valid".to_string(), self.is_valid().to_string())].into(),
                    }]);
                }
            },
            _ => {}
        }
        Ok(Vec::new())
    }

    fn is_interactive(&self) -> bool {
        !self.props.base.disabled && !self.props.readonly
    }

    fn accessibility_info(&self) -> AccessibilityInfo {
        AccessibilityInfo {
            label: self.props.label.clone().unwrap_or_else(|| "Input field".to_string()),
            description: self.props.help_text.clone(),
            role: "textbox".to_string(),
            states: {
                let mut states = Vec::new();
                if self.props.base.disabled { states.push("disabled".to_string()); }
                if self.props.readonly { states.push("readonly".to_string()); }
                if self.props.required { states.push("required".to_string()); }
                if self.state.invalid { states.push("invalid".to_string()); }
                states
            },
            actions: vec!["type".to_string(), "clear".to_string()],
        }
    }

    fn apply_theme(&mut self, _theme: &UITheme) -> RobinResult<()> {
        self.update_style();
        Ok(())
    }
}

impl Input {
    fn generate_attributes(&self) -> HashMap<String, String> {
        let mut attrs = HashMap::new();

        let input_type = match self.props.input_type {
            InputType::Text => "text",
            InputType::Password => "password",
            InputType::Email => "email",
            InputType::Number => "number",
            InputType::Tel => "tel",
            InputType::Url => "url",
            InputType::Search => "search",
            InputType::TextArea => "text",
        };

        if !matches!(self.props.input_type, InputType::TextArea) {
            attrs.insert("type".to_string(), input_type.to_string());
        }

        attrs.insert("id".to_string(), self.props.base.id.clone());
        attrs.insert("value".to_string(), self.props.value.clone());

        if let Some(ref placeholder) = self.props.placeholder {
            attrs.insert("placeholder".to_string(), placeholder.clone());
        }

        if self.props.base.disabled {
            attrs.insert("disabled".to_string(), "true".to_string());
        }

        if self.props.readonly {
            attrs.insert("readonly".to_string(), "true".to_string());
        }

        if self.props.required {
            attrs.insert("required".to_string(), "true".to_string());
        }

        if let Some(max_length) = self.props.max_length {
            attrs.insert("maxlength".to_string(), max_length.to_string());
        }

        if let Some(ref pattern) = self.props.pattern {
            attrs.insert("pattern".to_string(), pattern.clone());
        }

        // Accessibility
        if let Some(ref label) = self.props.label {
            attrs.insert("aria-label".to_string(), label.clone());
        }

        if self.state.invalid {
            attrs.insert("aria-invalid".to_string(), "true".to_string());
            if !self.state.validation_errors.is_empty() {
                attrs.insert("aria-describedby".to_string(), format!("{}-error", self.id));
            }
        }

        attrs
    }
}

/// Select/Dropdown component
#[derive(Debug)]
pub struct Select {
    id: ComponentId,
    props: SelectProps,
    state: SelectState,
    style: Style,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectProps {
    pub base: ComponentProps,
    pub options: Vec<SelectOption>,
    pub value: Option<String>,
    pub placeholder: Option<String>,
    pub label: Option<String>,
    pub multiple: bool,
    pub searchable: bool,
    pub size: Size,
    pub variant: InputVariant,
    pub clearable: bool,
    pub on_change: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
    pub disabled: bool,
    pub group: Option<String>,
}

#[derive(Debug, Clone)]
struct SelectState {
    open: bool,
    focused: bool,
    search_query: String,
    highlighted_index: Option<usize>,
}

impl Default for SelectProps {
    fn default() -> Self {
        Self {
            base: ComponentProps::default(),
            options: Vec::new(),
            value: None,
            placeholder: Some("Select an option...".to_string()),
            label: None,
            multiple: false,
            searchable: false,
            size: Size::Medium,
            variant: InputVariant::Outlined,
            clearable: false,
            on_change: None,
        }
    }
}

impl Default for SelectState {
    fn default() -> Self {
        Self {
            open: false,
            focused: false,
            search_query: String::new(),
            highlighted_index: None,
        }
    }
}

impl Select {
    pub fn new(props: SelectProps) -> Self {
        let id = utils::generate_component_id("select");
        let mut select = Self {
            id,
            props,
            state: SelectState::default(),
            style: Style::default(),
        };
        select.update_style();
        select
    }

    pub fn add_option(&mut self, value: String, label: String) {
        self.props.options.push(SelectOption {
            value,
            label,
            disabled: false,
            group: None,
        });
    }

    pub fn set_value(&mut self, value: Option<String>) {
        self.props.value = value;
    }

    pub fn get_value(&self) -> Option<&String> {
        self.props.value.as_ref()
    }

    fn update_style(&mut self) {
        // Similar to Input styling but with dropdown-specific styles
        self.style = Style::default();
        // Implementation would be similar to Input with dropdown-specific adjustments
    }

    fn get_filtered_options(&self) -> Vec<&SelectOption> {
        if self.props.searchable && !self.state.search_query.is_empty() {
            self.props.options.iter()
                .filter(|option| option.label.to_lowercase().contains(&self.state.search_query.to_lowercase()))
                .collect()
        } else {
            self.props.options.iter().collect()
        }
    }
}

impl Component for Select {
    fn id(&self) -> ComponentId {
        self.id.clone()
    }

    fn type_name(&self) -> &'static str {
        "select"
    }

    fn init(&mut self, _ctx: &mut ComponentContext) -> RobinResult<()> {
        Ok(())
    }

    fn update(&mut self, _ctx: &mut ComponentContext, _delta_time: f32) -> RobinResult<()> {
        Ok(())
    }

    fn render(&self, _ctx: &RenderContext) -> RobinResult<RenderOutput> {
        let bounds = Bounds {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 40.0,
        };

        let mut primitives = vec![
            RenderPrimitive::Rectangle {
                bounds,
                fill: Some(crate::engine::ui::modern_architecture::Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }),
                stroke: Some((crate::engine::ui::modern_architecture::Color { r: 0.8, g: 0.8, b: 0.8, a: 1.0 }, 1.0)),
                border_radius: 6.0,
            }
        ];

        let display_text = self.props.value.clone().unwrap_or_else(||
            self.props.placeholder.clone().unwrap_or_else(|| "Select...".to_string())
        );

        primitives.push(RenderPrimitive::Text {
            content: display_text,
            position: Vec2::new(bounds.x + 12.0, bounds.y + bounds.height / 2.0),
            font: crate::engine::ui::modern_architecture::FontStyle {
                family: "system-ui".to_string(),
                size: 16.0,
                weight: crate::engine::ui::modern_architecture::FontWeight::Regular,
                style: crate::engine::ui::modern_architecture::FontVariant::Normal,
            },
            color: if self.props.value.is_some() {
                crate::engine::ui::modern_architecture::Color { r: 0.2, g: 0.2, b: 0.2, a: 1.0 }
            } else {
                crate::engine::ui::modern_architecture::Color { r: 0.6, g: 0.6, b: 0.6, a: 1.0 }
            },
            align: crate::engine::ui::modern_architecture::TextAlign::Left,
        });

        // Add dropdown arrow
        primitives.push(RenderPrimitive::Text {
            content: "▼".to_string(),
            position: Vec2::new(bounds.x + bounds.width - 25.0, bounds.y + bounds.height / 2.0),
            font: crate::engine::ui::modern_architecture::FontStyle {
                family: "system-ui".to_string(),
                size: 12.0,
                weight: crate::engine::ui::modern_architecture::FontWeight::Regular,
                style: crate::engine::ui::modern_architecture::FontVariant::Normal,
            },
            color: crate::engine::ui::modern_architecture::Color { r: 0.5, g: 0.5, b: 0.5, a: 1.0 },
            align: crate::engine::ui::modern_architecture::TextAlign::Center,
        });

        Ok(RenderOutput {
            primitives,
            bounds,
            interaction_bounds: Some(bounds),
            element_type: None,
            content: None,
            style: None,
            attributes: None,
            children: None,
        })
    }
}

impl UIComponent for Select {
    fn component_type(&self) -> &'static str {
        "select"
    }

    fn props(&self) -> &ComponentProps {
        &self.props.base
    }

    fn set_props(&mut self, props: ComponentProps) -> RobinResult<()> {
        self.props.base = props;
        self.update_style();
        Ok(())
    }

    fn handle_event(&mut self, event: UIEvent) -> RobinResult<Vec<UIEvent>> {
        match event {
            UIEvent::Click { component_id, .. } if component_id == self.id => {
                self.state.open = !self.state.open;
                return Ok(vec![UIEvent::Custom {
                    event_type: "select_toggle".to_string(),
                    component_id: self.id.to_string(),
                    data: [("open".to_string(), self.state.open.to_string())].into(),
                }]);
            },
            UIEvent::Focus { component_id, focused } if component_id == self.id => {
                self.state.focused = focused;
                if !focused {
                    self.state.open = false;
                }
            },
            _ => {}
        }
        Ok(Vec::new())
    }

    fn is_interactive(&self) -> bool {
        !self.props.base.disabled
    }

    fn accessibility_info(&self) -> AccessibilityInfo {
        AccessibilityInfo {
            label: self.props.label.clone().unwrap_or_else(|| "Select".to_string()),
            description: None,
            role: "combobox".to_string(),
            states: {
                let mut states = Vec::new();
                if self.props.base.disabled { states.push("disabled".to_string()); }
                if self.state.open { states.push("expanded".to_string()); } else { states.push("collapsed".to_string()); }
                states
            },
            actions: vec!["open".to_string(), "close".to_string(), "select".to_string()],
        }
    }

    fn apply_theme(&mut self, _theme: &UITheme) -> RobinResult<()> {
        self.update_style();
        Ok(())
    }
}

impl Select {
    fn generate_attributes(&self) -> HashMap<String, String> {
        let mut attrs = HashMap::new();
        attrs.insert("id".to_string(), self.props.base.id.clone());
        attrs.insert("role".to_string(), "combobox".to_string());
        attrs.insert("aria-expanded".to_string(), self.state.open.to_string());

        if self.props.base.disabled {
            attrs.insert("disabled".to_string(), "true".to_string());
        }

        attrs
    }
}

/// Checkbox component
#[derive(Debug)]
pub struct Checkbox {
    id: ComponentId,
    props: CheckboxProps,
    state: CheckboxState,
    style: Style,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckboxProps {
    pub base: ComponentProps,
    pub checked: bool,
    pub indeterminate: bool,
    pub label: Option<String>,
    pub size: Size,
    pub color: ColorVariant,
    pub on_change: Option<String>,
}

#[derive(Debug, Clone)]
struct CheckboxState {
    focused: bool,
    hovered: bool,
}

impl Default for CheckboxProps {
    fn default() -> Self {
        Self {
            base: ComponentProps::default(),
            checked: false,
            indeterminate: false,
            label: None,
            size: Size::Medium,
            color: ColorVariant::Primary,
            on_change: None,
        }
    }
}

impl Default for CheckboxState {
    fn default() -> Self {
        Self {
            focused: false,
            hovered: false,
        }
    }
}

impl Checkbox {
    pub fn new(props: CheckboxProps) -> Self {
        let id = utils::generate_component_id("checkbox");
        let mut checkbox = Self {
            id,
            props,
            state: CheckboxState::default(),
            style: Style::default(),
        };
        checkbox.update_style();
        checkbox
    }

    pub fn set_checked(&mut self, checked: bool) {
        self.props.checked = checked;
        self.props.indeterminate = false;
        self.update_style();
    }

    pub fn is_checked(&self) -> bool {
        self.props.checked
    }

    fn update_style(&mut self) {
        self.style = Style::default();
        // Checkbox-specific styling implementation
    }
}

impl Component for Checkbox {
    fn id(&self) -> ComponentId {
        self.id.clone()
    }

    fn render(&self, _ctx: &RenderContext) -> RobinResult<RenderOutput> {
        let checkbox_size = 20.0;
        let bounds = Bounds {
            x: 0.0,
            y: 0.0,
            width: checkbox_size,
            height: checkbox_size,
        };

        let mut primitives = vec![
            RenderPrimitive::Rectangle {
                bounds,
                fill: Some(if self.props.checked {
                    crate::engine::ui::modern_architecture::Color { r: 0.0, g: 0.5, b: 1.0, a: 1.0 } // Blue when checked
                } else {
                    crate::engine::ui::modern_architecture::Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 } // White when unchecked
                }),
                stroke: Some((crate::engine::ui::modern_architecture::Color { r: 0.7, g: 0.7, b: 0.7, a: 1.0 }, 2.0)),
                border_radius: 3.0,
            }
        ];

        // Add check mark or indeterminate mark
        if self.props.checked {
            primitives.push(RenderPrimitive::Text {
                content: "✓".to_string(),
                position: Vec2::new(bounds.x + bounds.width / 2.0, bounds.y + bounds.height / 2.0),
                font: crate::engine::ui::modern_architecture::FontStyle {
                    family: "system-ui".to_string(),
                    size: 14.0,
                    weight: crate::engine::ui::modern_architecture::FontWeight::Bold,
                    style: crate::engine::ui::modern_architecture::FontVariant::Normal,
                },
                color: crate::engine::ui::modern_architecture::Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }, // White checkmark
                align: crate::engine::ui::modern_architecture::TextAlign::Center,
            });
        } else if self.props.indeterminate {
            primitives.push(RenderPrimitive::Text {
                content: "−".to_string(),
                position: Vec2::new(bounds.x + bounds.width / 2.0, bounds.y + bounds.height / 2.0),
                font: crate::engine::ui::modern_architecture::FontStyle {
                    family: "system-ui".to_string(),
                    size: 16.0,
                    weight: crate::engine::ui::modern_architecture::FontWeight::Bold,
                    style: crate::engine::ui::modern_architecture::FontVariant::Normal,
                },
                color: crate::engine::ui::modern_architecture::Color { r: 0.5, g: 0.5, b: 0.5, a: 1.0 },
                align: crate::engine::ui::modern_architecture::TextAlign::Center,
            });
        }

        Ok(RenderOutput {
            primitives,
            bounds,
            interaction_bounds: Some(bounds),
            element_type: None,
            content: None,
            style: None,
            attributes: None,
            children: None,
        })
    }

    fn type_name(&self) -> &'static str {
        "Checkbox"
    }

    fn init(&mut self, _ctx: &mut ComponentContext) -> RobinResult<()> {
        Ok(())
    }

    fn update(&mut self, _ctx: &mut ComponentContext, _delta_time: f32) -> RobinResult<()> {
        Ok(())
    }
}

impl UIComponent for Checkbox {
    fn component_type(&self) -> &'static str {
        "checkbox"
    }

    fn props(&self) -> &ComponentProps {
        &self.props.base
    }

    fn set_props(&mut self, props: ComponentProps) -> RobinResult<()> {
        self.props.base = props;
        self.update_style();
        Ok(())
    }

    fn handle_event(&mut self, event: UIEvent) -> RobinResult<Vec<UIEvent>> {
        match event {
            UIEvent::Click { component_id, .. } if component_id == self.id => {
                if !self.props.base.disabled {
                    self.props.checked = !self.props.checked;
                    self.props.indeterminate = false;
                    self.update_style();

                    return Ok(vec![UIEvent::Change {
                        component_id: self.id.to_string(),
                        value: self.props.checked.to_string(),
                    }]);
                }
            },
            UIEvent::Focus { component_id, focused } if component_id == self.id => {
                self.state.focused = focused;
                self.update_style();
            },
            UIEvent::Hover { component_id, entered } if component_id == self.id => {
                self.state.hovered = entered;
                self.update_style();
            },
            _ => {}
        }
        Ok(Vec::new())
    }

    fn is_interactive(&self) -> bool {
        !self.props.base.disabled
    }

    fn accessibility_info(&self) -> AccessibilityInfo {
        AccessibilityInfo {
            label: self.props.label.clone().unwrap_or_else(|| "Checkbox".to_string()),
            description: None,
            role: "checkbox".to_string(),
            states: {
                let mut states = Vec::new();
                if self.props.base.disabled { states.push("disabled".to_string()); }
                if self.props.checked { states.push("checked".to_string()); }
                if self.props.indeterminate { states.push("mixed".to_string()); }
                states
            },
            actions: vec!["toggle".to_string()],
        }
    }

    fn apply_theme(&mut self, _theme: &UITheme) -> RobinResult<()> {
        self.update_style();
        Ok(())
    }
}

impl Checkbox {
    fn generate_attributes(&self) -> HashMap<String, String> {
        let mut attrs = HashMap::new();
        attrs.insert("type".to_string(), "checkbox".to_string());
        attrs.insert("id".to_string(), self.props.base.id.clone());
        attrs.insert("checked".to_string(), self.props.checked.to_string());

        if self.props.indeterminate {
            attrs.insert("aria-checked".to_string(), "mixed".to_string());
        }

        if self.props.base.disabled {
            attrs.insert("disabled".to_string(), "true".to_string());
        }

        attrs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_validation() {
        let mut input = Input::new(InputProps {
            validation: vec![ValidationRule::Required, ValidationRule::Email],
            ..Default::default()
        });

        // Test empty value (should be invalid due to required)
        input.set_value("").unwrap();
        assert!(!input.is_valid());

        // Test invalid email (should be invalid)
        input.set_value("invalid-email").unwrap();
        assert!(!input.is_valid());

        // Test valid email (should be valid)
        input.set_value("test@example.com").unwrap();
        assert!(input.is_valid());
    }

    #[test]
    fn test_checkbox_toggle() {
        let mut checkbox = Checkbox::new(CheckboxProps::default());
        assert!(!checkbox.is_checked());

        let click_event = UIEvent::Click {
            component_id: checkbox.id(),
            position: (0.0, 0.0),
        };

        checkbox.handle_event(click_event).unwrap();
        assert!(checkbox.is_checked());
    }
}