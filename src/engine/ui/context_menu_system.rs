use crate::engine::{
    error::RobinResult,
    input::InputManager,
    math::{Vec2, Vec3},
    ui::modern_interface::{ModernUISystem, UITheme, Color, Rectangle, TextStyle},
    build_mode::{
        tools::{BuildTool, ToolType},
        interactive_elements::ElementType,
    },
};
use winit::event::{MouseButton, ElementState};
use std::collections::HashMap;

pub struct ContextMenuSystem {
    modern_ui: ModernUISystem,
    active_menu: Option<ContextMenu>,
    menu_history: Vec<ContextMenu>,
    global_actions: GlobalActionRegistry,
    context_providers: Vec<Box<dyn ContextProvider>>,
    animation_controller: MenuAnimationController,
    keyboard_navigation: KeyboardNavigation,
}

#[derive(Debug, Clone)]
pub struct ContextMenu {
    pub id: String,
    pub position: Vec2,
    pub size: Vec2,
    pub items: Vec<ContextMenuItem>,
    pub selected_index: Option<usize>,
    pub submenu: Option<Box<ContextMenu>>,
    pub parent_menu: Option<String>,
    pub context: MenuContext,
    pub animation_state: MenuAnimationState,
    pub visible: bool,
}

#[derive(Debug, Clone)]
pub struct ContextMenuItem {
    pub id: String,
    pub text: String,
    pub icon: Option<MenuIcon>,
    pub action: MenuAction,
    pub enabled: bool,
    pub visible: bool,
    pub separator_after: bool,
    pub submenu_items: Option<Vec<ContextMenuItem>>,
    pub keyboard_shortcut: Option<String>,
    pub tooltip: Option<String>,
    pub item_type: MenuItemType,
    pub state: MenuItemState,
}

#[derive(Debug, Clone)]
pub enum MenuItemType {
    Action,
    Submenu,
    Separator,
    Toggle,
    Radio,
    Slider,
    ColorPicker,
    Custom,
}

#[derive(Debug, Clone)]
pub struct MenuItemState {
    pub checked: bool,
    pub radio_group: Option<String>,
    pub slider_value: f32,
    pub color_value: Color,
}

#[derive(Debug, Clone)]
pub enum MenuAction {
    // Tool Actions
    SelectTool(ToolType),
    ToggleToolMode(ToolMode),
    ConfigureTool(ToolType, ToolConfiguration),

    // Element Actions
    PlaceElement(ElementType),
    EditElement(u32),
    DeleteElement(u32),
    DuplicateElement(u32),
    GroupElements(Vec<u32>),

    // Palette Actions
    AddToFavorites(ToolType),
    RemoveFromFavorites(ToolType),
    CreateToolGroup(String),
    EditToolGroup(String),

    // Interface Actions
    ShowProperties,
    ShowDocumentation(String),
    ToggleGrid,
    ToggleSnapping,
    ChangeViewMode(ViewMode),

    // System Actions
    Save,
    Load,
    Export(ExportFormat),
    Import(ImportFormat),
    Undo,
    Redo,

    // Custom Actions
    Custom(String, HashMap<String, String>),
}

#[derive(Debug, Clone)]
pub enum ToolMode {
    Place,
    Edit,
    Delete,
    Move,
    Rotate,
    Scale,
    Paint,
    Sculpt,
}

#[derive(Debug, Clone)]
pub struct ToolConfiguration {
    pub brush_size: Option<f32>,
    pub opacity: Option<f32>,
    pub material: Option<String>,
    pub auto_connect: Option<bool>,
    pub snap_to_grid: Option<bool>,
}

#[derive(Debug, Clone)]
pub enum ViewMode {
    Perspective,
    Orthographic,
    Top,
    Front,
    Side,
    Wireframe,
    Textured,
}

#[derive(Debug, Clone)]
pub enum ExportFormat {
    OBJ,
    FBX,
    GLTF,
    PNG,
    JSON,
}

#[derive(Debug, Clone)]
pub enum ImportFormat {
    OBJ,
    FBX,
    GLTF,
    Image,
    JSON,
}

#[derive(Debug, Clone)]
pub struct MenuContext {
    pub context_type: ContextType,
    pub target_id: Option<String>,
    pub position_3d: Option<Vec3>,
    pub selected_elements: Vec<u32>,
    pub current_tool: Option<ToolType>,
    pub modifier_keys: ModifierKeyState,
}

#[derive(Debug, Clone)]
pub enum ContextType {
    Viewport,
    ToolPalette,
    ElementInspector,
    Timeline,
    AssetBrowser,
    SceneHierarchy,
    Properties,
    Toolbar,
}

#[derive(Debug, Clone)]
pub struct ModifierKeyState {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

#[derive(Debug, Clone)]
pub struct MenuIcon {
    pub icon_type: IconType,
    pub color: Color,
    pub size: f32,
}

#[derive(Debug, Clone)]
pub enum IconType {
    Unicode(String),
    SVG(String),
    Bitmap(Vec<u8>),
    FontAwesome(String),
    Material(String),
}

#[derive(Debug, Clone)]
pub struct MenuAnimationState {
    pub scale: f32,
    pub opacity: f32,
    pub slide_offset: Vec2,
    pub item_animations: HashMap<String, ItemAnimation>,
    pub show_animation: AnimationPhase,
    pub hide_animation: AnimationPhase,
}

#[derive(Debug, Clone)]
pub struct ItemAnimation {
    pub hover_scale: f32,
    pub selection_glow: f32,
    pub slide_in_progress: f32,
}

#[derive(Debug, Clone)]
pub enum AnimationPhase {
    None,
    FadeIn,
    ScaleIn,
    SlideIn,
    FadeOut,
    ScaleOut,
    SlideOut,
}

pub struct GlobalActionRegistry {
    actions: HashMap<String, Box<dyn ContextAction>>,
    shortcuts: HashMap<String, String>,
}

pub trait ContextAction: Send + Sync {
    fn execute(&self, context: &MenuContext) -> RobinResult<()>;
    fn is_enabled(&self, context: &MenuContext) -> bool;
    fn get_display_text(&self, context: &MenuContext) -> String;
    fn get_tooltip(&self, context: &MenuContext) -> Option<String>;
}

pub trait ContextProvider: Send + Sync {
    fn get_context_type(&self) -> ContextType;
    fn provide_context_items(&self, context: &MenuContext) -> Vec<ContextMenuItem>;
    fn handle_context_action(&self, action: &MenuAction, context: &MenuContext) -> RobinResult<()>;
}

pub struct MenuAnimationController {
    show_duration: f32,
    hide_duration: f32,
    easing_function: EasingFunction,
    stagger_delay: f32,
}

#[derive(Debug, Clone)]
pub enum EasingFunction {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bounce,
    Elastic,
    Back,
}

pub struct KeyboardNavigation {
    enabled: bool,
    selected_path: Vec<usize>,
    wrap_around: bool,
    quick_search: String,
    search_timeout: f32,
}

impl ContextMenuSystem {
    pub fn new(modern_ui: ModernUISystem) -> Self {
        let mut system = Self {
            modern_ui,
            active_menu: None,
            menu_history: Vec::new(),
            global_actions: GlobalActionRegistry::new(),
            context_providers: Vec::new(),
            animation_controller: MenuAnimationController::new(),
            keyboard_navigation: KeyboardNavigation::new(),
        };

        system.register_default_providers();
        system.register_default_actions();
        system
    }

    fn register_default_providers(&mut self) {
        self.context_providers.push(Box::new(ViewportContextProvider::new()));
        self.context_providers.push(Box::new(ToolPaletteContextProvider::new()));
        self.context_providers.push(Box::new(ElementContextProvider::new()));
        self.context_providers.push(Box::new(SystemContextProvider::new()));
    }

    fn register_default_actions(&mut self) {
        self.global_actions.register("select_tool", Box::new(SelectToolAction));
        self.global_actions.register("place_element", Box::new(PlaceElementAction));
        self.global_actions.register("delete_element", Box::new(DeleteElementAction));
        self.global_actions.register("save_scene", Box::new(SaveSceneAction));
        self.global_actions.register("load_scene", Box::new(LoadSceneAction));
        self.global_actions.register("toggle_grid", Box::new(ToggleGridAction));
        self.global_actions.register("show_properties", Box::new(ShowPropertiesAction));
    }

    pub fn show_context_menu(&mut self, position: Vec2, context: MenuContext) -> RobinResult<()> {
        // Hide any existing menu
        self.hide_menu();

        // Collect menu items from providers
        let mut items = Vec::new();
        for provider in &self.context_providers {
            if self.is_provider_applicable(provider.as_ref(), &context) {
                let provider_items = provider.provide_context_items(&context);
                if !provider_items.is_empty() {
                    items.extend(provider_items);

                    // Add separator if not the last provider
                    if items.len() > 0 {
                        items.last_mut().unwrap().separator_after = true;
                    }
                }
            }
        }

        if items.is_empty() {
            return Ok(());
        }

        // Create menu
        let menu = ContextMenu {
            id: format!("context_menu_{}", chrono::Utc::now().timestamp_millis()),
            position,
            size: self.calculate_menu_size(&items),
            items,
            selected_index: None,
            submenu: None,
            parent_menu: None,
            context,
            animation_state: MenuAnimationState::new(),
            visible: true,
        };

        self.active_menu = Some(menu);
        self.start_show_animation();

        Ok(())
    }

    pub fn hide_menu(&mut self) {
        if let Some(menu) = &mut self.active_menu {
            self.start_hide_animation();
        }
    }

    pub fn update(&mut self, delta_time: f32, input: &InputManager) -> RobinResult<()> {
        if let Some(menu) = &mut self.active_menu {
            // Update animations
            self.animation_controller.update(&mut menu.animation_state, delta_time);

            // Handle input
            self.handle_menu_input(menu, input)?;

            // Update keyboard navigation
            self.keyboard_navigation.update(delta_time, input);

            // Clean up finished animations
            if !menu.visible && menu.animation_state.opacity <= 0.01 {
                self.active_menu = None;
            }
        }

        Ok(())
    }

    fn handle_menu_input(&mut self, menu: &mut ContextMenu, input: &InputManager) -> RobinResult<()> {
        let mouse_pos = Vec2::new(input.mouse_position().0, input.mouse_position().1);

        // Handle mouse movement for hovering
        self.update_hover_state(menu, mouse_pos);

        // Handle mouse clicks
        if input.is_mouse_button_just_pressed(MouseButton::Left) {
            if self.is_point_in_menu(mouse_pos, menu) {
                self.handle_menu_click(menu, mouse_pos)?;
            } else {
                self.hide_menu();
            }
        }

        // Handle right clicks for submenus
        if input.is_mouse_button_just_pressed(MouseButton::Right) {
            if !self.is_point_in_menu(mouse_pos, menu) {
                self.hide_menu();
            }
        }

        // Handle keyboard navigation
        if self.keyboard_navigation.enabled {
            self.handle_keyboard_navigation(menu, input)?;
        }

        Ok(())
    }

    fn update_hover_state(&mut self, menu: &mut ContextMenu, mouse_pos: Vec2) {
        let mut hovered_index = None;
        let item_height = 28.0;
        let mut current_y = menu.position.y + 4.0;

        for (index, item) in menu.items.iter().enumerate() {
            if !item.visible {
                continue;
            }

            let item_rect = Rectangle {
                x: menu.position.x,
                y: current_y,
                width: menu.size.x,
                height: item_height,
            };

            if self.point_in_rect(mouse_pos, &item_rect) {
                hovered_index = Some(index);
                break;
            }

            current_y += item_height + if item.separator_after { 4.0 } else { 0.0 };
        }

        // Update selection and animations
        if menu.selected_index != hovered_index {
            menu.selected_index = hovered_index;

            // Close any existing submenu if hovering different item
            if hovered_index.is_none() || menu.submenu.is_some() {
                menu.submenu = None;
            }

            // Show submenu if item has one
            if let Some(index) = hovered_index {
                if let Some(item) = menu.items.get(index) {
                    if let Some(submenu_items) = &item.submenu_items {
                        self.show_submenu(menu, index, submenu_items.clone());
                    }
                }
            }
        }
    }

    fn handle_menu_click(&mut self, menu: &mut ContextMenu, mouse_pos: Vec2) -> RobinResult<()> {
        if let Some(selected_index) = menu.selected_index {
            if let Some(item) = menu.items.get(selected_index) {
                if item.enabled {
                    // Handle different item types
                    match item.item_type {
                        MenuItemType::Action => {
                            self.execute_menu_action(&item.action, &menu.context)?;
                            self.hide_menu();
                        }
                        MenuItemType::Toggle => {
                            // Toggle the state and execute action
                            // In a real implementation, you'd update the item state
                            self.execute_menu_action(&item.action, &menu.context)?;
                        }
                        MenuItemType::Submenu => {
                            // Submenu was already shown on hover
                        }
                        _ => {
                            // Handle other item types
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn handle_keyboard_navigation(&mut self, menu: &mut ContextMenu, input: &InputManager) -> RobinResult<()> {
        // Arrow key navigation
        if input.is_named_key_just_pressed(winit::keyboard::NamedKey::ArrowDown) {
            self.navigate_menu(menu, NavigationDirection::Down);
        }
        if input.is_named_key_just_pressed(winit::keyboard::NamedKey::ArrowUp) {
            self.navigate_menu(menu, NavigationDirection::Up);
        }
        if input.is_named_key_just_pressed(winit::keyboard::NamedKey::ArrowRight) {
            self.navigate_menu(menu, NavigationDirection::Right);
        }
        if input.is_named_key_just_pressed(winit::keyboard::NamedKey::ArrowLeft) {
            self.navigate_menu(menu, NavigationDirection::Left);
        }

        // Enter to activate
        if input.is_named_key_just_pressed(winit::keyboard::NamedKey::Enter) {
            if let Some(selected_index) = menu.selected_index {
                if let Some(item) = menu.items.get(selected_index) {
                    if item.enabled {
                        self.execute_menu_action(&item.action, &menu.context)?;
                        self.hide_menu();
                    }
                }
            }
        }

        // Escape to close
        if input.is_named_key_just_pressed(winit::keyboard::NamedKey::Escape) {
            self.hide_menu();
        }

        Ok(())
    }

    fn navigate_menu(&mut self, menu: &mut ContextMenu, direction: NavigationDirection) {
        let visible_items: Vec<usize> = menu.items.iter()
            .enumerate()
            .filter(|(_, item)| item.visible && item.item_type != MenuItemType::Separator)
            .map(|(index, _)| index)
            .collect();

        if visible_items.is_empty() {
            return;
        }

        match direction {
            NavigationDirection::Down => {
                if let Some(current) = menu.selected_index {
                    if let Some(pos) = visible_items.iter().position(|&i| i == current) {
                        let next_pos = if self.keyboard_navigation.wrap_around {
                            (pos + 1) % visible_items.len()
                        } else {
                            (pos + 1).min(visible_items.len() - 1)
                        };
                        menu.selected_index = Some(visible_items[next_pos]);
                    }
                } else {
                    menu.selected_index = Some(visible_items[0]);
                }
            }
            NavigationDirection::Up => {
                if let Some(current) = menu.selected_index {
                    if let Some(pos) = visible_items.iter().position(|&i| i == current) {
                        let prev_pos = if self.keyboard_navigation.wrap_around {
                            if pos == 0 { visible_items.len() - 1 } else { pos - 1 }
                        } else {
                            pos.saturating_sub(1)
                        };
                        menu.selected_index = Some(visible_items[prev_pos]);
                    }
                } else {
                    menu.selected_index = Some(visible_items[visible_items.len() - 1]);
                }
            }
            NavigationDirection::Right => {
                // Show submenu if available
                if let Some(selected_index) = menu.selected_index {
                    if let Some(item) = menu.items.get(selected_index) {
                        if let Some(submenu_items) = &item.submenu_items {
                            self.show_submenu(menu, selected_index, submenu_items.clone());
                        }
                    }
                }
            }
            NavigationDirection::Left => {
                // Close submenu or go to parent
                if menu.submenu.is_some() {
                    menu.submenu = None;
                } else if menu.parent_menu.is_some() {
                    // Would navigate to parent menu
                }
            }
        }
    }

    fn show_submenu(&mut self, parent_menu: &mut ContextMenu, parent_index: usize, items: Vec<ContextMenuItem>) {
        let submenu_position = Vec2::new(
            parent_menu.position.x + parent_menu.size.x - 4.0,
            parent_menu.position.y + (parent_index as f32 * 28.0) + 4.0,
        );

        let submenu = ContextMenu {
            id: format!("submenu_{}_{}", parent_menu.id, parent_index),
            position: submenu_position,
            size: self.calculate_menu_size(&items),
            items,
            selected_index: None,
            submenu: None,
            parent_menu: Some(parent_menu.id.clone()),
            context: parent_menu.context.clone(),
            animation_state: MenuAnimationState::new(),
            visible: true,
        };

        parent_menu.submenu = Some(Box::new(submenu));
    }

    fn execute_menu_action(&mut self, action: &MenuAction, context: &MenuContext) -> RobinResult<()> {
        // Find appropriate provider to handle the action
        for provider in &self.context_providers {
            if self.is_provider_applicable(provider.as_ref(), context) {
                if let Ok(()) = provider.handle_context_action(action, context) {
                    return Ok(());
                }
            }
        }

        // Fallback to global actions
        match action {
            MenuAction::Custom(action_name, params) => {
                if let Some(global_action) = self.global_actions.get_action(action_name) {
                    global_action.execute(context)?;
                }
            }
            _ => {
                println!("Unhandled menu action: {:?}", action);
            }
        }

        Ok(())
    }

    fn calculate_menu_size(&self, items: &[ContextMenuItem]) -> Vec2 {
        let mut max_width = 120.0;
        let mut total_height = 8.0; // Top/bottom padding

        for item in items {
            if !item.visible {
                continue;
            }

            // Calculate text width (simplified)
            let text_width = item.text.len() as f32 * 7.0 + 20.0; // Rough estimate
            if let Some(shortcut) = &item.keyboard_shortcut {
                let shortcut_width = shortcut.len() as f32 * 6.0;
                max_width = max_width.max(text_width + shortcut_width + 40.0);
            } else {
                max_width = max_width.max(text_width + 20.0);
            }

            total_height += 28.0;
            if item.separator_after {
                total_height += 4.0;
            }
        }

        Vec2::new(max_width, total_height)
    }

    fn is_provider_applicable(&self, provider: &dyn ContextProvider, context: &MenuContext) -> bool {
        provider.get_context_type() == context.context_type ||
        matches!(provider.get_context_type(), ContextType::Viewport) // Viewport provider is always applicable
    }

    fn is_point_in_menu(&self, point: Vec2, menu: &ContextMenu) -> bool {
        self.point_in_rect(point, &Rectangle {
            x: menu.position.x,
            y: menu.position.y,
            width: menu.size.x,
            height: menu.size.y,
        })
    }

    fn point_in_rect(&self, point: Vec2, rect: &Rectangle) -> bool {
        point.x >= rect.x
            && point.x <= rect.x + rect.width
            && point.y >= rect.y
            && point.y <= rect.y + rect.height
    }

    fn start_show_animation(&mut self) {
        if let Some(menu) = &mut self.active_menu {
            menu.animation_state.show_animation = AnimationPhase::ScaleIn;
            menu.animation_state.scale = 0.0;
            menu.animation_state.opacity = 0.0;
        }
    }

    fn start_hide_animation(&mut self) {
        if let Some(menu) = &mut self.active_menu {
            menu.animation_state.hide_animation = AnimationPhase::FadeOut;
            menu.visible = false;
        }
    }

    pub fn render(&self, renderer: &mut dyn Renderer) -> RobinResult<()> {
        if let Some(menu) = &self.active_menu {
            self.render_menu(menu, renderer)?;

            // Render submenu
            if let Some(submenu) = &menu.submenu {
                self.render_menu(submenu, renderer)?;
            }
        }

        Ok(())
    }

    fn render_menu(&self, menu: &ContextMenu, renderer: &mut dyn Renderer) -> RobinResult<()> {
        if menu.animation_state.opacity <= 0.01 {
            return Ok(());
        }

        // Apply animation transformations
        let scale = menu.animation_state.scale;
        let opacity = menu.animation_state.opacity;

        let scaled_size = Vec2::new(menu.size.x * scale, menu.size.y * scale);
        let scaled_position = Vec2::new(
            menu.position.x + (menu.size.x - scaled_size.x) * 0.5,
            menu.position.y + (menu.size.y - scaled_size.y) * 0.5,
        );

        let menu_rect = Rectangle {
            x: scaled_position.x,
            y: scaled_position.y,
            width: scaled_size.x,
            height: scaled_size.y,
        };

        // Background
        let bg_color = self.modern_ui.get_theme().surface.overlay.with_alpha(opacity);
        renderer.fill_rect(&menu_rect, &bg_color)?;

        // Border
        let border_color = self.modern_ui.get_theme().border.primary.with_alpha(opacity);
        renderer.stroke_rect(&menu_rect, &border_color, 1.0)?;

        // Shadow effect
        let shadow_rect = Rectangle {
            x: menu_rect.x + 2.0,
            y: menu_rect.y + 2.0,
            width: menu_rect.width,
            height: menu_rect.height,
        };
        let shadow_color = Color::new(0.0, 0.0, 0.0, 0.3 * opacity);
        renderer.fill_rect(&shadow_rect, &shadow_color)?;

        // Render menu items
        self.render_menu_items(menu, &menu_rect, opacity, renderer)?;

        Ok(())
    }

    fn render_menu_items(&self, menu: &ContextMenu, menu_rect: &Rectangle, opacity: f32, renderer: &mut dyn Renderer) -> RobinResult<()> {
        let item_height = 28.0;
        let padding = 4.0;
        let mut current_y = menu_rect.y + padding;

        for (index, item) in menu.items.iter().enumerate() {
            if !item.visible {
                continue;
            }

            let item_rect = Rectangle {
                x: menu_rect.x + padding,
                y: current_y,
                width: menu_rect.width - (padding * 2.0),
                height: item_height,
            };

            let is_selected = menu.selected_index == Some(index);
            self.render_menu_item(item, &item_rect, is_selected, opacity, renderer)?;

            current_y += item_height;

            // Render separator
            if item.separator_after {
                let separator_y = current_y + 2.0;
                let separator_color = self.modern_ui.get_theme().border.secondary.with_alpha(opacity);

                renderer.draw_line(
                    Vec2::new(menu_rect.x + padding * 2.0, separator_y),
                    Vec2::new(menu_rect.x + menu_rect.width - padding * 2.0, separator_y),
                    &separator_color,
                    1.0,
                )?;

                current_y += 4.0;
            }
        }

        Ok(())
    }

    fn render_menu_item(&self, item: &ContextMenuItem, rect: &Rectangle, is_selected: bool, opacity: f32, renderer: &mut dyn Renderer) -> RobinResult<()> {
        // Background for selected item
        if is_selected {
            let selection_color = self.modern_ui.get_theme().interactive.hover.with_alpha(opacity);
            renderer.fill_rect(rect, &selection_color)?;
        }

        // Disabled overlay
        if !item.enabled {
            let disabled_overlay = Color::new(0.5, 0.5, 0.5, 0.5 * opacity);
            renderer.fill_rect(rect, &disabled_overlay)?;
        }

        // Icon
        let mut text_start_x = rect.x + 8.0;
        if let Some(icon) = &item.icon {
            let icon_rect = Rectangle {
                x: rect.x + 6.0,
                y: rect.y + 6.0,
                width: 16.0,
                height: 16.0,
            };
            self.render_menu_icon(icon, &icon_rect, opacity, renderer)?;
            text_start_x = rect.x + 28.0;
        }

        // Text
        let text_color = if item.enabled {
            self.modern_ui.get_theme().text.primary.with_alpha(opacity)
        } else {
            self.modern_ui.get_theme().text.disabled.with_alpha(opacity)
        };

        let text_style = TextStyle {
            font_family: "Inter".to_string(),
            font_size: 11.0,
            font_weight: if is_selected { 500 } else { 400 },
            color: text_color,
            line_height: 1.2,
        };

        let text_rect = Rectangle {
            x: text_start_x,
            y: rect.y + 6.0,
            width: rect.width - (text_start_x - rect.x) - 30.0,
            height: 16.0,
        };

        renderer.render_text(&item.text, &text_rect, &text_style)?;

        // Keyboard shortcut
        if let Some(shortcut) = &item.keyboard_shortcut {
            let shortcut_color = self.modern_ui.get_theme().text.tertiary.with_alpha(opacity);
            let shortcut_style = TextStyle {
                font_family: "JetBrains Mono".to_string(),
                font_size: 9.0,
                font_weight: 400,
                color: shortcut_color,
                line_height: 1.0,
            };

            let shortcut_rect = Rectangle {
                x: rect.x + rect.width - 25.0,
                y: rect.y + 7.0,
                width: 20.0,
                height: 14.0,
            };

            renderer.render_text(shortcut, &shortcut_rect, &shortcut_style)?;
        }

        // Submenu indicator
        if item.submenu_items.is_some() {
            let arrow_color = self.modern_ui.get_theme().text.secondary.with_alpha(opacity);
            let arrow_rect = Rectangle {
                x: rect.x + rect.width - 15.0,
                y: rect.y + 8.0,
                width: 12.0,
                height: 12.0,
            };

            renderer.render_text("▶", &arrow_rect, &TextStyle {
                font_family: "Inter".to_string(),
                font_size: 10.0,
                font_weight: 400,
                color: arrow_color,
                line_height: 1.0,
            })?;
        }

        // Toggle state indicator
        if item.item_type == MenuItemType::Toggle && item.state.checked {
            let check_color = self.modern_ui.get_theme().colors.success.with_alpha(opacity);
            let check_rect = Rectangle {
                x: rect.x + 4.0,
                y: rect.y + 8.0,
                width: 12.0,
                height: 12.0,
            };

            renderer.render_text("✓", &check_rect, &TextStyle {
                font_family: "Inter".to_string(),
                font_size: 10.0,
                font_weight: 600,
                color: check_color,
                line_height: 1.0,
            })?;
        }

        Ok(())
    }

    fn render_menu_icon(&self, icon: &MenuIcon, rect: &Rectangle, opacity: f32, renderer: &mut dyn Renderer) -> RobinResult<()> {
        match &icon.icon_type {
            IconType::Unicode(text) => {
                let icon_style = TextStyle {
                    font_family: "Inter".to_string(),
                    font_size: icon.size,
                    font_weight: 400,
                    color: icon.color.with_alpha(opacity),
                    line_height: 1.0,
                };
                renderer.render_text(text, rect, &icon_style)?;
            }
            IconType::FontAwesome(icon_name) => {
                // Would render FontAwesome icon
                let icon_style = TextStyle {
                    font_family: "FontAwesome".to_string(),
                    font_size: icon.size,
                    font_weight: 400,
                    color: icon.color.with_alpha(opacity),
                    line_height: 1.0,
                };
                renderer.render_text(icon_name, rect, &icon_style)?;
            }
            _ => {
                // Fallback: render colored square
                renderer.fill_rect(rect, &icon.color.with_alpha(opacity))?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
enum NavigationDirection {
    Up,
    Down,
    Left,
    Right,
}

// Context Providers Implementation
struct ViewportContextProvider;
struct ToolPaletteContextProvider;
struct ElementContextProvider;
struct SystemContextProvider;

impl ViewportContextProvider {
    fn new() -> Self {
        Self
    }
}

impl ContextProvider for ViewportContextProvider {
    fn get_context_type(&self) -> ContextType {
        ContextType::Viewport
    }

    fn provide_context_items(&self, context: &MenuContext) -> Vec<ContextMenuItem> {
        let mut items = Vec::new();

        // View mode options
        items.push(ContextMenuItem {
            id: "view_perspective".to_string(),
            text: "Perspective View".to_string(),
            icon: Some(MenuIcon {
                icon_type: IconType::Unicode("⬛".to_string()),
                color: Color::new(0.6, 0.8, 1.0, 1.0),
                size: 12.0,
            }),
            action: MenuAction::ChangeViewMode(ViewMode::Perspective),
            enabled: true,
            visible: true,
            separator_after: false,
            submenu_items: None,
            keyboard_shortcut: Some("1".to_string()),
            tooltip: Some("Switch to perspective view".to_string()),
            item_type: MenuItemType::Action,
            state: MenuItemState::default(),
        });

        items.push(ContextMenuItem {
            id: "view_orthographic".to_string(),
            text: "Orthographic View".to_string(),
            icon: Some(MenuIcon {
                icon_type: IconType::Unicode("⬜".to_string()),
                color: Color::new(0.6, 0.8, 1.0, 1.0),
                size: 12.0,
            }),
            action: MenuAction::ChangeViewMode(ViewMode::Orthographic),
            enabled: true,
            visible: true,
            separator_after: true,
            submenu_items: None,
            keyboard_shortcut: Some("2".to_string()),
            tooltip: Some("Switch to orthographic view".to_string()),
            item_type: MenuItemType::Action,
            state: MenuItemState::default(),
        });

        // Grid and snapping
        items.push(ContextMenuItem {
            id: "toggle_grid".to_string(),
            text: "Show Grid".to_string(),
            icon: Some(MenuIcon {
                icon_type: IconType::Unicode("⊞".to_string()),
                color: Color::new(0.7, 0.7, 0.7, 1.0),
                size: 12.0,
            }),
            action: MenuAction::ToggleGrid,
            enabled: true,
            visible: true,
            separator_after: false,
            submenu_items: None,
            keyboard_shortcut: Some("G".to_string()),
            tooltip: Some("Toggle grid visibility".to_string()),
            item_type: MenuItemType::Toggle,
            state: MenuItemState::default(),
        });

        items
    }

    fn handle_context_action(&self, action: &MenuAction, context: &MenuContext) -> RobinResult<()> {
        match action {
            MenuAction::ChangeViewMode(mode) => {
                println!("Changing view mode to: {:?}", mode);
                // Implement view mode change
                Ok(())
            }
            MenuAction::ToggleGrid => {
                println!("Toggling grid");
                // Implement grid toggle
                Ok(())
            }
            _ => Err(crate::engine::error::RobinError::General("Unsupported action".to_string())),
        }
    }
}

impl ToolPaletteContextProvider {
    fn new() -> Self {
        Self
    }
}

impl ContextProvider for ToolPaletteContextProvider {
    fn get_context_type(&self) -> ContextType {
        ContextType::ToolPalette
    }

    fn provide_context_items(&self, context: &MenuContext) -> Vec<ContextMenuItem> {
        let mut items = Vec::new();

        // Tool actions
        if let Some(tool_type) = context.current_tool {
            items.push(ContextMenuItem {
                id: "tool_properties".to_string(),
                text: "Tool Properties".to_string(),
                icon: Some(MenuIcon {
                    icon_type: IconType::Unicode("⚙".to_string()),
                    color: Color::new(0.8, 0.8, 0.6, 1.0),
                    size: 12.0,
                }),
                action: MenuAction::ConfigureTool(tool_type, ToolConfiguration::default()),
                enabled: true,
                visible: true,
                separator_after: false,
                submenu_items: None,
                keyboard_shortcut: None,
                tooltip: Some("Configure tool settings".to_string()),
                item_type: MenuItemType::Action,
                state: MenuItemState::default(),
            });

            items.push(ContextMenuItem {
                id: "add_to_favorites".to_string(),
                text: "Add to Favorites".to_string(),
                icon: Some(MenuIcon {
                    icon_type: IconType::Unicode("★".to_string()),
                    color: Color::new(1.0, 0.8, 0.2, 1.0),
                    size: 12.0,
                }),
                action: MenuAction::AddToFavorites(tool_type),
                enabled: true,
                visible: true,
                separator_after: true,
                submenu_items: None,
                keyboard_shortcut: None,
                tooltip: Some("Add this tool to favorites".to_string()),
                item_type: MenuItemType::Action,
                state: MenuItemState::default(),
            });
        }

        items
    }

    fn handle_context_action(&self, action: &MenuAction, context: &MenuContext) -> RobinResult<()> {
        match action {
            MenuAction::AddToFavorites(tool_type) => {
                println!("Adding tool to favorites: {:?}", tool_type);
                Ok(())
            }
            MenuAction::ConfigureTool(tool_type, config) => {
                println!("Configuring tool: {:?} with config: {:?}", tool_type, config);
                Ok(())
            }
            _ => Err(crate::engine::error::RobinError::General("Unsupported action".to_string())),
        }
    }
}

impl ElementContextProvider {
    fn new() -> Self {
        Self
    }
}

impl ContextProvider for ElementContextProvider {
    fn get_context_type(&self) -> ContextType {
        ContextType::Viewport // Elements are selected in viewport
    }

    fn provide_context_items(&self, context: &MenuContext) -> Vec<ContextMenuItem> {
        let mut items = Vec::new();

        if !context.selected_elements.is_empty() {
            items.push(ContextMenuItem {
                id: "delete_elements".to_string(),
                text: if context.selected_elements.len() == 1 { "Delete Element".to_string() } else { "Delete Elements".to_string() },
                icon: Some(MenuIcon {
                    icon_type: IconType::Unicode("🗑".to_string()),
                    color: Color::new(1.0, 0.4, 0.4, 1.0),
                    size: 12.0,
                }),
                action: MenuAction::DeleteElement(context.selected_elements[0]),
                enabled: true,
                visible: true,
                separator_after: false,
                submenu_items: None,
                keyboard_shortcut: Some("Del".to_string()),
                tooltip: Some("Delete selected elements".to_string()),
                item_type: MenuItemType::Action,
                state: MenuItemState::default(),
            });

            items.push(ContextMenuItem {
                id: "duplicate_elements".to_string(),
                text: if context.selected_elements.len() == 1 { "Duplicate Element".to_string() } else { "Duplicate Elements".to_string() },
                icon: Some(MenuIcon {
                    icon_type: IconType::Unicode("⧉".to_string()),
                    color: Color::new(0.6, 0.8, 0.6, 1.0),
                    size: 12.0,
                }),
                action: MenuAction::DuplicateElement(context.selected_elements[0]),
                enabled: true,
                visible: true,
                separator_after: false,
                submenu_items: None,
                keyboard_shortcut: Some("Ctrl+D".to_string()),
                tooltip: Some("Duplicate selected elements".to_string()),
                item_type: MenuItemType::Action,
                state: MenuItemState::default(),
            });

            if context.selected_elements.len() > 1 {
                items.push(ContextMenuItem {
                    id: "group_elements".to_string(),
                    text: "Group Elements".to_string(),
                    icon: Some(MenuIcon {
                        icon_type: IconType::Unicode("⧈".to_string()),
                        color: Color::new(0.8, 0.6, 0.8, 1.0),
                        size: 12.0,
                    }),
                    action: MenuAction::GroupElements(context.selected_elements.clone()),
                    enabled: true,
                    visible: true,
                    separator_after: true,
                    submenu_items: None,
                    keyboard_shortcut: Some("Ctrl+G".to_string()),
                    tooltip: Some("Group selected elements".to_string()),
                    item_type: MenuItemType::Action,
                    state: MenuItemState::default(),
                });
            }
        }

        items
    }

    fn handle_context_action(&self, action: &MenuAction, context: &MenuContext) -> RobinResult<()> {
        match action {
            MenuAction::DeleteElement(element_id) => {
                println!("Deleting element: {}", element_id);
                Ok(())
            }
            MenuAction::DuplicateElement(element_id) => {
                println!("Duplicating element: {}", element_id);
                Ok(())
            }
            MenuAction::GroupElements(element_ids) => {
                println!("Grouping elements: {:?}", element_ids);
                Ok(())
            }
            _ => Err(crate::engine::error::RobinError::General("Unsupported action".to_string())),
        }
    }
}

impl SystemContextProvider {
    fn new() -> Self {
        Self
    }
}

impl ContextProvider for SystemContextProvider {
    fn get_context_type(&self) -> ContextType {
        ContextType::Viewport
    }

    fn provide_context_items(&self, context: &MenuContext) -> Vec<ContextMenuItem> {
        vec![
            ContextMenuItem {
                id: "save_scene".to_string(),
                text: "Save Scene".to_string(),
                icon: Some(MenuIcon {
                    icon_type: IconType::Unicode("💾".to_string()),
                    color: Color::new(0.6, 0.8, 1.0, 1.0),
                    size: 12.0,
                }),
                action: MenuAction::Save,
                enabled: true,
                visible: true,
                separator_after: false,
                submenu_items: None,
                keyboard_shortcut: Some("Ctrl+S".to_string()),
                tooltip: Some("Save the current scene".to_string()),
                item_type: MenuItemType::Action,
                state: MenuItemState::default(),
            },
            ContextMenuItem {
                id: "load_scene".to_string(),
                text: "Load Scene".to_string(),
                icon: Some(MenuIcon {
                    icon_type: IconType::Unicode("📁".to_string()),
                    color: Color::new(0.8, 0.8, 0.6, 1.0),
                    size: 12.0,
                }),
                action: MenuAction::Load,
                enabled: true,
                visible: true,
                separator_after: false,
                submenu_items: None,
                keyboard_shortcut: Some("Ctrl+O".to_string()),
                tooltip: Some("Load a scene file".to_string()),
                item_type: MenuItemType::Action,
                state: MenuItemState::default(),
            },
        ]
    }

    fn handle_context_action(&self, action: &MenuAction, context: &MenuContext) -> RobinResult<()> {
        match action {
            MenuAction::Save => {
                println!("Saving scene");
                Ok(())
            }
            MenuAction::Load => {
                println!("Loading scene");
                Ok(())
            }
            _ => Err(crate::engine::error::RobinError::General("Unsupported action".to_string())),
        }
    }
}

// Global Actions Implementation
struct SelectToolAction;
struct PlaceElementAction;
struct DeleteElementAction;
struct SaveSceneAction;
struct LoadSceneAction;
struct ToggleGridAction;
struct ShowPropertiesAction;

impl ContextAction for SelectToolAction {
    fn execute(&self, context: &MenuContext) -> RobinResult<()> {
        println!("Executing select tool action");
        Ok(())
    }

    fn is_enabled(&self, context: &MenuContext) -> bool {
        true
    }

    fn get_display_text(&self, context: &MenuContext) -> String {
        "Select Tool".to_string()
    }

    fn get_tooltip(&self, context: &MenuContext) -> Option<String> {
        Some("Select the specified tool".to_string())
    }
}

impl ContextAction for PlaceElementAction {
    fn execute(&self, context: &MenuContext) -> RobinResult<()> {
        println!("Executing place element action");
        Ok(())
    }

    fn is_enabled(&self, context: &MenuContext) -> bool {
        context.current_tool.is_some()
    }

    fn get_display_text(&self, context: &MenuContext) -> String {
        "Place Element".to_string()
    }

    fn get_tooltip(&self, context: &MenuContext) -> Option<String> {
        Some("Place an element at the cursor position".to_string())
    }
}

impl ContextAction for DeleteElementAction {
    fn execute(&self, context: &MenuContext) -> RobinResult<()> {
        println!("Executing delete element action");
        Ok(())
    }

    fn is_enabled(&self, context: &MenuContext) -> bool {
        !context.selected_elements.is_empty()
    }

    fn get_display_text(&self, context: &MenuContext) -> String {
        if context.selected_elements.len() == 1 {
            "Delete Element".to_string()
        } else {
            "Delete Elements".to_string()
        }
    }

    fn get_tooltip(&self, context: &MenuContext) -> Option<String> {
        Some("Delete the selected elements".to_string())
    }
}

impl ContextAction for SaveSceneAction {
    fn execute(&self, context: &MenuContext) -> RobinResult<()> {
        println!("Saving scene");
        Ok(())
    }

    fn is_enabled(&self, context: &MenuContext) -> bool {
        true
    }

    fn get_display_text(&self, context: &MenuContext) -> String {
        "Save Scene".to_string()
    }

    fn get_tooltip(&self, context: &MenuContext) -> Option<String> {
        Some("Save the current scene to a file".to_string())
    }
}

impl ContextAction for LoadSceneAction {
    fn execute(&self, context: &MenuContext) -> RobinResult<()> {
        println!("Loading scene");
        Ok(())
    }

    fn is_enabled(&self, context: &MenuContext) -> bool {
        true
    }

    fn get_display_text(&self, context: &MenuContext) -> String {
        "Load Scene".to_string()
    }

    fn get_tooltip(&self, context: &MenuContext) -> Option<String> {
        Some("Load a scene from a file".to_string())
    }
}

impl ContextAction for ToggleGridAction {
    fn execute(&self, context: &MenuContext) -> RobinResult<()> {
        println!("Toggling grid");
        Ok(())
    }

    fn is_enabled(&self, context: &MenuContext) -> bool {
        true
    }

    fn get_display_text(&self, context: &MenuContext) -> String {
        "Toggle Grid".to_string()
    }

    fn get_tooltip(&self, context: &MenuContext) -> Option<String> {
        Some("Show or hide the construction grid".to_string())
    }
}

impl ContextAction for ShowPropertiesAction {
    fn execute(&self, context: &MenuContext) -> RobinResult<()> {
        println!("Showing properties");
        Ok(())
    }

    fn is_enabled(&self, context: &MenuContext) -> bool {
        !context.selected_elements.is_empty()
    }

    fn get_display_text(&self, context: &MenuContext) -> String {
        "Properties".to_string()
    }

    fn get_tooltip(&self, context: &MenuContext) -> Option<String> {
        Some("Show properties panel for selected elements".to_string())
    }
}

// Helper implementations
impl GlobalActionRegistry {
    fn new() -> Self {
        Self {
            actions: HashMap::new(),
            shortcuts: HashMap::new(),
        }
    }

    fn register(&mut self, name: &str, action: Box<dyn ContextAction>) {
        self.actions.insert(name.to_string(), action);
    }

    fn get_action(&self, name: &str) -> Option<&Box<dyn ContextAction>> {
        self.actions.get(name)
    }
}

impl MenuAnimationController {
    fn new() -> Self {
        Self {
            show_duration: 0.15,
            hide_duration: 0.1,
            easing_function: EasingFunction::EaseOut,
            stagger_delay: 0.02,
        }
    }

    fn update(&self, animation_state: &mut MenuAnimationState, delta_time: f32) {
        match animation_state.show_animation {
            AnimationPhase::ScaleIn => {
                let progress = (animation_state.scale + delta_time / self.show_duration).min(1.0);
                animation_state.scale = self.apply_easing(progress);
                animation_state.opacity = progress;

                if progress >= 1.0 {
                    animation_state.show_animation = AnimationPhase::None;
                }
            }
            _ => {}
        }

        match animation_state.hide_animation {
            AnimationPhase::FadeOut => {
                let progress = (animation_state.opacity - delta_time / self.hide_duration).max(0.0);
                animation_state.opacity = progress;
                animation_state.scale = progress;

                if progress <= 0.0 {
                    animation_state.hide_animation = AnimationPhase::None;
                }
            }
            _ => {}
        }
    }

    fn apply_easing(&self, t: f32) -> f32 {
        match self.easing_function {
            EasingFunction::Linear => t,
            EasingFunction::EaseOut => 1.0 - (1.0 - t).powi(3),
            EasingFunction::EaseIn => t.powi(3),
            EasingFunction::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
            EasingFunction::Bounce => {
                if t < 1.0 / 2.75 {
                    7.5625 * t * t
                } else if t < 2.0 / 2.75 {
                    let t = t - 1.5 / 2.75;
                    7.5625 * t * t + 0.75
                } else if t < 2.5 / 2.75 {
                    let t = t - 2.25 / 2.75;
                    7.5625 * t * t + 0.9375
                } else {
                    let t = t - 2.625 / 2.75;
                    7.5625 * t * t + 0.984375
                }
            }
            _ => t,
        }
    }
}

impl KeyboardNavigation {
    fn new() -> Self {
        Self {
            enabled: true,
            selected_path: Vec::new(),
            wrap_around: true,
            quick_search: String::new(),
            search_timeout: 0.0,
        }
    }

    fn update(&mut self, delta_time: f32, input: &InputManager) {
        // Update search timeout
        if self.search_timeout > 0.0 {
            self.search_timeout -= delta_time;
            if self.search_timeout <= 0.0 {
                self.quick_search.clear();
            }
        }

        // Handle quick search typing
        // This would be implemented with proper character input handling
    }
}

impl MenuAnimationState {
    fn new() -> Self {
        Self {
            scale: 0.0,
            opacity: 0.0,
            slide_offset: Vec2::new(0.0, 0.0),
            item_animations: HashMap::new(),
            show_animation: AnimationPhase::None,
            hide_animation: AnimationPhase::None,
        }
    }
}

impl MenuItemState {
    fn default() -> Self {
        Self {
            checked: false,
            radio_group: None,
            slider_value: 0.0,
            color_value: Color::new(1.0, 1.0, 1.0, 1.0),
        }
    }
}

impl ToolConfiguration {
    fn default() -> Self {
        Self {
            brush_size: None,
            opacity: None,
            material: None,
            auto_connect: None,
            snap_to_grid: None,
        }
    }
}

impl PartialEq for ContextType {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

// Placeholder trait for renderer
pub trait Renderer {
    fn fill_rect(&mut self, rect: &Rectangle, color: &Color) -> RobinResult<()>;
    fn stroke_rect(&mut self, rect: &Rectangle, color: &Color, width: f32) -> RobinResult<()>;
    fn render_text(&mut self, text: &str, rect: &Rectangle, style: &TextStyle) -> RobinResult<()>;
    fn draw_line(&mut self, start: Vec2, end: Vec2, color: &Color, width: f32) -> RobinResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_menu_system_creation() {
        let modern_ui = ModernUISystem::new();
        let menu_system = ContextMenuSystem::new(modern_ui);
        assert!(menu_system.active_menu.is_none());
    }

    #[test]
    fn test_menu_context_creation() {
        let context = MenuContext {
            context_type: ContextType::Viewport,
            target_id: None,
            position_3d: None,
            selected_elements: vec![],
            current_tool: None,
            modifier_keys: ModifierKeyState {
                shift: false,
                ctrl: false,
                alt: false,
                meta: false,
            },
        };

        assert_eq!(context.context_type, ContextType::Viewport);
        assert!(context.selected_elements.is_empty());
    }

    #[test]
    fn test_menu_item_creation() {
        let item = ContextMenuItem {
            id: "test_item".to_string(),
            text: "Test Item".to_string(),
            icon: None,
            action: MenuAction::Save,
            enabled: true,
            visible: true,
            separator_after: false,
            submenu_items: None,
            keyboard_shortcut: Some("Ctrl+S".to_string()),
            tooltip: Some("Test tooltip".to_string()),
            item_type: MenuItemType::Action,
            state: MenuItemState::default(),
        };

        assert_eq!(item.text, "Test Item");
        assert!(item.enabled);
        assert_eq!(item.keyboard_shortcut, Some("Ctrl+S".to_string()));
    }
}