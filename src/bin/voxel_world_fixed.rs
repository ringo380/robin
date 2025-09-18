/// Fixed Voxel World Demo - Phase 5 Complete Fix
///
/// This is the corrected version of the full voxel demo with proper initialization
/// and guaranteed visible geometry.

use winit::{
    event::{Event, WindowEvent, KeyEvent, ElementState, MouseButton, DeviceEvent},
    event_loop::{EventLoop, ControlFlow},
    window::{WindowBuilder, Window},
    dpi::PhysicalSize,
    keyboard::{KeyCode, PhysicalKey},
};
use wgpu::util::DeviceExt;
use cgmath::{Matrix4, Point3, Vector3, Deg, perspective, InnerSpace, SquareMatrix};
use std::time::Instant;
use std::sync::Arc;
use std::collections::{HashMap, HashSet};

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
                    offset: (std::mem::size_of::<[f32; 3]>() * 2) as wgpu::BufferAddress,
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

#[derive(Debug, Clone, Copy, PartialEq)]
enum VoxelType {
    Air,
    Stone,
    Earth,
    Grass,
}

impl VoxelType {
    fn get_color(&self) -> [f32; 3] {
        match self {
            VoxelType::Air => [0.0, 0.0, 0.0],
            VoxelType::Stone => [0.5, 0.5, 0.5],
            VoxelType::Earth => [0.4, 0.3, 0.2],
            VoxelType::Grass => [0.2, 0.6, 0.2],
        }
    }
}

const CHUNK_SIZE: usize = 16;

struct VoxelChunk {
    position: (i32, i32, i32),
    voxels: Vec<VoxelType>,
}

impl VoxelChunk {
    fn new(position: (i32, i32, i32)) -> Self {
        let mut voxels = vec![VoxelType::Air; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE];

        // Generate simple terrain - guarantee visible geometry
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                // Simple height map
                let height = 4 + ((x as f32 * 0.2).sin() * 2.0 + (z as f32 * 0.2).cos() * 2.0) as usize;
                for y in 0..height.min(CHUNK_SIZE) {
                    let idx = Self::index(x, y, z);
                    voxels[idx] = if y == height - 1 {
                        VoxelType::Grass
                    } else if y > height - 3 {
                        VoxelType::Earth
                    } else {
                        VoxelType::Stone
                    };
                }
            }
        }

        Self { position, voxels }
    }

    fn index(x: usize, y: usize, z: usize) -> usize {
        x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE
    }

    fn get_voxel(&self, x: usize, y: usize, z: usize) -> VoxelType {
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
            return VoxelType::Air;
        }
        self.voxels[Self::index(x, y, z)]
    }
}

struct VoxelWorld {
    chunks: HashMap<(i32, i32, i32), VoxelChunk>,
}

impl VoxelWorld {
    fn new() -> Self {
        let mut chunks = HashMap::new();

        // Generate initial chunks around origin
        for cx in -1..=1 {
            for cz in -1..=1 {
                let chunk = VoxelChunk::new((cx, 0, cz));
                chunks.insert((cx, 0, cz), chunk);
            }
        }

        Self { chunks }
    }

    fn generate_mesh(&self) -> (Vec<Vertex>, Vec<u32>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for (chunk_pos, chunk) in &self.chunks {
            let chunk_offset = (
                chunk_pos.0 * CHUNK_SIZE as i32,
                chunk_pos.1 * CHUNK_SIZE as i32,
                chunk_pos.2 * CHUNK_SIZE as i32,
            );

            for x in 0..CHUNK_SIZE {
                for y in 0..CHUNK_SIZE {
                    for z in 0..CHUNK_SIZE {
                        let voxel = chunk.get_voxel(x, y, z);
                        if voxel == VoxelType::Air {
                            continue;
                        }

                        let color = voxel.get_color();
                        let world_x = chunk_offset.0 + x as i32;
                        let world_y = chunk_offset.1 + y as i32;
                        let world_z = chunk_offset.2 + z as i32;

                        // Check each face for visibility
                        let faces = [
                            // Top (Y+)
                            (y == CHUNK_SIZE - 1 || chunk.get_voxel(x, y + 1, z) == VoxelType::Air,
                             [0.0, 1.0, 0.0],
                             [
                                 [world_x as f32 - 0.5, world_y as f32 + 0.5, world_z as f32 - 0.5],
                                 [world_x as f32 + 0.5, world_y as f32 + 0.5, world_z as f32 - 0.5],
                                 [world_x as f32 + 0.5, world_y as f32 + 0.5, world_z as f32 + 0.5],
                                 [world_x as f32 - 0.5, world_y as f32 + 0.5, world_z as f32 + 0.5],
                             ]),
                            // Bottom (Y-)
                            (y == 0 || chunk.get_voxel(x, y - 1, z) == VoxelType::Air,
                             [0.0, -1.0, 0.0],
                             [
                                 [world_x as f32 - 0.5, world_y as f32 - 0.5, world_z as f32 + 0.5],
                                 [world_x as f32 + 0.5, world_y as f32 - 0.5, world_z as f32 + 0.5],
                                 [world_x as f32 + 0.5, world_y as f32 - 0.5, world_z as f32 - 0.5],
                                 [world_x as f32 - 0.5, world_y as f32 - 0.5, world_z as f32 - 0.5],
                             ]),
                            // Front (Z+)
                            (z == CHUNK_SIZE - 1 || chunk.get_voxel(x, y, z + 1) == VoxelType::Air,
                             [0.0, 0.0, 1.0],
                             [
                                 [world_x as f32 - 0.5, world_y as f32 - 0.5, world_z as f32 + 0.5],
                                 [world_x as f32 + 0.5, world_y as f32 - 0.5, world_z as f32 + 0.5],
                                 [world_x as f32 + 0.5, world_y as f32 + 0.5, world_z as f32 + 0.5],
                                 [world_x as f32 - 0.5, world_y as f32 + 0.5, world_z as f32 + 0.5],
                             ]),
                            // Back (Z-)
                            (z == 0 || chunk.get_voxel(x, y, z - 1) == VoxelType::Air,
                             [0.0, 0.0, -1.0],
                             [
                                 [world_x as f32 + 0.5, world_y as f32 - 0.5, world_z as f32 - 0.5],
                                 [world_x as f32 - 0.5, world_y as f32 - 0.5, world_z as f32 - 0.5],
                                 [world_x as f32 - 0.5, world_y as f32 + 0.5, world_z as f32 - 0.5],
                                 [world_x as f32 + 0.5, world_y as f32 + 0.5, world_z as f32 - 0.5],
                             ]),
                            // Right (X+)
                            (x == CHUNK_SIZE - 1 || chunk.get_voxel(x + 1, y, z) == VoxelType::Air,
                             [1.0, 0.0, 0.0],
                             [
                                 [world_x as f32 + 0.5, world_y as f32 - 0.5, world_z as f32 + 0.5],
                                 [world_x as f32 + 0.5, world_y as f32 - 0.5, world_z as f32 - 0.5],
                                 [world_x as f32 + 0.5, world_y as f32 + 0.5, world_z as f32 - 0.5],
                                 [world_x as f32 + 0.5, world_y as f32 + 0.5, world_z as f32 + 0.5],
                             ]),
                            // Left (X-)
                            (x == 0 || chunk.get_voxel(x - 1, y, z) == VoxelType::Air,
                             [-1.0, 0.0, 0.0],
                             [
                                 [world_x as f32 - 0.5, world_y as f32 - 0.5, world_z as f32 - 0.5],
                                 [world_x as f32 - 0.5, world_y as f32 - 0.5, world_z as f32 + 0.5],
                                 [world_x as f32 - 0.5, world_y as f32 + 0.5, world_z as f32 + 0.5],
                                 [world_x as f32 - 0.5, world_y as f32 + 0.5, world_z as f32 - 0.5],
                             ]),
                        ];

                        for (visible, normal, positions) in faces {
                            if visible {
                                let base_idx = vertices.len() as u32;

                                // Add 4 vertices for the face
                                for pos in positions {
                                    vertices.push(Vertex {
                                        position: pos,
                                        color,
                                        normal,
                                    });
                                }

                                // Add 6 indices (2 triangles)
                                indices.extend_from_slice(&[
                                    base_idx, base_idx + 1, base_idx + 2,
                                    base_idx, base_idx + 2, base_idx + 3,
                                ]);
                            }
                        }
                    }
                }
            }
        }

        println!("📊 Generated mesh: {} vertices, {} indices", vertices.len(), indices.len());

        // Ensure we always have at least some geometry
        if vertices.is_empty() {
            println!("⚠️  No geometry generated! Creating fallback cube...");
            // Create a single cube as fallback
            let size = 1.0;
            let color = [0.5, 0.5, 0.5];

            // Add vertices for a cube (6 faces * 4 vertices)
            for face in 0..6 {
                let normal = match face {
                    0 => [0.0, 1.0, 0.0],
                    1 => [0.0, -1.0, 0.0],
                    2 => [0.0, 0.0, 1.0],
                    3 => [0.0, 0.0, -1.0],
                    4 => [1.0, 0.0, 0.0],
                    _ => [-1.0, 0.0, 0.0],
                };

                let positions = match face {
                    0 => [[-size, size, -size], [size, size, -size], [size, size, size], [-size, size, size]],
                    1 => [[-size, -size, size], [size, -size, size], [size, -size, -size], [-size, -size, -size]],
                    2 => [[-size, -size, size], [size, -size, size], [size, size, size], [-size, size, size]],
                    3 => [[size, -size, -size], [-size, -size, -size], [-size, size, -size], [size, size, -size]],
                    4 => [[size, -size, size], [size, -size, -size], [size, size, -size], [size, size, size]],
                    _ => [[-size, -size, -size], [-size, -size, size], [-size, size, size], [-size, size, -size]],
                };

                let base_idx = vertices.len() as u32;
                for pos in positions {
                    vertices.push(Vertex { position: pos, color, normal });
                }
                indices.extend_from_slice(&[
                    base_idx, base_idx + 1, base_idx + 2,
                    base_idx, base_idx + 2, base_idx + 3,
                ]);
            }
        }

        (vertices, indices)
    }
}

struct Camera {
    eye: Point3<f32>,
    target: Point3<f32>,
    up: Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        let view = Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = perspective(Deg(self.fovy), self.aspect, self.znear, self.zfar);
        proj * view
    }
}

struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    camera: Camera,
    voxel_world: VoxelWorld,
    start_time: Instant,
}

impl State {
    async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        // Initialize WGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: Some("Voxel Device"),
            },
            None,
        ).await.unwrap();

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
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Create camera
        let camera = Camera {
            eye: Point3::new(10.0, 10.0, 10.0),
            target: Point3::new(0.0, 5.0, 0.0),
            up: Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 60.0,
            znear: 0.1,
            zfar: 1000.0,
        };

        // Create voxel world and mesh
        let voxel_world = VoxelWorld::new();
        let (vertices, indices) = voxel_world.generate_mesh();

        // Create buffers
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        // Create uniforms
        let uniforms = Uniforms {
            view_proj: camera.build_view_projection_matrix().into(),
            view_pos: [camera.eye.x, camera.eye.y, camera.eye.z, 1.0],
            light_pos: [20.0, 30.0, 20.0, 1.0],
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

        // Shader
        let shader_source = r#"
struct Uniforms {
    view_proj: mat4x4<f32>,
    view_pos: vec4<f32>,
    light_pos: vec4<f32>,
    time: f32,
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
    @location(0) color: vec3<f32>,
    @location(1) world_pos: vec3<f32>,
    @location(2) normal: vec3<f32>,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.view_proj * vec4<f32>(model.position, 1.0);
    out.color = model.color;
    out.world_pos = model.position;
    out.normal = model.normal;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let light_dir = normalize(uniforms.light_pos.xyz - in.world_pos);
    let view_dir = normalize(uniforms.view_pos.xyz - in.world_pos);

    // Simple Phong shading
    let ambient = 0.15;
    let diffuse = max(dot(in.normal, light_dir), 0.0) * 0.7;
    let specular_strength = 0.5;
    let reflect_dir = reflect(-light_dir, in.normal);
    let spec = pow(max(dot(view_dir, reflect_dir), 0.0), 32.0);
    let specular = specular_strength * spec;

    let final_color = in.color * (ambient + diffuse) + vec3<f32>(specular);
    return vec4<f32>(final_color, 1.0);
}
"#;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Voxel Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        // Create render pipeline
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Voxel Pipeline"),
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        println!("✅ Voxel world initialized with {} indices", indices.len());

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
            voxel_world,
            start_time: Instant::now(),
        }
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.camera.aspect = new_size.width as f32 / new_size.height as f32;
        }
    }

    fn update(&mut self) {
        let elapsed = self.start_time.elapsed().as_secs_f32();

        // Rotate camera around world
        let radius = 20.0;
        self.camera.eye = Point3::new(
            radius * (elapsed * 0.5).cos(),
            10.0 + 5.0 * (elapsed * 0.3).sin(),
            radius * (elapsed * 0.5).sin(),
        );

        // Update uniforms
        let uniforms = Uniforms {
            view_proj: self.camera.build_view_projection_matrix().into(),
            view_pos: [self.camera.eye.x, self.camera.eye.y, self.camera.eye.z, 1.0],
            light_pos: [20.0, 30.0, 20.0, 1.0],
            time: elapsed,
            _padding: [0.0; 3],
        };

        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[uniforms]),
        );
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.update();

        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

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
                            r: 0.5,
                            g: 0.8,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn main() {
    println!("🚀 Robin Engine - Fixed Voxel World Demo");
    println!("=========================================");

    env_logger::init();

    let event_loop = EventLoop::new().expect("Failed to create event loop");

    let window = Arc::new(WindowBuilder::new()
        .with_title("Robin Engine - Voxel World")
        .with_inner_size(PhysicalSize::new(1280, 720))
        .with_position(winit::dpi::PhysicalPosition::new(100, 100))
        .with_visible(true)
        .build(&event_loop)
        .expect("Failed to create window"));

    window.set_visible(true);
    window.focus_window();

    let mut state = pollster::block_on(State::new(Arc::clone(&window)));

    println!("\n🎮 Controls:");
    println!("   ESC - Exit");
    println!("\n🌍 Explore the voxel world!");

    let _ = event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent { event, window_id } if window_id == window.id() => {
                match event {
                    WindowEvent::CloseRequested => {
                        println!("👋 Exiting...");
                        elwt.exit();
                    }
                    WindowEvent::Resized(physical_size) => {
                        state.resize(physical_size);
                    }
                    WindowEvent::KeyboardInput { event, .. } => {
                        if event.state == ElementState::Pressed {
                            if let PhysicalKey::Code(KeyCode::Escape) = event.physical_key {
                                elwt.exit();
                            }
                        }
                    }
                    WindowEvent::RedrawRequested => {
                        match state.render() {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                            Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                            Err(e) => eprintln!("Render error: {:?}", e),
                        }
                    }
                    _ => {}
                }
            }
            Event::AboutToWait => {
                window.request_redraw();
                std::thread::sleep(std::time::Duration::from_millis(16));
            }
            _ => {}
        }
    });
}