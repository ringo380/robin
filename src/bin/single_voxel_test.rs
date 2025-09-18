/// Single Voxel Test - Phase 4 of Robin Engine Validation
///
/// This test renders a single rotating voxel cube to validate
/// the voxel mesh generation at its simplest level.

use winit::{
    event::{Event, WindowEvent},
    event_loop::{EventLoop, ControlFlow},
    window::{WindowBuilder, Window},
    dpi::PhysicalSize,
};
use wgpu::util::DeviceExt;
use cgmath::{Matrix4, Point3, Vector3, Deg, perspective, SquareMatrix};
use std::time::Instant;
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
    light_pos: [f32; 4],
}

fn create_cube_vertices() -> (Vec<Vertex>, Vec<u32>) {
    let size = 0.5;
    let color = [0.5, 0.7, 1.0]; // Light blue

    // Define the 8 corners of a cube
    let positions = [
        [-size, -size, -size], // 0
        [ size, -size, -size], // 1
        [ size,  size, -size], // 2
        [-size,  size, -size], // 3
        [-size, -size,  size], // 4
        [ size, -size,  size], // 5
        [ size,  size,  size], // 6
        [-size,  size,  size], // 7
    ];

    // Define the 6 faces (each as 2 triangles)
    let faces = [
        // Front face (z = size)
        ([4, 5, 6, 7], [0.0, 0.0, 1.0]),
        // Back face (z = -size)
        ([1, 0, 3, 2], [0.0, 0.0, -1.0]),
        // Top face (y = size)
        ([3, 7, 6, 2], [0.0, 1.0, 0.0]),
        // Bottom face (y = -size)
        ([0, 1, 5, 4], [0.0, -1.0, 0.0]),
        // Right face (x = size)
        ([5, 1, 2, 6], [1.0, 0.0, 0.0]),
        // Left face (x = -size)
        ([0, 4, 7, 3], [-1.0, 0.0, 0.0]),
    ];

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for (face_indices, normal) in faces.iter() {
        let base_index = vertices.len() as u32;

        // Add 4 vertices for this face
        for &i in face_indices {
            vertices.push(Vertex {
                position: positions[i],
                color,
                normal: *normal,
            });
        }

        // Add 2 triangles (6 indices) for this face
        indices.push(base_index);
        indices.push(base_index + 1);
        indices.push(base_index + 2);

        indices.push(base_index);
        indices.push(base_index + 2);
        indices.push(base_index + 3);
    }

    println!("📦 Created cube: {} vertices, {} indices", vertices.len(), indices.len());
    (vertices, indices)
}

const SHADER_SOURCE: &str = r#"
struct Uniforms {
    view_proj: mat4x4<f32>,
    light_pos: vec4<f32>,
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
    @location(1) normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_position = model.position;
    out.clip_position = uniforms.view_proj * vec4<f32>(world_position, 1.0);
    out.color = model.color;
    out.normal = model.normal;
    out.world_position = world_position;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Simple directional lighting
    let light_dir = normalize(uniforms.light_pos.xyz - in.world_position);
    let ambient = 0.2;
    let diffuse = max(dot(in.normal, light_dir), 0.0);
    let brightness = ambient + diffuse * 0.8;

    return vec4<f32>(in.color * brightness, 1.0);
}
"#;

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
    uniforms: Uniforms,
    start_time: Instant,
    frame_count: u64,
}

impl State {
    async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        println!("🎨 Initializing WGPU...");

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
        ).await.expect("Failed to find adapter");

        println!("   ✅ Adapter: {:?}", adapter.get_info().name);

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: Some("Single Voxel Device"),
            },
            None,
        ).await.expect("Failed to create device");

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
        println!("   ✅ Surface configured");

        // Create cube geometry
        let (vertices, indices) = create_cube_vertices();

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

        // Create uniform buffer
        let uniforms = Uniforms {
            view_proj: Matrix4::identity().into(),
            light_pos: [5.0, 5.0, 5.0, 1.0],
        };

        let uniform_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Uniform Buffer"),
                contents: bytemuck::cast_slice(&[uniforms]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        // Create shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Voxel Shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SOURCE.into()),
        });

        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
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

        println!("   ✅ Pipeline created");

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
            uniforms,
            start_time: Instant::now(),
            frame_count: 0,
        }
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn update(&mut self) {
        let elapsed = self.start_time.elapsed().as_secs_f32();

        // Create view-projection matrix with rotation
        let eye = Point3::new(3.0 * elapsed.cos(), 2.0, 3.0 * elapsed.sin());
        let target = Point3::new(0.0, 0.0, 0.0);
        let up = Vector3::unit_y();

        let view = Matrix4::look_at_rh(eye, target, up);
        let proj = perspective(
            Deg(45.0),
            self.size.width as f32 / self.size.height as f32,
            0.1,
            100.0,
        );

        self.uniforms.view_proj = (proj * view).into();

        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniforms]),
        );
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.frame_count += 1;
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
                            r: 0.05,
                            g: 0.05,
                            b: 0.1,
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

        if self.frame_count % 60 == 0 {
            let elapsed = self.start_time.elapsed().as_secs_f32();
            let fps = self.frame_count as f32 / elapsed;
            println!("🎬 Frame {} | FPS: {:.2}", self.frame_count, fps);
        }

        Ok(())
    }
}

fn main() {
    println!("🚀 Robin Engine - Single Voxel Test");
    println!("====================================");

    env_logger::init();

    let event_loop = EventLoop::new().expect("Failed to create event loop");

    let window = Arc::new(WindowBuilder::new()
        .with_title("Robin Engine - Single Voxel Test")
        .with_inner_size(PhysicalSize::new(1280, 720))
        .with_position(winit::dpi::PhysicalPosition::new(100, 100))
        .with_visible(true)
        .build(&event_loop)
        .expect("Failed to create window"));

    window.set_visible(true);
    window.focus_window();

    let mut state = pollster::block_on(State::new(Arc::clone(&window)));
    println!("✅ Ready!");

    println!("\n🎮 Controls:");
    println!("   ESC - Exit");
    println!("\n🧊 Watch the voxel cube rotate!");

    let _ = event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent { event, window_id } if window_id == window.id() => {
                match event {
                    WindowEvent::CloseRequested => elwt.exit(),
                    WindowEvent::Resized(physical_size) => state.resize(physical_size),
                    WindowEvent::KeyboardInput { event, .. } => {
                        if event.state == winit::event::ElementState::Pressed {
                            if let winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape) = event.physical_key {
                                elwt.exit();
                            }
                        }
                    }
                    WindowEvent::RedrawRequested => {
                        match state.render() {
                            Ok(_) => {}
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