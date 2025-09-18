use std::sync::Arc;
use wgpu::util::DeviceExt;
use cgmath::{Matrix4, Vector3, Vector4, Point3, perspective, ortho, look_at};
use crate::engine::error::{RobinResult, RobinError};

#[derive(Debug, Clone)]
pub struct ShadowConfig {
    pub cascade_count: u32,
    pub shadow_map_size: u32,
    pub cascade_splits: Vec<f32>,
    pub depth_bias: f32,
    pub normal_bias: f32,
    pub pcf_radius: f32,
}

impl Default for ShadowConfig {
    fn default() -> Self {
        Self {
            cascade_count: 4,
            shadow_map_size: 2048,
            cascade_splits: vec![0.1, 10.0, 50.0, 200.0, 1000.0],
            depth_bias: 0.005,
            normal_bias: 0.01,
            pcf_radius: 1.5,
        }
    }
}

#[derive(Debug)]
pub struct ShadowMetrics {
    pub shadow_map_updates: u32,
    pub shadows_rendered: u32,
    pub cascade_switches: u32,
    pub pcf_samples: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct ShadowUniforms {
    light_view_proj_matrices: [Matrix4<f32>; 4],
    cascade_splits: [f32; 5],
    light_direction: Vector3<f32>,
    _padding1: f32,
    depth_bias: f32,
    normal_bias: f32,
    pcf_radius: f32,
    cascade_count: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CascadeData {
    view_proj_matrix: Matrix4<f32>,
    far_plane: f32,
    _padding: [f32; 3],
}

pub struct CascadedShadowMaps {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,

    // Shadow mapping resources
    shadow_depth_texture: wgpu::Texture,
    shadow_depth_view: wgpu::TextureView,
    shadow_sampler: wgpu::Sampler,

    // Cascade computation
    cascade_compute_pipeline: wgpu::ComputePipeline,
    shadow_render_pipeline: wgpu::RenderPipeline,

    // Uniforms and buffers
    shadow_uniform_buffer: wgpu::Buffer,
    cascade_buffer: wgpu::Buffer,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,

    // Configuration
    config: ShadowConfig,
    metrics: ShadowMetrics,
}

impl CascadedShadowMaps {
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        config: ShadowConfig
    ) -> RobinResult<Self> {
        // Create shadow depth texture array for cascades
        let shadow_depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Cascaded Shadow Map"),
            size: wgpu::Extent3d {
                width: config.shadow_map_size,
                height: config.shadow_map_size,
                depth_or_array_layers: config.cascade_count,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let shadow_depth_view = shadow_depth_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Cascaded Shadow Map View"),
            format: Some(wgpu::TextureFormat::Depth32Float),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            aspect: wgpu::TextureAspect::DepthOnly,
            base_mip_level: 0,
            mip_level_count: Some(1),
            base_array_layer: 0,
            array_layer_count: Some(config.cascade_count),
        });

        // Create shadow sampler with PCF
        let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Shadow Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        // Create uniform buffers
        let shadow_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shadow Uniform Buffer"),
            size: std::mem::size_of::<ShadowUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let cascade_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Cascade Data Buffer"),
            size: (std::mem::size_of::<CascadeData>() * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Shadow Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                    count: None,
                },
            ],
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Shadow Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: shadow_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: cascade_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&shadow_depth_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&shadow_sampler),
                },
            ],
        });

        // Create compute pipeline for cascade calculation
        let cascade_compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Cascade Computation Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/cascade_computation.wgsl").into()),
        });

        let cascade_compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Cascade Computation Pipeline"),
            layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Cascade Computation Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            })),
            module: &cascade_compute_shader,
            entry_point: "cs_main",
        });

        // Create shadow rendering pipeline
        let shadow_vertex_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shadow Vertex Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shadow_depth.wgsl").into()),
        });

        let shadow_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Shadow Render Pipeline"),
            layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Shadow Render Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            })),
            vertex: wgpu::VertexState {
                module: &shadow_vertex_shader,
                entry_point: "vs_main",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<[f32; 3]>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 0,
                                format: wgpu::VertexFormat::Float32x3,
                            },
                        ],
                    }
                ],
            },
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
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState {
                    constant: 2,
                    slope_scale: 2.0,
                    clamp: 0.0,
                },
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: None, // Depth-only pass
            multiview: None,
        });

        Ok(Self {
            device,
            queue,
            shadow_depth_texture,
            shadow_depth_view,
            shadow_sampler,
            cascade_compute_pipeline,
            shadow_render_pipeline,
            shadow_uniform_buffer,
            cascade_buffer,
            bind_group_layout,
            bind_group,
            config,
            metrics: ShadowMetrics {
                shadow_map_updates: 0,
                shadows_rendered: 0,
                cascade_switches: 0,
                pcf_samples: 0,
            },
        })
    }

    pub fn update_cascades(
        &mut self,
        camera_pos: Point3<f32>,
        camera_dir: Vector3<f32>,
        camera_up: Vector3<f32>,
        camera_fov: f32,
        camera_aspect: f32,
        light_direction: Vector3<f32>,
    ) -> RobinResult<()> {
        let mut cascade_matrices = [Matrix4::from_scale(1.0); 4];
        let mut cascade_data = Vec::new();

        // Calculate cascade splits based on logarithmic distribution
        for i in 0..self.config.cascade_count as usize {
            let near = if i == 0 { 0.1 } else { self.config.cascade_splits[i] };
            let far = self.config.cascade_splits[i + 1];

            // Create frustum for this cascade
            let proj_matrix = perspective(
                cgmath::Deg(camera_fov),
                camera_aspect,
                near,
                far
            );

            let view_matrix = look_at(camera_pos, camera_pos + camera_dir, camera_up);
            let view_proj = proj_matrix * view_matrix;

            // Calculate frustum corners in world space
            let inv_view_proj = view_proj.invert().unwrap();
            let mut frustum_corners = Vec::new();

            for z in &[-1.0, 1.0] {
                for y in &[-1.0, 1.0] {
                    for x in &[-1.0, 1.0] {
                        let corner = inv_view_proj * Vector4::new(*x, *y, *z, 1.0);
                        frustum_corners.push(Point3::new(
                            corner.x / corner.w,
                            corner.y / corner.w,
                            corner.z / corner.w,
                        ));
                    }
                }
            }

            // Calculate frustum center
            let frustum_center = frustum_corners.iter().fold(Point3::new(0.0, 0.0, 0.0), |acc, corner| {
                Point3::new(
                    acc.x + corner.x / 8.0,
                    acc.y + corner.y / 8.0,
                    acc.z + corner.z / 8.0,
                )
            });

            // Calculate bounding sphere radius
            let radius = frustum_corners.iter().map(|corner| {
                (corner - frustum_center).magnitude()
            }).fold(0.0f32, f32::max);

            // Create orthographic projection for shadow map
            let light_view = look_at(
                frustum_center - light_direction * radius,
                frustum_center,
                Vector3::new(0.0, 1.0, 0.0),
            );

            let light_proj = ortho(-radius, radius, -radius, radius, 0.1, radius * 2.0);
            cascade_matrices[i] = light_proj * light_view;

            cascade_data.push(CascadeData {
                view_proj_matrix: cascade_matrices[i],
                far_plane: far,
                _padding: [0.0; 3],
            });
        }

        // Update uniform buffer
        let shadow_uniforms = ShadowUniforms {
            light_view_proj_matrices: cascade_matrices,
            cascade_splits: [
                self.config.cascade_splits[0],
                self.config.cascade_splits[1],
                self.config.cascade_splits[2],
                self.config.cascade_splits[3],
                self.config.cascade_splits[4],
            ],
            light_direction,
            _padding1: 0.0,
            depth_bias: self.config.depth_bias,
            normal_bias: self.config.normal_bias,
            pcf_radius: self.config.pcf_radius,
            cascade_count: self.config.cascade_count,
        };

        self.queue.write_buffer(
            &self.shadow_uniform_buffer,
            0,
            bytemuck::cast_slice(&[shadow_uniforms]),
        );

        self.queue.write_buffer(
            &self.cascade_buffer,
            0,
            bytemuck::cast_slice(&cascade_data),
        );

        self.metrics.shadow_map_updates += 1;
        Ok(())
    }

    pub fn render_shadows<F>(&mut self, mut render_scene: F) -> RobinResult<()>
    where
        F: FnMut(&wgpu::RenderPass, u32),
    {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Shadow Map Encoder"),
        });

        // Render each cascade
        for cascade in 0..self.config.cascade_count {
            let shadow_view = self.shadow_depth_texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some(&format!("Shadow Map Cascade {}", cascade)),
                format: Some(wgpu::TextureFormat::Depth32Float),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::DepthOnly,
                base_mip_level: 0,
                mip_level_count: Some(1),
                base_array_layer: cascade,
                array_layer_count: Some(1),
            });

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&format!("Shadow Render Pass Cascade {}", cascade)),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &shadow_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.shadow_render_pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);

            // Render scene geometry for this cascade
            render_scene(&render_pass, cascade);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        self.metrics.shadows_rendered += 1;
        Ok(())
    }

    pub fn get_bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn get_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn get_metrics(&self) -> &ShadowMetrics {
        &self.metrics
    }

    pub fn get_config(&self) -> &ShadowConfig {
        &self.config
    }
}