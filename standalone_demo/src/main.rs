/*!
 * Robin Engine - Standalone Interactive 3D Demo
 * Optimized for Apple Silicon with Metal Backend
 *
 * Features:
 * - Real wgpu 3D graphics window on macOS
 * - Interactive first-person camera with WASD + mouse controls
 * - Voxel world construction with Engineer Build Mode
 * - Metal rendering optimization for Apple Silicon
 * - Clean architecture without external dependencies
 */

use winit::{
    event::{Event, WindowEvent, DeviceEvent, KeyEvent, ElementState, MouseButton, Modifiers},
    event_loop::EventLoop,
    window::{WindowBuilder, Window, Fullscreen, CursorGrabMode},
    keyboard::Key,
    dpi::PhysicalSize,
};
use wgpu::{Surface, SurfaceConfiguration, Device, Queue, util::DeviceExt};
use cgmath::{Matrix4, Vector3, Point3, InnerSpace, SquareMatrix, Rad, perspective};
use std::time::Instant;
use std::collections::HashMap;
use log::{info, warn, debug};
use bytemuck::{Pod, Zeroable};
use egui_wgpu::Renderer as EguiRenderer;
use egui_winit::State as EguiState;

// Module declarations
mod error;
mod error_recovery;
mod settings;
mod gpu_resources;
mod buffer_validation;
mod state_persistence;

// Error handling imports
use error::{RobinError, RobinResult, ErrorContext, IntoRobinError};
use error_recovery::{ErrorRecoverySystem, RecoveryResult};

// Settings module for configuration management
use settings::SettingsManager;

// GPU resource management
use gpu_resources::{GpuResourceManager, ManagedBuffer, ManagedResourceExt};
use std::sync::{Arc, Mutex};

// Buffer validation and overflow protection
use buffer_validation::{BufferValidator, BufferType, safe_operations};

// State persistence and crash recovery
use state_persistence::{StatePersistenceManager, StateSnapshot, SaveResult};

// Buffer size constants for safety and consistency
const VERTEX_BUFFER_SIZE: u64 = 4 * 1024 * 1024; // 4MB
const INDEX_BUFFER_SIZE: u64 = 4 * 1024 * 1024;  // 4MB
const MAX_BACKGROUND_VOXELS: usize = 3000;       // Maximum voxels for background scene (safe for 1MB buffer limit)
const MAX_VERTICES_SAFE: usize = 50000;          // Maximum vertices before early termination

// Platform detection optimized for Apple Silicon
#[derive(Debug, Clone)]
pub struct PlatformCapabilities {
    pub has_metal: bool,
    pub has_apple_silicon: bool,
    pub unified_memory: bool,
    pub max_texture_size: u32,
    pub gpu_family: String,
}

impl PlatformCapabilities {
    pub fn detect() -> Self {
        #[cfg(target_os = "macos")]
        {
            let is_apple_silicon = std::env::consts::ARCH == "aarch64";
            Self {
                has_metal: true,
                has_apple_silicon: is_apple_silicon,
                unified_memory: is_apple_silicon,
                max_texture_size: if is_apple_silicon { 16384 } else { 8192 },
                gpu_family: if is_apple_silicon {
                    "Apple M-Series".to_string()
                } else {
                    "Intel/AMD".to_string()
                },
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            Self {
                has_metal: false,
                has_apple_silicon: false,
                unified_memory: false,
                max_texture_size: 4096,
                gpu_family: "Generic".to_string(),
            }
        }
    }

    pub fn get_preferred_backend(&self) -> wgpu::Backends {
        if self.has_metal {
            wgpu::Backends::METAL
        } else {
            wgpu::Backends::VULKAN | wgpu::Backends::DX12
        }
    }
}

// VoxelType system with Apple Silicon optimized colors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VoxelType {
    Air,
    Stone,
    Wood,
    Crystal,
    Glass,
    Metal,
    Brick,
    Ice,
    Obsidian,
    Grass,
    Sand,
}

impl VoxelType {
    pub fn get_color(&self) -> [f32; 4] {
        match self {
            VoxelType::Air => [0.0, 0.0, 0.0, 0.0],
            VoxelType::Stone => [0.5, 0.5, 0.5, 1.0],
            VoxelType::Wood => [0.6, 0.4, 0.2, 1.0],
            VoxelType::Crystal => [0.8, 0.4, 0.8, 1.0],
            VoxelType::Glass => [0.9, 0.9, 0.9, 0.3],
            VoxelType::Metal => [0.7, 0.7, 0.8, 1.0],
            VoxelType::Brick => [0.8, 0.4, 0.3, 1.0],
            VoxelType::Ice => [0.8, 0.9, 1.0, 0.7],
            VoxelType::Obsidian => [0.1, 0.1, 0.1, 1.0],
            VoxelType::Grass => [0.2, 0.8, 0.2, 1.0],
            VoxelType::Sand => [0.9, 0.8, 0.6, 1.0],
        }
    }

    pub fn is_solid(&self) -> bool {
        *self != VoxelType::Air
    }

    pub fn get_all_types() -> Vec<VoxelType> {
        vec![
            VoxelType::Stone,
            VoxelType::Wood,
            VoxelType::Brick,
            VoxelType::Crystal,
            VoxelType::Metal,
            VoxelType::Glass,
            VoxelType::Ice,
            VoxelType::Obsidian,
            VoxelType::Grass,
            VoxelType::Sand,
        ]
    }
}

// Application states for menu/game flow
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ApplicationState {
    MainMenu,
    InGame,
    Settings,
    Controls,
    Paused,
}

// Engineer Build Mode system
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BuildMode {
    Single,     // Place single voxels
    Wall,       // Build walls
    Floor,      // Build floors
    Roof,       // Build roofs
    Template,   // Use templates
    Circle,     // Build circular structures
    Sphere,     // Build spherical structures
    Terrain,    // Terrain generation
    Copy,       // Copy regions
    Paste,      // Paste regions
}

impl BuildMode {
    pub fn get_all_modes() -> Vec<BuildMode> {
        vec![
            BuildMode::Single,
            BuildMode::Wall,
            BuildMode::Floor,
            BuildMode::Circle,
            BuildMode::Sphere,
            BuildMode::Template,
            BuildMode::Terrain,
            BuildMode::Copy,
            BuildMode::Paste,
        ]
    }

    pub fn get_key_binding(&self) -> char {
        match self {
            BuildMode::Single => '1',
            BuildMode::Wall => '2',
            BuildMode::Floor => '3',
            BuildMode::Roof => '4',
            BuildMode::Circle => '5',
            BuildMode::Sphere => '6',
            BuildMode::Template => '7',
            BuildMode::Terrain => '8',
            BuildMode::Copy => '9',
            BuildMode::Paste => '0',
        }
    }

    pub fn get_icon(&self) -> &'static str {
        match self {
            BuildMode::Single => "🔘",     // Single point
            BuildMode::Wall => "🧱",       // Wall/brick
            BuildMode::Floor => "⬜",      // Floor tile
            BuildMode::Roof => "🔺",       // Roof triangle
            BuildMode::Circle => "🔵",     // Circle
            BuildMode::Sphere => "🟣",     // Sphere
            BuildMode::Template => "📐",   // Template/blueprint
            BuildMode::Terrain => "🏔️",    // Mountain/terrain
            BuildMode::Copy => "📋",       // Copy clipboard
            BuildMode::Paste => "📄",      // Paste document
        }
    }

    pub fn get_color(&self) -> egui::Color32 {
        match self {
            BuildMode::Single => egui::Color32::WHITE,
            BuildMode::Wall => egui::Color32::from_rgb(100, 149, 237),  // Cornflower blue
            BuildMode::Floor => egui::Color32::from_rgb(50, 205, 50),   // Lime green
            BuildMode::Roof => egui::Color32::from_rgb(220, 20, 60),    // Crimson
            BuildMode::Circle => egui::Color32::from_rgb(255, 165, 0),  // Orange
            BuildMode::Sphere => egui::Color32::from_rgb(147, 112, 219), // Medium purple
            BuildMode::Template => egui::Color32::from_rgb(255, 215, 0), // Gold
            BuildMode::Terrain => egui::Color32::from_rgb(139, 69, 19), // Saddle brown
            BuildMode::Copy => egui::Color32::from_rgb(176, 196, 222),  // Light steel blue
            BuildMode::Paste => egui::Color32::from_rgb(144, 238, 144), // Light green
        }
    }

    pub fn get_description(&self) -> &'static str {
        match self {
            BuildMode::Single => "Place individual voxels",
            BuildMode::Wall => "Build walls between points",
            BuildMode::Floor => "Create floor/platform areas",
            BuildMode::Roof => "Construct angled roofs",
            BuildMode::Circle => "Draw circular structures",
            BuildMode::Sphere => "Create spherical volumes",
            BuildMode::Template => "Use predefined templates",
            BuildMode::Terrain => "Generate natural terrain",
            BuildMode::Copy => "Copy selected regions",
            BuildMode::Paste => "Paste copied regions",
        }
    }
}

// 3D Camera optimized for Apple Silicon performance
#[derive(Debug)]
pub struct Camera {
    pub position: Point3<f32>,
    pub direction: Vector3<f32>,
    pub up: Vector3<f32>,
    pub fovy: Rad<f32>,
    pub aspect: f32,
    pub znear: f32,
    pub zfar: f32,
    pub movement_speed: f32,
    pub rotation_speed: f32,
}

impl Camera {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            position: Point3::new(0.0, 5.0, 10.0),
            direction: Vector3::new(0.0, 0.0, -1.0),
            up: Vector3::unit_y(),
            fovy: Rad(std::f32::consts::FRAC_PI_4),
            aspect: width as f32 / height as f32,
            znear: 0.1,
            zfar: 1000.0,
            movement_speed: 8.0,  // Optimized for Apple Silicon
            rotation_speed: 0.003,
        }
    }

    pub fn get_view_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(self.position, self.position + self.direction, self.up)
    }

    pub fn get_projection_matrix(&self) -> Matrix4<f32> {
        perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }

    pub fn update_aspect_ratio(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn move_forward(&mut self, distance: f32) {
        self.position += self.direction * distance;
    }

    pub fn move_backward(&mut self, distance: f32) {
        self.position -= self.direction * distance;
    }

    pub fn strafe_left(&mut self, distance: f32) {
        let right = self.direction.cross(self.up).normalize();
        self.position -= right * distance;
    }

    pub fn strafe_right(&mut self, distance: f32) {
        let right = self.direction.cross(self.up).normalize();
        self.position += right * distance;
    }

    pub fn move_up(&mut self, distance: f32) {
        self.position += self.up * distance;
    }

    pub fn move_down(&mut self, distance: f32) {
        self.position -= self.up * distance;
    }

    pub fn rotate(&mut self, yaw: f32, pitch: f32) {
        // Rotate around Y axis (yaw)
        let yaw_rotation = Matrix4::from_angle_y(Rad(yaw));
        self.direction = (yaw_rotation * self.direction.extend(0.0)).truncate().normalize();

        // Rotate around local X axis (pitch)
        let right = self.direction.cross(self.up).normalize();
        let pitch_rotation = Matrix4::from_axis_angle(right, Rad(pitch));
        let new_direction = (pitch_rotation * self.direction.extend(0.0)).truncate().normalize();

        // Prevent camera from flipping by limiting pitch
        let up_dot = new_direction.dot(Vector3::unit_y());
        if up_dot.abs() < 0.98 {
            self.direction = new_direction;
        }
    }
}

// Voxel world optimized for Apple Silicon unified memory
#[derive(Debug)]
pub struct VoxelWorld {
    pub voxels: HashMap<(i32, i32, i32), VoxelType>,
    pub chunk_size: i32,
    pub max_height: i32,
    pub max_width: i32,
}

impl VoxelWorld {
    pub fn new() -> Self {
        let mut world = Self {
            voxels: HashMap::with_capacity(10000), // Pre-allocate for Apple Silicon
            chunk_size: 16,
            max_height: 64,
            max_width: 64,
        };

        // Create a more interesting starting world
        world.generate_terrain();
        world.add_sample_structures();

        world
    }

    fn generate_terrain(&mut self) {
        // Generate rolling hills terrain
        for x in -20..=20 {
            for z in -20..=20 {
                let height = (
                    (x as f32 * 0.1).sin() * 3.0 +
                    (z as f32 * 0.15).cos() * 2.0 +
                    ((x * x + z * z) as f32 * 0.001).sin() * 1.0
                ).max(0.0) as i32;

                // Place grass on top, stone below
                for y in 0..=height {
                    let voxel_type = if y == height && height > 0 {
                        VoxelType::Grass
                    } else if y > height - 3 {
                        VoxelType::Sand
                    } else {
                        VoxelType::Stone
                    };
                    self.set_voxel(x, y, z, voxel_type);
                }
            }
        }
    }

    fn add_sample_structures(&mut self) {
        // Add a sample tower
        for y in 1..=8 {
            self.set_voxel(5, y, 5, VoxelType::Wood);
            if y == 8 {
                // Add a crystal on top
                self.set_voxel(5, y + 1, 5, VoxelType::Crystal);
            }
        }

        // Add a brick wall
        for x in -3..=3 {
            for y in 1..=4 {
                self.set_voxel(x, y, -8, VoxelType::Brick);
            }
        }

        // Add glass windows in the wall
        self.set_voxel(-1, 2, -8, VoxelType::Glass);
        self.set_voxel(1, 2, -8, VoxelType::Glass);

        // Add an ice structure
        for x in -10..=-8 {
            for z in 8..=10 {
                for y in 1..=3 {
                    self.set_voxel(x, y, z, VoxelType::Ice);
                }
            }
        }
    }

    pub fn get_voxel(&self, x: i32, y: i32, z: i32) -> VoxelType {
        self.voxels.get(&(x, y, z)).copied().unwrap_or(VoxelType::Air)
    }

    pub fn set_voxel(&mut self, x: i32, y: i32, z: i32, voxel_type: VoxelType) {
        if voxel_type == VoxelType::Air {
            self.voxels.remove(&(x, y, z));
        } else {
            self.voxels.insert((x, y, z), voxel_type);
        }
    }

    pub fn raycast(&self, origin: Point3<f32>, direction: Vector3<f32>, max_distance: f32) -> Option<(i32, i32, i32, Vector3<f32>)> {
        let mut current = Vector3::new(origin.x, origin.y, origin.z);
        let step = direction.normalize() * 0.1;
        let mut distance = 0.0;
        let mut last_pos = (
            current.x.floor() as i32,
            current.y.floor() as i32,
            current.z.floor() as i32,
        );

        while distance < max_distance {
            let x = current.x.floor() as i32;
            let y = current.y.floor() as i32;
            let z = current.z.floor() as i32;

            if self.get_voxel(x, y, z).is_solid() {
                // Calculate face normal based on which face was hit
                let normal = Vector3::new(
                    (last_pos.0 - x) as f32,
                    (last_pos.1 - y) as f32,
                    (last_pos.2 - z) as f32,
                ).normalize();

                return Some((x, y, z, normal));
            }

            last_pos = (x, y, z);
            current += step;
            distance += 0.1;
        }

        None
    }

    pub fn place_voxel_at_surface(&mut self, hit_pos: (i32, i32, i32), normal: Vector3<f32>, voxel_type: VoxelType) -> (i32, i32, i32) {
        let place_pos = (
            hit_pos.0 + normal.x as i32,
            hit_pos.1 + normal.y as i32,
            hit_pos.2 + normal.z as i32,
        );
        self.set_voxel(place_pos.0, place_pos.1, place_pos.2, voxel_type);
        place_pos
    }

    pub fn build_with_mode(&mut self, center: (i32, i32, i32), mode: BuildMode, voxel_type: VoxelType) {
        match mode {
            BuildMode::Single => {
                self.set_voxel(center.0, center.1, center.2, voxel_type);
            }
            BuildMode::Wall => {
                for y in center.1..center.1 + 4 {
                    for x in center.0 - 2..=center.0 + 2 {
                        self.set_voxel(x, y, center.2, voxel_type);
                    }
                }
            }
            BuildMode::Floor => {
                for x in center.0 - 3..=center.0 + 3 {
                    for z in center.2 - 3..=center.2 + 3 {
                        self.set_voxel(x, center.1, z, voxel_type);
                    }
                }
            }
            BuildMode::Circle => {
                let radius = 3;
                for x in center.0 - radius..=center.0 + radius {
                    for z in center.2 - radius..=center.2 + radius {
                        let dx = x - center.0;
                        let dz = z - center.2;
                        if dx * dx + dz * dz <= radius * radius {
                            self.set_voxel(x, center.1, z, voxel_type);
                        }
                    }
                }
            }
            BuildMode::Sphere => {
                let radius = 2;
                for x in center.0 - radius..=center.0 + radius {
                    for y in center.1 - radius..=center.1 + radius {
                        for z in center.2 - radius..=center.2 + radius {
                            let dx = x - center.0;
                            let dy = y - center.1;
                            let dz = z - center.2;
                            if dx * dx + dy * dy + dz * dz <= radius * radius {
                                self.set_voxel(x, y, z, voxel_type);
                            }
                        }
                    }
                }
            }
            _ => {
                // Default to single for unimplemented modes
                self.set_voxel(center.0, center.1, center.2, voxel_type);
            }
        }
    }

    pub fn generate_cinematic_scene(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        // Create a visually interesting scene for the menu background
        // Generate rolling hills using sine waves
        for x in -30..30 {
            for z in -30..30 {
                let height = ((x as f32 * 0.1).sin() * 2.0 + (z as f32 * 0.15).cos() * 1.5) as i32 + 2;

                // Base terrain
                for y in 0..=height {
                    let voxel_type = if y == height {
                        VoxelType::Grass
                    } else if y > height - 2 {
                        VoxelType::Wood
                    } else {
                        VoxelType::Stone
                    };
                    self.set_voxel(x, y, z, voxel_type);
                }

                // Add some decorative elements
                if height > 1 && rng.gen_bool(0.05) {
                    // Crystal formations
                    for y in 0..3 {
                        self.set_voxel(x, height + 1 + y, z, VoxelType::Crystal);
                    }
                } else if rng.gen_bool(0.03) {
                    // Small structures
                    for y in 0..4 {
                        self.set_voxel(x, height + 1 + y, z, VoxelType::Brick);
                    }
                }
            }
        }

        // Add a floating island
        let island_center = (15, 15, 15);
        let island_radius = 8;
        for x in -island_radius..=island_radius {
            for z in -island_radius..=island_radius {
                for y in -2..=2 {
                    let dist = ((x*x + z*z) as f32).sqrt();
                    if dist < island_radius as f32 {
                        let thickness = ((island_radius as f32 - dist) * 0.5) as i32;
                        if (y as i32).abs() <= thickness {
                            let voxel_type = if y == thickness {
                                VoxelType::Grass
                            } else {
                                VoxelType::Stone
                            };
                            self.set_voxel(
                                island_center.0 + x,
                                island_center.1 + y,
                                island_center.2 + z,
                                voxel_type
                            );
                        }
                    }
                }
            }
        }

        // Add a castle-like structure
        let castle_pos = (-10, 3, -10);
        // Castle walls
        for x in 0..10 {
            for y in 0..8 {
                // Front and back walls
                self.set_voxel(castle_pos.0 + x, castle_pos.1 + y, castle_pos.2, VoxelType::Brick);
                self.set_voxel(castle_pos.0 + x, castle_pos.1 + y, castle_pos.2 + 10, VoxelType::Brick);
                // Side walls
                self.set_voxel(castle_pos.0, castle_pos.1 + y, castle_pos.2 + x, VoxelType::Brick);
                self.set_voxel(castle_pos.0 + 10, castle_pos.1 + y, castle_pos.2 + x, VoxelType::Brick);
            }
        }
        // Towers at corners
        for (dx, dz) in [(0, 0), (10, 0), (0, 10), (10, 10)].iter() {
            for y in 0..12 {
                self.set_voxel(castle_pos.0 + dx, castle_pos.1 + y, castle_pos.2 + dz, VoxelType::Stone);
                self.set_voxel(castle_pos.0 + dx - 1, castle_pos.1 + y, castle_pos.2 + dz, VoxelType::Stone);
                self.set_voxel(castle_pos.0 + dx, castle_pos.1 + y, castle_pos.2 + dz - 1, VoxelType::Stone);
            }
        }
    }
}

// GPU uniform data optimized for Metal
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],  // 64 bytes
    camera_pos: [f32; 4],      // 16 bytes
    light_dir: [f32; 4],       // 16 bytes
    time: f32,                 // 4 bytes
    _padding1: f32,
    _padding2: f32,
    _padding3: f32,
    _padding4: [f32; 4],
    _padding5: [f32; 4],
    _padding6: [f32; 4],
    _padding7: [f32; 4],
}

impl Uniforms {
    fn new() -> Self {
        Self {
            view_proj: Matrix4::identity().into(),
            camera_pos: [0.0, 0.0, 0.0, 1.0],
            light_dir: [0.577, 0.577, 0.577, 0.0], // Normalized (1,1,1)
            time: 0.0,
            _padding1: 0.0,
            _padding2: 0.0,
            _padding3: 0.0,
            _padding4: [0.0; 4],
            _padding5: [0.0; 4],
            _padding6: [0.0; 4],
            _padding7: [0.0; 4],
        }
    }

    fn update_view_proj(&mut self, camera: &Camera) {
        let view = camera.get_view_matrix();
        let proj = camera.get_projection_matrix();
        self.view_proj = (proj * view).into();
        self.camera_pos = [camera.position.x, camera.position.y, camera.position.z, 1.0];
    }
}

// Vertex data optimized for Apple Silicon unified memory
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
    normal: [f32; 3],
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 3]>() * 2) as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

// Main application state optimized for Apple Silicon
pub struct InteractiveDemo {
    // Graphics - optimized for Metal
    surface: Surface<'static>,
    device: Arc<Device>,
    queue: Queue,
    config: SurfaceConfiguration,
    size: PhysicalSize<u32>,

    // Rendering
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    uniforms: Uniforms,

    // Voxel rendering data
    voxel_vertices: Vec<Vertex>,
    voxel_indices: Vec<u16>,
    needs_mesh_update: bool,

    // Background rendering data for main menu
    background_vertices: Vec<Vertex>,
    background_indices: Vec<u16>,
    background_needs_update: bool,

    // Game state
    camera: Camera,
    world: VoxelWorld,
    current_build_mode: BuildMode,
    current_voxel_type: VoxelType,
    capabilities: PlatformCapabilities,

    // Input state
    keys_pressed: std::collections::HashSet<String>,
    mouse_sensitivity: f32,

    // Window management for immersive experience
    is_fullscreen: bool,
    is_cursor_captured: bool,
    cursor_visible: bool,

    // Timing - optimized for 120fps on Apple Silicon
    last_frame_time: Instant,
    target_fps: f32,

    // HUD system for professional overlay
    egui_renderer: EguiRenderer,
    egui_state: EguiState,
    show_hud: bool,

    // Mode change notification system
    mode_change_notification: Option<(BuildMode, Instant)>,
    last_mode_change: Instant,
    show_mode_selector: bool,

    // Voxel highlighting system
    highlighted_voxel: Option<(i32, i32, i32)>,
    wireframe_buffer: wgpu::Buffer,
    wireframe_vertices: Vec<Vertex>,

    // Click-to-capture system
    show_click_to_capture: bool,
    click_to_capture_alpha: f32,

    // Application state management
    app_state: ApplicationState,

    // Settings management system
    settings_manager: SettingsManager,
    settings_changed_timer: Option<std::time::Instant>,

    // Error recovery system
    error_recovery: ErrorRecoverySystem,

    // GPU resource management
    gpu_resource_manager: Arc<Mutex<GpuResourceManager>>,

    // Buffer validation and overflow protection
    buffer_validator: BufferValidator,

    // Cinematic background for menu
    background_camera: Camera,
    background_world: VoxelWorld,
    background_animation_time: f32,

    // Undo/redo system
    action_history: std::collections::VecDeque<VoxelAction>,
    redo_stack: Vec<VoxelAction>,
    last_action_notification: Option<(String, Instant)>,

    // UI accessibility
    ui_scale: f32,

    // Resolution settings
    available_resolutions: Vec<(u32, u32)>,
    selected_resolution: (u32, u32),
    pending_resize: Option<PhysicalSize<u32>>,

    // Keyboard modifiers state for shortcuts
    modifiers: Modifiers,

    // Performance mode and graceful degradation system
    performance_mode: PerformanceMode,
    memory_pressure_level: f32,
    last_memory_check: Instant,
    adaptive_quality_enabled: bool,
    original_settings: Option<QualitySettings>,
    degradation_active: bool,

    // Crash-safe state persistence system
    state_persistence: StatePersistenceManager,
    last_auto_save: Instant,
    auto_save_interval: std::time::Duration,
    state_changed: bool,
}

// Performance mode for graceful degradation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PerformanceMode {
    High,      // Full quality, all features enabled
    Medium,    // Reduced effects, some optimizations
    Low,       // Minimum quality, maximum performance
    Emergency, // Bare minimum for stability
}

impl Default for PerformanceMode {
    fn default() -> Self {
        PerformanceMode::High
    }
}

// Quality settings that can be adjusted for performance
#[derive(Debug, Clone)]
pub struct QualitySettings {
    pub render_distance: u32,
    pub texture_quality: f32,
    pub effects_enabled: bool,
    pub anti_aliasing: bool,
    pub shadows_enabled: bool,
    pub particle_count_multiplier: f32,
    pub mesh_detail_level: u32,
}

impl Default for QualitySettings {
    fn default() -> Self {
        Self {
            render_distance: 8,
            texture_quality: 1.0,
            effects_enabled: true,
            anti_aliasing: true,
            shadows_enabled: true,
            particle_count_multiplier: 1.0,
            mesh_detail_level: 2,
        }
    }
}

impl QualitySettings {
    /// Create low-performance settings for graceful degradation
    pub fn low_performance() -> Self {
        Self {
            render_distance: 4,
            texture_quality: 0.5,
            effects_enabled: false,
            anti_aliasing: false,
            shadows_enabled: false,
            particle_count_multiplier: 0.3,
            mesh_detail_level: 0,
        }
    }

    /// Create medium-performance settings
    pub fn medium_performance() -> Self {
        Self {
            render_distance: 6,
            texture_quality: 0.75,
            effects_enabled: true,
            anti_aliasing: false,
            shadows_enabled: false,
            particle_count_multiplier: 0.6,
            mesh_detail_level: 1,
        }
    }

    /// Create emergency settings for critical situations
    pub fn emergency() -> Self {
        Self {
            render_distance: 2,
            texture_quality: 0.25,
            effects_enabled: false,
            anti_aliasing: false,
            shadows_enabled: false,
            particle_count_multiplier: 0.1,
            mesh_detail_level: 0,
        }
    }
}

// Action tracking for undo/redo
#[derive(Debug, Clone, Copy)]
struct VoxelAction {
    action_type: ActionType,
    position: (i32, i32, i32),
    voxel_type: VoxelType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ActionType {
    Place,
    Remove,
}

impl InteractiveDemo {
    pub async fn new(window: &Window) -> Result<Self, Box<dyn std::error::Error>> {
        let size = window.inner_size();
        let capabilities = PlatformCapabilities::detect();

        // Create wgpu instance with Apple Silicon optimization
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: capabilities.get_preferred_backend(),
            ..Default::default()
        });

        let surface = unsafe { instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::from_window(&window)?) }?;

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: if capabilities.has_apple_silicon {
                wgpu::PowerPreference::HighPerformance
            } else {
                wgpu::PowerPreference::HighPerformance
            },
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.ok_or("Failed to find adapter")?;

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: if capabilities.has_apple_silicon {
                    wgpu::Limits {
                        max_texture_dimension_2d: capabilities.max_texture_size,
                        max_buffer_size: 1024 * 1024 * 256, // 256MB for unified memory
                        ..Default::default()
                    }
                } else {
                    wgpu::Limits::default()
                },
            },
            None,
        ).await?;

        // Wrap device in Arc for thread-safe sharing with resource managers
        let device = Arc::new(device);

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo, // V-sync for smooth performance
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Create shader module with Apple Silicon optimized shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Voxel Shader"),
            source: wgpu::ShaderSource::Wgsl(r#"
// Robin Engine - Apple Silicon Optimized Voxel Shader

struct Uniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    light_dir: vec4<f32>,
    time: f32,
    _padding1: f32,
    _padding2: f32,
    _padding3: f32,
    _padding4: vec4<f32>,
    _padding5: vec4<f32>,
    _padding6: vec4<f32>,
    _padding7: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) light_factor: f32,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.world_position = input.position;
    out.clip_position = uniforms.view_proj * vec4<f32>(input.position, 1.0);
    out.color = input.color;
    out.normal = input.normal;

    // Calculate lighting in vertex shader for Apple Silicon optimization
    let light_factor = max(dot(input.normal, uniforms.light_dir.xyz), 0.2);
    out.light_factor = light_factor;

    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Distance-based fog for depth perception
    let distance = length(input.world_position - uniforms.camera_pos.xyz);
    let fog_factor = 1.0 - min(distance / 100.0, 0.8);

    // Apply lighting and fog
    let lit_color = input.color * input.light_factor;
    let fog_color = vec3<f32>(0.6, 0.8, 1.0); // Sky blue
    let final_color = mix(fog_color, lit_color, fog_factor);

    return vec4<f32>(final_color, 1.0);
}
"#.into()),
        });

        // Create uniform buffer and bind group
        let uniforms = Uniforms::new();
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            label: Some("uniform_bind_group_layout"),
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
            label: Some("uniform_bind_group"),
        });

        // Create render pipeline optimized for Metal
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
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

        // Create initial empty buffers
        println!("🔧 Creating vertex buffer with size: {} bytes ({} MB)", VERTEX_BUFFER_SIZE, VERTEX_BUFFER_SIZE / (1024 * 1024));
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Buffer"),
            size: VERTEX_BUFFER_SIZE,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        println!("✅ Vertex buffer created successfully");

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Index Buffer"),
            size: INDEX_BUFFER_SIZE,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create wireframe buffer for block highlighting
        let wireframe_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Wireframe Buffer"),
            size: std::mem::size_of::<Vertex>() as u64 * 24, // 24 vertices for cube edges
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let camera = Camera::new(size.width, size.height);
        let world = VoxelWorld::new();

        // Calculate target FPS based on platform before moving capabilities
        let target_fps = if capabilities.has_apple_silicon { 120.0 } else { 60.0 };

        // Initialize egui for HUD system
        let egui_ctx = egui::Context::default();
        let egui_state = EguiState::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            window,
            None,
            None,
        );
        let egui_renderer = EguiRenderer::new(
            &device,
            config.format,
            None,
            1,
        );

        // Create settings manager first so we can use it in initialization
        let settings_manager = SettingsManager::new("config/robin_settings.toml");

        let mut demo = Self {
            surface,
            device: device.clone(),
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            uniform_bind_group,
            uniforms,
            voxel_vertices: Vec::new(),
            voxel_indices: Vec::new(),
            needs_mesh_update: true,

            // Initialize background rendering
            background_vertices: Vec::new(),
            background_indices: Vec::new(),
            background_needs_update: true,
            camera,
            world,
            current_build_mode: BuildMode::Single,
            current_voxel_type: VoxelType::Wood,
            capabilities,
            keys_pressed: std::collections::HashSet::new(),
            mouse_sensitivity: settings_manager.settings.controls.mouse_sensitivity,
            is_fullscreen: false, // Start windowed, then set fullscreen properly
            is_cursor_captured: false, // Start with cursor free, then capture properly
            cursor_visible: true, // Start with cursor visible, will be hidden when captured
            last_frame_time: Instant::now(),
            target_fps,
            egui_renderer,
            egui_state,
            show_hud: true, // Start with HUD visible
            mode_change_notification: None,
            last_mode_change: Instant::now(),
            show_mode_selector: false,
            highlighted_voxel: None,
            wireframe_buffer,
            wireframe_vertices: Vec::new(),
            show_click_to_capture: false,
            click_to_capture_alpha: 0.0,
            app_state: ApplicationState::MainMenu,  // Start in main menu
            settings_manager,
            settings_changed_timer: None,
            error_recovery: ErrorRecoverySystem::new(),

            // Initialize buffer validator with safety limits
            buffer_validator: BufferValidator::new(),

            // Initialize cinematic background system
            background_camera: {
                let mut cam = Camera::new(size.width, size.height);
                cam.position = cgmath::Point3::new(15.0, 8.0, 15.0);
                cam.direction = cgmath::Vector3::new(-0.5, -0.2, -0.5).normalize();
                cam
            },
            background_world: VoxelWorld::new(),
            background_animation_time: 0.0,

            // Initialize undo/redo system
            action_history: std::collections::VecDeque::with_capacity(10),
            redo_stack: Vec::new(),
            last_action_notification: None,

            // Initialize UI accessibility
            ui_scale: 1.0,

            // Initialize resolution settings
            available_resolutions: vec![
                (1280, 720),
                (1440, 900),
                (1680, 1050),
                (1920, 1080),
                (2560, 1440),
                (3840, 2160),
            ],
            selected_resolution: (size.width, size.height),
            pending_resize: None,
            modifiers: Modifiers::default(),

            // Initialize GPU resource manager (device is already Arc'd)
            gpu_resource_manager: GpuResourceManager::new(device.clone()),

            // Performance mode and graceful degradation system
            performance_mode: PerformanceMode::default(),
            memory_pressure_level: 0.0,
            last_memory_check: Instant::now(),
            adaptive_quality_enabled: true,
            original_settings: None,
            degradation_active: false,

            // Crash-safe state persistence system
            state_persistence: StatePersistenceManager::new("./state")?,
            last_auto_save: Instant::now(),
            auto_save_interval: std::time::Duration::from_secs(30), // Auto-save every 30 seconds
            state_changed: false,
        };

        // Create application lock to detect crashes
        demo.state_persistence.create_lock()?;

        // Attempt to load previous state on startup
        if let Ok(saved_state) = demo.state_persistence.load_state() {
            info!("📂 Loading saved state from: {}", saved_state.timestamp);
            demo.load_from_state_snapshot(&saved_state)?;
        } else {
            info!("🆕 Starting with fresh state - no saved state found");
        }

        // Generate initial mesh
        demo.update_voxel_mesh();

        // Generate cinematic background scene for main menu
        demo.background_world.generate_cinematic_scene();
        demo.update_background_mesh();

        Ok(demo)
    }

    fn update_voxel_mesh(&mut self) {
        self.voxel_vertices.clear();
        self.voxel_indices.clear();

        // Collect voxel data to avoid borrowing issues
        let voxel_data: Vec<((i32, i32, i32), VoxelType)> = self.world.voxels
            .iter()
            .map(|(&pos, &voxel_type)| (pos, voxel_type))
            .filter(|(_, voxel_type)| voxel_type.is_solid())
            .collect();

        // Generate optimized mesh for visible voxels
        for ((x, y, z), voxel_type) in voxel_data {
            self.add_voxel_to_mesh(x, y, z, voxel_type);
        }

        // Update GPU buffers with safety checks
        if !self.voxel_vertices.is_empty() {
            let vertex_data = bytemuck::cast_slice(&self.voxel_vertices);
            let vertex_buffer_size = VERTEX_BUFFER_SIZE as usize;

            if vertex_data.len() <= vertex_buffer_size {
                self.queue.write_buffer(
                    &self.vertex_buffer,
                    0,
                    vertex_data,
                );
                println!("✅ Voxel mesh: {} vertices ({} bytes)", self.voxel_vertices.len(), vertex_data.len());
            } else {
                println!("⚠️  Voxel mesh too large! {} bytes > {} bytes buffer. Truncating voxels for safety.", vertex_data.len(), vertex_buffer_size);
                // Truncate vertices to fit buffer
                let max_vertices = vertex_buffer_size / std::mem::size_of::<Vertex>();
                if max_vertices > 0 && max_vertices <= self.voxel_vertices.len() {
                    let truncated_data = bytemuck::cast_slice(&self.voxel_vertices[0..max_vertices]);
                    self.queue.write_buffer(
                        &self.vertex_buffer,
                        0,
                        truncated_data,
                    );
                    println!("✅ Truncated voxel mesh: {} vertices ({} bytes)", max_vertices, truncated_data.len());
                }
            }
        }

        if !self.voxel_indices.is_empty() {
            let index_data = bytemuck::cast_slice(&self.voxel_indices);
            let index_buffer_size = INDEX_BUFFER_SIZE as usize;

            if index_data.len() <= index_buffer_size {
                self.queue.write_buffer(
                    &self.index_buffer,
                    0,
                    index_data,
                );
                println!("✅ Voxel indices: {} indices ({} bytes)", self.voxel_indices.len(), index_data.len());
            } else {
                println!("⚠️  Voxel indices too large! {} bytes > {} bytes buffer. Truncating indices for safety.", index_data.len(), index_buffer_size);
                // Truncate indices to fit buffer
                let max_indices = index_buffer_size / std::mem::size_of::<u32>();
                if max_indices > 0 && max_indices <= self.voxel_indices.len() {
                    let truncated_data = bytemuck::cast_slice(&self.voxel_indices[0..max_indices]);
                    self.queue.write_buffer(
                        &self.index_buffer,
                        0,
                        truncated_data,
                    );
                    println!("✅ Truncated voxel indices: {} indices ({} bytes)", max_indices, truncated_data.len());
                }
            }
        }

        self.needs_mesh_update = false;
    }

    fn update_background_mesh(&mut self) {
        self.background_vertices.clear();
        self.background_indices.clear();

        // Collect background voxel data with safety limits
        let voxel_data: Vec<((i32, i32, i32), VoxelType)> = self.background_world.voxels
            .iter()
            .map(|(&pos, &voxel_type)| (pos, voxel_type))
            .filter(|(_, voxel_type)| voxel_type.is_solid())
            .take(MAX_BACKGROUND_VOXELS) // Safety limit: maximum voxels to prevent buffer overflow
            .collect();

        println!("🌍 Generating background mesh with {} voxels (max {} for safety)", voxel_data.len(), MAX_BACKGROUND_VOXELS);

        // Generate mesh for background voxels with progress tracking
        for (i, ((x, y, z), voxel_type)) in voxel_data.iter().enumerate() {
            self.add_background_voxel_to_mesh(*x, *y, *z, *voxel_type);

            // Check for early termination if mesh becomes too large during generation
            if self.background_vertices.len() > MAX_VERTICES_SAFE { // Safety limit for vertex count
                println!("⚠️  Background mesh exceeding safe size at voxel {}/{}. Stopping generation.", i + 1, voxel_data.len());
                break;
            }
        }

        // Update GPU buffers with background mesh data - with buffer overflow protection
        if !self.background_vertices.is_empty() {
            let vertex_data = bytemuck::cast_slice(&self.background_vertices);
            let vertex_buffer_size = VERTEX_BUFFER_SIZE as usize;

            if vertex_data.len() <= vertex_buffer_size {
                self.queue.write_buffer(
                    &self.vertex_buffer,
                    0,
                    vertex_data,
                );
                println!("✅ Background mesh: {} vertices ({} bytes)",
                    self.background_vertices.len(), vertex_data.len());
            } else {
                println!("⚠️  Background mesh too large! {} bytes > {} bytes buffer. Skipping background mesh.",
                    vertex_data.len(), vertex_buffer_size);
                println!("🔧 Consider implementing mesh simplification or LOD system.");
                // Clear the oversized mesh to prevent further issues
                self.background_vertices.clear();
                self.background_indices.clear();
                return;
            }
        }
        if !self.background_indices.is_empty() {
            let index_data = bytemuck::cast_slice(&self.background_indices);
            let index_buffer_size = INDEX_BUFFER_SIZE as usize;

            if index_data.len() <= index_buffer_size {
                self.queue.write_buffer(
                    &self.index_buffer,
                    0,
                    index_data,
                );
            } else {
                println!("⚠️  Background indices too large! {} bytes > {} bytes buffer. Skipping background indices.",
                    index_data.len(), index_buffer_size);
                // Clear the oversized mesh to prevent further issues
                self.background_vertices.clear();
                self.background_indices.clear();
                return;
            }
        }

        self.background_needs_update = false;
    }

    fn add_background_voxel_to_mesh(&mut self, x: i32, y: i32, z: i32, voxel_type: VoxelType) {
        let color = voxel_type.get_color();
        let color = [color[0], color[1], color[2]]; // RGB only

        let start_index = self.background_vertices.len() as u16;

        // Generate faces for the background voxel (same logic as regular voxels)
        let faces = [
            // Front face
            ([0.0, 0.0, 1.0], [(0.0, 0.0, 1.0), (1.0, 0.0, 1.0), (1.0, 1.0, 1.0), (0.0, 1.0, 1.0)]),
            // Back face
            ([0.0, 0.0, -1.0], [(1.0, 0.0, 0.0), (0.0, 0.0, 0.0), (0.0, 1.0, 0.0), (1.0, 1.0, 0.0)]),
            // Right face
            ([1.0, 0.0, 0.0], [(1.0, 0.0, 1.0), (1.0, 0.0, 0.0), (1.0, 1.0, 0.0), (1.0, 1.0, 1.0)]),
            // Left face
            ([-1.0, 0.0, 0.0], [(0.0, 0.0, 0.0), (0.0, 0.0, 1.0), (0.0, 1.0, 1.0), (0.0, 1.0, 0.0)]),
            // Top face
            ([0.0, 1.0, 0.0], [(0.0, 1.0, 1.0), (1.0, 1.0, 1.0), (1.0, 1.0, 0.0), (0.0, 1.0, 0.0)]),
            // Bottom face
            ([0.0, -1.0, 0.0], [(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (1.0, 0.0, 1.0), (0.0, 0.0, 1.0)]),
        ];

        for (normal, positions) in faces {
            // Check if this face should be visible (skip if adjacent voxel exists)
            let (nx, ny, nz) = (normal[0] as i32, normal[1] as i32, normal[2] as i32);
            if self.background_world.get_voxel(x + nx, y + ny, z + nz).is_solid() {
                continue; // Skip this face
            }

            // Add vertices for this face
            for (px, py, pz) in positions {
                self.background_vertices.push(Vertex {
                    position: [x as f32 + px, y as f32 + py, z as f32 + pz],
                    color,
                    normal,
                });
            }

            // Add indices for the two triangles of this face
            self.background_indices.extend_from_slice(&[
                start_index, start_index + 1, start_index + 2,
                start_index, start_index + 2, start_index + 3,
            ]);
        }
    }

    fn add_voxel_to_mesh(&mut self, x: i32, y: i32, z: i32, voxel_type: VoxelType) {
        let color = voxel_type.get_color();
        let color = [color[0], color[1], color[2]]; // RGB only

        let x = x as f32;
        let y = y as f32;
        let z = z as f32;

        let base_vertex = self.voxel_vertices.len() as u16;

        // Add vertices for a cube with proper normals
        let cube_vertices = [
            // Front face (z = z + 0.5)
            Vertex { position: [x - 0.5, y - 0.5, z + 0.5], color, normal: [0.0, 0.0, 1.0] },
            Vertex { position: [x + 0.5, y - 0.5, z + 0.5], color, normal: [0.0, 0.0, 1.0] },
            Vertex { position: [x + 0.5, y + 0.5, z + 0.5], color, normal: [0.0, 0.0, 1.0] },
            Vertex { position: [x - 0.5, y + 0.5, z + 0.5], color, normal: [0.0, 0.0, 1.0] },

            // Back face (z = z - 0.5)
            Vertex { position: [x - 0.5, y - 0.5, z - 0.5], color, normal: [0.0, 0.0, -1.0] },
            Vertex { position: [x + 0.5, y - 0.5, z - 0.5], color, normal: [0.0, 0.0, -1.0] },
            Vertex { position: [x + 0.5, y + 0.5, z - 0.5], color, normal: [0.0, 0.0, -1.0] },
            Vertex { position: [x - 0.5, y + 0.5, z - 0.5], color, normal: [0.0, 0.0, -1.0] },
        ];

        self.voxel_vertices.extend_from_slice(&cube_vertices);

        // Add indices for cube faces
        let cube_indices = [
            // Front face
            0, 1, 2,  2, 3, 0,
            // Back face
            4, 6, 5,  6, 4, 7,
            // Left face
            4, 0, 3,  3, 7, 4,
            // Right face
            1, 5, 6,  6, 2, 1,
            // Top face
            3, 2, 6,  6, 7, 3,
            // Bottom face
            4, 5, 1,  1, 0, 4,
        ];

        for index in cube_indices.iter() {
            self.voxel_indices.push(base_vertex + index);
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.camera.update_aspect_ratio(new_size.width, new_size.height);
        }
    }

    /// Load application state from a saved snapshot
    fn load_from_state_snapshot(&mut self, snapshot: &StateSnapshot) -> RobinResult<()> {
        info!("🔄 Loading state snapshot from version: {}", snapshot.version);

        // Load camera state
        self.camera.position = Point3::new(
            snapshot.camera_state.position[0],
            snapshot.camera_state.position[1],
            snapshot.camera_state.position[2],
        );
        self.camera.movement_speed = snapshot.camera_state.move_speed;
        self.mouse_sensitivity = snapshot.camera_state.mouse_sensitivity;

        // Update camera rotation (yaw, pitch)
        let yaw = snapshot.camera_state.rotation[0];
        let pitch = snapshot.camera_state.rotation[1];
        self.camera.direction = Vector3::new(
            yaw.cos() * pitch.cos(),
            pitch.sin(),
            yaw.sin() * pitch.cos(),
        ).normalize();

        // Load build state
        self.current_build_mode = match snapshot.build_state.current_build_mode {
            0 => BuildMode::Single,
            1 => BuildMode::Wall,
            2 => BuildMode::Floor,
            3 => BuildMode::Roof,
            4 => BuildMode::Circle,
            5 => BuildMode::Sphere,
            6 => BuildMode::Template,
            7 => BuildMode::Terrain,
            8 => BuildMode::Copy,
            9 => BuildMode::Paste,
            _ => BuildMode::Single,
        };

        self.current_voxel_type = match snapshot.build_state.current_voxel_type {
            0 => VoxelType::Air,
            1 => VoxelType::Stone,  // Earth -> Stone
            2 => VoxelType::Stone,
            3 => VoxelType::Ice,    // Water -> Ice
            4 => VoxelType::Grass,
            5 => VoxelType::Sand,
            6 => VoxelType::Wood,
            7 => VoxelType::Metal,
            _ => VoxelType::Stone,  // Earth -> Stone
        };

        self.show_hud = snapshot.build_state.show_hud;
        self.show_mode_selector = snapshot.build_state.show_mode_selector;

        // Load world state
        self.world.voxels.clear();
        for voxel_data in &snapshot.world_state.voxel_data {
            let pos = (voxel_data.position[0], voxel_data.position[1], voxel_data.position[2]);
            let voxel_type = match voxel_data.voxel_type {
                0 => VoxelType::Air,
                1 => VoxelType::Stone,  // Earth -> Stone
                2 => VoxelType::Stone,
                3 => VoxelType::Ice,    // Water -> Ice
                4 => VoxelType::Grass,
                5 => VoxelType::Sand,
                6 => VoxelType::Wood,
                7 => VoxelType::Metal,
                _ => VoxelType::Stone,  // Earth -> Stone
            };
            if voxel_type.is_solid() {
                self.world.voxels.insert(pos, voxel_type);
            }
        }

        // Load performance state
        self.performance_mode = match snapshot.performance_state.performance_mode {
            0 => PerformanceMode::High,
            1 => PerformanceMode::Medium,
            2 => PerformanceMode::Low,
            3 => PerformanceMode::Emergency,
            _ => PerformanceMode::High,
        };
        self.adaptive_quality_enabled = snapshot.performance_state.adaptive_quality_enabled;
        self.degradation_active = snapshot.performance_state.degradation_active;
        self.memory_pressure_level = snapshot.performance_state.memory_pressure_level;

        // Trigger mesh update to reflect loaded world
        self.needs_mesh_update = true;
        self.update_voxel_mesh();

        info!("✅ State loaded successfully: {} voxels, camera at ({:.1}, {:.1}, {:.1})",
              self.world.voxels.len(),
              self.camera.position.x,
              self.camera.position.y,
              self.camera.position.z);

        Ok(())
    }

    /// Create a state snapshot from current application state
    fn create_state_snapshot(&self) -> StateSnapshot {
        use crate::state_persistence::*;

        let camera_state = CameraState {
            position: [self.camera.position.x, self.camera.position.y, self.camera.position.z],
            rotation: [
                self.camera.direction.x.atan2(self.camera.direction.z), // yaw
                self.camera.direction.y.asin(), // pitch
            ],
            move_speed: self.camera.movement_speed,
            mouse_sensitivity: self.mouse_sensitivity,
        };

        let world_state = WorldState {
            voxel_data: self.world.voxels.iter().map(|(&pos, &voxel_type)| {
                VoxelData {
                    position: [pos.0, pos.1, pos.2],
                    voxel_type: voxel_type as u8,
                }
            }).collect(),
            world_seed: 12345, // Default seed for now
            last_modified: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        let build_state = BuildState {
            current_build_mode: self.current_build_mode as u8,
            current_voxel_type: self.current_voxel_type as u8,
            show_hud: self.show_hud,
            show_mode_selector: self.show_mode_selector,
        };

        let performance_state = PerformanceState {
            performance_mode: self.performance_mode as u8,
            adaptive_quality_enabled: self.adaptive_quality_enabled,
            degradation_active: self.degradation_active,
            memory_pressure_level: self.memory_pressure_level,
        };

        StateSnapshot {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            camera_state,
            world_state,
            build_state,
            performance_state,
            checksum: 0, // Will be calculated during save
        }
    }

    /// Perform clean shutdown with final save and cleanup
    pub fn shutdown(&mut self) -> RobinResult<()> {
        info!("🔄 Performing clean shutdown...");

        // Force save current state before shutdown
        if self.state_changed {
            let snapshot = self.create_state_snapshot();
            if let Err(error) = self.state_persistence.force_save_state(&snapshot) {
                warn!("⚠️ Failed to save state during shutdown: {}", error);
            } else {
                info!("💾 Final state saved successfully");
            }
        }

        // Remove crash detection lock file
        if let Err(error) = self.state_persistence.remove_lock() {
            warn!("⚠️ Failed to remove lock file during shutdown: {}", error);
        } else {
            info!("🔓 Clean shutdown lock removed");
        }

        info!("✅ Clean shutdown completed");
        Ok(())
    }

    pub fn input(&mut self, event: &WindowEvent, window: &Window) -> bool {
        // Let egui handle the event first if HUD is shown
        if self.show_hud {
            let response = self.egui_state.on_window_event(window, event);
            if response.consumed {
                return true;
            }
        }

        match event {
            WindowEvent::ModifiersChanged(new_modifiers) => {
                self.modifiers = *new_modifiers;
                return false;
            }
            WindowEvent::KeyboardInput { event: KeyEvent { logical_key, state, .. }, .. } => {
                let key_str = format!("{:?}", logical_key);
                match state {
                    ElementState::Pressed => {
                        self.keys_pressed.insert(key_str.clone());

                        // Handle special keys for window management
                        match logical_key {
                            Key::Named(winit::keyboard::NamedKey::Escape) => {
                                // ESC behavior depends on current state
                                match self.app_state {
                                    ApplicationState::InGame => {
                                        // Pause the game
                                        self.transition_to_state(ApplicationState::Paused, window);
                                    }
                                    ApplicationState::Paused => {
                                        // Resume the game
                                        self.transition_to_state(ApplicationState::InGame, window);
                                    }
                                    ApplicationState::Settings | ApplicationState::Controls => {
                                        // Go back to previous state (main menu or paused)
                                        self.transition_to_state(ApplicationState::MainMenu, window);
                                    }
                                    ApplicationState::MainMenu => {
                                        // Do nothing on ESC in main menu
                                    }
                                }
                                return true;
                            }
                            Key::Named(winit::keyboard::NamedKey::Tab) => {
                                // Toggle mode selector overlay
                                self.show_mode_selector = !self.show_mode_selector;
                                println!("🎯 Mode Selector {}", if self.show_mode_selector { "opened" } else { "closed" });
                                return true;
                            }
                            Key::Character(ref s) if s == "h" || s == "H" => {
                                // Toggle HUD display
                                self.show_hud = !self.show_hud;
                                println!("🎯 HUD {}", if self.show_hud { "enabled" } else { "disabled" });
                                return true;
                            }
                            Key::Character(ref s) if s == "z" || s == "Z" => {
                                // Z: Undo last action (will add CMD modifier later)
                                if self.app_state == ApplicationState::InGame {
                                    if self.undo_last_action() {
                                        println!("⚡ Undo successful");
                                    } else {
                                        println!("⚠️ Nothing to undo");
                                    }
                                }
                                return true;
                            }
                            Key::Character(ref s) if s == "u" || s == "U" => {
                                // U: Redo last action (using U instead of Y to avoid conflict)
                                if self.app_state == ApplicationState::InGame {
                                    if self.redo_last_action() {
                                        println!("⚡ Redo successful");
                                    } else {
                                        println!("⚠️ Nothing to redo");
                                    }
                                }
                                return true;
                            }
                            Key::Character(ref s) if s == "c" || s == "C" => {
                                // C: Toggle cursor capture (simplified - will add CMD+SHIFT later)
                                if self.app_state == ApplicationState::InGame {
                                    self.toggle_cursor_capture(window);
                                    println!("🎯 Cursor capture toggled: {}", self.is_cursor_captured);
                                }
                                return true;
                            }
                            Key::Character(c) => {
                                let ch = c.chars().next().unwrap_or('\0');
                                // Build mode switching with notifications
                                let old_mode = self.current_build_mode;
                                match ch {
                                    '1' => {
                                        self.current_build_mode = BuildMode::Single;
                                        if old_mode != self.current_build_mode {
                                            self.trigger_mode_change_notification(self.current_build_mode);
                                        }
                                    },
                                    '2' => {
                                        self.current_build_mode = BuildMode::Wall;
                                        if old_mode != self.current_build_mode {
                                            self.trigger_mode_change_notification(self.current_build_mode);
                                        }
                                    },
                                    '3' => {
                                        self.current_build_mode = BuildMode::Floor;
                                        if old_mode != self.current_build_mode {
                                            self.trigger_mode_change_notification(self.current_build_mode);
                                        }
                                    },
                                    '4' => {
                                        self.current_build_mode = BuildMode::Circle;
                                        if old_mode != self.current_build_mode {
                                            self.trigger_mode_change_notification(self.current_build_mode);
                                        }
                                    },
                                    '5' => {
                                        self.current_build_mode = BuildMode::Sphere;
                                        if old_mode != self.current_build_mode {
                                            self.trigger_mode_change_notification(self.current_build_mode);
                                        }
                                    },
                                    '6' => {
                                        self.current_build_mode = BuildMode::Template;
                                        if old_mode != self.current_build_mode {
                                            self.trigger_mode_change_notification(self.current_build_mode);
                                        }
                                    },
                                    '7' => {
                                        self.current_build_mode = BuildMode::Terrain;
                                        if old_mode != self.current_build_mode {
                                            self.trigger_mode_change_notification(self.current_build_mode);
                                        }
                                    },
                                    '8' => {
                                        self.current_build_mode = BuildMode::Copy;
                                        if old_mode != self.current_build_mode {
                                            self.trigger_mode_change_notification(self.current_build_mode);
                                        }
                                    },
                                    '9' => {
                                        self.current_build_mode = BuildMode::Paste;
                                        if old_mode != self.current_build_mode {
                                            self.trigger_mode_change_notification(self.current_build_mode);
                                        }
                                    },
                                    // Voxel type switching
                                    'q' => self.current_voxel_type = VoxelType::Stone,
                                    'e' => self.current_voxel_type = VoxelType::Wood,
                                    'r' => self.current_voxel_type = VoxelType::Brick,
                                    't' => self.current_voxel_type = VoxelType::Crystal,
                                    'y' => self.current_voxel_type = VoxelType::Metal,
                                    'u' => self.current_voxel_type = VoxelType::Glass,
                                    'i' => self.current_voxel_type = VoxelType::Ice,
                                    'o' => self.current_voxel_type = VoxelType::Obsidian,
                                    'p' => self.current_voxel_type = VoxelType::Grass,
                                    'l' => self.current_voxel_type = VoxelType::Sand,
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                        true
                    }
                    ElementState::Released => {
                        self.keys_pressed.remove(&key_str);
                        true
                    }
                }
            }
            WindowEvent::MouseInput { button: MouseButton::Left, state, .. } => {
                // Only handle mouse building when in game
                if self.app_state != ApplicationState::InGame {
                    return false;
                }

                if *state == ElementState::Pressed {
                    // If cursor is not captured, re-capture it instead of building
                    if !self.is_cursor_captured {
                        self.set_cursor_capture(window, true);
                        return true;
                    }

                    // Perform raycast and place/remove voxel (only when cursor is captured)
                    if let Some((x, y, z, normal)) = self.world.raycast(
                        self.camera.position,
                        self.camera.direction,
                        50.0
                    ) {
                        if self.keys_pressed.contains("Character(\"x\")") {
                            // Remove mode (X key held)
                            let existing_voxel = self.world.get_voxel(x, y, z);
                            if existing_voxel.is_solid() {
                                self.record_action(ActionType::Remove, (x, y, z), existing_voxel);
                                self.world.set_voxel(x, y, z, VoxelType::Air);
                            }
                        } else {
                            // Place mode
                            let place_pos = self.world.place_voxel_at_surface((x, y, z), normal, self.current_voxel_type);

                            // Record the placement action
                            self.record_action(ActionType::Place, place_pos, self.current_voxel_type);

                            // Use current build mode
                            if self.current_build_mode != BuildMode::Single {
                                self.world.build_with_mode(place_pos, self.current_build_mode, self.current_voxel_type);
                            }
                        }
                        self.needs_mesh_update = true;
                        self.state_changed = true; // Mark state for auto-save
                    }
                }
                true
            }
            _ => false,
        }
    }

    pub fn device_input(&mut self, event: &DeviceEvent) -> bool {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                // Only allow camera rotation when in game (not paused)
                if self.app_state == ApplicationState::InGame {
                    self.camera.rotate(
                        -delta.0 as f32 * self.mouse_sensitivity,
                        -delta.1 as f32 * self.mouse_sensitivity,
                    );
                }
                true
            }
            _ => false,
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;

        // Only update game logic when in game or paused states
        if self.app_state == ApplicationState::InGame || self.app_state == ApplicationState::Paused {
            let move_distance = self.camera.movement_speed * dt;

            // Only process movement when actually in game (not paused)
            if self.app_state == ApplicationState::InGame {
                // Handle keyboard movement
                if self.keys_pressed.contains("Character(\"w\")") || self.keys_pressed.contains("Named(ArrowUp)") {
                    self.camera.move_forward(move_distance);
                }
                if self.keys_pressed.contains("Character(\"s\")") || self.keys_pressed.contains("Named(ArrowDown)") {
                    self.camera.move_backward(move_distance);
                }
                if self.keys_pressed.contains("Character(\"a\")") || self.keys_pressed.contains("Named(ArrowLeft)") {
                    self.camera.strafe_left(move_distance);
                }
                if self.keys_pressed.contains("Character(\"d\")") || self.keys_pressed.contains("Named(ArrowRight)") {
                    self.camera.strafe_right(move_distance);
                }
                if self.keys_pressed.contains("Named(Space)") {
                    self.camera.move_up(move_distance);
                }
                if self.keys_pressed.contains("Named(ControlLeft)") {
                    self.camera.move_down(move_distance);
                }
            } // End of InGame-only movement

            // Update highlighted voxel based on camera direction
            self.highlighted_voxel = self.world.raycast(
                self.camera.position,
                self.camera.direction,
                50.0
            ).map(|(x, y, z, _)| (x, y, z));

            // Update wireframe vertices if highlighted voxel changed
            if let Some((x, y, z)) = self.highlighted_voxel {
                self.wireframe_vertices = Self::get_wireframe_vertices(x, y, z);
                self.queue.write_buffer(
                    &self.wireframe_buffer,
                    0,
                    bytemuck::cast_slice(&self.wireframe_vertices),
                );
            } else {
                self.wireframe_vertices.clear();
            }

            // Update mesh if needed
            if self.needs_mesh_update {
                self.update_voxel_mesh();
            }
        } // End of game/paused state updates

        // Update background camera animation for main menu
        if self.app_state == ApplicationState::MainMenu {
            self.background_animation_time += dt * 0.5; // Slow, cinematic movement

            // Smooth circular camera movement around the scene
            let radius = 20.0;
            let height = 12.0;
            let x = radius * self.background_animation_time.sin();
            let z = radius * self.background_animation_time.cos();

            self.background_camera.position = cgmath::Point3::new(x, height, z);

            // Always look towards the center of the scene
            let look_at = cgmath::Point3::new(0.0, 5.0, 0.0);
            let direction = (look_at - self.background_camera.position).normalize();
            self.background_camera.direction = direction;
        }

        // Update click-to-capture animation
        if self.show_click_to_capture {
            self.click_to_capture_alpha = (self.click_to_capture_alpha + dt * 2.0).min(1.0);
        } else {
            self.click_to_capture_alpha = (self.click_to_capture_alpha - dt * 4.0).max(0.0);
        }

        // Update uniforms with appropriate camera based on app state
        let camera = if self.app_state == ApplicationState::MainMenu {
            &self.background_camera
        } else {
            &self.camera
        };
        self.uniforms.update_view_proj(camera);
        self.uniforms.time = now.elapsed().as_secs_f32();

        // Safely write uniform buffer with overflow protection
        if let Err(error) = self.safe_write_buffer(&self.uniform_buffer, 0, &[self.uniforms]) {
            log::error!("🚨 Failed to write uniform buffer: {}", error);
            // Continue rendering with potentially stale uniforms rather than crashing
        }

        // Auto-save functionality with debouncing
        if self.state_changed && now.duration_since(self.last_auto_save) >= self.auto_save_interval {
            let snapshot = self.create_state_snapshot();
            match self.state_persistence.save_state(&snapshot) {
                crate::state_persistence::SaveResult::Success => {
                    self.last_auto_save = now;
                    self.state_changed = false;
                    debug!("💾 Auto-save completed successfully");
                }
                crate::state_persistence::SaveResult::Debounced => {
                    // Save was debounced, that's fine - will try again next interval
                }
                crate::state_persistence::SaveResult::Failed(error) => {
                    warn!("⚠️ Auto-save failed: {}", error);
                    // Don't reset state_changed, will retry next interval
                }
            }
        }
    }

    pub fn render(&mut self, window: &Window) -> Result<(), wgpu::SurfaceError> {
        // Check memory pressure and adapt performance if needed
        self.check_and_adapt_performance();

        // Safely acquire surface texture with error recovery
        let output = match self.safe_get_surface_texture() {
            Ok(texture) => texture,
            Err(error) => {
                log::error!("🚨 Failed to acquire surface texture: {}", error);
                // Return a SurfaceError for compatibility, but log the detailed error
                return Err(wgpu::SurfaceError::Lost);
            }
        };
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
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.4,
                            g: 0.6,
                            b: 0.9,
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
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            // Render appropriate world based on app state
            if self.app_state == ApplicationState::MainMenu {
                // Render background world for main menu
                if !self.background_indices.is_empty() {
                    render_pass.draw_indexed(0..self.background_indices.len() as u32, 0, 0..1);
                }
            } else {
                // Render main game world
                if !self.voxel_indices.is_empty() {
                    render_pass.draw_indexed(0..self.voxel_indices.len() as u32, 0, 0..1);
                }
            }

            // Render wireframe for highlighted voxel (only in game mode)
            if self.app_state == ApplicationState::InGame && !self.wireframe_vertices.is_empty() {
                render_pass.set_vertex_buffer(0, self.wireframe_buffer.slice(..));
                render_pass.draw(0..self.wireframe_vertices.len() as u32, 0..1);
            }
        }

        // Render HUD overlay
        if self.show_hud {
            self.render_hud(&view, &mut encoder, window);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        // Apply any pending resize operations now that the frame is complete
        if let Some(pending_size) = self.pending_resize.take() {
            let _ = window.request_inner_size(pending_size);
            self.resize(pending_size);
            println!("✅ Resolution applied: {}x{}", pending_size.width, pending_size.height);
        }

        Ok(())
    }

    fn render_hud(&mut self, view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder, window: &Window) {
        let egui_input = self.egui_state.take_egui_input(window);
        let egui_ctx = self.egui_state.egui_ctx();

        // Apply UI scale for accessibility
        egui_ctx.set_pixels_per_point(self.ui_scale);

        // Extract data needed for the closure to avoid borrow conflicts
        let mode_change_notification = self.mode_change_notification;
        let action_notification = self.last_action_notification.clone();
        let show_mode_selector = self.show_mode_selector;
        let current_build_mode = self.current_build_mode;
        let config_width = self.config.width;
        let config_height = self.config.height;
        let click_to_capture_alpha = self.click_to_capture_alpha;
        let app_state = self.app_state;
        let mouse_sensitivity_setting = self.settings_manager.settings.controls.mouse_sensitivity;
        let _vsync_enabled = self.settings_manager.settings.graphics.vsync_enabled;
        let master_volume = self.settings_manager.settings.audio.master_volume;
        let available_resolutions = self.available_resolutions.clone();
        let selected_resolution = self.selected_resolution;

        let mut selected_mode: Option<BuildMode> = None;
        let mut close_selector = false;
        let mut new_app_state: Option<ApplicationState> = None;
        let mut settings_changed = false;
        let mut new_resolution: Option<(u32, u32)> = None;
        let mut toggle_fullscreen_requested = false;

        let output = egui_ctx.run(egui_input, |ctx| {
            // Main Menu UI
            if app_state == ApplicationState::MainMenu {
                egui::CentralPanel::default()
                    .frame(egui::Frame::none().fill(egui::Color32::from_rgba_unmultiplied(20, 20, 30, 240)))
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(80.0);

                            // Title
                            ui.label(egui::RichText::new("🚀 Robin Engine")
                                .size(48.0)
                                .color(egui::Color32::WHITE)
                                .strong());
                            ui.label(egui::RichText::new("Interactive 3D Demo")
                                .size(24.0)
                                .color(egui::Color32::LIGHT_GRAY));

                            ui.add_space(60.0);

                            // Menu buttons
                            ui.set_max_width(300.0);

                            if ui.add_sized([250.0, 50.0], egui::Button::new(
                                egui::RichText::new("▶️ Start Demo").size(20.0)
                            )).clicked() {
                                new_app_state = Some(ApplicationState::InGame);
                            }

                            ui.add_space(10.0);

                            if ui.add_sized([250.0, 50.0], egui::Button::new(
                                egui::RichText::new("⚙️ Settings").size(20.0)
                            )).clicked() {
                                println!("🖱️ Settings button clicked!");
                                new_app_state = Some(ApplicationState::Settings);
                            }

                            ui.add_space(10.0);

                            if ui.add_sized([250.0, 50.0], egui::Button::new(
                                egui::RichText::new("🎮 Controls").size(20.0)
                            )).clicked() {
                                new_app_state = Some(ApplicationState::Controls);
                            }

                            ui.add_space(10.0);

                            if ui.add_sized([250.0, 50.0], egui::Button::new(
                                egui::RichText::new("🚪 Exit").size(20.0)
                            )).clicked() {
                                std::process::exit(0);
                            }

                            ui.add_space(40.0);

                            // Footer info
                            ui.label(egui::RichText::new("Apple Silicon Optimized")
                                .size(14.0)
                                .color(egui::Color32::from_rgb(100, 200, 100)));
                            ui.label(egui::RichText::new("Configure Fullscreen in Settings")
                                .size(12.0)
                                .color(egui::Color32::GRAY));
                        });
                    });
            }

            // Settings Menu - Responsive and Centered
            else if app_state == ApplicationState::Settings {
                // Calculate responsive sizing and positioning
                let window_size = egui::vec2(config_width as f32, config_height as f32);
                let base_scale = (window_size.x / 1920.0).clamp(0.6, 2.0);
                let settings_width = (window_size.x * 0.4).clamp(400.0, 800.0);
                let settings_height = (window_size.y * 0.8).clamp(500.0, 900.0);

                // Center the settings panel
                let settings_pos = egui::pos2(
                    (window_size.x - settings_width) / 2.0,
                    (window_size.y - settings_height) / 2.0,
                );

                egui::Area::new("settings_panel".into())
                    .fixed_pos(settings_pos)
                    .show(ctx, |ui| {
                        egui::Frame::window(&ctx.style())
                            .fill(egui::Color32::from_rgba_unmultiplied(15, 15, 25, 245))
                            .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(70, 70, 90)))
                            .rounding(egui::Rounding::same(8.0))
                            .inner_margin(egui::Margin::same(20.0 * base_scale))
                            .show(ui, |ui| {
                                ui.set_width(settings_width);
                                ui.set_height(settings_height);

                                egui::ScrollArea::vertical()
                                    .max_height(settings_height - 40.0 * base_scale)
                                    .show(ui, |ui| {
                                        ui.vertical_centered(|ui| {
                                            // Title with responsive scaling
                                            ui.label(egui::RichText::new("⚙️ Settings")
                                                .size(36.0 * base_scale)
                                                .color(egui::Color32::WHITE)
                                                .strong());

                                            ui.add_space(30.0 * base_scale);

                                            // Graphics Settings
                                            ui.separator();
                                            ui.label(egui::RichText::new("Graphics")
                                                .size(20.0 * base_scale)
                                                .color(egui::Color32::YELLOW));
                                            ui.add_space(15.0 * base_scale);

                                            // Resolution Selector
                                            ui.horizontal(|ui| {
                                                ui.label(egui::RichText::new("Resolution:")
                                                    .size(14.0 * base_scale)
                                                    .color(egui::Color32::WHITE));
                                                ui.add_space(10.0 * base_scale);

                                                let mut selected_resolution_index = available_resolutions
                                                    .iter()
                                                    .position(|&r| r == selected_resolution)
                                                    .unwrap_or(0);

                                                egui::ComboBox::from_id_source("resolution_selector")
                                                    .selected_text(format!("{}x{}", selected_resolution.0, selected_resolution.1))
                                                    .width(150.0 * base_scale)
                                                    .show_index(ui, &mut selected_resolution_index, available_resolutions.len(), |i| {
                                                        let res = available_resolutions[i];
                                                        format!("{}x{}", res.0, res.1)
                                                    });

                                                if selected_resolution_index < available_resolutions.len() {
                                                    let resolution = available_resolutions[selected_resolution_index];
                                                    if resolution != selected_resolution {
                                                        new_resolution = Some(resolution);
                                                        settings_changed = true;
                                                    }
                                                }
                                            });

                                            ui.add_space(15.0 * base_scale);

                                            // Fullscreen Toggle
                                            ui.horizontal(|ui| {
                                                ui.label(egui::RichText::new("Fullscreen:")
                                                    .size(14.0 * base_scale)
                                                    .color(egui::Color32::WHITE));
                                                ui.add_space(10.0 * base_scale);

                                                let mut fullscreen_enabled = self.is_fullscreen;
                                                if ui.checkbox(&mut fullscreen_enabled, "Enable fullscreen mode").changed() {
                                                    if fullscreen_enabled != self.is_fullscreen {
                                                        toggle_fullscreen_requested = true;
                                                        settings_changed = true;
                                                    }
                                                }
                                            });

                                            ui.add_space(15.0 * base_scale);

                                            // UI Scale Accessibility Setting
                                            ui.horizontal(|ui| {
                                                ui.label(egui::RichText::new("UI Scale:")
                                                    .size(14.0 * base_scale)
                                                    .color(egui::Color32::WHITE));
                                                ui.add_space(10.0 * base_scale);

                                                let mut ui_scale = self.ui_scale;
                                                let response = ui.add(egui::Slider::new(&mut ui_scale, 0.75..=2.0)
                                                    .step_by(0.05)
                                                    .text("scale")
                                                    .show_value(true));

                                                if response.changed() {
                                                    self.ui_scale = ui_scale;
                                                    settings_changed = true;
                                                }

                                                ui.add_space(10.0 * base_scale);
                                                ui.label(egui::RichText::new(format!("{}%", (ui_scale * 100.0) as i32))
                                                    .size(12.0 * base_scale)
                                                    .color(egui::Color32::LIGHT_GRAY));
                                            });

                                            ui.add_space(15.0 * base_scale);

                                            // VSync Toggle
                                            ui.horizontal(|ui| {
                                                ui.label(egui::RichText::new("VSync:")
                                                    .size(14.0 * base_scale)
                                                    .color(egui::Color32::WHITE));
                                                ui.add_space(10.0 * base_scale);

                                                let mut vsync_enabled = self.settings_manager.settings.graphics.vsync_enabled;
                                                if ui.checkbox(&mut vsync_enabled, "Vertical sync for smooth rendering").changed() {
                                                    self.settings_manager.settings.graphics.vsync_enabled = vsync_enabled;
                                                    settings_changed = true;
                                                }
                                            });

                                            ui.add_space(20.0 * base_scale);

                                            // Controls Settings
                                            ui.separator();
                                            ui.label(egui::RichText::new("Controls")
                                                .size(20.0 * base_scale)
                                                .color(egui::Color32::YELLOW));
                                            ui.add_space(15.0 * base_scale);

                                            ui.horizontal(|ui| {
                                                ui.label(egui::RichText::new("Mouse Sensitivity:")
                                                    .size(14.0 * base_scale)
                                                    .color(egui::Color32::WHITE));
                                                ui.add_space(10.0 * base_scale);
                                                let mut temp_sensitivity = mouse_sensitivity_setting;
                                                if ui.add(egui::Slider::new(&mut temp_sensitivity, 0.001..=0.01)
                                                    .custom_formatter(|n, _| format!("{:.3}", n))).changed() {
                                                    self.settings_manager.settings.controls.mouse_sensitivity = temp_sensitivity;
                                                    settings_changed = true;
                                                }
                                            });

                                            ui.add_space(20.0 * base_scale);

                                            // Audio Settings
                                            ui.separator();
                                            ui.label(egui::RichText::new("Audio")
                                                .size(20.0 * base_scale)
                                                .color(egui::Color32::YELLOW));
                                            ui.add_space(15.0 * base_scale);

                                            ui.horizontal(|ui| {
                                                ui.label(egui::RichText::new("Master Volume:")
                                                    .size(14.0 * base_scale)
                                                    .color(egui::Color32::WHITE));
                                                ui.add_space(10.0 * base_scale);
                                                let mut temp_volume = master_volume;
                                                if ui.add(egui::Slider::new(&mut temp_volume, 0.0..=1.0)
                                                    .custom_formatter(|n, _| format!("{:.0}%", n * 100.0))).changed() {
                                                    self.settings_manager.settings.audio.master_volume = temp_volume;
                                                    settings_changed = true;
                                                }
                                            });

                                            ui.add_space(40.0 * base_scale);

                                            // Back button - responsive sizing
                                            if ui.add_sized([150.0 * base_scale, 40.0 * base_scale], egui::Button::new(
                                                egui::RichText::new("← Back").size(18.0 * base_scale)
                                            )).clicked() {
                                                new_app_state = Some(ApplicationState::MainMenu);
                                            }
                                        });
                                    });
                            });
                    });
            }
            // Controls Display
            else if app_state == ApplicationState::Controls {
                egui::CentralPanel::default()
                    .frame(egui::Frame::none().fill(egui::Color32::from_rgba_unmultiplied(20, 20, 30, 240)))
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(40.0);

                            ui.label(egui::RichText::new("🎮 Controls")
                                .size(36.0)
                                .color(egui::Color32::WHITE)
                                .strong());

                            ui.add_space(30.0);
                            ui.set_max_width(500.0);

                            // Movement Controls
                            ui.separator();
                            ui.label(egui::RichText::new("Movement").size(20.0).color(egui::Color32::YELLOW));
                            ui.add_space(10.0);

                            ui.label("WASD / Arrow Keys - Move");
                            ui.label("Space - Move Up");
                            ui.label("Ctrl - Move Down");
                            ui.label("Mouse - Look Around");

                            ui.add_space(15.0);

                            // Building Controls
                            ui.separator();
                            ui.label(egui::RichText::new("Building").size(20.0).color(egui::Color32::YELLOW));
                            ui.add_space(10.0);

                            ui.label("Left Click - Place Block");
                            ui.label("Right Click - Remove Block");
                            ui.label("1-9 - Select Build Mode");
                            ui.label("Tab - Build Mode Selector");

                            ui.add_space(15.0);

                            // Material Selection
                            ui.separator();
                            ui.label(egui::RichText::new("Materials").size(20.0).color(egui::Color32::YELLOW));
                            ui.add_space(10.0);

                            ui.label("Q - Earth | E - Stone | R - Sand");
                            ui.label("T - Snow | Y - Dirt | U - Cobblestone");
                            ui.label("I - Water | O - Lava | P - Glass");
                            ui.label("L - Leaves | K - Wood");

                            ui.add_space(15.0);

                            // System Controls
                            ui.separator();
                            ui.label(egui::RichText::new("System").size(20.0).color(egui::Color32::YELLOW));
                            ui.add_space(10.0);

                            ui.label("ESC - Menu / Release Cursor");
                            ui.label("H - Toggle HUD");

                            ui.add_space(40.0);

                            // Back button
                            if ui.add_sized([150.0, 40.0], egui::Button::new(
                                egui::RichText::new("← Back").size(18.0)
                            )).clicked() {
                                new_app_state = Some(ApplicationState::MainMenu);
                            }
                        });
                    });
            }
            // Paused Menu
            else if app_state == ApplicationState::Paused {
                egui::CentralPanel::default()
                    .frame(egui::Frame::none().fill(egui::Color32::from_rgba_unmultiplied(20, 20, 30, 200)))
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(100.0);

                            ui.label(egui::RichText::new("⏸️ Paused")
                                .size(36.0)
                                .color(egui::Color32::WHITE)
                                .strong());

                            ui.add_space(40.0);
                            ui.set_max_width(250.0);

                            if ui.add_sized([200.0, 45.0], egui::Button::new(
                                egui::RichText::new("Resume").size(18.0)
                            )).clicked() {
                                new_app_state = Some(ApplicationState::InGame);
                            }

                            ui.add_space(10.0);

                            if ui.add_sized([200.0, 45.0], egui::Button::new(
                                egui::RichText::new("Settings").size(18.0)
                            )).clicked() {
                                new_app_state = Some(ApplicationState::Settings);
                            }

                            ui.add_space(10.0);

                            if ui.add_sized([200.0, 45.0], egui::Button::new(
                                egui::RichText::new("Main Menu").size(18.0)
                            )).clicked() {
                                new_app_state = Some(ApplicationState::MainMenu);
                            }
                        });
                    });
            }

            // Game HUD - only show when in game
            else if app_state == ApplicationState::InGame {
                // Mode change notification - center screen with fade
            if let Some((mode, start_time)) = mode_change_notification {
                let elapsed = start_time.elapsed().as_secs_f32();
                let notification_duration = 2.0; // 2 seconds

                if elapsed < notification_duration {
                    let alpha = if elapsed < 0.2 {
                        // Fade in over first 0.2 seconds
                        elapsed / 0.2
                    } else if elapsed > notification_duration - 0.5 {
                        // Fade out over last 0.5 seconds
                        (notification_duration - elapsed) / 0.5
                    } else {
                        1.0 // Fully visible
                    };

                    let alpha = (alpha * 255.0) as u8;

                    egui::Area::new("mode_change_notification".into())
                        .fixed_pos(egui::pos2(
                            config_width as f32 / 2.0 - 150.0,
                            config_height as f32 / 2.0 - 80.0,
                        ))
                        .show(ctx, |ui| {
                            egui::Frame::none()
                                .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, (alpha as f32 * 0.8) as u8))
                                .rounding(egui::Rounding::same(10.0))
                                .inner_margin(egui::Margin::same(20.0))
                                .show(ui, |ui| {
                                    ui.vertical_centered(|ui| {
                                        ui.label(egui::RichText::new(mode.get_icon())
                                            .size(48.0)
                                            .color(egui::Color32::from_rgba_unmultiplied(
                                                mode.get_color().r(),
                                                mode.get_color().g(),
                                                mode.get_color().b(),
                                                alpha
                                            )));
                                        ui.label(egui::RichText::new(format!("{:?} Mode", mode))
                                            .size(24.0)
                                            .color(egui::Color32::from_rgba_unmultiplied(255, 255, 255, alpha))
                                            .strong());
                                        ui.label(egui::RichText::new(mode.get_description())
                                            .size(16.0)
                                            .color(egui::Color32::from_rgba_unmultiplied(200, 200, 200, alpha)));
                                        ui.label(egui::RichText::new(format!("Press {}", mode.get_key_binding()))
                                            .size(12.0)
                                            .color(egui::Color32::from_rgba_unmultiplied(150, 150, 150, alpha)));
                                    });
                                });
                        });
                } else {
                    // Clear notification after duration
                    self.mode_change_notification = None;
                }
            }

            // Mode selector overlay - full screen modal
            if show_mode_selector {
                // Background overlay
                egui::CentralPanel::default()
                    .frame(egui::Frame::none().fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 180)))
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(50.0);

                            ui.label(egui::RichText::new("🏗️ BUILD MODE SELECTOR")
                                .size(32.0)
                                .color(egui::Color32::WHITE)
                                .strong());

                            ui.label(egui::RichText::new("Select a build mode or press Tab to close")
                                .size(16.0)
                                .color(egui::Color32::LIGHT_GRAY));

                            ui.add_space(30.0);

                            // Build modes in a 3x3 grid
                            let modes = BuildMode::get_all_modes();
                            egui::Grid::new("mode_grid")
                                .spacing([20.0, 20.0])
                                .show(ui, |ui| {
                                    for (index, mode) in modes.iter().enumerate() {
                                        if index % 3 == 0 && index > 0 {
                                            ui.end_row();
                                        }

                                        let is_current = *mode == current_build_mode;
                                        let button_color = if is_current {
                                            mode.get_color()
                                        } else {
                                            egui::Color32::from_rgba_unmultiplied(60, 60, 60, 200)
                                        };

                                        let button_frame = egui::Frame::none()
                                            .fill(button_color)
                                            .rounding(egui::Rounding::same(8.0))
                                            .inner_margin(egui::Margin::same(15.0));

                                        let response = ui.allocate_response(
                                            egui::Vec2::new(180.0, 120.0),
                                            egui::Sense::click()
                                        );

                                        if response.clicked() {
                                            if current_build_mode != *mode {
                                                selected_mode = Some(*mode);
                                            }
                                            close_selector = true;
                                        }

                                        let _ = button_frame.paint(response.rect);

                                        let text_color = if is_current {
                                            egui::Color32::BLACK
                                        } else {
                                            egui::Color32::WHITE
                                        };

                                        ui.allocate_ui_at_rect(response.rect, |ui| {
                                            ui.vertical_centered(|ui| {
                                                ui.add_space(10.0);
                                                ui.label(egui::RichText::new(mode.get_icon())
                                                    .size(32.0)
                                                    .color(if is_current { egui::Color32::BLACK } else { mode.get_color() }));
                                                ui.label(egui::RichText::new(format!("{:?}", mode))
                                                    .size(16.0)
                                                    .color(text_color)
                                                    .strong());
                                                ui.label(egui::RichText::new(mode.get_description())
                                                    .size(11.0)
                                                    .color(text_color));
                                                ui.label(egui::RichText::new(format!("Key: {}", mode.get_key_binding()))
                                                    .size(10.0)
                                                    .color(text_color));
                                            });
                                        });

                                        if response.hovered() {
                                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                        }
                                    }
                                });

                            ui.add_space(40.0);
                            ui.label(egui::RichText::new("💡 Tip: Use number keys (1-9) for quick mode switching")
                                .size(14.0)
                                .color(egui::Color32::YELLOW));
                        });
                    });
            }

            // Crosshair - always centered (but not when mode selector is open)
            if !show_mode_selector {
                egui::Area::new("crosshair".into())
                    .fixed_pos(egui::pos2(
                        config_width as f32 / 2.0 - 10.0,
                        config_height as f32 / 2.0 - 10.0,
                    ))
                    .show(ctx, |ui| {
                        ui.visuals_mut().override_text_color = Some(egui::Color32::WHITE);
                        ui.label(egui::RichText::new("🎯").color(egui::Color32::WHITE));
                    });
            }

            // Click-to-capture overlay - center screen
            if click_to_capture_alpha > 0.0 {
                let alpha = (click_to_capture_alpha * 255.0) as u8;
                egui::Area::new("click_to_capture_overlay".into())
                    .fixed_pos(egui::pos2(
                        config_width as f32 / 2.0 - 200.0,
                        config_height as f32 / 2.0 + 60.0,
                    ))
                    .show(ctx, |ui| {
                        egui::Frame::none()
                            .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, (alpha as f32 * 0.8) as u8))
                            .rounding(egui::Rounding::same(10.0))
                            .inner_margin(egui::Margin::same(20.0))
                            .show(ui, |ui| {
                                ui.vertical_centered(|ui| {
                                    ui.label(egui::RichText::new("🖱️")
                                        .size(32.0)
                                        .color(egui::Color32::from_rgba_unmultiplied(255, 255, 255, alpha)));
                                    ui.label(egui::RichText::new("Click anywhere to resume")
                                        .size(18.0)
                                        .color(egui::Color32::from_rgba_unmultiplied(255, 255, 255, alpha))
                                        .strong());
                                    ui.label(egui::RichText::new("Mouse control will be recaptured")
                                        .size(14.0)
                                        .color(egui::Color32::from_rgba_unmultiplied(200, 200, 200, alpha)));
                                });
                            });
                    });
            }

            // Status overlay - top left
            egui::Area::new("status_overlay".into())
                .fixed_pos(egui::pos2(20.0, 20.0))
                .show(ctx, |ui| {
                    ui.visuals_mut().panel_fill = egui::Color32::from_rgba_unmultiplied(0, 0, 0, 100);
                    ui.visuals_mut().override_text_color = Some(egui::Color32::WHITE);

                    egui::Frame::none()
                        .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 150))
                        .rounding(egui::Rounding::same(5.0))
                        .inner_margin(egui::Margin::same(10.0))
                        .show(ui, |ui| {
                            ui.heading("🚀 Robin Engine");
                            ui.separator();

                            // Enhanced build mode display with icon and color
                            ui.horizontal(|ui| {
                                let mode_icon = self.current_build_mode.get_icon();
                                let mode_color = self.current_build_mode.get_color();
                                let mode_desc = self.current_build_mode.get_description();

                                ui.label(egui::RichText::new(mode_icon).size(20.0).color(mode_color));
                                ui.vertical(|ui| {
                                    ui.label(egui::RichText::new(format!("Build Mode: {:?}", self.current_build_mode))
                                        .color(mode_color).strong());
                                    ui.label(egui::RichText::new(mode_desc).size(11.0).color(egui::Color32::LIGHT_GRAY));
                                    ui.label(egui::RichText::new(format!("Key: {}", self.current_build_mode.get_key_binding()))
                                        .size(10.0).color(egui::Color32::GRAY));
                                });
                            });
                            ui.label(format!("🧱 Material: {:?}", self.current_voxel_type));
                            ui.label(format!("📍 Camera: ({:.1}, {:.1}, {:.1})",
                                self.camera.position.x,
                                self.camera.position.y,
                                self.camera.position.z
                            ));
                            ui.label(format!("🧮 Voxels: {}", self.world.voxels.len()));
                            ui.separator();
                            ui.label("⌨️ Controls:");
                            ui.label("WASD - Move");
                            ui.label("Mouse - Look");
                            ui.label("ESC - Release Cursor");
                            ui.label("1-9 - Build Modes");
                            ui.label("Q,E,R,T,Y,U,I,O,P,L - Materials");
                        });
                });

            // Performance info - top right
            egui::Area::new("performance".into())
                .fixed_pos(egui::pos2(self.config.width as f32 - 200.0, 20.0))
                .show(ctx, |ui| {
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 150))
                        .rounding(egui::Rounding::same(5.0))
                        .inner_margin(egui::Margin::same(10.0))
                        .show(ui, |ui| {
                            ui.visuals_mut().override_text_color = Some(egui::Color32::WHITE);
                            ui.heading("🍎 Apple Silicon");
                            ui.separator();
                            ui.label(format!("🔧 Metal: {}", self.capabilities.has_metal));
                            ui.label(format!("💻 M-Series: {}", self.capabilities.has_apple_silicon));
                            ui.label("🧠 Unified Memory: true");
                            ui.label(format!("🎯 Target FPS: {}", self.target_fps as u32));
                        });
                });

            // Action notifications (undo/redo) - center of screen with fade animation
            if let Some((message, timestamp)) = action_notification {
                let elapsed = timestamp.elapsed().as_secs_f32();
                let fade_duration = 2.0; // 2 seconds

                if elapsed < fade_duration {
                    let alpha = ((fade_duration - elapsed) / fade_duration).clamp(0.0, 1.0);
                    let alpha = (alpha * 255.0) as u8;

                    egui::Area::new("action_notification".into())
                        .fixed_pos(egui::pos2(config_width as f32 / 2.0 - 120.0, config_height as f32 / 2.0 - 80.0))
                        .show(ctx, |ui| {
                            egui::Frame::none()
                                .fill(egui::Color32::from_rgba_unmultiplied(20, 20, 20, (180.0 * alpha as f32 / 255.0) as u8))
                                .rounding(egui::Rounding::same(10.0))
                                .inner_margin(egui::Margin::same(15.0))
                                .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgba_unmultiplied(100, 100, 100, alpha)))
                                .show(ui, |ui| {
                                    ui.visuals_mut().override_text_color = Some(egui::Color32::from_rgba_unmultiplied(255, 255, 255, alpha));
                                    ui.label(egui::RichText::new(&message)
                                        .size(16.0)
                                        .strong()
                                        .color(egui::Color32::from_rgba_unmultiplied(255, 255, 255, alpha)));
                                });
                        });
                }
            }

            // Health Monitoring Overlay - bottom right
            egui::Area::new("health_monitor".into())
                .fixed_pos(egui::pos2(config_width as f32 - 280.0, config_height as f32 - 250.0))
                .show(ctx, |ui| {
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgba_unmultiplied(20, 20, 30, 200))
                        .rounding(egui::Rounding::same(8.0))
                        .inner_margin(egui::Margin::same(12.0))
                        .stroke(egui::Stroke::new(1.5, egui::Color32::from_rgba_unmultiplied(100, 150, 200, 120)))
                        .show(ui, |ui| {
                            ui.visuals_mut().override_text_color = Some(egui::Color32::WHITE);

                            // Header with health icon
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("🏥").size(16.0));
                                ui.label(egui::RichText::new("System Health")
                                    .size(14.0)
                                    .color(egui::Color32::from_rgb(100, 200, 255))
                                    .strong());
                            });
                            ui.separator();

                            // Memory pressure indicator
                            let memory_pressure = self.buffer_validator.get_memory_pressure();
                            let memory_color = if memory_pressure > 0.9 {
                                egui::Color32::from_rgb(255, 80, 80)   // Critical - Red
                            } else if memory_pressure > 0.8 {
                                egui::Color32::from_rgb(255, 200, 80)  // Warning - Orange
                            } else if memory_pressure > 0.6 {
                                egui::Color32::from_rgb(255, 255, 80)  // Caution - Yellow
                            } else {
                                egui::Color32::from_rgb(80, 255, 80)   // Healthy - Green
                            };

                            ui.horizontal(|ui| {
                                let memory_icon = if memory_pressure > 0.8 { "⚠️" } else { "💾" };
                                ui.label(egui::RichText::new(memory_icon).size(12.0));
                                ui.label(egui::RichText::new(format!("Memory: {:.1}%", memory_pressure * 100.0))
                                    .size(11.0)
                                    .color(memory_color));
                            });

                            // Auto-save status
                            let time_since_save = self.last_auto_save.elapsed();
                            let save_color = if time_since_save > std::time::Duration::from_secs(60) {
                                egui::Color32::from_rgb(255, 200, 80)  // Warning - overdue
                            } else {
                                egui::Color32::from_rgb(80, 255, 80)   // Healthy
                            };

                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("💾").size(12.0));
                                ui.label(egui::RichText::new(format!("Save: {}s ago", time_since_save.as_secs()))
                                    .size(11.0)
                                    .color(save_color));
                            });

                            // Error recovery system status
                            let is_recovery_healthy = self.error_recovery.is_system_healthy();
                            let recovery_color = if is_recovery_healthy {
                                egui::Color32::from_rgb(80, 255, 80)   // Healthy
                            } else {
                                egui::Color32::from_rgb(255, 80, 80)   // Unhealthy
                            };

                            ui.horizontal(|ui| {
                                let recovery_icon = if is_recovery_healthy { "✅" } else { "🚨" };
                                ui.label(egui::RichText::new(recovery_icon).size(12.0));
                                ui.label(egui::RichText::new(format!("Recovery: {}",
                                    if is_recovery_healthy { "OK" } else { "DEGRADED" }))
                                    .size(11.0)
                                    .color(recovery_color));
                            });

                            // GPU resource status
                            if let Ok(gpu_manager) = self.gpu_resource_manager.lock() {
                                let gpu_stats = gpu_manager.get_stats();
                                let gpu_usage_mb = gpu_stats.memory_used as f64 / (1024.0 * 1024.0);
                                let gpu_color = if gpu_usage_mb > 200.0 {
                                    egui::Color32::from_rgb(255, 200, 80)  // Warning
                                } else {
                                    egui::Color32::from_rgb(80, 255, 80)   // Healthy
                                };

                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("🎮").size(12.0));
                                    ui.label(egui::RichText::new(format!("GPU: {:.1}MB", gpu_usage_mb))
                                        .size(11.0)
                                        .color(gpu_color));
                                });
                            }

                            // Performance indicator
                            let frame_duration = self.last_frame_time.elapsed();
                            let fps_estimate = if frame_duration.as_millis() > 0 {
                                1000.0 / frame_duration.as_millis() as f64
                            } else {
                                60.0
                            };

                            let fps_color = if fps_estimate < 30.0 {
                                egui::Color32::from_rgb(255, 80, 80)   // Poor performance
                            } else if fps_estimate < 50.0 {
                                egui::Color32::from_rgb(255, 200, 80)  // Fair performance
                            } else {
                                egui::Color32::from_rgb(80, 255, 80)   // Good performance
                            };

                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("⚡").size(12.0));
                                ui.label(egui::RichText::new(format!("FPS: {:.0}", fps_estimate))
                                    .size(11.0)
                                    .color(fps_color));
                            });

                            // Overall system status
                            ui.separator();
                            let overall_healthy = memory_pressure < 0.9 &&
                                                 is_recovery_healthy &&
                                                 time_since_save < std::time::Duration::from_secs(120);

                            let status_text = if overall_healthy { "🟢 All Systems Operational" } else { "🟡 Monitoring Active" };
                            let status_color = if overall_healthy {
                                egui::Color32::from_rgb(80, 255, 80)
                            } else {
                                egui::Color32::from_rgb(255, 200, 80)
                            };

                            ui.label(egui::RichText::new(status_text)
                                .size(10.0)
                                .color(status_color)
                                .strong());
                        });
                });

            } // End of InGame HUD
        });

        let primitives = egui_ctx.tessellate(output.shapes, output.pixels_per_point);
        for (id, image_delta) in &output.textures_delta.set {
            self.egui_renderer.update_texture(&self.device, &self.queue, *id, image_delta);
        }

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: output.pixels_per_point,
        };

        self.egui_renderer.update_buffers(&self.device, &self.queue, encoder, &primitives, &screen_descriptor);

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("HUD Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            self.egui_renderer.render(&mut render_pass, &primitives, &screen_descriptor);
        }

        for id in &output.textures_delta.free {
            self.egui_renderer.free_texture(id);
        }

        // Handle mode selector interactions
        if close_selector {
            self.show_mode_selector = false;
        }
        if let Some(mode) = selected_mode {
            self.current_build_mode = mode;
            self.trigger_mode_change_notification(mode);
        }

        // Update notification timer
        if let Some((_, start_time)) = self.mode_change_notification {
            if start_time.elapsed().as_secs_f32() >= 2.0 {
                self.mode_change_notification = None;
            }
        }

        // Handle application state transitions
        if let Some(new_state) = new_app_state {
            self.transition_to_state(new_state, window);
        }

        // Apply settings changes with debounced saving
        if settings_changed {
            self.mouse_sensitivity = self.settings_manager.settings.controls.mouse_sensitivity;
            // Start or reset the debounce timer
            self.settings_changed_timer = Some(std::time::Instant::now());
        }

        // Check if settings should be saved (debounced)
        if let Some(timer) = self.settings_changed_timer {
            if timer.elapsed().as_millis() > 500 {  // 500ms debounce
                if let Err(e) = self.settings_manager.save() {
                    println!("⚠️  Failed to save settings: {}", e);
                } else {
                    println!("💾 Settings saved (debounced)");
                }
                self.settings_changed_timer = None;
            }
        }

        // Queue resolution change for after frame completion
        if let Some(resolution) = new_resolution {
            self.selected_resolution = resolution;
            let new_size = winit::dpi::PhysicalSize::new(resolution.0, resolution.1);
            self.pending_resize = Some(new_size);
            println!("🖥️ Resolution change queued: {}x{}", resolution.0, resolution.1);
        }

        // Apply fullscreen toggle
        if toggle_fullscreen_requested {
            self.toggle_fullscreen(window);
        }
    }

    fn transition_to_state(&mut self, new_state: ApplicationState, window: &Window) {
        match (self.app_state, new_state) {
            // Transitioning TO InGame
            (_, ApplicationState::InGame) if self.app_state != ApplicationState::InGame => {
                self.app_state = ApplicationState::InGame;
                self.set_cursor_capture(window, true);
                self.initialize_window_mode(window);
                println!("🎮 Entering game mode");
            }

            // Transitioning FROM InGame to Paused
            (ApplicationState::InGame, ApplicationState::Paused) => {
                self.app_state = ApplicationState::Paused;
                self.set_cursor_capture(window, false);
                println!("⏸️ Game paused");
            }

            // Transitioning to MainMenu
            (_, ApplicationState::MainMenu) => {
                self.app_state = ApplicationState::MainMenu;
                self.set_cursor_capture(window, false);
                window.set_fullscreen(None); // Exit fullscreen when going to menu
                window.set_maximized(false); // Un-maximize when going to menu
                println!("📋 Main menu");
            }

            // Transitioning to Settings or Controls
            (_, ApplicationState::Settings) | (_, ApplicationState::Controls) => {
                println!("🔧 Transitioning to Settings state...");
                self.app_state = new_state;
                self.set_cursor_capture(window, false);
                println!("✅ Settings state transition complete");
            }

            _ => {
                self.app_state = new_state;
            }
        }
    }

    fn trigger_mode_change_notification(&mut self, new_mode: BuildMode) {
        self.mode_change_notification = Some((new_mode, Instant::now()));
        self.last_mode_change = Instant::now();
        println!("🔄 Build Mode changed to: {} {}", new_mode.get_icon(), new_mode.get_description());
    }

    // Undo/Redo system implementation
    fn record_action(&mut self, action_type: ActionType, position: (i32, i32, i32), voxel_type: VoxelType) {
        // Store values for logging before move
        let action_type_copy = action_type;
        let position_copy = position;

        let action = VoxelAction {
            action_type,
            position,
            voxel_type,
        };

        // Add to history and limit to 10 actions
        self.action_history.push_back(action);
        if self.action_history.len() > 10 {
            self.action_history.pop_front();
        }

        // Clear redo stack when new action is performed
        self.redo_stack.clear();

        println!("📝 Action recorded: {:?} at {:?}", action_type_copy, position_copy);
    }

    fn undo_last_action(&mut self) -> bool {
        if let Some(action) = self.action_history.pop_back() {
            // Store all values before moving action
            let position = action.position;
            let action_type = action.action_type;
            let voxel_type = action.voxel_type;

            // Reverse the action
            match action_type {
                ActionType::Place => {
                    // Undo place by removing the voxel
                    self.world.set_voxel(position.0, position.1, position.2, VoxelType::Air);
                    self.redo_stack.push(action);
                    self.last_action_notification = Some(("↩️ Undid voxel placement".to_string(), Instant::now()));
                    println!("↩️ Undid voxel placement at {:?}", position);
                }
                ActionType::Remove => {
                    // Undo remove by restoring the voxel
                    self.world.set_voxel(position.0, position.1, position.2, voxel_type);
                    self.redo_stack.push(action);
                    self.last_action_notification = Some(("↩️ Undid voxel removal".to_string(), Instant::now()));
                    println!("↩️ Undid voxel removal at {:?}", position);
                }
            }
            self.needs_mesh_update = true;
            true
        } else {
            self.last_action_notification = Some(("⚠️ Nothing to undo".to_string(), Instant::now()));
            false
        }
    }

    fn redo_last_action(&mut self) -> bool {
        if let Some(action) = self.redo_stack.pop() {
            // Store all values before moving action
            let position = action.position;
            let action_type = action.action_type;
            let voxel_type = action.voxel_type;

            // Reapply the action
            match action_type {
                ActionType::Place => {
                    self.world.set_voxel(position.0, position.1, position.2, voxel_type);
                    self.action_history.push_back(action);
                    self.last_action_notification = Some(("↪️ Redid voxel placement".to_string(), Instant::now()));
                    println!("↪️ Redid voxel placement at {:?}", position);
                }
                ActionType::Remove => {
                    self.world.set_voxel(position.0, position.1, position.2, VoxelType::Air);
                    self.action_history.push_back(action);
                    self.last_action_notification = Some(("↪️ Redid voxel removal".to_string(), Instant::now()));
                    println!("↪️ Redid voxel removal at {:?}", position);
                }
            }
            self.needs_mesh_update = true;
            true
        } else {
            self.last_action_notification = Some(("⚠️ Nothing to redo".to_string(), Instant::now()));
            false
        }
    }

    fn get_wireframe_vertices(x: i32, y: i32, z: i32) -> Vec<Vertex> {
        let x = x as f32;
        let y = y as f32;
        let z = z as f32;

        // Wireframe color - bright yellow for visibility
        let color = [1.0, 1.0, 0.0]; // Yellow wireframe
        let normal = [0.0, 1.0, 0.0]; // Default normal

        // Define the 8 corners of the cube with slight outward offset to prevent z-fighting
        let offset = 0.51; // Slightly larger than 0.5 to appear outside the voxel
        let corners = [
            [x - offset, y - offset, z - offset], // 0: bottom-front-left
            [x + offset, y - offset, z - offset], // 1: bottom-front-right
            [x + offset, y + offset, z - offset], // 2: top-front-right
            [x - offset, y + offset, z - offset], // 3: top-front-left
            [x - offset, y - offset, z + offset], // 4: bottom-back-left
            [x + offset, y - offset, z + offset], // 5: bottom-back-right
            [x + offset, y + offset, z + offset], // 6: top-back-right
            [x - offset, y + offset, z + offset], // 7: top-back-left
        ];

        // Create vertices for wireframe lines
        let mut vertices = Vec::new();

        // Add all 12 edges of the cube (each edge needs 2 vertices)
        let edges = [
            // Bottom face
            (0, 1), (1, 2), (2, 3), (3, 0),
            // Top face
            (4, 5), (5, 6), (6, 7), (7, 4),
            // Vertical edges
            (0, 4), (1, 5), (2, 6), (3, 7),
        ];

        for (start, end) in edges.iter() {
            vertices.push(Vertex {
                position: corners[*start],
                color,
                normal
            });
            vertices.push(Vertex {
                position: corners[*end],
                color,
                normal
            });
        }

        vertices
    }

    // Window management for immersive experience
    pub fn initialize_window_mode(&mut self, window: &Window) {
        // For better macOS compatibility, start with maximized window instead of fullscreen
        // This addresses the user's feedback that fullscreen initialization isn't working reliably
        window.set_maximized(true);
        self.is_fullscreen = false;
        println!("📺 Initialized as maximized window (for better macOS compatibility)");
        println!("💡 Use Settings menu to configure fullscreen mode");
    }

    pub fn toggle_fullscreen(&mut self, window: &Window) {
        self.is_fullscreen = !self.is_fullscreen;

        if self.is_fullscreen {
            // Enter fullscreen on primary monitor
            if let Some(monitor) = window.primary_monitor() {
                window.set_fullscreen(Some(Fullscreen::Borderless(Some(monitor))));
            } else {
                window.set_fullscreen(Some(Fullscreen::Borderless(None)));
            }
            println!("🖥️ Entered fullscreen mode");
        } else {
            // Exit fullscreen
            window.set_fullscreen(None);
            println!("🪟 Entered windowed mode");
        }
    }

    pub fn set_cursor_capture(&mut self, window: &Window, captured: bool) {
        self.is_cursor_captured = captured;
        self.cursor_visible = !captured;

        if captured {
            // Capture cursor for first-person control
            window.set_cursor_visible(false);
            self.show_click_to_capture = false; // Hide click-to-capture prompt
            if let Err(e) = window.set_cursor_grab(CursorGrabMode::Confined) {
                // Fallback to locked if confined not available
                if let Err(e2) = window.set_cursor_grab(CursorGrabMode::Locked) {
                    eprintln!("Failed to capture cursor: {} / {}", e, e2);
                } else {
                    println!("🎯 Cursor captured (locked mode)");
                }
            } else {
                println!("🎯 Cursor captured (confined mode)");
            }
        } else {
            // Release cursor and show click-to-capture prompt
            window.set_cursor_visible(true);
            self.show_click_to_capture = true; // Show click-to-capture prompt
            self.click_to_capture_alpha = 0.0; // Start fade-in animation
            if let Err(e) = window.set_cursor_grab(CursorGrabMode::None) {
                eprintln!("Failed to release cursor: {}", e);
            } else {
                println!("🖱️ Cursor released - Click anywhere to resume");
            }
        }
    }

    pub fn toggle_cursor_capture(&mut self, window: &Window) {
        self.set_cursor_capture(window, !self.is_cursor_captured);
    }

    pub fn print_status(&self) {
        println!("\n🎮 Current Status:");
        println!("   🏗️ Build Mode: {:?} ({})", self.current_build_mode, self.current_build_mode.get_key_binding());
        println!("   🧱 Voxel Type: {:?}", self.current_voxel_type);
        println!("   📍 Camera Pos: ({:.1}, {:.1}, {:.1})",
                 self.camera.position.x, self.camera.position.y, self.camera.position.z);
        println!("   🧮 Voxels: {} placed", self.world.voxels.len());
        println!("   🖥️ Fullscreen: {}", self.is_fullscreen);
        println!("   🎯 Cursor Captured: {}", self.is_cursor_captured);
        if self.capabilities.has_apple_silicon {
            println!("   🍎 Apple Silicon Optimizations: Active");
        }
    }

    /// Safely execute a GPU operation with error recovery
    fn safe_gpu_operation<T, F>(&self, operation_name: &str, operation: F) -> RobinResult<T>
    where
        F: FnOnce() -> Result<T, Box<dyn std::error::Error>>,
    {
        match operation() {
            Ok(result) => Ok(result),
            Err(error) => {
                let robin_error = RobinError::gpu_resource_error(
                    format!("GPU operation '{}' failed: {}", operation_name, error),
                    "gpu_operation"
                );

                // Log the error for debugging
                log::warn!("⚠️ GPU operation '{}' failed, attempting recovery: {}", operation_name, error);

                // For now, return the error - in a full implementation, we'd use the error recovery system
                Err(robin_error)
            }
        }
    }

    /// Safely write buffer data with overflow protection and error recovery
    fn safe_write_buffer<T>(&self, buffer: &wgpu::Buffer, offset: u64, data: &[T]) -> RobinResult<()>
    where
        T: bytemuck::Pod,
    {
        // Validate buffer write operation
        let data_size = (data.len() * std::mem::size_of::<T>()) as u64;
        self.buffer_validator.validate_data_write(
            buffer.size(),
            offset,
            data_size,
            "buffer_write"
        )?;

        // Perform the write operation with error recovery
        self.safe_gpu_operation("write_buffer", || {
            self.queue.write_buffer(buffer, offset, bytemuck::cast_slice(data));
            Ok(())
        })
    }

    /// Safely get surface texture with error recovery and fallback
    fn safe_get_surface_texture(&self) -> RobinResult<wgpu::SurfaceTexture> {
        self.safe_gpu_operation("get_current_texture", || {
            match self.surface.get_current_texture() {
                Ok(texture) => Ok(texture),
                Err(wgpu::SurfaceError::Lost) => {
                    log::warn!("🔄 Surface lost, attempting to reconfigure...");
                    // In a full implementation, we'd reconfigure the surface here
                    Err("Surface lost - needs reconfiguration".into())
                },
                Err(wgpu::SurfaceError::OutOfMemory) => {
                    log::error!("💾 GPU out of memory during surface texture acquisition");
                    Err("GPU out of memory".into())
                },
                Err(wgpu::SurfaceError::Outdated) => {
                    log::warn!("🔄 Surface outdated, continuing with degraded mode...");
                    // In a full implementation, we'd trigger a surface reconfigure
                    Err("Surface outdated".into())
                },
                Err(wgpu::SurfaceError::Timeout) => {
                    log::warn!("⏱️ Surface texture timeout, retrying...");
                    Err("Surface texture timeout".into())
                },
            }
        })
    }

    /// Check if the system is under memory pressure
    fn is_memory_pressure_high(&self) -> bool {
        self.buffer_validator.is_memory_pressure_high()
    }

    /// Get memory pressure level for adaptive quality
    fn get_memory_pressure(&self) -> f32 {
        self.buffer_validator.get_memory_pressure()
    }

    /// Check memory pressure and apply adaptive quality if needed
    fn check_and_adapt_performance(&mut self) {
        // Only check every 500ms to avoid excessive overhead
        if self.last_memory_check.elapsed().as_millis() < 500 {
            return;
        }
        self.last_memory_check = Instant::now();

        if !self.adaptive_quality_enabled {
            return;
        }

        let current_pressure = self.get_memory_pressure();
        self.memory_pressure_level = current_pressure;

        // Determine appropriate performance mode based on memory pressure
        let new_mode = if current_pressure > 0.9 {
            PerformanceMode::Emergency
        } else if current_pressure > 0.8 {
            PerformanceMode::Low
        } else if current_pressure > 0.6 {
            PerformanceMode::Medium
        } else {
            PerformanceMode::High
        };

        // Only change mode if different from current
        if new_mode != self.performance_mode {
            log::info!("🔄 Performance mode changing from {:?} to {:?} (memory pressure: {:.1}%)",
                      self.performance_mode, new_mode, current_pressure * 100.0);
            self.apply_performance_mode(new_mode);
        }
    }

    /// Apply a specific performance mode with graceful degradation
    fn apply_performance_mode(&mut self, mode: PerformanceMode) {
        // Store original settings before first degradation
        if self.original_settings.is_none() && mode != PerformanceMode::High {
            self.original_settings = Some(QualitySettings::default());
        }

        self.performance_mode = mode;
        self.degradation_active = mode != PerformanceMode::High;

        let settings = match mode {
            PerformanceMode::High => {
                // Restore original settings if available
                if let Some(original) = &self.original_settings {
                    original.clone()
                } else {
                    QualitySettings::default()
                }
            },
            PerformanceMode::Medium => QualitySettings::medium_performance(),
            PerformanceMode::Low => QualitySettings::low_performance(),
            PerformanceMode::Emergency => QualitySettings::emergency(),
        };

        // Apply the settings (this would be connected to actual rendering settings)
        self.apply_quality_settings(&settings);

        match mode {
            PerformanceMode::High => {
                log::info!("🟢 High performance mode: Full quality enabled");
                self.original_settings = None; // Clear stored settings
            },
            PerformanceMode::Medium => {
                log::info!("🟡 Medium performance mode: Some optimizations applied");
            },
            PerformanceMode::Low => {
                log::warn!("🟠 Low performance mode: Significant quality reduction");
            },
            PerformanceMode::Emergency => {
                log::error!("🔴 Emergency mode: Bare minimum rendering for stability");
            },
        }
    }

    /// Apply quality settings to the rendering system
    fn apply_quality_settings(&mut self, settings: &QualitySettings) {
        // In a real implementation, this would:
        // - Adjust render distance for voxel chunks
        // - Scale texture quality
        // - Enable/disable effects and shaders
        // - Adjust particle system limits
        // - Modify mesh detail levels

        log::debug!("📊 Applied quality settings: render_distance={}, texture_quality={:.2}, effects={}, particles={:.2}",
                   settings.render_distance, settings.texture_quality, settings.effects_enabled, settings.particle_count_multiplier);
    }

    /// Get current performance status for UI display
    fn get_performance_status(&self) -> String {
        let pressure_percent = (self.memory_pressure_level * 100.0) as u32;
        let mode_icon = match self.performance_mode {
            PerformanceMode::High => "🟢",
            PerformanceMode::Medium => "🟡",
            PerformanceMode::Low => "🟠",
            PerformanceMode::Emergency => "🔴",
        };

        if self.degradation_active {
            format!("{} {:?} Mode | Memory: {}% | Adaptive Quality: ON",
                   mode_icon, self.performance_mode, pressure_percent)
        } else {
            format!("{} {:?} Mode | Memory: {}%",
                   mode_icon, self.performance_mode, pressure_percent)
        }
    }

    /// Manually enable/disable adaptive quality
    fn set_adaptive_quality(&mut self, enabled: bool) {
        self.adaptive_quality_enabled = enabled;

        if enabled {
            log::info!("✅ Adaptive quality enabled - system will automatically adjust performance");
        } else {
            log::info!("❌ Adaptive quality disabled - manual performance control");
            // Restore high performance when disabled
            self.apply_performance_mode(PerformanceMode::High);
        }
    }

    /// Force a specific performance mode (overrides adaptive quality)
    fn force_performance_mode(&mut self, mode: PerformanceMode) {
        log::info!("🔧 Forcing performance mode: {:?}", mode);
        self.adaptive_quality_enabled = false;
        self.apply_performance_mode(mode);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();


    // Show enhanced startup message for Apple Silicon
    println!("🚀 Robin Engine - Standalone Interactive 3D Demo");
    println!("🍎 Apple Silicon Optimized with Metal Backend");
    println!("============================================================");

    // Show platform capabilities
    let capabilities = PlatformCapabilities::detect();
    println!("\n✨ Platform Capabilities:");
    println!("   🔧 Metal support: {}", capabilities.has_metal);
    println!("   💻 Apple Silicon: {}", capabilities.has_apple_silicon);
    println!("   🧠 Unified memory: {}", capabilities.unified_memory);
    println!("   🖼️ Max texture size: {}px", capabilities.max_texture_size);
    println!("   🎮 GPU Family: {}", capabilities.gpu_family);

    println!("\n🎮 Enhanced Controls:");
    println!("   WASD / Arrow Keys: Move camera");
    println!("   Space: Move up");
    println!("   Ctrl: Move down");
    println!("   Mouse Movement: Look around");
    println!("   Left Click: Place voxels");
    println!("   X + Left Click: Remove voxels");

    println!("\n🏗️ Build Modes:");
    for mode in BuildMode::get_all_modes() {
        println!("   {}({}): {:?}", mode.get_key_binding(), mode.get_key_binding(), mode);
    }

    println!("\n🧱 Voxel Types:");
    let voxel_keys = ["Q", "E", "R", "T", "Y", "U", "I", "O", "P", "L"];
    for (i, voxel_type) in VoxelType::get_all_types().iter().enumerate() {
        if i < voxel_keys.len() {
            println!("   {}({}): {:?}", voxel_keys[i], voxel_keys[i], voxel_type);
        }
    }

    if capabilities.has_apple_silicon {
        println!("\n🍎 Apple Silicon Optimizations:");
        println!("   • Metal rendering backend");
        println!("   • Unified memory architecture");
        println!("   • 120fps target framerate");
        println!("   • Optimized shader compilation");
    }

    println!("\n✅ Initializing 3D graphics...");

    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title("Robin Engine - Interactive 3D Demo (Apple Silicon Optimized)")
        .with_inner_size(PhysicalSize::new(1400, 900))
        .build(&event_loop)?;

    let mut demo = pollster::block_on(InteractiveDemo::new(&window))?;

    // Don't capture cursor or maximize window when starting in main menu
    // These will be done when transitioning to InGame state

    println!("🎯 3D Window Created Successfully!");
    demo.print_status();
    println!("============================================================");

    event_loop.run(move |event, control_flow| {
        match event {
            Event::WindowEvent { ref event, window_id } if window_id == window.id() => {
                if !demo.input(event, &window) {
                    match event {
                        WindowEvent::CloseRequested => {
                            // Perform clean shutdown before exiting
                            if let Err(error) = demo.shutdown() {
                                log::error!("🚨 Error during shutdown: {}", error);
                            }
                            control_flow.exit();
                        }
                        WindowEvent::Resized(physical_size) => {
                            demo.resize(*physical_size);
                        }
                        WindowEvent::RedrawRequested => {
                            demo.update();
                            match demo.render(&window) {
                                Ok(_) => {}
                                Err(wgpu::SurfaceError::Lost) => demo.resize(demo.size),
                                Err(wgpu::SurfaceError::OutOfMemory) => {
                                    eprintln!("Out of memory!");
                                    control_flow.exit();
                                }
                                Err(e) => eprintln!("Render error: {:?}", e),
                            }
                        }
                        _ => {}
                    }
                }
            }
            Event::DeviceEvent { ref event, .. } => {
                demo.device_input(event);
            }
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        }
    })?;

    Ok(())
}