use crate::engine::{
    error::RobinResult,
    input::InputManager,
    math::{Vec2, Vec3},
    ui::modern_interface::{ModernUISystem, UITheme, ComponentLibrary, Color, Rectangle, TextStyle, AnimationState},
    build_mode::{
        tools::{BuildTool, ToolType, ToolKit},
        interactive_elements::ElementType,
    },
};
use winit::event::MouseButton;
use std::collections::HashMap;

pub struct ToolPalette {
    modern_ui: ModernUISystem,
    tool_groups: Vec<ToolGroup>,
    selected_tool: Option<ToolType>,
    selected_group: usize,
    palette_position: Vec2,
    palette_size: Vec2,
    is_visible: bool,
    is_expanded: bool,
    hover_state: HoverState,
    animation_state: PaletteAnimationState,
    context_menu: ContextMenu,
    search_bar: ToolSearchBar,
    favorites: FavoriteTools,
    quick_access: QuickAccessBar,
}

#[derive(Debug, Clone)]
pub struct ToolGroup {
    pub name: String,
    pub icon: ToolIcon,
    pub tools: Vec<ToolDefinition>,
    pub expanded: bool,
    pub color_theme: Color,
}

#[derive(Debug, Clone)]
pub struct ToolDefinition {
    pub tool_type: ToolType,
    pub name: String,
    pub description: String,
    pub icon: ToolIcon,
    pub keyboard_shortcut: Option<String>,
    pub usage_count: u32,
    pub is_favorite: bool,
    pub preview_data: ToolPreviewData,
}

#[derive(Debug, Clone)]
pub struct ToolIcon {
    pub icon_type: IconType,
    pub color: Color,
    pub size: f32,
    pub background_color: Option<Color>,
}

#[derive(Debug, Clone)]
pub enum IconType {
    // Construction Tools
    Terrain(TerrainIcon),
    Structure(StructureIcon),
    Interactive(InteractiveIcon),
    Decoration(DecorationIcon),

    // System Tools
    Camera(CameraIcon),
    Physics(PhysicsIcon),
    Lighting(LightingIcon),
    Audio(AudioIcon),

    // Advanced Tools
    Scripting(ScriptingIcon),
    AI(AIIcon),
    Animation(AnimationIcon),
    Particle(ParticleIcon),
}

#[derive(Debug, Clone)]
pub enum TerrainIcon {
    Voxel,
    Sculpt,
    Paint,
    Texture,
    Height,
    Erosion,
    Vegetation,
}

#[derive(Debug, Clone)]
pub enum StructureIcon {
    Wall,
    Floor,
    Door,
    Window,
    Stairs,
    Platform,
    Bridge,
    Building,
}

#[derive(Debug, Clone)]
pub enum InteractiveIcon {
    Switch,
    Button,
    Trigger,
    Teleporter,
    Collectible,
    Vehicle,
    NPC,
    Questgiver,
}

#[derive(Debug, Clone)]
pub enum DecorationIcon {
    Prop,
    Plant,
    Rock,
    Tree,
    Furniture,
    Effect,
    Skybox,
}

#[derive(Debug, Clone)]
pub enum CameraIcon {
    Perspective,
    Orthographic,
    Cinematic,
    Security,
}

#[derive(Debug, Clone)]
pub enum PhysicsIcon {
    Gravity,
    Collision,
    Joint,
    Force,
}

#[derive(Debug, Clone)]
pub enum LightingIcon {
    Sun,
    Point,
    Spot,
    Area,
    Ambient,
}

#[derive(Debug, Clone)]
pub enum AudioIcon {
    Source,
    Zone,
    Reverb,
    Music,
}

#[derive(Debug, Clone)]
pub enum ScriptingIcon {
    Logic,
    Event,
    Condition,
    Action,
}

#[derive(Debug, Clone)]
pub enum AIIcon {
    Pathfinding,
    Behavior,
    State,
    Decision,
}

#[derive(Debug, Clone)]
pub enum AnimationIcon {
    Keyframe,
    Timeline,
    Bone,
    Morph,
}

#[derive(Debug, Clone)]
pub enum ParticleIcon {
    Emitter,
    Fire,
    Smoke,
    Magic,
}

#[derive(Debug, Clone)]
pub struct ToolPreviewData {
    pub thumbnail: Option<Vec<u8>>,
    pub preview_mesh: Option<PreviewMesh>,
    pub material_preview: Option<MaterialPreview>,
    pub animation_preview: Option<AnimationPreview>,
}

#[derive(Debug, Clone)]
pub struct PreviewMesh {
    pub vertices: Vec<Vec3>,
    pub indices: Vec<u32>,
    pub normals: Vec<Vec3>,
}

#[derive(Debug, Clone)]
pub struct MaterialPreview {
    pub diffuse_color: Color,
    pub metallic: f32,
    pub roughness: f32,
    pub emission: Color,
}

#[derive(Debug, Clone)]
pub struct AnimationPreview {
    pub frame_count: u32,
    pub duration: f32,
    pub loop_type: AnimationLoopType,
}

#[derive(Debug, Clone)]
pub enum AnimationLoopType {
    Once,
    Loop,
    PingPong,
}

#[derive(Debug, Clone)]
pub struct HoverState {
    pub hovered_tool: Option<ToolType>,
    pub hover_time: f32,
    pub tooltip_visible: bool,
    pub preview_visible: bool,
}

#[derive(Debug, Clone)]
pub struct PaletteAnimationState {
    pub expand_progress: f32,
    pub slide_offset: Vec2,
    pub fade_alpha: f32,
    pub tool_animations: HashMap<ToolType, ToolAnimationState>,
}

#[derive(Debug, Clone)]
pub struct ToolAnimationState {
    pub scale: f32,
    pub rotation: f32,
    pub glow_intensity: f32,
    pub bounce_offset: f32,
}

pub struct ContextMenu {
    position: Vec2,
    size: Vec2,
    visible: bool,
    items: Vec<ContextMenuItem>,
    selected_index: Option<usize>,
    submenu: Option<Box<ContextMenu>>,
    animation_state: f32,
}

#[derive(Debug, Clone)]
pub struct ContextMenuItem {
    pub text: String,
    pub icon: Option<ToolIcon>,
    pub action: ContextAction,
    pub enabled: bool,
    pub separator_after: bool,
    pub submenu_items: Option<Vec<ContextMenuItem>>,
}

#[derive(Debug, Clone)]
pub enum ContextAction {
    SelectTool(ToolType),
    AddToFavorites(ToolType),
    RemoveFromFavorites(ToolType),
    CreateCustomTool,
    DuplicateTool(ToolType),
    EditToolProperties(ToolType),
    ShowToolDocumentation(ToolType),
    ExportTool(ToolType),
    ImportTool,
    ResetToolSettings(ToolType),
    CreateToolGroup,
    ManageToolGroups,
    TogglePaletteMode,
    CustomizeInterface,
}

pub struct ToolSearchBar {
    query: String,
    visible: bool,
    results: Vec<ToolDefinition>,
    selected_result: Option<usize>,
    search_history: Vec<String>,
    auto_complete: Vec<String>,
}

pub struct FavoriteTools {
    tools: Vec<ToolType>,
    max_favorites: usize,
    quick_select_mode: bool,
}

pub struct QuickAccessBar {
    tools: Vec<ToolType>,
    position: Vec2,
    size: Vec2,
    visible: bool,
    horizontal_layout: bool,
}

impl ToolPalette {
    pub fn new(modern_ui: ModernUISystem) -> Self {
        let mut palette = Self {
            modern_ui,
            tool_groups: Vec::new(),
            selected_tool: None,
            selected_group: 0,
            palette_position: Vec2::new(20.0, 100.0),
            palette_size: Vec2::new(280.0, 600.0),
            is_visible: true,
            is_expanded: true,
            hover_state: HoverState {
                hovered_tool: None,
                hover_time: 0.0,
                tooltip_visible: false,
                preview_visible: false,
            },
            animation_state: PaletteAnimationState {
                expand_progress: 1.0,
                slide_offset: Vec2::new(0.0, 0.0),
                fade_alpha: 1.0,
                tool_animations: HashMap::new(),
            },
            context_menu: ContextMenu::new(),
            search_bar: ToolSearchBar::new(),
            favorites: FavoriteTools::new(),
            quick_access: QuickAccessBar::new(),
        };

        palette.initialize_default_tools();
        palette
    }

    fn initialize_default_tools(&mut self) {
        // Terrain Tools Group
        self.tool_groups.push(ToolGroup {
            name: "Terrain".to_string(),
            icon: ToolIcon {
                icon_type: IconType::Terrain(TerrainIcon::Voxel),
                color: Color::new(0.4, 0.8, 0.4, 1.0),
                size: 24.0,
                background_color: Some(Color::new(0.2, 0.4, 0.2, 0.8)),
            },
            tools: vec![
                ToolDefinition {
                    tool_type: ToolType::VoxelBrush,
                    name: "Voxel Brush".to_string(),
                    description: "Add and remove voxel terrain blocks with adjustable brush size".to_string(),
                    icon: ToolIcon {
                        icon_type: IconType::Terrain(TerrainIcon::Voxel),
                        color: Color::new(0.5, 0.9, 0.5, 1.0),
                        size: 20.0,
                        background_color: None,
                    },
                    keyboard_shortcut: Some("V".to_string()),
                    usage_count: 45,
                    is_favorite: true,
                    preview_data: ToolPreviewData::default(),
                },
                ToolDefinition {
                    tool_type: ToolType::TerrainSculpt,
                    name: "Terrain Sculpt".to_string(),
                    description: "Sculpt smooth terrain surfaces with natural erosion effects".to_string(),
                    icon: ToolIcon {
                        icon_type: IconType::Terrain(TerrainIcon::Sculpt),
                        color: Color::new(0.8, 0.6, 0.4, 1.0),
                        size: 20.0,
                        background_color: None,
                    },
                    keyboard_shortcut: Some("S".to_string()),
                    usage_count: 23,
                    is_favorite: false,
                    preview_data: ToolPreviewData::default(),
                },
                ToolDefinition {
                    tool_type: ToolType::TerrainPaint,
                    name: "Terrain Paint".to_string(),
                    description: "Paint textures and materials onto terrain surfaces".to_string(),
                    icon: ToolIcon {
                        icon_type: IconType::Terrain(TerrainIcon::Paint),
                        color: Color::new(0.9, 0.5, 0.7, 1.0),
                        size: 20.0,
                        background_color: None,
                    },
                    keyboard_shortcut: Some("P".to_string()),
                    usage_count: 18,
                    is_favorite: false,
                    preview_data: ToolPreviewData::default(),
                },
            ],
            expanded: true,
            color_theme: Color::new(0.4, 0.8, 0.4, 1.0),
        });

        // Structure Tools Group
        self.tool_groups.push(ToolGroup {
            name: "Structures".to_string(),
            icon: ToolIcon {
                icon_type: IconType::Structure(StructureIcon::Building),
                color: Color::new(0.6, 0.6, 0.9, 1.0),
                size: 24.0,
                background_color: Some(Color::new(0.3, 0.3, 0.5, 0.8)),
            },
            tools: vec![
                ToolDefinition {
                    tool_type: ToolType::StructurePlacer,
                    name: "Structure Placer".to_string(),
                    description: "Place pre-built structures like buildings, bridges, and platforms".to_string(),
                    icon: ToolIcon {
                        icon_type: IconType::Structure(StructureIcon::Building),
                        color: Color::new(0.7, 0.7, 1.0, 1.0),
                        size: 20.0,
                        background_color: None,
                    },
                    keyboard_shortcut: Some("B".to_string()),
                    usage_count: 31,
                    is_favorite: true,
                    preview_data: ToolPreviewData::default(),
                },
                ToolDefinition {
                    tool_type: ToolType::WallBuilder,
                    name: "Wall Builder".to_string(),
                    description: "Build walls, fences, and barriers with automatic connection".to_string(),
                    icon: ToolIcon {
                        icon_type: IconType::Structure(StructureIcon::Wall),
                        color: Color::new(0.8, 0.8, 0.6, 1.0),
                        size: 20.0,
                        background_color: None,
                    },
                    keyboard_shortcut: Some("W".to_string()),
                    usage_count: 28,
                    is_favorite: false,
                    preview_data: ToolPreviewData::default(),
                },
            ],
            expanded: true,
            color_theme: Color::new(0.6, 0.6, 0.9, 1.0),
        });

        // Interactive Elements Group
        self.tool_groups.push(ToolGroup {
            name: "Interactive".to_string(),
            icon: ToolIcon {
                icon_type: IconType::Interactive(InteractiveIcon::Switch),
                color: Color::new(0.9, 0.7, 0.4, 1.0),
                size: 24.0,
                background_color: Some(Color::new(0.5, 0.4, 0.2, 0.8)),
            },
            tools: vec![
                ToolDefinition {
                    tool_type: ToolType::ElementPlacer,
                    name: "Interactive Placer".to_string(),
                    description: "Place switches, buttons, doors, and other interactive elements".to_string(),
                    icon: ToolIcon {
                        icon_type: IconType::Interactive(InteractiveIcon::Switch),
                        color: Color::new(1.0, 0.8, 0.5, 1.0),
                        size: 20.0,
                        background_color: None,
                    },
                    keyboard_shortcut: Some("I".to_string()),
                    usage_count: 22,
                    is_favorite: true,
                    preview_data: ToolPreviewData::default(),
                },
                ToolDefinition {
                    tool_type: ToolType::NPCPlacer,
                    name: "NPC Placer".to_string(),
                    description: "Place non-player characters with AI behaviors and dialogue".to_string(),
                    icon: ToolIcon {
                        icon_type: IconType::Interactive(InteractiveIcon::NPC),
                        color: Color::new(0.8, 0.9, 0.6, 1.0),
                        size: 20.0,
                        background_color: None,
                    },
                    keyboard_shortcut: Some("N".to_string()),
                    usage_count: 15,
                    is_favorite: false,
                    preview_data: ToolPreviewData::default(),
                },
            ],
            expanded: false,
            color_theme: Color::new(0.9, 0.7, 0.4, 1.0),
        });

        // Initialize favorites
        self.favorites.tools = vec![
            ToolType::VoxelBrush,
            ToolType::StructurePlacer,
            ToolType::ElementPlacer,
        ];

        // Initialize quick access
        self.quick_access.tools = vec![
            ToolType::VoxelBrush,
            ToolType::StructurePlacer,
            ToolType::ElementPlacer,
            ToolType::TerrainSculpt,
        ];
    }

    pub fn update(&mut self, delta_time: f32, input: &InputManager) -> RobinResult<()> {
        // Update animations
        self.update_animations(delta_time);

        // Handle input
        self.handle_input(input)?;

        // Update hover state
        self.update_hover_state(delta_time, input);

        // Update context menu
        self.context_menu.update(delta_time, input);

        // Update search bar
        self.search_bar.update(input);

        Ok(())
    }

    fn update_animations(&mut self, delta_time: f32) {
        // Update expand/collapse animation
        let target_progress = if self.is_expanded { 1.0 } else { 0.0 };
        let animation_speed = 4.0;
        self.animation_state.expand_progress = lerp(
            self.animation_state.expand_progress,
            target_progress,
            animation_speed * delta_time,
        );

        // Update tool animations
        for (tool_type, animation) in &mut self.animation_state.tool_animations {
            let is_selected = self.selected_tool == Some(*tool_type);
            let is_hovered = self.hover_state.hovered_tool == Some(*tool_type);

            // Scale animation
            let target_scale = if is_selected { 1.1 } else if is_hovered { 1.05 } else { 1.0 };
            animation.scale = lerp(animation.scale, target_scale, 8.0 * delta_time);

            // Glow animation for selected tool
            let target_glow = if is_selected { 0.8 } else { 0.0 };
            animation.glow_intensity = lerp(animation.glow_intensity, target_glow, 6.0 * delta_time);

            // Bounce animation for interactions
            if animation.bounce_offset > 0.0 {
                animation.bounce_offset = (animation.bounce_offset - 4.0 * delta_time).max(0.0);
            }
        }
    }

    fn handle_input(&mut self, input: &InputManager) -> RobinResult<()> {
        // Toggle palette visibility
        if input.is_named_key_just_pressed(winit::keyboard::NamedKey::Tab) {
            self.toggle_visibility();
        }

        // Toggle expanded state
        if input.is_key_just_pressed(&winit::keyboard::Key::Character("t".into())) {
            self.toggle_expanded();
        }

        // Handle mouse interactions
        if self.is_visible {
            let mouse_pos = Vec2::new(input.mouse_position().0, input.mouse_position().1);

            if input.is_mouse_button_just_pressed(MouseButton::Left) {
                self.handle_mouse_click(mouse_pos)?;
            }

            if input.is_mouse_button_just_pressed(MouseButton::Right) {
                self.show_context_menu(mouse_pos);
            }

            // Handle keyboard shortcuts for tool selection
            self.handle_tool_shortcuts(input)?;
        }

        Ok(())
    }

    fn handle_mouse_click(&mut self, mouse_pos: Vec2) -> RobinResult<()> {
        // Check if click is within palette bounds
        let palette_rect = Rectangle {
            x: self.palette_position.x,
            y: self.palette_position.y,
            width: self.palette_size.x * self.animation_state.expand_progress,
            height: self.palette_size.y,
        };

        if !self.point_in_rect(mouse_pos, &palette_rect) {
            return Ok(());
        }

        // Check tool group headers
        let mut current_y = self.palette_position.y + 40.0;
        for (group_index, group) in self.tool_groups.iter_mut().enumerate() {
            let group_rect = Rectangle {
                x: self.palette_position.x + 10.0,
                y: current_y,
                width: self.palette_size.x - 20.0,
                height: 30.0,
            };

            if self.point_in_rect(mouse_pos, &group_rect) {
                group.expanded = !group.expanded;
                self.selected_group = group_index;
                return Ok(());
            }

            current_y += 35.0;

            // Check individual tools if group is expanded
            if group.expanded {
                for tool in &group.tools {
                    let tool_rect = Rectangle {
                        x: self.palette_position.x + 20.0,
                        y: current_y,
                        width: self.palette_size.x - 40.0,
                        height: 28.0,
                    };

                    if self.point_in_rect(mouse_pos, &tool_rect) {
                        self.select_tool(tool.tool_type);
                        return Ok(());
                    }

                    current_y += 32.0;
                }
            }
        }

        Ok(())
    }

    fn handle_tool_shortcuts(&mut self, input: &InputManager) -> RobinResult<()> {
        for group in &self.tool_groups {
            for tool in &group.tools {
                if let Some(shortcut) = &tool.keyboard_shortcut {
                    let key = winit::keyboard::Key::Character(shortcut.to_lowercase().into());
                    if input.is_key_just_pressed(&key) {
                        self.select_tool(tool.tool_type);
                        return Ok(());
                    }
                }
            }
        }

        // Quick access shortcuts (F1-F12)
        for (index, tool_type) in self.quick_access.tools.iter().enumerate() {
            if index < 12 {
                let f_key = match index + 1 {
                    1 => winit::keyboard::NamedKey::F1,
                    2 => winit::keyboard::NamedKey::F2,
                    3 => winit::keyboard::NamedKey::F3,
                    4 => winit::keyboard::NamedKey::F4,
                    5 => winit::keyboard::NamedKey::F5,
                    6 => winit::keyboard::NamedKey::F6,
                    7 => winit::keyboard::NamedKey::F7,
                    8 => winit::keyboard::NamedKey::F8,
                    9 => winit::keyboard::NamedKey::F9,
                    10 => winit::keyboard::NamedKey::F10,
                    11 => winit::keyboard::NamedKey::F11,
                    12 => winit::keyboard::NamedKey::F12,
                    _ => continue,
                };

                if input.is_named_key_just_pressed(f_key) {
                    self.select_tool(*tool_type);
                    return Ok(());
                }
            }
        }

        Ok(())
    }

    fn update_hover_state(&mut self, delta_time: f32, input: &InputManager) {
        let mouse_pos = Vec2::new(input.mouse_position().0, input.mouse_position().1);
        let mut hovered_tool = None;

        // Check which tool is being hovered
        let mut current_y = self.palette_position.y + 40.0;
        for group in &self.tool_groups {
            current_y += 35.0;
            if group.expanded {
                for tool in &group.tools {
                    let tool_rect = Rectangle {
                        x: self.palette_position.x + 20.0,
                        y: current_y,
                        width: self.palette_size.x - 40.0,
                        height: 28.0,
                    };

                    if self.point_in_rect(mouse_pos, &tool_rect) {
                        hovered_tool = Some(tool.tool_type);
                        break;
                    }

                    current_y += 32.0;
                }
            }
        }

        // Update hover state
        if hovered_tool == self.hover_state.hovered_tool {
            self.hover_state.hover_time += delta_time;
        } else {
            self.hover_state.hovered_tool = hovered_tool;
            self.hover_state.hover_time = 0.0;
            self.hover_state.tooltip_visible = false;
            self.hover_state.preview_visible = false;
        }

        // Show tooltip after hover delay
        if self.hover_state.hover_time > 0.8 {
            self.hover_state.tooltip_visible = true;
        }

        // Show preview after longer delay
        if self.hover_state.hover_time > 1.5 {
            self.hover_state.preview_visible = true;
        }
    }

    fn show_context_menu(&mut self, position: Vec2) {
        self.context_menu.position = position;
        self.context_menu.visible = true;
        self.context_menu.animation_state = 0.0;

        // Build context menu items based on what was clicked
        self.context_menu.items.clear();

        if let Some(tool_type) = self.hover_state.hovered_tool {
            // Tool-specific context menu
            self.context_menu.items.push(ContextMenuItem {
                text: "Select Tool".to_string(),
                icon: None,
                action: ContextAction::SelectTool(tool_type),
                enabled: true,
                separator_after: false,
                submenu_items: None,
            });

            let is_favorite = self.favorites.tools.contains(&tool_type);
            if is_favorite {
                self.context_menu.items.push(ContextMenuItem {
                    text: "Remove from Favorites".to_string(),
                    icon: None,
                    action: ContextAction::RemoveFromFavorites(tool_type),
                    enabled: true,
                    separator_after: false,
                    submenu_items: None,
                });
            } else {
                self.context_menu.items.push(ContextMenuItem {
                    text: "Add to Favorites".to_string(),
                    icon: None,
                    action: ContextAction::AddToFavorites(tool_type),
                    enabled: true,
                    separator_after: false,
                    submenu_items: None,
                });
            }

            self.context_menu.items.push(ContextMenuItem {
                text: "Tool Properties".to_string(),
                icon: None,
                action: ContextAction::EditToolProperties(tool_type),
                enabled: true,
                separator_after: true,
                submenu_items: None,
            });
        }

        // General palette options
        self.context_menu.items.push(ContextMenuItem {
            text: "Customize Interface".to_string(),
            icon: None,
            action: ContextAction::CustomizeInterface,
            enabled: true,
            separator_after: false,
            submenu_items: None,
        });

        self.context_menu.items.push(ContextMenuItem {
            text: "Manage Tool Groups".to_string(),
            icon: None,
            action: ContextAction::ManageToolGroups,
            enabled: true,
            separator_after: false,
            submenu_items: None,
        });
    }

    pub fn select_tool(&mut self, tool_type: ToolType) {
        self.selected_tool = Some(tool_type);

        // Update usage count
        for group in &mut self.tool_groups {
            for tool in &mut group.tools {
                if tool.tool_type == tool_type {
                    tool.usage_count += 1;
                    break;
                }
            }
        }

        // Add bounce animation
        if !self.animation_state.tool_animations.contains_key(&tool_type) {
            self.animation_state.tool_animations.insert(
                tool_type,
                ToolAnimationState {
                    scale: 1.0,
                    rotation: 0.0,
                    glow_intensity: 0.0,
                    bounce_offset: 0.0,
                },
            );
        }

        if let Some(animation) = self.animation_state.tool_animations.get_mut(&tool_type) {
            animation.bounce_offset = 1.0;
        }

        println!("Selected tool: {:?}", tool_type);
    }

    pub fn toggle_visibility(&mut self) {
        self.is_visible = !self.is_visible;
        if self.is_visible {
            self.animation_state.fade_alpha = 0.0;
        }
    }

    pub fn toggle_expanded(&mut self) {
        self.is_expanded = !self.is_expanded;
    }

    pub fn get_selected_tool(&self) -> Option<ToolType> {
        self.selected_tool
    }

    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    pub fn render(&self, renderer: &mut dyn Renderer) -> RobinResult<()> {
        if !self.is_visible || self.animation_state.fade_alpha <= 0.01 {
            return Ok(());
        }

        // Render main palette background
        let palette_rect = Rectangle {
            x: self.palette_position.x,
            y: self.palette_position.y,
            width: self.palette_size.x * self.animation_state.expand_progress,
            height: self.palette_size.y,
        };

        self.modern_ui.render_panel(&palette_rect, renderer)?;

        // Render palette header
        self.render_header(renderer)?;

        // Render tool groups
        self.render_tool_groups(renderer)?;

        // Render search bar if visible
        if self.search_bar.visible {
            self.search_bar.render(renderer)?;
        }

        // Render quick access bar
        self.quick_access.render(renderer)?;

        // Render context menu
        if self.context_menu.visible {
            self.context_menu.render(renderer)?;
        }

        // Render tooltips
        self.render_tooltips(renderer)?;

        Ok(())
    }

    fn render_header(&self, renderer: &mut dyn Renderer) -> RobinResult<()> {
        let header_rect = Rectangle {
            x: self.palette_position.x + 10.0,
            y: self.palette_position.y + 10.0,
            width: self.palette_size.x - 20.0,
            height: 25.0,
        };

        // Render title
        let title_style = TextStyle {
            font_family: "Inter".to_string(),
            font_size: 14.0,
            font_weight: 600,
            color: self.modern_ui.get_theme().text.primary,
            line_height: 1.2,
        };

        renderer.render_text("Tool Palette", &header_rect, &title_style)?;

        // Render expand/collapse button
        let button_rect = Rectangle {
            x: self.palette_position.x + self.palette_size.x - 30.0,
            y: self.palette_position.y + 12.0,
            width: 20.0,
            height: 20.0,
        };

        let expand_icon = if self.is_expanded { "−" } else { "+" };
        renderer.render_text(expand_icon, &button_rect, &title_style)?;

        Ok(())
    }

    fn render_tool_groups(&self, renderer: &mut dyn Renderer) -> RobinResult<()> {
        let mut current_y = self.palette_position.y + 45.0;

        for (group_index, group) in self.tool_groups.iter().enumerate() {
            // Render group header
            let group_rect = Rectangle {
                x: self.palette_position.x + 10.0,
                y: current_y,
                width: self.palette_size.x - 20.0,
                height: 30.0,
            };

            let is_selected = group_index == self.selected_group;
            self.render_group_header(group, &group_rect, is_selected, renderer)?;

            current_y += 35.0;

            // Render tools if expanded
            if group.expanded {
                for tool in &group.tools {
                    let tool_rect = Rectangle {
                        x: self.palette_position.x + 20.0,
                        y: current_y,
                        width: self.palette_size.x - 40.0,
                        height: 28.0,
                    };

                    let is_selected = self.selected_tool == Some(tool.tool_type);
                    let is_hovered = self.hover_state.hovered_tool == Some(tool.tool_type);
                    self.render_tool_item(tool, &tool_rect, is_selected, is_hovered, renderer)?;

                    current_y += 32.0;
                }
            }
        }

        Ok(())
    }

    fn render_group_header(
        &self,
        group: &ToolGroup,
        rect: &Rectangle,
        is_selected: bool,
        renderer: &mut dyn Renderer,
    ) -> RobinResult<()> {
        // Background
        let bg_color = if is_selected {
            group.color_theme.with_alpha(0.3)
        } else {
            self.modern_ui.get_theme().surface.secondary
        };

        renderer.fill_rect(rect, &bg_color)?;

        // Group icon
        let icon_rect = Rectangle {
            x: rect.x + 5.0,
            y: rect.y + 5.0,
            width: 20.0,
            height: 20.0,
        };

        self.render_tool_icon(&group.icon, &icon_rect, renderer)?;

        // Group name
        let text_rect = Rectangle {
            x: rect.x + 30.0,
            y: rect.y + 5.0,
            width: rect.width - 50.0,
            height: 20.0,
        };

        let text_style = TextStyle {
            font_family: "Inter".to_string(),
            font_size: 12.0,
            font_weight: 500,
            color: self.modern_ui.get_theme().text.primary,
            line_height: 1.2,
        };

        renderer.render_text(&group.name, &text_rect, &text_style)?;

        // Expand/collapse indicator
        let indicator_rect = Rectangle {
            x: rect.x + rect.width - 20.0,
            y: rect.y + 8.0,
            width: 15.0,
            height: 15.0,
        };

        let indicator = if group.expanded { "▼" } else { "▶" };
        renderer.render_text(indicator, &indicator_rect, &text_style)?;

        Ok(())
    }

    fn render_tool_item(
        &self,
        tool: &ToolDefinition,
        rect: &Rectangle,
        is_selected: bool,
        is_hovered: bool,
        renderer: &mut dyn Renderer,
    ) -> RobinResult<()> {
        // Get animation state
        let animation = self.animation_state.tool_animations.get(&tool.tool_type);
        let scale = animation.map_or(1.0, |a| a.scale);
        let glow_intensity = animation.map_or(0.0, |a| a.glow_intensity);

        // Background with selection/hover state
        let bg_color = if is_selected {
            self.modern_ui.get_theme().interactive.selected
        } else if is_hovered {
            self.modern_ui.get_theme().interactive.hover
        } else {
            Color::transparent()
        };

        // Apply scaling for animation
        let scaled_rect = Rectangle {
            x: rect.x + (rect.width * (1.0 - scale)) * 0.5,
            y: rect.y + (rect.height * (1.0 - scale)) * 0.5,
            width: rect.width * scale,
            height: rect.height * scale,
        };

        renderer.fill_rect(&scaled_rect, &bg_color)?;

        // Glow effect for selected tools
        if glow_intensity > 0.0 {
            let glow_color = Color::new(0.4, 0.7, 1.0, glow_intensity * 0.5);
            let glow_rect = Rectangle {
                x: scaled_rect.x - 2.0,
                y: scaled_rect.y - 2.0,
                width: scaled_rect.width + 4.0,
                height: scaled_rect.height + 4.0,
            };
            renderer.stroke_rect(&glow_rect, &glow_color, 2.0)?;
        }

        // Tool icon
        let icon_rect = Rectangle {
            x: scaled_rect.x + 4.0,
            y: scaled_rect.y + 4.0,
            width: 20.0,
            height: 20.0,
        };

        self.render_tool_icon(&tool.icon, &icon_rect, renderer)?;

        // Tool name
        let text_rect = Rectangle {
            x: scaled_rect.x + 28.0,
            y: scaled_rect.y + 4.0,
            width: scaled_rect.width - 60.0,
            height: 20.0,
        };

        let text_style = TextStyle {
            font_family: "Inter".to_string(),
            font_size: 11.0,
            font_weight: if is_selected { 500 } else { 400 },
            color: if is_selected {
                self.modern_ui.get_theme().text.primary
            } else {
                self.modern_ui.get_theme().text.secondary
            },
            line_height: 1.2,
        };

        renderer.render_text(&tool.name, &text_rect, &text_style)?;

        // Keyboard shortcut
        if let Some(shortcut) = &tool.keyboard_shortcut {
            let shortcut_rect = Rectangle {
                x: scaled_rect.x + scaled_rect.width - 25.0,
                y: scaled_rect.y + 6.0,
                width: 20.0,
                height: 16.0,
            };

            let shortcut_style = TextStyle {
                font_family: "JetBrains Mono".to_string(),
                font_size: 9.0,
                font_weight: 400,
                color: self.modern_ui.get_theme().text.tertiary,
                line_height: 1.0,
            };

            renderer.render_text(shortcut, &shortcut_rect, &shortcut_style)?;
        }

        // Favorite indicator
        if tool.is_favorite {
            let star_rect = Rectangle {
                x: scaled_rect.x + scaled_rect.width - 15.0,
                y: scaled_rect.y + 2.0,
                width: 12.0,
                height: 12.0,
            };

            let star_color = Color::new(1.0, 0.8, 0.2, 1.0);
            renderer.render_text("★", &star_rect, &TextStyle {
                font_family: "Inter".to_string(),
                font_size: 10.0,
                font_weight: 400,
                color: star_color,
                line_height: 1.0,
            })?;
        }

        Ok(())
    }

    fn render_tool_icon(&self, icon: &ToolIcon, rect: &Rectangle, renderer: &mut dyn Renderer) -> RobinResult<()> {
        // Render background if specified
        if let Some(bg_color) = &icon.background_color {
            renderer.fill_rect(rect, bg_color)?;
        }

        // Render icon based on type
        match &icon.icon_type {
            IconType::Terrain(terrain_icon) => {
                self.render_terrain_icon(terrain_icon, rect, &icon.color, renderer)?;
            }
            IconType::Structure(structure_icon) => {
                self.render_structure_icon(structure_icon, rect, &icon.color, renderer)?;
            }
            IconType::Interactive(interactive_icon) => {
                self.render_interactive_icon(interactive_icon, rect, &icon.color, renderer)?;
            }
            IconType::Decoration(decoration_icon) => {
                self.render_decoration_icon(decoration_icon, rect, &icon.color, renderer)?;
            }
            _ => {
                // Fallback: render a simple colored square
                renderer.fill_rect(rect, &icon.color)?;
            }
        }

        Ok(())
    }

    fn render_terrain_icon(&self, icon: &TerrainIcon, rect: &Rectangle, color: &Color, renderer: &mut dyn Renderer) -> RobinResult<()> {
        match icon {
            TerrainIcon::Voxel => {
                // Render a simple cube representation
                let cube_size = rect.width * 0.7;
                let cube_rect = Rectangle {
                    x: rect.x + (rect.width - cube_size) * 0.5,
                    y: rect.y + (rect.height - cube_size) * 0.5,
                    width: cube_size,
                    height: cube_size,
                };
                renderer.fill_rect(&cube_rect, color)?;
                renderer.stroke_rect(&cube_rect, &color.darken(0.3), 1.0)?;
            }
            TerrainIcon::Sculpt => {
                // Render wavy lines to represent sculpting
                for i in 0..3 {
                    let y_offset = rect.y + rect.height * 0.3 + (i as f32 * rect.height * 0.2);
                    // This would be a wavy line in a real implementation
                    renderer.draw_line(
                        Vec2::new(rect.x + rect.width * 0.2, y_offset),
                        Vec2::new(rect.x + rect.width * 0.8, y_offset),
                        color,
                        1.5,
                    )?;
                }
            }
            TerrainIcon::Paint => {
                // Render a paintbrush icon
                let brush_rect = Rectangle {
                    x: rect.x + rect.width * 0.3,
                    y: rect.y + rect.height * 0.2,
                    width: rect.width * 0.4,
                    height: rect.height * 0.6,
                };
                renderer.fill_rect(&brush_rect, color)?;
            }
            _ => {
                // Default fallback
                renderer.fill_rect(rect, color)?;
            }
        }

        Ok(())
    }

    fn render_structure_icon(&self, icon: &StructureIcon, rect: &Rectangle, color: &Color, renderer: &mut dyn Renderer) -> RobinResult<()> {
        match icon {
            StructureIcon::Building => {
                // Render a simple building silhouette
                let building_rect = Rectangle {
                    x: rect.x + rect.width * 0.2,
                    y: rect.y + rect.height * 0.3,
                    width: rect.width * 0.6,
                    height: rect.height * 0.7,
                };
                renderer.fill_rect(&building_rect, color)?;

                // Add some windows
                for i in 0..2 {
                    for j in 0..2 {
                        let window_rect = Rectangle {
                            x: building_rect.x + (i as f32 + 0.5) * building_rect.width * 0.3,
                            y: building_rect.y + (j as f32 + 0.5) * building_rect.height * 0.3,
                            width: building_rect.width * 0.15,
                            height: building_rect.height * 0.15,
                        };
                        renderer.fill_rect(&window_rect, &color.darken(0.4))?;
                    }
                }
            }
            StructureIcon::Wall => {
                // Render brick pattern
                let brick_height = rect.height * 0.15;
                let brick_width = rect.width * 0.3;

                for row in 0..5 {
                    let y = rect.y + row as f32 * brick_height;
                    let offset = if row % 2 == 0 { 0.0 } else { brick_width * 0.5 };

                    for col in 0..3 {
                        let x = rect.x + col as f32 * brick_width + offset;
                        if x + brick_width <= rect.x + rect.width {
                            let brick_rect = Rectangle {
                                x,
                                y,
                                width: brick_width - 1.0,
                                height: brick_height - 1.0,
                            };
                            renderer.fill_rect(&brick_rect, color)?;
                        }
                    }
                }
            }
            _ => {
                renderer.fill_rect(rect, color)?;
            }
        }

        Ok(())
    }

    fn render_interactive_icon(&self, icon: &InteractiveIcon, rect: &Rectangle, color: &Color, renderer: &mut dyn Renderer) -> RobinResult<()> {
        match icon {
            InteractiveIcon::Switch => {
                // Render a toggle switch
                let switch_rect = Rectangle {
                    x: rect.x + rect.width * 0.2,
                    y: rect.y + rect.height * 0.4,
                    width: rect.width * 0.6,
                    height: rect.height * 0.2,
                };
                renderer.stroke_rect(&switch_rect, color, 2.0)?;

                // Toggle indicator
                let indicator_rect = Rectangle {
                    x: switch_rect.x + switch_rect.width * 0.7,
                    y: switch_rect.y,
                    width: switch_rect.height,
                    height: switch_rect.height,
                };
                renderer.fill_rect(&indicator_rect, color)?;
            }
            InteractiveIcon::NPC => {
                // Render a simple person icon
                let head_radius = rect.width * 0.15;
                let head_center = Vec2::new(rect.x + rect.width * 0.5, rect.y + rect.height * 0.25);
                renderer.fill_circle(&head_center, head_radius, color)?;

                // Body
                let body_rect = Rectangle {
                    x: rect.x + rect.width * 0.35,
                    y: rect.y + rect.height * 0.4,
                    width: rect.width * 0.3,
                    height: rect.height * 0.5,
                };
                renderer.fill_rect(&body_rect, color)?;
            }
            _ => {
                renderer.fill_rect(rect, color)?;
            }
        }

        Ok(())
    }

    fn render_decoration_icon(&self, _icon: &DecorationIcon, rect: &Rectangle, color: &Color, renderer: &mut dyn Renderer) -> RobinResult<()> {
        // Simple fallback for decoration icons
        renderer.fill_rect(rect, color)?;
        Ok(())
    }

    fn render_tooltips(&self, renderer: &mut dyn Renderer) -> RobinResult<()> {
        if !self.hover_state.tooltip_visible {
            return Ok(());
        }

        if let Some(tool_type) = self.hover_state.hovered_tool {
            // Find the tool definition
            if let Some(tool) = self.find_tool_definition(tool_type) {
                // Calculate tooltip position
                let mouse_pos = Vec2::new(0.0, 0.0); // This would come from input manager
                let tooltip_pos = Vec2::new(mouse_pos.x + 15.0, mouse_pos.y - 10.0);

                self.render_tooltip(&tool.name, &tool.description, tooltip_pos, renderer)?;
            }
        }

        Ok(())
    }

    fn render_tooltip(&self, title: &str, description: &str, position: Vec2, renderer: &mut dyn Renderer) -> RobinResult<()> {
        // Calculate tooltip size
        let max_width = 250.0;
        let padding = 10.0;
        let line_height = 16.0;

        let title_height = line_height;
        let description_lines = (description.len() as f32 / 40.0).ceil() as i32;
        let description_height = description_lines as f32 * line_height;

        let tooltip_rect = Rectangle {
            x: position.x,
            y: position.y - title_height - description_height - padding * 2.0,
            width: max_width,
            height: title_height + description_height + padding * 2.0,
        };

        // Background
        let bg_color = self.modern_ui.get_theme().surface.overlay;
        renderer.fill_rect(&tooltip_rect, &bg_color)?;
        renderer.stroke_rect(&tooltip_rect, &self.modern_ui.get_theme().border.primary, 1.0)?;

        // Title
        let title_rect = Rectangle {
            x: tooltip_rect.x + padding,
            y: tooltip_rect.y + padding,
            width: tooltip_rect.width - padding * 2.0,
            height: title_height,
        };

        let title_style = TextStyle {
            font_family: "Inter".to_string(),
            font_size: 12.0,
            font_weight: 600,
            color: self.modern_ui.get_theme().text.primary,
            line_height: 1.2,
        };

        renderer.render_text(title, &title_rect, &title_style)?;

        // Description
        let desc_rect = Rectangle {
            x: tooltip_rect.x + padding,
            y: title_rect.y + title_rect.height + 2.0,
            width: tooltip_rect.width - padding * 2.0,
            height: description_height,
        };

        let desc_style = TextStyle {
            font_family: "Inter".to_string(),
            font_size: 10.0,
            font_weight: 400,
            color: self.modern_ui.get_theme().text.secondary,
            line_height: 1.4,
        };

        renderer.render_text(description, &desc_rect, &desc_style)?;

        Ok(())
    }

    fn find_tool_definition(&self, tool_type: ToolType) -> Option<&ToolDefinition> {
        for group in &self.tool_groups {
            for tool in &group.tools {
                if tool.tool_type == tool_type {
                    return Some(tool);
                }
            }
        }
        None
    }

    fn point_in_rect(&self, point: Vec2, rect: &Rectangle) -> bool {
        point.x >= rect.x
            && point.x <= rect.x + rect.width
            && point.y >= rect.y
            && point.y <= rect.y + rect.height
    }
}

// Helper functions
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

impl Color {
    fn with_alpha(&self, alpha: f32) -> Color {
        Color::new(self.r, self.g, self.b, alpha)
    }

    fn darken(&self, amount: f32) -> Color {
        Color::new(
            (self.r * (1.0 - amount)).max(0.0),
            (self.g * (1.0 - amount)).max(0.0),
            (self.b * (1.0 - amount)).max(0.0),
            self.a,
        )
    }

    fn transparent() -> Color {
        Color::new(0.0, 0.0, 0.0, 0.0)
    }
}

impl ToolPreviewData {
    pub fn default() -> Self {
        Self {
            thumbnail: None,
            preview_mesh: None,
            material_preview: None,
            animation_preview: None,
        }
    }
}

impl ContextMenu {
    pub fn new() -> Self {
        Self {
            position: Vec2::new(0.0, 0.0),
            size: Vec2::new(180.0, 200.0),
            visible: false,
            items: Vec::new(),
            selected_index: None,
            submenu: None,
            animation_state: 0.0,
        }
    }

    pub fn update(&mut self, delta_time: f32, input: &InputManager) {
        if !self.visible {
            return;
        }

        // Update animation
        self.animation_state = (self.animation_state + delta_time * 6.0).min(1.0);

        // Handle input
        if input.is_mouse_button_just_pressed(MouseButton::Left) {
            // Check if clicked outside menu
            let mouse_pos = Vec2::new(input.mouse_position().0, input.mouse_position().1);
            let menu_rect = Rectangle {
                x: self.position.x,
                y: self.position.y,
                width: self.size.x,
                height: self.size.y,
            };

            if !self.point_in_rect(mouse_pos, &menu_rect) {
                self.visible = false;
            }
        }
    }

    pub fn render(&self, renderer: &mut dyn Renderer) -> RobinResult<()> {
        if !self.visible || self.animation_state <= 0.01 {
            return Ok(());
        }

        // Scale animation
        let scale = self.animation_state;
        let scaled_size = Vec2::new(self.size.x * scale, self.size.y * scale);
        let scaled_pos = Vec2::new(
            self.position.x - (scaled_size.x - self.size.x) * 0.5,
            self.position.y - (scaled_size.y - self.size.y) * 0.5,
        );

        let menu_rect = Rectangle {
            x: scaled_pos.x,
            y: scaled_pos.y,
            width: scaled_size.x,
            height: scaled_size.y,
        };

        // Background
        let bg_color = Color::new(0.1, 0.1, 0.1, 0.95 * self.animation_state);
        renderer.fill_rect(&menu_rect, &bg_color)?;
        renderer.stroke_rect(&menu_rect, &Color::new(0.3, 0.3, 0.3, self.animation_state), 1.0)?;

        // Render items
        let item_height = 24.0;
        let mut current_y = menu_rect.y + 4.0;

        for (index, item) in self.items.iter().enumerate() {
            let item_rect = Rectangle {
                x: menu_rect.x + 4.0,
                y: current_y,
                width: menu_rect.width - 8.0,
                height: item_height,
            };

            let is_selected = self.selected_index == Some(index);
            self.render_menu_item(item, &item_rect, is_selected, renderer)?;

            current_y += item_height;
            if item.separator_after {
                current_y += 4.0;
            }
        }

        Ok(())
    }

    fn render_menu_item(&self, item: &ContextMenuItem, rect: &Rectangle, is_selected: bool, renderer: &mut dyn Renderer) -> RobinResult<()> {
        // Background
        if is_selected {
            let selection_color = Color::new(0.2, 0.4, 0.8, 0.3);
            renderer.fill_rect(rect, &selection_color)?;
        }

        // Text
        let text_style = TextStyle {
            font_family: "Inter".to_string(),
            font_size: 11.0,
            font_weight: 400,
            color: if item.enabled {
                Color::new(0.9, 0.9, 0.9, 1.0)
            } else {
                Color::new(0.5, 0.5, 0.5, 1.0)
            },
            line_height: 1.2,
        };

        let text_rect = Rectangle {
            x: rect.x + 8.0,
            y: rect.y + 4.0,
            width: rect.width - 16.0,
            height: rect.height - 8.0,
        };

        renderer.render_text(&item.text, &text_rect, &text_style)?;

        Ok(())
    }

    fn point_in_rect(&self, point: Vec2, rect: &Rectangle) -> bool {
        point.x >= rect.x
            && point.x <= rect.x + rect.width
            && point.y >= rect.y
            && point.y <= rect.y + rect.height
    }
}

impl ToolSearchBar {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            visible: false,
            results: Vec::new(),
            selected_result: None,
            search_history: Vec::new(),
            auto_complete: Vec::new(),
        }
    }

    pub fn update(&mut self, input: &InputManager) {
        // Update search functionality
        // This would handle keyboard input for search queries
    }

    pub fn render(&self, renderer: &mut dyn Renderer) -> RobinResult<()> {
        if !self.visible {
            return Ok(());
        }

        // Render search bar interface
        // Implementation would go here

        Ok(())
    }
}

impl FavoriteTools {
    pub fn new() -> Self {
        Self {
            tools: Vec::new(),
            max_favorites: 8,
            quick_select_mode: false,
        }
    }
}

impl QuickAccessBar {
    pub fn new() -> Self {
        Self {
            tools: Vec::new(),
            position: Vec2::new(20.0, 20.0),
            size: Vec2::new(200.0, 40.0),
            visible: true,
            horizontal_layout: true,
        }
    }

    pub fn render(&self, renderer: &mut dyn Renderer) -> RobinResult<()> {
        if !self.visible {
            return Ok(());
        }

        // Render quick access toolbar
        // Implementation would go here

        Ok(())
    }
}

// Placeholder trait and types for compilation
pub trait Renderer {
    fn fill_rect(&mut self, rect: &Rectangle, color: &Color) -> RobinResult<()>;
    fn stroke_rect(&mut self, rect: &Rectangle, color: &Color, width: f32) -> RobinResult<()>;
    fn render_text(&mut self, text: &str, rect: &Rectangle, style: &TextStyle) -> RobinResult<()>;
    fn draw_line(&mut self, start: Vec2, end: Vec2, color: &Color, width: f32) -> RobinResult<()>;
    fn fill_circle(&mut self, center: &Vec2, radius: f32, color: &Color) -> RobinResult<()>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolType {
    VoxelBrush,
    TerrainSculpt,
    TerrainPaint,
    StructurePlacer,
    WallBuilder,
    ElementPlacer,
    NPCPlacer,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_palette_creation() {
        let modern_ui = ModernUISystem::new(); // This would need to be implemented
        let palette = ToolPalette::new(modern_ui);
        assert!(palette.is_visible());
        assert!(!palette.tool_groups.is_empty());
    }

    #[test]
    fn test_tool_selection() {
        let modern_ui = ModernUISystem::new();
        let mut palette = ToolPalette::new(modern_ui);

        palette.select_tool(ToolType::VoxelBrush);
        assert_eq!(palette.get_selected_tool(), Some(ToolType::VoxelBrush));
    }

    #[test]
    fn test_context_menu() {
        let mut menu = ContextMenu::new();
        assert!(!menu.visible);

        menu.visible = true;
        assert!(menu.visible);
    }
}