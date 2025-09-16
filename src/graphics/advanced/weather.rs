use crate::engine::error::RobinResult;
use serde::{Serialize, Deserialize};
use std::time::{Duration, Instant};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherConfig {
    pub enable_dynamic_weather: bool,
    pub enable_seasonal_changes: bool,
    pub weather_transition_speed: f32,
    pub precipitation_quality: PrecipitationQuality,
    pub fog_quality: FogQuality,
    pub cloud_quality: CloudQuality,
    pub enable_wind_effects: bool,
    pub enable_temperature_effects: bool,
}

impl Default for WeatherConfig {
    fn default() -> Self {
        Self {
            enable_dynamic_weather: true,
            enable_seasonal_changes: true,
            weather_transition_speed: 1.0,
            precipitation_quality: PrecipitationQuality::High,
            fog_quality: FogQuality::High,
            cloud_quality: CloudQuality::Medium,
            enable_wind_effects: true,
            enable_temperature_effects: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrecipitationQuality {
    Ultra,
    High,
    Medium,
    Low,
}

impl PrecipitationQuality {
    pub fn get_particle_count(&self) -> u32 {
        match self {
            PrecipitationQuality::Ultra => 50000,
            PrecipitationQuality::High => 25000,
            PrecipitationQuality::Medium => 10000,
            PrecipitationQuality::Low => 5000,
        }
    }

    pub fn get_collision_accuracy(&self) -> f32 {
        match self {
            PrecipitationQuality::Ultra => 1.0,
            PrecipitationQuality::High => 0.8,
            PrecipitationQuality::Medium => 0.5,
            PrecipitationQuality::Low => 0.2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FogQuality {
    High,
    Medium,
    Low,
    Off,
}

impl FogQuality {
    pub fn get_volumetric_steps(&self) -> u32 {
        match self {
            FogQuality::High => 128,
            FogQuality::Medium => 64,
            FogQuality::Low => 32,
            FogQuality::Off => 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CloudQuality {
    Ultra,
    High,
    Medium,
    Low,
}

impl CloudQuality {
    pub fn get_cloud_layers(&self) -> u32 {
        match self {
            CloudQuality::Ultra => 4,
            CloudQuality::High => 3,
            CloudQuality::Medium => 2,
            CloudQuality::Low => 1,
        }
    }

    pub fn get_noise_resolution(&self) -> u32 {
        match self {
            CloudQuality::Ultra => 512,
            CloudQuality::High => 256,
            CloudQuality::Medium => 128,
            CloudQuality::Low => 64,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WeatherType {
    Clear,
    Overcast,
    Rain,
    HeavyRain,
    Snow,
    HeavySnow,
    Fog,
    Thunderstorm,
    Hail,
    Sandstorm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherState {
    pub current_weather: WeatherType,
    pub weather_intensity: f32, // 0.0 to 1.0
    pub transition_progress: f32, // 0.0 to 1.0
    pub target_weather: WeatherType,
    
    // Time and season
    pub time_of_day: f32, // 0.0 to 24.0 hours
    pub day_of_year: u32, // 1 to 365
    pub season: Season,
    
    // Atmospheric conditions
    pub temperature: f32, // Celsius
    pub humidity: f32, // 0.0 to 1.0
    pub pressure: f32, // millibars
    pub visibility: f32, // kilometers
    
    // Wind
    pub wind_direction: [f32; 3], // normalized direction vector
    pub wind_speed: f32, // m/s
    pub wind_turbulence: f32, // 0.0 to 1.0
    
    // Precipitation
    pub precipitation_amount: f32, // mm/hour
    pub precipitation_type: PrecipitationType,
    
    // Lighting
    pub sun_azimuth: f32, // degrees
    pub sun_elevation: f32, // degrees
    pub cloud_coverage: f32, // 0.0 to 1.0
    
    // Fog
    pub fog_density: f32, // 0.0 to 1.0
    pub fog_height: f32, // meters
    pub fog_color: [f32; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrecipitationType {
    None,
    Rain,
    Snow,
    Sleet,
    Hail,
}

impl Default for WeatherState {
    fn default() -> Self {
        Self {
            current_weather: WeatherType::Clear,
            weather_intensity: 0.0,
            transition_progress: 1.0,
            target_weather: WeatherType::Clear,
            
            time_of_day: 12.0,
            day_of_year: 180,
            season: Season::Summer,
            
            temperature: 20.0,
            humidity: 0.5,
            pressure: 1013.25,
            visibility: 10.0,
            
            wind_direction: [1.0, 0.0, 0.0],
            wind_speed: 5.0,
            wind_turbulence: 0.1,
            
            precipitation_amount: 0.0,
            precipitation_type: PrecipitationType::None,
            
            sun_azimuth: 180.0,
            sun_elevation: 45.0,
            cloud_coverage: 0.3,
            
            fog_density: 0.0,
            fog_height: 10.0,
            fog_color: [0.8, 0.8, 0.9],
        }
    }
}

#[derive(Debug, Clone)]
pub struct PrecipitationParticle {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub size: f32,
    pub life_time: f32,
    pub max_life_time: f32,
    pub particle_type: PrecipitationType,
}

#[derive(Debug)]
pub struct CloudLayer {
    pub altitude: f32,
    pub thickness: f32,
    pub density: f32,
    pub coverage: f32,
    pub speed: f32,
    pub direction: [f32; 2],
    pub noise_scale: f32,
    pub noise_offset: [f32; 3],
}

#[derive(Debug)]
pub struct AtmosphericScattering {
    pub rayleigh_coefficient: [f32; 3],
    pub mie_coefficient: f32,
    pub mie_direction: f32,
    pub turbidity: f32,
    pub sun_intensity: f32,
}

#[derive(Debug)]
pub struct WeatherSystem {
    config: WeatherConfig,
    current_state: WeatherState,
    weather_history: Vec<(Instant, WeatherState)>,
    precipitation_particles: Vec<PrecipitationParticle>,
    cloud_layers: Vec<CloudLayer>,
    atmospheric_scattering: AtmosphericScattering,
    
    // Weather progression
    weather_patterns: HashMap<Season, Vec<WeatherPattern>>,
    last_weather_change: Instant,
    next_weather_change: Instant,
    
    // Performance tracking
    particle_pool: Vec<PrecipitationParticle>,
    active_particles: usize,
    
    // Environmental effects
    temperature_map: HashMap<[i32; 2], f32>, // Grid-based temperature
    humidity_map: HashMap<[i32; 2], f32>, // Grid-based humidity
}

#[derive(Debug, Clone)]
pub struct WeatherPattern {
    pub weather_type: WeatherType,
    pub probability: f32,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub temperature_range: [f32; 2],
}

impl WeatherSystem {
    pub fn new(config: WeatherConfig) -> RobinResult<Self> {
        let mut system = Self {
            config,
            current_state: WeatherState::default(),
            weather_history: Vec::with_capacity(100),
            precipitation_particles: Vec::new(),
            cloud_layers: Vec::new(),
            atmospheric_scattering: AtmosphericScattering {
                rayleigh_coefficient: [0.0000055, 0.0000130, 0.0000224],
                mie_coefficient: 0.000021,
                mie_direction: 0.758,
                turbidity: 2.0,
                sun_intensity: 20.0,
            },
            weather_patterns: HashMap::new(),
            last_weather_change: Instant::now(),
            next_weather_change: Instant::now() + Duration::from_secs(300), // 5 minutes
            particle_pool: Vec::with_capacity(50000),
            active_particles: 0,
            temperature_map: HashMap::new(),
            humidity_map: HashMap::new(),
        };

        system.initialize_weather_patterns()?;
        system.initialize_cloud_layers()?;
        system.initialize_particle_pool()?;
        
        Ok(system)
    }

    pub fn update(&mut self, delta_time: f32, world_bounds: [f32; 6]) -> RobinResult<()> {
        // Update time of day
        self.update_time_of_day(delta_time);
        
        // Update seasonal progression
        self.update_season();
        
        // Check for weather transitions
        if self.config.enable_dynamic_weather {
            self.update_weather_transition(delta_time)?;
        }
        
        // Update atmospheric conditions
        self.update_atmospheric_conditions();
        
        // Update sun and lighting
        self.update_sun_position();
        
        // Update cloud layers
        self.update_clouds(delta_time);
        
        // Update precipitation
        if matches!(self.current_state.precipitation_type, PrecipitationType::Rain | PrecipitationType::Snow | PrecipitationType::Hail) {
            self.update_precipitation(delta_time, world_bounds)?;
        }
        
        // Update wind effects
        if self.config.enable_wind_effects {
            self.update_wind_effects(delta_time);
        }
        
        // Update temperature and humidity maps
        if self.config.enable_temperature_effects {
            self.update_environmental_maps();
        }
        
        Ok(())
    }

    pub fn set_weather(&mut self, weather_type: WeatherType, transition_duration: f32) -> RobinResult<()> {
        self.current_state.target_weather = weather_type;
        self.current_state.transition_progress = 0.0;
        
        // Calculate transition speed
        let transition_speed = if transition_duration > 0.0 {
            1.0 / transition_duration
        } else {
            10.0 // Instant transition
        };
        
        println!("Weather changing to {:?} over {:.1} seconds", self.current_state.target_weather, transition_duration);
        Ok(())
    }

    pub fn set_time_of_day(&mut self, hour: f32) {
        self.current_state.time_of_day = hour.clamp(0.0, 24.0);
        self.update_sun_position();
    }

    pub fn set_season(&mut self, season: Season) {
        self.current_state.season = season;
        self.update_atmospheric_conditions();
    }

    pub fn get_weather_state(&self) -> &WeatherState {
        &self.current_state
    }

    pub fn get_precipitation_particles(&self) -> &[PrecipitationParticle] {
        &self.precipitation_particles[..self.active_particles]
    }

    pub fn get_cloud_layers(&self) -> &[CloudLayer] {
        &self.cloud_layers
    }

    pub fn get_atmospheric_scattering(&self) -> &AtmosphericScattering {
        &self.atmospheric_scattering
    }

    pub fn get_wind_at_position(&self, position: [f32; 3]) -> [f32; 3] {
        let base_wind = [
            self.current_state.wind_direction[0] * self.current_state.wind_speed,
            self.current_state.wind_direction[1] * self.current_state.wind_speed,
            self.current_state.wind_direction[2] * self.current_state.wind_speed,
        ];
        
        // Add turbulence based on position
        let turbulence_factor = self.current_state.wind_turbulence;
        let turbulence = [
            (position[0] * 0.1).sin() * turbulence_factor,
            (position[1] * 0.1).cos() * turbulence_factor,
            (position[2] * 0.1).sin() * turbulence_factor,
        ];
        
        [
            base_wind[0] + turbulence[0],
            base_wind[1] + turbulence[1],
            base_wind[2] + turbulence[2],
        ]
    }

    pub fn get_temperature_at_position(&self, position: [f32; 3]) -> f32 {
        if !self.config.enable_temperature_effects {
            return self.current_state.temperature;
        }
        
        let grid_pos = [
            (position[0] / 100.0) as i32,
            (position[2] / 100.0) as i32,
        ];
        
        self.temperature_map.get(&grid_pos)
            .copied()
            .unwrap_or(self.current_state.temperature)
    }

    fn initialize_weather_patterns(&mut self) -> RobinResult<()> {
        // Spring patterns
        let spring_patterns = vec![
            WeatherPattern {
                weather_type: WeatherType::Clear,
                probability: 0.4,
                min_duration: Duration::from_secs(3600), // 1 hour
                max_duration: Duration::from_secs(14400), // 4 hours
                temperature_range: [10.0, 20.0],
            },
            WeatherPattern {
                weather_type: WeatherType::Overcast,
                probability: 0.3,
                min_duration: Duration::from_secs(1800),
                max_duration: Duration::from_secs(7200),
                temperature_range: [8.0, 18.0],
            },
            WeatherPattern {
                weather_type: WeatherType::Rain,
                probability: 0.25,
                min_duration: Duration::from_secs(900),
                max_duration: Duration::from_secs(3600),
                temperature_range: [5.0, 15.0],
            },
            WeatherPattern {
                weather_type: WeatherType::Thunderstorm,
                probability: 0.05,
                min_duration: Duration::from_secs(600),
                max_duration: Duration::from_secs(1800),
                temperature_range: [12.0, 22.0],
            },
        ];
        self.weather_patterns.insert(Season::Spring, spring_patterns);

        // Summer patterns
        let summer_patterns = vec![
            WeatherPattern {
                weather_type: WeatherType::Clear,
                probability: 0.6,
                min_duration: Duration::from_secs(7200),
                max_duration: Duration::from_secs(28800),
                temperature_range: [20.0, 35.0],
            },
            WeatherPattern {
                weather_type: WeatherType::Overcast,
                probability: 0.2,
                min_duration: Duration::from_secs(3600),
                max_duration: Duration::from_secs(10800),
                temperature_range: [18.0, 30.0],
            },
            WeatherPattern {
                weather_type: WeatherType::Thunderstorm,
                probability: 0.15,
                min_duration: Duration::from_secs(1200),
                max_duration: Duration::from_secs(3600),
                temperature_range: [22.0, 32.0],
            },
            WeatherPattern {
                weather_type: WeatherType::Rain,
                probability: 0.05,
                min_duration: Duration::from_secs(600),
                max_duration: Duration::from_secs(1800),
                temperature_range: [15.0, 25.0],
            },
        ];
        self.weather_patterns.insert(Season::Summer, summer_patterns);

        // Autumn patterns
        let autumn_patterns = vec![
            WeatherPattern {
                weather_type: WeatherType::Overcast,
                probability: 0.4,
                min_duration: Duration::from_secs(3600),
                max_duration: Duration::from_secs(14400),
                temperature_range: [5.0, 15.0],
            },
            WeatherPattern {
                weather_type: WeatherType::Rain,
                probability: 0.3,
                min_duration: Duration::from_secs(1800),
                max_duration: Duration::from_secs(7200),
                temperature_range: [3.0, 12.0],
            },
            WeatherPattern {
                weather_type: WeatherType::Clear,
                probability: 0.2,
                min_duration: Duration::from_secs(1800),
                max_duration: Duration::from_secs(7200),
                temperature_range: [8.0, 18.0],
            },
            WeatherPattern {
                weather_type: WeatherType::Fog,
                probability: 0.1,
                min_duration: Duration::from_secs(3600),
                max_duration: Duration::from_secs(10800),
                temperature_range: [2.0, 8.0],
            },
        ];
        self.weather_patterns.insert(Season::Autumn, autumn_patterns);

        // Winter patterns
        let winter_patterns = vec![
            WeatherPattern {
                weather_type: WeatherType::Overcast,
                probability: 0.4,
                min_duration: Duration::from_secs(7200),
                max_duration: Duration::from_secs(21600),
                temperature_range: [-5.0, 5.0],
            },
            WeatherPattern {
                weather_type: WeatherType::Snow,
                probability: 0.3,
                min_duration: Duration::from_secs(3600),
                max_duration: Duration::from_secs(14400),
                temperature_range: [-10.0, 2.0],
            },
            WeatherPattern {
                weather_type: WeatherType::Clear,
                probability: 0.2,
                min_duration: Duration::from_secs(1800),
                max_duration: Duration::from_secs(7200),
                temperature_range: [-5.0, 8.0],
            },
            WeatherPattern {
                weather_type: WeatherType::HeavySnow,
                probability: 0.1,
                min_duration: Duration::from_secs(1800),
                max_duration: Duration::from_secs(7200),
                temperature_range: [-15.0, -2.0],
            },
        ];
        self.weather_patterns.insert(Season::Winter, winter_patterns);

        Ok(())
    }

    fn initialize_cloud_layers(&mut self) -> RobinResult<()> {
        let layer_count = self.config.cloud_quality.get_cloud_layers();
        
        for i in 0..layer_count {
            let layer = CloudLayer {
                altitude: 2000.0 + (i as f32 * 1500.0),
                thickness: 300.0 + (i as f32 * 200.0),
                density: 0.3 - (i as f32 * 0.05),
                coverage: self.current_state.cloud_coverage,
                speed: 5.0 + (i as f32 * 2.0),
                direction: [1.0, 0.2 * i as f32],
                noise_scale: 0.001 * (1.0 + i as f32 * 0.3),
                noise_offset: [i as f32 * 100.0, 0.0, i as f32 * 50.0],
            };
            self.cloud_layers.push(layer);
        }
        
        Ok(())
    }

    fn initialize_particle_pool(&mut self) -> RobinResult<()> {
        let max_particles = self.config.precipitation_quality.get_particle_count() as usize;
        
        for _ in 0..max_particles {
            self.particle_pool.push(PrecipitationParticle {
                position: [0.0, 0.0, 0.0],
                velocity: [0.0, 0.0, 0.0],
                size: 1.0,
                life_time: 0.0,
                max_life_time: 1.0,
                particle_type: PrecipitationType::Rain,
            });
        }
        
        Ok(())
    }

    fn update_time_of_day(&mut self, delta_time: f32) {
        self.current_state.time_of_day += delta_time / 3600.0; // Convert seconds to hours
        
        if self.current_state.time_of_day >= 24.0 {
            self.current_state.time_of_day -= 24.0;
            self.current_state.day_of_year += 1;
            
            if self.current_state.day_of_year > 365 {
                self.current_state.day_of_year = 1;
            }
        }
    }

    fn update_season(&mut self) {
        let new_season = match self.current_state.day_of_year {
            1..=79 => Season::Winter,
            80..=171 => Season::Spring,
            172..=265 => Season::Summer,
            266..=355 => Season::Autumn,
            356..=365 => Season::Winter,
            _ => Season::Spring,
        };
        
        if !matches!(self.current_state.season, new_season) {
            self.current_state.season = new_season;
            println!("Season changed to {:?}", self.current_state.season);
        }
    }

    fn update_weather_transition(&mut self, delta_time: f32) -> RobinResult<()> {
        let now = Instant::now();
        
        // Check if it's time for a weather change
        if now >= self.next_weather_change && self.current_state.transition_progress >= 1.0 {
            self.initiate_weather_change()?;
        }
        
        // Update transition progress
        if self.current_state.transition_progress < 1.0 {
            self.current_state.transition_progress += delta_time * self.config.weather_transition_speed / 60.0;
            self.current_state.transition_progress = self.current_state.transition_progress.min(1.0);
            
            if self.current_state.transition_progress >= 1.0 {
                self.current_state.current_weather = self.current_state.target_weather.clone();
                self.apply_weather_effects();
                println!("Weather transition completed: {:?}", self.current_state.current_weather);
            }
        }
        
        Ok(())
    }

    fn initiate_weather_change(&mut self) -> RobinResult<()> {
        if let Some(patterns) = self.weather_patterns.get(&self.current_state.season) {
            // Choose next weather pattern based on probabilities
            let mut cumulative_probability = 0.0;
            let random_value: f32 = 0.5; // Mock random value
            
            for pattern in patterns {
                cumulative_probability += pattern.probability;
                if random_value <= cumulative_probability {
                    self.current_state.target_weather = pattern.weather_type.clone();
                    self.current_state.transition_progress = 0.0;
                    
                    // Set next weather change time
                    let duration_range = pattern.max_duration.as_secs() - pattern.min_duration.as_secs();
                    let duration = Duration::from_secs(pattern.min_duration.as_secs() + (duration_range as f32 * 0.5) as u64);
                    self.next_weather_change = Instant::now() + duration;
                    
                    break;
                }
            }
        }
        
        Ok(())
    }

    fn apply_weather_effects(&mut self) {
        match self.current_state.current_weather {
            WeatherType::Clear => {
                self.current_state.cloud_coverage = 0.1;
                self.current_state.precipitation_type = PrecipitationType::None;
                self.current_state.precipitation_amount = 0.0;
                self.current_state.fog_density = 0.0;
                self.current_state.visibility = 15.0;
            },
            WeatherType::Overcast => {
                self.current_state.cloud_coverage = 0.8;
                self.current_state.precipitation_type = PrecipitationType::None;
                self.current_state.precipitation_amount = 0.0;
                self.current_state.fog_density = 0.1;
                self.current_state.visibility = 8.0;
            },
            WeatherType::Rain => {
                self.current_state.cloud_coverage = 0.9;
                self.current_state.precipitation_type = PrecipitationType::Rain;
                self.current_state.precipitation_amount = 2.5;
                self.current_state.fog_density = 0.2;
                self.current_state.visibility = 3.0;
            },
            WeatherType::HeavyRain => {
                self.current_state.cloud_coverage = 1.0;
                self.current_state.precipitation_type = PrecipitationType::Rain;
                self.current_state.precipitation_amount = 10.0;
                self.current_state.fog_density = 0.3;
                self.current_state.visibility = 1.0;
            },
            WeatherType::Snow => {
                self.current_state.cloud_coverage = 0.8;
                self.current_state.precipitation_type = PrecipitationType::Snow;
                self.current_state.precipitation_amount = 1.0;
                self.current_state.fog_density = 0.1;
                self.current_state.visibility = 5.0;
            },
            WeatherType::HeavySnow => {
                self.current_state.cloud_coverage = 1.0;
                self.current_state.precipitation_type = PrecipitationType::Snow;
                self.current_state.precipitation_amount = 5.0;
                self.current_state.fog_density = 0.4;
                self.current_state.visibility = 0.5;
            },
            WeatherType::Fog => {
                self.current_state.cloud_coverage = 0.6;
                self.current_state.precipitation_type = PrecipitationType::None;
                self.current_state.precipitation_amount = 0.0;
                self.current_state.fog_density = 0.8;
                self.current_state.visibility = 0.2;
            },
            WeatherType::Thunderstorm => {
                self.current_state.cloud_coverage = 1.0;
                self.current_state.precipitation_type = PrecipitationType::Rain;
                self.current_state.precipitation_amount = 15.0;
                self.current_state.fog_density = 0.2;
                self.current_state.visibility = 2.0;
                self.current_state.wind_speed = 15.0;
                self.current_state.wind_turbulence = 0.8;
            },
            WeatherType::Hail => {
                self.current_state.cloud_coverage = 1.0;
                self.current_state.precipitation_type = PrecipitationType::Hail;
                self.current_state.precipitation_amount = 8.0;
                self.current_state.fog_density = 0.1;
                self.current_state.visibility = 3.0;
            },
            WeatherType::Sandstorm => {
                self.current_state.cloud_coverage = 0.3;
                self.current_state.precipitation_type = PrecipitationType::None;
                self.current_state.precipitation_amount = 0.0;
                self.current_state.fog_density = 0.6;
                self.current_state.visibility = 0.5;
                self.current_state.wind_speed = 25.0;
                self.current_state.wind_turbulence = 1.0;
            },
        }
        
        // Update cloud layers
        for layer in &mut self.cloud_layers {
            layer.coverage = self.current_state.cloud_coverage;
        }
    }

    fn update_atmospheric_conditions(&mut self) {
        // Update temperature based on season, time of day, and weather
        let base_temp = match self.current_state.season {
            Season::Spring => 15.0,
            Season::Summer => 25.0,
            Season::Autumn => 10.0,
            Season::Winter => 0.0,
        };
        
        // Daily temperature variation
        let hour_factor = (self.current_state.time_of_day - 6.0) / 12.0; // Peak at noon
        let daily_variation = (hour_factor * std::f32::consts::PI).sin() * 8.0;
        
        // Weather modifier
        let weather_modifier = match self.current_state.current_weather {
            WeatherType::Clear => 2.0,
            WeatherType::Overcast => -1.0,
            WeatherType::Rain | WeatherType::HeavyRain => -5.0,
            WeatherType::Snow | WeatherType::HeavySnow => -10.0,
            WeatherType::Thunderstorm => 3.0,
            WeatherType::Fog => -3.0,
            WeatherType::Hail => -8.0,
            WeatherType::Sandstorm => 5.0,
        };
        
        self.current_state.temperature = base_temp + daily_variation + weather_modifier;
        
        // Update humidity
        self.current_state.humidity = match self.current_state.current_weather {
            WeatherType::Clear => 0.3,
            WeatherType::Overcast => 0.6,
            WeatherType::Rain | WeatherType::HeavyRain => 0.9,
            WeatherType::Snow | WeatherType::HeavySnow => 0.8,
            WeatherType::Thunderstorm => 0.95,
            WeatherType::Fog => 0.98,
            WeatherType::Hail => 0.85,
            WeatherType::Sandstorm => 0.1,
        };
        
        // Update pressure
        self.current_state.pressure = 1013.25 + match self.current_state.current_weather {
            WeatherType::Clear => 5.0,
            WeatherType::Overcast => 0.0,
            WeatherType::Rain | WeatherType::HeavyRain => -8.0,
            WeatherType::Thunderstorm => -15.0,
            WeatherType::Snow | WeatherType::HeavySnow => -5.0,
            WeatherType::Fog => -3.0,
            WeatherType::Hail => -12.0,
            WeatherType::Sandstorm => 2.0,
        };
    }

    fn update_sun_position(&mut self) {
        // Calculate sun position based on time of day and day of year
        let hour_angle = (self.current_state.time_of_day - 12.0) * 15.0; // 15 degrees per hour
        
        // Solar declination (seasonal variation)
        let day_angle = 2.0 * std::f32::consts::PI * self.current_state.day_of_year as f32 / 365.0;
        let declination = 23.45 * (day_angle - 81.0 * std::f32::consts::PI / 180.0).sin();
        
        // Assume latitude of 45 degrees for calculation
        let latitude = 45.0;
        
        self.current_state.sun_elevation = (
            latitude.to_radians().sin() * declination.to_radians().sin() +
            latitude.to_radians().cos() * declination.to_radians().cos() * hour_angle.to_radians().cos()
        ).asin().to_degrees();
        
        self.current_state.sun_azimuth = (
            declination.to_radians().sin() * latitude.to_radians().cos() -
            declination.to_radians().cos() * latitude.to_radians().sin() * hour_angle.to_radians().cos()
        ).atan2(hour_angle.to_radians().sin()).to_degrees() + 180.0;
        
        // Ensure sun elevation doesn't go below -18 degrees (astronomical twilight)
        self.current_state.sun_elevation = self.current_state.sun_elevation.max(-18.0);
    }

    fn update_clouds(&mut self, delta_time: f32) {
        for layer in &mut self.cloud_layers {
            // Move clouds based on wind
            layer.noise_offset[0] += layer.direction[0] * layer.speed * delta_time;
            layer.noise_offset[2] += layer.direction[1] * layer.speed * delta_time;
            
            // Update coverage based on weather transition
            let target_coverage = self.current_state.cloud_coverage;
            layer.coverage += (target_coverage - layer.coverage) * delta_time * 0.1;
        }
    }

    fn update_precipitation(&mut self, delta_time: f32, world_bounds: [f32; 6]) -> RobinResult<()> {
        let target_particles = (self.config.precipitation_quality.get_particle_count() as f32 * 
                              self.current_state.precipitation_amount / 10.0) as usize;
        
        // Spawn new particles
        while self.active_particles < target_particles && self.active_particles < self.particle_pool.len() {
            let particle = &mut self.particle_pool[self.active_particles];
            
            // Random spawn position above world bounds
            particle.position = [
                world_bounds[0] + (world_bounds[3] - world_bounds[0]) * 0.5, // Mock random
                world_bounds[4] + 100.0, // 100m above max height
                world_bounds[2] + (world_bounds[5] - world_bounds[2]) * 0.3, // Mock random
            ];
            
            // Set velocity based on precipitation type
            match self.current_state.precipitation_type {
                PrecipitationType::Rain => {
                    particle.velocity = [
                        self.current_state.wind_direction[0] * 2.0,
                        -8.0, // Terminal velocity for raindrops
                        self.current_state.wind_direction[2] * 2.0,
                    ];
                    particle.size = 0.5;
                    particle.max_life_time = 20.0;
                },
                PrecipitationType::Snow => {
                    particle.velocity = [
                        self.current_state.wind_direction[0] * 3.0,
                        -2.0, // Slower fall for snow
                        self.current_state.wind_direction[2] * 3.0,
                    ];
                    particle.size = 2.0;
                    particle.max_life_time = 50.0;
                },
                PrecipitationType::Hail => {
                    particle.velocity = [
                        self.current_state.wind_direction[0] * 1.0,
                        -15.0, // Fast fall for hail
                        self.current_state.wind_direction[2] * 1.0,
                    ];
                    particle.size = 3.0;
                    particle.max_life_time = 15.0;
                },
                _ => {},
            }
            
            particle.particle_type = self.current_state.precipitation_type.clone();
            particle.life_time = 0.0;
            self.active_particles += 1;
        }
        
        // Update existing particles
        let mut particles_to_remove = Vec::new();
        
        for i in 0..self.active_particles {
            let particle = &mut self.particle_pool[i];
            
            // Update position
            particle.position[0] += particle.velocity[0] * delta_time;
            particle.position[1] += particle.velocity[1] * delta_time;
            particle.position[2] += particle.velocity[2] * delta_time;
            
            // Update lifetime
            particle.life_time += delta_time;
            
            // Check for removal conditions
            if particle.life_time >= particle.max_life_time || 
               particle.position[1] < world_bounds[1] {
                particles_to_remove.push(i);
            }
        }
        
        // Remove dead particles (swap with last active particle)
        for &remove_index in particles_to_remove.iter().rev() {
            if remove_index < self.active_particles {
                self.active_particles -= 1;
                if remove_index != self.active_particles {
                    self.particle_pool.swap(remove_index, self.active_particles);
                }
            }
        }
        
        Ok(())
    }

    fn update_wind_effects(&mut self, _delta_time: f32) {
        // Update wind direction with some variation
        let wind_variation = 0.1; // Small directional changes
        self.current_state.wind_direction[0] += (self.current_state.time_of_day * 0.1).sin() * wind_variation;
        self.current_state.wind_direction[2] += (self.current_state.time_of_day * 0.15).cos() * wind_variation;
        
        // Normalize wind direction
        let length = (self.current_state.wind_direction[0].powi(2) + 
                     self.current_state.wind_direction[2].powi(2)).sqrt();
        if length > 0.0 {
            self.current_state.wind_direction[0] /= length;
            self.current_state.wind_direction[2] /= length;
        }
        
        // Update wind speed based on weather
        let target_wind_speed = match self.current_state.current_weather {
            WeatherType::Clear => 3.0,
            WeatherType::Overcast => 5.0,
            WeatherType::Rain => 8.0,
            WeatherType::HeavyRain => 12.0,
            WeatherType::Snow => 6.0,
            WeatherType::HeavySnow => 10.0,
            WeatherType::Thunderstorm => 20.0,
            WeatherType::Fog => 2.0,
            WeatherType::Hail => 15.0,
            WeatherType::Sandstorm => 30.0,
        };
        
        // Smooth transition to target wind speed
        let speed_diff = target_wind_speed - self.current_state.wind_speed;
        self.current_state.wind_speed += speed_diff * 0.1;
    }

    fn update_environmental_maps(&mut self) {
        // Update temperature and humidity for a grid around the world
        // This is a simplified implementation
        for x in -10..=10 {
            for z in -10..=10 {
                let grid_pos = [x, z];
                
                // Base temperature with some spatial variation
                let base_temp = self.current_state.temperature;
                let spatial_variation = ((x as f32 * 0.1).sin() + (z as f32 * 0.15).cos()) * 2.0;
                
                self.temperature_map.insert(grid_pos, base_temp + spatial_variation);
                
                // Humidity varies less spatially
                let base_humidity = self.current_state.humidity;
                let humidity_variation = ((x as f32 * 0.05).sin() + (z as f32 * 0.08).cos()) * 0.1;
                
                self.humidity_map.insert(grid_pos, (base_humidity + humidity_variation).clamp(0.0, 1.0));
            }
        }
    }

    pub fn get_active_particle_count(&self) -> usize {
        self.active_particles
    }

    pub fn get_weather_forecast(&self, hours_ahead: u32) -> Vec<WeatherType> {
        // Simple forecast based on current patterns
        let mut forecast = Vec::new();
        let mut current_weather = self.current_state.current_weather.clone();
        
        for _ in 0..hours_ahead {
            // Mock forecast logic - in reality would use weather patterns and probabilities
            forecast.push(current_weather.clone());
        }
        
        forecast
    }

    pub fn shutdown(&mut self) -> RobinResult<()> {
        println!("Weather System shutdown:");
        println!("  Current weather: {:?}", self.current_state.current_weather);
        println!("  Active particles: {}", self.active_particles);
        println!("  Cloud layers: {}", self.cloud_layers.len());
        println!("  Temperature: {:.1}°C", self.current_state.temperature);

        self.precipitation_particles.clear();
        self.particle_pool.clear();
        self.cloud_layers.clear();
        self.weather_history.clear();
        self.temperature_map.clear();
        self.humidity_map.clear();

        Ok(())
    }
}

// Helper functions for weather calculations
pub fn interpolate_weather_values(from: f32, to: f32, progress: f32) -> f32 {
    from + (to - from) * progress
}

pub fn calculate_cloud_shadows(sun_elevation: f32, cloud_coverage: f32) -> f32 {
    let sun_intensity = (sun_elevation.to_radians().sin()).max(0.0);
    sun_intensity * (1.0 - cloud_coverage * 0.8)
}