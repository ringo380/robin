use nalgebra::{Vector3, Point3};
use super::{CharacterState, MovementMode};
use std::collections::HashMap;
use rapier3d::prelude::*;
use rapier3d::dynamics::{RigidBodySet, RigidBodyHandle, IslandManager, ImpulseJointSet, MultibodyJointSet, CCDSolver};
use rapier3d::geometry::{ColliderSet, ColliderHandle, ColliderBuilder, BroadPhase, NarrowPhase};
use rapier3d::pipeline::{PhysicsPipeline, QueryPipeline};

pub struct CharacterPhysics {
    // Physics constants
    gravity: f32,
    terminal_velocity: f32,
    ground_check_distance: f32,

    // Rapier3D physics world
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    query_pipeline: QueryPipeline,

    // Character-specific physics
    character_body_handle: Option<RigidBodyHandle>,
    character_collider_handle: Option<ColliderHandle>,
    character_radius: f32,
    character_height: f32,

    // Legacy collision system (for compatibility)
    collision_layers: HashMap<String, Vec<CollisionBox>>,

    // Ground detection
    ground_normal: Vector3<f32>,
    slope_limit: f32,
}

#[derive(Clone, Debug)]
pub struct CollisionBox {
    pub min: Point3<f32>,
    pub max: Point3<f32>,
    pub material: String,
    pub solid: bool,
}

impl CharacterPhysics {
    pub fn new() -> Self {
        Self {
            gravity: -18.0,
            terminal_velocity: -50.0,
            ground_check_distance: 0.1,

            // Initialize Rapier3D physics world
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(),

            // Character physics handles
            character_body_handle: None,
            character_collider_handle: None,
            character_radius: 0.3,
            character_height: 1.8,

            // Legacy collision system
            collision_layers: HashMap::new(),

            ground_normal: Vector3::new(0.0, 1.0, 0.0),
            slope_limit: 45.0_f32.to_radians(),
        }
    }

    pub fn update_character(&mut self, state: &mut CharacterState, delta_time: f32) {
        match state.movement_mode {
            MovementMode::Fly | MovementMode::Noclip => {
                // No physics for flying/noclip modes
                return;
            }
            _ => {
                // Update rapier3d physics world
                self.update_physics_world(delta_time);

                // Sync character state with physics body
                if let Some(physics_position) = self.get_character_position() {
                    state.position = physics_position;
                }

                if let Some(physics_velocity) = self.get_character_velocity() {
                    state.velocity = physics_velocity;
                }

                // Update grounded state
                state.is_grounded = self.is_character_grounded();

                // If character has no physics body yet, create one
                if self.character_body_handle.is_none() {
                    if let Err(e) = self.create_character_body(state.position) {
                        eprintln!("Failed to create character physics body: {}", e);
                    }
                }
            }
        }
    }

    fn apply_gravity(&self, state: &mut CharacterState, delta_time: f32) {
        if !state.is_grounded {
            state.velocity.y += self.gravity * delta_time;
            
            // Terminal velocity limit
            if state.velocity.y < self.terminal_velocity {
                state.velocity.y = self.terminal_velocity;
            }
        }
    }

    fn check_ground_collision(&mut self, state: &mut CharacterState) {
        // Simple ground detection - in a full implementation this would
        // raycast against the world geometry
        let ground_check_pos = Point3::new(
            state.position.x,
            state.position.y - self.character_height / 2.0 - self.ground_check_distance,
            state.position.z
        );

        // For now, assume ground at y = 0
        let was_grounded = state.is_grounded;
        state.is_grounded = ground_check_pos.y <= 0.0;

        if state.is_grounded && !was_grounded {
            // Just landed
            state.position.y = self.character_height / 2.0;
            if state.velocity.y < 0.0 {
                state.velocity.y = 0.0;
            }
        }

        // Update ground normal (simplified)
        self.ground_normal = Vector3::new(0.0, 1.0, 0.0);
    }

    fn resolve_collisions(&mut self, state: &mut CharacterState, delta_time: f32) {
        // Predict new position
        let new_position = state.position + state.velocity * delta_time;
        
        // Create character collision bounds
        let character_bounds = self.get_character_bounds(&new_position);
        
        // Check against world collisions
        let mut collision_occurred = false;
        let mut collision_normal = Vector3::new(0.0, 0.0, 0.0);
        
        // In a full implementation, this would check against world geometry
        // For now, we'll implement basic boundary checking
        
        // Simple world boundaries
        let world_bounds = CollisionBox {
            min: Point3::new(-1000.0, -100.0, -1000.0),
            max: Point3::new(1000.0, 1000.0, 1000.0),
            material: "world_bounds".to_string(),
            solid: true,
        };

        if !self.boxes_intersect(&character_bounds, &world_bounds) {
            // Outside world bounds - push back
            collision_occurred = true;
            
            if new_position.x < world_bounds.min.x {
                collision_normal.x = 1.0;
                state.position.x = world_bounds.min.x + self.character_radius;
            } else if new_position.x > world_bounds.max.x {
                collision_normal.x = -1.0;
                state.position.x = world_bounds.max.x - self.character_radius;
            }
            
            if new_position.z < world_bounds.min.z {
                collision_normal.z = 1.0;
                state.position.z = world_bounds.min.z + self.character_radius;
            } else if new_position.z > world_bounds.max.z {
                collision_normal.z = -1.0;
                state.position.z = world_bounds.max.z - self.character_radius;
            }
        } else {
            // No collision, update position
            state.position = new_position;
        }

        // Resolve velocity based on collision
        if collision_occurred {
            // Remove velocity component in collision normal direction
            let velocity_dot_normal = state.velocity.dot(&collision_normal);
            if velocity_dot_normal < 0.0 {
                state.velocity -= collision_normal * velocity_dot_normal;
            }
        }
    }

    fn get_character_bounds(&self, position: &Point3<f32>) -> CollisionBox {
        CollisionBox {
            min: Point3::new(
                position.x - self.character_radius,
                position.y - self.character_height / 2.0,
                position.z - self.character_radius,
            ),
            max: Point3::new(
                position.x + self.character_radius,
                position.y + self.character_height / 2.0,
                position.z + self.character_radius,
            ),
            material: "character".to_string(),
            solid: true,
        }
    }

    fn boxes_intersect(&self, a: &CollisionBox, b: &CollisionBox) -> bool {
        a.min.x < b.max.x && a.max.x > b.min.x &&
        a.min.y < b.max.y && a.max.y > b.min.y &&
        a.min.z < b.max.z && a.max.z > b.min.z
    }

    fn apply_friction(&self, state: &mut CharacterState, delta_time: f32) {
        let friction = if state.is_grounded { 8.0 } else { 1.0 };
        
        // Apply horizontal friction
        let horizontal_velocity = Vector3::new(state.velocity.x, 0.0, state.velocity.z);
        let friction_force = horizontal_velocity * -friction * delta_time;
        
        state.velocity.x += friction_force.x;
        state.velocity.z += friction_force.z;
        
        // Prevent micro-oscillations
        if horizontal_velocity.magnitude() < 0.1 && state.is_grounded {
            state.velocity.x = 0.0;
            state.velocity.z = 0.0;
        }
    }

    // Collision management
    pub fn add_collision_box(&mut self, layer: &str, collision_box: CollisionBox) {
        self.collision_layers.entry(layer.to_string())
            .or_insert_with(Vec::new)
            .push(collision_box);
    }

    pub fn remove_collision_layer(&mut self, layer: &str) {
        self.collision_layers.remove(layer);
    }

    pub fn clear_collisions(&mut self) {
        self.collision_layers.clear();
    }

    // Physics queries
    pub fn raycast(&self, origin: Point3<f32>, direction: Vector3<f32>, max_distance: f32) -> Option<RaycastHit> {
        // Simplified raycast implementation
        // In a full implementation, this would check against all collision geometry
        
        let end_point = origin + direction.normalize() * max_distance;
        
        // Check ground intersection
        if direction.y < 0.0 && origin.y > 0.0 && end_point.y <= 0.0 {
            let t = -origin.y / direction.y;
            let hit_point = origin + direction * t;
            
            return Some(RaycastHit {
                point: hit_point,
                normal: Vector3::new(0.0, 1.0, 0.0),
                distance: t,
                material: "ground".to_string(),
            });
        }
        
        None
    }

    pub fn sphere_cast(&self, center: Point3<f32>, radius: f32, direction: Vector3<f32>, max_distance: f32) -> Option<RaycastHit> {
        // Simplified sphere cast - similar to raycast but with radius
        self.raycast(center, direction, max_distance)
    }

    // Getters and setters
    pub fn set_gravity(&mut self, gravity: f32) {
        self.gravity = gravity;
    }

    pub fn get_gravity(&self) -> f32 {
        self.gravity
    }

    pub fn set_character_size(&mut self, radius: f32, height: f32) {
        self.character_radius = radius;
        self.character_height = height;
    }

    pub fn get_ground_normal(&self) -> Vector3<f32> {
        self.ground_normal
    }
}

#[derive(Clone, Debug)]
pub struct RaycastHit {
    pub point: Point3<f32>,
    pub normal: Vector3<f32>,
    pub distance: f32,
    pub material: String,
}

// Physics utilities
impl CharacterPhysics {
    pub fn calculate_jump_velocity_for_height(&self, height: f32) -> f32 {
        // v = sqrt(2 * g * h)
        (2.0 * -self.gravity * height).sqrt()
    }

    pub fn calculate_air_time_for_jump(&self, initial_velocity: f32) -> f32 {
        // t = 2 * v / g
        2.0 * initial_velocity / -self.gravity
    }

    pub fn is_on_slope(&self, normal: Vector3<f32>) -> bool {
        let angle = normal.angle(&Vector3::new(0.0, 1.0, 0.0));
        angle > self.slope_limit
    }

    pub fn get_slope_slide_direction(&self, normal: Vector3<f32>) -> Vector3<f32> {
        if !self.is_on_slope(normal) {
            return Vector3::zeros();
        }

        // Project gravity onto the slope
        let gravity_dir = Vector3::new(0.0, -1.0, 0.0);
        gravity_dir - normal * gravity_dir.dot(&normal)
    }

    // Rapier3D Integration Methods
    pub fn create_character_body(&mut self, position: Point3<f32>) -> Result<(), String> {
        // Create rigid body for character
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(vector![position.x, position.y, position.z])
            .lock_rotations() // Prevent character from rotating
            .linear_damping(0.8) // Add some damping for realistic movement
            .angular_damping(10.0)
            .build();

        let body_handle = self.rigid_body_set.insert(rigid_body);

        // Create capsule collider for character
        let collider = ColliderBuilder::capsule_y(self.character_height / 2.0, self.character_radius)
            .friction(0.8)
            .restitution(0.0) // No bouncing
            .collision_groups(InteractionGroups::new(
                Group::GROUP_1, // Character group
                Group::all()     // Collides with everything
            ))
            .build();

        let collider_handle = self.collider_set.insert_with_parent(
            collider,
            body_handle,
            &mut self.rigid_body_set
        );

        self.character_body_handle = Some(body_handle);
        self.character_collider_handle = Some(collider_handle);

        Ok(())
    }

    pub fn update_physics_world(&mut self, delta_time: f32) {
        let gravity = vector![0.0, self.gravity, 0.0];
        let integration_parameters = IntegrationParameters {
            dt: delta_time,
            ..Default::default()
        };

        self.physics_pipeline.step(
            &gravity,
            &integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &(),
            &()
        );
    }

    pub fn get_character_position(&self) -> Option<Point3<f32>> {
        if let Some(handle) = self.character_body_handle {
            if let Some(body) = self.rigid_body_set.get(handle) {
                let translation = body.translation();
                return Some(Point3::new(translation.x, translation.y, translation.z));
            }
        }
        None
    }

    pub fn set_character_velocity(&mut self, velocity: Vector3<f32>) {
        if let Some(handle) = self.character_body_handle {
            if let Some(body) = self.rigid_body_set.get_mut(handle) {
                body.set_linvel(vector![velocity.x, velocity.y, velocity.z], true);
            }
        }
    }

    pub fn get_character_velocity(&self) -> Option<Vector3<f32>> {
        if let Some(handle) = self.character_body_handle {
            if let Some(body) = self.rigid_body_set.get(handle) {
                let linvel = body.linvel();
                return Some(Vector3::new(linvel.x, linvel.y, linvel.z));
            }
        }
        None
    }

    pub fn apply_character_force(&mut self, force: Vector3<f32>) {
        if let Some(handle) = self.character_body_handle {
            if let Some(body) = self.rigid_body_set.get_mut(handle) {
                body.add_force(vector![force.x, force.y, force.z], true);
            }
        }
    }

    pub fn character_jump(&mut self, jump_velocity: f32) {
        if let Some(handle) = self.character_body_handle {
            if let Some(body) = self.rigid_body_set.get_mut(handle) {
                let mut velocity = body.linvel().clone();
                velocity.y = jump_velocity;
                body.set_linvel(velocity, true);
            }
        }
    }

    pub fn add_voxel_collider(&mut self, position: Point3<f32>, size: f32) -> ColliderHandle {
        let collider = ColliderBuilder::cuboid(size / 2.0, size / 2.0, size / 2.0)
            .translation(vector![position.x, position.y, position.z])
            .collision_groups(InteractionGroups::new(
                Group::GROUP_2, // Voxel group
                Group::all()     // Collides with everything
            ))
            .build();

        self.collider_set.insert(collider)
    }

    pub fn remove_voxel_collider(&mut self, handle: ColliderHandle) {
        self.collider_set.remove(
            handle,
            &mut self.island_manager,
            &mut self.rigid_body_set,
            true
        );
    }

    pub fn character_raycast(&self, origin: Point3<f32>, direction: Vector3<f32>, max_distance: f32) -> Option<(Point3<f32>, Vector3<f32>)> {
        let ray = Ray::new(
            point![origin.x, origin.y, origin.z],
            vector![direction.x, direction.y, direction.z]
        );

        let filter = QueryFilter::new().groups(InteractionGroups::new(
            Group::all(),
            Group::GROUP_2 // Only hit voxels
        ));

        if let Some((handle, toi)) = self.query_pipeline.cast_ray(
            &self.rigid_body_set,
            &self.collider_set,
            &ray,
            max_distance,
            true,
            filter
        ) {
            let hit_point = ray.point_at(toi);
            // For raycasts, we can use the ray direction to approximate the normal
            // or use cast_ray_and_get_normal if available
            let normal = if let Some(intersection) = self.query_pipeline.cast_ray_and_get_normal(
                &self.rigid_body_set,
                &self.collider_set,
                &ray,
                max_distance,
                true,
                filter
            ) {
                intersection.1.normal
            } else {
                // Fallback to upward normal
                vector![0.0, 1.0, 0.0]
            };

            return Some((
                Point3::new(hit_point.x, hit_point.y, hit_point.z),
                Vector3::new(normal.x, normal.y, normal.z)
            ));
        }

        None
    }

    pub fn is_character_grounded(&self) -> bool {
        if let Some(position) = self.get_character_position() {
            let ray_origin = Point3::new(
                position.x,
                position.y - self.character_height / 2.0,
                position.z
            );
            let ray_direction = Vector3::new(0.0, -1.0, 0.0);

            if let Some(_) = self.character_raycast(ray_origin, ray_direction, self.ground_check_distance + 0.1) {
                return true;
            }
        }
        false
    }

    // High-level movement methods for game integration
    pub fn apply_movement_input(&mut self, forward: f32, right: f32, jump: bool, movement_speed: f32) {
        if let Some(handle) = self.character_body_handle {
            // Calculate jump velocity before mutable borrow
            let is_grounded = self.is_character_grounded();
            let jump_velocity = if jump && is_grounded {
                self.calculate_jump_velocity_for_height(2.0) // 2 meter jump height
            } else {
                0.0
            };

            if let Some(body) = self.rigid_body_set.get_mut(handle) {
                // Calculate movement vector
                let mut movement = Vector3::new(right, 0.0, forward) * movement_speed;

                // Preserve existing vertical velocity for gravity/jumping
                let current_velocity = body.linvel();
                movement.y = current_velocity.y;

                // Handle jumping
                if jump && is_grounded {
                    movement.y = jump_velocity;
                }

                // Apply the movement
                body.set_linvel(vector![movement.x, movement.y, movement.z], true);
            }
        }
    }

    pub fn teleport_character(&mut self, position: Point3<f32>) {
        if let Some(handle) = self.character_body_handle {
            if let Some(body) = self.rigid_body_set.get_mut(handle) {
                body.set_translation(vector![position.x, position.y, position.z], true);
                body.set_linvel(vector![0.0, 0.0, 0.0], true);
            }
        }
    }

    /// Initialize the physics world with default settings
    pub fn initialize_physics_world(&mut self) -> Result<(), String> {
        // Physics world is already initialized in new(), but we can add validation here
        Ok(())
    }

    /// Enhanced movement update with voxel world collision
    pub fn update_movement(
        &mut self,
        character_state: &mut CharacterState,
        movement_input: Vector3<f32>,
        jump_requested: bool,
        world: &crate::engine::generation::voxel_system::VoxelWorld,
        delta_time: f32,
    ) {
        // Apply movement input to physics body
        if let Some(body_handle) = self.character_body_handle {
            if let Some(body) = self.rigid_body_set.get_mut(body_handle) {
                // Get current velocity
                let current_velocity = body.linvel();

                // Apply horizontal movement (preserve vertical velocity for gravity/jumping)
                let new_velocity = vector![
                    movement_input.x,
                    current_velocity.y, // Preserve vertical velocity
                    movement_input.z
                ];

                body.set_linvel(new_velocity, true);

                // Handle jumping
                if jump_requested && self.is_character_grounded() {
                    let jump_force = self.calculate_jump_velocity_for_height(2.0); // 2 meter jump height
                    body.apply_impulse(vector![0.0, jump_force, 0.0], true);
                }
            }
        }

        // Update physics world
        self.update_physics_world(delta_time);

        // Check for voxel collisions and adjust position
        self.handle_voxel_world_collision(character_state, world);
    }

    /// Update character state from physics simulation
    pub fn update_character_state(&mut self, character_state: &mut CharacterState, _delta_time: f32) {
        if let Some(position) = self.get_character_position() {
            character_state.position = position;
        }

        if let Some(velocity) = self.get_character_velocity() {
            character_state.velocity = velocity;
        }

        character_state.is_grounded = self.is_character_grounded();
    }

    /// Get eye height for camera positioning
    pub fn get_eye_height(&self) -> f32 {
        self.character_height * 0.85 // Eye height is 85% of character height
    }

    /// Handle collision with voxel world
    fn handle_voxel_world_collision(
        &mut self,
        _character_state: &mut CharacterState,
        world: &crate::engine::generation::voxel_system::VoxelWorld,
    ) {
        use cgmath::Vector3 as CgVector3;

        if let Some(character_pos) = self.get_character_position() {
            // Check collision in character's vicinity
            let check_radius = self.character_radius + 0.1;
            let check_positions = [
                // Check around character feet
                CgVector3::new(character_pos.x - check_radius, character_pos.y - self.character_height/2.0, character_pos.z),
                CgVector3::new(character_pos.x + check_radius, character_pos.y - self.character_height/2.0, character_pos.z),
                CgVector3::new(character_pos.x, character_pos.y - self.character_height/2.0, character_pos.z - check_radius),
                CgVector3::new(character_pos.x, character_pos.y - self.character_height/2.0, character_pos.z + check_radius),

                // Check around character head
                CgVector3::new(character_pos.x, character_pos.y + self.character_height/2.0, character_pos.z),
            ];

            for check_pos in &check_positions {
                if let Some(voxel_type) = world.get_voxel(*check_pos) {
                    use crate::engine::generation::voxel_system::VoxelType;

                    if voxel_type != VoxelType::Air {
                        // Found collision with solid voxel
                        // For now, we can add the voxel as a static collider in the physics world
                        let voxel_pos = Point3::new(
                            check_pos.x.floor(),
                            check_pos.y.floor(),
                            check_pos.z.floor(),
                        );
                        self.ensure_voxel_collider(voxel_pos);
                    }
                }
            }
        }
    }

    /// Ensure a voxel has a physics collider at the specified position
    fn ensure_voxel_collider(&mut self, position: Point3<f32>) {
        // Check if collider already exists at this position
        // For simplicity, we'll create a temporary collider for collision detection
        let collider = ColliderBuilder::cuboid(0.5, 0.5, 0.5)
            .translation(vector![position.x + 0.5, position.y + 0.5, position.z + 0.5])
            .build();

        // Add to collider set (in a full implementation, you'd want to cache these)
        self.collider_set.insert(collider);
    }
}