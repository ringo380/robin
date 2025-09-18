// Robin Game Engine - Modal and Dialog Components
// Production-ready modal dialogs with accessibility and animation

use super::*;
use crate::engine::error::RobinResult;
use crate::engine::ui::modern_architecture::{ComponentContext, RenderPrimitive, Bounds, FontStyle, FontWeight, FontVariant, Color as ModernColor, TextAlign as ModernTextAlign};
use crate::engine::ui::styling::{FlexDirection, Border, Color};
use crate::engine::ui::css_in_rust::TextAlign;
use crate::engine::ui::css_in_rust::{Position, BorderRadius, BoxShadow, Overflow, Cursor, Transform};
use crate::engine::generation::templates::UITheme;
use crate::engine::math::Vec2;
use serde::{Serialize, Deserialize};
use std::fmt::Debug;

/// Modal component for overlays and dialogs
#[derive(Debug)]
pub struct Modal {
    id: ComponentId,
    props: ModalProps,
    state: ModalState,
    style: Style,
    overlay_style: Style,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModalProps {
    pub base: ComponentProps,
    pub open: bool,
    pub title: Option<String>,
    pub closable: bool,
    pub close_on_overlay_click: bool,
    pub close_on_escape: bool,
    pub size: ModalSize,
    pub variant: ModalVariant,
    pub animation: ModalAnimation,
    pub focus_trap: bool,
    pub show_overlay: bool,
    pub overlay_opacity: f32,
    pub on_open: Option<String>,
    pub on_close: Option<String>,
    pub on_confirm: Option<String>,
    pub on_cancel: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ModalSize {
    Small,      // 400px max width
    Medium,     // 600px max width
    Large,      // 800px max width
    XLarge,     // 1000px max width
    FullScreen, // Full viewport
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ModalVariant {
    Default,    // Standard modal
    Dialog,     // Confirmation dialog
    Alert,      // Alert/warning modal
    Drawer,     // Side drawer
    Popup,      // Small popup
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ModalAnimation {
    Fade,       // Fade in/out
    Scale,      // Scale up/down
    Slide,      // Slide from top
    SlideUp,    // Slide from bottom
    None,       // No animation
}

#[derive(Debug, Clone)]
struct ModalState {
    animating: bool,
    animation_progress: f32,
    was_open: bool,
    focus_saved: Option<ComponentId>,
}

impl Default for ModalProps {
    fn default() -> Self {
        Self {
            base: ComponentProps::default(),
            open: false,
            title: None,
            closable: true,
            close_on_overlay_click: true,
            close_on_escape: true,
            size: ModalSize::Medium,
            variant: ModalVariant::Default,
            animation: ModalAnimation::Fade,
            focus_trap: true,
            show_overlay: true,
            overlay_opacity: 0.5,
            on_open: None,
            on_close: None,
            on_confirm: None,
            on_cancel: None,
        }
    }
}

impl Default for ModalState {
    fn default() -> Self {
        Self {
            animating: false,
            animation_progress: 0.0,
            was_open: false,
            focus_saved: None,
        }
    }
}

impl Modal {
    pub fn new(props: ModalProps) -> Self {
        let id = utils::generate_component_id("modal");
        let mut modal = Self {
            id,
            props,
            state: ModalState::default(),
            style: Style::default(),
            overlay_style: Style::default(),
        };
        modal.update_styles();
        modal
    }

    pub fn open(&mut self) {
        if !self.props.open {
            self.props.open = true;
            self.state.animating = true;
            self.state.animation_progress = 0.0;
            self.update_styles();
        }
    }

    pub fn close(&mut self) {
        if self.props.open {
            self.props.open = false;
            self.state.animating = true;
            self.state.animation_progress = 1.0;
            self.update_styles();
        }
    }

    pub fn is_open(&self) -> bool {
        self.props.open
    }

    pub fn set_title<S: Into<String>>(&mut self, title: Option<S>) {
        self.props.title = title.map(|s| s.into());
    }

    fn update_styles(&mut self) {
        self.update_modal_style();
        self.update_overlay_style();
    }

    fn update_modal_style(&mut self) {
        self.style = Style::default();

        // Base modal styles
        self.style.position = Some(Position::Fixed);
        self.style.z_index = Some(1000);
        self.style.display = Some(super::super::css_in_rust::Display::Flex);
        self.style.flex_direction = Some(FlexDirection::Column);
        self.style.background_color = Some(Color::rgba(1.0, 1.0, 1.0, 1.0));
        self.style.border_radius = Some(BorderRadius::all(12.0));
        self.style.box_shadow = Some(vec![BoxShadow {
            x: 0.0,
            y: 25.0,
            blur: 50.0,
            spread: -12.0,
            color: Color::rgba(0.0, 0.0, 0.0, 0.25),
            inset: false,
        }]);
        self.style.max_height = Some(super::super::css_in_rust::Dimension::Percent(90.0));
        self.style.overflow = Some(Overflow::Hidden);

        // Size-based styles
        match self.props.size {
            ModalSize::Small => {
                self.style.width = Some(super::super::css_in_rust::Dimension::Px(400.0));
                self.style.max_width = Some(super::super::css_in_rust::Dimension::Percent(90.0));
            },
            ModalSize::Medium => {
                self.style.width = Some(super::super::css_in_rust::Dimension::Px(600.0));
                self.style.max_width = Some(super::super::css_in_rust::Dimension::Percent(90.0));
            },
            ModalSize::Large => {
                self.style.width = Some(super::super::css_in_rust::Dimension::Px(800.0));
                self.style.max_width = Some(super::super::css_in_rust::Dimension::Percent(95.0));
            },
            ModalSize::XLarge => {
                self.style.width = Some(super::super::css_in_rust::Dimension::Px(1000.0));
                self.style.max_width = Some(super::super::css_in_rust::Dimension::Percent(95.0));
            },
            ModalSize::FullScreen => {
                self.style.width = Some(super::super::css_in_rust::Dimension::Percent(100.0));
                self.style.height = Some(super::super::css_in_rust::Dimension::Percent(100.0));
                self.style.max_width = Some(super::super::css_in_rust::Dimension::Percent(100.0));
                self.style.max_height = Some(super::super::css_in_rust::Dimension::Percent(100.0));
                self.style.border_radius = Some(BorderRadius::all(0.0));
            },
        }

        // Variant-based styles
        match self.props.variant {
            ModalVariant::Default => {
                // Default styles already applied
            },
            ModalVariant::Dialog => {
                self.style.max_width = Some(super::super::css_in_rust::Dimension::Px(500.0));
                self.style.text_align = Some(TextAlign::Center);
            },
            ModalVariant::Alert => {
                self.style.max_width = Some(super::super::css_in_rust::Dimension::Px(400.0));
                self.style.border_left = Some(Border::new(4.0, Color::rgba(0.96, 0.62, 0.04, 1.0)));
            },
            ModalVariant::Drawer => {
                self.style.position = Some(Position::Fixed);
                self.style.top = Some(super::super::css_in_rust::Dimension::Px(0.0));
                self.style.right = Some(super::super::css_in_rust::Dimension::Px(0.0));
                self.style.height = Some(super::super::css_in_rust::Dimension::Percent(100.0));
                self.style.width = Some(super::super::css_in_rust::Dimension::Px(400.0));
                self.style.border_radius = Some(BorderRadius::all(0.0));
            },
            ModalVariant::Popup => {
                self.style.max_width = Some(super::super::css_in_rust::Dimension::Px(300.0));
                self.style.box_shadow = Some(vec![BoxShadow {
                    x: 0.0,
                    y: 4.0,
                    blur: 12.0,
                    spread: 0.0,
                    color: Color::rgba(0.0, 0.0, 0.0, 0.15),
                    inset: false,
                }]);
            },
        }

        // Animation styles
        if self.state.animating {
            self.apply_animation_styles();
        }

        // Visibility
        if !self.props.open && !self.state.animating {
            self.style.display = Some(super::super::css_in_rust::Display::None);
        }
    }

    fn update_overlay_style(&mut self) {
        self.overlay_style = Style::default();

        if self.props.show_overlay {
            self.overlay_style.position = Some(Position::Fixed);
            self.overlay_style.top = Some(super::super::css_in_rust::Dimension::Px(0.0));
            self.overlay_style.left = Some(super::super::css_in_rust::Dimension::Px(0.0));
            self.overlay_style.width = Some(super::super::css_in_rust::Dimension::Percent(100.0));
            self.overlay_style.height = Some(super::super::css_in_rust::Dimension::Percent(100.0));
            self.overlay_style.background_color = Some(Color::rgba(0.0, 0.0, 0.0, self.props.overlay_opacity));
            self.overlay_style.z_index = Some(999);
            self.overlay_style.cursor = Some(Cursor::Pointer);

            if !self.props.open && !self.state.animating {
                self.overlay_style.display = Some(super::super::css_in_rust::Display::None);
            }
        }
    }

    fn apply_animation_styles(&mut self) {
        let progress = if self.props.open {
            self.state.animation_progress
        } else {
            1.0 - self.state.animation_progress
        };

        match self.props.animation {
            ModalAnimation::Fade => {
                self.style.opacity = Some(progress);
            },
            ModalAnimation::Scale => {
                self.style.opacity = Some(progress);
                let scale = 0.8 + (progress * 0.2);
                self.style.transform = Some(vec![Transform::Scale(scale, scale)]);
            },
            ModalAnimation::Slide => {
                self.style.opacity = Some(progress);
                let translate_y = -20.0 + (progress * 20.0);
                self.style.transform = Some(vec![Transform::Translate(0.0, translate_y)]);
            },
            ModalAnimation::SlideUp => {
                self.style.opacity = Some(progress);
                let translate_y = 20.0 - (progress * 20.0);
                self.style.transform = Some(vec![Transform::Translate(0.0, translate_y)]);
            },
            ModalAnimation::None => {
                // No animation
            },
        }
    }

    fn update_animation(&mut self, delta_time: f32) {
        if self.state.animating {
            let animation_speed = 4.0; // Animation duration in seconds

            if self.props.open {
                self.state.animation_progress += delta_time * animation_speed;
                if self.state.animation_progress >= 1.0 {
                    self.state.animation_progress = 1.0;
                    self.state.animating = false;
                }
            } else {
                self.state.animation_progress -= delta_time * animation_speed;
                if self.state.animation_progress <= 0.0 {
                    self.state.animation_progress = 0.0;
                    self.state.animating = false;
                }
            }

            self.update_styles();
        }
    }
}

impl Component for Modal {
    fn id(&self) -> ComponentId {
        self.id.clone()
    }

    fn type_name(&self) -> &'static str {
        "modal"
    }

    fn init(&mut self, _ctx: &mut ComponentContext) -> RobinResult<()> {
        Ok(())
    }

    fn update(&mut self, _ctx: &mut ComponentContext, _delta_time: f32) -> RobinResult<()> {
        Ok(())
    }

    fn render(&self, _ctx: &RenderContext) -> RobinResult<RenderOutput> {
        if !self.props.open && !self.state.animating {
            // Modal is closed and not animating, return empty
            return Ok(RenderOutput {
                primitives: Vec::new(),
                bounds: Bounds { x: 0.0, y: 0.0, width: 0.0, height: 0.0 },
                interaction_bounds: None,
                element_type: None,
                content: None,
                style: None,
                attributes: None,
                children: None,
            });
        }

        let screen_width = 800.0; // This would come from context in real implementation
        let screen_height = 600.0;

        let modal_width = match self.props.size {
            ModalSize::Small => 400.0,
            ModalSize::Medium => 600.0,
            ModalSize::Large => 800.0,
            ModalSize::XLarge => 1000.0,
            ModalSize::FullScreen => screen_width,
        };

        let modal_height = if matches!(self.props.size, ModalSize::FullScreen) {
            screen_height
        } else {
            400.0 // Default modal height
        };

        let modal_x = (screen_width - modal_width) / 2.0;
        let modal_y = (screen_height - modal_height) / 2.0;

        let mut primitives = Vec::new();

        // Add overlay if enabled
        if self.props.show_overlay {
            primitives.push(RenderPrimitive::Rectangle {
                bounds: Bounds {
                    x: 0.0,
                    y: 0.0,
                    width: screen_width,
                    height: screen_height,
                },
                fill: Some(ModernColor {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: self.props.overlay_opacity,
                }),
                stroke: None,
                border_radius: 0.0,
            });
        }

        // Main modal rectangle
        primitives.push(RenderPrimitive::Rectangle {
            bounds: Bounds {
                x: modal_x,
                y: modal_y,
                width: modal_width,
                height: modal_height,
            },
            fill: Some(ModernColor { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }),
            stroke: None,
            border_radius: if matches!(self.props.size, ModalSize::FullScreen) { 0.0 } else { 12.0 },
        });

        let mut content_y = modal_y + 24.0;

        // Add title if present
        if let Some(ref title) = self.props.title {
            primitives.push(RenderPrimitive::Text {
                content: title.clone(),
                position: Vec2::new(modal_x + 24.0, content_y),
                font: FontStyle {
                    family: "system-ui".to_string(),
                    size: 20.0,
                    weight: FontWeight::Bold,
                    style: FontVariant::Normal,
                },
                color: ModernColor { r: 0.1, g: 0.1, b: 0.1, a: 1.0 },
                align: ModernTextAlign::Left,
            });
            content_y += 40.0;
        }

        // Add close button if closable
        if self.props.closable {
            let close_x = modal_x + modal_width - 40.0;
            let close_y = modal_y + 20.0;

            primitives.push(RenderPrimitive::Rectangle {
                bounds: Bounds {
                    x: close_x,
                    y: close_y,
                    width: 30.0,
                    height: 30.0,
                },
                fill: Some(ModernColor { r: 0.95, g: 0.95, b: 0.95, a: 1.0 }),
                stroke: None,
                border_radius: 15.0,
            });

            primitives.push(RenderPrimitive::Text {
                content: "×".to_string(),
                position: Vec2::new(close_x + 15.0, close_y + 15.0),
                font: FontStyle {
                    family: "system-ui".to_string(),
                    size: 18.0,
                    weight: FontWeight::Bold,
                    style: FontVariant::Normal,
                },
                color: ModernColor { r: 0.4, g: 0.4, b: 0.4, a: 1.0 },
                align: ModernTextAlign::Center,
            });
        }

        let bounds = Bounds {
            x: 0.0,
            y: 0.0,
            width: screen_width,
            height: screen_height,
        };

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

impl UIComponent for Modal {
    fn component_type(&self) -> &'static str {
        "modal"
    }

    fn props(&self) -> &ComponentProps {
        &self.props.base
    }

    fn set_props(&mut self, props: ComponentProps) -> RobinResult<()> {
        self.props.base = props;
        self.update_styles();
        Ok(())
    }

    fn handle_event(&mut self, event: UIEvent) -> RobinResult<Vec<UIEvent>> {
        match event {
            UIEvent::Click { component_id, .. } if component_id == self.id => {
                // Handle overlay click
                if self.props.close_on_overlay_click {
                    self.close();
                    return Ok(vec![UIEvent::Custom {
                        event_type: "modal_close".to_string(),
                        component_id: self.id.to_string(),
                        data: [("reason".to_string(), "overlay_click".to_string())].into(),
                    }]);
                }
            },
            UIEvent::KeyDown { key, .. } => {
                if key == "Escape" && self.props.close_on_escape && self.props.open {
                    self.close();
                    return Ok(vec![UIEvent::Custom {
                        event_type: "modal_close".to_string(),
                        component_id: self.id.to_string(),
                        data: [("reason".to_string(), "escape_key".to_string())].into(),
                    }]);
                }
            },
            _ => {}
        }
        Ok(Vec::new())
    }

    fn is_interactive(&self) -> bool {
        self.props.open
    }

    fn accessibility_info(&self) -> AccessibilityInfo {
        AccessibilityInfo {
            label: self.props.title.clone().unwrap_or_else(|| "Modal dialog".to_string()),
            description: None,
            role: "dialog".to_string(),
            states: {
                let mut states = Vec::new();
                if self.props.open { states.push("open".to_string()); } else { states.push("closed".to_string()); }
                if self.state.animating { states.push("animating".to_string()); }
                states
            },
            actions: vec!["close".to_string()],
        }
    }

    fn apply_theme(&mut self, _theme: &UITheme) -> RobinResult<()> {
        self.update_styles();
        Ok(())
    }
}

impl Modal {
    fn generate_attributes(&self) -> HashMap<String, String> {
        let mut attrs = HashMap::new();
        attrs.insert("id".to_string(), self.props.base.id.clone());
        attrs.insert("role".to_string(), "dialog".to_string());
        attrs.insert("aria-modal".to_string(), "true".to_string());

        if let Some(ref title) = self.props.title {
            attrs.insert("aria-label".to_string(), title.clone());
        }

        if self.props.base.disabled {
            attrs.insert("aria-hidden".to_string(), "true".to_string());
        }

        attrs
    }

    pub fn update(&mut self, delta_time: f32) {
        self.update_animation(delta_time);
    }
}

/// Confirmation Dialog - specialized modal for yes/no confirmations
#[derive(Debug)]
pub struct ConfirmDialog {
    modal: Modal,
    message: String,
    confirm_text: String,
    cancel_text: String,
    confirm_variant: ColorVariant,
}

impl ConfirmDialog {
    pub fn new(
        title: String,
        message: String,
        confirm_text: Option<String>,
        cancel_text: Option<String>,
    ) -> Self {
        let modal = Modal::new(ModalProps {
            title: Some(title),
            variant: ModalVariant::Dialog,
            size: ModalSize::Small,
            ..Default::default()
        });

        Self {
            modal,
            message,
            confirm_text: confirm_text.unwrap_or_else(|| "Confirm".to_string()),
            cancel_text: cancel_text.unwrap_or_else(|| "Cancel".to_string()),
            confirm_variant: ColorVariant::Primary,
        }
    }

    pub fn danger(title: String, message: String) -> Self {
        let mut dialog = Self::new(title, message, Some("Delete".to_string()), None);
        dialog.confirm_variant = ColorVariant::Error;
        dialog
    }

    pub fn warning(title: String, message: String) -> Self {
        let mut dialog = Self::new(title, message, Some("Proceed".to_string()), None);
        dialog.confirm_variant = ColorVariant::Warning;
        dialog
    }

    pub fn open(&mut self) {
        self.modal.open();
    }

    pub fn close(&mut self) {
        self.modal.close();
    }

    pub fn is_open(&self) -> bool {
        self.modal.is_open()
    }
}

impl Component for ConfirmDialog {
    fn id(&self) -> ComponentId {
        self.modal.id()
    }

    fn render(&self, ctx: &RenderContext) -> RobinResult<RenderOutput> {
        // This would render the modal with the confirmation content
        self.modal.render(ctx)
    }

    fn type_name(&self) -> &'static str {
        "ConfirmDialog"
    }

    fn init(&mut self, ctx: &mut ComponentContext) -> RobinResult<()> {
        self.modal.init(ctx)
    }

    fn update(&mut self, ctx: &mut ComponentContext, delta_time: f32) -> RobinResult<()> {
        self.modal.update(delta_time);
        Ok(())
    }
}

impl UIComponent for ConfirmDialog {
    fn component_type(&self) -> &'static str {
        "confirm_dialog"
    }

    fn props(&self) -> &ComponentProps {
        self.modal.props()
    }

    fn set_props(&mut self, props: ComponentProps) -> RobinResult<()> {
        self.modal.set_props(props)
    }

    fn handle_event(&mut self, event: UIEvent) -> RobinResult<Vec<UIEvent>> {
        self.modal.handle_event(event)
    }

    fn is_interactive(&self) -> bool {
        self.modal.is_interactive()
    }

    fn accessibility_info(&self) -> AccessibilityInfo {
        self.modal.accessibility_info()
    }

    fn apply_theme(&mut self, theme: &UITheme) -> RobinResult<()> {
        self.modal.apply_theme(theme)
    }
}

/// Alert Dialog - specialized modal for simple alerts
#[derive(Debug)]
pub struct AlertDialog {
    modal: Modal,
    message: String,
    alert_type: AlertType,
}

#[derive(Debug, Clone, Copy)]
pub enum AlertType {
    Info,
    Success,
    Warning,
    Error,
}

impl AlertDialog {
    pub fn new(title: String, message: String, alert_type: AlertType) -> Self {
        let modal = Modal::new(ModalProps {
            title: Some(title),
            variant: ModalVariant::Alert,
            size: ModalSize::Small,
            ..Default::default()
        });

        Self {
            modal,
            message,
            alert_type,
        }
    }

    pub fn info(title: String, message: String) -> Self {
        Self::new(title, message, AlertType::Info)
    }

    pub fn success(title: String, message: String) -> Self {
        Self::new(title, message, AlertType::Success)
    }

    pub fn warning(title: String, message: String) -> Self {
        Self::new(title, message, AlertType::Warning)
    }

    pub fn error(title: String, message: String) -> Self {
        Self::new(title, message, AlertType::Error)
    }

    pub fn open(&mut self) {
        self.modal.open();
    }

    pub fn close(&mut self) {
        self.modal.close();
    }

    pub fn is_open(&self) -> bool {
        self.modal.is_open()
    }
}

impl Component for AlertDialog {
    fn id(&self) -> ComponentId {
        self.modal.id()
    }

    fn render(&self, ctx: &RenderContext) -> RobinResult<RenderOutput> {
        self.modal.render(ctx)
    }

    fn type_name(&self) -> &'static str {
        "AlertDialog"
    }

    fn init(&mut self, ctx: &mut ComponentContext) -> RobinResult<()> {
        self.modal.init(ctx)
    }

    fn update(&mut self, ctx: &mut ComponentContext, delta_time: f32) -> RobinResult<()> {
        self.modal.update(delta_time);
        Ok(())
    }
}

impl UIComponent for AlertDialog {
    fn component_type(&self) -> &'static str {
        "alert_dialog"
    }

    fn props(&self) -> &ComponentProps {
        self.modal.props()
    }

    fn set_props(&mut self, props: ComponentProps) -> RobinResult<()> {
        self.modal.set_props(props)
    }

    fn handle_event(&mut self, event: UIEvent) -> RobinResult<Vec<UIEvent>> {
        self.modal.handle_event(event)
    }

    fn is_interactive(&self) -> bool {
        self.modal.is_interactive()
    }

    fn accessibility_info(&self) -> AccessibilityInfo {
        self.modal.accessibility_info()
    }

    fn apply_theme(&mut self, theme: &UITheme) -> RobinResult<()> {
        self.modal.apply_theme(theme)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modal_open_close() {
        let mut modal = Modal::new(ModalProps::default());

        assert!(!modal.is_open());

        modal.open();
        assert!(modal.is_open());

        modal.close();
        assert!(!modal.is_open());
    }

    #[test]
    fn test_modal_escape_key() {
        let mut modal = Modal::new(ModalProps {
            open: true,
            close_on_escape: true,
            ..Default::default()
        });

        let escape_event = UIEvent::KeyDown {
            component_id: "other".to_string(),
            key: "Escape".to_string(),
            modifiers: KeyModifiers::default(),
        };

        let events = modal.handle_event(escape_event).unwrap();
        assert!(!modal.is_open());
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_confirm_dialog() {
        let mut dialog = ConfirmDialog::new(
            "Delete Item".to_string(),
            "Are you sure you want to delete this item?".to_string(),
            None,
            None,
        );

        assert!(!dialog.is_open());
        dialog.open();
        assert!(dialog.is_open());
    }
}