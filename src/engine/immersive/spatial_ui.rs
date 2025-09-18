// Robin Engine - Spatial UI Components for AR/VR
// Provides 3D user interface elements that work in immersive environments

use nalgebra::{Vector3, UnitQuaternion, Matrix4, Point3, Translation3, Scale3};
use std::collections::HashMap;
use crate::engine::error::RobinResult;

/// Spatial UI element that exists in 3D space
#[derive(Debug, Clone)]
pub struct SpatialUIElement {
    pub id: String,
    pub element_type: SpatialElementType,
    pub transform: SpatialTransform,
    pub content: UIContent,
    pub interaction: InteractionState,
    pub style: SpatialStyle,
    pub anchoring: AnchoringMode,
    pub visibility: VisibilitySettings,
}

/// Transform for spatial UI elements
#[derive(Debug, Clone)]
pub struct SpatialTransform {
    pub position: Vector3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub scale: Vector3<f32>,
    pub parent: Option<String>,
    pub local_matrix: Matrix4<f32>,
    pub world_matrix: Matrix4<f32>,
}

impl Default for SpatialTransform {
    fn default() -> Self {
        Self {
            position: Vector3::zeros(),
            rotation: UnitQuaternion::identity(),
            scale: Vector3::new(1.0, 1.0, 1.0),
            parent: None,
            local_matrix: Matrix4::identity(),
            world_matrix: Matrix4::identity(),
        }
    }
}

/// Types of spatial UI elements
#[derive(Debug, Clone)]
pub enum SpatialElementType {
    /// Floating panel with content
    Panel {
        width: f32,
        height: f32,
        corner_radius: f32,
    },
    /// 3D button that can be pressed
    Button3D {
        width: f32,
        height: f32,
        depth: f32,
        press_depth: f32,
    },
    /// Text that floats in space
    SpatialText {
        font_size: f32,
        max_width: f32,
        alignment: TextAlignment,
    },
    /// Holographic display for data
    HologramDisplay {
        projection_type: HologramType,
        animation_speed: f32,
    },
    /// 3D slider for value adjustment
    Slider3D {
        length: f32,
        min_value: f32,
        max_value: f32,
        current_value: f32,
    },
    /// Radial menu that appears around hand/controller
    RadialMenu {
        radius: f32,
        num_segments: u32,
        active_segment: Option<u32>,
    },
    /// 3D model viewer
    ModelViewer {
        model_path: String,
        auto_rotate: bool,
        scale_mode: ScaleMode,
    },
    /// Spatial toolbar
    Toolbar {
        tools: Vec<ToolDefinition>,
        orientation: ToolbarOrientation,
    },
}

/// Content displayed in UI element
#[derive(Debug, Clone)]
pub enum UIContent {
    Text(String),
    Image(String), // Path to texture
    Video(String), // Path to video
    Model(String), // Path to 3D model
    Custom(Vec<u8>), // Custom render data
    Mixed {
        text: Option<String>,
        image: Option<String>,
        metadata: HashMap<String, String>,
    },
}

/// Interaction state for spatial elements
#[derive(Debug, Clone)]
pub struct InteractionState {
    pub is_hovered: bool,
    pub is_selected: bool,
    pub is_grabbed: bool,
    pub hover_distance: f32,
    pub interaction_point: Option<Point3<f32>>,
    pub interacting_hand: Option<super::Hand>,
    pub gaze_dwell_time: f32,
}

impl Default for InteractionState {
    fn default() -> Self {
        Self {
            is_hovered: false,
            is_selected: false,
            is_grabbed: false,
            hover_distance: 0.5,
            interaction_point: None,
            interacting_hand: None,
            gaze_dwell_time: 0.0,
        }
    }
}

/// Visual style for spatial elements
#[derive(Debug, Clone)]
pub struct SpatialStyle {
    pub background_color: [f32; 4],
    pub border_color: [f32; 4],
    pub text_color: [f32; 4],
    pub opacity: f32,
    pub glass_effect: bool,
    pub glow_intensity: f32,
    pub shadow_intensity: f32,
    pub material_type: MaterialType,
}

impl Default for SpatialStyle {
    fn default() -> Self {
        Self {
            background_color: [0.1, 0.1, 0.1, 0.9],
            border_color: [0.3, 0.3, 0.8, 1.0],
            text_color: [1.0, 1.0, 1.0, 1.0],
            opacity: 0.95,
            glass_effect: true,
            glow_intensity: 0.2,
            shadow_intensity: 0.3,
            material_type: MaterialType::Glass,
        }
    }
}

/// How the element is anchored in space
#[derive(Debug, Clone)]
pub enum AnchoringMode {
    /// Fixed in world space
    World(Vector3<f32>),
    /// Follows the user's head
    HeadLocked {
        offset: Vector3<f32>,
        smoothing: f32,
    },
    /// Attached to hand/controller
    HandAttached {
        hand: super::Hand,
        offset: Vector3<f32>,
    },
    /// Anchored to detected surface
    SurfaceAnchored {
        surface_id: String,
        offset: Vector3<f32>,
    },
    /// Lazy follow - stays in place but repositions when too far
    LazyFollow {
        max_distance: f32,
        reposition_speed: f32,
    },
}

/// Visibility settings for spatial UI
#[derive(Debug, Clone)]
pub struct VisibilitySettings {
    pub is_visible: bool,
    pub fade_with_distance: bool,
    pub min_distance: f32,
    pub max_distance: f32,
    pub occlusion_enabled: bool,
    pub billboard_mode: BillboardMode,
}

impl Default for VisibilitySettings {
    fn default() -> Self {
        Self {
            is_visible: true,
            fade_with_distance: true,
            min_distance: 0.3,
            max_distance: 10.0,
            occlusion_enabled: true,
            billboard_mode: BillboardMode::None,
        }
    }
}

/// Billboard behavior for UI elements
#[derive(Debug, Clone)]
pub enum BillboardMode {
    None,
    FaceCamera,
    FaceCameraYOnly,
    FaceUser,
}

/// Text alignment for spatial text
#[derive(Debug, Clone)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
    Justify,
}

/// Types of holographic displays
#[derive(Debug, Clone)]
pub enum HologramType {
    Wireframe,
    Solid,
    Particle,
    Volumetric,
}

/// Scale modes for model viewer
#[derive(Debug, Clone)]
pub enum ScaleMode {
    Fixed,
    FitToBox(f32),
    UserAdjustable,
}

/// Tool definition for spatial toolbar
#[derive(Debug, Clone)]
pub struct ToolDefinition {
    pub id: String,
    pub icon: String,
    pub label: String,
    pub action: String,
    pub enabled: bool,
}

/// Toolbar orientation
#[derive(Debug, Clone)]
pub enum ToolbarOrientation {
    Horizontal,
    Vertical,
    Curved(f32), // Radius of curve
}

/// Material types for UI elements
#[derive(Debug, Clone)]
pub enum MaterialType {
    Solid,
    Glass,
    Holographic,
    Metal,
    Plastic,
}

/// Manages spatial UI elements in 3D space
#[derive(Debug)]
pub struct SpatialUISystem {
    elements: HashMap<String, SpatialUIElement>,
    active_element: Option<String>,
    interaction_rays: Vec<InteractionRay>,
    layout_manager: SpatialLayoutManager,
    animation_system: UIAnimationSystem,
}

/// Ray for interaction with spatial UI
#[derive(Debug, Clone)]
pub struct InteractionRay {
    pub origin: Point3<f32>,
    pub direction: Vector3<f32>,
    pub max_distance: f32,
    pub source: InteractionSource,
}

/// Source of interaction ray
#[derive(Debug, Clone)]
pub enum InteractionSource {
    Hand(super::Hand),
    Controller(u32),
    Gaze,
    Mouse,
}

/// Manages spatial layout of UI elements
#[derive(Debug)]
pub struct SpatialLayoutManager {
    layouts: HashMap<String, SpatialLayout>,
    active_layout: Option<String>,
}

/// Spatial layout configuration
#[derive(Debug, Clone)]
pub struct SpatialLayout {
    pub name: String,
    pub elements: Vec<String>,
    pub arrangement: LayoutArrangement,
    pub spacing: f32,
    pub alignment: LayoutAlignment,
}

/// How elements are arranged in space
#[derive(Debug, Clone)]
pub enum LayoutArrangement {
    Grid {
        rows: u32,
        columns: u32,
    },
    Circle {
        radius: f32,
    },
    Line {
        direction: Vector3<f32>,
    },
    Sphere {
        radius: f32,
    },
    Custom(Vec<Vector3<f32>>),
}

/// Layout alignment options
#[derive(Debug, Clone)]
pub enum LayoutAlignment {
    Center,
    TopLeft,
    TopCenter,
    TopRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

/// Animation system for UI elements
#[derive(Debug)]
pub struct UIAnimationSystem {
    animations: HashMap<String, UIAnimation>,
    active_animations: Vec<String>,
}

/// UI animation definition
#[derive(Debug, Clone)]
pub struct UIAnimation {
    pub element_id: String,
    pub animation_type: AnimationType,
    pub duration: f32,
    pub current_time: f32,
    pub loop_mode: LoopMode,
    pub easing: EasingFunction,
}

/// Types of UI animations
#[derive(Debug, Clone)]
pub enum AnimationType {
    FadeIn,
    FadeOut,
    Scale(Vector3<f32>, Vector3<f32>),
    Rotate(UnitQuaternion<f32>, UnitQuaternion<f32>),
    Move(Vector3<f32>, Vector3<f32>),
    Pulse(f32),
    Ripple,
}

/// Animation loop modes
#[derive(Debug, Clone)]
pub enum LoopMode {
    Once,
    Loop,
    PingPong,
}

/// Easing functions for animations
#[derive(Debug, Clone)]
pub enum EasingFunction {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bounce,
    Elastic,
}

impl SpatialUISystem {
    pub fn new() -> Self {
        Self {
            elements: HashMap::new(),
            active_element: None,
            interaction_rays: Vec::new(),
            layout_manager: SpatialLayoutManager {
                layouts: HashMap::new(),
                active_layout: None,
            },
            animation_system: UIAnimationSystem {
                animations: HashMap::new(),
                active_animations: Vec::new(),
            },
        }
    }

    /// Add a new spatial UI element
    pub fn add_element(&mut self, element: SpatialUIElement) -> RobinResult<()> {
        self.elements.insert(element.id.clone(), element);
        Ok(())
    }

    /// Remove a spatial UI element
    pub fn remove_element(&mut self, id: &str) -> RobinResult<()> {
        self.elements.remove(id);
        if self.active_element.as_ref() == Some(&id.to_string()) {
            self.active_element = None;
        }
        Ok(())
    }

    /// Update spatial UI system
    pub fn update(&mut self, delta_time: f32) -> RobinResult<()> {
        // Update animations
        self.update_animations(delta_time)?;

        // Update interaction states
        self.update_interactions()?;

        // Update element transforms based on anchoring
        self.update_transforms(delta_time)?;

        // Apply layouts
        self.apply_layouts()?;

        Ok(())
    }

    /// Process interaction ray
    pub fn process_interaction(&mut self, ray: InteractionRay) -> RobinResult<Option<String>> {
        // Store ray for processing
        self.interaction_rays.push(ray.clone());

        // Find intersected element
        let result = self.find_intersected_element(&ray);
        Ok(result)
    }

    /// Update animations
    fn update_animations(&mut self, delta_time: f32) -> RobinResult<()> {
        let mut completed_animations = Vec::new();

        for anim_id in &self.animation_system.active_animations {
            if let Some(animation) = self.animation_system.animations.get_mut(anim_id) {
                animation.current_time += delta_time;

                if animation.current_time >= animation.duration {
                    match animation.loop_mode {
                        LoopMode::Once => {
                            completed_animations.push(anim_id.clone());
                        }
                        LoopMode::Loop => {
                            animation.current_time = 0.0;
                        }
                        LoopMode::PingPong => {
                            // Reverse animation direction
                            animation.current_time = 0.0;
                        }
                    }
                }

                // Apply animation to element
                if self.elements.contains_key(&animation.element_id) {
                    // TODO: Apply animation when refactored to avoid borrow conflicts
                }
            }
        }

        // Remove completed animations
        for id in completed_animations {
            if let Some(index) = self.animation_system.active_animations.iter().position(|x| x == &id) {
                self.animation_system.active_animations.remove(index);
            }
        }

        Ok(())
    }

    /// Apply animation to element
    fn apply_animation(&self, element: &mut SpatialUIElement, animation: &UIAnimation) -> RobinResult<()> {
        let t = self.calculate_easing(animation.current_time / animation.duration, &animation.easing);

        match &animation.animation_type {
            AnimationType::FadeIn => {
                element.style.opacity = t;
            }
            AnimationType::FadeOut => {
                element.style.opacity = 1.0 - t;
            }
            AnimationType::Scale(from, to) => {
                element.transform.scale = from + (to - from) * t;
            }
            AnimationType::Move(from, to) => {
                element.transform.position = from + (to - from) * t;
            }
            AnimationType::Pulse(intensity) => {
                let pulse = (animation.current_time * std::f32::consts::PI * 2.0).sin();
                element.transform.scale = Vector3::new(1.0, 1.0, 1.0) * (1.0 + pulse * intensity);
            }
            _ => {}
        }

        Ok(())
    }

    /// Calculate easing value
    fn calculate_easing(&self, t: f32, easing: &EasingFunction) -> f32 {
        match easing {
            EasingFunction::Linear => t,
            EasingFunction::EaseIn => t * t,
            EasingFunction::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            EasingFunction::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            EasingFunction::Bounce => {
                let n1 = 7.5625;
                let d1 = 2.75;

                if t < 1.0 / d1 {
                    n1 * t * t
                } else if t < 2.0 / d1 {
                    let t = t - 1.5 / d1;
                    n1 * t * t + 0.75
                } else if t < 2.5 / d1 {
                    let t = t - 2.25 / d1;
                    n1 * t * t + 0.9375
                } else {
                    let t = t - 2.625 / d1;
                    n1 * t * t + 0.984375
                }
            }
            EasingFunction::Elastic => {
                if t == 0.0 || t == 1.0 {
                    t
                } else {
                    let p = 0.3;
                    -(2.0_f32.powf(10.0 * (t - 1.0)) * ((t - 1.0 - p / 4.0) * (2.0 * std::f32::consts::PI) / p).sin())
                }
            }
        }
    }

    /// Update interaction states
    fn update_interactions(&mut self) -> RobinResult<()> {
        // Clear previous hover states
        for element in self.elements.values_mut() {
            element.interaction.is_hovered = false;
            element.interaction.interaction_point = None;
        }

        // Collect interaction results without borrowing conflicts
        let mut interaction_results = Vec::new();
        for ray in &self.interaction_rays {
            let result = self.find_intersected_element(ray);
            if let Some(element_id) = result {
                interaction_results.push((element_id, ray.clone()));
            }
        }

        // Apply interaction results
        for (element_id, ray) in interaction_results {
            if let Some(element) = self.elements.get_mut(&element_id) {
                element.interaction.is_hovered = true;
                // Calculate interaction point
                let point = ray.origin + ray.direction * element.interaction.hover_distance;
                element.interaction.interaction_point = Some(point);
            }
        }

        self.interaction_rays.clear();
        Ok(())
    }

    /// Find intersected element without borrowing conflicts
    fn find_intersected_element(&self, ray: &InteractionRay) -> Option<String> {
        let mut closest_element = None;
        let mut closest_distance = f32::MAX;

        for (id, element) in &self.elements {
            if !element.visibility.is_visible {
                continue;
            }

            // Simple sphere intersection for now
            let element_pos = Vector3::new(
                element.transform.position.x,
                element.transform.position.y,
                element.transform.position.z,
            );

            let to_element = element_pos - Vector3::new(ray.origin.x, ray.origin.y, ray.origin.z);
            let distance = to_element.dot(&ray.direction);

            if distance > 0.0 && distance < ray.max_distance && distance < closest_distance {
                closest_distance = distance;
                closest_element = Some(id.clone());
            }
        }

        closest_element
    }

    /// Update element transforms
    fn update_transforms(&mut self, delta_time: f32) -> RobinResult<()> {
        for element in self.elements.values_mut() {
            match &element.anchoring {
                AnchoringMode::HeadLocked { offset, smoothing } => {
                    // Smooth follow head position
                    let target_pos = Vector3::zeros() + offset; // Would get actual head position
                    element.transform.position = element.transform.position + (target_pos - element.transform.position) * (smoothing * delta_time);
                }
                AnchoringMode::LazyFollow { max_distance, reposition_speed } => {
                    let user_pos = Vector3::zeros(); // Would get actual user position
                    let distance = (element.transform.position - user_pos).magnitude();

                    if distance > *max_distance {
                        let target_pos = user_pos + (element.transform.position - user_pos).normalize() * (*max_distance);
                        element.transform.position = element.transform.position + (target_pos - element.transform.position) * (reposition_speed * delta_time);
                    }
                }
                _ => {}
            }

            // Update matrices
            let translation = Translation3::from(element.transform.position);
            let rotation = element.transform.rotation.to_homogeneous();
            let scale = Scale3::from(element.transform.scale);
            element.transform.local_matrix = translation.to_homogeneous() * rotation * scale.to_homogeneous();
        }

        Ok(())
    }

    /// Apply active layout
    fn apply_layouts(&mut self) -> RobinResult<()> {
        if let Some(layout_name) = &self.layout_manager.active_layout {
            if let Some(layout) = self.layout_manager.layouts.get(layout_name) {
                let positions = self.calculate_layout_positions(layout)?;

                for (i, element_id) in layout.elements.iter().enumerate() {
                    if let Some(element) = self.elements.get_mut(element_id) {
                        if i < positions.len() {
                            element.transform.position = positions[i];
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Calculate positions for layout
    fn calculate_layout_positions(&self, layout: &SpatialLayout) -> RobinResult<Vec<Vector3<f32>>> {
        let mut positions = Vec::new();

        match &layout.arrangement {
            LayoutArrangement::Grid { rows, columns } => {
                for row in 0..*rows {
                    for col in 0..*columns {
                        let x = (col as f32 - (*columns as f32 - 1.0) / 2.0) * layout.spacing;
                        let y = (row as f32 - (*rows as f32 - 1.0) / 2.0) * layout.spacing;
                        positions.push(Vector3::new(x, y, 0.0));
                    }
                }
            }
            LayoutArrangement::Circle { radius } => {
                let count = layout.elements.len();
                for i in 0..count {
                    let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
                    let x = angle.cos() * radius;
                    let y = angle.sin() * radius;
                    positions.push(Vector3::new(x, y, 0.0));
                }
            }
            LayoutArrangement::Line { direction } => {
                let count = layout.elements.len();
                for i in 0..count {
                    let offset = (i as f32 - (count as f32 - 1.0) / 2.0) * layout.spacing;
                    positions.push(direction * offset);
                }
            }
            _ => {}
        }

        Ok(positions)
    }

    /// Create a standard panel
    pub fn create_panel(&mut self, id: String, content: UIContent, position: Vector3<f32>) -> RobinResult<()> {
        let panel = SpatialUIElement {
            id: id.clone(),
            element_type: SpatialElementType::Panel {
                width: 0.5,
                height: 0.3,
                corner_radius: 0.02,
            },
            transform: SpatialTransform {
                position,
                ..Default::default()
            },
            content,
            interaction: InteractionState::default(),
            style: SpatialStyle::default(),
            anchoring: AnchoringMode::World(position),
            visibility: VisibilitySettings::default(),
        };

        self.add_element(panel)?;
        Ok(())
    }

    /// Create a 3D button
    pub fn create_button(&mut self, id: String, label: String, position: Vector3<f32>) -> RobinResult<()> {
        let button = SpatialUIElement {
            id: id.clone(),
            element_type: SpatialElementType::Button3D {
                width: 0.15,
                height: 0.05,
                depth: 0.02,
                press_depth: 0.01,
            },
            transform: SpatialTransform {
                position,
                ..Default::default()
            },
            content: UIContent::Text(label),
            interaction: InteractionState::default(),
            style: SpatialStyle {
                background_color: [0.2, 0.3, 0.8, 0.95],
                ..Default::default()
            },
            anchoring: AnchoringMode::World(position),
            visibility: VisibilitySettings::default(),
        };

        self.add_element(button)?;
        Ok(())
    }

    /// Start an animation
    pub fn animate(&mut self, element_id: String, animation_type: AnimationType, duration: f32) -> RobinResult<()> {
        let anim_id = format!("{}_{}", element_id, self.animation_system.animations.len());

        let animation = UIAnimation {
            element_id,
            animation_type,
            duration,
            current_time: 0.0,
            loop_mode: LoopMode::Once,
            easing: EasingFunction::EaseInOut,
        };

        self.animation_system.animations.insert(anim_id.clone(), animation);
        self.animation_system.active_animations.push(anim_id);

        Ok(())
    }
}

/// Builder for creating spatial UI elements
pub struct SpatialUIBuilder {
    element: SpatialUIElement,
}

impl SpatialUIBuilder {
    pub fn new(id: String) -> Self {
        Self {
            element: SpatialUIElement {
                id,
                element_type: SpatialElementType::Panel {
                    width: 0.5,
                    height: 0.3,
                    corner_radius: 0.02,
                },
                transform: SpatialTransform::default(),
                content: UIContent::Text(String::new()),
                interaction: InteractionState::default(),
                style: SpatialStyle::default(),
                anchoring: AnchoringMode::World(Vector3::zeros()),
                visibility: VisibilitySettings::default(),
            },
        }
    }

    pub fn element_type(mut self, element_type: SpatialElementType) -> Self {
        self.element.element_type = element_type;
        self
    }

    pub fn position(mut self, position: Vector3<f32>) -> Self {
        self.element.transform.position = position;
        self
    }

    pub fn content(mut self, content: UIContent) -> Self {
        self.element.content = content;
        self
    }

    pub fn style(mut self, style: SpatialStyle) -> Self {
        self.element.style = style;
        self
    }

    pub fn anchoring(mut self, anchoring: AnchoringMode) -> Self {
        self.element.anchoring = anchoring;
        self
    }

    pub fn build(self) -> SpatialUIElement {
        self.element
    }
}