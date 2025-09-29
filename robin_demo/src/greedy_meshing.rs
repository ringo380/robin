// Greedy meshing algorithm for voxel terrain optimization
// Combines adjacent voxel faces into larger quads to reduce vertex count

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub normal: [f32; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VoxelType {
    Air,
    Earth,
    Stone,
    Water,
    Grass,
    Sand,
    Wood,
    Crystal,
    Lava,
}

impl VoxelType {
    pub fn is_transparent(&self) -> bool {
        matches!(self, VoxelType::Air | VoxelType::Water)
    }

    pub fn color(&self) -> [f32; 3] {
        match self {
            VoxelType::Air => [0.0, 0.0, 0.0],
            VoxelType::Earth => [0.4, 0.2, 0.1],
            VoxelType::Stone => [0.5, 0.5, 0.5],
            VoxelType::Water => [0.2, 0.4, 0.8],
            VoxelType::Grass => [0.2, 0.6, 0.2],
            VoxelType::Sand => [0.8, 0.7, 0.4],
            VoxelType::Wood => [0.5, 0.3, 0.1],
            VoxelType::Crystal => [0.7, 0.3, 0.9],
            VoxelType::Lava => [0.9, 0.1, 0.0],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Face {
    Front,
    Back,
    Right,
    Left,
    Top,
    Bottom,
}

impl Face {
    pub fn normal(&self) -> [f32; 3] {
        match self {
            Face::Front => [0.0, 0.0, 1.0],
            Face::Back => [0.0, 0.0, -1.0],
            Face::Right => [1.0, 0.0, 0.0],
            Face::Left => [-1.0, 0.0, 0.0],
            Face::Top => [0.0, 1.0, 0.0],
            Face::Bottom => [0.0, -1.0, 0.0],
        }
    }

    pub fn axis(&self) -> usize {
        match self {
            Face::Front | Face::Back => 2, // Z-axis
            Face::Right | Face::Left => 0, // X-axis
            Face::Top | Face::Bottom => 1, // Y-axis
        }
    }

    pub fn direction(&self) -> i32 {
        match self {
            Face::Front | Face::Right | Face::Top => 1,
            Face::Back | Face::Left | Face::Bottom => -1,
        }
    }
}

/// A quad represents a merged rectangular area of faces
#[derive(Debug, Clone)]
pub struct Quad {
    pub min: [usize; 3],
    pub max: [usize; 3],
    pub face: Face,
    pub voxel_type: VoxelType,
}

impl Quad {
    pub fn vertices(&self) -> Vec<Vertex> {
        let color = self.voxel_type.color();
        let normal = self.face.normal();

        let min_x = self.min[0] as f32;
        let min_y = self.min[1] as f32;
        let min_z = self.min[2] as f32;
        let max_x = (self.max[0] + 1) as f32;
        let max_y = (self.max[1] + 1) as f32;
        let max_z = (self.max[2] + 1) as f32;

        match self.face {
            Face::Top => vec![
                Vertex { position: [min_x, max_y, min_z], color, normal },
                Vertex { position: [max_x, max_y, min_z], color, normal },
                Vertex { position: [max_x, max_y, max_z], color, normal },
                Vertex { position: [min_x, max_y, max_z], color, normal },
            ],
            Face::Bottom => vec![
                Vertex { position: [min_x, min_y, max_z], color, normal },
                Vertex { position: [max_x, min_y, max_z], color, normal },
                Vertex { position: [max_x, min_y, min_z], color, normal },
                Vertex { position: [min_x, min_y, min_z], color, normal },
            ],
            Face::Right => vec![
                Vertex { position: [max_x, min_y, min_z], color, normal },
                Vertex { position: [max_x, min_y, max_z], color, normal },
                Vertex { position: [max_x, max_y, max_z], color, normal },
                Vertex { position: [max_x, max_y, min_z], color, normal },
            ],
            Face::Left => vec![
                Vertex { position: [min_x, min_y, max_z], color, normal },
                Vertex { position: [min_x, min_y, min_z], color, normal },
                Vertex { position: [min_x, max_y, min_z], color, normal },
                Vertex { position: [min_x, max_y, max_z], color, normal },
            ],
            Face::Front => vec![
                Vertex { position: [min_x, min_y, max_z], color, normal },
                Vertex { position: [max_x, min_y, max_z], color, normal },
                Vertex { position: [max_x, max_y, max_z], color, normal },
                Vertex { position: [min_x, max_y, max_z], color, normal },
            ],
            Face::Back => vec![
                Vertex { position: [max_x, min_y, min_z], color, normal },
                Vertex { position: [min_x, min_y, min_z], color, normal },
                Vertex { position: [min_x, max_y, min_z], color, normal },
                Vertex { position: [max_x, max_y, min_z], color, normal },
            ],
        }
    }
}

/// Greedy meshing implementation for voxel chunks
pub struct GreedyMesher {
    chunk_size: usize,
}

impl GreedyMesher {
    pub fn new(chunk_size: usize) -> Self {
        Self { chunk_size }
    }

    /// Generate optimized mesh using greedy meshing algorithm
    pub fn generate_mesh(
        &self,
        chunks: &HashMap<(i32, i32, i32), &Vec<VoxelType>>,
        get_voxel: impl Fn(i32, i32, i32) -> VoxelType,
    ) -> (Vec<Vertex>, Vec<u32>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for (&(chunk_x, chunk_y, chunk_z), chunk_data) in chunks {
            let quads = self.generate_chunk_quads(chunk_data, chunk_x, chunk_y, chunk_z, &get_voxel);

            for quad in quads {
                let base_index = vertices.len() as u32;
                vertices.extend(quad.vertices());

                // Add indices for two triangles forming the quad
                indices.extend(&[
                    base_index,     base_index + 1, base_index + 2,
                    base_index,     base_index + 2, base_index + 3,
                ]);
            }
        }

        (vertices, indices)
    }

    /// Generate quads for a single chunk using greedy meshing
    fn generate_chunk_quads(
        &self,
        chunk_data: &[VoxelType],
        chunk_x: i32,
        chunk_y: i32,
        chunk_z: i32,
        get_voxel: impl Fn(i32, i32, i32) -> VoxelType,
    ) -> Vec<Quad> {
        let mut quads = Vec::new();

        // Process each face direction
        for &face in &[Face::Front, Face::Back, Face::Right, Face::Left, Face::Top, Face::Bottom] {
            let face_quads = self.generate_face_quads(
                chunk_data, chunk_x, chunk_y, chunk_z, face, &get_voxel
            );
            quads.extend(face_quads);
        }

        quads
    }

    /// Generate quads for a specific face direction
    fn generate_face_quads(
        &self,
        chunk_data: &[VoxelType],
        chunk_x: i32,
        chunk_y: i32,
        chunk_z: i32,
        face: Face,
        get_voxel: impl Fn(i32, i32, i32) -> VoxelType,
    ) -> Vec<Quad> {
        let mut quads = Vec::new();
        let mut mask = vec![None; self.chunk_size * self.chunk_size];

        let axis = face.axis();
        let direction = face.direction();

        // Iterate through each slice perpendicular to the face direction
        for slice in 0..self.chunk_size {
            // Clear the mask for this slice
            mask.fill(None);

            // Fill the mask with voxel types that need faces
            for u in 0..self.chunk_size {
                for v in 0..self.chunk_size {
                    let mut pos = [0; 3];
                    match axis {
                        0 => { pos = [slice, u, v]; }, // X-axis (Right/Left faces)
                        1 => { pos = [u, slice, v]; }, // Y-axis (Top/Bottom faces)
                        2 => { pos = [u, v, slice]; }, // Z-axis (Front/Back faces)
                        _ => unreachable!(),
                    }

                    let voxel = self.get_chunk_voxel(chunk_data, pos[0], pos[1], pos[2]);
                    if voxel == VoxelType::Air {
                        continue;
                    }

                    // Calculate neighbor position
                    let world_x = chunk_x * self.chunk_size as i32 + pos[0] as i32;
                    let world_y = chunk_y * self.chunk_size as i32 + pos[1] as i32;
                    let world_z = chunk_z * self.chunk_size as i32 + pos[2] as i32;

                    let neighbor_pos = match face {
                        Face::Front => [world_x, world_y, world_z + 1],
                        Face::Back => [world_x, world_y, world_z - 1],
                        Face::Right => [world_x + 1, world_y, world_z],
                        Face::Left => [world_x - 1, world_y, world_z],
                        Face::Top => [world_x, world_y + 1, world_z],
                        Face::Bottom => [world_x, world_y - 1, world_z],
                    };

                    let neighbor = get_voxel(neighbor_pos[0], neighbor_pos[1], neighbor_pos[2]);

                    // Add face if neighbor is transparent or different material
                    if neighbor.is_transparent() || neighbor != voxel {
                        mask[v * self.chunk_size + u] = Some(voxel);
                    }
                }
            }

            // Greedy meshing: merge adjacent faces in the mask
            let slice_quads = self.mesh_slice(&mask, slice, face, chunk_x, chunk_y, chunk_z);
            quads.extend(slice_quads);
        }

        quads
    }

    /// Perform greedy meshing on a 2D slice
    fn mesh_slice(
        &self,
        mask: &[Option<VoxelType>],
        slice: usize,
        face: Face,
        chunk_x: i32,
        chunk_y: i32,
        chunk_z: i32,
    ) -> Vec<Quad> {
        let mut quads = Vec::new();
        let mut processed = vec![false; self.chunk_size * self.chunk_size];

        for v in 0..self.chunk_size {
            for u in 0..self.chunk_size {
                let index = v * self.chunk_size + u;

                if processed[index] || mask[index].is_none() {
                    continue;
                }

                let voxel_type = mask[index].unwrap();

                // Find the width of the quad (how far we can extend in u direction)
                let mut width = 1;
                while u + width < self.chunk_size {
                    let next_index = v * self.chunk_size + (u + width);
                    if processed[next_index] || mask[next_index] != Some(voxel_type) {
                        break;
                    }
                    width += 1;
                }

                // Find the height of the quad (how far we can extend in v direction)
                let mut height = 1;
                'outer: while v + height < self.chunk_size {
                    // Check if the entire row can be merged
                    for w in 0..width {
                        let check_index = (v + height) * self.chunk_size + (u + w);
                        if processed[check_index] || mask[check_index] != Some(voxel_type) {
                            break 'outer;
                        }
                    }
                    height += 1;
                }

                // Mark all cells in the quad as processed
                for dv in 0..height {
                    for du in 0..width {
                        let mark_index = (v + dv) * self.chunk_size + (u + du);
                        processed[mark_index] = true;
                    }
                }

                // Create the quad
                let mut min = [0; 3];
                let mut max = [0; 3];

                let world_x_base = chunk_x * self.chunk_size as i32;
                let world_y_base = chunk_y * self.chunk_size as i32;
                let world_z_base = chunk_z * self.chunk_size as i32;

                match face.axis() {
                    0 => { // X-axis (Right/Left faces)
                        min = [
                            (world_x_base + slice as i32) as usize,
                            (world_y_base + u as i32) as usize,
                            (world_z_base + v as i32) as usize,
                        ];
                        max = [
                            (world_x_base + slice as i32) as usize,
                            (world_y_base + u as i32 + width as i32 - 1) as usize,
                            (world_z_base + v as i32 + height as i32 - 1) as usize,
                        ];
                    },
                    1 => { // Y-axis (Top/Bottom faces)
                        min = [
                            (world_x_base + u as i32) as usize,
                            (world_y_base + slice as i32) as usize,
                            (world_z_base + v as i32) as usize,
                        ];
                        max = [
                            (world_x_base + u as i32 + width as i32 - 1) as usize,
                            (world_y_base + slice as i32) as usize,
                            (world_z_base + v as i32 + height as i32 - 1) as usize,
                        ];
                    },
                    2 => { // Z-axis (Front/Back faces)
                        min = [
                            (world_x_base + u as i32) as usize,
                            (world_y_base + v as i32) as usize,
                            (world_z_base + slice as i32) as usize,
                        ];
                        max = [
                            (world_x_base + u as i32 + width as i32 - 1) as usize,
                            (world_y_base + v as i32 + height as i32 - 1) as usize,
                            (world_z_base + slice as i32) as usize,
                        ];
                    },
                    _ => unreachable!(),
                }

                quads.push(Quad {
                    min,
                    max,
                    face,
                    voxel_type,
                });
            }
        }

        quads
    }

    /// Get voxel from chunk data
    fn get_chunk_voxel(&self, chunk_data: &[VoxelType], x: usize, y: usize, z: usize) -> VoxelType {
        if x >= self.chunk_size || y >= self.chunk_size || z >= self.chunk_size {
            return VoxelType::Air;
        }

        let index = z * self.chunk_size * self.chunk_size + y * self.chunk_size + x;
        chunk_data.get(index).copied().unwrap_or(VoxelType::Air)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greedy_meshing_simple() {
        let mesher = GreedyMesher::new(4);

        // Create a simple 2x2x2 cube of stone
        let mut chunk = vec![VoxelType::Air; 4 * 4 * 4];
        for x in 0..2 {
            for y in 0..2 {
                for z in 0..2 {
                    let index = z * 16 + y * 4 + x;
                    chunk[index] = VoxelType::Stone;
                }
            }
        }

        let mut chunks = HashMap::new();
        chunks.insert((0, 0, 0), &chunk);

        let get_voxel = |_: i32, _: i32, _: i32| VoxelType::Air;
        let (vertices, indices) = mesher.generate_mesh(&chunks, get_voxel);

        // Should create fewer vertices than naive approach
        // Naive: 2*2*2 voxels * 6 faces * 4 vertices = 192 vertices
        // Greedy: 6 faces * 4 vertices = 24 vertices (ideal case)
        assert!(vertices.len() < 100, "Greedy meshing should reduce vertex count significantly");
        assert_eq!(indices.len() % 6, 0, "Indices should be multiple of 6 (2 triangles per quad)");
    }

    #[test]
    fn test_quad_vertices() {
        let quad = Quad {
            min: [0, 0, 0],
            max: [1, 1, 1],
            face: Face::Top,
            voxel_type: VoxelType::Stone,
        };

        let vertices = quad.vertices();
        assert_eq!(vertices.len(), 4);

        // Check that all vertices have the same normal (top face)
        for vertex in &vertices {
            assert_eq!(vertex.normal, [0.0, 1.0, 0.0]);
        }
    }

    #[test]
    fn test_voxel_type_properties() {
        assert!(VoxelType::Air.is_transparent());
        assert!(VoxelType::Water.is_transparent());
        assert!(!VoxelType::Stone.is_transparent());

        let stone_color = VoxelType::Stone.color();
        assert_eq!(stone_color, [0.5, 0.5, 0.5]);
    }
}