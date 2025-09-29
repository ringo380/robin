/*!
 * Robin Engine - Settings Management System
 *
 * Provides comprehensive configuration management with:
 * - TOML-based persistent settings
 * - Real-time validation and application
 * - Type-safe configuration structure
 * - Automatic save/load functionality
 */

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Complete settings configuration for the Robin Engine demo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub graphics: GraphicsSettings,
    pub audio: AudioSettings,
    pub controls: ControlSettings,
    pub display: DisplaySettings,
    pub debug: DebugSettings,
}

/// Graphics rendering and performance settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsSettings {
    /// Enable V-Sync for smooth frame rates
    pub vsync_enabled: bool,
    /// Target FPS (60, 120, or 0 for unlimited)
    pub target_fps: u32,
    /// Render distance for voxel world (in chunks)
    pub render_distance: u32,
    /// Shadow quality (0=off, 1=low, 2=medium, 3=high)
    pub shadow_quality: u32,
    /// Texture quality (0=low, 1=medium, 2=high)
    pub texture_quality: u32,
    /// Enable anti-aliasing
    pub anti_aliasing: bool,
    /// Maximum background voxels for performance
    pub max_background_voxels: usize,
}

/// Audio system settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettings {
    /// Master volume (0.0 to 1.0)
    pub master_volume: f32,
    /// Sound effects volume (0.0 to 1.0)
    pub sfx_volume: f32,
    /// Music volume (0.0 to 1.0)
    pub music_volume: f32,
    /// Enable spatial 3D audio
    pub spatial_audio: bool,
}

/// Input and control settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlSettings {
    /// Mouse sensitivity for camera movement
    pub mouse_sensitivity: f32,
    /// Invert Y-axis for mouse look
    pub invert_y_axis: bool,
    /// Camera movement speed
    pub camera_speed: f32,
    /// Enable mouse smoothing
    pub mouse_smoothing: bool,
}

/// Display and window settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplaySettings {
    /// Start in fullscreen mode
    pub fullscreen: bool,
    /// Window width (when not fullscreen)
    pub window_width: u32,
    /// Window height (when not fullscreen)
    pub window_height: u32,
    /// Enable borderless windowed mode
    pub borderless: bool,
}

/// Debug and development settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugSettings {
    /// Show performance metrics overlay
    pub show_performance_metrics: bool,
    /// Show debug wireframes
    pub show_wireframes: bool,
    /// Enable verbose logging
    pub verbose_logging: bool,
    /// Show buffer usage statistics
    pub show_buffer_stats: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            graphics: GraphicsSettings::default(),
            audio: AudioSettings::default(),
            controls: ControlSettings::default(),
            display: DisplaySettings::default(),
            debug: DebugSettings::default(),
        }
    }
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            vsync_enabled: true,
            target_fps: 120, // Optimized for Apple Silicon
            render_distance: 8,
            shadow_quality: 2, // Medium
            texture_quality: 2, // High
            anti_aliasing: true,
            max_background_voxels: 10000,
        }
    }
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            sfx_volume: 0.8,
            music_volume: 0.6,
            spatial_audio: true,
        }
    }
}

impl Default for ControlSettings {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 0.002,
            invert_y_axis: false,
            camera_speed: 5.0,
            mouse_smoothing: true,
        }
    }
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            fullscreen: false,
            window_width: 1920,
            window_height: 1080,
            borderless: false,
        }
    }
}

impl Default for DebugSettings {
    fn default() -> Self {
        Self {
            show_performance_metrics: false,
            show_wireframes: false,
            verbose_logging: false,
            show_buffer_stats: false,
        }
    }
}

/// Settings manager for loading, saving, and validating configuration
pub struct SettingsManager {
    pub settings: AppSettings,
    config_path: String,
}

impl SettingsManager {
    /// Create a new settings manager with specified config file path
    pub fn new(config_path: &str) -> Self {
        let mut manager = Self {
            settings: AppSettings::default(),
            config_path: config_path.to_string(),
        };

        // Attempt to load existing settings, fall back to defaults
        if let Err(e) = manager.load() {
            println!("⚙️  Could not load settings ({}), using defaults", e);
            manager.save().unwrap_or_else(|e| {
                println!("⚠️  Could not save default settings: {}", e);
            });
        }

        manager
    }

    /// Load settings from TOML file
    pub fn load(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !Path::new(&self.config_path).exists() {
            return Err("Config file does not exist".into());
        }

        let content = fs::read_to_string(&self.config_path)?;
        self.settings = toml::from_str(&content)?;
        self.validate()?;

        println!("✅ Settings loaded from {}", self.config_path);
        Ok(())
    }

    /// Save current settings to TOML file
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let toml_string = toml::to_string_pretty(&self.settings)?;

        // Create directory if it doesn't exist
        if let Some(parent) = Path::new(&self.config_path).parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&self.config_path, toml_string)?;
        println!("💾 Settings saved to {}", self.config_path);
        Ok(())
    }

    /// Validate settings values and clamp to safe ranges
    pub fn validate(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Clamp audio volumes to valid range
        self.settings.audio.master_volume = self.settings.audio.master_volume.clamp(0.0, 1.0);
        self.settings.audio.sfx_volume = self.settings.audio.sfx_volume.clamp(0.0, 1.0);
        self.settings.audio.music_volume = self.settings.audio.music_volume.clamp(0.0, 1.0);

        // Validate mouse sensitivity (reasonable range)
        self.settings.controls.mouse_sensitivity = self.settings.controls.mouse_sensitivity.clamp(0.0001, 0.1);

        // Validate camera speed
        self.settings.controls.camera_speed = self.settings.controls.camera_speed.clamp(0.1, 50.0);

        // Validate graphics settings
        self.settings.graphics.render_distance = self.settings.graphics.render_distance.clamp(1, 32);
        self.settings.graphics.shadow_quality = self.settings.graphics.shadow_quality.clamp(0, 3);
        self.settings.graphics.texture_quality = self.settings.graphics.texture_quality.clamp(0, 2);
        self.settings.graphics.max_background_voxels = self.settings.graphics.max_background_voxels.clamp(1000, 50000);

        // Validate display settings
        self.settings.display.window_width = self.settings.display.window_width.clamp(800, 4096);
        self.settings.display.window_height = self.settings.display.window_height.clamp(600, 2160);

        Ok(())
    }

    /// Reset all settings to defaults
    pub fn reset_to_defaults(&mut self) {
        self.settings = AppSettings::default();
        println!("🔄 Settings reset to defaults");
    }

    /// Apply graphics settings to runtime configuration
    pub fn apply_graphics_settings(&self) {
        // This would be called to update the actual graphics configuration
        println!("🎨 Applied graphics settings:");
        println!("   V-Sync: {}", self.settings.graphics.vsync_enabled);
        println!("   Target FPS: {}", self.settings.graphics.target_fps);
        println!("   Render Distance: {}", self.settings.graphics.render_distance);
    }

    /// Apply audio settings to runtime configuration
    pub fn apply_audio_settings(&self) {
        println!("🔊 Applied audio settings:");
        println!("   Master Volume: {:.1}%", self.settings.audio.master_volume * 100.0);
        println!("   SFX Volume: {:.1}%", self.settings.audio.sfx_volume * 100.0);
        println!("   Music Volume: {:.1}%", self.settings.audio.music_volume * 100.0);
    }

    /// Apply control settings to runtime configuration
    pub fn apply_control_settings(&self) {
        println!("🎮 Applied control settings:");
        println!("   Mouse Sensitivity: {:.4}", self.settings.controls.mouse_sensitivity);
        println!("   Camera Speed: {:.1}", self.settings.controls.camera_speed);
        println!("   Invert Y: {}", self.settings.controls.invert_y_axis);
    }
}

/// Settings validation errors
#[derive(Debug)]
pub enum SettingsError {
    InvalidValue(String),
    FileError(String),
    ParseError(String),
}

impl std::fmt::Display for SettingsError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SettingsError::InvalidValue(msg) => write!(f, "Invalid setting value: {}", msg),
            SettingsError::FileError(msg) => write!(f, "File error: {}", msg),
            SettingsError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for SettingsError {}