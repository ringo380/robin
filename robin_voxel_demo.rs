/// Robin Voxel Engine - Interactive 3D Demo
///
/// A real windowed application demonstrating the voxel engine with:
/// - First-person camera controls (WASD + Mouse)
/// - Real-time voxel placement/destruction
/// - Dynamic lighting
/// - Physics simulation
/// - Multiple material types

use winit::{
    event::{Event, WindowEvent, ElementState, MouseButton, DeviceEvent},
    event_loop::{EventLoop, ControlFlow},
    window::{WindowBuilder, Window},
    dpi::PhysicalSize,
    keyboard::KeyCode,
};

use wgpu::util::DeviceExt;
use cgmath::{Matrix4, Vector3, Vector4, Deg, Point3, InnerSpace, perspective, Rad};
use std::time::Instant;
use std::collections::HashSet;
use std::sync::Arc;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
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
                    offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],
    view_pos: [f32; 4],
    light_pos: [f32; 4],
    time: f32,
    _padding: [f32; 3],
}

struct Camera {
    eye: Point3<f32>,
    target: Point3<f32>,
    up: Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
    yaw: f32,
    pitch: f32,
}

impl Camera {
    fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        let view = Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = perspective(Deg(self.fovy), self.aspect, self.znear, self.zfar);
        proj * view
    }

    fn update_from_input(&mut self, forward: f32, right: f32, up: f32, mouse_dx: f32, mouse_dy: f32) {
        // Mouse look
        self.yaw += mouse_dx * 0.002;
        self.pitch -= mouse_dy * 0.002;
        self.pitch = self.pitch.clamp(-1.5, 1.5);

        // Calculate forward and right vectors
        let forward_dir = Vector3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        ).normalize();

        let right_dir = forward_dir.cross(Vector3::unit_y()).normalize();

        // Move camera
        self.eye += forward_dir * forward * 0.3;
        self.eye += right_dir * right * 0.3;
        self.eye.y += up * 0.3;

        // Update target
        self.target = self.eye + forward_dir;
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum VoxelType {
    Air,
    Stone,
    Dirt,
    Grass,
    Sand,
    Water,
    Wood,
    Leaves,
    Crystal,
    Lava,
}

impl VoxelType {
    fn color(&self) -> [f32; 3] {
        match self {
            VoxelType::Air => [0.0, 0.0, 0.0],
            VoxelType::Stone => [0.5, 0.5, 0.5],
            VoxelType::Dirt => [0.4, 0.25, 0.13],
            VoxelType::Grass => [0.13, 0.55, 0.13],
            VoxelType::Sand => [0.93, 0.79, 0.69],
            VoxelType::Water => [0.0, 0.5, 1.0],
            VoxelType::Wood => [0.55, 0.27, 0.07],
            VoxelType::Leaves => [0.13, 0.7, 0.13],
            VoxelType::Crystal => [0.7, 0.3, 1.0],
            VoxelType::Lava => [1.0, 0.3, 0.0],
        }
    }

    fn is_transparent(&self) -> bool {
        matches!(self, VoxelType::Air | VoxelType::Water | VoxelType::Leaves)
    }

    fn is_emissive(&self) -> bool {
        matches!(self, VoxelType::Crystal | VoxelType::Lava)
    }

    fn has_gravity(&self) -> bool {
        matches!(self, VoxelType::Sand | VoxelType::Water)
    }
}

struct VoxelWorld {
    chunks: std::collections::HashMap<(i32, i32, i32), Chunk>,
    chunk_size: usize,
}

impl VoxelWorld {
    fn new() -> Self {
        let mut world = Self {
            chunks: std::collections::HashMap::new(),
            chunk_size: 16,
        };
        world.generate_terrain();
        world
    }

    fn generate_terrain(&mut self) {
        // Generate a simple terrain
        for cx in -2..=2 {
            for cz in -2..=2 {
                let mut chunk = Chunk::new(self.chunk_size);

                for x in 0..self.chunk_size {
                    for z in 0..self.chunk_size {
                        let world_x = cx * self.chunk_size as i32 + x as i32;
                        let world_z = cz * self.chunk_size as i32 + z as i32;

                        // Enhanced height map with more variation
                        let height = 8 + ((world_x as f32 * 0.1).sin() * 4.0
                                    + (world_z as f32 * 0.1).cos() * 4.0
                                    + (world_x as f32 * 0.05 + world_z as f32 * 0.05).sin() * 2.0) as i32;

                        for y in 0..self.chunk_size {
                            let world_y = y as i32;
                            let voxel = if world_y < height - 3 {
                                VoxelType::Stone
                            } else if world_y < height - 1 {
                                VoxelType::Dirt
                            } else if world_y < height {
                                VoxelType::Grass
                            } else {
                                VoxelType::Air
                            };
                            chunk.set_voxel(x, y, z, voxel);
                        }
                    }
                }

                self.chunks.insert((cx, 0, cz), chunk);
            }
        }

        // Add some features
        self.set_voxel_world(0, 12, 0, VoxelType::Crystal);
        self.set_voxel_world(5, 10, 5, VoxelType::Wood);
        self.set_voxel_world(5, 11, 5, VoxelType::Wood);
        self.set_voxel_world(5, 12, 5, VoxelType::Leaves);
    }

    fn set_voxel_world(&mut self, x: i32, y: i32, z: i32, voxel_type: VoxelType) {
        let chunk_x = x.div_euclid(self.chunk_size as i32);
        let chunk_z = z.div_euclid(self.chunk_size as i32);
        let chunk_y = y.div_euclid(self.chunk_size as i32);

        let local_x = x.rem_euclid(self.chunk_size as i32) as usize;
        let local_y = y.rem_euclid(self.chunk_size as i32) as usize;
        let local_z = z.rem_euclid(self.chunk_size as i32) as usize;

        if let Some(chunk) = self.chunks.get_mut(&(chunk_x, chunk_y, chunk_z)) {
            chunk.set_voxel(local_x, local_y, local_z, voxel_type);
            chunk.needs_rebuild = true;
        }
    }

    fn get_voxel_world(&self, x: i32, y: i32, z: i32) -> VoxelType {
        let chunk_x = x.div_euclid(self.chunk_size as i32);
        let chunk_z = z.div_euclid(self.chunk_size as i32);
        let chunk_y = y.div_euclid(self.chunk_size as i32);

        let local_x = x.rem_euclid(self.chunk_size as i32) as usize;
        let local_y = y.rem_euclid(self.chunk_size as i32) as usize;
        let local_z = z.rem_euclid(self.chunk_size as i32) as usize;

        if let Some(chunk) = self.chunks.get(&(chunk_x, chunk_y, chunk_z)) {
            chunk.get_voxel(local_x, local_y, local_z)
        } else {
            VoxelType::Air
        }
    }

    fn raycast(&self, origin: Vector3<f32>, direction: Vector3<f32>, max_distance: f32) -> Option<(i32, i32, i32)> {
        let mut current = origin;
        let step = direction.normalize() * 0.1;
        let mut distance = 0.0;

        while distance < max_distance {
            let x = current.x.floor() as i32;
            let y = current.y.floor() as i32;
            let z = current.z.floor() as i32;

            if self.get_voxel_world(x, y, z) != VoxelType::Air {
                return Some((x, y, z));
            }

            current += step;
            distance += 0.1;
        }

        None
    }

    fn update_physics(&mut self) {
        // Simple gravity for sand
        let mut updates = Vec::new();

        for ((cx, cy, cz), chunk) in &self.chunks {
            for x in 0..self.chunk_size {
                for y in 1..self.chunk_size {
                    for z in 0..self.chunk_size {
                        let voxel = chunk.get_voxel(x, y, z);
                        if voxel.has_gravity() {
                            let world_x = cx * self.chunk_size as i32 + x as i32;
                            let world_y = cy * self.chunk_size as i32 + y as i32;
                            let world_z = cz * self.chunk_size as i32 + z as i32;

                            if self.get_voxel_world(world_x, world_y - 1, world_z) == VoxelType::Air {
                                updates.push((world_x, world_y, world_z, world_x, world_y - 1, world_z, voxel));
                            }
                        }
                    }
                }
            }
        }

        for (fx, fy, fz, tx, ty, tz, voxel) in updates {
            self.set_voxel_world(fx, fy, fz, VoxelType::Air);
            self.set_voxel_world(tx, ty, tz, voxel);
        }
    }

    fn generate_mesh(&self) -> (Vec<Vertex>, Vec<u32>) {
        let mut vertices = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        for ((cx, cy, cz), chunk) in &self.chunks {
            if !chunk.needs_rebuild && !chunk.cached_vertices.is_empty() {
                let offset = vertices.len() as u32;
                vertices.extend(&chunk.cached_vertices);
                indices.extend(chunk.cached_indices.iter().map(|i| *i + offset));
                continue;
            }

            for x in 0..self.chunk_size {
                for y in 0..self.chunk_size {
                    for z in 0..self.chunk_size {
                        let voxel = chunk.get_voxel(x, y, z);
                        if voxel == VoxelType::Air {
                            continue;
                        }

                        let world_x = cx * self.chunk_size as i32 + x as i32;
                        let world_y = cy * self.chunk_size as i32 + y as i32;
                        let world_z = cz * self.chunk_size as i32 + z as i32;

                        let color = voxel.color();
                        let pos = [world_x as f32, world_y as f32, world_z as f32];

                        // Check each face
                        let faces = [
                            ([ 0.0,  0.0,  1.0], [0, 1, 0]), // Front
                            ([ 0.0,  0.0, -1.0], [0, -1, 0]), // Back
                            ([ 1.0,  0.0,  0.0], [1, 0, 0]), // Right
                            ([-1.0,  0.0,  0.0], [-1, 0, 0]), // Left
                            ([ 0.0,  1.0,  0.0], [0, 1, 0]), // Top
                            ([ 0.0, -1.0,  0.0], [0, -1, 0]), // Bottom
                        ];

                        for (normal, check_offset) in faces {
                            let neighbor = self.get_voxel_world(
                                world_x + check_offset[0],
                                world_y + check_offset[1],
                                world_z + check_offset[2],
                            );

                            if neighbor.is_transparent() && neighbor != voxel {
                                // Add face
                                let base_index = vertices.len() as u32;

                                // Calculate face vertices based on normal
                                let face_vertices = if normal[1] > 0.0 {
                                    // Top face
                                    vec![
                                        Vertex { position: [pos[0], pos[1] + 1.0, pos[2]], color, normal },
                                        Vertex { position: [pos[0] + 1.0, pos[1] + 1.0, pos[2]], color, normal },
                                        Vertex { position: [pos[0] + 1.0, pos[1] + 1.0, pos[2] + 1.0], color, normal },
                                        Vertex { position: [pos[0], pos[1] + 1.0, pos[2] + 1.0], color, normal },
                                    ]
                                } else if normal[1] < 0.0 {
                                    // Bottom face
                                    vec![
                                        Vertex { position: [pos[0], pos[1], pos[2] + 1.0], color, normal },
                                        Vertex { position: [pos[0] + 1.0, pos[1], pos[2] + 1.0], color, normal },
                                        Vertex { position: [pos[0] + 1.0, pos[1], pos[2]], color, normal },
                                        Vertex { position: [pos[0], pos[1], pos[2]], color, normal },
                                    ]
                                } else if normal[0] > 0.0 {
                                    // Right face
                                    vec![
                                        Vertex { position: [pos[0] + 1.0, pos[1], pos[2]], color, normal },
                                        Vertex { position: [pos[0] + 1.0, pos[1], pos[2] + 1.0], color, normal },
                                        Vertex { position: [pos[0] + 1.0, pos[1] + 1.0, pos[2] + 1.0], color, normal },
                                        Vertex { position: [pos[0] + 1.0, pos[1] + 1.0, pos[2]], color, normal },
                                    ]
                                } else if normal[0] < 0.0 {
                                    // Left face
                                    vec![
                                        Vertex { position: [pos[0], pos[1], pos[2] + 1.0], color, normal },
                                        Vertex { position: [pos[0], pos[1], pos[2]], color, normal },
                                        Vertex { position: [pos[0], pos[1] + 1.0, pos[2]], color, normal },
                                        Vertex { position: [pos[0], pos[1] + 1.0, pos[2] + 1.0], color, normal },
                                    ]
                                } else if normal[2] > 0.0 {
                                    // Front face
                                    vec![
                                        Vertex { position: [pos[0], pos[1], pos[2] + 1.0], color, normal },
                                        Vertex { position: [pos[0] + 1.0, pos[1], pos[2] + 1.0], color, normal },
                                        Vertex { position: [pos[0] + 1.0, pos[1] + 1.0, pos[2] + 1.0], color, normal },
                                        Vertex { position: [pos[0], pos[1] + 1.0, pos[2] + 1.0], color, normal },
                                    ]
                                } else {
                                    // Back face
                                    vec![
                                        Vertex { position: [pos[0] + 1.0, pos[1], pos[2]], color, normal },
                                        Vertex { position: [pos[0], pos[1], pos[2]], color, normal },
                                        Vertex { position: [pos[0], pos[1] + 1.0, pos[2]], color, normal },
                                        Vertex { position: [pos[0] + 1.0, pos[1] + 1.0, pos[2]], color, normal },
                                    ]
                                };

                                vertices.extend(&face_vertices);

                                // Add indices for two triangles
                                indices.extend(&[
                                    base_index, base_index + 1, base_index + 2,
                                    base_index, base_index + 2, base_index + 3,
                                ]);
                            }
                        }
                    }
                }
            }
        }

        (vertices, indices)
    }
}

struct Chunk {
    voxels: Vec<VoxelType>,
    size: usize,
    needs_rebuild: bool,
    cached_vertices: Vec<Vertex>,
    cached_indices: Vec<u32>,
}

impl Chunk {
    fn new(size: usize) -> Self {
        Self {
            voxels: vec![VoxelType::Air; size * size * size],
            size,
            needs_rebuild: true,
            cached_vertices: Vec::new(),
            cached_indices: Vec::new(),
        }
    }

    fn set_voxel(&mut self, x: usize, y: usize, z: usize, voxel: VoxelType) {
        if x < self.size && y < self.size && z < self.size {
            self.voxels[x + y * self.size + z * self.size * self.size] = voxel;
            self.needs_rebuild = true;
        }
    }

    fn get_voxel(&self, x: usize, y: usize, z: usize) -> VoxelType {
        if x < self.size && y < self.size && z < self.size {
            self.voxels[x + y * self.size + z * self.size * self.size]
        } else {
            VoxelType::Air
        }
    }
}

struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    camera: Camera,
    uniforms: Uniforms,
    voxel_world: VoxelWorld,
    start_time: Instant,
    last_physics_update: Instant,
    keys_pressed: HashSet<winit::keyboard::KeyCode>,
    mouse_delta: (f32, f32),
    current_voxel_type: VoxelType,
    debug_console_open: bool,
    frame_count: u32,
}

impl<'a> State<'a> {
    async fn new(window: &'a Window) -> State<'a> {
        println!("🔄 Initializing Robin Voxel Engine...");
        let size = window.inner_size();

        println!("📱 Creating WGPU instance...");
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::default(),
            dx12_shader_compiler: Default::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        });

        println!("🖥️  Creating surface...");
        let surface = unsafe { instance.create_surface(window) }.unwrap();

        println!("🔌 Requesting graphics adapter...");
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();
        println!("✅ Graphics adapter found!");

        println!("🎮 Creating device and queue...");
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        ).await.unwrap();
        println!("✅ Device and queue created!");

        println!("🔧 Configuring surface...");
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())
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
        println!("✅ Surface configured: {}x{}, format: {:?}", size.width, size.height, surface_format);

        println!("📷 Setting up camera...");
        let camera = Camera {
            eye: Point3::new(0.0, 12.0, -15.0),
            target: Point3::new(0.0, 8.0, 0.0),
            up: Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 60.0,
            znear: 0.1,
            zfar: 1000.0,
            yaw: 0.0,
            pitch: -0.2, // Look down slightly to see terrain
        };
        println!("✅ Camera positioned at: ({:.2}, {:.2}, {:.2})", camera.eye.x, camera.eye.y, camera.eye.z);
        println!("   Looking at: ({:.2}, {:.2}, {:.2})", camera.target.x, camera.target.y, camera.target.z);
        println!("   FOV: {:.1}°, Aspect: {:.2}", camera.fovy, camera.aspect);

        let mut uniforms = Uniforms {
            view_proj: camera.build_view_projection_matrix().into(),
            view_pos: [camera.eye.x, camera.eye.y, camera.eye.z, 1.0],
            light_pos: [10.0, 30.0, 10.0, 1.0],
            time: 0.0,
            _padding: [0.0; 3],
        };

        let uniform_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Uniform Buffer"),
                contents: bytemuck::cast_slice(&[uniforms]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

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
                }
            ],
            label: Some("uniform_bind_group_layout"),
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                }
            ],
            label: Some("uniform_bind_group"),
        });

        println!("🎨 Compiling shaders...");
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/voxel.wgsl").into()),
        });

        println!("✅ Shaders compiled!");

        println!("🔗 Creating render pipeline...");
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
            depth_stencil: None, // Temporarily removed for crash isolation
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });
        println!("✅ Render pipeline created!");

        println!("🌍 Generating voxel world...");
        print!("   Creating terrain...");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        let mut voxel_world = VoxelWorld::new();
        println!(" ✅ Done!");

        print!("   Creating terrain mesh...");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        let (vertices, indices) = voxel_world.generate_mesh();
        println!(" ✅ Done! Generated {} vertices, {} indices", vertices.len(), indices.len());

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            }
        );
        println!("✅ GPU buffers created!");

        println!("🎉 Initialization complete! Launching 3D demo...");
        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
            uniform_buffer,
            uniform_bind_group,
            camera,
            uniforms,
            voxel_world,
            start_time: Instant::now(),
            last_physics_update: Instant::now(),
            keys_pressed: HashSet::new(),
            mouse_delta: (0.0, 0.0),
            current_voxel_type: VoxelType::Stone,
            debug_console_open: false,
            frame_count: 0,
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
        println!("🎯 INPUT EVENT: {:?}", event);
        match event {
            WindowEvent::KeyboardInput {
                device_id: _,
                event: key_event,
                is_synthetic: false,
            } => {
                println!("⌨️  KEYBOARD: {:?}", key_event);
                if let winit::keyboard::PhysicalKey::Code(keycode) = key_event.physical_key {
                    let state = key_event.state;
                match state {
                    ElementState::Pressed => {
                        self.keys_pressed.insert(keycode);

                        // Handle debug console toggle
                        println!("🔧 KEYCODE: {:?}", keycode);
                        if keycode == winit::keyboard::KeyCode::Backquote || keycode == winit::keyboard::KeyCode::F1 {
                            self.debug_console_open = !self.debug_console_open;
                            println!("🔧 DEBUG CONSOLE: {}", if self.debug_console_open { "OPENED" } else { "CLOSED" });
                            println!("📊 === DIAGNOSTIC INFO ===");
                            println!("📷 Camera: pos=({:.2}, {:.2}, {:.2}), target=({:.2}, {:.2}, {:.2})",
                                self.camera.eye.x, self.camera.eye.y, self.camera.eye.z,
                                self.camera.target.x, self.camera.target.y, self.camera.target.z);
                            println!("🎮 Render: {} indices, {}x{} surface", self.num_indices, self.size.width, self.size.height);
                            println!("🌍 World: {} chunks loaded", self.voxel_world.chunks.len());
                            println!("⏱️  Time: {:.2}s since start", self.start_time.elapsed().as_secs_f32());
                            println!("🖱️  Controls: {} keys pressed", self.keys_pressed.len());

                            // Check first few voxel positions to see where terrain is
                            println!("🧱 Terrain sample:");
                            for y in 0..20 {
                                for z in 0..10 {
                                    for x in 0..10 {
                                        let voxel = self.voxel_world.get_voxel_world(x, y, z);
                                        if voxel != VoxelType::Air {
                                            println!("   Voxel at ({}, {}, {}): {:?}", x, y, z, voxel);
                                        }
                                    }
                                }
                            }
                            println!("========================");
                        }

                        // Handle voxel type switching
                        match keycode {
                            winit::keyboard::KeyCode::Digit1 => self.current_voxel_type = VoxelType::Stone,
                            winit::keyboard::KeyCode::Digit2 => self.current_voxel_type = VoxelType::Dirt,
                            winit::keyboard::KeyCode::Digit3 => self.current_voxel_type = VoxelType::Grass,
                            winit::keyboard::KeyCode::Digit4 => self.current_voxel_type = VoxelType::Sand,
                            winit::keyboard::KeyCode::Digit5 => self.current_voxel_type = VoxelType::Water,
                            winit::keyboard::KeyCode::Digit6 => self.current_voxel_type = VoxelType::Wood,
                            winit::keyboard::KeyCode::Digit7 => self.current_voxel_type = VoxelType::Crystal,
                            winit::keyboard::KeyCode::Digit8 => self.current_voxel_type = VoxelType::Lava,
                            _ => {}
                        }
                    }
                    ElementState::Released => {
                        self.keys_pressed.remove(&keycode);
                    }
                }
                    true
                } else {
                    false
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if *state == ElementState::Pressed {
                    let forward = (self.camera.target - self.camera.eye).normalize();

                    match button {
                        MouseButton::Left => {
                            // Remove voxel
                            if let Some((x, y, z)) = self.voxel_world.raycast(
                                Vector3::new(self.camera.eye.x, self.camera.eye.y, self.camera.eye.z),
                                forward,
                                50.0,
                            ) {
                                self.voxel_world.set_voxel_world(x, y, z, VoxelType::Air);
                                self.update_mesh();
                            }
                        }
                        MouseButton::Right => {
                            // Place voxel
                            if let Some((x, y, z)) = self.voxel_world.raycast(
                                Vector3::new(self.camera.eye.x, self.camera.eye.y, self.camera.eye.z),
                                forward,
                                50.0,
                            ) {
                                // Place adjacent to hit voxel
                                let place_pos = Vector3::new(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5);
                                let eye_pos = Vector3::new(self.camera.eye.x, self.camera.eye.y, self.camera.eye.z);
                                let to_eye = (eye_pos - place_pos).normalize();

                                let nx = x + to_eye.x.round() as i32;
                                let ny = y + to_eye.y.round() as i32;
                                let nz = z + to_eye.z.round() as i32;

                                self.voxel_world.set_voxel_world(nx, ny, nz, self.current_voxel_type);
                                self.update_mesh();
                            }
                        }
                        _ => {}
                    }
                }
                true
            }
            _ => false,
        }
    }

    fn update(&mut self) {
        // Update camera based on input
        let mut forward = 0.0;
        let mut right = 0.0;
        let mut up = 0.0;

        if self.keys_pressed.contains(&winit::keyboard::KeyCode::KeyW) {
            forward += 1.0;
        }
        if self.keys_pressed.contains(&winit::keyboard::KeyCode::KeyS) {
            forward -= 1.0;
        }
        if self.keys_pressed.contains(&winit::keyboard::KeyCode::KeyD) {
            right += 1.0;
        }
        if self.keys_pressed.contains(&winit::keyboard::KeyCode::KeyA) {
            right -= 1.0;
        }
        if self.keys_pressed.contains(&winit::keyboard::KeyCode::Space) {
            up += 1.0;
        }
        if self.keys_pressed.contains(&winit::keyboard::KeyCode::ShiftLeft) {
            up -= 1.0;
        }

        self.camera.update_from_input(forward, right, up, self.mouse_delta.0, self.mouse_delta.1);
        self.mouse_delta = (0.0, 0.0);

        // Update physics
        if self.last_physics_update.elapsed().as_millis() > 100 {
            self.voxel_world.update_physics();
            self.update_mesh();
            self.last_physics_update = Instant::now();
        }

        // Update uniforms
        self.uniforms.view_proj = self.camera.build_view_projection_matrix().into();
        self.uniforms.view_pos = [self.camera.eye.x, self.camera.eye.y, self.camera.eye.z, 1.0];
        self.uniforms.time = self.start_time.elapsed().as_secs_f32();

        // Animate light
        let t = self.uniforms.time;
        self.uniforms.light_pos = [
            30.0 * t.cos(),
            30.0 + 10.0 * (t * 2.0).sin(),
            30.0 * t.sin(),
            1.0,
        ];

        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniforms]),
        );
    }

    fn update_mesh(&mut self) {
        let (vertices, indices) = self.voxel_world.generate_mesh();

        // Recreate buffers if size changed significantly
        if vertices.len() > 0 {
            self.vertex_buffer = self.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                }
            );

            self.index_buffer = self.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX,
                }
            );

            self.num_indices = indices.len() as u32;
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.frame_count += 1;
        println!("🎨 RENDER FUNCTION CALLED - Frame {}", self.frame_count);

        // Enhanced debug logging - always log first few frames to diagnose issue
        if self.frame_count <= 5 || (self.debug_console_open && self.frame_count % 60 == 1) {
            println!("🎨 === RENDER DEBUG FRAME {} ===", self.frame_count);
            println!("📊 Drawing {} indices from {} vertices", self.num_indices, self.num_indices / 3 * 3);
            println!("📷 Camera matrix values:");
            let view_proj_matrix = self.camera.build_view_projection_matrix();
            println!("   Eye: ({:.2}, {:.2}, {:.2})", self.camera.eye.x, self.camera.eye.y, self.camera.eye.z);
            println!("   Target: ({:.2}, {:.2}, {:.2})", self.camera.target.x, self.camera.target.y, self.camera.target.z);
            println!("   View-Proj [0][0..3]: {:.3}, {:.3}, {:.3}", view_proj_matrix[0][0], view_proj_matrix[0][1], view_proj_matrix[0][2]);
            println!("🌍 Voxel chunks: {}", self.voxel_world.chunks.len());

            if self.frame_count == 1 {
                println!("🔍 FIRST FRAME TERRAIN CHECK:");
                let mut terrain_found = false;
                for y in 0..20 {
                    for z in -10..10 {
                        for x in -10..10 {
                            let voxel = self.voxel_world.get_voxel_world(x, y, z);
                            if voxel != VoxelType::Air {
                                println!("   ✅ Terrain at ({}, {}, {}): {:?}", x, y, z, voxel);
                                terrain_found = true;
                            }
                        }
                    }
                    if terrain_found { break; }
                }
                if !terrain_found {
                    println!("   ❌ NO TERRAIN FOUND in checked region!");
                }
            }
            println!("============================");
        }

        println!("📺 Getting surface texture...");
        let output = self.surface.get_current_texture()?;
        println!("✅ Got surface texture, creating view...");
        println!("🔍 GRANULAR: About to create texture view...");
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        println!("🔍 GRANULAR: Texture view created successfully");
        println!("✅ Created texture view");

        // Add window visibility debugging here at the exact crash point
        println!("🪟 WINDOW DEBUG: Checking window properties...");
        // Note: window reference needs to be passed to render function

        println!("🔍 GRANULAR: About to skip depth texture creation...");
        println!("⚠️  SKIPPING depth texture creation to isolate crash...");
        println!("✅ Skipped depth texture and view");

        println!("🔍 GRANULAR: About to create command encoder...");
        println!("📝 Creating command encoder...");
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        println!("🔍 GRANULAR: Command encoder created successfully");
        println!("✅ Created command encoder");

        {
            println!("🔍 GRANULAR: About to begin render pass...");
            println!("🎬 Beginning render pass...");
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.5,
                            g: 0.8,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None, // Temporarily removed for crash isolation
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            println!("✅ Created render pass");

            println!("🔧 Setting render pipeline...");
            render_pass.set_pipeline(&self.render_pipeline);
            println!("✅ Set render pipeline");

            println!("🔗 Setting bind group...");
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            println!("✅ Set bind group");

            println!("📦 Setting vertex buffer...");
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            println!("✅ Set vertex buffer");

            println!("📇 Setting index buffer...");
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            println!("✅ Set index buffer");

            println!("🎨 Drawing indexed (indices: {})...", self.num_indices);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
            println!("✅ Draw indexed completed");

            println!("🔍 GRANULAR: About to drop render pass...");
        }
        println!("🔍 GRANULAR: Render pass dropped successfully");

        println!("📤 Submitting render commands...");
        self.queue.submit(std::iter::once(encoder.finish()));
        println!("🖥️  Presenting frame...");
        output.present();
        println!("✅ RENDER COMPLETE - Frame {} finished", self.frame_count);

        Ok(())
    }
}

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

    println!("🚀 STARTUP: Initializing Robin Voxel Demo...");
    println!("🔧 Debug mode enabled - comprehensive logging active");
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(WindowBuilder::new()
        .with_title("Robin Voxel Engine - Interactive Demo")
        .with_inner_size(PhysicalSize::new(1280, 720))
        .with_position(winit::dpi::PhysicalPosition::new(100, 100))  // Explicitly position on screen
        .with_visible(true)
        .build(&event_loop)
        .unwrap());

    let window_id = window.id();
    println!("⚙️  INIT: Creating graphics state...");
    let mut state = pollster::block_on(State::new(&window));
    println!("✅ INIT: Graphics state created successfully");

    // Window visibility debugging
    println!("🪟 WINDOW DEBUG: Checking window properties...");
    println!("   Window title: '{}'", window.title());
    println!("   Window visible: {:?}", window.is_visible());
    println!("   Window minimized: {:?}", window.is_minimized());
    println!("   Window size: {:?}", window.inner_size());
    println!("   Window position: {:?}", window.outer_position().unwrap_or_default());

    // Make sure window is visible and focused
    window.set_visible(true);
    window.focus_window();

    println!("🔧 Forcing window visibility and focus...");
    println!("   Post-fix visible: {:?}", window.is_visible());

    println!("══════════════════════════════════════════════════════════");
    println!("         Robin Voxel Engine - Interactive 3D Demo         ");
    println!("══════════════════════════════════════════════════════════");
    println!();
    println!("🎮 Controls:");
    println!("   WASD        - Move");
    println!("   Mouse       - Look around");
    println!("   Space/Shift - Move up/down");
    println!("   Left Click  - Remove voxel");
    println!("   Right Click - Place voxel");
    println!("   1-8         - Select voxel type");
    println!("   `  (tilde)  - Toggle debug console");
    println!();
    println!("📦 Voxel Types:");
    println!("   1 - Stone    5 - Water");
    println!("   2 - Dirt     6 - Wood");
    println!("   3 - Grass    7 - Crystal (emissive)");
    println!("   4 - Sand     8 - Lava (emissive)");
    println!("══════════════════════════════════════════════════════════");

    let window_clone = Arc::clone(&window);
    event_loop.run(move |event, elwt| {
        // Use polling to keep window persistent and responsive
        elwt.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent {
                ref event,
                window_id: win_id,
            } if win_id == window_id => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested => elwt.exit(),
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { .. } => {
                            state.resize(state.size);
                        }
                        WindowEvent::RedrawRequested => {
                            println!("🔥 REDRAW REQUESTED - Starting render cycle");
                            state.update();
                            println!("🔄 UPDATE complete, calling render...");
                            match state.render() {
                                Ok(_) => {
                                    println!("✅ RENDER: Frame rendered successfully");
                                }
                                Err(wgpu::SurfaceError::Lost) => {
                                    println!("⚠️  RENDER: Surface lost, resizing...");
                                    state.resize(state.size);
                                }
                                Err(wgpu::SurfaceError::OutOfMemory) => {
                                    println!("❌ RENDER: Out of memory!");
                                    elwt.exit();
                                }
                                Err(e) => {
                                    println!("❌ RENDER ERROR: {:?}", e);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                state.mouse_delta.0 += delta.0 as f32;
                state.mouse_delta.1 += delta.1 as f32;
            }
            Event::AboutToWait => {
                // Force continuous redraws for the interactive demo
                window_clone.request_redraw();

                // Keep the event loop alive by scheduling another event
                std::thread::sleep(std::time::Duration::from_millis(16)); // ~60 FPS
            }
            _ => {}
        }
    });
}