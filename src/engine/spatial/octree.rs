use crate::engine::error::RobinResult;
use cgmath::{Point3, Vector3};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AABB {
    pub min: Point3<f32>,
    pub max: Point3<f32>,
}

impl AABB {
    pub fn new(min: Point3<f32>, max: Point3<f32>) -> Self {
        Self { min, max }
    }
    
    pub fn from_points(points: &[Point3<f32>]) -> Self {
        if points.is_empty() {
            return Self::new(Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 0.0, 0.0));
        }
        
        let mut min = points[0];
        let mut max = points[0];
        
        for &point in points.iter().skip(1) {
            min.x = min.x.min(point.x);
            min.y = min.y.min(point.y);
            min.z = min.z.min(point.z);
            max.x = max.x.max(point.x);
            max.y = max.y.max(point.y);
            max.z = max.z.max(point.z);
        }
        
        Self { min, max }
    }
    
    pub fn center(&self) -> Point3<f32> {
        Point3::new(
            (self.min.x + self.max.x) / 2.0,
            (self.min.y + self.max.y) / 2.0,
            (self.min.z + self.max.z) / 2.0,
        )
    }
    
    pub fn size(&self) -> Vector3<f32> {
        self.max - self.min
    }
    
    pub fn volume(&self) -> f32 {
        let size = self.size();
        size.x * size.y * size.z
    }
    
    pub fn intersects(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x &&
        self.min.y <= other.max.y && self.max.y >= other.min.y &&
        self.min.z <= other.max.z && self.max.z >= other.min.z
    }
    
    pub fn contains_point(&self, point: Point3<f32>) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y &&
        point.z >= self.min.z && point.z <= self.max.z
    }
    
    pub fn contains_aabb(&self, other: &AABB) -> bool {
        self.min.x <= other.min.x && self.max.x >= other.max.x &&
        self.min.y <= other.min.y && self.max.y >= other.max.y &&
        self.min.z <= other.min.z && self.max.z >= other.max.z
    }
    
    pub fn expand(&mut self, point: Point3<f32>) {
        self.min.x = self.min.x.min(point.x);
        self.min.y = self.min.y.min(point.y);
        self.min.z = self.min.z.min(point.z);
        self.max.x = self.max.x.max(point.x);
        self.max.y = self.max.y.max(point.y);
        self.max.z = self.max.z.max(point.z);
    }
    
    pub fn expand_aabb(&mut self, other: &AABB) {
        self.expand(other.min);
        self.expand(other.max);
    }
    
    pub fn distance_to_point(&self, point: Point3<f32>) -> f32 {
        let dx = (point.x - self.min.x).min(0.0).abs().max((point.x - self.max.x).max(0.0));
        let dy = (point.y - self.min.y).min(0.0).abs().max((point.y - self.max.y).max(0.0));
        let dz = (point.z - self.min.z).min(0.0).abs().max((point.z - self.max.z).max(0.0));
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

#[derive(Debug)]
pub struct OctreeNode {
    pub bounds: AABB,
    pub objects: Vec<u32>,
    pub children: Option<Box<[OctreeNode; 8]>>,
    pub depth: usize,
}

impl OctreeNode {
    pub fn new(bounds: AABB, depth: usize) -> Self {
        Self {
            bounds,
            objects: Vec::new(),
            children: None,
            depth,
        }
    }
    
    pub fn is_leaf(&self) -> bool {
        self.children.is_none()
    }
    
    pub fn subdivide(&mut self, max_objects: usize, max_depth: usize) {
        if self.children.is_some() || self.objects.len() <= max_objects || self.depth >= max_depth {
            return;
        }
        
        let center = self.bounds.center();
        let size = self.bounds.size();
        let half_size = size / 2.0;
        
        let child_bounds = [
            AABB::new(self.bounds.min, center),
            AABB::new(
                Point3::new(center.x, self.bounds.min.y, self.bounds.min.z),
                Point3::new(self.bounds.max.x, center.y, center.z)
            ),
            AABB::new(
                Point3::new(self.bounds.min.x, center.y, self.bounds.min.z),
                Point3::new(center.x, self.bounds.max.y, center.z)
            ),
            AABB::new(
                Point3::new(center.x, center.y, self.bounds.min.z),
                Point3::new(self.bounds.max.x, self.bounds.max.y, center.z)
            ),
            AABB::new(
                Point3::new(self.bounds.min.x, self.bounds.min.y, center.z),
                Point3::new(center.x, center.y, self.bounds.max.z)
            ),
            AABB::new(
                Point3::new(center.x, self.bounds.min.y, center.z),
                Point3::new(self.bounds.max.x, center.y, self.bounds.max.z)
            ),
            AABB::new(
                Point3::new(self.bounds.min.x, center.y, center.z),
                Point3::new(center.x, self.bounds.max.y, self.bounds.max.z)
            ),
            AABB::new(center, self.bounds.max),
        ];
        
        self.children = Some(Box::new([
            OctreeNode::new(child_bounds[0].clone(), self.depth + 1),
            OctreeNode::new(child_bounds[1].clone(), self.depth + 1),
            OctreeNode::new(child_bounds[2].clone(), self.depth + 1),
            OctreeNode::new(child_bounds[3].clone(), self.depth + 1),
            OctreeNode::new(child_bounds[4].clone(), self.depth + 1),
            OctreeNode::new(child_bounds[5].clone(), self.depth + 1),
            OctreeNode::new(child_bounds[6].clone(), self.depth + 1),
            OctreeNode::new(child_bounds[7].clone(), self.depth + 1),
        ]));
    }
    
    pub fn get_child_index(&self, bounds: &AABB) -> Option<usize> {
        if self.children.is_none() {
            return None;
        }
        
        let center = self.bounds.center();
        let obj_center = bounds.center();
        
        let mut index = 0;
        if obj_center.x > center.x { index |= 1; }
        if obj_center.y > center.y { index |= 2; }
        if obj_center.z > center.z { index |= 4; }
        
        Some(index)
    }
}

pub struct Octree {
    root: OctreeNode,
    max_objects_per_node: usize,
    max_depth: usize,
    object_bounds: HashMap<u32, AABB>,
}

impl Octree {
    pub fn new(bounds: AABB, max_objects_per_node: usize, max_depth: usize) -> Self {
        Self {
            root: OctreeNode::new(bounds, 0),
            max_objects_per_node,
            max_depth,
            object_bounds: HashMap::new(),
        }
    }
    
    pub fn insert(&mut self, object_id: u32, bounds: AABB) -> RobinResult<()> {
        self.object_bounds.insert(object_id, bounds.clone());
        Self::insert_recursive_static(&mut self.root, object_id, &bounds, &self.object_bounds, self.max_objects_per_node, self.max_depth);
        Ok(())
    }
    
    fn insert_recursive(&mut self, node: &mut OctreeNode, object_id: u32, bounds: &AABB) {
        if !node.bounds.intersects(bounds) {
            return;
        }
        
        if node.is_leaf() {
            node.objects.push(object_id);
            if node.objects.len() > self.max_objects_per_node && node.depth < self.max_depth {
                node.subdivide(self.max_objects_per_node, self.max_depth);
                
                if let Some(ref mut children) = node.children {
                    let objects_to_redistribute = node.objects.clone();
                    node.objects.clear();
                    
                    for obj_id in objects_to_redistribute {
                        if let Some(obj_bounds) = self.object_bounds.get(&obj_id).cloned() {
                            let mut placed = false;
                            for child in children.iter_mut() {
                                if child.bounds.contains_aabb(&obj_bounds) {
                                    self.insert_recursive(child, obj_id, &obj_bounds);
                                    placed = true;
                                    break;
                                }
                            }
                            if !placed {
                                node.objects.push(obj_id);
                            }
                        }
                    }
                }
            }
        } else {
            if let Some(ref mut children) = node.children {
                let mut placed = false;
                for child in children.iter_mut() {
                    if child.bounds.contains_aabb(bounds) {
                        self.insert_recursive(child, object_id, bounds);
                        placed = true;
                        break;
                    }
                }
                if !placed {
                    node.objects.push(object_id);
                }
            }
        }
    }

    fn insert_recursive_static(
        node: &mut OctreeNode,
        object_id: u32,
        bounds: &AABB,
        object_bounds: &HashMap<u32, AABB>,
        max_objects_per_node: usize,
        max_depth: usize
    ) {
        if !node.bounds.intersects(bounds) {
            return;
        }

        if node.is_leaf() {
            node.objects.push(object_id);
            if node.objects.len() > max_objects_per_node && node.depth < max_depth {
                node.subdivide(max_objects_per_node, max_depth);

                if let Some(ref mut children) = node.children {
                    let objects_to_redistribute = node.objects.clone();
                    node.objects.clear();

                    for obj_id in objects_to_redistribute {
                        if let Some(obj_bounds) = object_bounds.get(&obj_id).cloned() {
                            let mut placed = false;
                            for child in children.iter_mut() {
                                if child.bounds.contains_aabb(&obj_bounds) {
                                    Self::insert_recursive_static(child, obj_id, &obj_bounds, object_bounds, max_objects_per_node, max_depth);
                                    placed = true;
                                    break;
                                }
                            }
                            if !placed {
                                node.objects.push(obj_id);
                            }
                        }
                    }
                }
            }
        } else {
            if let Some(ref mut children) = node.children {
                let mut placed = false;
                for child in children.iter_mut() {
                    if child.bounds.contains_aabb(bounds) {
                        Self::insert_recursive_static(child, object_id, bounds, object_bounds, max_objects_per_node, max_depth);
                        placed = true;
                        break;
                    }
                }
                if !placed {
                    node.objects.push(object_id);
                }
            }
        }
    }

    pub fn remove(&mut self, object_id: u32) -> RobinResult<()> {
        if let Some(bounds) = self.object_bounds.remove(&object_id) {
            Self::remove_recursive_static(&mut self.root, object_id, &bounds);
        }
        Ok(())
    }
    
    fn remove_recursive(&mut self, node: &mut OctreeNode, object_id: u32, bounds: &AABB) {
        if !node.bounds.intersects(bounds) {
            return;
        }
        
        node.objects.retain(|&id| id != object_id);
        
        if let Some(ref mut children) = node.children {
            for child in children.iter_mut() {
                self.remove_recursive(child, object_id, bounds);
            }
        }
    }

    fn remove_recursive_static(node: &mut OctreeNode, object_id: u32, bounds: &AABB) {
        if !node.bounds.intersects(bounds) {
            return;
        }

        node.objects.retain(|&id| id != object_id);

        if let Some(ref mut children) = node.children {
            for child in children.iter_mut() {
                Self::remove_recursive_static(child, object_id, bounds);
            }
        }
    }

    pub fn query(&self, bounds: &AABB) -> Vec<u32> {
        let mut results = Vec::new();
        self.query_recursive(&self.root, bounds, &mut results);
        results
    }
    
    fn query_recursive(&self, node: &OctreeNode, bounds: &AABB, results: &mut Vec<u32>) {
        if !node.bounds.intersects(bounds) {
            return;
        }
        
        results.extend_from_slice(&node.objects);
        
        if let Some(ref children) = node.children {
            for child in children.iter() {
                self.query_recursive(child, bounds, results);
            }
        }
    }
    
    pub fn query_point(&self, point: Point3<f32>) -> Vec<u32> {
        let mut results = Vec::new();
        self.query_point_recursive(&self.root, point, &mut results);
        results
    }
    
    fn query_point_recursive(&self, node: &OctreeNode, point: Point3<f32>, results: &mut Vec<u32>) {
        if !node.bounds.contains_point(point) {
            return;
        }
        
        for &object_id in &node.objects {
            if let Some(bounds) = self.object_bounds.get(&object_id) {
                if bounds.contains_point(point) {
                    results.push(object_id);
                }
            }
        }
        
        if let Some(ref children) = node.children {
            for child in children.iter() {
                self.query_point_recursive(child, point, results);
            }
        }
    }
    
    pub fn get_max_depth(&self) -> usize {
        self.get_depth_recursive(&self.root)
    }
    
    fn get_depth_recursive(&self, node: &OctreeNode) -> usize {
        let mut max_depth = node.depth;
        
        if let Some(ref children) = node.children {
            for child in children.iter() {
                max_depth = max_depth.max(self.get_depth_recursive(child));
            }
        }
        
        max_depth
    }
    
    pub fn get_node_count(&self) -> usize {
        self.get_node_count_recursive(&self.root)
    }
    
    fn get_node_count_recursive(&self, node: &OctreeNode) -> usize {
        let mut count = 1;
        
        if let Some(ref children) = node.children {
            for child in children.iter() {
                count += self.get_node_count_recursive(child);
            }
        }
        
        count
    }
    
    pub fn clear(&mut self) {
        self.object_bounds.clear();
        self.root = OctreeNode::new(self.root.bounds.clone(), 0);
    }
    
    pub fn get_statistics(&self) -> OctreeStats {
        let mut stats = OctreeStats::default();
        self.collect_statistics(&self.root, &mut stats);
        stats
    }
    
    fn collect_statistics(&self, node: &OctreeNode, stats: &mut OctreeStats) {
        stats.total_nodes += 1;
        stats.total_objects += node.objects.len();
        
        if node.is_leaf() {
            stats.leaf_nodes += 1;
            stats.max_objects_per_leaf = stats.max_objects_per_leaf.max(node.objects.len());
            stats.min_objects_per_leaf = if stats.leaf_nodes == 1 {
                node.objects.len()
            } else {
                stats.min_objects_per_leaf.min(node.objects.len())
            };
        } else {
            stats.internal_nodes += 1;
            if let Some(ref children) = node.children {
                for child in children.iter() {
                    self.collect_statistics(child, stats);
                }
            }
        }
        
        stats.max_depth = stats.max_depth.max(node.depth);
    }
}

#[derive(Debug, Default)]
pub struct OctreeStats {
    pub total_nodes: usize,
    pub leaf_nodes: usize,
    pub internal_nodes: usize,
    pub total_objects: usize,
    pub max_objects_per_leaf: usize,
    pub min_objects_per_leaf: usize,
    pub max_depth: usize,
}