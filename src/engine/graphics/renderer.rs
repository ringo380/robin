use wgpu::util::DeviceExt;
use winit::window::Window;
use cgmath::SquareMatrix;
use std::collections::HashMap;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
    color: [f32; 4],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x2, 
        2 => Float32x4,
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
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteInstance {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub rotation: f32,
    pub color: [f32; 4],
    pub uv_coords: [f32; 8], // UV coordinates for each vertex [u0,v0, u1,v1, u2,v2, u3,v3]
}

impl SpriteInstance {
    pub fn new() -> Self {
        Self {
            position: [0.0, 0.0],
            size: [32.0, 32.0],
            rotation: 0.0,
            color: [1.0, 1.0, 1.0, 1.0],
            uv_coords: [0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0], // Default full texture
        }
    }

    pub fn from_sprite(sprite: &crate::engine::graphics::sprite::Sprite, position: [f32; 2]) -> Self {
        Self {
            position,
            size: [sprite.size.x, sprite.size.y],
            rotation: 0.0,
            color: sprite.color,
            uv_coords: sprite.get_uv_coords(),
        }
    }

    const ATTRIBS: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
        5 => Float32x2,  // position
        6 => Float32x2,  // size  
        7 => Float32,    // rotation
        8 => Float32x4,  // color
        // UV coords will be handled in shader with storage buffer
    ];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SpriteInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub struct RenderBatch {
    pub texture_name: String,
    pub instances: Vec<SpriteInstance>,
    pub instance_buffer: Option<wgpu::Buffer>,
    pub needs_update: bool,
}

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    
    // Rendering pipeline
    render_pipeline: wgpu::RenderPipeline,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    
    // Batch rendering
    batches: HashMap<String, RenderBatch>,
    quad_vertices: wgpu::Buffer,
    quad_indices: wgpu::Buffer,
    
    // Texture system
    texture_manager: crate::engine::graphics::texture::TextureManager,
    
    // Lighting system
    light_buffer: wgpu::Buffer,
    lights: Vec<Light>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Light {
    pub position: [f32; 2],
    pub color: [f32; 3],
    pub intensity: f32,
    pub radius: f32,
    pub _padding: [f32; 3],
}

const MAX_LIGHTS: usize = 64;

impl Renderer {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        let surface = unsafe { instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::from_window(window).unwrap()).unwrap() };
        
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
            },
            None,
        ).await.unwrap();
        
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
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
        
        // Create camera uniform buffer
        let camera_uniform = CameraUniform {
            view_proj: cgmath::Matrix4::identity().into(),
        };
        
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        
        // Create lighting buffer
        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Buffer"),
            contents: bytemuck::cast_slice(&vec![Light::default(); MAX_LIGHTS]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        
        // Initialize texture manager
        let mut texture_manager = crate::engine::graphics::texture::TextureManager::new(&device);
        texture_manager.create_default_textures(&device, &queue);

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // Camera uniform
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
                // Lights uniform
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("Main Bind Group Layout"),
        });
        
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: light_buffer.as_entire_binding(),
                },
            ],
            label: Some("Camera Bind Group"),
        });
        
        // Light bind group is same as camera bind group for this setup
        
        // Create shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Main Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/sprite.wgsl").into()),
        });
        
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout, texture_manager.get_bind_group_layout()],
            push_constant_ranges: &[],
        });
        
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc(), SpriteInstance::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
        
        // Create quad geometry for sprite rendering
        let quad_vertices = [
            Vertex { position: [-0.5, -0.5, 0.0], tex_coords: [0.0, 1.0], color: [1.0, 1.0, 1.0, 1.0] },
            Vertex { position: [ 0.5, -0.5, 0.0], tex_coords: [1.0, 1.0], color: [1.0, 1.0, 1.0, 1.0] },
            Vertex { position: [ 0.5,  0.5, 0.0], tex_coords: [1.0, 0.0], color: [1.0, 1.0, 1.0, 1.0] },
            Vertex { position: [-0.5,  0.5, 0.0], tex_coords: [0.0, 0.0], color: [1.0, 1.0, 1.0, 1.0] },
        ];
        
        let quad_indices = [0u16, 1, 2, 0, 2, 3];
        
        let quad_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Vertex Buffer"),
            contents: bytemuck::cast_slice(&quad_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        let quad_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Index Buffer"),
            contents: bytemuck::cast_slice(&quad_indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        
        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            camera_buffer,
            camera_bind_group,
            light_buffer,
            batches: HashMap::new(),
            quad_vertices: quad_vertex_buffer,
            quad_indices: quad_index_buffer,
            texture_manager,
            lights: Vec::new(),
        }
    }
    
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
    
    pub fn add_light(&mut self, light: Light) {
        if self.lights.len() < MAX_LIGHTS {
            self.lights.push(light);
        }
    }
    
    pub fn clear_lights(&mut self) {
        self.lights.clear();
    }
    
    pub fn update_camera(&self, view_proj_matrix: cgmath::Matrix4<f32>) {
        let camera_uniform = CameraUniform {
            view_proj: view_proj_matrix.into(),
        };
        
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[camera_uniform]));
    }

    pub fn add_sprite(&mut self, sprite: &crate::engine::graphics::sprite::Sprite, position: [f32; 2]) {
        let instance = SpriteInstance::from_sprite(sprite, position);
        self.add_sprite_instance(&sprite.texture_name, instance);
    }

    pub fn add_sprite_instance(&mut self, texture_name: &str, instance: SpriteInstance) {
        let batch = self.batches
            .entry(texture_name.to_string())
            .or_insert_with(|| RenderBatch {
                texture_name: texture_name.to_string(),
                instances: Vec::new(),
                instance_buffer: None,
                needs_update: false,
            });
        
        batch.instances.push(instance);
        batch.needs_update = true;
    }

    pub fn clear_sprites(&mut self) {
        for batch in self.batches.values_mut() {
            batch.instances.clear();
            batch.needs_update = true;
        }
    }

    pub fn load_texture<P: AsRef<std::path::Path>>(&mut self, name: &str, path: P) -> Result<(), Box<dyn std::error::Error>> {
        self.texture_manager.load_texture(&self.device, &self.queue, name, path)
    }

    pub fn create_solid_texture(&mut self, name: &str, color: [u8; 4], size: (u32, u32)) {
        self.texture_manager.create_solid_texture(&self.device, &self.queue, name, color, size);
    }

    fn update_sprite_buffers(&mut self) {
        for batch in self.batches.values_mut() {
            if batch.needs_update && !batch.instances.is_empty() {
                // Create or update instance buffer
                let buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{} Instance Buffer", batch.texture_name)),
                    contents: bytemuck::cast_slice(&batch.instances),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                });
                
                batch.instance_buffer = Some(buffer);
                batch.needs_update = false;
            }
        }
    }
    
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // Update sprite buffers if needed
        self.update_sprite_buffers();
        
        // Update lights buffer
        let mut light_data = vec![Light::default(); MAX_LIGHTS];
        for (i, light) in self.lights.iter().enumerate() {
            if i < MAX_LIGHTS {
                light_data[i] = *light;
            }
        }
        self.queue.write_buffer(&self.light_buffer, 0, bytemuck::cast_slice(&light_data));
        
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
                            r: 0.02,
                            g: 0.02,
                            b: 0.04,
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
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.quad_vertices.slice(..));
            render_pass.set_index_buffer(self.quad_indices.slice(..), wgpu::IndexFormat::Uint16);
            
            // Render all sprite batches
            for batch in self.batches.values() {
                if batch.instances.is_empty() {
                    continue;
                }
                
                if let Some(instance_buffer) = &batch.instance_buffer {
                    // Set texture bind group
                    if let Some(texture_bind_group) = self.texture_manager.get_bind_group(&batch.texture_name) {
                        render_pass.set_bind_group(1, texture_bind_group, &[]);
                        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                        render_pass.draw_indexed(0..6, 0, 0..batch.instances.len() as u32);
                    } else {
                        // Fallback to white texture if texture not found
                        if let Some(white_bind_group) = self.texture_manager.get_bind_group("white") {
                            render_pass.set_bind_group(1, white_bind_group, &[]);
                            render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                            render_pass.draw_indexed(0..6, 0, 0..batch.instances.len() as u32);
                        }
                    }
                }
            }
        }
        
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        
        Ok(())
    }
}

impl Default for Light {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0],
            color: [1.0, 1.0, 1.0],
            intensity: 1.0,
            radius: 100.0,
            _padding: [0.0; 3],
        }
    }
}