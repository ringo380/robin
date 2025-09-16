// Physics System for Robin Engine
// Provides realistic physics simulation for the Engineer Build Mode

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PhysicsEngine {
    pub world: PhysicsWorld,
    pub collision_detector: CollisionDetector,
    pub constraint_solver: ConstraintSolver,
    pub integration_method: IntegrationMethod,
    pub gravity: [f32; 3],
    pub time_step: f32,
}

impl PhysicsEngine {
    pub fn new() -> Self {
        Self {
            world: PhysicsWorld::new(),
            collision_detector: CollisionDetector::new(),
            constraint_solver: ConstraintSolver::new(),
            integration_method: IntegrationMethod::Verlet,
            gravity: [0.0, -9.81, 0.0], // Earth gravity
            time_step: 1.0 / 60.0, // 60 FPS
        }
    }
    
    pub fn initialize(&mut self) -> Result<(), String> {
        self.world.initialize()?;
        self.collision_detector.initialize()?;
        self.constraint_solver.initialize()?;
        Ok(())
    }
    
    pub fn step_simulation(&mut self, delta_time: f32) -> Result<(), String> {
        let steps = (delta_time / self.time_step).ceil() as u32;
        let actual_step_size = delta_time / steps as f32;
        
        for _ in 0..steps {
            self.step_physics(actual_step_size)?;
        }
        
        Ok(())
    }
    
    fn step_physics(&mut self, dt: f32) -> Result<(), String> {
        // 1. Apply forces (gravity, user forces, etc.)
        self.apply_forces(dt)?;
        
        // 2. Integrate motion
        self.integrate_motion(dt)?;
        
        // 3. Detect collisions
        let collisions = self.collision_detector.detect_collisions(&self.world)?;
        
        // 4. Resolve collisions
        self.resolve_collisions(&collisions)?;
        
        // 5. Solve constraints
        self.constraint_solver.solve_constraints(&mut self.world, dt)?;
        
        Ok(())
    }
    
    pub fn add_rigid_body(&mut self, body: RigidBody) -> BodyId {
        self.world.add_body(body)
    }
    
    pub fn remove_rigid_body(&mut self, body_id: BodyId) {
        self.world.remove_body(body_id);
    }
    
    pub fn set_body_position(&mut self, body_id: BodyId, position: [f32; 3]) {
        if let Some(body) = self.world.get_body_mut(body_id) {
            body.position = position;
        }
    }
    
    pub fn set_body_velocity(&mut self, body_id: BodyId, velocity: [f32; 3]) {
        if let Some(body) = self.world.get_body_mut(body_id) {
            body.velocity = velocity;
        }
    }
    
    pub fn apply_force(&mut self, body_id: BodyId, force: [f32; 3]) {
        if let Some(body) = self.world.get_body_mut(body_id) {
            body.accumulated_force[0] += force[0];
            body.accumulated_force[1] += force[1];
            body.accumulated_force[2] += force[2];
        }
    }
    
    pub fn apply_impulse(&mut self, body_id: BodyId, impulse: [f32; 3]) {
        if let Some(body) = self.world.get_body_mut(body_id) {
            let inv_mass = 1.0 / body.mass;
            body.velocity[0] += impulse[0] * inv_mass;
            body.velocity[1] += impulse[1] * inv_mass;
            body.velocity[2] += impulse[2] * inv_mass;
        }
    }
    
    pub fn raycast(&self, origin: [f32; 3], direction: [f32; 3], max_distance: f32) -> Option<RaycastHit> {
        self.collision_detector.raycast(&self.world, origin, direction, max_distance)
    }
    
    fn apply_forces(&mut self, dt: f32) -> Result<(), String> {
        for (_, body) in &mut self.world.bodies {
            if body.body_type == RigidBodyType::Dynamic {
                // Apply gravity
                let gravity_force = [
                    self.gravity[0] * body.mass,
                    self.gravity[1] * body.mass,
                    self.gravity[2] * body.mass,
                ];
                
                body.accumulated_force[0] += gravity_force[0];
                body.accumulated_force[1] += gravity_force[1];
                body.accumulated_force[2] += gravity_force[2];
                
                // Apply drag
                let drag_coefficient = 0.1;
                let drag_force = [
                    -body.velocity[0] * drag_coefficient,
                    -body.velocity[1] * drag_coefficient,
                    -body.velocity[2] * drag_coefficient,
                ];
                
                body.accumulated_force[0] += drag_force[0];
                body.accumulated_force[1] += drag_force[1];
                body.accumulated_force[2] += drag_force[2];
            }
        }
        
        Ok(())
    }
    
    fn integrate_motion(&mut self, dt: f32) -> Result<(), String> {
        match self.integration_method {
            IntegrationMethod::Euler => self.integrate_euler(dt),
            IntegrationMethod::Verlet => self.integrate_verlet(dt),
            IntegrationMethod::RungeKutta4 => self.integrate_rk4(dt),
        }
    }
    
    fn integrate_euler(&mut self, dt: f32) -> Result<(), String> {
        for (_, body) in &mut self.world.bodies {
            if body.body_type == RigidBodyType::Dynamic {
                let inv_mass = 1.0 / body.mass;
                
                // Update velocity
                body.velocity[0] += body.accumulated_force[0] * inv_mass * dt;
                body.velocity[1] += body.accumulated_force[1] * inv_mass * dt;
                body.velocity[2] += body.accumulated_force[2] * inv_mass * dt;
                
                // Update position
                body.position[0] += body.velocity[0] * dt;
                body.position[1] += body.velocity[1] * dt;
                body.position[2] += body.velocity[2] * dt;
                
                // Clear accumulated forces
                body.accumulated_force = [0.0, 0.0, 0.0];
            }
        }
        
        Ok(())
    }
    
    fn integrate_verlet(&mut self, dt: f32) -> Result<(), String> {
        for (_, body) in &mut self.world.bodies {
            if body.body_type == RigidBodyType::Dynamic {
                let inv_mass = 1.0 / body.mass;
                
                // Calculate acceleration
                let acceleration = [
                    body.accumulated_force[0] * inv_mass,
                    body.accumulated_force[1] * inv_mass,
                    body.accumulated_force[2] * inv_mass,
                ];
                
                // Verlet integration
                let new_position = [
                    body.position[0] + body.velocity[0] * dt + 0.5 * acceleration[0] * dt * dt,
                    body.position[1] + body.velocity[1] * dt + 0.5 * acceleration[1] * dt * dt,
                    body.position[2] + body.velocity[2] * dt + 0.5 * acceleration[2] * dt * dt,
                ];
                
                // Update velocity (using average acceleration)
                body.velocity[0] += acceleration[0] * dt;
                body.velocity[1] += acceleration[1] * dt;
                body.velocity[2] += acceleration[2] * dt;
                
                body.position = new_position;
                body.accumulated_force = [0.0, 0.0, 0.0];
            }
        }
        
        Ok(())
    }
    
    fn integrate_rk4(&mut self, dt: f32) -> Result<(), String> {
        // Runge-Kutta 4th order integration (more accurate but slower)
        // Implementation would go here
        self.integrate_verlet(dt) // Fallback to Verlet for now
    }
    
    fn resolve_collisions(&mut self, collisions: &[Collision]) -> Result<(), String> {
        for collision in collisions {
            self.resolve_collision(collision)?;
        }
        Ok(())
    }
    
    fn resolve_collision(&mut self, collision: &Collision) -> Result<(), String> {
        let body_a = self.world.get_body_mut(collision.body_a_id).unwrap();
        let mass_a = body_a.mass;
        let vel_a = body_a.velocity;
        
        let body_b = self.world.get_body_mut(collision.body_b_id).unwrap();
        let mass_b = body_b.mass;
        let vel_b = body_b.velocity;
        
        // Simple elastic collision response
        let restitution = 0.8; // Coefficient of restitution
        
        // Calculate relative velocity
        let relative_velocity = [
            vel_a[0] - vel_b[0],
            vel_a[1] - vel_b[1],
            vel_a[2] - vel_b[2],
        ];
        
        // Calculate relative velocity along normal
        let vel_along_normal = 
            relative_velocity[0] * collision.normal[0] +
            relative_velocity[1] * collision.normal[1] +
            relative_velocity[2] * collision.normal[2];
        
        // Don't resolve if velocities are separating
        if vel_along_normal > 0.0 {
            return Ok(());
        }
        
        // Calculate impulse scalar
        let impulse_scalar = -(1.0 + restitution) * vel_along_normal / (1.0/mass_a + 1.0/mass_b);
        
        // Apply impulse
        let impulse = [
            impulse_scalar * collision.normal[0],
            impulse_scalar * collision.normal[1],
            impulse_scalar * collision.normal[2],
        ];
        
        // Update velocities
        let body_a = self.world.get_body_mut(collision.body_a_id).unwrap();
        body_a.velocity[0] += impulse[0] / mass_a;
        body_a.velocity[1] += impulse[1] / mass_a;
        body_a.velocity[2] += impulse[2] / mass_a;
        
        let body_b = self.world.get_body_mut(collision.body_b_id).unwrap();
        body_b.velocity[0] -= impulse[0] / mass_b;
        body_b.velocity[1] -= impulse[1] / mass_b;
        body_b.velocity[2] -= impulse[2] / mass_b;
        
        // Separate objects to prevent overlap
        let separation = collision.penetration_depth * 0.8; // Position correction percentage
        let total_mass = mass_a + mass_b;
        
        let correction = [
            (separation / total_mass) * collision.normal[0],
            (separation / total_mass) * collision.normal[1],
            (separation / total_mass) * collision.normal[2],
        ];
        
        let body_a = self.world.get_body_mut(collision.body_a_id).unwrap();
        body_a.position[0] += correction[0] * mass_b;
        body_a.position[1] += correction[1] * mass_b;
        body_a.position[2] += correction[2] * mass_b;
        
        let body_b = self.world.get_body_mut(collision.body_b_id).unwrap();
        body_b.position[0] -= correction[0] * mass_a;
        body_b.position[1] -= correction[1] * mass_a;
        body_b.position[2] -= correction[2] * mass_a;
        
        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PhysicsWorld {
    pub bodies: HashMap<BodyId, RigidBody>,
    pub next_body_id: u32,
}

impl PhysicsWorld {
    pub fn new() -> Self {
        Self {
            bodies: HashMap::new(),
            next_body_id: 1,
        }
    }
    
    pub fn initialize(&mut self) -> Result<(), String> {
        Ok(())
    }
    
    pub fn add_body(&mut self, body: RigidBody) -> BodyId {
        let id = BodyId(self.next_body_id);
        self.next_body_id += 1;
        self.bodies.insert(id, body);
        id
    }
    
    pub fn remove_body(&mut self, body_id: BodyId) {
        self.bodies.remove(&body_id);
    }
    
    pub fn get_body(&self, body_id: BodyId) -> Option<&RigidBody> {
        self.bodies.get(&body_id)
    }
    
    pub fn get_body_mut(&mut self, body_id: BodyId) -> Option<&mut RigidBody> {
        self.bodies.get_mut(&body_id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct BodyId(pub u32);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RigidBody {
    pub position: [f32; 3],
    pub rotation: [f32; 4], // Quaternion
    pub velocity: [f32; 3],
    pub angular_velocity: [f32; 3],
    pub mass: f32,
    pub inertia_tensor: [[f32; 3]; 3],
    pub accumulated_force: [f32; 3],
    pub accumulated_torque: [f32; 3],
    pub body_type: RigidBodyType,
    pub collider: Collider,
    pub material: PhysicsMaterial,
    pub is_sleeping: bool,
}

impl RigidBody {
    pub fn new_dynamic(mass: f32, collider: Collider) -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0], // Identity quaternion
            velocity: [0.0, 0.0, 0.0],
            angular_velocity: [0.0, 0.0, 0.0],
            mass,
            inertia_tensor: calculate_inertia_tensor(&collider, mass),
            accumulated_force: [0.0, 0.0, 0.0],
            accumulated_torque: [0.0, 0.0, 0.0],
            body_type: RigidBodyType::Dynamic,
            collider,
            material: PhysicsMaterial::default(),
            is_sleeping: false,
        }
    }
    
    pub fn new_static(collider: Collider) -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            velocity: [0.0, 0.0, 0.0],
            angular_velocity: [0.0, 0.0, 0.0],
            mass: f32::INFINITY,
            inertia_tensor: [[0.0; 3]; 3],
            accumulated_force: [0.0, 0.0, 0.0],
            accumulated_torque: [0.0, 0.0, 0.0],
            body_type: RigidBodyType::Static,
            collider,
            material: PhysicsMaterial::default(),
            is_sleeping: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RigidBodyType {
    Static,     // Never moves
    Dynamic,    // Affected by forces
    Kinematic,  // Moves but not affected by forces
}

#[derive(Debug, Clone)]
pub enum Collider {
    Box { half_extents: [f32; 3] },
    Sphere { radius: f32 },
    Capsule { radius: f32, height: f32 },
    Mesh { vertices: Vec<[f32; 3]>, indices: Vec<u32> },
}

#[derive(Debug, Clone)]
pub struct PhysicsMaterial {
    pub friction: f32,
    pub restitution: f32,
    pub density: f32,
}

impl Default for PhysicsMaterial {
    fn default() -> Self {
        Self {
            friction: 0.5,
            restitution: 0.3,
            density: 1.0,
        }
    }
}

#[derive(Debug, Clone)]
pub enum IntegrationMethod {
    Euler,
    Verlet,
    RungeKutta4,
}

#[derive(Debug, Clone)]
pub struct CollisionDetector {
    pub broad_phase: BroadPhase,
    pub narrow_phase: NarrowPhase,
}

impl CollisionDetector {
    pub fn new() -> Self {
        Self {
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
        }
    }
    
    pub fn initialize(&mut self) -> Result<(), String> {
        Ok(())
    }
    
    pub fn detect_collisions(&mut self, world: &PhysicsWorld) -> Result<Vec<Collision>, String> {
        // Broad phase: find potential collision pairs
        let potential_pairs = self.broad_phase.find_pairs(world)?;
        
        // Narrow phase: detailed collision detection
        let mut collisions = Vec::new();
        for pair in potential_pairs {
            if let Some(collision) = self.narrow_phase.test_collision(world, pair)? {
                collisions.push(collision);
            }
        }
        
        Ok(collisions)
    }
    
    pub fn raycast(&self, world: &PhysicsWorld, origin: [f32; 3], direction: [f32; 3], max_distance: f32) -> Option<RaycastHit> {
        let mut closest_hit: Option<RaycastHit> = None;
        let mut closest_distance = max_distance;
        
        for (body_id, body) in &world.bodies {
            if let Some(hit) = self.test_ray_body(origin, direction, body) {
                if hit.distance < closest_distance {
                    closest_distance = hit.distance;
                    closest_hit = Some(RaycastHit {
                        body_id: *body_id,
                        point: hit.point,
                        normal: hit.normal,
                        distance: hit.distance,
                    });
                }
            }
        }
        
        closest_hit
    }
    
    fn test_ray_body(&self, origin: [f32; 3], direction: [f32; 3], body: &RigidBody) -> Option<RaycastHit> {
        match &body.collider {
            Collider::Box { half_extents } => {
                self.ray_box_intersection(origin, direction, body.position, *half_extents)
            },
            Collider::Sphere { radius } => {
                self.ray_sphere_intersection(origin, direction, body.position, *radius)
            },
            _ => None, // TODO: Implement other shapes
        }
    }
    
    fn ray_box_intersection(&self, origin: [f32; 3], direction: [f32; 3], box_center: [f32; 3], half_extents: [f32; 3]) -> Option<RaycastHit> {
        // AABB ray intersection test
        let mut t_min = f32::NEG_INFINITY;
        let mut t_max = f32::INFINITY;
        let mut hit_normal = [0.0f32; 3];
        
        for i in 0..3 {
            let min_bound = box_center[i] - half_extents[i];
            let max_bound = box_center[i] + half_extents[i];
            
            if direction[i].abs() < f32::EPSILON {
                // Ray is parallel to slab
                if origin[i] < min_bound || origin[i] > max_bound {
                    return None; // No intersection
                }
            } else {
                let inv_dir = 1.0 / direction[i];
                let mut t1 = (min_bound - origin[i]) * inv_dir;
                let mut t2 = (max_bound - origin[i]) * inv_dir;
                
                if t1 > t2 {
                    std::mem::swap(&mut t1, &mut t2);
                }
                
                if t1 > t_min {
                    t_min = t1;
                    hit_normal = [0.0; 3];
                    hit_normal[i] = if direction[i] > 0.0 { -1.0 } else { 1.0 };
                }
                
                t_max = t_max.min(t2);
                
                if t_min > t_max {
                    return None; // No intersection
                }
            }
        }
        
        if t_min < 0.0 {
            return None; // Intersection behind ray origin
        }
        
        let hit_point = [
            origin[0] + direction[0] * t_min,
            origin[1] + direction[1] * t_min,
            origin[2] + direction[2] * t_min,
        ];
        
        Some(RaycastHit {
            body_id: BodyId(0), // Will be set by caller
            point: hit_point,
            normal: hit_normal,
            distance: t_min,
        })
    }
    
    fn ray_sphere_intersection(&self, origin: [f32; 3], direction: [f32; 3], sphere_center: [f32; 3], radius: f32) -> Option<RaycastHit> {
        let oc = [
            origin[0] - sphere_center[0],
            origin[1] - sphere_center[1],
            origin[2] - sphere_center[2],
        ];
        
        let a = direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2];
        let b = 2.0 * (oc[0] * direction[0] + oc[1] * direction[1] + oc[2] * direction[2]);
        let c = oc[0] * oc[0] + oc[1] * oc[1] + oc[2] * oc[2] - radius * radius;
        
        let discriminant = b * b - 4.0 * a * c;
        
        if discriminant < 0.0 {
            return None; // No intersection
        }
        
        let sqrt_discriminant = discriminant.sqrt();
        let t1 = (-b - sqrt_discriminant) / (2.0 * a);
        let t2 = (-b + sqrt_discriminant) / (2.0 * a);
        
        let t = if t1 > 0.0 { t1 } else if t2 > 0.0 { t2 } else { return None; };
        
        let hit_point = [
            origin[0] + direction[0] * t,
            origin[1] + direction[1] * t,
            origin[2] + direction[2] * t,
        ];
        
        let hit_normal = [
            (hit_point[0] - sphere_center[0]) / radius,
            (hit_point[1] - sphere_center[1]) / radius,
            (hit_point[2] - sphere_center[2]) / radius,
        ];
        
        Some(RaycastHit {
            body_id: BodyId(0),
            point: hit_point,
            normal: hit_normal,
            distance: t,
        })
    }
}

#[derive(Debug, Clone)]
pub struct BroadPhase {
    pub spatial_hash: SpatialHash,
}

impl BroadPhase {
    pub fn new() -> Self {
        Self {
            spatial_hash: SpatialHash::new(10.0), // 10 unit grid cells
        }
    }
    
    pub fn find_pairs(&mut self, world: &PhysicsWorld) -> Result<Vec<(BodyId, BodyId)>, String> {
        self.spatial_hash.clear();
        
        // Insert all bodies into spatial hash
        for (body_id, body) in &world.bodies {
            self.spatial_hash.insert(*body_id, body);
        }
        
        // Find overlapping pairs
        Ok(self.spatial_hash.get_overlapping_pairs())
    }
}

#[derive(Debug, Clone)]
pub struct SpatialHash {
    pub cell_size: f32,
    pub cells: HashMap<(i32, i32, i32), Vec<BodyId>>,
}

impl SpatialHash {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: HashMap::new(),
        }
    }
    
    pub fn clear(&mut self) {
        self.cells.clear();
    }
    
    pub fn insert(&mut self, body_id: BodyId, body: &RigidBody) {
        let bounds = self.calculate_body_bounds(body);
        
        let min_cell = self.world_to_cell(bounds.min);
        let max_cell = self.world_to_cell(bounds.max);
        
        for x in min_cell.0..=max_cell.0 {
            for y in min_cell.1..=max_cell.1 {
                for z in min_cell.2..=max_cell.2 {
                    self.cells.entry((x, y, z)).or_insert_with(Vec::new).push(body_id);
                }
            }
        }
    }
    
    pub fn get_overlapping_pairs(&self) -> Vec<(BodyId, BodyId)> {
        let mut pairs = Vec::new();
        
        for bodies in self.cells.values() {
            for i in 0..bodies.len() {
                for j in i+1..bodies.len() {
                    pairs.push((bodies[i], bodies[j]));
                }
            }
        }
        
        pairs
    }
    
    fn world_to_cell(&self, position: [f32; 3]) -> (i32, i32, i32) {
        (
            (position[0] / self.cell_size).floor() as i32,
            (position[1] / self.cell_size).floor() as i32,
            (position[2] / self.cell_size).floor() as i32,
        )
    }
    
    fn calculate_body_bounds(&self, body: &RigidBody) -> AABB {
        match &body.collider {
            Collider::Box { half_extents } => AABB {
                min: [
                    body.position[0] - half_extents[0],
                    body.position[1] - half_extents[1],
                    body.position[2] - half_extents[2],
                ],
                max: [
                    body.position[0] + half_extents[0],
                    body.position[1] + half_extents[1],
                    body.position[2] + half_extents[2],
                ],
            },
            Collider::Sphere { radius } => AABB {
                min: [
                    body.position[0] - radius,
                    body.position[1] - radius,
                    body.position[2] - radius,
                ],
                max: [
                    body.position[0] + radius,
                    body.position[1] + radius,
                    body.position[2] + radius,
                ],
            },
            _ => AABB {
                min: body.position,
                max: body.position,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct AABB {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

#[derive(Debug, Clone)]
pub struct NarrowPhase;

impl NarrowPhase {
    pub fn new() -> Self {
        Self
    }
    
    pub fn test_collision(&self, world: &PhysicsWorld, pair: (BodyId, BodyId)) -> Result<Option<Collision>, String> {
        let body_a = world.get_body(pair.0).unwrap();
        let body_b = world.get_body(pair.1).unwrap();
        
        match (&body_a.collider, &body_b.collider) {
            (Collider::Sphere { radius: r1 }, Collider::Sphere { radius: r2 }) => {
                Ok(self.sphere_sphere_collision(pair.0, body_a, pair.1, body_b, *r1, *r2))
            },
            (Collider::Box { half_extents: he1 }, Collider::Box { half_extents: he2 }) => {
                Ok(self.box_box_collision(pair.0, body_a, pair.1, body_b, *he1, *he2))
            },
            _ => Ok(None), // TODO: Implement other collision combinations
        }
    }
    
    fn sphere_sphere_collision(&self, id_a: BodyId, body_a: &RigidBody, id_b: BodyId, body_b: &RigidBody, radius_a: f32, radius_b: f32) -> Option<Collision> {
        let distance_vec = [
            body_b.position[0] - body_a.position[0],
            body_b.position[1] - body_a.position[1],
            body_b.position[2] - body_a.position[2],
        ];
        
        let distance = (distance_vec[0] * distance_vec[0] + distance_vec[1] * distance_vec[1] + distance_vec[2] * distance_vec[2]).sqrt();
        let min_distance = radius_a + radius_b;
        
        if distance < min_distance {
            let normal = if distance > 0.0 {
                [
                    distance_vec[0] / distance,
                    distance_vec[1] / distance,
                    distance_vec[2] / distance,
                ]
            } else {
                [1.0, 0.0, 0.0] // Arbitrary direction for overlapping spheres
            };
            
            Some(Collision {
                body_a_id: id_a,
                body_b_id: id_b,
                contact_point: [
                    body_a.position[0] + normal[0] * radius_a,
                    body_a.position[1] + normal[1] * radius_a,
                    body_a.position[2] + normal[2] * radius_a,
                ],
                normal,
                penetration_depth: min_distance - distance,
            })
        } else {
            None
        }
    }
    
    fn box_box_collision(&self, _id_a: BodyId, _body_a: &RigidBody, _id_b: BodyId, _body_b: &RigidBody, _he_a: [f32; 3], _he_b: [f32; 3]) -> Option<Collision> {
        // TODO: Implement SAT (Separating Axis Theorem) for box-box collision
        None
    }
}

#[derive(Debug, Clone)]
pub struct Collision {
    pub body_a_id: BodyId,
    pub body_b_id: BodyId,
    pub contact_point: [f32; 3],
    pub normal: [f32; 3],
    pub penetration_depth: f32,
}

#[derive(Debug, Clone)]
pub struct RaycastHit {
    pub body_id: BodyId,
    pub point: [f32; 3],
    pub normal: [f32; 3],
    pub distance: f32,
}

#[derive(Debug, Clone)]
pub struct ConstraintSolver;

impl ConstraintSolver {
    pub fn new() -> Self {
        Self
    }
    
    pub fn initialize(&mut self) -> Result<(), String> {
        Ok(())
    }
    
    pub fn solve_constraints(&mut self, _world: &mut PhysicsWorld, _dt: f32) -> Result<(), String> {
        // TODO: Implement constraint solving for joints, etc.
        Ok(())
    }
}

// Helper functions

fn calculate_inertia_tensor(collider: &Collider, mass: f32) -> [[f32; 3]; 3] {
    match collider {
        Collider::Box { half_extents } => {
            let x2 = half_extents[0] * half_extents[0];
            let y2 = half_extents[1] * half_extents[1];
            let z2 = half_extents[2] * half_extents[2];
            
            let ixx = mass / 12.0 * (4.0 * y2 + 4.0 * z2);
            let iyy = mass / 12.0 * (4.0 * x2 + 4.0 * z2);
            let izz = mass / 12.0 * (4.0 * x2 + 4.0 * y2);
            
            [
                [ixx, 0.0, 0.0],
                [0.0, iyy, 0.0],
                [0.0, 0.0, izz],
            ]
        },
        Collider::Sphere { radius } => {
            let i = 0.4 * mass * radius * radius;
            [
                [i, 0.0, 0.0],
                [0.0, i, 0.0],
                [0.0, 0.0, i],
            ]
        },
        _ => {
            // Default to unit inertia
            [
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ]
        }
    }
}