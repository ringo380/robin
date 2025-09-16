use crate::engine::math::{Vec3, Point3};
use crate::engine::npc::{NPC, DailyRoutine, ScheduleEntry, Occupation};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RoutineSystem {
    routine_templates: HashMap<String, RoutineTemplate>,
    dynamic_schedules: HashMap<String, DynamicSchedule>,
    routine_exceptions: HashMap<String, Vec<RoutineException>>,
    seasonal_adjustments: SeasonalAdjustments,
    workplace_schedules: HashMap<String, WorkplaceSchedule>,
}

#[derive(Debug, Clone)]
pub struct RoutineTemplate {
    pub template_name: String,
    pub occupation_type: Occupation,
    pub base_schedule: Vec<TemplateEntry>,
    pub flexibility_factor: f32,
    pub seasonal_variations: HashMap<String, Vec<ScheduleModification>>,
    pub personality_adjustments: HashMap<String, f32>,
}

#[derive(Debug, Clone)]
pub struct TemplateEntry {
    pub activity: String,
    pub start_time: u32,
    pub duration: u32,
    pub priority: f32,
    pub location_type: LocationType,
    pub energy_requirement: f32,
    pub mood_impact: f32,
    pub social_component: bool,
    pub weather_dependency: Option<WeatherRequirement>,
}

#[derive(Debug, Clone)]
pub enum LocationType {
    Home,
    Workplace,
    Community,
    Market,
    Recreation,
    Social,
    Travel,
    Any,
}

#[derive(Debug, Clone)]
pub struct WeatherRequirement {
    pub preferred_weather: Vec<String>,
    pub avoided_weather: Vec<String>,
    pub indoor_alternative: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DynamicSchedule {
    pub npc_id: String,
    pub current_schedule: Vec<ScheduleEntry>,
    pub adaptation_history: Vec<ScheduleAdaptation>,
    pub routine_satisfaction: f32,
    pub disruption_tolerance: f32,
    pub last_update: u64,
}

#[derive(Debug, Clone)]
pub struct ScheduleAdaptation {
    pub timestamp: u64,
    pub adaptation_type: AdaptationType,
    pub reason: String,
    pub success_rate: f32,
    pub satisfaction_impact: f32,
}

#[derive(Debug, Clone)]
pub enum AdaptationType {
    TimeShift,
    ActivityReplacement,
    LocationChange,
    DurationAdjustment,
    PriorityChange,
    SocialIntegration,
    EmergencyInterruption,
}

#[derive(Debug, Clone)]
pub struct RoutineException {
    pub exception_id: String,
    pub trigger: ExceptionTrigger,
    pub duration: ExceptionDuration,
    pub schedule_override: Vec<ScheduleEntry>,
    pub priority: f32,
    pub recurring: bool,
}

#[derive(Debug, Clone)]
pub enum ExceptionTrigger {
    SpecificDate(u32), // Day of year
    WeatherCondition(String),
    NPCState(String), // "sick", "stressed", etc.
    SocialEvent(String),
    ResourceAvailability(String, f32),
    EmergencyEvent,
}

#[derive(Debug, Clone)]
pub enum ExceptionDuration {
    OneTime,
    Daily(u32), // Number of days
    Weekly(u32), // Number of weeks
    Seasonal,
    UntilResolved,
}

#[derive(Debug, Clone)]
pub struct ScheduleModification {
    pub target_activity: String,
    pub modification_type: ModificationType,
    pub value: f32,
    pub condition: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ModificationType {
    TimeShift(i32), // Minutes to shift
    DurationChange(f32), // Multiplier
    PriorityAdjustment(f32),
    LocationReplacement(LocationType),
    ActivityReplacement(String),
    Remove,
    Add(TemplateEntry),
}

#[derive(Debug, Clone)]
pub struct SeasonalAdjustments {
    pub current_season: String,
    pub daylight_hours: (u32, u32), // Dawn, dusk in minutes from midnight
    pub temperature_factor: f32,
    pub activity_modifiers: HashMap<String, f32>,
}

#[derive(Debug, Clone)]
pub struct WorkplaceSchedule {
    pub location: Point3,
    pub operating_hours: (u32, u32), // Start, end time in minutes
    pub break_times: Vec<(u32, u32)>, // Break start, duration
    pub shift_patterns: Vec<ShiftPattern>,
    pub capacity: usize,
    pub current_workers: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ShiftPattern {
    pub shift_name: String,
    pub start_time: u32,
    pub end_time: u32,
    pub days_of_week: Vec<u32>, // 0-6, Sunday-Saturday
    pub required_skills: Vec<String>,
    pub max_workers: usize,
}

#[derive(Debug, Clone)]
pub struct RoutineMetrics {
    pub adherence_rate: f32,
    pub satisfaction_score: f32,
    pub efficiency_rating: f32,
    pub social_integration: f32,
    pub stress_factors: Vec<String>,
    pub improvement_suggestions: Vec<String>,
}

impl RoutineSystem {
    pub fn new() -> Self {
        let mut system = Self {
            routine_templates: HashMap::new(),
            dynamic_schedules: HashMap::new(),
            routine_exceptions: HashMap::new(),
            seasonal_adjustments: SeasonalAdjustments::default(),
            workplace_schedules: HashMap::new(),
        };
        
        system.initialize_routine_templates();
        system.initialize_workplace_schedules();
        system
    }

    fn initialize_routine_templates(&mut self) {
        // Builder routine template
        let builder_template = RoutineTemplate {
            template_name: "builder".to_string(),
            occupation_type: Occupation::Builder,
            base_schedule: vec![
                TemplateEntry {
                    activity: "wake_up".to_string(),
                    start_time: 360, // 6:00 AM
                    duration: 30,
                    priority: 9.0,
                    location_type: LocationType::Home,
                    energy_requirement: 0.0,
                    mood_impact: 0.1,
                    social_component: false,
                    weather_dependency: None,
                },
                TemplateEntry {
                    activity: "breakfast".to_string(),
                    start_time: 390, // 6:30 AM
                    duration: 30,
                    priority: 8.5,
                    location_type: LocationType::Home,
                    energy_requirement: 5.0,
                    mood_impact: 0.2,
                    social_component: false,
                    weather_dependency: None,
                },
                TemplateEntry {
                    activity: "travel_to_work".to_string(),
                    start_time: 420, // 7:00 AM
                    duration: 60,
                    priority: 8.0,
                    location_type: LocationType::Travel,
                    energy_requirement: 10.0,
                    mood_impact: -0.1,
                    social_component: false,
                    weather_dependency: Some(WeatherRequirement {
                        preferred_weather: vec!["sunny".to_string(), "cloudy".to_string()],
                        avoided_weather: vec!["stormy".to_string()],
                        indoor_alternative: None,
                    }),
                },
                TemplateEntry {
                    activity: "work_construction".to_string(),
                    start_time: 480, // 8:00 AM
                    duration: 480, // 8 hours
                    priority: 9.5,
                    location_type: LocationType::Workplace,
                    energy_requirement: 30.0,
                    mood_impact: 0.3,
                    social_component: true,
                    weather_dependency: Some(WeatherRequirement {
                        preferred_weather: vec!["sunny".to_string(), "cloudy".to_string()],
                        avoided_weather: vec!["stormy".to_string(), "rainy".to_string()],
                        indoor_alternative: Some("indoor_work".to_string()),
                    }),
                },
                TemplateEntry {
                    activity: "lunch_break".to_string(),
                    start_time: 720, // 12:00 PM
                    duration: 60,
                    priority: 8.0,
                    location_type: LocationType::Workplace,
                    energy_requirement: 5.0,
                    mood_impact: 0.2,
                    social_component: true,
                    weather_dependency: None,
                },
                TemplateEntry {
                    activity: "travel_home".to_string(),
                    start_time: 960, // 4:00 PM
                    duration: 60,
                    priority: 7.5,
                    location_type: LocationType::Travel,
                    energy_requirement: 10.0,
                    mood_impact: 0.1,
                    social_component: false,
                    weather_dependency: None,
                },
                TemplateEntry {
                    activity: "dinner".to_string(),
                    start_time: 1080, // 6:00 PM
                    duration: 45,
                    priority: 8.0,
                    location_type: LocationType::Home,
                    energy_requirement: 5.0,
                    mood_impact: 0.3,
                    social_component: false,
                    weather_dependency: None,
                },
                TemplateEntry {
                    activity: "evening_leisure".to_string(),
                    start_time: 1125, // 6:45 PM
                    duration: 135,
                    priority: 5.0,
                    location_type: LocationType::Home,
                    energy_requirement: -5.0,
                    mood_impact: 0.4,
                    social_component: false,
                    weather_dependency: None,
                },
                TemplateEntry {
                    activity: "sleep".to_string(),
                    start_time: 1320, // 10:00 PM
                    duration: 480, // 8 hours
                    priority: 9.0,
                    location_type: LocationType::Home,
                    energy_requirement: -60.0,
                    mood_impact: 0.2,
                    social_component: false,
                    weather_dependency: None,
                },
            ],
            flexibility_factor: 0.3,
            seasonal_variations: HashMap::new(),
            personality_adjustments: {
                let mut adjustments = HashMap::new();
                adjustments.insert("conscientiousness".to_string(), 0.2);
                adjustments.insert("extroversion".to_string(), -0.1);
                adjustments
            },
        };

        self.routine_templates.insert("builder".to_string(), builder_template);

        // Merchant routine template
        let merchant_template = RoutineTemplate {
            template_name: "merchant".to_string(),
            occupation_type: Occupation::Merchant,
            base_schedule: vec![
                TemplateEntry {
                    activity: "wake_up".to_string(),
                    start_time: 390, // 6:30 AM
                    duration: 30,
                    priority: 8.5,
                    location_type: LocationType::Home,
                    energy_requirement: 0.0,
                    mood_impact: 0.0,
                    social_component: false,
                    weather_dependency: None,
                },
                TemplateEntry {
                    activity: "prepare_goods".to_string(),
                    start_time: 420, // 7:00 AM
                    duration: 90,
                    priority: 9.0,
                    location_type: LocationType::Workplace,
                    energy_requirement: 15.0,
                    mood_impact: 0.1,
                    social_component: false,
                    weather_dependency: None,
                },
                TemplateEntry {
                    activity: "open_shop".to_string(),
                    start_time: 510, // 8:30 AM
                    duration: 480, // 8 hours
                    priority: 9.5,
                    location_type: LocationType::Market,
                    energy_requirement: 25.0,
                    mood_impact: 0.2,
                    social_component: true,
                    weather_dependency: None,
                },
                TemplateEntry {
                    activity: "evening_accounting".to_string(),
                    start_time: 1050, // 5:30 PM
                    duration: 60,
                    priority: 7.0,
                    location_type: LocationType::Workplace,
                    energy_requirement: 10.0,
                    mood_impact: -0.1,
                    social_component: false,
                    weather_dependency: None,
                },
                TemplateEntry {
                    activity: "dinner_social".to_string(),
                    start_time: 1140, // 7:00 PM
                    duration: 90,
                    priority: 6.0,
                    location_type: LocationType::Social,
                    energy_requirement: 5.0,
                    mood_impact: 0.4,
                    social_component: true,
                    weather_dependency: None,
                },
            ],
            flexibility_factor: 0.4,
            seasonal_variations: HashMap::new(),
            personality_adjustments: {
                let mut adjustments = HashMap::new();
                adjustments.insert("extroversion".to_string(), 0.3);
                adjustments.insert("agreeableness".to_string(), 0.2);
                adjustments
            },
        };

        self.routine_templates.insert("merchant".to_string(), merchant_template);

        // Add more occupation templates...
    }

    fn initialize_workplace_schedules(&mut self) {
        // Construction site schedule
        let construction_site = WorkplaceSchedule {
            location: Point3::new(100.0, 0.0, 50.0),
            operating_hours: (480, 960), // 8 AM to 4 PM
            break_times: vec![(600, 15), (720, 60), (840, 15)], // Morning, lunch, afternoon breaks
            shift_patterns: vec![
                ShiftPattern {
                    shift_name: "day_shift".to_string(),
                    start_time: 480,
                    end_time: 960,
                    days_of_week: vec![1, 2, 3, 4, 5], // Monday to Friday
                    required_skills: vec!["construction".to_string()],
                    max_workers: 10,
                },
            ],
            capacity: 10,
            current_workers: Vec::new(),
        };

        let location_key = format!("{:.1}_{:.1}_{:.1}", construction_site.location.x, construction_site.location.y, construction_site.location.z);
        self.workplace_schedules.insert(location_key, construction_site);

        // Market schedule
        let market = WorkplaceSchedule {
            location: Point3::new(50.0, 0.0, 100.0),
            operating_hours: (510, 1020), // 8:30 AM to 5 PM
            break_times: vec![(720, 30)], // Lunch break
            shift_patterns: vec![
                ShiftPattern {
                    shift_name: "market_day".to_string(),
                    start_time: 510,
                    end_time: 1020,
                    days_of_week: vec![1, 2, 3, 4, 5, 6], // Monday to Saturday
                    required_skills: vec!["commerce".to_string()],
                    max_workers: 5,
                },
            ],
            capacity: 5,
            current_workers: Vec::new(),
        };

        let location_key = format!("{:.1}_{:.1}_{:.1}", market.location.x, market.location.y, market.location.z);
        self.workplace_schedules.insert(location_key, market);
    }

    pub fn update(&mut self, npcs: &mut HashMap<String, NPC>, world_time: u32, weather: &str) {
        // Update seasonal adjustments
        self.update_seasonal_adjustments(world_time);
        
        // Update dynamic schedules for each NPC
        for (npc_id, npc) in npcs.iter_mut() {
            self.update_npc_schedule(npc, world_time, weather);
        }
        
        // Process routine exceptions
        self.process_routine_exceptions(npcs, world_time);
        
        // Update workplace schedules
        self.update_workplace_schedules(npcs, world_time);
    }

    fn update_seasonal_adjustments(&mut self, world_time: u32) {
        let day_of_year = world_time / 1440; // Assuming 1440 minutes per day
        
        // Simple seasonal calculation
        let season = match day_of_year % 365 {
            0..=90 => "spring",
            91..=180 => "summer",
            181..=270 => "autumn",
            _ => "winter",
        };
        
        self.seasonal_adjustments.current_season = season.to_string();
        
        // Adjust daylight hours based on season
        match season {
            "spring" => {
                self.seasonal_adjustments.daylight_hours = (360, 1080); // 6 AM to 6 PM
                self.seasonal_adjustments.temperature_factor = 0.7;
            },
            "summer" => {
                self.seasonal_adjustments.daylight_hours = (300, 1140); // 5 AM to 7 PM
                self.seasonal_adjustments.temperature_factor = 1.0;
            },
            "autumn" => {
                self.seasonal_adjustments.daylight_hours = (420, 1020); // 7 AM to 5 PM
                self.seasonal_adjustments.temperature_factor = 0.6;
            },
            "winter" => {
                self.seasonal_adjustments.daylight_hours = (450, 990); // 7:30 AM to 4:30 PM
                self.seasonal_adjustments.temperature_factor = 0.3;
            },
            _ => {},
        }
        
        // Update activity modifiers
        self.seasonal_adjustments.activity_modifiers.clear();
        match season {
            "summer" => {
                self.seasonal_adjustments.activity_modifiers.insert("outdoor_work".to_string(), 1.2);
                self.seasonal_adjustments.activity_modifiers.insert("social_gathering".to_string(), 1.3);
            },
            "winter" => {
                self.seasonal_adjustments.activity_modifiers.insert("indoor_activities".to_string(), 1.2);
                self.seasonal_adjustments.activity_modifiers.insert("home_time".to_string(), 1.4);
            },
            _ => {
                self.seasonal_adjustments.activity_modifiers.insert("balanced_activities".to_string(), 1.0);
            },
        }
    }

    fn update_npc_schedule(&mut self, npc: &mut NPC, world_time: u32, weather: &str) {
        // Get or create dynamic schedule
        let npc_id = npc.id.clone();
        
        if !self.dynamic_schedules.contains_key(&npc_id) {
            self.create_dynamic_schedule_for_npc(npc);
        }
        
        let needs_update = if let Some(dynamic_schedule) = self.dynamic_schedules.get(&npc_id) {
            let time_since_update = world_time.saturating_sub(dynamic_schedule.last_update as u32);
            time_since_update > 1440 // Update daily
        } else {
            false
        };

        if needs_update {
            if let Some(dynamic_schedule) = self.dynamic_schedules.get_mut(&npc_id) {
                Self::adapt_schedule_to_circumstances_static(dynamic_schedule, npc, world_time, weather);
                dynamic_schedule.last_update = world_time as u64;
            }
        }

        // Update NPC's routine from dynamic schedule
        if let Some(dynamic_schedule) = self.dynamic_schedules.get(&npc_id) {
            npc.daily_routine.schedule = dynamic_schedule.current_schedule.clone();
        }
    }

    fn create_dynamic_schedule_for_npc(&mut self, npc: &NPC) {
        let template_name = match npc.occupation {
            Occupation::Builder => "builder",
            Occupation::Merchant => "merchant",
            Occupation::Craftsperson => "builder", // Use builder template as fallback
            _ => "general",
        };

        let mut schedule_entries = Vec::new();
        
        if let Some(template) = self.routine_templates.get(template_name) {
            for template_entry in &template.base_schedule {
                let entry = ScheduleEntry {
                    start_time: template_entry.start_time / 60, // Convert to hours
                    duration: template_entry.duration,
                    activity: template_entry.activity.clone(),
                    location: match template_entry.location_type {
                        LocationType::Home => npc.home,
                        LocationType::Workplace => npc.workplace,
                        LocationType::Market => Some(Point3::new(50.0, 0.0, 100.0)), // Fixed market location
                        _ => None,
                    },
                    required_npcs: Vec::new(),
                    priority: template_entry.priority,
                    conditions: Vec::new(),
                };
                
                schedule_entries.push(entry);
            }
        }

        let dynamic_schedule = DynamicSchedule {
            npc_id: npc.id.clone(),
            current_schedule: schedule_entries,
            adaptation_history: Vec::new(),
            routine_satisfaction: 0.8,
            disruption_tolerance: npc.personality.traits.get("openness").unwrap_or(&0.5) * 0.5 + 0.25,
            last_update: 0,
        };

        self.dynamic_schedules.insert(npc.id.clone(), dynamic_schedule);
    }

    fn adapt_schedule_to_circumstances(&mut self, schedule: &mut DynamicSchedule, npc: &NPC, world_time: u32, weather: &str) {
        let mut adaptations_made = Vec::new();
        
        // Weather-based adaptations
        for entry in &mut schedule.current_schedule {
            if weather == "stormy" || weather == "heavy_rain" {
                match entry.activity.as_str() {
                    "work_construction" => {
                        // Move outdoor construction indoors or postpone
                        entry.activity = "indoor_work".to_string();
                        entry.duration = (entry.duration as f32 * 0.7) as u32; // Reduced efficiency
                        
                        adaptations_made.push(ScheduleAdaptation {
                            timestamp: world_time as u64,
                            adaptation_type: AdaptationType::ActivityReplacement,
                            reason: format!("Weather: {}", weather),
                            success_rate: 0.6,
                            satisfaction_impact: -0.1,
                        });
                    },
                    "travel_to_work" | "travel_home" => {
                        // Increase travel time in bad weather
                        entry.duration = (entry.duration as f32 * 1.5) as u32;
                        
                        adaptations_made.push(ScheduleAdaptation {
                            timestamp: world_time as u64,
                            adaptation_type: AdaptationType::DurationAdjustment,
                            reason: "Bad weather travel delay".to_string(),
                            success_rate: 0.9,
                            satisfaction_impact: -0.05,
                        });
                    },
                    _ => {},
                }
            }
        }
        
        // Personality-based adaptations
        let conscientiousness = npc.personality.traits.get("conscientiousness").unwrap_or(&0.5);
        let extroversion = npc.personality.traits.get("extroversion").unwrap_or(&0.5);
        
        if *extroversion > 0.7 {
            // Extroverted NPCs prefer more social activities
            for entry in &mut schedule.current_schedule {
                if entry.activity == "evening_leisure" {
                    entry.activity = "social_evening".to_string();
                    entry.location = Some(Point3::new(75.0, 0.0, 75.0)); // Social venue
                    
                    adaptations_made.push(ScheduleAdaptation {
                        timestamp: world_time as u64,
                        adaptation_type: AdaptationType::SocialIntegration,
                        reason: "High extroversion preference".to_string(),
                        success_rate: 0.8,
                        satisfaction_impact: 0.2,
                    });
                }
            }
        }
        
        if *conscientiousness > 0.7 {
            // Conscientious NPCs start work earlier and work longer
            for entry in &mut schedule.current_schedule {
                if entry.activity.contains("work") {
                    entry.start_time = entry.start_time.saturating_sub(1); // Start 1 hour earlier
                    entry.duration = (entry.duration as f32 * 1.1) as u32; // 10% longer
                    
                    adaptations_made.push(ScheduleAdaptation {
                        timestamp: world_time as u64,
                        adaptation_type: AdaptationType::TimeShift,
                        reason: "High conscientiousness".to_string(),
                        success_rate: 0.9,
                        satisfaction_impact: 0.1,
                    });
                }
            }
        }
        
        // Energy and mood-based adaptations
        if npc.energy < 30.0 {
            // Tired NPCs need more rest
            for entry in &mut schedule.current_schedule {
                if entry.activity == "sleep" {
                    entry.duration += 60; // Extra hour of sleep
                    
                    adaptations_made.push(ScheduleAdaptation {
                        timestamp: world_time as u64,
                        adaptation_type: AdaptationType::DurationAdjustment,
                        reason: "Low energy recovery".to_string(),
                        success_rate: 0.95,
                        satisfaction_impact: 0.15,
                    });
                }
            }
        }
        
        if npc.stress > 70.0 {
            // Stressed NPCs need stress relief activities
            let stress_relief_entry = ScheduleEntry {
                start_time: 19, // 7 PM
                duration: 60,
                activity: "stress_relief".to_string(),
                location: npc.home,
                required_npcs: Vec::new(),
                priority: 8.0,
                conditions: Vec::new(),
            };
            
            schedule.current_schedule.push(stress_relief_entry);
            
            adaptations_made.push(ScheduleAdaptation {
                timestamp: world_time as u64,
                adaptation_type: AdaptationType::ActivityReplacement,
                reason: "High stress level".to_string(),
                success_rate: 0.7,
                satisfaction_impact: 0.25,
            });
        }
        
        // Apply adaptations to history
        schedule.adaptation_history.extend(adaptations_made);
        
        // Update satisfaction based on adaptations
        let satisfaction_change: f32 = schedule.adaptation_history.iter()
            .filter(|a| a.timestamp > world_time.saturating_sub(1440) as u64)
            .map(|a| a.satisfaction_impact)
            .sum();
        
        schedule.routine_satisfaction = (schedule.routine_satisfaction + satisfaction_change * 0.1).clamp(0.0, 1.0);
        
        // Sort schedule by start time
        schedule.current_schedule.sort_by_key(|entry| entry.start_time);
    }

    fn adapt_schedule_to_circumstances_static(schedule: &mut DynamicSchedule, npc: &NPC, world_time: u32, weather: &str) {
        let mut adaptations_made = Vec::new();

        // Weather-based adaptations
        for entry in &mut schedule.current_schedule {
            if weather == "stormy" || weather == "heavy_rain" {
                match entry.activity.as_str() {
                    "work_construction" => {
                        // Move outdoor construction indoors or postpone
                        entry.activity = "indoor_work".to_string();
                        entry.duration = (entry.duration as f32 * 0.7) as u32; // Reduced efficiency

                        adaptations_made.push(ScheduleAdaptation {
                            timestamp: world_time as u64,
                            adaptation_type: AdaptationType::ActivityReplacement,
                            reason: format!("Weather: {}", weather),
                            success_rate: 0.6,
                            satisfaction_impact: -0.1,
                        });
                    },
                    "travel_to_work" | "travel_home" => {
                        // Increase travel time in bad weather
                        entry.duration = (entry.duration as f32 * 1.5) as u32;

                        adaptations_made.push(ScheduleAdaptation {
                            timestamp: world_time as u64,
                            adaptation_type: AdaptationType::DurationAdjustment,
                            reason: "Bad weather travel delay".to_string(),
                            success_rate: 0.9,
                            satisfaction_impact: -0.05,
                        });
                    },
                    _ => {},
                }
            }
        }

        // Personality-based adaptations
        let conscientiousness = npc.personality.traits.get("conscientiousness").unwrap_or(&0.5);
        let extroversion = npc.personality.traits.get("extroversion").unwrap_or(&0.5);

        if *extroversion > 0.7 {
            // Extroverted NPCs prefer more social activities
            for entry in &mut schedule.current_schedule {
                if entry.activity == "evening_leisure" {
                    entry.activity = "social_evening".to_string();
                    entry.location = Some(Point3::new(75.0, 0.0, 75.0)); // Social venue

                    adaptations_made.push(ScheduleAdaptation {
                        timestamp: world_time as u64,
                        adaptation_type: AdaptationType::SocialIntegration,
                        reason: "High extroversion preference".to_string(),
                        success_rate: 0.8,
                        satisfaction_impact: 0.2,
                    });
                }
            }
        }

        if *conscientiousness > 0.7 {
            // Conscientious NPCs start work earlier and work longer
            for entry in &mut schedule.current_schedule {
                if entry.activity.contains("work") {
                    entry.start_time = entry.start_time.saturating_sub(1); // Start 1 hour earlier
                    entry.duration = (entry.duration as f32 * 1.1) as u32; // 10% longer

                    adaptations_made.push(ScheduleAdaptation {
                        timestamp: world_time as u64,
                        adaptation_type: AdaptationType::TimeShift,
                        reason: "High conscientiousness".to_string(),
                        success_rate: 0.9,
                        satisfaction_impact: 0.1,
                    });
                }
            }
        }

        // Energy and mood-based adaptations
        if npc.energy < 30.0 {
            // Tired NPCs need more rest
            for entry in &mut schedule.current_schedule {
                if entry.activity == "sleep" {
                    entry.duration += 60; // Extra hour of sleep

                    adaptations_made.push(ScheduleAdaptation {
                        timestamp: world_time as u64,
                        adaptation_type: AdaptationType::DurationAdjustment,
                        reason: "Low energy recovery".to_string(),
                        success_rate: 0.95,
                        satisfaction_impact: 0.15,
                    });
                }
            }
        }

        if npc.stress > 70.0 {
            // Stressed NPCs need stress relief activities
            let stress_relief_entry = ScheduleEntry {
                start_time: 19, // 7 PM
                duration: 60,
                activity: "stress_relief".to_string(),
                location: npc.home,
                required_npcs: Vec::new(),
                priority: 8.0,
                conditions: Vec::new(),
            };

            schedule.current_schedule.push(stress_relief_entry);

            adaptations_made.push(ScheduleAdaptation {
                timestamp: world_time as u64,
                adaptation_type: AdaptationType::ActivityReplacement,
                reason: "High stress level".to_string(),
                success_rate: 0.7,
                satisfaction_impact: 0.25,
            });
        }

        // Apply adaptations to history
        schedule.adaptation_history.extend(adaptations_made);

        // Update satisfaction based on adaptations
        let satisfaction_change: f32 = schedule.adaptation_history.iter()
            .filter(|a| a.timestamp > world_time.saturating_sub(1440) as u64)
            .map(|a| a.satisfaction_impact)
            .sum();

        schedule.routine_satisfaction = (schedule.routine_satisfaction + satisfaction_change * 0.1).clamp(0.0, 1.0);

        // Sort schedule by start time
        schedule.current_schedule.sort_by_key(|entry| entry.start_time);
    }

    fn process_routine_exceptions(&mut self, npcs: &mut HashMap<String, NPC>, world_time: u32) {
        for (npc_id, exceptions) in &self.routine_exceptions {
            if let Some(npc) = npcs.get_mut(npc_id) {
                for exception in exceptions {
                    if self.should_apply_exception(exception, npc, world_time) {
                        self.apply_routine_exception(exception, npc);
                    }
                }
            }
        }
    }

    fn should_apply_exception(&self, exception: &RoutineException, npc: &NPC, world_time: u32) -> bool {
        match &exception.trigger {
            ExceptionTrigger::SpecificDate(day) => {
                let current_day = world_time / 1440;
                current_day == *day
            },
            ExceptionTrigger::WeatherCondition(condition) => {
                // Would check current weather against condition
                false // Simplified for now
            },
            ExceptionTrigger::NPCState(state) => {
                match state.as_str() {
                    "sick" => npc.health < 50.0,
                    "stressed" => npc.stress > 80.0,
                    "tired" => npc.energy < 20.0,
                    _ => false,
                }
            },
            ExceptionTrigger::EmergencyEvent => {
                // Would check for emergency conditions
                false
            },
            _ => false,
        }
    }

    fn apply_routine_exception(&self, exception: &RoutineException, npc: &mut NPC) {
        // Replace current routine with exception schedule
        npc.daily_routine.schedule = exception.schedule_override.clone();
        
        // Add memory of the exception
        npc.add_memory(
            format!("Schedule changed due to {}", exception.exception_id),
            0.3
        );
    }

    fn update_workplace_schedules(&mut self, npcs: &HashMap<String, NPC>, world_time: u32) {
        let locations: Vec<String> = self.workplace_schedules.keys().cloned().collect();
        for location in locations {
            if let Some(workplace) = self.workplace_schedules.get_mut(&location) {
                // Clear current workers
                workplace.current_workers.clear();

                // Find NPCs who should be at this workplace
                for (npc_id, npc) in npcs {
                    if let Some(work_location) = npc.workplace {
                        if Self::workplace_matches_location_static(&location, work_location) {
                            let current_hour = (world_time / 60) % 24;
                            let work_start = workplace.operating_hours.0 / 60;
                            let work_end = workplace.operating_hours.1 / 60;

                            if current_hour >= work_start && current_hour < work_end {
                                workplace.current_workers.push(npc_id.clone());
                            }
                        }
                    }
                }

                // Enforce capacity limits
                if workplace.current_workers.len() > workplace.capacity {
                    workplace.current_workers.truncate(workplace.capacity);
                }
            }
        }
    }

    fn locations_match(&self, loc1: Point3, loc2: Point3) -> bool {
        let distance = ((loc1.x - loc2.x).powi(2) +
                       (loc1.y - loc2.y).powi(2) +
                       (loc1.z - loc2.z).powi(2)).sqrt();
        distance < 10.0 // Within 10 units
    }

    fn workplace_matches_location(&self, location_name: &str, work_position: Point3) -> bool {
        Self::workplace_matches_location_static(location_name, work_position)
    }

    fn workplace_matches_location_static(location_name: &str, work_position: Point3) -> bool {
        // For now, we'll do a simple string-based check
        // In a more sophisticated implementation, we'd have a mapping from location names to positions
        // This is a stub implementation that always returns false to prevent type errors
        // TODO: Implement proper location name to position mapping
        false
    }

    // Public interface methods
    pub fn assign_routine_to_npc(&mut self, npc: &mut NPC, template_name: &str) {
        if let Some(template) = self.routine_templates.get(template_name) {
            let mut schedule = Vec::new();
            
            for template_entry in &template.base_schedule {
                let entry = ScheduleEntry {
                    start_time: template_entry.start_time / 60,
                    duration: template_entry.duration,
                    activity: template_entry.activity.clone(),
                    location: match template_entry.location_type {
                        LocationType::Home => npc.home,
                        LocationType::Workplace => npc.workplace,
                        _ => None,
                    },
                    required_npcs: Vec::new(),
                    priority: template_entry.priority,
                    conditions: Vec::new(),
                };
                
                schedule.push(entry);
            }
            
            npc.daily_routine.schedule = schedule;
            npc.daily_routine.flexibility = template.flexibility_factor;
        }
    }

    pub fn add_routine_exception(&mut self, npc_id: String, exception: RoutineException) {
        self.routine_exceptions.entry(npc_id)
            .or_insert_with(Vec::new)
            .push(exception);
    }

    pub fn get_routine_metrics(&self, npc_id: &str) -> Option<RoutineMetrics> {
        if let Some(schedule) = self.dynamic_schedules.get(npc_id) {
            let recent_adaptations: Vec<_> = schedule.adaptation_history.iter()
                .filter(|a| a.timestamp > (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() - 86400))
                .collect();
            
            let adherence_rate = if recent_adaptations.is_empty() {
                1.0
            } else {
                recent_adaptations.iter()
                    .map(|a| a.success_rate)
                    .sum::<f32>() / recent_adaptations.len() as f32
            };
            
            let stress_factors: Vec<String> = recent_adaptations.iter()
                .filter(|a| a.satisfaction_impact < 0.0)
                .map(|a| a.reason.clone())
                .collect();
            
            Some(RoutineMetrics {
                adherence_rate,
                satisfaction_score: schedule.routine_satisfaction,
                efficiency_rating: adherence_rate * schedule.routine_satisfaction,
                social_integration: 0.7, // Would calculate from social activities
                stress_factors,
                improvement_suggestions: self.generate_improvement_suggestions(schedule),
            })
        } else {
            None
        }
    }

    fn generate_improvement_suggestions(&self, schedule: &DynamicSchedule) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        if schedule.routine_satisfaction < 0.6 {
            suggestions.push("Consider adding more leisure activities".to_string());
        }
        
        if schedule.adaptation_history.iter()
            .filter(|a| matches!(a.adaptation_type, AdaptationType::EmergencyInterruption))
            .count() > 3 {
            suggestions.push("Build more flexibility into daily routine".to_string());
        }
        
        suggestions
    }
}

impl Default for SeasonalAdjustments {
    fn default() -> Self {
        Self {
            current_season: "spring".to_string(),
            daylight_hours: (360, 1080), // 6 AM to 6 PM
            temperature_factor: 0.7,
            activity_modifiers: HashMap::new(),
        }
    }
}