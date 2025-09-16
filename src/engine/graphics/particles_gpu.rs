use crate::engine::error::RobinResult;
use cgmath::{Vector3, Vector4, Point3};
use wgpu::util::DeviceExt;
use serde::{Serialize, Deserialize};
use std::{mem, sync::Arc};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(C)]
pub struct GPUParticle {
    pub position: [f32; 4],        // xyz + life
    pub velocity: [f32; 4],        // xyz + mass
    pub color: [f32; 4],           // rgba
    pub size_rotation: [f32; 4],   // size, rotation, angular_vel, padding
}

unsafe impl bytemuck::Pod for GPUParticle {}
unsafe impl bytemuck::Zeroable for GPUParticle {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GPUParticleEmitterConfig {
    pub max_particles: u32,
    pub spawn_rate: f32,
    pub lifetime: f32,
    pub lifetime_variance: f32,
    pub initial_speed: f32,
    pub initial_speed_variance: f32,
    pub size: f32,
    pub size_variance: f32,
    pub color_start: [f32; 4],
    pub color_end: [f32; 4],
    pub gravity: Vector3<f32>,
    pub emission_shape: EmissionShape,
    pub simulation_space: SimulationSpace,
    pub texture_atlas_size: u32,
}

impl Default for GPUParticleEmitterConfig {
    fn default() -> Self {
        Self {
            max_particles: 1000,
            spawn_rate: 50.0,
            lifetime: 3.0,
            lifetime_variance: 0.5,
            initial_speed: 5.0,
            initial_speed_variance: 2.0,
            size: 1.0,
            size_variance: 0.2,
            color_start: [1.0, 1.0, 1.0, 1.0],
            color_end: [1.0, 1.0, 1.0, 0.0],
            gravity: Vector3::new(0.0, -9.81, 0.0),
            emission_shape: EmissionShape::Point,
            simulation_space: SimulationSpace::World,
            texture_atlas_size: 4,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmissionShape {
    Point,
    Sphere { radius: f32 },
    Box { size: Vector3<f32> },
    Cone { radius: f32, angle: f32 },
    Circle { radius: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimulationSpace {
    World,
    Local,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ParticleSystemUniforms {
    pub emitter_position: [f32; 4],
    pub emitter_direction: [f32; 4],
    pub gravity: [f32; 4],
    pub delta_time: f32,
    pub spawn_rate: f32,
    pub max_particles: u32,
    pub current_time: f32,
    pub color_start: [f32; 4],
    pub color_end: [f32; 4],
    pub size_curve: [f32; 4],      // x=start, y=end, z=curve_power, w=padding
    pub emission_params: [f32; 4],  // shape-specific parameters
}

unsafe impl bytemuck::Pod for ParticleSystemUniforms {}
unsafe impl bytemuck::Zeroable for ParticleSystemUniforms {}

pub struct GPUParticleSystem {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    
    // Compute pipeline for particle simulation
    compute_pipeline: wgpu::ComputePipeline,
    
    // Render pipeline for particle rendering
    render_pipeline: wgpu::RenderPipeline,
    
    // Buffers
    particle_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    indirect_buffer: wgpu::Buffer,
    counter_buffer: wgpu::Buffer,
    
    // Bind groups
    compute_bind_group: wgpu::BindGroup,
    render_bind_group: wgpu::BindGroup,
    
    // Configuration
    config: GPUParticleEmitterConfig,
    
    // State
    current_time: f32,
    last_spawn_time: f32,
    active_particles: u32,
}

impl GPUParticleSystem {
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        surface_format: wgpu::TextureFormat,
        config: GPUParticleEmitterConfig,
    ) -> RobinResult<Self> {
        let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Particle Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/particle_compute.wgsl").into()),
        });

        let render_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Particle Render Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/particle_render.wgsl").into()),
        });

        // Create buffers
        let particle_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Particle Buffer"),
            size: (config.max_particles as u64) * mem::size_of::<GPUParticle>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Particle System Uniforms"),
            size: mem::size_of::<ParticleSystemUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let indirect_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Particle Indirect Buffer"),
            size: 16,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::INDIRECT,
            mapped_at_creation: false,
        });

        let counter_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Particle Counter Buffer"),
            size: 4,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create compute pipeline
        let compute_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Particle Compute Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Particle Compute Pipeline Layout"),
            bind_group_layouts: &[&compute_bind_group_layout],
            push_constant_ranges: &[],
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Particle Compute Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: "cs_main",
            compilation_options: Default::default(),
        });

        // Create render pipeline
        let render_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Particle Render Bind Group Layout"),
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
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Particle Render Pipeline Layout"),
            bind_group_layouts: &[&render_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Particle Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &render_shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: mem::size_of::<GPUParticle>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                            shader_location: 1,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: (mem::size_of::<[f32; 4]>() * 2) as wgpu::BufferAddress,
                            shader_location: 2,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: (mem::size_of::<[f32; 4]>() * 3) as wgpu::BufferAddress,
                            shader_location: 3,
                        },
                    ],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &render_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // Create bind groups (will need texture and sampler for render bind group)
        let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Particle Compute Bind Group"),
            layout: &compute_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: particle_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: indirect_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: counter_buffer.as_entire_binding(),
                },
            ],
        });

        // Create default texture and sampler for rendering (placeholder)
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Default Particle Texture"),
            size: wgpu::Extent3d {
                width: 64,
                height: 64,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Particle Texture Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Particle Render Bind Group"),
            layout: &render_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Ok(Self {
            device,
            queue,
            compute_pipeline,
            render_pipeline,
            particle_buffer,
            uniform_buffer,
            indirect_buffer,
            counter_buffer,
            compute_bind_group,
            render_bind_group,
            config,
            current_time: 0.0,
            last_spawn_time: 0.0,
            active_particles: 0,
        })
    }

    pub fn update(&mut self, delta_time: f32, emitter_position: Point3<f32>, emitter_direction: Vector3<f32>) {
        self.current_time += delta_time;
        
        let uniforms = ParticleSystemUniforms {
            emitter_position: [emitter_position.x, emitter_position.y, emitter_position.z, 1.0],
            emitter_direction: [emitter_direction.x, emitter_direction.y, emitter_direction.z, 0.0],
            gravity: [self.config.gravity.x, self.config.gravity.y, self.config.gravity.z, 0.0],
            delta_time,
            spawn_rate: self.config.spawn_rate,
            max_particles: self.config.max_particles,
            current_time: self.current_time,
            color_start: self.config.color_start,
            color_end: self.config.color_end,
            size_curve: [self.config.size, self.config.size * 0.1, 2.0, 0.0],
            emission_params: match &self.config.emission_shape {
                EmissionShape::Point => [0.0, 0.0, 0.0, 0.0],
                EmissionShape::Sphere { radius } => [*radius, 0.0, 0.0, 1.0],
                EmissionShape::Box { size } => [size.x, size.y, size.z, 2.0],
                EmissionShape::Cone { radius, angle } => [*radius, *angle, 0.0, 3.0],
                EmissionShape::Circle { radius } => [*radius, 0.0, 0.0, 4.0],
            },
        };

        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }

    pub fn dispatch_compute(&self, encoder: &mut wgpu::CommandEncoder) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Particle Simulation Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&self.compute_pipeline);
        compute_pass.set_bind_group(0, &self.compute_bind_group, &[]);
        
        let workgroup_size = 64;
        let num_workgroups = (self.config.max_particles + workgroup_size - 1) / workgroup_size;
        compute_pass.dispatch_workgroups(num_workgroups, 1, 1);
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, camera_uniform_bind_group: &'a wgpu::BindGroup) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, camera_uniform_bind_group, &[]);
        render_pass.set_bind_group(1, &self.render_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.particle_buffer.slice(..));
        render_pass.draw_indirect(&self.indirect_buffer, 0);
    }

    pub fn set_config(&mut self, config: GPUParticleEmitterConfig) {
        if config.max_particles != self.config.max_particles {
            self.recreate_buffers(config.max_particles);
        }
        self.config = config;
    }

    fn recreate_buffers(&mut self, max_particles: u32) {
        self.particle_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Particle Buffer"),
            size: (max_particles as u64) * mem::size_of::<GPUParticle>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        // Recreate compute bind group with new particle buffer
        let compute_bind_group_layout = self.compute_pipeline.get_bind_group_layout(0);
        self.compute_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Particle Compute Bind Group"),
            layout: &compute_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.particle_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.indirect_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.counter_buffer.as_entire_binding(),
                },
            ],
        });
    }

    pub fn reset(&mut self) {
        self.current_time = 0.0;
        self.last_spawn_time = 0.0;
        self.active_particles = 0;
        
        // Clear counter buffer
        self.queue.write_buffer(&self.counter_buffer, 0, &[0u8; 4]);
        
        // Clear indirect draw buffer
        let indirect_data: [u32; 4] = [4, 0, 0, 0]; // vertex_count, instance_count, first_vertex, first_instance
        self.queue.write_buffer(&self.indirect_buffer, 0, bytemuck::cast_slice(&indirect_data));
    }

    pub fn get_active_particle_count(&self) -> u32 {
        self.active_particles
    }

    pub fn get_max_particles(&self) -> u32 {
        self.config.max_particles
    }
}

pub struct GPUParticleSystemManager {
    systems: Vec<GPUParticleSystem>,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    surface_format: wgpu::TextureFormat,
}

impl GPUParticleSystemManager {
    pub fn new(device: wgpu::Device, queue: wgpu::Queue, surface_format: wgpu::TextureFormat) -> Self {
        Self {
            systems: Vec::new(),
            device: Arc::new(device),
            queue: Arc::new(queue),
            surface_format,
        }
    }

    pub fn create_system(&mut self, config: GPUParticleEmitterConfig) -> RobinResult<usize> {
        let system = GPUParticleSystem::new(
            Arc::clone(&self.device),
            Arc::clone(&self.queue),
            self.surface_format,
            config,
        )?;
        
        let id = self.systems.len();
        self.systems.push(system);
        Ok(id)
    }

    pub fn update_system(&mut self, id: usize, delta_time: f32, position: Point3<f32>, direction: Vector3<f32>) -> RobinResult<()> {
        if let Some(system) = self.systems.get_mut(id) {
            system.update(delta_time, position, direction);
        }
        Ok(())
    }

    pub fn update_all(&mut self, delta_time: f32) {
        for system in &mut self.systems {
            // Update with default position and direction - systems can override
            system.update(delta_time, Point3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 0.0));
        }
    }

    pub fn dispatch_all_compute(&self, encoder: &mut wgpu::CommandEncoder) {
        for system in &self.systems {
            system.dispatch_compute(encoder);
        }
    }

    pub fn render_all<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, camera_bind_group: &'a wgpu::BindGroup) {
        for system in &self.systems {
            system.render(render_pass, camera_bind_group);
        }
    }

    pub fn get_system(&mut self, id: usize) -> Option<&mut GPUParticleSystem> {
        self.systems.get_mut(id)
    }

    pub fn remove_system(&mut self, id: usize) {
        if id < self.systems.len() {
            self.systems.remove(id);
        }
    }

    pub fn clear_all(&mut self) {
        self.systems.clear();
    }

    pub fn get_total_particles(&self) -> u32 {
        self.systems.iter().map(|s| s.get_active_particle_count()).sum()
    }

    pub fn get_system_count(&self) -> usize {
        self.systems.len()
    }
}

#[derive(Debug)]
pub struct GPUParticleStats {
    pub total_systems: usize,
    pub total_particles: u32,
    pub max_particles: u32,
    pub average_particles_per_system: f32,
    pub compute_time_ms: f32,
    pub render_time_ms: f32,
}

impl GPUParticleStats {
    pub fn collect(manager: &GPUParticleSystemManager) -> Self {
        let total_systems = manager.get_system_count();
        let total_particles = manager.get_total_particles();
        let max_particles = manager.systems.iter().map(|s| s.get_max_particles()).sum();
        let average_particles_per_system = if total_systems > 0 {
            total_particles as f32 / total_systems as f32
        } else {
            0.0
        };

        Self {
            total_systems,
            total_particles,
            max_particles,
            average_particles_per_system,
            compute_time_ms: 0.0,
            render_time_ms: 0.0,
        }
    }
}