use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::engine::math::Vec2;
use cgmath::InnerSpace;

/// Represents a loaded sound resource
#[derive(Debug)]
pub struct Sound {
    pub data: Vec<u8>,
    pub name: String,
    pub is_music: bool,
    pub default_volume: f32,
}

/// Configuration for spatial audio
#[derive(Debug)]
pub struct SpatialConfig {
    pub listener_position: Vec2,
    pub listener_direction: Vec2,
    pub max_distance: f32,
    pub rolloff_factor: f32,
    pub reference_distance: f32,
}

impl Default for SpatialConfig {
    fn default() -> Self {
        Self {
            listener_position: Vec2::new(0.0, 0.0),
            listener_direction: Vec2::new(0.0, 1.0),
            max_distance: 1000.0,
            rolloff_factor: 1.0,
            reference_distance: 100.0,
        }
    }
}

/// Main audio manager for the engine
#[derive(Debug)]
pub struct AudioManager {
    sounds: HashMap<String, Arc<Sound>>,
    music_tracks: HashMap<String, Arc<Sound>>,
    master_volume: f32,
    sfx_volume: f32,
    music_volume: f32,
    spatial_config: SpatialConfig,
    next_instance_id: u32,
    muted: bool,
    current_music: Option<u32>,
}

impl AudioManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            sounds: HashMap::new(),
            music_tracks: HashMap::new(),
            master_volume: 1.0,
            sfx_volume: 1.0,
            music_volume: 0.7,
            spatial_config: SpatialConfig::default(),
            next_instance_id: 0,
            muted: false,
            current_music: None,
        })
    }

    pub fn dummy() -> Self {
        Self {
            sounds: HashMap::new(),
            music_tracks: HashMap::new(),
            master_volume: 0.0, // Muted dummy
            sfx_volume: 0.0,
            music_volume: 0.0,
            spatial_config: SpatialConfig::default(),
            next_instance_id: 0,
            muted: true,
            current_music: None,
        }
    }

    /// Load a sound effect from bytes
    pub fn load_sound_from_bytes(&mut self, name: &str, data: Vec<u8>, default_volume: f32) {
        let sound = Arc::new(Sound {
            data,
            name: name.to_string(),
            is_music: false,
            default_volume,
        });
        self.sounds.insert(name.to_string(), sound);
        log::info!("Loaded sound effect: {}", name);
    }

    /// Load a music track from bytes
    pub fn load_music_from_bytes(&mut self, name: &str, data: Vec<u8>) {
        let sound = Arc::new(Sound {
            data,
            name: name.to_string(),
            is_music: true,
            default_volume: 1.0,
        });
        self.music_tracks.insert(name.to_string(), sound);
        log::info!("Loaded music track: {}", name);
    }

    /// Play a sound effect (simplified for now)
    pub fn play_sound(&mut self, name: &str) -> Option<u32> {
        if let Some(sound) = self.sounds.get(name) {
            let id = self.next_instance_id;
            self.next_instance_id += 1;
            
            log::info!("Playing sound '{}' (id: {})", name, id);
            
            // For now, we'll just log the action since actual audio playback
            // would require more complex integration with the audio thread
            Some(id)
        } else {
            log::warn!("Sound '{}' not found", name);
            None
        }
    }

    /// Play a sound effect with custom parameters
    pub fn play_sound_with_params(
        &mut self, 
        name: &str, 
        volume: f32, 
        _looping: bool,
        position: Option<Vec2>
    ) -> Option<u32> {
        if let Some(sound) = self.sounds.get(name) {
            let id = self.next_instance_id;
            self.next_instance_id += 1;
            
            let effective_volume = self.calculate_effective_volume(
                volume * sound.default_volume,
                false,
                position
            );
            
            log::info!("Playing sound '{}' at volume {:.2} (id: {})", name, effective_volume, id);
            
            Some(id)
        } else {
            log::warn!("Sound '{}' not found", name);
            None
        }
    }

    /// Play a sound effect at a specific position (spatial audio)
    pub fn play_sound_at(&mut self, name: &str, position: Vec2, volume: f32) -> Option<u32> {
        self.play_sound_with_params(name, volume, false, Some(position))
    }

    /// Play background music
    pub fn play_music(&mut self, name: &str, _looping: bool) -> Option<u32> {
        if let Some(music) = self.music_tracks.get(name) {
            let id = self.next_instance_id;
            self.next_instance_id += 1;
            
            // Stop current music if playing
            if let Some(current_id) = self.current_music {
                log::info!("Stopping current music (id: {})", current_id);
            }
            
            self.current_music = Some(id);
            
            let effective_volume = self.calculate_effective_volume(1.0, true, None);
            log::info!("Playing music '{}' at volume {:.2} (id: {})", name, effective_volume, id);
            
            Some(id)
        } else {
            log::warn!("Music track '{}' not found", name);
            None
        }
    }

    /// Stop a specific sound instance (for now, just log)
    pub fn stop_sound(&mut self, id: u32) {
        log::info!("Stopping sound (id: {})", id);
        if Some(id) == self.current_music {
            self.current_music = None;
        }
    }

    /// Pause a sound (for now, just log)
    pub fn pause_sound(&mut self, id: u32) {
        log::info!("Pausing sound (id: {})", id);
    }

    /// Resume a paused sound (for now, just log)
    pub fn resume_sound(&mut self, id: u32) {
        log::info!("Resuming sound (id: {})", id);
    }

    /// Set the volume of a specific sound instance
    pub fn set_sound_volume(&mut self, id: u32, volume: f32) {
        log::info!("Setting volume for sound {} to {:.2}", id, volume);
    }

    /// Update the position of a sound for spatial audio
    pub fn update_sound_position(&mut self, id: u32, position: Vec2) {
        log::debug!("Updating sound {} position to ({:.1}, {:.1})", id, position.x, position.y);
    }

    /// Set master volume (affects all sounds)
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
        log::info!("Set master volume to {:.2}", self.master_volume);
    }

    /// Set sound effects volume
    pub fn set_sfx_volume(&mut self, volume: f32) {
        self.sfx_volume = volume.clamp(0.0, 1.0);
        log::info!("Set SFX volume to {:.2}", self.sfx_volume);
    }

    /// Set music volume
    pub fn set_music_volume(&mut self, volume: f32) {
        self.music_volume = volume.clamp(0.0, 1.0);
        log::info!("Set music volume to {:.2}", self.music_volume);
    }

    /// Mute/unmute all audio
    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
        log::info!("Audio {}", if muted { "muted" } else { "unmuted" });
    }

    /// Update listener position for spatial audio
    pub fn set_listener_position(&mut self, position: Vec2) {
        self.spatial_config.listener_position = position;
    }

    /// Update listener direction for spatial audio
    pub fn set_listener_direction(&mut self, direction: Vec2) {
        self.spatial_config.listener_direction = direction;
    }

    /// Stop all sounds
    pub fn stop_all_sounds(&mut self) {
        log::info!("Stopping all sounds");
        self.current_music = None;
    }

    /// Stop all sound effects (keep music playing)
    pub fn stop_all_sfx(&mut self) {
        log::info!("Stopping all sound effects");
    }

    /// Clean up finished sounds (for now, just log)
    pub fn update(&mut self) {
        // In a full implementation, this would clean up finished sounds
    }

    /// Get the number of active sound instances
    pub fn active_sound_count(&self) -> usize {
        0 // For now, return 0
    }

    /// Check if a sound is currently playing
    pub fn is_playing(&self, _id: u32) -> bool {
        false // For now, return false
    }

    /// Check if music is currently playing
    pub fn is_music_playing(&self) -> bool {
        self.current_music.is_some()
    }

    /// Get the current master volume
    pub fn get_master_volume(&self) -> f32 {
        self.master_volume
    }

    // === HELPER METHODS ===

    fn calculate_effective_volume(&self, base_volume: f32, is_music: bool, position: Option<Vec2>) -> f32 {
        if self.muted {
            return 0.0;
        }
        
        let category_volume = if is_music { self.music_volume } else { self.sfx_volume };
        let mut volume = base_volume * category_volume * self.master_volume;
        
        // Apply spatial audio attenuation if position is provided
        if let Some(pos) = position {
            let distance = (pos - self.spatial_config.listener_position).magnitude();
            volume *= self.calculate_distance_attenuation(distance);
        }
        
        volume.clamp(0.0, 1.0)
    }

    fn calculate_distance_attenuation(&self, distance: f32) -> f32 {
        if distance <= self.spatial_config.reference_distance {
            1.0
        } else if distance >= self.spatial_config.max_distance {
            0.0
        } else {
            let ref_dist = self.spatial_config.reference_distance;
            let rolloff = self.spatial_config.rolloff_factor;
            (ref_dist / (ref_dist + rolloff * (distance - ref_dist))).clamp(0.0, 1.0)
        }
    }

    /// Create default sound effects for the engine
    pub fn load_default_sounds(&mut self) {
        // For now, create empty placeholder sounds
        // In a full implementation, these would be generated procedurally or loaded from files
        
        self.load_sound_from_bytes("beep", vec![0; 1000], 0.5);
        self.load_sound_from_bytes("explosion", vec![0; 2000], 0.8);
        self.load_sound_from_bytes("coin", vec![0; 1500], 0.6);
        self.load_sound_from_bytes("jump", vec![0; 800], 0.7);
        
        log::info!("Loaded default sound effects: beep, explosion, coin, jump");
    }
}

// Thread-safe wrapper for the AudioManager
#[derive(Debug)]
pub struct AudioSystem {
    manager: Arc<Mutex<AudioManager>>,
}

impl AudioSystem {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            manager: Arc::new(Mutex::new(AudioManager::new()?)),
        })
    }
    
    pub fn with_manager<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut AudioManager) -> R,
    {
        let mut manager = self.manager.lock().unwrap();
        f(&mut manager)
    }
}