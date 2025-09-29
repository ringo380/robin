/// Content Manager
///
/// Central system for managing and loading showcase content

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use cgmath::{Vector3, Matrix4, Quaternion};
use crate::engine::{
    generation::voxel_system::{VoxelWorld, VoxelType},
    build_mode::{BuildMode, TemplateType},
    graphics::Material,
    animation::{Animation, AnimationClip},
};

/// Type of showcase content
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ContentType {
    InteractivePlayground,
    EngineerBuildMode,
    GameplaySystems,
    CollaborationPreview,
    PerformanceBenchmark,
    VisualShowcase,
}

/// Individual piece of showcase content
pub struct ShowcaseContent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub content_type: ContentType,
    pub world_data: Option<VoxelWorld>,
    pub templates: Vec<BuildTemplate>,
    pub materials: Vec<Material>,
    pub animations: Vec<AnimationClip>,
    pub camera_positions: Vec<CameraPosition>,
    pub metadata: ContentMetadata,
}

/// Build template for showcase structures
pub struct BuildTemplate {
    pub name: String,
    pub description: String,
    pub voxel_data: Vec<(Vector3<i32>, VoxelType)>,
    pub build_time: Duration,
    pub complexity: ComplexityLevel,
    pub tags: Vec<String>,
}

/// Camera position for showcases
#[derive(Clone)]
pub struct CameraPosition {
    pub name: String,
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub fov: f32,
    pub focus_point: Option<Vector3<f32>>,
    pub transition_time: f32,
}

/// Complexity level for templates
#[derive(Debug, Clone, PartialEq)]
pub enum ComplexityLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

/// Metadata for content
pub struct ContentMetadata {
    pub author: String,
    pub version: String,
    pub created_date: String,
    pub last_modified: String,
    pub tags: Vec<String>,
    pub estimated_duration: Duration,
    pub required_features: Vec<String>,
}

/// Main content manager
pub struct ContentManager {
    content_cache: Arc<RwLock<HashMap<String, Arc<ShowcaseContent>>>>,
    loading_queue: Arc<RwLock<Vec<String>>>,
    preload_buffer: Arc<RwLock<HashMap<ContentType, Vec<Arc<ShowcaseContent>>>>>,
    current_content: Option<Arc<ShowcaseContent>>,
    content_registry: HashMap<ContentType, Vec<String>>,
    memory_limit: usize,
    current_memory: usize,
    load_time_budget: Duration,
}

impl ContentManager {
    pub fn new() -> Self {
        let mut registry = HashMap::new();

        // Register content IDs for each type
        registry.insert(ContentType::InteractivePlayground, vec![
            "tutorial_basic_building".to_string(),
            "tutorial_advanced_tools".to_string(),
            "sandbox_starter".to_string(),
            "creative_playground".to_string(),
        ]);

        registry.insert(ContentType::EngineerBuildMode, vec![
            "bridge_template".to_string(),
            "tower_template".to_string(),
            "factory_template".to_string(),
            "castle_template".to_string(),
            "modern_house_template".to_string(),
        ]);

        registry.insert(ContentType::VisualShowcase, vec![
            "lighting_gallery".to_string(),
            "material_showcase".to_string(),
            "weather_effects".to_string(),
            "particle_demo".to_string(),
            "water_simulation".to_string(),
        ]);

        registry.insert(ContentType::PerformanceBenchmark, vec![
            "voxel_stress_test".to_string(),
            "particle_benchmark".to_string(),
            "culling_demo".to_string(),
            "lod_demonstration".to_string(),
            "memory_profiling".to_string(),
        ]);

        Self {
            content_cache: Arc::new(RwLock::new(HashMap::new())),
            loading_queue: Arc::new(RwLock::new(Vec::new())),
            preload_buffer: Arc::new(RwLock::new(HashMap::new())),
            current_content: None,
            content_registry: registry,
            memory_limit: 1024 * 1024 * 512, // 512MB limit
            current_memory: 0,
            load_time_budget: Duration::from_millis(100), // 100ms per frame
        }
    }

    /// Load content by ID
    pub fn load_content(&mut self, content_id: &str) -> Result<Arc<ShowcaseContent>, ContentError> {
        // Check cache first
        if let Ok(cache) = self.content_cache.read() {
            if let Some(content) = cache.get(content_id) {
                self.current_content = Some(content.clone());
                return Ok(content.clone());
            }
        }

        // Load from disk
        let content = self.load_from_disk(content_id)?;
        let content_arc = Arc::new(content);

        // Add to cache
        if let Ok(mut cache) = self.content_cache.write() {
            cache.insert(content_id.to_string(), content_arc.clone());
        }

        self.current_content = Some(content_arc.clone());
        Ok(content_arc)
    }

    /// Preload content for a specific type
    pub fn preload_content_type(&mut self, content_type: ContentType) {
        if let Some(content_ids) = self.content_registry.get(&content_type) {
            let mut queue = self.loading_queue.write().unwrap();
            for id in content_ids {
                if !queue.contains(id) {
                    queue.push(id.clone());
                }
            }
        }
    }

    /// Process loading queue (call each frame)
    pub fn update(&mut self, delta_time: f32) {
        let start_time = Instant::now();

        while start_time.elapsed() < self.load_time_budget {
            let content_id = {
                let mut queue = self.loading_queue.write().unwrap();
                queue.pop()
            };

            if let Some(id) = content_id {
                if let Ok(content) = self.load_from_disk(&id) {
                    let content_arc = Arc::new(content);

                    if let Ok(mut cache) = self.content_cache.write() {
                        cache.insert(id, content_arc);
                    }
                }
            } else {
                break; // Queue is empty
            }
        }

        // Clean up old content if memory limit exceeded
        if self.current_memory > self.memory_limit {
            self.cleanup_cache();
        }
    }

    /// Load content from disk (or generate procedurally)
    fn load_from_disk(&mut self, content_id: &str) -> Result<ShowcaseContent, ContentError> {
        // This would normally load from files, but we'll generate procedurally for now
        match content_id {
            "tutorial_basic_building" => Ok(self.generate_basic_tutorial()),
            "bridge_template" => Ok(self.generate_bridge_template()),
            "lighting_gallery" => Ok(self.generate_lighting_gallery()),
            "voxel_stress_test" => Ok(self.generate_stress_test()),
            _ => Ok(self.generate_default_content(content_id)),
        }
    }

    /// Generate basic building tutorial
    fn generate_basic_tutorial(&self) -> ShowcaseContent {
        let mut world = VoxelWorld::new("Basic Tutorial".to_string(), (50, 50, 50));

        // Create a simple platform
        for x in -10..10 {
            for z in -10..10 {
                world.set_voxel(Vector3::new(x as f32, 0.0, z as f32), VoxelType::Stone);
            }
        }

        // Add guide markers
        for y in 1..5 {
            world.set_voxel(Vector3::new(0.0, y as f32, 0.0), VoxelType::Light);
        }

        ShowcaseContent {
            id: "tutorial_basic_building".to_string(),
            name: "Basic Building Tutorial".to_string(),
            description: "Learn the fundamentals of voxel construction".to_string(),
            content_type: ContentType::InteractivePlayground,
            world_data: Some(world),
            templates: vec![
                BuildTemplate {
                    name: "Simple Wall".to_string(),
                    description: "A basic 5x5 wall structure".to_string(),
                    voxel_data: Self::generate_wall_voxels(5, 5),
                    build_time: Duration::from_secs(10),
                    complexity: ComplexityLevel::Beginner,
                    tags: vec!["wall".to_string(), "basic".to_string()],
                },
            ],
            materials: vec![],
            animations: vec![],
            camera_positions: vec![
                CameraPosition {
                    name: "Overview".to_string(),
                    position: Vector3::new(15.0, 10.0, 15.0),
                    rotation: Quaternion::new(0.924, -0.383, 0.0, 0.0),
                    fov: 60.0,
                    focus_point: Some(Vector3::new(0.0, 2.0, 0.0)),
                    transition_time: 2.0,
                },
            ],
            metadata: ContentMetadata {
                author: "Robin Engine Team".to_string(),
                version: "1.0.0".to_string(),
                created_date: "2025-01-28".to_string(),
                last_modified: "2025-01-28".to_string(),
                tags: vec!["tutorial".to_string(), "beginner".to_string()],
                estimated_duration: Duration::from_secs(300),
                required_features: vec!["basic_building".to_string()],
            },
        }
    }

    /// Generate bridge template
    fn generate_bridge_template(&self) -> ShowcaseContent {
        let mut world = VoxelWorld::new("Bridge Template".to_string(), (100, 50, 50));

        // Create bridge structure
        for x in -20..20 {
            // Bridge deck
            for z in -2..2 {
                world.set_voxel(Vector3::new(x as f32, 10.0, z as f32), VoxelType::Wood);
            }

            // Support pillars
            if x % 10 == 0 {
                for y in 0..10 {
                    world.set_voxel(Vector3::new(x as f32, y as f32, -2.0), VoxelType::Stone);
                    world.set_voxel(Vector3::new(x as f32, y as f32, 2.0), VoxelType::Stone);
                }
            }
        }

        // Add railings
        for x in -20..20 {
            world.set_voxel(Vector3::new(x as f32, 11.0, -2.0), VoxelType::Metal);
            world.set_voxel(Vector3::new(x as f32, 11.0, 2.0), VoxelType::Metal);
        }

        ShowcaseContent {
            id: "bridge_template".to_string(),
            name: "Suspension Bridge".to_string(),
            description: "An engineering marvel spanning great distances".to_string(),
            content_type: ContentType::EngineerBuildMode,
            world_data: Some(world),
            templates: vec![],
            materials: vec![],
            animations: vec![],
            camera_positions: vec![
                CameraPosition {
                    name: "Side View".to_string(),
                    position: Vector3::new(0.0, 15.0, 30.0),
                    rotation: Quaternion::new(0.0, 0.0, 0.0, 1.0),
                    fov: 60.0,
                    focus_point: Some(Vector3::new(0.0, 10.0, 0.0)),
                    transition_time: 3.0,
                },
            ],
            metadata: ContentMetadata {
                author: "Robin Engine Team".to_string(),
                version: "1.0.0".to_string(),
                created_date: "2025-01-28".to_string(),
                last_modified: "2025-01-28".to_string(),
                tags: vec!["engineering".to_string(), "bridge".to_string()],
                estimated_duration: Duration::from_secs(180),
                required_features: vec!["engineering_tools".to_string()],
            },
        }
    }

    /// Generate lighting gallery
    fn generate_lighting_gallery(&self) -> ShowcaseContent {
        let mut world = VoxelWorld::new("Lighting Gallery".to_string(), (80, 40, 80));

        // Create display platforms
        let materials = vec![
            VoxelType::Metal,
            VoxelType::Glass,
            VoxelType::Wood,
            VoxelType::Stone,
        ];

        for (i, material) in materials.iter().enumerate() {
            let x_offset = (i as f32 - 1.5) * 15.0;

            // Platform
            for x in -5..5 {
                for z in -5..5 {
                    world.set_voxel(
                        Vector3::new(x as f32 + x_offset, 0.0, z as f32),
                        VoxelType::Concrete,
                    );
                }
            }

            // Display object
            for y in 1..6 {
                world.set_voxel(
                    Vector3::new(x_offset, y as f32, 0.0),
                    material.clone(),
                );
            }

            // Light source above
            world.set_voxel(
                Vector3::new(x_offset, 10.0, 0.0),
                VoxelType::Light,
            );
        }

        ShowcaseContent {
            id: "lighting_gallery".to_string(),
            name: "Material & Lighting Gallery".to_string(),
            description: "Explore how different materials interact with light".to_string(),
            content_type: ContentType::VisualShowcase,
            world_data: Some(world),
            templates: vec![],
            materials: vec![],
            animations: vec![],
            camera_positions: vec![
                CameraPosition {
                    name: "Gallery Overview".to_string(),
                    position: Vector3::new(0.0, 20.0, 30.0),
                    rotation: Quaternion::new(0.966, -0.259, 0.0, 0.0),
                    fov: 70.0,
                    focus_point: Some(Vector3::new(0.0, 5.0, 0.0)),
                    transition_time: 2.0,
                },
            ],
            metadata: ContentMetadata {
                author: "Robin Engine Team".to_string(),
                version: "1.0.0".to_string(),
                created_date: "2025-01-28".to_string(),
                last_modified: "2025-01-28".to_string(),
                tags: vec!["visual".to_string(), "lighting".to_string(), "materials".to_string()],
                estimated_duration: Duration::from_secs(240),
                required_features: vec!["pbr_rendering".to_string()],
            },
        }
    }

    /// Generate stress test content
    fn generate_stress_test(&self) -> ShowcaseContent {
        let mut world = VoxelWorld::new("Stress Test".to_string(), (200, 100, 200));

        // Generate massive amount of voxels
        for x in -50..50 {
            for z in -50..50 {
                let height = ((x as f32 * 0.1).sin() * 10.0 +
                             (z as f32 * 0.1).cos() * 10.0 + 20.0) as i32;

                for y in 0..height {
                    let voxel_type = match y % 4 {
                        0 => VoxelType::Stone,
                        1 => VoxelType::Wood,
                        2 => VoxelType::Metal,
                        _ => VoxelType::Glass,
                    };

                    world.set_voxel(Vector3::new(x as f32, y as f32, z as f32), voxel_type);
                }
            }
        }

        ShowcaseContent {
            id: "voxel_stress_test".to_string(),
            name: "Voxel Stress Test".to_string(),
            description: "Push the engine to its limits with massive voxel counts".to_string(),
            content_type: ContentType::PerformanceBenchmark,
            world_data: Some(world),
            templates: vec![],
            materials: vec![],
            animations: vec![],
            camera_positions: vec![
                CameraPosition {
                    name: "Aerial View".to_string(),
                    position: Vector3::new(0.0, 80.0, 80.0),
                    rotation: Quaternion::new(0.924, -0.383, 0.0, 0.0),
                    fov: 90.0,
                    focus_point: Some(Vector3::new(0.0, 20.0, 0.0)),
                    transition_time: 3.0,
                },
            ],
            metadata: ContentMetadata {
                author: "Robin Engine Team".to_string(),
                version: "1.0.0".to_string(),
                created_date: "2025-01-28".to_string(),
                last_modified: "2025-01-28".to_string(),
                tags: vec!["performance".to_string(), "stress_test".to_string()],
                estimated_duration: Duration::from_secs(120),
                required_features: vec!["performance_monitoring".to_string()],
            },
        }
    }

    /// Generate default content as fallback
    fn generate_default_content(&self, content_id: &str) -> ShowcaseContent {
        let mut world = VoxelWorld::new(format!("Default: {}", content_id), (50, 50, 50));

        // Simple test pattern
        for x in -5..5 {
            for y in 0..5 {
                for z in -5..5 {
                    if (x + y + z) % 2 == 0 {
                        world.set_voxel(
                            Vector3::new(x as f32, y as f32, z as f32),
                            VoxelType::Stone,
                        );
                    }
                }
            }
        }

        ShowcaseContent {
            id: content_id.to_string(),
            name: format!("Content: {}", content_id),
            description: "Placeholder content".to_string(),
            content_type: ContentType::InteractivePlayground,
            world_data: Some(world),
            templates: vec![],
            materials: vec![],
            animations: vec![],
            camera_positions: vec![],
            metadata: ContentMetadata {
                author: "System".to_string(),
                version: "1.0.0".to_string(),
                created_date: "2025-01-28".to_string(),
                last_modified: "2025-01-28".to_string(),
                tags: vec!["placeholder".to_string()],
                estimated_duration: Duration::from_secs(60),
                required_features: vec![],
            },
        }
    }

    /// Generate wall voxels helper
    fn generate_wall_voxels(width: i32, height: i32) -> Vec<(Vector3<i32>, VoxelType)> {
        let mut voxels = Vec::new();

        for x in 0..width {
            for y in 0..height {
                voxels.push((
                    Vector3::new(x, y, 0),
                    VoxelType::Brick,
                ));
            }
        }

        voxels
    }

    /// Clean up cache to free memory
    fn cleanup_cache(&mut self) {
        // Simple LRU-style cleanup
        if let Ok(mut cache) = self.content_cache.write() {
            // Keep only the most recent content
            if let Some(current) = &self.current_content {
                let current_id = &current.id;
                cache.retain(|id, _| id == current_id);
            } else {
                cache.clear();
            }
        }

        self.current_memory = 0; // Reset memory counter
    }

    /// Get current loaded content
    pub fn get_current_content(&self) -> Option<Arc<ShowcaseContent>> {
        self.current_content.clone()
    }

    /// Get content for a specific type
    pub fn get_content_by_type(&self, content_type: ContentType) -> Vec<String> {
        self.content_registry
            .get(&content_type)
            .cloned()
            .unwrap_or_default()
    }

    /// Clear all cached content
    pub fn clear_cache(&mut self) {
        if let Ok(mut cache) = self.content_cache.write() {
            cache.clear();
        }

        if let Ok(mut buffer) = self.preload_buffer.write() {
            buffer.clear();
        }

        self.current_content = None;
        self.current_memory = 0;
    }
}

/// Content loading error
#[derive(Debug)]
pub enum ContentError {
    NotFound(String),
    LoadFailed(String),
    InvalidFormat(String),
    MemoryExceeded,
}

impl std::fmt::Display for ContentError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ContentError::NotFound(id) => write!(f, "Content not found: {}", id),
            ContentError::LoadFailed(msg) => write!(f, "Failed to load content: {}", msg),
            ContentError::InvalidFormat(msg) => write!(f, "Invalid content format: {}", msg),
            ContentError::MemoryExceeded => write!(f, "Memory limit exceeded"),
        }
    }
}

impl std::error::Error for ContentError {}