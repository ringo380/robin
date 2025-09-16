// 3D Spatial Audio System for Robin Engine
// Provides realistic positional audio with distance attenuation, doppler effects, and HRTF

use crate::engine::error::RobinResult;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialAudioConfig {
    pub max_audio_sources: u32,
    pub distance_model: DistanceModel,
    pub doppler_config: DopplerConfig,
    pub enable_hrtf: bool,
    pub enable_environment_occlusion: bool,
    pub audio_update_rate: f32,
    pub max_audible_distance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DistanceModel {
    Linear,
    Inverse,
    Exponential,
    InverseSquare,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DopplerConfig {
    pub enable_doppler: bool,
    pub doppler_factor: f32,
    pub speed_of_sound: f32,
    pub max_doppler_shift: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioSourceType {
    OneShot,
    Looping,
    Streaming,
    UI,
    Voice,
    Music,
    Ambient,
}

#[derive(Debug, Clone)]
pub enum AudioPlaybackState {
    Playing,
    Paused,
    Stopped,
    Fading,
}

impl Default for SpatialAudioConfig {
    fn default() -> Self {
        Self {
            max_audio_sources: 256,
            distance_model: DistanceModel::InverseSquare,
            doppler_config: DopplerConfig {
                enable_doppler: true,
                doppler_factor: 1.0,
                speed_of_sound: 343.0, // m/s
                max_doppler_shift: 2.0,
            },
            enable_hrtf: true,
            enable_environment_occlusion: true,
            audio_update_rate: 60.0,
            max_audible_distance: 1000.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AudioSource {
    pub id: u32,
    pub sound_id: String,
    pub source_type: AudioSourceType,
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub volume: f32,
    pub pitch: f32,
    pub loop_enabled: bool,
    pub playback_state: AudioPlaybackState,
    pub distance_attenuation: f32,
    pub doppler_shift: f32,
    pub occlusion_factor: f32,
    pub last_update: Instant,
    pub fade_target_volume: f32,
    pub fade_duration: f32,
    pub fade_timer: f32,
}

impl AudioSource {
    pub fn new(id: u32, sound_id: String, position: [f32; 3], source_type: AudioSourceType, volume: f32) -> Self {
        Self {
            id,
            sound_id,
            source_type,
            position,
            velocity: [0.0; 3],
            volume,
            pitch: 1.0,
            loop_enabled: false,
            playback_state: AudioPlaybackState::Playing,
            distance_attenuation: 1.0,
            doppler_shift: 1.0,
            occlusion_factor: 1.0,
            last_update: Instant::now(),
            fade_target_volume: volume,
            fade_duration: 0.0,
            fade_timer: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AudioListener {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub forward: [f32; 3],
    pub up: [f32; 3],
    pub right: [f32; 3],
    pub gain: f32,
}

impl Default for AudioListener {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            velocity: [0.0; 3],
            forward: [0.0, 0.0, -1.0],
            up: [0.0, 1.0, 0.0],
            right: [1.0, 0.0, 0.0],
            gain: 1.0,
        }
    }
}

#[derive(Debug)]
pub struct SpatialAudioSystem {
    config: SpatialAudioConfig,
    audio_sources: HashMap<u32, AudioSource>,
    listener: AudioListener,
    next_source_id: u32,
    active_source_count: u32,
    update_timer: f32,
    culled_sources: Vec<u32>,
    priority_queue: Vec<(u32, f32)>, // (source_id, priority_score)
}

impl SpatialAudioSystem {
    pub fn new(config: SpatialAudioConfig) -> RobinResult<Self> {
        Ok(Self {
            config,
            audio_sources: HashMap::new(),
            listener: AudioListener::default(),
            next_source_id: 1,
            active_source_count: 0,
            update_timer: 0.0,
            culled_sources: Vec::new(),
            priority_queue: Vec::new(),
        })
    }

    pub fn initialize(&mut self) -> RobinResult<()> {
        println!("Spatial Audio System initialized:");
        println!("  Max sources: {}", self.config.max_audio_sources);
        println!("  Distance model: {:?}", self.config.distance_model);
        println!("  HRTF enabled: {}", self.config.enable_hrtf);
        println!("  Doppler enabled: {}", self.config.doppler_config.enable_doppler);
        Ok(())
    }

    pub fn update(&mut self, delta_time: f32, listener_position: [f32; 3], listener_velocity: [f32; 3]) -> RobinResult<()> {
        // Update listener
        self.listener.position = listener_position;
        self.listener.velocity = listener_velocity;

        self.update_timer += delta_time;
        if self.update_timer >= 1.0 / self.config.audio_update_rate {
            self.update_spatial_audio(delta_time)?;
            self.cull_distant_sources()?;
            self.manage_source_priority()?;
            self.update_timer = 0.0;
        }

        // Update individual source fading and states
        let source_ids: Vec<u32> = self.audio_sources.keys().cloned().collect();
        for source_id in source_ids {
            self.update_source_fade(source_id, delta_time)?;
        }

        Ok(())
    }

    pub fn create_source(&mut self, sound_id: String, position: [f32; 3], source_type: AudioSourceType, volume: f32) -> RobinResult<u32> {
        if self.audio_sources.len() >= self.config.max_audio_sources as usize {
            // Remove lowest priority source to make room
            if let Some((lowest_priority_id, _)) = self.priority_queue.first().cloned() {
                self.remove_source(lowest_priority_id)?;
            }
        }

        let source_id = self.next_source_id;
        self.next_source_id += 1;

        let source = AudioSource::new(source_id, sound_id, position, source_type, volume);
        self.audio_sources.insert(source_id, source);
        self.active_source_count += 1;

        Ok(source_id)
    }

    pub fn remove_source(&mut self, source_id: u32) -> RobinResult<()> {
        if self.audio_sources.remove(&source_id).is_some() {
            self.active_source_count = self.active_source_count.saturating_sub(1);
        }
        Ok(())
    }

    pub fn set_source_position(&mut self, source_id: u32, position: [f32; 3]) -> RobinResult<()> {
        if let Some(source) = self.audio_sources.get_mut(&source_id) {
            source.position = position;
        }
        Ok(())
    }

    pub fn set_source_velocity(&mut self, source_id: u32, velocity: [f32; 3]) -> RobinResult<()> {
        if let Some(source) = self.audio_sources.get_mut(&source_id) {
            source.velocity = velocity;
        }
        Ok(())
    }

    pub fn set_source_volume(&mut self, source_id: u32, volume: f32) -> RobinResult<()> {
        if let Some(source) = self.audio_sources.get_mut(&source_id) {
            source.volume = volume.clamp(0.0, 1.0);
        }
        Ok(())
    }

    pub fn fade_source(&mut self, source_id: u32, target_volume: f32, duration: f32) -> RobinResult<()> {
        if let Some(source) = self.audio_sources.get_mut(&source_id) {
            source.fade_target_volume = target_volume.clamp(0.0, 1.0);
            source.fade_duration = duration;
            source.fade_timer = 0.0;
            source.playback_state = AudioPlaybackState::Fading;
        }
        Ok(())
    }

    pub fn play_source(&mut self, source_id: u32) -> RobinResult<()> {
        if let Some(source) = self.audio_sources.get_mut(&source_id) {
            source.playback_state = AudioPlaybackState::Playing;
        }
        Ok(())
    }

    pub fn pause_source(&mut self, source_id: u32) -> RobinResult<()> {
        if let Some(source) = self.audio_sources.get_mut(&source_id) {
            source.playback_state = AudioPlaybackState::Paused;
        }
        Ok(())
    }

    pub fn stop_source(&mut self, source_id: u32) -> RobinResult<()> {
        if let Some(source) = self.audio_sources.get_mut(&source_id) {
            source.playback_state = AudioPlaybackState::Stopped;
        }
        Ok(())
    }

    pub fn pause_all(&mut self) -> RobinResult<()> {
        for source in self.audio_sources.values_mut() {
            if matches!(source.playback_state, AudioPlaybackState::Playing) {
                source.playback_state = AudioPlaybackState::Paused;
            }
        }
        Ok(())
    }

    pub fn resume_all(&mut self) -> RobinResult<()> {
        for source in self.audio_sources.values_mut() {
            if matches!(source.playback_state, AudioPlaybackState::Paused) {
                source.playback_state = AudioPlaybackState::Playing;
            }
        }
        Ok(())
    }

    pub fn set_listener_transform(&mut self, position: [f32; 3], forward: [f32; 3], up: [f32; 3]) -> RobinResult<()> {
        self.listener.position = position;
        self.listener.forward = Self::normalize_vec3(forward);
        self.listener.up = Self::normalize_vec3(up);
        self.listener.right = Self::cross_product(self.listener.forward, self.listener.up);
        Ok(())
    }

    pub fn set_listener_velocity(&mut self, velocity: [f32; 3]) -> RobinResult<()> {
        self.listener.velocity = velocity;
        Ok(())
    }

    pub fn get_active_source_count(&self) -> u32 {
        self.active_source_count
    }

    pub fn get_source(&self, source_id: u32) -> Option<&AudioSource> {
        self.audio_sources.get(&source_id)
    }

    pub fn set_quality_settings(&mut self, quality: crate::audio::AudioQuality) -> RobinResult<()> {
        // Adjust spatial audio settings based on quality
        match quality {
            crate::audio::AudioQuality::Ultra => {
                self.config.enable_hrtf = true;
                self.config.enable_environment_occlusion = true;
                self.config.audio_update_rate = 120.0;
                self.config.max_audio_sources = 512;
            },
            crate::audio::AudioQuality::High => {
                self.config.enable_hrtf = true;
                self.config.enable_environment_occlusion = true;
                self.config.audio_update_rate = 60.0;
                self.config.max_audio_sources = 256;
            },
            crate::audio::AudioQuality::Medium => {
                self.config.enable_hrtf = false;
                self.config.enable_environment_occlusion = false;
                self.config.audio_update_rate = 30.0;
                self.config.max_audio_sources = 128;
            },
            crate::audio::AudioQuality::Low => {
                self.config.enable_hrtf = false;
                self.config.enable_environment_occlusion = false;
                self.config.audio_update_rate = 20.0;
                self.config.max_audio_sources = 64;
            },
        }
        Ok(())
    }

    fn update_spatial_audio(&mut self, delta_time: f32) -> RobinResult<()> {
        for source in self.audio_sources.values_mut() {
            // Calculate distance to listener
            let distance = Self::distance_vec3(source.position, self.listener.position);

            // Apply distance attenuation
            source.distance_attenuation = self.calculate_distance_attenuation(distance);

            // Apply doppler effect if enabled
            if self.config.doppler_config.enable_doppler {
                source.doppler_shift = self.calculate_doppler_shift(source, distance);
            }

            // Apply environment occlusion if enabled
            if self.config.enable_environment_occlusion {
                source.occlusion_factor = self.calculate_occlusion_factor(source.position);
            }

            source.last_update = Instant::now();
        }
        Ok(())
    }

    fn calculate_distance_attenuation(&self, distance: f32) -> f32 {
        if distance <= 1.0 {
            return 1.0;
        }

        match self.config.distance_model {
            DistanceModel::Linear => {
                (1.0 - (distance / self.config.max_audible_distance)).max(0.0)
            },
            DistanceModel::Inverse => {
                1.0 / distance
            },
            DistanceModel::Exponential => {
                (-distance / 100.0).exp()
            },
            DistanceModel::InverseSquare => {
                1.0 / (distance * distance)
            },
        }
    }

    fn calculate_doppler_shift(&self, source: &AudioSource, distance: f32) -> f32 {
        if distance < 0.1 {
            return 1.0;
        }

        // Vector from source to listener
        let to_listener = [
            self.listener.position[0] - source.position[0],
            self.listener.position[1] - source.position[1],
            self.listener.position[2] - source.position[2],
        ];
        let to_listener_normalized = Self::normalize_vec3(to_listener);

        // Relative velocity component along the line of sight
        let source_velocity_component = Self::dot_product(source.velocity, to_listener_normalized);
        let listener_velocity_component = Self::dot_product(self.listener.velocity, to_listener_normalized);

        let relative_velocity = listener_velocity_component - source_velocity_component;
        
        // Calculate doppler shift
        let doppler_shift = (self.config.doppler_config.speed_of_sound + relative_velocity) / 
                          self.config.doppler_config.speed_of_sound;

        // Clamp to reasonable limits
        doppler_shift.clamp(
            1.0 / self.config.doppler_config.max_doppler_shift,
            self.config.doppler_config.max_doppler_shift
        )
    }

    fn calculate_occlusion_factor(&self, source_position: [f32; 3]) -> f32 {
        // Simplified occlusion calculation - in a real implementation, this would use raycasting
        // against the world geometry to determine if the source is occluded
        let distance = Self::distance_vec3(source_position, self.listener.position);
        
        // Mock occlusion based on distance and some environmental factors
        if distance > 50.0 {
            0.7 // Partially occluded at long distances
        } else {
            1.0 // No occlusion at close distances
        }
    }

    fn cull_distant_sources(&mut self) -> RobinResult<()> {
        self.culled_sources.clear();
        
        for (&source_id, source) in &self.audio_sources {
            let distance = Self::distance_vec3(source.position, self.listener.position);
            if distance > self.config.max_audible_distance {
                self.culled_sources.push(source_id);
            }
        }

        for &source_id in &self.culled_sources {
            self.remove_source(source_id)?;
        }

        Ok(())
    }

    fn manage_source_priority(&mut self) -> RobinResult<()> {
        self.priority_queue.clear();

        for (&source_id, source) in &self.audio_sources {
            let priority = self.calculate_source_priority(source);
            self.priority_queue.push((source_id, priority));
        }

        // Sort by priority (higher priority first)
        self.priority_queue.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        Ok(())
    }

    fn calculate_source_priority(&self, source: &AudioSource) -> f32 {
        let distance = Self::distance_vec3(source.position, self.listener.position);
        let distance_factor = 1.0 / (1.0 + distance * 0.1);
        
        let type_priority = match source.source_type {
            AudioSourceType::Voice => 1.0,
            AudioSourceType::UI => 0.9,
            AudioSourceType::OneShot => 0.8,
            AudioSourceType::Music => 0.7,
            AudioSourceType::Streaming => 0.6,
            AudioSourceType::Looping => 0.5,
            AudioSourceType::Ambient => 0.4,
        };

        let volume_factor = source.volume;
        
        distance_factor * type_priority * volume_factor
    }

    fn update_source_fade(&mut self, source_id: u32, delta_time: f32) -> RobinResult<()> {
        if let Some(source) = self.audio_sources.get_mut(&source_id) {
            if matches!(source.playback_state, AudioPlaybackState::Fading) {
                source.fade_timer += delta_time;
                
                if source.fade_timer >= source.fade_duration {
                    source.volume = source.fade_target_volume;
                    source.playback_state = AudioPlaybackState::Playing;
                    source.fade_timer = 0.0;
                } else {
                    let progress = source.fade_timer / source.fade_duration;
                    source.volume = source.volume * (1.0 - progress) + source.fade_target_volume * progress;
                }
            }
        }
        Ok(())
    }

    // Helper math functions
    fn distance_vec3(a: [f32; 3], b: [f32; 3]) -> f32 {
        let dx = a[0] - b[0];
        let dy = a[1] - b[1];
        let dz = a[2] - b[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    fn normalize_vec3(v: [f32; 3]) -> [f32; 3] {
        let length = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        if length > 0.0 {
            [v[0] / length, v[1] / length, v[2] / length]
        } else {
            [0.0, 0.0, 0.0]
        }
    }

    fn dot_product(a: [f32; 3], b: [f32; 3]) -> f32 {
        a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
    }

    fn cross_product(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    }

    pub fn shutdown(&mut self) -> RobinResult<()> {
        println!("Spatial Audio System shutdown:");
        println!("  Sources processed: {}", self.next_source_id - 1);
        println!("  Active sources: {}", self.active_source_count);
        
        self.audio_sources.clear();
        self.active_source_count = 0;
        
        Ok(())
    }
}