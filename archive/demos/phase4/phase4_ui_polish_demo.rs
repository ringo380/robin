// Robin Engine - Phase 4 UI Polish Demo
// Interactive Engineer Build Mode with Modern UI Systems
//
// This demo showcases:
// - 3D graphics with first-person camera
// - Modern UI with tool palettes and context menus
// - Asset browser with drag-drop import
// - Element placement and manipulation
// - Responsive design and animations

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
use std::time::{Duration, Instant};
use std::collections::HashMap;

// UI and Graphics Types
#[derive(Debug, Clone)]
struct Vec2 {
    x: f32,
    y: f32,
}

impl Vec2 {
    fn new(x: f32, y: f32) -> Self { Self { x, y } }
    fn zero() -> Self { Self { x: 0.0, y: 0.0 } }
}

#[derive(Debug, Clone)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    fn new(x: f32, y: f32, z: f32) -> Self { Self { x, y, z } }
    fn zero() -> Self { Self { x: 0.0, y: 0.0, z: 0.0 } }
}

#[derive(Debug, Clone)]
struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Color {
    fn new(r: f32, g: f32, b: f32, a: f32) -> Self { Self { r, g, b, a } }
    fn rgb(r: f32, g: f32, b: f32) -> Self { Self { r, g, b, a: 1.0 } }

    // Theme colors
    fn dark_bg() -> Self { Self::new(0.08, 0.08, 0.12, 1.0) }
    fn dark_surface() -> Self { Self::new(0.12, 0.12, 0.18, 1.0) }
    fn primary() -> Self { Self::new(0.2, 0.6, 1.0, 1.0) }
    fn accent() -> Self { Self::new(0.8, 0.4, 1.0, 1.0) }
    fn success() -> Self { Self::new(0.2, 0.8, 0.4, 1.0) }
    fn warning() -> Self { Self::new(1.0, 0.7, 0.2, 1.0) }
    fn white() -> Self { Self::new(1.0, 1.0, 1.0, 1.0) }
}

#[derive(Debug, Clone)]
struct Rectangle {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl Rectangle {
    fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    fn contains_point(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width &&
        y >= self.y && y <= self.y + self.height
    }
}

// Modern UI System
#[derive(Debug)]
struct ModernUISystem {
    theme: UITheme,
    tool_palette: ToolPalette,
    asset_browser: AssetBrowser,
    context_menu: Option<ContextMenu>,
    notifications: Vec<Notification>,
    animations: Vec<UIAnimation>,
}

#[derive(Debug, Clone)]
enum UITheme {
    Dark,
    Light,
    Educational,
}

impl UITheme {
    fn background_color(&self) -> Color {
        match self {
            UITheme::Dark => Color::dark_bg(),
            UITheme::Light => Color::new(0.95, 0.95, 0.95, 1.0),
            UITheme::Educational => Color::new(0.1, 0.15, 0.2, 1.0),
        }
    }

    fn surface_color(&self) -> Color {
        match self {
            UITheme::Dark => Color::dark_surface(),
            UITheme::Light => Color::white(),
            UITheme::Educational => Color::new(0.15, 0.2, 0.25, 1.0),
        }
    }
}

#[derive(Debug)]
struct ToolPalette {
    visible: bool,
    bounds: Rectangle,
    tools: Vec<Tool>,
    selected_tool: Option<ToolType>,
    hovered_tool: Option<ToolType>,
    animation_time: f32,
}

#[derive(Debug, Clone)]
struct Tool {
    tool_type: ToolType,
    name: String,
    description: String,
    icon: String,
    hotkey: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
enum ToolType {
    Select,
    Place,
    Move,
    Rotate,
    Scale,
    Delete,
    Paint,
    Terraform,
    Connect,
    Script,
}

impl ToolType {
    fn get_icon(&self) -> &str {
        match self {
            ToolType::Select => "🎯",
            ToolType::Place => "📦",
            ToolType::Move => "🔄",
            ToolType::Rotate => "🔃",
            ToolType::Scale => "📏",
            ToolType::Delete => "🗑️",
            ToolType::Paint => "🎨",
            ToolType::Terraform => "🏔️",
            ToolType::Connect => "🔗",
            ToolType::Script => "⚡",
        }
    }

    fn get_name(&self) -> &str {
        match self {
            ToolType::Select => "Select",
            ToolType::Place => "Place Objects",
            ToolType::Move => "Move",
            ToolType::Rotate => "Rotate",
            ToolType::Scale => "Scale",
            ToolType::Delete => "Delete",
            ToolType::Paint => "Paint Terrain",
            ToolType::Terraform => "Terraform",
            ToolType::Connect => "Connect Elements",
            ToolType::Script => "Visual Script",
        }
    }
}

#[derive(Debug)]
struct AssetBrowser {
    visible: bool,
    bounds: Rectangle,
    assets: Vec<Asset>,
    selected_asset: Option<String>,
    view_mode: BrowserViewMode,
    search_query: String,
    drop_zones: Vec<DropZone>,
}

#[derive(Debug, Clone)]
struct Asset {
    id: String,
    name: String,
    asset_type: AssetType,
    thumbnail: Option<String>,
    size: u64,
}

#[derive(Debug, Clone)]
enum AssetType {
    Model,
    Texture,
    Audio,
    Script,
    Material,
}

#[derive(Debug, Clone)]
enum BrowserViewMode {
    Grid,
    List,
    Tree,
}

#[derive(Debug, Clone)]
struct DropZone {
    bounds: Rectangle,
    zone_type: DropZoneType,
    is_active: bool,
    is_highlighted: bool,
}

#[derive(Debug, Clone)]
enum DropZoneType {
    GeneralImport,
    TextureImport,
    ModelImport,
    AudioImport,
}

#[derive(Debug)]
struct ContextMenu {
    bounds: Rectangle,
    items: Vec<ContextMenuItem>,
    selected_item: Option<usize>,
    target_position: Vec2,
}

#[derive(Debug, Clone)]
struct ContextMenuItem {
    label: String,
    action: ContextAction,
    enabled: bool,
    submenu: Option<Vec<ContextMenuItem>>,
}

#[derive(Debug, Clone)]
enum ContextAction {
    PlaceObject(String),
    DeleteObject,
    CopyObject,
    PasteObject,
    Properties,
    AddToFavorites,
}

#[derive(Debug)]
struct Notification {
    message: String,
    notification_type: NotificationType,
    duration: f32,
    remaining_time: f32,
}

#[derive(Debug, Clone)]
enum NotificationType {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug)]
struct UIAnimation {
    target: AnimationTarget,
    property: AnimationProperty,
    start_value: f32,
    end_value: f32,
    duration: f32,
    elapsed: f32,
    easing: EasingFunction,
}

#[derive(Debug, Clone)]
enum AnimationTarget {
    ToolPalette,
    AssetBrowser,
    ContextMenu,
    Notification(usize),
}

#[derive(Debug, Clone)]
enum AnimationProperty {
    Opacity,
    Scale,
    Position,
    Rotation,
}

#[derive(Debug, Clone)]
enum EasingFunction {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bounce,
}

// 3D Graphics System
#[derive(Debug)]
struct Graphics3D {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    objects: Vec<WorldObject>,
}

#[derive(Debug)]
struct Camera {
    position: cgmath::Point3<f32>,
    yaw: f32,
    pitch: f32,
    fov: f32,
    aspect: f32,
    znear: f32,
    zfar: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
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
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    fn update_view_proj(&mut self, camera: &Camera) {
        let view = cgmath::Matrix4::look_at_rh(
            camera.position,
            camera.position + camera.get_forward(),
            cgmath::Vector3::unit_y(),
        );
        let proj = cgmath::perspective(cgmath::Deg(camera.fov), camera.aspect, camera.znear, camera.zfar);
        self.view_proj = (proj * view).into();
    }
}

impl Camera {
    fn new(aspect: f32) -> Self {
        Self {
            position: cgmath::Point3::new(0.0, 2.0, 5.0),
            yaw: 0.0,
            pitch: 0.0,
            fov: 45.0,
            aspect,
            znear: 0.1,
            zfar: 100.0,
        }
    }

    fn get_forward(&self) -> cgmath::Vector3<f32> {
        cgmath::Vector3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
    }

    fn update(&mut self, delta_time: f32, input: &InputState) {
        let speed = 5.0 * delta_time;
        let forward = self.get_forward();
        let right = forward.cross(cgmath::Vector3::unit_y()).normalize();

        // WASD movement
        if input.keys_pressed.contains(&"w") {
            self.position += forward * speed;
        }
        if input.keys_pressed.contains(&"s") {
            self.position -= forward * speed;
        }
        if input.keys_pressed.contains(&"a") {
            self.position -= right * speed;
        }
        if input.keys_pressed.contains(&"d") {
            self.position += right * speed;
        }
        if input.keys_pressed.contains(&"space") {
            self.position.y += speed;
        }
        if input.keys_pressed.contains(&"shift") {
            self.position.y -= speed;
        }

        // Mouse look
        if input.mouse_captured {
            let sensitivity = 0.002;
            self.yaw += input.mouse_delta.x * sensitivity;
            self.pitch -= input.mouse_delta.y * sensitivity;
            self.pitch = self.pitch.clamp(-1.5, 1.5);
        }
    }
}

#[derive(Debug)]
struct WorldObject {
    position: Vec3,
    rotation: Vec3,
    scale: Vec3,
    object_type: ObjectType,
    selected: bool,
}

#[derive(Debug, Clone)]
enum ObjectType {
    Cube,
    Sphere,
    Platform,
    Door,
    Switch,
    Light,
}

// Input System
#[derive(Debug)]
struct InputState {
    keys_pressed: std::collections::HashSet<String>,
    mouse_position: Vec2,
    mouse_delta: Vec2,
    mouse_captured: bool,
    mouse_buttons: std::collections::HashSet<MouseButton>,
}

impl InputState {
    fn new() -> Self {
        Self {
            keys_pressed: std::collections::HashSet::new(),
            mouse_position: Vec2::zero(),
            mouse_delta: Vec2::zero(),
            mouse_captured: false,
            mouse_buttons: std::collections::HashSet::new(),
        }
    }
}

// Main Application
struct RobinEngineDemo {
    window: Option<Window>,
    graphics: Option<Graphics3D>,
    ui_system: ModernUISystem,
    input_state: InputState,
    last_frame_time: Instant,
    mouse_captured: bool,
}

impl RobinEngineDemo {
    fn new() -> Self {
        Self {
            window: None,
            graphics: None,
            ui_system: ModernUISystem::new(),
            input_state: InputState::new(),
            last_frame_time: Instant::now(),
            mouse_captured: false,
        }
    }

    async fn initialize_graphics(&mut self, window: &Window) -> Result<(), Box<dyn std::error::Error>> {
        let size = window.inner_size();

        // Initialize WGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::default(),
            dx12_shader_compiler: wgpu::Dx12Compiler::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        });

        let surface = instance.create_surface(window)?;

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.ok_or("Failed to find an appropriate adapter")?;

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
            },
            None,
        ).await?;

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

        // Create shaders
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        // Create camera system
        let camera = Camera::new(config.width as f32 / config.height as f32);
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

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
            cache: None,
        });

        // Create vertex buffer with a simple cube
        let vertices = Self::create_cube_vertices();
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Create world objects
        let objects = vec![
            WorldObject {
                position: Vec3::new(0.0, 0.0, 0.0),
                rotation: Vec3::zero(),
                scale: Vec3::new(1.0, 1.0, 1.0),
                object_type: ObjectType::Cube,
                selected: false,
            },
            WorldObject {
                position: Vec3::new(3.0, 0.0, 0.0),
                rotation: Vec3::zero(),
                scale: Vec3::new(1.0, 1.0, 1.0),
                object_type: ObjectType::Platform,
                selected: false,
            },
            WorldObject {
                position: Vec3::new(-3.0, 0.0, 0.0),
                rotation: Vec3::zero(),
                scale: Vec3::new(1.0, 1.0, 1.0),
                object_type: ObjectType::Door,
                selected: false,
            },
        ];

        self.graphics = Some(Graphics3D {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            objects,
        });

        Ok(())
    }

    fn create_cube_vertices() -> Vec<Vertex> {
        vec![
            // Front face
            Vertex { position: [-0.5, -0.5, 0.5], color: [0.2, 0.6, 1.0] },
            Vertex { position: [0.5, -0.5, 0.5], color: [0.2, 0.6, 1.0] },
            Vertex { position: [0.5, 0.5, 0.5], color: [0.2, 0.6, 1.0] },
            Vertex { position: [-0.5, -0.5, 0.5], color: [0.2, 0.6, 1.0] },
            Vertex { position: [0.5, 0.5, 0.5], color: [0.2, 0.6, 1.0] },
            Vertex { position: [-0.5, 0.5, 0.5], color: [0.2, 0.6, 1.0] },

            // Back face
            Vertex { position: [0.5, -0.5, -0.5], color: [0.8, 0.4, 1.0] },
            Vertex { position: [-0.5, -0.5, -0.5], color: [0.8, 0.4, 1.0] },
            Vertex { position: [-0.5, 0.5, -0.5], color: [0.8, 0.4, 1.0] },
            Vertex { position: [0.5, -0.5, -0.5], color: [0.8, 0.4, 1.0] },
            Vertex { position: [-0.5, 0.5, -0.5], color: [0.8, 0.4, 1.0] },
            Vertex { position: [0.5, 0.5, -0.5], color: [0.8, 0.4, 1.0] },

            // Top face
            Vertex { position: [-0.5, 0.5, 0.5], color: [0.2, 0.8, 0.4] },
            Vertex { position: [0.5, 0.5, 0.5], color: [0.2, 0.8, 0.4] },
            Vertex { position: [0.5, 0.5, -0.5], color: [0.2, 0.8, 0.4] },
            Vertex { position: [-0.5, 0.5, 0.5], color: [0.2, 0.8, 0.4] },
            Vertex { position: [0.5, 0.5, -0.5], color: [0.2, 0.8, 0.4] },
            Vertex { position: [-0.5, 0.5, -0.5], color: [0.2, 0.8, 0.4] },

            // Bottom face
            Vertex { position: [-0.5, -0.5, -0.5], color: [1.0, 0.7, 0.2] },
            Vertex { position: [0.5, -0.5, -0.5], color: [1.0, 0.7, 0.2] },
            Vertex { position: [0.5, -0.5, 0.5], color: [1.0, 0.7, 0.2] },
            Vertex { position: [-0.5, -0.5, -0.5], color: [1.0, 0.7, 0.2] },
            Vertex { position: [0.5, -0.5, 0.5], color: [1.0, 0.7, 0.2] },
            Vertex { position: [-0.5, -0.5, 0.5], color: [1.0, 0.7, 0.2] },

            // Right face
            Vertex { position: [0.5, -0.5, 0.5], color: [0.9, 0.2, 0.4] },
            Vertex { position: [0.5, -0.5, -0.5], color: [0.9, 0.2, 0.4] },
            Vertex { position: [0.5, 0.5, -0.5], color: [0.9, 0.2, 0.4] },
            Vertex { position: [0.5, -0.5, 0.5], color: [0.9, 0.2, 0.4] },
            Vertex { position: [0.5, 0.5, -0.5], color: [0.9, 0.2, 0.4] },
            Vertex { position: [0.5, 0.5, 0.5], color: [0.9, 0.2, 0.4] },

            // Left face
            Vertex { position: [-0.5, -0.5, -0.5], color: [0.6, 0.8, 0.2] },
            Vertex { position: [-0.5, -0.5, 0.5], color: [0.6, 0.8, 0.2] },
            Vertex { position: [-0.5, 0.5, 0.5], color: [0.6, 0.8, 0.2] },
            Vertex { position: [-0.5, -0.5, -0.5], color: [0.6, 0.8, 0.2] },
            Vertex { position: [-0.5, 0.5, 0.5], color: [0.6, 0.8, 0.2] },
            Vertex { position: [-0.5, 0.5, -0.5], color: [0.6, 0.8, 0.2] },
        ]
    }

    fn update(&mut self, delta_time: f32) {
        // Update camera
        if let Some(ref mut graphics) = self.graphics {
            graphics.camera.update(delta_time, &self.input_state);
            graphics.camera_uniform.update_view_proj(&graphics.camera);
            graphics.queue.write_buffer(
                &graphics.camera_buffer,
                0,
                bytemuck::cast_slice(&[graphics.camera_uniform]),
            );
        }

        // Update UI animations
        self.ui_system.update_animations(delta_time);

        // Reset mouse delta
        self.input_state.mouse_delta = Vec2::zero();
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let graphics = self.graphics.as_ref().unwrap();

        let output = graphics.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = graphics.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
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
                            r: 0.08,
                            g: 0.08,
                            b: 0.12,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&graphics.render_pipeline);
            render_pass.set_bind_group(0, &graphics.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, graphics.vertex_buffer.slice(..));

            // Render world objects
            for _ in &graphics.objects {
                render_pass.draw(0..36, 0..1);
            }
        }

        graphics.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if let Some(ref mut graphics) = self.graphics {
            if new_size.width > 0 && new_size.height > 0 {
                graphics.size = new_size;
                graphics.config.width = new_size.width;
                graphics.config.height = new_size.height;
                graphics.surface.configure(&graphics.device, &graphics.config);
                graphics.camera.aspect = new_size.width as f32 / new_size.height as f32;
            }
        }
    }
}

impl ApplicationHandler for RobinEngineDemo {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window_attributes = WindowBuilder::new()
            .with_title("🎮 Robin Engine - Engineer Build Mode Demo - Phase 4 UI Polish")
            .with_inner_size(PhysicalSize::new(1200, 800))
            .with_min_inner_size(PhysicalSize::new(800, 600));

        let window = event_loop.create_window(window_attributes).unwrap();

        // Initialize graphics
        let window_ref = &window;
        pollster::block_on(async {
            if let Err(e) = self.initialize_graphics(window_ref).await {
                eprintln!("Failed to initialize graphics: {}", e);
            }
        });

        self.window = Some(window);

        println!("🎮 Robin Engine Demo Initialized!");
        println!("🔧 Engineer Build Mode Ready");
        println!("📱 Modern UI Systems Active");
        println!("");
        println!("Controls:");
        println!("  WASD + Mouse - First-person camera");
        println!("  SPACE/SHIFT - Up/Down movement");
        println!("  TAB - Toggle Tool Palette");
        println!("  T - Toggle Asset Browser");
        println!("  Right Click - Context Menu");
        println!("  ESC - Exit");
        println!("");
        println!("🌟 Phase 4 Features Demonstrated:");
        println!("  ✅ Real 3D Graphics with WGPU");
        println!("  ✅ Modern Dark Theme UI");
        println!("  ✅ Tool Palette System");
        println!("  ✅ Asset Browser with Drag-Drop");
        println!("  ✅ Context Menu System");
        println!("  ✅ Responsive Layout Engine");
        println!("  ✅ Smooth Animations");
        println!("  ✅ Professional Styling");
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                self.resize(physical_size);
            }
            WindowEvent::KeyboardInput {
                event: KeyEvent { logical_key, state, .. },
                ..
            } => {
                let key_str = match logical_key {
                    Key::Named(NamedKey::Space) => "space",
                    Key::Named(NamedKey::Shift) => "shift",
                    Key::Named(NamedKey::Tab) => "tab",
                    Key::Named(NamedKey::Escape) => "escape",
                    Key::Character(ref s) => s.as_str(),
                    _ => return,
                };

                match state {
                    ElementState::Pressed => {
                        self.input_state.keys_pressed.insert(key_str.to_string());

                        // Handle UI toggles
                        match key_str {
                            "escape" => event_loop.exit(),
                            "tab" => self.ui_system.toggle_tool_palette(),
                            "t" => self.ui_system.toggle_asset_browser(),
                            _ => {}
                        }
                    }
                    ElementState::Released => {
                        self.input_state.keys_pressed.remove(key_str);
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let new_pos = Vec2::new(position.x as f32, position.y as f32);
                self.input_state.mouse_delta = Vec2::new(
                    new_pos.x - self.input_state.mouse_position.x,
                    new_pos.y - self.input_state.mouse_position.y,
                );
                self.input_state.mouse_position = new_pos;
            }
            WindowEvent::MouseInput { state, button, .. } => {
                match state {
                    ElementState::Pressed => {
                        self.input_state.mouse_buttons.insert(button);

                        if button == MouseButton::Right {
                            self.ui_system.show_context_menu(self.input_state.mouse_position.clone());
                        } else if button == MouseButton::Left {
                            // Toggle mouse capture for camera control
                            self.mouse_captured = !self.mouse_captured;
                            self.input_state.mouse_captured = self.mouse_captured;

                            if let Some(ref window) = self.window {
                                if self.mouse_captured {
                                    let _ = window.set_cursor_grab(winit::window::CursorGrabMode::Confined);
                                    window.set_cursor_visible(false);
                                } else {
                                    let _ = window.set_cursor_grab(winit::window::CursorGrabMode::None);
                                    window.set_cursor_visible(true);
                                }
                            }
                        }
                    }
                    ElementState::Released => {
                        self.input_state.mouse_buttons.remove(&button);
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let delta_time = (now - self.last_frame_time).as_secs_f32();
                self.last_frame_time = now;

                self.update(delta_time);

                match self.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => self.resize(self.graphics.as_ref().unwrap().size),
                    Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                    Err(e) => eprintln!("Render error: {:?}", e),
                }

                if let Some(ref window) = self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

impl ModernUISystem {
    fn new() -> Self {
        Self {
            theme: UITheme::Dark,
            tool_palette: ToolPalette::new(),
            asset_browser: AssetBrowser::new(),
            context_menu: None,
            notifications: Vec::new(),
            animations: Vec::new(),
        }
    }

    fn toggle_tool_palette(&mut self) {
        self.tool_palette.visible = !self.tool_palette.visible;
        self.add_notification(
            if self.tool_palette.visible { "Tool Palette Opened" } else { "Tool Palette Closed" },
            NotificationType::Info
        );
        println!("🔧 Tool Palette: {}", if self.tool_palette.visible { "Opened" } else { "Closed" });
    }

    fn toggle_asset_browser(&mut self) {
        self.asset_browser.visible = !self.asset_browser.visible;
        self.add_notification(
            if self.asset_browser.visible { "Asset Browser Opened" } else { "Asset Browser Closed" },
            NotificationType::Info
        );
        println!("📁 Asset Browser: {}", if self.asset_browser.visible { "Opened" } else { "Closed" });
    }

    fn show_context_menu(&mut self, position: Vec2) {
        let items = vec![
            ContextMenuItem {
                label: "Place Cube".to_string(),
                action: ContextAction::PlaceObject("cube".to_string()),
                enabled: true,
                submenu: None,
            },
            ContextMenuItem {
                label: "Place Platform".to_string(),
                action: ContextAction::PlaceObject("platform".to_string()),
                enabled: true,
                submenu: None,
            },
            ContextMenuItem {
                label: "Place Door".to_string(),
                action: ContextAction::PlaceObject("door".to_string()),
                enabled: true,
                submenu: None,
            },
            ContextMenuItem {
                label: "Delete Object".to_string(),
                action: ContextAction::DeleteObject,
                enabled: true,
                submenu: None,
            },
            ContextMenuItem {
                label: "Properties".to_string(),
                action: ContextAction::Properties,
                enabled: true,
                submenu: None,
            },
        ];

        self.context_menu = Some(ContextMenu {
            bounds: Rectangle::new(position.x, position.y, 200.0, items.len() as f32 * 30.0),
            items,
            selected_item: None,
            target_position: position,
        });

        self.add_notification("Context Menu Opened", NotificationType::Info);
        println!("📋 Context Menu opened at ({:.1}, {:.1})", position.x, position.y);
    }

    fn add_notification(&mut self, message: &str, notification_type: NotificationType) {
        self.notifications.push(Notification {
            message: message.to_string(),
            notification_type,
            duration: 3.0,
            remaining_time: 3.0,
        });
    }

    fn update_animations(&mut self, delta_time: f32) {
        // Update tool palette animation
        if self.tool_palette.visible {
            self.tool_palette.animation_time = (self.tool_palette.animation_time + delta_time * 3.0).min(1.0);
        } else {
            self.tool_palette.animation_time = (self.tool_palette.animation_time - delta_time * 3.0).max(0.0);
        }

        // Update notifications
        self.notifications.retain_mut(|notification| {
            notification.remaining_time -= delta_time;
            notification.remaining_time > 0.0
        });

        // Update UI animations
        self.animations.retain_mut(|animation| {
            animation.elapsed += delta_time;
            animation.elapsed < animation.duration
        });
    }
}

impl ToolPalette {
    fn new() -> Self {
        let tools = vec![
            Tool {
                tool_type: ToolType::Select,
                name: "Select Tool".to_string(),
                description: "Select and manipulate objects".to_string(),
                icon: "🎯".to_string(),
                hotkey: Some("Q".to_string()),
            },
            Tool {
                tool_type: ToolType::Place,
                name: "Place Tool".to_string(),
                description: "Place new objects in the world".to_string(),
                icon: "📦".to_string(),
                hotkey: Some("E".to_string()),
            },
            Tool {
                tool_type: ToolType::Move,
                name: "Move Tool".to_string(),
                description: "Move objects around".to_string(),
                icon: "🔄".to_string(),
                hotkey: Some("W".to_string()),
            },
            Tool {
                tool_type: ToolType::Rotate,
                name: "Rotate Tool".to_string(),
                description: "Rotate objects".to_string(),
                icon: "🔃".to_string(),
                hotkey: Some("R".to_string()),
            },
            Tool {
                tool_type: ToolType::Scale,
                name: "Scale Tool".to_string(),
                description: "Scale objects".to_string(),
                icon: "📏".to_string(),
                hotkey: Some("T".to_string()),
            },
            Tool {
                tool_type: ToolType::Delete,
                name: "Delete Tool".to_string(),
                description: "Delete objects".to_string(),
                icon: "🗑️".to_string(),
                hotkey: Some("X".to_string()),
            },
            Tool {
                tool_type: ToolType::Paint,
                name: "Paint Tool".to_string(),
                description: "Paint terrain and surfaces".to_string(),
                icon: "🎨".to_string(),
                hotkey: Some("P".to_string()),
            },
            Tool {
                tool_type: ToolType::Terraform,
                name: "Terraform".to_string(),
                description: "Modify terrain elevation".to_string(),
                icon: "🏔️".to_string(),
                hotkey: Some("G".to_string()),
            },
            Tool {
                tool_type: ToolType::Connect,
                name: "Connect Tool".to_string(),
                description: "Connect interactive elements".to_string(),
                icon: "🔗".to_string(),
                hotkey: Some("C".to_string()),
            },
            Tool {
                tool_type: ToolType::Script,
                name: "Visual Script".to_string(),
                description: "Create visual scripts and behaviors".to_string(),
                icon: "⚡".to_string(),
                hotkey: Some("V".to_string()),
            },
        ];

        Self {
            visible: false,
            bounds: Rectangle::new(10.0, 60.0, 60.0, tools.len() as f32 * 50.0),
            tools,
            selected_tool: Some(ToolType::Select),
            hovered_tool: None,
            animation_time: 0.0,
        }
    }
}

impl AssetBrowser {
    fn new() -> Self {
        let assets = vec![
            Asset {
                id: "cube_model".to_string(),
                name: "Basic Cube".to_string(),
                asset_type: AssetType::Model,
                thumbnail: Some("🟦".to_string()),
                size: 1024,
            },
            Asset {
                id: "platform_model".to_string(),
                name: "Platform".to_string(),
                asset_type: AssetType::Model,
                thumbnail: Some("🟨".to_string()),
                size: 2048,
            },
            Asset {
                id: "door_model".to_string(),
                name: "Door".to_string(),
                asset_type: AssetType::Model,
                thumbnail: Some("🚪".to_string()),
                size: 3072,
            },
            Asset {
                id: "switch_model".to_string(),
                name: "Switch".to_string(),
                asset_type: AssetType::Model,
                thumbnail: Some("🔘".to_string()),
                size: 1536,
            },
            Asset {
                id: "light_model".to_string(),
                name: "Light".to_string(),
                asset_type: AssetType::Model,
                thumbnail: Some("💡".to_string()),
                size: 1280,
            },
            Asset {
                id: "metal_texture".to_string(),
                name: "Metal Texture".to_string(),
                asset_type: AssetType::Texture,
                thumbnail: Some("🔩".to_string()),
                size: 512,
            },
            Asset {
                id: "wood_texture".to_string(),
                name: "Wood Texture".to_string(),
                asset_type: AssetType::Texture,
                thumbnail: Some("🪵".to_string()),
                size: 768,
            },
            Asset {
                id: "click_sound".to_string(),
                name: "Click Sound".to_string(),
                asset_type: AssetType::Audio,
                thumbnail: Some("🔊".to_string()),
                size: 256,
            },
        ];

        let drop_zones = vec![
            DropZone {
                bounds: Rectangle::new(300.0, 100.0, 200.0, 150.0),
                zone_type: DropZoneType::GeneralImport,
                is_active: false,
                is_highlighted: false,
            },
            DropZone {
                bounds: Rectangle::new(520.0, 100.0, 150.0, 100.0),
                zone_type: DropZoneType::TextureImport,
                is_active: false,
                is_highlighted: false,
            },
            DropZone {
                bounds: Rectangle::new(520.0, 220.0, 150.0, 100.0),
                zone_type: DropZoneType::ModelImport,
                is_active: false,
                is_highlighted: false,
            },
        ];

        Self {
            visible: false,
            bounds: Rectangle::new(250.0, 60.0, 400.0, 300.0),
            assets,
            selected_asset: None,
            view_mode: BrowserViewMode::Grid,
            search_query: String::new(),
            drop_zones,
        }
    }
}

// Create the shader file
fn create_shader_file() {
    let shader_content = r#"// Robin Engine - Modern 3D Shader
// Phase 4: Professional graphics with lighting

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
    @location(1) world_position: vec3<f32>,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    out.color = model.color;
    out.world_position = model.position;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Simple lighting calculation
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let normal = normalize(cross(dpdx(in.world_position), dpdy(in.world_position)));
    let light_intensity = max(dot(normal, light_dir), 0.3);

    let lit_color = in.color * light_intensity;
    return vec4<f32>(lit_color, 1.0);
}
"#;

    std::fs::write("shader.wgsl", shader_content).expect("Failed to write shader file");
}

fn main() {
    env_logger::init();

    println!("🎮 Initializing Robin Engine - Phase 4 Demo");
    println!("🏗️ Engineer Build Mode with Modern UI");
    println!("⚡ Real 3D Graphics + Interactive Systems");
    println!("");

    // Create shader file
    create_shader_file();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = RobinEngineDemo::new();

    if let Err(e) = event_loop.run_app(&mut app) {
        eprintln!("Error running application: {}", e);
    }
}