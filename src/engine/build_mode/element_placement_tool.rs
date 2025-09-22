use cgmath::{Vector3, Quaternion, InnerSpace, Zero, One, Rotation3, Rad};
use crate::engine::{
    math::{Vec3, Vec2},
    input::InputManager,
    error::RobinResult,
};
use winit::event::MouseButton;
use std::collections::HashMap;

use super::{
    interactive_elements::{InteractiveElementsSystem, ElementTemplate, InteractiveElement, ConnectionType},
    tools::{BuildTool, ToolKit},
    GridSystem, SelectionManager,
};

pub struct ElementPlacementTool {
    elements_system: InteractiveElementsSystem,
    selected_template: Option<String>,
    placement_preview: Option<PlacementPreview>,
    connection_mode: bool,
    connection_start: Option<u32>,
    element_browser: ElementBrowser,
    placement_settings: PlacementSettings,
}

#[derive(Debug, Clone)]
pub struct PlacementPreview {
    template_name: String,
    position: Vector3<f32>,
    rotation: Quaternion<f32>,
    scale: Vector3<f32>,
    valid_placement: bool,
}

#[derive(Debug, Clone)]
pub struct PlacementSettings {
    auto_connect: bool,
    connect_range: f32,
    surface_snap: bool,
    collision_check: bool,
    random_rotation: bool,
    scale_variance: f32,
}

pub struct ElementBrowser {
    categories: HashMap<String, Vec<String>>,
    current_category: String,
    current_selection: usize,
    visible: bool,
    search_filter: String,
}

impl ElementPlacementTool {
    pub fn new() -> Self {
        let mut tool = Self {
            elements_system: InteractiveElementsSystem::new(),
            selected_template: None,
            placement_preview: None,
            connection_mode: false,
            connection_start: None,
            element_browser: ElementBrowser::new(),
            placement_settings: PlacementSettings::default(),
        };

        // Set default template
        tool.selected_template = Some("basic_door".to_string());
        tool
    }

    pub fn update(&mut self, delta_time: f32, input: &InputManager, grid: &GridSystem, selection: &SelectionManager) -> RobinResult<()> {
        // Update the elements system
        let player_position = Vector3::new(0.0, 0.0, 0.0); // TODO: Get actual player position
        self.elements_system.update(delta_time, player_position);

        // Update element browser
        self.element_browser.update(input);

        // Handle tool input
        self.handle_input(input, grid, selection)?;

        // Update placement preview
        self.update_placement_preview(input, grid)?;

        Ok(())
    }

    fn handle_input(&mut self, input: &InputManager, grid: &GridSystem, selection: &SelectionManager) -> RobinResult<()> {
        // Toggle element browser
        if input.is_named_key_just_pressed(winit::keyboard::NamedKey::Tab) {
            self.element_browser.toggle();
        }

        // Toggle connection mode
        if input.is_key_just_pressed(&winit::keyboard::Key::Character("c".into())) {
            self.connection_mode = !self.connection_mode;
            if !self.connection_mode {
                self.connection_start = None;
            }
            println!("Connection mode: {}", self.connection_mode);
        }

        // Place element on left click
        if input.is_mouse_button_just_pressed(MouseButton::Left) {
            if self.connection_mode {
                self.handle_connection_click(input)?;
            } else {
                self.place_element(grid)?;
            }
        }

        // Delete element on right click
        if input.is_mouse_button_just_pressed(MouseButton::Right) {
            self.delete_element_at_cursor(input)?;
        }

        // Rotate preview with scroll
        if input.scroll_delta() != 0.0 {
            if let Some(preview) = &mut self.placement_preview {
                let rotation_delta = input.scroll_delta() * 0.1;
                let rotation = Quaternion::from_axis_angle(Vector3::unit_y(), Rad(rotation_delta));
                preview.rotation = preview.rotation * rotation;
            }
        }

        // Cycle through templates with number keys
        for i in 1..=9 {
            let key = winit::keyboard::Key::Character(i.to_string().into());
            if input.is_key_just_pressed(&key) {
                self.select_template_by_index(i - 1);
            }
        }

        Ok(())
    }

    fn update_placement_preview(&mut self, input: &InputManager, grid: &GridSystem) -> RobinResult<()> {
        if let Some(template_name) = &self.selected_template {
            // Calculate world position from mouse (this is simplified - in real implementation
            // you'd use camera ray casting)
            let mouse_pos = input.mouse_position();
            let world_pos = Vector3::new(
                (mouse_pos.0 as f32 - 400.0) * 0.01,
                0.0,
                (mouse_pos.1 as f32 - 300.0) * 0.01,
            );

            // Snap to grid if enabled
            let snapped_pos = if grid.enabled {
                Vector3::new(
                    grid.snap_position(Vec3::new(world_pos.x, world_pos.y, world_pos.z)).x,
                    grid.snap_position(Vec3::new(world_pos.x, world_pos.y, world_pos.z)).y,
                    grid.snap_position(Vec3::new(world_pos.x, world_pos.y, world_pos.z)).z,
                )
            } else {
                world_pos
            };

            // Check if placement is valid
            let valid_placement = self.check_placement_validity(&snapped_pos);

            self.placement_preview = Some(PlacementPreview {
                template_name: template_name.clone(),
                position: snapped_pos,
                rotation: Quaternion::one(),
                scale: Vector3::new(1.0, 1.0, 1.0),
                valid_placement,
            });
        }

        Ok(())
    }

    fn place_element(&mut self, _grid: &GridSystem) -> RobinResult<()> {
        if let Some(preview) = self.placement_preview.clone() {
            if preview.valid_placement {
                let element_id = self.elements_system.add_element(
                    &preview.template_name,
                    preview.position,
                )?;

                // Apply preview transformations
                if let Some(element) = self.elements_system.get_element_mut(element_id) {
                    element.rotation = preview.rotation;
                    element.scale = preview.scale;
                }

                // Auto-connect to nearby elements if enabled
                if self.placement_settings.auto_connect {
                    self.auto_connect_element(element_id)?;
                }

                println!("Placed {} at {:?}", preview.template_name, preview.position);
            }
        }

        Ok(())
    }

    fn delete_element_at_cursor(&mut self, input: &InputManager) -> RobinResult<()> {
        // Find element at cursor position (simplified implementation)
        let mouse_pos = input.mouse_position();
        let world_pos = Vector3::new(
            (mouse_pos.0 as f32 - 400.0) * 0.01,
            0.0,
            (mouse_pos.1 as f32 - 300.0) * 0.01,
        );

        // Find closest element within range
        let mut closest_element = None;
        let mut closest_distance = f32::INFINITY;

        for (id, element) in self.elements_system.get_elements() {
            let distance = (element.position - world_pos).magnitude();
            if distance < 1.0 && distance < closest_distance {
                closest_distance = distance;
                closest_element = Some(*id);
            }
        }

        if let Some(element_id) = closest_element {
            self.elements_system.remove_element(element_id)?;
            println!("Deleted element {}", element_id);
        }

        Ok(())
    }

    fn handle_connection_click(&mut self, input: &InputManager) -> RobinResult<()> {
        // Find element at cursor
        let mouse_pos = input.mouse_position();
        let world_pos = Vector3::new(
            (mouse_pos.0 as f32 - 400.0) * 0.01,
            0.0,
            (mouse_pos.1 as f32 - 300.0) * 0.01,
        );

        let mut clicked_element = None;
        for (id, element) in self.elements_system.get_elements() {
            let distance = (element.position - world_pos).magnitude();
            if distance < 1.0 {
                clicked_element = Some(*id);
                break;
            }
        }

        if let Some(element_id) = clicked_element {
            if let Some(start_id) = self.connection_start {
                // Create connection
                self.elements_system.connect_elements(
                    start_id,
                    element_id,
                    ConnectionType::Activate,
                )?;
                println!("Connected element {} to {}", start_id, element_id);
                self.connection_start = None;
            } else {
                // Start connection
                self.connection_start = Some(element_id);
                println!("Starting connection from element {}", element_id);
            }
        }

        Ok(())
    }

    fn auto_connect_element(&mut self, new_element_id: u32) -> RobinResult<()> {
        // First, collect all compatible elements to avoid borrow conflicts
        let compatible_elements: Vec<u32> = {
            if let Some(new_element) = self.elements_system.get_elements().get(&new_element_id) {
                let new_pos = new_element.position;
                let new_element_type = new_element.element_type.clone();

                self.elements_system.get_elements()
                    .iter()
                    .filter_map(|(other_id, other_element)| {
                        if *other_id != new_element_id {
                            let distance = (other_element.position - new_pos).magnitude();
                            if distance <= self.placement_settings.connect_range &&
                               self.are_elements_compatible(&new_element_type, &other_element.element_type) {
                                Some(*other_id)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect()
            } else {
                Vec::new()
            }
        };

        // Now connect the elements
        for other_id in compatible_elements {
            self.elements_system.connect_elements(
                other_id,
                new_element_id,
                ConnectionType::Activate,
            )?;
            println!("Auto-connected {} to {}", other_id, new_element_id);
        }

        Ok(())
    }

    fn are_elements_compatible(&self, type1: &super::interactive_elements::ElementType, type2: &super::interactive_elements::ElementType) -> bool {
        use super::interactive_elements::ElementType;

        match (type1, type2) {
            // Switches can activate doors, platforms, etc.
            (ElementType::Switch { .. }, ElementType::Door { .. }) => true,
            (ElementType::Switch { .. }, ElementType::Platform { .. }) => true,
            (ElementType::Switch { .. }, ElementType::Teleporter { .. }) => true,

            // Triggers can activate switches
            (ElementType::Trigger { .. }, ElementType::Switch { .. }) => true,
            (ElementType::Trigger { .. }, ElementType::Door { .. }) => true,

            // Default to false for safety
            _ => false,
        }
    }

    fn check_placement_validity(&self, position: &Vector3<f32>) -> bool {
        if !self.placement_settings.collision_check {
            return true;
        }

        // Check for overlapping elements
        for element in self.elements_system.get_elements().values() {
            let distance = (element.position - *position).magnitude();
            if distance < 0.5 {
                return false;
            }
        }

        true
    }

    fn select_template_by_index(&mut self, index: usize) {
        let templates: Vec<_> = self.elements_system.get_templates().keys().cloned().collect();
        if index < templates.len() {
            self.selected_template = Some(templates[index].clone());
            println!("Selected template: {}", templates[index]);
        }
    }

    pub fn get_selected_template(&self) -> Option<&String> {
        self.selected_template.as_ref()
    }

    pub fn set_selected_template(&mut self, template_name: String) {
        if self.elements_system.get_templates().contains_key(&template_name) {
            self.selected_template = Some(template_name);
        }
    }

    pub fn get_placement_preview(&self) -> Option<&PlacementPreview> {
        self.placement_preview.as_ref()
    }

    pub fn get_elements_system(&self) -> &InteractiveElementsSystem {
        &self.elements_system
    }

    pub fn get_elements_system_mut(&mut self) -> &mut InteractiveElementsSystem {
        &mut self.elements_system
    }

    pub fn is_connection_mode(&self) -> bool {
        self.connection_mode
    }

    pub fn get_connection_start(&self) -> Option<u32> {
        self.connection_start
    }
}

impl ElementBrowser {
    pub fn new() -> Self {
        let mut categories = HashMap::new();
        categories.insert("Interactive".to_string(), vec![
            "basic_door".to_string(),
            "moving_platform".to_string(),
            "teleporter".to_string(),
        ]);
        categories.insert("Triggers".to_string(), vec![
            "pressure_plate".to_string(),
        ]);
        categories.insert("Collectibles".to_string(), vec![
            "coin".to_string(),
        ]);
        categories.insert("Hazards".to_string(), vec![
            "spikes".to_string(),
        ]);

        Self {
            categories,
            current_category: "Interactive".to_string(),
            current_selection: 0,
            visible: false,
            search_filter: String::new(),
        }
    }

    pub fn update(&mut self, input: &InputManager) {
        if !self.visible {
            return;
        }

        // Navigate categories with arrow keys
        if input.is_named_key_just_pressed(winit::keyboard::NamedKey::ArrowUp) {
            if self.current_selection > 0 {
                self.current_selection -= 1;
            }
        }
        if input.is_named_key_just_pressed(winit::keyboard::NamedKey::ArrowDown) {
            if let Some(items) = self.categories.get(&self.current_category) {
                if self.current_selection < items.len() - 1 {
                    self.current_selection += 1;
                }
            }
        }

        // Switch categories with left/right arrows
        if input.is_named_key_just_pressed(winit::keyboard::NamedKey::ArrowLeft) {
            self.previous_category();
        }
        if input.is_named_key_just_pressed(winit::keyboard::NamedKey::ArrowRight) {
            self.next_category();
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn get_current_selection(&self) -> Option<String> {
        self.categories
            .get(&self.current_category)?
            .get(self.current_selection)
            .cloned()
    }

    pub fn get_current_category(&self) -> &str {
        &self.current_category
    }

    pub fn get_categories(&self) -> Vec<&String> {
        self.categories.keys().collect()
    }

    fn previous_category(&mut self) {
        let categories: Vec<_> = self.categories.keys().cloned().collect();
        if let Some(current_index) = categories.iter().position(|c| c == &self.current_category) {
            let new_index = if current_index == 0 {
                categories.len() - 1
            } else {
                current_index - 1
            };
            self.current_category = categories[new_index].clone();
            self.current_selection = 0;
        }
    }

    fn next_category(&mut self) {
        let categories: Vec<_> = self.categories.keys().cloned().collect();
        if let Some(current_index) = categories.iter().position(|c| c == &self.current_category) {
            let new_index = (current_index + 1) % categories.len();
            self.current_category = categories[new_index].clone();
            self.current_selection = 0;
        }
    }
}

impl PlacementSettings {
    pub fn default() -> Self {
        Self {
            auto_connect: true,
            connect_range: 3.0,
            surface_snap: true,
            collision_check: true,
            random_rotation: false,
            scale_variance: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_placement_tool_creation() {
        let tool = ElementPlacementTool::new();
        assert!(tool.get_selected_template().is_some());
        assert!(!tool.is_connection_mode());
    }

    #[test]
    fn test_element_browser() {
        let mut browser = ElementBrowser::new();
        assert!(!browser.is_visible());

        browser.toggle();
        assert!(browser.is_visible());

        assert!(browser.get_current_selection().is_some());
    }

    #[test]
    fn test_placement_validity() {
        let tool = ElementPlacementTool::new();
        let position = Vector3::new(0.0, 0.0, 0.0);
        assert!(tool.check_placement_validity(&position));
    }
}