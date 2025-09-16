use crate::engine::math::Vec2;
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use cgmath::InnerSpace;

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ColliderShape {
    Circle { radius: f32 },
    Rectangle { width: f32, height: f32 },
    Capsule { radius: f32, height: f32 },
}

impl ColliderShape {
    pub fn get_bounds(&self) -> (f32, f32) {
        match self {
            ColliderShape::Circle { radius } => (*radius * 2.0, *radius * 2.0),
            ColliderShape::Rectangle { width, height } => (*width, *height),
            ColliderShape::Capsule { radius, height } => (*radius * 2.0, *height + *radius * 2.0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum BodyType {
    Static,   // Doesn't move, infinite mass
    Dynamic,  // Affected by forces and gravity
    Kinematic, // Moves but not affected by forces
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RigidBody {
    pub id: u32,
    pub body_type: BodyType,
    pub position: Vec2,
    pub velocity: Vec2,
    pub angular_velocity: f32,
    pub rotation: f32,
    pub mass: f32,
    pub inverse_mass: f32,
    pub restitution: f32,    // Bounce factor (0.0 - 1.0)
    pub friction: f32,       // Surface friction (0.0 - 1.0) 
    pub linear_damping: f32, // Air resistance
    pub angular_damping: f32,
    pub gravity_scale: f32,  // How much gravity affects this body
    pub is_sensor: bool,     // Triggers collision events but doesn't resolve
    pub user_data: Option<String>, // Custom data for game logic
}

impl RigidBody {
    pub fn new(id: u32, body_type: BodyType, position: Vec2) -> Self {
        let (mass, inverse_mass) = match body_type {
            BodyType::Static => (f32::INFINITY, 0.0),
            _ => (1.0, 1.0),
        };

        Self {
            id,
            body_type,
            position,
            velocity: Vec2::new(0.0, 0.0),
            angular_velocity: 0.0,
            rotation: 0.0,
            mass,
            inverse_mass,
            restitution: 0.2,
            friction: 0.5,
            linear_damping: 0.01,
            angular_damping: 0.01,
            gravity_scale: 1.0,
            is_sensor: false,
            user_data: None,
        }
    }

    pub fn set_mass(&mut self, mass: f32) {
        self.mass = mass;
        self.inverse_mass = if mass == 0.0 || mass.is_infinite() { 0.0 } else { 1.0 / mass };
    }

    pub fn apply_force(&mut self, force: Vec2) {
        if self.body_type == BodyType::Dynamic {
            self.velocity += force * self.inverse_mass;
        }
    }

    pub fn apply_impulse(&mut self, impulse: Vec2) {
        if self.body_type == BodyType::Dynamic {
            self.velocity += impulse * self.inverse_mass;
        }
    }

    pub fn set_velocity(&mut self, velocity: Vec2) {
        if self.body_type != BodyType::Static {
            self.velocity = velocity;
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Collider {
    pub body_id: u32,
    pub shape: ColliderShape,
    pub offset: Vec2, // Offset from body position
    pub collision_groups: u32, // Bitmask for collision filtering
    pub collision_mask: u32,   // What groups this collider collides with
}

impl Collider {
    pub fn new(body_id: u32, shape: ColliderShape) -> Self {
        Self {
            body_id,
            shape,
            offset: Vec2::new(0.0, 0.0),
            collision_groups: 1,
            collision_mask: !0, // Collide with everything by default
        }
    }

    pub fn with_offset(mut self, offset: Vec2) -> Self {
        self.offset = offset;
        self
    }

    pub fn with_collision_groups(mut self, groups: u32, mask: u32) -> Self {
        self.collision_groups = groups;
        self.collision_mask = mask;
        self
    }

    pub fn get_world_position(&self, body_position: Vec2) -> Vec2 {
        body_position + self.offset
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Collision {
    pub body_a: u32,
    pub body_b: u32,
    pub point: Vec2,
    pub normal: Vec2,    // From A to B
    pub penetration: f32,
    pub is_sensor_collision: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PhysicsWorld {
    pub gravity: Vec2,
    pub bodies: HashMap<u32, RigidBody>,
    pub colliders: HashMap<u32, Collider>,
    pub collisions: Vec<Collision>,
    #[serde(skip)]
    pub collision_listeners: HashMap<String, Box<dyn Fn(&Collision)>>,
    next_body_id: u32,
    time_accumulator: f32,
    fixed_timestep: f32,
    broad_phase_grid: SpatialGrid,
}

impl Clone for PhysicsWorld {
    fn clone(&self) -> Self {
        Self {
            gravity: self.gravity,
            bodies: self.bodies.clone(),
            colliders: self.colliders.clone(),
            collisions: self.collisions.clone(),
            collision_listeners: HashMap::new(), // Reset function pointers
            next_body_id: self.next_body_id,
            time_accumulator: self.time_accumulator,
            fixed_timestep: self.fixed_timestep,
            broad_phase_grid: self.broad_phase_grid.clone(),
        }
    }
}

impl std::fmt::Debug for PhysicsWorld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PhysicsWorld")
            .field("gravity", &self.gravity)
            .field("bodies", &self.bodies)
            .field("colliders", &self.colliders)
            .field("collisions", &self.collisions)
            .field("collision_listeners", &format!("[{} listeners]", self.collision_listeners.len()))
            .field("next_body_id", &self.next_body_id)
            .field("time_accumulator", &self.time_accumulator)
            .field("fixed_timestep", &self.fixed_timestep)
            .field("broad_phase_grid", &self.broad_phase_grid)
            .finish()
    }
}

impl PhysicsWorld {
    pub fn new() -> Self {
        Self {
            gravity: Vec2::new(0.0, -980.0), // Default Earth gravity
            bodies: HashMap::new(),
            colliders: HashMap::new(),
            collisions: Vec::new(),
            collision_listeners: HashMap::new(),
            next_body_id: 0,
            time_accumulator: 0.0,
            fixed_timestep: 1.0 / 60.0, // 60 Hz physics
            broad_phase_grid: SpatialGrid::new(100.0), // 100 unit grid cells
        }
    }

    pub fn set_gravity(&mut self, gravity: Vec2) {
        self.gravity = gravity;
    }

    pub fn add_rigid_body(&mut self, body_type: BodyType, position: Vec2) -> u32 {
        let id = self.next_body_id;
        self.next_body_id += 1;
        
        let body = RigidBody::new(id, body_type, position);
        self.bodies.insert(id, body);
        id
    }

    pub fn add_collider(&mut self, body_id: u32, shape: ColliderShape) -> Option<u32> {
        if self.bodies.contains_key(&body_id) {
            let collider = Collider::new(body_id, shape);
            self.colliders.insert(body_id, collider);
            Some(body_id)
        } else {
            None
        }
    }

    pub fn get_body(&self, id: u32) -> Option<&RigidBody> {
        self.bodies.get(&id)
    }

    pub fn get_body_mut(&mut self, id: u32) -> Option<&mut RigidBody> {
        self.bodies.get_mut(&id)
    }

    pub fn remove_body(&mut self, id: u32) {
        self.bodies.remove(&id);
        self.colliders.remove(&id);
    }

    pub fn step(&mut self, delta_time: f32) {
        self.time_accumulator += delta_time;

        // Fixed timestep integration for stability
        while self.time_accumulator >= self.fixed_timestep {
            self.integrate_forces(self.fixed_timestep);
            self.update_broad_phase();
            self.detect_collisions();
            self.resolve_collisions();
            self.integrate_velocities(self.fixed_timestep);
            self.time_accumulator -= self.fixed_timestep;
        }
    }

    fn integrate_forces(&mut self, dt: f32) {
        for body in self.bodies.values_mut() {
            if body.body_type != BodyType::Dynamic {
                continue;
            }

            // Apply gravity
            let gravity_force = self.gravity * body.gravity_scale;
            body.apply_force(gravity_force * dt);

            // Apply damping
            body.velocity *= 1.0 - (body.linear_damping * dt);
            body.angular_velocity *= 1.0 - (body.angular_damping * dt);
        }
    }

    fn integrate_velocities(&mut self, dt: f32) {
        for body in self.bodies.values_mut() {
            if body.body_type == BodyType::Static {
                continue;
            }

            // Update position and rotation
            body.position += body.velocity * dt;
            body.rotation += body.angular_velocity * dt;
        }
    }

    fn update_broad_phase(&mut self) {
        self.broad_phase_grid.clear();
        
        for (body_id, body) in &self.bodies {
            if let Some(collider) = self.colliders.get(body_id) {
                let world_pos = collider.get_world_position(body.position);
                let (width, height) = collider.shape.get_bounds();
                self.broad_phase_grid.insert(*body_id, world_pos, width, height);
            }
        }
    }

    fn detect_collisions(&mut self) {
        self.collisions.clear();
        
        let potential_pairs = self.broad_phase_grid.get_collision_pairs();
        
        for (id_a, id_b) in potential_pairs {
            if let (Some(body_a), Some(body_b)) = (self.bodies.get(&id_a), self.bodies.get(&id_b)) {
                if let (Some(collider_a), Some(collider_b)) = (self.colliders.get(&id_a), self.colliders.get(&id_b)) {
                    // Check collision groups
                    if (collider_a.collision_groups & collider_b.collision_mask) == 0 ||
                       (collider_b.collision_groups & collider_a.collision_mask) == 0 {
                        continue;
                    }

                    if let Some(collision) = self.check_collision(body_a, collider_a, body_b, collider_b) {
                        self.collisions.push(collision);
                    }
                }
            }
        }
    }

    fn check_collision(&self, body_a: &RigidBody, collider_a: &Collider, body_b: &RigidBody, collider_b: &Collider) -> Option<Collision> {
        let pos_a = collider_a.get_world_position(body_a.position);
        let pos_b = collider_b.get_world_position(body_b.position);

        match (&collider_a.shape, &collider_b.shape) {
            (ColliderShape::Circle { radius: r1 }, ColliderShape::Circle { radius: r2 }) => {
                let distance = (pos_b - pos_a).magnitude();
                let combined_radius = r1 + r2;
                
                if distance < combined_radius {
                    let normal = if distance > 0.001 {
                        (pos_b - pos_a) / distance
                    } else {
                        Vec2::new(1.0, 0.0) // Arbitrary direction for coincident circles
                    };
                    
                    let penetration = combined_radius - distance;
                    let contact_point = pos_a + normal * (*r1);
                    
                    Some(Collision {
                        body_a: body_a.id,
                        body_b: body_b.id,
                        point: contact_point,
                        normal,
                        penetration,
                        is_sensor_collision: body_a.is_sensor || body_b.is_sensor,
                    })
                } else {
                    None
                }
            }
            // Add more collision detection for other shape combinations
            _ => None, // Placeholder for other shape combinations
        }
    }

    fn resolve_collisions(&mut self) {
        for collision in &self.collisions {
            if collision.is_sensor_collision {
                // Trigger collision events but don't resolve
                self.trigger_collision_events(collision);
                continue;
            }

            // Get body IDs to avoid borrowing issues
            let body_a_id = collision.body_a;
            let body_b_id = collision.body_b;
            
            // Calculate collision response values first to avoid borrowing issues
            let (body_a_props, body_b_props) = {
                let body_a = self.bodies.get(&body_a_id);
                let body_b = self.bodies.get(&body_b_id);
                
                match (body_a, body_b) {
                    (Some(a), Some(b)) => (
                        (a.body_type, a.inverse_mass, a.restitution, a.friction, a.velocity),
                        (b.body_type, b.inverse_mass, b.restitution, b.friction, b.velocity)
                    ),
                    _ => continue,
                }
            };
            
            // Positional correction (separate overlapping bodies)
            let percent = 0.8; // How much to correct (0.0 - 1.0)
            let slop = 0.01;   // Allow small overlap
            let correction = (collision.penetration - slop).max(0.0) / (body_a_props.1 + body_b_props.1) * percent;
            
            let correction_vector = collision.normal * correction;
            
            // Calculate velocity resolution values
            let relative_velocity = body_b_props.4 - body_a_props.4;
            let vel_along_normal = relative_velocity.dot(collision.normal);
            
            if vel_along_normal <= 0.0 { // Only resolve if objects are approaching
                let restitution = (body_a_props.2 + body_b_props.2) * 0.5;
                let impulse_scalar = -(1.0 + restitution) * vel_along_normal / (body_a_props.1 + body_b_props.1);
                let impulse = collision.normal * impulse_scalar;
                
                // Apply changes to body A
                if let Some(body_a) = self.bodies.get_mut(&body_a_id) {
                    if body_a.body_type == BodyType::Dynamic {
                        body_a.position -= correction_vector * body_a.inverse_mass;
                        body_a.velocity -= impulse * body_a.inverse_mass;
                    }
                }
                
                // Apply changes to body B
                if let Some(body_b) = self.bodies.get_mut(&body_b_id) {
                    if body_b.body_type == BodyType::Dynamic {
                        body_b.position += correction_vector * body_b.inverse_mass;
                        body_b.velocity += impulse * body_b.inverse_mass;
                    }
                }

            }

            self.trigger_collision_events(collision);
        }
    }

    fn trigger_collision_events(&self, collision: &Collision) {
        // Trigger collision callbacks
        for (_, listener) in &self.collision_listeners {
            listener(collision);
        }
    }

    pub fn add_collision_listener<F>(&mut self, name: String, listener: F) 
    where 
        F: Fn(&Collision) + 'static
    {
        self.collision_listeners.insert(name, Box::new(listener));
    }

    pub fn remove_collision_listener(&mut self, name: &str) {
        self.collision_listeners.remove(name);
    }

    // Utility methods for common physics operations
    pub fn apply_explosion(&mut self, center: Vec2, force: f32, radius: f32) {
        for body in self.bodies.values_mut() {
            if body.body_type != BodyType::Dynamic {
                continue;
            }

            let distance_vec = body.position - center;
            let distance = distance_vec.magnitude();
            
            if distance < radius && distance > 0.001 {
                let direction = distance_vec / distance;
                let falloff = 1.0 - (distance / radius);
                let impulse = direction * force * falloff;
                body.apply_impulse(impulse);
            }
        }
    }

    pub fn query_point(&self, point: Vec2) -> Vec<u32> {
        let mut results = Vec::new();
        
        for (body_id, body) in &self.bodies {
            if let Some(collider) = self.colliders.get(body_id) {
                let world_pos = collider.get_world_position(body.position);
                
                match collider.shape {
                    ColliderShape::Circle { radius } => {
                        if (point - world_pos).magnitude() <= radius {
                            results.push(*body_id);
                        }
                    }
                    ColliderShape::Rectangle { width, height } => {
                        let half_width = width * 0.5;
                        let half_height = height * 0.5;
                        let relative_pos = point - world_pos;
                        
                        if relative_pos.x.abs() <= half_width && relative_pos.y.abs() <= half_height {
                            results.push(*body_id);
                        }
                    }
                    _ => {} // Add other shapes as needed
                }
            }
        }
        
        results
    }

    pub fn query_aabb(&self, min: Vec2, max: Vec2) -> Vec<u32> {
        let mut results = Vec::new();
        
        for (body_id, body) in &self.bodies {
            if let Some(collider) = self.colliders.get(body_id) {
                let world_pos = collider.get_world_position(body.position);
                let (width, height) = collider.shape.get_bounds();
                let half_width = width * 0.5;
                let half_height = height * 0.5;
                
                let body_min = Vec2::new(world_pos.x - half_width, world_pos.y - half_height);
                let body_max = Vec2::new(world_pos.x + half_width, world_pos.y + half_height);
                
                // AABB overlap test
                if body_min.x <= max.x && body_max.x >= min.x &&
                   body_min.y <= max.y && body_max.y >= min.y {
                    results.push(*body_id);
                }
            }
        }
        
        results
    }
}

// Simple spatial partitioning for broad-phase collision detection
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct SpatialGrid {
    cell_size: f32,
    cells: HashMap<(i32, i32), HashSet<u32>>,
}

impl SpatialGrid {
    fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: HashMap::new(),
        }
    }

    fn clear(&mut self) {
        self.cells.clear();
    }

    fn get_cell_coords(&self, pos: Vec2) -> (i32, i32) {
        (
            (pos.x / self.cell_size).floor() as i32,
            (pos.y / self.cell_size).floor() as i32,
        )
    }

    fn insert(&mut self, body_id: u32, position: Vec2, width: f32, height: f32) {
        let half_width = width * 0.5;
        let half_height = height * 0.5;
        
        let min_cell = self.get_cell_coords(Vec2::new(position.x - half_width, position.y - half_height));
        let max_cell = self.get_cell_coords(Vec2::new(position.x + half_width, position.y + half_height));
        
        for x in min_cell.0..=max_cell.0 {
            for y in min_cell.1..=max_cell.1 {
                self.cells.entry((x, y)).or_insert_with(HashSet::new).insert(body_id);
            }
        }
    }

    fn get_collision_pairs(&self) -> Vec<(u32, u32)> {
        let mut pairs = Vec::new();
        let mut checked_pairs = HashSet::new();
        
        for cell in self.cells.values() {
            let cell_bodies: Vec<_> = cell.iter().collect();
            
            for i in 0..cell_bodies.len() {
                for j in (i + 1)..cell_bodies.len() {
                    let id_a = *cell_bodies[i];
                    let id_b = *cell_bodies[j];
                    let pair = if id_a < id_b { (id_a, id_b) } else { (id_b, id_a) };
                    
                    if checked_pairs.insert(pair) {
                        pairs.push(pair);
                    }
                }
            }
        }
        
        pairs
    }
}