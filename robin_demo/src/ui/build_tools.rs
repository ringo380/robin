// Build tools panel for construction controls

use imgui::*;
use crate::game::{EngineerBuildSystem, BuildMode};
use crate::ui::UIAction;

pub struct BuildToolsPanel {
    window_open: bool,
}

impl BuildToolsPanel {
    pub fn new() -> Self {
        Self {
            window_open: true,
        }
    }

    pub fn render(&mut self, ui: &Ui, build_system: &EngineerBuildSystem) -> Vec<UIAction> {
        let mut actions = Vec::new();

        ui.window("🔧 Build Tools")
            .size([280.0, 350.0], Condition::FirstUseEver)
            .position([360.0, 80.0], Condition::FirstUseEver)
            .opened(&mut self.window_open)
            .build(|| {
                ui.text("Build Mode");
                ui.separator();

                let current_mode = build_system.get_current_mode();

                // Build mode buttons
                let modes = [
                    (BuildMode::Single, "🔨", "Single Block"),
                    (BuildMode::Wall, "🧱", "Wall (5x3)"),
                    (BuildMode::Floor, "📦", "Floor (5x5)"),
                    (BuildMode::Roof, "🏠", "Roof"),
                    (BuildMode::Template, "🏗️", "Template"),
                ];

                for (mode, icon, description) in modes.iter() {
                    let is_selected = current_mode == *mode;

                    let _style = if is_selected {
                        Some(ui.push_style_color(StyleColor::Button, [0.3, 0.7, 0.3, 1.0]))
                    } else {
                        None
                    };

                    if ui.button_with_size(
                        &format!("{} {}", icon, description),
                        [240.0, 30.0]
                    ) {
                        actions.push(UIAction::SetBuildMode(*mode));
                    }
                }

                ui.spacing();
                ui.separator();
                ui.text("Options");

                // Grid snap toggle
                let grid_snap = build_system.is_grid_snap_enabled();
                let mut grid_snap_mut = grid_snap;
                if ui.checkbox("Grid Snap", &mut grid_snap_mut) {
                    if grid_snap_mut != grid_snap {
                        actions.push(UIAction::ToggleGridSnap);
                    }
                }

                ui.spacing();
                ui.separator();
                ui.text("Actions");

                // Action buttons
                if ui.button_with_size("⬅️ Undo (Z)", [115.0, 25.0]) {
                    actions.push(UIAction::Undo);
                }
                ui.same_line();
                if ui.button_with_size("➡️ Redo (Y)", [115.0, 25.0]) {
                    actions.push(UIAction::Redo);
                }

                ui.spacing();

                if ui.button_with_size("💾 Save World", [240.0, 25.0]) {
                    actions.push(UIAction::SaveWorld);
                }

                if ui.button_with_size("📁 Load World", [240.0, 25.0]) {
                    actions.push(UIAction::LoadWorld);
                }

                if ui.button_with_size("🗑️ Clear World", [240.0, 25.0]) {
                    actions.push(UIAction::ClearWorld);
                }

                ui.spacing();
                ui.separator();
                ui.text("Status");

                match current_mode {
                    BuildMode::Single => {
                        ui.text("Single block placement mode");
                        ui.text("Left click: Remove | Right click: Place");
                    }
                    BuildMode::Wall => {
                        ui.text("Wall construction mode (5x3)");
                        ui.text("Right click to build wall segment");
                    }
                    BuildMode::Floor => {
                        ui.text("Floor construction mode (5x5)");
                        ui.text("Right click to build floor area");
                    }
                    BuildMode::Roof => {
                        ui.text("Roof construction mode");
                        ui.text("Right click to build pyramid roof");
                    }
                    BuildMode::Template => {
                        ui.text("Template placement mode");
                        ui.text("Use 'T' to cycle templates");
                    }
                    // Enhanced build modes
                    BuildMode::Circle => {
                        ui.text("Circle construction mode");
                        ui.text("Right click to place circular structure");
                    }
                    BuildMode::Sphere => {
                        ui.text("Sphere construction mode");
                        ui.text("Right click to place spherical structure");
                    }
                    BuildMode::Terrain => {
                        ui.text("Terrain sculpting mode");
                        ui.text("Right click to sculpt terrain");
                    }
                    BuildMode::Copy => {
                        ui.text("Copy structure mode");
                        ui.text("Right click to copy structures");
                    }
                    BuildMode::Paste => {
                        ui.text("Paste structure mode");
                        ui.text("Right click to paste copied structures");
                    }
                }

                if grid_snap {
                    ui.text_colored([0.0, 1.0, 0.0, 1.0], "Grid Snap: ON");
                } else {
                    ui.text_colored([1.0, 0.0, 0.0, 1.0], "Grid Snap: OFF");
                }
            });

        actions
    }

    pub fn is_open(&self) -> bool {
        self.window_open
    }

    pub fn set_open(&mut self, open: bool) {
        self.window_open = open;
    }
}