// Immersive Haptic Feedback System for Robin Engine
// Provides tactile feedback for enhanced user interaction and immersion

use crate::engine::error::RobinResult;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HapticConfig {
    pub max_concurrent_effects: u32,
    pub global_intensity_multiplier: f32,
    pub enable_spatial_haptics: bool,
    pub enable_audio_sync: bool,
    pub update_rate: f32,
    pub battery_conservation_mode: bool,
}

impl Default for HapticConfig {
    fn default() -> Self {
        Self {
            max_concurrent_effects: 16,
            global_intensity_multiplier: 1.0,
            enable_spatial_haptics: true,
            enable_audio_sync: true,
            update_rate: 60.0,
            battery_conservation_mode: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HapticDevice {
    Gamepad,
    Mouse,
    Keyboard,
    Phone,
    VRController { hand: VRHand },
    Custom { device_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VRHand {
    Left,
    Right,
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HapticPattern {
    // Basic patterns
    Pulse,
    Heartbeat,
    Click,
    DoubleClick,
    LongPress,
    
    // Game-specific patterns
    WeaponRecoil,
    Footstep,
    Impact,
    Explosion,
    EngineVibration,
    
    // Construction patterns
    Hammer,
    Drill,
    Saw,
    Weld,
    Polish,
    
    // Environmental patterns
    Rain,
    Wind,
    Thunder,
    Earthquake,
    WaterSplash,
    
    // UI patterns
    MenuSelect,
    MenuHover,
    ButtonPress,
    Notification,
    Error,
    Success,
    
    // Communication patterns
    IncomingMessage,
    TypingIndicator,
    CallRinging,
    
    // Custom patterns
    Custom { 
        name: String,
        waveform: Vec<HapticEvent>
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HapticEvent {
    pub time_offset: f32,     // Seconds from pattern start
    pub intensity: f32,       // 0.0 to 1.0
    pub duration: f32,        // Seconds
    pub frequency: f32,       // Hz for compatible devices
    pub sharpness: f32,       // 0.0 to 1.0, texture quality
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeedbackIntensity {
    Subtle,    // 0.1-0.3 intensity
    Light,     // 0.3-0.5 intensity  
    Medium,    // 0.5-0.7 intensity
    Strong,    // 0.7-0.9 intensity
    Maximum,   // 0.9-1.0 intensity
    Custom(f32), // Custom intensity value
}

impl FeedbackIntensity {
    pub fn get_intensity(&self) -> f32 {
        match self {
            FeedbackIntensity::Subtle => 0.2,
            FeedbackIntensity::Light => 0.4,
            FeedbackIntensity::Medium => 0.6,
            FeedbackIntensity::Strong => 0.8,
            FeedbackIntensity::Maximum => 1.0,
            FeedbackIntensity::Custom(value) => value.clamp(0.0, 1.0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HapticTrigger {
    // User interaction triggers
    PlayerMovement,
    PlayerJump,
    PlayerLand,
    PlayerHurt,
    PlayerDeath,
    
    // Construction triggers
    PlaceBlock,
    RemoveBlock,
    UseTool,
    CraftItem,
    
    // Combat triggers
    WeaponFire,
    WeaponHit,
    TakeDamage,
    BlockAttack,
    
    // Environmental triggers
    WeatherChange,
    DayNightTransition,
    SeasonChange,
    
    // UI triggers
    MenuInteraction,
    InventoryAction,
    DialogueAdvance,
    
    // Audio-synced triggers
    MusicBeat,
    SoundEffect(String),
    VoiceLine,
    
    // Custom triggers
    Custom(String),
}

#[derive(Debug)]
struct ActiveHapticEffect {
    id: u32,
    device: HapticDevice,
    pattern: HapticPattern,
    intensity: f32,
    start_time: Instant,
    duration: Duration,
    current_event_index: usize,
    spatial_position: Option<[f32; 3]>,
    fade_in_time: f32,
    fade_out_time: f32,
    loop_count: i32, // -1 for infinite
    priority: u8,
}

#[derive(Debug, Default)]
pub struct HapticStats {
    pub active_effects: u32,
    pub effects_triggered: u64,
    pub battery_level: f32, // For devices that report battery
    pub temperature: f32,   // For thermal management
    pub device_count: u32,
    pub cpu_usage_percent: f32,
}

#[derive(Debug)]
pub struct HapticFeedbackSystem {
    config: HapticConfig,
    devices: HashMap<String, HapticDevice>,
    patterns: HashMap<String, HapticPattern>,
    triggers: HashMap<HapticTrigger, Vec<(HapticPattern, FeedbackIntensity)>>,
    active_effects: HashMap<u32, ActiveHapticEffect>,
    effect_queue: Vec<QueuedEffect>,
    next_effect_id: u32,
    update_timer: f32,
    stats: HapticStats,
    spatial_listener_position: [f32; 3],
}

#[derive(Debug)]
struct QueuedEffect {
    device: HapticDevice,
    pattern: HapticPattern,
    intensity: f32,
    delay: f32,
    priority: u8,
}

impl HapticFeedbackSystem {
    pub fn new() -> RobinResult<Self> {
        let mut system = Self {
            config: HapticConfig::default(),
            devices: HashMap::new(),
            patterns: HashMap::new(),
            triggers: HashMap::new(),
            active_effects: HashMap::new(),
            effect_queue: Vec::new(),
            next_effect_id: 1,
            update_timer: 0.0,
            stats: HapticStats::default(),
            spatial_listener_position: [0.0; 3],
        };

        system.initialize_default_patterns()?;
        system.initialize_default_triggers()?;

        Ok(system)
    }

    pub fn initialize(&mut self) -> RobinResult<()> {
        // Detect and initialize haptic devices
        self.detect_haptic_devices()?;
        
        println!("Haptic Feedback System initialized:");
        println!("  Devices detected: {}", self.devices.len());
        println!("  Patterns loaded: {}", self.patterns.len());
        println!("  Triggers configured: {}", self.triggers.len());
        println!("  Spatial haptics: {}", self.config.enable_spatial_haptics);
        println!("  Audio sync: {}", self.config.enable_audio_sync);
        
        Ok(())
    }

    pub fn update(&mut self, delta_time: f32) -> RobinResult<()> {
        self.update_timer += delta_time;
        
        if self.update_timer >= 1.0 / self.config.update_rate {
            self.process_effect_queue()?;
            self.update_active_effects(delta_time)?;
            self.update_stats();
            self.update_timer = 0.0;
        }

        Ok(())
    }

    pub fn trigger_feedback(&mut self, pattern: HapticPattern, intensity: f32) -> RobinResult<u32> {
        self.trigger_feedback_on_device(HapticDevice::Gamepad, pattern, intensity, 0.0)
    }

    pub fn trigger_feedback_on_device(&mut self, device: HapticDevice, pattern: HapticPattern, intensity: f32, delay: f32) -> RobinResult<u32> {
        let effect_id = self.next_effect_id;
        self.next_effect_id += 1;

        if delay > 0.0 {
            self.effect_queue.push(QueuedEffect {
                device,
                pattern,
                intensity: intensity.clamp(0.0, 1.0) * self.config.global_intensity_multiplier,
                delay,
                priority: 5, // Default priority
            });
        } else {
            self.start_haptic_effect(effect_id, device, pattern, intensity)?;
        }

        self.stats.effects_triggered += 1;
        Ok(effect_id)
    }

    pub fn trigger_spatial_feedback(&mut self, pattern: HapticPattern, intensity: f32, position: [f32; 3]) -> RobinResult<u32> {
        if !self.config.enable_spatial_haptics {
            return self.trigger_feedback(pattern, intensity);
        }

        // Calculate distance-based intensity adjustment
        let distance = Self::distance_vec3(position, self.spatial_listener_position);
        let spatial_intensity = self.calculate_spatial_intensity(intensity, distance);

        let effect_id = self.trigger_feedback(pattern, spatial_intensity)?;

        // Update the effect with spatial position
        if let Some(effect) = self.active_effects.get_mut(&effect_id) {
            effect.spatial_position = Some(position);
        }

        Ok(effect_id)
    }

    pub fn trigger_haptic_trigger(&mut self, trigger: HapticTrigger) -> RobinResult<Vec<u32>> {
        let mut effect_ids = Vec::new();

        if let Some(trigger_effects) = self.triggers.get(&trigger) {
            for (pattern, intensity) in trigger_effects {
                let intensity_value = intensity.get_intensity();
                let effect_id = self.trigger_feedback(pattern.clone(), intensity_value)?;
                effect_ids.push(effect_id);
            }
        }

        Ok(effect_ids)
    }

    pub fn stop_effect(&mut self, effect_id: u32) -> RobinResult<()> {
        self.active_effects.remove(&effect_id);
        Ok(())
    }

    pub fn stop_all_effects(&mut self) -> RobinResult<()> {
        self.active_effects.clear();
        self.effect_queue.clear();
        Ok(())
    }

    pub fn fade_out_effect(&mut self, effect_id: u32, fade_time: f32) -> RobinResult<()> {
        if let Some(effect) = self.active_effects.get_mut(&effect_id) {
            effect.fade_out_time = fade_time;
        }
        Ok(())
    }

    pub fn set_global_intensity(&mut self, intensity: f32) -> RobinResult<()> {
        self.config.global_intensity_multiplier = intensity.clamp(0.0, 2.0);
        Ok(())
    }

    pub fn set_spatial_listener_position(&mut self, position: [f32; 3]) -> RobinResult<()> {
        self.spatial_listener_position = position;
        Ok(())
    }

    pub fn add_device(&mut self, device_name: String, device: HapticDevice) -> RobinResult<()> {
        self.devices.insert(device_name, device);
        self.stats.device_count = self.devices.len() as u32;
        Ok(())
    }

    pub fn remove_device(&mut self, device_name: &str) -> RobinResult<()> {
        self.devices.remove(device_name);
        self.stats.device_count = self.devices.len() as u32;
        Ok(())
    }

    pub fn register_pattern(&mut self, name: String, pattern: HapticPattern) -> RobinResult<()> {
        self.patterns.insert(name, pattern);
        Ok(())
    }

    pub fn register_trigger(&mut self, trigger: HapticTrigger, effects: Vec<(HapticPattern, FeedbackIntensity)>) -> RobinResult<()> {
        self.triggers.insert(trigger, effects);
        Ok(())
    }

    pub fn sync_with_audio(&mut self, audio_event: &str, intensity: f32) -> RobinResult<()> {
        if !self.config.enable_audio_sync {
            return Ok(());
        }

        // Map audio events to haptic patterns
        let pattern = match audio_event {
            "bass_hit" => HapticPattern::Pulse,
            "explosion" => HapticPattern::Explosion,
            "gunshot" => HapticPattern::WeaponRecoil,
            "footstep" => HapticPattern::Footstep,
            "heartbeat" => HapticPattern::Heartbeat,
            _ => return Ok(()),
        };

        self.trigger_feedback(pattern, intensity)?;
        Ok(())
    }

    pub fn set_battery_conservation_mode(&mut self, enabled: bool) -> RobinResult<()> {
        self.config.battery_conservation_mode = enabled;
        
        if enabled {
            self.config.global_intensity_multiplier *= 0.7;
            self.config.max_concurrent_effects = (self.config.max_concurrent_effects / 2).max(4);
        }
        
        Ok(())
    }

    pub fn get_stats(&self) -> &HapticStats {
        &self.stats
    }

    pub fn get_event_count(&self) -> u32 {
        self.stats.effects_triggered as u32
    }

    fn initialize_default_patterns(&mut self) -> RobinResult<()> {
        // Basic patterns
        self.patterns.insert("pulse".to_string(), HapticPattern::Pulse);
        self.patterns.insert("click".to_string(), HapticPattern::Click);
        self.patterns.insert("heartbeat".to_string(), HapticPattern::Heartbeat);
        
        // Game patterns
        self.patterns.insert("weapon_recoil".to_string(), HapticPattern::WeaponRecoil);
        self.patterns.insert("explosion".to_string(), HapticPattern::Explosion);
        self.patterns.insert("footstep".to_string(), HapticPattern::Footstep);
        
        // Construction patterns
        self.patterns.insert("hammer".to_string(), HapticPattern::Hammer);
        self.patterns.insert("drill".to_string(), HapticPattern::Drill);
        self.patterns.insert("saw".to_string(), HapticPattern::Saw);
        
        // Environmental patterns
        self.patterns.insert("rain".to_string(), HapticPattern::Rain);
        self.patterns.insert("thunder".to_string(), HapticPattern::Thunder);
        self.patterns.insert("earthquake".to_string(), HapticPattern::Earthquake);
        
        Ok(())
    }

    fn initialize_default_triggers(&mut self) -> RobinResult<()> {
        // Player movement triggers
        self.triggers.insert(HapticTrigger::PlayerMovement, vec![
            (HapticPattern::Footstep, FeedbackIntensity::Light)
        ]);
        
        self.triggers.insert(HapticTrigger::PlayerJump, vec![
            (HapticPattern::Click, FeedbackIntensity::Medium)
        ]);
        
        self.triggers.insert(HapticTrigger::PlayerLand, vec![
            (HapticPattern::Impact, FeedbackIntensity::Strong)
        ]);
        
        // Construction triggers
        self.triggers.insert(HapticTrigger::PlaceBlock, vec![
            (HapticPattern::Click, FeedbackIntensity::Light)
        ]);
        
        self.triggers.insert(HapticTrigger::UseToolJavaScript, vec![
            (HapticPattern::Hammer, FeedbackIntensity::Medium)
        ]);
        
        // Combat triggers
        self.triggers.insert(HapticTrigger::WeaponFire, vec![
            (HapticPattern::WeaponRecoil, FeedbackIntensity::Strong)
        ]);
        
        // UI triggers
        self.triggers.insert(HapticTrigger::MenuInteraction, vec![
            (HapticPattern::MenuSelect, FeedbackIntensity::Subtle)
        ]);
        
        Ok(())
    }

    fn detect_haptic_devices(&mut self) -> RobinResult<()> {
        // Mock device detection - in a real implementation, this would query the system
        self.devices.insert("primary_gamepad".to_string(), HapticDevice::Gamepad);
        self.devices.insert("mouse".to_string(), HapticDevice::Mouse);
        
        self.stats.device_count = self.devices.len() as u32;
        Ok(())
    }

    fn start_haptic_effect(&mut self, effect_id: u32, device: HapticDevice, pattern: HapticPattern, intensity: f32) -> RobinResult<()> {
        if self.active_effects.len() >= self.config.max_concurrent_effects as usize {
            // Remove lowest priority effect
            if let Some((lowest_id, _)) = self.active_effects.iter()
                .min_by_key(|(_, effect)| effect.priority) {
                let lowest_id = *lowest_id;
                self.active_effects.remove(&lowest_id);
            }
        }

        let duration = self.get_pattern_duration(&pattern);
        
        let effect = ActiveHapticEffect {
            id: effect_id,
            device,
            pattern,
            intensity: intensity * self.config.global_intensity_multiplier,
            start_time: Instant::now(),
            duration,
            current_event_index: 0,
            spatial_position: None,
            fade_in_time: 0.05,
            fade_out_time: 0.0,
            loop_count: 0,
            priority: 5,
        };

        self.active_effects.insert(effect_id, effect);
        self.stats.active_effects += 1;

        Ok(())
    }

    fn process_effect_queue(&mut self) -> RobinResult<()> {
        let mut effects_to_start = Vec::new();
        
        for (index, queued_effect) in self.effect_queue.iter_mut().enumerate() {
            queued_effect.delay -= 1.0 / self.config.update_rate;
            if queued_effect.delay <= 0.0 {
                effects_to_start.push(index);
            }
        }

        // Start queued effects and remove them from queue
        for &index in effects_to_start.iter().rev() {
            let queued_effect = self.effect_queue.remove(index);
            let effect_id = self.next_effect_id;
            self.next_effect_id += 1;
            self.start_haptic_effect(effect_id, queued_effect.device, queued_effect.pattern, queued_effect.intensity)?;
        }

        Ok(())
    }

    fn update_active_effects(&mut self, delta_time: f32) -> RobinResult<()> {
        let mut effects_to_remove = Vec::new();

        for (&effect_id, effect) in &mut self.active_effects {
            let elapsed = effect.start_time.elapsed();
            
            if elapsed >= effect.duration {
                if effect.loop_count > 0 {
                    effect.loop_count -= 1;
                    effect.start_time = Instant::now();
                    effect.current_event_index = 0;
                } else if effect.loop_count == -1 {
                    // Infinite loop - restart
                    effect.start_time = Instant::now();
                    effect.current_event_index = 0;
                } else {
                    effects_to_remove.push(effect_id);
                }
            }

            // Apply spatial positioning updates if enabled
            if self.config.enable_spatial_haptics {
                if let Some(position) = effect.spatial_position {
                    let distance = Self::distance_vec3(position, self.spatial_listener_position);
                    let spatial_intensity = self.calculate_spatial_intensity(effect.intensity, distance);
                    
                    // In a real implementation, this would update the actual device output
                    let _ = spatial_intensity; // Suppress unused variable warning
                }
            }
        }

        for effect_id in effects_to_remove {
            self.active_effects.remove(&effect_id);
            self.stats.active_effects = self.stats.active_effects.saturating_sub(1);
        }

        Ok(())
    }

    fn get_pattern_duration(&self, pattern: &HapticPattern) -> Duration {
        match pattern {
            HapticPattern::Click => Duration::from_millis(50),
            HapticPattern::DoubleClick => Duration::from_millis(150),
            HapticPattern::LongPress => Duration::from_millis(500),
            HapticPattern::Pulse => Duration::from_millis(100),
            HapticPattern::Heartbeat => Duration::from_millis(800),
            HapticPattern::WeaponRecoil => Duration::from_millis(200),
            HapticPattern::Explosion => Duration::from_millis(1000),
            HapticPattern::Footstep => Duration::from_millis(100),
            HapticPattern::Hammer => Duration::from_millis(150),
            HapticPattern::Drill => Duration::from_millis(2000),
            HapticPattern::Thunder => Duration::from_millis(3000),
            HapticPattern::Earthquake => Duration::from_millis(5000),
            _ => Duration::from_millis(250),
        }
    }

    fn calculate_spatial_intensity(&self, base_intensity: f32, distance: f32) -> f32 {
        if distance <= 1.0 {
            return base_intensity;
        }
        
        // Inverse square law for haptic intensity
        let falloff = 1.0 / (distance * distance);
        (base_intensity * falloff).clamp(0.0, 1.0)
    }

    fn distance_vec3(a: [f32; 3], b: [f32; 3]) -> f32 {
        let dx = a[0] - b[0];
        let dy = a[1] - b[1];
        let dz = a[2] - b[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    fn update_stats(&mut self) {
        self.stats.active_effects = self.active_effects.len() as u32;
        self.stats.cpu_usage_percent = (self.stats.active_effects as f32 * 0.2).min(10.0);
        
        // Mock battery and temperature readings
        self.stats.battery_level = 0.85; // 85%
        self.stats.temperature = 35.0 + (self.stats.active_effects as f32 * 0.5); // Mock thermal load
    }

    pub fn shutdown(&mut self) -> RobinResult<()> {
        println!("Haptic Feedback System shutdown:");
        println!("  Total effects triggered: {}", self.stats.effects_triggered);
        println!("  Devices: {}", self.stats.device_count);
        println!("  Peak active effects: {}", self.stats.active_effects);
        
        self.active_effects.clear();
        self.effect_queue.clear();
        self.devices.clear();
        
        Ok(())
    }
}