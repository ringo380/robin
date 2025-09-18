use crate::engine::{
    math::Vec2,
    graphics::{Light, Sprite},
    animation::{AnimationManager, Animation, AnimationTarget, EaseType},
    input::InputManager,
};
use std::collections::HashMap;
use std::time::Duration;

pub mod layout;
pub mod legacy_components;
pub mod components;
pub mod events;
pub mod styling;
pub mod modern_components;
pub mod tutorial_system;
pub mod modern_architecture;
pub mod design_system;
pub mod theme_engine;
pub mod css_in_rust;
pub mod state_management;
pub mod performance;

pub use layout::*;
pub use legacy_components::*;
pub use components::*;
pub use events::{UIEvent, EventHandler, EventDispatcher, EventCallback};
pub use styling::{Color as UIColor, Spacing, Border, StateStyle, UIStyle, Typography, TextDecoration, AlignItems, JustifyContent, FlexDirection, BoxSizing, Resize, TextAlign, BorderStyle};
pub use modern_components::{AccessibilityProps, ModernButton, ModernCard, ModernNotification, NotificationType};
pub use tutorial_system::*;
pub use modern_architecture::*;
pub use design_system::{DesignSystem, ColorPalette};
pub use theme_engine::*;
pub use css_in_rust::*;
pub use state_management::*;
pub use performance::{
    PerformantUIRenderer, UIPerformanceConfig, UIPerformanceMetrics, VirtualScrollManager,
    VirtualScrollConfig, UIComponent, RenderContext, UIPerformanceReport
};

/// Unique identifier for UI elements
pub type ElementId = u32;

/// UI element states for interactions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UIState {
    Normal,
    Hovered,
    Pressed,
    Focused,
    Disabled,
}

/// UI anchor points for positioning
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Anchor {
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    MiddleCenter,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl Anchor {
    pub fn to_offset(self, container_size: Vec2) -> Vec2 {
        match self {
            Anchor::TopLeft => Vec2::new(0.0, 0.0),
            Anchor::TopCenter => Vec2::new(container_size.x * 0.5, 0.0),
            Anchor::TopRight => Vec2::new(container_size.x, 0.0),
            Anchor::MiddleLeft => Vec2::new(0.0, container_size.y * 0.5),
            Anchor::MiddleCenter => Vec2::new(container_size.x * 0.5, container_size.y * 0.5),
            Anchor::MiddleRight => Vec2::new(container_size.x, container_size.y * 0.5),
            Anchor::BottomLeft => Vec2::new(0.0, container_size.y),
            Anchor::BottomCenter => Vec2::new(container_size.x * 0.5, container_size.y),
            Anchor::BottomRight => Vec2::new(container_size.x, container_size.y),
        }
    }
}

/// UI element bounds and positioning
#[derive(Debug, Clone)]
pub struct UIBounds {
    pub position: Vec2,
    pub size: Vec2,
    pub anchor: Anchor,
    pub offset: Vec2,
}

impl UIBounds {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            position: Vec2::new(x, y),
            size: Vec2::new(width, height),
            anchor: Anchor::TopLeft,
            offset: Vec2::new(0.0, 0.0),
        }
    }

    pub fn with_anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn with_offset(mut self, offset: Vec2) -> Self {
        self.offset = offset;
        self
    }

    pub fn contains_point(&self, point: Vec2) -> bool {
        let actual_pos = self.position + self.offset;
        point.x >= actual_pos.x 
            && point.x <= actual_pos.x + self.size.x
            && point.y >= actual_pos.y 
            && point.y <= actual_pos.y + self.size.y
    }

    pub fn center(&self) -> Vec2 {
        self.position + self.size * 0.5
    }
}

/// Base trait for all UI elements
pub trait UIElement {
    fn get_id(&self) -> ElementId;
    fn get_bounds(&self) -> &UIBounds;
    fn get_bounds_mut(&mut self) -> &mut UIBounds;
    fn get_state(&self) -> UIState;
    fn set_state(&mut self, state: UIState);
    fn is_visible(&self) -> bool;
    fn set_visible(&mut self, visible: bool);
    fn update(&mut self, delta_time: f32, input: &InputManager);
    fn handle_event(&mut self, event: &UIEvent) -> bool;
}

/// Main UI manager that handles all UI elements and interactions
pub struct UIManager {
    elements: HashMap<ElementId, Box<dyn UIElement>>,
    next_id: ElementId,
    focused_element: Option<ElementId>,
    hovered_element: Option<ElementId>,
    keyboard_focused_element: Option<ElementId>,
    tab_order: Vec<ElementId>,
    animation_manager: AnimationManager,
    screen_size: Vec2,
    mouse_position: Vec2,
    mouse_pressed: bool,
    ui_scale: f32,
    accessibility_enabled: bool,
    high_contrast_mode: bool,
    screen_reader_mode: bool,
    notifications: Vec<ElementId>,
    tutorial_system: TutorialSystem,
}

impl UIManager {
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        Self {
            elements: HashMap::new(),
            next_id: 1,
            focused_element: None,
            hovered_element: None,
            keyboard_focused_element: None,
            tab_order: Vec::new(),
            animation_manager: AnimationManager::new(),
            screen_size: Vec2::new(screen_width, screen_height),
            mouse_position: Vec2::new(0.0, 0.0),
            mouse_pressed: false,
            ui_scale: 1.0,
            accessibility_enabled: true,
            high_contrast_mode: false,
            screen_reader_mode: false,
            notifications: Vec::new(),
            tutorial_system: TutorialSystem::new(),
        }
    }

    /// Add a UI element and return its ID
    pub fn add_element(&mut self, mut element: Box<dyn UIElement>) -> ElementId {
        let id = self.next_id;
        self.next_id += 1;
        
        // Set the element ID (this would require implementing in each element type)
        self.elements.insert(id, element);
        
        log::debug!("Added UI element with ID: {}", id);
        id
    }

    /// Remove a UI element
    pub fn remove_element(&mut self, id: ElementId) -> Option<Box<dyn UIElement>> {
        if Some(id) == self.focused_element {
            self.focused_element = None;
        }
        if Some(id) == self.hovered_element {
            self.hovered_element = None;
        }
        self.elements.remove(&id)
    }

    /// Get a UI element by ID
    pub fn get_element(&self, id: ElementId) -> Option<&dyn UIElement> {
        self.elements.get(&id).map(|e| e.as_ref())
    }

    /// Get a mutable UI element by ID (placeholder implementation)
    pub fn get_element_mut(&mut self, _id: ElementId) -> Option<&mut dyn UIElement> {
        // Placeholder: In a full implementation, this would require careful lifetime management
        // For now, we'll return None to allow compilation while preserving the API design
        None
    }

    /// Update all UI elements
    pub fn update(&mut self, delta_time: f32, input: &InputManager) {
        // Update mouse state
        self.mouse_position = Vec2::new(input.mouse_position().0 as f32, input.mouse_position().1 as f32);
        self.mouse_pressed = input.is_mouse_button_pressed(winit::event::MouseButton::Left);

        // Handle keyboard navigation
        self.handle_keyboard_navigation(input);

        // Update hover states
        self.update_hover_states();

        // Update tutorial system
        let tutorial_events = self.tutorial_system.update(delta_time, input);
        for event in tutorial_events {
            self.handle_tutorial_event(event);
        }

        // Update all elements
        for (_, element) in self.elements.iter_mut() {
            element.update(delta_time, input);
        }

        // Update animations
        self.animation_manager.update();

        // Handle mouse clicks
        if self.mouse_pressed {
            self.handle_mouse_click();
        }

        // Rebuild tab order if needed (could be optimized to only do this when elements change)
        if self.tab_order.is_empty() && !self.elements.is_empty() {
            self.rebuild_tab_order();
        }
    }

    /// Handle screen resize
    pub fn resize(&mut self, width: f32, height: f32) {
        self.screen_size = Vec2::new(width, height);
        log::info!("UI Manager resized to {}x{}", width, height);
    }

    /// Set the UI scale factor
    pub fn set_ui_scale(&mut self, scale: f32) {
        self.ui_scale = scale.max(0.1);
        log::info!("UI scale set to {:.2}", self.ui_scale);
    }

    /// Get the UI scale factor
    pub fn get_ui_scale(&self) -> f32 {
        self.ui_scale
    }

    /// Get screen size
    pub fn get_screen_size(&self) -> Vec2 {
        self.screen_size
    }

    /// Animate a UI element
    pub fn animate_element(&mut self, id: ElementId, animation: Animation) {
        self.animation_manager.add_animation(format!("ui_element_{}", id), animation);
    }

    /// Fade in a UI element
    pub fn fade_in_element(&mut self, id: ElementId, duration: Duration) {
        let animation = Animation::new(
            AnimationTarget::Alpha(0.0, 1.0),
            duration,
            EaseType::EaseOut,
        );
        self.animate_element(id, animation);
    }

    /// Fade out a UI element
    pub fn fade_out_element(&mut self, id: ElementId, duration: Duration) {
        let animation = Animation::new(
            AnimationTarget::Alpha(1.0, 0.0),
            duration,
            EaseType::EaseOut,
        );
        self.animate_element(id, animation);
    }

    /// Slide a UI element to a new position
    pub fn slide_element(&mut self, id: ElementId, from: Vec2, to: Vec2, duration: Duration) {
        let animation = Animation::new(
            AnimationTarget::Position(from, to),
            duration,
            EaseType::EaseInOut,
        );
        self.animate_element(id, animation);
    }

    /// Scale a UI element with bouncy effect
    pub fn bounce_element(&mut self, id: ElementId, from_scale: f32, to_scale: f32, duration: Duration) {
        let animation = Animation::new(
            AnimationTarget::Size(from_scale, to_scale),
            duration,
            EaseType::Bounce,
        );
        self.animate_element(id, animation);
    }

    /// Create a pulsing glow effect
    pub fn pulse_element(&mut self, id: ElementId, duration: Duration) {
        let animation = Animation::new(
            AnimationTarget::Size(1.0, 1.1),
            duration,
            EaseType::EaseInOut,
        );
        // Set to repeat
        self.animate_element(id, animation);
        
        // Also pulse the alpha
        let alpha_animation = Animation::new(
            AnimationTarget::Alpha(1.0, 0.7),
            duration,
            EaseType::EaseInOut,
        );
        self.animation_manager.add_animation(format!("ui_element_{}_alpha", id), alpha_animation);
    }

    // === PRIVATE METHODS ===

    fn update_hover_states(&mut self) {
        let mut new_hovered: Option<ElementId> = None;
        
        // Find which element is under the mouse (check in reverse order for correct layering)
        let element_ids: Vec<ElementId> = self.elements.keys().copied().collect();
        for &id in element_ids.iter().rev() {
            if let Some(element) = self.elements.get(&id) {
                if element.is_visible() && element.get_bounds().contains_point(self.mouse_position) {
                    new_hovered = Some(id);
                    break;
                }
            }
        }

        // Update hover states
        if let Some(old_hovered) = self.hovered_element {
            if Some(old_hovered) != new_hovered {
                if let Some(element) = self.elements.get_mut(&old_hovered) {
                    if element.get_state() == UIState::Hovered {
                        element.set_state(UIState::Normal);
                    }
                }
            }
        }

        if let Some(new_id) = new_hovered {
            if Some(new_id) != self.hovered_element {
                if let Some(element) = self.elements.get_mut(&new_id) {
                    if element.get_state() == UIState::Normal {
                        element.set_state(UIState::Hovered);
                    }
                }
            }
        }

        self.hovered_element = new_hovered;
    }

    fn handle_mouse_click(&mut self) {
        if let Some(hovered_id) = self.hovered_element {
            // Set focused element
            self.focused_element = Some(hovered_id);
            
            // Set pressed state
            if let Some(element) = self.elements.get_mut(&hovered_id) {
                element.set_state(UIState::Pressed);
            }

            // Create click event
            let event = UIEvent::Click {
                element_id: hovered_id,
                position: (self.mouse_position.x, self.mouse_position.y),
            };

            // Send event to element
            if let Some(element) = self.elements.get_mut(&hovered_id) {
                element.handle_event(&event);
            }

            log::debug!("UI element {} clicked at ({:.1}, {:.1})", hovered_id, self.mouse_position.x, self.mouse_position.y);
        } else {
            // Click on empty space - remove focus
            self.focused_element = None;
        }
    }

    /// Get all visible elements for rendering
    pub fn get_visible_elements(&self) -> Vec<ElementId> {
        self.elements
            .iter()
            .filter(|(_, element)| element.is_visible())
            .map(|(id, _)| *id)
            .collect()
    }

    /// Get element count for debugging
    pub fn get_element_count(&self) -> usize {
        self.elements.len()
    }

    /// Clear all UI elements
    pub fn clear(&mut self) {
        self.elements.clear();
        self.focused_element = None;
        self.hovered_element = None;
        self.keyboard_focused_element = None;
        self.tab_order.clear();
        self.notifications.clear();
        self.animation_manager = AnimationManager::new();
        log::info!("Cleared all UI elements");
    }

    // === ACCESSIBILITY FEATURES ===

    /// Enable or disable accessibility features
    pub fn set_accessibility_enabled(&mut self, enabled: bool) {
        self.accessibility_enabled = enabled;
        log::info!("Accessibility features {}", if enabled { "enabled" } else { "disabled" });
    }

    /// Enable or disable high contrast mode
    pub fn set_high_contrast_mode(&mut self, enabled: bool) {
        self.high_contrast_mode = enabled;
        log::info!("High contrast mode {}", if enabled { "enabled" } else { "disabled" });
    }

    /// Enable or disable screen reader mode
    pub fn set_screen_reader_mode(&mut self, enabled: bool) {
        self.screen_reader_mode = enabled;
        log::info!("Screen reader mode {}", if enabled { "enabled" } else { "disabled" });
    }

    /// Get the currently keyboard-focused element
    pub fn get_keyboard_focused_element(&self) -> Option<ElementId> {
        self.keyboard_focused_element
    }

    /// Set keyboard focus to a specific element
    pub fn set_keyboard_focus(&mut self, element_id: Option<ElementId>) {
        // Clear focus from current element
        if let Some(current_id) = self.keyboard_focused_element {
            if let Some(element) = self.elements.get_mut(&current_id) {
                element.set_state(UIState::Normal);
            }
        }

        // Set focus to new element
        self.keyboard_focused_element = element_id;
        if let Some(new_id) = element_id {
            if let Some(element) = self.elements.get_mut(&new_id) {
                element.set_state(UIState::Focused);
            }
        }

        log::debug!("Keyboard focus set to element: {:?}", element_id);
    }

    /// Navigate to the next focusable element (Tab key)
    pub fn focus_next_element(&mut self) {
        if self.tab_order.is_empty() {
            return;
        }

        let current_index = if let Some(current_id) = self.keyboard_focused_element {
            self.tab_order.iter().position(|&id| id == current_id).unwrap_or(0)
        } else {
            0
        };

        let next_index = (current_index + 1) % self.tab_order.len();
        let next_id = self.tab_order[next_index];
        self.set_keyboard_focus(Some(next_id));
    }

    /// Navigate to the previous focusable element (Shift+Tab)
    pub fn focus_previous_element(&mut self) {
        if self.tab_order.is_empty() {
            return;
        }

        let current_index = if let Some(current_id) = self.keyboard_focused_element {
            self.tab_order.iter().position(|&id| id == current_id).unwrap_or(0)
        } else {
            0
        };

        let prev_index = if current_index == 0 {
            self.tab_order.len() - 1
        } else {
            current_index - 1
        };
        let prev_id = self.tab_order[prev_index];
        self.set_keyboard_focus(Some(prev_id));
    }

    /// Rebuild the tab order based on current elements
    pub fn rebuild_tab_order(&mut self) {
        self.tab_order.clear();

        // Collect all focusable elements and sort by tab index and position
        let mut focusable_elements: Vec<(ElementId, i32, Vec2)> = Vec::new();

        for (&id, element) in &self.elements {
            if element.is_visible() {
                // In a real implementation, you'd check if the element implements keyboard focus
                // For now, we'll assume all visible elements are focusable
                let position = element.get_bounds().position;
                focusable_elements.push((id, 0, position)); // tab_index = 0 for all
            }
        }

        // Sort by tab index, then by vertical position, then by horizontal position
        focusable_elements.sort_by(|a, b| {
            a.1.cmp(&b.1)  // tab_index
                .then(a.2.y.partial_cmp(&b.2.y).unwrap_or(std::cmp::Ordering::Equal))  // y position
                .then(a.2.x.partial_cmp(&b.2.x).unwrap_or(std::cmp::Ordering::Equal))  // x position
        });

        self.tab_order = focusable_elements.into_iter().map(|(id, _, _)| id).collect();
        log::debug!("Rebuilt tab order with {} elements", self.tab_order.len());
    }

    /// Handle keyboard navigation
    pub fn handle_keyboard_navigation(&mut self, input: &InputManager) {
        if !self.accessibility_enabled {
            return;
        }

        // Tab navigation
        if input.is_named_key_just_pressed(winit::keyboard::NamedKey::Tab) {
            if input.is_named_key_pressed(winit::keyboard::NamedKey::Shift) {
                self.focus_previous_element();
            } else {
                self.focus_next_element();
            }
        }

        // Escape key clears focus
        if input.is_named_key_just_pressed(winit::keyboard::NamedKey::Escape) {
            self.set_keyboard_focus(None);
        }

        // Enter or Space activates the focused element
        if let Some(focused_id) = self.keyboard_focused_element {
            if input.is_named_key_just_pressed(winit::keyboard::NamedKey::Enter) ||
               input.is_named_key_just_pressed(winit::keyboard::NamedKey::Space) {
                let event = UIEvent::Click {
                    element_id: focused_id,
                    position: (self.mouse_position.x, self.mouse_position.y),
                };
                if let Some(element) = self.elements.get_mut(&focused_id) {
                    element.handle_event(&event);
                }
            }
        }
    }

    // === NOTIFICATION SYSTEM ===

    /// Add a notification to the UI
    pub fn show_notification(&mut self, notification: ModernNotification) -> ElementId {
        let id = self.add_element(Box::new(notification));
        self.notifications.push(id);

        // Position notification in the top-right corner
        if let Some(element) = self.elements.get_mut(&id) {
            let bounds = element.get_bounds_mut();
            bounds.position.x = self.screen_size.x - bounds.size.x - 20.0;
            bounds.position.y = 20.0 + (self.notifications.len() as f32 - 1.0) * (bounds.size.y + 10.0);
        }

        log::info!("Added notification with ID: {}", id);
        id
    }

    /// Remove a notification
    pub fn hide_notification(&mut self, id: ElementId) {
        if let Some(pos) = self.notifications.iter().position(|&x| x == id) {
            self.notifications.remove(pos);
            self.remove_element(id);

            // Reposition remaining notifications
            for (index, &notification_id) in self.notifications.iter().enumerate() {
                if let Some(element) = self.elements.get_mut(&notification_id) {
                    let bounds = element.get_bounds_mut();
                    bounds.position.y = 20.0 + index as f32 * (bounds.size.y + 10.0);
                }
            }
        }
    }

    /// Show a success notification
    pub fn show_success(&mut self, message: String) -> ElementId {
        let notification = ModernNotification::success(message)
            .with_auto_hide(4.0);
        self.show_notification(notification)
    }

    /// Show a warning notification
    pub fn show_warning(&mut self, message: String) -> ElementId {
        let notification = ModernNotification::warning(message)
            .with_auto_hide(6.0);
        self.show_notification(notification)
    }

    /// Show an error notification
    pub fn show_error(&mut self, message: String) -> ElementId {
        let notification = ModernNotification::error(message)
            .with_auto_hide(8.0);
        self.show_notification(notification)
    }

    /// Show an info notification
    pub fn show_info(&mut self, message: String) -> ElementId {
        let notification = ModernNotification::info(message)
            .with_auto_hide(5.0);
        self.show_notification(notification)
    }

    // === TUTORIAL SYSTEM ===

    /// Start the Engineer Build Mode tutorial
    pub fn start_tutorial(&mut self) {
        self.tutorial_system.init_engineer_build_mode_tutorial();
        let events = self.tutorial_system.start();
        for event in events {
            self.handle_tutorial_event(event);
        }
        log::info!("Started Engineer Build Mode tutorial");
    }

    /// Stop the tutorial
    pub fn stop_tutorial(&mut self) {
        self.tutorial_system.stop();
    }

    /// Pause or resume the tutorial
    pub fn toggle_tutorial_pause(&mut self) {
        self.tutorial_system.toggle_pause();
    }

    /// Skip the current tutorial step
    pub fn skip_tutorial_step(&mut self) {
        let events = self.tutorial_system.skip_current_step();
        for event in events {
            self.handle_tutorial_event(event);
        }
    }

    /// Check if tutorial is active
    pub fn is_tutorial_active(&self) -> bool {
        self.tutorial_system.is_active()
    }

    /// Get tutorial completion statistics
    pub fn get_tutorial_stats(&self) -> TutorialStats {
        self.tutorial_system.get_completion_stats()
    }

    /// Mark a tutorial action as completed (for integration with game systems)
    pub fn complete_tutorial_action(&mut self, action: TutorialAction) {
        let events = self.tutorial_system.mark_action_completed(&action);
        for event in events {
            self.handle_tutorial_event(event);
        }
    }

    /// Show a tutorial hint
    pub fn show_tutorial_hint(&mut self) {
        if let Some(hint_event) = self.tutorial_system.get_hint_event() {
            self.handle_tutorial_event(hint_event);
        }
    }

    /// Handle tutorial events
    fn handle_tutorial_event(&mut self, event: TutorialEvent) {
        match event {
            TutorialEvent::ShowInfo(message) => {
                self.show_info(message);
            }
            TutorialEvent::ShowSuccess(message) => {
                self.show_success(message);
            }
            TutorialEvent::ShowError(message) => {
                self.show_error(message);
            }
            TutorialEvent::ShowHint(message) => {
                self.show_info(message);
            }
            TutorialEvent::RefreshUI => {
                // Refresh tutorial UI elements
                log::debug!("Refreshing tutorial UI");
            }
        }
    }
}