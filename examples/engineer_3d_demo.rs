// Robin Engine 3D Engineer Build Mode Demo
// A working first-person 3D demo using wgpu for graphics

use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};
use wgpu::util::DeviceExt;
use cgmath::{Matrix4, Point3, Vector3, Rad, perspective, SquareMatrix};
use std::time::{Duration, Instant};

fn main() {
    println!("🎮 Robin Engine 3D Engineer Build Mode Demo");
    println!("==========================================");
    
    // Create event loop and window
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("Robin Engine - Engineer Build Mode")
        .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
        .build(&event_loop)
        .unwrap();
    
    // Initialize the demo
    let mut demo = pollster::block_on(Demo::new(&window));
    
    println!("🔧 Graphics initialized successfully!");
    println!("\n📋 Controls:");
    println!("  WASD - Move around");
    println!("  Mouse - Look around");
    println!("  Space - Move up");
    println!("  Shift - Move down");
    println!("  Left Click - Place block");
    println!("  Right Click - Remove block");
    println!("  1-3 - Select material");
    println!("  Escape - Exit");
    
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
                            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => demo.resize(demo.size),
                            Err(wgpu::SurfaceError::OutOfMemory) => target.exit(),
                            Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                        }
                    }
                    WindowEvent::KeyboardInput { event, .. } => {
                        demo.input(event);
                    }
                    WindowEvent::MouseInput { button, state, .. } => {
                        demo.mouse_input(*button, *state);
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        demo.mouse_moved(position.x as f32, position.y as f32);
                    }
                    _ => {}
                }
            }
            Event::AboutToWait => {
                // Redraw the application
                window.request_redraw();
            }
            _ => {}
        }
    }).unwrap();
}

struct Demo {
    // Graphics
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: std::sync::Arc<Window>,
    render_pipeline: wgpu::RenderPipeline,
    
    // Camera
    camera: Camera,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    
    // World
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    
    // Input state
    input_state: InputState,
    
    // Build mode
    blocks: Vec<Block>,
    selected_material: MaterialType,
    
    // Performance
    last_fps_update: Instant,
    frame_count: u32,
    fps: f32,
}

impl Demo {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        
        // Create wgpu instance and surface
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });
        
        let surface = instance.create_surface(window.clone()).unwrap();
        
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();
        
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
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
        
        // Create shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/basic.wgsl").into()),
        });
        
        // Create camera
        let mut camera = Camera::new((0.0, 5.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
        let camera_uniform = CameraUniform::new();
        
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
        
        // Create render pipeline
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
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
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
        
        // Create initial world geometry
        let (vertices, indices) = create_world_geometry();
        
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });
        
        let num_indices = indices.len() as u32;
        
        // Create depth texture
        let depth_texture = create_depth_texture(&device, &config);
        
        Self {
            surface,
            device,
            queue,
            config,
            size,
            window: std::sync::Arc::new(window.clone()),
            render_pipeline,
            camera,
            camera_buffer,
            camera_bind_group,
            vertex_buffer,
            index_buffer,
            num_indices,
            input_state: InputState::new(),
            blocks: create_initial_blocks(),
            selected_material: MaterialType::Concrete,
            last_fps_update: Instant::now(),
            frame_count: 0,
            fps: 0.0,
        }
    }
    
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            
            // Update camera projection
            self.camera.projection = perspective(
                Rad(std::f32::consts::FRAC_PI_4),
                new_size.width as f32 / new_size.height as f32,
                0.1,
                100.0,
            );
        }
    }
    
    fn input(&mut self, event: &KeyEvent) {
        match event.state {
            ElementState::Pressed => {
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::KeyW) => self.input_state.forward = true,
                    PhysicalKey::Code(KeyCode::KeyS) => self.input_state.backward = true,
                    PhysicalKey::Code(KeyCode::KeyA) => self.input_state.left = true,
                    PhysicalKey::Code(KeyCode::KeyD) => self.input_state.right = true,
                    PhysicalKey::Code(KeyCode::Space) => self.input_state.up = true,
                    PhysicalKey::Code(KeyCode::ShiftLeft) => self.input_state.down = true,
                    PhysicalKey::Code(KeyCode::Digit1) => self.selected_material = MaterialType::Concrete,
                    PhysicalKey::Code(KeyCode::Digit2) => self.selected_material = MaterialType::Steel,
                    PhysicalKey::Code(KeyCode::Digit3) => self.selected_material = MaterialType::Wood,
                    _ => {}
                }
            }
            ElementState::Released => {
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::KeyW) => self.input_state.forward = false,
                    PhysicalKey::Code(KeyCode::KeyS) => self.input_state.backward = false,
                    PhysicalKey::Code(KeyCode::KeyA) => self.input_state.left = false,
                    PhysicalKey::Code(KeyCode::KeyD) => self.input_state.right = false,
                    PhysicalKey::Code(KeyCode::Space) => self.input_state.up = false,
                    PhysicalKey::Code(KeyCode::ShiftLeft) => self.input_state.down = false,
                    _ => {}
                }
            }
        }
    }
    
    fn mouse_input(&mut self, button: winit::event::MouseButton, state: ElementState) {
        match button {
            winit::event::MouseButton::Left => {
                self.input_state.left_click = state == ElementState::Pressed;
                if state == ElementState::Pressed {
                    self.place_block();
                }
            }
            winit::event::MouseButton::Right => {
                self.input_state.right_click = state == ElementState::Pressed;
                if state == ElementState::Pressed {
                    self.remove_block();
                }
            }
            _ => {}
        }
    }
    
    fn mouse_moved(&mut self, x: f32, y: f32) {
        if !self.input_state.mouse_captured {
            self.input_state.mouse_captured = true;
            self.input_state.last_mouse_x = x;
            self.input_state.last_mouse_y = y;
            return;
        }
        
        let dx = x - self.input_state.last_mouse_x;
        let dy = y - self.input_state.last_mouse_y;
        
        self.camera.process_mouse(dx, dy);
        
        self.input_state.last_mouse_x = x;
        self.input_state.last_mouse_y = y;
    }
    
    fn place_block(&mut self) {
        // Simple raycast to place block
        let ray_origin = self.camera.position;
        let ray_direction = self.camera.get_forward_vector();
        
        // Find intersection with ground or existing blocks
        let mut best_distance = f32::INFINITY;
        let mut place_position = None;
        
        // Check ground plane (y = 0)
        if ray_direction.y < 0.0 {
            let t = -ray_origin.y / ray_direction.y;
            if t > 0.0 && t < best_distance {
                let hit_point = ray_origin + ray_direction * t;
                place_position = Some(Point3::new(
                    (hit_point.x / 2.0).round() * 2.0,
                    1.0, // Place one unit above ground
                    (hit_point.z / 2.0).round() * 2.0,
                ));
                best_distance = t;
            }
        }
        
        // Check existing blocks
        for block in &self.blocks {
            if let Some(t) = self.ray_box_intersection(ray_origin, ray_direction, block.position, 1.0) {
                if t < best_distance {
                    // Place adjacent to hit block
                    let hit_point = ray_origin + ray_direction * t;
                    let diff = hit_point - block.position;
                    
                    let mut new_pos = block.position;
                    if diff.x.abs() > diff.y.abs() && diff.x.abs() > diff.z.abs() {
                        new_pos.x += if diff.x > 0.0 { 2.0 } else { -2.0 };
                    } else if diff.y.abs() > diff.z.abs() {
                        new_pos.y += if diff.y > 0.0 { 2.0 } else { -2.0 };
                    } else {
                        new_pos.z += if diff.z > 0.0 { 2.0 } else { -2.0 };
                    }
                    
                    place_position = Some(new_pos);
                    best_distance = t;
                }
            }
        }
        
        if let Some(pos) = place_position {
            // Check if position is already occupied
            let occupied = self.blocks.iter().any(|b| {
                (b.position - pos).magnitude() < 1.0
            });
            
            if !occupied {
                self.blocks.push(Block {
                    position: pos,
                    material: self.selected_material,
                });
                self.update_world_geometry();
                println!("🧱 Placed {:?} block at ({:.1}, {:.1}, {:.1})", 
                        self.selected_material, pos.x, pos.y, pos.z);
            }
        }
    }
    
    fn remove_block(&mut self) {
        let ray_origin = self.camera.position;
        let ray_direction = self.camera.get_forward_vector();
        
        let mut closest_block = None;
        let mut closest_distance = f32::INFINITY;
        
        for (i, block) in self.blocks.iter().enumerate() {
            if let Some(t) = self.ray_box_intersection(ray_origin, ray_direction, block.position, 1.0) {
                if t < closest_distance {
                    closest_distance = t;
                    closest_block = Some(i);
                }
            }
        }
        
        if let Some(index) = closest_block {
            let removed_block = self.blocks.remove(index);
            self.update_world_geometry();
            println!("💥 Removed {:?} block", removed_block.material);
        }
    }
    
    fn ray_box_intersection(&self, ray_origin: Point3<f32>, ray_dir: Vector3<f32>, box_center: Point3<f32>, half_size: f32) -> Option<f32> {
        let box_min = box_center - Vector3::new(half_size, half_size, half_size);
        let box_max = box_center + Vector3::new(half_size, half_size, half_size);
        
        let mut t_min = 0.0f32;
        let mut t_max = f32::INFINITY;
        
        for i in 0..3 {
            let origin_component = match i {
                0 => ray_origin.x,
                1 => ray_origin.y,
                2 => ray_origin.z,
                _ => unreachable!(),
            };
            
            let dir_component = match i {
                0 => ray_dir.x,
                1 => ray_dir.y,
                2 => ray_dir.z,
                _ => unreachable!(),
            };
            
            let box_min_component = match i {
                0 => box_min.x,
                1 => box_min.y,
                2 => box_min.z,
                _ => unreachable!(),
            };
            
            let box_max_component = match i {
                0 => box_max.x,
                1 => box_max.y,
                2 => box_max.z,
                _ => unreachable!(),
            };
            
            if dir_component.abs() < 1e-6 {
                if origin_component < box_min_component || origin_component > box_max_component {
                    return None;
                }
            } else {
                let t1 = (box_min_component - origin_component) / dir_component;
                let t2 = (box_max_component - origin_component) / dir_component;
                
                let (t1, t2) = if t1 > t2 { (t2, t1) } else { (t1, t2) };
                
                t_min = t_min.max(t1);
                t_max = t_max.min(t2);
                
                if t_min > t_max {
                    return None;
                }
            }
        }
        
        if t_min > 0.0 {
            Some(t_min)
        } else if t_max > 0.0 {
            Some(t_max)
        } else {
            None
        }
    }
    
    fn update_world_geometry(&mut self) {
        let (vertices, indices) = create_world_geometry_with_blocks(&self.blocks);
        
        // Recreate vertex buffer
        self.vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        
        // Recreate index buffer
        self.index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });
        
        self.num_indices = indices.len() as u32;
    }
    
    fn update(&mut self, delta_time: f32) {
        // Update camera from input
        self.camera.update_from_input(&self.input_state, delta_time);
        
        // Update camera uniform
        let camera_uniform = CameraUniform::from_camera(&self.camera);
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[camera_uniform]));
        
        // Update FPS counter
        self.frame_count += 1;
        if self.last_fps_update.elapsed() >= Duration::from_secs(1) {
            self.fps = self.frame_count as f32 / self.last_fps_update.elapsed().as_secs_f32();
            self.frame_count = 0;
            self.last_fps_update = Instant::now();
        }
    }
    
    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let depth_texture = create_depth_texture(&self.device, &self.config);
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
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }
        
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        
        Ok(())
    }
}

// Data structures
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
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }
    
    fn from_camera(camera: &Camera) -> Self {
        Self {
            view_proj: (camera.projection * camera.view_matrix()).into(),
        }
    }
}

struct Camera {
    position: Point3<f32>,
    yaw: cgmath::Rad<f32>,
    pitch: cgmath::Rad<f32>,
    projection: Matrix4<f32>,
    speed: f32,
    sensitivity: f32,
}

impl Camera {
    fn new(position: (f32, f32, f32), yaw: cgmath::Deg<f32>, pitch: cgmath::Deg<f32>) -> Self {
        Self {
            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
            projection: perspective(Rad(std::f32::consts::FRAC_PI_4), 1280.0 / 720.0, 0.1, 100.0),
            speed: 5.0,
            sensitivity: 0.002,
        }
    }
    
    fn view_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_to_rh(
            self.position,
            self.get_forward_vector(),
            Vector3::unit_y(),
        )
    }
    
    fn get_forward_vector(&self) -> Vector3<f32> {
        Vector3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        ).normalize()
    }
    
    fn process_mouse(&mut self, dx: f32, dy: f32) {
        self.yaw += Rad(dx * self.sensitivity);
        self.pitch -= Rad(dy * self.sensitivity);
        
        // Constrain pitch
        self.pitch = Rad(self.pitch.0.clamp(-std::f32::consts::FRAC_PI_2 + 0.1, std::f32::consts::FRAC_PI_2 - 0.1));
    }
    
    fn update_from_input(&mut self, input: &InputState, delta_time: f32) {
        let forward = self.get_forward_vector();
        let right = forward.cross(Vector3::unit_y()).normalize();
        
        let movement_speed = self.speed * delta_time;
        
        if input.forward {
            self.position += forward * movement_speed;
        }
        if input.backward {
            self.position -= forward * movement_speed;
        }
        if input.right {
            self.position += right * movement_speed;
        }
        if input.left {
            self.position -= right * movement_speed;
        }
        if input.up {
            self.position += Vector3::unit_y() * movement_speed;
        }
        if input.down {
            self.position -= Vector3::unit_y() * movement_speed;
        }
        
        // Prevent going underground
        self.position.y = self.position.y.max(0.2);
    }
}

#[derive(Debug)]
struct InputState {
    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
    up: bool,
    down: bool,
    left_click: bool,
    right_click: bool,
    mouse_captured: bool,
    last_mouse_x: f32,
    last_mouse_y: f32,
}

impl InputState {
    fn new() -> Self {
        Self {
            forward: false,
            backward: false,
            left: false,
            right: false,
            up: false,
            down: false,
            left_click: false,
            right_click: false,
            mouse_captured: false,
            last_mouse_x: 0.0,
            last_mouse_y: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum MaterialType {
    Concrete,
    Steel,
    Wood,
}

#[derive(Debug, Clone)]
struct Block {
    position: Point3<f32>,
    material: MaterialType,
}

fn create_world_geometry() -> (Vec<Vertex>, Vec<u16>) {
    let blocks = create_initial_blocks();
    create_world_geometry_with_blocks(&blocks)
}

fn create_initial_blocks() -> Vec<Block> {
    let mut blocks = Vec::new();
    
    // Create a small starting structure
    blocks.push(Block {
        position: Point3::new(0.0, 1.0, 0.0),
        material: MaterialType::Concrete,
    });
    
    blocks.push(Block {
        position: Point3::new(2.0, 1.0, 0.0),
        material: MaterialType::Steel,
    });
    
    blocks.push(Block {
        position: Point3::new(4.0, 1.0, 0.0),
        material: MaterialType::Wood,
    });
    
    blocks
}

fn create_world_geometry_with_blocks(blocks: &[Block]) -> (Vec<Vertex>, Vec<u16>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    
    // Ground plane
    let ground_color = [0.4, 0.6, 0.3]; // Green
    let ground_size = 50.0;
    let ground_vertices = [
        Vertex { position: [-ground_size, 0.0, -ground_size], color: ground_color },
        Vertex { position: [ground_size, 0.0, -ground_size], color: ground_color },
        Vertex { position: [ground_size, 0.0, ground_size], color: ground_color },
        Vertex { position: [-ground_size, 0.0, ground_size], color: ground_color },
    ];
    
    let base_index = vertices.len() as u16;
    vertices.extend_from_slice(&ground_vertices);
    indices.extend_from_slice(&[
        base_index, base_index + 1, base_index + 2,
        base_index, base_index + 2, base_index + 3,
    ]);
    
    // Add blocks
    for block in blocks {
        let color = match block.material {
            MaterialType::Concrete => [0.7, 0.7, 0.7],
            MaterialType::Steel => [0.8, 0.8, 0.9],
            MaterialType::Wood => [0.6, 0.4, 0.2],
        };
        
        add_cube_to_mesh(&mut vertices, &mut indices, block.position, 1.0, color);
    }
    
    (vertices, indices)
}

fn add_cube_to_mesh(vertices: &mut Vec<Vertex>, indices: &mut Vec<u16>, position: Point3<f32>, size: f32, color: [f32; 3]) {
    let half_size = size / 2.0;
    let base_index = vertices.len() as u16;
    
    // Cube vertices
    let cube_vertices = [
        // Front face
        Vertex { position: [position.x - half_size, position.y - half_size, position.z + half_size], color },
        Vertex { position: [position.x + half_size, position.y - half_size, position.z + half_size], color },
        Vertex { position: [position.x + half_size, position.y + half_size, position.z + half_size], color },
        Vertex { position: [position.x - half_size, position.y + half_size, position.z + half_size], color },
        
        // Back face
        Vertex { position: [position.x + half_size, position.y - half_size, position.z - half_size], color },
        Vertex { position: [position.x - half_size, position.y - half_size, position.z - half_size], color },
        Vertex { position: [position.x - half_size, position.y + half_size, position.z - half_size], color },
        Vertex { position: [position.x + half_size, position.y + half_size, position.z - half_size], color },
        
        // Left face
        Vertex { position: [position.x - half_size, position.y - half_size, position.z - half_size], color },
        Vertex { position: [position.x - half_size, position.y - half_size, position.z + half_size], color },
        Vertex { position: [position.x - half_size, position.y + half_size, position.z + half_size], color },
        Vertex { position: [position.x - half_size, position.y + half_size, position.z - half_size], color },
        
        // Right face
        Vertex { position: [position.x + half_size, position.y - half_size, position.z + half_size], color },
        Vertex { position: [position.x + half_size, position.y - half_size, position.z - half_size], color },
        Vertex { position: [position.x + half_size, position.y + half_size, position.z - half_size], color },
        Vertex { position: [position.x + half_size, position.y + half_size, position.z + half_size], color },
        
        // Top face
        Vertex { position: [position.x - half_size, position.y + half_size, position.z + half_size], color },
        Vertex { position: [position.x + half_size, position.y + half_size, position.z + half_size], color },
        Vertex { position: [position.x + half_size, position.y + half_size, position.z - half_size], color },
        Vertex { position: [position.x - half_size, position.y + half_size, position.z - half_size], color },
        
        // Bottom face
        Vertex { position: [position.x - half_size, position.y - half_size, position.z - half_size], color },
        Vertex { position: [position.x + half_size, position.y - half_size, position.z - half_size], color },
        Vertex { position: [position.x + half_size, position.y - half_size, position.z + half_size], color },
        Vertex { position: [position.x - half_size, position.y - half_size, position.z + half_size], color },
    ];
    
    vertices.extend_from_slice(&cube_vertices);
    
    // Cube indices
    let cube_indices = [
        // Front face
        base_index, base_index + 1, base_index + 2,
        base_index, base_index + 2, base_index + 3,
        
        // Back face
        base_index + 4, base_index + 5, base_index + 6,
        base_index + 4, base_index + 6, base_index + 7,
        
        // Left face
        base_index + 8, base_index + 9, base_index + 10,
        base_index + 8, base_index + 10, base_index + 11,
        
        // Right face
        base_index + 12, base_index + 13, base_index + 14,
        base_index + 12, base_index + 14, base_index + 15,
        
        // Top face
        base_index + 16, base_index + 17, base_index + 18,
        base_index + 16, base_index + 18, base_index + 19,
        
        // Bottom face
        base_index + 20, base_index + 21, base_index + 22,
        base_index + 20, base_index + 22, base_index + 23,
    ];
    
    indices.extend_from_slice(&cube_indices);
}

fn create_depth_texture(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> wgpu::Texture {
    let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
        size: wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        label: None,
        view_formats: &[],
    });
    
    depth_texture
}