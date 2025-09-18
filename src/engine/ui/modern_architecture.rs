// Modern UI Component Architecture for Robin Engine
// Implements a flexible, composable, and reactive component system

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};
use serde::{Serialize, Deserialize};

use crate::engine::{
    math::Vec2,
    error::{RobinResult, RobinError},
};

// ============================================================================
// Core Component Architecture
// ============================================================================

/// Unique identifier for components
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComponentId(pub u64);

impl ComponentId {
    pub fn new() -> Self {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        Self(COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst))
    }
}

impl std::fmt::Display for ComponentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Component({})", self.0)
    }
}

impl PartialEq<String> for ComponentId {
    fn eq(&self, other: &String) -> bool {
        self.to_string() == *other
    }
}

impl PartialEq<ComponentId> for String {
    fn eq(&self, other: &ComponentId) -> bool {
        *self == other.to_string()
    }
}

impl PartialEq<&str> for ComponentId {
    fn eq(&self, other: &&str) -> bool {
        self.to_string() == *other
    }
}

impl PartialEq<ComponentId> for &str {
    fn eq(&self, other: &ComponentId) -> bool {
        *self == other.to_string()
    }
}

/// Component lifecycle events
#[derive(Debug, Clone, PartialEq)]
pub enum LifecycleEvent {
    Created,
    Mounted,
    Updated,
    Unmounted,
    Destroyed,
}

/// Component properties trait
pub trait ComponentProps: Debug + Clone + Send + Sync + 'static {
    fn default() -> Self where Self: Sized;
    fn validate(&self) -> RobinResult<()> { Ok(()) }
}

/// Component state trait
pub trait ComponentState: Debug + Clone + Send + Sync + 'static {
    fn initial() -> Self where Self: Sized;
}

/// Base component trait that all UI components must implement
pub trait Component: Debug + Send + Sync + 'static {
    /// Get the component's unique ID
    fn id(&self) -> ComponentId;

    /// Get the component's type name
    fn type_name(&self) -> &'static str;

    /// Initialize the component
    fn init(&mut self, ctx: &mut ComponentContext) -> RobinResult<()>;

    /// Update component logic
    fn update(&mut self, ctx: &mut ComponentContext, delta_time: f32) -> RobinResult<()>;

    /// Render the component
    fn render(&self, ctx: &RenderContext) -> RobinResult<RenderOutput>;

    /// Handle lifecycle events
    fn on_lifecycle(&mut self, event: LifecycleEvent, ctx: &mut ComponentContext) -> RobinResult<()> {
        match event {
            LifecycleEvent::Created => self.on_create(ctx),
            LifecycleEvent::Mounted => self.on_mount(ctx),
            LifecycleEvent::Updated => self.on_update(ctx),
            LifecycleEvent::Unmounted => self.on_unmount(ctx),
            LifecycleEvent::Destroyed => self.on_destroy(ctx),
        }
    }

    /// Lifecycle hooks
    fn on_create(&mut self, _ctx: &mut ComponentContext) -> RobinResult<()> { Ok(()) }
    fn on_mount(&mut self, _ctx: &mut ComponentContext) -> RobinResult<()> { Ok(()) }
    fn on_update(&mut self, _ctx: &mut ComponentContext) -> RobinResult<()> { Ok(()) }
    fn on_unmount(&mut self, _ctx: &mut ComponentContext) -> RobinResult<()> { Ok(()) }
    fn on_destroy(&mut self, _ctx: &mut ComponentContext) -> RobinResult<()> { Ok(()) }

    /// Get children components
    fn children(&self) -> Vec<ComponentId> { Vec::new() }

    /// Add a child component
    fn add_child(&mut self, _child: ComponentId) -> RobinResult<()> { Ok(()) }

    /// Remove a child component
    fn remove_child(&mut self, _child: ComponentId) -> RobinResult<()> { Ok(()) }
}

// ============================================================================
// Component Context and State Management
// ============================================================================

/// Context provided to components for accessing shared state and services
pub struct ComponentContext {
    /// Global state store
    pub state_store: Arc<RwLock<StateStore>>,

    /// Event bus for component communication
    pub event_bus: Arc<RwLock<EventBus>>,

    /// Service locator for accessing engine services
    pub services: Arc<ServiceLocator>,

    /// Current component's ID
    pub current_id: ComponentId,

    /// Parent component's ID (if any)
    pub parent_id: Option<ComponentId>,
}

impl ComponentContext {
    pub fn new(
        state_store: Arc<RwLock<StateStore>>,
        event_bus: Arc<RwLock<EventBus>>,
        services: Arc<ServiceLocator>,
        current_id: ComponentId,
        parent_id: Option<ComponentId>,
    ) -> Self {
        Self {
            state_store,
            event_bus,
            services,
            current_id,
            parent_id,
        }
    }

    /// Get state from the store
    pub fn get_state<T: ComponentState>(&self, key: &str) -> Option<T> {
        self.state_store.read().unwrap().get(key)
    }

    /// Set state in the store
    pub fn set_state<T: ComponentState>(&mut self, key: &str, value: T) {
        self.state_store.write().unwrap().set(key, value);
    }

    /// Emit an event
    pub fn emit_event(&mut self, event: ComponentEvent) {
        self.event_bus.write().unwrap().emit(event);
    }

    /// Subscribe to events
    pub fn subscribe<F>(&mut self, event_type: String, handler: F) -> SubscriptionId
    where
        F: Fn(&ComponentEvent) + Send + Sync + 'static,
    {
        self.event_bus.write().unwrap().subscribe(event_type, handler)
    }
}

/// Global state store for components
pub struct StateStore {
    states: HashMap<String, Box<dyn Any + Send + Sync>>,
    observers: HashMap<String, Vec<Arc<dyn Fn() + Send + Sync>>>,
}

impl std::fmt::Debug for StateStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StateStore")
            .field("states", &format!("{} states", self.states.len()))
            .field("observers", &format!("{} observer groups", self.observers.len()))
            .finish()
    }
}

impl StateStore {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            observers: HashMap::new(),
        }
    }

    pub fn get<T: ComponentState>(&self, key: &str) -> Option<T> {
        self.states.get(key)
            .and_then(|state| state.downcast_ref::<T>())
            .cloned()
    }

    pub fn set<T: ComponentState>(&mut self, key: &str, value: T) {
        self.states.insert(key.to_string(), Box::new(value));
        self.notify_observers(key);
    }

    pub fn observe<F>(&mut self, key: &str, observer: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.observers.entry(key.to_string())
            .or_insert_with(Vec::new)
            .push(Arc::new(observer));
    }

    fn notify_observers(&self, key: &str) {
        if let Some(observers) = self.observers.get(key) {
            for observer in observers {
                observer();
            }
        }
    }
}

// ============================================================================
// Event System
// ============================================================================

/// Component events for communication
#[derive(Debug, Clone)]
pub struct ComponentEvent {
    pub event_type: String,
    pub source_id: ComponentId,
    pub target_id: Option<ComponentId>,
    pub data: EventData,
    pub timestamp: std::time::SystemTime,
    pub propagation: EventPropagation,
}

#[derive(Debug, Clone)]
pub enum EventData {
    Empty,
    String(String),
    Number(f64),
    Boolean(bool),
    Position(Vec2),
    Custom(HashMap<String, String>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum EventPropagation {
    Bubble,     // Event bubbles up to parents
    Capture,    // Event captures down to children
    Direct,     // Event goes directly to target
}

/// Subscription ID for event listeners
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriptionId(u64);

/// Event bus for component communication
pub struct EventBus {
    listeners: HashMap<String, Vec<(SubscriptionId, Arc<dyn Fn(&ComponentEvent) + Send + Sync>)>>,
    next_subscription_id: u64,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            listeners: HashMap::new(),
            next_subscription_id: 0,
        }
    }

    pub fn subscribe<F>(&mut self, event_type: String, handler: F) -> SubscriptionId
    where
        F: Fn(&ComponentEvent) + Send + Sync + 'static,
    {
        let id = SubscriptionId(self.next_subscription_id);
        self.next_subscription_id += 1;

        self.listeners.entry(event_type)
            .or_insert_with(Vec::new)
            .push((id, Arc::new(handler)));

        id
    }

    pub fn unsubscribe(&mut self, event_type: &str, subscription_id: SubscriptionId) {
        if let Some(listeners) = self.listeners.get_mut(event_type) {
            listeners.retain(|(id, _)| *id != subscription_id);
        }
    }

    pub fn emit(&self, event: ComponentEvent) {
        if let Some(listeners) = self.listeners.get(&event.event_type) {
            for (_, handler) in listeners {
                handler(&event);
            }
        }
    }
}

// ============================================================================
// Rendering System
// ============================================================================

/// Render context provided to components during rendering
pub struct RenderContext {
    pub viewport_size: Vec2,
    pub scale_factor: f32,
    pub theme: Arc<Theme>,
    pub depth: u32,
    pub parent_bounds: Option<Bounds>,
}

/// Output from component rendering
#[derive(Debug, Clone)]
pub struct RenderOutput {
    // Modern rendering primitives
    pub primitives: Vec<RenderPrimitive>,
    pub bounds: Bounds,
    pub interaction_bounds: Option<Bounds>,

    // Legacy fields for compatibility with existing component code
    pub element_type: Option<String>,
    pub content: Option<String>,
    pub style: Option<crate::engine::ui::css_in_rust::Style>,
    pub attributes: Option<std::collections::HashMap<String, String>>,
    pub children: Option<Vec<RenderOutput>>,
}

impl RenderOutput {
    /// Create a new RenderOutput with default values for modern rendering
    pub fn new(bounds: Bounds) -> Self {
        Self {
            primitives: Vec::new(),
            bounds,
            interaction_bounds: Some(bounds),
            element_type: None,
            content: None,
            style: None,
            attributes: None,
            children: None,
        }
    }

    /// Create a legacy-style RenderOutput for compatibility with existing components
    pub fn legacy(
        element_type: &str,
        content: &str,
        style: crate::engine::ui::css_in_rust::Style,
        attributes: std::collections::HashMap<String, String>,
        children: Vec<RenderOutput>,
    ) -> Self {
        Self {
            primitives: Vec::new(),
            bounds: Bounds::new(0.0, 0.0, 100.0, 100.0), // Default bounds
            interaction_bounds: None,
            element_type: Some(element_type.to_string()),
            content: Some(content.to_string()),
            style: Some(style),
            attributes: Some(attributes),
            children: Some(children),
        }
    }

    /// Helper to wrap values in Some() for legacy components
    pub fn with_legacy_fields(
        element_type: String,
        content: String,
        style: crate::engine::ui::css_in_rust::Style,
        attributes: std::collections::HashMap<String, String>,
        children: Vec<RenderOutput>,
    ) -> Self {
        Self {
            primitives: Vec::new(),
            bounds: Bounds::new(0.0, 0.0, 100.0, 100.0), // Default bounds
            interaction_bounds: None,
            element_type: Some(element_type),
            content: Some(content),
            style: Some(style),
            attributes: Some(attributes),
            children: Some(children),
        }
    }
}

/// Primitive rendering operations
#[derive(Debug, Clone)]
pub enum RenderPrimitive {
    Rectangle {
        bounds: Bounds,
        fill: Option<Color>,
        stroke: Option<(Color, f32)>,
        border_radius: f32,
    },
    Text {
        content: String,
        position: Vec2,
        font: FontStyle,
        color: Color,
        align: TextAlign,
    },
    Image {
        source: String,
        bounds: Bounds,
        tint: Option<Color>,
    },
    Path {
        points: Vec<Vec2>,
        stroke: (Color, f32),
        fill: Option<Color>,
    },
    Custom {
        shader: String,
        params: HashMap<String, f32>,
        bounds: Bounds,
    },
}

/// Component bounds
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Bounds {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.x && point.x <= self.x + self.width &&
        point.y >= self.y && point.y <= self.y + self.height
    }

    pub fn intersects(&self, other: &Bounds) -> bool {
        !(self.x + self.width < other.x || other.x + other.width < self.x ||
          self.y + self.height < other.y || other.y + other.height < self.y)
    }
}

// ============================================================================
// Styling System
// ============================================================================

/// Color representation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const TRANSPARENT: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_hex(hex: &str) -> RobinResult<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 && hex.len() != 8 {
            return Err(RobinError::InvalidInput("Invalid hex color format".into()));
        }

        let r = u8::from_str_radix(&hex[0..2], 16)? as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16)? as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16)? as f32 / 255.0;
        let a = if hex.len() == 8 {
            u8::from_str_radix(&hex[6..8], 16)? as f32 / 255.0
        } else {
            1.0
        };

        Ok(Self { r, g, b, a })
    }
}

/// Font styling
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FontStyle {
    pub family: String,
    pub size: f32,
    pub weight: FontWeight,
    pub style: FontVariant,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FontWeight {
    Thin,
    Light,
    Regular,
    Medium,
    Bold,
    Black,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FontVariant {
    Normal,
    Italic,
    Oblique,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
}

/// Theme system for consistent styling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub colors: ThemeColors,
    pub typography: ThemeTypography,
    pub spacing: ThemeSpacing,
    pub borders: ThemeBorders,
    pub shadows: ThemeShadows,
    pub animations: ThemeAnimations,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub background: Color,
    pub surface: Color,
    pub text: Color,
    pub text_secondary: Color,
    pub error: Color,
    pub warning: Color,
    pub success: Color,
    pub info: Color,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeTypography {
    pub font_family: String,
    pub heading1: FontStyle,
    pub heading2: FontStyle,
    pub heading3: FontStyle,
    pub body: FontStyle,
    pub caption: FontStyle,
    pub button: FontStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSpacing {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub xxl: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeBorders {
    pub radius_sm: f32,
    pub radius_md: f32,
    pub radius_lg: f32,
    pub width_thin: f32,
    pub width_medium: f32,
    pub width_thick: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeShadows {
    pub sm: ShadowStyle,
    pub md: ShadowStyle,
    pub lg: ShadowStyle,
    pub xl: ShadowStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowStyle {
    pub offset: Vec2,
    pub blur: f32,
    pub spread: f32,
    pub color: Color,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeAnimations {
    pub duration_fast: f32,
    pub duration_normal: f32,
    pub duration_slow: f32,
    pub easing_linear: String,
    pub easing_ease_in: String,
    pub easing_ease_out: String,
    pub easing_ease_in_out: String,
    pub easing_spring: String,
}

// ============================================================================
// Service Locator
// ============================================================================

/// Service locator for accessing engine services
pub struct ServiceLocator {
    services: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl ServiceLocator {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    pub fn register<T: 'static + Send + Sync>(&mut self, service: T) {
        self.services.insert(TypeId::of::<T>(), Box::new(service));
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.services.get(&TypeId::of::<T>())
            .and_then(|service| service.downcast_ref::<T>())
    }
}

// ============================================================================
// Component Factory
// ============================================================================

/// Factory for creating components
pub struct ComponentFactory {
    creators: HashMap<String, Box<dyn Fn() -> Box<dyn Component> + Send + Sync>>,
}

impl ComponentFactory {
    pub fn new() -> Self {
        Self {
            creators: HashMap::new(),
        }
    }

    pub fn register<F>(&mut self, component_type: &str, creator: F)
    where
        F: Fn() -> Box<dyn Component> + Send + Sync + 'static,
    {
        self.creators.insert(component_type.to_string(), Box::new(creator));
    }

    pub fn create(&self, component_type: &str) -> Option<Box<dyn Component>> {
        self.creators.get(component_type).map(|creator| creator())
    }
}

// ============================================================================
// Reactive System
// ============================================================================

/// Reactive property that notifies on change
pub struct Reactive<T> {
    value: T,
    listeners: Vec<Arc<dyn Fn(&T) + Send + Sync>>,
}

impl<T: std::fmt::Debug> std::fmt::Debug for Reactive<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Reactive")
            .field("value", &self.value)
            .field("listeners", &format!("{} listeners", self.listeners.len()))
            .finish()
    }
}

impl<T: Clone> Reactive<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            listeners: Vec::new(),
        }
    }

    pub fn get(&self) -> &T {
        &self.value
    }

    pub fn set(&mut self, value: T) {
        self.value = value;
        self.notify();
    }

    pub fn update<F>(&mut self, updater: F)
    where
        F: FnOnce(&mut T),
    {
        updater(&mut self.value);
        self.notify();
    }

    pub fn subscribe<F>(&mut self, listener: F)
    where
        F: Fn(&T) + Send + Sync + 'static,
    {
        self.listeners.push(Arc::new(listener));
    }

    fn notify(&self) {
        for listener in &self.listeners {
            listener(&self.value);
        }
    }
}

// ============================================================================
// Layout System
// ============================================================================

/// Layout constraints for components
#[derive(Debug, Clone, Copy)]
pub struct LayoutConstraints {
    pub min_width: Option<f32>,
    pub max_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_height: Option<f32>,
    pub preferred_width: Option<f32>,
    pub preferred_height: Option<f32>,
}

impl Default for LayoutConstraints {
    fn default() -> Self {
        Self {
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
            preferred_width: None,
            preferred_height: None,
        }
    }
}

/// Layout properties for components
#[derive(Debug, Clone)]
pub struct LayoutProps {
    pub constraints: LayoutConstraints,
    pub margin: EdgeInsets,
    pub padding: EdgeInsets,
    pub flex: Option<f32>,
    pub align_self: Option<Alignment>,
    pub position: PositionType,
}

#[derive(Debug, Clone, Copy)]
pub struct EdgeInsets {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl EdgeInsets {
    pub fn all(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Alignment {
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PositionType {
    Relative,
    Absolute,
    Fixed,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_id_generation() {
        let id1 = ComponentId::new();
        let id2 = ComponentId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_color_from_hex() {
        let color = Color::from_hex("#FF0000").unwrap();
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 0.0);
        assert_eq!(color.b, 0.0);
        assert_eq!(color.a, 1.0);

        let color_with_alpha = Color::from_hex("#FF0000AA").unwrap();
        assert!((color_with_alpha.a - 0.667).abs() < 0.01);
    }

    #[test]
    fn test_bounds_contains() {
        let bounds = Bounds::new(10.0, 10.0, 100.0, 100.0);
        assert!(bounds.contains(Vec2::new(50.0, 50.0)));
        assert!(!bounds.contains(Vec2::new(5.0, 5.0)));
        assert!(!bounds.contains(Vec2::new(120.0, 120.0)));
    }

    #[test]
    fn test_reactive_property() {
        let mut reactive = Reactive::new(42);
        let mut received_value = 0;

        reactive.subscribe(move |value| {
            received_value = *value;
        });

        reactive.set(100);
        assert_eq!(*reactive.get(), 100);
    }
}