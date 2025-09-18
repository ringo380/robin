/*!
 * Interactive Tutorial and Onboarding System
 *
 * Comprehensive tutorial system for Robin Engine's Engineer Build Mode.
 * Part of Phase 3.1: User Interface and Experience Polish.
 */

use crate::engine::{
    math::Vec2,
    ui::{
        UIBounds, UIState, events::UIEvent, UIElement, ElementId, UIManager,
        modern_components::{ModernButton, ModernCard, ModernNotification, NotificationType, AccessibilityProps},
        design_system::DesignSystem, styling::Spacing
    },
    input::InputManager,
    generation::templates::UITheme,
};
use std::collections::HashMap;

/// Events that the tutorial system sends to the UI manager
#[derive(Debug, Clone)]
pub enum TutorialEvent {
    ShowInfo(String),
    ShowSuccess(String),
    ShowError(String),
    ShowHint(String),
    RefreshUI,
}

/// Tutorial step definition
#[derive(Debug, Clone)]
pub struct TutorialStep {
    pub id: String,
    pub title: String,
    pub description: String,
    pub objective: String,
    pub highlight_elements: Vec<String>,
    pub actions_required: Vec<TutorialAction>,
    pub hints: Vec<String>,
    pub skip_allowed: bool,
    pub auto_advance: bool,
    pub completion_condition: CompletionCondition,
}

/// Actions that the user needs to perform
#[derive(Debug, Clone)]
pub enum TutorialAction {
    ClickElement(String),
    PressKey(String),
    HoverElement(String),
    BuildStructure(String),
    PlaceVoxel { material: String, position: Vec2 },
    OpenMenu(String),
    CompleteTask(String),
    Wait(f32), // Wait for X seconds
}

/// Conditions for completing a tutorial step
#[derive(Debug, Clone)]
pub enum CompletionCondition {
    Manual,                          // User must click "Next"
    ActionCompleted(TutorialAction), // Specific action must be completed
    AllActionsCompleted,             // All actions in the step must be completed
    TimeElapsed(f32),               // Auto-advance after time
    CustomCondition(String),         // Custom logic condition
}

/// Tutorial progress tracking
#[derive(Debug, Clone)]
pub struct TutorialProgress {
    pub current_step: usize,
    pub completed_steps: Vec<String>,
    pub skipped_steps: Vec<String>,
    pub actions_completed: HashMap<String, bool>,
    pub start_time: std::time::SystemTime,
    pub step_start_time: std::time::SystemTime,
    pub total_hints_used: u32,
}

impl Default for TutorialProgress {
    fn default() -> Self {
        let now = std::time::SystemTime::now();
        Self {
            current_step: 0,
            completed_steps: Vec::new(),
            skipped_steps: Vec::new(),
            actions_completed: HashMap::new(),
            start_time: now,
            step_start_time: now,
            total_hints_used: 0,
        }
    }
}

/// Main tutorial system manager
pub struct TutorialSystem {
    steps: Vec<TutorialStep>,
    progress: TutorialProgress,
    active: bool,
    paused: bool,
    ui_elements: Vec<ElementId>,
    highlight_overlay: Option<ElementId>,
    tutorial_panel: Option<ElementId>,
    accessibility_enabled: bool,
    auto_pause_on_confusion: bool,
    user_preferences: TutorialPreferences,
}

/// User preferences for tutorial experience
#[derive(Debug, Clone)]
pub struct TutorialPreferences {
    pub show_hints_automatically: bool,
    pub auto_advance_delay: f32,
    pub skip_completed_sections: bool,
    pub voice_narration: bool,
    pub reduced_motion: bool,
    pub high_contrast_highlights: bool,
}

impl Default for TutorialPreferences {
    fn default() -> Self {
        Self {
            show_hints_automatically: true,
            auto_advance_delay: 3.0,
            skip_completed_sections: false,
            voice_narration: false,
            reduced_motion: false,
            high_contrast_highlights: false,
        }
    }
}

impl TutorialSystem {
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            progress: TutorialProgress::default(),
            active: false,
            paused: false,
            ui_elements: Vec::new(),
            highlight_overlay: None,
            tutorial_panel: None,
            accessibility_enabled: true,
            auto_pause_on_confusion: true,
            user_preferences: TutorialPreferences::default(),
        }
    }

    /// Initialize the Engineer Build Mode tutorial
    pub fn init_engineer_build_mode_tutorial(&mut self) {
        self.steps = vec![
            // Step 1: Welcome and Overview
            TutorialStep {
                id: "welcome".to_string(),
                title: "Welcome to Engineer Build Mode".to_string(),
                description: "You are now an engineer in a dynamic 3D world where you can build, create, and experiment in real-time.".to_string(),
                objective: "Learn the basics of Engineer Build Mode".to_string(),
                highlight_elements: vec!["main_viewport".to_string()],
                actions_required: vec![],
                hints: vec![
                    "Use WASD keys to move around".to_string(),
                    "Move your mouse to look around".to_string(),
                    "This tutorial will guide you through all the features".to_string(),
                ],
                skip_allowed: true,
                auto_advance: false,
                completion_condition: CompletionCondition::Manual,
            },

            // Step 2: Basic Movement
            TutorialStep {
                id: "movement".to_string(),
                title: "Basic Movement Controls".to_string(),
                description: "Learn to navigate the 3D world using keyboard and mouse controls.".to_string(),
                objective: "Move around using WASD keys and mouse look".to_string(),
                highlight_elements: vec!["movement_indicator".to_string()],
                actions_required: vec![
                    TutorialAction::PressKey("W".to_string()),
                    TutorialAction::PressKey("A".to_string()),
                    TutorialAction::PressKey("S".to_string()),
                    TutorialAction::PressKey("D".to_string()),
                ],
                hints: vec![
                    "W = Move Forward".to_string(),
                    "S = Move Backward".to_string(),
                    "A = Move Left".to_string(),
                    "D = Move Right".to_string(),
                    "Move your mouse to look around".to_string(),
                ],
                skip_allowed: false,
                auto_advance: false,
                completion_condition: CompletionCondition::AllActionsCompleted,
            },

            // Step 3: Building Tools Introduction
            TutorialStep {
                id: "building_tools".to_string(),
                title: "Building Tools Overview".to_string(),
                description: "Discover the powerful building tools available in Engineer Build Mode.".to_string(),
                objective: "Open the building tools menu".to_string(),
                highlight_elements: vec!["tools_menu_button".to_string()],
                actions_required: vec![
                    TutorialAction::ClickElement("tools_menu_button".to_string()),
                ],
                hints: vec![
                    "Click the tools icon in the toolbar".to_string(),
                    "You can also press 'B' to open building tools".to_string(),
                ],
                skip_allowed: true,
                auto_advance: false,
                completion_condition: CompletionCondition::ActionCompleted(
                    TutorialAction::ClickElement("tools_menu_button".to_string())
                ),
            },

            // Step 4: First Voxel Placement
            TutorialStep {
                id: "place_voxel".to_string(),
                title: "Place Your First Voxel".to_string(),
                description: "Learn to place voxels in the world to start building.".to_string(),
                objective: "Place a stone voxel in the world".to_string(),
                highlight_elements: vec!["material_selector".to_string(), "placement_cursor".to_string()],
                actions_required: vec![
                    TutorialAction::PlaceVoxel {
                        material: "stone".to_string(),
                        position: Vec2::new(0.0, 0.0),
                    },
                ],
                hints: vec![
                    "Select stone from the material palette".to_string(),
                    "Left-click to place a voxel".to_string(),
                    "Right-click to remove a voxel".to_string(),
                ],
                skip_allowed: false,
                auto_advance: false,
                completion_condition: CompletionCondition::ActionCompleted(
                    TutorialAction::PlaceVoxel {
                        material: "stone".to_string(),
                        position: Vec2::new(0.0, 0.0),
                    }
                ),
            },

            // Step 5: AI Assistant
            TutorialStep {
                id: "ai_assistant".to_string(),
                title: "Meet Your AI Assistant".to_string(),
                description: "The AI Assistant can help you build more efficiently and suggest improvements.".to_string(),
                objective: "Open the AI Assistant panel".to_string(),
                highlight_elements: vec!["ai_assistant_button".to_string()],
                actions_required: vec![
                    TutorialAction::ClickElement("ai_assistant_button".to_string()),
                ],
                hints: vec![
                    "The AI Assistant learns from your building patterns".to_string(),
                    "It can suggest optimal designs and materials".to_string(),
                    "Press 'H' for AI help anytime".to_string(),
                ],
                skip_allowed: true,
                auto_advance: false,
                completion_condition: CompletionCondition::ActionCompleted(
                    TutorialAction::ClickElement("ai_assistant_button".to_string())
                ),
            },

            // Step 6: Story System
            TutorialStep {
                id: "story_system".to_string(),
                title: "Dynamic Story System".to_string(),
                description: "Your actions create dynamic stories and quests in the world.".to_string(),
                objective: "Check your current quest log".to_string(),
                highlight_elements: vec!["quest_log_button".to_string()],
                actions_required: vec![
                    TutorialAction::ClickElement("quest_log_button".to_string()),
                ],
                hints: vec![
                    "Stories emerge from your building choices".to_string(),
                    "NPCs will react to your constructions".to_string(),
                    "Complete quests to unlock new materials and tools".to_string(),
                ],
                skip_allowed: true,
                auto_advance: false,
                completion_condition: CompletionCondition::ActionCompleted(
                    TutorialAction::ClickElement("quest_log_button".to_string())
                ),
            },

            // Step 7: Advanced Features Preview
            TutorialStep {
                id: "advanced_preview".to_string(),
                title: "Advanced Features".to_string(),
                description: "Preview of advanced features: vehicles, NPCs, scripting, and multiplayer.".to_string(),
                objective: "Learn about advanced capabilities".to_string(),
                highlight_elements: vec![],
                actions_required: vec![],
                hints: vec![
                    "Build vehicles for transportation".to_string(),
                    "Create NPCs with custom behaviors".to_string(),
                    "Use visual scripting for game logic".to_string(),
                    "Collaborate with other engineers in multiplayer".to_string(),
                ],
                skip_allowed: true,
                auto_advance: true,
                completion_condition: CompletionCondition::TimeElapsed(8.0),
            },

            // Step 8: Free Play
            TutorialStep {
                id: "free_play".to_string(),
                title: "You're Ready to Build!".to_string(),
                description: "Congratulations! You've completed the tutorial. The world is yours to engineer.".to_string(),
                objective: "Start your engineering adventure".to_string(),
                highlight_elements: vec![],
                actions_required: vec![],
                hints: vec![
                    "Remember: Press F1 for help anytime".to_string(),
                    "Experiment and have fun!".to_string(),
                    "Share your creations with the community".to_string(),
                ],
                skip_allowed: false,
                auto_advance: false,
                completion_condition: CompletionCondition::Manual,
            },
        ];

        log::info!("Initialized Engineer Build Mode tutorial with {} steps", self.steps.len());
    }

    /// Start the tutorial
    pub fn start(&mut self) -> Vec<TutorialEvent> {
        self.active = true;
        self.paused = false;
        self.progress = TutorialProgress::default();

        let mut events = Vec::new();
        events.push(TutorialEvent::RefreshUI); // Will create tutorial UI
        events.append(&mut self.begin_current_step());
        events.push(TutorialEvent::ShowInfo("Tutorial started! Press ESC to pause anytime.".to_string()));

        log::info!("Tutorial started");
        events
    }

    /// Stop the tutorial
    pub fn stop(&mut self) {
        self.active = false;
        self.cleanup_ui();

        log::info!("Tutorial stopped");
    }

    /// Pause or resume the tutorial
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
        log::info!("Tutorial {}", if self.paused { "paused" } else { "resumed" });
    }

    /// Skip the current step
    pub fn skip_current_step(&mut self) -> Vec<TutorialEvent> {
        let mut events = Vec::new();

        if let Some(step) = self.get_current_step().cloned() {
            if step.skip_allowed {
                self.progress.skipped_steps.push(step.id.clone());
                events.append(&mut self.advance_to_next_step());
                events.push(TutorialEvent::ShowSuccess("Step skipped.".to_string()));
            } else {
                events.push(TutorialEvent::ShowError("This step cannot be skipped.".to_string()));
            }
        }

        events
    }

    /// Get the current tutorial step
    pub fn get_current_step(&self) -> Option<&TutorialStep> {
        self.steps.get(self.progress.current_step)
    }

    /// Check if tutorial is active
    pub fn is_active(&self) -> bool {
        self.active && !self.paused
    }

    /// Update the tutorial system
    pub fn update(&mut self, delta_time: f32, input: &InputManager) -> Vec<TutorialEvent> {
        let mut events = Vec::new();
        if !self.active || self.paused {
            return events;
        }

        // Handle ESC key to pause
        if input.is_named_key_just_pressed(winit::keyboard::NamedKey::Escape) {
            self.toggle_pause();
            return events;
        }

        // Handle hint request (F1 key)
        if input.is_named_key_just_pressed(winit::keyboard::NamedKey::F1) {
            if let Some(hint_event) = self.get_hint_event() {
                events.push(hint_event);
            }
        }

        // Update current step
        if let Some(step) = self.get_current_step().cloned() {
            let mut step_events = self.update_step_progress(&step, delta_time, input);
            events.append(&mut step_events);
        }

        events
    }

    /// Get a hint event for the current step
    pub fn get_hint_event(&mut self) -> Option<TutorialEvent> {
        if let Some(step) = self.get_current_step().cloned() {
            if !step.hints.is_empty() {
                let hint_index = (self.progress.total_hints_used as usize) % step.hints.len();
                let hint = &step.hints[hint_index];
                self.progress.total_hints_used += 1;
                return Some(TutorialEvent::ShowHint(format!("💡 Hint: {}", hint)));
            }
        }
        None
    }

    /// Mark a tutorial action as completed
    pub fn mark_action_completed(&mut self, action: &TutorialAction) -> Vec<TutorialEvent> {
        let action_id = format!("{:?}", action);
        self.progress.actions_completed.insert(action_id, true);

        // Check if current step is completed
        if let Some(step) = self.get_current_step().cloned() {
            if self.is_step_completed(&step) {
                return self.complete_current_step();
            }
        }

        Vec::new()
    }

    /// Create the tutorial UI elements
    fn create_tutorial_ui(&mut self, ui_manager: &mut UIManager) {
        // Create tutorial panel
        let panel_bounds = UIBounds::new(20.0, 20.0, 400.0, 200.0);
        let tutorial_card = ModernCard::new(0, panel_bounds)
            .with_title("Tutorial".to_string())
            .with_content(vec!["Welcome to the tutorial!".to_string()]);

        self.tutorial_panel = Some(ui_manager.add_element(Box::new(tutorial_card)));

        // Create control buttons
        let next_button = ModernButton::primary()
            .with_text("Next".to_string())
            .with_click_callback(|| {
                // In a real implementation, this would call advance_to_next_step
                log::info!("Next button clicked");
            });

        let skip_button = ModernButton::secondary()
            .with_text("Skip".to_string())
            .with_click_callback(|| {
                // In a real implementation, this would call skip_current_step
                log::info!("Skip button clicked");
            });

        let hint_button = ModernButton::ghost()
            .with_text("Hint".to_string())
            .with_keyboard_shortcut("F1".to_string())
            .with_click_callback(|| {
                // In a real implementation, this would call show_hint
                log::info!("Hint button clicked");
            });

        self.ui_elements.push(ui_manager.add_element(Box::new(next_button)));
        self.ui_elements.push(ui_manager.add_element(Box::new(skip_button)));
        self.ui_elements.push(ui_manager.add_element(Box::new(hint_button)));
    }

    /// Update progress for the current step
    fn update_step_progress(&mut self, step: &TutorialStep, delta_time: f32, _input: &InputManager) -> Vec<TutorialEvent> {
        let mut events = Vec::new();

        // Check completion condition
        match &step.completion_condition {
            CompletionCondition::TimeElapsed(duration) => {
                let elapsed = self.progress.step_start_time.elapsed().unwrap_or_default().as_secs_f32();
                if elapsed >= *duration {
                    events.append(&mut self.complete_current_step());
                }
            }
            CompletionCondition::AllActionsCompleted => {
                if step.actions_required.iter().all(|action| {
                    let action_id = format!("{:?}", action);
                    *self.progress.actions_completed.get(&action_id).unwrap_or(&false)
                }) {
                    events.append(&mut self.complete_current_step());
                }
            }
            _ => {
                // Other conditions are handled elsewhere
            }
        }

        // Auto-show hints if enabled and user seems stuck
        if self.user_preferences.show_hints_automatically {
            let elapsed = self.progress.step_start_time.elapsed().unwrap_or_default().as_secs_f32();
            if elapsed > 30.0 && self.progress.total_hints_used == 0 {
                if let Some(hint_event) = self.get_hint_event() {
                    events.push(hint_event);
                }
            }
        }

        events
    }

    /// Check if the current step is completed
    fn is_step_completed(&self, step: &TutorialStep) -> bool {
        match &step.completion_condition {
            CompletionCondition::Manual => false, // Requires explicit "Next" click
            CompletionCondition::ActionCompleted(action) => {
                let action_id = format!("{:?}", action);
                *self.progress.actions_completed.get(&action_id).unwrap_or(&false)
            }
            CompletionCondition::AllActionsCompleted => {
                step.actions_required.iter().all(|action| {
                    let action_id = format!("{:?}", action);
                    *self.progress.actions_completed.get(&action_id).unwrap_or(&false)
                })
            }
            CompletionCondition::TimeElapsed(_) => {
                // Handled in update_step_progress
                false
            }
            CompletionCondition::CustomCondition(_) => {
                // Would need custom implementation
                false
            }
        }
    }

    /// Complete the current step and advance
    fn complete_current_step(&mut self) -> Vec<TutorialEvent> {
        let mut events = Vec::new();

        if let Some(step) = self.get_current_step().cloned() {
            self.progress.completed_steps.push(step.id.clone());
            events.push(TutorialEvent::ShowSuccess(format!("✓ Completed: {}", step.title)));
        }

        events.append(&mut self.advance_to_next_step());
        events
    }

    /// Advance to the next tutorial step
    fn advance_to_next_step(&mut self) -> Vec<TutorialEvent> {
        self.progress.current_step += 1;
        self.progress.step_start_time = std::time::SystemTime::now();

        if self.progress.current_step >= self.steps.len() {
            // Tutorial completed
            self.stop();
            Vec::new()
        } else {
            self.begin_current_step()
        }
    }

    /// Begin the current step
    fn begin_current_step(&mut self) -> Vec<TutorialEvent> {
        let mut events = Vec::new();

        if let Some(step) = self.get_current_step().cloned() {
            events.push(TutorialEvent::ShowInfo(format!("📖 {}: {}", step.title, step.objective)));
            events.push(TutorialEvent::RefreshUI);

            log::info!("Started tutorial step: {} - {}", step.id, step.title);
        }

        events
    }

    /// Refresh the tutorial UI with current step info
    fn refresh_tutorial_ui(&mut self, _ui_manager: &mut UIManager) {
        // Update tutorial panel content with current step information
        // In a real implementation, this would update the tutorial card content
        log::debug!("Refreshed tutorial UI for step {}", self.progress.current_step);
    }

    /// Clean up tutorial UI elements
    fn cleanup_ui(&mut self) {
        // Clear UI element references (actual removal handled by UIManager)
        self.ui_elements.clear();
        self.tutorial_panel = None;

        if let Some(_overlay_id) = self.highlight_overlay {
            self.highlight_overlay = None;
        }
    }

    /// Get tutorial completion statistics
    pub fn get_completion_stats(&self) -> TutorialStats {
        TutorialStats {
            total_steps: self.steps.len(),
            completed_steps: self.progress.completed_steps.len(),
            skipped_steps: self.progress.skipped_steps.len(),
            current_step: self.progress.current_step,
            total_time: self.progress.start_time.elapsed().unwrap_or_default(),
            hints_used: self.progress.total_hints_used,
            completion_percentage: if self.steps.is_empty() {
                100.0
            } else {
                (self.progress.completed_steps.len() as f32 / self.steps.len() as f32) * 100.0
            },
        }
    }
}

/// Tutorial completion statistics
#[derive(Debug, Clone)]
pub struct TutorialStats {
    pub total_steps: usize,
    pub completed_steps: usize,
    pub skipped_steps: usize,
    pub current_step: usize,
    pub total_time: std::time::Duration,
    pub hints_used: u32,
    pub completion_percentage: f32,
}

impl TutorialStats {
    pub fn is_completed(&self) -> bool {
        self.completed_steps + self.skipped_steps >= self.total_steps
    }
}