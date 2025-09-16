// Advanced Audio Mixer for Robin Engine
// Provides multi-channel mixing, dynamic range compression, equalization, and effects processing

use crate::engine::error::RobinResult;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum AudioChannel {
    Master,
    Music,
    SFX,
    Voice,
    UI,
    Ambient,
    Engine,
    Footsteps,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixerChannel {
    pub volume: f32,
    pub mute: bool,
    pub solo: bool,
    pub pan: f32,
    pub low_pass_freq: f32,
    pub high_pass_freq: f32,
    pub compression: CompressionSettings,
    pub equalizer_bands: Vec<EqualizerBand>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionSettings {
    pub enabled: bool,
    pub threshold: f32,    // dB
    pub ratio: f32,       // X:1 ratio
    pub attack_ms: f32,
    pub release_ms: f32,
    pub makeup_gain: f32, // dB
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EqualizerBand {
    pub frequency: f32,    // Hz
    pub gain: f32,        // dB
    pub q_factor: f32,    // Quality factor
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioFormat {
    PCM16,
    PCM24,
    PCM32,
    Float32,
}

#[derive(Debug, Clone)]
pub struct AudioBus {
    pub name: String,
    pub channels: Vec<AudioChannel>,
    pub volume: f32,
    pub effects: Vec<String>,
}

#[derive(Debug)]
pub struct VolumeControl {
    current_volume: f32,
    target_volume: f32,
    fade_duration: f32,
    fade_timer: f32,
    fade_curve: FadeCurve,
}

#[derive(Debug, Clone)]
pub enum FadeCurve {
    Linear,
    Exponential,
    Logarithmic,
    SCurve,
}

#[derive(Debug, Default)]
pub struct MixerStats {
    pub active_sounds: u32,
    pub peak_level_db: f32,
    pub rms_level_db: f32,
    pub cpu_usage_percent: f32,
    pub buffer_underruns: u32,
    pub clipping_events: u32,
    pub dynamic_range_db: f32,
}

impl Default for MixerChannel {
    fn default() -> Self {
        Self {
            volume: 1.0,
            mute: false,
            solo: false,
            pan: 0.0,
            low_pass_freq: 20000.0,
            high_pass_freq: 20.0,
            compression: CompressionSettings::default(),
            equalizer_bands: Self::create_default_eq_bands(),
        }
    }
}

impl MixerChannel {
    fn create_default_eq_bands() -> Vec<EqualizerBand> {
        vec![
            EqualizerBand { frequency: 60.0, gain: 0.0, q_factor: 0.7, enabled: false },   // Sub bass
            EqualizerBand { frequency: 170.0, gain: 0.0, q_factor: 0.7, enabled: false },  // Bass
            EqualizerBand { frequency: 310.0, gain: 0.0, q_factor: 0.7, enabled: false },  // Low mid
            EqualizerBand { frequency: 600.0, gain: 0.0, q_factor: 0.7, enabled: false },  // Mid
            EqualizerBand { frequency: 1000.0, gain: 0.0, q_factor: 0.7, enabled: false }, // Upper mid
            EqualizerBand { frequency: 3000.0, gain: 0.0, q_factor: 0.7, enabled: false }, // Presence
            EqualizerBand { frequency: 6000.0, gain: 0.0, q_factor: 0.7, enabled: false }, // Brightness
            EqualizerBand { frequency: 12000.0, gain: 0.0, q_factor: 0.7, enabled: false }, // Air
        ]
    }
}

impl Default for CompressionSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            threshold: -12.0,
            ratio: 4.0,
            attack_ms: 10.0,
            release_ms: 100.0,
            makeup_gain: 0.0,
        }
    }
}

impl VolumeControl {
    pub fn new(initial_volume: f32) -> Self {
        Self {
            current_volume: initial_volume,
            target_volume: initial_volume,
            fade_duration: 0.0,
            fade_timer: 0.0,
            fade_curve: FadeCurve::Linear,
        }
    }

    pub fn fade_to(&mut self, target: f32, duration: f32, curve: FadeCurve) {
        self.target_volume = target.clamp(0.0, 1.0);
        self.fade_duration = duration;
        self.fade_timer = 0.0;
        self.fade_curve = curve;
    }

    pub fn update(&mut self, delta_time: f32) {
        if self.fade_timer < self.fade_duration {
            self.fade_timer += delta_time;
            let progress = (self.fade_timer / self.fade_duration).min(1.0);
            
            let curved_progress = match self.fade_curve {
                FadeCurve::Linear => progress,
                FadeCurve::Exponential => progress * progress,
                FadeCurve::Logarithmic => 1.0 - (1.0 - progress) * (1.0 - progress),
                FadeCurve::SCurve => progress * progress * (3.0 - 2.0 * progress),
            };
            
            self.current_volume = self.current_volume * (1.0 - curved_progress) + 
                                self.target_volume * curved_progress;
        }
    }

    pub fn get_volume(&self) -> f32 {
        self.current_volume
    }
}

#[derive(Debug)]
pub struct AudioMixer {
    channels: HashMap<AudioChannel, MixerChannel>,
    volume_controls: HashMap<AudioChannel, VolumeControl>,
    audio_buses: HashMap<String, AudioBus>,
    active_sounds: HashMap<String, ActiveSound>,
    sample_rate: u32,
    buffer_size: u32,
    mixer_stats: MixerStats,
    master_volume: VolumeControl,
    ducking_enabled: bool,
    ducking_threshold: f32,
    limiter_enabled: bool,
    limiter_threshold: f32,
    next_sound_id: u32,
}

#[derive(Debug, Clone)]
struct ActiveSound {
    id: u32,
    sound_id: String,
    channel: AudioChannel,
    volume: f32,
    pan: f32,
    pitch: f32,
    loop_enabled: bool,
    playback_position: f32,
    fade_control: VolumeControl,
}

impl AudioMixer {
    pub fn new(channel_count: u32, sample_rate: u32, buffer_size: u32) -> RobinResult<Self> {
        let mut channels = HashMap::new();
        let mut volume_controls = HashMap::new();

        // Initialize default channels
        let default_channels = [
            AudioChannel::Master,
            AudioChannel::Music,
            AudioChannel::SFX,
            AudioChannel::Voice,
            AudioChannel::UI,
            AudioChannel::Ambient,
            AudioChannel::Engine,
            AudioChannel::Footsteps,
        ];

        for &channel in &default_channels {
            channels.insert(channel, MixerChannel::default());
            volume_controls.insert(channel, VolumeControl::new(1.0));
        }

        Ok(Self {
            channels,
            volume_controls,
            audio_buses: HashMap::new(),
            active_sounds: HashMap::new(),
            sample_rate,
            buffer_size,
            mixer_stats: MixerStats::default(),
            master_volume: VolumeControl::new(1.0),
            ducking_enabled: false,
            ducking_threshold: -20.0, // dB
            limiter_enabled: true,
            limiter_threshold: -0.1, // dB
            next_sound_id: 1,
        })
    }

    pub fn initialize(&mut self) -> RobinResult<()> {
        // Create default audio buses
        self.create_audio_bus("Music".to_string(), vec![AudioChannel::Music])?;
        self.create_audio_bus("GameAudio".to_string(), vec![
            AudioChannel::SFX, 
            AudioChannel::Ambient, 
            AudioChannel::Engine, 
            AudioChannel::Footsteps
        ])?;
        self.create_audio_bus("Dialog".to_string(), vec![AudioChannel::Voice, AudioChannel::UI])?;

        // Set up default compression for music and dialog
        self.set_channel_compression(AudioChannel::Music, CompressionSettings {
            enabled: true,
            threshold: -18.0,
            ratio: 3.0,
            attack_ms: 5.0,
            release_ms: 50.0,
            makeup_gain: 2.0,
        })?;

        self.set_channel_compression(AudioChannel::Voice, CompressionSettings {
            enabled: true,
            threshold: -15.0,
            ratio: 4.0,
            attack_ms: 2.0,
            release_ms: 20.0,
            makeup_gain: 3.0,
        })?;

        println!("Audio Mixer initialized:");
        println!("  Sample rate: {}Hz", self.sample_rate);
        println!("  Buffer size: {} samples", self.buffer_size);
        println!("  Channels: {}", self.channels.len());
        println!("  Audio buses: {}", self.audio_buses.len());

        Ok(())
    }

    pub fn update(&mut self, delta_time: f32) -> RobinResult<()> {
        // Update volume controls
        self.master_volume.update(delta_time);
        for volume_control in self.volume_controls.values_mut() {
            volume_control.update(delta_time);
        }

        // Update active sound fades
        let sound_ids: Vec<String> = self.active_sounds.keys().cloned().collect();
        for sound_id in sound_ids {
            if let Some(sound) = self.active_sounds.get_mut(&sound_id) {
                sound.fade_control.update(delta_time);
            }
        }

        // Update mixer statistics
        self.update_mixer_stats();

        // Apply ducking if enabled
        if self.ducking_enabled {
            self.apply_ducking()?;
        }

        Ok(())
    }

    pub fn play_sound(&mut self, sound_id: &str, volume: f32, pan: f32) -> RobinResult<u32> {
        self.play_sound_on_channel(sound_id, AudioChannel::SFX, volume, pan)
    }

    pub fn play_sound_on_channel(&mut self, sound_id: &str, channel: AudioChannel, volume: f32, pan: f32) -> RobinResult<u32> {
        let id = self.next_sound_id;
        self.next_sound_id += 1;

        let sound = ActiveSound {
            id,
            sound_id: sound_id.to_string(),
            channel,
            volume: volume.clamp(0.0, 1.0),
            pan: pan.clamp(-1.0, 1.0),
            pitch: 1.0,
            loop_enabled: false,
            playback_position: 0.0,
            fade_control: VolumeControl::new(volume),
        };

        self.active_sounds.insert(format!("{}_{}", sound_id, id), sound);
        self.mixer_stats.active_sounds += 1;

        Ok(id)
    }

    pub fn stop_sound(&mut self, sound_key: &str) -> RobinResult<()> {
        if self.active_sounds.remove(sound_key).is_some() {
            self.mixer_stats.active_sounds = self.mixer_stats.active_sounds.saturating_sub(1);
        }
        Ok(())
    }

    pub fn fade_sound(&mut self, sound_key: &str, target_volume: f32, duration: f32, curve: FadeCurve) -> RobinResult<()> {
        if let Some(sound) = self.active_sounds.get_mut(sound_key) {
            sound.fade_control.fade_to(target_volume, duration, curve);
        }
        Ok(())
    }

    pub fn set_master_volume(&mut self, volume: f32) -> RobinResult<()> {
        self.master_volume.fade_to(volume.clamp(0.0, 1.0), 0.1, FadeCurve::Linear);
        Ok(())
    }

    pub fn set_channel_volume(&mut self, channel: AudioChannel, volume: f32) -> RobinResult<()> {
        if let Some(volume_control) = self.volume_controls.get_mut(&channel) {
            volume_control.fade_to(volume.clamp(0.0, 1.0), 0.1, FadeCurve::Linear);
        }
        Ok(())
    }

    pub fn mute_channel(&mut self, channel: AudioChannel) -> RobinResult<()> {
        if let Some(mixer_channel) = self.channels.get_mut(&channel) {
            mixer_channel.mute = true;
        }
        Ok(())
    }

    pub fn unmute_channel(&mut self, channel: AudioChannel) -> RobinResult<()> {
        if let Some(mixer_channel) = self.channels.get_mut(&channel) {
            mixer_channel.mute = false;
        }
        Ok(())
    }

    pub fn solo_channel(&mut self, channel: AudioChannel) -> RobinResult<()> {
        // First, unsolo all channels
        for mixer_channel in self.channels.values_mut() {
            mixer_channel.solo = false;
        }

        // Then solo the requested channel
        if let Some(mixer_channel) = self.channels.get_mut(&channel) {
            mixer_channel.solo = true;
        }
        Ok(())
    }

    pub fn unsolo_all(&mut self) -> RobinResult<()> {
        for mixer_channel in self.channels.values_mut() {
            mixer_channel.solo = false;
        }
        Ok(())
    }

    pub fn mute_all(&mut self) -> RobinResult<()> {
        for mixer_channel in self.channels.values_mut() {
            mixer_channel.mute = true;
        }
        Ok(())
    }

    pub fn unmute_all(&mut self) -> RobinResult<()> {
        for mixer_channel in self.channels.values_mut() {
            mixer_channel.mute = false;
        }
        Ok(())
    }

    pub fn pause_all(&mut self) -> RobinResult<()> {
        // In a real implementation, this would pause all active sounds
        // For now, we'll just fade them down quickly
        let active_sound_keys: Vec<String> = self.active_sounds.keys().cloned().collect();
        for sound_key in active_sound_keys {
            self.fade_sound(&sound_key, 0.0, 0.05, FadeCurve::Exponential)?;
        }
        Ok(())
    }

    pub fn resume_all(&mut self) -> RobinResult<()> {
        // Resume all sounds by fading them back up
        let active_sound_keys: Vec<String> = self.active_sounds.keys().cloned().collect();
        for sound_key in active_sound_keys {
            if let Some(sound) = self.active_sounds.get(&sound_key) {
                let original_volume = sound.volume;
                self.fade_sound(&sound_key, original_volume, 0.05, FadeCurve::Exponential)?;
            }
        }
        Ok(())
    }

    pub fn set_channel_compression(&mut self, channel: AudioChannel, settings: CompressionSettings) -> RobinResult<()> {
        if let Some(mixer_channel) = self.channels.get_mut(&channel) {
            mixer_channel.compression = settings;
        }
        Ok(())
    }

    pub fn set_channel_eq_band(&mut self, channel: AudioChannel, band_index: usize, band: EqualizerBand) -> RobinResult<()> {
        if let Some(mixer_channel) = self.channels.get_mut(&channel) {
            if band_index < mixer_channel.equalizer_bands.len() {
                mixer_channel.equalizer_bands[band_index] = band;
            }
        }
        Ok(())
    }

    pub fn set_channel_pan(&mut self, channel: AudioChannel, pan: f32) -> RobinResult<()> {
        if let Some(mixer_channel) = self.channels.get_mut(&channel) {
            mixer_channel.pan = pan.clamp(-1.0, 1.0);
        }
        Ok(())
    }

    pub fn create_audio_bus(&mut self, name: String, channels: Vec<AudioChannel>) -> RobinResult<()> {
        let bus = AudioBus {
            name: name.clone(),
            channels,
            volume: 1.0,
            effects: Vec::new(),
        };
        self.audio_buses.insert(name, bus);
        Ok(())
    }

    pub fn set_bus_volume(&mut self, bus_name: &str, volume: f32) -> RobinResult<()> {
        if let Some(bus) = self.audio_buses.get_mut(bus_name) {
            bus.volume = volume.clamp(0.0, 1.0);
        }
        Ok(())
    }

    pub fn enable_ducking(&mut self, threshold_db: f32) -> RobinResult<()> {
        self.ducking_enabled = true;
        self.ducking_threshold = threshold_db;
        Ok(())
    }

    pub fn disable_ducking(&mut self) -> RobinResult<()> {
        self.ducking_enabled = false;
        Ok(())
    }

    pub fn set_limiter(&mut self, enabled: bool, threshold_db: f32) -> RobinResult<()> {
        self.limiter_enabled = enabled;
        self.limiter_threshold = threshold_db;
        Ok(())
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) -> RobinResult<()> {
        self.sample_rate = sample_rate;
        // In a real implementation, this would reinitialize audio buffers
        Ok(())
    }

    pub fn get_active_sound_count(&self) -> u32 {
        self.mixer_stats.active_sounds
    }

    pub fn get_mixer_stats(&self) -> &MixerStats {
        &self.mixer_stats
    }

    pub fn get_current_latency(&self) -> f32 {
        // Calculate latency based on buffer size and sample rate
        (self.buffer_size as f32 / self.sample_rate as f32) * 1000.0
    }

    pub fn get_channel_level(&self, channel: AudioChannel) -> f32 {
        // Mock implementation - would return actual channel RMS level
        if let Some(volume_control) = self.volume_controls.get(&channel) {
            volume_control.get_volume()
        } else {
            0.0
        }
    }

    fn update_mixer_stats(&mut self) {
        // Update peak and RMS levels
        let mut peak_level = 0.0;
        let mut total_rms = 0.0;

        for (_, sound) in &self.active_sounds {
            let sound_level = sound.volume * sound.fade_control.get_volume();
            peak_level = peak_level.max(sound_level);
            total_rms += sound_level * sound_level;
        }

        if self.active_sounds.len() > 0 {
            total_rms = (total_rms / self.active_sounds.len() as f32).sqrt();
        }

        // Convert to dB
        self.mixer_stats.peak_level_db = if peak_level > 0.0 {
            20.0 * peak_level.log10()
        } else {
            -60.0
        };

        self.mixer_stats.rms_level_db = if total_rms > 0.0 {
            20.0 * total_rms.log10()
        } else {
            -60.0
        };

        // Calculate dynamic range
        self.mixer_stats.dynamic_range_db = self.mixer_stats.peak_level_db - self.mixer_stats.rms_level_db;

        // Mock CPU usage calculation
        self.mixer_stats.cpu_usage_percent = (self.active_sounds.len() as f32 * 0.5).min(25.0);
    }

    fn apply_ducking(&mut self) -> RobinResult<()> {
        // Simple ducking implementation - reduce music volume when voice is active
        let voice_active = self.active_sounds.values()
            .any(|sound| sound.channel == AudioChannel::Voice && sound.fade_control.get_volume() > 0.1);

        if voice_active {
            self.set_channel_volume(AudioChannel::Music, 0.3)?;
        } else {
            self.set_channel_volume(AudioChannel::Music, 1.0)?;
        }

        Ok(())
    }

    pub fn shutdown(&mut self) -> RobinResult<()> {
        println!("Audio Mixer shutdown:");
        println!("  Sounds processed: {}", self.next_sound_id - 1);
        println!("  Peak level: {:.1} dB", self.mixer_stats.peak_level_db);
        println!("  Average CPU usage: {:.1}%", self.mixer_stats.cpu_usage_percent);
        
        self.active_sounds.clear();
        self.mixer_stats.active_sounds = 0;
        
        Ok(())
    }
}