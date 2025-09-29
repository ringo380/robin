/// Camera Tours System
///
/// Cinematic camera movements and guided tours for showcase presentations

use std::time::{Duration, Instant};
use std::collections::HashMap;
use cgmath::{Vector3, Matrix4, Quaternion, Rad, InnerSpace, EulerAngles};
use crate::engine::{
    graphics::Camera,
    animation::{Keyframe, AnimationCurve, EasingFunction},
};

/// Main Camera Tours controller
pub struct CameraTourController {
    // Available tours
    tours: HashMap<String, CameraTour>,
    current_tour: Option<String>,

    // Playback state
    playing: bool,
    paused: bool,
    current_time: f32,
    playback_speed: f32,
    loop_mode: bool,

    // Camera state
    camera_position: Vector3<f32>,
    camera_rotation: Quaternion<f32>,
    camera_fov: f32,

    // Interactive controls
    can_pause: bool,
    can_skip: bool,
    show_ui: bool,
    auto_advance: bool,

    // Timing
    last_update_time: Instant,
}

/// Complete camera tour with multiple segments
pub struct CameraTour {
    pub name: String,
    pub description: String,
    pub total_duration: Duration,
    pub segments: Vec<TourSegment>,
    pub narrator_text: Vec<NarratorCue>,
    pub tour_type: TourType,
    pub difficulty: TourDifficulty,
}

/// Individual segment of a tour
pub struct TourSegment {
    pub name: String,
    pub description: String,
    pub start_time: f32,
    pub duration: f32,
    pub camera_path: CameraPath,
    pub focus_points: Vec<FocusPoint>,
    pub tour_points: Vec<TourPoint>,
    pub transition_type: TransitionType,
}

/// Camera path through 3D space
pub struct CameraPath {
    pub keyframes: Vec<CameraKeyframe>,
    pub interpolation: PathInterpolation,
    pub smoothing: f32,
    pub look_ahead: f32,
}

/// Camera keyframe with position, rotation, and properties
#[derive(Clone)]
pub struct CameraKeyframe {
    pub time: f32,
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub fov: f32,
    pub focus_target: Option<Vector3<f32>>,
    pub transition_duration: f32,
    pub easing: EasingFunction,
}

/// Path interpolation methods
#[derive(Debug, Clone, PartialEq)]
pub enum PathInterpolation {
    Linear,
    CubicSpline,
    Bezier,
    CatmullRom,
    Hermite,
}

/// Focus point with contextual information
pub struct FocusPoint {
    pub position: Vector3<f32>,
    pub radius: f32,
    pub title: String,
    pub description: String,
    pub highlight_color: [f32; 4],
    pub show_label: bool,
    pub auto_focus_duration: f32,
}

/// Tour point for guided presentation
pub struct TourPoint {
    pub id: String,
    pub position: Vector3<f32>,
    pub camera_position: Vector3<f32>,
    pub camera_target: Vector3<f32>,
    pub title: String,
    pub description: String,
    pub stay_duration: f32,
    pub transition_in: TransitionType,
    pub transition_out: TransitionType,
    pub interactive_elements: Vec<InteractiveElement>,
}

/// Types of camera transitions
#[derive(Debug, Clone, PartialEq)]
pub enum TransitionType {
    Cut,
    Fade { duration: f32 },
    Dissolve { duration: f32 },
    Slide { direction: SlideDirection, duration: f32 },
    Zoom { zoom_point: Vector3<f32>, duration: f32 },
    Orbit { center: Vector3<f32>, duration: f32 },
    Fly { curve_height: f32, duration: f32 },
}

/// Slide directions for transitions
#[derive(Debug, Clone, PartialEq)]
pub enum SlideDirection {
    Left,
    Right,
    Up,
    Down,
    Forward,
    Backward,
}

/// Interactive elements during tours
pub struct InteractiveElement {
    pub element_type: InteractiveType,
    pub position: Vector3<f32>,
    pub size: Vector2<f32>,
    pub text: String,
    pub action: InteractiveAction,
    pub visible_duration: Option<f32>,
}

/// Types of interactive elements
#[derive(Debug, Clone, PartialEq)]
pub enum InteractiveType {
    Hotspot,
    InfoPanel,
    ClickableObject,
    ProgressIndicator,
    NavigationHint,
}

/// Actions for interactive elements
#[derive(Debug, Clone, PartialEq)]
pub enum InteractiveAction {
    ShowInfo(String),
    JumpToPoint(String),
    ToggleFeature(String),
    OpenURL(String),
    PlayAnimation(String),
}

/// Narrator cue for voiceover or text
pub struct NarratorCue {
    pub start_time: f32,
    pub duration: f32,
    pub text: String,
    pub voice_file: Option<String>,
    pub text_style: TextStyle,
    pub position: TextPosition,
}

/// Text styling for narrator
pub struct TextStyle {
    pub font_size: f32,
    pub color: [f32; 4],
    pub background_color: Option<[f32; 4]>,
    pub fade_in_duration: f32,
    pub fade_out_duration: f32,
}

/// Text positioning options
#[derive(Debug, Clone, PartialEq)]
pub enum TextPosition {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
    WorldPosition(Vector3<f32>),
}

/// Types of tours
#[derive(Debug, Clone, PartialEq)]
pub enum TourType {
    Overview,        // General engine showcase
    Technical,       // Deep technical features
    Interactive,     // User can interact
    Cinematic,       // Pure viewing experience
    Educational,     // Learning-focused
    Marketing,       // Sales presentation
}

/// Tour difficulty levels
#[derive(Debug, Clone, PartialEq)]
pub enum TourDifficulty {
    Beginner,     // 2-3 minutes, basic concepts
    Intermediate, // 5-7 minutes, detailed features
    Advanced,     // 8-12 minutes, technical depth
    Expert,       // 15+ minutes, comprehensive
}

/// Camera tour builder for easy tour creation
pub struct TourBuilder {
    tour: CameraTour,
    current_time: f32,
}

use cgmath::Vector2;

impl CameraTourController {
    pub fn new() -> Self {
        let mut controller = Self {
            tours: HashMap::new(),
            current_tour: None,
            playing: false,
            paused: false,
            current_time: 0.0,
            playback_speed: 1.0,
            loop_mode: false,
            camera_position: Vector3::new(0.0, 10.0, 20.0),
            camera_rotation: Quaternion::from(EulerAngles::new(Rad(-0.2), Rad(0.0), Rad(0.0))),
            camera_fov: 60.0,
            can_pause: true,
            can_skip: true,
            show_ui: true,
            auto_advance: true,
            last_update_time: Instant::now(),
        };

        // Create default tours
        controller.create_default_tours();
        controller
    }

    /// Create default showcase tours
    fn create_default_tours(&mut self) {
        // Overview Tour
        let overview_tour = TourBuilder::new("Engine Overview", "Complete Robin Engine showcase")
            .set_type(TourType::Overview)
            .set_difficulty(TourDifficulty::Intermediate)
            .add_segment("Welcome", 0.0, 10.0)
                .camera_path()
                    .keyframe(0.0, Vector3::new(0.0, 50.0, 100.0), "overview_start")
                    .keyframe(5.0, Vector3::new(30.0, 30.0, 50.0), "overview_mid")
                    .keyframe(10.0, Vector3::new(-20.0, 20.0, 80.0), "overview_end")
                    .finish_path()
                .add_narrator(0.0, 3.0, "Welcome to Robin Engine - a powerful 3D voxel game engine")
                .add_narrator(4.0, 4.0, "Built from scratch in Rust for maximum performance")
                .add_focus_point(Vector3::new(0.0, 10.0, 0.0), "Engine Core", "Main engine systems")
                .finish_segment()
            .add_segment("Voxel System", 10.0, 15.0)
                .camera_path()
                    .keyframe(0.0, Vector3::new(-20.0, 20.0, 80.0), "voxel_start")
                    .keyframe(7.5, Vector3::new(0.0, 15.0, 30.0), "voxel_close")
                    .keyframe(15.0, Vector3::new(20.0, 25.0, 40.0), "voxel_end")
                    .finish_path()
                .add_narrator(0.0, 5.0, "Our voxel system supports millions of blocks with real-time editing")
                .add_narrator(6.0, 4.0, "Features include frustum culling and greedy meshing for optimization")
                .add_focus_point(Vector3::new(5.0, 10.0, 5.0), "Voxel World", "Dynamic voxel terrain")
                .finish_segment()
            .add_segment("Rendering", 25.0, 20.0)
                .camera_path()
                    .keyframe(0.0, Vector3::new(20.0, 25.0, 40.0), "render_start")
                    .keyframe(10.0, Vector3::new(-10.0, 35.0, 60.0), "render_mid")
                    .keyframe(20.0, Vector3::new(0.0, 40.0, 80.0), "render_end")
                    .finish_path()
                .add_narrator(0.0, 6.0, "Advanced rendering with PBR materials and dynamic lighting")
                .add_narrator(7.0, 6.0, "Real-time shadows, reflections, and post-processing effects")
                .add_narrator(14.0, 5.0, "Optimized for Apple Silicon with Metal rendering")
                .add_focus_point(Vector3::new(0.0, 20.0, 0.0), "Renderer", "Graphics pipeline")
                .finish_segment()
            .build();

        self.tours.insert("overview".to_string(), overview_tour);

        // Technical Deep Dive Tour
        let technical_tour = TourBuilder::new("Technical Deep Dive", "In-depth technical features")
            .set_type(TourType::Technical)
            .set_difficulty(TourDifficulty::Advanced)
            .add_segment("Performance", 0.0, 30.0)
                .camera_path()
                    .keyframe(0.0, Vector3::new(100.0, 50.0, 100.0), "perf_overview")
                    .keyframe(10.0, Vector3::new(0.0, 30.0, 50.0), "perf_close")
                    .keyframe(20.0, Vector3::new(-50.0, 40.0, 70.0), "perf_profile")
                    .keyframe(30.0, Vector3::new(50.0, 60.0, 90.0), "perf_stats")
                    .finish_path()
                .add_narrator(0.0, 8.0, "Robin Engine maintains 60fps even with millions of voxels")
                .add_narrator(9.0, 8.0, "92% frustum culling efficiency reduces draw calls significantly")
                .add_narrator(18.0, 8.0, "Greedy meshing provides 60-80% vertex reduction")
                .add_narrator(27.0, 6.0, "Memory usage stays under 1GB for large worlds")
                .finish_segment()
            .add_segment("AI Systems", 30.0, 25.0)
                .camera_path()
                    .keyframe(0.0, Vector3::new(50.0, 60.0, 90.0), "ai_start")
                    .keyframe(12.5, Vector3::new(0.0, 20.0, 40.0), "ai_demo")
                    .keyframe(25.0, Vector3::new(-30.0, 35.0, 65.0), "ai_end")
                    .finish_path()
                .add_narrator(0.0, 8.0, "Integrated AI systems for procedural content generation")
                .add_narrator(9.0, 8.0, "Machine learning for intelligent world building")
                .add_narrator(18.0, 6.0, "Smart NPCs with behavior trees and pathfinding")
                .finish_segment()
            .build();

        self.tours.insert("technical".to_string(), technical_tour);

        // Interactive Tour
        let interactive_tour = TourBuilder::new("Interactive Demo", "Hands-on experience")
            .set_type(TourType::Interactive)
            .set_difficulty(TourDifficulty::Beginner)
            .add_segment("Building Basics", 0.0, 20.0)
                .camera_path()
                    .keyframe(0.0, Vector3::new(10.0, 15.0, 25.0), "build_start")
                    .keyframe(10.0, Vector3::new(5.0, 10.0, 15.0), "build_close")
                    .keyframe(20.0, Vector3::new(15.0, 20.0, 30.0), "build_complete")
                    .finish_path()
                .add_narrator(0.0, 5.0, "Try building your first structure")
                .add_narrator(6.0, 5.0, "Use the mouse to place and remove blocks")
                .add_narrator(12.0, 7.0, "Experiment with different materials and tools")
                .add_interactive_element(
                    InteractiveType::Hotspot,
                    Vector3::new(0.0, 5.0, 0.0),
                    "Click here to place a block",
                    InteractiveAction::ShowInfo("Place blocks by left-clicking".to_string())
                )
                .finish_segment()
            .add_segment("Advanced Tools", 20.0, 15.0)
                .camera_path()
                    .keyframe(0.0, Vector3::new(15.0, 20.0, 30.0), "tools_start")
                    .keyframe(7.5, Vector3::new(0.0, 25.0, 40.0), "tools_demo")
                    .keyframe(15.0, Vector3::new(-10.0, 30.0, 50.0), "tools_end")
                    .finish_path()
                .add_narrator(0.0, 5.0, "Discover advanced building tools")
                .add_narrator(6.0, 4.0, "Line tool, box tool, and symmetry modes")
                .add_narrator(11.0, 4.0, "Undo and redo support for safe experimentation")
                .finish_segment()
            .build();

        self.tours.insert("interactive".to_string(), interactive_tour);

        // Cinematic Showcase
        let cinematic_tour = TourBuilder::new("Cinematic Showcase", "Stunning visual presentation")
            .set_type(TourType::Cinematic)
            .set_difficulty(TourDifficulty::Intermediate)
            .add_segment("Dawn", 0.0, 25.0)
                .camera_path()
                    .keyframe(0.0, Vector3::new(80.0, 30.0, 80.0), "dawn_start")
                    .keyframe(8.0, Vector3::new(40.0, 20.0, 60.0), "dawn_approach")
                    .keyframe(16.0, Vector3::new(0.0, 15.0, 30.0), "dawn_close")
                    .keyframe(25.0, Vector3::new(-40.0, 25.0, 50.0), "dawn_end")
                    .finish_path()
                .add_narrator(0.0, 6.0, "As dawn breaks over the voxel world...")
                .add_narrator(10.0, 8.0, "Dynamic lighting creates realistic shadows and atmosphere")
                .add_narrator(20.0, 5.0, "Watch as the day-night cycle transforms the landscape")
                .finish_segment()
            .add_segment("Storm", 25.0, 20.0)
                .camera_path()
                    .keyframe(0.0, Vector3::new(-40.0, 25.0, 50.0), "storm_start")
                    .keyframe(6.0, Vector3::new(-20.0, 40.0, 70.0), "storm_rise")
                    .keyframe(14.0, Vector3::new(10.0, 60.0, 90.0), "storm_peak")
                    .keyframe(20.0, Vector3::new(30.0, 35.0, 60.0), "storm_end")
                    .finish_path()
                .add_narrator(0.0, 5.0, "Weather systems bring the world to life")
                .add_narrator(6.0, 6.0, "Rain, snow, and storms with particle effects")
                .add_narrator(13.0, 6.0, "Volumetric lighting through atmospheric effects")
                .finish_segment()
            .build();

        self.tours.insert("cinematic".to_string(), cinematic_tour);
    }

    /// Start playing a tour
    pub fn start_tour(&mut self, tour_name: &str) -> Result<(), String> {
        if self.tours.contains_key(tour_name) {
            self.current_tour = Some(tour_name.to_string());
            self.current_time = 0.0;
            self.playing = true;
            self.paused = false;
            self.last_update_time = Instant::now();
            Ok(())
        } else {
            Err(format!("Tour '{}' not found", tour_name))
        }
    }

    /// Stop current tour
    pub fn stop_tour(&mut self) {
        self.playing = false;
        self.paused = false;
        self.current_tour = None;
        self.current_time = 0.0;
    }

    /// Pause/resume tour
    pub fn toggle_pause(&mut self) {
        if self.can_pause {
            self.paused = !self.paused;
        }
    }

    /// Skip to next segment
    pub fn skip_to_next_segment(&mut self) {
        if self.can_skip && self.current_tour.is_some() {
            if let Some(tour) = self.get_current_tour() {
                // Find next segment
                for segment in &tour.segments {
                    if segment.start_time > self.current_time {
                        self.current_time = segment.start_time;
                        break;
                    }
                }
            }
        }
    }

    /// Jump to specific time
    pub fn seek_to_time(&mut self, time: f32) {
        if let Some(tour) = self.get_current_tour() {
            self.current_time = time.max(0.0).min(tour.total_duration.as_secs_f32());
        }
    }

    /// Update tour playback
    pub fn update(&mut self, delta_time: f32) -> CameraState {
        if !self.playing || self.paused || self.current_tour.is_none() {
            return CameraState {
                position: self.camera_position,
                rotation: self.camera_rotation,
                fov: self.camera_fov,
                focus_target: None,
                active_narrator: None,
                focus_points: Vec::new(),
                interactive_elements: Vec::new(),
            };
        }

        // Update playback time
        self.current_time += delta_time * self.playback_speed;

        let tour = self.get_current_tour().unwrap();

        // Check if tour is complete
        if self.current_time >= tour.total_duration.as_secs_f32() {
            if self.loop_mode {
                self.current_time = 0.0;
            } else {
                self.stop_tour();
                return CameraState {
                    position: self.camera_position,
                    rotation: self.camera_rotation,
                    fov: self.camera_fov,
                    focus_target: None,
                    active_narrator: None,
                    focus_points: Vec::new(),
                    interactive_elements: Vec::new(),
                };
            }
        }

        // Find current segment
        let current_segment = tour.segments.iter()
            .find(|segment| self.current_time >= segment.start_time &&
                          self.current_time < segment.start_time + segment.duration);

        if let Some(segment) = current_segment {
            let segment_time = self.current_time - segment.start_time;
            let segment_progress = segment_time / segment.duration;

            // Update camera from path
            self.update_camera_from_path(&segment.camera_path, segment_progress);

            // Get active narrator
            let active_narrator = tour.narrator_text.iter()
                .find(|cue| self.current_time >= cue.start_time &&
                           self.current_time < cue.start_time + cue.duration)
                .cloned();

            // Collect focus points for this segment
            let focus_points = segment.focus_points.clone();

            // Collect interactive elements
            let interactive_elements = segment.tour_points.iter()
                .flat_map(|point| &point.interactive_elements)
                .cloned()
                .collect();

            CameraState {
                position: self.camera_position,
                rotation: self.camera_rotation,
                fov: self.camera_fov,
                focus_target: None,
                active_narrator,
                focus_points,
                interactive_elements,
            }
        } else {
            // Between segments or no active segment
            CameraState {
                position: self.camera_position,
                rotation: self.camera_rotation,
                fov: self.camera_fov,
                focus_target: None,
                active_narrator: None,
                focus_points: Vec::new(),
                interactive_elements: Vec::new(),
            }
        }
    }

    /// Update camera position from path
    fn update_camera_from_path(&mut self, path: &CameraPath, progress: f32) {
        if path.keyframes.is_empty() {
            return;
        }

        if path.keyframes.len() == 1 {
            let keyframe = &path.keyframes[0];
            self.camera_position = keyframe.position;
            self.camera_rotation = keyframe.rotation;
            self.camera_fov = keyframe.fov;
            return;
        }

        // Find keyframes to interpolate between
        let total_time = path.keyframes.last().unwrap().time;
        let current_time = progress * total_time;

        let mut start_keyframe = &path.keyframes[0];
        let mut end_keyframe = &path.keyframes[1];

        for i in 0..path.keyframes.len() - 1 {
            if current_time >= path.keyframes[i].time && current_time <= path.keyframes[i + 1].time {
                start_keyframe = &path.keyframes[i];
                end_keyframe = &path.keyframes[i + 1];
                break;
            }
        }

        // Interpolate between keyframes
        let keyframe_duration = end_keyframe.time - start_keyframe.time;
        let keyframe_progress = if keyframe_duration > 0.0 {
            (current_time - start_keyframe.time) / keyframe_duration
        } else {
            0.0
        };

        // Apply easing
        let eased_progress = Self::apply_easing(keyframe_progress, &start_keyframe.easing);

        // Interpolate position
        self.camera_position = start_keyframe.position +
            (end_keyframe.position - start_keyframe.position) * eased_progress;

        // Interpolate rotation (slerp for quaternions)
        self.camera_rotation = start_keyframe.rotation.slerp(end_keyframe.rotation, eased_progress);

        // Interpolate FOV
        self.camera_fov = start_keyframe.fov + (end_keyframe.fov - start_keyframe.fov) * eased_progress;

        // Apply smoothing
        if path.smoothing > 0.0 {
            // Simple smoothing - in a real implementation this would use more sophisticated methods
            self.camera_position = self.camera_position * (1.0 - path.smoothing) +
                                  self.camera_position * path.smoothing;
        }
    }

    /// Apply easing function to progress
    fn apply_easing(progress: f32, easing: &EasingFunction) -> f32 {
        match easing {
            EasingFunction::Linear => progress,
            EasingFunction::EaseInQuad => progress * progress,
            EasingFunction::EaseOutQuad => 1.0 - (1.0 - progress) * (1.0 - progress),
            EasingFunction::EaseInOutQuad => {
                if progress < 0.5 {
                    2.0 * progress * progress
                } else {
                    1.0 - 2.0 * (1.0 - progress) * (1.0 - progress)
                }
            }
            EasingFunction::EaseInCubic => progress * progress * progress,
            EasingFunction::EaseOutCubic => 1.0 - (1.0 - progress).powi(3),
            EasingFunction::EaseInOutCubic => {
                if progress < 0.5 {
                    4.0 * progress * progress * progress
                } else {
                    1.0 - 4.0 * (1.0 - progress).powi(3)
                }
            }
            _ => progress, // Default to linear for other easing types
        }
    }

    /// Get current tour
    fn get_current_tour(&self) -> Option<&CameraTour> {
        self.current_tour.as_ref().and_then(|name| self.tours.get(name))
    }

    /// Get available tours
    pub fn get_available_tours(&self) -> Vec<&str> {
        self.tours.keys().map(|s| s.as_str()).collect()
    }

    /// Get tour progress (0.0 to 1.0)
    pub fn get_progress(&self) -> f32 {
        if let Some(tour) = self.get_current_tour() {
            self.current_time / tour.total_duration.as_secs_f32()
        } else {
            0.0
        }
    }

    /// Check if currently playing
    pub fn is_playing(&self) -> bool {
        self.playing && !self.paused
    }

    /// Check if paused
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Get current camera state
    pub fn get_camera_state(&self) -> CameraState {
        CameraState {
            position: self.camera_position,
            rotation: self.camera_rotation,
            fov: self.camera_fov,
            focus_target: None,
            active_narrator: None,
            focus_points: Vec::new(),
            interactive_elements: Vec::new(),
        }
    }

    /// Set playback speed
    pub fn set_playback_speed(&mut self, speed: f32) {
        self.playback_speed = speed.max(0.1).min(5.0);
    }

    /// Enable/disable loop mode
    pub fn set_loop_mode(&mut self, enabled: bool) {
        self.loop_mode = enabled;
    }

    /// Handle user interaction
    pub fn handle_interaction(&mut self, interaction: TourInteraction) {
        match interaction {
            TourInteraction::Pause => self.toggle_pause(),
            TourInteraction::Play => {
                self.paused = false;
                self.playing = true;
            }
            TourInteraction::Stop => self.stop_tour(),
            TourInteraction::Skip => self.skip_to_next_segment(),
            TourInteraction::Seek(time) => self.seek_to_time(time),
            TourInteraction::SetSpeed(speed) => self.set_playback_speed(speed),
        }
    }
}

/// Current camera state from tour
pub struct CameraState {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub fov: f32,
    pub focus_target: Option<Vector3<f32>>,
    pub active_narrator: Option<NarratorCue>,
    pub focus_points: Vec<FocusPoint>,
    pub interactive_elements: Vec<InteractiveElement>,
}

/// User interactions with tours
pub enum TourInteraction {
    Pause,
    Play,
    Stop,
    Skip,
    Seek(f32),
    SetSpeed(f32),
}

impl TourBuilder {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            tour: CameraTour {
                name: name.to_string(),
                description: description.to_string(),
                total_duration: Duration::from_secs(0),
                segments: Vec::new(),
                narrator_text: Vec::new(),
                tour_type: TourType::Overview,
                difficulty: TourDifficulty::Beginner,
            },
            current_time: 0.0,
        }
    }

    pub fn set_type(mut self, tour_type: TourType) -> Self {
        self.tour.tour_type = tour_type;
        self
    }

    pub fn set_difficulty(mut self, difficulty: TourDifficulty) -> Self {
        self.tour.difficulty = difficulty;
        self
    }

    pub fn add_segment(mut self, name: &str, start_time: f32, duration: f32) -> SegmentBuilder {
        SegmentBuilder {
            tour_builder: self,
            segment: TourSegment {
                name: name.to_string(),
                description: String::new(),
                start_time,
                duration,
                camera_path: CameraPath {
                    keyframes: Vec::new(),
                    interpolation: PathInterpolation::CubicSpline,
                    smoothing: 0.1,
                    look_ahead: 1.0,
                },
                focus_points: Vec::new(),
                tour_points: Vec::new(),
                transition_type: TransitionType::Fade { duration: 1.0 },
            },
        }
    }

    pub fn build(mut self) -> CameraTour {
        // Calculate total duration
        if let Some(last_segment) = self.tour.segments.last() {
            self.tour.total_duration = Duration::from_secs_f32(last_segment.start_time + last_segment.duration);
        }
        self.tour
    }
}

/// Builder for tour segments
pub struct SegmentBuilder {
    tour_builder: TourBuilder,
    segment: TourSegment,
}

impl SegmentBuilder {
    pub fn camera_path(self) -> CameraPathBuilder {
        CameraPathBuilder {
            segment_builder: self,
            path: CameraPath {
                keyframes: Vec::new(),
                interpolation: PathInterpolation::CubicSpline,
                smoothing: 0.1,
                look_ahead: 1.0,
            },
        }
    }

    pub fn add_narrator(mut self, start_time: f32, duration: f32, text: &str) -> Self {
        self.tour_builder.tour.narrator_text.push(NarratorCue {
            start_time: self.segment.start_time + start_time,
            duration,
            text: text.to_string(),
            voice_file: None,
            text_style: TextStyle {
                font_size: 16.0,
                color: [1.0, 1.0, 1.0, 1.0],
                background_color: Some([0.0, 0.0, 0.0, 0.7]),
                fade_in_duration: 0.5,
                fade_out_duration: 0.5,
            },
            position: TextPosition::BottomCenter,
        });
        self
    }

    pub fn add_focus_point(mut self, position: Vector3<f32>, title: &str, description: &str) -> Self {
        self.segment.focus_points.push(FocusPoint {
            position,
            radius: 5.0,
            title: title.to_string(),
            description: description.to_string(),
            highlight_color: [1.0, 1.0, 0.0, 0.5],
            show_label: true,
            auto_focus_duration: 2.0,
        });
        self
    }

    pub fn add_interactive_element(mut self, element_type: InteractiveType, position: Vector3<f32>, text: &str, action: InteractiveAction) -> Self {
        self.segment.tour_points.push(TourPoint {
            id: format!("interactive_{}", self.segment.tour_points.len()),
            position,
            camera_position: position + Vector3::new(5.0, 5.0, 5.0),
            camera_target: position,
            title: text.to_string(),
            description: String::new(),
            stay_duration: 3.0,
            transition_in: TransitionType::Fade { duration: 0.5 },
            transition_out: TransitionType::Fade { duration: 0.5 },
            interactive_elements: vec![InteractiveElement {
                element_type,
                position,
                size: Vector2::new(100.0, 50.0),
                text: text.to_string(),
                action,
                visible_duration: None,
            }],
        });
        self
    }

    pub fn finish_segment(mut self) -> TourBuilder {
        self.tour_builder.tour.segments.push(self.segment);
        self.tour_builder
    }
}

/// Builder for camera paths
pub struct CameraPathBuilder {
    segment_builder: SegmentBuilder,
    path: CameraPath,
}

impl CameraPathBuilder {
    pub fn keyframe(mut self, time: f32, position: Vector3<f32>, name: &str) -> Self {
        self.path.keyframes.push(CameraKeyframe {
            time,
            position,
            rotation: Quaternion::from(EulerAngles::new(Rad(0.0), Rad(0.0), Rad(0.0))),
            fov: 60.0,
            focus_target: None,
            transition_duration: 1.0,
            easing: EasingFunction::EaseInOutCubic,
        });
        self
    }

    pub fn finish_path(mut self) -> SegmentBuilder {
        self.segment_builder.segment.camera_path = self.path;
        self.segment_builder
    }
}