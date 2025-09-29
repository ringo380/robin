/// Enhanced Welcome Flow UI for Robin Engine Demo
///
/// Provides visually polished onboarding experience with:
/// - Animated welcome screens with smooth transitions
/// - Professional branding and visual identity
/// - Interactive progress indicators and achievements
/// - Contextual help overlays and tooltips
/// - Responsive design with accessibility features
/// - Cinematic presentation for investor demonstrations

use imgui::*;
use crate::demo_state::{OnboardingSystem, OnboardingState, TutorialStep};
use std::time::Instant;

/// Visual themes for different presentation contexts
#[derive(Debug, Clone, PartialEq)]
pub enum WelcomePresentationMode {
    /// Standard user onboarding experience
    Standard,
    /// Professional demonstration for investors/stakeholders
    InvestorDemo,
    /// Technical showcase for developers
    TechnicalShowcase,
    /// Trade show/conference presentation
    TradeShow,
}

/// Enhanced welcome flow UI with visual polish
pub struct WelcomeFlowUI {
    /// Current presentation mode
    presentation_mode: WelcomePresentationMode,
    /// Animation and timing
    animation_time: f32,
    fade_alpha: f32,
    slide_offset: f32,
    /// Visual state
    show_welcome: bool,
    show_background_animation: bool,
    show_particle_effects: bool,
    /// Professional styling
    use_cinematic_mode: bool,
    use_dark_theme: bool,
    logo_scale: f32,
    text_scale: f32,
    /// Interactive elements
    progress_animation: f32,
    achievement_glow: f32,
    button_hover_scale: f32,
    /// Content state
    current_slide: usize,
    total_slides: usize,
    auto_advance_timer: f32,
    /// Accessibility
    high_contrast_mode: bool,
    large_text_mode: bool,
    screen_reader_mode: bool,
}

impl WelcomeFlowUI {
    pub fn new() -> Self {
        Self {
            presentation_mode: WelcomePresentationMode::Standard,
            animation_time: 0.0,
            fade_alpha: 0.0,
            slide_offset: 0.0,
            show_welcome: true,
            show_background_animation: true,
            show_particle_effects: true,
            use_cinematic_mode: false,
            use_dark_theme: true,
            logo_scale: 1.0,
            text_scale: 1.0,
            progress_animation: 0.0,
            achievement_glow: 0.0,
            button_hover_scale: 1.0,
            current_slide: 0,
            total_slides: 6, // Number of onboarding states
            auto_advance_timer: 0.0,
            high_contrast_mode: false,
            large_text_mode: false,
            screen_reader_mode: false,
        }
    }

    /// Set presentation mode for different contexts
    pub fn set_presentation_mode(&mut self, mode: WelcomePresentationMode) {
        self.presentation_mode = mode;

        // Adjust settings based on presentation mode
        match mode {
            WelcomePresentationMode::InvestorDemo => {
                self.use_cinematic_mode = true;
                self.show_particle_effects = true;
                self.auto_advance_timer = 0.0; // Manual advancement for demo control
            }
            WelcomePresentationMode::TechnicalShowcase => {
                self.use_cinematic_mode = false;
                self.show_particle_effects = true;
                self.text_scale = 1.1; // Slightly larger text for readability
            }
            WelcomePresentationMode::TradeShow => {
                self.use_cinematic_mode = true;
                self.show_particle_effects = true;
                self.logo_scale = 1.2; // Larger branding
                self.auto_advance_timer = 8.0; // Auto-advance for unattended demo
            }
            WelcomePresentationMode::Standard => {
                self.use_cinematic_mode = false;
                self.show_particle_effects = false;
                self.auto_advance_timer = 0.0; // User-controlled
            }
        }
    }

    /// Update animations and transitions
    pub fn update(&mut self, delta_time: f32) {
        self.animation_time += delta_time;

        // Smooth fade-in animation
        if self.show_welcome {
            self.fade_alpha = (self.fade_alpha + delta_time * 2.0).min(1.0);
        } else {
            self.fade_alpha = (self.fade_alpha - delta_time * 3.0).max(0.0);
        }

        // Slide animation for smooth transitions
        let target_offset = if self.show_welcome { 0.0 } else { -50.0 };
        self.slide_offset += (target_offset - self.slide_offset) * delta_time * 5.0;

        // Progress animation (heartbeat effect)
        self.progress_animation = (self.animation_time * 2.0).sin() * 0.1 + 1.0;

        // Achievement glow effect
        self.achievement_glow = ((self.animation_time * 3.0).sin() * 0.5 + 0.5) * 0.3 + 0.7;

        // Auto-advance timer
        if self.auto_advance_timer > 0.0 {
            self.auto_advance_timer -= delta_time;
        }
    }

    /// Render the enhanced welcome flow
    pub fn render(&mut self, ui: &Ui, onboarding: &OnboardingSystem) -> Vec<WelcomeFlowAction> {
        let mut actions = Vec::new();

        if !self.show_welcome || self.fade_alpha < 0.01 {
            return actions;
        }

        // Apply professional styling
        self.apply_professional_styling(ui);

        // Main welcome overlay (full screen)
        let screen_size = ui.io().display_size;
        ui.set_next_window_pos([0.0, 0.0], Condition::Always);
        ui.set_next_window_size(screen_size, Condition::Always);

        let mut welcome_open = self.show_welcome;

        if let Some(_window) = ui.window("##WelcomeFlow")
            .opened(&mut welcome_open)
            .no_title_bar(true)
            .no_resize(true)
            .no_move(true)
            .no_scrollbar(true)
            .no_collapse(true)
            .no_decoration(true)
            .draw_background(false)
            .begin()
        {
            // Render based on current onboarding state
            match onboarding.get_current_state() {
                OnboardingState::WelcomeScreen => {
                    actions.extend(self.render_welcome_screen(ui));
                }
                OnboardingState::EngineOverview => {
                    actions.extend(self.render_engine_overview(ui));
                }
                OnboardingState::ControlsTutorial => {
                    actions.extend(self.render_controls_tutorial(ui));
                }
                OnboardingState::DemoModesIntro => {
                    actions.extend(self.render_demo_modes_intro(ui));
                }
                OnboardingState::FirstBuildExperience => {
                    actions.extend(self.render_first_build_experience(ui));
                }
                OnboardingState::AchievementShowcase => {
                    actions.extend(self.render_achievement_showcase(ui, onboarding));
                }
                OnboardingState::Completed => {
                    self.show_welcome = false;
                }
            }

            // Render progress indicator
            self.render_progress_indicator(ui, onboarding);

            // Render professional branding
            self.render_branding(ui);

            // Render accessibility options if needed
            if self.screen_reader_mode {
                self.render_accessibility_info(ui, onboarding);
            }
        }

        self.show_welcome = welcome_open;
        actions
    }

    /// Apply professional styling theme
    fn apply_professional_styling(&self, ui: &Ui) {
        let style = ui.style_mut();

        if self.use_dark_theme {
            // Professional dark theme
            style.colors[StyleColor::WindowBg as usize] = [0.06, 0.06, 0.07, 0.95];
            style.colors[StyleColor::Text as usize] = [0.95, 0.95, 0.96, self.fade_alpha];
            style.colors[StyleColor::Button as usize] = [0.15, 0.15, 0.16, 0.9];
            style.colors[StyleColor::ButtonHovered as usize] = [0.20, 0.20, 0.22, 0.9];
            style.colors[StyleColor::ButtonActive as usize] = [0.25, 0.25, 0.27, 0.9];
        }

        // Enhanced typography
        style.frame_rounding = 8.0;
        style.window_rounding = 12.0;
        style.grab_rounding = 6.0;
        style.window_padding = [20.0, 20.0];
        style.frame_padding = [12.0, 8.0];
        style.item_spacing = [12.0, 8.0];
    }

    /// Render professional welcome screen
    fn render_welcome_screen(&mut self, ui: &Ui) -> Vec<WelcomeFlowAction> {
        let mut actions = Vec::new();
        let screen_size = ui.io().display_size;

        // Center content with professional layout
        let content_width = 800.0;
        let content_height = 600.0;
        let start_x = (screen_size[0] - content_width) * 0.5;
        let start_y = (screen_size[1] - content_height) * 0.5 + self.slide_offset;

        ui.set_cursor_pos([start_x, start_y]);

        ui.begin_group();

        // Animated logo area
        self.render_animated_logo(ui);

        ui.spacing();
        ui.spacing();

        // Professional title with animation
        let title_color = [0.4, 0.7, 1.0, self.fade_alpha];
        ui.push_style_color(StyleColor::Text, title_color);
        ui.text("ROBIN ENGINE");
        ui.pop_style_color();

        ui.same_line();
        ui.text_colored([0.8, 0.8, 0.8, self.fade_alpha], "v1.0");

        ui.spacing();

        // Professional subtitle
        ui.text_colored([0.9, 0.9, 0.9, self.fade_alpha], "Professional 3D Voxel Game Engine");
        ui.text_colored([0.7, 0.7, 0.7, self.fade_alpha], "Built from scratch in Rust • Apple Silicon optimized • Metal rendering");

        ui.spacing();
        ui.spacing();

        // Feature highlights with icons
        ui.text_colored([0.0, 1.0, 0.5, self.fade_alpha], "✓ 92% frustum culling efficiency");
        ui.text_colored([0.0, 1.0, 0.5, self.fade_alpha], "✓ 60-80% vertex reduction via greedy meshing");
        ui.text_colored([0.0, 1.0, 0.5, self.fade_alpha], "✓ Real-time physics and particle effects");
        ui.text_colored([0.0, 1.0, 0.5, self.fade_alpha], "✓ Advanced AI and ML integration");

        ui.spacing();
        ui.spacing();

        // Professional call-to-action based on presentation mode
        match self.presentation_mode {
            WelcomePresentationMode::InvestorDemo => {
                ui.text_colored([1.0, 0.8, 0.0, self.fade_alpha], "🌟 Investment-Ready Technology Demonstration");
                ui.text_colored([0.8, 0.8, 0.8, self.fade_alpha], "Click anywhere to begin the investor showcase");
            }
            WelcomePresentationMode::TechnicalShowcase => {
                ui.text_colored([0.5, 0.8, 1.0, self.fade_alpha], "🔧 Technical Deep Dive");
                ui.text_colored([0.8, 0.8, 0.8, self.fade_alpha], "Explore the engineering excellence behind Robin Engine");
            }
            WelcomePresentationMode::TradeShow => {
                ui.text_colored([1.0, 0.5, 0.0, self.fade_alpha], "🎮 Live Interactive Demo");
                ui.text_colored([0.8, 0.8, 0.8, self.fade_alpha], "Experience next-generation voxel gaming technology");
            }
            WelcomePresentationMode::Standard => {
                ui.text_colored([0.7, 0.9, 1.0, self.fade_alpha], "🚀 Welcome to the Future of Voxel Gaming");
                ui.text_colored([0.8, 0.8, 0.8, self.fade_alpha], "Click anywhere to begin your journey");
            }
        }

        ui.spacing();
        ui.spacing();

        // Interactive advancement
        if ui.is_window_hovered() && ui.is_mouse_clicked(MouseButton::Left) {
            actions.push(WelcomeFlowAction::AdvanceToNextStep);
        }

        // Skip option for experienced users
        if ui.button("Skip Introduction") {
            actions.push(WelcomeFlowAction::SkipOnboarding);
        }

        ui.end_group();

        actions
    }

    /// Render animated logo with professional effects
    fn render_animated_logo(&self, ui: &Ui) {
        // Animated logo placeholder (would be actual logo in production)
        let logo_size = 80.0 * self.logo_scale;
        let logo_color = [0.4, 0.7, 1.0, self.fade_alpha];

        // Logo background with glow effect
        let glow_alpha = (self.animation_time * 2.0).sin() * 0.1 + 0.9;
        ui.text_colored([logo_color[0], logo_color[1], logo_color[2], glow_alpha * self.fade_alpha], "🏗️");
    }

    /// Render engine overview with technical highlights
    fn render_engine_overview(&mut self, ui: &Ui) -> Vec<WelcomeFlowAction> {
        let mut actions = Vec::new();
        let screen_size = ui.io().display_size;

        // Professional technical overview layout
        let content_width = 900.0;
        let start_x = (screen_size[0] - content_width) * 0.5;
        let start_y = 100.0 + self.slide_offset;

        ui.set_cursor_pos([start_x, start_y]);

        ui.begin_group();

        // Technical overview header
        ui.text_colored([0.4, 0.8, 1.0, self.fade_alpha], "🔧 TECHNICAL OVERVIEW");
        ui.separator();
        ui.spacing();

        // Architecture highlights
        ui.columns(2, "tech_overview", true);

        // Core Technology
        ui.text_colored([0.0, 1.0, 0.8, self.fade_alpha], "CORE TECHNOLOGY:");
        ui.text_colored([0.9, 0.9, 0.9, self.fade_alpha], "• Rust programming language");
        ui.text_colored([0.9, 0.9, 0.9, self.fade_alpha], "• WebGPU rendering pipeline");
        ui.text_colored([0.9, 0.9, 0.9, self.fade_alpha], "• Apple Metal optimization");
        ui.text_colored([0.9, 0.9, 0.9, self.fade_alpha], "• Multi-threaded architecture");

        ui.spacing();

        // Performance Features
        ui.text_colored([0.0, 1.0, 0.5, self.fade_alpha], "PERFORMANCE FEATURES:");
        ui.text_colored([0.9, 0.9, 0.9, self.fade_alpha], "• 92% frustum culling efficiency");
        ui.text_colored([0.9, 0.9, 0.9, self.fade_alpha], "• 60-80% vertex reduction");
        ui.text_colored([0.9, 0.9, 0.9, self.fade_alpha], "• Real-time LOD system");
        ui.text_colored([0.9, 0.9, 0.9, self.fade_alpha], "• GPU-accelerated physics");

        ui.next_column();

        // Advanced Features
        ui.text_colored([1.0, 0.8, 0.0, self.fade_alpha], "ADVANCED FEATURES:");
        ui.text_colored([0.9, 0.9, 0.9, self.fade_alpha], "• AI-powered content generation");
        ui.text_colored([0.9, 0.9, 0.9, self.fade_alpha], "• Machine learning integration");
        ui.text_colored([0.9, 0.9, 0.9, self.fade_alpha], "• Procedural world generation");
        ui.text_colored([0.9, 0.9, 0.9, self.fade_alpha], "• Advanced particle systems");

        ui.spacing();

        // Platform Support
        ui.text_colored([1.0, 0.5, 1.0, self.fade_alpha], "PLATFORM SUPPORT:");
        ui.text_colored([0.9, 0.9, 0.9, self.fade_alpha], "• macOS (optimized)");
        ui.text_colored([0.9, 0.9, 0.9, self.fade_alpha], "• Windows (planned)");
        ui.text_colored([0.9, 0.9, 0.9, self.fade_alpha], "• Web (WebAssembly)");
        ui.text_colored([0.9, 0.9, 0.9, self.fade_alpha], "• Mobile (future)");

        ui.columns(1, "", false);
        ui.spacing();

        // Auto-advance or click to continue
        ui.text_colored([0.6, 0.6, 0.6, self.fade_alpha], "Advancing automatically...");

        if ui.is_window_hovered() && ui.is_mouse_clicked(MouseButton::Left) {
            actions.push(WelcomeFlowAction::AdvanceToNextStep);
        }

        ui.end_group();

        actions
    }

    /// Render interactive controls tutorial
    fn render_controls_tutorial(&mut self, ui: &Ui) -> Vec<WelcomeFlowAction> {
        let mut actions = Vec::new();

        // Interactive controls tutorial with visual keyboard layout
        let screen_size = ui.io().display_size;
        let start_x = (screen_size[0] - 700.0) * 0.5;
        let start_y = 150.0 + self.slide_offset;

        ui.set_cursor_pos([start_x, start_y]);

        ui.begin_group();

        ui.text_colored([0.4, 1.0, 0.4, self.fade_alpha], "🎮 INTERACTIVE CONTROLS");
        ui.separator();
        ui.spacing();

        // Visual control layout
        ui.text_colored([1.0, 1.0, 0.0, self.fade_alpha], "MOVEMENT CONTROLS:");
        ui.text("┌─────┬─────┬─────┬─────┐");
        ui.text("│     │  W  │     │     │ ← Forward");
        ui.text("├─────┼─────┼─────┼─────┤");
        ui.text("│  A  │  S  │  D  │     │ ← Left/Back/Right");
        ui.text("└─────┴─────┴─────┴─────┘");

        ui.spacing();

        ui.text_colored([0.0, 1.0, 1.0, self.fade_alpha], "INTERACTION CONTROLS:");
        ui.text("🖱️  Mouse Movement    → Look around");
        ui.text("🖱️  Left Click       → Place blocks");
        ui.text("🖱️  Right Click      → Remove blocks");
        ui.text("⌨️  SPACE            → Move up");
        ui.text("⌨️  SHIFT            → Move down");

        ui.spacing();

        ui.text_colored([1.0, 0.5, 1.0, self.fade_alpha], "DEMO MODES (F1-F6):");
        ui.text("F1 → Interactive Playground");
        ui.text("F2 → Engineer Build Showcase");
        ui.text("F3 → Gameplay Systems Demo");
        ui.text("F4 → Collaboration Preview");
        ui.text("F5 → Performance Benchmarks");
        ui.text("F6 → Visual Showcase");

        ui.spacing();
        ui.text_colored([0.8, 0.8, 0.0, self.fade_alpha], "🎯 Try using any control to continue!");

        ui.end_group();

        actions
    }

    /// Render demo modes introduction
    fn render_demo_modes_intro(&mut self, ui: &Ui) -> Vec<WelcomeFlowAction> {
        let mut actions = Vec::new();

        // Demo modes showcase with visual previews
        let screen_size = ui.io().display_size;
        let start_x = (screen_size[0] - 800.0) * 0.5;
        let start_y = 120.0 + self.slide_offset;

        ui.set_cursor_pos([start_x, start_y]);

        ui.begin_group();

        ui.text_colored([1.0, 0.7, 0.0, self.fade_alpha], "🎭 DEMONSTRATION MODES");
        ui.separator();
        ui.spacing();

        // Mode cards layout
        ui.columns(2, "demo_modes", true);

        // Interactive Playground
        ui.text_colored([0.0, 1.0, 0.5, self.fade_alpha], "🎮 INTERACTIVE PLAYGROUND (F1)");
        ui.text("Experience free-form building and exploration");
        ui.text("• Physics-based voxel interactions");
        ui.text("• Real-time particle effects");
        ui.text("• Unlimited creative potential");

        ui.spacing();

        // Engineer Build Showcase
        ui.text_colored([0.0, 0.8, 1.0, self.fade_alpha], "🔧 ENGINEER BUILD SHOWCASE (F2)");
        ui.text("Advanced construction tools and templates");
        ui.text("• Professional building modes");
        ui.text("• Architectural templates");
        ui.text("• Precision placement tools");

        ui.spacing();

        // Gameplay Systems Demo
        ui.text_colored([1.0, 0.8, 0.0, self.fade_alpha], "⚡ GAMEPLAY SYSTEMS DEMO (F3)");
        ui.text("Resource management and progression");
        ui.text("• Crafting and progression systems");
        ui.text("• Dynamic world events");
        ui.text("• Player achievement tracking");

        ui.next_column();

        // Collaboration Preview
        ui.text_colored([1.0, 0.0, 0.8, self.fade_alpha], "👥 COLLABORATION PREVIEW (F4)");
        ui.text("Multiplayer building and teamwork");
        ui.text("• Real-time collaboration");
        ui.text("• Shared world building");
        ui.text("• Community features");

        ui.spacing();

        // Performance Benchmarks
        ui.text_colored([0.8, 0.0, 1.0, self.fade_alpha], "📊 PERFORMANCE BENCHMARKS (F5)");
        ui.text("Technical performance showcase");
        ui.text("• Real-time performance metrics");
        ui.text("• Optimization demonstrations");
        ui.text("• Technical deep dive");

        ui.spacing();

        // Visual Showcase
        ui.text_colored([1.0, 0.0, 0.0, self.fade_alpha], "🎨 VISUAL SHOWCASE (F6)");
        ui.text("Stunning graphics and effects");
        ui.text("• Dynamic lighting systems");
        ui.text("• Advanced shader effects");
        ui.text("• Atmospheric rendering");

        ui.columns(1, "", false);
        ui.spacing();

        ui.text_colored([0.0, 1.0, 1.0, self.fade_alpha], "🎯 Press F1 or F2 to explore your first demo mode!");

        ui.end_group();

        actions
    }

    /// Render first build experience tutorial
    fn render_first_build_experience(&mut self, ui: &Ui) -> Vec<WelcomeFlowAction> {
        let mut actions = Vec::new();

        // Interactive building tutorial
        let screen_size = ui.io().display_size;
        let start_x = (screen_size[0] - 600.0) * 0.5;
        let start_y = 200.0 + self.slide_offset;

        ui.set_cursor_pos([start_x, start_y]);

        ui.begin_group();

        ui.text_colored([0.0, 1.0, 0.0, self.fade_alpha], "🏗️ YOUR FIRST BUILD");
        ui.separator();
        ui.spacing();

        // Step-by-step building guide
        ui.text_colored([1.0, 1.0, 0.0, self.fade_alpha], "BUILDING BASICS:");
        ui.text("1. 🖱️ Left Click  → Place blocks");
        ui.text("2. 🖱️ Right Click → Remove blocks");
        ui.text("3. ⌨️ B          → Cycle building modes");
        ui.text("4. ⌨️ T          → Change templates");
        ui.text("5. ⌨️ G          → Toggle grid snap");

        ui.spacing();

        // Visual building preview
        ui.text_colored([0.0, 0.8, 1.0, self.fade_alpha], "WATCH FOR EFFECTS:");
        ui.text("• ✨ Particle effects on block placement");
        ui.text("• 🎆 Debris effects on block removal");
        ui.text("• 🔊 Audio feedback for interactions");
        ui.text("• 📊 Real-time performance metrics");

        ui.spacing();

        ui.text_colored([1.0, 0.8, 0.0, self.fade_alpha], "🎯 Complete 5 building actions to continue!");

        ui.end_group();

        actions
    }

    /// Render achievement showcase
    fn render_achievement_showcase(&mut self, ui: &Ui, onboarding: &OnboardingSystem) -> Vec<WelcomeFlowAction> {
        let mut actions = Vec::new();

        // Animated achievement celebration
        let screen_size = ui.io().display_size;
        let start_x = (screen_size[0] - 700.0) * 0.5;
        let start_y = 150.0 + self.slide_offset;

        ui.set_cursor_pos([start_x, start_y]);

        ui.begin_group();

        // Animated celebration header
        let celebration_color = [1.0, 0.8, 0.0, self.achievement_glow * self.fade_alpha];
        ui.text_colored(celebration_color, "🎉 ACHIEVEMENTS UNLOCKED!");
        ui.separator();
        ui.spacing();

        // Achievement badges with glow effects
        ui.text_colored([0.0, 1.0, 0.0, self.achievement_glow], "🏆 FIRST BUILDER");
        ui.text("You've mastered the basic building controls!");
        ui.spacing();

        ui.text_colored([0.0, 0.8, 1.0, self.achievement_glow], "🎯 EXPLORER");
        ui.text("You've discovered multiple demo modes!");
        ui.spacing();

        ui.text_colored([1.0, 0.0, 1.0, self.achievement_glow], "⭐ ENGINE EXPERT");
        ui.text("You understand Robin's capabilities!");
        ui.spacing();

        // Professional completion certificate
        if let Some(certificate) = onboarding.get_completion_certificate() {
            ui.text_colored([1.0, 1.0, 0.0, self.fade_alpha], "📜 CERTIFICATION:");
            ui.text_wrapped(&certificate);
        }

        ui.spacing();

        // Professional completion message
        match self.presentation_mode {
            WelcomePresentationMode::InvestorDemo => {
                ui.text_colored([1.0, 0.8, 0.0, self.fade_alpha], "🌟 READY FOR INVESTMENT DISCUSSIONS");
                ui.text("You've experienced Robin Engine's professional capabilities");
            }
            _ => {
                ui.text_colored([0.0, 1.0, 0.8, self.fade_alpha], "🚀 READY FOR ADVANCED FEATURES");
                ui.text("You're now ready to explore Robin Engine's full potential");
            }
        }

        ui.spacing();

        if ui.button("Continue to Full Experience") {
            actions.push(WelcomeFlowAction::CompleteOnboarding);
        }

        ui.end_group();

        actions
    }

    /// Render progress indicator
    fn render_progress_indicator(&self, ui: &Ui, onboarding: &OnboardingSystem) {
        let screen_size = ui.io().display_size;
        let indicator_width = 400.0;
        let start_x = (screen_size[0] - indicator_width) * 0.5;
        let start_y = screen_size[1] - 100.0;

        ui.set_cursor_pos([start_x, start_y]);

        // Calculate progress based on onboarding state
        let progress = match onboarding.get_current_state() {
            OnboardingState::WelcomeScreen => 0.0,
            OnboardingState::EngineOverview => 0.167,
            OnboardingState::ControlsTutorial => 0.333,
            OnboardingState::DemoModesIntro => 0.5,
            OnboardingState::FirstBuildExperience => 0.667,
            OnboardingState::AchievementShowcase => 0.833,
            OnboardingState::Completed => 1.0,
        };

        // Animated progress bar
        let animated_progress = progress * self.progress_animation;
        ui.progress_bar(animated_progress)
            .size([indicator_width, 20.0])
            .overlay_text(format!("Welcome Flow Progress: {:.0}%", progress * 100.0))
            .build();

        // Progress steps
        ui.text_colored([0.6, 0.6, 0.6, self.fade_alpha], onboarding.get_progress_summary());
    }

    /// Render professional branding
    fn render_branding(&self, ui: &Ui) {
        let screen_size = ui.io().display_size;

        // Robin Engine branding in corner
        ui.set_cursor_pos([screen_size[0] - 200.0, 20.0]);
        ui.text_colored([0.4, 0.7, 1.0, self.fade_alpha * 0.7], "Robin Engine v1.0");
        ui.text_colored([0.6, 0.6, 0.6, self.fade_alpha * 0.7], "Professional Demo");
    }

    /// Render accessibility information
    fn render_accessibility_info(&self, ui: &Ui, onboarding: &OnboardingSystem) {
        let screen_size = ui.io().display_size;
        ui.set_cursor_pos([20.0, screen_size[1] - 150.0]);

        ui.begin_group();
        ui.text_colored([0.8, 0.8, 0.0, self.fade_alpha], "ACCESSIBILITY:");

        if let Some(step) = onboarding.get_current_tutorial_step() {
            ui.text_wrapped(&format!("Current: {}", step.title));
            ui.text_wrapped(&format!("Instructions: {}", step.instructions));
        }

        ui.end_group();
    }

    /// Show/hide welcome flow
    pub fn set_visible(&mut self, visible: bool) {
        self.show_welcome = visible;
    }

    /// Check if welcome flow is visible
    pub fn is_visible(&self) -> bool {
        self.show_welcome && self.fade_alpha > 0.1
    }
}

/// Actions that can be triggered from the welcome flow
#[derive(Debug, Clone)]
pub enum WelcomeFlowAction {
    /// Advance to the next onboarding step
    AdvanceToNextStep,
    /// Skip the entire onboarding process
    SkipOnboarding,
    /// Complete onboarding and enter normal operation
    CompleteOnboarding,
    /// Change presentation mode
    SetPresentationMode(WelcomePresentationMode),
    /// Toggle accessibility features
    ToggleAccessibility,
}