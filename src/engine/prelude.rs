// Re-export commonly used types for game development
pub use crate::engine::{
    Engine,
    GameBuilder,
};

// Selective imports to avoid ambiguous re-exports
pub use crate::engine::graphics::{Renderer, Camera, Mesh, Texture, Color};
pub use crate::engine::input::InputManager;
pub use crate::engine::audio::{AudioSystem, Sound};
pub use crate::engine::math::{Vec2, Vec3, Vec4, Mat4, Transform3D};
pub use crate::engine::assets::{AssetManager, AssetMetadata, AssetType};
pub use crate::engine::scene::{SceneManager, GameObject, Transform as SceneTransform};
pub use crate::engine::animation::{AnimationManager, Animation, EaseType};
pub use crate::engine::physics::{PhysicsWorld, RigidBody, Collider};
pub use crate::engine::ui::{UIManager, UIElement, UIEvent};

pub use std::time::Duration;