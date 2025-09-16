use crate::engine::error::RobinResult;
use crate::engine::performance::{PerformanceConfig, QualitySettings, OptimizationLevel, ShadowQuality, TextureQuality, LightingQuality, AntiAliasing};

pub struct PerformanceOptimizer {
    config: PerformanceConfig,
    optimization_strategies: Vec<OptimizationStrategy>,
    current_level: OptimizationLevel,
}

#[derive(Debug, Clone)]
pub struct OptimizationStrategy {
    pub name: String,
    pub description: String,
    pub level: OptimizationLevel,
    pub performance_impact: f32,
    pub visual_impact: f32,
    pub apply_fn: OptimizationFunction,
}

#[derive(Debug, Clone)]
pub enum OptimizationFunction {
    ReduceRenderScale(f32),
    LowerShadowQuality,
    LowerTextureQuality,
    DisablePostProcessing,
    ReduceParticles(f32),
    SimplifyLighting,
    DisableAntiAliasing,
    ReduceLodBias(f32),
    DisableVSync,
    ReduceAnisotropic(u32),
}

impl PerformanceOptimizer {
    pub fn new(config: &PerformanceConfig) -> RobinResult<Self> {
        let mut optimizer = Self {
            config: config.clone(),
            optimization_strategies: Vec::new(),
            current_level: OptimizationLevel::None,
        };
        
        optimizer.initialize_strategies();
        Ok(optimizer)
    }
    
    pub fn optimize(&mut self, quality_settings: &mut QualitySettings, target_level: OptimizationLevel) -> RobinResult<()> {
        if target_level == self.current_level {
            return Ok(());
        }
        
        // Apply optimizations based on target level
        match target_level {
            OptimizationLevel::None => self.apply_maximum_quality(quality_settings),
            OptimizationLevel::Basic => self.apply_basic_optimizations(quality_settings),
            OptimizationLevel::Moderate => self.apply_moderate_optimizations(quality_settings),
            OptimizationLevel::Aggressive => self.apply_aggressive_optimizations(quality_settings),
            OptimizationLevel::Ultra => self.apply_ultra_optimizations(quality_settings),
        }
        
        self.current_level = target_level;
        Ok(())
    }
    
    pub fn get_recommendations(&self, current_settings: &QualitySettings, target_fps: f32, current_fps: f32) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();
        
        let performance_gap = target_fps / current_fps;
        
        if performance_gap > 1.5 {
            // Significant performance issue - recommend aggressive optimizations
            recommendations.push(OptimizationRecommendation {
                priority: 1,
                title: "Reduce Render Scale".to_string(),
                description: "Lower the render resolution to improve performance".to_string(),
                expected_fps_gain: performance_gap * 0.3,
                visual_impact: 0.6,
                current_value: current_settings.render_scale.to_string(),
                recommended_value: (current_settings.render_scale * 0.8).to_string(),
                optimization_function: OptimizationFunction::ReduceRenderScale(current_settings.render_scale * 0.8),
            });
            
            if current_settings.shadow_quality != ShadowQuality::Low {
                recommendations.push(OptimizationRecommendation {
                    priority: 2,
                    title: "Lower Shadow Quality".to_string(),
                    description: "Reduce shadow resolution and filtering".to_string(),
                    expected_fps_gain: performance_gap * 0.2,
                    visual_impact: 0.4,
                    current_value: format!("{:?}", current_settings.shadow_quality),
                    recommended_value: "Low".to_string(),
                    optimization_function: OptimizationFunction::LowerShadowQuality,
                });
            }
        } else if performance_gap > 1.2 {
            // Moderate performance issue
            if current_settings.anti_aliasing != AntiAliasing::None && current_settings.anti_aliasing != AntiAliasing::FXAA {
                recommendations.push(OptimizationRecommendation {
                    priority: 1,
                    title: "Use FXAA Instead of MSAA".to_string(),
                    description: "Switch to faster anti-aliasing method".to_string(),
                    expected_fps_gain: performance_gap * 0.15,
                    visual_impact: 0.2,
                    current_value: format!("{:?}", current_settings.anti_aliasing),
                    recommended_value: "FXAA".to_string(),
                    optimization_function: OptimizationFunction::DisableAntiAliasing,
                });
            }
            
            if current_settings.particle_density > 0.7 {
                recommendations.push(OptimizationRecommendation {
                    priority: 2,
                    title: "Reduce Particle Density".to_string(),
                    description: "Lower the number of particles for better performance".to_string(),
                    expected_fps_gain: performance_gap * 0.1,
                    visual_impact: 0.3,
                    current_value: current_settings.particle_density.to_string(),
                    recommended_value: "0.7".to_string(),
                    optimization_function: OptimizationFunction::ReduceParticles(0.7),
                });
            }
        }
        
        // Sort by priority
        recommendations.sort_by_key(|r| r.priority);
        
        recommendations
    }
    
    pub fn auto_optimize(&mut self, quality_settings: &mut QualitySettings, current_fps: f32, target_fps: f32) -> RobinResult<OptimizationResult> {
        let performance_ratio = current_fps / target_fps;
        
        let target_level = if performance_ratio < 0.5 {
            OptimizationLevel::Ultra
        } else if performance_ratio < 0.7 {
            OptimizationLevel::Aggressive
        } else if performance_ratio < 0.85 {
            OptimizationLevel::Moderate
        } else if performance_ratio < 0.95 {
            OptimizationLevel::Basic
        } else {
            OptimizationLevel::None
        };
        
        let old_settings = quality_settings.clone();
        self.optimize(quality_settings, target_level)?;
        
        Ok(OptimizationResult {
            applied_optimizations: self.get_applied_optimizations(&old_settings, quality_settings),
            expected_fps_improvement: self.calculate_expected_improvement(&old_settings, quality_settings),
            visual_quality_impact: self.calculate_visual_impact(&old_settings, quality_settings),
            optimization_level: target_level,
        })
    }
    
    // Private methods
    
    fn initialize_strategies(&mut self) {
        self.optimization_strategies = vec![
            OptimizationStrategy {
                name: "Reduce Render Scale".to_string(),
                description: "Lower the render resolution".to_string(),
                level: OptimizationLevel::Basic,
                performance_impact: 0.3,
                visual_impact: 0.6,
                apply_fn: OptimizationFunction::ReduceRenderScale(0.9),
            },
            OptimizationStrategy {
                name: "Lower Shadow Quality".to_string(),
                description: "Reduce shadow resolution and filtering".to_string(),
                level: OptimizationLevel::Moderate,
                performance_impact: 0.2,
                visual_impact: 0.4,
                apply_fn: OptimizationFunction::LowerShadowQuality,
            },
            OptimizationStrategy {
                name: "Lower Texture Quality".to_string(),
                description: "Use lower resolution textures".to_string(),
                level: OptimizationLevel::Moderate,
                performance_impact: 0.15,
                visual_impact: 0.5,
                apply_fn: OptimizationFunction::LowerTextureQuality,
            },
            OptimizationStrategy {
                name: "Disable Post Processing".to_string(),
                description: "Turn off post-processing effects".to_string(),
                level: OptimizationLevel::Aggressive,
                performance_impact: 0.25,
                visual_impact: 0.7,
                apply_fn: OptimizationFunction::DisablePostProcessing,
            },
            OptimizationStrategy {
                name: "Reduce Particles".to_string(),
                description: "Lower particle density".to_string(),
                level: OptimizationLevel::Basic,
                performance_impact: 0.1,
                visual_impact: 0.3,
                apply_fn: OptimizationFunction::ReduceParticles(0.7),
            },
            OptimizationStrategy {
                name: "Simplify Lighting".to_string(),
                description: "Use forward rendering instead of deferred".to_string(),
                level: OptimizationLevel::Aggressive,
                performance_impact: 0.2,
                visual_impact: 0.4,
                apply_fn: OptimizationFunction::SimplifyLighting,
            },
            OptimizationStrategy {
                name: "Disable Anti-Aliasing".to_string(),
                description: "Turn off anti-aliasing".to_string(),
                level: OptimizationLevel::Ultra,
                performance_impact: 0.15,
                visual_impact: 0.5,
                apply_fn: OptimizationFunction::DisableAntiAliasing,
            },
        ];
    }
    
    fn apply_maximum_quality(&self, settings: &mut QualitySettings) {
        settings.render_scale = 1.0;
        settings.shadow_quality = ShadowQuality::Ultra;
        settings.texture_quality = TextureQuality::Ultra;
        settings.particle_density = 1.0;
        settings.lighting_quality = LightingQuality::PBR;
        settings.post_processing_enabled = true;
        settings.anti_aliasing = AntiAliasing::TAA;
        settings.anisotropic_filtering = 16;
        settings.vsync_enabled = true;
        settings.lod_bias = 0.0;
    }
    
    fn apply_basic_optimizations(&self, settings: &mut QualitySettings) {
        settings.render_scale = 0.9;
        settings.particle_density = 0.8;
        settings.anisotropic_filtering = 8;
        settings.lod_bias = 0.2;
    }
    
    fn apply_moderate_optimizations(&self, settings: &mut QualitySettings) {
        settings.render_scale = 0.8;
        settings.shadow_quality = ShadowQuality::Medium;
        settings.texture_quality = TextureQuality::High;
        settings.particle_density = 0.7;
        settings.anti_aliasing = AntiAliasing::FXAA;
        settings.anisotropic_filtering = 4;
        settings.lod_bias = 0.4;
    }
    
    fn apply_aggressive_optimizations(&self, settings: &mut QualitySettings) {
        settings.render_scale = 0.7;
        settings.shadow_quality = ShadowQuality::Low;
        settings.texture_quality = TextureQuality::Medium;
        settings.particle_density = 0.5;
        settings.lighting_quality = LightingQuality::Forward;
        settings.post_processing_enabled = false;
        settings.anti_aliasing = AntiAliasing::None;
        settings.anisotropic_filtering = 2;
        settings.vsync_enabled = false;
        settings.lod_bias = 0.6;
    }
    
    fn apply_ultra_optimizations(&self, settings: &mut QualitySettings) {
        settings.render_scale = 0.5;
        settings.shadow_quality = ShadowQuality::Off;
        settings.texture_quality = TextureQuality::Low;
        settings.particle_density = 0.3;
        settings.lighting_quality = LightingQuality::Forward;
        settings.post_processing_enabled = false;
        settings.anti_aliasing = AntiAliasing::None;
        settings.anisotropic_filtering = 1;
        settings.vsync_enabled = false;
        settings.lod_bias = 1.0;
    }
    
    fn get_applied_optimizations(&self, old: &QualitySettings, new: &QualitySettings) -> Vec<String> {
        let mut optimizations = Vec::new();
        
        if old.render_scale != new.render_scale {
            optimizations.push(format!("Render scale: {:.1} -> {:.1}", old.render_scale, new.render_scale));
        }
        
        if old.shadow_quality != new.shadow_quality {
            optimizations.push(format!("Shadow quality: {:?} -> {:?}", old.shadow_quality, new.shadow_quality));
        }
        
        if old.texture_quality != new.texture_quality {
            optimizations.push(format!("Texture quality: {:?} -> {:?}", old.texture_quality, new.texture_quality));
        }
        
        if old.particle_density != new.particle_density {
            optimizations.push(format!("Particle density: {:.1} -> {:.1}", old.particle_density, new.particle_density));
        }
        
        if old.lighting_quality != new.lighting_quality {
            optimizations.push(format!("Lighting: {:?} -> {:?}", old.lighting_quality, new.lighting_quality));
        }
        
        if old.post_processing_enabled != new.post_processing_enabled {
            optimizations.push(format!("Post processing: {} -> {}", old.post_processing_enabled, new.post_processing_enabled));
        }
        
        if old.anti_aliasing != new.anti_aliasing {
            optimizations.push(format!("Anti-aliasing: {:?} -> {:?}", old.anti_aliasing, new.anti_aliasing));
        }
        
        optimizations
    }
    
    fn calculate_expected_improvement(&self, old: &QualitySettings, new: &QualitySettings) -> f32 {
        let mut improvement = 1.0;
        
        // Render scale has the biggest impact
        improvement *= new.render_scale / old.render_scale;
        
        // Shadow quality impact
        improvement *= match (old.shadow_quality, new.shadow_quality) {
            (ShadowQuality::Ultra, ShadowQuality::High) => 1.15,
            (ShadowQuality::Ultra, ShadowQuality::Medium) => 1.3,
            (ShadowQuality::Ultra, ShadowQuality::Low) => 1.5,
            (ShadowQuality::Ultra, ShadowQuality::Off) => 1.8,
            (ShadowQuality::High, ShadowQuality::Medium) => 1.1,
            (ShadowQuality::High, ShadowQuality::Low) => 1.25,
            (ShadowQuality::High, ShadowQuality::Off) => 1.6,
            (ShadowQuality::Medium, ShadowQuality::Low) => 1.15,
            (ShadowQuality::Medium, ShadowQuality::Off) => 1.4,
            (ShadowQuality::Low, ShadowQuality::Off) => 1.2,
            _ => 1.0,
        };
        
        // Anti-aliasing impact
        improvement *= match (old.anti_aliasing, new.anti_aliasing) {
            (AntiAliasing::MSAA8x, _) => 1.4,
            (AntiAliasing::MSAA4x, AntiAliasing::FXAA) => 1.25,
            (AntiAliasing::MSAA4x, AntiAliasing::None) => 1.3,
            (AntiAliasing::MSAA2x, AntiAliasing::FXAA) => 1.1,
            (AntiAliasing::MSAA2x, AntiAliasing::None) => 1.15,
            (AntiAliasing::TAA, AntiAliasing::FXAA) => 1.15,
            (AntiAliasing::TAA, AntiAliasing::None) => 1.2,
            (AntiAliasing::FXAA, AntiAliasing::None) => 1.05,
            _ => 1.0,
        };
        
        // Post-processing impact
        if old.post_processing_enabled && !new.post_processing_enabled {
            improvement *= 1.2;
        }
        
        improvement
    }
    
    fn calculate_visual_impact(&self, old: &QualitySettings, new: &QualitySettings) -> f32 {
        let mut impact = 0.0;
        
        if old.render_scale != new.render_scale {
            impact += (old.render_scale - new.render_scale).abs() * 0.6;
        }
        
        if old.shadow_quality != new.shadow_quality {
            impact += match (old.shadow_quality, new.shadow_quality) {
                (ShadowQuality::Ultra, ShadowQuality::Off) => 0.8,
                (ShadowQuality::High, ShadowQuality::Off) => 0.7,
                (ShadowQuality::Medium, ShadowQuality::Off) => 0.5,
                (ShadowQuality::Low, ShadowQuality::Off) => 0.3,
                _ => 0.2,
            };
        }
        
        if old.texture_quality != new.texture_quality {
            impact += 0.4;
        }
        
        if old.anti_aliasing != new.anti_aliasing {
            impact += 0.3;
        }
        
        if old.post_processing_enabled && !new.post_processing_enabled {
            impact += 0.5;
        }
        
        impact.min(1.0)
    }
}

#[derive(Debug, Clone)]
pub struct OptimizationRecommendation {
    pub priority: u32,
    pub title: String,
    pub description: String,
    pub expected_fps_gain: f32,
    pub visual_impact: f32,
    pub current_value: String,
    pub recommended_value: String,
    pub optimization_function: OptimizationFunction,
}

#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub applied_optimizations: Vec<String>,
    pub expected_fps_improvement: f32,
    pub visual_quality_impact: f32,
    pub optimization_level: OptimizationLevel,
}