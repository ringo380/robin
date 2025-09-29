// Advanced PBR Lighting System for Robin Engine
// Implements physically-based rendering with modern lighting techniques
// Optimized for voxel environments and Apple Silicon Metal rendering

use std::collections::HashMap;
use crate::material_batching::MaterialType;
use cgmath::{Vector3, Point3, InnerSpace};

/// Vector3 extension trait for linear interpolation
trait Vector3Ext {
    fn lerp(&self, other: Self, t: f32) -> Self;
}

impl Vector3Ext for Vector3<f32> {
    fn lerp(&self, other: Self, t: f32) -> Self {
        *self + (other - *self) * t.clamp(0.0, 1.0)
    }
}

/// PBR material properties for physically accurate lighting
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PBRMaterial {
    pub albedo: [f32; 4],           // Base color (RGB + metallic factor)
    pub roughness: f32,             // Surface roughness (0.0 = mirror, 1.0 = completely rough)
    pub metallic: f32,              // Metallic factor (0.0 = dielectric, 1.0 = metallic)
    pub normal_strength: f32,       // Normal map intensity
    pub emission: [f32; 3],         // Emissive color (for glowing materials)
    pub ao_strength: f32,           // Ambient occlusion strength
    pub subsurface: f32,            // Subsurface scattering factor (for organic materials)
    pub specular: f32,              // Specular reflection intensity
    pub anisotropy: f32,            // Anisotropic reflection (for brushed metal, wood grain)
    pub sheen: f32,                 // Fabric-like reflection
    pub clearcoat: f32,             // Clear coat layer (for car paint, ceramics)
    pub clearcoat_roughness: f32,   // Clear coat roughness
}

impl Default for PBRMaterial {
    fn default() -> Self {
        Self {
            albedo: [0.8, 0.8, 0.8, 0.0],
            roughness: 0.5,
            metallic: 0.0,
            normal_strength: 1.0,
            emission: [0.0, 0.0, 0.0],
            ao_strength: 1.0,
            subsurface: 0.0,
            specular: 0.5,
            anisotropy: 0.0,
            sheen: 0.0,
            clearcoat: 0.0,
            clearcoat_roughness: 0.1,
        }
    }
}

/// Light types for the PBR system
#[derive(Debug, Clone)]
pub enum LightType {
    Directional {
        direction: Vector3<f32>,
        color: Vector3<f32>,
        intensity: f32,
        shadow_cascade_count: u32,
    },
    Point {
        position: Point3<f32>,
        color: Vector3<f32>,
        intensity: f32,
        radius: f32,
        falloff: f32,
    },
    Spot {
        position: Point3<f32>,
        direction: Vector3<f32>,
        color: Vector3<f32>,
        intensity: f32,
        inner_cone: f32,
        outer_cone: f32,
        radius: f32,
    },
    Area {
        position: Point3<f32>,
        size: Vector3<f32>,
        color: Vector3<f32>,
        intensity: f32,
        texture_id: Option<u32>,
    },
}

/// GPU-compatible light data structure
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GPULight {
    pub position: [f32; 4],         // w component stores light type
    pub direction: [f32; 4],        // w component stores inner cone angle
    pub color: [f32; 4],            // w component stores intensity
    pub params: [f32; 4],           // radius, falloff, outer_cone, shadow_index
}

/// Advanced light for complex materials
pub struct Light {
    pub light_type: LightType,
    pub enabled: bool,
    pub cast_shadows: bool,
    pub volumetric: bool,
    pub temperature: f32,           // Color temperature in Kelvin
    pub ies_profile: Option<String>, // IES light profile for realistic lighting
}

impl Light {
    pub fn new_sun() -> Self {
        Self {
            light_type: LightType::Directional {
                direction: Vector3::new(-0.3, -0.8, -0.5).normalize(),
                color: Self::temperature_to_rgb(5778.0), // Sun's color temperature
                intensity: 3.0,
                shadow_cascade_count: 4,
            },
            enabled: true,
            cast_shadows: true,
            volumetric: true,
            temperature: 5778.0,
            ies_profile: None,
        }
    }

    pub fn new_torch() -> Self {
        Self {
            light_type: LightType::Point {
                position: Point3::new(0.0, 0.0, 0.0),
                color: Self::temperature_to_rgb(1900.0), // Flame temperature
                intensity: 2.0,
                radius: 15.0,
                falloff: 2.0,
            },
            enabled: true,
            cast_shadows: true,
            volumetric: true,
            temperature: 1900.0,
            ies_profile: None,
        }
    }

    pub fn new_lava_glow() -> Self {
        Self {
            light_type: LightType::Point {
                position: Point3::new(0.0, 0.0, 0.0),
                color: Vector3::new(1.0, 0.3, 0.1),
                intensity: 1.5,
                radius: 8.0,
                falloff: 1.5,
            },
            enabled: true,
            cast_shadows: false,
            volumetric: true,
            temperature: 1200.0,
            ies_profile: None,
        }
    }

    /// Convert color temperature (Kelvin) to RGB
    fn temperature_to_rgb(temp: f32) -> Vector3<f32> {
        let temp = temp.clamp(1000.0, 40000.0);
        let temp_100 = temp / 100.0;

        let red = if temp <= 6600.0 {
            1.0
        } else {
            let r = 329.698727446 * (temp_100 - 60.0).powf(-0.1332047592);
            (r / 255.0).clamp(0.0, 1.0)
        };

        let green = if temp <= 6600.0 {
            let g = 99.4708025861 * temp_100.ln() - 161.1195681661;
            (g / 255.0).clamp(0.0, 1.0)
        } else {
            let g = 288.1221695283 * (temp_100 - 60.0).powf(-0.0755148492);
            (g / 255.0).clamp(0.0, 1.0)
        };

        let blue = if temp >= 6600.0 {
            1.0
        } else if temp <= 1900.0 {
            0.0
        } else {
            let b = 138.5177312231 * (temp_100 - 10.0).ln() - 305.0447927307;
            (b / 255.0).clamp(0.0, 1.0)
        };

        Vector3::new(red, green, blue)
    }

    pub fn to_gpu_light(&self) -> GPULight {
        match &self.light_type {
            LightType::Directional { direction, color, intensity, shadow_cascade_count } => {
                GPULight {
                    position: [0.0, 0.0, 0.0, 0.0], // Directional lights don't have position
                    direction: [direction.x, direction.y, direction.z, 0.0],
                    color: [color.x, color.y, color.z, *intensity],
                    params: [0.0, 0.0, 0.0, *shadow_cascade_count as f32],
                }
            }
            LightType::Point { position, color, intensity, radius, falloff } => {
                GPULight {
                    position: [position.x, position.y, position.z, 1.0],
                    direction: [0.0, 0.0, 0.0, 0.0],
                    color: [color.x, color.y, color.z, *intensity],
                    params: [*radius, *falloff, 0.0, 0.0],
                }
            }
            LightType::Spot { position, direction, color, intensity, inner_cone, outer_cone, radius } => {
                GPULight {
                    position: [position.x, position.y, position.z, 2.0],
                    direction: [direction.x, direction.y, direction.z, *inner_cone],
                    color: [color.x, color.y, color.z, *intensity],
                    params: [*radius, 0.0, *outer_cone, 0.0],
                }
            }
            LightType::Area { position, size, color, intensity, texture_id } => {
                GPULight {
                    position: [position.x, position.y, position.z, 3.0],
                    direction: [size.x, size.y, size.z, 0.0],
                    color: [color.x, color.y, color.z, *intensity],
                    params: [texture_id.unwrap_or(0) as f32, 0.0, 0.0, 0.0],
                }
            }
        }
    }
}

/// Environmental lighting configuration with enhanced atmospheric effects
#[derive(Debug, Clone)]
pub struct EnvironmentLighting {
    pub sky_color: Vector3<f32>,
    pub horizon_color: Vector3<f32>,
    pub ground_color: Vector3<f32>,
    pub ambient_intensity: f32,
    pub exposure: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub fog_density: f32,
    pub fog_color: Vector3<f32>,
    pub fog_height_falloff: f32,
    pub sun_disk_size: f32,
    pub sun_disk_intensity: f32,
    pub atmospheric_scattering: bool,
    // Enhanced atmospheric effects
    pub rayleigh_scattering: f32,
    pub mie_scattering: f32,
    pub ozone_absorption: f32,
    pub cloud_coverage: f32,
    pub cloud_density: f32,
    pub cloud_opacity: f32,
    pub star_intensity: f32,
    pub moon_phase: f32,              // 0.0 = new moon, 1.0 = full moon
    pub aurora_intensity: f32,        // Northern lights effect
    pub volumetric_fog: bool,
    pub god_rays: bool,
    pub light_shafts_intensity: f32,
}

impl Default for EnvironmentLighting {
    fn default() -> Self {
        Self {
            sky_color: Vector3::new(0.5, 0.7, 1.0),
            horizon_color: Vector3::new(1.0, 0.9, 0.8),
            ground_color: Vector3::new(0.1, 0.1, 0.1),
            ambient_intensity: 0.2,
            exposure: 1.0,
            contrast: 1.0,
            saturation: 1.0,
            fog_density: 0.001,
            fog_color: Vector3::new(0.8, 0.9, 1.0),
            fog_height_falloff: 0.1,
            sun_disk_size: 0.5,
            sun_disk_intensity: 1.0,
            atmospheric_scattering: true,
            // Enhanced atmospheric defaults
            rayleigh_scattering: 0.8,
            mie_scattering: 0.2,
            ozone_absorption: 0.1,
            cloud_coverage: 0.3,
            cloud_density: 0.5,
            cloud_opacity: 0.7,
            star_intensity: 0.8,
            moon_phase: 0.5,
            aurora_intensity: 0.0,
            volumetric_fog: true,
            god_rays: true,
            light_shafts_intensity: 0.5,
        }
    }
}

/// Shadow configuration for advanced lighting
#[derive(Debug, Clone)]
pub struct ShadowConfig {
    pub enabled: bool,
    pub cascade_count: u32,
    pub cascade_distances: Vec<f32>,
    pub shadow_map_size: u32,
    pub pcf_samples: u32,
    pub bias: f32,
    pub normal_bias: f32,
    pub soft_shadows: bool,
    pub contact_shadows: bool,
}

impl Default for ShadowConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cascade_count: 3,
            cascade_distances: vec![10.0, 50.0, 200.0],
            shadow_map_size: 2048,
            pcf_samples: 16,
            bias: 0.001,
            normal_bias: 0.1,
            soft_shadows: true,
            contact_shadows: false,
        }
    }
}

/// Material definitions for different voxel types
pub struct MaterialLibrary {
    materials: HashMap<MaterialType, PBRMaterial>,
}

impl MaterialLibrary {
    pub fn new() -> Self {
        let mut materials = HashMap::new();

        // Earth materials
        materials.insert(MaterialType::Earth, PBRMaterial {
            albedo: [0.4, 0.3, 0.2, 0.0],
            roughness: 0.9,
            metallic: 0.0,
            normal_strength: 1.2,
            ao_strength: 1.1,
            subsurface: 0.1,
            ..Default::default()
        });

        // Note: Dirt is similar to Earth for now since MaterialType::Dirt doesn't exist
        // We can add it later when the material system is expanded

        // Stone materials
        materials.insert(MaterialType::Stone, PBRMaterial {
            albedo: [0.6, 0.6, 0.65, 0.0],
            roughness: 0.8,
            metallic: 0.0,
            normal_strength: 1.5,
            ao_strength: 1.0,
            ..Default::default()
        });

        // Water materials
        materials.insert(MaterialType::Water, PBRMaterial {
            albedo: [0.1, 0.3, 0.8, 0.7], // Alpha for transparency
            roughness: 0.02,
            metallic: 0.0,
            normal_strength: 0.5,
            subsurface: 0.8,
            specular: 1.0,
            ..Default::default()
        });

        // Grass materials
        materials.insert(MaterialType::Grass, PBRMaterial {
            albedo: [0.2, 0.6, 0.2, 0.0],
            roughness: 0.7,
            metallic: 0.0,
            normal_strength: 0.8,
            ao_strength: 0.9,
            subsurface: 0.3,
            ..Default::default()
        });

        // Sand materials
        materials.insert(MaterialType::Sand, PBRMaterial {
            albedo: [0.8, 0.7, 0.5, 0.0],
            roughness: 0.85,
            metallic: 0.0,
            normal_strength: 0.6,
            ao_strength: 1.0,
            subsurface: 0.05,
            ..Default::default()
        });

        // Wood materials
        materials.insert(MaterialType::Wood, PBRMaterial {
            albedo: [0.4, 0.25, 0.15, 0.0],
            roughness: 0.6,
            metallic: 0.0,
            normal_strength: 1.2,
            ao_strength: 1.1,
            anisotropy: 0.3, // Wood grain effect
            subsurface: 0.2,
            ..Default::default()
        });

        // Crystal materials (glowing)
        materials.insert(MaterialType::Crystal, PBRMaterial {
            albedo: [0.9, 0.9, 1.0, 0.0],
            roughness: 0.1,
            metallic: 0.0,
            normal_strength: 2.0,
            emission: [0.2, 0.3, 0.8], // Blue glow
            clearcoat: 1.0,
            clearcoat_roughness: 0.05,
            specular: 1.0,
            ..Default::default()
        });

        // Lava materials (highly emissive)
        materials.insert(MaterialType::Lava, PBRMaterial {
            albedo: [0.2, 0.1, 0.05, 0.0],
            roughness: 0.9,
            metallic: 0.0,
            normal_strength: 1.5,
            emission: [2.0, 0.8, 0.2], // Bright orange glow
            subsurface: 0.5,
            ..Default::default()
        });

        Self { materials }
    }

    pub fn get_material(&self, material_type: MaterialType) -> PBRMaterial {
        self.materials.get(&material_type).copied().unwrap_or_default()
    }

    pub fn update_material(&mut self, material_type: MaterialType, material: PBRMaterial) {
        self.materials.insert(material_type, material);
    }

    pub fn get_all_materials(&self) -> &HashMap<MaterialType, PBRMaterial> {
        &self.materials
    }
}

/// Main PBR lighting system with enhanced atmospheric effects
pub struct PBRLightingSystem {
    pub material_library: MaterialLibrary,
    pub lights: Vec<Light>,
    pub environment: EnvironmentLighting,
    pub shadow_config: ShadowConfig,
    pub time_of_day: f32, // 0.0 = midnight, 0.5 = noon, 1.0 = midnight
    pub weather_intensity: f32, // 0.0 = clear, 1.0 = storm
    pub global_illumination: bool,
    pub screen_space_reflections: bool,
    pub bloom_enabled: bool,
    pub tone_mapping: ToneMappingMode,
    // Enhanced dynamic lighting features
    pub season: f32,                    // 0.0 = spring, 0.25 = summer, 0.5 = autumn, 0.75 = winter
    pub weather_type: WeatherType,
    pub atmospheric_perspective: bool,
    pub dynamic_exposure: bool,
    pub eye_adaptation_speed: f32,
    pub light_pollution: f32,          // City lights effect on night sky
    pub temporal_noise: f32,           // For realistic lighting flicker
    pub volumetric_light_samples: u32, // Quality of volumetric lighting
}

#[derive(Debug, Clone)]
pub enum ToneMappingMode {
    None,
    Reinhard,
    Filmic,
    ACES,
    Uncharted2,
    AgX,
    Khronos,
}

/// Weather types for atmospheric effects
#[derive(Debug, Clone, PartialEq)]
pub enum WeatherType {
    Clear,
    Cloudy,
    Overcast,
    Rainy,
    Stormy,
    Foggy,
    Snowy,
    Windy,
    Hazy,
}

/// Atmospheric presets for different times and weather
#[derive(Debug, Clone)]
pub struct AtmosphericPreset {
    pub name: String,
    pub time_range: (f32, f32),        // Start and end time (0.0-1.0)
    pub weather_type: WeatherType,
    pub sky_color_day: Vector3<f32>,
    pub sky_color_night: Vector3<f32>,
    pub horizon_color: Vector3<f32>,
    pub fog_density: f32,
    pub cloud_coverage: f32,
    pub sun_intensity_multiplier: f32,
    pub ambient_multiplier: f32,
    pub star_visibility: f32,
}

impl PBRLightingSystem {
    pub fn new() -> Self {
        let mut system = Self {
            material_library: MaterialLibrary::new(),
            lights: Vec::new(),
            environment: EnvironmentLighting::default(),
            shadow_config: ShadowConfig::default(),
            time_of_day: 0.5,
            weather_intensity: 0.0,
            global_illumination: false,
            screen_space_reflections: true,
            bloom_enabled: true,
            tone_mapping: ToneMappingMode::ACES,
            // Enhanced features
            season: 0.25, // Default to summer
            weather_type: WeatherType::Clear,
            atmospheric_perspective: true,
            dynamic_exposure: true,
            eye_adaptation_speed: 2.0,
            light_pollution: 0.0,
            temporal_noise: 0.02,
            volumetric_light_samples: 32,
        };

        // Add default sun and moon lights
        system.lights.push(Light::new_sun());
        system.add_moon_light();
        system
    }

    /// Add a moon light for night illumination
    fn add_moon_light(&mut self) {
        let moon_light = Light {
            light_type: LightType::Directional {
                direction: Vector3::new(0.3, -0.7, 0.5).normalize(),
                color: Vector3::new(0.6, 0.7, 1.0), // Cool moonlight
                intensity: 0.3,
                shadow_cascade_count: 2,
            },
            enabled: false, // Will be enabled at night
            cast_shadows: true,
            volumetric: true,
            temperature: 4100.0, // Cool moon temperature
            ies_profile: None,
        };
        self.lights.push(moon_light);
    }

    /// Update time of day and adjust lighting accordingly with enhanced atmospheric effects
    pub fn update_time_of_day(&mut self, delta_time: f32) {
        self.time_of_day += delta_time * 0.01; // Slow day/night cycle
        if self.time_of_day > 1.0 {
            self.time_of_day -= 1.0;
        }

        // Calculate sun and moon positions
        let sun_angle = (self.time_of_day - 0.5) * std::f32::consts::PI;
        let sun_elevation = -sun_angle.cos();
        let moon_elevation = sun_angle.cos(); // Moon opposite to sun

        // Update sun light (index 0)
        if let Some(sun_light) = self.lights.get_mut(0) {
            self.update_sun_light(sun_light, sun_angle, sun_elevation);
        }

        // Update moon light (index 1)
        if self.lights.len() > 1 {
            if let Some(moon_light) = self.lights.get_mut(1) {
                self.update_moon_light(moon_light, sun_angle, moon_elevation);
            }
        }

        // Update atmospheric effects
        self.update_atmospheric_effects(sun_elevation, moon_elevation);

        // Update weather-dependent effects
        self.update_weather_effects();

        // Update seasonal variations
        self.update_seasonal_effects();
    }

    /// Update sun light properties based on elevation
    fn update_sun_light(&mut self, sun_light: &mut Light, sun_angle: f32, sun_elevation: f32) {
        if let LightType::Directional { direction, color, intensity, .. } = &mut sun_light.light_type {
            // Update sun direction with seasonal variation
            let seasonal_tilt = (self.season * 2.0 * std::f32::consts::PI).sin() * 0.4; // ±23.5° seasonal tilt
            *direction = Vector3::new(
                sun_angle.sin() * 0.3,
                -sun_elevation + seasonal_tilt,
                -0.5
            ).normalize();

            // Enhanced intensity calculation
            *intensity = if sun_elevation > 0.0 {
                let base_intensity = (sun_elevation * 3.5).max(0.0);
                // Atmospheric absorption
                let atmospheric_factor = 1.0 - self.environment.rayleigh_scattering * 0.3;
                // Weather modulation
                let weather_factor = match self.weather_type {
                    WeatherType::Clear => 1.0,
                    WeatherType::Cloudy => 0.7,
                    WeatherType::Overcast => 0.4,
                    WeatherType::Stormy => 0.2,
                    WeatherType::Foggy => 0.3,
                    _ => 0.8,
                };
                base_intensity * atmospheric_factor * weather_factor
            } else {
                0.0
            };

            // Dynamic color temperature
            let temp = self.calculate_sun_temperature(sun_elevation);
            let base_color = Light::temperature_to_rgb(temp);

            // Atmospheric scattering effects
            let scattering_factor = self.calculate_atmospheric_scattering(sun_elevation);
            *color = base_color * scattering_factor;

            // Enable/disable sun based on elevation
            sun_light.enabled = sun_elevation > -0.1;
        }
    }

    /// Update moon light properties
    fn update_moon_light(&mut self, moon_light: &mut Light, sun_angle: f32, moon_elevation: f32) {
        if let LightType::Directional { direction, color, intensity, .. } = &mut moon_light.light_type {
            // Moon direction opposite to sun
            *direction = Vector3::new(
                -sun_angle.sin() * 0.3,
                moon_elevation,
                0.5
            ).normalize();

            // Moon intensity based on phase and elevation
            let phase_factor = (self.environment.moon_phase * 2.0).min(2.0 - self.environment.moon_phase * 2.0);
            *intensity = if moon_elevation > 0.0 {
                0.3 * moon_elevation * phase_factor * (1.0 - self.light_pollution)
            } else {
                0.0
            };

            // Moon color (cooler than sun)
            let moon_temp = 4100.0; // Moon's reflected sunlight
            *color = Light::temperature_to_rgb(moon_temp) * phase_factor;

            // Enable moon at night
            moon_light.enabled = moon_elevation > -0.1 && self.environment.moon_phase > 0.1;
        }
    }

    /// Calculate sun color temperature based on elevation
    fn calculate_sun_temperature(&self, sun_elevation: f32) -> f32 {
        if sun_elevation > 0.3 {
            5778.0 // High noon - pure daylight
        } else if sun_elevation > 0.0 {
            // Sunset/sunrise gradient
            let transition = sun_elevation / 0.3;
            5778.0 - (5778.0 - 2700.0) * (1.0 - transition)
        } else {
            2700.0 // Night
        }
    }

    /// Calculate atmospheric scattering effects
    fn calculate_atmospheric_scattering(&self, sun_elevation: f32) -> Vector3<f32> {
        let base_color = Vector3::new(1.0, 1.0, 1.0);

        if sun_elevation <= 0.0 {
            return base_color * 0.1; // Minimal scattering at night
        }

        // Rayleigh scattering (blue light scattered more)
        let rayleigh = self.environment.rayleigh_scattering;
        let mie = self.environment.mie_scattering;

        // More red/orange when sun is low (more atmosphere to travel through)
        let atmosphere_thickness = 1.0 / (sun_elevation + 0.1);

        let red_transmission = (1.0 - rayleigh * 0.1 * atmosphere_thickness).max(0.1);
        let green_transmission = (1.0 - rayleigh * 0.3 * atmosphere_thickness).max(0.1);
        let blue_transmission = (1.0 - rayleigh * 0.8 * atmosphere_thickness).max(0.1);

        // Mie scattering adds haze
        let mie_factor = 1.0 + mie * 0.5;

        Vector3::new(
            red_transmission * mie_factor,
            green_transmission * mie_factor,
            blue_transmission * mie_factor
        )
    }

    /// Update atmospheric effects based on time and conditions
    fn update_atmospheric_effects(&mut self, sun_elevation: f32, moon_elevation: f32) {
        // Sky color transitions
        let day_factor = sun_elevation.max(0.0);
        let night_factor = moon_elevation.max(0.0) * self.environment.moon_phase;

        // Dynamic sky colors
        if day_factor > 0.1 {
            // Daytime sky
            let clear_sky = Vector3::new(0.4, 0.6, 1.0);
            let sunset_sky = Vector3::new(1.0, 0.5, 0.2);
            let transition = (day_factor - 0.1) / 0.9;
            self.environment.sky_color = clear_sky.lerp(sunset_sky, 1.0 - transition);

            // Horizon color during sunset/sunrise
            if day_factor < 0.5 {
                self.environment.horizon_color = Vector3::new(1.0, 0.7, 0.3) * (1.0 - day_factor * 2.0);
            } else {
                self.environment.horizon_color = Vector3::new(0.9, 0.9, 0.8);
            }
        } else {
            // Nighttime sky
            let night_sky = Vector3::new(0.02, 0.02, 0.08);
            let moonlit_sky = Vector3::new(0.1, 0.15, 0.3);
            self.environment.sky_color = night_sky.lerp(moonlit_sky, night_factor);
            self.environment.horizon_color = Vector3::new(0.05, 0.05, 0.1);
        }

        // Star visibility
        self.environment.star_intensity = if day_factor < 0.1 {
            (1.0 - self.light_pollution) * (1.0 - self.environment.cloud_coverage)
        } else {
            0.0
        };

        // Ambient lighting
        let base_ambient = 0.15;
        let day_ambient = base_ambient + day_factor * 0.4;
        let night_ambient = base_ambient + night_factor * 0.1;
        self.environment.ambient_intensity = day_ambient.max(night_ambient);

        // Fog effects
        self.update_fog_effects(day_factor);
    }

    /// Update fog effects based on conditions
    fn update_fog_effects(&mut self, day_factor: f32) {
        let base_fog = match self.weather_type {
            WeatherType::Clear => 0.001,
            WeatherType::Cloudy => 0.003,
            WeatherType::Overcast => 0.005,
            WeatherType::Foggy => 0.02,
            WeatherType::Rainy => 0.008,
            WeatherType::Stormy => 0.015,
            WeatherType::Snowy => 0.012,
            _ => 0.002,
        };

        // Morning fog is denser
        let time_fog_factor = if self.time_of_day > 0.2 && self.time_of_day < 0.4 {
            2.0 // Morning fog
        } else {
            1.0
        };

        self.environment.fog_density = base_fog * time_fog_factor;

        // Fog color varies with time of day
        if day_factor > 0.1 {
            self.environment.fog_color = Vector3::new(0.8, 0.9, 1.0); // Blue-ish day fog
        } else {
            self.environment.fog_color = Vector3::new(0.3, 0.3, 0.4); // Gray night fog
        }
    }

    /// Update weather-specific effects
    fn update_weather_effects(&mut self) {
        match self.weather_type {
            WeatherType::Stormy => {
                // Lightning effects could be added here
                self.temporal_noise = 0.1; // More dramatic lighting changes
                self.environment.cloud_coverage = 0.9;
                self.environment.cloud_opacity = 0.8;
            }
            WeatherType::Rainy => {
                self.environment.cloud_coverage = 0.7;
                self.environment.saturation = 0.8; // Desaturated in rain
            }
            WeatherType::Foggy => {
                self.environment.fog_density *= 5.0;
                self.environment.contrast = 0.6; // Low contrast in fog
            }
            WeatherType::Clear => {
                self.environment.cloud_coverage = 0.1;
                self.environment.saturation = 1.1; // Slightly more saturated
                self.environment.contrast = 1.1;
            }
            _ => {}
        }
    }

    /// Update seasonal lighting variations
    fn update_seasonal_effects(&mut self) {
        // Seasonal color temperature shifts
        let seasonal_warmth = match self.season {
            s if s < 0.25 => 1.0 + s * 0.2,      // Spring: gradually warmer
            s if s < 0.5 => 1.05 + (s - 0.25) * 0.2, // Summer: warmest
            s if s < 0.75 => 1.1 - (s - 0.5) * 0.3,  // Autumn: cooling, warmer colors
            _ => 0.95 - (self.season - 0.75) * 0.2,  // Winter: coolest
        };

        // Adjust ambient lighting for seasons
        let seasonal_ambient_factor = match self.season {
            s if s < 0.25 => 1.0,     // Spring: normal
            s if s < 0.5 => 1.1,      // Summer: brighter
            s if s < 0.75 => 0.95,    // Autumn: slightly dimmer
            _ => 0.85,                // Winter: much dimmer
        };

        self.environment.ambient_intensity *= seasonal_ambient_factor;
    }

    /// Add a light to the system
    pub fn add_light(&mut self, light: Light) -> usize {
        self.lights.push(light);
        self.lights.len() - 1
    }

    /// Remove a light from the system
    pub fn remove_light(&mut self, index: usize) {
        if index < self.lights.len() {
            self.lights.remove(index);
        }
    }

    /// Get all GPU-compatible light data
    pub fn get_gpu_lights(&self) -> Vec<GPULight> {
        self.lights.iter()
            .filter(|light| light.enabled)
            .map(|light| light.to_gpu_light())
            .collect()
    }

    /// Set weather type and intensity
    pub fn set_weather(&mut self, weather_type: WeatherType, intensity: f32) {
        self.weather_type = weather_type;
        self.weather_intensity = intensity.clamp(0.0, 1.0);
    }

    /// Set seasonal variation (0.0 = spring, 0.25 = summer, 0.5 = autumn, 0.75 = winter)
    pub fn set_season(&mut self, season: f32) {
        self.season = season % 1.0;
    }

    /// Set moon phase (0.0 = new moon, 1.0 = full moon)
    pub fn set_moon_phase(&mut self, phase: f32) {
        self.environment.moon_phase = phase.clamp(0.0, 1.0);
    }

    /// Set light pollution level (0.0 = no pollution, 1.0 = heavy urban)
    pub fn set_light_pollution(&mut self, pollution: f32) {
        self.light_pollution = pollution.clamp(0.0, 1.0);
    }

    /// Enable/disable aurora effects
    pub fn set_aurora_intensity(&mut self, intensity: f32) {
        self.environment.aurora_intensity = intensity.clamp(0.0, 1.0);
    }

    /// Get current atmospheric preset based on time and weather
    pub fn get_current_atmospheric_preset(&self) -> AtmosphericPreset {
        let time_period = if self.time_of_day < 0.25 {
            "Dawn"
        } else if self.time_of_day < 0.75 {
            "Day"
        } else {
            "Dusk"
        };

        AtmosphericPreset {
            name: format!("{} - {:?}", time_period, self.weather_type),
            time_range: (self.time_of_day, self.time_of_day),
            weather_type: self.weather_type.clone(),
            sky_color_day: self.environment.sky_color,
            sky_color_night: Vector3::new(0.02, 0.02, 0.08),
            horizon_color: self.environment.horizon_color,
            fog_density: self.environment.fog_density,
            cloud_coverage: self.environment.cloud_coverage,
            sun_intensity_multiplier: 1.0,
            ambient_multiplier: self.environment.ambient_intensity / 0.2,
            star_visibility: self.environment.star_intensity,
        }
    }

    /// Get material for a voxel type
    pub fn get_material(&self, material_type: MaterialType) -> PBRMaterial {
        self.material_library.get_material(material_type)
    }

    /// Update a material
    pub fn update_material(&mut self, material_type: MaterialType, material: PBRMaterial) {
        self.material_library.update_material(material_type, material);
    }

    /// Calculate lighting contribution for debugging
    pub fn calculate_lighting_debug(&self, world_pos: Point3<f32>, normal: Vector3<f32>) -> Vector3<f32> {
        let mut total_light = self.environment.sky_color * self.environment.ambient_intensity;

        for light in &self.lights {
            if !light.enabled { continue; }

            match &light.light_type {
                LightType::Directional { direction, color, intensity, .. } => {
                    let ndotl = normal.dot(-*direction).max(0.0);
                    total_light += *color * *intensity * ndotl;
                }
                LightType::Point { position, color, intensity, radius, .. } => {
                    let light_dir = *position - world_pos;
                    let distance = light_dir.magnitude();

                    if distance < *radius {
                        let light_dir = light_dir / distance;
                        let ndotl = normal.dot(light_dir).max(0.0);
                        let attenuation = 1.0 - (distance / *radius).powf(2.0);
                        total_light += *color * *intensity * ndotl * attenuation;
                    }
                }
                _ => {} // TODO: Implement other light types
            }
        }

        total_light
    }

    /// Get GPU-compatible environment data for shaders
    pub fn get_environment_uniforms(&self) -> EnvironmentUniforms {
        EnvironmentUniforms {
            sky_color: [self.environment.sky_color.x, self.environment.sky_color.y, self.environment.sky_color.z, 1.0],
            horizon_color: [self.environment.horizon_color.x, self.environment.horizon_color.y, self.environment.horizon_color.z, 1.0],
            fog_color: [self.environment.fog_color.x, self.environment.fog_color.y, self.environment.fog_color.z, self.environment.fog_density],
            atmospheric_params: [self.environment.rayleigh_scattering, self.environment.mie_scattering, self.environment.ozone_absorption, self.time_of_day],
            weather_params: [self.environment.cloud_coverage, self.environment.cloud_density, self.weather_intensity, self.season],
            night_params: [self.environment.star_intensity, self.environment.moon_phase, self.environment.aurora_intensity, self.light_pollution],
            post_process_params: [self.environment.exposure, self.environment.contrast, self.environment.saturation, self.eye_adaptation_speed],
        }
    }

    /// Quick preset for different times of day
    pub fn apply_time_preset(&mut self, preset: TimePreset) {
        match preset {
            TimePreset::EarlyMorning => {
                self.time_of_day = 0.25;
                self.weather_type = WeatherType::Clear;
                self.environment.fog_density = 0.008; // Morning mist
            }
            TimePreset::Noon => {
                self.time_of_day = 0.5;
                self.weather_type = WeatherType::Clear;
                self.environment.fog_density = 0.001;
            }
            TimePreset::GoldenHour => {
                self.time_of_day = 0.75;
                self.weather_type = WeatherType::Clear;
                self.environment.fog_density = 0.003;
            }
            TimePreset::Midnight => {
                self.time_of_day = 0.0;
                self.weather_type = WeatherType::Clear;
                self.environment.moon_phase = 1.0; // Full moon
                self.environment.star_intensity = 0.9;
            }
            TimePreset::StormyDay => {
                self.time_of_day = 0.45;
                self.weather_type = WeatherType::Stormy;
                self.weather_intensity = 0.8;
            }
        }
    }

    /// Get performance metrics for the lighting system
    pub fn get_performance_info(&self) -> LightingPerformanceInfo {
        LightingPerformanceInfo {
            active_lights: self.lights.iter().filter(|l| l.enabled).count(),
            shadow_casting_lights: self.lights.iter().filter(|l| l.enabled && l.cast_shadows).count(),
            volumetric_lights: self.lights.iter().filter(|l| l.enabled && l.volumetric).count(),
            volumetric_samples: self.volumetric_light_samples,
            global_illumination_enabled: self.global_illumination,
            atmospheric_scattering_enabled: self.environment.atmospheric_scattering,
            weather_effects_active: self.weather_type != WeatherType::Clear,
        }
    }
}

/// GPU-compatible environment uniforms
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct EnvironmentUniforms {
    pub sky_color: [f32; 4],
    pub horizon_color: [f32; 4],
    pub fog_color: [f32; 4],             // w = fog_density
    pub atmospheric_params: [f32; 4],    // rayleigh, mie, ozone, time_of_day
    pub weather_params: [f32; 4],        // cloud_coverage, cloud_density, weather_intensity, season
    pub night_params: [f32; 4],          // star_intensity, moon_phase, aurora_intensity, light_pollution
    pub post_process_params: [f32; 4],   // exposure, contrast, saturation, eye_adaptation_speed
}

/// Time of day presets for quick setup
#[derive(Debug, Clone, Copy)]
pub enum TimePreset {
    EarlyMorning,
    Noon,
    GoldenHour,
    Midnight,
    StormyDay,
}

/// Performance information for the lighting system
#[derive(Debug, Clone)]
pub struct LightingPerformanceInfo {
    pub active_lights: usize,
    pub shadow_casting_lights: usize,
    pub volumetric_lights: usize,
    pub volumetric_samples: u32,
    pub global_illumination_enabled: bool,
    pub atmospheric_scattering_enabled: bool,
    pub weather_effects_active: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_library() {
        let library = MaterialLibrary::new();
        let stone = library.get_material(MaterialType::Stone);
        assert!(stone.metallic < 0.1); // Stone should not be metallic
        assert!(stone.roughness > 0.5); // Stone should be rough
    }

    #[test]
    fn test_light_creation() {
        let sun = Light::new_sun();
        assert!(sun.enabled);
        assert!(sun.cast_shadows);

        let torch = Light::new_torch();
        assert_eq!(torch.temperature, 1900.0);
    }

    #[test]
    fn test_temperature_conversion() {
        let daylight = Light::temperature_to_rgb(5778.0);
        assert!(daylight.x > 0.8); // Should be mostly white
        assert!(daylight.y > 0.8);
        assert!(daylight.z > 0.8);

        let candle = Light::temperature_to_rgb(1900.0);
        assert!(candle.x > candle.z); // Should be more red than blue
    }

    #[test]
    fn test_time_of_day() {
        let mut pbr_system = PBRLightingSystem::new();
        pbr_system.time_of_day = 0.0; // Midnight
        pbr_system.update_time_of_day(0.0);

        // Sun should be pointing down at midnight
        if let LightType::Directional { direction, .. } = &pbr_system.lights[0].light_type {
            assert!(direction.y < 0.0);
        }
    }

    #[test]
    fn test_gpu_light_conversion() {
        let point_light = Light {
            light_type: LightType::Point {
                position: Point3::new(1.0, 2.0, 3.0),
                color: Vector3::new(1.0, 0.5, 0.0),
                intensity: 2.0,
                radius: 10.0,
                falloff: 1.5,
            },
            enabled: true,
            cast_shadows: true,
            volumetric: false,
            temperature: 3000.0,
            ies_profile: None,
        };

        let gpu_light = point_light.to_gpu_light();
        assert_eq!(gpu_light.position[0], 1.0);
        assert_eq!(gpu_light.position[1], 2.0);
        assert_eq!(gpu_light.position[2], 3.0);
        assert_eq!(gpu_light.position[3], 1.0); // Point light type
        assert_eq!(gpu_light.color[3], 2.0); // Intensity
    }

    #[test]
    fn test_atmospheric_effects() {
        let mut pbr = PBRLightingSystem::new();
        pbr.set_weather(WeatherType::Stormy, 0.8);
        pbr.set_season(0.75); // Winter
        pbr.set_moon_phase(0.0); // New moon

        pbr.update_time_of_day(0.0); // Update once

        assert_eq!(pbr.weather_type, WeatherType::Stormy);
        assert_eq!(pbr.season, 0.75);
        assert_eq!(pbr.environment.moon_phase, 0.0);
    }

    #[test]
    fn test_time_presets() {
        let mut pbr = PBRLightingSystem::new();
        pbr.apply_time_preset(TimePreset::Midnight);

        assert_eq!(pbr.time_of_day, 0.0);
        assert_eq!(pbr.environment.moon_phase, 1.0);
        assert!(pbr.environment.star_intensity > 0.5);
    }

    #[test]
    fn test_vector_lerp() {
        let v1 = Vector3::new(0.0, 0.0, 0.0);
        let v2 = Vector3::new(1.0, 1.0, 1.0);
        let lerped = v1.lerp(v2, 0.5);

        assert!((lerped.x - 0.5).abs() < 0.001);
        assert!((lerped.y - 0.5).abs() < 0.001);
        assert!((lerped.z - 0.5).abs() < 0.001);
    }
}