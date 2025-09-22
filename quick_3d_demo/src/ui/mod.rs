// UI system for Robin Engine using ImGui
// Provides overlays for inventory, build tools, and settings

use imgui::*;
use crate::game::{VoxelType, BuildMode};

pub mod inventory;
pub mod build_tools;
pub mod mini_map;
pub mod simple_ui;

pub use inventory::InventoryPanel;
pub use build_tools::BuildToolsPanel;
pub use mini_map::MiniMapPanel;

pub struct UISystem {
    context: Context,
    inventory_panel: InventoryPanel,
    build_tools_panel: BuildToolsPanel,
    mini_map_panel: MiniMapPanel,
    show_ui: bool,
    show_inventory: bool,
    show_build_tools: bool,
    show_mini_map: bool,
    show_settings: bool,
    font_texture_id: Option<TextureId>,
    font_texture_data: Option<Vec<u8>>,
    font_texture_width: u32,
    font_texture_height: u32,
}

impl UISystem {
    pub fn new() -> Self {
        let mut context = Context::create();
        context.set_ini_filename(None); // Don't save window positions

        // Build font atlas before configuring style
        let (font_texture_data, font_texture_width, font_texture_height) = Self::build_font_atlas(&mut context);

        // Configure ImGui style for dark theme
        Self::setup_dark_style(&mut context);

        Self {
            context,
            inventory_panel: InventoryPanel::new(),
            build_tools_panel: BuildToolsPanel::new(),
            mini_map_panel: MiniMapPanel::new(),
            show_ui: true,
            show_inventory: true,
            show_build_tools: true,
            show_mini_map: false,
            show_settings: false,
            font_texture_id: None, // Will be set later when uploaded to Metal
            font_texture_data: Some(font_texture_data),
            font_texture_width,
            font_texture_height,
        }
    }

    fn build_font_atlas(context: &mut Context) -> (Vec<u8>, u32, u32) {
        // Get the font atlas from the context
        let fonts = context.fonts();

        // Add default font to the atlas
        fonts.add_font(&[FontSource::DefaultFontData { config: None }]);

        // Build the atlas and get texture data
        let font_atlas = fonts.build_rgba32_texture();

        // Extract texture data, width, and height
        let texture_data = font_atlas.data.to_vec();
        let width = font_atlas.width;
        let height = font_atlas.height;

        println!("📝 Built font atlas: {}x{} pixels, {} bytes", width, height, texture_data.len());

        (texture_data, width, height)
    }

    fn setup_dark_style(context: &mut Context) {
        let style = context.style_mut();

        // Colors
        style.colors[StyleColor::Text as usize] = [0.90, 0.90, 0.90, 1.00];
        style.colors[StyleColor::TextDisabled as usize] = [0.60, 0.60, 0.60, 1.00];
        style.colors[StyleColor::WindowBg as usize] = [0.10, 0.10, 0.10, 0.85];
        style.colors[StyleColor::ChildBg as usize] = [0.00, 0.00, 0.00, 0.00];
        style.colors[StyleColor::PopupBg as usize] = [0.05, 0.05, 0.05, 0.90];
        style.colors[StyleColor::Border as usize] = [0.70, 0.70, 0.70, 0.65];
        style.colors[StyleColor::BorderShadow as usize] = [0.00, 0.00, 0.00, 0.00];
        style.colors[StyleColor::FrameBg as usize] = [0.20, 0.20, 0.20, 0.80];
        style.colors[StyleColor::FrameBgHovered as usize] = [0.25, 0.25, 0.25, 0.80];
        style.colors[StyleColor::FrameBgActive as usize] = [0.30, 0.30, 0.30, 0.80];
        style.colors[StyleColor::TitleBg as usize] = [0.15, 0.15, 0.15, 1.00];
        style.colors[StyleColor::TitleBgActive as usize] = [0.20, 0.20, 0.20, 1.00];
        style.colors[StyleColor::TitleBgCollapsed as usize] = [0.15, 0.15, 0.15, 0.75];
        style.colors[StyleColor::MenuBarBg as usize] = [0.15, 0.15, 0.15, 1.00];
        style.colors[StyleColor::ScrollbarBg as usize] = [0.10, 0.10, 0.10, 0.80];
        style.colors[StyleColor::ScrollbarGrab as usize] = [0.30, 0.30, 0.30, 1.00];
        style.colors[StyleColor::ScrollbarGrabHovered as usize] = [0.40, 0.40, 0.40, 1.00];
        style.colors[StyleColor::ScrollbarGrabActive as usize] = [0.50, 0.50, 0.50, 1.00];
        style.colors[StyleColor::CheckMark as usize] = [0.70, 0.70, 0.70, 1.00];
        style.colors[StyleColor::SliderGrab as usize] = [0.70, 0.70, 0.70, 1.00];
        style.colors[StyleColor::SliderGrabActive as usize] = [0.80, 0.80, 0.80, 1.00];
        style.colors[StyleColor::Button as usize] = [0.20, 0.20, 0.20, 1.00];
        style.colors[StyleColor::ButtonHovered as usize] = [0.30, 0.30, 0.30, 1.00];
        style.colors[StyleColor::ButtonActive as usize] = [0.40, 0.40, 0.40, 1.00];
        style.colors[StyleColor::Header as usize] = [0.25, 0.25, 0.25, 1.00];
        style.colors[StyleColor::HeaderHovered as usize] = [0.35, 0.35, 0.35, 1.00];
        style.colors[StyleColor::HeaderActive as usize] = [0.45, 0.45, 0.45, 1.00];

        // Spacing and rounding
        style.window_rounding = 4.0;
        style.frame_rounding = 4.0;
        style.grab_rounding = 4.0;
        style.window_padding = [8.0, 8.0];
        style.frame_padding = [4.0, 3.0];
        style.item_spacing = [8.0, 4.0];
        style.item_inner_spacing = [4.0, 4.0];
    }

    pub fn toggle_ui(&mut self) {
        self.show_ui = !self.show_ui;
    }

    pub fn is_ui_visible(&self) -> bool {
        self.show_ui
    }

    pub fn handle_key_input(&mut self, key_code: u16) {
        use crate::window::key_codes;

        match key_code {
            key_codes::TAB => self.toggle_ui(),
            key_codes::KEY_1 => self.show_inventory = !self.show_inventory,
            key_codes::KEY_2 => self.show_build_tools = !self.show_build_tools,
            key_codes::KEY_3 => self.show_mini_map = !self.show_mini_map,
            key_codes::ESCAPE => self.show_settings = !self.show_settings,
            _ => {}
        }
    }

    /*
    pub fn update_and_render(
        &mut self,
        window_size: CGSize,
        build_system: &mut EngineerBuildSystem,
        camera: &Camera,
        delta_time: f32,
    ) -> (Vec<UIAction>, Option<&imgui::DrawData>) {
        if !self.show_ui {
            return (Vec::new(), None);
        }

        let mut actions = Vec::new();

        // Update ImGui frame
        {
            let io = self.context.io_mut();
            io.display_size = [window_size.width as f32, window_size.height as f32];
            io.delta_time = delta_time;
        }

        // Build UI and get draw data in one scope
        let draw_data = {
            let ui = self.context.frame();

            // Main menu bar
            if let Some(menu_bar) = ui.begin_main_menu_bar() {
            if let Some(_menu) = ui.begin_menu("View") {
                if ui.menu_item_config("Inventory").selected(self.show_inventory).build() {
                    self.show_inventory = !self.show_inventory;
                }
                if ui.menu_item_config("Build Tools").selected(self.show_build_tools).build() {
                    self.show_build_tools = !self.show_build_tools;
                }
                if ui.menu_item_config("Mini Map").selected(self.show_mini_map).build() {
                    self.show_mini_map = !self.show_mini_map;
                }
                ui.separator();
                if ui.menu_item_config("Settings").selected(self.show_settings).build() {
                    self.show_settings = !self.show_settings;
                }
            }
            if let Some(_menu) = ui.begin_menu("Help") {
                if ui.menu_item("Controls") {
                    // TODO: Show controls help
                }
                if ui.menu_item("About") {
                    // TODO: Show about dialog
                }
            }
            menu_bar.end();
        }

        // Debug window to verify UI is working
        ui.window("Debug")
            .size([200.0, 100.0], Condition::FirstUseEver)
            .position([10.0, 50.0], Condition::FirstUseEver)
            .build(|| {
                ui.text("UI System Active");
                ui.text(format!("Inventory: {}", self.show_inventory));
                ui.text(format!("Build Tools: {}", self.show_build_tools));
                ui.text(format!("Mini Map: {}", self.show_mini_map));
            });

        // Render panels
        if self.show_inventory {
            actions.extend(self.inventory_panel.render(&ui, build_system));
        }

        if self.show_build_tools {
            actions.extend(self.build_tools_panel.render(&ui, build_system));
        }

        if self.show_mini_map {
            actions.extend(self.mini_map_panel.render(&ui, camera));
        }

        if self.show_settings {
            // Create a simple inline settings window to avoid borrowing issues
            let mut settings_open = self.show_settings;
            if let Some(_window) = ui.window("Settings")
                .size([300.0, 400.0], Condition::FirstUseEver)
                .position([50.0, 100.0], Condition::FirstUseEver)
                .opened(&mut settings_open)
                .begin() {

                ui.text("Graphics Settings");
                ui.separator();
                ui.text("- Render Distance: [slider]");
                ui.text("- Shadow Quality: [dropdown]");
                ui.text("- Texture Quality: [dropdown]");

                ui.spacing();
                ui.text("Controls");
                ui.separator();
                ui.text("WASD - Movement");
                ui.text("Mouse - Look around");
                ui.text("Space/Shift - Up/Down");
                ui.text("Left Click - Remove blocks");
                ui.text("Right Click - Place blocks");
                ui.text("B - Cycle build mode");
                ui.text("T - Change template");
                ui.text("G - Toggle grid snap");
                ui.text("Z/Y - Undo/Redo");
                ui.text("Tab - Toggle UI");
                ui.text("1-8 - Select materials");

                ui.spacing();
                if ui.button("Close") {
                    settings_open = false;
                }
            });
            self.show_settings = settings_open;
        }

            // UI frame scope ends here, dropping ui and completing the frame
        };

        // Now render to get draw data
        let render_data = self.context.render();

        (actions, Some(render_data))
    }
    */


    pub fn context(&self) -> &Context {
        &self.context
    }

    pub fn context_mut(&mut self) -> &mut Context {
        &mut self.context
    }

    pub fn get_font_texture_data(&self) -> Option<&Vec<u8>> {
        self.font_texture_data.as_ref()
    }

    pub fn get_font_texture_dimensions(&self) -> (u32, u32) {
        (self.font_texture_width, self.font_texture_height)
    }

    pub fn set_font_texture_id(&mut self, texture_id: TextureId) {
        self.font_texture_id = Some(texture_id);
        // Set the texture ID in the font atlas
        self.context.fonts().tex_id = texture_id;
    }

}

#[derive(Debug, Clone)]
pub enum UIAction {
    SelectMaterial(VoxelType),
    SetBuildMode(BuildMode),
    ToggleGridSnap,
    Undo,
    Redo,
    SaveWorld,
    LoadWorld,
    ClearWorld,
}