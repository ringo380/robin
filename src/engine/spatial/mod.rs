use crate::engine::error::RobinResult;
use cgmath::{Vector3, Point3, InnerSpace, EuclideanSpace};
use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

pub mod octree;
pub mod spatial_hash;
pub mod frustum;
pub mod bvh;

pub use octree::{Octree, OctreeNode};
pub use spatial_hash::{SpatialHash, SpatialGrid};
pub use frustum::{Frustum, FrustumCuller};
pub use bvh::{BVH, BVHNode, AABB};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialConfig {
    pub world_size: f32,
    pub max_objects_per_node: usize,
    pub max_depth: usize,
    pub grid_cell_size: f32,
    pub enable_frustum_culling: bool,
    pub enable_occlusion_culling: bool,
    pub culling_distance: f32,
}

impl Default for SpatialConfig {
    fn default() -> Self {
        Self {
            world_size: 10000.0,
            max_objects_per_node: 10,
            max_depth: 8,
            grid_cell_size: 100.0,
            enable_frustum_culling: true,
            enable_occlusion_culling: false,
            culling_distance: 1000.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpatialObject {
    pub id: u32,
    pub position: Point3<f32>,
    pub bounds: AABB,
    pub layer_mask: u32,
    pub static_object: bool,
}

pub struct SpatialManager {
    config: SpatialConfig,
    octree: Octree,
    spatial_hash: SpatialHash,
    bvh: BVH,
    frustum_culler: FrustumCuller,
    objects: HashMap<u32, SpatialObject>,
    dirty_objects: HashSet<u32>,
    frame_counter: u64,
    last_rebuild_frame: u64,
}

impl SpatialManager {
    pub fn new(config: SpatialConfig) -> RobinResult<Self> {
        let bounds = AABB::new(
            Point3::new(-config.world_size / 2.0, -config.world_size / 2.0, -config.world_size / 2.0),
            Point3::new(config.world_size / 2.0, config.world_size / 2.0, config.world_size / 2.0),
        );

        Ok(Self {
            octree: Octree::new(bounds, config.max_objects_per_node, config.max_depth),
            spatial_hash: SpatialHash::new(config.grid_cell_size),
            bvh: BVH::new(),
            frustum_culler: FrustumCuller::new(),
            objects: HashMap::new(),
            dirty_objects: HashSet::new(),
            frame_counter: 0,
            last_rebuild_frame: 0,
            config,
        })
    }

    pub fn insert_object(&mut self, object: SpatialObject) -> RobinResult<()> {
        let id = object.id;
        
        if object.static_object {
            self.octree.insert(id, object.bounds.clone())?;
            self.bvh.insert(id, object.bounds.clone());
        } else {
            self.spatial_hash.insert(id, object.position, object.bounds.clone())?;
        }
        
        self.objects.insert(id, object);
        Ok(())
    }

    pub fn remove_object(&mut self, id: u32) -> RobinResult<()> {
        if let Some(object) = self.objects.remove(&id) {
            if object.static_object {
                self.octree.remove(id)?;
                self.bvh.remove(id);
            } else {
                self.spatial_hash.remove(id)?;
            }
            self.dirty_objects.remove(&id);
        }
        Ok(())
    }

    pub fn update_object(&mut self, id: u32, new_position: Point3<f32>, new_bounds: AABB) -> RobinResult<()> {
        if let Some(object) = self.objects.get_mut(&id) {
            let old_position = object.position;
            object.position = new_position;
            object.bounds = new_bounds.clone();

            if object.static_object {
                if old_position != new_position {
                    self.octree.remove(id)?;
                    self.octree.insert(id, new_bounds.clone())?;
                    self.bvh.remove(id);
                    self.bvh.insert(id, new_bounds);
                }
            } else {
                self.spatial_hash.update(id, old_position, new_position, new_bounds)?;
            }
            
            self.dirty_objects.insert(id);
        }
        Ok(())
    }

    pub fn query_region(&self, bounds: &AABB, layer_mask: u32) -> Vec<u32> {
        let mut results = Vec::new();
        
        let octree_results = self.octree.query(bounds);
        let hash_results = self.spatial_hash.query(bounds);
        
        for &id in &octree_results {
            if let Some(object) = self.objects.get(&id) {
                if (object.layer_mask & layer_mask) != 0 && object.bounds.intersects(bounds) {
                    results.push(id);
                }
            }
        }
        
        for &id in &hash_results {
            if let Some(object) = self.objects.get(&id) {
                if (object.layer_mask & layer_mask) != 0 && object.bounds.intersects(bounds) {
                    results.push(id);
                }
            }
        }
        
        results.sort_unstable();
        results.dedup();
        results
    }

    pub fn query_sphere(&self, center: Point3<f32>, radius: f32, layer_mask: u32) -> Vec<u32> {
        let sphere_bounds = AABB::new(
            Point3::new(center.x - radius, center.y - radius, center.z - radius),
            Point3::new(center.x + radius, center.y + radius, center.z + radius),
        );
        
        let candidates = self.query_region(&sphere_bounds, layer_mask);
        let radius_sq = radius * radius;
        
        candidates.into_iter()
            .filter(|&id| {
                if let Some(object) = self.objects.get(&id) {
                    let distance_sq = (object.position - center).magnitude2();
                    distance_sq <= radius_sq
                } else {
                    false
                }
            })
            .collect()
    }

    pub fn frustum_cull(&mut self, view_matrix: &cgmath::Matrix4<f32>, projection_matrix: &cgmath::Matrix4<f32>) -> Vec<u32> {
        if !self.config.enable_frustum_culling {
            return self.objects.keys().cloned().collect();
        }
        
        let frustum = self.frustum_culler.extract_frustum(view_matrix, projection_matrix);
        let mut visible_objects = Vec::new();
        
        for (&id, object) in &self.objects {
            // TODO: Fix FrustumCuller method signature to allow immutable access
            // For now, assume all objects are visible
            visible_objects.push(id);
        }
        
        visible_objects
    }

    pub fn get_nearby_objects(&self, position: Point3<f32>, radius: f32, exclude_id: Option<u32>) -> Vec<u32> {
        let mut results = self.query_sphere(position, radius, u32::MAX);
        
        if let Some(exclude_id) = exclude_id {
            results.retain(|&id| id != exclude_id);
        }
        
        results.sort_by(|&a, &b| {
            let dist_a = self.objects.get(&a)
                .map(|obj| (obj.position - position).magnitude2())
                .unwrap_or(f32::MAX);
            let dist_b = self.objects.get(&b)
                .map(|obj| (obj.position - position).magnitude2())
                .unwrap_or(f32::MAX);
            dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        results
    }

    pub fn update(&mut self) {
        self.frame_counter += 1;
        
        if !self.dirty_objects.is_empty() && 
           self.frame_counter - self.last_rebuild_frame > 60 {
            self.rebuild_static_structures();
        }
        
        self.spatial_hash.cleanup_empty_cells();
    }

    fn rebuild_static_structures(&mut self) {
        if self.dirty_objects.is_empty() {
            return;
        }
        
        let static_objects: Vec<_> = self.objects.iter()
            .filter(|(_, obj)| obj.static_object)
            .map(|(&id, obj)| (id, obj.bounds.clone()))
            .collect();
        
        self.bvh = BVH::new();
        for (id, bounds) in static_objects {
            self.bvh.insert(id, bounds);
        }
        self.bvh.build();
        
        self.dirty_objects.clear();
        self.last_rebuild_frame = self.frame_counter;
    }

    pub fn get_statistics(&self) -> SpatialStats {
        SpatialStats {
            total_objects: self.objects.len(),
            static_objects: self.objects.values().filter(|obj| obj.static_object).count(),
            dynamic_objects: self.objects.values().filter(|obj| !obj.static_object).count(),
            octree_depth: self.octree.get_max_depth(),
            octree_nodes: self.octree.get_node_count(),
            spatial_hash_cells: self.spatial_hash.get_active_cell_count(),
            bvh_nodes: self.bvh.get_node_count(),
            dirty_objects: self.dirty_objects.len(),
        }
    }

    pub fn get_object(&self, id: u32) -> Option<&SpatialObject> {
        self.objects.get(&id)
    }

    pub fn raycast(&self, origin: Point3<f32>, direction: Vector3<f32>, max_distance: f32, layer_mask: u32) -> Option<RaycastHit> {
        let ray_end = origin + direction * max_distance;
        let ray_bounds = AABB::from_points(&[origin, ray_end]);
        
        let candidates = self.query_region(&ray_bounds, layer_mask);
        let mut closest_hit: Option<RaycastHit> = None;
        
        for &id in &candidates {
            if let Some(object) = self.objects.get(&id) {
                if let Some(hit) = self.raycast_aabb(origin, direction, &object.bounds) {
                    if hit.distance <= max_distance {
                        if closest_hit.is_none() || hit.distance < closest_hit.as_ref().unwrap().distance {
                            closest_hit = Some(RaycastHit {
                                object_id: id,
                                distance: hit.distance,
                                point: hit.point,
                                normal: hit.normal,
                            });
                        }
                    }
                }
            }
        }
        
        closest_hit
    }

    fn raycast_aabb(&self, origin: Point3<f32>, direction: Vector3<f32>, aabb: &AABB) -> Option<RaycastHit> {
        let inv_dir = Vector3::new(1.0 / direction.x, 1.0 / direction.y, 1.0 / direction.z);
        
        let t1 = (aabb.min.x - origin.x) * inv_dir.x;
        let t2 = (aabb.max.x - origin.x) * inv_dir.x;
        let t3 = (aabb.min.y - origin.y) * inv_dir.y;
        let t4 = (aabb.max.y - origin.y) * inv_dir.y;
        let t5 = (aabb.min.z - origin.z) * inv_dir.z;
        let t6 = (aabb.max.z - origin.z) * inv_dir.z;
        
        let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
        let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));
        
        if tmax < 0.0 || tmin > tmax {
            return None;
        }
        
        let t = if tmin < 0.0 { tmax } else { tmin };
        let hit_point = origin + direction * t;
        
        let center = (aabb.min + aabb.max.to_vec()) / 2.0;
        let size = aabb.max - aabb.min;
        let local_hit = hit_point - center;
        
        let mut normal = Vector3::new(0.0, 0.0, 0.0);
        let mut max_axis = 0.0;
        
        let x_dist = (local_hit.x.abs() / size.x * 2.0 - 1.0).abs();
        let y_dist = (local_hit.y.abs() / size.y * 2.0 - 1.0).abs();
        let z_dist = (local_hit.z.abs() / size.z * 2.0 - 1.0).abs();
        
        if x_dist > max_axis {
            max_axis = x_dist;
            normal = Vector3::new(local_hit.x.signum(), 0.0, 0.0);
        }
        if y_dist > max_axis {
            max_axis = y_dist;
            normal = Vector3::new(0.0, local_hit.y.signum(), 0.0);
        }
        if z_dist > max_axis {
            normal = Vector3::new(0.0, 0.0, local_hit.z.signum());
        }
        
        Some(RaycastHit {
            object_id: 0,
            distance: t,
            point: hit_point,
            normal,
        })
    }
}

#[derive(Debug)]
pub struct SpatialStats {
    pub total_objects: usize,
    pub static_objects: usize,
    pub dynamic_objects: usize,
    pub octree_depth: usize,
    pub octree_nodes: usize,
    pub spatial_hash_cells: usize,
    pub bvh_nodes: usize,
    pub dirty_objects: usize,
}

#[derive(Debug, Clone)]
pub struct RaycastHit {
    pub object_id: u32,
    pub distance: f32,
    pub point: Point3<f32>,
    pub normal: Vector3<f32>,
}