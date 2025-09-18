use std::collections::HashMap;
use wgpu::util::DeviceExt;
use cgmath::{Matrix4, Vector3, Vector4, Point3, perspective, Deg, InnerSpace, SquareMatrix, Matrix};
use crate::engine::error::RobinResult;
use crate::engine::graphics::{Vertex, Mesh, Camera};
// use crate::engine::performance::PerformanceManager; // Temporarily disabled

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex3D {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2],
    pub tangent: [f32; 3],
    pub color: [f32; 4],
}

impl Vertex3D {
    const ATTRIBS: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
        0 => Float32x3, // position
        1 => Float32x3, // normal
        2 => Float32x2, // tex_coords
        3 => Float32x3, // tangent
        4 => Float32x4, // color
    ];
    
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceData {
    pub model_matrix: [[f32; 4]; 4],
    pub normal_matrix: [[f32; 4]; 4],
    pub material_id: u32,
    pub _padding: [u32; 3],
}

impl InstanceData {
    const ATTRIBS: [wgpu::VertexAttribute; 10] = wgpu::vertex_attr_array![
        5 => Float32x4, // model_matrix col 0
        6 => Float32x4, // model_matrix col 1
        7 => Float32x4, // model_matrix col 2
        8 => Float32x4, // model_matrix col 3
        9 => Float32x4, // normal_matrix col 0
        10 => Float32x4, // normal_matrix col 1
        11 => Float32x4, // normal_matrix col 2
        12 => Float32x4, // normal_matrix col 3
        13 => Uint32,   // material_id
        14 => Uint32x3, // padding
    ];
    
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_matrix: [[f32; 4]; 4],
    pub projection_matrix: [[f32; 4]; 4],
    pub view_projection_matrix: [[f32; 4]; 4],
    pub camera_position: [f32; 4],
    pub camera_direction: [f32; 4],
    pub near_plane: f32,
    pub far_plane: f32,
    pub fov: f32,
    pub aspect_ratio: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    pub light_type: u32,
    pub position: [f32; 3],
    pub direction: [f32; 3],
    pub color: [f32; 3],
    pub intensity: f32,
    pub range: f32,
    pub inner_cone: f32,
    pub outer_cone: f32,
    pub _padding: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaterialUniform {
    pub albedo: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub emission: [f32; 3],
    pub alpha_cutoff: f32,
    pub texture_flags: u32,
    pub _padding: [u32; 2],
}

#[derive(Debug, Clone, Copy)]
pub enum LightType {
    Directional = 0,
    Point = 1,
    Spot = 2,
}

pub struct Light {
    pub light_type: LightType,
    pub position: Vector3<f32>,
    pub direction: Vector3<f32>,
    pub color: Vector3<f32>,
    pub intensity: f32,
    pub range: f32,
    pub inner_cone: f32,
    pub outer_cone: f32,
    pub cast_shadows: bool,
    pub enabled: bool,
}

impl Default for Light {
    fn default() -> Self {
        Self {
            light_type: LightType::Directional,
            position: Vector3::new(0.0, 10.0, 0.0),
            direction: Vector3::new(0.0, -1.0, 0.0),
            color: Vector3::new(1.0, 1.0, 1.0),
            intensity: 1.0,
            range: 100.0,
            inner_cone: 30.0,
            outer_cone: 45.0,
            cast_shadows: true,
            enabled: true,
        }
    }
}

pub struct Material {
    pub name: String,
    pub albedo: Vector4<f32>,
    pub metallic: f32,
    pub roughness: f32,
    pub emission: Vector3<f32>,
    pub alpha_cutoff: f32,
    pub albedo_texture: Option<u32>,
    pub normal_texture: Option<u32>,
    pub metallic_roughness_texture: Option<u32>,
    pub emission_texture: Option<u32>,
    pub occlusion_texture: Option<u32>,
    pub double_sided: bool,
    pub alpha_mode: AlphaMode,
}

#[derive(Debug, Clone, Copy)]
pub enum AlphaMode {
    Opaque,
    Mask,
    Blend,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            albedo: Vector4::new(0.8, 0.8, 0.8, 1.0),
            metallic: 0.0,
            roughness: 0.5,
            emission: Vector3::new(0.0, 0.0, 0.0),
            alpha_cutoff: 0.5,
            albedo_texture: None,
            normal_texture: None,
            metallic_roughness_texture: None,
            emission_texture: None,
            occlusion_texture: None,
            double_sided: false,
            alpha_mode: AlphaMode::Opaque,
        }
    }
}

pub struct Renderer3D<'a> {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'a>,
    surface_config: wgpu::SurfaceConfiguration,
    
    // Render pipelines
    forward_pipeline: wgpu::RenderPipeline,
    shadow_pipeline: wgpu::RenderPipeline,
    skybox_pipeline: wgpu::RenderPipeline,
    post_process_pipeline: wgpu::RenderPipeline,
    
    // Buffers
    camera_buffer: wgpu::Buffer,
    lights_buffer: wgpu::Buffer,
    materials_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    
    // Bind groups
    camera_bind_group: wgpu::BindGroup,
    lights_bind_group: wgpu::BindGroup,
    materials_bind_group: wgpu::BindGroup,
    
    // Textures
    depth_texture: wgpu::Texture,
    depth_texture_view: wgpu::TextureView,
    shadow_maps: Vec<wgpu::Texture>,
    color_attachment: wgpu::Texture,
    color_attachment_view: wgpu::TextureView,
    
    // State
    camera: Camera3D,
    lights: Vec<Light>,
    materials: Vec<Material>,
    meshes: HashMap<u32, Mesh3D>,
    instances: Vec<InstanceData>,
    
    // Rendering settings
    shadow_map_size: u32,
    max_lights: u32,
    msaa_samples: u32,
    enable_shadows: bool,
    enable_post_processing: bool,
    
    // Statistics
    draw_calls: u32,
    vertices_rendered: u32,
    triangles_rendered: u32,
}

pub struct Camera3D {
    pub position: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
    pub fov: f32,
    pub aspect_ratio: f32,
    pub near_plane: f32,
    pub far_plane: f32,
}

impl Default for Camera3D {
    fn default() -> Self {
        Self {
            position: Point3::new(0.0, 2.0, 5.0),
            target: Point3::new(0.0, 0.0, 0.0),
            up: Vector3::new(0.0, 1.0, 0.0),
            fov: 45.0,
            aspect_ratio: 16.0 / 9.0,
            near_plane: 0.1,
            far_plane: 1000.0,
        }
    }
}

impl Camera3D {
    pub fn view_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(self.position, self.target, self.up)
    }
    
    pub fn projection_matrix(&self) -> Matrix4<f32> {
        perspective(Deg(self.fov), self.aspect_ratio, self.near_plane, self.far_plane)
    }
    
    pub fn view_projection_matrix(&self) -> Matrix4<f32> {
        self.projection_matrix() * self.view_matrix()
    }
    
    pub fn forward(&self) -> Vector3<f32> {
        (self.target - self.position).normalize()
    }
    
    pub fn right(&self) -> Vector3<f32> {
        self.forward().cross(self.up).normalize()
    }
}

pub struct Mesh3D {
    pub vertices: Vec<Vertex3D>,
    pub indices: Vec<u32>,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub material_id: u32,
    pub bounding_box: BoundingBox,
}

#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub min: Vector3<f32>,
    pub max: Vector3<f32>,
}

impl BoundingBox {
    pub fn new(min: Vector3<f32>, max: Vector3<f32>) -> Self {
        Self { min, max }
    }
    
    pub fn center(&self) -> Vector3<f32> {
        (self.min + self.max) * 0.5
    }
    
    pub fn size(&self) -> Vector3<f32> {
        self.max - self.min
    }
    
    pub fn contains_point(&self, point: Vector3<f32>) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y &&
        point.z >= self.min.z && point.z <= self.max.z
    }
    
    pub fn intersects(&self, other: &BoundingBox) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x &&
        self.min.y <= other.max.y && self.max.y >= other.min.y &&
        self.min.z <= other.max.z && self.max.z >= other.min.z
    }
}

impl Default for BoundingBox {
    fn default() -> Self {
        Self {
            min: Vector3::new(0.0, 0.0, 0.0),
            max: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}

impl<'a> Renderer3D<'a> {
    pub async fn new(
        window: &'a winit::window::Window,
        // performance_manager: &mut PerformanceManager, // Temporarily disabled
    ) -> RobinResult<Self> {
        // Initialize WGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: Default::default(),
            dx12_shader_compiler: Default::default(),
            gles_minor_version: Default::default(),
        });
        
        let surface = unsafe { instance.create_surface(window) }.unwrap();
        
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
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
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
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);
        
        // Create depth texture
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        
        let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        // Create color attachment for post-processing
        let color_attachment = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Color Attachment"),
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: surface_config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        
        let color_attachment_view = color_attachment.create_view(&wgpu::TextureViewDescriptor::default());
        
        // Create uniform buffers
        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        let lights_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Lights Buffer"),
            size: (std::mem::size_of::<LightUniform>() * 64) as u64, // Max 64 lights
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        let materials_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Materials Buffer"),
            size: (std::mem::size_of::<MaterialUniform>() * 256) as u64, // Max 256 materials
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: (std::mem::size_of::<InstanceData>() * 10000) as u64, // Max 10000 instances
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        // Create shaders and pipelines
        let forward_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Forward Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/forward.wgsl").into()),
        });
        
        let shadow_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shadow Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shadow.wgsl").into()),
        });
        
        // Create bind group layouts
        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera Bind Group Layout"),
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
        
        let lights_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Lights Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        
        let materials_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Materials Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        
        // Create bind groups
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });
        
        let lights_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Lights Bind Group"),
            layout: &lights_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: lights_buffer.as_entire_binding(),
            }],
        });
        
        let materials_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Materials Bind Group"),
            layout: &materials_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: materials_buffer.as_entire_binding(),
            }],
        });
        
        // Create render pipeline
        let forward_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Forward Pipeline Layout"),
            bind_group_layouts: &[
                &camera_bind_group_layout,
                &lights_bind_group_layout,
                &materials_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });
        
        let forward_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Forward Render Pipeline"),
            layout: Some(&forward_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &forward_shader,
                entry_point: "vs_main",
                buffers: &[Vertex3D::desc(), InstanceData::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &forward_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
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
        
        // Initialize default camera
        let camera = Camera3D::default();
        
        // Initialize with default light
        let lights = vec![Light::default()];
        
        // Initialize with default material
        let materials = vec![Material::default()];
        
        // Create shadow pipeline before moving device
        let shadow_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Shadow Render Pipeline"),
                layout: Some(&forward_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &forward_shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex3D::desc(), InstanceData::desc()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &forward_shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: surface_config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
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
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            });
        
        // Create skybox pipeline before moving device
        let skybox_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Skybox Render Pipeline"),
                layout: Some(&forward_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &forward_shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex3D::desc(), InstanceData::desc()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &forward_shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: surface_config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
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
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            });
        
        // Create post-process pipeline before moving device
        let post_process_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Post Process Render Pipeline"),
                layout: Some(&forward_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &forward_shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex3D::desc(), InstanceData::desc()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &forward_shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: surface_config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
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
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            });
        
        Ok(Self {
            device,
            queue,
            surface,
            surface_config: surface_config.clone(),
            forward_pipeline,
            shadow_pipeline,
            skybox_pipeline,
            post_process_pipeline,
            camera_buffer,
            lights_buffer,
            materials_buffer,
            instance_buffer,
            camera_bind_group,
            lights_bind_group,
            materials_bind_group,
            depth_texture,
            depth_texture_view,
            shadow_maps: Vec::new(),
            color_attachment,
            color_attachment_view,
            camera,
            lights,
            materials,
            meshes: HashMap::new(),
            instances: Vec::new(),
            shadow_map_size: 2048,
            max_lights: 64,
            msaa_samples: 1,
            enable_shadows: true,
            enable_post_processing: false,
            draw_calls: 0,
            vertices_rendered: 0,
            triangles_rendered: 0,
        })
    }
    
    pub fn resize(&mut self, width: u32, height: u32) -> RobinResult<()> {
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);
        
        self.camera.aspect_ratio = width as f32 / height as f32;
        
        // Recreate depth texture
        self.depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        
        self.depth_texture_view = self.depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        Ok(())
    }
    
    pub fn render(&mut self, /* performance_manager: &mut PerformanceManager */ ) -> RobinResult<()> {
        let start_time = std::time::Instant::now();
        
        // Reset per-frame statistics
        self.draw_calls = 0;
        self.vertices_rendered = 0;
        self.triangles_rendered = 0;
        
        // Update camera uniform
        self.update_camera_uniform()?;
        
        // Update lights uniform
        self.update_lights_uniform()?;
        
        // Update materials uniform
        self.update_materials_uniform()?;
        
        // Get surface texture
        let output = self.surface.get_current_texture().unwrap();
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        
        // Shadow pass (if enabled)
        if self.enable_shadows {
            self.render_shadow_maps(&mut encoder)?;
        }
        
        // Update instance buffer before render pass
        if !self.instances.is_empty() {
            self.queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&self.instances));
        }
        
        // Forward pass
        let render_stats = {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Forward Render Pass"),
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
                    view: &self.depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            
            render_pass.set_pipeline(&self.forward_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.lights_bind_group, &[]);
            render_pass.set_bind_group(2, &self.materials_bind_group, &[]);
            
            // Render all meshes (now doesn't need mutable self)
            self.render_meshes(&mut render_pass)
        };
        
        // Update statistics after render pass ends
        self.draw_calls += render_stats.0;
        self.vertices_rendered += render_stats.1;
        self.triangles_rendered += render_stats.2;
        
        // Post-processing pass (if enabled)
        if self.enable_post_processing {
            self.render_post_processing(&mut encoder)?;
        }
        
        // Submit commands
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        
        // Update performance metrics
        let render_duration = start_time.elapsed();
        // Performance metrics recording temporarily disabled
        // TODO: Re-enable when PerformanceManager is implemented
        
        Ok(())
    }
    
    pub fn add_mesh(&mut self, mesh_id: u32, vertices: Vec<Vertex3D>, indices: Vec<u32>, material_id: u32) -> RobinResult<()> {
        // Calculate bounding box
        let mut min = Vector3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max = Vector3::new(f32::MIN, f32::MIN, f32::MIN);
        
        for vertex in &vertices {
            let pos = Vector3::new(vertex.position[0], vertex.position[1], vertex.position[2]);
            min.x = min.x.min(pos.x);
            min.y = min.y.min(pos.y);
            min.z = min.z.min(pos.z);
            max.x = max.x.max(pos.x);
            max.y = max.y.max(pos.y);
            max.z = max.z.max(pos.z);
        }
        
        let bounding_box = BoundingBox::new(min, max);
        
        // Create GPU buffers
        let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        
        let mesh = Mesh3D {
            vertices,
            indices,
            vertex_buffer,
            index_buffer,
            material_id,
            bounding_box,
        };
        
        self.meshes.insert(mesh_id, mesh);
        Ok(())
    }
    
    pub fn add_instance(&mut self, mesh_id: u32, transform: Matrix4<f32>) -> RobinResult<()> {
        let normal_matrix = transform.invert().unwrap().transpose();
        
        let instance = InstanceData {
            model_matrix: transform.into(),
            normal_matrix: normal_matrix.into(),
            material_id: self.meshes.get(&mesh_id).map(|m| m.material_id).unwrap_or(0),
            _padding: [0; 3],
        };
        
        self.instances.push(instance);
        Ok(())
    }
    
    pub fn clear_instances(&mut self) {
        self.instances.clear();
    }
    
    pub fn add_light(&mut self, light: Light) -> RobinResult<()> {
        if self.lights.len() < self.max_lights as usize {
            self.lights.push(light);
        }
        Ok(())
    }
    
    pub fn add_material(&mut self, material: Material) -> RobinResult<u32> {
        let material_id = self.materials.len() as u32;
        self.materials.push(material);
        Ok(material_id)
    }
    
    pub fn set_camera(&mut self, camera: Camera3D) {
        self.camera = camera;
    }
    
    pub fn get_camera(&self) -> &Camera3D {
        &self.camera
    }
    
    pub fn get_camera_mut(&mut self) -> &mut Camera3D {
        &mut self.camera
    }
    
    // Private methods
    
    fn update_camera_uniform(&self) -> RobinResult<()> {
        let view_matrix = self.camera.view_matrix();
        let projection_matrix = self.camera.projection_matrix();
        let view_projection_matrix = projection_matrix * view_matrix;
        
        let camera_uniform = CameraUniform {
            view_matrix: view_matrix.into(),
            projection_matrix: projection_matrix.into(),
            view_projection_matrix: view_projection_matrix.into(),
            camera_position: [self.camera.position.x, self.camera.position.y, self.camera.position.z, 1.0],
            camera_direction: {
                let dir = self.camera.forward();
                [dir.x, dir.y, dir.z, 0.0]
            },
            near_plane: self.camera.near_plane,
            far_plane: self.camera.far_plane,
            fov: self.camera.fov,
            aspect_ratio: self.camera.aspect_ratio,
        };
        
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[camera_uniform]));
        Ok(())
    }
    
    fn update_lights_uniform(&self) -> RobinResult<()> {
        let mut light_uniforms = Vec::new();
        
        for light in &self.lights {
            if light.enabled {
                let light_uniform = LightUniform {
                    light_type: light.light_type as u32,
                    position: light.position.into(),
                    direction: light.direction.into(),
                    color: light.color.into(),
                    intensity: light.intensity,
                    range: light.range,
                    inner_cone: light.inner_cone,
                    outer_cone: light.outer_cone,
                    _padding: 0,
                };
                light_uniforms.push(light_uniform);
            }
        }
        
        // Pad to max lights
        light_uniforms.resize(self.max_lights as usize, LightUniform {
            light_type: 0,
            position: [0.0; 3],
            direction: [0.0; 3],
            color: [0.0; 3],
            intensity: 0.0,
            range: 0.0,
            inner_cone: 0.0,
            outer_cone: 0.0,
            _padding: 0,
        });
        
        self.queue.write_buffer(&self.lights_buffer, 0, bytemuck::cast_slice(&light_uniforms));
        Ok(())
    }
    
    fn update_materials_uniform(&self) -> RobinResult<()> {
        let mut material_uniforms = Vec::new();
        
        for material in &self.materials {
            let texture_flags = 
                (if material.albedo_texture.is_some() { 1 } else { 0 }) |
                (if material.normal_texture.is_some() { 2 } else { 0 }) |
                (if material.metallic_roughness_texture.is_some() { 4 } else { 0 }) |
                (if material.emission_texture.is_some() { 8 } else { 0 }) |
                (if material.occlusion_texture.is_some() { 16 } else { 0 });
            
            let material_uniform = MaterialUniform {
                albedo: material.albedo.into(),
                metallic: material.metallic,
                roughness: material.roughness,
                emission: material.emission.into(),
                alpha_cutoff: material.alpha_cutoff,
                texture_flags,
                _padding: [0; 2],
            };
            material_uniforms.push(material_uniform);
        }
        
        // Pad to max materials
        material_uniforms.resize(256, MaterialUniform {
            albedo: [1.0; 4],
            metallic: 0.0,
            roughness: 0.5,
            emission: [0.0; 3],
            alpha_cutoff: 0.5,
            texture_flags: 0,
            _padding: [0; 2],
        });
        
        self.queue.write_buffer(&self.materials_buffer, 0, bytemuck::cast_slice(&material_uniforms));
        Ok(())
    }
    
    fn render_meshes<'rp>(&'rp self, render_pass: &mut wgpu::RenderPass<'rp>) -> (u32, u32, u32) 
    where 'a: 'rp {
        // Instance buffer already updated before render pass
        let instance_buffer_slice = self.instance_buffer.slice(..);
        render_pass.set_vertex_buffer(1, instance_buffer_slice);
        
        let mut draw_calls = 0;
        let mut vertices_rendered = 0;
        let mut triangles_rendered = 0;
        
        // Group instances by mesh for efficient rendering
        let mut mesh_instances: HashMap<u32, Vec<usize>> = HashMap::new();
        for (i, instance) in self.instances.iter().enumerate() {
            // Find mesh by material ID (simplified)
            for (mesh_id, mesh) in &self.meshes {
                if mesh.material_id == instance.material_id {
                    mesh_instances.entry(*mesh_id).or_default().push(i);
                    break;
                }
            }
        }
        
        // Render each mesh with its instances
        for (mesh_id, instance_indices) in mesh_instances {
            if let Some(mesh) = self.meshes.get(&mesh_id) {
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                
                // Render instances in batches
                let batch_size = 1000;
                for batch in instance_indices.chunks(batch_size) {
                    let start_instance = batch[0] as u32;
                    let instance_count = batch.len() as u32;
                    
                    render_pass.draw_indexed(
                        0..mesh.indices.len() as u32,
                        0,
                        start_instance..start_instance + instance_count,
                    );
                    
                    // Update statistics
                    draw_calls += 1;
                    vertices_rendered += mesh.vertices.len() as u32 * instance_count;
                    triangles_rendered += (mesh.indices.len() / 3) as u32 * instance_count;
                }
            }
        }
        
        (draw_calls, vertices_rendered, triangles_rendered)
    }
    
    fn render_shadow_maps(&mut self, _encoder: &mut wgpu::CommandEncoder) -> RobinResult<()> {
        // Shadow mapping implementation would go here
        // This is a placeholder
        Ok(())
    }
    
    fn render_post_processing(&mut self, _encoder: &mut wgpu::CommandEncoder) -> RobinResult<()> {
        // Post-processing implementation would go here
        // This is a placeholder
        Ok(())
    }
}