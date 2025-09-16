use crate::engine::{
    game_builder::GameBuilder,
    input::InputManager,
    ui::ElementId,
};
use std::time::{Duration, Instant};

/// Comprehensive UI system demonstration
/// Shows all UI components, animations, and interactions
pub struct UIDemo {
    game_builder: GameBuilder,
    demo_start_time: Instant,
    current_screen: DemoScreen,
    menu_elements: Vec<ElementId>,
    settings_elements: Vec<ElementId>,
    loading_elements: Vec<ElementId>,
    component_elements: Vec<ElementId>,
    loading_progress: f32,
}

#[derive(Debug, Clone, PartialEq)]
enum DemoScreen {
    MainMenu,
    ComponentShowcase,
    Settings,
    Loading,
}

impl UIDemo {
    pub fn new() -> Self {
        let mut game_builder = GameBuilder::new();
        
        // Set up initial UI
        game_builder.set_ui_scale(1.0);
        
        log::info!("UI Demo initialized - showcasing Robin's UI system");
        
        Self {
            game_builder,
            demo_start_time: Instant::now(),
            current_screen: DemoScreen::MainMenu,
            menu_elements: Vec::new(),
            settings_elements: Vec::new(),
            loading_elements: Vec::new(),
            component_elements: Vec::new(),
            loading_progress: 0.0,
        }
    }
    
    pub fn start_demo(&mut self) {
        log::info!("=== Robin UI System Demo ===");
        log::info!("Showcasing UI components, animations, and interactions");
        
        self.show_main_menu();
        
        // Create background ambiance
        self.game_builder.add_light(400.0, 300.0, (0.2, 0.3, 0.8), 0.3);
        self.game_builder.create_fog(400.0, 300.0);
        
        log::info!("Demo started - use mouse to interact with UI elements");
    }
    
    pub fn update(&mut self, input: &InputManager) {
        let elapsed = self.demo_start_time.elapsed().as_secs_f32();
        
        // Update all game systems
        let _animations = self.game_builder.update(1.0 / 60.0, input);
        
        match self.current_screen {
            DemoScreen::MainMenu => {
                self.update_main_menu(input, elapsed);
            }
            DemoScreen::ComponentShowcase => {
                self.update_component_showcase(input, elapsed);
            }
            DemoScreen::Settings => {
                self.update_settings(input, elapsed);
            }
            DemoScreen::Loading => {
                self.update_loading_screen(elapsed);
            }
        }
        
        // Auto-transition between screens for demo purposes
        self.handle_auto_transitions(elapsed);
    }
    
    fn show_main_menu(&mut self) {
        log::info!("Showing main menu with animated buttons");
        
        // Clear previous UI
        self.clear_all_ui();
        
        // Create main menu
        let buttons = vec![
            ("Start Game", 300.0, 200.0),
            ("Settings", 300.0, 270.0),
            ("Components", 300.0, 340.0),
            ("Exit", 300.0, 410.0),
        ];
        
        self.menu_elements = self.game_builder.create_simple_menu("Robin Engine UI Demo", &buttons);
        
        // Add magical effects
        for (i, &element_id) in self.menu_elements.iter().enumerate() {
            if i > 0 { // Skip title
                self.game_builder.pulse_ui(element_id, 2.0);
                
                // Add sparkle effects at button positions
                let x = 350.0;
                let y = 170.0 + (i as f32 - 1.0) * 70.0;
                self.game_builder.create_magic_trail(x, y);
            }
        }
        
        self.current_screen = DemoScreen::MainMenu;
        log::info!("Main menu created with {} animated elements", self.menu_elements.len());
    }
    
    fn show_component_showcase(&mut self) {
        log::info!("Showing component showcase");
        
        self.clear_all_ui();
        
        // Create title
        let title = self.game_builder.create_label(200.0, 50.0, 400.0, 60.0, "UI Components Showcase");
        self.component_elements.push(title);
        
        // Create various components in a grid layout
        let mut y_pos = 120.0;
        let spacing = 80.0;
        
        // Buttons section
        let button_label = self.game_builder.create_label(50.0, y_pos, 200.0, 30.0, "Buttons:");
        self.component_elements.push(button_label);
        
        let normal_button = self.game_builder.create_button(250.0, y_pos, 120.0, 40.0, "Normal");
        let magic_button = self.game_builder.create_button(380.0, y_pos, 120.0, 40.0, "Magic");
        let disabled_button = self.game_builder.create_button(510.0, y_pos, 120.0, 40.0, "Disabled");
        
        self.component_elements.extend(vec![normal_button, magic_button, disabled_button]);
        
        // Add effects to magic button
        self.game_builder.pulse_ui(magic_button, 1.5);
        self.game_builder.create_magic_trail(440.0, y_pos + 20.0);
        
        y_pos += spacing;
        
        // Sliders section
        let slider_label = self.game_builder.create_label(50.0, y_pos, 200.0, 30.0, "Sliders:");
        self.component_elements.push(slider_label);
        
        let volume_slider = self.game_builder.create_slider(250.0, y_pos, 200.0, 30.0, 0.0, 100.0);
        let brightness_slider = self.game_builder.create_slider(460.0, y_pos, 200.0, 30.0, 0.0, 1.0);
        
        self.component_elements.extend(vec![volume_slider, brightness_slider]);
        
        y_pos += spacing;
        
        // Progress bars section
        let progress_label = self.game_builder.create_label(50.0, y_pos, 200.0, 30.0, "Progress Bars:");
        self.component_elements.push(progress_label);
        
        let health_bar = self.game_builder.create_progress_bar(250.0, y_pos, 200.0, 20.0, 100.0);
        let mana_bar = self.game_builder.create_progress_bar(460.0, y_pos, 200.0, 20.0, 100.0);
        
        self.component_elements.extend(vec![health_bar, mana_bar]);
        
        y_pos += spacing;
        
        // Panels section
        let panel_label = self.game_builder.create_label(50.0, y_pos, 200.0, 30.0, "Panels:");
        self.component_elements.push(panel_label);
        
        let info_panel = self.game_builder.create_panel(250.0, y_pos, 180.0, 60.0);
        let tooltip_panel = self.game_builder.create_panel(440.0, y_pos, 180.0, 60.0);
        
        self.component_elements.extend(vec![info_panel, tooltip_panel]);
        
        // Back button
        let back_button = self.game_builder.create_button(50.0, 500.0, 100.0, 40.0, "Back");
        self.component_elements.push(back_button);
        
        // Animate all elements with staggered entrance
        for (i, &element_id) in self.component_elements.iter().enumerate() {
            self.game_builder.fade_in_ui(element_id, 0.3 + (i as f32) * 0.1);
            self.game_builder.slide_ui(element_id, -100.0, 0.0, 0.0, 0.0, 0.5 + (i as f32) * 0.05);
        }
        
        // Add atmospheric effects
        self.game_builder.create_magic_trail(100.0, 300.0);
        self.game_builder.create_magic_trail(700.0, 300.0);
        
        self.current_screen = DemoScreen::ComponentShowcase;
        log::info!("Component showcase created with {} elements", self.component_elements.len());
    }
    
    fn show_settings_panel(&mut self) {
        log::info!("Showing settings panel");
        
        self.clear_all_ui();
        
        // Create settings panel
        self.settings_elements = self.game_builder.create_settings_panel(250.0, 200.0);
        
        // Add title
        let title = self.game_builder.create_label(200.0, 100.0, 400.0, 60.0, "Game Settings");
        self.settings_elements.insert(0, title);
        
        // Add magical portal effect in background
        let portal_effects = self.game_builder.create_portal(400.0, 300.0);
        log::info!("Created portal with {} effects", portal_effects.len());
        
        self.current_screen = DemoScreen::Settings;
        log::info!("Settings panel created with {} elements", self.settings_elements.len());
    }
    
    fn show_loading_screen(&mut self) {
        log::info!("Showing loading screen demo");
        
        self.clear_all_ui();
        self.loading_progress = 0.0;
        
        // Create loading screen
        self.loading_elements = self.game_builder.create_loading_screen("Loading Game Assets...");
        
        // Add loading particles
        self.game_builder.create_magic_trail(400.0, 300.0);
        
        self.current_screen = DemoScreen::Loading;
        log::info!("Loading screen created with {} elements", self.loading_elements.len());
    }
    
    fn update_main_menu(&mut self, _input: &InputManager, elapsed: f32) {
        // Add floating animation to menu buttons
        if elapsed > 2.0 {
            for (i, &element_id) in self.menu_elements.iter().enumerate() {
                if i > 0 { // Skip title
                    let offset = (elapsed * 2.0 + i as f32).sin() * 5.0;
                    // This would need proper animation system integration
                    // For now, we just log the intended animation
                    if (elapsed as i32) % 5 == 0 && (elapsed * 10.0) as i32 % 10 == 0 {
                        log::debug!("Floating animation: button {} offset {:.1}", i, offset);
                    }
                }
            }
        }
    }
    
    fn update_component_showcase(&mut self, _input: &InputManager, elapsed: f32) {
        // Animate progress bars to show dynamic values
        if elapsed as i32 % 2 == 0 {
            let progress = ((elapsed * 2.0).sin() * 0.5 + 0.5) * 100.0;
            // This would update progress bar values in a real implementation
            if (elapsed * 10.0) as i32 % 30 == 0 {
                log::debug!("Progress animation: {:.1}%", progress);
            }
        }
    }
    
    fn update_settings(&mut self, _input: &InputManager, _elapsed: f32) {
        // Settings panel is mostly static, but could handle slider value changes here
    }
    
    fn update_loading_screen(&mut self, elapsed: f32) {
        // Simulate loading progress
        let loading_time = elapsed % 8.0; // Reset every 8 seconds
        self.loading_progress = (loading_time / 8.0 * 100.0).min(100.0);
        
        if (elapsed * 10.0) as i32 % 10 == 0 {
            log::debug!("Loading progress: {:.1}%", self.loading_progress);
        }
        
        // Change status text based on progress
        let status_text = match self.loading_progress as i32 {
            0..=20 => "Loading textures...",
            21..=40 => "Loading audio...",
            41..=60 => "Loading levels...",
            61..=80 => "Initializing systems...",
            81..=99 => "Finalizing...",
            _ => "Complete!",
        };
        
        // In a real implementation, we would update the status label text here
        if (elapsed * 5.0) as i32 % 10 == 0 {
            log::debug!("Status: {}", status_text);
        }
    }
    
    fn handle_auto_transitions(&mut self, elapsed: f32) {
        // Auto-transition between screens for demo purposes
        match self.current_screen {
            DemoScreen::MainMenu if elapsed > 8.0 => {
                self.show_component_showcase();
            }
            DemoScreen::ComponentShowcase if elapsed > 18.0 => {
                self.show_settings_panel();
            }
            DemoScreen::Settings if elapsed > 28.0 => {
                self.show_loading_screen();
            }
            DemoScreen::Loading if elapsed > 38.0 => {
                // Loop back to main menu
                self.demo_start_time = Instant::now();
                self.show_main_menu();
            }
            _ => {}
        }
    }
    
    fn clear_all_ui(&mut self) {
        // Clear all UI elements
        self.game_builder.clear_ui();
        self.menu_elements.clear();
        self.settings_elements.clear();
        self.loading_elements.clear();
        self.component_elements.clear();
    }
    
    /// Run the complete UI demo
    pub fn run_demo() {
        log::info!("Starting comprehensive UI system demo");
        
        let mut demo = UIDemo::new();
        demo.start_demo();
        
        // Create mock input manager for demo
        let input = InputManager::new();
        
        // Run demo for 45 seconds to show all screens
        let demo_duration = Duration::from_secs(45);
        let start_time = Instant::now();
        
        while start_time.elapsed() < demo_duration {
            demo.update(&input);
            
            // Sleep to maintain ~60 FPS for the demo
            std::thread::sleep(Duration::from_millis(16));
        }
        
        // Final statistics
        log::info!("=== UI Demo Complete ===");
        log::info!("Total UI elements created: {}", demo.game_builder.ui_manager.get_element_count());
        log::info!("Active particle effects: {}", demo.game_builder.get_particle_count());
        log::info!("Active lights: {}", demo.game_builder.get_light_count());
        log::info!("UI scale factor: {:.2}", demo.game_builder.ui_manager.get_ui_scale());
        
        // Demonstrate cleanup
        demo.clear_all_ui();
        log::info!("UI cleanup complete - all elements removed");
        
        log::info!("Robin UI system demo finished successfully!");
    }
}

impl Default for UIDemo {
    fn default() -> Self {
        Self::new()
    }
}

// Convenience function for running the demo
pub fn run_ui_demo() {
    UIDemo::run_demo();
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ui_demo_creation() {
        let demo = UIDemo::new();
        assert_eq!(demo.current_screen, DemoScreen::MainMenu);
        assert_eq!(demo.menu_elements.len(), 0);
    }
    
    #[test]
    fn test_screen_transitions() {
        let mut demo = UIDemo::new();
        
        // Test main menu creation
        demo.show_main_menu();
        assert_eq!(demo.current_screen, DemoScreen::MainMenu);
        assert!(!demo.menu_elements.is_empty());
        
        // Test component showcase
        demo.show_component_showcase();
        assert_eq!(demo.current_screen, DemoScreen::ComponentShowcase);
        assert!(!demo.component_elements.is_empty());
        
        // Test settings panel
        demo.show_settings_panel();
        assert_eq!(demo.current_screen, DemoScreen::Settings);
        assert!(!demo.settings_elements.is_empty());
        
        // Test loading screen
        demo.show_loading_screen();
        assert_eq!(demo.current_screen, DemoScreen::Loading);
        assert!(!demo.loading_elements.is_empty());
    }
    
    #[test]
    fn test_ui_cleanup() {
        let mut demo = UIDemo::new();
        
        // Create some UI elements
        demo.show_main_menu();
        assert!(!demo.menu_elements.is_empty());
        
        // Test cleanup
        demo.clear_all_ui();
        assert!(demo.menu_elements.is_empty());
        assert!(demo.component_elements.is_empty());
        assert!(demo.settings_elements.is_empty());
        assert!(demo.loading_elements.is_empty());
    }
}