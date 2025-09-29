// Configuration system for Robin Engine Demo
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use crate::logging::{LogCategory, log_info, log_warn, log_error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsConfig {
    pub enable_vsync: bool,
    pub enable_msaa: bool,
    pub msaa_samples: u32,
    pub target_fps: u32,
    pub enable_frustum_culling: bool,
    pub enable_lod: bool,
    pub max_render_distance: f32,
    pub shadow_quality: ShadowQuality,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShadowQuality {
    Off,
    Low,
    Medium,
    High,
    Ultra,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputConfig {
    pub mouse_sensitivity: f32,
    pub invert_mouse_y: bool,
    pub camera_acceleration: f32,
    pub camera_max_speed: f32,
    pub enable_smooth_movement: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub master_volume: f32,
    pub effects_volume: f32,
    pub ambient_volume: f32,
    pub enable_spatial_audio: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub enable_performance_monitoring: bool,
    pub log_frame_times: bool,
    pub max_chunk_generation_per_frame: u32,
    pub vertex_buffer_size: usize,
    pub enable_gpu_profiling: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoConfig {
    pub graphics: GraphicsConfig,
    pub input: InputConfig,
    pub audio: AudioConfig,
    pub performance: PerformanceConfig,
    pub window_width: u32,
    pub window_height: u32,
    pub fullscreen: bool,
    pub show_debug_ui: bool,
    pub enable_wireframe: bool,
}

impl Default for GraphicsConfig {
    fn default() -> Self {
        Self {
            enable_vsync: true,
            enable_msaa: true,
            msaa_samples: 4,
            target_fps: 60,
            enable_frustum_culling: true,
            enable_lod: true,
            max_render_distance: 500.0,
            shadow_quality: ShadowQuality::Medium,
        }
    }
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 0.002,
            invert_mouse_y: false,
            camera_acceleration: 0.8,
            camera_max_speed: 0.5,
            enable_smooth_movement: true,
        }
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: 0.7,
            effects_volume: 0.8,
            ambient_volume: 0.6,
            enable_spatial_audio: true,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enable_performance_monitoring: true,
            log_frame_times: false,
            max_chunk_generation_per_frame: 4,
            vertex_buffer_size: 1024 * 1024, // 1MB
            enable_gpu_profiling: false,
        }
    }
}

impl Default for DemoConfig {
    fn default() -> Self {
        Self {
            graphics: GraphicsConfig::default(),
            input: InputConfig::default(),
            audio: AudioConfig::default(),
            performance: PerformanceConfig::default(),
            window_width: 1280,
            window_height: 720,
            fullscreen: false,
            show_debug_ui: true,
            enable_wireframe: false,
        }
    }
}

impl DemoConfig {
    const CONFIG_FILENAME: &'static str = "robin_demo_config.toml";

    pub fn load() -> Self {
        match Self::load_from_file(Self::CONFIG_FILENAME) {
            Ok(config) => {
                log_info!(LogCategory::Config, "Configuration loaded from {}", Self::CONFIG_FILENAME);
                config
            }
            Err(e) => {
                log_warn!(LogCategory::Config, "Failed to load config: {}. Using defaults.", e);
                let default_config = Self::default();
                if let Err(save_err) = default_config.save() {
                    log_error!(LogCategory::Config, "Failed to save default config: {}", save_err);
                }
                default_config
            }
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_str = toml::to_string_pretty(self)?;
        fs::write(Self::CONFIG_FILENAME, config_str)?;
        log_info!(LogCategory::Config, "Configuration saved to {}", Self::CONFIG_FILENAME);
        Ok(())
    }

    fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = fs::read_to_string(path)?;
        let config: Self = toml::from_str(&config_str)?;

        // Validate loaded config
        config.validate()?;

        Ok(config)
    }

    fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Validate graphics settings
        if self.graphics.msaa_samples > 16 {
            return Err("MSAA samples cannot exceed 16".into());
        }
        if self.graphics.target_fps == 0 || self.graphics.target_fps > 300 {
            return Err("Target FPS must be between 1 and 300".into());
        }
        if self.graphics.max_render_distance <= 0.0 {
            return Err("Max render distance must be positive".into());
        }

        // Validate input settings
        if self.input.mouse_sensitivity <= 0.0 {
            return Err("Mouse sensitivity must be positive".into());
        }
        if self.input.camera_max_speed <= 0.0 {
            return Err("Camera max speed must be positive".into());
        }

        // Validate audio settings
        if !(0.0..=1.0).contains(&self.audio.master_volume) {
            return Err("Master volume must be between 0.0 and 1.0".into());
        }
        if !(0.0..=1.0).contains(&self.audio.effects_volume) {
            return Err("Effects volume must be between 0.0 and 1.0".into());
        }
        if !(0.0..=1.0).contains(&self.audio.ambient_volume) {
            return Err("Ambient volume must be between 0.0 and 1.0".into());
        }

        // Validate window settings
        if self.window_width < 320 || self.window_width > 7680 {
            return Err("Window width must be between 320 and 7680".into());
        }
        if self.window_height < 240 || self.window_height > 4320 {
            return Err("Window height must be between 240 and 4320".into());
        }

        // Validate performance settings
        if self.performance.max_chunk_generation_per_frame == 0 {
            return Err("Max chunk generation per frame must be at least 1".into());
        }
        if self.performance.vertex_buffer_size < 1024 {
            return Err("Vertex buffer size must be at least 1024 bytes".into());
        }

        Ok(())
    }

    pub fn get_mouse_sensitivity(&self) -> f32 {
        self.input.mouse_sensitivity
    }

    pub fn get_camera_speed(&self) -> f32 {
        self.input.camera_max_speed
    }

    pub fn is_frustum_culling_enabled(&self) -> bool {
        self.graphics.enable_frustum_culling
    }

    pub fn is_lod_enabled(&self) -> bool {
        self.graphics.enable_lod
    }

    pub fn get_max_render_distance(&self) -> f32 {
        self.graphics.max_render_distance
    }

    pub fn is_performance_monitoring_enabled(&self) -> bool {
        self.performance.enable_performance_monitoring
    }

    pub fn should_log_frame_times(&self) -> bool {
        self.performance.log_frame_times
    }

    pub fn get_target_fps(&self) -> u32 {
        self.graphics.target_fps
    }

    pub fn is_vsync_enabled(&self) -> bool {
        self.graphics.enable_vsync
    }

    pub fn is_msaa_enabled(&self) -> bool {
        self.graphics.enable_msaa
    }

    pub fn get_msaa_samples(&self) -> u32 {
        self.graphics.msaa_samples
    }
}

/// Configuration manager for runtime configuration updates
pub struct ConfigManager {
    config: DemoConfig,
    dirty: bool,
    auto_save: bool,
}

impl ConfigManager {
    pub fn new() -> Self {
        Self {
            config: DemoConfig::load(),
            dirty: false,
            auto_save: true,
        }
    }

    pub fn get(&self) -> &DemoConfig {
        &self.config
    }

    pub fn get_mut(&mut self) -> &mut DemoConfig {
        self.dirty = true;
        &mut self.config
    }

    pub fn set_auto_save(&mut self, enabled: bool) {
        self.auto_save = enabled;
    }

    pub fn save_if_dirty(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.dirty && self.auto_save {
            self.config.save()?;
            self.dirty = false;
        }
        Ok(())
    }

    pub fn reload(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.config = DemoConfig::load();
        self.dirty = false;
        Ok(())
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}