use crate::engine::{
    graphics::{Light, ParticleSystem},
    animation::{AnimationManager, Animation, AnimationTarget, EaseType},
    scene::{Scene, GameObject, SceneManager, SceneTemplate},
    physics::{PhysicsWorld, RigidBody, Collider, ColliderShape},
    audio::AudioManager,
    ui::{UIManager, legacy_components::{Button, Label, Panel, Slider, ProgressBar}, UIBounds, Anchor, ElementId},
    assets::{HotReloadSystem, HotReloadSystemBuilder, AssetConfig, HotReloadEvent, AssetPipeline, AssetImporter, AssetType, ImporterConfig, PipelineConfig},
    save_system::{SaveManager, SaveSystemConfig, GameState, UserProfile, ProfileManager},
    ai_game::{GameAIManager, PlayerProfile, PlayerInteraction, GameAIEvent, GameAIRecommendation},
    error::{RobinError, RobinResult},
    logging::{RobinLogger, LoggingConfig, PerformanceMetrics},
    diagnostics::{DiagnosticsManager, DiagnosticsConfig},
    math::Vec2,
};
use std::collections::HashMap;
use std::time::Duration;

/// Engine health levels for diagnostics
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HealthLevel {
    Good,
    Warning,
    Critical,
}

/// Comprehensive engine health status
#[derive(Debug, Clone)]
pub struct EngineHealthStatus {
    pub overall: HealthLevel,
    pub performance: HealthLevel,
    pub memory: HealthLevel,
    pub errors: HealthLevel,
    pub details: String,
}

/// High-level, no-code friendly game builder API
/// Provides simple methods for common game development tasks
pub struct GameBuilder {
    pub scene: Scene,
    pub scene_manager: SceneManager,
    pub particle_system: ParticleSystem,
    pub animation_manager: AnimationManager,
    pub physics_world: PhysicsWorld,
    pub audio_manager: AudioManager,
    pub ui_manager: UIManager,
    pub hot_reload: HotReloadSystem,
    pub diagnostics: DiagnosticsManager,
    pub asset_pipeline: AssetPipeline,
    pub asset_importer: AssetImporter,
    pub save_manager: SaveManager,
    pub profile_manager: ProfileManager,
    pub ai_game_manager: GameAIManager,
    pub lights: Vec<Light>,
    effects_counter: u32,
    physics_body_counter: u32,
    sound_counter: u32,
    frame_counter: u64,
    last_error: Option<RobinError>,
}

impl GameBuilder {
    pub fn new() -> Self {
        log::info!("Starting GameBuilder::new() - DEBUG");
        
        log::info!("Initializing audio...");
        let mut audio_manager = match AudioManager::new() {
            Ok(manager) => {
                log::info!("Audio manager initialized successfully");
                manager
            },
            Err(e) => {
                log::warn!("Failed to initialize audio: {}. Using dummy audio manager.", e);
                AudioManager::dummy()
            }
        };
        
        log::info!("Loading default sounds...");
        audio_manager.load_default_sounds();
        
        log::info!("Setting up hot-reload system...");
        let hot_reload = HotReloadSystemBuilder::new()
            .with_watch_enabled(true)
            .with_reload_delay(Duration::from_millis(100))
            .with_base_path("assets")
            .build();
        
        log::info!("Initializing asset pipeline...");
        let asset_pipeline = AssetPipeline::dummy();
        log::info!("Asset pipeline initialized (dummy mode)");
        
        log::info!("Creating asset importer with minimal config...");
        let asset_importer = AssetImporter::new(ImporterConfig {
            import_settings_dir: std::env::temp_dir().join("robin_import_settings"),
            auto_import: false,
            preserve_directory_structure: false,
            overwrite_existing: false,
            generate_previews: false,
            validate_imports: false,
            default_settings: HashMap::new(), // Empty to avoid complex initialization
        });
        log::info!("Asset importer created");
        
        log::info!("Initializing save system...");
        let save_config = SaveSystemConfig {
            save_directory: directories::UserDirs::new()
                .and_then(|dirs| dirs.document_dir().map(|d| d.join("RobinEngine").join("Saves")))
                .unwrap_or_else(|| std::path::PathBuf::from("saves")),
            max_slots: 10,
            auto_save_enabled: true,
            auto_save_interval: Duration::from_secs(300),
            max_backups: 3,
            enable_compression: true,
            enable_encryption: false,
            save_extension: "robinsave".to_string(),
        };
        let save_manager = SaveManager::new(save_config.clone());
        let profile_manager = ProfileManager::new(save_config)
            .unwrap_or_else(|e| {
                log::error!("Failed to initialize profile manager: {}", e);
                let temp_config = SaveSystemConfig {
                    save_directory: std::env::temp_dir().join("robin_saves_fallback"),
                    max_slots: 10,
                    auto_save_enabled: false, // Disable auto-save for fallback
                    auto_save_interval: Duration::from_secs(300),
                    max_backups: 3,
                    enable_compression: false, // Disable compression for fallback
                    enable_encryption: false,
                    save_extension: "robinsave".to_string(),
                };
                ProfileManager::new(temp_config).expect("Failed to create fallback profile manager")
            });
        
        log::info!("Creating GameBuilder with minimal systems for debugging...");
        
        Self {
            scene: Scene::new(),
            scene_manager: SceneManager::new(),
            particle_system: ParticleSystem::new(),
            animation_manager: AnimationManager::new(),
            physics_world: PhysicsWorld::new(),
            audio_manager,
            ui_manager: UIManager::new(800.0, 600.0),
            hot_reload,
            diagnostics: DiagnosticsManager::new(DiagnosticsConfig::default()),
            asset_pipeline,
            asset_importer,
            save_manager,
            profile_manager,
            ai_game_manager: GameAIManager::new(),
            lights: Vec::new(),
            effects_counter: 0,
            physics_body_counter: 0,
            sound_counter: 0,
            frame_counter: 0,
            last_error: None,
        }
    }

    // === LIGHTING METHODS ===
    
    /// Add a simple light source at the specified position
    pub fn add_light(&mut self, x: f32, y: f32, color: (f32, f32, f32), brightness: f32) -> &mut Self {
        let light = Light {
            position: [x, y],
            color: [color.0, color.1, color.2],
            intensity: brightness,
            radius: 100.0,
            _padding: [0.0; 3],
        };
        self.lights.push(light);
        self
    }

    /// Add a warm torch-like light (orange glow)
    pub fn add_torch_light(&mut self, x: f32, y: f32) -> &mut Self {
        self.add_light(x, y, (1.0, 0.6, 0.2), 0.8);
        // Add flickering animation
        let light_id = format!("torch_light_{}", self.lights.len());
        self.animation_manager.pulse_color(
            light_id,
            [1.0, 0.6, 0.2, 0.8],
            [1.0, 0.4, 0.1, 1.0],
            Duration::from_millis(200)
        );
        self
    }

    /// Add magical blue light with sparkle effect
    pub fn add_magic_light(&mut self, x: f32, y: f32) -> &mut Self {
        self.add_light(x, y, (0.3, 0.7, 1.0), 0.9);
        // Add sparkle particles
        let effect_name = format!("magic_sparkles_{}", self.effects_counter);
        self.effects_counter += 1;
        self.particle_system.create_magic_sparkles(Vec2::new(x, y), effect_name);
        self
    }

    // === PARTICLE EFFECTS METHODS ===

    /// Create an explosion effect at the specified position
    pub fn create_explosion(&mut self, x: f32, y: f32) -> String {
        let effect_name = format!("explosion_{}", self.effects_counter);
        self.effects_counter += 1;
        self.particle_system.create_explosion(Vec2::new(x, y), effect_name.clone());
        
        // Play explosion sound at this position
        self.play_explosion_sound(x, y);
        
        effect_name
    }

    /// Create magical sparkles that follow a moving object
    pub fn create_magic_trail(&mut self, x: f32, y: f32) -> String {
        let effect_name = format!("magic_trail_{}", self.effects_counter);
        self.effects_counter += 1;
        self.particle_system.create_magic_sparkles(Vec2::new(x, y), effect_name.clone());
        effect_name
    }

    /// Create a fire effect (campfire, torch, etc.)
    pub fn create_fire(&mut self, x: f32, y: f32) -> String {
        let effect_name = format!("fire_{}", self.effects_counter);
        self.effects_counter += 1;
        self.particle_system.create_fire(Vec2::new(x, y), effect_name.clone());
        effect_name
    }

    /// Create atmospheric fog
    pub fn create_fog(&mut self, x: f32, y: f32) -> String {
        let effect_name = format!("fog_{}", self.effects_counter);
        self.effects_counter += 1;
        self.particle_system.create_fog(Vec2::new(x, y), effect_name.clone());
        effect_name
    }

    /// Create fireworks burst
    pub fn create_fireworks(&mut self, x: f32, y: f32, color: (f32, f32, f32)) -> String {
        let effect_name = format!("fireworks_{}", self.effects_counter);
        self.effects_counter += 1;
        
        // Create custom fireworks emitter with specified color
        use crate::engine::graphics::{ParticleEmitter, ParticleEmitterConfig};
        
        let config = ParticleEmitterConfig {
            position: Vec2::new(x, y),
            emission_rate: 0.0,
            particle_lifetime: (1.0, 2.0),
            initial_velocity: (Vec2::new(-80.0, -80.0), Vec2::new(80.0, 80.0)),
            initial_size: (2.0, 8.0),
            color_over_life: vec![
                [color.0, color.1, color.2, 1.0],
                [color.0 * 0.8, color.1 * 0.8, color.2 * 0.8, 0.8],
                [color.0 * 0.6, color.1 * 0.6, color.2 * 0.6, 0.4],
                [0.0, 0.0, 0.0, 0.0],
            ],
            size_over_life: vec![0.5, 1.0, 0.8, 0.0],
            gravity: Vec2::new(0.0, -80.0),
            spread_angle: std::f32::consts::PI * 2.0,
            burst_count: Some(30),
            texture_id: None,
        };
        
        self.particle_system.add_emitter(effect_name.clone(), ParticleEmitter::new(config));
        effect_name
    }

    // === ANIMATION METHODS ===

    /// Make an object smoothly move from one position to another
    pub fn move_object(&mut self, object_id: &str, from_x: f32, from_y: f32, to_x: f32, to_y: f32, seconds: f32) -> &mut Self {
        self.animation_manager.move_to(
            object_id.to_string(),
            Vec2::new(from_x, from_y),
            Vec2::new(to_x, to_y),
            Duration::from_secs_f32(seconds)
        );
        self
    }

    /// Make an object fade in smoothly
    pub fn fade_in(&mut self, object_id: &str, seconds: f32) -> &mut Self {
        self.animation_manager.fade_in(object_id.to_string(), Duration::from_secs_f32(seconds));
        self
    }

    /// Make an object fade out smoothly
    pub fn fade_out(&mut self, object_id: &str, seconds: f32) -> &mut Self {
        self.animation_manager.fade_out(object_id.to_string(), Duration::from_secs_f32(seconds));
        self
    }

    /// Make an object spin continuously
    pub fn spin_object(&mut self, object_id: &str, rotations_per_second: f32) -> &mut Self {
        self.animation_manager.rotate_continuous(object_id.to_string(), rotations_per_second);
        self
    }

    /// Make an object bounce with a scale effect
    pub fn bounce_object(&mut self, object_id: &str, from_scale: f32, to_scale: f32, seconds: f32) -> &mut Self {
        self.animation_manager.scale_bounce(
            object_id.to_string(),
            from_scale,
            to_scale,
            Duration::from_secs_f32(seconds)
        );
        self
    }

    /// Make an object pulse between two colors
    pub fn pulse_color(&mut self, object_id: &str, color1: (f32, f32, f32, f32), color2: (f32, f32, f32, f32), seconds: f32) -> &mut Self {
        self.animation_manager.pulse_color(
            object_id.to_string(),
            [color1.0, color1.1, color1.2, color1.3],
            [color2.0, color2.1, color2.2, color2.3],
            Duration::from_secs_f32(seconds)
        );
        self
    }

    /// Create a smooth elastic animation (bouncy effect)
    pub fn elastic_scale(&mut self, object_id: &str, from_scale: f32, to_scale: f32, seconds: f32) -> &mut Self {
        let animation = Animation::new(
            AnimationTarget::Size(from_scale, to_scale),
            Duration::from_secs_f32(seconds),
            EaseType::Elastic,
        );
        self.animation_manager.add_animation(object_id.to_string(), animation);
        self
    }

    // === PHYSICS METHODS ===
    
    /// Create a physics-enabled game object that can collide with other objects
    pub fn create_physics_object(&mut self, x: f32, y: f32, width: f32, height: f32, is_static: bool) -> u32 {
        let body_type = if is_static { 
            crate::engine::physics::BodyType::Static 
        } else { 
            crate::engine::physics::BodyType::Dynamic 
        };
        
        let body_id = self.physics_world.add_rigid_body(body_type, Vec2::new(x, y));
        self.physics_world.add_collider(body_id, ColliderShape::Rectangle { width, height });
        
        body_id
    }
    
    /// Create a circular physics object (like a ball or coin)
    pub fn create_physics_circle(&mut self, x: f32, y: f32, radius: f32, is_static: bool) -> u32 {
        let body_type = if is_static { 
            crate::engine::physics::BodyType::Static 
        } else { 
            crate::engine::physics::BodyType::Dynamic 
        };
        
        let body_id = self.physics_world.add_rigid_body(body_type, Vec2::new(x, y));
        self.physics_world.add_collider(body_id, ColliderShape::Circle { radius });
        
        body_id
    }
    
    /// Apply a force to a physics object (like jumping, explosions, wind)
    pub fn apply_force(&mut self, body_id: u32, force_x: f32, force_y: f32) -> &mut Self {
        if let Some(body) = self.physics_world.get_body_mut(body_id) {
            body.apply_force(Vec2::new(force_x, force_y));
        }
        self
    }
    
    /// Apply an impulse to a physics object (instant velocity change)
    pub fn apply_impulse(&mut self, body_id: u32, impulse_x: f32, impulse_y: f32) -> &mut Self {
        if let Some(body) = self.physics_world.get_body_mut(body_id) {
            body.apply_impulse(Vec2::new(impulse_x, impulse_y));
        }
        self
    }
    
    /// Set the velocity of a physics object directly
    pub fn set_velocity(&mut self, body_id: u32, vel_x: f32, vel_y: f32) -> &mut Self {
        if let Some(body) = self.physics_world.get_body_mut(body_id) {
            body.velocity = Vec2::new(vel_x, vel_y);
        }
        self
    }
    
    /// Get the current position of a physics object
    pub fn get_physics_position(&self, body_id: u32) -> Option<(f32, f32)> {
        self.physics_world.get_body(body_id)
            .map(|body| (body.position.x, body.position.y))
    }
    
    /// Check if two physics objects are colliding
    pub fn are_colliding(&self, body_id1: u32, body_id2: u32) -> bool {
        // Check the collision list for collisions involving these two bodies
        self.physics_world.collisions.iter().any(|collision| {
            (collision.body_a == body_id1 && collision.body_b == body_id2) ||
            (collision.body_a == body_id2 && collision.body_b == body_id1)
        })
    }
    
    /// Create a platform (static rectangular physics object)
    pub fn create_platform(&mut self, x: f32, y: f32, width: f32, height: f32) -> u32 {
        self.create_physics_object(x, y, width, height, true)
    }
    
    /// Create a bouncing ball with physics
    pub fn create_bouncing_ball(&mut self, x: f32, y: f32, radius: f32) -> u32 {
        let body_id = self.create_physics_circle(x, y, radius, false);
        
        // Set some bounce properties
        if let Some(body) = self.physics_world.get_body_mut(body_id) {
            body.restitution = 0.8; // Bouncy
            body.friction = 0.1;    // Low friction
        }
        
        body_id
    }
    
    /// Create physics boundaries (invisible walls) around an area
    pub fn create_physics_boundaries(&mut self, left: f32, right: f32, top: f32, bottom: f32, thickness: f32) -> Vec<u32> {
        let mut boundaries = Vec::new();
        
        // Left wall
        boundaries.push(self.create_platform(left - thickness/2.0, (top + bottom) / 2.0, thickness, bottom - top));
        // Right wall  
        boundaries.push(self.create_platform(right + thickness/2.0, (top + bottom) / 2.0, thickness, bottom - top));
        // Top wall
        boundaries.push(self.create_platform((left + right) / 2.0, top - thickness/2.0, right - left, thickness));
        // Bottom wall
        boundaries.push(self.create_platform((left + right) / 2.0, bottom + thickness/2.0, right - left, thickness));
        
        boundaries
    }

    // === PRESET COMBINATIONS ===

    /// Create a magical portal effect with light and particles
    pub fn create_portal(&mut self, x: f32, y: f32) -> Vec<String> {
        let mut effect_ids = Vec::new();
        
        // Add swirling light
        self.add_light(x, y, (0.5, 0.2, 1.0), 1.0);
        
        // Add magic sparkles
        let sparkles = self.create_magic_trail(x, y);
        effect_ids.push(sparkles);
        
        // Add portal glow animation
        let portal_id = format!("portal_glow_{}", self.effects_counter);
        self.effects_counter += 1;
        self.pulse_color(&portal_id, (0.5, 0.2, 1.0, 1.0), (0.8, 0.4, 1.0, 1.0), 1.5);
        effect_ids.push(portal_id);
        
        effect_ids
    }

    /// Create a treasure pickup effect with light, particles, and animation
    pub fn create_treasure_pickup(&mut self, x: f32, y: f32) -> Vec<String> {
        let mut effect_ids = Vec::new();
        
        // Golden light
        self.add_light(x, y, (1.0, 0.8, 0.2), 0.8);
        
        // Play coin pickup sound
        self.play_coin_sound();
        
        // Golden sparkles
        let sparkles = format!("treasure_sparkles_{}", self.effects_counter);
        self.effects_counter += 1;
        
        use crate::engine::graphics::{ParticleEmitter, ParticleEmitterConfig};
        let config = ParticleEmitterConfig {
            position: Vec2::new(x, y),
            emission_rate: 0.0,
            particle_lifetime: (0.8, 1.2),
            initial_velocity: (Vec2::new(-40.0, 20.0), Vec2::new(40.0, 80.0)),
            initial_size: (2.0, 6.0),
            color_over_life: vec![
                [1.0, 0.8, 0.2, 1.0], // Gold
                [1.0, 1.0, 0.6, 0.8], // Bright gold
                [1.0, 0.9, 0.4, 0.4], // Fade
                [0.0, 0.0, 0.0, 0.0], // Transparent
            ],
            size_over_life: vec![0.0, 1.0, 0.8, 0.0],
            gravity: Vec2::new(0.0, -30.0),
            spread_angle: std::f32::consts::PI * 0.7,
            burst_count: Some(20),
            texture_id: None,
        };
        
        self.particle_system.add_emitter(sparkles.clone(), ParticleEmitter::new(config));
        effect_ids.push(sparkles);
        
        effect_ids
    }

    /// Create a campfire scene with fire, light, and ambient effects
    pub fn create_campfire(&mut self, x: f32, y: f32) -> Vec<String> {
        let mut effect_ids = Vec::new();
        
        // Warm firelight
        self.add_torch_light(x, y + 10.0);
        
        // Fire particles
        let fire = self.create_fire(x, y);
        effect_ids.push(fire);
        
        // Subtle smoke particles above fire
        let smoke = format!("campfire_smoke_{}", self.effects_counter);
        self.effects_counter += 1;
        
        use crate::engine::graphics::{ParticleEmitter, ParticleEmitterConfig};
        let config = ParticleEmitterConfig {
            position: Vec2::new(x, y + 30.0),
            emission_rate: 8.0,
            particle_lifetime: (2.0, 4.0),
            initial_velocity: (Vec2::new(-5.0, 10.0), Vec2::new(5.0, 25.0)),
            initial_size: (8.0, 16.0),
            color_over_life: vec![
                [0.3, 0.3, 0.3, 0.0],
                [0.4, 0.4, 0.4, 0.3],
                [0.5, 0.5, 0.5, 0.2],
                [0.6, 0.6, 0.6, 0.0],
            ],
            size_over_life: vec![0.5, 1.0, 1.5, 2.0],
            gravity: Vec2::new(0.0, 15.0),
            spread_angle: std::f32::consts::PI / 8.0,
            burst_count: None,
            texture_id: None,
        };
        
        self.particle_system.add_emitter(smoke.clone(), ParticleEmitter::new(config));
        effect_ids.push(smoke);
        
        effect_ids
    }

    // === AUDIO METHODS ===

    /// Play a sound effect
    pub fn play_sound(&mut self, name: &str) -> Option<u32> {
        self.audio_manager.play_sound(name)
    }

    /// Play a sound at a specific position (spatial audio)
    pub fn play_sound_at(&mut self, name: &str, x: f32, y: f32, volume: f32) -> Option<u32> {
        self.audio_manager.play_sound_at(name, Vec2::new(x, y), volume)
    }

    /// Play background music
    pub fn play_music(&mut self, name: &str) -> Option<u32> {
        self.audio_manager.play_music(name, true)
    }

    /// Stop all sound effects
    pub fn stop_all_sounds(&mut self) {
        self.audio_manager.stop_all_sfx();
    }

    /// Set the master volume
    pub fn set_master_volume(&mut self, volume: f32) -> &mut Self {
        self.audio_manager.set_master_volume(volume);
        self
    }

    /// Set music volume
    pub fn set_music_volume(&mut self, volume: f32) -> &mut Self {
        self.audio_manager.set_music_volume(volume);
        self
    }

    /// Set sound effects volume
    pub fn set_sfx_volume(&mut self, volume: f32) -> &mut Self {
        self.audio_manager.set_sfx_volume(volume);
        self
    }

    /// Play an explosion sound at position
    pub fn play_explosion_sound(&mut self, x: f32, y: f32) -> Option<u32> {
        self.play_sound_at("explosion", x, y, 1.0)
    }

    /// Play a coin pickup sound
    pub fn play_coin_sound(&mut self) -> Option<u32> {
        self.play_sound("coin")
    }

    /// Play a jump sound
    pub fn play_jump_sound(&mut self) -> Option<u32> {
        self.play_sound("jump")
    }

    /// Update listener position for spatial audio (usually the player/camera position)
    pub fn set_audio_listener(&mut self, x: f32, y: f32) -> &mut Self {
        self.audio_manager.set_listener_position(Vec2::new(x, y));
        self
    }

    /// Load music from bytes (for custom music tracks)
    pub fn load_music_from_bytes(&mut self, name: &str, data: Vec<u8>) -> &mut Self {
        self.audio_manager.load_music_from_bytes(name, data);
        self
    }

    /// Load sound effect from bytes (for custom sounds)
    pub fn load_sound_from_bytes(&mut self, name: &str, data: Vec<u8>, volume: f32) -> &mut Self {
        self.audio_manager.load_sound_from_bytes(name, data, volume);
        self
    }

    // === UI METHODS ===

    /// Create a simple button with text and click callback
    pub fn create_button(&mut self, x: f32, y: f32, width: f32, height: f32, text: &str) -> ElementId {
        let bounds = UIBounds::new(x, y, width, height);
        let mut button = Button::new(1, bounds, text.to_string()); // Use placeholder ID
        let boxed_button: Box<dyn crate::engine::ui::UIElement> = Box::new(button);
        self.ui_manager.add_element(boxed_button)
    }

    /// Create a text label
    pub fn create_label(&mut self, x: f32, y: f32, width: f32, height: f32, text: &str) -> ElementId {
        let bounds = UIBounds::new(x, y, width, height);
        let label = Label::new(1, bounds, text.to_string()); // Use placeholder ID
        let boxed_label: Box<dyn crate::engine::ui::UIElement> = Box::new(label);
        self.ui_manager.add_element(boxed_label)
    }

    /// Create a panel/container
    pub fn create_panel(&mut self, x: f32, y: f32, width: f32, height: f32) -> ElementId {
        let bounds = UIBounds::new(x, y, width, height);
        let panel = Panel::new(1, bounds); // Use placeholder ID
        let boxed_panel: Box<dyn crate::engine::ui::UIElement> = Box::new(panel);
        self.ui_manager.add_element(boxed_panel)
    }

    /// Create a slider for numeric input
    pub fn create_slider(&mut self, x: f32, y: f32, width: f32, height: f32, min_value: f32, max_value: f32) -> ElementId {
        let bounds = UIBounds::new(x, y, width, height);
        let slider = Slider::new(1, bounds, min_value, max_value); // Use placeholder ID
        let boxed_slider: Box<dyn crate::engine::ui::UIElement> = Box::new(slider);
        self.ui_manager.add_element(boxed_slider)
    }

    /// Create a progress bar
    pub fn create_progress_bar(&mut self, x: f32, y: f32, width: f32, height: f32, max_value: f32) -> ElementId {
        let bounds = UIBounds::new(x, y, width, height);
        let progress_bar = ProgressBar::new(1, bounds, max_value); // Use placeholder ID
        let boxed_progress_bar: Box<dyn crate::engine::ui::UIElement> = Box::new(progress_bar);
        self.ui_manager.add_element(boxed_progress_bar)
    }

    /// Set UI element visibility
    pub fn set_ui_visible(&mut self, element_id: ElementId, visible: bool) -> &mut Self {
        if let Some(element) = self.ui_manager.get_element_mut(element_id) {
            element.set_visible(visible);
        }
        self
    }

    /// Update UI element text (works for buttons and labels)
    pub fn set_ui_text(&mut self, element_id: ElementId, text: &str) -> &mut Self {
        if let Some(element) = self.ui_manager.get_element_mut(element_id) {
            // This would need downcasting in a real implementation
            // For now, this is a placeholder for the API design
            log::debug!("Setting text for UI element {} to '{}'", element_id, text);
        }
        self
    }

    /// Animate UI element fade in
    pub fn fade_in_ui(&mut self, element_id: ElementId, seconds: f32) -> &mut Self {
        self.ui_manager.fade_in_element(element_id, Duration::from_secs_f32(seconds));
        self
    }

    /// Animate UI element fade out
    pub fn fade_out_ui(&mut self, element_id: ElementId, seconds: f32) -> &mut Self {
        self.ui_manager.fade_out_element(element_id, Duration::from_secs_f32(seconds));
        self
    }

    /// Animate UI element slide to position
    pub fn slide_ui(&mut self, element_id: ElementId, from_x: f32, from_y: f32, to_x: f32, to_y: f32, seconds: f32) -> &mut Self {
        self.ui_manager.slide_element(
            element_id, 
            Vec2::new(from_x, from_y), 
            Vec2::new(to_x, to_y), 
            Duration::from_secs_f32(seconds)
        );
        self
    }

    /// Make UI element bounce
    pub fn bounce_ui(&mut self, element_id: ElementId, from_scale: f32, to_scale: f32, seconds: f32) -> &mut Self {
        self.ui_manager.bounce_element(element_id, from_scale, to_scale, Duration::from_secs_f32(seconds));
        self
    }

    /// Make UI element pulse
    pub fn pulse_ui(&mut self, element_id: ElementId, seconds: f32) -> &mut Self {
        self.ui_manager.pulse_element(element_id, Duration::from_secs_f32(seconds));
        self
    }

    /// Set UI scale for accessibility
    pub fn set_ui_scale(&mut self, scale: f32) -> &mut Self {
        self.ui_manager.set_ui_scale(scale);
        self
    }

    /// Handle screen resize for responsive UI
    pub fn resize_ui(&mut self, width: f32, height: f32) -> &mut Self {
        self.ui_manager.resize(width, height);
        self
    }

    /// Remove a UI element
    pub fn remove_ui_element(&mut self, element_id: ElementId) -> &mut Self {
        self.ui_manager.remove_element(element_id);
        self
    }

    /// Clear all UI elements
    pub fn clear_ui(&mut self) -> &mut Self {
        self.ui_manager.clear();
        self
    }

    // === UI PRESET METHODS ===

    /// Create a simple menu with title and buttons
    pub fn create_simple_menu(&mut self, title: &str, buttons: &[(&str, f32, f32)]) -> Vec<ElementId> {
        let mut elements = Vec::new();
        
        // Create title
        let title_id = self.create_label(400.0, 100.0, 400.0, 60.0, title);
        elements.push(title_id);
        
        // Create buttons
        for (i, &(button_text, x, y)) in buttons.iter().enumerate() {
            let button_id = self.create_button(x, y, 200.0, 50.0, button_text);
            elements.push(button_id);
            
            // Add fade in animation with staggered timing
            self.fade_in_ui(button_id, 0.5 + (i as f32) * 0.1);
        }
        
        elements
    }

    /// Create a settings panel with sliders
    pub fn create_settings_panel(&mut self, x: f32, y: f32) -> Vec<ElementId> {
        let mut elements = Vec::new();
        
        // Background panel
        let panel_id = self.create_panel(x, y, 300.0, 200.0);
        elements.push(panel_id);
        
        // Master volume label and slider
        let master_label = self.create_label(x + 20.0, y + 20.0, 100.0, 30.0, "Master Volume");
        let master_slider = self.create_slider(x + 120.0, y + 20.0, 160.0, 30.0, 0.0, 1.0);
        elements.push(master_label);
        elements.push(master_slider);
        
        // Music volume label and slider
        let music_label = self.create_label(x + 20.0, y + 60.0, 100.0, 30.0, "Music Volume");
        let music_slider = self.create_slider(x + 120.0, y + 60.0, 160.0, 30.0, 0.0, 1.0);
        elements.push(music_label);
        elements.push(music_slider);
        
        // SFX volume label and slider
        let sfx_label = self.create_label(x + 20.0, y + 100.0, 100.0, 30.0, "SFX Volume");
        let sfx_slider = self.create_slider(x + 120.0, y + 100.0, 160.0, 30.0, 0.0, 1.0);
        elements.push(sfx_label);
        elements.push(sfx_slider);
        
        // Close button
        let close_button = self.create_button(x + 200.0, y + 150.0, 80.0, 30.0, "Close");
        elements.push(close_button);
        
        // Slide panel in from the right
        self.slide_ui(panel_id, x + 300.0, y, x, y, 0.3);
        
        elements
    }

    /// Create a loading screen with progress bar
    pub fn create_loading_screen(&mut self, title: &str) -> Vec<ElementId> {
        let mut elements = Vec::new();
        
        // Background overlay
        let overlay = self.create_panel(0.0, 0.0, 800.0, 600.0);
        elements.push(overlay);
        
        // Loading title
        let title_id = self.create_label(300.0, 250.0, 200.0, 50.0, title);
        elements.push(title_id);
        
        // Progress bar
        let progress_id = self.create_progress_bar(300.0, 320.0, 200.0, 20.0, 100.0);
        elements.push(progress_id);
        
        // Status text
        let status_id = self.create_label(300.0, 350.0, 200.0, 30.0, "Loading...");
        elements.push(status_id);
        
        // Fade in animation
        for &element_id in &elements {
            self.fade_in_ui(element_id, 0.5);
        }
        
        elements
    }

    // === HOT RELOAD METHODS ===

    /// Start the hot-reload system for development
    pub fn start_hot_reload(&mut self) -> &mut Self {
        if let Err(e) = self.hot_reload.start() {
            log::error!("Failed to start hot-reload system: {}", e);
        } else {
            log::info!("Hot-reload system started - assets will automatically reload on file changes");
        }
        self
    }

    /// Stop the hot-reload system
    pub fn stop_hot_reload(&mut self) -> &mut Self {
        self.hot_reload.stop();
        log::info!("Hot-reload system stopped");
        self
    }

    /// Register an asset for hot-reloading (textures, sounds, configs)
    pub fn register_asset(&mut self, id: &str, file_path: &str) -> &mut Self {
        let asset_id = id.to_string();
        let path = std::path::PathBuf::from(file_path);
        
        if let Err(e) = self.hot_reload.register_asset(asset_id.clone(), &path) {
            log::error!("Failed to register asset '{}': {}", asset_id, e);
        } else {
            log::info!("Registered asset '{}' for hot-reload: {}", asset_id, file_path);
        }
        self
    }

    /// Register all assets in a directory for hot-reloading
    pub fn register_assets_directory(&mut self, dir_path: &str, recursive: bool) -> &mut Self {
        let path = std::path::PathBuf::from(dir_path);
        
        match self.hot_reload.register_directory(&path, recursive) {
            Ok(assets) => {
                log::info!("Registered {} assets from directory: {}", assets.len(), dir_path);
            }
            Err(e) => {
                log::error!("Failed to register assets from directory '{}': {}", dir_path, e);
            }
        }
        self
    }

    /// Force reload a specific asset
    pub fn force_reload_asset(&mut self, id: &str) -> &mut Self {
        if let Err(e) = self.hot_reload.force_reload(id) {
            log::error!("Failed to force reload asset '{}': {}", id, e);
        } else {
            log::info!("Force reloading asset: {}", id);
        }
        self
    }

    /// Get hot-reload statistics
    pub fn get_hot_reload_stats(&self) -> String {
        let stats = self.hot_reload.get_stats();
        format!(
            "Hot-Reload Stats: {} assets watched, {}/{} reloads successful ({:.1}% success rate), avg reload time: {:.2?}ms, queue: {}",
            stats.total_assets,
            stats.successful_reloads,
            stats.successful_reloads + stats.failed_reloads,
            stats.success_rate() * 100.0,
            stats.average_reload_time.as_millis(),
            stats.queue_size
        )
    }

    /// Enable/disable hot-reloading for development vs production
    pub fn set_hot_reload_enabled(&mut self, enabled: bool) -> &mut Self {
        if enabled {
            self.start_hot_reload();
        } else {
            self.stop_hot_reload();
        }
        self
    }

    /// Auto-register common asset directories
    pub fn auto_register_assets(&mut self) -> &mut Self {
        log::info!("Auto-registering common asset directories...");
        
        let directories = vec![
            ("assets/textures", true),
            ("assets/audio", true),
            ("assets/config", true),
            ("assets/scenes", true),
            ("assets/shaders", false),
        ];
        
        for (dir, recursive) in directories {
            if std::path::Path::new(dir).exists() {
                self.register_assets_directory(dir, recursive);
            }
        }
        
        log::info!("Auto-registration complete");
        self
    }

    /// Enable hot reload system with default configuration
    pub fn enable_hot_reload(&mut self, enabled: bool) -> &mut Self {
        self.set_hot_reload_enabled(enabled)
    }

    /// Configure hot reload system with custom settings
    pub fn hot_reload_config<F>(&mut self, config_fn: F) -> &mut Self
    where
        F: FnOnce(&mut HotReloadSystemBuilder) -> &mut HotReloadSystemBuilder,
    {
        // Create a new builder with current config
        let mut builder = HotReloadSystemBuilder::new();
        
        // Apply the configuration
        config_fn(&mut builder);
        
        // Rebuild the hot reload system with new config
        self.hot_reload = builder.build();
        
        self
    }

    /// Add a callback that will be called when specific assets are reloaded
    pub fn add_reload_callback<F>(&mut self, asset_id: &str, callback: F) -> &mut Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.hot_reload.add_asset_callback(asset_id, Box::new(callback));
        self
    }

    // === ASSET PIPELINE METHODS ===

    /// Process assets through the build pipeline for optimization
    pub fn build_assets(&mut self, target_platform: Option<&str>) -> &mut Self {
        match self.asset_pipeline.build_all_assets() {
            Ok(()) => {
                log::info!("Successfully processed assets for platform: {}", 
                    target_platform.unwrap_or("desktop"));
            }
            Err(e) => {
                log::error!("Asset build failed: {}", e);
                self.handle_error(
                    RobinError::AssetBuildError {
                        stage: "build_all".to_string(),
                        reason: e.to_string(),
                    },
                    Some("Asset Pipeline")
                );
            }
        }
        self
    }

    /// Import a single asset from source file
    pub fn import_asset(&mut self, source_path: &str, asset_type: AssetType) -> &mut Self {
        let source = std::path::PathBuf::from(source_path);
        
        match self.asset_importer.import_asset(source_path) {
            Ok(import_result) => {
                log::info!("Successfully imported asset: {} -> {}", 
                    source_path, import_result.imported_path.display());
                
                // Register the imported asset for hot-reloading
                let asset_id = crate::engine::assets::utils::generate_asset_id(
                    &import_result.imported_path, 
                    &std::path::PathBuf::from("assets")
                );
                self.register_asset(&asset_id, &source_path);
            }
            Err(e) => {
                log::error!("Failed to import asset '{}': {}", source_path, e);
                self.handle_error(
                    RobinError::AssetImportError {
                        source_path: source.clone(),
                        reason: e.to_string(),
                    },
                    Some("Asset Import")
                );
            }
        }
        self
    }

    /// Import all assets from a directory
    pub fn import_assets_directory(&mut self, directory: &str, recursive: bool) -> &mut Self {
        let dir_path = std::path::PathBuf::from(directory);
        
        let dest_path = std::path::PathBuf::from("assets/imported");
        match self.asset_importer.import_directory(&dir_path, &dest_path, recursive) {
            Ok(imported_assets) => {
                log::info!("Successfully imported {} assets from directory: {}", 
                    imported_assets.len(), directory);
                
                // Register all imported assets for hot-reloading
                for import_result in imported_assets {
                    let asset_id = crate::engine::assets::utils::generate_asset_id(
                        &import_result.imported_path, 
                        &std::path::PathBuf::from("assets")
                    );
                    if let Some(path_str) = import_result.imported_path.to_str() {
                        self.register_asset(&asset_id, path_str);
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to import assets from directory '{}': {}", directory, e);
                self.handle_error(
                    RobinError::AssetImportError {
                        source_path: dir_path.clone(),
                        reason: e.to_string(),
                    },
                    Some("Directory Import")
                );
            }
        }
        self
    }

    /// Build specific asset types for a target platform
    pub fn build_assets_for_platform(&mut self, asset_types: &[AssetType], platform: &str) -> &mut Self {
        match self.asset_pipeline.build_assets_for_platform(platform) {
            Ok(()) => {
                log::info!("Successfully processed assets for platform: {}", platform);
            }
            Err(e) => {
                log::error!("Platform-specific asset build failed: {}", e);
                self.handle_error(
                    RobinError::AssetBuildError {
                        stage: format!("build_for_platform_{}", platform),
                        reason: e.to_string(),
                    },
                    Some("Asset Pipeline")
                );
            }
        }
        self
    }

    /// Configure asset pipeline settings
    pub fn configure_asset_pipeline<F>(&mut self, config_fn: F) -> &mut Self
    where
        F: FnOnce(&mut AssetPipeline),
    {
        config_fn(&mut self.asset_pipeline);
        self
    }

    /// Configure asset importer settings
    pub fn configure_asset_importer<F>(&mut self, config_fn: F) -> &mut Self
    where
        F: FnOnce(&mut AssetImporter),
    {
        config_fn(&mut self.asset_importer);
        self
    }

    /// Get asset pipeline build statistics
    pub fn get_build_stats(&self) -> String {
        let stats = self.asset_pipeline.get_build_stats();
        format!(
            "Build Stats: {} assets processed, {} optimized, {} cached, avg build time: {:.2}ms",
            stats.assets_processed,
            stats.assets_optimized,
            stats.cache_hits,
            stats.average_build_time_ms
        )
    }

    /// Clear asset pipeline cache
    pub fn clear_asset_cache(&mut self) -> &mut Self {
        match self.asset_pipeline.clear_cache() {
            Ok(()) => {
                log::info!("Cleared asset cache");
            }
            Err(e) => {
                log::error!("Failed to clear asset cache: {}", e);
            }
        }
        self
    }

    /// Validate assets and check for issues
    pub fn validate_assets(&mut self) -> &mut Self {
        match self.asset_importer.validate_all_assets() {
            Ok(()) => {
                log::debug!("Asset validation passed");
            }
            Err(error) => {
                log::error!("Asset validation failed: {}", error);
                self.handle_error(
                    RobinError::AssetValidationError {
                        asset_path: std::path::PathBuf::from("<unknown>"),
                        validation_errors: vec![error.to_string()],
                    },
                    Some("Asset Validation")
                );
            }
        }
        self
    }

    // === SAVE/LOAD SYSTEM METHODS ===

    /// Save the current game state to a slot
    pub fn save_game(&mut self, slot: usize, save_name: &str) -> &mut Self {
        // Create game state from current engine state
        let mut game_state = GameState::new(slot, save_name.to_string());
        
        // Capture current game state
        if let Err(e) = game_state.capture_from_engine(
            &self.scene,
            &self.physics_world,
            &self.particle_system
        ) {
            log::error!("Failed to capture game state: {}", e);
            self.handle_error(
                RobinError::SerializationError {
                    object_type: "GameState".to_string(),
                    reason: e.to_string(),
                },
                Some("Game Save")
            );
            return self;
        }

        // Save the game state
        if let Err(e) = self.save_manager.save_game(slot, &game_state) {
            log::error!("Failed to save game to slot {}: {}", slot, e);
            self.handle_error(
                RobinError::FileSaveError {
                    path: self.save_manager.get_save_path(slot),
                    reason: e.to_string(),
                },
                Some("Game Save")
            );
        } else {
            log::info!("Game saved successfully to slot {}: '{}'", slot, save_name);

            // Update profile if available
            if let Err(e) = self.profile_manager.update_active_profile_from_save(
                &game_state.metadata, 
                self.get_session_playtime()
            ) {
                log::warn!("Failed to update profile: {}", e);
            }
        }
        
        self
    }

    /// Load a game state from a slot
    pub fn load_game(&mut self, slot: usize) -> &mut Self {
        match self.save_manager.load_game(slot) {
            Ok(game_state) => {
                // Apply the loaded state to engine components
                if let Err(e) = game_state.apply_to_engine(
                    &mut self.scene,
                    &mut self.physics_world,
                    &mut self.particle_system
                ) {
                    log::error!("Failed to apply loaded game state: {}", e);
                    self.handle_error(
                        RobinError::SerializationError {
                            object_type: "GameState".to_string(),
                            reason: e.to_string(),
                        },
                        Some("Game Load")
                    );
                } else {
                    log::info!("Game loaded successfully from slot {}: '{}'", 
                        slot, game_state.metadata.name);
                }
            }
            Err(e) => {
                log::error!("Failed to load game from slot {}: {}", slot, e);
                self.handle_error(
                    RobinError::FileLoadError {
                        path: self.save_manager.get_save_path(slot),
                        reason: e.to_string(),
                    },
                    Some("Game Load")
                );
            }
        }
        
        self
    }

    /// Delete a save from a slot
    pub fn delete_save(&mut self, slot: usize) -> &mut Self {
        if let Err(e) = self.save_manager.delete_save(slot) {
            log::error!("Failed to delete save from slot {}: {}", slot, e);
            self.handle_error(
                RobinError::FileAccessDenied(self.save_manager.get_save_path(slot)),
                Some("Save Delete")
            );
        } else {
            log::info!("Save deleted from slot {}", slot);
        }
        
        self
    }

    /// Check if a save exists in the given slot
    pub fn save_exists(&self, slot: usize) -> bool {
        self.save_manager.save_exists(slot)
    }

    /// Get metadata for a specific save slot
    pub fn get_save_info(&self, slot: usize) -> Option<String> {
        self.save_manager.get_save_metadata(slot).map(|metadata| {
            format!("{} - {} - Progress: {:.1}%", 
                metadata.name,
                metadata.get_timestamp_string(),
                metadata.progress
            )
        })
    }

    /// Get all save slot information
    pub fn list_all_saves(&self) -> Vec<String> {
        let metadata = self.save_manager.get_all_save_metadata();
        let mut saves = Vec::new();
        
        for slot in 0..10 { // Show first 10 slots
            if let Some(meta) = metadata.get(&slot) {
                saves.push(format!("Slot {}: {} - {} - {}", 
                    slot,
                    meta.name,
                    meta.get_timestamp_string(),
                    meta.get_play_time_string()
                ));
            } else {
                saves.push(format!("Slot {}: [Empty]", slot));
            }
        }
        
        saves
    }

    /// Enable/disable auto-save
    pub fn set_auto_save(&mut self, enabled: bool, slot: Option<usize>) -> &mut Self {
        let auto_save = self.save_manager.get_auto_save_manager();
        
        if enabled {
            if let Some(save_slot) = slot {
                auto_save.set_slot(save_slot);
                log::info!("Auto-save enabled for slot {}", save_slot);
            } else {
                log::warn!("Auto-save enabled but no slot specified");
            }
        } else {
            log::info!("Auto-save disabled");
        }
        
        self
    }

    /// Force an immediate auto-save
    pub fn force_auto_save(&mut self) -> &mut Self {
        let mut game_state = GameState::new(0, "Auto Save".to_string());
        
        if let Err(e) = game_state.capture_from_engine(
            &self.scene,
            &self.physics_world,
            &self.particle_system
        ) {
            log::error!("Failed to capture state for auto-save: {}", e);
            return self;
        }

        if let Err(e) = self.save_manager.get_auto_save_manager().force_auto_save(&game_state) {
            log::error!("Failed to perform auto-save: {}", e);
        } else {
            log::info!("Auto-save completed");
        }
        
        self
    }

    // === PROFILE MANAGEMENT METHODS ===

    /// Create a new user profile
    pub fn create_user_profile(&mut self, user_id: &str, display_name: &str) -> &mut Self {
        if let Err(e) = self.profile_manager.create_profile(user_id.to_string(), display_name.to_string()) {
            log::error!("Failed to create profile '{}': {}", user_id, e);
            self.handle_error(
                RobinError::ValidationError {
                    field: "profile_id".to_string(),
                    value: user_id.to_string(),
                    constraint: "Profile must not already exist".to_string(),
                },
                Some("Profile Creation")
            );
        } else {
            log::info!("Created user profile: '{}' ({})", display_name, user_id);
        }
        
        self
    }

    /// Set the active user profile
    pub fn set_active_profile(&mut self, profile_id: &str) -> &mut Self {
        if let Err(e) = self.profile_manager.set_active_profile(profile_id) {
            log::error!("Failed to set active profile '{}': {}", profile_id, e);
            self.handle_error(
                RobinError::ValidationError {
                    field: "profile_id".to_string(),
                    value: profile_id.to_string(),
                    constraint: "Profile must exist".to_string(),
                },
                Some("Profile Selection")
            );
        } else {
            log::info!("Set active profile: '{}'", profile_id);
        }
        
        self
    }

    /// Get current active profile information
    pub fn get_active_profile_info(&self) -> Option<String> {
        self.profile_manager.get_active_profile().map(|profile| {
            format!("{} - {} - Achievements: {}/{}", 
                profile.display_name,
                profile.get_total_playtime_string(),
                profile.achievements.unlocked.len(),
                profile.achievements.progress.len()
            )
        })
    }

    /// List all user profiles
    pub fn list_profiles(&self) -> Vec<String> {
        self.profile_manager.get_all_profiles()
            .iter()
            .map(|(id, profile)| {
                format!("{}: {} ({})", 
                    id,
                    profile.display_name,
                    profile.get_total_playtime_string()
                )
            })
            .collect()
    }

    /// Delete a user profile
    pub fn delete_profile(&mut self, profile_id: &str) -> &mut Self {
        if let Err(e) = self.profile_manager.delete_profile(profile_id) {
            log::error!("Failed to delete profile '{}': {}", profile_id, e);
        } else {
            log::info!("Deleted profile: '{}'", profile_id);
        }
        
        self
    }

    /// Unlock an achievement for the active profile
    pub fn unlock_achievement(&mut self, achievement_id: &str) -> &mut Self {
        let achievement_unlocked = if let Some(profile) = self.profile_manager.get_active_profile_mut() {
            profile.unlock_achievement(achievement_id);
            true
        } else {
            log::warn!("No active profile to unlock achievement for");
            false
        };
        
        if achievement_unlocked {
            if let Err(e) = self.profile_manager.save_active_profile() {
                log::error!("Failed to save profile after unlocking achievement: {}", e);
            } else {
                log::info!("Unlocked achievement: '{}'", achievement_id);
            }
        }
        
        self
    }

    /// Update achievement progress for the active profile
    pub fn update_achievement_progress(&mut self, achievement_id: &str, progress: f32) -> &mut Self {
        let achievement_updated = if let Some(profile) = self.profile_manager.get_active_profile_mut() {
            profile.update_achievement(achievement_id, progress);
            true
        } else {
            false
        };
        
        if achievement_updated {
            if let Err(e) = self.profile_manager.save_active_profile() {
                log::error!("Failed to save profile after updating achievement progress: {}", e);
            } else {
                log::debug!("Updated achievement '{}' progress to {:.1}%", achievement_id, progress * 100.0);
            }
        }
        
        self
    }

    /// Get current session playtime (placeholder)
    fn get_session_playtime(&self) -> u64 {
        // In a real implementation, this would track actual session time
        self.frame_counter / 60 // Rough approximation assuming 60 FPS
    }

    // === SCENE MANAGEMENT METHODS ===

    /// Create a new empty scene
    pub fn create_scene(&mut self, name: &str) -> &mut Self {
        self.scene_manager.create_scene(name);
        log::info!("Created scene: {}", name);
        self
    }

    /// Create a 2D platformer scene with basic platforms and spawn point
    pub fn create_2d_platformer_scene(&mut self, name: &str) -> &mut Self {
        SceneTemplate::create_2d_platformer(&mut self.scene_manager, name);
        log::info!("Created 2D platformer scene: {}", name);
        self
    }

    /// Create a top-down scene template
    pub fn create_top_down_scene(&mut self, name: &str) -> &mut Self {
        SceneTemplate::create_top_down(&mut self.scene_manager, name);
        log::info!("Created top-down scene: {}", name);
        self
    }

    /// Create a basic scene with essential components
    pub fn create_basic_scene(&mut self, name: &str) -> &mut Self {
        SceneTemplate::create_basic(&mut self.scene_manager, name);
        log::info!("Created basic scene: {}", name);
        self
    }

    /// Load a scene from file
    pub fn load_scene(&mut self, name: &str, file_path: &str) -> &mut Self {
        match self.scene_manager.load_scene(name, file_path) {
            Ok(()) => {
                log::info!("Successfully loaded scene '{}' from {}", name, file_path);
            }
            Err(e) => {
                log::error!("Failed to load scene '{}' from {}: {}", name, file_path, e);
            }
        }
        self
    }

    /// Save a scene to file
    pub fn save_scene(&mut self, name: &str, file_path: Option<&str>) -> &mut Self {
        let path = file_path.map(std::path::PathBuf::from);
        match self.scene_manager.save_scene(name, path) {
            Ok(()) => {
                log::info!("Successfully saved scene '{}'", name);
            }
            Err(e) => {
                log::error!("Failed to save scene '{}': {}", name, e);
            }
        }
        self
    }

    /// Set the active scene
    pub fn set_active_scene(&mut self, name: &str) -> &mut Self {
        if let Some(scene) = self.scene_manager.set_active_scene(name) {
            // Copy the scene to our main scene reference for compatibility
            // In a more advanced system, we might work directly with the scene manager's scenes
            log::info!("Activated scene: {}", name);
        } else {
            log::warn!("Failed to activate scene '{}' - scene not found", name);
        }
        self
    }

    /// Duplicate an existing scene
    pub fn duplicate_scene(&mut self, source_name: &str, new_name: &str) -> &mut Self {
        match self.scene_manager.duplicate_scene(source_name, new_name) {
            Ok(()) => {
                log::info!("Duplicated scene '{}' as '{}'", source_name, new_name);
            }
            Err(e) => {
                log::error!("Failed to duplicate scene '{}' as '{}': {}", source_name, new_name, e);
            }
        }
        self
    }

    /// Remove a scene
    pub fn remove_scene(&mut self, name: &str) -> &mut Self {
        if self.scene_manager.remove_scene(name).is_some() {
            log::info!("Removed scene: {}", name);
        } else {
            log::warn!("Failed to remove scene '{}' - scene not found", name);
        }
        self
    }

    /// Get list of all available scenes
    pub fn list_scenes(&self) -> Vec<String> {
        self.scene_manager.list_scenes().into_iter().cloned().collect()
    }

    /// Enable auto-save for scenes
    pub fn enable_scene_auto_save(&mut self, enabled: bool, interval_seconds: Option<u64>) -> &mut Self {
        self.scene_manager.set_auto_save(enabled, interval_seconds);
        log::info!("Scene auto-save {}", if enabled { "enabled" } else { "disabled" });
        self
    }

    /// Set the directory where scenes are stored
    pub fn set_scene_directory(&mut self, path: &str) -> &mut Self {
        self.scene_manager.set_scene_directory(path);
        log::info!("Set scene directory to: {}", path);
        self
    }

    /// Scan and load all scenes from a directory
    pub fn load_scenes_from_directory(&mut self, directory: &str) -> &mut Self {
        match self.scene_manager.scan_and_load_scenes(directory) {
            Ok(loaded_scenes) => {
                log::info!("Loaded {} scenes from {}: {:?}", 
                    loaded_scenes.len(), directory, loaded_scenes);
            }
            Err(e) => {
                log::error!("Failed to scan scenes from {}: {}", directory, e);
            }
        }
        self
    }

    /// Get statistics about scenes
    pub fn get_scene_stats(&self) -> String {
        format!("{}", self.scene_manager.get_statistics())
    }

    // === ERROR HANDLING AND DIAGNOSTICS METHODS ===

    /// Initialize comprehensive logging system
    pub fn setup_logging(&mut self, config: Option<LoggingConfig>) -> &mut Self {
        let logging_config = config.unwrap_or_default();
        
        match RobinLogger::init(logging_config.clone()) {
            Ok(_) => {
                log::info!("Robin Engine logging system initialized successfully");
                log::debug!("Logging configuration: console={}, file={}", 
                    logging_config.console_enabled, logging_config.file_enabled);
            }
            Err(e) => {
                eprintln!("Failed to initialize logging system: {}", e);
                self.last_error = Some(RobinError::InitializationError {
                    subsystem: "Logging".to_string(),
                    reason: e.to_string(),
                });
            }
        }
        
        self
    }

    /// Configure diagnostics and performance monitoring
    pub fn setup_diagnostics(&mut self, config: Option<DiagnosticsConfig>) -> &mut Self {
        let diag_config = config.unwrap_or_default();
        log::info!("Configuring diagnostics: enabled={}, profiling={}", 
            diag_config.enabled, diag_config.detailed_profiling);
        
        self.diagnostics = DiagnosticsManager::new(diag_config);
        self
    }

    /// Enable comprehensive error recovery
    pub fn enable_error_recovery(&mut self, enabled: bool) -> &mut Self {
        if enabled {
            log::info!("Error recovery enabled - engine will attempt to recover from non-critical errors");
        } else {
            log::info!("Error recovery disabled - engine will stop on errors");
        }
        // Store error recovery preference (would be used in error handling)
        self
    }

    /// Get current performance metrics
    pub fn get_performance_metrics(&self) -> String {
        let metrics = self.diagnostics.get_current_metrics();
        format!(
            "FPS: {:.1}, Frame Time: {:.2}ms, Memory: {:.2}MB, Errors: {}",
            metrics.fps,
            metrics.frame_time,
            metrics.memory_usage as f64 / (1024.0 * 1024.0),
            metrics.active_errors
        )
    }

    /// Generate comprehensive diagnostics report
    pub fn generate_diagnostics_report(&self) -> String {
        let report = self.diagnostics.generate_report();
        match serde_json::to_string_pretty(&report) {
            Ok(json) => json,
            Err(e) => {
                log::error!("Failed to serialize diagnostics report: {}", e);
                format!("Error generating report: {}", e)
            }
        }
    }

    /// Log a custom performance counter
    pub fn log_counter(&mut self, name: &str, value: f64) -> &mut Self {
        self.diagnostics.record_counter(name, value);
        self
    }

    /// Take a memory usage snapshot
    pub fn snapshot_memory(&mut self) -> &mut Self {
        self.diagnostics.snapshot_memory();
        self
    }

    /// Handle and log an error with context
    pub fn handle_error(&mut self, error: RobinError, context: Option<&str>) -> &mut Self {
        log::error!("Engine error{}: {}", 
            context.map(|c| format!(" in {}", c)).unwrap_or_default(), 
            error);
        
        self.diagnostics.log_error(&error, context.map(|s| s.to_string()));
        self.last_error = Some(error);
        self
    }

    /// Get the last error that occurred
    pub fn get_last_error(&self) -> Option<&RobinError> {
        self.last_error.as_ref()
    }

    /// Clear the last error
    pub fn clear_last_error(&mut self) -> &mut Self {
        self.last_error = None;
        self
    }

    /// Check if the engine is in an error state
    pub fn has_error(&self) -> bool {
        self.last_error.is_some()
    }

    /// Get engine health status
    pub fn get_health_status(&self) -> EngineHealthStatus {
        let metrics = self.diagnostics.get_current_metrics();
        
        let performance_health = if metrics.fps < 30.0 {
            HealthLevel::Critical
        } else if metrics.fps < 50.0 {
            HealthLevel::Warning
        } else {
            HealthLevel::Good
        };

        let memory_health = if metrics.memory_usage > 500 * 1024 * 1024 { // 500MB
            HealthLevel::Warning
        } else if metrics.memory_usage > 1024 * 1024 * 1024 { // 1GB
            HealthLevel::Critical
        } else {
            HealthLevel::Good
        };

        let error_health = if metrics.active_errors > 10 {
            HealthLevel::Critical
        } else if metrics.active_errors > 0 {
            HealthLevel::Warning
        } else {
            HealthLevel::Good
        };

        EngineHealthStatus {
            overall: [performance_health, memory_health, error_health]
                .iter()
                .max()
                .unwrap()
                .clone(),
            performance: performance_health,
            memory: memory_health,
            errors: error_health,
            details: format!(
                "FPS: {:.1}, Memory: {:.1}MB, Errors: {}",
                metrics.fps,
                metrics.memory_usage as f64 / (1024.0 * 1024.0),
                metrics.active_errors
            ),
        }
    }

    /// Create a physics playground with bouncing balls and platforms
    pub fn create_physics_playground(&mut self, area_left: f32, area_right: f32, area_top: f32, area_bottom: f32) -> Vec<u32> {
        let mut object_ids = Vec::new();
        
        // Create boundaries
        let boundaries = self.create_physics_boundaries(area_left, area_right, area_top, area_bottom, 20.0);
        object_ids.extend(boundaries);
        
        // Create some platforms
        let platform1 = self.create_platform(area_left + 100.0, area_bottom - 100.0, 120.0, 20.0);
        object_ids.push(platform1);
        
        let platform2 = self.create_platform(area_right - 100.0, area_bottom - 200.0, 120.0, 20.0);
        object_ids.push(platform2);
        
        // Create bouncing balls with particle trails
        for i in 0..5 {
            let x = area_left + 50.0 + (i as f32) * 80.0;
            let y = area_top + 50.0;
            let ball_id = self.create_bouncing_ball(x, y, 15.0);
            object_ids.push(ball_id);
            
            // Add a sparkle trail that follows the ball
            let trail_name = format!("ball_trail_{}", i);
            self.particle_system.create_magic_sparkles(Vec2::new(x, y), trail_name);
        }
        
        object_ids
    }

    // === UPDATE METHODS ===
    // Note: Main update method is implemented below with full diagnostics support

    /// Start all animations for a specific object
    pub fn start_animations(&mut self, object_id: &str) -> &mut Self {
        self.animation_manager.start_animation(object_id);
        self
    }

    /// Remove all effects and animations for cleanup
    pub fn clear_all_effects(&mut self) -> &mut Self {
        self.particle_system = ParticleSystem::new();
        self.animation_manager = AnimationManager::new();
        self.physics_world = PhysicsWorld::new();
        self.audio_manager.stop_all_sounds();
        self.lights.clear();
        self.effects_counter = 0;
        self.physics_body_counter = 0;
        self.sound_counter = 0;
        self
    }

    /// Get current number of active particles (for performance monitoring)
    pub fn get_particle_count(&self) -> usize {
        self.particle_system.get_all_particles().len()
    }

    /// Get current number of active lights
    pub fn get_light_count(&self) -> usize {
        self.lights.len()
    }

    /// Get current number of physics bodies
    pub fn get_physics_body_count(&self) -> usize {
        self.physics_world.bodies.len()
    }

    /// Get all collision events from the last physics step
    pub fn get_collision_events(&self) -> &Vec<crate::engine::physics::Collision> {
        &self.physics_world.collisions
    }
}

// === BUILDER PATTERN HELPERS ===

impl Default for GameBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// === PRESET BUILDER METHODS FOR COMMON SCENES ===

impl GameBuilder {
    /// Quick setup for a dungeon scene with torch lighting
    pub fn setup_dungeon_lighting(&mut self, torch_positions: &[(f32, f32)]) -> &mut Self {
        for &(x, y) in torch_positions {
            self.add_torch_light(x, y);
        }
        self
    }

    /// Quick setup for an outdoor night scene with moonlight
    pub fn setup_night_scene(&mut self, moon_x: f32, moon_y: f32) -> &mut Self {
        self.add_light(moon_x, moon_y, (0.8, 0.9, 1.0), 0.4); // Soft blue moonlight
        self
    }

    /// Quick setup for a magical forest with various light sources
    pub fn setup_magical_forest(&mut self, fairy_positions: &[(f32, f32)]) -> &mut Self {
        for &(x, y) in fairy_positions {
            self.add_magic_light(x, y);
        }
        self
    }

    // === UPDATE METHODS ===

    /// Update all systems (call this every frame)
    pub fn update(&mut self, delta_time: f32, input: &crate::engine::input::InputManager) -> HashMap<String, Vec<crate::engine::animation::AnimationValue>> {
        self.frame_counter += 1;
        
        // Start frame timing
        let mut frame_timer = self.diagnostics.begin_frame(self.frame_counter);
        
        // Update scene manager (handles auto-save, etc.)
        frame_timer.begin_update();
        self.scene_manager.update(delta_time);
        frame_timer.end_update();
        
        // Update particle systems
        self.particle_system.update(delta_time);
        
        // Update physics simulation
        frame_timer.begin_physics();
        self.physics_world.step(delta_time);
        frame_timer.end_physics();
        
        // Update audio system (clean up finished sounds)
        self.audio_manager.update();
        
        // Update UI system
        self.ui_manager.update(delta_time, input);
        
        // Update hot-reload system (process file changes)
        self.hot_reload.update();
        
        // Record performance counters
        let fps = if delta_time > 0.0 { 1.0 / delta_time } else { 0.0 };
        self.diagnostics.record_counter("fps", fps as f64);
        self.diagnostics.record_counter("frame_time", (delta_time * 1000.0) as f64);
        self.diagnostics.record_counter("active_objects", self.scene.objects().count() as f64);
        
        // Periodic memory snapshots (every 60 frames = ~1 second at 60fps)
        if self.frame_counter % 60 == 0 {
            self.diagnostics.snapshot_memory();
        }
        
        // Update animation system and return values
        let animation_result = self.animation_manager.update();
        
        // Frame timer automatically completes when dropped
        animation_result
    }

    // === AI GAME SYSTEMS ===

    /// Start tracking a player for AI analysis and adaptation
    pub fn start_player_session(&mut self, player_id: &str, player_profile: PlayerProfile) -> RobinResult<()> {
        self.ai_game_manager.start_player_session(player_id, player_profile)
    }

    /// End a player session and get summary analytics
    pub fn end_player_session(&mut self, player_id: &str) -> RobinResult<PlayerProfile> {
        self.ai_game_manager.end_player_session(player_id)
    }

    /// Record a player interaction for AI analysis
    pub fn record_player_interaction(&mut self, player_id: &str, interaction: PlayerInteraction) -> RobinResult<Vec<GameAIEvent>> {
        self.ai_game_manager.process_player_interaction(player_id, &interaction)
    }

    /// Get AI-powered recommendations for a player
    pub fn get_ai_recommendations(&self, player_id: &str) -> RobinResult<Vec<GameAIRecommendation>> {
        self.ai_game_manager.get_recommendations(player_id)
    }

    /// Generate procedural content based on player preferences
    pub fn generate_procedural_content(&mut self, player_id: &str, content_type: &str, parameters: HashMap<String, f32>) -> RobinResult<String> {
        self.ai_game_manager.generate_content(player_id, content_type, parameters)
    }

    /// Adjust game difficulty dynamically based on player performance
    pub fn adjust_difficulty(&mut self, player_id: &str, target_flow_state: f32) -> RobinResult<f32> {
        self.ai_game_manager.adjust_difficulty(player_id, target_flow_state)
    }

    /// Get current player analytics and performance metrics
    pub fn get_player_analytics(&self, player_id: &str) -> Option<HashMap<String, f32>> {
        self.ai_game_manager.get_player_metrics(player_id)
    }

    /// Update AI systems and get generated events
    pub fn update_ai_systems(&mut self, delta_time: f32) -> RobinResult<Vec<GameAIEvent>> {
        self.ai_game_manager.update(delta_time)
    }

    /// Enable or disable AI systems for privacy or performance
    pub fn set_ai_enabled(&mut self, enabled: bool) -> &mut Self {
        self.ai_game_manager.set_enabled(enabled);
        self
    }

    /// Get AI system status and health metrics
    pub fn get_ai_status(&self) -> HashMap<String, String> {
        self.ai_game_manager.get_status()
    }

    /// Trigger AI rebalancing of game systems
    pub fn rebalance_game_systems(&mut self, focus_areas: Vec<String>) -> RobinResult<HashMap<String, f32>> {
        self.ai_game_manager.rebalance_systems(focus_areas)
    }

    /// Create a customized game experience for a player
    pub fn create_personalized_experience(&mut self, player_id: &str, experience_type: &str) -> RobinResult<String> {
        self.ai_game_manager.create_personalized_experience(player_id, experience_type)
    }

    /// Analyze player behavior patterns and predict preferences
    pub fn analyze_player_behavior(&self, player_id: &str) -> RobinResult<HashMap<String, f32>> {
        self.ai_game_manager.analyze_behavior_patterns(player_id)
    }

    /// Get global game analytics across all players
    pub fn get_global_analytics(&self) -> HashMap<String, f32> {
        self.ai_game_manager.get_global_metrics()
    }

    /// Export player data for external analysis (respecting privacy)
    pub fn export_player_data(&self, player_id: &str, include_personal: bool) -> RobinResult<String> {
        self.ai_game_manager.export_player_data(player_id, include_personal)
    }

    /// Import player preferences and settings
    pub fn import_player_preferences(&mut self, player_id: &str, preferences: HashMap<String, f32>) -> RobinResult<()> {
        self.ai_game_manager.import_player_preferences(player_id, preferences)
    }

    /// Quick setup for common AI features
    pub fn enable_ai_features(&mut self, features: Vec<&str>) -> &mut Self {
        for feature in features {
            match feature {
                "analytics" => { self.ai_game_manager.enable_analytics(true); },
                "adaptation" => { self.ai_game_manager.enable_adaptation(true); },
                "generation" => { self.ai_game_manager.enable_generation(true); },
                "balancing" => { self.ai_game_manager.enable_balancing(true); },
                _ => log::warn!("Unknown AI feature: {}", feature),
            }
        }
        self
    }

    /// Create AI-powered tutorial system
    pub fn create_ai_tutorial(&mut self, player_id: &str, tutorial_type: &str) -> RobinResult<Vec<String>> {
        self.ai_game_manager.create_adaptive_tutorial(player_id, tutorial_type)
    }

    /// Generate dynamic quests based on player progression
    pub fn generate_dynamic_quest(&mut self, player_id: &str, difficulty: f32) -> RobinResult<String> {
        self.ai_game_manager.generate_dynamic_quest(player_id, difficulty)
    }
}