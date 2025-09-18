use crate::engine::error::{RobinResult, RobinError};
use wgpu::util::DeviceExt;
use std::sync::Arc;
use cgmath::{Matrix4, Vector3, Vector4, InnerSpace};

#[derive(Debug)]
pub struct SSAOSystem {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,

    // SSAO pipeline
    ssao_pipeline: wgpu::ComputePipeline,
    blur_pipeline: wgpu::ComputePipeline,

    // Buffers and textures
    ssao_uniforms_buffer: wgpu::Buffer,
    noise_texture: wgpu::Texture,
    noise_texture_view: wgpu::TextureView,
    kernel_buffer: wgpu::Buffer,

    // Bind groups
    ssao_bind_group: wgpu::BindGroup,
    blur_bind_group: wgpu::BindGroup,

    // Configuration
    config: SSAOConfig,
    kernel_samples: Vec<Vector3<f32>>,
    noise_vectors: Vec<Vector4<f32>>,
}

#[derive(Debug, Clone)]
pub struct SSAOConfig {
    pub enabled: bool,
    pub sample_count: u32,
    pub radius: f32,
    pub bias: f32,
    pub power: f32,
    pub intensity: f32,
    pub blur_enabled: bool,
    pub temporal_filtering: bool,
    pub noise_scale: f32,
}

impl Default for SSAOConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sample_count: 64,
            radius: 1.0,
            bias: 0.025,
            power: 2.0,
            intensity: 1.0,
            blur_enabled: true,
            temporal_filtering: true,
            noise_scale: 1.0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct SSAOUniforms {
    view_matrix: [[f32; 4]; 4],
    proj_matrix: [[f32; 4]; 4],
    inv_proj_matrix: [[f32; 4]; 4],
    sample_count: u32,
    radius: f32,
    bias: f32,
    power: f32,
    intensity: f32,
    noise_scale: f32,
    screen_size: [f32; 2],
}

impl SSAOSystem {
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        config: SSAOConfig,
    ) -> RobinResult<Self> {
        // Generate hemisphere kernel samples
        let kernel_samples = Self::generate_hemisphere_kernel(config.sample_count as usize);

        // Generate noise vectors for randomization
        let noise_vectors = Self::generate_noise_vectors(16); // 4x4 noise texture

        // Create noise texture
        let noise_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("SSAO Noise Texture"),
            size: wgpu::Extent3d {
                width: 4,
                height: 4,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let noise_texture_view = noise_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Upload noise data
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &noise_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&noise_vectors),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * 4 * 16), // 4 pixels * 4 components * 4 bytes
                rows_per_image: Some(4),
            },
            wgpu::Extent3d {
                width: 4,
                height: 4,
                depth_or_array_layers: 1,
            },
        );

        // Create kernel buffer
        let kernel_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("SSAO Kernel Buffer"),
            contents: bytemuck::cast_slice(&kernel_samples),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        // Create uniforms buffer
        let ssao_uniforms_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("SSAO Uniforms Buffer"),
            size: std::mem::size_of::<SSAOUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create shaders
        let ssao_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("SSAO Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/ssao.wgsl").into()),
        });

        let blur_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("SSAO Blur Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/ssao_blur.wgsl").into()),
        });

        // Create bind group layouts
        let ssao_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("SSAO Bind Group Layout"),
            entries: &[
                // Depth texture
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Normal texture
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Noise texture
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // SSAO output texture
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::R32Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                // Uniforms
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Kernel samples
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let blur_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("SSAO Blur Bind Group Layout"),
            entries: &[
                // SSAO input texture
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Blurred output texture
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::R32Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });

        // Create pipelines
        let ssao_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("SSAO Pipeline Layout"),
            bind_group_layouts: &[&ssao_bind_group_layout],
            push_constant_ranges: &[],
        });

        let ssao_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("SSAO Compute Pipeline"),
            layout: Some(&ssao_pipeline_layout),
            module: &ssao_shader,
            entry_point: "cs_main",
            compilation_options: Default::default(),
        });

        let blur_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("SSAO Blur Pipeline Layout"),
            bind_group_layouts: &[&blur_bind_group_layout],
            push_constant_ranges: &[],
        });

        let blur_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("SSAO Blur Pipeline"),
            layout: Some(&blur_pipeline_layout),
            module: &blur_shader,
            entry_point: "cs_main",
            compilation_options: Default::default(),
        });

        // Temporary bind groups (will be recreated with actual textures)
        let ssao_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("SSAO Bind Group"),
            layout: &ssao_bind_group_layout,
            entries: &[],
        });

        let blur_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("SSAO Blur Bind Group"),
            layout: &blur_bind_group_layout,
            entries: &[],
        });

        Ok(Self {
            device,
            queue,
            ssao_pipeline,
            blur_pipeline,
            ssao_uniforms_buffer,
            noise_texture,
            noise_texture_view,
            kernel_buffer,
            ssao_bind_group,
            blur_bind_group,
            config,
            kernel_samples,
            noise_vectors,
        })
    }

    fn generate_hemisphere_kernel(sample_count: usize) -> Vec<Vector3<f32>> {
        use rand::{Rng, SeedableRng};
        let mut rng = rand::rngs::StdRng::seed_from_u64(42); // Deterministic for consistency

        let mut samples = Vec::with_capacity(sample_count);

        for i in 0..sample_count {
            // Generate random point in hemisphere
            let mut sample = Vector3::new(
                rng.gen::<f32>() * 2.0 - 1.0, // -1 to 1
                rng.gen::<f32>() * 2.0 - 1.0, // -1 to 1
                rng.gen::<f32>(),             // 0 to 1 (hemisphere)
            );

            // Normalize and scale
            sample = sample.normalize();

            // Scale with accelerating interpolation to distribute samples closer to origin
            let scale = i as f32 / sample_count as f32;
            let scale = 0.1 + scale * scale * 0.9; // lerp(0.1, 1.0, scale²)
            sample = sample * scale;

            samples.push(sample);
        }

        samples
    }

    fn generate_noise_vectors(count: usize) -> Vec<Vector4<f32>> {
        use rand::{Rng, SeedableRng};
        let mut rng = rand::rngs::StdRng::seed_from_u64(123);

        let mut noise = Vec::with_capacity(count);

        for _ in 0..count {
            // Generate random rotation around Z-axis
            let rotation = Vector4::new(
                rng.gen::<f32>() * 2.0 - 1.0, // x: -1 to 1
                rng.gen::<f32>() * 2.0 - 1.0, // y: -1 to 1
                0.0,                          // z: 0 (tangent space)
                1.0,                          // w: padding
            );
            noise.push(rotation);
        }

        noise
    }

    pub fn render_ssao(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        depth_texture_view: &wgpu::TextureView,
        normal_texture_view: &wgpu::TextureView,
        ssao_output_view: &wgpu::TextureView,
        view_matrix: Matrix4<f32>,
        proj_matrix: Matrix4<f32>,
        screen_size: (u32, u32),
    ) -> RobinResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        // Update uniforms
        let inv_proj_matrix = proj_matrix.invert().unwrap_or(Matrix4::from_scale(1.0));

        let uniforms = SSAOUniforms {
            view_matrix: view_matrix.into(),
            proj_matrix: proj_matrix.into(),
            inv_proj_matrix: inv_proj_matrix.into(),
            sample_count: self.config.sample_count,
            radius: self.config.radius,
            bias: self.config.bias,
            power: self.config.power,
            intensity: self.config.intensity,
            noise_scale: self.config.noise_scale,
            screen_size: [screen_size.0 as f32, screen_size.1 as f32],
        };

        self.queue.write_buffer(&self.ssao_uniforms_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        // SSAO generation pass
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("SSAO Generation Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&self.ssao_pipeline);
            compute_pass.set_bind_group(0, &self.ssao_bind_group, &[]);

            let workgroup_size = 8;
            let dispatch_x = (screen_size.0 + workgroup_size - 1) / workgroup_size;
            let dispatch_y = (screen_size.1 + workgroup_size - 1) / workgroup_size;

            compute_pass.dispatch_workgroups(dispatch_x, dispatch_y, 1);
        }

        // Optional blur pass
        if self.config.blur_enabled {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("SSAO Blur Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&self.blur_pipeline);
            compute_pass.set_bind_group(0, &self.blur_bind_group, &[]);

            let workgroup_size = 8;
            let dispatch_x = (screen_size.0 + workgroup_size - 1) / workgroup_size;
            let dispatch_y = (screen_size.1 + workgroup_size - 1) / workgroup_size;

            compute_pass.dispatch_workgroups(dispatch_x, dispatch_y, 1);
        }

        Ok(())
    }

    pub fn update_config(&mut self, config: SSAOConfig) -> RobinResult<()> {
        self.config = config;

        // Regenerate kernel if sample count changed
        if self.kernel_samples.len() != self.config.sample_count as usize {
            self.kernel_samples = Self::generate_hemisphere_kernel(self.config.sample_count as usize);

            // Update kernel buffer
            self.queue.write_buffer(
                &self.kernel_buffer,
                0,
                bytemuck::cast_slice(&self.kernel_samples),
            );
        }

        Ok(())
    }

    pub fn get_config(&self) -> &SSAOConfig {
        &self.config
    }

    pub fn create_bind_groups(
        &mut self,
        depth_texture_view: &wgpu::TextureView,
        normal_texture_view: &wgpu::TextureView,
        ssao_output_view: &wgpu::TextureView,
        sampler: &wgpu::Sampler,
    ) -> RobinResult<()> {
        // Get bind group layouts from pipelines
        let ssao_layout = self.ssao_pipeline.get_bind_group_layout(0);
        let blur_layout = self.blur_pipeline.get_bind_group_layout(0);

        // Create SSAO bind group
        self.ssao_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("SSAO Bind Group"),
            layout: &ssao_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(depth_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(normal_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&self.noise_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(ssao_output_view),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: self.ssao_uniforms_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: self.kernel_buffer.as_entire_binding(),
                },
            ],
        });

        Ok(())
    }
}

pub struct SSAOMetrics {
    pub generation_time_ms: f32,
    pub blur_time_ms: f32,
    pub sample_count: u32,
    pub memory_usage_mb: f32,
    pub quality_level: f32,
}