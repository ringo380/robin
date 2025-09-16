// Script Templates Library for Robin Engine
// Provides pre-built visual scripts for common game mechanics and patterns

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::engine::error::RobinResult;
use super::{ScriptValue, visual_editor::{NodeGraph, GraphNode, NodeConnection, NodeDefinition, NodeRegistry}};

/// Library of script templates for common game mechanics
#[derive(Debug)]
pub struct TemplateLibrary {
    /// All available templates organized by category
    templates: HashMap<String, ScriptTemplate>,
    
    /// Templates grouped by category
    categories: HashMap<String, Vec<String>>,
    
    /// Template metadata for searching and filtering
    metadata: HashMap<String, TemplateMetadata>,
    
    /// Usage statistics for templates
    usage_stats: HashMap<String, TemplateUsage>,
}

impl TemplateLibrary {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
            categories: HashMap::new(),
            metadata: HashMap::new(),
            usage_stats: HashMap::new(),
        }
    }
    
    /// Load default templates for common game mechanics
    pub fn load_default_templates(&mut self) -> RobinResult<()> {
        // Building and Construction Templates
        self.add_template(self.create_place_blocks_template())?;
        self.add_template(self.create_build_structure_template())?;
        self.add_template(self.create_copy_paste_template())?;
        self.add_template(self.create_terrain_modification_template())?;
        
        // NPC Behavior Templates  
        self.add_template(self.create_npc_patrol_template())?;
        self.add_template(self.create_npc_conversation_template())?;
        self.add_template(self.create_npc_merchant_template())?;
        self.add_template(self.create_npc_guard_template())?;
        
        // Player Interaction Templates
        self.add_template(self.create_inventory_management_template())?;
        self.add_template(self.create_quest_giver_template())?;
        self.add_template(self.create_door_interaction_template())?;
        self.add_template(self.create_pickup_item_template())?;
        
        // World Events Templates
        self.add_template(self.create_day_night_cycle_template())?;
        self.add_template(self.create_weather_system_template())?;
        self.add_template(self.create_seasonal_changes_template())?;
        self.add_template(self.create_random_events_template())?;
        
        // AI and Automation Templates
        self.add_template(self.create_auto_builder_template())?;
        self.add_template(self.create_resource_manager_template())?;
        self.add_template(self.create_defense_system_template())?;
        self.add_template(self.create_smart_lighting_template())?;
        
        // Game Mechanics Templates
        self.add_template(self.create_health_system_template())?;
        self.add_template(self.create_crafting_system_template())?;
        self.add_template(self.create_combat_system_template())?;
        self.add_template(self.create_magic_system_template())?;
        
        println!("Loaded {} default script templates", self.templates.len());
        Ok(())
    }
    
    /// Add a template to the library
    pub fn add_template(&mut self, template: ScriptTemplate) -> RobinResult<()> {
        let template_id = template.id.clone();
        let category = template.category.clone();
        
        // Add to categories
        self.categories.entry(category).or_insert_with(Vec::new).push(template_id.clone());
        
        // Create metadata
        let metadata = TemplateMetadata {
            name: template.name.clone(),
            description: template.description.clone(),
            category: template.category.clone(),
            tags: template.tags.clone(),
            difficulty: template.difficulty,
            author: template.author.clone(),
            version: template.version.clone(),
            created_at: chrono::Utc::now(),
        };
        
        // Initialize usage stats
        let usage = TemplateUsage {
            use_count: 0,
            last_used: None,
            average_rating: 0.0,
            rating_count: 0,
        };
        
        self.templates.insert(template_id.clone(), template);
        self.metadata.insert(template_id.clone(), metadata);
        self.usage_stats.insert(template_id, usage);
        
        Ok(())
    }
    
    /// Get template by ID
    pub fn get_template(&self, template_id: &str) -> RobinResult<&ScriptTemplate> {
        self.templates.get(template_id)
            .ok_or_else(|| format!("Template not found: {}", template_id).into())
    }
    
    /// Search templates by category, tags, or name
    pub fn search_templates(&self, query: &TemplateQuery) -> Vec<TemplateSearchResult> {
        let mut results = Vec::new();
        
        for (template_id, template) in &self.templates {
            let metadata = &self.metadata[template_id];
            let usage = &self.usage_stats[template_id];
            
            let mut score = 0.0;
            
            // Category match
            if let Some(ref category) = query.category {
                if template.category == *category {
                    score += 10.0;
                } else {
                    continue; // Skip if category doesn't match
                }
            }
            
            // Name/description match
            if let Some(ref search_text) = query.search_text {
                let search_lower = search_text.to_lowercase();
                if template.name.to_lowercase().contains(&search_lower) {
                    score += 8.0;
                }
                if template.description.to_lowercase().contains(&search_lower) {
                    score += 5.0;
                }
            }
            
            // Tags match
            if !query.tags.is_empty() {
                let matching_tags: Vec<_> = template.tags.iter()
                    .filter(|tag| query.tags.contains(tag))
                    .collect();
                score += matching_tags.len() as f32 * 3.0;
                
                if matching_tags.is_empty() && !query.tags.is_empty() {
                    continue; // Skip if no tags match when tags are specified
                }
            }
            
            // Difficulty match
            if let Some(difficulty) = query.difficulty {
                if template.difficulty == difficulty {
                    score += 2.0;
                }
            }
            
            // Popularity bonus
            score += (usage.use_count as f32).log10().max(0.0);
            score += usage.average_rating;
            
            if score > 0.0 {
                results.push(TemplateSearchResult {
                    template_id: template_id.clone(),
                    template: template.clone(),
                    metadata: metadata.clone(),
                    usage: usage.clone(),
                    relevance_score: score,
                });
            }
        }
        
        // Sort by relevance score
        results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
        results
    }
    
    /// Get templates by category
    pub fn get_templates_by_category(&self, category: &str) -> Vec<&ScriptTemplate> {
        if let Some(template_ids) = self.categories.get(category) {
            template_ids.iter()
                .filter_map(|id| self.templates.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Get all categories
    pub fn get_categories(&self) -> Vec<String> {
        self.categories.keys().cloned().collect()
    }
    
    /// Mark template as used
    pub fn mark_template_used(&mut self, template_id: &str) {
        if let Some(usage) = self.usage_stats.get_mut(template_id) {
            usage.use_count += 1;
            usage.last_used = Some(chrono::Utc::now());
        }
    }
    
    /// Rate a template
    pub fn rate_template(&mut self, template_id: &str, rating: f32) -> RobinResult<()> {
        if rating < 1.0 || rating > 5.0 {
            return Err("Rating must be between 1.0 and 5.0".into());
        }
        
        if let Some(usage) = self.usage_stats.get_mut(template_id) {
            let total_score = usage.average_rating * usage.rating_count as f32 + rating;
            usage.rating_count += 1;
            usage.average_rating = total_score / usage.rating_count as f32;
        }
        
        Ok(())
    }
    
    /// Get template count
    pub fn template_count(&self) -> usize {
        self.templates.len()
    }
    
    /// Get popular templates
    pub fn get_popular_templates(&self, limit: usize) -> Vec<TemplateSearchResult> {
        let mut results: Vec<_> = self.templates.iter().map(|(id, template)| {
            let metadata = &self.metadata[id];
            let usage = &self.usage_stats[id];
            
            TemplateSearchResult {
                template_id: id.clone(),
                template: template.clone(),
                metadata: metadata.clone(),
                usage: usage.clone(),
                relevance_score: usage.use_count as f32 + usage.average_rating * 10.0,
            }
        }).collect();
        
        results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
        results.truncate(limit);
        results
    }

    // Template creation methods for different categories
    
    /// Create template for placing blocks in patterns
    fn create_place_blocks_template(&self) -> ScriptTemplate {
        let mut template = ScriptTemplate::new(
            "place_blocks_pattern".to_string(),
            "Place Blocks Pattern".to_string(),
            "Building".to_string(),
        );
        
        template.description = "Creates a pattern of blocks based on user input. Useful for building walls, floors, or decorative patterns.".to_string();
        template.tags = vec!["building".to_string(), "blocks".to_string(), "pattern".to_string()];
        template.difficulty = TemplateDifficulty::Beginner;
        
        // The actual node graph would be created here
        // For brevity, we'll just set up the basic structure
        template.parameters = vec![
            TemplateParameter {
                name: "start_position".to_string(),
                display_name: "Start Position".to_string(),
                param_type: ParameterType::Vector3,
                default_value: Some(ScriptValue::Vector3([0.0, 0.0, 0.0])),
                description: "Starting position for the pattern".to_string(),
            },
            TemplateParameter {
                name: "block_type".to_string(),
                display_name: "Block Type".to_string(),
                param_type: ParameterType::String,
                default_value: Some(ScriptValue::String("Stone".to_string())),
                description: "Type of block to place".to_string(),
            },
            TemplateParameter {
                name: "pattern_size".to_string(),
                display_name: "Pattern Size".to_string(),
                param_type: ParameterType::Vector3,
                default_value: Some(ScriptValue::Vector3([5.0, 1.0, 5.0])),
                description: "Dimensions of the pattern (width, height, depth)".to_string(),
            },
        ];
        
        template
    }
    
    /// Create template for building complex structures
    fn create_build_structure_template(&self) -> ScriptTemplate {
        let mut template = ScriptTemplate::new(
            "build_structure".to_string(),
            "Build Structure".to_string(),
            "Building".to_string(),
        );
        
        template.description = "Automated building system that constructs complex structures from blueprints.".to_string();
        template.tags = vec!["building".to_string(), "structure".to_string(), "automation".to_string()];
        template.difficulty = TemplateDifficulty::Intermediate;
        
        template.parameters = vec![
            TemplateParameter {
                name: "structure_type".to_string(),
                display_name: "Structure Type".to_string(),
                param_type: ParameterType::String,
                default_value: Some(ScriptValue::String("house".to_string())),
                description: "Type of structure to build (house, tower, bridge, etc.)".to_string(),
            },
            TemplateParameter {
                name: "base_position".to_string(),
                display_name: "Base Position".to_string(),
                param_type: ParameterType::Vector3,
                default_value: Some(ScriptValue::Vector3([0.0, 0.0, 0.0])),
                description: "Base position for the structure".to_string(),
            },
            TemplateParameter {
                name: "materials".to_string(),
                display_name: "Materials".to_string(),
                param_type: ParameterType::Object,
                default_value: None,
                description: "Materials to use for different parts".to_string(),
            },
        ];
        
        template
    }
    
    /// Create template for NPC patrol behavior
    fn create_npc_patrol_template(&self) -> ScriptTemplate {
        let mut template = ScriptTemplate::new(
            "npc_patrol".to_string(),
            "NPC Patrol Behavior".to_string(),
            "NPC".to_string(),
        );
        
        template.description = "Makes an NPC patrol between waypoints with customizable behavior.".to_string();
        template.tags = vec!["npc".to_string(), "patrol".to_string(), "ai".to_string(), "movement".to_string()];
        template.difficulty = TemplateDifficulty::Beginner;
        
        template.parameters = vec![
            TemplateParameter {
                name: "waypoints".to_string(),
                display_name: "Waypoints".to_string(),
                param_type: ParameterType::Array,
                default_value: Some(ScriptValue::Array(vec![
                    ScriptValue::Vector3([0.0, 0.0, 0.0]),
                    ScriptValue::Vector3([10.0, 0.0, 10.0]),
                ])),
                description: "List of positions to patrol between".to_string(),
            },
            TemplateParameter {
                name: "wait_time".to_string(),
                display_name: "Wait Time".to_string(),
                param_type: ParameterType::Float,
                default_value: Some(ScriptValue::Float(2.0)),
                description: "Time to wait at each waypoint".to_string(),
            },
            TemplateParameter {
                name: "movement_speed".to_string(),
                display_name: "Movement Speed".to_string(),
                param_type: ParameterType::Float,
                default_value: Some(ScriptValue::Float(3.0)),
                description: "Speed of NPC movement".to_string(),
            },
        ];
        
        template
    }
    
    /// Create template for NPC conversation system
    fn create_npc_conversation_template(&self) -> ScriptTemplate {
        let mut template = ScriptTemplate::new(
            "npc_conversation".to_string(),
            "NPC Conversation System".to_string(),
            "NPC".to_string(),
        );
        
        template.description = "Interactive conversation system with dialogue trees and branching responses.".to_string();
        template.tags = vec!["npc".to_string(), "dialogue".to_string(), "conversation".to_string(), "story".to_string()];
        template.difficulty = TemplateDifficulty::Intermediate;
        
        template.parameters = vec![
            TemplateParameter {
                name: "greeting_message".to_string(),
                display_name: "Greeting Message".to_string(),
                param_type: ParameterType::String,
                default_value: Some(ScriptValue::String("Hello there! How can I help you?".to_string())),
                description: "Initial greeting when player approaches".to_string(),
            },
            TemplateParameter {
                name: "dialogue_tree".to_string(),
                display_name: "Dialogue Tree".to_string(),
                param_type: ParameterType::Object,
                default_value: None,
                description: "Conversation flow and responses".to_string(),
            },
        ];
        
        template
    }
    
    /// Create template for day/night cycle
    fn create_day_night_cycle_template(&self) -> ScriptTemplate {
        let mut template = ScriptTemplate::new(
            "day_night_cycle".to_string(),
            "Day/Night Cycle System".to_string(),
            "World Events".to_string(),
        );
        
        template.description = "Manages day/night transitions with lighting and NPC behavior changes.".to_string();
        template.tags = vec!["time".to_string(), "lighting".to_string(), "atmosphere".to_string(), "world".to_string()];
        template.difficulty = TemplateDifficulty::Advanced;
        
        template.parameters = vec![
            TemplateParameter {
                name: "day_length".to_string(),
                display_name: "Day Length (minutes)".to_string(),
                param_type: ParameterType::Float,
                default_value: Some(ScriptValue::Float(20.0)),
                description: "Length of a full day cycle in real minutes".to_string(),
            },
            TemplateParameter {
                name: "sunrise_hour".to_string(),
                display_name: "Sunrise Hour".to_string(),
                param_type: ParameterType::Int,
                default_value: Some(ScriptValue::Int(6)),
                description: "In-game hour when sunrise begins".to_string(),
            },
            TemplateParameter {
                name: "sunset_hour".to_string(),
                display_name: "Sunset Hour".to_string(),
                param_type: ParameterType::Int,
                default_value: Some(ScriptValue::Int(18)),
                description: "In-game hour when sunset begins".to_string(),
            },
        ];
        
        template
    }
    
    /// Create template for auto-builder AI
    fn create_auto_builder_template(&self) -> ScriptTemplate {
        let mut template = ScriptTemplate::new(
            "auto_builder_ai".to_string(),
            "Auto-Builder AI Assistant".to_string(),
            "AI & Automation".to_string(),
        );
        
        template.description = "AI that automatically builds and improves structures based on player preferences and needs.".to_string();
        template.tags = vec!["ai".to_string(), "automation".to_string(), "building".to_string(), "assistant".to_string()];
        template.difficulty = TemplateDifficulty::Expert;
        
        template.parameters = vec![
            TemplateParameter {
                name: "build_preferences".to_string(),
                display_name: "Building Preferences".to_string(),
                param_type: ParameterType::Object,
                default_value: None,
                description: "AI preferences for materials, styles, and layouts".to_string(),
            },
            TemplateParameter {
                name: "resource_management".to_string(),
                display_name: "Manage Resources".to_string(),
                param_type: ParameterType::Bool,
                default_value: Some(ScriptValue::Bool(true)),
                description: "Whether AI should manage resource collection".to_string(),
            },
        ];
        
        template
    }
    
    /// Create more templates for other categories...
    fn create_copy_paste_template(&self) -> ScriptTemplate {
        ScriptTemplate::new("copy_paste".to_string(), "Copy & Paste Tool".to_string(), "Building".to_string())
    }
    
    fn create_terrain_modification_template(&self) -> ScriptTemplate {
        ScriptTemplate::new("terrain_mod".to_string(), "Terrain Modification".to_string(), "Building".to_string())
    }
    
    fn create_npc_merchant_template(&self) -> ScriptTemplate {
        ScriptTemplate::new("npc_merchant".to_string(), "Merchant NPC".to_string(), "NPC".to_string())
    }
    
    fn create_npc_guard_template(&self) -> ScriptTemplate {
        ScriptTemplate::new("npc_guard".to_string(), "Guard NPC".to_string(), "NPC".to_string())
    }
    
    fn create_inventory_management_template(&self) -> ScriptTemplate {
        ScriptTemplate::new("inventory_mgmt".to_string(), "Inventory Management".to_string(), "Player Interaction".to_string())
    }
    
    fn create_quest_giver_template(&self) -> ScriptTemplate {
        ScriptTemplate::new("quest_giver".to_string(), "Quest Giver NPC".to_string(), "Player Interaction".to_string())
    }
    
    fn create_door_interaction_template(&self) -> ScriptTemplate {
        ScriptTemplate::new("door_interaction".to_string(), "Door Interaction".to_string(), "Player Interaction".to_string())
    }
    
    fn create_pickup_item_template(&self) -> ScriptTemplate {
        ScriptTemplate::new("pickup_item".to_string(), "Item Pickup".to_string(), "Player Interaction".to_string())
    }
    
    fn create_weather_system_template(&self) -> ScriptTemplate {
        ScriptTemplate::new("weather_system".to_string(), "Weather System".to_string(), "World Events".to_string())
    }
    
    fn create_seasonal_changes_template(&self) -> ScriptTemplate {
        ScriptTemplate::new("seasonal_changes".to_string(), "Seasonal Changes".to_string(), "World Events".to_string())
    }
    
    fn create_random_events_template(&self) -> ScriptTemplate {
        ScriptTemplate::new("random_events".to_string(), "Random Events".to_string(), "World Events".to_string())
    }
    
    fn create_resource_manager_template(&self) -> ScriptTemplate {
        ScriptTemplate::new("resource_manager".to_string(), "Resource Manager AI".to_string(), "AI & Automation".to_string())
    }
    
    fn create_defense_system_template(&self) -> ScriptTemplate {
        ScriptTemplate::new("defense_system".to_string(), "Defense System".to_string(), "AI & Automation".to_string())
    }
    
    fn create_smart_lighting_template(&self) -> ScriptTemplate {
        ScriptTemplate::new("smart_lighting".to_string(), "Smart Lighting".to_string(), "AI & Automation".to_string())
    }
    
    fn create_health_system_template(&self) -> ScriptTemplate {
        ScriptTemplate::new("health_system".to_string(), "Health System".to_string(), "Game Mechanics".to_string())
    }
    
    fn create_crafting_system_template(&self) -> ScriptTemplate {
        ScriptTemplate::new("crafting_system".to_string(), "Crafting System".to_string(), "Game Mechanics".to_string())
    }
    
    fn create_combat_system_template(&self) -> ScriptTemplate {
        ScriptTemplate::new("combat_system".to_string(), "Combat System".to_string(), "Game Mechanics".to_string())
    }
    
    fn create_magic_system_template(&self) -> ScriptTemplate {
        ScriptTemplate::new("magic_system".to_string(), "Magic System".to_string(), "Game Mechanics".to_string())
    }
}

/// Individual script template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptTemplate {
    /// Unique identifier
    pub id: String,
    
    /// Display name
    pub name: String,
    
    /// Category/group
    pub category: String,
    
    /// Description of what the template does
    pub description: String,
    
    /// Search tags
    pub tags: Vec<String>,
    
    /// Difficulty level
    pub difficulty: TemplateDifficulty,
    
    /// Template author
    pub author: String,
    
    /// Template version
    pub version: String,
    
    /// Parameters that can be customized
    pub parameters: Vec<TemplateParameter>,
    
    /// Node graph structure (simplified representation)
    pub node_structure: Vec<TemplateNode>,
    
    /// Connections between nodes
    pub connections: Vec<TemplateConnection>,
    
    /// Example usage or demo data
    pub examples: Vec<TemplateExample>,
}

impl ScriptTemplate {
    pub fn new(id: String, name: String, category: String) -> Self {
        Self {
            id,
            name,
            category,
            description: String::new(),
            tags: Vec::new(),
            difficulty: TemplateDifficulty::Beginner,
            author: "Robin Engine".to_string(),
            version: "1.0.0".to_string(),
            parameters: Vec::new(),
            node_structure: Vec::new(),
            connections: Vec::new(),
            examples: Vec::new(),
        }
    }
    
    /// Build a node graph from this template
    pub fn build_graph(&self, graph: &mut NodeGraph, node_registry: &NodeRegistry) -> RobinResult<()> {
        let mut node_id_mapping = HashMap::new();
        
        // Create nodes
        for template_node in &self.node_structure {
            if let Some(node_def) = node_registry.get_node_definition(&template_node.node_type) {
                let node_id = graph.add_node(node_def, template_node.position)?;
                node_id_mapping.insert(template_node.id.clone(), node_id);
                
                // Set parameter values
                if let Some(node) = graph.nodes.get_mut(&node_id_mapping[&template_node.id]) {
                    for (param_name, param_value) in &template_node.parameters {
                        node.properties.insert(param_name.clone(), param_value.clone());
                    }
                }
            }
        }
        
        // Create connections
        for connection in &self.connections {
            if let (Some(from_id), Some(to_id)) = (
                node_id_mapping.get(&connection.from_node),
                node_id_mapping.get(&connection.to_node)
            ) {
                graph.connect_nodes(from_id, &connection.from_output, to_id, &connection.to_input)?;
            }
        }
        
        Ok(())
    }
    
    /// Instantiate template with custom parameters
    pub fn instantiate(&self, custom_params: HashMap<String, ScriptValue>) -> ScriptTemplate {
        let mut instance = self.clone();
        
        // Apply custom parameters
        for node in &mut instance.node_structure {
            for (param_name, default_value) in &mut node.parameters {
                if let Some(custom_value) = custom_params.get(param_name) {
                    *default_value = custom_value.clone();
                }
            }
        }
        
        instance
    }
    
    /// Validate template structure
    pub fn validate(&self) -> RobinResult<TemplateValidationResult> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // Check for required fields
        if self.name.is_empty() {
            errors.push("Template name cannot be empty".to_string());
        }
        
        if self.category.is_empty() {
            errors.push("Template category cannot be empty".to_string());
        }
        
        if self.node_structure.is_empty() {
            warnings.push("Template has no nodes".to_string());
        }
        
        // Validate node references in connections
        let node_ids: std::collections::HashSet<_> = self.node_structure.iter().map(|n| &n.id).collect();
        for connection in &self.connections {
            if !node_ids.contains(&connection.from_node) {
                errors.push(format!("Connection references unknown from_node: {}", connection.from_node));
            }
            if !node_ids.contains(&connection.to_node) {
                errors.push(format!("Connection references unknown to_node: {}", connection.to_node));
            }
        }
        
        // Check parameter types
        for param in &self.parameters {
            if let Some(ref default_value) = param.default_value {
                if !param.param_type.matches_value(default_value) {
                    warnings.push(format!("Parameter '{}' default value doesn't match declared type", param.name));
                }
            }
        }
        
        Ok(TemplateValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        })
    }
}

/// Template parameter that can be customized
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateParameter {
    pub name: String,
    pub display_name: String,
    pub param_type: ParameterType,
    pub default_value: Option<ScriptValue>,
    pub description: String,
}

/// Parameter types for templates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    Bool,
    Int,
    Float,
    String,
    Vector3,
    Color,
    Object,
    Array,
}

impl ParameterType {
    pub fn matches_value(&self, value: &ScriptValue) -> bool {
        match (self, value) {
            (ParameterType::Bool, ScriptValue::Bool(_)) => true,
            (ParameterType::Int, ScriptValue::Int(_)) => true,
            (ParameterType::Float, ScriptValue::Float(_)) => true,
            (ParameterType::String, ScriptValue::String(_)) => true,
            (ParameterType::Vector3, ScriptValue::Vector3(_)) => true,
            (ParameterType::Color, ScriptValue::Color(_)) => true,
            (ParameterType::Object, ScriptValue::Object(_)) => true,
            (ParameterType::Array, ScriptValue::Array(_)) => true,
            _ => false,
        }
    }
}

/// Simplified node representation for templates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateNode {
    pub id: String,
    pub node_type: String,
    pub name: String,
    pub position: [f32; 2],
    pub parameters: HashMap<String, ScriptValue>,
}

/// Connection between template nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConnection {
    pub from_node: String,
    pub from_output: String,
    pub to_node: String,
    pub to_input: String,
}

/// Example usage of a template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateExample {
    pub name: String,
    pub description: String,
    pub parameters: HashMap<String, ScriptValue>,
    pub expected_result: String,
}

/// Difficulty levels for templates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemplateDifficulty {
    Beginner,     // Simple, few nodes, clear purpose
    Intermediate, // Moderate complexity, some logic
    Advanced,     // Complex logic, multiple systems
    Expert,       // Very complex, advanced concepts
}

impl std::fmt::Display for TemplateDifficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TemplateDifficulty::Beginner => write!(f, "Beginner"),
            TemplateDifficulty::Intermediate => write!(f, "Intermediate"),
            TemplateDifficulty::Advanced => write!(f, "Advanced"),
            TemplateDifficulty::Expert => write!(f, "Expert"),
        }
    }
}

/// Script category enum for organization
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScriptCategory {
    Building,
    NPC,
    PlayerInteraction,
    WorldEvents,
    AIAutomation,
    GameMechanics,
    Combat,
    Crafting,
    Quest,
    Audio,
    Visual,
    Utility,
    Custom,
}

impl ScriptCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            ScriptCategory::Building => "Building",
            ScriptCategory::NPC => "NPC",
            ScriptCategory::PlayerInteraction => "Player Interaction",
            ScriptCategory::WorldEvents => "World Events",
            ScriptCategory::AIAutomation => "AI & Automation",
            ScriptCategory::GameMechanics => "Game Mechanics",
            ScriptCategory::Combat => "Combat",
            ScriptCategory::Crafting => "Crafting",
            ScriptCategory::Quest => "Quest",
            ScriptCategory::Audio => "Audio",
            ScriptCategory::Visual => "Visual",
            ScriptCategory::Utility => "Utility",
            ScriptCategory::Custom => "Custom",
        }
    }
    
    pub fn all_categories() -> Vec<ScriptCategory> {
        vec![
            ScriptCategory::Building,
            ScriptCategory::NPC,
            ScriptCategory::PlayerInteraction,
            ScriptCategory::WorldEvents,
            ScriptCategory::AIAutomation,
            ScriptCategory::GameMechanics,
            ScriptCategory::Combat,
            ScriptCategory::Crafting,
            ScriptCategory::Quest,
            ScriptCategory::Audio,
            ScriptCategory::Visual,
            ScriptCategory::Utility,
            ScriptCategory::Custom,
        ]
    }
}

/// Template metadata for search and organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMetadata {
    pub name: String,
    pub description: String,
    pub category: String,
    pub tags: Vec<String>,
    pub difficulty: TemplateDifficulty,
    pub author: String,
    pub version: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Template usage statistics
#[derive(Debug, Clone)]
pub struct TemplateUsage {
    pub use_count: u64,
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
    pub average_rating: f32,
    pub rating_count: u32,
}

/// Query for searching templates
#[derive(Debug, Default)]
pub struct TemplateQuery {
    pub search_text: Option<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub difficulty: Option<TemplateDifficulty>,
    pub author: Option<String>,
    pub min_rating: Option<f32>,
    pub sort_by: TemplateSortBy,
}

/// Sort options for template search
#[derive(Debug)]
pub enum TemplateSortBy {
    Relevance,
    Name,
    Popularity,
    Rating,
    Recent,
    Difficulty,
}

impl Default for TemplateSortBy {
    fn default() -> Self {
        TemplateSortBy::Relevance
    }
}

/// Template search result
#[derive(Debug, Clone)]
pub struct TemplateSearchResult {
    pub template_id: String,
    pub template: ScriptTemplate,
    pub metadata: TemplateMetadata,
    pub usage: TemplateUsage,
    pub relevance_score: f32,
}

/// Template validation result
#[derive(Debug)]
pub struct TemplateValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Template builder for easy creation
pub struct TemplateBuilder {
    template: ScriptTemplate,
    node_counter: usize,
}

impl TemplateBuilder {
    pub fn new(name: String, category: String) -> Self {
        let id = format!("template_{}", uuid::Uuid::new_v4().simple());
        Self {
            template: ScriptTemplate::new(id, name, category),
            node_counter: 0,
        }
    }
    
    pub fn with_description(mut self, description: String) -> Self {
        self.template.description = description;
        self
    }
    
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.template.tags = tags;
        self
    }
    
    pub fn with_difficulty(mut self, difficulty: TemplateDifficulty) -> Self {
        self.template.difficulty = difficulty;
        self
    }
    
    pub fn add_parameter(mut self, param: TemplateParameter) -> Self {
        self.template.parameters.push(param);
        self
    }
    
    pub fn add_node(mut self, node_type: String, name: String, position: [f32; 2]) -> (Self, String) {
        self.node_counter += 1;
        let node_id = format!("node_{}", self.node_counter);
        
        let template_node = TemplateNode {
            id: node_id.clone(),
            node_type,
            name,
            position,
            parameters: HashMap::new(),
        };
        
        self.template.node_structure.push(template_node);
        (self, node_id)
    }
    
    pub fn connect_nodes(mut self, from_node: String, from_output: String, to_node: String, to_input: String) -> Self {
        let connection = TemplateConnection {
            from_node,
            from_output,
            to_node,
            to_input,
        };
        
        self.template.connections.push(connection);
        self
    }
    
    pub fn build(self) -> ScriptTemplate {
        self.template
    }
}

// Helper functions for creating common parameter types
impl TemplateParameter {
    pub fn vector3(name: String, display_name: String, default: [f32; 3]) -> Self {
        Self {
            name,
            display_name,
            param_type: ParameterType::Vector3,
            default_value: Some(ScriptValue::Vector3(default)),
            description: String::new(),
        }
    }
    
    pub fn string(name: String, display_name: String, default: String) -> Self {
        Self {
            name,
            display_name,
            param_type: ParameterType::String,
            default_value: Some(ScriptValue::String(default)),
            description: String::new(),
        }
    }
    
    pub fn float(name: String, display_name: String, default: f64) -> Self {
        Self {
            name,
            display_name,
            param_type: ParameterType::Float,
            default_value: Some(ScriptValue::Float(default)),
            description: String::new(),
        }
    }
    
    pub fn bool(name: String, display_name: String, default: bool) -> Self {
        Self {
            name,
            display_name,
            param_type: ParameterType::Bool,
            default_value: Some(ScriptValue::Bool(default)),
            description: String::new(),
        }
    }
}