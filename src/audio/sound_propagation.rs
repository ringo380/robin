// Sound Propagation System for Robin Engine
// Provides realistic audio occlusion, reverb effects, and sound propagation physics

use crate::engine::error::RobinResult;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundPropagationConfig {
    pub max_ray_bounces: u32,
    pub max_propagation_distance: f32,
    pub occlusion_resolution: f32,
    pub reverb_tail_length: f32,
    pub enable_early_reflections: bool,
    pub enable_diffraction: bool,
    pub enable_transmission: bool,
    pub air_absorption_enabled: bool,
    pub doppler_enabled: bool,
}

impl Default for SoundPropagationConfig {
    fn default() -> Self {
        Self {
            max_ray_bounces: 8,
            max_propagation_distance: 500.0,
            occlusion_resolution: 0.5,
            reverb_tail_length: 5.0,
            enable_early_reflections: true,
            enable_diffraction: true,
            enable_transmission: true,
            air_absorption_enabled: true,
            doppler_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReverbSettings {
    pub room_size: f32,
    pub damping: f32,
    pub wet_level: f32,
    pub dry_level: f32,
    pub pre_delay: f32,
    pub decay_time: f32,
    pub early_reflections_level: f32,
    pub high_frequency_decay_ratio: f32,
    pub reverb_type: ReverbType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReverbType {
    Hall,
    Room,
    Cathedral,
    Cave,
    Underwater,
    Forest,
    Canyon,
    Stadium,
    Bathroom,
    Tunnel,
    Custom { impulse_response: String },
}

impl Default for ReverbSettings {
    fn default() -> Self {
        Self {
            room_size: 0.5,
            damping: 0.5,
            wet_level: 0.3,
            dry_level: 0.7,
            pre_delay: 0.02,
            decay_time: 1.5,
            early_reflections_level: 0.2,
            high_frequency_decay_ratio: 0.8,
            reverb_type: ReverbType::Room,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialAcoustics {
    pub absorption_coefficient: f32,  // 0.0 = fully reflective, 1.0 = fully absorptive
    pub transmission_coefficient: f32, // How much sound passes through
    pub density: f32,                 // Material density affects transmission
    pub surface_roughness: f32,       // Affects scattering
    pub frequency_response: FrequencyResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequencyResponse {
    pub low_freq_absorption: f32,    // <250Hz
    pub mid_freq_absorption: f32,    // 250Hz-4kHz  
    pub high_freq_absorption: f32,   // >4kHz
    pub low_freq_transmission: f32,
    pub mid_freq_transmission: f32,
    pub high_freq_transmission: f32,
}

impl Default for MaterialAcoustics {
    fn default() -> Self {
        Self {
            absorption_coefficient: 0.1,
            transmission_coefficient: 0.05,
            density: 1.0,
            surface_roughness: 0.5,
            frequency_response: FrequencyResponse {
                low_freq_absorption: 0.05,
                mid_freq_absorption: 0.1,
                high_freq_absorption: 0.15,
                low_freq_transmission: 0.8,
                mid_freq_transmission: 0.3,
                high_freq_transmission: 0.1,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReverbZone {
    pub name: String,
    pub center: [f32; 3],
    pub radius: f32,
    pub settings: ReverbSettings,
    pub priority: u8,
    pub fade_distance: f32,
    pub active: bool,
}

#[derive(Debug, Clone)]
pub struct AudioOcclusion {
    pub source_id: u32,
    pub occlusion_factor: f32,     // 0.0 = not occluded, 1.0 = fully occluded
    pub low_pass_frequency: f32,   // Hz
    pub volume_attenuation: f32,   // Multiplicative factor
    pub obstruction_factor: f32,   // Partial blocking
    pub transmission_factor: f32,  // Sound passing through materials
}

#[derive(Debug)]
pub struct PropagationPath {
    pub source_position: [f32; 3],
    pub listener_position: [f32; 3],
    pub direct_path_length: f32,
    pub reflection_points: Vec<[f32; 3]>,
    pub total_path_length: f32,
    pub materials_encountered: Vec<String>,
    pub diffraction_points: Vec<[f32; 3]>,
}

#[derive(Debug)]
pub struct EchoEffect {
    pub delay_time: f32,
    pub feedback_level: f32,
    pub wet_level: f32,
    pub filter_frequency: f32,
    pub echo_count: u32,
}

#[derive(Debug)]
pub struct SoundRaycast {
    pub origin: [f32; 3],
    pub direction: [f32; 3],
    pub max_distance: f32,
    pub ray_type: RayType,
}

#[derive(Debug, Clone, Copy)]
pub enum RayType {
    Direct,
    Reflection,
    Transmission,
    Diffraction,
}

#[derive(Debug, Default)]
pub struct PropagationStats {
    pub active_occlusions: u32,
    pub reverb_zones_active: u32,
    pub rays_cast_per_frame: u32,
    pub reflections_calculated: u32,
    pub cpu_usage_percent: f32,
    pub memory_usage_mb: f32,
    pub average_ray_bounces: f32,
}

#[derive(Debug)]
pub struct SoundPropagationSystem {
    config: SoundPropagationConfig,
    reverb_zones: HashMap<String, ReverbZone>,
    material_library: HashMap<String, MaterialAcoustics>,
    active_occlusions: HashMap<u32, AudioOcclusion>,
    propagation_cache: HashMap<String, PropagationPath>,
    listener_position: [f32; 3],
    listener_velocity: [f32; 3],
    world_geometry: Vec<WorldSurface>,
    stats: PropagationStats,
    update_timer: f32,
    echo_effects: HashMap<String, EchoEffect>,
    early_reflections: Vec<EarlyReflection>,
}

#[derive(Debug, Clone)]
struct WorldSurface {
    vertices: Vec<[f32; 3]>,
    normal: [f32; 3],
    material_id: String,
    bounds: BoundingBox,
}

#[derive(Debug, Clone)]
struct BoundingBox {
    min: [f32; 3],
    max: [f32; 3],
}

#[derive(Debug, Clone)]
struct EarlyReflection {
    delay_time: f32,
    attenuation: f32,
    direction: [f32; 3],
    frequency_response: FrequencyResponse,
}

impl SoundPropagationSystem {
    pub fn new() -> RobinResult<Self> {
        Ok(Self {
            config: SoundPropagationConfig::default(),
            reverb_zones: HashMap::new(),
            material_library: HashMap::new(),
            active_occlusions: HashMap::new(),
            propagation_cache: HashMap::new(),
            listener_position: [0.0; 3],
            listener_velocity: [0.0; 3],
            world_geometry: Vec::new(),
            stats: PropagationStats::default(),
            update_timer: 0.0,
            echo_effects: HashMap::new(),
            early_reflections: Vec::new(),
        })
    }

    pub fn initialize(&mut self) -> RobinResult<()> {
        self.load_default_materials()?;
        self.create_default_reverb_zones()?;
        self.setup_default_world_geometry()?;

        println!("Sound Propagation System initialized:");
        println!("  Max ray bounces: {}", self.config.max_ray_bounces);
        println!("  Max propagation distance: {}m", self.config.max_propagation_distance);
        println!("  Material library: {} materials", self.material_library.len());
        println!("  Early reflections: {}", self.config.enable_early_reflections);
        println!("  Diffraction enabled: {}", self.config.enable_diffraction);
        println!("  Transmission enabled: {}", self.config.enable_transmission);

        Ok(())
    }

    pub fn update(&mut self, delta_time: f32, listener_position: [f32; 3]) -> RobinResult<()> {
        self.listener_position = listener_position;
        self.update_timer += delta_time;

        // Update every 50ms for performance
        if self.update_timer >= 0.05 {
            self.update_reverb_zones()?;
            self.update_occlusion_calculations()?;
            self.calculate_early_reflections()?;
            self.update_stats();
            self.update_timer = 0.0;
        }

        Ok(())
    }

    pub fn process_audio_source(&mut self, source_id: u32, source_position: [f32; 3]) -> RobinResult<()> {
        // Calculate occlusion for this audio source
        let occlusion = self.calculate_occlusion(source_id, source_position)?;
        self.active_occlusions.insert(source_id, occlusion);

        // Update propagation path cache
        let cache_key = format!("source_{}", source_id);
        let propagation_path = self.calculate_propagation_path(source_position, self.listener_position)?;
        self.propagation_cache.insert(cache_key, propagation_path);

        Ok(())
    }

    pub fn add_reverb_zone(&mut self, name: String, center: [f32; 3], radius: f32, settings: ReverbSettings) -> RobinResult<()> {
        let zone = ReverbZone {
            name: name.clone(),
            center,
            radius,
            settings,
            priority: 5,
            fade_distance: radius * 0.2,
            active: true,
        };

        self.reverb_zones.insert(name, zone);
        Ok(())
    }

    pub fn remove_reverb_zone(&mut self, name: &str) -> RobinResult<()> {
        self.reverb_zones.remove(name);
        Ok(())
    }

    pub fn add_material(&mut self, name: String, acoustics: MaterialAcoustics) -> RobinResult<()> {
        self.material_library.insert(name, acoustics);
        Ok(())
    }

    pub fn add_world_surface(&mut self, vertices: Vec<[f32; 3]>, material_id: String) -> RobinResult<()> {
        if vertices.len() < 3 {
            return Err(crate::engine::error::RobinError::new("Surface must have at least 3 vertices".to_string()));
        }

        let normal = Self::calculate_surface_normal(&vertices[0], &vertices[1], &vertices[2]);
        let bounds = Self::calculate_bounding_box(&vertices);

        let surface = WorldSurface {
            vertices,
            normal,
            material_id,
            bounds,
        };

        self.world_geometry.push(surface);
        Ok(())
    }

    pub fn create_echo_effect(&mut self, name: String, delay_time: f32, feedback: f32) -> RobinResult<()> {
        let echo = EchoEffect {
            delay_time,
            feedback_level: feedback.clamp(0.0, 0.95),
            wet_level: 0.3,
            filter_frequency: 5000.0,
            echo_count: ((delay_time * feedback * 10.0) as u32).min(20),
        };

        self.echo_effects.insert(name, echo);
        Ok(())
    }

    pub fn get_occlusion_for_source(&self, source_id: u32) -> Option<&AudioOcclusion> {
        self.active_occlusions.get(&source_id)
    }

    pub fn get_current_reverb_settings(&self) -> ReverbSettings {
        // Find the highest priority reverb zone that contains the listener
        let mut current_settings = ReverbSettings::default();
        let mut highest_priority = 0;

        for zone in self.reverb_zones.values() {
            if zone.active && Self::point_in_sphere(self.listener_position, zone.center, zone.radius) {
                if zone.priority >= highest_priority {
                    current_settings = zone.settings.clone();
                    highest_priority = zone.priority;
                }
            }
        }

        current_settings
    }

    pub fn raycast_audio(&self, ray: SoundRaycast) -> Option<RaycastHit> {
        let mut closest_hit: Option<RaycastHit> = None;
        let mut closest_distance = ray.max_distance;

        for surface in &self.world_geometry {
            if let Some(hit_distance) = self.ray_intersect_surface(&ray, surface) {
                if hit_distance < closest_distance {
                    closest_distance = hit_distance;
                    let hit_point = [
                        ray.origin[0] + ray.direction[0] * hit_distance,
                        ray.origin[1] + ray.direction[1] * hit_distance,
                        ray.origin[2] + ray.direction[2] * hit_distance,
                    ];

                    closest_hit = Some(RaycastHit {
                        point: hit_point,
                        normal: surface.normal,
                        distance: hit_distance,
                        material_id: surface.material_id.clone(),
                    });
                }
            }
        }

        closest_hit
    }

    pub fn get_stats(&self) -> &PropagationStats {
        &self.stats
    }

    fn load_default_materials(&mut self) -> RobinResult<()> {
        // Wood materials
        self.material_library.insert("wood".to_string(), MaterialAcoustics {
            absorption_coefficient: 0.15,
            transmission_coefficient: 0.1,
            density: 0.6,
            surface_roughness: 0.7,
            frequency_response: FrequencyResponse {
                low_freq_absorption: 0.1,
                mid_freq_absorption: 0.15,
                high_freq_absorption: 0.2,
                low_freq_transmission: 0.2,
                mid_freq_transmission: 0.1,
                high_freq_transmission: 0.05,
            },
        });

        // Concrete materials
        self.material_library.insert("concrete".to_string(), MaterialAcoustics {
            absorption_coefficient: 0.05,
            transmission_coefficient: 0.01,
            density: 1.8,
            surface_roughness: 0.4,
            frequency_response: FrequencyResponse {
                low_freq_absorption: 0.02,
                mid_freq_absorption: 0.05,
                high_freq_absorption: 0.08,
                low_freq_transmission: 0.05,
                mid_freq_transmission: 0.01,
                high_freq_transmission: 0.005,
            },
        });

        // Metal materials
        self.material_library.insert("metal".to_string(), MaterialAcoustics {
            absorption_coefficient: 0.02,
            transmission_coefficient: 0.001,
            density: 2.5,
            surface_roughness: 0.2,
            frequency_response: FrequencyResponse {
                low_freq_absorption: 0.01,
                mid_freq_absorption: 0.02,
                high_freq_absorption: 0.05,
                low_freq_transmission: 0.002,
                mid_freq_transmission: 0.001,
                high_freq_transmission: 0.0005,
            },
        });

        // Glass materials
        self.material_library.insert("glass".to_string(), MaterialAcoustics {
            absorption_coefficient: 0.08,
            transmission_coefficient: 0.15,
            density: 1.2,
            surface_roughness: 0.1,
            frequency_response: FrequencyResponse {
                low_freq_absorption: 0.05,
                mid_freq_absorption: 0.08,
                high_freq_absorption: 0.12,
                low_freq_transmission: 0.2,
                mid_freq_transmission: 0.15,
                high_freq_transmission: 0.1,
            },
        });

        // Fabric/carpet materials
        self.material_library.insert("carpet".to_string(), MaterialAcoustics {
            absorption_coefficient: 0.6,
            transmission_coefficient: 0.3,
            density: 0.3,
            surface_roughness: 0.9,
            frequency_response: FrequencyResponse {
                low_freq_absorption: 0.3,
                mid_freq_absorption: 0.6,
                high_freq_absorption: 0.8,
                low_freq_transmission: 0.5,
                mid_freq_transmission: 0.3,
                high_freq_transmission: 0.1,
            },
        });

        // Water materials
        self.material_library.insert("water".to_string(), MaterialAcoustics {
            absorption_coefficient: 0.95,
            transmission_coefficient: 0.8,
            density: 1.0,
            surface_roughness: 0.1,
            frequency_response: FrequencyResponse {
                low_freq_absorption: 0.1,
                mid_freq_absorption: 0.5,
                high_freq_absorption: 0.95,
                low_freq_transmission: 0.9,
                mid_freq_transmission: 0.8,
                high_freq_transmission: 0.3,
            },
        });

        Ok(())
    }

    fn create_default_reverb_zones(&mut self) -> RobinResult<()> {
        // Large hall reverb
        self.add_reverb_zone("main_hall".to_string(), [0.0, 0.0, 0.0], 50.0, ReverbSettings {
            room_size: 0.9,
            damping: 0.3,
            wet_level: 0.4,
            dry_level: 0.6,
            pre_delay: 0.05,
            decay_time: 3.0,
            early_reflections_level: 0.3,
            high_frequency_decay_ratio: 0.6,
            reverb_type: ReverbType::Hall,
        })?;

        // Small room reverb
        self.add_reverb_zone("small_room".to_string(), [100.0, 0.0, 0.0], 10.0, ReverbSettings {
            room_size: 0.3,
            damping: 0.7,
            wet_level: 0.2,
            dry_level: 0.8,
            pre_delay: 0.01,
            decay_time: 0.8,
            early_reflections_level: 0.4,
            high_frequency_decay_ratio: 0.9,
            reverb_type: ReverbType::Room,
        })?;

        // Cave reverb
        self.add_reverb_zone("cave".to_string(), [-50.0, -10.0, 0.0], 30.0, ReverbSettings {
            room_size: 0.7,
            damping: 0.2,
            wet_level: 0.6,
            dry_level: 0.4,
            pre_delay: 0.08,
            decay_time: 5.0,
            early_reflections_level: 0.2,
            high_frequency_decay_ratio: 0.4,
            reverb_type: ReverbType::Cave,
        })?;

        Ok(())
    }

    fn setup_default_world_geometry(&mut self) -> RobinResult<()> {
        // Create some example walls and surfaces
        // Floor
        self.add_world_surface(vec![
            [-100.0, -5.0, -100.0],
            [100.0, -5.0, -100.0],
            [100.0, -5.0, 100.0],
            [-100.0, -5.0, 100.0],
        ], "concrete".to_string())?;

        // Walls
        self.add_world_surface(vec![
            [-100.0, -5.0, -100.0],
            [-100.0, 20.0, -100.0],
            [100.0, 20.0, -100.0],
            [100.0, -5.0, -100.0],
        ], "concrete".to_string())?;

        // Some furniture/obstacles
        self.add_world_surface(vec![
            [10.0, -5.0, 10.0],
            [15.0, -5.0, 10.0],
            [15.0, 5.0, 10.0],
            [10.0, 5.0, 10.0],
        ], "wood".to_string())?;

        Ok(())
    }

    fn calculate_occlusion(&self, source_id: u32, source_position: [f32; 3]) -> RobinResult<AudioOcclusion> {
        let mut occlusion_factor = 0.0;
        let mut transmission_factor = 1.0;
        let mut low_pass_frequency = 20000.0;

        // Cast ray from source to listener
        let ray = SoundRaycast {
            origin: source_position,
            direction: Self::normalize_vec3([
                self.listener_position[0] - source_position[0],
                self.listener_position[1] - source_position[1],
                self.listener_position[2] - source_position[2],
            ]),
            max_distance: Self::distance_vec3(source_position, self.listener_position),
            ray_type: RayType::Direct,
        };

        if let Some(hit) = self.raycast_audio(ray) {
            if let Some(material) = self.material_library.get(&hit.material_id) {
                // Calculate occlusion based on material properties
                occlusion_factor = material.absorption_coefficient;
                transmission_factor = material.transmission_coefficient;
                
                // Frequency-dependent attenuation
                low_pass_frequency = 20000.0 * (1.0 - material.frequency_response.high_freq_absorption);
            }
        }

        // Calculate diffraction if enabled
        if self.config.enable_diffraction && occlusion_factor > 0.5 {
            let diffraction_factor = self.calculate_diffraction_factor(source_position);
            occlusion_factor *= (1.0 - diffraction_factor);
        }

        Ok(AudioOcclusion {
            source_id,
            occlusion_factor,
            low_pass_frequency: low_pass_frequency.max(100.0),
            volume_attenuation: 1.0 - occlusion_factor * 0.8,
            obstruction_factor: occlusion_factor * 0.5,
            transmission_factor,
        })
    }

    fn calculate_propagation_path(&self, source_pos: [f32; 3], listener_pos: [f32; 3]) -> RobinResult<PropagationPath> {
        let direct_path_length = Self::distance_vec3(source_pos, listener_pos);
        let mut reflection_points = Vec::new();
        let mut materials_encountered = Vec::new();
        let mut total_path_length = direct_path_length;

        // Calculate first-order reflections if enabled
        if self.config.enable_early_reflections {
            reflection_points = self.calculate_reflection_points(source_pos, listener_pos)?;
            
            // Add reflection path lengths
            for point in &reflection_points {
                total_path_length += Self::distance_vec3(source_pos, *point) + 
                                   Self::distance_vec3(*point, listener_pos);
            }
        }

        Ok(PropagationPath {
            source_position: source_pos,
            listener_position: listener_pos,
            direct_path_length,
            reflection_points,
            total_path_length,
            materials_encountered,
            diffraction_points: Vec::new(),
        })
    }

    fn calculate_reflection_points(&self, source_pos: [f32; 3], listener_pos: [f32; 3]) -> RobinResult<Vec<[f32; 3]>> {
        let mut reflection_points = Vec::new();

        // For each surface, calculate potential mirror reflection point
        for surface in &self.world_geometry {
            if let Some(reflection_point) = self.calculate_mirror_reflection(source_pos, listener_pos, surface) {
                reflection_points.push(reflection_point);
                
                // Limit number of reflections for performance
                if reflection_points.len() >= self.config.max_ray_bounces as usize {
                    break;
                }
            }
        }

        Ok(reflection_points)
    }

    fn calculate_mirror_reflection(&self, source_pos: [f32; 3], listener_pos: [f32; 3], surface: &WorldSurface) -> Option<[f32; 3]> {
        // Simplified mirror reflection calculation
        // In a real implementation, this would be more sophisticated
        
        if surface.vertices.len() < 3 {
            return None;
        }

        let surface_center = Self::calculate_surface_center(&surface.vertices);
        
        // Check if reflection is geometrically possible
        let source_to_surface = Self::subtract_vec3(surface_center, source_pos);
        let listener_to_surface = Self::subtract_vec3(surface_center, listener_pos);
        
        let source_dot = Self::dot_product(source_to_surface, surface.normal);
        let listener_dot = Self::dot_product(listener_to_surface, surface.normal);
        
        // Both source and listener should be on the same side of the surface for reflection
        if source_dot * listener_dot < 0.0 {
            return None;
        }

        // Calculate reflection point (simplified)
        Some(surface_center)
    }

    fn calculate_diffraction_factor(&self, source_position: [f32; 3]) -> f32 {
        // Simplified diffraction calculation
        // Real implementation would use Fresnel diffraction theory
        
        let distance_to_listener = Self::distance_vec3(source_position, self.listener_position);
        
        // Diffraction is more effective at longer distances and lower frequencies
        let distance_factor = (1.0 / (1.0 + distance_to_listener * 0.01)).min(1.0);
        
        // Simplified diffraction factor
        distance_factor * 0.3
    }

    fn calculate_early_reflections(&mut self) -> RobinResult<()> {
        self.early_reflections.clear();
        
        if !self.config.enable_early_reflections {
            return Ok(());
        }

        // Calculate early reflections from nearby surfaces
        for surface in &self.world_geometry {
            if let Some(reflection) = self.calculate_early_reflection_from_surface(surface) {
                self.early_reflections.push(reflection);
                
                // Limit early reflections for performance
                if self.early_reflections.len() >= 20 {
                    break;
                }
            }
        }

        Ok(())
    }

    fn calculate_early_reflection_from_surface(&self, surface: &WorldSurface) -> Option<EarlyReflection> {
        let surface_center = Self::calculate_surface_center(&surface.vertices);
        let distance_to_surface = Self::distance_vec3(self.listener_position, surface_center);
        
        // Only calculate reflections for nearby surfaces
        if distance_to_surface > 50.0 {
            return None;
        }

        let material = self.material_library.get(&surface.material_id)?;
        
        // Calculate delay time based on distance
        let delay_time = (distance_to_surface * 2.0) / 343.0; // Speed of sound approximation
        
        // Calculate attenuation based on distance and material absorption
        let distance_attenuation = 1.0 / (1.0 + distance_to_surface * 0.01);
        let material_attenuation = 1.0 - material.absorption_coefficient;
        let total_attenuation = distance_attenuation * material_attenuation;
        
        // Direction from surface to listener
        let direction = Self::normalize_vec3(Self::subtract_vec3(self.listener_position, surface_center));

        Some(EarlyReflection {
            delay_time,
            attenuation: total_attenuation,
            direction,
            frequency_response: material.frequency_response.clone(),
        })
    }

    fn update_reverb_zones(&mut self) -> RobinResult<()> {
        let mut zones_in_range = 0;
        
        for zone in self.reverb_zones.values_mut() {
            let distance = Self::distance_vec3(self.listener_position, zone.center);
            zone.active = distance <= (zone.radius + zone.fade_distance);
            
            if zone.active {
                zones_in_range += 1;
            }
        }
        
        self.stats.reverb_zones_active = zones_in_range;
        Ok(())
    }

    fn update_occlusion_calculations(&mut self) -> RobinResult<()> {
        self.stats.active_occlusions = self.active_occlusions.len() as u32;
        
        // Clean up old occlusions (in a real implementation, this would be managed differently)
        if self.active_occlusions.len() > 100 {
            // Remove oldest entries
            let keys_to_remove: Vec<u32> = self.active_occlusions.keys().take(20).cloned().collect();
            for key in keys_to_remove {
                self.active_occlusions.remove(&key);
            }
        }
        
        Ok(())
    }

    fn ray_intersect_surface(&self, ray: &SoundRaycast, surface: &WorldSurface) -> Option<f32> {
        // Simplified ray-surface intersection
        // Real implementation would use more sophisticated triangle intersection
        
        if surface.vertices.len() < 3 {
            return None;
        }

        let surface_center = Self::calculate_surface_center(&surface.vertices);
        let to_surface = Self::subtract_vec3(surface_center, ray.origin);
        let distance = Self::dot_product(to_surface, surface.normal) / Self::dot_product(ray.direction, surface.normal);
        
        if distance > 0.0 && distance <= ray.max_distance {
            Some(distance)
        } else {
            None
        }
    }

    fn update_stats(&mut self) {
        self.stats.rays_cast_per_frame = (self.active_occlusions.len() * 2) as u32;
        self.stats.reflections_calculated = self.early_reflections.len() as u32;
        self.stats.cpu_usage_percent = (self.stats.active_occlusions as f32 * 0.1 + 
                                       self.stats.reverb_zones_active as f32 * 0.2 + 
                                       self.stats.reflections_calculated as f32 * 0.05).min(10.0);
        self.stats.memory_usage_mb = 16.0 + self.active_occlusions.len() as f32 * 0.1 + 
                                   self.early_reflections.len() as f32 * 0.05;
        
        if self.stats.rays_cast_per_frame > 0 {
            self.stats.average_ray_bounces = self.stats.reflections_calculated as f32 / self.stats.rays_cast_per_frame as f32;
        }
    }

    // Utility math functions
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

    fn subtract_vec3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
        [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
    }

    fn calculate_surface_normal(p1: &[f32; 3], p2: &[f32; 3], p3: &[f32; 3]) -> [f32; 3] {
        let v1 = Self::subtract_vec3(*p2, *p1);
        let v2 = Self::subtract_vec3(*p3, *p1);
        
        let normal = [
            v1[1] * v2[2] - v1[2] * v2[1],
            v1[2] * v2[0] - v1[0] * v2[2],
            v1[0] * v2[1] - v1[1] * v2[0],
        ];
        
        Self::normalize_vec3(normal)
    }

    fn calculate_surface_center(vertices: &[[f32; 3]]) -> [f32; 3] {
        let mut center = [0.0; 3];
        for vertex in vertices {
            center[0] += vertex[0];
            center[1] += vertex[1];
            center[2] += vertex[2];
        }
        
        let count = vertices.len() as f32;
        [center[0] / count, center[1] / count, center[2] / count]
    }

    fn calculate_bounding_box(vertices: &[[f32; 3]]) -> BoundingBox {
        let mut min = [f32::INFINITY; 3];
        let mut max = [f32::NEG_INFINITY; 3];
        
        for vertex in vertices {
            for i in 0..3 {
                min[i] = min[i].min(vertex[i]);
                max[i] = max[i].max(vertex[i]);
            }
        }
        
        BoundingBox { min, max }
    }

    fn point_in_sphere(point: [f32; 3], center: [f32; 3], radius: f32) -> bool {
        Self::distance_vec3(point, center) <= radius
    }

    pub fn shutdown(&mut self) -> RobinResult<()> {
        println!("Sound Propagation System shutdown:");
        println!("  Reverb zones: {}", self.reverb_zones.len());
        println!("  Materials in library: {}", self.material_library.len());
        println!("  World surfaces: {}", self.world_geometry.len());
        println!("  Peak active occlusions: {}", self.stats.active_occlusions);
        println!("  Peak rays per frame: {}", self.stats.rays_cast_per_frame);
        
        self.active_occlusions.clear();
        self.propagation_cache.clear();
        self.early_reflections.clear();
        
        Ok(())
    }
}

#[derive(Debug)]
pub struct RaycastHit {
    pub point: [f32; 3],
    pub normal: [f32; 3],
    pub distance: f32,
    pub material_id: String,
}