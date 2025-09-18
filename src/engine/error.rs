use std::fmt;
use std::error::Error as StdError;
use std::path::PathBuf;

/// Result type alias for Robin Engine operations
pub type RobinResult<T> = Result<T, RobinError>;

/// Comprehensive error type for the Robin Engine
#[derive(Debug, Clone)]
pub enum RobinError {
    // === GRAPHICS ERRORS ===
    GraphicsInitError(String),
    ShaderCompilationError {
        shader_type: String,
        source: String,
        error: String,
    },
    TextureLoadError {
        path: PathBuf,
        reason: String,
    },
    RenderingError(String),
    
    // === AUDIO ERRORS ===
    AudioInitError(String),
    AudioLoadError {
        path: PathBuf,
        format: String,
        reason: String,
    },
    AudioPlaybackError {
        sound_id: String,
        reason: String,
    },
    
    // === PHYSICS ERRORS ===
    PhysicsError(String),
    CollisionDetectionError {
        object1_id: u32,
        object2_id: u32,
        reason: String,
    },
    
    // === ASSET ERRORS ===
    AssetNotFound {
        asset_id: String,
        asset_type: String,
        searched_paths: Vec<PathBuf>,
    },
    DatabaseError(String),
    AssetLoadError {
        asset_id: String,
        path: PathBuf,
        reason: String,
    },
    AssetParseError {
        asset_id: String,
        format: String,
        line: Option<usize>,
        column: Option<usize>,
        message: String,
    },
    AssetBuildError {
        stage: String,
        reason: String,
    },
    AssetImportError {
        source_path: PathBuf,
        reason: String,
    },
    AssetValidationError {
        asset_path: PathBuf,
        validation_errors: Vec<String>,
    },
    HotReloadError(String),
    
    // === SCENE ERRORS ===
    SceneNotFound(String),
    SceneSerializationError {
        scene_name: String,
        format: String,
        reason: String,
    },
    SceneDeserializationError {
        path: PathBuf,
        format: String,
        reason: String,
    },
    GameObjectNotFound {
        scene_name: String,
        object_id: u32,
    },
    ComponentError {
        object_id: u32,
        component_type: String,
        operation: String,
        reason: String,
    },
    
    // === INPUT ERRORS ===
    InputError(String),
    InputDeviceError {
        device_type: String,
        device_id: Option<String>,
        reason: String,
    },
    
    // === UI ERRORS ===
    UIElementNotFound {
        element_id: String,
        context: String,
    },
    UILayoutError {
        element_id: String,
        reason: String,
    },
    UIStyleError {
        property: String,
        value: String,
        reason: String,
    },
    
    // === ANIMATION ERRORS ===
    AnimationError {
        animation_name: String,
        reason: String,
    },
    AnimationSequenceError {
        sequence_name: String,
        frame: usize,
        reason: String,
    },
    
    // === SCRIPTING ERRORS ===
    ScriptError {
        script_path: PathBuf,
        line: Option<usize>,
        reason: String,
    },
    ScriptNotFound(PathBuf),
    ScriptingError {
        message: String,
    },
    ScriptCompilationError {
        script_id: String,
        node_id: Option<String>,
        error: String,
    },
    ScriptExecutionError {
        script_id: String,
        instruction_address: Option<usize>,
        error: String,
    },
    BehaviorTreeError {
        tree_id: String,
        node_id: Option<String>,
        error: String,
    },
    EventSystemError {
        event_type: Option<String>,
        error: String,
    },
    
    // === CONFIGURATION ERRORS ===
    ConfigurationError {
        config_file: PathBuf,
        key: String,
        expected_type: String,
        found_type: String,
    },
    ConfigurationMissing {
        config_file: PathBuf,
        required_key: String,
    },
    
    // === SYSTEM ERRORS ===
    InitializationError {
        subsystem: String,
        reason: String,
    },
    IoError(String),
    DestructionError(String),
    GeneralError(String),
    ResourceExhausted {
        resource_type: String,
        limit: usize,
        requested: usize,
    },
    ThreadError(String),
    
    // === I/O ERRORS ===
    FileNotFound(PathBuf),
    FileAccessDenied(PathBuf),
    DirectoryError {
        path: PathBuf,
        operation: String,
        reason: String,
    },
    FileSaveError {
        path: PathBuf,
        reason: String,
    },
    FileLoadError {
        path: PathBuf,
        reason: String,
    },
    
    // === VALIDATION ERRORS ===
    ValidationError {
        field: String,
        value: String,
        constraint: String,
    },
    RangeError {
        parameter: String,
        value: f64,
        min: f64,
        max: f64,
    },
    
    // === NETWORK ERRORS ===
    NetworkError {
        operation: String,
        endpoint: String,
        reason: String,
    },
    
    // === MULTIPLAYER ERRORS ===
    MultiplayerError(String),
    SessionError(String),
    CollaborationError(String),
    ConflictResolutionError(String),
    SerializationError {
        object_type: String,
        reason: String,
    },
    
    // === GENERIC ERRORS ===
    InvalidOperation {
        operation: String,
        context: String,
        reason: String,
    },
    NotImplemented {
        feature: String,
        planned_version: Option<String>,
    },
    InternalError(String),
    
    // === ADDITIONAL ERROR VARIANTS ===
    BuildError(String),
    PlatformError(String),
    DeploymentError(String),
    GenericError(String),
    PackagingError(String),
    InvalidResource(String),
    BufferError(String),
    InvalidInput(String),
    Custom(String),
    OutOfMemory,
    GPUMemoryError(String),
    GraphicsError(String),
    ComputeError(String),
    TemplateError(String),
    SystemError(String),
    StudyNotFound(String),
    ResourceLimitExceeded(String),
    NoiseError(String),
    UIGenerationError(String),
    ResourceLimit(String),
    QuantumSimulatorNotFound(String),
    QuantumAlgorithmNotFound(String),
    GPUError(String),
    ExportError(String),
    VRSystem(String),
    Story(String),
    AssistanceError(String),
    Other(String),
}

impl fmt::Display for RobinError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Graphics errors
            RobinError::GraphicsInitError(reason) => {
                write!(f, "Graphics initialization failed: {}", reason)
            }
            RobinError::ShaderCompilationError { shader_type, source, error } => {
                write!(f, "Shader compilation failed ({} shader from {}): {}", shader_type, source, error)
            }
            RobinError::TextureLoadError { path, reason } => {
                write!(f, "Failed to load texture '{}': {}", path.display(), reason)
            }
            RobinError::RenderingError(reason) => {
                write!(f, "Rendering error: {}", reason)
            }
            
            // Audio errors
            RobinError::AudioInitError(reason) => {
                write!(f, "Audio system initialization failed: {}", reason)
            }
            RobinError::AudioLoadError { path, format, reason } => {
                write!(f, "Failed to load {} audio file '{}': {}", format, path.display(), reason)
            }
            RobinError::AudioPlaybackError { sound_id, reason } => {
                write!(f, "Audio playback error for '{}': {}", sound_id, reason)
            }
            
            // Physics errors
            RobinError::PhysicsError(reason) => {
                write!(f, "Physics system error: {}", reason)
            }
            RobinError::CollisionDetectionError { object1_id, object2_id, reason } => {
                write!(f, "Collision detection error between objects {} and {}: {}", object1_id, object2_id, reason)
            }
            
            // Asset errors
            RobinError::AssetNotFound { asset_id, asset_type, searched_paths } => {
                write!(f, "{} asset '{}' not found. Searched paths: {:?}", asset_type, asset_id, searched_paths)
            }
            RobinError::DatabaseError(reason) => {
                write!(f, "Database error: {}", reason)
            }
            RobinError::AssetLoadError { asset_id, path, reason } => {
                write!(f, "Failed to load asset '{}' from '{}': {}", asset_id, path.display(), reason)
            }
            RobinError::AssetParseError { asset_id, format, line, column, message } => {
                match (line, column) {
                    (Some(line), Some(col)) => write!(f, "Parse error in {} asset '{}' at line {}, column {}: {}", format, asset_id, line, col, message),
                    (Some(line), None) => write!(f, "Parse error in {} asset '{}' at line {}: {}", format, asset_id, line, message),
                    _ => write!(f, "Parse error in {} asset '{}': {}", format, asset_id, message),
                }
            }
            RobinError::AssetBuildError { stage, reason } => {
                write!(f, "Asset build error at stage '{}': {}", stage, reason)
            }
            RobinError::AssetImportError { source_path, reason } => {
                write!(f, "Asset import error for '{}': {}", source_path.display(), reason)
            }
            RobinError::AssetValidationError { asset_path, validation_errors } => {
                write!(f, "Asset validation failed for '{}': {}", asset_path.display(), validation_errors.join(", "))
            }
            RobinError::HotReloadError(reason) => {
                write!(f, "Hot reload error: {}", reason)
            }
            
            // Scene errors
            RobinError::SceneNotFound(name) => {
                write!(f, "Scene '{}' not found", name)
            }
            RobinError::SceneSerializationError { scene_name, format, reason } => {
                write!(f, "Failed to serialize scene '{}' to {}: {}", scene_name, format, reason)
            }
            RobinError::SceneDeserializationError { path, format, reason } => {
                write!(f, "Failed to deserialize {} scene from '{}': {}", format, path.display(), reason)
            }
            RobinError::GameObjectNotFound { scene_name, object_id } => {
                write!(f, "Game object {} not found in scene '{}'", object_id, scene_name)
            }
            RobinError::ComponentError { object_id, component_type, operation, reason } => {
                write!(f, "Component error: {} operation on {} component of object {}: {}", operation, component_type, object_id, reason)
            }
            
            // Input errors
            RobinError::InputError(reason) => {
                write!(f, "Input system error: {}", reason)
            }
            RobinError::InputDeviceError { device_type, device_id, reason } => {
                match device_id {
                    Some(id) => write!(f, "{} device '{}' error: {}", device_type, id, reason),
                    None => write!(f, "{} device error: {}", device_type, reason),
                }
            }
            
            // UI errors
            RobinError::UIElementNotFound { element_id, context } => {
                write!(f, "UI element '{}' not found in context '{}'", element_id, context)
            }
            RobinError::UILayoutError { element_id, reason } => {
                write!(f, "UI layout error for element '{}': {}", element_id, reason)
            }
            RobinError::UIStyleError { property, value, reason } => {
                write!(f, "UI style error: property '{}' with value '{}': {}", property, value, reason)
            }
            
            // Animation errors
            RobinError::AnimationError { animation_name, reason } => {
                write!(f, "Animation error for '{}': {}", animation_name, reason)
            }
            RobinError::AnimationSequenceError { sequence_name, frame, reason } => {
                write!(f, "Animation sequence error in '{}' at frame {}: {}", sequence_name, frame, reason)
            }
            
            // Scripting errors
            RobinError::ScriptError { script_path, line, reason } => {
                match line {
                    Some(line_num) => write!(f, "Script error in '{}' at line {}: {}", script_path.display(), line_num, reason),
                    None => write!(f, "Script error in '{}': {}", script_path.display(), reason),
                }
            }
            RobinError::ScriptNotFound(path) => {
                write!(f, "Script not found: {}", path.display())
            }
            RobinError::ScriptingError { message } => {
                write!(f, "Scripting error: {}", message)
            }
            RobinError::ScriptCompilationError { script_id, node_id, error } => {
                match node_id {
                    Some(node) => write!(f, "Script compilation error in '{}' at node '{}': {}", script_id, node, error),
                    None => write!(f, "Script compilation error in '{}': {}", script_id, error),
                }
            }
            RobinError::ScriptExecutionError { script_id, instruction_address, error } => {
                match instruction_address {
                    Some(addr) => write!(f, "Script execution error in '{}' at address {}: {}", script_id, addr, error),
                    None => write!(f, "Script execution error in '{}': {}", script_id, error),
                }
            }
            
            // Configuration errors
            RobinError::ConfigurationError { config_file, key, expected_type, found_type } => {
                write!(f, "Configuration error in '{}': key '{}' expected {} but found {}", config_file.display(), key, expected_type, found_type)
            }
            RobinError::ConfigurationMissing { config_file, required_key } => {
                write!(f, "Missing required configuration key '{}' in '{}'", required_key, config_file.display())
            }
            
            // System errors
            RobinError::InitializationError { subsystem, reason } => {
                write!(f, "Failed to initialize {}: {}", subsystem, reason)
            }
            RobinError::ResourceExhausted { resource_type, limit, requested } => {
                write!(f, "{} resource exhausted: requested {} but limit is {}", resource_type, requested, limit)
            }
            RobinError::ThreadError(reason) => {
                write!(f, "Thread error: {}", reason)
            }
            
            // I/O errors
            RobinError::FileNotFound(path) => {
                write!(f, "File not found: {}", path.display())
            }
            RobinError::FileAccessDenied(path) => {
                write!(f, "Access denied to file: {}", path.display())
            }
            RobinError::DirectoryError { path, operation, reason } => {
                write!(f, "Directory {} error on '{}': {}", operation, path.display(), reason)
            }
            RobinError::FileSaveError { path, reason } => {
                write!(f, "Failed to save file '{}': {}", path.display(), reason)
            }
            RobinError::FileLoadError { path, reason } => {
                write!(f, "Failed to load file '{}': {}", path.display(), reason)
            }
            
            // Validation errors
            RobinError::ValidationError { field, value, constraint } => {
                write!(f, "Validation error: field '{}' with value '{}' violates constraint: {}", field, value, constraint)
            }
            RobinError::RangeError { parameter, value, min, max } => {
                write!(f, "Range error: parameter '{}' value {} is outside range [{}, {}]", parameter, value, min, max)
            }
            
            // Network errors
            RobinError::NetworkError { operation, endpoint, reason } => {
                write!(f, "Network error during {} to '{}': {}", operation, endpoint, reason)
            }
            
            // Multiplayer errors
            RobinError::MultiplayerError(reason) => {
                write!(f, "Multiplayer error: {}", reason)
            }
            RobinError::SessionError(reason) => {
                write!(f, "Session error: {}", reason)
            }
            RobinError::CollaborationError(reason) => {
                write!(f, "Collaboration error: {}", reason)
            }
            RobinError::ConflictResolutionError(reason) => {
                write!(f, "Conflict resolution error: {}", reason)
            }
            RobinError::SerializationError { object_type, reason } => {
                write!(f, "Serialization error for {}: {}", object_type, reason)
            }
            
            // Generic errors
            RobinError::InvalidOperation { operation, context, reason } => {
                write!(f, "Invalid operation '{}' in context '{}': {}", operation, context, reason)
            }
            RobinError::NotImplemented { feature, planned_version } => {
                match planned_version {
                    Some(version) => write!(f, "Feature '{}' not yet implemented (planned for version {})", feature, version),
                    None => write!(f, "Feature '{}' not yet implemented", feature),
                }
            }
            RobinError::InternalError(reason) => {
                write!(f, "Internal engine error: {}", reason)
            }
            
            // Additional error variants
            RobinError::BuildError(reason) => {
                write!(f, "Build error: {}", reason)
            }
            RobinError::PlatformError(reason) => {
                write!(f, "Platform error: {}", reason)
            }
            RobinError::DeploymentError(reason) => {
                write!(f, "Deployment error: {}", reason)
            }
            RobinError::GenericError(reason) => {
                write!(f, "Generic error: {}", reason)
            }
            RobinError::PackagingError(reason) => {
                write!(f, "Packaging error: {}", reason)
            }
            RobinError::InvalidResource(reason) => {
                write!(f, "Invalid resource: {}", reason)
            }
            RobinError::BufferError(reason) => {
                write!(f, "Buffer error: {}", reason)
            }
            RobinError::InvalidInput(reason) => {
                write!(f, "Invalid input: {}", reason)
            }
            RobinError::Custom(reason) => {
                write!(f, "Custom error: {}", reason)
            }
            RobinError::OutOfMemory => {
                write!(f, "Out of memory")
            }
            RobinError::GPUMemoryError(reason) => {
                write!(f, "GPU memory error: {}", reason)
            }
            RobinError::GraphicsError(reason) => {
                write!(f, "Graphics error: {}", reason)
            }
            RobinError::ComputeError(reason) => {
                write!(f, "Compute error: {}", reason)
            }
            RobinError::TemplateError(reason) => {
                write!(f, "Template error: {}", reason)
            }
            RobinError::SystemError(reason) => {
                write!(f, "System error: {}", reason)
            }
            RobinError::StudyNotFound(reason) => {
                write!(f, "Study not found: {}", reason)
            }
            RobinError::ResourceLimitExceeded(reason) => {
                write!(f, "Resource limit exceeded: {}", reason)
            }
            RobinError::NoiseError(reason) => {
                write!(f, "Noise error: {}", reason)
            }
            RobinError::UIGenerationError(reason) => {
                write!(f, "UI generation error: {}", reason)
            }
            RobinError::ResourceLimit(reason) => {
                write!(f, "Resource limit: {}", reason)
            }
            RobinError::QuantumSimulatorNotFound(reason) => {
                write!(f, "Quantum simulator not found: {}", reason)
            }
            RobinError::QuantumAlgorithmNotFound(reason) => {
                write!(f, "Quantum algorithm not found: {}", reason)
            }
            RobinError::GPUError(reason) => {
                write!(f, "GPU error: {}", reason)
            }
            RobinError::ExportError(reason) => {
                write!(f, "Export error: {}", reason)
            }
            RobinError::VRSystem(reason) => {
                write!(f, "VR system error: {}", reason)
            }
            RobinError::Story(reason) => {
                write!(f, "Story system error: {}", reason)
            }
            RobinError::AssistanceError(reason) => {
                write!(f, "AI assistance error: {}", reason)
            }
            RobinError::Other(reason) => {
                write!(f, "Other error: {}", reason)
            }
            RobinError::BehaviorTreeError { tree_id, node_id, error } => {
                match node_id {
                    Some(node) => write!(f, "Behavior tree error in '{}' at node '{}': {}", tree_id, node, error),
                    None => write!(f, "Behavior tree error in '{}': {}", tree_id, error),
                }
            }
            RobinError::EventSystemError { event_type, error } => {
                match event_type {
                    Some(event) => write!(f, "Event system error with event '{}': {}", event, error),
                    None => write!(f, "Event system error: {}", error),
                }
            }
            RobinError::IoError(reason) => {
                write!(f, "I/O error: {}", reason)
            }
            RobinError::DestructionError(reason) => {
                write!(f, "Destruction error: {}", reason)
            }
            RobinError::GeneralError(reason) => {
                write!(f, "General error: {}", reason)
            }
        }
    }
}

impl StdError for RobinError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}

// Conversion implementations for common error types
impl From<std::io::Error> for RobinError {
    fn from(error: std::io::Error) -> Self {
        match error.kind() {
            std::io::ErrorKind::NotFound => {
                // Try to extract path from error if possible
                RobinError::InternalError(format!("I/O Error: {}", error))
            }
            std::io::ErrorKind::PermissionDenied => {
                RobinError::InternalError(format!("Permission denied: {}", error))
            }
            _ => RobinError::InternalError(format!("I/O Error: {}", error)),
        }
    }
}

impl From<serde_json::Error> for RobinError {
    fn from(error: serde_json::Error) -> Self {
        RobinError::AssetParseError {
            asset_id: "unknown".to_string(),
            format: "JSON".to_string(),
            line: Some(error.line()),
            column: Some(error.column()),
            message: error.to_string(),
        }
    }
}

impl From<toml::de::Error> for RobinError {
    fn from(error: toml::de::Error) -> Self {
        RobinError::AssetParseError {
            asset_id: "unknown".to_string(),
            format: "TOML".to_string(),
            line: None, // TOML error doesn't provide line info
            column: None, // TOML error doesn't provide column info
            message: error.to_string(),
        }
    }
}

impl From<serde_yaml::Error> for RobinError {
    fn from(error: serde_yaml::Error) -> Self {
        RobinError::AssetParseError {
            asset_id: "unknown".to_string(),
            format: "YAML".to_string(),
            line: error.location().map(|loc| loc.line()),
            column: error.location().map(|loc| loc.column()),
            message: error.to_string(),
        }
    }
}

impl From<&str> for RobinError {
    fn from(error: &str) -> Self {
        RobinError::GeneralError(error.to_string())
    }
}

impl From<String> for RobinError {
    fn from(error: String) -> Self {
        RobinError::GeneralError(error)
    }
}

impl From<Box<dyn std::error::Error>> for RobinError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        RobinError::GeneralError(error.to_string())
    }
}

impl From<crate::engine::assets::AssetError> for RobinError {
    fn from(error: crate::engine::assets::AssetError) -> Self {
        match error {
            crate::engine::assets::AssetError::FileNotFound(path) => {
                RobinError::FileNotFound(path)
            }
            crate::engine::assets::AssetError::LoadFailed(msg) => {
                RobinError::AssetLoadError {
                    asset_id: "unknown".to_string(),
                    path: PathBuf::from("unknown"),
                    reason: msg,
                }
            }
            crate::engine::assets::AssetError::UnsupportedFormat(format) => {
                RobinError::AssetParseError {
                    asset_id: "unknown".to_string(),
                    format,
                    line: None,
                    column: None,
                    message: "Unsupported format".to_string(),
                }
            }
            crate::engine::assets::AssetError::FileTooLarge(size) => {
                RobinError::AssetLoadError {
                    asset_id: "unknown".to_string(),
                    path: PathBuf::from("unknown"),
                    reason: format!("File too large: {} bytes", size),
                }
            }
            crate::engine::assets::AssetError::PermissionDenied(path) => {
                RobinError::FileAccessDenied(path)
            }
            crate::engine::assets::AssetError::WatcherError(msg) => {
                RobinError::HotReloadError(msg)
            }
            crate::engine::assets::AssetError::SerializationError(msg) => {
                RobinError::SerializationError {
                    object_type: "unknown".to_string(),
                    reason: msg,
                }
            }
            crate::engine::assets::AssetError::InvalidPath(msg) => {
                RobinError::InvalidInput(msg)
            }
        }
    }
}

impl From<std::num::ParseIntError> for RobinError {
    fn from(error: std::num::ParseIntError) -> Self {
        RobinError::ValidationError {
            field: "number_parsing".to_string(),
            value: "unknown".to_string(),
            constraint: format!("Failed to parse integer: {}", error),
        }
    }
}

impl From<std::num::ParseFloatError> for RobinError {
    fn from(error: std::num::ParseFloatError) -> Self {
        RobinError::ValidationError {
            field: "number_parsing".to_string(),
            value: "unknown".to_string(),
            constraint: format!("Failed to parse float: {}", error),
        }
    }
}

impl From<glob::PatternError> for RobinError {
    fn from(error: glob::PatternError) -> Self {
        RobinError::InvalidInput(format!("Invalid glob pattern: {}", error))
    }
}

impl RobinError {
    /// Create a new general error
    pub fn new(message: &str) -> Self {
        RobinError::GeneralError(message.to_string())
    }
}

/// Error context extension trait for better error reporting
pub trait ErrorContext<T> {
    /// Add context to an error
    fn with_context<F>(self, f: F) -> RobinResult<T>
    where
        F: FnOnce() -> String;
        
    /// Add asset context to an error
    fn with_asset_context(self, asset_id: &str, asset_type: &str) -> RobinResult<T>;
    
    /// Add scene context to an error
    fn with_scene_context(self, scene_name: &str) -> RobinResult<T>;
    
    /// Add component context to an error
    fn with_component_context(self, object_id: u32, component_type: &str) -> RobinResult<T>;
}

impl<T> ErrorContext<T> for RobinResult<T> {
    fn with_context<F>(self, f: F) -> RobinResult<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|err| {
            let context = f();
            RobinError::InternalError(format!("{}: {}", context, err))
        })
    }
    
    fn with_asset_context(self, asset_id: &str, asset_type: &str) -> RobinResult<T> {
        self.with_context(|| format!("Asset '{}' ({})", asset_id, asset_type))
    }
    
    fn with_scene_context(self, scene_name: &str) -> RobinResult<T> {
        self.with_context(|| format!("Scene '{}'", scene_name))
    }
    
    fn with_component_context(self, object_id: u32, component_type: &str) -> RobinResult<T> {
        self.with_context(|| format!("Component '{}' on object {}", component_type, object_id))
    }
}

/// Helper macros for common error patterns
#[macro_export]
macro_rules! robin_error {
    ($error_type:ident, $($field:ident: $value:expr),* $(,)?) => {
        $crate::engine::error::RobinError::$error_type {
            $($field: $value),*
        }
    };
}

#[macro_export]
macro_rules! bail_robin {
    ($error:expr) => {
        return Err($error)
    };
}

#[macro_export]
macro_rules! ensure_robin {
    ($cond:expr, $error:expr) => {
        if !$cond {
            return Err($error);
        }
    };
}

/// Error recovery strategies
pub enum RecoveryStrategy {
    Retry(usize),           // Retry N times
    Fallback(String),       // Use fallback value/resource
    Skip,                   // Skip the operation
    Abort,                  // Abort execution
    Ignore,                 // Ignore the error
}

pub trait Recoverable<T> {
    fn recover_with(self, strategy: RecoveryStrategy) -> RobinResult<Option<T>>;
}

impl<T> Recoverable<T> for RobinResult<T> {
    fn recover_with(self, strategy: RecoveryStrategy) -> RobinResult<Option<T>> {
        match self {
            Ok(value) => Ok(Some(value)),
            Err(error) => {
                match strategy {
                    RecoveryStrategy::Skip => {
                        log::warn!("Skipping operation due to error: {}", error);
                        Ok(None)
                    }
                    RecoveryStrategy::Ignore => {
                        log::debug!("Ignoring error: {}", error);
                        Ok(None)
                    }
                    RecoveryStrategy::Abort => Err(error),
                    RecoveryStrategy::Retry(_) | RecoveryStrategy::Fallback(_) => {
                        // These would need to be implemented with the specific operation
                        log::error!("Recovery strategy not implemented for this context: {}", error);
                        Err(error)
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = RobinError::AssetNotFound {
            asset_id: "player_sprite".to_string(),
            asset_type: "Texture".to_string(),
            searched_paths: vec![
                PathBuf::from("assets/player.png"),
                PathBuf::from("textures/player.png"),
            ],
        };
        
        let display = format!("{}", error);
        assert!(display.contains("player_sprite"));
        assert!(display.contains("Texture"));
        assert!(display.contains("assets/player.png"));
    }

    #[test]
    fn test_error_context() {
        let result: RobinResult<()> = Err(RobinError::InternalError("test error".to_string()));
        let with_context = result.with_asset_context("test_asset", "Texture");
        
        assert!(with_context.is_err());
        let error_msg = format!("{}", with_context.unwrap_err());
        assert!(error_msg.contains("test_asset"));
        assert!(error_msg.contains("Texture"));
    }

    #[test]
    fn test_recovery_strategy() {
        let error: RobinResult<String> = Err(RobinError::InternalError("test".to_string()));
        let recovered = error.recover_with(RecoveryStrategy::Skip);
        
        assert!(recovered.is_ok());
        assert!(recovered.unwrap().is_none());
    }

    #[test]
    fn test_macro_error_creation() {
        let error = robin_error!(AssetNotFound,
            asset_id: "test".to_string(),
            asset_type: "Texture".to_string(),
            searched_paths: vec![]
        );
        
        match error {
            RobinError::AssetNotFound { asset_id, .. } => {
                assert_eq!(asset_id, "test");
            }
            _ => panic!("Wrong error type"),
        }
    }
}