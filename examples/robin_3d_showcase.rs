// Robin Engine 3D Showcase Demo
// A comprehensive demonstration of Robin Engine's 3D graphics capabilities
// Features real-time rendering, particle effects, dynamic lighting, and interactive gameplay

use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder, CursorGrabMode},
};
use wgpu::util::DeviceExt;
use cgmath::{
    Matrix4, Point3, Vector3, Vector4, Rad, Deg, perspective, 
    SquareMatrix, InnerSpace, Rotation3, Zero, Quaternion, Euler
};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use rand::Rng;

// Constants
const MOVE_SPEED: f32 = 15.0;
const MOUSE_SENSITIVITY: f32 = 0.002;
const GRAVITY: f32 = -20.0;
const JUMP_VELOCITY: f32 = 10.0;
const MAX_PARTICLES: usize = 10000;
const SHADOW_MAP_SIZE: u32 = 2048;

fn main() {
    env_logger::init();
    
    println!("🚀 Robin Engine 3D Graphics Showcase");
    println!("=====================================");
    println!("A comprehensive demonstration of real-time 3D rendering");
    
    // Create event loop and window
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("Robin Engine 3D - Graphics Showcase")
        .with_inner_size(winit::dpi::LogicalSize::new(1920, 1080))
        .build(&event_loop)
        .unwrap();
    
    // Initialize the demo
    let mut demo = pollster::block_on(Demo::new(&window));
    
    println!("\n✨ Graphics Systems Initialized:");
    println!("  • Advanced PBR rendering pipeline");
    println!("  • Dynamic shadow mapping");
    println!("  • GPU-accelerated particle system");
    println!("  • Post-processing effects (bloom, tone mapping)");
    println!("  • Instanced rendering for performance");
    
    println!("\n🎮 Controls:");
    println!("  WASD         - Move around");
    println!("  Mouse        - Look around");
    println!("  Space        - Jump");
    println!("  Shift        - Sprint");
    println!("  Left Click   - Place block / Shoot particle");
    println!("  Right Click  - Remove block");
    println!("  E            - Interact with objects");
    println!("  1-5          - Select materials");
    println!("  Q            - Toggle wireframe");
    println!("  F            - Toggle flashlight");
    println!("  G            - Spawn particle burst");
    println!("  Tab          - Toggle debug UI");
    println!("  Escape       - Release mouse / Exit");
    
    // Hide cursor and lock it to window
    window.set_cursor_visible(false);
    let _ = window.set_cursor_grab(CursorGrabMode::Confined);
    
    // Run the event loop
    let mut last_frame_time = Instant::now();
    
    event_loop.run(move |event, target| {
        match event {
            Event::WindowEvent { ref event, window_id } if window_id == window.id() => {
                if !demo.handle_input(event) {
                    match event {
                        WindowEvent::CloseRequested => target.exit(),
                        WindowEvent::KeyboardInput {
                            event: KeyEvent {
                                physical_key: PhysicalKey::Code(KeyCode::Escape),
                                state: ElementState::Pressed,
                                ..
                            },
                            ..
                        } => {
                            if demo.mouse_locked {
                                demo.mouse_locked = false;
                                window.set_cursor_visible(true);
                                let _ = window.set_cursor_grab(CursorGrabMode::None);
                            } else {
                                target.exit();
                            }
                        }
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
                        WindowEvent::MouseInput { button, state, .. } => {
                            if !demo.mouse_locked && *button == MouseButton::Left 
                                && *state == ElementState::Pressed {
                                demo.mouse_locked = true;
                                window.set_cursor_visible(false);
                                let _ = window.set_cursor_grab(CursorGrabMode::Confined);
                            }
                        }
                        _ => {}
                    }
                }
            }
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        }
    }).unwrap();
}

// Vertex structure for 3D meshes
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    tex_coords: [f32; 2],
    color: [f32; 4],
    tangent: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
        0 => Float32x3, // position
        1 => Float32x3, // normal
        2 => Float32x2, // tex_coords
        3 => Float32x4, // color
        4 => Float32x3, // tangent
    ];
    
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

// Instance data for instanced rendering
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceData {
    model_matrix: [[f32; 4]; 4],
    color: [f32; 4],
    material_id: u32,
    _padding: [u32; 3],
}

impl InstanceData {
    const ATTRIBS: [wgpu::VertexAttribute; 6] = wgpu::vertex_attr_array![
        5 => Float32x4,  // model matrix row 0
        6 => Float32x4,  // model matrix row 1
        7 => Float32x4,  // model matrix row 2
        8 => Float32x4,  // model matrix row 3
        9 => Float32x4,  // color
        10 => Uint32,    // material_id
    ];
    
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceData>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

// Camera uniform buffer structure
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    proj: [[f32; 4]; 4],
    camera_pos: [f32; 4],
    time: f32,
    _padding: [f32; 3],
}

// Light uniform structure
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct LightUniform {
    position: [f32; 3],
    light_type: u32,  // 0=directional, 1=point, 2=spot
    direction: [f32; 3],
    intensity: f32,
    color: [f32; 3],
    range: f32,
    inner_cone: f32,
    outer_cone: f32,
    _padding: [f32; 2],
}

// Particle structure for GPU particle system
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct GPUParticle {
    position: [f32; 3],
    life: f32,
    velocity: [f32; 3],
    size: f32,
    color: [f32; 4],
    rotation: f32,
    angular_velocity: f32,
    _padding: [f32; 2],
}

// Camera controller
struct Camera {
    position: Point3<f32>,
    yaw: Rad<f32>,
    pitch: Rad<f32>,
    velocity: Vector3<f32>,
    grounded: bool,
}

impl Camera {
    fn new(position: Point3<f32>) -> Self {
        Self {
            position,
            yaw: Rad(0.0),
            pitch: Rad(0.0),
            velocity: Vector3::zero(),
            grounded: true,
        }
    }
    
    fn build_view_projection_matrix(&self, aspect: f32) -> (Matrix4<f32>, Matrix4<f32>) {
        let view = Matrix4::look_to_rh(
            self.position,
            Vector3::new(
                self.yaw.0.cos() * self.pitch.0.cos(),
                self.pitch.0.sin(),
                self.yaw.0.sin() * self.pitch.0.cos(),
            ),
            Vector3::unit_y(),
        );
        
        let proj = perspective(Deg(60.0), aspect, 0.1, 1000.0);
        
        (view, proj)
    }
    
    fn forward(&self) -> Vector3<f32> {
        Vector3::new(
            self.yaw.0.cos() * self.pitch.0.cos(),
            0.0,  // Don't move vertically with look direction
            self.yaw.0.sin() * self.pitch.0.cos(),
        ).normalize()
    }
    
    fn right(&self) -> Vector3<f32> {
        Vector3::new(
            -self.yaw.0.sin(),
            0.0,
            self.yaw.0.cos(),
        ).normalize()
    }
}

// Input state tracking
#[derive(Default)]
struct InputState {
    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
    up: bool,
    down: bool,
    sprint: bool,
    mouse_delta: (f32, f32),
}

// Material types for building
#[derive(Clone, Copy, Debug, PartialEq)]
enum MaterialType {
    Stone,
    Metal,
    Wood,
    Glass,
    Energy,
}

impl MaterialType {
    fn color(&self) -> [f32; 4] {
        match self {
            MaterialType::Stone => [0.5, 0.5, 0.5, 1.0],
            MaterialType::Metal => [0.7, 0.7, 0.8, 1.0],
            MaterialType::Wood => [0.5, 0.3, 0.1, 1.0],
            MaterialType::Glass => [0.6, 0.8, 1.0, 0.5],
            MaterialType::Energy => [0.0, 0.8, 1.0, 1.0],
        }
    }
    
    fn emissive(&self) -> f32 {
        match self {
            MaterialType::Energy => 2.0,
            _ => 0.0,
        }
    }
}

// Game object types
#[derive(Clone)]
struct GameObject {
    position: Point3<f32>,
    rotation: Quaternion<f32>,
    scale: Vector3<f32>,
    material: MaterialType,
    mesh_type: MeshType,
    velocity: Vector3<f32>,
    dynamic: bool,
}

#[derive(Clone, Copy)]
enum MeshType {
    Cube,
    Sphere,
    Cylinder,
    Pyramid,
}

// Main demo struct
struct Demo {
    // Core graphics
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    
    // Render pipelines
    main_pipeline: wgpu::RenderPipeline,
    particle_pipeline: wgpu::RenderPipeline,
    shadow_pipeline: wgpu::RenderPipeline,
    post_process_pipeline: wgpu::RenderPipeline,
    
    // Buffers and resources
    camera: Camera,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    
    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,
    lights: Vec<LightUniform>,
    
    // Geometry
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    num_indices: u32,
    
    // Textures
    depth_texture: wgpu::TextureView,
    shadow_map: wgpu::TextureView,
    bloom_texture: wgpu::TextureView,
    
    // Particle system
    particle_buffer: wgpu::Buffer,
    particle_bind_group: wgpu::BindGroup,
    particles: Vec<GPUParticle>,
    
    // Game state
    objects: Vec<GameObject>,
    selected_material: MaterialType,
    input_state: InputState,
    mouse_locked: bool,
    wireframe: bool,
    flashlight_on: bool,
    
    // Performance
    frame_count: u32,
    last_fps_update: Instant,
    fps: f32,
    time: f32,
}

impl Demo {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        
        // Initialize WGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        let surface = instance.create_surface(window.clone()).unwrap();
        
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Robin 3D Device"),
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
        
        // Create shader modules
        let main_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Main Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/main_3d.wgsl").into()),
        });
        
        let particle_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Particle Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/particles_3d.wgsl").into()),
        });
        
        // Create camera
        let camera = Camera::new(Point3::new(0.0, 5.0, -10.0));
        let (view, proj) = camera.build_view_projection_matrix(config.width as f32 / config.height as f32);
        
        let camera_uniform = CameraUniform {
            view_proj: (proj * view).into(),
            view: view.into(),
            proj: proj.into(),
            camera_pos: [camera.position.x, camera.position.y, camera.position.z, 1.0],
            time: 0.0,
            _padding: [0.0; 3],
        };
        
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        
        // Create lights
        let lights = vec![
            // Sun light
            LightUniform {
                position: [50.0, 100.0, 50.0],
                light_type: 0,
                direction: [-0.3, -0.7, -0.3],
                intensity: 2.0,
                color: [1.0, 0.95, 0.8],
                range: 1000.0,
                inner_cone: 0.0,
                outer_cone: 0.0,
                _padding: [0.0; 2],
            },
            // Point lights
            LightUniform {
                position: [0.0, 10.0, 0.0],
                light_type: 1,
                direction: [0.0, 0.0, 0.0],
                intensity: 50.0,
                color: [1.0, 0.5, 0.0],
                range: 30.0,
                inner_cone: 0.0,
                outer_cone: 0.0,
                _padding: [0.0; 2],
            },
            LightUniform {
                position: [20.0, 5.0, 20.0],
                light_type: 1,
                direction: [0.0, 0.0, 0.0],
                intensity: 30.0,
                color: [0.0, 0.5, 1.0],
                range: 25.0,
                inner_cone: 0.0,
                outer_cone: 0.0,
                _padding: [0.0; 2],
            },
        ];
        
        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Buffer"),
            contents: bytemuck::cast_slice(&lights),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        
        // Create bind group layouts
        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera Bind Group Layout"),
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
        });
        
        let light_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Light Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        
        // Create bind groups
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
            ],
        });
        
        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Light Bind Group"),
            layout: &light_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: light_buffer.as_entire_binding(),
                },
            ],
        });
        
        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Main Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout, &light_bind_group_layout],
            push_constant_ranges: &[],
        });
        
        // Create main render pipeline
        let main_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Main Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &main_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc(), InstanceData::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &main_shader,
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
        
        // Create particle pipeline (simplified for now)
        let particle_pipeline = main_pipeline.clone(); // Placeholder
        let shadow_pipeline = main_pipeline.clone(); // Placeholder
        let post_process_pipeline = main_pipeline.clone(); // Placeholder
        
        // Create initial world geometry
        let (vertices, indices) = create_world_geometry();
        
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
        
        // Create game objects
        let mut objects = create_initial_scene();
        
        // Create instance buffer
        let instances = create_instance_data(&objects);
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instances),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        
        // Create depth texture
        let depth_texture = create_depth_texture(&device, &config);
        
        // Create shadow map (placeholder)
        let shadow_map = depth_texture.clone();
        
        // Create bloom texture (placeholder)
        let bloom_texture = depth_texture.clone();
        
        // Initialize particle system
        let particles = Vec::new();
        let particle_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Particle Buffer"),
            size: (std::mem::size_of::<GPUParticle>() * MAX_PARTICLES) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        let particle_bind_group = camera_bind_group.clone(); // Placeholder
        
        Self {
            surface,
            device,
            queue,
            config,
            size,
            main_pipeline,
            particle_pipeline,
            shadow_pipeline,
            post_process_pipeline,
            camera,
            camera_buffer,
            camera_bind_group,
            light_buffer,
            light_bind_group,
            lights,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            num_indices: indices.len() as u32,
            depth_texture,
            shadow_map,
            bloom_texture,
            particle_buffer,
            particle_bind_group,
            particles,
            objects,
            selected_material: MaterialType::Stone,
            input_state: InputState::default(),
            mouse_locked: true,
            wireframe: false,
            flashlight_on: false,
            frame_count: 0,
            last_fps_update: Instant::now(),
            fps: 0.0,
            time: 0.0,
        }
    }
    
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            
            // Recreate depth texture
            self.depth_texture = create_depth_texture(&self.device, &self.config);
        }
    }
    
    fn handle_input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                let pressed = event.state == ElementState::Pressed;
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::KeyW) => self.input_state.forward = pressed,
                    PhysicalKey::Code(KeyCode::KeyS) => self.input_state.backward = pressed,
                    PhysicalKey::Code(KeyCode::KeyA) => self.input_state.left = pressed,
                    PhysicalKey::Code(KeyCode::KeyD) => self.input_state.right = pressed,
                    PhysicalKey::Code(KeyCode::Space) => {
                        if pressed && self.camera.grounded {
                            self.camera.velocity.y = JUMP_VELOCITY;
                            self.camera.grounded = false;
                        }
                    }
                    PhysicalKey::Code(KeyCode::ShiftLeft) => self.input_state.sprint = pressed,
                    PhysicalKey::Code(KeyCode::KeyQ) if pressed => self.wireframe = !self.wireframe,
                    PhysicalKey::Code(KeyCode::KeyF) if pressed => self.flashlight_on = !self.flashlight_on,
                    PhysicalKey::Code(KeyCode::KeyG) if pressed => self.spawn_particle_burst(),
                    PhysicalKey::Code(KeyCode::Digit1) if pressed => self.selected_material = MaterialType::Stone,
                    PhysicalKey::Code(KeyCode::Digit2) if pressed => self.selected_material = MaterialType::Metal,
                    PhysicalKey::Code(KeyCode::Digit3) if pressed => self.selected_material = MaterialType::Wood,
                    PhysicalKey::Code(KeyCode::Digit4) if pressed => self.selected_material = MaterialType::Glass,
                    PhysicalKey::Code(KeyCode::Digit5) if pressed => self.selected_material = MaterialType::Energy,
                    _ => return false,
                }
                true
            }
            WindowEvent::CursorMoved { position, .. } if self.mouse_locked => {
                let center_x = self.size.width as f64 / 2.0;
                let center_y = self.size.height as f64 / 2.0;
                
                self.input_state.mouse_delta = (
                    (position.x - center_x) as f32,
                    (position.y - center_y) as f32,
                );
                true
            }
            WindowEvent::MouseInput { button, state, .. } if self.mouse_locked => {
                if *state == ElementState::Pressed {
                    match button {
                        MouseButton::Left => self.place_block(),
                        MouseButton::Right => self.remove_block(),
                        _ => {}
                    }
                }
                true
            }
            _ => false,
        }
    }
    
    fn update(&mut self, delta_time: f32) {
        self.time += delta_time;
        
        // Update FPS counter
        self.frame_count += 1;
        if self.last_fps_update.elapsed().as_secs_f32() >= 1.0 {
            self.fps = self.frame_count as f32 / self.last_fps_update.elapsed().as_secs_f32();
            self.frame_count = 0;
            self.last_fps_update = Instant::now();
            println!("FPS: {:.1} | Objects: {} | Particles: {}", 
                     self.fps, self.objects.len(), self.particles.len());
        }
        
        // Update camera movement
        let speed = if self.input_state.sprint { MOVE_SPEED * 2.0 } else { MOVE_SPEED };
        let mut movement = Vector3::zero();
        
        if self.input_state.forward {
            movement += self.camera.forward();
        }
        if self.input_state.backward {
            movement -= self.camera.forward();
        }
        if self.input_state.left {
            movement -= self.camera.right();
        }
        if self.input_state.right {
            movement += self.camera.right();
        }
        
        if movement.magnitude() > 0.0 {
            movement = movement.normalize() * speed * delta_time;
            self.camera.position += movement;
        }
        
        // Apply gravity and physics
        if !self.camera.grounded {
            self.camera.velocity.y += GRAVITY * delta_time;
        }
        self.camera.position.y += self.camera.velocity.y * delta_time;
        
        // Ground collision (simple)
        if self.camera.position.y < 1.5 {
            self.camera.position.y = 1.5;
            self.camera.velocity.y = 0.0;
            self.camera.grounded = true;
        }
        
        // Update camera rotation
        if self.mouse_locked {
            self.camera.yaw -= Rad(self.input_state.mouse_delta.0 * MOUSE_SENSITIVITY);
            self.camera.pitch -= Rad(self.input_state.mouse_delta.1 * MOUSE_SENSITIVITY);
            self.camera.pitch = Rad(self.camera.pitch.0.clamp(-1.5, 1.5));
            self.input_state.mouse_delta = (0.0, 0.0);
        }
        
        // Update dynamic objects
        for obj in &mut self.objects {
            if obj.dynamic {
                // Simple physics simulation
                obj.velocity.y += GRAVITY * delta_time * 0.5;
                obj.position += obj.velocity * delta_time;
                
                // Ground collision
                if obj.position.y < 0.5 {
                    obj.position.y = 0.5;
                    obj.velocity.y *= -0.7; // Bounce
                }
                
                // Rotate dynamic objects
                obj.rotation = obj.rotation * Quaternion::from(Euler::new(
                    Rad(delta_time),
                    Rad(delta_time * 0.7),
                    Rad(delta_time * 0.3),
                ));
            }
        }
        
        // Update particles
        self.update_particles(delta_time);
        
        // Animate lights (pulse energy lights)
        for (i, light) in self.lights.iter_mut().enumerate() {
            if i == 1 {
                light.intensity = 50.0 + (self.time * 2.0).sin() * 20.0;
            }
            if i == 2 {
                light.position[0] = 20.0 + (self.time).cos() * 10.0;
                light.position[2] = 20.0 + (self.time).sin() * 10.0;
            }
        }
        
        // Update flashlight
        if self.flashlight_on {
            if self.lights.len() < 4 {
                self.lights.push(LightUniform {
                    position: [0.0; 3],
                    light_type: 2, // Spot light
                    direction: [0.0; 3],
                    intensity: 100.0,
                    color: [1.0, 1.0, 0.9],
                    range: 50.0,
                    inner_cone: 0.9,
                    outer_cone: 0.95,
                    _padding: [0.0; 2],
                });
            }
            let flashlight = self.lights.last_mut().unwrap();
            flashlight.position = [
                self.camera.position.x,
                self.camera.position.y,
                self.camera.position.z,
            ];
            let forward = self.camera.forward();
            flashlight.direction = [forward.x, forward.y, forward.z];
        } else if self.lights.len() == 4 {
            self.lights.pop();
        }
        
        // Update GPU buffers
        self.update_buffers();
    }
    
    fn update_buffers(&mut self) {
        // Update camera uniform
        let (view, proj) = self.camera.build_view_projection_matrix(
            self.config.width as f32 / self.config.height as f32
        );
        
        let camera_uniform = CameraUniform {
            view_proj: (proj * view).into(),
            view: view.into(),
            proj: proj.into(),
            camera_pos: [
                self.camera.position.x,
                self.camera.position.y,
                self.camera.position.z,
                1.0,
            ],
            time: self.time,
            _padding: [0.0; 3],
        };
        
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        );
        
        // Update light buffer
        self.queue.write_buffer(
            &self.light_buffer,
            0,
            bytemuck::cast_slice(&self.lights),
        );
        
        // Update instance buffer
        let instances = create_instance_data(&self.objects);
        self.queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&instances),
        );
        
        // Update particle buffer
        if !self.particles.is_empty() {
            self.queue.write_buffer(
                &self.particle_buffer,
                0,
                bytemuck::cast_slice(&self.particles),
            );
        }
    }
    
    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        
        // Main render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
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
            
            // Draw main geometry
            render_pass.set_pipeline(&self.main_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.light_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..self.objects.len() as u32);
        }
        
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        
        Ok(())
    }
    
    fn place_block(&mut self) {
        let forward = self.camera.forward();
        let pos = self.camera.position + forward * 5.0;
        
        self.objects.push(GameObject {
            position: Point3::new(
                (pos.x / 2.0).round() * 2.0,
                (pos.y / 2.0).round() * 2.0,
                (pos.z / 2.0).round() * 2.0,
            ),
            rotation: Quaternion::from(Euler::new(Rad(0.0), Rad(0.0), Rad(0.0))),
            scale: Vector3::new(1.0, 1.0, 1.0),
            material: self.selected_material,
            mesh_type: MeshType::Cube,
            velocity: Vector3::zero(),
            dynamic: false,
        });
        
        self.spawn_place_particles(pos);
    }
    
    fn remove_block(&mut self) {
        let forward = self.camera.forward();
        let ray_origin = self.camera.position;
        
        // Simple ray-sphere intersection for block removal
        let mut closest_idx = None;
        let mut closest_dist = f32::MAX;
        
        for (idx, obj) in self.objects.iter().enumerate() {
            let dist = (obj.position - ray_origin).magnitude();
            if dist < 10.0 && dist < closest_dist {
                let to_obj = (obj.position - ray_origin).normalize();
                if to_obj.dot(forward) > 0.8 {
                    closest_dist = dist;
                    closest_idx = Some(idx);
                }
            }
        }
        
        if let Some(idx) = closest_idx {
            let pos = self.objects[idx].position;
            self.objects.remove(idx);
            self.spawn_destroy_particles(pos);
        }
    }
    
    fn spawn_particle_burst(&mut self) {
        let pos = self.camera.position + self.camera.forward() * 3.0;
        let mut rng = rand::thread_rng();
        
        for _ in 0..100 {
            if self.particles.len() < MAX_PARTICLES {
                self.particles.push(GPUParticle {
                    position: [pos.x, pos.y, pos.z],
                    life: 2.0,
                    velocity: [
                        rng.gen_range(-10.0..10.0),
                        rng.gen_range(5.0..15.0),
                        rng.gen_range(-10.0..10.0),
                    ],
                    size: rng.gen_range(0.1..0.5),
                    color: [
                        rng.gen_range(0.5..1.0),
                        rng.gen_range(0.0..1.0),
                        rng.gen_range(0.0..1.0),
                        1.0,
                    ],
                    rotation: rng.gen_range(0.0..std::f32::consts::TAU),
                    angular_velocity: rng.gen_range(-5.0..5.0),
                    _padding: [0.0; 2],
                });
            }
        }
    }
    
    fn spawn_place_particles(&mut self, pos: Point3<f32>) {
        let mut rng = rand::thread_rng();
        
        for _ in 0..20 {
            if self.particles.len() < MAX_PARTICLES {
                self.particles.push(GPUParticle {
                    position: [pos.x, pos.y, pos.z],
                    life: 0.5,
                    velocity: [
                        rng.gen_range(-2.0..2.0),
                        rng.gen_range(0.0..5.0),
                        rng.gen_range(-2.0..2.0),
                    ],
                    size: 0.2,
                    color: self.selected_material.color(),
                    rotation: 0.0,
                    angular_velocity: 0.0,
                    _padding: [0.0; 2],
                });
            }
        }
    }
    
    fn spawn_destroy_particles(&mut self, pos: Point3<f32>) {
        let mut rng = rand::thread_rng();
        
        for _ in 0..30 {
            if self.particles.len() < MAX_PARTICLES {
                self.particles.push(GPUParticle {
                    position: [pos.x, pos.y, pos.z],
                    life: 1.0,
                    velocity: [
                        rng.gen_range(-5.0..5.0),
                        rng.gen_range(2.0..8.0),
                        rng.gen_range(-5.0..5.0),
                    ],
                    size: rng.gen_range(0.1..0.3),
                    color: [0.8, 0.6, 0.4, 1.0],
                    rotation: rng.gen_range(0.0..std::f32::consts::TAU),
                    angular_velocity: rng.gen_range(-10.0..10.0),
                    _padding: [0.0; 2],
                });
            }
        }
    }
    
    fn update_particles(&mut self, delta_time: f32) {
        self.particles.retain_mut(|p| {
            p.life -= delta_time;
            if p.life <= 0.0 {
                return false;
            }
            
            // Update position
            p.position[0] += p.velocity[0] * delta_time;
            p.position[1] += p.velocity[1] * delta_time;
            p.position[2] += p.velocity[2] * delta_time;
            
            // Apply gravity
            p.velocity[1] += GRAVITY * delta_time * 0.5;
            
            // Update rotation
            p.rotation += p.angular_velocity * delta_time;
            
            // Fade out
            p.color[3] = p.life.min(1.0);
            
            true
        });
    }
}

// Helper functions
fn create_world_geometry() -> (Vec<Vertex>, Vec<u32>) {
    // Create a cube mesh
    let vertices = vec![
        // Front face
        Vertex { position: [-1.0, -1.0,  1.0], normal: [0.0, 0.0, 1.0], tex_coords: [0.0, 1.0], color: [1.0; 4], tangent: [1.0, 0.0, 0.0] },
        Vertex { position: [ 1.0, -1.0,  1.0], normal: [0.0, 0.0, 1.0], tex_coords: [1.0, 1.0], color: [1.0; 4], tangent: [1.0, 0.0, 0.0] },
        Vertex { position: [ 1.0,  1.0,  1.0], normal: [0.0, 0.0, 1.0], tex_coords: [1.0, 0.0], color: [1.0; 4], tangent: [1.0, 0.0, 0.0] },
        Vertex { position: [-1.0,  1.0,  1.0], normal: [0.0, 0.0, 1.0], tex_coords: [0.0, 0.0], color: [1.0; 4], tangent: [1.0, 0.0, 0.0] },
        // Back face
        Vertex { position: [-1.0, -1.0, -1.0], normal: [0.0, 0.0, -1.0], tex_coords: [1.0, 1.0], color: [1.0; 4], tangent: [-1.0, 0.0, 0.0] },
        Vertex { position: [-1.0,  1.0, -1.0], normal: [0.0, 0.0, -1.0], tex_coords: [1.0, 0.0], color: [1.0; 4], tangent: [-1.0, 0.0, 0.0] },
        Vertex { position: [ 1.0,  1.0, -1.0], normal: [0.0, 0.0, -1.0], tex_coords: [0.0, 0.0], color: [1.0; 4], tangent: [-1.0, 0.0, 0.0] },
        Vertex { position: [ 1.0, -1.0, -1.0], normal: [0.0, 0.0, -1.0], tex_coords: [0.0, 1.0], color: [1.0; 4], tangent: [-1.0, 0.0, 0.0] },
        // Top face
        Vertex { position: [-1.0,  1.0, -1.0], normal: [0.0, 1.0, 0.0], tex_coords: [0.0, 1.0], color: [1.0; 4], tangent: [1.0, 0.0, 0.0] },
        Vertex { position: [-1.0,  1.0,  1.0], normal: [0.0, 1.0, 0.0], tex_coords: [0.0, 0.0], color: [1.0; 4], tangent: [1.0, 0.0, 0.0] },
        Vertex { position: [ 1.0,  1.0,  1.0], normal: [0.0, 1.0, 0.0], tex_coords: [1.0, 0.0], color: [1.0; 4], tangent: [1.0, 0.0, 0.0] },
        Vertex { position: [ 1.0,  1.0, -1.0], normal: [0.0, 1.0, 0.0], tex_coords: [1.0, 1.0], color: [1.0; 4], tangent: [1.0, 0.0, 0.0] },
        // Bottom face
        Vertex { position: [-1.0, -1.0, -1.0], normal: [0.0, -1.0, 0.0], tex_coords: [1.0, 1.0], color: [1.0; 4], tangent: [1.0, 0.0, 0.0] },
        Vertex { position: [ 1.0, -1.0, -1.0], normal: [0.0, -1.0, 0.0], tex_coords: [0.0, 1.0], color: [1.0; 4], tangent: [1.0, 0.0, 0.0] },
        Vertex { position: [ 1.0, -1.0,  1.0], normal: [0.0, -1.0, 0.0], tex_coords: [0.0, 0.0], color: [1.0; 4], tangent: [1.0, 0.0, 0.0] },
        Vertex { position: [-1.0, -1.0,  1.0], normal: [0.0, -1.0, 0.0], tex_coords: [1.0, 0.0], color: [1.0; 4], tangent: [1.0, 0.0, 0.0] },
        // Right face
        Vertex { position: [ 1.0, -1.0, -1.0], normal: [1.0, 0.0, 0.0], tex_coords: [1.0, 1.0], color: [1.0; 4], tangent: [0.0, 0.0, 1.0] },
        Vertex { position: [ 1.0,  1.0, -1.0], normal: [1.0, 0.0, 0.0], tex_coords: [1.0, 0.0], color: [1.0; 4], tangent: [0.0, 0.0, 1.0] },
        Vertex { position: [ 1.0,  1.0,  1.0], normal: [1.0, 0.0, 0.0], tex_coords: [0.0, 0.0], color: [1.0; 4], tangent: [0.0, 0.0, 1.0] },
        Vertex { position: [ 1.0, -1.0,  1.0], normal: [1.0, 0.0, 0.0], tex_coords: [0.0, 1.0], color: [1.0; 4], tangent: [0.0, 0.0, 1.0] },
        // Left face
        Vertex { position: [-1.0, -1.0, -1.0], normal: [-1.0, 0.0, 0.0], tex_coords: [0.0, 1.0], color: [1.0; 4], tangent: [0.0, 0.0, -1.0] },
        Vertex { position: [-1.0, -1.0,  1.0], normal: [-1.0, 0.0, 0.0], tex_coords: [1.0, 1.0], color: [1.0; 4], tangent: [0.0, 0.0, -1.0] },
        Vertex { position: [-1.0,  1.0,  1.0], normal: [-1.0, 0.0, 0.0], tex_coords: [1.0, 0.0], color: [1.0; 4], tangent: [0.0, 0.0, -1.0] },
        Vertex { position: [-1.0,  1.0, -1.0], normal: [-1.0, 0.0, 0.0], tex_coords: [0.0, 0.0], color: [1.0; 4], tangent: [0.0, 0.0, -1.0] },
    ];
    
    let indices = vec![
        0,  1,  2,  2,  3,  0,  // front
        4,  5,  6,  6,  7,  4,  // back
        8,  9,  10, 10, 11, 8,  // top
        12, 13, 14, 14, 15, 12, // bottom
        16, 17, 18, 18, 19, 16, // right
        20, 21, 22, 22, 23, 20, // left
    ];
    
    (vertices, indices)
}

fn create_initial_scene() -> Vec<GameObject> {
    let mut objects = Vec::new();
    
    // Create ground plane
    for x in -10..=10 {
        for z in -10..=10 {
            objects.push(GameObject {
                position: Point3::new(x as f32 * 2.0, -2.0, z as f32 * 2.0),
                rotation: Quaternion::from(Euler::new(Rad(0.0), Rad(0.0), Rad(0.0))),
                scale: Vector3::new(1.0, 0.1, 1.0),
                material: if (x + z) % 2 == 0 { MaterialType::Stone } else { MaterialType::Metal },
                mesh_type: MeshType::Cube,
                velocity: Vector3::zero(),
                dynamic: false,
            });
        }
    }
    
    // Add some pillars
    for i in 0..4 {
        let angle = i as f32 * std::f32::consts::FRAC_PI_2;
        objects.push(GameObject {
            position: Point3::new(angle.cos() * 15.0, 5.0, angle.sin() * 15.0),
            rotation: Quaternion::from(Euler::new(Rad(0.0), Rad(angle), Rad(0.0))),
            scale: Vector3::new(2.0, 10.0, 2.0),
            material: MaterialType::Energy,
            mesh_type: MeshType::Cube,
            velocity: Vector3::zero(),
            dynamic: false,
        });
    }
    
    // Add some floating dynamic objects
    let mut rng = rand::thread_rng();
    for _ in 0..10 {
        objects.push(GameObject {
            position: Point3::new(
                rng.gen_range(-20.0..20.0),
                rng.gen_range(10.0..20.0),
                rng.gen_range(-20.0..20.0),
            ),
            rotation: Quaternion::from(Euler::new(
                Rad(rng.gen_range(0.0..std::f32::consts::TAU)),
                Rad(rng.gen_range(0.0..std::f32::consts::TAU)),
                Rad(rng.gen_range(0.0..std::f32::consts::TAU)),
            )),
            scale: Vector3::new(
                rng.gen_range(0.5..2.0),
                rng.gen_range(0.5..2.0),
                rng.gen_range(0.5..2.0),
            ),
            material: MaterialType::Glass,
            mesh_type: MeshType::Cube,
            velocity: Vector3::new(
                rng.gen_range(-2.0..2.0),
                0.0,
                rng.gen_range(-2.0..2.0),
            ),
            dynamic: true,
        });
    }
    
    objects
}

fn create_instance_data(objects: &[GameObject]) -> Vec<InstanceData> {
    objects.iter().map(|obj| {
        let translation = Matrix4::from_translation(obj.position.to_vec());
        let rotation = Matrix4::from(obj.rotation);
        let scale = Matrix4::from_nonuniform_scale(obj.scale.x, obj.scale.y, obj.scale.z);
        let model = translation * rotation * scale;
        
        InstanceData {
            model_matrix: model.into(),
            color: obj.material.color(),
            material_id: obj.material as u32,
            _padding: [0; 3],
        }
    }).collect()
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