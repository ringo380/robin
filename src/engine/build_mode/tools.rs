/*!
 * Build Mode Tools - The Engineer's Arsenal
 *
 * This module implements the construction tools that make game creation
 * feel like playing an FPS. Each tool has intuitive controls and provides
 * immediate visual feedback.
 */

use crate::engine::{
    math::Vec3,
    graphics::Color,
    input::InputManager,
    error::RobinResult,
};
use cgmath::{InnerSpace, Zero, One};
use super::{GridSystem, SelectionManager};
use winit::keyboard::{Key, KeyCode, NamedKey};

// Helper function to convert KeyCode to Key
fn keycode_to_key(keycode: KeyCode) -> Key {
    match keycode {
        KeyCode::KeyQ => Key::Character("q".into()),
        KeyCode::KeyW => Key::Character("w".into()),
        KeyCode::KeyE => Key::Character("e".into()),
        KeyCode::KeyR => Key::Character("r".into()),
        KeyCode::KeyT => Key::Character("t".into()),
        KeyCode::KeyY => Key::Character("y".into()),
        KeyCode::KeyU => Key::Character("u".into()),
        KeyCode::KeyX => Key::Character("x".into()),
        KeyCode::KeyZ => Key::Character("z".into()),
        KeyCode::KeyC => Key::Character("c".into()),
        KeyCode::KeyV => Key::Character("v".into()),
        KeyCode::KeyD => Key::Character("d".into()),
        KeyCode::KeyF => Key::Character("f".into()),
        KeyCode::Digit1 => Key::Character("1".into()),
        KeyCode::Digit2 => Key::Character("2".into()),
        KeyCode::Digit3 => Key::Character("3".into()),
        KeyCode::Digit4 => Key::Character("4".into()),
        KeyCode::Digit5 => Key::Character("5".into()),
        KeyCode::BracketLeft => Key::Character("[".into()),
        KeyCode::BracketRight => Key::Character("]".into()),
        KeyCode::Escape => Key::Named(NamedKey::Escape),
        KeyCode::Space => Key::Named(NamedKey::Space),
        KeyCode::Tab => Key::Named(NamedKey::Tab),
        KeyCode::ControlLeft => Key::Named(NamedKey::Control),
        KeyCode::ShiftLeft => Key::Named(NamedKey::Shift),
        KeyCode::AltLeft => Key::Named(NamedKey::Alt),
        _ => Key::Character("unknown".into()),
    }
}

/// The engineer's toolkit containing all available tools
#[derive(Debug)]
pub struct ToolKit {
    tools: Vec<BuildTool>,
    matter_manipulator: MatterManipulator,
    transform_tool: TransformTool,
    clone_tool: CloneTool,
    terrain_sculptor: TerrainSculptor,
    wire_tool: WireTool,
}

impl ToolKit {
    pub fn new() -> Self {
        let mut toolkit = Self {
            tools: Vec::new(),
            matter_manipulator: MatterManipulator::new(),
            transform_tool: TransformTool::new(),
            clone_tool: CloneTool::new(),
            terrain_sculptor: TerrainSculptor::new(),
            wire_tool: WireTool::new(),
        };

        // Initialize available tools
        toolkit.tools = vec![
            BuildTool::MatterManipulator,
            BuildTool::TransformTool,
            BuildTool::CloneTool,
            BuildTool::TerrainSculptor,
            BuildTool::WireTool,
        ];

        toolkit
    }

    pub fn get_tool_by_index(&self, index: usize) -> Option<BuildTool> {
        self.tools.get(index).cloned()
    }

    pub fn update_tool(
        &mut self,
        tool: &BuildTool,
        delta_time: f32,
        input: &InputManager,
        grid: &mut GridSystem,
        selection: &mut SelectionManager,
    ) -> RobinResult<()> {
        match tool {
            BuildTool::MatterManipulator => {
                self.matter_manipulator.update(delta_time, input, grid)?;
            }
            BuildTool::TransformTool => {
                self.transform_tool.update(delta_time, input, grid, selection)?;
            }
            BuildTool::CloneTool => {
                self.clone_tool.update(delta_time, input, grid, selection)?;
            }
            BuildTool::TerrainSculptor => {
                self.terrain_sculptor.update(delta_time, input, grid)?;
            }
            BuildTool::WireTool => {
                self.wire_tool.update(delta_time, input)?;
            }
        }
        Ok(())
    }

    pub fn get_matter_manipulator(&self) -> &MatterManipulator {
        &self.matter_manipulator
    }

    pub fn get_transform_tool(&self) -> &TransformTool {
        &self.transform_tool
    }
}

/// Available build tools
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BuildTool {
    MatterManipulator,
    TransformTool,
    CloneTool,
    TerrainSculptor,
    WireTool,
}

/// Matter Manipulator - Primary voxel placement tool
#[derive(Debug)]
pub struct MatterManipulator {
    /// Current selected material
    pub selected_material: u32,

    /// Available materials in the material wheel
    pub materials: Vec<VoxelMaterial>,

    /// Current placement mode
    pub placement_mode: PlacementMode,

    /// Brush size for area operations
    pub brush_size: f32,

    /// Preview of what will be placed
    pub preview: Option<VoxelPreview>,

    /// Whether we're in removal mode
    pub removal_mode: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlacementMode {
    /// Single voxel placement
    Single,
    /// Line placement (hold Shift)
    Line,
    /// Plane placement (hold Ctrl)
    Plane,
    /// Volume placement (hold Alt)
    Volume,
    /// Sphere placement
    Sphere,
}

#[derive(Debug, Clone)]
pub struct VoxelMaterial {
    pub id: u32,
    pub name: String,
    pub color: Color,
    pub texture_id: Option<String>,
    pub properties: MaterialProperties,
}

#[derive(Debug, Clone)]
pub struct MaterialProperties {
    pub solid: bool,
    pub breakable: bool,
    pub bouncy: f32,
    pub friction: f32,
    pub emit_light: bool,
    pub light_color: Color,
}

#[derive(Debug, Clone)]
pub struct VoxelPreview {
    pub positions: Vec<Vec3>,
    pub material_id: u32,
    pub valid: bool,
}

impl MatterManipulator {
    pub fn new() -> Self {
        let materials = vec![
            VoxelMaterial {
                id: 0,
                name: "Stone".to_string(),
                color: Color::new(0.5, 0.5, 0.5, 1.0),
                texture_id: None,
                properties: MaterialProperties {
                    solid: true,
                    breakable: true,
                    bouncy: 0.1,
                    friction: 0.8,
                    emit_light: false,
                    light_color: Color::new(0.0, 0.0, 0.0, 0.0),
                },
            },
            VoxelMaterial {
                id: 1,
                name: "Wood".to_string(),
                color: Color::new(0.6, 0.4, 0.2, 1.0),
                texture_id: None,
                properties: MaterialProperties {
                    solid: true,
                    breakable: true,
                    bouncy: 0.2,
                    friction: 0.6,
                    emit_light: false,
                    light_color: Color::new(0.0, 0.0, 0.0, 0.0),
                },
            },
            VoxelMaterial {
                id: 2,
                name: "Metal".to_string(),
                color: Color::new(0.8, 0.8, 0.9, 1.0),
                texture_id: None,
                properties: MaterialProperties {
                    solid: true,
                    breakable: false,
                    bouncy: 0.05,
                    friction: 0.3,
                    emit_light: false,
                    light_color: Color::new(0.0, 0.0, 0.0, 0.0),
                },
            },
            VoxelMaterial {
                id: 3,
                name: "Glass".to_string(),
                color: Color::new(0.8, 0.9, 1.0, 0.3),
                texture_id: None,
                properties: MaterialProperties {
                    solid: true,
                    breakable: true,
                    bouncy: 0.1,
                    friction: 0.1,
                    emit_light: false,
                    light_color: Color::new(0.0, 0.0, 0.0, 0.0),
                },
            },
            VoxelMaterial {
                id: 4,
                name: "Light".to_string(),
                color: Color::new(1.0, 1.0, 0.8, 1.0),
                texture_id: None,
                properties: MaterialProperties {
                    solid: true,
                    breakable: true,
                    bouncy: 0.1,
                    friction: 0.5,
                    emit_light: true,
                    light_color: Color::new(1.0, 1.0, 0.8, 1.0),
                },
            },
        ];

        Self {
            selected_material: 0,
            materials,
            placement_mode: PlacementMode::Single,
            brush_size: 1.0,
            preview: None,
            removal_mode: false,
        }
    }

    pub fn update(
        &mut self,
        _delta_time: f32,
        input: &InputManager,
        grid: &GridSystem,
    ) -> RobinResult<()> {
        // Handle material selection with mouse wheel
        let scroll_delta = input.scroll_delta();
        if scroll_delta != 0.0 {
            if scroll_delta > 0.0 {
                self.selected_material = (self.selected_material + 1) % self.materials.len() as u32;
            } else if self.selected_material > 0 {
                self.selected_material -= 1;
            } else {
                self.selected_material = (self.materials.len() - 1) as u32;
            }
            log::debug!("Selected material: {}", self.get_current_material().name);
        }

        // Handle placement mode changes
        self.placement_mode = if input.is_named_key_pressed(winit::keyboard::NamedKey::Shift) {
            PlacementMode::Line
        } else if input.is_named_key_pressed(winit::keyboard::NamedKey::Control) {
            PlacementMode::Plane
        } else if input.is_named_key_pressed(winit::keyboard::NamedKey::Alt) {
            PlacementMode::Volume
        } else {
            PlacementMode::Single
        };

        // Handle removal mode toggle
        self.removal_mode = input.is_mouse_button_pressed(winit::event::MouseButton::Right);

        // Handle brush size adjustment
        if input.is_named_key_pressed(winit::keyboard::NamedKey::Control) {
            if input.is_key_just_pressed(&winit::keyboard::Key::Character("=".into())) {
                self.brush_size = (self.brush_size + 0.5).min(10.0);
                log::debug!("Brush size: {}", self.brush_size);
            }
            if input.is_key_just_pressed(&winit::keyboard::Key::Character("-".into())) {
                self.brush_size = (self.brush_size - 0.5).max(0.5);
                log::debug!("Brush size: {}", self.brush_size);
            }
        }

        // Update preview
        self.update_preview(input, grid)?;

        // Handle placement/removal
        if input.is_mouse_button_just_pressed(winit::event::MouseButton::Left) {
            self.execute_placement(grid)?;
        }

        Ok(())
    }

    fn update_preview(&mut self, input: &InputManager, grid: &GridSystem) -> RobinResult<()> {
        // Cast ray from camera through mouse cursor to find target position
        let target_position = self.raycast_target_position(input, grid)?;
        let snapped_position = grid.snap_position(target_position);

        let positions = match self.placement_mode {
            PlacementMode::Single => vec![snapped_position],
            PlacementMode::Line => {
                // TODO: Calculate line from start to end position
                vec![snapped_position]
            }
            PlacementMode::Plane => {
                // TODO: Calculate plane positions
                self.calculate_plane_positions(snapped_position, self.brush_size as i32)
            }
            PlacementMode::Volume => {
                // TODO: Calculate volume positions
                self.calculate_volume_positions(snapped_position, self.brush_size as i32)
            }
            PlacementMode::Sphere => {
                // TODO: Calculate sphere positions
                self.calculate_sphere_positions(snapped_position, self.brush_size)
            }
        };

        self.preview = Some(VoxelPreview {
            positions,
            material_id: self.selected_material,
            valid: true, // TODO: Implement validity checking
        });

        Ok(())
    }

    fn execute_placement(&self, grid: &GridSystem) -> RobinResult<()> {
        if let Some(preview) = &self.preview {
            if preview.valid {
                let action = if self.removal_mode {
                    BuildActionType::RemoveVoxels(preview.positions.clone())
                } else {
                    BuildActionType::PlaceVoxels {
                        positions: preview.positions.clone(),
                        material_id: preview.material_id,
                    }
                };

                for position in &preview.positions {
                    if self.removal_mode {
                        log::debug!("Removing voxel at {:?}", position);
                        grid.remove_voxel_at(*position)?;
                    } else {
                        log::debug!("Placing {} voxel at {:?}",
                            self.get_current_material().name, position);
                        grid.place_voxel_at(*position, preview.material_id)?;
                    }
                }

                // Add action to build history for undo/redo
                // TODO: Pass action to build history system
            }
        }
        Ok(())
    }

    fn calculate_plane_positions(&self, center: Vec3, size: i32) -> Vec<Vec3> {
        let mut positions = Vec::new();
        let half_size = size / 2;

        for x in -half_size..=half_size {
            for z in -half_size..=half_size {
                positions.push(Vec3::new(
                    center.x + x as f32,
                    center.y,
                    center.z + z as f32,
                ));
            }
        }

        positions
    }

    fn calculate_volume_positions(&self, center: Vec3, size: i32) -> Vec<Vec3> {
        let mut positions = Vec::new();
        let half_size = size / 2;

        for x in -half_size..=half_size {
            for y in -half_size..=half_size {
                for z in -half_size..=half_size {
                    positions.push(Vec3::new(
                        center.x + x as f32,
                        center.y + y as f32,
                        center.z + z as f32,
                    ));
                }
            }
        }

        positions
    }

    fn calculate_sphere_positions(&self, center: Vec3, radius: f32) -> Vec<Vec3> {
        let mut positions = Vec::new();
        let max_dist = radius.ceil() as i32;

        for x in -max_dist..=max_dist {
            for y in -max_dist..=max_dist {
                for z in -max_dist..=max_dist {
                    let pos = Vec3::new(
                        center.x + x as f32,
                        center.y + y as f32,
                        center.z + z as f32,
                    );

                    let distance = (pos - center).magnitude();
                    if distance <= radius {
                        positions.push(pos);
                    }
                }
            }
        }

        positions
    }

    pub fn get_current_material(&self) -> &VoxelMaterial {
        &self.materials[self.selected_material as usize]
    }

    pub fn get_materials(&self) -> &[VoxelMaterial] {
        &self.materials
    }

    pub fn get_preview(&self) -> Option<&VoxelPreview> {
        self.preview.as_ref()
    }

    /// Cast a ray from camera through mouse cursor to find target position
    fn raycast_target_position(&self, input: &InputManager, grid: &GridSystem) -> RobinResult<Vec3> {
        // Get mouse position in normalized device coordinates
        let mouse_pos = input.mouse_position();
        let viewport_size = input.viewport_size();

        let ndc_x = (mouse_pos.0 / viewport_size.0) * 2.0 - 1.0;
        let ndc_y = 1.0 - (mouse_pos.1 / viewport_size.1) * 2.0;

        // TODO: Get actual camera matrices from build mode viewport
        // For now, use placeholder calculation
        let camera_position = Vec3::new(0.0, 10.0, 10.0);
        let camera_direction = Vec3::new(ndc_x as f32 * 0.1, -1.0, ndc_y as f32 * 0.1).normalize();

        // Cast ray into the world and find intersection with terrain or grid
        let ray_result = grid.raycast(camera_position, camera_direction, 100.0)?;

        Ok(ray_result.unwrap_or_else(|| {
            // If no intersection, place at a default distance from camera
            camera_position + camera_direction * 10.0
        }))
    }

    /// Check if a position is valid for placement (not colliding with existing objects)
    fn is_position_valid(&self, position: Vec3, grid: &GridSystem) -> bool {
        // Check if position is within world bounds
        if position.y < 0.0 || position.y > 256.0 {
            return false;
        }

        // Check if there's already a voxel at this position (for placement)
        if !self.removal_mode {
            if grid.has_voxel_at(position) {
                return false;
            }
        } else {
            // For removal, we need a voxel to exist
            if !grid.has_voxel_at(position) {
                return false;
            }
        }

        true
    }
}

/// Transform Tool - Move, rotate, and scale objects
#[derive(Debug)]
pub struct TransformTool {
    pub transform_mode: TransformMode,
    pub coordinate_space: CoordinateSpace,
    pub snap_increment: f32,
    pub active_axis: Option<TransformAxis>,
    pub gizmo_visible: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransformMode {
    Move,
    Rotate,
    Scale,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CoordinateSpace {
    World,
    Local,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransformAxis {
    X,
    Y,
    Z,
    XY,
    XZ,
    YZ,
    XYZ,
}

impl TransformTool {
    pub fn new() -> Self {
        Self {
            transform_mode: TransformMode::Move,
            coordinate_space: CoordinateSpace::World,
            snap_increment: 1.0,
            active_axis: None,
            gizmo_visible: true,
        }
    }

    pub fn update(
        &mut self,
        _delta_time: f32,
        input: &InputManager,
        _grid: &GridSystem,
        selection: &SelectionManager,
    ) -> RobinResult<()> {
        // Switch transform modes with Q, W, E
        if input.is_key_just_pressed(&keycode_to_key(KeyCode::KeyQ)) {
            self.transform_mode = TransformMode::Move;
            log::debug!("Transform mode: Move");
        }
        if input.is_key_just_pressed(&keycode_to_key(KeyCode::KeyW)) {
            self.transform_mode = TransformMode::Rotate;
            log::debug!("Transform mode: Rotate");
        }
        if input.is_key_just_pressed(&keycode_to_key(KeyCode::KeyE)) {
            self.transform_mode = TransformMode::Scale;
            log::debug!("Transform mode: Scale");
        }

        // Toggle coordinate space with R
        if input.is_key_just_pressed(&keycode_to_key(KeyCode::KeyR)) {
            self.coordinate_space = match self.coordinate_space {
                CoordinateSpace::World => CoordinateSpace::Local,
                CoordinateSpace::Local => CoordinateSpace::World,
            };
            log::debug!("Coordinate space: {:?}", self.coordinate_space);
        }

        // Handle axis constraints with X, Y, Z keys
        if input.is_key_just_pressed(&keycode_to_key(winit::keyboard::KeyCode::KeyX)) {
            self.active_axis = Some(TransformAxis::X);
        }
        if input.is_key_just_pressed(&keycode_to_key(winit::keyboard::KeyCode::KeyY)) {
            self.active_axis = Some(TransformAxis::Y);
        }
        if input.is_key_just_pressed(&keycode_to_key(winit::keyboard::KeyCode::KeyZ)) {
            self.active_axis = Some(TransformAxis::Z);
        }

        // Clear axis constraint with Escape
        if input.is_key_just_pressed(&keycode_to_key(winit::keyboard::KeyCode::Escape)) {
            self.active_axis = None;
        }

        // Apply transformations if objects are selected
        if !selection.get_selected_objects().is_empty() {
            self.apply_transformations(input, selection)?;
        }

        Ok(())
    }

    fn apply_transformations(
        &mut self,
        input: &InputManager,
        selection: &SelectionManager,
    ) -> RobinResult<()> {
        if input.is_mouse_button_pressed(winit::event::MouseButton::Left) {
            let mouse_delta = input.mouse_delta();
            if mouse_delta.0 != 0.0 || mouse_delta.1 != 0.0 {
                for &object_id in selection.get_selected_objects() {
                    self.transform_object(object_id, mouse_delta)?;
                }
            }
        }
        Ok(())
    }

    fn transform_object(&self, object_id: u32, mouse_delta: (f64, f64)) -> RobinResult<()> {
        let sensitivity = 0.01;
        let delta_x = mouse_delta.0 as f32 * sensitivity;
        let delta_y = mouse_delta.1 as f32 * sensitivity;

        match self.transform_mode {
            TransformMode::Move => {
                let movement = match self.active_axis {
                    Some(TransformAxis::X) => Vec3::new(delta_x, 0.0, 0.0),
                    Some(TransformAxis::Y) => Vec3::new(0.0, -delta_y, 0.0),
                    Some(TransformAxis::Z) => Vec3::new(0.0, 0.0, delta_y),
                    Some(TransformAxis::XY) => Vec3::new(delta_x, -delta_y, 0.0),
                    Some(TransformAxis::XZ) => Vec3::new(delta_x, 0.0, delta_y),
                    Some(TransformAxis::YZ) => Vec3::new(0.0, -delta_y, delta_y),
                    _ => Vec3::new(delta_x, 0.0, delta_y), // Default plane movement
                };

                // Snap to increment if snapping is enabled
                let snapped_movement = if self.snap_increment > 0.0 {
                    Vec3::new(
                        (movement.x / self.snap_increment).round() * self.snap_increment,
                        (movement.y / self.snap_increment).round() * self.snap_increment,
                        (movement.z / self.snap_increment).round() * self.snap_increment,
                    )
                } else {
                    movement
                };

                log::debug!("Moving object {} by {:?}", object_id, snapped_movement);
                // TODO: Apply actual movement to object in scene
            }
            TransformMode::Rotate => {
                let rotation_speed = 2.0;
                let rotation = match self.active_axis {
                    Some(TransformAxis::X) => Vec3::new(delta_y * rotation_speed, 0.0, 0.0),
                    Some(TransformAxis::Y) => Vec3::new(0.0, delta_x * rotation_speed, 0.0),
                    Some(TransformAxis::Z) => Vec3::new(0.0, 0.0, delta_x * rotation_speed),
                    _ => Vec3::new(0.0, delta_x * rotation_speed, 0.0), // Default Y-axis rotation
                };

                log::debug!("Rotating object {} by {:?} radians", object_id, rotation);
                // TODO: Apply actual rotation to object in scene
            }
            TransformMode::Scale => {
                let scale_factor = 1.0 + delta_y;
                let scale = match self.active_axis {
                    Some(TransformAxis::X) => Vec3::new(scale_factor, 1.0, 1.0),
                    Some(TransformAxis::Y) => Vec3::new(1.0, scale_factor, 1.0),
                    Some(TransformAxis::Z) => Vec3::new(1.0, 1.0, scale_factor),
                    _ => Vec3::new(scale_factor, scale_factor, scale_factor), // Uniform scaling
                };

                log::debug!("Scaling object {} by {:?}", object_id, scale);
                // TODO: Apply actual scaling to object in scene
            }
        }
        Ok(())
    }

    pub fn get_transform_mode(&self) -> TransformMode {
        self.transform_mode
    }

    pub fn get_active_axis(&self) -> Option<TransformAxis> {
        self.active_axis
    }
}

/// Clone Tool - Copy and paste objects with transformations
#[derive(Debug)]
pub struct CloneTool {
    pub clipboard: Vec<ClonedObject>,
    pub clone_mode: CloneMode,
    pub array_count: i32,
    pub array_offset: Vec3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CloneMode {
    Single,
    Array,
    Mirror,
    Instance,
}

#[derive(Debug, Clone)]
pub struct ClonedObject {
    pub object_type: String,
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
    pub properties: Vec<(String, String)>,
}

impl CloneTool {
    pub fn new() -> Self {
        Self {
            clipboard: Vec::new(),
            clone_mode: CloneMode::Single,
            array_count: 3,
            array_offset: Vec3::new(2.0, 0.0, 0.0),
        }
    }

    pub fn update(
        &mut self,
        _delta_time: f32,
        input: &InputManager,
        _grid: &GridSystem,
        selection: &SelectionManager,
    ) -> RobinResult<()> {
        // Copy with Ctrl+C
        if input.is_key_pressed(&keycode_to_key(winit::keyboard::KeyCode::ControlLeft)) &&
           input.is_key_just_pressed(&keycode_to_key(winit::keyboard::KeyCode::KeyC)) {
            self.copy_selection(selection)?;
        }

        // Paste with Ctrl+V
        if input.is_key_pressed(&keycode_to_key(winit::keyboard::KeyCode::ControlLeft)) &&
           input.is_key_just_pressed(&keycode_to_key(winit::keyboard::KeyCode::KeyV)) {
            self.paste_clipboard()?;
        }

        // Duplicate with Ctrl+D
        if input.is_key_pressed(&keycode_to_key(winit::keyboard::KeyCode::ControlLeft)) &&
           input.is_key_just_pressed(&keycode_to_key(winit::keyboard::KeyCode::KeyD)) {
            self.duplicate_selection(selection)?;
        }

        Ok(())
    }

    fn copy_selection(&mut self, selection: &SelectionManager) -> RobinResult<()> {
        self.clipboard.clear();

        for &object_id in selection.get_selected_objects() {
            // TODO: Get object data and add to clipboard
            log::debug!("Copied object {} to clipboard", object_id);

            // Placeholder object
            self.clipboard.push(ClonedObject {
                object_type: "voxel".to_string(),
                position: Vec3::new(0.0, 0.0, 0.0),
                rotation: Vec3::new(0.0, 0.0, 0.0),
                scale: Vec3::new(1.0, 1.0, 1.0),
                properties: Vec::new(),
            });
        }

        log::info!("Copied {} objects to clipboard", self.clipboard.len());
        Ok(())
    }

    fn paste_clipboard(&self) -> RobinResult<()> {
        for object in &self.clipboard {
            log::debug!("Pasting object: {:?}", object.object_type);
            // TODO: Create new object from clipboard data
        }

        log::info!("Pasted {} objects from clipboard", self.clipboard.len());
        Ok(())
    }

    fn duplicate_selection(&self, selection: &SelectionManager) -> RobinResult<()> {
        for &object_id in selection.get_selected_objects() {
            log::debug!("Duplicating object {}", object_id);
            // TODO: Duplicate object with slight offset
        }

        log::info!("Duplicated {} objects", selection.get_selected_objects().len());
        Ok(())
    }
}

/// Terrain Sculptor - Organic terrain modification
#[derive(Debug)]
pub struct TerrainSculptor {
    pub sculpt_mode: SculptMode,
    pub brush_size: f32,
    pub brush_strength: f32,
    pub brush_falloff: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SculptMode {
    Raise,
    Lower,
    Smooth,
    Flatten,
    Noise,
}

impl TerrainSculptor {
    pub fn new() -> Self {
        Self {
            sculpt_mode: SculptMode::Raise,
            brush_size: 5.0,
            brush_strength: 1.0,
            brush_falloff: 0.5,
        }
    }

    pub fn update(
        &mut self,
        _delta_time: f32,
        input: &InputManager,
        _grid: &GridSystem,
    ) -> RobinResult<()> {
        // Switch sculpt modes with number keys
        if input.is_key_just_pressed(&keycode_to_key(winit::keyboard::KeyCode::Digit1)) {
            self.sculpt_mode = SculptMode::Raise;
        }
        if input.is_key_just_pressed(&keycode_to_key(winit::keyboard::KeyCode::Digit2)) {
            self.sculpt_mode = SculptMode::Lower;
        }
        if input.is_key_just_pressed(&keycode_to_key(winit::keyboard::KeyCode::Digit3)) {
            self.sculpt_mode = SculptMode::Smooth;
        }
        if input.is_key_just_pressed(&keycode_to_key(winit::keyboard::KeyCode::Digit4)) {
            self.sculpt_mode = SculptMode::Flatten;
        }
        if input.is_key_just_pressed(&keycode_to_key(winit::keyboard::KeyCode::Digit5)) {
            self.sculpt_mode = SculptMode::Noise;
        }

        // Adjust brush size with [ and ]
        if input.is_key_just_pressed(&keycode_to_key(winit::keyboard::KeyCode::BracketLeft)) {
            self.brush_size = (self.brush_size - 1.0).max(1.0);
            log::debug!("Brush size: {}", self.brush_size);
        }
        if input.is_key_just_pressed(&keycode_to_key(winit::keyboard::KeyCode::BracketRight)) {
            self.brush_size = (self.brush_size + 1.0).min(20.0);
            log::debug!("Brush size: {}", self.brush_size);
        }

        // Apply sculpting
        if input.is_mouse_button_pressed(winit::event::MouseButton::Left) {
            self.apply_sculpting()?;
        }

        Ok(())
    }

    fn apply_sculpting(&self) -> RobinResult<()> {
        match self.sculpt_mode {
            SculptMode::Raise => {
                log::debug!("Raising terrain");
                // TODO: Implement terrain raising
            }
            SculptMode::Lower => {
                log::debug!("Lowering terrain");
                // TODO: Implement terrain lowering
            }
            SculptMode::Smooth => {
                log::debug!("Smoothing terrain");
                // TODO: Implement terrain smoothing
            }
            SculptMode::Flatten => {
                log::debug!("Flattening terrain");
                // TODO: Implement terrain flattening
            }
            SculptMode::Noise => {
                log::debug!("Adding noise to terrain");
                // TODO: Implement terrain noise
            }
        }
        Ok(())
    }
}

/// Wire Tool - Connect logic nodes with visual connections
#[derive(Debug)]
pub struct WireTool {
    pub connection_start: Option<u32>, // Node ID
    pub active_connection: Option<ActiveConnection>,
    pub wire_type: WireType,
}

#[derive(Debug, Clone)]
pub struct ActiveConnection {
    pub start_node: u32,
    pub start_position: Vec3,
    pub current_position: Vec3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WireType {
    Data,
    Signal,
    Power,
}

impl WireTool {
    pub fn new() -> Self {
        Self {
            connection_start: None,
            active_connection: None,
            wire_type: WireType::Data,
        }
    }

    pub fn update(&mut self, _delta_time: f32, input: &InputManager) -> RobinResult<()> {
        // Switch wire types with T, Y, U
        if input.is_key_just_pressed(&keycode_to_key(winit::keyboard::KeyCode::KeyT)) {
            self.wire_type = WireType::Data;
            log::debug!("Wire type: Data");
        }
        if input.is_key_just_pressed(&keycode_to_key(winit::keyboard::KeyCode::KeyY)) {
            self.wire_type = WireType::Signal;
            log::debug!("Wire type: Signal");
        }
        if input.is_key_just_pressed(&keycode_to_key(winit::keyboard::KeyCode::KeyU)) {
            self.wire_type = WireType::Power;
            log::debug!("Wire type: Power");
        }

        // Handle connection creation
        if input.is_mouse_button_just_pressed(winit::event::MouseButton::Left) {
            self.handle_connection_start()?;
        }

        // Cancel connection with right click
        if input.is_mouse_button_just_pressed(winit::event::MouseButton::Right) {
            self.cancel_connection();
        }

        Ok(())
    }

    fn handle_connection_start(&mut self) -> RobinResult<()> {
        if self.connection_start.is_none() {
            // TODO: Ray cast to find node under cursor
            let node_id = 0; // Placeholder
            self.connection_start = Some(node_id);
            log::debug!("Started connection from node {}", node_id);
        } else {
            // TODO: Complete connection to target node
            let target_node = 1; // Placeholder
            if let Some(start_node) = self.connection_start {
                log::debug!("Connected node {} to node {}", start_node, target_node);
                // TODO: Create actual connection
            }
            self.connection_start = None;
        }
        Ok(())
    }

    fn cancel_connection(&mut self) {
        if self.connection_start.is_some() {
            log::debug!("Cancelled connection");
            self.connection_start = None;
            self.active_connection = None;
        }
    }
}

impl Default for ToolKit {
    fn default() -> Self {
        Self::new()
    }
}

// Additional types and implementations for the build system

/// Represents different types of build actions for undo/redo
#[derive(Debug, Clone)]
pub enum BuildActionType {
    PlaceVoxels {
        positions: Vec<Vec3>,
        material_id: u32,
    },
    RemoveVoxels(Vec<Vec3>),
    TransformObjects {
        object_ids: Vec<u32>,
        old_transforms: Vec<Transform>,
        new_transforms: Vec<Transform>,
    },
    CreateObjects {
        object_data: Vec<ObjectData>,
    },
    DeleteObjects {
        object_ids: Vec<u32>,
        object_data: Vec<ObjectData>, // For restoration
    },
}

#[derive(Debug, Clone)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
}

#[derive(Debug, Clone)]
pub struct ObjectData {
    pub id: u32,
    pub object_type: String,
    pub transform: Transform,
    pub properties: std::collections::HashMap<String, String>,
}

/// Extended input manager methods for build mode
pub trait InputManagerExt {
    fn mouse_position(&self) -> (f64, f64);
    fn viewport_size(&self) -> (f64, f64);
    fn scroll_delta(&self) -> f32;
    fn mouse_delta(&self) -> (f64, f64);
}

/// Extended grid system methods for voxel manipulation
pub trait GridSystemExt {
    fn place_voxel_at(&self, position: Vec3, material_id: u32) -> RobinResult<()>;
    fn remove_voxel_at(&self, position: Vec3) -> RobinResult<()>;
    fn has_voxel_at(&self, position: Vec3) -> bool;
    fn raycast(&self, origin: Vec3, direction: Vec3, max_distance: f32) -> RobinResult<Option<Vec3>>;
}

// Implementation placeholders - these would be implemented on the actual types
impl InputManagerExt for InputManager {
    fn mouse_position(&self) -> (f64, f64) {
        // TODO: Get actual mouse position from winit
        (0.0, 0.0)
    }

    fn viewport_size(&self) -> (f64, f64) {
        // TODO: Get actual viewport size
        (1920.0, 1080.0)
    }

    fn scroll_delta(&self) -> f32 {
        // TODO: Get actual scroll delta from input events
        0.0
    }

    fn mouse_delta(&self) -> (f64, f64) {
        // TODO: Get mouse movement delta
        (0.0, 0.0)
    }
}

impl GridSystemExt for GridSystem {
    fn place_voxel_at(&self, position: Vec3, material_id: u32) -> RobinResult<()> {
        // TODO: Implement actual voxel placement
        log::debug!("Placing voxel at {:?} with material {}", position, material_id);
        Ok(())
    }

    fn remove_voxel_at(&self, position: Vec3) -> RobinResult<()> {
        // TODO: Implement actual voxel removal
        log::debug!("Removing voxel at {:?}", position);
        Ok(())
    }

    fn has_voxel_at(&self, position: Vec3) -> bool {
        // TODO: Check if voxel exists at position
        false
    }

    fn raycast(&self, origin: Vec3, direction: Vec3, max_distance: f32) -> RobinResult<Option<Vec3>> {
        // TODO: Implement actual raycasting
        // For now, return a point along the ray
        let hit_point = origin + direction * (max_distance * 0.5);
        Ok(Some(hit_point))
    }
}

/// Performance metrics for tool operations
#[derive(Debug)]
pub struct ToolPerformanceMetrics {
    pub last_update_time: std::time::Instant,
    pub average_frame_time: f32,
    pub peak_frame_time: f32,
    pub operations_per_second: u32,
}

impl ToolPerformanceMetrics {
    pub fn new() -> Self {
        Self {
            last_update_time: std::time::Instant::now(),
            average_frame_time: 0.0,
            peak_frame_time: 0.0,
            operations_per_second: 0,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        self.average_frame_time = self.average_frame_time * 0.95 + delta_time * 0.05;
        self.peak_frame_time = self.peak_frame_time.max(delta_time);
        self.last_update_time = std::time::Instant::now();
    }
}

/// Tool validation and safety checks
pub struct ToolValidator {
    max_operations_per_frame: u32,
    max_brush_size: f32,
    min_snap_increment: f32,
}

impl ToolValidator {
    pub fn new() -> Self {
        Self {
            max_operations_per_frame: 1000,
            max_brush_size: 50.0,
            min_snap_increment: 0.01,
        }
    }

    pub fn validate_brush_size(&self, size: f32) -> f32 {
        size.clamp(0.5, self.max_brush_size)
    }

    pub fn validate_snap_increment(&self, increment: f32) -> f32 {
        increment.max(self.min_snap_increment)
    }

    pub fn validate_operation_count(&self, count: u32) -> bool {
        count <= self.max_operations_per_frame
    }
}