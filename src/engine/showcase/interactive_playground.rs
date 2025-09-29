/// Interactive Playground Showcase
///
/// Guided tutorials and sandbox features for learning voxel construction

use std::collections::HashMap;
use std::time::{Duration, Instant};
use cgmath::{Vector3, Vector2, Matrix4};
use crate::engine::{
    generation::voxel_system::{VoxelWorld, VoxelType},
    build_mode::{BuildMode, BuildTool, TemplateType},
    ui::production_ui::{UIComponent, UIColor},
};

/// Tutorial step in the playground
pub struct TutorialStep {
    pub id: String,
    pub title: String,
    pub description: String,
    pub objective: TutorialObjective,
    pub hints: Vec<String>,
    pub required_tools: Vec<BuildTool>,
    pub target_structure: Option<Vec<(Vector3<i32>, VoxelType)>>,
    pub completion_criteria: CompletionCriteria,
    pub reward: TutorialReward,
}

/// Tutorial objective types
#[derive(Debug, Clone)]
pub enum TutorialObjective {
    PlaceBlocks { count: u32, voxel_type: VoxelType },
    BuildStructure { template_name: String },
    UseTools { tools: Vec<BuildTool>, times: u32 },
    ReachHeight { height: i32 },
    CreatePattern { pattern_type: PatternType },
    ConnectPoints { start: Vector3<i32>, end: Vector3<i32> },
}

/// Pattern types for tutorials
#[derive(Debug, Clone)]
pub enum PatternType {
    Checkerboard,
    Spiral,
    Pyramid,
    Arch,
    Custom(String),
}

/// Completion criteria
#[derive(Debug, Clone)]
pub struct CompletionCriteria {
    pub blocks_placed: Option<u32>,
    pub structure_match: Option<f32>, // Percentage match
    pub time_limit: Option<Duration>,
    pub accuracy_threshold: Option<f32>,
}

/// Tutorial rewards
#[derive(Debug, Clone)]
pub struct TutorialReward {
    pub experience_points: u32,
    pub unlocked_tools: Vec<BuildTool>,
    pub unlocked_materials: Vec<VoxelType>,
    pub achievement: Option<String>,
}

/// Main Interactive Playground system
pub struct InteractivePlayground {
    // Tutorial system
    tutorials: Vec<Tutorial>,
    current_tutorial: Option<usize>,
    current_step: usize,
    tutorial_progress: HashMap<String, TutorialProgress>,

    // Sandbox features
    sandbox_world: VoxelWorld,
    available_materials: Vec<VoxelType>,
    quick_templates: Vec<QuickTemplate>,
    undo_history: Vec<WorldSnapshot>,
    redo_history: Vec<WorldSnapshot>,
    max_history_size: usize,

    // UI elements
    tutorial_overlay: TutorialOverlay,
    material_palette: MaterialPalette,
    tool_selector: ToolSelector,
    progress_tracker: ProgressTracker,

    // State
    creative_mode: bool,
    hints_enabled: bool,
    auto_save: bool,
    last_save_time: Instant,
}

/// Individual tutorial
pub struct Tutorial {
    pub id: String,
    pub name: String,
    pub description: String,
    pub difficulty: TutorialDifficulty,
    pub steps: Vec<TutorialStep>,
    pub estimated_time: Duration,
    pub prerequisites: Vec<String>,
}

/// Tutorial difficulty
#[derive(Debug, Clone, PartialEq)]
pub enum TutorialDifficulty {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

/// Tutorial progress tracking
pub struct TutorialProgress {
    pub tutorial_id: String,
    pub completed_steps: Vec<String>,
    pub total_time: Duration,
    pub attempts: u32,
    pub best_score: f32,
    pub completed: bool,
}

/// Quick template for sandbox
pub struct QuickTemplate {
    pub name: String,
    pub icon: String,
    pub voxel_data: Vec<(Vector3<i32>, VoxelType)>,
    pub description: String,
    pub category: TemplateCategory,
}

/// Template categories
#[derive(Debug, Clone, PartialEq)]
pub enum TemplateCategory {
    Structures,
    Decorations,
    Utilities,
    Nature,
    Abstract,
}

/// World snapshot for undo/redo
pub struct WorldSnapshot {
    pub timestamp: Instant,
    pub voxel_changes: Vec<(Vector3<i32>, Option<VoxelType>, Option<VoxelType>)>,
    pub description: String,
}

/// Tutorial overlay UI
pub struct TutorialOverlay {
    pub visible: bool,
    pub current_step_display: StepDisplay,
    pub hint_bubbles: Vec<HintBubble>,
    pub progress_bar: f32,
    pub objective_markers: Vec<ObjectiveMarker>,
}

/// Step display in overlay
pub struct StepDisplay {
    pub title: String,
    pub description: String,
    pub objectives: Vec<String>,
    pub hints_shown: u32,
    pub time_elapsed: Duration,
}

/// Hint bubble UI element
pub struct HintBubble {
    pub position: Vector2<f32>,
    pub text: String,
    pub priority: HintPriority,
    pub dismiss_time: Option<Duration>,
}

/// Hint priority levels
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum HintPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Objective marker in 3D space
pub struct ObjectiveMarker {
    pub world_position: Vector3<f32>,
    pub screen_position: Option<Vector2<f32>>,
    pub label: String,
    pub color: UIColor,
    pub pulse_animation: bool,
}

/// Material palette UI
pub struct MaterialPalette {
    pub visible: bool,
    pub materials: Vec<MaterialSlot>,
    pub selected_material: usize,
    pub favorites: Vec<VoxelType>,
    pub recent_materials: Vec<VoxelType>,
}

/// Material slot in palette
pub struct MaterialSlot {
    pub material: VoxelType,
    pub count: Option<u32>, // None for unlimited
    pub locked: bool,
    pub hotkey: Option<char>,
}

/// Tool selector UI
pub struct ToolSelector {
    pub visible: bool,
    pub tools: Vec<ToolSlot>,
    pub selected_tool: usize,
    pub tool_settings: HashMap<BuildTool, ToolSettings>,
}

/// Tool slot in selector
pub struct ToolSlot {
    pub tool: BuildTool,
    pub unlocked: bool,
    pub hotkey: Option<char>,
    pub usage_count: u32,
}

/// Tool-specific settings
pub struct ToolSettings {
    pub brush_size: u32,
    pub symmetry_mode: SymmetryMode,
    pub snap_to_grid: bool,
    pub preview_enabled: bool,
}

/// Symmetry modes
#[derive(Debug, Clone, PartialEq)]
pub enum SymmetryMode {
    None,
    MirrorX,
    MirrorY,
    MirrorZ,
    Radial { segments: u32 },
}

/// Progress tracker UI
pub struct ProgressTracker {
    pub visible: bool,
    pub current_progress: f32,
    pub milestones: Vec<Milestone>,
    pub achievements: Vec<Achievement>,
    pub statistics: PlaygroundStatistics,
}

/// Progress milestone
pub struct Milestone {
    pub name: String,
    pub description: String,
    pub progress: f32,
    pub completed: bool,
    pub reward: Option<TutorialReward>,
}

/// Achievement in playground
pub struct Achievement {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub unlocked: bool,
    pub unlock_date: Option<Instant>,
    pub rarity: AchievementRarity,
}

/// Achievement rarity levels
#[derive(Debug, Clone, PartialEq)]
pub enum AchievementRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

/// Playground statistics
pub struct PlaygroundStatistics {
    pub total_blocks_placed: u64,
    pub total_blocks_removed: u64,
    pub tutorials_completed: u32,
    pub templates_used: u32,
    pub play_time: Duration,
    pub favorite_material: Option<VoxelType>,
    pub favorite_tool: Option<BuildTool>,
}

impl InteractivePlayground {
    pub fn new() -> Self {
        let sandbox_world = VoxelWorld::new("Sandbox".to_string(), (100, 100, 100));

        Self {
            tutorials: Self::create_tutorials(),
            current_tutorial: None,
            current_step: 0,
            tutorial_progress: HashMap::new(),

            sandbox_world,
            available_materials: Self::get_default_materials(),
            quick_templates: Self::create_quick_templates(),
            undo_history: Vec::new(),
            redo_history: Vec::new(),
            max_history_size: 50,

            tutorial_overlay: TutorialOverlay {
                visible: false,
                current_step_display: StepDisplay {
                    title: String::new(),
                    description: String::new(),
                    objectives: Vec::new(),
                    hints_shown: 0,
                    time_elapsed: Duration::from_secs(0),
                },
                hint_bubbles: Vec::new(),
                progress_bar: 0.0,
                objective_markers: Vec::new(),
            },

            material_palette: MaterialPalette {
                visible: true,
                materials: Self::create_material_slots(),
                selected_material: 0,
                favorites: vec![VoxelType::Stone, VoxelType::Wood],
                recent_materials: Vec::new(),
            },

            tool_selector: ToolSelector {
                visible: true,
                tools: Self::create_tool_slots(),
                selected_tool: 0,
                tool_settings: Self::create_default_tool_settings(),
            },

            progress_tracker: ProgressTracker {
                visible: false,
                current_progress: 0.0,
                milestones: Vec::new(),
                achievements: Self::create_achievements(),
                statistics: PlaygroundStatistics {
                    total_blocks_placed: 0,
                    total_blocks_removed: 0,
                    tutorials_completed: 0,
                    templates_used: 0,
                    play_time: Duration::from_secs(0),
                    favorite_material: None,
                    favorite_tool: None,
                },
            },

            creative_mode: false,
            hints_enabled: true,
            auto_save: true,
            last_save_time: Instant::now(),
        }
    }

    /// Create tutorial content
    fn create_tutorials() -> Vec<Tutorial> {
        vec![
            Tutorial {
                id: "basic_placement".to_string(),
                name: "Block Placement Basics".to_string(),
                description: "Learn how to place and remove blocks".to_string(),
                difficulty: TutorialDifficulty::Beginner,
                steps: vec![
                    TutorialStep {
                        id: "place_first_block".to_string(),
                        title: "Place Your First Block".to_string(),
                        description: "Click to place a stone block on the platform".to_string(),
                        objective: TutorialObjective::PlaceBlocks {
                            count: 1,
                            voxel_type: VoxelType::Stone,
                        },
                        hints: vec![
                            "Left click to place blocks".to_string(),
                            "Aim at the ground platform".to_string(),
                        ],
                        required_tools: vec![BuildTool::PlaceTool],
                        target_structure: None,
                        completion_criteria: CompletionCriteria {
                            blocks_placed: Some(1),
                            structure_match: None,
                            time_limit: None,
                            accuracy_threshold: None,
                        },
                        reward: TutorialReward {
                            experience_points: 10,
                            unlocked_tools: vec![],
                            unlocked_materials: vec![],
                            achievement: Some("first_block".to_string()),
                        },
                    },
                ],
                estimated_time: Duration::from_secs(60),
                prerequisites: vec![],
            },
            Tutorial {
                id: "build_house".to_string(),
                name: "Build a Simple House".to_string(),
                description: "Construct your first complete structure".to_string(),
                difficulty: TutorialDifficulty::Intermediate,
                steps: vec![
                    TutorialStep {
                        id: "foundation".to_string(),
                        title: "Lay the Foundation".to_string(),
                        description: "Create a 5x5 foundation using stone blocks".to_string(),
                        objective: TutorialObjective::PlaceBlocks {
                            count: 25,
                            voxel_type: VoxelType::Stone,
                        },
                        hints: vec![
                            "Start at the corner".to_string(),
                            "Use the grid overlay for alignment".to_string(),
                        ],
                        required_tools: vec![BuildTool::PlaceTool],
                        target_structure: Some(Self::generate_foundation_voxels()),
                        completion_criteria: CompletionCriteria {
                            blocks_placed: Some(25),
                            structure_match: Some(0.8),
                            time_limit: Some(Duration::from_secs(120)),
                            accuracy_threshold: Some(0.8),
                        },
                        reward: TutorialReward {
                            experience_points: 50,
                            unlocked_tools: vec![BuildTool::LineTool],
                            unlocked_materials: vec![VoxelType::Wood],
                            achievement: Some("foundation_builder".to_string()),
                        },
                    },
                ],
                estimated_time: Duration::from_secs(300),
                prerequisites: vec!["basic_placement".to_string()],
            },
        ]
    }

    /// Generate foundation voxels for tutorial
    fn generate_foundation_voxels() -> Vec<(Vector3<i32>, VoxelType)> {
        let mut voxels = Vec::new();
        for x in 0..5 {
            for z in 0..5 {
                voxels.push((Vector3::new(x, 0, z), VoxelType::Stone));
            }
        }
        voxels
    }

    /// Get default materials
    fn get_default_materials() -> Vec<VoxelType> {
        vec![
            VoxelType::Stone,
            VoxelType::Wood,
            VoxelType::Brick,
            VoxelType::Glass,
            VoxelType::Metal,
            VoxelType::Concrete,
        ]
    }

    /// Create quick templates
    fn create_quick_templates() -> Vec<QuickTemplate> {
        vec![
            QuickTemplate {
                name: "Wall Section".to_string(),
                icon: "wall_icon".to_string(),
                voxel_data: Self::generate_wall_template(),
                description: "5x5 wall section".to_string(),
                category: TemplateCategory::Structures,
            },
            QuickTemplate {
                name: "Stairs".to_string(),
                icon: "stairs_icon".to_string(),
                voxel_data: Self::generate_stairs_template(),
                description: "Basic staircase".to_string(),
                category: TemplateCategory::Utilities,
            },
            QuickTemplate {
                name: "Tree".to_string(),
                icon: "tree_icon".to_string(),
                voxel_data: Self::generate_tree_template(),
                description: "Simple tree".to_string(),
                category: TemplateCategory::Nature,
            },
        ]
    }

    /// Generate wall template
    fn generate_wall_template() -> Vec<(Vector3<i32>, VoxelType)> {
        let mut voxels = Vec::new();
        for x in 0..5 {
            for y in 0..5 {
                voxels.push((Vector3::new(x, y, 0), VoxelType::Brick));
            }
        }
        voxels
    }

    /// Generate stairs template
    fn generate_stairs_template() -> Vec<(Vector3<i32>, VoxelType)> {
        let mut voxels = Vec::new();
        for i in 0..5 {
            for x in 0..=i {
                voxels.push((Vector3::new(x, i, 0), VoxelType::Stone));
            }
        }
        voxels
    }

    /// Generate tree template
    fn generate_tree_template() -> Vec<(Vector3<i32>, VoxelType)> {
        let mut voxels = Vec::new();

        // Trunk
        for y in 0..4 {
            voxels.push((Vector3::new(0, y, 0), VoxelType::Wood));
        }

        // Leaves
        for x in -1..=1 {
            for z in -1..=1 {
                for y in 4..6 {
                    if x != 0 || z != 0 || y != 4 {
                        voxels.push((Vector3::new(x, y, z), VoxelType::Solid)); // Using Solid for leaves
                    }
                }
            }
        }

        voxels
    }

    /// Create material slots
    fn create_material_slots() -> Vec<MaterialSlot> {
        Self::get_default_materials()
            .into_iter()
            .enumerate()
            .map(|(i, material)| MaterialSlot {
                material,
                count: None, // Unlimited in playground
                locked: false,
                hotkey: (i < 9).then(|| char::from_digit((i + 1) as u32, 10).unwrap()),
            })
            .collect()
    }

    /// Create tool slots
    fn create_tool_slots() -> Vec<ToolSlot> {
        vec![
            ToolSlot {
                tool: BuildTool::PlaceTool,
                unlocked: true,
                hotkey: Some('Q'),
                usage_count: 0,
            },
            ToolSlot {
                tool: BuildTool::RemoveTool,
                unlocked: true,
                hotkey: Some('E'),
                usage_count: 0,
            },
            ToolSlot {
                tool: BuildTool::LineTool,
                unlocked: false,
                hotkey: Some('R'),
                usage_count: 0,
            },
            ToolSlot {
                tool: BuildTool::BoxTool,
                unlocked: false,
                hotkey: Some('T'),
                usage_count: 0,
            },
        ]
    }

    /// Create default tool settings
    fn create_default_tool_settings() -> HashMap<BuildTool, ToolSettings> {
        let mut settings = HashMap::new();

        settings.insert(BuildTool::PlaceTool, ToolSettings {
            brush_size: 1,
            symmetry_mode: SymmetryMode::None,
            snap_to_grid: true,
            preview_enabled: true,
        });

        settings.insert(BuildTool::RemoveTool, ToolSettings {
            brush_size: 1,
            symmetry_mode: SymmetryMode::None,
            snap_to_grid: true,
            preview_enabled: true,
        });

        settings
    }

    /// Create achievements
    fn create_achievements() -> Vec<Achievement> {
        vec![
            Achievement {
                id: "first_block".to_string(),
                name: "First Steps".to_string(),
                description: "Place your first block".to_string(),
                icon: "block_icon".to_string(),
                unlocked: false,
                unlock_date: None,
                rarity: AchievementRarity::Common,
            },
            Achievement {
                id: "foundation_builder".to_string(),
                name: "Foundation Builder".to_string(),
                description: "Complete a foundation".to_string(),
                icon: "foundation_icon".to_string(),
                unlocked: false,
                unlock_date: None,
                rarity: AchievementRarity::Uncommon,
            },
            Achievement {
                id: "master_builder".to_string(),
                name: "Master Builder".to_string(),
                description: "Complete all tutorials".to_string(),
                icon: "master_icon".to_string(),
                unlocked: false,
                unlock_date: None,
                rarity: AchievementRarity::Legendary,
            },
        ]
    }

    /// Start a tutorial
    pub fn start_tutorial(&mut self, tutorial_index: usize) {
        if tutorial_index < self.tutorials.len() {
            self.current_tutorial = Some(tutorial_index);
            self.current_step = 0;
            self.tutorial_overlay.visible = true;

            // Update overlay with first step
            if let Some(tutorial) = self.tutorials.get(tutorial_index) {
                if let Some(step) = tutorial.steps.first() {
                    self.update_step_display(step);
                }
            }
        }
    }

    /// Update step display
    fn update_step_display(&mut self, step: &TutorialStep) {
        self.tutorial_overlay.current_step_display = StepDisplay {
            title: step.title.clone(),
            description: step.description.clone(),
            objectives: vec![format!("{:?}", step.objective)],
            hints_shown: 0,
            time_elapsed: Duration::from_secs(0),
        };

        // Add objective markers
        if let Some(target) = &step.target_structure {
            for (pos, _) in target {
                self.tutorial_overlay.objective_markers.push(ObjectiveMarker {
                    world_position: Vector3::new(pos.x as f32, pos.y as f32, pos.z as f32),
                    screen_position: None,
                    label: "Target".to_string(),
                    color: UIColor::new(0.0, 1.0, 0.0, 0.5),
                    pulse_animation: true,
                });
            }
        }
    }

    /// Update playground state
    pub fn update(&mut self, delta_time: f32) {
        // Update tutorial progress
        if self.tutorial_overlay.visible {
            self.tutorial_overlay.current_step_display.time_elapsed += Duration::from_secs_f32(delta_time);
            self.check_tutorial_completion();
        }

        // Update statistics
        self.progress_tracker.statistics.play_time += Duration::from_secs_f32(delta_time);

        // Auto-save
        if self.auto_save && self.last_save_time.elapsed() > Duration::from_secs(30) {
            self.save_progress();
            self.last_save_time = Instant::now();
        }

        // Update hint bubbles
        self.update_hint_bubbles(delta_time);
    }

    /// Check if current tutorial step is complete
    fn check_tutorial_completion(&mut self) {
        // Implementation would check completion criteria
        // This is a placeholder
    }

    /// Update hint bubbles
    fn update_hint_bubbles(&mut self, delta_time: f32) {
        // Remove expired hints
        self.tutorial_overlay.hint_bubbles.retain(|hint| {
            if let Some(dismiss_time) = hint.dismiss_time {
                dismiss_time > Duration::from_secs_f32(delta_time)
            } else {
                true
            }
        });
    }

    /// Save progress
    fn save_progress(&mut self) {
        // Save tutorial progress and statistics
        // This would normally save to disk
    }

    /// Place a block in sandbox
    pub fn place_block(&mut self, position: Vector3<i32>, voxel_type: VoxelType) {
        // Record for undo
        let previous = self.sandbox_world.get_voxel(Vector3::new(
            position.x as f32,
            position.y as f32,
            position.z as f32,
        ));

        self.undo_history.push(WorldSnapshot {
            timestamp: Instant::now(),
            voxel_changes: vec![(position, previous, Some(voxel_type))],
            description: format!("Place {:?}", voxel_type),
        });

        // Limit history size
        if self.undo_history.len() > self.max_history_size {
            self.undo_history.remove(0);
        }

        // Clear redo history on new action
        self.redo_history.clear();

        // Place the block
        self.sandbox_world.set_voxel(
            Vector3::new(position.x as f32, position.y as f32, position.z as f32),
            voxel_type,
        );

        // Update statistics
        self.progress_tracker.statistics.total_blocks_placed += 1;
    }

    /// Undo last action
    pub fn undo(&mut self) {
        if let Some(snapshot) = self.undo_history.pop() {
            // Apply reverse changes
            for (pos, old_type, _) in &snapshot.voxel_changes {
                if let Some(voxel_type) = old_type {
                    self.sandbox_world.set_voxel(
                        Vector3::new(pos.x as f32, pos.y as f32, pos.z as f32),
                        *voxel_type,
                    );
                } else {
                    self.sandbox_world.remove_voxel(
                        Vector3::new(pos.x as f32, pos.y as f32, pos.z as f32),
                    );
                }
            }

            // Add to redo history
            self.redo_history.push(snapshot);
        }
    }

    /// Redo last undone action
    pub fn redo(&mut self) {
        if let Some(snapshot) = self.redo_history.pop() {
            // Apply changes
            for (pos, _, new_type) in &snapshot.voxel_changes {
                if let Some(voxel_type) = new_type {
                    self.sandbox_world.set_voxel(
                        Vector3::new(pos.x as f32, pos.y as f32, pos.z as f32),
                        *voxel_type,
                    );
                } else {
                    self.sandbox_world.remove_voxel(
                        Vector3::new(pos.x as f32, pos.y as f32, pos.z as f32),
                    );
                }
            }

            // Add back to undo history
            self.undo_history.push(snapshot);
        }
    }

    /// Apply a quick template
    pub fn apply_template(&mut self, template_index: usize, position: Vector3<i32>) {
        if let Some(template) = self.quick_templates.get(template_index) {
            let mut changes = Vec::new();

            for (offset, voxel_type) in &template.voxel_data {
                let world_pos = position + offset;
                let previous = self.sandbox_world.get_voxel(Vector3::new(
                    world_pos.x as f32,
                    world_pos.y as f32,
                    world_pos.z as f32,
                ));

                changes.push((world_pos, previous, Some(*voxel_type)));

                self.sandbox_world.set_voxel(
                    Vector3::new(world_pos.x as f32, world_pos.y as f32, world_pos.z as f32),
                    *voxel_type,
                );
            }

            // Record for undo
            self.undo_history.push(WorldSnapshot {
                timestamp: Instant::now(),
                voxel_changes: changes,
                description: format!("Apply template: {}", template.name),
            });

            // Update statistics
            self.progress_tracker.statistics.templates_used += 1;
        }
    }

    /// Get current world for rendering
    pub fn get_world(&self) -> &VoxelWorld {
        &self.sandbox_world
    }

    /// Check if hints are enabled
    pub fn hints_enabled(&self) -> bool {
        self.hints_enabled
    }

    /// Toggle creative mode
    pub fn toggle_creative_mode(&mut self) {
        self.creative_mode = !self.creative_mode;

        // Unlock all materials and tools in creative mode
        if self.creative_mode {
            for slot in &mut self.material_palette.materials {
                slot.locked = false;
                slot.count = None;
            }

            for slot in &mut self.tool_selector.tools {
                slot.unlocked = true;
            }
        }
    }
}