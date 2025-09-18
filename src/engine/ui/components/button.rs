// Robin Game Engine - Button Components
// Modern, accessible button implementations with multiple variants

use super::*;
use crate::engine::error::RobinResult;
use crate::engine::ui::modern_architecture::ComponentContext;
use crate::engine::ui::css_in_rust::{
    Display, Transition, TimingFunction,
    TextAlign, TextTransform, FontWeight, Dimension,
    Cursor, BoxSpacing, BorderStyle, BorderRadius,
    Filter, Transform, BoxShadow
};
use crate::engine::ui::styling::{
    AlignItems, JustifyContent, BoxSizing, Color as StylingColor,
    Border, TextDecoration
};
use crate::engine::generation::templates::UITheme;
use serde::{Serialize, Deserialize};
use std::fmt::Debug;
use std::collections::HashMap;

/// Button variant types
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ButtonVariant {
    Filled,      // Solid background
    Outlined,    // Border only
    Text,        // Text only, no background
    Ghost,       // Subtle background on hover
    Link,        // Styled like a link
}

impl Default for ButtonVariant {
    fn default() -> Self {
        ButtonVariant::Filled
    }
}

/// Button component properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonProps {
    pub base: ComponentProps,
    pub text: String,
    pub variant: ButtonVariant,
    pub color: ColorVariant,
    pub size: Size,
    pub icon: Option<String>,
    pub icon_position: IconPosition,
    pub loading: bool,
    pub full_width: bool,
    pub elevation: u8,
    pub on_click: Option<String>, // Event handler name
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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

impl Default for ButtonProps {
    fn default() -> Self {
        Self {
            base: ComponentProps::default(),
            text: "Button".to_string(),
            variant: ButtonVariant::default(),
            color: ColorVariant::default(),
            size: Size::default(),
            icon: None,
            icon_position: IconPosition::default(),
            loading: false,
            full_width: false,
            elevation: 1,
            on_click: None,
        }
    }
}

/// Button component implementation
#[derive(Debug)]
pub struct Button {
    id: ComponentId,
    props: ButtonProps,
    state: ButtonState,
    style: Style,
}

#[derive(Debug, Clone)]
struct ButtonState {
    hovered: bool,
    pressed: bool,
    focused: bool,
    animation_state: AnimationState,
}

impl Default for ButtonState {
    fn default() -> Self {
        Self {
            hovered: false,
            pressed: false,
            focused: false,
            animation_state: AnimationState::Idle,
        }
    }
}

impl Button {
    pub fn new(props: ButtonProps) -> Self {
        let id = utils::generate_component_id("button");
        let mut button = Self {
            id,
            props,
            state: ButtonState::default(),
            style: Style::default(),
        };
        button.update_style();
        button
    }

    pub fn with_text<S: Into<String>>(mut self, text: S) -> Self {
        self.props.text = text.into();
        self.update_style();
        self
    }

    pub fn with_variant(mut self, variant: ButtonVariant) -> Self {
        self.props.variant = variant;
        self.update_style();
        self
    }

    pub fn with_color(mut self, color: ColorVariant) -> Self {
        self.props.color = color;
        self.update_style();
        self
    }

    pub fn with_size(mut self, size: Size) -> Self {
        self.props.size = size;
        self.update_style();
        self
    }

    pub fn with_icon<S: Into<String>>(mut self, icon: S, position: IconPosition) -> Self {
        self.props.icon = Some(icon.into());
        self.props.icon_position = position;
        self.update_style();
        self
    }

    pub fn full_width(mut self) -> Self {
        self.props.full_width = true;
        self.update_style();
        self
    }

    pub fn loading(mut self, loading: bool) -> Self {
        self.props.loading = loading;
        self.update_style();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.props.base.disabled = disabled;
        self.update_style();
        self
    }

    fn update_style(&mut self) {
        self.style = self.generate_base_style();
        self.apply_variant_style();
        self.apply_size_style();
        self.apply_state_style();
    }

    fn generate_base_style(&self) -> Style {
        let mut style = Style::default();

        // Base button styles
        style.display = Some(super::super::css_in_rust::Display::Flex);
        style.align_items = Some(AlignItems::Center);
        style.justify_content = Some(JustifyContent::Center);
        style.border_radius = Some(BorderRadius::all(8.0));
        style.cursor = Some(Cursor::Pointer);
        // style.user_select would need proper enum
        style.transition = Some(vec![Transition {
            property: "all".to_string(),
            duration: 0.2,
            timing: TimingFunction::EaseInOut,
            delay: 0.0,
        }]);

        // Typography
        style.font_family = Some("system-ui, -apple-system, sans-serif".to_string());
        style.font_weight = Some(FontWeight::Medium);
        style.text_align = Some(TextAlign::Center);
        style.font_size = Some(16.0);
        style.line_height = Some(1.5);
        style.letter_spacing = Some(0.01);

        // Full width
        if self.props.full_width {
            style.width = Some(Dimension::Percent(100.0));
        }

        // Disabled state
        if self.props.base.disabled {
            style.opacity = Some(0.6);
            style.cursor = Some(Cursor::NotAllowed);
        }

        style
    }

    fn apply_variant_style(&mut self) {
        use crate::engine::ui::modern_architecture::Color;

        // Default colors (would come from theme in real implementation)
        let primary_color = "#007bff";
        let secondary_color = "#6c757d";
        let success_color = "#28a745";
        let warning_color = "#ffc107";
        let error_color = "#dc3545";
        let info_color = "#17a2b8";
        let neutral_color = "#6c757d";

        let color_hex = match self.props.color {
            ColorVariant::Primary => primary_color,
            ColorVariant::Secondary => secondary_color,
            ColorVariant::Success => success_color,
            ColorVariant::Warning => warning_color,
            ColorVariant::Error => error_color,
            ColorVariant::Info => info_color,
            ColorVariant::Neutral => neutral_color,
        };
        let color = StylingColor::from_hex(color_hex.trim_start_matches('#').parse::<u32>().unwrap_or(0x000000));

        match self.props.variant {
            ButtonVariant::Filled => {
                self.style.background_color = Some(color);
                self.style.color = Some(StylingColor::WHITE);
                self.style.border_width = Some(1.0);
                self.style.border_style = Some(BorderStyle::Solid);
                self.style.border_color = Some(color);
            },
            ButtonVariant::Outlined => {
                self.style.background_color = None;
                self.style.color = Some(color);
                self.style.border_width = Some(1.0);
                self.style.border_style = Some(BorderStyle::Solid);
                self.style.border_color = Some(color);
            },
            ButtonVariant::Text => {
                self.style.background_color = None;
                self.style.color = Some(color);
                self.style.border = None;
            },
            ButtonVariant::Ghost => {
                self.style.background_color = None;
                self.style.color = Some(color);
                self.style.border = Some(Border::new(1.0, StylingColor::TRANSPARENT));
            },
            ButtonVariant::Link => {
                self.style.background_color = None;
                self.style.color = Some(color);
                self.style.border = None;
                self.style.text_decoration = Some(TextDecoration::Underline);
            },
        }
    }

    fn apply_size_style(&mut self) {
        utils::apply_size_to_style(&mut self.style, self.props.size);

        // Add specific button size adjustments
        let (min_width, gap) = match self.props.size {
            Size::XSmall => (64.0, 4.0),
            Size::Small => (80.0, 6.0),
            Size::Medium => (96.0, 8.0),
            Size::Large => (112.0, 10.0),
            Size::XLarge => (128.0, 12.0),
        };

        self.style.min_width = Some(super::super::css_in_rust::Dimension::Px(min_width));
        self.style.gap = Some(gap);
    }

    fn apply_state_style(&mut self) {
        if self.state.hovered && !self.props.base.disabled {
            self.apply_hover_style();
        }

        if self.state.pressed && !self.props.base.disabled {
            self.apply_active_style();
        }

        if self.state.focused {
            self.apply_focus_style();
        }

        if self.props.loading {
            self.apply_loading_style();
        }
    }

    fn apply_hover_style(&mut self) {
        match self.props.variant {
            ButtonVariant::Filled => {
                self.style.filter = Some(vec![Filter::Brightness(0.9)]);
                self.style.transform = Some(vec![Transform::Translate(0.0, -1.0)]);
                self.style.box_shadow = Some(vec![BoxShadow {
                    x: 0.0,
                    y: 4.0,
                    blur: 8.0,
                    spread: 0.0,
                    color: StylingColor::new(0.0, 0.0, 0.0, 0.12),
                    inset: false,
                }]);
            },
            ButtonVariant::Outlined | ButtonVariant::Text => {
                self.style.background_color = Some(StylingColor::new(0.0, 0.0, 0.0, 0.04));
            },
            ButtonVariant::Ghost => {
                self.style.background_color = Some(StylingColor::new(0.0, 0.0, 0.0, 0.08));
            },
            ButtonVariant::Link => {
                self.style.text_decoration = None;
            },
        }
    }

    fn apply_active_style(&mut self) {
        self.style.transform = Some(vec![Transform::Translate(0.0, 0.0)]);
        self.style.filter = Some(vec![Filter::Brightness(0.85)]);
    }

    fn apply_focus_style(&mut self) {
        self.style.outline = Some(Border::new(2.0, StylingColor::from_hex(0x007bff)));
        self.style.outline_offset = Some(2.0);
    }

    fn apply_loading_style(&mut self) {
        self.style.cursor = Some(Cursor::Wait);
        // Add spinner animation styles here
    }
}

impl Component for Button {
    fn id(&self) -> ComponentId {
        self.id.clone()
    }

    fn type_name(&self) -> &'static str {
        "button"
    }

    fn init(&mut self, _ctx: &mut ComponentContext) -> RobinResult<()> {
        Ok(())
    }

    fn update(&mut self, _ctx: &mut ComponentContext, _delta_time: f32) -> RobinResult<()> {
        Ok(())
    }

    fn render(&self, ctx: &RenderContext) -> RobinResult<RenderOutput> {
        use crate::engine::ui::modern_architecture::{RenderPrimitive, Bounds, Color, FontStyle, FontWeight, TextAlign};

        let mut content = Vec::new();

        // Add loading spinner if loading
        if self.props.loading {
            content.push("🔄".to_string()); // Simple loading indicator
        }

        // Add icon if present
        if let Some(ref icon) = self.props.icon {
            match self.props.icon_position {
                IconPosition::Left | IconPosition::Only => {
                    content.push(icon.clone());
                }
                _ => {}
            }
        }

        // Add text if not icon-only
        if self.props.icon_position != IconPosition::Only {
            content.push(self.props.text.clone());
        }

        // Add right icon
        if let Some(ref icon) = self.props.icon {
            if matches!(self.props.icon_position, IconPosition::Right) {
                content.push(icon.clone());
            }
        }

        // Create bounds based on size prop
        let (width, height) = match self.props.size {
            Size::XSmall => (60.0, 24.0),
            Size::Small => (80.0, 30.0),
            Size::Medium => (100.0, 40.0),
            Size::Large => (120.0, 50.0),
            Size::XLarge => (140.0, 56.0),
        };

        let bounds = Bounds::new(0.0, 0.0, width, height);

        // Create render primitives for the button
        let mut primitives = vec![
            // Button background
            RenderPrimitive::Rectangle {
                bounds,
                fill: Some(Color::from_hex("#007bff").unwrap_or(Color::new(0.0, 0.48, 1.0, 1.0))),
                stroke: None,
                border_radius: 4.0,
            },
            // Button text
            RenderPrimitive::Text {
                content: content.join(" "),
                position: crate::engine::math::Vec2::new(bounds.x + bounds.width / 2.0, bounds.y + bounds.height / 2.0),
                font: FontStyle {
                    family: "sans-serif".to_string(),
                    size: 14.0,
                    weight: FontWeight::Medium,
                    style: crate::engine::ui::modern_architecture::FontVariant::Normal,
                },
                color: Color::WHITE,
                align: TextAlign::Center,
            },
        ];

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

impl UIComponent for Button {
    fn component_type(&self) -> &'static str {
        "button"
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
            UIEvent::Click { component_id, position } if component_id == self.id.0.to_string() => {
                if !self.props.base.disabled && !self.props.loading {
                    // Trigger click event
                    return Ok(vec![UIEvent::Custom {
                        event_type: "button_click".to_string(),
                        component_id: self.id.0.to_string(),
                        data: [("text".to_string(), self.props.text.clone())].into(),
                    }]);
                }
            },
            UIEvent::Hover { component_id, entered } if component_id == self.id => {
                self.state.hovered = entered;
                self.update_style();
            },
            UIEvent::Focus { component_id, focused } if component_id == self.id => {
                self.state.focused = focused;
                self.update_style();
            },
            UIEvent::KeyDown { component_id, key, .. } if component_id == self.id => {
                if key == "Enter" || key == " " {
                    self.state.pressed = true;
                    self.update_style();
                }
            },
            UIEvent::KeyUp { component_id, key, .. } if component_id == self.id => {
                if (key == "Enter" || key == " ") && self.state.pressed {
                    self.state.pressed = false;
                    self.update_style();

                    // Trigger click event
                    return Ok(vec![UIEvent::Custom {
                        event_type: "button_click".to_string(),
                        component_id: self.id.0.to_string(),
                        data: [("text".to_string(), self.props.text.clone())].into(),
                    }]);
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
            label: self.props.text.clone(),
            description: None,
            role: "button".to_string(),
            states: {
                let mut states = Vec::new();
                if self.props.base.disabled { states.push("disabled".to_string()); }
                if self.props.loading { states.push("busy".to_string()); }
                if self.state.pressed { states.push("pressed".to_string()); }
                states
            },
            actions: vec!["click".to_string(), "activate".to_string()],
        }
    }

    fn apply_theme(&mut self, theme: &UITheme) -> RobinResult<()> {
        // Apply theme colors and update style
        self.update_style();
        Ok(())
    }
}

impl Button {
    fn generate_attributes(&self) -> HashMap<String, String> {
        let mut attrs = HashMap::new();

        attrs.insert("type".to_string(), "button".to_string());
        attrs.insert("id".to_string(), self.props.base.id.clone());

        if let Some(ref class) = self.props.base.class_name {
            attrs.insert("class".to_string(), class.clone());
        }

        if self.props.base.disabled {
            attrs.insert("disabled".to_string(), "true".to_string());
        }

        // Accessibility attributes
        if let Some(ref label) = self.props.base.accessibility.aria_label {
            attrs.insert("aria-label".to_string(), label.clone());
        }

        if self.props.loading {
            attrs.insert("aria-busy".to_string(), "true".to_string());
        }

        // Data attributes
        for (key, value) in &self.props.base.data_attributes {
            attrs.insert(format!("data-{}", key), value.clone());
        }

        attrs
    }
}

/// Button builder for fluent API
pub struct ButtonBuilder {
    props: ButtonProps,
}

impl ButtonBuilder {
    pub fn new() -> Self {
        Self {
            props: ButtonProps::default(),
        }
    }

    pub fn text<S: Into<String>>(mut self, text: S) -> Self {
        self.props.text = text.into();
        self
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.props.variant = variant;
        self
    }

    pub fn color(mut self, color: ColorVariant) -> Self {
        self.props.color = color;
        self
    }

    pub fn size(mut self, size: Size) -> Self {
        self.props.size = size;
        self
    }

    pub fn icon<S: Into<String>>(mut self, icon: S, position: IconPosition) -> Self {
        self.props.icon = Some(icon.into());
        self.props.icon_position = position;
        self
    }

    pub fn full_width(mut self) -> Self {
        self.props.full_width = true;
        self
    }

    pub fn loading(mut self, loading: bool) -> Self {
        self.props.loading = loading;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.props.base.disabled = disabled;
        self
    }

    pub fn id<S: Into<String>>(mut self, id: S) -> Self {
        self.props.base.id = id.into();
        self
    }

    pub fn class<S: Into<String>>(mut self, class: S) -> Self {
        self.props.base.class_name = Some(class.into());
        self
    }

    pub fn on_click<S: Into<String>>(mut self, handler: S) -> Self {
        self.props.on_click = Some(handler.into());
        self
    }

    pub fn build(self) -> Button {
        Button::new(self.props)
    }
}

impl Default for ButtonBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_creation() {
        let button = ButtonBuilder::new()
            .text("Click me")
            .variant(ButtonVariant::Outlined)
            .color(ColorVariant::Primary)
            .size(Size::Large)
            .build();

        assert_eq!(button.props.text, "Click me");
        assert!(matches!(button.props.variant, ButtonVariant::Outlined));
        assert!(matches!(button.props.color, ColorVariant::Primary));
        assert!(matches!(button.props.size, Size::Large));
    }

    #[test]
    fn test_button_events() {
        let mut button = ButtonBuilder::new().text("Test").build();

        let click_event = UIEvent::Click {
            component_id: button.id().0.to_string(),
            position: (0.0, 0.0),
        };

        let events = button.handle_event(click_event).unwrap();
        assert_eq!(events.len(), 1);

        if let UIEvent::Custom { event_type, .. } = &events[0] {
            assert_eq!(event_type, "button_click");
        } else {
            panic!("Expected custom button_click event");
        }
    }

    #[test]
    fn test_disabled_button() {
        let mut button = ButtonBuilder::new()
            .text("Disabled")
            .disabled(true)
            .build();

        let click_event = UIEvent::Click {
            component_id: button.id().0.to_string(),
            position: (0.0, 0.0),
        };

        let events = button.handle_event(click_event).unwrap();
        assert_eq!(events.len(), 0); // Disabled button should not emit events
    }
}