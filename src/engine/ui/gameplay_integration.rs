/*!
 * Gameplay Systems UI Integration
 *
 * This module bridges the gap between core gameplay systems (build mode, physics,
 * networking) and the UI framework, providing seamless integration for all
 * interactive elements.
 */

use crate::engine::{
    error::RobinResult,
    build_mode::{
        EngineerBuildMode, BuildMode, BuildModeState, TemplateType,
        BuildAction, PerformanceProfile
    },
    networking::{NetworkManager, NetworkEvent},
    save_system::SaveManager,
    world::VoxelType,
    ui::main_menu::MenuAction,
    input::InputManager,
    math::Vec3,
};
use std::time::{Duration, Instant};

/// Comprehensive UI integration for all gameplay systems
#[derive(Debug)]
pub struct GameplayUIIntegration {
    /// Build mode UI integration
    build_mode_ui: BuildModeUI,

    /// Multiplayer UI integration
    multiplayer_ui: MultiplayerUI,

    /// Save/Load UI integration
    save_load_ui: SaveLoadUI,

    /// Performance monitoring UI
    performance_ui: PerformanceMonitorUI,

    /// Inventory and materials UI
    inventory_ui: InventoryUI,

    /// Status and notifications UI
    status_ui: StatusUI,

    /// Integration state
    is_visible: bool,
    current_tab: GameplayTab,
    last_update: Instant,
}

/// Available UI tabs for gameplay integration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GameplayTab {
    BuildMode,
    Multiplayer,
    SaveLoad,
    Performance,
    Inventory,
    Settings,
}

/// Build mode UI controls and overlays
#[derive(Debug)]
struct BuildModeUI {
    selected_mode: BuildMode,
    selected_template: TemplateType,
    selected_material: VoxelType,
    grid_snap_enabled: bool,
    tool_palette_open: bool,
    mode_transition_overlay: bool,
    quick_action_buttons: Vec<QuickActionButton>,
    build_stats: BuildStatistics,
}

/// Quick action buttons for common build operations
#[derive(Debug, Clone)]
struct QuickActionButton {
    label: String,
    tooltip: String,
    action: BuildAction,
    shortcut: Option<String>,
    enabled: bool,
}

/// Real-time build statistics
#[derive(Debug, Default)]
struct BuildStatistics {
    voxels_placed: u32,
    voxels_removed: u32,
    templates_used: u32,
    build_time: Duration,
    undo_count: u32,
    redo_count: u32,
}

/// Multiplayer UI integration
#[derive(Debug)]
struct MultiplayerUI {
    connection_status: ConnectionStatus,
    player_list: Vec<PlayerInfo>,
    chat_window: ChatWindow,
    server_info: Option<ServerInfo>,
    collaboration_overlay: bool,
    sync_progress: f32,
}

#[derive(Debug, Clone)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected { ping: u32 },
    Error(String),
}

#[derive(Debug, Clone)]
pub struct PlayerInfo {
    id: u32,
    name: String,
    position: Vec3,
    is_building: bool,
    last_action: String,
}

#[derive(Debug, Clone)]
struct ServerInfo {
    name: String,
    player_count: usize,
    max_players: usize,
    world_seed: u64,
    uptime: Duration,
}

#[derive(Debug)]
struct ChatWindow {
    messages: Vec<ChatMessage>,
    input_buffer: String,
    is_open: bool,
    auto_scroll: bool,
    max_messages: usize,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    timestamp: Instant,
    player_name: String,
    content: String,
    message_type: ChatMessageType,
}

#[derive(Debug, Clone)]
pub enum ChatMessageType {
    Player,
    System,
    Server,
    Error,
}

/// Save/Load UI integration
#[derive(Debug)]
struct SaveLoadUI {
    world_list: Vec<WorldEntry>,
    selected_world: Option<usize>,
    auto_save_enabled: bool,
    auto_save_interval: u32,
    last_save_time: Option<Instant>,
    save_in_progress: bool,
    load_in_progress: bool,
    backup_count: usize,
}

#[derive(Debug, Clone)]
struct WorldEntry {
    name: String,
    metadata: WorldMetadata,
    file_size: u64,
    last_modified: std::time::SystemTime,
    thumbnail: Option<u32>, // Texture ID
}

/// Performance monitoring UI
#[derive(Debug)]
struct PerformanceMonitorUI {
    show_stats: bool,
    show_optimization_tips: bool,
    performance_history: Vec<PerformanceSnapshot>,
    current_profile: PerformanceProfile,
    auto_optimize: bool,
    history_length: usize,
}

#[derive(Debug, Clone)]
struct PerformanceSnapshot {
    timestamp: Instant,
    fps: f32,
    frame_time: f32,
    memory_usage: u64,
    voxel_count: u32,
    chunk_count: u32,
}

/// Inventory and materials UI
#[derive(Debug)]
struct InventoryUI {
    material_grid: Vec<MaterialSlot>,
    selected_slot: Option<usize>,
    filter_text: String,
    sort_mode: InventorySortMode,
    show_quantities: bool,
    quick_access_slots: [Option<VoxelType>; 9],
}

#[derive(Debug, Clone)]
struct MaterialSlot {
    material: VoxelType,
    quantity: u32,
    icon: Option<u32>, // Texture ID
    is_available: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InventorySortMode {
    Alphabetical,
    Quantity,
    Type,
    RecentlyUsed,
}

/// Status and notifications UI
#[derive(Debug)]
struct StatusUI {
    notifications: Vec<Notification>,
    status_bar_items: Vec<StatusItem>,
    show_tooltips: bool,
    notification_duration: Duration,
}

#[derive(Debug, Clone)]
struct Notification {
    id: u32,
    title: String,
    message: String,
    notification_type: NotificationType,
    timestamp: Instant,
    duration: Duration,
    actions: Vec<NotificationAction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationType {
    Info,
    Success,
    Warning,
    Error,
    Achievement,
}

#[derive(Debug, Clone)]
struct NotificationAction {
    label: String,
    action: String,
}

#[derive(Debug, Clone)]
struct StatusItem {
    label: String,
    value: String,
    tooltip: Option<String>,
    color: [f32; 4],
}

impl GameplayUIIntegration {
    pub fn new() -> Self {
        Self {
            build_mode_ui: BuildModeUI::new(),
            multiplayer_ui: MultiplayerUI::new(),
            save_load_ui: SaveLoadUI::new(),
            performance_ui: PerformanceMonitorUI::new(),
            inventory_ui: InventoryUI::new(),
            status_ui: StatusUI::new(),
            is_visible: true,
            current_tab: GameplayTab::BuildMode,
            last_update: Instant::now(),
        }
    }

    /// Update all UI systems with current gameplay state
    pub fn update(
        &mut self,
        build_mode: &mut EngineerBuildMode,
        network_manager: &mut NetworkManager,
        save_manager: &mut SaveManager,
        input: &InputManager,
        delta_time: f32,
    ) -> RobinResult<Vec<MenuAction>> {
        let mut actions = Vec::new();

        // Update build mode UI
        self.build_mode_ui.update(build_mode, input, delta_time)?;

        // Update multiplayer UI
        if let Some(multiplayer_actions) = self.multiplayer_ui.update(network_manager, input, delta_time)? {
            actions.extend(multiplayer_actions);
        }

        // Update save/load UI
        if let Some(save_actions) = self.save_load_ui.update(save_manager, input, delta_time)? {
            actions.extend(save_actions);
        }

        // Update performance monitoring
        self.performance_ui.update(build_mode, delta_time)?;

        // Update inventory UI
        self.inventory_ui.update(build_mode, input, delta_time)?;

        // Update status and notifications
        self.status_ui.update(delta_time)?;

        // Handle UI visibility toggle
        if input.is_key_just_pressed(&winit::keyboard::Key::Named(winit::keyboard::NamedKey::Tab)) {
            self.is_visible = !self.is_visible;
        }

        self.last_update = Instant::now();

        Ok(actions)
    }

    /// Render the gameplay UI integration overlay
    /// Note: Actual rendering implementation will be integrated with the existing UI system
    pub fn render(&mut self) -> RobinResult<()> {
        if !self.is_visible {
            return Ok(());
        }

        // TODO: Integrate with existing Robin UI rendering system
        // This will be connected to the main UI renderer once fully integrated
        log::debug!("Rendering gameplay UI integration - tab: {:?}", self.current_tab);

        Ok(())
    }

    /// Add a notification to the UI
    pub fn add_notification(&mut self, title: String, message: String, notification_type: NotificationType) {
        self.status_ui.add_notification(title, message, notification_type);
    }

    /// Update connection status
    pub fn update_connection_status(&mut self, status: ConnectionStatus) {
        self.multiplayer_ui.connection_status = status;
    }

    /// Add chat message
    pub fn add_chat_message(&mut self, player_name: String, content: String, message_type: ChatMessageType) {
        self.multiplayer_ui.chat_window.add_message(player_name, content, message_type);
    }

    /// Update player list
    pub fn update_player_list(&mut self, players: Vec<PlayerInfo>) {
        self.multiplayer_ui.player_list = players;
    }

    /// Set current build mode
    pub fn set_build_mode(&mut self, mode: BuildMode) {
        self.build_mode_ui.selected_mode = mode;
    }

    /// Set current material
    pub fn set_current_material(&mut self, material: VoxelType) {
        self.build_mode_ui.selected_material = material;
        self.inventory_ui.set_selected_material(material);
    }
}

impl BuildModeUI {
    fn new() -> Self {
        Self {
            selected_mode: BuildMode::Single,
            selected_template: TemplateType::Stairs,
            selected_material: VoxelType::Stone,
            grid_snap_enabled: true,
            tool_palette_open: false,
            mode_transition_overlay: false,
            quick_action_buttons: Self::create_quick_actions(),
            build_stats: BuildStatistics::default(),
        }
    }

    fn create_quick_actions() -> Vec<QuickActionButton> {
        vec![
            QuickActionButton {
                label: "Undo".to_string(),
                tooltip: "Undo last action".to_string(),
                action: BuildAction::PlaceVoxel { position: Vec3::new(0.0, 0.0, 0.0), material_id: 0 },
                shortcut: Some("Ctrl+Z".to_string()),
                enabled: true,
            },
            QuickActionButton {
                label: "Redo".to_string(),
                tooltip: "Redo last undone action".to_string(),
                action: BuildAction::RemoveVoxel { position: Vec3::new(0.0, 0.0, 0.0), material_id: 0 },
                shortcut: Some("Ctrl+Y".to_string()),
                enabled: true,
            },
        ]
    }

    fn update(&mut self, build_mode: &mut EngineerBuildMode, _input: &InputManager, _delta_time: f32) -> RobinResult<()> {
        // Sync with build mode state
        self.selected_mode = build_mode.get_mode_system().get_current_build_mode();

        // Update build statistics
        self.build_stats.build_time += Duration::from_secs_f32(_delta_time);

        Ok(())
    }

    fn render(&mut self) -> RobinResult<()> {
        // Build mode selection
        ui.text("Build Mode:");
        let mode_names = ["Single", "Wall", "Floor", "Roof", "Template", "Circle", "Sphere", "Terrain", "Copy", "Paste"];
        let mut current_mode = self.selected_mode as usize;

        if ui.combo("##build_mode", &mut current_mode, &mode_names, |item| std::borrow::Cow::Borrowed(item)) {
            self.selected_mode = match current_mode {
                0 => BuildMode::Single,
                1 => BuildMode::Wall,
                2 => BuildMode::Floor,
                3 => BuildMode::Roof,
                4 => BuildMode::Template,
                5 => BuildMode::Circle,
                6 => BuildMode::Sphere,
                7 => BuildMode::Terrain,
                8 => BuildMode::Copy,
                9 => BuildMode::Paste,
                _ => BuildMode::Single,
            };
        }

        ui.separator();

        // Template selection (if in template mode)
        if self.selected_mode == BuildMode::Template {
            ui.text("Template:");
            let template_names = ["Stairs", "Arch", "Bridge", "Tower", "House", "Castle", "Garden", "Workshop", "Fortress", "Lighthouse", "Windmill"];
            let mut current_template = self.selected_template as usize;

            if ui.combo("##template", &mut current_template, &template_names, |item| std::borrow::Cow::Borrowed(item)) {
                self.selected_template = match current_template {
                    0 => TemplateType::Stairs,
                    1 => TemplateType::Arch,
                    2 => TemplateType::Bridge,
                    3 => TemplateType::Tower,
                    4 => TemplateType::House,
                    5 => TemplateType::Castle,
                    6 => TemplateType::Garden,
                    7 => TemplateType::Workshop,
                    8 => TemplateType::Fortress,
                    9 => TemplateType::Lighthouse,
                    10 => TemplateType::Windmill,
                    _ => TemplateType::Stairs,
                };
            }
        }

        ui.separator();

        // Material selection
        ui.text("Material:");
        let material_names = ["Stone", "Wood", "Dirt", "Grass", "Sand", "Water", "Glass", "Metal", "Brick"];
        let mut current_material = self.selected_material as usize;

        if ui.combo("##material", &mut current_material, &material_names, |item| std::borrow::Cow::Borrowed(item)) {
            self.selected_material = match current_material {
                0 => VoxelType::Stone,
                1 => VoxelType::Wood,
                2 => VoxelType::Dirt,
                3 => VoxelType::Grass,
                4 => VoxelType::Sand,
                5 => VoxelType::Water,
                6 => VoxelType::Glass,
                7 => VoxelType::Metal,
                8 => VoxelType::Brick,
                _ => VoxelType::Stone,
            };
        }

        ui.separator();

        // Grid snap toggle
        ui.checkbox("Grid Snap", &mut self.grid_snap_enabled);

        ui.separator();

        // Quick action buttons
        ui.text("Quick Actions:");
        for button in &self.quick_action_buttons {
            if !button.enabled {
                ui.begin_disabled();
            }

            if ui.button(&button.label) {
                // Handle quick action
            }

            if !button.enabled {
                ui.end_disabled();
            }

            if ui.is_item_hovered() && !button.tooltip.is_empty() {
                ui.tooltip_text(&button.tooltip);
            }

            ui.same_line();
        }
        ui.new_line();

        ui.separator();

        // Build statistics
        ui.text("Build Statistics:");
        ui.text(format!("Voxels Placed: {}", self.build_stats.voxels_placed));
        ui.text(format!("Voxels Removed: {}", self.build_stats.voxels_removed));
        ui.text(format!("Templates Used: {}", self.build_stats.templates_used));
        ui.text(format!("Build Time: {:.1}s", self.build_stats.build_time.as_secs_f32()));

        Ok(())
    }
}

impl MultiplayerUI {
    fn new() -> Self {
        Self {
            connection_status: ConnectionStatus::Disconnected,
            player_list: Vec::new(),
            chat_window: ChatWindow::new(),
            server_info: None,
            collaboration_overlay: false,
            sync_progress: 0.0,
        }
    }

    fn update(&mut self, network_manager: &mut NetworkManager, _input: &InputManager, _delta_time: f32) -> RobinResult<Option<Vec<MenuAction>>> {
        // Update connection status
        if let Some(client) = network_manager.get_client() {
            if client.is_connected() {
                self.connection_status = ConnectionStatus::Connected { ping: client.get_ping() };
            } else {
                self.connection_status = ConnectionStatus::Disconnected;
            }
        }

        // Process network events
        let events = network_manager.get_events();
        for event in events {
            match event {
                NetworkEvent::PlayerJoined { player_id, name } => {
                    self.chat_window.add_message(
                        "System".to_string(),
                        format!("{} joined the server", name),
                        ChatMessageType::System,
                    );
                    self.player_list.push(PlayerInfo {
                        id: player_id,
                        name,
                        position: Vec3::new(0.0, 0.0, 0.0),
                        is_building: false,
                        last_action: "Joined".to_string(),
                    });
                }
                NetworkEvent::PlayerLeft { player_id, name } => {
                    self.chat_window.add_message(
                        "System".to_string(),
                        format!("{} left the server", name),
                        ChatMessageType::System,
                    );
                    self.player_list.retain(|p| p.id != player_id);
                }
                NetworkEvent::ChatReceived { player_id, message } => {
                    if let Some(player) = self.player_list.iter().find(|p| p.id == player_id) {
                        self.chat_window.add_message(
                            player.name.clone(),
                            message,
                            ChatMessageType::Player,
                        );
                    }
                }
                _ => {}
            }
        }

        Ok(None)
    }

    fn render(&mut self) -> RobinResult<()> {
        // Connection status
        ui.text("Connection Status:");
        match &self.connection_status {
            ConnectionStatus::Disconnected => {
                ui.text_colored([1.0, 0.0, 0.0, 1.0], "Disconnected");
            }
            ConnectionStatus::Connecting => {
                ui.text_colored([1.0, 1.0, 0.0, 1.0], "Connecting...");
            }
            ConnectionStatus::Connected { ping } => {
                ui.text_colored([0.0, 1.0, 0.0, 1.0], format!("Connected ({}ms)", ping));
            }
            ConnectionStatus::Error(err) => {
                ui.text_colored([1.0, 0.0, 0.0, 1.0], format!("Error: {}", err));
            }
        }

        ui.separator();

        // Server info
        if let Some(ref server_info) = self.server_info {
            ui.text(format!("Server: {}", server_info.name));
            ui.text(format!("Players: {}/{}", server_info.player_count, server_info.max_players));
            ui.text(format!("World Seed: {}", server_info.world_seed));
            ui.separator();
        }

        // Player list
        ui.text("Players Online:");
        for player in &self.player_list {
            let status_color = if player.is_building { [0.0, 1.0, 0.0, 1.0] } else { [0.8, 0.8, 0.8, 1.0] };
            ui.text_colored(status_color, format!("• {} ({})", player.name, player.last_action));
        }

        ui.separator();

        // Chat window
        self.chat_window.render(ui)?;

        Ok(())
    }
}

impl ChatWindow {
    fn new() -> Self {
        Self {
            messages: Vec::new(),
            input_buffer: String::new(),
            is_open: true,
            auto_scroll: true,
            max_messages: 100,
        }
    }

    fn add_message(&mut self, player_name: String, content: String, message_type: ChatMessageType) {
        self.messages.push(ChatMessage {
            timestamp: Instant::now(),
            player_name,
            content,
            message_type,
        });

        // Limit message history
        if self.messages.len() > self.max_messages {
            self.messages.remove(0);
        }
    }

    fn render(&mut self) -> RobinResult<()> {
        ui.text("Chat:");

        if let Some(_child) = ui.child_window("chat_messages")
            .size([0.0, 150.0])
            .horizontal_scrollbar(false)
            .begin() {

            for message in &self.messages {
                let color = match message.message_type {
                    ChatMessageType::Player => [1.0, 1.0, 1.0, 1.0],
                    ChatMessageType::System => [0.8, 0.8, 0.0, 1.0],
                    ChatMessageType::Server => [0.0, 0.8, 1.0, 1.0],
                    ChatMessageType::Error => [1.0, 0.0, 0.0, 1.0],
                };

                let timestamp = message.timestamp.elapsed().as_secs();
                ui.text_colored(color, format!("[{}s] {}: {}", timestamp, message.player_name, message.content));
            }

            if self.auto_scroll {
                ui.set_scroll_here_y_with_ratio(1.0);
            }
        }

        // Chat input
        ui.input_text("##chat_input", &mut self.input_buffer)
            .hint("Type a message...")
            .enter_returns_true(true)
            .build();

        if ui.is_item_deactivated_after_edit() && !self.input_buffer.is_empty() {
            // Send message
            self.input_buffer.clear();
        }

        Ok(())
    }
}

impl SaveLoadUI {
    fn new() -> Self {
        Self {
            world_list: Vec::new(),
            selected_world: None,
            auto_save_enabled: true,
            auto_save_interval: 300, // 5 minutes
            last_save_time: None,
            save_in_progress: false,
            load_in_progress: false,
            backup_count: 3,
        }
    }

    fn update(&mut self, save_manager: &mut SaveManager, _input: &InputManager, _delta_time: f32) -> RobinResult<Option<Vec<MenuAction>>> {
        // Update world list from save manager
        // TODO: Implement world list synchronization

        Ok(None)
    }

    fn render(&mut self) -> RobinResult<()> {
        ui.text("World Management:");

        // Auto-save settings
        ui.checkbox("Auto-save enabled", &mut self.auto_save_enabled);
        ui.same_line();
        ui.text("Interval:");
        ui.same_line();
        ui.set_next_item_width(100.0);
        ui.input_scalar("##autosave_interval", &mut self.auto_save_interval)
            .build();
        ui.same_line();
        ui.text("seconds");

        ui.separator();

        // Save/Load buttons
        if self.save_in_progress {
            ui.text_colored([1.0, 1.0, 0.0, 1.0], "Saving...");
        } else if ui.button("Save World") {
            self.save_in_progress = true;
        }

        ui.same_line();

        if self.load_in_progress {
            ui.text_colored([1.0, 1.0, 0.0, 1.0], "Loading...");
        } else if ui.button("Load World") && self.selected_world.is_some() {
            self.load_in_progress = true;
        }

        ui.separator();

        // World list
        ui.text("Available Worlds:");
        for (i, world) in self.world_list.iter().enumerate() {
            let is_selected = self.selected_world == Some(i);
            if ui.selectable(&world.name, is_selected) {
                self.selected_world = Some(i);
            }

            // Show world info on hover
            if ui.is_item_hovered() {
                ui.tooltip(|| {
                    ui.text(format!("Size: {} KB", world.file_size / 1024));
                    ui.text(format!("Seed: {}", world.metadata.seed));
                    ui.text(format!("Version: {}", world.metadata.version));
                });
            }
        }

        Ok(())
    }
}

impl PerformanceMonitorUI {
    fn new() -> Self {
        Self {
            show_stats: false,
            show_optimization_tips: false,
            performance_history: Vec::new(),
            current_profile: PerformanceProfile::Balanced,
            auto_optimize: false,
            history_length: 60,
        }
    }

    fn update(&mut self, build_mode: &EngineerBuildMode, _delta_time: f32) -> RobinResult<()> {
        // Sample performance data
        let snapshot = PerformanceSnapshot {
            timestamp: Instant::now(),
            fps: 60.0, // TODO: Get real FPS
            frame_time: 16.7, // TODO: Get real frame time
            memory_usage: 0, // TODO: Get real memory usage
            voxel_count: 0, // TODO: Get real voxel count
            chunk_count: 0, // TODO: Get real chunk count
        };

        self.performance_history.push(snapshot);

        // Limit history
        if self.performance_history.len() > self.history_length {
            self.performance_history.remove(0);
        }

        Ok(())
    }

    fn render(&mut self) -> RobinResult<()> {
        ui.checkbox("Show Performance Stats", &mut self.show_stats);
        ui.checkbox("Show Optimization Tips", &mut self.show_optimization_tips);
        ui.checkbox("Auto-optimize", &mut self.auto_optimize);

        ui.separator();

        // Performance profile selection
        ui.text("Performance Profile:");
        let profile_names = ["High Quality", "Balanced", "Performance"];
        let mut current_profile = self.current_profile as usize;

        if ui.combo("##perf_profile", &mut current_profile, &profile_names, |item| std::borrow::Cow::Borrowed(item)) {
            self.current_profile = match current_profile {
                0 => PerformanceProfile::HighQuality,
                1 => PerformanceProfile::Balanced,
                2 => PerformanceProfile::Performance,
                _ => PerformanceProfile::Balanced,
            };
        }

        if self.show_stats {
            ui.separator();
            ui.text("Performance Metrics:");

            if let Some(latest) = self.performance_history.last() {
                ui.text(format!("FPS: {:.1}", latest.fps));
                ui.text(format!("Frame Time: {:.1}ms", latest.frame_time));
                ui.text(format!("Memory: {}MB", latest.memory_usage / (1024 * 1024)));
                ui.text(format!("Voxels: {}", latest.voxel_count));
                ui.text(format!("Chunks: {}", latest.chunk_count));
            }
        }

        Ok(())
    }
}

impl InventoryUI {
    fn new() -> Self {
        Self {
            material_grid: Vec::new(),
            selected_slot: None,
            filter_text: String::new(),
            sort_mode: InventorySortMode::Type,
            show_quantities: true,
            quick_access_slots: [None; 9],
        }
    }

    fn update(&mut self, build_mode: &EngineerBuildMode, _input: &InputManager, _delta_time: f32) -> RobinResult<()> {
        // TODO: Sync with build mode inventory
        Ok(())
    }

    fn render(&mut self) -> RobinResult<()> {
        ui.text("Material Inventory:");

        // Quick access slots (1-9)
        ui.text("Quick Access (1-9):");
        for i in 0..9 {
            let slot_size = [40.0, 40.0];

            if i > 0 {
                ui.same_line();
            }

            let label = format!("{}##quick_{}", i + 1, i);
            if ui.button_with_size(&label, slot_size) {
                // Select quick access slot
            }
        }

        ui.separator();

        // Filter and sort controls
        ui.input_text("Filter", &mut self.filter_text)
            .hint("Search materials...")
            .build();

        ui.same_line();

        let sort_names = ["Alphabetical", "Quantity", "Type", "Recently Used"];
        let mut current_sort = self.sort_mode as usize;
        ui.set_next_item_width(120.0);
        if ui.combo("##sort", &mut current_sort, &sort_names, |item| std::borrow::Cow::Borrowed(item)) {
            self.sort_mode = match current_sort {
                0 => InventorySortMode::Alphabetical,
                1 => InventorySortMode::Quantity,
                2 => InventorySortMode::Type,
                3 => InventorySortMode::RecentlyUsed,
                _ => InventorySortMode::Type,
            };
        }

        ui.checkbox("Show Quantities", &mut self.show_quantities);

        ui.separator();

        // Material grid
        ui.text("Materials:");

        let grid_cols = 4;
        let slot_size = [60.0, 60.0];

        for (i, slot) in self.material_grid.iter().enumerate() {
            if i % grid_cols != 0 {
                ui.same_line();
            }

            let is_selected = self.selected_slot == Some(i);
            let color = if is_selected { [0.2, 0.6, 1.0, 1.0] } else { [0.4, 0.4, 0.4, 1.0] };

            ui.color_button(format!("##slot_{}", i), color, slot_size);

            if ui.is_item_clicked() {
                self.selected_slot = Some(i);
            }

            if ui.is_item_hovered() {
                ui.tooltip(|| {
                    ui.text(format!("{:?}", slot.material));
                    if self.show_quantities {
                        ui.text(format!("Quantity: {}", slot.quantity));
                    }
                });
            }
        }

        Ok(())
    }

    fn set_selected_material(&mut self, material: VoxelType) {
        // Find and select the material slot
        for (i, slot) in self.material_grid.iter().enumerate() {
            if slot.material == material {
                self.selected_slot = Some(i);
                break;
            }
        }
    }
}

impl StatusUI {
    fn new() -> Self {
        Self {
            notifications: Vec::new(),
            status_bar_items: Vec::new(),
            show_tooltips: true,
            notification_duration: Duration::from_secs(5),
        }
    }

    fn update(&mut self, _delta_time: f32) -> RobinResult<()> {
        // Remove expired notifications
        let now = Instant::now();
        self.notifications.retain(|n| now.duration_since(n.timestamp) < n.duration);

        Ok(())
    }

    fn render_status_bar(&mut self) -> RobinResult<()> {
        // Status bar at bottom of screen
        let display_size = ui.io().display_size;
        ui.set_next_window_pos([0.0, display_size[1] - 30.0], Condition::Always);
        ui.set_next_window_size([display_size[0], 30.0], Condition::Always);

        Window::new("status_bar")
            .title_bar(false)
            .resizable(false)
            .movable(false)
            .scrollable(false)
            .build(ui, || {
                for (i, item) in self.status_bar_items.iter().enumerate() {
                    if i > 0 {
                        ui.same_line();
                        ui.text(" | ");
                        ui.same_line();
                    }

                    ui.text_colored(item.color, format!("{}: {}", item.label, item.value));

                    if ui.is_item_hovered() && item.tooltip.is_some() && self.show_tooltips {
                        ui.tooltip_text(item.tooltip.as_ref().unwrap());
                    }
                }
            });

        Ok(())
    }

    fn render_notifications(&mut self) -> RobinResult<()> {
        let display_size = ui.io().display_size;

        for (i, notification) in self.notifications.iter().enumerate() {
            let y_offset = i as f32 * 80.0;
            ui.set_next_window_pos([display_size[0] - 320.0, 50.0 + y_offset], Condition::Always);
            ui.set_next_window_size([300.0, 70.0], Condition::Always);

            let color = match notification.notification_type {
                NotificationType::Info => [0.2, 0.6, 1.0, 1.0],
                NotificationType::Success => [0.2, 1.0, 0.2, 1.0],
                NotificationType::Warning => [1.0, 0.8, 0.2, 1.0],
                NotificationType::Error => [1.0, 0.2, 0.2, 1.0],
                NotificationType::Achievement => [1.0, 0.6, 0.2, 1.0],
            };

            ui.push_style_color(StyleColor::TitleBg, color);
            ui.push_style_color(StyleColor::TitleBgActive, color);

            let window_label = format!("{}##notification_{}", notification.title, notification.id);
            Window::new(&window_label)
                .resizable(false)
                .movable(false)
                .build(ui, || {
                    ui.text_wrapped(&notification.message);

                    for action in &notification.actions {
                        if ui.small_button(&action.label) {
                            // Handle notification action
                        }
                        ui.same_line();
                    }
                });

            ui.pop_style_color();
            ui.pop_style_color();
        }

        Ok(())
    }

    fn add_notification(&mut self, title: String, message: String, notification_type: NotificationType) {
        let id = self.notifications.len() as u32;
        self.notifications.push(Notification {
            id,
            title,
            message,
            notification_type,
            timestamp: Instant::now(),
            duration: self.notification_duration,
            actions: Vec::new(),
        });
    }
}

// Extension traits for existing build mode system
impl EngineerBuildMode {
    /// Get current build mode for UI integration
    pub fn get_current_build_mode(&self) -> BuildMode {
        match self.get_mode() {
            BuildModeState::Build => BuildMode::Single, // Default to single in build mode
            BuildModeState::Test => BuildMode::Single,  // Limited mode in test
            BuildModeState::Play => BuildMode::Single,  // No building in play mode
        }
    }
}

// Extension for mode system
impl crate::engine::build_mode::ModeSystem {
    pub fn get_current_build_mode(&self) -> BuildMode {
        BuildMode::Single // Default implementation
    }
}