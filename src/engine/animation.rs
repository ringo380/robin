use crate::engine::math::{Vec2, Vec3};
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug)]
pub enum EaseType {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    EaseInQuart,
    EaseOutQuart,
    EaseInOutQuart,
    EaseInBack,
    EaseOutBack,
    EaseInOutBack,
    Bounce,
    Elastic,
}

impl EaseType {
    pub fn apply(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        
        match self {
            EaseType::Linear => t,
            EaseType::EaseIn => t * t,
            EaseType::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            EaseType::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - 2.0 * (1.0 - t) * (1.0 - t)
                }
            },
            EaseType::EaseInQuart => t * t * t * t,
            EaseType::EaseOutQuart => 1.0 - (1.0 - t).powi(4),
            EaseType::EaseInOutQuart => {
                if t < 0.5 {
                    8.0 * t.powi(4)
                } else {
                    1.0 - 8.0 * (1.0 - t).powi(4)
                }
            },
            EaseType::EaseInBack => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                c3 * t * t * t - c1 * t * t
            },
            EaseType::EaseOutBack => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
            },
            EaseType::EaseInOutBack => {
                let c1 = 1.70158;
                let c2 = c1 * 1.525;
                
                if t < 0.5 {
                    ((2.0 * t).powi(2) * ((c2 + 1.0) * 2.0 * t - c2)) / 2.0
                } else {
                    ((2.0 * t - 2.0).powi(2) * ((c2 + 1.0) * (t * 2.0 - 2.0) + c2) + 2.0) / 2.0
                }
            },
            EaseType::Bounce => {
                if t < 1.0 / 2.75 {
                    7.5625 * t * t
                } else if t < 2.0 / 2.75 {
                    7.5625 * (t - 1.5 / 2.75) * (t - 1.5 / 2.75) + 0.75
                } else if t < 2.5 / 2.75 {
                    7.5625 * (t - 2.25 / 2.75) * (t - 2.25 / 2.75) + 0.9375
                } else {
                    7.5625 * (t - 2.625 / 2.75) * (t - 2.625 / 2.75) + 0.984375
                }
            },
            EaseType::Elastic => {
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    let c4 = (2.0 * std::f32::consts::PI) / 3.0;
                    -(2.0_f32.powf(10.0 * (t - 1.0))) * ((t - 1.0) * c4 - std::f32::consts::PI / 2.0).sin()
                }
            },
        }
    }
}

#[derive(Clone)]
pub enum AnimationTarget {
    Position(Vec2, Vec2),        // (start, end)
    Scale(Vec2, Vec2),           // (start, end)
    Rotation(f32, f32),          // (start, end) in radians
    Color([f32; 4], [f32; 4]),   // (start, end) RGBA
    Alpha(f32, f32),             // (start, end) alpha only
    Size(f32, f32),              // (start, end) uniform scale
}

impl AnimationTarget {
    pub fn interpolate(&self, t: f32) -> AnimationValue {
        match self {
            AnimationTarget::Position(start, end) => {
                AnimationValue::Position(Vec2::new(
                    start.x + (end.x - start.x) * t,
                    start.y + (end.y - start.y) * t,
                ))
            },
            AnimationTarget::Scale(start, end) => {
                AnimationValue::Scale(Vec2::new(
                    start.x + (end.x - start.x) * t,
                    start.y + (end.y - start.y) * t,
                ))
            },
            AnimationTarget::Rotation(start, end) => {
                AnimationValue::Rotation(start + (end - start) * t)
            },
            AnimationTarget::Color(start, end) => {
                AnimationValue::Color([
                    start[0] + (end[0] - start[0]) * t,
                    start[1] + (end[1] - start[1]) * t,
                    start[2] + (end[2] - start[2]) * t,
                    start[3] + (end[3] - start[3]) * t,
                ])
            },
            AnimationTarget::Alpha(start, end) => {
                AnimationValue::Alpha(start + (end - start) * t)
            },
            AnimationTarget::Size(start, end) => {
                AnimationValue::Size(start + (end - start) * t)
            },
        }
    }
}

#[derive(Clone)]
pub enum AnimationValue {
    Position(Vec2),
    Scale(Vec2),
    Rotation(f32),
    Color([f32; 4]),
    Alpha(f32),
    Size(f32),
}

pub struct Animation {
    pub target: AnimationTarget,
    pub duration: Duration,
    pub ease_type: EaseType,
    pub delay: Duration,
    pub repeat: bool,
    pub ping_pong: bool,
    pub start_time: Option<Instant>,
    pub completed: bool,
    forward: bool,
}

impl Animation {
    pub fn new(target: AnimationTarget, duration: Duration, ease_type: EaseType) -> Self {
        Self {
            target,
            duration,
            ease_type,
            delay: Duration::ZERO,
            repeat: false,
            ping_pong: false,
            start_time: None,
            completed: false,
            forward: true,
        }
    }

    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    pub fn with_repeat(mut self, repeat: bool) -> Self {
        self.repeat = repeat;
        self
    }

    pub fn with_ping_pong(mut self, ping_pong: bool) -> Self {
        self.ping_pong = ping_pong;
        self
    }

    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
        self.completed = false;
    }

    pub fn update(&mut self) -> Option<AnimationValue> {
        if self.completed && !self.repeat {
            return None;
        }

        let start_time = self.start_time?;
        let elapsed = start_time.elapsed();

        if elapsed < self.delay {
            return None;
        }

        let animation_elapsed = elapsed - self.delay;
        let mut progress = animation_elapsed.as_secs_f32() / self.duration.as_secs_f32();

        if progress >= 1.0 {
            if self.ping_pong {
                self.forward = !self.forward;
                self.start_time = Some(Instant::now());
                progress = 0.0;
            } else if self.repeat {
                self.start_time = Some(Instant::now());
                progress = 0.0;
            } else {
                progress = 1.0;
                self.completed = true;
            }
        }

        let final_progress = if self.ping_pong && !self.forward {
            1.0 - progress
        } else {
            progress
        };

        let eased_progress = self.ease_type.apply(final_progress);
        Some(self.target.interpolate(eased_progress))
    }

    pub fn is_completed(&self) -> bool {
        self.completed && !self.repeat
    }
}

#[derive(Default)]
pub struct AnimationManager {
    animations: HashMap<String, Vec<Animation>>,
    sequences: HashMap<String, AnimationSequence>,
}

impl AnimationManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_animation(&mut self, entity_id: String, animation: Animation) {
        let animations = self.animations.entry(entity_id).or_insert_with(Vec::new);
        animations.push(animation);
    }

    pub fn remove_animations(&mut self, entity_id: &str) {
        self.animations.remove(entity_id);
    }

    pub fn start_animation(&mut self, entity_id: &str) {
        if let Some(animations) = self.animations.get_mut(entity_id) {
            for animation in animations {
                animation.start();
            }
        }
    }

    pub fn add_sequence(&mut self, name: String, sequence: AnimationSequence) {
        self.sequences.insert(name, sequence);
    }

    pub fn start_sequence(&mut self, name: &str) {
        if let Some(sequence) = self.sequences.get_mut(name) {
            sequence.start();
        }
    }

    pub fn update(&mut self) -> HashMap<String, Vec<AnimationValue>> {
        let mut results = HashMap::new();

        // Update individual animations
        for (entity_id, animations) in &mut self.animations {
            let mut values = Vec::new();
            animations.retain_mut(|animation| {
                if let Some(value) = animation.update() {
                    values.push(value);
                }
                !animation.is_completed()
            });
            
            if !values.is_empty() {
                results.insert(entity_id.clone(), values);
            }
        }

        // Update sequences
        for (name, sequence) in &mut self.sequences {
            if let Some(values) = sequence.update() {
                results.extend(values);
            }
        }

        // Clean up completed animations
        self.animations.retain(|_, animations| !animations.is_empty());
        self.sequences.retain(|_, sequence| !sequence.is_completed());

        results
    }

    // Utility methods for common animations
    pub fn fade_in(&mut self, entity_id: String, duration: Duration) {
        let animation = Animation::new(
            AnimationTarget::Alpha(0.0, 1.0),
            duration,
            EaseType::EaseOut,
        );
        self.add_animation(entity_id, animation);
    }

    pub fn fade_out(&mut self, entity_id: String, duration: Duration) {
        let animation = Animation::new(
            AnimationTarget::Alpha(1.0, 0.0),
            duration,
            EaseType::EaseIn,
        );
        self.add_animation(entity_id, animation);
    }

    pub fn move_to(&mut self, entity_id: String, from: Vec2, to: Vec2, duration: Duration) {
        let animation = Animation::new(
            AnimationTarget::Position(from, to),
            duration,
            EaseType::EaseInOut,
        );
        self.add_animation(entity_id, animation);
    }

    pub fn scale_bounce(&mut self, entity_id: String, from_scale: f32, to_scale: f32, duration: Duration) {
        let animation = Animation::new(
            AnimationTarget::Size(from_scale, to_scale),
            duration,
            EaseType::Bounce,
        );
        self.add_animation(entity_id, animation);
    }

    pub fn rotate_continuous(&mut self, entity_id: String, speed: f32) {
        let animation = Animation::new(
            AnimationTarget::Rotation(0.0, std::f32::consts::PI * 2.0),
            Duration::from_secs_f32(1.0 / speed),
            EaseType::Linear,
        ).with_repeat(true);
        self.add_animation(entity_id, animation);
    }

    pub fn pulse_color(&mut self, entity_id: String, color1: [f32; 4], color2: [f32; 4], duration: Duration) {
        let animation = Animation::new(
            AnimationTarget::Color(color1, color2),
            duration,
            EaseType::EaseInOut,
        ).with_repeat(true).with_ping_pong(true);
        self.add_animation(entity_id, animation);
    }
}

pub struct AnimationSequence {
    steps: Vec<(String, Animation)>, // (entity_id, animation)
    current_step: usize,
    started: bool,
    completed: bool,
}

impl AnimationSequence {
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            current_step: 0,
            started: false,
            completed: false,
        }
    }

    pub fn add_step(&mut self, entity_id: String, animation: Animation) {
        self.steps.push((entity_id, animation));
    }

    pub fn start(&mut self) {
        if !self.steps.is_empty() {
            self.started = true;
            self.current_step = 0;
            self.steps[0].1.start();
        }
    }

    pub fn update(&mut self) -> Option<HashMap<String, Vec<AnimationValue>>> {
        if !self.started || self.completed {
            return None;
        }

        if self.current_step >= self.steps.len() {
            self.completed = true;
            return None;
        }

        let mut results = HashMap::new();
        let (entity_id, animation) = &mut self.steps[self.current_step];

        if let Some(value) = animation.update() {
            results.insert(entity_id.clone(), vec![value]);
        }

        if animation.is_completed() {
            self.current_step += 1;
            if self.current_step < self.steps.len() {
                self.steps[self.current_step].1.start();
            } else {
                self.completed = true;
            }
        }

        if results.is_empty() {
            None
        } else {
            Some(results)
        }
    }

    pub fn is_completed(&self) -> bool {
        self.completed
    }
}