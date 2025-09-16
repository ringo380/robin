use crate::engine::{
    math::{Vec2, Vec3},
    scene::{Scene, GameObject, Transform},
};
use serde::{Deserialize, Serialize, Serializer, Deserializer};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

/// Serializable representation of a Transform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableTransform {
    pub position: [f32; 3],
    pub rotation: f32,
    pub scale: [f32; 2],
}

impl From<Transform> for SerializableTransform {
    fn from(transform: Transform) -> Self {
        Self {
            position: [transform.position.x, transform.position.y, transform.position.z],
            rotation: transform.rotation,
            scale: [transform.scale.x, transform.scale.y],
        }
    }
}

impl From<SerializableTransform> for Transform {
    fn from(serializable: SerializableTransform) -> Self {
        Self {
            position: Vec3::new(
                serializable.position[0], 
                serializable.position[1], 
                serializable.position[2]
            ),
            rotation: serializable.rotation,
            scale: Vec2::new(serializable.scale[0], serializable.scale[1]),
        }
    }
}

/// Serializable component data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SerializableComponent {
    Sprite {
        texture_path: String,
        width: f32,
        height: f32,
        color: [f32; 4], // RGBA
    },
    RigidBody {
        mass: f32,
        velocity: [f32; 2],
        angular_velocity: f32,
        body_type: String, // "dynamic", "static", "kinematic"
    },
    Collider {
        shape: String, // "circle", "rectangle", "polygon"
        radius: Option<f32>, // For circles
        width: Option<f32>,  // For rectangles
        height: Option<f32>, // For rectangles
        vertices: Option<Vec<[f32; 2]>>, // For polygons
        is_sensor: bool,
    },
    AudioSource {
        audio_path: String,
        volume: f32,
        looping: bool,
        auto_play: bool,
    },
    ParticleSystem {
        emission_rate: f32,
        lifetime: f32,
        start_color: [f32; 4],
        end_color: [f32; 4],
        start_size: f32,
        end_size: f32,
        gravity: [f32; 2],
    },
    Animation {
        animation_name: String,
        current_frame: u32,
        frame_rate: f32,
        looping: bool,
        playing: bool,
    },
    Script {
        script_path: String,
        parameters: HashMap<String, ScriptParameter>,
    },
    Light {
        light_type: String, // "point", "directional", "spot"
        color: [f32; 3],
        intensity: f32,
        range: Option<f32>, // For point and spot lights
        direction: Option<[f32; 3]>, // For directional and spot lights
        cone_angle: Option<f32>, // For spot lights
    },
    Custom {
        component_type: String,
        data: serde_json::Value,
    },
}

/// Script parameter types for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ScriptParameter {
    Float(f32),
    Int(i32),
    Bool(bool),
    String(String),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Color([f32; 4]),
}

/// Serializable representation of a GameObject
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableGameObject {
    pub id: u32,
    pub name: String,
    pub transform: SerializableTransform,
    pub active: bool,
    pub components: Vec<SerializableComponent>,
    pub tags: Vec<String>,
}

/// Serializable representation of a Scene
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SerializableScene {
    pub name: String,
    pub version: String,
    pub metadata: SceneMetadata,
    pub objects: Vec<SerializableGameObject>,
    pub global_settings: GlobalSceneSettings,
}

/// Metadata about the scene
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneMetadata {
    pub created_at: String,
    pub last_modified: String,
    pub author: String,
    pub description: String,
    pub tags: Vec<String>,
    pub engine_version: String,
}

/// Global scene settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSceneSettings {
    pub gravity: [f32; 2],
    pub background_color: [f32; 4],
    pub ambient_light_color: [f32; 3],
    pub ambient_light_intensity: f32,
    pub physics_enabled: bool,
    pub physics_timestep: f32,
    pub render_layers: Vec<String>,
    pub audio_settings: AudioSettings,
}

/// Audio settings for the scene
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettings {
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub doppler_factor: f32,
    pub speed_of_sound: f32,
}

impl Default for SceneMetadata {
    fn default() -> Self {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
        
        Self {
            created_at: now.clone(),
            last_modified: now,
            author: "Robin Engine".to_string(),
            description: "Generated scene".to_string(),
            tags: Vec::new(),
            engine_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

impl Default for GlobalSceneSettings {
    fn default() -> Self {
        Self {
            gravity: [0.0, -9.81],
            background_color: [0.2, 0.3, 0.4, 1.0],
            ambient_light_color: [1.0, 1.0, 1.0],
            ambient_light_intensity: 0.1,
            physics_enabled: true,
            physics_timestep: 1.0 / 60.0,
            render_layers: vec![
                "Background".to_string(),
                "Main".to_string(),
                "Foreground".to_string(),
                "UI".to_string(),
            ],
            audio_settings: AudioSettings::default(),
        }
    }
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            music_volume: 0.7,
            sfx_volume: 1.0,
            doppler_factor: 1.0,
            speed_of_sound: 343.0,
        }
    }
}

/// Scene serialization formats
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SceneFormat {
    Json,
    Toml,
    Binary,
    Yaml,
}

impl SceneFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "json" => Some(SceneFormat::Json),
            "toml" => Some(SceneFormat::Toml),
            "bin" | "scene" => Some(SceneFormat::Binary),
            "yaml" | "yml" => Some(SceneFormat::Yaml),
            _ => None,
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            SceneFormat::Json => "json",
            SceneFormat::Toml => "toml",
            SceneFormat::Binary => "scene",
            SceneFormat::Yaml => "yaml",
        }
    }
}

/// Scene serialization errors
#[derive(Debug)]
pub enum SceneSerializationError {
    IoError(std::io::Error),
    SerializationError(String),
    DeserializationError(String),
    UnsupportedFormat(String),
    InvalidPath(PathBuf),
    ComponentConversionError(String),
    VersionMismatch { expected: String, found: String },
}

impl std::fmt::Display for SceneSerializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SceneSerializationError::IoError(e) => write!(f, "IO error: {}", e),
            SceneSerializationError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            SceneSerializationError::DeserializationError(msg) => write!(f, "Deserialization error: {}", msg),
            SceneSerializationError::UnsupportedFormat(format) => write!(f, "Unsupported format: {}", format),
            SceneSerializationError::InvalidPath(path) => write!(f, "Invalid path: {}", path.display()),
            SceneSerializationError::ComponentConversionError(msg) => write!(f, "Component conversion error: {}", msg),
            SceneSerializationError::VersionMismatch { expected, found } => {
                write!(f, "Version mismatch: expected {}, found {}", expected, found)
            }
        }
    }
}

impl std::error::Error for SceneSerializationError {}

impl From<std::io::Error> for SceneSerializationError {
    fn from(error: std::io::Error) -> Self {
        SceneSerializationError::IoError(error)
    }
}

impl From<serde_json::Error> for SceneSerializationError {
    fn from(error: serde_json::Error) -> Self {
        SceneSerializationError::SerializationError(error.to_string())
    }
}

pub type SceneResult<T> = Result<T, SceneSerializationError>;

/// Main scene serializer/deserializer
pub struct SceneSerializer;

impl SceneSerializer {
    /// Serialize a scene to a file
    pub fn save_scene<P: AsRef<Path>>(
        scene: &SerializableScene, 
        path: P,
        format: Option<SceneFormat>
    ) -> SceneResult<()> {
        let path = path.as_ref();
        
        // Determine format from extension if not specified
        let format = match format {
            Some(fmt) => fmt,
            None => {
                let ext = path.extension()
                    .and_then(|ext| ext.to_str())
                    .ok_or_else(|| SceneSerializationError::InvalidPath(path.to_path_buf()))?;
                
                SceneFormat::from_extension(ext)
                    .ok_or_else(|| SceneSerializationError::UnsupportedFormat(ext.to_string()))?
            }
        };

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Serialize based on format
        let data = match format {
            SceneFormat::Json => {
                serde_json::to_string_pretty(scene)
                    .map_err(|e| SceneSerializationError::SerializationError(e.to_string()))?
            }
            SceneFormat::Toml => {
                toml::to_string(scene)
                    .map_err(|e| SceneSerializationError::SerializationError(e.to_string()))?
            }
            SceneFormat::Yaml => {
                serde_yaml::to_string(scene)
                    .map_err(|e| SceneSerializationError::SerializationError(e.to_string()))?
            }
            SceneFormat::Binary => {
                let binary_data = bincode::serialize(scene)
                    .map_err(|e| SceneSerializationError::SerializationError(e.to_string()))?;
                
                fs::write(path, binary_data)?;
                return Ok(());
            }
        };

        fs::write(path, data)?;
        Ok(())
    }

    /// Load a scene from a file
    pub fn load_scene<P: AsRef<Path>>(path: P) -> SceneResult<SerializableScene> {
        let path = path.as_ref();
        
        if !path.exists() {
            return Err(SceneSerializationError::InvalidPath(path.to_path_buf()));
        }

        // Determine format from extension
        let ext = path.extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| SceneSerializationError::InvalidPath(path.to_path_buf()))?;
        
        let format = SceneFormat::from_extension(ext)
            .ok_or_else(|| SceneSerializationError::UnsupportedFormat(ext.to_string()))?;

        // Load and deserialize based on format
        match format {
            SceneFormat::Json => {
                let data = fs::read_to_string(path)?;
                serde_json::from_str(&data)
                    .map_err(|e| SceneSerializationError::DeserializationError(e.to_string()))
            }
            SceneFormat::Toml => {
                let data = fs::read_to_string(path)?;
                toml::from_str(&data)
                    .map_err(|e| SceneSerializationError::DeserializationError(e.to_string()))
            }
            SceneFormat::Yaml => {
                let data = fs::read_to_string(path)?;
                serde_yaml::from_str(&data)
                    .map_err(|e| SceneSerializationError::DeserializationError(e.to_string()))
            }
            SceneFormat::Binary => {
                let data = fs::read(path)?;
                bincode::deserialize(&data)
                    .map_err(|e| SceneSerializationError::DeserializationError(e.to_string()))
            }
        }
    }

    /// Validate scene version compatibility
    pub fn validate_version(scene: &SerializableScene) -> SceneResult<()> {
        let current_version = env!("CARGO_PKG_VERSION");
        
        if scene.metadata.engine_version != current_version {
            log::warn!("Scene version mismatch: scene={}, engine={}", 
                scene.metadata.engine_version, current_version);
        }
        
        Ok(())
    }

    /// Convert engine Scene to SerializableScene
    pub fn scene_to_serializable(
        scene: &Scene, 
        name: String,
        metadata: Option<SceneMetadata>,
        settings: Option<GlobalSceneSettings>
    ) -> SerializableScene {
        let objects: Vec<SerializableGameObject> = scene
            .objects()
            .map(|obj| SerializableGameObject {
                id: obj.id,
                name: format!("GameObject_{}", obj.id),
                transform: obj.transform.clone().into(),
                active: obj.active,
                components: Vec::new(), // TODO: Convert actual components
                tags: Vec::new(),
            })
            .collect();

        SerializableScene {
            name,
            version: "1.0".to_string(),
            metadata: metadata.unwrap_or_default(),
            objects,
            global_settings: settings.unwrap_or_default(),
        }
    }

    /// Convert SerializableScene to engine Scene
    pub fn serializable_to_scene(serializable: &SerializableScene) -> SceneResult<Scene> {
        let mut scene = Scene::new();
        
        for obj_data in &serializable.objects {
            let mut obj = scene.create_object();
            obj.id = obj_data.id;
            obj.transform = obj_data.transform.clone().into();
            obj.active = obj_data.active;
            
            // TODO: Convert and add components back to the object
            // This would require a component factory or registration system
        }
        
        Ok(scene)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_scene_format_detection() {
        assert_eq!(SceneFormat::from_extension("json"), Some(SceneFormat::Json));
        assert_eq!(SceneFormat::from_extension("toml"), Some(SceneFormat::Toml));
        assert_eq!(SceneFormat::from_extension("yaml"), Some(SceneFormat::Yaml));
        assert_eq!(SceneFormat::from_extension("scene"), Some(SceneFormat::Binary));
        assert_eq!(SceneFormat::from_extension("txt"), None);
    }

    #[test]
    fn test_transform_serialization() {
        let transform = Transform {
            position: Vec3::new(1.0, 2.0, 3.0),
            rotation: 45.0,
            scale: Vec2::new(2.0, 3.0),
        };

        let serializable: SerializableTransform = transform.into();
        assert_eq!(serializable.position, [1.0, 2.0, 3.0]);
        assert_eq!(serializable.rotation, 45.0);
        assert_eq!(serializable.scale, [2.0, 3.0]);

        let restored: Transform = serializable.into();
        assert_eq!(restored.position.x, 1.0);
        assert_eq!(restored.rotation, 45.0);
        assert_eq!(restored.scale.x, 2.0);
    }

    #[test]
    fn test_scene_save_load_json() {
        let temp_dir = TempDir::new().unwrap();
        let scene_path = temp_dir.path().join("test_scene.json");

        let scene = SerializableScene {
            name: "Test Scene".to_string(),
            version: "1.0".to_string(),
            metadata: SceneMetadata::default(),
            objects: vec![SerializableGameObject {
                id: 1,
                name: "Test Object".to_string(),
                transform: SerializableTransform {
                    position: [0.0, 0.0, 0.0],
                    rotation: 0.0,
                    scale: [1.0, 1.0],
                },
                active: true,
                components: Vec::new(),
                tags: vec!["test".to_string()],
            }],
            global_settings: GlobalSceneSettings::default(),
        };

        // Save scene
        SceneSerializer::save_scene(&scene, &scene_path, None).unwrap();
        assert!(scene_path.exists());

        // Load scene
        let loaded_scene = SceneSerializer::load_scene(&scene_path).unwrap();
        assert_eq!(loaded_scene.name, "Test Scene");
        assert_eq!(loaded_scene.objects.len(), 1);
        assert_eq!(loaded_scene.objects[0].name, "Test Object");
    }

    #[test]
    fn test_component_serialization() {
        let sprite_component = SerializableComponent::Sprite {
            texture_path: "player.png".to_string(),
            width: 32.0,
            height: 32.0,
            color: [1.0, 1.0, 1.0, 1.0],
        };

        let json = serde_json::to_string(&sprite_component).unwrap();
        let deserialized: SerializableComponent = serde_json::from_str(&json).unwrap();

        match deserialized {
            SerializableComponent::Sprite { texture_path, width, height, color } => {
                assert_eq!(texture_path, "player.png");
                assert_eq!(width, 32.0);
                assert_eq!(height, 32.0);
                assert_eq!(color, [1.0, 1.0, 1.0, 1.0]);
            }
            _ => panic!("Wrong component type"),
        }
    }
}