use std::sync::Arc;
use wgpu::util::DeviceExt;
use cgmath::{Matrix4, Vector3, Vector4};
use crate::engine::error::{RobinResult, RobinError};

#[derive(Debug, Clone)]
pub struct VolumetricConfig {
    pub volume_resolution: (u32, u32, u32),
    pub fog_density: f32,
    pub fog_color: Vector3<f32>,
    pub scattering_coefficient: f32,
    pub absorption_coefficient: f32,
    pub phase_function_g: f32,
    pub temporal_upsampling: bool,
    pub ray_marching_steps: u32,
}

impl Default for VolumetricConfig {
    fn default() -> Self {
        Self {
            volume_resolution: (160, 90, 64),
            fog_density: 0.1,
            fog_color: Vector3::new(0.7, 0.8, 1.0),
            scattering_coefficient: 0.8,
            absorption_coefficient: 0.2,
            phase_function_g: 0.3,
            temporal_upsampling: true,
            ray_marching_steps: 64,
        }
    }
}

#[derive(Debug)]
pub struct VolumetricMetrics {
    pub volume_updates: u32,
    pub rays_processed: u32,
    pub light_samples: u32,
    pub upsampling_operations: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct VolumetricUniforms {
    view_matrix: Matrix4<f32>,
    proj_matrix: Matrix4<f32>,
    inv_view_proj_matrix: Matrix4<f32>,
    camera_position: Vector3<f32>,
    _padding1: f32,
    fog_density: f32,
    fog_color: Vector3<f32>,
    scattering_coefficient: f32,
    absorption_coefficient: f32,
    phase_function_g: f32,
    ray_marching_steps: u32,
    frame_index: u32,
    volume_resolution: Vector3<f32>,
    _padding2: f32,
    z_near: f32,
    z_far: f32,
    time: f32,
    _padding3: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct AtmosphereUniforms {
    sun_direction: Vector3<f32>,
    _padding1: f32,
    sun_intensity: f32,
    rayleigh_coefficient: Vector3<f32>,
    mie_coefficient: f32,
    rayleigh_scale_height: f32,
    mie_scale_height: f32,
    planet_radius: f32,
    atmosphere_radius: f32,
    _padding2: [f32; 3],
}

pub struct VolumetricSystem {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,

    // Volume textures
    volume_texture: wgpu::Texture,
    volume_view: wgpu::TextureView,
    volume_history_texture: wgpu::Texture,
    volume_history_view: wgpu::TextureView,

    // Atmospheric scattering
    atmosphere_lut_texture: wgpu::Texture,
    atmosphere_lut_view: wgpu::TextureView,

    // Compute pipelines
    volume_ray_marching_pipeline: wgpu::ComputePipeline,
    temporal_upsampling_pipeline: wgpu::ComputePipeline,
    atmosphere_precompute_pipeline: wgpu::ComputePipeline,

    // Buffers and bind groups
    volumetric_uniform_buffer: wgpu::Buffer,
    atmosphere_uniform_buffer: wgpu::Buffer,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,

    // Samplers
    volume_sampler: wgpu::Sampler,
    atmosphere_sampler: wgpu::Sampler,

    // Configuration and metrics
    config: VolumetricConfig,
    metrics: VolumetricMetrics,
    frame_index: u32,
}

impl VolumetricSystem {
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        config: VolumetricConfig
    ) -> RobinResult<Self> {
        // Create volume texture for fog/volumetric lighting
        let volume_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Volumetric Fog Texture"),
            size: wgpu::Extent3d {
                width: config.volume_resolution.0,
                height: config.volume_resolution.1,
                depth_or_array_layers: config.volume_resolution.2,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let volume_view = volume_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create history texture for temporal upsampling
        let volume_history_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Volumetric History Texture"),
            size: wgpu::Extent3d {
                width: config.volume_resolution.0,
                height: config.volume_resolution.1,
                depth_or_array_layers: config.volume_resolution.2,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let volume_history_view = volume_history_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create atmospheric scattering lookup table
        let atmosphere_lut_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Atmosphere LUT"),
            size: wgpu::Extent3d {
                width: 256,
                height: 128,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let atmosphere_lut_view = atmosphere_lut_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create samplers
        let volume_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Volume Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let atmosphere_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Atmosphere Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // Create uniform buffers
        let volumetric_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Volumetric Uniform Buffer"),
            size: std::mem::size_of::<VolumetricUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let atmosphere_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Atmosphere Uniform Buffer"),
            size: std::mem::size_of::<AtmosphereUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Volumetric Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
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
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba16Float,
                        view_dimension: wgpu::TextureViewDimension::D3,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D3,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba16Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Volumetric Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: volumetric_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: atmosphere_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&volume_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&volume_history_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&atmosphere_lut_view),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(&atmosphere_lut_view),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::Sampler(&volume_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::Sampler(&atmosphere_sampler),
                },
            ],
        });

        // Create compute pipelines
        let volume_ray_marching_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Volume Ray Marching Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/volume_ray_marching.wgsl").into()),
        });

        let volume_ray_marching_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Volume Ray Marching Pipeline"),
            layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Volume Ray Marching Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            })),
            module: &volume_ray_marching_shader,
            entry_point: "cs_main",
        });

        let temporal_upsampling_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Temporal Upsampling Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/temporal_upsampling.wgsl").into()),
        });

        let temporal_upsampling_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Temporal Upsampling Pipeline"),
            layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Temporal Upsampling Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            })),
            module: &temporal_upsampling_shader,
            entry_point: "cs_main",
        });

        let atmosphere_precompute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Atmosphere Precompute Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/atmosphere_precompute.wgsl").into()),
        });

        let atmosphere_precompute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Atmosphere Precompute Pipeline"),
            layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Atmosphere Precompute Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            })),
            module: &atmosphere_precompute_shader,
            entry_point: "cs_main",
        });

        Ok(Self {
            device,
            queue,
            volume_texture,
            volume_view,
            volume_history_texture,
            volume_history_view,
            atmosphere_lut_texture,
            atmosphere_lut_view,
            volume_ray_marching_pipeline,
            temporal_upsampling_pipeline,
            atmosphere_precompute_pipeline,
            volumetric_uniform_buffer,
            atmosphere_uniform_buffer,
            bind_group_layout,
            bind_group,
            volume_sampler,
            atmosphere_sampler,
            config,
            metrics: VolumetricMetrics {
                volume_updates: 0,
                rays_processed: 0,
                light_samples: 0,
                upsampling_operations: 0,
            },
            frame_index: 0,
        })
    }

    pub fn update(
        &mut self,
        view_matrix: Matrix4<f32>,
        proj_matrix: Matrix4<f32>,
        camera_position: Vector3<f32>,
        sun_direction: Vector3<f32>,
        time: f32,
    ) -> RobinResult<()> {
        // Update volumetric uniforms
        let inv_view_proj = (proj_matrix * view_matrix).invert().unwrap();

        let volumetric_uniforms = VolumetricUniforms {
            view_matrix,
            proj_matrix,
            inv_view_proj_matrix: inv_view_proj,
            camera_position,
            _padding1: 0.0,
            fog_density: self.config.fog_density,
            fog_color: self.config.fog_color,
            scattering_coefficient: self.config.scattering_coefficient,
            absorption_coefficient: self.config.absorption_coefficient,
            phase_function_g: self.config.phase_function_g,
            ray_marching_steps: self.config.ray_marching_steps,
            frame_index: self.frame_index,
            volume_resolution: Vector3::new(
                self.config.volume_resolution.0 as f32,
                self.config.volume_resolution.1 as f32,
                self.config.volume_resolution.2 as f32,
            ),
            _padding2: 0.0,
            z_near: 0.1,
            z_far: 1000.0,
            time,
            _padding3: 0.0,
        };

        // Update atmosphere uniforms
        let atmosphere_uniforms = AtmosphereUniforms {
            sun_direction,
            _padding1: 0.0,
            sun_intensity: 22.0,
            rayleigh_coefficient: Vector3::new(5.8e-6, 13.5e-6, 33.1e-6),
            mie_coefficient: 21e-6,
            rayleigh_scale_height: 8000.0,
            mie_scale_height: 1200.0,
            planet_radius: 6371000.0,
            atmosphere_radius: 6471000.0,
            _padding2: [0.0; 3],
        };

        self.queue.write_buffer(
            &self.volumetric_uniform_buffer,
            0,
            bytemuck::cast_slice(&[volumetric_uniforms]),
        );

        self.queue.write_buffer(
            &self.atmosphere_uniform_buffer,
            0,
            bytemuck::cast_slice(&[atmosphere_uniforms]),
        );

        self.frame_index += 1;
        Ok(())
    }

    pub fn compute_volumetric_lighting(&mut self) -> RobinResult<()> {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Volumetric Lighting Encoder"),
        });

        // Precompute atmospheric scattering LUT (only occasionally)
        if self.frame_index % 60 == 0 {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Atmosphere Precompute Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&self.atmosphere_precompute_pipeline);
            compute_pass.set_bind_group(0, &self.bind_group, &[]);
            compute_pass.dispatch_workgroups(32, 16, 1); // 256x128 LUT

            drop(compute_pass);
        }

        // Ray marching pass
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Volume Ray Marching Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&self.volume_ray_marching_pipeline);
        compute_pass.set_bind_group(0, &self.bind_group, &[]);

        let workgroup_count_x = (self.config.volume_resolution.0 + 7) / 8;
        let workgroup_count_y = (self.config.volume_resolution.1 + 7) / 8;
        let workgroup_count_z = (self.config.volume_resolution.2 + 3) / 4;

        compute_pass.dispatch_workgroups(workgroup_count_x, workgroup_count_y, workgroup_count_z);

        drop(compute_pass);

        // Temporal upsampling pass (if enabled)
        if self.config.temporal_upsampling {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Temporal Upsampling Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&self.temporal_upsampling_pipeline);
            compute_pass.set_bind_group(0, &self.bind_group, &[]);
            compute_pass.dispatch_workgroups(workgroup_count_x, workgroup_count_y, workgroup_count_z);

            drop(compute_pass);
            self.metrics.upsampling_operations += 1;
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        self.metrics.volume_updates += 1;
        self.metrics.rays_processed += (self.config.volume_resolution.0 *
                                       self.config.volume_resolution.1 *
                                       self.config.volume_resolution.2) *
                                      self.config.ray_marching_steps;

        Ok(())
    }

    pub fn get_volume_texture(&self) -> &wgpu::Texture {
        &self.volume_texture
    }

    pub fn get_volume_view(&self) -> &wgpu::TextureView {
        &self.volume_view
    }

    pub fn get_atmosphere_lut(&self) -> &wgpu::TextureView {
        &self.atmosphere_lut_view
    }

    pub fn get_bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn get_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn get_metrics(&self) -> &VolumetricMetrics {
        &self.metrics
    }

    pub fn get_config(&self) -> &VolumetricConfig {
        &self.config
    }
}