/*!
 * Modern UI Components
 *
 * Enhanced UI components with accessibility features and modern design.
 * Part of Phase 3.1: User Interface and Experience Polish.
 */

use crate::engine::{
    math::Vec2,
    ui::{
        UIBounds, UIState, events::UIEvent, UIElement, ElementId, styling::{StateStyle, Color, Spacing, UITheme, DesignSystem}
    },
    input::InputManager,
};
use std::collections::HashMap;

/// Accessibility features for UI components
#[derive(Debug, Clone)]
pub struct AccessibilityProps {
    pub aria_label: Option<String>,
    pub aria_description: Option<String>,
    pub role: String,
    pub tab_index: i32,
    pub keyboard_shortcuts: Vec<String>,
    pub screen_reader_text: Option<String>,
}

impl Default for AccessibilityProps {
    fn default() -> Self {
        Self {
            aria_label: None,
            aria_description: None,
            role: "generic".to_string(),
            tab_index: 0,
            keyboard_shortcuts: Vec::new(),
            screen_reader_text: None,
        }
    }
}

/// Enhanced modern button with accessibility features
pub struct ModernButton {
    id: ElementId,
    bounds: UIBounds,
    state: UIState,
    visible: bool,
    enabled: bool,
    text: String,
    style: StateStyle,
    accessibility: AccessibilityProps,
    click_callback: Option<Box<dyn Fn() + Send + Sync>>,
    keyboard_focus: bool,
    animation_progress: f32,
}

impl ModernButton {
    pub fn new(id: ElementId, bounds: UIBounds, text: String) -> Self {
        Self {
            id,
            bounds,
            state: UIState::Normal,
            visible: true,
            enabled: true,
            text: text.clone(),
            style: UITheme::modern_button(),
            accessibility: AccessibilityProps {
                aria_label: Some(text),
                role: "button".to_string(),
                tab_index: 0,
                ..AccessibilityProps::default()
            },
            click_callback: None,
            keyboard_focus: false,
            animation_progress: 0.0,
        }
    }

    pub fn primary() -> Self {
        let mut button = Self::new(0, UIBounds::new(0.0, 0.0, 120.0, 40.0), "Button".to_string());
        button.style = UITheme::modern_button();
        button
    }

    pub fn secondary() -> Self {
        let mut button = Self::new(0, UIBounds::new(0.0, 0.0, 120.0, 40.0), "Button".to_string());
        button.style = UITheme::secondary_button();
        button
    }

    pub fn ghost() -> Self {
        let mut button = Self::new(0, UIBounds::new(0.0, 0.0, 120.0, 40.0), "Button".to_string());
        button.style = UITheme::ghost_button();
        button
    }

    pub fn with_text(mut self, text: String) -> Self {
        self.text = text.clone();
        self.accessibility.aria_label = Some(text);
        self
    }

    pub fn with_accessibility(mut self, accessibility: AccessibilityProps) -> Self {
        self.accessibility = accessibility;
        self
    }

    pub fn with_keyboard_shortcut(mut self, shortcut: String) -> Self {
        self.accessibility.keyboard_shortcuts.push(shortcut);
        self
    }

    pub fn with_click_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.click_callback = Some(Box::new(callback));
        self
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.state = UIState::Disabled;
        }
    }

    pub fn is_keyboard_focusable(&self) -> bool {
        self.enabled && self.visible && self.accessibility.tab_index >= 0
    }

    pub fn set_keyboard_focus(&mut self, focused: bool) {
        self.keyboard_focus = focused;
        if focused && self.state == UIState::Normal {
            self.state = UIState::Focused;
        } else if !focused && self.state == UIState::Focused {
            self.state = UIState::Normal;
        }
    }

    pub fn get_accessibility_info(&self) -> &AccessibilityProps {
        &self.accessibility
    }

    pub fn activate(&mut self) {
        if self.enabled {
            if let Some(callback) = &self.click_callback {
                callback();
            }
        }
    }
}

impl UIElement for ModernButton {
    fn get_id(&self) -> ElementId {
        self.id
    }

    fn get_bounds(&self) -> &UIBounds {
        &self.bounds
    }

    fn get_bounds_mut(&mut self) -> &mut UIBounds {
        &mut self.bounds
    }

    fn get_state(&self) -> UIState {
        self.state
    }

    fn set_state(&mut self, state: UIState) {
        if self.enabled {
            self.state = state;
        }
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    fn update(&mut self, delta_time: f32, input: &InputManager) {
        // Update animation progress for smooth transitions
        match self.state {
            UIState::Hovered | UIState::Pressed | UIState::Focused => {
                self.animation_progress = (self.animation_progress + delta_time * 8.0).min(1.0);
            }
            _ => {
                self.animation_progress = (self.animation_progress - delta_time * 8.0).max(0.0);
            }
        }

        // Handle keyboard input for accessibility
        if self.keyboard_focus && self.enabled {
            if input.is_named_key_just_pressed(winit::keyboard::NamedKey::Enter) ||
               input.is_named_key_just_pressed(winit::keyboard::NamedKey::Space) {
                self.activate();
            }
        }
    }

    fn handle_event(&mut self, event: &UIEvent) -> bool {
        match event {
            UIEvent::Click { element_id, .. } if *element_id == self.id => {
                if self.enabled {
                    self.activate();
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

/// Modern card component with glass morphism effects
pub struct ModernCard {
    id: ElementId,
    bounds: UIBounds,
    state: UIState,
    visible: bool,
    title: Option<String>,
    content: Vec<String>,
    style: StateStyle,
    accessibility: AccessibilityProps,
    is_interactive: bool,
    click_callback: Option<Box<dyn Fn() + Send + Sync>>,
}

impl ModernCard {
    pub fn new(id: ElementId, bounds: UIBounds) -> Self {
        Self {
            id,
            bounds,
            state: UIState::Normal,
            visible: true,
            title: None,
            content: Vec::new(),
            style: UITheme::modern_card(),
            accessibility: AccessibilityProps {
                role: "article".to_string(),
                ..AccessibilityProps::default()
            },
            is_interactive: false,
            click_callback: None,
        }
    }

    pub fn glass() -> Self {
        let mut card = Self::new(0, UIBounds::new(0.0, 0.0, 300.0, 200.0));
        card.style = UITheme::glass_card();
        card
    }

    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title.clone());
        self.accessibility.aria_label = Some(title);
        self
    }

    pub fn with_content(mut self, content: Vec<String>) -> Self {
        self.content = content;
        self
    }

    pub fn interactive(mut self) -> Self {
        self.is_interactive = true;
        self.accessibility.tab_index = 0;
        self.accessibility.role = "button".to_string();
        self
    }

    pub fn with_click_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.click_callback = Some(Box::new(callback));
        self.is_interactive = true;
        self
    }
}

impl UIElement for ModernCard {
    fn get_id(&self) -> ElementId {
        self.id
    }

    fn get_bounds(&self) -> &UIBounds {
        &self.bounds
    }

    fn get_bounds_mut(&mut self) -> &mut UIBounds {
        &mut self.bounds
    }

    fn get_state(&self) -> UIState {
        self.state
    }

    fn set_state(&mut self, state: UIState) {
        if self.is_interactive {
            self.state = state;
        }
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    fn update(&mut self, _delta_time: f32, _input: &InputManager) {
        // Cards are typically static, but can be made interactive
    }

    fn handle_event(&mut self, event: &UIEvent) -> bool {
        if !self.is_interactive {
            return false;
        }

        match event {
            UIEvent::Click { element_id, .. } if *element_id == self.id => {
                if let Some(callback) = &self.click_callback {
                    callback();
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

/// Modern input field with enhanced accessibility
pub struct ModernInput {
    id: ElementId,
    bounds: UIBounds,
    state: UIState,
    visible: bool,
    enabled: bool,
    value: String,
    placeholder: String,
    style: StateStyle,
    accessibility: AccessibilityProps,
    is_password: bool,
    cursor_position: usize,
    keyboard_focus: bool,
    validation_message: Option<String>,
    validation_state: ValidationState,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValidationState {
    Valid,
    Invalid,
    Warning,
    Neutral,
}

impl ModernInput {
    pub fn new(id: ElementId, bounds: UIBounds) -> Self {
        Self {
            id,
            bounds,
            state: UIState::Normal,
            visible: true,
            enabled: true,
            value: String::new(),
            placeholder: String::new(),
            style: UITheme::modern_input(),
            accessibility: AccessibilityProps {
                role: "textbox".to_string(),
                tab_index: 0,
                ..AccessibilityProps::default()
            },
            is_password: false,
            cursor_position: 0,
            keyboard_focus: false,
            validation_message: None,
            validation_state: ValidationState::Neutral,
        }
    }

    pub fn with_placeholder(mut self, placeholder: String) -> Self {
        self.placeholder = placeholder.clone();
        self.accessibility.aria_description = Some(format!("Input field with placeholder: {}", placeholder));
        self
    }

    pub fn password(mut self) -> Self {
        self.is_password = true;
        self.accessibility.aria_label = Some("Password input".to_string());
        self
    }

    pub fn with_validation(mut self, state: ValidationState, message: Option<String>) -> Self {
        self.validation_state = state;
        self.validation_message = message;

        // Update style based on validation state
        match state {
            ValidationState::Invalid => {
                self.style.normal.border.color = DesignSystem::ERROR;
                self.style.focused.border.color = DesignSystem::ERROR;
            }
            ValidationState::Warning => {
                self.style.normal.border.color = DesignSystem::WARNING;
                self.style.focused.border.color = DesignSystem::WARNING;
            }
            ValidationState::Valid => {
                self.style.normal.border.color = DesignSystem::SUCCESS;
                self.style.focused.border.color = DesignSystem::SUCCESS;
            }
            ValidationState::Neutral => {
                self.style = UITheme::modern_input();
            }
        }

        self
    }

    pub fn get_value(&self) -> &str {
        &self.value
    }

    pub fn set_value(&mut self, value: String) {
        self.value = value;
        self.cursor_position = self.value.len();
    }

    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor_position = 0;
    }

    pub fn is_keyboard_focusable(&self) -> bool {
        self.enabled && self.visible
    }

    pub fn set_keyboard_focus(&mut self, focused: bool) {
        self.keyboard_focus = focused;
        if focused {
            self.state = UIState::Focused;
        } else if self.state == UIState::Focused {
            self.state = UIState::Normal;
        }
    }
}

impl UIElement for ModernInput {
    fn get_id(&self) -> ElementId {
        self.id
    }

    fn get_bounds(&self) -> &UIBounds {
        &self.bounds
    }

    fn get_bounds_mut(&mut self) -> &mut UIBounds {
        &mut self.bounds
    }

    fn get_state(&self) -> UIState {
        self.state
    }

    fn set_state(&mut self, state: UIState) {
        if self.enabled {
            self.state = state;
        }
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    fn update(&mut self, _delta_time: f32, input: &InputManager) {
        if !self.keyboard_focus || !self.enabled {
            return;
        }

        // Handle text input (simplified - in a real implementation, you'd want proper text input handling)
        // This is a placeholder for demonstration purposes
        if input.is_named_key_just_pressed(winit::keyboard::NamedKey::Backspace) {
            if self.cursor_position > 0 {
                self.value.remove(self.cursor_position - 1);
                self.cursor_position -= 1;
            }
        }

        // Handle navigation keys
        if input.is_named_key_just_pressed(winit::keyboard::NamedKey::ArrowLeft) {
            self.cursor_position = self.cursor_position.saturating_sub(1);
        }
        if input.is_named_key_just_pressed(winit::keyboard::NamedKey::ArrowRight) {
            self.cursor_position = (self.cursor_position + 1).min(self.value.len());
        }
        if input.is_named_key_just_pressed(winit::keyboard::NamedKey::Home) {
            self.cursor_position = 0;
        }
        if input.is_named_key_just_pressed(winit::keyboard::NamedKey::End) {
            self.cursor_position = self.value.len();
        }
    }

    fn handle_event(&mut self, event: &UIEvent) -> bool {
        match event {
            UIEvent::Click { element_id, .. } if *element_id == self.id => {
                if self.enabled {
                    self.set_keyboard_focus(true);
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

/// Notification component for user feedback
pub struct ModernNotification {
    id: ElementId,
    bounds: UIBounds,
    state: UIState,
    visible: bool,
    message: String,
    notification_type: NotificationType,
    style: StateStyle,
    accessibility: AccessibilityProps,
    auto_hide_duration: Option<f32>,
    elapsed_time: f32,
    close_callback: Option<Box<dyn Fn() + Send + Sync>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NotificationType {
    Success,
    Warning,
    Error,
    Info,
}

impl ModernNotification {
    pub fn new(id: ElementId, bounds: UIBounds, message: String, notification_type: NotificationType) -> Self {
        let (style, role) = match notification_type {
            NotificationType::Success => (UITheme::success_notification(), "status"),
            NotificationType::Warning => (UITheme::warning_notification(), "alert"),
            NotificationType::Error => (UITheme::error_notification(), "alert"),
            NotificationType::Info => (UITheme::modern_card(), "status"),
        };

        Self {
            id,
            bounds,
            state: UIState::Normal,
            visible: true,
            message: message.clone(),
            notification_type,
            style,
            accessibility: AccessibilityProps {
                aria_label: Some(message),
                role: role.to_string(),
                ..AccessibilityProps::default()
            },
            auto_hide_duration: None,
            elapsed_time: 0.0,
            close_callback: None,
        }
    }

    pub fn success(message: String) -> Self {
        Self::new(0, UIBounds::new(0.0, 0.0, 400.0, 60.0), message, NotificationType::Success)
    }

    pub fn warning(message: String) -> Self {
        Self::new(0, UIBounds::new(0.0, 0.0, 400.0, 60.0), message, NotificationType::Warning)
    }

    pub fn error(message: String) -> Self {
        Self::new(0, UIBounds::new(0.0, 0.0, 400.0, 60.0), message, NotificationType::Error)
    }

    pub fn info(message: String) -> Self {
        Self::new(0, UIBounds::new(0.0, 0.0, 400.0, 60.0), message, NotificationType::Info)
    }

    pub fn with_auto_hide(mut self, duration_seconds: f32) -> Self {
        self.auto_hide_duration = Some(duration_seconds);
        self
    }

    pub fn with_close_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.close_callback = Some(Box::new(callback));
        self
    }

    pub fn close(&mut self) {
        if let Some(callback) = &self.close_callback {
            callback();
        }
        self.visible = false;
    }
}

impl UIElement for ModernNotification {
    fn get_id(&self) -> ElementId {
        self.id
    }

    fn get_bounds(&self) -> &UIBounds {
        &self.bounds
    }

    fn get_bounds_mut(&mut self) -> &mut UIBounds {
        &mut self.bounds
    }

    fn get_state(&self) -> UIState {
        self.state
    }

    fn set_state(&mut self, state: UIState) {
        self.state = state;
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    fn update(&mut self, delta_time: f32, _input: &InputManager) {
        if let Some(duration) = self.auto_hide_duration {
            self.elapsed_time += delta_time;
            if self.elapsed_time >= duration {
                self.close();
            }
        }
    }

    fn handle_event(&mut self, event: &UIEvent) -> bool {
        match event {
            UIEvent::Click { element_id, .. } if *element_id == self.id => {
                self.close();
                true
            }
            _ => false,
        }
    }
}