use crate::engine::graphics::{SSAOConfig, TAAConfig, VolumetricConfig, ShadowConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualityLevel {
    Low,
    Medium,
    High,
    Ultra,
}

impl Default for QualityLevel {
    fn default() -> Self {
        QualityLevel::High
    }
}

#[derive(Debug, Clone)]
pub struct QualitySettings {
    pub level: QualityLevel,
    pub ssao: SSAOConfig,
    pub taa: TAAConfig,
    pub volumetric: VolumetricConfig,
    pub shadows: ShadowConfig,
}

impl QualitySettings {
    pub fn new(level: QualityLevel) -> Self {
        let (ssao, taa, volumetric, shadows) = match level {
            QualityLevel::Low => Self::low_quality_configs(),
            QualityLevel::Medium => Self::medium_quality_configs(),
            QualityLevel::High => Self::high_quality_configs(),
            QualityLevel::Ultra => Self::ultra_quality_configs(),
        };

        Self {
            level,
            ssao,
            taa,
            volumetric,
            shadows,
        }
    }

    fn low_quality_configs() -> (SSAOConfig, TAAConfig, VolumetricConfig, ShadowConfig) {
        let ssao = SSAOConfig {
            enabled: true,
            sample_count: 16,
            radius: 0.8,
            intensity: 1.0,
            bias: 0.01,
            blur_passes: 1,
            ..Default::default()
        };

        let taa = TAAConfig {
            enabled: false, // Disabled for low quality
            ..Default::default()
        };

        let volumetric = VolumetricConfig {
            volume_resolution: (80, 45, 32),
            ray_marching_steps: 32,
            temporal_upsampling: false,
            ..Default::default()
        };

        let shadows = ShadowConfig {
            cascade_count: 2,
            shadow_map_size: 1024,
            pcf_radius: 1.0,
            ..Default::default()
        };

        (ssao, taa, volumetric, shadows)
    }

    fn medium_quality_configs() -> (SSAOConfig, TAAConfig, VolumetricConfig, ShadowConfig) {
        let ssao = SSAOConfig {
            enabled: true,
            sample_count: 32,
            radius: 1.0,
            intensity: 1.2,
            bias: 0.008,
            blur_passes: 2,
            ..Default::default()
        };

        let taa = TAAConfig {
            enabled: true,
            temporal_weight: 0.9,
            sharpness: 0.15,
            ..Default::default()
        };

        let volumetric = VolumetricConfig {
            volume_resolution: (120, 68, 48),
            ray_marching_steps: 48,
            temporal_upsampling: true,
            ..Default::default()
        };

        let shadows = ShadowConfig {
            cascade_count: 3,
            shadow_map_size: 1536,
            pcf_radius: 1.5,
            ..Default::default()
        };

        (ssao, taa, volumetric, shadows)
    }

    fn high_quality_configs() -> (SSAOConfig, TAAConfig, VolumetricConfig, ShadowConfig) {
        let ssao = SSAOConfig {
            enabled: true,
            sample_count: 64,
            radius: 1.0,
            intensity: 1.5,
            bias: 0.005,
            blur_passes: 2,
            ..Default::default()
        };

        let taa = TAAConfig {
            enabled: true,
            temporal_weight: 0.95,
            sharpness: 0.2,
            ..Default::default()
        };

        let volumetric = VolumetricConfig {
            volume_resolution: (160, 90, 64),
            ray_marching_steps: 64,
            temporal_upsampling: true,
            ..Default::default()
        };

        let shadows = ShadowConfig {
            cascade_count: 4,
            shadow_map_size: 2048,
            pcf_radius: 1.5,
            ..Default::default()
        };

        (ssao, taa, volumetric, shadows)
    }

    fn ultra_quality_configs() -> (SSAOConfig, TAAConfig, VolumetricConfig, ShadowConfig) {
        let ssao = SSAOConfig {
            enabled: true,
            sample_count: 128,
            radius: 1.2,
            intensity: 1.8,
            bias: 0.003,
            blur_passes: 3,
            ..Default::default()
        };

        let taa = TAAConfig {
            enabled: true,
            temporal_weight: 0.97,
            sharpness: 0.25,
            max_history_samples: 32,
            ..Default::default()
        };

        let volumetric = VolumetricConfig {
            volume_resolution: (240, 135, 96),
            ray_marching_steps: 96,
            temporal_upsampling: true,
            fog_density: 0.12,
            scattering_coefficient: 0.9,
            ..Default::default()
        };

        let shadows = ShadowConfig {
            cascade_count: 4,
            shadow_map_size: 4096,
            pcf_radius: 2.0,
            depth_bias: 0.003,
            normal_bias: 0.008,
            ..Default::default()
        };

        (ssao, taa, volumetric, shadows)
    }

    /// Get estimated GPU memory usage in megabytes
    pub fn estimated_memory_usage_mb(&self) -> f32 {
        let mut total_mb = 0.0;

        // SSAO textures
        let ssao_textures = 2.0 * 4.0; // 2 textures at R32Float (4 bytes per pixel) at 1080p
        total_mb += ssao_textures * 1920.0 * 1080.0 / (1024.0 * 1024.0);

        // TAA history buffers
        if self.taa.enabled {
            let taa_textures = 2.0 * 8.0; // 2 textures at Rgba16Float (8 bytes per pixel)
            total_mb += taa_textures * 1920.0 * 1080.0 / (1024.0 * 1024.0);
        }

        // Volumetric textures
        let vol_res = &self.volumetric.volume_resolution;
        let vol_memory = (vol_res.0 * vol_res.1 * vol_res.2) as f32 * 8.0; // Rgba16Float
        total_mb += vol_memory * 2.0 / (1024.0 * 1024.0); // Current + history

        // Shadow maps
        let shadow_memory = (self.shadows.shadow_map_size * self.shadows.shadow_map_size) as f32
            * self.shadows.cascade_count as f32
            * 4.0; // Depth32Float
        total_mb += shadow_memory / (1024.0 * 1024.0);

        total_mb
    }

    /// Get estimated frame time impact in milliseconds at 1080p
    pub fn estimated_frame_time_ms(&self) -> f32 {
        let mut total_ms = 0.0;

        // SSAO cost scales with sample count
        total_ms += (self.ssao.sample_count as f32 / 64.0) * 1.5;

        // TAA cost is relatively fixed
        if self.taa.enabled {
            total_ms += 0.8;
        }

        // Volumetric cost scales with volume size and ray marching steps
        let vol_cost_factor = (self.volumetric.volume_resolution.0 *
                             self.volumetric.volume_resolution.1 *
                             self.volumetric.volume_resolution.2) as f32 / (160.0 * 90.0 * 64.0);
        let ray_cost_factor = self.volumetric.ray_marching_steps as f32 / 64.0;
        total_ms += vol_cost_factor * ray_cost_factor * 2.0;

        // Shadow cost scales with map size and cascade count
        let shadow_cost_factor = (self.shadows.shadow_map_size * self.shadows.shadow_map_size) as f32 / (2048.0 * 2048.0);
        total_ms += shadow_cost_factor * self.shadows.cascade_count as f32 * 0.5;

        total_ms
    }

    /// Auto-detect appropriate quality level based on available GPU memory
    pub fn auto_detect_quality(available_vram_mb: f32) -> QualityLevel {
        if available_vram_mb >= 8192.0 {
            QualityLevel::Ultra
        } else if available_vram_mb >= 4096.0 {
            QualityLevel::High
        } else if available_vram_mb >= 2048.0 {
            QualityLevel::Medium
        } else {
            QualityLevel::Low
        }
    }

    /// Get adaptive settings based on current performance metrics
    pub fn adaptive_quality(
        current_level: QualityLevel,
        avg_frame_time_ms: f32,
        target_frame_time_ms: f32,
    ) -> QualityLevel {
        let performance_ratio = avg_frame_time_ms / target_frame_time_ms;

        match current_level {
            QualityLevel::Ultra => {
                if performance_ratio > 1.2 {
                    QualityLevel::High
                } else {
                    QualityLevel::Ultra
                }
            }
            QualityLevel::High => {
                if performance_ratio > 1.3 {
                    QualityLevel::Medium
                } else if performance_ratio < 0.7 {
                    QualityLevel::Ultra
                } else {
                    QualityLevel::High
                }
            }
            QualityLevel::Medium => {
                if performance_ratio > 1.4 {
                    QualityLevel::Low
                } else if performance_ratio < 0.6 {
                    QualityLevel::High
                } else {
                    QualityLevel::Medium
                }
            }
            QualityLevel::Low => {
                if performance_ratio < 0.5 {
                    QualityLevel::Medium
                } else {
                    QualityLevel::Low
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct QualityMetrics {
    pub current_level: QualityLevel,
    pub memory_usage_mb: f32,
    pub estimated_frame_time_ms: f32,
    pub actual_frame_time_ms: f32,
    pub performance_headroom: f32,
}

impl QualityMetrics {
    pub fn new(settings: &QualitySettings, actual_frame_time_ms: f32) -> Self {
        let estimated_frame_time = settings.estimated_frame_time_ms();
        let memory_usage = settings.estimated_memory_usage_mb();
        let performance_headroom = estimated_frame_time / actual_frame_time_ms.max(0.001);

        Self {
            current_level: settings.level,
            memory_usage_mb: memory_usage,
            estimated_frame_time_ms: estimated_frame_time,
            actual_frame_time_ms,
            performance_headroom,
        }
    }

    pub fn should_adjust_quality(&self, target_fps: f32) -> Option<QualityLevel> {
        let target_frame_time = 1000.0 / target_fps;

        if self.actual_frame_time_ms > target_frame_time * 1.2 {
            // Performance too low, reduce quality
            match self.current_level {
                QualityLevel::Ultra => Some(QualityLevel::High),
                QualityLevel::High => Some(QualityLevel::Medium),
                QualityLevel::Medium => Some(QualityLevel::Low),
                QualityLevel::Low => None,
            }
        } else if self.actual_frame_time_ms < target_frame_time * 0.7 {
            // Performance headroom available, increase quality
            match self.current_level {
                QualityLevel::Low => Some(QualityLevel::Medium),
                QualityLevel::Medium => Some(QualityLevel::High),
                QualityLevel::High => Some(QualityLevel::Ultra),
                QualityLevel::Ultra => None,
            }
        } else {
            None // Current quality is appropriate
        }
    }
}