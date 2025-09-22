/*!
 * Robin Engine - Unified Interactive Demo
 *
 * The flagship demonstration of Robin Engine capabilities featuring:
 * - Native wgpu rendering optimized for all platforms
 * - Voxel world construction with advanced building tools
 * - Real-time physics and dynamic lighting
 * - Engineer Build Mode with sophisticated construction capabilities
 * - Unified architecture eliminating code duplication
 */

use robin::engine::{
    generation::VoxelWorld,
    world::VoxelType,
    build_mode::EngineerBuildMode,
    graphics::{PlatformCapabilities, detect_best_backend, Camera, Mesh},
    input::InputManager,
    ui::UIManager,
    error::{RobinResult, RobinError},
    math::Vec3,
    physics3d::{PhysicsWorld3D, PhysicsHandle, BodyDescriptor, ColliderShape3D, Physics3DConfig},
};
use rand::Rng;
use cgmath::{InnerSpace, EuclideanSpace, SquareMatrix};

use winit::{
    event::{Event, WindowEvent, KeyEvent, ElementState, MouseButton},
    event_loop::EventLoop,
    window::{WindowBuilder, Window},
    keyboard::{Key, NamedKey},
    dpi::PhysicalSize,
};
use wgpu::{Surface, SurfaceConfiguration, Device, Queue, util::DeviceExt};
use std::time::{Instant, Duration};
use std::sync::Arc;
use std::collections::HashMap;

#[repr(C, align(16))]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],
    view_pos: [f32; 4],
    light_pos: [f32; 4],
    light_color: [f32; 4],
    // Cascaded shadow maps (3 cascades)
    light_space_matrix_0: [[f32; 4]; 4],
    light_space_matrix_1: [[f32; 4]; 4],
    light_space_matrix_2: [[f32; 4]; 4],
    cascade_splits: [f32; 4], // x,y,z = split distances, w = num_cascades
    shadow_bias: [f32; 4], // x = bias, y = normal_bias, z = pcf_radius, w = enable_shadows
    time: f32,
    _padding: [f32; 3],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct ShadowUniforms {
    light_view_proj: [[f32; 4]; 4],
}

#[derive(Debug, Clone)]
struct RaycastResult {
    pub hit: bool,
    pub position: Vec3,
    pub block_position: (i32, i32, i32),
    pub face_normal: Vec3,
    pub distance: f32,
}

impl Default for RaycastResult {
    fn default() -> Self {
        Self {
            hit: false,
            position: Vec3::new(0.0, 0.0, 0.0),
            block_position: (0, 0, 0),
            face_normal: Vec3::new(0.0, 0.0, 0.0),
            distance: 0.0,
        }
    }
}

// Chunk system constants
const CHUNK_SIZE: i32 = 32;
const CHUNK_SIZE_F: f32 = 32.0;
const CHUNK_RENDER_DISTANCE: i32 = 4; // Load chunks within this distance

/// Data structure for saving voxel world data
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct WorldSaveData {
    version: String,
    save_name: String,
    created_at: std::time::SystemTime,
    player_position: Vec3,
    player_target: Vec3,
    chunks: std::collections::HashMap<(i32, i32, i32), ChunkSaveData>,
}

/// Data structure for saving individual chunk data
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ChunkSaveData {
    position: (i32, i32, i32),
    voxels: std::collections::HashMap<(u8, u8, u8), VoxelType>,
    created_at: std::time::SystemTime,
}

impl ChunkSaveData {
    fn from_chunk(chunk: &Chunk) -> Self {
        Self {
            position: chunk.position,
            voxels: chunk.voxels.clone(),
            created_at: std::time::SystemTime::now(),
        }
    }
}

// Chunk data structure
/// Level of Detail levels for chunks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LODLevel {
    High = 0,    // Full detail - all voxels rendered
    Medium = 1,  // Half detail - every 2nd voxel
    Low = 2,     // Quarter detail - every 4th voxel
    VeryLow = 3, // Eighth detail - every 8th voxel or simplified shape
}

impl LODLevel {
    fn from_distance(distance: f32) -> Self {
        if distance < 50.0 {
            LODLevel::High
        } else if distance < 100.0 {
            LODLevel::Medium
        } else if distance < 200.0 {
            LODLevel::Low
        } else {
            LODLevel::VeryLow
        }
    }

    fn voxel_step(&self) -> u8 {
        match self {
            LODLevel::High => 1,
            LODLevel::Medium => 2,
            LODLevel::Low => 4,
            LODLevel::VeryLow => 8,
        }
    }

    fn should_render_voxel(&self, x: u8, y: u8, z: u8) -> bool {
        let step = self.voxel_step();
        x % step == 0 && y % step == 0 && z % step == 0
    }
}

struct Chunk {
    position: (i32, i32, i32), // Chunk coordinates
    voxels: HashMap<(u8, u8, u8), VoxelType>, // Local coordinates (0-31)
    mesh: Mesh,
    mesh_dirty: bool, // Whether mesh needs regeneration
    last_accessed: std::time::Instant,
    lod_level: LODLevel, // Current level of detail
    distance_to_camera: f32, // Distance from camera for LOD calculation
}

impl Chunk {
    fn new(chunk_x: i32, chunk_y: i32, chunk_z: i32) -> Self {
        Self {
            position: (chunk_x, chunk_y, chunk_z),
            voxels: HashMap::new(),
            mesh: Mesh::default(),
            mesh_dirty: true,
            last_accessed: std::time::Instant::now(),
            lod_level: LODLevel::High,
            distance_to_camera: 0.0,
        }
    }

    fn set_voxel(&mut self, local_x: u8, local_y: u8, local_z: u8, voxel_type: VoxelType) {
        self.voxels.insert((local_x, local_y, local_z), voxel_type);
        self.mesh_dirty = true;
        self.last_accessed = std::time::Instant::now();
    }

    fn get_voxel(&self, local_x: u8, local_y: u8, local_z: u8) -> Option<VoxelType> {
        self.voxels.get(&(local_x, local_y, local_z)).copied()
    }

    fn remove_voxel(&mut self, local_x: u8, local_y: u8, local_z: u8) -> bool {
        let removed = self.voxels.remove(&(local_x, local_y, local_z)).is_some();
        if removed {
            self.mesh_dirty = true;
            self.last_accessed = std::time::Instant::now();
        }
        removed
    }

    /// Update the chunk's LOD level based on distance from camera
    fn update_lod(&mut self, camera_position: Vec3) {
        // Calculate distance from camera to chunk center
        let chunk_center = Vec3::new(
            (self.position.0 as f32 * CHUNK_SIZE_F) + (CHUNK_SIZE_F * 0.5),
            (self.position.1 as f32 * CHUNK_SIZE_F) + (CHUNK_SIZE_F * 0.5),
            (self.position.2 as f32 * CHUNK_SIZE_F) + (CHUNK_SIZE_F * 0.5),
        );

        let distance = (camera_position - chunk_center).magnitude();
        self.distance_to_camera = distance;

        let new_lod = LODLevel::from_distance(distance);
        if new_lod != self.lod_level {
            self.lod_level = new_lod;
            self.mesh_dirty = true; // Mark for regeneration with new LOD
        }
    }

    /// Check if this chunk should be rendered based on LOD and distance
    fn should_render(&self, max_render_distance: f32) -> bool {
        self.distance_to_camera <= max_render_distance
    }

    /// Get effective voxel count for this LOD level (for performance tracking)
    fn effective_voxel_count(&self) -> usize {
        let step = self.lod_level.voxel_step() as usize;
        self.voxels.len() / (step * step * step).max(1)
    }
}

// Coordinate conversion utilities
fn world_to_chunk_coords(world_x: i32, world_y: i32, world_z: i32) -> ((i32, i32, i32), (u8, u8, u8)) {
    let chunk_x = if world_x >= 0 { world_x / CHUNK_SIZE } else { (world_x - CHUNK_SIZE + 1) / CHUNK_SIZE };
    let chunk_y = if world_y >= 0 { world_y / CHUNK_SIZE } else { (world_y - CHUNK_SIZE + 1) / CHUNK_SIZE };
    let chunk_z = if world_z >= 0 { world_z / CHUNK_SIZE } else { (world_z - CHUNK_SIZE + 1) / CHUNK_SIZE };

    let local_x = (world_x - chunk_x * CHUNK_SIZE) as u8;
    let local_y = (world_y - chunk_y * CHUNK_SIZE) as u8;
    let local_z = (world_z - chunk_z * CHUNK_SIZE) as u8;

    ((chunk_x, chunk_y, chunk_z), (local_x, local_y, local_z))
}

fn chunk_to_world_coords(chunk_x: i32, chunk_y: i32, chunk_z: i32, local_x: u8, local_y: u8, local_z: u8) -> (i32, i32, i32) {
    (
        chunk_x * CHUNK_SIZE + local_x as i32,
        chunk_y * CHUNK_SIZE + local_y as i32,
        chunk_z * CHUNK_SIZE + local_z as i32,
    )
}

// Chunk management system
#[derive(Debug)]
struct ChunkManager {
    chunks: HashMap<(i32, i32, i32), Chunk>,
    camera_position: Vec3,
    render_distance: i32,
    loading_queue: Vec<(i32, i32, i32)>,
    unloading_queue: Vec<(i32, i32, i32)>,
    max_chunks_loaded: usize,
    chunks_per_frame: usize,
    last_position: Vec3,
    movement_threshold: f32,
    performance_stats: ChunkPerformanceStats,
}

#[derive(Debug, Clone, Default)]
struct ChunkPerformanceStats {
    total_chunks_loaded: usize,
    chunks_generated_this_frame: usize,
    chunks_unloaded_this_frame: usize,
    memory_usage_estimate: usize,
    average_generation_time: f32,
}

impl ChunkManager {
    fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            camera_position: Vec3::new(0.0, 0.0, 0.0),
            render_distance: CHUNK_RENDER_DISTANCE,
            loading_queue: Vec::new(),
            unloading_queue: Vec::new(),
            max_chunks_loaded: 512, // Limit to ~512 chunks for memory management
            chunks_per_frame: 2,    // Generate max 2 chunks per frame to avoid frame drops
            last_position: Vec3::new(0.0, 0.0, 0.0),
            movement_threshold: 8.0, // Only update chunks if moved 8+ units
            performance_stats: ChunkPerformanceStats::default(),
        }
    }

    fn update_camera_position(&mut self, new_position: Vec3) {
        // Only update if player moved significantly
        let movement_distance = (new_position - self.last_position).magnitude();
        if movement_distance > self.movement_threshold {
            self.camera_position = new_position;
            self.last_position = new_position;
            self.queue_chunks_for_loading();
            self.queue_chunks_for_unloading();
        }

        // Update LOD levels for all chunks based on new camera position
        self.update_chunk_lod_levels(new_position);

        // Process queues every frame regardless
        self.process_loading_queue();
        self.process_unloading_queue();
        self.update_performance_stats();
    }

    fn queue_chunks_for_loading(&mut self) {
        let camera_chunk_x = (self.camera_position.x / CHUNK_SIZE_F) as i32;
        let camera_chunk_y = (self.camera_position.y / CHUNK_SIZE_F) as i32;
        let camera_chunk_z = (self.camera_position.z / CHUNK_SIZE_F) as i32;

        let mut needed_chunks = Vec::new();

        // Calculate which chunks should be loaded (prioritize by distance)
        for dx in -self.render_distance..=self.render_distance {
            for dy in -2..=2 { // Limit vertical chunks for now
                for dz in -self.render_distance..=self.render_distance {
                    let chunk_coords = (
                        camera_chunk_x + dx,
                        camera_chunk_y + dy,
                        camera_chunk_z + dz,
                    );

                    if !self.chunks.contains_key(&chunk_coords) &&
                       !self.loading_queue.contains(&chunk_coords) {
                        let distance = dx.abs().max(dy.abs()).max(dz.abs());
                        needed_chunks.push((chunk_coords, distance));
                    }
                }
            }
        }

        // Sort by distance (closest first) and add to loading queue
        needed_chunks.sort_by_key(|(_, distance)| *distance);
        for (chunk_coords, _) in needed_chunks {
            self.loading_queue.push(chunk_coords);
        }
    }

    fn queue_chunks_for_unloading(&mut self) {
        let camera_chunk_x = (self.camera_position.x / CHUNK_SIZE_F) as i32;
        let camera_chunk_y = (self.camera_position.y / CHUNK_SIZE_F) as i32;
        let camera_chunk_z = (self.camera_position.z / CHUNK_SIZE_F) as i32;

        // Find chunks to unload (distance or memory pressure)
        let chunks_to_unload: Vec<_> = self.chunks.keys()
            .filter(|(cx, cy, cz)| {
                let distance = (cx - camera_chunk_x).abs().max((cy - camera_chunk_y).abs()).max((cz - camera_chunk_z).abs());
                distance > self.render_distance + 2 || self.chunks.len() > self.max_chunks_loaded
            })
            .cloned()
            .collect();

        for chunk_coords in chunks_to_unload {
            if !self.unloading_queue.contains(&chunk_coords) {
                self.unloading_queue.push(chunk_coords);
            }
        }
    }

    fn process_loading_queue(&mut self) {
        let mut chunks_processed = 0;
        self.performance_stats.chunks_generated_this_frame = 0;

        while chunks_processed < self.chunks_per_frame && !self.loading_queue.is_empty() {
            if let Some(chunk_coords) = self.loading_queue.pop() {
                let start_time = std::time::Instant::now();

                let mut chunk = Chunk::new(chunk_coords.0, chunk_coords.1, chunk_coords.2);
                self.generate_chunk_terrain(&mut chunk);
                chunk.mesh_dirty = true; // Mark that mesh needs regeneration
                self.chunks.insert(chunk_coords, chunk);

                let generation_time = start_time.elapsed().as_secs_f32();
                self.performance_stats.average_generation_time =
                    (self.performance_stats.average_generation_time + generation_time) / 2.0;

                chunks_processed += 1;
                self.performance_stats.chunks_generated_this_frame += 1;
                self.performance_stats.total_chunks_loaded += 1;
            }
        }
    }

    fn process_unloading_queue(&mut self) {
        let max_unloads = 3; // Don't unload too many at once
        let mut chunks_unloaded = 0;
        self.performance_stats.chunks_unloaded_this_frame = 0;

        while chunks_unloaded < max_unloads && !self.unloading_queue.is_empty() {
            if let Some(chunk_coords) = self.unloading_queue.pop() {
                if self.chunks.remove(&chunk_coords).is_some() {
                    chunks_unloaded += 1;
                    self.performance_stats.chunks_unloaded_this_frame += 1;
                }
            }
        }
    }

    fn update_performance_stats(&mut self) {
        // Estimate memory usage (rough calculation)
        self.performance_stats.memory_usage_estimate =
            self.chunks.len() * std::mem::size_of::<Chunk>() +
            self.chunks.values().map(|chunk| chunk.voxels.len() * std::mem::size_of::<VoxelType>()).sum::<usize>();
    }

    fn generate_chunk_terrain(&self, chunk: &mut Chunk) {
        let (chunk_x, chunk_y, chunk_z) = chunk.position;

        // Generate terrain only for chunks at or below ground level
        if chunk_y > 0 {
            return; // Don't generate terrain in sky chunks
        }

        let mut voxels_generated = 0;

        for local_x in 0..32u8 {
            for local_z in 0..32u8 {
                let world_x = chunk_x * CHUNK_SIZE + local_x as i32;
                let world_z = chunk_z * CHUNK_SIZE + local_z as i32;

                // Enhanced terrain generation - create more substantial ground
                let base_height = 8; // Ground level at y=8
                let noise = ((world_x.abs() + world_z.abs()) % 8) as i32;
                let height = base_height + noise; // Height varies from 8 to 15

                for local_y in 0..32u8 {
                    let world_y = chunk_y * CHUNK_SIZE + local_y as i32;

                    if world_y <= height {
                        let voxel_type = if world_y == height {
                            VoxelType::Grass
                        } else if world_y > height - 3 {
                            VoxelType::Dirt
                        } else {
                            VoxelType::Stone
                        };
                        chunk.set_voxel(local_x, local_y, local_z, voxel_type);
                        voxels_generated += 1;
                    }
                }
            }
        }

        // Debug output for terrain generation
        if voxels_generated > 0 {
            println!("🌍 Generated {} voxels in chunk ({}, {}, {})",
                     voxels_generated, chunk_x, chunk_y, chunk_z);
        }
    }

    fn set_voxel(&mut self, world_x: i32, world_y: i32, world_z: i32, voxel_type: VoxelType) -> bool {
        let ((chunk_x, chunk_y, chunk_z), (local_x, local_y, local_z)) =
            world_to_chunk_coords(world_x, world_y, world_z);

        if let Some(chunk) = self.chunks.get_mut(&(chunk_x, chunk_y, chunk_z)) {
            chunk.set_voxel(local_x, local_y, local_z, voxel_type);
            true
        } else {
            false
        }
    }

    fn remove_voxel(&mut self, world_x: i32, world_y: i32, world_z: i32) -> bool {
        let ((chunk_x, chunk_y, chunk_z), (local_x, local_y, local_z)) =
            world_to_chunk_coords(world_x, world_y, world_z);

        if let Some(chunk) = self.chunks.get_mut(&(chunk_x, chunk_y, chunk_z)) {
            chunk.remove_voxel(local_x, local_y, local_z)
        } else {
            false
        }
    }

    fn get_voxel(&self, world_x: i32, world_y: i32, world_z: i32) -> Option<VoxelType> {
        let ((chunk_x, chunk_y, chunk_z), (local_x, local_y, local_z)) =
            world_to_chunk_coords(world_x, world_y, world_z);

        if let Some(chunk) = self.chunks.get(&(chunk_x, chunk_y, chunk_z)) {
            chunk.get_voxel(local_x, local_y, local_z)
        } else {
            None
        }
    }

    fn get_loaded_chunks(&self) -> &HashMap<(i32, i32, i32), Chunk> {
        &self.chunks
    }

    fn get_loaded_chunks_mut(&mut self) -> &mut HashMap<(i32, i32, i32), Chunk> {
        &mut self.chunks
    }

    fn get_performance_stats(&self) -> &ChunkPerformanceStats {
        &self.performance_stats
    }

    fn print_performance_stats(&self) {
        let stats = &self.performance_stats;
        println!(
            "📊 Chunk Performance: {} loaded, {:.1}MB mem, avg gen: {:.3}ms, +{} -{} this frame",
            stats.total_chunks_loaded,
            stats.memory_usage_estimate as f32 / (1024.0 * 1024.0),
            stats.average_generation_time * 1000.0,
            stats.chunks_generated_this_frame,
            stats.chunks_unloaded_this_frame
        );
    }

    /// Update LOD levels for all loaded chunks based on camera position
    fn update_chunk_lod_levels(&mut self, camera_position: Vec3) {
        for chunk in self.chunks.values_mut() {
            chunk.update_lod(camera_position);
        }
    }

    /// Get chunks sorted by distance for LOD rendering
    fn get_chunks_by_distance(&self) -> Vec<&Chunk> {
        let mut chunks: Vec<&Chunk> = self.chunks.values().collect();
        chunks.sort_by(|a, b| a.distance_to_camera.partial_cmp(&b.distance_to_camera).unwrap());
        chunks
    }

    /// Get rendering statistics for LOD system
    fn get_lod_stats(&self) -> (usize, usize, usize, usize) {
        let mut high = 0;
        let mut medium = 0;
        let mut low = 0;
        let mut very_low = 0;

        for chunk in self.chunks.values() {
            match chunk.lod_level {
                LODLevel::High => high += 1,
                LODLevel::Medium => medium += 1,
                LODLevel::Low => low += 1,
                LODLevel::VeryLow => very_low += 1,
            }
        }

        (high, medium, low, very_low)
    }
}

// Data structures for greedy meshing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum FaceDirection {
    PosX, NegX, // East, West
    PosY, NegY, // Up, Down
    PosZ, NegZ, // North, South
}

#[derive(Debug, Clone)]
struct Face {
    position: (i32, i32, i32),     // Voxel position
    direction: FaceDirection,       // Which face of the voxel
    material: VoxelType,           // Material for coloring
    width: u32,                    // Greedy meshing: face width
    height: u32,                   // Greedy meshing: face height
}

impl Face {
    fn new(position: (i32, i32, i32), direction: FaceDirection, material: VoxelType) -> Self {
        Self {
            position,
            direction,
            material,
            width: 1,
            height: 1,
        }
    }

    fn get_normal(&self) -> [f32; 3] {
        match self.direction {
            FaceDirection::PosX => [1.0, 0.0, 0.0],
            FaceDirection::NegX => [-1.0, 0.0, 0.0],
            FaceDirection::PosY => [0.0, 1.0, 0.0],
            FaceDirection::NegY => [0.0, -1.0, 0.0],
            FaceDirection::PosZ => [0.0, 0.0, 1.0],
            FaceDirection::NegZ => [0.0, 0.0, -1.0],
        }
    }

    fn get_adjacent_voxel_pos(&self) -> (i32, i32, i32) {
        let (x, y, z) = self.position;
        match self.direction {
            FaceDirection::PosX => (x + 1, y, z),
            FaceDirection::NegX => (x - 1, y, z),
            FaceDirection::PosY => (x, y + 1, z),
            FaceDirection::NegY => (x, y - 1, z),
            FaceDirection::PosZ => (x, y, z + 1),
            FaceDirection::NegZ => (x, y, z - 1),
        }
    }
}

/// Particle system for visual effects when blocks are placed or removed
#[derive(Debug, Clone)]
struct Particle {
    position: Vec3,
    velocity: Vec3,
    color: [f32; 4], // RGBA
    life: f32,       // 0.0 to 1.0, where 1.0 is newly born and 0.0 is dead
    max_life: f32,   // Total lifetime in seconds
    size: f32,       // Size multiplier
}

impl Particle {
    fn new(position: Vec3, velocity: Vec3, color: [f32; 4], max_life: f32, size: f32) -> Self {
        Self {
            position,
            velocity,
            color,
            life: 1.0,
            max_life,
            size,
        }
    }

    fn update(&mut self, delta_time: f32) -> bool {
        // Update position
        self.position.x += self.velocity.x * delta_time;
        self.position.y += self.velocity.y * delta_time;
        self.position.z += self.velocity.z * delta_time;

        // Apply gravity
        self.velocity.y -= 9.8 * delta_time;

        // Apply air resistance
        self.velocity.x *= 0.98;
        self.velocity.z *= 0.98;

        // Update life
        self.life -= delta_time / self.max_life;

        // Return true if particle is still alive
        self.life > 0.0
    }
}

#[derive(Debug, Clone)]
struct ParticleSystem {
    particles: Vec<Particle>,
    max_particles: usize,
}

impl ParticleSystem {
    fn new(max_particles: usize) -> Self {
        Self {
            particles: Vec::new(),
            max_particles,
        }
    }

    fn emit_block_place_particles(&mut self, position: Vec3, voxel_type: VoxelType) {
        let particle_count = 8; // Number of particles to emit
        let mut rng = rand::thread_rng();

        let base_color = Self::get_voxel_color(voxel_type);

        for _ in 0..particle_count {
            // Emit particles in random directions with some upward bias
            let velocity = Vec3::new(
                rng.gen_range(-3.0..3.0),
                rng.gen_range(1.0..5.0), // Upward bias
                rng.gen_range(-3.0..3.0),
            );

            // Vary color slightly
            let color = [
                (base_color[0] + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0),
                (base_color[1] + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0),
                (base_color[2] + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0),
                1.0,
            ];

            let particle = Particle::new(
                position + Vec3::new(
                    rng.gen_range(-0.3..0.3),
                    rng.gen_range(-0.3..0.3),
                    rng.gen_range(-0.3..0.3),
                ),
                velocity,
                color,
                rng.gen_range(0.5..1.5), // Random lifetime
                rng.gen_range(0.05..0.15), // Random size
            );

            self.add_particle(particle);
        }
    }

    fn emit_block_remove_particles(&mut self, position: Vec3, voxel_type: VoxelType) {
        let particle_count = 12; // More particles for destruction effect
        let mut rng = rand::thread_rng();

        let base_color = Self::get_voxel_color(voxel_type);

        for _ in 0..particle_count {
            // Emit particles in all directions for destruction effect
            let velocity = Vec3::new(
                rng.gen_range(-4.0..4.0),
                rng.gen_range(-1.0..4.0),
                rng.gen_range(-4.0..4.0),
            );

            // Vary color and make slightly darker for destruction
            let color = [
                (base_color[0] * 0.8 + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0),
                (base_color[1] * 0.8 + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0),
                (base_color[2] * 0.8 + rng.gen_range(-0.1..0.1)).clamp(0.0, 1.0),
                1.0,
            ];

            let particle = Particle::new(
                position + Vec3::new(
                    rng.gen_range(-0.4..0.4),
                    rng.gen_range(-0.4..0.4),
                    rng.gen_range(-0.4..0.4),
                ),
                velocity,
                color,
                rng.gen_range(1.0..2.0), // Longer lifetime for destruction
                rng.gen_range(0.08..0.2), // Slightly larger fragments
            );

            self.add_particle(particle);
        }
    }

    fn get_voxel_color(voxel_type: VoxelType) -> [f32; 3] {
        match voxel_type {
            VoxelType::Stone => [0.6, 0.6, 0.6],
            VoxelType::Dirt => [0.4, 0.3, 0.2],
            VoxelType::Grass => [0.2, 0.8, 0.2],
            VoxelType::Sand => [0.9, 0.8, 0.4],
            VoxelType::Wood => [0.6, 0.4, 0.2],
            VoxelType::Leaves => [0.2, 0.6, 0.2],
            VoxelType::Crystal => [0.8, 0.2, 0.8],
            VoxelType::Glass => [0.8, 0.9, 1.0],
            VoxelType::Metal => [0.7, 0.7, 0.8],
            VoxelType::Brick => [0.8, 0.4, 0.3],
            VoxelType::Ice => [0.8, 0.9, 1.0],
            VoxelType::Water => [0.3, 0.5, 1.0],
            VoxelType::Lava => [1.0, 0.3, 0.0],
            VoxelType::Obsidian => [0.2, 0.1, 0.3],
            VoxelType::Air => [1.0, 1.0, 1.0], // Shouldn't be used
        }
    }

    fn add_particle(&mut self, particle: Particle) {
        if self.particles.len() < self.max_particles {
            self.particles.push(particle);
        } else {
            // Replace oldest particle if at max capacity
            if let Some(oldest_index) = self.particles.iter()
                .enumerate()
                .min_by(|a, b| a.1.life.partial_cmp(&b.1.life).unwrap())
                .map(|(i, _)| i) {
                self.particles[oldest_index] = particle;
            }
        }
    }

    fn update(&mut self, delta_time: f32) {
        // Update all particles and remove dead ones
        self.particles.retain_mut(|particle| particle.update(delta_time));
    }

    fn get_particles(&self) -> &[Particle] {
        &self.particles
    }

    fn clear(&mut self) {
        self.particles.clear();
    }
}

struct RobinApp {
    window: Arc<Window>,
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    device: Device,
    queue: Queue,
    // Rendering pipeline
    render_pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    depth_texture: wgpu::Texture,
    depth_texture_view: wgpu::TextureView,
    // Shadow mapping
    shadow_map_size: u32,
    shadow_maps: [wgpu::Texture; 3], // 3 cascades
    shadow_map_views: [wgpu::TextureView; 3],
    shadow_sampler: wgpu::Sampler,
    shadow_bind_group: wgpu::BindGroup,
    shadow_pipeline: wgpu::RenderPipeline,
    shadow_uniform_buffer: wgpu::Buffer,
    shadow_bind_group_layout: wgpu::BindGroupLayout,
    // Mesh buffers
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    // Wireframe highlighting
    wireframe_pipeline: wgpu::RenderPipeline,
    wireframe_vertex_buffer: wgpu::Buffer,
    target_highlight_visible: bool,
    placement_highlight_visible: bool,
    target_highlight_position: Vec3,
    placement_highlight_position: Vec3,
    // UI Rendering
    ui_pipeline: wgpu::RenderPipeline,
    ui_vertex_buffer: Option<wgpu::Buffer>,
    ui_index_buffer: Option<wgpu::Buffer>,
    // Engine systems
    camera: Camera,
    input_manager: InputManager,
    voxel_world: VoxelWorld,
    build_system: EngineerBuildMode,
    ui_system: UIManager,
    world_mesh: Mesh,
    last_frame_time: Instant,
    frame_count: u32,
    fps_timer: Instant,
    // Interaction system
    raycast_result: RaycastResult,
    // Chunk-based voxel world
    chunk_manager: ChunkManager,
    // Texture management
    texture_manager: robin::engine::graphics::TextureManager,
    // Physics system
    physics_world: PhysicsWorld3D,
    player_physics_handle: Option<PhysicsHandle>,
    player_velocity: Vec3,
    player_grounded: bool,
    // Wall construction state
    wall_construction: WallConstructionState,
    // Template system
    template_system: TemplateSystem,
    // Undo/Redo system
    undo_system: UndoRedoSystem,
    // UI Overlay system
    ui_overlay: UIOverlay,
    // Particle system for block interactions
    particle_system: ParticleSystem,
}

#[derive(Debug, Clone)]
struct WallConstructionState {
    is_constructing: bool,
    start_position: Option<(i32, i32, i32)>,
    current_position: Option<(i32, i32, i32)>,
    preview_positions: Vec<(i32, i32, i32)>,
}

#[derive(Debug, Clone)]
enum VoxelAction {
    Place {
        position: (i32, i32, i32),
        voxel_type: VoxelType,
        previous_voxel: Option<VoxelType>,
    },
    Remove {
        position: (i32, i32, i32),
        previous_voxel: VoxelType,
    },
    Batch {
        actions: Vec<VoxelAction>,
        description: String,
    },
}

#[derive(Debug, Clone)]
struct UIOverlay {
    enabled: bool,
    show_help: bool,
    last_fps_update: Instant,
    current_fps: f32,
    frame_count: u32,
    ui_elements: Vec<UIElement>,
}

#[derive(Debug, Clone)]
struct UIElement {
    text: String,
    position: (f32, f32),
    color: [f32; 3],
    size: f32,
}

#[derive(Debug, Clone, Copy)]
struct UIVertex {
    position: [f32; 2],
    color: [f32; 3],
}

// Safety: UIVertex only contains f32 types which are Pod
unsafe impl bytemuck::Pod for UIVertex {}
unsafe impl bytemuck::Zeroable for UIVertex {}

impl UIOverlay {
    fn new() -> Self {
        Self {
            enabled: true,
            show_help: false,
            last_fps_update: Instant::now(),
            current_fps: 0.0,
            frame_count: 0,
            ui_elements: Vec::new(),
        }
    }

    fn update(&mut self, delta_time: f32) {
        self.frame_count += 1;

        // Update FPS every second
        if self.last_fps_update.elapsed().as_secs_f32() >= 1.0 {
            self.current_fps = self.frame_count as f32 / self.last_fps_update.elapsed().as_secs_f32();
            self.frame_count = 0;
            self.last_fps_update = Instant::now();
        }
    }

    fn update_ui_elements(&mut self, robin_app: &RobinApp) {
        let current_mode = robin_app.build_system.get_mode();
        let current_material = robin::engine::world::construction::VoxelType::Stone; // Placeholder
        self.update_ui_elements_with_data(robin_app, current_mode, current_material);
    }

    fn update_ui_elements_with_data(&mut self, robin_app: &RobinApp, current_mode: robin::engine::build_mode::BuildModeState, current_material: VoxelType) {
        self.ui_elements.clear();

        if !self.enabled {
            return;
        }

        let mut y_offset = 20.0;
        let line_height = 25.0;

        // Mode info
        self.ui_elements.push(UIElement {
            text: format!("Mode: {}", Self::get_mode_name(current_mode)),
            position: (20.0, y_offset),
            color: [1.0, 1.0, 1.0],
            size: 16.0,
        });
        y_offset += line_height;

        // Material info
        self.ui_elements.push(UIElement {
            text: format!("Material: {}", Self::get_material_name(current_material)),
            position: (20.0, y_offset),
            color: Self::get_material_color(current_material),
            size: 16.0,
        });
        y_offset += line_height;

        // Template info if in template mode
        if let Some(template) = robin_app.template_system.get_current_template() {
            self.ui_elements.push(UIElement {
                text: format!("Template: {} ({}°)", template.name, robin_app.template_system.current_rotation * 90),
                position: (20.0, y_offset),
                color: [1.0, 1.0, 0.5],
                size: 16.0,
            });
            y_offset += line_height;
        }

        // FPS info
        self.ui_elements.push(UIElement {
            text: format!("FPS: {:.1}", self.current_fps),
            position: (20.0, y_offset),
            color: [0.8, 0.8, 0.8],
            size: 16.0,
        });
        y_offset += line_height;

        // Undo/Redo status
        self.ui_elements.push(UIElement {
            text: format!("Undo: {} | Redo: {}",
                if robin_app.undo_system.can_undo() { "Yes" } else { "No" },
                if robin_app.undo_system.can_redo() { "Yes" } else { "No" }
            ),
            position: (20.0, y_offset),
            color: [0.7, 0.9, 0.7],
            size: 16.0,
        });
        y_offset += line_height;

        // Help text if enabled
        if self.show_help {
            y_offset += 20.0;
            let help_texts = vec![
                "=== CONTROLS ===",
                "WASD: Move camera",
                "Mouse: Look around",
                "1-9: Select material",
                "M: Cycle build mode",
                "T: Cycle template",
                "R: Rotate template",
                "Ctrl+Z: Undo",
                "Ctrl+Y: Redo",
                "F1: Toggle help",
                "F2: Toggle overlay",
                "Left Click: Remove voxel",
                "Right Click: Place voxel",
            ];

            for help_text in help_texts {
                self.ui_elements.push(UIElement {
                    text: help_text.to_string(),
                    position: (20.0, y_offset),
                    color: [0.9, 0.9, 0.6],
                    size: 14.0,
                });
                y_offset += line_height;
            }
        }
    }

    fn generate_ui_mesh(&self) -> (Vec<UIVertex>, Vec<u32>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut index_offset = 0;

        for element in &self.ui_elements {
            // Create a simple rectangle for each text element (simulating text background)
            let x = element.position.0;
            let y = element.position.1;
            let width = element.text.len() as f32 * element.size * 0.6; // Approximate width
            let height = element.size;

            // Convert to normalized device coordinates (-1 to 1)
            let screen_width = 1200.0;
            let screen_height = 800.0;

            let left = (x / screen_width) * 2.0 - 1.0;
            let right = ((x + width) / screen_width) * 2.0 - 1.0;
            let top = 1.0 - (y / screen_height) * 2.0;
            let bottom = 1.0 - ((y + height) / screen_height) * 2.0;

            // Create rectangle vertices
            vertices.extend_from_slice(&[
                UIVertex { position: [left, top], color: element.color },
                UIVertex { position: [right, top], color: element.color },
                UIVertex { position: [right, bottom], color: element.color },
                UIVertex { position: [left, bottom], color: element.color },
            ]);

            // Create indices for two triangles forming a rectangle
            indices.extend_from_slice(&[
                index_offset, index_offset + 1, index_offset + 2,
                index_offset, index_offset + 2, index_offset + 3,
            ]);
            index_offset += 4;
        }

        (vertices, indices)
    }

    fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    fn toggle_enabled(&mut self) {
        self.enabled = !self.enabled;
    }

    fn get_material_color(voxel_type: VoxelType) -> [f32; 3] {
        match voxel_type {
            VoxelType::Air => [0.0, 0.0, 0.0], // Should not be visible
            VoxelType::Stone => [0.5, 0.5, 0.5],
            VoxelType::Dirt => [0.4, 0.25, 0.1],
            VoxelType::Grass => [0.2, 0.8, 0.2],
            VoxelType::Sand => [0.9, 0.8, 0.4],
            VoxelType::Wood => [0.6, 0.3, 0.1],
            VoxelType::Leaves => [0.2, 0.5, 0.2],
            VoxelType::Crystal => [0.8, 0.4, 0.9],
            VoxelType::Glass => [0.8, 0.9, 1.0],
            VoxelType::Metal => [0.7, 0.7, 0.8],
            VoxelType::Brick => [0.6, 0.3, 0.2],
            VoxelType::Ice => [0.7, 0.9, 1.0],
            VoxelType::Water => [0.2, 0.4, 0.9],
            VoxelType::Lava => [1.0, 0.3, 0.0],
            VoxelType::Obsidian => [0.1, 0.1, 0.1],
        }
    }

    fn get_material_name(voxel_type: VoxelType) -> &'static str {
        match voxel_type {
            VoxelType::Air => "Air",
            VoxelType::Stone => "Stone",
            VoxelType::Dirt => "Dirt",
            VoxelType::Grass => "Grass",
            VoxelType::Sand => "Sand",
            VoxelType::Wood => "Wood",
            VoxelType::Leaves => "Leaves",
            VoxelType::Crystal => "Crystal",
            VoxelType::Glass => "Glass",
            VoxelType::Metal => "Metal",
            VoxelType::Brick => "Brick",
            VoxelType::Ice => "Ice",
            VoxelType::Water => "Water",
            VoxelType::Lava => "Lava",
            VoxelType::Obsidian => "Obsidian",
        }
    }

    fn get_mode_name(mode: robin::engine::build_mode::BuildModeState) -> &'static str {
        use robin::engine::build_mode::BuildModeState;
        match mode {
            BuildModeState::Build => "Build",
            BuildModeState::Test => "Test",
            BuildModeState::Play => "Play",
        }
    }
}

#[derive(Debug, Clone)]
struct UndoRedoSystem {
    undo_stack: Vec<VoxelAction>,
    redo_stack: Vec<VoxelAction>,
    max_history: usize,
    batch_mode: bool,
    current_batch: Vec<VoxelAction>,
    current_batch_description: String,
}

impl UndoRedoSystem {
    fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history: 100, // Store last 100 actions
            batch_mode: false,
            current_batch: Vec::new(),
            current_batch_description: String::new(),
        }
    }

    fn record_action(&mut self, action: VoxelAction) {
        if self.batch_mode {
            self.current_batch.push(action);
        } else {
            self.undo_stack.push(action);
            // Clear redo stack when new action is performed
            self.redo_stack.clear();

            // Limit history size
            if self.undo_stack.len() > self.max_history {
                self.undo_stack.remove(0);
            }
        }
    }

    fn start_batch(&mut self, description: String) {
        self.batch_mode = true;
        self.current_batch.clear();
        self.current_batch_description = description;
    }

    fn end_batch(&mut self) {
        if self.batch_mode && !self.current_batch.is_empty() {
            let batch_action = VoxelAction::Batch {
                actions: self.current_batch.clone(),
                description: self.current_batch_description.clone(),
            };
            self.undo_stack.push(batch_action);
            // Clear redo stack when new action is performed
            self.redo_stack.clear();

            // Limit history size
            if self.undo_stack.len() > self.max_history {
                self.undo_stack.remove(0);
            }
        }
        self.batch_mode = false;
        self.current_batch.clear();
        self.current_batch_description.clear();
    }

    fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    fn get_undo_description(&self) -> Option<String> {
        self.undo_stack.last().map(|action| match action {
            VoxelAction::Place { .. } => "Place voxel".to_string(),
            VoxelAction::Remove { .. } => "Remove voxel".to_string(),
            VoxelAction::Batch { description, .. } => description.clone(),
        })
    }

    fn get_redo_description(&self) -> Option<String> {
        self.redo_stack.last().map(|action| match action {
            VoxelAction::Place { .. } => "Place voxel".to_string(),
            VoxelAction::Remove { .. } => "Remove voxel".to_string(),
            VoxelAction::Batch { description, .. } => description.clone(),
        })
    }

    fn undo(&mut self) -> Option<VoxelAction> {
        if let Some(action) = self.undo_stack.pop() {
            self.redo_stack.push(action.clone());
            Some(action)
        } else {
            None
        }
    }

    fn redo(&mut self) -> Option<VoxelAction> {
        if let Some(action) = self.redo_stack.pop() {
            self.undo_stack.push(action.clone());
            Some(action)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
struct TemplateSystem {
    templates: HashMap<String, Template>,
    current_template: Option<String>,
    current_rotation: u8, // 0-3 for 90-degree rotations
    preview_active: bool,
    preview_position: Option<(i32, i32, i32)>,
    preview_voxels: Vec<(i32, i32, i32, VoxelType)>,
}

#[derive(Debug, Clone)]
struct Template {
    name: String,
    description: String,
    voxels: Vec<(i32, i32, i32, VoxelType)>, // Relative positions
    size: (i32, i32, i32), // Bounding box
}

impl TemplateSystem {
    fn new() -> Self {
        let mut templates = HashMap::new();

        // Simple house template
        let house_voxels = vec![
            // Foundation (4x4)
            (0, 0, 0, VoxelType::Stone), (1, 0, 0, VoxelType::Stone), (2, 0, 0, VoxelType::Stone), (3, 0, 0, VoxelType::Stone),
            (0, 0, 1, VoxelType::Stone), (1, 0, 1, VoxelType::Stone), (2, 0, 1, VoxelType::Stone), (3, 0, 1, VoxelType::Stone),
            (0, 0, 2, VoxelType::Stone), (1, 0, 2, VoxelType::Stone), (2, 0, 2, VoxelType::Stone), (3, 0, 2, VoxelType::Stone),
            (0, 0, 3, VoxelType::Stone), (1, 0, 3, VoxelType::Stone), (2, 0, 3, VoxelType::Stone), (3, 0, 3, VoxelType::Stone),

            // Walls (height 3)
            (0, 1, 0, VoxelType::Wood), (0, 2, 0, VoxelType::Wood), (0, 3, 0, VoxelType::Wood),
            (3, 1, 0, VoxelType::Wood), (3, 2, 0, VoxelType::Wood), (3, 3, 0, VoxelType::Wood),
            (0, 1, 3, VoxelType::Wood), (0, 2, 3, VoxelType::Wood), (0, 3, 3, VoxelType::Wood),
            (3, 1, 3, VoxelType::Wood), (3, 2, 3, VoxelType::Wood), (3, 3, 3, VoxelType::Wood),

            (1, 1, 0, VoxelType::Wood), (2, 1, 0, VoxelType::Wood),
            (1, 2, 0, VoxelType::Wood), (2, 2, 0, VoxelType::Wood),
            (1, 3, 0, VoxelType::Wood), (2, 3, 0, VoxelType::Wood),

            (1, 1, 3, VoxelType::Wood), (2, 1, 3, VoxelType::Wood),
            (1, 2, 3, VoxelType::Wood), (2, 2, 3, VoxelType::Wood),
            (1, 3, 3, VoxelType::Wood), (2, 3, 3, VoxelType::Wood),

            // Roof
            (0, 4, 0, VoxelType::Wood), (1, 4, 0, VoxelType::Wood), (2, 4, 0, VoxelType::Wood), (3, 4, 0, VoxelType::Wood),
            (0, 4, 1, VoxelType::Wood), (1, 4, 1, VoxelType::Wood), (2, 4, 1, VoxelType::Wood), (3, 4, 1, VoxelType::Wood),
            (0, 4, 2, VoxelType::Wood), (1, 4, 2, VoxelType::Wood), (2, 4, 2, VoxelType::Wood), (3, 4, 2, VoxelType::Wood),
            (0, 4, 3, VoxelType::Wood), (1, 4, 3, VoxelType::Wood), (2, 4, 3, VoxelType::Wood), (3, 4, 3, VoxelType::Wood),
        ];

        let house = Template {
            name: "Simple House".to_string(),
            description: "A basic 4x4 house with walls and roof".to_string(),
            voxels: house_voxels,
            size: (4, 5, 4),
        };

        // Bridge template
        let bridge_voxels = vec![
            // Bridge deck (7 blocks long, 3 wide)
            (0, 0, 0, VoxelType::Wood), (0, 0, 1, VoxelType::Wood), (0, 0, 2, VoxelType::Wood),
            (1, 0, 0, VoxelType::Wood), (1, 0, 1, VoxelType::Wood), (1, 0, 2, VoxelType::Wood),
            (2, 0, 0, VoxelType::Wood), (2, 0, 1, VoxelType::Wood), (2, 0, 2, VoxelType::Wood),
            (3, 0, 0, VoxelType::Wood), (3, 0, 1, VoxelType::Wood), (3, 0, 2, VoxelType::Wood),
            (4, 0, 0, VoxelType::Wood), (4, 0, 1, VoxelType::Wood), (4, 0, 2, VoxelType::Wood),
            (5, 0, 0, VoxelType::Wood), (5, 0, 1, VoxelType::Wood), (5, 0, 2, VoxelType::Wood),
            (6, 0, 0, VoxelType::Wood), (6, 0, 1, VoxelType::Wood), (6, 0, 2, VoxelType::Wood),

            // Side railings
            (0, 1, 0, VoxelType::Wood), (0, 2, 0, VoxelType::Wood),
            (0, 1, 2, VoxelType::Wood), (0, 2, 2, VoxelType::Wood),
            (6, 1, 0, VoxelType::Wood), (6, 2, 0, VoxelType::Wood),
            (6, 1, 2, VoxelType::Wood), (6, 2, 2, VoxelType::Wood),
        ];

        let bridge = Template {
            name: "Simple Bridge".to_string(),
            description: "A wooden bridge with railings".to_string(),
            voxels: bridge_voxels,
            size: (7, 3, 3),
        };

        // Tower template
        let tower_voxels = vec![
            // Base (3x3, height 8)
            (0, 0, 0, VoxelType::Stone), (1, 0, 0, VoxelType::Stone), (2, 0, 0, VoxelType::Stone),
            (0, 0, 1, VoxelType::Stone), (2, 0, 1, VoxelType::Stone), // Hollow center
            (0, 0, 2, VoxelType::Stone), (1, 0, 2, VoxelType::Stone), (2, 0, 2, VoxelType::Stone),

            (0, 1, 0, VoxelType::Stone), (1, 1, 0, VoxelType::Stone), (2, 1, 0, VoxelType::Stone),
            (0, 1, 1, VoxelType::Stone), (2, 1, 1, VoxelType::Stone),
            (0, 1, 2, VoxelType::Stone), (1, 1, 2, VoxelType::Stone), (2, 1, 2, VoxelType::Stone),

            (0, 2, 0, VoxelType::Stone), (1, 2, 0, VoxelType::Stone), (2, 2, 0, VoxelType::Stone),
            (0, 2, 1, VoxelType::Stone), (2, 2, 1, VoxelType::Stone),
            (0, 2, 2, VoxelType::Stone), (1, 2, 2, VoxelType::Stone), (2, 2, 2, VoxelType::Stone),

            (0, 3, 0, VoxelType::Stone), (1, 3, 0, VoxelType::Stone), (2, 3, 0, VoxelType::Stone),
            (0, 3, 1, VoxelType::Stone), (2, 3, 1, VoxelType::Stone),
            (0, 3, 2, VoxelType::Stone), (1, 3, 2, VoxelType::Stone), (2, 3, 2, VoxelType::Stone),

            (0, 4, 0, VoxelType::Stone), (1, 4, 0, VoxelType::Stone), (2, 4, 0, VoxelType::Stone),
            (0, 4, 1, VoxelType::Stone), (2, 4, 1, VoxelType::Stone),
            (0, 4, 2, VoxelType::Stone), (1, 4, 2, VoxelType::Stone), (2, 4, 2, VoxelType::Stone),

            (0, 5, 0, VoxelType::Stone), (1, 5, 0, VoxelType::Stone), (2, 5, 0, VoxelType::Stone),
            (0, 5, 1, VoxelType::Stone), (2, 5, 1, VoxelType::Stone),
            (0, 5, 2, VoxelType::Stone), (1, 5, 2, VoxelType::Stone), (2, 5, 2, VoxelType::Stone),

            (0, 6, 0, VoxelType::Stone), (1, 6, 0, VoxelType::Stone), (2, 6, 0, VoxelType::Stone),
            (0, 6, 1, VoxelType::Stone), (2, 6, 1, VoxelType::Stone),
            (0, 6, 2, VoxelType::Stone), (1, 6, 2, VoxelType::Stone), (2, 6, 2, VoxelType::Stone),

            (0, 7, 0, VoxelType::Stone), (1, 7, 0, VoxelType::Stone), (2, 7, 0, VoxelType::Stone),
            (0, 7, 1, VoxelType::Stone), (2, 7, 1, VoxelType::Stone),
            (0, 7, 2, VoxelType::Stone), (1, 7, 2, VoxelType::Stone), (2, 7, 2, VoxelType::Stone),

            // Top platform
            (0, 8, 0, VoxelType::Stone), (1, 8, 0, VoxelType::Stone), (2, 8, 0, VoxelType::Stone),
            (0, 8, 1, VoxelType::Stone), (1, 8, 1, VoxelType::Stone), (2, 8, 1, VoxelType::Stone),
            (0, 8, 2, VoxelType::Stone), (1, 8, 2, VoxelType::Stone), (2, 8, 2, VoxelType::Stone),
        ];

        let tower = Template {
            name: "Watch Tower".to_string(),
            description: "A tall stone tower with hollow interior".to_string(),
            voxels: tower_voxels,
            size: (3, 9, 3),
        };

        templates.insert("house".to_string(), house);
        templates.insert("bridge".to_string(), bridge);
        templates.insert("tower".to_string(), tower);

        Self {
            templates,
            current_template: Some("house".to_string()),
            current_rotation: 0,
            preview_active: false,
            preview_position: None,
            preview_voxels: Vec::new(),
        }
    }

    fn get_current_template(&self) -> Option<&Template> {
        if let Some(template_name) = &self.current_template {
            self.templates.get(template_name)
        } else {
            None
        }
    }

    fn cycle_template(&mut self) {
        let template_names: Vec<String> = self.templates.keys().cloned().collect();
        if template_names.is_empty() {
            return;
        }

        let current_index = if let Some(current) = &self.current_template {
            template_names.iter().position(|name| name == current).unwrap_or(0)
        } else {
            0
        };

        let next_index = (current_index + 1) % template_names.len();
        self.current_template = Some(template_names[next_index].clone());
    }

    fn rotate_template(&mut self) {
        self.current_rotation = (self.current_rotation + 1) % 4;
    }

    fn apply_rotation(&self, voxel: (i32, i32, i32, VoxelType)) -> (i32, i32, i32, VoxelType) {
        let (x, y, z, material) = voxel;
        match self.current_rotation {
            0 => (x, y, z, material), // No rotation
            1 => (-z, y, x, material), // 90 degrees
            2 => (-x, y, -z, material), // 180 degrees
            3 => (z, y, -x, material), // 270 degrees
            _ => (x, y, z, material),
        }
    }
}

impl Default for WallConstructionState {
    fn default() -> Self {
        Self {
            is_constructing: false,
            start_position: None,
            current_position: None,
            preview_positions: Vec::new(),
        }
    }
}

impl RobinApp {
    async fn new(window: Arc<Window>) -> RobinResult<Self> {
        let size = window.inner_size();

        // Create wgpu surface
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone())
            .map_err(|e| RobinError::GraphicsInitError(format!("Failed to create surface: {}", e)))?;

        // Request adapter
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.ok_or_else(|| RobinError::GraphicsInitError("Failed to find adapter".to_string()))?;

        // Request device and queue
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
            },
            None,
        ).await.map_err(|e| RobinError::GraphicsInitError(format!("Failed to create device: {}", e)))?;

        // Configure surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        // Create render pipeline
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Voxel Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/voxel.wgsl").into()),
        });

        // Create uniform buffer
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: std::mem::size_of::<Uniforms>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group layout with uniforms, texture atlas, and sampler
        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Uniform Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // Shadow maps for cascaded shadow mapping
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Depth,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Depth,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Depth,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                    count: None,
                },
            ],
        });

        // Create texture manager and voxel atlas
        let mut texture_manager = robin::engine::graphics::TextureManager::new(&device);

        // Create a simple programmatic 4x4 voxel texture atlas
        let atlas_size = 256; // 4x4 grid of 64x64 textures
        let cell_size = atlas_size / 4;
        let mut atlas_pixels = Vec::with_capacity((atlas_size * atlas_size * 4) as usize);

        for row in 0..4 {
            for cell_row in 0..cell_size {
                for col in 0..4 {
                    let color = match (col, row) {
                        (0, 0) => [139, 69, 19, 255],   // Dirt - brown
                        (1, 0) => [128, 128, 128, 255], // Stone - gray
                        (2, 0) => [34, 139, 34, 255],   // Grass - green
                        (3, 0) => [255, 215, 0, 255],   // Sand - yellow
                        (0, 1) => [70, 130, 180, 255],  // Water - blue
                        (1, 1) => [220, 20, 60, 255],   // Lava - red
                        (2, 1) => [148, 0, 211, 255],   // Crystal - purple
                        (3, 1) => [255, 165, 0, 255],   // Crystal2 - orange
                        _ => [200, 200, 200, 255], // Default - light gray
                    };

                    for _ in 0..cell_size {
                        atlas_pixels.extend_from_slice(&color);
                    }
                }
            }
        }

        // Create the atlas texture
        let atlas_texture = robin::engine::graphics::Texture::create_solid_color(
            &device,
            &queue,
            [255, 255, 255, 255], // This will be overwritten
            (atlas_size, atlas_size),
            Some("voxel_atlas"),
        );

        // Write the actual atlas data
        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &atlas_texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &atlas_pixels,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * atlas_size),
                rows_per_image: Some(atlas_size),
            },
            wgpu::Extent3d {
                width: atlas_size,
                height: atlas_size,
                depth_or_array_layers: 1,
            },
        );

        // Create shadow mapping resources first (needed for bind group)
        let shadow_map_size = 2048u32;

        // Create shadow maps (3 cascades)
        let shadow_maps = [
            Self::create_shadow_map(&device, shadow_map_size),
            Self::create_shadow_map(&device, shadow_map_size),
            Self::create_shadow_map(&device, shadow_map_size),
        ];

        let shadow_map_views = [
            shadow_maps[0].create_view(&wgpu::TextureViewDescriptor::default()),
            shadow_maps[1].create_view(&wgpu::TextureViewDescriptor::default()),
            shadow_maps[2].create_view(&wgpu::TextureViewDescriptor::default()),
        ];

        // Create shadow sampler
        let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Shadow Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        // Create bind group with uniforms, texture, and sampler
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniform Bind Group"),
            layout: &uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&atlas_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&atlas_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&shadow_map_views[0]),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&shadow_map_views[1]),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(&shadow_map_views[2]),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::Sampler(&shadow_sampler),
                },
            ],
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create depth texture
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create render pipeline
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Voxel Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<robin::engine::graphics::Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x3, // position
                        },
                        wgpu::VertexAttribute {
                            offset: (std::mem::size_of::<[f32; 3]>() * 2 + std::mem::size_of::<[f32; 2]>()) as wgpu::BufferAddress,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x3, // color (location 1 in shader)
                        },
                        wgpu::VertexAttribute {
                            offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                            shader_location: 2,
                            format: wgpu::VertexFormat::Float32x3, // normal (location 2 in shader)
                        },
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // Create wireframe shader and pipeline for highlighting
        let wireframe_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Wireframe Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/wireframe.wgsl").into()),
        });

        let wireframe_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Wireframe Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &wireframe_shader,
                entry_point: "vs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: (std::mem::size_of::<[f32; 3]>() * 2) as wgpu::BufferAddress, // position + color
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x3, // position
                        },
                        wgpu::VertexAttribute {
                            offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x3, // color
                        },
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &wireframe_shader,
                entry_point: "fs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList, // Lines instead of triangles
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // Don't cull lines
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false, // Don't write depth for wireframes
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // Create wireframe vertex buffer (will be updated each frame)
        let wireframe_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Wireframe Vertex Buffer"),
            size: (std::mem::size_of::<[f32; 6]>() * 48) as wgpu::BufferAddress, // 48 vertices for two cube wireframes
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create UI shader and pipeline for overlay rendering
        let ui_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("UI Shader"),
            source: wgpu::ShaderSource::Wgsl(r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 0.0, 1.0);
    out.color = model.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 0.8); // Semi-transparent UI elements
}
"#.into()),
        });

        let ui_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("UI Render Pipeline"),
            layout: None, // Simple pipeline without bind groups
            vertex: wgpu::VertexState {
                module: &ui_shader,
                entry_point: "vs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: (std::mem::size_of::<[f32; 2]>() + std::mem::size_of::<[f32; 3]>()) as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x2, // position
                        },
                        wgpu::VertexAttribute {
                            offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x3, // color
                        },
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &ui_shader,
                entry_point: "fs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // Don't cull UI
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None, // UI doesn't need depth testing
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // Initialize Robin engine systems
        let mut camera = Camera::new(size.width as f32 / size.height as f32);
        // Position camera above ground level to see terrain (ground is at y=8-15)
        camera.position = Vec3::new(0.0, 20.0, 10.0);
        let input_manager = InputManager::new();
        let voxel_world = VoxelWorld::new("demo_world".to_string(), (64, 64, 64));
        let build_system = EngineerBuildMode::new();
        let ui_system = UIManager::new(size.width as f32, size.height as f32);

        // Create chunk manager and initialize world
        let mut chunk_manager = ChunkManager::new();
        chunk_manager.update_camera_position(camera.position);

        // Create initial world mesh from chunks
        let mut world_mesh = Mesh::default();
        Self::generate_world_mesh_from_chunks(&chunk_manager, &mut world_mesh, &camera);

        // Create shadow uniform buffer
        let shadow_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shadow Uniform Buffer"),
            size: std::mem::size_of::<[[f32; 4]; 4]>() as wgpu::BufferAddress, // One 4x4 matrix
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create shadow bind group layout
        let shadow_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Shadow Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // Create shadow bind group
        let shadow_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Shadow Bind Group"),
            layout: &shadow_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: shadow_uniform_buffer.as_entire_binding(),
                },
            ],
        });

        // Create shadow pipeline
        let shadow_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shadow Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/shadow.wgsl").into()),
        });

        let shadow_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Shadow Pipeline Layout"),
            bind_group_layouts: &[&shadow_bind_group_layout],
            push_constant_ranges: &[],
        });

        let shadow_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Shadow Pipeline"),
            layout: Some(&shadow_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shadow_shader,
                entry_point: "vs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: wgpu::VertexFormat::Float32x3,
                    }],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shadow_shader,
                entry_point: "fs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let mut app = Self {
            window,
            surface,
            surface_config,
            device,
            queue,
            // Rendering pipeline
            render_pipeline,
            uniform_buffer,
            uniform_bind_group,
            depth_texture,
            depth_texture_view,
            // Shadow mapping
            shadow_map_size,
            shadow_maps,
            shadow_map_views,
            shadow_sampler,
            shadow_bind_group,
            shadow_pipeline,
            shadow_uniform_buffer,
            shadow_bind_group_layout,
            // Mesh buffers (will be created when mesh is generated)
            vertex_buffer: None,
            index_buffer: None,
            // Wireframe highlighting
            wireframe_pipeline,
            wireframe_vertex_buffer,
            target_highlight_visible: false,
            placement_highlight_visible: false,
            target_highlight_position: Vec3::new(0.0, 0.0, 0.0),
            placement_highlight_position: Vec3::new(0.0, 0.0, 0.0),
            // UI Rendering
            ui_pipeline,
            ui_vertex_buffer: None,
            ui_index_buffer: None,
            // Engine systems
            camera,
            input_manager,
            voxel_world,
            build_system,
            ui_system,
            world_mesh,
            last_frame_time: Instant::now(),
            frame_count: 0,
            fps_timer: Instant::now(),
            // Interaction system
            raycast_result: RaycastResult::default(),
            // Chunk-based voxel world
            chunk_manager,
            // Texture management
            texture_manager,
            // Physics system
            physics_world: PhysicsWorld3D::new(Physics3DConfig::default()),
            player_physics_handle: None,
            player_velocity: Vec3::new(0.0, 0.0, 0.0),
            player_grounded: false,
            // Wall construction state
            wall_construction: WallConstructionState::default(),
            // Template system
            template_system: TemplateSystem::new(),
            // Undo/Redo system
            undo_system: UndoRedoSystem::new(),
            // UI Overlay system
            ui_overlay: UIOverlay::new(),
            // Particle system for block interactions
            particle_system: ParticleSystem::new(1000), // Max 1000 particles
        };

        // Create player physics body
        app.create_player_physics()?;

        // Create GPU buffers for the initial mesh
        app.create_mesh_buffers();

        Ok(app)
    }

    /// Create the player physics body
    fn create_player_physics(&mut self) -> RobinResult<()> {
        // Create player physics body at the camera position
        let player_descriptor = BodyDescriptor::player_character(self.camera.position);
        let player_shape = ColliderShape3D::player_character();

        let handle = self.physics_world.create_body(
            player_descriptor,
            player_shape,
            Some("player".to_string()),
        )?;

        self.player_physics_handle = Some(handle);
        Ok(())
    }

    /// Update the physics world and player movement
    fn update_physics(&mut self, delta_time: f32) -> RobinResult<()> {
        // Step the physics world
        self.physics_world.step(delta_time)?;

        // Check for collision events to detect ground contact
        self.update_ground_detection();

        // Update camera position based on player physics body
        if let Some(handle) = self.player_physics_handle {
            if let Some(position) = self.physics_world.get_body_position(handle) {
                // Update camera position to match player physics body
                // Add slight offset for camera height (player is a capsule, camera is at eye level)
                let eye_height_offset = Vec3::new(0.0, 0.8, 0.0); // 0.8 units above center of capsule
                let old_target_offset = self.camera.target - self.camera.position;
                self.camera.position = position + eye_height_offset;
                self.camera.target = self.camera.position + old_target_offset;
            }
        }

        Ok(())
    }

    /// Update ground detection for jumping mechanics
    fn update_ground_detection(&mut self) {
        // Reset grounded state
        self.player_grounded = false;

        if let Some(player_handle) = self.player_physics_handle {
            // Check collision events to see if player is touching ground
            for collision_event in self.physics_world.collision_events() {
                if collision_event.handle1.0 == player_handle.0 || collision_event.handle2.0 == player_handle.0 {
                    // Player is in contact with something
                    if collision_event.started {
                        // Check if the collision normal indicates ground contact (upward normal)
                        if collision_event.contact_normal.y > 0.5 {
                            self.player_grounded = true;
                            break;
                        }
                    }
                }
            }

            // Alternative ground detection using raycast downward from player
            if !self.player_grounded {
                if let Some(player_pos) = self.physics_world.get_body_position(player_handle) {
                    let ray_origin = player_pos;
                    let ray_direction = Vec3::new(0.0, -1.0, 0.0); // Downward
                    let max_distance = 1.1; // Slightly more than capsule half-height

                    if let Some((_hit_handle, distance, _hit_point)) =
                        self.physics_world.raycast(ray_origin, ray_direction, max_distance) {
                        // If raycast hits something within reasonable distance, player is grounded
                        if distance <= 1.0 {
                            self.player_grounded = true;
                        }
                    }
                }
            }
        }
    }

    /// Create static physics bodies for voxel blocks in the world
    fn create_voxel_physics_bodies(&mut self) -> RobinResult<()> {
        // Create physics bodies for some basic terrain blocks
        // For now, create a simple ground plane of blocks
        for x in -10..10 {
            for z in -10..10 {
                for y in 0..3 {
                    let world_pos = Vec3::new(x as f32, y as f32, z as f32);
                    let block_descriptor = BodyDescriptor::static_block(world_pos);
                    let block_shape = ColliderShape3D::voxel_block();

                    self.physics_world.create_body(
                        block_descriptor,
                        block_shape,
                        Some(format!("block_{}_{}", x, z)),
                    )?;
                }
            }
        }

        println!("✅ Created {} static voxel blocks for physics", 20 * 20 * 3);
        Ok(())
    }

    fn create_shadow_map(device: &wgpu::Device, size: u32) -> wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Shadow Map"),
            size: wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
    }

    fn initialize_voxel_data(voxel_data: &mut HashMap<(i32, i32, i32), VoxelType>, camera: &Camera) {
        let camera_pos = camera.position;
        let render_distance = 16; // Larger initial area

        for x in -render_distance..render_distance {
            for z in -render_distance..render_distance {
                let world_x = (camera_pos.x as i32) + x;
                let world_z = (camera_pos.z as i32) + z;

                // Create simple procedural terrain (matching original logic)
                let height = if (world_x + world_z) % 4 == 0 { 2 } else { 1 };

                for y in 0..height {
                    let voxel_type = if y == height - 1 {
                        VoxelType::Grass
                    } else {
                        VoxelType::Stone
                    };
                    voxel_data.insert((world_x, y, world_z), voxel_type);
                }
            }
        }

        println!("Initialized voxel data: {} blocks", voxel_data.len());
    }

    fn generate_world_mesh_from_data(voxel_data: &HashMap<(i32, i32, i32), VoxelType>, mesh: &mut Mesh, camera: &Camera) {
        mesh.vertices.clear();
        mesh.indices.clear();

        let camera_pos = camera.position;
        let render_distance = 8;

        // Step 1: Generate all visible faces with culling
        let mut visible_faces = Vec::new();

        for x in -render_distance..render_distance {
            for z in -render_distance..render_distance {
                let world_x = (camera_pos.x as i32) + x;
                let world_z = (camera_pos.z as i32) + z;

                for y in 0..8 {
                    if let Some(&voxel_type) = voxel_data.get(&(world_x, y, world_z)) {
                        Self::collect_visible_faces(voxel_data, (world_x, y, world_z), voxel_type, &mut visible_faces);
                    }
                }
            }
        }

        // Step 2: Apply greedy meshing to combine adjacent faces
        let optimized_faces = Self::greedy_mesh_faces(visible_faces);

        // Step 3: Generate mesh from optimized faces
        Self::generate_mesh_from_faces(&optimized_faces, mesh);

        println!("Generated optimized mesh: {} vertices, {} indices (from {} faces)",
                mesh.vertices.len(), mesh.indices.len(), optimized_faces.len());
    }

    fn generate_world_mesh_from_chunks(chunk_manager: &ChunkManager, mesh: &mut Mesh, camera: &Camera) {
        mesh.vertices.clear();
        mesh.indices.clear();

        let camera_pos = camera.position;
        let camera_chunk_x = (camera_pos.x / CHUNK_SIZE_F) as i32;
        let camera_chunk_y = (camera_pos.y / CHUNK_SIZE_F) as i32;
        let camera_chunk_z = (camera_pos.z / CHUNK_SIZE_F) as i32;

        // Step 1: Collect all visible faces from loaded chunks
        let mut visible_faces = Vec::new();

        for ((chunk_x, chunk_y, chunk_z), chunk) in chunk_manager.get_loaded_chunks() {
            // Skip distant chunks for performance
            let distance = ((chunk_x - camera_chunk_x).abs().max((chunk_y - camera_chunk_y).abs()).max((chunk_z - camera_chunk_z).abs()));
            if distance > 4 { // Increased render distance for LOD
                continue;
            }

            // Check if chunk should be rendered based on LOD distance
            if !chunk.should_render(300.0) { // Max render distance
                continue;
            }

            // Generate faces for voxels in this chunk based on LOD level
            for (&(local_x, local_y, local_z), &voxel_type) in &chunk.voxels {
                // Apply LOD filtering - skip voxels based on LOD level
                if !chunk.lod_level.should_render_voxel(local_x, local_y, local_z) {
                    continue;
                }

                let world_pos = chunk_to_world_coords(*chunk_x, *chunk_y, *chunk_z, local_x, local_y, local_z);
                Self::collect_visible_faces_from_chunks(chunk_manager, world_pos, voxel_type, &mut visible_faces);
            }
        }

        // Step 2: Apply greedy meshing to combine adjacent faces
        let optimized_faces = Self::greedy_mesh_faces(visible_faces);

        // Step 3: Generate mesh from optimized faces
        Self::generate_mesh_from_faces(&optimized_faces, mesh);

        println!("Generated chunk-based mesh: {} vertices, {} indices (from {} faces, {} chunks loaded)",
                mesh.vertices.len(), mesh.indices.len(), optimized_faces.len(), chunk_manager.get_loaded_chunks().len());
    }

    fn collect_visible_faces_from_chunks(
        chunk_manager: &ChunkManager,
        position: (i32, i32, i32),
        voxel_type: VoxelType,
        visible_faces: &mut Vec<Face>
    ) {
        // Check all 6 faces of the voxel
        let face_directions = [
            FaceDirection::PosX, FaceDirection::NegX,
            FaceDirection::PosY, FaceDirection::NegY,
            FaceDirection::PosZ, FaceDirection::NegZ,
        ];

        for direction in face_directions {
            let face = Face::new(position, direction, voxel_type);
            let adjacent_pos = face.get_adjacent_voxel_pos();

            // Face is visible if adjacent position has no voxel
            if chunk_manager.get_voxel(adjacent_pos.0, adjacent_pos.1, adjacent_pos.2).is_none() {
                visible_faces.push(face);
            }
        }
    }

    fn create_mesh_buffers(&mut self) {
        if !self.world_mesh.vertices.is_empty() && !self.world_mesh.indices.is_empty() {
            // Create vertex buffer
            self.vertex_buffer = Some(self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&self.world_mesh.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }));

            // Create index buffer
            self.index_buffer = Some(self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&self.world_mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            }));

            println!("Created GPU buffers for {} vertices, {} indices",
                    self.world_mesh.vertices.len(), self.world_mesh.indices.len());
        }
    }


    fn collect_visible_faces(
        voxel_data: &HashMap<(i32, i32, i32), VoxelType>,
        position: (i32, i32, i32),
        voxel_type: VoxelType,
        visible_faces: &mut Vec<Face>
    ) {
        // Check all 6 faces of the voxel
        let face_directions = [
            FaceDirection::PosX, FaceDirection::NegX,
            FaceDirection::PosY, FaceDirection::NegY,
            FaceDirection::PosZ, FaceDirection::NegZ,
        ];

        for direction in face_directions {
            let face = Face::new(position, direction, voxel_type);
            let adjacent_pos = face.get_adjacent_voxel_pos();

            // Face is visible if adjacent position is empty (not in voxel_data)
            if !voxel_data.contains_key(&adjacent_pos) {
                visible_faces.push(face);
            }
        }
    }

    fn greedy_mesh_faces(faces: Vec<Face>) -> Vec<Face> {
        let mut optimized_faces = Vec::new();
        let original_face_count = faces.len();

        // Group faces by direction first for efficiency
        let mut faces_by_direction = std::collections::HashMap::new();
        for face in faces {
            faces_by_direction.entry(face.direction).or_insert(Vec::new()).push(face);
        }

        // Process each direction separately
        for (_direction, mut direction_faces) in faces_by_direction {
            // Sort faces for consistent processing
            direction_faces.sort_by_key(|f| (f.position.0, f.position.1, f.position.2));

            let mut used = vec![false; direction_faces.len()];

            for i in 0..direction_faces.len() {
                if used[i] { continue; }

                let mut current_face = direction_faces[i].clone();
                used[i] = true;

                // Try to grow the face horizontally first
                current_face = Self::grow_face_horizontally(&direction_faces, &mut used, current_face);

                // Then try to grow vertically
                current_face = Self::grow_face_vertically(&direction_faces, &mut used, current_face);

                optimized_faces.push(current_face);
            }
        }

        println!("Greedy meshing: {} original faces -> {} optimized faces ({:.1}% reduction)",
                original_face_count, optimized_faces.len(),
                (1.0 - optimized_faces.len() as f32 / original_face_count.max(1) as f32) * 100.0);

        optimized_faces
    }

    fn grow_face_horizontally(faces: &[Face], used: &mut [bool], mut face: Face) -> Face {
        // This is a simplified horizontal growth - in a full implementation,
        // we'd need more sophisticated 2D greedy meshing based on the face direction
        let can_grow = |test_pos: (i32, i32, i32)| {
            faces.iter().enumerate().any(|(idx, f)| {
                !used[idx] &&
                f.position == test_pos &&
                f.direction == face.direction &&
                f.material == face.material
            })
        };

        // Try to grow in the primary horizontal direction for this face type
        let (dx, dy, dz) = match face.direction {
            FaceDirection::PosY | FaceDirection::NegY => (1, 0, 0), // Grow along X
            FaceDirection::PosX | FaceDirection::NegX => (0, 0, 1), // Grow along Z
            FaceDirection::PosZ | FaceDirection::NegZ => (1, 0, 0), // Grow along X
        };

        let mut growth = 0;
        loop {
            let test_pos = (
                face.position.0 + dx * (growth + 1),
                face.position.1 + dy * (growth + 1),
                face.position.2 + dz * (growth + 1)
            );

            if let Some((idx, _)) = faces.iter().enumerate().find(|(idx, f)| {
                !used[*idx] && f.position == test_pos && f.direction == face.direction && f.material == face.material
            }) {
                used[idx] = true;
                growth += 1;
                face.width += 1;
            } else {
                break;
            }
        }

        face
    }

    fn grow_face_vertically(faces: &[Face], used: &mut [bool], mut face: Face) -> Face {
        // Simplified vertical growth
        let (dx, dy, dz) = match face.direction {
            FaceDirection::PosY | FaceDirection::NegY => (0, 0, 1), // Grow along Z
            FaceDirection::PosX | FaceDirection::NegX => (0, 1, 0), // Grow along Y
            FaceDirection::PosZ | FaceDirection::NegZ => (0, 1, 0), // Grow along Y
        };

        let mut growth = 0;
        'outer: loop {
            // Check if entire width can grow vertically
            for w in 0..face.width {
                let test_pos = (
                    face.position.0 + dx * (growth + 1) + if matches!(face.direction, FaceDirection::PosY | FaceDirection::NegY | FaceDirection::PosZ | FaceDirection::NegZ) { w as i32 } else { 0 },
                    face.position.1 + dy * (growth + 1) + if matches!(face.direction, FaceDirection::PosX | FaceDirection::NegX) { w as i32 } else { 0 },
                    face.position.2 + dz * (growth + 1) + if matches!(face.direction, FaceDirection::PosX | FaceDirection::NegX) { w as i32 } else { 0 }
                );

                if !faces.iter().enumerate().any(|(idx, f)| {
                    !used[idx] && f.position == test_pos && f.direction == face.direction && f.material == face.material
                }) {
                    break 'outer;
                }
            }

            // Mark all faces in this row as used
            for w in 0..face.width {
                let test_pos = (
                    face.position.0 + dx * (growth + 1) + if matches!(face.direction, FaceDirection::PosY | FaceDirection::NegY | FaceDirection::PosZ | FaceDirection::NegZ) { w as i32 } else { 0 },
                    face.position.1 + dy * (growth + 1) + if matches!(face.direction, FaceDirection::PosX | FaceDirection::NegX) { w as i32 } else { 0 },
                    face.position.2 + dz * (growth + 1) + if matches!(face.direction, FaceDirection::PosX | FaceDirection::NegX) { w as i32 } else { 0 }
                );

                if let Some((idx, _)) = faces.iter().enumerate().find(|(idx, f)| {
                    !used[*idx] && f.position == test_pos && f.direction == face.direction && f.material == face.material
                }) {
                    used[idx] = true;
                }
            }

            growth += 1;
            face.height += 1;
        }

        face
    }

    fn generate_mesh_from_faces(faces: &[Face], mesh: &mut Mesh) {
        for face in faces {
            Self::add_face_to_mesh(mesh, face);
        }
    }

    fn add_face_to_mesh(mesh: &mut Mesh, face: &Face) {
        let (x, y, z) = (face.position.0 as f32, face.position.1 as f32, face.position.2 as f32);
        let color = face.material.get_color();
        let normal = face.get_normal();

        // Get texture coordinates for this face
        let face_normal = match face.direction {
            FaceDirection::PosZ => [0.0, 0.0, 1.0],    // North
            FaceDirection::NegZ => [0.0, 0.0, -1.0],   // South
            FaceDirection::NegX => [-1.0, 0.0, 0.0],   // West
            FaceDirection::PosX => [1.0, 0.0, 0.0],    // East
            FaceDirection::PosY => [0.0, 1.0, 0.0],    // Up
            FaceDirection::NegY => [0.0, -1.0, 0.0],   // Down
        };
        let face_uvs = face.material.get_face_texture_coords(face_normal);

        // Calculate vertices based on face direction and size
        let vertices = match face.direction {
            FaceDirection::PosY => { // Top face
                [
                    [x, y + 1.0, z],
                    [x + face.width as f32, y + 1.0, z],
                    [x + face.width as f32, y + 1.0, z + face.height as f32],
                    [x, y + 1.0, z + face.height as f32],
                ]
            },
            FaceDirection::NegY => { // Bottom face
                [
                    [x, y, z + face.height as f32],
                    [x + face.width as f32, y, z + face.height as f32],
                    [x + face.width as f32, y, z],
                    [x, y, z],
                ]
            },
            FaceDirection::PosX => { // Right face
                [
                    [x + 1.0, y, z],
                    [x + 1.0, y, z + face.width as f32],
                    [x + 1.0, y + face.height as f32, z + face.width as f32],
                    [x + 1.0, y + face.height as f32, z],
                ]
            },
            FaceDirection::NegX => { // Left face
                [
                    [x, y, z + face.width as f32],
                    [x, y, z],
                    [x, y + face.height as f32, z],
                    [x, y + face.height as f32, z + face.width as f32],
                ]
            },
            FaceDirection::PosZ => { // Front face
                [
                    [x, y, z + 1.0],
                    [x + face.width as f32, y, z + 1.0],
                    [x + face.width as f32, y + face.height as f32, z + 1.0],
                    [x, y + face.height as f32, z + 1.0],
                ]
            },
            FaceDirection::NegZ => { // Back face
                [
                    [x + face.width as f32, y, z],
                    [x, y, z],
                    [x, y + face.height as f32, z],
                    [x + face.width as f32, y + face.height as f32, z],
                ]
            },
        };

        let base_index = mesh.vertices.len() as u32;

        // Add vertices with proper UV coordinates and PBR properties
        for (i, vertex_pos) in vertices.iter().enumerate() {
            // Encode PBR properties in vertex color channels:
            // r = base color red, g = roughness, b = metallic, a = alpha
            let roughness = face.material.get_roughness();
            let metallic = face.material.get_metallic();
            mesh.vertices.push(robin::engine::graphics::Vertex {
                position: (*vertex_pos).into(),
                normal: normal.into(),
                uv: face_uvs[i].into(),
                color: [color[0], roughness, metallic, 1.0],
            });
        }

        // Add triangles (two per face)
        mesh.indices.extend_from_slice(&[
            base_index, base_index + 1, base_index + 2,
            base_index, base_index + 2, base_index + 3,
        ]);
    }

    fn add_voxel_to_mesh(mesh: &mut Mesh, pos: (i32, i32, i32), voxel_type: VoxelType) {
        let (x, y, z) = (pos.0 as f32, pos.1 as f32, pos.2 as f32);
        let color = voxel_type.get_color();

        // Define 8 vertices of a cube
        let vertices = [
            [x,     y,     z    ], // 0: Bottom-left-front
            [x + 1.0, y,     z    ], // 1: Bottom-right-front
            [x + 1.0, y + 1.0, z    ], // 2: Top-right-front
            [x,     y + 1.0, z    ], // 3: Top-left-front
            [x,     y,     z + 1.0], // 4: Bottom-left-back
            [x + 1.0, y,     z + 1.0], // 5: Bottom-right-back
            [x + 1.0, y + 1.0, z + 1.0], // 6: Top-right-back
            [x,     y + 1.0, z + 1.0], // 7: Top-left-back
        ];

        // Define 6 faces with their vertices and normals
        let faces = [
            // Front face (+Z)
            ([0, 1, 2, 3], [0.0, 0.0, 1.0]),
            // Back face (-Z)
            ([5, 4, 7, 6], [0.0, 0.0, -1.0]),
            // Right face (+X)
            ([1, 5, 6, 2], [1.0, 0.0, 0.0]),
            // Left face (-X)
            ([4, 0, 3, 7], [-1.0, 0.0, 0.0]),
            // Top face (+Y)
            ([3, 2, 6, 7], [0.0, 1.0, 0.0]),
            // Bottom face (-Y)
            ([4, 5, 1, 0], [0.0, -1.0, 0.0]),
        ];

        let base_index = mesh.vertices.len() as u32;

        for (face_index, (face_vertices, normal)) in faces.iter().enumerate() {
            // Add vertices for this face with PBR properties
            for &vertex_idx in face_vertices {
                let pos = vertices[vertex_idx];
                // Encode PBR properties in vertex color channels:
                // r = base color red, g = roughness, b = metallic, a = alpha
                let roughness = voxel_type.get_roughness();
                let metallic = voxel_type.get_metallic();
                mesh.vertices.push(robin::engine::graphics::Vertex {
                    position: pos.into(),
                    normal: (*normal).into(),
                    uv: [0.0, 0.0].into(),
                    color: [color[0], roughness, metallic, 1.0],
                });
            }

            // Add triangles (two per face) - each face has 4 vertices
            let face_base = base_index + (face_index * 4) as u32;
            mesh.indices.extend_from_slice(&[
                face_base, face_base + 1, face_base + 2,
                face_base, face_base + 2, face_base + 3,
            ]);
        }
    }

    fn raycast_from_camera(&self) -> RaycastResult {
        let max_distance = 10.0;
        let step_size = 0.1;

        // Calculate camera forward direction
        let forward = (self.camera.target - self.camera.position).normalize();

        // Cast ray from camera position
        let mut current_pos = self.camera.position;
        let mut distance = 0.0;

        while distance < max_distance {
            // Check if there's a voxel at this position
            let world_x = current_pos.x.floor() as i32;
            let world_y = current_pos.y.floor() as i32;
            let world_z = current_pos.z.floor() as i32;

            // Check if a voxel exists at this position in our chunk system
            if self.chunk_manager.get_voxel(world_x, world_y, world_z).is_some() {
                // Calculate face normal based on entry direction
                let prev_pos = current_pos - forward * step_size;
                let prev_x = prev_pos.x.floor() as i32;
                let prev_y = prev_pos.y.floor() as i32;
                let prev_z = prev_pos.z.floor() as i32;

                let face_normal = if prev_x != world_x {
                    if prev_x < world_x { Vec3::new(-1.0, 0.0, 0.0) } else { Vec3::new(1.0, 0.0, 0.0) }
                } else if prev_y != world_y {
                    if prev_y < world_y { Vec3::new(0.0, -1.0, 0.0) } else { Vec3::new(0.0, 1.0, 0.0) }
                } else if prev_z != world_z {
                    if prev_z < world_z { Vec3::new(0.0, 0.0, -1.0) } else { Vec3::new(0.0, 0.0, 1.0) }
                } else {
                    Vec3::new(0.0, 1.0, 0.0) // Default to top face
                };

                return RaycastResult {
                    hit: true,
                    position: current_pos,
                    block_position: (world_x, world_y, world_z),
                    face_normal,
                    distance,
                };
            }

            current_pos += forward * step_size;
            distance += step_size;
        }

        RaycastResult::default()
    }

    fn update(&mut self, delta_time: f32) -> RobinResult<()> {
        // Reset frame state
        self.input_manager.reset_frame();

        // Update camera based on input
        self.update_camera(delta_time);

        // Update physics world
        self.update_physics(delta_time)?;

        // Update raycast result based on current camera position
        self.raycast_result = self.raycast_from_camera();

        // Debug output for raycast (first few frames only)
        if self.frame_count < 3 && self.raycast_result.hit {
            println!("Raycast hit block at {:?} (distance: {:.2})",
                    self.raycast_result.block_position, self.raycast_result.distance);
        }

        // Update wireframe highlights based on raycast
        self.update_wireframe_highlights();

        // Update build system
        self.build_system.update(delta_time, &self.input_manager)?;

        // Update UI system
        self.ui_system.update(delta_time, &self.input_manager);

        // Update UI overlay system
        self.ui_overlay.update(delta_time);

        // Update particle system
        self.particle_system.update(delta_time);

        // Update UI elements for graphics rendering - extract values to avoid borrowing issues
        let current_mode = self.build_system.get_mode();
        let current_material = VoxelType::Stone; // TODO: implement material system in EngineerBuildMode
        let template_info = self.template_system.get_current_template().map(|t| (t.name.clone(), self.template_system.current_rotation));
        let can_undo = self.undo_system.can_undo();
        let can_redo = self.undo_system.can_redo();

        // Now update UI with extracted values
        self.ui_overlay.update_ui_elements_extracted(current_mode, current_material, template_info, can_undo, can_redo);

        // Create/update UI mesh buffers
        self.create_ui_mesh_buffers();

        // Update chunk loading and regenerate world mesh if camera moved significantly
        static mut LAST_CAMERA_POS: Vec3 = Vec3::new(0.0, 0.0, 0.0);
        let current_pos = self.camera.position;
        unsafe {
            // Update chunk manager every frame for background loading
            self.chunk_manager.update_camera_position(current_pos);

            // Check if any chunks need mesh regeneration
            let chunks_need_update = self.chunk_manager.get_loaded_chunks()
                .values()
                .any(|chunk| chunk.mesh_dirty);

            let distance = (current_pos - LAST_CAMERA_POS).magnitude();

            // Regenerate mesh if camera moved significantly OR new chunks were loaded
            if distance > 10.0 || chunks_need_update {
                Self::generate_world_mesh_from_chunks(&self.chunk_manager, &mut self.world_mesh, &self.camera);
                self.create_mesh_buffers(); // Recreate GPU buffers with new mesh data

                // Mark all chunks as clean
                for chunk in self.chunk_manager.get_loaded_chunks_mut().values_mut() {
                    chunk.mesh_dirty = false;
                }

                if distance > 10.0 {
                    LAST_CAMERA_POS = current_pos;
                }

                // Print performance stats periodically
                if self.frame_count % 60 == 0 { // Every ~1 second at 60 FPS
                    self.chunk_manager.print_performance_stats();

                    // Print LOD statistics
                    let (high, medium, low, very_low) = self.chunk_manager.get_lod_stats();
                    println!(
                        "🎯 LOD Stats: High: {}, Medium: {}, Low: {}, VeryLow: {}",
                        high, medium, low, very_low
                    );
                }
            }
        }

        Ok(())
    }

    fn update_camera(&mut self, delta_time: f32) {
        let move_force = 300.0; // Force to apply for movement (Newtons)
        let rotation_speed = 2.0 * delta_time;

        // Calculate camera direction vectors
        let forward = (self.camera.target - self.camera.position).normalize();
        let right = forward.cross(self.camera.up).normalize();

        // Apply physics-based movement forces to player body
        if let Some(handle) = self.player_physics_handle {
            let mut movement_force = Vec3::new(0.0, 0.0, 0.0);

            // Handle horizontal movement (WASD)
            if self.input_manager.is_key_pressed(&Key::Character("w".into())) {
                // Only apply horizontal component of forward vector (no flying)
                let horizontal_forward = Vec3::new(forward.x, 0.0, forward.z).normalize();
                movement_force += horizontal_forward * move_force;
            }
            if self.input_manager.is_key_pressed(&Key::Character("s".into())) {
                let horizontal_forward = Vec3::new(forward.x, 0.0, forward.z).normalize();
                movement_force -= horizontal_forward * move_force;
            }
            if self.input_manager.is_key_pressed(&Key::Character("a".into())) {
                movement_force -= right * move_force;
            }
            if self.input_manager.is_key_pressed(&Key::Character("d".into())) {
                movement_force += right * move_force;
            }

            // Apply the accumulated movement force
            if movement_force.magnitude() > 0.0 {
                if let Err(e) = self.physics_world.apply_force(handle, movement_force) {
                    println!("Warning: Failed to apply movement force: {:?}", e);
                }
            }

            // Handle jumping (Space) - only if grounded
            if self.input_manager.is_key_pressed(&Key::Named(NamedKey::Space)) && self.player_grounded {
                let jump_impulse = Vec3::new(0.0, 250.0, 0.0); // Upward impulse for jumping
                if let Err(e) = self.physics_world.apply_impulse(handle, jump_impulse) {
                    println!("Warning: Failed to apply jump impulse: {:?}", e);
                }
                self.player_grounded = false; // Player is now airborne
            }

            // Note: Removed Shift for downward movement - let gravity handle falling
        }

        // Handle rotation (Arrow keys) - this still directly affects camera orientation
        if self.input_manager.is_key_pressed(&Key::Named(NamedKey::ArrowLeft)) {
            let rotation = cgmath::Matrix3::from_angle_y(cgmath::Rad(rotation_speed));
            let direction = self.camera.target - self.camera.position;
            let new_direction = rotation * direction;
            self.camera.target = self.camera.position + new_direction;
        }
        if self.input_manager.is_key_pressed(&Key::Named(NamedKey::ArrowRight)) {
            let rotation = cgmath::Matrix3::from_angle_y(cgmath::Rad(-rotation_speed));
            let direction = self.camera.target - self.camera.position;
            let new_direction = rotation * direction;
            self.camera.target = self.camera.position + new_direction;
        }
        if self.input_manager.is_key_pressed(&Key::Named(NamedKey::ArrowUp)) {
            let right = forward.cross(self.camera.up).normalize();
            let rotation = cgmath::Matrix3::from_axis_angle(right, cgmath::Rad(rotation_speed));
            let direction = self.camera.target - self.camera.position;
            let new_direction = rotation * direction;
            self.camera.target = self.camera.position + new_direction;
        }
        if self.input_manager.is_key_pressed(&Key::Named(NamedKey::ArrowDown)) {
            let right = forward.cross(self.camera.up).normalize();
            let rotation = cgmath::Matrix3::from_axis_angle(right, cgmath::Rad(-rotation_speed));
            let direction = self.camera.target - self.camera.position;
            let new_direction = rotation * direction;
            self.camera.target = self.camera.position + new_direction;
        }
    }

    fn calculate_sun_position(&self, time: f32) -> [f32; 4] {
        // Day cycle duration: 24 seconds = 1 full day
        let day_cycle_duration = 24.0;
        let cycle_time = (time % day_cycle_duration) / day_cycle_duration;

        // Sun follows an arc across the sky
        let angle = cycle_time * 2.0 * std::f32::consts::PI - std::f32::consts::PI; // -π to π

        // Sun radius from center (distance)
        let radius = 50.0;

        // Calculate position: high at noon (angle=0), low at midnight (angle=π)
        let x = radius * angle.cos();
        let y = 20.0 + 30.0 * angle.sin().max(0.0); // Sun never goes below y=20
        let z = radius * angle.sin() * 0.3; // Slight arc variation

        [x, y, z, 1.0]
    }

    fn calculate_light_color(&self, time: f32) -> [f32; 4] {
        // Day cycle duration: 24 seconds = 1 full day
        let day_cycle_duration = 24.0;
        let cycle_time = (time % day_cycle_duration) / day_cycle_duration;

        // Calculate sun angle for lighting intensity
        let angle = cycle_time * 2.0 * std::f32::consts::PI - std::f32::consts::PI;
        let sun_height = (angle.sin() + 1.0) / 2.0; // 0 to 1

        // Define color phases
        let night_color = [0.1, 0.1, 0.3]; // Dark blue
        let dawn_color = [1.0, 0.4, 0.2];  // Orange
        let day_color = [1.0, 1.0, 0.9];   // Bright white-yellow
        let dusk_color = [1.0, 0.3, 0.1];  // Red-orange

        let color = if sun_height < 0.1 {
            // Night time
            night_color
        } else if sun_height < 0.3 {
            // Dawn transition
            let t = (sun_height - 0.1) / 0.2;
            [
                night_color[0] + t * (dawn_color[0] - night_color[0]),
                night_color[1] + t * (dawn_color[1] - night_color[1]),
                night_color[2] + t * (dawn_color[2] - night_color[2]),
            ]
        } else if sun_height < 0.7 {
            // Day time
            let t = (sun_height - 0.3) / 0.4;
            [
                dawn_color[0] + t * (day_color[0] - dawn_color[0]),
                dawn_color[1] + t * (day_color[1] - dawn_color[1]),
                dawn_color[2] + t * (day_color[2] - dawn_color[2]),
            ]
        } else if sun_height < 0.9 {
            // Dusk transition
            let t = (sun_height - 0.7) / 0.2;
            [
                day_color[0] + t * (dusk_color[0] - day_color[0]),
                day_color[1] + t * (dusk_color[1] - day_color[1]),
                day_color[2] + t * (dusk_color[2] - day_color[2]),
            ]
        } else {
            // Back to night
            let t = (sun_height - 0.9) / 0.1;
            [
                dusk_color[0] + t * (night_color[0] - dusk_color[0]),
                dusk_color[1] + t * (night_color[1] - dusk_color[1]),
                dusk_color[2] + t * (night_color[2] - dusk_color[2]),
            ]
        };

        // Apply intensity based on sun height
        let intensity = (sun_height * 2.0).min(1.0);
        [
            color[0] * intensity,
            color[1] * intensity,
            color[2] * intensity,
            1.0
        ]
    }

    fn render(&mut self) -> RobinResult<()> {
        let output = self.surface.get_current_texture()
            .map_err(|e| RobinError::RenderingError(format!("Failed to acquire surface texture: {}", e)))?;

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Update uniform buffer with camera data
        let view_matrix = self.camera.view_matrix();
        let proj_matrix = self.camera.projection_matrix();
        let view_proj_matrix = proj_matrix * view_matrix;

        let current_time = self.last_frame_time.elapsed().as_secs_f32();

        // Calculate light space matrices for cascaded shadow mapping
        let light_pos = self.calculate_sun_position(current_time);
        let light_dir = cgmath::Vector3::new(-light_pos[0], -light_pos[1], -light_pos[2]).normalize();
        let light_space_matrices = self.calculate_light_space_matrices(&view_matrix, &proj_matrix, light_dir);

        let uniforms = Uniforms {
            view_proj: view_proj_matrix.into(),
            view_pos: [self.camera.position.x, self.camera.position.y, self.camera.position.z, 1.0],
            light_pos,
            light_color: self.calculate_light_color(current_time),
            // Cascaded shadow maps with calculated light space matrices
            light_space_matrix_0: light_space_matrices[0].into(),
            light_space_matrix_1: light_space_matrices[1].into(),
            light_space_matrix_2: light_space_matrices[2].into(),
            cascade_splits: [10.0, 50.0, 200.0, 3.0], // Near, mid, far splits + cascade count
            shadow_bias: [0.005, 0.01, 2.0, 1.0], // bias, normal_bias, pcf_radius, enable_shadows
            time: current_time,
            _padding: [0.0; 3],
        };
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // Render shadow maps for each cascade
        self.render_shadow_maps(&mut encoder, &light_space_matrices)?;

        // Begin render pass with depth buffer
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Render voxel mesh if buffers exist
            if let (Some(vertex_buffer), Some(index_buffer)) = (&self.vertex_buffer, &self.index_buffer) {
                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..self.world_mesh.indices.len() as u32, 0, 0..1);

                // Debug output for first few frames
                if self.frame_count < 3 {
                    println!("Rendering {} indices from mesh", self.world_mesh.indices.len());
                }
            } else if self.frame_count < 3 {
                println!("No vertex/index buffers available for rendering");
            }

            // Render wireframe highlights
            if self.target_highlight_visible || self.placement_highlight_visible {
                render_pass.set_pipeline(&self.wireframe_pipeline);
                render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.wireframe_vertex_buffer.slice(..));

                let vertex_count = if self.target_highlight_visible && self.placement_highlight_visible {
                    48 // 24 vertices per cube * 2 cubes
                } else {
                    24 // 24 vertices for one cube
                };

                render_pass.draw(0..vertex_count, 0..1);
            }

            // Render particles as small colored points using wireframe pipeline
            self.render_particles(&mut render_pass);

            // Render UI overlay
            if let (Some(ui_vertex_buffer), Some(ui_index_buffer)) = (&self.ui_vertex_buffer, &self.ui_index_buffer) {
                if !self.ui_overlay.ui_elements.is_empty() {
                    render_pass.set_pipeline(&self.ui_pipeline);
                    render_pass.set_vertex_buffer(0, ui_vertex_buffer.slice(..));
                    render_pass.set_index_buffer(ui_index_buffer.slice(..), wgpu::IndexFormat::Uint32);

                    let indices_count = self.ui_overlay.ui_elements.len() * 6; // 6 indices per UI element
                    render_pass.draw_indexed(0..indices_count as u32, 0, 0..1);
                }
            }
        }

        // Submit the command buffer
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        // Update frame counter
        self.frame_count += 1;
        if self.fps_timer.elapsed() >= Duration::from_secs(1) {
            println!("FPS: {}", self.frame_count);
            self.frame_count = 0;
            self.fps_timer = Instant::now();
        }

        Ok(())
    }

    fn calculate_light_space_matrices(
        &self,
        view_matrix: &cgmath::Matrix4<f32>,
        proj_matrix: &cgmath::Matrix4<f32>,
        light_dir: cgmath::Vector3<f32>,
    ) -> [cgmath::Matrix4<f32>; 3] {
        let cascade_splits = [10.0, 50.0, 200.0];
        let mut light_space_matrices = [cgmath::Matrix4::identity(); 3];

        for (i, &far_distance) in cascade_splits.iter().enumerate() {
            let near_distance = if i == 0 { 0.1 } else { cascade_splits[i - 1] };

            // Calculate the frustum corners for this cascade
            let inv_view_proj = (proj_matrix * view_matrix).invert().unwrap();
            let mut frustum_corners = Vec::new();

            // Generate the 8 corners of the view frustum for this cascade
            for z in &[near_distance, far_distance] {
                let z_ndc = if *z == 0.1 { -1.0 } else { (*z - 0.1) / (200.0 - 0.1) * 2.0 - 1.0 };
                for y in &[-1.0, 1.0] {
                    for x in &[-1.0, 1.0] {
                        let corner_ndc = cgmath::Vector4::new(*x, *y, z_ndc, 1.0);
                        let corner_world = inv_view_proj * corner_ndc;
                        let corner_world = corner_world.truncate() / corner_world.w;
                        frustum_corners.push(corner_world);
                    }
                }
            }

            // Calculate the center of the frustum
            let center = frustum_corners.iter().sum::<cgmath::Vector3<f32>>() / frustum_corners.len() as f32;

            // Create light view matrix
            let light_pos = center - light_dir * 50.0; // Position light away from center
            let up = cgmath::Vector3::new(0.0, 1.0, 0.0);
            let light_view = cgmath::Matrix4::look_at_rh(
                cgmath::Point3::from_vec(light_pos),
                cgmath::Point3::from_vec(center),
                up,
            );

            // Transform frustum corners to light space to calculate bounds
            let mut min_x = f32::INFINITY;
            let mut max_x = f32::NEG_INFINITY;
            let mut min_y = f32::INFINITY;
            let mut max_y = f32::NEG_INFINITY;
            let mut min_z = f32::INFINITY;
            let mut max_z = f32::NEG_INFINITY;

            for corner in &frustum_corners {
                let corner_4d = cgmath::Vector4::new(corner.x, corner.y, corner.z, 1.0);
                let light_space_corner_4d = light_view * corner_4d;
                let light_space_corner = light_space_corner_4d.truncate();

                min_x = min_x.min(light_space_corner.x);
                max_x = max_x.max(light_space_corner.x);
                min_y = min_y.min(light_space_corner.y);
                max_y = max_y.max(light_space_corner.y);
                min_z = min_z.min(light_space_corner.z);
                max_z = max_z.max(light_space_corner.z);
            }

            // Expand bounds slightly to avoid edge artifacts
            let margin = 2.0;
            min_x -= margin;
            max_x += margin;
            min_y -= margin;
            max_y += margin;
            min_z -= margin;
            max_z += margin;

            // Create orthographic projection for shadow map
            let light_proj = cgmath::ortho(min_x, max_x, min_y, max_y, min_z, max_z);

            light_space_matrices[i] = light_proj * light_view;
        }

        light_space_matrices
    }

    fn render_particles(&mut self, render_pass: &mut wgpu::RenderPass) {
        let particles = self.particle_system.get_particles();
        if particles.is_empty() {
            return;
        }

        // Create particle vertex data on the fly (for simplicity)
        let mut particle_vertices = Vec::new();

        for particle in particles {
            // Create a small cube for each particle
            let size = particle.size * particle.life; // Scale by life for fade effect
            let pos = particle.position;
            let color = [
                particle.color[0],
                particle.color[1],
                particle.color[2],
            ];

            // Generate 8 vertices for a small cube around the particle position
            let half_size = size * 0.5;
            let cube_vertices = [
                // Bottom face
                [pos.x - half_size, pos.y - half_size, pos.z - half_size, color[0], color[1], color[2]],
                [pos.x + half_size, pos.y - half_size, pos.z - half_size, color[0], color[1], color[2]],
                [pos.x + half_size, pos.y - half_size, pos.z + half_size, color[0], color[1], color[2]],
                [pos.x - half_size, pos.y - half_size, pos.z + half_size, color[0], color[1], color[2]],
                // Top face
                [pos.x - half_size, pos.y + half_size, pos.z - half_size, color[0], color[1], color[2]],
                [pos.x + half_size, pos.y + half_size, pos.z - half_size, color[0], color[1], color[2]],
                [pos.x + half_size, pos.y + half_size, pos.z + half_size, color[0], color[1], color[2]],
                [pos.x - half_size, pos.y + half_size, pos.z + half_size, color[0], color[1], color[2]],
            ];

            // Add wireframe edges for the cube (12 edges = 24 vertices)
            let edges = [
                // Bottom face edges
                (0, 1), (1, 2), (2, 3), (3, 0),
                // Top face edges
                (4, 5), (5, 6), (6, 7), (7, 4),
                // Vertical edges
                (0, 4), (1, 5), (2, 6), (3, 7),
            ];

            for (start, end) in edges.iter() {
                particle_vertices.extend_from_slice(&cube_vertices[*start]);
                particle_vertices.extend_from_slice(&cube_vertices[*end]);
            }
        }

        // Only render if we have particles
        if !particle_vertices.is_empty() {
            // Create a temporary vertex buffer for particles
            let particle_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Particle Vertex Buffer"),
                contents: bytemuck::cast_slice(&particle_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

            render_pass.set_pipeline(&self.wireframe_pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.set_vertex_buffer(0, particle_buffer.slice(..));
            render_pass.draw(0..particle_vertices.len() as u32 / 6, 0..1); // 6 floats per vertex
        }
    }

    fn render_shadow_maps(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        light_space_matrices: &[cgmath::Matrix4<f32>; 3],
    ) -> RobinResult<()> {
        // Render each cascade shadow map
        for (i, &light_space_matrix) in light_space_matrices.iter().enumerate() {
            // Create shadow uniforms for this cascade
            let shadow_uniforms = ShadowUniforms {
                light_view_proj: light_space_matrix.into(),
            };

            // Update shadow uniform buffer
            self.queue.write_buffer(
                &self.shadow_uniform_buffer,
                0,
                bytemuck::cast_slice(&[shadow_uniforms])
            );

            // Select the appropriate shadow map view
            let shadow_view = match i {
                0 => &self.shadow_map_views[0],
                1 => &self.shadow_map_views[1],
                2 => &self.shadow_map_views[2],
                _ => continue,
            };

            // Shadow render pass
            {
                let mut shadow_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some(&format!("Shadow Pass {}", i)),
                    color_attachments: &[],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: shadow_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                // Render scene geometry for shadow map
                if let (Some(vertex_buffer), Some(index_buffer)) = (&self.vertex_buffer, &self.index_buffer) {
                    shadow_pass.set_pipeline(&self.shadow_pipeline);
                    shadow_pass.set_bind_group(0, &self.shadow_bind_group, &[]);
                    shadow_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    shadow_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    shadow_pass.draw_indexed(0..self.world_mesh.indices.len() as u32, 0, 0..1);
                }
            }
        }

        Ok(())
    }

    fn handle_window_event(&mut self, event: &WindowEvent) -> RobinResult<()> {
        match event {
            WindowEvent::Resized(physical_size) => {
                self.resize(*physical_size)?;
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.handle_keyboard_input(event);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.handle_mouse_input(*state, *button)?;
            }
            WindowEvent::CursorMoved { .. } => {
                self.handle_mouse_movement()?;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_mouse_movement(&mut self) -> RobinResult<()> {
        use robin::engine::build_mode::BuildModeState;

        match self.build_system.get_mode() {
            BuildModeState::Build => {
                // Update wall construction preview if actively constructing
                if self.wall_construction.is_constructing {
                    if self.raycast_result.hit {
                        let new_position = self.calculate_placement_position(&self.raycast_result);
                        self.wall_construction.current_position = Some(new_position);

                        // Update preview positions for visual feedback
                        if let Some(start_pos) = self.wall_construction.start_position {
                            self.wall_construction.preview_positions = self.calculate_wall_positions(start_pos, new_position);
                        }
                    }
                }
            }
            BuildModeState::Test => {
                // Update floor construction preview if actively constructing
                if self.wall_construction.is_constructing {
                    if self.raycast_result.hit {
                        let new_position = self.calculate_placement_position(&self.raycast_result);
                        self.wall_construction.current_position = Some(new_position);

                        // Update preview positions for visual feedback
                        if let Some(start_pos) = self.wall_construction.start_position {
                            self.wall_construction.preview_positions = self.calculate_floor_positions(start_pos, new_position);
                        }
                    }
                }
            }
            BuildModeState::Play => {
                // Update template preview position
                if self.raycast_result.hit {
                    let new_position = self.calculate_placement_position(&self.raycast_result);
                    self.template_system.preview_position = Some(new_position);
                    self.template_system.preview_active = true;
                    self.update_template_preview(new_position);
                } else {
                    self.template_system.preview_active = false;
                    self.template_system.preview_voxels.clear();
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_keyboard_input(&mut self, event: &KeyEvent) {
        self.input_manager.update_key(event.logical_key.clone(), event.state);

        // Handle build system controls
        if event.state == ElementState::Pressed {
            match &event.logical_key {
                // Undo/Redo controls with modifiers
                winit::keyboard::Key::Character(ref s) if s.len() == 1 => {
                    let ch = s.chars().next().unwrap();
                    let is_ctrl = self.input_manager.is_named_key_pressed(winit::keyboard::NamedKey::Control);

                    if is_ctrl {
                        match ch {
                            'z' | 'Z' => {
                                if let Err(e) = self.perform_undo() {
                                    eprintln!("Undo error: {}", e);
                                }
                            }
                            'y' | 'Y' => {
                                if let Err(e) = self.perform_redo() {
                                    eprintln!("Redo error: {}", e);
                                }
                            }
                            's' | 'S' => {
                                // Ctrl+S - Save world
                                let save_name = format!("world_{}",
                                    std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_secs()
                                );
                                if let Err(e) = self.save_world(&save_name) {
                                    eprintln!("Save error: {}", e);
                                }
                            }
                            'l' | 'L' => {
                                // Ctrl+L - List and potentially load worlds
                                match Self::list_world_saves() {
                                    Ok(saves) => {
                                        if saves.is_empty() {
                                            println!("📂 No saved worlds found");
                                        } else {
                                            println!("📂 Available worlds:");
                                            for (i, save) in saves.iter().enumerate() {
                                                println!("  {}. {}", i + 1, save);
                                            }
                                            // For now, just list. Could add selection UI later
                                            if !saves.is_empty() {
                                                // Load the most recent save for demo purposes
                                                let recent_save = &saves[saves.len() - 1];
                                                println!("🔄 Loading most recent world: {}", recent_save);
                                                if let Err(e) = self.load_world(recent_save) {
                                                    eprintln!("Load error: {}", e);
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => eprintln!("Error listing saves: {}", e),
                                }
                            }
                            _ => {
                                // Handle other character inputs if not Ctrl+Z/Y
                                self.handle_character_input(ch);
                            }
                        }
                    } else {
                        self.handle_character_input(ch);
                    }
                }
                winit::keyboard::Key::Named(NamedKey::F1) => {
                    self.ui_overlay.toggle_help();
                    println!("Help overlay: {}", if self.ui_overlay.show_help { "ON" } else { "OFF" });
                    self.display_status_info();
                }
                winit::keyboard::Key::Named(NamedKey::F2) => {
                    self.ui_overlay.toggle_enabled();
                    println!("UI overlay: {}", if self.ui_overlay.enabled { "ON" } else { "OFF" });
                    self.display_status_info();
                }
                _ => {}
            }
        }
    }

    fn handle_character_input(&mut self, ch: char) {
        match ch {
            // Material selection with number keys
            '1' => {
                // TODO: implement material system in EngineerBuildMode
                // self.build_system.set_material(VoxelType::Stone);
                println!("Selected material: Stone");
                self.display_status_info();
            }
            '2' => {
                // TODO: implement material system in EngineerBuildMode
                // self.build_system.set_material(VoxelType::Dirt);
                println!("Selected material: Dirt");
                self.display_status_info();
            }
            '3' => {
                // TODO: implement material system in EngineerBuildMode
                // self.build_system.set_material(VoxelType::Grass);
                println!("Selected material: Grass");
                self.display_status_info();
            }
            '4' => {
                // TODO: implement material system in EngineerBuildMode
                // self.build_system.set_material(VoxelType::Sand);
                println!("Selected material: Sand");
                self.display_status_info();
            }
            '5' => {
                // TODO: implement material system in EngineerBuildMode
                // self.build_system.set_material(VoxelType::Wood);
                println!("Selected material: Wood");
                self.display_status_info();
            }
            '6' => {
                // TODO: implement material system in EngineerBuildMode
                // self.build_system.set_material(VoxelType::Glass);
                println!("Selected material: Glass");
                self.display_status_info();
            }
            '7' => {
                // TODO: implement material system in EngineerBuildMode
                // self.build_system.set_material(VoxelType::Metal);
                println!("Selected material: Metal");
                self.display_status_info();
            }
            '8' => {
                // TODO: implement material system in EngineerBuildMode
                // self.build_system.set_material(VoxelType::Water);
                println!("Selected material: Water");
                self.display_status_info();
            }
            '9' => {
                // TODO: implement material system in EngineerBuildMode
                // self.build_system.set_material(VoxelType::Lava);
                println!("Selected material: Lava");
                self.display_status_info();
            }
            'm' | 'M' => {
                self.cycle_build_mode();
                self.display_status_info();
            }
            't' | 'T' => {
                self.template_system.cycle_template();
                if let Some(template) = self.template_system.get_current_template() {
                    println!("Selected template: {}", template.name);
                }
            }
            'r' | 'R' => {
                self.template_system.rotate_template();
                println!("Template rotation: {}°", self.template_system.current_rotation * 90);
            }
            _ => {}
        }
    }


    fn cycle_build_mode(&mut self) {
        use robin::engine::build_mode::BuildModeState;

        let current_mode = self.build_system.get_mode();
        let new_mode = match current_mode {
            BuildModeState::Build => BuildModeState::Test,
            BuildModeState::Test => BuildModeState::Play,
            BuildModeState::Play => BuildModeState::Build,
        };

        self.build_system.switch_mode(new_mode);
        println!("Build mode: {:?}", new_mode);
    }

    fn handle_mouse_input(&mut self, state: ElementState, button: MouseButton) -> RobinResult<()> {
        use robin::engine::build_mode::BuildModeState;

        match button {
            MouseButton::Left => {
                if state == ElementState::Pressed {
                    // Remove block (all modes)
                    if self.raycast_result.hit {
                        self.remove_voxel(self.raycast_result.block_position)?;
                    }
                }
            }
            MouseButton::Right => {
                match self.build_system.get_mode() {
                    BuildModeState::Build => {
                        self.handle_wall_mode_mouse(state)?;
                    }
                    BuildModeState::Test => {
                        self.handle_floor_mode_mouse(state)?;
                    }
                    BuildModeState::Play => {
                        self.handle_template_mode_mouse(state)?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_wall_mode_mouse(&mut self, state: ElementState) -> RobinResult<()> {
        match state {
            ElementState::Pressed => {
                if self.raycast_result.hit {
                    let placement_pos = self.calculate_placement_position(&self.raycast_result);

                    if !self.wall_construction.is_constructing {
                        // Start wall construction
                        self.wall_construction.is_constructing = true;
                        self.wall_construction.start_position = Some(placement_pos);
                        self.wall_construction.current_position = Some(placement_pos);
                        self.wall_construction.preview_positions.clear();
                        println!("Started wall construction at {:?}", placement_pos);
                    }
                }
            }
            ElementState::Released => {
                if self.wall_construction.is_constructing {
                    // Finish wall construction
                    self.complete_wall_construction()?;
                    self.wall_construction.is_constructing = false;
                    self.wall_construction.start_position = None;
                    self.wall_construction.current_position = None;
                    self.wall_construction.preview_positions.clear();
                    println!("Completed wall construction");
                }
            }
        }
        Ok(())
    }

    fn calculate_placement_position(&self, raycast_result: &RaycastResult) -> (i32, i32, i32) {
        let (x, y, z) = raycast_result.block_position;
        let normal = raycast_result.face_normal;

        (
            x + normal.x as i32,
            y + normal.y as i32,
            z + normal.z as i32,
        )
    }

    fn complete_wall_construction(&mut self) -> RobinResult<()> {
        if let (Some(start_pos), Some(end_pos)) = (self.wall_construction.start_position, self.wall_construction.current_position) {
            let wall_positions = self.calculate_wall_positions(start_pos, end_pos);
            let material = VoxelType::Stone; // TODO: implement material system in EngineerBuildMode

            let mut placed_count = 0;
            for pos in wall_positions {
                // Check if position is valid before placing
                if !self.chunk_manager.get_voxel(pos.0, pos.1, pos.2).is_some() {
                    if self.chunk_manager.set_voxel(pos.0, pos.1, pos.2, material) {
                        placed_count += 1;
                    }
                }
            }

            if placed_count > 0 {
                println!("Placed {} voxels in wall", placed_count);
                // Regenerate the mesh
                Self::generate_world_mesh_from_chunks(&self.chunk_manager, &mut self.world_mesh, &self.camera);
                self.create_mesh_buffers();
            }
        }
        Ok(())
    }

    fn calculate_wall_positions(&self, start: (i32, i32, i32), end: (i32, i32, i32)) -> Vec<(i32, i32, i32)> {
        let mut positions = Vec::new();

        let dx = end.0 - start.0;
        let dy = end.1 - start.1;
        let dz = end.2 - start.2;

        let steps = dx.abs().max(dy.abs()).max(dz.abs());

        if steps == 0 {
            positions.push(start);
            return positions;
        }

        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let x = start.0 + (dx as f32 * t).round() as i32;
            let y = start.1 + (dy as f32 * t).round() as i32;
            let z = start.2 + (dz as f32 * t).round() as i32;
            positions.push((x, y, z));
        }

        positions
    }

    fn handle_floor_mode_mouse(&mut self, state: ElementState) -> RobinResult<()> {
        match state {
            ElementState::Pressed => {
                if self.raycast_result.hit {
                    let placement_pos = self.calculate_placement_position(&self.raycast_result);

                    if !self.wall_construction.is_constructing {
                        // Start floor construction
                        self.wall_construction.is_constructing = true;
                        self.wall_construction.start_position = Some(placement_pos);
                        self.wall_construction.current_position = Some(placement_pos);
                        self.wall_construction.preview_positions.clear();
                        println!("Started floor construction at {:?}", placement_pos);
                    }
                }
            }
            ElementState::Released => {
                if self.wall_construction.is_constructing {
                    // Finish floor construction
                    self.complete_floor_construction()?;
                    self.wall_construction.is_constructing = false;
                    self.wall_construction.start_position = None;
                    self.wall_construction.current_position = None;
                    self.wall_construction.preview_positions.clear();
                    println!("Completed floor construction");
                }
            }
        }
        Ok(())
    }

    fn complete_floor_construction(&mut self) -> RobinResult<()> {
        if let (Some(start_pos), Some(end_pos)) = (self.wall_construction.start_position, self.wall_construction.current_position) {
            let floor_positions = self.calculate_floor_positions(start_pos, end_pos);
            let material = VoxelType::Stone; // TODO: implement material system in EngineerBuildMode

            let mut placed_count = 0;
            for pos in floor_positions {
                // Check if position is valid before placing
                if !self.chunk_manager.get_voxel(pos.0, pos.1, pos.2).is_some() {
                    if self.chunk_manager.set_voxel(pos.0, pos.1, pos.2, material) {
                        placed_count += 1;
                    }
                }
            }

            if placed_count > 0 {
                println!("Placed {} voxels in floor", placed_count);
                // Regenerate the mesh
                Self::generate_world_mesh_from_chunks(&self.chunk_manager, &mut self.world_mesh, &self.camera);
                self.create_mesh_buffers();
            }
        }
        Ok(())
    }

    fn calculate_floor_positions(&self, start: (i32, i32, i32), end: (i32, i32, i32)) -> Vec<(i32, i32, i32)> {
        let mut positions = Vec::new();

        // Calculate rectangular area boundaries
        let min_x = start.0.min(end.0);
        let max_x = start.0.max(end.0);
        let min_z = start.2.min(end.2);
        let max_z = start.2.max(end.2);

        // Use the Y coordinate from the start position for the floor level
        let y = start.1;

        // Fill the rectangular area
        for x in min_x..=max_x {
            for z in min_z..=max_z {
                positions.push((x, y, z));
            }
        }

        positions
    }

    fn handle_template_mode_mouse(&mut self, state: ElementState) -> RobinResult<()> {
        if state == ElementState::Pressed && self.raycast_result.hit {
            self.place_template()?;
        }
        Ok(())
    }

    fn update_template_preview(&mut self, position: (i32, i32, i32)) {
        self.template_system.preview_voxels.clear();

        // Extract template data first to avoid borrowing conflict
        let template_voxels = if let Some(template) = self.template_system.get_current_template() {
            template.voxels.clone()
        } else {
            return;
        };

        for &voxel in &template_voxels {
            let rotated_voxel = self.template_system.apply_rotation(voxel);
            let world_pos = (
                position.0 + rotated_voxel.0,
                position.1 + rotated_voxel.1,
                position.2 + rotated_voxel.2,
                rotated_voxel.3,
            );
            self.template_system.preview_voxels.push(world_pos);
        }
    }

    fn place_template(&mut self) -> RobinResult<()> {
        if let Some(position) = self.template_system.preview_position {
            if let Some(template) = self.template_system.get_current_template() {
                let mut placed_count = 0;

                for &voxel in &template.voxels {
                    let rotated_voxel = self.template_system.apply_rotation(voxel);
                    let world_pos = (
                        position.0 + rotated_voxel.0,
                        position.1 + rotated_voxel.1,
                        position.2 + rotated_voxel.2,
                    );

                    // Check if position is valid before placing
                    if !self.chunk_manager.get_voxel(world_pos.0, world_pos.1, world_pos.2).is_some() {
                        if self.chunk_manager.set_voxel(world_pos.0, world_pos.1, world_pos.2, rotated_voxel.3) {
                            placed_count += 1;
                        }
                    }
                }

                if placed_count > 0 {
                    println!("Placed template '{}' with {} voxels", template.name, placed_count);
                    // Regenerate the mesh
                    Self::generate_world_mesh_from_chunks(&self.chunk_manager, &mut self.world_mesh, &self.camera);
                    self.create_mesh_buffers();
                }
            }
        }
        Ok(())
    }

    fn remove_voxel(&mut self, block_position: (i32, i32, i32)) -> RobinResult<()> {
        let (x, y, z) = block_position;

        // Record the previous voxel for undo before removing it
        if let Some(previous_voxel) = self.chunk_manager.get_voxel(x, y, z) {
            if self.chunk_manager.remove_voxel(x, y, z) {
                // Record the action for undo
                let action = VoxelAction::Remove {
                    position: block_position,
                    previous_voxel,
                };
                self.record_voxel_action(action);

                // Emit destruction particles
                let particle_position = Vec3::new(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5);
                self.particle_system.emit_block_remove_particles(particle_position, previous_voxel);

                println!("Removed voxel at {:?}", block_position);

                // Regenerate the mesh
                Self::generate_world_mesh_from_chunks(&self.chunk_manager, &mut self.world_mesh, &self.camera);
                self.create_mesh_buffers();
            }
        }
        Ok(())
    }

    fn place_voxel(&mut self, raycast_result: &RaycastResult) -> RobinResult<()> {
        if !raycast_result.hit {
            return Ok(());
        }

        // Calculate placement position by moving one block in the direction of the face normal
        let (x, y, z) = raycast_result.block_position;
        let normal = raycast_result.face_normal;

        let place_x = x + normal.x as i32;
        let place_y = y + normal.y as i32;
        let place_z = z + normal.z as i32;

        let placement_pos = (place_x, place_y, place_z);

        // Don't place if there's already a block there
        if self.chunk_manager.get_voxel(place_x, place_y, place_z).is_some() {
            println!("Cannot place - block already exists at {:?}", placement_pos);
            return Ok(());
        }

        // Place block using current material from build system
        let material = VoxelType::Stone; // TODO: implement material system in EngineerBuildMode

        // Record the previous state for undo (None since there was no voxel)
        let previous_voxel = self.chunk_manager.get_voxel(place_x, place_y, place_z);

        if self.chunk_manager.set_voxel(place_x, place_y, place_z, material) {
            // Record the action for undo
            let action = VoxelAction::Place {
                position: placement_pos,
                voxel_type: material,
                previous_voxel,
            };
            self.record_voxel_action(action);

            // Emit placement particles
            let particle_position = Vec3::new(place_x as f32 + 0.5, place_y as f32 + 0.5, place_z as f32 + 0.5);
            self.particle_system.emit_block_place_particles(particle_position, material);

            println!("Placed voxel at {:?}", placement_pos);

            // Regenerate the mesh
            Self::generate_world_mesh_from_chunks(&self.chunk_manager, &mut self.world_mesh, &self.camera);
            self.create_mesh_buffers();
        }

        Ok(())
    }

    fn generate_wireframe_cube(position: Vec3, color: [f32; 3]) -> Vec<f32> {
        // Generate 12 edges of a cube as line segments (24 vertices total)
        let x = position.x;
        let y = position.y;
        let z = position.z;

        vec![
            // Bottom face edges (4 lines = 8 vertices)
            x, y, z, color[0], color[1], color[2],       // bottom-front-left
            x+1.0, y, z, color[0], color[1], color[2],   // bottom-front-right

            x+1.0, y, z, color[0], color[1], color[2],   // bottom-front-right
            x+1.0, y, z+1.0, color[0], color[1], color[2], // bottom-back-right

            x+1.0, y, z+1.0, color[0], color[1], color[2], // bottom-back-right
            x, y, z+1.0, color[0], color[1], color[2],   // bottom-back-left

            x, y, z+1.0, color[0], color[1], color[2],   // bottom-back-left
            x, y, z, color[0], color[1], color[2],       // bottom-front-left

            // Top face edges (4 lines = 8 vertices)
            x, y+1.0, z, color[0], color[1], color[2],       // top-front-left
            x+1.0, y+1.0, z, color[0], color[1], color[2],   // top-front-right

            x+1.0, y+1.0, z, color[0], color[1], color[2],   // top-front-right
            x+1.0, y+1.0, z+1.0, color[0], color[1], color[2], // top-back-right

            x+1.0, y+1.0, z+1.0, color[0], color[1], color[2], // top-back-right
            x, y+1.0, z+1.0, color[0], color[1], color[2],   // top-back-left

            x, y+1.0, z+1.0, color[0], color[1], color[2],   // top-back-left
            x, y+1.0, z, color[0], color[1], color[2],       // top-front-left

            // Vertical edges (4 lines = 8 vertices)
            x, y, z, color[0], color[1], color[2],           // bottom-front-left
            x, y+1.0, z, color[0], color[1], color[2],       // top-front-left

            x+1.0, y, z, color[0], color[1], color[2],       // bottom-front-right
            x+1.0, y+1.0, z, color[0], color[1], color[2],   // top-front-right

            x+1.0, y, z+1.0, color[0], color[1], color[2],   // bottom-back-right
            x+1.0, y+1.0, z+1.0, color[0], color[1], color[2], // top-back-right

            x, y, z+1.0, color[0], color[1], color[2],       // bottom-back-left
            x, y+1.0, z+1.0, color[0], color[1], color[2],   // top-back-left
        ]
    }

    fn update_wireframe_highlights(&mut self) {
        if self.raycast_result.hit {
            // Show red wireframe around target block
            let (x, y, z) = self.raycast_result.block_position;
            self.target_highlight_position = Vec3::new(x as f32, y as f32, z as f32);
            self.target_highlight_visible = true;

            // Show green wireframe for placement position
            let normal = self.raycast_result.face_normal;
            let place_x = x + normal.x as i32;
            let place_y = y + normal.y as i32;
            let place_z = z + normal.z as i32;

            // Only show placement preview if position is empty
            if self.chunk_manager.get_voxel(place_x, place_y, place_z).is_none() {
                self.placement_highlight_position = Vec3::new(place_x as f32, place_y as f32, place_z as f32);
                self.placement_highlight_visible = true;
            } else {
                self.placement_highlight_visible = false;
            }
        } else {
            self.target_highlight_visible = false;
            self.placement_highlight_visible = false;
        }

        // Update wireframe vertex buffer
        let mut wireframe_data = Vec::new();

        if self.target_highlight_visible {
            let red_wireframe = Self::generate_wireframe_cube(
                self.target_highlight_position,
                [1.0, 0.0, 0.0] // Red color
            );
            wireframe_data.extend(red_wireframe);
        }

        if self.placement_highlight_visible {
            let green_wireframe = Self::generate_wireframe_cube(
                self.placement_highlight_position,
                [0.0, 1.0, 0.0] // Green color
            );
            wireframe_data.extend(green_wireframe);
        }

        // Update GPU buffer
        if !wireframe_data.is_empty() {
            self.queue.write_buffer(
                &self.wireframe_vertex_buffer,
                0,
                bytemuck::cast_slice(&wireframe_data),
            );
        }
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) -> RobinResult<()> {
        if new_size.width > 0 && new_size.height > 0 {
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);
            self.camera.aspect = new_size.width as f32 / new_size.height as f32;
        }
        Ok(())
    }

    // Undo/Redo System Methods
    fn record_voxel_action(&mut self, action: VoxelAction) {
        self.undo_system.record_action(action);
    }

    fn start_batch_operation(&mut self, description: String) {
        self.undo_system.start_batch(description);
    }

    fn end_batch_operation(&mut self) {
        self.undo_system.end_batch();
    }

    fn perform_undo(&mut self) -> RobinResult<()> {
        if let Some(action) = self.undo_system.undo() {
            self.apply_undo_action(action)?;
            // Regenerate world mesh after undo
            Self::generate_world_mesh_from_chunks(&self.chunk_manager, &mut self.world_mesh, &self.camera);
            self.create_mesh_buffers();
            println!("✅ Undo completed");
        } else {
            println!("❌ Nothing to undo");
        }
        Ok(())
    }

    fn perform_redo(&mut self) -> RobinResult<()> {
        if let Some(action) = self.undo_system.redo() {
            self.apply_redo_action(action)?;
            // Regenerate world mesh after redo
            Self::generate_world_mesh_from_chunks(&self.chunk_manager, &mut self.world_mesh, &self.camera);
            self.create_mesh_buffers();
            println!("✅ Redo completed");
        } else {
            println!("❌ Nothing to redo");
        }
        Ok(())
    }

    fn apply_undo_action(&mut self, action: VoxelAction) -> RobinResult<()> {
        match action {
            VoxelAction::Place { position, previous_voxel, .. } => {
                // Undo a place operation by restoring the previous voxel or removing it
                match previous_voxel {
                    Some(prev_voxel) => {
                        self.chunk_manager.set_voxel(position.0, position.1, position.2, prev_voxel);
                    }
                    None => {
                        self.chunk_manager.remove_voxel(position.0, position.1, position.2);
                    }
                }
            }
            VoxelAction::Remove { position, previous_voxel } => {
                // Undo a remove operation by placing back the previous voxel
                self.chunk_manager.set_voxel(position.0, position.1, position.2, previous_voxel);
            }
            VoxelAction::Batch { actions, .. } => {
                // Undo batch operations in reverse order
                for action in actions.into_iter().rev() {
                    self.apply_undo_action(action)?;
                }
            }
        }
        Ok(())
    }

    fn apply_redo_action(&mut self, action: VoxelAction) -> RobinResult<()> {
        match action {
            VoxelAction::Place { position, voxel_type, .. } => {
                // Redo a place operation
                self.chunk_manager.set_voxel(position.0, position.1, position.2, voxel_type);
            }
            VoxelAction::Remove { position, .. } => {
                // Redo a remove operation
                self.chunk_manager.remove_voxel(position.0, position.1, position.2);
            }
            VoxelAction::Batch { actions, .. } => {
                // Redo batch operations in original order
                for action in actions {
                    self.apply_redo_action(action)?;
                }
            }
        }
        Ok(())
    }

    fn create_ui_mesh_buffers(&mut self) {
        // Generate UI mesh from overlay
        let (vertices, indices) = self.ui_overlay.generate_ui_mesh();

        if !vertices.is_empty() && !indices.is_empty() {
            // Create vertex buffer
            self.ui_vertex_buffer = Some(self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("UI Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }));

            // Create index buffer
            self.ui_index_buffer = Some(self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("UI Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            }));
        } else {
            self.ui_vertex_buffer = None;
            self.ui_index_buffer = None;
        }
    }

    // Display current status information
    fn display_status_info(&self) {
        if !self.ui_overlay.enabled {
            return;
        }

        // Build status line by line
        let current_mode = self.build_system.get_mode();
        let current_material = VoxelType::Stone; // TODO: implement material system in EngineerBuildMode

        println!("\n=== Robin Engine Status ===");
        println!("Build Mode: {:?}", current_mode); // TODO: fix get_mode_name for BuildModeState
        println!("Material: {}", UIOverlay::get_material_name(current_material));

        if let Some(template) = self.template_system.get_current_template() {
            println!("Template: {} ({}°)", template.name, self.template_system.current_rotation * 90);
        }

        println!("FPS: {:.1}", self.ui_overlay.current_fps);
        println!("Undo Available: {}", if self.undo_system.can_undo() { "Yes" } else { "No" });
        println!("Redo Available: {}", if self.undo_system.can_redo() { "Yes" } else { "No" });

        if self.ui_overlay.show_help {
            println!("\n=== Controls ===");
            println!("WASD: Move camera");
            println!("Mouse: Look around");
            println!("1-9: Select material");
            println!("M: Cycle build mode");
            println!("T: Cycle template (Template mode)");
            println!("R: Rotate template (Template mode)");
            println!("Ctrl+Z: Undo");
            println!("Ctrl+Y: Redo");
            println!("Ctrl+S: Save world");
            println!("Ctrl+L: List/Load worlds");
            println!("F1: Toggle help");
            println!("F2: Toggle UI overlay");
            println!("Left Click: Remove voxel");
            println!("Right Click: Place voxel");
        }
        println!("===========================\n");
    }

    /// Save the current voxel world to a file
    fn save_world(&self, save_name: &str) -> RobinResult<()> {
        use std::fs;
        use serde_json;

        // Create saves directory if it doesn't exist
        let saves_dir = std::env::current_dir()?.join("saves");
        if !saves_dir.exists() {
            fs::create_dir_all(&saves_dir)?;
        }

        // Create world save data
        let world_save = WorldSaveData {
            version: "1.0".to_string(),
            save_name: save_name.to_string(),
            created_at: std::time::SystemTime::now(),
            player_position: self.camera.position,
            player_target: self.camera.target,
            chunks: self.chunk_manager.get_loaded_chunks()
                .iter()
                .map(|(pos, chunk)| (pos.clone(), ChunkSaveData::from_chunk(chunk)))
                .collect(),
        };

        // Save to file
        let file_path = saves_dir.join(format!("{}.robinworld", save_name));
        let file = fs::File::create(&file_path)?;
        serde_json::to_writer_pretty(file, &world_save)?;

        println!("💾 World saved to: {:?}", file_path);
        Ok(())
    }

    /// Load a voxel world from a file
    fn load_world(&mut self, save_name: &str) -> RobinResult<()> {
        use std::fs;
        use serde_json;

        let saves_dir = std::env::current_dir()?.join("saves");
        let file_path = saves_dir.join(format!("{}.robinworld", save_name));

        if !file_path.exists() {
            return Err(RobinError::FileNotFound(file_path));
        }

        // Load world data
        let file = fs::File::open(&file_path)?;
        let world_save: WorldSaveData = serde_json::from_reader(file)?;

        println!("📂 Loading world: {}", world_save.save_name);

        // Clear current world
        self.chunk_manager = ChunkManager::new();

        // Restore chunks
        for ((chunk_x, chunk_y, chunk_z), chunk_data) in world_save.chunks {
            let mut chunk = Chunk::new(chunk_x, chunk_y, chunk_z);

            // Restore voxels
            for ((local_x, local_y, local_z), voxel_type) in chunk_data.voxels {
                chunk.set_voxel(local_x, local_y, local_z, voxel_type);
            }

            self.chunk_manager.chunks.insert((chunk_x, chunk_y, chunk_z), chunk);
        }

        // Restore player position
        self.camera.position = world_save.player_position;
        self.camera.target = world_save.player_target;

        // Update camera position in chunk manager
        self.chunk_manager.update_camera_position(self.camera.position);

        // Regenerate mesh for the loaded world
        Self::generate_world_mesh_from_chunks(&self.chunk_manager, &mut self.world_mesh, &self.camera);
        self.create_mesh_buffers();

        println!("✅ World loaded successfully");
        Ok(())
    }

    /// List available world saves
    fn list_world_saves() -> RobinResult<Vec<String>> {
        use std::fs;

        let saves_dir = std::env::current_dir()?.join("saves");
        if !saves_dir.exists() {
            return Ok(Vec::new());
        }

        let mut saves = Vec::new();
        for entry in fs::read_dir(saves_dir)? {
            let entry = entry?;
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(".robinworld") {
                    let save_name = file_name.strip_suffix(".robinworld").unwrap().to_string();
                    saves.push(save_name);
                }
            }
        }

        saves.sort();
        Ok(saves)
    }
}

fn main() -> RobinResult<()> {
    env_logger::init();

    println!("🚀 Starting Robin Engine - Unified 3D Demo");

    // Detect platform capabilities
    let capabilities = PlatformCapabilities::detect();
    println!("✨ Platform capabilities:");
    println!("   Metal support: {}", capabilities.has_metal);
    println!("   Apple Silicon: {}", capabilities.has_apple_silicon);
    println!("   Unified memory: {}", capabilities.unified_memory);
    println!("   Max texture size: {}", capabilities.max_texture_size);

    let best_backend = detect_best_backend();
    println!("   ✅ Best backend: {}", best_backend);

    // Create event loop and window
    let event_loop = EventLoop::new()
        .map_err(|e| RobinError::GraphicsInitError(format!("Failed to create event loop: {}", e)))?;

    let window = Arc::new(WindowBuilder::new()
        .with_title("Robin Engine - 3D Voxel World")
        .with_inner_size(PhysicalSize::new(1200, 800))
        .build(&event_loop)
        .map_err(|e| RobinError::GraphicsInitError(format!("Failed to create window: {}", e)))?);

    println!("✅ Created window: {}x{}", 1200, 800);

    // Create the application
    let mut app = pollster::block_on(RobinApp::new(window.clone()))?;
    println!("✅ Initialized Robin Engine systems");

    println!("\n🎮 Controls:");
    println!("   WASD - Move camera");
    println!("   Arrow keys - Look around");
    println!("   Space/Shift - Up/Down");
    println!("   Escape - Exit");
    println!("\n🌍 Rendering voxel world...");

    // Run the event loop
    let mut last_frame_time = Instant::now();
    event_loop.run(move |event, event_loop_window_target| {
        match event {
            Event::WindowEvent { window_id, event } if window_id == window.id() => {
                match event {
                    WindowEvent::CloseRequested => {
                        event_loop_window_target.exit();
                    }
                    WindowEvent::KeyboardInput { event, .. } => {
                        if event.logical_key == Key::Named(NamedKey::Escape) {
                            if event.state == ElementState::Pressed {
                                event_loop_window_target.exit();
                            }
                        }
                        app.handle_keyboard_input(&event);
                    }
                    _ => {
                        if let Err(e) = app.handle_window_event(&event) {
                            eprintln!("Error handling window event: {}", e);
                        }
                    }
                }
            }
            Event::AboutToWait => {
                // Update and render
                let current_time = Instant::now();
                let delta_time = current_time.duration_since(last_frame_time).as_secs_f32();
                last_frame_time = current_time;

                if let Err(e) = app.update(delta_time) {
                    eprintln!("Error updating app: {}", e);
                }

                if let Err(e) = app.render() {
                    eprintln!("Error rendering: {}", e);
                }

                // Request redraw
                window.request_redraw();
            }
            _ => {}
        }
    }).map_err(|e| RobinError::GraphicsInitError(format!("Event loop error: {}", e)))?;

    Ok(())
}