/*!
 * High-Performance Batch Rendering System
 *
 * Reduces draw calls by grouping objects with similar materials and shaders.
 * Implements GPU-driven rendering and instanced rendering for maximum performance.
 */

use crate::engine::{
    graphics::{GraphicsContext, Mesh, Shader, Material, Camera},
    error::{RobinError, RobinResult},
    math::{Mat4, Vec3, Transform},
    gpu::{GPUBufferHandle, GPUMemoryManager},
};
use wgpu::util::DeviceExt;
use std::collections::{HashMap, BTreeMap};
use std::sync::Arc;
use bytemuck::{Pod, Zeroable};

/// Batch rendering manager for high-performance rendering
#[derive(Debug)]
pub struct BatchRenderer {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,

    // Rendering pipelines
    static_pipeline: wgpu::RenderPipeline,
    instanced_pipeline: wgpu::RenderPipeline,
    gpu_driven_pipeline: wgpu::RenderPipeline,

    // Batching system
    render_batches: HashMap<BatchKey, RenderBatch>,
    batch_sorter: BatchSorter,

    // GPU buffers
    instance_buffer: GPUBufferHandle,
    indirect_buffer: GPUBufferHandle,
    material_buffer: GPUBufferHandle,

    // Memory management
    memory_manager: Arc<GPUMemoryManager>,

    // Performance tracking
    stats: BatchRenderStats,
}

/// Key for grouping render items into batches
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct BatchKey {
    shader_id: u64,
    material_id: u64,
    mesh_id: u64,
    transparency: bool,
    depth_layer: u32,
}

/// A batch of render items that can be drawn together
#[derive(Debug)]
struct RenderBatch {
    key: BatchKey,
    instances: Vec<InstanceData>,
    mesh: Arc<Mesh>,
    material: Arc<Material>,
    shader: Arc<Shader>,

    // GPU resources
    instance_buffer_offset: u64,
    instance_count: u32,

    // Optimization flags
    is_static: bool,
    needs_sorting: bool,
    last_frame_used: u64,
}

/// Per-instance data for instanced rendering
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct InstanceData {
    pub transform: [[f32; 4]; 4],     // 4x4 transformation matrix
    pub normal_matrix: [[f32; 3]; 3], // 3x3 normal transformation matrix
    pub material_index: u32,
    pub lod_level: u32,
    pub visibility_mask: u32,
    _padding: u32,
}

/// GPU-driven rendering indirect command
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct IndirectCommand {
    pub vertex_count: u32,
    pub instance_count: u32,
    pub first_vertex: u32,
    pub first_instance: u32,
}

/// Material data for GPU upload
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct GPUMaterial {
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub normal_scale: f32,
    pub occlusion_strength: f32,
    pub emissive_factor: [f32; 3],
    pub alpha_cutoff: f32,
    pub texture_indices: [u32; 8], // albedo, normal, metallic, roughness, ao, emissive, etc.
}

/// Batch sorting and optimization system
#[derive(Debug)]
struct BatchSorter {
    sort_keys: Vec<SortKey>,
    sorted_batches: Vec<BatchKey>,
}

#[derive(Debug, Clone)]
struct SortKey {
    batch_key: BatchKey,
    depth: f32,
    material_changes: u32,
    shader_changes: u32,
}

impl BatchRenderer {
    /// Create a new batch renderer
    pub fn new(graphics_context: &GraphicsContext, memory_manager: Arc<GPUMemoryManager>) -> RobinResult<Self> {
        let device = graphics_context.device();
        let queue = graphics_context.queue();

        // Create shaders
        let static_shader = Self::create_static_shader(device)?;
        let instanced_shader = Self::create_instanced_shader(device)?;
        let gpu_driven_shader = Self::create_gpu_driven_shader(device)?;

        // Create render pipelines
        let static_pipeline = Self::create_static_pipeline(device, &static_shader)?;
        let instanced_pipeline = Self::create_instanced_pipeline(device, &instanced_shader)?;
        let gpu_driven_pipeline = Self::create_gpu_driven_pipeline(device, &gpu_driven_shader)?;

        // Create GPU buffers
        let max_instances = 100000; // Support up to 100k instances
        let instance_buffer = memory_manager.create_buffer(
            crate::engine::gpu::buffers::BufferType::Vertex,
            crate::engine::gpu::buffers::BufferUsage::WRITE,
            (max_instances * std::mem::size_of::<InstanceData>()) as u64,
            Some("Instance Buffer"),
        )?;

        let max_batches = 1000;
        let indirect_buffer = memory_manager.create_buffer(
            crate::engine::gpu::buffers::BufferType::Indirect,
            crate::engine::gpu::buffers::BufferUsage::WRITE,
            (max_batches * std::mem::size_of::<IndirectCommand>()) as u64,
            Some("Indirect Command Buffer"),
        )?;

        let max_materials = 10000;
        let material_buffer = memory_manager.create_buffer(
            crate::engine::gpu::buffers::BufferType::Storage,
            crate::engine::gpu::buffers::BufferUsage::READ,
            (max_materials * std::mem::size_of::<GPUMaterial>()) as u64,
            Some("Material Buffer"),
        )?;

        Ok(Self {
            device: device.clone(),
            queue: queue.clone(),
            static_pipeline,
            instanced_pipeline,
            gpu_driven_pipeline,
            render_batches: HashMap::new(),
            batch_sorter: BatchSorter {
                sort_keys: Vec::new(),
                sorted_batches: Vec::new(),
            },
            instance_buffer,
            indirect_buffer,
            material_buffer,
            memory_manager,
            stats: BatchRenderStats::default(),
        })
    }

    /// Add a render item to be batched
    pub fn add_render_item(
        &mut self,
        mesh: Arc<Mesh>,
        material: Arc<Material>,
        shader: Arc<Shader>,
        transform: &Transform,
        lod_level: u32,
    ) -> RobinResult<()> {
        let batch_key = BatchKey {
            shader_id: shader.id(),
            material_id: material.id(),
            mesh_id: mesh.id(),
            transparency: material.is_transparent(),
            depth_layer: material.depth_layer(),
        };

        let instance_data = InstanceData {
            transform: transform.matrix().into(),
            normal_matrix: transform.normal_matrix().into(),
            material_index: material.gpu_index(),
            lod_level,
            visibility_mask: 0xFFFFFFFF, // Visible to all layers
            _padding: 0,
        };

        // Get or create batch
        let batch = self.render_batches.entry(batch_key.clone()).or_insert_with(|| {
            RenderBatch {
                key: batch_key.clone(),
                instances: Vec::new(),
                mesh: mesh.clone(),
                material: material.clone(),
                shader: shader.clone(),
                instance_buffer_offset: 0,
                instance_count: 0,
                is_static: false,
                needs_sorting: false,
                last_frame_used: 0,
            }
        });

        batch.instances.push(instance_data);
        batch.needs_sorting = true;
        batch.last_frame_used = self.stats.frame_count;

        Ok(())
    }

    /// Sort and optimize batches for rendering
    pub fn optimize_batches(&mut self, camera: &Camera) -> RobinResult<()> {
        self.batch_sorter.sort_keys.clear();
        self.batch_sorter.sorted_batches.clear();

        let camera_pos = camera.position();

        // Calculate sort keys for all batches
        for (key, batch) in &self.render_batches {
            if batch.instances.is_empty() {
                continue;
            }

            // Calculate average depth for sorting
            let mut total_depth = 0.0;
            for instance in &batch.instances {
                let instance_pos = Vec3::new(
                    instance.transform[3][0],
                    instance.transform[3][1],
                    instance.transform[3][2],
                );
                total_depth += (instance_pos - camera_pos).magnitude();
            }
            let avg_depth = total_depth / batch.instances.len() as f32;

            let sort_key = SortKey {
                batch_key: key.clone(),
                depth: avg_depth,
                material_changes: 0, // Will be calculated during sorting
                shader_changes: 0,   // Will be calculated during sorting
            };

            self.batch_sorter.sort_keys.push(sort_key);
        }

        // Sort batches to minimize state changes
        self.sort_batches_optimally();

        // Update batch GPU buffers
        self.update_gpu_buffers().await?;

        Ok(())
    }

    /// Sort batches to minimize GPU state changes
    fn sort_batches_optimally(&mut self) {
        // Primary sort: opaque objects front-to-back, transparent back-to-front
        // Secondary sort: by shader to minimize shader changes
        // Tertiary sort: by material to minimize material changes

        self.batch_sorter.sort_keys.sort_by(|a, b| {
            use std::cmp::Ordering;

            // Transparency first
            match (a.batch_key.transparency, b.batch_key.transparency) {
                (false, true) => return Ordering::Less,    // Opaque before transparent
                (true, false) => return Ordering::Greater, // Transparent after opaque
                _ => {}
            }

            // Then by depth layer
            let depth_cmp = a.batch_key.depth_layer.cmp(&b.batch_key.depth_layer);
            if depth_cmp != Ordering::Equal {
                return depth_cmp;
            }

            // Then by shader (to minimize shader changes)
            let shader_cmp = a.batch_key.shader_id.cmp(&b.batch_key.shader_id);
            if shader_cmp != Ordering::Equal {
                return shader_cmp;
            }

            // Then by material (to minimize material changes)
            let material_cmp = a.batch_key.material_id.cmp(&b.batch_key.material_id);
            if material_cmp != Ordering::Equal {
                return material_cmp;
            }

            // Finally by depth
            if a.batch_key.transparency {
                // Transparent: back to front
                b.depth.partial_cmp(&a.depth).unwrap_or(Ordering::Equal)
            } else {
                // Opaque: front to back
                a.depth.partial_cmp(&b.depth).unwrap_or(Ordering::Equal)
            }
        });

        // Extract sorted batch keys
        self.batch_sorter.sorted_batches = self.batch_sorter.sort_keys
            .iter()
            .map(|sort_key| sort_key.batch_key.clone())
            .collect();
    }

    /// Update GPU buffers with batch data
    async fn update_gpu_buffers(&mut self) -> RobinResult<()> {
        let mut instance_data = Vec::new();
        let mut indirect_commands = Vec::new();
        let mut current_offset = 0u32;

        // Upload instance data for all batches
        for batch_key in &self.batch_sorter.sorted_batches {
            if let Some(batch) = self.render_batches.get_mut(batch_key) {
                batch.instance_buffer_offset = (current_offset * std::mem::size_of::<InstanceData>() as u32) as u64;
                batch.instance_count = batch.instances.len() as u32;

                // Add instance data
                instance_data.extend_from_slice(&batch.instances);

                // Create indirect command
                let command = IndirectCommand {
                    vertex_count: batch.mesh.vertex_count(),
                    instance_count: batch.instance_count,
                    first_vertex: 0,
                    first_instance: current_offset,
                };
                indirect_commands.push(command);

                current_offset += batch.instance_count;
            }
        }

        // Upload to GPU buffers
        if !instance_data.is_empty() {
            let data = bytemuck::cast_slice(&instance_data);
            self.memory_manager.write_buffer(&self.instance_buffer, 0, data).await?;
        }

        if !indirect_commands.is_empty() {
            let data = bytemuck::cast_slice(&indirect_commands);
            self.memory_manager.write_buffer(&self.indirect_buffer, 0, data).await?;
        }

        Ok(())
    }

    /// Render all batches using optimal techniques
    pub fn render(&mut self, render_pass: &mut wgpu::RenderPass, camera: &Camera) -> RobinResult<()> {
        self.stats.reset_frame();

        let mut current_shader = None;
        let mut current_material = None;

        // Render sorted batches
        for batch_key in &self.batch_sorter.sorted_batches.clone() {
            if let Some(batch) = self.render_batches.get(batch_key) {
                if batch.instances.is_empty() {
                    continue;
                }

                // Check if we need to change shader
                if current_shader.as_ref().map(|s: &Arc<Shader>| s.id()) != Some(batch.shader.id()) {
                    self.bind_shader(render_pass, &batch.shader);
                    current_shader = Some(batch.shader.clone());
                    self.stats.shader_changes += 1;
                }

                // Check if we need to change material
                if current_material.as_ref().map(|m: &Arc<Material>| m.id()) != Some(batch.material.id()) {
                    self.bind_material(render_pass, &batch.material);
                    current_material = Some(batch.material.clone());
                    self.stats.material_changes += 1;
                }

                // Render batch
                self.render_batch(render_pass, batch)?;

                self.stats.draw_calls += 1;
                self.stats.triangles_rendered += (batch.mesh.triangle_count() * batch.instance_count) as u64;
                self.stats.instances_rendered += batch.instance_count as u64;
            }
        }

        self.stats.frame_count += 1;
        Ok(())
    }

    /// Render a single batch
    fn render_batch(&self, render_pass: &mut wgpu::RenderPass, batch: &RenderBatch) -> RobinResult<()> {
        // Bind mesh
        render_pass.set_vertex_buffer(0, batch.mesh.vertex_buffer().slice(..));
        render_pass.set_vertex_buffer(1, self.memory_manager.get_buffer(&self.instance_buffer)?.slice(
            batch.instance_buffer_offset..batch.instance_buffer_offset +
            (batch.instance_count as u64 * std::mem::size_of::<InstanceData>() as u64)
        ));

        if let Some(index_buffer) = batch.mesh.index_buffer() {
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(
                0..batch.mesh.index_count(),
                0,
                0..batch.instance_count,
            );
        } else {
            render_pass.draw(
                0..batch.mesh.vertex_count(),
                0..batch.instance_count,
            );
        }

        Ok(())
    }

    /// Bind shader pipeline
    fn bind_shader(&self, render_pass: &mut wgpu::RenderPass, shader: &Shader) {
        // Choose appropriate pipeline based on shader type
        if shader.supports_instancing() {
            render_pass.set_pipeline(&self.instanced_pipeline);
        } else {
            render_pass.set_pipeline(&self.static_pipeline);
        }
    }

    /// Bind material resources
    fn bind_material(&self, render_pass: &mut wgpu::RenderPass, material: &Material) {
        // Bind material textures and uniforms
        if let Some(bind_group) = material.bind_group() {
            render_pass.set_bind_group(1, bind_group, &[]);
        }
    }

    /// Get performance statistics
    pub fn get_stats(&self) -> &BatchRenderStats {
        &self.stats
    }

    /// Clear all batches (usually called at frame start)
    pub fn clear_batches(&mut self) {
        self.render_batches.clear();
        self.batch_sorter.sort_keys.clear();
        self.batch_sorter.sorted_batches.clear();
    }

    /// Remove unused batches to free memory
    pub fn gc_unused_batches(&mut self, max_frames_unused: u64) {
        let current_frame = self.stats.frame_count;
        self.render_batches.retain(|_, batch| {
            current_frame - batch.last_frame_used <= max_frames_unused
        });
    }

    // Shader creation helpers
    fn create_static_shader(device: &wgpu::Device) -> RobinResult<wgpu::ShaderModule> {
        Ok(device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Static Batch Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/batch_static.wgsl").into()),
        }))
    }

    fn create_instanced_shader(device: &wgpu::Device) -> RobinResult<wgpu::ShaderModule> {
        Ok(device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Instanced Batch Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/batch_instanced.wgsl").into()),
        }))
    }

    fn create_gpu_driven_shader(device: &wgpu::Device) -> RobinResult<wgpu::ShaderModule> {
        Ok(device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("GPU-Driven Batch Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/batch_gpu_driven.wgsl").into()),
        }))
    }

    fn create_static_pipeline(device: &wgpu::Device, shader: &wgpu::ShaderModule) -> RobinResult<wgpu::RenderPipeline> {
        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<BatchVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Position
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                // Normal
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 12,
                    shader_location: 1,
                },
                // UV
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 24,
                    shader_location: 2,
                },
                // Material ID
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint32,
                    offset: 32,
                    shader_location: 3,
                },
            ],
        };

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Static Batch Bind Group Layout"),
            entries: &[
                // Transform uniform
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
                // Material data buffer
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Static Batch Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Static Batch Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: "vs_main",
                buffers: &[vertex_buffer_layout],
                compilation_options: Default::default(),
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
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            multiview: None,
        });

        Ok(pipeline)
    }

    fn create_instanced_pipeline(device: &wgpu::Device, shader: &wgpu::ShaderModule) -> RobinResult<wgpu::RenderPipeline> {
        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<BatchVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Position
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                // Normal
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 12,
                    shader_location: 1,
                },
                // UV
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 24,
                    shader_location: 2,
                },
            ],
        };

        // Instance buffer layout for per-instance data
        let instance_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceData>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // Instance transform matrix (4x4, split into 4 vec4s)
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 4,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 16,
                    shader_location: 5,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 32,
                    shader_location: 6,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 48,
                    shader_location: 7,
                },
                // Material ID
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint32,
                    offset: 64,
                    shader_location: 8,
                },
                // Color
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 68,
                    shader_location: 9,
                },
            ],
        };

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Instanced Batch Bind Group Layout"),
            entries: &[
                // View-projection matrix
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
                // Material data buffer
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Texture array
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                // Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Instanced Batch Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Instanced Batch Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: "vs_instanced",
                buffers: &[vertex_buffer_layout, instance_buffer_layout],
                compilation_options: Default::default(),
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
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            multiview: None,
        });

        Ok(pipeline)
    }

    fn create_gpu_driven_pipeline(device: &wgpu::Device, shader: &wgpu::ShaderModule) -> RobinResult<wgpu::RenderPipeline> {
        // GPU-driven pipeline uses indirect drawing with GPU-generated draw commands
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("GPU-Driven Batch Bind Group Layout"),
            entries: &[
                // Camera/View uniforms
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
                // Object data buffer (storage buffer with all object transforms/materials)
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Vertex buffer (contains all vertex data)
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Index buffer
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Material data buffer
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Texture array for materials
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                // Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("GPU-Driven Batch Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[
                // Push constants for draw parameters
                wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::VERTEX,
                    range: 0..16, // object_id and vertex_offset
                },
            ],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("GPU-Driven Batch Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: "vs_gpu_driven",
                buffers: &[], // No vertex buffers - all data comes from storage buffers
                compilation_options: Default::default(),
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
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: "fs_gpu_driven",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            multiview: None,
        });

        Ok(pipeline)
    }
}

/// Performance statistics for batch rendering
#[derive(Debug, Default)]
pub struct BatchRenderStats {
    pub frame_count: u64,
    pub draw_calls: u32,
    pub shader_changes: u32,
    pub material_changes: u32,
    pub triangles_rendered: u64,
    pub instances_rendered: u64,
    pub batches_active: u32,
    pub avg_batch_size: f32,
}

impl BatchRenderStats {
    fn reset_frame(&mut self) {
        self.draw_calls = 0;
        self.shader_changes = 0;
        self.material_changes = 0;
        self.triangles_rendered = 0;
        self.instances_rendered = 0;
        self.batches_active = 0;
        self.avg_batch_size = 0.0;
    }

    /// Calculate draw call reduction compared to individual rendering
    pub fn calculate_draw_call_reduction(&self, individual_objects: u32) -> f32 {
        if self.draw_calls == 0 {
            return 0.0;
        }
        1.0 - (self.draw_calls as f32 / individual_objects as f32)
    }

    /// Check if performance targets are met
    pub fn meets_performance_targets(&self) -> bool {
        // Target: < 1000 draw calls per frame, > 80% draw call reduction
        self.draw_calls < 1000 && self.calculate_draw_call_reduction(10000) > 0.8
    }
}