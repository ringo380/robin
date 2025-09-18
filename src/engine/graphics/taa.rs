use crate::engine::error::{RobinResult, RobinError};
use wgpu::util::DeviceExt;
use std::sync::Arc;
use cgmath::{Matrix4, Vector2, Vector3, Vector4};

#[derive(Debug)]
pub struct TAASystem {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,

    // TAA pipeline
    taa_pipeline: wgpu::ComputePipeline,
    motion_vector_pipeline: wgpu::ComputePipeline,

    // History buffers (double buffered)
    history_color_texture: wgpu::Texture,
    history_color_view: wgpu::TextureView,
    history_depth_texture: wgpu::Texture,
    history_depth_view: wgpu::TextureView,

    // Motion vector texture
    motion_vector_texture: wgpu::Texture,
    motion_vector_view: wgpu::TextureView,

    // Uniforms and buffers
    taa_uniforms_buffer: wgpu::Buffer,
    motion_uniforms_buffer: wgpu::Buffer,

    // Bind groups
    taa_bind_group: Option<wgpu::BindGroup>,
    motion_bind_group: Option<wgpu::BindGroup>,

    // Configuration
    config: TAAConfig,

    // Frame tracking
    frame_index: u32,
    previous_view_proj_matrix: Matrix4<f32>,
    jitter_sequence: Vec<Vector2<f32>>,
    screen_size: (u32, u32),
}

#[derive(Debug, Clone)]
pub struct TAAConfig {
    pub enabled: bool,
    pub temporal_weight: f32,        // Blend weight for temporal accumulation
    pub motion_threshold: f32,       // Threshold for detecting motion
    pub sharpness: f32,              // Sharpening filter strength
    pub max_history_samples: u32,    // Maximum temporal samples to accumulate
    pub ghosting_reduction: f32,     // Strength of ghosting reduction
    pub velocity_rejection: bool,    // Enable velocity-based sample rejection
    pub luminance_weighting: bool,   // Use luminance-based weighting
    pub neighborhood_clamping: bool, // Enable neighborhood color clamping
}

impl Default for TAAConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            temporal_weight: 0.95,
            motion_threshold: 0.01,
            sharpness: 0.2,
            max_history_samples: 16,
            ghosting_reduction: 0.8,
            velocity_rejection: true,
            luminance_weighting: true,
            neighborhood_clamping: true,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct TAAUniforms {
    temporal_weight: f32,
    motion_threshold: f32,
    sharpness: f32,
    ghosting_reduction: f32,
    screen_size: [f32; 2],
    jitter_offset: [f32; 2],
    frame_index: u32,
    velocity_rejection: u32,
    luminance_weighting: u32,
    neighborhood_clamping: u32,
    _padding: [f32; 2],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct MotionUniforms {
    view_proj_matrix: [[f32; 4]; 4],
    prev_view_proj_matrix: [[f32; 4]; 4],
    inv_view_proj_matrix: [[f32; 4]; 4],
    screen_size: [f32; 2],
    _padding: [f32; 2],
}

impl TAASystem {
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        screen_size: (u32, u32),
        config: TAAConfig,
    ) -> RobinResult<Self> {
        // Create history textures
        let history_color_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("TAA History Color Texture"),
            size: wgpu::Extent3d {
                width: screen_size.0,
                height: screen_size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let history_color_view = history_color_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let history_depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("TAA History Depth Texture"),
            size: wgpu::Extent3d {
                width: screen_size.0,
                height: screen_size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let history_depth_view = history_depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create motion vector texture
        let motion_vector_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("TAA Motion Vector Texture"),
            size: wgpu::Extent3d {
                width: screen_size.0,
                height: screen_size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rg16Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let motion_vector_view = motion_vector_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create uniform buffers
        let taa_uniforms_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("TAA Uniforms Buffer"),
            size: std::mem::size_of::<TAAUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let motion_uniforms_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Motion Vector Uniforms Buffer"),
            size: std::mem::size_of::<MotionUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create shaders
        let taa_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("TAA Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/taa.wgsl").into()),
        });

        let motion_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Motion Vector Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/motion_vectors.wgsl").into()),
        });

        // Create pipelines
        let taa_pipeline = Self::create_taa_pipeline(&device, &taa_shader)?;
        let motion_vector_pipeline = Self::create_motion_pipeline(&device, &motion_shader)?;

        // Generate jitter sequence (Halton sequence for stable temporal sampling)
        let jitter_sequence = Self::generate_halton_jitter_sequence(16);

        Ok(Self {
            device,
            queue,
            taa_pipeline,
            motion_vector_pipeline,
            history_color_texture,
            history_color_view,
            history_depth_texture,
            history_depth_view,
            motion_vector_texture,
            motion_vector_view,
            taa_uniforms_buffer,
            motion_uniforms_buffer,
            taa_bind_group: None,
            motion_bind_group: None,
            config,
            frame_index: 0,
            previous_view_proj_matrix: Matrix4::from_scale(1.0),
            jitter_sequence,
            screen_size,
        })
    }

    fn create_taa_pipeline(
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
    ) -> RobinResult<wgpu::ComputePipeline> {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("TAA Bind Group Layout"),
            entries: &[
                // Current color texture
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
                // History color texture
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Motion vector texture
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Output texture
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba16Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                // Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
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
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("TAA Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        Ok(device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("TAA Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: shader,
            entry_point: "cs_main",
            compilation_options: Default::default(),
        }))
    }

    fn create_motion_pipeline(
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
    ) -> RobinResult<wgpu::ComputePipeline> {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Motion Vector Bind Group Layout"),
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
                // Motion vector output
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rg16Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                // Uniforms
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Motion Vector Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        Ok(device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Motion Vector Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: shader,
            entry_point: "cs_main",
            compilation_options: Default::default(),
        }))
    }

    fn generate_halton_jitter_sequence(count: usize) -> Vec<Vector2<f32>> {
        let mut sequence = Vec::with_capacity(count);

        for i in 0..count {
            let x = Self::halton_sequence(i + 1, 2);
            let y = Self::halton_sequence(i + 1, 3);

            // Convert to pixel offset range [-0.5, 0.5]
            sequence.push(Vector2::new(x - 0.5, y - 0.5));
        }

        sequence
    }

    fn halton_sequence(index: usize, base: usize) -> f32 {
        let mut result = 0.0;
        let mut f = 1.0 / base as f32;
        let mut i = index;

        while i > 0 {
            result += f * (i % base) as f32;
            i /= base;
            f /= base as f32;
        }

        result
    }

    pub fn get_jitter_offset(&self) -> Vector2<f32> {
        if !self.config.enabled {
            return Vector2::new(0.0, 0.0);
        }

        let index = (self.frame_index as usize) % self.jitter_sequence.len();
        self.jitter_sequence[index]
    }

    pub fn render_motion_vectors(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        depth_texture_view: &wgpu::TextureView,
        current_view_proj_matrix: Matrix4<f32>,
    ) -> RobinResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        // Update motion vector uniforms
        let inv_view_proj_matrix = current_view_proj_matrix.invert()
            .ok_or_else(|| RobinError::Graphics("Failed to invert view-projection matrix".to_string()))?;

        let motion_uniforms = MotionUniforms {
            view_proj_matrix: current_view_proj_matrix.into(),
            prev_view_proj_matrix: self.previous_view_proj_matrix.into(),
            inv_view_proj_matrix: inv_view_proj_matrix.into(),
            screen_size: [self.screen_size.0 as f32, self.screen_size.1 as f32],
            _padding: [0.0, 0.0],
        };

        self.queue.write_buffer(
            &self.motion_uniforms_buffer,
            0,
            bytemuck::cast_slice(&[motion_uniforms]),
        );

        // Create motion vector bind group if needed
        if self.motion_bind_group.is_none() {
            let layout = self.motion_vector_pipeline.get_bind_group_layout(0);
            self.motion_bind_group = Some(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Motion Vector Bind Group"),
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(depth_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&self.motion_vector_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: self.motion_uniforms_buffer.as_entire_binding(),
                    },
                ],
            }));
        }

        // Dispatch motion vector generation
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Motion Vector Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&self.motion_vector_pipeline);
            compute_pass.set_bind_group(0, self.motion_bind_group.as_ref().unwrap(), &[]);

            let workgroup_size = 8;
            let dispatch_x = (self.screen_size.0 + workgroup_size - 1) / workgroup_size;
            let dispatch_y = (self.screen_size.1 + workgroup_size - 1) / workgroup_size;

            compute_pass.dispatch_workgroups(dispatch_x, dispatch_y, 1);
        }

        // Store current matrix for next frame
        self.previous_view_proj_matrix = current_view_proj_matrix;

        Ok(())
    }

    pub fn render_taa(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        current_color_texture: &wgpu::Texture,
        current_color_view: &wgpu::TextureView,
        output_color_texture: &wgpu::Texture,
        output_color_view: &wgpu::TextureView,
        sampler: &wgpu::Sampler,
    ) -> RobinResult<()> {
        if !self.config.enabled {
            // If TAA disabled, just copy current to output
            encoder.copy_texture_to_texture(
                wgpu::ImageCopyTexture {
                    texture: current_color_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::ImageCopyTexture {
                    texture: output_color_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::Extent3d {
                    width: self.screen_size.0,
                    height: self.screen_size.1,
                    depth_or_array_layers: 1,
                },
            );
            return Ok(());
        }

        // Update TAA uniforms
        let jitter_offset = self.get_jitter_offset();
        let taa_uniforms = TAAUniforms {
            temporal_weight: self.config.temporal_weight,
            motion_threshold: self.config.motion_threshold,
            sharpness: self.config.sharpness,
            ghosting_reduction: self.config.ghosting_reduction,
            screen_size: [self.screen_size.0 as f32, self.screen_size.1 as f32],
            jitter_offset: [jitter_offset.x, jitter_offset.y],
            frame_index: self.frame_index,
            velocity_rejection: self.config.velocity_rejection as u32,
            luminance_weighting: self.config.luminance_weighting as u32,
            neighborhood_clamping: self.config.neighborhood_clamping as u32,
            _padding: [0.0, 0.0],
        };

        self.queue.write_buffer(
            &self.taa_uniforms_buffer,
            0,
            bytemuck::cast_slice(&[taa_uniforms]),
        );

        // Create TAA bind group if needed
        if self.taa_bind_group.is_none() {
            let layout = self.taa_pipeline.get_bind_group_layout(0);
            self.taa_bind_group = Some(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("TAA Bind Group"),
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(current_color_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&self.history_color_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&self.motion_vector_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(output_color_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: self.taa_uniforms_buffer.as_entire_binding(),
                    },
                ],
            }));
        }

        // Dispatch TAA
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("TAA Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&self.taa_pipeline);
            compute_pass.set_bind_group(0, self.taa_bind_group.as_ref().unwrap(), &[]);

            let workgroup_size = 8;
            let dispatch_x = (self.screen_size.0 + workgroup_size - 1) / workgroup_size;
            let dispatch_y = (self.screen_size.1 + workgroup_size - 1) / workgroup_size;

            compute_pass.dispatch_workgroups(dispatch_x, dispatch_y, 1);
        }

        // Copy output to history for next frame
        encoder.copy_texture_to_texture(
            wgpu::ImageCopyTexture {
                texture: output_color_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyTexture {
                texture: &self.history_color_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: self.screen_size.0,
                height: self.screen_size.1,
                depth_or_array_layers: 1,
            },
        );

        self.frame_index += 1;
        Ok(())
    }

    pub fn update_config(&mut self, config: TAAConfig) {
        self.config = config;
    }

    pub fn get_config(&self) -> &TAAConfig {
        &self.config
    }

    pub fn resize(&mut self, new_size: (u32, u32)) -> RobinResult<()> {
        if new_size == self.screen_size {
            return Ok(());
        }

        self.screen_size = new_size;

        // Recreate textures with new size
        self.history_color_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("TAA History Color Texture"),
            size: wgpu::Extent3d {
                width: new_size.0,
                height: new_size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        self.history_color_view = self.history_color_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Recreate motion vector texture
        self.motion_vector_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("TAA Motion Vector Texture"),
            size: wgpu::Extent3d {
                width: new_size.0,
                height: new_size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rg16Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        self.motion_vector_view = self.motion_vector_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Invalidate bind groups to force recreation
        self.taa_bind_group = None;
        self.motion_bind_group = None;

        Ok(())
    }
}

pub struct TAAMetrics {
    pub temporal_stability: f32,
    pub ghosting_level: f32,
    pub sharpness_score: f32,
    pub performance_cost_ms: f32,
    pub memory_usage_mb: f32,
}