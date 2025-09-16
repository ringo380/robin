use crate::engine::math::{Vec3, Point3};
use crate::engine::graphics::Color;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MeasurementTools {
    active_measurements: HashMap<String, Measurement>,
    measurement_history: Vec<Measurement>,
    display_settings: MeasurementDisplaySettings,
    precision_level: PrecisionLevel,
}

#[derive(Debug, Clone)]
pub struct Measurement {
    pub id: String,
    pub measurement_type: MeasurementType,
    pub points: Vec<Point3>,
    pub result: MeasurementResult,
    pub timestamp: u64,
    pub color: Color,
    pub visible: bool,
}

#[derive(Debug, Clone)]
pub enum MeasurementType {
    Distance,
    Area,
    Volume,
    Angle,
    Slope,
    Height,
    Perimeter,
    CircleRadius,
    ArcLength,
    SurfaceArea,
}

#[derive(Debug, Clone)]
pub enum MeasurementResult {
    Distance(f32),
    Area(f32),
    Volume(f32),
    Angle(f32),
    Slope(f32),
    Height(f32),
    Perimeter(f32),
    Multiple(Vec<f32>),
}

#[derive(Debug, Clone)]
pub struct MeasurementDisplaySettings {
    pub show_labels: bool,
    pub show_dimensions: bool,
    pub show_grid_snap: bool,
    pub label_size: f32,
    pub line_thickness: f32,
    pub transparency: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum PrecisionLevel {
    Coarse,    // 1.0 unit precision
    Normal,    // 0.1 unit precision
    Fine,      // 0.01 unit precision
    Precise,   // 0.001 unit precision
}

#[derive(Debug, Clone)]
pub struct GridSystem {
    pub enabled: bool,
    pub size: f32,
    pub subdivisions: u32,
    pub snap_enabled: bool,
    pub snap_threshold: f32,
    pub color: Color,
}

#[derive(Debug, Clone)]
pub struct AlignmentTools {
    pub alignment_mode: AlignmentMode,
    pub reference_points: Vec<Point3>,
    pub tolerance: f32,
}

#[derive(Debug, Clone)]
pub enum AlignmentMode {
    None,
    ToGrid,
    ToPoint,
    ToEdge,
    ToSurface,
    ToAxis,
    Parallel,
    Perpendicular,
}

impl MeasurementTools {
    pub fn new() -> Self {
        Self {
            active_measurements: HashMap::new(),
            measurement_history: Vec::new(),
            display_settings: MeasurementDisplaySettings::default(),
            precision_level: PrecisionLevel::Normal,
        }
    }

    pub fn update(&mut self, _delta_time: f32) {
        // Update measurement display, fade old measurements, etc.
        for measurement in self.active_measurements.values_mut() {
            if measurement.color.a > 0.1 {
                measurement.color.a *= 0.999; // Slight fade
            }
        }
    }

    pub fn start_measurement(&mut self, measurement_type: MeasurementType, point: Point3) -> String {
        let id = format!("measurement_{}", self.active_measurements.len());
        let measurement = Measurement {
            id: id.clone(),
            measurement_type,
            points: vec![point],
            result: MeasurementResult::Distance(0.0),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            color: Color::new(1.0, 1.0, 0.0, 0.8),
            visible: true,
        };

        self.active_measurements.insert(id.clone(), measurement);
        id
    }

    pub fn add_measurement_point(&mut self, measurement_id: &str, point: Point3) -> bool {
        if let Some(measurement) = self.active_measurements.get_mut(measurement_id) {
            measurement.points.push(point);
            let mut measurement_clone = measurement.clone();

            // Drop the mutable borrow before calling the method
            drop(measurement);

            self.calculate_measurement_result(&mut measurement_clone);

            // Get mutable reference again to copy result back
            if let Some(measurement) = self.active_measurements.get_mut(measurement_id) {
                measurement.result = measurement_clone.result;
            }
            true
        } else {
            false
        }
    }

    pub fn finish_measurement(&mut self, measurement_id: &str) -> Option<MeasurementResult> {
        if let Some(measurement) = self.active_measurements.remove(measurement_id) {
            let result = measurement.result.clone();
            self.measurement_history.push(measurement);
            Some(result)
        } else {
            None
        }
    }

    fn calculate_measurement_result(&self, measurement: &mut Measurement) {
        match measurement.measurement_type {
            MeasurementType::Distance => {
                if measurement.points.len() >= 2 {
                    let distance = self.calculate_distance(&measurement.points[0], &measurement.points[1]);
                    measurement.result = MeasurementResult::Distance(distance);
                }
            }
            MeasurementType::Area => {
                if measurement.points.len() >= 3 {
                    let area = self.calculate_polygon_area(&measurement.points);
                    measurement.result = MeasurementResult::Area(area);
                }
            }
            MeasurementType::Volume => {
                if measurement.points.len() >= 4 {
                    let volume = self.calculate_volume(&measurement.points);
                    measurement.result = MeasurementResult::Volume(volume);
                }
            }
            MeasurementType::Angle => {
                if measurement.points.len() >= 3 {
                    let angle = self.calculate_angle(&measurement.points[0], &measurement.points[1], &measurement.points[2]);
                    measurement.result = MeasurementResult::Angle(angle);
                }
            }
            MeasurementType::Slope => {
                if measurement.points.len() >= 2 {
                    let slope = self.calculate_slope(&measurement.points[0], &measurement.points[1]);
                    measurement.result = MeasurementResult::Slope(slope);
                }
            }
            _ => {
                // Handle other measurement types
            }
        }
    }

    fn calculate_distance(&self, p1: &Point3, p2: &Point3) -> f32 {
        let dx = p2.x - p1.x;
        let dy = p2.y - p1.y;
        let dz = p2.z - p1.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    fn calculate_polygon_area(&self, points: &[Point3]) -> f32 {
        if points.len() < 3 {
            return 0.0;
        }

        let mut area = 0.0;
        for i in 0..points.len() {
            let j = (i + 1) % points.len();
            area += points[i].x * points[j].y;
            area -= points[j].x * points[i].y;
        }
        area.abs() / 2.0
    }

    fn calculate_volume(&self, _points: &[Point3]) -> f32 {
        // Simplified volume calculation for demonstration
        // In practice, this would use more sophisticated algorithms
        100.0
    }

    fn calculate_angle(&self, p1: &Point3, vertex: &Point3, p2: &Point3) -> f32 {
        let v1 = Vec3::new(p1.x - vertex.x, p1.y - vertex.y, p1.z - vertex.z);
        let v2 = Vec3::new(p2.x - vertex.x, p2.y - vertex.y, p2.z - vertex.z);
        
        let dot = v1.x * v2.x + v1.y * v2.y + v1.z * v2.z;
        let mag1 = (v1.x * v1.x + v1.y * v1.y + v1.z * v1.z).sqrt();
        let mag2 = (v2.x * v2.x + v2.y * v2.y + v2.z * v2.z).sqrt();
        
        if mag1 == 0.0 || mag2 == 0.0 {
            return 0.0;
        }
        
        (dot / (mag1 * mag2)).acos().to_degrees()
    }

    fn calculate_slope(&self, p1: &Point3, p2: &Point3) -> f32 {
        let horizontal_distance = ((p2.x - p1.x).powi(2) + (p2.z - p1.z).powi(2)).sqrt();
        if horizontal_distance == 0.0 {
            return f32::INFINITY;
        }
        (p2.y - p1.y) / horizontal_distance
    }

    pub fn snap_to_grid(&self, point: Point3, grid_size: f32) -> Point3 {
        Point3::new(
            (point.x / grid_size).round() * grid_size,
            (point.y / grid_size).round() * grid_size,
            (point.z / grid_size).round() * grid_size,
        )
    }

    pub fn find_nearest_alignment(&self, point: Point3, alignment: &AlignmentTools) -> Point3 {
        match alignment.alignment_mode {
            AlignmentMode::ToGrid => {
                // Snap to nearest grid intersection
                point // Simplified
            }
            AlignmentMode::ToPoint => {
                if let Some(reference) = alignment.reference_points.first() {
                    *reference
                } else {
                    point
                }
            }
            AlignmentMode::ToAxis => {
                // Snap to nearest axis
                let distances = [
                    (Point3::new(point.x, 0.0, 0.0), point.y.abs() + point.z.abs()),
                    (Point3::new(0.0, point.y, 0.0), point.x.abs() + point.z.abs()),
                    (Point3::new(0.0, 0.0, point.z), point.x.abs() + point.y.abs()),
                ];
                
                distances.iter()
                    .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                    .map(|(p, _)| *p)
                    .unwrap_or(point)
            }
            _ => point,
        }
    }

    pub fn get_measurement_summary(&self) -> MeasurementSummary {
        let total_measurements = self.measurement_history.len() + self.active_measurements.len();
        let mut type_counts = HashMap::new();
        
        for measurement in &self.measurement_history {
            *type_counts.entry(format!("{:?}", measurement.measurement_type)).or_insert(0) += 1;
        }
        
        for measurement in self.active_measurements.values() {
            *type_counts.entry(format!("{:?}", measurement.measurement_type)).or_insert(0) += 1;
        }

        MeasurementSummary {
            total_measurements,
            active_measurements: self.active_measurements.len(),
            type_counts,
            precision_level: self.precision_level,
        }
    }

    pub fn clear_all_measurements(&mut self) {
        self.active_measurements.clear();
    }

    pub fn export_measurements(&self) -> String {
        let mut output = String::new();
        output.push_str("Measurement Report\n");
        output.push_str(&"=".repeat(50));
        output.push('\n');

        for measurement in &self.measurement_history {
            output.push_str(&format!("ID: {}\n", measurement.id));
            output.push_str(&format!("Type: {:?}\n", measurement.measurement_type));
            output.push_str(&format!("Result: {:?}\n", measurement.result));
            output.push_str(&format!("Points: {} points\n", measurement.points.len()));
            output.push_str(&"-".repeat(30));
            output.push('\n');
        }

        output
    }
}

#[derive(Debug)]
pub struct MeasurementSummary {
    pub total_measurements: usize,
    pub active_measurements: usize,
    pub type_counts: HashMap<String, usize>,
    pub precision_level: PrecisionLevel,
}

impl Default for MeasurementDisplaySettings {
    fn default() -> Self {
        Self {
            show_labels: true,
            show_dimensions: true,
            show_grid_snap: true,
            label_size: 12.0,
            line_thickness: 2.0,
            transparency: 0.8,
        }
    }
}

impl PrecisionLevel {
    pub fn get_precision(&self) -> f32 {
        match self {
            PrecisionLevel::Coarse => 1.0,
            PrecisionLevel::Normal => 0.1,
            PrecisionLevel::Fine => 0.01,
            PrecisionLevel::Precise => 0.001,
        }
    }

    pub fn round_to_precision(&self, value: f32) -> f32 {
        let precision = self.get_precision();
        (value / precision).round() * precision
    }
}

impl Default for GridSystem {
    fn default() -> Self {
        Self {
            enabled: true,
            size: 1.0,
            subdivisions: 10,
            snap_enabled: true,
            snap_threshold: 0.5,
            color: Color::new(0.5, 0.5, 0.5, 0.3),
        }
    }
}

impl Default for AlignmentTools {
    fn default() -> Self {
        Self {
            alignment_mode: AlignmentMode::None,
            reference_points: Vec::new(),
            tolerance: 0.1,
        }
    }
}