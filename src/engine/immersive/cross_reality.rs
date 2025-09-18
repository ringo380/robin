// Robin Engine - Cross-Reality Bridge
// Seamless transitions between Desktop, VR, AR, and Mixed Reality modes

use nalgebra::{Vector3, UnitQuaternion, Matrix4, Point3};
use std::collections::HashMap;
use crate::engine::error::RobinResult;
use super::{ImmersiveMode, TrackingData};

/// Cross-Reality Bridge manages transitions between different reality modes
#[derive(Debug)]
pub struct CrossRealityBridge {
    pub current_mode: ImmersiveMode,
    pub target_mode: Option<ImmersiveMode>,
    pub transition_state: TransitionState,
    pub mode_capabilities: HashMap<ImmersiveMode, ModeCapabilities>,
    pub content_adapters: HashMap<String, ContentAdapter>,
    pub spatial_mapping: SpatialMapping,
    pub user_preferences: UserPreferences,
    pub transition_animations: TransitionAnimations,
}

/// State of mode transition
#[derive(Debug, Clone)]
pub struct TransitionState {
    pub is_transitioning: bool,
    pub progress: f32,
    pub duration: f32,
    pub transition_type: TransitionType,
    pub fade_level: f32,
    pub scale_factor: f32,
    pub repositioning_ui: bool,
}

impl Default for TransitionState {
    fn default() -> Self {
        Self {
            is_transitioning: false,
            progress: 0.0,
            duration: 2.0,
            transition_type: TransitionType::Fade,
            fade_level: 0.0,
            scale_factor: 1.0,
            repositioning_ui: false,
        }
    }
}

/// Types of transitions between modes
#[derive(Debug, Clone)]
pub enum TransitionType {
    /// Simple fade to black and back
    Fade,
    /// Iris-style transition
    Iris,
    /// Scale world in/out
    Scale,
    /// Blend between modes
    Blend,
    /// Portal-style transition
    Portal,
    /// Instant switch (no animation)
    Instant,
}

/// Capabilities of each reality mode
#[derive(Debug, Clone)]
pub struct ModeCapabilities {
    pub supports_6dof: bool,
    pub supports_hand_tracking: bool,
    pub supports_eye_tracking: bool,
    pub supports_spatial_audio: bool,
    pub supports_haptics: bool,
    pub supports_passthrough: bool,
    pub max_fov: f32,
    pub max_render_distance: f32,
    pub spatial_resolution: f32,
    pub tracking_accuracy: TrackingAccuracy,
}

/// Tracking accuracy levels
#[derive(Debug, Clone)]
pub enum TrackingAccuracy {
    None,
    Basic,
    Accurate,
    Precise,
}

/// Adapts content between different reality modes
#[derive(Debug, Clone)]
pub struct ContentAdapter {
    pub id: String,
    pub source_mode: ImmersiveMode,
    pub target_mode: ImmersiveMode,
    pub adapter_type: AdapterType,
    pub scale_factor: f32,
    pub position_offset: Vector3<f32>,
    pub rotation_offset: UnitQuaternion<f32>,
    pub ui_layout: UILayoutMode,
}

/// Types of content adaptation
#[derive(Debug, Clone)]
pub enum AdapterType {
    /// UI elements positioning
    UI,
    /// 3D world objects
    World,
    /// Camera/viewport settings
    Camera,
    /// Audio spatialization
    Audio,
    /// Input mapping
    Input,
    /// Physics scale
    Physics,
}

/// UI layout modes for different reality types
#[derive(Debug, Clone)]
pub enum UILayoutMode {
    /// Traditional flat screen layout
    Screen2D,
    /// Floating panels in 3D space
    Floating3D,
    /// Attached to hands/controllers
    HandAttached,
    /// Head-locked overlay
    HeadLocked,
    /// World-space attached to surfaces
    WorldAttached,
    /// Adaptive layout that changes based on context
    Adaptive,
}

/// Spatial mapping for cross-reality content
#[derive(Debug)]
pub struct SpatialMapping {
    pub coordinate_system: CoordinateSystem,
    pub scale_reference: ScaleReference,
    pub anchor_points: HashMap<String, SpatialAnchor>,
    pub room_bounds: Option<RoomBounds>,
    pub play_area: Option<PlayArea>,
}

/// Coordinate system types
#[derive(Debug, Clone)]
pub enum CoordinateSystem {
    /// Room-scale tracking
    RoomScale,
    /// Seated experience
    Seated,
    /// Standing experience
    Standing,
    /// World-scale AR
    WorldScale,
    /// Desktop screen coordinates
    Screen,
}

/// Reference for scale calculations
#[derive(Debug, Clone)]
pub enum ScaleReference {
    /// User height as reference
    UserHeight(f32),
    /// Room size as reference
    RoomSize(Vector3<f32>),
    /// Object size as reference
    ObjectSize(f32),
    /// Arbitrary scale
    Custom(f32),
}

/// Spatial anchor for cross-reality alignment
#[derive(Debug, Clone)]
pub struct SpatialAnchor {
    pub id: String,
    pub position: Vector3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub confidence: f32,
    pub anchor_type: AnchorType,
    pub persistence: PersistenceLevel,
    pub associated_content: Vec<String>,
}

/// Types of spatial anchors
#[derive(Debug, Clone)]
pub enum AnchorType {
    /// Virtual anchor in space
    Virtual,
    /// Physical object anchor
    Physical,
    /// Surface-based anchor
    Surface,
    /// User-defined anchor
    UserDefined,
    /// System-generated anchor
    System,
}

/// Persistence levels for anchors
#[derive(Debug, Clone)]
pub enum PersistenceLevel {
    /// Temporary (session only)
    Session,
    /// Persistent (saved locally)
    Local,
    /// Cloud-synced (shared)
    Cloud,
}

/// Room bounds for spatial awareness
#[derive(Debug, Clone)]
pub struct RoomBounds {
    pub vertices: Vec<Vector3<f32>>,
    pub floor_height: f32,
    pub ceiling_height: f32,
    pub walls: Vec<Wall>,
}

/// Wall definition
#[derive(Debug, Clone)]
pub struct Wall {
    pub start: Vector3<f32>,
    pub end: Vector3<f32>,
    pub height: f32,
    pub openings: Vec<Opening>,
}

/// Opening in wall (door, window)
#[derive(Debug, Clone)]
pub struct Opening {
    pub position: Vector3<f32>,
    pub size: Vector3<f32>,
    pub opening_type: OpeningType,
}

/// Types of openings
#[derive(Debug, Clone)]
pub enum OpeningType {
    Door,
    Window,
    Archway,
    Other,
}

/// Play area definition
#[derive(Debug, Clone)]
pub struct PlayArea {
    pub center: Vector3<f32>,
    pub size: Vector3<f32>,
    pub boundaries: Vec<Vector3<f32>>,
    pub safe_zone: f32,
}

/// User preferences for cross-reality experience
#[derive(Debug, Clone)]
pub struct UserPreferences {
    pub preferred_mode: ImmersiveMode,
    pub comfort_settings: ComfortSettings,
    pub accessibility: AccessibilitySettings,
    pub ui_scale: f32,
    pub movement_speed: f32,
    pub auto_transition: bool,
    pub transition_duration: f32,
}

/// Comfort settings for immersive experiences
#[derive(Debug, Clone)]
pub struct ComfortSettings {
    pub vignetting: bool,
    pub snap_turning: bool,
    pub teleport_movement: bool,
    pub reduce_motion: bool,
    pub comfort_mode_intensity: f32,
}

/// Accessibility settings
#[derive(Debug, Clone)]
pub struct AccessibilitySettings {
    pub subtitles: bool,
    pub high_contrast: bool,
    pub large_text: bool,
    pub audio_descriptions: bool,
    pub voice_commands: bool,
    pub gesture_alternatives: bool,
}

/// Transition animations between modes
#[derive(Debug)]
pub struct TransitionAnimations {
    pub fade_animation: FadeAnimation,
    pub scale_animation: ScaleAnimation,
    pub portal_animation: PortalAnimation,
    pub ui_repositioning: UIRepositioning,
}

/// Fade transition animation
#[derive(Debug, Clone)]
pub struct FadeAnimation {
    pub duration: f32,
    pub fade_to_color: [f32; 4],
    pub easing: EasingType,
}

/// Scale transition animation
#[derive(Debug, Clone)]
pub struct ScaleAnimation {
    pub duration: f32,
    pub scale_factor_range: (f32, f32),
    pub focal_point: Vector3<f32>,
    pub easing: EasingType,
}

/// Portal transition animation
#[derive(Debug, Clone)]
pub struct PortalAnimation {
    pub duration: f32,
    pub portal_size: f32,
    pub portal_position: Vector3<f32>,
    pub particle_effects: bool,
}

/// UI repositioning during transitions
#[derive(Debug, Clone)]
pub struct UIRepositioning {
    pub duration: f32,
    pub smooth_repositioning: bool,
    pub maintain_relative_positions: bool,
    pub scale_ui_elements: bool,
}

/// Easing types for animations
#[derive(Debug, Clone)]
pub enum EasingType {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bounce,
    Elastic,
}

impl CrossRealityBridge {
    pub fn new() -> Self {
        let mut mode_capabilities = HashMap::new();

        // Define capabilities for each mode
        mode_capabilities.insert(ImmersiveMode::Desktop, ModeCapabilities {
            supports_6dof: false,
            supports_hand_tracking: false,
            supports_eye_tracking: false,
            supports_spatial_audio: false,
            supports_haptics: false,
            supports_passthrough: false,
            max_fov: 110.0,
            max_render_distance: 1000.0,
            spatial_resolution: 0.0,
            tracking_accuracy: TrackingAccuracy::None,
        });

        mode_capabilities.insert(ImmersiveMode::VR, ModeCapabilities {
            supports_6dof: true,
            supports_hand_tracking: true,
            supports_eye_tracking: true,
            supports_spatial_audio: true,
            supports_haptics: true,
            supports_passthrough: false,
            max_fov: 120.0,
            max_render_distance: 100.0,
            spatial_resolution: 0.01,
            tracking_accuracy: TrackingAccuracy::Precise,
        });

        mode_capabilities.insert(ImmersiveMode::AR, ModeCapabilities {
            supports_6dof: true,
            supports_hand_tracking: true,
            supports_eye_tracking: false,
            supports_spatial_audio: true,
            supports_haptics: false,
            supports_passthrough: true,
            max_fov: 90.0,
            max_render_distance: 50.0,
            spatial_resolution: 0.05,
            tracking_accuracy: TrackingAccuracy::Accurate,
        });

        mode_capabilities.insert(ImmersiveMode::MixedReality, ModeCapabilities {
            supports_6dof: true,
            supports_hand_tracking: true,
            supports_eye_tracking: true,
            supports_spatial_audio: true,
            supports_haptics: true,
            supports_passthrough: true,
            max_fov: 110.0,
            max_render_distance: 100.0,
            spatial_resolution: 0.02,
            tracking_accuracy: TrackingAccuracy::Precise,
        });

        Self {
            current_mode: ImmersiveMode::Desktop,
            target_mode: None,
            transition_state: TransitionState::default(),
            mode_capabilities,
            content_adapters: HashMap::new(),
            spatial_mapping: SpatialMapping {
                coordinate_system: CoordinateSystem::Screen,
                scale_reference: ScaleReference::Custom(1.0),
                anchor_points: HashMap::new(),
                room_bounds: None,
                play_area: None,
            },
            user_preferences: UserPreferences {
                preferred_mode: ImmersiveMode::Desktop,
                comfort_settings: ComfortSettings {
                    vignetting: true,
                    snap_turning: false,
                    teleport_movement: false,
                    reduce_motion: false,
                    comfort_mode_intensity: 0.5,
                },
                accessibility: AccessibilitySettings {
                    subtitles: false,
                    high_contrast: false,
                    large_text: false,
                    audio_descriptions: false,
                    voice_commands: false,
                    gesture_alternatives: true,
                },
                ui_scale: 1.0,
                movement_speed: 1.0,
                auto_transition: false,
                transition_duration: 2.0,
            },
            transition_animations: TransitionAnimations {
                fade_animation: FadeAnimation {
                    duration: 1.5,
                    fade_to_color: [0.0, 0.0, 0.0, 1.0],
                    easing: EasingType::EaseInOut,
                },
                scale_animation: ScaleAnimation {
                    duration: 2.0,
                    scale_factor_range: (0.1, 2.0),
                    focal_point: Vector3::zeros(),
                    easing: EasingType::EaseInOut,
                },
                portal_animation: PortalAnimation {
                    duration: 3.0,
                    portal_size: 2.0,
                    portal_position: Vector3::new(0.0, 0.0, -2.0),
                    particle_effects: true,
                },
                ui_repositioning: UIRepositioning {
                    duration: 1.0,
                    smooth_repositioning: true,
                    maintain_relative_positions: true,
                    scale_ui_elements: true,
                },
            },
        }
    }

    /// Initialize the cross-reality bridge
    pub fn initialize(&mut self) -> RobinResult<()> {
        // Set up default content adapters
        self.setup_default_adapters()?;

        // Initialize spatial mapping
        self.initialize_spatial_mapping()?;

        Ok(())
    }

    /// Request transition to a new immersive mode
    pub fn request_mode_transition(&mut self, target_mode: ImmersiveMode, transition_type: TransitionType) -> RobinResult<()> {
        if self.transition_state.is_transitioning {
            return Ok(()); // Already transitioning
        }

        if target_mode == self.current_mode {
            return Ok(()); // Already in target mode
        }

        // Check if transition is supported
        if !self.is_transition_supported(&self.current_mode, &target_mode)? {
            return Err(crate::engine::error::RobinError::Other(
                format!("Transition from {:?} to {:?} is not supported", self.current_mode, target_mode)
            ));
        }

        self.target_mode = Some(target_mode);
        self.transition_state = TransitionState {
            is_transitioning: true,
            progress: 0.0,
            duration: self.user_preferences.transition_duration,
            transition_type,
            fade_level: 0.0,
            scale_factor: 1.0,
            repositioning_ui: true,
        };

        println!("🔄 Starting transition from {:?} to {:?}", self.current_mode, target_mode);
        Ok(())
    }

    /// Update the cross-reality bridge
    pub fn update(&mut self, delta_time: f32, tracking_data: &TrackingData) -> RobinResult<Vec<CrossRealityEvent>> {
        let mut events = Vec::new();

        if self.transition_state.is_transitioning {
            events.extend(self.update_transition(delta_time)?);
        }

        // Update spatial mapping based on tracking data
        self.update_spatial_mapping(tracking_data)?;

        // Check for automatic transitions based on user preferences
        if self.user_preferences.auto_transition {
            events.extend(self.check_auto_transitions(tracking_data)?);
        }

        Ok(events)
    }

    /// Update ongoing transition
    fn update_transition(&mut self, delta_time: f32) -> RobinResult<Vec<CrossRealityEvent>> {
        let mut events = Vec::new();

        self.transition_state.progress += delta_time / self.transition_state.duration;

        if self.transition_state.progress >= 1.0 {
            // Transition complete
            self.transition_state.progress = 1.0;

            if let Some(target_mode) = self.target_mode.take() {
                let old_mode = self.current_mode;
                self.current_mode = target_mode;

                events.push(CrossRealityEvent::TransitionCompleted {
                    from_mode: old_mode,
                    to_mode: target_mode,
                });

                println!("✅ Transition completed: {:?} -> {:?}", old_mode, target_mode);
            }

            self.transition_state.is_transitioning = false;
            self.transition_state.repositioning_ui = false;
        } else {
            // Update transition effects
            self.update_transition_effects()?;

            events.push(CrossRealityEvent::TransitionProgress {
                progress: self.transition_state.progress,
                transition_type: self.transition_state.transition_type.clone(),
            });
        }

        Ok(events)
    }

    /// Update transition visual effects
    fn update_transition_effects(&mut self) -> RobinResult<()> {
        let progress = self.transition_state.progress;
        let eased_progress = self.apply_easing(progress, &EasingType::EaseInOut);

        match self.transition_state.transition_type {
            TransitionType::Fade => {
                // Fade to black and back
                self.transition_state.fade_level = if progress < 0.5 {
                    eased_progress * 2.0
                } else {
                    2.0 - (eased_progress * 2.0)
                };
            }
            TransitionType::Scale => {
                // Scale world
                let (min_scale, max_scale) = self.transition_animations.scale_animation.scale_factor_range;
                self.transition_state.scale_factor = if progress < 0.5 {
                    1.0 + (min_scale - 1.0) * (eased_progress * 2.0)
                } else {
                    min_scale + (max_scale - min_scale) * ((eased_progress - 0.5) * 2.0)
                };
            }
            TransitionType::Iris => {
                // Iris effect (similar to fade but circular)
                self.transition_state.fade_level = if progress < 0.5 {
                    eased_progress * 2.0
                } else {
                    2.0 - (eased_progress * 2.0)
                };
            }
            _ => {}
        }

        Ok(())
    }

    /// Apply easing function to value
    fn apply_easing(&self, t: f32, easing: &EasingType) -> f32 {
        match easing {
            EasingType::Linear => t,
            EasingType::EaseIn => t * t,
            EasingType::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            EasingType::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            EasingType::Bounce => {
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
            EasingType::Elastic => {
                if t == 0.0 || t == 1.0 {
                    t
                } else {
                    let p = 0.3;
                    -(2.0_f32.powf(10.0 * (t - 1.0)) * ((t - 1.0 - p / 4.0) * (2.0 * std::f32::consts::PI) / p).sin())
                }
            }
        }
    }

    /// Check if transition between modes is supported
    fn is_transition_supported(&self, from: &ImmersiveMode, to: &ImmersiveMode) -> RobinResult<bool> {
        // All transitions are supported in this implementation
        // In a real system, you might check hardware capabilities
        Ok(from != to)
    }

    /// Set up default content adapters
    fn setup_default_adapters(&mut self) -> RobinResult<()> {
        // Desktop to VR adapter
        self.content_adapters.insert(
            "desktop_to_vr".to_string(),
            ContentAdapter {
                id: "desktop_to_vr".to_string(),
                source_mode: ImmersiveMode::Desktop,
                target_mode: ImmersiveMode::VR,
                adapter_type: AdapterType::UI,
                scale_factor: 0.001, // Scale down from pixels to meters
                position_offset: Vector3::new(0.0, 1.6, -2.0), // In front of user
                rotation_offset: UnitQuaternion::identity(),
                ui_layout: UILayoutMode::Floating3D,
            }
        );

        // VR to AR adapter
        self.content_adapters.insert(
            "vr_to_ar".to_string(),
            ContentAdapter {
                id: "vr_to_ar".to_string(),
                source_mode: ImmersiveMode::VR,
                target_mode: ImmersiveMode::AR,
                adapter_type: AdapterType::UI,
                scale_factor: 0.8, // Slightly smaller for AR
                position_offset: Vector3::new(0.0, 0.0, -1.0),
                rotation_offset: UnitQuaternion::identity(),
                ui_layout: UILayoutMode::WorldAttached,
            }
        );

        Ok(())
    }

    /// Initialize spatial mapping
    fn initialize_spatial_mapping(&mut self) -> RobinResult<()> {
        // Set up coordinate system based on current mode
        match self.current_mode {
            ImmersiveMode::Desktop => {
                self.spatial_mapping.coordinate_system = CoordinateSystem::Screen;
                self.spatial_mapping.scale_reference = ScaleReference::Custom(1.0);
            }
            ImmersiveMode::VR => {
                self.spatial_mapping.coordinate_system = CoordinateSystem::RoomScale;
                self.spatial_mapping.scale_reference = ScaleReference::UserHeight(1.7);
            }
            ImmersiveMode::AR => {
                self.spatial_mapping.coordinate_system = CoordinateSystem::WorldScale;
                self.spatial_mapping.scale_reference = ScaleReference::UserHeight(1.7);
            }
            _ => {}
        }

        Ok(())
    }

    /// Update spatial mapping based on tracking data
    fn update_spatial_mapping(&mut self, _tracking_data: &TrackingData) -> RobinResult<()> {
        // Update room bounds and play area based on tracking data
        // This would integrate with the actual tracking system
        Ok(())
    }

    /// Check for automatic transitions
    fn check_auto_transitions(&self, _tracking_data: &TrackingData) -> RobinResult<Vec<CrossRealityEvent>> {
        // Implement logic for automatic mode switching based on context
        // For example, switch to AR when hand tracking is detected
        Ok(Vec::new())
    }

    /// Get adapter for mode transition
    pub fn get_adapter(&self, from: &ImmersiveMode, to: &ImmersiveMode) -> Option<&ContentAdapter> {
        let key = format!("{:?}_to_{:?}", from, to).to_lowercase();
        self.content_adapters.get(&key)
    }

    /// Add custom content adapter
    pub fn add_adapter(&mut self, adapter: ContentAdapter) {
        self.content_adapters.insert(adapter.id.clone(), adapter);
    }

    /// Get current mode capabilities
    pub fn get_current_capabilities(&self) -> Option<&ModeCapabilities> {
        self.mode_capabilities.get(&self.current_mode)
    }

    /// Update user preferences
    pub fn update_preferences(&mut self, preferences: UserPreferences) {
        self.user_preferences = preferences;
    }

    /// Create spatial anchor
    pub fn create_spatial_anchor(&mut self, id: String, position: Vector3<f32>, rotation: UnitQuaternion<f32>) -> RobinResult<()> {
        let anchor = SpatialAnchor {
            id: id.clone(),
            position,
            rotation,
            confidence: 1.0,
            anchor_type: AnchorType::UserDefined,
            persistence: PersistenceLevel::Session,
            associated_content: Vec::new(),
        };

        self.spatial_mapping.anchor_points.insert(id, anchor);
        Ok(())
    }

    /// Get spatial anchor by ID
    pub fn get_spatial_anchor(&self, id: &str) -> Option<&SpatialAnchor> {
        self.spatial_mapping.anchor_points.get(id)
    }

    /// Update play area
    pub fn update_play_area(&mut self, center: Vector3<f32>, size: Vector3<f32>, boundaries: Vec<Vector3<f32>>) {
        self.spatial_mapping.play_area = Some(PlayArea {
            center,
            size,
            boundaries,
            safe_zone: 0.5, // 50cm safety margin
        });
    }

    /// Check if position is within play area
    pub fn is_position_safe(&self, position: Vector3<f32>) -> bool {
        if let Some(play_area) = &self.spatial_mapping.play_area {
            let distance = (position - play_area.center).magnitude();
            let max_distance = (play_area.size.magnitude() / 2.0) - play_area.safe_zone;
            distance <= max_distance
        } else {
            true // No play area defined, assume safe
        }
    }

    /// Handle mode change event (compatibility method)
    pub fn handle_mode_change(&mut self, previous_mode: ImmersiveMode, mode: ImmersiveMode) -> RobinResult<()> {
        if previous_mode != mode {
            self.request_mode_transition(mode, TransitionType::Fade)?;
        }
        Ok(())
    }
}

/// Events generated by the cross-reality system
#[derive(Debug, Clone)]
pub enum CrossRealityEvent {
    /// Mode transition started
    TransitionStarted {
        from_mode: ImmersiveMode,
        to_mode: ImmersiveMode,
        transition_type: TransitionType,
    },
    /// Mode transition in progress
    TransitionProgress {
        progress: f32,
        transition_type: TransitionType,
    },
    /// Mode transition completed
    TransitionCompleted {
        from_mode: ImmersiveMode,
        to_mode: ImmersiveMode,
    },
    /// Spatial anchor created
    SpatialAnchorCreated {
        anchor_id: String,
        position: Vector3<f32>,
    },
    /// Play area updated
    PlayAreaUpdated {
        center: Vector3<f32>,
        size: Vector3<f32>,
    },
    /// User left safe area
    SafeAreaViolation {
        position: Vector3<f32>,
        distance_from_safe: f32,
    },
    /// Content adapted for new mode
    ContentAdapted {
        content_id: String,
        from_mode: ImmersiveMode,
        to_mode: ImmersiveMode,
    },
}

/// Builder for creating cross-reality experiences
pub struct CrossRealityBuilder {
    bridge: CrossRealityBridge,
}

impl CrossRealityBuilder {
    pub fn new() -> Self {
        Self {
            bridge: CrossRealityBridge::new(),
        }
    }

    pub fn with_initial_mode(mut self, mode: ImmersiveMode) -> Self {
        self.bridge.current_mode = mode;
        self
    }

    pub fn with_user_preferences(mut self, preferences: UserPreferences) -> Self {
        self.bridge.user_preferences = preferences;
        self
    }

    pub fn with_adapter(mut self, adapter: ContentAdapter) -> Self {
        self.bridge.content_adapters.insert(adapter.id.clone(), adapter);
        self
    }

    pub fn build(self) -> CrossRealityBridge {
        self.bridge
    }
}

impl Default for CrossRealityBridge {
    fn default() -> Self {
        Self::new()
    }
}