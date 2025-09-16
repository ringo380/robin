// Level-of-Detail (LOD) System for Robin Engine
// Provides automatic quality scaling based on distance and performance requirements

use crate::engine::error::RobinResult;
use std::collections::HashMap;
use std::time::Instant;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LODConfig {
    pub enabled: bool,
    pub max_distance: f32,
    pub distance_bias: f32,
    pub target_triangle_count: u32,
    pub quality_levels: Vec<LODLevel>,
    pub hysteresis_factor: f32,
    pub update_frequency_ms: f32,
}

impl Default for LODConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_distance: 1000.0,
            distance_bias: 1.0,
            target_triangle_count: 100000,
            quality_levels: vec![
                LODLevel { distance_threshold: 50.0, quality_multiplier: 1.0, triangle_reduction: 0.0 },
                LODLevel { distance_threshold: 150.0, quality_multiplier: 0.75, triangle_reduction: 0.25 },
                LODLevel { distance_threshold: 300.0, quality_multiplier: 0.5, triangle_reduction: 0.5 },
                LODLevel { distance_threshold: 600.0, quality_multiplier: 0.25, triangle_reduction: 0.75 },
                LODLevel { distance_threshold: 1000.0, quality_multiplier: 0.1, triangle_reduction: 0.9 },
            ],
            hysteresis_factor: 0.1,
            update_frequency_ms: 16.67, // 60 FPS
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LODLevel {
    pub distance_threshold: f32,
    pub quality_multiplier: f32,
    pub triangle_reduction: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderableObject {
    pub id: String,
    pub position: [f32; 3],
    pub bounding_radius: f32,
    pub base_triangle_count: u32,
    pub materials: Vec<String>,
    pub importance_weight: f32,
    pub force_high_lod: bool,
    pub custom_lod_distances: Option<Vec<f32>>,
}

impl RenderableObject {
    pub fn new(id: String, position: [f32; 3], triangle_count: u32) -> Self {
        Self {
            id,
            position,
            bounding_radius: 5.0,
            base_triangle_count: triangle_count,
            materials: Vec::new(),
            importance_weight: 1.0,
            force_high_lod: false,
            custom_lod_distances: None,
        }
    }

    pub fn with_importance(mut self, weight: f32) -> Self {
        self.importance_weight = weight;
        self
    }

    pub fn with_high_lod_forced(mut self) -> Self {
        self.force_high_lod = true;
        self
    }

    pub fn with_custom_lod_distances(mut self, distances: Vec<f32>) -> Self {
        self.custom_lod_distances = Some(distances);
        self
    }
}

#[derive(Debug, Default, Clone)]
pub struct LODMetrics {
    pub total_objects: u32,
    pub objects_rendered: u32,
    pub objects_culled: u32,
    pub average_lod_level: f32,
    pub triangles_rendered: u32,
    pub triangles_saved: u32,
    pub update_time_ms: f32,
    pub distance_calculations: u32,
}

#[derive(Debug)]
struct ObjectLODState {
    object: RenderableObject,
    current_lod_level: usize,
    distance_to_camera: f32,
    last_update: Instant,
    visible: bool,
    rendered_triangles: u32,
}

#[derive(Debug)]
pub struct LODSystem {
    config: LODConfig,
    objects: HashMap<String, ObjectLODState>,
    camera_position: [f32; 3],
    quality_multiplier: f32,
    metrics: LODMetrics,
    last_update: Instant,
    enabled: bool,
}

impl LODSystem {
    pub fn new(config: LODConfig) -> RobinResult<Self> {
        Ok(Self {
            enabled: config.enabled,
            config,
            objects: HashMap::new(),
            camera_position: [0.0, 0.0, 0.0],
            quality_multiplier: 1.0,
            metrics: LODMetrics::default(),
            last_update: Instant::now(),
        })
    }

    pub fn initialize(&mut self) -> RobinResult<()> {
        self.metrics = LODMetrics::default();
        self.last_update = Instant::now();
        
        if self.enabled {
            println!("LOD System initialized:");
            println!("  Max Distance: {:.1}m", self.config.max_distance);
            println!("  Quality Levels: {}", self.config.quality_levels.len());
            println!("  Target Triangle Count: {}", self.config.target_triangle_count);
        }

        Ok(())
    }

    pub fn update(&mut self, camera_position: [f32; 3]) -> RobinResult<()> {
        if !self.enabled {
            return Ok(());
        }

        let update_start = Instant::now();
        self.camera_position = camera_position;

        // Check if we need to update based on frequency
        if self.last_update.elapsed().as_secs_f32() * 1000.0 < self.config.update_frequency_ms {
            return Ok(());
        }

        self.update_object_lod_levels()?;
        self.update_metrics();
        
        self.metrics.update_time_ms = update_start.elapsed().as_secs_f32() * 1000.0;
        self.last_update = Instant::now();

        Ok(())
    }

    fn update_object_lod_levels(&mut self) -> RobinResult<()> {
        let mut distance_calculations = 0;

        for (_, state) in self.objects.iter_mut() {
            // Calculate distance to camera
            let dx = state.object.position[0] - self.camera_position[0];
            let dy = state.object.position[1] - self.camera_position[1];
            let dz = state.object.position[2] - self.camera_position[2];
            
            let distance = (dx * dx + dy * dy + dz * dz).sqrt();
            state.distance_to_camera = distance * self.config.distance_bias;
            distance_calculations += 1;

            // Adjust distance based on object importance
            let effective_distance = distance / state.object.importance_weight.max(0.1);

            // Force high LOD for important objects
            if state.object.force_high_lod {
                state.current_lod_level = 0;
                state.visible = effective_distance <= self.config.max_distance;
            } else {
                // Determine appropriate LOD level
                let new_lod_level = Self::calculate_lod_level_static(effective_distance, &state.object, &self.config);
                
                // Apply hysteresis to prevent flickering
                if new_lod_level != state.current_lod_level {
                    let hysteresis_distance = self.config.hysteresis_factor * self.config.max_distance;
                    
                    if new_lod_level > state.current_lod_level {
                        // Moving to lower quality - apply hysteresis
                        if effective_distance > Self::get_lod_threshold_static(new_lod_level, &self.config) + hysteresis_distance {
                            state.current_lod_level = new_lod_level;
                        }
                    } else {
                        // Moving to higher quality - apply hysteresis
                        if effective_distance < Self::get_lod_threshold_static(new_lod_level, &self.config) - hysteresis_distance {
                            state.current_lod_level = new_lod_level;
                        }
                    }
                }

                state.visible = effective_distance <= self.config.max_distance;
            }

            // Calculate rendered triangles based on LOD level
            if state.visible && state.current_lod_level < self.config.quality_levels.len() {
                let quality = self.config.quality_levels[state.current_lod_level].quality_multiplier;
                state.rendered_triangles = (state.object.base_triangle_count as f32 * quality * self.quality_multiplier) as u32;
            } else {
                state.rendered_triangles = 0;
            }
        }

        self.metrics.distance_calculations = distance_calculations;
        Ok(())
    }

    fn calculate_lod_level(&self, distance: f32, object: &RenderableObject) -> usize {
        Self::calculate_lod_level_static(distance * self.quality_multiplier, object, &self.config)
    }

    fn get_lod_threshold(&self, lod_level: usize) -> f32 {
        Self::get_lod_threshold_static(lod_level, &self.config)
    }

    fn calculate_lod_level_static(distance: f32, object: &RenderableObject, config: &LODConfig) -> usize {
        // Use custom LOD distances if specified
        if let Some(ref custom_distances) = object.custom_lod_distances {
            for (level, &threshold) in custom_distances.iter().enumerate() {
                if distance <= threshold {
                    return level;
                }
            }
            return custom_distances.len().saturating_sub(1);
        }

        // Use default LOD levels
        for (level, lod_level) in config.quality_levels.iter().enumerate() {
            if distance <= lod_level.distance_threshold {
                return level;
            }
        }

        config.quality_levels.len().saturating_sub(1)
    }

    fn get_lod_threshold_static(lod_level: usize, config: &LODConfig) -> f32 {
        if lod_level < config.quality_levels.len() {
            config.quality_levels[lod_level].distance_threshold
        } else {
            config.max_distance
        }
    }

    fn update_metrics(&mut self) {
        let mut total_objects = 0;
        let mut objects_rendered = 0;
        let mut objects_culled = 0;
        let mut total_lod_levels = 0.0;
        let mut triangles_rendered = 0;
        let mut triangles_saved = 0;

        for state in self.objects.values() {
            total_objects += 1;

            if state.visible {
                objects_rendered += 1;
                total_lod_levels += state.current_lod_level as f32;
                triangles_rendered += state.rendered_triangles;
                triangles_saved += state.object.base_triangle_count.saturating_sub(state.rendered_triangles);
            } else {
                objects_culled += 1;
                triangles_saved += state.object.base_triangle_count;
            }
        }

        self.metrics.total_objects = total_objects;
        self.metrics.objects_rendered = objects_rendered;
        self.metrics.objects_culled = objects_culled;
        self.metrics.average_lod_level = if objects_rendered > 0 {
            total_lod_levels / objects_rendered as f32
        } else {
            0.0
        };
        self.metrics.triangles_rendered = triangles_rendered;
        self.metrics.triangles_saved = triangles_saved;
    }

    // Public API
    pub fn register_object(&mut self, object: RenderableObject) -> RobinResult<String> {
        let object_id = object.id.clone();
        
        let state = ObjectLODState {
            current_lod_level: 0,
            distance_to_camera: 0.0,
            last_update: Instant::now(),
            visible: true,
            rendered_triangles: object.base_triangle_count,
            object,
        };

        self.objects.insert(object_id.clone(), state);
        Ok(object_id)
    }

    pub fn unregister_object(&mut self, object_id: &str) -> RobinResult<()> {
        self.objects.remove(object_id);
        Ok(())
    }

    pub fn update_object_position(&mut self, object_id: &str, position: [f32; 3]) -> RobinResult<()> {
        if let Some(state) = self.objects.get_mut(object_id) {
            state.object.position = position;
        }
        Ok(())
    }

    pub fn set_object_importance(&mut self, object_id: &str, importance: f32) -> RobinResult<()> {
        if let Some(state) = self.objects.get_mut(object_id) {
            state.object.importance_weight = importance.max(0.1);
        }
        Ok(())
    }

    pub fn get_object_lod_level(&self, object_id: &str) -> Option<usize> {
        self.objects.get(object_id).map(|state| state.current_lod_level)
    }

    pub fn get_object_triangle_count(&self, object_id: &str) -> Option<u32> {
        self.objects.get(object_id).map(|state| state.rendered_triangles)
    }

    pub fn is_object_visible(&self, object_id: &str) -> bool {
        self.objects.get(object_id).map_or(false, |state| state.visible)
    }

    pub fn get_rendered_objects(&self) -> Vec<String> {
        self.objects.iter()
            .filter(|(_, state)| state.visible)
            .map(|(id, _)| id.clone())
            .collect()
    }

    pub fn set_quality_multiplier(&mut self, multiplier: f32) {
        self.quality_multiplier = multiplier.clamp(0.1, 2.0);
    }

    pub fn get_quality_multiplier(&self) -> f32 {
        self.quality_multiplier
    }

    pub fn get_metrics(&self) -> &LODMetrics {
        &self.metrics
    }

    pub fn get_rendered_object_count(&self) -> u32 {
        self.metrics.objects_rendered
    }

    pub fn get_total_triangle_count(&self) -> u32 {
        self.metrics.triangles_rendered
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
        // When disabled, render all objects at full quality
        for state in self.objects.values_mut() {
            state.current_lod_level = 0;
            state.visible = true;
            state.rendered_triangles = state.object.base_triangle_count;
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn get_performance_impact(&self) -> f32 {
        if self.metrics.triangles_rendered + self.metrics.triangles_saved == 0 {
            return 1.0;
        }
        
        self.metrics.triangles_rendered as f32 / 
        (self.metrics.triangles_rendered + self.metrics.triangles_saved) as f32
    }

    pub fn adjust_for_performance(&mut self, target_triangle_count: u32) -> RobinResult<()> {
        if self.metrics.triangles_rendered <= target_triangle_count {
            return Ok(());
        }

        // Calculate required quality reduction
        let reduction_factor = target_triangle_count as f32 / self.metrics.triangles_rendered as f32;
        self.set_quality_multiplier(self.quality_multiplier * reduction_factor);

        Ok(())
    }

    pub fn shutdown(&mut self) -> RobinResult<()> {
        if self.enabled {
            println!("LOD System shutdown:");
            println!("  Objects managed: {}", self.metrics.total_objects);
            println!("  Triangles saved: {}", self.metrics.triangles_saved);
            println!("  Average LOD level: {:.1}", self.metrics.average_lod_level);
        }

        self.objects.clear();
        self.metrics = LODMetrics::default();
        
        Ok(())
    }
}