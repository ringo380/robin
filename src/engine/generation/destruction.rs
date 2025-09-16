/*!
 * Robin Engine Destructible Environment System
 * 
 * A comprehensive system for creating dynamic, destructible environments with realistic
 * physics-based destruction, structural integrity, and visual effects.
 */

use crate::engine::{
    graphics::{Texture, Color, ParticleSystem},
    math::{Vec2, Vec3, Transform},
    error::{RobinError, RobinResult},
    audio::AudioSystem,
};
use cgmath::InnerSpace;

/// Integer-based coordinate for voxel positions (hashable)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VoxelPos {
    pub x: i32,
    pub y: i32, 
    pub z: i32,
}

impl From<Vec3> for VoxelPos {
    fn from(v: Vec3) -> Self {
        Self {
            x: v.x.round() as i32,
            y: v.y.round() as i32,
            z: v.z.round() as i32,
        }
    }
}

impl From<VoxelPos> for Vec3 {
    fn from(vp: VoxelPos) -> Self {
        Vec3::new(vp.x as f32, vp.y as f32, vp.z as f32)
    }
}
use super::{VoxelSystem, VoxelType, VoxelWorld, PixelScatterSystem};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::Instant;

/// Main destructible environment system
#[derive(Debug)]
pub struct DestructionSystem {
    config: DestructionConfig,
    /// Active destruction events being processed
    active_destructions: HashMap<String, DestructionEvent>,
    /// Structural integrity analyzer
    integrity_analyzer: StructuralIntegritySystem,
    /// Physics simulation for falling debris
    debris_physics: DebrisPhysics,
    /// Particle effects for destruction
    particle_system: DestructionParticles,
    /// Sound effects
    audio_system: Option<AudioSystem>,
    /// Performance metrics
    stats: DestructionStats,
}

impl DestructionSystem {
    pub fn new(config: DestructionConfig) -> Self {
        Self {
            integrity_analyzer: StructuralIntegritySystem::new(config.integrity_config.clone()),
            debris_physics: DebrisPhysics::new(config.physics_config.clone()),
            particle_system: DestructionParticles::new(config.particle_config.clone()),
            audio_system: None,
            stats: DestructionStats::default(),
            active_destructions: HashMap::new(),
            config,
        }
    }

    /// Generate a destructible environment
    pub fn generate_destructible_environment(&mut self, params: DestructionParams) -> RobinResult<DestructibleEnvironment> {
        let start_time = Instant::now();

        // Create base structure using voxel system
        let mut structure = self.create_base_structure(&params)?;
        
        // Analyze structural integrity
        self.analyze_structural_integrity(&mut structure)?;
        
        // Add destruction triggers
        self.add_destruction_triggers(&mut structure, &params)?;
        
        // Configure environmental effects
        self.configure_environmental_effects(&mut structure, &params)?;

        let generation_time = start_time.elapsed().as_secs_f32();
        self.stats.total_generation_time += generation_time;
        self.stats.environments_generated += 1;

        Ok(structure)
    }

    /// Trigger destruction at a specific location
    pub fn trigger_destruction(&mut self, world_id: &str, position: Vec3, destruction_type: DestructionType, force: f32) -> RobinResult<String> {
        let event_id = format!("dest_{}_{}", world_id, self.stats.destruction_events);
        
        let event = DestructionEvent {
            id: event_id.clone(),
            world_id: world_id.to_string(),
            position,
            destruction_type,
            force,
            radius: self.calculate_destruction_radius(force, &destruction_type),
            start_time: Instant::now(),
            affected_voxels: HashSet::new(),
            phase: DestructionPhase::Initial,
            particles: Vec::new(),
            debris: Vec::new(),
        };

        self.active_destructions.insert(event_id.clone(), event);
        self.stats.destruction_events += 1;

        Ok(event_id)
    }

    /// Update the destruction system (call each frame)
    pub fn update(&mut self, delta_time: f32) {
        self.update_active_destructions(delta_time);
        self.debris_physics.update(delta_time);
        self.particle_system.update(delta_time);
        self.update_environmental_effects(delta_time);
    }

    /// Process destruction event for a voxel world
    pub fn process_destruction(&mut self, voxel_world: &mut VoxelWorld, event_id: &str) -> RobinResult<DestructionResult> {
        // Clone the event to avoid borrowing conflicts
        let event_opt = self.active_destructions.get(event_id).cloned();
        let mut event = event_opt.ok_or_else(|| RobinError::DestructionError(format!("Event not found: {}", event_id)))?;

        let result = match event.phase {
            DestructionPhase::Initial => self.process_initial_destruction(voxel_world, &mut event),
            DestructionPhase::Propagating => self.process_propagation(voxel_world, &mut event),
            DestructionPhase::Collapsing => self.process_collapse(voxel_world, &mut event),
            DestructionPhase::Settling => self.process_settling(voxel_world, &mut event),
            DestructionPhase::Complete => Ok(DestructionResult::Complete),
        };

        // Update the event back in the HashMap
        if let Some(stored_event) = self.active_destructions.get_mut(event_id) {
            *stored_event = event;
        }

        result
    }

    /// Check if a voxel can be destroyed
    pub fn can_destroy_voxel(&self, voxel_type: &VoxelType, destruction_type: &DestructionType, force: f32) -> bool {
        let resistance = match voxel_type {
            VoxelType::Air => 0.0,
            VoxelType::Solid => 40.0,
            VoxelType::Liquid => 1.0,
            VoxelType::Gas => 0.1,
            VoxelType::Light => 0.0,
            VoxelType::Stone => 80.0,
            VoxelType::Wood => 30.0,
            VoxelType::Metal => 120.0,
            VoxelType::Glass => 5.0,
            VoxelType::Concrete => 100.0,
            VoxelType::Brick => 60.0,
            VoxelType::Custom(_) => 50.0, // TODO: Add destruction_resistance field to custom voxel properties
        };

        let destruction_power = match destruction_type {
            DestructionType::Explosion => force * 1.5,
            DestructionType::Impact => force * 1.0,
            DestructionType::Erosion => force * 0.3,
            DestructionType::Cutting => force * 2.0,
            DestructionType::Melting => force * 0.8,
            DestructionType::Freezing => force * 0.6,
        };

        destruction_power > resistance
    }

    /// Get destruction statistics
    pub fn get_stats(&self) -> &DestructionStats {
        &self.stats
    }

    /// Clear completed destruction events
    pub fn cleanup_completed_events(&mut self) {
        self.active_destructions.retain(|_, event| {
            event.phase != DestructionPhase::Complete
        });
    }

    fn create_base_structure(&self, params: &DestructionParams) -> RobinResult<DestructibleEnvironment> {
        let mut structure = DestructibleEnvironment {
            id: params.environment_id.clone(),
            dimensions: params.dimensions,
            voxel_data: HashMap::new(),
            integrity_map: HashMap::new(),
            destruction_triggers: Vec::new(),
            environmental_effects: Vec::new(),
            structural_supports: HashMap::new(),
        };

        // Generate base architecture
        match params.structure_type {
            StructureType::Building => self.generate_building_structure(&mut structure, params)?,
            StructureType::Bridge => self.generate_bridge_structure(&mut structure, params)?,
            StructureType::Tower => self.generate_tower_structure(&mut structure, params)?,
            StructureType::Wall => self.generate_wall_structure(&mut structure, params)?,
            StructureType::Terrain => self.generate_terrain_structure(&mut structure, params)?,
            StructureType::Custom => self.generate_custom_structure(&mut structure, params)?,
        }

        Ok(structure)
    }

    fn generate_building_structure(&self, structure: &mut DestructibleEnvironment, params: &DestructionParams) -> RobinResult<()> {
        let (width, height, depth) = (params.dimensions.x as usize, params.dimensions.y as usize, params.dimensions.z as usize);

        // Foundation
        for x in 0..width {
            for z in 0..depth {
                let pos = Vec3::new(x as f32, 0.0, z as f32);
                let voxel_pos = VoxelPos::from(pos);
                structure.voxel_data.insert(voxel_pos, VoxelType::Concrete);
                structure.integrity_map.insert(voxel_pos, 100.0);
            }
        }

        // Walls with structural supports
        for y in 1..height {
            // Exterior walls
            for x in 0..width {
                for z in [0, depth - 1] {
                    let pos = Vec3::new(x as f32, y as f32, z as f32);
                    let voxel_pos = VoxelPos::from(pos);
                    if x % 4 == 0 && y % 3 == 1 { // Structural columns
                        structure.voxel_data.insert(voxel_pos, VoxelType::Metal);
                        structure.structural_supports.insert(voxel_pos, SupportType::Column);
                        structure.integrity_map.insert(voxel_pos, 150.0);
                    } else {
                        structure.voxel_data.insert(voxel_pos, VoxelType::Brick);
                        structure.integrity_map.insert(voxel_pos, 60.0);
                    }
                }
            }
            
            for z in 0..depth {
                for x in [0, width - 1] {
                    let pos = Vec3::new(x as f32, y as f32, z as f32);
                    let voxel_pos = VoxelPos::from(pos);
                    if z % 4 == 0 && y % 3 == 1 {
                        structure.voxel_data.insert(voxel_pos, VoxelType::Metal);
                        structure.structural_supports.insert(voxel_pos, SupportType::Column);
                        structure.integrity_map.insert(voxel_pos, 150.0);
                    } else {
                        structure.voxel_data.insert(voxel_pos, VoxelType::Brick);
                        structure.integrity_map.insert(voxel_pos, 60.0);
                    }
                }
            }
        }

        // Floors/ceilings
        for floor in (3..height).step_by(4) {
            for x in 1..(width - 1) {
                for z in 1..(depth - 1) {
                    let pos = Vec3::new(x as f32, floor as f32, z as f32);
                    let voxel_pos = VoxelPos::from(pos);
                    structure.voxel_data.insert(voxel_pos, VoxelType::Wood);
                    structure.integrity_map.insert(voxel_pos, 30.0);
                }
            }
        }

        // Add windows (weak points)
        for y in (2..height).step_by(4) {
            for x in (2..width - 2).step_by(3) {
                let pos = Vec3::new(x as f32, y as f32, 0.0);
                let voxel_pos = VoxelPos::from(pos);
                structure.voxel_data.insert(voxel_pos, VoxelType::Glass);
                structure.integrity_map.insert(voxel_pos, 5.0);
            }
        }

        Ok(())
    }

    fn generate_bridge_structure(&self, structure: &mut DestructibleEnvironment, params: &DestructionParams) -> RobinResult<()> {
        let (width, height, depth) = (params.dimensions.x as usize, params.dimensions.y as usize, params.dimensions.z as usize);

        // Support pillars
        for pillar_x in (0..width).step_by(width / 4) {
            for y in 0..height {
                let pos = Vec3::new(pillar_x as f32, y as f32, depth as f32 / 2.0);
                let voxel_pos = VoxelPos::from(pos);
                structure.voxel_data.insert(voxel_pos, VoxelType::Concrete);
                structure.structural_supports.insert(voxel_pos, SupportType::Pillar);
                structure.integrity_map.insert(voxel_pos, 120.0);
            }
        }

        // Bridge deck
        let deck_y = height as f32 * 0.8;
        for x in 0..width {
            for z in (depth / 3)..(2 * depth / 3) {
                let pos = Vec3::new(x as f32, deck_y, z as f32);
                let voxel_pos = VoxelPos::from(pos);
                structure.voxel_data.insert(voxel_pos, VoxelType::Wood);
                structure.integrity_map.insert(voxel_pos, 40.0);
            }
        }

        // Cables/supports
        for x in (0..width).step_by(2) {
            for support_height in ((deck_y as usize)..height).step_by(2) {
                let pos = Vec3::new(x as f32, support_height as f32, depth as f32 / 2.0);
                let voxel_pos = VoxelPos::from(pos);
                structure.voxel_data.insert(voxel_pos, VoxelType::Metal);
                structure.structural_supports.insert(voxel_pos, SupportType::Cable);
                structure.integrity_map.insert(voxel_pos, 80.0);
            }
        }

        Ok(())
    }

    fn generate_tower_structure(&self, structure: &mut DestructibleEnvironment, params: &DestructionParams) -> RobinResult<()> {
        let (width, height, depth) = (params.dimensions.x as usize, params.dimensions.y as usize, params.dimensions.z as usize);
        let center_x = width as f32 / 2.0;
        let center_z = depth as f32 / 2.0;

        for y in 0..height {
            let radius = (width as f32 / 2.0) * (1.0 - y as f32 / height as f32 * 0.3);
            
            for x in 0..width {
                for z in 0..depth {
                    let distance = ((x as f32 - center_x).powi(2) + (z as f32 - center_z).powi(2)).sqrt();
                    
                    if distance <= radius && distance >= radius - 2.0 {
                        let pos = Vec3::new(x as f32, y as f32, z as f32);
                        let voxel_pos = VoxelPos::from(pos);
                        
                        if y % 10 == 0 { // Reinforcement levels
                            structure.voxel_data.insert(voxel_pos, VoxelType::Metal);
                            structure.structural_supports.insert(voxel_pos, SupportType::Ring);
                            structure.integrity_map.insert(voxel_pos, 100.0);
                        } else {
                            structure.voxel_data.insert(voxel_pos, VoxelType::Stone);
                            structure.integrity_map.insert(voxel_pos, 70.0);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn generate_wall_structure(&self, structure: &mut DestructibleEnvironment, params: &DestructionParams) -> RobinResult<()> {
        let (width, height, depth) = (params.dimensions.x as usize, params.dimensions.y as usize, params.dimensions.z as usize);

        for y in 0..height {
            for x in 0..width {
                for z in 0..2.min(depth) {
                    let pos = Vec3::new(x as f32, y as f32, z as f32);
                    let voxel_pos = VoxelPos::from(pos);
                    
                    if y == 0 { // Foundation
                        structure.voxel_data.insert(voxel_pos, VoxelType::Concrete);
                        structure.integrity_map.insert(voxel_pos, 120.0);
                    } else if x % 8 == 0 { // Support columns
                        structure.voxel_data.insert(voxel_pos, VoxelType::Metal);
                        structure.structural_supports.insert(voxel_pos, SupportType::Column);
                        structure.integrity_map.insert(voxel_pos, 100.0);
                    } else {
                        structure.voxel_data.insert(voxel_pos, VoxelType::Brick);
                        structure.integrity_map.insert(voxel_pos, 60.0);
                    }
                }
            }
        }

        Ok(())
    }

    fn generate_terrain_structure(&self, structure: &mut DestructibleEnvironment, params: &DestructionParams) -> RobinResult<()> {
        // Generate terrain with different material layers
        let (width, height, depth) = (params.dimensions.x as usize, params.dimensions.y as usize, params.dimensions.z as usize);

        for x in 0..width {
            for z in 0..depth {
                // Terrain height variation
                let height_variation = ((x as f32 * 0.1).sin() + (z as f32 * 0.1).cos()) * 5.0;
                let terrain_height = (height as f32 * 0.6 + height_variation) as usize;

                for y in 0..terrain_height.min(height) {
                    let pos = Vec3::new(x as f32, y as f32, z as f32);
                    let voxel_pos = VoxelPos::from(pos);
                    
                    let material = match y {
                        y if y < terrain_height / 4 => VoxelType::Stone,
                        y if y < terrain_height * 2 / 3 => VoxelType::Custom(1), // Dirt material ID
                        _ => VoxelType::Custom(2), // Grass material ID
                    };

                    structure.voxel_data.insert(voxel_pos, material);
                    structure.integrity_map.insert(voxel_pos, 50.0);
                }
            }
        }

        Ok(())
    }

    fn generate_custom_structure(&self, structure: &mut DestructibleEnvironment, params: &DestructionParams) -> RobinResult<()> {
        // Implement custom structure generation based on params
        // This would be extended based on specific custom structure requirements
        Ok(())
    }

    fn analyze_structural_integrity(&mut self, structure: &mut DestructibleEnvironment) -> RobinResult<()> {
        self.integrity_analyzer.analyze_structure(structure)?;
        Ok(())
    }

    fn add_destruction_triggers(&self, structure: &mut DestructibleEnvironment, params: &DestructionParams) -> RobinResult<()> {
        // Add various trigger points for destruction
        for trigger in &params.triggers {
            structure.destruction_triggers.push(trigger.clone());
        }
        Ok(())
    }

    fn configure_environmental_effects(&self, structure: &mut DestructibleEnvironment, params: &DestructionParams) -> RobinResult<()> {
        // Configure environmental effects like wind, erosion, seismic activity
        for effect in &params.environmental_effects {
            structure.environmental_effects.push(effect.clone());
        }
        Ok(())
    }

    fn calculate_destruction_radius(&self, force: f32, destruction_type: &DestructionType) -> f32 {
        let base_radius = force.sqrt() * 2.0;
        match destruction_type {
            DestructionType::Explosion => base_radius * 1.5,
            DestructionType::Impact => base_radius * 0.8,
            DestructionType::Erosion => base_radius * 2.0,
            DestructionType::Cutting => base_radius * 0.3,
            DestructionType::Melting => base_radius * 1.2,
            DestructionType::Freezing => base_radius * 1.0,
        }
    }

    fn update_active_destructions(&mut self, delta_time: f32) {
        let mut completed_events = Vec::new();
        
        for (event_id, event) in &mut self.active_destructions {
            match event.phase {
                DestructionPhase::Complete => {
                    completed_events.push(event_id.clone());
                },
                _ => {
                    // Update destruction event phase based on elapsed time
                    let elapsed = event.start_time.elapsed().as_secs_f32();
                    
                    event.phase = match elapsed {
                        t if t < 0.1 => DestructionPhase::Initial,
                        t if t < 0.5 => DestructionPhase::Propagating,
                        t if t < 2.0 => DestructionPhase::Collapsing,
                        t if t < 5.0 => DestructionPhase::Settling,
                        _ => DestructionPhase::Complete,
                    };
                }
            }
        }

        for event_id in completed_events {
            self.active_destructions.remove(&event_id);
        }
    }

    fn update_environmental_effects(&mut self, delta_time: f32) {
        // Update ongoing environmental effects
        self.stats.update_time += delta_time;
    }

    fn process_initial_destruction(&mut self, voxel_world: &mut VoxelWorld, event: &mut DestructionEvent) -> RobinResult<DestructionResult> {
        // Remove voxels in immediate impact area
        let mut destroyed_voxels = Vec::new();
        
        for x in -2i32..=2 {
            for y in -2i32..=2 {
                for z in -2i32..=2 {
                    let pos = Vec3::new(
                        event.position.x + x as f32,
                        event.position.y + y as f32,
                        event.position.z + z as f32,
                    );
                    
                    let distance = ((x * x + y * y + z * z) as f32).sqrt();
                    if distance <= event.radius {
                        if let Some(voxel_type) = voxel_world.get_voxel(pos) {
                            if self.can_destroy_voxel(&voxel_type, &event.destruction_type, event.force) {
                                destroyed_voxels.push(pos);
                                event.affected_voxels.insert((pos.x as i32, pos.y as i32, pos.z as i32));
                            }
                        }
                    }
                }
            }
        }

        for pos in destroyed_voxels {
            voxel_world.set_voxel(pos, VoxelType::Air);
            
            // Create debris particle
            event.debris.push(DebrisParticle {
                position: pos,
                velocity: self.calculate_debris_velocity(pos, event.position, event.force),
                voxel_type: voxel_world.get_voxel(pos).unwrap_or(VoxelType::Stone),
                life_time: 5.0,
                angular_velocity: Vec3::new(
                    (rand::random::<f32>() - 0.5) * 10.0,
                    (rand::random::<f32>() - 0.5) * 10.0,
                    (rand::random::<f32>() - 0.5) * 10.0,
                ),
            });
        }

        self.stats.voxels_destroyed += event.affected_voxels.len();
        Ok(DestructionResult::Continuing)
    }

    fn process_propagation(&mut self, voxel_world: &mut VoxelWorld, event: &mut DestructionEvent) -> RobinResult<DestructionResult> {
        // Propagate destruction based on structural integrity
        let mut new_destructions = Vec::new();
        
        for affected_pos in &event.affected_voxels.clone() {
            // Check surrounding voxels for structural failure
            for x in -1i32..=1 {
                for y in -1i32..=1 {
                    for z in -1i32..=1 {
                        if x == 0 && y == 0 && z == 0 { continue; }
                        
                        let pos = Vec3::new(
                            affected_pos.0 as f32 + x as f32,
                            affected_pos.1 as f32 + y as f32,
                            affected_pos.2 as f32 + z as f32,
                        );
                        
                        if !event.affected_voxels.contains(&(pos.x as i32, pos.y as i32, pos.z as i32)) {
                            if let Some(voxel_type) = voxel_world.get_voxel(pos) {
                                // Check if this voxel should collapse due to lost support
                                if self.should_collapse_from_lost_support(voxel_world, pos, &event.affected_voxels) {
                                    new_destructions.push(pos);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Get length before consuming the vector
        let destroyed_count = new_destructions.len();

        for pos in new_destructions {
            if let Some(voxel_type) = voxel_world.get_voxel(pos) {
                voxel_world.set_voxel(pos, VoxelType::Air);
                event.affected_voxels.insert((pos.x as i32, pos.y as i32, pos.z as i32));
                
                // Create debris
                event.debris.push(DebrisParticle {
                    position: pos,
                    velocity: Vec3::new(0.0, -2.0, 0.0), // Falling debris
                    voxel_type,
                    life_time: 8.0,
                    angular_velocity: Vec3::new(
                        (rand::random::<f32>() - 0.5) * 5.0,
                        (rand::random::<f32>() - 0.5) * 5.0,
                        (rand::random::<f32>() - 0.5) * 5.0,
                    ),
                });
            }
        }

        self.stats.voxels_destroyed += destroyed_count;
        Ok(DestructionResult::Continuing)
    }

    fn process_collapse(&mut self, voxel_world: &mut VoxelWorld, event: &mut DestructionEvent) -> RobinResult<DestructionResult> {
        // Handle major structural collapses
        let mut collapse_areas = self.identify_collapse_areas(voxel_world, &event.affected_voxels)?;
        
        for collapse_area in collapse_areas {
            for pos in collapse_area {
                if let Some(voxel_type) = voxel_world.get_voxel(pos) {
                    voxel_world.set_voxel(pos, VoxelType::Air);
                    event.affected_voxels.insert((pos.x as i32, pos.y as i32, pos.z as i32));
                    
                    // Create larger debris chunks for collapsed sections
                    event.debris.push(DebrisParticle {
                        position: pos,
                        velocity: Vec3::new(
                            (rand::random::<f32>() - 0.5) * 3.0,
                            -5.0 - rand::random::<f32>() * 3.0,
                            (rand::random::<f32>() - 0.5) * 3.0,
                        ),
                        voxel_type,
                        life_time: 10.0,
                        angular_velocity: Vec3::new(
                            (rand::random::<f32>() - 0.5) * 8.0,
                            (rand::random::<f32>() - 0.5) * 8.0,
                            (rand::random::<f32>() - 0.5) * 8.0,
                        ),
                    });
                }
            }
        }

        Ok(DestructionResult::Continuing)
    }

    fn process_settling(&mut self, _voxel_world: &mut VoxelWorld, event: &mut DestructionEvent) -> RobinResult<DestructionResult> {
        // Handle settling physics and final debris movement
        
        // Update debris physics
        for debris in &mut event.debris {
            debris.life_time -= 0.016; // Assume 60fps
            
            if debris.life_time <= 0.0 {
                continue;
            }
            
            // Apply gravity
            debris.velocity.y -= 9.81 * 0.016;
            
            // Update position
            debris.position.x += debris.velocity.x * 0.016;
            debris.position.y += debris.velocity.y * 0.016;
            debris.position.z += debris.velocity.z * 0.016;
            
            // Apply air resistance
            debris.velocity = debris.velocity * 0.98;
            debris.angular_velocity = debris.angular_velocity * 0.95;
        }

        // Remove expired debris
        event.debris.retain(|debris| debris.life_time > 0.0);

        Ok(DestructionResult::Continuing)
    }

    fn should_collapse_from_lost_support(&self, voxel_world: &VoxelWorld, pos: Vec3, destroyed_voxels: &HashSet<(i32, i32, i32)>) -> bool {
        // Check if this voxel has lost critical support
        let mut support_count = 0;
        let mut total_neighbors = 0;

        for x in -1i32..=1 {
            for y in -1i32..=1 {
                for z in -1i32..=1 {
                    if x == 0 && y == 0 && z == 0 { continue; }
                    
                    let neighbor_pos = (
                        pos.x as i32 + x,
                        pos.y as i32 + y,
                        pos.z as i32 + z,
                    );
                    let neighbor_vec3 = Vec3::new(neighbor_pos.0 as f32, neighbor_pos.1 as f32, neighbor_pos.2 as f32);
                    
                    total_neighbors += 1;
                    
                    if !destroyed_voxels.contains(&neighbor_pos) {
                        if let Some(voxel_type) = voxel_world.get_voxel(neighbor_vec3) {
                            if voxel_type != VoxelType::Air {
                                support_count += 1;
                            }
                        }
                    }
                }
            }
        }

        // If more than 60% of neighbors are gone, this voxel should collapse
        (support_count as f32 / total_neighbors as f32) < 0.4
    }

    fn identify_collapse_areas(&self, voxel_world: &VoxelWorld, affected_voxels: &HashSet<(i32, i32, i32)>) -> RobinResult<Vec<Vec<Vec3>>> {
        let mut collapse_areas = Vec::new();
        let mut visited: HashSet<(i32, i32, i32)> = HashSet::new();

        for pos in affected_voxels {
            if visited.contains(pos) { continue; }

            let collapse_area = self.flood_fill_collapse_area(voxel_world, *pos, &mut visited, affected_voxels)?;
            if collapse_area.len() > 5 { // Only consider significant collapses
                collapse_areas.push(collapse_area);
            }
        }

        Ok(collapse_areas)
    }

    fn flood_fill_collapse_area(&self, voxel_world: &VoxelWorld, start_pos: (i32, i32, i32), visited: &mut HashSet<(i32, i32, i32)>, affected_voxels: &HashSet<(i32, i32, i32)>) -> RobinResult<Vec<Vec3>> {
        let mut area = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(start_pos);

        while let Some(pos) = queue.pop_front() {
            if visited.contains(&pos) { continue; }
            visited.insert(pos);

            let pos_vec3 = Vec3::new(pos.0 as f32, pos.1 as f32, pos.2 as f32);
            if self.should_collapse_from_lost_support(voxel_world, pos_vec3, affected_voxels) {
                area.push(pos_vec3);

                // Check neighbors
                for x in -1i32..=1 {
                    for y in -1i32..=1 {
                        for z in -1i32..=1 {
                            if x == 0 && y == 0 && z == 0 { continue; }
                            
                            let neighbor_pos = (
                                pos.0 + x,
                                pos.1 + y,
                                pos.2 + z,
                            );
                            
                            if !visited.contains(&neighbor_pos) {
                                queue.push_back(neighbor_pos);
                            }
                        }
                    }
                }
            }
        }

        Ok(area)
    }

    fn calculate_debris_velocity(&self, debris_pos: Vec3, explosion_center: Vec3, force: f32) -> Vec3 {
        let direction = debris_pos - explosion_center;
        let distance = direction.magnitude().max(1.0);
        let normalized_direction = direction / distance;
        
        let velocity_magnitude = force / distance.sqrt();
        normalized_direction * velocity_magnitude
    }
}

/// Configuration for the destruction system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DestructionConfig {
    pub integrity_config: IntegrityConfig,
    pub physics_config: PhysicsConfig,
    pub particle_config: ParticleConfig,
    pub max_concurrent_events: usize,
    pub debris_lifetime: f32,
    pub enable_sound_effects: bool,
    pub performance_mode: PerformanceMode,
}

impl Default for DestructionConfig {
    fn default() -> Self {
        Self {
            integrity_config: IntegrityConfig::default(),
            physics_config: PhysicsConfig::default(),
            particle_config: ParticleConfig::default(),
            max_concurrent_events: 10,
            debris_lifetime: 30.0,
            enable_sound_effects: true,
            performance_mode: PerformanceMode::Balanced,
        }
    }
}

/// Parameters for generating destructible environments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DestructionParams {
    pub environment_id: String,
    pub dimensions: Vec3,
    pub structure_type: StructureType,
    pub material_distribution: MaterialDistribution,
    pub structural_integrity: f32,
    pub triggers: Vec<DestructionTrigger>,
    pub environmental_effects: Vec<EnvironmentalEffect>,
}

/// Types of destructible structures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StructureType {
    Building,
    Bridge,
    Tower,
    Wall,
    Terrain,
    Custom,
}

/// A destructible environment
#[derive(Debug)]
pub struct DestructibleEnvironment {
    pub id: String,
    pub dimensions: Vec3,
    pub voxel_data: HashMap<VoxelPos, VoxelType>,
    pub integrity_map: HashMap<VoxelPos, f32>,
    pub destruction_triggers: Vec<DestructionTrigger>,
    pub environmental_effects: Vec<EnvironmentalEffect>,
    pub structural_supports: HashMap<VoxelPos, SupportType>,
}

/// Types of structural support
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportType {
    Column,
    Beam,
    Pillar,
    Cable,
    Ring,
    Foundation,
}

/// Active destruction event
#[derive(Debug)]
pub struct DestructionEvent {
    pub id: String,
    pub world_id: String,
    pub position: Vec3,
    pub destruction_type: DestructionType,
    pub force: f32,
    pub radius: f32,
    pub start_time: Instant,
    pub affected_voxels: HashSet<(i32, i32, i32)>,
    pub phase: DestructionPhase,
    pub particles: Vec<ParticleEffect>,
    pub debris: Vec<DebrisParticle>,
}

impl Clone for DestructionEvent {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            world_id: self.world_id.clone(),
            position: self.position,
            destruction_type: self.destruction_type,
            force: self.force,
            radius: self.radius,
            start_time: Instant::now(), // Use current time for cloned event
            affected_voxels: self.affected_voxels.clone(),
            phase: self.phase,
            particles: self.particles.clone(),
            debris: self.debris.clone(),
        }
    }
}

/// Phases of destruction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DestructionPhase {
    Initial,
    Propagating,
    Collapsing,
    Settling,
    Complete,
}

/// Types of destruction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DestructionType {
    Explosion,
    Impact,
    Erosion,
    Cutting,
    Melting,
    Freezing,
}

/// Debris particle for physics simulation
#[derive(Debug, Clone)]
pub struct DebrisParticle {
    pub position: Vec3,
    pub velocity: Vec3,
    pub angular_velocity: Vec3,
    pub voxel_type: VoxelType,
    pub life_time: f32,
}

/// Destruction trigger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DestructionTrigger {
    pub trigger_type: TriggerType,
    pub position: Vec3,
    pub radius: f32,
    pub force: f32,
    pub activation_condition: ActivationCondition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerType {
    Proximity,
    Impact,
    Timer,
    External,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivationCondition {
    PlayerNearby(f32),
    HealthBelow(f32),
    TimeElapsed(f32),
    ExternalSignal(String),
}

/// Environmental effects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentalEffect {
    pub effect_type: EnvironmentalEffectType,
    pub intensity: f32,
    pub area: EffectArea,
    pub duration: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnvironmentalEffectType {
    Wind,
    Earthquake,
    Erosion,
    Fire,
    Flood,
    Freeze,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectArea {
    pub center: Vec3,
    pub radius: f32,
    pub shape: AreaShape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AreaShape {
    Sphere,
    Cylinder,
    Box,
    Cone,
}

/// Material distribution for generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialDistribution {
    pub primary_material: VoxelType,
    pub secondary_materials: Vec<(VoxelType, f32)>, // material, probability
    pub structural_materials: Vec<VoxelType>,
}

/// Structural integrity analysis system
#[derive(Debug)]
pub struct StructuralIntegritySystem {
    config: IntegrityConfig,
}

impl StructuralIntegritySystem {
    fn new(config: IntegrityConfig) -> Self {
        Self { config }
    }

    fn analyze_structure(&self, structure: &mut DestructibleEnvironment) -> RobinResult<()> {
        // Analyze load-bearing elements and calculate integrity values
        let mut integrity_updates = HashMap::new();
        for (pos, voxel_type) in &structure.voxel_data {
            let integrity = self.calculate_voxel_integrity(*pos, voxel_type, structure);
            integrity_updates.insert(*pos, integrity);
        }
        structure.integrity_map.extend(integrity_updates);
        Ok(())
    }

    fn calculate_voxel_integrity(&self, pos: VoxelPos, voxel_type: &VoxelType, structure: &DestructibleEnvironment) -> f32 {
        let base_integrity = match voxel_type {
            VoxelType::Air => 0.0,
            VoxelType::Solid => 40.0,
            VoxelType::Liquid => 5.0,
            VoxelType::Gas => 1.0,
            VoxelType::Light => 0.0,
            VoxelType::Stone => 80.0,
            VoxelType::Wood => 40.0,
            VoxelType::Metal => 120.0,
            VoxelType::Glass => 10.0,
            VoxelType::Concrete => 100.0,
            VoxelType::Brick => 60.0,
            VoxelType::Custom(_) => 50.0, // TODO: Add destruction_resistance field to custom voxel properties
        };

        // Modify based on structural position
        let support_bonus = if structure.structural_supports.contains_key(&pos) {
            20.0
        } else {
            0.0
        };

        // Modify based on surrounding support
        let neighbor_support = self.calculate_neighbor_support(pos, structure);

        base_integrity + support_bonus + neighbor_support
    }

    fn calculate_neighbor_support(&self, pos: VoxelPos, structure: &DestructibleEnvironment) -> f32 {
        let mut support = 0.0;
        let mut neighbors = 0;

        for x in -1i32..=1 {
            for y in -1i32..=1 {
                for z in -1i32..=1 {
                    if x == 0 && y == 0 && z == 0 { continue; }
                    
                    let neighbor_pos = VoxelPos {
                        x: pos.x as i32 + x,
                        y: pos.y as i32 + y,
                        z: pos.z as i32 + z,
                    };
                    
                    if let Some(neighbor_type) = structure.voxel_data.get(&neighbor_pos) {
                        if *neighbor_type != VoxelType::Air {
                            neighbors += 1;
                            support += match neighbor_type {
                                VoxelType::Metal => 5.0,
                                VoxelType::Concrete => 4.0,
                                VoxelType::Stone => 3.0,
                                VoxelType::Brick => 2.0,
                                VoxelType::Wood => 1.0,
                                _ => 0.5,
                            };
                        }
                    }
                }
            }
        }

        if neighbors > 0 {
            support / neighbors as f32
        } else {
            0.0
        }
    }
}

/// Debris physics system
#[derive(Debug)]
pub struct DebrisPhysics {
    config: PhysicsConfig,
    active_debris: Vec<DebrisParticle>,
}

impl DebrisPhysics {
    fn new(config: PhysicsConfig) -> Self {
        Self {
            config,
            active_debris: Vec::new(),
        }
    }

    fn update(&mut self, delta_time: f32) {
        // Update all active debris
        for debris in &mut self.active_debris {
            Self::update_debris_particle(debris, delta_time, &self.config);
        }

        // Remove expired debris
        self.active_debris.retain(|debris| debris.life_time > 0.0);
    }

    fn update_debris_particle(debris: &mut DebrisParticle, delta_time: f32, config: &PhysicsConfig) {
        // Apply gravity
        debris.velocity.y -= config.gravity * delta_time;
        
        // Apply air resistance
        debris.velocity = debris.velocity * (1.0 - config.air_resistance * delta_time);
        debris.angular_velocity = debris.angular_velocity * (1.0 - config.angular_damping * delta_time);
        
        // Update position
        debris.position = debris.position + debris.velocity * delta_time;
        
        // Update lifetime
        debris.life_time -= delta_time;
    }

    fn add_debris(&mut self, debris: DebrisParticle) {
        if self.active_debris.len() < self.config.max_debris_particles {
            self.active_debris.push(debris);
        }
    }
}

/// Particle effects for destruction
#[derive(Debug)]
pub struct DestructionParticles {
    config: ParticleConfig,
    active_effects: Vec<ParticleEffect>,
}

impl DestructionParticles {
    fn new(config: ParticleConfig) -> Self {
        Self {
            config,
            active_effects: Vec::new(),
        }
    }

    fn update(&mut self, delta_time: f32) {
        for effect in &mut self.active_effects {
            effect.lifetime -= delta_time;
        }

        self.active_effects.retain(|effect| effect.lifetime > 0.0);
    }
}

/// Particle effect
#[derive(Debug, Clone)]
pub struct ParticleEffect {
    pub effect_type: ParticleEffectType,
    pub position: Vec3,
    pub intensity: f32,
    pub lifetime: f32,
    pub color: Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleEffectType {
    Dust,
    Smoke,
    Fire,
    Sparks,
    Debris,
}

/// Configuration structs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityConfig {
    pub enable_structural_analysis: bool,
    pub support_calculation_depth: u32,
    pub integrity_update_frequency: f32,
}

impl Default for IntegrityConfig {
    fn default() -> Self {
        Self {
            enable_structural_analysis: true,
            support_calculation_depth: 5,
            integrity_update_frequency: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsConfig {
    pub gravity: f32,
    pub air_resistance: f32,
    pub angular_damping: f32,
    pub max_debris_particles: usize,
    pub collision_detection: bool,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            gravity: 9.81,
            air_resistance: 0.02,
            angular_damping: 0.05,
            max_debris_particles: 1000,
            collision_detection: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleConfig {
    pub max_particle_effects: usize,
    pub default_particle_lifetime: f32,
    pub enable_particle_physics: bool,
}

impl Default for ParticleConfig {
    fn default() -> Self {
        Self {
            max_particle_effects: 500,
            default_particle_lifetime: 5.0,
            enable_particle_physics: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PerformanceMode {
    High,
    Balanced,
    Performance,
}

/// Result of destruction processing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DestructionResult {
    Continuing,
    Complete,
    Failed,
}

/// Custom voxel properties for terrain and special materials
#[derive(Debug, Clone)]
pub struct CustomVoxelProperties {
    pub hardness: f32,
    pub destruction_resistance: f32,
    pub color: Color,
    pub emissive: bool,
}

/// Destruction statistics
#[derive(Debug, Default)]
pub struct DestructionStats {
    pub environments_generated: u64,
    pub destruction_events: u64,
    pub voxels_destroyed: usize,
    pub total_generation_time: f32,
    pub update_time: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_destruction_system_creation() {
        let config = DestructionConfig::default();
        let system = DestructionSystem::new(config);
        
        assert!(system.active_destructions.is_empty());
        assert_eq!(system.stats.environments_generated, 0);
    }

    #[test]
    fn test_can_destroy_voxel() {
        let config = DestructionConfig::default();
        let system = DestructionSystem::new(config);
        
        // Glass should be easily destroyed
        assert!(system.can_destroy_voxel(&VoxelType::Glass, &DestructionType::Impact, 10.0));
        
        // Metal should require more force
        assert!(!system.can_destroy_voxel(&VoxelType::Metal, &DestructionType::Impact, 10.0));
        assert!(system.can_destroy_voxel(&VoxelType::Metal, &DestructionType::Explosion, 100.0));
    }

    #[test]
    fn test_destruction_radius_calculation() {
        let config = DestructionConfig::default();
        let system = DestructionSystem::new(config);
        
        let explosion_radius = system.calculate_destruction_radius(100.0, &DestructionType::Explosion);
        let impact_radius = system.calculate_destruction_radius(100.0, &DestructionType::Impact);
        
        assert!(explosion_radius > impact_radius);
    }
}