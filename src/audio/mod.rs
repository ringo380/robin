// Comprehensive Audio and Immersion System for Robin Engine
// Provides 3D positional audio, dynamic mixing, and immersive audio effects

pub mod spatial_audio;
pub mod audio_mixer;
pub mod haptic_feedback;
pub mod ambient_generation;
pub mod sound_propagation;
pub mod adaptive_music;

pub use spatial_audio::{
    SpatialAudioSystem, AudioSource, AudioListener, SpatialAudioConfig,
    DistanceModel, DopplerConfig, AudioSourceType, AudioPlaybackState
};

pub use audio_mixer::{
    AudioMixer, AudioChannel, MixerChannel, AudioBus, VolumeControl,
    CompressionSettings, EqualizerBand, MixerStats, AudioFormat
};

pub use haptic_feedback::{
    HapticFeedbackSystem, HapticDevice, HapticEffect, HapticPattern,
    FeedbackIntensity, HapticTrigger, HapticConfig
};

pub use ambient_generation::{
    AmbientSoundGenerator, EnvironmentAudioProfile, WeatherAudio,
    BiomeAmbience, ProceduralAudio, AmbientLayer, SoundscapeConfig
};

pub use sound_propagation::{
    SoundPropagationSystem, AudioOcclusion, ReverbSettings, ReverbZone,
    MaterialAcoustics, PropagationPath, EchoEffect, SoundRaycast
};

pub use adaptive_music::{
    AdaptiveMusicSystem, MusicTrack, MusicLayer, GameplayMood,
    MusicTransition, DynamicComposition, MusicCue, InteractiveAudio
};

use crate::engine::error::RobinResult;
use std::time::Instant;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSystemConfig {
    pub spatial_audio: SpatialAudioConfig,
    pub mixer_channels: u32,
    pub sample_rate: u32,
    pub buffer_size: u32,
    pub enable_haptic_feedback: bool,
    pub enable_ambient_generation: bool,
    pub enable_sound_propagation: bool,
    pub enable_adaptive_music: bool,
    pub audio_quality: AudioQuality,
    pub max_concurrent_sounds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioQuality {
    Ultra,   // 48kHz, 32-bit, full surround
    High,    // 44.1kHz, 24-bit, stereo/5.1
    Medium,  // 44.1kHz, 16-bit, stereo
    Low,     // 22kHz, 16-bit, mono/stereo
}

impl AudioQuality {
    pub fn get_sample_rate(&self) -> u32 {
        match self {
            AudioQuality::Ultra => 48000,
            AudioQuality::High => 44100,
            AudioQuality::Medium => 44100,
            AudioQuality::Low => 22050,
        }
    }

    pub fn get_bit_depth(&self) -> u16 {
        match self {
            AudioQuality::Ultra => 32,
            AudioQuality::High => 24,
            AudioQuality::Medium => 16,
            AudioQuality::Low => 16,
        }
    }

    pub fn get_channel_count(&self) -> u16 {
        match self {
            AudioQuality::Ultra => 8, // 7.1 surround
            AudioQuality::High => 6,  // 5.1 surround
            AudioQuality::Medium => 2, // Stereo
            AudioQuality::Low => 1,   // Mono
        }
    }
}

impl Default for AudioSystemConfig {
    fn default() -> Self {
        Self {
            spatial_audio: SpatialAudioConfig::default(),
            mixer_channels: 64,
            sample_rate: 44100,
            buffer_size: 1024,
            enable_haptic_feedback: true,
            enable_ambient_generation: true,
            enable_sound_propagation: true,
            enable_adaptive_music: true,
            audio_quality: AudioQuality::High,
            max_concurrent_sounds: 256,
        }
    }
}

#[derive(Debug)]
pub struct AudioSystem {
    config: AudioSystemConfig,
    spatial_audio: SpatialAudioSystem,
    mixer: AudioMixer,
    haptic_system: Option<HapticFeedbackSystem>,
    ambient_generator: Option<AmbientSoundGenerator>,
    sound_propagation: Option<SoundPropagationSystem>,
    adaptive_music: Option<AdaptiveMusicSystem>,
    audio_stats: AudioStats,
    frame_counter: u64,
    last_stats_update: Instant,
}

#[derive(Debug, Default)]
pub struct AudioStats {
    pub active_sources: u32,
    pub cpu_usage_percent: f32,
    pub memory_usage_mb: f32,
    pub buffer_underruns: u32,
    pub latency_ms: f32,
    pub haptic_events: u32,
    pub ambient_layers_active: u32,
    pub music_transitions: u32,
}

impl AudioSystem {
    pub fn new(config: AudioSystemConfig) -> RobinResult<Self> {
        let spatial_audio = SpatialAudioSystem::new(config.spatial_audio.clone())?;
        let mixer = AudioMixer::new(config.mixer_channels, config.sample_rate, config.buffer_size)?;

        let haptic_system = if config.enable_haptic_feedback {
            Some(HapticFeedbackSystem::new()?)
        } else {
            None
        };

        let ambient_generator = if config.enable_ambient_generation {
            Some(AmbientSoundGenerator::new()?)
        } else {
            None
        };

        let sound_propagation = if config.enable_sound_propagation {
            Some(SoundPropagationSystem::new()?)
        } else {
            None
        };

        let adaptive_music = if config.enable_adaptive_music {
            Some(AdaptiveMusicSystem::new()?)
        } else {
            None
        };

        Ok(Self {
            config,
            spatial_audio,
            mixer,
            haptic_system,
            ambient_generator,
            sound_propagation,
            adaptive_music,
            audio_stats: AudioStats::default(),
            frame_counter: 0,
            last_stats_update: Instant::now(),
        })
    }

    pub fn initialize(&mut self) -> RobinResult<()> {
        self.spatial_audio.initialize()?;
        self.mixer.initialize()?;

        if let Some(ref mut haptic) = self.haptic_system {
            haptic.initialize()?;
        }

        if let Some(ref mut ambient) = self.ambient_generator {
            ambient.load_default_soundscapes()?;
        }

        if let Some(ref mut propagation) = self.sound_propagation {
            propagation.initialize()?;
        }

        if let Some(ref mut music) = self.adaptive_music {
            music.load_default_tracks()?;
        }

        println!("Audio System initialized:");
        println!("  Sample Rate: {}Hz", self.config.sample_rate);
        println!("  Quality: {:?}", self.config.audio_quality);
        println!("  Mixer Channels: {}", self.config.mixer_channels);
        println!("  Haptic Feedback: {}", self.config.enable_haptic_feedback);
        println!("  Ambient Generation: {}", self.config.enable_ambient_generation);
        println!("  Sound Propagation: {}", self.config.enable_sound_propagation);
        println!("  Adaptive Music: {}", self.config.enable_adaptive_music);

        Ok(())
    }

    pub fn update(&mut self, delta_time: f32, listener_position: [f32; 3], listener_velocity: [f32; 3]) -> RobinResult<()> {
        self.frame_counter += 1;

        // Update spatial audio system
        self.spatial_audio.update(delta_time, listener_position, listener_velocity)?;

        // Update audio mixer
        self.mixer.update(delta_time)?;

        // Update haptic feedback
        if let Some(ref mut haptic) = self.haptic_system {
            haptic.update(delta_time)?;
        }

        // Update ambient sound generation
        if let Some(ref mut ambient) = self.ambient_generator {
            ambient.update(delta_time, listener_position)?;
        }

        // Update sound propagation
        if let Some(ref mut propagation) = self.sound_propagation {
            propagation.update(delta_time, listener_position)?;
        }

        // Update adaptive music
        if let Some(ref mut music) = self.adaptive_music {
            music.update(delta_time)?;
        }

        // Update statistics
        if self.last_stats_update.elapsed().as_secs_f32() > 1.0 {
            self.update_audio_stats();
            self.last_stats_update = Instant::now();
        }

        Ok(())
    }

    pub fn play_sound_3d(&mut self, sound_id: &str, position: [f32; 3], volume: f32) -> RobinResult<u32> {
        let source_id = self.spatial_audio.create_source(
            sound_id.to_string(),
            position,
            AudioSourceType::OneShot,
            volume
        )?;

        // Apply sound propagation effects if enabled
        if let Some(ref mut propagation) = self.sound_propagation {
            propagation.process_audio_source(source_id, position)?;
        }

        Ok(source_id)
    }

    pub fn play_sound_2d(&mut self, sound_id: &str, volume: f32) -> RobinResult<u32> {
        self.mixer.play_sound(sound_id, volume, 0.0) // 0.0 pan for center
    }

    pub fn set_listener_transform(&mut self, position: [f32; 3], forward: [f32; 3], up: [f32; 3]) -> RobinResult<()> {
        self.spatial_audio.set_listener_transform(position, forward, up)
    }

    pub fn trigger_haptic_feedback(&mut self, pattern: HapticPattern, intensity: f32) -> RobinResult<()> {
        if let Some(ref mut haptic) = self.haptic_system {
            haptic.trigger_feedback(pattern, intensity)?;
        }
        Ok(())
    }

    pub fn set_environment_profile(&mut self, profile: EnvironmentAudioProfile) -> RobinResult<()> {
        if let Some(ref mut ambient) = self.ambient_generator {
            ambient.set_environment_profile(profile)?;
        }
        Ok(())
    }

    pub fn set_weather_audio(&mut self, weather_type: WeatherAudio, intensity: f32) -> RobinResult<()> {
        if let Some(ref mut ambient) = self.ambient_generator {
            ambient.set_weather_audio(weather_type, intensity)?;
        }
        Ok(())
    }

    pub fn add_reverb_zone(&mut self, name: String, center: [f32; 3], radius: f32, reverb: ReverbSettings) -> RobinResult<()> {
        if let Some(ref mut propagation) = self.sound_propagation {
            propagation.add_reverb_zone(name, center, radius, reverb)?;
        }
        Ok(())
    }

    pub fn set_gameplay_mood(&mut self, mood: GameplayMood, intensity: f32) -> RobinResult<()> {
        if let Some(ref mut music) = self.adaptive_music {
            music.set_gameplay_mood(mood, intensity)?;
        }
        Ok(())
    }

    pub fn transition_music(&mut self, track_name: &str, transition_time: f32) -> RobinResult<()> {
        if let Some(ref mut music) = self.adaptive_music {
            music.transition_to_track(track_name, transition_time)?;
        }
        Ok(())
    }

    pub fn set_master_volume(&mut self, volume: f32) -> RobinResult<()> {
        self.mixer.set_master_volume(volume)
    }

    pub fn set_channel_volume(&mut self, channel: AudioChannel, volume: f32) -> RobinResult<()> {
        self.mixer.set_channel_volume(channel, volume)
    }

    pub fn mute_all_audio(&mut self) -> RobinResult<()> {
        self.mixer.mute_all()
    }

    pub fn unmute_all_audio(&mut self) -> RobinResult<()> {
        self.mixer.unmute_all()
    }

    pub fn pause_all_audio(&mut self) -> RobinResult<()> {
        self.spatial_audio.pause_all()?;
        self.mixer.pause_all()?;
        if let Some(ref mut music) = self.adaptive_music {
            music.pause()?;
        }
        Ok(())
    }

    pub fn resume_all_audio(&mut self) -> RobinResult<()> {
        self.spatial_audio.resume_all()?;
        self.mixer.resume_all()?;
        if let Some(ref mut music) = self.adaptive_music {
            music.resume()?;
        }
        Ok(())
    }

    pub fn get_audio_stats(&self) -> &AudioStats {
        &self.audio_stats
    }

    pub fn get_active_source_count(&self) -> u32 {
        self.spatial_audio.get_active_source_count() + self.mixer.get_active_sound_count()
    }

    pub fn optimize_for_quality(&mut self, quality: AudioQuality) -> RobinResult<()> {
        self.config.audio_quality = quality.clone();
        self.config.sample_rate = quality.get_sample_rate();

        // Adjust concurrent sound limits based on quality
        self.config.max_concurrent_sounds = match quality {
            AudioQuality::Ultra => 512,
            AudioQuality::High => 256,
            AudioQuality::Medium => 128,
            AudioQuality::Low => 64,
        };

        // Reconfigure subsystems
        self.spatial_audio.set_quality_settings(quality.clone())?;
        self.mixer.set_sample_rate(self.config.sample_rate)?;

        Ok(())
    }

    fn update_audio_stats(&mut self) {
        self.audio_stats.active_sources = self.get_active_source_count();
        self.audio_stats.cpu_usage_percent = self.calculate_cpu_usage();
        self.audio_stats.memory_usage_mb = self.calculate_memory_usage();
        self.audio_stats.latency_ms = self.mixer.get_current_latency();

        if let Some(ref haptic) = self.haptic_system {
            self.audio_stats.haptic_events = haptic.get_event_count();
        }

        if let Some(ref ambient) = self.ambient_generator {
            self.audio_stats.ambient_layers_active = ambient.get_active_layer_count();
        }

        if let Some(ref music) = self.adaptive_music {
            self.audio_stats.music_transitions = music.get_transition_count();
        }
    }

    fn calculate_cpu_usage(&self) -> f32 {
        // Mock CPU usage calculation
        let base_usage = (self.audio_stats.active_sources as f32 * 0.1).min(15.0);
        let quality_multiplier = match self.config.audio_quality {
            AudioQuality::Ultra => 1.5,
            AudioQuality::High => 1.2,
            AudioQuality::Medium => 1.0,
            AudioQuality::Low => 0.8,
        };
        base_usage * quality_multiplier
    }

    fn calculate_memory_usage(&self) -> f32 {
        // Mock memory usage calculation
        let base_memory = 64.0; // Base system memory
        let source_memory = self.audio_stats.active_sources as f32 * 0.5;
        let quality_multiplier = match self.config.audio_quality {
            AudioQuality::Ultra => 2.0,
            AudioQuality::High => 1.5,
            AudioQuality::Medium => 1.0,
            AudioQuality::Low => 0.7,
        };
        (base_memory + source_memory) * quality_multiplier
    }

    pub fn shutdown(&mut self) -> RobinResult<()> {
        println!("Audio System shutdown:");
        println!("  Total frames processed: {}", self.frame_counter);
        println!("  Peak active sources: {}", self.audio_stats.active_sources);
        println!("  Average CPU usage: {:.1}%", self.audio_stats.cpu_usage_percent);
        println!("  Peak memory usage: {:.1}MB", self.audio_stats.memory_usage_mb);

        self.spatial_audio.shutdown()?;
        self.mixer.shutdown()?;

        if let Some(ref mut haptic) = self.haptic_system {
            haptic.shutdown()?;
        }

        if let Some(ref mut ambient) = self.ambient_generator {
            ambient.shutdown()?;
        }

        if let Some(ref mut propagation) = self.sound_propagation {
            propagation.shutdown()?;
        }

        if let Some(ref mut music) = self.adaptive_music {
            music.shutdown()?;
        }

        Ok(())
    }
}