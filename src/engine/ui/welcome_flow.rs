use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{Instant, Duration};
use nalgebra::{Vector2, Vector3, Vector4};
use crate::engine::error::RobinResult;
use crate::engine::ui::production_ui::{
    UIComponent, ComponentType, PanelComponent, ButtonComponent, TextComponent,
    ButtonState, Icon, BuiltinIcon, Rect, Color, TextAlignment,
};
use crate::engine::ui::transitions::{TransitionSystem, SlideDirection};

/// Welcome Flow and Mode Selection System
#[derive(Debug)]
pub struct WelcomeFlow {
    pub mode_selector: ModeSelector,
    pub onboarding_manager: OnboardingManager,
    pub tutorial_overlay: TutorialOverlay,
    pub achievement_tracker: AchievementTracker,
    pub demo_previewer: DemoPrevier,
    pub controls_guide: ControlsGuide,
    pub progress_tracker: ProgressTracker,
    pub intro_animator: IntroAnimator,
    config: WelcomeConfig,
    current_state: WelcomeState,
}

#[derive(Debug, Clone)]
pub struct WelcomeConfig {
    pub show_splash_screen: bool,
    pub splash_duration: Duration,
    pub enable_onboarding: bool,
    pub show_mode_previews: bool,
    pub animate_transitions: bool,
    pub show_achievements: bool,
    pub auto_select_last_mode: bool,
    pub tutorial_enabled: bool,
    pub first_time_user: bool,
}

impl Default for WelcomeConfig {
    fn default() -> Self {
        Self {
            show_splash_screen: true,
            splash_duration: Duration::from_secs(3),
            enable_onboarding: true,
            show_mode_previews: true,
            animate_transitions: true,
            show_achievements: true,
            auto_select_last_mode: false,
            tutorial_enabled: true,
            first_time_user: true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum WelcomeState {
    SplashScreen,
    ModeSelection,
    Onboarding,
    Tutorial,
    Loading,
    Active,
}

impl WelcomeFlow {
    pub fn new(config: WelcomeConfig) -> RobinResult<Self> {
        let mode_selector = ModeSelector::new(&config);
        let onboarding_manager = OnboardingManager::new(&config);
        let tutorial_overlay = TutorialOverlay::new();
        let achievement_tracker = AchievementTracker::new();
        let demo_previewer = DemoPrevier::new();
        let controls_guide = ControlsGuide::new();
        let progress_tracker = ProgressTracker::new();
        let intro_animator = IntroAnimator::new();

        Ok(Self {
            mode_selector,
            onboarding_manager,
            tutorial_overlay,
            achievement_tracker,
            demo_previewer,
            controls_guide,
            progress_tracker,
            intro_animator,
            config,
            current_state: WelcomeState::SplashScreen,
        })
    }

    pub fn start(&mut self, transition_system: &mut TransitionSystem) -> RobinResult<()> {
        if self.config.show_splash_screen {
            self.show_splash_screen(transition_system)?;
        } else {
            self.current_state = WelcomeState::ModeSelection;
            self.show_mode_selection(transition_system)?;
        }

        Ok(())
    }

    fn show_splash_screen(&mut self, transition_system: &mut TransitionSystem) -> RobinResult<()> {
        self.current_state = WelcomeState::SplashScreen;

        // Animate splash screen elements
        transition_system.fade_in("splash_background", Duration::from_millis(500))?;
        transition_system.slide_in("robin_logo", SlideDirection::Top, Duration::from_millis(800))?;
        transition_system.fade_in("tagline", Duration::from_millis(1200))?;
        transition_system.glow_pulse("start_button", 1.0, Duration::from_millis(2000))?;

        Ok(())
    }

    fn show_mode_selection(&mut self, transition_system: &mut TransitionSystem) -> RobinResult<()> {
        self.current_state = WelcomeState::ModeSelection;

        // Animate mode cards entrance
        for (i, mode) in DemoMode::all().iter().enumerate() {
            let delay = Duration::from_millis(i as u64 * 100);
            let card_id = format!("mode_card_{}", mode.id());

            transition_system.slide_in(
                &card_id,
                SlideDirection::Bottom,
                Duration::from_millis(500) + delay,
            )?;

            if self.config.show_mode_previews {
                transition_system.fade_in(
                    &format!("{}_preview", card_id),
                    Duration::from_millis(800) + delay,
                )?;
            }
        }

        Ok(())
    }

    pub fn create_splash_components(&self) -> Vec<UIComponent> {
        let mut components = Vec::new();

        // Background panel
        let background = UIComponent::new_panel(
            "splash_background",
            Rect::new(0.0, 0.0, 1920.0, 1080.0),
            PanelComponent {
                glass_effect: false,
                blur_intensity: 0.0,
                cast_shadow: false,
                show_border: false,
            },
        );
        components.push(background);

        // Robin logo
        let logo = UIComponent::new_text(
            "robin_logo",
            Rect::new(960.0, 350.0, 0.0, 0.0),
            TextComponent {
                content: "ROBIN ENGINE".to_string(),
                font_size: 72.0,
                color: Some(Color::white()),
                alignment: TextAlignment::Center,
                drop_shadow: true,
            },
        );
        components.push(logo);

        // Version text
        let version = UIComponent::new_text(
            "version",
            Rect::new(960.0, 430.0, 0.0, 0.0),
            TextComponent {
                content: "v4.0.0 Production Ready".to_string(),
                font_size: 18.0,
                color: Some(Color::gray(0.7)),
                alignment: TextAlignment::Center,
                drop_shadow: false,
            },
        );
        components.push(version);

        // Tagline
        let tagline = UIComponent::new_text(
            "tagline",
            Rect::new(960.0, 480.0, 0.0, 0.0),
            TextComponent {
                content: "Next-Generation Voxel Game Engine".to_string(),
                font_size: 24.0,
                color: Some(Color::gray(0.9)),
                alignment: TextAlignment::Center,
                drop_shadow: true,
            },
        );
        components.push(tagline);

        // Start button
        let start_button = UIComponent::new_button(
            "start_button",
            Rect::new(810.0, 600.0, 300.0, 60.0),
            ButtonComponent {
                label: "START DEMO".to_string(),
                icon: Some(Icon::Builtin(BuiltinIcon::Play)),
                state: ButtonState::Normal,
                elevated: true,
                ripple_effect: None,
                font_size: Some(20.0),
            },
        );
        components.push(start_button);

        // Loading bar (initially hidden)
        let loading_bar = UIComponent::new_panel(
            "loading_bar",
            Rect::new(760.0, 700.0, 400.0, 4.0),
            PanelComponent {
                glass_effect: false,
                blur_intensity: 0.0,
                cast_shadow: false,
                show_border: false,
            },
        );
        components.push(loading_bar);

        components
    }

    pub fn create_mode_selection_components(&self) -> Vec<UIComponent> {
        let mut components = Vec::new();

        // Title
        let title = UIComponent::new_text(
            "mode_selection_title",
            Rect::new(960.0, 80.0, 0.0, 0.0),
            TextComponent {
                content: "Select Demo Mode".to_string(),
                font_size: 36.0,
                color: Some(Color::white()),
                alignment: TextAlignment::Center,
                drop_shadow: true,
            },
        );
        components.push(title);

        // Subtitle
        let subtitle = UIComponent::new_text(
            "mode_selection_subtitle",
            Rect::new(960.0, 120.0, 0.0, 0.0),
            TextComponent {
                content: "Choose a mode to explore Robin Engine's capabilities".to_string(),
                font_size: 18.0,
                color: Some(Color::gray(0.8)),
                alignment: TextAlignment::Center,
                drop_shadow: false,
            },
        );
        components.push(subtitle);

        // Create mode cards
        let modes = DemoMode::all();
        let cards_per_row = 3;
        let card_width = 360.0;
        let card_height = 280.0;
        let card_spacing = 40.0;

        for (i, mode) in modes.iter().enumerate() {
            let row = i / cards_per_row;
            let col = i % cards_per_row;

            let x = 480.0 + (col as f32) * (card_width + card_spacing);
            let y = 200.0 + (row as f32) * (card_height + card_spacing);

            // Card panel
            let card_panel = UIComponent::new_panel(
                &format!("mode_card_{}", mode.id()),
                Rect::new(x, y, card_width, card_height),
                PanelComponent {
                    glass_effect: true,
                    blur_intensity: 4.0,
                    cast_shadow: true,
                    show_border: true,
                },
            );
            components.push(card_panel);

            // Mode icon
            let icon_component = UIComponent::new_text(
                &format!("mode_icon_{}", mode.id()),
                Rect::new(x + card_width * 0.5, y + 40.0, 0.0, 0.0),
                TextComponent {
                    content: mode.icon().to_string(),
                    font_size: 48.0,
                    color: Some(mode.color()),
                    alignment: TextAlignment::Center,
                    drop_shadow: false,
                },
            );
            components.push(icon_component);

            // Mode title
            let title_component = UIComponent::new_text(
                &format!("mode_title_{}", mode.id()),
                Rect::new(x + card_width * 0.5, y + 100.0, 0.0, 0.0),
                TextComponent {
                    content: mode.title().to_string(),
                    font_size: 24.0,
                    color: Some(Color::white()),
                    alignment: TextAlignment::Center,
                    drop_shadow: true,
                },
            );
            components.push(title_component);

            // Mode description
            let desc_component = UIComponent::new_text(
                &format!("mode_desc_{}", mode.id()),
                Rect::new(x + 20.0, y + 140.0, card_width - 40.0, 80.0),
                TextComponent {
                    content: mode.description().to_string(),
                    font_size: 14.0,
                    color: Some(Color::gray(0.8)),
                    alignment: TextAlignment::Center,
                    drop_shadow: false,
                },
            );
            components.push(desc_component);

            // Launch button
            let launch_button = UIComponent::new_button(
                &format!("mode_launch_{}", mode.id()),
                Rect::new(x + 80.0, y + 220.0, 200.0, 40.0),
                ButtonComponent {
                    label: "LAUNCH".to_string(),
                    icon: None,
                    state: ButtonState::Normal,
                    elevated: false,
                    ripple_effect: None,
                    font_size: Some(16.0),
                },
            );
            components.push(launch_button);

            // Progress indicator (if applicable)
            if let Some(progress) = self.progress_tracker.get_mode_progress(mode) {
                let progress_text = UIComponent::new_text(
                    &format!("mode_progress_{}", mode.id()),
                    Rect::new(x + card_width - 60.0, y + 20.0, 0.0, 0.0),
                    TextComponent {
                        content: format!("{}%", (progress * 100.0) as u32),
                        font_size: 12.0,
                        color: Some(Color::new(0.2, 1.0, 0.3, 1.0)),
                        alignment: TextAlignment::Right,
                        drop_shadow: false,
                    },
                );
                components.push(progress_text);
            }
        }

        // Settings button
        let settings_button = UIComponent::new_button(
            "settings_button",
            Rect::new(1850.0, 20.0, 50.0, 50.0),
            ButtonComponent {
                label: String::new(),
                icon: Some(Icon::Builtin(BuiltinIcon::Settings)),
                state: ButtonState::Normal,
                elevated: false,
                ripple_effect: None,
                font_size: None,
            },
        );
        components.push(settings_button);

        // Help button
        let help_button = UIComponent::new_button(
            "help_button",
            Rect::new(1790.0, 20.0, 50.0, 50.0),
            ButtonComponent {
                label: "?".to_string(),
                icon: None,
                state: ButtonState::Normal,
                elevated: false,
                ripple_effect: None,
                font_size: Some(20.0),
            },
        );
        components.push(help_button);

        components
    }

    pub fn handle_mode_selection(&mut self, mode: DemoMode) -> RobinResult<()> {
        self.current_state = WelcomeState::Loading;
        self.mode_selector.select_mode(mode)?;

        // Check if onboarding is needed
        if self.config.enable_onboarding && self.onboarding_manager.should_show_onboarding(mode) {
            self.current_state = WelcomeState::Onboarding;
            self.onboarding_manager.start_onboarding(mode)?;
        } else if self.config.tutorial_enabled && self.config.first_time_user {
            self.current_state = WelcomeState::Tutorial;
            self.tutorial_overlay.show_tutorial(mode)?;
        } else {
            self.current_state = WelcomeState::Active;
        }

        Ok(())
    }

    pub fn update(&mut self, delta: Duration) -> RobinResult<()> {
        match self.current_state {
            WelcomeState::SplashScreen => {
                self.intro_animator.update(delta)?;
            }
            WelcomeState::ModeSelection => {
                self.demo_previewer.update(delta)?;
            }
            WelcomeState::Onboarding => {
                self.onboarding_manager.update(delta)?;
            }
            WelcomeState::Tutorial => {
                self.tutorial_overlay.update(delta)?;
            }
            _ => {}
        }

        // Update achievements if enabled
        if self.config.show_achievements {
            self.achievement_tracker.update()?;
        }

        Ok(())
    }

    pub fn skip_splash(&mut self) -> RobinResult<()> {
        if self.current_state == WelcomeState::SplashScreen {
            self.current_state = WelcomeState::ModeSelection;
        }
        Ok(())
    }

    pub fn skip_tutorial(&mut self) -> RobinResult<()> {
        if self.current_state == WelcomeState::Tutorial {
            self.current_state = WelcomeState::Active;
            self.tutorial_overlay.hide()?;
        }
        Ok(())
    }

    pub fn get_current_state(&self) -> WelcomeState {
        self.current_state
    }
}

/// Demo modes available for selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoMode {
    InteractivePlayground,
    EngineerBuildMode,
    GameplaySystems,
    CollaborationPreview,
    PerformanceBenchmark,
    VisualShowcase,
}

impl DemoMode {
    pub fn all() -> Vec<Self> {
        vec![
            Self::InteractivePlayground,
            Self::EngineerBuildMode,
            Self::GameplaySystems,
            Self::CollaborationPreview,
            Self::PerformanceBenchmark,
            Self::VisualShowcase,
        ]
    }

    pub fn id(&self) -> &str {
        match self {
            Self::InteractivePlayground => "playground",
            Self::EngineerBuildMode => "build_mode",
            Self::GameplaySystems => "gameplay",
            Self::CollaborationPreview => "collaboration",
            Self::PerformanceBenchmark => "performance",
            Self::VisualShowcase => "visual",
        }
    }

    pub fn title(&self) -> &str {
        match self {
            Self::InteractivePlayground => "Interactive Playground",
            Self::EngineerBuildMode => "Engineer Build Mode",
            Self::GameplaySystems => "Gameplay Systems",
            Self::CollaborationPreview => "Collaboration Preview",
            Self::PerformanceBenchmark => "Performance Benchmark",
            Self::VisualShowcase => "Visual Showcase",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Self::InteractivePlayground => "Free-form building with guided tutorials and interactive challenges",
            Self::EngineerBuildMode => "Advanced construction tools with templates and precision building",
            Self::GameplaySystems => "Resource mining, crafting, and progression systems",
            Self::CollaborationPreview => "Multi-user building simulation and version control",
            Self::PerformanceBenchmark => "Real-time performance metrics and optimization showcase",
            Self::VisualShowcase => "Lighting, materials, and visual effects demonstration",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            Self::InteractivePlayground => "🎮",
            Self::EngineerBuildMode => "🔧",
            Self::GameplaySystems => "⚔️",
            Self::CollaborationPreview => "👥",
            Self::PerformanceBenchmark => "📊",
            Self::VisualShowcase => "✨",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Self::InteractivePlayground => Color::new(0.3, 0.8, 0.3, 1.0),
            Self::EngineerBuildMode => Color::new(0.8, 0.5, 0.2, 1.0),
            Self::GameplaySystems => Color::new(0.8, 0.2, 0.2, 1.0),
            Self::CollaborationPreview => Color::new(0.2, 0.5, 0.8, 1.0),
            Self::PerformanceBenchmark => Color::new(0.8, 0.8, 0.2, 1.0),
            Self::VisualShowcase => Color::new(0.8, 0.2, 0.8, 1.0),
        }
    }
}

// Supporting components (simplified implementations)

#[derive(Debug)]
pub struct ModeSelector {
    selected_mode: Option<DemoMode>,
    mode_configs: HashMap<DemoMode, ModeConfig>,
}

#[derive(Debug, Clone)]
pub struct ModeConfig {
    pub auto_start: bool,
    pub preload_assets: bool,
    pub custom_settings: HashMap<String, String>,
}

impl ModeSelector {
    pub fn new(_config: &WelcomeConfig) -> Self {
        let mut mode_configs = HashMap::new();
        for mode in DemoMode::all() {
            mode_configs.insert(mode, ModeConfig {
                auto_start: false,
                preload_assets: true,
                custom_settings: HashMap::new(),
            });
        }

        Self {
            selected_mode: None,
            mode_configs,
        }
    }

    pub fn select_mode(&mut self, mode: DemoMode) -> RobinResult<()> {
        self.selected_mode = Some(mode);
        Ok(())
    }

    pub fn get_selected_mode(&self) -> Option<DemoMode> {
        self.selected_mode
    }
}

macro_rules! define_welcome_component {
    ($name:ident) => {
        #[derive(Debug)]
        pub struct $name;

        impl $name {
            pub fn new() -> Self {
                Self
            }
        }
    };
}

define_welcome_component!(OnboardingManager);
define_welcome_component!(TutorialOverlay);
define_welcome_component!(AchievementTracker);
define_welcome_component!(DemoPrevier);
define_welcome_component!(ControlsGuide);
define_welcome_component!(ProgressTracker);
define_welcome_component!(IntroAnimator);

// Implement key methods for components

impl OnboardingManager {
    pub fn new(_config: &WelcomeConfig) -> Self { Self }

    pub fn should_show_onboarding(&self, _mode: DemoMode) -> bool {
        false // Simplified - would check user preferences
    }

    pub fn start_onboarding(&mut self, _mode: DemoMode) -> RobinResult<()> {
        Ok(())
    }

    pub fn update(&mut self, _delta: Duration) -> RobinResult<()> {
        Ok(())
    }
}

impl TutorialOverlay {
    pub fn show_tutorial(&mut self, _mode: DemoMode) -> RobinResult<()> {
        Ok(())
    }

    pub fn hide(&mut self) -> RobinResult<()> {
        Ok(())
    }

    pub fn update(&mut self, _delta: Duration) -> RobinResult<()> {
        Ok(())
    }
}

impl AchievementTracker {
    pub fn update(&mut self) -> RobinResult<()> {
        Ok(())
    }

    pub fn unlock_achievement(&mut self, _achievement: &str) -> RobinResult<()> {
        Ok(())
    }
}

impl DemoPrevier {
    pub fn update(&mut self, _delta: Duration) -> RobinResult<()> {
        Ok(())
    }

    pub fn generate_preview(&self, _mode: DemoMode) -> RobinResult<Vec<u8>> {
        Ok(Vec::new()) // Would return preview image data
    }
}

impl ProgressTracker {
    pub fn get_mode_progress(&self, mode: &DemoMode) -> Option<f32> {
        // Simulated progress values
        match mode {
            DemoMode::InteractivePlayground => Some(0.75),
            DemoMode::EngineerBuildMode => Some(0.5),
            DemoMode::GameplaySystems => Some(0.25),
            _ => None,
        }
    }

    pub fn update_progress(&mut self, _mode: DemoMode, _progress: f32) -> RobinResult<()> {
        Ok(())
    }
}

impl IntroAnimator {
    pub fn update(&mut self, _delta: Duration) -> RobinResult<()> {
        Ok(())
    }
}