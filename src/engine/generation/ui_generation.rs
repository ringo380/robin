/*!
 * Robin Engine Dynamic UI Generation System
 * 
 * A comprehensive system for procedurally generating user interface elements,
 * menus, HUDs, and interactive components with adaptive layouts and theming.
 */

use crate::engine::{
    graphics::{Texture, Color},
    math::{Vec2, Vec3, Transform},
    error::{RobinError, RobinResult},
};
use super::templates::{UITheme, UIComponent, UIComponentType, UILayoutProperties, UIAnchor, UISpacing, UILayout};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main UI generation system
#[derive(Debug)]
pub struct UIGenerator {
    config: UIGenerationConfig,
    themes: HashMap<String, UITheme>,
    templates: HashMap<String, UIElementTemplate>,
    layouts: HashMap<String, LayoutSystem>,
    style_cache: HashMap<String, ComputedStyle>,
    animation_presets: HashMap<String, AnimationPreset>,
}

impl UIGenerator {
    pub fn new(config: UIGenerationConfig) -> Self {
        let mut generator = Self {
            config,
            themes: HashMap::new(),
            templates: HashMap::new(),
            layouts: HashMap::new(),
            style_cache: HashMap::new(),
            animation_presets: HashMap::new(),
        };

        generator.load_default_themes();
        generator.load_default_templates();
        generator.load_default_layouts();
        generator.load_animation_presets();
        generator
    }

    /// Generate a complete UI element
    pub fn generate_element(&mut self, params: UIGenerationParams) -> RobinResult<GeneratedUIElement> {
        let theme = self.get_theme(&params.theme_name)?;
        let template = self.get_template(params.element_type.clone())?;
        
        let element = match params.element_type {
            UIElementType::Button => self.generate_button(params, theme, template)?,
            UIElementType::Panel => self.generate_panel(params, theme, template)?,
            UIElementType::Menu => self.generate_menu(params, theme, template)?,
            UIElementType::HUD => self.generate_hud(params, theme, template)?,
            UIElementType::Dialog => self.generate_dialog(params, theme, template)?,
            UIElementType::ProgressBar => self.generate_progress_bar(params, theme, template)?,
            UIElementType::Slider => self.generate_slider(params, theme, template)?,
            UIElementType::TextInput => self.generate_text_input(params, theme, template)?,
            UIElementType::Inventory => self.generate_inventory(params, theme, template)?,
            UIElementType::MiniMap => self.generate_minimap(params, theme, template)?,
            UIElementType::SkillTree => self.generate_skill_tree(params, theme, template)?,
            UIElementType::ChatBox => self.generate_chat_box(params, theme, template)?,
        };

        Ok(element)
    }

    /// Generate a complete UI layout
    pub fn generate_layout(&mut self, params: LayoutGenerationParams) -> RobinResult<GeneratedUILayout> {
        // Clone the data we need to avoid borrow conflicts
        let layout_system = self.get_layout(params.layout_type.clone())?.clone();
        let theme = self.get_theme(&params.theme_name)?.clone();

        let mut elements = Vec::new();
        let mut positions = Vec::new();

        for element_spec in &params.elements {
            let element = self.generate_element(UIGenerationParams {
                element_type: element_spec.element_type,
                dimensions: Some(element_spec.dimensions),
                theme_name: params.theme_name.clone(),
                customization: element_spec.customization.clone(),
                adaptive: params.adaptive,
                responsive_breakpoints: params.responsive_breakpoints.clone(),
            })?;

            let position = layout_system.calculate_position(
                &element,
                &element_spec.constraints,
                &params.container_size,
            )?;

            elements.push(element);
            positions.push(position);
        }

        Ok(GeneratedUILayout {
            elements,
            positions,
            layout_type: params.layout_type,
            container_size: params.container_size,
            adaptive_rules: self.generate_adaptive_rules(&params)?,
        })
    }

    /// Generate theme-aware styles
    pub fn generate_styles(&self, element_type: UIElementType, theme: &UITheme, customization: &Option<UICustomization>) -> RobinResult<ComputedStyle> {
        let base_style = self.get_base_style(element_type, theme)?;
        
        let mut computed_style = base_style.clone();
        
        if let Some(custom) = customization {
            self.apply_customization(&mut computed_style, custom)?;
        }

        Ok(computed_style)
    }

    /// Generate adaptive UI that responds to screen size
    pub fn generate_adaptive_ui(&mut self, params: AdaptiveUIParams) -> RobinResult<GeneratedAdaptiveUI> {
        let mut breakpoint_layouts = HashMap::new();

        for breakpoint in &params.breakpoints {
            let layout_params = LayoutGenerationParams {
                layout_type: self.select_layout_for_breakpoint(breakpoint, &params.preferred_layout)?,
                elements: self.adapt_elements_for_breakpoint(&params.base_elements, breakpoint)?,
                container_size: breakpoint.screen_size,
                theme_name: params.theme_name.clone(),
                adaptive: true,
                responsive_breakpoints: params.breakpoints.clone(),
            };

            let layout = self.generate_layout(layout_params)?;
            breakpoint_layouts.insert(breakpoint.name.clone(), layout);
        }

        Ok(GeneratedAdaptiveUI {
            breakpoint_layouts,
            transition_animations: self.generate_transition_animations(&params.breakpoints)?,
            adaptive_rules: self.generate_global_adaptive_rules(&params)?,
        })
    }

    /// Generate interactive UI with animations
    pub fn generate_interactive_element(&mut self, params: InteractiveUIParams) -> RobinResult<GeneratedInteractiveElement> {
        // Clone params for later use before partial move
        let params_clone = params.clone();
        
        let base_element = self.generate_element(params.base_params)?;
        
        let states = self.generate_interaction_states(&params_clone)?;
        let animations = self.generate_state_animations(&params_clone, &states)?;
        let event_handlers = self.generate_event_handlers(&params_clone)?;

        Ok(GeneratedInteractiveElement {
            base_element,
            states,
            animations,
            event_handlers,
            interaction_config: params.interaction_config,
        })
    }

    fn load_default_themes(&mut self) {
        // Fantasy theme
        self.themes.insert("fantasy".to_string(), UITheme::Fantasy);
        
        // Sci-fi theme
        self.themes.insert("scifi".to_string(), UITheme::SciFi);
        
        // Minimal theme
        self.themes.insert("minimal".to_string(), UITheme::Minimal);
        
        // Dark theme
        self.themes.insert("dark".to_string(), UITheme::Dark);
        
        // Retro theme
        self.themes.insert("retro".to_string(), UITheme::Retro);
    }

    fn load_default_templates(&mut self) {
        // Button templates
        self.templates.insert("button".to_string(), UIElementTemplate {
            name: "Standard Button".to_string(),
            element_type: UIElementType::Button,
            base_dimensions: Vec2::new(120.0, 40.0),
            style_properties: vec![
                ("border-radius".to_string(), "8px".to_string()),
                ("padding".to_string(), "8px 16px".to_string()),
                ("font-weight".to_string(), "bold".to_string()),
            ],
            animations: vec!["hover_scale".to_string(), "click_feedback".to_string()],
            variants: vec![
                ("primary".to_string(), vec![("color".to_string(), "primary".to_string())]),
                ("secondary".to_string(), vec![("color".to_string(), "secondary".to_string())]),
                ("danger".to_string(), vec![("color".to_string(), "danger".to_string())]),
            ],
        });

        // Panel templates
        self.templates.insert("panel".to_string(), UIElementTemplate {
            name: "Standard Panel".to_string(),
            element_type: UIElementType::Panel,
            base_dimensions: Vec2::new(300.0, 200.0),
            style_properties: vec![
                ("background".to_string(), "panel_bg".to_string()),
                ("border".to_string(), "panel_border".to_string()),
                ("padding".to_string(), "16px".to_string()),
            ],
            animations: vec!["fade_in".to_string()],
            variants: vec![
                ("modal".to_string(), vec![("backdrop".to_string(), "true".to_string())]),
                ("floating".to_string(), vec![("shadow".to_string(), "large".to_string())]),
            ],
        });

        // Menu templates
        self.templates.insert("menu".to_string(), UIElementTemplate {
            name: "Navigation Menu".to_string(),
            element_type: UIElementType::Menu,
            base_dimensions: Vec2::new(200.0, 300.0),
            style_properties: vec![
                ("background".to_string(), "menu_bg".to_string()),
                ("item-height".to_string(), "40px".to_string()),
            ],
            animations: vec!["slide_in".to_string(), "item_highlight".to_string()],
            variants: vec![
                ("horizontal".to_string(), vec![("direction".to_string(), "row".to_string())]),
                ("vertical".to_string(), vec![("direction".to_string(), "column".to_string())]),
            ],
        });

        // HUD templates
        self.templates.insert("hud".to_string(), UIElementTemplate {
            name: "Game HUD".to_string(),
            element_type: UIElementType::HUD,
            base_dimensions: Vec2::new(1920.0, 1080.0),
            style_properties: vec![
                ("overlay".to_string(), "true".to_string()),
                ("pointer-events".to_string(), "none".to_string()),
            ],
            animations: vec!["health_pulse".to_string(), "mana_flow".to_string()],
            variants: vec![
                ("compact".to_string(), vec![("scale".to_string(), "0.8".to_string())]),
                ("expanded".to_string(), vec![("scale".to_string(), "1.2".to_string())]),
            ],
        });
    }

    fn load_default_layouts(&mut self) {
        // Grid layout
        self.layouts.insert("grid".to_string(), LayoutSystem {
            layout_type: UILayout::Grid,
            spacing: Vec2::new(8.0, 8.0),
            alignment: LayoutAlignment::Start,
            wrap: true,
            responsive_rules: vec![
                ResponsiveRule {
                    breakpoint: "mobile".to_string(),
                    columns: 1,
                    spacing_override: Some(Vec2::new(4.0, 4.0)),
                },
                ResponsiveRule {
                    breakpoint: "tablet".to_string(),
                    columns: 2,
                    spacing_override: None,
                },
                ResponsiveRule {
                    breakpoint: "desktop".to_string(),
                    columns: 3,
                    spacing_override: None,
                },
            ],
        });

        // Flex layout
        self.layouts.insert("flex".to_string(), LayoutSystem {
            layout_type: UILayout::Flex,
            spacing: Vec2::new(12.0, 12.0),
            alignment: LayoutAlignment::Center,
            wrap: false,
            responsive_rules: vec![],
        });

        // Stack layout
        self.layouts.insert("stack".to_string(), LayoutSystem {
            layout_type: UILayout::Stack,
            spacing: Vec2::new(0.0, 8.0),
            alignment: LayoutAlignment::Stretch,
            wrap: false,
            responsive_rules: vec![],
        });
    }

    fn load_animation_presets(&mut self) {
        // Hover effects
        self.animation_presets.insert("hover_scale".to_string(), AnimationPreset {
            name: "Hover Scale".to_string(),
            duration: 0.2,
            easing: AnimationEasing::EaseOut,
            properties: vec![
                AnimatedProperty {
                    property: "scale".to_string(),
                    from_value: "1.0".to_string(),
                    to_value: "1.05".to_string(),
                },
            ],
            trigger: AnimationTrigger::Hover,
        });

        // Click feedback
        self.animation_presets.insert("click_feedback".to_string(), AnimationPreset {
            name: "Click Feedback".to_string(),
            duration: 0.1,
            easing: AnimationEasing::EaseInOut,
            properties: vec![
                AnimatedProperty {
                    property: "scale".to_string(),
                    from_value: "1.0".to_string(),
                    to_value: "0.95".to_string(),
                },
            ],
            trigger: AnimationTrigger::Click,
        });

        // Fade in
        self.animation_presets.insert("fade_in".to_string(), AnimationPreset {
            name: "Fade In".to_string(),
            duration: 0.3,
            easing: AnimationEasing::EaseOut,
            properties: vec![
                AnimatedProperty {
                    property: "opacity".to_string(),
                    from_value: "0.0".to_string(),
                    to_value: "1.0".to_string(),
                },
            ],
            trigger: AnimationTrigger::Show,
        });
    }

    fn generate_button(&self, params: UIGenerationParams, theme: &UITheme, template: &UIElementTemplate) -> RobinResult<GeneratedUIElement> {
        let style = self.generate_styles(UIElementType::Button, theme, &params.customization)?;
        
        Ok(GeneratedUIElement {
            element_type: UIElementType::Button,
            dimensions: params.dimensions.unwrap_or(template.base_dimensions),
            style,
            content: UIContent::Text("Button".to_string()),
            children: vec![],
            interaction_handlers: self.generate_button_handlers()?,
            animations: self.get_template_animations(template)?,
            accessibility: self.generate_accessibility_attributes(UIElementType::Button)?,
        })
    }

    fn generate_panel(&self, params: UIGenerationParams, theme: &UITheme, template: &UIElementTemplate) -> RobinResult<GeneratedUIElement> {
        let style = self.generate_styles(UIElementType::Panel, theme, &params.customization)?;
        
        Ok(GeneratedUIElement {
            element_type: UIElementType::Panel,
            dimensions: params.dimensions.unwrap_or(template.base_dimensions),
            style,
            content: UIContent::Container,
            children: vec![],
            interaction_handlers: vec![],
            animations: self.get_template_animations(template)?,
            accessibility: self.generate_accessibility_attributes(UIElementType::Panel)?,
        })
    }

    fn generate_menu(&self, params: UIGenerationParams, theme: &UITheme, template: &UIElementTemplate) -> RobinResult<GeneratedUIElement> {
        let style = self.generate_styles(UIElementType::Menu, theme, &params.customization)?;
        
        // Generate menu items
        let mut menu_items = vec![];
        if let Some(UICustomization { menu_items: Some(items), .. }) = &params.customization {
            for item in items {
                menu_items.push(self.generate_menu_item(item, theme)?);
            }
        }
        
        Ok(GeneratedUIElement {
            element_type: UIElementType::Menu,
            dimensions: params.dimensions.unwrap_or(template.base_dimensions),
            style,
            content: UIContent::Menu(menu_items.clone()),
            children: menu_items,
            interaction_handlers: self.generate_menu_handlers()?,
            animations: self.get_template_animations(template)?,
            accessibility: self.generate_accessibility_attributes(UIElementType::Menu)?,
        })
    }

    fn generate_hud(&self, params: UIGenerationParams, theme: &UITheme, template: &UIElementTemplate) -> RobinResult<GeneratedUIElement> {
        let style = self.generate_styles(UIElementType::HUD, theme, &params.customization)?;
        
        // Generate HUD components
        let mut hud_components = vec![];
        
        // Health bar
        hud_components.push(self.generate_health_bar(theme)?);
        
        // Mana bar
        hud_components.push(self.generate_mana_bar(theme)?);
        
        // Mini-map
        hud_components.push(self.generate_minimap_component(theme)?);
        
        // Inventory quick bar
        hud_components.push(self.generate_inventory_quick_bar(theme)?);
        
        Ok(GeneratedUIElement {
            element_type: UIElementType::HUD,
            dimensions: params.dimensions.unwrap_or(template.base_dimensions),
            style,
            content: UIContent::HUD(hud_components.clone()),
            children: hud_components,
            interaction_handlers: self.generate_hud_handlers()?,
            animations: self.get_template_animations(template)?,
            accessibility: self.generate_accessibility_attributes(UIElementType::HUD)?,
        })
    }

    fn generate_dialog(&self, params: UIGenerationParams, theme: &UITheme, template: &UIElementTemplate) -> RobinResult<GeneratedUIElement> {
        let style = self.generate_styles(UIElementType::Dialog, theme, &params.customization)?;
        
        Ok(GeneratedUIElement {
            element_type: UIElementType::Dialog,
            dimensions: params.dimensions.unwrap_or(template.base_dimensions),
            style,
            content: UIContent::Text("Dialog Content".to_string()),
            children: vec![],
            interaction_handlers: self.generate_dialog_handlers()?,
            animations: self.get_template_animations(template)?,
            accessibility: self.generate_accessibility_attributes(UIElementType::Dialog)?,
        })
    }

    fn generate_progress_bar(&self, params: UIGenerationParams, theme: &UITheme, template: &UIElementTemplate) -> RobinResult<GeneratedUIElement> {
        let style = self.generate_styles(UIElementType::ProgressBar, theme, &params.customization)?;
        
        Ok(GeneratedUIElement {
            element_type: UIElementType::ProgressBar,
            dimensions: params.dimensions.unwrap_or(Vec2::new(200.0, 20.0)),
            style,
            content: UIContent::ProgressBar { value: 0.5, max_value: 1.0 },
            children: vec![],
            interaction_handlers: vec![],
            animations: vec!["progress_fill".to_string()],
            accessibility: self.generate_accessibility_attributes(UIElementType::ProgressBar)?,
        })
    }

    fn generate_slider(&self, params: UIGenerationParams, theme: &UITheme, template: &UIElementTemplate) -> RobinResult<GeneratedUIElement> {
        let style = self.generate_styles(UIElementType::Slider, theme, &params.customization)?;
        
        Ok(GeneratedUIElement {
            element_type: UIElementType::Slider,
            dimensions: params.dimensions.unwrap_or(Vec2::new(200.0, 20.0)),
            style,
            content: UIContent::Slider { value: 0.5, min_value: 0.0, max_value: 1.0 },
            children: vec![],
            interaction_handlers: self.generate_slider_handlers()?,
            animations: vec!["handle_highlight".to_string()],
            accessibility: self.generate_accessibility_attributes(UIElementType::Slider)?,
        })
    }

    fn generate_text_input(&self, params: UIGenerationParams, theme: &UITheme, template: &UIElementTemplate) -> RobinResult<GeneratedUIElement> {
        let style = self.generate_styles(UIElementType::TextInput, theme, &params.customization)?;
        
        Ok(GeneratedUIElement {
            element_type: UIElementType::TextInput,
            dimensions: params.dimensions.unwrap_or(Vec2::new(200.0, 32.0)),
            style,
            content: UIContent::TextInput { value: String::new(), placeholder: "Enter text...".to_string() },
            children: vec![],
            interaction_handlers: self.generate_text_input_handlers()?,
            animations: vec!["focus_glow".to_string()],
            accessibility: self.generate_accessibility_attributes(UIElementType::TextInput)?,
        })
    }

    fn generate_inventory(&self, params: UIGenerationParams, theme: &UITheme, template: &UIElementTemplate) -> RobinResult<GeneratedUIElement> {
        let style = self.generate_styles(UIElementType::Inventory, theme, &params.customization)?;
        
        // Generate inventory grid
        let mut inventory_slots = vec![];
        let slot_count = 20; // Default inventory size
        
        for i in 0..slot_count {
            inventory_slots.push(self.generate_inventory_slot(i, theme)?);
        }
        
        Ok(GeneratedUIElement {
            element_type: UIElementType::Inventory,
            dimensions: params.dimensions.unwrap_or(Vec2::new(400.0, 300.0)),
            style,
            content: UIContent::Inventory(inventory_slots.clone()),
            children: inventory_slots,
            interaction_handlers: self.generate_inventory_handlers()?,
            animations: vec!["item_pickup".to_string(), "slot_highlight".to_string()],
            accessibility: self.generate_accessibility_attributes(UIElementType::Inventory)?,
        })
    }

    fn generate_minimap(&self, params: UIGenerationParams, theme: &UITheme, template: &UIElementTemplate) -> RobinResult<GeneratedUIElement> {
        let style = self.generate_styles(UIElementType::MiniMap, theme, &params.customization)?;
        
        Ok(GeneratedUIElement {
            element_type: UIElementType::MiniMap,
            dimensions: params.dimensions.unwrap_or(Vec2::new(150.0, 150.0)),
            style,
            content: UIContent::MiniMap,
            children: vec![],
            interaction_handlers: self.generate_minimap_handlers()?,
            animations: vec!["radar_sweep".to_string()],
            accessibility: self.generate_accessibility_attributes(UIElementType::MiniMap)?,
        })
    }

    fn generate_skill_tree(&self, params: UIGenerationParams, theme: &UITheme, template: &UIElementTemplate) -> RobinResult<GeneratedUIElement> {
        let style = self.generate_styles(UIElementType::SkillTree, theme, &params.customization)?;
        
        Ok(GeneratedUIElement {
            element_type: UIElementType::SkillTree,
            dimensions: params.dimensions.unwrap_or(Vec2::new(600.0, 400.0)),
            style,
            content: UIContent::SkillTree,
            children: vec![],
            interaction_handlers: self.generate_skill_tree_handlers()?,
            animations: vec!["skill_unlock".to_string(), "connection_glow".to_string()],
            accessibility: self.generate_accessibility_attributes(UIElementType::SkillTree)?,
        })
    }

    fn generate_chat_box(&self, params: UIGenerationParams, theme: &UITheme, template: &UIElementTemplate) -> RobinResult<GeneratedUIElement> {
        let style = self.generate_styles(UIElementType::ChatBox, theme, &params.customization)?;
        
        Ok(GeneratedUIElement {
            element_type: UIElementType::ChatBox,
            dimensions: params.dimensions.unwrap_or(Vec2::new(400.0, 200.0)),
            style,
            content: UIContent::ChatBox { messages: vec![], input_value: String::new() },
            children: vec![],
            interaction_handlers: self.generate_chat_handlers()?,
            animations: vec!["message_slide_in".to_string()],
            accessibility: self.generate_accessibility_attributes(UIElementType::ChatBox)?,
        })
    }

    fn get_theme(&self, theme_name: &str) -> RobinResult<&UITheme> {
        self.themes.get(theme_name)
            .ok_or_else(|| RobinError::UIGenerationError(format!("Theme not found: {}", theme_name)))
    }

    fn get_template(&self, element_type: UIElementType) -> RobinResult<&UIElementTemplate> {
        let template_name = match element_type {
            UIElementType::Button => "button",
            UIElementType::Panel => "panel",
            UIElementType::Menu => "menu",
            UIElementType::HUD => "hud",
            _ => "button", // Default fallback
        };

        self.templates.get(template_name)
            .ok_or_else(|| RobinError::UIGenerationError(format!("Template not found: {}", template_name)))
    }

    fn get_layout(&self, layout_type: UILayout) -> RobinResult<&LayoutSystem> {
        let layout_name = match layout_type {
            UILayout::Grid => "grid",
            UILayout::Flex => "flex",
            UILayout::Stack => "stack",
            UILayout::Fixed => "fixed",
        };

        self.layouts.get(layout_name)
            .ok_or_else(|| RobinError::UIGenerationError(format!("Layout not found: {}", layout_name)))
    }

    fn get_base_style(&self, element_type: UIElementType, theme: &UITheme) -> RobinResult<ComputedStyle> {
        // Generate base style based on element type and theme
        let base_colors = self.get_theme_colors(theme);
        let base_fonts = self.get_theme_fonts(theme);
        
        Ok(ComputedStyle {
            background_color: base_colors.background,
            text_color: base_colors.text,
            border_color: base_colors.border,
            font_family: base_fonts.primary,
            font_size: 14.0,
            padding: UISpacing { left: 8.0, right: 8.0, top: 8.0, bottom: 8.0 },
            margin: UISpacing { left: 0.0, right: 0.0, top: 0.0, bottom: 0.0 },
            border_radius: 4.0,
            border_width: 1.0,
            opacity: 1.0,
            shadow: None,
        })
    }

    fn get_theme_colors(&self, theme: &UITheme) -> ThemeColors {
        match theme {
            UITheme::Fantasy => ThemeColors {
                background: Color::new(0.2, 0.15, 0.1, 1.0),
                text: Color::new(0.9, 0.85, 0.7, 1.0),
                border: Color::new(0.6, 0.4, 0.2, 1.0),
                primary: Color::new(0.8, 0.6, 0.2, 1.0),
                secondary: Color::new(0.4, 0.6, 0.8, 1.0),
                accent: Color::new(0.9, 0.3, 0.3, 1.0),
            },
            UITheme::SciFi => ThemeColors {
                background: Color::new(0.1, 0.1, 0.15, 1.0),
                text: Color::new(0.2, 0.8, 1.0, 1.0),
                border: Color::new(0.3, 0.6, 0.9, 1.0),
                primary: Color::new(0.0, 0.7, 1.0, 1.0),
                secondary: Color::new(0.5, 0.5, 0.5, 1.0),
                accent: Color::new(1.0, 0.3, 0.0, 1.0),
            },
            UITheme::Dark => ThemeColors {
                background: Color::new(0.15, 0.15, 0.15, 1.0),
                text: Color::new(0.9, 0.9, 0.9, 1.0),
                border: Color::new(0.3, 0.3, 0.3, 1.0),
                primary: Color::new(0.2, 0.6, 1.0, 1.0),
                secondary: Color::new(0.5, 0.5, 0.5, 1.0),
                accent: Color::new(0.9, 0.9, 0.1, 1.0),
            },
            _ => ThemeColors {
                background: Color::new(0.95, 0.95, 0.95, 1.0),
                text: Color::new(0.1, 0.1, 0.1, 1.0),
                border: Color::new(0.7, 0.7, 0.7, 1.0),
                primary: Color::new(0.2, 0.5, 0.8, 1.0),
                secondary: Color::new(0.5, 0.5, 0.5, 1.0),
                accent: Color::new(0.8, 0.2, 0.2, 1.0),
            },
        }
    }

    fn get_theme_fonts(&self, theme: &UITheme) -> ThemeFonts {
        match theme {
            UITheme::Fantasy => ThemeFonts {
                primary: "Cinzel".to_string(),
                secondary: "Almendra".to_string(),
                monospace: "Uncial Antiqua".to_string(),
            },
            UITheme::SciFi => ThemeFonts {
                primary: "Orbitron".to_string(),
                secondary: "Exo".to_string(),
                monospace: "Share Tech Mono".to_string(),
            },
            UITheme::Retro => ThemeFonts {
                primary: "Press Start 2P".to_string(),
                secondary: "VT323".to_string(),
                monospace: "Courier Prime".to_string(),
            },
            _ => ThemeFonts {
                primary: "Inter".to_string(),
                secondary: "Source Sans Pro".to_string(),
                monospace: "Source Code Pro".to_string(),
            },
        }
    }

    fn apply_customization(&self, style: &mut ComputedStyle, customization: &UICustomization) -> RobinResult<()> {
        if let Some(bg_color) = customization.background_color {
            style.background_color = bg_color;
        }
        
        if let Some(text_color) = customization.text_color {
            style.text_color = text_color;
        }
        
        if let Some(font_size) = customization.font_size {
            style.font_size = font_size;
        }
        
        Ok(())
    }

    fn generate_button_handlers(&self) -> RobinResult<Vec<InteractionHandler>> {
        Ok(vec![
            InteractionHandler {
                event_type: UIEventType::Click,
                action: UIAction::TriggerAnimation("click_feedback".to_string()),
            },
            InteractionHandler {
                event_type: UIEventType::Hover,
                action: UIAction::TriggerAnimation("hover_scale".to_string()),
            },
        ])
    }

    fn generate_menu_handlers(&self) -> RobinResult<Vec<InteractionHandler>> {
        Ok(vec![
            InteractionHandler {
                event_type: UIEventType::Click,
                action: UIAction::Navigate,
            },
        ])
    }

    fn generate_hud_handlers(&self) -> RobinResult<Vec<InteractionHandler>> {
        Ok(vec![
            InteractionHandler {
                event_type: UIEventType::Update,
                action: UIAction::UpdateValues,
            },
        ])
    }

    fn generate_dialog_handlers(&self) -> RobinResult<Vec<InteractionHandler>> {
        Ok(vec![
            InteractionHandler {
                event_type: UIEventType::KeyPress,
                action: UIAction::Close,
            },
        ])
    }

    fn generate_slider_handlers(&self) -> RobinResult<Vec<InteractionHandler>> {
        Ok(vec![
            InteractionHandler {
                event_type: UIEventType::Drag,
                action: UIAction::UpdateValue,
            },
        ])
    }

    fn generate_text_input_handlers(&self) -> RobinResult<Vec<InteractionHandler>> {
        Ok(vec![
            InteractionHandler {
                event_type: UIEventType::Focus,
                action: UIAction::TriggerAnimation("focus_glow".to_string()),
            },
            InteractionHandler {
                event_type: UIEventType::KeyPress,
                action: UIAction::UpdateText,
            },
        ])
    }

    fn generate_inventory_handlers(&self) -> RobinResult<Vec<InteractionHandler>> {
        Ok(vec![
            InteractionHandler {
                event_type: UIEventType::Click,
                action: UIAction::SelectItem,
            },
            InteractionHandler {
                event_type: UIEventType::Drag,
                action: UIAction::MoveItem,
            },
        ])
    }

    fn generate_minimap_handlers(&self) -> RobinResult<Vec<InteractionHandler>> {
        Ok(vec![
            InteractionHandler {
                event_type: UIEventType::Click,
                action: UIAction::Navigate,
            },
        ])
    }

    fn generate_skill_tree_handlers(&self) -> RobinResult<Vec<InteractionHandler>> {
        Ok(vec![
            InteractionHandler {
                event_type: UIEventType::Click,
                action: UIAction::UnlockSkill,
            },
        ])
    }

    fn generate_chat_handlers(&self) -> RobinResult<Vec<InteractionHandler>> {
        Ok(vec![
            InteractionHandler {
                event_type: UIEventType::KeyPress,
                action: UIAction::SendMessage,
            },
        ])
    }

    fn get_template_animations(&self, template: &UIElementTemplate) -> RobinResult<Vec<String>> {
        Ok(template.animations.clone())
    }

    fn generate_accessibility_attributes(&self, element_type: UIElementType) -> RobinResult<AccessibilityAttributes> {
        Ok(AccessibilityAttributes {
            role: self.get_aria_role(element_type),
            label: self.get_default_label(element_type),
            description: None,
            keyboard_navigation: true,
            screen_reader_friendly: true,
        })
    }

    fn get_aria_role(&self, element_type: UIElementType) -> String {
        match element_type {
            UIElementType::Button => "button".to_string(),
            UIElementType::Menu => "menu".to_string(),
            UIElementType::Dialog => "dialog".to_string(),
            UIElementType::ProgressBar => "progressbar".to_string(),
            UIElementType::Slider => "slider".to_string(),
            UIElementType::TextInput => "textbox".to_string(),
            _ => "generic".to_string(),
        }
    }

    fn get_default_label(&self, element_type: UIElementType) -> String {
        match element_type {
            UIElementType::Button => "Button".to_string(),
            UIElementType::Menu => "Navigation Menu".to_string(),
            UIElementType::Dialog => "Dialog".to_string(),
            UIElementType::ProgressBar => "Progress Bar".to_string(),
            UIElementType::Slider => "Slider".to_string(),
            UIElementType::TextInput => "Text Input".to_string(),
            _ => "UI Element".to_string(),
        }
    }

    // Helper methods for specific components
    fn generate_menu_item(&self, item_spec: &MenuItemSpec, theme: &UITheme) -> RobinResult<GeneratedUIElement> {
        let style = self.get_base_style(UIElementType::Button, theme)?;
        
        Ok(GeneratedUIElement {
            element_type: UIElementType::Button,
            dimensions: Vec2::new(180.0, 32.0),
            style,
            content: UIContent::Text(item_spec.label.clone()),
            children: vec![],
            interaction_handlers: vec![
                InteractionHandler {
                    event_type: UIEventType::Click,
                    action: UIAction::Navigate,
                },
            ],
            animations: vec!["item_highlight".to_string()],
            accessibility: AccessibilityAttributes {
                role: "menuitem".to_string(),
                label: item_spec.label.clone(),
                description: None,
                keyboard_navigation: true,
                screen_reader_friendly: true,
            },
        })
    }

    fn generate_health_bar(&self, theme: &UITheme) -> RobinResult<GeneratedUIElement> {
        let mut style = self.get_base_style(UIElementType::ProgressBar, theme)?;
        style.background_color = Color::new(0.2, 0.0, 0.0, 0.8);
        
        Ok(GeneratedUIElement {
            element_type: UIElementType::ProgressBar,
            dimensions: Vec2::new(200.0, 20.0),
            style,
            content: UIContent::ProgressBar { value: 1.0, max_value: 1.0 },
            children: vec![],
            interaction_handlers: vec![],
            animations: vec!["health_pulse".to_string()],
            accessibility: AccessibilityAttributes {
                role: "progressbar".to_string(),
                label: "Health".to_string(),
                description: Some("Player health bar".to_string()),
                keyboard_navigation: false,
                screen_reader_friendly: true,
            },
        })
    }

    fn generate_mana_bar(&self, theme: &UITheme) -> RobinResult<GeneratedUIElement> {
        let mut style = self.get_base_style(UIElementType::ProgressBar, theme)?;
        style.background_color = Color::new(0.0, 0.0, 0.5, 0.8);
        
        Ok(GeneratedUIElement {
            element_type: UIElementType::ProgressBar,
            dimensions: Vec2::new(200.0, 20.0),
            style,
            content: UIContent::ProgressBar { value: 1.0, max_value: 1.0 },
            children: vec![],
            interaction_handlers: vec![],
            animations: vec!["mana_flow".to_string()],
            accessibility: AccessibilityAttributes {
                role: "progressbar".to_string(),
                label: "Mana".to_string(),
                description: Some("Player mana bar".to_string()),
                keyboard_navigation: false,
                screen_reader_friendly: true,
            },
        })
    }

    fn generate_minimap_component(&self, theme: &UITheme) -> RobinResult<GeneratedUIElement> {
        let style = self.get_base_style(UIElementType::MiniMap, theme)?;
        
        Ok(GeneratedUIElement {
            element_type: UIElementType::MiniMap,
            dimensions: Vec2::new(150.0, 150.0),
            style,
            content: UIContent::MiniMap,
            children: vec![],
            interaction_handlers: vec![],
            animations: vec!["radar_sweep".to_string()],
            accessibility: AccessibilityAttributes {
                role: "img".to_string(),
                label: "Mini Map".to_string(),
                description: Some("Miniature view of the game world".to_string()),
                keyboard_navigation: false,
                screen_reader_friendly: true,
            },
        })
    }

    fn generate_inventory_quick_bar(&self, theme: &UITheme) -> RobinResult<GeneratedUIElement> {
        let style = self.get_base_style(UIElementType::Inventory, theme)?;
        
        Ok(GeneratedUIElement {
            element_type: UIElementType::Inventory,
            dimensions: Vec2::new(400.0, 50.0),
            style,
            content: UIContent::Inventory(vec![]),
            children: vec![],
            interaction_handlers: vec![
                InteractionHandler {
                    event_type: UIEventType::Click,
                    action: UIAction::UseItem,
                },
            ],
            animations: vec!["slot_highlight".to_string()],
            accessibility: AccessibilityAttributes {
                role: "toolbar".to_string(),
                label: "Quick Access Bar".to_string(),
                description: Some("Quick access inventory slots".to_string()),
                keyboard_navigation: true,
                screen_reader_friendly: true,
            },
        })
    }

    fn generate_inventory_slot(&self, index: usize, theme: &UITheme) -> RobinResult<GeneratedUIElement> {
        let style = self.get_base_style(UIElementType::Button, theme)?;
        
        Ok(GeneratedUIElement {
            element_type: UIElementType::Button,
            dimensions: Vec2::new(40.0, 40.0),
            style,
            content: UIContent::InventorySlot { item: None, index },
            children: vec![],
            interaction_handlers: vec![
                InteractionHandler {
                    event_type: UIEventType::Click,
                    action: UIAction::SelectItem,
                },
            ],
            animations: vec!["slot_highlight".to_string()],
            accessibility: AccessibilityAttributes {
                role: "button".to_string(),
                label: format!("Inventory Slot {}", index + 1),
                description: Some("Item storage slot".to_string()),
                keyboard_navigation: true,
                screen_reader_friendly: true,
            },
        })
    }

    fn generate_adaptive_rules(&self, params: &LayoutGenerationParams) -> RobinResult<Vec<AdaptiveRule>> {
        Ok(vec![
            AdaptiveRule {
                condition: AdaptiveCondition::ScreenWidth(768.0),
                action: AdaptiveAction::SwitchLayout(UILayout::Stack),
            },
            AdaptiveRule {
                condition: AdaptiveCondition::ScreenHeight(600.0),
                action: AdaptiveAction::AdjustSpacing(Vec2::new(4.0, 4.0)),
            },
        ])
    }

    fn select_layout_for_breakpoint(&self, breakpoint: &ResponsiveBreakpoint, preferred: &UILayout) -> RobinResult<UILayout> {
        match breakpoint.name.as_str() {
            "mobile" => Ok(UILayout::Stack),
            "tablet" => Ok(UILayout::Flex),
            "desktop" => Ok(*preferred),
            _ => Ok(*preferred),
        }
    }

    fn adapt_elements_for_breakpoint(&self, base_elements: &[ElementSpec], breakpoint: &ResponsiveBreakpoint) -> RobinResult<Vec<ElementSpec>> {
        let mut adapted_elements = base_elements.to_vec();
        
        for element in &mut adapted_elements {
            // Scale dimensions based on screen size
            let scale_factor = (breakpoint.screen_size.x / 1920.0).min(1.0).max(0.5);
            element.dimensions = element.dimensions * scale_factor;
        }
        
        Ok(adapted_elements)
    }

    fn generate_transition_animations(&self, breakpoints: &[ResponsiveBreakpoint]) -> RobinResult<Vec<TransitionAnimation>> {
        Ok(vec![
            TransitionAnimation {
                from_breakpoint: "desktop".to_string(),
                to_breakpoint: "tablet".to_string(),
                duration: 0.3,
                easing: AnimationEasing::EaseInOut,
            },
            TransitionAnimation {
                from_breakpoint: "tablet".to_string(),
                to_breakpoint: "mobile".to_string(),
                duration: 0.3,
                easing: AnimationEasing::EaseInOut,
            },
        ])
    }

    fn generate_global_adaptive_rules(&self, params: &AdaptiveUIParams) -> RobinResult<Vec<AdaptiveRule>> {
        Ok(vec![
            AdaptiveRule {
                condition: AdaptiveCondition::ScreenWidth(480.0),
                action: AdaptiveAction::HideElements(vec!["secondary_navigation".to_string()]),
            },
        ])
    }

    fn generate_interaction_states(&self, params: &InteractiveUIParams) -> RobinResult<HashMap<String, UIState>> {
        let mut states = HashMap::new();
        
        states.insert("default".to_string(), UIState {
            style_overrides: HashMap::new(),
            visibility: true,
            enabled: true,
        });
        
        states.insert("hover".to_string(), UIState {
            style_overrides: {
                let mut overrides = HashMap::new();
                overrides.insert("opacity".to_string(), "0.8".to_string());
                overrides
            },
            visibility: true,
            enabled: true,
        });
        
        states.insert("disabled".to_string(), UIState {
            style_overrides: {
                let mut overrides = HashMap::new();
                overrides.insert("opacity".to_string(), "0.5".to_string());
                overrides
            },
            visibility: true,
            enabled: false,
        });
        
        Ok(states)
    }

    fn generate_state_animations(&self, params: &InteractiveUIParams, states: &HashMap<String, UIState>) -> RobinResult<HashMap<String, String>> {
        let mut animations = HashMap::new();
        
        animations.insert("default_to_hover".to_string(), "hover_scale".to_string());
        animations.insert("hover_to_default".to_string(), "hover_scale_reverse".to_string());
        animations.insert("any_to_disabled".to_string(), "fade_out_partial".to_string());
        
        Ok(animations)
    }

    fn generate_event_handlers(&self, params: &InteractiveUIParams) -> RobinResult<HashMap<String, String>> {
        let mut handlers = HashMap::new();
        
        handlers.insert("click".to_string(), "handle_click".to_string());
        handlers.insert("hover".to_string(), "handle_hover".to_string());
        handlers.insert("focus".to_string(), "handle_focus".to_string());
        
        Ok(handlers)
    }
}

// Configuration and parameter structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIGenerationConfig {
    pub default_theme: String,
    pub responsive_enabled: bool,
    pub animation_enabled: bool,
    pub accessibility_enabled: bool,
    pub cache_size: usize,
}

impl Default for UIGenerationConfig {
    fn default() -> Self {
        Self {
            default_theme: "minimal".to_string(),
            responsive_enabled: true,
            animation_enabled: true,
            accessibility_enabled: true,
            cache_size: 100,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UIGenerationParams {
    pub element_type: UIElementType,
    pub dimensions: Option<Vec2>,
    pub theme_name: String,
    pub customization: Option<UICustomization>,
    pub adaptive: bool,
    pub responsive_breakpoints: Vec<ResponsiveBreakpoint>,
}

#[derive(Debug, Clone)]
pub struct LayoutGenerationParams {
    pub layout_type: UILayout,
    pub elements: Vec<ElementSpec>,
    pub container_size: Vec2,
    pub theme_name: String,
    pub adaptive: bool,
    pub responsive_breakpoints: Vec<ResponsiveBreakpoint>,
}

#[derive(Debug, Clone)]
pub struct AdaptiveUIParams {
    pub base_elements: Vec<ElementSpec>,
    pub breakpoints: Vec<ResponsiveBreakpoint>,
    pub preferred_layout: UILayout,
    pub theme_name: String,
}

#[derive(Debug, Clone)]
pub struct InteractiveUIParams {
    pub base_params: UIGenerationParams,
    pub interaction_config: InteractionConfig,
}

// Core data structures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UIElementType {
    Button,
    Panel,
    Menu,
    HUD,
    Dialog,
    ProgressBar,
    Slider,
    TextInput,
    Inventory,
    MiniMap,
    SkillTree,
    ChatBox,
}

#[derive(Debug, Clone)]
pub struct GeneratedUIElement {
    pub element_type: UIElementType,
    pub dimensions: Vec2,
    pub style: ComputedStyle,
    pub content: UIContent,
    pub children: Vec<GeneratedUIElement>,
    pub interaction_handlers: Vec<InteractionHandler>,
    pub animations: Vec<String>,
    pub accessibility: AccessibilityAttributes,
}

#[derive(Debug, Clone)]
pub struct GeneratedUILayout {
    pub elements: Vec<GeneratedUIElement>,
    pub positions: Vec<Vec2>,
    pub layout_type: UILayout,
    pub container_size: Vec2,
    pub adaptive_rules: Vec<AdaptiveRule>,
}

#[derive(Debug, Clone)]
pub struct GeneratedAdaptiveUI {
    pub breakpoint_layouts: HashMap<String, GeneratedUILayout>,
    pub transition_animations: Vec<TransitionAnimation>,
    pub adaptive_rules: Vec<AdaptiveRule>,
}

#[derive(Debug, Clone)]
pub struct GeneratedInteractiveElement {
    pub base_element: GeneratedUIElement,
    pub states: HashMap<String, UIState>,
    pub animations: HashMap<String, String>,
    pub event_handlers: HashMap<String, String>,
    pub interaction_config: InteractionConfig,
}

// Style and theming structures
#[derive(Debug, Clone)]
pub struct ComputedStyle {
    pub background_color: Color,
    pub text_color: Color,
    pub border_color: Color,
    pub font_family: String,
    pub font_size: f32,
    pub padding: UISpacing,
    pub margin: UISpacing,
    pub border_radius: f32,
    pub border_width: f32,
    pub opacity: f32,
    pub shadow: Option<UIShadow>,
}

#[derive(Debug, Clone)]
pub struct ThemeColors {
    pub background: Color,
    pub text: Color,
    pub border: Color,
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
}

#[derive(Debug, Clone)]
pub struct ThemeFonts {
    pub primary: String,
    pub secondary: String,
    pub monospace: String,
}

#[derive(Debug, Clone)]
pub struct UIShadow {
    pub color: Color,
    pub offset: Vec2,
    pub blur: f32,
    pub spread: f32,
}

// Content types
#[derive(Debug, Clone)]
pub enum UIContent {
    Text(String),
    Image(String),
    Container,
    ProgressBar { value: f32, max_value: f32 },
    Slider { value: f32, min_value: f32, max_value: f32 },
    TextInput { value: String, placeholder: String },
    Menu(Vec<GeneratedUIElement>),
    HUD(Vec<GeneratedUIElement>),
    Inventory(Vec<GeneratedUIElement>),
    InventorySlot { item: Option<String>, index: usize },
    MiniMap,
    SkillTree,
    ChatBox { messages: Vec<String>, input_value: String },
}

// Templates and layouts
#[derive(Debug, Clone)]
pub struct UIElementTemplate {
    pub name: String,
    pub element_type: UIElementType,
    pub base_dimensions: Vec2,
    pub style_properties: Vec<(String, String)>,
    pub animations: Vec<String>,
    pub variants: Vec<(String, Vec<(String, String)>)>,
}

#[derive(Debug, Clone)]
pub struct LayoutSystem {
    pub layout_type: UILayout,
    pub spacing: Vec2,
    pub alignment: LayoutAlignment,
    pub wrap: bool,
    pub responsive_rules: Vec<ResponsiveRule>,
}

impl LayoutSystem {
    fn calculate_position(&self, element: &GeneratedUIElement, constraints: &LayoutConstraints, container_size: &Vec2) -> RobinResult<Vec2> {
        // Simple position calculation based on layout type
        match self.layout_type {
            UILayout::Fixed => Ok(constraints.position.unwrap_or(Vec2::new(0.0, 0.0))),
            UILayout::Flex => {
                let x = constraints.position.map(|p| p.x).unwrap_or(0.0);
                let y = constraints.position.map(|p| p.y).unwrap_or(0.0);
                Ok(Vec2::new(x, y))
            },
            UILayout::Grid => {
                // Grid positioning logic
                Ok(Vec2::new(0.0, 0.0))
            },
            UILayout::Stack => {
                // Stack positioning logic
                Ok(Vec2::new(0.0, 0.0))
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutAlignment {
    Start,
    Center,
    End,
    Stretch,
}

// Responsive design structures
#[derive(Debug, Clone)]
pub struct ResponsiveBreakpoint {
    pub name: String,
    pub screen_size: Vec2,
    pub min_width: f32,
    pub max_width: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct ResponsiveRule {
    pub breakpoint: String,
    pub columns: u32,
    pub spacing_override: Option<Vec2>,
}

#[derive(Debug, Clone)]
pub struct ElementSpec {
    pub element_type: UIElementType,
    pub dimensions: Vec2,
    pub constraints: LayoutConstraints,
    pub customization: Option<UICustomization>,
}

#[derive(Debug, Clone)]
pub struct LayoutConstraints {
    pub position: Option<Vec2>,
    pub anchor: Option<UIAnchor>,
    pub min_size: Option<Vec2>,
    pub max_size: Option<Vec2>,
    pub flex_grow: f32,
    pub flex_shrink: f32,
}

// Customization structures
#[derive(Debug, Clone)]
pub struct UICustomization {
    pub background_color: Option<Color>,
    pub text_color: Option<Color>,
    pub font_size: Option<f32>,
    pub menu_items: Option<Vec<MenuItemSpec>>,
}

#[derive(Debug, Clone)]
pub struct MenuItemSpec {
    pub label: String,
    pub action: String,
    pub icon: Option<String>,
}

// Interaction and animation structures
#[derive(Debug, Clone)]
pub struct InteractionHandler {
    pub event_type: UIEventType,
    pub action: UIAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UIEventType {
    Click,
    Hover,
    Focus,
    Blur,
    KeyPress,
    Drag,
    Update,
}

#[derive(Debug, Clone)]
pub enum UIAction {
    TriggerAnimation(String),
    Navigate,
    Close,
    UpdateValue,
    UpdateText,
    UpdateValues,
    SelectItem,
    MoveItem,
    UseItem,
    UnlockSkill,
    SendMessage,
}

#[derive(Debug, Clone)]
pub struct AnimationPreset {
    pub name: String,
    pub duration: f32,
    pub easing: AnimationEasing,
    pub properties: Vec<AnimatedProperty>,
    pub trigger: AnimationTrigger,
}

#[derive(Debug, Clone)]
pub struct AnimatedProperty {
    pub property: String,
    pub from_value: String,
    pub to_value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationEasing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationTrigger {
    Hover,
    Click,
    Focus,
    Show,
    Hide,
}

#[derive(Debug, Clone)]
pub struct TransitionAnimation {
    pub from_breakpoint: String,
    pub to_breakpoint: String,
    pub duration: f32,
    pub easing: AnimationEasing,
}

// Adaptive design structures
#[derive(Debug, Clone)]
pub struct AdaptiveRule {
    pub condition: AdaptiveCondition,
    pub action: AdaptiveAction,
}

#[derive(Debug, Clone)]
pub enum AdaptiveCondition {
    ScreenWidth(f32),
    ScreenHeight(f32),
    AspectRatio(f32),
    DeviceType(DeviceType),
}

#[derive(Debug, Clone)]
pub enum AdaptiveAction {
    SwitchLayout(UILayout),
    AdjustSpacing(Vec2),
    HideElements(Vec<String>),
    ShowElements(Vec<String>),
    ScaleElements(f32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    Desktop,
    Tablet,
    Mobile,
}

// State management
#[derive(Debug, Clone)]
pub struct UIState {
    pub style_overrides: HashMap<String, String>,
    pub visibility: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct InteractionConfig {
    pub hover_enabled: bool,
    pub focus_enabled: bool,
    pub drag_enabled: bool,
    pub keyboard_navigation: bool,
}

// Accessibility
#[derive(Debug, Clone)]
pub struct AccessibilityAttributes {
    pub role: String,
    pub label: String,
    pub description: Option<String>,
    pub keyboard_navigation: bool,
    pub screen_reader_friendly: bool,
}

// Type alias for import compatibility
pub type UIGenerationSystem = UIGenerator;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_generator_creation() {
        let config = UIGenerationConfig::default();
        let generator = UIGenerator::new(config);
        
        assert!(!generator.themes.is_empty());
        assert!(!generator.templates.is_empty());
    }

    #[test]
    fn test_button_generation() {
        let config = UIGenerationConfig::default();
        let mut generator = UIGenerator::new(config);
        
        let params = UIGenerationParams {
            element_type: UIElementType::Button,
            dimensions: Some(Vec2::new(100.0, 40.0)),
            theme_name: "minimal".to_string(),
            customization: None,
            adaptive: false,
            responsive_breakpoints: vec![],
        };
        
        let button = generator.generate_element(params);
        assert!(button.is_ok());
        
        let btn = button.unwrap();
        assert_eq!(btn.element_type, UIElementType::Button);
        assert_eq!(btn.dimensions, Vec2::new(100.0, 40.0));
    }

    #[test]
    fn test_layout_generation() {
        let config = UIGenerationConfig::default();
        let mut generator = UIGenerator::new(config);
        
        let params = LayoutGenerationParams {
            layout_type: UILayout::Flex,
            elements: vec![
                ElementSpec {
                    element_type: UIElementType::Button,
                    dimensions: Vec2::new(100.0, 40.0),
                    constraints: LayoutConstraints {
                        position: Some(Vec2::new(10.0, 10.0)),
                        anchor: Some(UIAnchor::TopLeft),
                        min_size: None,
                        max_size: None,
                        flex_grow: 1.0,
                        flex_shrink: 1.0,
                    },
                    customization: None,
                },
            ],
            container_size: Vec2::new(800.0, 600.0),
            theme_name: "minimal".to_string(),
            adaptive: false,
            responsive_breakpoints: vec![],
        };
        
        let layout = generator.generate_layout(params);
        assert!(layout.is_ok());
        
        let lay = layout.unwrap();
        assert_eq!(lay.elements.len(), 1);
        assert_eq!(lay.layout_type, UILayout::Flex);
    }
}