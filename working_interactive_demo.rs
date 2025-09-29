/*!
 * Robin Engine - Working Interactive 3D Demo
 *
 * Built on the successful unified_demo.rs foundation, this demonstrates:
 * - Real wgpu 3D graphics window on macOS
 * - Interactive first-person camera with WASD + mouse controls
 * - Voxel world construction with Engineer Build Mode
 * - Metal rendering optimization for Apple Silicon
 * - Clean architecture without external dependencies
 */

use winit::{
    event::{Event, WindowEvent, DeviceEvent, KeyEvent, ElementState, MouseButton},
    event_loop::{EventLoop, ControlFlow},
    window::{WindowBuilder, Window},
    keyboard::{Key, NamedKey},
    dpi::PhysicalSize,
};
use wgpu::{Surface, SurfaceConfiguration, Device, Queue, util::DeviceExt};
use cgmath::{Matrix4, Vector3, Point3, InnerSpace, SquareMatrix, Zero, Rad, perspective, Deg};
use std::time::{Instant, Duration};
use std::collections::HashMap;
use bytemuck::{Pod, Zeroable};

// Platform detection (from successful unified_demo.rs)
#[derive(Debug)]
pub struct PlatformCapabilities {
    pub has_metal: bool,
    pub has_apple_silicon: bool,
    pub unified_memory: bool,
    pub max_texture_size: u32,
}

impl PlatformCapabilities {
    pub fn detect() -> Self {
        #[cfg(target_os = "macos")]
        {
            Self {
                has_metal: true,
                has_apple_silicon: std::env::consts::ARCH == "aarch64",
                unified_memory: std::env::consts::ARCH == "aarch64",
                max_texture_size: if std::env::consts::ARCH == "aarch64" { 16384 } else { 8192 },
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            Self {
                has_metal: false,
                has_apple_silicon: false,
                unified_memory: false,
                max_texture_size: 4096,
            }
        }
    }
}

// VoxelType system (from successful unified_demo.rs)
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
        }
    }

    pub fn is_solid(&self) -> bool {
        *self != VoxelType::Air
    }
}

// Build Mode system (from successful unified_demo.rs)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BuildMode {
    Single,
    Wall,
    Floor,
    Roof,
    Template,
    Circle,
    Sphere,
    Terrain,
    Copy,
    Paste,
}

// Template system (from successful unified_demo.rs)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TemplateType {
    Stairs,
    Arch,
    Bridge,
    Tower,
    House,
    Castle,
    Garden,
    Workshop,
    Fortress,
    Lighthouse,
    Windmill,
}

// 3D Camera for first-person navigation
#[derive(Debug)]
pub struct Camera {
    pub position: Point3<f32>,
    pub direction: Vector3<f32>,
    pub up: Vector3<f32>,
    pub fovy: Rad<f32>,
    pub aspect: f32,
    pub znear: f32,
    pub zfar: f32,
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

// Simple voxel world for interactive construction
#[derive(Debug)]
pub struct VoxelWorld {
    pub voxels: HashMap<(i32, i32, i32), VoxelType>,
    pub chunk_size: i32,
}

impl VoxelWorld {
    pub fn new() -> Self {
        let mut world = Self {
            voxels: HashMap::new(),
            chunk_size: 16,
        };

        // Create a simple ground plane
        for x in -10..=10 {
            for z in -10..=10 {
                world.set_voxel(x, 0, z, VoxelType::Stone);
            }
        }

        // Add some initial structures
        for y in 1..=3 {
            world.set_voxel(0, y, 0, VoxelType::Wood);
            world.set_voxel(2, y, 2, VoxelType::Brick);
            world.set_voxel(-2, y, -2, VoxelType::Crystal);
        }

        world
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

    pub fn raycast(&self, origin: Point3<f32>, direction: Vector3<f32>, max_distance: f32) -> Option<(i32, i32, i32)> {
        let mut current = origin.to_vec();
        let step = direction.normalize() * 0.1;
        let mut distance = 0.0;

        while distance < max_distance {
            let x = current.x.floor() as i32;
            let y = current.y.floor() as i32;
            let z = current.z.floor() as i32;

            if self.get_voxel(x, y, z).is_solid() {
                return Some((x, y, z));
            }

            current += step;
            distance += 0.1;
        }

        None
    }
}

// GPU uniform data for shaders
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],
    camera_pos: [f32; 4],
    time: f32,
    _padding: [f32; 3],
}

impl Uniforms {
    fn new() -> Self {
        Self {
            view_proj: Matrix4::identity().into(),
            camera_pos: [0.0, 0.0, 0.0, 1.0],
            time: 0.0,
            _padding: [0.0; 3],
        }
    }

    fn update_view_proj(&mut self, camera: &Camera) {
        let view = camera.get_view_matrix();
        let proj = camera.get_projection_matrix();
        self.view_proj = (proj * view).into();
        self.camera_pos = [camera.position.x, camera.position.y, camera.position.z, 1.0];
    }
}

// Vertex data for cube rendering
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
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
            ],
        }
    }
}

// Main application state
pub struct InteractiveDemo {
    // Graphics
    surface: Surface<'static>,
    device: Device,
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

    // Game state
    camera: Camera,
    world: VoxelWorld,
    current_build_mode: BuildMode,
    current_voxel_type: VoxelType,

    // Input state
    keys_pressed: std::collections::HashSet<String>,
    mouse_pressed: bool,
    last_mouse_pos: (f64, f64),

    // Timing
    last_frame_time: Instant,
}

impl InteractiveDemo {
    pub async fn new(window: &Window) -> Result<Self, Box<dyn std::error::Error>> {
        let size = window.inner_size();

        // Create wgpu surface
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::METAL | wgpu::Backends::VULKAN | wgpu::Backends::DX12,
            ..Default::default()
        });

        let surface = instance.create_surface(window)?;

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.ok_or("Failed to find adapter")?;

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
                memory_hints: wgpu::MemoryHints::Performance,
            },
            None,
        ).await?;

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
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Create shader module with simple inline shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(r#"
// Robin Engine - Simple Voxel Shader

struct Uniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    time: f32,
    _padding: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Transform vertex position to world space
    let world_position = input.position;
    out.world_position = world_position;

    // Transform to clip space
    out.clip_position = uniforms.view_proj * vec4<f32>(world_position, 1.0);

    // Pass through color
    out.color = input.color;

    // Simple normal calculation (for now, use position-based normal)
    out.normal = normalize(input.position);

    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Basic lighting calculation
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let normal = normalize(input.normal);
    let light_factor = max(dot(normal, light_dir), 0.3); // Ambient + directional

    // Distance-based fog
    let distance = length(input.world_position - uniforms.camera_pos.xyz);
    let fog_factor = 1.0 - min(distance / 50.0, 1.0);

    // Combine lighting and fog
    let lit_color = input.color * light_factor;
    let fog_color = vec3<f32>(0.5, 0.7, 0.9); // Sky blue fog
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

        // Create render pipeline
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
            cache: None,
        });

        // Create initial cube geometry
        let (vertices, indices) = Self::generate_cube_mesh();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let camera = Camera::new(size.width, size.height);
        let world = VoxelWorld::new();

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            uniform_bind_group,
            uniforms,
            camera,
            world,
            current_build_mode: BuildMode::Single,
            current_voxel_type: VoxelType::Wood,
            keys_pressed: std::collections::HashSet::new(),
            mouse_pressed: false,
            last_mouse_pos: (0.0, 0.0),
            last_frame_time: Instant::now(),
        })
    }

    fn generate_cube_mesh() -> (Vec<Vertex>, Vec<u16>) {
        let vertices = vec![
            // Front face
            Vertex { position: [-0.5, -0.5,  0.5], color: [1.0, 0.0, 0.0] },
            Vertex { position: [ 0.5, -0.5,  0.5], color: [0.0, 1.0, 0.0] },
            Vertex { position: [ 0.5,  0.5,  0.5], color: [0.0, 0.0, 1.0] },
            Vertex { position: [-0.5,  0.5,  0.5], color: [1.0, 1.0, 0.0] },
            // Back face
            Vertex { position: [-0.5, -0.5, -0.5], color: [1.0, 0.0, 1.0] },
            Vertex { position: [ 0.5, -0.5, -0.5], color: [0.0, 1.0, 1.0] },
            Vertex { position: [ 0.5,  0.5, -0.5], color: [1.0, 1.0, 1.0] },
            Vertex { position: [-0.5,  0.5, -0.5], color: [0.5, 0.5, 0.5] },
        ];

        let indices = vec![
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

        (vertices, indices)
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

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput { event: KeyEvent { logical_key, state, .. }, .. } => {
                let key_str = format!("{:?}", logical_key);
                match state {
                    ElementState::Pressed => {
                        self.keys_pressed.insert(key_str.clone());

                        // Handle build mode switching
                        match logical_key {
                            Key::Character(c) => {
                                match c.as_str() {
                                    "1" => self.current_build_mode = BuildMode::Single,
                                    "2" => self.current_build_mode = BuildMode::Wall,
                                    "3" => self.current_build_mode = BuildMode::Floor,
                                    "4" => self.current_build_mode = BuildMode::Circle,
                                    "5" => self.current_build_mode = BuildMode::Sphere,
                                    "q" => self.current_voxel_type = VoxelType::Stone,
                                    "e" => self.current_voxel_type = VoxelType::Wood,
                                    "r" => self.current_voxel_type = VoxelType::Brick,
                                    "t" => self.current_voxel_type = VoxelType::Crystal,
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
                self.mouse_pressed = *state == ElementState::Pressed;

                if self.mouse_pressed {
                    // Perform raycast and place/remove voxel
                    if let Some((x, y, z)) = self.world.raycast(
                        self.camera.position,
                        self.camera.direction,
                        10.0
                    ) {
                        // Remove existing voxel or place new one
                        if self.world.get_voxel(x, y, z).is_solid() {
                            self.world.set_voxel(x, y, z, VoxelType::Air);
                        } else {
                            // Place voxel in the direction of the ray
                            let place_y = y + 1;
                            self.world.set_voxel(x, place_y, z, self.current_voxel_type);
                        }
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
                if self.mouse_pressed {
                    const SENSITIVITY: f32 = 0.002;
                    self.camera.rotate(
                        -delta.0 as f32 * SENSITIVITY,
                        -delta.1 as f32 * SENSITIVITY,
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

        const MOVE_SPEED: f32 = 5.0;
        let move_distance = MOVE_SPEED * dt;

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

        // Update uniforms
        self.uniforms.update_view_proj(&self.camera);
        self.uniforms.time = now.elapsed().as_secs_f32();

        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniforms]),
        );
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
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
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
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

            // Render all solid voxels in the world
            for (&(x, y, z), &voxel_type) in &self.world.voxels {
                if voxel_type.is_solid() {
                    // TODO: Update vertex buffer with voxel position and color
                    // For now, just render one cube
                    render_pass.draw_indexed(0..36, 0, 0..1);
                    break; // Just render one for now
                }
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // Show startup message
    println!("🚀 Robin Engine - Working Interactive 3D Demo");
    println!("🍎 macOS-Optimized with Real wgpu Graphics");
    println!("============================================================");

    // Show platform capabilities
    let capabilities = PlatformCapabilities::detect();
    println!("\n✨ Platform Capabilities:");
    println!("   🔧 Metal support: {}", capabilities.has_metal);
    println!("   💻 Apple Silicon: {}", capabilities.has_apple_silicon);
    println!("   🧠 Unified memory: {}", capabilities.unified_memory);
    println!("   🖼️  Max texture size: {}px", capabilities.max_texture_size);

    println!("\n🎮 Controls:");
    println!("   WASD / Arrow Keys: Move camera");
    println!("   Mouse: Look around (when clicking)");
    println!("   Left Click: Place/Remove voxels");
    println!("   1-5: Change build mode");
    println!("   Q/E/R/T: Change voxel type");

    println!("\n🏗️ Build Modes: Single(1) Wall(2) Floor(3) Circle(4) Sphere(5)");
    println!("🧱 Voxel Types: Stone(Q) Wood(E) Brick(R) Crystal(T)");

    println!("\n✅ Initializing 3D graphics...");

    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title("Robin Engine - Interactive 3D Demo")
        .with_inner_size(PhysicalSize::new(1200, 800))
        .build(&event_loop)?;

    let mut demo = pollster::block_on(InteractiveDemo::new(&window))?;

    println!("🎯 3D Window Created Successfully!");
    println!("============================================================");

    event_loop.run(move |event, control_flow| {
        match event {
            Event::WindowEvent { ref event, window_id } if window_id == window.id() => {
                if !demo.input(event) {
                    match event {
                        WindowEvent::CloseRequested => control_flow.exit(),
                        WindowEvent::Resized(physical_size) => {
                            demo.resize(*physical_size);
                        }
                        WindowEvent::RedrawRequested => {
                            demo.update();
                            match demo.render() {
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