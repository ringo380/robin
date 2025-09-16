use crate::engine::error::RobinResult;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostProcessingConfig {
    pub enable_hdr: bool,
    pub enable_bloom: bool,
    pub enable_tone_mapping: bool,
    pub enable_anti_aliasing: bool,
    pub enable_screen_space_reflections: bool,
    pub enable_ambient_occlusion: bool,
    pub enable_motion_blur: bool,
    pub enable_depth_of_field: bool,
    pub enable_color_grading: bool,
    pub enable_film_grain: bool,
    pub enable_vignette: bool,
    pub render_scale: f32,
    pub target_quality: PostProcessingQuality,
}

impl Default for PostProcessingConfig {
    fn default() -> Self {
        Self {
            enable_hdr: true,
            enable_bloom: true,
            enable_tone_mapping: true,
            enable_anti_aliasing: true,
            enable_screen_space_reflections: true,
            enable_ambient_occlusion: true,
            enable_motion_blur: false,
            enable_depth_of_field: false,
            enable_color_grading: true,
            enable_film_grain: false,
            enable_vignette: false,
            render_scale: 1.0,
            target_quality: PostProcessingQuality::High,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PostProcessingQuality {
    Ultra,
    High,
    Medium,
    Low,
}

impl PostProcessingQuality {
    pub fn get_bloom_iterations(&self) -> u32 {
        match self {
            PostProcessingQuality::Ultra => 6,
            PostProcessingQuality::High => 4,
            PostProcessingQuality::Medium => 3,
            PostProcessingQuality::Low => 2,
        }
    }

    pub fn get_ssao_samples(&self) -> u32 {
        match self {
            PostProcessingQuality::Ultra => 64,
            PostProcessingQuality::High => 32,
            PostProcessingQuality::Medium => 16,
            PostProcessingQuality::Low => 8,
        }
    }

    pub fn get_ssr_steps(&self) -> u32 {
        match self {
            PostProcessingQuality::Ultra => 128,
            PostProcessingQuality::High => 64,
            PostProcessingQuality::Medium => 32,
            PostProcessingQuality::Low => 16,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToneMappingOperator {
    Linear,
    Reinhard,
    ReinhardExtended,
    ACES,
    Uncharted2,
    Filmic,
    Custom { curve_params: [f32; 4] },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AntiAliasingMethod {
    None,
    MSAA { samples: u32 },
    FXAA,
    TAA,
    SMAA,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BloomSettings {
    pub enabled: bool,
    pub threshold: f32,
    pub intensity: f32,
    pub blur_passes: u32,
    pub radius: f32,
    pub dirt_texture: Option<String>,
    pub dirt_intensity: f32,
}

impl Default for BloomSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            threshold: 1.0,
            intensity: 0.8,
            blur_passes: 4,
            radius: 1.0,
            dirt_texture: None,
            dirt_intensity: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SSAOSettings {
    pub enabled: bool,
    pub radius: f32,
    pub intensity: f32,
    pub bias: f32,
    pub sample_count: u32,
    pub blur_passes: u32,
}

impl Default for SSAOSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            radius: 0.5,
            intensity: 1.0,
            bias: 0.025,
            sample_count: 32,
            blur_passes: 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SSRSettings {
    pub enabled: bool,
    pub max_steps: u32,
    pub step_size: f32,
    pub thickness: f32,
    pub falloff: f32,
    pub intensity: f32,
}

impl Default for SSRSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            max_steps: 64,
            step_size: 1.0,
            thickness: 0.5,
            falloff: 2.0,
            intensity: 0.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DOFSettings {
    pub enabled: bool,
    pub focus_distance: f32,
    pub aperture: f32,
    pub focal_length: f32,
    pub blur_quality: DOFQuality,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DOFQuality {
    High,
    Medium,
    Low,
}

impl Default for DOFSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            focus_distance: 10.0,
            aperture: 2.8,
            focal_length: 50.0,
            blur_quality: DOFQuality::Medium,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorGradingSettings {
    pub enabled: bool,
    pub exposure: f32,
    pub contrast: f32,
    pub brightness: f32,
    pub saturation: f32,
    pub gamma: f32,
    pub temperature: f32, // Kelvin
    pub tint: f32,
    pub shadows: [f32; 3], // RGB lift
    pub midtones: [f32; 3], // RGB gamma
    pub highlights: [f32; 3], // RGB gain
    pub lut_texture: Option<String>, // 3D LUT for color grading
    pub lut_intensity: f32,
}

impl Default for ColorGradingSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            exposure: 0.0,
            contrast: 1.0,
            brightness: 0.0,
            saturation: 1.0,
            gamma: 2.2,
            temperature: 6500.0,
            tint: 0.0,
            shadows: [0.0, 0.0, 0.0],
            midtones: [1.0, 1.0, 1.0],
            highlights: [1.0, 1.0, 1.0],
            lut_texture: None,
            lut_intensity: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotionBlurSettings {
    pub enabled: bool,
    pub intensity: f32,
    pub sample_count: u32,
    pub velocity_scale: f32,
    pub max_blur_radius: f32,
}

impl Default for MotionBlurSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            intensity: 0.5,
            sample_count: 8,
            velocity_scale: 1.0,
            max_blur_radius: 32.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilmGrainSettings {
    pub enabled: bool,
    pub intensity: f32,
    pub size: f32,
    pub luminance_contribution: f32,
}

impl Default for FilmGrainSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            intensity: 0.1,
            size: 1.0,
            luminance_contribution: 0.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VignetteSettings {
    pub enabled: bool,
    pub intensity: f32,
    pub smoothness: f32,
    pub roundness: f32,
    pub color: [f32; 3],
}

impl Default for VignetteSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            intensity: 0.3,
            smoothness: 0.4,
            roundness: 1.0,
            color: [0.0, 0.0, 0.0],
        }
    }
}

#[derive(Debug)]
pub struct RenderTarget {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub samples: u32,
    pub mip_levels: u32,
}

#[derive(Debug, Clone)]
pub enum TextureFormat {
    RGBA8,
    RGBA16F,
    RGBA32F,
    R11G11B10F,
    RGB10A2,
    R8,
    RG8,
    R16F,
    RG16F,
    Depth24Stencil8,
    Depth32F,
}

#[derive(Debug)]
pub struct PostProcessingPass {
    pub name: String,
    pub vertex_shader: String,
    pub fragment_shader: String,
    pub input_textures: Vec<String>,
    pub output_target: String,
    pub enabled: bool,
    pub uniform_data: HashMap<String, UniformValue>,
}

#[derive(Debug, Clone)]
pub enum UniformValue {
    Float(f32),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Int(i32),
    Bool(bool),
    Texture(String),
}

#[derive(Debug, Default)]
pub struct PostProcessingStats {
    pub total_passes: u32,
    pub active_passes: u32,
    pub render_targets_used: u32,
    pub texture_switches: u32,
    pub processing_time_ms: f32,
    pub memory_usage_mb: f32,
}

#[derive(Debug)]
pub struct PostProcessingPipeline {
    config: PostProcessingConfig,
    render_targets: HashMap<String, RenderTarget>,
    passes: Vec<PostProcessingPass>,
    
    // Settings for each effect
    bloom_settings: BloomSettings,
    ssao_settings: SSAOSettings,
    ssr_settings: SSRSettings,
    dof_settings: DOFSettings,
    color_grading_settings: ColorGradingSettings,
    motion_blur_settings: MotionBlurSettings,
    film_grain_settings: FilmGrainSettings,
    vignette_settings: VignetteSettings,
    
    // Tone mapping
    tone_mapping_operator: ToneMappingOperator,
    exposure: f32,
    
    // Anti-aliasing
    aa_method: AntiAliasingMethod,
    
    // Performance tracking
    stats: PostProcessingStats,
    
    // Temporal data for TAA and motion blur
    velocity_buffer: Option<String>,
    history_buffer: Option<String>,
    jitter_samples: Vec<[f32; 2]>,
    current_jitter_index: usize,
}

impl PostProcessingPipeline {
    pub fn new(config: PostProcessingConfig, viewport_width: u32, viewport_height: u32) -> RobinResult<Self> {
        let mut pipeline = Self {
            config: config.clone(),
            render_targets: HashMap::new(),
            passes: Vec::new(),
            bloom_settings: BloomSettings::default(),
            ssao_settings: SSAOSettings::default(),
            ssr_settings: SSRSettings::default(),
            dof_settings: DOFSettings::default(),
            color_grading_settings: ColorGradingSettings::default(),
            motion_blur_settings: MotionBlurSettings::default(),
            film_grain_settings: FilmGrainSettings::default(),
            vignette_settings: VignetteSettings::default(),
            tone_mapping_operator: ToneMappingOperator::ACES,
            exposure: 1.0,
            aa_method: AntiAliasingMethod::TAA,
            stats: PostProcessingStats::default(),
            velocity_buffer: None,
            history_buffer: None,
            jitter_samples: Self::generate_halton_jitter_samples(8),
            current_jitter_index: 0,
        };

        pipeline.create_render_targets(viewport_width, viewport_height)?;
        pipeline.setup_passes()?;
        
        Ok(pipeline)
    }

    pub fn process_frame(&mut self, input_color: &str, input_depth: &str, camera_matrices: &CameraMatrices) -> RobinResult<String> {
        let start_time = std::time::Instant::now();
        
        self.stats.active_passes = 0;
        self.stats.texture_switches = 0;
        
        let mut current_input = input_color.to_string();
        
        // 1. Screen Space Ambient Occlusion
        if self.config.enable_ambient_occlusion && self.ssao_settings.enabled {
            current_input = self.process_ssao(&current_input, input_depth)?;
            self.stats.active_passes += 1;
        }
        
        // 2. Screen Space Reflections
        if self.config.enable_screen_space_reflections && self.ssr_settings.enabled {
            current_input = self.process_ssr(&current_input, input_depth, camera_matrices)?;
            self.stats.active_passes += 1;
        }
        
        // 3. Bloom
        if self.config.enable_bloom && self.bloom_settings.enabled {
            current_input = self.process_bloom(&current_input)?;
            self.stats.active_passes += 1;
        }
        
        // 4. Depth of Field
        if self.config.enable_depth_of_field && self.dof_settings.enabled {
            current_input = self.process_depth_of_field(&current_input, input_depth)?;
            self.stats.active_passes += 1;
        }
        
        // 5. Motion Blur
        if self.config.enable_motion_blur && self.motion_blur_settings.enabled {
            current_input = self.process_motion_blur(&current_input, camera_matrices)?;
            self.stats.active_passes += 1;
        }
        
        // 6. Tone Mapping
        if self.config.enable_tone_mapping {
            current_input = self.process_tone_mapping(&current_input)?;
            self.stats.active_passes += 1;
        }
        
        // 7. Color Grading
        if self.config.enable_color_grading && self.color_grading_settings.enabled {
            current_input = self.process_color_grading(&current_input)?;
            self.stats.active_passes += 1;
        }
        
        // 8. Anti-aliasing
        if self.config.enable_anti_aliasing {
            current_input = self.process_anti_aliasing(&current_input, camera_matrices)?;
            self.stats.active_passes += 1;
        }
        
        // 9. Film Grain
        if self.config.enable_film_grain && self.film_grain_settings.enabled {
            current_input = self.process_film_grain(&current_input)?;
            self.stats.active_passes += 1;
        }
        
        // 10. Vignette
        if self.config.enable_vignette && self.vignette_settings.enabled {
            current_input = self.process_vignette(&current_input)?;
            self.stats.active_passes += 1;
        }
        
        self.stats.processing_time_ms = start_time.elapsed().as_secs_f32() * 1000.0;
        self.update_jitter();
        
        Ok(current_input)
    }

    fn process_ssao(&mut self, input: &str, depth_buffer: &str) -> RobinResult<String> {
        let output_target = "ssao_result";
        
        // Generate SSAO noise texture and sample kernel if not already done
        // This would typically be done once during initialization
        
        let pass = PostProcessingPass {
            name: "SSAO".to_string(),
            vertex_shader: Self::get_fullscreen_vertex_shader(),
            fragment_shader: self.generate_ssao_fragment_shader(),
            input_textures: vec![input.to_string(), depth_buffer.to_string(), "ssao_noise".to_string()],
            output_target: output_target.to_string(),
            enabled: true,
            uniform_data: {
                let mut uniforms = HashMap::new();
                uniforms.insert("u_radius".to_string(), UniformValue::Float(self.ssao_settings.radius));
                uniforms.insert("u_intensity".to_string(), UniformValue::Float(self.ssao_settings.intensity));
                uniforms.insert("u_bias".to_string(), UniformValue::Float(self.ssao_settings.bias));
                uniforms.insert("u_sample_count".to_string(), UniformValue::Int(self.ssao_settings.sample_count as i32));
                uniforms
            },
        };
        
        self.execute_pass(&pass)?;
        
        // Apply bilateral blur to reduce noise
        if self.ssao_settings.blur_passes > 0 {
            return self.apply_bilateral_blur(output_target, self.ssao_settings.blur_passes);
        }
        
        Ok(output_target.to_string())
    }

    fn process_ssr(&mut self, input: &str, depth_buffer: &str, camera_matrices: &CameraMatrices) -> RobinResult<String> {
        let output_target = "ssr_result";
        
        let pass = PostProcessingPass {
            name: "SSR".to_string(),
            vertex_shader: Self::get_fullscreen_vertex_shader(),
            fragment_shader: self.generate_ssr_fragment_shader(),
            input_textures: vec![input.to_string(), depth_buffer.to_string()],
            output_target: output_target.to_string(),
            enabled: true,
            uniform_data: {
                let mut uniforms = HashMap::new();
                uniforms.insert("u_max_steps".to_string(), UniformValue::Int(self.ssr_settings.max_steps as i32));
                uniforms.insert("u_step_size".to_string(), UniformValue::Float(self.ssr_settings.step_size));
                uniforms.insert("u_thickness".to_string(), UniformValue::Float(self.ssr_settings.thickness));
                uniforms.insert("u_intensity".to_string(), UniformValue::Float(self.ssr_settings.intensity));
                // Camera matrices would be set here
                uniforms
            },
        };
        
        self.execute_pass(&pass)?;
        Ok(output_target.to_string())
    }

    fn process_bloom(&mut self, input: &str) -> RobinResult<String> {
        // 1. Extract bright pixels
        let bright_pass_output = self.extract_bright_pixels(input)?;
        
        // 2. Apply Gaussian blur with multiple passes
        let mut blur_input = bright_pass_output;
        let iterations = self.config.target_quality.get_bloom_iterations();
        
        for i in 0..iterations {
            blur_input = self.apply_gaussian_blur(&blur_input, i as f32 + 1.0)?;
        }
        
        // 3. Composite with original
        let output_target = "bloom_composite";
        let pass = PostProcessingPass {
            name: "Bloom Composite".to_string(),
            vertex_shader: Self::get_fullscreen_vertex_shader(),
            fragment_shader: self.generate_bloom_composite_shader(),
            input_textures: vec![input.to_string(), blur_input],
            output_target: output_target.to_string(),
            enabled: true,
            uniform_data: {
                let mut uniforms = HashMap::new();
                uniforms.insert("u_bloom_intensity".to_string(), UniformValue::Float(self.bloom_settings.intensity));
                if let Some(dirt_texture) = &self.bloom_settings.dirt_texture {
                    uniforms.insert("u_dirt_texture".to_string(), UniformValue::Texture(dirt_texture.clone()));
                    uniforms.insert("u_dirt_intensity".to_string(), UniformValue::Float(self.bloom_settings.dirt_intensity));
                }
                uniforms
            },
        };
        
        self.execute_pass(&pass)?;
        Ok(output_target.to_string())
    }

    fn process_depth_of_field(&mut self, input: &str, depth_buffer: &str) -> RobinResult<String> {
        let output_target = "dof_result";
        
        let pass = PostProcessingPass {
            name: "Depth of Field".to_string(),
            vertex_shader: Self::get_fullscreen_vertex_shader(),
            fragment_shader: self.generate_dof_fragment_shader(),
            input_textures: vec![input.to_string(), depth_buffer.to_string()],
            output_target: output_target.to_string(),
            enabled: true,
            uniform_data: {
                let mut uniforms = HashMap::new();
                uniforms.insert("u_focus_distance".to_string(), UniformValue::Float(self.dof_settings.focus_distance));
                uniforms.insert("u_aperture".to_string(), UniformValue::Float(self.dof_settings.aperture));
                uniforms.insert("u_focal_length".to_string(), UniformValue::Float(self.dof_settings.focal_length));
                uniforms
            },
        };
        
        self.execute_pass(&pass)?;
        Ok(output_target.to_string())
    }

    fn process_motion_blur(&mut self, input: &str, camera_matrices: &CameraMatrices) -> RobinResult<String> {
        let output_target = "motion_blur_result";
        
        // Motion blur requires velocity buffer, which would be generated during geometry pass
        let velocity_buffer = self.velocity_buffer.as_ref()
            .ok_or_else(|| crate::engine::error::RobinError::new("Motion blur requires velocity buffer".to_string()))?;
        
        let pass = PostProcessingPass {
            name: "Motion Blur".to_string(),
            vertex_shader: Self::get_fullscreen_vertex_shader(),
            fragment_shader: self.generate_motion_blur_shader(),
            input_textures: vec![input.to_string(), velocity_buffer.clone()],
            output_target: output_target.to_string(),
            enabled: true,
            uniform_data: {
                let mut uniforms = HashMap::new();
                uniforms.insert("u_intensity".to_string(), UniformValue::Float(self.motion_blur_settings.intensity));
                uniforms.insert("u_sample_count".to_string(), UniformValue::Int(self.motion_blur_settings.sample_count as i32));
                uniforms.insert("u_max_blur_radius".to_string(), UniformValue::Float(self.motion_blur_settings.max_blur_radius));
                uniforms
            },
        };
        
        self.execute_pass(&pass)?;
        Ok(output_target.to_string())
    }

    fn process_tone_mapping(&mut self, input: &str) -> RobinResult<String> {
        let output_target = "tone_mapped";
        
        let pass = PostProcessingPass {
            name: "Tone Mapping".to_string(),
            vertex_shader: Self::get_fullscreen_vertex_shader(),
            fragment_shader: self.generate_tone_mapping_shader(),
            input_textures: vec![input.to_string()],
            output_target: output_target.to_string(),
            enabled: true,
            uniform_data: {
                let mut uniforms = HashMap::new();
                uniforms.insert("u_exposure".to_string(), UniformValue::Float(self.exposure));
                uniforms.insert("u_tone_mapping_mode".to_string(), UniformValue::Int(self.get_tone_mapping_mode_index()));
                uniforms
            },
        };
        
        self.execute_pass(&pass)?;
        Ok(output_target.to_string())
    }

    fn process_color_grading(&mut self, input: &str) -> RobinResult<String> {
        let output_target = "color_graded";
        
        let pass = PostProcessingPass {
            name: "Color Grading".to_string(),
            vertex_shader: Self::get_fullscreen_vertex_shader(),
            fragment_shader: self.generate_color_grading_shader(),
            input_textures: vec![input.to_string()],
            output_target: output_target.to_string(),
            enabled: true,
            uniform_data: {
                let mut uniforms = HashMap::new();
                uniforms.insert("u_exposure".to_string(), UniformValue::Float(self.color_grading_settings.exposure));
                uniforms.insert("u_contrast".to_string(), UniformValue::Float(self.color_grading_settings.contrast));
                uniforms.insert("u_brightness".to_string(), UniformValue::Float(self.color_grading_settings.brightness));
                uniforms.insert("u_saturation".to_string(), UniformValue::Float(self.color_grading_settings.saturation));
                uniforms.insert("u_gamma".to_string(), UniformValue::Float(self.color_grading_settings.gamma));
                uniforms.insert("u_temperature".to_string(), UniformValue::Float(self.color_grading_settings.temperature));
                uniforms.insert("u_tint".to_string(), UniformValue::Float(self.color_grading_settings.tint));
                uniforms.insert("u_shadows".to_string(), UniformValue::Vec3(self.color_grading_settings.shadows));
                uniforms.insert("u_midtones".to_string(), UniformValue::Vec3(self.color_grading_settings.midtones));
                uniforms.insert("u_highlights".to_string(), UniformValue::Vec3(self.color_grading_settings.highlights));
                
                if let Some(lut_texture) = &self.color_grading_settings.lut_texture {
                    uniforms.insert("u_lut_texture".to_string(), UniformValue::Texture(lut_texture.clone()));
                    uniforms.insert("u_lut_intensity".to_string(), UniformValue::Float(self.color_grading_settings.lut_intensity));
                }
                
                uniforms
            },
        };
        
        self.execute_pass(&pass)?;
        Ok(output_target.to_string())
    }

    fn process_anti_aliasing(&mut self, input: &str, camera_matrices: &CameraMatrices) -> RobinResult<String> {
        match self.aa_method {
            AntiAliasingMethod::None => Ok(input.to_string()),
            AntiAliasingMethod::FXAA => self.process_fxaa(input),
            AntiAliasingMethod::TAA => self.process_taa(input, camera_matrices),
            AntiAliasingMethod::SMAA => self.process_smaa(input),
            AntiAliasingMethod::MSAA { .. } => {
                // MSAA is handled during rasterization, not post-processing
                Ok(input.to_string())
            },
        }
    }

    fn process_fxaa(&mut self, input: &str) -> RobinResult<String> {
        let output_target = "fxaa_result";
        
        let pass = PostProcessingPass {
            name: "FXAA".to_string(),
            vertex_shader: Self::get_fullscreen_vertex_shader(),
            fragment_shader: self.generate_fxaa_shader(),
            input_textures: vec![input.to_string()],
            output_target: output_target.to_string(),
            enabled: true,
            uniform_data: HashMap::new(),
        };
        
        self.execute_pass(&pass)?;
        Ok(output_target.to_string())
    }

    fn process_taa(&mut self, input: &str, camera_matrices: &CameraMatrices) -> RobinResult<String> {
        let output_target = "taa_result";
        let history_buffer = self.history_buffer.as_ref()
            .unwrap_or(&"black_texture".to_string())
            .clone();
        
        let pass = PostProcessingPass {
            name: "TAA".to_string(),
            vertex_shader: Self::get_fullscreen_vertex_shader(),
            fragment_shader: self.generate_taa_shader(),
            input_textures: vec![input.to_string(), history_buffer.clone()],
            output_target: output_target.to_string(),
            enabled: true,
            uniform_data: {
                let mut uniforms = HashMap::new();
                let jitter = self.jitter_samples[self.current_jitter_index];
                uniforms.insert("u_jitter_offset".to_string(), UniformValue::Vec2(jitter));
                // Previous frame matrices would be set here
                uniforms
            },
        };
        
        self.execute_pass(&pass)?;
        
        // Copy current result to history buffer for next frame
        self.history_buffer = Some(output_target.to_string());
        
        Ok(output_target.to_string())
    }

    fn process_smaa(&mut self, input: &str) -> RobinResult<String> {
        // SMAA is a multi-pass technique (edge detection, blending weight calculation, final blend)
        let edges = self.smaa_edge_detection(input)?;
        let weights = self.smaa_blending_weights(&edges)?;
        self.smaa_final_blend(input, &weights)
    }

    fn process_film_grain(&mut self, input: &str) -> RobinResult<String> {
        let output_target = "film_grain_result";
        
        let pass = PostProcessingPass {
            name: "Film Grain".to_string(),
            vertex_shader: Self::get_fullscreen_vertex_shader(),
            fragment_shader: self.generate_film_grain_shader(),
            input_textures: vec![input.to_string()],
            output_target: output_target.to_string(),
            enabled: true,
            uniform_data: {
                let mut uniforms = HashMap::new();
                uniforms.insert("u_intensity".to_string(), UniformValue::Float(self.film_grain_settings.intensity));
                uniforms.insert("u_size".to_string(), UniformValue::Float(self.film_grain_settings.size));
                uniforms.insert("u_luminance_contrib".to_string(), UniformValue::Float(self.film_grain_settings.luminance_contribution));
                // Time uniform for animated grain
                uniforms.insert("u_time".to_string(), UniformValue::Float(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs_f32()));
                uniforms
            },
        };
        
        self.execute_pass(&pass)?;
        Ok(output_target.to_string())
    }

    fn process_vignette(&mut self, input: &str) -> RobinResult<String> {
        let output_target = "vignette_result";
        
        let pass = PostProcessingPass {
            name: "Vignette".to_string(),
            vertex_shader: Self::get_fullscreen_vertex_shader(),
            fragment_shader: self.generate_vignette_shader(),
            input_textures: vec![input.to_string()],
            output_target: output_target.to_string(),
            enabled: true,
            uniform_data: {
                let mut uniforms = HashMap::new();
                uniforms.insert("u_intensity".to_string(), UniformValue::Float(self.vignette_settings.intensity));
                uniforms.insert("u_smoothness".to_string(), UniformValue::Float(self.vignette_settings.smoothness));
                uniforms.insert("u_roundness".to_string(), UniformValue::Float(self.vignette_settings.roundness));
                uniforms.insert("u_color".to_string(), UniformValue::Vec3(self.vignette_settings.color));
                uniforms
            },
        };
        
        self.execute_pass(&pass)?;
        Ok(output_target.to_string())
    }

    // Helper methods for specific techniques
    fn extract_bright_pixels(&mut self, input: &str) -> RobinResult<String> {
        let output_target = "bright_pass";
        
        let pass = PostProcessingPass {
            name: "Bright Pass".to_string(),
            vertex_shader: Self::get_fullscreen_vertex_shader(),
            fragment_shader: self.generate_bright_pass_shader(),
            input_textures: vec![input.to_string()],
            output_target: output_target.to_string(),
            enabled: true,
            uniform_data: {
                let mut uniforms = HashMap::new();
                uniforms.insert("u_threshold".to_string(), UniformValue::Float(self.bloom_settings.threshold));
                uniforms
            },
        };
        
        self.execute_pass(&pass)?;
        Ok(output_target.to_string())
    }

    fn apply_gaussian_blur(&mut self, input: &str, blur_radius: f32) -> RobinResult<String> {
        // Two-pass separable Gaussian blur (horizontal then vertical)
        let horizontal_output = format!("{}_blur_h", input);
        let vertical_output = format!("{}_blur_v", input);
        
        // Horizontal pass
        let h_pass = PostProcessingPass {
            name: "Gaussian Blur H".to_string(),
            vertex_shader: Self::get_fullscreen_vertex_shader(),
            fragment_shader: self.generate_gaussian_blur_shader(true),
            input_textures: vec![input.to_string()],
            output_target: horizontal_output.clone(),
            enabled: true,
            uniform_data: {
                let mut uniforms = HashMap::new();
                uniforms.insert("u_blur_radius".to_string(), UniformValue::Float(blur_radius));
                uniforms
            },
        };
        self.execute_pass(&h_pass)?;
        
        // Vertical pass
        let v_pass = PostProcessingPass {
            name: "Gaussian Blur V".to_string(),
            vertex_shader: Self::get_fullscreen_vertex_shader(),
            fragment_shader: self.generate_gaussian_blur_shader(false),
            input_textures: vec![horizontal_output],
            output_target: vertical_output.clone(),
            enabled: true,
            uniform_data: {
                let mut uniforms = HashMap::new();
                uniforms.insert("u_blur_radius".to_string(), UniformValue::Float(blur_radius));
                uniforms
            },
        };
        self.execute_pass(&v_pass)?;
        
        Ok(vertical_output)
    }

    fn apply_bilateral_blur(&mut self, input: &str, passes: u32) -> RobinResult<String> {
        let mut current_input = input.to_string();
        
        for i in 0..passes {
            let output = format!("{}_bilateral_{}", input, i);
            let pass = PostProcessingPass {
                name: format!("Bilateral Blur {}", i),
                vertex_shader: Self::get_fullscreen_vertex_shader(),
                fragment_shader: self.generate_bilateral_blur_shader(),
                input_textures: vec![current_input],
                output_target: output.clone(),
                enabled: true,
                uniform_data: HashMap::new(),
            };
            self.execute_pass(&pass)?;
            current_input = output;
        }
        
        Ok(current_input)
    }

    // SMAA helper methods
    fn smaa_edge_detection(&mut self, input: &str) -> RobinResult<String> {
        let output = "smaa_edges";
        let pass = PostProcessingPass {
            name: "SMAA Edge Detection".to_string(),
            vertex_shader: Self::get_fullscreen_vertex_shader(),
            fragment_shader: self.generate_smaa_edge_shader(),
            input_textures: vec![input.to_string()],
            output_target: output.to_string(),
            enabled: true,
            uniform_data: HashMap::new(),
        };
        self.execute_pass(&pass)?;
        Ok(output.to_string())
    }

    fn smaa_blending_weights(&mut self, edges: &str) -> RobinResult<String> {
        let output = "smaa_weights";
        let pass = PostProcessingPass {
            name: "SMAA Blending Weights".to_string(),
            vertex_shader: Self::get_fullscreen_vertex_shader(),
            fragment_shader: self.generate_smaa_blend_weights_shader(),
            input_textures: vec![edges.to_string()],
            output_target: output.to_string(),
            enabled: true,
            uniform_data: HashMap::new(),
        };
        self.execute_pass(&pass)?;
        Ok(output.to_string())
    }

    fn smaa_final_blend(&mut self, color: &str, weights: &str) -> RobinResult<String> {
        let output = "smaa_final";
        let pass = PostProcessingPass {
            name: "SMAA Final Blend".to_string(),
            vertex_shader: Self::get_fullscreen_vertex_shader(),
            fragment_shader: self.generate_smaa_final_shader(),
            input_textures: vec![color.to_string(), weights.to_string()],
            output_target: output.to_string(),
            enabled: true,
            uniform_data: HashMap::new(),
        };
        self.execute_pass(&pass)?;
        Ok(output.to_string())
    }

    fn execute_pass(&mut self, pass: &PostProcessingPass) -> RobinResult<()> {
        // This would bind the framebuffer, set uniforms, bind textures, and draw
        // For now, we just track statistics
        self.stats.texture_switches += pass.input_textures.len() as u32;
        println!("Executing post-processing pass: {}", pass.name);
        Ok(())
    }

    fn create_render_targets(&mut self, width: u32, height: u32) -> RobinResult<()> {
        let render_width = (width as f32 * self.config.render_scale) as u32;
        let render_height = (height as f32 * self.config.render_scale) as u32;

        // HDR color buffer
        if self.config.enable_hdr {
            self.render_targets.insert("hdr_color".to_string(), RenderTarget {
                name: "HDR Color".to_string(),
                width: render_width,
                height: render_height,
                format: TextureFormat::RGBA16F,
                samples: 1,
                mip_levels: 1,
            });
        }

        // Various intermediate buffers
        let intermediate_targets = [
            ("ssao_result", TextureFormat::R8),
            ("ssr_result", TextureFormat::RGBA8),
            ("bloom_bright", TextureFormat::RGBA16F),
            ("bloom_blur_temp", TextureFormat::RGBA16F),
            ("bloom_composite", TextureFormat::RGBA16F),
            ("tone_mapped", TextureFormat::RGBA8),
            ("color_graded", TextureFormat::RGBA8),
            ("aa_result", TextureFormat::RGBA8),
        ];

        for (name, format) in &intermediate_targets {
            self.render_targets.insert(name.to_string(), RenderTarget {
                name: name.to_string(),
                width: render_width,
                height: render_height,
                format: format.clone(),
                samples: 1,
                mip_levels: 1,
            });
        }

        Ok(())
    }

    fn setup_passes(&mut self) -> RobinResult<()> {
        self.stats.total_passes = 0;
        
        // Count enabled passes
        if self.config.enable_ambient_occlusion { self.stats.total_passes += 1; }
        if self.config.enable_screen_space_reflections { self.stats.total_passes += 1; }
        if self.config.enable_bloom { self.stats.total_passes += 3; } // Bright pass + blur + composite
        if self.config.enable_depth_of_field { self.stats.total_passes += 1; }
        if self.config.enable_motion_blur { self.stats.total_passes += 1; }
        if self.config.enable_tone_mapping { self.stats.total_passes += 1; }
        if self.config.enable_color_grading { self.stats.total_passes += 1; }
        if self.config.enable_anti_aliasing { self.stats.total_passes += 1; }
        if self.config.enable_film_grain { self.stats.total_passes += 1; }
        if self.config.enable_vignette { self.stats.total_passes += 1; }

        Ok(())
    }

    fn update_jitter(&mut self) {
        self.current_jitter_index = (self.current_jitter_index + 1) % self.jitter_samples.len();
    }

    fn generate_halton_jitter_samples(count: usize) -> Vec<[f32; 2]> {
        let mut samples = Vec::with_capacity(count);
        
        for i in 0..count {
            let x = Self::halton_sequence(i + 1, 2) - 0.5;
            let y = Self::halton_sequence(i + 1, 3) - 0.5;
            samples.push([x, y]);
        }
        
        samples
    }

    fn halton_sequence(index: usize, base: usize) -> f32 {
        let mut result = 0.0;
        let mut f = 1.0;
        let mut i = index;
        
        while i > 0 {
            f = f / base as f32;
            result = result + f * (i % base) as f32;
            i = i / base;
        }
        
        result
    }

    fn get_tone_mapping_mode_index(&self) -> i32 {
        match self.tone_mapping_operator {
            ToneMappingOperator::Linear => 0,
            ToneMappingOperator::Reinhard => 1,
            ToneMappingOperator::ReinhardExtended => 2,
            ToneMappingOperator::ACES => 3,
            ToneMappingOperator::Uncharted2 => 4,
            ToneMappingOperator::Filmic => 5,
            ToneMappingOperator::Custom { .. } => 6,
        }
    }

    // Shader generation methods - these would generate GLSL/WGSL code
    fn get_fullscreen_vertex_shader() -> String {
        r#"
#version 450 core
layout(location = 0) in vec2 a_position;
layout(location = 1) in vec2 a_uv;

out vec2 v_uv;

void main() {
    v_uv = a_uv;
    gl_Position = vec4(a_position, 0.0, 1.0);
}
"#.to_string()
    }

    fn generate_ssao_fragment_shader(&self) -> String {
        format!(r#"
#version 450 core

in vec2 v_uv;
out float FragColor;

uniform sampler2D u_color_texture;
uniform sampler2D u_depth_texture;
uniform sampler2D u_noise_texture;
uniform float u_radius;
uniform float u_intensity;
uniform float u_bias;
uniform int u_sample_count;

void main() {{
    // SSAO implementation here
    // This is a simplified version
    float depth = texture(u_depth_texture, v_uv).r;
    float occlusion = 0.0;
    
    // Sample surrounding pixels
    for (int i = 0; i < u_sample_count; i++) {{
        // Generate sample positions and calculate occlusion
        occlusion += 0.1; // Simplified calculation
    }}
    
    occlusion = 1.0 - (occlusion / u_sample_count);
    occlusion = pow(occlusion, u_intensity);
    
    FragColor = occlusion;
}}
"#)
    }

    fn generate_ssr_fragment_shader(&self) -> String {
        "// SSR shader implementation would go here".to_string()
    }

    fn generate_bloom_composite_shader(&self) -> String {
        "// Bloom composite shader implementation would go here".to_string()
    }

    fn generate_dof_fragment_shader(&self) -> String {
        "// DOF shader implementation would go here".to_string()
    }

    fn generate_motion_blur_shader(&self) -> String {
        "// Motion blur shader implementation would go here".to_string()
    }

    fn generate_tone_mapping_shader(&self) -> String {
        "// Tone mapping shader implementation would go here".to_string()
    }

    fn generate_color_grading_shader(&self) -> String {
        "// Color grading shader implementation would go here".to_string()
    }

    fn generate_fxaa_shader(&self) -> String {
        "// FXAA shader implementation would go here".to_string()
    }

    fn generate_taa_shader(&self) -> String {
        "// TAA shader implementation would go here".to_string()
    }

    fn generate_film_grain_shader(&self) -> String {
        "// Film grain shader implementation would go here".to_string()
    }

    fn generate_vignette_shader(&self) -> String {
        "// Vignette shader implementation would go here".to_string()
    }

    fn generate_bright_pass_shader(&self) -> String {
        "// Bright pass shader implementation would go here".to_string()
    }

    fn generate_gaussian_blur_shader(&self, horizontal: bool) -> String {
        format!("// Gaussian blur shader ({}) implementation would go here", 
                if horizontal { "horizontal" } else { "vertical" })
    }

    fn generate_bilateral_blur_shader(&self) -> String {
        "// Bilateral blur shader implementation would go here".to_string()
    }

    fn generate_smaa_edge_shader(&self) -> String {
        "// SMAA edge detection shader implementation would go here".to_string()
    }

    fn generate_smaa_blend_weights_shader(&self) -> String {
        "// SMAA blending weights shader implementation would go here".to_string()
    }

    fn generate_smaa_final_shader(&self) -> String {
        "// SMAA final blend shader implementation would go here".to_string()
    }

    // Public API methods
    pub fn set_bloom_settings(&mut self, settings: BloomSettings) {
        self.bloom_settings = settings;
    }

    pub fn set_ssao_settings(&mut self, settings: SSAOSettings) {
        self.ssao_settings = settings;
    }

    pub fn set_color_grading_settings(&mut self, settings: ColorGradingSettings) {
        self.color_grading_settings = settings;
    }

    pub fn set_tone_mapping_operator(&mut self, operator: ToneMappingOperator) {
        self.tone_mapping_operator = operator;
    }

    pub fn set_exposure(&mut self, exposure: f32) {
        self.exposure = exposure;
    }

    pub fn get_stats(&self) -> &PostProcessingStats {
        &self.stats
    }

    pub fn get_current_jitter(&self) -> [f32; 2] {
        self.jitter_samples[self.current_jitter_index]
    }

    pub fn resize(&mut self, width: u32, height: u32) -> RobinResult<()> {
        self.create_render_targets(width, height)
    }

    pub fn shutdown(&mut self) -> RobinResult<()> {
        println!("Post-Processing Pipeline shutdown:");
        println!("  Total passes configured: {}", self.stats.total_passes);
        println!("  Render targets: {}", self.render_targets.len());
        println!("  Processing time: {:.2}ms", self.stats.processing_time_ms);

        self.render_targets.clear();
        self.passes.clear();

        Ok(())
    }
}

// Helper structures
#[derive(Debug, Clone)]
pub struct CameraMatrices {
    pub view: [[f32; 4]; 4],
    pub projection: [[f32; 4]; 4],
    pub view_projection: [[f32; 4]; 4],
    pub inverse_view_projection: [[f32; 4]; 4],
    pub previous_view_projection: [[f32; 4]; 4],
}

impl Default for CameraMatrices {
    fn default() -> Self {
        let identity = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        
        Self {
            view: identity,
            projection: identity,
            view_projection: identity,
            inverse_view_projection: identity,
            previous_view_projection: identity,
        }
    }
}