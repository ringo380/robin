use crate::engine::{
    math::Vec2,
    ui::ElementId,
};

/// UI events that can be triggered by user interactions
#[derive(Debug, Clone)]
pub enum UIEvent {
    /// Mouse click event
    Click {
        element_id: ElementId,
        position: (f32, f32),
    },
    /// Mouse hover enter event
    HoverEnter {
        element_id: ElementId,
    },
    /// Mouse hover exit event
    HoverExit {
        element_id: ElementId,
    },
    /// Element focus gained
    FocusGained {
        element_id: ElementId,
    },
    /// Element focus lost
    FocusLost {
        element_id: ElementId,
    },
    /// Button pressed event
    ButtonPressed {
        element_id: ElementId,
    },
    /// Button released event
    ButtonReleased {
        element_id: ElementId,
    },
    /// Text input event
    TextInput {
        element_id: ElementId,
        text: String,
    },
    /// Value changed event (for sliders, inputs, etc.)
    ValueChanged {
        element_id: ElementId,
        old_value: f32,
        new_value: f32,
    },
    /// Custom event with arbitrary data
    Custom {
        element_id: ElementId,
        event_type: String,
        data: String,
    },
}

impl UIEvent {
    pub fn get_element_id(&self) -> ElementId {
        match self {
            UIEvent::Click { element_id, .. } => *element_id,
            UIEvent::HoverEnter { element_id } => *element_id,
            UIEvent::HoverExit { element_id } => *element_id,
            UIEvent::FocusGained { element_id } => *element_id,
            UIEvent::FocusLost { element_id } => *element_id,
            UIEvent::ButtonPressed { element_id } => *element_id,
            UIEvent::ButtonReleased { element_id } => *element_id,
            UIEvent::TextInput { element_id, .. } => *element_id,
            UIEvent::ValueChanged { element_id, .. } => *element_id,
            UIEvent::Custom { element_id, .. } => *element_id,
        }
    }
}

/// Event handler trait for UI elements
pub trait EventHandler {
    fn handle_event(&mut self, event: &UIEvent) -> bool;
}

/// Event listener callback type
pub type EventCallback = Box<dyn Fn(&UIEvent) + Send + Sync>;

/// Event dispatcher for managing UI event callbacks
pub struct EventDispatcher {
    listeners: std::collections::HashMap<ElementId, Vec<EventCallback>>,
    global_listeners: Vec<EventCallback>,
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self {
            listeners: std::collections::HashMap::new(),
            global_listeners: Vec::new(),
        }
    }

    /// Add an event listener for a specific element
    pub fn add_listener(&mut self, element_id: ElementId, callback: EventCallback) {
        self.listeners
            .entry(element_id)
            .or_insert_with(Vec::new)
            .push(callback);
    }

    /// Add a global event listener that receives all events
    pub fn add_global_listener(&mut self, callback: EventCallback) {
        self.global_listeners.push(callback);
    }

    /// Dispatch an event to all relevant listeners
    pub fn dispatch(&self, event: &UIEvent) {
        let element_id = event.get_element_id();

        // Call element-specific listeners
        if let Some(listeners) = self.listeners.get(&element_id) {
            for callback in listeners {
                callback(event);
            }
        }

        // Call global listeners
        for callback in &self.global_listeners {
            callback(event);
        }
    }

    /// Remove all listeners for an element
    pub fn remove_element_listeners(&mut self, element_id: ElementId) {
        self.listeners.remove(&element_id);
    }

    /// Clear all listeners
    pub fn clear(&mut self) {
        self.listeners.clear();
        self.global_listeners.clear();
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}