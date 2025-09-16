use crate::engine::math::{Vec3, Point3, InnerSpace};
use std::collections::HashMap;

pub mod vehicle_controller;
pub mod transportation_network;
pub mod vehicle_designer;
pub mod route_planner;
pub mod traffic_manager;

pub use vehicle_controller::{VehicleController, TireType};
pub use transportation_network::TransportationNetwork;
pub use vehicle_designer::VehicleDesigner;
pub use route_planner::RoutePlanner;
pub use traffic_manager::{TrafficManager, TrafficAnalysisResult};

// Note: Point3 and Vec3 should be imported from crate::engine::math by submodules

// Type aliases for compatibility with submodules
pub type Brakes = BrakeSystem;
pub type Tires = TireSystem;
pub type Electronics = ElectronicsSystem;

// Missing type definitions
#[derive(Debug, Clone)]
pub enum DrivetrainType {
    FrontWheelDrive,
    RearWheelDrive,
    AllWheelDrive,
    FourWheelDrive,
}

#[derive(Debug, Clone)]
pub enum VehicleClass {
    Compact,
    Sedan,
    SUV,
    Truck,
    Motorcycle,
    Bus,
    Van,
    Sports,
    Luxury,
    Utility,
}

#[derive(Debug, Clone)]
pub struct VehicleSpecs {
    pub engine_power: f32,
    pub max_speed: f32,
    pub acceleration: f32,
    pub fuel_efficiency: f32,
    pub cargo_capacity: f32,
    pub passenger_capacity: u32,
    pub seating_capacity: u32,
    pub length: f32,
    pub width: f32,
    pub height: f32,
    pub weight: f32,
    pub wheelbase: f32,
    pub ground_clearance: f32,
    pub drag_coefficient: f32,
    pub frontal_area: f32,
    pub fuel_capacity: f32,
}

#[derive(Debug, Clone)]
pub struct VehicleStats {
    pub total_distance: f32,
    pub fuel_consumed: f32,
    pub average_speed: f32,
    pub max_speed: f32,
    pub top_speed: f32, // Alias for max_speed for compatibility
    pub acceleration_0_60: f32,
    pub braking_60_0: f32,
    pub fuel_economy_city: f32,
    pub fuel_economy_highway: f32,
    pub fuel_efficiency: f32,
    pub power_to_weight_ratio: f32,
    pub handling_rating: f32,
    pub comfort_rating: f32,
    pub reliability_rating: f32,
    pub maintenance_score: f32,
    pub performance_rating: f32,
}

#[derive(Debug, Clone)]
pub struct TireSystem {
    pub front_left: TireInfo,
    pub front_right: TireInfo,
    pub rear_left: TireInfo,
    pub rear_right: TireInfo,
    // System-wide tire properties
    pub tire_type: TireType,
    pub size: String,
    pub pressure: f32,
    pub tread_depth: f32,
    pub compound: String,
    pub max_grip: f32,
    pub rolling_resistance: f32,
    pub wear_rate: f32,
    pub width: u32,
    pub aspect_ratio: u32,
    pub diameter: u32,
    pub grip_coefficient: f32,
    pub optimal_pressure: f32,
    pub load_rating: f32,
}

#[derive(Debug, Clone)]
pub struct TireInfo {
    pub pressure: f32,
    pub wear: f32,
    pub temperature: f32,
    pub grip_coefficient: f32,
}

#[derive(Debug, Clone)]
pub struct TransportNode {
    pub id: String,
    pub position: Point3,
    pub node_type: NodeType,
    pub connections: Vec<String>,
    pub capacity: u32,
}

// NodeType is defined later in the file

#[derive(Debug)]
pub struct VehicleSystem {
    pub vehicle_controller: VehicleController,
    pub transportation_network: TransportationNetwork,
    pub vehicle_designer: VehicleDesigner,
    pub route_planner: RoutePlanner,
    pub traffic_manager: TrafficManager,
    
    // Active vehicles and infrastructure
    pub active_vehicles: HashMap<String, Vehicle>,
    pub infrastructure: HashMap<String, TransportInfrastructure>,
    pub route_network: RouteNetwork,
    
    // System configuration
    pub physics_settings: VehiclePhysicsSettings,
    pub traffic_settings: TrafficSettings,
}

#[derive(Debug, Clone)]
pub struct Vehicle {
    pub id: String,
    pub name: String,
    pub vehicle_type: VehicleType,
    pub vehicle_class: VehicleClass,
    pub position: Point3,
    pub velocity: Vec3,
    pub rotation: Vec3,
    pub acceleration: Vec3,
    
    // Physical properties
    pub dimensions: VehicleDimensions,
    pub mass: f32,
    pub engine_power: f32,
    pub max_speed: f32,
    pub fuel_capacity: f32,
    pub current_fuel: f32,
    pub cargo_capacity: f32,
    pub current_cargo: f32,
    
    // Component systems
    pub engine: Engine,
    pub transmission: Transmission,
    pub suspension: Suspension,
    pub brakes: BrakeSystem,
    pub tires: TireSystem,
    pub steering: SteeringSystem,
    pub electronics: ElectronicsSystem,
    
    // Operational state
    pub operational_state: VehicleState,
    pub state: VehiclePhysicalState,
    pub stats: VehicleStats,
    pub specs: VehicleSpecs,
    pub driver: Option<String>, // NPC or Player ID
    pub passengers: Vec<String>,
    pub cargo: Vec<CargoItem>,
    pub maintenance_level: f32,
    pub damage_level: f32,
    
    // Navigation and routing
    pub current_route: Option<String>,
    pub destination: Option<Point3>,
    pub waypoints: Vec<Point3>,
    pub navigation_state: NavigationState,
    
    // AI and behavior
    pub behavior_profile: VehicleBehaviorProfile,
    pub ai_controller: Option<VehicleAI>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum VehicleType {
    // Ground vehicles
    Car,
    Truck,
    Bus,
    Motorcycle,
    Bicycle,
    Train,
    Tram,
    Tank,
    ConstructionVehicle,
    
    // Aerial vehicles
    Aircraft,
    Helicopter,
    Drone,
    Balloon,
    Glider,
    
    // Naval vehicles
    Boat,
    Ship,
    Submarine,
    Hovercraft,
    
    // Specialized vehicles
    EmergencyVehicle,
    Emergency, // Alias for EmergencyVehicle
    MilitaryVehicle,
    Military, // Alias for MilitaryVehicle
    UtilityVehicle,
    RecreationalVehicle,
    Construction,
    Racing,
    
    // Custom vehicles
    CustomDesign(String),
}

#[derive(Debug, Clone)]
pub struct VehicleDimensions {
    pub length: f32,
    pub width: f32,
    pub height: f32,
    pub wheelbase: f32,
    pub ground_clearance: f32,
    pub turning_radius: f32,
}

#[derive(Debug, Clone)]
pub struct Engine {
    pub engine_type: EngineType,
    pub displacement: f32,
    pub cylinders: u32,
    pub horsepower: f32,
    pub max_power: f32,
    pub torque: f32,
    pub max_torque: f32,
    pub redline_rpm: f32,
    pub idle_rpm: f32,
    pub aspiration: String,
    pub efficiency_map: String,
    pub thermal_properties: String,
    pub fuel_efficiency: f32,
    pub emission_level: f32,
    pub temperature: f32,
    pub condition: f32,
    pub fuel_type: FuelType,
}

#[derive(Debug, Clone)]
pub enum EngineType {
    Internal(InternalCombustionType),
    Electric(ElectricMotorType),
    Hybrid(HybridConfiguration),
    Steam,
    Jet,
    Rocket,
    Nuclear,
    Gasoline, // Convenience variant for common usage
    CustomPower(String),
}

#[derive(Debug, Clone)]
pub enum InternalCombustionType {
    Gasoline,
    Diesel,
    NaturalGas,
    Ethanol,
    Hydrogen,
}

#[derive(Debug, Clone)]
pub enum ElectricMotorType {
    DC,
    AC,
    Brushless,
    Stepper,
}

#[derive(Debug, Clone)]
pub enum HybridConfiguration {
    Series,
    Parallel,
    SeriesParallel,
}

#[derive(Debug, Clone)]
pub enum FuelType {
    Gasoline,
    Diesel,
    Electric,
    Hydrogen,
    Biofuel,
    Nuclear,
    Solar,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct Transmission {
    pub transmission_type: TransmissionType,
    pub gear_ratios: Vec<f32>,
    pub current_gear: u8,
    pub final_drive_ratio: f32,
    pub shift_time: f32,
    pub torque_converter_ratio: f32,
    pub efficiency: f32,
    pub condition: f32,
}

#[derive(Debug, Clone)]
pub enum TransmissionType {
    Manual(u8), // Number of gears
    Automatic(u8),
    CVT,
    DCT,
    Direct, // For electric vehicles
}

#[derive(Debug, Clone)]
pub struct Suspension {
    pub suspension_type: SuspensionType,
    pub spring_rate: f32,
    pub damping_coefficient: f32,
    pub ground_clearance: f32,
    pub compression: f32,
    pub anti_roll_bar_stiffness: f32,
    pub ride_height: f32,
    pub travel: f32,
    pub preload: f32,
    pub condition: f32,
}

#[derive(Debug, Clone)]
pub enum SuspensionType {
    Independent,
    Dependent,
    Active,
    Adaptive,
    AirSuspension,
    MagneticSuspension,
}

#[derive(Debug, Clone)]
pub struct BrakeSystem {
    pub brake_type: BrakeType,
    pub brake_force: f32,
    pub brake_balance: f32,
    pub abs_enabled: bool,
    pub brake_condition: f32,
    pub brake_temperature: f32,
    pub max_braking_force: f32,
    pub brake_bias: f32,
    pub brake_assist_enabled: bool,
    pub electronic_brakeforce_distribution: bool,
    pub thermal_capacity: f32,
}

#[derive(Debug, Clone)]
pub enum BrakeType {
    Disc,
    Drum,
    Regenerative,
    AirBrake,
    HydraulicBrake,
}

#[derive(Debug, Clone)]
pub struct SteeringSystem {
    pub steering_type: SteeringType,
    pub steering_ratio: f32,
    pub power_assist: f32,
    pub steering_angle: f32,
    pub wheel_alignment: WheelAlignment,
}

#[derive(Debug, Clone)]
pub enum SteeringType {
    Mechanical,
    HydraulicPowerSteering,
    ElectricPowerSteering,
    SteerByWire,
    FourWheelSteering,
}

#[derive(Debug, Clone)]
pub struct WheelAlignment {
    pub camber: f32,
    pub caster: f32,
    pub toe: f32,
}

#[derive(Debug, Clone)]
pub struct ElectronicsSystem {
    pub stability_control: bool,
    pub traction_control: bool,
    pub anti_lock_brakes: bool,
    pub cruise_control: bool,
    pub adaptive_cruise: bool,
    pub lane_keeping: bool,
    pub collision_avoidance: bool,
    pub parking_assist: bool,
    pub autonomous_level: AutonomyLevel,
    // Additional fields for compatibility
    pub ecu_version: String,
    pub sensors: Vec<String>,
    pub safety_systems: Vec<String>,
    pub infotainment_features: Vec<String>,
    pub driver_assistance: Vec<String>,
    pub connectivity: Vec<String>,
    pub ecu_enabled: bool,
    pub abs_enabled: bool,
    pub traction_control_enabled: bool,
    pub stability_control_enabled: bool,
    pub launch_control_enabled: bool,
    pub adaptive_suspension_enabled: bool,
    pub torque_vectoring_enabled: bool,
}

#[derive(Debug, Clone)]
pub enum AutonomyLevel {
    None,        // Level 0: No automation
    Assisted,    // Level 1: Driver assistance
    Partial,     // Level 2: Partial automation
    Conditional, // Level 3: Conditional automation
    High,        // Level 4: High automation
    Full,        // Level 5: Full automation
}

#[derive(Debug, Clone)]
pub enum VehicleState {
    Parked,
    Idle,
    Accelerating,
    Cruising,
    Decelerating,
    Turning,
    Reversing,
    Emergency,
    Maintenance,
    Refueling,
    Loading,
    Unloading,
    Broken,
}

#[derive(Debug, Clone)]
pub struct VehiclePhysicalState {
    pub position: Point3,
    pub rotation: Vec3,
    pub velocity: Vec3,
    pub acceleration: Vec3,
    pub angular_velocity: Vec3,
    pub engine_rpm: f32,
    pub gear: i32,
    pub throttle_position: f32,
    pub brake_pressure: f32,
    pub steering_angle: f32,
    pub fuel_level: f32,
    pub speed: f32,
    pub is_engine_running: bool,
}

#[derive(Debug, Clone)]
pub struct CargoItem {
    pub id: String,
    pub name: String,
    pub weight: f32,
    pub volume: f32,
    pub cargo_type: CargoType,
    pub destination: Option<Point3>,
    pub priority: f32,
    pub special_requirements: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum CargoType {
    General,
    Fragile,
    Hazardous,
    Perishable,
    Livestock,
    Liquid,
    Bulk,
    Container,
    Passengers,
    Mail,
}

#[derive(Debug, Clone)]
pub struct NavigationState {
    pub current_waypoint: usize,
    pub distance_to_destination: f32,
    pub estimated_arrival: u64,
    pub traffic_conditions: TrafficCondition,
    pub route_deviations: Vec<RouteDeviation>,
}

#[derive(Debug, Clone)]
pub enum TrafficCondition {
    Clear,
    Light,
    Moderate,
    Heavy,
    Stopped,
    Blocked,
}

#[derive(Debug, Clone)]
pub struct RouteDeviation {
    pub deviation_type: DeviationType,
    pub location: Point3,
    pub severity: f32,
    pub estimated_delay: u64,
}

#[derive(Debug, Clone)]
pub enum DeviationType {
    Construction,
    Accident,
    Weather,
    RoadClosure,
    Traffic,
    Detour,
    Breakdown,
}

#[derive(Debug, Clone)]
pub struct VehicleBehaviorProfile {
    pub driving_style: DrivingStyle,
    pub aggressiveness: f32,
    pub following_distance: f32,
    pub speed_preference: f32, // Ratio of speed limit
    pub lane_change_frequency: f32,
    pub risk_tolerance: f32,
    pub fuel_efficiency_priority: f32,
    pub time_priority: f32,
}

#[derive(Debug, Clone)]
pub enum DrivingStyle {
    Conservative,
    Normal,
    Aggressive,
    Sporty,
    Economic,
    Defensive,
    Racing,
}

#[derive(Debug, Clone)]
pub struct VehicleAI {
    pub ai_type: AIType,
    pub decision_tree: DecisionTree,
    pub learning_enabled: bool,
    pub experience_level: f32,
    pub local_knowledge: LocalKnowledge,
    pub communication_range: f32,
}

#[derive(Debug, Clone)]
pub enum AIType {
    Basic,
    Advanced,
    LearningAI,
    SwarmIntelligence,
    NeuralNetwork,
    Fuzzy,
}

#[derive(Debug, Clone)]
pub struct DecisionTree {
    pub root_node: DecisionNode,
    pub decision_history: Vec<DecisionRecord>,
}

#[derive(Debug, Clone)]
pub struct DecisionNode {
    pub condition: String,
    pub true_branch: Option<Box<DecisionNode>>,
    pub false_branch: Option<Box<DecisionNode>>,
    pub action: Option<VehicleAction>,
}

#[derive(Debug, Clone)]
pub struct DecisionRecord {
    pub timestamp: u64,
    pub situation: String,
    pub decision: String,
    pub outcome: String,
    pub success_rating: f32,
}

#[derive(Debug, Clone)]
pub enum VehicleAction {
    Accelerate(f32),
    Brake(f32),
    Steer(f32),
    ChangeGear(u8),
    ChangeLane(LaneDirection),
    Signal(SignalType),
    Stop,
    Park,
    Yield,
    Merge,
    Overtake,
    Follow(String), // Follow vehicle ID
}

// Data structure for vehicle decision making to avoid borrowing conflicts
#[derive(Debug, Clone)]
pub struct VehicleDecisionData {
    pub position: Point3,
    pub velocity: Vec3,
    pub acceleration: Vec3,
    pub operational_state: VehicleState,
    pub current_route: Option<String>,
    pub destination: Option<Point3>,
    pub navigation_state: NavigationState,
}

impl VehicleAI {
    pub fn make_decision(&mut self, _vehicle: &Vehicle, _route_network: &RouteNetwork, _delta_time: f32) -> VehicleAction {
        // Placeholder implementation for AI decision making
        // In a real implementation, this would analyze the vehicle's current state,
        // surrounding traffic, route conditions, and make appropriate driving decisions
        VehicleAction::Accelerate(0.5)
    }

    pub fn make_decision_with_data(&mut self, _vehicle_data: &VehicleDecisionData, _route_network: &RouteNetwork, _delta_time: f32) -> VehicleAction {
        // Placeholder implementation using vehicle data instead of direct vehicle reference
        // This avoids borrowing conflicts by working with copied/cloned data
        VehicleAction::Accelerate(0.5)
    }
}

#[derive(Debug, Clone)]
pub enum LaneDirection {
    Left,
    Right,
    Center,
}

#[derive(Debug, Clone)]
pub enum SignalType {
    LeftTurn,
    RightTurn,
    Hazard,
    Brake,
    Reverse,
    Emergency,
}

#[derive(Debug, Clone)]
pub struct LocalKnowledge {
    pub known_routes: HashMap<String, RouteKnowledge>,
    pub traffic_patterns: Vec<TrafficPattern>,
    pub hazard_locations: Vec<HazardLocation>,
    pub preferred_routes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RouteKnowledge {
    pub route_id: String,
    pub average_travel_time: f32,
    pub traffic_history: Vec<TrafficHistoryEntry>,
    pub road_conditions: RoadConditions,
    pub safety_rating: f32,
}

#[derive(Debug, Clone)]
pub struct TrafficHistoryEntry {
    pub timestamp: u64,
    pub traffic_level: f32,
    pub travel_time: f32,
    pub incidents: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TrafficPattern {
    pub pattern_id: String,
    pub time_pattern: TimePattern,
    pub location: Point3,
    pub traffic_intensity: f32,
    pub duration: u64,
}

#[derive(Debug, Clone)]
pub enum TimePattern {
    Daily(u32), // Hour of day
    Weekly(u8), // Day of week
    Monthly(u8), // Day of month
    Seasonal(Season),
    Event(String),
}

#[derive(Debug, Clone)]
pub enum Season {
    Spring,
    Summer,
    Fall,
    Winter,
}

#[derive(Debug, Clone)]
pub struct HazardLocation {
    pub location: Point3,
    pub hazard_type: HazardType,
    pub severity: f32,
    pub permanent: bool,
    pub time_restrictions: Vec<TimeRestriction>,
}

#[derive(Debug, Clone)]
pub enum HazardType {
    Construction,
    PoorRoadCondition,
    SharpTurn,
    SteepGrade,
    WeatherSensitive,
    AccidentProne,
    Flooding,
    Wildlife,
}

#[derive(Debug, Clone)]
pub struct TimeRestriction {
    pub start_time: u32,
    pub end_time: u32,
    pub days_of_week: Vec<u8>,
    pub restriction_type: RestrictionType,
}

#[derive(Debug, Clone)]
pub enum RestrictionType {
    SpeedLimit(f32),
    NoParking,
    NoStopping,
    OneWay,
    Closed,
    PermitRequired,
    LoadRestriction(f32),
}

#[derive(Debug, Clone)]
pub struct TransportInfrastructure {
    pub id: String,
    pub name: String,
    pub infrastructure_type: InfrastructureType,
    pub location: Point3,
    pub capacity: f32,
    pub current_utilization: f32,
    pub condition: f32,
    pub maintenance_schedule: MaintenanceSchedule,
    pub connected_infrastructure: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum InfrastructureType {
    // Roads and pathways
    Road(RoadType),
    Bridge,
    Tunnel,
    Intersection,
    Roundabout,
    Highway,
    
    // Rail infrastructure
    RailTrack,
    RailStation,
    RailYard,
    RailBridge,
    RailTunnel,
    
    // Air infrastructure
    Airport,
    Runway,
    Helipad,
    AirTrafficControl,
    
    // Naval infrastructure
    Port,
    Marina,
    Dock,
    Canal,
    Lock,
    
    // Supporting infrastructure
    ParkingLot,
    GasStation,
    ChargingStation,
    MaintenanceFacility,
    TrafficLight,
    RoadSign,
    Barrier,
}

#[derive(Debug, Clone)]
pub enum RoadType {
    Local,
    Arterial,
    Collector,
    Highway,
    Interstate,
    Residential,
    Commercial,
    Industrial,
    Rural,
    Urban,
}

#[derive(Debug, Clone)]
pub struct MaintenanceSchedule {
    pub routine_maintenance: Vec<MaintenanceTask>,
    pub last_maintenance: u64,
    pub next_maintenance: u64,
    pub emergency_repairs: Vec<EmergencyRepair>,
}

#[derive(Debug, Clone)]
pub struct MaintenanceTask {
    pub task_type: MaintenanceType,
    pub frequency: u64, // Time interval
    pub estimated_duration: u64,
    pub cost: f32,
    pub required_resources: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum MaintenanceType {
    Inspection,
    Cleaning,
    Lubrication,
    Replacement,
    Repair,
    Upgrade,
    Calibration,
}

#[derive(Debug, Clone)]
pub struct EmergencyRepair {
    pub repair_id: String,
    pub damage_type: DamageType,
    pub severity: f32,
    pub estimated_repair_time: u64,
    pub impact_on_traffic: f32,
}

#[derive(Debug, Clone)]
pub enum DamageType {
    Structural,
    Surface,
    Electrical,
    Mechanical,
    Environmental,
}

#[derive(Debug, Clone)]
pub struct RouteNetwork {
    pub nodes: HashMap<String, RouteNode>,
    pub edges: HashMap<String, RouteEdge>,
    pub route_graph: HashMap<String, Vec<String>>, // Adjacency list
    pub traffic_zones: HashMap<String, TrafficZone>,
}

#[derive(Debug, Clone)]
pub struct RouteNode {
    pub id: String,
    pub position: Point3,
    pub node_type: NodeType,
    pub connections: Vec<String>, // Connected edge IDs
    pub traffic_capacity: f32,
    pub current_traffic: f32,
}

#[derive(Debug, Clone)]
pub enum NodeType {
    Intersection,
    Destination,
    Waypoint,
    ServicePoint,
    ParkingArea,
    LoadingZone,
    TrafficControl,
}

#[derive(Debug, Clone)]
pub struct RouteEdge {
    pub id: String,
    pub from_node: String,
    pub to_node: String,
    pub length: f32,
    pub road_type: RoadType,
    pub lane_count: u8,
    pub speed_limit: f32,
    pub road_conditions: RoadConditions,
    pub traffic_flow: TrafficFlow,
    pub restrictions: Vec<RouteRestriction>,
}

#[derive(Debug, Clone)]
pub struct RoadConditions {
    pub surface_quality: f32,
    pub weather_impact: f32,
    pub visibility: f32,
    pub construction_zones: Vec<ConstructionZone>,
    pub incidents: Vec<RoadIncident>,
}

#[derive(Debug, Clone)]
pub struct ConstructionZone {
    pub zone_id: String,
    pub start_position: f32,
    pub end_position: f32,
    pub lane_restrictions: u8,
    pub speed_reduction: f32,
    pub estimated_completion: u64,
}

#[derive(Debug, Clone)]
pub struct RoadIncident {
    pub incident_id: String,
    pub incident_type: IncidentType,
    pub position: f32,
    pub severity: f32,
    pub estimated_clearance: u64,
    pub lanes_affected: u8,
}

#[derive(Debug, Clone)]
pub enum IncidentType {
    Accident,
    Breakdown,
    Debris,
    Weather,
    Emergency,
    Maintenance,
}

#[derive(Debug, Clone)]
pub struct TrafficFlow {
    pub current_density: f32,
    pub average_speed: f32,
    pub flow_direction: FlowDirection,
    pub congestion_level: f32,
}

#[derive(Debug, Clone)]
pub enum FlowDirection {
    Bidirectional,
    OneWay,
    Reversible,
}

#[derive(Debug, Clone)]
pub struct RouteRestriction {
    pub restriction_type: RestrictionType,
    pub applies_to: Vec<VehicleType>,
    pub time_restrictions: Vec<TimeRestriction>,
    pub conditional_restrictions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TrafficZone {
    pub zone_id: String,
    pub zone_type: ZoneType,
    pub boundaries: Vec<Point3>,
    pub traffic_rules: Vec<TrafficRule>,
    pub speed_limits: HashMap<VehicleType, f32>,
    pub access_restrictions: Vec<AccessRestriction>,
}

#[derive(Debug, Clone)]
pub enum ZoneType {
    Residential,
    Commercial,
    Industrial,
    School,
    Hospital,
    EmergencyServices,
    PedestrianOnly,
    RestrictedAccess,
}

#[derive(Debug, Clone)]
pub struct TrafficRule {
    pub rule_id: String,
    pub rule_type: RuleType,
    pub enforcement_level: f32,
    pub penalty: Penalty,
}

#[derive(Debug, Clone)]
pub enum RuleType {
    SpeedLimit,
    StopSign,
    YieldSign,
    NoParking,
    NoUturn,
    RightOfWay,
    TrafficLight,
    LaneRestriction,
}

#[derive(Debug, Clone)]
pub struct Penalty {
    pub penalty_type: PenaltyType,
    pub severity: f32,
    pub cost: f32,
}

#[derive(Debug, Clone)]
pub enum PenaltyType {
    Warning,
    Fine,
    Impoundment,
    LicenseSuspension,
    RouteBan,
}

#[derive(Debug, Clone)]
pub struct AccessRestriction {
    pub restriction_id: String,
    pub allowed_vehicles: Vec<VehicleType>,
    pub permit_requirements: Vec<String>,
    pub time_windows: Vec<TimeWindow>,
}

#[derive(Debug, Clone)]
pub struct TimeWindow {
    pub start_time: u32,
    pub end_time: u32,
    pub days: Vec<u8>,
    pub exceptions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct VehiclePhysicsSettings {
    pub gravity: f32,
    pub air_density: f32,
    pub friction_coefficients: HashMap<String, f32>, // Surface type -> friction
    pub collision_detection_enabled: bool,
    pub realistic_physics: bool,
    pub damage_system_enabled: bool,
    pub fuel_consumption_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct TrafficSettings {
    pub traffic_density: f32,
    pub ai_vehicle_spawn_rate: f32,
    pub traffic_enforcement: bool,
    pub dynamic_traffic_lights: bool,
    pub emergency_vehicle_priority: bool,
    pub adaptive_routing: bool,
    pub congestion_pricing: bool,
}

impl VehicleSystem {
    pub fn new() -> Self {
        Self {
            vehicle_controller: VehicleController::new(),
            transportation_network: TransportationNetwork::new(),
            vehicle_designer: VehicleDesigner::new(),
            route_planner: RoutePlanner::new(),
            traffic_manager: TrafficManager::new(),
            
            active_vehicles: HashMap::new(),
            infrastructure: HashMap::new(),
            route_network: RouteNetwork::new(),
            
            physics_settings: VehiclePhysicsSettings::default(),
            traffic_settings: TrafficSettings::default(),
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        // Update vehicle physics and movement
        self.vehicle_controller.update(&mut self.active_vehicles, delta_time, &self.physics_settings);
        
        // Update traffic management
        self.traffic_manager.update(delta_time, &self.active_vehicles);
        
        // Update route planning
        let mut vehicles_vec: Vec<Vehicle> = self.active_vehicles.values().cloned().collect();
        self.route_planner.update_routes(&mut vehicles_vec, &self.route_network);
        // Update the HashMap with potentially modified vehicles
        for vehicle in vehicles_vec {
            self.active_vehicles.insert(vehicle.id.clone(), vehicle);
        }
        
        // Update transportation network
        self.transportation_network.update(delta_time);
        
        // Process vehicle AI decisions
        self.process_vehicle_ai(delta_time);
        
        // Handle maintenance and repairs
        self.handle_maintenance();
    }

    pub fn spawn_vehicle(&mut self, vehicle_template: VehicleTemplate, position: Point3) -> String {
        let designer_template = vehicle_designer::VehicleTemplate {
            name: vehicle_template.template_name.clone(),
            vehicle_type: vehicle_template.base_vehicle_type,
            vehicle_class: VehicleClass::Sedan, // Default value
            default_components: vehicle_designer::ComponentSelection::default(),
            default_customizations: vehicle_designer::DesignCustomizations::default(),
            performance_targets: vehicle_designer::PerformanceTargets::default(),
            constraints: vehicle_designer::DesignConstraints::default(),
        };
        let vehicle = self.vehicle_designer.create_vehicle_from_template(designer_template, position);
        let vehicle_id = vehicle.id.clone();
        self.active_vehicles.insert(vehicle_id.clone(), vehicle);
        vehicle_id
    }

    pub fn despawn_vehicle(&mut self, vehicle_id: &str) -> bool {
        self.active_vehicles.remove(vehicle_id).is_some()
    }

    pub fn add_infrastructure(&mut self, infrastructure: TransportInfrastructure) {
        self.infrastructure.insert(infrastructure.id.clone(), infrastructure);
        self.route_network.rebuild_network(&self.infrastructure);
    }

    pub fn plan_route(&mut self, vehicle_id: &str, destination: Point3) -> Option<Vec<Point3>> {
        if let Some(vehicle) = self.active_vehicles.get(vehicle_id) {
            let route_request = crate::engine::vehicle::route_planner::RouteRequest {
                request_id: format!("route_{}", vehicle_id),
                start_location: vehicle.position,
                destination,
                waypoints: Vec::new(),
                departure_time: None,
                arrival_time: None,
                planning_type: crate::engine::vehicle::route_planner::RoutePlanningType::Fastest,
                vehicle_constraints: crate::engine::vehicle::route_planner::VehicleConstraints::default(),
                user_preferences: crate::engine::vehicle::route_planner::UserRoutePreferences::default(),
                optimization_criteria: crate::engine::vehicle::route_planner::OptimizationCriteria::default(),
                avoid_traffic: false,
                avoid_tolls: false,
                avoid_highways: false,
                avoid_closed_roads: true,
                emergency_mode: false,
                alternative_preference: 0.5,
            };
            match self.route_planner.plan_route(route_request) {
                Ok(route_result) => {
                    // Extract waypoints from RouteResult
                    // TODO: Implement proper conversion from RouteResult to Vec<Point3>
                    Some(vec![destination]) // Stub implementation
                },
                Err(_) => None,
            }
        } else {
            None
        }
    }

    pub fn get_traffic_analysis(&self) -> TrafficAnalysis {
        self.traffic_manager.analyze_traffic(&self.active_vehicles, &self.route_network)
    }

    pub fn get_vehicle_status(&self, vehicle_id: &str) -> Option<VehicleStatus> {
        self.active_vehicles.get(vehicle_id).map(|v| VehicleStatus::from(v))
    }

    pub fn get_system_performance(&self) -> VehicleSystemPerformance {
        VehicleSystemPerformance {
            active_vehicle_count: self.active_vehicles.len(),
            infrastructure_count: self.infrastructure.len(),
            average_vehicle_speed: self.calculate_average_speed(),
            traffic_congestion_level: self.calculate_congestion_level(),
            fuel_efficiency: self.calculate_system_efficiency(),
            maintenance_alerts: self.get_maintenance_alerts(),
        }
    }

    fn process_vehicle_ai(&mut self, delta_time: f32) {
        let vehicle_ids: Vec<_> = self.active_vehicles.keys().cloned().collect();

        for vehicle_id in vehicle_ids {
            // First, get the decision without holding mutable references
            let decision_option = if let Some(vehicle) = self.active_vehicles.get_mut(&vehicle_id) {
                if let Some(ai) = &mut vehicle.ai_controller {
                    // Clone the vehicle data needed for decision making to avoid borrowing conflicts
                    let vehicle_data = VehicleDecisionData {
                        position: vehicle.position,
                        velocity: vehicle.velocity,
                        acceleration: vehicle.acceleration,
                        operational_state: vehicle.operational_state.clone(),
                        current_route: vehicle.current_route.clone(),
                        destination: vehicle.destination,
                        navigation_state: vehicle.navigation_state.clone(),
                    };
                    Some(ai.make_decision_with_data(&vehicle_data, &self.route_network, delta_time))
                } else {
                    None
                }
            } else {
                None
            };

            // Now execute the action with a fresh mutable borrow
            if let (Some(decision), Some(vehicle)) = (decision_option, self.active_vehicles.get_mut(&vehicle_id)) {
                Self::execute_vehicle_action_direct(vehicle, decision);
            }
        }
    }

    fn execute_vehicle_action(&mut self, vehicle: &mut Vehicle, action: VehicleAction) {
        Self::execute_vehicle_action_direct(vehicle, action);
    }

    fn execute_vehicle_action_direct(vehicle: &mut Vehicle, action: VehicleAction) {
        match action {
            VehicleAction::Accelerate(force) => {
                vehicle.acceleration = vehicle.velocity.normalize() * force;
                vehicle.operational_state = VehicleState::Accelerating;
            },
            VehicleAction::Brake(force) => {
                let brake_vector = vehicle.velocity.normalize() * -force;
                vehicle.acceleration = brake_vector;
                vehicle.operational_state = VehicleState::Decelerating;
            },
            VehicleAction::Steer(angle) => {
                vehicle.steering.steering_angle = angle;
                vehicle.operational_state = VehicleState::Turning;
            },
            _ => {
                // Handle other actions
            }
        }
    }

    fn handle_maintenance(&mut self) {
        for vehicle in self.active_vehicles.values_mut() {
            // Decrease maintenance level over time
            vehicle.maintenance_level -= 0.001;
            
            // Check if maintenance is needed
            if vehicle.maintenance_level < 0.3 {
                vehicle.operational_state = VehicleState::Maintenance;
            }
        }
        
        for infrastructure in self.infrastructure.values_mut() {
            // Update infrastructure condition
            infrastructure.condition -= 0.0001;
        }
    }

    fn calculate_average_speed(&self) -> f32 {
        if self.active_vehicles.is_empty() {
            return 0.0;
        }
        
        let total_speed: f32 = self.active_vehicles.values()
            .map(|v| v.velocity.magnitude())
            .sum();
        
        total_speed / self.active_vehicles.len() as f32
    }

    fn calculate_congestion_level(&self) -> f32 {
        // Simplified congestion calculation
        let total_capacity: f32 = self.route_network.nodes.values()
            .map(|node| node.traffic_capacity)
            .sum();
        
        let current_traffic: f32 = self.route_network.nodes.values()
            .map(|node| node.current_traffic)
            .sum();
        
        if total_capacity > 0.0 {
            current_traffic / total_capacity
        } else {
            0.0
        }
    }

    fn calculate_system_efficiency(&self) -> f32 {
        if self.active_vehicles.is_empty() {
            return 1.0;
        }
        
        let total_efficiency: f32 = self.active_vehicles.values()
            .map(|v| v.engine.fuel_efficiency)
            .sum();
        
        total_efficiency / self.active_vehicles.len() as f32
    }

    fn get_maintenance_alerts(&self) -> Vec<MaintenanceAlert> {
        let mut alerts = Vec::new();
        
        for vehicle in self.active_vehicles.values() {
            if vehicle.maintenance_level < 0.3 {
                alerts.push(MaintenanceAlert {
                    alert_type: AlertType::VehicleMaintenance,
                    entity_id: vehicle.id.clone(),
                    severity: AlertSeverity::Medium,
                    description: format!("Vehicle {} requires maintenance", vehicle.name),
                });
            }
        }
        
        for infrastructure in self.infrastructure.values() {
            if infrastructure.condition < 0.5 {
                alerts.push(MaintenanceAlert {
                    alert_type: AlertType::InfrastructureMaintenance,
                    entity_id: infrastructure.id.clone(),
                    severity: AlertSeverity::High,
                    description: format!("Infrastructure {} needs repair", infrastructure.name),
                });
            }
        }
        
        alerts
    }
}

impl RouteNetwork {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            route_graph: HashMap::new(),
            traffic_zones: HashMap::new(),
        }
    }

    pub fn rebuild_network(&mut self, infrastructure: &HashMap<String, TransportInfrastructure>) {
        // Rebuild the route network based on current infrastructure
        self.nodes.clear();
        self.edges.clear();
        self.route_graph.clear();
        
        // Add nodes from infrastructure
        for infra in infrastructure.values() {
            let node = RouteNode {
                id: infra.id.clone(),
                position: infra.location,
                node_type: self.infrastructure_to_node_type(&infra.infrastructure_type),
                connections: Vec::new(),
                traffic_capacity: infra.capacity,
                current_traffic: infra.current_utilization,
            };
            
            self.nodes.insert(node.id.clone(), node);
        }
        
        // Build connections between nodes (simplified)
        self.build_connections();
    }

    fn infrastructure_to_node_type(&self, infra_type: &InfrastructureType) -> NodeType {
        match infra_type {
            InfrastructureType::Intersection => NodeType::Intersection,
            InfrastructureType::ParkingLot => NodeType::ParkingArea,
            InfrastructureType::GasStation => NodeType::ServicePoint,
            _ => NodeType::Waypoint,
        }
    }

    fn build_connections(&mut self) {
        // Simplified connection building - in reality would use spatial algorithms
        let node_ids: Vec<_> = self.nodes.keys().cloned().collect();
        
        for i in 0..node_ids.len() {
            for j in (i + 1)..node_ids.len() {
                let distance = {
                    let node1 = &self.nodes[&node_ids[i]];
                    let node2 = &self.nodes[&node_ids[j]];
                    (node1.position - node2.position).magnitude()
                };
                
                // Connect nodes within reasonable distance
                if distance < 1000.0 {
                    let edge_id = format!("{}_{}", node_ids[i], node_ids[j]);
                    let edge = RouteEdge {
                        id: edge_id.clone(),
                        from_node: node_ids[i].clone(),
                        to_node: node_ids[j].clone(),
                        length: distance,
                        road_type: RoadType::Local,
                        lane_count: 2,
                        speed_limit: 50.0,
                        road_conditions: RoadConditions {
                            surface_quality: 0.8,
                            weather_impact: 0.0,
                            visibility: 1.0,
                            construction_zones: Vec::new(),
                            incidents: Vec::new(),
                        },
                        traffic_flow: TrafficFlow {
                            current_density: 0.3,
                            average_speed: 45.0,
                            flow_direction: FlowDirection::Bidirectional,
                            congestion_level: 0.2,
                        },
                        restrictions: Vec::new(),
                    };
                    
                    self.edges.insert(edge_id.clone(), edge);
                    
                    // Add to adjacency list
                    self.route_graph.entry(node_ids[i].clone())
                        .or_insert_with(Vec::new)
                        .push(node_ids[j].clone());
                    self.route_graph.entry(node_ids[j].clone())
                        .or_insert_with(Vec::new)
                        .push(node_ids[i].clone());
                }
            }
        }
    }
}

impl Default for VehiclePhysicsSettings {
    fn default() -> Self {
        let mut friction_coefficients = HashMap::new();
        friction_coefficients.insert("asphalt".to_string(), 0.7);
        friction_coefficients.insert("concrete".to_string(), 0.8);
        friction_coefficients.insert("gravel".to_string(), 0.5);
        friction_coefficients.insert("dirt".to_string(), 0.4);
        friction_coefficients.insert("ice".to_string(), 0.1);
        friction_coefficients.insert("snow".to_string(), 0.3);
        
        Self {
            gravity: 9.81,
            air_density: 1.225,
            friction_coefficients,
            collision_detection_enabled: true,
            realistic_physics: true,
            damage_system_enabled: true,
            fuel_consumption_enabled: true,
        }
    }
}

impl Default for TrafficSettings {
    fn default() -> Self {
        Self {
            traffic_density: 0.5,
            ai_vehicle_spawn_rate: 0.1,
            traffic_enforcement: true,
            dynamic_traffic_lights: true,
            emergency_vehicle_priority: true,
            adaptive_routing: true,
            congestion_pricing: false,
        }
    }
}

// Additional supporting types
#[derive(Debug, Clone)]
pub struct VehicleTemplate {
    pub template_name: String,
    pub base_vehicle_type: VehicleType,
    pub customizations: Vec<VehicleCustomization>,
}

#[derive(Debug, Clone)]
pub enum VehicleCustomization {
    EngineUpgrade(Engine),
    TransmissionModification(Transmission),
    SuspensionTuning(Suspension),
    ElectronicsPackage(ElectronicsSystem),
    VisualModification(String),
}

#[derive(Debug)]
pub struct VehicleStatus {
    pub position: Point3,
    pub speed: f32,
    pub fuel_level: f32,
    pub maintenance_level: f32,
    pub operational_state: VehicleState,
    pub destination: Option<Point3>,
}

impl From<&Vehicle> for VehicleStatus {
    fn from(vehicle: &Vehicle) -> Self {
        Self {
            position: vehicle.position,
            speed: vehicle.velocity.magnitude(),
            fuel_level: vehicle.current_fuel / vehicle.fuel_capacity,
            maintenance_level: vehicle.maintenance_level,
            operational_state: vehicle.operational_state.clone(),
            destination: vehicle.destination,
        }
    }
}

#[derive(Debug)]
pub struct EnvironmentalImpact {
    pub carbon_emissions: f32,
    pub co2_emissions: f32, // Alias for carbon_emissions
    pub fuel_consumption: f32,
    pub energy_consumption: f32, // Additional energy consumption field
    pub noise_level: f32,
    pub air_quality_index: f32,
}

#[derive(Debug)]
pub struct TrafficAnalysis {
    pub average_speed: f32,
    pub congestion_level: f32,
    pub bottlenecks: Vec<TrafficBottleneck>,
    pub flow_efficiency: f32,
    // Additional fields for compatibility with traffic_manager.rs
    pub total_vehicles: u32,
    pub traffic_flow_rate: f32,
    pub incident_count: u32,
    pub efficiency_score: f32,
    pub environmental_impact: EnvironmentalImpact,
    pub recommendations: Vec<String>,
}

#[derive(Debug)]
pub struct TrafficBottleneck {
    pub location: Point3,
    pub severity: f32,
    pub cause: String,
    pub estimated_delay: u64,
}

#[derive(Debug)]
pub struct VehicleSystemPerformance {
    pub active_vehicle_count: usize,
    pub infrastructure_count: usize,
    pub average_vehicle_speed: f32,
    pub traffic_congestion_level: f32,
    pub fuel_efficiency: f32,
    pub maintenance_alerts: Vec<MaintenanceAlert>,
}

#[derive(Debug)]
pub struct MaintenanceAlert {
    pub alert_type: AlertType,
    pub entity_id: String,
    pub severity: AlertSeverity,
    pub description: String,
}

#[derive(Debug)]
pub enum AlertType {
    VehicleMaintenance,
    InfrastructureMaintenance,
    TrafficIncident,
    SystemWarning,
}

#[derive(Debug)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

// Vec3 extension for vehicle calculations
// Note: Vec3 and Point3 operations are provided by cgmath crate

// Missing structs for vehicle designer

// Note: Tires and Electronics are already defined as type aliases above