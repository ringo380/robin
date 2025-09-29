/// VoxelPhysicsSystem - Physics-based voxel interactions with rapier3d
///
/// This system provides realistic physics simulation for voxel interactions including:
/// - Dynamic voxel blocks that can fall, bounce, and collide
/// - Physics-based building with realistic placement constraints
/// - Particle effects for block breaking with debris simulation
/// - Realistic terrain interaction and collision detection
/// - Support for different voxel material properties

use crate::renderer::mesh::{Mesh, Vertex};
use cgmath::{Vector3, Point3, Matrix4, InnerSpace};
use robin::engine::physics3d::{
    PhysicsWorld3D, Physics3DConfig, PhysicsHandle, BodyDescriptor,
    ColliderShape3D, BodyType3D, CollisionEvent3D
};
use robin::engine::generation::voxel_system::{VoxelWorld, VoxelType};
use robin::engine::error::RobinResult;
use std::collections::HashMap;
use rapier3d::prelude::*;

/// Properties defining how different voxel materials behave in physics
#[derive(Debug, Clone)]
pub struct VoxelPhysicsProperties {
    /// Mass of a single voxel block (kg)
    pub mass: f32,
    /// Friction coefficient (0.0 = no friction, 1.0 = high friction)
    pub friction: f32,
    /// Restitution/bounciness (0.0 = no bounce, 1.0 = perfect bounce)
    pub restitution: f32,
    /// Whether this material can fall under gravity
    pub can_fall: bool,
    /// Hardness - affects breaking and particle generation
    pub hardness: f32,
    /// Density - affects physics calculations
    pub density: f32,
    /// Whether blocks of this type can be stacked
    pub stackable: bool,
    /// Maximum stack height before collapse
    pub max_stack_height: u32,
}

impl VoxelPhysicsProperties {
    /// Get physics properties for a voxel type
    pub fn for_voxel_type(voxel_type: &VoxelType) -> Self {
        match voxel_type {
            VoxelType::Air => Self {
                mass: 0.0,
                friction: 0.0,
                restitution: 0.0,
                can_fall: false,
                hardness: 0.0,
                density: 0.0,
                stackable: false,
                max_stack_height: 0,
            },
            VoxelType::Stone => Self {
                mass: 50.0,
                friction: 0.8,
                restitution: 0.1,
                can_fall: true,
                hardness: 8.0,
                density: 2.7,
                stackable: true,
                max_stack_height: 64,
            },
            VoxelType::Wood => Self {
                mass: 15.0,
                friction: 0.6,
                restitution: 0.3,
                can_fall: true,
                hardness: 4.0,
                density: 0.8,
                stackable: true,
                max_stack_height: 32,
            },
            VoxelType::Glass => Self {
                mass: 25.0,
                friction: 0.3,
                restitution: 0.05,
                can_fall: true,
                hardness: 2.0,
                density: 2.5,
                stackable: true,
                max_stack_height: 16,
            },
            VoxelType::Metal => Self {
                mass: 80.0,
                friction: 0.5,
                restitution: 0.2,
                can_fall: true,
                hardness: 9.0,
                density: 7.8,
                stackable: true,
                max_stack_height: 48,
            },
            VoxelType::Brick => Self {
                mass: 35.0,
                friction: 0.7,
                restitution: 0.1,
                can_fall: true,
                hardness: 6.0,
                density: 2.0,
                stackable: true,
                max_stack_height: 40,
            },
            VoxelType::Concrete => Self {
                mass: 45.0,
                friction: 0.8,
                restitution: 0.05,
                can_fall: true,
                hardness: 7.0,
                density: 2.4,
                stackable: true,
                max_stack_height: 56,
            },
            VoxelType::Solid => Self {
                mass: 30.0,
                friction: 0.6,
                restitution: 0.2,
                can_fall: true,
                hardness: 5.0,
                density: 2.0,
                stackable: true,
                max_stack_height: 32,
            },
            VoxelType::Liquid => Self {
                mass: 10.0,
                friction: 0.1,
                restitution: 0.0,
                can_fall: true,
                hardness: 1.0,
                density: 1.0,
                stackable: false,
                max_stack_height: 1,
            },
            VoxelType::Gas => Self {
                mass: 0.1,
                friction: 0.0,
                restitution: 0.0,
                can_fall: false,
                hardness: 0.1,
                density: 0.001,
                stackable: false,
                max_stack_height: 1,
            },
            VoxelType::Light => Self {
                mass: 1.0,
                friction: 0.2,
                restitution: 0.8,
                can_fall: false,
                hardness: 1.0,
                density: 0.1,
                stackable: true,
                max_stack_height: 8,
            },
            VoxelType::Custom(_) => Self {
                mass: 20.0,
                friction: 0.5,
                restitution: 0.3,
                can_fall: true,
                hardness: 5.0,
                density: 1.5,
                stackable: true,
                max_stack_height: 24,
            },
        }
    }
}

/// A physics particle representing voxel debris
#[derive(Debug, Clone)]
pub struct VoxelDebrisParticle {
    /// Physics handle for the particle
    pub physics_handle: PhysicsHandle,
    /// Original voxel type (affects appearance and behavior)
    pub voxel_type: VoxelType,
    /// Particle lifetime remaining (seconds)
    pub lifetime: f32,
    /// Maximum lifetime for fade calculations
    pub max_lifetime: f32,
    /// Size of the debris particle (0.0 - 1.0 scale of full voxel)
    pub size: f32,
    /// Position for rendering
    pub position: Vector3<f32>,
    /// Velocity for trail effects
    pub velocity: Vector3<f32>,
}

/// A dynamic voxel block that exists in the physics world
#[derive(Debug, Clone)]
pub struct DynamicVoxelBlock {
    /// Physics handle for the block
    pub physics_handle: PhysicsHandle,
    /// Voxel type and properties
    pub voxel_type: VoxelType,
    /// Grid position in the voxel world (when placed)
    pub grid_position: Option<Vector3<i32>>,
    /// Whether this block is currently being placed
    pub is_being_placed: bool,
    /// Time since creation (for settling behavior)
    pub age: f32,
}

/// Configuration for voxel physics behavior
#[derive(Debug, Clone)]
pub struct VoxelPhysicsConfig {
    /// Enable falling voxel physics
    pub enable_falling_blocks: bool,
    /// Enable debris particles when breaking blocks
    pub enable_debris_particles: bool,
    /// Maximum number of debris particles active at once
    pub max_debris_particles: usize,
    /// Debris particle lifetime (seconds)
    pub debris_lifetime: f32,
    /// Enable realistic stacking physics
    pub enable_stacking_physics: bool,
    /// Enable collision sounds and effects
    pub enable_collision_effects: bool,
    /// Gravity multiplier for voxel objects
    pub voxel_gravity_scale: f32,
    /// Air resistance for falling blocks
    pub air_resistance: f32,
    /// Minimum velocity for physics updates
    pub velocity_threshold: f32,
}

impl Default for VoxelPhysicsConfig {
    fn default() -> Self {
        Self {
            enable_falling_blocks: true,
            enable_debris_particles: true,
            max_debris_particles: 100,
            debris_lifetime: 3.0,
            enable_stacking_physics: true,
            enable_collision_effects: true,
            voxel_gravity_scale: 1.0,
            air_resistance: 0.02,
            velocity_threshold: 0.01,
        }
    }
}

/// Events generated by voxel physics interactions
#[derive(Debug, Clone)]
pub enum VoxelPhysicsEvent {
    /// A voxel block has settled into the world grid
    BlockSettled {
        handle: PhysicsHandle,
        voxel_type: VoxelType,
        position: Vector3<i32>,
    },
    /// A voxel block was broken and generated debris
    BlockBroken {
        voxel_type: VoxelType,
        position: Vector3<f32>,
        debris_count: usize,
    },
    /// Two voxel blocks collided
    BlockCollision {
        handle1: PhysicsHandle,
        handle2: PhysicsHandle,
        impact_force: f32,
        position: Vector3<f32>,
    },
    /// A structure collapsed due to physics
    StructureCollapse {
        center: Vector3<f32>,
        affected_blocks: Vec<PhysicsHandle>,
    },
}

/// Main voxel physics system integrating rapier3d with voxel world
pub struct VoxelPhysicsSystem {
    /// Rapier3D physics world
    physics_world: PhysicsWorld3D,
    /// Configuration for physics behavior
    config: VoxelPhysicsConfig,
    /// Active dynamic voxel blocks
    dynamic_blocks: HashMap<PhysicsHandle, DynamicVoxelBlock>,
    /// Active debris particles
    debris_particles: Vec<VoxelDebrisParticle>,
    /// Events generated this frame
    events: Vec<VoxelPhysicsEvent>,
    /// Mesh for rendering debris particles
    debris_mesh: Mesh,
    /// Static terrain colliders (for performance)
    terrain_colliders: HashMap<Vector3<i32>, PhysicsHandle>,
    /// Performance metrics
    active_physics_objects: usize,
    last_cleanup_time: f32,
}

impl VoxelPhysicsSystem {
    /// Create a new voxel physics system
    pub fn new(config: VoxelPhysicsConfig) -> Self {
        let physics_config = Physics3DConfig {
            gravity: Vector3::new(0.0, -9.81 * config.voxel_gravity_scale, 0.0),
            timestep: 1.0 / 60.0,
            max_velocity: 50.0,
            solver_iterations: 8,
        };

        let physics_world = PhysicsWorld3D::new(physics_config);

        Self {
            physics_world,
            config,
            dynamic_blocks: HashMap::new(),
            debris_particles: Vec::new(),
            events: Vec::new(),
            debris_mesh: Mesh::new(),
            terrain_colliders: HashMap::new(),
            active_physics_objects: 0,
            last_cleanup_time: 0.0,
        }
    }

    /// Initialize GPU resources for rendering
    pub fn initialize(&mut self, device: &metal::DeviceRef) {
        self.create_debris_mesh();
        self.debris_mesh.create_buffers(device);
    }

    /// Create a mesh for rendering debris particles
    fn create_debris_mesh(&mut self) {
        self.debris_mesh.vertices.clear();
        self.debris_mesh.indices.clear();

        // Create a simple cube mesh for debris particles
        let positions = [
            // Front face
            [-0.5, -0.5,  0.5], [ 0.5, -0.5,  0.5], [ 0.5,  0.5,  0.5], [-0.5,  0.5,  0.5],
            // Back face
            [-0.5, -0.5, -0.5], [-0.5,  0.5, -0.5], [ 0.5,  0.5, -0.5], [ 0.5, -0.5, -0.5],
        ];

        let normals = [
            [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0],
            [0.0, 0.0, -1.0], [0.0, 0.0, -1.0], [0.0, 0.0, -1.0], [0.0, 0.0, -1.0],
        ];

        for (pos, normal) in positions.iter().zip(normals.iter()) {
            self.debris_mesh.vertices.push(Vertex::new(
                *pos,
                [0.5, 0.5, 0.5], // Default gray color
                *normal,
            ));
        }

        // Cube indices
        let indices = [
            0, 1, 2, 2, 3, 0, // Front
            4, 5, 6, 6, 7, 4, // Back
            0, 3, 5, 5, 4, 0, // Left
            1, 7, 6, 6, 2, 1, // Right
            3, 2, 6, 6, 5, 3, // Top
            0, 4, 7, 7, 1, 0, // Bottom
        ];

        self.debris_mesh.indices.extend(&indices);
        self.debris_mesh.vertex_count = self.debris_mesh.vertices.len();
        self.debris_mesh.index_count = self.debris_mesh.indices.len();
    }

    /// Add a dynamic voxel block to the physics world
    pub fn add_dynamic_block(
        &mut self,
        voxel_type: VoxelType,
        position: Vector3<f32>,
        initial_velocity: Option<Vector3<f32>>,
    ) -> RobinResult<PhysicsHandle> {
        let properties = VoxelPhysicsProperties::for_voxel_type(&voxel_type);

        let mut descriptor = BodyDescriptor {
            body_type: BodyType3D::Dynamic,
            position,
            velocity: initial_velocity.unwrap_or(Vector3::new(0.0, 0.0, 0.0)),
            mass: properties.mass,
            friction: properties.friction,
            restitution: properties.restitution,
            linear_damping: self.config.air_resistance,
            gravity_scale: if properties.can_fall { 1.0 } else { 0.0 },
            ..Default::default()
        };

        let shape = ColliderShape3D::voxel_block();
        let user_data = format!("voxel_{:?}", voxel_type);

        let handle = self.physics_world.create_body(descriptor, shape, Some(user_data))?;

        let dynamic_block = DynamicVoxelBlock {
            physics_handle: handle,
            voxel_type,
            grid_position: None,
            is_being_placed: false,
            age: 0.0,
        };

        self.dynamic_blocks.insert(handle, dynamic_block);
        self.active_physics_objects += 1;

        Ok(handle)
    }

    /// Create debris particles when a voxel block is broken
    pub fn create_debris(
        &mut self,
        voxel_type: VoxelType,
        position: Vector3<f32>,
        impact_velocity: Vector3<f32>,
        debris_count: usize,
    ) -> RobinResult<()> {
        if !self.config.enable_debris_particles {
            return Ok(());
        }

        let properties = VoxelPhysicsProperties::for_voxel_type(&voxel_type);
        let actual_count = debris_count.min(self.config.max_debris_particles - self.debris_particles.len());

        for i in 0..actual_count {
            // Random offset from break position
            let random_offset = Vector3::new(
                (i as f32 * 0.2) % 1.0 - 0.5,
                (i as f32 * 0.3) % 1.0 - 0.5,
                (i as f32 * 0.4) % 1.0 - 0.5,
            ) * 0.3;

            let debris_position = position + random_offset;

            // Random velocity based on impact
            let random_velocity = Vector3::new(
                (i as f32 * 0.7) % 2.0 - 1.0,
                (i as f32 * 0.8) % 1.0 + 0.5, // Bias upward
                (i as f32 * 0.9) % 2.0 - 1.0,
            ) * 2.0 + impact_velocity * 0.3;

            let descriptor = BodyDescriptor {
                body_type: BodyType3D::Dynamic,
                position: debris_position,
                velocity: random_velocity,
                mass: properties.mass * 0.1, // Debris is lighter
                friction: properties.friction,
                restitution: properties.restitution * 1.5, // More bouncy
                linear_damping: self.config.air_resistance * 2.0,
                gravity_scale: 1.0,
                ..Default::default()
            };

            let size = 0.1 + (i as f32 * 0.05) % 0.15; // Variable debris size
            let shape = ColliderShape3D::Box {
                half_extents: Vector3::new(size, size, size),
            };

            let user_data = format!("debris_{:?}_{}", voxel_type, i);
            let handle = self.physics_world.create_body(descriptor, shape, Some(user_data))?;

            let debris = VoxelDebrisParticle {
                physics_handle: handle,
                voxel_type,
                lifetime: self.config.debris_lifetime,
                max_lifetime: self.config.debris_lifetime,
                size: size * 2.0, // Full diameter
                position: debris_position,
                velocity: random_velocity,
            };

            self.debris_particles.push(debris);
        }

        // Generate event
        self.events.push(VoxelPhysicsEvent::BlockBroken {
            voxel_type,
            position,
            debris_count: actual_count,
        });

        Ok(())
    }

    /// Add static terrain collision for a voxel chunk
    pub fn add_terrain_collider(
        &mut self,
        chunk_position: Vector3<i32>,
        voxel_world: &VoxelWorld,
    ) -> RobinResult<()> {
        // Skip if already exists
        if self.terrain_colliders.contains_key(&chunk_position) {
            return Ok(());
        }

        // Create a simplified collision mesh for the chunk
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let chunk_size = 32; // Standard chunk size
        let base_pos = Vector3::new(
            chunk_position.x as f32 * chunk_size as f32,
            chunk_position.y as f32 * chunk_size as f32,
            chunk_position.z as f32 * chunk_size as f32,
        );

        // Sample voxels in the chunk and create collision geometry
        for x in 0..chunk_size {
            for y in 0..chunk_size {
                for z in 0..chunk_size {
                    let world_pos = Vector3::new(
                        base_pos.x + x as f32,
                        base_pos.y + y as f32,
                        base_pos.z + z as f32,
                    );

                    if let Some(voxel_type) = voxel_world.get_voxel(world_pos) {
                        if voxel_type != VoxelType::Air {
                            // Add a cube for this voxel
                            let cube_vertices = self.generate_cube_vertices(world_pos);
                            let base_index = vertices.len();
                            vertices.extend(cube_vertices);

                            // Add cube indices
                            let cube_indices = [
                                0, 1, 2, 2, 3, 0, // Front
                                4, 5, 6, 6, 7, 4, // Back
                                0, 3, 5, 5, 4, 0, // Left
                                1, 7, 6, 6, 2, 1, // Right
                                3, 2, 6, 6, 5, 3, // Top
                                0, 4, 7, 7, 1, 0, // Bottom
                            ];

                            for &idx in &cube_indices {
                                indices.push([
                                    (base_index + idx as usize) as u32,
                                    (base_index + (idx + 1) as usize) as u32,
                                    (base_index + (idx + 2) as usize) as u32,
                                ]);
                            }
                        }
                    }
                }
            }
        }

        // Only create collider if we have geometry
        if !vertices.is_empty() {
            let descriptor = BodyDescriptor::static_block(base_pos);
            let shape = ColliderShape3D::Mesh { vertices, indices };
            let user_data = format!("terrain_chunk_{}_{}_{}",
                chunk_position.x, chunk_position.y, chunk_position.z);

            let handle = self.physics_world.create_body(descriptor, shape, Some(user_data))?;
            self.terrain_colliders.insert(chunk_position, handle);
        }

        Ok(())
    }

    /// Generate vertices for a unit cube at the given position
    fn generate_cube_vertices(&self, position: Vector3<f32>) -> Vec<Point3<f32>> {
        let half_size = 0.5;
        vec![
            // Front face
            Point3::new(position.x - half_size, position.y - half_size, position.z + half_size),
            Point3::new(position.x + half_size, position.y - half_size, position.z + half_size),
            Point3::new(position.x + half_size, position.y + half_size, position.z + half_size),
            Point3::new(position.x - half_size, position.y + half_size, position.z + half_size),
            // Back face
            Point3::new(position.x - half_size, position.y - half_size, position.z - half_size),
            Point3::new(position.x - half_size, position.y + half_size, position.z - half_size),
            Point3::new(position.x + half_size, position.y + half_size, position.z - half_size),
            Point3::new(position.x + half_size, position.y - half_size, position.z - half_size),
        ]
    }

    /// Update the physics simulation
    pub fn update(&mut self, delta_time: f32, voxel_world: &mut VoxelWorld) -> RobinResult<()> {
        // Clear events from last frame
        self.events.clear();

        // Step physics simulation
        self.physics_world.step(delta_time)?;

        // Update dynamic blocks
        self.update_dynamic_blocks(delta_time, voxel_world)?;

        // Update debris particles
        self.update_debris_particles(delta_time)?;

        // Process collision events
        self.process_collision_events();

        // Periodic cleanup
        self.last_cleanup_time += delta_time;
        if self.last_cleanup_time >= 5.0 {
            self.cleanup_inactive_objects()?;
            self.last_cleanup_time = 0.0;
        }

        Ok(())
    }

    /// Update dynamic voxel blocks
    fn update_dynamic_blocks(&mut self, delta_time: f32, voxel_world: &mut VoxelWorld) -> RobinResult<()> {
        let mut blocks_to_settle = Vec::new();

        for (handle, block) in &mut self.dynamic_blocks {
            block.age += delta_time;

            // Get current physics state
            if let Some(position) = self.physics_world.get_body_position(*handle) {
                if let Some(velocity) = self.physics_world.get_body_velocity(*handle) {
                    // Check if block should settle into grid
                    if velocity.magnitude() < self.config.velocity_threshold && block.age > 1.0 {
                        let grid_pos = Vector3::new(
                            position.x.round() as i32,
                            position.y.round() as i32,
                            position.z.round() as i32,
                        );

                        // Check if grid position is valid and empty
                        if let Some(existing_voxel) = voxel_world.get_voxel(Vector3::new(
                            grid_pos.x as f32,
                            grid_pos.y as f32,
                            grid_pos.z as f32,
                        )) {
                            if existing_voxel == VoxelType::Air {
                                blocks_to_settle.push((*handle, grid_pos));
                            }
                        }
                    }
                }
            }
        }

        // Settle blocks into the grid
        for (handle, grid_pos) in blocks_to_settle {
            if let Some(block) = self.dynamic_blocks.remove(&handle) {
                // Place voxel in world
                voxel_world.set_voxel(
                    Vector3::new(grid_pos.x as f32, grid_pos.y as f32, grid_pos.z as f32),
                    block.voxel_type,
                );

                // Remove from physics world
                self.physics_world.remove_body(handle)?;
                self.active_physics_objects -= 1;

                // Generate settlement event
                self.events.push(VoxelPhysicsEvent::BlockSettled {
                    handle,
                    voxel_type: block.voxel_type,
                    position: grid_pos,
                });
            }
        }

        Ok(())
    }

    /// Update debris particles
    fn update_debris_particles(&mut self, delta_time: f32) -> RobinResult<()> {
        let mut particles_to_remove = Vec::new();

        for (i, particle) in self.debris_particles.iter_mut().enumerate() {
            particle.lifetime -= delta_time;

            // Update position from physics
            if let Some(position) = self.physics_world.get_body_position(particle.physics_handle) {
                particle.position = position;
            }
            if let Some(velocity) = self.physics_world.get_body_velocity(particle.physics_handle) {
                particle.velocity = velocity;
            }

            // Mark for removal if lifetime expired
            if particle.lifetime <= 0.0 {
                particles_to_remove.push(i);
            }
        }

        // Remove expired particles
        for &i in particles_to_remove.iter().rev() {
            let particle = self.debris_particles.remove(i);
            self.physics_world.remove_body(particle.physics_handle)?;
        }

        Ok(())
    }

    /// Process collision events from physics world
    fn process_collision_events(&mut self) {
        for collision in self.physics_world.collision_events() {
            if collision.started {
                // Check if this is a significant collision
                if collision.impulse > 5.0 {
                    self.events.push(VoxelPhysicsEvent::BlockCollision {
                        handle1: collision.handle1,
                        handle2: collision.handle2,
                        impact_force: collision.impulse,
                        position: collision.contact_point,
                    });
                }
            }
        }
    }

    /// Clean up inactive physics objects
    fn cleanup_inactive_objects(&mut self) -> RobinResult<()> {
        let mut blocks_to_remove = Vec::new();

        for (handle, block) in &self.dynamic_blocks {
            if let Some(position) = self.physics_world.get_body_position(*handle) {
                // Remove blocks that have fallen too far
                if position.y < -100.0 {
                    blocks_to_remove.push(*handle);
                }
            }
        }

        for handle in blocks_to_remove {
            self.dynamic_blocks.remove(&handle);
            self.physics_world.remove_body(handle)?;
            self.active_physics_objects -= 1;
        }

        Ok(())
    }

    /// Raycast into the physics world
    pub fn raycast(&self, origin: Vector3<f32>, direction: Vector3<f32>, max_distance: f32)
        -> Option<(PhysicsHandle, f32, Vector3<f32>)> {
        self.physics_world.raycast(origin, direction, max_distance)
    }

    /// Apply force to a dynamic block
    pub fn apply_force_to_block(&mut self, handle: PhysicsHandle, force: Vector3<f32>) -> RobinResult<()> {
        self.physics_world.apply_force(handle, force)
    }

    /// Apply impulse to a dynamic block
    pub fn apply_impulse_to_block(&mut self, handle: PhysicsHandle, impulse: Vector3<f32>) -> RobinResult<()> {
        self.physics_world.apply_impulse(handle, impulse)
    }

    /// Get physics events from this frame
    pub fn get_events(&self) -> &[VoxelPhysicsEvent] {
        &self.events
    }

    /// Get all active dynamic blocks
    pub fn get_dynamic_blocks(&self) -> &HashMap<PhysicsHandle, DynamicVoxelBlock> {
        &self.dynamic_blocks
    }

    /// Get all debris particles
    pub fn get_debris_particles(&self) -> &[VoxelDebrisParticle] {
        &self.debris_particles
    }

    /// Get performance metrics
    pub fn get_performance_metrics(&self) -> VoxelPhysicsMetrics {
        VoxelPhysicsMetrics {
            active_dynamic_blocks: self.dynamic_blocks.len(),
            active_debris_particles: self.debris_particles.len(),
            terrain_colliders: self.terrain_colliders.len(),
            total_physics_objects: self.active_physics_objects,
        }
    }

    /// Render debug visualization
    pub fn render_debug(
        &self,
        encoder: &metal::RenderCommandEncoderRef,
        renderer: &crate::renderer::MetalRenderer,
        view_matrix: &Matrix4<f32>,
    ) {
        // Render dynamic block wireframes
        for (handle, block) in &self.dynamic_blocks {
            if let Some(position) = self.physics_world.get_body_position(*handle) {
                // Create a simple wireframe cube at the block position
                // This would be implemented with the renderer's debug wireframe capability
            }
        }

        // Render debris particles
        for particle in &self.debris_particles {
            // Render small cubes for debris particles
            // This would use the debris mesh with appropriate transforms
        }
    }
}

/// Performance metrics for the voxel physics system
#[derive(Debug, Clone)]
pub struct VoxelPhysicsMetrics {
    pub active_dynamic_blocks: usize,
    pub active_debris_particles: usize,
    pub terrain_colliders: usize,
    pub total_physics_objects: usize,
}

/// Helper function to get voxel color for debris rendering
pub fn get_voxel_color_for_debris(voxel_type: &VoxelType) -> [f32; 3] {
    match voxel_type {
        VoxelType::Air => [0.0, 0.0, 0.0],
        VoxelType::Stone => [0.5, 0.5, 0.5],
        VoxelType::Wood => [0.6, 0.4, 0.2],
        VoxelType::Glass => [0.8, 0.9, 1.0],
        VoxelType::Metal => [0.7, 0.7, 0.8],
        VoxelType::Brick => [0.8, 0.4, 0.3],
        VoxelType::Concrete => [0.6, 0.6, 0.6],
        VoxelType::Solid => [0.4, 0.4, 0.4],
        VoxelType::Liquid => [0.2, 0.4, 0.8],
        VoxelType::Gas => [0.9, 0.9, 0.9],
        VoxelType::Light => [1.0, 1.0, 0.8],
        VoxelType::Custom(_) => [0.5, 0.5, 0.5],
    }
}