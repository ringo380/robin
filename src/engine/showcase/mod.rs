/// Showcase Module
///
/// Production-ready demo content for Robin Engine

pub mod content_manager;
pub mod interactive_playground;
pub mod engineer_showcase;
pub mod gameplay_showcase;
pub mod collaboration_showcase;
pub mod performance_showcase;
pub mod visual_showcase;
pub mod camera_tours;
pub mod showcase_integration;

#[cfg(test)]
pub mod showcase_test;

pub use content_manager::{ContentManager, ContentType, ShowcaseContent};
pub use camera_tours::{CameraTour, TourPoint, TourController};
pub use showcase_integration::{ShowcaseIntegration, ShowcaseConfig, ShowcaseEvent};