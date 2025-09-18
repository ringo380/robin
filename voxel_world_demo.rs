use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder, CursorGrabMode},
};
use wgpu::util::DeviceExt;
use std::time::Instant;
use std::sync::Arc;
use cgmath::*;

// Voxel data structures (simplified from our VoxelEngine)
#[derive(Clone, Copy, Debug)]
pub struct Voxel {
    pub material_id: u8,
    pub color: [f32; 4],
}

#[derive(Clone, Copy, Debug)]
pub enum MaterialType {
    Air = 0,
    Earth = 1,
    Stone = 2,
    Water = 3,
    Grass = 4,
    Sand = 5,
}

// Vertex structure for voxel rendering
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct VoxelVertex {
    position: [f32; 3],
    normal: [f32; 3],
    color: [f32; 4],
    material_id: u32,
}

impl VoxelVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<VoxelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Normal
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Color
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 3]>() * 2) as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // Material ID
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 3]>() * 2 + std::mem::size_of::<[f32; 4]>()) as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Uint32,
                },
            ],
        }
    }
}

// Camera system (adapted from EngineerController concepts)
#[repr(C, align(16))]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],      // 64 bytes
    camera_pos: [f32; 4],          // 16 bytes  
    sun_direction: [f32; 4],       // 16 bytes
    time_of_day: [f32; 4],         // 16 bytes - vec4 alignment
}

impl CameraUniform {
    fn new() -> Self {
        Self {
            view_proj: Matrix4::identity().into(),
            camera_pos: [0.0, 0.0, 0.0, 1.0],
            sun_direction: [0.5, 1.0, 0.3, 0.0], // Default sun direction
            time_of_day: [0.4, 0.0, 0.0, 0.0], // Start at mid-day
        }
    }

    fn update_view_proj(&mut self, view_proj: Matrix4<f32>, camera_pos: Point3<f32>) {
        self.view_proj = view_proj.into();
        self.camera_pos = [camera_pos.x, camera_pos.y, camera_pos.z, 1.0];
    }
    
    fn update_sun(&mut self, sun_direction: Vector3<f32>, time_of_day: f32) {
        self.sun_direction = [sun_direction.x, sun_direction.y, sun_direction.z, 0.0];
        self.time_of_day = [time_of_day, 0.0, 0.0, 0.0];
    }
}

struct Camera {
    position: Point3<f32>,
    yaw: Rad<f32>,
    pitch: Rad<f32>,
    aspect: f32,
    fovy: Rad<f32>,
    znear: f32,
    zfar: f32,
}

impl Camera {
    fn new(aspect: f32) -> Self {
        Self {
            position: Point3::new(0.0, generate_terrain_height(0.0, 0.0) + 1.8, 0.0), // Start on terrain surface
            yaw: Rad(0.0),
            pitch: Rad(-0.3), // Look down slightly
            aspect,
            fovy: Rad(std::f32::consts::PI / 4.0),
            znear: 0.1,
            zfar: 1000.0,
        }
    }

    fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        let view = Matrix4::look_at_rh(
            self.position,
            self.position + Vector3::new(
                self.yaw.0.cos() * self.pitch.0.cos(),
                self.pitch.0.sin(),
                self.yaw.0.sin() * self.pitch.0.cos(),
            ),
            Vector3::unit_y(),
        );
        let proj = perspective(self.fovy, self.aspect, self.znear, self.zfar);
        proj * view
    }
}

// Enhanced camera controller with proper physics
struct CameraController {
    // Movement
    move_speed: f32,
    fly_speed: f32,
    mouse_sensitivity: f32,
    
    // Input state
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_running: bool,
    
    // Physics
    velocity: Vector3<f32>,
    is_flying: bool,
    gravity: f32,
    jump_force: f32,
    ground_friction: f32,
}

impl CameraController {
    fn new(move_speed: f32, mouse_sensitivity: f32) -> Self {
        Self {
            move_speed,
            fly_speed: move_speed * 3.0,
            mouse_sensitivity,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_up_pressed: false,
            is_down_pressed: false,
            is_running: false,
            velocity: Vector3::zero(),
            is_flying: false, // Start in walking mode
            gravity: -30.0,
            jump_force: 15.0,
            ground_friction: 10.0,
        }
    }

    fn process_keyboard(&mut self, key: PhysicalKey, state: ElementState) -> bool {
        let is_pressed = state == ElementState::Pressed;
        match key {
            PhysicalKey::Code(KeyCode::KeyW) => {
                self.is_forward_pressed = is_pressed;
                true
            }
            PhysicalKey::Code(KeyCode::KeyS) => {
                self.is_backward_pressed = is_pressed;
                true
            }
            PhysicalKey::Code(KeyCode::KeyA) => {
                self.is_left_pressed = is_pressed;
                true
            }
            PhysicalKey::Code(KeyCode::KeyD) => {
                self.is_right_pressed = is_pressed;
                true
            }
            PhysicalKey::Code(KeyCode::Space) => {
                self.is_up_pressed = is_pressed;
                true
            }
            PhysicalKey::Code(KeyCode::ControlLeft) => {
                self.is_down_pressed = is_pressed;
                true
            }
            PhysicalKey::Code(KeyCode::ShiftLeft) => {
                self.is_running = is_pressed;
                true
            }
            PhysicalKey::Code(KeyCode::KeyF) if is_pressed => {
                self.is_flying = !self.is_flying;
                if self.is_flying {
                    self.velocity.y = 0.0; // Stop falling when entering fly mode
                }
                true
            }
            _ => false,
        }
    }

    fn process_mouse(&mut self, _mouse_dx: f64, _mouse_dy: f64) {
        // Mouse input is handled in the main update function
    }

    fn update_camera(&mut self, camera: &mut Camera, dt: f32, mouse_delta: (f64, f64)) {
        // Mouse look
        camera.yaw += Rad(mouse_delta.0 as f32 * self.mouse_sensitivity);
        camera.pitch += Rad(-mouse_delta.1 as f32 * self.mouse_sensitivity);
        
        // Clamp pitch
        camera.pitch.0 = camera.pitch.0.clamp(-std::f32::consts::FRAC_PI_2 * 0.99, std::f32::consts::FRAC_PI_2 * 0.99);

        // Calculate movement vectors (fixed orientation)
        let forward = Vector3::new(camera.yaw.0.cos(), 0.0, camera.yaw.0.sin()).normalize();
        let right = Vector3::new(-camera.yaw.0.sin(), 0.0, camera.yaw.0.cos()).normalize();
        let up = Vector3::unit_y();

        // Speed calculation
        let current_speed = if self.is_running { 
            if self.is_flying { self.fly_speed * 2.0 } else { self.move_speed * 2.0 }
        } else {
            if self.is_flying { self.fly_speed } else { self.move_speed }
        };

        // Input processing
        let mut input_direction = Vector3::zero();
        
        if self.is_forward_pressed {
            input_direction += forward;
        }
        if self.is_backward_pressed {
            input_direction -= forward;
        }
        if self.is_right_pressed {
            input_direction += right;
        }
        if self.is_left_pressed {
            input_direction -= right;
        }

        // Flying mode
        if self.is_flying {
            if self.is_up_pressed {
                input_direction += up;
            }
            if self.is_down_pressed {
                input_direction -= up;
            }
            
            // Normalize and apply speed
            if input_direction.magnitude() > 0.0 {
                input_direction = input_direction.normalize() * current_speed;
            }
            
            // Smooth velocity interpolation for flying
            self.velocity = self.velocity * 0.85 + input_direction * 0.15;
        } else {
            // Ground-based movement with terrain following
            let horizontal_input = Vector3::new(input_direction.x, 0.0, input_direction.z);
            if horizontal_input.magnitude() > 0.0 {
                let normalized_input = horizontal_input.normalize() * current_speed;
                self.velocity.x = normalized_input.x;
                self.velocity.z = normalized_input.z;
            } else {
                // Apply friction
                self.velocity.x *= 1.0 - self.ground_friction * dt;
                self.velocity.z *= 1.0 - self.ground_friction * dt;
            }
            
            // Jump only when on ground
            if self.is_up_pressed && self.is_on_ground(camera.position) {
                self.velocity.y = self.jump_force;
            }
            
            // Apply gravity
            self.velocity.y += self.gravity * dt;
            
            // Predict new position
            let new_x = camera.position.x + self.velocity.x * dt;
            let new_z = camera.position.z + self.velocity.z * dt;
            let new_y = camera.position.y + self.velocity.y * dt;
            
            // Get terrain height at new position
            let ground_height = self.get_ground_height(new_x, new_z);
            let eye_height = 1.8;
            
            // Terrain following and collision
            if new_y <= ground_height + eye_height {
                // On or below ground - snap to surface
                camera.position.x = new_x;
                camera.position.z = new_z; 
                camera.position.y = ground_height + eye_height;
                if self.velocity.y < 0.0 {
                    self.velocity.y = 0.0;
                }
            } else {
                // In air - apply all movement
                camera.position.x = new_x;
                camera.position.z = new_z;
                camera.position.y = new_y;
            }
        }
    }


    fn get_ground_height(&self, x: f32, z: f32) -> f32 {
        // Use the same terrain generation as our mesh
        generate_terrain_height(x, z)
    }
    
    fn is_on_ground(&self, position: Point3<f32>) -> bool {
        let ground_height = self.get_ground_height(position.x, position.z);
        (position.y - ground_height - 1.8).abs() < 0.1 // Small tolerance for ground detection
    }
}

// Voxel world generation (simplified from our VoxelEngine)
const CHUNK_SIZE: i32 = 32;
const WORLD_HEIGHT: i32 = 64;

fn generate_terrain_height(x: f32, z: f32) -> f32 {
    // Primary terrain features
    let base_height = 20.0;
    let mountains = 15.0 * ((x * 0.01).sin() * (z * 0.01).cos()).abs();
    let hills = 8.0 * ((x * 0.03).cos() + (z * 0.025).sin());
    let detail = 3.0 * ((x * 0.1).sin() * (z * 0.08).cos());
    
    base_height + mountains + hills + detail
}

fn get_voxel_type(x: f32, y: f32, z: f32) -> MaterialType {
    let surface_height = generate_terrain_height(x, z);
    let depth = surface_height - y;
    
    if y > surface_height {
        MaterialType::Air
    } else if depth < 1.0 {
        // Surface layer
        if surface_height > 35.0 {
            MaterialType::Stone // Mountain peaks
        } else if surface_height > 25.0 {
            MaterialType::Earth // Hills
        } else if surface_height < 18.0 {
            MaterialType::Sand // Beaches
        } else {
            MaterialType::Grass // Grassland
        }
    } else if depth < 3.0 {
        MaterialType::Earth // Subsurface
    } else {
        MaterialType::Stone // Deep underground
    }
}

fn get_material_color(material: MaterialType) -> [f32; 4] {
    match material {
        MaterialType::Air => [0.0, 0.0, 0.0, 0.0],
        MaterialType::Earth => [0.4, 0.3, 0.2, 1.0],
        MaterialType::Stone => [0.6, 0.6, 0.65, 1.0],
        MaterialType::Water => [0.2, 0.4, 0.8, 0.8],
        MaterialType::Grass => [0.2, 0.8, 0.2, 1.0],
        MaterialType::Sand => [0.9, 0.8, 0.4, 1.0],
    }
}

// Generate voxel world mesh
fn generate_voxel_world(center_x: i32, center_z: i32, render_distance: i32) -> Vec<VoxelVertex> {
    let mut vertices = Vec::new();
    
    for chunk_x in (center_x - render_distance)..(center_x + render_distance) {
        for chunk_z in (center_z - render_distance)..(center_z + render_distance) {
            let chunk_vertices = generate_chunk_mesh(chunk_x, chunk_z);
            vertices.extend(chunk_vertices);
        }
    }
    
    vertices
}

fn generate_chunk_mesh(chunk_x: i32, chunk_z: i32) -> Vec<VoxelVertex> {
    let mut vertices = Vec::new();
    
    // Generate voxels for this chunk
    let start_x = chunk_x * CHUNK_SIZE;
    let start_z = chunk_z * CHUNK_SIZE;
    
    for local_x in 0..CHUNK_SIZE {
        for local_z in 0..CHUNK_SIZE {
            let world_x = start_x + local_x;
            let world_z = start_z + local_z;
            
            let surface_height = generate_terrain_height(world_x as f32, world_z as f32) as i32;
            let max_height = surface_height.min(WORLD_HEIGHT - 1);
            
            // Generate terrain from bottom up to surface
            for y in 0..=max_height {
                let voxel_type = get_voxel_type(world_x as f32, y as f32, world_z as f32);
                
                if voxel_type as u8 != MaterialType::Air as u8 {
                    let voxel_vertices = generate_voxel_mesh(
                        Point3::new(world_x as f32, y as f32, world_z as f32),
                        voxel_type,
                        world_x, y, world_z,
                    );
                    vertices.extend(voxel_vertices);
                }
            }
        }
    }
    
    vertices
}

fn generate_voxel_mesh(position: Point3<f32>, material: MaterialType, world_x: i32, world_y: i32, world_z: i32) -> Vec<VoxelVertex> {
    let mut vertices = Vec::new();
    let color = get_material_color(material);
    
    // Check which faces should be rendered (simplified - assumes air above surface)
    let faces_to_render = get_visible_faces(world_x, world_y, world_z);
    
    for face in faces_to_render {
        let face_vertices = create_face_vertices(position, face, color, material as u32);
        vertices.extend(face_vertices);
    }
    
    vertices
}

#[derive(Clone, Copy)]
enum Face {
    Top,
    Bottom,
    North,
    South,
    East,
    West,
}

fn get_visible_faces(x: i32, y: i32, z: i32) -> Vec<Face> {
    let mut faces = Vec::new();
    let surface_height = generate_terrain_height(x as f32, z as f32) as i32;

    // Only generate faces for surface and near-surface voxels
    if y > surface_height + 2 || y < surface_height - 5 {
        return faces; // Skip voxels too far from surface
    }

    let voxel_type = get_voxel_type(x as f32, y as f32, z as f32);
    if voxel_type as u8 == MaterialType::Air as u8 {
        return faces; // Skip air voxels
    }

    // Top face - always render for surface voxels, or if air above
    if y == surface_height {
        faces.push(Face::Top);
    } else {
        let neighbor_up = get_voxel_type(x as f32, (y + 1) as f32, z as f32);
        if neighbor_up as u8 == MaterialType::Air as u8 {
            faces.push(Face::Top);
        }
    }

    // Side faces - check neighboring height to determine visibility
    // East face (+X direction)
    let neighbor_east_height = generate_terrain_height((x + 1) as f32, z as f32) as i32;
    if get_voxel_type((x + 1) as f32, y as f32, z as f32) as u8 == MaterialType::Air as u8
        || y > neighbor_east_height {
        faces.push(Face::East);
    }

    // West face (-X direction)
    let neighbor_west_height = generate_terrain_height((x - 1) as f32, z as f32) as i32;
    if get_voxel_type((x - 1) as f32, y as f32, z as f32) as u8 == MaterialType::Air as u8
        || y > neighbor_west_height {
        faces.push(Face::West);
    }

    // North face (+Z direction)
    let neighbor_north_height = generate_terrain_height(x as f32, (z + 1) as f32) as i32;
    if get_voxel_type(x as f32, y as f32, (z + 1) as f32) as u8 == MaterialType::Air as u8
        || y > neighbor_north_height {
        faces.push(Face::North);
    }

    // South face (-Z direction)
    let neighbor_south_height = generate_terrain_height(x as f32, (z - 1) as f32) as i32;
    if get_voxel_type(x as f32, y as f32, (z - 1) as f32) as u8 == MaterialType::Air as u8
        || y > neighbor_south_height {
        faces.push(Face::South);
    }

    // Bottom face - only for voxels with air below
    if get_voxel_type(x as f32, (y - 1) as f32, z as f32) as u8 == MaterialType::Air as u8 {
        faces.push(Face::Bottom);
    }

    faces
}

fn create_face_vertices(position: Point3<f32>, face: Face, color: [f32; 4], material_id: u32) -> Vec<VoxelVertex> {
    let x = position.x;
    let y = position.y;
    let z = position.z;
    
    match face {
        Face::Top => vec![
            VoxelVertex { position: [x, y + 1.0, z], normal: [0.0, 1.0, 0.0], color, material_id },
            VoxelVertex { position: [x + 1.0, y + 1.0, z], normal: [0.0, 1.0, 0.0], color, material_id },
            VoxelVertex { position: [x + 1.0, y + 1.0, z + 1.0], normal: [0.0, 1.0, 0.0], color, material_id },
            VoxelVertex { position: [x, y + 1.0, z], normal: [0.0, 1.0, 0.0], color, material_id },
            VoxelVertex { position: [x + 1.0, y + 1.0, z + 1.0], normal: [0.0, 1.0, 0.0], color, material_id },
            VoxelVertex { position: [x, y + 1.0, z + 1.0], normal: [0.0, 1.0, 0.0], color, material_id },
        ],
        Face::Bottom => vec![
            VoxelVertex { position: [x, y, z], normal: [0.0, -1.0, 0.0], color, material_id },
            VoxelVertex { position: [x, y, z + 1.0], normal: [0.0, -1.0, 0.0], color, material_id },
            VoxelVertex { position: [x + 1.0, y, z + 1.0], normal: [0.0, -1.0, 0.0], color, material_id },
            VoxelVertex { position: [x, y, z], normal: [0.0, -1.0, 0.0], color, material_id },
            VoxelVertex { position: [x + 1.0, y, z + 1.0], normal: [0.0, -1.0, 0.0], color, material_id },
            VoxelVertex { position: [x + 1.0, y, z], normal: [0.0, -1.0, 0.0], color, material_id },
        ],
        Face::North => vec![
            VoxelVertex { position: [x, y, z + 1.0], normal: [0.0, 0.0, 1.0], color, material_id },
            VoxelVertex { position: [x, y + 1.0, z + 1.0], normal: [0.0, 0.0, 1.0], color, material_id },
            VoxelVertex { position: [x + 1.0, y + 1.0, z + 1.0], normal: [0.0, 0.0, 1.0], color, material_id },
            VoxelVertex { position: [x, y, z + 1.0], normal: [0.0, 0.0, 1.0], color, material_id },
            VoxelVertex { position: [x + 1.0, y + 1.0, z + 1.0], normal: [0.0, 0.0, 1.0], color, material_id },
            VoxelVertex { position: [x + 1.0, y, z + 1.0], normal: [0.0, 0.0, 1.0], color, material_id },
        ],
        Face::South => vec![
            VoxelVertex { position: [x, y, z], normal: [0.0, 0.0, -1.0], color, material_id },
            VoxelVertex { position: [x + 1.0, y, z], normal: [0.0, 0.0, -1.0], color, material_id },
            VoxelVertex { position: [x + 1.0, y + 1.0, z], normal: [0.0, 0.0, -1.0], color, material_id },
            VoxelVertex { position: [x, y, z], normal: [0.0, 0.0, -1.0], color, material_id },
            VoxelVertex { position: [x + 1.0, y + 1.0, z], normal: [0.0, 0.0, -1.0], color, material_id },
            VoxelVertex { position: [x, y + 1.0, z], normal: [0.0, 0.0, -1.0], color, material_id },
        ],
        Face::East => vec![
            VoxelVertex { position: [x + 1.0, y, z], normal: [1.0, 0.0, 0.0], color, material_id },
            VoxelVertex { position: [x + 1.0, y, z + 1.0], normal: [1.0, 0.0, 0.0], color, material_id },
            VoxelVertex { position: [x + 1.0, y + 1.0, z + 1.0], normal: [1.0, 0.0, 0.0], color, material_id },
            VoxelVertex { position: [x + 1.0, y, z], normal: [1.0, 0.0, 0.0], color, material_id },
            VoxelVertex { position: [x + 1.0, y + 1.0, z + 1.0], normal: [1.0, 0.0, 0.0], color, material_id },
            VoxelVertex { position: [x + 1.0, y + 1.0, z], normal: [1.0, 0.0, 0.0], color, material_id },
        ],
        Face::West => vec![
            VoxelVertex { position: [x, y, z], normal: [-1.0, 0.0, 0.0], color, material_id },
            VoxelVertex { position: [x, y + 1.0, z], normal: [-1.0, 0.0, 0.0], color, material_id },
            VoxelVertex { position: [x, y + 1.0, z + 1.0], normal: [-1.0, 0.0, 0.0], color, material_id },
            VoxelVertex { position: [x, y, z], normal: [-1.0, 0.0, 0.0], color, material_id },
            VoxelVertex { position: [x, y + 1.0, z + 1.0], normal: [-1.0, 0.0, 0.0], color, material_id },
            VoxelVertex { position: [x, y, z + 1.0], normal: [-1.0, 0.0, 0.0], color, material_id },
        ],
    }
}

// Professional outdoor lighting shader with skybox and sun
const VOXEL_SHADER: &str = r#"
struct CameraUniform {
    view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    sun_direction: vec4<f32>,
    time_of_day: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
    @location(3) material_id: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
    @location(3) material_id: u32,
    @location(4) view_direction: vec3<f32>,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.world_position = model.position;
    out.normal = normalize(model.normal);
    out.color = model.color;
    out.material_id = model.material_id;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    out.view_direction = normalize(camera.camera_pos.xyz - model.position);
    return out;
}

// Sky color calculation for skybox
fn get_sky_color(view_dir: vec3<f32>) -> vec3<f32> {
    let sun_dir = normalize(camera.sun_direction.xyz);
    
    // Sky gradient based on height
    let horizon_factor = clamp(view_dir.y + 0.1, 0.0, 1.0);
    
    // Time of day colors
    let day_sky_color = vec3<f32>(0.4, 0.7, 1.0);
    let day_horizon_color = vec3<f32>(0.8, 0.9, 1.0);
    let sunset_sky_color = vec3<f32>(1.0, 0.4, 0.2);
    let sunset_horizon_color = vec3<f32>(1.0, 0.6, 0.3);
    let night_sky_color = vec3<f32>(0.05, 0.05, 0.2);
    let night_horizon_color = vec3<f32>(0.1, 0.1, 0.3);
    
    // Time interpolation
    let time_factor = camera.time_of_day.x;
    var sky_color: vec3<f32>;
    var horizon_color: vec3<f32>;
    
    if (time_factor < 0.3) {
        // Night to dawn
        let t = time_factor / 0.3;
        sky_color = mix(night_sky_color, day_sky_color, t);
        horizon_color = mix(night_horizon_color, day_horizon_color, t);
    } else if (time_factor < 0.7) {
        // Day
        sky_color = day_sky_color;
        horizon_color = day_horizon_color;
    } else {
        // Dusk to night
        let t = (time_factor - 0.7) / 0.3;
        sky_color = mix(day_sky_color, sunset_sky_color, t);
        horizon_color = mix(day_horizon_color, sunset_horizon_color, t);
    }
    
    let base_color = mix(horizon_color, sky_color, horizon_factor);
    
    // Sun disk (visible when above horizon)
    let sun_dot = dot(view_dir, sun_dir);
    var sun_contribution = vec3<f32>(0.0);
    
    if (sun_dir.y > 0.0) { // Sun is above horizon
        let sun_disk = pow(max(sun_dot, 0.0), 512.0) * 8.0; // Sharp, bright sun
        let sun_glow = pow(max(sun_dot, 0.0), 8.0) * 1.5;   // Softer glow
        let sun_corona = pow(max(sun_dot, 0.0), 2.0) * 0.3; // Wide corona
        
        sun_contribution = vec3<f32>(1.0, 0.9, 0.7) * (sun_disk + sun_glow + sun_corona);
    }
    
    // Moon (opposite side of sky from sun)
    var moon_dir = -sun_dir;
    moon_dir.y = abs(moon_dir.y); // Moon always visible above horizon
    let moon_dot = dot(view_dir, normalize(moon_dir));
    var moon_contribution = vec3<f32>(0.0);
    
    if (sun_dir.y <= 0.0 && moon_dot > 0.999) { // Moon visible only at night and very precise
        let moon_disk = pow(max(moon_dot, 0.0), 2048.0) * 2.0; // Small, dim moon
        moon_contribution = vec3<f32>(0.8, 0.8, 1.0) * moon_disk;
    }
    
    return base_color + sun_contribution + moon_contribution;
}

// Cloud noise function
fn noise3d(p: vec3<f32>) -> f32 {
    let p_int = floor(p);
    let p_frac = fract(p);
    
    // Simple hash-based noise
    let hash = fract(sin(dot(p_int, vec3<f32>(127.1, 311.7, 74.7))) * 43758.5453);
    return mix(hash, fract(sin(dot(p_int + vec3<f32>(1.0), vec3<f32>(127.1, 311.7, 74.7))) * 43758.5453), 
               smoothstep(0.0, 1.0, p_frac.x));
}

fn get_cloud_coverage(world_pos: vec3<f32>) -> f32 {
    let cloud_height = 60.0;
    let cloud_scale = 0.01;
    
    // Animated clouds
    let wind_offset = vec3<f32>(camera.time_of_day.x * 10.0, 0.0, camera.time_of_day.x * 5.0);
    let cloud_pos = (world_pos + wind_offset) * cloud_scale;
    
    let noise1 = noise3d(cloud_pos);
    let noise2 = noise3d(cloud_pos * 2.0) * 0.5;
    let noise3 = noise3d(cloud_pos * 4.0) * 0.25;
    
    let cloud_density = (noise1 + noise2 + noise3) - 0.5;
    return clamp(cloud_density, 0.0, 1.0);
}

@fragment  
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let sun_dir = normalize(camera.sun_direction.xyz);
    let normal = normalize(in.normal);
    let view_dir = normalize(in.view_direction);
    
    // Material properties based on material_id
    var base_color = in.color.rgb;
    var roughness = 0.8;
    var metallic = 0.0;
    var ambient_occlusion = 1.0;
    
    // Enhanced material properties
    switch (in.material_id) {
        case 1u: { // Earth
            base_color = vec3<f32>(0.4, 0.3, 0.2);
            roughness = 0.9;
            ambient_occlusion = 0.8;
        }
        case 2u: { // Stone  
            base_color = vec3<f32>(0.6, 0.6, 0.65);
            roughness = 0.7;
            ambient_occlusion = 0.9;
        }
        case 3u: { // Water
            base_color = vec3<f32>(0.1, 0.3, 0.6);
            roughness = 0.1;
            metallic = 0.9;
        }
        case 4u: { // Grass
            base_color = vec3<f32>(0.2, 0.6, 0.2);
            roughness = 0.8;
            ambient_occlusion = 0.7;
        }
        case 5u: { // Sand
            base_color = vec3<f32>(0.9, 0.8, 0.4);
            roughness = 0.9;
            ambient_occlusion = 0.85;
        }
        default: {
            base_color = in.color.rgb;
        }
    }
    
    // Sun lighting (directional light)
    let sun_intensity = max(dot(normal, sun_dir), 0.0);
    let sun_color = vec3<f32>(1.0, 0.95, 0.8) * 2.0; // Warm sunlight
    
    // Sky lighting (ambient)
    let sky_factor = clamp(normal.y * 0.5 + 0.5, 0.0, 1.0);
    let sky_color = get_sky_color(normal) * 0.3;
    
    // Ground bounce lighting
    let ground_factor = clamp(-normal.y * 0.5 + 0.5, 0.0, 1.0);
    let ground_color = vec3<f32>(0.3, 0.25, 0.2) * 0.1;
    
    // Combine lighting
    let diffuse = base_color * (
        sun_color * sun_intensity +
        sky_color * sky_factor +
        ground_color * ground_factor
    ) * ambient_occlusion;
    
    // Simple specular highlight for water and wet surfaces
    var specular = vec3<f32>(0.0);
    if (metallic > 0.5) {
        let half_vector = normalize(sun_dir + view_dir);
        let spec_power = mix(32.0, 512.0, 1.0 - roughness);
        let spec_intensity = pow(max(dot(normal, half_vector), 0.0), spec_power);
        specular = sun_color * spec_intensity * metallic;
    }
    
    // Atmospheric perspective (distance fog)
    let distance = length(camera.camera_pos.xyz - in.world_position);
    let fog_factor = clamp(1.0 - distance / 300.0, 0.0, 1.0);
    let fog_color = get_sky_color(normalize(in.world_position - camera.camera_pos.xyz)) * 0.8;
    
    // Cloud shadows
    let cloud_shadow = 1.0 - get_cloud_coverage(in.world_position) * 0.4;
    
    var final_color = (diffuse + specular) * cloud_shadow;
    final_color = mix(fog_color, final_color, fog_factor);
    
    return vec4<f32>(final_color, in.color.a);
}
"#;

// Main rendering state
struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    camera: Camera,
    camera_controller: CameraController,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    last_frame_time: Instant,
    mouse_delta: (f64, f64),
    vertex_count: u32,
    start_time: Instant,
}

impl State {
    async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        
        println!("🚀 Creating WGPU instance...");
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        println!("🖥️  Creating surface...");
        let surface = instance.create_surface(Arc::clone(&window)).unwrap();
        
        println!("🔍 Finding adapter...");
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();
        
        println!("📱 Creating device and queue...");
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        ).await.unwrap();
        
        println!("⚙️  Configuring surface...");
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
            
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        
        surface.configure(&device, &config);
        
        // Create camera
        let camera = Camera::new(config.width as f32 / config.height as f32);
        let camera_controller = CameraController::new(25.0, 0.004);
        
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(camera.build_view_projection_matrix(), camera.position);
        
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        
        println!("🎨 Creating shader...");
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Voxel Shader"),
            source: wgpu::ShaderSource::Wgsl(VOXEL_SHADER.into()),
        });
        
        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                }
            ],
            label: Some("camera_bind_group_layout"),
        });
        
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });
        
        println!("📐 Creating render pipeline...");
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout],
            push_constant_ranges: &[],
        });
        
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Voxel Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[VoxelVertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
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
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });
        
        println!("🌍 Generating voxel world...");
        let vertices = generate_voxel_world(0, 0, 2); // 2x2 chunks around origin
        let vertex_count = vertices.len() as u32;
        
        println!("📊 Creating vertex buffer ({} vertices)...", vertex_count);
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Voxel Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        println!("✅ Voxel world initialization complete!");
        
        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            camera,
            camera_controller,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            last_frame_time: Instant::now(),
            mouse_delta: (0.0, 0.0),
            vertex_count,
            start_time: Instant::now(),
        }
    }
    
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.camera.aspect = new_size.width as f32 / new_size.height as f32;
        }
    }
    
    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    physical_key,
                    state,
                    ..
                },
                ..
            } => self.camera_controller.process_keyboard(*physical_key, *state),
            _ => false,
        }
    }
    
    fn mouse_input(&mut self, delta: (f64, f64)) {
        self.mouse_delta = delta;
    }
    
    fn update(&mut self) {
        let now = Instant::now();
        let dt = (now - self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;
        
        self.camera_controller.update_camera(&mut self.camera, dt, self.mouse_delta);
        self.mouse_delta = (0.0, 0.0);
        
        // Update sun position based on time of day (1/8th real time = 3 hour day cycle)
        let elapsed_time = now.duration_since(self.start_time).as_secs_f32();
        let time_of_day = (elapsed_time / 180.0) % 1.0; // Day cycle every 3 minutes (1/8th of 24 hours)
        
        // Sun moves in an arc across the sky
        let sun_angle = time_of_day * std::f32::consts::PI * 2.0 - std::f32::consts::PI;
        let sun_direction = Vector3::new(
            sun_angle.cos() * 0.5,
            sun_angle.sin().max(0.0) + 0.2, // Keep sun above horizon
            0.3
        ).normalize();
        
        self.camera_uniform.update_view_proj(self.camera.build_view_projection_matrix(), self.camera.position);
        self.camera_uniform.update_sun(sun_direction, time_of_day);
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
    }
    
    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        // Create depth texture
        let depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: self.config.width,
                height: self.config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            label: Some("depth_texture"),
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Voxel Render Encoder"),
        });
        
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Voxel Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.4,
                            g: 0.7,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..self.vertex_count, 0..1);
        }
        
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        
        Ok(())
    }
}

async fn run() {
    env_logger::init();
    
    println!("🎮 Starting Professional Voxel World Demo");
    println!("🌍 Creating immersive voxel landscape with proper physics!");
    
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(WindowBuilder::new()
        .with_title("🚀 Robin Engine - Professional Voxel World")
        .with_inner_size(winit::dpi::LogicalSize::new(1400, 900))
        .build(&event_loop)
        .unwrap());
        
    // Capture mouse for proper FPS controls
    window.set_cursor_grab(CursorGrabMode::Confined).ok();
    window.set_cursor_visible(false);
        
    println!("🖼️  Window created! Initializing voxel engine...");
    
    let mut state = State::new(window.clone()).await;
    let mut last_mouse_pos: Option<winit::dpi::PhysicalPosition<f64>> = None;
    
    println!("🎯 Voxel world ready!");
    println!("📝 Controls:");
    println!("   WASD - Walk around terrain");
    println!("   SPACE - Jump");
    println!("   SHIFT - Run faster");
    println!("   ESC - Exit");
    println!("   Mouse - Look around");
    println!("🌟 Explore the procedurally generated outdoor landscape!");
    println!("☀️  Watch the realistic day/night cycle with moving sun and moon!");
    
    event_loop.run(move |event, target| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    state: ElementState::Pressed,
                                    physical_key: PhysicalKey::Code(KeyCode::Escape),
                                    ..
                                },
                            ..
                        } => target.exit(),
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::CursorMoved { position, .. } => {
                            if let Some(last_pos) = last_mouse_pos {
                                let delta_x = position.x - last_pos.x;
                                let delta_y = position.y - last_pos.y;
                                state.mouse_input((delta_x, delta_y));
                            }
                            last_mouse_pos = Some(*position);
                        }
                        WindowEvent::RedrawRequested => {
                            state.update();
                            match state.render() {
                                Ok(_) => {}
                                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                    state.resize(state.size)
                                }
                                Err(wgpu::SurfaceError::OutOfMemory) => {
                                    eprintln!("OutOfMemory");
                                    target.exit();
                                }
                                Err(wgpu::SurfaceError::Timeout) => {
                                    eprintln!("Surface timeout")
                                }
                            }
                        }
                        _ => {}
                    }
                }
            },
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        }
    }).unwrap();
}

fn main() {
    pollster::block_on(run());
}