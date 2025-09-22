use crate::engine::graphics::Mesh;
use crate::math::{Vec3, Mat4, Quat};
use cgmath::{SquareMatrix, One, Zero, perspective, Point3, Rad};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Editor {
    pub active_tool: EditorTool,
    pub gizmo: TransformGizmo,
    pub snapping: SnappingSettings,
    pub selection: SelectionSet,
    pub clipboard: EditorClipboard,
    pub viewport: EditorViewport,
    pub history: EditorHistory,
    pub preferences: EditorPreferences,
}

#[derive(Debug, Clone)]
pub enum EditorTool {
    Select,
    Move,
    Rotate,
    Scale,
    Extrude,
    Knife,
    Merge,
    Subdivide,
    Mirror,
    Array,
}

#[derive(Debug, Clone)]
pub struct TransformGizmo {
    pub visible: bool,
    pub mode: GizmoMode,
    pub space: TransformSpace,
    pub size: f32,
    pub highlight: Option<GizmoAxis>,
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

#[derive(Debug, Clone)]
pub enum GizmoMode {
    Translation,
    Rotation,
    Scale,
    Universal,
}

#[derive(Debug, Clone)]
pub enum TransformSpace {
    World,
    Local,
    View,
}

#[derive(Debug, Clone)]
pub enum GizmoAxis {
    X,
    Y,
    Z,
    XY,
    XZ,
    YZ,
    XYZ,
}

#[derive(Debug, Clone)]
pub struct SnappingSettings {
    pub enabled: bool,
    pub grid_snap: bool,
    pub vertex_snap: bool,
    pub edge_snap: bool,
    pub face_snap: bool,
    pub angle_snap: bool,
    pub grid_size: f32,
    pub angle_increment: f32,
}

#[derive(Debug, Clone)]
pub struct SelectionSet {
    pub objects: Vec<u32>,
    pub vertices: Vec<u32>,
    pub edges: Vec<u32>,
    pub faces: Vec<u32>,
    pub mode: SelectionMode,
    pub bounding_box: Option<BoundingBox>,
}

#[derive(Debug, Clone)]
pub enum SelectionMode {
    Object,
    Vertex,
    Edge,
    Face,
    Component,
}

#[derive(Debug, Clone)]
pub struct BoundingBox {
    pub min: Vec3,
    pub max: Vec3,
}

#[derive(Debug, Clone)]
pub struct EditorClipboard {
    pub objects: Vec<ClipboardObject>,
    pub transform_data: Vec<TransformData>,
    pub material_data: Vec<MaterialData>,
}

#[derive(Debug, Clone)]
pub struct ClipboardObject {
    pub id: u32,
    pub mesh_data: Vec<u8>,
    pub transform: Mat4,
    pub material_id: u32,
}

#[derive(Debug, Clone)]
pub struct TransformData {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

#[derive(Debug, Clone)]
pub struct MaterialData {
    pub id: u32,
    pub name: String,
    pub properties: HashMap<String, f32>,
}

#[derive(Debug, Clone)]
pub struct EditorViewport {
    pub camera_position: Vec3,
    pub camera_target: Vec3,
    pub field_of_view: f32,
    pub near_plane: f32,
    pub far_plane: f32,
    pub wireframe_mode: bool,
    pub shading_mode: ShadingMode,
    pub grid_visible: bool,
    pub axes_visible: bool,
    pub statistics_visible: bool,
}

#[derive(Debug, Clone)]
pub enum ShadingMode {
    Solid,
    Wireframe,
    Textured,
    Material,
}

#[derive(Debug, Clone)]
pub struct EditorHistory {
    pub actions: Vec<EditorAction>,
    pub current_index: usize,
    pub max_history: usize,
}

#[derive(Debug, Clone)]
pub struct EditorAction {
    pub name: String,
    pub timestamp: u64,
    pub undo_data: ActionData,
    pub redo_data: ActionData,
}

#[derive(Debug, Clone)]
pub enum ActionData {
    Transform(Vec<TransformData>),
    Geometry(Vec<GeometryData>),
    Material(Vec<MaterialData>),
    Delete(Vec<u32>),
    Create(Vec<ClipboardObject>),
}

#[derive(Debug, Clone)]
pub struct GeometryData {
    pub object_id: u32,
    pub vertices: Vec<Vec3>,
    pub indices: Vec<u32>,
    pub normals: Vec<Vec3>,
    pub uvs: Vec<[f32; 2]>,
}

#[derive(Debug, Clone)]
pub struct EditorPreferences {
    pub auto_save_interval: u32,
    pub backup_count: u32,
    pub grid_size: f32,
    pub camera_speed: f32,
    pub selection_outline_color: [f32; 4],
    pub gizmo_size: f32,
    pub snap_distance: f32,
    pub double_click_time: u32,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            active_tool: EditorTool::Select,
            gizmo: TransformGizmo::new(),
            snapping: SnappingSettings::default(),
            selection: SelectionSet::new(),
            clipboard: EditorClipboard::new(),
            viewport: EditorViewport::default(),
            history: EditorHistory::new(),
            preferences: EditorPreferences::default(),
        }
    }

    pub fn select_tool(&mut self, tool: EditorTool) {
        self.active_tool = tool;
        self.update_gizmo_mode();
    }

    pub fn update_gizmo_mode(&mut self) {
        self.gizmo.mode = match self.active_tool {
            EditorTool::Move => GizmoMode::Translation,
            EditorTool::Rotate => GizmoMode::Rotation,
            EditorTool::Scale => GizmoMode::Scale,
            _ => GizmoMode::Universal,
        };
    }

    pub fn select_objects(&mut self, object_ids: Vec<u32>) {
        self.selection.objects = object_ids;
        self.update_selection_bounds();
        self.update_gizmo_position();
    }

    pub fn add_to_selection(&mut self, object_id: u32) {
        if !self.selection.objects.contains(&object_id) {
            self.selection.objects.push(object_id);
            self.update_selection_bounds();
            self.update_gizmo_position();
        }
    }

    pub fn remove_from_selection(&mut self, object_id: u32) {
        self.selection.objects.retain(|&id| id != object_id);
        self.update_selection_bounds();
        self.update_gizmo_position();
    }

    pub fn clear_selection(&mut self) {
        self.selection.objects.clear();
        self.selection.bounding_box = None;
        self.gizmo.visible = false;
    }

    pub fn update_selection_bounds(&mut self) {
        if self.selection.objects.is_empty() {
            self.selection.bounding_box = None;
            return;
        }

        // Calculate bounding box for selected objects
        let mut min = Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
        let mut max = Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);

        // TODO: Get actual object bounds from scene
        for _object_id in &self.selection.objects {
            // Placeholder calculation
            min = Vec3::new(min.x.min(-1.0), min.y.min(-1.0), min.z.min(-1.0));
            max = Vec3::new(max.x.max(1.0), max.y.max(1.0), max.z.max(1.0));
        }

        self.selection.bounding_box = Some(BoundingBox { min, max });
    }

    pub fn update_gizmo_position(&mut self) {
        if let Some(bbox) = &self.selection.bounding_box {
            let center = Vec3::new(
                (bbox.min.x + bbox.max.x) * 0.5,
                (bbox.min.y + bbox.max.y) * 0.5,
                (bbox.min.z + bbox.max.z) * 0.5,
            );
            self.gizmo.position = center;
            self.gizmo.visible = true;
        } else {
            self.gizmo.visible = false;
        }
    }

    pub fn copy_selected(&mut self) {
        self.clipboard.objects.clear();
        self.clipboard.transform_data.clear();
        self.clipboard.material_data.clear();

        for &object_id in &self.selection.objects {
            // TODO: Get actual object data from scene
            let clipboard_object = ClipboardObject {
                id: object_id,
                mesh_data: vec![], // Placeholder
                transform: Mat4::identity(),
                material_id: 0,
            };
            self.clipboard.objects.push(clipboard_object);
        }
    }

    pub fn paste(&mut self) -> Vec<u32> {
        let mut new_ids = Vec::new();

        for clipboard_object in &self.clipboard.objects {
            // TODO: Create new object in scene
            let new_id = clipboard_object.id + 1000; // Placeholder ID generation
            new_ids.push(new_id);
        }

        self.select_objects(new_ids.clone());
        new_ids
    }

    pub fn duplicate_selected(&mut self) -> Vec<u32> {
        self.copy_selected();
        self.paste()
    }

    pub fn delete_selected(&mut self) {
        let deleted_objects = self.selection.objects.clone();

        // Record action for undo
        let action = EditorAction {
            name: "Delete Objects".to_string(),
            timestamp: 0, // TODO: Get actual timestamp
            undo_data: ActionData::Delete(deleted_objects.clone()),
            redo_data: ActionData::Delete(deleted_objects),
        };
        self.history.add_action(action);

        // TODO: Actually delete objects from scene
        self.clear_selection();
    }

    pub fn undo(&mut self) -> bool {
        self.history.undo()
    }

    pub fn redo(&mut self) -> bool {
        self.history.redo()
    }

    pub fn transform_selected(&mut self, transform: Mat4) {
        if self.selection.objects.is_empty() {
            return;
        }

        // TODO: Apply transform to selected objects
        // Record action for undo
        let action = EditorAction {
            name: "Transform Objects".to_string(),
            timestamp: 0, // TODO: Get actual timestamp
            undo_data: ActionData::Transform(vec![]), // TODO: Store previous transforms
            redo_data: ActionData::Transform(vec![]), // TODO: Store new transforms
        };
        self.history.add_action(action);
    }

    pub fn extrude_selected(&mut self, distance: f32) {
        // TODO: Implement extrusion for selected faces/objects
    }

    pub fn mirror_selected(&mut self, axis: GizmoAxis) {
        // TODO: Implement mirroring along specified axis
    }

    pub fn array_selected(&mut self, count: u32, offset: Vec3) {
        // TODO: Create array of selected objects
    }
}

impl TransformGizmo {
    pub fn new() -> Self {
        Self {
            visible: false,
            mode: GizmoMode::Universal,
            space: TransformSpace::World,
            size: 1.0,
            highlight: None,
            position: Vec3::zero(),
            rotation: Quat::one(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn set_mode(&mut self, mode: GizmoMode) {
        self.mode = mode;
    }

    pub fn set_space(&mut self, space: TransformSpace) {
        self.space = space;
    }

    pub fn hit_test(&self, ray_origin: Vec3, ray_direction: Vec3) -> Option<GizmoAxis> {
        // TODO: Implement ray-gizmo intersection testing
        None
    }
}

impl SnappingSettings {
    pub fn default() -> Self {
        Self {
            enabled: false,
            grid_snap: true,
            vertex_snap: false,
            edge_snap: false,
            face_snap: false,
            angle_snap: false,
            grid_size: 1.0,
            angle_increment: 15.0,
        }
    }

    pub fn snap_position(&self, position: Vec3) -> Vec3 {
        if !self.enabled || !self.grid_snap {
            return position;
        }

        Vec3::new(
            (position.x / self.grid_size).round() * self.grid_size,
            (position.y / self.grid_size).round() * self.grid_size,
            (position.z / self.grid_size).round() * self.grid_size,
        )
    }

    pub fn snap_angle(&self, angle: f32) -> f32 {
        if !self.enabled || !self.angle_snap {
            return angle;
        }

        let increment_rad = self.angle_increment.to_radians();
        (angle / increment_rad).round() * increment_rad
    }
}

impl SelectionSet {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            vertices: Vec::new(),
            edges: Vec::new(),
            faces: Vec::new(),
            mode: SelectionMode::Object,
            bounding_box: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        match self.mode {
            SelectionMode::Object => self.objects.is_empty(),
            SelectionMode::Vertex => self.vertices.is_empty(),
            SelectionMode::Edge => self.edges.is_empty(),
            SelectionMode::Face => self.faces.is_empty(),
            SelectionMode::Component => {
                self.vertices.is_empty() && self.edges.is_empty() && self.faces.is_empty()
            }
        }
    }

    pub fn count(&self) -> usize {
        match self.mode {
            SelectionMode::Object => self.objects.len(),
            SelectionMode::Vertex => self.vertices.len(),
            SelectionMode::Edge => self.edges.len(),
            SelectionMode::Face => self.faces.len(),
            SelectionMode::Component => {
                self.vertices.len() + self.edges.len() + self.faces.len()
            }
        }
    }
}

impl EditorClipboard {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            transform_data: Vec::new(),
            material_data: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    pub fn clear(&mut self) {
        self.objects.clear();
        self.transform_data.clear();
        self.material_data.clear();
    }
}

impl EditorViewport {
    pub fn default() -> Self {
        Self {
            camera_position: Vec3::new(0.0, 5.0, 10.0),
            camera_target: Vec3::zero(),
            field_of_view: 45.0,
            near_plane: 0.1,
            far_plane: 1000.0,
            wireframe_mode: false,
            shading_mode: ShadingMode::Textured,
            grid_visible: true,
            axes_visible: true,
            statistics_visible: false,
        }
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(
            Point3::new(self.camera_position.x, self.camera_position.y, self.camera_position.z),
            Point3::new(self.camera_target.x, self.camera_target.y, self.camera_target.z),
            Vec3::unit_y()
        )
    }

    pub fn get_projection_matrix(&self, aspect_ratio: f32) -> Mat4 {
        perspective(
            Rad(self.field_of_view.to_radians()),
            aspect_ratio,
            self.near_plane,
            self.far_plane,
        )
    }
}

impl EditorHistory {
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
            current_index: 0,
            max_history: 100,
        }
    }

    pub fn add_action(&mut self, action: EditorAction) {
        // Remove any actions after current index
        self.actions.truncate(self.current_index);

        // Add new action
        self.actions.push(action);
        self.current_index = self.actions.len();

        // Limit history size
        if self.actions.len() > self.max_history {
            self.actions.remove(0);
            self.current_index = self.actions.len();
        }
    }

    pub fn can_undo(&self) -> bool {
        self.current_index > 0
    }

    pub fn can_redo(&self) -> bool {
        self.current_index < self.actions.len()
    }

    pub fn undo(&mut self) -> bool {
        if self.can_undo() {
            self.current_index -= 1;
            // TODO: Apply undo data
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if self.can_redo() {
            // TODO: Apply redo data
            self.current_index += 1;
            true
        } else {
            false
        }
    }

    pub fn get_current_action(&self) -> Option<&EditorAction> {
        if self.current_index > 0 {
            self.actions.get(self.current_index - 1)
        } else {
            None
        }
    }
}

impl EditorPreferences {
    pub fn default() -> Self {
        Self {
            auto_save_interval: 300, // 5 minutes
            backup_count: 5,
            grid_size: 1.0,
            camera_speed: 5.0,
            selection_outline_color: [1.0, 0.5, 0.0, 1.0], // Orange
            gizmo_size: 1.0,
            snap_distance: 0.5,
            double_click_time: 500, // milliseconds
        }
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), std::io::Error> {
        // TODO: Serialize and save preferences
        Ok(())
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<(), std::io::Error> {
        // TODO: Load and deserialize preferences
        Ok(())
    }
}