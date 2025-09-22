use winit::{
    application::ApplicationHandler,
    event::{Event, WindowEvent, DeviceEvent, ElementState, MouseButton, KeyEvent},
    event_loop::{EventLoop, ControlFlow},
    window::{Window, WindowBuilder},
    keyboard::{NamedKey, Key},
    dpi::PhysicalSize,
};
use wgpu::util::DeviceExt;
use cgmath::prelude::*;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x3
    ];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

struct Camera {
    position: cgmath::Point3<f32>,
    yaw: cgmath::Rad<f32>,
    pitch: cgmath::Rad<f32>,
}

impl Camera {
    fn new() -> Self {
        Self {
            position: cgmath::Point3::new(0.0, 1.0, 3.0),
            yaw: cgmath::Rad(0.0),
            pitch: cgmath::Rad(0.0),
        }
    }

    fn calc_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(
            self.position,
            self.position + self.get_forward(),
            cgmath::Vector3::unit_y(),
        );
        let proj = cgmath::perspective(
            cgmath::Deg(45.0),
            1.0,
            0.1,
            100.0
        );
        proj * view
    }

    fn get_forward(&self) -> cgmath::Vector3<f32> {
        cgmath::Vector3::new(
            self.yaw.0.cos() * self.pitch.0.cos(),
            self.pitch.0.sin(),
            self.yaw.0.sin() * self.pitch.0.cos(),
        ).normalize()
    }
}

struct EngineUIDemo {
    window: Option<Window>,
    surface: Option<wgpu::Surface<'static>>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    config: Option<wgpu::SurfaceConfiguration>,
    size: winit::dpi::PhysicalSize<u32>,

    // Demo state
    camera: Camera,
    camera_buffer: Option<wgpu::Buffer>,
    camera_bind_group: Option<wgpu::BindGroup>,
    render_pipeline: Option<wgpu::RenderPipeline>,
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    num_indices: u32,

    // Input state
    keys: std::collections::HashSet<String>,
    mouse_delta: (f64, f64),
    cursor_grabbed: bool,

    // UI state
    show_tool_palette: bool,
    show_asset_browser: bool,
    show_properties: bool,
    demo_time: f32,
}

impl EngineUIDemo {
    fn new() -> Self {
        Self {
            window: None,
            surface: None,
            device: None,
            queue: None,
            config: None,
            size: PhysicalSize::new(1200, 800),
            camera: Camera::new(),
            camera_buffer: None,
            camera_bind_group: None,
            render_pipeline: None,
            vertex_buffer: None,
            index_buffer: None,
            num_indices: 0,
            keys: std::collections::HashSet::new(),
            mouse_delta: (0.0, 0.0),
            cursor_grabbed: false,
            show_tool_palette: true,
            show_asset_browser: false,
            show_properties: true,
            demo_time: 0.0,
        }
    }

    async fn initialize_graphics(&mut self, window: &Window) -> Result<(), Box<dyn std::error::Error>> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window)?;

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.unwrap();

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

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: self.size.width,
            height: self.size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Create camera uniform buffer
        let camera_uniform = CameraUniform {
            view_proj: self.camera.calc_matrix().into(),
        };

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("camera_bind_group_layout"),
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        // Create shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout],
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // Create demo scene vertices (a colorful cube)
        let vertices = &[
            // Front face
            Vertex { position: [-1.0, -1.0,  1.0], color: [1.0, 0.0, 0.0] },
            Vertex { position: [ 1.0, -1.0,  1.0], color: [0.0, 1.0, 0.0] },
            Vertex { position: [ 1.0,  1.0,  1.0], color: [0.0, 0.0, 1.0] },
            Vertex { position: [-1.0,  1.0,  1.0], color: [1.0, 1.0, 0.0] },

            // Back face
            Vertex { position: [-1.0, -1.0, -1.0], color: [1.0, 0.0, 1.0] },
            Vertex { position: [ 1.0, -1.0, -1.0], color: [0.0, 1.0, 1.0] },
            Vertex { position: [ 1.0,  1.0, -1.0], color: [0.5, 0.5, 0.5] },
            Vertex { position: [-1.0,  1.0, -1.0], color: [1.0, 1.0, 1.0] },
        ];

        let indices = &[
            0,1,2, 2,3,0, // front
            4,7,6, 6,5,4, // back
            0,4,5, 5,1,0, // bottom
            2,6,7, 7,3,2, // top
            0,3,7, 7,4,0, // left
            1,5,6, 6,2,1, // right
        ];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        self.surface = Some(surface);
        self.device = Some(device);
        self.queue = Some(queue);
        self.config = Some(config);
        self.camera_buffer = Some(camera_buffer);
        self.camera_bind_group = Some(camera_bind_group);
        self.render_pipeline = Some(render_pipeline);
        self.vertex_buffer = Some(vertex_buffer);
        self.index_buffer = Some(index_buffer);
        self.num_indices = indices.len() as u32;

        Ok(())
    }

    fn update(&mut self, dt: f32) {
        self.demo_time += dt;

        let speed = 5.0;
        let sensitivity = 0.1;

        // Handle camera movement
        let forward = self.camera.get_forward();
        let right = forward.cross(cgmath::Vector3::unit_y()).normalize();

        if self.keys.contains("w") || self.keys.contains("W") {
            self.camera.position += forward * speed * dt;
        }
        if self.keys.contains("s") || self.keys.contains("S") {
            self.camera.position -= forward * speed * dt;
        }
        if self.keys.contains("a") || self.keys.contains("A") {
            self.camera.position -= right * speed * dt;
        }
        if self.keys.contains("d") || self.keys.contains("D") {
            self.camera.position += right * speed * dt;
        }

        // Handle mouse look
        if self.cursor_grabbed {
            self.camera.yaw += cgmath::Rad(self.mouse_delta.0 as f32 * sensitivity * dt);
            self.camera.pitch -= cgmath::Rad(self.mouse_delta.1 as f32 * sensitivity * dt);

            // Clamp pitch
            let max_pitch = cgmath::Rad(std::f32::consts::FRAC_PI_2 - 0.01);
            self.camera.pitch = self.camera.pitch.clamp(-max_pitch, max_pitch);
        }

        self.mouse_delta = (0.0, 0.0);

        // Update camera uniform
        if let (Some(camera_buffer), Some(queue)) = (&self.camera_buffer, &self.queue) {
            let camera_uniform = CameraUniform {
                view_proj: self.camera.calc_matrix().into(),
            };
            queue.write_buffer(
                camera_buffer,
                0,
                bytemuck::cast_slice(&[camera_uniform]),
            );
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let surface = self.surface.as_ref().unwrap();
        let device = self.device.as_ref().unwrap();
        let queue = self.queue.as_ref().unwrap();
        let render_pipeline = self.render_pipeline.as_ref().unwrap();
        let vertex_buffer = self.vertex_buffer.as_ref().unwrap();
        let index_buffer = self.index_buffer.as_ref().unwrap();
        let camera_bind_group = self.camera_bind_group.as_ref().unwrap();

        let output = surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
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
                            g: 0.12,
                            b: 0.16,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(render_pipeline);
            render_pass.set_bind_group(0, camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn print_ui_overlay(&self) {
        // Print UI information to console to simulate UI elements
        println!("\n=== ROBIN ENGINE - PHASE 4 UI DEMO ===");
        println!("Time: {:.2}s", self.demo_time);
        println!("Camera Position: {:.2}, {:.2}, {:.2}",
                 self.camera.position.x, self.camera.position.y, self.camera.position.z);

        if self.show_tool_palette {
            println!("\n[TAB] Tool Palette: [Selected: Build Tool]");
            println!("  🔨 Build Tool  🎨 Paint Tool  🔧 Edit Tool");
            println!("  ⚡ Physics    🌿 Terrain     💡 Lighting");
        }

        if self.show_asset_browser {
            println!("\n[T] Asset Browser:");
            println!("  📁 Models/    📁 Textures/   📁 Audio/");
            println!("  🧱 cube.obj   🎨 grass.png   🎵 ambient.ogg");
        }

        if self.show_properties {
            println!("\n[P] Properties Panel:");
            println!("  Material: Stone");
            println!("  Scale: 1.0x1.0x1.0");
            println!("  Rotation: 0°, 0°, 0°");
        }

        println!("\nControls:");
        println!("  WASD: Move Camera  Mouse: Look Around");
        println!("  Click: Grab Cursor  ESC: Release Cursor");
        println!("  TAB: Toggle Tools  T: Toggle Assets  P: Toggle Properties");
        println!("  Q: Quit Demo");

        println!("\n🎯 This demonstrates the Robin Engine Phase 4 UI system!");
        println!("Real 3D graphics with modern UI overlay concepts.");
    }
}

impl ApplicationHandler for EngineUIDemo {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = event_loop.create_window(
            WindowBuilder::new()
                .with_title("Robin Engine - Phase 4 UI Polish Demo")
                .with_inner_size(self.size)
        ).unwrap();

        let window_ref = &window;

        pollster::block_on(async {
            if let Err(e) = self.initialize_graphics(window_ref).await {
                eprintln!("Failed to initialize graphics: {}", e);
                event_loop.exit();
                return;
            }
        });

        self.window = Some(window);

        // Show initial UI state
        self.print_ui_overlay();
    }

    fn window_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, _window_id: winit::window::WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Demo completed successfully! 🎉");
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                self.size = physical_size;
                if let (Some(surface), Some(config), Some(device)) =
                    (&self.surface, &mut self.config, &self.device) {
                    config.width = physical_size.width;
                    config.height = physical_size.height;
                    surface.configure(device, config);
                }
            }
            WindowEvent::RedrawRequested => {
                self.update(0.016); // ~60 FPS
                match self.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => {
                        if let (Some(surface), Some(config), Some(device)) =
                            (&self.surface, &self.config, &self.device) {
                            surface.configure(device, config);
                        }
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        event_loop.exit();
                    }
                    Err(e) => {
                        eprintln!("{:?}", e);
                    }
                }

                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::KeyboardInput { event: KeyEvent { logical_key, state, .. }, .. } => {
                let key_str = match logical_key {
                    Key::Named(NamedKey::Tab) => "Tab".to_string(),
                    Key::Named(NamedKey::Escape) => "Escape".to_string(),
                    Key::Character(s) => s.to_string(),
                    _ => return,
                };

                match state {
                    ElementState::Pressed => {
                        match key_str.as_str() {
                            "Tab" => {
                                self.show_tool_palette = !self.show_tool_palette;
                                self.print_ui_overlay();
                            }
                            "t" | "T" => {
                                self.show_asset_browser = !self.show_asset_browser;
                                self.print_ui_overlay();
                            }
                            "p" | "P" => {
                                self.show_properties = !self.show_properties;
                                self.print_ui_overlay();
                            }
                            "q" | "Q" => {
                                event_loop.exit();
                            }
                            "Escape" => {
                                self.cursor_grabbed = false;
                                if let Some(window) = &self.window {
                                    let _ = window.set_cursor_grab(winit::window::CursorGrabMode::None);
                                    window.set_cursor_visible(true);
                                }
                            }
                            _ => {
                                self.keys.insert(key_str);
                            }
                        }
                    }
                    ElementState::Released => {
                        self.keys.remove(&key_str);
                    }
                }
            }
            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {
                self.cursor_grabbed = true;
                if let Some(window) = &self.window {
                    let _ = window.set_cursor_grab(winit::window::CursorGrabMode::Confined);
                    window.set_cursor_visible(false);
                }
            }
            _ => {}
        }
    }

    fn device_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, _device_id: winit::event::DeviceId, event: DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = event {
            if self.cursor_grabbed {
                self.mouse_delta.0 += delta.0;
                self.mouse_delta.1 += delta.1;
            }
        }
    }
}

fn main() {
    env_logger::init();

    println!("🚀 Starting Robin Engine Phase 4 UI Polish Demo...");
    println!("This demonstrates the modern UI system with real 3D graphics!");

    let event_loop = EventLoop::new().unwrap();
    let mut app = EngineUIDemo::new();
    event_loop.run_app(&mut app).unwrap();
}