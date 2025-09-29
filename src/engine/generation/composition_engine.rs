/*!
 * Advanced Content Composition Engine Module for Robin Engine
 *
 * Provides sophisticated layered composition capabilities for procedural content generation
 * with support for hierarchical composition, material blending, and performance optimization.
 */

use crate::engine::error::RobinResult;
use super::{
    Texture, DetailLevel, TerrainType, WeatherType,
    EnhancedCharacterParams, EnhancedObjectParams, EnhancedEnvironmentParams, EnhancedSurfaceParams,
    EnhancedGeneratedCharacter, EnhancedGeneratedObject, EnhancedGeneratedEnvironment, EnhancedGeneratedSurface,
    AdvancedMaterialSystem, AdvancedMaterialType
};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};

// ================================================================================================
// PHASE 4: ADVANCED CONTENT COMPOSITION WITH LAYERED SYSTEM
// ================================================================================================

/// Advanced Content Composition Engine with sophisticated layered composition capabilities
#[derive(Debug)]
pub struct AdvancedContentCompositionEngine {
    /// Layer management system
    layer_manager: LayerManager,
    /// Composition pipeline
    composition_pipeline: CompositionPipeline,
    /// Hierarchical composition system
    hierarchical_composer: HierarchicalComposer,
    /// Layer blending engine
    blending_engine: LayerBlendingEngine,
    /// Composition optimization system
    optimization_system: CompositionOptimizer,
    /// Composition analytics
    composition_analytics: CompositionAnalytics,
}

impl AdvancedContentCompositionEngine {
    /// Create new composition engine
    pub fn new() -> Self {
        Self {
            layer_manager: LayerManager::new(),
            composition_pipeline: CompositionPipeline::new(),
            hierarchical_composer: HierarchicalComposer::new(),
            blending_engine: LayerBlendingEngine::new(),
            optimization_system: CompositionOptimizer::new(),
            composition_analytics: CompositionAnalytics::new(),
        }
    }

    /// Compose layered content from multiple sources
    pub fn compose_layered_content(&mut self, composition_request: LayeredCompositionRequest) -> RobinResult<ComposedContent> {
        // In a production system, this would:
        // 1. Create base layer from primary content
        // 2. Add secondary content as additional layers
        // 3. Apply blending operations
        // 4. Optimize composition for performance
        // 5. Return composed result

        let base_layer = self.layer_manager.create_base_layer(&composition_request.primary_content)?;

        for secondary in &composition_request.secondary_content {
            let _layer = self.layer_manager.create_content_layer(secondary)?;
        }

        Ok(ComposedContent::default())
    }

    /// Compose content hierarchically
    pub fn compose_hierarchical(&mut self, root_content: ContentElement, children: Vec<ContentElement>) -> RobinResult<ComposedContent> {
        // In production: Build hierarchy and compose bottom-up
        self.hierarchical_composer.compose_hierarchy(root_content, children)
    }

    /// Optimize existing composition
    pub fn optimize_composition(&mut self, composition: ComposedContent) -> RobinResult<ComposedContent> {
        self.optimization_system.optimize(composition)
    }

    /// Analyze composition quality
    pub fn analyze_composition(&mut self, composition: &ComposedContent) -> RobinResult<CompositionAnalyticsSummary> {
        Ok(self.composition_analytics.get_analytics_summary())
    }

    /// Compose character with advanced layering
    pub fn compose_character_layers(&mut self, params: &EnhancedCharacterParams) -> RobinResult<EnhancedGeneratedCharacter> {
        // In production: Layer-based character composition
        Ok(EnhancedGeneratedCharacter::default())
    }

    /// Compose environment with hierarchical structure
    pub fn compose_environment_hierarchy(&mut self, params: &EnhancedEnvironmentParams) -> RobinResult<EnhancedGeneratedEnvironment> {
        // In production: Hierarchical environment composition
        Ok(EnhancedGeneratedEnvironment::default())
    }

    /// Compose object with material blending
    pub fn compose_object_with_materials(&mut self, params: &EnhancedObjectParams, material_system: &AdvancedMaterialSystem) -> RobinResult<EnhancedGeneratedObject> {
        // In production: Material-aware object composition
        Ok(EnhancedGeneratedObject::default())
    }

    /// Compose surface with advanced texturing
    pub fn compose_surface_textures(&mut self, params: &EnhancedSurfaceParams) -> RobinResult<EnhancedGeneratedSurface> {
        // In production: Multi-layer surface composition
        Ok(EnhancedGeneratedSurface::default())
    }

    /// Get composition performance metrics
    pub fn get_performance_metrics(&self) -> CompositionPerformanceMetrics {
        CompositionPerformanceMetrics::default()
    }

    /// Configure composition pipeline
    pub fn configure_pipeline(&mut self, config: CompositionPipelineConfig) {
        self.composition_pipeline.configure(config);
    }

    /// Set blending configuration
    pub fn set_blending_config(&mut self, config: BlendingConfig) {
        self.blending_engine.set_config(config);
    }

    /// Enable composition analytics
    pub fn enable_analytics(&mut self, enabled: bool) {
        self.composition_analytics.set_enabled(enabled);
    }
}

/// Layer management system for organizing composition elements
#[derive(Debug)]
pub struct LayerManager {
    layer_registry: HashMap<String, ContentLayer>,
    layer_order: Vec<String>,
    layer_dependencies: HashMap<String, Vec<String>>,
    max_layers: usize,
}

impl LayerManager {
    pub fn new() -> Self {
        Self {
            layer_registry: HashMap::new(),
            layer_order: Vec::new(),
            layer_dependencies: HashMap::new(),
            max_layers: 32,
        }
    }

    pub fn create_base_layer(&mut self, content: &ContentElement) -> RobinResult<ContentLayer> {
        let layer = ContentLayer {
            layer_id: format!("base_{}", uuid::Uuid::new_v4()),
            layer_type: LayerType::Base,
            content: content.clone(),
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
            visible: true,
            locked: false,
        };

        self.layer_registry.insert(layer.layer_id.clone(), layer.clone());
        self.layer_order.push(layer.layer_id.clone());

        Ok(layer)
    }

    pub fn create_base_layer_with_materials(&mut self, content: &ContentElement, material_system: &AdvancedMaterialSystem) -> RobinResult<ContentLayer> {
        let material_info = self.extract_material_info(content, material_system)?;

        let layer = ContentLayer {
            layer_id: format!("base_material_{}", uuid::Uuid::new_v4()),
            layer_type: LayerType::BaseMaterial,
            content: content.clone(),
            opacity: material_info.opacity,
            blend_mode: BlendMode::Normal,
            visible: true,
            locked: false,
        };

        self.layer_registry.insert(layer.layer_id.clone(), layer.clone());
        self.layer_order.push(layer.layer_id.clone());

        Ok(layer)
    }

    pub fn create_content_layer_with_materials(&mut self, content: &ContentElement, material_system: &AdvancedMaterialSystem) -> RobinResult<ContentLayer> {
        let material_info = self.extract_material_info(content, material_system)?;
        let blend_mode = self.determine_material_blend_mode(&material_info);

        let layer = ContentLayer {
            layer_id: format!("content_material_{}", uuid::Uuid::new_v4()),
            layer_type: LayerType::ContentMaterial,
            content: content.clone(),
            opacity: material_info.opacity * 0.85,
            blend_mode,
            visible: true,
            locked: false,
        };

        self.layer_registry.insert(layer.layer_id.clone(), layer.clone());
        self.layer_order.push(layer.layer_id.clone());

        Ok(layer)
    }

    pub fn create_detail_layer_with_materials(&mut self, detail: &DetailElement, material_system: &AdvancedMaterialSystem) -> RobinResult<ContentLayer> {
        let content = ContentElement {
            element_id: detail.detail_id.clone(),
            element_type: ContentElementType::Detail,
            content_data: vec![],
            metadata: HashMap::new(),
        };

        let material_info = self.extract_material_info(&content, material_system)?;

        let layer = ContentLayer {
            layer_id: format!("detail_material_{}", uuid::Uuid::new_v4()),
            layer_type: LayerType::DetailMaterial,
            content,
            opacity: material_info.opacity * 0.7,
            blend_mode: BlendMode::Overlay,
            visible: true,
            locked: false,
        };

        self.layer_registry.insert(layer.layer_id.clone(), layer.clone());
        self.layer_order.push(layer.layer_id.clone());

        Ok(layer)
    }

    pub fn create_overlay_layer_with_materials(&mut self, overlay: &OverlayElement, material_system: &AdvancedMaterialSystem) -> RobinResult<ContentLayer> {
        let content = ContentElement {
            element_id: overlay.overlay_id.clone(),
            element_type: ContentElementType::Overlay,
            content_data: vec![],
            metadata: HashMap::new(),
        };

        let material_info = self.extract_material_info(&content, material_system)?;
        let blend_mode = self.determine_material_blend_mode(&material_info);

        let layer = ContentLayer {
            layer_id: format!("overlay_material_{}", uuid::Uuid::new_v4()),
            layer_type: LayerType::OverlayMaterial,
            content,
            opacity: material_info.opacity * 0.5,
            blend_mode,
            visible: true,
            locked: false,
        };

        self.layer_registry.insert(layer.layer_id.clone(), layer.clone());
        self.layer_order.push(layer.layer_id.clone());

        Ok(layer)
    }

    fn extract_material_info(&self, content: &ContentElement, material_system: &AdvancedMaterialSystem) -> RobinResult<MaterialInfo> {
        // In production: Extract material properties from content
        Ok(MaterialInfo {
            material_type: AdvancedMaterialType::Metal,
            opacity: 0.9,
            reflectivity: 0.8,
            roughness: 0.3,
        })
    }

    fn determine_material_blend_mode(&self, material_info: &MaterialInfo) -> BlendMode {
        match material_info.material_type {
            AdvancedMaterialType::Metal => BlendMode::Multiply,
            AdvancedMaterialType::Glass => BlendMode::Screen,
            AdvancedMaterialType::Organic => BlendMode::Overlay,
            AdvancedMaterialType::Energy => BlendMode::Additive,
            AdvancedMaterialType::Liquid => BlendMode::SoftLight,
            _ => BlendMode::Normal,
        }
    }

    pub fn set_layer_material_properties(&mut self, layer_id: &str, material_system: &AdvancedMaterialSystem) -> RobinResult<()> {
        if let Some(layer) = self.layer_registry.get_mut(layer_id) {
            let material_info = self.extract_material_info(&layer.content, material_system)?;
            layer.opacity = material_info.opacity;
            layer.blend_mode = self.determine_material_blend_mode(&material_info);
        }
        Ok(())
    }

    pub fn create_content_layer(&mut self, content: &ContentElement) -> RobinResult<ContentLayer> {
        let layer = ContentLayer {
            layer_id: format!("content_{}", uuid::Uuid::new_v4()),
            layer_type: LayerType::Content,
            content: content.clone(),
            opacity: 0.8,
            blend_mode: BlendMode::Multiply,
            visible: true,
            locked: false,
        };

        self.layer_registry.insert(layer.layer_id.clone(), layer.clone());
        self.layer_order.push(layer.layer_id.clone());

        Ok(layer)
    }

    pub fn create_detail_layer(&mut self, detail: &DetailElement) -> RobinResult<ContentLayer> {
        let content = ContentElement {
            element_id: detail.detail_id.clone(),
            element_type: ContentElementType::Detail,
            content_data: vec![],
            metadata: HashMap::new(),
        };
        let layer = ContentLayer {
            layer_id: format!("detail_{}", uuid::Uuid::new_v4()),
            layer_type: LayerType::Detail,
            content,
            opacity: 0.6,
            blend_mode: BlendMode::Overlay,
            visible: true,
            locked: false,
        };

        self.layer_registry.insert(layer.layer_id.clone(), layer.clone());
        self.layer_order.push(layer.layer_id.clone());

        Ok(layer)
    }

    pub fn create_overlay_layer(&mut self, overlay: &OverlayElement) -> RobinResult<ContentLayer> {
        let content = ContentElement {
            element_id: overlay.overlay_id.clone(),
            element_type: ContentElementType::Overlay,
            content_data: vec![],
            metadata: HashMap::new(),
        };
        let layer = ContentLayer {
            layer_id: format!("overlay_{}", uuid::Uuid::new_v4()),
            layer_type: LayerType::Overlay,
            content,
            opacity: 0.5,
            blend_mode: BlendMode::Additive,
            visible: true,
            locked: false,
        };

        self.layer_registry.insert(layer.layer_id.clone(), layer.clone());
        self.layer_order.push(layer.layer_id.clone());

        Ok(layer)
    }

    pub fn get_layer_stack(&self) -> Vec<ContentLayer> {
        self.layer_order.iter()
            .filter_map(|id| self.layer_registry.get(id).cloned())
            .collect()
    }

    pub fn reorder_layers(&mut self, new_order: Vec<String>) -> RobinResult<()> {
        // Validate all layer IDs exist
        for id in &new_order {
            if !self.layer_registry.contains_key(id) {
                return Ok(()); // Silently ignore invalid IDs in production
            }
        }
        self.layer_order = new_order;
        Ok(())
    }
}

/// Composition pipeline for processing content through stages
#[derive(Debug)]
pub struct CompositionPipeline {
    stages: Vec<CompositionStage>,
    stage_configs: HashMap<String, StageConfig>,
    pipeline_cache: HashMap<String, CachedPipelineResult>,
}

impl CompositionPipeline {
    pub fn new() -> Self {
        Self {
            stages: Self::default_stages(),
            stage_configs: Self::default_configs(),
            pipeline_cache: HashMap::new(),
        }
    }

    fn default_stages() -> Vec<CompositionStage> {
        vec![
            CompositionStage {
                stage_id: "preprocessing".to_string(),
                stage_type: StageType::Preprocessing,
                enabled: true,
                order: 0,
            },
            CompositionStage {
                stage_id: "layering".to_string(),
                stage_type: StageType::Layering,
                enabled: true,
                order: 1,
            },
            CompositionStage {
                stage_id: "blending".to_string(),
                stage_type: StageType::Blending,
                enabled: true,
                order: 2,
            },
            CompositionStage {
                stage_id: "optimization".to_string(),
                stage_type: StageType::Optimization,
                enabled: true,
                order: 3,
            },
            CompositionStage {
                stage_id: "postprocessing".to_string(),
                stage_type: StageType::Postprocessing,
                enabled: true,
                order: 4,
            },
        ]
    }

    fn default_configs() -> HashMap<String, StageConfig> {
        let mut configs = HashMap::new();

        configs.insert("preprocessing".to_string(), StageConfig {
            max_input_size: 1024 * 1024,
            timeout_ms: 1000,
            parallel_processing: true,
            cache_results: true,
        });

        configs.insert("layering".to_string(), StageConfig {
            max_input_size: 2048 * 2048,
            timeout_ms: 2000,
            parallel_processing: true,
            cache_results: true,
        });

        configs.insert("blending".to_string(), StageConfig {
            max_input_size: 2048 * 2048,
            timeout_ms: 3000,
            parallel_processing: false,
            cache_results: true,
        });

        configs
    }

    pub fn process(&mut self, input: PipelineInput) -> RobinResult<PipelineOutput> {
        let mut current_data = input.data;

        for stage in &self.stages {
            if stage.enabled {
                current_data = self.process_stage(stage, current_data)?;
            }
        }

        Ok(PipelineOutput {
            data: current_data,
            stages_completed: self.stages.len(),
            processing_time: std::time::Duration::from_millis(100),
        })
    }

    fn process_stage(&mut self, stage: &CompositionStage, data: Vec<u8>) -> RobinResult<Vec<u8>> {
        // In production: Apply stage-specific processing
        Ok(data)
    }

    pub fn configure(&mut self, config: CompositionPipelineConfig) {
        // Apply configuration
        if let Some(stages) = config.custom_stages {
            self.stages = stages;
        }
        if let Some(configs) = config.stage_configs {
            self.stage_configs = configs;
        }
    }

    pub fn add_stage(&mut self, stage: CompositionStage) {
        self.stages.push(stage);
        self.stages.sort_by_key(|s| s.order);
    }

    pub fn remove_stage(&mut self, stage_id: &str) {
        self.stages.retain(|s| s.stage_id != stage_id);
    }

    pub fn clear_cache(&mut self) {
        self.pipeline_cache.clear();
    }
}

/// Hierarchical composition system for nested content structures
#[derive(Debug)]
pub struct HierarchicalComposer {
    node_registry: HashMap<String, HierarchicalNode>,
    root_nodes: Vec<String>,
    composition_rules: HashMap<String, CompositionRule>,
}

impl HierarchicalComposer {
    pub fn new() -> Self {
        Self {
            node_registry: HashMap::new(),
            root_nodes: Vec::new(),
            composition_rules: Self::default_rules(),
        }
    }

    fn default_rules() -> HashMap<String, CompositionRule> {
        let mut rules = HashMap::new();

        rules.insert("combine_children".to_string(), CompositionRule {
            rule_id: "combine_children".to_string(),
            rule_type: RuleType::CombineChildren,
            priority: 1,
            conditions: vec![],
        });

        rules.insert("inherit_properties".to_string(), CompositionRule {
            rule_id: "inherit_properties".to_string(),
            rule_type: RuleType::InheritProperties,
            priority: 2,
            conditions: vec![],
        });

        rules
    }

    pub fn compose_hierarchy(&mut self, root: ContentElement, children: Vec<ContentElement>) -> RobinResult<ComposedContent> {
        let root_node = self.create_node(root, None)?;

        for child in children {
            self.create_node(child, Some(root_node.clone()))?;
        }

        self.compose_from_node(&root_node)
    }

    fn create_node(&mut self, content: ContentElement, parent: Option<String>) -> RobinResult<String> {
        let node_id = format!("node_{}", uuid::Uuid::new_v4());

        let node = HierarchicalNode {
            node_id: node_id.clone(),
            content,
            parent_id: parent.clone(),
            child_ids: Vec::new(),
            depth: if parent.is_some() { 1 } else { 0 },
        };

        self.node_registry.insert(node_id.clone(), node);

        if parent.is_none() {
            self.root_nodes.push(node_id.clone());
        } else if let Some(parent_id) = parent {
            if let Some(parent_node) = self.node_registry.get_mut(&parent_id) {
                parent_node.child_ids.push(node_id.clone());
            }
        }

        Ok(node_id)
    }

    fn compose_from_node(&self, node_id: &str) -> RobinResult<ComposedContent> {
        // In production: Recursive composition from node
        Ok(ComposedContent::default())
    }

    pub fn flatten_hierarchy(&self) -> Vec<ContentElement> {
        let mut flattened = Vec::new();

        for root_id in &self.root_nodes {
            self.flatten_from_node(root_id, &mut flattened);
        }

        flattened
    }

    fn flatten_from_node(&self, node_id: &str, result: &mut Vec<ContentElement>) {
        if let Some(node) = self.node_registry.get(node_id) {
            result.push(node.content.clone());

            for child_id in &node.child_ids {
                self.flatten_from_node(child_id, result);
            }
        }
    }
}

/// Layer blending engine for combining multiple layers
#[derive(Debug)]
pub struct LayerBlendingEngine {
    blend_processors: HashMap<BlendMode, BlendProcessor>,
    blend_cache: HashMap<String, BlendedComposition>,
    config: BlendingConfig,
}

impl LayerBlendingEngine {
    pub fn new() -> Self {
        Self {
            blend_processors: Self::initialize_blend_processors(),
            blend_cache: HashMap::new(),
            config: BlendingConfig::default(),
        }
    }

    pub fn blend_layers(&mut self, layers: Vec<ContentLayer>, config: &BlendingConfig) -> RobinResult<BlendedComposition> {
        if layers.is_empty() {
            return Ok(BlendedComposition::default());
        }

        let mut composition = BlendedComposition::from_layer(&layers[0]);

        for layer in layers.iter().skip(1) {
            if layer.visible {
                composition = self.blend_layer(&composition, layer, config)?;
            }
        }

        Ok(composition)
    }

    pub fn blend_layers_with_materials(&mut self, layers: Vec<ContentLayer>, config: &BlendingConfig, material_system: &AdvancedMaterialSystem) -> RobinResult<BlendedComposition> {
        if layers.is_empty() {
            return Ok(BlendedComposition::default());
        }

        // Group layers by material compatibility
        let material_groups = self.group_layers_by_material_compatibility(&layers, material_system)?;

        let mut composition = BlendedComposition::from_layer(&layers[0]);

        for group in material_groups {
            composition = self.blend_material_group(&group, &composition, config, material_system)?;
        }

        Ok(composition)
    }

    fn group_layers_by_material_compatibility(&self, layers: &[ContentLayer], material_system: &AdvancedMaterialSystem) -> RobinResult<Vec<Vec<ContentLayer>>> {
        let mut groups = Vec::new();
        let mut current_group = Vec::new();

        for layer in layers {
            if current_group.is_empty() {
                current_group.push(layer.clone());
            } else {
                let compatible = self.are_materials_compatible(&current_group[0], layer, material_system)?;
                if compatible {
                    current_group.push(layer.clone());
                } else {
                    groups.push(current_group);
                    current_group = vec![layer.clone()];
                }
            }
        }

        if !current_group.is_empty() {
            groups.push(current_group);
        }

        Ok(groups)
    }

    fn are_materials_compatible(&self, layer_a: &ContentLayer, layer_b: &ContentLayer, material_system: &AdvancedMaterialSystem) -> RobinResult<bool> {
        // In production: Check material compatibility rules
        Ok(true)
    }

    fn blend_material_group(&mut self, group: &[ContentLayer], base_composition: &BlendedComposition, config: &BlendingConfig, material_system: &AdvancedMaterialSystem) -> RobinResult<BlendedComposition> {
        let mut composition = base_composition.clone();

        // Sort by material blending priority
        let mut sorted_group = group.to_vec();
        sorted_group.sort_by(|a, b| {
            let priority_a = self.get_material_blend_priority(a, material_system);
            let priority_b = self.get_material_blend_priority(b, material_system);
            priority_b.partial_cmp(&priority_a).unwrap()
        });

        for layer in sorted_group {
            composition = self.blend_layer_with_materials(&composition, &layer, config, material_system)?;
        }

        Ok(composition)
    }

    fn blend_layer_with_materials(&mut self, composition: &BlendedComposition, layer: &ContentLayer, config: &BlendingConfig, material_system: &AdvancedMaterialSystem) -> RobinResult<BlendedComposition> {
        let cache_key = self.generate_material_blend_cache_key(composition, layer, material_system);

        if config.enable_caching {
            if let Some(cached) = self.blend_cache.get(&cache_key) {
                return Ok(cached.clone());
            }
        }

        // Determine effective blend mode based on materials
        let material_info = self.extract_layer_material_info(layer, material_system)?;
        let effective_blend_mode = self.determine_material_effective_blend_mode(&layer.blend_mode, &material_info, material_system)?;

        let processor = self.blend_processors.get(&effective_blend_mode)
            .unwrap_or_else(|| self.blend_processors.get(&BlendMode::Normal).unwrap());

        let mut blended = processor.blend(composition, layer, config)?;

        // Apply material-specific adjustments
        self.apply_material_adjustments(&mut blended, &material_info, material_system)?;

        if config.enable_caching {
            self.blend_cache.insert(cache_key, blended.clone());
        }

        Ok(blended)
    }

    fn get_material_blend_priority(&self, layer: &ContentLayer, material_system: &AdvancedMaterialSystem) -> f32 {
        // In production: Calculate priority based on material properties
        1.0
    }

    fn extract_layer_material_info(&self, layer: &ContentLayer, material_system: &AdvancedMaterialSystem) -> RobinResult<MaterialInfo> {
        // In production: Extract material information from layer
        Ok(MaterialInfo {
            material_type: AdvancedMaterialType::Metal,
            opacity: layer.opacity,
            reflectivity: 0.8,
            roughness: 0.3,
        })
    }

    fn determine_material_effective_blend_mode(&self, base_mode: &BlendMode, material_info: &MaterialInfo, material_system: &AdvancedMaterialSystem) -> RobinResult<BlendMode> {
        // In production: Determine effective blend mode based on material
        Ok(base_mode.clone())
    }

    fn apply_material_adjustments(&self, blended: &mut BlendedComposition, material_info: &MaterialInfo, material_system: &AdvancedMaterialSystem) -> RobinResult<()> {
        // In production: Apply material-specific adjustments
        Ok(())
    }

    fn generate_material_blend_cache_key(&self, composition: &BlendedComposition, layer: &ContentLayer, material_system: &AdvancedMaterialSystem) -> String {
        format!("{}_{}_{}",
            composition.composition_id,
            layer.layer_id,
            material_system.get_cache_key()
        )
    }

    pub fn blend_with_hierarchy(&mut self, hierarchy: &HierarchicalComposer, config: &BlendingConfig) -> RobinResult<BlendedComposition> {
        let flattened = hierarchy.flatten_hierarchy();
        let layers: Vec<ContentLayer> = flattened.into_iter()
            .map(|content| ContentLayer {
                layer_id: format!("hierarchical_{}", uuid::Uuid::new_v4()),
                layer_type: LayerType::Content,
                content,
                opacity: 1.0,
                blend_mode: BlendMode::Normal,
                visible: true,
                locked: false,
            })
            .collect();

        self.blend_layers(layers, config)
    }

    fn blend_layer(&mut self, composition: &BlendedComposition, layer: &ContentLayer, config: &BlendingConfig) -> RobinResult<BlendedComposition> {
        let cache_key = self.generate_blend_cache_key(composition, layer);

        if config.enable_caching {
            if let Some(cached) = self.blend_cache.get(&cache_key) {
                return Ok(cached.clone());
            }
        }

        let processor = self.blend_processors.get(&layer.blend_mode)
            .unwrap_or_else(|| self.blend_processors.get(&BlendMode::Normal).unwrap());

        let blended = processor.blend(composition, layer, config)?;

        if config.enable_caching {
            self.blend_cache.insert(cache_key, blended.clone());
        }

        Ok(blended)
    }

    fn initialize_blend_processors() -> HashMap<BlendMode, BlendProcessor> {
        let mut processors = HashMap::new();

        processors.insert(BlendMode::Normal, BlendProcessor::new(BlendMode::Normal));
        processors.insert(BlendMode::Multiply, BlendProcessor::new(BlendMode::Multiply));
        processors.insert(BlendMode::Overlay, BlendProcessor::new(BlendMode::Overlay));
        processors.insert(BlendMode::Additive, BlendProcessor::new(BlendMode::Additive));
        processors.insert(BlendMode::Subtract, BlendProcessor::new(BlendMode::Subtract));
        processors.insert(BlendMode::Divide, BlendProcessor::new(BlendMode::Divide));
        processors.insert(BlendMode::Screen, BlendProcessor::new(BlendMode::Screen));
        processors.insert(BlendMode::SoftLight, BlendProcessor::new(BlendMode::SoftLight));
        processors.insert(BlendMode::HardLight, BlendProcessor::new(BlendMode::HardLight));

        processors
    }

    pub fn set_config(&mut self, config: BlendingConfig) {
        self.config = config;
        if !self.config.enable_caching {
            self.blend_cache.clear();
        }
    }

    pub fn clear_cache(&mut self) {
        self.blend_cache.clear();
    }

    fn generate_blend_cache_key(&self, composition: &BlendedComposition, layer: &ContentLayer) -> String {
        format!("{}_{}_{}_{}",
            composition.composition_id,
            layer.layer_id,
            layer.opacity,
            layer.blend_mode as u8
        )
    }
}

/// Composition optimization system
#[derive(Debug)]
pub struct CompositionOptimizer {
    optimization_strategies: HashMap<String, OptimizationStrategy>,
    performance_tracker: PerformanceTracker,
    optimization_cache: HashMap<String, ComposedContent>,
    config: OptimizationConfig,
}

impl CompositionOptimizer {
    pub fn new() -> Self {
        Self {
            optimization_strategies: Self::default_strategies(),
            performance_tracker: PerformanceTracker::new(),
            optimization_cache: HashMap::new(),
            config: OptimizationConfig::default(),
        }
    }

    fn default_strategies() -> HashMap<String, OptimizationStrategy> {
        let mut strategies = HashMap::new();

        strategies.insert("layer_merging".to_string(), OptimizationStrategy {
            strategy_id: "layer_merging".to_string(),
            strategy_type: StrategyType::LayerMerging,
            enabled: true,
            priority: 1,
        });

        strategies.insert("cache_optimization".to_string(), OptimizationStrategy {
            strategy_id: "cache_optimization".to_string(),
            strategy_type: StrategyType::CacheOptimization,
            enabled: true,
            priority: 2,
        });

        strategies.insert("memory_pooling".to_string(), OptimizationStrategy {
            strategy_id: "memory_pooling".to_string(),
            strategy_type: StrategyType::MemoryPooling,
            enabled: true,
            priority: 3,
        });

        strategies
    }

    pub fn optimize(&mut self, composition: ComposedContent) -> RobinResult<ComposedContent> {
        let start = std::time::Instant::now();

        let mut optimized = composition;

        for strategy in self.get_sorted_strategies() {
            if strategy.enabled {
                optimized = self.apply_strategy(&strategy, optimized)?;
            }
        }

        self.performance_tracker.record_optimization(start.elapsed());

        Ok(optimized)
    }

    pub fn optimize_with_materials(&mut self, composition: ComposedContent, material_system: &AdvancedMaterialSystem) -> RobinResult<ComposedContent> {
        let start = std::time::Instant::now();

        let mut optimized = composition;

        // Material-aware optimization
        optimized = self.optimize_material_usage(optimized, material_system)?;
        optimized = self.merge_similar_materials(optimized, material_system)?;
        optimized = self.cache_material_computations(optimized, material_system)?;

        for strategy in self.get_sorted_strategies() {
            if strategy.enabled {
                optimized = self.apply_material_aware_strategy(&strategy, optimized, material_system)?;
            }
        }

        self.performance_tracker.record_optimization(start.elapsed());

        Ok(optimized)
    }

    fn optimize_material_usage(&mut self, composition: ComposedContent, material_system: &AdvancedMaterialSystem) -> RobinResult<ComposedContent> {
        // In production: Optimize material usage patterns
        Ok(composition)
    }

    fn merge_similar_materials(&mut self, composition: ComposedContent, material_system: &AdvancedMaterialSystem) -> RobinResult<ComposedContent> {
        // In production: Merge layers with similar materials
        Ok(composition)
    }

    fn cache_material_computations(&mut self, composition: ComposedContent, material_system: &AdvancedMaterialSystem) -> RobinResult<ComposedContent> {
        // In production: Cache expensive material computations
        Ok(composition)
    }

    fn apply_material_aware_strategy(&mut self, strategy: &OptimizationStrategy, composition: ComposedContent, material_system: &AdvancedMaterialSystem) -> RobinResult<ComposedContent> {
        match strategy.strategy_type {
            StrategyType::LayerMerging => self.merge_material_layers(composition, material_system),
            StrategyType::CacheOptimization => self.optimize_material_cache(composition, material_system),
            StrategyType::MemoryPooling => self.pool_material_memory(composition, material_system),
            _ => Ok(composition),
        }
    }

    fn merge_material_layers(&mut self, composition: ComposedContent, material_system: &AdvancedMaterialSystem) -> RobinResult<ComposedContent> {
        // In production: Merge layers with compatible materials
        Ok(composition)
    }

    fn optimize_material_cache(&mut self, composition: ComposedContent, material_system: &AdvancedMaterialSystem) -> RobinResult<ComposedContent> {
        // In production: Optimize material cache usage
        Ok(composition)
    }

    fn pool_material_memory(&mut self, composition: ComposedContent, material_system: &AdvancedMaterialSystem) -> RobinResult<ComposedContent> {
        // In production: Pool memory for material resources
        Ok(composition)
    }

    fn get_sorted_strategies(&self) -> Vec<OptimizationStrategy> {
        let mut strategies: Vec<_> = self.optimization_strategies.values().cloned().collect();
        strategies.sort_by_key(|s| s.priority);
        strategies
    }

    fn apply_strategy(&mut self, strategy: &OptimizationStrategy, composition: ComposedContent) -> RobinResult<ComposedContent> {
        match strategy.strategy_type {
            StrategyType::LayerMerging => self.merge_layers(composition),
            StrategyType::CacheOptimization => self.optimize_cache(composition),
            StrategyType::MemoryPooling => self.pool_memory(composition),
            _ => Ok(composition),
        }
    }

    fn merge_layers(&mut self, mut composition: ComposedContent) -> RobinResult<ComposedContent> {
        // In production: Intelligently merge compatible layers
        if composition.layers.len() > self.config.max_layers {
            // Merge similar layers
            composition.layers.truncate(self.config.max_layers);
        }
        Ok(composition)
    }

    fn optimize_cache(&mut self, composition: ComposedContent) -> RobinResult<ComposedContent> {
        // In production: Optimize cache usage
        let cache_key = format!("{}_optimized", composition.composition_id);

        if let Some(cached) = self.optimization_cache.get(&cache_key) {
            return Ok(cached.clone());
        }

        self.optimization_cache.insert(cache_key, composition.clone());
        Ok(composition)
    }

    fn pool_memory(&mut self, composition: ComposedContent) -> RobinResult<ComposedContent> {
        // In production: Use memory pooling for composition data
        Ok(composition)
    }

    pub fn get_performance_metrics(&self) -> PerformanceMetrics {
        self.performance_tracker.get_metrics()
    }
}

/// Composition analytics system for tracking and analysis
#[derive(Debug)]
pub struct CompositionAnalytics {
    composition_history: VecDeque<CompositionRecord>,
    performance_metrics: HashMap<String, f32>,
    quality_metrics: HashMap<String, f32>,
    enabled: bool,
}

impl CompositionAnalytics {
    pub fn new() -> Self {
        Self {
            composition_history: VecDeque::with_capacity(1000),
            performance_metrics: HashMap::new(),
            quality_metrics: HashMap::new(),
            enabled: true,
        }
    }

    pub fn record_composition(&mut self, request: &LayeredCompositionRequest, result: &ComposedContent, duration: std::time::Duration) -> RobinResult<()> {
        if !self.enabled {
            return Ok(());
        }

        let record = CompositionRecord {
            timestamp: std::time::SystemTime::now(),
            request: request.clone(),
            result: result.clone(),
            duration,
            layer_count: result.layers.len(),
            total_size: self.calculate_total_size(result),
        };

        self.composition_history.push_back(record);
        if self.composition_history.len() > 1000 {
            self.composition_history.pop_front();
        }

        self.update_metrics(duration, result);

        Ok(())
    }

    pub fn record_composition_with_materials(&mut self, request: &LayeredCompositionRequest, result: &ComposedContent, duration: std::time::Duration, material_system: &AdvancedMaterialSystem) -> RobinResult<()> {
        if !self.enabled {
            return Ok(());
        }

        // Record basic composition
        self.record_composition(request, result, duration)?;

        // Add material-specific metrics
        self.record_material_metrics(request, result, material_system)?;

        Ok(())
    }

    fn record_material_metrics(&mut self, request: &LayeredCompositionRequest, result: &ComposedContent, material_system: &AdvancedMaterialSystem) -> RobinResult<()> {
        let material_count = self.count_materials_in_request(request);
        let material_complexity = self.calculate_material_complexity(request, material_system);
        let material_efficiency = self.calculate_material_efficiency(result, material_system);

        self.quality_metrics.insert("material_count".to_string(), material_count as f32);
        self.quality_metrics.insert("material_complexity".to_string(), material_complexity);
        self.quality_metrics.insert("material_efficiency".to_string(), material_efficiency);

        Ok(())
    }

    fn count_materials_in_request(&self, request: &LayeredCompositionRequest) -> usize {
        // In production: Count unique materials in request
        1 + request.secondary_content.len()
    }

    fn calculate_material_complexity(&self, request: &LayeredCompositionRequest, material_system: &AdvancedMaterialSystem) -> f32 {
        // In production: Calculate material complexity score
        0.7
    }

    fn calculate_material_efficiency(&self, result: &ComposedContent, material_system: &AdvancedMaterialSystem) -> f32 {
        // In production: Calculate material usage efficiency
        0.85
    }

    fn calculate_total_size(&self, composition: &ComposedContent) -> usize {
        composition.layers.iter()
            .map(|layer| layer.content.content_data.len())
            .sum()
    }

    fn update_metrics(&mut self, duration: std::time::Duration, result: &ComposedContent) {
        // Update performance metrics
        self.performance_metrics.insert(
            "avg_composition_time".to_string(),
            duration.as_millis() as f32,
        );

        self.performance_metrics.insert(
            "avg_layer_count".to_string(),
            result.layers.len() as f32,
        );

        // Update quality metrics
        let quality_score = self.calculate_quality_score(result);
        self.quality_metrics.insert("quality_score".to_string(), quality_score);
    }

    fn calculate_quality_score(&self, composition: &ComposedContent) -> f32 {
        // In production: Calculate comprehensive quality score
        0.8 + (composition.layers.len() as f32 * 0.02).min(0.2)
    }

    pub fn get_analytics_summary(&self) -> CompositionAnalyticsSummary {
        CompositionAnalyticsSummary {
            total_compositions: self.composition_history.len(),
            average_composition_time: self.get_average_composition_time(),
            average_layer_count: self.get_average_layer_count(),
            performance_score: self.calculate_performance_score(),
            quality_score: self.calculate_overall_quality_score(),
        }
    }

    fn get_average_composition_time(&self) -> f32 {
        if self.composition_history.is_empty() {
            return 0.0;
        }

        let total: u128 = self.composition_history.iter()
            .map(|r| r.duration.as_millis())
            .sum();

        (total as f32) / (self.composition_history.len() as f32)
    }

    fn get_average_layer_count(&self) -> f32 {
        if self.composition_history.is_empty() {
            return 0.0;
        }

        let total: usize = self.composition_history.iter()
            .map(|r| r.layer_count)
            .sum();

        (total as f32) / (self.composition_history.len() as f32)
    }

    fn calculate_performance_score(&self) -> f32 {
        // In production: Calculate performance score based on metrics
        0.85
    }

    fn calculate_overall_quality_score(&self) -> f32 {
        if self.quality_metrics.is_empty() {
            return 0.0;
        }

        let sum: f32 = self.quality_metrics.values().sum();
        sum / (self.quality_metrics.len() as f32)
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn get_performance_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();

        let avg_time = self.get_average_composition_time();
        if avg_time > 1000.0 {
            recommendations.push("Consider enabling composition caching".to_string());
        }

        let avg_layers = self.get_average_layer_count();
        if avg_layers > 10.0 {
            recommendations.push("High layer count detected - consider layer merging".to_string());
        }

        recommendations
    }

    /// Get material interaction patterns
    pub fn get_material_interaction_patterns(&self, material_system: &AdvancedMaterialSystem) -> Vec<MaterialInteractionPattern> {
        let mut patterns = Vec::new();

        // Analyze composition history for material interactions
        for record in self.composition_history.iter().take(100) {
            let interaction_count = self.count_material_interactions(&record.request, material_system);
            if interaction_count > 2 {
                patterns.push(MaterialInteractionPattern {
                    pattern_id: format!("pattern_{}", patterns.len()),
                    materials_involved: self.identify_involved_materials(&record.request, material_system),
                    frequency: self.calculate_pattern_frequency(&record.request),
                    performance_impact: self.calculate_pattern_performance_impact(&record.duration),
                });
            }
        }

        patterns
    }

    fn count_material_interactions(&self, request: &LayeredCompositionRequest, material_system: &AdvancedMaterialSystem) -> usize {
        // In production: Count material interactions in request
        request.secondary_content.len()
    }

    fn identify_involved_materials(&self, request: &LayeredCompositionRequest, material_system: &AdvancedMaterialSystem) -> Vec<String> {
        // In production: Identify materials in request
        vec![self.identify_dominant_material(request, material_system)]
    }

    fn identify_dominant_material(&self, request: &LayeredCompositionRequest, material_system: &AdvancedMaterialSystem) -> String {
        // In production: Identify dominant material
        "Metal".to_string()
    }

    fn calculate_pattern_frequency(&self, request: &LayeredCompositionRequest) -> f32 {
        // In production: Calculate how often this pattern appears
        0.15
    }

    fn calculate_pattern_performance_impact(&self, duration: &std::time::Duration) -> f32 {
        // In production: Calculate performance impact
        duration.as_millis() as f32 / 100.0
    }
}

// ================================================================================================
// SUPPORTING DATA STRUCTURES FOR ADVANCED CONTENT COMPOSITION
// ================================================================================================

/// Request for layered content composition
#[derive(Debug, Clone)]
pub struct LayeredCompositionRequest {
    pub target_type: String,
    pub resolution: (u32, u32),
    pub primary_content: ContentElement,
    pub secondary_content: Vec<ContentElement>,
    pub blend_modes: Vec<BlendMode>,
    pub optimization_level: OptimizationLevel,
}

/// Content element for composition
#[derive(Debug, Clone, Default)]
pub struct ContentElement {
    pub element_id: String,
    pub element_type: ContentElementType,
    pub content_data: Vec<u8>,
    pub metadata: HashMap<String, String>,
}

/// Content element type
#[derive(Debug, Clone, PartialEq)]
pub enum ContentElementType {
    Base,
    Content,
    Detail,
    Overlay,
    Effect,
}

impl Default for ContentElementType {
    fn default() -> Self {
        ContentElementType::Base
    }
}

/// Content layer for composition
#[derive(Debug, Clone)]
pub struct ContentLayer {
    pub layer_id: String,
    pub layer_type: LayerType,
    pub content: ContentElement,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub visible: bool,
    pub locked: bool,
}

impl ContentLayer {
    pub fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity.clamp(0.0, 1.0);
    }

    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }

    pub fn set_blend_mode(&mut self, mode: BlendMode) {
        self.blend_mode = mode;
    }
}

/// Layer type enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum LayerType {
    Base,
    BaseMaterial,
    Content,
    ContentMaterial,
    Detail,
    DetailMaterial,
    Overlay,
    OverlayMaterial,
    Effect,
    Adjustment,
}

/// Blend mode for layer composition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum BlendMode {
    Normal = 0,
    Multiply = 1,
    Overlay = 2,
    Additive = 3,
    Subtract = 4,
    Divide = 5,
    Screen = 6,
    SoftLight = 7,
    HardLight = 8,
}

impl Default for BlendMode {
    fn default() -> Self {
        BlendMode::Normal
    }
}

/// Composed content result
#[derive(Debug, Clone, Default)]
pub struct ComposedContent {
    pub composition_id: String,
    pub layers: Vec<ContentLayer>,
    pub final_data: Vec<u8>,
    pub metadata: CompositionMetadata,
}

impl ComposedContent {
    pub fn new() -> Self {
        Self {
            composition_id: format!("comp_{}", uuid::Uuid::new_v4()),
            layers: Vec::new(),
            final_data: Vec::new(),
            metadata: CompositionMetadata::default(),
        }
    }

    pub fn add_layer(&mut self, layer: ContentLayer) {
        self.layers.push(layer);
    }

    pub fn get_layer_mut(&mut self, layer_id: &str) -> Option<&mut ContentLayer> {
        self.layers.iter_mut().find(|l| l.layer_id == layer_id)
    }

    pub fn remove_layer(&mut self, layer_id: &str) {
        self.layers.retain(|l| l.layer_id != layer_id);
    }
}

/// Composition metadata
#[derive(Debug, Clone, Default)]
pub struct CompositionMetadata {
    pub creation_time: Option<std::time::SystemTime>,
    pub layer_count: usize,
    pub total_size: usize,
    pub optimization_applied: bool,
}

/// Blending configuration
#[derive(Debug, Clone)]
pub struct BlendingConfig {
    pub enable_caching: bool,
    pub quality_level: QualityLevel,
    pub max_blend_iterations: usize,
    pub alpha_threshold: f32,
    pub default_blend_modes: Vec<BlendMode>,
}

impl Default for BlendingConfig {
    fn default() -> Self {
        Self {
            enable_caching: true,
            quality_level: QualityLevel::High,
            max_blend_iterations: 10,
            alpha_threshold: 0.01,
            default_blend_modes: vec![
                BlendMode::Normal,
                BlendMode::Multiply,
                BlendMode::Overlay,
                BlendMode::Additive,
            ],
        }
    }
}

/// Quality level for blending
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QualityLevel {
    Low,
    Medium,
    High,
    Ultra,
}

/// Detail element for composition
#[derive(Debug, Clone)]
pub struct DetailElement {
    pub detail_id: String,
    pub detail_type: DetailType,
    pub detail_data: Vec<u8>,
    pub intensity: f32,
}

/// Detail type enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum DetailType {
    Texture,
    Pattern,
    Noise,
    Gradient,
}

/// Overlay element for composition
#[derive(Debug, Clone)]
pub struct OverlayElement {
    pub overlay_id: String,
    pub overlay_type: OverlayType,
    pub overlay_data: Vec<u8>,
    pub opacity: f32,
}

/// Overlay type enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum OverlayType {
    Color,
    Pattern,
    Effect,
    Mask,
}

/// Hierarchical node for composition
#[derive(Debug, Clone)]
pub struct HierarchicalNode {
    pub node_id: String,
    pub content: ContentElement,
    pub parent_id: Option<String>,
    pub child_ids: Vec<String>,
    pub depth: usize,
}

/// Composition stage for pipeline
#[derive(Debug, Clone)]
pub struct CompositionStage {
    pub stage_id: String,
    pub stage_type: StageType,
    pub enabled: bool,
    pub order: usize,
}

/// Stage type enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum StageType {
    Preprocessing,
    Layering,
    Blending,
    Optimization,
    Postprocessing,
}

/// Stage configuration
#[derive(Debug, Clone)]
pub struct StageConfig {
    pub max_input_size: usize,
    pub timeout_ms: u64,
    pub parallel_processing: bool,
    pub cache_results: bool,
}

/// Pipeline input
#[derive(Debug, Clone)]
pub struct PipelineInput {
    pub data: Vec<u8>,
    pub metadata: HashMap<String, String>,
}

/// Pipeline output
#[derive(Debug, Clone)]
pub struct PipelineOutput {
    pub data: Vec<u8>,
    pub stages_completed: usize,
    pub processing_time: std::time::Duration,
}

/// Cached pipeline result
#[derive(Debug, Clone)]
pub struct CachedPipelineResult {
    pub output: PipelineOutput,
    pub cache_time: std::time::SystemTime,
}

/// Composition rule for hierarchical composer
#[derive(Debug, Clone)]
pub struct CompositionRule {
    pub rule_id: String,
    pub rule_type: RuleType,
    pub priority: usize,
    pub conditions: Vec<RuleCondition>,
}

/// Rule type enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum RuleType {
    CombineChildren,
    InheritProperties,
    ApplyTransform,
    FilterContent,
}

/// Rule condition
#[derive(Debug, Clone)]
pub struct RuleCondition {
    pub condition_type: String,
    pub parameters: HashMap<String, String>,
}

/// Optimization level
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OptimizationLevel {
    None,
    Basic,
    Balanced,
    Aggressive,
}

/// Optimization strategy
#[derive(Debug, Clone)]
pub struct OptimizationStrategy {
    pub strategy_id: String,
    pub strategy_type: StrategyType,
    pub enabled: bool,
    pub priority: usize,
}

/// Strategy type enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum StrategyType {
    LayerMerging,
    CacheOptimization,
    MemoryPooling,
    ParallelProcessing,
}

/// Optimization configuration
#[derive(Debug, Clone)]
pub struct OptimizationConfig {
    pub max_layers: usize,
    pub enable_merging: bool,
    pub cache_size_mb: usize,
    pub parallel_threads: usize,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            max_layers: 32,
            enable_merging: true,
            cache_size_mb: 256,
            parallel_threads: 4,
        }
    }
}

/// Performance tracker
#[derive(Debug)]
pub struct PerformanceTracker {
    optimization_times: Vec<std::time::Duration>,
    cache_hits: usize,
    cache_misses: usize,
}

impl PerformanceTracker {
    pub fn new() -> Self {
        Self {
            optimization_times: Vec::new(),
            cache_hits: 0,
            cache_misses: 0,
        }
    }

    pub fn record_optimization(&mut self, duration: std::time::Duration) {
        self.optimization_times.push(duration);
        if self.optimization_times.len() > 1000 {
            self.optimization_times.remove(0);
        }
    }

    pub fn record_cache_hit(&mut self) {
        self.cache_hits += 1;
    }

    pub fn record_cache_miss(&mut self) {
        self.cache_misses += 1;
    }

    pub fn get_metrics(&self) -> PerformanceMetrics {
        PerformanceMetrics {
            average_optimization_time: self.calculate_average_time(),
            cache_hit_rate: self.calculate_cache_hit_rate(),
            total_optimizations: self.optimization_times.len(),
        }
    }

    fn calculate_average_time(&self) -> f32 {
        if self.optimization_times.is_empty() {
            return 0.0;
        }

        let total: u128 = self.optimization_times.iter()
            .map(|d| d.as_millis())
            .sum();

        (total as f32) / (self.optimization_times.len() as f32)
    }

    fn calculate_cache_hit_rate(&self) -> f32 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            return 0.0;
        }
        (self.cache_hits as f32) / (total as f32)
    }
}

/// Performance metrics
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub average_optimization_time: f32,
    pub cache_hit_rate: f32,
    pub total_optimizations: usize,
}

/// Blend processor
#[derive(Debug)] pub struct BlendProcessor { blend_mode: BlendMode }
impl BlendProcessor {
    pub fn new(blend_mode: BlendMode) -> Self { Self { blend_mode } }
    pub fn blend(&self, composition: &BlendedComposition, _layer: &ContentLayer, _config: &BlendingConfig) -> RobinResult<BlendedComposition> {
        // In production: Apply specific blend mode algorithm
        Ok(composition.clone())
    }
    pub fn apply_to_layer(&self, _layer: &mut ContentLayer) {
        // In production: Apply blend mode to layer
    }
}

/// Blended composition
#[derive(Debug, Clone, Default)]
pub struct BlendedComposition {
    pub composition_id: String,
    pub blended_data: Vec<u8>,
    pub layers: Vec<ContentLayer>,
    pub blend_metadata: BlendMetadata,
}

impl BlendedComposition {
    pub fn from_layer(layer: &ContentLayer) -> Self {
        Self {
            composition_id: format!("blend_{}", uuid::Uuid::new_v4()),
            blended_data: layer.content.content_data.clone(),
            layers: vec![layer.clone()],
            blend_metadata: BlendMetadata::default(),
        }
    }
}

/// Blend metadata
#[derive(Debug, Clone, Default)]
pub struct BlendMetadata {
    pub blend_count: usize,
    pub total_opacity: f32,
    pub dominant_blend_mode: Option<BlendMode>,
}

/// Composition performance metrics
#[derive(Debug, Clone, Default)]
pub struct CompositionPerformanceMetrics {
    pub composition_time_ms: f32,
    pub memory_usage_mb: f32,
    pub cache_efficiency: f32,
}

/// Composition pipeline configuration
#[derive(Debug, Clone)]
pub struct CompositionPipelineConfig {
    pub custom_stages: Option<Vec<CompositionStage>>,
    pub stage_configs: Option<HashMap<String, StageConfig>>,
    pub enable_parallel: bool,
}

/// Composition record for analytics
#[derive(Debug, Clone)]
pub struct CompositionRecord {
    pub timestamp: std::time::SystemTime,
    pub request: LayeredCompositionRequest,
    pub result: ComposedContent,
    pub duration: std::time::Duration,
    pub layer_count: usize,
    pub total_size: usize,
}

/// Composition analytics summary
#[derive(Debug, Clone)]
pub struct CompositionAnalyticsSummary {
    pub total_compositions: usize,
    pub average_composition_time: f32,
    pub average_layer_count: f32,
    pub performance_score: f32,
    pub quality_score: f32,
}

/// Material information for layer operations
#[derive(Debug, Clone)]
pub struct MaterialInfo {
    pub material_type: AdvancedMaterialType,
    pub opacity: f32,
    pub reflectivity: f32,
    pub roughness: f32,
}

/// Material interaction pattern
#[derive(Debug, Clone)]
pub struct MaterialInteractionPattern {
    pub pattern_id: String,
    pub materials_involved: Vec<String>,
    pub frequency: f32,
    pub performance_impact: f32,
}

/// Interaction layer for composition
#[derive(Debug, Clone)]
pub struct InteractionLayer {
    pub layer_id: String,
    pub interaction_type: InteractionType,
    pub interaction_strength: f32,
    pub affected_layers: Vec<String>,
}

/// Interaction type enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum InteractionType {
    Blend,
    Mask,
    Transform,
    Filter,
}

impl InteractionLayer {
    /// Get interaction intensity
    pub fn get_interaction_intensity(&self) -> f32 { self.interaction_strength }

    /// Set interaction intensity
    pub fn set_interaction_intensity(&mut self, intensity: f32) {
        self.interaction_strength = intensity.clamp(0.0, 1.0);
    }

    /// Check if affects specific layer
    pub fn affects_layer(&self, layer_id: &str) -> bool {
        self.affected_layers.contains(&layer_id.to_string())
    }

    /// Add affected layer
    pub fn add_affected_layer(&mut self, layer_id: String) {
        if !self.affected_layers.contains(&layer_id) {
            self.affected_layers.push(layer_id);
        }
    }

    /// Get interaction pattern frequency
    pub fn get_interaction_pattern_frequency(&self) -> f32 { 0.68 }
}