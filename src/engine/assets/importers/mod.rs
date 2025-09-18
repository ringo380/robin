// Robin Game Engine - Asset Importers
// Comprehensive asset import system for multiple file formats

use crate::engine::error::RobinResult;
use std::path::Path;
use serde::{Serialize, Deserialize};

pub mod gltf_importer;
pub mod fbx_importer;
pub mod obj_importer;
pub mod texture_importer;
pub mod audio_importer;

/// Common asset import trait
pub trait AssetImporter: Send + Sync {
    fn name(&self) -> &'static str;
    fn supported_extensions(&self) -> Vec<&'static str>;
    fn import(&self, path: &Path, options: &ImportOptions) -> RobinResult<ImportedAsset>;
    fn can_import(&self, path: &Path) -> bool;
    fn validate(&self, path: &Path) -> RobinResult<ValidationResult>;
}

/// Import configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportOptions {
    pub quality: QualityLevel,
    pub target_platform: TargetPlatform,
    pub optimize: bool,
    pub generate_lods: bool,
    pub compress_textures: bool,
    pub bake_lighting: bool,
    pub scale_factor: f32,
    pub custom_settings: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityLevel {
    Low,
    Medium,
    High,
    Ultra,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TargetPlatform {
    Desktop,
    Mobile,
    Web,
    Console,
}

/// Imported asset data
#[derive(Debug, Clone)]
pub struct ImportedAsset {
    pub id: String,
    pub name: String,
    pub asset_type: AssetType,
    pub data: AssetData,
    pub metadata: AssetMetadata,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum AssetType {
    Mesh,
    Texture,
    Material,
    Animation,
    Audio,
    Scene,
    Prefab,
}

#[derive(Debug, Clone)]
pub enum AssetData {
    Mesh(MeshData),
    Texture(TextureData),
    Material(MaterialData),
    Animation(AnimationData),
    Audio(AudioData),
    Scene(SceneData),
}

#[derive(Debug, Clone)]
pub struct MeshData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub material_id: Option<String>,
    pub bounding_box: BoundingBox,
    pub lod_levels: Vec<LodLevel>,
}

#[derive(Debug, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub tangent: Option<[f32; 3]>,
    pub color: Option<[f32; 4]>,
}

#[derive(Debug, Clone)]
pub struct TextureData {
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub data: Vec<u8>,
    pub mip_levels: Option<Vec<MipLevel>>,
}

#[derive(Debug, Clone)]
pub enum TextureFormat {
    RGBA8,
    RGB8,
    DXT1,
    DXT5,
    BC7,
    ASTC,
}

#[derive(Debug, Clone)]
pub struct MaterialData {
    pub name: String,
    pub shader: String,
    pub properties: std::collections::HashMap<String, MaterialProperty>,
    pub textures: std::collections::HashMap<String, String>, // slot -> texture_id
}

#[derive(Debug, Clone)]
pub enum MaterialProperty {
    Float(f32),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Bool(bool),
    Int(i32),
}

#[derive(Debug, Clone)]
pub struct AnimationData {
    pub name: String,
    pub duration: f32,
    pub channels: Vec<AnimationChannel>,
    pub events: Vec<AnimationEvent>,
}

#[derive(Debug, Clone)]
pub struct AnimationChannel {
    pub target: String,
    pub property: AnimationProperty,
    pub keyframes: Vec<Keyframe>,
}

#[derive(Debug, Clone)]
pub enum AnimationProperty {
    Translation,
    Rotation,
    Scale,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct Keyframe {
    pub time: f32,
    pub value: KeyframeValue,
    pub interpolation: InterpolationType,
}

#[derive(Debug, Clone)]
pub enum KeyframeValue {
    Vec3([f32; 3]),
    Quaternion([f32; 4]),
    Float(f32),
}

#[derive(Debug, Clone)]
pub enum InterpolationType {
    Linear,
    Step,
    CubicSpline,
}

#[derive(Debug, Clone)]
pub struct AudioData {
    pub sample_rate: u32,
    pub channels: u16,
    pub format: AudioFormat,
    pub data: Vec<u8>,
    pub duration: f32,
    pub loop_points: Option<(u32, u32)>,
}

#[derive(Debug, Clone)]
pub enum AudioFormat {
    PCM16,
    PCM24,
    Float32,
    Vorbis,
    MP3,
}

#[derive(Debug, Clone)]
pub struct SceneData {
    pub name: String,
    pub nodes: Vec<SceneNode>,
    pub cameras: Vec<Camera>,
    pub lights: Vec<Light>,
}

#[derive(Debug, Clone)]
pub struct SceneNode {
    pub id: String,
    pub name: String,
    pub transform: Transform,
    pub mesh_id: Option<String>,
    pub children: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Transform {
    pub translation: [f32; 3],
    pub rotation: [f32; 4], // quaternion
    pub scale: [f32; 3],
}

#[derive(Debug, Clone)]
pub struct Camera {
    pub name: String,
    pub projection: CameraProjection,
    pub transform: Transform,
}

#[derive(Debug, Clone)]
pub enum CameraProjection {
    Perspective { fov: f32, aspect: f32, near: f32, far: f32 },
    Orthographic { left: f32, right: f32, top: f32, bottom: f32, near: f32, far: f32 },
}

#[derive(Debug, Clone)]
pub struct Light {
    pub name: String,
    pub light_type: LightType,
    pub transform: Transform,
    pub intensity: f32,
    pub color: [f32; 3],
}

#[derive(Debug, Clone)]
pub enum LightType {
    Directional,
    Point { range: f32 },
    Spot { inner_angle: f32, outer_angle: f32, range: f32 },
}

#[derive(Debug, Clone)]
pub struct BoundingBox {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

#[derive(Debug, Clone)]
pub struct LodLevel {
    pub distance: f32,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct MipLevel {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct AnimationEvent {
    pub time: f32,
    pub name: String,
    pub data: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AssetMetadata {
    pub file_size: u64,
    pub creation_time: chrono::DateTime<chrono::Utc>,
    pub modification_time: chrono::DateTime<chrono::Utc>,
    pub checksum: String,
    pub import_settings: ImportOptions,
    pub source_file: String,
    pub vertex_count: Option<u32>,
    pub triangle_count: Option<u32>,
    pub texture_memory: Option<u64>,
    pub compression_ratio: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub recommendations: Vec<String>,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            quality: QualityLevel::High,
            target_platform: TargetPlatform::Desktop,
            optimize: true,
            generate_lods: true,
            compress_textures: true,
            bake_lighting: false,
            scale_factor: 1.0,
            custom_settings: std::collections::HashMap::new(),
        }
    }
}

impl ImportOptions {
    pub fn for_platform(platform: TargetPlatform) -> Self {
        match platform {
            TargetPlatform::Mobile => Self {
                quality: QualityLevel::Medium,
                target_platform: platform,
                optimize: true,
                generate_lods: true,
                compress_textures: true,
                scale_factor: 0.5,
                ..Default::default()
            },
            TargetPlatform::Web => Self {
                quality: QualityLevel::Medium,
                target_platform: platform,
                optimize: true,
                generate_lods: false,
                compress_textures: true,
                scale_factor: 0.75,
                ..Default::default()
            },
            _ => Self {
                target_platform: platform,
                ..Default::default()
            }
        }
    }

    pub fn quick_import() -> Self {
        Self {
            optimize: false,
            generate_lods: false,
            compress_textures: false,
            bake_lighting: false,
            ..Default::default()
        }
    }

    pub fn production_ready() -> Self {
        Self {
            quality: QualityLevel::Ultra,
            optimize: true,
            generate_lods: true,
            compress_textures: true,
            bake_lighting: true,
            ..Default::default()
        }
    }
}

/// Asset importer registry
pub struct ImporterRegistry {
    importers: Vec<Box<dyn AssetImporter>>,
}

impl ImporterRegistry {
    pub fn new() -> Self {
        Self {
            importers: Vec::new(),
        }
    }

    pub fn register<T: AssetImporter + 'static>(&mut self, importer: T) {
        self.importers.push(Box::new(importer));
    }

    pub fn find_importer(&self, path: &Path) -> Option<&dyn AssetImporter> {
        self.importers.iter()
            .find(|importer| importer.can_import(path))
            .map(|boxed| boxed.as_ref())
    }

    pub fn get_supported_formats(&self) -> Vec<&'static str> {
        self.importers.iter()
            .flat_map(|importer| importer.supported_extensions())
            .collect()
    }

    pub fn validate_asset(&self, path: &Path) -> RobinResult<ValidationResult> {
        if let Some(importer) = self.find_importer(path) {
            importer.validate(path)
        } else {
            Ok(ValidationResult {
                valid: false,
                warnings: Vec::new(),
                errors: vec!["No suitable importer found".to_string()],
                recommendations: vec!["Check file format and extension".to_string()],
            })
        }
    }
}

impl Default for ImporterRegistry {
    fn default() -> Self {
        let mut registry = Self::new();

        // Register built-in importers
        registry.register(gltf_importer::GltfImporter::new());
        registry.register(fbx_importer::FbxImporter::new());
        registry.register(obj_importer::ObjImporter::new());
        registry.register(texture_importer::TextureImporter::new());
        registry.register(audio_importer::AudioImporter::new());

        registry
    }
}