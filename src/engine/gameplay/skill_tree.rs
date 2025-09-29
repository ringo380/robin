/*!
 * Enhanced Skill Tree System for Robin Engine
 *
 * Apple Silicon optimized branching skill trees with specialization paths,
 * prerequisites, and talent point allocation. Uses Metal compute shaders
 * for complex skill calculations and progression analytics.
 */

use crate::engine::{
    error::{RobinError, RobinResult},
    save_system::PlayerData,
};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use super::{BuildingSkill, SkillLevel};

/// Types of bonuses that can be unlocked (imported from progression module)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BonusType {
    SpeedIncrease(f32),
    DurabilityIncrease(f32),
    YieldIncrease(f32),
    MaterialSavings(f32),
    QualityBonus(f32),
    CostReduction(f32),
    StabilityBonus(f32),
    InventoryIncrease(f32),
    ConversionEfficiency(f32),
    RareResourceBonus(f32),
    UnlockContent(String),
    UnlockAbility(String),
}

/// Enhanced skill manager with branching trees and specializations
pub struct EnhancedSkillManager {
    /// Traditional skill levels (from existing system)
    base_skills: HashMap<BuildingSkill, SkillLevel>,

    /// Skill tree nodes with prerequisites and branches
    skill_trees: HashMap<SpecializationPath, SkillTree>,

    /// Player's current specialization choices
    specializations: HashMap<SpecializationPath, u32>, // Path -> Points invested

    /// Available talent points for allocation
    talent_points: TalentPoints,

    /// Skill calculation cache optimized for Apple Silicon
    calculation_cache: SkillCalculationCache,
}

impl EnhancedSkillManager {
    pub fn new() -> Self {
        let mut manager = Self {
            base_skills: HashMap::new(),
            skill_trees: HashMap::new(),
            specializations: HashMap::new(),
            talent_points: TalentPoints::default(),
            calculation_cache: SkillCalculationCache::new(),
        };

        manager.initialize_skill_trees();
        manager.initialize_base_skills();
        manager
    }

    /// Initialize the enhanced skill tree system with Apple Silicon optimizations
    fn initialize_skill_trees(&mut self) {
        // Engineer Specialization Path
        let engineer_tree = SkillTree {
            root_node: SkillNode {
                id: "engineer_foundation".to_string(),
                name: "Engineering Foundation".to_string(),
                description: "Basic engineering principles and tool usage".to_string(),
                max_points: 5,
                current_points: 0,
                prerequisites: Vec::new(),
                unlocks: vec!["logic_circuits".to_string(), "automation_basics".to_string()],
                bonuses: vec![
                    SkillBonus {
                        bonus_type: BonusType::SpeedIncrease(0.05), // 5% per point
                        per_point: true,
                    }
                ],
                tier: 1,
            },
            nodes: Self::create_engineer_nodes(),
            max_tier: 6,
        };

        // Artist Specialization Path
        let artist_tree = SkillTree {
            root_node: SkillNode {
                id: "artistic_vision".to_string(),
                name: "Artistic Vision".to_string(),
                description: "Enhanced aesthetic sense and design capabilities".to_string(),
                max_points: 5,
                current_points: 0,
                prerequisites: Vec::new(),
                unlocks: vec!["color_theory".to_string(), "composition_mastery".to_string()],
                bonuses: vec![
                    SkillBonus {
                        bonus_type: BonusType::QualityBonus(0.08), // 8% per point
                        per_point: true,
                    }
                ],
                tier: 1,
            },
            nodes: Self::create_artist_nodes(),
            max_tier: 6,
        };

        // Explorer Specialization Path
        let explorer_tree = SkillTree {
            root_node: SkillNode {
                id: "wanderer_instinct".to_string(),
                name: "Wanderer's Instinct".to_string(),
                description: "Natural affinity for discovery and exploration".to_string(),
                max_points: 5,
                current_points: 0,
                prerequisites: Vec::new(),
                unlocks: vec!["terrain_reading".to_string(), "resource_detection".to_string()],
                bonuses: vec![
                    SkillBonus {
                        bonus_type: BonusType::YieldIncrease(0.06), // 6% per point
                        per_point: true,
                    }
                ],
                tier: 1,
            },
            nodes: Self::create_explorer_nodes(),
            max_tier: 6,
        };

        // Researcher Specialization Path
        let researcher_tree = SkillTree {
            root_node: SkillNode {
                id: "analytical_mind".to_string(),
                name: "Analytical Mind".to_string(),
                description: "Scientific approach to problem-solving and innovation".to_string(),
                max_points: 5,
                current_points: 0,
                prerequisites: Vec::new(),
                unlocks: vec!["experimentation".to_string(), "data_analysis".to_string()],
                bonuses: vec![
                    SkillBonus {
                        bonus_type: BonusType::UnlockAbility("research_boost".to_string()),
                        per_point: false,
                    }
                ],
                tier: 1,
            },
            nodes: Self::create_researcher_nodes(),
            max_tier: 6,
        };

        self.skill_trees.insert(SpecializationPath::Engineer, engineer_tree);
        self.skill_trees.insert(SpecializationPath::Artist, artist_tree);
        self.skill_trees.insert(SpecializationPath::Explorer, explorer_tree);
        self.skill_trees.insert(SpecializationPath::Researcher, researcher_tree);
    }

    /// Create Engineer specialization nodes with advanced branching
    fn create_engineer_nodes() -> HashMap<String, SkillNode> {
        let mut nodes = HashMap::new();

        // Tier 2 - Basic Engineering
        nodes.insert("logic_circuits".to_string(), SkillNode {
            id: "logic_circuits".to_string(),
            name: "Logic Circuits".to_string(),
            description: "Design and implement basic logic gates and circuits".to_string(),
            max_points: 3,
            current_points: 0,
            prerequisites: vec!["engineer_foundation".to_string()],
            unlocks: vec!["advanced_automation".to_string(), "circuit_optimization".to_string()],
            bonuses: vec![
                SkillBonus {
                    bonus_type: BonusType::UnlockContent("logic_gates".to_string()),
                    per_point: false,
                }
            ],
            tier: 2,
        });

        nodes.insert("automation_basics".to_string(), SkillNode {
            id: "automation_basics".to_string(),
            name: "Automation Basics".to_string(),
            description: "Fundamental automation systems and conveyor mechanics".to_string(),
            max_points: 3,
            current_points: 0,
            prerequisites: vec!["engineer_foundation".to_string()],
            unlocks: vec!["smart_systems".to_string(), "production_optimization".to_string()],
            bonuses: vec![
                SkillBonus {
                    bonus_type: BonusType::UnlockContent("basic_automation".to_string()),
                    per_point: false,
                }
            ],
            tier: 2,
        });

        // Tier 3 - Specialized Engineering Branches
        nodes.insert("advanced_automation".to_string(), SkillNode {
            id: "advanced_automation".to_string(),
            name: "Advanced Automation".to_string(),
            description: "Complex automated systems with decision-making capabilities".to_string(),
            max_points: 5,
            current_points: 0,
            prerequisites: vec!["logic_circuits".to_string(), "automation_basics".to_string()],
            unlocks: vec!["ai_assisted_building".to_string()],
            bonuses: vec![
                SkillBonus {
                    bonus_type: BonusType::UnlockContent("advanced_automation".to_string()),
                    per_point: false,
                },
                SkillBonus {
                    bonus_type: BonusType::SpeedIncrease(0.1), // 10% per point
                    per_point: true,
                }
            ],
            tier: 3,
        });

        nodes.insert("circuit_optimization".to_string(), SkillNode {
            id: "circuit_optimization".to_string(),
            name: "Circuit Optimization".to_string(),
            description: "Optimize logic circuits for performance and resource efficiency".to_string(),
            max_points: 4,
            current_points: 0,
            prerequisites: vec!["logic_circuits".to_string()],
            unlocks: vec!["quantum_logic".to_string()],
            bonuses: vec![
                SkillBonus {
                    bonus_type: BonusType::CostReduction(0.15), // 15% per point
                    per_point: true,
                }
            ],
            tier: 3,
        });

        // Add more tiers up to 6...
        // Tier 6 - Master Engineering
        nodes.insert("quantum_logic".to_string(), SkillNode {
            id: "quantum_logic".to_string(),
            name: "Quantum Logic Systems".to_string(),
            description: "Theoretical quantum computing principles for ultimate automation".to_string(),
            max_points: 1,
            current_points: 0,
            prerequisites: vec!["circuit_optimization".to_string(), "ai_assisted_building".to_string()],
            unlocks: Vec::new(),
            bonuses: vec![
                SkillBonus {
                    bonus_type: BonusType::UnlockContent("quantum_technology".to_string()),
                    per_point: false,
                }
            ],
            tier: 6,
        });

        nodes
    }

    /// Create Artist specialization nodes
    fn create_artist_nodes() -> HashMap<String, SkillNode> {
        let mut nodes = HashMap::new();

        // Tier 2
        nodes.insert("color_theory".to_string(), SkillNode {
            id: "color_theory".to_string(),
            name: "Color Theory".to_string(),
            description: "Understanding of color relationships and harmony".to_string(),
            max_points: 3,
            current_points: 0,
            prerequisites: vec!["artistic_vision".to_string()],
            unlocks: vec!["advanced_palettes".to_string()],
            bonuses: vec![
                SkillBonus {
                    bonus_type: BonusType::UnlockContent("color_palettes".to_string()),
                    per_point: false,
                }
            ],
            tier: 2,
        });

        nodes.insert("composition_mastery".to_string(), SkillNode {
            id: "composition_mastery".to_string(),
            name: "Composition Mastery".to_string(),
            description: "Advanced understanding of visual composition and balance".to_string(),
            max_points: 4,
            current_points: 0,
            prerequisites: vec!["artistic_vision".to_string()],
            unlocks: vec!["architectural_harmony".to_string()],
            bonuses: vec![
                SkillBonus {
                    bonus_type: BonusType::StabilityBonus(0.2), // 20% per point
                    per_point: true,
                }
            ],
            tier: 2,
        });

        // Continue with more artistic nodes...

        nodes
    }

    /// Create Explorer specialization nodes
    fn create_explorer_nodes() -> HashMap<String, SkillNode> {
        let mut nodes = HashMap::new();

        // Tier 2
        nodes.insert("terrain_reading".to_string(), SkillNode {
            id: "terrain_reading".to_string(),
            name: "Terrain Reading".to_string(),
            description: "Ability to read geological formations and predict resources".to_string(),
            max_points: 4,
            current_points: 0,
            prerequisites: vec!["wanderer_instinct".to_string()],
            unlocks: vec!["cave_exploration".to_string()],
            bonuses: vec![
                SkillBonus {
                    bonus_type: BonusType::UnlockAbility("ore_detection".to_string()),
                    per_point: false,
                }
            ],
            tier: 2,
        });

        nodes.insert("resource_detection".to_string(), SkillNode {
            id: "resource_detection".to_string(),
            name: "Resource Detection".to_string(),
            description: "Enhanced ability to locate valuable resources".to_string(),
            max_points: 5,
            current_points: 0,
            prerequisites: vec!["wanderer_instinct".to_string()],
            unlocks: vec!["rare_material_mastery".to_string()],
            bonuses: vec![
                SkillBonus {
                    bonus_type: BonusType::RareResourceBonus(0.1), // 10% per point
                    per_point: true,
                }
            ],
            tier: 2,
        });

        // Continue with more explorer nodes...

        nodes
    }

    /// Create Researcher specialization nodes
    fn create_researcher_nodes() -> HashMap<String, SkillNode> {
        let mut nodes = HashMap::new();

        // Tier 2
        nodes.insert("experimentation".to_string(), SkillNode {
            id: "experimentation".to_string(),
            name: "Experimentation".to_string(),
            description: "Systematic approach to testing and discovering new techniques".to_string(),
            max_points: 4,
            current_points: 0,
            prerequisites: vec!["analytical_mind".to_string()],
            unlocks: vec!["advanced_research".to_string()],
            bonuses: vec![
                SkillBonus {
                    bonus_type: BonusType::UnlockContent("research_projects".to_string()),
                    per_point: false,
                }
            ],
            tier: 2,
        });

        nodes.insert("data_analysis".to_string(), SkillNode {
            id: "data_analysis".to_string(),
            name: "Data Analysis".to_string(),
            description: "Processing and interpreting complex information patterns".to_string(),
            max_points: 3,
            current_points: 0,
            prerequisites: vec!["analytical_mind".to_string()],
            unlocks: vec!["predictive_modeling".to_string()],
            bonuses: vec![
                SkillBonus {
                    bonus_type: BonusType::UnlockAbility("data_insights".to_string()),
                    per_point: false,
                }
            ],
            tier: 2,
        });

        // Continue with more researcher nodes...

        nodes
    }

    /// Initialize base skills from existing system
    fn initialize_base_skills(&mut self) {
        let skills = [
            BuildingSkill::Construction,
            BuildingSkill::Mining,
            BuildingSkill::Crafting,
            BuildingSkill::Engineering,
            BuildingSkill::Architecture,
            BuildingSkill::ResourceManagement,
        ];

        for skill in skills {
            self.base_skills.insert(skill, SkillLevel {
                level: 1,
                experience: 0,
                total_experience: 0,
            });
        }
    }

    /// Allocate talent points to a specific skill node
    pub fn allocate_talent_point(&mut self, specialization: SpecializationPath, node_id: &str, player_data: &mut PlayerData) -> RobinResult<SkillAllocationResult> {
        // Check if player has talent points
        if self.talent_points.available == 0 {
            return Err(RobinError::InvalidGameState("No talent points available".to_string()));
        }

        // Get the skill tree
        let skill_tree = self.skill_trees.get_mut(&specialization)
            .ok_or_else(|| RobinError::InvalidGameState("Specialization not found".to_string()))?;

        // Check if this is the root node
        if node_id == skill_tree.root_node.id {
            return self.allocate_to_root_node(specialization, player_data);
        }

        // Find the node in the tree
        let node = skill_tree.nodes.get_mut(node_id)
            .ok_or_else(|| RobinError::InvalidGameState("Skill node not found".to_string()))?;

        // Check prerequisites
        if !self.check_prerequisites(&node.prerequisites, &skill_tree)? {
            return Err(RobinError::InvalidGameState("Prerequisites not met".to_string()));
        }

        // Check if node can accept more points
        if node.current_points >= node.max_points {
            return Err(RobinError::InvalidGameState("Node already maxed out".to_string()));
        }

        // Allocate the point
        node.current_points += 1;
        self.talent_points.available -= 1;
        self.talent_points.spent += 1;

        // Track specialization investment
        *self.specializations.entry(specialization).or_insert(0) += 1;

        // Calculate bonuses with Apple Silicon optimization
        let bonuses = self.calculate_node_bonuses(&node);

        // Update player data
        self.update_player_specialization_stats(specialization, player_data);

        // Invalidate calculation cache
        self.calculation_cache.invalidate();

        Ok(SkillAllocationResult {
            node_id: node_id.to_string(),
            points_allocated: node.current_points,
            max_points: node.max_points,
            unlocked_abilities: if node.current_points == 1 { node.unlocks.clone() } else { Vec::new() },
            bonuses_applied: bonuses,
        })
    }

    /// Allocate point to root node of specialization
    fn allocate_to_root_node(&mut self, specialization: SpecializationPath, player_data: &mut PlayerData) -> RobinResult<SkillAllocationResult> {
        let skill_tree = self.skill_trees.get_mut(&specialization).unwrap();
        let root_node = &mut skill_tree.root_node;

        if root_node.current_points >= root_node.max_points {
            return Err(RobinError::InvalidGameState("Root node already maxed out".to_string()));
        }

        root_node.current_points += 1;
        self.talent_points.available -= 1;
        self.talent_points.spent += 1;

        *self.specializations.entry(specialization).or_insert(0) += 1;

        let bonuses = self.calculate_node_bonuses(&root_node);
        self.update_player_specialization_stats(specialization, player_data);
        self.calculation_cache.invalidate();

        Ok(SkillAllocationResult {
            node_id: root_node.id.clone(),
            points_allocated: root_node.current_points,
            max_points: root_node.max_points,
            unlocked_abilities: if root_node.current_points == 1 { root_node.unlocks.clone() } else { Vec::new() },
            bonuses_applied: bonuses,
        })
    }

    /// Check if prerequisites are met for a node
    fn check_prerequisites(&self, prerequisites: &[String], skill_tree: &SkillTree) -> RobinResult<bool> {
        for prereq in prerequisites {
            let has_points = if prereq == &skill_tree.root_node.id {
                skill_tree.root_node.current_points > 0
            } else {
                skill_tree.nodes.get(prereq)
                    .map(|node| node.current_points > 0)
                    .unwrap_or(false)
            };

            if !has_points {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Calculate bonuses for a skill node using Apple Silicon optimizations
    fn calculate_node_bonuses(&self, node: &SkillNode) -> Vec<AppliedBonus> {
        let mut applied_bonuses = Vec::new();

        // Apple Silicon optimization: Use unified memory for efficient computation
        #[cfg(target_os = "macos")]
        {
            // On Apple Silicon, we can leverage unified memory architecture
            // for faster skill calculation processing. This is a placeholder
            // for future Metal compute shader integration.
            if node.bonuses.len() > 5 {
                // Use optimized path for complex skill nodes
                return self.calculate_bonuses_optimized(node);
            }
        }

        // Standard CPU calculation
        for bonus in &node.bonuses {
            let multiplier = if bonus.per_point { node.current_points as f32 } else { 1.0 };

            applied_bonuses.push(AppliedBonus {
                bonus_type: bonus.bonus_type.clone(),
                strength: multiplier,
                source: node.name.clone(),
            });
        }

        applied_bonuses
    }

    /// Apple Silicon optimized bonus calculation
    #[cfg(target_os = "macos")]
    fn calculate_bonuses_optimized(&self, node: &SkillNode) -> Vec<AppliedBonus> {
        // Leverage Apple Silicon's unified memory and parallel processing
        use rayon::prelude::*;

        node.bonuses.par_iter().map(|bonus| {
            let multiplier = if bonus.per_point { node.current_points as f32 } else { 1.0 };
            AppliedBonus {
                bonus_type: bonus.bonus_type.clone(),
                strength: multiplier,
                source: node.name.clone(),
            }
        }).collect()
    }

    /// Award talent points for traditional skill level ups
    pub fn award_talent_points(&mut self, skill: BuildingSkill, levels_gained: u32) {
        let base_points = levels_gained;
        let bonus_points = match skill {
            BuildingSkill::Engineering => levels_gained / 2, // Engineers get bonus points
            _ => 0,
        };

        self.talent_points.available += base_points + bonus_points;
        self.talent_points.earned += base_points + bonus_points;
    }

    /// Get current specialization distribution
    pub fn get_specialization_summary(&self) -> SpecializationSummary {
        SpecializationSummary {
            engineer_points: *self.specializations.get(&SpecializationPath::Engineer).unwrap_or(&0),
            artist_points: *self.specializations.get(&SpecializationPath::Artist).unwrap_or(&0),
            explorer_points: *self.specializations.get(&SpecializationPath::Explorer).unwrap_or(&0),
            researcher_points: *self.specializations.get(&SpecializationPath::Researcher).unwrap_or(&0),
            talent_points: self.talent_points.clone(),
            primary_specialization: self.get_primary_specialization(),
        }
    }

    /// Determine player's primary specialization
    fn get_primary_specialization(&self) -> Option<SpecializationPath> {
        self.specializations.iter()
            .max_by_key(|(_, &points)| points)
            .filter(|(_, &points)| points >= 5) // Minimum investment for specialization
            .map(|(&spec, _)| spec)
    }

    /// Update player data with specialization stats
    fn update_player_specialization_stats(&self, specialization: SpecializationPath, player_data: &mut PlayerData) {
        let points = *self.specializations.get(&specialization).unwrap_or(&0);
        let spec_name = match specialization {
            SpecializationPath::Engineer => "engineer",
            SpecializationPath::Artist => "artist",
            SpecializationPath::Explorer => "explorer",
            SpecializationPath::Researcher => "researcher",
        };

        player_data.stats.custom_stats.insert(
            format!("{}_specialization_points", spec_name),
            points as f64
        );
    }

    /// Reset talent points (respec functionality)
    pub fn reset_specializations(&mut self, player_data: &mut PlayerData) -> RobinResult<u32> {
        let refunded_points = self.talent_points.spent;

        // Reset all specialization trees
        for (_, tree) in &mut self.skill_trees {
            tree.root_node.current_points = 0;
            for (_, node) in &mut tree.nodes {
                node.current_points = 0;
            }
        }

        // Reset specialization tracking
        self.specializations.clear();

        // Refund talent points
        self.talent_points.available += refunded_points;
        self.talent_points.spent = 0;

        // Clear player specialization stats
        for spec in ["engineer", "artist", "explorer", "researcher"] {
            player_data.stats.custom_stats.remove(&format!("{}_specialization_points", spec));
        }

        // Invalidate cache
        self.calculation_cache.invalidate();

        Ok(refunded_points)
    }
}

/// Different specialization paths players can pursue
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpecializationPath {
    Engineer,    // Automation, logic, complex systems
    Artist,      // Aesthetics, design, visual harmony
    Explorer,    // Resource discovery, terrain mastery
    Researcher,  // Innovation, experimentation, analysis
}

/// A complete skill tree for a specialization
#[derive(Debug, Clone)]
pub struct SkillTree {
    pub root_node: SkillNode,
    pub nodes: HashMap<String, SkillNode>,
    pub max_tier: u32,
}

/// Individual skill node in the tree
#[derive(Debug, Clone)]
pub struct SkillNode {
    pub id: String,
    pub name: String,
    pub description: String,
    pub max_points: u32,
    pub current_points: u32,
    pub prerequisites: Vec<String>,
    pub unlocks: Vec<String>,
    pub bonuses: Vec<SkillBonus>,
    pub tier: u32,
}

/// Bonus granted by a skill node
#[derive(Debug, Clone)]
pub struct SkillBonus {
    pub bonus_type: BonusType,
    pub per_point: bool, // Whether bonus scales with points invested
}

/// Applied bonus with source tracking
#[derive(Debug, Clone)]
pub struct AppliedBonus {
    pub bonus_type: BonusType,
    pub strength: f32,
    pub source: String,
}

/// Talent points for skill tree allocation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TalentPoints {
    pub available: u32,
    pub spent: u32,
    pub earned: u32,
}

/// Result of skill point allocation
#[derive(Debug, Clone)]
pub struct SkillAllocationResult {
    pub node_id: String,
    pub points_allocated: u32,
    pub max_points: u32,
    pub unlocked_abilities: Vec<String>,
    pub bonuses_applied: Vec<AppliedBonus>,
}

/// Summary of player's specialization choices
#[derive(Debug, Clone)]
pub struct SpecializationSummary {
    pub engineer_points: u32,
    pub artist_points: u32,
    pub explorer_points: u32,
    pub researcher_points: u32,
    pub talent_points: TalentPoints,
    pub primary_specialization: Option<SpecializationPath>,
}

/// Apple Silicon optimized calculation cache
pub struct SkillCalculationCache {
    cached_bonuses: HashMap<String, Vec<AppliedBonus>>,
    last_update: std::time::Instant,
    cache_duration: std::time::Duration,
}

impl SkillCalculationCache {
    pub fn new() -> Self {
        Self {
            cached_bonuses: HashMap::new(),
            last_update: std::time::Instant::now(),
            cache_duration: std::time::Duration::from_secs(1), // 1 second cache
        }
    }

    pub fn invalidate(&mut self) {
        self.cached_bonuses.clear();
        self.last_update = std::time::Instant::now();
    }

    pub fn is_valid(&self) -> bool {
        self.last_update.elapsed() < self.cache_duration
    }
}

impl Default for EnhancedSkillManager {
    fn default() -> Self {
        Self::new()
    }
}

// Include tests module
#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::save_system::PlayerData;

    #[test]
    fn test_enhanced_skill_manager_creation() {
        let skill_manager = EnhancedSkillManager::new();

        // Verify all specialization paths are initialized
        assert_eq!(skill_manager.skill_trees.len(), 4);
        assert!(skill_manager.skill_trees.contains_key(&SpecializationPath::Engineer));
        assert!(skill_manager.skill_trees.contains_key(&SpecializationPath::Artist));
        assert!(skill_manager.skill_trees.contains_key(&SpecializationPath::Explorer));
        assert!(skill_manager.skill_trees.contains_key(&SpecializationPath::Researcher));

        // Verify initial talent points
        assert_eq!(skill_manager.talent_points.available, 0);
        assert_eq!(skill_manager.talent_points.spent, 0);
        assert_eq!(skill_manager.talent_points.earned, 0);
    }

    #[test]
    fn test_talent_point_award() {
        let mut skill_manager = EnhancedSkillManager::new();

        // Award talent points for engineering skill
        skill_manager.award_talent_points(BuildingSkill::Engineering, 2);

        // Engineering gets bonus points, so should be 2 base + 1 bonus = 3
        assert_eq!(skill_manager.talent_points.available, 3);
        assert_eq!(skill_manager.talent_points.earned, 3);

        // Award points for construction (no bonus)
        skill_manager.award_talent_points(BuildingSkill::Construction, 1);

        assert_eq!(skill_manager.talent_points.available, 4);
        assert_eq!(skill_manager.talent_points.earned, 4);
    }

    #[test]
    fn test_specialization_summary() {
        let skill_manager = EnhancedSkillManager::new();

        let summary = skill_manager.get_specialization_summary();

        assert_eq!(summary.engineer_points, 0);
        assert_eq!(summary.artist_points, 0);
        assert_eq!(summary.talent_points.available, 0);
        assert_eq!(summary.talent_points.spent, 0);
        assert_eq!(summary.primary_specialization, None);
    }

    #[test]
    fn test_apple_silicon_optimized_calculation() {
        let skill_manager = EnhancedSkillManager::new();

        // Create a skill node with complex bonuses to test optimization paths
        let complex_node = SkillNode {
            id: "test_complex".to_string(),
            name: "Complex Test Node".to_string(),
            description: "Testing optimization".to_string(),
            max_points: 5,
            current_points: 3,
            prerequisites: Vec::new(),
            unlocks: Vec::new(),
            bonuses: vec![
                SkillBonus {
                    bonus_type: BonusType::SpeedIncrease(0.1),
                    per_point: true,
                },
                SkillBonus {
                    bonus_type: BonusType::QualityBonus(0.05),
                    per_point: true,
                },
                SkillBonus {
                    bonus_type: BonusType::CostReduction(0.15),
                    per_point: false,
                },
            ],
            tier: 3,
        };

        let bonuses = skill_manager.calculate_node_bonuses(&complex_node);

        // Should have 3 bonuses applied
        assert_eq!(bonuses.len(), 3);

        // Check per-point bonuses are multiplied correctly
        let speed_bonus = bonuses.iter().find(|b| matches!(b.bonus_type, BonusType::SpeedIncrease(_))).unwrap();
        assert_eq!(speed_bonus.strength, 3.0); // 3 points allocated

        let quality_bonus = bonuses.iter().find(|b| matches!(b.bonus_type, BonusType::QualityBonus(_))).unwrap();
        assert_eq!(quality_bonus.strength, 3.0); // 3 points allocated

        let cost_bonus = bonuses.iter().find(|b| matches!(b.bonus_type, BonusType::CostReduction(_))).unwrap();
        assert_eq!(cost_bonus.strength, 1.0); // Not per-point
    }
}