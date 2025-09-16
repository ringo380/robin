use crate::engine::spatial::octree::AABB;
use cgmath::{Vector3, Vector4, Matrix4, Point3, InnerSpace, EuclideanSpace};

#[derive(Debug, Clone, Copy)]
pub struct Plane {
    pub normal: Vector3<f32>,
    pub distance: f32,
}

impl Plane {
    pub fn new(normal: Vector3<f32>, distance: f32) -> Self {
        Self { normal, distance }
    }
    
    pub fn from_point_normal(point: Point3<f32>, normal: Vector3<f32>) -> Self {
        let normalized = normal.normalize();
        let distance = normalized.dot(point.to_vec());
        Self {
            normal: normalized,
            distance,
        }
    }
    
    pub fn distance_to_point(&self, point: Point3<f32>) -> f32 {
        self.normal.dot(point.to_vec()) - self.distance
    }
    
    pub fn normalize(&mut self) {
        let magnitude = self.normal.magnitude();
        if magnitude > 0.0 {
            self.normal /= magnitude;
            self.distance /= magnitude;
        }
    }
}

#[derive(Debug, Clone)]
pub struct Frustum {
    pub planes: [Plane; 6], // left, right, bottom, top, near, far
}

impl Frustum {
    pub fn new() -> Self {
        Self {
            planes: [
                Plane::new(Vector3::new(1.0, 0.0, 0.0), 0.0), // left
                Plane::new(Vector3::new(-1.0, 0.0, 0.0), 0.0), // right
                Plane::new(Vector3::new(0.0, 1.0, 0.0), 0.0), // bottom
                Plane::new(Vector3::new(0.0, -1.0, 0.0), 0.0), // top
                Plane::new(Vector3::new(0.0, 0.0, 1.0), 0.0), // near
                Plane::new(Vector3::new(0.0, 0.0, -1.0), 0.0), // far
            ],
        }
    }
    
    pub fn extract_from_matrix(view_projection: &Matrix4<f32>) -> Self {
        let m = view_projection;
        
        let mut frustum = Self::new();
        
        // Left plane
        frustum.planes[0] = Plane::new(
            Vector3::new(m.w.x + m.x.x, m.w.y + m.x.y, m.w.z + m.x.z),
            m.w.w + m.x.w
        );
        
        // Right plane
        frustum.planes[1] = Plane::new(
            Vector3::new(m.w.x - m.x.x, m.w.y - m.x.y, m.w.z - m.x.z),
            m.w.w - m.x.w
        );
        
        // Bottom plane
        frustum.planes[2] = Plane::new(
            Vector3::new(m.w.x + m.y.x, m.w.y + m.y.y, m.w.z + m.y.z),
            m.w.w + m.y.w
        );
        
        // Top plane
        frustum.planes[3] = Plane::new(
            Vector3::new(m.w.x - m.y.x, m.w.y - m.y.y, m.w.z - m.y.z),
            m.w.w - m.y.w
        );
        
        // Near plane
        frustum.planes[4] = Plane::new(
            Vector3::new(m.w.x + m.z.x, m.w.y + m.z.y, m.w.z + m.z.z),
            m.w.w + m.z.w
        );
        
        // Far plane
        frustum.planes[5] = Plane::new(
            Vector3::new(m.w.x - m.z.x, m.w.y - m.z.y, m.w.z - m.z.z),
            m.w.w - m.z.w
        );
        
        // Normalize all planes
        for plane in &mut frustum.planes {
            plane.normalize();
        }
        
        frustum
    }
    
    pub fn contains_point(&self, point: Point3<f32>) -> bool {
        for plane in &self.planes {
            if plane.distance_to_point(point) < 0.0 {
                return false;
            }
        }
        true
    }
    
    pub fn intersects_sphere(&self, center: Point3<f32>, radius: f32) -> bool {
        for plane in &self.planes {
            let distance = plane.distance_to_point(center);
            if distance < -radius {
                return false;
            }
        }
        true
    }
    
    pub fn intersects_aabb(&self, aabb: &AABB) -> bool {
        for plane in &self.planes {
            let mut positive_vertex = aabb.min;
            let mut negative_vertex = aabb.max;
            
            if plane.normal.x >= 0.0 {
                positive_vertex.x = aabb.max.x;
                negative_vertex.x = aabb.min.x;
            }
            if plane.normal.y >= 0.0 {
                positive_vertex.y = aabb.max.y;
                negative_vertex.y = aabb.min.y;
            }
            if plane.normal.z >= 0.0 {
                positive_vertex.z = aabb.max.z;
                negative_vertex.z = aabb.min.z;
            }
            
            if plane.distance_to_point(positive_vertex) < 0.0 {
                return false;
            }
        }
        true
    }
    
    pub fn classify_aabb(&self, aabb: &AABB) -> FrustumClassification {
        let mut inside_count = 0;
        
        for plane in &self.planes {
            let mut positive_vertex = aabb.min;
            let mut negative_vertex = aabb.max;
            
            if plane.normal.x >= 0.0 {
                positive_vertex.x = aabb.max.x;
                negative_vertex.x = aabb.min.x;
            }
            if plane.normal.y >= 0.0 {
                positive_vertex.y = aabb.max.y;
                negative_vertex.y = aabb.min.y;
            }
            if plane.normal.z >= 0.0 {
                positive_vertex.z = aabb.max.z;
                negative_vertex.z = aabb.min.z;
            }
            
            if plane.distance_to_point(negative_vertex) > 0.0 {
                inside_count += 1;
            } else if plane.distance_to_point(positive_vertex) < 0.0 {
                return FrustumClassification::Outside;
            }
        }
        
        if inside_count == 6 {
            FrustumClassification::Inside
        } else {
            FrustumClassification::Intersecting
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FrustumClassification {
    Inside,
    Outside,
    Intersecting,
}

pub struct FrustumCuller {
    current_frustum: Frustum,
    cull_distance: f32,
    culling_stats: CullingStats,
}

impl FrustumCuller {
    pub fn new() -> Self {
        Self {
            current_frustum: Frustum::new(),
            cull_distance: 1000.0,
            culling_stats: CullingStats::default(),
        }
    }
    
    pub fn extract_frustum(&mut self, view_matrix: &Matrix4<f32>, projection_matrix: &Matrix4<f32>) -> &Frustum {
        let view_projection = projection_matrix * view_matrix;
        self.current_frustum = Frustum::extract_from_matrix(&view_projection);
        &self.current_frustum
    }
    
    pub fn is_aabb_in_frustum(&mut self, frustum: &Frustum, aabb: &AABB) -> bool {
        self.culling_stats.total_tests += 1;

        let result = Self::is_aabb_in_frustum_static(frustum, aabb);

        if result {
            self.culling_stats.objects_passed += 1;
        } else {
            self.culling_stats.objects_culled += 1;
        }

        result
    }

    pub fn is_aabb_in_frustum_static(frustum: &Frustum, aabb: &AABB) -> bool {
        frustum.intersects_aabb(aabb)
    }
    
    pub fn is_sphere_in_frustum(&mut self, frustum: &Frustum, center: Point3<f32>, radius: f32) -> bool {
        self.culling_stats.total_tests += 1;
        
        let result = frustum.intersects_sphere(center, radius);
        
        if result {
            self.culling_stats.objects_passed += 1;
        } else {
            self.culling_stats.objects_culled += 1;
        }
        
        result
    }
    
    pub fn distance_cull(&mut self, camera_position: Point3<f32>, object_position: Point3<f32>) -> bool {
        self.culling_stats.total_tests += 1;
        
        let distance = (object_position - camera_position).magnitude();
        let result = distance <= self.cull_distance;
        
        if result {
            self.culling_stats.objects_passed += 1;
        } else {
            self.culling_stats.objects_culled += 1;
        }
        
        result
    }
    
    pub fn cull_objects<'a, T, F>(&mut self, 
                              objects: &'a [T], 
                              camera_position: Point3<f32>,
                              get_bounds: F) -> Vec<&'a T>
    where
        F: Fn(&T) -> (Point3<f32>, AABB),
    {
        let mut visible_objects = Vec::new();
        
        for object in objects {
            let (position, bounds) = get_bounds(object);
            
            // Distance culling
            if !self.distance_cull(camera_position, position) {
                continue;
            }
            
            // Frustum culling
            if !Self::is_aabb_in_frustum_static(&self.current_frustum, &bounds) {
                continue;
            }
            
            visible_objects.push(object);
        }
        
        visible_objects
    }
    
    pub fn set_cull_distance(&mut self, distance: f32) {
        self.cull_distance = distance;
    }
    
    pub fn get_cull_distance(&self) -> f32 {
        self.cull_distance
    }
    
    pub fn get_culling_stats(&self) -> &CullingStats {
        &self.culling_stats
    }
    
    pub fn reset_stats(&mut self) {
        self.culling_stats = CullingStats::default();
    }
    
    pub fn get_frustum(&self) -> &Frustum {
        &self.current_frustum
    }
    
    pub fn debug_draw_frustum(&self) -> Vec<(Point3<f32>, Point3<f32>)> {
        let mut lines = Vec::new();
        
        // This is a simplified debug representation
        // In a real implementation, you'd compute the actual frustum vertices
        // and create lines connecting them
        
        // For now, we'll create a basic wireframe based on the planes
        let near_corners = self.compute_plane_intersection_points();
        
        if near_corners.len() >= 8 {
            // Connect the corners to form a frustum wireframe
            // Near face
            lines.push((near_corners[0], near_corners[1]));
            lines.push((near_corners[1], near_corners[2]));
            lines.push((near_corners[2], near_corners[3]));
            lines.push((near_corners[3], near_corners[0]));
            
            // Far face
            lines.push((near_corners[4], near_corners[5]));
            lines.push((near_corners[5], near_corners[6]));
            lines.push((near_corners[6], near_corners[7]));
            lines.push((near_corners[7], near_corners[4]));
            
            // Connecting lines
            lines.push((near_corners[0], near_corners[4]));
            lines.push((near_corners[1], near_corners[5]));
            lines.push((near_corners[2], near_corners[6]));
            lines.push((near_corners[3], near_corners[7]));
        }
        
        lines
    }
    
    fn compute_plane_intersection_points(&self) -> Vec<Point3<f32>> {
        // This is a placeholder implementation
        // Computing actual frustum corners requires solving plane intersections
        // which is mathematically complex. For now, we return empty points.
        
        vec![
            Point3::new(0.0, 0.0, 0.0); 8
        ]
    }
}

#[derive(Debug, Default)]
pub struct CullingStats {
    pub total_tests: u64,
    pub objects_passed: u64,
    pub objects_culled: u64,
    pub frustum_culled: u64,
    pub distance_culled: u64,
    pub occlusion_culled: u64,
}

impl CullingStats {
    pub fn get_cull_ratio(&self) -> f32 {
        if self.total_tests == 0 {
            0.0
        } else {
            self.objects_culled as f32 / self.total_tests as f32
        }
    }
    
    pub fn get_pass_ratio(&self) -> f32 {
        if self.total_tests == 0 {
            0.0
        } else {
            self.objects_passed as f32 / self.total_tests as f32
        }
    }
}