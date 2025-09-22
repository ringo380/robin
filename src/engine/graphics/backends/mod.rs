/*!
 * Graphics Backend Abstraction
 *
 * Unified interface for different rendering backends (Metal, wgpu, etc.)
 * Allows runtime selection of optimal renderer for each platform.
 */

use crate::engine::{
    error::RobinResult,
    graphics::Color,
};

pub mod metal;

#[cfg(target_os = "macos")]
pub use metal::MetalRenderer;

/// Core trait for all rendering backends
pub trait RenderBackend {
    /// Get the name of this renderer
    fn name(&self) -> &str;

    /// Begin a new frame
    fn begin_frame(&mut self) -> RobinResult<()>;

    /// End the current frame and present
    fn end_frame(&mut self) -> RobinResult<()>;

    /// Clear the screen with specified color
    fn clear(&mut self, color: Color) -> RobinResult<()>;

    /// Set the viewport dimensions
    fn set_viewport(&mut self, x: u32, y: u32, width: u32, height: u32) -> RobinResult<()>;
}

/// Unified renderer that automatically selects the best backend
pub enum UnifiedRenderer {
    #[cfg(target_os = "macos")]
    Metal(MetalRenderer),
    // TODO: Add wgpu backend for cross-platform support
    // Wgpu(WgpuRenderer),
}

impl UnifiedRenderer {
    /// Create a new unified renderer with automatic backend selection
    pub fn new(window_handle: PlatformWindowHandle, window_size: (f32, f32)) -> RobinResult<Self> {
        #[cfg(target_os = "macos")]
        {
            match window_handle {
                PlatformWindowHandle::MacOS(layer_ptr) => {
                    let metal_renderer = MetalRenderer::new(layer_ptr, window_size)?;
                    println!("✅ Initialized Metal renderer for macOS");
                    Ok(UnifiedRenderer::Metal(metal_renderer))
                }
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            Err(crate::engine::error::RobinError::Initialization(
                "No supported renderer backend for this platform".to_string()
            ))
        }
    }

    /// Get the active renderer's name
    pub fn backend_name(&self) -> &str {
        match self {
            #[cfg(target_os = "macos")]
            UnifiedRenderer::Metal(renderer) => renderer.name(),
        }
    }
}

impl RenderBackend for UnifiedRenderer {
    fn name(&self) -> &str {
        match self {
            #[cfg(target_os = "macos")]
            UnifiedRenderer::Metal(renderer) => renderer.name(),
        }
    }

    fn begin_frame(&mut self) -> RobinResult<()> {
        match self {
            #[cfg(target_os = "macos")]
            UnifiedRenderer::Metal(renderer) => renderer.begin_frame(),
        }
    }

    fn end_frame(&mut self) -> RobinResult<()> {
        match self {
            #[cfg(target_os = "macos")]
            UnifiedRenderer::Metal(renderer) => renderer.end_frame(),
        }
    }

    fn clear(&mut self, color: Color) -> RobinResult<()> {
        match self {
            #[cfg(target_os = "macos")]
            UnifiedRenderer::Metal(renderer) => renderer.clear(color),
        }
    }

    fn set_viewport(&mut self, x: u32, y: u32, width: u32, height: u32) -> RobinResult<()> {
        match self {
            #[cfg(target_os = "macos")]
            UnifiedRenderer::Metal(renderer) => renderer.set_viewport(x, y, width, height),
        }
    }
}

/// Platform-specific window handle
pub enum PlatformWindowHandle {
    #[cfg(target_os = "macos")]
    MacOS(*mut std::ffi::c_void), // Metal layer pointer
}

/// Detect the best rendering backend for current platform
pub fn detect_best_backend() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "Metal"
    }

    #[cfg(not(target_os = "macos"))]
    {
        "wgpu"
    }
}

/// Get platform capabilities
#[derive(Debug, Clone)]
pub struct PlatformCapabilities {
    pub has_metal: bool,
    pub has_apple_silicon: bool,
    pub unified_memory: bool,
    pub max_texture_size: u32,
}

impl PlatformCapabilities {
    pub fn detect() -> Self {
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;

            // Detect Apple Silicon
            let has_apple_silicon = Command::new("uname")
                .arg("-m")
                .output()
                .map(|output| String::from_utf8_lossy(&output.stdout).contains("arm64"))
                .unwrap_or(false);

            Self {
                has_metal: true,
                has_apple_silicon,
                unified_memory: has_apple_silicon, // Apple Silicon has unified memory
                max_texture_size: if has_apple_silicon { 16384 } else { 8192 },
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            Self {
                has_metal: false,
                has_apple_silicon: false,
                unified_memory: false,
                max_texture_size: 4096,
            }
        }
    }
}