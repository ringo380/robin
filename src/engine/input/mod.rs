use winit::event::{ElementState, MouseButton};
use winit::keyboard::{Key, NamedKey};
use std::collections::HashMap;

pub struct InputManager {
    keys: HashMap<Key, bool>,
    keys_just_pressed: HashMap<Key, bool>,
    keys_just_released: HashMap<Key, bool>,
    mouse_buttons: HashMap<MouseButton, bool>,
    mouse_position: (f64, f64),
    mouse_delta: (f64, f64),
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
            keys_just_pressed: HashMap::new(),
            keys_just_released: HashMap::new(),
            mouse_buttons: HashMap::new(),
            mouse_position: (0.0, 0.0),
            mouse_delta: (0.0, 0.0),
        }
    }

    pub fn update_key(&mut self, key: Key, state: ElementState) {
        let pressed = state == ElementState::Pressed;
        let was_pressed = self.keys.get(&key).copied().unwrap_or(false);

        // Track just pressed/released states
        if pressed && !was_pressed {
            self.keys_just_pressed.insert(key.clone(), true);
        } else if !pressed && was_pressed {
            self.keys_just_released.insert(key.clone(), true);
        }

        self.keys.insert(key, pressed);
    }

    pub fn update_mouse_button(&mut self, button: MouseButton, state: ElementState) {
        let pressed = state == ElementState::Pressed;
        self.mouse_buttons.insert(button, pressed);
    }

    pub fn update_mouse_position(&mut self, position: (f64, f64)) {
        let delta = (
            position.0 - self.mouse_position.0,
            position.1 - self.mouse_position.1,
        );
        self.mouse_delta = delta;
        self.mouse_position = position;
    }

    pub fn is_key_pressed(&self, key: &Key) -> bool {
        self.keys.get(key).copied().unwrap_or(false)
    }

    pub fn is_named_key_pressed(&self, named_key: NamedKey) -> bool {
        self.keys.get(&Key::Named(named_key)).copied().unwrap_or(false)
    }

    pub fn is_key_just_pressed(&self, key: &Key) -> bool {
        self.keys_just_pressed.get(key).copied().unwrap_or(false)
    }

    pub fn is_named_key_just_pressed(&self, named_key: NamedKey) -> bool {
        self.keys_just_pressed.get(&Key::Named(named_key)).copied().unwrap_or(false)
    }

    pub fn is_key_just_released(&self, key: &Key) -> bool {
        self.keys_just_released.get(key).copied().unwrap_or(false)
    }

    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons.get(&button).copied().unwrap_or(false)
    }

    pub fn mouse_position(&self) -> (f64, f64) {
        self.mouse_position
    }

    pub fn mouse_delta(&self) -> (f64, f64) {
        self.mouse_delta
    }

    pub fn reset_frame(&mut self) {
        self.mouse_delta = (0.0, 0.0);
        self.keys_just_pressed.clear();
        self.keys_just_released.clear();
    }
}