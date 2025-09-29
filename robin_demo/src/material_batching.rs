// Material batching system for optimizing draw calls
// Groups vertices by material type and renders them in batches

use std::collections::HashMap;
use crate::renderer::{Mesh, Vertex};
use crate::greedy_meshing::VoxelType;
use metal::*;

/// Material identifier for batching similar materials together
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum MaterialType {
    Earth,
    Stone,
    Water,
    Grass,
    Sand,
    Wood,
    Crystal,
    Lava,
    Air,
}

impl MaterialType {
    /// Get material type from voxel type
    pub fn from_voxel_type(voxel_type: VoxelType) -> Self {
        match voxel_type {
            VoxelType::Earth => MaterialType::Earth,
            VoxelType::Stone => MaterialType::Stone,
            VoxelType::Water => MaterialType::Water,
            VoxelType::Grass => MaterialType::Grass,
            VoxelType::Sand => MaterialType::Sand,
            VoxelType::Wood => MaterialType::Wood,
            VoxelType::Crystal => MaterialType::Crystal,
            VoxelType::Lava => MaterialType::Lava,
            VoxelType::Air => MaterialType::Air,
        }
    }

    /// Get material color for identification
    pub fn color(&self) -> [f32; 3] {
        match self {
            MaterialType::Earth => [0.4, 0.2, 0.1],
            MaterialType::Stone => [0.5, 0.5, 0.5],
            MaterialType::Water => [0.2, 0.4, 0.8],
            MaterialType::Grass => [0.2, 0.6, 0.2],
            MaterialType::Sand => [0.8, 0.7, 0.4],
            MaterialType::Wood => [0.5, 0.3, 0.1],
            MaterialType::Crystal => [0.7, 0.3, 0.9],
            MaterialType::Lava => [0.9, 0.1, 0.0],
            MaterialType::Air => [0.0, 0.0, 0.0],
        }
    }

    /// Check if material is opaque (affects rendering order)
    pub fn is_opaque(&self) -> bool {
        !matches!(self, MaterialType::Water | MaterialType::Air)
    }

    /// Get rendering priority (lower = render first)
    pub fn render_priority(&self) -> u32 {
        match self {
            // Opaque materials first
            MaterialType::Stone => 0,
            MaterialType::Earth => 1,
            MaterialType::Grass => 2,
            MaterialType::Sand => 3,
            MaterialType::Wood => 4,
            MaterialType::Crystal => 5,
            MaterialType::Lava => 6,
            // Transparent materials last
            MaterialType::Water => 10,
            MaterialType::Air => 11,
        }
    }
}

/// Represents a batch of vertices for a specific material
#[derive(Debug, Clone)]
pub struct MaterialBatch {
    pub material: MaterialType,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub vertex_buffer: Option<Buffer>,
    pub index_buffer: Option<Buffer>,
    pub draw_calls_saved: usize,
}

impl MaterialBatch {
    pub fn new(material: MaterialType) -> Self {
        Self {
            material,
            vertices: Vec::new(),
            indices: Vec::new(),
            vertex_buffer: None,
            index_buffer: None,
            draw_calls_saved: 0,
        }
    }

    /// Add vertices from a mesh to this batch
    pub fn add_mesh_data(&mut self, vertices: &[Vertex], indices: &[u32]) {
        let vertex_offset = self.vertices.len() as u32;

        // Add vertices
        self.vertices.extend_from_slice(vertices);

        // Add indices with offset
        for &index in indices {
            self.indices.push(index + vertex_offset);
        }

        self.draw_calls_saved += 1;
    }

    /// Create GPU buffers for this batch
    pub fn create_buffers(&mut self, device: &Device) -> Result<(), String> {
        if self.vertices.is_empty() {
            return Ok(());
        }

        // Create vertex buffer
        let vertex_buffer = device.new_buffer_with_data(
            self.vertices.as_ptr() as *const std::ffi::c_void,
            (self.vertices.len() * std::mem::size_of::<Vertex>()) as u64,
            MTLResourceOptions::CPUCacheModeDefaultCache,
        );
        self.vertex_buffer = Some(vertex_buffer);

        // Create index buffer
        let index_buffer = device.new_buffer_with_data(
            self.indices.as_ptr() as *const std::ffi::c_void,
            (self.indices.len() * std::mem::size_of::<u32>()) as u64,
            MTLResourceOptions::CPUCacheModeDefaultCache,
        );
        self.index_buffer = Some(index_buffer);

        Ok(())
    }

    /// Get the number of triangles in this batch
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Clear the batch data
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.vertex_buffer = None;
        self.index_buffer = None;
        self.draw_calls_saved = 0;
    }
}

/// Material batching system that groups vertices by material
pub struct MaterialBatcher {
    batches: HashMap<MaterialType, MaterialBatch>,
    total_draw_calls_saved: usize,
    stats: BatchingStats,
}

impl MaterialBatcher {
    pub fn new() -> Self {
        Self {
            batches: HashMap::new(),
            total_draw_calls_saved: 0,
            stats: BatchingStats::new(),
        }
    }

    /// Add a mesh to the appropriate material batch
    pub fn add_mesh(&mut self, mesh: &Mesh) {
        // Extract material information from vertices
        // Since vertices are grouped by material in the greedy meshing,
        // we can determine material from the first vertex
        if mesh.vertices.is_empty() {
            return;
        }

        let material = self.identify_material_from_vertex(&mesh.vertices[0]);
        let batch = self.batches.entry(material).or_insert_with(|| MaterialBatch::new(material));

        batch.add_mesh_data(&mesh.vertices, &mesh.indices);
        self.stats.meshes_added += 1;
        self.stats.vertices_processed += mesh.vertices.len();
        self.stats.indices_processed += mesh.indices.len();
    }

    /// Identify material type from vertex color
    fn identify_material_from_vertex(&self, vertex: &Vertex) -> MaterialType {
        let color = vertex.color;

        // Match color to material type (with tolerance for floating point precision)
        let tolerance = 0.01;

        for material in [
            MaterialType::Earth, MaterialType::Stone, MaterialType::Water,
            MaterialType::Grass, MaterialType::Sand, MaterialType::Wood,
            MaterialType::Crystal, MaterialType::Lava,
        ] {
            let mat_color = material.color();
            if (color[0] - mat_color[0]).abs() < tolerance &&
               (color[1] - mat_color[1]).abs() < tolerance &&
               (color[2] - mat_color[2]).abs() < tolerance {
                return material;
            }
        }

        // Default to stone if no match
        MaterialType::Stone
    }

    /// Finalize batches by creating GPU buffers
    pub fn finalize_batches(&mut self, device: &Device) -> Result<(), String> {
        let mut total_saved = 0;

        for (_, batch) in self.batches.iter_mut() {
            batch.create_buffers(device)?;
            total_saved += batch.draw_calls_saved.saturating_sub(1); // -1 because we still need 1 draw call
        }

        self.total_draw_calls_saved = total_saved;
        self.stats.batches_created = self.batches.len();
        self.stats.draw_calls_saved = total_saved;

        log::info!("🎨 Material batching: {} batches created, {} draw calls saved",
                  self.batches.len(), total_saved);

        Ok(())
    }

    /// Get batches sorted by render priority (opaque first, then transparent)
    pub fn get_sorted_batches(&self) -> Vec<&MaterialBatch> {
        let mut batches: Vec<&MaterialBatch> = self.batches.values().collect();
        batches.sort_by_key(|batch| batch.material.render_priority());
        batches
    }

    /// Get batches for a specific material type
    pub fn get_batch(&self, material: MaterialType) -> Option<&MaterialBatch> {
        self.batches.get(&material)
    }

    /// Get all batches
    pub fn get_all_batches(&self) -> &HashMap<MaterialType, MaterialBatch> {
        &self.batches
    }

    /// Clear all batches
    pub fn clear(&mut self) {
        self.batches.clear();
        self.total_draw_calls_saved = 0;
        self.stats = BatchingStats::new();
    }

    /// Get batching statistics
    pub fn get_stats(&self) -> &BatchingStats {
        &self.stats
    }

    /// Get total draw calls saved
    pub fn get_draw_calls_saved(&self) -> usize {
        self.total_draw_calls_saved
    }

    /// Get current batch count
    pub fn get_batch_count(&self) -> usize {
        self.batches.len()
    }
}

impl Default for MaterialBatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for material batching performance
#[derive(Debug, Clone)]
pub struct BatchingStats {
    pub meshes_added: usize,
    pub vertices_processed: usize,
    pub indices_processed: usize,
    pub batches_created: usize,
    pub draw_calls_saved: usize,
}

impl BatchingStats {
    pub fn new() -> Self {
        Self {
            meshes_added: 0,
            vertices_processed: 0,
            indices_processed: 0,
            batches_created: 0,
            draw_calls_saved: 0,
        }
    }

    /// Calculate batching efficiency as a percentage
    pub fn efficiency_percentage(&self) -> f32 {
        if self.meshes_added == 0 {
            return 0.0;
        }
        (self.draw_calls_saved as f32 / self.meshes_added as f32) * 100.0
    }

    /// Get a formatted summary of the statistics
    pub fn format_summary(&self) -> String {
        format!(
            "Batching: {} meshes → {} batches ({:.1}% fewer draws), {} vertices, {} indices",
            self.meshes_added,
            self.batches_created,
            self.efficiency_percentage(),
            self.vertices_processed,
            self.indices_processed
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_type_from_voxel() {
        assert_eq!(MaterialType::from_voxel_type(VoxelType::Stone), MaterialType::Stone);
        assert_eq!(MaterialType::from_voxel_type(VoxelType::Water), MaterialType::Water);
        assert_eq!(MaterialType::from_voxel_type(VoxelType::Grass), MaterialType::Grass);
    }

    #[test]
    fn test_material_render_priority() {
        assert!(MaterialType::Stone.render_priority() < MaterialType::Water.render_priority());
        assert!(MaterialType::Earth.render_priority() < MaterialType::Water.render_priority());
        assert!(MaterialType::Crystal.render_priority() < MaterialType::Water.render_priority());
    }

    #[test]
    fn test_material_opacity() {
        assert!(MaterialType::Stone.is_opaque());
        assert!(MaterialType::Earth.is_opaque());
        assert!(!MaterialType::Water.is_opaque());
        assert!(!MaterialType::Air.is_opaque());
    }

    #[test]
    fn test_material_batch_creation() {
        let mut batch = MaterialBatch::new(MaterialType::Stone);
        assert_eq!(batch.material, MaterialType::Stone);
        assert!(batch.vertices.is_empty());
        assert!(batch.indices.is_empty());
        assert_eq!(batch.draw_calls_saved, 0);
    }

    #[test]
    fn test_batch_add_mesh_data() {
        let mut batch = MaterialBatch::new(MaterialType::Stone);

        let vertices = vec![
            Vertex::new([0.0, 0.0, 0.0], [0.5, 0.5, 0.5], [0.0, 1.0, 0.0]),
            Vertex::new([1.0, 0.0, 0.0], [0.5, 0.5, 0.5], [0.0, 1.0, 0.0]),
        ];
        let indices = vec![0, 1, 2];

        batch.add_mesh_data(&vertices, &indices);

        assert_eq!(batch.vertices.len(), 2);
        assert_eq!(batch.indices.len(), 3);
        assert_eq!(batch.draw_calls_saved, 1);
    }

    #[test]
    fn test_batching_stats() {
        let mut stats = BatchingStats::new();
        stats.meshes_added = 10;
        stats.draw_calls_saved = 6;

        assert_eq!(stats.efficiency_percentage(), 60.0);
    }

    #[test]
    fn test_material_color_identification() {
        let batcher = MaterialBatcher::new();

        // Test stone material identification
        let stone_vertex = Vertex::new([0.0, 0.0, 0.0], [0.5, 0.5, 0.5], [0.0, 1.0, 0.0]);
        assert_eq!(batcher.identify_material_from_vertex(&stone_vertex), MaterialType::Stone);

        // Test grass material identification
        let grass_vertex = Vertex::new([0.0, 0.0, 0.0], [0.2, 0.6, 0.2], [0.0, 1.0, 0.0]);
        assert_eq!(batcher.identify_material_from_vertex(&grass_vertex), MaterialType::Grass);
    }
}