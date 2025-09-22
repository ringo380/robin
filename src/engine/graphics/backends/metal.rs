/*!
 * Metal Renderer Backend for Robin Engine
 *
 * Native Metal implementation optimized for macOS (Apple Silicon + Intel).
 * Provides maximum performance by leveraging unified memory architecture
 * and Apple's GPU optimizations.
 */

#[cfg(target_os = "macos")]
pub use self::metal_impl::*;

#[cfg(target_os = "macos")]
mod metal_impl {
    use metal::*;
    use metal::foreign_types::ForeignType;
    use core_graphics::geometry::CGSize;
    use objc::{msg_send, sel, sel_impl};

    use crate::engine::{
        error::{RobinError, RobinResult},
        graphics::Color,
        // Removed unused imports: Camera, Vec2, Vec3
    };
    use crate::engine::graphics::backends::RenderBackend;
    use std::collections::HashMap;

    /// Metal-specific vertex structure optimized for Apple Silicon
    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub struct MetalVertex {
        pub position: [f32; 3],
        pub color: [f32; 3],
        pub normal: [f32; 3],
        pub tex_coords: [f32; 2],
    }

    /// Metal-specific uniforms structure
    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub struct MetalUniforms {
        pub view_projection_matrix: [[f32; 4]; 4],
        pub view_position: [f32; 4],
        pub light_position: [f32; 4],
        pub time: f32,
        pub _padding: [f32; 3],
    }

    /// Metal mesh representation
    pub struct MetalMesh {
        pub vertices: Vec<MetalVertex>,
        pub indices: Vec<u32>,
        pub vertex_buffer: Option<Buffer>,
        pub index_buffer: Option<Buffer>,
        pub vertex_count: usize,
        pub index_count: usize,
    }

    impl MetalMesh {
        pub fn new() -> Self {
            Self {
                vertices: Vec::new(),
                indices: Vec::new(),
                vertex_buffer: None,
                index_buffer: None,
                vertex_count: 0,
                index_count: 0,
            }
        }

        pub fn create_buffers(&mut self, device: &DeviceRef) {
            if !self.vertices.is_empty() {
                self.vertex_buffer = Some(device.new_buffer_with_data(
                    self.vertices.as_ptr() as *const _,
                    (self.vertices.len() * std::mem::size_of::<MetalVertex>()) as u64,
                    MTLResourceOptions::StorageModeShared,
                ));
                self.vertex_count = self.vertices.len();
            }

            if !self.indices.is_empty() {
                self.index_buffer = Some(device.new_buffer_with_data(
                    self.indices.as_ptr() as *const _,
                    (self.indices.len() * std::mem::size_of::<u32>()) as u64,
                    MTLResourceOptions::StorageModeShared,
                ));
                self.index_count = self.indices.len();
            }
        }

        pub fn update_buffers(&mut self, device: &DeviceRef) {
            // Recreate buffers with new data
            self.create_buffers(device);
        }

        pub fn clear(&mut self) {
            self.vertices.clear();
            self.indices.clear();
            self.vertex_count = 0;
            self.index_count = 0;
        }

        pub fn add_quad(&mut self, vertices: [[f32; 3]; 4], color: [f32; 3], normal: [f32; 3], uvs: [[f32; 2]; 4]) {
            let base_index = self.vertices.len() as u32;

            // Add vertices
            for (i, &pos) in vertices.iter().enumerate() {
                self.vertices.push(MetalVertex {
                    position: pos,
                    color,
                    normal,
                    tex_coords: uvs[i],
                });
            }

            // Add indices for two triangles
            self.indices.extend_from_slice(&[
                base_index, base_index + 1, base_index + 2,
                base_index, base_index + 2, base_index + 3,
            ]);
        }
    }

    /// Metal renderer implementation
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

        // Texture management
        texture_cache: HashMap<String, Texture>,
        atlas_texture: Option<Texture>,
        atlas_sampler: Option<SamplerState>,

        // UI support
        ui_pipeline: Option<RenderPipelineState>,
        font_texture: Option<Texture>,
    }

    impl MetalRenderer {
        pub fn new(window_layer: *mut std::ffi::c_void, window_size: (f32, f32)) -> RobinResult<Self> {
            // Create Metal device (automatic selection of best GPU)
            let device = Device::system_default()
                .ok_or_else(|| RobinError::GraphicsInitError("No Metal device found".to_string()))?;

            println!("🚀 Using Metal device: {}", device.name());

            // Create command queue
            let command_queue = device.new_command_queue();

            // Get the Metal layer
            let layer = unsafe { MetalLayer::from_ptr(window_layer as *mut _) };

            // Configure the layer
            layer.set_device(&device);
            layer.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
            layer.set_presents_with_transaction(false);

            // Enable Apple Silicon optimizations
            if device.supports_family(MTLGPUFamily::Apple7) {
                layer.set_framebuffer_only(false); // Allow compute access
                println!("✅ Enabled Apple Silicon GPU optimizations");
            }

            // Create shaders and pipelines
            let library = Self::create_shader_library(&device)?;
            let render_pipeline = Self::create_render_pipeline(&device, &library)?;
            let alpha_pipeline = Self::create_alpha_pipeline(&device, &library)?;
            let sky_pipeline = Self::create_sky_pipeline(&device, &library)?;
            let depth_stencil_state = Self::create_depth_stencil_state(&device);

            // Create uniform buffer
            let uniform_buffer = device.new_buffer(
                std::mem::size_of::<MetalUniforms>() as u64,
                MTLResourceOptions::StorageModeShared,
            );

            // Set drawable size
            let drawable_size = CGSize::new(window_size.0 as f64, window_size.1 as f64);
            unsafe {
                let _: () = msg_send![layer.as_ptr(), setDrawableSize: drawable_size];
            }

            Ok(Self {
                device,
                command_queue,
                layer,
                render_pipeline,
                alpha_pipeline,
                sky_pipeline,
                depth_stencil_state,
                uniform_buffer,
                drawable_size,
                texture_cache: HashMap::new(),
                atlas_texture: None,
                atlas_sampler: None,
                ui_pipeline: None,
                font_texture: None,
            })
        }

        fn create_shader_library(device: &DeviceRef) -> RobinResult<Library> {
            let shader_source = include_str!("../shaders/metal/combined.metal");

            let library = device
                .new_library_with_source(shader_source, &CompileOptions::new())
                .map_err(|e| RobinError::GraphicsInitError(format!("Failed to compile Metal shaders: {}", e)))?;

            println!("✅ Metal shaders compiled successfully");
            Ok(library)
        }

        fn create_render_pipeline(device: &DeviceRef, library: &LibraryRef) -> RobinResult<RenderPipelineState> {
            let vertex_function = library
                .get_function("vertex_main", None)
                .map_err(|e| RobinError::GraphicsInitError(format!("Vertex function not found: {}", e)))?;

            let fragment_function = library
                .get_function("fragment_main", None)
                .map_err(|e| RobinError::GraphicsInitError(format!("Fragment function not found: {}", e)))?;

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

            // Configure vertex descriptor for MetalVertex
            let vertex_descriptor = VertexDescriptor::new();
            let attributes = vertex_descriptor.attributes();
            let layouts = vertex_descriptor.layouts();

            // Position attribute (0)
            attributes.object_at(0).unwrap().set_format(MTLVertexFormat::Float3);
            attributes.object_at(0).unwrap().set_offset(0);
            attributes.object_at(0).unwrap().set_buffer_index(0);

            // Color attribute (1)
            attributes.object_at(1).unwrap().set_format(MTLVertexFormat::Float3);
            attributes.object_at(1).unwrap().set_offset(12);
            attributes.object_at(1).unwrap().set_buffer_index(0);

            // Normal attribute (2)
            attributes.object_at(2).unwrap().set_format(MTLVertexFormat::Float3);
            attributes.object_at(2).unwrap().set_offset(24);
            attributes.object_at(2).unwrap().set_buffer_index(0);

            // Texture coordinates attribute (3)
            attributes.object_at(3).unwrap().set_format(MTLVertexFormat::Float2);
            attributes.object_at(3).unwrap().set_offset(36);
            attributes.object_at(3).unwrap().set_buffer_index(0);

            // Layout (MetalVertex size = 44 bytes)
            layouts.object_at(0).unwrap().set_stride(44);
            layouts.object_at(0).unwrap().set_step_rate(1);
            layouts.object_at(0).unwrap().set_step_function(MTLVertexStepFunction::PerVertex);

            pipeline_descriptor.set_vertex_descriptor(Some(&vertex_descriptor));

            let pipeline_state = device
                .new_render_pipeline_state(&pipeline_descriptor)
                .map_err(|e| RobinError::GraphicsInitError(format!("Failed to create render pipeline: {}", e)))?;

            Ok(pipeline_state)
        }

        fn create_alpha_pipeline(device: &DeviceRef, library: &LibraryRef) -> RobinResult<RenderPipelineState> {
            // Similar to render pipeline but with alpha blending enabled
            let vertex_function = library.get_function("vertex_main", None)
                .map_err(|e| RobinError::GraphicsInitError(format!("Vertex function not found: {}", e)))?;
            let fragment_function = library.get_function("fragment_main", None)
                .map_err(|e| RobinError::GraphicsInitError(format!("Fragment function not found: {}", e)))?;

            let pipeline_descriptor = RenderPipelineDescriptor::new();
            pipeline_descriptor.set_vertex_function(Some(&vertex_function));
            pipeline_descriptor.set_fragment_function(Some(&fragment_function));

            // Configure color attachment with alpha blending
            let color_attachments = pipeline_descriptor.color_attachments();
            let color_attachment = color_attachments.object_at(0).unwrap();
            color_attachment.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
            color_attachment.set_blending_enabled(true);
            color_attachment.set_source_rgb_blend_factor(MTLBlendFactor::SourceAlpha);
            color_attachment.set_destination_rgb_blend_factor(MTLBlendFactor::OneMinusSourceAlpha);

            pipeline_descriptor.set_depth_attachment_pixel_format(MTLPixelFormat::Depth32Float);

            let pipeline_state = device
                .new_render_pipeline_state(&pipeline_descriptor)
                .map_err(|e| RobinError::GraphicsInitError(format!("Failed to create alpha pipeline: {}", e)))?;

            Ok(pipeline_state)
        }

        fn create_sky_pipeline(device: &DeviceRef, library: &LibraryRef) -> RobinResult<RenderPipelineState> {
            let vertex_function = library.get_function("sky_vertex", None)
                .map_err(|e| RobinError::GraphicsInitError(format!("Sky vertex function not found: {}", e)))?;
            let fragment_function = library.get_function("sky_fragment", None)
                .map_err(|e| RobinError::GraphicsInitError(format!("Sky fragment function not found: {}", e)))?;

            let pipeline_descriptor = RenderPipelineDescriptor::new();
            pipeline_descriptor.set_vertex_function(Some(&vertex_function));
            pipeline_descriptor.set_fragment_function(Some(&fragment_function));

            let color_attachments = pipeline_descriptor.color_attachments();
            color_attachments.object_at(0).unwrap().set_pixel_format(MTLPixelFormat::BGRA8Unorm);
            pipeline_descriptor.set_depth_attachment_pixel_format(MTLPixelFormat::Depth32Float);

            let pipeline_state = device
                .new_render_pipeline_state(&pipeline_descriptor)
                .map_err(|e| RobinError::GraphicsInitError(format!("Failed to create sky pipeline: {}", e)))?;

            Ok(pipeline_state)
        }

        fn create_depth_stencil_state(device: &DeviceRef) -> DepthStencilState {
            let depth_stencil_descriptor = DepthStencilDescriptor::new();
            depth_stencil_descriptor.set_depth_compare_function(MTLCompareFunction::Less);
            depth_stencil_descriptor.set_depth_write_enabled(true);
            device.new_depth_stencil_state(&depth_stencil_descriptor)
        }

        pub fn get_device(&self) -> &Device {
            &self.device
        }

        pub fn resize(&mut self, new_size: (f32, f32)) {
            self.drawable_size = CGSize::new(new_size.0 as f64, new_size.1 as f64);
            unsafe {
                let _: () = msg_send![self.layer.as_ptr(), setDrawableSize: self.drawable_size];
            }
        }

        pub fn create_texture(&mut self, data: &[u8], width: u32, height: u32) -> RobinResult<String> {
            let texture_descriptor = TextureDescriptor::new();
            texture_descriptor.set_pixel_format(MTLPixelFormat::RGBA8Unorm);
            texture_descriptor.set_width(width as u64);
            texture_descriptor.set_height(height as u64);
            texture_descriptor.set_usage(MTLTextureUsage::ShaderRead);

            let texture = self.device.new_texture(&texture_descriptor);

            let region = MTLRegion::new_2d(0, 0, width as u64, height as u64);
            texture.replace_region(region, 0, data.as_ptr() as *const _, (width * 4) as u64);

            let texture_id = format!("texture_{}", self.texture_cache.len());
            self.texture_cache.insert(texture_id.clone(), texture);

            Ok(texture_id)
        }
    }

    impl RenderBackend for MetalRenderer {
        fn name(&self) -> &str {
            "Metal"
        }

        fn begin_frame(&mut self) -> RobinResult<()> {
            // Metal frame begins when we get a drawable
            Ok(())
        }

        fn end_frame(&mut self) -> RobinResult<()> {
            // Metal frame ends when we present the drawable
            Ok(())
        }

        fn clear(&mut self, color: Color) -> RobinResult<()> {
            // Clearing is handled in render pass setup
            Ok(())
        }

        fn set_viewport(&mut self, x: u32, y: u32, width: u32, height: u32) -> RobinResult<()> {
            // Viewport is set in render pass
            Ok(())
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod metal_impl {
    use crate::engine::{error::RobinResult, graphics::Color};
    use crate::engine::graphics::backends::RenderBackend;

    pub struct MetalRenderer;

    impl MetalRenderer {
        pub fn new(_window_layer: *mut std::ffi::c_void, _window_size: (f32, f32)) -> RobinResult<Self> {
            Err(crate::engine::error::RobinError::GraphicsInitError(
                "Metal renderer only available on macOS".to_string()
            ))
        }
    }

    impl RenderBackend for MetalRenderer {
        fn name(&self) -> &str { "Metal (Unavailable)" }
        fn begin_frame(&mut self) -> RobinResult<()> { Ok(()) }
        fn end_frame(&mut self) -> RobinResult<()> { Ok(()) }
        fn clear(&mut self, _color: Color) -> RobinResult<()> { Ok(()) }
        fn set_viewport(&mut self, _x: u32, _y: u32, _width: u32, _height: u32) -> RobinResult<()> { Ok(()) }
    }
}