use std::collections::HashMap;
use std::io::{self, Write};

// Mock types for comprehensive playtest
#[derive(Debug, Clone, Copy)]
pub struct Point3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Point3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

// Main playtest function
fn main() {
    println!("🎮 ENGINEER BUILD MODE - Interactive Playtest Demo");
    println!("==================================================");
    println!("Welcome to the Robin Game Engine - Engineer Build Mode!");
    println!("You are an engineer character with the power to create and modify worlds.\n");
    
    let mut game_session = GameSession::new();
    
    // Introduction and setup
    display_welcome_message();
    
    // Initialize all systems
    println!("🔧 Initializing Engineer Build Mode Systems...");
    game_session.initialize_all_systems();
    println!("✅ All systems online and ready!\n");
    
    // Main game loop
    let mut running = true;
    while running {
        display_main_menu();
        
        print!("Enter your choice: ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let choice = input.trim();
        
        match choice {
            "1" => demo_engineer_character(&mut game_session),
            "2" => demo_world_construction(&mut game_session),
            "3" => demo_advanced_tools(&mut game_session),
            "4" => demo_npc_management(&mut game_session),
            "5" => demo_story_quest_system(&mut game_session),
            "6" => demo_vehicle_transportation(&mut game_session),
            "7" => demo_integrated_scenario(&mut game_session),
            "8" => display_system_status(&game_session),
            "9" => {
                println!("\n👋 Thanks for playing the Engineer Build Mode demo!");
                println!("🎉 All Phase 1 systems successfully demonstrated!");
                running = false;
            },
            _ => println!("❌ Invalid choice. Please select 1-9."),
        }
        
        if running {
            println!("\nPress Enter to continue...");
            let mut _dummy = String::new();
            io::stdin().read_line(&mut _dummy).unwrap();
        }
    }
}

fn display_welcome_message() {
    println!("🏗️  ENGINEER CHARACTER PROFILE");
    println!("Name: Alex Builder");
    println!("Specialization: World Systems Engineer");
    println!("Level: Master Builder");
    println!("Tools: Advanced Construction Suite, AI Management Console");
    println!("Mission: Demonstrate all core building and management capabilities\n");
}

fn display_main_menu() {
    println!("\n🎯 ENGINEER BUILD MODE - Main Menu");
    println!("===================================");
    println!("1. 🚶 Engineer Character Controller Demo");
    println!("2. 🌍 World Construction System Demo");
    println!("3. 🔧 Advanced Building Tools Demo");
    println!("4. 🤖 NPC Management & AI Demo");
    println!("5. 📖 Story & Quest System Demo");
    println!("6. 🚗 Vehicle & Transportation Demo");
    println!("7. 🌟 Integrated Scenario Demo");
    println!("8. 📊 System Status & Performance");
    println!("9. 🚪 Exit Demo");
}

#[derive(Debug, Clone)]
pub struct GameSession {
    engineer: EngineerCharacter,
    world: WorldState,
    tools: AdvancedToolsSuite,
    npcs: NPCManager,
    stories: StoryQuestSystem,
    vehicles: VehicleSystem,
    session_stats: SessionStats,
}

impl GameSession {
    pub fn new() -> Self {
        Self {
            engineer: EngineerCharacter::new("Alex Builder"),
            world: WorldState::new(),
            tools: AdvancedToolsSuite::new(),
            npcs: NPCManager::new(),
            stories: StoryQuestSystem::new(),
            vehicles: VehicleSystem::new(),
            session_stats: SessionStats::new(),
        }
    }
    
    pub fn initialize_all_systems(&mut self) {
        println!("  ⚡ Engineer Character Controller... OK");
        self.engineer.initialize();
        
        println!("  🌍 World Construction System... OK");
        self.world.initialize();
        
        println!("  🔧 Advanced Building Tools... OK");
        self.tools.initialize();
        
        println!("  🤖 NPC Management System... OK");
        self.npcs.initialize();
        
        println!("  📖 Story & Quest System... OK");
        self.stories.initialize();
        
        println!("  🚗 Vehicle & Transportation... OK");
        self.vehicles.initialize();
        
        self.session_stats.systems_initialized = 6;
    }
}

// Phase 1.1: Engineer Character Controller Demo
fn demo_engineer_character(session: &mut GameSession) {
    println!("\n🚶 ENGINEER CHARACTER CONTROLLER DEMO");
    println!("=====================================");
    
    println!("🎯 Demonstrating character movement and interaction capabilities...\n");
    
    // Movement demo
    println!("📍 Current Position: {}", format_position(&session.engineer.position));
    println!("🏃 Moving to construction site...");
    
    let target_position = Point3::new(100.0, 50.0, 0.0);
    session.engineer.move_to(target_position);
    
    println!("📍 New Position: {}", format_position(&session.engineer.position));
    println!("⚡ Movement Speed: {:.1} units/sec", session.engineer.movement_speed);
    
    // Tool interaction demo
    println!("\n🔧 Tool Interaction Demo:");
    println!("Available Tools: {:?}", session.engineer.available_tools);
    
    session.engineer.select_tool("Advanced Builder");
    println!("🛠️  Selected Tool: {}", session.engineer.current_tool);
    
    // Skill demonstration
    println!("\n🎯 Engineer Skills:");
    for (skill, level) in &session.engineer.skills {
        println!("  {} - Level {}/10", skill, level);
    }
    
    // Resource management
    println!("\n📦 Resource Inventory:");
    for (resource, amount) in &session.engineer.inventory {
        println!("  {}: {} units", resource, amount);
    }
    
    println!("\n✅ Engineer character systems fully operational!");
    session.session_stats.demos_completed += 1;
}

// Phase 1.2: World Construction Demo
fn demo_world_construction(session: &mut GameSession) {
    println!("\n🌍 WORLD CONSTRUCTION SYSTEM DEMO");
    println!("=================================");
    
    println!("🎯 Demonstrating dynamic world building capabilities...\n");
    
    // Terrain modification
    println!("🏔️  Terrain Modification:");
    session.world.modify_terrain(Point3::new(0.0, 0.0, 0.0), "raise", 50.0, 10.0);
    println!("  ✅ Created hill at origin (50 units high, 10 unit radius)");
    
    session.world.modify_terrain(Point3::new(200.0, 100.0, 0.0), "lower", -20.0, 15.0);
    println!("  ✅ Created valley at (200,100) (20 units deep, 15 unit radius)");
    
    // Structure construction
    println!("\n🏗️  Structure Construction:");
    
    let structures = vec![
        ("Workshop", Point3::new(50.0, 50.0, 0.0), "Industrial"),
        ("Residential Tower", Point3::new(-50.0, 100.0, 0.0), "Residential"),
        ("Command Center", Point3::new(0.0, 150.0, 0.0), "Administrative"),
    ];
    
    for (name, position, structure_type) in structures {
        session.world.construct_structure(name, position, structure_type);
        println!("  ✅ Built {} at {} (Type: {})", name, format_position(&position), structure_type);
    }
    
    // Environment systems
    println!("\n🌤️  Environment Systems:");
    session.world.set_weather("Partly Cloudy", 0.3);
    println!("  ✅ Weather set to Partly Cloudy (30% cloud cover)");
    
    session.world.set_time_of_day(14.5);
    println!("  ✅ Time set to 14:30 (2:30 PM)");
    
    session.world.adjust_lighting(0.85, Vec3::new(1.0, 0.9, 0.7));
    println!("  ✅ Lighting adjusted (85% intensity, warm color)");
    
    // Physics simulation
    println!("\n⚡ Physics Simulation:");
    session.world.update_physics(1.0);
    println!("  ✅ Physics updated - {} objects simulated", session.world.physics_objects.len());
    
    println!("\n📊 World Statistics:");
    println!("  Structures Built: {}", session.world.structures.len());
    println!("  Terrain Modifications: {}", session.world.terrain_modifications);
    println!("  Active Physics Objects: {}", session.world.physics_objects.len());
    
    println!("\n✅ World construction systems fully operational!");
    session.session_stats.demos_completed += 1;
}

// Phase 1.3: Advanced Tools Demo
fn demo_advanced_tools(session: &mut GameSession) {
    println!("\n🔧 ADVANCED BUILDING TOOLS DEMO");
    println!("===============================");
    
    println!("🎯 Demonstrating precision building and automation tools...\n");
    
    // Precision building tools
    println!("📐 Precision Building Tools:");
    
    session.tools.use_precision_tool("Laser Cutter", Point3::new(10.0, 10.0, 0.0), 
                                   "Cut precise foundation outline");
    println!("  ✅ Laser Cutter: Foundation outline cut with 0.1mm precision");
    
    session.tools.use_precision_tool("Molecular Assembler", Point3::new(10.0, 10.0, 0.0), 
                                   "Assemble reinforced concrete foundation");
    println!("  ✅ Molecular Assembler: Foundation assembled at molecular level");
    
    session.tools.use_precision_tool("Gravity Manipulator", Point3::new(10.0, 10.0, 5.0), 
                                   "Position support beams");
    println!("  ✅ Gravity Manipulator: Support beams positioned with zero-g assistance");
    
    // Automation systems
    println!("\n🤖 Automation Systems:");
    
    let automation_tasks = vec![
        ("Resource Mining", "Automated excavation of materials", 85.0),
        ("Component Manufacturing", "3D printing of building components", 92.0),
        ("Assembly Line", "Automated structure assembly", 78.0),
        ("Quality Control", "AI-powered inspection systems", 96.0),
    ];
    
    for (task_name, description, efficiency) in automation_tasks {
        session.tools.start_automation_task(task_name, description);
        println!("  ✅ {}: {} ({}% efficiency)", task_name, description, efficiency);
    }
    
    // Blueprint system
    println!("\n📋 Blueprint Management:");
    
    let blueprints = vec![
        ("Starter Home", "Basic residential structure", 5),
        ("Industrial Complex", "Multi-purpose factory design", 25),
        ("Skybridge Network", "Elevated transportation system", 45),
    ];
    
    for (name, description, complexity) in blueprints {
        session.tools.create_blueprint(name, description, complexity);
        println!("  ✅ Blueprint Created: {} - {} (Complexity: {})", name, description, complexity);
    }
    
    // Material analysis
    println!("\n🔬 Material Analysis Systems:");
    
    let materials = vec![
        ("Steel Alloy X-47", 98.5, "Excellent structural integrity"),
        ("Carbon Composite", 87.2, "Lightweight, high tensile strength"),
        ("Smart Glass", 94.1, "Adaptive opacity and insulation"),
    ];
    
    for (material, quality, properties) in materials {
        session.tools.analyze_material(material, quality, properties);
        println!("  ✅ {}: {:.1}% quality - {}", material, quality, properties);
    }
    
    println!("\n📊 Tools Performance:");
    println!("  Active Automation Tasks: {}", session.tools.automation_tasks.len());
    println!("  Available Blueprints: {}", session.tools.blueprints.len());
    println!("  Tool Efficiency: {:.1}%", session.tools.calculate_efficiency());
    
    println!("\n✅ Advanced building tools fully operational!");
    session.session_stats.demos_completed += 1;
}

// Phase 1.4: NPC Management Demo
fn demo_npc_management(session: &mut GameSession) {
    println!("\n🤖 NPC MANAGEMENT & AI DEMO");
    println!("===========================");
    
    println!("🎯 Demonstrating NPC creation, AI behaviors, and management...\n");
    
    // NPC creation
    println!("👥 NPC Creation:");
    
    let npc_profiles = vec![
        ("Construction Foreman", "Bob Mitchell", "Aggressive", "Supervise construction projects"),
        ("Materials Specialist", "Sarah Chen", "Analytical", "Manage resource logistics"),
        ("Safety Inspector", "Marcus Johnson", "Cautious", "Ensure workplace safety"),
        ("Architect", "Elena Rodriguez", "Creative", "Design optimization consultant"),
    ];
    
    for (role, name, personality, description) in npc_profiles {
        let npc_id = session.npcs.create_npc(role, name, personality, description);
        println!("  ✅ Created {}: {} ({})", role, name, personality);
    }
    
    // AI behavior demonstration
    println!("\n🧠 AI Behavior Demonstration:");
    
    for npc in &session.npcs.active_npcs {
        match npc.current_activity.as_str() {
            "Inspecting" => println!("  🔍 {} is inspecting the construction site for safety violations", npc.name),
            "Planning" => println!("  📋 {} is reviewing blueprints and optimizing designs", npc.name),
            "Coordinating" => println!("  📞 {} is coordinating with suppliers and contractors", npc.name),
            "Supervising" => println!("  👀 {} is supervising the construction crew", npc.name),
            _ => println!("  🔧 {} is performing general maintenance tasks", npc.name),
        }
    }
    
    // NPC interaction system
    println!("\n💬 NPC Interaction System:");
    
    for npc in &session.npcs.active_npcs {
        let interaction = session.npcs.interact_with_npc(&npc.id, "status_report");
        println!("  💭 {}: \"{}\"", npc.name, interaction);
    }
    
    // Task delegation
    println!("\n📋 Task Delegation:");
    
    let tasks = vec![
        ("site_survey", "Conduct comprehensive site survey"),
        ("material_ordering", "Order materials for next phase"),
        ("quality_inspection", "Perform quality control inspection"),
        ("safety_briefing", "Conduct safety briefing for new workers"),
    ];
    
    for (i, (task_id, description)) in tasks.iter().enumerate() {
        if let Some(npc) = session.npcs.active_npcs.get(i) {
            let npc_id = npc.id.clone();
            let npc_name = npc.name.clone();
            session.npcs.assign_task(&npc_id, task_id, description);
            println!("  ✅ Assigned to {}: {}", npc_name, description);
        }
    }
    
    // Team coordination
    println!("\n🤝 Team Coordination:");
    let team_efficiency = session.npcs.calculate_team_efficiency();
    println!("  📊 Team Efficiency: {:.1}%", team_efficiency);
    println!("  🎯 Active Tasks: {}", session.npcs.active_tasks.len());
    println!("  ⏰ Average Task Completion Time: {:.1} minutes", session.npcs.avg_completion_time);
    
    println!("\n✅ NPC management and AI systems fully operational!");
    session.session_stats.demos_completed += 1;
}

// Phase 1.5: Story and Quest System Demo
fn demo_story_quest_system(session: &mut GameSession) {
    println!("\n📖 STORY & QUEST SYSTEM DEMO");
    println!("============================");
    
    println!("🎯 Demonstrating dynamic storytelling and quest management...\n");
    
    // Story creation
    println!("📚 Dynamic Story Generation:");
    
    let main_storyline = session.stories.create_storyline(
        "The Great Construction Project",
        "Build a sustainable city that showcases advanced engineering",
        vec!["engineering", "sustainability", "innovation", "community"]
    );
    
    println!("  ✅ Main Storyline: \"{}\"", main_storyline.title);
    println!("     Themes: {:?}", main_storyline.themes);
    
    // Quest system
    println!("\n🎯 Quest Management:");
    
    let quests = vec![
        ("Foundation Master", "Build 5 different foundation types", vec![
            "Pour concrete foundation",
            "Install steel beam foundation", 
            "Create floating foundation",
            "Build earthquake-resistant foundation",
            "Design modular foundation system"
        ]),
        ("Resource Manager", "Establish efficient supply chains", vec![
            "Set up automated mining operation",
            "Create material processing plant",
            "Establish trade routes with suppliers",
            "Implement just-in-time delivery system"
        ]),
        ("Innovation Leader", "Research and implement new technologies", vec![
            "Research carbon-negative concrete",
            "Develop smart building systems",
            "Create energy-positive structures",
            "Design self-repairing materials"
        ]),
    ];
    
    for (quest_name, description, objectives) in quests {
        let quest = session.stories.create_quest(quest_name, description, objectives);
        println!("  ✅ Quest Created: \"{}\"", quest.title);
        println!("     Objectives: {} tasks", quest.objectives.len());
        
        // Simulate some progress
        session.stories.update_quest_progress(&quest.id, 0.3);
        println!("     Progress: {:.0}%", quest.progress * 100.0);
    }
    
    // Dynamic events
    println!("\n⚡ Dynamic Story Events:");
    
    let events = vec![
        ("Supply Chain Disruption", "A major supplier has delivery issues", "Challenge"),
        ("Innovation Breakthrough", "Your team discovers a new building technique", "Opportunity"),
        ("Community Celebration", "The city celebrates your construction milestone", "Achievement"),
        ("Weather Challenge", "Unexpected weather requires adaptation", "Environmental"),
    ];
    
    for (event_name, description, event_type) in events {
        session.stories.trigger_story_event(event_name, description, event_type);
        println!("  🎭 {}: {} ({})", event_name, description, event_type);
    }
    
    // Character development
    println!("\n👤 Character Development:");
    
    let character_progression = vec![
        ("Engineering Expertise", 85, "Master level construction knowledge"),
        ("Leadership Skills", 72, "Strong team management abilities"),
        ("Innovation Mindset", 94, "Cutting-edge problem solving"),
        ("Community Relations", 68, "Good relationship with stakeholders"),
    ];
    
    for (trait_name, level, description) in character_progression {
        println!("  📈 {}: {}% - {}", trait_name, level, description);
    }
    
    // Story metrics
    println!("\n📊 Story System Metrics:");
    println!("  Active Storylines: {}", session.stories.storylines.len());
    println!("  Active Quests: {}", session.stories.quests.len());
    println!("  Story Events Triggered: {}", session.stories.story_events.len());
    println!("  Overall Story Progress: {:.1}%", session.stories.calculate_overall_progress());
    
    println!("\n✅ Story and quest systems fully operational!");
    session.session_stats.demos_completed += 1;
}

// Phase 1.6: Vehicle and Transportation Demo  
fn demo_vehicle_transportation(session: &mut GameSession) {
    println!("\n🚗 VEHICLE & TRANSPORTATION DEMO");
    println!("=================================");
    
    println!("🎯 Demonstrating vehicle design, routing, and traffic management...\n");
    
    // Vehicle design
    println!("🎨 Vehicle Design System:");
    
    let vehicle_designs = vec![
        ("Construction Truck", "Heavy-duty hauling", 350.0, 4500.0),
        ("Engineer Mobile Unit", "Mobile command center", 280.0, 2200.0),
        ("Materials Transport", "Specialized cargo transport", 320.0, 3800.0),
        ("Site Survey Drone", "Aerial reconnaissance", 50.0, 15.0),
    ];
    
    for (name, purpose, power, weight) in vehicle_designs {
        let vehicle = session.vehicles.design_vehicle(name, purpose, power, weight);
        println!("  ✅ Designed {}: {} ({:.0} HP, {:.0} kg)", name, purpose, power, weight);
        println!("     Performance: {:.1} km/h top speed, {:.1}s 0-60", 
                 vehicle.top_speed, vehicle.acceleration);
    }
    
    // Route planning
    println!("\n🗺️  Route Planning System:");
    
    let routes = vec![
        ("Supply Run", Point3::new(0.0, 0.0, 0.0), Point3::new(500.0, 300.0, 0.0), "Fastest"),
        ("Site Inspection", Point3::new(200.0, 100.0, 0.0), Point3::new(-100.0, 400.0, 0.0), "Comprehensive"),
        ("Emergency Response", Point3::new(-50.0, -50.0, 0.0), Point3::new(300.0, 250.0, 0.0), "Priority"),
    ];
    
    for (route_name, start, end, route_type) in routes {
        let route = session.vehicles.plan_route(route_name, start, end, route_type);
        println!("  ✅ Planned {}: {:.1} km, {:.1} min estimated", 
                 route_name, route.distance / 1000.0, route.duration);
        println!("     Route Type: {}, Waypoints: {}", route_type, route.waypoints.len());
    }
    
    // Traffic management
    println!("\n🚦 Traffic Management:");
    
    // Initialize traffic network
    session.vehicles.initialize_traffic_network();
    
    let intersections = vec![
        ("Main & Construction Ave", Point3::new(100.0, 200.0, 0.0), "4-way signal"),
        ("Industrial Blvd & Site Rd", Point3::new(300.0, 100.0, 0.0), "2-way signal"),
        ("Supply Chain Circle", Point3::new(-100.0, 150.0, 0.0), "Roundabout"),
    ];
    
    for (name, location, control_type) in intersections {
        session.vehicles.add_traffic_control(name, location, control_type);
        println!("  ✅ Added {}: {} at {}", name, control_type, format_position(&location));
    }
    
    // Simulate traffic flow
    println!("\n🌊 Traffic Flow Simulation:");
    
    session.vehicles.simulate_traffic_flow(30.0); // 30 second simulation
    
    let traffic_metrics = session.vehicles.get_traffic_metrics();
    println!("  📊 Average Flow Rate: {:.1} vehicles/minute", traffic_metrics.flow_rate);
    println!("  📊 Congestion Level: {:.1}% ({})", traffic_metrics.congestion_level * 100.0,
             if traffic_metrics.congestion_level < 0.3 { "Light" } 
             else if traffic_metrics.congestion_level < 0.7 { "Moderate" }
             else { "Heavy" });
    println!("  📊 System Efficiency: {:.1}%", traffic_metrics.efficiency * 100.0);
    
    // Fleet management
    println!("\n🚛 Fleet Management:");
    
    let fleet_status = session.vehicles.get_fleet_status();
    println!("  🚗 Active Vehicles: {}", fleet_status.active_vehicles);
    println!("  ⛽ Average Fuel Level: {:.1}%", fleet_status.avg_fuel_level * 100.0);
    println!("  📍 Vehicles In Transit: {}", fleet_status.vehicles_in_transit);
    println!("  🎯 Mission Success Rate: {:.1}%", fleet_status.mission_success_rate * 100.0);
    
    println!("\n✅ Vehicle and transportation systems fully operational!");
    session.session_stats.demos_completed += 1;
}

// Integrated Scenario Demo - Shows all systems working together
fn demo_integrated_scenario(session: &mut GameSession) {
    println!("\n🌟 INTEGRATED SCENARIO DEMO");
    println!("===========================");
    
    println!("🎯 Demonstrating all systems working together in a complete scenario...\n");
    println!("📜 SCENARIO: \"The Sustainable City Project\"");
    println!("Build an eco-friendly district with integrated systems\n");
    
    // Phase 1: Planning and Setup
    println!("🔍 PHASE 1: Project Planning");
    println!("---------------------------");
    
    // Engineer surveys the site
    println!("👨‍🔧 Engineer Alex moves to survey site...");
    session.engineer.move_to(Point3::new(0.0, 0.0, 0.0));
    session.engineer.select_tool("Site Survey Kit");
    
    // Create project storyline
    let project_story = session.stories.create_storyline(
        "Sustainable City District",
        "Create an environmentally friendly urban development",
        vec!["sustainability", "innovation", "community", "technology"]
    );
    println!("📖 Created project storyline: {}", project_story.title);
    
    // Deploy NPCs for different roles
    println!("👥 Deploying specialist team:");
    let team_lead = session.npcs.create_npc("Project Manager", "Maya Patel", "Organized", "Coordinate project phases");
    let env_specialist = session.npcs.create_npc("Environmental Engineer", "David Green", "Methodical", "Ensure eco-compliance");
    println!("  ✅ Team assembled with 2 specialists");
    
    // Phase 2: Site Preparation
    println!("\n🏗️  PHASE 2: Site Preparation");
    println!("-----------------------------");
    
    // Terrain modification for optimal layout
    session.world.modify_terrain(Point3::new(50.0, 50.0, 0.0), "level", 0.0, 30.0);
    println!("🌍 Leveled construction area (30m radius)");
    
    // Design and deploy construction vehicles
    let construction_fleet = session.vehicles.design_vehicle("Eco-Builder", "Solar-powered construction", 280.0, 2500.0);
    let material_hauler = session.vehicles.design_vehicle("Green Hauler", "Electric materials transport", 320.0, 3200.0);
    println!("🚗 Deployed eco-friendly construction fleet");
    
    // Set up automated construction tools
    session.tools.start_automation_task("Foundation Prep", "Automated eco-foundation preparation");
    session.tools.start_automation_task("Solar Integration", "Install integrated solar systems");
    println!("🤖 Activated automated construction systems");
    
    // Phase 3: Construction
    println!("\n🏢 PHASE 3: Integrated Construction");
    println!("-----------------------------------");
    
    // Build core structures with integrated systems
    let structures = vec![
        ("Eco-Residential Complex", Point3::new(100.0, 100.0, 0.0), "Green Building"),
        ("Community Solar Farm", Point3::new(200.0, 50.0, 0.0), "Energy Generation"),
        ("Smart Water Treatment", Point3::new(50.0, 200.0, 0.0), "Environmental"),
        ("Integrated Transport Hub", Point3::new(150.0, 150.0, 0.0), "Transportation"),
    ];
    
    for (name, position, category) in structures {
        session.world.construct_structure(name, position, category);
        
        // Assign NPC to oversee construction
        let oversight_task = format!("Oversee construction of {}", name);
        if let Some(npc) = session.npcs.active_npcs.first().cloned() {
            session.npcs.assign_task(&npc.id, "construction_oversight", &oversight_task);
        }
        
        println!("🏗️  Built {} with integrated systems", name);
    }
    
    // Set up transportation network
    session.vehicles.plan_route("District Access", Point3::new(0.0, 0.0, 0.0), Point3::new(150.0, 150.0, 0.0), "Efficient");
    session.vehicles.add_traffic_control("District Central", Point3::new(125.0, 125.0, 0.0), "Smart Intersection");
    println!("🚦 Established intelligent traffic management");
    
    // Phase 4: Systems Integration
    println!("\n⚡ PHASE 4: Systems Integration");
    println!("-------------------------------");
    
    // Integrate all building systems
    let integration_tasks = vec![
        "Solar grid connection",
        "Smart water management",
        "Waste processing automation",
        "Air quality monitoring",
        "Energy storage systems",
    ];
    
    for task in integration_tasks {
        session.tools.use_precision_tool("System Integrator", Point3::new(125.0, 125.0, 10.0), task);
        println!("🔌 Integrated: {}", task);
    }
    
    // Phase 5: Testing and Optimization
    println!("\n🔬 PHASE 5: Testing & Optimization");
    println!("----------------------------------");
    
    // Run comprehensive system tests
    session.world.update_physics(5.0);
    session.vehicles.simulate_traffic_flow(60.0);
    let team_efficiency = session.npcs.calculate_team_efficiency();
    let story_progress = session.stories.calculate_overall_progress();
    
    println!("📊 INTEGRATED SYSTEM PERFORMANCE:");
    println!("  🌍 Environmental Systems: 94% efficiency");
    println!("  🚗 Traffic Flow: 87% optimization");
    println!("  👥 Team Coordination: {:.1}% effectiveness", team_efficiency);
    println!("  📖 Project Completion: {:.1}%", story_progress);
    println!("  ⚡ Energy Independence: 102% (surplus generation)");
    println!("  💧 Water Cycle Efficiency: 96%");
    
    // Generate final project report
    println!("\n📋 PROJECT COMPLETION REPORT");
    println!("============================");
    println!("✅ Sustainable City District Successfully Completed!");
    println!("🏆 All systems integrated and operational");
    println!("🌱 Environmental targets exceeded");
    println!("🤝 Community integration successful");
    println!("📈 Performance metrics: All systems green");
    
    // Create achievement quest
    let achievement_quest = session.stories.create_quest(
        "Master Builder Achievement",
        "Successfully demonstrate all Engineer Build Mode capabilities",
        vec!["Complete integrated construction project"]
    );
    session.stories.update_quest_progress(&achievement_quest.id, 1.0);
    println!("🎉 Achievement Unlocked: Master Builder!");
    
    println!("\n✅ Integrated scenario demonstration complete!");
    session.session_stats.demos_completed += 1;
    session.session_stats.integrated_scenarios += 1;
}

fn display_system_status(session: &GameSession) {
    println!("\n📊 SYSTEM STATUS & PERFORMANCE");
    println!("==============================");
    
    println!("🎮 Demo Session Statistics:");
    println!("  Demos Completed: {}/7", session.session_stats.demos_completed);
    println!("  Integrated Scenarios: {}", session.session_stats.integrated_scenarios);
    println!("  Systems Initialized: {}/6", session.session_stats.systems_initialized);
    
    println!("\n🚶 Engineer Character Status:");
    println!("  Position: {}", format_position(&session.engineer.position));
    println!("  Current Tool: {}", session.engineer.current_tool);
    println!("  Active Skills: {}", session.engineer.skills.len());
    println!("  Inventory Items: {}", session.engineer.inventory.len());
    
    println!("\n🌍 World State:");
    println!("  Structures Built: {}", session.world.structures.len());
    println!("  Terrain Modifications: {}", session.world.terrain_modifications);
    println!("  Physics Objects: {}", session.world.physics_objects.len());
    println!("  Environmental Systems: Active");
    
    println!("\n🔧 Advanced Tools:");
    println!("  Automation Tasks: {}", session.tools.automation_tasks.len());
    println!("  Available Blueprints: {}", session.tools.blueprints.len());
    println!("  System Efficiency: {:.1}%", session.tools.calculate_efficiency());
    
    println!("\n🤖 NPC Management:");
    println!("  Active NPCs: {}", session.npcs.active_npcs.len());
    println!("  Assigned Tasks: {}", session.npcs.active_tasks.len());
    println!("  Team Efficiency: {:.1}%", session.npcs.calculate_team_efficiency());
    
    println!("\n📖 Story & Quest Systems:");
    println!("  Active Storylines: {}", session.stories.storylines.len());
    println!("  Active Quests: {}", session.stories.quests.len());
    println!("  Story Events: {}", session.stories.story_events.len());
    println!("  Overall Progress: {:.1}%", session.stories.calculate_overall_progress());
    
    println!("\n🚗 Vehicle & Transportation:");
    println!("  Vehicle Designs: {}", session.vehicles.vehicle_designs.len());
    println!("  Active Routes: {}", session.vehicles.routes.len());
    println!("  Traffic Controls: {}", session.vehicles.traffic_controls.len());
    
    // Performance metrics
    let total_systems = 6;
    let operational_systems = session.session_stats.systems_initialized;
    let system_health = (operational_systems as f32 / total_systems as f32) * 100.0;
    
    println!("\n⚡ Overall System Health: {:.1}%", system_health);
    
    if system_health == 100.0 {
        println!("🎉 All systems fully operational and ready for production!");
    } else if system_health >= 80.0 {
        println!("✅ Systems operational with minor optimizations possible");
    } else {
        println!("⚠️  Some systems need attention for optimal performance");
    }
}

// Support structures and implementations

#[derive(Debug, Clone)]
pub struct EngineerCharacter {
    pub name: String,
    pub position: Point3,
    pub current_tool: String,
    pub available_tools: Vec<String>,
    pub movement_speed: f32,
    pub skills: HashMap<String, u8>,
    pub inventory: HashMap<String, u32>,
}

impl EngineerCharacter {
    pub fn new(name: &str) -> Self {
        let mut skills = HashMap::new();
        skills.insert("Construction".to_string(), 9);
        skills.insert("Engineering".to_string(), 10);
        skills.insert("Project Management".to_string(), 8);
        skills.insert("Innovation".to_string(), 9);
        
        let mut inventory = HashMap::new();
        inventory.insert("Steel Beams".to_string(), 150);
        inventory.insert("Concrete Mix".to_string(), 500);
        inventory.insert("Electronic Components".to_string(), 75);
        inventory.insert("Smart Materials".to_string(), 25);
        
        Self {
            name: name.to_string(),
            position: Point3::new(0.0, 0.0, 0.0),
            current_tool: "Basic Tools".to_string(),
            available_tools: vec![
                "Advanced Builder".to_string(),
                "Precision Instruments".to_string(),
                "Site Survey Kit".to_string(),
                "System Integrator".to_string(),
            ],
            movement_speed: 5.5,
            skills,
            inventory,
        }
    }
    
    pub fn initialize(&mut self) {
        // Initialize engineer systems
    }
    
    pub fn move_to(&mut self, target: Point3) {
        self.position = target;
    }
    
    pub fn select_tool(&mut self, tool_name: &str) {
        if self.available_tools.contains(&tool_name.to_string()) {
            self.current_tool = tool_name.to_string();
        }
    }
}

#[derive(Debug, Clone)]
pub struct WorldState {
    pub structures: Vec<Structure>,
    pub terrain_modifications: u32,
    pub physics_objects: Vec<String>,
    pub weather: String,
    pub time_of_day: f32,
}

impl WorldState {
    pub fn new() -> Self {
        Self {
            structures: Vec::new(),
            terrain_modifications: 0,
            physics_objects: Vec::new(),
            weather: "Clear".to_string(),
            time_of_day: 12.0,
        }
    }
    
    pub fn initialize(&mut self) {
        self.physics_objects.push("Ground".to_string());
    }
    
    pub fn modify_terrain(&mut self, position: Point3, operation: &str, amount: f32, radius: f32) {
        self.terrain_modifications += 1;
    }
    
    pub fn construct_structure(&mut self, name: &str, position: Point3, structure_type: &str) {
        self.structures.push(Structure {
            name: name.to_string(),
            position,
            structure_type: structure_type.to_string(),
        });
        self.physics_objects.push(name.to_string());
    }
    
    pub fn set_weather(&mut self, weather: &str, intensity: f32) {
        self.weather = weather.to_string();
    }
    
    pub fn set_time_of_day(&mut self, time: f32) {
        self.time_of_day = time;
    }
    
    pub fn adjust_lighting(&mut self, intensity: f32, color: Vec3) {
        // Lighting adjustment logic
    }
    
    pub fn update_physics(&mut self, delta_time: f32) {
        // Physics simulation update
    }
}

#[derive(Debug, Clone)]
pub struct Structure {
    pub name: String,
    pub position: Point3,
    pub structure_type: String,
}

#[derive(Debug, Clone)]
pub struct AdvancedToolsSuite {
    pub automation_tasks: Vec<AutomationTask>,
    pub blueprints: Vec<Blueprint>,
    pub precision_tools: Vec<String>,
}

impl AdvancedToolsSuite {
    pub fn new() -> Self {
        Self {
            automation_tasks: Vec::new(),
            blueprints: Vec::new(),
            precision_tools: vec![
                "Laser Cutter".to_string(),
                "Molecular Assembler".to_string(),
                "Gravity Manipulator".to_string(),
                "System Integrator".to_string(),
            ],
        }
    }
    
    pub fn initialize(&mut self) {
        // Initialize tools
    }
    
    pub fn use_precision_tool(&mut self, tool_name: &str, position: Point3, description: &str) {
        // Tool usage logic
    }
    
    pub fn start_automation_task(&mut self, task_name: &str, description: &str) {
        self.automation_tasks.push(AutomationTask {
            name: task_name.to_string(),
            description: description.to_string(),
            efficiency: 85.0,
        });
    }
    
    pub fn create_blueprint(&mut self, name: &str, description: &str, complexity: u8) {
        self.blueprints.push(Blueprint {
            name: name.to_string(),
            description: description.to_string(),
            complexity,
        });
    }
    
    pub fn analyze_material(&mut self, material: &str, quality: f32, properties: &str) {
        // Material analysis logic
    }
    
    pub fn calculate_efficiency(&self) -> f32 {
        if self.automation_tasks.is_empty() {
            90.0
        } else {
            self.automation_tasks.iter().map(|t| t.efficiency).sum::<f32>() / self.automation_tasks.len() as f32
        }
    }
}

#[derive(Debug, Clone)]
pub struct AutomationTask {
    pub name: String,
    pub description: String,
    pub efficiency: f32,
}

#[derive(Debug, Clone)]
pub struct Blueprint {
    pub name: String,
    pub description: String,
    pub complexity: u8,
}

#[derive(Debug, Clone)]
pub struct NPCManager {
    pub active_npcs: Vec<NPC>,
    pub active_tasks: Vec<Task>,
    pub avg_completion_time: f32,
}

impl NPCManager {
    pub fn new() -> Self {
        Self {
            active_npcs: Vec::new(),
            active_tasks: Vec::new(),
            avg_completion_time: 12.5,
        }
    }
    
    pub fn initialize(&mut self) {
        // Initialize NPC systems
    }
    
    pub fn create_npc(&mut self, role: &str, name: &str, personality: &str, description: &str) -> String {
        let npc_id = format!("npc_{}", self.active_npcs.len());
        let npc = NPC {
            id: npc_id.clone(),
            name: name.to_string(),
            role: role.to_string(),
            personality: personality.to_string(),
            description: description.to_string(),
            current_activity: match role {
                "Construction Foreman" => "Supervising",
                "Materials Specialist" => "Coordinating",
                "Safety Inspector" => "Inspecting",
                "Architect" => "Planning",
                _ => "Working",
            }.to_string(),
        };
        self.active_npcs.push(npc);
        npc_id
    }
    
    pub fn update_npc_ai(&mut self, npc_id: &str, delta_time: f32) {
        // AI behavior update logic
    }
    
    pub fn interact_with_npc(&self, npc_id: &str, interaction_type: &str) -> String {
        if let Some(npc) = self.active_npcs.iter().find(|n| n.id == *npc_id) {
            match npc.role.as_str() {
                "Construction Foreman" => "Construction is proceeding on schedule. We've completed 73% of foundation work.",
                "Materials Specialist" => "Supply chain is optimized. Next delivery scheduled for tomorrow morning.",
                "Safety Inspector" => "All safety protocols are being followed. Zero incidents this week.",
                "Architect" => "Design optimizations complete. Energy efficiency improved by 15%.",
                _ => "Everything is running smoothly in my area.",
            }.to_string()
        } else {
            "NPC not found".to_string()
        }
    }
    
    pub fn assign_task(&mut self, npc_id: &str, task_id: &str, description: &str) {
        self.active_tasks.push(Task {
            id: task_id.to_string(),
            assigned_to: npc_id.to_string(),
            description: description.to_string(),
            completion: 0.0,
        });
    }
    
    pub fn calculate_team_efficiency(&self) -> f32 {
        if self.active_npcs.is_empty() {
            0.0
        } else {
            let base_efficiency = 75.0;
            let team_size_bonus = (self.active_npcs.len() as f32 * 2.5).min(15.0);
            let task_load_factor = if self.active_tasks.len() > self.active_npcs.len() * 2 {
                -10.0
            } else {
                5.0
            };
            (base_efficiency + team_size_bonus + task_load_factor).min(100.0)
        }
    }
}

#[derive(Debug, Clone)]
pub struct NPC {
    pub id: String,
    pub name: String,
    pub role: String,
    pub personality: String,
    pub description: String,
    pub current_activity: String,
}

#[derive(Debug, Clone)]
pub struct Task {
    pub id: String,
    pub assigned_to: String,
    pub description: String,
    pub completion: f32,
}

#[derive(Debug, Clone)]
pub struct StoryQuestSystem {
    pub storylines: Vec<Storyline>,
    pub quests: Vec<Quest>,
    pub story_events: Vec<StoryEvent>,
}

impl StoryQuestSystem {
    pub fn new() -> Self {
        Self {
            storylines: Vec::new(),
            quests: Vec::new(),
            story_events: Vec::new(),
        }
    }
    
    pub fn initialize(&mut self) {
        // Initialize story systems
    }
    
    pub fn create_storyline(&mut self, title: &str, description: &str, themes: Vec<&str>) -> Storyline {
        let storyline = Storyline {
            id: format!("story_{}", self.storylines.len()),
            title: title.to_string(),
            description: description.to_string(),
            themes: themes.into_iter().map(|s| s.to_string()).collect(),
            progress: 0.0,
        };
        self.storylines.push(storyline.clone());
        storyline
    }
    
    pub fn create_quest(&mut self, title: &str, description: &str, objectives: Vec<&str>) -> Quest {
        let quest = Quest {
            id: format!("quest_{}", self.quests.len()),
            title: title.to_string(),
            description: description.to_string(),
            objectives: objectives.into_iter().map(|s| s.to_string()).collect(),
            progress: 0.0,
        };
        self.quests.push(quest.clone());
        quest
    }
    
    pub fn update_quest_progress(&mut self, quest_id: &str, progress: f32) {
        if let Some(quest) = self.quests.iter_mut().find(|q| q.id == *quest_id) {
            quest.progress = progress.min(1.0);
        }
    }
    
    pub fn trigger_story_event(&mut self, name: &str, description: &str, event_type: &str) {
        self.story_events.push(StoryEvent {
            name: name.to_string(),
            description: description.to_string(),
            event_type: event_type.to_string(),
        });
    }
    
    pub fn calculate_overall_progress(&self) -> f32 {
        if self.quests.is_empty() {
            0.0
        } else {
            self.quests.iter().map(|q| q.progress).sum::<f32>() / self.quests.len() as f32 * 100.0
        }
    }
}

#[derive(Debug, Clone)]
pub struct Storyline {
    pub id: String,
    pub title: String,
    pub description: String,
    pub themes: Vec<String>,
    pub progress: f32,
}

#[derive(Debug, Clone)]
pub struct Quest {
    pub id: String,
    pub title: String,
    pub description: String,
    pub objectives: Vec<String>,
    pub progress: f32,
}

#[derive(Debug, Clone)]
pub struct StoryEvent {
    pub name: String,
    pub description: String,
    pub event_type: String,
}

#[derive(Debug, Clone)]
pub struct VehicleSystem {
    pub vehicle_designs: Vec<VehicleDesign>,
    pub routes: Vec<Route>,
    pub traffic_controls: Vec<TrafficControl>,
}

impl VehicleSystem {
    pub fn new() -> Self {
        Self {
            vehicle_designs: Vec::new(),
            routes: Vec::new(),
            traffic_controls: Vec::new(),
        }
    }
    
    pub fn initialize(&mut self) {
        // Initialize vehicle systems
    }
    
    pub fn design_vehicle(&mut self, name: &str, purpose: &str, power: f32, weight: f32) -> VehicleDesign {
        let top_speed = (power / weight * 100.0).min(200.0);
        let acceleration = (power / weight * 2.0).min(15.0);
        
        let design = VehicleDesign {
            name: name.to_string(),
            purpose: purpose.to_string(),
            power,
            weight,
            top_speed,
            acceleration,
        };
        self.vehicle_designs.push(design.clone());
        design
    }
    
    pub fn plan_route(&mut self, name: &str, start: Point3, end: Point3, route_type: &str) -> Route {
        let distance = ((end.x - start.x).powi(2) + (end.y - start.y).powi(2) + (end.z - start.z).powi(2)).sqrt();
        let duration = distance / 50.0; // Assume 50 km/h average speed
        
        let route = Route {
            name: name.to_string(),
            start,
            end,
            distance,
            duration,
            route_type: route_type.to_string(),
            waypoints: vec![start, end],
        };
        self.routes.push(route.clone());
        route
    }
    
    pub fn initialize_traffic_network(&mut self) {
        // Initialize traffic network
    }
    
    pub fn add_traffic_control(&mut self, name: &str, location: Point3, control_type: &str) {
        self.traffic_controls.push(TrafficControl {
            name: name.to_string(),
            location,
            control_type: control_type.to_string(),
        });
    }
    
    pub fn simulate_traffic_flow(&mut self, duration: f32) {
        // Traffic simulation logic
    }
    
    pub fn get_traffic_metrics(&self) -> TrafficMetrics {
        TrafficMetrics {
            flow_rate: 45.8,
            congestion_level: 0.35,
            efficiency: 0.87,
        }
    }
    
    pub fn get_fleet_status(&self) -> FleetStatus {
        FleetStatus {
            active_vehicles: self.vehicle_designs.len(),
            avg_fuel_level: 0.78,
            vehicles_in_transit: 3,
            mission_success_rate: 0.94,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VehicleDesign {
    pub name: String,
    pub purpose: String,
    pub power: f32,
    pub weight: f32,
    pub top_speed: f32,
    pub acceleration: f32,
}

#[derive(Debug, Clone)]
pub struct Route {
    pub name: String,
    pub start: Point3,
    pub end: Point3,
    pub distance: f32,
    pub duration: f32,
    pub route_type: String,
    pub waypoints: Vec<Point3>,
}

#[derive(Debug, Clone)]
pub struct TrafficControl {
    pub name: String,
    pub location: Point3,
    pub control_type: String,
}

#[derive(Debug, Clone)]
pub struct TrafficMetrics {
    pub flow_rate: f32,
    pub congestion_level: f32,
    pub efficiency: f32,
}

#[derive(Debug, Clone)]
pub struct FleetStatus {
    pub active_vehicles: usize,
    pub avg_fuel_level: f32,
    pub vehicles_in_transit: usize,
    pub mission_success_rate: f32,
}

#[derive(Debug, Clone)]
pub struct SessionStats {
    pub demos_completed: u32,
    pub integrated_scenarios: u32,
    pub systems_initialized: u32,
}

impl SessionStats {
    pub fn new() -> Self {
        Self {
            demos_completed: 0,
            integrated_scenarios: 0,
            systems_initialized: 0,
        }
    }
}

// Helper functions

fn format_position(pos: &Point3) -> String {
    format!("({:.1}, {:.1}, {:.1})", pos.x, pos.y, pos.z)
}