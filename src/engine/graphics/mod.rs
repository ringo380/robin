pub mod renderer;
pub mod renderer_3d;
pub mod texture;
pub mod shader;
pub mod camera;
pub mod particles;
pub mod particles_gpu;
pub mod sprite;
pub mod color;
pub mod mesh;
pub mod lighting;
pub mod lod;

pub use renderer::{Renderer, Light, SpriteInstance};
pub use renderer_3d::{Renderer3D, Camera3D, Material, Light as Light3D, Vertex3D, Mesh3D, BoundingBox};
pub use texture::{Texture, TextureManager};
pub use camera::Camera;
pub use particles::{ParticleSystem, ParticleEmitter, ParticleEmitterConfig, Particle};
pub use particles_gpu::{GPUParticleSystem, GPUParticleSystemManager, GPUParticleEmitterConfig, GPUParticle};
pub use sprite::{Sprite, AnimatedSprite, SpriteAnimation, SpriteFrame, SpriteManager, UVRect};
pub use color::Color;
pub use mesh::{Mesh, Vertex};
pub use lighting::{LightingSystem, ShadowRenderer, GlobalIlluminationSystem};
pub use lod::{LODManager, LODGroup, MeshSimplifier};

/// Graphics context for AI and rendering systems
#[derive(Debug)]
pub struct GraphicsContext {
    pub device_info: String,
    pub capabilities: GraphicsCapabilities,
}

#[derive(Debug, Clone)]
pub struct GraphicsCapabilities {
    pub max_texture_size: u32,
    pub supports_compute_shaders: bool,
    pub gpu_memory_mb: u32,
}

impl GraphicsContext {
    pub fn new() -> crate::engine::error::RobinResult<Self> {
        Ok(Self {
            device_info: "Mock Graphics Context".to_string(),
            capabilities: GraphicsCapabilities {
                max_texture_size: 8192,
                supports_compute_shaders: true,
                gpu_memory_mb: 4096,
            },
        })
    }
}

/// Mock graphics context for testing
#[derive(Debug, Clone)]
pub struct MockGraphicsContext {
    pub device_info: String,
    pub capabilities: GraphicsCapabilities,
}

impl MockGraphicsContext {
    pub fn new() -> Self {
        Self {
            device_info: "Mock Graphics Context".to_string(),
            capabilities: GraphicsCapabilities {
                max_texture_size: 8192,
                supports_compute_shaders: true,
                gpu_memory_mb: 4096,
            },
        }
    }
}