/*!
 * 3D Physics System for Robin Engine
 *
 * This module provides 3D physics simulation using rapier3d for the voxel world.
 * It handles player movement, gravity, collisions, and block physics.
 */

use crate::engine::{
    error::{RobinError, RobinResult},
    math::Vec3,
};
use rapier3d::prelude::*;
use std::collections::HashMap;
use cgmath::{Vector3, Point3};
use nalgebra::UnitQuaternion;

/// Configuration for the 3D physics world
#[derive(Debug, Clone)]
pub struct Physics3DConfig {
    /// Gravity vector (typically [0.0, -9.81, 0.0] for Earth-like gravity)
    pub gravity: Vector3<f32>,
    /// Fixed timestep for physics simulation (e.g., 1.0/60.0 for 60 FPS)
    pub timestep: f32,
    /// Maximum velocity for objects (prevents tunneling)
    pub max_velocity: f32,
    /// Number of solver iterations per physics step
    pub solver_iterations: usize,
}

impl Default for Physics3DConfig {
    fn default() -> Self {
        Self {
            gravity: Vector3::new(0.0, -9.81, 0.0),
            timestep: 1.0 / 60.0,
            max_velocity: 50.0,
            solver_iterations: 8,
        }
    }
}

/// Handle to a physics body in the 3D world
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PhysicsHandle(pub RigidBodyHandle);

/// Type of physics body
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BodyType3D {
    /// Static body - never moves, infinite mass
    Static,
    /// Dynamic body - affected by forces and gravity
    Dynamic,
    /// Kinematic body - moves but not affected by forces
    Kinematic,
}

/// Shape type for colliders
#[derive(Debug, Clone)]
pub enum ColliderShape3D {
    /// Box collider with half-extents
    Box { half_extents: Vector3<f32> },
    /// Sphere collider with radius
    Sphere { radius: f32 },
    /// Capsule collider (cylinder with rounded ends)
    Capsule { half_height: f32, radius: f32 },
    /// Custom mesh collider (for complex shapes)
    Mesh { vertices: Vec<Point3<f32>>, indices: Vec<[u32; 3]> },
}

/// Properties for creating a physics body
#[derive(Debug, Clone)]
pub struct BodyDescriptor {
    pub body_type: BodyType3D,
    pub position: Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
    pub velocity: Vector3<f32>,
    pub angular_velocity: Vector3<f32>,
    pub mass: f32,
    pub restitution: f32,  // Bounciness (0.0 = no bounce, 1.0 = perfect bounce)
    pub friction: f32,     // Surface friction (0.0 = no friction, 1.0 = high friction)
    pub linear_damping: f32,  // Air resistance for linear motion
    pub angular_damping: f32, // Air resistance for rotation
    pub gravity_scale: f32,   // Multiplier for gravity effect
    pub is_sensor: bool,      // True if this should trigger events but not collisions
}

impl Default for BodyDescriptor {
    fn default() -> Self {
        Self {
            body_type: BodyType3D::Dynamic,
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: cgmath::Quaternion::new(1.0, 0.0, 0.0, 0.0),
            velocity: Vector3::new(0.0, 0.0, 0.0),
            angular_velocity: Vector3::new(0.0, 0.0, 0.0),
            mass: 1.0,
            restitution: 0.2,
            friction: 0.5,
            linear_damping: 0.01,
            angular_damping: 0.05,
            gravity_scale: 1.0,
            is_sensor: false,
        }
    }
}

/// Collision event data
#[derive(Debug, Clone)]
pub struct CollisionEvent3D {
    pub handle1: PhysicsHandle,
    pub handle2: PhysicsHandle,
    pub contact_point: Vector3<f32>,
    pub contact_normal: Vector3<f32>,
    pub impulse: f32,
    pub started: bool,  // True if collision started, false if ended
}

/// The main 3D physics world
pub struct PhysicsWorld3D {
    /// Rapier physics pipeline
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    query_pipeline: QueryPipeline,

    /// Physics hooks
    physics_hooks: (),
    event_handler: ChannelEventCollector,

    /// Configuration
    config: Physics3DConfig,

    /// Time accumulator for fixed timestep
    time_accumulator: f32,

    /// Mapping from handles to user data
    body_user_data: HashMap<RigidBodyHandle, String>,

    /// Collision events from last step
    collision_events: Vec<CollisionEvent3D>,
}

impl PhysicsWorld3D {
    /// Create a new 3D physics world
    pub fn new(config: Physics3DConfig) -> Self {
        let mut integration_parameters = IntegrationParameters::default();
        integration_parameters.dt = config.timestep;
        // Note: Newer rapier3d versions use different parameter names
        // integration_parameters.max_velocity_iterations = config.solver_iterations;
        // integration_parameters.max_velocity_friction_iterations = config.solver_iterations;
        // integration_parameters.max_stabilization_iterations = config.solver_iterations;

        let rigid_body_set = RigidBodySet::new();
        let collider_set = ColliderSet::new();
        let physics_pipeline = PhysicsPipeline::new();
        let island_manager = IslandManager::new();
        let broad_phase = BroadPhase::new();
        let narrow_phase = NarrowPhase::new();
        let impulse_joint_set = ImpulseJointSet::new();
        let multibody_joint_set = MultibodyJointSet::new();
        let ccd_solver = CCDSolver::new();
        let query_pipeline = QueryPipeline::new();
        let event_handler = ChannelEventCollector::new(1000, 1000);

        Self {
            rigid_body_set,
            collider_set,
            integration_parameters,
            physics_pipeline,
            island_manager,
            broad_phase,
            narrow_phase,
            impulse_joint_set,
            multibody_joint_set,
            ccd_solver,
            query_pipeline,
            physics_hooks: (),
            event_handler,
            config,
            time_accumulator: 0.0,
            body_user_data: HashMap::new(),
            collision_events: Vec::new(),
        }
    }

    /// Create a new physics body with the given descriptor
    pub fn create_body(&mut self, descriptor: BodyDescriptor, shape: ColliderShape3D, user_data: Option<String>) -> RobinResult<PhysicsHandle> {
        // Create rigid body
        let mut rigid_body_builder = match descriptor.body_type {
            BodyType3D::Static => RigidBodyBuilder::fixed(),
            BodyType3D::Dynamic => RigidBodyBuilder::dynamic(),
            BodyType3D::Kinematic => RigidBodyBuilder::kinematic_velocity_based(),
        };

        // Set position and rotation
        let position = Isometry::from_parts(
            Translation::new(descriptor.position.x, descriptor.position.y, descriptor.position.z),
            UnitQuaternion::new_normalize(nalgebra::Quaternion::new(
                descriptor.rotation.s,
                descriptor.rotation.v.x,
                descriptor.rotation.v.y,
                descriptor.rotation.v.z,
            )),
        );
        rigid_body_builder = rigid_body_builder.translation(nalgebra::Vector3::new(descriptor.position.x, descriptor.position.y, descriptor.position.z));

        // Set physics properties
        if descriptor.body_type == BodyType3D::Dynamic {
            rigid_body_builder = rigid_body_builder
                .linvel(nalgebra::Vector3::new(descriptor.velocity.x, descriptor.velocity.y, descriptor.velocity.z))
                .angvel(nalgebra::Vector3::new(descriptor.angular_velocity.x, descriptor.angular_velocity.y, descriptor.angular_velocity.z))
                .linear_damping(descriptor.linear_damping)
                .angular_damping(descriptor.angular_damping)
                .gravity_scale(descriptor.gravity_scale);
        }

        let rigid_body = rigid_body_builder.build();
        let body_handle = self.rigid_body_set.insert(rigid_body);

        // Create collider
        let collider_builder = match shape {
            ColliderShape3D::Box { half_extents } => {
                ColliderBuilder::cuboid(half_extents.x, half_extents.y, half_extents.z)
            },
            ColliderShape3D::Sphere { radius } => {
                ColliderBuilder::ball(radius)
            },
            ColliderShape3D::Capsule { half_height, radius } => {
                ColliderBuilder::capsule_y(half_height, radius)
            },
            ColliderShape3D::Mesh { vertices, indices } => {
                let vertices: Vec<nalgebra::Point3<f32>> = vertices.into_iter()
                    .map(|v| nalgebra::Point3::new(v.x, v.y, v.z))
                    .collect();
                ColliderBuilder::trimesh(vertices, indices)
            },
        };

        let mut collider = collider_builder
            .restitution(descriptor.restitution)
            .friction(descriptor.friction)
            .sensor(descriptor.is_sensor)
            .build();

        // Set mass for dynamic bodies
        if descriptor.body_type == BodyType3D::Dynamic {
            collider.set_mass(descriptor.mass);
        }

        self.collider_set.insert_with_parent(collider, body_handle, &mut self.rigid_body_set);

        // Store user data
        if let Some(data) = user_data {
            self.body_user_data.insert(body_handle, data);
        }

        Ok(PhysicsHandle(body_handle))
    }

    /// Get the position of a physics body
    pub fn get_body_position(&self, handle: PhysicsHandle) -> Option<Vector3<f32>> {
        let body = self.rigid_body_set.get(handle.0)?;
        let translation = body.translation();
        Some(Vector3::new(translation.x, translation.y, translation.z))
    }

    /// Get the rotation of a physics body as a quaternion
    pub fn get_body_rotation(&self, handle: PhysicsHandle) -> Option<cgmath::Quaternion<f32>> {
        let body = self.rigid_body_set.get(handle.0)?;
        let rotation = body.rotation();
        Some(cgmath::Quaternion::new(rotation.w, rotation.i, rotation.j, rotation.k))
    }

    /// Set the position of a physics body
    pub fn set_body_position(&mut self, handle: PhysicsHandle, position: Vector3<f32>) -> RobinResult<()> {
        let body = self.rigid_body_set.get_mut(handle.0)
            .ok_or_else(|| RobinError::PhysicsError("Invalid physics handle".to_string()))?;

        body.set_translation(nalgebra::Vector3::new(position.x, position.y, position.z), true);
        Ok(())
    }

    /// Set the velocity of a physics body
    pub fn set_body_velocity(&mut self, handle: PhysicsHandle, velocity: Vector3<f32>) -> RobinResult<()> {
        let body = self.rigid_body_set.get_mut(handle.0)
            .ok_or_else(|| RobinError::PhysicsError("Invalid physics handle".to_string()))?;

        body.set_linvel(nalgebra::Vector3::new(velocity.x, velocity.y, velocity.z), true);
        Ok(())
    }

    /// Apply a force to a physics body at its center of mass
    pub fn apply_force(&mut self, handle: PhysicsHandle, force: Vector3<f32>) -> RobinResult<()> {
        let body = self.rigid_body_set.get_mut(handle.0)
            .ok_or_else(|| RobinError::PhysicsError("Invalid physics handle".to_string()))?;

        body.add_force(nalgebra::Vector3::new(force.x, force.y, force.z), true);
        Ok(())
    }

    /// Apply an impulse to a physics body at its center of mass
    pub fn apply_impulse(&mut self, handle: PhysicsHandle, impulse: Vector3<f32>) -> RobinResult<()> {
        let body = self.rigid_body_set.get_mut(handle.0)
            .ok_or_else(|| RobinError::PhysicsError("Invalid physics handle".to_string()))?;

        body.apply_impulse(nalgebra::Vector3::new(impulse.x, impulse.y, impulse.z), true);
        Ok(())
    }

    /// Remove a physics body from the world
    pub fn remove_body(&mut self, handle: PhysicsHandle) -> RobinResult<()> {
        self.rigid_body_set.remove(
            handle.0,
            &mut self.island_manager,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            true,
        );
        self.body_user_data.remove(&handle.0);
        Ok(())
    }

    /// Perform a raycast in the physics world
    pub fn raycast(&self, origin: Vector3<f32>, direction: Vector3<f32>, max_toi: f32) -> Option<(PhysicsHandle, f32, Vector3<f32>)> {
        let ray = Ray::new(
            nalgebra::Point3::new(origin.x, origin.y, origin.z),
            nalgebra::Vector3::new(direction.x, direction.y, direction.z),
        );

        if let Some((handle, toi)) = self.query_pipeline.cast_ray(
            &self.rigid_body_set,
            &self.collider_set,
            &ray,
            max_toi,
            true,
            QueryFilter::default(),
        ) {
            let hit_point = ray.point_at(toi);
            Some((
                PhysicsHandle(handle),
                toi,
                Vector3::new(hit_point.x, hit_point.y, hit_point.z),
            ))
        } else {
            None
        }
    }

    /// Update the physics world by the given delta time
    pub fn step(&mut self, delta_time: f32) -> RobinResult<()> {
        self.time_accumulator += delta_time;

        // Process collision events from previous step
        self.collision_events.clear();
        while let Ok(collision_event) = self.event_handler.collision_events.try_recv() {
            if let (Some(collider1), Some(collider2)) = (
                self.collider_set.get(collision_event.collider1()),
                self.collider_set.get(collision_event.collider2()),
            ) {
                if let (Some(body1), Some(body2)) = (
                    collider1.parent().and_then(|h| self.rigid_body_set.get(h)),
                    collider2.parent().and_then(|h| self.rigid_body_set.get(h)),
                ) {
                    let event = CollisionEvent3D {
                        handle1: PhysicsHandle(collider1.parent().unwrap()),
                        handle2: PhysicsHandle(collider2.parent().unwrap()),
                        contact_point: Vector3::new(0.0, 0.0, 0.0), // TODO: Get actual contact point
                        contact_normal: Vector3::new(0.0, 1.0, 0.0), // TODO: Get actual normal
                        impulse: 0.0, // TODO: Get actual impulse
                        started: collision_event.started(),
                    };
                    self.collision_events.push(event);
                }
            }
        }

        // Run physics steps with fixed timestep
        while self.time_accumulator >= self.config.timestep {
            self.integration_parameters.dt = self.config.timestep;

            // Set gravity
            let gravity = nalgebra::Vector3::new(
                self.config.gravity.x,
                self.config.gravity.y,
                self.config.gravity.z,
            );

            // Step the physics simulation
            self.physics_pipeline.step(
                &gravity,
                &self.integration_parameters,
                &mut self.island_manager,
                &mut self.broad_phase,
                &mut self.narrow_phase,
                &mut self.rigid_body_set,
                &mut self.collider_set,
                &mut self.impulse_joint_set,
                &mut self.multibody_joint_set,
                &mut self.ccd_solver,
                Some(&mut self.query_pipeline),
                &self.physics_hooks,
                &self.event_handler,
            );

            self.time_accumulator -= self.config.timestep;
        }

        Ok(())
    }

    /// Get collision events from the last physics step
    pub fn collision_events(&self) -> &[CollisionEvent3D] {
        &self.collision_events
    }

    /// Get user data for a physics body
    pub fn get_user_data(&self, handle: PhysicsHandle) -> Option<&str> {
        self.body_user_data.get(&handle.0).map(|s| s.as_str())
    }

    /// Set the gravity of the physics world
    pub fn set_gravity(&mut self, gravity: Vector3<f32>) {
        self.config.gravity = gravity;
    }

    /// Get the current gravity
    pub fn gravity(&self) -> Vector3<f32> {
        self.config.gravity
    }
}

// Convenience functions for common shapes
impl ColliderShape3D {
    /// Create a box collider for a voxel block (1x1x1 unit cube)
    pub fn voxel_block() -> Self {
        Self::Box {
            half_extents: Vector3::new(0.5, 0.5, 0.5),
        }
    }

    /// Create a box collider for a player character
    pub fn player_character() -> Self {
        Self::Capsule {
            half_height: 0.9, // 1.8 units tall
            radius: 0.3,      // 0.6 units wide
        }
    }

    /// Create a sphere collider for a ball or projectile
    pub fn ball(radius: f32) -> Self {
        Self::Sphere { radius }
    }
}

// Convenience functions for common body types
impl BodyDescriptor {
    /// Create a descriptor for a static voxel block
    pub fn static_block(position: Vector3<f32>) -> Self {
        Self {
            body_type: BodyType3D::Static,
            position,
            mass: 0.0, // Static bodies have infinite mass
            friction: 0.6,
            restitution: 0.1,
            ..Default::default()
        }
    }

    /// Create a descriptor for a player character
    pub fn player_character(position: Vector3<f32>) -> Self {
        Self {
            body_type: BodyType3D::Dynamic,
            position,
            mass: 70.0, // 70 kg human
            friction: 0.8,
            restitution: 0.0, // No bouncing for player
            linear_damping: 0.1, // Air resistance
            angular_damping: 0.9, // High rotational damping to prevent spinning
            gravity_scale: 1.0,
            ..Default::default()
        }
    }

    /// Create a descriptor for a falling block (like sand or gravel)
    pub fn falling_block(position: Vector3<f32>) -> Self {
        Self {
            body_type: BodyType3D::Dynamic,
            position,
            mass: 10.0,
            friction: 0.5,
            restitution: 0.2,
            linear_damping: 0.02,
            angular_damping: 0.05,
            gravity_scale: 1.0,
            ..Default::default()
        }
    }
}