use std::collections::HashMap;
use crate::engine::math::{Point3, Vec3};
use crate::engine::vehicle::{
    Vehicle, VehicleType, EngineType, TransmissionType, DrivetrainType,
    SuspensionType, BrakeType, TireType, VehicleClass, FuelType,
    Engine, Transmission, Suspension, Brakes, Tires, Electronics,
    VehicleSpecs, VehicleStats, VehicleState, VehiclePhysicalState,
};

#[derive(Debug, Clone)]
pub struct VehicleDesigner {
    design_templates: HashMap<String, VehicleTemplate>,
    component_library: ComponentLibrary,
    customization_options: CustomizationOptions,
    design_validator: DesignValidator,
    performance_calculator: PerformanceCalculator,
    cost_calculator: CostCalculator,
    active_designs: HashMap<String, VehicleDesign>,
}

impl VehicleDesigner {
    pub fn new() -> Self {
        let mut designer = Self {
            design_templates: HashMap::new(),
            component_library: ComponentLibrary::new(),
            customization_options: CustomizationOptions::new(),
            design_validator: DesignValidator::new(),
            performance_calculator: PerformanceCalculator::new(),
            cost_calculator: CostCalculator::new(),
            active_designs: HashMap::new(),
        };
        
        designer.initialize_default_templates();
        designer
    }
    
    pub fn create_vehicle_design(&mut self, design_name: &str, template_name: &str) -> Result<String, String> {
        if let Some(template) = self.design_templates.get(template_name) {
            let design = VehicleDesign::from_template(design_name, template.clone());
            let design_id = format!("design_{}", self.active_designs.len());
            self.active_designs.insert(design_id.clone(), design);
            Ok(design_id)
        } else {
            Err(format!("Template '{}' not found", template_name))
        }
    }
    
    pub fn customize_component(&mut self, design_id: &str, component_type: ComponentType, component_id: &str) -> Result<(), String> {
        if let Some(design) = self.active_designs.get_mut(design_id) {
            if let Some(component) = self.component_library.get_component(component_type, component_id) {
                design.set_component(component_type, component);
                self.recalculate_performance(design_id)?;
                Ok(())
            } else {
                Err(format!("Component '{}' not found", component_id))
            }
        } else {
            Err(format!("Design '{}' not found", design_id))
        }
    }
    
    pub fn validate_design(&self, design_id: &str) -> Result<DesignValidationResult, String> {
        if let Some(design) = self.active_designs.get(design_id) {
            Ok(self.design_validator.validate(design))
        } else {
            Err(format!("Design '{}' not found", design_id))
        }
    }
    
    pub fn calculate_performance(&self, design_id: &str) -> Result<PerformanceMetrics, String> {
        if let Some(design) = self.active_designs.get(design_id) {
            Ok(self.performance_calculator.calculate(design))
        } else {
            Err(format!("Design '{}' not found", design_id))
        }
    }
    
    pub fn calculate_cost(&self, design_id: &str) -> Result<CostBreakdown, String> {
        if let Some(design) = self.active_designs.get(design_id) {
            Ok(self.cost_calculator.calculate(design))
        } else {
            Err(format!("Design '{}' not found", design_id))
        }
    }
    
    pub fn build_vehicle(&self, design_id: &str) -> Result<Vehicle, String> {
        if let Some(design) = self.active_designs.get(design_id) {
            let validation = self.design_validator.validate(design);
            if validation.is_valid {
                Ok(design.build_vehicle())
            } else {
                Err(format!("Design validation failed: {:?}", validation.errors))
            }
        } else {
            Err(format!("Design '{}' not found", design_id))
        }
    }
    
    pub fn clone_design(&mut self, source_design_id: &str, new_name: &str) -> Result<String, String> {
        if let Some(source_design) = self.active_designs.get(source_design_id).cloned() {
            let mut cloned_design = source_design;
            cloned_design.name = new_name.to_string();
            let new_design_id = format!("design_{}", self.active_designs.len());
            self.active_designs.insert(new_design_id.clone(), cloned_design);
            Ok(new_design_id)
        } else {
            Err(format!("Source design '{}' not found", source_design_id))
        }
    }
    
    pub fn get_available_templates(&self) -> Vec<String> {
        self.design_templates.keys().cloned().collect()
    }
    
    pub fn get_available_components(&self, component_type: ComponentType) -> Vec<String> {
        self.component_library.get_component_ids(component_type)
    }
    
    pub fn save_design_as_template(&mut self, design_id: &str, template_name: &str) -> Result<(), String> {
        if let Some(design) = self.active_designs.get(design_id) {
            let template = VehicleTemplate::from_design(design);
            self.design_templates.insert(template_name.to_string(), template);
            Ok(())
        } else {
            Err(format!("Design '{}' not found", design_id))
        }
    }
    
    fn initialize_default_templates(&mut self) {
        self.design_templates.insert("sports_car".to_string(), VehicleTemplate::sports_car());
        self.design_templates.insert("sedan".to_string(), VehicleTemplate::sedan());
        self.design_templates.insert("suv".to_string(), VehicleTemplate::suv());
        self.design_templates.insert("truck".to_string(), VehicleTemplate::truck());
        self.design_templates.insert("motorcycle".to_string(), VehicleTemplate::motorcycle());
        self.design_templates.insert("aircraft".to_string(), VehicleTemplate::aircraft());
        self.design_templates.insert("boat".to_string(), VehicleTemplate::boat());
    }
    
    fn recalculate_performance(&self, design_id: &str) -> Result<(), String> {
        if let Some(_design) = self.active_designs.get(design_id) {
            Ok(())
        } else {
            Err(format!("Design '{}' not found", design_id))
        }
    }

    pub fn create_vehicle_from_template(&self, template: VehicleTemplate, position: Point3) -> Vehicle {
        // Create a basic vehicle from the template
        // This is a simplified implementation that creates a vehicle with default components
        let specs = Self::default_specs();
        let stats = Self::default_stats();
        let physical_state = VehiclePhysicalState {
            position,
            rotation: Vec3::new(0.0, 0.0, 0.0),
            velocity: Vec3::new(0.0, 0.0, 0.0),
            acceleration: Vec3::new(0.0, 0.0, 0.0),
            angular_velocity: Vec3::new(0.0, 0.0, 0.0),
            engine_rpm: 800.0,
            gear: 1,
            throttle_position: 0.0,
            brake_pressure: 0.0,
            steering_angle: 0.0,
            fuel_level: 1.0,
            speed: 0.0,
            is_engine_running: false,
        };

        Vehicle {
            id: format!("vehicle_{}", rand::random::<u32>()),
            name: template.name.clone(),
            vehicle_type: template.vehicle_type,
            vehicle_class: template.vehicle_class,
            position: physical_state.position,
            velocity: physical_state.velocity,
            rotation: physical_state.rotation,
            acceleration: physical_state.acceleration,

            // Physical properties
            dimensions: super::VehicleDimensions {
                length: specs.length,
                width: specs.width,
                height: specs.height,
                wheelbase: specs.wheelbase,
                ground_clearance: specs.ground_clearance,
                turning_radius: 5.0,
            },
            mass: specs.weight,
            engine_power: specs.engine_power,
            max_speed: specs.max_speed,
            fuel_capacity: specs.fuel_capacity,
            current_fuel: specs.fuel_capacity * physical_state.fuel_level,
            cargo_capacity: specs.cargo_capacity,
            current_cargo: 0.0,

            // Component systems
            engine: Engine::default(),
            transmission: Self::default_transmission(),
            suspension: Self::default_suspension(),
            brakes: Self::default_brakes(),
            tires: Self::default_tires(),
            steering: super::SteeringSystem {
                steering_type: super::SteeringType::ElectricPowerSteering,
                steering_ratio: 16.0,
                power_assist: 1.0,
                steering_angle: physical_state.steering_angle,
                wheel_alignment: super::WheelAlignment {
                    camber: 0.0,
                    caster: 3.0,
                    toe: 0.0,
                },
            },
            electronics: Self::default_electronics(),

            // Operational state
            operational_state: super::VehicleState::Idle,
            state: physical_state,
            stats,
            specs,
            driver: None,
            passengers: Vec::new(),
            cargo: Vec::new(),
            maintenance_level: 1.0,
            damage_level: 0.0,

            // Navigation and routing
            current_route: None,
            destination: None,
            waypoints: Vec::new(),
            navigation_state: super::NavigationState {
                current_waypoint: 0,
                distance_to_destination: 0.0,
                estimated_arrival: 0,
                traffic_conditions: super::TrafficCondition::Clear,
                route_deviations: Vec::new(),
            },

            // AI and behavior
            behavior_profile: super::VehicleBehaviorProfile {
                driving_style: super::DrivingStyle::Normal,
                aggressiveness: 0.5,
                following_distance: 2.0,
                speed_preference: 1.0,
                lane_change_frequency: 0.3,
                risk_tolerance: 0.5,
                fuel_efficiency_priority: 0.7,
                time_priority: 0.6,
            },
            ai_controller: None,
        }
    }

    fn default_transmission() -> Transmission {
        Transmission {
            transmission_type: TransmissionType::Automatic(6),
            gear_ratios: vec![3.0, 2.0, 1.5, 1.0, 0.8, 0.6],
            current_gear: 1,
            final_drive_ratio: 3.5,
            shift_time: 0.5,
            torque_converter_ratio: 2.5,
            efficiency: 0.9,
            condition: 1.0,
        }
    }

    fn default_suspension() -> Suspension {
        Suspension {
            suspension_type: SuspensionType::Independent,
            spring_rate: 20000.0,
            damping_coefficient: 1500.0,
            ground_clearance: 0.15,
            compression: 0.0,
            anti_roll_bar_stiffness: 5000.0,
            ride_height: 0.15,
            travel: 0.15,
            preload: 0.05,
            condition: 1.0,
        }
    }

    fn default_brakes() -> Brakes {
        Brakes {
            brake_type: BrakeType::Disc,
            brake_force: 8000.0,
            brake_balance: 0.6,
            abs_enabled: true,
            brake_condition: 1.0,
            brake_temperature: 20.0,
            max_braking_force: 8000.0,
            brake_bias: 0.6,
            brake_assist_enabled: true,
            electronic_brakeforce_distribution: true,
            thermal_capacity: 2000.0,
        }
    }

    fn default_tires() -> Tires {
        Tires {
            front_left: super::TireInfo {
                pressure: 32.0,
                wear: 0.0,
                temperature: 20.0,
                grip_coefficient: 1.0,
            },
            front_right: super::TireInfo {
                pressure: 32.0,
                wear: 0.0,
                temperature: 20.0,
                grip_coefficient: 1.0,
            },
            rear_left: super::TireInfo {
                pressure: 32.0,
                wear: 0.0,
                temperature: 20.0,
                grip_coefficient: 1.0,
            },
            rear_right: super::TireInfo {
                pressure: 32.0,
                wear: 0.0,
                temperature: 20.0,
                grip_coefficient: 1.0,
            },
            tire_type: TireType::AllSeason,
            size: "225/60R16".to_string(),
            pressure: 32.0,
            tread_depth: 8.0,
            compound: "Standard".to_string(),
            max_grip: 1.0,
            rolling_resistance: 0.01,
            wear_rate: 0.0001,
            width: 225,
            aspect_ratio: 60,
            diameter: 16,
            grip_coefficient: 1.0,
            optimal_pressure: 32.0,
            load_rating: 1000.0,
        }
    }

    fn default_electronics() -> Electronics {
        Electronics {
            stability_control: true,
            traction_control: true,
            anti_lock_brakes: true,
            cruise_control: true,
            adaptive_cruise: false,
            lane_keeping: false,
            collision_avoidance: false,
            parking_assist: false,
            autonomous_level: super::AutonomyLevel::None,
            ecu_version: "1.0".to_string(),
            sensors: vec!["speed".to_string(), "rpm".to_string(), "throttle".to_string()],
            safety_systems: vec!["abs".to_string(), "airbags".to_string()],
            infotainment_features: vec!["radio".to_string(), "bluetooth".to_string()],
            driver_assistance: vec!["cruise_control".to_string()],
            connectivity: vec!["bluetooth".to_string()],
            ecu_enabled: true,
            abs_enabled: true,
            traction_control_enabled: true,
            stability_control_enabled: true,
            launch_control_enabled: false,
            adaptive_suspension_enabled: false,
            torque_vectoring_enabled: false,
        }
    }

    fn default_specs() -> VehicleSpecs {
        VehicleSpecs {
            engine_power: 150.0,
            max_speed: 180.0,
            acceleration: 8.0,
            fuel_efficiency: 25.0,
            cargo_capacity: 400.0,
            passenger_capacity: 5,
            seating_capacity: 5,
            length: 4.5,
            width: 1.8,
            height: 1.6,
            weight: 1500.0,
            wheelbase: 2.7,
            ground_clearance: 0.15,
            drag_coefficient: 0.3,
            frontal_area: 2.5,
            fuel_capacity: 60.0,
        }
    }

    fn default_stats() -> VehicleStats {
        VehicleStats {
            total_distance: 0.0,
            fuel_consumed: 0.0,
            average_speed: 0.0,
            max_speed: 200.0,
            top_speed: 200.0,
            acceleration_0_60: 8.0,
            braking_60_0: 40.0,
            fuel_economy_city: 25.0,
            fuel_economy_highway: 35.0,
            fuel_efficiency: 25.0,
            power_to_weight_ratio: 0.1,
            handling_rating: 7.0,
            comfort_rating: 7.0,
            reliability_rating: 8.0,
            maintenance_score: 7.5,
            performance_rating: 7.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VehicleDesign {
    pub name: String,
    pub vehicle_type: VehicleType,
    pub vehicle_class: VehicleClass,
    pub selected_components: ComponentSelection,
    pub customizations: DesignCustomizations,
    pub performance_targets: PerformanceTargets,
    pub constraints: DesignConstraints,
}

impl VehicleDesign {
    pub fn from_template(name: &str, template: VehicleTemplate) -> Self {
        Self {
            name: name.to_string(),
            vehicle_type: template.vehicle_type,
            vehicle_class: template.vehicle_class,
            selected_components: template.default_components,
            customizations: template.default_customizations,
            performance_targets: template.performance_targets,
            constraints: template.constraints,
        }
    }
    
    pub fn set_component(&mut self, component_type: ComponentType, component: ComponentData) {
        match component_type {
            ComponentType::Engine => self.selected_components.engine = Some(component),
            ComponentType::Transmission => self.selected_components.transmission = Some(component),
            ComponentType::Suspension => self.selected_components.suspension = Some(component),
            ComponentType::Brakes => self.selected_components.brakes = Some(component),
            ComponentType::Tires => self.selected_components.tires = Some(component),
            ComponentType::Electronics => self.selected_components.electronics = Some(component),
        }
    }
    
    pub fn build_vehicle(&self) -> Vehicle {
        let engine = self.build_engine();
        let transmission = self.build_transmission();
        let suspension = self.build_suspension();
        let brakes = self.build_brakes();
        let tires = self.build_tires();
        let electronics = self.build_electronics();
        
        let specs = VehicleSpecs {
            engine_power: engine.max_power,
            max_speed: 200.0,
            acceleration: 6.0,
            fuel_efficiency: 25.0,
            cargo_capacity: 500.0,
            passenger_capacity: 5,
            seating_capacity: 5,
            length: self.customizations.dimensions.length,
            width: self.customizations.dimensions.width,
            height: self.customizations.dimensions.height,
            weight: self.calculate_total_weight(),
            wheelbase: self.customizations.dimensions.wheelbase,
            ground_clearance: self.customizations.dimensions.ground_clearance,
            drag_coefficient: self.customizations.aerodynamics.drag_coefficient,
            frontal_area: self.customizations.aerodynamics.frontal_area,
            fuel_capacity: self.customizations.fuel_system.capacity,
        };
        
        let stats = VehicleStats {
            total_distance: 0.0,
            fuel_consumed: 0.0,
            average_speed: 0.0,
            max_speed: 200.0,
            top_speed: 200.0,
            acceleration_0_60: 6.0,
            braking_60_0: 40.0,
            fuel_economy_city: 25.0,
            fuel_economy_highway: 35.0,
            fuel_efficiency: 25.0,
            power_to_weight_ratio: engine.max_power / specs.weight,
            handling_rating: 8.0,
            comfort_rating: 7.0,
            reliability_rating: 8.5,
            maintenance_score: 9.0,
            performance_rating: 8.5,
        };
        
        let state = VehiclePhysicalState {
            position: Point3::new(0.0, 0.0, 0.0),
            rotation: Vec3::new(0.0, 0.0, 0.0),
            velocity: Vec3::new(0.0, 0.0, 0.0),
            acceleration: Vec3::new(0.0, 0.0, 0.0),
            angular_velocity: Vec3::new(0.0, 0.0, 0.0),
            engine_rpm: 0.0,
            gear: 0,
            throttle_position: 0.0,
            brake_pressure: 0.0,
            steering_angle: 0.0,
            fuel_level: 1.0,
            speed: 0.0,
            is_engine_running: false,
        };
        
        Vehicle {
            id: format!("vehicle_{}", rand::random::<u32>()),
            name: self.name.clone(),
            vehicle_type: self.vehicle_type.clone(),
            vehicle_class: self.vehicle_class.clone(),
            position: state.position,
            velocity: state.velocity,
            rotation: state.rotation,
            acceleration: state.acceleration,

            // Physical properties
            dimensions: super::VehicleDimensions {
                length: specs.length,
                width: specs.width,
                height: specs.height,
                wheelbase: specs.wheelbase,
                ground_clearance: specs.ground_clearance,
                turning_radius: 5.0,
            },
            mass: specs.weight,
            engine_power: specs.engine_power,
            max_speed: specs.max_speed,
            fuel_capacity: specs.fuel_capacity,
            current_fuel: specs.fuel_capacity * state.fuel_level,
            cargo_capacity: specs.cargo_capacity,
            current_cargo: 0.0,

            // Component systems
            engine,
            transmission,
            suspension,
            brakes,
            tires,
            steering: super::SteeringSystem {
                steering_type: super::SteeringType::ElectricPowerSteering,
                steering_ratio: 16.0,
                power_assist: 1.0,
                steering_angle: state.steering_angle,
                wheel_alignment: super::WheelAlignment {
                    camber: 0.0,
                    caster: 3.0,
                    toe: 0.0,
                },
            },
            electronics,

            // Operational state
            operational_state: super::VehicleState::Idle,
            state,
            stats,
            specs,
            driver: None,
            passengers: Vec::new(),
            cargo: Vec::new(),
            maintenance_level: 1.0,
            damage_level: 0.0,

            // Navigation and routing
            current_route: None,
            destination: None,
            waypoints: Vec::new(),
            navigation_state: super::NavigationState {
                current_waypoint: 0,
                distance_to_destination: 0.0,
                estimated_arrival: 0,
                traffic_conditions: super::TrafficCondition::Clear,
                route_deviations: Vec::new(),
            },

            // AI and behavior
            behavior_profile: super::VehicleBehaviorProfile {
                driving_style: super::DrivingStyle::Normal,
                aggressiveness: 0.5,
                following_distance: 2.0,
                speed_preference: 1.0,
                lane_change_frequency: 0.3,
                risk_tolerance: 0.5,
                fuel_efficiency_priority: 0.7,
                time_priority: 0.6,
            },
            ai_controller: None,
        }
    }
    
    fn build_engine(&self) -> Engine {
        if let Some(ref engine_data) = self.selected_components.engine {
            Engine {
                engine_type: engine_data.engine_type.clone(),
                displacement: engine_data.displacement,
                cylinders: engine_data.cylinders as u32,
                horsepower: engine_data.max_power,
                max_power: engine_data.max_power,
                torque: engine_data.max_torque,
                max_torque: engine_data.max_torque,
                redline_rpm: engine_data.redline_rpm,
                idle_rpm: engine_data.idle_rpm,
                aspiration: engine_data.aspiration.clone(),
                efficiency_map: format!("{:?}", engine_data.efficiency_map), // Convert Vec to String
                thermal_properties: format!("{:?}", engine_data.thermal_properties), // Convert Vec to String
                fuel_efficiency: 25.0,
                emission_level: 0.8,
                temperature: 90.0,
                condition: 1.0,
                fuel_type: FuelType::Custom(engine_data.fuel_type.clone()), // Convert String to FuelType
            }
        } else {
            Engine::default()
        }
    }
    
    fn build_transmission(&self) -> Transmission {
        if let Some(ref trans_data) = self.selected_components.transmission {
            Transmission {
                transmission_type: trans_data.transmission_type.clone(),
                gear_ratios: trans_data.gear_ratios.clone(),
                current_gear: 1,
                final_drive_ratio: trans_data.final_drive_ratio,
                efficiency: trans_data.efficiency,
                shift_time: trans_data.shift_time,
                torque_converter_ratio: trans_data.torque_converter_ratio,
                condition: 1.0,
            }
        } else {
            Transmission::default()
        }
    }
    
    fn build_suspension(&self) -> Suspension {
        Suspension {
            suspension_type: SuspensionType::Independent,
            spring_rate: 25000.0,
            damping_coefficient: 2000.0,
            ground_clearance: 0.15,
            compression: 0.0,
            anti_roll_bar_stiffness: 15000.0,
            ride_height: 0.15,
            travel: 0.2,
            preload: 0.05,
            condition: 1.0,
        }
    }
    
    fn build_brakes(&self) -> Brakes {
        Brakes {
            brake_type: BrakeType::Disc,
            brake_force: 12000.0,
            brake_balance: 0.6,
            abs_enabled: true,
            brake_condition: 1.0,
            brake_temperature: 20.0,
            max_braking_force: 12000.0,
            brake_bias: 0.6,
            brake_assist_enabled: true,
            electronic_brakeforce_distribution: true,
            thermal_capacity: 2000.0,
        }
    }
    
    fn build_tires(&self) -> Tires {
        Tires {
            front_left: super::TireInfo {
                pressure: 32.0,
                wear: 0.0,
                temperature: 20.0,
                grip_coefficient: 1.1,
            },
            front_right: super::TireInfo {
                pressure: 32.0,
                wear: 0.0,
                temperature: 20.0,
                grip_coefficient: 1.1,
            },
            rear_left: super::TireInfo {
                pressure: 32.0,
                wear: 0.0,
                temperature: 20.0,
                grip_coefficient: 1.1,
            },
            rear_right: super::TireInfo {
                pressure: 32.0,
                wear: 0.0,
                temperature: 20.0,
                grip_coefficient: 1.1,
            },
            tire_type: TireType::Performance,
            size: "225/50R18".to_string(),
            pressure: 32.0,
            tread_depth: 8.0,
            compound: "Performance".to_string(),
            max_grip: 1.1,
            rolling_resistance: 0.01,
            wear_rate: 0.001,
            width: 225,
            aspect_ratio: 50,
            diameter: 18,
            grip_coefficient: 1.1,
            optimal_pressure: 32.0,
            load_rating: 1200.0,
        }
    }
    
    fn build_electronics(&self) -> Electronics {
        Electronics {
            stability_control: true,
            traction_control: true,
            anti_lock_brakes: true,
            cruise_control: true,
            adaptive_cruise: false,
            lane_keeping: false,
            collision_avoidance: false,
            parking_assist: false,
            autonomous_level: super::AutonomyLevel::None,
            ecu_version: "2.0".to_string(),
            sensors: vec!["speed".to_string(), "rpm".to_string(), "throttle".to_string()],
            safety_systems: vec!["abs".to_string(), "airbags".to_string()],
            infotainment_features: vec!["radio".to_string(), "bluetooth".to_string()],
            driver_assistance: vec!["cruise_control".to_string()],
            connectivity: vec!["bluetooth".to_string()],
            ecu_enabled: true,
            abs_enabled: true,
            traction_control_enabled: true,
            stability_control_enabled: true,
            launch_control_enabled: false,
            adaptive_suspension_enabled: false,
            torque_vectoring_enabled: false,
        }
    }
    
    fn calculate_total_weight(&self) -> f32 {
        let base_weight = match self.vehicle_type {
            VehicleType::Car => 1500.0,
            VehicleType::Truck => 2500.0,
            VehicleType::Motorcycle => 200.0,
            VehicleType::Aircraft => 1200.0,
            VehicleType::Boat => 2000.0,
            VehicleType::Bus => 8000.0,
            VehicleType::Emergency => 1800.0,
            VehicleType::Military => 5000.0,
            VehicleType::Construction => 15000.0,
            VehicleType::Racing => 1000.0,
            VehicleType::Bicycle => 15.0,
            VehicleType::Train => 20000.0,
            VehicleType::Tram => 25000.0,
            VehicleType::Tank => 35000.0,
            VehicleType::ConstructionVehicle => 12000.0,
            VehicleType::Helicopter => 1500.0,
            VehicleType::Drone => 2.0,
            VehicleType::Balloon => 500.0,
            VehicleType::Glider => 300.0,
            VehicleType::Ship => 50000.0,
            VehicleType::Submarine => 75000.0,
            VehicleType::Hovercraft => 3000.0,
            VehicleType::EmergencyVehicle => 1800.0,
            VehicleType::MilitaryVehicle => 5000.0,
            VehicleType::UtilityVehicle => 2800.0,
            VehicleType::RecreationalVehicle => 3500.0,
            VehicleType::CustomDesign(_) => 1500.0, // Default weight for custom designs
        };
        
        base_weight + self.customizations.additional_weight
    }
}

#[derive(Debug, Clone)]
pub struct VehicleTemplate {
    pub name: String,
    pub vehicle_type: VehicleType,
    pub vehicle_class: VehicleClass,
    pub default_components: ComponentSelection,
    pub default_customizations: DesignCustomizations,
    pub performance_targets: PerformanceTargets,
    pub constraints: DesignConstraints,
}

impl VehicleTemplate {
    pub fn sports_car() -> Self {
        Self {
            name: "Sports Car".to_string(),
            vehicle_type: VehicleType::Car,
            vehicle_class: VehicleClass::Sports,
            default_components: ComponentSelection::sports_car_default(),
            default_customizations: DesignCustomizations::sports_car_default(),
            performance_targets: PerformanceTargets::sports_car(),
            constraints: DesignConstraints::sports_car(),
        }
    }
    
    pub fn sedan() -> Self {
        Self {
            name: "Sedan".to_string(),
            vehicle_type: VehicleType::Car,
            vehicle_class: VehicleClass::Sedan,
            default_components: ComponentSelection::sedan_default(),
            default_customizations: DesignCustomizations::sedan_default(),
            performance_targets: PerformanceTargets::sedan(),
            constraints: DesignConstraints::sedan(),
        }
    }
    
    pub fn suv() -> Self {
        Self {
            name: "SUV".to_string(),
            vehicle_type: VehicleType::Car,
            vehicle_class: VehicleClass::SUV,
            default_components: ComponentSelection::suv_default(),
            default_customizations: DesignCustomizations::suv_default(),
            performance_targets: PerformanceTargets::suv(),
            constraints: DesignConstraints::suv(),
        }
    }
    
    pub fn truck() -> Self {
        Self {
            name: "Truck".to_string(),
            vehicle_type: VehicleType::Truck,
            vehicle_class: VehicleClass::Utility,
            default_components: ComponentSelection::truck_default(),
            default_customizations: DesignCustomizations::truck_default(),
            performance_targets: PerformanceTargets::truck(),
            constraints: DesignConstraints::truck(),
        }
    }
    
    pub fn motorcycle() -> Self {
        Self {
            name: "Motorcycle".to_string(),
            vehicle_type: VehicleType::Motorcycle,
            vehicle_class: VehicleClass::Sports,
            default_components: ComponentSelection::motorcycle_default(),
            default_customizations: DesignCustomizations::motorcycle_default(),
            performance_targets: PerformanceTargets::motorcycle(),
            constraints: DesignConstraints::motorcycle(),
        }
    }
    
    pub fn aircraft() -> Self {
        Self {
            name: "Aircraft".to_string(),
            vehicle_type: VehicleType::Aircraft,
            vehicle_class: VehicleClass::Utility,
            default_components: ComponentSelection::aircraft_default(),
            default_customizations: DesignCustomizations::aircraft_default(),
            performance_targets: PerformanceTargets::aircraft(),
            constraints: DesignConstraints::aircraft(),
        }
    }
    
    pub fn boat() -> Self {
        Self {
            name: "Boat".to_string(),
            vehicle_type: VehicleType::Boat,
            vehicle_class: VehicleClass::Utility,
            default_components: ComponentSelection::boat_default(),
            default_customizations: DesignCustomizations::boat_default(),
            performance_targets: PerformanceTargets::boat(),
            constraints: DesignConstraints::boat(),
        }
    }
    
    pub fn from_design(design: &VehicleDesign) -> Self {
        Self {
            name: format!("{}_template", design.name),
            vehicle_type: design.vehicle_type.clone(),
            vehicle_class: design.vehicle_class.clone(),
            default_components: design.selected_components.clone(),
            default_customizations: design.customizations.clone(),
            performance_targets: design.performance_targets.clone(),
            constraints: design.constraints.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ComponentLibrary {
    engines: HashMap<String, ComponentData>,
    transmissions: HashMap<String, ComponentData>,
    suspensions: HashMap<String, ComponentData>,
    brakes: HashMap<String, ComponentData>,
    tires: HashMap<String, ComponentData>,
    electronics: HashMap<String, ComponentData>,
}

impl ComponentLibrary {
    pub fn new() -> Self {
        let mut library = Self {
            engines: HashMap::new(),
            transmissions: HashMap::new(),
            suspensions: HashMap::new(),
            brakes: HashMap::new(),
            tires: HashMap::new(),
            electronics: HashMap::new(),
        };
        
        library.initialize_default_components();
        library
    }
    
    pub fn get_component(&self, component_type: ComponentType, component_id: &str) -> Option<ComponentData> {
        match component_type {
            ComponentType::Engine => self.engines.get(component_id).cloned(),
            ComponentType::Transmission => self.transmissions.get(component_id).cloned(),
            ComponentType::Suspension => self.suspensions.get(component_id).cloned(),
            ComponentType::Brakes => self.brakes.get(component_id).cloned(),
            ComponentType::Tires => self.tires.get(component_id).cloned(),
            ComponentType::Electronics => self.electronics.get(component_id).cloned(),
        }
    }
    
    pub fn get_component_ids(&self, component_type: ComponentType) -> Vec<String> {
        match component_type {
            ComponentType::Engine => self.engines.keys().cloned().collect(),
            ComponentType::Transmission => self.transmissions.keys().cloned().collect(),
            ComponentType::Suspension => self.suspensions.keys().cloned().collect(),
            ComponentType::Brakes => self.brakes.keys().cloned().collect(),
            ComponentType::Tires => self.tires.keys().cloned().collect(),
            ComponentType::Electronics => self.electronics.keys().cloned().collect(),
        }
    }
    
    fn initialize_default_components(&mut self) {
        self.engines.insert("v8_sports".to_string(), ComponentData::v8_sports_engine());
        self.engines.insert("i4_economy".to_string(), ComponentData::i4_economy_engine());
        self.engines.insert("v6_balanced".to_string(), ComponentData::v6_balanced_engine());
        
        self.transmissions.insert("manual_6speed".to_string(), ComponentData::manual_6speed());
        self.transmissions.insert("auto_8speed".to_string(), ComponentData::auto_8speed());
        self.transmissions.insert("cvt_efficient".to_string(), ComponentData::cvt_efficient());
    }
}

#[derive(Debug, Clone, Default)]
pub struct ComponentSelection {
    pub engine: Option<ComponentData>,
    pub transmission: Option<ComponentData>,
    pub suspension: Option<ComponentData>,
    pub brakes: Option<ComponentData>,
    pub tires: Option<ComponentData>,
    pub electronics: Option<ComponentData>,
}

impl ComponentSelection {
    pub fn sports_car_default() -> Self {
        Self {
            engine: Some(ComponentData::v8_sports_engine()),
            transmission: Some(ComponentData::manual_6speed()),
            suspension: None,
            brakes: None,
            tires: None,
            electronics: None,
        }
    }
    
    pub fn sedan_default() -> Self {
        Self {
            engine: Some(ComponentData::i4_economy_engine()),
            transmission: Some(ComponentData::cvt_efficient()),
            suspension: None,
            brakes: None,
            tires: None,
            electronics: None,
        }
    }
    
    pub fn suv_default() -> Self {
        Self {
            engine: Some(ComponentData::v6_balanced_engine()),
            transmission: Some(ComponentData::auto_8speed()),
            suspension: None,
            brakes: None,
            tires: None,
            electronics: None,
        }
    }
    
    pub fn truck_default() -> Self {
        Self {
            engine: Some(ComponentData::v8_sports_engine()),
            transmission: Some(ComponentData::auto_8speed()),
            suspension: None,
            brakes: None,
            tires: None,
            electronics: None,
        }
    }
    
    pub fn motorcycle_default() -> Self {
        Self {
            engine: Some(ComponentData::i4_economy_engine()),
            transmission: Some(ComponentData::manual_6speed()),
            suspension: None,
            brakes: None,
            tires: None,
            electronics: None,
        }
    }
    
    pub fn aircraft_default() -> Self {
        Self {
            engine: Some(ComponentData::v6_balanced_engine()),
            transmission: None,
            suspension: None,
            brakes: None,
            tires: None,
            electronics: None,
        }
    }
    
    pub fn boat_default() -> Self {
        Self {
            engine: Some(ComponentData::v8_sports_engine()),
            transmission: None,
            suspension: None,
            brakes: None,
            tires: None,
            electronics: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ComponentData {
    pub name: String,
    pub component_type: ComponentType,
    pub engine_type: EngineType,
    pub transmission_type: TransmissionType,
    pub displacement: f32,
    pub cylinders: u8,
    pub max_power: f32,
    pub max_torque: f32,
    pub redline_rpm: f32,
    pub idle_rpm: f32,
    pub fuel_type: String,
    pub aspiration: String,
    pub efficiency_map: Vec<(f32, f32, f32)>,
    pub thermal_properties: Vec<f32>,
    pub gear_ratios: Vec<f32>,
    pub final_drive_ratio: f32,
    pub efficiency: f32,
    pub shift_time: f32,
    pub torque_converter_ratio: f32,
    pub cost: f32,
    pub weight: f32,
    pub reliability_rating: f32,
}

impl ComponentData {
    pub fn v8_sports_engine() -> Self {
        Self {
            name: "V8 Sports Engine".to_string(),
            component_type: ComponentType::Engine,
            engine_type: EngineType::Gasoline,
            transmission_type: TransmissionType::Manual(6), // 6-speed manual
            displacement: 5.0,
            cylinders: 8,
            max_power: 450.0,
            max_torque: 500.0,
            redline_rpm: 7000.0,
            idle_rpm: 800.0,
            fuel_type: "Premium Gasoline".to_string(),
            aspiration: "Naturally Aspirated".to_string(),
            efficiency_map: vec![(0.3, 2000.0, 0.25), (0.8, 4000.0, 0.35)],
            thermal_properties: vec![90.0, 110.0, 130.0],
            gear_ratios: vec![],
            final_drive_ratio: 0.0,
            efficiency: 0.0,
            shift_time: 0.0,
            torque_converter_ratio: 0.0,
            cost: 15000.0,
            weight: 250.0,
            reliability_rating: 8.5,
        }
    }
    
    pub fn i4_economy_engine() -> Self {
        Self {
            name: "I4 Economy Engine".to_string(),
            component_type: ComponentType::Engine,
            engine_type: EngineType::Gasoline,
            transmission_type: TransmissionType::Manual(6), // 6-speed manual
            displacement: 2.0,
            cylinders: 4,
            max_power: 150.0,
            max_torque: 200.0,
            redline_rpm: 6500.0,
            idle_rpm: 700.0,
            fuel_type: "Regular Gasoline".to_string(),
            aspiration: "Naturally Aspirated".to_string(),
            efficiency_map: vec![(0.2, 1500.0, 0.30), (0.6, 3000.0, 0.40)],
            thermal_properties: vec![85.0, 100.0, 120.0],
            gear_ratios: vec![],
            final_drive_ratio: 0.0,
            efficiency: 0.0,
            shift_time: 0.0,
            torque_converter_ratio: 0.0,
            cost: 5000.0,
            weight: 120.0,
            reliability_rating: 9.2,
        }
    }
    
    pub fn v6_balanced_engine() -> Self {
        Self {
            name: "V6 Balanced Engine".to_string(),
            component_type: ComponentType::Engine,
            engine_type: EngineType::Gasoline,
            transmission_type: TransmissionType::Manual(6), // 6-speed manual
            displacement: 3.5,
            cylinders: 6,
            max_power: 280.0,
            max_torque: 320.0,
            redline_rpm: 6800.0,
            idle_rpm: 750.0,
            fuel_type: "Regular Gasoline".to_string(),
            aspiration: "Turbocharged".to_string(),
            efficiency_map: vec![(0.25, 1800.0, 0.28), (0.7, 3500.0, 0.38)],
            thermal_properties: vec![88.0, 105.0, 125.0],
            gear_ratios: vec![],
            final_drive_ratio: 0.0,
            efficiency: 0.0,
            shift_time: 0.0,
            torque_converter_ratio: 0.0,
            cost: 8500.0,
            weight: 180.0,
            reliability_rating: 8.8,
        }
    }
    
    pub fn manual_6speed() -> Self {
        Self {
            name: "6-Speed Manual".to_string(),
            component_type: ComponentType::Transmission,
            engine_type: EngineType::Gasoline,
            transmission_type: TransmissionType::Manual(6), // 6-speed manual
            displacement: 0.0,
            cylinders: 0,
            max_power: 0.0,
            max_torque: 0.0,
            redline_rpm: 0.0,
            idle_rpm: 0.0,
            fuel_type: String::new(),
            aspiration: String::new(),
            efficiency_map: vec![],
            thermal_properties: vec![],
            gear_ratios: vec![3.36, 2.07, 1.43, 1.00, 0.84, 0.56],
            final_drive_ratio: 3.73,
            efficiency: 0.95,
            shift_time: 0.5,
            torque_converter_ratio: 1.0,
            cost: 2500.0,
            weight: 60.0,
            reliability_rating: 9.0,
        }
    }
    
    pub fn auto_8speed() -> Self {
        Self {
            name: "8-Speed Automatic".to_string(),
            component_type: ComponentType::Transmission,
            engine_type: EngineType::Gasoline,
            transmission_type: TransmissionType::Automatic(8), // 8-speed automatic
            displacement: 0.0,
            cylinders: 0,
            max_power: 0.0,
            max_torque: 0.0,
            redline_rpm: 0.0,
            idle_rpm: 0.0,
            fuel_type: String::new(),
            aspiration: String::new(),
            efficiency_map: vec![],
            thermal_properties: vec![],
            gear_ratios: vec![4.70, 3.14, 2.11, 1.67, 1.29, 1.00, 0.84, 0.67],
            final_drive_ratio: 3.31,
            efficiency: 0.92,
            shift_time: 0.2,
            torque_converter_ratio: 2.5,
            cost: 4000.0,
            weight: 85.0,
            reliability_rating: 8.7,
        }
    }
    
    pub fn cvt_efficient() -> Self {
        Self {
            name: "CVT Efficient".to_string(),
            component_type: ComponentType::Transmission,
            engine_type: EngineType::Gasoline,
            transmission_type: TransmissionType::CVT,
            displacement: 0.0,
            cylinders: 0,
            max_power: 0.0,
            max_torque: 0.0,
            redline_rpm: 0.0,
            idle_rpm: 0.0,
            fuel_type: String::new(),
            aspiration: String::new(),
            efficiency_map: vec![],
            thermal_properties: vec![],
            gear_ratios: vec![],
            final_drive_ratio: 1.0,
            efficiency: 0.88,
            shift_time: 0.0,
            torque_converter_ratio: 1.5,
            cost: 3200.0,
            weight: 70.0,
            reliability_rating: 8.2,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DesignCustomizations {
    pub dimensions: VehicleDimensions,
    pub aerodynamics: AerodynamicProperties,
    pub fuel_system: FuelSystemProperties,
    pub additional_weight: f32,
    pub color_scheme: ColorScheme,
    pub interior_options: InteriorOptions,
    pub exterior_styling: ExteriorStyling,
}

impl DesignCustomizations {
    pub fn sports_car_default() -> Self {
        Self {
            dimensions: VehicleDimensions {
                length: 4.2,
                width: 1.8,
                height: 1.3,
                wheelbase: 2.5,
                ground_clearance: 0.12,
            },
            aerodynamics: AerodynamicProperties {
                drag_coefficient: 0.28,
                frontal_area: 2.1,
                downforce_coefficient: 0.15,
            },
            fuel_system: FuelSystemProperties {
                capacity: 60.0,
                fuel_type: "Premium".to_string(),
            },
            additional_weight: 50.0,
            color_scheme: ColorScheme::default(),
            interior_options: InteriorOptions::default(),
            exterior_styling: ExteriorStyling::default(),
        }
    }
    
    pub fn sedan_default() -> Self {
        Self {
            dimensions: VehicleDimensions {
                length: 4.8,
                width: 1.8,
                height: 1.5,
                wheelbase: 2.8,
                ground_clearance: 0.15,
            },
            aerodynamics: AerodynamicProperties {
                drag_coefficient: 0.32,
                frontal_area: 2.3,
                downforce_coefficient: 0.05,
            },
            fuel_system: FuelSystemProperties {
                capacity: 65.0,
                fuel_type: "Regular".to_string(),
            },
            additional_weight: 0.0,
            color_scheme: ColorScheme::default(),
            interior_options: InteriorOptions::default(),
            exterior_styling: ExteriorStyling::default(),
        }
    }
    
    pub fn suv_default() -> Self {
        Self {
            dimensions: VehicleDimensions {
                length: 5.0,
                width: 1.9,
                height: 1.8,
                wheelbase: 2.9,
                ground_clearance: 0.22,
            },
            aerodynamics: AerodynamicProperties {
                drag_coefficient: 0.35,
                frontal_area: 2.8,
                downforce_coefficient: 0.02,
            },
            fuel_system: FuelSystemProperties {
                capacity: 75.0,
                fuel_type: "Regular".to_string(),
            },
            additional_weight: 200.0,
            color_scheme: ColorScheme::default(),
            interior_options: InteriorOptions::default(),
            exterior_styling: ExteriorStyling::default(),
        }
    }
    
    pub fn truck_default() -> Self {
        Self {
            dimensions: VehicleDimensions {
                length: 5.8,
                width: 2.0,
                height: 1.9,
                wheelbase: 3.5,
                ground_clearance: 0.25,
            },
            aerodynamics: AerodynamicProperties {
                drag_coefficient: 0.42,
                frontal_area: 3.2,
                downforce_coefficient: 0.01,
            },
            fuel_system: FuelSystemProperties {
                capacity: 95.0,
                fuel_type: "Regular".to_string(),
            },
            additional_weight: 500.0,
            color_scheme: ColorScheme::default(),
            interior_options: InteriorOptions::default(),
            exterior_styling: ExteriorStyling::default(),
        }
    }
    
    pub fn motorcycle_default() -> Self {
        Self {
            dimensions: VehicleDimensions {
                length: 2.1,
                width: 0.8,
                height: 1.2,
                wheelbase: 1.5,
                ground_clearance: 0.14,
            },
            aerodynamics: AerodynamicProperties {
                drag_coefficient: 0.60,
                frontal_area: 0.5,
                downforce_coefficient: 0.0,
            },
            fuel_system: FuelSystemProperties {
                capacity: 18.0,
                fuel_type: "Regular".to_string(),
            },
            additional_weight: 0.0,
            color_scheme: ColorScheme::default(),
            interior_options: InteriorOptions::default(),
            exterior_styling: ExteriorStyling::default(),
        }
    }
    
    pub fn aircraft_default() -> Self {
        Self {
            dimensions: VehicleDimensions {
                length: 8.5,
                width: 10.0,
                height: 2.8,
                wheelbase: 0.0,
                ground_clearance: 1.5,
            },
            aerodynamics: AerodynamicProperties {
                drag_coefficient: 0.025,
                frontal_area: 1.8,
                downforce_coefficient: -0.5,
            },
            fuel_system: FuelSystemProperties {
                capacity: 200.0,
                fuel_type: "Aviation Fuel".to_string(),
            },
            additional_weight: 100.0,
            color_scheme: ColorScheme::default(),
            interior_options: InteriorOptions::default(),
            exterior_styling: ExteriorStyling::default(),
        }
    }
    
    pub fn boat_default() -> Self {
        Self {
            dimensions: VehicleDimensions {
                length: 7.5,
                width: 2.5,
                height: 2.0,
                wheelbase: 0.0,
                ground_clearance: -0.8,
            },
            aerodynamics: AerodynamicProperties {
                drag_coefficient: 0.15,
                frontal_area: 4.0,
                downforce_coefficient: 0.0,
            },
            fuel_system: FuelSystemProperties {
                capacity: 150.0,
                fuel_type: "Marine Diesel".to_string(),
            },
            additional_weight: 300.0,
            color_scheme: ColorScheme::default(),
            interior_options: InteriorOptions::default(),
            exterior_styling: ExteriorStyling::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CustomizationOptions {
    pub available_engines: Vec<String>,
    pub available_transmissions: Vec<String>,
    pub available_suspensions: Vec<String>,
    pub available_brakes: Vec<String>,
    pub available_tires: Vec<String>,
    pub available_electronics: Vec<String>,
    pub dimension_constraints: DimensionConstraints,
    pub weight_constraints: WeightConstraints,
    pub performance_constraints: PerformanceConstraints,
}

impl CustomizationOptions {
    pub fn new() -> Self {
        Self {
            available_engines: vec![
                "v8_sports".to_string(),
                "i4_economy".to_string(),
                "v6_balanced".to_string(),
            ],
            available_transmissions: vec![
                "manual_6speed".to_string(),
                "auto_8speed".to_string(),
                "cvt_efficient".to_string(),
            ],
            available_suspensions: vec![],
            available_brakes: vec![],
            available_tires: vec![],
            available_electronics: vec![],
            dimension_constraints: DimensionConstraints::default(),
            weight_constraints: WeightConstraints::default(),
            performance_constraints: PerformanceConstraints::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DesignValidator {
    validation_rules: ValidationRules,
    compatibility_matrix: CompatibilityMatrix,
}

impl DesignValidator {
    pub fn new() -> Self {
        Self {
            validation_rules: ValidationRules::new(),
            compatibility_matrix: CompatibilityMatrix::new(),
        }
    }
    
    pub fn validate(&self, design: &VehicleDesign) -> DesignValidationResult {
        let mut result = DesignValidationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            suggestions: Vec::new(),
        };
        
        self.validate_component_compatibility(design, &mut result);
        self.validate_dimensions(design, &mut result);
        self.validate_weight_distribution(design, &mut result);
        self.validate_performance_feasibility(design, &mut result);
        
        result
    }
    
    fn validate_component_compatibility(&self, design: &VehicleDesign, result: &mut DesignValidationResult) {
        if let (Some(engine), Some(transmission)) = (&design.selected_components.engine, &design.selected_components.transmission) {
            if !self.compatibility_matrix.is_compatible(engine, transmission) {
                result.is_valid = false;
                result.errors.push("Engine and transmission are not compatible".to_string());
            }
        }
    }
    
    fn validate_dimensions(&self, design: &VehicleDesign, result: &mut DesignValidationResult) {
        let dims = &design.customizations.dimensions;
        
        if dims.length <= 0.0 || dims.width <= 0.0 || dims.height <= 0.0 {
            result.is_valid = false;
            result.errors.push("Vehicle dimensions must be positive".to_string());
        }
        
        if dims.wheelbase >= dims.length {
            result.is_valid = false;
            result.errors.push("Wheelbase cannot be greater than or equal to vehicle length".to_string());
        }
    }
    
    fn validate_weight_distribution(&self, _design: &VehicleDesign, _result: &mut DesignValidationResult) {
    }
    
    fn validate_performance_feasibility(&self, _design: &VehicleDesign, _result: &mut DesignValidationResult) {
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceCalculator {
    calculation_models: CalculationModels,
}

impl PerformanceCalculator {
    pub fn new() -> Self {
        Self {
            calculation_models: CalculationModels::new(),
        }
    }
    
    pub fn calculate(&self, design: &VehicleDesign) -> PerformanceMetrics {
        let weight = design.calculate_total_weight();
        let power = design.selected_components.engine.as_ref()
            .map(|e| e.max_power)
            .unwrap_or(100.0);
        
        let power_to_weight = power / weight;
        let estimated_0_60 = self.calculate_0_60_time(power_to_weight, weight);
        let estimated_top_speed = self.calculate_top_speed(power, design);
        let estimated_fuel_economy = self.calculate_fuel_economy(design);
        
        PerformanceMetrics {
            estimated_0_60_time: estimated_0_60,
            estimated_top_speed,
            estimated_quarter_mile: estimated_0_60 * 2.2,
            estimated_braking_distance: self.calculate_braking_distance(weight),
            estimated_fuel_economy_city: estimated_fuel_economy.0,
            estimated_fuel_economy_highway: estimated_fuel_economy.1,
            power_to_weight_ratio: power_to_weight,
            weight_distribution_front: 60.0,
            weight_distribution_rear: 40.0,
            estimated_handling_score: self.calculate_handling_score(design),
        }
    }
    
    fn calculate_0_60_time(&self, power_to_weight: f32, weight: f32) -> f32 {
        let base_time = 12.0 / (power_to_weight / 100.0).sqrt();
        let weight_penalty = (weight - 1500.0) / 1000.0 * 0.5;
        (base_time + weight_penalty).max(2.0)
    }
    
    fn calculate_top_speed(&self, power: f32, design: &VehicleDesign) -> f32 {
        let drag_coeff = design.customizations.aerodynamics.drag_coefficient;
        let frontal_area = design.customizations.aerodynamics.frontal_area;
        
        (power / (drag_coeff * frontal_area * 0.5)).sqrt() * 15.0
    }
    
    fn calculate_fuel_economy(&self, design: &VehicleDesign) -> (f32, f32) {
        let base_economy = match design.vehicle_type {
            VehicleType::Car => (28.0, 35.0),
            VehicleType::Truck => (18.0, 24.0),
            VehicleType::Motorcycle => (45.0, 55.0),
            _ => (20.0, 28.0),
        };
        
        let weight_factor = 1.0 - (design.calculate_total_weight() - 1500.0) / 10000.0;
        let aero_factor = 1.0 - (design.customizations.aerodynamics.drag_coefficient - 0.3) / 2.0;
        
        let modifier = weight_factor * aero_factor;
        (base_economy.0 * modifier, base_economy.1 * modifier)
    }
    
    fn calculate_braking_distance(&self, weight: f32) -> f32 {
        35.0 + (weight - 1500.0) / 100.0
    }
    
    fn calculate_handling_score(&self, design: &VehicleDesign) -> f32 {
        let weight_penalty = (design.calculate_total_weight() - 1500.0) / 1000.0;
        let height_penalty = (design.customizations.dimensions.height - 1.4) / 0.5;
        
        (9.0 - weight_penalty - height_penalty).max(1.0).min(10.0)
    }
}

#[derive(Debug, Clone)]
pub struct CostCalculator {
    component_costs: HashMap<ComponentType, f32>,
    manufacturing_costs: ManufacturingCosts,
}

impl CostCalculator {
    pub fn new() -> Self {
        Self {
            component_costs: HashMap::new(),
            manufacturing_costs: ManufacturingCosts::new(),
        }
    }
    
    pub fn calculate(&self, design: &VehicleDesign) -> CostBreakdown {
        let component_cost = self.calculate_component_costs(design);
        let manufacturing_cost = self.calculate_manufacturing_costs(design);
        let development_cost = self.calculate_development_costs(design);
        
        CostBreakdown {
            component_costs: component_cost,
            manufacturing_costs: manufacturing_cost,
            development_costs: development_cost,
            total_cost: component_cost + manufacturing_cost + development_cost,
            estimated_retail_price: (component_cost + manufacturing_cost + development_cost) * 1.8,
        }
    }
    
    fn calculate_component_costs(&self, design: &VehicleDesign) -> f32 {
        let mut total = 0.0;
        
        if let Some(engine) = &design.selected_components.engine {
            total += engine.cost;
        }
        
        if let Some(transmission) = &design.selected_components.transmission {
            total += transmission.cost;
        }
        
        total += 5000.0;
        total
    }
    
    fn calculate_manufacturing_costs(&self, _design: &VehicleDesign) -> f32 {
        8000.0
    }
    
    fn calculate_development_costs(&self, _design: &VehicleDesign) -> f32 {
        12000.0
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ComponentType {
    Engine,
    Transmission,
    Suspension,
    Brakes,
    Tires,
    Electronics,
}

#[derive(Debug, Clone, Default)]
pub struct VehicleDimensions {
    pub length: f32,
    pub width: f32,
    pub height: f32,
    pub wheelbase: f32,
    pub ground_clearance: f32,
}

#[derive(Debug, Clone, Default)]
pub struct AerodynamicProperties {
    pub drag_coefficient: f32,
    pub frontal_area: f32,
    pub downforce_coefficient: f32,
}

#[derive(Debug, Clone, Default)]
pub struct FuelSystemProperties {
    pub capacity: f32,
    pub fuel_type: String,
}

#[derive(Debug, Clone)]
pub struct ColorScheme {
    pub primary_color: String,
    pub secondary_color: String,
    pub accent_color: String,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            primary_color: "Black".to_string(),
            secondary_color: "Gray".to_string(),
            accent_color: "Red".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct InteriorOptions {
    pub seat_material: String,
    pub trim_level: String,
    pub infotainment_system: String,
}

impl Default for InteriorOptions {
    fn default() -> Self {
        Self {
            seat_material: "Fabric".to_string(),
            trim_level: "Base".to_string(),
            infotainment_system: "Standard".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExteriorStyling {
    pub body_kit: String,
    pub wheel_style: String,
    pub lighting_package: String,
}

impl Default for ExteriorStyling {
    fn default() -> Self {
        Self {
            body_kit: "Standard".to_string(),
            wheel_style: "Alloy".to_string(),
            lighting_package: "Halogen".to_string(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PerformanceTargets {
    pub target_0_60_time: f32,
    pub target_top_speed: f32,
    pub target_fuel_economy: f32,
    pub target_handling_score: f32,
}

impl PerformanceTargets {
    pub fn sports_car() -> Self {
        Self {
            target_0_60_time: 4.5,
            target_top_speed: 280.0,
            target_fuel_economy: 25.0,
            target_handling_score: 9.0,
        }
    }
    
    pub fn sedan() -> Self {
        Self {
            target_0_60_time: 8.0,
            target_top_speed: 200.0,
            target_fuel_economy: 35.0,
            target_handling_score: 7.0,
        }
    }
    
    pub fn suv() -> Self {
        Self {
            target_0_60_time: 9.0,
            target_top_speed: 180.0,
            target_fuel_economy: 28.0,
            target_handling_score: 6.0,
        }
    }
    
    pub fn truck() -> Self {
        Self {
            target_0_60_time: 10.0,
            target_top_speed: 160.0,
            target_fuel_economy: 20.0,
            target_handling_score: 5.0,
        }
    }
    
    pub fn motorcycle() -> Self {
        Self {
            target_0_60_time: 3.5,
            target_top_speed: 320.0,
            target_fuel_economy: 50.0,
            target_handling_score: 8.5,
        }
    }
    
    pub fn aircraft() -> Self {
        Self {
            target_0_60_time: 0.0,
            target_top_speed: 500.0,
            target_fuel_economy: 15.0,
            target_handling_score: 7.5,
        }
    }
    
    pub fn boat() -> Self {
        Self {
            target_0_60_time: 0.0,
            target_top_speed: 120.0,
            target_fuel_economy: 8.0,
            target_handling_score: 6.5,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DesignConstraints {
    pub max_weight: f32,
    pub max_dimensions: VehicleDimensions,
    pub min_fuel_economy: f32,
    pub max_cost: f32,
    pub required_features: Vec<String>,
}

impl DesignConstraints {
    pub fn sports_car() -> Self {
        Self {
            max_weight: 1800.0,
            max_dimensions: VehicleDimensions {
                length: 4.5,
                width: 2.0,
                height: 1.4,
                wheelbase: 2.8,
                ground_clearance: 0.15,
            },
            min_fuel_economy: 20.0,
            max_cost: 80000.0,
            required_features: vec!["ABS".to_string(), "Stability Control".to_string()],
        }
    }
    
    pub fn sedan() -> Self {
        Self {
            max_weight: 2000.0,
            max_dimensions: VehicleDimensions {
                length: 5.2,
                width: 1.9,
                height: 1.6,
                wheelbase: 3.0,
                ground_clearance: 0.18,
            },
            min_fuel_economy: 30.0,
            max_cost: 50000.0,
            required_features: vec!["ABS".to_string(), "Airbags".to_string()],
        }
    }
    
    pub fn suv() -> Self {
        Self {
            max_weight: 2500.0,
            max_dimensions: VehicleDimensions {
                length: 5.5,
                width: 2.0,
                height: 2.0,
                wheelbase: 3.2,
                ground_clearance: 0.25,
            },
            min_fuel_economy: 25.0,
            max_cost: 60000.0,
            required_features: vec!["ABS".to_string(), "AWD".to_string()],
        }
    }
    
    pub fn truck() -> Self {
        Self {
            max_weight: 3500.0,
            max_dimensions: VehicleDimensions {
                length: 6.5,
                width: 2.2,
                height: 2.2,
                wheelbase: 4.0,
                ground_clearance: 0.3,
            },
            min_fuel_economy: 18.0,
            max_cost: 70000.0,
            required_features: vec!["Tow Package".to_string(), "4WD".to_string()],
        }
    }
    
    pub fn motorcycle() -> Self {
        Self {
            max_weight: 300.0,
            max_dimensions: VehicleDimensions {
                length: 2.5,
                width: 1.0,
                height: 1.4,
                wheelbase: 1.8,
                ground_clearance: 0.16,
            },
            min_fuel_economy: 40.0,
            max_cost: 25000.0,
            required_features: vec!["ABS".to_string()],
        }
    }
    
    pub fn aircraft() -> Self {
        Self {
            max_weight: 2000.0,
            max_dimensions: VehicleDimensions {
                length: 10.0,
                width: 12.0,
                height: 3.5,
                wheelbase: 0.0,
                ground_clearance: 2.0,
            },
            min_fuel_economy: 12.0,
            max_cost: 200000.0,
            required_features: vec!["GPS".to_string(), "Radio".to_string()],
        }
    }
    
    pub fn boat() -> Self {
        Self {
            max_weight: 3000.0,
            max_dimensions: VehicleDimensions {
                length: 10.0,
                width: 3.0,
                height: 2.5,
                wheelbase: 0.0,
                ground_clearance: -1.0,
            },
            min_fuel_economy: 5.0,
            max_cost: 100000.0,
            required_features: vec!["GPS".to_string(), "Safety Equipment".to_string()],
        }
    }
}

#[derive(Debug, Clone)]
pub struct DimensionConstraints {
    pub min_length: f32,
    pub max_length: f32,
    pub min_width: f32,
    pub max_width: f32,
    pub min_height: f32,
    pub max_height: f32,
}

impl Default for DimensionConstraints {
    fn default() -> Self {
        Self {
            min_length: 2.0,
            max_length: 10.0,
            min_width: 0.5,
            max_width: 3.0,
            min_height: 1.0,
            max_height: 3.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WeightConstraints {
    pub min_weight: f32,
    pub max_weight: f32,
}

impl Default for WeightConstraints {
    fn default() -> Self {
        Self {
            min_weight: 100.0,
            max_weight: 10000.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceConstraints {
    pub min_power: f32,
    pub max_power: f32,
    pub min_efficiency: f32,
    pub max_emissions: f32,
}

impl Default for PerformanceConstraints {
    fn default() -> Self {
        Self {
            min_power: 50.0,
            max_power: 1000.0,
            min_efficiency: 0.15,
            max_emissions: 200.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValidationRules {
    pub component_compatibility: Vec<CompatibilityRule>,
    pub dimension_limits: DimensionConstraints,
    pub weight_limits: WeightConstraints,
    pub performance_requirements: PerformanceConstraints,
}

impl ValidationRules {
    pub fn new() -> Self {
        Self {
            component_compatibility: Vec::new(),
            dimension_limits: DimensionConstraints::default(),
            weight_limits: WeightConstraints::default(),
            performance_requirements: PerformanceConstraints::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompatibilityRule {
    pub component_a: ComponentType,
    pub component_b: ComponentType,
    pub compatibility: bool,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct CompatibilityMatrix {
    pub rules: HashMap<(ComponentType, ComponentType), bool>,
}

impl CompatibilityMatrix {
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }
    
    pub fn is_compatible(&self, _component_a: &ComponentData, _component_b: &ComponentData) -> bool {
        true
    }
}

#[derive(Debug, Clone)]
pub struct DesignValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub estimated_0_60_time: f32,
    pub estimated_top_speed: f32,
    pub estimated_quarter_mile: f32,
    pub estimated_braking_distance: f32,
    pub estimated_fuel_economy_city: f32,
    pub estimated_fuel_economy_highway: f32,
    pub power_to_weight_ratio: f32,
    pub weight_distribution_front: f32,
    pub weight_distribution_rear: f32,
    pub estimated_handling_score: f32,
}

#[derive(Debug, Clone)]
pub struct CostBreakdown {
    pub component_costs: f32,
    pub manufacturing_costs: f32,
    pub development_costs: f32,
    pub total_cost: f32,
    pub estimated_retail_price: f32,
}

#[derive(Debug, Clone)]
pub struct CalculationModels {
    pub acceleration_model: AccelerationModel,
    pub top_speed_model: TopSpeedModel,
    pub fuel_economy_model: FuelEconomyModel,
    pub handling_model: HandlingModel,
}

impl CalculationModels {
    pub fn new() -> Self {
        Self {
            acceleration_model: AccelerationModel::new(),
            top_speed_model: TopSpeedModel::new(),
            fuel_economy_model: FuelEconomyModel::new(),
            handling_model: HandlingModel::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AccelerationModel {
    pub base_coefficients: Vec<f32>,
    pub weight_factor: f32,
    pub power_factor: f32,
    pub drivetrain_modifiers: HashMap<DrivetrainType, f32>,
}

impl AccelerationModel {
    pub fn new() -> Self {
        Self {
            base_coefficients: vec![1.0, 0.5, 0.2],
            weight_factor: 0.8,
            power_factor: 1.2,
            drivetrain_modifiers: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TopSpeedModel {
    pub drag_coefficients: Vec<f32>,
    pub power_scaling: f32,
    pub aerodynamic_efficiency: f32,
}

impl TopSpeedModel {
    pub fn new() -> Self {
        Self {
            drag_coefficients: vec![1.0, 0.8, 0.6],
            power_scaling: 1.1,
            aerodynamic_efficiency: 0.9,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FuelEconomyModel {
    pub base_consumption: f32,
    pub weight_penalty: f32,
    pub aerodynamic_bonus: f32,
    pub engine_efficiency_map: HashMap<EngineType, f32>,
}

impl FuelEconomyModel {
    pub fn new() -> Self {
        Self {
            base_consumption: 8.0,
            weight_penalty: 0.002,
            aerodynamic_bonus: 0.1,
            engine_efficiency_map: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HandlingModel {
    pub weight_distribution_factor: f32,
    pub center_of_gravity_factor: f32,
    pub suspension_stiffness_factor: f32,
    pub tire_grip_factor: f32,
}

impl HandlingModel {
    pub fn new() -> Self {
        Self {
            weight_distribution_factor: 0.3,
            center_of_gravity_factor: 0.25,
            suspension_stiffness_factor: 0.2,
            tire_grip_factor: 0.25,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ManufacturingCosts {
    pub base_cost_per_unit: f32,
    pub complexity_multipliers: HashMap<VehicleType, f32>,
    pub material_costs: HashMap<String, f32>,
    pub labor_costs: HashMap<String, f32>,
}

impl ManufacturingCosts {
    pub fn new() -> Self {
        Self {
            base_cost_per_unit: 5000.0,
            complexity_multipliers: HashMap::new(),
            material_costs: HashMap::new(),
            labor_costs: HashMap::new(),
        }
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            engine_type: EngineType::Gasoline,
            displacement: 2.0,
            cylinders: 4,
            horsepower: 150.0,
            max_power: 150.0,
            torque: 200.0,
            max_torque: 200.0,
            redline_rpm: 6500.0,
            idle_rpm: 700.0,
            aspiration: "Naturally Aspirated".to_string(),
            efficiency_map: format!("{:?}", vec![(0.2, 1500.0, 0.30)]), // Convert Vec to String
            thermal_properties: format!("{:?}", vec![85.0, 100.0, 120.0]), // Convert Vec to String
            fuel_efficiency: 25.0,
            emission_level: 0.8,
            temperature: 90.0,
            condition: 1.0,
            fuel_type: FuelType::Custom("Regular Gasoline".to_string()),
        }
    }
}

impl Default for Transmission {
    fn default() -> Self {
        Self {
            transmission_type: TransmissionType::Automatic(8), // 8-speed automatic
            gear_ratios: vec![3.36, 2.07, 1.43, 1.00, 0.84, 0.56],
            current_gear: 1,
            final_drive_ratio: 3.73,
            shift_time: 0.3,
            torque_converter_ratio: 2.0,
            efficiency: 0.95,
            condition: 1.0,
        }
    }
}