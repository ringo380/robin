use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{Instant, Duration};
use wgpu::util::DeviceExt;
use nalgebra::{Matrix4, Vector2, Vector3, Vector4};
use crate::engine::error::RobinResult;

/// Professional Production UI System with Metal/wgpu rendering
#[derive(Debug)]
pub struct ProductionUISystem {
    pub renderer: UIRenderer,
    pub layout_engine: LayoutEngine,
    pub animation_system: AnimationSystem,
    pub interaction_manager: InteractionManager,
    pub theme_engine: ThemeEngine,
    pub component_registry: ComponentRegistry,
    pub event_dispatcher: EventDispatcher,
    pub performance_overlay: PerformanceOverlay,
    config: UIConfig,
    enabled: bool,
}

#[derive(Debug, Clone)]
pub struct UIConfig {
    pub screen_width: f32,
    pub screen_height: f32,
    pub scale_factor: f32,
    pub enable_animations: bool,
    pub animation_speed: f32,
    pub enable_performance_overlay: bool,
    pub theme: UITheme,
    pub font_size_base: f32,
    pub padding_base: f32,
    pub border_radius: f32,
    pub shadow_intensity: f32,
    pub glass_morphism: bool,
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            screen_width: 1920.0,
            screen_height: 1080.0,
            scale_factor: 1.0,
            enable_animations: true,
            animation_speed: 1.0,
            enable_performance_overlay: true,
            theme: UITheme::DarkProfessional,
            font_size_base: 14.0,
            padding_base: 8.0,
            border_radius: 8.0,
            shadow_intensity: 0.3,
            glass_morphism: true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UITheme {
    DarkProfessional,
    LightMinimal,
    HighContrast,
    Custom,
}

impl ProductionUISystem {
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        config: UIConfig,
    ) -> RobinResult<Self> {
        let renderer = UIRenderer::new(device.clone(), queue.clone(), &config)?;
        let layout_engine = LayoutEngine::new(&config);
        let animation_system = AnimationSystem::new(&config);
        let interaction_manager = InteractionManager::new();
        let theme_engine = ThemeEngine::new(config.theme);
        let component_registry = ComponentRegistry::new();
        let event_dispatcher = EventDispatcher::new();
        let performance_overlay = PerformanceOverlay::new(&config);

        Ok(Self {
            renderer,
            layout_engine,
            animation_system,
            interaction_manager,
            theme_engine,
            component_registry,
            event_dispatcher,
            performance_overlay,
            config,
            enabled: true,
        })
    }

    pub fn render_frame(&mut self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) -> RobinResult<()> {
        if !self.enabled {
            return Ok(());
        }

        // Update animations
        self.animation_system.update(Duration::from_secs_f32(1.0 / 60.0));

        // Process layout
        self.layout_engine.compute_layout(&self.component_registry)?;

        // Render components
        self.renderer.begin_frame(encoder, target)?;

        // Render all registered components
        for component in self.component_registry.get_visible_components() {
            self.render_component(component)?;
        }

        // Render performance overlay if enabled
        if self.config.enable_performance_overlay {
            self.performance_overlay.render(&mut self.renderer)?;
        }

        self.renderer.end_frame()?;

        Ok(())
    }

    fn render_component(&mut self, component: &UIComponent) -> RobinResult<()> {
        match &component.component_type {
            ComponentType::Panel(panel) => self.render_panel(component, panel)?,
            ComponentType::Button(button) => self.render_button(component, button)?,
            ComponentType::Text(text) => self.render_text(component, text)?,
            ComponentType::Slider(slider) => self.render_slider(component, slider)?,
            ComponentType::Graph(graph) => self.render_graph(component, graph)?,
            ComponentType::Modal(modal) => self.render_modal(component, modal)?,
        }
        Ok(())
    }

    fn render_panel(&mut self, component: &UIComponent, panel: &PanelComponent) -> RobinResult<()> {
        let colors = self.theme_engine.get_panel_colors();
        let bounds = component.get_animated_bounds(&self.animation_system);

        // Glass morphism effect if enabled
        if self.config.glass_morphism && panel.glass_effect {
            self.renderer.draw_glass_panel(
                bounds,
                colors.background.with_alpha(0.8),
                panel.blur_intensity,
                self.config.border_radius,
            )?;
        } else {
            // Standard panel with shadow
            if panel.cast_shadow {
                self.renderer.draw_shadow(
                    bounds.expand(4.0),
                    self.config.shadow_intensity,
                    8.0,
                )?;
            }

            self.renderer.draw_rounded_rect(
                bounds,
                colors.background,
                self.config.border_radius,
            )?;
        }

        // Border
        if panel.show_border {
            self.renderer.draw_rounded_rect_outline(
                bounds,
                colors.border,
                self.config.border_radius,
                1.0,
            )?;
        }

        Ok(())
    }

    fn render_button(&mut self, component: &UIComponent, button: &ButtonComponent) -> RobinResult<()> {
        let colors = self.theme_engine.get_button_colors(button.state);
        let bounds = component.get_animated_bounds(&self.animation_system);

        // Hover animation
        let hover_scale = if button.state == ButtonState::Hovered {
            1.05
        } else if button.state == ButtonState::Pressed {
            0.95
        } else {
            1.0
        };

        let animated_bounds = bounds.scale_from_center(hover_scale);

        // Shadow for elevation
        if button.elevated {
            let shadow_offset = if button.state == ButtonState::Pressed { 2.0 } else { 4.0 };
            self.renderer.draw_shadow(
                animated_bounds.offset(0.0, shadow_offset),
                0.2,
                8.0,
            )?;
        }

        // Button background
        self.renderer.draw_rounded_rect(
            animated_bounds,
            colors.background,
            self.config.border_radius,
        )?;

        // Ripple effect for click feedback
        if let Some(ripple) = &button.ripple_effect {
            self.renderer.draw_ripple(
                animated_bounds.center(),
                ripple.radius,
                ripple.alpha,
                colors.accent,
            )?;
        }

        // Button text
        self.renderer.draw_text(
            &button.label,
            animated_bounds.center(),
            colors.text,
            button.font_size.unwrap_or(self.config.font_size_base),
            TextAlignment::Center,
        )?;

        // Icon if present
        if let Some(icon) = &button.icon {
            self.render_icon(icon, animated_bounds.left_center(), colors.text)?;
        }

        Ok(())
    }

    fn render_text(&mut self, component: &UIComponent, text: &TextComponent) -> RobinResult<()> {
        let colors = self.theme_engine.get_text_colors();
        let bounds = component.get_animated_bounds(&self.animation_system);

        // Text shadow for readability
        if text.drop_shadow {
            self.renderer.draw_text(
                &text.content,
                bounds.position.offset(1.0, 1.0),
                Color::black().with_alpha(0.5),
                text.font_size,
                text.alignment,
            )?;
        }

        // Main text
        self.renderer.draw_text(
            &text.content,
            bounds.position,
            text.color.unwrap_or(colors.primary),
            text.font_size,
            text.alignment,
        )?;

        Ok(())
    }

    fn render_slider(&mut self, component: &UIComponent, slider: &SliderComponent) -> RobinResult<()> {
        let colors = self.theme_engine.get_slider_colors();
        let bounds = component.get_animated_bounds(&self.animation_system);

        // Track background
        let track_bounds = Rect::new(
            bounds.x,
            bounds.y + bounds.height * 0.4,
            bounds.width,
            bounds.height * 0.2,
        );

        self.renderer.draw_rounded_rect(
            track_bounds,
            colors.track,
            track_bounds.height * 0.5,
        )?;

        // Filled portion
        let fill_width = bounds.width * slider.value;
        let fill_bounds = Rect::new(
            bounds.x,
            track_bounds.y,
            fill_width,
            track_bounds.height,
        );

        self.renderer.draw_rounded_rect(
            fill_bounds,
            colors.fill,
            track_bounds.height * 0.5,
        )?;

        // Knob
        let knob_x = bounds.x + fill_width;
        let knob_radius = bounds.height * 0.4;

        // Knob shadow
        self.renderer.draw_circle(
            Vector2::new(knob_x, bounds.y + bounds.height * 0.5 + 2.0),
            knob_radius,
            Color::black().with_alpha(0.2),
        )?;

        // Knob
        self.renderer.draw_circle(
            Vector2::new(knob_x, bounds.y + bounds.height * 0.5),
            knob_radius,
            colors.knob,
        )?;

        // Value label if shown
        if slider.show_value {
            let value_text = format!("{:.1}%", slider.value * 100.0);
            self.renderer.draw_text(
                &value_text,
                Vector2::new(knob_x, bounds.y - 10.0),
                colors.text,
                self.config.font_size_base * 0.9,
                TextAlignment::Center,
            )?;
        }

        Ok(())
    }

    fn render_graph(&mut self, component: &UIComponent, graph: &GraphComponent) -> RobinResult<()> {
        let colors = self.theme_engine.get_graph_colors();
        let bounds = component.get_animated_bounds(&self.animation_system);

        // Background
        self.renderer.draw_rounded_rect(
            bounds,
            colors.background,
            self.config.border_radius,
        )?;

        // Grid lines
        self.render_graph_grid(bounds, &colors)?;

        // Plot data points
        for (i, series) in graph.data_series.iter().enumerate() {
            let color = colors.series_colors[i % colors.series_colors.len()];
            self.render_graph_series(bounds, series, color, graph.graph_type)?;
        }

        // Axes labels
        self.render_graph_axes(bounds, graph, &colors)?;

        // Legend if enabled
        if graph.show_legend {
            self.render_graph_legend(bounds, graph, &colors)?;
        }

        Ok(())
    }

    fn render_modal(&mut self, component: &UIComponent, modal: &ModalComponent) -> RobinResult<()> {
        // Dimmed background
        self.renderer.draw_rect(
            Rect::new(0.0, 0.0, self.config.screen_width, self.config.screen_height),
            Color::black().with_alpha(modal.backdrop_opacity),
        )?;

        let colors = self.theme_engine.get_modal_colors();
        let bounds = component.get_animated_bounds(&self.animation_system);

        // Modal shadow
        self.renderer.draw_shadow(
            bounds.expand(8.0),
            0.4,
            16.0,
        )?;

        // Modal background with glass effect
        if self.config.glass_morphism {
            self.renderer.draw_glass_panel(
                bounds,
                colors.background.with_alpha(0.95),
                12.0,
                self.config.border_radius * 1.5,
            )?;
        } else {
            self.renderer.draw_rounded_rect(
                bounds,
                colors.background,
                self.config.border_radius * 1.5,
            )?;
        }

        // Title bar
        if let Some(title) = &modal.title {
            let title_bounds = Rect::new(
                bounds.x,
                bounds.y,
                bounds.width,
                40.0,
            );

            self.renderer.draw_rounded_rect_top(
                title_bounds,
                colors.header,
                self.config.border_radius * 1.5,
            )?;

            self.renderer.draw_text(
                title,
                title_bounds.center(),
                colors.text,
                self.config.font_size_base * 1.2,
                TextAlignment::Center,
            )?;
        }

        // Close button
        if modal.show_close_button {
            let close_bounds = Rect::new(
                bounds.x + bounds.width - 30.0,
                bounds.y + 10.0,
                20.0,
                20.0,
            );

            self.render_close_button(close_bounds, &colors)?;
        }

        Ok(())
    }

    fn render_icon(&mut self, icon: &Icon, position: Vector2<f32>, color: Color) -> RobinResult<()> {
        match icon {
            Icon::Custom(path) => {
                self.renderer.draw_svg_icon(path, position, 20.0, color)?;
            }
            Icon::Builtin(icon_type) => {
                self.renderer.draw_builtin_icon(*icon_type, position, 20.0, color)?;
            }
        }
        Ok(())
    }

    fn render_close_button(&mut self, bounds: Rect, colors: &ModalColors) -> RobinResult<()> {
        // X icon
        let center = bounds.center();
        let half_size = bounds.width * 0.3;

        self.renderer.draw_line(
            center - Vector2::new(half_size, half_size),
            center + Vector2::new(half_size, half_size),
            colors.text,
            2.0,
        )?;

        self.renderer.draw_line(
            center - Vector2::new(-half_size, half_size),
            center + Vector2::new(-half_size, half_size),
            colors.text,
            2.0,
        )?;

        Ok(())
    }

    fn render_graph_grid(&mut self, bounds: Rect, colors: &GraphColors) -> RobinResult<()> {
        // Horizontal grid lines
        for i in 1..5 {
            let y = bounds.y + (bounds.height * i as f32 / 5.0);
            self.renderer.draw_line(
                Vector2::new(bounds.x, y),
                Vector2::new(bounds.x + bounds.width, y),
                colors.grid.with_alpha(0.2),
                1.0,
            )?;
        }

        // Vertical grid lines
        for i in 1..10 {
            let x = bounds.x + (bounds.width * i as f32 / 10.0);
            self.renderer.draw_line(
                Vector2::new(x, bounds.y),
                Vector2::new(x, bounds.y + bounds.height),
                colors.grid.with_alpha(0.2),
                1.0,
            )?;
        }

        Ok(())
    }

    fn render_graph_series(
        &mut self,
        bounds: Rect,
        series: &DataSeries,
        color: Color,
        graph_type: GraphType,
    ) -> RobinResult<()> {
        if series.values.is_empty() {
            return Ok(());
        }

        let points: Vec<Vector2<f32>> = series.values.iter().enumerate().map(|(i, value)| {
            let x = bounds.x + (i as f32 / series.values.len() as f32) * bounds.width;
            let y = bounds.y + bounds.height - (*value * bounds.height);
            Vector2::new(x, y)
        }).collect();

        match graph_type {
            GraphType::Line => {
                self.renderer.draw_polyline(&points, color, 2.0)?;
            }
            GraphType::Area => {
                self.renderer.draw_filled_polygon(&points, color.with_alpha(0.3))?;
                self.renderer.draw_polyline(&points, color, 2.0)?;
            }
            GraphType::Bar => {
                for (i, point) in points.iter().enumerate() {
                    let bar_width = bounds.width / series.values.len() as f32 * 0.8;
                    let bar_bounds = Rect::new(
                        point.x - bar_width * 0.5,
                        point.y,
                        bar_width,
                        bounds.y + bounds.height - point.y,
                    );
                    self.renderer.draw_rect(bar_bounds, color)?;
                }
            }
            GraphType::Scatter => {
                for point in &points {
                    self.renderer.draw_circle(*point, 3.0, color)?;
                }
            }
        }

        Ok(())
    }

    fn render_graph_axes(&mut self, bounds: Rect, graph: &GraphComponent, colors: &GraphColors) -> RobinResult<()> {
        // X-axis label
        if let Some(x_label) = &graph.x_axis_label {
            self.renderer.draw_text(
                x_label,
                Vector2::new(bounds.x + bounds.width * 0.5, bounds.y + bounds.height + 20.0),
                colors.text,
                self.config.font_size_base * 0.9,
                TextAlignment::Center,
            )?;
        }

        // Y-axis label
        if let Some(y_label) = &graph.y_axis_label {
            self.renderer.draw_text_rotated(
                y_label,
                Vector2::new(bounds.x - 30.0, bounds.y + bounds.height * 0.5),
                colors.text,
                self.config.font_size_base * 0.9,
                -90.0,
                TextAlignment::Center,
            )?;
        }

        Ok(())
    }

    fn render_graph_legend(&mut self, bounds: Rect, graph: &GraphComponent, colors: &GraphColors) -> RobinResult<()> {
        let legend_x = bounds.x + bounds.width - 100.0;
        let legend_y = bounds.y + 10.0;

        for (i, series) in graph.data_series.iter().enumerate() {
            let y_offset = i as f32 * 20.0;
            let color = colors.series_colors[i % colors.series_colors.len()];

            // Color indicator
            self.renderer.draw_rect(
                Rect::new(legend_x, legend_y + y_offset, 15.0, 15.0),
                color,
            )?;

            // Series name
            self.renderer.draw_text(
                &series.name,
                Vector2::new(legend_x + 20.0, legend_y + y_offset + 7.5),
                colors.text,
                self.config.font_size_base * 0.85,
                TextAlignment::Left,
            )?;
        }

        Ok(())
    }

    pub fn handle_input(&mut self, event: &InputEvent) -> RobinResult<()> {
        self.interaction_manager.process_input(event)?;
        self.event_dispatcher.dispatch_event(event)?;
        Ok(())
    }

    pub fn create_splash_screen(&mut self) -> RobinResult<()> {
        let splash = UIComponent::new_modal(
            "splash",
            Rect::centered(600.0, 400.0),
            ModalComponent {
                title: None,
                show_close_button: false,
                backdrop_opacity: 0.95,
                auto_close_duration: Some(Duration::from_secs(3)),
            },
        );

        self.component_registry.register(splash);

        // Robin Engine logo
        let logo = UIComponent::new_text(
            "logo",
            Rect::new(300.0, 150.0, 0.0, 0.0),
            TextComponent {
                content: "ROBIN ENGINE".to_string(),
                font_size: 48.0,
                color: Some(Color::white()),
                alignment: TextAlignment::Center,
                drop_shadow: true,
            },
        );

        self.component_registry.register(logo);

        // Tagline
        let tagline = UIComponent::new_text(
            "tagline",
            Rect::new(300.0, 220.0, 0.0, 0.0),
            TextComponent {
                content: "Next-Generation Voxel Game Engine".to_string(),
                font_size: 18.0,
                color: Some(Color::gray(0.8)),
                alignment: TextAlignment::Center,
                drop_shadow: false,
            },
        );

        self.component_registry.register(tagline);

        // Animate splash screen entrance
        self.animation_system.animate_fade_in("splash", Duration::from_millis(500))?;
        self.animation_system.animate_slide_in("logo", SlideDirection::Top, Duration::from_millis(800))?;
        self.animation_system.animate_slide_in("tagline", SlideDirection::Bottom, Duration::from_millis(1000))?;

        Ok(())
    }
}

// Supporting structures and components

#[derive(Debug)]
pub struct UIRenderer {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    texture_atlas: TextureAtlas,
    font_renderer: FontRenderer,
    current_vertices: Vec<UIVertex>,
    current_indices: Vec<u32>,
}

#[derive(Debug)]
pub struct UIComponent {
    pub id: String,
    pub bounds: Rect,
    pub component_type: ComponentType,
    pub visible: bool,
    pub interactive: bool,
    pub z_index: i32,
    pub animation_state: AnimationState,
}

#[derive(Debug)]
pub enum ComponentType {
    Panel(PanelComponent),
    Button(ButtonComponent),
    Text(TextComponent),
    Slider(SliderComponent),
    Graph(GraphComponent),
    Modal(ModalComponent),
}

#[derive(Debug)]
pub struct PanelComponent {
    pub glass_effect: bool,
    pub blur_intensity: f32,
    pub cast_shadow: bool,
    pub show_border: bool,
}

#[derive(Debug)]
pub struct ButtonComponent {
    pub label: String,
    pub icon: Option<Icon>,
    pub state: ButtonState,
    pub elevated: bool,
    pub ripple_effect: Option<RippleEffect>,
    pub font_size: Option<f32>,
}

#[derive(Debug)]
pub struct TextComponent {
    pub content: String,
    pub font_size: f32,
    pub color: Option<Color>,
    pub alignment: TextAlignment,
    pub drop_shadow: bool,
}

#[derive(Debug)]
pub struct SliderComponent {
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub show_value: bool,
}

#[derive(Debug)]
pub struct GraphComponent {
    pub data_series: Vec<DataSeries>,
    pub graph_type: GraphType,
    pub x_axis_label: Option<String>,
    pub y_axis_label: Option<String>,
    pub show_legend: bool,
    pub show_grid: bool,
}

#[derive(Debug)]
pub struct ModalComponent {
    pub title: Option<String>,
    pub show_close_button: bool,
    pub backdrop_opacity: f32,
    pub auto_close_duration: Option<Duration>,
}

#[derive(Debug, Clone)]
pub struct DataSeries {
    pub name: String,
    pub values: Vec<f32>,
    pub color: Option<Color>,
}

#[derive(Debug, Clone, Copy)]
pub enum GraphType {
    Line,
    Area,
    Bar,
    Scatter,
}

#[derive(Debug, Clone, Copy)]
pub enum ButtonState {
    Normal,
    Hovered,
    Pressed,
    Disabled,
}

#[derive(Debug, Clone)]
pub enum Icon {
    Custom(String),
    Builtin(BuiltinIcon),
}

#[derive(Debug, Clone, Copy)]
pub enum BuiltinIcon {
    Menu,
    Close,
    Settings,
    Play,
    Pause,
    Stop,
}

#[derive(Debug, Clone)]
pub struct RippleEffect {
    pub center: Vector2<f32>,
    pub radius: f32,
    pub alpha: f32,
    pub start_time: Instant,
}

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    pub fn centered(width: f32, height: f32) -> Self {
        Self {
            x: (1920.0 - width) * 0.5,
            y: (1080.0 - height) * 0.5,
            width,
            height,
        }
    }

    pub fn center(&self) -> Vector2<f32> {
        Vector2::new(self.x + self.width * 0.5, self.y + self.height * 0.5)
    }

    pub fn expand(&self, amount: f32) -> Self {
        Self {
            x: self.x - amount,
            y: self.y - amount,
            width: self.width + amount * 2.0,
            height: self.height + amount * 2.0,
        }
    }

    pub fn scale_from_center(&self, scale: f32) -> Self {
        let center = self.center();
        let new_width = self.width * scale;
        let new_height = self.height * scale;
        Self {
            x: center.x - new_width * 0.5,
            y: center.y - new_height * 0.5,
            width: new_width,
            height: new_height,
        }
    }

    pub fn offset(&self, x: f32, y: f32) -> Self {
        Self {
            x: self.x + x,
            y: self.y + y,
            width: self.width,
            height: self.height,
        }
    }

    pub fn left_center(&self) -> Vector2<f32> {
        Vector2::new(self.x, self.y + self.height * 0.5)
    }

    pub fn position(&self) -> Vector2<f32> {
        Vector2::new(self.x, self.y)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn white() -> Self {
        Self::new(1.0, 1.0, 1.0, 1.0)
    }

    pub fn black() -> Self {
        Self::new(0.0, 0.0, 0.0, 1.0)
    }

    pub fn gray(value: f32) -> Self {
        Self::new(value, value, value, 1.0)
    }

    pub fn with_alpha(&self, alpha: f32) -> Self {
        Self { r: self.r, g: self.g, b: self.b, a: alpha }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
}

// Simplified implementations for supporting components

macro_rules! define_ui_subsystem {
    ($name:ident) => {
        #[derive(Debug)]
        pub struct $name;

        impl $name {
            pub fn new(_config: &UIConfig) -> Self {
                Self
            }
        }
    };
}

define_ui_subsystem!(LayoutEngine);
define_ui_subsystem!(AnimationSystem);
define_ui_subsystem!(InteractionManager);
define_ui_subsystem!(EventDispatcher);
define_ui_subsystem!(PerformanceOverlay);
define_ui_subsystem!(TextureAtlas);
define_ui_subsystem!(FontRenderer);

#[derive(Debug)]
pub struct ThemeEngine {
    theme: UITheme,
}

impl ThemeEngine {
    pub fn new(theme: UITheme) -> Self {
        Self { theme }
    }

    pub fn get_panel_colors(&self) -> PanelColors {
        match self.theme {
            UITheme::DarkProfessional => PanelColors {
                background: Color::new(0.1, 0.1, 0.12, 1.0),
                border: Color::new(0.3, 0.3, 0.35, 1.0),
                shadow: Color::black().with_alpha(0.3),
            },
            _ => PanelColors::default(),
        }
    }

    pub fn get_button_colors(&self, state: ButtonState) -> ButtonColors {
        match self.theme {
            UITheme::DarkProfessional => match state {
                ButtonState::Normal => ButtonColors {
                    background: Color::new(0.2, 0.4, 0.8, 1.0),
                    text: Color::white(),
                    border: Color::new(0.3, 0.5, 0.9, 1.0),
                    accent: Color::new(0.4, 0.6, 1.0, 1.0),
                },
                ButtonState::Hovered => ButtonColors {
                    background: Color::new(0.25, 0.45, 0.85, 1.0),
                    text: Color::white(),
                    border: Color::new(0.35, 0.55, 0.95, 1.0),
                    accent: Color::new(0.5, 0.7, 1.0, 1.0),
                },
                ButtonState::Pressed => ButtonColors {
                    background: Color::new(0.15, 0.35, 0.75, 1.0),
                    text: Color::white(),
                    border: Color::new(0.25, 0.45, 0.85, 1.0),
                    accent: Color::new(0.3, 0.5, 0.9, 1.0),
                },
                ButtonState::Disabled => ButtonColors {
                    background: Color::new(0.15, 0.15, 0.17, 1.0),
                    text: Color::gray(0.5),
                    border: Color::gray(0.3),
                    accent: Color::gray(0.4),
                },
            },
            _ => ButtonColors::default(),
        }
    }

    pub fn get_text_colors(&self) -> TextColors {
        match self.theme {
            UITheme::DarkProfessional => TextColors {
                primary: Color::gray(0.95),
                secondary: Color::gray(0.7),
                accent: Color::new(0.4, 0.6, 1.0, 1.0),
            },
            _ => TextColors::default(),
        }
    }

    pub fn get_slider_colors(&self) -> SliderColors {
        match self.theme {
            UITheme::DarkProfessional => SliderColors {
                track: Color::gray(0.2),
                fill: Color::new(0.2, 0.4, 0.8, 1.0),
                knob: Color::white(),
                text: Color::gray(0.9),
            },
            _ => SliderColors::default(),
        }
    }

    pub fn get_graph_colors(&self) -> GraphColors {
        match self.theme {
            UITheme::DarkProfessional => GraphColors {
                background: Color::new(0.05, 0.05, 0.07, 1.0),
                grid: Color::gray(0.3),
                text: Color::gray(0.8),
                series_colors: vec![
                    Color::new(0.2, 0.6, 1.0, 1.0),
                    Color::new(1.0, 0.3, 0.3, 1.0),
                    Color::new(0.3, 1.0, 0.3, 1.0),
                    Color::new(1.0, 0.8, 0.2, 1.0),
                ],
            },
            _ => GraphColors::default(),
        }
    }

    pub fn get_modal_colors(&self) -> ModalColors {
        match self.theme {
            UITheme::DarkProfessional => ModalColors {
                background: Color::new(0.12, 0.12, 0.15, 1.0),
                header: Color::new(0.08, 0.08, 0.1, 1.0),
                text: Color::gray(0.95),
            },
            _ => ModalColors::default(),
        }
    }
}

// Color scheme structures
#[derive(Debug, Default)]
pub struct PanelColors {
    pub background: Color,
    pub border: Color,
    pub shadow: Color,
}

#[derive(Debug, Default)]
pub struct ButtonColors {
    pub background: Color,
    pub text: Color,
    pub border: Color,
    pub accent: Color,
}

#[derive(Debug, Default)]
pub struct TextColors {
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
}

#[derive(Debug, Default)]
pub struct SliderColors {
    pub track: Color,
    pub fill: Color,
    pub knob: Color,
    pub text: Color,
}

#[derive(Debug, Default)]
pub struct GraphColors {
    pub background: Color,
    pub grid: Color,
    pub text: Color,
    pub series_colors: Vec<Color>,
}

#[derive(Debug, Default)]
pub struct ModalColors {
    pub background: Color,
    pub header: Color,
    pub text: Color,
}

// Additional UI components and helpers would continue here...