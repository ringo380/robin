// Hierarchical frustum culling system for optimal voxel chunk rendering
use cgmath::{Vector3, Point3, Matrix4};
use std::collections::HashMap;
use crate::logging::{LogCategory, log_debug, log_info};

/// Simple plane representation for frustum culling
#[derive(Debug, Clone)]
pub struct Plane {
    pub normal: Vector3<f32>,
    pub distance: f32,
}

impl Plane {
    pub fn new(a: f32, b: f32, c: f32, d: f32) -> Self {
        let normal = Vector3::new(a, b, c);
        let length = (a * a + b * b + c * c).sqrt();

        if length > 0.0 {
            Self {
                normal: normal / length,
                distance: d / length,
            }
        } else {
            Self {
                normal: Vector3::new(0.0, 1.0, 0.0),
                distance: 0.0,
            }
        }
    }

    pub fn normalize(self) -> Self {
        self // Already normalized in constructor
    }

    pub fn distance_to_point(&self, point: Point3<f32>) -> f32 {
        use cgmath::InnerSpace;
        self.normal.dot(Vector3::new(point.x, point.y, point.z)) + self.distance
    }
}

/// Axis-aligned bounding box for spatial queries
#[derive(Debug, Clone)]
pub struct AABB {
    pub min: Point3<f32>,
    pub max: Point3<f32>,
}

impl AABB {
    pub fn new(min: Point3<f32>, max: Point3<f32>) -> Self {
        Self { min, max }
    }

    pub fn from_chunk_coords(x: i32, y: i32, z: i32, chunk_size: f32) -> Self {
        let min = Point3::new(
            x as f32 * chunk_size,
            y as f32 * chunk_size,
            z as f32 * chunk_size,
        );
        let max = Point3::new(
            (x + 1) as f32 * chunk_size,
            (y + 1) as f32 * chunk_size,
            (z + 1) as f32 * chunk_size,
        );
        Self { min, max }
    }

    pub fn center(&self) -> Point3<f32> {
        Point3::new(
            (self.min.x + self.max.x) * 0.5,
            (self.min.y + self.max.y) * 0.5,
            (self.min.z + self.max.z) * 0.5,
        )
    }

    pub fn size(&self) -> Vector3<f32> {
        Vector3::new(
            self.max.x - self.min.x,
            self.max.y - self.min.y,
            self.max.z - self.min.z,
        )
    }

    pub fn contains_point(&self, point: &Point3<f32>) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y &&
        point.z >= self.min.z && point.z <= self.max.z
    }

    pub fn intersects(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x &&
        self.min.y <= other.max.y && self.max.y >= other.min.y &&
        self.min.z <= other.max.z && self.max.z >= other.min.z
    }

    pub fn merge(&self, other: &AABB) -> AABB {
        AABB {
            min: Point3::new(
                self.min.x.min(other.min.x),
                self.min.y.min(other.min.y),
                self.min.z.min(other.min.z),
            ),
            max: Point3::new(
                self.max.x.max(other.max.x),
                self.max.y.max(other.max.y),
                self.max.z.max(other.max.z),
            ),
        }
    }
}

/// Octree node for hierarchical spatial organization
#[derive(Debug, Clone)]
pub struct OctreeNode {
    pub bounds: AABB,
    pub chunks: Vec<ChunkId>,
    pub children: Option<Box<[OctreeNode; 8]>>,
    pub level: u32,
}

impl OctreeNode {
    pub fn new(bounds: AABB, level: u32) -> Self {
        Self {
            bounds,
            chunks: Vec::new(),
            children: None,
            level,
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.children.is_none()
    }

    pub fn subdivide(&mut self, max_depth: u32) {
        if self.level >= max_depth || self.chunks.len() <= 8 {
            return;
        }

        let center = self.bounds.center();
        let mut children = Vec::with_capacity(8);

        // Create 8 octants
        for i in 0..8 {
            let x_offset = if i & 1 != 0 { 1 } else { 0 };
            let y_offset = if i & 2 != 0 { 1 } else { 0 };
            let z_offset = if i & 4 != 0 { 1 } else { 0 };

            let min = Point3::new(
                if x_offset == 0 { self.bounds.min.x } else { center.x },
                if y_offset == 0 { self.bounds.min.y } else { center.y },
                if z_offset == 0 { self.bounds.min.z } else { center.z },
            );

            let max = Point3::new(
                if x_offset == 0 { center.x } else { self.bounds.max.x },
                if y_offset == 0 { center.y } else { self.bounds.max.y },
                if z_offset == 0 { center.z } else { self.bounds.max.z },
            );

            children.push(OctreeNode::new(AABB::new(min, max), self.level + 1));
        }

        self.children = Some(children.into_boxed_slice().try_into().unwrap());
    }

    pub fn insert_chunk(&mut self, chunk_id: ChunkId, chunk_bounds: &AABB, max_depth: u32) {
        if self.level < max_depth && self.chunks.len() >= 8 && self.children.is_none() {
            self.subdivide(max_depth);
        }

        if let Some(ref mut children) = self.children {
            // Find which child should contain this chunk
            for child in children.iter_mut() {
                if child.bounds.intersects(chunk_bounds) {
                    child.insert_chunk(chunk_id, chunk_bounds, max_depth);
                    return;
                }
            }
        }

        // Insert into this node if no appropriate child found
        self.chunks.push(chunk_id);
    }
}

/// Chunk identifier for spatial indexing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkId {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl ChunkId {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn distance_to_point(&self, point: &Point3<f32>, chunk_size: f32) -> f32 {
        let chunk_center = Point3::new(
            (self.x as f32 + 0.5) * chunk_size,
            (self.y as f32 + 0.5) * chunk_size,
            (self.z as f32 + 0.5) * chunk_size,
        );

        let dx = point.x - chunk_center.x;
        let dy = point.y - chunk_center.y;
        let dz = point.z - chunk_center.z;

        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

/// Frustum culling results
#[derive(Debug, Clone)]
pub enum CullResult {
    Inside,    // Completely inside frustum
    Outside,   // Completely outside frustum
    Intersect, // Partially inside frustum
}

/// Camera frustum for culling calculations
#[derive(Debug, Clone)]
pub struct CameraFrustum {
    pub planes: [Plane; 6], // Left, Right, Bottom, Top, Near, Far
}

impl CameraFrustum {
    pub fn from_view_projection(view_proj: &Matrix4<f32>) -> Self {
        // Extract frustum planes from view-projection matrix
        let m = view_proj;

        let planes = [
            // Left plane: m[3] + m[0]
            Plane::new(
                m[0][3] + m[0][0],
                m[1][3] + m[1][0],
                m[2][3] + m[2][0],
                m[3][3] + m[3][0],
            ),
            // Right plane: m[3] - m[0]
            Plane::new(
                m[0][3] - m[0][0],
                m[1][3] - m[1][0],
                m[2][3] - m[2][0],
                m[3][3] - m[3][0],
            ),
            // Bottom plane: m[3] + m[1]
            Plane::new(
                m[0][3] + m[0][1],
                m[1][3] + m[1][1],
                m[2][3] + m[2][1],
                m[3][3] + m[3][1],
            ),
            // Top plane: m[3] - m[1]
            Plane::new(
                m[0][3] - m[0][1],
                m[1][3] - m[1][1],
                m[2][3] - m[2][1],
                m[3][3] - m[3][1],
            ),
            // Near plane: m[3] + m[2]
            Plane::new(
                m[0][3] + m[0][2],
                m[1][3] + m[1][2],
                m[2][3] + m[2][2],
                m[3][3] + m[3][2],
            ),
            // Far plane: m[3] - m[2]
            Plane::new(
                m[0][3] - m[0][2],
                m[1][3] - m[1][2],
                m[2][3] - m[2][2],
                m[3][3] - m[3][2],
            ),
        ];

        Self { planes }
    }

    pub fn test_aabb(&self, bounds: &AABB) -> CullResult {
        let mut inside_count = 0;
        let corners = [
            bounds.min,
            Point3::new(bounds.max.x, bounds.min.y, bounds.min.z),
            Point3::new(bounds.min.x, bounds.max.y, bounds.min.z),
            Point3::new(bounds.max.x, bounds.max.y, bounds.min.z),
            Point3::new(bounds.min.x, bounds.min.y, bounds.max.z),
            Point3::new(bounds.max.x, bounds.min.y, bounds.max.z),
            Point3::new(bounds.min.x, bounds.max.y, bounds.max.z),
            bounds.max,
        ];

        for plane in &self.planes {
            let mut points_inside = 0;

            for corner in &corners {
                let distance = plane.distance_to_point(*corner);
                if distance >= 0.0 {
                    points_inside += 1;
                }
            }

            if points_inside == 0 {
                return CullResult::Outside;
            } else if points_inside == 8 {
                inside_count += 1;
            }
        }

        if inside_count == 6 {
            CullResult::Inside
        } else {
            CullResult::Intersect
        }
    }
}

/// Hierarchical frustum culling system
pub struct HierarchicalCuller {
    octree: OctreeNode,
    chunk_bounds: HashMap<ChunkId, AABB>,
    chunk_size: f32,
    max_octree_depth: u32,
    total_chunks: usize,
    culled_chunks: usize,
}

impl HierarchicalCuller {
    pub fn new(world_bounds: AABB, chunk_size: f32, max_octree_depth: u32) -> Self {
        log_info!(LogCategory::Performance, "Initializing hierarchical frustum culler");
        log_debug!(LogCategory::Performance, "World bounds: {:?}, Chunk size: {}, Max depth: {}", world_bounds, chunk_size, max_octree_depth);

        Self {
            octree: OctreeNode::new(world_bounds, 0),
            chunk_bounds: HashMap::new(),
            chunk_size,
            max_octree_depth,
            total_chunks: 0,
            culled_chunks: 0,
        }
    }

    pub fn register_chunk(&mut self, chunk_id: ChunkId) {
        let bounds = AABB::from_chunk_coords(chunk_id.x, chunk_id.y, chunk_id.z, self.chunk_size);
        self.chunk_bounds.insert(chunk_id, bounds.clone());
        self.octree.insert_chunk(chunk_id, &bounds, self.max_octree_depth);
        self.total_chunks += 1;

        log_debug!(LogCategory::Performance, "Registered chunk {:?} at bounds {:?}", chunk_id, bounds);
    }

    pub fn unregister_chunk(&mut self, chunk_id: ChunkId) {
        if self.chunk_bounds.remove(&chunk_id).is_some() {
            self.total_chunks = self.total_chunks.saturating_sub(1);
            // Note: For performance, we don't rebuild the octree immediately
            // Instead, we mark chunks as removed and rebuild periodically
            log_debug!(LogCategory::Performance, "Unregistered chunk {:?}", chunk_id);
        }
    }

    pub fn cull_chunks(&mut self, frustum: &CameraFrustum, camera_pos: &Point3<f32>) -> Vec<ChunkId> {
        self.culled_chunks = 0;
        let mut visible_chunks = Vec::new();

        // Clone the octree root to avoid borrowing issues
        let octree_root = self.octree.clone();
        let mut culled_count = 0;
        self.cull_node_immutable(&octree_root, frustum, camera_pos, &mut visible_chunks, &mut culled_count);
        self.culled_chunks = culled_count;

        let cull_percentage = if self.total_chunks > 0 {
            (self.culled_chunks as f32 / self.total_chunks as f32) * 100.0
        } else {
            0.0
        };

        log_debug!(LogCategory::Performance,
            "Culling complete: {}/{} chunks visible ({:.1}% culled)",
            visible_chunks.len(),
            self.total_chunks,
            cull_percentage
        );

        visible_chunks
    }

    fn cull_node_immutable(&self, node: &OctreeNode, frustum: &CameraFrustum, camera_pos: &Point3<f32>, visible_chunks: &mut Vec<ChunkId>, culled_chunks: &mut usize) {
        let cull_result = frustum.test_aabb(&node.bounds);

        match cull_result {
            CullResult::Outside => {
                // Entire node is outside frustum - cull all chunks
                let chunks_to_cull = self.count_chunks_in_node(node);
                *culled_chunks += chunks_to_cull;
                return;
            }
            CullResult::Inside => {
                // Entire node is inside frustum - add all chunks
                self.collect_all_chunks_in_node(node, visible_chunks);
                return;
            }
            CullResult::Intersect => {
                // Node intersects frustum - need to test children/chunks individually
            }
        }

        // If node has children, recurse into them
        if let Some(ref children) = node.children {
            for child in children.iter() {
                self.cull_node_immutable(child, frustum, camera_pos, visible_chunks, culled_chunks);
            }
        } else {
            // Leaf node - test individual chunks
            let mut local_culled = 0;
            for &chunk_id in &node.chunks {
                if let Some(chunk_bounds) = self.chunk_bounds.get(&chunk_id) {
                    match frustum.test_aabb(chunk_bounds) {
                        CullResult::Outside => {
                            local_culled += 1;
                        }
                        CullResult::Inside | CullResult::Intersect => {
                            visible_chunks.push(chunk_id);
                        }
                    }
                }
            }
            *culled_chunks += local_culled;
        }
    }

    fn cull_node(&mut self, node: &OctreeNode, frustum: &CameraFrustum, camera_pos: &Point3<f32>, visible_chunks: &mut Vec<ChunkId>) {
        let cull_result = frustum.test_aabb(&node.bounds);

        match cull_result {
            CullResult::Outside => {
                // Entire node is outside frustum - cull all chunks
                let chunks_to_cull = self.count_chunks_in_node(node);
                self.culled_chunks += chunks_to_cull;
                return;
            }
            CullResult::Inside => {
                // Entire node is inside frustum - add all chunks
                self.collect_all_chunks_in_node(node, visible_chunks);
                return;
            }
            CullResult::Intersect => {
                // Node intersects frustum - need to test children/chunks individually
            }
        }

        // If node has children, recurse into them
        if let Some(ref children) = node.children {
            for child in children.iter() {
                self.cull_node(child, frustum, camera_pos, visible_chunks);
            }
        } else {
            // Leaf node - test individual chunks
            let mut local_culled = 0;
            for &chunk_id in &node.chunks {
                if let Some(chunk_bounds) = self.chunk_bounds.get(&chunk_id) {
                    match frustum.test_aabb(chunk_bounds) {
                        CullResult::Outside => {
                            local_culled += 1;
                        }
                        CullResult::Inside | CullResult::Intersect => {
                            visible_chunks.push(chunk_id);
                        }
                    }
                }
            }
            self.culled_chunks += local_culled;
        }
    }

    fn count_chunks_in_node(&self, node: &OctreeNode) -> usize {
        let mut count = node.chunks.len();
        if let Some(ref children) = node.children {
            for child in children.iter() {
                count += self.count_chunks_in_node(child);
            }
        }
        count
    }

    fn collect_all_chunks_in_node(&self, node: &OctreeNode, visible_chunks: &mut Vec<ChunkId>) {
        visible_chunks.extend(&node.chunks);
        if let Some(ref children) = node.children {
            for child in children.iter() {
                self.collect_all_chunks_in_node(child, visible_chunks);
            }
        }
    }

    pub fn get_statistics(&self) -> CullingStatistics {
        CullingStatistics {
            total_chunks: self.total_chunks,
            visible_chunks: self.total_chunks - self.culled_chunks,
            culled_chunks: self.culled_chunks,
            cull_efficiency: if self.total_chunks > 0 {
                (self.culled_chunks as f32 / self.total_chunks as f32) * 100.0
            } else {
                0.0
            },
        }
    }

    pub fn rebuild_octree(&mut self) {
        log_info!(LogCategory::Performance, "Rebuilding octree with {} chunks", self.total_chunks);

        // Collect all active chunks
        let active_chunks: Vec<(ChunkId, AABB)> = self.chunk_bounds.iter()
            .map(|(&id, bounds)| (id, bounds.clone()))
            .collect();

        // Calculate world bounds from active chunks
        if let Some((first_id, first_bounds)) = active_chunks.first() {
            let mut world_bounds = first_bounds.clone();
            for (_, bounds) in &active_chunks[1..] {
                world_bounds = world_bounds.merge(bounds);
            }

            // Create new octree
            self.octree = OctreeNode::new(world_bounds, 0);

            // Re-insert all chunks
            for (chunk_id, bounds) in active_chunks {
                self.octree.insert_chunk(chunk_id, &bounds, self.max_octree_depth);
            }
        }

        log_info!(LogCategory::Performance, "Octree rebuild complete");
    }
}

#[derive(Debug, Clone)]
pub struct CullingStatistics {
    pub total_chunks: usize,
    pub visible_chunks: usize,
    pub culled_chunks: usize,
    pub cull_efficiency: f32,
}

impl CullingStatistics {
    pub fn format_summary(&self) -> String {
        format!(
            "Chunks: {}/{} visible ({:.1}% culled)",
            self.visible_chunks,
            self.total_chunks,
            self.cull_efficiency
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aabb_creation() {
        let aabb = AABB::from_chunk_coords(0, 0, 0, 32.0);
        assert_eq!(aabb.min, Point3::new(0.0, 0.0, 0.0));
        assert_eq!(aabb.max, Point3::new(32.0, 32.0, 32.0));
    }

    #[test]
    fn test_aabb_contains_point() {
        let aabb = AABB::new(Point3::new(0.0, 0.0, 0.0), Point3::new(10.0, 10.0, 10.0));
        assert!(aabb.contains_point(&Point3::new(5.0, 5.0, 5.0)));
        assert!(!aabb.contains_point(&Point3::new(15.0, 5.0, 5.0)));
    }

    #[test]
    fn test_aabb_intersection() {
        let aabb1 = AABB::new(Point3::new(0.0, 0.0, 0.0), Point3::new(10.0, 10.0, 10.0));
        let aabb2 = AABB::new(Point3::new(5.0, 5.0, 5.0), Point3::new(15.0, 15.0, 15.0));
        let aabb3 = AABB::new(Point3::new(20.0, 20.0, 20.0), Point3::new(30.0, 30.0, 30.0));

        assert!(aabb1.intersects(&aabb2));
        assert!(!aabb1.intersects(&aabb3));
    }

    #[test]
    fn test_chunk_distance() {
        let chunk = ChunkId::new(0, 0, 0);
        let point = Point3::new(16.0, 16.0, 16.0); // Center of chunk
        let distance = chunk.distance_to_point(&point, 32.0);
        assert!(distance < 0.1); // Should be very close to 0
    }

    #[test]
    fn test_hierarchical_culler_creation() {
        let world_bounds = AABB::new(
            Point3::new(-512.0, -512.0, -512.0),
            Point3::new(512.0, 512.0, 512.0)
        );
        let culler = HierarchicalCuller::new(world_bounds, 32.0, 4);
        assert_eq!(culler.total_chunks, 0);
        assert_eq!(culler.chunk_size, 32.0);
    }
}