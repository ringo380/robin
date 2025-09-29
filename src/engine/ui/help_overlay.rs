/*!
 * Help Overlay System
 *
 * Displays keyboard shortcuts and controls in an overlay accessible via 'H' key.
 * Part of the Robin Engine UI system for user guidance.
 */

use crate::engine::{
    error::RobinResult,
    input::InputManager,
    math::Vec2,
};

/// Help overlay state and content
#[derive(Debug, Clone)]
pub struct HelpOverlay {
    pub visible: bool,
    pub shortcuts: Vec<HelpSection>,
    pub position: Vec2,
    pub size: Vec2,
    pub opacity: f32,
    pub animation_time: f32,
}

/// A section of help content (e.g., "Movement", "Build Mode", etc.)
#[derive(Debug, Clone)]
pub struct HelpSection {
    pub title: String,
    pub shortcuts: Vec<KeyboardShortcut>,
}

/// Individual keyboard shortcut entry
#[derive(Debug, Clone)]
pub struct KeyboardShortcut {
    pub keys: String,       // e.g., "WASD", "M/E", "Ctrl+Z"
    pub description: String, // e.g., "Move camera", "Cycle build mode"
}

impl HelpOverlay {
    pub fn new() -> Self {
        Self {
            visible: false,
            shortcuts: Self::create_default_shortcuts(),
            position: Vec2::new(50.0, 50.0), // Centered position offset
            size: Vec2::new(600.0, 400.0),
            opacity: 0.95,
            animation_time: 0.0,
        }
    }

    /// Toggle the help overlay visibility
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
        self.animation_time = 0.0;
    }

    /// Show the help overlay
    pub fn show(&mut self) {
        self.visible = true;
        self.animation_time = 0.0;
    }

    /// Hide the help overlay
    pub fn hide(&mut self) {
        self.visible = false;
        self.animation_time = 0.0;
    }

    /// Update the overlay (for animations, etc.)
    pub fn update(&mut self, delta_time: f32, input: &InputManager) -> RobinResult<()> {
        if self.visible {
            self.animation_time += delta_time;

            // Handle Escape key to close overlay
            if input.is_key_just_pressed(&winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape)) {
                self.hide();
            }
        }
        Ok(())
    }

    /// Check if the overlay is currently visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Get formatted help text for rendering
    pub fn get_help_text(&self) -> Vec<String> {
        let mut lines = Vec::new();

        lines.push("=== ROBIN ENGINE CONTROLS ===".to_string());
        lines.push("".to_string());

        for section in &self.shortcuts {
            lines.push(format!("[ {} ]", section.title));
            for shortcut in &section.shortcuts {
                lines.push(format!("  {}: {}", shortcut.keys, shortcut.description));
            }
            lines.push("".to_string());
        }

        lines.push("Press H to close this help or ESC".to_string());
        lines
    }

    /// Create the complete keyboard shortcuts documentation
    fn create_default_shortcuts() -> Vec<HelpSection> {
        vec![
            HelpSection {
                title: "Camera & Movement".to_string(),
                shortcuts: vec![
                    KeyboardShortcut {
                        keys: "WASD".to_string(),
                        description: "Move camera horizontally".to_string(),
                    },
                    KeyboardShortcut {
                        keys: "Space".to_string(),
                        description: "Jump (when grounded)".to_string(),
                    },
                    KeyboardShortcut {
                        keys: "Mouse".to_string(),
                        description: "Look around (first-person view)".to_string(),
                    },
                    KeyboardShortcut {
                        keys: "Arrow Keys".to_string(),
                        description: "Rotate camera orientation".to_string(),
                    },
                    KeyboardShortcut {
                        keys: "Scroll Wheel".to_string(),
                        description: "Zoom in/out".to_string(),
                    },
                ],
            },
            HelpSection {
                title: "Build Mode System".to_string(),
                shortcuts: vec![
                    KeyboardShortcut {
                        keys: "M / E".to_string(),
                        description: "Cycle build mode (Build/Test/Play)".to_string(),
                    },
                    KeyboardShortcut {
                        keys: "Left Click".to_string(),
                        description: "Place voxel at cursor".to_string(),
                    },
                    KeyboardShortcut {
                        keys: "Right Click".to_string(),
                        description: "Remove voxel at cursor".to_string(),
                    },
                ],
            },
            HelpSection {
                title: "Material Selection".to_string(),
                shortcuts: vec![
                    KeyboardShortcut {
                        keys: "1".to_string(),
                        description: "Select Stone material".to_string(),
                    },
                    KeyboardShortcut {
                        keys: "2".to_string(),
                        description: "Select Dirt material".to_string(),
                    },
                    KeyboardShortcut {
                        keys: "3".to_string(),
                        description: "Select Grass material".to_string(),
                    },
                    KeyboardShortcut {
                        keys: "4".to_string(),
                        description: "Select Sand material".to_string(),
                    },
                    KeyboardShortcut {
                        keys: "5".to_string(),
                        description: "Select Wood material".to_string(),
                    },
                    KeyboardShortcut {
                        keys: "6".to_string(),
                        description: "Select Glass material".to_string(),
                    },
                    KeyboardShortcut {
                        keys: "7".to_string(),
                        description: "Select Metal material".to_string(),
                    },
                    KeyboardShortcut {
                        keys: "8".to_string(),
                        description: "Select Water material".to_string(),
                    },
                    KeyboardShortcut {
                        keys: "9".to_string(),
                        description: "Select Obsidian material".to_string(),
                    },
                ],
            },
            HelpSection {
                title: "Templates & Construction".to_string(),
                shortcuts: vec![
                    KeyboardShortcut {
                        keys: "T".to_string(),
                        description: "Cycle through templates".to_string(),
                    },
                    KeyboardShortcut {
                        keys: "R".to_string(),
                        description: "Rotate current template (90° steps)".to_string(),
                    },
                    KeyboardShortcut {
                        keys: "Ctrl+Z".to_string(),
                        description: "Undo last action".to_string(),
                    },
                    KeyboardShortcut {
                        keys: "Ctrl+Y".to_string(),
                        description: "Redo last undone action".to_string(),
                    },
                ],
            },
            HelpSection {
                title: "World Management".to_string(),
                shortcuts: vec![
                    KeyboardShortcut {
                        keys: "Ctrl+S".to_string(),
                        description: "Save current world".to_string(),
                    },
                    KeyboardShortcut {
                        keys: "Ctrl+L".to_string(),
                        description: "List and load saved worlds".to_string(),
                    },
                ],
            },
            HelpSection {
                title: "Interface & System".to_string(),
                shortcuts: vec![
                    KeyboardShortcut {
                        keys: "H".to_string(),
                        description: "Toggle this help overlay".to_string(),
                    },
                    KeyboardShortcut {
                        keys: "ESC".to_string(),
                        description: "Close overlays / Exit application".to_string(),
                    },
                    KeyboardShortcut {
                        keys: "F1".to_string(),
                        description: "Toggle legacy help (console)".to_string(),
                    },
                    KeyboardShortcut {
                        keys: "F2".to_string(),
                        description: "Toggle UI overlay display".to_string(),
                    },
                ],
            },
        ]
    }

    /// Add a custom shortcut section
    pub fn add_section(&mut self, section: HelpSection) {
        self.shortcuts.push(section);
    }

    /// Update an existing shortcut
    pub fn update_shortcut(&mut self, section_title: &str, key: &str, new_description: &str) {
        for section in &mut self.shortcuts {
            if section.title == section_title {
                for shortcut in &mut section.shortcuts {
                    if shortcut.keys == key {
                        shortcut.description = new_description.to_string();
                        return;
                    }
                }
            }
        }
    }

    /// Get the current animation alpha for fade-in effect
    pub fn get_animation_alpha(&self) -> f32 {
        if !self.visible {
            return 0.0;
        }

        // Fade in over 0.3 seconds
        let fade_duration = 0.3;
        (self.animation_time / fade_duration).min(1.0)
    }
}

impl Default for HelpOverlay {
    fn default() -> Self {
        Self::new()
    }
}