use nalgebra::{Vector3, Point3};
use super::{CharacterState, MovementMode};
use std::collections::HashMap;

pub struct CharacterPhysics {
    // Physics constants
    gravity: f32,
    terminal_velocity: f32,
    ground_check_distance: f32,
    
    // Collision system
    collision_layers: HashMap<String, Vec<CollisionBox>>,
    character_radius: f32,
    character_height: f32,
    
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
            
            collision_layers: HashMap::new(),
            character_radius: 0.3,
            character_height: 1.8,
            
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
                self.apply_gravity(state, delta_time);
                self.check_ground_collision(state);
                self.resolve_collisions(state, delta_time);
                self.apply_friction(state, delta_time);
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
}