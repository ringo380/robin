use std::collections::{HashMap, VecDeque};
use cgmath::{Vector2, Vector3};
use crate::engine::error::{RobinResult, RobinError};
use uuid::Uuid;
use smartcore::linalg::basic::matrix::DenseMatrix;
use smartcore::cluster::kmeans::KMeans;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BiomeType {
    Forest,
    Desert,
    Mountains,
    Plains,
    Ocean,
    Swamp,
    Arctic,
    Jungle,
    Volcanic,
    Cave,
    Sky,
    Underground,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TerrainFeature {
    River,
    Lake,
    Hill,
    Valley,
    Canyon,
    Plateau,
    Ridge,
    Crater,
    Geyser,
    Waterfall,
    Bridge,
    Road,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ResourceType {
    Wood,
    Stone,
    Metal,
    Crystal,
    Food,
    Water,
    Energy,
    Rare,
    Magical,
    Toxic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldCell {
    pub position: Vector3<i32>,
    pub elevation: f32,
    pub temperature: f32,
    pub humidity: f32,
    pub fertility: f32,
    pub biome: BiomeType,
    pub features: Vec<TerrainFeature>,
    pub resources: HashMap<ResourceType, f32>,
    pub accessibility: f32,
    pub danger_level: f32,
    pub population_capacity: u32,
    pub last_modified: std::time::SystemTime,
}

#[derive(Debug, Clone)]
pub struct NoiseParameters {
    pub octaves: u32,
    pub frequency: f32,
    pub amplitude: f32,
    pub lacunarity: f32,
    pub persistence: f32,
    pub seed: u64,
}

impl Default for NoiseParameters {
    fn default() -> Self {
        Self {
            octaves: 6,
            frequency: 0.01,
            amplitude: 1.0,
            lacunarity: 2.0,
            persistence: 0.5,
            seed: 42,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BiomeRule {
    pub elevation_range: (f32, f32),
    pub temperature_range: (f32, f32),
    pub humidity_range: (f32, f32),
    pub biome_type: BiomeType,
    pub probability: f32,
    pub required_features: Vec<TerrainFeature>,
    pub incompatible_biomes: Vec<BiomeType>,
}

#[derive(Debug)]
pub struct GenerationHistory {
    pub timestamp: std::time::SystemTime,
    pub algorithm: String,
    pub parameters: HashMap<String, String>,
    pub area_affected: (Vector3<i32>, Vector3<i32>),
    pub quality_score: f32,
}

pub struct ProceduralGenerationSystem {
    world_grid: HashMap<Vector3<i32>, WorldCell>,
    noise_generators: HashMap<String, NoiseParameters>,
    biome_rules: Vec<BiomeRule>,
    resource_distributions: HashMap<ResourceType, f32>,
    feature_generators: HashMap<TerrainFeature, Box<dyn Fn(Vector3<i32>) -> bool + Send + Sync>>,
    generation_history: VecDeque<GenerationHistory>,
    rng: ChaCha8Rng,
    quality_metrics: HashMap<String, f32>,
    performance_data: HashMap<String, std::time::Duration>,
    
    // Advanced generation settings
    erosion_enabled: bool,
    weather_simulation: bool,
    tectonic_activity: bool,
    climate_zones: bool,
    river_generation: bool,
    settlement_placement: bool,
    road_networks: bool,
    ecosystem_simulation: bool,
}

impl Default for ProceduralGenerationSystem {
    fn default() -> Self {
        let mut system = Self {
            world_grid: HashMap::new(),
            noise_generators: HashMap::new(),
            biome_rules: Vec::new(),
            resource_distributions: HashMap::new(),
            feature_generators: HashMap::new(),
            generation_history: VecDeque::with_capacity(100),
            rng: ChaCha8Rng::from_entropy(),
            quality_metrics: HashMap::new(),
            performance_data: HashMap::new(),
            erosion_enabled: true,
            weather_simulation: false,
            tectonic_activity: false,
            climate_zones: true,
            river_generation: true,
            settlement_placement: false,
            road_networks: false,
            ecosystem_simulation: false,
        };
        
        system.initialize_default_parameters();
        system
    }
}

impl ProceduralGenerationSystem {
    pub fn new() -> Self {
        Self::default()
    }
    
    fn initialize_default_parameters(&mut self) {
        // Initialize noise parameters for different terrain aspects
        self.noise_generators.insert("elevation".to_string(), NoiseParameters {
            octaves: 8,
            frequency: 0.008,
            amplitude: 100.0,
            lacunarity: 2.1,
            persistence: 0.55,
            seed: 12345,
        });
        
        self.noise_generators.insert("temperature".to_string(), NoiseParameters {
            octaves: 4,
            frequency: 0.005,
            amplitude: 40.0,
            lacunarity: 2.0,
            persistence: 0.6,
            seed: 54321,
        });
        
        self.noise_generators.insert("humidity".to_string(), NoiseParameters {
            octaves: 5,
            frequency: 0.012,
            amplitude: 1.0,
            lacunarity: 1.8,
            persistence: 0.4,
            seed: 98765,
        });
        
        // Initialize biome rules
        self.biome_rules = vec![
            BiomeRule {
                elevation_range: (80.0, 200.0),
                temperature_range: (-10.0, 10.0),
                humidity_range: (0.0, 0.3),
                biome_type: BiomeType::Arctic,
                probability: 0.9,
                required_features: vec![],
                incompatible_biomes: vec![BiomeType::Desert, BiomeType::Jungle],
            },
            BiomeRule {
                elevation_range: (-10.0, 30.0),
                temperature_range: (25.0, 45.0),
                humidity_range: (0.0, 0.2),
                biome_type: BiomeType::Desert,
                probability: 0.85,
                required_features: vec![],
                incompatible_biomes: vec![BiomeType::Forest, BiomeType::Swamp],
            },
            BiomeRule {
                elevation_range: (20.0, 80.0),
                temperature_range: (10.0, 25.0),
                humidity_range: (0.4, 1.0),
                biome_type: BiomeType::Forest,
                probability: 0.8,
                required_features: vec![],
                incompatible_biomes: vec![BiomeType::Desert],
            },
            BiomeRule {
                elevation_range: (100.0, 300.0),
                temperature_range: (-5.0, 15.0),
                humidity_range: (0.2, 0.8),
                biome_type: BiomeType::Mountains,
                probability: 0.75,
                required_features: vec![],
                incompatible_biomes: vec![BiomeType::Plains, BiomeType::Swamp],
            },
        ];
        
        // Initialize resource distributions
        self.resource_distributions.insert(ResourceType::Wood, 0.6);
        self.resource_distributions.insert(ResourceType::Stone, 0.8);
        self.resource_distributions.insert(ResourceType::Metal, 0.3);
        self.resource_distributions.insert(ResourceType::Crystal, 0.1);
        self.resource_distributions.insert(ResourceType::Water, 0.7);
        self.resource_distributions.insert(ResourceType::Food, 0.5);
        self.resource_distributions.insert(ResourceType::Energy, 0.2);
        self.resource_distributions.insert(ResourceType::Rare, 0.05);
        self.resource_distributions.insert(ResourceType::Magical, 0.02);
    }
    
    pub fn generate_region(&mut self, min_bounds: Vector3<i32>, max_bounds: Vector3<i32>) -> RobinResult<()> {
        let start_time = std::time::Instant::now();
        
        // Clear existing data in the region
        for x in min_bounds.x..=max_bounds.x {
            for y in min_bounds.y..=max_bounds.y {
                for z in min_bounds.z..=max_bounds.z {
                    let pos = Vector3::new(x, y, z);
                    self.world_grid.remove(&pos);
                }
            }
        }
        
        // Generate base terrain
        self.generate_base_terrain(min_bounds, max_bounds)?;
        
        // Apply erosion if enabled
        if self.erosion_enabled {
            self.apply_erosion(min_bounds, max_bounds)?;
        }
        
        // Generate biomes
        self.generate_biomes(min_bounds, max_bounds)?;
        
        // Generate features
        self.generate_terrain_features(min_bounds, max_bounds)?;
        
        // Generate rivers if enabled
        if self.river_generation {
            self.generate_rivers(min_bounds, max_bounds)?;
        }
        
        // Place resources
        self.place_resources(min_bounds, max_bounds)?;
        
        // Calculate accessibility
        self.calculate_accessibility(min_bounds, max_bounds)?;
        
        // Record generation history
        let generation_time = start_time.elapsed();
        self.performance_data.insert("region_generation".to_string(), generation_time);
        
        let history = GenerationHistory {
            timestamp: std::time::SystemTime::now(),
            algorithm: "full_region_generation".to_string(),
            parameters: self.get_generation_parameters(),
            area_affected: (min_bounds, max_bounds),
            quality_score: self.calculate_quality_score(min_bounds, max_bounds)?,
        };
        
        self.generation_history.push_back(history);
        if self.generation_history.len() > 100 {
            self.generation_history.pop_front();
        }
        
        Ok(())
    }
    
    fn generate_base_terrain(&mut self, min_bounds: Vector3<i32>, max_bounds: Vector3<i32>) -> RobinResult<()> {
        // Clone parameters to avoid borrowing conflicts
        let elevation_params = self.noise_generators.get("elevation").unwrap().clone();
        let temperature_params = self.noise_generators.get("temperature").unwrap().clone();
        let humidity_params = self.noise_generators.get("humidity").unwrap().clone();

        for x in min_bounds.x..=max_bounds.x {
            for y in min_bounds.y..=max_bounds.y {
                for z in min_bounds.z..=max_bounds.z {
                    let pos = Vector3::new(x, y, z);

                    // Generate base values using noise
                    let elevation = self.generate_noise(x as f32, y as f32, &elevation_params);
                    let temperature = self.generate_noise(x as f32, y as f32, &temperature_params) +
                                    (25.0 - elevation * 0.1); // Temperature decreases with elevation
                    let humidity = self.generate_noise(x as f32, y as f32, &humidity_params).abs();
                    
                    let cell = WorldCell {
                        position: pos,
                        elevation,
                        temperature,
                        humidity: humidity.clamp(0.0, 1.0),
                        fertility: self.calculate_fertility(elevation, temperature, humidity),
                        biome: BiomeType::Plains, // Will be determined later
                        features: Vec::new(),
                        resources: HashMap::new(),
                        accessibility: 0.0, // Will be calculated later
                        danger_level: self.rng.gen_range(0.0..0.3),
                        population_capacity: 0,
                        last_modified: std::time::SystemTime::now(),
                    };
                    
                    self.world_grid.insert(pos, cell);
                }
            }
        }
        
        Ok(())
    }
    
    fn generate_noise(&mut self, x: f32, y: f32, params: &NoiseParameters) -> f32 {
        let mut value = 0.0;
        let mut amplitude = params.amplitude;
        let mut frequency = params.frequency;
        
        for _ in 0..params.octaves {
            // Simple noise implementation - in a real system you'd use a proper noise library
            let noise_x = (x * frequency + params.seed as f32).sin();
            let noise_y = (y * frequency + params.seed as f32 + 1000.0).sin();
            let combined = (noise_x + noise_y) * 0.5;
            
            value += combined * amplitude;
            amplitude *= params.persistence;
            frequency *= params.lacunarity;
        }
        
        value
    }
    
    fn apply_erosion(&mut self, min_bounds: Vector3<i32>, max_bounds: Vector3<i32>) -> RobinResult<()> {
        let iterations = 5;
        let erosion_strength = 0.1;
        
        for _ in 0..iterations {
            let mut elevation_changes = HashMap::new();
            
            for x in min_bounds.x..=max_bounds.x {
                for y in min_bounds.y..=max_bounds.y {
                    for z in min_bounds.z..=max_bounds.z {
                        let pos = Vector3::new(x, y, z);
                        
                        if let Some(cell) = self.world_grid.get(&pos) {
                            let neighbors = self.get_neighbors(pos);
                            let avg_elevation = neighbors.iter()
                                .filter_map(|&np| self.world_grid.get(&np))
                                .map(|nc| nc.elevation)
                                .sum::<f32>() / neighbors.len() as f32;
                            
                            if cell.elevation > avg_elevation + 2.0 {
                                let erosion = (cell.elevation - avg_elevation) * erosion_strength;
                                elevation_changes.insert(pos, cell.elevation - erosion);
                            }
                        }
                    }
                }
            }
            
            // Apply elevation changes
            for (pos, new_elevation) in elevation_changes {
                if let Some(cell) = self.world_grid.get_mut(&pos) {
                    cell.elevation = new_elevation;
                }
            }
        }
        
        Ok(())
    }
    
    fn generate_biomes(&mut self, min_bounds: Vector3<i32>, max_bounds: Vector3<i32>) -> RobinResult<()> {
        for x in min_bounds.x..=max_bounds.x {
            for y in min_bounds.y..=max_bounds.y {
                for z in min_bounds.z..=max_bounds.z {
                    let pos = Vector3::new(x, y, z);
                    
                    if let Some(cell) = self.world_grid.get(&pos) {
                        let elevation = cell.elevation;
                        let temperature = cell.temperature;
                        let humidity = cell.humidity;
                        let biome = self.determine_biome(elevation, temperature, humidity)?;
                        let capacity = Self::calculate_population_capacity_values_static(elevation, temperature, humidity, biome.clone());

                        // Apply the changes after releasing the immutable borrow
                        if let Some(cell) = self.world_grid.get_mut(&pos) {
                            cell.biome = biome;
                            cell.population_capacity = capacity;
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn determine_biome(&mut self, elevation: f32, temperature: f32, humidity: f32) -> RobinResult<BiomeType> {
        let mut best_match = &self.biome_rules[0];
        let mut best_score = 0.0;
        
        for rule in &self.biome_rules {
            let mut score = 0.0;
            
            // Check elevation fit
            if elevation >= rule.elevation_range.0 && elevation <= rule.elevation_range.1 {
                score += 0.4;
            }
            
            // Check temperature fit
            if temperature >= rule.temperature_range.0 && temperature <= rule.temperature_range.1 {
                score += 0.3;
            }
            
            // Check humidity fit
            if humidity >= rule.humidity_range.0 && humidity <= rule.humidity_range.1 {
                score += 0.2;
            }
            
            // Add probability factor
            score *= rule.probability;
            
            // Add some randomness
            score += self.rng.gen_range(-0.1..0.1);
            
            if score > best_score {
                best_score = score;
                best_match = rule;
            }
        }
        
        Ok(best_match.biome_type.clone())
    }
    
    fn generate_terrain_features(&mut self, min_bounds: Vector3<i32>, max_bounds: Vector3<i32>) -> RobinResult<()> {
        for x in min_bounds.x..=max_bounds.x {
            for y in min_bounds.y..=max_bounds.y {
                for z in min_bounds.z..=max_bounds.z {
                    let pos = Vector3::new(x, y, z);
                    
                    // First, gather data without borrowing mutably
                    let neighbors = self.get_neighbors(pos);
                    let elevation_variance = self.calculate_elevation_variance(pos, &neighbors);

                    let surrounding_elevations: Vec<f32> = neighbors.iter()
                        .filter_map(|&np| self.world_grid.get(&np))
                        .map(|nc| nc.elevation)
                        .collect();

                    // Now get mutable reference and apply changes
                    if let Some(cell) = self.world_grid.get_mut(&pos) {
                        // Generate hills and valleys based on elevation variance
                        if elevation_variance > 15.0 && cell.elevation > 50.0 {
                            cell.features.push(TerrainFeature::Hill);
                        } else if elevation_variance > 20.0 && cell.elevation < -10.0 {
                            cell.features.push(TerrainFeature::Valley);
                        }

                        // Generate canyons in dry, elevated areas
                        if matches!(cell.biome, BiomeType::Desert | BiomeType::Mountains) &&
                           cell.humidity < 0.3 && elevation_variance > 25.0 {
                            if self.rng.gen_bool(0.1) {
                                cell.features.push(TerrainFeature::Canyon);
                            }
                        }

                        // Generate plateaus
                        if cell.elevation > 80.0 && elevation_variance < 5.0 {
                            let surrounding_lower = surrounding_elevations.iter()
                                .any(|&elev| elev < cell.elevation - 20.0);

                            if surrounding_lower {
                                cell.features.push(TerrainFeature::Plateau);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn generate_rivers(&mut self, min_bounds: Vector3<i32>, max_bounds: Vector3<i32>) -> RobinResult<()> {
        // Find high elevation points as river sources
        let mut sources = Vec::new();
        
        for x in min_bounds.x..=max_bounds.x {
            for y in min_bounds.y..=max_bounds.y {
                for z in min_bounds.z..=max_bounds.z {
                    let pos = Vector3::new(x, y, z);
                    
                    if let Some(cell) = self.world_grid.get(&pos) {
                        if cell.elevation > 70.0 && cell.humidity > 0.6 {
                            if self.rng.gen_bool(0.05) {
                                sources.push(pos);
                            }
                        }
                    }
                }
            }
        }
        
        // Generate rivers from sources
        for source in sources {
            self.trace_river(source, max_bounds)?;
        }
        
        Ok(())
    }
    
    fn trace_river(&mut self, start: Vector3<i32>, max_bounds: Vector3<i32>) -> RobinResult<()> {
        let mut current = start;
        let mut path = Vec::new();
        let max_length = 50;
        
        for _ in 0..max_length {
            path.push(current);
            
            // Find lowest neighbor
            let neighbors = self.get_neighbors(current);
            let mut lowest = current;
            let mut lowest_elevation = f32::MAX;
            
            for &neighbor in &neighbors {
                if neighbor.x > max_bounds.x || neighbor.y > max_bounds.y || neighbor.z > max_bounds.z {
                    continue;
                }
                
                if let Some(cell) = self.world_grid.get(&neighbor) {
                    if cell.elevation < lowest_elevation {
                        lowest_elevation = cell.elevation;
                        lowest = neighbor;
                    }
                }
            }
            
            // Stop if no lower neighbor found or reached sea level
            if lowest == current || lowest_elevation <= 0.0 {
                break;
            }
            
            current = lowest;
        }
        
        // Apply river to cells in path
        for &pos in &path {
            if let Some(cell) = self.world_grid.get_mut(&pos) {
                cell.features.push(TerrainFeature::River);
                cell.humidity = (cell.humidity + 0.3).min(1.0);
                cell.fertility = (cell.fertility + 0.2).min(1.0);
                
                // Add water resource
                cell.resources.insert(ResourceType::Water, 1.0);
            }
        }
        
        Ok(())
    }
    
    fn place_resources(&mut self, min_bounds: Vector3<i32>, max_bounds: Vector3<i32>) -> RobinResult<()> {
        for x in min_bounds.x..=max_bounds.x {
            for y in min_bounds.y..=max_bounds.y {
                for z in min_bounds.z..=max_bounds.z {
                    let pos = Vector3::new(x, y, z);
                    
                    if let Some(cell) = self.world_grid.get(&pos) {
                        let cell_copy = cell.clone();
                        let resources = Self::calculate_resources_for_cell_static(&cell_copy, &self.resource_distributions)?;

                        // Apply the resources after releasing the immutable borrow
                        if let Some(cell) = self.world_grid.get_mut(&pos) {
                            cell.resources = resources;
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn place_resources_for_cell(&mut self, cell: &mut WorldCell) -> RobinResult<()> {
        for (resource_type, base_probability) in &self.resource_distributions {
            let mut probability = *base_probability;
            
            // Modify probability based on biome and terrain
            probability *= match resource_type {
                ResourceType::Wood => match cell.biome {
                    BiomeType::Forest => 2.0,
                    BiomeType::Jungle => 1.8,
                    BiomeType::Desert => 0.1,
                    BiomeType::Arctic => 0.3,
                    _ => 1.0,
                },
                ResourceType::Stone => match cell.biome {
                    BiomeType::Mountains => 2.5,
                    BiomeType::Desert => 1.5,
                    BiomeType::Plains => 0.7,
                    _ => 1.0,
                },
                ResourceType::Metal => {
                    if cell.elevation > 60.0 { 2.0 } else { 0.5 }
                },
                ResourceType::Crystal => {
                    if cell.features.contains(&TerrainFeature::Canyon) ||
                       cell.features.contains(&TerrainFeature::Crater) { 3.0 } else { 1.0 }
                },
                ResourceType::Food => cell.fertility * 2.0,
                ResourceType::Water => {
                    if cell.features.contains(&TerrainFeature::River) ||
                       cell.features.contains(&TerrainFeature::Lake) { 5.0 } else { cell.humidity * 2.0 }
                },
                ResourceType::Energy => {
                    if matches!(cell.biome, BiomeType::Volcanic) { 4.0 } else { 0.5 }
                },
                ResourceType::Magical => {
                    if cell.elevation > 100.0 || cell.elevation < -50.0 { 2.0 } else { 0.3 }
                },
                _ => 1.0,
            };
            
            if self.rng.gen::<f32>() < probability {
                let amount = self.rng.gen_range(0.1..1.0) * probability;
                cell.resources.insert(resource_type.clone(), amount);
            }
        }
        
        Ok(())
    }

    fn calculate_resources_for_cell_static(
        cell: &WorldCell,
        resource_distributions: &std::collections::HashMap<ResourceType, f32>
    ) -> RobinResult<std::collections::HashMap<ResourceType, f32>> {
        let mut resources = std::collections::HashMap::new();

        for (resource_type, base_probability) in resource_distributions {
            let mut probability = *base_probability;

            // Modify probability based on biome and terrain (simplified without RNG)
            probability *= match resource_type {
                ResourceType::Wood => match cell.biome {
                    BiomeType::Forest => 2.0,
                    BiomeType::Jungle => 1.8,
                    BiomeType::Desert => 0.1,
                    BiomeType::Arctic => 0.3,
                    _ => 1.0,
                },
                ResourceType::Stone => match cell.biome {
                    BiomeType::Mountains => 2.5,
                    BiomeType::Desert => 1.5,
                    BiomeType::Plains => 0.7,
                    _ => 1.0,
                },
                ResourceType::Metal => {
                    if cell.elevation > 60.0 { 2.0 } else { 0.5 }
                },
                ResourceType::Crystal => {
                    if cell.features.contains(&TerrainFeature::Canyon) ||
                       cell.features.contains(&TerrainFeature::Crater) { 3.0 } else { 1.0 }
                },
                ResourceType::Food => cell.fertility * 2.0,
                ResourceType::Water => {
                    if cell.features.contains(&TerrainFeature::River) ||
                       cell.features.contains(&TerrainFeature::Lake) { 5.0 } else { cell.humidity * 2.0 }
                },
                ResourceType::Energy => {
                    if matches!(cell.biome, BiomeType::Volcanic) { 4.0 } else { 0.5 }
                },
                ResourceType::Magical => {
                    if cell.elevation > 100.0 || cell.elevation < -50.0 { 2.0 } else { 0.3 }
                },
                _ => 1.0,
            };

            // For static version, deterministically assign resources based on probability
            if probability > 0.5 {
                let amount = probability * 0.5; // Simplified amount calculation
                resources.insert(resource_type.clone(), amount);
            }
        }

        Ok(resources)
    }

    fn calculate_accessibility(&mut self, min_bounds: Vector3<i32>, max_bounds: Vector3<i32>) -> RobinResult<()> {
        for x in min_bounds.x..=max_bounds.x {
            for y in min_bounds.y..=max_bounds.y {
                for z in min_bounds.z..=max_bounds.z {
                    let pos = Vector3::new(x, y, z);
                    
                    if let Some(cell) = self.world_grid.get_mut(&pos) {
                        let mut accessibility = 1.0f32;
                        
                        // Reduce accessibility based on elevation extremes
                        if cell.elevation > 100.0 {
                            accessibility *= 0.7;
                        } else if cell.elevation < -20.0 {
                            accessibility *= 0.8;
                        }
                        
                        // Reduce accessibility for difficult terrain
                        if cell.features.contains(&TerrainFeature::Canyon) {
                            accessibility *= 0.3;
                        }
                        if cell.features.contains(&TerrainFeature::Hill) {
                            accessibility *= 0.8;
                        }
                        
                        // Improve accessibility for roads and rivers
                        if cell.features.contains(&TerrainFeature::River) {
                            accessibility *= 1.2;
                        }
                        if cell.features.contains(&TerrainFeature::Road) {
                            accessibility *= 1.5;
                        }
                        
                        // Biome-based accessibility
                        accessibility *= match cell.biome {
                            BiomeType::Plains => 1.2,
                            BiomeType::Forest => 0.9,
                            BiomeType::Desert => 0.7,
                            BiomeType::Mountains => 0.5,
                            BiomeType::Swamp => 0.4,
                            BiomeType::Arctic => 0.3,
                            _ => 1.0,
                        };
                        
                        cell.accessibility = accessibility.clamp(0.0, 2.0);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    // Helper methods
    
    fn get_neighbors(&self, pos: Vector3<i32>) -> Vec<Vector3<i32>> {
        vec![
            Vector3::new(pos.x + 1, pos.y, pos.z),
            Vector3::new(pos.x - 1, pos.y, pos.z),
            Vector3::new(pos.x, pos.y + 1, pos.z),
            Vector3::new(pos.x, pos.y - 1, pos.z),
            Vector3::new(pos.x + 1, pos.y + 1, pos.z),
            Vector3::new(pos.x + 1, pos.y - 1, pos.z),
            Vector3::new(pos.x - 1, pos.y + 1, pos.z),
            Vector3::new(pos.x - 1, pos.y - 1, pos.z),
        ]
    }
    
    fn calculate_elevation_variance(&self, pos: Vector3<i32>, neighbors: &[Vector3<i32>]) -> f32 {
        if let Some(center_cell) = self.world_grid.get(&pos) {
            let neighbor_elevations: Vec<f32> = neighbors.iter()
                .filter_map(|&np| self.world_grid.get(&np))
                .map(|nc| nc.elevation)
                .collect();
            
            if neighbor_elevations.is_empty() {
                return 0.0;
            }
            
            let mean = neighbor_elevations.iter().sum::<f32>() / neighbor_elevations.len() as f32;
            let variance = neighbor_elevations.iter()
                .map(|&e| (e - mean).powi(2))
                .sum::<f32>() / neighbor_elevations.len() as f32;
            
            variance.sqrt()
        } else {
            0.0
        }
    }
    
    fn calculate_fertility(&self, elevation: f32, temperature: f32, humidity: f32) -> f32 {
        let temp_factor = if temperature >= 10.0 && temperature <= 30.0 {
            1.0 - ((temperature - 20.0).abs() / 10.0)
        } else {
            0.0
        };
        
        let elevation_factor = if elevation >= -10.0 && elevation <= 100.0 {
            1.0 - (elevation.abs() / 100.0)
        } else {
            0.0
        };
        
        let humidity_factor = humidity;
        
        (temp_factor * 0.4 + elevation_factor * 0.3 + humidity_factor * 0.3).clamp(0.0, 1.0)
    }
    
    fn calculate_population_capacity(&self, cell: &WorldCell) -> u32 {
        let base_capacity = match cell.biome {
            BiomeType::Plains => 50,
            BiomeType::Forest => 30,
            BiomeType::Desert => 5,
            BiomeType::Mountains => 15,
            BiomeType::Ocean => 0,
            BiomeType::Swamp => 10,
            BiomeType::Arctic => 3,
            BiomeType::Jungle => 25,
            BiomeType::Volcanic => 8,
            _ => 20,
        };

        let fertility_modifier = cell.fertility;
        let accessibility_modifier = cell.accessibility;
        let danger_modifier = 1.0 - cell.danger_level;

        let water_modifier = if cell.resources.contains_key(&ResourceType::Water) { 1.2 } else { 0.8 };
        let food_modifier = if cell.resources.contains_key(&ResourceType::Food) { 1.1 } else { 0.9 };

        let final_capacity = base_capacity as f32 * fertility_modifier * accessibility_modifier *
                           danger_modifier * water_modifier * food_modifier;

        final_capacity.max(0.0) as u32
    }

    fn calculate_population_capacity_values(&self, elevation: f32, temperature: f32, humidity: f32, biome: BiomeType) -> u32 {
        Self::calculate_population_capacity_values_static(elevation, temperature, humidity, biome)
    }

    fn calculate_population_capacity_values_static(elevation: f32, temperature: f32, humidity: f32, biome: BiomeType) -> u32 {
        let base_capacity = match biome {
            BiomeType::Plains => 50,
            BiomeType::Forest => 30,
            BiomeType::Desert => 5,
            BiomeType::Mountains => 15,
            BiomeType::Ocean => 0,
            BiomeType::Swamp => 10,
            BiomeType::Arctic => 3,
            BiomeType::Jungle => 25,
            BiomeType::Volcanic => 8,
            _ => 20,
        };

        // Simple calculation based on environmental factors
        let fertility_modifier = (temperature * 0.5 + humidity * 0.5).clamp(0.1, 2.0);
        let elevation_modifier = (1.0 - (elevation.abs() / 100.0)).clamp(0.1, 1.5);

        let final_capacity = base_capacity as f32 * fertility_modifier * elevation_modifier;
        final_capacity.max(0.0) as u32
    }
    
    fn get_generation_parameters(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("erosion_enabled".to_string(), self.erosion_enabled.to_string());
        params.insert("weather_simulation".to_string(), self.weather_simulation.to_string());
        params.insert("river_generation".to_string(), self.river_generation.to_string());
        params.insert("climate_zones".to_string(), self.climate_zones.to_string());
        params.insert("biome_rules_count".to_string(), self.biome_rules.len().to_string());
        params.insert("resource_types_count".to_string(), self.resource_distributions.len().to_string());
        params
    }
    
    fn calculate_quality_score(&self, min_bounds: Vector3<i32>, max_bounds: Vector3<i32>) -> RobinResult<f32> {
        let mut total_score = 0.0;
        let mut cell_count = 0;
        
        for x in min_bounds.x..=max_bounds.x {
            for y in min_bounds.y..=max_bounds.y {
                for z in min_bounds.z..=max_bounds.z {
                    let pos = Vector3::new(x, y, z);
                    
                    if let Some(cell) = self.world_grid.get(&pos) {
                        let mut cell_score = 0.0;
                        
                        // Biome diversity score
                        cell_score += 0.2;
                        
                        // Feature richness score
                        cell_score += cell.features.len() as f32 * 0.1;
                        
                        // Resource variety score
                        cell_score += cell.resources.len() as f32 * 0.05;
                        
                        // Accessibility score
                        cell_score += cell.accessibility * 0.1;
                        
                        // Fertility score
                        cell_score += cell.fertility * 0.1;
                        
                        // Population capacity score (normalized)
                        cell_score += (cell.population_capacity as f32 / 100.0).min(1.0) * 0.1;
                        
                        total_score += cell_score;
                        cell_count += 1;
                    }
                }
            }
        }
        
        Ok(if cell_count > 0 { total_score / cell_count as f32 } else { 0.0 })
    }
    
    // Public query methods
    
    pub fn get_world_cell(&self, pos: Vector3<i32>) -> Option<&WorldCell> {
        self.world_grid.get(&pos)
    }
    
    pub fn get_region_cells(&self, min_bounds: Vector3<i32>, max_bounds: Vector3<i32>) -> Vec<&WorldCell> {
        let mut cells = Vec::new();
        
        for x in min_bounds.x..=max_bounds.x {
            for y in min_bounds.y..=max_bounds.y {
                for z in min_bounds.z..=max_bounds.z {
                    let pos = Vector3::new(x, y, z);
                    if let Some(cell) = self.world_grid.get(&pos) {
                        cells.push(cell);
                    }
                }
            }
        }
        
        cells
    }
    
    pub fn find_cells_by_biome(&self, biome: BiomeType) -> Vec<&WorldCell> {
        self.world_grid.values()
            .filter(|cell| std::mem::discriminant(&cell.biome) == std::mem::discriminant(&biome))
            .collect()
    }
    
    pub fn find_cells_with_resource(&self, resource: ResourceType) -> Vec<&WorldCell> {
        self.world_grid.values()
            .filter(|cell| cell.resources.contains_key(&resource))
            .collect()
    }
    
    pub fn get_generation_history(&self) -> &VecDeque<GenerationHistory> {
        &self.generation_history
    }
    
    pub fn get_quality_metrics(&self) -> &HashMap<String, f32> {
        &self.quality_metrics
    }
    
    pub fn get_performance_data(&self) -> &HashMap<String, std::time::Duration> {
        &self.performance_data
    }
    
    // Advanced generation features
    
    pub fn enable_advanced_features(&mut self, features: Vec<&str>) {
        for feature in features {
            match feature {
                "erosion" => self.erosion_enabled = true,
                "weather" => self.weather_simulation = true,
                "tectonic" => self.tectonic_activity = true,
                "climate_zones" => self.climate_zones = true,
                "rivers" => self.river_generation = true,
                "settlements" => self.settlement_placement = true,
                "roads" => self.road_networks = true,
                "ecosystem" => self.ecosystem_simulation = true,
                _ => {}
            }
        }
    }
    
    pub fn set_noise_parameters(&mut self, noise_type: &str, params: NoiseParameters) {
        self.noise_generators.insert(noise_type.to_string(), params);
    }
    
    pub fn add_biome_rule(&mut self, rule: BiomeRule) {
        self.biome_rules.push(rule);
    }
    
    pub fn set_resource_distribution(&mut self, resource: ResourceType, probability: f32) {
        self.resource_distributions.insert(resource, probability.clamp(0.0, 1.0));
    }
}