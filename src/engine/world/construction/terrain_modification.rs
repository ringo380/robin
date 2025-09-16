use nalgebra::{Vector3, Point3};
use std::collections::HashMap;
use super::{Material, MaterialType, Voxel};

pub struct TerrainModifier {
    // Terrain operations
    pub active_operation: Option<TerrainOperation>,
    pub operation_strength: f32,
    pub operation_size: f32,
    pub falloff_curve: FalloffCurve,
    
    // Brush settings
    pub brush_shape: BrushShape,
    pub brush_hardness: f32,
    pub brush_spacing: f32,
    
    // Terrain layers
    pub height_map: HashMap<Point3<i32>, f32>,
    pub material_map: HashMap<Point3<i32>, String>,
    pub erosion_map: HashMap<Point3<i32>, f32>,
    
    // Environmental effects
    pub weather_effects: WeatherEffects,
    pub natural_processes: NaturalProcesses,
    
    // Performance settings
    pub modification_range: f32,
    pub update_frequency: f32,
    pub batch_size: usize,
    
    // History
    pub modification_history: Vec<TerrainModification>,
    pub history_index: usize,
}

#[derive(Clone, Debug)]
pub enum TerrainOperation {
    Raise,
    Lower,
    Flatten,
    Smooth,
    Roughen,
    Paint,
    Erode,
    Build,
    Carve,
    Blend,
}

#[derive(Clone, Debug)]
pub enum BrushShape {
    Circle,
    Square,
    Triangle,
    Custom(Vec<Point3<f32>>),
}

#[derive(Clone, Debug)]
pub enum FalloffCurve {
    Linear,
    Smooth,
    Sharp,
    Constant,
    Custom(Vec<f32>),
}

#[derive(Clone, Debug)]
pub struct WeatherEffects {
    pub erosion_rate: f32,
    pub vegetation_growth: f32,
    pub ice_formation: f32,
    pub thermal_expansion: f32,
    pub wind_strength: Vector3<f32>,
    pub precipitation: f32,
}

#[derive(Clone, Debug)]
pub struct NaturalProcesses {
    pub tectonic_activity: f32,
    pub volcanic_activity: f32,
    pub sedimentation: f32,
    pub chemical_weathering: f32,
    pub biological_activity: f32,
}

#[derive(Clone, Debug)]
pub struct TerrainModification {
    pub operation: TerrainOperation,
    pub position: Point3<f32>,
    pub area: f32,
    pub strength: f32,
    pub affected_points: Vec<Point3<i32>>,
    pub old_heights: Vec<f32>,
    pub new_heights: Vec<f32>,
    pub timestamp: u64,
    pub engineer_id: String,
}

#[derive(Clone, Debug)]
pub struct TerrainLayer {
    pub name: String,
    pub material: String,
    pub thickness: f32,
    pub hardness: f32,
    pub fertility: f32,
    pub porosity: f32,
    pub composition: HashMap<String, f32>,
}

impl Default for WeatherEffects {
    fn default() -> Self {
        Self {
            erosion_rate: 0.1,
            vegetation_growth: 0.05,
            ice_formation: 0.0,
            thermal_expansion: 0.01,
            wind_strength: Vector3::new(1.0, 0.0, 0.0),
            precipitation: 0.3,
        }
    }
}

impl Default for NaturalProcesses {
    fn default() -> Self {
        Self {
            tectonic_activity: 0.001,
            volcanic_activity: 0.0,
            sedimentation: 0.02,
            chemical_weathering: 0.01,
            biological_activity: 0.03,
        }
    }
}

impl TerrainModifier {
    pub fn new() -> Self {
        Self {
            active_operation: None,
            operation_strength: 1.0,
            operation_size: 5.0,
            falloff_curve: FalloffCurve::Smooth,
            
            brush_shape: BrushShape::Circle,
            brush_hardness: 0.5,
            brush_spacing: 1.0,
            
            height_map: HashMap::new(),
            material_map: HashMap::new(),
            erosion_map: HashMap::new(),
            
            weather_effects: WeatherEffects::default(),
            natural_processes: NaturalProcesses::default(),
            
            modification_range: 50.0,
            update_frequency: 1.0,
            batch_size: 100,
            
            modification_history: Vec::new(),
            history_index: 0,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        // Apply natural processes
        self.apply_natural_processes(delta_time);
        
        // Apply weather effects
        self.apply_weather_effects(delta_time);
        
        // Update erosion simulation
        self.update_erosion_simulation(delta_time);
        
        // Process any active terrain modifications
        if let Some(_operation) = &self.active_operation {
            // Process ongoing operation
        }
    }

    pub fn start_terrain_modification(&mut self, operation: TerrainOperation, position: Point3<f32>, engineer_id: &str) -> Result<(), String> {
        self.active_operation = Some(operation.clone());
        
        // Calculate affected area
        let affected_points = self.get_affected_points(position);
        
        // Store old heights for undo functionality
        let old_heights: Vec<f32> = affected_points.iter()
            .map(|p| self.get_height_at(*p))
            .collect();
        
        // Apply terrain modification
        let new_heights = self.apply_terrain_operation(&operation, position, &affected_points)?;
        
        // Record modification in history
        let modification = TerrainModification {
            operation,
            position,
            area: self.operation_size,
            strength: self.operation_strength,
            affected_points: affected_points.clone(),
            old_heights,
            new_heights: new_heights.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            engineer_id: engineer_id.to_string(),
        };
        
        self.modification_history.push(modification);
        self.history_index = self.modification_history.len();
        
        // Update height map
        for (i, &point) in affected_points.iter().enumerate() {
            self.height_map.insert(point, new_heights[i]);
        }
        
        self.active_operation = None;
        Ok(())
    }

    fn get_affected_points(&self, center: Point3<f32>) -> Vec<Point3<i32>> {
        let mut points = Vec::new();
        let radius = self.operation_size;
        let center_i = Point3::new(center.x as i32, 0, center.z as i32);
        
        let min_x = (center.x - radius) as i32;
        let max_x = (center.x + radius) as i32;
        let min_z = (center.z - radius) as i32;
        let max_z = (center.z + radius) as i32;
        
        for x in min_x..=max_x {
            for z in min_z..=max_z {
                let point = Point3::new(x, 0, z);
                let distance = ((point.x - center_i.x).pow(2) + (point.z - center_i.z).pow(2)) as f32;
                
                if self.is_point_in_brush(distance.sqrt(), radius) {
                    points.push(point);
                }
            }
        }
        
        points
    }

    fn is_point_in_brush(&self, distance: f32, radius: f32) -> bool {
        match self.brush_shape {
            BrushShape::Circle => distance <= radius,
            BrushShape::Square => distance <= radius * 1.414, // Approximate
            BrushShape::Triangle => distance <= radius && (distance / radius) < 0.866, // Approximate
            BrushShape::Custom(_) => distance <= radius, // Simplified
        }
    }

    fn apply_terrain_operation(&self, operation: &TerrainOperation, center: Point3<f32>, points: &[Point3<i32>]) -> Result<Vec<f32>, String> {
        let mut new_heights = Vec::new();
        
        for &point in points {
            let distance = ((point.x as f32 - center.x).powi(2) + (point.z as f32 - center.z).powi(2)).sqrt();
            let falloff = self.calculate_falloff(distance);
            let current_height = self.get_height_at(point);
            
            let new_height = match operation {
                TerrainOperation::Raise => {
                    current_height + self.operation_strength * falloff
                }
                TerrainOperation::Lower => {
                    current_height - self.operation_strength * falloff
                }
                TerrainOperation::Flatten => {
                    let target_height = center.y;
                    current_height + (target_height - current_height) * falloff * self.operation_strength
                }
                TerrainOperation::Smooth => {
                    let avg_height = self.get_average_height_around(point, 2.0);
                    current_height + (avg_height - current_height) * falloff * self.operation_strength
                }
                TerrainOperation::Roughen => {
                    let noise = self.generate_noise(point.x as f32, point.z as f32);
                    current_height + noise * falloff * self.operation_strength
                }
                TerrainOperation::Erode => {
                    self.apply_erosion_at(point, falloff * self.operation_strength)
                }
                TerrainOperation::Carve => {
                    let carve_depth = self.operation_strength * falloff;
                    current_height - carve_depth
                }
                _ => current_height,
            };
            
            new_heights.push(new_height.clamp(-1000.0, 1000.0));
        }
        
        Ok(new_heights)
    }

    fn calculate_falloff(&self, distance: f32) -> f32 {
        let normalized_distance = (distance / self.operation_size).clamp(0.0, 1.0);
        
        match self.falloff_curve {
            FalloffCurve::Linear => 1.0 - normalized_distance,
            FalloffCurve::Smooth => {
                let t = 1.0 - normalized_distance;
                t * t * (3.0 - 2.0 * t) // Smoothstep
            }
            FalloffCurve::Sharp => {
                if normalized_distance < 0.5 {
                    1.0
                } else {
                    2.0 * (1.0 - normalized_distance)
                }
            }
            FalloffCurve::Constant => 1.0,
            FalloffCurve::Custom(_) => 1.0 - normalized_distance, // Simplified
        }
    }

    fn get_height_at(&self, point: Point3<i32>) -> f32 {
        self.height_map.get(&point).copied().unwrap_or(0.0)
    }

    fn get_average_height_around(&self, center: Point3<i32>, radius: f32) -> f32 {
        let mut total_height = 0.0;
        let mut count = 0;
        
        let min_x = center.x - radius as i32;
        let max_x = center.x + radius as i32;
        let min_z = center.z - radius as i32;
        let max_z = center.z + radius as i32;
        
        for x in min_x..=max_x {
            for z in min_z..=max_z {
                let point = Point3::new(x, 0, z);
                let distance = ((point.x - center.x).pow(2) + (point.z - center.z).pow(2)) as f32;
                
                if distance <= radius * radius {
                    total_height += self.get_height_at(point);
                    count += 1;
                }
            }
        }
        
        if count > 0 {
            total_height / count as f32
        } else {
            self.get_height_at(center)
        }
    }

    fn generate_noise(&self, x: f32, z: f32) -> f32 {
        // Simple noise function - in practice would use proper noise library
        ((x * 0.1).sin() * (z * 0.1).cos() + (x * 0.05).cos() * (z * 0.07).sin()) * 0.5
    }

    fn apply_erosion_at(&self, point: Point3<i32>, strength: f32) -> f32 {
        let current_height = self.get_height_at(point);
        let erosion_factor = self.erosion_map.get(&point).copied().unwrap_or(0.1);
        
        // Simulate water erosion
        let slope = self.calculate_slope_at(point);
        let erosion_amount = strength * erosion_factor * slope.clamp(0.0, 1.0);
        
        current_height - erosion_amount
    }

    fn calculate_slope_at(&self, point: Point3<i32>) -> f32 {
        let center_height = self.get_height_at(point);
        let neighbors = [
            Point3::new(point.x + 1, 0, point.z),
            Point3::new(point.x - 1, 0, point.z),
            Point3::new(point.x, 0, point.z + 1),
            Point3::new(point.x, 0, point.z - 1),
        ];
        
        let mut max_slope = 0.0;
        
        for neighbor in &neighbors {
            let neighbor_height = self.get_height_at(*neighbor);
            let height_diff = (center_height - neighbor_height).abs();
            max_slope = max_slope.max(height_diff);
        }
        
        max_slope
    }

    fn apply_natural_processes(&mut self, delta_time: f32) {
        // Apply tectonic shifts
        if self.natural_processes.tectonic_activity > 0.0 {
            self.apply_tectonic_activity(delta_time);
        }
        
        // Apply sedimentation
        if self.natural_processes.sedimentation > 0.0 {
            self.apply_sedimentation(delta_time);
        }
        
        // Apply chemical weathering
        if self.natural_processes.chemical_weathering > 0.0 {
            self.apply_chemical_weathering(delta_time);
        }
    }

    fn apply_weather_effects(&mut self, delta_time: f32) {
        // Apply wind erosion
        if self.weather_effects.wind_strength.magnitude() > 0.0 {
            self.apply_wind_erosion(delta_time);
        }
        
        // Apply precipitation effects
        if self.weather_effects.precipitation > 0.0 {
            self.apply_precipitation_effects(delta_time);
        }
        
        // Apply thermal effects
        if self.weather_effects.thermal_expansion > 0.0 {
            self.apply_thermal_effects(delta_time);
        }
    }

    fn update_erosion_simulation(&mut self, delta_time: f32) {
        // Update erosion patterns based on water flow simulation
        let mut new_erosion_values = HashMap::new();
        
        for (&point, &height) in &self.height_map {
            let slope = self.calculate_slope_at(point);
            let water_accumulation = self.calculate_water_accumulation(point);
            
            let erosion_rate = slope * water_accumulation * self.weather_effects.erosion_rate * delta_time;
            let current_erosion = self.erosion_map.get(&point).copied().unwrap_or(0.0);
            let new_erosion = (current_erosion + erosion_rate).clamp(0.0, 1.0);
            
            new_erosion_values.insert(point, new_erosion);
        }
        
        // Update erosion map
        for (point, erosion) in new_erosion_values {
            self.erosion_map.insert(point, erosion);
        }
    }

    fn calculate_water_accumulation(&self, point: Point3<i32>) -> f32 {
        // Simplified water accumulation calculation
        let height = self.get_height_at(point);
        let mut accumulation = self.weather_effects.precipitation;
        
        // Water flows from higher to lower areas
        let neighbors = [
            Point3::new(point.x + 1, 0, point.z),
            Point3::new(point.x - 1, 0, point.z),
            Point3::new(point.x, 0, point.z + 1),
            Point3::new(point.x, 0, point.z - 1),
        ];
        
        for neighbor in &neighbors {
            let neighbor_height = self.get_height_at(*neighbor);
            if neighbor_height > height {
                accumulation += (neighbor_height - height) * 0.1;
            }
        }
        
        accumulation.clamp(0.0, 10.0)
    }

    fn apply_tectonic_activity(&mut self, delta_time: f32) {
        // Simplified tectonic simulation
        for (&point, height) in &mut self.height_map {
            let tectonic_noise = self.generate_noise(point.x as f32 * 0.001, point.z as f32 * 0.001);
            let uplift = tectonic_noise * self.natural_processes.tectonic_activity * delta_time;
            *height += uplift;
        }
    }

    fn apply_sedimentation(&mut self, delta_time: f32) {
        // Move sediment from high slope areas to low areas
        let mut sediment_changes = HashMap::new();
        
        for (&point, &_height) in &self.height_map {
            let slope = self.calculate_slope_at(point);
            let erosion_rate = slope * self.natural_processes.sedimentation * delta_time;
            
            if slope > 0.1 {
                // Erode from high slope areas
                sediment_changes.insert(point, -erosion_rate);
                
                // Deposit in lower areas
                let downstream = self.find_downstream_point(point);
                if let Some(dep_point) = downstream {
                    *sediment_changes.entry(dep_point).or_insert(0.0) += erosion_rate * 0.5;
                }
            }
        }
        
        // Apply sediment changes
        for (point, change) in sediment_changes {
            if let Some(height) = self.height_map.get_mut(&point) {
                *height += change;
            }
        }
    }

    fn apply_chemical_weathering(&mut self, _delta_time: f32) {
        // Apply chemical weathering effects
        // This would modify rock hardness, create caves, etc.
    }

    fn apply_wind_erosion(&mut self, delta_time: f32) {
        let wind_dir = self.weather_effects.wind_strength.normalize();
        let wind_strength = self.weather_effects.wind_strength.magnitude();
        
        // Apply wind erosion effects based on wind direction and strength
        for (&point, height) in &mut self.height_map {
            let exposure = self.calculate_wind_exposure(point, wind_dir);
            let erosion = wind_strength * exposure * delta_time * 0.01;
            *height -= erosion;
        }
    }

    fn apply_precipitation_effects(&mut self, delta_time: f32) {
        // Apply rainfall erosion and vegetation growth
        let precipitation = self.weather_effects.precipitation * delta_time;
        
        for (&point, _height) in &self.height_map {
            let current_erosion = self.erosion_map.get(&point).copied().unwrap_or(0.0);
            let new_erosion = current_erosion + precipitation * 0.1;
            self.erosion_map.insert(point, new_erosion.clamp(0.0, 1.0));
        }
    }

    fn apply_thermal_effects(&mut self, _delta_time: f32) {
        // Apply freeze-thaw cycles and thermal expansion
    }

    fn calculate_wind_exposure(&self, point: Point3<i32>, wind_dir: Vector3<f32>) -> f32 {
        // Calculate how exposed a point is to wind
        let height = self.get_height_at(point);
        let upwind_point = Point3::new(
            point.x - (wind_dir.x * 5.0) as i32,
            0,
            point.z - (wind_dir.z * 5.0) as i32,
        );
        
        let upwind_height = self.get_height_at(upwind_point);
        
        if height > upwind_height {
            1.0 // Fully exposed
        } else {
            0.5 // Partially sheltered
        }
    }

    fn find_downstream_point(&self, point: Point3<i32>) -> Option<Point3<i32>> {
        let current_height = self.get_height_at(point);
        let neighbors = [
            Point3::new(point.x + 1, 0, point.z),
            Point3::new(point.x - 1, 0, point.z),
            Point3::new(point.x, 0, point.z + 1),
            Point3::new(point.x, 0, point.z - 1),
        ];
        
        let mut lowest_neighbor = None;
        let mut lowest_height = current_height;
        
        for neighbor in &neighbors {
            let neighbor_height = self.get_height_at(*neighbor);
            if neighbor_height < lowest_height {
                lowest_height = neighbor_height;
                lowest_neighbor = Some(*neighbor);
            }
        }
        
        lowest_neighbor
    }

    pub fn undo_last_modification(&mut self) -> Result<(), String> {
        if self.history_index == 0 {
            return Err("Nothing to undo".to_string());
        }
        
        self.history_index -= 1;
        let modification = &self.modification_history[self.history_index];
        
        // Restore old heights
        for (i, &point) in modification.affected_points.iter().enumerate() {
            self.height_map.insert(point, modification.old_heights[i]);
        }
        
        Ok(())
    }

    pub fn redo_modification(&mut self) -> Result<(), String> {
        if self.history_index >= self.modification_history.len() {
            return Err("Nothing to redo".to_string());
        }
        
        let modification = &self.modification_history[self.history_index];
        self.history_index += 1;
        
        // Restore new heights
        for (i, &point) in modification.affected_points.iter().enumerate() {
            self.height_map.insert(point, modification.new_heights[i]);
        }
        
        Ok(())
    }

    pub fn paint_material(&mut self, position: Point3<f32>, material: &str, engineer_id: &str) -> Result<(), String> {
        let affected_points = self.get_affected_points(position);
        
        for point in affected_points {
            self.material_map.insert(point, material.to_string());
        }
        
        // Record in history
        let modification = TerrainModification {
            operation: TerrainOperation::Paint,
            position,
            area: self.operation_size,
            strength: self.operation_strength,
            affected_points: vec![Point3::new(position.x as i32, 0, position.z as i32)],
            old_heights: vec![],
            new_heights: vec![],
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            engineer_id: engineer_id.to_string(),
        };
        
        self.modification_history.push(modification);
        self.history_index = self.modification_history.len();
        
        Ok(())
    }

    // Getters and setters
    pub fn set_operation_strength(&mut self, strength: f32) {
        self.operation_strength = strength.clamp(0.0, 10.0);
    }

    pub fn set_operation_size(&mut self, size: f32) {
        self.operation_size = size.clamp(0.5, 50.0);
    }

    pub fn set_brush_shape(&mut self, shape: BrushShape) {
        self.brush_shape = shape;
    }

    pub fn set_falloff_curve(&mut self, curve: FalloffCurve) {
        self.falloff_curve = curve;
    }

    pub fn get_height_at_world(&self, position: Point3<f32>) -> f32 {
        let point = Point3::new(position.x as i32, 0, position.z as i32);
        self.get_height_at(point)
    }

    pub fn get_material_at(&self, position: Point3<f32>) -> Option<&String> {
        let point = Point3::new(position.x as i32, 0, position.z as i32);
        self.material_map.get(&point)
    }

    pub fn get_erosion_at(&self, position: Point3<f32>) -> f32 {
        let point = Point3::new(position.x as i32, 0, position.z as i32);
        self.erosion_map.get(&point).copied().unwrap_or(0.0)
    }

    pub fn get_terrain_info(&self, position: Point3<f32>) -> TerrainInfo {
        let point = Point3::new(position.x as i32, 0, position.z as i32);
        
        TerrainInfo {
            height: self.get_height_at(point),
            material: self.material_map.get(&point).cloned(),
            erosion: self.erosion_map.get(&point).copied().unwrap_or(0.0),
            slope: self.calculate_slope_at(point),
            water_accumulation: self.calculate_water_accumulation(point),
            stability: 1.0 - self.calculate_slope_at(point).clamp(0.0, 1.0),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TerrainInfo {
    pub height: f32,
    pub material: Option<String>,
    pub erosion: f32,
    pub slope: f32,
    pub water_accumulation: f32,
    pub stability: f32,
}