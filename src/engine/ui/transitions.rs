use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{Instant, Duration};
use nalgebra::{Vector2, Vector3, Vector4};
use crate::engine::error::RobinResult;

/// Advanced UI Transition and Animation System
#[derive(Debug)]
pub struct TransitionSystem {
    pub animator: Animator,
    pub easing_engine: EasingEngine,
    pub timeline_manager: TimelineManager,
    pub interpolator: Interpolator,
    pub effect_processor: EffectProcessor,
    pub transition_queue: TransitionQueue,
    pub parallel_animator: ParallelAnimator,
    config: TransitionConfig,
}

#[derive(Debug, Clone)]
pub struct TransitionConfig {
    pub default_duration: Duration,
    pub default_easing: EasingFunction,
    pub enable_spring_physics: bool,
    pub spring_stiffness: f32,
    pub spring_damping: f32,
    pub enable_motion_blur: bool,
    pub motion_blur_samples: u32,
    pub parallel_animations: bool,
    pub max_concurrent_animations: usize,
}

impl Default for TransitionConfig {
    fn default() -> Self {
        Self {
            default_duration: Duration::from_millis(300),
            default_easing: EasingFunction::EaseInOutCubic,
            enable_spring_physics: true,
            spring_stiffness: 200.0,
            spring_damping: 20.0,
            enable_motion_blur: true,
            motion_blur_samples: 5,
            parallel_animations: true,
            max_concurrent_animations: 10,
        }
    }
}

impl TransitionSystem {
    pub fn new(config: TransitionConfig) -> Self {
        Self {
            animator: Animator::new(&config),
            easing_engine: EasingEngine::new(),
            timeline_manager: TimelineManager::new(),
            interpolator: Interpolator::new(),
            effect_processor: EffectProcessor::new(&config),
            transition_queue: TransitionQueue::new(),
            parallel_animator: ParallelAnimator::new(&config),
            config,
        }
    }

    pub fn update(&mut self, delta: Duration) -> RobinResult<()> {
        // Update all active animations
        self.animator.update(delta)?;

        // Process timeline events
        self.timeline_manager.update(delta)?;

        // Update parallel animations
        if self.config.parallel_animations {
            self.parallel_animator.update(delta)?;
        }

        // Process queued transitions
        self.transition_queue.process(delta)?;

        // Apply post-processing effects
        self.effect_processor.update(delta)?;

        Ok(())
    }

    pub fn fade_in(&mut self, element_id: &str, duration: Duration) -> RobinResult<AnimationHandle> {
        let animation = Animation {
            id: AnimationId::new(),
            element_id: element_id.to_string(),
            animation_type: AnimationType::Fade(FadeAnimation {
                start_opacity: 0.0,
                end_opacity: 1.0,
            }),
            duration,
            elapsed: Duration::ZERO,
            easing: self.config.default_easing,
            state: AnimationState::Pending,
            repeat: RepeatMode::None,
            reverse_on_complete: false,
        };

        self.animator.add_animation(animation)
    }

    pub fn fade_out(&mut self, element_id: &str, duration: Duration) -> RobinResult<AnimationHandle> {
        let animation = Animation {
            id: AnimationId::new(),
            element_id: element_id.to_string(),
            animation_type: AnimationType::Fade(FadeAnimation {
                start_opacity: 1.0,
                end_opacity: 0.0,
            }),
            duration,
            elapsed: Duration::ZERO,
            easing: self.config.default_easing,
            state: AnimationState::Pending,
            repeat: RepeatMode::None,
            reverse_on_complete: false,
        };

        self.animator.add_animation(animation)
    }

    pub fn slide_in(
        &mut self,
        element_id: &str,
        direction: SlideDirection,
        duration: Duration,
    ) -> RobinResult<AnimationHandle> {
        let (start_offset, end_offset) = match direction {
            SlideDirection::Left => (Vector2::new(-100.0, 0.0), Vector2::zeros()),
            SlideDirection::Right => (Vector2::new(100.0, 0.0), Vector2::zeros()),
            SlideDirection::Top => (Vector2::new(0.0, -100.0), Vector2::zeros()),
            SlideDirection::Bottom => (Vector2::new(0.0, 100.0), Vector2::zeros()),
        };

        let animation = Animation {
            id: AnimationId::new(),
            element_id: element_id.to_string(),
            animation_type: AnimationType::Slide(SlideAnimation {
                start_position: start_offset,
                end_position: end_offset,
                relative: true,
            }),
            duration,
            elapsed: Duration::ZERO,
            easing: EasingFunction::EaseOutBack,
            state: AnimationState::Pending,
            repeat: RepeatMode::None,
            reverse_on_complete: false,
        };

        self.animator.add_animation(animation)
    }

    pub fn scale_bounce(
        &mut self,
        element_id: &str,
        scale_factor: f32,
        duration: Duration,
    ) -> RobinResult<AnimationHandle> {
        let animation = Animation {
            id: AnimationId::new(),
            element_id: element_id.to_string(),
            animation_type: AnimationType::Scale(ScaleAnimation {
                start_scale: Vector2::new(1.0, 1.0),
                end_scale: Vector2::new(scale_factor, scale_factor),
                anchor: ScaleAnchor::Center,
            }),
            duration,
            elapsed: Duration::ZERO,
            easing: EasingFunction::EaseOutElastic,
            state: AnimationState::Pending,
            repeat: RepeatMode::None,
            reverse_on_complete: true,
        };

        self.animator.add_animation(animation)
    }

    pub fn rotate(&mut self, element_id: &str, angle: f32, duration: Duration) -> RobinResult<AnimationHandle> {
        let animation = Animation {
            id: AnimationId::new(),
            element_id: element_id.to_string(),
            animation_type: AnimationType::Rotate(RotateAnimation {
                start_angle: 0.0,
                end_angle: angle,
                anchor: RotationAnchor::Center,
            }),
            duration,
            elapsed: Duration::ZERO,
            easing: self.config.default_easing,
            state: AnimationState::Pending,
            repeat: RepeatMode::None,
            reverse_on_complete: false,
        };

        self.animator.add_animation(animation)
    }

    pub fn morph_shape(
        &mut self,
        element_id: &str,
        target_shape: ShapeDescriptor,
        duration: Duration,
    ) -> RobinResult<AnimationHandle> {
        let animation = Animation {
            id: AnimationId::new(),
            element_id: element_id.to_string(),
            animation_type: AnimationType::Morph(MorphAnimation {
                target_shape,
                interpolation_mode: MorphInterpolation::Smooth,
            }),
            duration,
            elapsed: Duration::ZERO,
            easing: EasingFunction::EaseInOutQuad,
            state: AnimationState::Pending,
            repeat: RepeatMode::None,
            reverse_on_complete: false,
        };

        self.animator.add_animation(animation)
    }

    pub fn create_timeline(&mut self, name: &str) -> RobinResult<TimelineHandle> {
        self.timeline_manager.create_timeline(name)
    }

    pub fn add_to_timeline(
        &mut self,
        timeline: TimelineHandle,
        animation: Animation,
        start_time: Duration,
    ) -> RobinResult<()> {
        self.timeline_manager.add_animation(timeline, animation, start_time)
    }

    pub fn play_timeline(&mut self, timeline: TimelineHandle) -> RobinResult<()> {
        self.timeline_manager.play(timeline)
    }

    pub fn create_spring_animation(
        &mut self,
        element_id: &str,
        target_position: Vector2<f32>,
    ) -> RobinResult<AnimationHandle> {
        if !self.config.enable_spring_physics {
            return self.slide_in(element_id, SlideDirection::Left, self.config.default_duration);
        }

        let spring_anim = SpringAnimation {
            element_id: element_id.to_string(),
            current_position: Vector2::zeros(),
            target_position,
            velocity: Vector2::zeros(),
            stiffness: self.config.spring_stiffness,
            damping: self.config.spring_damping,
            threshold: 0.01,
        };

        self.parallel_animator.add_spring_animation(spring_anim)
    }

    pub fn chain_animations(&mut self, animations: Vec<Animation>) -> RobinResult<ChainHandle> {
        self.transition_queue.chain_animations(animations)
    }

    pub fn parallel_animations(&mut self, animations: Vec<Animation>) -> RobinResult<ParallelHandle> {
        self.parallel_animator.run_parallel(animations)
    }

    pub fn apply_blur_transition(
        &mut self,
        element_id: &str,
        blur_amount: f32,
        duration: Duration,
    ) -> RobinResult<AnimationHandle> {
        let animation = Animation {
            id: AnimationId::new(),
            element_id: element_id.to_string(),
            animation_type: AnimationType::Blur(BlurAnimation {
                start_blur: 0.0,
                end_blur: blur_amount,
            }),
            duration,
            elapsed: Duration::ZERO,
            easing: EasingFunction::EaseInOutSine,
            state: AnimationState::Pending,
            repeat: RepeatMode::None,
            reverse_on_complete: false,
        };

        self.animator.add_animation(animation)
    }

    pub fn glow_pulse(
        &mut self,
        element_id: &str,
        glow_intensity: f32,
        duration: Duration,
    ) -> RobinResult<AnimationHandle> {
        let animation = Animation {
            id: AnimationId::new(),
            element_id: element_id.to_string(),
            animation_type: AnimationType::Glow(GlowAnimation {
                start_intensity: 0.0,
                end_intensity: glow_intensity,
                color: Vector3::new(0.4, 0.6, 1.0),
            }),
            duration,
            elapsed: Duration::ZERO,
            easing: EasingFunction::EaseInOutSine,
            state: AnimationState::Pending,
            repeat: RepeatMode::Infinite,
            reverse_on_complete: true,
        };

        self.animator.add_animation(animation)
    }
}

/// Core Animator that manages all active animations
#[derive(Debug)]
pub struct Animator {
    active_animations: HashMap<AnimationId, Animation>,
    completed_animations: Vec<AnimationId>,
    animation_values: HashMap<String, AnimationValue>,
    config: TransitionConfig,
}

impl Animator {
    pub fn new(config: &TransitionConfig) -> Self {
        Self {
            active_animations: HashMap::new(),
            completed_animations: Vec::new(),
            animation_values: HashMap::new(),
            config: config.clone(),
        }
    }

    pub fn update(&mut self, delta: Duration) -> RobinResult<()> {
        self.completed_animations.clear();

        for (id, animation) in &mut self.active_animations {
            animation.elapsed = animation.elapsed.saturating_add(delta);

            if animation.elapsed >= animation.duration {
                match animation.repeat {
                    RepeatMode::None => {
                        animation.state = AnimationState::Completed;
                        self.completed_animations.push(*id);
                    }
                    RepeatMode::Loop(count) => {
                        animation.elapsed = Duration::ZERO;
                        if animation.reverse_on_complete {
                            animation.reverse();
                        }
                    }
                    RepeatMode::Infinite => {
                        animation.elapsed = Duration::ZERO;
                        if animation.reverse_on_complete {
                            animation.reverse();
                        }
                    }
                }
            } else {
                animation.state = AnimationState::Running;
                self.calculate_animation_value(animation)?;
            }
        }

        // Remove completed animations
        for id in &self.completed_animations {
            self.active_animations.remove(id);
        }

        Ok(())
    }

    pub fn add_animation(&mut self, animation: Animation) -> RobinResult<AnimationHandle> {
        let handle = AnimationHandle(animation.id);
        self.active_animations.insert(animation.id, animation);
        Ok(handle)
    }

    fn calculate_animation_value(&mut self, animation: &Animation) -> RobinResult<()> {
        let progress = animation.elapsed.as_secs_f32() / animation.duration.as_secs_f32();
        let eased_progress = EasingEngine::ease(progress, animation.easing);

        let value = match &animation.animation_type {
            AnimationType::Fade(fade) => {
                let opacity = fade.start_opacity + (fade.end_opacity - fade.start_opacity) * eased_progress;
                AnimationValue::Opacity(opacity)
            }
            AnimationType::Slide(slide) => {
                let position = slide.start_position + (slide.end_position - slide.start_position) * eased_progress;
                AnimationValue::Position(position)
            }
            AnimationType::Scale(scale) => {
                let scale_value = scale.start_scale + (scale.end_scale - scale.start_scale) * eased_progress;
                AnimationValue::Scale(scale_value)
            }
            AnimationType::Rotate(rotate) => {
                let angle = rotate.start_angle + (rotate.end_angle - rotate.start_angle) * eased_progress;
                AnimationValue::Rotation(angle)
            }
            AnimationType::Blur(blur) => {
                let blur_amount = blur.start_blur + (blur.end_blur - blur.start_blur) * eased_progress;
                AnimationValue::Blur(blur_amount)
            }
            AnimationType::Glow(glow) => {
                let intensity = glow.start_intensity + (glow.end_intensity - glow.start_intensity) * eased_progress;
                AnimationValue::Glow(intensity, glow.color)
            }
            _ => AnimationValue::None,
        };

        self.animation_values.insert(animation.element_id.clone(), value);
        Ok(())
    }

    pub fn get_animation_value(&self, element_id: &str) -> Option<&AnimationValue> {
        self.animation_values.get(element_id)
    }
}

/// Easing functions for smooth animations
#[derive(Debug)]
pub struct EasingEngine;

impl EasingEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn ease(t: f32, function: EasingFunction) -> f32 {
        let t = t.clamp(0.0, 1.0);

        match function {
            EasingFunction::Linear => t,
            EasingFunction::EaseInQuad => t * t,
            EasingFunction::EaseOutQuad => t * (2.0 - t),
            EasingFunction::EaseInOutQuad => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            }
            EasingFunction::EaseInCubic => t * t * t,
            EasingFunction::EaseOutCubic => {
                let t = t - 1.0;
                t * t * t + 1.0
            }
            EasingFunction::EaseInOutCubic => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    let t = 2.0 * t - 2.0;
                    1.0 + t * t * t / 2.0
                }
            }
            EasingFunction::EaseInQuart => t * t * t * t,
            EasingFunction::EaseOutQuart => {
                let t = t - 1.0;
                1.0 - t * t * t * t
            }
            EasingFunction::EaseInOutQuart => {
                if t < 0.5 {
                    8.0 * t * t * t * t
                } else {
                    let t = t - 1.0;
                    1.0 - 8.0 * t * t * t * t
                }
            }
            EasingFunction::EaseInSine => 1.0 - (t * std::f32::consts::FRAC_PI_2).cos(),
            EasingFunction::EaseOutSine => (t * std::f32::consts::FRAC_PI_2).sin(),
            EasingFunction::EaseInOutSine => -(((std::f32::consts::PI * t).cos() - 1.0) / 2.0),
            EasingFunction::EaseInExpo => {
                if t == 0.0 {
                    0.0
                } else {
                    2.0_f32.powf(10.0 * t - 10.0)
                }
            }
            EasingFunction::EaseOutExpo => {
                if t == 1.0 {
                    1.0
                } else {
                    1.0 - 2.0_f32.powf(-10.0 * t)
                }
            }
            EasingFunction::EaseInOutExpo => {
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else if t < 0.5 {
                    2.0_f32.powf(20.0 * t - 10.0) / 2.0
                } else {
                    (2.0 - 2.0_f32.powf(-20.0 * t + 10.0)) / 2.0
                }
            }
            EasingFunction::EaseOutBack => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
            }
            EasingFunction::EaseInBack => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                c3 * t * t * t - c1 * t * t
            }
            EasingFunction::EaseInOutBack => {
                let c1 = 1.70158;
                let c2 = c1 * 1.525;

                if t < 0.5 {
                    ((2.0 * t).powi(2) * ((c2 + 1.0) * 2.0 * t - c2)) / 2.0
                } else {
                    ((2.0 * t - 2.0).powi(2) * ((c2 + 1.0) * (t * 2.0 - 2.0) + c2) + 2.0) / 2.0
                }
            }
            EasingFunction::EaseOutElastic => {
                if t == 0.0 || t == 1.0 {
                    t
                } else {
                    let c4 = (2.0 * std::f32::consts::PI) / 3.0;
                    2.0_f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * c4).sin() + 1.0
                }
            }
            EasingFunction::EaseOutBounce => {
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
            EasingFunction::Custom(func) => func(t),
        }
    }
}

// Animation structures and types

#[derive(Debug, Clone)]
pub struct Animation {
    pub id: AnimationId,
    pub element_id: String,
    pub animation_type: AnimationType,
    pub duration: Duration,
    pub elapsed: Duration,
    pub easing: EasingFunction,
    pub state: AnimationState,
    pub repeat: RepeatMode,
    pub reverse_on_complete: bool,
}

impl Animation {
    fn reverse(&mut self) {
        match &mut self.animation_type {
            AnimationType::Fade(fade) => {
                std::mem::swap(&mut fade.start_opacity, &mut fade.end_opacity);
            }
            AnimationType::Slide(slide) => {
                std::mem::swap(&mut slide.start_position, &mut slide.end_position);
            }
            AnimationType::Scale(scale) => {
                std::mem::swap(&mut scale.start_scale, &mut scale.end_scale);
            }
            AnimationType::Rotate(rotate) => {
                std::mem::swap(&mut rotate.start_angle, &mut rotate.end_angle);
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AnimationId(u64);

impl AnimationId {
    fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Debug, Clone)]
pub enum AnimationType {
    Fade(FadeAnimation),
    Slide(SlideAnimation),
    Scale(ScaleAnimation),
    Rotate(RotateAnimation),
    Morph(MorphAnimation),
    Blur(BlurAnimation),
    Glow(GlowAnimation),
}

#[derive(Debug, Clone)]
pub struct FadeAnimation {
    pub start_opacity: f32,
    pub end_opacity: f32,
}

#[derive(Debug, Clone)]
pub struct SlideAnimation {
    pub start_position: Vector2<f32>,
    pub end_position: Vector2<f32>,
    pub relative: bool,
}

#[derive(Debug, Clone)]
pub struct ScaleAnimation {
    pub start_scale: Vector2<f32>,
    pub end_scale: Vector2<f32>,
    pub anchor: ScaleAnchor,
}

#[derive(Debug, Clone)]
pub struct RotateAnimation {
    pub start_angle: f32,
    pub end_angle: f32,
    pub anchor: RotationAnchor,
}

#[derive(Debug, Clone)]
pub struct MorphAnimation {
    pub target_shape: ShapeDescriptor,
    pub interpolation_mode: MorphInterpolation,
}

#[derive(Debug, Clone)]
pub struct BlurAnimation {
    pub start_blur: f32,
    pub end_blur: f32,
}

#[derive(Debug, Clone)]
pub struct GlowAnimation {
    pub start_intensity: f32,
    pub end_intensity: f32,
    pub color: Vector3<f32>,
}

#[derive(Debug, Clone)]
pub struct SpringAnimation {
    pub element_id: String,
    pub current_position: Vector2<f32>,
    pub target_position: Vector2<f32>,
    pub velocity: Vector2<f32>,
    pub stiffness: f32,
    pub damping: f32,
    pub threshold: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum EasingFunction {
    Linear,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseInQuart,
    EaseOutQuart,
    EaseInOutQuart,
    EaseInSine,
    EaseOutSine,
    EaseInOutSine,
    EaseInExpo,
    EaseOutExpo,
    EaseInOutExpo,
    EaseInBack,
    EaseOutBack,
    EaseInOutBack,
    EaseOutElastic,
    EaseOutBounce,
    Custom(fn(f32) -> f32),
}

#[derive(Debug, Clone, Copy)]
pub enum AnimationState {
    Pending,
    Running,
    Paused,
    Completed,
}

#[derive(Debug, Clone)]
pub enum RepeatMode {
    None,
    Loop(u32),
    Infinite,
}

#[derive(Debug, Clone, Copy)]
pub enum SlideDirection {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug, Clone, Copy)]
pub enum ScaleAnchor {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

#[derive(Debug, Clone, Copy)]
pub enum RotationAnchor {
    TopLeft,
    Center,
    Custom(Vector2<f32>),
}

#[derive(Debug, Clone)]
pub enum ShapeDescriptor {
    Rectangle { width: f32, height: f32 },
    Circle { radius: f32 },
    RoundedRect { width: f32, height: f32, radius: f32 },
    Polygon { vertices: Vec<Vector2<f32>> },
}

#[derive(Debug, Clone, Copy)]
pub enum MorphInterpolation {
    Linear,
    Smooth,
    Elastic,
}

#[derive(Debug, Clone)]
pub enum AnimationValue {
    None,
    Opacity(f32),
    Position(Vector2<f32>),
    Scale(Vector2<f32>),
    Rotation(f32),
    Blur(f32),
    Glow(f32, Vector3<f32>),
}

// Handles for animation control
#[derive(Debug, Clone, Copy)]
pub struct AnimationHandle(AnimationId);

#[derive(Debug, Clone, Copy)]
pub struct TimelineHandle(u64);

#[derive(Debug, Clone, Copy)]
pub struct ChainHandle(u64);

#[derive(Debug, Clone, Copy)]
pub struct ParallelHandle(u64);

// Supporting components (simplified implementations)
macro_rules! define_transition_component {
    ($name:ident) => {
        #[derive(Debug)]
        pub struct $name;

        impl $name {
            pub fn new() -> Self {
                Self
            }
        }
    };
}

define_transition_component!(TimelineManager);
define_transition_component!(Interpolator);
define_transition_component!(EffectProcessor);
define_transition_component!(TransitionQueue);
define_transition_component!(ParallelAnimator);

// Implement key methods for simplified components
impl TimelineManager {
    pub fn new() -> Self { Self }

    pub fn create_timeline(&mut self, _name: &str) -> RobinResult<TimelineHandle> {
        Ok(TimelineHandle(0))
    }

    pub fn add_animation(
        &mut self,
        _timeline: TimelineHandle,
        _animation: Animation,
        _start_time: Duration,
    ) -> RobinResult<()> {
        Ok(())
    }

    pub fn play(&mut self, _timeline: TimelineHandle) -> RobinResult<()> {
        Ok(())
    }

    pub fn update(&mut self, _delta: Duration) -> RobinResult<()> {
        Ok(())
    }
}

impl TransitionQueue {
    pub fn new() -> Self { Self }

    pub fn chain_animations(&mut self, _animations: Vec<Animation>) -> RobinResult<ChainHandle> {
        Ok(ChainHandle(0))
    }

    pub fn process(&mut self, _delta: Duration) -> RobinResult<()> {
        Ok(())
    }
}

impl ParallelAnimator {
    pub fn new(_config: &TransitionConfig) -> Self { Self }

    pub fn add_spring_animation(&mut self, _animation: SpringAnimation) -> RobinResult<AnimationHandle> {
        Ok(AnimationHandle(AnimationId::new()))
    }

    pub fn run_parallel(&mut self, _animations: Vec<Animation>) -> RobinResult<ParallelHandle> {
        Ok(ParallelHandle(0))
    }

    pub fn update(&mut self, _delta: Duration) -> RobinResult<()> {
        Ok(())
    }
}

impl EffectProcessor {
    pub fn new(_config: &TransitionConfig) -> Self { Self }

    pub fn update(&mut self, _delta: Duration) -> RobinResult<()> {
        Ok(())
    }
}