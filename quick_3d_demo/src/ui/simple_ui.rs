// Simple UI system for testing ImGui Metal integration
use imgui::*;
use core_graphics::geometry::CGSize;
use crate::game::{EngineerBuildSystem, VoxelType};
use crate::renderer::Camera;

#[derive(Debug, Clone)]
pub enum UIAction {
    SelectMaterial(VoxelType),
    ToggleBuildMode,
    Undo,
    Redo,
    SetTimeSpeed(f32),
    SetTimeOfDay(f32),
    ToggleTimePause,
}

pub struct SimpleUISystem {
    context: Context,
    show_ui: bool,
    font_texture_id: Option<TextureId>,
    font_texture_data: Option<Vec<u8>>,
    font_texture_width: u32,
    font_texture_height: u32,
}

impl SimpleUISystem {
    pub fn new() -> Self {
        let mut context = Context::create();
        context.set_ini_filename(None);

        // Build font atlas
        let (font_texture_data, font_texture_width, font_texture_height) = Self::build_font_atlas(&mut context);

        Self {
            context,
            show_ui: true,
            font_texture_id: None,
            font_texture_data: Some(font_texture_data),
            font_texture_width,
            font_texture_height,
        }
    }

    fn build_font_atlas(context: &mut Context) -> (Vec<u8>, u32, u32) {
        let fonts = context.fonts();
        fonts.add_font(&[FontSource::DefaultFontData { config: None }]);
        let font_atlas = fonts.build_rgba32_texture();
        let texture_data = font_atlas.data.to_vec();
        (texture_data, font_atlas.width, font_atlas.height)
    }

    pub fn get_font_texture_data(&self) -> Option<&[u8]> {
        self.font_texture_data.as_deref()
    }

    pub fn get_font_texture_dimensions(&self) -> (u32, u32) {
        (self.font_texture_width, self.font_texture_height)
    }

    pub fn set_font_texture_id(&mut self, texture_id: TextureId) {
        self.font_texture_id = Some(texture_id);
        self.context.fonts().tex_id = texture_id;
    }

    pub fn handle_key_input(&mut self, key_code: u16) {
        match key_code {
            48 => self.show_ui = !self.show_ui, // Tab key
            _ => {}
        }
    }

    pub fn update_and_render(
        &mut self,
        window_size: CGSize,
        _build_system: &mut EngineerBuildSystem,
        camera: &Camera,
        delta_time: f32,
        _time_of_day: f32,
        time_speed: f32,
        time_paused: bool,
        time_string: &str,
        day_phase: &str,
    ) -> (Vec<UIAction>, Option<&imgui::DrawData>) {
        let mut actions = Vec::new();

        // CRITICAL: Always maintain proper ImGui frame lifecycle
        // Update IO state first
        {
            let io = self.context.io_mut();
            io.display_size = [window_size.width as f32, window_size.height as f32];
            io.delta_time = delta_time;
        }

        // Create frame - this must be done every frame to maintain lifecycle
        let ui = self.context.frame();

        if self.show_ui {
            // Build debug window when UI is visible
            ui.window("Robin Engine Debug")
                .size([400.0, 300.0], Condition::FirstUseEver)
                .position([10.0, 10.0], Condition::FirstUseEver)
                .build(|| {
                    ui.text("🎨 UI System Working!");
                    ui.separator();
                    ui.text(format!("FPS: {:.1}", 1.0 / delta_time));
                    ui.text(format!("Window: {:.0}x{:.0}", window_size.width, window_size.height));
                    ui.text(format!("Camera: {:.1}, {:.1}, {:.1}", camera.eye.x, camera.eye.y, camera.eye.z));

                    ui.separator();
                    ui.text("🌅 Time of Day System:");
                    ui.text(format!("Current Time: {}", time_string));
                    ui.text(format!("Day Phase: {}", day_phase));

                    let pause_text = if time_paused { "⏸️ Paused" } else { "▶️ Running" };
                    ui.text(format!("Status: {}", pause_text));
                    ui.text(format!("Speed: {:.1}x", time_speed));

                    // Time controls
                    ui.spacing();
                    if ui.button(if time_paused { "▶️ Resume" } else { "⏸️ Pause" }) {
                        actions.push(UIAction::ToggleTimePause);
                    }

                    ui.same_line();
                    if ui.button("🌅 Dawn") {
                        actions.push(UIAction::SetTimeOfDay(6.0));
                    }
                    ui.same_line();
                    if ui.button("☀️ Noon") {
                        actions.push(UIAction::SetTimeOfDay(12.0));
                    }
                    ui.same_line();
                    if ui.button("🌇 Dusk") {
                        actions.push(UIAction::SetTimeOfDay(18.0));
                    }
                    ui.same_line();
                    if ui.button("🌙 Night") {
                        actions.push(UIAction::SetTimeOfDay(0.0));
                    }

                    // Speed controls
                    ui.spacing();
                    ui.text("Time Speed:");
                    if ui.button("0.5x") {
                        actions.push(UIAction::SetTimeSpeed(0.5));
                    }
                    ui.same_line();
                    if ui.button("1x") {
                        actions.push(UIAction::SetTimeSpeed(1.0));
                    }
                    ui.same_line();
                    if ui.button("2x") {
                        actions.push(UIAction::SetTimeSpeed(2.0));
                    }
                    ui.same_line();
                    if ui.button("5x") {
                        actions.push(UIAction::SetTimeSpeed(5.0));
                    }

                    ui.separator();
                    ui.text("Controls:");
                    ui.text("WASD - Move");
                    ui.text("Mouse - Look");
                    ui.text("Left Click - Remove blocks");
                    ui.text("Right Click - Place blocks");
                    ui.text("B - Build mode");
                    ui.text("Tab - Toggle UI");
                });
        }
        // If UI is hidden, we still need to complete the frame lifecycle
        // but don't build any windows

        // Complete the frame lifecycle (UI token drops automatically)

        // Always call render() to complete the frame
        let draw_data = self.context.render();

        // Return draw data only if UI is visible, None if hidden
        let result_draw_data = if self.show_ui { Some(draw_data) } else { None };

        (actions, result_draw_data)
    }
}