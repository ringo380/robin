// Standalone Interactive Voxel Demo for macOS
// This is a self-contained demo that doesn't require the full Robin library

use std::time::{Duration, Instant};
use winit::{
    event::{Event, WindowEvent, DeviceEvent, ElementState, KeyboardInput, VirtualKeyCode, MouseButton},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window},
};
use wgpu::util::DeviceExt;

// Math utilities
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    color: [f32; 3],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],
    light_pos: [f32; 4],
    eye_pos: [f32; 4],
    time: f32,
    _padding: [f32; 3],
}

// Simple voxel world
struct VoxelWorld {
    voxels: Vec<Vec<Vec<Option<VoxelType>>>>,
    size: usize,
}

#[derive(Clone, Copy, Debug)]
enum VoxelType {
    Stone,
    Grass,
    Dirt,
    Water,
    Crystal,
}

impl VoxelType {
    fn color(&self) -> [f32; 3] {
        match self {
            VoxelType::Stone => [0.5, 0.5, 0.5],
            VoxelType::Grass => [0.2, 0.8, 0.2],
            VoxelType::Dirt => [0.4, 0.3, 0.1],
            VoxelType::Water => [0.2, 0.4, 0.8],
            VoxelType::Crystal => [0.8, 0.3, 0.9],
        }
    }
}

impl VoxelWorld {
    fn new(size: usize) -> Self {
        let mut voxels = vec![vec![vec![None; size]; size]; size];

        // Create a simple terrain
        for x in 0..size {
            for z in 0..size {
                let height = 5 + ((x as f32 * 0.1).sin() * 2.0) as usize;
                for y in 0..height.min(size) {
                    voxels[x][y][z] = Some(if y == height - 1 {
                        VoxelType::Grass
                    } else if y > height - 3 {
                        VoxelType::Dirt
                    } else {
                        VoxelType::Stone
                    });
                }
            }
        }

        // Add some crystals
        for _ in 0..10 {
            let x = (rand::random::<f32>() * size as f32) as usize;
            let z = (rand::random::<f32>() * size as f32) as usize;
            for y in 0..size {
                if voxels[x.min(size-1)][y][z.min(size-1)].is_some() {
                    if y + 1 < size {
                        voxels[x.min(size-1)][y + 1][z.min(size-1)] = Some(VoxelType::Crystal);
                    }
                    break;
                }
            }
        }

        Self { voxels, size }
    }

    fn generate_mesh(&self) -> Vec<Vertex> {
        let mut vertices = Vec::new();

        for x in 0..self.size {
            for y in 0..self.size {
                for z in 0..self.size {
                    if let Some(voxel_type) = self.voxels[x][y][z] {
                        let color = voxel_type.color();
                        let pos = [x as f32, y as f32, z as f32];

                        // Check each face and add if exposed
                        // Front face (z+)
                        if z + 1 >= self.size || self.voxels[x][y][z + 1].is_none() {
                            add_face(&mut vertices, pos, color, 0);
                        }
                        // Back face (z-)
                        if z == 0 || self.voxels[x][y][z - 1].is_none() {
                            add_face(&mut vertices, pos, color, 1);
                        }
                        // Right face (x+)
                        if x + 1 >= self.size || self.voxels[x + 1][y][z].is_none() {
                            add_face(&mut vertices, pos, color, 2);
                        }
                        // Left face (x-)
                        if x == 0 || self.voxels[x - 1][y][z].is_none() {
                            add_face(&mut vertices, pos, color, 3);
                        }
                        // Top face (y+)
                        if y + 1 >= self.size || self.voxels[x][y + 1][z].is_none() {
                            add_face(&mut vertices, pos, color, 4);
                        }
                        // Bottom face (y-)
                        if y == 0 || self.voxels[x][y - 1][z].is_none() {
                            add_face(&mut vertices, pos, color, 5);
                        }
                    }
                }
            }
        }

        vertices
    }
}

fn add_face(vertices: &mut Vec<Vertex>, pos: [f32; 3], color: [f32; 3], face: usize) {
    let x = pos[0];
    let y = pos[1];
    let z = pos[2];

    let face_vertices = match face {
        0 => vec![ // Front (z+)
            Vertex { position: [x, y, z + 1.0], normal: [0.0, 0.0, 1.0], color },
            Vertex { position: [x + 1.0, y, z + 1.0], normal: [0.0, 0.0, 1.0], color },
            Vertex { position: [x + 1.0, y + 1.0, z + 1.0], normal: [0.0, 0.0, 1.0], color },
            Vertex { position: [x, y, z + 1.0], normal: [0.0, 0.0, 1.0], color },
            Vertex { position: [x + 1.0, y + 1.0, z + 1.0], normal: [0.0, 0.0, 1.0], color },
            Vertex { position: [x, y + 1.0, z + 1.0], normal: [0.0, 0.0, 1.0], color },
        ],
        1 => vec![ // Back (z-)
            Vertex { position: [x, y, z], normal: [0.0, 0.0, -1.0], color },
            Vertex { position: [x, y + 1.0, z], normal: [0.0, 0.0, -1.0], color },
            Vertex { position: [x + 1.0, y + 1.0, z], normal: [0.0, 0.0, -1.0], color },
            Vertex { position: [x, y, z], normal: [0.0, 0.0, -1.0], color },
            Vertex { position: [x + 1.0, y + 1.0, z], normal: [0.0, 0.0, -1.0], color },
            Vertex { position: [x + 1.0, y, z], normal: [0.0, 0.0, -1.0], color },
        ],
        2 => vec![ // Right (x+)
            Vertex { position: [x + 1.0, y, z], normal: [1.0, 0.0, 0.0], color },
            Vertex { position: [x + 1.0, y + 1.0, z], normal: [1.0, 0.0, 0.0], color },
            Vertex { position: [x + 1.0, y + 1.0, z + 1.0], normal: [1.0, 0.0, 0.0], color },
            Vertex { position: [x + 1.0, y, z], normal: [1.0, 0.0, 0.0], color },
            Vertex { position: [x + 1.0, y + 1.0, z + 1.0], normal: [1.0, 0.0, 0.0], color },
            Vertex { position: [x + 1.0, y, z + 1.0], normal: [1.0, 0.0, 0.0], color },
        ],
        3 => vec![ // Left (x-)
            Vertex { position: [x, y, z], normal: [-1.0, 0.0, 0.0], color },
            Vertex { position: [x, y, z + 1.0], normal: [-1.0, 0.0, 0.0], color },
            Vertex { position: [x, y + 1.0, z + 1.0], normal: [-1.0, 0.0, 0.0], color },
            Vertex { position: [x, y, z], normal: [-1.0, 0.0, 0.0], color },
            Vertex { position: [x, y + 1.0, z + 1.0], normal: [-1.0, 0.0, 0.0], color },
            Vertex { position: [x, y + 1.0, z], normal: [-1.0, 0.0, 0.0], color },
        ],
        4 => vec![ // Top (y+)
            Vertex { position: [x, y + 1.0, z], normal: [0.0, 1.0, 0.0], color },
            Vertex { position: [x, y + 1.0, z + 1.0], normal: [0.0, 1.0, 0.0], color },
            Vertex { position: [x + 1.0, y + 1.0, z + 1.0], normal: [0.0, 1.0, 0.0], color },
            Vertex { position: [x, y + 1.0, z], normal: [0.0, 1.0, 0.0], color },
            Vertex { position: [x + 1.0, y + 1.0, z + 1.0], normal: [0.0, 1.0, 0.0], color },
            Vertex { position: [x + 1.0, y + 1.0, z], normal: [0.0, 1.0, 0.0], color },
        ],
        _ => vec![ // Bottom (y-)
            Vertex { position: [x, y, z], normal: [0.0, -1.0, 0.0], color },
            Vertex { position: [x + 1.0, y, z], normal: [0.0, -1.0, 0.0], color },
            Vertex { position: [x + 1.0, y, z + 1.0], normal: [0.0, -1.0, 0.0], color },
            Vertex { position: [x, y, z], normal: [0.0, -1.0, 0.0], color },
            Vertex { position: [x + 1.0, y, z + 1.0], normal: [0.0, -1.0, 0.0], color },
            Vertex { position: [x, y, z + 1.0], normal: [0.0, -1.0, 0.0], color },
        ],
    };

    vertices.extend(face_vertices);
}

// Simple random number generation
mod rand {
    use std::sync::atomic::{AtomicU64, Ordering};

    static SEED: AtomicU64 = AtomicU64::new(0x123456789ABCDEF0);

    pub fn random<T>() -> T
    where T: Random {
        T::random()
    }

    pub trait Random {
        fn random() -> Self;
    }

    impl Random for f32 {
        fn random() -> Self {
            let mut seed = SEED.load(Ordering::Relaxed);
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            SEED.store(seed, Ordering::Relaxed);
            ((seed >> 32) as f32) / (u32::MAX as f32)
        }
    }
}

// Camera controller
struct Camera {
    position: [f32; 3],
    yaw: f32,
    pitch: f32,
}

impl Camera {
    fn new() -> Self {
        Self {
            position: [15.0, 10.0, 15.0],
            yaw: -45.0_f32.to_radians(),
            pitch: -20.0_f32.to_radians(),
        }
    }

    fn view_matrix(&self) -> [[f32; 4]; 4] {
        let cos_pitch = self.pitch.cos();
        let sin_pitch = self.pitch.sin();
        let cos_yaw = self.yaw.cos();
        let sin_yaw = self.yaw.sin();

        let xaxis = [cos_yaw, 0.0, -sin_yaw];
        let yaxis = [sin_yaw * sin_pitch, cos_pitch, cos_yaw * sin_pitch];
        let zaxis = [sin_yaw * cos_pitch, -sin_pitch, cos_yaw * cos_pitch];

        let x = -(xaxis[0] * self.position[0] + xaxis[1] * self.position[1] + xaxis[2] * self.position[2]);
        let y = -(yaxis[0] * self.position[0] + yaxis[1] * self.position[1] + yaxis[2] * self.position[2]);
        let z = -(zaxis[0] * self.position[0] + zaxis[1] * self.position[1] + zaxis[2] * self.position[2]);

        [
            [xaxis[0], yaxis[0], zaxis[0], 0.0],
            [xaxis[1], yaxis[1], zaxis[1], 0.0],
            [xaxis[2], yaxis[2], zaxis[2], 0.0],
            [x, y, z, 1.0],
        ]
    }
}

fn projection_matrix(aspect_ratio: f32) -> [[f32; 4]; 4] {
    let fov = 60.0_f32.to_radians();
    let near = 0.1;
    let far = 1000.0;

    let f = 1.0 / (fov / 2.0).tan();

    [
        [f / aspect_ratio, 0.0, 0.0, 0.0],
        [0.0, f, 0.0, 0.0],
        [0.0, 0.0, (far + near) / (near - far), -1.0],
        [0.0, 0.0, (2.0 * far * near) / (near - far), 0.0],
    ]
}

fn multiply_matrices(a: [[f32; 4]; 4], b: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut result = [[0.0; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            for k in 0..4 {
                result[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    result
}

async fn run() {
    // Create window
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Robin Voxel Engine - Interactive 3D Demo")
        .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
        .build(&event_loop)
        .unwrap();

    // Create WGPU instance
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        dx12_shader_compiler: Default::default(),
        flags: wgpu::InstanceFlags::default(),
        gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
    });

    let surface = unsafe { instance.create_surface(&window).unwrap() };

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Device"),
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            },
            None,
        )
        .await
        .unwrap();

    let size = window.inner_size();
    let surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface.get_capabilities(&adapter).formats[0],
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::AutoVsync,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: vec![],
    };
    surface.configure(&device, &surface_config);

    // Create shader
    let shader_source = r#"
struct Uniforms {
    view_proj: mat4x4<f32>,
    light_pos: vec4<f32>,
    eye_pos: vec4<f32>,
    time: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = vec4<f32>(in.position, 1.0);
    out.clip_position = uniforms.view_proj * world_pos;
    out.world_position = in.position;
    out.normal = in.normal;
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let light_dir = normalize(uniforms.light_pos.xyz - in.world_position);
    let view_dir = normalize(uniforms.eye_pos.xyz - in.world_position);

    // Ambient
    let ambient = 0.3 * in.color;

    // Diffuse
    let diff = max(dot(in.normal, light_dir), 0.0);
    let diffuse = diff * in.color;

    // Specular
    let reflect_dir = reflect(-light_dir, in.normal);
    let spec = pow(max(dot(view_dir, reflect_dir), 0.0), 32.0);
    let specular = vec3<f32>(0.3) * spec;

    let final_color = ambient + diffuse + specular;

    return vec4<f32>(final_color, 1.0);
}
"#;

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Voxel Shader"),
        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    });

    // Create voxel world and mesh
    println!("Generating voxel world...");
    let world = VoxelWorld::new(32);
    let vertices = world.generate_mesh();
    println!("Generated {} vertices ({} triangles)", vertices.len(), vertices.len() / 3);

    // Create vertex buffer
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    // Create uniform buffer
    let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Uniform Buffer"),
        size: std::mem::size_of::<Uniforms>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // Create bind group layout
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Bind Group Layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    // Create bind group
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Bind Group"),
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });

    // Create pipeline
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Vertex>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: wgpu::VertexFormat::Float32x3,
                    },
                    wgpu::VertexAttribute {
                        offset: 12,
                        shader_location: 1,
                        format: wgpu::VertexFormat::Float32x3,
                    },
                    wgpu::VertexAttribute {
                        offset: 24,
                        shader_location: 2,
                        format: wgpu::VertexFormat::Float32x3,
                    },
                ],
            }],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    });

    // Camera and input state
    let mut camera = Camera::new();
    let mut keys_pressed = std::collections::HashSet::new();
    let start_time = Instant::now();

    println!("\n🎮 Controls:");
    println!("   WASD        - Move camera");
    println!("   Arrow Keys  - Look around");
    println!("   Space/Shift - Move up/down");
    println!("   ESC         - Exit");
    println!("\n✨ Voxel world ready! Enjoy the interactive 3D experience!");

    // Event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput {
                    input: KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                    ..
                } => {
                    match state {
                        ElementState::Pressed => {
                            keys_pressed.insert(keycode);
                            if keycode == VirtualKeyCode::Escape {
                                *control_flow = ControlFlow::Exit;
                            }
                        }
                        ElementState::Released => {
                            keys_pressed.remove(&keycode);
                        }
                    }
                }
                WindowEvent::Resized(new_size) => {
                    if new_size.width > 0 && new_size.height > 0 {
                        surface.configure(&device, &wgpu::SurfaceConfiguration {
                            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                            format: surface_config.format,
                            width: new_size.width,
                            height: new_size.height,
                            present_mode: wgpu::PresentMode::AutoVsync,
                            alpha_mode: wgpu::CompositeAlphaMode::Auto,
                            view_formats: vec![],
                        });
                    }
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                // Update camera based on input
                let speed = 0.5;
                let turn_speed = 0.05;

                if keys_pressed.contains(&VirtualKeyCode::W) {
                    camera.position[0] += camera.yaw.sin() * speed;
                    camera.position[2] += camera.yaw.cos() * speed;
                }
                if keys_pressed.contains(&VirtualKeyCode::S) {
                    camera.position[0] -= camera.yaw.sin() * speed;
                    camera.position[2] -= camera.yaw.cos() * speed;
                }
                if keys_pressed.contains(&VirtualKeyCode::A) {
                    camera.position[0] += (camera.yaw - std::f32::consts::PI / 2.0).sin() * speed;
                    camera.position[2] += (camera.yaw - std::f32::consts::PI / 2.0).cos() * speed;
                }
                if keys_pressed.contains(&VirtualKeyCode::D) {
                    camera.position[0] -= (camera.yaw - std::f32::consts::PI / 2.0).sin() * speed;
                    camera.position[2] -= (camera.yaw - std::f32::consts::PI / 2.0).cos() * speed;
                }
                if keys_pressed.contains(&VirtualKeyCode::Space) {
                    camera.position[1] += speed;
                }
                if keys_pressed.contains(&VirtualKeyCode::LShift) {
                    camera.position[1] -= speed;
                }
                if keys_pressed.contains(&VirtualKeyCode::Left) {
                    camera.yaw -= turn_speed;
                }
                if keys_pressed.contains(&VirtualKeyCode::Right) {
                    camera.yaw += turn_speed;
                }
                if keys_pressed.contains(&VirtualKeyCode::Up) {
                    camera.pitch = (camera.pitch + turn_speed).min(std::f32::consts::PI / 2.0 - 0.01);
                }
                if keys_pressed.contains(&VirtualKeyCode::Down) {
                    camera.pitch = (camera.pitch - turn_speed).max(-std::f32::consts::PI / 2.0 + 0.01);
                }

                // Render
                let output = surface.get_current_texture().unwrap();
                let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

                // Update uniforms
                let aspect_ratio = window.inner_size().width as f32 / window.inner_size().height as f32;
                let view_proj = multiply_matrices(projection_matrix(aspect_ratio), camera.view_matrix());
                let time = start_time.elapsed().as_secs_f32();

                let uniforms = Uniforms {
                    view_proj,
                    light_pos: [20.0, 30.0, 20.0, 1.0],
                    eye_pos: [camera.position[0], camera.position[1], camera.position[2], 1.0],
                    time,
                    _padding: [0.0; 3],
                };

                queue.write_buffer(&uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

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
                        depth_stencil_attachment: None,
                        occlusion_query_set: None,
                        timestamp_writes: None,
                    });

                    render_pass.set_pipeline(&pipeline);
                    render_pass.set_bind_group(0, &bind_group, &[]);
                    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    render_pass.draw(0..vertices.len() as u32, 0..1);
                }

                queue.submit(std::iter::once(encoder.finish()));
                output.present();
            }
            _ => {}
        }
    });
}

fn main() {
    println!("═══════════════════════════════════════════════════════════════");
    println!("         Robin Voxel Engine - Interactive 3D Demo             ");
    println!("═══════════════════════════════════════════════════════════════");

    pollster::block_on(run());
}