use crate::engine::error::RobinResult;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationSystemConfig {
    pub max_animated_objects: u32,
    pub max_bones_per_skeleton: u32,
    pub enable_bone_animations: bool,
    pub enable_morph_targets: bool,
    pub enable_procedural_animation: bool,
    pub enable_physics_animation: bool,
    pub animation_quality: AnimationQuality,
    pub interpolation_method: InterpolationMethod,
}

impl Default for AnimationSystemConfig {
    fn default() -> Self {
        Self {
            max_animated_objects: 1000,
            max_bones_per_skeleton: 256,
            enable_bone_animations: true,
            enable_morph_targets: true,
            enable_procedural_animation: true,
            enable_physics_animation: false,
            animation_quality: AnimationQuality::High,
            interpolation_method: InterpolationMethod::Linear,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnimationQuality {
    Ultra,
    High,
    Medium,
    Low,
}

impl AnimationQuality {
    pub fn get_update_frequency(&self) -> f32 {
        match self {
            AnimationQuality::Ultra => 120.0, // 120 FPS
            AnimationQuality::High => 60.0,   // 60 FPS
            AnimationQuality::Medium => 30.0, // 30 FPS
            AnimationQuality::Low => 15.0,    // 15 FPS
        }
    }

    pub fn get_bone_limit(&self) -> u32 {
        match self {
            AnimationQuality::Ultra => 256,
            AnimationQuality::High => 128,
            AnimationQuality::Medium => 64,
            AnimationQuality::Low => 32,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterpolationMethod {
    Linear,
    Cubic,
    Bezier,
    Hermite,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlaybackMode {
    Once,
    Loop,
    PingPong,
    Clamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlendMode {
    Override,
    Additive,
    Subtract,
    Multiply,
}

#[derive(Debug, Clone)]
pub struct Transform {
    pub translation: [f32; 3],
    pub rotation: [f32; 4], // Quaternion (x, y, z, w)
    pub scale: [f32; 3],
}

impl Transform {
    pub fn identity() -> Self {
        Self {
            translation: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0], // Identity quaternion
            scale: [1.0, 1.0, 1.0],
        }
    }

    pub fn to_matrix(&self) -> [[f32; 4]; 4] {
        // Convert transform to 4x4 matrix
        let [x, y, z, w] = self.rotation;
        let [sx, sy, sz] = self.scale;
        let [tx, ty, tz] = self.translation;

        // Rotation matrix from quaternion
        let xx = x * x;
        let yy = y * y;
        let zz = z * z;
        let xy = x * y;
        let xz = x * z;
        let yz = y * z;
        let wx = w * x;
        let wy = w * y;
        let wz = w * z;

        [
            [sx * (1.0 - 2.0 * (yy + zz)), sx * 2.0 * (xy - wz), sx * 2.0 * (xz + wy), tx],
            [sy * 2.0 * (xy + wz), sy * (1.0 - 2.0 * (xx + zz)), sy * 2.0 * (yz - wx), ty],
            [sz * 2.0 * (xz - wy), sz * 2.0 * (yz + wx), sz * (1.0 - 2.0 * (xx + yy)), tz],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }
}

#[derive(Debug, Clone)]
pub struct Keyframe {
    pub time: f32,
    pub value: KeyframeValue,
    pub tangent_in: Option<[f32; 3]>,
    pub tangent_out: Option<[f32; 3]>,
}

#[derive(Debug, Clone)]
pub enum KeyframeValue {
    Translation([f32; 3]),
    Rotation([f32; 4]), // Quaternion
    Scale([f32; 3]),
    MorphWeight(f32),
    Float(f32),
    Color([f32; 4]),
    Transform(Transform),
}

#[derive(Debug, Clone)]
pub struct AnimationTrack {
    pub target_property: String, // "translation", "rotation", "scale", etc.
    pub keyframes: Vec<Keyframe>,
    pub interpolation: InterpolationMethod,
    pub pre_behavior: PlaybackMode,
    pub post_behavior: PlaybackMode,
}

#[derive(Debug, Clone)]
pub struct AnimationClip {
    pub name: String,
    pub duration: f32,
    pub tracks: Vec<AnimationTrack>,
    pub loop_start: f32,
    pub loop_end: f32,
    pub events: Vec<AnimationEvent>,
}

#[derive(Debug, Clone)]
pub struct AnimationEvent {
    pub name: String,
    pub time: f32,
    pub parameters: HashMap<String, EventParameter>,
}

#[derive(Debug, Clone)]
pub enum EventParameter {
    String(String),
    Float(f32),
    Int(i32),
    Bool(bool),
}

#[derive(Debug)]
pub struct AnimationState {
    pub clip: AnimationClip,
    pub time: f32,
    pub speed: f32,
    pub weight: f32,
    pub playback_mode: PlaybackMode,
    pub blend_mode: BlendMode,
    pub enabled: bool,
    pub priority: i32,
    pub fade_in_time: f32,
    pub fade_out_time: f32,
    pub current_fade: f32,
    pub fade_direction: FadeDirection,
    pub last_event_index: usize,
}

#[derive(Debug, Clone)]
pub enum FadeDirection {
    None,
    In,
    Out,
}

#[derive(Debug)]
pub struct Bone {
    pub name: String,
    pub parent_index: Option<usize>,
    pub rest_transform: Transform,
    pub inverse_bind_matrix: [[f32; 4]; 4],
}

#[derive(Debug)]
pub struct Skeleton {
    pub name: String,
    pub bones: Vec<Bone>,
    pub root_bone: usize,
}

#[derive(Debug)]
pub struct SkinnedMesh {
    pub skeleton_id: String,
    pub bone_weights: Vec<[f32; 4]>,    // Up to 4 weights per vertex
    pub bone_indices: Vec<[u32; 4]>,    // Up to 4 bone indices per vertex
}

#[derive(Debug)]
pub struct MorphTarget {
    pub name: String,
    pub vertex_deltas: Vec<[f32; 3]>,   // Position deltas
    pub normal_deltas: Option<Vec<[f32; 3]>>, // Normal deltas
    pub weight: f32,
}

#[derive(Debug)]
pub struct AnimatedObject {
    pub name: String,
    pub transform: Transform,
    pub skeleton: Option<SkinnedMesh>,
    pub morph_targets: Vec<MorphTarget>,
    pub animation_states: Vec<AnimationState>,
    pub bone_matrices: Vec<[[f32; 4]; 4]>, // Final bone transformation matrices
    pub enabled: bool,
    pub update_frequency: f32,
    pub last_update: Instant,
}

#[derive(Debug)]
pub struct ProceduralAnimation {
    pub name: String,
    pub animation_type: ProceduralType,
    pub parameters: HashMap<String, f32>,
    pub target_property: String,
    pub amplitude: f32,
    pub frequency: f32,
    pub phase: f32,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub enum ProceduralType {
    Sine,
    Cosine,
    Noise,
    Bounce,
    Spring,
    Breathing,
    Floating,
    Rotation,
    Custom(String),
}

#[derive(Debug, Default)]
pub struct AnimationStats {
    pub active_animations: u32,
    pub active_objects: u32,
    pub bones_processed: u32,
    pub morph_targets_processed: u32,
    pub animation_events_fired: u32,
    pub update_time_ms: f32,
    pub interpolation_calls: u32,
    pub memory_usage_mb: f32,
}

#[derive(Debug)]
pub struct AnimationSystem {
    config: AnimationSystemConfig,
    animated_objects: HashMap<String, AnimatedObject>,
    skeletons: HashMap<String, Skeleton>,
    animation_clips: HashMap<String, AnimationClip>,
    procedural_animations: HashMap<String, ProceduralAnimation>,
    animation_presets: HashMap<String, AnimationClip>,
    event_callbacks: HashMap<String, Box<dyn Fn(&AnimationEvent) + Send + Sync>>,
    stats: AnimationStats,
    global_time_scale: f32,
}

impl AnimationSystem {
    pub fn new(config: AnimationSystemConfig) -> RobinResult<Self> {
        let mut system = Self {
            config,
            animated_objects: HashMap::new(),
            skeletons: HashMap::new(),
            animation_clips: HashMap::new(),
            procedural_animations: HashMap::new(),
            animation_presets: HashMap::new(),
            event_callbacks: HashMap::new(),
            stats: AnimationStats::default(),
            global_time_scale: 1.0,
        };

        system.initialize_presets()?;
        Ok(system)
    }

    pub fn create_animated_object(&mut self, name: String, skeleton_id: Option<String>) -> RobinResult<()> {
        if self.animated_objects.len() >= self.config.max_animated_objects as usize {
            return Err(crate::engine::error::RobinError::new(
                "Maximum number of animated objects reached".to_string()
            ));
        }

        let skeleton = if let Some(skeleton_id) = skeleton_id {
            Some(SkinnedMesh {
                skeleton_id: skeleton_id.clone(),
                bone_weights: Vec::new(),
                bone_indices: Vec::new(),
            })
        } else {
            None
        };

        let bone_count = skeleton.as_ref()
            .and_then(|s| self.skeletons.get(&s.skeleton_id))
            .map(|s| s.bones.len())
            .unwrap_or(0);

        let object = AnimatedObject {
            name: name.clone(),
            transform: Transform::identity(),
            skeleton,
            morph_targets: Vec::new(),
            animation_states: Vec::new(),
            bone_matrices: vec![Transform::identity().to_matrix(); bone_count],
            enabled: true,
            update_frequency: self.config.animation_quality.get_update_frequency(),
            last_update: Instant::now(),
        };

        self.animated_objects.insert(name, object);
        Ok(())
    }

    pub fn create_skeleton(&mut self, name: String, bones: Vec<(String, Option<usize>, Transform)>) -> RobinResult<()> {
        let mut skeleton_bones = Vec::new();
        
        for (bone_name, parent_index, rest_transform) in bones {
            let bone = Bone {
                name: bone_name,
                parent_index,
                rest_transform: rest_transform.clone(),
                inverse_bind_matrix: self.calculate_inverse_bind_matrix(&rest_transform),
            };
            skeleton_bones.push(bone);
        }

        let skeleton = Skeleton {
            name: name.clone(),
            bones: skeleton_bones,
            root_bone: 0, // Assume first bone is root
        };

        self.skeletons.insert(name, skeleton);
        Ok(())
    }

    pub fn play_animation(&mut self, object_name: &str, clip_name: &str, blend_mode: BlendMode, weight: f32, fade_in_time: f32) -> RobinResult<()> {
        let clip = self.animation_clips.get(clip_name)
            .or_else(|| self.animation_presets.get(clip_name))
            .ok_or_else(|| crate::engine::error::RobinError::new(
                format!("Animation clip '{}' not found", clip_name)
            ))?;

        if let Some(object) = self.animated_objects.get_mut(object_name) {
            let animation_state = AnimationState {
                clip: clip.clone(),
                time: 0.0,
                speed: 1.0,
                weight,
                playback_mode: PlaybackMode::Loop,
                blend_mode,
                enabled: true,
                priority: 0,
                fade_in_time,
                fade_out_time: 0.0,
                current_fade: if fade_in_time > 0.0 { 0.0 } else { 1.0 },
                fade_direction: if fade_in_time > 0.0 { FadeDirection::In } else { FadeDirection::None },
                last_event_index: 0,
            };

            object.animation_states.push(animation_state);
        }

        Ok(())
    }

    pub fn stop_animation(&mut self, object_name: &str, clip_name: &str, fade_out_time: f32) -> RobinResult<()> {
        if let Some(object) = self.animated_objects.get_mut(object_name) {
            for state in &mut object.animation_states {
                if state.clip.name == clip_name && state.enabled {
                    if fade_out_time > 0.0 {
                        state.fade_out_time = fade_out_time;
                        state.fade_direction = FadeDirection::Out;
                    } else {
                        state.enabled = false;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn update(&mut self, delta_time: f32) -> RobinResult<()> {
        let update_start = Instant::now();
        let scaled_delta = delta_time * self.global_time_scale;

        self.stats.active_animations = 0;
        self.stats.bones_processed = 0;
        self.stats.morph_targets_processed = 0;
        self.stats.animation_events_fired = 0;
        self.stats.interpolation_calls = 0;

        // Update all animated objects
        let object_names: Vec<String> = self.animated_objects.keys().cloned().collect();
        for object_name in object_names {
            self.update_animated_object(&object_name, scaled_delta)?;
        }

        // Update procedural animations
        self.update_procedural_animations(scaled_delta)?;

        // Update statistics
        self.stats.active_objects = self.animated_objects.len() as u32;
        self.stats.update_time_ms = update_start.elapsed().as_secs_f32() * 1000.0;

        Ok(())
    }

    fn update_animated_object(&mut self, object_name: &str, delta_time: f32) -> RobinResult<()> {
        let object = match self.animated_objects.get_mut(object_name) {
            Some(obj) => obj,
            None => return Ok(()),
        };

        if !object.enabled {
            return Ok(());
        }

        // Check if we need to update based on quality settings
        let time_since_update = object.last_update.elapsed().as_secs_f32();
        let update_interval = 1.0 / object.update_frequency;
        
        if time_since_update < update_interval {
            return Ok(());
        }

        object.last_update = Instant::now();

        // Update animation states
        let mut states_to_remove = Vec::new();
        for (index, state) in object.animation_states.iter_mut().enumerate() {
            if !state.enabled {
                continue;
            }

            // Update time
            state.time += delta_time * state.speed;

            // Handle playback modes
            match state.playback_mode {
                PlaybackMode::Once => {
                    if state.time >= state.clip.duration {
                        state.time = state.clip.duration;
                        state.enabled = false;
                    }
                },
                PlaybackMode::Loop => {
                    if state.time >= state.clip.duration {
                        state.time = state.time % state.clip.duration;
                    }
                },
                PlaybackMode::PingPong => {
                    if state.time >= state.clip.duration {
                        state.speed = -state.speed;
                        state.time = state.clip.duration;
                    } else if state.time < 0.0 {
                        state.speed = -state.speed;
                        state.time = 0.0;
                    }
                },
                PlaybackMode::Clamp => {
                    state.time = state.time.clamp(0.0, state.clip.duration);
                },
            }

            // Update fade
            match state.fade_direction {
                FadeDirection::In => {
                    state.current_fade += delta_time / state.fade_in_time;
                    if state.current_fade >= 1.0 {
                        state.current_fade = 1.0;
                        state.fade_direction = FadeDirection::None;
                    }
                },
                FadeDirection::Out => {
                    state.current_fade -= delta_time / state.fade_out_time;
                    if state.current_fade <= 0.0 {
                        state.current_fade = 0.0;
                        state.enabled = false;
                        states_to_remove.push(index);
                    }
                },
                FadeDirection::None => {},
            }

            // Check for animation events
            self.check_animation_events(state)?;

            self.stats.active_animations += 1;
        }

        // Remove finished animations
        for &index in states_to_remove.iter().rev() {
            object.animation_states.remove(index);
        }

        // Apply animations
        if !object.animation_states.is_empty() {
            self.apply_animations_to_object(object)?;
        }

        Ok(())
    }

    fn apply_animations_to_object(&mut self, object: &mut AnimatedObject) -> RobinResult<()> {
        // Reset bone matrices to bind pose
        if let Some(skeleton_mesh) = &object.skeleton {
            if let Some(skeleton) = self.skeletons.get(&skeleton_mesh.skeleton_id) {
                for (i, bone) in skeleton.bones.iter().enumerate() {
                    object.bone_matrices[i] = bone.rest_transform.to_matrix();
                }
                self.stats.bones_processed += skeleton.bones.len() as u32;
            }
        }

        // Sort animation states by priority
        object.animation_states.sort_by_key(|s| s.priority);

        // Apply each animation state
        for state in &object.animation_states {
            if !state.enabled || state.current_fade <= 0.0 {
                continue;
            }

            let effective_weight = state.weight * state.current_fade;
            self.apply_animation_state(object, state, effective_weight)?;
        }

        // Update morph targets
        for morph_target in &mut object.morph_targets {
            self.stats.morph_targets_processed += 1;
        }

        Ok(())
    }

    fn apply_animation_state(&mut self, object: &mut AnimatedObject, state: &AnimationState, weight: f32) -> RobinResult<()> {
        for track in &state.clip.tracks {
            let current_value = self.sample_track(track, state.time)?;
            self.apply_track_value(object, &track.target_property, current_value, weight, &state.blend_mode)?;
        }
        Ok(())
    }

    fn sample_track(&mut self, track: &AnimationTrack, time: f32) -> RobinResult<KeyframeValue> {
        self.stats.interpolation_calls += 1;

        if track.keyframes.is_empty() {
            return Err(crate::engine::error::RobinError::new("Track has no keyframes".to_string()));
        }

        if track.keyframes.len() == 1 {
            return Ok(track.keyframes[0].value.clone());
        }

        // Find the two keyframes to interpolate between
        let mut prev_keyframe = &track.keyframes[0];
        let mut next_keyframe = &track.keyframes[1];

        for i in 1..track.keyframes.len() {
            if track.keyframes[i].time > time {
                prev_keyframe = &track.keyframes[i - 1];
                next_keyframe = &track.keyframes[i];
                break;
            }
        }

        // Calculate interpolation factor
        let duration = next_keyframe.time - prev_keyframe.time;
        let t = if duration > 0.0 {
            (time - prev_keyframe.time) / duration
        } else {
            0.0
        };

        // Interpolate based on method
        match track.interpolation {
            InterpolationMethod::Linear => {
                self.linear_interpolate(&prev_keyframe.value, &next_keyframe.value, t)
            },
            InterpolationMethod::Cubic => {
                self.cubic_interpolate(&prev_keyframe.value, &next_keyframe.value, t, 
                                     &prev_keyframe.tangent_out, &next_keyframe.tangent_in)
            },
            _ => self.linear_interpolate(&prev_keyframe.value, &next_keyframe.value, t),
        }
    }

    fn linear_interpolate(&self, a: &KeyframeValue, b: &KeyframeValue, t: f32) -> RobinResult<KeyframeValue> {
        match (a, b) {
            (KeyframeValue::Translation(a), KeyframeValue::Translation(b)) => {
                Ok(KeyframeValue::Translation([
                    a[0] + (b[0] - a[0]) * t,
                    a[1] + (b[1] - a[1]) * t,
                    a[2] + (b[2] - a[2]) * t,
                ]))
            },
            (KeyframeValue::Rotation(a), KeyframeValue::Rotation(b)) => {
                // Spherical linear interpolation for quaternions
                Ok(KeyframeValue::Rotation(self.slerp_quaternion(*a, *b, t)))
            },
            (KeyframeValue::Scale(a), KeyframeValue::Scale(b)) => {
                Ok(KeyframeValue::Scale([
                    a[0] + (b[0] - a[0]) * t,
                    a[1] + (b[1] - a[1]) * t,
                    a[2] + (b[2] - a[2]) * t,
                ]))
            },
            (KeyframeValue::Float(a), KeyframeValue::Float(b)) => {
                Ok(KeyframeValue::Float(a + (b - a) * t))
            },
            _ => Err(crate::engine::error::RobinError::new("Mismatched keyframe types".to_string())),
        }
    }

    fn cubic_interpolate(&self, a: &KeyframeValue, b: &KeyframeValue, t: f32, 
                        tangent_out: &Option<[f32; 3]>, tangent_in: &Option<[f32; 3]>) -> RobinResult<KeyframeValue> {
        // For simplicity, fall back to linear interpolation
        // In a full implementation, this would use Hermite or Bézier interpolation
        self.linear_interpolate(a, b, t)
    }

    fn slerp_quaternion(&self, q1: [f32; 4], q2: [f32; 4], t: f32) -> [f32; 4] {
        // Simplified SLERP implementation
        let dot = q1[0] * q2[0] + q1[1] * q2[1] + q1[2] * q2[2] + q1[3] * q2[3];
        
        if dot.abs() > 0.95 {
            // Use linear interpolation for very close quaternions
            let result = [
                q1[0] + (q2[0] - q1[0]) * t,
                q1[1] + (q2[1] - q1[1]) * t,
                q1[2] + (q2[2] - q1[2]) * t,
                q1[3] + (q2[3] - q1[3]) * t,
            ];
            
            // Normalize
            let length = (result[0] * result[0] + result[1] * result[1] + 
                         result[2] * result[2] + result[3] * result[3]).sqrt();
            
            if length > 0.0 {
                [result[0] / length, result[1] / length, result[2] / length, result[3] / length]
            } else {
                [0.0, 0.0, 0.0, 1.0]
            }
        } else {
            // Full SLERP
            let theta = dot.acos();
            let sin_theta = theta.sin();
            
            let w1 = ((1.0 - t) * theta).sin() / sin_theta;
            let w2 = (t * theta).sin() / sin_theta;
            
            [
                q1[0] * w1 + q2[0] * w2,
                q1[1] * w1 + q2[1] * w2,
                q1[2] * w1 + q2[2] * w2,
                q1[3] * w1 + q2[3] * w2,
            ]
        }
    }

    fn apply_track_value(&self, object: &mut AnimatedObject, property: &str, value: KeyframeValue, weight: f32, blend_mode: &BlendMode) -> RobinResult<()> {
        match property {
            "translation" => {
                if let KeyframeValue::Translation(new_translation) = value {
                    match blend_mode {
                        BlendMode::Override => {
                            object.transform.translation = [
                                object.transform.translation[0] * (1.0 - weight) + new_translation[0] * weight,
                                object.transform.translation[1] * (1.0 - weight) + new_translation[1] * weight,
                                object.transform.translation[2] * (1.0 - weight) + new_translation[2] * weight,
                            ];
                        },
                        BlendMode::Additive => {
                            object.transform.translation[0] += new_translation[0] * weight;
                            object.transform.translation[1] += new_translation[1] * weight;
                            object.transform.translation[2] += new_translation[2] * weight;
                        },
                        _ => {}, // Other blend modes would be implemented
                    }
                }
            },
            "rotation" => {
                if let KeyframeValue::Rotation(new_rotation) = value {
                    // Blend quaternions
                    object.transform.rotation = self.slerp_quaternion(object.transform.rotation, new_rotation, weight);
                }
            },
            "scale" => {
                if let KeyframeValue::Scale(new_scale) = value {
                    match blend_mode {
                        BlendMode::Override => {
                            object.transform.scale = [
                                object.transform.scale[0] * (1.0 - weight) + new_scale[0] * weight,
                                object.transform.scale[1] * (1.0 - weight) + new_scale[1] * weight,
                                object.transform.scale[2] * (1.0 - weight) + new_scale[2] * weight,
                            ];
                        },
                        BlendMode::Multiply => {
                            object.transform.scale[0] *= 1.0 + (new_scale[0] - 1.0) * weight;
                            object.transform.scale[1] *= 1.0 + (new_scale[1] - 1.0) * weight;
                            object.transform.scale[2] *= 1.0 + (new_scale[2] - 1.0) * weight;
                        },
                        _ => {},
                    }
                }
            },
            _ => {
                // Handle bone-specific properties or morph targets
                if property.starts_with("bone.") {
                    // Bone animation would be handled here
                } else if property.starts_with("morph.") {
                    // Morph target animation would be handled here
                }
            },
        }
        Ok(())
    }

    fn update_procedural_animations(&mut self, delta_time: f32) -> RobinResult<()> {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f32();

        for procedural in self.procedural_animations.values_mut() {
            if !procedural.enabled {
                continue;
            }

            let phase_time = current_time * procedural.frequency + procedural.phase;
            
            let value = match procedural.animation_type {
                ProceduralType::Sine => (phase_time).sin() * procedural.amplitude,
                ProceduralType::Cosine => (phase_time).cos() * procedural.amplitude,
                ProceduralType::Bounce => {
                    let t = (phase_time % (2.0 * std::f32::consts::PI)) / (2.0 * std::f32::consts::PI);
                    let bounce = if t < 0.5 { 4.0 * t * t } else { 1.0 - 4.0 * (t - 1.0) * (t - 1.0) };
                    bounce * procedural.amplitude
                },
                ProceduralType::Floating => {
                    (phase_time).sin() * 0.5 + (phase_time * 1.3).sin() * 0.3 * procedural.amplitude
                },
                _ => 0.0,
            };

            // Apply the procedural value to the target property
            // This would modify the animated objects based on the procedural animation
        }

        Ok(())
    }

    fn check_animation_events(&mut self, state: &mut AnimationState) -> RobinResult<()> {
        for (i, event) in state.clip.events.iter().enumerate() {
            if i <= state.last_event_index {
                continue;
            }

            if state.time >= event.time {
                self.fire_animation_event(event)?;
                state.last_event_index = i;
                self.stats.animation_events_fired += 1;
            }
        }
        Ok(())
    }

    fn fire_animation_event(&self, event: &AnimationEvent) -> RobinResult<()> {
        if let Some(callback) = self.event_callbacks.get(&event.name) {
            callback(event);
        }
        
        println!("Animation event fired: {} at time {}", event.name, event.time);
        Ok(())
    }

    fn calculate_inverse_bind_matrix(&self, transform: &Transform) -> [[f32; 4]; 4] {
        // Calculate the inverse of the bind pose transformation matrix
        // This is a simplified implementation - would need proper matrix inversion
        let matrix = transform.to_matrix();
        
        // For now, return identity (in a real implementation, this would invert the matrix)
        [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }

    fn initialize_presets(&mut self) -> RobinResult<()> {
        // Idle animation preset
        let idle_clip = AnimationClip {
            name: "Idle".to_string(),
            duration: 4.0,
            tracks: vec![
                AnimationTrack {
                    target_property: "translation".to_string(),
                    keyframes: vec![
                        Keyframe {
                            time: 0.0,
                            value: KeyframeValue::Translation([0.0, 0.0, 0.0]),
                            tangent_in: None,
                            tangent_out: None,
                        },
                        Keyframe {
                            time: 2.0,
                            value: KeyframeValue::Translation([0.0, 0.1, 0.0]),
                            tangent_in: None,
                            tangent_out: None,
                        },
                        Keyframe {
                            time: 4.0,
                            value: KeyframeValue::Translation([0.0, 0.0, 0.0]),
                            tangent_in: None,
                            tangent_out: None,
                        },
                    ],
                    interpolation: InterpolationMethod::Linear,
                    pre_behavior: PlaybackMode::Loop,
                    post_behavior: PlaybackMode::Loop,
                },
            ],
            loop_start: 0.0,
            loop_end: 4.0,
            events: vec![],
        };
        self.animation_presets.insert("Idle".to_string(), idle_clip);

        // Walk animation preset
        let walk_clip = AnimationClip {
            name: "Walk".to_string(),
            duration: 1.0,
            tracks: vec![
                AnimationTrack {
                    target_property: "translation".to_string(),
                    keyframes: vec![
                        Keyframe {
                            time: 0.0,
                            value: KeyframeValue::Translation([0.0, 0.0, 0.0]),
                            tangent_in: None,
                            tangent_out: None,
                        },
                        Keyframe {
                            time: 0.5,
                            value: KeyframeValue::Translation([0.0, 0.05, 1.0]),
                            tangent_in: None,
                            tangent_out: None,
                        },
                        Keyframe {
                            time: 1.0,
                            value: KeyframeValue::Translation([0.0, 0.0, 2.0]),
                            tangent_in: None,
                            tangent_out: None,
                        },
                    ],
                    interpolation: InterpolationMethod::Linear,
                    pre_behavior: PlaybackMode::Loop,
                    post_behavior: PlaybackMode::Loop,
                },
            ],
            loop_start: 0.0,
            loop_end: 1.0,
            events: vec![
                AnimationEvent {
                    name: "Footstep".to_string(),
                    time: 0.25,
                    parameters: HashMap::new(),
                },
                AnimationEvent {
                    name: "Footstep".to_string(),
                    time: 0.75,
                    parameters: HashMap::new(),
                },
            ],
        };
        self.animation_presets.insert("Walk".to_string(), walk_clip);

        // Jump animation preset
        let jump_clip = AnimationClip {
            name: "Jump".to_string(),
            duration: 1.5,
            tracks: vec![
                AnimationTrack {
                    target_property: "translation".to_string(),
                    keyframes: vec![
                        Keyframe {
                            time: 0.0,
                            value: KeyframeValue::Translation([0.0, 0.0, 0.0]),
                            tangent_in: None,
                            tangent_out: None,
                        },
                        Keyframe {
                            time: 0.3,
                            value: KeyframeValue::Translation([0.0, -0.2, 0.0]), // Crouch
                            tangent_in: None,
                            tangent_out: None,
                        },
                        Keyframe {
                            time: 0.7,
                            value: KeyframeValue::Translation([0.0, 2.0, 0.0]), // Peak of jump
                            tangent_in: None,
                            tangent_out: None,
                        },
                        Keyframe {
                            time: 1.5,
                            value: KeyframeValue::Translation([0.0, 0.0, 0.0]), // Landing
                            tangent_in: None,
                            tangent_out: None,
                        },
                    ],
                    interpolation: InterpolationMethod::Cubic,
                    pre_behavior: PlaybackMode::Clamp,
                    post_behavior: PlaybackMode::Clamp,
                },
            ],
            loop_start: 0.0,
            loop_end: 1.5,
            events: vec![
                AnimationEvent {
                    name: "JumpStart".to_string(),
                    time: 0.3,
                    parameters: HashMap::new(),
                },
                AnimationEvent {
                    name: "JumpLand".to_string(),
                    time: 1.5,
                    parameters: HashMap::new(),
                },
            ],
        };
        self.animation_presets.insert("Jump".to_string(), jump_clip);

        Ok(())
    }

    // Public API methods
    pub fn add_morph_target(&mut self, object_name: &str, morph_target: MorphTarget) -> RobinResult<()> {
        if let Some(object) = self.animated_objects.get_mut(object_name) {
            object.morph_targets.push(morph_target);
        }
        Ok(())
    }

    pub fn create_procedural_animation(&mut self, name: String, animation_type: ProceduralType, target_property: String) -> RobinResult<()> {
        let procedural = ProceduralAnimation {
            name: name.clone(),
            animation_type,
            parameters: HashMap::new(),
            target_property,
            amplitude: 1.0,
            frequency: 1.0,
            phase: 0.0,
            enabled: true,
        };

        self.procedural_animations.insert(name, procedural);
        Ok(())
    }

    pub fn register_event_callback<F>(&mut self, event_name: String, callback: F)
    where
        F: Fn(&AnimationEvent) + Send + Sync + 'static,
    {
        self.event_callbacks.insert(event_name, Box::new(callback));
    }

    pub fn set_time_scale(&mut self, scale: f32) {
        self.global_time_scale = scale.max(0.0);
    }

    pub fn get_animation_time(&self, object_name: &str, clip_name: &str) -> Option<f32> {
        self.animated_objects.get(object_name)?
            .animation_states
            .iter()
            .find(|s| s.clip.name == clip_name)
            .map(|s| s.time)
    }

    pub fn get_stats(&self) -> &AnimationStats {
        &self.stats
    }

    pub fn get_object_names(&self) -> Vec<String> {
        self.animated_objects.keys().cloned().collect()
    }

    pub fn get_preset_names(&self) -> Vec<String> {
        self.animation_presets.keys().cloned().collect()
    }

    pub fn get_bone_matrices(&self, object_name: &str) -> Option<&[[[f32; 4]; 4]]> {
        self.animated_objects.get(object_name)
            .map(|obj| obj.bone_matrices.as_slice())
    }

    pub fn shutdown(&mut self) -> RobinResult<()> {
        println!("Animation System shutdown:");
        println!("  Animated objects: {}", self.animated_objects.len());
        println!("  Animation clips: {}", self.animation_clips.len());
        println!("  Skeletons: {}", self.skeletons.len());
        println!("  Update time: {:.2}ms", self.stats.update_time_ms);

        self.animated_objects.clear();
        self.skeletons.clear();
        self.animation_clips.clear();
        self.procedural_animations.clear();
        self.event_callbacks.clear();

        Ok(())
    }
}