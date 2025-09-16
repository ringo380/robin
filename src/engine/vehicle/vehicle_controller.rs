use crate::engine::math::{Vec3, Point3};
use crate::engine::vehicle::{Vehicle, VehicleAction, VehicleState, VehiclePhysicsSettings};
use std::collections::HashMap;
use cgmath::InnerSpace;

#[derive(Debug, Clone)]
pub struct VehicleController {
    physics_engine: PhysicsEngine,
    collision_detector: CollisionDetector,
    vehicle_dynamics: VehicleDynamics,
    input_handler: InputHandler,
}

#[derive(Debug, Clone)]
pub struct PhysicsEngine {
    pub gravity: Vec3,
    pub air_resistance_coefficient: f32,
    pub ground_friction: f32,
    pub simulation_timestep: f32,
    pub integration_method: IntegrationMethod,
}

#[derive(Debug, Clone)]
pub enum IntegrationMethod {
    Euler,
    RungeKutta4,
    Verlet,
    LeapFrog,
}

#[derive(Debug, Clone)]
pub struct CollisionDetector {
    pub collision_grid: SpatialGrid,
    pub broad_phase_method: BroadPhaseMethod,
    pub narrow_phase_method: NarrowPhaseMethod,
    pub collision_margin: f32,
}

#[derive(Debug, Clone)]
pub enum BroadPhaseMethod {
    SpatialGrid,
    Octree,
    SweepAndPrune,
    BoundingVolumeHierarchy,
}

#[derive(Debug, Clone)]
pub enum NarrowPhaseMethod {
    SeparatingAxisTheorem,
    GilbertJohnsonKeerthi,
    MinkowskiPortalRefinement,
    BoxBoxCollision,
}

#[derive(Debug, Clone)]
pub struct SpatialGrid {
    pub cell_size: f32,
    pub grid_bounds: BoundingBox,
    pub cells: HashMap<GridCoordinate, Vec<String>>, // Cell -> Vehicle IDs
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct GridCoordinate {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone)]
pub struct BoundingBox {
    pub min: Point3,
    pub max: Point3,
}

#[derive(Debug, Clone)]
pub struct VehicleDynamics {
    pub tire_model: TireModel,
    pub aerodynamics_model: AerodynamicsModel,
    pub powertrain_model: PowertrainModel,
    pub suspension_model: SuspensionModel,
}

#[derive(Debug, Clone)]
pub struct TireModel {
    pub tire_type: TireType,
    pub grip_coefficient: f32,
    pub slip_curve: Vec<SlipPoint>,
    pub temperature_effects: TemperatureEffects,
    pub wear_model: WearModel,
}

#[derive(Debug, Clone)]
pub enum TireType {
    Street,
    Performance,
    OffRoad,
    Racing,
    Winter,
    AllSeason,
}

#[derive(Debug, Clone)]
pub struct SlipPoint {
    pub slip_ratio: f32,
    pub force_coefficient: f32,
}

#[derive(Debug, Clone)]
pub struct TemperatureEffects {
    pub optimal_temperature: f32,
    pub current_temperature: f32,
    pub temperature_coefficient: f32,
}

#[derive(Debug, Clone)]
pub struct WearModel {
    pub current_wear: f32,
    pub wear_rate: f32,
    pub performance_degradation: f32,
}

#[derive(Debug, Clone)]
pub struct AerodynamicsModel {
    pub drag_coefficient: f32,
    pub frontal_area: f32,
    pub downforce_coefficient: f32,
    pub center_of_pressure: Point3,
}

#[derive(Debug, Clone)]
pub struct PowertrainModel {
    pub engine_torque_curve: Vec<TorquePoint>,
    pub transmission_efficiency: f32,
    pub differential_ratio: f32,
    pub drive_type: DriveType,
}

#[derive(Debug, Clone)]
pub struct TorquePoint {
    pub rpm: f32,
    pub torque: f32,
}

#[derive(Debug, Clone)]
pub enum DriveType {
    FrontWheelDrive,
    RearWheelDrive,
    AllWheelDrive,
    FourWheelDrive,
}

#[derive(Debug, Clone)]
pub struct SuspensionModel {
    pub spring_rates: Vec<f32>, // Per wheel
    pub damper_rates: Vec<f32>, // Per wheel
    pub anti_roll_bar_stiffness: f32,
    pub suspension_geometry: SuspensionGeometry,
}

#[derive(Debug, Clone)]
pub struct SuspensionGeometry {
    pub camber_curve: Vec<CamberPoint>,
    pub toe_curve: Vec<ToePoint>,
    pub roll_center_height: f32,
}

#[derive(Debug, Clone)]
pub struct CamberPoint {
    pub suspension_travel: f32,
    pub camber_angle: f32,
}

#[derive(Debug, Clone)]
pub struct ToePoint {
    pub suspension_travel: f32,
    pub toe_angle: f32,
}

#[derive(Debug, Clone)]
pub struct InputHandler {
    pub input_smoothing: f32,
    pub dead_zone: f32,
    pub input_mapping: InputMapping,
    pub driver_assists: DriverAssists,
}

#[derive(Debug, Clone)]
pub struct InputMapping {
    pub throttle_sensitivity: f32,
    pub brake_sensitivity: f32,
    pub steering_sensitivity: f32,
    pub steering_linearity: f32,
}

#[derive(Debug, Clone)]
pub struct DriverAssists {
    pub abs_enabled: bool,
    pub traction_control_enabled: bool,
    pub stability_control_enabled: bool,
    pub auto_brake_enabled: bool,
    pub lane_keeping_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct VehicleInput {
    pub throttle: f32,    // 0.0 to 1.0
    pub brake: f32,       // 0.0 to 1.0
    pub steering: f32,    // -1.0 to 1.0
    pub handbrake: bool,
    pub clutch: f32,      // 0.0 to 1.0 for manual transmission
    pub gear_up: bool,
    pub gear_down: bool,
    pub horn: bool,
    pub lights: bool,
}

#[derive(Debug, Clone)]
pub struct VehicleForces {
    pub engine_force: Vec3,
    pub brake_force: Vec3,
    pub tire_forces: Vec<Vec3>, // Per wheel
    pub aerodynamic_force: Vec3,
    pub gravity_force: Vec3,
    pub suspension_forces: Vec<Vec3>, // Per wheel
}

#[derive(Debug, Clone)]
pub struct CollisionInfo {
    pub collision_point: Point3,
    pub collision_normal: Vec3,
    pub penetration_depth: f32,
    pub relative_velocity: Vec3,
    pub involved_vehicles: Vec<String>,
}

impl VehicleController {
    pub fn new() -> Self {
        Self {
            physics_engine: PhysicsEngine::new(),
            collision_detector: CollisionDetector::new(),
            vehicle_dynamics: VehicleDynamics::new(),
            input_handler: InputHandler::new(),
        }
    }

    pub fn update(&mut self, vehicles: &mut HashMap<String, Vehicle>, delta_time: f32, physics_settings: &VehiclePhysicsSettings) {
        // Update physics settings
        self.physics_engine.update_settings(physics_settings);
        
        // Update spatial grid for collision detection
        self.collision_detector.update_spatial_grid(vehicles);
        
        // Collect vehicle IDs for collision processing
        let vehicle_ids: Vec<String> = vehicles.keys().cloned().collect();

        // Process each vehicle
        for (vehicle_id, vehicle) in vehicles.iter_mut() {
            // Skip if vehicle is not operational
            if !self.should_simulate_vehicle(vehicle) {
                continue;
            }

            // Calculate forces acting on vehicle
            let forces = self.calculate_vehicle_forces(vehicle, physics_settings);

            // Apply physics integration
            self.integrate_physics(vehicle, &forces, delta_time);

            // Update vehicle systems
            self.update_vehicle_systems(vehicle, delta_time);
        }

        // Handle collisions separately to avoid borrowing conflicts
        for vehicle_id in vehicle_ids {
            let other_vehicles: HashMap<String, Vehicle> = vehicles.iter()
                .filter(|(id, _)| *id != &vehicle_id)
                .map(|(id, v)| (id.clone(), v.clone()))
                .collect();

            if let Some(vehicle) = vehicles.get_mut(&vehicle_id) {
                if self.should_simulate_vehicle(vehicle) {
                    self.handle_vehicle_collisions(&vehicle_id, vehicle, &other_vehicles);
                }
            }
        }
        
        // Post-processing
        self.post_process_physics(vehicles, delta_time);
    }

    pub fn apply_input(&mut self, vehicle_id: &str, input: VehicleInput, vehicles: &mut HashMap<String, Vehicle>) {
        if let Some(vehicle) = vehicles.get_mut(vehicle_id) {
            let processed_input = self.input_handler.process_input(input, vehicle);
            self.apply_processed_input(vehicle, processed_input);
        }
    }

    pub fn execute_vehicle_action(&mut self, vehicle: &mut Vehicle, action: VehicleAction) {
        match action {
            VehicleAction::Accelerate(force) => {
                let input = VehicleInput {
                    throttle: force.clamp(0.0, 1.0),
                    brake: 0.0,
                    steering: 0.0,
                    handbrake: false,
                    clutch: 1.0,
                    gear_up: false,
                    gear_down: false,
                    horn: false,
                    lights: false,
                };
                self.apply_processed_input(vehicle, input);
            },
            VehicleAction::Brake(force) => {
                let input = VehicleInput {
                    throttle: 0.0,
                    brake: force.clamp(0.0, 1.0),
                    steering: 0.0,
                    handbrake: false,
                    clutch: 1.0,
                    gear_up: false,
                    gear_down: false,
                    horn: false,
                    lights: false,
                };
                self.apply_processed_input(vehicle, input);
            },
            VehicleAction::Steer(angle) => {
                let input = VehicleInput {
                    throttle: 0.0,
                    brake: 0.0,
                    steering: angle.clamp(-1.0, 1.0),
                    handbrake: false,
                    clutch: 1.0,
                    gear_up: false,
                    gear_down: false,
                    horn: false,
                    lights: false,
                };
                self.apply_processed_input(vehicle, input);
            },
            VehicleAction::Stop => {
                vehicle.operational_state = VehicleState::Parked;
                vehicle.velocity = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
                vehicle.acceleration = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            },
            _ => {
                // Handle other actions as needed
            }
        }
    }

    fn should_simulate_vehicle(&self, vehicle: &Vehicle) -> bool {
        match vehicle.operational_state {
            VehicleState::Broken | VehicleState::Maintenance => false,
            _ => true,
        }
    }

    fn calculate_vehicle_forces(&mut self, vehicle: &Vehicle, physics_settings: &VehiclePhysicsSettings) -> VehicleForces {
        let mut forces = VehicleForces {
            engine_force: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
            brake_force: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
            tire_forces: vec![Vec3 { x: 0.0, y: 0.0, z: 0.0 }; 4],
            aerodynamic_force: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
            gravity_force: Vec3 { x: 0.0, y: -vehicle.mass * physics_settings.gravity, z: 0.0 },
            suspension_forces: vec![Vec3 { x: 0.0, y: 0.0, z: 0.0 }; 4],
        };
        
        // Calculate engine force
        forces.engine_force = self.calculate_engine_force(vehicle);
        
        // Calculate brake forces
        forces.brake_force = self.calculate_brake_force(vehicle);
        
        // Calculate tire forces
        forces.tire_forces = self.calculate_tire_forces(vehicle);
        
        // Calculate aerodynamic forces
        forces.aerodynamic_force = self.calculate_aerodynamic_force(vehicle, physics_settings);
        
        // Calculate suspension forces
        forces.suspension_forces = self.calculate_suspension_forces(vehicle);
        
        forces
    }

    fn calculate_engine_force(&self, vehicle: &Vehicle) -> Vec3 {
        let engine_torque = self.get_engine_torque_at_rpm(vehicle, self.calculate_engine_rpm(vehicle));
        let wheel_force = engine_torque * vehicle.transmission.gear_ratios[vehicle.transmission.current_gear as usize] * vehicle.transmission.efficiency;
        
        // Convert to world space force
        let forward_direction = self.get_vehicle_forward_direction(vehicle);
        forward_direction * wheel_force
    }

    fn calculate_brake_force(&self, vehicle: &Vehicle) -> Vec3 {
        let max_brake_force = vehicle.brakes.brake_force;
        let brake_effectiveness = vehicle.brakes.brake_condition;
        let actual_brake_force = max_brake_force * brake_effectiveness;
        
        // Apply brake force opposite to velocity direction
        if vehicle.velocity.magnitude() > 0.01 {
            let brake_direction = vehicle.velocity.normalize() * -1.0;
            brake_direction * actual_brake_force
        } else {
            Vec3 { x: 0.0, y: 0.0, z: 0.0 }
        }
    }

    fn calculate_tire_forces(&self, vehicle: &Vehicle) -> Vec<Vec3> {
        let mut tire_forces = Vec::new();
        
        // Simplified tire model - would be more complex in reality
        for i in 0..4 {
            let normal_force = vehicle.mass * 9.81 / 4.0; // Assume equal weight distribution
            let grip = self.vehicle_dynamics.tire_model.grip_coefficient;
            let max_force = normal_force * grip;
            
            // Calculate tire force based on slip
            let tire_force = Vec3 {
                x: max_force * 0.1, // Lateral force
                y: 0.0,
                z: max_force * 0.8, // Longitudinal force
            };
            
            tire_forces.push(tire_force);
        }
        
        tire_forces
    }

    fn calculate_aerodynamic_force(&self, vehicle: &Vehicle, physics_settings: &VehiclePhysicsSettings) -> Vec3 {
        let velocity_magnitude = vehicle.velocity.magnitude();
        if velocity_magnitude < 0.1 {
            return Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        }
        
        let drag_coefficient = self.vehicle_dynamics.aerodynamics_model.drag_coefficient;
        let frontal_area = self.vehicle_dynamics.aerodynamics_model.frontal_area;
        let air_density = physics_settings.air_density;
        
        let drag_magnitude = 0.5 * air_density * drag_coefficient * frontal_area * velocity_magnitude * velocity_magnitude;
        let drag_direction = vehicle.velocity.normalize() * -1.0;
        
        drag_direction * drag_magnitude
    }

    fn calculate_suspension_forces(&self, vehicle: &Vehicle) -> Vec<Vec3> {
        let mut suspension_forces = Vec::new();
        
        for i in 0..4 {
            // Simplified suspension model
            let spring_force = self.vehicle_dynamics.suspension_model.spring_rates[i] * 0.1; // Spring compression
            let damper_force = self.vehicle_dynamics.suspension_model.damper_rates[i] * 0.05; // Damping
            
            let suspension_force = Vec3 {
                x: 0.0,
                y: spring_force + damper_force,
                z: 0.0,
            };
            
            suspension_forces.push(suspension_force);
        }
        
        suspension_forces
    }

    fn integrate_physics(&mut self, vehicle: &mut Vehicle, forces: &VehicleForces, delta_time: f32) {
        // Calculate total force
        let mut total_force = forces.engine_force + forces.brake_force + forces.aerodynamic_force + forces.gravity_force;
        
        for tire_force in &forces.tire_forces {
            total_force = total_force + *tire_force;
        }
        
        for suspension_force in &forces.suspension_forces {
            total_force = total_force + *suspension_force;
        }
        
        // Calculate acceleration (F = ma)
        let acceleration = Vec3 {
            x: total_force.x / vehicle.mass,
            y: total_force.y / vehicle.mass,
            z: total_force.z / vehicle.mass,
        };
        
        // Integration method
        match self.physics_engine.integration_method {
            IntegrationMethod::Euler => {
                self.euler_integration(vehicle, acceleration, delta_time);
            },
            IntegrationMethod::RungeKutta4 => {
                self.runge_kutta_integration(vehicle, acceleration, delta_time);
            },
            _ => {
                self.euler_integration(vehicle, acceleration, delta_time);
            }
        }
        
        // Update vehicle state based on motion
        self.update_vehicle_state_from_motion(vehicle);
    }

    fn euler_integration(&mut self, vehicle: &mut Vehicle, acceleration: Vec3, delta_time: f32) {
        // Update velocity
        vehicle.velocity.x += acceleration.x * delta_time;
        vehicle.velocity.y += acceleration.y * delta_time;
        vehicle.velocity.z += acceleration.z * delta_time;
        
        // Update position
        vehicle.position.x += vehicle.velocity.x * delta_time;
        vehicle.position.y += vehicle.velocity.y * delta_time;
        vehicle.position.z += vehicle.velocity.z * delta_time;
        
        // Store acceleration
        vehicle.acceleration = acceleration;
    }

    fn runge_kutta_integration(&mut self, vehicle: &mut Vehicle, acceleration: Vec3, delta_time: f32) {
        // Simplified RK4 - would need proper implementation for production
        self.euler_integration(vehicle, acceleration, delta_time);
    }

    fn update_vehicle_systems(&mut self, vehicle: &mut Vehicle, delta_time: f32) {
        // Update engine
        self.update_engine(vehicle, delta_time);
        
        // Update transmission
        self.update_transmission(vehicle, delta_time);
        
        // Update fuel consumption
        self.update_fuel_consumption(vehicle, delta_time);
        
        // Update tire wear
        self.update_tire_wear(vehicle, delta_time);
        
        // Update maintenance
        self.update_maintenance(vehicle, delta_time);
    }

    fn update_engine(&mut self, vehicle: &mut Vehicle, delta_time: f32) {
        let rpm = self.calculate_engine_rpm(vehicle);
        
        // Update engine temperature
        let heat_generation = rpm * 0.001 + vehicle.velocity.magnitude() * 0.01;
        let heat_dissipation = vehicle.engine.temperature * 0.1;
        vehicle.engine.temperature += (heat_generation - heat_dissipation) * delta_time;
        vehicle.engine.temperature = vehicle.engine.temperature.clamp(20.0, 120.0);
        
        // Update engine condition based on temperature
        if vehicle.engine.temperature > 100.0 {
            vehicle.engine.condition -= 0.001 * delta_time;
        }
        
        vehicle.engine.condition = vehicle.engine.condition.clamp(0.0, 1.0);
    }

    fn update_transmission(&mut self, vehicle: &mut Vehicle, delta_time: f32) {
        // Automatic transmission logic
        if let TransmissionType::Automatic(max_gears) = vehicle.transmission.transmission_type {
            let rpm = self.calculate_engine_rpm(vehicle);
            let current_gear = vehicle.transmission.current_gear;
            
            // Shift up if RPM is too high
            if rpm > 4000.0 && current_gear < max_gears {
                vehicle.transmission.current_gear += 1;
            }
            // Shift down if RPM is too low
            else if rpm < 1500.0 && current_gear > 1 {
                vehicle.transmission.current_gear -= 1;
            }
        }
    }

    fn update_fuel_consumption(&mut self, vehicle: &mut Vehicle, delta_time: f32) {
        let engine_load = vehicle.velocity.magnitude() / vehicle.max_speed;
        let fuel_rate = vehicle.engine.fuel_efficiency * engine_load * delta_time;
        
        vehicle.current_fuel -= fuel_rate;
        vehicle.current_fuel = vehicle.current_fuel.max(0.0);
        
        // Update vehicle state if out of fuel
        if vehicle.current_fuel <= 0.0 {
            vehicle.operational_state = VehicleState::Broken;
        }
    }

    fn update_tire_wear(&mut self, vehicle: &mut Vehicle, delta_time: f32) {
        let distance_traveled = vehicle.velocity.magnitude() * delta_time;
        let wear_rate = self.vehicle_dynamics.tire_model.wear_model.wear_rate;
        
        self.vehicle_dynamics.tire_model.wear_model.current_wear += distance_traveled * wear_rate;
        
        // Update performance based on wear
        let wear_factor = 1.0 - (self.vehicle_dynamics.tire_model.wear_model.current_wear * 0.01);
        self.vehicle_dynamics.tire_model.wear_model.performance_degradation = wear_factor.clamp(0.1, 1.0);
    }

    fn update_maintenance(&mut self, vehicle: &mut Vehicle, delta_time: f32) {
        let usage_factor = vehicle.velocity.magnitude() * delta_time * 0.001;
        vehicle.maintenance_level -= usage_factor;
        vehicle.maintenance_level = vehicle.maintenance_level.clamp(0.0, 1.0);
    }

    fn handle_vehicle_collisions(&mut self, vehicle_id: &str, vehicle: &mut Vehicle, vehicles: &HashMap<String, Vehicle>) {
        let potential_collisions = self.collision_detector.get_potential_collisions(vehicle_id, vehicles);
        
        for collision_info in potential_collisions {
            self.resolve_collision(vehicle, &collision_info);
        }
    }

    fn resolve_collision(&mut self, vehicle: &mut Vehicle, collision_info: &CollisionInfo) {
        // Apply collision impulse
        let collision_impulse = collision_info.collision_normal * collision_info.penetration_depth * 1000.0;
        
        vehicle.velocity.x += collision_impulse.x / vehicle.mass;
        vehicle.velocity.y += collision_impulse.y / vehicle.mass;
        vehicle.velocity.z += collision_impulse.z / vehicle.mass;
        
        // Apply damage
        let damage_amount = collision_info.relative_velocity.magnitude() * 0.1;
        vehicle.damage_level += damage_amount;
        vehicle.damage_level = vehicle.damage_level.clamp(0.0, 1.0);
        
        // Update vehicle state
        if vehicle.damage_level > 0.8 {
            vehicle.operational_state = VehicleState::Broken;
        }
    }

    fn post_process_physics(&mut self, vehicles: &mut HashMap<String, Vehicle>, delta_time: f32) {
        // Apply constraints and corrections
        for vehicle in vehicles.values_mut() {
            // Ground constraint
            if vehicle.position.y < 0.0 {
                vehicle.position.y = 0.0;
                if vehicle.velocity.y < 0.0 {
                    vehicle.velocity.y = 0.0;
                }
            }
            
            // Speed limits
            let speed = vehicle.velocity.magnitude();
            if speed > vehicle.max_speed {
                let speed_ratio = vehicle.max_speed / speed;
                vehicle.velocity.x *= speed_ratio;
                vehicle.velocity.y *= speed_ratio;
                vehicle.velocity.z *= speed_ratio;
            }
        }
    }

    fn apply_processed_input(&mut self, vehicle: &mut Vehicle, input: VehicleInput) {
        // Apply throttle
        if input.throttle > 0.0 {
            vehicle.operational_state = VehicleState::Accelerating;
        }
        
        // Apply brake
        if input.brake > 0.0 {
            vehicle.operational_state = VehicleState::Decelerating;
        }
        
        // Apply steering
        if input.steering.abs() > 0.1 {
            vehicle.steering.steering_angle = input.steering * vehicle.steering.steering_ratio;
            vehicle.operational_state = VehicleState::Turning;
        }
        
        // Update idle state
        if input.throttle == 0.0 && input.brake == 0.0 && input.steering.abs() < 0.1 {
            if vehicle.velocity.magnitude() < 0.1 {
                vehicle.operational_state = VehicleState::Idle;
            } else {
                vehicle.operational_state = VehicleState::Cruising;
            }
        }
    }

    fn update_vehicle_state_from_motion(&mut self, vehicle: &mut Vehicle) {
        let speed = vehicle.velocity.magnitude();
        
        if speed < 0.1 {
            vehicle.operational_state = VehicleState::Idle;
        } else if vehicle.acceleration.magnitude() > 1.0 {
            vehicle.operational_state = VehicleState::Accelerating;
        } else if vehicle.acceleration.magnitude() < -1.0 {
            vehicle.operational_state = VehicleState::Decelerating;
        } else {
            vehicle.operational_state = VehicleState::Cruising;
        }
    }

    fn get_engine_torque_at_rpm(&self, vehicle: &Vehicle, rpm: f32) -> f32 {
        // Simplified torque curve lookup
        for torque_point in &self.vehicle_dynamics.powertrain_model.engine_torque_curve {
            if rpm <= torque_point.rpm {
                return torque_point.torque;
            }
        }
        
        // Default torque if no curve point found
        100.0
    }

    fn calculate_engine_rpm(&self, vehicle: &Vehicle) -> f32 {
        let wheel_speed = vehicle.velocity.magnitude();
        let gear_ratio = vehicle.transmission.gear_ratios[vehicle.transmission.current_gear as usize];
        let differential_ratio = self.vehicle_dynamics.powertrain_model.differential_ratio;
        
        // Simplified RPM calculation
        wheel_speed * gear_ratio * differential_ratio * 60.0 / (2.0 * std::f32::consts::PI)
    }

    fn get_vehicle_forward_direction(&self, vehicle: &Vehicle) -> Vec3 {
        // Simplified - assumes vehicle is oriented along Z axis
        Vec3 { x: 0.0, y: 0.0, z: 1.0 }
    }
}

impl PhysicsEngine {
    pub fn new() -> Self {
        Self {
            gravity: Vec3 { x: 0.0, y: -9.81, z: 0.0 },
            air_resistance_coefficient: 0.3,
            ground_friction: 0.7,
            simulation_timestep: 0.016,
            integration_method: IntegrationMethod::Euler,
        }
    }

    pub fn update_settings(&mut self, settings: &VehiclePhysicsSettings) {
        self.gravity.y = -settings.gravity;
    }
}

impl CollisionDetector {
    pub fn new() -> Self {
        Self {
            collision_grid: SpatialGrid::new(),
            broad_phase_method: BroadPhaseMethod::SpatialGrid,
            narrow_phase_method: NarrowPhaseMethod::BoxBoxCollision,
            collision_margin: 0.1,
        }
    }

    pub fn update_spatial_grid(&mut self, vehicles: &HashMap<String, Vehicle>) {
        self.collision_grid.clear();
        
        for (vehicle_id, vehicle) in vehicles {
            let grid_coord = self.collision_grid.world_to_grid(vehicle.position);
            self.collision_grid.cells
                .entry(grid_coord)
                .or_insert_with(Vec::new)
                .push(vehicle_id.clone());
        }
    }

    pub fn get_potential_collisions(&self, vehicle_id: &str, vehicles: &HashMap<String, Vehicle>) -> Vec<CollisionInfo> {
        let mut collisions = Vec::new();
        
        if let Some(vehicle) = vehicles.get(vehicle_id) {
            let grid_coord = self.collision_grid.world_to_grid(vehicle.position);
            
            // Check current cell and adjacent cells
            for dx in -1..=1 {
                for dy in -1..=1 {
                    for dz in -1..=1 {
                        let check_coord = GridCoordinate {
                            x: grid_coord.x + dx,
                            y: grid_coord.y + dy,
                            z: grid_coord.z + dz,
                        };
                        
                        if let Some(cell_vehicles) = self.collision_grid.cells.get(&check_coord) {
                            for other_vehicle_id in cell_vehicles {
                                if other_vehicle_id != vehicle_id {
                                    if let Some(other_vehicle) = vehicles.get(other_vehicle_id) {
                                        if let Some(collision_info) = self.check_collision(vehicle, other_vehicle) {
                                            collisions.push(collision_info);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        collisions
    }

    fn check_collision(&self, vehicle1: &Vehicle, vehicle2: &Vehicle) -> Option<CollisionInfo> {
        let distance = (vehicle1.position - vehicle2.position).magnitude();
        let collision_threshold = (vehicle1.dimensions.length + vehicle2.dimensions.length) / 2.0;
        
        if distance < collision_threshold {
            let collision_normal = (vehicle2.position - vehicle1.position).normalize();
            let penetration_depth = collision_threshold - distance;
            let relative_velocity = vehicle1.velocity - vehicle2.velocity;
            
            Some(CollisionInfo {
                collision_point: Point3 {
                    x: (vehicle1.position.x + vehicle2.position.x) / 2.0,
                    y: (vehicle1.position.y + vehicle2.position.y) / 2.0,
                    z: (vehicle1.position.z + vehicle2.position.z) / 2.0,
                },
                collision_normal,
                penetration_depth,
                relative_velocity,
                involved_vehicles: vec![vehicle1.id.clone(), vehicle2.id.clone()],
            })
        } else {
            None
        }
    }
}

impl SpatialGrid {
    pub fn new() -> Self {
        Self {
            cell_size: 50.0,
            grid_bounds: BoundingBox {
                min: Point3 { x: -5000.0, y: -1000.0, z: -5000.0 },
                max: Point3 { x: 5000.0, y: 1000.0, z: 5000.0 },
            },
            cells: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.cells.clear();
    }

    pub fn world_to_grid(&self, position: Point3) -> GridCoordinate {
        GridCoordinate {
            x: (position.x / self.cell_size).floor() as i32,
            y: (position.y / self.cell_size).floor() as i32,
            z: (position.z / self.cell_size).floor() as i32,
        }
    }
}

impl VehicleDynamics {
    pub fn new() -> Self {
        Self {
            tire_model: TireModel::new(),
            aerodynamics_model: AerodynamicsModel::new(),
            powertrain_model: PowertrainModel::new(),
            suspension_model: SuspensionModel::new(),
        }
    }
}

impl TireModel {
    pub fn new() -> Self {
        Self {
            tire_type: TireType::Street,
            grip_coefficient: 0.8,
            slip_curve: vec![
                SlipPoint { slip_ratio: 0.0, force_coefficient: 0.0 },
                SlipPoint { slip_ratio: 0.1, force_coefficient: 0.8 },
                SlipPoint { slip_ratio: 0.2, force_coefficient: 1.0 },
                SlipPoint { slip_ratio: 0.3, force_coefficient: 0.9 },
                SlipPoint { slip_ratio: 1.0, force_coefficient: 0.7 },
            ],
            temperature_effects: TemperatureEffects {
                optimal_temperature: 80.0,
                current_temperature: 20.0,
                temperature_coefficient: 0.02,
            },
            wear_model: WearModel {
                current_wear: 0.0,
                wear_rate: 0.001,
                performance_degradation: 1.0,
            },
        }
    }
}

impl AerodynamicsModel {
    pub fn new() -> Self {
        Self {
            drag_coefficient: 0.3,
            frontal_area: 2.5,
            downforce_coefficient: 0.0,
            center_of_pressure: Point3 { x: 0.0, y: 0.0, z: 0.0 },
        }
    }
}

impl PowertrainModel {
    pub fn new() -> Self {
        Self {
            engine_torque_curve: vec![
                TorquePoint { rpm: 1000.0, torque: 150.0 },
                TorquePoint { rpm: 2000.0, torque: 200.0 },
                TorquePoint { rpm: 3000.0, torque: 250.0 },
                TorquePoint { rpm: 4000.0, torque: 280.0 },
                TorquePoint { rpm: 5000.0, torque: 260.0 },
                TorquePoint { rpm: 6000.0, torque: 220.0 },
            ],
            transmission_efficiency: 0.95,
            differential_ratio: 3.5,
            drive_type: DriveType::FrontWheelDrive,
        }
    }
}

impl SuspensionModel {
    pub fn new() -> Self {
        Self {
            spring_rates: vec![25000.0, 25000.0, 30000.0, 30000.0], // N/m per wheel
            damper_rates: vec![3000.0, 3000.0, 3500.0, 3500.0], // Ns/m per wheel
            anti_roll_bar_stiffness: 50000.0,
            suspension_geometry: SuspensionGeometry {
                camber_curve: vec![
                    CamberPoint { suspension_travel: -0.1, camber_angle: -2.0 },
                    CamberPoint { suspension_travel: 0.0, camber_angle: 0.0 },
                    CamberPoint { suspension_travel: 0.1, camber_angle: 2.0 },
                ],
                toe_curve: vec![
                    ToePoint { suspension_travel: -0.1, toe_angle: 0.5 },
                    ToePoint { suspension_travel: 0.0, toe_angle: 0.0 },
                    ToePoint { suspension_travel: 0.1, toe_angle: -0.5 },
                ],
                roll_center_height: 0.1,
            },
        }
    }
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            input_smoothing: 0.1,
            dead_zone: 0.05,
            input_mapping: InputMapping {
                throttle_sensitivity: 1.0,
                brake_sensitivity: 1.0,
                steering_sensitivity: 1.0,
                steering_linearity: 1.0,
            },
            driver_assists: DriverAssists {
                abs_enabled: true,
                traction_control_enabled: true,
                stability_control_enabled: true,
                auto_brake_enabled: false,
                lane_keeping_enabled: false,
            },
        }
    }

    pub fn process_input(&self, input: VehicleInput, vehicle: &Vehicle) -> VehicleInput {
        let mut processed = input;
        
        // Apply dead zone
        if processed.throttle.abs() < self.dead_zone {
            processed.throttle = 0.0;
        }
        if processed.brake.abs() < self.dead_zone {
            processed.brake = 0.0;
        }
        if processed.steering.abs() < self.dead_zone {
            processed.steering = 0.0;
        }
        
        // Apply sensitivity
        processed.throttle *= self.input_mapping.throttle_sensitivity;
        processed.brake *= self.input_mapping.brake_sensitivity;
        processed.steering *= self.input_mapping.steering_sensitivity;
        
        // Apply driver assists
        if self.driver_assists.abs_enabled {
            processed = self.apply_abs(processed, vehicle);
        }
        
        if self.driver_assists.traction_control_enabled {
            processed = self.apply_traction_control(processed, vehicle);
        }
        
        processed
    }

    fn apply_abs(&self, mut input: VehicleInput, vehicle: &Vehicle) -> VehicleInput {
        // Simplified ABS logic
        let wheel_speed = vehicle.velocity.magnitude();
        if input.brake > 0.5 && wheel_speed > 10.0 {
            input.brake = 0.7; // Limit brake force to prevent wheel lockup
        }
        input
    }

    fn apply_traction_control(&self, mut input: VehicleInput, vehicle: &Vehicle) -> VehicleInput {
        // Simplified traction control
        let wheel_speed = vehicle.velocity.magnitude();
        if input.throttle > 0.8 && wheel_speed < 5.0 {
            input.throttle = 0.6; // Limit throttle to prevent wheel spin
        }
        input
    }
}

// NOTE: Cannot implement traits for external types (cgmath Vec3)
// These operators are already provided by cgmath
/*
// Subtraction operator for Vec3
impl std::ops::Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

// Addition operator for Vec3
impl std::ops::Add for Vec3 {
    type Output = Vec3;

    fn add(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

// Scalar multiplication for Vec3
impl std::ops::Mul<f32> for Vec3 {
    type Output = Vec3;

    fn mul(self, scalar: f32) -> Vec3 {
        Vec3 {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}
*/

use crate::engine::vehicle::TransmissionType;