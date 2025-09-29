// Community Panel for Robin Engine
// Provides access to all social building features

use imgui::*;
use uuid::Uuid;
use std::collections::HashMap;

use robin::engine::{
    community::{
        CommunityManager, CommunityEvent,
        project_sharing::{ProjectSummary, CreateProjectParams, Visibility},
        social_rooms::{RoomSummary, CreateRoomParams, SessionType},
        community_gallery::{SubmissionSummary, BrowseFilter, SortBy},
        user_profiles::{PublicProfile, LeaderboardCriteria, LeaderboardEntry},
        collaboration::{SessionSummary as CollabSessionSummary, SessionParams},
    },
    error::RobinResult,
};

use super::UIAction;

/// Community panel providing access to all social features
pub struct CommunityPanel {
    // Panel state
    show_panel: bool,
    active_tab: CommunityTab,

    // UI state
    scroll_positions: HashMap<CommunityTab, f32>,
    search_query: String,

    // Project sharing state
    new_project_name: String,
    new_project_description: String,
    project_list: Vec<ProjectSummary>,

    // Social rooms state
    new_room_name: String,
    new_room_description: String,
    room_password: String,
    room_list: Vec<RoomSummary>,

    // Gallery state
    gallery_items: Vec<SubmissionSummary>,
    gallery_filter: BrowseFilter,

    // Profile state
    user_profile: Option<PublicProfile>,
    leaderboard: Vec<LeaderboardEntry>,
    leaderboard_criteria: LeaderboardCriteria,

    // Collaboration state
    active_sessions: Vec<CollabSessionSummary>,
    new_session_name: String,
    new_session_description: String,

    // Current user ID (mock for demo)
    current_user_id: Uuid,
}

/// Community panel tabs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommunityTab {
    Projects,
    Rooms,
    Gallery,
    Profile,
    Collaborate,
}

impl CommunityPanel {
    pub fn new() -> Self {
        let mut scroll_positions = HashMap::new();
        scroll_positions.insert(CommunityTab::Projects, 0.0);
        scroll_positions.insert(CommunityTab::Rooms, 0.0);
        scroll_positions.insert(CommunityTab::Gallery, 0.0);
        scroll_positions.insert(CommunityTab::Profile, 0.0);
        scroll_positions.insert(CommunityTab::Collaborate, 0.0);

        Self {
            show_panel: false,
            active_tab: CommunityTab::Projects,
            scroll_positions,
            search_query: String::new(),

            new_project_name: String::new(),
            new_project_description: String::new(),
            project_list: Vec::new(),

            new_room_name: String::new(),
            new_room_description: String::new(),
            room_password: String::new(),
            room_list: Vec::new(),

            gallery_items: Vec::new(),
            gallery_filter: BrowseFilter::default(),

            user_profile: None,
            leaderboard: Vec::new(),
            leaderboard_criteria: LeaderboardCriteria::Level,

            active_sessions: Vec::new(),
            new_session_name: String::new(),
            new_session_description: String::new(),

            current_user_id: Uuid::new_v4(), // Mock user ID
        }
    }

    pub fn toggle(&mut self) {
        self.show_panel = !self.show_panel;
    }

    pub fn is_visible(&self) -> bool {
        self.show_panel
    }

    pub fn render(&mut self, ui: &Ui) -> Vec<UIAction> {
        if !self.show_panel {
            return Vec::new();
        }

        let mut actions = Vec::new();

        // Main community window
        let mut window_open = self.show_panel;

        ui.window("🌍 Robin Community")
            .size([600.0, 500.0], Condition::FirstUseEver)
            .position([50.0, 50.0], Condition::FirstUseEver)
            .opened(&mut window_open)
            .build(|| {
                // Tab bar
                self.render_tab_bar(ui);

                ui.separator();

                // Content area based on active tab
                match self.active_tab {
                    CommunityTab::Projects => self.render_projects_tab(ui, &mut actions),
                    CommunityTab::Rooms => self.render_rooms_tab(ui, &mut actions),
                    CommunityTab::Gallery => self.render_gallery_tab(ui, &mut actions),
                    CommunityTab::Profile => self.render_profile_tab(ui, &mut actions),
                    CommunityTab::Collaborate => self.render_collaborate_tab(ui, &mut actions),
                }
            });

        self.show_panel = window_open;
        actions
    }

    fn render_tab_bar(&mut self, ui: &Ui) {
        ui.child_window("TabBar")
            .size([0.0, 40.0])
            .build(|| {
                let tab_width = ui.content_region_avail()[0] / 5.0;

                // Projects tab
                if ui.button_with_size("📁 Projects", [tab_width - 5.0, 30.0]) {
                    self.active_tab = CommunityTab::Projects;
                }
                ui.same_line();

                // Rooms tab
                if ui.button_with_size("🏠 Rooms", [tab_width - 5.0, 30.0]) {
                    self.active_tab = CommunityTab::Rooms;
                }
                ui.same_line();

                // Gallery tab
                if ui.button_with_size("🎨 Gallery", [tab_width - 5.0, 30.0]) {
                    self.active_tab = CommunityTab::Gallery;
                }
                ui.same_line();

                // Profile tab
                if ui.button_with_size("👤 Profile", [tab_width - 5.0, 30.0]) {
                    self.active_tab = CommunityTab::Profile;
                }
                ui.same_line();

                // Collaborate tab
                if ui.button_with_size("🤝 Collaborate", [tab_width - 5.0, 30.0]) {
                    self.active_tab = CommunityTab::Collaborate;
                }
            });
    }

    fn render_projects_tab(&mut self, ui: &Ui, actions: &mut Vec<UIAction>) {
        // Header with search and create button
        ui.columns(2, "ProjectsHeader", false);

        // Search box
        ui.text("Search Projects:");
        ui.set_column_width(0, 200.0);
        ui.input_text("##search", &mut self.search_query)
            .build();

        ui.next_column();

        // Create project button
        if ui.button("➕ Create Project") {
            // Show create project modal
            ui.open_popup("CreateProject");
        }

        ui.columns(1, "", false);
        ui.separator();

        // Create project modal
        ui.modal("CreateProject")
            .always_auto_resize(true)
            .build(|| {
                ui.text("Create New Project");
                ui.separator();

                ui.input_text("Name", &mut self.new_project_name)
                    .build();

                ui.input_text_multiline("Description", &mut self.new_project_description, [300.0, 80.0])
                    .build();

                ui.separator();

                if ui.button("Create") {
                    if !self.new_project_name.trim().is_empty() {
                        actions.push(UIAction::Community(CommunityAction::CreateProject {
                            name: self.new_project_name.clone(),
                            description: self.new_project_description.clone(),
                        }));
                        self.new_project_name.clear();
                        self.new_project_description.clear();
                        ui.close_current_popup();
                    }
                }
                ui.same_line();
                if ui.button("Cancel") {
                    ui.close_current_popup();
                }
            });

        // Projects list
        ui.child_window("ProjectsList")
            .build(|| {
                if self.project_list.is_empty() {
                    ui.text_colored([0.6, 0.6, 0.6, 1.0], "No projects found. Create your first project!");
                } else {
                    for project in &self.project_list {
                        self.render_project_item(ui, project, actions);
                    }
                }
            });
    }

    fn render_project_item(&self, ui: &Ui, project: &ProjectSummary, actions: &mut Vec<UIAction>) {
        ui.group(|| {
            ui.text(&project.name);
            ui.text_disabled(&project.description);

            ui.separator();

            // Project stats
            ui.columns(3, &format!("ProjectStats{}", project.id), false);
            ui.text(format!("👁 {}", project.view_count));
            ui.next_column();
            ui.text(format!("⬇ {}", project.download_count));
            ui.next_column();
            ui.text(format!("⭐ {:.1}", project.average_rating));
            ui.columns(1, "", false);

            // Action buttons
            if ui.small_button("View") {
                actions.push(UIAction::Community(CommunityAction::ViewProject(project.id)));
            }
            ui.same_line();
            if ui.small_button("Download") {
                actions.push(UIAction::Community(CommunityAction::DownloadProject(project.id)));
            }
            ui.same_line();
            if ui.small_button("Fork") {
                actions.push(UIAction::Community(CommunityAction::ForkProject(project.id)));
            }
        });

        ui.separator();
    }

    fn render_rooms_tab(&mut self, ui: &Ui, actions: &mut Vec<UIAction>) {
        // Header
        ui.columns(2, "RoomsHeader", false);

        ui.text("Active Social Rooms:");
        ui.set_column_width(0, 200.0);

        ui.next_column();

        if ui.button("🏠 Create Room") {
            ui.open_popup("CreateRoom");
        }

        ui.columns(1, "", false);
        ui.separator();

        // Create room modal
        ui.modal("CreateRoom")
            .always_auto_resize(true)
            .build(|| {
                ui.text("Create Social Room");
                ui.separator();

                ui.input_text("Room Name", &mut self.new_room_name)
                    .build();

                ui.input_text_multiline("Description", &mut self.new_room_description, [300.0, 60.0])
                    .build();

                ui.input_text("Password (Optional)", &mut self.room_password)
                    .password(true)
                    .build();

                ui.separator();

                if ui.button("Create") {
                    if !self.new_room_name.trim().is_empty() {
                        actions.push(UIAction::Community(CommunityAction::CreateRoom {
                            name: self.new_room_name.clone(),
                            description: self.new_room_description.clone(),
                            password: if self.room_password.is_empty() { None } else { Some(self.room_password.clone()) },
                        }));
                        self.new_room_name.clear();
                        self.new_room_description.clear();
                        self.room_password.clear();
                        ui.close_current_popup();
                    }
                }
                ui.same_line();
                if ui.button("Cancel") {
                    ui.close_current_popup();
                }
            });

        // Rooms list
        ui.child_window("RoomsList")
            .build(|| {
                if self.room_list.is_empty() {
                    ui.text_colored([0.6, 0.6, 0.6, 1.0], "No active rooms. Create or join a room to start building together!");
                } else {
                    for room in &self.room_list {
                        self.render_room_item(ui, room, actions);
                    }
                }
            });
    }

    fn render_room_item(&self, ui: &Ui, room: &RoomSummary, actions: &mut Vec<UIAction>) {
        ui.group(|| {
            ui.text(&room.name);
            ui.text_disabled(&room.description);

            // Room info
            ui.columns(3, &format!("RoomInfo{}", room.id), false);
            ui.text(format!("👥 {}/{}", room.participant_count, room.max_participants));
            ui.next_column();
            ui.text(if room.requires_invite { "🔒 Private" } else { "🌐 Public" });
            ui.next_column();
            ui.text(if room.voice_enabled { "🎤 Voice" } else { "💬 Text" });
            ui.columns(1, "", false);

            // Join button
            if ui.button("🚪 Join Room") {
                actions.push(UIAction::Community(CommunityAction::JoinRoom(room.id)));
            }
        });

        ui.separator();
    }

    fn render_gallery_tab(&mut self, ui: &Ui, actions: &mut Vec<UIAction>) {
        // Header with sort options
        ui.text("Community Gallery");
        ui.same_line();

        // Sort dropdown
        let sort_options = ["Newest", "Popular", "Trending", "Most Viewed", "Rating"];
        let mut current_sort = match self.gallery_filter.sort_by {
            SortBy::Newest => 0,
            SortBy::Popular => 1,
            SortBy::Trending => 2,
            SortBy::MostViewed => 3,
            SortBy::Rating => 4,
        };

        ui.combo("Sort", &mut current_sort, &sort_options, |item| (*item).into());

        self.gallery_filter.sort_by = match current_sort {
            0 => SortBy::Newest,
            1 => SortBy::Popular,
            2 => SortBy::Trending,
            3 => SortBy::MostViewed,
            4 => SortBy::Rating,
            _ => SortBy::Newest,
        };

        ui.separator();

        // Gallery grid
        ui.child_window("GalleryGrid")
            .build(|| {
                if self.gallery_items.is_empty() {
                    ui.text_colored([0.6, 0.6, 0.6, 1.0], "No submissions in the gallery yet. Be the first to share your creation!");
                } else {
                    let columns = 2;
                    ui.columns(columns, "GalleryColumns", false);

                    for (i, item) in self.gallery_items.iter().enumerate() {
                        self.render_gallery_item(ui, item, actions);

                        if i % columns == columns - 1 && i + 1 < self.gallery_items.len() {
                            ui.next_column();
                        }
                    }

                    ui.columns(1, "", false);
                }
            });
    }

    fn render_gallery_item(&self, ui: &Ui, item: &SubmissionSummary, actions: &mut Vec<UIAction>) {
        ui.group(|| {
            // Title
            ui.text(&item.title);
            ui.text_disabled(&item.description);

            // Thumbnail placeholder
            ui.button_with_size("📷 Thumbnail", [120.0, 90.0]);

            // Stats
            ui.columns(3, &format!("GalleryStats{}", item.id), false);
            ui.text(format!("👁 {}", item.view_count));
            ui.next_column();
            ui.text(format!("⬇ {}", item.download_count));
            ui.next_column();
            ui.text(format!("⭐ {:.1}", item.average_rating));
            ui.columns(1, "", false);

            // Actions
            if ui.small_button("View") {
                actions.push(UIAction::Community(CommunityAction::ViewGalleryItem(item.id)));
            }
            ui.same_line();
            if ui.small_button("Like") {
                actions.push(UIAction::Community(CommunityAction::LikeGalleryItem(item.id)));
            }
        });

        ui.separator();
    }

    fn render_profile_tab(&mut self, ui: &Ui, actions: &mut Vec<UIAction>) {
        ui.columns(2, "ProfileLayout", false);

        // Left column: User profile
        ui.set_column_width(0, 250.0);

        ui.text("👤 Your Profile");
        ui.separator();

        if let Some(profile) = &self.user_profile {
            ui.text(&profile.display_name);
            if let Some(bio) = &profile.bio {
                ui.text_wrapped(bio);
            }

            ui.separator();
            ui.text(format!("Level: {}", profile.level));
            ui.text(format!("Achievements: {}", profile.achievements.len()));
            if let Some(reputation) = profile.reputation {
                ui.text(format!("Reputation: {}", reputation));
            }

            if ui.button("Edit Profile") {
                actions.push(UIAction::Community(CommunityAction::EditProfile));
            }
        } else {
            ui.text_colored([0.6, 0.6, 0.6, 1.0], "Loading profile...");
        }

        ui.next_column();

        // Right column: Leaderboard
        ui.text("🏆 Leaderboards");
        ui.separator();

        // Leaderboard criteria selection
        let criteria_options = ["Level", "Experience", "Build Time", "Voxels Placed", "Projects", "Reputation"];
        let mut current_criteria = match self.leaderboard_criteria {
            LeaderboardCriteria::Level => 0,
            LeaderboardCriteria::ExperiencePoints => 1,
            LeaderboardCriteria::BuildTime => 2,
            LeaderboardCriteria::VoxelsPlaced => 3,
            LeaderboardCriteria::ProjectsCreated => 4,
            LeaderboardCriteria::Reputation => 5,
            LeaderboardCriteria::Achievements => 6,
        };

        if ui.combo("Category", &mut current_criteria, &criteria_options, |item| (*item).into()) {
            self.leaderboard_criteria = match current_criteria {
                0 => LeaderboardCriteria::Level,
                1 => LeaderboardCriteria::ExperiencePoints,
                2 => LeaderboardCriteria::BuildTime,
                3 => LeaderboardCriteria::VoxelsPlaced,
                4 => LeaderboardCriteria::ProjectsCreated,
                5 => LeaderboardCriteria::Reputation,
                _ => LeaderboardCriteria::Level,
            };
            actions.push(UIAction::Community(CommunityAction::RefreshLeaderboard(self.leaderboard_criteria)));
        }

        // Leaderboard list
        ui.child_window("Leaderboard")
            .build(|| {
                for entry in &self.leaderboard {
                    ui.text(format!("{}. {} ({:.0})", entry.rank, entry.display_name, entry.score));
                }
            });

        ui.columns(1, "", false);
    }

    fn render_collaborate_tab(&mut self, ui: &Ui, actions: &mut Vec<UIAction>) {
        ui.text("🤝 Active Collaboration Sessions");

        if ui.button("➕ Start Session") {
            ui.open_popup("CreateSession");
        }

        ui.separator();

        // Create session modal
        ui.modal("CreateSession")
            .always_auto_resize(true)
            .build(|| {
                ui.text("Start Collaboration Session");
                ui.separator();

                ui.input_text("Session Name", &mut self.new_session_name)
                    .build();

                ui.input_text_multiline("Description", &mut self.new_session_description, [300.0, 60.0])
                    .build();

                ui.separator();

                if ui.button("Start") {
                    if !self.new_session_name.trim().is_empty() {
                        actions.push(UIAction::Community(CommunityAction::StartCollaboration {
                            name: self.new_session_name.clone(),
                            description: self.new_session_description.clone(),
                        }));
                        self.new_session_name.clear();
                        self.new_session_description.clear();
                        ui.close_current_popup();
                    }
                }
                ui.same_line();
                if ui.button("Cancel") {
                    ui.close_current_popup();
                }
            });

        // Active sessions list
        ui.child_window("ActiveSessions")
            .build(|| {
                if self.active_sessions.is_empty() {
                    ui.text_colored([0.6, 0.6, 0.6, 1.0], "No active collaboration sessions. Start one to build together!");
                } else {
                    for session in &self.active_sessions {
                        ui.group(|| {
                            ui.text(&session.name);
                            ui.text_disabled(&session.description);
                            ui.text(format!("👥 {} participants", session.participant_count));

                            if ui.button("Join") {
                                actions.push(UIAction::Community(CommunityAction::JoinCollaboration(session.id)));
                            }
                        });
                        ui.separator();
                    }
                }
            });
    }
}

/// Community-specific UI actions
#[derive(Debug, Clone)]
pub enum CommunityAction {
    // Projects
    CreateProject { name: String, description: String },
    ViewProject(Uuid),
    DownloadProject(Uuid),
    ForkProject(Uuid),

    // Rooms
    CreateRoom { name: String, description: String, password: Option<String> },
    JoinRoom(Uuid),

    // Gallery
    ViewGalleryItem(Uuid),
    LikeGalleryItem(Uuid),

    // Profile
    EditProfile,
    RefreshLeaderboard(LeaderboardCriteria),

    // Collaboration
    StartCollaboration { name: String, description: String },
    JoinCollaboration(Uuid),
}

// Extend existing UIAction enum
impl From<CommunityAction> for UIAction {
    fn from(action: CommunityAction) -> Self {
        UIAction::Community(action)
    }
}