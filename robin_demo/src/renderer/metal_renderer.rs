// Metal renderer for macOS-native performance
// Optimized for Apple Silicon with unified memory architecture

use metal::*;
use metal::foreign_types::ForeignType;
use core_graphics::geometry::CGSize;
use imgui::{DrawCmd, DrawData, DrawList, DrawVert};
use objc::{msg_send, sel, sel_impl};

use crate::window::NativeWindow;
use crate::culling::{HierarchicalCuller, ChunkId, CameraFrustum, AABB};
use crate::lod_system::{LodSystem, LodConfig, LodLevel, LodStatistics};
use crate::material_batching::{MaterialBatcher, MaterialType, BatchingStats};
use crate::performance_monitor::{PerformanceMonitor as BatchingPerformanceMonitor, PerformanceMetrics, BatchPerformanceStats, MonitorConfig};
use crate::dynamic_texture_atlas::{DynamicTextureAtlas, AtlasStats, AtlasUV};
use crate::pbr_lighting::{PBRLightingSystem, PBRMaterial, Light, GPULight, EnvironmentLighting, ToneMappingMode};
use crate::chunk_streaming::{ChunkStreamingSystem, StreamingConfig, simple_terrain_generator};
use super::mesh::Mesh;
use super::shaders::COMBINED_SHADER_SOURCE;
use super::{Uniforms, Camera};
use super::error_handling::{MetalError, MetalResult, ErrorRecovery, PerformanceMonitor};

pub struct MetalRenderer {
    device: Device,
    command_queue: CommandQueue,
    layer: MetalLayer,
    render_pipeline: RenderPipelineState,
    alpha_pipeline: RenderPipelineState,
    sky_pipeline: RenderPipelineState,
    depth_stencil_state: DepthStencilState,
    uniform_buffer: Buffer,
    drawable_size: CGSize,
    font_texture: Option<Texture>,
    atlas_texture: Option<Texture>,
    atlas_sampler: Option<SamplerState>,
    // Sky rendering
    sky_mesh: Mesh,
    // Celestial body rendering (sun/moon)
    celestial_pipeline: RenderPipelineState,
    celestial_uniform_buffer: Buffer,
    sun_mesh: Mesh,
    moon_mesh: Mesh,
    // UI rendering
    ui_pipeline: Option<RenderPipelineState>,
    ui_vertex_buffer: Option<Buffer>,
    ui_index_buffer: Option<Buffer>,
    ui_uniform_buffer: Option<Buffer>,
    font_sampler: Option<SamplerState>,
    // Performance monitoring and error recovery
    performance_monitor: PerformanceMonitor,
    last_frame_time: std::time::Instant,
    // Hierarchical frustum culling
    hierarchical_culler: HierarchicalCuller,
    visible_chunks: Vec<ChunkId>,
    // Level of Detail system
    lod_system: LodSystem,
    chunk_positions: std::collections::HashMap<ChunkId, cgmath::Vector3<f32>>,
    // Material batching system
    material_batcher: MaterialBatcher,
    // Batching performance monitoring
    batching_performance_monitor: BatchingPerformanceMonitor,
    // Dynamic texture atlas system
    dynamic_texture_atlas: DynamicTextureAtlas,
    // PBR lighting system
    pbr_lighting_system: PBRLightingSystem,
    light_buffer: Buffer,
    material_buffer: Buffer,
    // Chunk streaming system
    chunk_streaming_system: ChunkStreamingSystem,
}

impl MetalRenderer {
    pub fn new(window: &NativeWindow) -> Result<Self, Box<dyn std::error::Error>> {
        // Create Metal device with proper error handling
        let device = ErrorRecovery::retry_with_backoff(
            || Device::system_default().ok_or(MetalError::DeviceNotFound),
            3,
            "Metal device creation"
        ).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        log::info!("🚀 Using Metal device: {}", device.name());

        // Validate device capabilities
        if !device.supports_family(MTLGPUFamily::Mac2) {
            log::warn!("⚠️  Metal device may have limited capabilities");
        }

        // Create command queue
        let command_queue = device.new_command_queue();

        // Get the Metal layer from the window (already created by NativeWindow)
        let metal_layer_id = window.get_metal_layer();
        let layer = unsafe { MetalLayer::from_ptr(metal_layer_id as *mut _) };

        // Configure the layer
        layer.set_device(&device);
        layer.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
        layer.set_presents_with_transaction(false);

        // Enable for Apple Silicon optimizations
        if device.supports_family(MTLGPUFamily::Apple7) {
            layer.set_framebuffer_only(false); // Allow compute access
        }

        // Create shaders and pipelines
        let library = Self::create_shader_library(&device)?;
        let render_pipeline = Self::create_render_pipeline(&device, &library)?;
        let alpha_pipeline = Self::create_alpha_pipeline(&device, &library)?;
        let sky_pipeline = Self::create_sky_pipeline(&device, &library)?;
        let celestial_pipeline = Self::create_celestial_pipeline(&device, &library)?;
        let depth_stencil_state = Self::create_depth_stencil_state(&device);

        // Create uniform buffer
        let uniform_buffer = device.new_buffer(
            std::mem::size_of::<Uniforms>() as u64,
            MTLResourceOptions::StorageModeShared,
        );

        // Create celestial uniform buffer (for separate celestial shader uniforms)
        let celestial_uniform_buffer = device.new_buffer(
            80, // CelestialUniforms struct size (4x4 matrix + 4 vec4s = 80 bytes)
            MTLResourceOptions::StorageModeShared,
        );

        // Create sky cube mesh
        let mut sky_mesh = Self::create_sky_cube();
        sky_mesh.create_buffers(&device);

        // Create celestial body meshes
        let mut sun_mesh = Self::create_sphere_mesh(32);  // High quality for sun
        sun_mesh.create_buffers(&device);
        let mut moon_mesh = Self::create_sphere_mesh(24); // Medium quality for moon
        moon_mesh.create_buffers(&device);

        // Get initial drawable size from window
        let window_size = window.get_size();
        // Set drawable size using message send to avoid type issues
        unsafe {
            let drawable_size = CGSize::new(window_size.width as f64, window_size.height as f64);
            let _: () = msg_send![layer.as_ptr(), setDrawableSize: drawable_size];
        }

        // Initialize hierarchical frustum culler with world bounds
        let world_bounds = AABB::new(
            cgmath::Point3::new(-1024.0, -256.0, -1024.0),
            cgmath::Point3::new(1024.0, 256.0, 1024.0)
        );
        let hierarchical_culler = HierarchicalCuller::new(world_bounds, 32.0, 6);

        // Initialize LOD system with default configuration
        let lod_config = LodConfig::default();
        let lod_system = LodSystem::new(lod_config);

        // Initialize material batching system
        let material_batcher = MaterialBatcher::new();

        // Initialize batching performance monitor
        let batching_performance_monitor = BatchingPerformanceMonitor::new(MonitorConfig::default());

        // Initialize dynamic texture atlas system (1024x1024 atlases)
        let mut dynamic_texture_atlas = DynamicTextureAtlas::new(1024);
        dynamic_texture_atlas.preload_materials()?;
        dynamic_texture_atlas.create_metal_textures(&device)?;

        // Initialize PBR lighting system
        let pbr_lighting_system = PBRLightingSystem::new();

        // Create GPU buffers for lighting data
        let max_lights = 32;
        let light_buffer = device.new_buffer(
            (std::mem::size_of::<GPULight>() * max_lights) as u64,
            MTLResourceOptions::StorageModeShared
        );

        let max_materials = 16;
        let material_buffer = device.new_buffer(
            (std::mem::size_of::<PBRMaterial>() * max_materials) as u64,
            MTLResourceOptions::StorageModeShared
        );

        // Initialize chunk streaming system
        let streaming_config = StreamingConfig {
            chunk_size: 32,
            render_distance: 12,
            full_detail_distance: 96.0,
            half_detail_distance: 192.0,
            quarter_detail_distance: 384.0,
            max_loaded_chunks: 500,
            max_memory_mb: 256.0,
            background_thread_count: 2,
            load_queue_size: 30,
            unload_queue_size: 15,
            priority_update_interval: std::time::Duration::from_millis(200),
            memory_cleanup_interval: std::time::Duration::from_secs(10),
        };

        let generation_fn = Box::new(simple_terrain_generator);
        let render_distance = streaming_config.render_distance;
        let chunk_streaming_system = ChunkStreamingSystem::new(streaming_config, generation_fn);

        log::info!("🔧 Hierarchical frustum culler initialized");
        log::info!("📊 LOD system initialized with default configuration");
        log::info!("🎨 Material batching system initialized");
        log::info!("📈 Batching performance monitor initialized");
        log::info!("🖼️  Dynamic texture atlas system initialized");
        log::info!("💡 PBR lighting system initialized with {} materials", pbr_lighting_system.material_library.get_all_materials().len());
        log::info!("🌍 Chunk streaming system initialized with {} render distance", render_distance);

        Ok(Self {
            device,
            command_queue,
            layer,
            render_pipeline,
            alpha_pipeline,
            sky_pipeline,
            celestial_pipeline,
            depth_stencil_state,
            uniform_buffer,
            celestial_uniform_buffer,
            drawable_size: window_size,
            font_texture: None,
            atlas_texture: None,
            atlas_sampler: None,
            sky_mesh,
            sun_mesh,
            moon_mesh,
            ui_pipeline: None,
            ui_vertex_buffer: None,
            ui_index_buffer: None,
            ui_uniform_buffer: None,
            font_sampler: None,
            performance_monitor: PerformanceMonitor::new(),
            last_frame_time: std::time::Instant::now(),
            hierarchical_culler,
            visible_chunks: Vec::new(),
            lod_system,
            chunk_positions: std::collections::HashMap::new(),
            material_batcher,
            batching_performance_monitor,
            dynamic_texture_atlas,
            pbr_lighting_system,
            light_buffer,
            material_buffer,
            chunk_streaming_system,
        })
    }

    fn create_shader_library(device: &DeviceRef) -> Result<Library, Box<dyn std::error::Error>> {
        let library = device
            .new_library_with_source(COMBINED_SHADER_SOURCE, &CompileOptions::new())
            .map_err(|e| format!("Failed to compile shaders: {}", e))?;

        println!("✅ Metal shaders compiled successfully");
        Ok(library)
    }

    fn create_render_pipeline(
        device: &DeviceRef,
        library: &LibraryRef,
    ) -> Result<RenderPipelineState, Box<dyn std::error::Error>> {
        let vertex_function = library
            .get_function("vertex_main", None)
            .map_err(|e| format!("Vertex function not found: {}", e))?;

        let fragment_function = library
            .get_function("fragment_main", None)
            .map_err(|e| format!("Fragment function not found: {}", e))?;

        let pipeline_descriptor = RenderPipelineDescriptor::new();
        pipeline_descriptor.set_vertex_function(Some(&vertex_function));
        pipeline_descriptor.set_fragment_function(Some(&fragment_function));

        // Configure color attachment
        let color_attachments = pipeline_descriptor.color_attachments();
        color_attachments
            .object_at(0)
            .unwrap()
            .set_pixel_format(MTLPixelFormat::BGRA8Unorm);

        // Configure depth attachment
        pipeline_descriptor.set_depth_attachment_pixel_format(MTLPixelFormat::Depth32Float);

        // Configure vertex descriptor
        let vertex_descriptor = VertexDescriptor::new();
        let attributes = vertex_descriptor.attributes();
        let layouts = vertex_descriptor.layouts();

        // Position attribute
        attributes.object_at(0).unwrap().set_format(MTLVertexFormat::Float3);
        attributes.object_at(0).unwrap().set_offset(0);
        attributes.object_at(0).unwrap().set_buffer_index(0);

        // Color attribute
        attributes.object_at(1).unwrap().set_format(MTLVertexFormat::Float3);
        attributes.object_at(1).unwrap().set_offset(12);
        attributes.object_at(1).unwrap().set_buffer_index(0);

        // Normal attribute
        attributes.object_at(2).unwrap().set_format(MTLVertexFormat::Float3);
        attributes.object_at(2).unwrap().set_offset(24);
        attributes.object_at(2).unwrap().set_buffer_index(0);

        // Texture coordinates attribute
        attributes.object_at(3).unwrap().set_format(MTLVertexFormat::Float2);
        attributes.object_at(3).unwrap().set_offset(36);
        attributes.object_at(3).unwrap().set_buffer_index(0);

        // Layout (use size of our vertex struct - 44 bytes)
        layouts.object_at(0).unwrap().set_stride(44);
        layouts.object_at(0).unwrap().set_step_rate(1);
        layouts.object_at(0).unwrap().set_step_function(MTLVertexStepFunction::PerVertex);

        pipeline_descriptor.set_vertex_descriptor(Some(&vertex_descriptor));

        let pipeline_state = device
            .new_render_pipeline_state(&pipeline_descriptor)
            .map_err(|e| format!("Failed to create render pipeline: {}", e))?;

        println!("✅ Metal render pipeline created");
        Ok(pipeline_state)
    }

    fn create_alpha_pipeline(
        device: &DeviceRef,
        library: &LibraryRef,
    ) -> Result<RenderPipelineState, Box<dyn std::error::Error>> {
        let vertex_function = library
            .get_function("vertex_main", None)
            .map_err(|e| format!("Vertex function not found: {}", e))?;

        let fragment_function = library
            .get_function("fragment_main", None)
            .map_err(|e| format!("Fragment function not found: {}", e))?;

        let pipeline_descriptor = RenderPipelineDescriptor::new();
        pipeline_descriptor.set_vertex_function(Some(&vertex_function));
        pipeline_descriptor.set_fragment_function(Some(&fragment_function));

        // Configure color attachment with alpha blending for ghost blocks
        let color_attachments = pipeline_descriptor.color_attachments();
        let color_attachment = color_attachments.object_at(0).unwrap();
        color_attachment.set_pixel_format(MTLPixelFormat::BGRA8Unorm);

        // Enable alpha blending for transparent ghost blocks
        color_attachment.set_blending_enabled(true);
        color_attachment.set_source_rgb_blend_factor(MTLBlendFactor::SourceAlpha);
        color_attachment.set_destination_rgb_blend_factor(MTLBlendFactor::OneMinusSourceAlpha);
        color_attachment.set_rgb_blend_operation(MTLBlendOperation::Add);
        color_attachment.set_source_alpha_blend_factor(MTLBlendFactor::One);
        color_attachment.set_destination_alpha_blend_factor(MTLBlendFactor::OneMinusSourceAlpha);
        color_attachment.set_alpha_blend_operation(MTLBlendOperation::Add);

        // Configure depth attachment
        pipeline_descriptor.set_depth_attachment_pixel_format(MTLPixelFormat::Depth32Float);

        // Configure vertex descriptor (same as main pipeline)
        let vertex_descriptor = VertexDescriptor::new();
        let attributes = vertex_descriptor.attributes();
        let layouts = vertex_descriptor.layouts();

        // Position attribute
        attributes.object_at(0).unwrap().set_format(MTLVertexFormat::Float3);
        attributes.object_at(0).unwrap().set_offset(0);
        attributes.object_at(0).unwrap().set_buffer_index(0);

        // Color attribute
        attributes.object_at(1).unwrap().set_format(MTLVertexFormat::Float3);
        attributes.object_at(1).unwrap().set_offset(12);
        attributes.object_at(1).unwrap().set_buffer_index(0);

        // Normal attribute
        attributes.object_at(2).unwrap().set_format(MTLVertexFormat::Float3);
        attributes.object_at(2).unwrap().set_offset(24);
        attributes.object_at(2).unwrap().set_buffer_index(0);

        // Texture coordinates attribute
        attributes.object_at(3).unwrap().set_format(MTLVertexFormat::Float2);
        attributes.object_at(3).unwrap().set_offset(36);
        attributes.object_at(3).unwrap().set_buffer_index(0);

        // Layout (use size of our vertex struct - 44 bytes)
        layouts.object_at(0).unwrap().set_stride(44);
        layouts.object_at(0).unwrap().set_step_rate(1);
        layouts.object_at(0).unwrap().set_step_function(MTLVertexStepFunction::PerVertex);

        pipeline_descriptor.set_vertex_descriptor(Some(&vertex_descriptor));

        let pipeline_state = device
            .new_render_pipeline_state(&pipeline_descriptor)
            .map_err(|e| format!("Failed to create alpha render pipeline: {}", e))?;

        println!("✅ Metal alpha render pipeline created");
        Ok(pipeline_state)
    }

    fn create_sky_pipeline(
        device: &DeviceRef,
        library: &LibraryRef,
    ) -> Result<RenderPipelineState, Box<dyn std::error::Error>> {
        let vertex_function = library
            .get_function("sky_vertex_main", None)
            .map_err(|e| format!("Sky vertex function not found: {}", e))?;

        let fragment_function = library
            .get_function("sky_fragment_main", None)
            .map_err(|e| format!("Sky fragment function not found: {}", e))?;

        let pipeline_descriptor = RenderPipelineDescriptor::new();
        pipeline_descriptor.set_vertex_function(Some(&vertex_function));
        pipeline_descriptor.set_fragment_function(Some(&fragment_function));

        // Configure color attachment (no alpha blending for sky)
        let color_attachments = pipeline_descriptor.color_attachments();
        let color_attachment = color_attachments.object_at(0).unwrap();
        color_attachment.set_pixel_format(MTLPixelFormat::BGRA8Unorm);

        // Configure depth attachment
        pipeline_descriptor.set_depth_attachment_pixel_format(MTLPixelFormat::Depth32Float);

        // Configure vertex descriptor for sky vertices (position only)
        let vertex_descriptor = VertexDescriptor::new();
        let attributes = vertex_descriptor.attributes();
        let layouts = vertex_descriptor.layouts();

        // Position attribute (3D)
        attributes.object_at(0).unwrap().set_format(MTLVertexFormat::Float3);
        attributes.object_at(0).unwrap().set_offset(0);
        attributes.object_at(0).unwrap().set_buffer_index(0);

        // Layout (only position - 12 bytes)
        layouts.object_at(0).unwrap().set_stride(12);
        layouts.object_at(0).unwrap().set_step_rate(1);
        layouts.object_at(0).unwrap().set_step_function(MTLVertexStepFunction::PerVertex);

        pipeline_descriptor.set_vertex_descriptor(Some(&vertex_descriptor));

        let pipeline_state = device
            .new_render_pipeline_state(&pipeline_descriptor)
            .map_err(|e| format!("Failed to create sky render pipeline: {}", e))?;

        println!("✅ Metal sky render pipeline created");
        Ok(pipeline_state)
    }

    fn create_celestial_pipeline(
        device: &DeviceRef,
        library: &LibraryRef,
    ) -> Result<RenderPipelineState, Box<dyn std::error::Error>> {
        let vertex_function = library
            .get_function("celestial_vertex_main", None)
            .map_err(|e| format!("Celestial vertex function not found: {}", e))?;

        let fragment_function = library
            .get_function("celestial_fragment_main", None)
            .map_err(|e| format!("Celestial fragment function not found: {}", e))?;

        let pipeline_descriptor = RenderPipelineDescriptor::new();
        pipeline_descriptor.set_vertex_function(Some(&vertex_function));
        pipeline_descriptor.set_fragment_function(Some(&fragment_function));

        // Configure color attachment with alpha blending for celestial glow
        let color_attachments = pipeline_descriptor.color_attachments();
        let color_attachment = color_attachments.object_at(0).unwrap();
        color_attachment.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
        color_attachment.set_blending_enabled(true);
        color_attachment.set_source_rgb_blend_factor(MTLBlendFactor::SourceAlpha);
        color_attachment.set_destination_rgb_blend_factor(MTLBlendFactor::OneMinusSourceAlpha);
        color_attachment.set_rgb_blend_operation(MTLBlendOperation::Add);
        color_attachment.set_source_alpha_blend_factor(MTLBlendFactor::One);
        color_attachment.set_destination_alpha_blend_factor(MTLBlendFactor::OneMinusSourceAlpha);
        color_attachment.set_alpha_blend_operation(MTLBlendOperation::Add);

        // Configure depth attachment
        pipeline_descriptor.set_depth_attachment_pixel_format(MTLPixelFormat::Depth32Float);

        // Configure vertex descriptor for celestial vertices (position + UV)
        let vertex_descriptor = VertexDescriptor::new();
        let attributes = vertex_descriptor.attributes();
        let layouts = vertex_descriptor.layouts();

        // Position attribute (3D)
        attributes.object_at(0).unwrap().set_format(MTLVertexFormat::Float3);
        attributes.object_at(0).unwrap().set_offset(0);
        attributes.object_at(0).unwrap().set_buffer_index(0);

        // UV coordinates attribute
        attributes.object_at(1).unwrap().set_format(MTLVertexFormat::Float2);
        attributes.object_at(1).unwrap().set_offset(12);
        attributes.object_at(1).unwrap().set_buffer_index(0);

        // Layout (position + UV = 20 bytes)
        layouts.object_at(0).unwrap().set_stride(20);
        layouts.object_at(0).unwrap().set_step_rate(1);
        layouts.object_at(0).unwrap().set_step_function(MTLVertexStepFunction::PerVertex);

        pipeline_descriptor.set_vertex_descriptor(Some(&vertex_descriptor));

        let pipeline_state = device
            .new_render_pipeline_state(&pipeline_descriptor)
            .map_err(|e| format!("Failed to create celestial render pipeline: {}", e))?;

        println!("✅ Metal celestial render pipeline created");
        Ok(pipeline_state)
    }

    fn create_sky_cube() -> Mesh {
        let mut mesh = Mesh::new();

        // Create a simple cube with 8 vertices
        let positions = [
            // Back face
            [-1.0, -1.0, -1.0], [1.0, -1.0, -1.0], [1.0, 1.0, -1.0], [-1.0, 1.0, -1.0],
            // Front face
            [-1.0, -1.0, 1.0], [1.0, -1.0, 1.0], [1.0, 1.0, 1.0], [-1.0, 1.0, 1.0],
        ];

        // Create vertices with only position data (no color, normal, or UV)
        for pos in positions.iter() {
            mesh.vertices.push(super::mesh::Vertex {
                position: *pos,
                color: [0.0, 0.0, 0.0], // Unused for sky
                normal: [0.0, 0.0, 0.0], // Unused for sky
                tex_coords: [0.0, 0.0], // Unused for sky
            });
        }

        // Define cube faces (12 triangles)
        let indices = [
            // Back face
            0, 1, 2, 0, 2, 3,
            // Front face
            4, 6, 5, 4, 7, 6,
            // Left face
            4, 0, 3, 4, 3, 7,
            // Right face
            1, 5, 6, 1, 6, 2,
            // Bottom face
            4, 5, 1, 4, 1, 0,
            // Top face
            3, 2, 6, 3, 6, 7,
        ];

        mesh.indices.extend_from_slice(&indices);
        mesh.vertex_count = mesh.vertices.len();
        mesh.index_count = mesh.indices.len();

        mesh
    }

    fn create_sphere_mesh(subdivisions: usize) -> Mesh {
        let mut mesh = Mesh::new();

        // Generate UV sphere with specified subdivisions
        let rings = subdivisions;
        let sectors = subdivisions;

        // Generate vertices
        for i in 0..=rings {
            let v = i as f32 / rings as f32;
            let phi = v * std::f32::consts::PI;

            for j in 0..=sectors {
                let u = j as f32 / sectors as f32;
                let theta = u * 2.0 * std::f32::consts::PI;

                let x = phi.sin() * theta.cos();
                let y = phi.cos();
                let z = phi.sin() * theta.sin();

                mesh.vertices.push(super::mesh::Vertex {
                    position: [x, y, z],
                    color: [1.0, 1.0, 1.0], // White base color
                    normal: [x, y, z], // For sphere, normal = position
                    tex_coords: [u, v],
                });
            }
        }

        // Generate indices for triangles
        for i in 0..rings {
            for j in 0..sectors {
                let first = i * (sectors + 1) + j;
                let second = first + sectors + 1;

                // First triangle
                mesh.indices.push(first as u32);
                mesh.indices.push(second as u32);
                mesh.indices.push((first + 1) as u32);

                // Second triangle
                mesh.indices.push(second as u32);
                mesh.indices.push((second + 1) as u32);
                mesh.indices.push((first + 1) as u32);
            }
        }

        mesh.index_count = mesh.indices.len();
        mesh
    }

    fn create_depth_stencil_state(device: &DeviceRef) -> DepthStencilState {
        let depth_stencil_descriptor = DepthStencilDescriptor::new();
        depth_stencil_descriptor.set_depth_compare_function(MTLCompareFunction::Less);
        depth_stencil_descriptor.set_depth_write_enabled(true);

        device.new_depth_stencil_state(&depth_stencil_descriptor)
    }

    fn create_ui_pipeline(
        device: &DeviceRef,
        library: &LibraryRef,
    ) -> Result<RenderPipelineState, Box<dyn std::error::Error>> {
        let vertex_function = library
            .get_function("ui_vertex_main", None)
            .map_err(|e| format!("UI Vertex function not found: {}", e))?;

        let fragment_function = library
            .get_function("ui_fragment_main", None)
            .map_err(|e| format!("UI Fragment function not found: {}", e))?;

        let pipeline_descriptor = RenderPipelineDescriptor::new();
        pipeline_descriptor.set_vertex_function(Some(&vertex_function));
        pipeline_descriptor.set_fragment_function(Some(&fragment_function));

        // Configure color attachment for alpha blending
        let color_attachments = pipeline_descriptor.color_attachments();
        let color_attachment = color_attachments.object_at(0).unwrap();
        color_attachment.set_pixel_format(MTLPixelFormat::BGRA8Unorm);

        // Enable alpha blending for UI
        color_attachment.set_blending_enabled(true);
        color_attachment.set_source_rgb_blend_factor(MTLBlendFactor::SourceAlpha);
        color_attachment.set_destination_rgb_blend_factor(MTLBlendFactor::OneMinusSourceAlpha);
        color_attachment.set_rgb_blend_operation(MTLBlendOperation::Add);
        color_attachment.set_source_alpha_blend_factor(MTLBlendFactor::One);
        color_attachment.set_destination_alpha_blend_factor(MTLBlendFactor::OneMinusSourceAlpha);
        color_attachment.set_alpha_blend_operation(MTLBlendOperation::Add);

        // Configure vertex descriptor for ImGui vertices
        let vertex_descriptor = VertexDescriptor::new();
        let attributes = vertex_descriptor.attributes();
        let layouts = vertex_descriptor.layouts();

        // Position attribute (2D)
        attributes.object_at(0).unwrap().set_format(MTLVertexFormat::Float2);
        attributes.object_at(0).unwrap().set_offset(0);
        attributes.object_at(0).unwrap().set_buffer_index(0);

        // Texture coordinates attribute
        attributes.object_at(1).unwrap().set_format(MTLVertexFormat::Float2);
        attributes.object_at(1).unwrap().set_offset(8);
        attributes.object_at(1).unwrap().set_buffer_index(0);

        // Color attribute (RGBA)
        attributes.object_at(2).unwrap().set_format(MTLVertexFormat::UChar4Normalized);
        attributes.object_at(2).unwrap().set_offset(16);
        attributes.object_at(2).unwrap().set_buffer_index(0);

        // Layout - ImGui vertex size is 20 bytes (pos:8 + uv:8 + col:4)
        layouts.object_at(0).unwrap().set_stride(20);
        layouts.object_at(0).unwrap().set_step_rate(1);
        layouts.object_at(0).unwrap().set_step_function(MTLVertexStepFunction::PerVertex);

        pipeline_descriptor.set_vertex_descriptor(Some(&vertex_descriptor));

        let pipeline_state = device
            .new_render_pipeline_state(&pipeline_descriptor)
            .map_err(|e| format!("Failed to create UI render pipeline: {}", e))?;

        println!("✅ UI Metal render pipeline created");
        Ok(pipeline_state)
    }

    pub fn resize(&mut self, size: CGSize) {
        self.drawable_size = size;
        // Set drawable size using message send to avoid type issues
        unsafe {
            let _: () = msg_send![self.layer.as_ptr(), setDrawableSize: size];
        }
    }

    pub fn begin_frame(&self) -> bool {
        // For now, just return true to indicate we can render
        // In a real implementation, this would set up the render pass
        true
    }

    pub fn render_frame(&mut self, mesh: &Mesh, camera: &Camera, time: f32, time_of_day: f32) -> bool {
        let frame_start = std::time::Instant::now();
        let command_buffer = self.command_queue.new_command_buffer();

        let drawable = match self.layer.next_drawable() {
            Some(d) => d,
            None => return false,
        };

        // Update uniforms
        self.update_uniforms(camera, time, time_of_day);

        // PBR lighting will be updated outside render frame

        // Create depth texture
        let depth_texture = {
            let descriptor = TextureDescriptor::new();
            descriptor.set_pixel_format(MTLPixelFormat::Depth32Float);
            descriptor.set_width(self.drawable_size.width as u64);
            descriptor.set_height(self.drawable_size.height as u64);
            descriptor.set_usage(MTLTextureUsage::RenderTarget);

            // Apple Silicon optimization: use memoryless storage for TBDR
            if self.device.supports_family(MTLGPUFamily::Apple1) {
                descriptor.set_storage_mode(MTLStorageMode::Memoryless);
            }

            self.device.new_texture(&descriptor)
        };

        let render_pass_descriptor = RenderPassDescriptor::new();

        // Color attachment
        let color_attachment = render_pass_descriptor.color_attachments().object_at(0).unwrap();
        color_attachment.set_texture(Some(drawable.texture()));
        color_attachment.set_load_action(MTLLoadAction::Clear);
        color_attachment.set_clear_color(MTLClearColor::new(0.5, 0.8, 1.0, 1.0)); // Sky blue
        color_attachment.set_store_action(MTLStoreAction::Store);

        // Depth attachment
        let depth_attachment = render_pass_descriptor.depth_attachment().unwrap();
        depth_attachment.set_texture(Some(&depth_texture));
        depth_attachment.set_load_action(MTLLoadAction::Clear);
        depth_attachment.set_clear_depth(1.0);
        depth_attachment.set_store_action(MTLStoreAction::DontCare); // Memoryless optimization

        let encoder = command_buffer.new_render_command_encoder(&render_pass_descriptor);
        encoder.set_render_pipeline_state(&self.render_pipeline);
        encoder.set_depth_stencil_state(&self.depth_stencil_state);

        // Set viewport to match drawable size
        let viewport = MTLViewport {
            originX: 0.0,
            originY: 0.0,
            width: self.drawable_size.width,
            height: self.drawable_size.height,
            znear: 0.0,
            zfar: 1.0,
        };
        encoder.set_viewport(viewport);

        command_buffer.set_label("Main Render Pass");

        // Render mesh
        self.render_mesh(&encoder, mesh);

        // End encoding and present
        encoder.end_encoding();
        command_buffer.present_drawable(&drawable);
        command_buffer.commit();

        // Record performance metrics
        let frame_time = frame_start.elapsed();
        let fps = if frame_time.as_secs_f32() > 0.0 { 1.0 / frame_time.as_secs_f32() } else { 0.0 };

        // Collect batching statistics
        let batching_stats = self.material_batcher.get_stats();
        let batch_count = self.material_batcher.get_batch_count();

        // Create performance metrics
        let metrics = PerformanceMetrics {
            frame_time,
            fps,
            vertex_count: mesh.vertices.len(),
            triangle_count: mesh.indices.len() / 3,
            draw_calls: if batch_count > 0 { batch_count } else { 1 }, // At least 1 draw call
            batching_efficiency: batching_stats.efficiency_percentage(),
            memory_usage_mb: 0.0, // TODO: Implement memory tracking
            gpu_usage_percent: 0.0, // TODO: Implement GPU usage tracking
        };

        // Record the frame metrics
        self.batching_performance_monitor.record_frame(metrics);

        // Record batching statistics if we have batches
        if batch_count > 0 {
            let batch_perf = BatchPerformanceStats {
                material_distributions: std::collections::HashMap::new(), // TODO: Implement material distribution tracking
                batch_sizes: vec![],
                average_batch_size: 0.0,
                total_vertices_saved: batching_stats.vertices_processed,
                total_draw_calls_saved: batching_stats.draw_calls_saved,
                batching_overhead_ms: 0.0, // TODO: Measure batching overhead
            };

            self.batching_performance_monitor.record_batching_stats(batching_stats, batch_perf);
        }

        true
    }

    pub fn render_frame_with_ui(
        &mut self,
        mesh: &Mesh,
        preview_mesh: &Mesh,
        camera: &Camera,
        time: f32,
        time_of_day: f32,
        ui_draw_data: Option<&DrawData>,
    ) -> bool {
        // Update chunk streaming system with current camera position
        self.chunk_streaming_system.update_player_position(camera.eye);
        self.chunk_streaming_system.process_streaming();

        let command_buffer = self.command_queue.new_command_buffer();

        let drawable = match self.layer.next_drawable() {
            Some(d) => d,
            None => return false,
        };

        // Update uniforms
        self.update_uniforms(camera, time, time_of_day);

        // PBR lighting will be updated outside render frame

        // Create depth texture
        let depth_texture = {
            let descriptor = TextureDescriptor::new();
            descriptor.set_pixel_format(MTLPixelFormat::Depth32Float);
            descriptor.set_width(self.drawable_size.width as u64);
            descriptor.set_height(self.drawable_size.height as u64);
            descriptor.set_usage(MTLTextureUsage::RenderTarget);

            // Apple Silicon optimization: use memoryless storage for TBDR
            if self.device.supports_family(MTLGPUFamily::Apple1) {
                descriptor.set_storage_mode(MTLStorageMode::Memoryless);
            }

            self.device.new_texture(&descriptor)
        };

        let render_pass_descriptor = RenderPassDescriptor::new();

        // Color attachment
        let color_attachment = render_pass_descriptor.color_attachments().object_at(0).unwrap();
        color_attachment.set_texture(Some(drawable.texture()));
        color_attachment.set_load_action(MTLLoadAction::Clear);
        color_attachment.set_clear_color(MTLClearColor::new(0.5, 0.8, 1.0, 1.0)); // Sky blue
        color_attachment.set_store_action(MTLStoreAction::Store);

        // Depth attachment
        let depth_attachment = render_pass_descriptor.depth_attachment().unwrap();
        depth_attachment.set_texture(Some(&depth_texture));
        depth_attachment.set_load_action(MTLLoadAction::Clear);
        depth_attachment.set_clear_depth(1.0);
        depth_attachment.set_store_action(MTLStoreAction::DontCare); // Memoryless optimization

        let encoder = command_buffer.new_render_command_encoder(&render_pass_descriptor);

        // Set viewport to match drawable size
        let viewport = MTLViewport {
            originX: 0.0,
            originY: 0.0,
            width: self.drawable_size.width,
            height: self.drawable_size.height,
            znear: 0.0,
            zfar: 1.0,
        };
        encoder.set_viewport(viewport);

        // Render 3D scene first
        command_buffer.set_label("Main Render Pass");

        // Render sky first (behind everything)
        encoder.set_render_pipeline_state(&self.sky_pipeline);
        encoder.set_depth_stencil_state(&self.depth_stencil_state);
        self.render_sky(&encoder);

        // Render celestial bodies (sun/moon) after sky but before terrain
        encoder.set_depth_stencil_state(&self.depth_stencil_state);
        self.render_celestials(&encoder, camera, time_of_day);

        // Render main scene
        encoder.set_render_pipeline_state(&self.render_pipeline);
        encoder.set_depth_stencil_state(&self.depth_stencil_state);

        // Render mesh
        self.render_mesh(&encoder, mesh);

        // Render preview mesh with alpha blending
        if !preview_mesh.vertices.is_empty() {
            // Use alpha pipeline for transparent ghost blocks
            encoder.set_render_pipeline_state(&self.alpha_pipeline);
            self.render_mesh(&encoder, preview_mesh);
        }

        // Render UI overlay if available
        if let Some(draw_data) = ui_draw_data {
            // Get references to avoid borrow checker issues
            let ui_pipeline = self.ui_pipeline.as_ref();
            let ui_uniform_buffer = self.ui_uniform_buffer.as_ref();
            let font_sampler = self.font_sampler.as_ref();
            let font_texture = self.font_texture.as_ref();

            if let (Some(pipeline), Some(uniform_buffer), Some(sampler), Some(texture)) =
                (ui_pipeline, ui_uniform_buffer, font_sampler, font_texture) {

                if let Err(e) = self.render_ui_with_refs(&encoder, draw_data, pipeline, uniform_buffer, sampler, texture) {
                    eprintln!("UI render error: {}", e);
                }
            }
        }

        // End encoding and present
        encoder.end_encoding();
        command_buffer.present_drawable(&drawable);
        command_buffer.commit();

        true
    }

    pub fn update_uniforms(&self, camera: &Camera, time: f32, time_of_day: f32) {
        let view_proj = camera.build_view_projection_matrix();

        // Calculate sun position for dynamic lighting
        let light_pos = self.calculate_sun_light_position(camera, time_of_day);

        // Calculate time-based lighting factors
        let (ambient_factor, light_intensity) = self.calculate_lighting_factors(time_of_day);

        let uniforms = Uniforms {
            view_proj: view_proj.into(),
            view_pos: [camera.eye.x, camera.eye.y, camera.eye.z, 1.0],
            light_pos,
            time,
            ambient_factor,
            light_intensity,
            _padding0: 0.0,
        };

        // Update uniform buffer with zero-copy on unified memory
        let contents = self.uniform_buffer.contents();
        unsafe {
            std::ptr::copy_nonoverlapping(
                &uniforms as *const Uniforms as *const u8,
                contents as *mut u8,
                std::mem::size_of::<Uniforms>(),
            );
        }
    }

    fn calculate_lighting_factors(&self, time_of_day: f32) -> (f32, f32) {
        let day_progress = (time_of_day % 24.0) / 24.0;
        let sun_angle = (day_progress - 0.25) * 2.0 * std::f32::consts::PI;
        let sun_height = sun_angle.sin();

        // Calculate ambient factor based on time of day
        let ambient_factor = if sun_height > 0.0 {
            // Daytime: bright ambient lighting
            0.4 + (sun_height * 0.3) // 0.4 to 0.7 based on sun height
        } else if sun_height > -0.3 {
            // Dawn/dusk transition
            let transition = (sun_height + 0.3) / 0.3;
            0.1 + (transition * 0.3) // 0.1 to 0.4 during twilight
        } else {
            // Night time: low ambient lighting
            0.05 + 0.05 * (1.0 + (-sun_height - 0.3) / 0.7).min(1.0) // 0.05 to 0.1 at night
        };

        // Calculate light intensity based on celestial body visibility
        let light_intensity = if sun_height > -0.1 {
            // Sun is visible
            if sun_height > 0.0 {
                1.0 // Full daylight
            } else {
                0.5 + (sun_height + 0.1) * 5.0 // Fade in as sun rises
            }
        } else {
            // Check moon visibility
            let moon_angle = (day_progress + 0.25) * 2.0 * std::f32::consts::PI;
            let moon_height = moon_angle.sin();

            if moon_height > -0.1 {
                // Moon is visible - soft lighting
                if moon_height > 0.0 {
                    0.3 // Full moonlight
                } else {
                    0.1 + (moon_height + 0.1) * 2.0 // Fade in as moon rises
                }
            } else {
                // No celestial body visible - minimal lighting
                0.05
            }
        };

        (ambient_factor, light_intensity)
    }

    fn calculate_sun_light_position(&self, camera: &Camera, time_of_day: f32) -> [f32; 4] {
        // Same calculation as celestial rendering for consistency
        let day_progress = (time_of_day % 24.0) / 24.0;
        let sun_angle = (day_progress - 0.25) * 2.0 * std::f32::consts::PI;
        let sun_height = sun_angle.sin();

        // For lighting purposes, use a closer radius than visual rendering
        let light_radius = 100.0; // Much closer than the 800.0 used for visual rendering

        if sun_height > -0.1 {
            // Sun is above horizon - use sun position
            let position = [
                camera.eye.x + light_radius * sun_angle.cos(),
                camera.eye.y + light_radius * sun_height,
                camera.eye.z,
                1.0
            ];
            position
        } else {
            // Sun is below horizon - use moon position with reduced intensity
            let moon_angle = (day_progress + 0.25) * 2.0 * std::f32::consts::PI;
            let moon_height = moon_angle.sin();

            if moon_height > -0.1 {
                // Moon is visible - use moon position for soft lighting
                let position = [
                    camera.eye.x + light_radius * moon_angle.cos(),
                    camera.eye.y + light_radius * moon_height,
                    camera.eye.z,
                    0.3 // Reduced intensity for moonlight
                ];
                position
            } else {
                // Both sun and moon below horizon - minimal ambient lighting
                let position = [
                    camera.eye.x,
                    camera.eye.y + 50.0, // High up for even distribution
                    camera.eye.z,
                    0.1 // Very low intensity
                ];
                position
            }
        }
    }

    pub fn render_mesh(&self, encoder: &RenderCommandEncoderRef, mesh: &Mesh) {
        if let (Some(vertex_buffer), Some(index_buffer)) = (&mesh.vertex_buffer, &mesh.index_buffer) {
            encoder.set_vertex_buffer(0, Some(vertex_buffer), 0);
            encoder.set_vertex_buffer(1, Some(&self.uniform_buffer), 0);
            encoder.set_fragment_buffer(1, Some(&self.uniform_buffer), 0);

            // Bind atlas texture and sampler for fragment shader
            if let Some(atlas_texture) = &self.atlas_texture {
                encoder.set_fragment_texture(0, Some(atlas_texture));
            }
            if let Some(atlas_sampler) = &self.atlas_sampler {
                encoder.set_fragment_sampler_state(0, Some(atlas_sampler));
            }

            encoder.draw_indexed_primitives(
                MTLPrimitiveType::Triangle,
                mesh.index_count as u64,
                MTLIndexType::UInt32,
                index_buffer,
                0,
            );
        }
    }

    pub fn render_sky(&self, encoder: &RenderCommandEncoderRef) {
        if let (Some(vertex_buffer), Some(index_buffer)) = (&self.sky_mesh.vertex_buffer, &self.sky_mesh.index_buffer) {
            encoder.set_vertex_buffer(0, Some(vertex_buffer), 0);
            encoder.set_vertex_buffer(1, Some(&self.uniform_buffer), 0);
            encoder.set_fragment_buffer(1, Some(&self.uniform_buffer), 0);

            encoder.draw_indexed_primitives(
                MTLPrimitiveType::Triangle,
                self.sky_mesh.index_count as u64,
                MTLIndexType::UInt32,
                index_buffer,
                0,
            );
        }
    }

    pub fn render_celestials(&self, encoder: &RenderCommandEncoderRef, camera: &Camera, time_of_day: f32) {
        encoder.set_render_pipeline_state(&self.celestial_pipeline);

        // Calculate sun position and visibility
        let day_progress = (time_of_day % 24.0) / 24.0;
        let sun_angle = (day_progress - 0.25) * 2.0 * std::f32::consts::PI;
        let sun_height = sun_angle.sin();

        // Only render sun if it's above horizon
        if sun_height > -0.1 {
            self.render_celestial_body(encoder, camera, time_of_day, 0.0, &self.sun_mesh);
        }

        // Calculate moon position and visibility (opposite to sun)
        let moon_height = -sun_height;

        // Only render moon if it's above horizon
        if moon_height > -0.1 {
            self.render_celestial_body(encoder, camera, time_of_day, 1.0, &self.moon_mesh);
        }
    }

    fn render_celestial_body(
        &self,
        encoder: &RenderCommandEncoderRef,
        camera: &Camera,
        time_of_day: f32,
        celestial_type: f32, // 0.0 = sun, 1.0 = moon
        mesh: &Mesh,
    ) {
        if let (Some(vertex_buffer), Some(index_buffer)) = (&mesh.vertex_buffer, &mesh.index_buffer) {
            // Calculate position based on time of day
            let day_progress = (time_of_day % 24.0) / 24.0;
            let angle = if celestial_type < 0.5 {
                // Sun: rises at 6AM (0.25), peaks at noon (0.5), sets at 6PM (0.75)
                (day_progress - 0.25) * 2.0 * std::f32::consts::PI
            } else {
                // Moon: opposite to sun, peaks at midnight
                (day_progress + 0.25) * 2.0 * std::f32::consts::PI
            };

            // Calculate position on arc across sky
            let radius = 800.0; // Distance from camera
            let position = [
                camera.eye.x + radius * angle.cos(),
                camera.eye.y + radius * angle.sin(),
                camera.eye.z,
            ];

            // Calculate size and color based on celestial body type and time
            let (scale, color, _intensity) = if celestial_type < 0.5 {
                // Sun configuration
                let sun_height = angle.sin();
                let scale = 30.0; // Sun size
                let intensity = if sun_height > 0.0 { 1.0 } else { 0.5 + sun_height }; // Fade near horizon
                let color = [1.0, 0.9, 0.6, intensity]; // Warm yellow-orange
                (scale, color, intensity)
            } else {
                // Moon configuration
                let moon_height = -angle.sin();
                let scale = 25.0; // Moon size (slightly smaller)
                let intensity = if moon_height > 0.0 { 0.8 } else { 0.3 + moon_height }; // Dimmer than sun
                let color = [0.9, 0.9, 1.0, intensity]; // Cool blue-white
                (scale, color, intensity)
            };

            // Create celestial uniforms
            #[repr(C)]
            struct CelestialUniforms {
                view_proj: [[f32; 4]; 4],
                view_pos: [f32; 4],
                celestial_pos: [f32; 4],    // xyz = position, w = scale
                celestial_color: [f32; 4],  // rgb = color, a = intensity
                time: f32,
                celestial_type: f32,
                _padding0: f32,
                _padding1: f32,
            }

            let view_proj = camera.build_view_projection_matrix();
            let celestial_uniforms = CelestialUniforms {
                view_proj: view_proj.into(),
                view_pos: [camera.eye.x, camera.eye.y, camera.eye.z, 1.0],
                celestial_pos: [position[0], position[1], position[2], scale],
                celestial_color: color,
                time: time_of_day,
                celestial_type,
                _padding0: 0.0,
                _padding1: 0.0,
            };

            // Update celestial uniform buffer
            let contents = self.celestial_uniform_buffer.contents();
            unsafe {
                std::ptr::copy_nonoverlapping(
                    &celestial_uniforms as *const CelestialUniforms as *const u8,
                    contents as *mut u8,
                    std::mem::size_of::<CelestialUniforms>(),
                );
            }

            // Set buffers and render
            encoder.set_vertex_buffer(0, Some(vertex_buffer), 0);
            encoder.set_vertex_buffer(1, Some(&self.celestial_uniform_buffer), 0);
            encoder.set_fragment_buffer(1, Some(&self.celestial_uniform_buffer), 0);

            encoder.draw_indexed_primitives(
                MTLPrimitiveType::Triangle,
                mesh.index_count as u64,
                MTLIndexType::UInt32,
                index_buffer,
                0,
            );
        }
    }


    pub fn create_font_texture(&mut self, texture_data: &[u8], width: u32, height: u32) -> Result<u64, Box<dyn std::error::Error>> {
        let descriptor = TextureDescriptor::new();
        descriptor.set_pixel_format(MTLPixelFormat::RGBA8Unorm);
        descriptor.set_width(width as u64);
        descriptor.set_height(height as u64);
        descriptor.set_usage(MTLTextureUsage::ShaderRead);
        descriptor.set_storage_mode(MTLStorageMode::Shared);

        let texture = self.device.new_texture(&descriptor);

        // Upload texture data
        let region = MTLRegion {
            origin: MTLOrigin { x: 0, y: 0, z: 0 },
            size: MTLSize {
                width: width as u64,
                height: height as u64,
                depth: 1,
            },
        };

        texture.replace_region(
            region,
            0, // mipmap level
            texture_data.as_ptr() as *const std::ffi::c_void,
            (width * 4) as u64, // bytes per row (RGBA = 4 bytes per pixel)
        );

        // Store the texture
        self.font_texture = Some(texture);

        // Return a texture ID for ImGui (using a simple counter or hash)
        let texture_id = 1u64; // Simple ID for font texture

        println!("✅ Font texture uploaded: {}x{} pixels, ID: {}", width, height, texture_id);
        Ok(texture_id)
    }

    pub fn create_atlas_texture(&mut self, texture_data: &[u8], width: u32, height: u32) -> Result<(), Box<dyn std::error::Error>> {
        let descriptor = TextureDescriptor::new();
        descriptor.set_pixel_format(MTLPixelFormat::RGBA8Unorm);
        descriptor.set_width(width as u64);
        descriptor.set_height(height as u64);
        descriptor.set_usage(MTLTextureUsage::ShaderRead);
        descriptor.set_storage_mode(MTLStorageMode::Shared);

        let texture = self.device.new_texture(&descriptor);

        // Upload texture data
        let region = MTLRegion {
            origin: MTLOrigin { x: 0, y: 0, z: 0 },
            size: MTLSize {
                width: width as u64,
                height: height as u64,
                depth: 1,
            },
        };

        texture.replace_region(
            region,
            0, // mipmap level
            texture_data.as_ptr() as *const std::ffi::c_void,
            (width * 4) as u64, // bytes per row (RGBA = 4 bytes per pixel)
        );

        // Create sampler for the atlas texture
        let sampler_descriptor = SamplerDescriptor::new();
        sampler_descriptor.set_min_filter(MTLSamplerMinMagFilter::Linear);
        sampler_descriptor.set_mag_filter(MTLSamplerMinMagFilter::Linear);
        sampler_descriptor.set_mip_filter(MTLSamplerMipFilter::NotMipmapped);
        sampler_descriptor.set_address_mode_s(MTLSamplerAddressMode::ClampToEdge);
        sampler_descriptor.set_address_mode_t(MTLSamplerAddressMode::ClampToEdge);
        let sampler = self.device.new_sampler(&sampler_descriptor);

        // Store the texture and sampler
        self.atlas_texture = Some(texture);
        self.atlas_sampler = Some(sampler);

        println!("✅ Atlas texture uploaded: {}x{} pixels", width, height);
        Ok(())
    }

    pub fn initialize_ui(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Create shader library (reuse existing)
        let library = Self::create_shader_library(&self.device)?;

        // Create UI pipeline
        self.ui_pipeline = Some(Self::create_ui_pipeline(&self.device, &library)?);

        // Create UI uniform buffer
        let ui_uniform_buffer = self.device.new_buffer(
            64, // 4x4 matrix = 64 bytes
            MTLResourceOptions::StorageModeShared,
        );
        self.ui_uniform_buffer = Some(ui_uniform_buffer);

        // Create font sampler
        let sampler_descriptor = SamplerDescriptor::new();
        sampler_descriptor.set_min_filter(MTLSamplerMinMagFilter::Linear);
        sampler_descriptor.set_mag_filter(MTLSamplerMinMagFilter::Linear);
        sampler_descriptor.set_address_mode_s(MTLSamplerAddressMode::ClampToEdge);
        sampler_descriptor.set_address_mode_t(MTLSamplerAddressMode::ClampToEdge);

        self.font_sampler = Some(self.device.new_sampler(&sampler_descriptor));

        println!("✅ UI rendering system initialized");
        Ok(())
    }

    pub fn render_ui(
        &mut self,
        encoder: &RenderCommandEncoderRef,
        draw_data: &DrawData,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if draw_data.draw_lists_count() == 0 {
            return Ok(());
        }

        let ui_pipeline = self.ui_pipeline.as_ref()
            .ok_or("UI pipeline not initialized")?;
        let ui_uniform_buffer = self.ui_uniform_buffer.as_ref()
            .ok_or("UI uniform buffer not initialized")?;
        let font_sampler = self.font_sampler.as_ref()
            .ok_or("Font sampler not initialized")?;
        let font_texture = self.font_texture.as_ref()
            .ok_or("Font texture not available")?;

        // Set up orthographic projection matrix
        let width = self.drawable_size.width as f32;
        let height = self.drawable_size.height as f32;

        #[repr(C)]
        struct UIUniforms {
            projection: [[f32; 4]; 4],
        }

        let projection = [
            [2.0 / width, 0.0, 0.0, 0.0],
            [0.0, -2.0 / height, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0, 1.0],
        ];

        let uniforms = UIUniforms { projection };

        // Update uniform buffer
        let contents = ui_uniform_buffer.contents();
        unsafe {
            std::ptr::copy_nonoverlapping(
                &uniforms as *const UIUniforms as *const u8,
                contents as *mut u8,
                std::mem::size_of::<UIUniforms>(),
            );
        }

        // Set pipeline state
        encoder.set_render_pipeline_state(ui_pipeline);
        encoder.set_vertex_buffer(1, Some(ui_uniform_buffer), 0);
        encoder.set_fragment_texture(0, Some(font_texture));
        encoder.set_fragment_sampler_state(0, Some(font_sampler));

        // Render each draw list
        for draw_list in draw_data.draw_lists() {
            self.render_draw_list(encoder, &draw_list)?;
        }

        Ok(())
    }

    fn render_draw_list(
        &mut self,
        encoder: &RenderCommandEncoderRef,
        draw_list: &DrawList,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let vertex_buffer = draw_list.vtx_buffer();
        let index_buffer = draw_list.idx_buffer();

        if vertex_buffer.is_empty() || index_buffer.is_empty() {
            return Ok(());
        }

        // Create or update vertex buffer
        let vertex_data_size = vertex_buffer.len() * std::mem::size_of::<DrawVert>();
        let vertex_buffer_metal = self.device.new_buffer_with_data(
            vertex_buffer.as_ptr() as *const std::ffi::c_void,
            vertex_data_size as u64,
            MTLResourceOptions::StorageModeShared,
        );

        // Create or update index buffer
        let index_data_size = index_buffer.len() * std::mem::size_of::<u16>();
        let index_buffer_metal = self.device.new_buffer_with_data(
            index_buffer.as_ptr() as *const std::ffi::c_void,
            index_data_size as u64,
            MTLResourceOptions::StorageModeShared,
        );

        encoder.set_vertex_buffer(0, Some(&vertex_buffer_metal), 0);

        // Render commands
        let mut idx_offset = 0;
        for cmd in draw_list.commands() {
            match cmd {
                DrawCmd::Elements { count, cmd_params } => {
                    if count > 0 {
                        // Set scissor test
                        let clip_rect = cmd_params.clip_rect;
                        let scissor_rect = MTLScissorRect {
                            x: clip_rect[0].max(0.0) as u64,
                            y: clip_rect[1].max(0.0) as u64,
                            width: (clip_rect[2] - clip_rect[0]).max(0.0) as u64,
                            height: (clip_rect[3] - clip_rect[1]).max(0.0) as u64,
                        };
                        encoder.set_scissor_rect(scissor_rect);

                        encoder.draw_indexed_primitives(
                            MTLPrimitiveType::Triangle,
                            count as u64,
                            MTLIndexType::UInt16,
                            &index_buffer_metal,
                            (idx_offset * std::mem::size_of::<u16>()) as u64,
                        );
                    }
                    idx_offset += count;
                }
                DrawCmd::ResetRenderState => {
                    // Handle render state reset if needed
                }
                DrawCmd::RawCallback { .. } => {
                    // Handle raw callbacks if needed
                }
            }
        }

        Ok(())
    }

    fn render_ui_with_refs(
        &self,
        encoder: &RenderCommandEncoderRef,
        draw_data: &DrawData,
        ui_pipeline: &RenderPipelineState,
        ui_uniform_buffer: &Buffer,
        font_sampler: &SamplerState,
        font_texture: &Texture,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if draw_data.draw_lists_count() == 0 {
            return Ok(());
        }

        // Set up orthographic projection matrix
        let width = self.drawable_size.width as f32;
        let height = self.drawable_size.height as f32;

        #[repr(C)]
        struct UIUniforms {
            projection: [[f32; 4]; 4],
        }

        let projection = [
            [2.0 / width, 0.0, 0.0, 0.0],
            [0.0, -2.0 / height, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0, 1.0],
        ];

        let uniforms = UIUniforms { projection };

        // Update uniform buffer
        let contents = ui_uniform_buffer.contents();
        unsafe {
            std::ptr::copy_nonoverlapping(
                &uniforms as *const UIUniforms as *const u8,
                contents as *mut u8,
                std::mem::size_of::<UIUniforms>(),
            );
        }

        // Set pipeline state
        encoder.set_render_pipeline_state(ui_pipeline);
        encoder.set_vertex_buffer(1, Some(ui_uniform_buffer), 0);
        encoder.set_fragment_texture(0, Some(font_texture));
        encoder.set_fragment_sampler_state(0, Some(font_sampler));

        // Render each draw list
        for draw_list in draw_data.draw_lists() {
            self.render_draw_list_with_device(encoder, &draw_list)?;
        }

        Ok(())
    }

    fn render_draw_list_with_device(
        &self,
        encoder: &RenderCommandEncoderRef,
        draw_list: &DrawList,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let vertex_buffer = draw_list.vtx_buffer();
        let index_buffer = draw_list.idx_buffer();

        if vertex_buffer.is_empty() || index_buffer.is_empty() {
            return Ok(());
        }

        // Create or update vertex buffer
        let vertex_data_size = vertex_buffer.len() * std::mem::size_of::<DrawVert>();
        let vertex_buffer_metal = self.device.new_buffer_with_data(
            vertex_buffer.as_ptr() as *const std::ffi::c_void,
            vertex_data_size as u64,
            MTLResourceOptions::StorageModeShared,
        );

        // Create or update index buffer
        let index_data_size = index_buffer.len() * std::mem::size_of::<u16>();
        let index_buffer_metal = self.device.new_buffer_with_data(
            index_buffer.as_ptr() as *const std::ffi::c_void,
            index_data_size as u64,
            MTLResourceOptions::StorageModeShared,
        );

        encoder.set_vertex_buffer(0, Some(&vertex_buffer_metal), 0);

        // Render commands
        let mut idx_offset = 0;
        for cmd in draw_list.commands() {
            match cmd {
                DrawCmd::Elements { count, cmd_params } => {
                    if count > 0 {
                        // Set scissor test
                        let clip_rect = cmd_params.clip_rect;
                        let scissor_rect = MTLScissorRect {
                            x: clip_rect[0].max(0.0) as u64,
                            y: clip_rect[1].max(0.0) as u64,
                            width: (clip_rect[2] - clip_rect[0]).max(0.0) as u64,
                            height: (clip_rect[3] - clip_rect[1]).max(0.0) as u64,
                        };
                        encoder.set_scissor_rect(scissor_rect);

                        encoder.draw_indexed_primitives(
                            MTLPrimitiveType::Triangle,
                            count as u64,
                            MTLIndexType::UInt16,
                            &index_buffer_metal,
                            (idx_offset * std::mem::size_of::<u16>()) as u64,
                        );
                    }
                    idx_offset += count;
                }
                DrawCmd::ResetRenderState => {
                    // Handle render state reset if needed
                }
                DrawCmd::RawCallback { .. } => {
                    // Handle raw callbacks if needed
                }
            }
        }

        Ok(())
    }

    pub fn get_device(&self) -> &Device {
        &self.device
    }

    // Performance monitoring and error recovery methods
    pub fn start_frame_timing(&mut self) {
        self.last_frame_time = std::time::Instant::now();
    }

    pub fn end_frame_timing(&mut self) {
        let frame_time = self.last_frame_time.elapsed().as_secs_f32();
        self.performance_monitor.record_frame_time(frame_time);

        // Log performance warnings if needed
        if frame_time > 0.033 { // More than 30 FPS threshold
            self.performance_monitor.log_performance_warning();
        }
    }

    pub fn record_draw_call(&mut self, vertex_count: u32, triangle_count: u32) {
        self.performance_monitor.record_draw_call(vertex_count, triangle_count);
    }

    pub fn get_performance_stats(&self) -> (f32, u32, u32, u32) {
        let fps = self.performance_monitor.get_average_fps();
        let (draw_calls, vertices, triangles) = self.performance_monitor.get_frame_stats();
        (fps, draw_calls, vertices, triangles)
    }

    pub fn reset_frame_stats(&mut self) {
        self.performance_monitor.reset_frame_stats();
    }

    pub fn validate_resources(&self) -> MetalResult<()> {
        ErrorRecovery::validate_resource(&self.font_texture, "font_texture")?;
        ErrorRecovery::validate_resource(&self.atlas_texture, "atlas_texture")?;
        ErrorRecovery::validate_resource(&self.ui_pipeline, "ui_pipeline")?;
        Ok(())
    }

    pub fn cleanup_resources(&mut self) {
        log::info!("🧹 Cleaning up Metal renderer resources");

        // Clear optional resources
        self.font_texture = None;
        self.atlas_texture = None;
        self.atlas_sampler = None;
        self.ui_pipeline = None;
        self.ui_vertex_buffer = None;
        self.ui_index_buffer = None;
        self.ui_uniform_buffer = None;
        self.font_sampler = None;

        // Reset performance monitor
        self.performance_monitor = PerformanceMonitor::new();

        log::info!("✅ Metal renderer resources cleaned up");
    }

    // Hierarchical frustum culling methods
    pub fn register_chunk(&mut self, x: i32, y: i32, z: i32) {
        let chunk_id = ChunkId::new(x, y, z);
        self.hierarchical_culler.register_chunk(chunk_id);
    }

    pub fn unregister_chunk(&mut self, x: i32, y: i32, z: i32) {
        let chunk_id = ChunkId::new(x, y, z);
        self.hierarchical_culler.unregister_chunk(chunk_id);
    }

    pub fn update_frustum_culling(&mut self, camera: &Camera) {
        // Create view-projection matrix
        let view_proj = camera.build_view_projection_matrix();

        // Create camera frustum
        let frustum = CameraFrustum::from_view_projection(&view_proj);

        // Get camera position
        let camera_pos = camera.eye;

        // Perform frustum culling
        self.visible_chunks = self.hierarchical_culler.cull_chunks(&frustum, &camera_pos);

        // Update performance statistics
        let stats = self.hierarchical_culler.get_statistics();
        self.performance_monitor.record_culling_stats(
            stats.total_chunks,
            stats.visible_chunks,
            stats.culled_chunks
        );
    }

    /// Update LOD levels for all chunks based on camera distance
    pub fn update_lod_system(&mut self, camera: &Camera) {
        let camera_pos = cgmath::Vector3::new(camera.eye.x, camera.eye.y, camera.eye.z);
        self.lod_system.update_lod_levels(&camera_pos, &self.chunk_positions);

        // Log statistics periodically
        let stats = self.lod_system.get_statistics();
        stats.log_if_significant();
    }

    /// Register a chunk position for LOD calculations
    pub fn register_chunk_position(&mut self, chunk_id: ChunkId, position: cgmath::Vector3<f32>) {
        self.chunk_positions.insert(chunk_id, position);
    }

    /// Unregister a chunk position when chunk is removed
    pub fn unregister_chunk_position(&mut self, chunk_id: ChunkId) {
        self.chunk_positions.remove(&chunk_id);
    }

    /// Get the current LOD level for a chunk
    pub fn get_chunk_lod(&self, chunk_id: &ChunkId) -> Option<LodLevel> {
        self.lod_system.get_chunk_lod(chunk_id)
    }

    /// Get chunks that need LOD updates this frame
    pub fn get_chunks_needing_lod_update(&mut self) -> Vec<ChunkId> {
        self.lod_system.get_chunks_needing_update()
    }

    /// Get LOD system statistics for performance monitoring
    pub fn get_lod_statistics(&self) -> LodStatistics {
        self.lod_system.get_statistics()
    }

    /// Update LOD system configuration
    pub fn update_lod_config(&mut self, config: LodConfig) {
        self.lod_system.update_config(config);
        log::info!("📊 LOD system configuration updated");
    }

    /// Enable or disable the LOD system
    pub fn set_lod_enabled(&mut self, enabled: bool) {
        self.lod_system.set_enabled(enabled);
        log::info!("📊 LOD system {}", if enabled { "enabled" } else { "disabled" });
    }

    // Material Batching System Methods

    /// Add a mesh to the material batching system
    pub fn add_mesh_to_batch(&mut self, mesh: &Mesh) {
        self.material_batcher.add_mesh(mesh);
    }

    /// Finalize material batches and create GPU buffers
    pub fn finalize_material_batches(&mut self) -> Result<(), String> {
        self.material_batcher.finalize_batches(&self.device)
    }

    /// Render all material batches with optimized draw calls
    pub fn render_material_batches(&mut self, encoder: &RenderCommandEncoderRef) {
        let batches = self.material_batcher.get_sorted_batches();

        for batch in batches {
            if let (Some(vertex_buffer), Some(index_buffer)) = (&batch.vertex_buffer, &batch.index_buffer) {
                // Set vertex and uniform buffers
                encoder.set_vertex_buffer(0, Some(vertex_buffer), 0);
                encoder.set_vertex_buffer(1, Some(&self.uniform_buffer), 0);
                encoder.set_fragment_buffer(1, Some(&self.uniform_buffer), 0);

                // Bind atlas texture and sampler for fragment shader
                if let Some(atlas_texture) = &self.atlas_texture {
                    encoder.set_fragment_texture(0, Some(atlas_texture));
                }
                if let Some(atlas_sampler) = &self.atlas_sampler {
                    encoder.set_fragment_sampler_state(0, Some(atlas_sampler));
                }

                // Single draw call for entire batch
                encoder.draw_indexed_primitives(
                    MTLPrimitiveType::Triangle,
                    batch.indices.len() as u64,
                    MTLIndexType::UInt32,
                    index_buffer,
                    0,
                );

                // Record performance stats
                self.performance_monitor.record_draw_call(
                    batch.vertices.len() as u32,
                    batch.triangle_count() as u32
                );
            }
        }
    }

    /// Clear material batches for the next frame
    pub fn clear_material_batches(&mut self) {
        self.material_batcher.clear();
    }

    /// Get material batching statistics
    pub fn get_batching_stats(&self) -> &BatchingStats {
        self.material_batcher.get_stats()
    }

    /// Get the number of draw calls saved by batching
    pub fn get_draw_calls_saved(&self) -> usize {
        self.material_batcher.get_draw_calls_saved()
    }

    // Performance Monitoring System Methods

    /// Get the batching performance monitor
    pub fn get_batching_performance_monitor(&self) -> &BatchingPerformanceMonitor {
        &self.batching_performance_monitor
    }

    /// Get current average FPS
    pub fn get_average_fps(&self) -> f32 {
        self.batching_performance_monitor.get_average_fps()
    }

    /// Get current frame time statistics (min, avg, max)
    pub fn get_frame_time_stats(&self) -> (std::time::Duration, std::time::Duration, std::time::Duration) {
        self.batching_performance_monitor.get_frame_time_stats()
    }

    /// Get performance alerts
    pub fn get_performance_alerts(&self) -> &[crate::performance_monitor::PerformanceAlert] {
        self.batching_performance_monitor.get_alerts()
    }

    /// Get batching summary statistics
    pub fn get_batching_summary(&self) -> Option<crate::performance_monitor::BatchingSummary> {
        self.batching_performance_monitor.get_batching_summary()
    }

    /// Update performance monitoring configuration
    pub fn update_performance_config(&mut self, config: MonitorConfig) {
        self.batching_performance_monitor.update_config(config);
        log::info!("📈 Performance monitoring configuration updated");
    }

    /// Enable or disable performance monitoring
    pub fn set_performance_monitoring_enabled(&mut self, enabled: bool) {
        self.batching_performance_monitor.set_enabled(enabled);
        log::info!("📈 Performance monitoring {}", if enabled { "enabled" } else { "disabled" });
    }

    pub fn get_visible_chunks(&self) -> &[ChunkId] {
        &self.visible_chunks
    }

    pub fn should_render_chunk(&self, x: i32, y: i32, z: i32) -> bool {
        let chunk_id = ChunkId::new(x, y, z);
        self.visible_chunks.contains(&chunk_id)
    }

    pub fn get_culling_statistics(&self) -> crate::culling::CullingStatistics {
        self.hierarchical_culler.get_statistics()
    }

    pub fn rebuild_culling_octree(&mut self) {
        self.hierarchical_culler.rebuild_octree();
    }

    // Dynamic Texture Atlas System Methods

    /// Get UV coordinates for a material type
    pub fn get_material_uv(&mut self, material_type: MaterialType) -> Option<AtlasUV> {
        self.dynamic_texture_atlas.get_material_uv(material_type)
    }

    /// Get Metal texture for a material type
    pub fn get_material_texture(&self, material_type: MaterialType) -> Option<&Texture> {
        self.dynamic_texture_atlas.get_material_texture(material_type)
    }

    /// Get all atlas textures for GPU binding
    pub fn get_atlas_textures(&self) -> Vec<&Texture> {
        self.dynamic_texture_atlas.get_all_textures()
    }

    /// Get texture atlas performance statistics
    pub fn get_atlas_stats(&self) -> &AtlasStats {
        self.dynamic_texture_atlas.get_stats()
    }

    /// Preload common material textures
    pub fn preload_atlas_materials(&mut self) -> Result<(), String> {
        self.dynamic_texture_atlas.preload_materials()
    }

    /// Add a custom material texture to the atlas system
    pub fn add_material_to_atlas(&mut self, material_type: MaterialType, size: u32) -> Option<AtlasUV> {
        self.dynamic_texture_atlas.add_material_texture(material_type, size)
    }

    /// Clear all texture atlases
    pub fn clear_texture_atlases(&mut self) {
        self.dynamic_texture_atlas.clear();
        log::info!("🖼️  Texture atlases cleared");
    }

    // ===== PBR LIGHTING METHODS =====

    /// Update PBR lighting system
    pub fn update_lighting(&mut self, delta_time: f32) {
        // Update time of day cycle
        self.pbr_lighting_system.update_time_of_day(delta_time);

        // Update GPU light buffer
        self.update_light_buffer();

        // Update GPU material buffer
        self.update_material_buffer();
    }

    /// Update GPU light buffer with current light data
    fn update_light_buffer(&mut self) {
        let gpu_lights = self.pbr_lighting_system.get_gpu_lights();

        if !gpu_lights.is_empty() {
            let buffer_size = std::mem::size_of::<GPULight>() * gpu_lights.len();
            if buffer_size <= self.light_buffer.length() as usize {
                unsafe {
                    let contents = self.light_buffer.contents() as *mut GPULight;
                    std::ptr::copy_nonoverlapping(gpu_lights.as_ptr(), contents, gpu_lights.len());
                }
            } else {
                log::warn!("⚠️  Too many lights for buffer ({}), truncating to fit", gpu_lights.len());
                let max_lights = self.light_buffer.length() as usize / std::mem::size_of::<GPULight>();
                unsafe {
                    let contents = self.light_buffer.contents() as *mut GPULight;
                    std::ptr::copy_nonoverlapping(gpu_lights.as_ptr(), contents, max_lights);
                }
            }
        }
    }

    /// Update GPU material buffer with current material data
    fn update_material_buffer(&mut self) {
        let materials = self.pbr_lighting_system.material_library.get_all_materials();
        let mut material_array = Vec::new();

        // Convert materials to array in consistent order
        for material_type in [
            MaterialType::Earth, MaterialType::Stone, MaterialType::Water,
            MaterialType::Grass, MaterialType::Sand, MaterialType::Wood,
            MaterialType::Crystal, MaterialType::Lava, MaterialType::Air
        ] {
            if let Some(material) = materials.get(&material_type) {
                material_array.push(*material);
            } else {
                material_array.push(PBRMaterial::default());
            }
        }

        if !material_array.is_empty() {
            let buffer_size = std::mem::size_of::<PBRMaterial>() * material_array.len();
            if buffer_size <= self.material_buffer.length() as usize {
                unsafe {
                    let contents = self.material_buffer.contents() as *mut PBRMaterial;
                    std::ptr::copy_nonoverlapping(material_array.as_ptr(), contents, material_array.len());
                }
            }
        }
    }

    /// Add a light to the PBR system
    pub fn add_light(&mut self, light: Light) -> usize {
        self.pbr_lighting_system.add_light(light)
    }

    /// Remove a light from the PBR system
    pub fn remove_light(&mut self, index: usize) {
        self.pbr_lighting_system.remove_light(index);
    }

    /// Set weather intensity (0.0 = clear, 1.0 = storm)
    pub fn set_weather(&mut self, intensity: f32) {
        self.pbr_lighting_system.set_weather(intensity);
    }

    /// Get current time of day (0.0 = midnight, 0.5 = noon, 1.0 = midnight)
    pub fn get_time_of_day(&self) -> f32 {
        self.pbr_lighting_system.time_of_day
    }

    /// Set time of day manually
    pub fn set_time_of_day(&mut self, time: f32) {
        self.pbr_lighting_system.time_of_day = time.clamp(0.0, 1.0);
    }

    /// Get PBR material for a voxel type
    pub fn get_pbr_material(&self, material_type: MaterialType) -> PBRMaterial {
        self.pbr_lighting_system.get_material(material_type)
    }

    /// Update a PBR material
    pub fn update_pbr_material(&mut self, material_type: MaterialType, material: PBRMaterial) {
        self.pbr_lighting_system.update_material(material_type, material);
    }

    /// Get current environment lighting settings
    pub fn get_environment_lighting(&self) -> &EnvironmentLighting {
        &self.pbr_lighting_system.environment
    }

    /// Enable/disable global illumination
    pub fn set_global_illumination(&mut self, enabled: bool) {
        self.pbr_lighting_system.global_illumination = enabled;
        log::info!("🌍 Global illumination: {}", if enabled { "enabled" } else { "disabled" });
    }

    /// Enable/disable screen space reflections
    pub fn set_screen_space_reflections(&mut self, enabled: bool) {
        self.pbr_lighting_system.screen_space_reflections = enabled;
        log::info!("🪞 Screen space reflections: {}", if enabled { "enabled" } else { "disabled" });
    }

    /// Enable/disable bloom effect
    pub fn set_bloom(&mut self, enabled: bool) {
        self.pbr_lighting_system.bloom_enabled = enabled;
        log::info!("✨ Bloom effect: {}", if enabled { "enabled" } else { "disabled" });
    }

    /// Set tone mapping mode
    pub fn set_tone_mapping(&mut self, mode: ToneMappingMode) {
        log::info!("🎨 Tone mapping: {:?}", mode);
        self.pbr_lighting_system.tone_mapping = mode;
    }

    /// Get lighting debug info for a position
    pub fn debug_lighting(&self, world_pos: cgmath::Point3<f32>, normal: cgmath::Vector3<f32>) -> cgmath::Vector3<f32> {
        self.pbr_lighting_system.calculate_lighting_debug(world_pos, normal)
    }

    /// Get chunk streaming statistics
    pub fn get_streaming_stats(&self) -> &crate::chunk_streaming::StreamingStats {
        self.chunk_streaming_system.get_statistics()
    }

    /// Get streaming configuration
    pub fn get_streaming_config(&self) -> &crate::chunk_streaming::StreamingConfig {
        self.chunk_streaming_system.get_config()
    }

    /// Get streamed chunks for rendering
    pub fn get_streamed_chunks(&self, camera: &Camera) -> Vec<&crate::chunk_streaming::ChunkData> {
        let frustum = camera.get_frustum();
        self.chunk_streaming_system.get_visible_chunks(Some(&frustum))
    }

    /// Force load a specific chunk (for testing)
    pub fn force_load_chunk(&mut self, coords: crate::chunk_streaming::ChunkCoords) {
        self.chunk_streaming_system.force_load_chunk(coords);
    }
}

impl Drop for MetalRenderer {
    fn drop(&mut self) {
        log::info!("🔄 Dropping MetalRenderer - performing cleanup");
        self.cleanup_resources();
        ErrorRecovery::graceful_shutdown();
    }
}