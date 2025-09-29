/*!
 * Engineer Build Mode - Core game creation interface
 *
 * This module implements Robin's revolutionary Engineer Build Mode where users
 * create games using FPS-style tools in a 3D environment. Game logic is
 * represented as physical objects that can be placed and connected spatially.
 */

use crate::engine::{
    math::{Vec3, Vec2},
    input::InputManager,
    error::RobinResult,
};
use cgmath::InnerSpace;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, VecDeque};
use crate::engine::world::VoxelType;

pub mod tools;
pub mod logic;
pub mod interactive_elements;
pub mod element_placement_tool;
pub mod mode_system;
pub mod content_creation;
pub mod components;
pub mod testing;
pub mod editor;
pub mod enhanced_templates;

// Build Mode tools and systems - explicit exports to avoid conflicts
pub use tools::{BuildTool, ToolKit};
pub use logic::{LogicSystem, LogicNode, LogicNodeType, LogicValue, VisualLogicSystem};
pub use logic::ActionType as LogicActionType;
pub use interactive_elements::{InteractiveElement, InteractiveElementsSystem};
pub use interactive_elements::TriggerType as InteractionTriggerType;
pub use element_placement_tool::{ElementPlacementTool, PlacementSettings};
pub use mode_system::{ModeSystem, ModeSettings, PerformanceStats, OptimizationSuggestion, PerformanceProfile};
pub use mode_system::{PerformanceMonitor as BuildPerformanceMonitor, DebugOverlay as BuildDebugOverlay};
pub use content_creation::{ContentCreationSystem, ContentType, ContentTemplate, QualityMetrics};
pub use content_creation::ActionType as CreationActionType;
pub use components::{ComponentLibrary, InteractiveComponent};
pub use components::TriggerType as ComponentTriggerType;
pub use testing::{TestingSystem, TestScenario};
pub use testing::{PerformanceMonitor as TestPerformanceMonitor, DebugOverlay as TestDebugOverlay};
pub use editor::{Editor, EditorTool, EditorPreferences};
pub use enhanced_templates::{
    EnhancedTemplateLibrary, EnhancedTemplate, TemplateCategory, TemplateComplexity,
    TemplateStructure, BuildRequirements, TemplateVariation, InteractiveTemplateElement,
    TemplateAnimation, TemplateOptimization, SearchCriteria
};

/// Enhanced build modes for voxel construction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BuildMode {
    Single,
    Wall,
    Floor,
    Roof,
    Template,
    // Enhanced Build Modes (from archived demos)
    Circle,      // Circular structures
    Sphere,      // 3D spherical structures
    Terrain,     // Terrain sculpting
    Copy,        // Copy existing structures
    Paste,       // Paste copied structures
}

/// Template types for predefined structure building
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TemplateType {
    Stairs,
    Arch,
    Bridge,
    Tower,
    House,
    // Enhanced Templates (from archived demos)
    Castle,      // Large fortified structures
    Garden,      // Landscaped areas
    Workshop,    // Crafting and tool areas
    Fortress,    // Military installations
    Lighthouse,  // Navigation towers
    Windmill,    // Resource generation
}

/// Simple build system for voxel-based construction (compatibility layer)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoxelBuildSystem {
    mode: BuildMode,
    current_material: VoxelType,
    current_template: TemplateType,
    inventory: HashMap<VoxelType, u32>,
    grid_snap: bool,
    undo_stack: VecDeque<VoxelBuildAction>,
    redo_stack: VecDeque<VoxelBuildAction>,
    // Enhanced template support for compatibility
    current_enhanced_template: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoxelBuildAction {
    pub operations: Vec<VoxelBuildOperation>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoxelBuildOperation {
    pub position: (i32, i32, i32),
    pub old_voxel: VoxelType,
    pub new_voxel: VoxelType,
}

impl VoxelBuildSystem {
    pub fn new() -> Self {
        let mut inventory = HashMap::new();

        // Initialize with basic materials
        inventory.insert(VoxelType::Stone, 1000);
        inventory.insert(VoxelType::Wood, 500);
        inventory.insert(VoxelType::Dirt, 1000);
        inventory.insert(VoxelType::Grass, 200);
        inventory.insert(VoxelType::Sand, 300);
        inventory.insert(VoxelType::Glass, 100);
        inventory.insert(VoxelType::Metal, 150);
        inventory.insert(VoxelType::Brick, 250);

        Self {
            mode: BuildMode::Single,
            current_material: VoxelType::Stone,
            current_template: TemplateType::Stairs,
            inventory,
            grid_snap: true,
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            current_enhanced_template: None,
        }
    }

    pub fn get_current_mode(&self) -> BuildMode {
        self.mode
    }

    pub fn get_mode(&self) -> BuildMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: BuildMode) {
        self.mode = mode;
    }

    pub fn get_current_material(&self) -> VoxelType {
        self.current_material
    }

    pub fn set_material(&mut self, material: VoxelType) {
        self.current_material = material;
    }

    pub fn get_current_template(&self) -> TemplateType {
        self.current_template
    }

    pub fn cycle_template(&mut self) {
        self.current_template = match self.current_template {
            TemplateType::Stairs => TemplateType::Arch,
            TemplateType::Arch => TemplateType::Bridge,
            TemplateType::Bridge => TemplateType::Tower,
            TemplateType::Tower => TemplateType::House,
            TemplateType::House => TemplateType::Castle,
            TemplateType::Castle => TemplateType::Garden,
            TemplateType::Garden => TemplateType::Workshop,
            TemplateType::Workshop => TemplateType::Fortress,
            TemplateType::Fortress => TemplateType::Lighthouse,
            TemplateType::Lighthouse => TemplateType::Windmill,
            TemplateType::Windmill => TemplateType::Stairs,
        };
    }

    pub fn is_grid_snap_enabled(&self) -> bool {
        self.grid_snap
    }

    pub fn toggle_grid_snap(&mut self) {
        self.grid_snap = !self.grid_snap;
    }

    pub fn get_inventory(&self) -> &HashMap<VoxelType, u32> {
        &self.inventory
    }

    pub fn add_to_inventory(&mut self, material: VoxelType, amount: u32) {
        *self.inventory.entry(material).or_insert(0) += amount;
    }

    pub fn use_material(&mut self, material: VoxelType, amount: u32) -> bool {
        if let Some(count) = self.inventory.get_mut(&material) {
            if *count >= amount {
                *count -= amount;
                return true;
            }
        }
        false
    }

    pub fn record_action(&mut self, action: VoxelBuildAction) {
        self.undo_stack.push_back(action);
        self.redo_stack.clear();

        // Limit undo history
        if self.undo_stack.len() > 100 {
            self.undo_stack.pop_front();
        }
    }

    pub fn undo(&mut self) -> Option<VoxelBuildAction> {
        if let Some(action) = self.undo_stack.pop_back() {
            self.redo_stack.push_back(action.clone());
            Some(action)
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<VoxelBuildAction> {
        if let Some(action) = self.redo_stack.pop_back() {
            self.undo_stack.push_back(action.clone());
            Some(action)
        } else {
            None
        }
    }

    pub fn cycle_mode(&mut self) {
        self.mode = match self.mode {
            BuildMode::Single => BuildMode::Wall,
            BuildMode::Wall => BuildMode::Floor,
            BuildMode::Floor => BuildMode::Roof,
            BuildMode::Roof => BuildMode::Template,
            BuildMode::Template => BuildMode::Circle,
            BuildMode::Circle => BuildMode::Sphere,
            BuildMode::Sphere => BuildMode::Terrain,
            BuildMode::Terrain => BuildMode::Copy,
            BuildMode::Copy => BuildMode::Paste,
            BuildMode::Paste => BuildMode::Single,
        };
    }

    /// Get current enhanced template ID
    pub fn get_current_enhanced_template(&self) -> Option<&String> {
        self.current_enhanced_template.as_ref()
    }

    /// Set current enhanced template
    pub fn set_enhanced_template(&mut self, template_id: Option<String>) {
        self.current_enhanced_template = template_id;
        if let Some(ref id) = self.current_enhanced_template {
            log::debug!("Selected enhanced template: {}", id);
        } else {
            log::debug!("Cleared enhanced template selection");
        }
    }

    /// Check if using enhanced template mode
    pub fn is_using_enhanced_template(&self) -> bool {
        self.current_enhanced_template.is_some()
    }
}

/// The main Engineer Build Mode system
pub struct EngineerBuildMode {
    /// Current active tool
    active_tool: Option<BuildTool>,

    /// Available tools in the engineer's toolkit
    tools: ToolKit,

    /// Advanced mode system with transitions and state management
    mode_system: ModeSystem,

    /// Grid settings for snapping
    grid: GridSystem,

    /// Selection system for multi-object operations
    selection: SelectionManager,

    /// History for undo/redo operations
    history: BuildHistory,

    /// Camera and viewport settings
    viewport: BuildViewport,

    /// Interactive elements system
    elements_system: InteractiveElementsSystem,

    /// Element placement tool
    element_placement_tool: ElementPlacementTool,

    /// Visual logic system
    logic_system: VisualLogicSystem,

    /// Content creation system for projects, templates, and wizards
    content_creation_system: ContentCreationSystem,

    /// Enhanced template library for sophisticated content creation
    enhanced_template_library: EnhancedTemplateLibrary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BuildModeState {
    /// Full creative control with all tools
    Build,
    /// Play mode with debug overlay
    Test,
    /// Pure play mode as end user would experience
    Play,
}

impl EngineerBuildMode {
    pub fn new() -> Self {
        Self {
            active_tool: None,
            tools: ToolKit::new(),
            mode_system: ModeSystem::new(),
            grid: GridSystem::new(),
            selection: SelectionManager::new(),
            history: BuildHistory::new(),
            viewport: BuildViewport::new(),
            elements_system: InteractiveElementsSystem::new(),
            element_placement_tool: ElementPlacementTool::new(),
            logic_system: VisualLogicSystem::new(),
            content_creation_system: ContentCreationSystem::new(),
            enhanced_template_library: EnhancedTemplateLibrary::new(),
        }
    }

    /// Update the build mode system
    pub fn update(&mut self, delta_time: f32, input: &InputManager) -> RobinResult<()> {
        // Update the advanced mode system first
        self.mode_system.update(delta_time, input)?;

        // Get current mode from the mode system
        let current_mode = self.mode_system.get_current_mode();

        match current_mode {
            BuildModeState::Build => {
                self.update_build_mode(delta_time, input)?;
            }
            BuildModeState::Test => {
                self.update_test_mode(delta_time, input)?;
            }
            BuildModeState::Play => {
                self.update_play_mode(delta_time, input)?;
            }
        }

        Ok(())
    }

    /// Switch between build modes
    pub fn cycle_mode(&mut self) {
        if let Err(e) = self.mode_system.cycle_mode() {
            log::error!("Failed to cycle mode: {:?}", e);
        }
    }

    /// Get current mode
    pub fn get_mode(&self) -> BuildModeState {
        self.mode_system.get_current_mode()
    }

    /// Switch to a specific mode
    pub fn switch_mode(&mut self, mode: BuildModeState) {
        if let Err(e) = self.mode_system.switch_mode(mode) {
            log::error!("Failed to switch to {:?} mode: {:?}", mode, e);
        }
    }

    /// Check if currently transitioning between modes
    pub fn is_transitioning(&self) -> bool {
        self.mode_system.is_transitioning()
    }

    /// Get transition progress (0.0 to 1.0)
    pub fn get_transition_progress(&self) -> f32 {
        self.mode_system.get_transition_progress()
    }

    /// Set active tool
    pub fn set_active_tool(&mut self, tool: BuildTool) {
        self.active_tool = Some(tool);
        log::debug!("Activated {:?} tool", tool);
    }

    /// Get active tool
    pub fn get_active_tool(&self) -> Option<&BuildTool> {
        self.active_tool.as_ref()
    }

    fn update_build_mode(&mut self, delta_time: f32, input: &InputManager) -> RobinResult<()> {
        // Update tool selection (number keys 1-9) - only if tools are enabled for this mode
        let enabled_tools = self.mode_system.get_enabled_tools();
        for i in 1..=9 {
            let key = match i {
                1 => winit::keyboard::Key::Character("1".into()),
                2 => winit::keyboard::Key::Character("2".into()),
                3 => winit::keyboard::Key::Character("3".into()),
                4 => winit::keyboard::Key::Character("4".into()),
                5 => winit::keyboard::Key::Character("5".into()),
                6 => winit::keyboard::Key::Character("6".into()),
                7 => winit::keyboard::Key::Character("7".into()),
                8 => winit::keyboard::Key::Character("8".into()),
                9 => winit::keyboard::Key::Character("9".into()),
                _ => continue,
            };

            if input.is_key_just_pressed(&key) {
                if let Some(tool) = self.tools.get_tool_by_index(i - 1) {
                    // Check if tool is enabled for current mode
                    if enabled_tools.iter().any(|t| t.contains(&format!("{:?}", tool))) {
                        self.set_active_tool(tool);
                    }
                }
            }
        }

        // Update active tool if enabled
        if let Some(tool) = &mut self.active_tool {
            self.tools.update_tool(tool, delta_time, input, &mut self.grid, &mut self.selection)?;
        }

        // Update interactive elements and placement tool (enabled in build mode)
        if self.mode_system.is_tool_enabled("element_placement") {
            self.element_placement_tool.update(delta_time, input, &self.grid, &self.selection)?;
        }

        // Update visual logic system (enabled in build mode)
        if self.mode_system.is_tool_enabled("logic_connector") {
            self.logic_system.update(delta_time, input)?;
        }

        // Update content creation system (always enabled in build mode)
        self.content_creation_system.update(delta_time, input)?;

        // Update systems
        self.grid.update(delta_time, input);
        self.selection.update(delta_time, input);
        self.history.update(delta_time);
        self.viewport.update(delta_time, input);

        // Handle undo/redo
        if input.is_key_pressed(&winit::keyboard::Key::Named(winit::keyboard::NamedKey::Control)) {
            if input.is_key_just_pressed(&winit::keyboard::Key::Character("z".into())) {
                if input.is_key_pressed(&winit::keyboard::Key::Named(winit::keyboard::NamedKey::Shift)) {
                    self.history.redo()?;
                } else {
                    self.history.undo()?;
                }
            }
        }

        Ok(())
    }

    fn update_test_mode(&mut self, delta_time: f32, input: &InputManager) -> RobinResult<()> {
        // Test mode allows basic movement and interaction but shows debug overlays
        self.viewport.update(delta_time, input);

        // Update interactive elements for testing
        let player_position = cgmath::Vector3::new(
            self.viewport.camera_position.x,
            self.viewport.camera_position.y,
            self.viewport.camera_position.z,
        );
        self.elements_system.update(delta_time, player_position);

        // Update logic system for testing
        self.logic_system.update(delta_time, input)?;

        // Limited tool access for quick edits
        if self.mode_system.is_tool_enabled("quick_edit") {
            self.element_placement_tool.update(delta_time, input, &self.grid, &self.selection)?;
        }

        // Show debug overlays based on mode settings
        let debug_overlays = self.mode_system.get_debug_overlays();
        for overlay in debug_overlays {
            // TODO: Render debug overlay based on type
            log::trace!("Debug overlay active: {:?}", overlay);
        }

        Ok(())
    }

    fn update_play_mode(&mut self, delta_time: f32, input: &InputManager) -> RobinResult<()> {
        // Pure play mode - focus on game experience with minimal developer tools
        self.viewport.update(delta_time, input);

        // Update interactive elements for gameplay
        let player_position = cgmath::Vector3::new(
            self.viewport.camera_position.x,
            self.viewport.camera_position.y,
            self.viewport.camera_position.z,
        );
        self.elements_system.update(delta_time, player_position);

        // Execute logic system for gameplay
        self.logic_system.update(delta_time, input)?;

        // No tools available in play mode by default
        // History is still updated for potential debugging

        Ok(())
    }

    /// Get grid system for external systems
    pub fn get_grid(&self) -> &GridSystem {
        &self.grid
    }

    /// Get selection manager for external systems
    pub fn get_selection(&self) -> &SelectionManager {
        &self.selection
    }

    /// Get build history for external systems
    pub fn get_history(&self) -> &BuildHistory {
        &self.history
    }

    /// Add an action to the history
    pub fn add_history_action(&mut self, action: BuildAction) {
        self.history.add_action(action);
    }

    /// Get the interactive elements system
    pub fn get_elements_system(&self) -> &InteractiveElementsSystem {
        &self.elements_system
    }

    /// Get mutable access to the interactive elements system
    pub fn get_elements_system_mut(&mut self) -> &mut InteractiveElementsSystem {
        &mut self.elements_system
    }

    /// Get the element placement tool
    pub fn get_element_placement_tool(&self) -> &ElementPlacementTool {
        &self.element_placement_tool
    }

    /// Get mutable access to the element placement tool
    pub fn get_element_placement_tool_mut(&mut self) -> &mut ElementPlacementTool {
        &mut self.element_placement_tool
    }

    /// Get the visual logic system
    pub fn get_logic_system(&self) -> &VisualLogicSystem {
        &self.logic_system
    }

    /// Get mutable access to the visual logic system
    pub fn get_logic_system_mut(&mut self) -> &mut VisualLogicSystem {
        &mut self.logic_system
    }

    /// Get the mode system
    pub fn get_mode_system(&self) -> &ModeSystem {
        &self.mode_system
    }

    /// Get mutable access to the mode system
    pub fn get_mode_system_mut(&mut self) -> &mut ModeSystem {
        &mut self.mode_system
    }

    /// Get current performance statistics
    pub fn get_performance_stats(&self) -> PerformanceStats {
        self.mode_system.get_performance_stats()
    }

    /// Get current optimization suggestions
    pub fn get_optimization_suggestions(&self) -> &[OptimizationSuggestion] {
        self.mode_system.get_optimization_suggestions()
    }

    /// Apply a performance profile to the current mode
    pub fn apply_performance_profile(&mut self, profile: PerformanceProfile) -> RobinResult<()> {
        self.mode_system.apply_performance_profile(profile)
    }

    /// Get the content creation system
    pub fn get_content_creation_system(&self) -> &ContentCreationSystem {
        &self.content_creation_system
    }

    /// Get mutable access to the content creation system
    pub fn get_content_creation_system_mut(&mut self) -> &mut ContentCreationSystem {
        &mut self.content_creation_system
    }

    /// Create a new content project from a template
    pub fn create_project_from_template(&mut self, template_name: &str, project_name: String) -> RobinResult<String> {
        self.content_creation_system.create_new_project(template_name, project_name)
    }

    /// Save the current content project
    pub fn save_current_project(&mut self) -> RobinResult<()> {
        self.content_creation_system.save_project()
    }

    /// Get available content templates
    pub fn get_available_templates(&self) -> Vec<&ContentTemplate> {
        self.content_creation_system.get_available_templates()
    }

    /// Start a content creation wizard
    pub fn start_content_wizard(&mut self, wizard_name: &str) -> RobinResult<()> {
        self.content_creation_system.start_wizard(wizard_name)
    }

    /// Get current project quality metrics
    pub fn get_project_quality_metrics(&self) -> Option<QualityMetrics> {
        self.content_creation_system.get_quality_metrics()
    }

    /// Get the enhanced template library
    pub fn get_enhanced_template_library(&self) -> &EnhancedTemplateLibrary {
        &self.enhanced_template_library
    }

    /// Get mutable access to the enhanced template library
    pub fn get_enhanced_template_library_mut(&mut self) -> &mut EnhancedTemplateLibrary {
        &mut self.enhanced_template_library
    }

    /// Search enhanced templates by criteria
    pub fn search_enhanced_templates(&self, criteria: &SearchCriteria) -> Vec<&EnhancedTemplate> {
        self.enhanced_template_library.search_templates(criteria)
    }

    /// Get enhanced template by ID
    pub fn get_enhanced_template(&self, template_id: &str) -> Option<&EnhancedTemplate> {
        self.enhanced_template_library.get_template(template_id)
    }

    /// Get enhanced templates by category
    pub fn get_enhanced_templates_by_category(&self, category: TemplateCategory) -> Vec<&EnhancedTemplate> {
        self.enhanced_template_library.get_templates_by_category(category)
    }

    /// Get featured enhanced templates
    pub fn get_featured_enhanced_templates(&self) -> Vec<&EnhancedTemplate> {
        self.enhanced_template_library.get_featured_templates()
    }

    /// Apply enhanced template at position
    pub fn apply_enhanced_template(&mut self, template_id: &str, position: Vec3, variation_id: Option<&str>) -> RobinResult<()> {
        if let Some(template) = self.enhanced_template_library.get_template(template_id) {
            // Create build action for applying template
            let action = BuildAction::CreateObject {
                object_id: rand::random(),
                object_type: format!("enhanced_template_{}", template_id),
                position,
            };

            self.add_history_action(action);

            // TODO: Implement actual template application with voxel placement
            log::info!("Applied enhanced template '{}' at position {:?}", template.name, position);

            if let Some(var_id) = variation_id {
                log::info!("Using template variation: {}", var_id);
            }

            Ok(())
        } else {
            Err(crate::engine::error::RobinError::ResourceNotFound {
                resource_type: "EnhancedTemplate".to_string(),
                resource_id: template_id.to_string(),
            })
        }
    }

    /// Get template compilation status
    pub fn get_template_compilation_status(&self, template_id: &str) -> Option<bool> {
        self.enhanced_template_library.is_template_compiled(template_id)
    }

    /// Compile template for optimized use
    pub fn compile_enhanced_template(&mut self, template_id: &str) -> RobinResult<()> {
        self.enhanced_template_library.compile_template(template_id)
    }
}

/// Grid system for precise object placement
#[derive(Debug, Clone)]
pub struct GridSystem {
    pub enabled: bool,
    pub size: f32,
    pub visible: bool,
    pub snap_position: bool,
    pub snap_rotation: bool,
    pub snap_scale: bool,
}

impl GridSystem {
    pub fn new() -> Self {
        Self {
            enabled: true,
            size: 1.0,
            visible: true,
            snap_position: true,
            snap_rotation: true,
            snap_scale: false,
        }
    }

    pub fn update(&mut self, _delta_time: f32, input: &InputManager) {
        // Toggle grid with G key
        if input.is_key_just_pressed(&winit::keyboard::Key::Character("g".into())) {
            self.enabled = !self.enabled;
            log::debug!("Grid {}", if self.enabled { "enabled" } else { "disabled" });
        }

        // Adjust grid size with [ and ]
        if input.is_key_just_pressed(&winit::keyboard::Key::Character("[".into())) {
            self.size = (self.size * 0.5).max(0.125);
            log::debug!("Grid size: {}", self.size);
        }
        if input.is_key_just_pressed(&winit::keyboard::Key::Character("]".into())) {
            self.size = (self.size * 2.0).min(16.0);
            log::debug!("Grid size: {}", self.size);
        }
    }

    /// Snap position to grid
    pub fn snap_position(&self, position: Vec3) -> Vec3 {
        if !self.enabled || !self.snap_position {
            return position;
        }

        Vec3::new(
            (position.x / self.size).round() * self.size,
            (position.y / self.size).round() * self.size,
            (position.z / self.size).round() * self.size,
        )
    }

    /// Snap rotation to grid (15-degree increments)
    pub fn snap_rotation(&self, rotation: f32) -> f32 {
        if !self.enabled || !self.snap_rotation {
            return rotation;
        }

        let snap_angle = 15.0_f32.to_radians();
        (rotation / snap_angle).round() * snap_angle
    }
}

/// Selection manager for multi-object operations
#[derive(Debug)]
pub struct SelectionManager {
    selected_objects: Vec<u32>, // Object IDs
    selection_box: Option<SelectionBox>,
    multi_select_mode: bool,
}

#[derive(Debug)]
pub struct SelectionBox {
    start: Vec3,
    end: Vec3,
    active: bool,
}

impl SelectionManager {
    pub fn new() -> Self {
        Self {
            selected_objects: Vec::new(),
            selection_box: None,
            multi_select_mode: false,
        }
    }

    pub fn update(&mut self, _delta_time: f32, input: &InputManager) {
        // Toggle multi-select with Shift
        self.multi_select_mode = input.is_key_pressed(&winit::keyboard::Key::Named(winit::keyboard::NamedKey::Shift));

        // Clear selection with Escape
        if input.is_key_just_pressed(&winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape)) {
            self.clear_selection();
        }
    }

    pub fn add_to_selection(&mut self, object_id: u32) {
        if !self.multi_select_mode {
            self.selected_objects.clear();
        }

        if !self.selected_objects.contains(&object_id) {
            self.selected_objects.push(object_id);
        }
    }

    pub fn remove_from_selection(&mut self, object_id: u32) {
        self.selected_objects.retain(|&id| id != object_id);
    }

    pub fn clear_selection(&mut self) {
        self.selected_objects.clear();
    }

    pub fn get_selected_objects(&self) -> &[u32] {
        &self.selected_objects
    }

    pub fn is_selected(&self, object_id: u32) -> bool {
        self.selected_objects.contains(&object_id)
    }
}

/// Build history for undo/redo operations
#[derive(Debug)]
pub struct BuildHistory {
    actions: Vec<BuildAction>,
    current_index: usize,
    max_history: usize,
}

#[derive(Debug, Clone)]
pub enum BuildAction {
    PlaceVoxel { position: Vec3, material_id: u32 },
    RemoveVoxel { position: Vec3, material_id: u32 },
    MoveObject { object_id: u32, old_pos: Vec3, new_pos: Vec3 },
    RotateObject { object_id: u32, old_rot: Vec3, new_rot: Vec3 },
    ScaleObject { object_id: u32, old_scale: Vec3, new_scale: Vec3 },
    CreateObject { object_id: u32, object_type: String, position: Vec3 },
    DeleteObject { object_id: u32, object_type: String, position: Vec3 },
}

impl BuildHistory {
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
            current_index: 0,
            max_history: 1000,
        }
    }

    pub fn update(&mut self, _delta_time: f32) {
        // Clean up old history if needed
        if self.actions.len() > self.max_history {
            let remove_count = self.actions.len() - self.max_history;
            self.actions.drain(0..remove_count);
            self.current_index = self.current_index.saturating_sub(remove_count);
        }
    }

    pub fn add_action(&mut self, action: BuildAction) {
        // Remove any actions after current index (when user does something after undoing)
        if self.current_index < self.actions.len() {
            self.actions.truncate(self.current_index);
        }

        self.actions.push(action);
        self.current_index = self.actions.len();

        log::debug!("Added build action, history size: {}", self.actions.len());
    }

    pub fn undo(&mut self) -> RobinResult<()> {
        if self.current_index > 0 {
            self.current_index -= 1;
            let action = &self.actions[self.current_index];
            self.execute_undo_action(action)?;
            log::debug!("Undid action: {:?}", action);
        }
        Ok(())
    }

    pub fn redo(&mut self) -> RobinResult<()> {
        if self.current_index < self.actions.len() {
            let action = &self.actions[self.current_index];
            self.execute_redo_action(action)?;
            self.current_index += 1;
            log::debug!("Redid action: {:?}", action);
        }
        Ok(())
    }

    fn execute_undo_action(&self, action: &BuildAction) -> RobinResult<()> {
        match action {
            BuildAction::PlaceVoxel { position, .. } => {
                // Remove the voxel that was placed
                log::debug!("Undo place voxel at {:?}", position);
                // TODO: Implement voxel removal
            }
            BuildAction::RemoveVoxel { position, material_id } => {
                // Restore the voxel that was removed
                log::debug!("Undo remove voxel at {:?}", position);
                // TODO: Implement voxel placement
            }
            BuildAction::MoveObject { object_id, old_pos, .. } => {
                // Move object back to old position
                log::debug!("Undo move object {} to {:?}", object_id, old_pos);
                // TODO: Implement object movement
            }
            // TODO: Implement other undo actions
            _ => {}
        }
        Ok(())
    }

    fn execute_redo_action(&self, action: &BuildAction) -> RobinResult<()> {
        match action {
            BuildAction::PlaceVoxel { position, material_id } => {
                // Place the voxel again
                log::debug!("Redo place voxel at {:?}", position);
                // TODO: Implement voxel placement
            }
            BuildAction::RemoveVoxel { position, .. } => {
                // Remove the voxel again
                log::debug!("Redo remove voxel at {:?}", position);
                // TODO: Implement voxel removal
            }
            BuildAction::MoveObject { object_id, new_pos, .. } => {
                // Move object to new position
                log::debug!("Redo move object {} to {:?}", object_id, new_pos);
                // TODO: Implement object movement
            }
            // TODO: Implement other redo actions
            _ => {}
        }
        Ok(())
    }

    pub fn can_undo(&self) -> bool {
        self.current_index > 0
    }

    pub fn can_redo(&self) -> bool {
        self.current_index < self.actions.len()
    }
}

/// Viewport and camera management for build mode
#[derive(Debug)]
pub struct BuildViewport {
    pub camera_position: Vec3,
    pub camera_rotation: Vec2, // pitch, yaw
    pub movement_speed: f32,
    pub look_sensitivity: f32,
    pub fly_mode: bool,
}

impl BuildViewport {
    pub fn new() -> Self {
        Self {
            camera_position: Vec3::new(0.0, 10.0, 10.0),
            camera_rotation: Vec2::new(0.0, 0.0),
            movement_speed: 10.0,
            look_sensitivity: 0.002,
            fly_mode: true,
        }
    }

    pub fn update(&mut self, delta_time: f32, input: &InputManager) {
        // Mouse look
        let mouse_delta = input.mouse_delta();
        if mouse_delta.0 != 0.0 || mouse_delta.1 != 0.0 {
            self.camera_rotation.x -= mouse_delta.1 as f32 * self.look_sensitivity;
            self.camera_rotation.y -= mouse_delta.0 as f32 * self.look_sensitivity;

            // Clamp pitch
            self.camera_rotation.x = self.camera_rotation.x.clamp(-1.5, 1.5);
        }

        // Movement
        let mut movement = Vec3::new(0.0, 0.0, 0.0);

        if input.is_key_pressed(&winit::keyboard::Key::Character("w".into())) {
            movement.z -= 1.0;
        }
        if input.is_key_pressed(&winit::keyboard::Key::Character("s".into())) {
            movement.z += 1.0;
        }
        if input.is_key_pressed(&winit::keyboard::Key::Character("a".into())) {
            movement.x -= 1.0;
        }
        if input.is_key_pressed(&winit::keyboard::Key::Character("d".into())) {
            movement.x += 1.0;
        }

        if self.fly_mode {
            if input.is_key_pressed(&winit::keyboard::Key::Named(winit::keyboard::NamedKey::Space)) {
                movement.y += 1.0;
            }
            if input.is_key_pressed(&winit::keyboard::Key::Named(winit::keyboard::NamedKey::Shift)) {
                movement.y -= 1.0;
            }
        }

        // Apply movement speed modifier
        let speed_multiplier = if input.is_key_pressed(&winit::keyboard::Key::Named(winit::keyboard::NamedKey::Control)) {
            0.1 // Slow mode
        } else if input.is_key_pressed(&winit::keyboard::Key::Named(winit::keyboard::NamedKey::Shift)) {
            5.0 // Fast mode
        } else {
            1.0
        };

        // Transform movement relative to camera orientation
        if movement.magnitude() > 0.0 {
            movement = movement.normalize();

            let yaw = self.camera_rotation.y;
            let movement_world = Vec3::new(
                movement.x * yaw.cos() - movement.z * yaw.sin(),
                movement.y,
                movement.x * yaw.sin() + movement.z * yaw.cos(),
            );

            self.camera_position += movement_world * self.movement_speed * speed_multiplier * delta_time;
        }

        // Toggle fly mode
        if input.is_key_just_pressed(&winit::keyboard::Key::Character("f".into())) {
            self.fly_mode = !self.fly_mode;
            log::debug!("Fly mode: {}", self.fly_mode);
        }
    }

    pub fn get_view_matrix(&self) -> [[f32; 4]; 4] {
        // TODO: Implement proper view matrix calculation
        // For now, return identity matrix
        [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }

    pub fn get_forward_vector(&self) -> Vec3 {
        let yaw = self.camera_rotation.y;
        let pitch = self.camera_rotation.x;

        Vec3::new(
            yaw.sin() * pitch.cos(),
            -pitch.sin(),
            -yaw.cos() * pitch.cos(),
        )
    }

    pub fn get_right_vector(&self) -> Vec3 {
        let yaw = self.camera_rotation.y;
        Vec3::new(yaw.cos(), 0.0, yaw.sin())
    }

    pub fn get_up_vector(&self) -> Vec3 {
        Vec3::new(0.0, 1.0, 0.0)
    }
}

impl Default for EngineerBuildMode {
    fn default() -> Self {
        Self::new()
    }
}