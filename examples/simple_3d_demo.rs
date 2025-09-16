// Simple 3D Demo for Robin Engine
// A minimal working example that demonstrates 3D graphics with less complexity

use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};
use wgpu::util::DeviceExt;
use cgmath::{Matrix4, Point3, Vector3, Rad, perspective, SquareMatrix};
use std::time::Instant;

fn main() {
    env_logger::init();
    
    println!("🎮 Robin Engine - Simple 3D Graphics Demo");
    println!("=========================================");
    println!("A minimal working example of 3D rendering with wgpu");
    
    // Create event loop and window
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("Robin Engine - Simple 3D Demo")
        .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
        .build(&event_loop)
        .unwrap();
    
    // Initialize the demo
    let mut demo = pollster::block_on(Simple3DDemo::new(&window));
    
    println!("\n📋 Controls:");
    println!("  Arrow Keys - Rotate camera");
    println!("  Space      - Reset camera");
    println!("  Escape     - Exit");
    
    // Run the event loop
    let mut last_frame_time = Instant::now();
    
    event_loop.run(move |event, target| {
        match event {
            Event::WindowEvent { ref event, window_id } if window_id == window.id() => {
                match event {
                    WindowEvent::CloseRequested 
                    | WindowEvent::KeyboardInput {
                        event: KeyEvent {
                            physical_key: PhysicalKey::Code(KeyCode::Escape),
                            state: ElementState::Pressed,
                            ..
                        },
                        ..
                    } => target.exit(),
                    
                    WindowEvent::Resized(physical_size) => {
                        demo.resize(*physical_size);
                    }
                    
                    WindowEvent::RedrawRequested => {
                        let current_time = Instant::now();
                        let delta_time = current_time.duration_since(last_frame_time).as_secs_f32();
                        last_frame_time = current_time;
                        
                        demo.update(delta_time);
                        match demo.render() {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                demo.resize(demo.size);
                            }
                            Err(wgpu::SurfaceError::OutOfMemory) => {
                                eprintln!("Out of memory!");
                                target.exit();
                            }
                            Err(wgpu::SurfaceError::Timeout) => {
                                log::warn!("Surface timeout");
                            }
                        }
                    }
                    
                    WindowEvent::KeyboardInput { event, .. } => {
                        demo.input(event);
                    }
                    
                    _ => {}
                }
            }
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        }
    }).unwrap();
}

// Simple vertex structure
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
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

// Camera uniform buffer
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],
}

// Main demo struct
struct Simple3DDemo {
    surface: wgpu::Surface<'static>,
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
    depth_texture: wgpu::TextureView,
    
    // Camera state
    camera_rotation_y: f32,
    camera_rotation_x: f32,
    time: f32,
    
    // Input state
    arrow_left: bool,
    arrow_right: bool,
    arrow_up: bool,
    arrow_down: bool,
}

impl Simple3DDemo {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        
        // Initialize WGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        let surface = instance.create_surface(window).unwrap();
        
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        
        println!("🔧 GPU: {}", adapter.get_info().name);
        println!("🔧 Backend: {:?}", adapter.get_info().backend);
        
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();
        
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
        
        // Create shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Simple 3D Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/simple_3d.wgsl").into()),
        });
        
        // Create uniform buffer
        let uniforms = Uniforms {
            view_proj: cgmath::Matrix4::identity().into(),
        };
        
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        
        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
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
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
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
        
        // Create colorful cube geometry
        let (vertices, indices) = create_cube_geometry();
        
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        
        let num_indices = indices.len() as u32;
        
        // Create depth texture
        let depth_texture = create_depth_texture(&device, &config);
        
        println!("✅ 3D Graphics initialized successfully!");
        
        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            uniform_buffer,
            uniform_bind_group,
            depth_texture,
            camera_rotation_y: 0.0,
            camera_rotation_x: 0.0,
            time: 0.0,
            arrow_left: false,
            arrow_right: false,
            arrow_up: false,
            arrow_down: false,
        }
    }
    
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture = create_depth_texture(&self.device, &self.config);
        }
    }
    
    fn input(&mut self, event: &KeyEvent) {
        let pressed = event.state == ElementState::Pressed;
        match event.physical_key {
            PhysicalKey::Code(KeyCode::ArrowLeft) => self.arrow_left = pressed,
            PhysicalKey::Code(KeyCode::ArrowRight) => self.arrow_right = pressed,
            PhysicalKey::Code(KeyCode::ArrowUp) => self.arrow_up = pressed,
            PhysicalKey::Code(KeyCode::ArrowDown) => self.arrow_down = pressed,
            PhysicalKey::Code(KeyCode::Space) if pressed => {
                self.camera_rotation_x = 0.0;
                self.camera_rotation_y = 0.0;
            }
            _ => {}
        }
    }
    
    fn update(&mut self, dt: f32) {
        self.time += dt;
        
        // Update camera rotation based on input
        let rotation_speed = 2.0;
        if self.arrow_left {
            self.camera_rotation_y -= rotation_speed * dt;
        }
        if self.arrow_right {
            self.camera_rotation_y += rotation_speed * dt;
        }
        if self.arrow_up {
            self.camera_rotation_x -= rotation_speed * dt;
        }
        if self.arrow_down {
            self.camera_rotation_x += rotation_speed * dt;
        }
        
        // Clamp vertical rotation
        self.camera_rotation_x = self.camera_rotation_x.clamp(-1.5, 1.5);
        
        // Create camera matrix
        let eye = Point3::new(
            self.camera_rotation_y.cos() * self.camera_rotation_x.cos() * 5.0,
            self.camera_rotation_x.sin() * 5.0,
            self.camera_rotation_y.sin() * self.camera_rotation_x.cos() * 5.0,
        );
        
        let target = Point3::new(0.0, 0.0, 0.0);
        let up = Vector3::unit_y();
        
        let view = Matrix4::look_at_rh(eye, target, up);
        let proj = perspective(
            Rad(std::f32::consts::FRAC_PI_4),
            self.config.width as f32 / self.config.height as f32,
            0.1,
            100.0,
        );
        
        let uniforms = Uniforms {
            view_proj: (proj * view).into(),
        };
        
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }
    
    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
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
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }
        
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        
        Ok(())
    }
}

// Create a colorful cube
fn create_cube_geometry() -> (Vec<Vertex>, Vec<u16>) {
    let vertices = vec![
        // Front face (red)
        Vertex { position: [-1.0, -1.0,  1.0], color: [1.0, 0.0, 0.0] },
        Vertex { position: [ 1.0, -1.0,  1.0], color: [1.0, 0.0, 0.0] },
        Vertex { position: [ 1.0,  1.0,  1.0], color: [1.0, 0.0, 0.0] },
        Vertex { position: [-1.0,  1.0,  1.0], color: [1.0, 0.0, 0.0] },
        
        // Back face (green)
        Vertex { position: [-1.0, -1.0, -1.0], color: [0.0, 1.0, 0.0] },
        Vertex { position: [-1.0,  1.0, -1.0], color: [0.0, 1.0, 0.0] },
        Vertex { position: [ 1.0,  1.0, -1.0], color: [0.0, 1.0, 0.0] },
        Vertex { position: [ 1.0, -1.0, -1.0], color: [0.0, 1.0, 0.0] },
        
        // Top face (blue)
        Vertex { position: [-1.0,  1.0, -1.0], color: [0.0, 0.0, 1.0] },
        Vertex { position: [-1.0,  1.0,  1.0], color: [0.0, 0.0, 1.0] },
        Vertex { position: [ 1.0,  1.0,  1.0], color: [0.0, 0.0, 1.0] },
        Vertex { position: [ 1.0,  1.0, -1.0], color: [0.0, 0.0, 1.0] },
        
        // Bottom face (yellow)
        Vertex { position: [-1.0, -1.0, -1.0], color: [1.0, 1.0, 0.0] },
        Vertex { position: [ 1.0, -1.0, -1.0], color: [1.0, 1.0, 0.0] },
        Vertex { position: [ 1.0, -1.0,  1.0], color: [1.0, 1.0, 0.0] },
        Vertex { position: [-1.0, -1.0,  1.0], color: [1.0, 1.0, 0.0] },
        
        // Right face (magenta)
        Vertex { position: [ 1.0, -1.0, -1.0], color: [1.0, 0.0, 1.0] },
        Vertex { position: [ 1.0,  1.0, -1.0], color: [1.0, 0.0, 1.0] },
        Vertex { position: [ 1.0,  1.0,  1.0], color: [1.0, 0.0, 1.0] },
        Vertex { position: [ 1.0, -1.0,  1.0], color: [1.0, 0.0, 1.0] },
        
        // Left face (cyan)
        Vertex { position: [-1.0, -1.0, -1.0], color: [0.0, 1.0, 1.0] },
        Vertex { position: [-1.0, -1.0,  1.0], color: [0.0, 1.0, 1.0] },
        Vertex { position: [-1.0,  1.0,  1.0], color: [0.0, 1.0, 1.0] },
        Vertex { position: [-1.0,  1.0, -1.0], color: [0.0, 1.0, 1.0] },
    ];
    
    let indices = vec![
        0,  1,  2,   2,  3,  0,  // front
        4,  5,  6,   6,  7,  4,  // back
        8,  9,  10,  10, 11, 8,  // top
        12, 13, 14,  14, 15, 12, // bottom
        16, 17, 18,  18, 19, 16, // right
        20, 21, 22,  22, 23, 20, // left
    ];
    
    (vertices, indices)
}

fn create_depth_texture(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> wgpu::TextureView {
    let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Depth Texture"),
        size: wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    
    depth_texture.create_view(&wgpu::TextureViewDescriptor::default())
}