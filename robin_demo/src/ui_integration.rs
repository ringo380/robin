/// UI Integration Module
///
/// Bridges the new production UI system with the existing robin_demo application

use crate::renderer::MetalRenderer;
use robin::engine::ui::{
    production_ui::{ProductionUISystem, UIConfig, UICommand, UIEvent},
    welcome_flow::{WelcomeFlow, DemoMode},
    dashboard::PerformanceDashboard,
    transitions::TransitionSystem,
};
use cgmath::{Vector2, Vector3};
use std::time::Instant;

pub struct IntegratedUI {
    production_ui: ProductionUISystem,
    welcome_flow: WelcomeFlow,
    dashboard: PerformanceDashboard,
    transition_system: TransitionSystem,

    // State
    show_welcome: bool,
    show_dashboard: bool,
    selected_mode: Option<DemoMode>,
    startup_time: Instant,
    last_frame_time: Instant,

    // Metrics
    frame_count: u64,
    total_time: f32,
}

impl IntegratedUI {
    pub fn new(window_size: (f32, f32)) -> Self {
        let config = UIConfig {
            window_size,
            theme: "dark".to_string(),
            scale_factor: 1.0,
            animations_enabled: true,
            vsync_enabled: true,
            max_fps: 120,
            font_size: 14.0,
            debug_mode: false,
        };

        Self {
            production_ui: ProductionUISystem::new(config.clone()),
            welcome_flow: WelcomeFlow::new(config.clone()),
            dashboard: PerformanceDashboard::new(config.clone()),
            transition_system: TransitionSystem::new(),

            show_welcome: true,
            show_dashboard: false,
            selected_mode: None,
            startup_time: Instant::now(),
            last_frame_time: Instant::now(),

            frame_count: 0,
            total_time: 0.0,
        }
    }

    pub fn handle_startup(&mut self) {
        // Initialize welcome flow
        self.welcome_flow.show_splash_screen();

        // Start fade-in animation
        self.transition_system.start_transition(
            "welcome_fade_in",
            robin::engine::ui::transitions::TransitionConfig {
                duration: 1.5,
                delay: 0.0,
                easing: robin::engine::ui::transitions::EasingFunction::EaseOutCubic,
                reverse: false,
                repeat: false,
                yoyo: false,
            },
        );
    }

    pub fn update(&mut self, delta_time: f32) {
        // Update frame metrics
        self.frame_count += 1;
        self.total_time += delta_time;

        let now = Instant::now();
        let frame_time = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;

        // Update all systems
        self.production_ui.update(delta_time);
        self.transition_system.update(delta_time);

        if self.show_welcome {
            self.welcome_flow.update(delta_time);

            // Check if splash is done
            if self.welcome_flow.is_splash_complete() && self.selected_mode.is_none() {
                self.welcome_flow.show_mode_selection();
            }

            // Check for mode selection
            if let Some(mode) = self.welcome_flow.get_selected_mode() {
                self.selected_mode = Some(mode);
                self.handle_mode_selection(mode);
            }
        }

        if self.show_dashboard {
            // Update performance metrics
            self.dashboard.update_fps(1.0 / frame_time);
            self.dashboard.update_frame_time(frame_time * 1000.0); // Convert to ms

            // Update memory (placeholder - would get from system)
            self.dashboard.update_memory_usage(
                self.get_memory_usage(),
                self.get_total_memory(),
            );

            self.dashboard.update(delta_time);
        }
    }

    pub fn render(&mut self, renderer: &mut MetalRenderer) {
        // Render based on current state
        if self.show_welcome && !self.welcome_flow.is_complete() {
            self.render_welcome(renderer);
        } else {
            self.render_main_ui(renderer);
        }

        // Always render dashboard if enabled
        if self.show_dashboard {
            self.render_dashboard(renderer);
        }
    }

    fn render_welcome(&mut self, renderer: &mut MetalRenderer) {
        // Get welcome flow UI components
        let components = self.welcome_flow.get_ui_components();

        // Render through production UI system
        for component in components {
            self.production_ui.render_component(renderer, component);
        }
    }

    fn render_main_ui(&mut self, renderer: &mut MetalRenderer) {
        // Render main UI elements
        self.production_ui.render(renderer);
    }

    fn render_dashboard(&mut self, renderer: &mut MetalRenderer) {
        // Get dashboard components
        let metrics = self.dashboard.get_render_data();

        // Render dashboard overlay
        self.production_ui.render_overlay(renderer, metrics);
    }

    pub fn handle_mouse_move(&mut self, x: f32, y: f32) {
        self.production_ui.handle_mouse_move(Vector2::new(x, y));

        if self.show_welcome {
            self.welcome_flow.handle_mouse_move(x, y);
        }
    }

    pub fn handle_mouse_click(&mut self, x: f32, y: f32, button: u8) {
        let event = UIEvent::MouseClick {
            position: Vector2::new(x, y),
            button,
        };

        self.production_ui.handle_event(event);

        if self.show_welcome {
            self.welcome_flow.handle_click(x, y);
        }
    }

    pub fn handle_key(&mut self, key: char, pressed: bool) {
        // Toggle dashboard with Tab
        if key == '\t' && pressed {
            self.toggle_dashboard();
        }

        // Pass to UI systems
        let event = UIEvent::KeyPress {
            key,
            modifiers: 0,
        };

        if pressed {
            self.production_ui.handle_event(event);
        }
    }

    pub fn toggle_dashboard(&mut self) {
        self.show_dashboard = !self.show_dashboard;

        if self.show_dashboard {
            // Animate dashboard in
            self.transition_system.start_transition(
                "dashboard_slide_in",
                robin::engine::ui::transitions::TransitionConfig {
                    duration: 0.3,
                    delay: 0.0,
                    easing: robin::engine::ui::transitions::EasingFunction::EaseOutBack,
                    reverse: false,
                    repeat: false,
                    yoyo: false,
                },
            );
        } else {
            // Animate dashboard out
            self.transition_system.start_transition(
                "dashboard_slide_out",
                robin::engine::ui::transitions::TransitionConfig {
                    duration: 0.2,
                    delay: 0.0,
                    easing: robin::engine::ui::transitions::EasingFunction::EaseInCubic,
                    reverse: false,
                    repeat: false,
                    yoyo: false,
                },
            );
        }
    }

    fn handle_mode_selection(&mut self, mode: DemoMode) {
        // Hide welcome after selection
        self.show_welcome = false;

        // Configure UI based on selected mode
        match mode {
            DemoMode::InteractivePlayground => {
                self.production_ui.enable_playground_mode();
            }
            DemoMode::EngineerBuildMode => {
                self.production_ui.enable_build_mode_ui();
            }
            DemoMode::GameplaySystems => {
                self.production_ui.enable_gameplay_ui();
            }
            DemoMode::CollaborationPreview => {
                self.production_ui.enable_collaboration_ui();
            }
            DemoMode::PerformanceBenchmark => {
                self.show_dashboard = true;
                self.production_ui.enable_benchmark_mode();
            }
            DemoMode::VisualShowcase => {
                self.production_ui.enable_showcase_mode();
            }
        }

        // Start transition to main UI
        self.transition_system.start_transition(
            "mode_transition",
            robin::engine::ui::transitions::TransitionConfig {
                duration: 0.5,
                delay: 0.0,
                easing: robin::engine::ui::transitions::EasingFunction::EaseInOutQuad,
                reverse: false,
                repeat: false,
                yoyo: false,
            },
        );
    }

    pub fn is_welcome_complete(&self) -> bool {
        !self.show_welcome || self.selected_mode.is_some()
    }

    pub fn get_selected_mode(&self) -> Option<DemoMode> {
        self.selected_mode
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.production_ui.resize((width, height));
        self.welcome_flow.resize(width, height);
        self.dashboard.resize(width, height);
    }

    // Utility methods
    fn get_memory_usage(&self) -> f64 {
        // Placeholder - would get actual memory usage
        // In production, use system APIs or memory allocator stats
        512.0 // MB
    }

    fn get_total_memory(&self) -> f64 {
        // Placeholder - would get total available memory
        8192.0 // MB
    }

    pub fn get_metrics(&self) -> UIMetrics {
        UIMetrics {
            fps: if self.total_time > 0.0 {
                self.frame_count as f32 / self.total_time
            } else {
                0.0
            },
            frame_time: if self.frame_count > 0 {
                (self.total_time * 1000.0) / self.frame_count as f32
            } else {
                0.0
            },
            ui_elements: self.production_ui.get_element_count(),
            animations_active: self.transition_system.get_active_count(),
            memory_ui: self.production_ui.get_memory_usage(),
        }
    }
}

pub struct UIMetrics {
    pub fps: f32,
    pub frame_time: f32,
    pub ui_elements: usize,
    pub animations_active: usize,
    pub memory_ui: usize,
}

// Helper functions for demo mode configuration
impl DemoMode {
    pub fn get_ui_config(&self) -> UIConfig {
        let base_config = UIConfig {
            window_size: (1200.0, 800.0),
            theme: "dark".to_string(),
            scale_factor: 1.0,
            animations_enabled: true,
            vsync_enabled: true,
            max_fps: 120,
            font_size: 14.0,
            debug_mode: false,
        };

        match self {
            DemoMode::InteractivePlayground => UIConfig {
                animations_enabled: true,
                debug_mode: true,
                ..base_config
            },
            DemoMode::EngineerBuildMode => UIConfig {
                font_size: 12.0,
                ..base_config
            },
            DemoMode::GameplaySystems => UIConfig {
                max_fps: 144,
                ..base_config
            },
            DemoMode::CollaborationPreview => UIConfig {
                theme: "team".to_string(),
                ..base_config
            },
            DemoMode::PerformanceBenchmark => UIConfig {
                debug_mode: true,
                max_fps: 0, // Unlimited
                ..base_config
            },
            DemoMode::VisualShowcase => UIConfig {
                animations_enabled: true,
                vsync_enabled: true,
                ..base_config
            },
        }
    }
}