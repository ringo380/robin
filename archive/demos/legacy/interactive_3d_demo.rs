use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};
use wgpu::util::DeviceExt;
use std::time::Instant;
use std::sync::Arc;
use cgmath::*;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
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

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        Self {
            view_proj: Matrix4::identity().into(),
        }
    }

    fn update_view_proj(&mut self, view_proj: Matrix4<f32>) {
        self.view_proj = view_proj.into();
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
            position: Point3::new(0.0, 5.0, 10.0),
            yaw: Rad(0.0),
            pitch: Rad(0.0),
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

struct CameraController {
    move_speed: f32,
    mouse_sensitivity: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_jump_pressed: bool,
    is_crouch_pressed: bool,
    velocity_y: f32,
    is_on_ground: bool,
}

impl CameraController {
    fn new(move_speed: f32, mouse_sensitivity: f32) -> Self {
        Self {
            move_speed,
            mouse_sensitivity,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_jump_pressed: false,
            is_crouch_pressed: false,
            velocity_y: 0.0,
            is_on_ground: true,
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
                self.is_jump_pressed = is_pressed;
                true
            }
            PhysicalKey::Code(KeyCode::ControlLeft) => {
                self.is_crouch_pressed = is_pressed;
                true
            }
            _ => false,
        }
    }

    fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        // Intentionally left empty for now - will add mouse look if needed
    }

    fn update_camera(&mut self, camera: &mut Camera, dt: f32) {
        let forward = Vector3::new(camera.yaw.0.cos(), 0.0, camera.yaw.0.sin()).normalize();
        let right = Vector3::new(camera.yaw.0.sin(), 0.0, -camera.yaw.0.cos()).normalize();

        // Horizontal movement
        if self.is_forward_pressed {
            camera.position += forward * self.move_speed * dt;
        }
        if self.is_backward_pressed {
            camera.position -= forward * self.move_speed * dt;
        }
        if self.is_right_pressed {
            camera.position += right * self.move_speed * dt;
        }
        if self.is_left_pressed {
            camera.position -= right * self.move_speed * dt;
        }

        // Jump mechanics
        if self.is_jump_pressed && self.is_on_ground {
            self.velocity_y = 15.0;
            self.is_on_ground = false;
        }

        // Apply gravity
        if !self.is_on_ground {
            self.velocity_y -= 30.0 * dt; // gravity
            camera.position.y += self.velocity_y * dt;

            // Simple ground collision
            let ground_height = self.get_ground_height(camera.position.x, camera.position.z);
            if camera.position.y <= ground_height + 2.0 {
                camera.position.y = ground_height + 2.0;
                self.velocity_y = 0.0;
                self.is_on_ground = true;
            }
        }

        // Crouch
        if self.is_crouch_pressed {
            camera.position.y = (camera.position.y - 2.0 * dt).max(self.get_ground_height(camera.position.x, camera.position.z) + 1.0);
        }

        // Rotate camera with arrow keys
        if self.is_left_pressed && !self.is_right_pressed {
            camera.yaw -= Rad(1.0 * dt);
        }
        if self.is_right_pressed && !self.is_left_pressed {
            camera.yaw += Rad(1.0 * dt);
        }
    }

    fn get_ground_height(&self, x: f32, z: f32) -> f32 {
        // Use the same terrain height function as the mesh generation
        terrain_height(x, z)
    }
}

// Calculate terrain height at any point (shared function)
fn terrain_height(x: f32, z: f32) -> f32 {
    // Base sea level
    let sea_level = 0.0;
    
    // Primary terrain waves - larger features
    let primary_hills = 8.0 * ((x * 0.05).sin() * (z * 0.05).cos());
    
    // Secondary terrain features - medium details
    let secondary_features = 3.0 * ((x * 0.1).cos() + (z * 0.1).sin());
    
    // Fine detail noise
    let fine_detail = 1.0 * ((x * 0.3).sin() * (z * 0.25).cos());
    
    // Combine all terrain features with minimum sea level
    (sea_level + primary_hills + secondary_features + fine_detail).max(sea_level)
}

// Generate continuous terrain with proper ground plane
fn generate_landscape() -> Vec<Vertex> {
    let mut vertices = Vec::new();
    
    // Create a continuous terrain mesh
    let terrain_size = 100; // 200x200 unit terrain
    let terrain_resolution = 1.0; // 1 unit between vertices
    
    for x in -terrain_size..(terrain_size - 1) {
        for z in -terrain_size..(terrain_size - 1) {
            let x_f = x as f32 * terrain_resolution;
            let z_f = z as f32 * terrain_resolution;
            
            // Calculate heights for each corner of the quad
            let h00 = terrain_height(x_f, z_f);
            let h10 = terrain_height(x_f + terrain_resolution, z_f);
            let h01 = terrain_height(x_f, z_f + terrain_resolution);
            let h11 = terrain_height(x_f + terrain_resolution, z_f + terrain_resolution);
            
            // Color based on height and terrain type
            let get_terrain_color = |height: f32| -> [f32; 3] {
                if height > 8.0 {
                    [0.7, 0.7, 0.8] // Rocky peaks - gray
                } else if height > 4.0 {
                    [0.4, 0.6, 0.2] // Hills - dark green
                } else if height > 1.0 {
                    [0.2, 0.8, 0.2] // Grassland - bright green
                } else if height > 0.1 {
                    [0.8, 0.7, 0.3] // Beach/sand - tan
                } else {
                    [0.1, 0.4, 0.8] // Water - blue
                }
            };
            
            let color00 = get_terrain_color(h00);
            let color10 = get_terrain_color(h10);
            let color01 = get_terrain_color(h01);
            let color11 = get_terrain_color(h11);
            
            // First triangle (bottom-left)
            vertices.extend_from_slice(&[
                Vertex { position: [x_f, h00, z_f], color: color00 },
                Vertex { position: [x_f + terrain_resolution, h10, z_f], color: color10 },
                Vertex { position: [x_f, h01, z_f + terrain_resolution], color: color01 },
            ]);
            
            // Second triangle (top-right)
            vertices.extend_from_slice(&[
                Vertex { position: [x_f + terrain_resolution, h10, z_f], color: color10 },
                Vertex { position: [x_f + terrain_resolution, h11, z_f + terrain_resolution], color: color11 },
                Vertex { position: [x_f, h01, z_f + terrain_resolution], color: color01 },
            ]);
        }
    }
    
    // Add some buildings (indoor areas)
    // Building 1
    for i in 0..6 {
        for j in 0..6 {
            let x = 10.0 + i as f32;
            let z = 10.0 + j as f32;
            let height = 8.0;
            
            // Walls
            vertices.extend_from_slice(&[
                Vertex { position: [x, 0.0, z], color: [0.7, 0.7, 0.7] },
                Vertex { position: [x, height, z], color: [0.7, 0.7, 0.7] },
                Vertex { position: [x + 1.0, 0.0, z], color: [0.7, 0.7, 0.7] },
                
                Vertex { position: [x, height, z], color: [0.7, 0.7, 0.7] },
                Vertex { position: [x + 1.0, height, z], color: [0.7, 0.7, 0.7] },
                Vertex { position: [x + 1.0, 0.0, z], color: [0.7, 0.7, 0.7] },
            ]);
        }
    }
    
    // Trees for outdoor variety
    for tree_x in [-30, -10, 25, 35] {
        for tree_z in [-25, -5, 15, 30] {
            let x = tree_x as f32;
            let z = tree_z as f32;
            let trunk_height = 8.0;
            
            // Tree trunk
            vertices.extend_from_slice(&[
                Vertex { position: [x, 0.0, z], color: [0.5, 0.3, 0.1] },
                Vertex { position: [x, trunk_height, z], color: [0.5, 0.3, 0.1] },
                Vertex { position: [x + 1.0, 0.0, z], color: [0.5, 0.3, 0.1] },
                
                Vertex { position: [x, trunk_height, z], color: [0.5, 0.3, 0.1] },
                Vertex { position: [x + 1.0, trunk_height, z], color: [0.5, 0.3, 0.1] },
                Vertex { position: [x + 1.0, 0.0, z], color: [0.5, 0.3, 0.1] },
            ]);
            
            // Tree canopy
            vertices.extend_from_slice(&[
                Vertex { position: [x - 2.0, trunk_height, z - 2.0], color: [0.0, 0.8, 0.0] },
                Vertex { position: [x + 3.0, trunk_height + 5.0, z], color: [0.0, 0.8, 0.0] },
                Vertex { position: [x - 2.0, trunk_height, z + 3.0], color: [0.0, 0.8, 0.0] },
            ]);
        }
    }
    
    vertices
}

const SHADER_SOURCE: &str = r#"
struct CameraUniform {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    return out;
}

@fragment  
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
"#;

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
        
        // Create camera with logical starting position
        let mut camera = Camera::new(config.width as f32 / config.height as f32);
        
        // Set camera to start at a safe, logical position above the terrain
        let start_x = 0.0;
        let start_z = 0.0;
        let ground_height = terrain_height(start_x, start_z);
        camera.position = Point3::new(start_x, ground_height + 3.0, start_z); // 3 units above ground
        
        let camera_controller = CameraController::new(20.0, 0.4);
        
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(camera.build_view_projection_matrix());
        
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        
        println!("🎨 Creating shader...");
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SOURCE.into()),
        });
        
        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        
        println!("🏞️  Generating landscape...");
        let vertices = generate_landscape();
        
        println!("📊 Creating vertex buffer...");
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        println!("✅ Graphics initialization complete!");
        
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
    
    fn update(&mut self) {
        let now = Instant::now();
        let dt = (now - self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;
        
        self.camera_controller.update_camera(&mut self.camera, dt);
        self.camera_uniform.update_view_proj(self.camera.build_view_projection_matrix());
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
            // Calculate exact vertex count from terrain generation
            let terrain_size = 100;
            let vertex_count = (terrain_size * 2 - 2) * (terrain_size * 2 - 2) * 6; // 6 vertices per quad
            render_pass.draw(0..vertex_count as u32, 0..1);
        }
        
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        
        Ok(())
    }
}

async fn run() {
    env_logger::init();
    
    println!("🎮 Starting Interactive 3D Demo");
    println!("This is a REAL interactive first-person 3D environment!");
    
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(WindowBuilder::new()
        .with_title("🚀 Robin Engine - Interactive 3D Demo")
        .with_inner_size(winit::dpi::LogicalSize::new(1200, 800))
        .build(&event_loop)
        .unwrap());
        
    println!("🖼️  Window created! Initializing 3D world...");
    
    let mut state = State::new(window.clone()).await;
    
    println!("🎯 Interactive 3D world ready!");
    println!("📝 Controls:");
    println!("   WASD - Move around");
    println!("   SPACE - Jump");
    println!("   LEFT CTRL - Crouch");
    println!("   ESC - Exit");
    println!("   Navigate the landscape with hills, trees, buildings!");
    
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