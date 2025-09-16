// Script Templates Library for Robin Engine
// Provides pre-built logic templates for common game mechanics

use crate::engine::error::RobinResult;
use std::collections::{HashMap, HashSet};
use std::time::Instant;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeValue {
    Bool(bool),
    Int(i32),
    Float(f32),
    String(String),
    Vector3([f32; 3]),
    Array(Vec<RuntimeValue>),
    Object(HashMap<String, RuntimeValue>),
    None,
}

impl RuntimeValue {
    pub fn as_bool(&self) -> bool {
        match self {
            RuntimeValue::Bool(b) => *b,
            RuntimeValue::Int(i) => *i != 0,
            RuntimeValue::Float(f) => *f != 0.0,
            RuntimeValue::String(s) => !s.is_empty(),
            RuntimeValue::Vector3(_) => true, // Non-zero vector is considered true
            RuntimeValue::Array(a) => !a.is_empty(),
            RuntimeValue::Object(o) => !o.is_empty(),
            RuntimeValue::None => false,
        }
    }

    pub fn as_int(&self) -> i32 {
        match self {
            RuntimeValue::Int(i) => *i,
            RuntimeValue::Float(f) => *f as i32,
            RuntimeValue::Bool(b) => if *b { 1 } else { 0 },
            _ => 0,
        }
    }

    pub fn as_float(&self) -> f32 {
        match self {
            RuntimeValue::Float(f) => *f,
            RuntimeValue::Int(i) => *i as f32,
            RuntimeValue::Bool(b) => if *b { 1.0 } else { 0.0 },
            _ => 0.0,
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            RuntimeValue::String(s) => s.clone(),
            RuntimeValue::Bool(b) => b.to_string(),
            RuntimeValue::Int(i) => i.to_string(),
            RuntimeValue::Float(f) => f.to_string(),
            RuntimeValue::Vector3(v) => format!("({}, {}, {})", v[0], v[1], v[2]),
            RuntimeValue::Array(_) => "[Array]".to_string(),
            RuntimeValue::Object(_) => "[Object]".to_string(),
            RuntimeValue::None => "None".to_string(),
        }
    }

    pub fn as_vector3(&self) -> [f32; 3] {
        match self {
            RuntimeValue::Vector3(v) => *v,
            _ => [0.0, 0.0, 0.0],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    Bool,
    Int,
    Float,
    String,
    Vector3,
    Array(Box<ParameterType>),
    Object(HashMap<String, ParameterType>),
    Choice(Vec<String>), // Dropdown/enum selection
    Range { min: f32, max: f32, step: f32 }, // Slider/numeric input with constraints
}

impl ParameterType {
    pub fn default_value(&self) -> RuntimeValue {
        match self {
            ParameterType::Bool => RuntimeValue::Bool(false),
            ParameterType::Int => RuntimeValue::Int(0),
            ParameterType::Float => RuntimeValue::Float(0.0),
            ParameterType::String => RuntimeValue::String("".to_string()),
            ParameterType::Vector3 => RuntimeValue::Vector3([0.0, 0.0, 0.0]),
            ParameterType::Array(_) => RuntimeValue::Array(Vec::new()),
            ParameterType::Object(_) => RuntimeValue::Object(HashMap::new()),
            ParameterType::Choice(choices) => {
                RuntimeValue::String(choices.first().unwrap_or(&"".to_string()).clone())
            }
            ParameterType::Range { min, .. } => RuntimeValue::Float(*min),
        }
    }

    pub fn validate(&self, value: &RuntimeValue) -> bool {
        match (self, value) {
            (ParameterType::Bool, RuntimeValue::Bool(_)) => true,
            (ParameterType::Int, RuntimeValue::Int(_)) => true,
            (ParameterType::Float, RuntimeValue::Float(_)) => true,
            (ParameterType::String, RuntimeValue::String(_)) => true,
            (ParameterType::Vector3, RuntimeValue::Vector3(_)) => true,
            (ParameterType::Array(inner_type), RuntimeValue::Array(values)) => {
                values.iter().all(|v| inner_type.validate(v))
            }
            (ParameterType::Object(expected_fields), RuntimeValue::Object(obj)) => {
                expected_fields.iter().all(|(key, expected_type)| {
                    obj.get(key).map_or(false, |value| expected_type.validate(value))
                })
            }
            (ParameterType::Choice(choices), RuntimeValue::String(value)) => {
                choices.contains(value)
            }
            (ParameterType::Range { min, max, .. }, RuntimeValue::Float(value)) => {
                *value >= *min && *value <= *max
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateParameter {
    pub name: String,
    pub parameter_type: ParameterType,
    pub description: String,
    pub required: bool,
    pub default_value: Option<RuntimeValue>,
    pub display_name: String,
    pub tooltip: Option<String>,
}

impl TemplateParameter {
    pub fn new(name: String, parameter_type: ParameterType, description: String) -> Self {
        let default_value = Some(parameter_type.default_value());
        Self {
            display_name: name.clone(),
            name,
            parameter_type,
            description,
            required: true,
            default_value,
            tooltip: None,
        }
    }

    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }

    pub fn with_default(mut self, default: RuntimeValue) -> Self {
        self.default_value = Some(default);
        self.required = false;
        self
    }

    pub fn with_display_name(mut self, display_name: String) -> Self {
        self.display_name = display_name;
        self
    }

    pub fn with_tooltip(mut self, tooltip: String) -> Self {
        self.tooltip = Some(tooltip);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum TemplateCategory {
    Building,
    Player,
    NPC,
    Environment,
    UI,
    Audio,
    Logic,
    Math,
    Utility,
    Game,
    Custom(String),
}

impl TemplateCategory {
    pub fn as_string(&self) -> String {
        match self {
            TemplateCategory::Building => "Building".to_string(),
            TemplateCategory::Player => "Player".to_string(),
            TemplateCategory::NPC => "NPC".to_string(),
            TemplateCategory::Environment => "Environment".to_string(),
            TemplateCategory::UI => "UI".to_string(),
            TemplateCategory::Audio => "Audio".to_string(),
            TemplateCategory::Logic => "Logic".to_string(),
            TemplateCategory::Math => "Math".to_string(),
            TemplateCategory::Utility => "Utility".to_string(),
            TemplateCategory::Game => "Game".to_string(),
            TemplateCategory::Custom(name) => name.clone(),
        }
    }

    pub fn get_all_categories() -> Vec<TemplateCategory> {
        vec![
            TemplateCategory::Building,
            TemplateCategory::Player,
            TemplateCategory::NPC,
            TemplateCategory::Environment,
            TemplateCategory::UI,
            TemplateCategory::Audio,
            TemplateCategory::Logic,
            TemplateCategory::Math,
            TemplateCategory::Utility,
            TemplateCategory::Game,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: TemplateCategory,
    pub parameters: Vec<TemplateParameter>,
    pub script_code: String,
    pub visual_nodes: Option<String>, // JSON representation of visual node graph
    pub tags: Vec<String>,
    pub author: String,
    pub version: String,
    pub created_at: u64,
    pub usage_count: u64,
    pub rating: f32,
    pub dependencies: Vec<String>, // Other template IDs this depends on
    pub examples: Vec<TemplateExample>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateExample {
    pub name: String,
    pub description: String,
    pub parameters: HashMap<String, RuntimeValue>,
    pub expected_output: Option<String>,
}

impl ScriptTemplate {
    pub fn new(name: String, description: String, category: TemplateCategory) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            category,
            parameters: Vec::new(),
            script_code: String::new(),
            visual_nodes: None,
            tags: Vec::new(),
            author: "System".to_string(),
            version: "1.0.0".to_string(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            usage_count: 0,
            rating: 0.0,
            dependencies: Vec::new(),
            examples: Vec::new(),
        }
    }

    pub fn with_parameter(mut self, parameter: TemplateParameter) -> Self {
        self.parameters.push(parameter);
        self
    }

    pub fn with_script_code(mut self, code: String) -> Self {
        self.script_code = code;
        self
    }

    pub fn with_visual_nodes(mut self, nodes_json: String) -> Self {
        self.visual_nodes = Some(nodes_json);
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_author(mut self, author: String) -> Self {
        self.author = author;
        self
    }

    pub fn with_version(mut self, version: String) -> Self {
        self.version = version;
        self
    }

    pub fn with_dependency(mut self, template_id: String) -> Self {
        self.dependencies.push(template_id);
        self
    }

    pub fn with_example(mut self, example: TemplateExample) -> Self {
        self.examples.push(example);
        self
    }

    pub fn validate_parameters(&self, provided: &HashMap<String, RuntimeValue>) -> RobinResult<()> {
        for param in &self.parameters {
            if param.required && !provided.contains_key(&param.name) {
                return Err(crate::engine::error::RobinError::InvalidInput(
                    format!("Required parameter '{}' is missing", param.name)
                ));
            }

            if let Some(value) = provided.get(&param.name) {
                if !param.parameter_type.validate(value) {
                    return Err(crate::engine::error::RobinError::InvalidInput(
                        format!("Parameter '{}' has invalid type", param.name)
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn get_effective_parameters(&self, provided: &HashMap<String, RuntimeValue>) -> HashMap<String, RuntimeValue> {
        let mut effective = HashMap::new();
        
        // Add provided parameters
        for (key, value) in provided {
            effective.insert(key.clone(), value.clone());
        }
        
        // Add default values for missing optional parameters
        for param in &self.parameters {
            if !effective.contains_key(&param.name) {
                if let Some(default) = &param.default_value {
                    effective.insert(param.name.clone(), default.clone());
                }
            }
        }
        
        effective
    }

    pub fn instantiate(&self, parameters: HashMap<String, RuntimeValue>) -> RobinResult<String> {
        self.validate_parameters(&parameters)?;
        let effective_params = self.get_effective_parameters(&parameters);
        
        // Simple template substitution - in a real implementation, this would be more sophisticated
        let mut instantiated_code = self.script_code.clone();
        
        for (key, value) in &effective_params {
            let placeholder = format!("{{{}}}", key);
            let replacement = match value {
                RuntimeValue::String(s) => format!("\"{}\"", s),
                RuntimeValue::Int(i) => i.to_string(),
                RuntimeValue::Float(f) => f.to_string(),
                RuntimeValue::Bool(b) => b.to_string(),
                RuntimeValue::Vector3(v) => format!("vec3({}, {}, {})", v[0], v[1], v[2]),
                _ => value.as_string(),
            };
            instantiated_code = instantiated_code.replace(&placeholder, &replacement);
        }
        
        Ok(instantiated_code)
    }
}

#[derive(Debug, Clone)]
pub struct CustomTemplate {
    pub template: ScriptTemplate,
    pub source_file: Option<String>,
    pub is_user_created: bool,
    pub modification_time: Option<Instant>,
}

impl CustomTemplate {
    pub fn new(template: ScriptTemplate) -> Self {
        Self {
            template,
            source_file: None,
            is_user_created: true,
            modification_time: Some(Instant::now()),
        }
    }

    pub fn from_file(template: ScriptTemplate, file_path: String) -> Self {
        Self {
            template,
            source_file: Some(file_path),
            is_user_created: false,
            modification_time: Some(Instant::now()),
        }
    }
}

#[derive(Debug)]
pub struct TemplateLibrary {
    templates: HashMap<String, ScriptTemplate>,
    categories: HashMap<TemplateCategory, Vec<String>>,
    tags: HashMap<String, HashSet<String>>, // tag -> template IDs
    usage_stats: HashMap<String, TemplateUsageStats>,
}

#[derive(Debug, Clone)]
pub struct TemplateUsageStats {
    pub template_id: String,
    pub usage_count: u64,
    pub last_used: Option<Instant>,
    pub average_rating: f32,
    pub total_ratings: u32,
    pub instantiation_time_ms: f32,
}

impl TemplateLibrary {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
            categories: HashMap::new(),
            tags: HashMap::new(),
            usage_stats: HashMap::new(),
        }
    }

    pub fn add_template(&mut self, template: ScriptTemplate) {
        let template_id = template.id.clone();
        let category = template.category.clone();
        
        // Add to categories index
        self.categories.entry(category)
            .or_insert_with(Vec::new)
            .push(template_id.clone());
        
        // Add to tags index
        for tag in &template.tags {
            self.tags.entry(tag.clone())
                .or_insert_with(HashSet::new)
                .insert(template_id.clone());
        }
        
        // Initialize usage stats
        self.usage_stats.insert(template_id.clone(), TemplateUsageStats {
            template_id: template_id.clone(),
            usage_count: 0,
            last_used: None,
            average_rating: 0.0,
            total_ratings: 0,
            instantiation_time_ms: 0.0,
        });
        
        self.templates.insert(template_id, template);
    }

    pub fn get_template(&self, template_id: &str) -> Option<&ScriptTemplate> {
        self.templates.get(template_id)
    }

    pub fn get_templates_by_category(&self, category: &TemplateCategory) -> Vec<&ScriptTemplate> {
        self.categories.get(category)
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|id| self.templates.get(id))
            .collect()
    }

    pub fn get_templates_by_tag(&self, tag: &str) -> Vec<&ScriptTemplate> {
        self.tags.get(tag)
            .unwrap_or(&HashSet::new())
            .iter()
            .filter_map(|id| self.templates.get(id))
            .collect()
    }

    pub fn search_templates(&self, query: &str) -> Vec<&ScriptTemplate> {
        let query_lower = query.to_lowercase();
        self.templates.values()
            .filter(|template| {
                template.name.to_lowercase().contains(&query_lower) ||
                template.description.to_lowercase().contains(&query_lower) ||
                template.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    pub fn get_popular_templates(&self, limit: usize) -> Vec<&ScriptTemplate> {
        let mut templates_with_usage: Vec<_> = self.templates.values()
            .map(|template| {
                let usage = self.usage_stats.get(&template.id)
                    .map(|stats| stats.usage_count)
                    .unwrap_or(0);
                (template, usage)
            })
            .collect();
        
        templates_with_usage.sort_by(|a, b| b.1.cmp(&a.1));
        templates_with_usage.into_iter()
            .take(limit)
            .map(|(template, _)| template)
            .collect()
    }

    pub fn get_recent_templates(&self, limit: usize) -> Vec<&ScriptTemplate> {
        let mut templates: Vec<_> = self.templates.values().collect();
        templates.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        templates.into_iter().take(limit).collect()
    }

    pub fn record_usage(&mut self, template_id: &str) {
        if let Some(stats) = self.usage_stats.get_mut(template_id) {
            stats.usage_count += 1;
            stats.last_used = Some(Instant::now());
        }
    }

    pub fn rate_template(&mut self, template_id: &str, rating: f32) -> RobinResult<()> {
        if !(1.0..=5.0).contains(&rating) {
            return Err(crate::engine::error::RobinError::InvalidInput(
                "Rating must be between 1.0 and 5.0".to_string()
            ));
        }

        if let Some(stats) = self.usage_stats.get_mut(template_id) {
            let total_score = stats.average_rating * stats.total_ratings as f32;
            stats.total_ratings += 1;
            stats.average_rating = (total_score + rating) / stats.total_ratings as f32;
        }
        Ok(())
    }

    pub fn remove_template(&mut self, template_id: &str) -> Option<ScriptTemplate> {
        if let Some(template) = self.templates.remove(template_id) {
            // Remove from categories
            if let Some(category_templates) = self.categories.get_mut(&template.category) {
                category_templates.retain(|id| id != template_id);
            }
            
            // Remove from tags
            for tag in &template.tags {
                if let Some(tag_templates) = self.tags.get_mut(tag) {
                    tag_templates.remove(template_id);
                }
            }
            
            // Remove usage stats
            self.usage_stats.remove(template_id);
            
            Some(template)
        } else {
            None
        }
    }

    pub fn get_template_count(&self) -> usize {
        self.templates.len()
    }

    pub fn get_categories(&self) -> Vec<TemplateCategory> {
        self.categories.keys().cloned().collect()
    }

    pub fn get_all_tags(&self) -> Vec<String> {
        self.tags.keys().cloned().collect()
    }

    pub fn get_usage_stats(&self, template_id: &str) -> Option<&TemplateUsageStats> {
        self.usage_stats.get(template_id)
    }
}

#[derive(Debug)]
pub struct BuiltinTemplates;

impl BuiltinTemplates {
    pub fn create_all() -> Vec<ScriptTemplate> {
        let mut templates = Vec::new();
        
        // Building Templates
        templates.extend(Self::create_building_templates());
        
        // Player Templates  
        templates.extend(Self::create_player_templates());
        
        // NPC Templates
        templates.extend(Self::create_npc_templates());
        
        // Environment Templates
        templates.extend(Self::create_environment_templates());
        
        // Logic Templates
        templates.extend(Self::create_logic_templates());
        
        // Math Templates
        templates.extend(Self::create_math_templates());
        
        // Utility Templates
        templates.extend(Self::create_utility_templates());
        
        templates
    }

    fn create_building_templates() -> Vec<ScriptTemplate> {
        vec![
            ScriptTemplate::new(
                "Auto Builder".to_string(),
                "Automatically builds structures based on blueprints".to_string(),
                TemplateCategory::Building
            )
            .with_parameter(TemplateParameter::new(
                "blueprint_name".to_string(),
                ParameterType::String,
                "Name of the blueprint to build".to_string()
            ))
            .with_parameter(TemplateParameter::new(
                "start_position".to_string(),
                ParameterType::Vector3,
                "Starting position for the build".to_string()
            ))
            .with_parameter(TemplateParameter::new(
                "build_speed".to_string(),
                ParameterType::Range { min: 0.1, max: 10.0, step: 0.1 },
                "Speed of construction in blocks per second".to_string()
            ).with_default(RuntimeValue::Float(1.0)).optional())
            .with_script_code(r#"
function auto_build(blueprint_name, start_position, build_speed) {
    let blueprint = load_blueprint({blueprint_name});
    let pos = {start_position};
    let speed = {build_speed};
    
    for (let block of blueprint.blocks) {
        let world_pos = add_vectors(pos, block.relative_position);
        place_block(world_pos, block.type, block.properties);
        wait(1.0 / speed);
    }
    
    log("Auto build completed: " + blueprint_name);
    return true;
}
"#.to_string())
            .with_tags(vec!["building".to_string(), "automation".to_string(), "construction".to_string()])
            .with_example(TemplateExample {
                name: "Build House".to_string(),
                description: "Build a simple house at origin".to_string(),
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("blueprint_name".to_string(), RuntimeValue::String("simple_house".to_string()));
                    params.insert("start_position".to_string(), RuntimeValue::Vector3([0.0, 0.0, 0.0]));
                    params.insert("build_speed".to_string(), RuntimeValue::Float(2.0));
                    params
                },
                expected_output: Some("House built successfully".to_string()),
            }),

            ScriptTemplate::new(
                "Block Replacer".to_string(),
                "Replace all blocks of one type with another in a specified area".to_string(),
                TemplateCategory::Building
            )
            .with_parameter(TemplateParameter::new(
                "area_min".to_string(),
                ParameterType::Vector3,
                "Minimum corner of the area".to_string()
            ))
            .with_parameter(TemplateParameter::new(
                "area_max".to_string(),
                ParameterType::Vector3,
                "Maximum corner of the area".to_string()
            ))
            .with_parameter(TemplateParameter::new(
                "from_block".to_string(),
                ParameterType::Choice(vec!["stone".to_string(), "wood".to_string(), "grass".to_string(), "dirt".to_string()]),
                "Block type to replace".to_string()
            ))
            .with_parameter(TemplateParameter::new(
                "to_block".to_string(),
                ParameterType::Choice(vec!["stone".to_string(), "wood".to_string(), "grass".to_string(), "dirt".to_string()]),
                "Block type to replace with".to_string()
            ))
            .with_script_code(r#"
function replace_blocks(area_min, area_max, from_block, to_block) {
    let min = {area_min};
    let max = {area_max};
    let from = "{from_block}";
    let to = "{to_block}";
    
    let replaced_count = 0;
    
    for (let x = min.x; x <= max.x; x++) {
        for (let y = min.y; y <= max.y; y++) {
            for (let z = min.z; z <= max.z; z++) {
                let pos = vec3(x, y, z);
                let current_block = get_block_type(pos);
                
                if (current_block === from) {
                    set_block_type(pos, to);
                    replaced_count++;
                }
            }
        }
    }
    
    log("Replaced " + replaced_count + " blocks from " + from + " to " + to);
    return replaced_count;
}
"#.to_string())
            .with_tags(vec!["building".to_string(), "replacement".to_string(), "area".to_string()]),

            ScriptTemplate::new(
                "Structure Copier".to_string(),
                "Copy a structure from one location and paste it at another".to_string(),
                TemplateCategory::Building
            )
            .with_parameter(TemplateParameter::new(
                "source_min".to_string(),
                ParameterType::Vector3,
                "Minimum corner of source area".to_string()
            ))
            .with_parameter(TemplateParameter::new(
                "source_max".to_string(),
                ParameterType::Vector3,
                "Maximum corner of source area".to_string()
            ))
            .with_parameter(TemplateParameter::new(
                "destination".to_string(),
                ParameterType::Vector3,
                "Destination position (corresponds to source_min)".to_string()
            ))
            .with_parameter(TemplateParameter::new(
                "include_air".to_string(),
                ParameterType::Bool,
                "Whether to copy air blocks".to_string()
            ).with_default(RuntimeValue::Bool(false)).optional())
            .with_script_code(r#"
function copy_structure(source_min, source_max, destination, include_air) {
    let s_min = {source_min};
    let s_max = {source_max};
    let dest = {destination};
    let copy_air = {include_air};
    
    let copied_blocks = 0;
    let offset = subtract_vectors(dest, s_min);
    
    for (let x = s_min.x; x <= s_max.x; x++) {
        for (let y = s_min.y; y <= s_max.y; y++) {
            for (let z = s_min.z; z <= s_max.z; z++) {
                let source_pos = vec3(x, y, z);
                let dest_pos = add_vectors(source_pos, offset);
                
                let block_type = get_block_type(source_pos);
                let block_props = get_block_properties(source_pos);
                
                if (block_type !== "air" || copy_air) {
                    set_block_type(dest_pos, block_type);
                    set_block_properties(dest_pos, block_props);
                    copied_blocks++;
                }
            }
        }
    }
    
    log("Copied " + copied_blocks + " blocks to destination");
    return copied_blocks;
}
"#.to_string())
            .with_tags(vec!["building".to_string(), "copy".to_string(), "paste".to_string(), "structure".to_string()]),
        ]
    }

    fn create_player_templates() -> Vec<ScriptTemplate> {
        vec![
            ScriptTemplate::new(
                "Health Monitor".to_string(),
                "Monitor player health and trigger events when health is low".to_string(),
                TemplateCategory::Player
            )
            .with_parameter(TemplateParameter::new(
                "player_id".to_string(),
                ParameterType::String,
                "ID of the player to monitor".to_string()
            ))
            .with_parameter(TemplateParameter::new(
                "low_health_threshold".to_string(),
                ParameterType::Range { min: 1.0, max: 100.0, step: 1.0 },
                "Health percentage that triggers low health warning".to_string()
            ).with_default(RuntimeValue::Float(25.0)).optional())
            .with_parameter(TemplateParameter::new(
                "critical_health_threshold".to_string(),
                ParameterType::Range { min: 1.0, max: 50.0, step: 1.0 },
                "Health percentage that triggers critical health warning".to_string()
            ).with_default(RuntimeValue::Float(10.0)).optional())
            .with_script_code(r#"
function monitor_health(player_id, low_threshold, critical_threshold) {
    let id = "{player_id}";
    let low = {low_health_threshold};
    let critical = {critical_health_threshold};
    
    let previous_health = get_player_health(id);
    
    while (true) {
        let current_health = get_player_health(id);
        
        if (current_health <= critical && previous_health > critical) {
            trigger_event("player.health.critical", {
                player_id: id,
                health: current_health
            });
            show_notification(id, "CRITICAL HEALTH!", "red");
        } else if (current_health <= low && previous_health > low) {
            trigger_event("player.health.low", {
                player_id: id,
                health: current_health
            });
            show_notification(id, "Low Health Warning", "orange");
        }
        
        previous_health = current_health;
        wait(1.0);
    }
}
"#.to_string())
            .with_tags(vec!["player".to_string(), "health".to_string(), "monitoring".to_string(), "alerts".to_string()]),

            ScriptTemplate::new(
                "Auto Respawn".to_string(),
                "Automatically respawn player at a safe location when they die".to_string(),
                TemplateCategory::Player
            )
            .with_parameter(TemplateParameter::new(
                "player_id".to_string(),
                ParameterType::String,
                "ID of the player".to_string()
            ))
            .with_parameter(TemplateParameter::new(
                "respawn_position".to_string(),
                ParameterType::Vector3,
                "Position where player should respawn".to_string()
            ))
            .with_parameter(TemplateParameter::new(
                "respawn_delay".to_string(),
                ParameterType::Range { min: 0.0, max: 60.0, step: 0.5 },
                "Delay in seconds before respawning".to_string()
            ).with_default(RuntimeValue::Float(3.0)).optional())
            .with_parameter(TemplateParameter::new(
                "restore_inventory".to_string(),
                ParameterType::Bool,
                "Whether to restore player's inventory on respawn".to_string()
            ).with_default(RuntimeValue::Bool(false)).optional())
            .with_script_code(r#"
function auto_respawn(player_id, respawn_position, respawn_delay, restore_inventory) {
    let id = "{player_id}";
    let pos = {respawn_position};
    let delay = {respawn_delay};
    let restore_inv = {restore_inventory};
    
    // Listen for player death event
    on_event("player.death", function(event) {
        if (event.player_id === id) {
            let saved_inventory = null;
            if (restore_inv) {
                saved_inventory = get_player_inventory(id);
            }
            
            show_notification(id, "You died! Respawning in " + delay + " seconds...", "red");
            
            wait(delay);
            
            set_player_position(id, pos);
            set_player_health(id, 100);
            
            if (restore_inv && saved_inventory) {
                set_player_inventory(id, saved_inventory);
            }
            
            show_notification(id, "You have been respawned!", "green");
            
            trigger_event("player.respawned", {
                player_id: id,
                position: pos
            });
        }
    });
}
"#.to_string())
            .with_tags(vec!["player".to_string(), "respawn".to_string(), "death".to_string(), "automation".to_string()]),
        ]
    }

    fn create_npc_templates() -> Vec<ScriptTemplate> {
        vec![
            ScriptTemplate::new(
                "Patrol Behavior".to_string(),
                "Make an NPC patrol between a series of waypoints".to_string(),
                TemplateCategory::NPC
            )
            .with_parameter(TemplateParameter::new(
                "npc_id".to_string(),
                ParameterType::String,
                "ID of the NPC".to_string()
            ))
            .with_parameter(TemplateParameter::new(
                "waypoints".to_string(),
                ParameterType::Array(Box::new(ParameterType::Vector3)),
                "List of waypoint positions".to_string()
            ))
            .with_parameter(TemplateParameter::new(
                "patrol_speed".to_string(),
                ParameterType::Range { min: 0.1, max: 10.0, step: 0.1 },
                "Movement speed of the NPC".to_string()
            ).with_default(RuntimeValue::Float(2.0)).optional())
            .with_parameter(TemplateParameter::new(
                "wait_time".to_string(),
                ParameterType::Range { min: 0.0, max: 30.0, step: 0.5 },
                "Time to wait at each waypoint".to_string()
            ).with_default(RuntimeValue::Float(2.0)).optional())
            .with_script_code(r#"
function npc_patrol(npc_id, waypoints, patrol_speed, wait_time) {
    let id = "{npc_id}";
    let points = {waypoints};
    let speed = {patrol_speed};
    let wait = {wait_time};
    
    let current_waypoint = 0;
    
    while (true) {
        let target = points[current_waypoint];
        
        // Move to waypoint
        move_npc_to(id, target, speed);
        
        // Wait until NPC reaches waypoint
        while (distance(get_npc_position(id), target) > 1.0) {
            wait(0.1);
        }
        
        // Wait at waypoint
        wait(wait);
        
        // Move to next waypoint
        current_waypoint = (current_waypoint + 1) % points.length;
        
        trigger_event("npc.waypoint_reached", {
            npc_id: id,
            waypoint: current_waypoint,
            position: target
        });
    }
}
"#.to_string())
            .with_tags(vec!["npc".to_string(), "patrol".to_string(), "movement".to_string(), "ai".to_string()]),

            ScriptTemplate::new(
                "Guard Behavior".to_string(),
                "Make an NPC guard an area and react to intruders".to_string(),
                TemplateCategory::NPC
            )
            .with_parameter(TemplateParameter::new(
                "npc_id".to_string(),
                ParameterType::String,
                "ID of the guard NPC".to_string()
            ))
            .with_parameter(TemplateParameter::new(
                "guard_position".to_string(),
                ParameterType::Vector3,
                "Position the NPC should guard".to_string()
            ))
            .with_parameter(TemplateParameter::new(
                "detection_radius".to_string(),
                ParameterType::Range { min: 1.0, max: 50.0, step: 1.0 },
                "Radius in which to detect intruders".to_string()
            ).with_default(RuntimeValue::Float(10.0)).optional())
            .with_parameter(TemplateParameter::new(
                "alert_message".to_string(),
                ParameterType::String,
                "Message to display when intruder detected".to_string()
            ).with_default(RuntimeValue::String("Intruder detected!".to_string())).optional())
            .with_script_code(r#"
function guard_area(npc_id, guard_position, detection_radius, alert_message) {
    let id = "{npc_id}";
    let pos = {guard_position};
    let radius = {detection_radius};
    let message = "{alert_message}";
    
    // Move to guard position
    move_npc_to(id, pos, 2.0);
    
    while (true) {
        let nearby_players = get_players_in_radius(pos, radius);
        
        for (let player of nearby_players) {
            if (get_player_id() !== player.id) { // Don't alert for the owner
                // Face the intruder
                face_npc_towards(id, player.position);
                
                // Show alert message
                npc_speak(id, message);
                
                // Trigger alert event
                trigger_event("npc.intruder_detected", {
                    guard_id: id,
                    intruder_id: player.id,
                    position: player.position
                });
                
                // Follow intruder for a bit
                let chase_time = 0;
                while (chase_time < 5.0 && distance(get_npc_position(id), player.position) < radius * 2) {
                    move_npc_to(id, player.position, 3.0);
                    wait(0.5);
                    chase_time += 0.5;
                }
                
                // Return to guard position
                move_npc_to(id, pos, 2.0);
            }
        }
        
        wait(1.0);
    }
}
"#.to_string())
            .with_tags(vec!["npc".to_string(), "guard".to_string(), "security".to_string(), "detection".to_string()]),
        ]
    }

    fn create_environment_templates() -> Vec<ScriptTemplate> {
        vec![
            ScriptTemplate::new(
                "Day Night Cycle".to_string(),
                "Create a realistic day/night cycle with gradual lighting changes".to_string(),
                TemplateCategory::Environment
            )
            .with_parameter(TemplateParameter::new(
                "day_duration".to_string(),
                ParameterType::Range { min: 60.0, max: 3600.0, step: 60.0 },
                "Duration of a full day cycle in seconds".to_string()
            ).with_default(RuntimeValue::Float(1200.0)).optional())
            .with_parameter(TemplateParameter::new(
                "start_time".to_string(),
                ParameterType::Range { min: 0.0, max: 24.0, step: 0.5 },
                "Starting time of day (0-24 hours)".to_string()
            ).with_default(RuntimeValue::Float(6.0)).optional())
            .with_script_code(r#"
function day_night_cycle(day_duration, start_time) {
    let duration = {day_duration};
    let current_time = {start_time};
    
    while (true) {
        // Calculate lighting based on time
        let light_intensity = calculate_sun_intensity(current_time);
        let sky_color = calculate_sky_color(current_time);
        
        // Update world lighting
        set_ambient_light(light_intensity);
        set_sky_color(sky_color);
        
        // Trigger time-based events
        let hour = Math.floor(current_time);
        if (hour === 6 && Math.floor(current_time - 0.1) !== 6) {
            trigger_event("world.sunrise", { time: current_time });
        } else if (hour === 18 && Math.floor(current_time - 0.1) !== 18) {
            trigger_event("world.sunset", { time: current_time });
        }
        
        // Advance time
        current_time += (24.0 / duration);
        if (current_time >= 24.0) {
            current_time -= 24.0;
            trigger_event("world.new_day", { day: Math.floor(get_world_time() / duration) });
        }
        
        wait(1.0);
    }
}

function calculate_sun_intensity(time) {
    if (time >= 6.0 && time <= 18.0) {
        // Day time - use sine curve for realistic lighting
        let day_progress = (time - 6.0) / 12.0;
        return Math.sin(day_progress * Math.PI) * 0.8 + 0.2;
    } else {
        // Night time - low ambient light
        return 0.1;
    }
}

function calculate_sky_color(time) {
    if (time >= 5.0 && time <= 7.0) {
        // Sunrise colors
        return interpolate_color([1.0, 0.4, 0.2], [0.5, 0.8, 1.0], (time - 5.0) / 2.0);
    } else if (time >= 7.0 && time <= 17.0) {
        // Day colors
        return [0.5, 0.8, 1.0];
    } else if (time >= 17.0 && time <= 19.0) {
        // Sunset colors
        return interpolate_color([0.5, 0.8, 1.0], [1.0, 0.4, 0.2], (time - 17.0) / 2.0);
    } else {
        // Night colors
        return [0.1, 0.1, 0.2];
    }
}
"#.to_string())
            .with_tags(vec!["environment".to_string(), "lighting".to_string(), "time".to_string(), "atmosphere".to_string()]),

            ScriptTemplate::new(
                "Weather System".to_string(),
                "Dynamic weather system with rain, snow, and storms".to_string(),
                TemplateCategory::Environment
            )
            .with_parameter(TemplateParameter::new(
                "weather_change_interval".to_string(),
                ParameterType::Range { min: 60.0, max: 1800.0, step: 30.0 },
                "Time between weather changes in seconds".to_string()
            ).with_default(RuntimeValue::Float(300.0)).optional())
            .with_parameter(TemplateParameter::new(
                "enable_storms".to_string(),
                ParameterType::Bool,
                "Whether to include storm weather".to_string()
            ).with_default(RuntimeValue::Bool(true)).optional())
            .with_script_code(r#"
function weather_system(change_interval, enable_storms) {
    let interval = {weather_change_interval};
    let storms_enabled = {enable_storms};
    
    let weather_types = ["clear", "cloudy", "rain", "snow"];
    if (storms_enabled) {
        weather_types.push("storm");
    }
    
    let current_weather = "clear";
    
    while (true) {
        wait(interval);
        
        // Choose new weather
        let new_weather = weather_types[Math.floor(Math.random() * weather_types.length)];
        
        // Don't repeat the same weather
        if (new_weather === current_weather) {
            continue;
        }
        
        // Transition to new weather
        transition_weather(current_weather, new_weather);
        current_weather = new_weather;
        
        trigger_event("world.weather_change", {
            from: current_weather,
            to: new_weather,
            timestamp: get_world_time()
        });
        
        log("Weather changed to: " + new_weather);
    }
}

function transition_weather(from, to) {
    // Gradual transition over 30 seconds
    for (let i = 0; i <= 30; i++) {
        let progress = i / 30.0;
        apply_weather_effects(from, to, progress);
        wait(1.0);
    }
}

function apply_weather_effects(from, to, progress) {
    switch (to) {
        case "rain":
            set_particle_effect("rain", lerp(0, 100, progress));
            set_ambient_sound("rain", lerp(0, 0.7, progress));
            break;
        case "snow":
            set_particle_effect("snow", lerp(0, 80, progress));
            set_ambient_sound("wind", lerp(0, 0.5, progress));
            break;
        case "storm":
            set_particle_effect("rain", lerp(0, 150, progress));
            set_ambient_sound("thunder", lerp(0, 0.8, progress));
            if (progress > 0.5 && Math.random() < 0.1) {
                trigger_lightning_flash();
            }
            break;
        case "clear":
            set_particle_effect("none", 0);
            set_ambient_sound("none", 0);
            break;
    }
}
"#.to_string())
            .with_tags(vec!["environment".to_string(), "weather".to_string(), "particles".to_string(), "atmosphere".to_string()]),
        ]
    }

    fn create_logic_templates() -> Vec<ScriptTemplate> {
        vec![
            ScriptTemplate::new(
                "State Machine".to_string(),
                "Generic finite state machine for complex logic".to_string(),
                TemplateCategory::Logic
            )
            .with_parameter(TemplateParameter::new(
                "initial_state".to_string(),
                ParameterType::String,
                "Name of the initial state".to_string()
            ))
            .with_parameter(TemplateParameter::new(
                "states".to_string(),
                ParameterType::Array(Box::new(ParameterType::String)),
                "List of all possible states".to_string()
            ))
            .with_script_code(r#"
function state_machine(initial_state, states) {
    let current_state = "{initial_state}";
    let valid_states = {states};
    let state_data = {};
    
    let machine = {
        get_current_state: function() {
            return current_state;
        },
        
        transition_to: function(new_state) {
            if (!valid_states.includes(new_state)) {
                log("Error: Invalid state transition to " + new_state);
                return false;
            }
            
            let old_state = current_state;
            current_state = new_state;
            
            trigger_event("state_machine.transition", {
                from: old_state,
                to: new_state,
                timestamp: get_time()
            });
            
            log("State transition: " + old_state + " -> " + new_state);
            return true;
        },
        
        set_state_data: function(key, value) {
            state_data[key] = value;
        },
        
        get_state_data: function(key) {
            return state_data[key];
        },
        
        is_in_state: function(state) {
            return current_state === state;
        }
    };
    
    return machine;
}
"#.to_string())
            .with_tags(vec!["logic".to_string(), "state".to_string(), "machine".to_string(), "control".to_string()]),

            ScriptTemplate::new(
                "Timer System".to_string(),
                "Flexible timer system with multiple concurrent timers".to_string(),
                TemplateCategory::Logic
            )
            .with_parameter(TemplateParameter::new(
                "timer_name".to_string(),
                ParameterType::String,
                "Unique name for this timer".to_string()
            ))
            .with_parameter(TemplateParameter::new(
                "duration".to_string(),
                ParameterType::Range { min: 0.1, max: 3600.0, step: 0.1 },
                "Timer duration in seconds".to_string()
            ))
            .with_parameter(TemplateParameter::new(
                "repeat".to_string(),
                ParameterType::Bool,
                "Whether the timer should repeat automatically".to_string()
            ).with_default(RuntimeValue::Bool(false)).optional())
            .with_script_code(r#"
let global_timers = {};

function create_timer(timer_name, duration, repeat) {
    let name = "{timer_name}";
    let time = {duration};
    let repeating = {repeat};
    
    global_timers[name] = {
        duration: time,
        remaining: time,
        repeating: repeating,
        active: false,
        paused: false
    };
    
    return {
        start: function() {
            global_timers[name].active = true;
            global_timers[name].paused = false;
            global_timers[name].remaining = global_timers[name].duration;
            
            trigger_event("timer.started", { name: name });
        },
        
        pause: function() {
            global_timers[name].paused = true;
            trigger_event("timer.paused", { name: name });
        },
        
        resume: function() {
            global_timers[name].paused = false;
            trigger_event("timer.resumed", { name: name });
        },
        
        stop: function() {
            global_timers[name].active = false;
            global_timers[name].remaining = global_timers[name].duration;
            trigger_event("timer.stopped", { name: name });
        },
        
        get_remaining: function() {
            return global_timers[name].remaining;
        },
        
        get_progress: function() {
            return 1.0 - (global_timers[name].remaining / global_timers[name].duration);
        }
    };
}

// Update function that should be called every frame
function update_timers(delta_time) {
    for (let name in global_timers) {
        let timer = global_timers[name];
        
        if (timer.active && !timer.paused) {
            timer.remaining -= delta_time;
            
            if (timer.remaining <= 0) {
                trigger_event("timer.finished", { 
                    name: name,
                    duration: timer.duration
                });
                
                if (timer.repeating) {
                    timer.remaining = timer.duration;
                } else {
                    timer.active = false;
                }
            }
        }
    }
}
"#.to_string())
            .with_tags(vec!["logic".to_string(), "timer".to_string(), "scheduling".to_string(), "utility".to_string()]),
        ]
    }

    fn create_math_templates() -> Vec<ScriptTemplate> {
        vec![
            ScriptTemplate::new(
                "Interpolation Functions".to_string(),
                "Collection of interpolation and easing functions".to_string(),
                TemplateCategory::Math
            )
            .with_parameter(TemplateParameter::new(
                "start_value".to_string(),
                ParameterType::Float,
                "Starting value for interpolation".to_string()
            ))
            .with_parameter(TemplateParameter::new(
                "end_value".to_string(),
                ParameterType::Float,
                "Ending value for interpolation".to_string()
            ))
            .with_parameter(TemplateParameter::new(
                "easing_type".to_string(),
                ParameterType::Choice(vec![
                    "linear".to_string(),
                    "ease_in".to_string(),
                    "ease_out".to_string(),
                    "ease_in_out".to_string(),
                    "bounce".to_string()
                ]),
                "Type of easing to apply".to_string()
            ).with_default(RuntimeValue::String("linear".to_string())).optional())
            .with_script_code(r#"
function interpolate(start_value, end_value, progress, easing_type) {
    let start = {start_value};
    let end = {end_value};
    let t = Math.max(0, Math.min(1, progress));
    let easing = "{easing_type}";
    
    // Apply easing function
    let eased_t = apply_easing(t, easing);
    
    return start + (end - start) * eased_t;
}

function apply_easing(t, type) {
    switch (type) {
        case "linear":
            return t;
        case "ease_in":
            return t * t;
        case "ease_out":
            return 1 - (1 - t) * (1 - t);
        case "ease_in_out":
            return t < 0.5 ? 2 * t * t : 1 - Math.pow(-2 * t + 2, 2) / 2;
        case "bounce":
            if (t < 1 / 2.75) {
                return 7.5625 * t * t;
            } else if (t < 2 / 2.75) {
                return 7.5625 * (t -= 1.5 / 2.75) * t + 0.75;
            } else if (t < 2.5 / 2.75) {
                return 7.5625 * (t -= 2.25 / 2.75) * t + 0.9375;
            } else {
                return 7.5625 * (t -= 2.625 / 2.75) * t + 0.984375;
            }
        default:
            return t;
    }
}

// Interpolate between two vectors
function interpolate_vector3(start, end, progress, easing_type) {
    return [
        interpolate(start[0], end[0], progress, easing_type),
        interpolate(start[1], end[1], progress, easing_type),
        interpolate(start[2], end[2], progress, easing_type)
    ];
}

// Interpolate between two colors
function interpolate_color(start_color, end_color, progress, easing_type) {
    return [
        interpolate(start_color[0], end_color[0], progress, easing_type),
        interpolate(start_color[1], end_color[1], progress, easing_type),
        interpolate(start_color[2], end_color[2], progress, easing_type)
    ];
}
"#.to_string())
            .with_tags(vec!["math".to_string(), "interpolation".to_string(), "easing".to_string(), "animation".to_string()]),
        ]
    }

    fn create_utility_templates() -> Vec<ScriptTemplate> {
        vec![
            ScriptTemplate::new(
                "Debug Logger".to_string(),
                "Advanced logging system with different log levels and formatting".to_string(),
                TemplateCategory::Utility
            )
            .with_parameter(TemplateParameter::new(
                "log_level".to_string(),
                ParameterType::Choice(vec![
                    "debug".to_string(),
                    "info".to_string(),
                    "warning".to_string(),
                    "error".to_string()
                ]),
                "Minimum log level to display".to_string()
            ).with_default(RuntimeValue::String("info".to_string())).optional())
            .with_parameter(TemplateParameter::new(
                "include_timestamp".to_string(),
                ParameterType::Bool,
                "Whether to include timestamps in log messages".to_string()
            ).with_default(RuntimeValue::Bool(true)).optional())
            .with_script_code(r#"
let log_levels = { debug: 0, info: 1, warning: 2, error: 3 };
let current_log_level = log_levels["{log_level}"];
let include_timestamps = {include_timestamp};

function debug_log(level, message, data) {
    if (log_levels[level] < current_log_level) {
        return;
    }
    
    let timestamp = include_timestamps ? "[" + format_time(get_time()) + "] " : "";
    let level_tag = "[" + level.toUpperCase() + "] ";
    let formatted_message = timestamp + level_tag + message;
    
    if (data) {
        formatted_message += " | Data: " + JSON.stringify(data);
    }
    
    console.log(formatted_message);
    
    // Trigger log event for external systems
    trigger_event("debug.log", {
        level: level,
        message: message,
        data: data,
        timestamp: get_time()
    });
}

function format_time(timestamp) {
    let date = new Date(timestamp * 1000);
    return date.toISOString().substr(11, 8);
}

// Convenience functions
function log_debug(message, data) { debug_log("debug", message, data); }
function log_info(message, data) { debug_log("info", message, data); }
function log_warning(message, data) { debug_log("warning", message, data); }
function log_error(message, data) { debug_log("error", message, data); }
"#.to_string())
            .with_tags(vec!["utility".to_string(), "logging".to_string(), "debug".to_string(), "development".to_string()]),

            ScriptTemplate::new(
                "Performance Monitor".to_string(),
                "Monitor script performance and system resource usage".to_string(),
                TemplateCategory::Utility
            )
            .with_parameter(TemplateParameter::new(
                "monitoring_interval".to_string(),
                ParameterType::Range { min: 1.0, max: 60.0, step: 1.0 },
                "How often to collect performance data (seconds)".to_string()
            ).with_default(RuntimeValue::Float(5.0)).optional())
            .with_parameter(TemplateParameter::new(
                "log_performance".to_string(),
                ParameterType::Bool,
                "Whether to log performance data".to_string()
            ).with_default(RuntimeValue::Bool(true)).optional())
            .with_script_code(r#"
let performance_data = {
    frame_times: [],
    memory_usage: [],
    script_execution_times: {}
};

function start_performance_monitoring(monitoring_interval, log_performance) {
    let interval = {monitoring_interval};
    let should_log = {log_performance};
    
    while (true) {
        let frame_time = get_frame_time();
        let memory = get_memory_usage();
        let script_times = get_script_execution_times();
        
        // Store data
        performance_data.frame_times.push(frame_time);
        performance_data.memory_usage.push(memory);
        performance_data.script_execution_times = script_times;
        
        // Keep only last 100 samples
        if (performance_data.frame_times.length > 100) {
            performance_data.frame_times.shift();
            performance_data.memory_usage.shift();
        }
        
        // Calculate averages
        let avg_frame_time = performance_data.frame_times.reduce((a, b) => a + b) / performance_data.frame_times.length;
        let avg_memory = performance_data.memory_usage.reduce((a, b) => a + b) / performance_data.memory_usage.length;
        
        if (should_log) {
            log_info("Performance Stats", {
                avg_frame_time: avg_frame_time.toFixed(3) + "ms",
                fps: (1000 / avg_frame_time).toFixed(1),
                memory_usage: (avg_memory / 1024 / 1024).toFixed(1) + "MB",
                script_count: Object.keys(script_times).length
            });
        }
        
        // Trigger performance event
        trigger_event("performance.update", {
            frame_time: frame_time,
            memory_usage: memory,
            fps: 1000 / frame_time,
            script_times: script_times
        });
        
        // Check for performance issues
        if (frame_time > 16.67 * 2) { // Less than 30 FPS
            trigger_event("performance.warning", {
                type: "low_fps",
                frame_time: frame_time,
                fps: 1000 / frame_time
            });
        }
        
        wait(interval);
    }
}

function get_performance_summary() {
    let avg_frame_time = performance_data.frame_times.reduce((a, b) => a + b) / performance_data.frame_times.length;
    let avg_memory = performance_data.memory_usage.reduce((a, b) => a + b) / performance_data.memory_usage.length;
    
    return {
        average_frame_time: avg_frame_time,
        average_fps: 1000 / avg_frame_time,
        average_memory_mb: avg_memory / 1024 / 1024,
        sample_count: performance_data.frame_times.length
    };
}
"#.to_string())
            .with_tags(vec!["utility".to_string(), "performance".to_string(), "monitoring".to_string(), "optimization".to_string()]),
        ]
    }
}

#[derive(Debug)]
pub struct ScriptTemplateManager {
    library: TemplateLibrary,
    custom_templates: HashMap<String, CustomTemplate>,
    active_instances: HashMap<String, TemplateInstance>,
    builtin_loaded: bool,
}

#[derive(Debug, Clone)]
pub struct TemplateInstance {
    pub instance_id: String,
    pub template_id: String,
    pub parameters: HashMap<String, RuntimeValue>,
    pub generated_code: String,
    pub created_at: Instant,
    pub is_active: bool,
}

impl TemplateInstance {
    pub fn new(template_id: String, parameters: HashMap<String, RuntimeValue>, generated_code: String) -> Self {
        Self {
            instance_id: Uuid::new_v4().to_string(),
            template_id,
            parameters,
            generated_code,
            created_at: Instant::now(),
            is_active: true,
        }
    }
}

impl ScriptTemplateManager {
    pub fn new() -> RobinResult<Self> {
        Ok(Self {
            library: TemplateLibrary::new(),
            custom_templates: HashMap::new(),
            active_instances: HashMap::new(),
            builtin_loaded: false,
        })
    }

    pub fn load_builtin_templates(&mut self) -> RobinResult<()> {
        if self.builtin_loaded {
            return Ok(());
        }

        let builtin_templates = BuiltinTemplates::create_all();
        
        for template in builtin_templates {
            self.library.add_template(template);
        }
        
        self.builtin_loaded = true;
        
        println!("Loaded {} builtin script templates", self.library.get_template_count());
        Ok(())
    }

    pub fn add_custom_template(&mut self, name: String, custom_template: CustomTemplate) -> RobinResult<()> {
        let template_id = custom_template.template.id.clone();
        self.library.add_template(custom_template.template.clone());
        self.custom_templates.insert(name.clone(), custom_template);

        println!("Added custom template: {} (ID: {})", name, template_id);
        Ok(())
    }

    pub fn instantiate_template(&mut self, template_name: &str, parameters: HashMap<String, RuntimeValue>) -> RobinResult<String> {
        // Find template by name
        let template = self.library.templates.values()
            .find(|t| t.name == template_name)
            .cloned()
            .ok_or_else(|| crate::engine::error::RobinError::InvalidInput(
                format!("Template '{}' not found", template_name)
            ))?;

        let template_id = template.id.clone();

        // Record usage
        self.library.record_usage(&template_id);
        
        // Generate code
        let generated_code = template.instantiate(parameters.clone())?;
        
        // Create instance
        let instance = TemplateInstance::new(template_id, parameters, generated_code);
        let instance_id = instance.instance_id.clone();
        
        self.active_instances.insert(instance_id.clone(), instance);
        
        Ok(instance_id)
    }

    pub fn get_template_names(&self) -> Vec<String> {
        self.library.templates.values().map(|t| t.name.clone()).collect()
    }

    pub fn get_templates_by_category(&self, category: &TemplateCategory) -> Vec<&ScriptTemplate> {
        self.library.get_templates_by_category(category)
    }

    pub fn get_template_count(&self) -> usize {
        self.library.get_template_count()
    }

    pub fn get_active_instance_count(&self) -> u32 {
        self.active_instances.len() as u32
    }

    pub fn get_instance(&self, instance_id: &str) -> Option<&TemplateInstance> {
        self.active_instances.get(instance_id)
    }

    pub fn get_generated_code(&self, instance_id: &str) -> Option<&str> {
        self.active_instances.get(instance_id).map(|i| i.generated_code.as_str())
    }

    pub fn deactivate_instance(&mut self, instance_id: &str) -> RobinResult<()> {
        if let Some(instance) = self.active_instances.get_mut(instance_id) {
            instance.is_active = false;
        }
        Ok(())
    }

    pub fn remove_instance(&mut self, instance_id: &str) -> Option<TemplateInstance> {
        self.active_instances.remove(instance_id)
    }

    pub fn search_templates(&self, query: &str) -> Vec<&ScriptTemplate> {
        self.library.search_templates(query)
    }

    pub fn get_popular_templates(&self, limit: usize) -> Vec<&ScriptTemplate> {
        self.library.get_popular_templates(limit)
    }

    pub fn rate_template(&mut self, template_name: &str, rating: f32) -> RobinResult<()> {
        let template_id = self.library.templates.values()
            .find(|t| t.name == template_name)
            .map(|t| t.id.clone())
            .ok_or_else(|| crate::engine::error::RobinError::InvalidInput(
                format!("Template '{}' not found", template_name)
            ))?;

        self.library.rate_template(&template_id, rating)
    }

    pub fn shutdown(&mut self) -> RobinResult<()> {
        println!("Script Template Manager shutdown:");
        println!("  Total templates: {}", self.library.get_template_count());
        println!("  Custom templates: {}", self.custom_templates.len());
        println!("  Active instances: {}", self.active_instances.len());
        println!("  Categories: {}", self.library.get_categories().len());
        println!("  Total tags: {}", self.library.get_all_tags().len());

        self.active_instances.clear();
        self.custom_templates.clear();
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_type_validation() {
        let bool_param = ParameterType::Bool;
        assert!(bool_param.validate(&RuntimeValue::Bool(true)));
        assert!(!bool_param.validate(&RuntimeValue::Int(1)));

        let range_param = ParameterType::Range { min: 0.0, max: 10.0, step: 1.0 };
        assert!(range_param.validate(&RuntimeValue::Float(5.0)));
        assert!(!range_param.validate(&RuntimeValue::Float(15.0)));
        
        let choice_param = ParameterType::Choice(vec!["a".to_string(), "b".to_string()]);
        assert!(choice_param.validate(&RuntimeValue::String("a".to_string())));
        assert!(!choice_param.validate(&RuntimeValue::String("c".to_string())));
    }

    #[test]
    fn test_template_parameter_creation() {
        let param = TemplateParameter::new(
            "test_param".to_string(),
            ParameterType::Int,
            "Test parameter".to_string()
        )
        .optional()
        .with_default(RuntimeValue::Int(42))
        .with_display_name("Test Parameter".to_string())
        .with_tooltip("This is a test parameter".to_string());

        assert_eq!(param.name, "test_param");
        assert_eq!(param.display_name, "Test Parameter");
        assert!(!param.required);
        assert_eq!(param.default_value, Some(RuntimeValue::Int(42)));
        assert!(param.tooltip.is_some());
    }

    #[test]
    fn test_script_template_instantiation() {
        let template = ScriptTemplate::new(
            "Test Template".to_string(),
            "A test template".to_string(),
            TemplateCategory::Logic
        )
        .with_parameter(TemplateParameter::new(
            "name".to_string(),
            ParameterType::String,
            "Name parameter".to_string()
        ))
        .with_parameter(TemplateParameter::new(
            "count".to_string(),
            ParameterType::Int,
            "Count parameter".to_string()
        ))
        .with_script_code("Hello {name}, count is {count}".to_string());

        let mut params = HashMap::new();
        params.insert("name".to_string(), RuntimeValue::String("World".to_string()));
        params.insert("count".to_string(), RuntimeValue::Int(5));

        let result = template.instantiate(params).unwrap();
        assert_eq!(result, "Hello \"World\", count is 5");
    }

    #[test]
    fn test_template_library_operations() {
        let mut library = TemplateLibrary::new();
        
        let template = ScriptTemplate::new(
            "Test Template".to_string(),
            "A test template for testing".to_string(),
            TemplateCategory::Logic
        )
        .with_tags(vec!["test".to_string(), "example".to_string()]);

        let template_id = template.id.clone();
        library.add_template(template);

        assert_eq!(library.get_template_count(), 1);
        assert!(library.get_template(&template_id).is_some());
        
        let logic_templates = library.get_templates_by_category(&TemplateCategory::Logic);
        assert_eq!(logic_templates.len(), 1);
        
        let test_templates = library.get_templates_by_tag("test");
        assert_eq!(test_templates.len(), 1);
        
        let search_results = library.search_templates("test");
        assert_eq!(search_results.len(), 1);
    }

    #[test]
    fn test_template_manager_workflow() {
        let mut manager = ScriptTemplateManager::new().unwrap();
        manager.load_builtin_templates().unwrap();
        
        assert!(manager.get_template_count() > 0);
        
        let template_names = manager.get_template_names();
        assert!(!template_names.is_empty());
        
        // Try to instantiate a template (this will fail with built-in templates without proper parameters)
        let building_templates = manager.get_templates_by_category(&TemplateCategory::Building);
        assert!(!building_templates.is_empty());
    }
}