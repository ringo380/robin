use crate::engine::error::{RobinResult, RobinError};
use wgpu::util::DeviceExt;
use std::sync::Arc;
use cgmath::{Matrix4, Vector3, Vector4, Point3, InnerSpace, Zero};

/// Maximum number of lights supported per cluster
const MAX_LIGHTS_PER_CLUSTER: u32 = 256;

/// Maximum total lights in the system
const MAX_TOTAL_LIGHTS: u32 = 2048;

/// Cluster dimensions (16x9x24 = 3456 clusters)
const CLUSTER_DIMENSIONS: [u32; 3] = [16, 9, 24];

#[derive(Debug)]
pub struct ClusteredLightingSystem {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,

    // Compute pipelines
    cluster_culling_pipeline: wgpu::ComputePipeline,
    light_shading_pipeline: wgpu::ComputePipeline,

    // Light data buffers
    light_buffer: wgpu::Buffer,
    light_count_buffer: wgpu::Buffer,

    // Cluster data
    cluster_aabb_buffer: wgpu::Buffer,
    cluster_light_indices_buffer: wgpu::Buffer,
    cluster_light_counts_buffer: wgpu::Buffer,

    // Uniforms
    lighting_uniforms_buffer: wgpu::Buffer,

    // Bind groups
    culling_bind_group: Option<wgpu::BindGroup>,
    shading_bind_group: Option<wgpu::BindGroup>,

    // Light management
    lights: Vec<DynamicLight>,
    config: ClusteredLightingConfig,
    cluster_count: u32,
}

#[derive(Debug, Clone)]
pub struct ClusteredLightingConfig {
    pub enabled: bool,
    pub max_lights_per_cluster: u32,
    pub cluster_z_slices: u32,
    pub z_near: f32,
    pub z_far: f32,
    pub light_intensity_threshold: f32,
    pub ambient_intensity: f32,
    pub shadow_enabled: bool,
    pub volumetric_lighting: bool,
    pub light_culling_enabled: bool,
}

impl Default for ClusteredLightingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_lights_per_cluster: MAX_LIGHTS_PER_CLUSTER,
            cluster_z_slices: CLUSTER_DIMENSIONS[2],
            z_near: 0.1,
            z_far: 1000.0,
            light_intensity_threshold: 0.01,
            ambient_intensity: 0.1,
            shadow_enabled: true,
            volumetric_lighting: false,
            light_culling_enabled: true,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DynamicLight {
    pub position: [f32; 4],      // w = light type (0=directional, 1=point, 2=spot)
    pub direction: [f32; 4],     // w = range
    pub color: [f32; 4],         // rgb + intensity
    pub params: [f32; 4],        // spot_angle, spot_softness, shadow_bias, enabled
}

impl Default for DynamicLight {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0, 1.0], // Point light by default
            direction: [0.0, -1.0, 0.0, 10.0], // Down direction, 10 unit range
            color: [1.0, 1.0, 1.0, 1.0],    // White light, intensity 1.0
            params: [45.0, 0.1, 0.001, 1.0], // 45° spot, soft edge, low bias, enabled
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct ClusterAABB {
    min_bounds: [f32; 4],
    max_bounds: [f32; 4],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct LightingUniforms {
    view_matrix: [[f32; 4]; 4],
    proj_matrix: [[f32; 4]; 4],
    inv_proj_matrix: [[f32; 4]; 4],
    camera_position: [f32; 4],
    screen_size: [f32; 2],
    z_near: f32,
    z_far: f32,
    cluster_dimensions: [u32; 3],
    num_lights: u32,
    ambient_intensity: f32,
    light_intensity_threshold: f32,
    _padding: [f32; 2],
}

impl ClusteredLightingSystem {
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        config: ClusteredLightingConfig,
    ) -> RobinResult<Self> {
        let cluster_count = CLUSTER_DIMENSIONS[0] * CLUSTER_DIMENSIONS[1] * CLUSTER_DIMENSIONS[2];

        // Create light buffer
        let light_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Dynamic Light Buffer"),
            size: (std::mem::size_of::<DynamicLight>() * MAX_TOTAL_LIGHTS as usize) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let light_count_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Count Buffer"),
            contents: bytemuck::cast_slice(&[0u32]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        // Create cluster AABB buffer
        let cluster_aabb_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Cluster AABB Buffer"),
            size: (std::mem::size_of::<ClusterAABB>() * cluster_count as usize) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create cluster light index buffer
        let cluster_light_indices_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Cluster Light Indices Buffer"),
            size: (std::mem::size_of::<u32>() * cluster_count as usize * MAX_LIGHTS_PER_CLUSTER as usize) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create cluster light count buffer
        let cluster_light_counts_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Cluster Light Counts Buffer"),
            size: (std::mem::size_of::<u32>() * cluster_count as usize) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create uniforms buffer
        let lighting_uniforms_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Clustered Lighting Uniforms"),
            size: std::mem::size_of::<LightingUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create shaders
        let cluster_culling_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Cluster Light Culling Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/cluster_culling.wgsl").into()),
        });

        let light_shading_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Clustered Light Shading Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/clustered_lighting.wgsl").into()),
        });

        // Create pipelines
        let cluster_culling_pipeline = Self::create_culling_pipeline(&device, &cluster_culling_shader)?;
        let light_shading_pipeline = Self::create_shading_pipeline(&device, &light_shading_shader)?;

        Ok(Self {
            device,
            queue,
            cluster_culling_pipeline,
            light_shading_pipeline,
            light_buffer,
            light_count_buffer,
            cluster_aabb_buffer,
            cluster_light_indices_buffer,
            cluster_light_counts_buffer,
            lighting_uniforms_buffer,
            culling_bind_group: None,
            shading_bind_group: None,
            lights: Vec::new(),
            config,
            cluster_count,
        })
    }

    fn create_culling_pipeline(
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
    ) -> RobinResult<wgpu::ComputePipeline> {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Cluster Culling Bind Group Layout"),
            entries: &[
                // Lights buffer
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Cluster AABB buffer
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Cluster light indices (output)
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
                // Cluster light counts (output)
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
                // Uniforms
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
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
            label: Some("Cluster Culling Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        Ok(device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Cluster Light Culling Pipeline"),
            layout: Some(&pipeline_layout),
            module: shader,
            entry_point: "cs_main",
            compilation_options: Default::default(),
        }))
    }

    fn create_shading_pipeline(
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
    ) -> RobinResult<wgpu::ComputePipeline> {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Clustered Shading Bind Group Layout"),
            entries: &[
                // G-Buffer albedo
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
                // G-Buffer normals
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
                // G-Buffer depth
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
                // Output color
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
                // Lights buffer
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Cluster light indices
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Cluster light counts
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
                // Uniforms
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
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
            label: Some("Clustered Shading Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        Ok(device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Clustered Light Shading Pipeline"),
            layout: Some(&pipeline_layout),
            module: shader,
            entry_point: "cs_main",
            compilation_options: Default::default(),
        }))
    }

    pub fn add_light(&mut self, light: DynamicLight) -> u32 {
        let light_id = self.lights.len() as u32;
        self.lights.push(light);

        // Update GPU buffer
        if !self.lights.is_empty() {
            self.queue.write_buffer(
                &self.light_buffer,
                0,
                bytemuck::cast_slice(&self.lights),
            );

            self.queue.write_buffer(
                &self.light_count_buffer,
                0,
                bytemuck::cast_slice(&[self.lights.len() as u32]),
            );
        }

        light_id
    }

    pub fn update_light(&mut self, light_id: u32, light: DynamicLight) -> RobinResult<()> {
        if light_id >= self.lights.len() as u32 {
            return Err(RobinError::IndexOutOfBounds {
                index: light_id as usize,
                length: self.lights.len(),
            });
        }

        self.lights[light_id as usize] = light;

        // Update GPU buffer
        self.queue.write_buffer(
            &self.light_buffer,
            (light_id as u64) * std::mem::size_of::<DynamicLight>() as u64,
            bytemuck::cast_slice(&[light]),
        );

        Ok(())
    }

    pub fn remove_light(&mut self, light_id: u32) -> RobinResult<()> {
        if light_id >= self.lights.len() as u32 {
            return Err(RobinError::IndexOutOfBounds {
                index: light_id as usize,
                length: self.lights.len(),
            });
        }

        self.lights.remove(light_id as usize);

        // Update entire GPU buffer
        if !self.lights.is_empty() {
            self.queue.write_buffer(
                &self.light_buffer,
                0,
                bytemuck::cast_slice(&self.lights),
            );
        }

        self.queue.write_buffer(
            &self.light_count_buffer,
            0,
            bytemuck::cast_slice(&[self.lights.len() as u32]),
        );

        Ok(())
    }

    pub fn render_clustered_lighting(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view_matrix: Matrix4<f32>,
        proj_matrix: Matrix4<f32>,
        camera_position: Point3<f32>,
        screen_size: (u32, u32),
        gbuffer_albedo: &wgpu::TextureView,
        gbuffer_normals: &wgpu::TextureView,
        gbuffer_depth: &wgpu::TextureView,
        output_texture: &wgpu::TextureView,
    ) -> RobinResult<()> {
        if !self.config.enabled || self.lights.is_empty() {
            return Ok(());
        }

        // Update uniforms
        let inv_proj_matrix = proj_matrix.invert().unwrap_or(Matrix4::from_scale(1.0));

        let uniforms = LightingUniforms {
            view_matrix: view_matrix.into(),
            proj_matrix: proj_matrix.into(),
            inv_proj_matrix: inv_proj_matrix.into(),
            camera_position: [camera_position.x, camera_position.y, camera_position.z, 1.0],
            screen_size: [screen_size.0 as f32, screen_size.1 as f32],
            z_near: self.config.z_near,
            z_far: self.config.z_far,
            cluster_dimensions: CLUSTER_DIMENSIONS,
            num_lights: self.lights.len() as u32,
            ambient_intensity: self.config.ambient_intensity,
            light_intensity_threshold: self.config.light_intensity_threshold,
            _padding: [0.0, 0.0],
        };

        self.queue.write_buffer(
            &self.lighting_uniforms_buffer,
            0,
            bytemuck::cast_slice(&[uniforms]),
        );

        // Pre-compute cluster AABBs if needed
        self.compute_cluster_aabbs(view_matrix, proj_matrix, screen_size)?;

        // Phase 1: Light culling - assign lights to clusters
        if self.config.light_culling_enabled {
            self.cull_lights_to_clusters(encoder)?;
        }

        // Phase 2: Lighting computation
        self.compute_clustered_lighting(encoder, gbuffer_albedo, gbuffer_normals, gbuffer_depth, output_texture)?;

        Ok(())
    }

    fn compute_cluster_aabbs(
        &self,
        view_matrix: Matrix4<f32>,
        proj_matrix: Matrix4<f32>,
        screen_size: (u32, u32),
    ) -> RobinResult<()> {
        let mut cluster_aabbs = Vec::with_capacity(self.cluster_count as usize);

        let inv_proj = proj_matrix.invert().unwrap_or(Matrix4::from_scale(1.0));

        for z in 0..CLUSTER_DIMENSIONS[2] {
            for y in 0..CLUSTER_DIMENSIONS[1] {
                for x in 0..CLUSTER_DIMENSIONS[0] {
                    // Calculate cluster bounds in screen space
                    let x_min = (x as f32 / CLUSTER_DIMENSIONS[0] as f32) * 2.0 - 1.0;
                    let x_max = ((x + 1) as f32 / CLUSTER_DIMENSIONS[0] as f32) * 2.0 - 1.0;
                    let y_min = (y as f32 / CLUSTER_DIMENSIONS[1] as f32) * 2.0 - 1.0;
                    let y_max = ((y + 1) as f32 / CLUSTER_DIMENSIONS[1] as f32) * 2.0 - 1.0;

                    // Exponential Z distribution for better light distribution
                    let z_near = self.config.z_near;
                    let z_far = self.config.z_far;
                    let z_ratio = z_far / z_near;
                    let z_slice_factor = z_ratio.powf(1.0 / CLUSTER_DIMENSIONS[2] as f32);

                    let z_min = z_near * z_slice_factor.powi(z as i32);
                    let z_max = z_near * z_slice_factor.powi((z + 1) as i32);

                    // Convert to view space bounds
                    let corners = [
                        Vector4::new(x_min, y_min, z_min, 1.0),
                        Vector4::new(x_max, y_min, z_min, 1.0),
                        Vector4::new(x_min, y_max, z_min, 1.0),
                        Vector4::new(x_max, y_max, z_min, 1.0),
                        Vector4::new(x_min, y_min, z_max, 1.0),
                        Vector4::new(x_max, y_min, z_max, 1.0),
                        Vector4::new(x_min, y_max, z_max, 1.0),
                        Vector4::new(x_max, y_max, z_max, 1.0),
                    ];

                    let mut min_bounds = Vector3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
                    let mut max_bounds = Vector3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);

                    for corner in &corners {
                        let view_space = inv_proj * corner;
                        let view_pos = Vector3::new(
                            view_space.x / view_space.w,
                            view_space.y / view_space.w,
                            view_space.z / view_space.w,
                        );

                        min_bounds.x = min_bounds.x.min(view_pos.x);
                        min_bounds.y = min_bounds.y.min(view_pos.y);
                        min_bounds.z = min_bounds.z.min(view_pos.z);

                        max_bounds.x = max_bounds.x.max(view_pos.x);
                        max_bounds.y = max_bounds.y.max(view_pos.y);
                        max_bounds.z = max_bounds.z.max(view_pos.z);
                    }

                    cluster_aabbs.push(ClusterAABB {
                        min_bounds: [min_bounds.x, min_bounds.y, min_bounds.z, 0.0],
                        max_bounds: [max_bounds.x, max_bounds.y, max_bounds.z, 0.0],
                    });
                }
            }
        }

        // Upload cluster AABBs to GPU
        self.queue.write_buffer(
            &self.cluster_aabb_buffer,
            0,
            bytemuck::cast_slice(&cluster_aabbs),
        );

        Ok(())
    }

    fn cull_lights_to_clusters(&mut self, encoder: &mut wgpu::CommandEncoder) -> RobinResult<()> {
        // Create culling bind group if needed
        if self.culling_bind_group.is_none() {
            let layout = self.cluster_culling_pipeline.get_bind_group_layout(0);
            self.culling_bind_group = Some(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Cluster Culling Bind Group"),
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.light_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: self.cluster_aabb_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: self.cluster_light_indices_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: self.cluster_light_counts_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: self.lighting_uniforms_buffer.as_entire_binding(),
                    },
                ],
            }));
        }

        // Dispatch light culling
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Cluster Light Culling Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&self.cluster_culling_pipeline);
            compute_pass.set_bind_group(0, self.culling_bind_group.as_ref().unwrap(), &[]);

            // Dispatch one workgroup per cluster
            let workgroup_size = 64;
            let dispatch_x = (self.cluster_count + workgroup_size - 1) / workgroup_size;
            compute_pass.dispatch_workgroups(dispatch_x, 1, 1);
        }

        Ok(())
    }

    fn compute_clustered_lighting(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        gbuffer_albedo: &wgpu::TextureView,
        gbuffer_normals: &wgpu::TextureView,
        gbuffer_depth: &wgpu::TextureView,
        output_texture: &wgpu::TextureView,
    ) -> RobinResult<()> {
        // Create shading bind group if needed
        if self.shading_bind_group.is_none() {
            let layout = self.light_shading_pipeline.get_bind_group_layout(0);
            self.shading_bind_group = Some(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Clustered Shading Bind Group"),
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(gbuffer_albedo),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(gbuffer_normals),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(gbuffer_depth),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(output_texture),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: self.light_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: self.cluster_light_indices_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: self.cluster_light_counts_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: self.lighting_uniforms_buffer.as_entire_binding(),
                    },
                ],
            }));
        }

        // Dispatch lighting computation
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Clustered Lighting Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&self.light_shading_pipeline);
            compute_pass.set_bind_group(0, self.shading_bind_group.as_ref().unwrap(), &[]);

            // Dispatch per pixel
            let workgroup_size = 8;
            let dispatch_x = (output_texture.texture().width() + workgroup_size - 1) / workgroup_size;
            let dispatch_y = (output_texture.texture().height() + workgroup_size - 1) / workgroup_size;

            compute_pass.dispatch_workgroups(dispatch_x, dispatch_y, 1);
        }

        Ok(())
    }

    pub fn get_light_count(&self) -> u32 {
        self.lights.len() as u32
    }

    pub fn get_config(&self) -> &ClusteredLightingConfig {
        &self.config
    }

    pub fn update_config(&mut self, config: ClusteredLightingConfig) {
        self.config = config;
        // Invalidate bind groups on config change
        self.culling_bind_group = None;
        self.shading_bind_group = None;
    }
}

pub struct ClusteredLightingMetrics {
    pub total_lights: u32,
    pub lights_per_cluster_avg: f32,
    pub lights_per_cluster_max: u32,
    pub culling_time_ms: f32,
    pub shading_time_ms: f32,
    pub memory_usage_mb: f32,
    pub cluster_utilization: f32,
}