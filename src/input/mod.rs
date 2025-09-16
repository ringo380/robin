// Input System for Robin Engine
// Handles keyboard, mouse, and controller input for the Engineer Build Mode

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct InputSystem {
    pub keyboard: KeyboardState,
    pub mouse: MouseState,
    pub controller: ControllerState,
    pub bindings: InputBindings,
    pub input_buffer: InputBuffer,
}

impl InputSystem {
    pub fn new() -> Self {
        Self {
            keyboard: KeyboardState::new(),
            mouse: MouseState::new(),
            controller: ControllerState::new(),
            bindings: InputBindings::default(),
            input_buffer: InputBuffer::new(),
        }
    }
    
    pub fn update(&mut self, delta_time: f32) {
        self.mouse.update_delta();
        self.input_buffer.update(delta_time);
        
        // Update input states
        self.update_movement_input();
        self.update_camera_input();
        self.update_action_input();
        self.update_ui_input();
    }
    
    pub fn is_key_pressed(&self, key: Key) -> bool {
        self.keyboard.is_pressed(key)
    }
    
    pub fn is_key_just_pressed(&self, key: Key) -> bool {
        self.keyboard.is_just_pressed(key)
    }
    
    pub fn is_key_released(&self, key: Key) -> bool {
        !self.keyboard.is_pressed(key)
    }
    
    pub fn get_mouse_position(&self) -> [f32; 2] {
        self.mouse.position
    }
    
    pub fn get_mouse_delta(&self) -> [f32; 2] {
        self.mouse.delta
    }
    
    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse.is_button_pressed(button)
    }
    
    pub fn get_movement_input(&self) -> MovementInput {
        MovementInput {
            forward: self.is_key_pressed(self.bindings.move_forward),
            backward: self.is_key_pressed(self.bindings.move_backward),
            left: self.is_key_pressed(self.bindings.move_left),
            right: self.is_key_pressed(self.bindings.move_right),
            up: self.is_key_pressed(self.bindings.move_up),
            down: self.is_key_pressed(self.bindings.move_down),
            sprint: self.is_key_pressed(self.bindings.sprint),
            crouch: self.is_key_pressed(self.bindings.crouch),
        }
    }
    
    pub fn get_camera_input(&self) -> CameraInput {
        CameraInput {
            mouse_delta: self.get_mouse_delta(),
            zoom: self.mouse.scroll_delta,
            reset: self.is_key_just_pressed(self.bindings.camera_reset),
        }
    }
    
    pub fn get_action_input(&self) -> ActionInput {
        ActionInput {
            primary_action: self.is_mouse_button_pressed(MouseButton::Left),
            secondary_action: self.is_mouse_button_pressed(MouseButton::Right),
            interact: self.is_key_just_pressed(self.bindings.interact),
            build: self.is_key_just_pressed(self.bindings.build),
            destroy: self.is_key_just_pressed(self.bindings.destroy),
            tool_select: self.get_tool_selection_input(),
            quick_save: self.is_key_just_pressed(self.bindings.quick_save),
            quick_load: self.is_key_just_pressed(self.bindings.quick_load),
        }
    }
    
    pub fn get_ui_input(&self) -> UIInput {
        UIInput {
            menu_toggle: self.is_key_just_pressed(self.bindings.menu_toggle),
            inventory_toggle: self.is_key_just_pressed(self.bindings.inventory_toggle),
            console_toggle: self.is_key_just_pressed(self.bindings.console_toggle),
            fullscreen_toggle: self.is_key_just_pressed(self.bindings.fullscreen_toggle),
            screenshot: self.is_key_just_pressed(self.bindings.screenshot),
        }
    }
    
    fn update_movement_input(&mut self) {
        // Handle movement input smoothing and acceleration
        // This would integrate with the physics system
    }
    
    fn update_camera_input(&mut self) {
        // Handle mouse sensitivity and smoothing
        if self.mouse.delta[0].abs() > 0.1 || self.mouse.delta[1].abs() > 0.1 {
            // Apply mouse sensitivity
            self.mouse.delta[0] *= self.bindings.mouse_sensitivity;
            self.mouse.delta[1] *= self.bindings.mouse_sensitivity;
        }
    }
    
    fn update_action_input(&mut self) {
        // Handle action input timing and combos
        // This would feed into the Engineer Build Mode systems
    }
    
    fn update_ui_input(&mut self) {
        // Handle UI navigation and shortcuts
    }
    
    fn get_tool_selection_input(&self) -> Option<u8> {
        for i in 0..10 {
            if self.is_key_just_pressed(Key::Num(i)) {
                return Some(i);
            }
        }
        None
    }
    
    // Event handling methods
    pub fn handle_key_down(&mut self, key: Key) {
        self.keyboard.set_key_state(key, true);
        self.input_buffer.add_event(InputEvent::KeyPressed(key));
    }
    
    pub fn handle_key_up(&mut self, key: Key) {
        self.keyboard.set_key_state(key, false);
        self.input_buffer.add_event(InputEvent::KeyReleased(key));
    }
    
    pub fn handle_mouse_move(&mut self, position: [f32; 2]) {
        let old_position = self.mouse.position;
        self.mouse.position = position;
        self.mouse.delta = [
            position[0] - old_position[0],
            position[1] - old_position[1],
        ];
        self.input_buffer.add_event(InputEvent::MouseMoved { position, delta: self.mouse.delta });
    }
    
    pub fn handle_mouse_button_down(&mut self, button: MouseButton) {
        self.mouse.set_button_state(button, true);
        self.input_buffer.add_event(InputEvent::MouseButtonPressed(button));
    }
    
    pub fn handle_mouse_button_up(&mut self, button: MouseButton) {
        self.mouse.set_button_state(button, false);
        self.input_buffer.add_event(InputEvent::MouseButtonReleased(button));
    }
    
    pub fn handle_mouse_scroll(&mut self, delta: f32) {
        self.mouse.scroll_delta = delta;
        self.input_buffer.add_event(InputEvent::MouseScrolled(delta));
    }
}

#[derive(Debug, Clone)]
pub struct KeyboardState {
    keys: HashMap<Key, KeyState>,
}

impl KeyboardState {
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
        }
    }
    
    pub fn is_pressed(&self, key: Key) -> bool {
        self.keys.get(&key).map_or(false, |state| state.pressed)
    }
    
    pub fn is_just_pressed(&self, key: Key) -> bool {
        self.keys.get(&key).map_or(false, |state| state.just_pressed)
    }
    
    pub fn set_key_state(&mut self, key: Key, pressed: bool) {
        let state = self.keys.entry(key).or_insert(KeyState::default());
        state.just_pressed = pressed && !state.pressed;
        state.pressed = pressed;
    }
    
    pub fn update(&mut self) {
        // Reset just_pressed flags
        for state in self.keys.values_mut() {
            state.just_pressed = false;
        }
    }
}

#[derive(Debug, Clone)]
pub struct KeyState {
    pub pressed: bool,
    pub just_pressed: bool,
    pub hold_time: f32,
}

impl Default for KeyState {
    fn default() -> Self {
        Self {
            pressed: false,
            just_pressed: false,
            hold_time: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MouseState {
    pub position: [f32; 2],
    pub delta: [f32; 2],
    pub scroll_delta: f32,
    pub buttons: HashMap<MouseButton, bool>,
}

impl MouseState {
    pub fn new() -> Self {
        Self {
            position: [0.0, 0.0],
            delta: [0.0, 0.0],
            scroll_delta: 0.0,
            buttons: HashMap::new(),
        }
    }
    
    pub fn is_button_pressed(&self, button: MouseButton) -> bool {
        self.buttons.get(&button).copied().unwrap_or(false)
    }
    
    pub fn set_button_state(&mut self, button: MouseButton, pressed: bool) {
        self.buttons.insert(button, pressed);
    }
    
    pub fn update_delta(&mut self) {
        // Reset scroll delta each frame
        self.scroll_delta = 0.0;
        
        // Decay mouse delta for smoothing
        self.delta[0] *= 0.8;
        self.delta[1] *= 0.8;
    }
}

#[derive(Debug, Clone)]
pub struct ControllerState {
    pub connected: bool,
    pub left_stick: [f32; 2],
    pub right_stick: [f32; 2],
    pub left_trigger: f32,
    pub right_trigger: f32,
    pub buttons: HashMap<ControllerButton, bool>,
}

impl ControllerState {
    pub fn new() -> Self {
        Self {
            connected: false,
            left_stick: [0.0, 0.0],
            right_stick: [0.0, 0.0],
            left_trigger: 0.0,
            right_trigger: 0.0,
            buttons: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct InputBindings {
    // Movement
    pub move_forward: Key,
    pub move_backward: Key,
    pub move_left: Key,
    pub move_right: Key,
    pub move_up: Key,
    pub move_down: Key,
    pub sprint: Key,
    pub crouch: Key,
    
    // Camera
    pub camera_reset: Key,
    pub mouse_sensitivity: f32,
    
    // Actions
    pub interact: Key,
    pub build: Key,
    pub destroy: Key,
    pub quick_save: Key,
    pub quick_load: Key,
    
    // UI
    pub menu_toggle: Key,
    pub inventory_toggle: Key,
    pub console_toggle: Key,
    pub fullscreen_toggle: Key,
    pub screenshot: Key,
}

impl Default for InputBindings {
    fn default() -> Self {
        Self {
            // WASD movement
            move_forward: Key::W,
            move_backward: Key::S,
            move_left: Key::A,
            move_right: Key::D,
            move_up: Key::Space,
            move_down: Key::LShift,
            sprint: Key::LCtrl,
            crouch: Key::C,
            
            // Camera
            camera_reset: Key::R,
            mouse_sensitivity: 1.0,
            
            // Actions
            interact: Key::E,
            build: Key::B,
            destroy: Key::X,
            quick_save: Key::F5,
            quick_load: Key::F9,
            
            // UI
            menu_toggle: Key::Escape,
            inventory_toggle: Key::Tab,
            console_toggle: Key::Tilde,
            fullscreen_toggle: Key::F11,
            screenshot: Key::F12,
        }
    }
}

#[derive(Debug, Clone)]
pub struct InputBuffer {
    events: Vec<InputEvent>,
    max_size: usize,
}

impl InputBuffer {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            max_size: 100,
        }
    }
    
    pub fn add_event(&mut self, event: InputEvent) {
        self.events.push(event);
        if self.events.len() > self.max_size {
            self.events.remove(0);
        }
    }
    
    pub fn get_events(&self) -> &[InputEvent] {
        &self.events
    }
    
    pub fn clear(&mut self) {
        self.events.clear();
    }
    
    pub fn update(&mut self, _delta_time: f32) {
        // Age out old events if needed
    }
}

#[derive(Debug, Clone)]
pub enum InputEvent {
    KeyPressed(Key),
    KeyReleased(Key),
    MouseMoved { position: [f32; 2], delta: [f32; 2] },
    MouseButtonPressed(MouseButton),
    MouseButtonReleased(MouseButton),
    MouseScrolled(f32),
    ControllerButtonPressed(ControllerButton),
    ControllerButtonReleased(ControllerButton),
}

// Input data structures for different systems

#[derive(Debug, Clone)]
pub struct MovementInput {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub sprint: bool,
    pub crouch: bool,
}

#[derive(Debug, Clone)]
pub struct CameraInput {
    pub mouse_delta: [f32; 2],
    pub zoom: f32,
    pub reset: bool,
}

#[derive(Debug, Clone)]
pub struct ActionInput {
    pub primary_action: bool,
    pub secondary_action: bool,
    pub interact: bool,
    pub build: bool,
    pub destroy: bool,
    pub tool_select: Option<u8>,
    pub quick_save: bool,
    pub quick_load: bool,
}

#[derive(Debug, Clone)]
pub struct UIInput {
    pub menu_toggle: bool,
    pub inventory_toggle: bool,
    pub console_toggle: bool,
    pub fullscreen_toggle: bool,
    pub screenshot: bool,
}

// Input enums

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    // Letters
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    
    // Numbers
    Num(u8), // 0-9
    
    // Special keys
    Space,
    Enter,
    Escape,
    Tab,
    Backspace,
    Delete,
    
    // Modifiers
    LShift, RShift,
    LCtrl, RCtrl,
    LAlt, RAlt,
    
    // Function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    
    // Arrow keys
    Up, Down, Left, Right,
    
    // Other
    Tilde,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    X1,
    X2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ControllerButton {
    A, B, X, Y,
    LeftBumper, RightBumper,
    LeftStick, RightStick,
    Start, Select,
    DPadUp, DPadDown, DPadLeft, DPadRight,
}

// Input context system for different game states
#[derive(Debug, Clone)]
pub struct InputContext {
    pub name: String,
    pub active: bool,
    pub priority: u8,
    pub bindings: HashMap<InputAction, InputBinding>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum InputAction {
    // Engineer actions
    EngineerMove,
    EngineerLook,
    EngineerInteract,
    EngineerBuild,
    EngineerDestroy,
    EngineerToolSelect,
    
    // World construction actions
    WorldModify,
    WorldPaint,
    WorldCopy,
    WorldPaste,
    
    // UI actions
    UINavigate,
    UISelect,
    UICancel,
    UIMenu,
    
    // Camera actions
    CameraMove,
    CameraRotate,
    CameraZoom,
    CameraReset,
}

#[derive(Debug, Clone)]
pub enum InputBinding {
    Key(Key),
    MouseButton(MouseButton),
    MouseAxis(MouseAxis),
    ControllerButton(ControllerButton),
    ControllerAxis(ControllerAxis),
    Combination(Vec<InputBinding>),
}

#[derive(Debug, Clone)]
pub enum MouseAxis {
    X,
    Y,
    Scroll,
}

#[derive(Debug, Clone)]
pub enum ControllerAxis {
    LeftStickX,
    LeftStickY,
    RightStickX,
    RightStickY,
    LeftTrigger,
    RightTrigger,
}

// Input validation and security
#[derive(Debug, Clone)]
pub struct InputValidator {
    pub max_input_rate: f32, // Max inputs per second
    pub input_history: Vec<(f32, InputEvent)>, // Time, Event
    pub blocked_combinations: Vec<Vec<Key>>,
}

impl InputValidator {
    pub fn new() -> Self {
        Self {
            max_input_rate: 30.0, // 30 inputs per second max
            input_history: Vec::new(),
            blocked_combinations: Vec::new(),
        }
    }
    
    pub fn validate_input(&mut self, event: &InputEvent, current_time: f32) -> bool {
        // Rate limiting
        self.input_history.retain(|(time, _)| current_time - time < 1.0);
        
        if self.input_history.len() as f32 > self.max_input_rate {
            return false; // Rate limit exceeded
        }
        
        // Check for blocked key combinations
        // Implementation would check current key state against blocked_combinations
        
        self.input_history.push((current_time, event.clone()));
        true
    }
}