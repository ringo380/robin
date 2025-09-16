pub use crate::engine::spatial::octree::AABB;
use cgmath::{Point3, Vector3};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct BVHNode {
    pub bounds: AABB,
    pub object_ids: Vec<u32>,
    pub left_child: Option<Box<BVHNode>>,
    pub right_child: Option<Box<BVHNode>>,
    pub is_leaf: bool,
}

impl BVHNode {
    pub fn new_leaf(bounds: AABB, object_ids: Vec<u32>) -> Self {
        Self {
            bounds,
            object_ids,
            left_child: None,
            right_child: None,
            is_leaf: true,
        }
    }
    
    pub fn new_internal(bounds: AABB, left: BVHNode, right: BVHNode) -> Self {
        Self {
            bounds,
            object_ids: Vec::new(),
            left_child: Some(Box::new(left)),
            right_child: Some(Box::new(right)),
            is_leaf: false,
        }
    }
}

pub struct BVH {
    root: Option<BVHNode>,
    objects: HashMap<u32, AABB>,
    max_objects_per_leaf: usize,
    max_depth: usize,
    built: bool,
}

impl BVH {
    pub fn new() -> Self {
        Self {
            root: None,
            objects: HashMap::new(),
            max_objects_per_leaf: 4,
            max_depth: 20,
            built: false,
        }
    }
    
    pub fn insert(&mut self, object_id: u32, bounds: AABB) {
        self.objects.insert(object_id, bounds);
        self.built = false;
    }
    
    pub fn remove(&mut self, object_id: u32) {
        self.objects.remove(&object_id);
        self.built = false;
    }
    
    pub fn build(&mut self) {
        if self.objects.is_empty() {
            self.root = None;
            self.built = true;
            return;
        }
        
        let object_ids: Vec<u32> = self.objects.keys().cloned().collect();
        let mut world_bounds = self.objects.values().next().unwrap().clone();
        
        for bounds in self.objects.values() {
            world_bounds.expand_aabb(bounds);
        }
        
        self.root = Some(self.build_recursive(object_ids, world_bounds, 0));
        self.built = true;
    }
    
    fn build_recursive(&self, object_ids: Vec<u32>, bounds: AABB, depth: usize) -> BVHNode {
        if object_ids.len() <= self.max_objects_per_leaf || depth >= self.max_depth {
            return BVHNode::new_leaf(bounds, object_ids);
        }
        
        let axis = self.get_longest_axis(&bounds);
        let mut sorted_objects = object_ids;
        
        sorted_objects.sort_by(|&a, &b| {
            let bounds_a = &self.objects[&a];
            let bounds_b = &self.objects[&b];
            let center_a = bounds_a.center();
            let center_b = bounds_b.center();
            
            let value_a = match axis {
                0 => center_a.x,
                1 => center_a.y,
                _ => center_a.z,
            };
            let value_b = match axis {
                0 => center_b.x,
                1 => center_b.y,
                _ => center_b.z,
            };
            
            value_a.partial_cmp(&value_b).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        let split_index = sorted_objects.len() / 2;
        let left_objects = sorted_objects[..split_index].to_vec();
        let right_objects = sorted_objects[split_index..].to_vec();
        
        let left_bounds = self.compute_bounds(&left_objects);
        let right_bounds = self.compute_bounds(&right_objects);
        
        let left_node = self.build_recursive(left_objects, left_bounds, depth + 1);
        let right_node = self.build_recursive(right_objects, right_bounds, depth + 1);
        
        BVHNode::new_internal(bounds, left_node, right_node)
    }
    
    fn get_longest_axis(&self, bounds: &AABB) -> usize {
        let size = bounds.size();
        if size.x >= size.y && size.x >= size.z {
            0 // X axis
        } else if size.y >= size.z {
            1 // Y axis
        } else {
            2 // Z axis
        }
    }
    
    fn compute_bounds(&self, object_ids: &[u32]) -> AABB {
        if object_ids.is_empty() {
            return AABB::new(Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 0.0, 0.0));
        }
        
        let mut bounds = self.objects[&object_ids[0]].clone();
        for &object_id in object_ids.iter().skip(1) {
            bounds.expand_aabb(&self.objects[&object_id]);
        }
        bounds
    }
    
    pub fn query(&self, bounds: &AABB) -> Vec<u32> {
        if !self.built {
            return Vec::new();
        }
        
        let mut results = Vec::new();
        if let Some(ref root) = self.root {
            self.query_recursive(root, bounds, &mut results);
        }
        results
    }
    
    fn query_recursive(&self, node: &BVHNode, bounds: &AABB, results: &mut Vec<u32>) {
        if !node.bounds.intersects(bounds) {
            return;
        }
        
        if node.is_leaf {
            for &object_id in &node.object_ids {
                if let Some(object_bounds) = self.objects.get(&object_id) {
                    if object_bounds.intersects(bounds) {
                        results.push(object_id);
                    }
                }
            }
        } else {
            if let Some(ref left) = node.left_child {
                self.query_recursive(left, bounds, results);
            }
            if let Some(ref right) = node.right_child {
                self.query_recursive(right, bounds, results);
            }
        }
    }
    
    pub fn query_ray(&self, origin: Point3<f32>, direction: Vector3<f32>, max_distance: f32) -> Vec<u32> {
        if !self.built {
            return Vec::new();
        }
        
        let mut results = Vec::new();
        if let Some(ref root) = self.root {
            self.query_ray_recursive(root, origin, direction, max_distance, &mut results);
        }
        results
    }
    
    fn query_ray_recursive(&self, node: &BVHNode, origin: Point3<f32>, direction: Vector3<f32>, max_distance: f32, results: &mut Vec<u32>) {
        if !self.intersect_ray_aabb(origin, direction, &node.bounds, max_distance) {
            return;
        }
        
        if node.is_leaf {
            for &object_id in &node.object_ids {
                if let Some(object_bounds) = self.objects.get(&object_id) {
                    if self.intersect_ray_aabb(origin, direction, object_bounds, max_distance) {
                        results.push(object_id);
                    }
                }
            }
        } else {
            if let Some(ref left) = node.left_child {
                self.query_ray_recursive(left, origin, direction, max_distance, results);
            }
            if let Some(ref right) = node.right_child {
                self.query_ray_recursive(right, origin, direction, max_distance, results);
            }
        }
    }
    
    fn intersect_ray_aabb(&self, origin: Point3<f32>, direction: Vector3<f32>, aabb: &AABB, max_distance: f32) -> bool {
        let inv_dir = Vector3::new(
            if direction.x != 0.0 { 1.0 / direction.x } else { f32::INFINITY },
            if direction.y != 0.0 { 1.0 / direction.y } else { f32::INFINITY },
            if direction.z != 0.0 { 1.0 / direction.z } else { f32::INFINITY },
        );
        
        let t1 = (aabb.min.x - origin.x) * inv_dir.x;
        let t2 = (aabb.max.x - origin.x) * inv_dir.x;
        let t3 = (aabb.min.y - origin.y) * inv_dir.y;
        let t4 = (aabb.max.y - origin.y) * inv_dir.y;
        let t5 = (aabb.min.z - origin.z) * inv_dir.z;
        let t6 = (aabb.max.z - origin.z) * inv_dir.z;
        
        let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
        let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));
        
        if tmax < 0.0 || tmin > tmax {
            return false;
        }
        
        let t = if tmin < 0.0 { tmax } else { tmin };
        t >= 0.0 && t <= max_distance
    }
    
    pub fn update_object(&mut self, object_id: u32, new_bounds: AABB) {
        if self.objects.contains_key(&object_id) {
            self.objects.insert(object_id, new_bounds);
            self.built = false;
        }
    }
    
    pub fn refit(&mut self) {
        if !self.built || self.root.is_none() {
            return;
        }
        
        if let Some(ref mut root) = self.root {
            Self::refit_recursive_static(root, &self.objects);
        }
    }
    
    fn refit_recursive(&mut self, node: &mut BVHNode) {
        if node.is_leaf {
            if !node.object_ids.is_empty() {
                node.bounds = self.objects[&node.object_ids[0]].clone();
                for &object_id in node.object_ids.iter().skip(1) {
                    node.bounds.expand_aabb(&self.objects[&object_id]);
                }
            }
        } else {
            if let Some(ref mut left) = node.left_child {
                self.refit_recursive(left);
            }
            if let Some(ref mut right) = node.right_child {
                self.refit_recursive(right);
            }
            
            let mut new_bounds = AABB::new(Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 0.0, 0.0));
            let mut first = true;
            
            if let Some(ref left) = node.left_child {
                if first {
                    new_bounds = left.bounds.clone();
                    first = false;
                } else {
                    new_bounds.expand_aabb(&left.bounds);
                }
            }
            
            if let Some(ref right) = node.right_child {
                if first {
                    new_bounds = right.bounds.clone();
                } else {
                    new_bounds.expand_aabb(&right.bounds);
                }
            }
            
            node.bounds = new_bounds;
        }
    }

    fn refit_recursive_static(node: &mut BVHNode, objects: &HashMap<u32, AABB>) {
        if node.is_leaf {
            if !node.object_ids.is_empty() {
                if let Some(first_bounds) = objects.get(&node.object_ids[0]) {
                    node.bounds = first_bounds.clone();
                    for &object_id in node.object_ids.iter().skip(1) {
                        if let Some(obj_bounds) = objects.get(&object_id) {
                            node.bounds.expand_aabb(obj_bounds);
                        }
                    }
                }
            }
        } else {
            if let Some(ref mut left) = node.left_child {
                Self::refit_recursive_static(left, objects);
            }
            if let Some(ref mut right) = node.right_child {
                Self::refit_recursive_static(right, objects);
            }

            let mut new_bounds = AABB::new(Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 0.0, 0.0));
            let mut first = true;

            if let Some(ref left) = node.left_child {
                if first {
                    new_bounds = left.bounds.clone();
                    first = false;
                } else {
                    new_bounds.expand_aabb(&left.bounds);
                }
            }

            if let Some(ref right) = node.right_child {
                if first {
                    new_bounds = right.bounds.clone();
                } else {
                    new_bounds.expand_aabb(&right.bounds);
                }
            }

            node.bounds = new_bounds;
        }
    }

    pub fn get_node_count(&self) -> usize {
        if let Some(ref root) = self.root {
            self.count_nodes_recursive(root)
        } else {
            0
        }
    }
    
    fn count_nodes_recursive(&self, node: &BVHNode) -> usize {
        let mut count = 1;
        
        if let Some(ref left) = node.left_child {
            count += self.count_nodes_recursive(left);
        }
        if let Some(ref right) = node.right_child {
            count += self.count_nodes_recursive(right);
        }
        
        count
    }
    
    pub fn get_depth(&self) -> usize {
        if let Some(ref root) = self.root {
            self.get_depth_recursive(root, 0)
        } else {
            0
        }
    }
    
    fn get_depth_recursive(&self, node: &BVHNode, current_depth: usize) -> usize {
        if node.is_leaf {
            return current_depth;
        }
        
        let mut max_depth = current_depth;
        
        if let Some(ref left) = node.left_child {
            max_depth = max_depth.max(self.get_depth_recursive(left, current_depth + 1));
        }
        if let Some(ref right) = node.right_child {
            max_depth = max_depth.max(self.get_depth_recursive(right, current_depth + 1));
        }
        
        max_depth
    }
    
    pub fn get_statistics(&self) -> BVHStats {
        if let Some(ref root) = self.root {
            let mut stats = BVHStats::default();
            self.collect_stats_recursive(root, &mut stats, 0);
            stats.max_depth = self.get_depth();
            stats.total_objects = self.objects.len();
            stats
        } else {
            BVHStats::default()
        }
    }
    
    fn collect_stats_recursive(&self, node: &BVHNode, stats: &mut BVHStats, depth: usize) {
        stats.total_nodes += 1;
        
        if node.is_leaf {
            stats.leaf_nodes += 1;
            stats.total_leaf_objects += node.object_ids.len();
            stats.max_objects_per_leaf = stats.max_objects_per_leaf.max(node.object_ids.len());
            stats.min_objects_per_leaf = if stats.leaf_nodes == 1 {
                node.object_ids.len()
            } else {
                stats.min_objects_per_leaf.min(node.object_ids.len())
            };
        } else {
            stats.internal_nodes += 1;
            
            if let Some(ref left) = node.left_child {
                self.collect_stats_recursive(left, stats, depth + 1);
            }
            if let Some(ref right) = node.right_child {
                self.collect_stats_recursive(right, stats, depth + 1);
            }
        }
    }
    
    pub fn is_built(&self) -> bool {
        self.built
    }
    
    pub fn clear(&mut self) {
        self.objects.clear();
        self.root = None;
        self.built = false;
    }
    
    pub fn validate(&self) -> bool {
        if let Some(ref root) = self.root {
            self.validate_recursive(root)
        } else {
            self.objects.is_empty()
        }
    }
    
    fn validate_recursive(&self, node: &BVHNode) -> bool {
        if node.is_leaf {
            // Validate leaf node - all objects should fit within bounds
            for &object_id in &node.object_ids {
                if let Some(object_bounds) = self.objects.get(&object_id) {
                    if !node.bounds.contains_aabb(object_bounds) {
                        return false;
                    }
                } else {
                    return false; // Object not found in BVH
                }
            }
        } else {
            // Validate internal node - children bounds should fit within parent bounds
            if let Some(ref left) = node.left_child {
                if !node.bounds.contains_aabb(&left.bounds) || !self.validate_recursive(left) {
                    return false;
                }
            }
            if let Some(ref right) = node.right_child {
                if !node.bounds.contains_aabb(&right.bounds) || !self.validate_recursive(right) {
                    return false;
                }
            }
        }
        
        true
    }
}

#[derive(Debug, Default)]
pub struct BVHStats {
    pub total_nodes: usize,
    pub leaf_nodes: usize,
    pub internal_nodes: usize,
    pub total_objects: usize,
    pub total_leaf_objects: usize,
    pub max_objects_per_leaf: usize,
    pub min_objects_per_leaf: usize,
    pub max_depth: usize,
    pub average_objects_per_leaf: f32,
}

impl BVHStats {
    pub fn finalize(&mut self) {
        if self.leaf_nodes > 0 {
            self.average_objects_per_leaf = self.total_leaf_objects as f32 / self.leaf_nodes as f32;
        }
    }
}