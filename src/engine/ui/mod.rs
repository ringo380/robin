use crate::engine::{
    math::Vec2,
    graphics::{Light, Sprite},
    animation::{AnimationManager, Animation, AnimationTarget, EaseType},
    input::InputManager,
};
use std::collections::HashMap;
use std::time::Duration;

pub mod layout;
pub mod components;
pub mod events;
pub mod styling;

pub use layout::*;
pub use components::*;
pub use events::*;
pub use styling::*;

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
    animation_manager: AnimationManager,
    screen_size: Vec2,
    mouse_position: Vec2,
    mouse_pressed: bool,
    ui_scale: f32,
}

impl UIManager {
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        Self {
            elements: HashMap::new(),
            next_id: 1,
            focused_element: None,
            hovered_element: None,
            animation_manager: AnimationManager::new(),
            screen_size: Vec2::new(screen_width, screen_height),
            mouse_position: Vec2::new(0.0, 0.0),
            mouse_pressed: false,
            ui_scale: 1.0,
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

        // Update hover states
        self.update_hover_states();

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
                position: self.mouse_position,
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
        self.animation_manager = AnimationManager::new();
        log::info!("Cleared all UI elements");
    }
}