// Mini-map panel for world overview

use imgui::*;
use crate::renderer::Camera;
use crate::ui::UIAction;

pub struct MiniMapPanel {
    window_open: bool,
    map_size: f32,
}

impl MiniMapPanel {
    pub fn new() -> Self {
        Self {
            window_open: false, // Start closed
            map_size: 200.0,
        }
    }

    pub fn render(&mut self, ui: &Ui, camera: &Camera) -> Vec<UIAction> {
        let _actions: Vec<UIAction> = Vec::new();

        ui.window("🗺️ Mini Map")
            .size([240.0, 280.0], Condition::FirstUseEver)
            .position([660.0, 80.0], Condition::FirstUseEver)
            .opened(&mut self.window_open)
            .build(|| {
                ui.text("World Overview");
                ui.separator();

                // Map area
                let draw_list = ui.get_window_draw_list();
                let canvas_pos = ui.cursor_screen_pos();
                let canvas_size = [self.map_size, self.map_size];

                // Draw map background
                draw_list
                    .add_rect(
                        canvas_pos,
                        [canvas_pos[0] + canvas_size[0], canvas_pos[1] + canvas_size[1]],
                        [0.2, 0.2, 0.2, 1.0],
                    )
                    .filled(true)
                    .build();

                // Draw grid
                let grid_spacing = 20.0;
                for i in 0..=(self.map_size / grid_spacing) as i32 {
                    let x = canvas_pos[0] + i as f32 * grid_spacing;
                    let y = canvas_pos[1] + i as f32 * grid_spacing;

                    // Vertical lines
                    if x <= canvas_pos[0] + canvas_size[0] {
                        draw_list
                            .add_line(
                                [x, canvas_pos[1]],
                                [x, canvas_pos[1] + canvas_size[1]],
                                [0.4, 0.4, 0.4, 0.5],
                            )
                            .build();
                    }

                    // Horizontal lines
                    if y <= canvas_pos[1] + canvas_size[1] {
                        draw_list
                            .add_line(
                                [canvas_pos[0], y],
                                [canvas_pos[0] + canvas_size[0], y],
                                [0.4, 0.4, 0.4, 0.5],
                            )
                            .build();
                    }
                }

                // Draw player position
                let player_x = canvas_pos[0] + canvas_size[0] * 0.5 + camera.eye.x * 2.0;
                let player_y = canvas_pos[1] + canvas_size[1] * 0.5 + camera.eye.z * 2.0;

                // Clamp to canvas bounds
                let player_x = player_x.max(canvas_pos[0]).min(canvas_pos[0] + canvas_size[0]);
                let player_y = player_y.max(canvas_pos[1]).min(canvas_pos[1] + canvas_size[1]);

                // Draw player dot
                draw_list
                    .add_circle([player_x, player_y], 4.0, [1.0, 0.0, 0.0, 1.0])
                    .filled(true)
                    .build();

                // Draw player direction
                let direction_length = 12.0;
                let direction_x = camera.yaw.cos() * direction_length;
                let direction_y = camera.yaw.sin() * direction_length;

                draw_list
                    .add_line(
                        [player_x, player_y],
                        [player_x + direction_x, player_y + direction_y],
                        [1.0, 1.0, 0.0, 1.0],
                    )
                    .thickness(2.0)
                    .build();

                ui.dummy(canvas_size);

                ui.spacing();
                ui.separator();
                ui.text("Player Info");
                ui.text(format!("X: {:.1}", camera.eye.x));
                ui.text(format!("Y: {:.1}", camera.eye.y));
                ui.text(format!("Z: {:.1}", camera.eye.z));
                ui.text(format!("Yaw: {:.1}°", camera.yaw.to_degrees()));
                ui.text(format!("Pitch: {:.1}°", camera.pitch.to_degrees()));
            });

        _actions
    }

    pub fn is_open(&self) -> bool {
        self.window_open
    }

    pub fn set_open(&mut self, open: bool) {
        self.window_open = open;
    }
}