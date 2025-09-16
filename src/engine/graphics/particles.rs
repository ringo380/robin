use crate::engine::math::Vec2;
use cgmath::InnerSpace;
use rand::Rng;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Particle {
    pub position: Vec2,
    pub velocity: Vec2,
    pub acceleration: Vec2,
    pub color: [f32; 4],
    pub size: f32,
    pub life: f32,
    pub max_life: f32,
    pub rotation: f32,
    pub angular_velocity: f32,
}

impl Particle {
    pub fn new() -> Self {
        Self {
            position: Vec2::new(0.0, 0.0),
            velocity: Vec2::new(0.0, 0.0),
            acceleration: Vec2::new(0.0, 0.0),
            color: [1.0, 1.0, 1.0, 1.0],
            size: 1.0,
            life: 1.0,
            max_life: 1.0,
            rotation: 0.0,
            angular_velocity: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.velocity += self.acceleration * dt;
        self.position += self.velocity * dt;
        self.rotation += self.angular_velocity * dt;
        self.life -= dt;
    }

    pub fn is_alive(&self) -> bool {
        self.life > 0.0
    }

    pub fn life_ratio(&self) -> f32 {
        self.life / self.max_life
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParticleEmitterConfig {
    pub position: Vec2,
    pub emission_rate: f32,
    pub particle_lifetime: (f32, f32), // (min, max)
    pub initial_velocity: (Vec2, Vec2), // (min, max)
    pub initial_size: (f32, f32),
    pub color_over_life: Vec<[f32; 4]>,
    pub size_over_life: Vec<f32>,
    pub gravity: Vec2,
    pub spread_angle: f32, // In radians
    pub burst_count: Option<u32>,
    pub texture_id: Option<u32>,
}

impl Default for ParticleEmitterConfig {
    fn default() -> Self {
        Self {
            position: Vec2::new(0.0, 0.0),
            emission_rate: 10.0,
            particle_lifetime: (1.0, 2.0),
            initial_velocity: (Vec2::new(-50.0, -50.0), Vec2::new(50.0, 50.0)),
            initial_size: (2.0, 8.0),
            color_over_life: vec![[1.0, 1.0, 1.0, 1.0], [1.0, 1.0, 1.0, 0.0]],
            size_over_life: vec![1.0, 0.0],
            gravity: Vec2::new(0.0, -98.0),
            spread_angle: std::f32::consts::PI * 2.0,
            burst_count: None,
            texture_id: None,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParticleEmitter {
    pub config: ParticleEmitterConfig,
    pub particles: Vec<Particle>,
    pub active: bool,
    emission_timer: f32,
    burst_emitted: bool,
}

impl ParticleEmitter {
    pub fn new(config: ParticleEmitterConfig) -> Self {
        Self {
            config,
            particles: Vec::new(),
            active: true,
            emission_timer: 0.0,
            burst_emitted: false,
        }
    }

    pub fn update(&mut self, dt: f32) {
        // Update existing particles
        self.particles.retain_mut(|particle| {
            particle.update(dt);
            
            // Apply gravity
            particle.acceleration = self.config.gravity;
            
            // Update color and size based on life
            let life_ratio = particle.life_ratio();
            if self.config.color_over_life.len() >= 2 {
                particle.color = Self::interpolate_color(
                    &self.config.color_over_life,
                    1.0 - life_ratio
                );
            }
            
            if self.config.size_over_life.len() >= 2 {
                particle.size = Self::interpolate_size(
                    &self.config.size_over_life,
                    1.0 - life_ratio
                ) * particle.size;
            }
            
            particle.is_alive()
        });

        // Emit new particles
        if self.active {
            if let Some(burst_count) = self.config.burst_count {
                if !self.burst_emitted {
                    self.emit_burst(burst_count);
                    self.burst_emitted = true;
                }
            } else {
                self.emission_timer += dt;
                let emission_interval = 1.0 / self.config.emission_rate;
                
                while self.emission_timer >= emission_interval {
                    self.emit_particle();
                    self.emission_timer -= emission_interval;
                }
            }
        }
    }

    fn emit_particle(&mut self) {
        let mut rng = rand::thread_rng();
        
        let lifetime = rng.gen_range(
            self.config.particle_lifetime.0..=self.config.particle_lifetime.1
        );
        
        let speed = rng.gen_range(
            self.config.initial_velocity.0.magnitude()..=self.config.initial_velocity.1.magnitude()
        );
        
        let angle = if self.config.spread_angle >= std::f32::consts::PI * 2.0 {
            rng.gen_range(0.0..std::f32::consts::PI * 2.0)
        } else {
            let base_angle = std::f32::consts::PI / 2.0; // Up direction
            let half_spread = self.config.spread_angle / 2.0;
            rng.gen_range((base_angle - half_spread)..=(base_angle + half_spread))
        };
        
        let velocity = Vec2::new(
            angle.cos() * speed,
            angle.sin() * speed
        );
        
        let size = rng.gen_range(self.config.initial_size.0..=self.config.initial_size.1);
        
        let mut particle = Particle::new();
        particle.position = self.config.position;
        particle.velocity = velocity;
        particle.life = lifetime;
        particle.max_life = lifetime;
        particle.size = size;
        particle.color = self.config.color_over_life.first()
            .copied()
            .unwrap_or([1.0, 1.0, 1.0, 1.0]);
        particle.angular_velocity = rng.gen_range(-5.0..5.0);
        
        self.particles.push(particle);
    }

    fn emit_burst(&mut self, count: u32) {
        for _ in 0..count {
            self.emit_particle();
        }
    }

    fn interpolate_color(colors: &[[f32; 4]], t: f32) -> [f32; 4] {
        if colors.len() < 2 {
            return colors.first().copied().unwrap_or([1.0, 1.0, 1.0, 1.0]);
        }
        
        let t = t.clamp(0.0, 1.0);
        let segment_size = 1.0 / (colors.len() - 1) as f32;
        let segment_index = (t / segment_size) as usize;
        let local_t = (t % segment_size) / segment_size;
        
        let start_color = colors[segment_index.min(colors.len() - 1)];
        let end_color = colors[(segment_index + 1).min(colors.len() - 1)];
        
        [
            start_color[0] + (end_color[0] - start_color[0]) * local_t,
            start_color[1] + (end_color[1] - start_color[1]) * local_t,
            start_color[2] + (end_color[2] - start_color[2]) * local_t,
            start_color[3] + (end_color[3] - start_color[3]) * local_t,
        ]
    }

    fn interpolate_size(sizes: &[f32], t: f32) -> f32 {
        if sizes.len() < 2 {
            return sizes.first().copied().unwrap_or(1.0);
        }
        
        let t = t.clamp(0.0, 1.0);
        let segment_size = 1.0 / (sizes.len() - 1) as f32;
        let segment_index = (t / segment_size) as usize;
        let local_t = (t % segment_size) / segment_size;
        
        let start_size = sizes[segment_index.min(sizes.len() - 1)];
        let end_size = sizes[(segment_index + 1).min(sizes.len() - 1)];
        
        start_size + (end_size - start_size) * local_t
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParticleSystem {
    pub emitters: HashMap<String, ParticleEmitter>,
}

impl ParticleSystem {
    pub fn new() -> Self {
        Self {
            emitters: HashMap::new(),
        }
    }

    pub fn add_emitter(&mut self, name: String, emitter: ParticleEmitter) {
        self.emitters.insert(name, emitter);
    }

    pub fn remove_emitter(&mut self, name: &str) {
        self.emitters.remove(name);
    }

    pub fn update(&mut self, dt: f32) {
        // Update all emitters
        self.emitters.retain(|_name, emitter| {
            emitter.update(dt);
            
            // Keep emitter if it's active or has living particles
            emitter.active || !emitter.particles.is_empty()
        });
    }

    pub fn get_all_particles(&self) -> Vec<&Particle> {
        self.emitters
            .values()
            .flat_map(|emitter| emitter.particles.iter())
            .collect()
    }

    // Predefined effect presets for easy use
    pub fn create_explosion(&mut self, position: Vec2, name: String) {
        let config = ParticleEmitterConfig {
            position,
            emission_rate: 0.0, // Burst only
            particle_lifetime: (0.5, 1.5),
            initial_velocity: (Vec2::new(-100.0, -100.0), Vec2::new(100.0, 100.0)),
            initial_size: (3.0, 12.0),
            color_over_life: vec![
                [1.0, 0.8, 0.2, 1.0], // Bright orange
                [1.0, 0.3, 0.1, 1.0], // Red
                [0.2, 0.2, 0.2, 0.5], // Dark smoke
                [0.0, 0.0, 0.0, 0.0], // Transparent
            ],
            size_over_life: vec![0.5, 1.0, 1.2, 0.0],
            gravity: Vec2::new(0.0, -50.0),
            spread_angle: std::f32::consts::PI * 2.0,
            burst_count: Some(50),
            texture_id: None,
        };
        
        self.add_emitter(name, ParticleEmitter::new(config));
    }

    pub fn create_magic_sparkles(&mut self, position: Vec2, name: String) {
        let config = ParticleEmitterConfig {
            position,
            emission_rate: 30.0,
            particle_lifetime: (1.0, 2.0),
            initial_velocity: (Vec2::new(-30.0, -30.0), Vec2::new(30.0, 30.0)),
            initial_size: (1.0, 4.0),
            color_over_life: vec![
                [0.8, 0.4, 1.0, 1.0], // Purple
                [0.4, 0.8, 1.0, 1.0], // Light blue
                [1.0, 1.0, 1.0, 0.8], // White
                [1.0, 1.0, 1.0, 0.0], // Fade out
            ],
            size_over_life: vec![0.0, 1.0, 0.8, 0.0],
            gravity: Vec2::new(0.0, -20.0),
            spread_angle: std::f32::consts::PI * 2.0,
            burst_count: None,
            texture_id: None,
        };
        
        self.add_emitter(name, ParticleEmitter::new(config));
    }

    pub fn create_fire(&mut self, position: Vec2, name: String) {
        let config = ParticleEmitterConfig {
            position,
            emission_rate: 40.0,
            particle_lifetime: (0.8, 1.5),
            initial_velocity: (Vec2::new(-10.0, 20.0), Vec2::new(10.0, 60.0)),
            initial_size: (4.0, 8.0),
            color_over_life: vec![
                [1.0, 0.2, 0.1, 1.0], // Bright red
                [1.0, 0.6, 0.1, 1.0], // Orange
                [1.0, 1.0, 0.3, 0.8], // Yellow
                [0.3, 0.3, 0.3, 0.2], // Smoke
                [0.0, 0.0, 0.0, 0.0], // Transparent
            ],
            size_over_life: vec![0.5, 1.0, 1.5, 1.0, 0.0],
            gravity: Vec2::new(0.0, 30.0), // Upward motion for fire
            spread_angle: std::f32::consts::PI / 4.0, // 45 degrees
            burst_count: None,
            texture_id: None,
        };
        
        self.add_emitter(name, ParticleEmitter::new(config));
    }

    pub fn create_fog(&mut self, position: Vec2, name: String) {
        let config = ParticleEmitterConfig {
            position,
            emission_rate: 15.0,
            particle_lifetime: (3.0, 5.0),
            initial_velocity: (Vec2::new(-20.0, -5.0), Vec2::new(20.0, 10.0)),
            initial_size: (20.0, 40.0),
            color_over_life: vec![
                [0.8, 0.8, 0.9, 0.0], // Transparent white-blue
                [0.8, 0.8, 0.9, 0.3], // Semi-transparent
                [0.7, 0.7, 0.8, 0.4], // Peak opacity
                [0.6, 0.6, 0.7, 0.2], // Fade
                [0.5, 0.5, 0.6, 0.0], // Transparent
            ],
            size_over_life: vec![0.0, 0.5, 1.0, 1.2, 0.8],
            gravity: Vec2::new(0.0, 5.0), // Slight upward drift
            spread_angle: std::f32::consts::PI / 6.0, // 30 degrees
            burst_count: None,
            texture_id: None,
        };
        
        self.add_emitter(name, ParticleEmitter::new(config));
    }
}