// Metal renderer for macOS-native performance
// Optimized for Apple Silicon with unified memory architecture

use metal::*;
use metal::foreign_types::ForeignType;
use core_graphics::geometry::CGSize;
use imgui::{DrawCmd, DrawData, DrawList, DrawVert};
use objc::{msg_send, sel, sel_impl};

use crate::window::NativeWindow;
use super::mesh::Mesh;
use super::shaders::COMBINED_SHADER_SOURCE;
use super::{Uniforms, Camera};

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
}

impl MetalRenderer {
    pub fn new(window: &NativeWindow) -> Result<Self, Box<dyn std::error::Error>> {
        // Create Metal device (automatic selection of best GPU)
        let device = Device::system_default().ok_or("No Metal device found")?;
        println!("🚀 Using Metal device: {}", device.name());

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
        let command_buffer = self.command_queue.new_command_buffer();

        let drawable = match self.layer.next_drawable() {
            Some(d) => d,
            None => return false,
        };

        // Update uniforms
        self.update_uniforms(camera, time, time_of_day);

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

        command_buffer.set_label("Main Render Pass");

        // Render mesh
        self.render_mesh(&encoder, mesh);

        // End encoding and present
        encoder.end_encoding();
        command_buffer.present_drawable(&drawable);
        command_buffer.commit();

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
        let command_buffer = self.command_queue.new_command_buffer();

        let drawable = match self.layer.next_drawable() {
            Some(d) => d,
            None => return false,
        };

        // Update uniforms
        self.update_uniforms(camera, time, time_of_day);

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
}