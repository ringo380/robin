use crate::engine::error::RobinResult;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleSystemConfig {
    pub max_particles_per_system: u32,
    pub max_active_systems: u32,
    pub enable_gpu_simulation: bool,
    pub enable_collision_detection: bool,
    pub enable_lighting_interaction: bool,
    pub enable_shadows: bool,
    pub quality_level: ParticleQuality,
    pub pooling_enabled: bool,
}

impl Default for ParticleSystemConfig {
    fn default() -> Self {
        Self {
            max_particles_per_system: 10000,
            max_active_systems: 50,
            enable_gpu_simulation: true,
            enable_collision_detection: true,
            enable_lighting_interaction: true,
            enable_shadows: false,
            quality_level: ParticleQuality::High,
            pooling_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParticleQuality {
    Ultra,
    High,
    Medium,
    Low,
}

impl ParticleQuality {
    pub fn get_max_particles(&self) -> u32 {
        match self {
            ParticleQuality::Ultra => 50000,
            ParticleQuality::High => 25000,
            ParticleQuality::Medium => 10000,
            ParticleQuality::Low => 5000,
        }
    }

    pub fn get_update_frequency(&self) -> f32 {
        match self {
            ParticleQuality::Ultra => 120.0, // 120 FPS
            ParticleQuality::High => 60.0,   // 60 FPS
            ParticleQuality::Medium => 30.0, // 30 FPS
            ParticleQuality::Low => 15.0,    // 15 FPS
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParticleType {
    Fire,
    Smoke,
    Steam,
    Water,
    Snow,
    Sparks,
    Magic,
    Explosion,
    Dust,
    Leaves,
    Ash,
    Blood,
    Energy,
    Lightning,
    Portal,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmissionShape {
    Point,
    Sphere { radius: f32 },
    Box { dimensions: [f32; 3] },
    Cone { radius: f32, height: f32, angle: f32 },
    Cylinder { radius: f32, height: f32 },
    Ring { inner_radius: f32, outer_radius: f32 },
    Mesh { mesh_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlendMode {
    Alpha,
    Additive,
    Multiply,
    Screen,
    Subtract,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleEmitterSettings {
    pub emission_rate: f32, // particles per second
    pub burst_count: u32,
    pub burst_interval: f32,
    pub emission_shape: EmissionShape,
    pub emission_direction: [f32; 3],
    pub emission_spread: f32, // cone angle in degrees
    pub inherit_velocity: f32, // how much emitter velocity affects particles
    
    // Particle properties
    pub initial_velocity: [f32; 2], // min, max
    pub initial_size: [f32; 2],     // min, max
    pub initial_rotation: [f32; 2], // min, max (degrees)
    pub initial_color: [f32; 4],
    pub lifetime: [f32; 2],         // min, max seconds
    
    // Physics
    pub gravity: [f32; 3],
    pub drag: f32,
    pub bounce_factor: f32,
    pub collision_radius: f32,
    
    // Rendering
    pub texture_id: Option<String>,
    pub blend_mode: BlendMode,
    pub sort_particles: bool,
    pub cast_shadows: bool,
    pub receive_lighting: bool,
}

impl Default for ParticleEmitterSettings {
    fn default() -> Self {
        Self {
            emission_rate: 50.0,
            burst_count: 0,
            burst_interval: 0.0,
            emission_shape: EmissionShape::Point,
            emission_direction: [0.0, 1.0, 0.0],
            emission_spread: 15.0,
            inherit_velocity: 0.1,
            
            initial_velocity: [1.0, 5.0],
            initial_size: [0.5, 1.5],
            initial_rotation: [0.0, 360.0],
            initial_color: [1.0, 1.0, 1.0, 1.0],
            lifetime: [2.0, 5.0],
            
            gravity: [0.0, -9.81, 0.0],
            drag: 0.1,
            bounce_factor: 0.3,
            collision_radius: 0.1,
            
            texture_id: None,
            blend_mode: BlendMode::Alpha,
            sort_particles: true,
            cast_shadows: false,
            receive_lighting: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Particle {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub size: f32,
    pub rotation: f32,
    pub angular_velocity: f32,
    pub color: [f32; 4],
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub mass: f32,
    pub age: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleForce {
    pub name: String,
    pub force_type: ForceType,
    pub strength: f32,
    pub radius: f32,
    pub position: [f32; 3],
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub enum ForceType {
    Gravity,
    Wind,
    Vortex,
    Turbulence,
    Attraction,
    Repulsion,
    Drag,
    Brownian,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParticleSystem {
    pub name: String,
    pub particle_type: ParticleType,
    pub emitter_settings: ParticleEmitterSettings,
    pub particles: Vec<Particle>,
    pub active_particles: usize,
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub velocity: [f32; 3],
    pub enabled: bool,
    pub looping: bool,
    
    // Animation curves
    pub size_over_lifetime: Vec<([f32; 2])>, // time, size pairs
    pub color_over_lifetime: Vec<([f32; 5])>, // time, r, g, b, a
    pub velocity_over_lifetime: Vec<([f32; 4])>, // time, x, y, z
    
    // Performance tracking
    pub emission_accumulator: f32,
    pub last_burst_time: f32,
    pub creation_time: Instant,
    pub last_update: Instant,
    
    // Forces
    pub local_forces: Vec<ParticleForce>,
}

#[derive(Debug)]
pub struct ParticleManager {
    config: ParticleSystemConfig,
    systems: HashMap<String, ParticleSystem>,
    global_forces: Vec<ParticleForce>,
    particle_pools: HashMap<ParticleType, Vec<Particle>>,
    presets: HashMap<String, ParticleEmitterSettings>,
    texture_cache: HashMap<String, TextureInfo>,
    performance_stats: ParticleStats,
    collision_grid: SpatialGrid,
    render_batches: Vec<RenderBatch>,
}

#[derive(Debug, Clone)]
struct TextureInfo {
    pub width: u32,
    pub height: u32,
    pub frames: u32, // For animated textures
    pub fps: f32,
}

#[derive(Debug, Default)]
pub struct ParticleStats {
    pub total_particles: u32,
    pub active_systems: u32,
    pub particles_spawned_this_frame: u32,
    pub particles_destroyed_this_frame: u32,
    pub update_time_ms: f32,
    pub render_time_ms: f32,
    pub collision_checks: u32,
    pub memory_usage_mb: f32,
}

#[derive(Debug)]
struct SpatialGrid {
    cell_size: f32,
    cells: HashMap<[i32; 3], Vec<usize>>, // particle indices
    bounds: [f32; 6], // min_x, min_y, min_z, max_x, max_y, max_z
}

#[derive(Debug)]
struct RenderBatch {
    pub texture_id: Option<String>,
    pub blend_mode: BlendMode,
    pub particle_indices: Vec<(String, usize)>, // system_name, particle_index
    pub depth_sorted: bool,
}

impl ParticleManager {
    pub fn new(config: ParticleSystemConfig) -> RobinResult<Self> {
        let mut manager = Self {
            config,
            systems: HashMap::new(),
            global_forces: Vec::new(),
            particle_pools: HashMap::new(),
            presets: HashMap::new(),
            texture_cache: HashMap::new(),
            performance_stats: ParticleStats::default(),
            collision_grid: SpatialGrid::new(10.0, [-1000.0, -1000.0, -1000.0, 1000.0, 1000.0, 1000.0]),
            render_batches: Vec::new(),
        };

        manager.initialize_presets()?;
        manager.initialize_particle_pools()?;
        
        Ok(manager)
    }

    pub fn create_system(&mut self, name: String, particle_type: ParticleType, position: [f32; 3]) -> RobinResult<()> {
        if self.systems.len() >= self.config.max_active_systems as usize {
            return Err(crate::engine::error::RobinError::new(
                "Maximum number of particle systems reached".to_string()
            ));
        }

        let preset_name = format!("{:?}", particle_type);
        let emitter_settings = self.presets.get(&preset_name)
            .cloned()
            .unwrap_or_else(|| ParticleEmitterSettings::default());

        let max_particles = self.config.max_particles_per_system.min(
            self.config.quality_level.get_max_particles()
        );

        let system = ParticleSystem {
            name: name.clone(),
            particle_type,
            emitter_settings,
            particles: Vec::with_capacity(max_particles as usize),
            active_particles: 0,
            position,
            rotation: [0.0, 0.0, 0.0],
            velocity: [0.0, 0.0, 0.0],
            enabled: true,
            looping: true,
            size_over_lifetime: vec![[0.0, 1.0], [1.0, 0.0]], // Shrink over time
            color_over_lifetime: vec![[0.0, 1.0, 1.0, 1.0, 1.0], [1.0, 1.0, 1.0, 1.0, 0.0]], // Fade out
            velocity_over_lifetime: vec![[0.0, 1.0, 1.0, 1.0], [1.0, 0.5, 0.5, 0.5]], // Slow down
            emission_accumulator: 0.0,
            last_burst_time: 0.0,
            creation_time: Instant::now(),
            last_update: Instant::now(),
            local_forces: Vec::new(),
        };

        self.systems.insert(name, system);
        Ok(())
    }

    pub fn create_system_from_preset(&mut self, name: String, preset_name: &str, position: [f32; 3]) -> RobinResult<()> {
        let particle_type = match preset_name {
            "Fire" => ParticleType::Fire,
            "Smoke" => ParticleType::Smoke,
            "Water" => ParticleType::Water,
            "Magic" => ParticleType::Magic,
            "Explosion" => ParticleType::Explosion,
            _ => ParticleType::Custom(preset_name.to_string()),
        };

        self.create_system(name, particle_type, position)
    }

    pub fn update(&mut self, delta_time: f32) -> RobinResult<()> {
        let update_start = Instant::now();

        // Update performance stats
        self.performance_stats.particles_spawned_this_frame = 0;
        self.performance_stats.particles_destroyed_this_frame = 0;
        self.performance_stats.collision_checks = 0;

        // Clear collision grid
        self.collision_grid.clear();

        // Update all active particle systems
        let system_names: Vec<String> = self.systems.keys().cloned().collect();
        for system_name in system_names {
            self.update_system(&system_name, delta_time)?;
        }

        // Update global statistics
        self.performance_stats.total_particles = self.systems.values()
            .map(|s| s.active_particles as u32)
            .sum();
        self.performance_stats.active_systems = self.systems.len() as u32;
        self.performance_stats.update_time_ms = update_start.elapsed().as_secs_f32() * 1000.0;

        // Prepare render batches
        self.prepare_render_batches()?;

        Ok(())
    }

    pub fn emit_burst(&mut self, system_name: &str, count: u32) -> RobinResult<()> {
        if let Some(system) = self.systems.get_mut(system_name) {
            for _ in 0..count {
                self.spawn_particle_in_system(system)?;
            }
        }
        Ok(())
    }

    pub fn set_system_position(&mut self, system_name: &str, position: [f32; 3]) -> RobinResult<()> {
        if let Some(system) = self.systems.get_mut(system_name) {
            system.position = position;
        }
        Ok(())
    }

    pub fn set_system_velocity(&mut self, system_name: &str, velocity: [f32; 3]) -> RobinResult<()> {
        if let Some(system) = self.systems.get_mut(system_name) {
            system.velocity = velocity;
        }
        Ok(())
    }

    pub fn enable_system(&mut self, system_name: &str, enabled: bool) -> RobinResult<()> {
        if let Some(system) = self.systems.get_mut(system_name) {
            system.enabled = enabled;
        }
        Ok(())
    }

    pub fn destroy_system(&mut self, system_name: &str) -> RobinResult<()> {
        if let Some(system) = self.systems.remove(system_name) {
            // Return particles to pool
            if self.config.pooling_enabled {
                if let Some(pool) = self.particle_pools.get_mut(&system.particle_type) {
                    pool.extend(system.particles.into_iter().take(system.active_particles));
                }
            }
        }
        Ok(())
    }

    pub fn add_global_force(&mut self, force: ParticleForce) {
        self.global_forces.push(force);
    }

    pub fn add_system_force(&mut self, system_name: &str, force: ParticleForce) -> RobinResult<()> {
        if let Some(system) = self.systems.get_mut(system_name) {
            system.local_forces.push(force);
        }
        Ok(())
    }

    pub fn get_system_info(&self, system_name: &str) -> Option<(u32, ParticleType)> {
        self.systems.get(system_name)
            .map(|s| (s.active_particles as u32, s.particle_type.clone()))
    }

    pub fn get_stats(&self) -> &ParticleStats {
        &self.performance_stats
    }

    pub fn get_render_batches(&self) -> &[RenderBatch] {
        &self.render_batches
    }

    pub fn get_particles_in_system(&self, system_name: &str) -> Option<&[Particle]> {
        self.systems.get(system_name)
            .map(|s| &s.particles[..s.active_particles])
    }

    fn update_system(&mut self, system_name: &str, delta_time: f32) -> RobinResult<()> {
        let system = match self.systems.get_mut(system_name) {
            Some(s) => s,
            None => return Ok(()),
        };

        if !system.enabled {
            return Ok(());
        }

        let dt = delta_time;
        system.last_update = Instant::now();

        // Emit new particles
        if system.looping || system.creation_time.elapsed().as_secs_f32() < 10.0 {
            system.emission_accumulator += system.emitter_settings.emission_rate * dt;
            
            while system.emission_accumulator >= 1.0 && system.active_particles < system.particles.capacity() {
                self.spawn_particle_in_system(system)?;
                system.emission_accumulator -= 1.0;
                self.performance_stats.particles_spawned_this_frame += 1;
            }
        }

        // Handle burst emissions
        if system.emitter_settings.burst_count > 0 {
            let time_since_creation = system.creation_time.elapsed().as_secs_f32();
            let time_since_last_burst = time_since_creation - system.last_burst_time;
            
            if time_since_last_burst >= system.emitter_settings.burst_interval {
                for _ in 0..system.emitter_settings.burst_count.min(
                    (system.particles.capacity() - system.active_particles) as u32
                ) {
                    self.spawn_particle_in_system(system)?;
                    self.performance_stats.particles_spawned_this_frame += 1;
                }
                system.last_burst_time = time_since_creation;
            }
        }

        // Update existing particles
        let mut particles_to_remove = Vec::new();
        
        for i in 0..system.active_particles {
            let particle = &mut system.particles[i];
            
            // Age the particle
            particle.age += dt;
            let life_progress = particle.age / particle.max_lifetime;
            
            // Check if particle should die
            if particle.age >= particle.max_lifetime {
                particles_to_remove.push(i);
                continue;
            }
            
            // Apply animation curves
            self.apply_particle_curves(particle, system, life_progress);
            
            // Apply forces
            self.apply_forces_to_particle(particle, system);
            
            // Update physics
            particle.velocity[0] += system.emitter_settings.gravity[0] * dt;
            particle.velocity[1] += system.emitter_settings.gravity[1] * dt;
            particle.velocity[2] += system.emitter_settings.gravity[2] * dt;
            
            // Apply drag
            let drag_factor = 1.0 - (system.emitter_settings.drag * dt).min(1.0);
            particle.velocity[0] *= drag_factor;
            particle.velocity[1] *= drag_factor;
            particle.velocity[2] *= drag_factor;
            
            // Update position
            particle.position[0] += particle.velocity[0] * dt;
            particle.position[1] += particle.velocity[1] * dt;
            particle.position[2] += particle.velocity[2] * dt;
            
            // Update rotation
            particle.rotation += particle.angular_velocity * dt;
            
            // Handle collisions if enabled
            if self.config.enable_collision_detection {
                self.handle_particle_collision(particle, system);
            }
            
            // Add to collision grid
            self.collision_grid.add_particle(particle.position, i);
        }
        
        // Remove dead particles
        for &remove_index in particles_to_remove.iter().rev() {
            if remove_index < system.active_particles {
                system.active_particles -= 1;
                if remove_index != system.active_particles {
                    system.particles.swap(remove_index, system.active_particles);
                }
                self.performance_stats.particles_destroyed_this_frame += 1;
            }
        }

        Ok(())
    }

    fn spawn_particle_in_system(&mut self, system: &mut ParticleSystem) -> RobinResult<()> {
        if system.active_particles >= system.particles.capacity() {
            return Ok(());
        }

        // Get or create particle
        let mut particle = if self.config.pooling_enabled {
            if let Some(pool) = self.particle_pools.get_mut(&system.particle_type) {
                pool.pop().unwrap_or_else(|| self.create_default_particle())
            } else {
                self.create_default_particle()
            }
        } else {
            self.create_default_particle()
        };

        // Initialize particle based on emitter settings
        self.initialize_particle(&mut particle, system);

        // Add to system
        if system.active_particles < system.particles.len() {
            system.particles[system.active_particles] = particle;
        } else {
            system.particles.push(particle);
        }
        system.active_particles += 1;

        Ok(())
    }

    fn create_default_particle(&self) -> Particle {
        Particle {
            position: [0.0, 0.0, 0.0],
            velocity: [0.0, 0.0, 0.0],
            size: 1.0,
            rotation: 0.0,
            angular_velocity: 0.0,
            color: [1.0, 1.0, 1.0, 1.0],
            lifetime: 1.0,
            max_lifetime: 1.0,
            mass: 1.0,
            age: 0.0,
        }
    }

    fn initialize_particle(&self, particle: &mut Particle, system: &ParticleSystem) {
        let settings = &system.emitter_settings;
        
        // Set position based on emission shape
        particle.position = self.sample_emission_shape(&settings.emission_shape, &system.position);
        
        // Set initial velocity
        let speed = self.lerp(settings.initial_velocity[0], settings.initial_velocity[1], 0.5); // Mock random
        let direction = self.get_emission_direction(settings, particle.position, system.position);
        particle.velocity = [
            direction[0] * speed + system.velocity[0] * settings.inherit_velocity,
            direction[1] * speed + system.velocity[1] * settings.inherit_velocity,
            direction[2] * speed + system.velocity[2] * settings.inherit_velocity,
        ];
        
        // Set other properties
        particle.size = self.lerp(settings.initial_size[0], settings.initial_size[1], 0.5);
        particle.rotation = self.lerp(settings.initial_rotation[0], settings.initial_rotation[1], 0.5).to_radians();
        particle.angular_velocity = (0.5 - 0.5) * 10.0; // Mock random angular velocity
        particle.color = settings.initial_color;
        particle.max_lifetime = self.lerp(settings.lifetime[0], settings.lifetime[1], 0.5);
        particle.age = 0.0;
        particle.mass = particle.size * particle.size; // Mass proportional to area
    }

    fn sample_emission_shape(&self, shape: &EmissionShape, emitter_pos: &[f32; 3]) -> [f32; 3] {
        match shape {
            EmissionShape::Point => *emitter_pos,
            EmissionShape::Sphere { radius } => {
                // Mock spherical sampling
                let offset = 0.5 * radius; // Mock random offset
                [
                    emitter_pos[0] + offset,
                    emitter_pos[1] + offset,
                    emitter_pos[2] + offset,
                ]
            },
            EmissionShape::Box { dimensions } => {
                [
                    emitter_pos[0] + (0.5 - 0.5) * dimensions[0],
                    emitter_pos[1] + (0.5 - 0.5) * dimensions[1], 
                    emitter_pos[2] + (0.5 - 0.5) * dimensions[2],
                ]
            },
            EmissionShape::Cone { radius, height, .. } => {
                let r = 0.5 * radius; // Mock random radius
                let h = 0.5 * height; // Mock random height
                [
                    emitter_pos[0] + r,
                    emitter_pos[1] + h,
                    emitter_pos[2] + r,
                ]
            },
            _ => *emitter_pos, // Default to point emission
        }
    }

    fn get_emission_direction(&self, settings: &ParticleEmitterSettings, particle_pos: [f32; 3], emitter_pos: [f32; 3]) -> [f32; 3] {
        let mut direction = settings.emission_direction;
        
        // Add spread
        if settings.emission_spread > 0.0 {
            let spread_rad = settings.emission_spread.to_radians();
            let cone_offset = spread_rad * 0.5; // Mock random cone offset
            direction[0] += cone_offset;
            direction[2] += cone_offset;
        }
        
        // Normalize direction
        let length = (direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2]).sqrt();
        if length > 0.0 {
            direction[0] /= length;
            direction[1] /= length;
            direction[2] /= length;
        }
        
        direction
    }

    fn apply_particle_curves(&self, particle: &mut Particle, system: &ParticleSystem, life_progress: f32) {
        // Apply size over lifetime
        if !system.size_over_lifetime.is_empty() {
            let size_multiplier = self.sample_curve(&system.size_over_lifetime, life_progress);
            particle.size *= size_multiplier;
        }
        
        // Apply color over lifetime
        if !system.color_over_lifetime.is_empty() {
            let color_sample = self.sample_color_curve(&system.color_over_lifetime, life_progress);
            particle.color = [
                particle.color[0] * color_sample[0],
                particle.color[1] * color_sample[1],
                particle.color[2] * color_sample[2],
                particle.color[3] * color_sample[3],
            ];
        }
    }

    fn apply_forces_to_particle(&self, particle: &mut Particle, system: &ParticleSystem) {
        // Apply global forces
        for force in &self.global_forces {
            if force.enabled {
                self.apply_force(particle, force);
            }
        }
        
        // Apply local forces
        for force in &system.local_forces {
            if force.enabled {
                self.apply_force(particle, force);
            }
        }
    }

    fn apply_force(&self, particle: &mut Particle, force: &ParticleForce) {
        let distance_sq = (particle.position[0] - force.position[0]).powi(2) +
                         (particle.position[1] - force.position[1]).powi(2) +
                         (particle.position[2] - force.position[2]).powi(2);
        
        if distance_sq > force.radius * force.radius {
            return; // Outside force radius
        }
        
        match force.force_type {
            ForceType::Gravity => {
                particle.velocity[1] -= force.strength * 0.016; // Assume 60 FPS
            },
            ForceType::Wind => {
                particle.velocity[0] += force.strength * 0.1;
                particle.velocity[2] += force.strength * 0.05;
            },
            ForceType::Vortex => {
                let dx = particle.position[0] - force.position[0];
                let dz = particle.position[2] - force.position[2];
                particle.velocity[0] += -dz * force.strength * 0.01;
                particle.velocity[2] += dx * force.strength * 0.01;
            },
            ForceType::Attraction => {
                let dx = force.position[0] - particle.position[0];
                let dy = force.position[1] - particle.position[1];
                let dz = force.position[2] - particle.position[2];
                let distance = (dx * dx + dy * dy + dz * dz).sqrt();
                if distance > 0.0 {
                    let strength = force.strength / (distance * distance + 1.0);
                    particle.velocity[0] += dx * strength * 0.01;
                    particle.velocity[1] += dy * strength * 0.01;
                    particle.velocity[2] += dz * strength * 0.01;
                }
            },
            ForceType::Turbulence => {
                particle.velocity[0] += (particle.position[0] * 0.1).sin() * force.strength * 0.01;
                particle.velocity[1] += (particle.position[1] * 0.15).cos() * force.strength * 0.01;
                particle.velocity[2] += (particle.position[2] * 0.08).sin() * force.strength * 0.01;
            },
            _ => {}, // Other force types would be implemented similarly
        }
    }

    fn handle_particle_collision(&self, particle: &mut Particle, system: &ParticleSystem) {
        // Simple ground collision
        if particle.position[1] < 0.0 {
            particle.position[1] = 0.0;
            particle.velocity[1] = -particle.velocity[1] * system.emitter_settings.bounce_factor;
        }
        
        self.performance_stats.collision_checks += 1;
    }

    fn sample_curve(&self, curve: &[([f32; 2])], time: f32) -> f32 {
        if curve.is_empty() {
            return 1.0;
        }
        
        if curve.len() == 1 {
            return curve[0][1];
        }
        
        // Find the two keyframes to interpolate between
        for i in 0..curve.len() - 1 {
            if time >= curve[i][0] && time <= curve[i + 1][0] {
                let t = (time - curve[i][0]) / (curve[i + 1][0] - curve[i][0]);
                return self.lerp(curve[i][1], curve[i + 1][1], t);
            }
        }
        
        // Return last value if time is beyond curve
        curve.last().unwrap()[1]
    }

    fn sample_color_curve(&self, curve: &[([f32; 5])], time: f32) -> [f32; 4] {
        if curve.is_empty() {
            return [1.0, 1.0, 1.0, 1.0];
        }
        
        if curve.len() == 1 {
            return [curve[0][1], curve[0][2], curve[0][3], curve[0][4]];
        }
        
        // Find interpolation keyframes
        for i in 0..curve.len() - 1 {
            if time >= curve[i][0] && time <= curve[i + 1][0] {
                let t = (time - curve[i][0]) / (curve[i + 1][0] - curve[i][0]);
                return [
                    self.lerp(curve[i][1], curve[i + 1][1], t),
                    self.lerp(curve[i][2], curve[i + 1][2], t),
                    self.lerp(curve[i][3], curve[i + 1][3], t),
                    self.lerp(curve[i][4], curve[i + 1][4], t),
                ];
            }
        }
        
        let last = curve.last().unwrap();
        [last[1], last[2], last[3], last[4]]
    }

    fn prepare_render_batches(&mut self) -> RobinResult<()> {
        self.render_batches.clear();
        
        // Group particles by texture and blend mode for efficient rendering
        let mut batches: HashMap<(Option<String>, BlendMode), Vec<(String, usize)>> = HashMap::new();
        
        for (system_name, system) in &self.systems {
            let texture_id = system.emitter_settings.texture_id.clone();
            let blend_mode = system.emitter_settings.blend_mode.clone();
            let key = (texture_id, blend_mode);
            
            let batch = batches.entry(key.clone()).or_insert_with(Vec::new);
            
            for i in 0..system.active_particles {
                batch.push((system_name.clone(), i));
            }
        }
        
        // Convert to render batches
        for ((texture_id, blend_mode), particle_indices) in batches {
            let needs_sorting = particle_indices.iter().any(|(system_name, _)| {
                self.systems.get(system_name)
                    .map(|s| s.emitter_settings.sort_particles)
                    .unwrap_or(false)
            });
            
            self.render_batches.push(RenderBatch {
                texture_id,
                blend_mode,
                particle_indices,
                depth_sorted: needs_sorting,
            });
        }
        
        Ok(())
    }

    fn initialize_presets(&mut self) -> RobinResult<()> {
        // Fire preset
        let mut fire_settings = ParticleEmitterSettings::default();
        fire_settings.emission_rate = 100.0;
        fire_settings.initial_velocity = [2.0, 8.0];
        fire_settings.initial_size = [0.5, 2.0];
        fire_settings.initial_color = [1.0, 0.4, 0.1, 0.8];
        fire_settings.lifetime = [0.8, 2.0];
        fire_settings.gravity = [0.0, 5.0, 0.0]; // Upward buoyancy
        fire_settings.blend_mode = BlendMode::Additive;
        fire_settings.emission_shape = EmissionShape::Cone { radius: 0.5, height: 1.0, angle: 30.0 };
        self.presets.insert("Fire".to_string(), fire_settings);

        // Smoke preset
        let mut smoke_settings = ParticleEmitterSettings::default();
        smoke_settings.emission_rate = 30.0;
        smoke_settings.initial_velocity = [0.5, 3.0];
        smoke_settings.initial_size = [1.0, 3.0];
        smoke_settings.initial_color = [0.2, 0.2, 0.2, 0.6];
        smoke_settings.lifetime = [3.0, 8.0];
        smoke_settings.gravity = [0.0, 2.0, 0.0];
        smoke_settings.drag = 0.3;
        smoke_settings.blend_mode = BlendMode::Alpha;
        self.presets.insert("Smoke".to_string(), smoke_settings);

        // Water preset
        let mut water_settings = ParticleEmitterSettings::default();
        water_settings.emission_rate = 200.0;
        water_settings.initial_velocity = [3.0, 12.0];
        water_settings.initial_size = [0.1, 0.3];
        water_settings.initial_color = [0.3, 0.5, 1.0, 0.8];
        water_settings.lifetime = [1.0, 3.0];
        water_settings.gravity = [0.0, -20.0, 0.0];
        water_settings.bounce_factor = 0.1;
        water_settings.collision_radius = 0.05;
        self.presets.insert("Water".to_string(), water_settings);

        // Magic preset
        let mut magic_settings = ParticleEmitterSettings::default();
        magic_settings.emission_rate = 80.0;
        magic_settings.initial_velocity = [1.0, 4.0];
        magic_settings.initial_size = [0.3, 1.0];
        magic_settings.initial_color = [0.5, 0.2, 1.0, 0.9];
        magic_settings.lifetime = [2.0, 5.0];
        magic_settings.gravity = [0.0, 1.0, 0.0];
        magic_settings.blend_mode = BlendMode::Additive;
        magic_settings.emission_shape = EmissionShape::Sphere { radius: 1.0 };
        self.presets.insert("Magic".to_string(), magic_settings);

        // Explosion preset
        let mut explosion_settings = ParticleEmitterSettings::default();
        explosion_settings.emission_rate = 0.0; // Burst only
        explosion_settings.burst_count = 200;
        explosion_settings.initial_velocity = [10.0, 25.0];
        explosion_settings.initial_size = [0.5, 2.5];
        explosion_settings.initial_color = [1.0, 0.6, 0.1, 1.0];
        explosion_settings.lifetime = [0.5, 2.0];
        explosion_settings.gravity = [0.0, -15.0, 0.0];
        explosion_settings.blend_mode = BlendMode::Additive;
        explosion_settings.emission_shape = EmissionShape::Sphere { radius: 0.1 };
        explosion_settings.emission_spread = 360.0;
        self.presets.insert("Explosion".to_string(), explosion_settings);

        Ok(())
    }

    fn initialize_particle_pools(&mut self) -> RobinResult<()> {
        if !self.config.pooling_enabled {
            return Ok(());
        }

        let pool_size = 1000; // Pre-allocate particles per type
        let particle_types = [
            ParticleType::Fire,
            ParticleType::Smoke,
            ParticleType::Water,
            ParticleType::Magic,
            ParticleType::Explosion,
            ParticleType::Sparks,
            ParticleType::Dust,
        ];

        for particle_type in &particle_types {
            let mut pool = Vec::with_capacity(pool_size);
            for _ in 0..pool_size {
                pool.push(self.create_default_particle());
            }
            self.particle_pools.insert(particle_type.clone(), pool);
        }

        Ok(())
    }

    fn lerp(&self, a: f32, b: f32, t: f32) -> f32 {
        a + (b - a) * t
    }

    pub fn get_system_names(&self) -> Vec<String> {
        self.systems.keys().cloned().collect()
    }

    pub fn get_preset_names(&self) -> Vec<String> {
        self.presets.keys().cloned().collect()
    }

    pub fn shutdown(&mut self) -> RobinResult<()> {
        println!("Particle Manager shutdown:");
        println!("  Active systems: {}", self.systems.len());
        println!("  Total particles: {}", self.performance_stats.total_particles);
        println!("  Memory usage: {:.1} MB", self.performance_stats.memory_usage_mb);

        self.systems.clear();
        self.particle_pools.clear();
        self.render_batches.clear();
        self.global_forces.clear();

        Ok(())
    }
}

impl SpatialGrid {
    fn new(cell_size: f32, bounds: [f32; 6]) -> Self {
        Self {
            cell_size,
            cells: HashMap::new(),
            bounds,
        }
    }

    fn clear(&mut self) {
        self.cells.clear();
    }

    fn add_particle(&mut self, position: [f32; 3], particle_index: usize) {
        let cell = [
            (position[0] / self.cell_size) as i32,
            (position[1] / self.cell_size) as i32,
            (position[2] / self.cell_size) as i32,
        ];

        self.cells.entry(cell).or_insert_with(Vec::new).push(particle_index);
    }

    fn get_nearby_particles(&self, position: [f32; 3]) -> Vec<usize> {
        let cell = [
            (position[0] / self.cell_size) as i32,
            (position[1] / self.cell_size) as i32,
            (position[2] / self.cell_size) as i32,
        ];

        let mut nearby = Vec::new();

        // Check surrounding cells (3x3x3 neighborhood)
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    let check_cell = [cell[0] + dx, cell[1] + dy, cell[2] + dz];
                    if let Some(particles) = self.cells.get(&check_cell) {
                        nearby.extend(particles.iter().cloned());
                    }
                }
            }
        }

        nearby
    }
}