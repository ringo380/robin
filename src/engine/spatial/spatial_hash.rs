use crate::engine::error::RobinResult;
use crate::engine::spatial::octree::AABB;
use cgmath::{Point3, InnerSpace};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct GridCell {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl GridCell {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
    
    pub fn from_position(position: Point3<f32>, cell_size: f32) -> Self {
        Self {
            x: (position.x / cell_size).floor() as i32,
            y: (position.y / cell_size).floor() as i32,
            z: (position.z / cell_size).floor() as i32,
        }
    }
    
    pub fn to_world_position(&self, cell_size: f32) -> Point3<f32> {
        Point3::new(
            self.x as f32 * cell_size,
            self.y as f32 * cell_size,
            self.z as f32 * cell_size,
        )
    }
}

#[derive(Debug, Clone)]
pub struct SpatialGrid {
    pub cell: GridCell,
    pub objects: HashSet<u32>,
    pub last_accessed: u64,
}

impl SpatialGrid {
    pub fn new(cell: GridCell) -> Self {
        Self {
            cell,
            objects: HashSet::new(),
            last_accessed: 0,
        }
    }
}

pub struct SpatialHash {
    cell_size: f32,
    grids: HashMap<GridCell, SpatialGrid>,
    object_positions: HashMap<u32, Point3<f32>>,
    object_cells: HashMap<u32, Vec<GridCell>>,
    frame_counter: u64,
}

impl SpatialHash {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            grids: HashMap::new(),
            object_positions: HashMap::new(),
            object_cells: HashMap::new(),
            frame_counter: 0,
        }
    }
    
    pub fn insert(&mut self, object_id: u32, position: Point3<f32>, bounds: AABB) -> RobinResult<()> {
        self.remove(object_id)?;
        
        let cells = self.get_overlapping_cells(&bounds);
        
        for cell in &cells {
            let grid = self.grids.entry(cell.clone()).or_insert_with(|| SpatialGrid::new(cell.clone()));
            grid.objects.insert(object_id);
            grid.last_accessed = self.frame_counter;
        }
        
        self.object_positions.insert(object_id, position);
        self.object_cells.insert(object_id, cells);
        
        Ok(())
    }
    
    pub fn remove(&mut self, object_id: u32) -> RobinResult<()> {
        if let Some(cells) = self.object_cells.remove(&object_id) {
            for cell in cells {
                if let Some(grid) = self.grids.get_mut(&cell) {
                    grid.objects.remove(&object_id);
                }
            }
        }
        
        self.object_positions.remove(&object_id);
        Ok(())
    }
    
    pub fn update(&mut self, object_id: u32, old_position: Point3<f32>, new_position: Point3<f32>, new_bounds: AABB) -> RobinResult<()> {
        let old_cells = self.get_overlapping_cells(&AABB::new(
            Point3::new(old_position.x - 1.0, old_position.y - 1.0, old_position.z - 1.0),
            Point3::new(old_position.x + 1.0, old_position.y + 1.0, old_position.z + 1.0),
        ));
        let new_cells = self.get_overlapping_cells(&new_bounds);
        
        if old_cells == new_cells {
            self.object_positions.insert(object_id, new_position);
            return Ok(());
        }
        
        for cell in &old_cells {
            if !new_cells.contains(cell) {
                if let Some(grid) = self.grids.get_mut(cell) {
                    grid.objects.remove(&object_id);
                }
            }
        }
        
        for cell in &new_cells {
            if !old_cells.contains(cell) {
                let grid = self.grids.entry(cell.clone()).or_insert_with(|| SpatialGrid::new(cell.clone()));
                grid.objects.insert(object_id);
                grid.last_accessed = self.frame_counter;
            }
        }
        
        self.object_positions.insert(object_id, new_position);
        self.object_cells.insert(object_id, new_cells);
        
        Ok(())
    }
    
    pub fn query(&self, bounds: &AABB) -> Vec<u32> {
        let cells = self.get_overlapping_cells(bounds);
        let mut results = HashSet::new();
        
        for cell in cells {
            if let Some(grid) = self.grids.get(&cell) {
                results.extend(&grid.objects);
            }
        }
        
        results.into_iter().collect()
    }
    
    pub fn query_sphere(&self, center: Point3<f32>, radius: f32) -> Vec<u32> {
        let sphere_bounds = AABB::new(
            Point3::new(center.x - radius, center.y - radius, center.z - radius),
            Point3::new(center.x + radius, center.y + radius, center.z + radius),
        );
        
        let candidates = self.query(&sphere_bounds);
        let radius_sq = radius * radius;
        
        candidates.into_iter()
            .filter(|&id| {
                if let Some(&position) = self.object_positions.get(&id) {
                    let distance_sq = (position - center).magnitude2();
                    distance_sq <= radius_sq
                } else {
                    false
                }
            })
            .collect()
    }
    
    pub fn get_nearby_objects(&self, position: Point3<f32>, radius: f32, max_results: usize) -> Vec<(u32, f32)> {
        let mut results = Vec::new();
        let candidates = self.query_sphere(position, radius);
        
        for object_id in candidates {
            if let Some(&obj_position) = self.object_positions.get(&object_id) {
                let distance = (obj_position - position).magnitude();
                results.push((object_id, distance));
            }
        }
        
        results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(max_results);
        results
    }
    
    fn get_overlapping_cells(&self, bounds: &AABB) -> Vec<GridCell> {
        let min_cell = GridCell::from_position(bounds.min, self.cell_size);
        let max_cell = GridCell::from_position(bounds.max, self.cell_size);
        
        let mut cells = Vec::new();
        
        for x in min_cell.x..=max_cell.x {
            for y in min_cell.y..=max_cell.y {
                for z in min_cell.z..=max_cell.z {
                    cells.push(GridCell::new(x, y, z));
                }
            }
        }
        
        cells
    }
    
    pub fn cleanup_empty_cells(&mut self) {
        let current_frame = self.frame_counter;
        let cleanup_threshold = 300;
        
        self.grids.retain(|_, grid| {
            !grid.objects.is_empty() || 
            (current_frame - grid.last_accessed) < cleanup_threshold
        });
    }
    
    pub fn get_active_cell_count(&self) -> usize {
        self.grids.len()
    }
    
    pub fn get_object_count(&self) -> usize {
        self.object_positions.len()
    }
    
    pub fn update_frame(&mut self) {
        self.frame_counter += 1;
    }
    
    pub fn get_cell_statistics(&self) -> SpatialHashStats {
        let total_cells = self.grids.len();
        let mut occupied_cells = 0;
        let mut total_objects = 0;
        let mut max_objects_per_cell = 0;
        let mut min_objects_per_cell = usize::MAX;
        
        for grid in self.grids.values() {
            let object_count = grid.objects.len();
            total_objects += object_count;
            
            if object_count > 0 {
                occupied_cells += 1;
                max_objects_per_cell = max_objects_per_cell.max(object_count);
                min_objects_per_cell = min_objects_per_cell.min(object_count);
            }
        }
        
        if occupied_cells == 0 {
            min_objects_per_cell = 0;
        }
        
        SpatialHashStats {
            total_cells,
            occupied_cells,
            empty_cells: total_cells - occupied_cells,
            total_objects,
            average_objects_per_cell: if occupied_cells > 0 { 
                total_objects as f32 / occupied_cells as f32 
            } else { 
                0.0 
            },
            max_objects_per_cell,
            min_objects_per_cell,
            cell_size: self.cell_size,
        }
    }
    
    pub fn get_cells_in_radius(&self, center: Point3<f32>, radius: f32) -> Vec<GridCell> {
        let radius_cells = (radius / self.cell_size).ceil() as i32;
        let center_cell = GridCell::from_position(center, self.cell_size);
        
        let mut cells = Vec::new();
        
        for x in (center_cell.x - radius_cells)..=(center_cell.x + radius_cells) {
            for y in (center_cell.y - radius_cells)..=(center_cell.y + radius_cells) {
                for z in (center_cell.z - radius_cells)..=(center_cell.z + radius_cells) {
                    let cell = GridCell::new(x, y, z);
                    let cell_center = cell.to_world_position(self.cell_size);
                    
                    let cell_size_half = self.cell_size / 2.0;
                    let cell_actual_center = Point3::new(
                        cell_center.x + cell_size_half,
                        cell_center.y + cell_size_half,
                        cell_center.z + cell_size_half,
                    );
                    
                    let distance = (cell_actual_center - center).magnitude();
                    if distance <= radius + (self.cell_size * 1.732) { // sqrt(3) for diagonal
                        cells.push(cell);
                    }
                }
            }
        }
        
        cells
    }
    
    pub fn debug_print_grid(&self, center: Point3<f32>, radius: f32) {
        let cells = self.get_cells_in_radius(center, radius);
        
        println!("Spatial Hash Debug - Center: {:?}, Radius: {:.2}", center, radius);
        println!("Cell Size: {:.2}", self.cell_size);
        println!("Active Cells in Range: {}", cells.len());
        
        for cell in &cells {
            if let Some(grid) = self.grids.get(cell) {
                if !grid.objects.is_empty() {
                    println!("  Cell ({}, {}, {}): {} objects", 
                             cell.x, cell.y, cell.z, grid.objects.len());
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct SpatialHashStats {
    pub total_cells: usize,
    pub occupied_cells: usize,
    pub empty_cells: usize,
    pub total_objects: usize,
    pub average_objects_per_cell: f32,
    pub max_objects_per_cell: usize,
    pub min_objects_per_cell: usize,
    pub cell_size: f32,
}