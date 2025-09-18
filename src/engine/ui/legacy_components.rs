use crate::engine::{
    math::Vec2,
    ui::{UIBounds, UIState, events::UIEvent, UIElement, ElementId, EventHandler, styling::{StateStyle, Color, UIStyle}},
    input::InputManager,
};
use std::collections::HashMap;

/// Button component with click handling
pub struct Button {
    id: ElementId,
    bounds: UIBounds,
    state: UIState,
    visible: bool,
    text: String,
    style: StateStyle,
    click_callback: Option<Box<dyn Fn() + Send + Sync>>,
    enabled: bool,
}

impl Button {
    pub fn new(id: ElementId, bounds: UIBounds, text: String) -> Self {
        Self {
            id,
            bounds,
            state: UIState::Normal,
            visible: true,
            text,
            style: crate::engine::ui::styling::UITheme::modern_button(),
            click_callback: None,
            enabled: true,
        }
    }

    pub fn with_style(mut self, style: StateStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_click_callback<F>(mut self, callback: F) -> Self 
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.click_callback = Some(Box::new(callback));
        self
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
    }

    pub fn get_text(&self) -> &str {
        &self.text
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.state = UIState::Disabled;
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn get_current_style(&self) -> &UIStyle {
        self.style.get_style(self.state)
    }
}

impl UIElement for Button {
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
        if self.enabled || state == UIState::Disabled {
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
        // Button-specific update logic can go here
    }

    fn handle_event(&mut self, event: &UIEvent) -> bool {
        match event {
            UIEvent::Click { element_id, .. } if *element_id == self.id => {
                if self.enabled {
                    if let Some(callback) = &self.click_callback {
                        callback();
                    }
                    log::info!("Button '{}' clicked", self.text);
                }
                true
            }
            _ => false,
        }
    }
}

/// Text label component
pub struct Label {
    id: ElementId,
    bounds: UIBounds,
    state: UIState,
    visible: bool,
    text: String,
    style: UIStyle,
    selectable: bool,
}

impl Label {
    pub fn new(id: ElementId, bounds: UIBounds, text: String) -> Self {
        Self {
            id,
            bounds,
            state: UIState::Normal,
            visible: true,
            text,
            style: UIStyle::default(),
            selectable: false,
        }
    }

    pub fn with_style(mut self, style: UIStyle) -> Self {
        self.style = style;
        self
    }

    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
    }

    pub fn get_text(&self) -> &str {
        &self.text
    }

    pub fn get_style(&self) -> &UIStyle {
        &self.style
    }
}

impl UIElement for Label {
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
        if self.selectable {
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
        // Label update logic
    }

    fn handle_event(&mut self, _event: &UIEvent) -> bool {
        false // Labels don't handle events by default
    }
}

/// Panel/Container component
pub struct Panel {
    id: ElementId,
    bounds: UIBounds,
    state: UIState,
    visible: bool,
    style: UIStyle,
    children: Vec<ElementId>,
    scrollable: bool,
    scroll_offset: Vec2,
}

impl Panel {
    pub fn new(id: ElementId, bounds: UIBounds) -> Self {
        Self {
            id,
            bounds: bounds.clone(),
            state: UIState::Normal,
            visible: true,
            style: crate::engine::ui::styling::UITheme::card(),
            children: Vec::new(),
            scrollable: false,
            scroll_offset: Vec2::new(0.0, 0.0),
        }
    }

    pub fn with_style(mut self, style: UIStyle) -> Self {
        self.style = style;
        self
    }

    pub fn scrollable(mut self, scrollable: bool) -> Self {
        self.scrollable = scrollable;
        self
    }

    pub fn add_child(&mut self, child_id: ElementId) {
        if !self.children.contains(&child_id) {
            self.children.push(child_id);
        }
    }

    pub fn remove_child(&mut self, child_id: ElementId) {
        self.children.retain(|&id| id != child_id);
    }

    pub fn get_children(&self) -> &[ElementId] {
        &self.children
    }

    pub fn get_style(&self) -> &UIStyle {
        &self.style
    }

    pub fn set_scroll_offset(&mut self, offset: Vec2) {
        if self.scrollable {
            self.scroll_offset = offset;
        }
    }

    pub fn get_scroll_offset(&self) -> Vec2 {
        self.scroll_offset
    }
}

impl UIElement for Panel {
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

    fn update(&mut self, _delta_time: f32, _input: &InputManager) {
        // Panel update logic
    }

    fn handle_event(&mut self, _event: &UIEvent) -> bool {
        false // Panels don't handle events by default
    }
}

/// Slider component for numeric input
pub struct Slider {
    id: ElementId,
    bounds: UIBounds,
    state: UIState,
    visible: bool,
    value: f32,
    min_value: f32,
    max_value: f32,
    step: f32,
    style: StateStyle,
    dragging: bool,
    value_changed_callback: Option<Box<dyn Fn(f32) + Send + Sync>>,
}

impl Slider {
    pub fn new(id: ElementId, bounds: UIBounds, min_value: f32, max_value: f32) -> Self {
        Self {
            id,
            bounds: bounds.clone(),
            state: UIState::Normal,
            visible: true,
            value: min_value,
            min_value,
            max_value,
            step: 0.01,
            style: Self::default_style(),
            dragging: false,
            value_changed_callback: None,
        }
    }

    pub fn with_value(mut self, value: f32) -> Self {
        self.value = value.clamp(self.min_value, self.max_value);
        self
    }

    pub fn with_step(mut self, step: f32) -> Self {
        self.step = step;
        self
    }

    pub fn with_value_changed_callback<F>(mut self, callback: F) -> Self 
    where
        F: Fn(f32) + Send + Sync + 'static,
    {
        self.value_changed_callback = Some(Box::new(callback));
        self
    }

    pub fn get_value(&self) -> f32 {
        self.value
    }

    pub fn set_value(&mut self, value: f32) {
        let new_value = value.clamp(self.min_value, self.max_value);
        if (new_value - self.value).abs() > f32::EPSILON {
            self.value = new_value;
            if let Some(callback) = &self.value_changed_callback {
                callback(self.value);
            }
        }
    }

    pub fn get_normalized_value(&self) -> f32 {
        if self.max_value == self.min_value {
            0.0
        } else {
            (self.value - self.min_value) / (self.max_value - self.min_value)
        }
    }

    fn default_style() -> StateStyle {
        let mut base = UIStyle {
            background_color: Color::LIGHT_GRAY,
            border: crate::engine::ui::styling::Border::new(1.0, Color::GRAY).with_radius(4.0),
            ..UIStyle::default()
        };

        let mut style = StateStyle::new(base);
        style.hovered.background_color = Color::from_hex(0xE0E0E0);
        style.pressed.background_color = Color::from_hex(0xD0D0D0);
        style.disabled.background_color = Color::GRAY;

        style
    }

    pub fn get_current_style(&self) -> &UIStyle {
        self.style.get_style(self.state)
    }
}

impl UIElement for Slider {
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

    fn update(&mut self, _delta_time: f32, input: &InputManager) {
        if self.dragging && input.is_mouse_button_pressed(winit::event::MouseButton::Left) {
            let mouse_pos = Vec2::new(input.mouse_position().0 as f32, input.mouse_position().1 as f32);
            
            // Calculate new value based on mouse position
            let relative_x = mouse_pos.x - self.bounds.position.x;
            let normalized = (relative_x / self.bounds.size.x).clamp(0.0, 1.0);
            let new_value = self.min_value + normalized * (self.max_value - self.min_value);
            
            // Snap to step
            let stepped_value = if self.step > 0.0 {
                (new_value / self.step).round() * self.step
            } else {
                new_value
            };
            
            self.set_value(stepped_value);
        } else {
            self.dragging = false;
        }
    }

    fn handle_event(&mut self, event: &UIEvent) -> bool {
        match event {
            UIEvent::Click { element_id, position } if *element_id == self.id => {
                self.dragging = true;
                
                // Calculate new value based on click position
                let relative_x = position.0 - self.bounds.position.x;
                let normalized = (relative_x / self.bounds.size.x).clamp(0.0, 1.0);
                let new_value = self.min_value + normalized * (self.max_value - self.min_value);
                
                // Snap to step
                let stepped_value = if self.step > 0.0 {
                    (new_value / self.step).round() * self.step
                } else {
                    new_value
                };
                
                self.set_value(stepped_value);
                true
            }
            _ => false,
        }
    }
}

/// Progress bar component
pub struct ProgressBar {
    id: ElementId,
    bounds: UIBounds,
    state: UIState,
    visible: bool,
    value: f32,
    max_value: f32,
    style: UIStyle,
    fill_color: Color,
    animated: bool,
    animation_speed: f32,
}

impl ProgressBar {
    pub fn new(id: ElementId, bounds: UIBounds, max_value: f32) -> Self {
        Self {
            id,
            bounds: bounds.clone(),
            state: UIState::Normal,
            visible: true,
            value: 0.0,
            max_value,
            style: Self::default_style(),
            fill_color: Color::from_hex(0x4CAF50), // Green
            animated: true,
            animation_speed: 1.0,
        }
    }

    pub fn with_value(mut self, value: f32) -> Self {
        self.value = value.clamp(0.0, self.max_value);
        self
    }

    pub fn with_fill_color(mut self, color: Color) -> Self {
        self.fill_color = color;
        self
    }

    pub fn animated(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }

    pub fn get_value(&self) -> f32 {
        self.value
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = value.clamp(0.0, self.max_value);
    }

    pub fn get_progress(&self) -> f32 {
        if self.max_value == 0.0 {
            0.0
        } else {
            self.value / self.max_value
        }
    }

    fn default_style() -> UIStyle {
        UIStyle {
            background_color: Color::LIGHT_GRAY,
            border: crate::engine::ui::styling::Border::new(1.0, Color::GRAY).with_radius(4.0),
            ..UIStyle::default()
        }
    }

    pub fn get_style(&self) -> &UIStyle {
        &self.style
    }

    pub fn get_fill_color(&self) -> Color {
        self.fill_color
    }
}

impl UIElement for ProgressBar {
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
        // Progress bar animations could go here
        if self.animated {
            // Add any animated effects
        }
    }

    fn handle_event(&mut self, _event: &UIEvent) -> bool {
        false // Progress bars don't handle events by default
    }
}