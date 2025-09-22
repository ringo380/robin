// Inventory panel for material management

use imgui::*;
use crate::game::{EngineerBuildSystem, VoxelType};
use crate::ui::UIAction;

pub struct InventoryPanel {
    window_open: bool,
}

impl InventoryPanel {
    pub fn new() -> Self {
        Self {
            window_open: true,
        }
    }

    pub fn render(&mut self, ui: &Ui, build_system: &EngineerBuildSystem) -> Vec<UIAction> {
        let mut actions = Vec::new();

        ui.window("📦 Inventory")
            .size([320.0, 400.0], Condition::FirstUseEver)
            .position([20.0, 80.0], Condition::FirstUseEver)
            .opened(&mut self.window_open)
            .build(|| {
                ui.text("Materials");
                ui.separator();

                let current_material = build_system.get_current_material();
                let inventory = build_system.get_inventory();

                // Material grid
                let materials = [
                    (VoxelType::Stone, "🪨", "Stone"),
                    (VoxelType::Dirt, "🟫", "Dirt"),
                    (VoxelType::Grass, "🟩", "Grass"),
                    (VoxelType::Sand, "🟨", "Sand"),
                    (VoxelType::Water, "🟦", "Water"),
                    (VoxelType::Wood, "🟤", "Wood"),
                    (VoxelType::Crystal, "💎", "Crystal"),
                    (VoxelType::Lava, "🔥", "Lava"),
                ];

                let button_size = [60.0, 60.0];
                let mut column = 0;

                for (material, icon, name) in materials.iter() {
                    if column > 0 {
                        ui.same_line();
                    }

                    let count = inventory.get(material).unwrap_or(&0);
                    let is_selected = current_material == *material;

                    // Button styling for selection
                    let _style = if is_selected {
                        Some(ui.push_style_color(StyleColor::Button, [0.3, 0.7, 0.3, 1.0]))
                    } else {
                        None
                    };

                    let button_text = format!("{}\n{}", icon, count);
                    if ui.button_with_size(&button_text, button_size) {
                        actions.push(UIAction::SelectMaterial(*material));
                    }

                    if ui.is_item_hovered() {
                        ui.tooltip(|| {
                            ui.text(format!("{}: {} blocks", name, count));
                        });
                    }

                    column = (column + 1) % 4;
                    if column == 0 {
                        // Start new row
                    }
                }

                ui.spacing();
                ui.separator();
                ui.text("Selected Material:");
                ui.same_line();

                let selected_info = materials.iter()
                    .find(|(mat, _, _)| *mat == current_material)
                    .unwrap_or(&(VoxelType::Stone, "🪨", "Stone"));

                ui.text_colored([1.0, 1.0, 0.0, 1.0], format!("{} {}", selected_info.1, selected_info.2));

                if let Some(count) = inventory.get(&current_material) {
                    ui.text(format!("Available: {}", count));
                } else {
                    ui.text("Available: 0");
                }

                ui.spacing();
                ui.text("Quick Selection:");
                ui.text("Press 1-8 to select materials");
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