/*!
 * Production Main Menu System
 *
 * Modern main menu interface for Robin Engine using the production dark theme
 * and advanced UI components. Provides game mode selection and settings access.
 */

use crate::engine::{
    ui::{
        production_theme_simple::ProductionDarkTheme,
        modern_components::{ModernButton, AccessibilityProps},
        css_in_rust::{Style, StyleSheet},
        UIBounds, UIState, UIElement,
    },
    input::InputManager,
    error::RobinResult,
    networking::{NetworkManager, NetworkMode, ServerConfig, NetworkEvent},
    save_system::{SaveManager, SaveMetadata, SaveSystemConfig},
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Instant;

/// Main menu state and actions
#[derive(Debug, Clone)]
pub enum MenuAction {
    StartSinglePlayer,
    StartCreativeMode,
    StartMultiplayer,
    HostServer,
    JoinServer,
    ServerBrowser,
    ConnectToServer(SocketAddr, String),
    LoadWorld(String),
    CreateWorld { name: String, seed: u64 },
    OpenSettings,
    OpenTutorial,
    OpenHelp,
    QuitApplication,
    Back,
    NetworkEvent(NetworkEvent),
    None,
}

/// Game mode selection
#[derive(Debug, Clone, PartialEq)]
pub enum GameMode {
    SinglePlayer,
    CreativeMode,
    Multiplayer,
    Tutorial,
}

/// Menu screens/states
#[derive(Debug, Clone, PartialEq)]
pub enum MenuScreen {
    MainMenu,
    GameModeSelection,
    WorldSelection,
    MultiplayerMenu,
    HostServer,
    JoinServer,
    ServerBrowser,
    CreateWorld,
    Settings,
    Tutorial,
    About,
}

/// Main menu system manager
pub struct MainMenuSystem {
    theme: ProductionDarkTheme,
    current_screen: MenuScreen,
    menu_actions: Vec<MenuAction>,

    // UI elements
    buttons: HashMap<String, ModernButton>,
    selected_button_index: usize,
    button_order: Vec<String>,

    // State
    visible: bool,
    animation_progress: f32,
    selected_game_mode: Option<GameMode>,
    available_worlds: Vec<WorldInfo>,

    // Networking
    network_manager: NetworkManager,
    server_list: Vec<ServerInfo>,
    network_ui_state: NetworkUIState,
    save_manager: SaveManager,
    error_message: Option<String>,

    // Styling
    styles: HashMap<String, Style>,
    stylesheet: StyleSheet,
}

/// World information for world selection
#[derive(Debug, Clone)]
pub struct WorldInfo {
    pub name: String,
    pub path: String,
    pub created_at: std::time::SystemTime,
    pub last_played: std::time::SystemTime,
    pub description: String,
    pub thumbnail_path: Option<String>,
    pub size_mb: f64,
}

/// Server information for multiplayer browser
#[derive(Debug, Clone)]
pub struct ServerInfo {
    pub name: String,
    pub address: SocketAddr,
    pub player_count: usize,
    pub max_players: usize,
    pub ping: Option<u32>,
    pub version: String,
    pub has_password: bool,
    pub last_update: Instant,
}

/// UI input state for forms
#[derive(Debug, Default)]
pub struct NetworkUIState {
    pub player_name: String,
    pub server_address: String,
    pub server_port: String,
    pub server_password: String,
    pub world_name: String,
    pub world_seed: String,
    pub host_config: ServerConfig,
    pub selected_server: Option<usize>,
}

impl MainMenuSystem {
    pub fn new() -> Self {
        let theme = ProductionDarkTheme::new();
        let styles = theme.create_main_menu_styles();
        let stylesheet = theme.generate_stylesheet();
        let network_manager = NetworkManager::new();
        let save_config = SaveSystemConfig::default();
        let save_manager = SaveManager::new(save_config);
        let mut network_ui_state = NetworkUIState::default();
        network_ui_state.player_name = whoami::username();
        network_ui_state.host_config = ServerConfig::default();

        let mut system = Self {
            theme,
            current_screen: MenuScreen::MainMenu,
            menu_actions: Vec::new(),
            buttons: HashMap::new(),
            selected_button_index: 0,
            button_order: Vec::new(),
            visible: true,
            animation_progress: 0.0,
            selected_game_mode: None,
            available_worlds: Vec::new(),
            network_manager,
            server_list: Vec::new(),
            network_ui_state,
            save_manager,
            error_message: None,
            styles,
            stylesheet,
        };

        system.initialize_main_menu();
        system
    }

    /// Initialize main menu buttons and layout
    fn initialize_main_menu(&mut self) {
        self.current_screen = MenuScreen::MainMenu;
        self.buttons.clear();
        self.button_order.clear();
        self.menu_actions.clear();

        // Main menu buttons with accessibility
        let button_configs = vec![
            ("start_game", "🎮 Start Game", MenuAction::StartSinglePlayer, "Start a new single player game"),
            ("creative_mode", "🔨 Creative Mode", MenuAction::StartCreativeMode, "Build freely without constraints"),
            ("multiplayer", "🌐 Multiplayer", MenuAction::StartMultiplayer, "Play with friends online"),
            ("load_world", "📁 Load World", MenuAction::LoadWorld("".to_string()), "Load an existing world"),
            ("tutorial", "🎓 Tutorial", MenuAction::OpenTutorial, "Learn how to play Robin Engine"),
            ("settings", "⚙️ Settings", MenuAction::OpenSettings, "Adjust graphics, audio, and controls"),
            ("help", "❓ Help", MenuAction::OpenHelp, "View controls and documentation"),
            ("quit", "🚪 Quit", MenuAction::QuitApplication, "Exit Robin Engine"),
        ];

        for (i, (id, text, action, description)) in button_configs.iter().enumerate() {
            let button_bounds = UIBounds::new(
                150.0,                           // x - centered
                250.0 + i as f32 * 60.0,        // y - stacked vertically
                300.0,                           // width
                50.0,                            // height
            );

            let accessibility = AccessibilityProps {
                aria_label: Some(text.to_string()),
                aria_description: Some(description.to_string()),
                role: "button".to_string(),
                tab_index: i as i32,
                keyboard_shortcuts: if *id == "start_game" {
                    vec!["Enter".to_string()]
                } else {
                    vec![]
                },
                screen_reader_text: Some(format!("{}, {}", text, description)),
            };

            let mut button = ModernButton::primary()
                .with_text(text.to_string())
                .with_accessibility(accessibility);

            // Set bounds manually since we don't have with_bounds in the current API
            *button.get_bounds_mut() = button_bounds;

            // Store the action for this button
            self.menu_actions.push(action.clone());

            self.buttons.insert(id.to_string(), button);
            self.button_order.push(id.to_string());
        }

        // Set initial selection
        if let Some(first_button_id) = self.button_order.first() {
            if let Some(button) = self.buttons.get_mut(first_button_id) {
                button.set_keyboard_focus(true);
            }
        }
    }

    /// Initialize game mode selection screen
    fn initialize_game_mode_selection(&mut self) {
        self.current_screen = MenuScreen::GameModeSelection;
        self.buttons.clear();
        self.button_order.clear();
        self.menu_actions.clear();

        let mode_configs = vec![
            ("survival", "🏠 Survival Mode", GameMode::SinglePlayer,
             "Gather resources, build shelter, and survive"),
            ("creative", "🎨 Creative Mode", GameMode::CreativeMode,
             "Unlimited resources and freedom to build"),
            ("multiplayer", "👥 Multiplayer", GameMode::Multiplayer,
             "Collaborative building with friends"),
            ("tutorial", "📚 Guided Tutorial", GameMode::Tutorial,
             "Step-by-step introduction to Robin Engine"),
            ("back", "← Back", GameMode::SinglePlayer, "Return to main menu"),
        ];

        for (i, (id, text, mode, description)) in mode_configs.iter().enumerate() {
            let button_bounds = UIBounds::new(
                150.0,
                200.0 + i as f32 * 70.0,
                300.0,
                60.0,
            );

            let accessibility = AccessibilityProps {
                aria_label: Some(text.to_string()),
                aria_description: Some(description.to_string()),
                role: "button".to_string(),
                tab_index: i as i32,
                keyboard_shortcuts: vec![],
                screen_reader_text: Some(format!("{}, {}", text, description)),
            };

            let button_style = if *id == "back" {
                ModernButton::secondary()
            } else {
                ModernButton::primary()
            };

            let mut button = button_style
                .with_text(text.to_string())
                .with_accessibility(accessibility);

            *button.get_bounds_mut() = button_bounds;

            let action = if *id == "back" {
                MenuAction::Back
            } else {
                match mode {
                    GameMode::SinglePlayer => MenuAction::StartSinglePlayer,
                    GameMode::CreativeMode => MenuAction::StartCreativeMode,
                    GameMode::Multiplayer => MenuAction::StartMultiplayer,
                    GameMode::Tutorial => MenuAction::OpenTutorial,
                }
            };

            self.menu_actions.push(action);
            self.buttons.insert(id.to_string(), button);
            self.button_order.push(id.to_string());
        }
    }

    /// Update the menu system
    pub async fn update(&mut self, delta_time: f32, input: &InputManager) -> RobinResult<Vec<MenuAction>> {
        let mut actions = Vec::new();

        if !self.visible {
            return Ok(actions);
        }

        // Update networking
        if self.network_manager.get_mode() != NetworkMode::SinglePlayer {
            let network_events = self.network_manager.update().await?;
            for event in network_events {
                actions.push(MenuAction::NetworkEvent(event));
            }
        }

        // Update animation
        self.animation_progress = (self.animation_progress + delta_time * 3.0).min(1.0);

        // Handle keyboard navigation
        self.handle_keyboard_navigation(input);

        // Update all buttons
        for button in self.buttons.values_mut() {
            button.update(delta_time, input);
        }

        // Check for button activations
        self.check_button_activations(&mut actions);

        // Handle escape key
        if input.is_named_key_just_pressed(winit::keyboard::NamedKey::Escape) {
            match self.current_screen {
                MenuScreen::MainMenu => {
                    actions.push(MenuAction::QuitApplication);
                }
                _ => {
                    self.initialize_main_menu();
                }
            }
        }

        Ok(actions)
    }

    /// Handle keyboard navigation between menu items
    fn handle_keyboard_navigation(&mut self, input: &InputManager) {
        let button_count = self.button_order.len();
        if button_count == 0 { return; }

        let mut new_index = self.selected_button_index;

        // Navigate up/down
        if input.is_named_key_just_pressed(winit::keyboard::NamedKey::ArrowUp) {
            new_index = if self.selected_button_index == 0 {
                button_count - 1
            } else {
                self.selected_button_index - 1
            };
        } else if input.is_named_key_just_pressed(winit::keyboard::NamedKey::ArrowDown) {
            new_index = (self.selected_button_index + 1) % button_count;
        }

        // Update selection if changed
        if new_index != self.selected_button_index {
            // Remove focus from current button
            if let Some(current_id) = self.button_order.get(self.selected_button_index) {
                if let Some(button) = self.buttons.get_mut(current_id) {
                    button.set_keyboard_focus(false);
                }
            }

            // Set focus on new button
            self.selected_button_index = new_index;
            if let Some(new_id) = self.button_order.get(self.selected_button_index) {
                if let Some(button) = self.buttons.get_mut(new_id) {
                    button.set_keyboard_focus(true);
                }
            }
        }

        // Activate current button with Enter or Space
        if input.is_named_key_just_pressed(winit::keyboard::NamedKey::Enter) ||
           input.is_named_key_just_pressed(winit::keyboard::NamedKey::Space) {
            if let Some(button_id) = self.button_order.get(self.selected_button_index) {
                if let Some(button) = self.buttons.get_mut(button_id) {
                    button.activate();
                }
            }
        }
    }

    /// Check for button activations and generate actions
    fn check_button_activations(&mut self, actions: &mut Vec<MenuAction>) {
        let mut pending_screen_transitions = Vec::new();

        for (i, button_id) in self.button_order.iter().enumerate() {
            if let Some(button) = self.buttons.get_mut(button_id) {
                if button.get_state() == UIState::Pressed {
                    // Get the corresponding action
                    if let Some(action) = self.menu_actions.get(i) {
                        actions.push(action.clone());

                        // Collect screen transitions to handle after the loop
                        match action {
                            MenuAction::StartSinglePlayer => {
                                pending_screen_transitions.push("game_mode_selection");
                            }
                            MenuAction::StartMultiplayer => {
                                pending_screen_transitions.push("multiplayer_menu");
                            }
                            MenuAction::HostServer => {
                                pending_screen_transitions.push("host_server");
                            }
                            MenuAction::JoinServer => {
                                pending_screen_transitions.push("join_server");
                            }
                            MenuAction::ServerBrowser => {
                                pending_screen_transitions.push("server_browser");
                            }
                            MenuAction::Back => {
                                pending_screen_transitions.push("main_menu");
                            }
                            MenuAction::LoadWorld(_) => {
                                pending_screen_transitions.push("world_selection");
                            }
                            MenuAction::CreateWorld { .. } => {
                                pending_screen_transitions.push("create_world");
                            }
                            _ => {}
                        }
                    }

                    // Reset button state
                    button.set_state(UIState::Normal);
                }
            }
        }

        // Handle screen transitions after we're done with the mutable borrow
        for transition in pending_screen_transitions {
            match transition {
                "game_mode_selection" => self.initialize_game_mode_selection(),
                "main_menu" => self.initialize_main_menu(),
                "world_selection" => self.initialize_world_selection(),
                "multiplayer_menu" => self.initialize_multiplayer_menu(),
                "host_server" => self.initialize_host_server(),
                "join_server" => self.initialize_join_server(),
                "server_browser" => self.initialize_server_browser(),
                "create_world" => self.initialize_create_world(),
                _ => {}
            }
        }
    }

    /// Initialize world selection screen
    fn initialize_world_selection(&mut self) {
        self.current_screen = MenuScreen::WorldSelection;
        self.buttons.clear();
        self.button_order.clear();
        self.menu_actions.clear();

        // Load available worlds
        self.load_available_worlds();

        // Create buttons for each world
        for (i, world) in self.available_worlds.iter().enumerate() {
            let button_id = format!("world_{}", i);
            let button_bounds = UIBounds::new(
                100.0,
                150.0 + i as f32 * 80.0,
                400.0,
                70.0,
            );

            let world_text = format!("🌍 {}", world.name);
            let description = format!("{} - Last played: {}",
                world.description,
                "Recently" // TODO: Format timestamp
            );

            let accessibility = AccessibilityProps {
                aria_label: Some(world_text.clone()),
                aria_description: Some(description.clone()),
                role: "button".to_string(),
                tab_index: i as i32,
                keyboard_shortcuts: vec![],
                screen_reader_text: Some(format!("{}, {}", world_text, description)),
            };

            let mut button = ModernButton::primary()
                .with_text(world_text)
                .with_accessibility(accessibility);

            *button.get_bounds_mut() = button_bounds;

            self.menu_actions.push(MenuAction::LoadWorld(world.path.clone()));
            self.buttons.insert(button_id.clone(), button);
            self.button_order.push(button_id);
        }

        // Add back button
        let back_bounds = UIBounds::new(100.0, 150.0 + self.available_worlds.len() as f32 * 80.0 + 20.0, 200.0, 50.0);
        let mut back_button = ModernButton::secondary()
            .with_text("← Back".to_string());
        *back_button.get_bounds_mut() = back_bounds;

        self.menu_actions.push(MenuAction::Back);
        self.buttons.insert("back".to_string(), back_button);
        self.button_order.push("back".to_string());
    }

    /// Load available worlds from filesystem
    fn load_available_worlds(&mut self) {
        // TODO: Implement actual world loading from saves directory
        self.available_worlds = vec![
            WorldInfo {
                name: "My First World".to_string(),
                path: "saves/world1".to_string(),
                created_at: std::time::SystemTime::now(),
                last_played: std::time::SystemTime::now(),
                description: "A beautiful starting world".to_string(),
                thumbnail_path: None,
                size_mb: 12.5,
            },
            WorldInfo {
                name: "Castle Build".to_string(),
                path: "saves/castle".to_string(),
                created_at: std::time::SystemTime::now(),
                last_played: std::time::SystemTime::now(),
                description: "Medieval castle construction".to_string(),
                thumbnail_path: None,
                size_mb: 25.8,
            },
        ];
    }

    /// Initialize multiplayer menu
    fn initialize_multiplayer_menu(&mut self) {
        self.current_screen = MenuScreen::MultiplayerMenu;
        self.buttons.clear();
        self.button_order.clear();
        self.menu_actions.clear();

        let mp_configs = vec![
            ("host_server", "🖥️ Host Server", MenuAction::HostServer, "Create and host a multiplayer server"),
            ("join_server", "🔗 Join Server", MenuAction::JoinServer, "Connect to an existing server"),
            ("server_browser", "📋 Server Browser", MenuAction::ServerBrowser, "Browse available servers"),
            ("back", "← Back", MenuAction::Back, "Return to main menu"),
        ];

        for (i, (id, text, action, description)) in mp_configs.iter().enumerate() {
            let button_bounds = UIBounds::new(150.0, 200.0 + i as f32 * 70.0, 300.0, 60.0);
            let accessibility = AccessibilityProps {
                aria_label: Some(text.to_string()),
                aria_description: Some(description.to_string()),
                role: "button".to_string(),
                tab_index: i as i32,
                keyboard_shortcuts: vec![],
                screen_reader_text: Some(format!("{}, {}", text, description)),
            };

            let button_style = if *id == "back" { ModernButton::secondary() } else { ModernButton::primary() };
            let mut button = button_style.with_text(text.to_string()).with_accessibility(accessibility);
            *button.get_bounds_mut() = button_bounds;

            self.menu_actions.push(action.clone());
            self.buttons.insert(id.to_string(), button);
            self.button_order.push(id.to_string());
        }
    }

    /// Initialize host server menu
    fn initialize_host_server(&mut self) {
        self.current_screen = MenuScreen::HostServer;
        self.buttons.clear();
        self.button_order.clear();
        self.menu_actions.clear();

        // Server configuration will be handled via text inputs in the actual UI
        // For now, just add start server and back buttons
        let configs = vec![
            ("start_server", "🚀 Start Server", MenuAction::HostServer, "Start hosting the server"),
            ("back", "← Back", MenuAction::Back, "Return to multiplayer menu"),
        ];

        for (i, (id, text, action, description)) in configs.iter().enumerate() {
            let button_bounds = UIBounds::new(150.0, 400.0 + i as f32 * 70.0, 300.0, 60.0);
            let accessibility = AccessibilityProps {
                aria_label: Some(text.to_string()),
                aria_description: Some(description.to_string()),
                role: "button".to_string(),
                tab_index: i as i32,
                keyboard_shortcuts: vec![],
                screen_reader_text: Some(format!("{}, {}", text, description)),
            };

            let button_style = if *id == "back" { ModernButton::secondary() } else { ModernButton::primary() };
            let mut button = button_style.with_text(text.to_string()).with_accessibility(accessibility);
            *button.get_bounds_mut() = button_bounds;

            self.menu_actions.push(action.clone());
            self.buttons.insert(id.to_string(), button);
            self.button_order.push(id.to_string());
        }
    }

    /// Initialize join server menu
    fn initialize_join_server(&mut self) {
        self.current_screen = MenuScreen::JoinServer;
        self.buttons.clear();
        self.button_order.clear();
        self.menu_actions.clear();

        // Quick connect buttons
        let configs = vec![
            ("connect_localhost", "🏠 Connect to Localhost",
             MenuAction::ConnectToServer("127.0.0.1:25565".parse().unwrap(), self.network_ui_state.player_name.clone()),
             "Connect to local server"),
            ("connect_custom", "🔗 Connect", MenuAction::JoinServer, "Connect to custom server"),
            ("back", "← Back", MenuAction::Back, "Return to multiplayer menu"),
        ];

        for (i, (id, text, action, description)) in configs.iter().enumerate() {
            let button_bounds = UIBounds::new(150.0, 400.0 + i as f32 * 70.0, 300.0, 60.0);
            let accessibility = AccessibilityProps {
                aria_label: Some(text.to_string()),
                aria_description: Some(description.to_string()),
                role: "button".to_string(),
                tab_index: i as i32,
                keyboard_shortcuts: vec![],
                screen_reader_text: Some(format!("{}, {}", text, description)),
            };

            let button_style = if *id == "back" { ModernButton::secondary() } else { ModernButton::primary() };
            let mut button = button_style.with_text(text.to_string()).with_accessibility(accessibility);
            *button.get_bounds_mut() = button_bounds;

            self.menu_actions.push(action.clone());
            self.buttons.insert(id.to_string(), button);
            self.button_order.push(id.to_string());
        }
    }

    /// Initialize server browser
    fn initialize_server_browser(&mut self) {
        self.current_screen = MenuScreen::ServerBrowser;
        self.buttons.clear();
        self.button_order.clear();
        self.menu_actions.clear();

        // Add server entries
        for (i, server) in self.server_list.iter().enumerate() {
            let server_id = format!("server_{}", i);
            let ping_text = server.ping.map_or("???ms".to_string(), |p| format!("{}ms", p));
            let lock_icon = if server.has_password { "🔒 " } else { "" };
            let server_text = format!("{}{} ({}/{})", lock_icon, server.name, server.player_count, server.max_players);

            let button_bounds = UIBounds::new(100.0, 150.0 + i as f32 * 60.0, 500.0, 50.0);
            let accessibility = AccessibilityProps {
                aria_label: Some(server_text.clone()),
                aria_description: Some(format!("Server: {}, Ping: {}", server.name, ping_text)),
                role: "button".to_string(),
                tab_index: i as i32,
                keyboard_shortcuts: vec![],
                screen_reader_text: Some(format!("{}, Ping: {}", server_text, ping_text)),
            };

            let mut button = ModernButton::primary().with_text(server_text).with_accessibility(accessibility);
            *button.get_bounds_mut() = button_bounds;

            self.menu_actions.push(MenuAction::ConnectToServer(server.address, self.network_ui_state.player_name.clone()));
            self.buttons.insert(server_id.clone(), button);
            self.button_order.push(server_id);
        }

        // Add refresh and back buttons
        let control_configs = vec![
            ("refresh", "🔄 Refresh", MenuAction::ServerBrowser, "Refresh server list"),
            ("back", "← Back", MenuAction::Back, "Return to multiplayer menu"),
        ];

        let start_y = 150.0 + self.server_list.len() as f32 * 60.0 + 20.0;
        for (i, (id, text, action, description)) in control_configs.iter().enumerate() {
            let button_bounds = UIBounds::new(100.0, start_y + i as f32 * 70.0, 200.0, 50.0);
            let accessibility = AccessibilityProps {
                aria_label: Some(text.to_string()),
                aria_description: Some(description.to_string()),
                role: "button".to_string(),
                tab_index: (self.server_list.len() + i) as i32,
                keyboard_shortcuts: vec![],
                screen_reader_text: Some(format!("{}, {}", text, description)),
            };

            let button_style = if *id == "back" { ModernButton::secondary() } else { ModernButton::primary() };
            let mut button = button_style.with_text(text.to_string()).with_accessibility(accessibility);
            *button.get_bounds_mut() = button_bounds;

            self.menu_actions.push(action.clone());
            self.buttons.insert(id.to_string(), button);
            self.button_order.push(id.to_string());
        }
    }

    /// Initialize create world menu
    fn initialize_create_world(&mut self) {
        self.current_screen = MenuScreen::CreateWorld;
        self.buttons.clear();
        self.button_order.clear();
        self.menu_actions.clear();

        let configs = vec![
            ("create_world", "🌍 Create World",
             MenuAction::CreateWorld {
                 name: self.network_ui_state.world_name.clone(),
                 seed: self.network_ui_state.world_seed.parse().unwrap_or(rand::random())
             }, "Create the new world"),
            ("back", "← Back", MenuAction::Back, "Return to world selection"),
        ];

        for (i, (id, text, action, description)) in configs.iter().enumerate() {
            let button_bounds = UIBounds::new(150.0, 400.0 + i as f32 * 70.0, 300.0, 60.0);
            let accessibility = AccessibilityProps {
                aria_label: Some(text.to_string()),
                aria_description: Some(description.to_string()),
                role: "button".to_string(),
                tab_index: i as i32,
                keyboard_shortcuts: vec![],
                screen_reader_text: Some(format!("{}, {}", text, description)),
            };

            let button_style = if *id == "back" { ModernButton::secondary() } else { ModernButton::primary() };
            let mut button = button_style.with_text(text.to_string()).with_accessibility(accessibility);
            *button.get_bounds_mut() = button_bounds;

            self.menu_actions.push(action.clone());
            self.buttons.insert(id.to_string(), button);
            self.button_order.push(id.to_string());
        }
    }

    /// Show the main menu
    pub fn show(&mut self) {
        self.visible = true;
        self.animation_progress = 0.0;
        self.initialize_main_menu();
    }

    /// Hide the main menu
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Check if menu is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Get current screen
    pub fn get_current_screen(&self) -> MenuScreen {
        self.current_screen.clone()
    }

    /// Get menu buttons for rendering
    pub fn get_buttons(&self) -> &HashMap<String, ModernButton> {
        &self.buttons
    }

    /// Get menu styles for rendering
    pub fn get_styles(&self) -> &HashMap<String, Style> {
        &self.styles
    }

    /// Get current animation progress (0.0 to 1.0)
    pub fn get_animation_progress(&self) -> f32 {
        self.animation_progress
    }

    /// Get the current theme
    pub fn get_theme(&self) -> &ProductionDarkTheme {
        &self.theme
    }

    /// Get menu title for current screen
    pub fn get_screen_title(&self) -> String {
        match self.current_screen {
            MenuScreen::MainMenu => "ROBIN ENGINE".to_string(),
            MenuScreen::GameModeSelection => "SELECT GAME MODE".to_string(),
            MenuScreen::WorldSelection => "SELECT WORLD".to_string(),
            MenuScreen::MultiplayerMenu => "MULTIPLAYER".to_string(),
            MenuScreen::HostServer => "HOST SERVER".to_string(),
            MenuScreen::JoinServer => "JOIN SERVER".to_string(),
            MenuScreen::ServerBrowser => "SERVER BROWSER".to_string(),
            MenuScreen::CreateWorld => "CREATE WORLD".to_string(),
            MenuScreen::Settings => "SETTINGS".to_string(),
            MenuScreen::Tutorial => "TUTORIAL".to_string(),
            MenuScreen::About => "ABOUT".to_string(),
        }
    }

    /// Get menu subtitle/description for current screen
    pub fn get_screen_subtitle(&self) -> String {
        match self.current_screen {
            MenuScreen::MainMenu => "3D Voxel Game Engine - Build. Create. Explore.".to_string(),
            MenuScreen::GameModeSelection => "Choose how you want to experience Robin Engine".to_string(),
            MenuScreen::WorldSelection => "Load an existing world or create a new one".to_string(),
            MenuScreen::MultiplayerMenu => "Connect with friends and explore together".to_string(),
            MenuScreen::HostServer => "Create and configure your multiplayer server".to_string(),
            MenuScreen::JoinServer => "Connect to an existing multiplayer server".to_string(),
            MenuScreen::ServerBrowser => "Browse and connect to available servers".to_string(),
            MenuScreen::CreateWorld => "Design your new world".to_string(),
            MenuScreen::Settings => "Customize your Robin Engine experience".to_string(),
            MenuScreen::Tutorial => "Learn the basics of building in Robin Engine".to_string(),
            MenuScreen::About => "About Robin Engine and its features".to_string(),
        }
    }

    /// Get network manager for external access
    pub fn get_network_manager(&mut self) -> &mut NetworkManager {
        &mut self.network_manager
    }

    /// Add server to the server list
    pub fn add_server(&mut self, server: ServerInfo) {
        // Remove existing entry for same address
        self.server_list.retain(|s| s.address != server.address);
        self.server_list.push(server);

        // Refresh server browser if currently viewing it
        if self.current_screen == MenuScreen::ServerBrowser {
            self.initialize_server_browser();
        }
    }

    /// Get current network UI state
    pub fn get_network_ui_state(&self) -> &NetworkUIState {
        &self.network_ui_state
    }

    /// Update network UI state
    pub fn update_network_ui_state(&mut self, state: NetworkUIState) {
        self.network_ui_state = state;
    }

    /// Show error message
    pub fn show_error(&mut self, message: String) {
        self.error_message = Some(message);
    }

    /// Clear error message
    pub fn clear_error(&mut self) {
        self.error_message = None;
    }

    /// Get current error message
    pub fn get_error_message(&self) -> Option<&String> {
        self.error_message.as_ref()
    }

    /// Get save manager
    pub fn get_save_manager(&mut self) -> &mut SaveManager {
        &mut self.save_manager
    }

    /// Check if currently in a multiplayer state
    pub fn is_multiplayer_mode(&self) -> bool {
        matches!(
            self.current_screen,
            MenuScreen::MultiplayerMenu | MenuScreen::HostServer | MenuScreen::JoinServer | MenuScreen::ServerBrowser
        )
    }

    /// Get current network mode
    pub fn get_network_mode(&self) -> NetworkMode {
        self.network_manager.get_mode()
    }
}

impl Default for MainMenuSystem {
    fn default() -> Self {
        Self::new()
    }
}

// Helper implementations for navigation
impl MenuScreen {
    pub fn is_main_menu(&self) -> bool {
        matches!(self, MenuScreen::MainMenu)
    }

    pub fn can_go_back(&self) -> bool {
        !matches!(self, MenuScreen::MainMenu)
    }
}

impl GameMode {
    pub fn get_description(&self) -> &'static str {
        match self {
            GameMode::SinglePlayer => "Explore and build in a persistent world with survival mechanics",
            GameMode::CreativeMode => "Unlimited resources and creative freedom to build anything",
            GameMode::Multiplayer => "Join friends to build and explore together",
            GameMode::Tutorial => "Interactive lessons to learn Robin Engine's features",
        }
    }

    pub fn get_icon(&self) -> &'static str {
        match self {
            GameMode::SinglePlayer => "🏠",
            GameMode::CreativeMode => "🎨",
            GameMode::Multiplayer => "👥",
            GameMode::Tutorial => "📚",
        }
    }
}