/// Visual Effects Gallery
///
/// Stunning visual demonstrations of Robin Engine's rendering capabilities

use std::time::{Duration, Instant};
use cgmath::{Vector3, Vector4, Matrix4, Quaternion, Rad, InnerSpace};
use crate::engine::{
    generation::voxel_system::{VoxelWorld, VoxelType},
    graphics::{Material, Shader, Texture, RenderPass},
    animation::{Animation, AnimationClip, Keyframe},
};

/// Main Visual Showcase system
pub struct VisualShowcase {
    // Showcase scenes
    lighting_gallery: LightingGallery,
    material_showcase: MaterialShowcase,
    weather_system: WeatherEffects,
    post_processing: PostProcessingDemo,
    water_simulation: WaterSimulation,
    particle_gallery: ParticleGallery,

    // Current state
    active_scene: VisualScene,
    transition_progress: f32,
    time_of_day: f32,
    weather_intensity: f32,

    // Performance metrics
    frame_time: f32,
    draw_calls: u32,
    triangles_rendered: u32,
}

/// Visual scene types
#[derive(Debug, Clone, PartialEq)]
pub enum VisualScene {
    LightingGallery,
    MaterialShowcase,
    WeatherEffects,
    PostProcessing,
    WaterSimulation,
    ParticleGallery,
}

/// Lighting Gallery - Dynamic lighting demonstrations
pub struct LightingGallery {
    // Scene elements
    world: VoxelWorld,
    light_sources: Vec<LightSource>,
    shadow_casters: Vec<ShadowCaster>,

    // Day/night cycle
    sun_position: Vector3<f32>,
    moon_position: Vector3<f32>,
    ambient_color: Vector4<f32>,
    fog_density: f32,

    // Dynamic lights
    torches: Vec<Torch>,
    fireflies: Vec<Firefly>,
    lightning_effects: Vec<Lightning>,

    // Settings
    enable_shadows: bool,
    shadow_quality: ShadowQuality,
    enable_global_illumination: bool,
    enable_volumetric_lighting: bool,
}

/// Light source in the scene
#[derive(Clone)]
pub struct LightSource {
    pub position: Vector3<f32>,
    pub color: Vector4<f32>,
    pub intensity: f32,
    pub radius: f32,
    pub light_type: LightType,
    pub cast_shadows: bool,
}

/// Light types
#[derive(Debug, Clone, PartialEq)]
pub enum LightType {
    Directional,
    Point,
    Spot { angle: f32, direction: Vector3<f32> },
    Area { width: f32, height: f32 },
}

/// Shadow caster object
pub struct ShadowCaster {
    pub position: Vector3<f32>,
    pub bounds: Vector3<f32>,
    pub opacity: f32,
}

/// Shadow quality settings
#[derive(Debug, Clone, PartialEq)]
pub enum ShadowQuality {
    Low,    // 1024x1024
    Medium, // 2048x2048
    High,   // 4096x4096
    Ultra,  // 8192x8192
}

/// Torch with flickering light
pub struct Torch {
    pub position: Vector3<f32>,
    pub base_intensity: f32,
    pub flicker_speed: f32,
    pub flicker_amount: f32,
    pub color: Vector4<f32>,
    pub particle_emitter: Option<ParticleEmitter>,
}

/// Firefly with moving light
pub struct Firefly {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub color: Vector4<f32>,
    pub intensity: f32,
    pub blink_pattern: Vec<f32>,
    pub blink_index: usize,
}

/// Lightning effect
pub struct Lightning {
    pub start_pos: Vector3<f32>,
    pub end_pos: Vector3<f32>,
    pub branches: Vec<LightningBranch>,
    pub intensity: f32,
    pub duration: Duration,
    pub created_at: Instant,
}

/// Lightning branch
pub struct LightningBranch {
    pub points: Vec<Vector3<f32>>,
    pub width: f32,
    pub intensity: f32,
}

/// Material Showcase - PBR materials demonstration
pub struct MaterialShowcase {
    // Display platforms
    platforms: Vec<DisplayPlatform>,

    // Material samples
    materials: Vec<MaterialSample>,

    // Environment
    environment_map: Option<Texture>,
    exposure: f32,

    // Interactive controls
    roughness_override: Option<f32>,
    metallic_override: Option<f32>,
    rotation_speed: f32,
}

/// Display platform for materials
pub struct DisplayPlatform {
    pub position: Vector3<f32>,
    pub size: f32,
    pub material_index: usize,
    pub rotation: Quaternion<f32>,
    pub spotlight: LightSource,
}

/// Material sample
pub struct MaterialSample {
    pub name: String,
    pub base_color: Vector4<f32>,
    pub roughness: f32,
    pub metallic: f32,
    pub emissive: Vector3<f32>,
    pub normal_map: Option<Texture>,
    pub ao_map: Option<Texture>,
    pub category: MaterialCategory,
}

/// Material categories
#[derive(Debug, Clone, PartialEq)]
pub enum MaterialCategory {
    Metals,
    Dielectrics,
    Organic,
    Synthetic,
    Special,
}

/// Weather Effects system
pub struct WeatherEffects {
    // Weather states
    current_weather: WeatherType,
    next_weather: Option<WeatherType>,
    transition_progress: f32,

    // Precipitation
    rain_particles: Vec<RainParticle>,
    snow_particles: Vec<SnowParticle>,
    rain_intensity: f32,
    snow_intensity: f32,

    // Atmospheric effects
    fog_density: f32,
    fog_color: Vector4<f32>,
    wind_direction: Vector3<f32>,
    wind_strength: f32,

    // Cloud system
    clouds: Vec<Cloud>,
    cloud_coverage: f32,

    // Thunder and lightning
    thunder_cooldown: f32,
    lightning_probability: f32,
}

/// Weather types
#[derive(Debug, Clone, PartialEq)]
pub enum WeatherType {
    Clear,
    Cloudy,
    Rain,
    Storm,
    Snow,
    Fog,
}

/// Rain particle
pub struct RainParticle {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub length: f32,
    pub opacity: f32,
    pub splash_on_impact: bool,
}

/// Snow particle
pub struct SnowParticle {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub size: f32,
    pub rotation: f32,
    pub sway_offset: f32,
}

/// Cloud in the sky
pub struct Cloud {
    pub position: Vector3<f32>,
    pub size: Vector3<f32>,
    pub density: f32,
    pub velocity: Vector3<f32>,
    pub cloud_type: CloudType,
}

/// Cloud types
#[derive(Debug, Clone, PartialEq)]
pub enum CloudType {
    Cumulus,
    Stratus,
    Cirrus,
    Cumulonimbus,
}

/// Post-Processing effects demo
pub struct PostProcessingDemo {
    // Effect toggles
    bloom_enabled: bool,
    depth_of_field_enabled: bool,
    motion_blur_enabled: bool,
    chromatic_aberration_enabled: bool,
    vignette_enabled: bool,
    film_grain_enabled: bool,
    tone_mapping_enabled: bool,

    // Bloom settings
    bloom_threshold: f32,
    bloom_intensity: f32,
    bloom_radius: f32,

    // Depth of field settings
    focal_distance: f32,
    focal_range: f32,
    bokeh_strength: f32,

    // Motion blur settings
    motion_blur_samples: u32,
    motion_blur_strength: f32,

    // Color grading
    exposure: f32,
    contrast: f32,
    saturation: f32,
    temperature: f32,
    tint: f32,

    // Film effects
    film_grain_intensity: f32,
    vignette_intensity: f32,
    chromatic_aberration_strength: f32,
}

/// Water Simulation system
pub struct WaterSimulation {
    // Water bodies
    water_surfaces: Vec<WaterSurface>,

    // Wave simulation
    wave_amplitude: f32,
    wave_frequency: f32,
    wave_speed: f32,
    wave_direction: Vector2<f32>,

    // Rendering settings
    water_color: Vector4<f32>,
    water_depth_color: Vector4<f32>,
    transparency: f32,
    reflection_strength: f32,
    refraction_strength: f32,

    // Caustics
    caustics_enabled: bool,
    caustics_intensity: f32,
    caustics_scale: f32,

    // Foam and splash
    foam_particles: Vec<FoamParticle>,
    splash_particles: Vec<SplashParticle>,

    // Interaction
    ripple_sources: Vec<RippleSource>,
}

/// Water surface
pub struct WaterSurface {
    pub position: Vector3<f32>,
    pub size: Vector2<f32>,
    pub depth: f32,
    pub flow_direction: Option<Vector2<f32>>,
    pub turbulence: f32,
}

/// Foam particle
pub struct FoamParticle {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub size: f32,
    pub lifetime: f32,
    pub opacity: f32,
}

/// Splash particle
pub struct SplashParticle {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub size: f32,
    pub gravity_scale: f32,
}

/// Ripple source
pub struct RippleSource {
    pub position: Vector2<f32>,
    pub amplitude: f32,
    pub frequency: f32,
    pub decay_rate: f32,
    pub created_at: Instant,
}

/// Particle Gallery
pub struct ParticleGallery {
    // Particle systems
    fire_system: ParticleSystem,
    smoke_system: ParticleSystem,
    magic_system: ParticleSystem,
    explosion_system: ParticleSystem,
    ambient_particles: ParticleSystem,

    // Emitters
    emitters: Vec<ParticleEmitter>,

    // Settings
    max_particles: usize,
    physics_enabled: bool,
    collision_enabled: bool,
}

/// Particle system
pub struct ParticleSystem {
    pub name: String,
    pub particles: Vec<Particle>,
    pub emitter_settings: EmitterSettings,
    pub behavior: ParticleBehavior,
    pub rendering: ParticleRendering,
}

/// Individual particle
pub struct Particle {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub acceleration: Vector3<f32>,
    pub size: f32,
    pub rotation: f32,
    pub color: Vector4<f32>,
    pub lifetime: f32,
    pub age: f32,
}

/// Particle emitter
pub struct ParticleEmitter {
    pub position: Vector3<f32>,
    pub direction: Vector3<f32>,
    pub spread_angle: f32,
    pub emission_rate: f32,
    pub particle_lifetime: f32,
    pub initial_velocity: f32,
    pub velocity_variance: f32,
    pub size_over_lifetime: Option<AnimationCurve>,
    pub color_over_lifetime: Option<ColorGradient>,
}

/// Emitter settings
pub struct EmitterSettings {
    pub emission_shape: EmissionShape,
    pub emission_rate: f32,
    pub burst_count: Option<u32>,
    pub lifetime: f32,
    pub loop_emission: bool,
}

/// Emission shapes
#[derive(Debug, Clone, PartialEq)]
pub enum EmissionShape {
    Point,
    Sphere { radius: f32 },
    Cone { angle: f32, radius: f32 },
    Box { size: Vector3<f32> },
    Mesh { vertices: Vec<Vector3<f32>> },
}

/// Particle behavior settings
pub struct ParticleBehavior {
    pub gravity: Vector3<f32>,
    pub drag: f32,
    pub turbulence: f32,
    pub collision_response: CollisionResponse,
    pub attraction_points: Vec<Vector3<f32>>,
}

/// Collision response types
#[derive(Debug, Clone, PartialEq)]
pub enum CollisionResponse {
    None,
    Bounce { restitution: f32 },
    Stick,
    Die,
}

/// Particle rendering settings
pub struct ParticleRendering {
    pub blend_mode: BlendMode,
    pub texture: Option<Texture>,
    pub billboard: bool,
    pub soft_particles: bool,
    pub depth_write: bool,
}

/// Blend modes
#[derive(Debug, Clone, PartialEq)]
pub enum BlendMode {
    Additive,
    Alpha,
    Multiply,
    Screen,
}

/// Animation curve for values over time
pub struct AnimationCurve {
    pub keyframes: Vec<(f32, f32)>,
    pub interpolation: Interpolation,
}

/// Interpolation types
#[derive(Debug, Clone, PartialEq)]
pub enum Interpolation {
    Linear,
    Smooth,
    Step,
}

/// Color gradient over time
pub struct ColorGradient {
    pub colors: Vec<(f32, Vector4<f32>)>,
}

impl VisualShowcase {
    pub fn new() -> Self {
        Self {
            lighting_gallery: Self::create_lighting_gallery(),
            material_showcase: Self::create_material_showcase(),
            weather_system: Self::create_weather_system(),
            post_processing: Self::create_post_processing(),
            water_simulation: Self::create_water_simulation(),
            particle_gallery: Self::create_particle_gallery(),

            active_scene: VisualScene::LightingGallery,
            transition_progress: 0.0,
            time_of_day: 12.0,
            weather_intensity: 0.5,

            frame_time: 16.0,
            draw_calls: 0,
            triangles_rendered: 0,
        }
    }

    /// Create lighting gallery
    fn create_lighting_gallery() -> LightingGallery {
        let mut world = VoxelWorld::new("Lighting Gallery".to_string(), (100, 50, 100));

        // Create scene geometry
        Self::build_lighting_scene(&mut world);

        // Create light sources
        let light_sources = vec![
            LightSource {
                position: Vector3::new(0.0, 30.0, 0.0),
                color: Vector4::new(1.0, 0.95, 0.8, 1.0),
                intensity: 1.0,
                radius: 100.0,
                light_type: LightType::Directional,
                cast_shadows: true,
            },
        ];

        // Create torches
        let torches = vec![
            Torch {
                position: Vector3::new(-10.0, 5.0, -10.0),
                base_intensity: 0.8,
                flicker_speed: 3.0,
                flicker_amount: 0.2,
                color: Vector4::new(1.0, 0.6, 0.2, 1.0),
                particle_emitter: None,
            },
            Torch {
                position: Vector3::new(10.0, 5.0, -10.0),
                base_intensity: 0.8,
                flicker_speed: 2.5,
                flicker_amount: 0.15,
                color: Vector4::new(1.0, 0.6, 0.2, 1.0),
                particle_emitter: None,
            },
        ];

        LightingGallery {
            world,
            light_sources,
            shadow_casters: Vec::new(),
            sun_position: Vector3::new(50.0, 100.0, 50.0),
            moon_position: Vector3::new(-50.0, 80.0, -50.0),
            ambient_color: Vector4::new(0.2, 0.2, 0.3, 1.0),
            fog_density: 0.02,
            torches,
            fireflies: Vec::new(),
            lightning_effects: Vec::new(),
            enable_shadows: true,
            shadow_quality: ShadowQuality::High,
            enable_global_illumination: true,
            enable_volumetric_lighting: true,
        }
    }

    /// Build lighting scene geometry
    fn build_lighting_scene(world: &mut VoxelWorld) {
        // Create floor
        for x in -20..20 {
            for z in -20..20 {
                world.set_voxel(
                    Vector3::new(x as f32, 0.0, z as f32),
                    VoxelType::Stone,
                );
            }
        }

        // Create pillars for shadow testing
        for x in [-10, 0, 10] {
            for z in [-10, 0, 10] {
                for y in 1..8 {
                    world.set_voxel(
                        Vector3::new(x as f32, y as f32, z as f32),
                        VoxelType::Concrete,
                    );
                }
            }
        }

        // Create glass roof sections
        for x in -15..15 {
            for z in -15..15 {
                if (x + z) % 4 == 0 {
                    world.set_voxel(
                        Vector3::new(x as f32, 10.0, z as f32),
                        VoxelType::Glass,
                    );
                }
            }
        }
    }

    /// Create material showcase
    fn create_material_showcase() -> MaterialShowcase {
        let materials = vec![
            MaterialSample {
                name: "Polished Gold".to_string(),
                base_color: Vector4::new(1.0, 0.766, 0.336, 1.0),
                roughness: 0.1,
                metallic: 1.0,
                emissive: Vector3::new(0.0, 0.0, 0.0),
                normal_map: None,
                ao_map: None,
                category: MaterialCategory::Metals,
            },
            MaterialSample {
                name: "Rough Iron".to_string(),
                base_color: Vector4::new(0.56, 0.57, 0.58, 1.0),
                roughness: 0.7,
                metallic: 1.0,
                emissive: Vector3::new(0.0, 0.0, 0.0),
                normal_map: None,
                ao_map: None,
                category: MaterialCategory::Metals,
            },
            MaterialSample {
                name: "Glass".to_string(),
                base_color: Vector4::new(0.9, 0.95, 1.0, 0.1),
                roughness: 0.0,
                metallic: 0.0,
                emissive: Vector3::new(0.0, 0.0, 0.0),
                normal_map: None,
                ao_map: None,
                category: MaterialCategory::Dielectrics,
            },
            MaterialSample {
                name: "Emissive Crystal".to_string(),
                base_color: Vector4::new(0.2, 0.5, 1.0, 1.0),
                roughness: 0.2,
                metallic: 0.0,
                emissive: Vector3::new(0.2, 0.5, 1.0),
                normal_map: None,
                ao_map: None,
                category: MaterialCategory::Special,
            },
        ];

        let mut platforms = Vec::new();
        for (i, _) in materials.iter().enumerate() {
            let angle = (i as f32 / materials.len() as f32) * std::f32::consts::PI * 2.0;
            let radius = 10.0;

            platforms.push(DisplayPlatform {
                position: Vector3::new(angle.cos() * radius, 2.0, angle.sin() * radius),
                size: 2.0,
                material_index: i,
                rotation: Quaternion::from_axis_angle(Vector3::unit_y(), Rad(0.0)),
                spotlight: LightSource {
                    position: Vector3::new(angle.cos() * radius, 8.0, angle.sin() * radius),
                    color: Vector4::new(1.0, 1.0, 1.0, 1.0),
                    intensity: 2.0,
                    radius: 10.0,
                    light_type: LightType::Spot {
                        angle: 30.0,
                        direction: Vector3::new(0.0, -1.0, 0.0),
                    },
                    cast_shadows: true,
                },
            });
        }

        MaterialShowcase {
            platforms,
            materials,
            environment_map: None,
            exposure: 1.0,
            roughness_override: None,
            metallic_override: None,
            rotation_speed: 0.5,
        }
    }

    /// Create weather system
    fn create_weather_system() -> WeatherEffects {
        WeatherEffects {
            current_weather: WeatherType::Clear,
            next_weather: None,
            transition_progress: 0.0,
            rain_particles: Vec::new(),
            snow_particles: Vec::new(),
            rain_intensity: 0.0,
            snow_intensity: 0.0,
            fog_density: 0.01,
            fog_color: Vector4::new(0.7, 0.7, 0.8, 1.0),
            wind_direction: Vector3::new(1.0, 0.0, 0.5).normalize(),
            wind_strength: 5.0,
            clouds: Self::generate_clouds(),
            cloud_coverage: 0.3,
            thunder_cooldown: 0.0,
            lightning_probability: 0.01,
        }
    }

    /// Generate clouds
    fn generate_clouds() -> Vec<Cloud> {
        let mut clouds = Vec::new();

        for i in 0..10 {
            clouds.push(Cloud {
                position: Vector3::new(
                    (i as f32 - 5.0) * 20.0,
                    50.0 + (i as f32 * 2.0),
                    (i as f32 - 5.0) * 10.0,
                ),
                size: Vector3::new(15.0, 5.0, 10.0),
                density: 0.5 + (i as f32 * 0.05),
                velocity: Vector3::new(2.0, 0.0, 1.0),
                cloud_type: match i % 4 {
                    0 => CloudType::Cumulus,
                    1 => CloudType::Stratus,
                    2 => CloudType::Cirrus,
                    _ => CloudType::Cumulonimbus,
                },
            });
        }

        clouds
    }

    /// Create post-processing demo
    fn create_post_processing() -> PostProcessingDemo {
        PostProcessingDemo {
            bloom_enabled: true,
            depth_of_field_enabled: false,
            motion_blur_enabled: false,
            chromatic_aberration_enabled: false,
            vignette_enabled: true,
            film_grain_enabled: false,
            tone_mapping_enabled: true,

            bloom_threshold: 0.8,
            bloom_intensity: 1.0,
            bloom_radius: 4.0,

            focal_distance: 10.0,
            focal_range: 5.0,
            bokeh_strength: 0.5,

            motion_blur_samples: 8,
            motion_blur_strength: 0.5,

            exposure: 1.0,
            contrast: 1.0,
            saturation: 1.0,
            temperature: 0.0,
            tint: 0.0,

            film_grain_intensity: 0.05,
            vignette_intensity: 0.3,
            chromatic_aberration_strength: 0.002,
        }
    }

    /// Create water simulation
    fn create_water_simulation() -> WaterSimulation {
        WaterSimulation {
            water_surfaces: vec![
                WaterSurface {
                    position: Vector3::new(0.0, 5.0, 0.0),
                    size: Vector2::new(30.0, 30.0),
                    depth: 10.0,
                    flow_direction: Some(Vector2::new(1.0, 0.0)),
                    turbulence: 0.2,
                },
            ],

            wave_amplitude: 0.5,
            wave_frequency: 0.1,
            wave_speed: 2.0,
            wave_direction: Vector2::new(1.0, 0.5).normalize(),

            water_color: Vector4::new(0.0, 0.3, 0.5, 0.8),
            water_depth_color: Vector4::new(0.0, 0.1, 0.2, 1.0),
            transparency: 0.7,
            reflection_strength: 0.5,
            refraction_strength: 0.3,

            caustics_enabled: true,
            caustics_intensity: 0.8,
            caustics_scale: 2.0,

            foam_particles: Vec::new(),
            splash_particles: Vec::new(),
            ripple_sources: Vec::new(),
        }
    }

    /// Create particle gallery
    fn create_particle_gallery() -> ParticleGallery {
        ParticleGallery {
            fire_system: Self::create_fire_system(),
            smoke_system: Self::create_smoke_system(),
            magic_system: Self::create_magic_system(),
            explosion_system: Self::create_explosion_system(),
            ambient_particles: Self::create_ambient_system(),

            emitters: Vec::new(),
            max_particles: 10000,
            physics_enabled: true,
            collision_enabled: true,
        }
    }

    /// Create fire particle system
    fn create_fire_system() -> ParticleSystem {
        ParticleSystem {
            name: "Fire".to_string(),
            particles: Vec::new(),
            emitter_settings: EmitterSettings {
                emission_shape: EmissionShape::Cone {
                    angle: 30.0,
                    radius: 1.0,
                },
                emission_rate: 50.0,
                burst_count: None,
                lifetime: 2.0,
                loop_emission: true,
            },
            behavior: ParticleBehavior {
                gravity: Vector3::new(0.0, 2.0, 0.0), // Fire rises
                drag: 0.1,
                turbulence: 0.5,
                collision_response: CollisionResponse::Die,
                attraction_points: Vec::new(),
            },
            rendering: ParticleRendering {
                blend_mode: BlendMode::Additive,
                texture: None,
                billboard: true,
                soft_particles: true,
                depth_write: false,
            },
        }
    }

    /// Create smoke particle system
    fn create_smoke_system() -> ParticleSystem {
        ParticleSystem {
            name: "Smoke".to_string(),
            particles: Vec::new(),
            emitter_settings: EmitterSettings {
                emission_shape: EmissionShape::Sphere { radius: 0.5 },
                emission_rate: 20.0,
                burst_count: None,
                lifetime: 5.0,
                loop_emission: true,
            },
            behavior: ParticleBehavior {
                gravity: Vector3::new(0.0, 0.5, 0.0),
                drag: 0.3,
                turbulence: 0.2,
                collision_response: CollisionResponse::None,
                attraction_points: Vec::new(),
            },
            rendering: ParticleRendering {
                blend_mode: BlendMode::Alpha,
                texture: None,
                billboard: true,
                soft_particles: true,
                depth_write: false,
            },
        }
    }

    /// Create magic particle system
    fn create_magic_system() -> ParticleSystem {
        ParticleSystem {
            name: "Magic".to_string(),
            particles: Vec::new(),
            emitter_settings: EmitterSettings {
                emission_shape: EmissionShape::Point,
                emission_rate: 30.0,
                burst_count: None,
                lifetime: 3.0,
                loop_emission: true,
            },
            behavior: ParticleBehavior {
                gravity: Vector3::new(0.0, -0.1, 0.0),
                drag: 0.05,
                turbulence: 1.0,
                collision_response: CollisionResponse::Bounce { restitution: 0.8 },
                attraction_points: vec![Vector3::new(0.0, 5.0, 0.0)],
            },
            rendering: ParticleRendering {
                blend_mode: BlendMode::Additive,
                texture: None,
                billboard: true,
                soft_particles: true,
                depth_write: false,
            },
        }
    }

    /// Create explosion particle system
    fn create_explosion_system() -> ParticleSystem {
        ParticleSystem {
            name: "Explosion".to_string(),
            particles: Vec::new(),
            emitter_settings: EmitterSettings {
                emission_shape: EmissionShape::Sphere { radius: 0.1 },
                emission_rate: 0.0,
                burst_count: Some(100),
                lifetime: 1.0,
                loop_emission: false,
            },
            behavior: ParticleBehavior {
                gravity: Vector3::new(0.0, -9.81, 0.0),
                drag: 0.2,
                turbulence: 0.0,
                collision_response: CollisionResponse::Bounce { restitution: 0.5 },
                attraction_points: Vec::new(),
            },
            rendering: ParticleRendering {
                blend_mode: BlendMode::Additive,
                texture: None,
                billboard: true,
                soft_particles: false,
                depth_write: false,
            },
        }
    }

    /// Create ambient particle system
    fn create_ambient_system() -> ParticleSystem {
        ParticleSystem {
            name: "Ambient Dust".to_string(),
            particles: Vec::new(),
            emitter_settings: EmitterSettings {
                emission_shape: EmissionShape::Box {
                    size: Vector3::new(50.0, 20.0, 50.0),
                },
                emission_rate: 5.0,
                burst_count: None,
                lifetime: 10.0,
                loop_emission: true,
            },
            behavior: ParticleBehavior {
                gravity: Vector3::new(0.0, -0.05, 0.0),
                drag: 0.01,
                turbulence: 0.1,
                collision_response: CollisionResponse::None,
                attraction_points: Vec::new(),
            },
            rendering: ParticleRendering {
                blend_mode: BlendMode::Alpha,
                texture: None,
                billboard: true,
                soft_particles: true,
                depth_write: false,
            },
        }
    }

    /// Update visual showcase
    pub fn update(&mut self, delta_time: f32) {
        // Update time of day
        self.time_of_day += delta_time * 0.05; // 1 game hour per 20 real seconds
        if self.time_of_day >= 24.0 {
            self.time_of_day -= 24.0;
        }

        // Update active scene
        match self.active_scene {
            VisualScene::LightingGallery => self.update_lighting(delta_time),
            VisualScene::MaterialShowcase => self.update_materials(delta_time),
            VisualScene::WeatherEffects => self.update_weather(delta_time),
            VisualScene::PostProcessing => self.update_post_processing(delta_time),
            VisualScene::WaterSimulation => self.update_water(delta_time),
            VisualScene::ParticleGallery => self.update_particles(delta_time),
        }
    }

    /// Update lighting gallery
    fn update_lighting(&mut self, delta_time: f32) {
        // Update sun position based on time of day
        let sun_angle = (self.time_of_day / 24.0) * std::f32::consts::PI * 2.0;
        self.lighting_gallery.sun_position = Vector3::new(
            sun_angle.cos() * 100.0,
            sun_angle.sin() * 100.0 + 50.0,
            50.0,
        );

        // Update torch flicker
        for torch in &mut self.lighting_gallery.torches {
            let flicker = (delta_time * torch.flicker_speed).sin() * torch.flicker_amount;
            // Intensity would be applied to the actual light source
        }

        // Update fireflies
        for firefly in &mut self.lighting_gallery.fireflies {
            firefly.position += firefly.velocity * delta_time;

            // Simple boundary check
            if firefly.position.x.abs() > 30.0 {
                firefly.velocity.x *= -1.0;
            }
            if firefly.position.z.abs() > 30.0 {
                firefly.velocity.z *= -1.0;
            }
        }
    }

    /// Update material showcase
    fn update_materials(&mut self, delta_time: f32) {
        // Rotate display platforms
        for platform in &mut self.material_showcase.platforms {
            let rotation = Quaternion::from_axis_angle(
                Vector3::unit_y(),
                Rad(self.material_showcase.rotation_speed * delta_time),
            );
            platform.rotation = platform.rotation * rotation;
        }
    }

    /// Update weather effects
    fn update_weather(&mut self, delta_time: f32) {
        // Update rain particles
        for particle in &mut self.weather_system.rain_particles {
            particle.position += particle.velocity * delta_time;

            // Reset if below ground
            if particle.position.y < 0.0 {
                particle.position.y = 50.0;
                particle.position.x = (rand::random::<f32>() - 0.5) * 100.0;
                particle.position.z = (rand::random::<f32>() - 0.5) * 100.0;
            }
        }

        // Update snow particles
        for particle in &mut self.weather_system.snow_particles {
            // Add sway motion
            particle.sway_offset += delta_time;
            let sway = particle.sway_offset.sin() * 2.0;

            particle.position.x += sway * delta_time;
            particle.position += particle.velocity * delta_time;

            // Reset if below ground
            if particle.position.y < 0.0 {
                particle.position.y = 50.0;
                particle.position.x = (rand::random::<f32>() - 0.5) * 100.0;
                particle.position.z = (rand::random::<f32>() - 0.5) * 100.0;
            }
        }

        // Update clouds
        for cloud in &mut self.weather_system.clouds {
            cloud.position += cloud.velocity * delta_time;

            // Wrap around
            if cloud.position.x > 100.0 {
                cloud.position.x = -100.0;
            }
        }
    }

    /// Update post-processing
    fn update_post_processing(&mut self, _delta_time: f32) {
        // Post-processing settings are mostly static
        // Could add animated transitions here
    }

    /// Update water simulation
    fn update_water(&mut self, delta_time: f32) {
        // Update ripples
        self.water_simulation.ripple_sources.retain(|ripple| {
            ripple.created_at.elapsed().as_secs_f32() < 3.0
        });

        // Update foam particles
        for particle in &mut self.water_simulation.foam_particles {
            particle.lifetime -= delta_time;
            particle.opacity = particle.lifetime / 2.0;
            particle.position += particle.velocity * delta_time;
        }

        self.water_simulation.foam_particles.retain(|p| p.lifetime > 0.0);

        // Update splash particles
        for particle in &mut self.water_simulation.splash_particles {
            particle.velocity.y -= 9.81 * particle.gravity_scale * delta_time;
            particle.position += particle.velocity * delta_time;
        }

        self.water_simulation.splash_particles.retain(|p| p.position.y > 0.0);
    }

    /// Update particle systems
    fn update_particles(&mut self, delta_time: f32) {
        // Update all particle systems
        Self::update_particle_system(&mut self.particle_gallery.fire_system, delta_time);
        Self::update_particle_system(&mut self.particle_gallery.smoke_system, delta_time);
        Self::update_particle_system(&mut self.particle_gallery.magic_system, delta_time);
        Self::update_particle_system(&mut self.particle_gallery.explosion_system, delta_time);
        Self::update_particle_system(&mut self.particle_gallery.ambient_particles, delta_time);
    }

    /// Update individual particle system
    fn update_particle_system(system: &mut ParticleSystem, delta_time: f32) {
        // Update existing particles
        for particle in &mut system.particles {
            particle.age += delta_time;

            // Apply physics
            particle.velocity += particle.acceleration * delta_time;
            particle.velocity += system.behavior.gravity * delta_time;
            particle.velocity *= 1.0 - system.behavior.drag * delta_time;

            // Apply turbulence
            if system.behavior.turbulence > 0.0 {
                particle.velocity.x += (rand::random::<f32>() - 0.5) * system.behavior.turbulence;
                particle.velocity.z += (rand::random::<f32>() - 0.5) * system.behavior.turbulence;
            }

            // Update position
            particle.position += particle.velocity * delta_time;

            // Age-based changes
            let age_ratio = particle.age / particle.lifetime;
            particle.size *= 1.0 - age_ratio * 0.5;
            particle.color.w = 1.0 - age_ratio;
        }

        // Remove dead particles
        system.particles.retain(|p| p.age < p.lifetime);

        // Emit new particles
        if system.emitter_settings.loop_emission {
            let emit_count = (system.emitter_settings.emission_rate * delta_time) as u32;
            for _ in 0..emit_count {
                system.particles.push(Self::create_particle(&system.emitter_settings));
            }
        }
    }

    /// Create a new particle
    fn create_particle(settings: &EmitterSettings) -> Particle {
        let position = match settings.emission_shape {
            EmissionShape::Point => Vector3::new(0.0, 0.0, 0.0),
            EmissionShape::Sphere { radius } => {
                let theta = rand::random::<f32>() * std::f32::consts::PI * 2.0;
                let phi = rand::random::<f32>() * std::f32::consts::PI;
                Vector3::new(
                    radius * phi.sin() * theta.cos(),
                    radius * phi.sin() * theta.sin(),
                    radius * phi.cos(),
                )
            }
            _ => Vector3::new(0.0, 0.0, 0.0),
        };

        Particle {
            position,
            velocity: Vector3::new(
                (rand::random::<f32>() - 0.5) * 2.0,
                rand::random::<f32>() * 5.0,
                (rand::random::<f32>() - 0.5) * 2.0,
            ),
            acceleration: Vector3::new(0.0, 0.0, 0.0),
            size: 0.5 + rand::random::<f32>() * 0.5,
            rotation: rand::random::<f32>() * std::f32::consts::PI * 2.0,
            color: Vector4::new(1.0, 0.5, 0.0, 1.0),
            lifetime: settings.lifetime,
            age: 0.0,
        }
    }

    /// Switch to a different scene
    pub fn switch_scene(&mut self, scene: VisualScene) {
        self.active_scene = scene;
        self.transition_progress = 0.0;
    }

    /// Get current scene
    pub fn get_current_scene(&self) -> &VisualScene {
        &self.active_scene
    }

    /// Get lighting gallery for rendering
    pub fn get_lighting_gallery(&self) -> &LightingGallery {
        &self.lighting_gallery
    }

    /// Get material showcase for rendering
    pub fn get_material_showcase(&self) -> &MaterialShowcase {
        &self.material_showcase
    }

    /// Get weather system for rendering
    pub fn get_weather_system(&self) -> &WeatherEffects {
        &self.weather_system
    }

    /// Get water simulation for rendering
    pub fn get_water_simulation(&self) -> &WaterSimulation {
        &self.water_simulation
    }

    /// Get particle gallery for rendering
    pub fn get_particle_gallery(&self) -> &ParticleGallery {
        &self.particle_gallery
    }
}

// Helper module for vector math
use cgmath::Vector2;

// External dependency for randomness
extern crate rand;