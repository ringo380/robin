use crate::engine::error::RobinResult;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedLightingConfig {
    pub enable_shadows: bool,
    pub shadow_map_size: u32,
    pub cascade_count: u32,
    pub enable_screen_space_reflections: bool,
    pub enable_ambient_occlusion: bool,
    pub enable_global_illumination: bool,
    pub enable_volumetric_lighting: bool,
    pub light_limit: u32,
    pub shadow_quality: ShadowQuality,
}

impl Default for AdvancedLightingConfig {
    fn default() -> Self {
        Self {
            enable_shadows: true,
            shadow_map_size: 2048,
            cascade_count: 4,
            enable_screen_space_reflections: true,
            enable_ambient_occlusion: true,
            enable_global_illumination: false,
            enable_volumetric_lighting: true,
            light_limit: 256,
            shadow_quality: ShadowQuality::High,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShadowQuality {
    Ultra,
    High,
    Medium,
    Low,
    Off,
}

impl ShadowQuality {
    pub fn get_shadow_map_size(&self) -> u32 {
        match self {
            ShadowQuality::Ultra => 4096,
            ShadowQuality::High => 2048,
            ShadowQuality::Medium => 1024,
            ShadowQuality::Low => 512,
            ShadowQuality::Off => 0,
        }
    }

    pub fn get_pcf_samples(&self) -> u32 {
        match self {
            ShadowQuality::Ultra => 16,
            ShadowQuality::High => 9,
            ShadowQuality::Medium => 4,
            ShadowQuality::Low => 1,
            ShadowQuality::Off => 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectionalLight {
    pub name: String,
    pub direction: [f32; 3],
    pub color: [f32; 3],
    pub intensity: f32,
    pub cast_shadows: bool,
    pub cascade_distances: [f32; 4],
    pub shadow_bias: f32,
    pub shadow_normal_bias: f32,
    pub enabled: bool,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self {
            name: "Sun".to_string(),
            direction: [-0.3, -0.8, -0.5],
            color: [1.0, 0.95, 0.8],
            intensity: 3.0,
            cast_shadows: true,
            cascade_distances: [10.0, 50.0, 200.0, 1000.0],
            shadow_bias: 0.005,
            shadow_normal_bias: 0.01,
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointLight {
    pub name: String,
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub intensity: f32,
    pub range: f32,
    pub cast_shadows: bool,
    pub shadow_bias: f32,
    pub enabled: bool,
    
    // Animation properties
    pub animation_type: LightAnimation,
    pub animation_speed: f32,
    pub animation_amplitude: f32,
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            name: "Point Light".to_string(),
            position: [0.0, 2.0, 0.0],
            color: [1.0, 1.0, 1.0],
            intensity: 1.0,
            range: 10.0,
            cast_shadows: true,
            shadow_bias: 0.01,
            enabled: true,
            animation_type: LightAnimation::None,
            animation_speed: 1.0,
            animation_amplitude: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotLight {
    pub name: String,
    pub position: [f32; 3],
    pub direction: [f32; 3],
    pub color: [f32; 3],
    pub intensity: f32,
    pub range: f32,
    pub inner_cone_angle: f32, // In degrees
    pub outer_cone_angle: f32, // In degrees
    pub cast_shadows: bool,
    pub shadow_bias: f32,
    pub enabled: bool,
    
    // Animation properties
    pub animation_type: LightAnimation,
    pub animation_speed: f32,
}

impl Default for SpotLight {
    fn default() -> Self {
        Self {
            name: "Spot Light".to_string(),
            position: [0.0, 3.0, 0.0],
            direction: [0.0, -1.0, 0.0],
            color: [1.0, 1.0, 1.0],
            intensity: 2.0,
            range: 20.0,
            inner_cone_angle: 30.0,
            outer_cone_angle: 45.0,
            cast_shadows: true,
            shadow_bias: 0.01,
            enabled: true,
            animation_type: LightAnimation::None,
            animation_speed: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AreaLight {
    pub name: String,
    pub position: [f32; 3],
    pub direction: [f32; 3],
    pub right: [f32; 3],
    pub color: [f32; 3],
    pub intensity: f32,
    pub width: f32,
    pub height: f32,
    pub shape: AreaLightShape,
    pub cast_shadows: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AreaLightShape {
    Rectangle,
    Circle,
    Tube,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LightAnimation {
    None,
    Flicker,
    Pulse,
    Oscillate,
    ColorCycle,
    Strobe,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmbientLighting {
    pub sky_color: [f32; 3],
    pub ground_color: [f32; 3],
    pub intensity: f32,
    pub environment_map: Option<String>,
    pub environment_intensity: f32,
    pub environment_rotation: f32,
}

impl Default for AmbientLighting {
    fn default() -> Self {
        Self {
            sky_color: [0.4, 0.6, 1.0],
            ground_color: [0.2, 0.15, 0.1],
            intensity: 0.1,
            environment_map: None,
            environment_intensity: 1.0,
            environment_rotation: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumetricLighting {
    pub enabled: bool,
    pub density: f32,
    pub scattering_coefficient: f32,
    pub phase_function_g: f32, // Henyey-Greenstein phase function parameter
    pub max_steps: u32,
    pub step_size: f32,
    pub intensity: f32,
    pub fog_color: [f32; 3],
}

impl Default for VolumetricLighting {
    fn default() -> Self {
        Self {
            enabled: false,
            density: 0.02,
            scattering_coefficient: 0.1,
            phase_function_g: 0.3,
            max_steps: 32,
            step_size: 1.0,
            intensity: 1.0,
            fog_color: [0.6, 0.7, 0.9],
        }
    }
}

#[derive(Debug)]
pub struct ShadowMap {
    pub light_name: String,
    pub resolution: u32,
    pub light_space_matrices: Vec<[[f32; 4]; 4]>,
    pub depth_texture_id: String,
    pub frame_buffer_id: String,
}

#[derive(Debug)]
pub struct LightingStats {
    pub directional_lights: u32,
    pub point_lights: u32,
    pub spot_lights: u32,
    pub area_lights: u32,
    pub shadow_maps_rendered: u32,
    pub light_culling_time_ms: f32,
    pub shadow_render_time_ms: f32,
    pub total_light_setup_time_ms: f32,
}

#[derive(Debug)]
pub struct AdvancedLightingSystem {
    config: AdvancedLightingConfig,
    directional_lights: HashMap<String, DirectionalLight>,
    point_lights: HashMap<String, PointLight>,
    spot_lights: HashMap<String, SpotLight>,
    area_lights: HashMap<String, AreaLight>,
    ambient_lighting: AmbientLighting,
    volumetric_lighting: VolumetricLighting,
    shadow_maps: HashMap<String, ShadowMap>,
    light_animations: HashMap<String, LightAnimationState>,
    culled_lights: Vec<String>,
    visible_lights: Vec<String>,
    stats: LightingStats,
    last_frame_time: Instant,
}

#[derive(Debug)]
struct LightAnimationState {
    pub start_time: Instant,
    pub current_phase: f32,
    pub original_intensity: f32,
    pub original_color: [f32; 3],
}

impl AdvancedLightingSystem {
    pub fn new(config: AdvancedLightingConfig) -> RobinResult<Self> {
        let mut system = Self {
            config,
            directional_lights: HashMap::new(),
            point_lights: HashMap::new(),
            spot_lights: HashMap::new(),
            area_lights: HashMap::new(),
            ambient_lighting: AmbientLighting::default(),
            volumetric_lighting: VolumetricLighting::default(),
            shadow_maps: HashMap::new(),
            light_animations: HashMap::new(),
            culled_lights: Vec::new(),
            visible_lights: Vec::new(),
            stats: LightingStats::default(),
            last_frame_time: Instant::now(),
        };

        system.initialize_default_lighting()?;
        Ok(system)
    }

    pub fn add_directional_light(&mut self, name: String, light: DirectionalLight) -> RobinResult<()> {
        if self.config.enable_shadows && light.cast_shadows {
            self.create_directional_shadow_map(&name, &light)?;
        }
        self.directional_lights.insert(name, light);
        Ok(())
    }

    pub fn add_point_light(&mut self, name: String, light: PointLight) -> RobinResult<()> {
        if light.animation_type != LightAnimation::None {
            self.light_animations.insert(name.clone(), LightAnimationState {
                start_time: Instant::now(),
                current_phase: 0.0,
                original_intensity: light.intensity,
                original_color: light.color,
            });
        }

        if self.config.enable_shadows && light.cast_shadows {
            self.create_point_shadow_map(&name, &light)?;
        }
        
        self.point_lights.insert(name, light);
        Ok(())
    }

    pub fn add_spot_light(&mut self, name: String, light: SpotLight) -> RobinResult<()> {
        if light.animation_type != LightAnimation::None {
            self.light_animations.insert(name.clone(), LightAnimationState {
                start_time: Instant::now(),
                current_phase: 0.0,
                original_intensity: light.intensity,
                original_color: light.color,
            });
        }

        if self.config.enable_shadows && light.cast_shadows {
            self.create_spot_shadow_map(&name, &light)?;
        }
        
        self.spot_lights.insert(name, light);
        Ok(())
    }

    pub fn add_area_light(&mut self, name: String, light: AreaLight) -> RobinResult<()> {
        self.area_lights.insert(name, light);
        Ok(())
    }

    pub fn update(&mut self, delta_time: f32, camera_position: [f32; 3], camera_frustum: &Frustum) -> RobinResult<()> {
        let frame_start = Instant::now();

        // Update light animations
        self.update_light_animations(delta_time)?;

        // Perform light culling
        self.perform_light_culling(camera_position, camera_frustum)?;

        // Update shadow maps for visible lights
        self.update_shadow_maps()?;

        // Update statistics
        self.stats.total_light_setup_time_ms = frame_start.elapsed().as_secs_f32() * 1000.0;
        self.last_frame_time = Instant::now();

        Ok(())
    }

    pub fn render_shadows(&mut self, scene_objects: &[RenderObject]) -> RobinResult<()> {
        let shadow_start = Instant::now();
        let mut maps_rendered = 0;

        // Render directional light shadows (cascaded shadow maps)
        for (name, light) in &self.directional_lights {
            if light.enabled && light.cast_shadows && self.visible_lights.contains(name) {
                self.render_directional_shadows(name, light, scene_objects)?;
                maps_rendered += 1;
            }
        }

        // Render point light shadows (cube shadow maps)
        for (name, light) in &self.point_lights {
            if light.enabled && light.cast_shadows && self.visible_lights.contains(name) {
                self.render_point_shadows(name, light, scene_objects)?;
                maps_rendered += 1;
            }
        }

        // Render spot light shadows
        for (name, light) in &self.spot_lights {
            if light.enabled && light.cast_shadows && self.visible_lights.contains(name) {
                self.render_spot_shadows(name, light, scene_objects)?;
                maps_rendered += 1;
            }
        }

        self.stats.shadow_maps_rendered = maps_rendered;
        self.stats.shadow_render_time_ms = shadow_start.elapsed().as_secs_f32() * 1000.0;

        Ok(())
    }

    pub fn setup_lighting_uniforms(&self, shader_uniforms: &mut ShaderUniforms) -> RobinResult<()> {
        // Set ambient lighting
        shader_uniforms.set_vec3("u_ambient_sky", &self.ambient_lighting.sky_color)?;
        shader_uniforms.set_vec3("u_ambient_ground", &self.ambient_lighting.ground_color)?;
        shader_uniforms.set_float("u_ambient_intensity", self.ambient_lighting.intensity)?;

        // Set directional lights
        let mut dir_count = 0;
        for (name, light) in &self.directional_lights {
            if light.enabled && self.visible_lights.contains(name) && dir_count < 4 {
                let prefix = format!("u_directional_lights[{}]", dir_count);
                shader_uniforms.set_vec3(&format!("{}.direction", prefix), &light.direction)?;
                shader_uniforms.set_vec3(&format!("{}.color", prefix), &light.color)?;
                shader_uniforms.set_float(&format!("{}.intensity", prefix), light.intensity)?;
                shader_uniforms.set_bool(&format!("{}.cast_shadows", prefix), light.cast_shadows)?;
                
                // Set shadow map uniforms if applicable
                if light.cast_shadows && self.shadow_maps.contains_key(name) {
                    let shadow_map = self.shadow_maps.get(name).unwrap();
                    for (cascade, matrix) in shadow_map.light_space_matrices.iter().enumerate() {
                        shader_uniforms.set_mat4(&format!("{}.shadow_matrices[{}]", prefix, cascade), matrix)?;
                    }
                    shader_uniforms.set_sampler(&format!("{}.shadow_map", prefix), &shadow_map.depth_texture_id)?;
                }
                
                dir_count += 1;
            }
        }
        shader_uniforms.set_int("u_directional_light_count", dir_count as i32)?;

        // Set point lights
        let mut point_count = 0;
        for (name, light) in &self.point_lights {
            if light.enabled && self.visible_lights.contains(name) && point_count < 64 {
                let prefix = format!("u_point_lights[{}]", point_count);
                shader_uniforms.set_vec3(&format!("{}.position", prefix), &light.position)?;
                shader_uniforms.set_vec3(&format!("{}.color", prefix), &light.color)?;
                shader_uniforms.set_float(&format!("{}.intensity", prefix), light.intensity)?;
                shader_uniforms.set_float(&format!("{}.range", prefix), light.range)?;
                shader_uniforms.set_bool(&format!("{}.cast_shadows", prefix), light.cast_shadows)?;
                
                if light.cast_shadows && self.shadow_maps.contains_key(name) {
                    let shadow_map = self.shadow_maps.get(name).unwrap();
                    shader_uniforms.set_sampler_cube(&format!("{}.shadow_cube", prefix), &shadow_map.depth_texture_id)?;
                }
                
                point_count += 1;
            }
        }
        shader_uniforms.set_int("u_point_light_count", point_count as i32)?;

        // Set spot lights
        let mut spot_count = 0;
        for (name, light) in &self.spot_lights {
            if light.enabled && self.visible_lights.contains(name) && spot_count < 32 {
                let prefix = format!("u_spot_lights[{}]", spot_count);
                shader_uniforms.set_vec3(&format!("{}.position", prefix), &light.position)?;
                shader_uniforms.set_vec3(&format!("{}.direction", prefix), &light.direction)?;
                shader_uniforms.set_vec3(&format!("{}.color", prefix), &light.color)?;
                shader_uniforms.set_float(&format!("{}.intensity", prefix), light.intensity)?;
                shader_uniforms.set_float(&format!("{}.range", prefix), light.range)?;
                shader_uniforms.set_float(&format!("{}.inner_angle", prefix), light.inner_cone_angle.to_radians())?;
                shader_uniforms.set_float(&format!("{}.outer_angle", prefix), light.outer_cone_angle.to_radians())?;
                
                if light.cast_shadows && self.shadow_maps.contains_key(name) {
                    let shadow_map = self.shadow_maps.get(name).unwrap();
                    shader_uniforms.set_mat4(&format!("{}.shadow_matrix", prefix), &shadow_map.light_space_matrices[0])?;
                    shader_uniforms.set_sampler(&format!("{}.shadow_map", prefix), &shadow_map.depth_texture_id)?;
                }
                
                spot_count += 1;
            }
        }
        shader_uniforms.set_int("u_spot_light_count", spot_count as i32)?;

        // Set volumetric lighting if enabled
        if self.config.enable_volumetric_lighting && self.volumetric_lighting.enabled {
            shader_uniforms.set_bool("u_volumetric_enabled", true)?;
            shader_uniforms.set_float("u_volumetric_density", self.volumetric_lighting.density)?;
            shader_uniforms.set_float("u_volumetric_scattering", self.volumetric_lighting.scattering_coefficient)?;
            shader_uniforms.set_float("u_volumetric_phase_g", self.volumetric_lighting.phase_function_g)?;
            shader_uniforms.set_vec3("u_fog_color", &self.volumetric_lighting.fog_color)?;
        } else {
            shader_uniforms.set_bool("u_volumetric_enabled", false)?;
        }

        Ok(())
    }

    fn initialize_default_lighting(&mut self) -> RobinResult<()> {
        // Add default sun light
        let sun = DirectionalLight::default();
        self.add_directional_light("Sun".to_string(), sun)?;

        // Enable volumetric lighting for atmosphere
        self.volumetric_lighting.enabled = self.config.enable_volumetric_lighting;

        Ok(())
    }

    fn update_light_animations(&mut self, delta_time: f32) -> RobinResult<()> {
        let current_time = Instant::now();
        
        for (light_name, animation_state) in self.light_animations.iter_mut() {
            let elapsed = current_time.duration_since(animation_state.start_time).as_secs_f32();
            
            // Update point lights
            if let Some(light) = self.point_lights.get_mut(light_name) {
                match light.animation_type {
                    LightAnimation::Flicker => {
                        let noise = (elapsed * light.animation_speed * 20.0).sin() * 
                                  (elapsed * light.animation_speed * 13.7).cos() * 0.1;
                        light.intensity = animation_state.original_intensity * (1.0 + noise * light.animation_amplitude);
                    },
                    LightAnimation::Pulse => {
                        let pulse = ((elapsed * light.animation_speed * 2.0).sin() + 1.0) * 0.5;
                        light.intensity = animation_state.original_intensity * 
                                        (0.3 + pulse * 0.7 * light.animation_amplitude);
                    },
                    LightAnimation::Oscillate => {
                        let osc = (elapsed * light.animation_speed).sin();
                        light.position[1] += osc * light.animation_amplitude * delta_time;
                    },
                    LightAnimation::ColorCycle => {
                        let hue = (elapsed * light.animation_speed * 0.5) % 1.0;
                        light.color = hue_to_rgb(hue);
                    },
                    LightAnimation::Strobe => {
                        let strobe = ((elapsed * light.animation_speed * 10.0).sin() > 0.5) as i32 as f32;
                        light.intensity = animation_state.original_intensity * strobe;
                    },
                    LightAnimation::None => {},
                }
            }
            
            // Update spot lights similarly
            if let Some(light) = self.spot_lights.get_mut(light_name) {
                match light.animation_type {
                    LightAnimation::Flicker => {
                        let noise = (elapsed * light.animation_speed * 20.0).sin() * 
                                  (elapsed * light.animation_speed * 13.7).cos() * 0.1;
                        light.intensity = animation_state.original_intensity * (1.0 + noise);
                    },
                    LightAnimation::ColorCycle => {
                        let hue = (elapsed * light.animation_speed * 0.5) % 1.0;
                        light.color = hue_to_rgb(hue);
                    },
                    _ => {},
                }
            }
        }
        
        Ok(())
    }

    fn perform_light_culling(&mut self, camera_position: [f32; 3], frustum: &Frustum) -> RobinResult<()> {
        let cull_start = Instant::now();
        
        self.visible_lights.clear();
        self.culled_lights.clear();

        // Directional lights are always visible (infinite range)
        for name in self.directional_lights.keys() {
            self.visible_lights.push(name.clone());
        }

        // Cull point lights
        for (name, light) in &self.point_lights {
            if light.enabled && self.is_light_visible_point(light, frustum, camera_position) {
                self.visible_lights.push(name.clone());
            } else {
                self.culled_lights.push(name.clone());
            }
        }

        // Cull spot lights
        for (name, light) in &self.spot_lights {
            if light.enabled && self.is_light_visible_spot(light, frustum, camera_position) {
                self.visible_lights.push(name.clone());
            } else {
                self.culled_lights.push(name.clone());
            }
        }

        self.stats.light_culling_time_ms = cull_start.elapsed().as_secs_f32() * 1000.0;
        Ok(())
    }

    fn is_light_visible_point(&self, light: &PointLight, frustum: &Frustum, _camera_pos: [f32; 3]) -> bool {
        // Simple sphere-frustum intersection test
        frustum.intersects_sphere(&light.position, light.range)
    }

    fn is_light_visible_spot(&self, light: &SpotLight, frustum: &Frustum, _camera_pos: [f32; 3]) -> bool {
        // More complex cone-frustum intersection test
        // For now, just check if the light position is within range
        frustum.intersects_sphere(&light.position, light.range)
    }

    fn create_directional_shadow_map(&mut self, name: &str, light: &DirectionalLight) -> RobinResult<()> {
        let resolution = self.config.shadow_map_size;
        let shadow_map = ShadowMap {
            light_name: name.to_string(),
            resolution,
            light_space_matrices: vec![[[0.0; 4]; 4]; self.config.cascade_count as usize],
            depth_texture_id: format!("{}_shadow_depth", name),
            frame_buffer_id: format!("{}_shadow_fb", name),
        };
        
        self.shadow_maps.insert(name.to_string(), shadow_map);
        Ok(())
    }

    fn create_point_shadow_map(&mut self, name: &str, _light: &PointLight) -> RobinResult<()> {
        let resolution = self.config.shadow_map_size;
        let shadow_map = ShadowMap {
            light_name: name.to_string(),
            resolution,
            light_space_matrices: vec![[[0.0; 4]; 4]; 6], // 6 faces for cube map
            depth_texture_id: format!("{}_shadow_cube", name),
            frame_buffer_id: format!("{}_shadow_fb", name),
        };
        
        self.shadow_maps.insert(name.to_string(), shadow_map);
        Ok(())
    }

    fn create_spot_shadow_map(&mut self, name: &str, _light: &SpotLight) -> RobinResult<()> {
        let resolution = self.config.shadow_map_size;
        let shadow_map = ShadowMap {
            light_name: name.to_string(),
            resolution,
            light_space_matrices: vec![[[0.0; 4]; 4]; 1],
            depth_texture_id: format!("{}_shadow_depth", name),
            frame_buffer_id: format!("{}_shadow_fb", name),
        };
        
        self.shadow_maps.insert(name.to_string(), shadow_map);
        Ok(())
    }

    fn update_shadow_maps(&mut self) -> RobinResult<()> {
        // Update light-space matrices for shadow mapping
        // This would calculate the appropriate projection matrices for each light
        Ok(())
    }

    fn render_directional_shadows(&mut self, _name: &str, _light: &DirectionalLight, _objects: &[RenderObject]) -> RobinResult<()> {
        // Render scene from light's perspective for cascaded shadow mapping
        Ok(())
    }

    fn render_point_shadows(&mut self, _name: &str, _light: &PointLight, _objects: &[RenderObject]) -> RobinResult<()> {
        // Render scene to cube map for point light shadows
        Ok(())
    }

    fn render_spot_shadows(&mut self, _name: &str, _light: &SpotLight, _objects: &[RenderObject]) -> RobinResult<()> {
        // Render scene from spot light's perspective
        Ok(())
    }

    pub fn set_time_of_day(&mut self, hour: f32) -> RobinResult<()> {
        // Update sun light based on time of day (0-24 hours)
        if let Some(sun) = self.directional_lights.get_mut("Sun") {
            let angle = (hour / 24.0 * 2.0 * std::f32::consts::PI) - std::f32::consts::PI / 2.0;
            
            // Update sun direction
            sun.direction = [
                0.0,
                angle.sin(),
                angle.cos(),
            ];
            
            // Update sun color based on time
            let sun_height = angle.sin();
            if sun_height > 0.0 {
                // Day time
                let warmth = (1.0 - sun_height).min(0.3);
                sun.color = [1.0, 1.0 - warmth * 0.2, 1.0 - warmth * 0.4];
                sun.intensity = 3.0 * sun_height;
            } else {
                // Night time
                sun.intensity = 0.0;
            }
            
            // Update ambient lighting
            if sun_height > 0.0 {
                self.ambient_lighting.intensity = 0.1 + sun_height * 0.1;
            } else {
                self.ambient_lighting.intensity = 0.05; // Night ambient
            }
        }
        
        Ok(())
    }

    pub fn get_stats(&self) -> &LightingStats {
        &self.stats
    }

    pub fn get_visible_light_count(&self) -> usize {
        self.visible_lights.len()
    }

    pub fn get_shadow_map_count(&self) -> usize {
        self.shadow_maps.len()
    }

    pub fn shutdown(&mut self) -> RobinResult<()> {
        println!("Advanced Lighting System shutdown:");
        println!("  Directional lights: {}", self.directional_lights.len());
        println!("  Point lights: {}", self.point_lights.len());
        println!("  Spot lights: {}", self.spot_lights.len());
        println!("  Area lights: {}", self.area_lights.len());
        println!("  Shadow maps: {}", self.shadow_maps.len());

        self.directional_lights.clear();
        self.point_lights.clear();
        self.spot_lights.clear();
        self.area_lights.clear();
        self.shadow_maps.clear();
        self.light_animations.clear();

        Ok(())
    }
}

impl Default for LightingStats {
    fn default() -> Self {
        Self {
            directional_lights: 0,
            point_lights: 0,
            spot_lights: 0,
            area_lights: 0,
            shadow_maps_rendered: 0,
            light_culling_time_ms: 0.0,
            shadow_render_time_ms: 0.0,
            total_light_setup_time_ms: 0.0,
        }
    }
}

// Helper structures and functions

#[derive(Debug, Clone)]
pub struct Frustum {
    pub planes: [[f32; 4]; 6], // 6 frustum planes (left, right, bottom, top, near, far)
}

impl Frustum {
    pub fn intersects_sphere(&self, center: &[f32; 3], radius: f32) -> bool {
        // Test sphere against all 6 frustum planes
        for plane in &self.planes {
            let distance = plane[0] * center[0] + plane[1] * center[1] + plane[2] * center[2] + plane[3];
            if distance < -radius {
                return false; // Sphere is completely outside this plane
            }
        }
        true // Sphere intersects or is inside frustum
    }
}

#[derive(Debug, Clone)]
pub struct RenderObject {
    pub position: [f32; 3],
    pub scale: [f32; 3],
    pub mesh_id: String,
    pub material_id: String,
}

// Placeholder for shader uniform interface
pub trait ShaderUniforms {
    fn set_float(&mut self, name: &str, value: f32) -> RobinResult<()>;
    fn set_int(&mut self, name: &str, value: i32) -> RobinResult<()>;
    fn set_bool(&mut self, name: &str, value: bool) -> RobinResult<()>;
    fn set_vec3(&mut self, name: &str, value: &[f32; 3]) -> RobinResult<()>;
    fn set_mat4(&mut self, name: &str, value: &[[f32; 4]; 4]) -> RobinResult<()>;
    fn set_sampler(&mut self, name: &str, texture_id: &str) -> RobinResult<()>;
    fn set_sampler_cube(&mut self, name: &str, texture_id: &str) -> RobinResult<()>;
}

fn hue_to_rgb(hue: f32) -> [f32; 3] {
    let h = hue * 6.0;
    let c = 1.0;
    let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
    
    match h as i32 {
        0 => [c, x, 0.0],
        1 => [x, c, 0.0],
        2 => [0.0, c, x],
        3 => [0.0, x, c],
        4 => [x, 0.0, c],
        5 => [c, 0.0, x],
        _ => [c, x, 0.0],
    }
}