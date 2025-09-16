/*!
 * Robin Engine Bounding Volume System
 * 
 * Bounding boxes, spheres, and other geometric primitives
 * for collision detection and spatial operations.
 */

use crate::engine::math::{Vec3, Mat4};
use cgmath::{InnerSpace, Vector3, Transform, Point3, EuclideanSpace};

/// Axis-aligned bounding box
#[derive(Debug, Clone, PartialEq)]
pub struct BoundingBox {
    pub min: Vec3,
    pub max: Vec3,
}

impl BoundingBox {
    /// Create a new bounding box
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// Create an empty bounding box
    pub fn empty() -> Self {
        Self {
            min: Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            max: Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
        }
    }

    /// Create a bounding box that encompasses everything
    pub fn infinite() -> Self {
        Self {
            min: Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
            max: Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
        }
    }

    /// Get the center point of the bounding box
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Get the size (extents) of the bounding box
    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    /// Get the diagonal length of the bounding box
    pub fn diagonal(&self) -> f32 {
        self.size().magnitude()
    }

    /// Check if the bounding box is empty
    pub fn is_empty(&self) -> bool {
        self.min.x > self.max.x || self.min.y > self.max.y || self.min.z > self.max.z
    }

    /// Check if the bounding box is valid
    pub fn is_valid(&self) -> bool {
        !self.is_empty() && 
        self.min.x.is_finite() && self.min.y.is_finite() && self.min.z.is_finite() &&
        self.max.x.is_finite() && self.max.y.is_finite() && self.max.z.is_finite()
    }

    /// Expand the bounding box to include a point
    pub fn expand_to_include_point(&mut self, point: &Vec3) {
        self.min.x = self.min.x.min(point.x);
        self.min.y = self.min.y.min(point.y);
        self.min.z = self.min.z.min(point.z);
        self.max.x = self.max.x.max(point.x);
        self.max.y = self.max.y.max(point.y);
        self.max.z = self.max.z.max(point.z);
    }

    /// Expand the bounding box to include another bounding box
    pub fn expand_to_include_box(&mut self, other: &BoundingBox) {
        if other.is_empty() {
            return;
        }
        
        if self.is_empty() {
            *self = other.clone();
            return;
        }

        self.min.x = self.min.x.min(other.min.x);
        self.min.y = self.min.y.min(other.min.y);
        self.min.z = self.min.z.min(other.min.z);
        self.max.x = self.max.x.max(other.max.x);
        self.max.y = self.max.y.max(other.max.y);
        self.max.z = self.max.z.max(other.max.z);
    }

    /// Check if this bounding box intersects with another
    pub fn intersects(&self, other: &BoundingBox) -> bool {
        !self.is_empty() && !other.is_empty() &&
        self.min.x <= other.max.x && self.max.x >= other.min.x &&
        self.min.y <= other.max.y && self.max.y >= other.min.y &&
        self.min.z <= other.max.z && self.max.z >= other.min.z
    }

    /// Check if this bounding box contains a point
    pub fn contains_point(&self, point: &Vec3) -> bool {
        !self.is_empty() &&
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y &&
        point.z >= self.min.z && point.z <= self.max.z
    }

    /// Check if this bounding box fully contains another bounding box
    pub fn contains_box(&self, other: &BoundingBox) -> bool {
        !self.is_empty() && !other.is_empty() &&
        self.min.x <= other.min.x && self.max.x >= other.max.x &&
        self.min.y <= other.min.y && self.max.y >= other.max.y &&
        self.min.z <= other.min.z && self.max.z >= other.max.z
    }

    /// Transform the bounding box by a matrix
    pub fn transform(&self, matrix: &Mat4) -> BoundingBox {
        if self.is_empty() {
            return BoundingBox::empty();
        }

        let corners = [
            Vec3::new(self.min.x, self.min.y, self.min.z),
            Vec3::new(self.max.x, self.min.y, self.min.z),
            Vec3::new(self.min.x, self.max.y, self.min.z),
            Vec3::new(self.max.x, self.max.y, self.min.z),
            Vec3::new(self.min.x, self.min.y, self.max.z),
            Vec3::new(self.max.x, self.min.y, self.max.z),
            Vec3::new(self.min.x, self.max.y, self.max.z),
            Vec3::new(self.max.x, self.max.y, self.max.z),
        ];

        let mut result = BoundingBox::empty();
        for corner in &corners {
            let point = Point3::from_vec(*corner);
            let transformed = matrix.transform_point(point);
            let transformed_vec = transformed.to_vec();
            result.expand_to_include_point(&transformed_vec);
        }

        result
    }

    /// Get the vertices of the bounding box
    pub fn get_vertices(&self) -> [Vec3; 8] {
        [
            Vec3::new(self.min.x, self.min.y, self.min.z),
            Vec3::new(self.max.x, self.min.y, self.min.z),
            Vec3::new(self.min.x, self.max.y, self.min.z),
            Vec3::new(self.max.x, self.max.y, self.min.z),
            Vec3::new(self.min.x, self.min.y, self.max.z),
            Vec3::new(self.max.x, self.min.y, self.max.z),
            Vec3::new(self.min.x, self.max.y, self.max.z),
            Vec3::new(self.max.x, self.max.y, self.max.z),
        ]
    }

    /// Create a bounding box from a set of points
    pub fn from_points(points: &[Vec3]) -> BoundingBox {
        if points.is_empty() {
            return BoundingBox::empty();
        }

        let mut result = BoundingBox {
            min: points[0],
            max: points[0],
        };

        for point in points.iter().skip(1) {
            result.expand_to_include_point(point);
        }

        result
    }
}

/// Bounding sphere
#[derive(Debug, Clone, PartialEq)]
pub struct BoundingSphere {
    pub center: Vec3,
    pub radius: f32,
}

impl BoundingSphere {
    /// Create a new bounding sphere
    pub fn new(center: Vec3, radius: f32) -> Self {
        Self { center, radius }
    }

    /// Create an empty bounding sphere
    pub fn empty() -> Self {
        Self {
            center: Vec3::new(0.0, 0.0, 0.0),
            radius: 0.0,
        }
    }

    /// Check if the sphere is valid
    pub fn is_valid(&self) -> bool {
        self.radius >= 0.0 && 
        self.center.x.is_finite() && 
        self.center.y.is_finite() && 
        self.center.z.is_finite()
    }

    /// Check if the sphere is empty
    pub fn is_empty(&self) -> bool {
        self.radius <= 0.0
    }

    /// Get the surface area of the sphere
    pub fn surface_area(&self) -> f32 {
        4.0 * std::f32::consts::PI * self.radius * self.radius
    }

    /// Get the volume of the sphere
    pub fn volume(&self) -> f32 {
        (4.0 / 3.0) * std::f32::consts::PI * self.radius * self.radius * self.radius
    }

    /// Check if this sphere intersects with another sphere
    pub fn intersects(&self, other: &BoundingSphere) -> bool {
        if self.is_empty() || other.is_empty() {
            return false;
        }

        let distance_squared = (self.center - other.center).magnitude2();
        let combined_radius = self.radius + other.radius;
        distance_squared <= combined_radius * combined_radius
    }

    /// Check if this sphere contains a point
    pub fn contains_point(&self, point: &Vec3) -> bool {
        if self.is_empty() {
            return false;
        }

        let distance_squared = (*point - self.center).magnitude2();
        distance_squared <= self.radius * self.radius
    }

    /// Check if this sphere fully contains another sphere
    pub fn contains_sphere(&self, other: &BoundingSphere) -> bool {
        if self.is_empty() || other.is_empty() {
            return false;
        }

        let distance = (self.center - other.center).magnitude();
        distance + other.radius <= self.radius
    }

    /// Expand the sphere to include a point
    pub fn expand_to_include_point(&mut self, point: &Vec3) {
        if self.is_empty() {
            self.center = *point;
            self.radius = 0.0;
            return;
        }

        let distance = (*point - self.center).magnitude();
        if distance > self.radius {
            let new_radius = (self.radius + distance) * 0.5;
            let expansion = new_radius - self.radius;
            self.center = self.center + ((*point - self.center).normalize() * expansion);
            self.radius = new_radius;
        }
    }

    /// Expand the sphere to include another sphere
    pub fn expand_to_include_sphere(&mut self, other: &BoundingSphere) {
        if other.is_empty() {
            return;
        }

        if self.is_empty() {
            *self = other.clone();
            return;
        }

        let distance = (other.center - self.center).magnitude();
        let required_radius = distance + other.radius;
        
        if required_radius > self.radius {
            let new_radius = (self.radius + required_radius) * 0.5;
            let expansion = new_radius - self.radius;
            if distance > 0.0 {
                self.center = self.center + ((other.center - self.center).normalize() * expansion);
            }
            self.radius = new_radius;
        }
    }

    /// Transform the bounding sphere by a matrix
    pub fn transform(&self, matrix: &Mat4) -> BoundingSphere {
        if self.is_empty() {
            return BoundingSphere::empty();
        }

        let transformed_center_point = matrix.transform_point(cgmath::Point3::new(self.center.x, self.center.y, self.center.z));
        let transformed_center = Vec3::new(transformed_center_point.x, transformed_center_point.y, transformed_center_point.z);
        
        // Calculate the scale factor by transforming a unit vector
        let unit_vec = Vector3::new(1.0, 0.0, 0.0);
        let transformed_vec4 = matrix * unit_vec.extend(0.0);
        let scale_vector = Vector3::new(transformed_vec4.x, transformed_vec4.y, transformed_vec4.z);
        let scale_factor = scale_vector.magnitude();
        
        BoundingSphere::new(transformed_center, self.radius * scale_factor)
    }

    /// Create a bounding sphere from a set of points
    pub fn from_points(points: &[Vec3]) -> BoundingSphere {
        if points.is_empty() {
            return BoundingSphere::empty();
        }

        // Calculate center as average of points
        let mut center = Vec3::new(0.0, 0.0, 0.0);
        for point in points {
            center = center + *point;
        }
        center = center / points.len() as f32;

        // Find maximum distance from center
        let mut max_distance_squared: f32 = 0.0;
        for point in points {
            let distance_squared = (*point - center).magnitude2();
            max_distance_squared = max_distance_squared.max(distance_squared);
        }

        BoundingSphere::new(center, max_distance_squared.sqrt())
    }

    /// Create a bounding sphere from a bounding box
    pub fn from_box(bbox: &BoundingBox) -> BoundingSphere {
        if bbox.is_empty() {
            return BoundingSphere::empty();
        }

        let center = bbox.center();
        let radius = (bbox.max - center).magnitude();
        BoundingSphere::new(center, radius)
    }
}

/// Convert a bounding box to a bounding sphere
impl From<&BoundingBox> for BoundingSphere {
    fn from(bbox: &BoundingBox) -> Self {
        BoundingSphere::from_box(bbox)
    }
}

/// Convert a bounding sphere to a bounding box
impl From<&BoundingSphere> for BoundingBox {
    fn from(sphere: &BoundingSphere) -> Self {
        if sphere.is_empty() {
            return BoundingBox::empty();
        }

        let offset = Vec3::new(sphere.radius, sphere.radius, sphere.radius);
        BoundingBox::new(sphere.center - offset, sphere.center + offset)
    }
}