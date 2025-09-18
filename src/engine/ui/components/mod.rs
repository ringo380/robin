// Robin Game Engine - Modern UI Component Library
// Phase 3: Production-ready UI components with modern architecture

use crate::engine::error::RobinResult;
use crate::engine::generation::templates::UITheme;
use super::{
    design_system::DesignSystem,
    css_in_rust::{Style, BoxSpacing, Dimension},
    state_management::*,
    modern_architecture::{Component, ComponentId, RenderContext, RenderOutput, Theme}
};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

// Re-export all component modules
pub mod button;
pub mod form;
pub mod modal;
pub mod navigation;
pub mod data_display;
pub mod feedback;
pub mod layout;

pub use button::*;
pub use form::*;
pub use modal::*;
pub use navigation::*;
pub use data_display::*;
pub use feedback::*;
pub use layout::*;

/// Core trait for all UI components
pub trait UIComponent: Component {
    /// Component type identifier
    fn component_type(&self) -> &'static str;

    /// Get component props
    fn props(&self) -> &ComponentProps;

    /// Set component props
    fn set_props(&mut self, props: ComponentProps) -> RobinResult<()>;

    /// Handle events
    fn handle_event(&mut self, event: UIEvent) -> RobinResult<Vec<UIEvent>>;

    /// Check if component is interactive
    fn is_interactive(&self) -> bool { false }

    /// Get accessibility information
    fn accessibility_info(&self) -> AccessibilityInfo;

    /// Apply theme to component
    fn apply_theme(&mut self, theme: &UITheme) -> RobinResult<()>;
}

/// Base properties for all components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentProps {
    pub id: String,
    pub class_name: Option<String>,
    pub style: Option<Style>,
    pub disabled: bool,
    pub visible: bool,
    pub accessibility: AccessibilityProps,
    pub data_attributes: HashMap<String, String>,
}

impl Default for ComponentProps {
    fn default() -> Self {
        Self {
            id: String::new(),
            class_name: None,
            style: None,
            disabled: false,
            visible: true,
            accessibility: AccessibilityProps::default(),
            data_attributes: HashMap::new(),
        }
    }
}

/// Accessibility properties for components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityProps {
    pub aria_label: Option<String>,
    pub aria_describedby: Option<String>,
    pub aria_labelledby: Option<String>,
    pub aria_hidden: bool,
    pub aria_expanded: Option<bool>,
    pub aria_pressed: Option<bool>,
    pub aria_checked: Option<bool>,
    pub role: Option<String>,
    pub tabindex: Option<i32>,
}

impl Default for AccessibilityProps {
    fn default() -> Self {
        Self {
            aria_label: None,
            aria_describedby: None,
            aria_labelledby: None,
            aria_hidden: false,
            aria_expanded: None,
            aria_pressed: None,
            aria_checked: None,
            role: None,
            tabindex: None,
        }
    }
}

/// Accessibility information for screen readers
#[derive(Debug, Clone)]
pub struct AccessibilityInfo {
    pub label: String,
    pub description: Option<String>,
    pub role: String,
    pub states: Vec<String>,
    pub actions: Vec<String>,
}

impl Default for AccessibilityInfo {
    fn default() -> Self {
        Self {
            label: String::new(),
            description: None,
            role: "generic".to_string(),
            states: Vec::new(),
            actions: Vec::new(),
        }
    }
}

/// UI events that components can emit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UIEvent {
    Click { component_id: String, position: (f32, f32) },
    Hover { component_id: String, entered: bool },
    Focus { component_id: String, focused: bool },
    Input { component_id: String, value: String },
    Change { component_id: String, value: String },
    Submit { component_id: String, data: HashMap<String, String> },
    KeyDown { component_id: String, key: String, modifiers: KeyModifiers },
    KeyUp { component_id: String, key: String, modifiers: KeyModifiers },
    Resize { component_id: String, width: f32, height: f32 },
    Custom { event_type: String, component_id: String, data: HashMap<String, String> },
}

/// Keyboard modifiers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyModifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub meta: bool,
}

impl Default for KeyModifiers {
    fn default() -> Self {
        Self {
            ctrl: false,
            shift: false,
            alt: false,
            meta: false,
        }
    }
}

/// Component size variants
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Size {
    XSmall,
    Small,
    Medium,
    Large,
    XLarge,
}

impl Default for Size {
    fn default() -> Self {
        Size::Medium
    }
}

/// Component color variants
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ColorVariant {
    Primary,
    Secondary,
    Success,
    Warning,
    Error,
    Info,
    Neutral,
}

impl Default for ColorVariant {
    fn default() -> Self {
        ColorVariant::Primary
    }
}

/// Animation states for components
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AnimationState {
    Idle,
    Hover,
    Active,
    Focus,
    Disabled,
    Loading,
}

impl Default for AnimationState {
    fn default() -> Self {
        AnimationState::Idle
    }
}

/// Component registry for managing component instances
pub struct ComponentRegistry {
    components: HashMap<ComponentId, Box<dyn UIComponent>>,
    design_system: DesignSystem,
    event_handlers: HashMap<String, Box<dyn Fn(&UIEvent) -> RobinResult<()>>>,
}

impl std::fmt::Debug for ComponentRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComponentRegistry")
            .field("components", &format!("{} components", self.components.len()))
            .field("design_system", &self.design_system)
            .field("event_handlers", &format!("{} event handlers", self.event_handlers.len()))
            .finish()
    }
}

impl ComponentRegistry {
    pub fn new(design_system: DesignSystem) -> Self {
        Self {
            components: HashMap::new(),
            design_system,
            event_handlers: HashMap::new(),
        }
    }

    pub fn register_component(&mut self, component: Box<dyn UIComponent>) -> RobinResult<ComponentId> {
        let id = component.id();
        self.components.insert(id.clone(), component);
        Ok(id)
    }

    pub fn get_component(&self, id: &ComponentId) -> Option<&dyn UIComponent> {
        self.components.get(id).map(|c| c.as_ref())
    }

    pub fn get_component_mut(&mut self, id: &ComponentId) -> Option<&mut dyn UIComponent> {
        self.components.get_mut(id).map(|c| c.as_mut())
    }

    pub fn handle_event(&mut self, event: UIEvent) -> RobinResult<Vec<UIEvent>> {
        let mut events = Vec::new();

        // Find the target component and handle the event
        if let Some(component_id) = self.extract_component_id(&event) {
            if let Some(component) = self.components.get_mut(&component_id) {
                let mut component_events = component.handle_event(event.clone())?;
                events.append(&mut component_events);
            }
        }

        // Call registered event handlers
        for (event_type, handler) in &self.event_handlers {
            if self.event_matches_type(&event, event_type) {
                handler(&event)?;
            }
        }

        Ok(events)
    }

    pub fn render_all(&self, ctx: &RenderContext) -> RobinResult<Vec<RenderOutput>> {
        let mut outputs = Vec::new();

        for component in self.components.values() {
            if component.props().visible {
                outputs.push(component.render(ctx)?);
            }
        }

        Ok(outputs)
    }

    pub fn apply_theme(&mut self, theme: &UITheme) -> RobinResult<()> {
        for component in self.components.values_mut() {
            component.apply_theme(theme)?;
        }
        Ok(())
    }

    fn extract_component_id(&self, event: &UIEvent) -> Option<ComponentId> {
        match event {
            UIEvent::Click { component_id, .. } |
            UIEvent::Hover { component_id, .. } |
            UIEvent::Focus { component_id, .. } |
            UIEvent::Input { component_id, .. } |
            UIEvent::Change { component_id, .. } |
            UIEvent::Submit { component_id, .. } |
            UIEvent::KeyDown { component_id, .. } |
            UIEvent::KeyUp { component_id, .. } |
            UIEvent::Resize { component_id, .. } |
            UIEvent::Custom { component_id, .. } => {
                // TODO: Fix this properly - ComponentId should be consistent across the system
                // For now, generate a new ComponentId since we can't convert from String
                Some(ComponentId::new())
            },
        }
    }

    fn event_matches_type(&self, event: &UIEvent, event_type: &str) -> bool {
        match event {
            UIEvent::Click { .. } => event_type == "click",
            UIEvent::Hover { .. } => event_type == "hover",
            UIEvent::Focus { .. } => event_type == "focus",
            UIEvent::Input { .. } => event_type == "input",
            UIEvent::Change { .. } => event_type == "change",
            UIEvent::Submit { .. } => event_type == "submit",
            UIEvent::KeyDown { .. } => event_type == "keydown",
            UIEvent::KeyUp { .. } => event_type == "keyup",
            UIEvent::Resize { .. } => event_type == "resize",
            UIEvent::Custom { event_type: custom_type, .. } => event_type == custom_type,
        }
    }

    pub fn register_event_handler<F>(&mut self, event_type: String, handler: F)
    where
        F: Fn(&UIEvent) -> RobinResult<()> + 'static
    {
        self.event_handlers.insert(event_type, Box::new(handler));
    }
}

/// Utility functions for component development
pub mod utils {
    use super::*;

    pub fn generate_component_id(component_type: &str) -> ComponentId {
        // Generate a unique numeric ID rather than string-based
        ComponentId::new()
    }

    pub fn merge_styles(base: Option<Style>, override_style: Option<Style>) -> Option<Style> {
        match (base, override_style) {
            (Some(mut base), Some(override_style)) => {
                // Merge styles with override taking precedence
                if let Some(width) = override_style.width { base.width = Some(width); }
                if let Some(height) = override_style.height { base.height = Some(height); }
                if let Some(color) = override_style.color { base.color = Some(color); }
                if let Some(background) = override_style.background_color { base.background_color = Some(background); }
                Some(base)
            },
            (None, Some(override_style)) => Some(override_style),
            (Some(base), None) => Some(base),
            (None, None) => None,
        }
    }

    pub fn apply_size_to_style(style: &mut Style, size: Size) {
        use super::super::css_in_rust::Dimension;

        let (height, padding, font_size) = match size {
            Size::XSmall => (Dimension::Px(24.0), 4.0, 12.0),
            Size::Small => (Dimension::Px(32.0), 8.0, 14.0),
            Size::Medium => (Dimension::Px(40.0), 12.0, 16.0),
            Size::Large => (Dimension::Px(48.0), 16.0, 18.0),
            Size::XLarge => (Dimension::Px(56.0), 20.0, 20.0),
        };

        style.height = Some(height);
        style.padding = Some(BoxSpacing::all(padding));
        if let Some(ref mut typography) = style.typography {
            typography.font_size = font_size;
        }
    }

    pub fn get_color_for_variant(variant: ColorVariant, theme: &UITheme) -> String {
        match (variant, theme) {
            (ColorVariant::Primary, UITheme::Dark) => "#3B82F6".to_string(),
            (ColorVariant::Primary, _) => "#2563EB".to_string(),
            (ColorVariant::Secondary, UITheme::Dark) => "#6B7280".to_string(),
            (ColorVariant::Secondary, _) => "#64748B".to_string(),
            (ColorVariant::Success, _) => "#10B981".to_string(),
            (ColorVariant::Warning, _) => "#F59E0B".to_string(),
            (ColorVariant::Error, _) => "#EF4444".to_string(),
            (ColorVariant::Info, _) => "#3B82F6".to_string(),
            (ColorVariant::Neutral, UITheme::Dark) => "#374151".to_string(),
            (ColorVariant::Neutral, _) => "#6B7280".to_string(),
        }
    }
}

// Add uuid dependency - this would need to be added to Cargo.toml
mod uuid {
    pub struct Uuid;
    impl Uuid {
        pub fn new_v4() -> Self { Self }
        pub fn to_string(&self) -> String {
            format!("{:x}", rand::random::<u64>())
        }
    }
}