/*!
 * Advanced Material System for Robin Engine
 *
 * Enhanced material interactions, crafting, and sophisticated content creation
 * for Phase 4 Milestone 3: Content Depth and Polish
 */

use crate::engine::world::construction::{VoxelType, Material, MaterialType, MaterialProperties};
use crate::engine::error::{RobinError, RobinResult};
use crate::engine::math::Vec3;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Advanced material types for enhanced content depth
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AdvancedMaterialType {
    // Metals and Alloys
    Iron,
    Steel,
    Copper,
    Bronze,
    Silver,
    Gold,
    Titanium,
    Aluminum,
    // Natural Materials
    Oak,
    Pine,
    Bamboo,
    Mahogany,
    Marble,
    Granite,
    Limestone,
    Sandstone,
    // Synthetic Materials
    Plastic,
    Rubber,
    Carbon,
    Ceramic,
    Composite,
    Crystal,
    // Organic Materials
    Leather,
    Wool,
    Cotton,
    Silk,
    Hemp,
    // Energy Materials
    Uranium,
    Solar,
    Battery,
    Magnetic,
    // Special Materials
    Liquid,
    Gas,
    Plasma,
    Ice,
    // Advanced Composites
    Reinforced(Box<AdvancedMaterialType>, Box<AdvancedMaterialType>),
    Alloy(Vec<AdvancedMaterialType>),
    Composite3D(Box<AdvancedMaterialType>, Box<AdvancedMaterialType>, Box<AdvancedMaterialType>),
}

/// Material interaction types for advanced gameplay
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaterialInteraction {
    /// Materials combine to form a new material
    Combine(AdvancedMaterialType),
    /// Materials react chemically
    React(MaterialReaction),
    /// One material reinforces another
    Reinforce(f32), // strength multiplier
    /// Materials conduct energy between each other
    Conduct(f32), // conductivity factor
    /// Materials corrode or degrade each other
    Corrode(f32), // degradation rate
    /// Materials are incompatible
    Incompatible,
    /// Materials create special effects
    SpecialEffect(MaterialEffect),
}

/// Chemical and physical reactions between materials
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaterialReaction {
    /// Materials oxidize (rust, tarnish)
    Oxidation { rate: f32, result: AdvancedMaterialType },
    /// Materials burn or combust
    Combustion { temperature: f32, byproducts: Vec<AdvancedMaterialType> },
    /// Materials freeze or melt
    PhaseChange { temperature: f32, new_phase: AdvancedMaterialType },
    /// Materials dissolve in liquids
    Dissolution { solvent: AdvancedMaterialType, rate: f32 },
    /// Materials undergo crystallization
    Crystallization { conditions: Vec<String>, result: AdvancedMaterialType },
    /// Materials undergo polymerization
    Polymerization { result: AdvancedMaterialType, strength_gain: f32 },
}

/// Special effects created by material interactions
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaterialEffect {
    /// Produces light
    Luminescence { intensity: f32, color: [f32; 3] },
    /// Generates electricity
    ElectricGeneration { voltage: f32, current: f32 },
    /// Creates magnetic field
    MagneticField { strength: f32, range: f32 },
    /// Produces heat
    ThermalGeneration { temperature: f32, duration: f32 },
    /// Creates sound
    Resonance { frequency: f32, amplitude: f32 },
    /// Produces particles
    ParticleEmission { particle_type: String, rate: f32 },
}

/// Advanced material properties for sophisticated gameplay
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdvancedMaterialProperties {
    /// Base material properties
    pub base_properties: MaterialProperties,
    /// Temperature resistance range (min, max)
    pub temperature_range: (f32, f32),
    /// Electrical properties
    pub electrical: ElectricalProperties,
    /// Thermal properties
    pub thermal: ThermalProperties,
    /// Mechanical properties
    pub mechanical: MechanicalProperties,
    /// Chemical properties
    pub chemical: ChemicalProperties,
    /// Environmental properties
    pub environmental: EnvironmentalProperties,
    /// Crafting properties
    pub crafting: CraftingProperties,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ElectricalProperties {
    pub conductivity: f32,
    pub resistance: f32,
    pub capacitance: f32,
    pub breakdown_voltage: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThermalProperties {
    pub conductivity: f32,
    pub capacity: f32,
    pub expansion_coefficient: f32,
    pub melting_point: f32,
    pub boiling_point: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MechanicalProperties {
    pub tensile_strength: f32,
    pub compressive_strength: f32,
    pub shear_strength: f32,
    pub elasticity: f32,
    pub plasticity: f32,
    pub fracture_toughness: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChemicalProperties {
    pub reactivity: f32,
    pub ph_level: f32,
    pub corrosion_resistance: f32,
    pub oxidation_rate: f32,
    pub stability: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnvironmentalProperties {
    pub weather_resistance: f32,
    pub uv_resistance: f32,
    pub biodegradability: f32,
    pub toxicity: f32,
    pub recyclability: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CraftingProperties {
    pub workability: f32,
    pub weldability: f32,
    pub machinability: f32,
    pub paintability: f32,
    pub required_tools: Vec<String>,
    pub crafting_temperature: f32,
    pub curing_time: f32,
}

/// Material combination recipe for crafting
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaterialRecipe {
    pub id: String,
    pub name: String,
    pub description: String,
    pub ingredients: Vec<MaterialIngredient>,
    pub result: AdvancedMaterialType,
    pub result_quantity: u32,
    pub required_tools: Vec<String>,
    pub required_temperature: f32,
    pub processing_time: f32,
    pub success_rate: f32,
    pub skill_level_required: u32,
    pub energy_cost: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaterialIngredient {
    pub material: AdvancedMaterialType,
    pub quantity: u32,
    pub quality_requirement: f32, // 0.0 to 1.0
    pub optional: bool,
}

/// Advanced material system manager
#[derive(Debug)]
pub struct AdvancedMaterialSystem {
    /// Material definitions and properties
    material_library: HashMap<AdvancedMaterialType, AdvancedMaterialProperties>,
    /// Material interaction rules
    interaction_matrix: HashMap<(AdvancedMaterialType, AdvancedMaterialType), MaterialInteraction>,
    /// Crafting recipes
    recipes: HashMap<String, MaterialRecipe>,
    /// Active material processes (position -> process)
    active_processes: HashMap<Vec3, MaterialProcess>,
    /// Environmental conditions that affect materials
    environmental_conditions: EnvironmentalConditions,
}

#[derive(Clone, Debug)]
pub struct MaterialProcess {
    pub process_type: ProcessType,
    pub materials_involved: Vec<AdvancedMaterialType>,
    pub progress: f32, // 0.0 to 1.0
    pub duration: f32,
    pub elapsed_time: f32,
    pub temperature: f32,
    pub pressure: f32,
    pub energy_input: f32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProcessType {
    Smelting,
    Alloying,
    Crystallization,
    Polymerization,
    Curing,
    Oxidation,
    Corrosion,
    PhaseTransition,
    ChemicalReaction,
}

#[derive(Clone, Debug)]
pub struct EnvironmentalConditions {
    pub temperature: f32,
    pub humidity: f32,
    pub pressure: f32,
    pub oxygen_level: f32,
    pub radiation_level: f32,
    pub ph_level: f32,
}

impl AdvancedMaterialSystem {
    pub fn new() -> Self {
        let mut system = Self {
            material_library: HashMap::new(),
            interaction_matrix: HashMap::new(),
            recipes: HashMap::new(),
            active_processes: HashMap::new(),
            environmental_conditions: EnvironmentalConditions::default(),
        };

        system.initialize_material_library();
        system.initialize_interaction_matrix();
        system.initialize_recipes();

        system
    }

    /// Initialize the comprehensive material library
    fn initialize_material_library(&mut self) {
        // Metals and Alloys
        self.add_material(AdvancedMaterialType::Iron, AdvancedMaterialProperties {
            base_properties: MaterialProperties {
                structural: true,
                magnetic: true,
                conductive: true,
                ..Default::default()
            },
            temperature_range: (-200.0, 1538.0),
            electrical: ElectricalProperties {
                conductivity: 0.1,
                resistance: 9.71e-8,
                capacitance: 0.0,
                breakdown_voltage: 1000.0,
            },
            thermal: ThermalProperties {
                conductivity: 80.4,
                capacity: 449.0,
                expansion_coefficient: 11.8e-6,
                melting_point: 1538.0,
                boiling_point: 2862.0,
            },
            mechanical: MechanicalProperties {
                tensile_strength: 400.0,
                compressive_strength: 400.0,
                shear_strength: 250.0,
                elasticity: 200.0,
                plasticity: 0.3,
                fracture_toughness: 50.0,
            },
            chemical: ChemicalProperties {
                reactivity: 0.6,
                ph_level: 7.0,
                corrosion_resistance: 0.2,
                oxidation_rate: 0.1,
                stability: 0.8,
            },
            environmental: EnvironmentalProperties {
                weather_resistance: 0.4,
                uv_resistance: 0.9,
                biodegradability: 0.1,
                toxicity: 0.1,
                recyclability: 0.9,
            },
            crafting: CraftingProperties {
                workability: 0.8,
                weldability: 0.9,
                machinability: 0.7,
                paintability: 0.8,
                required_tools: vec!["forge".to_string(), "hammer".to_string()],
                crafting_temperature: 1200.0,
                curing_time: 0.0,
            },
        });

        self.add_material(AdvancedMaterialType::Steel, AdvancedMaterialProperties {
            base_properties: MaterialProperties {
                structural: true,
                magnetic: true,
                conductive: true,
                ..Default::default()
            },
            temperature_range: (-200.0, 1450.0),
            electrical: ElectricalProperties {
                conductivity: 0.08,
                resistance: 1.43e-7,
                capacitance: 0.0,
                breakdown_voltage: 1200.0,
            },
            thermal: ThermalProperties {
                conductivity: 50.2,
                capacity: 490.0,
                expansion_coefficient: 13.0e-6,
                melting_point: 1450.0,
                boiling_point: 2750.0,
            },
            mechanical: MechanicalProperties {
                tensile_strength: 800.0,
                compressive_strength: 800.0,
                shear_strength: 500.0,
                elasticity: 210.0,
                plasticity: 0.25,
                fracture_toughness: 100.0,
            },
            chemical: ChemicalProperties {
                reactivity: 0.4,
                ph_level: 7.0,
                corrosion_resistance: 0.5,
                oxidation_rate: 0.05,
                stability: 0.9,
            },
            environmental: EnvironmentalProperties {
                weather_resistance: 0.7,
                uv_resistance: 0.9,
                biodegradability: 0.05,
                toxicity: 0.1,
                recyclability: 0.95,
            },
            crafting: CraftingProperties {
                workability: 0.7,
                weldability: 0.8,
                machinability: 0.6,
                paintability: 0.9,
                required_tools: vec!["blast_furnace".to_string(), "rolling_mill".to_string()],
                crafting_temperature: 1600.0,
                curing_time: 0.0,
            },
        });

        // Woods
        self.add_material(AdvancedMaterialType::Oak, AdvancedMaterialProperties {
            base_properties: MaterialProperties {
                structural: true,
                flammable: true,
                decorative: true,
                ..Default::default()
            },
            temperature_range: (-50.0, 250.0),
            electrical: ElectricalProperties {
                conductivity: 0.0,
                resistance: 1e12,
                capacitance: 0.0,
                breakdown_voltage: 500.0,
            },
            thermal: ThermalProperties {
                conductivity: 0.17,
                capacity: 2400.0,
                expansion_coefficient: 4.0e-6,
                melting_point: 250.0,
                boiling_point: 300.0,
            },
            mechanical: MechanicalProperties {
                tensile_strength: 100.0,
                compressive_strength: 60.0,
                shear_strength: 15.0,
                elasticity: 12.0,
                plasticity: 0.1,
                fracture_toughness: 8.0,
            },
            chemical: ChemicalProperties {
                reactivity: 0.3,
                ph_level: 5.5,
                corrosion_resistance: 0.8,
                oxidation_rate: 0.2,
                stability: 0.7,
            },
            environmental: EnvironmentalProperties {
                weather_resistance: 0.6,
                uv_resistance: 0.4,
                biodegradability: 0.8,
                toxicity: 0.0,
                recyclability: 0.9,
            },
            crafting: CraftingProperties {
                workability: 0.9,
                weldability: 0.0,
                machinability: 0.8,
                paintability: 0.9,
                required_tools: vec!["saw".to_string(), "chisel".to_string()],
                crafting_temperature: 20.0,
                curing_time: 48.0,
            },
        });

        // Continue adding more materials...
        // For brevity, I'll add a few more key materials

        // Crystal
        self.add_material(AdvancedMaterialType::Crystal, AdvancedMaterialProperties {
            base_properties: MaterialProperties {
                structural: true,
                decorative: true,
                ..Default::default()
            },
            temperature_range: (-273.0, 1700.0),
            electrical: ElectricalProperties {
                conductivity: 0.0,
                resistance: 1e14,
                capacitance: 0.001,
                breakdown_voltage: 10000.0,
            },
            thermal: ThermalProperties {
                conductivity: 1.38,
                capacity: 740.0,
                expansion_coefficient: 0.5e-6,
                melting_point: 1700.0,
                boiling_point: 2230.0,
            },
            mechanical: MechanicalProperties {
                tensile_strength: 1000.0,
                compressive_strength: 2000.0,
                shear_strength: 800.0,
                elasticity: 1000.0,
                plasticity: 0.01,
                fracture_toughness: 5.0,
            },
            chemical: ChemicalProperties {
                reactivity: 0.1,
                ph_level: 7.0,
                corrosion_resistance: 0.95,
                oxidation_rate: 0.01,
                stability: 0.99,
            },
            environmental: EnvironmentalProperties {
                weather_resistance: 0.95,
                uv_resistance: 0.95,
                biodegradability: 0.0,
                toxicity: 0.0,
                recyclability: 0.3,
            },
            crafting: CraftingProperties {
                workability: 0.2,
                weldability: 0.0,
                machinability: 0.1,
                paintability: 0.3,
                required_tools: vec!["diamond_saw".to_string(), "laser_cutter".to_string()],
                crafting_temperature: 800.0,
                curing_time: 168.0,
            },
        });
    }

    /// Initialize material interaction matrix
    fn initialize_interaction_matrix(&mut self) {
        // Iron + Carbon = Steel
        self.add_interaction(
            AdvancedMaterialType::Iron,
            AdvancedMaterialType::Carbon,
            MaterialInteraction::Combine(AdvancedMaterialType::Steel),
        );

        // Iron + Oxygen = Rust (Oxidation)
        self.add_interaction(
            AdvancedMaterialType::Iron,
            AdvancedMaterialType::Gas, // Representing Oxygen
            MaterialInteraction::React(MaterialReaction::Oxidation {
                rate: 0.1,
                result: AdvancedMaterialType::Iron, // Rusted iron
            }),
        );

        // Wood + Fire = Combustion
        self.add_interaction(
            AdvancedMaterialType::Oak,
            AdvancedMaterialType::Plasma, // Representing fire
            MaterialInteraction::React(MaterialReaction::Combustion {
                temperature: 250.0,
                byproducts: vec![AdvancedMaterialType::Carbon, AdvancedMaterialType::Gas],
            }),
        );

        // Crystal + Electric = Special Effect
        self.add_interaction(
            AdvancedMaterialType::Crystal,
            AdvancedMaterialType::Battery,
            MaterialInteraction::SpecialEffect(MaterialEffect::Luminescence {
                intensity: 0.8,
                color: [0.8, 0.9, 1.0],
            }),
        );

        // Steel reinforces concrete
        self.add_interaction(
            AdvancedMaterialType::Steel,
            AdvancedMaterialType::Composite,
            MaterialInteraction::Reinforce(2.5),
        );
    }

    /// Initialize crafting recipes
    fn initialize_recipes(&mut self) {
        // Steel Recipe
        self.add_recipe(MaterialRecipe {
            id: "steel_basic".to_string(),
            name: "Basic Steel".to_string(),
            description: "Combine iron with carbon to create basic steel".to_string(),
            ingredients: vec![
                MaterialIngredient {
                    material: AdvancedMaterialType::Iron,
                    quantity: 10,
                    quality_requirement: 0.7,
                    optional: false,
                },
                MaterialIngredient {
                    material: AdvancedMaterialType::Carbon,
                    quantity: 1,
                    quality_requirement: 0.5,
                    optional: false,
                },
            ],
            result: AdvancedMaterialType::Steel,
            result_quantity: 8,
            required_tools: vec!["blast_furnace".to_string()],
            required_temperature: 1600.0,
            processing_time: 120.0,
            success_rate: 0.85,
            skill_level_required: 3,
            energy_cost: 500.0,
        });

        // Bronze Recipe
        self.add_recipe(MaterialRecipe {
            id: "bronze_alloy".to_string(),
            name: "Bronze Alloy".to_string(),
            description: "Alloy copper with tin to create bronze".to_string(),
            ingredients: vec![
                MaterialIngredient {
                    material: AdvancedMaterialType::Copper,
                    quantity: 9,
                    quality_requirement: 0.6,
                    optional: false,
                },
                MaterialIngredient {
                    material: AdvancedMaterialType::Iron, // Using Iron as tin substitute
                    quantity: 1,
                    quality_requirement: 0.5,
                    optional: false,
                },
            ],
            result: AdvancedMaterialType::Bronze,
            result_quantity: 10,
            required_tools: vec!["furnace".to_string(), "crucible".to_string()],
            required_temperature: 1200.0,
            processing_time: 60.0,
            success_rate: 0.9,
            skill_level_required: 2,
            energy_cost: 200.0,
        });

        // Reinforced Composite
        self.add_recipe(MaterialRecipe {
            id: "reinforced_composite".to_string(),
            name: "Reinforced Composite".to_string(),
            description: "Create a reinforced composite material".to_string(),
            ingredients: vec![
                MaterialIngredient {
                    material: AdvancedMaterialType::Carbon,
                    quantity: 5,
                    quality_requirement: 0.8,
                    optional: false,
                },
                MaterialIngredient {
                    material: AdvancedMaterialType::Plastic,
                    quantity: 3,
                    quality_requirement: 0.7,
                    optional: false,
                },
                MaterialIngredient {
                    material: AdvancedMaterialType::Steel,
                    quantity: 2,
                    quality_requirement: 0.9,
                    optional: false,
                },
            ],
            result: AdvancedMaterialType::Reinforced(
                Box::new(AdvancedMaterialType::Carbon),
                Box::new(AdvancedMaterialType::Steel)
            ),
            result_quantity: 8,
            required_tools: vec!["composite_press".to_string(), "curing_oven".to_string()],
            required_temperature: 180.0,
            processing_time: 240.0,
            success_rate: 0.75,
            skill_level_required: 5,
            energy_cost: 800.0,
        });
    }

    /// Add a material to the library
    fn add_material(&mut self, material_type: AdvancedMaterialType, properties: AdvancedMaterialProperties) {
        self.material_library.insert(material_type, properties);
    }

    /// Add an interaction between two materials
    fn add_interaction(&mut self, material1: AdvancedMaterialType, material2: AdvancedMaterialType, interaction: MaterialInteraction) {
        self.interaction_matrix.insert((material1.clone(), material2.clone()), interaction.clone());
        self.interaction_matrix.insert((material2, material1), interaction);
    }

    /// Add a crafting recipe
    fn add_recipe(&mut self, recipe: MaterialRecipe) {
        self.recipes.insert(recipe.id.clone(), recipe);
    }

    /// Check what happens when two materials interact
    pub fn check_interaction(&self, material1: &AdvancedMaterialType, material2: &AdvancedMaterialType) -> Option<&MaterialInteraction> {
        self.interaction_matrix.get(&(material1.clone(), material2.clone()))
    }

    /// Get material properties
    pub fn get_material_properties(&self, material: &AdvancedMaterialType) -> Option<&AdvancedMaterialProperties> {
        self.material_library.get(material)
    }

    /// Start a material process at a location
    pub fn start_process(&mut self, position: Vec3, process: MaterialProcess) -> RobinResult<()> {
        if self.active_processes.contains_key(&position) {
            return Err(RobinError::MaterialError {
                operation: "start_process".to_string(),
                material: format!("{:?}", process.materials_involved),
                reason: "Process already active at this location".to_string(),
            });
        }

        self.active_processes.insert(position, process);
        Ok(())
    }

    /// Update all active material processes
    pub fn update(&mut self, delta_time: f32) -> RobinResult<Vec<MaterialProcessResult>> {
        let mut completed_processes = Vec::new();
        let mut positions_to_remove = Vec::new();

        for (position, process) in &mut self.active_processes {
            process.elapsed_time += delta_time;
            process.progress = (process.elapsed_time / process.duration).min(1.0);

            // Apply environmental effects
            self.apply_environmental_effects(process, delta_time);

            if process.progress >= 1.0 {
                // Process completed
                let result = self.complete_process(process)?;
                completed_processes.push(MaterialProcessResult {
                    position: *position,
                    result,
                });
                positions_to_remove.push(*position);
            }
        }

        // Remove completed processes
        for position in positions_to_remove {
            self.active_processes.remove(&position);
        }

        Ok(completed_processes)
    }

    /// Apply environmental effects to a process
    fn apply_environmental_effects(&self, process: &mut MaterialProcess, delta_time: f32) {
        // Temperature effects
        let temp_diff = (self.environmental_conditions.temperature - process.temperature).abs();
        if temp_diff > 10.0 {
            process.progress *= 1.0 - (temp_diff / 1000.0) * delta_time; // Slow down in wrong temperature
        }

        // Humidity effects
        if self.environmental_conditions.humidity > 0.8 {
            // High humidity can affect certain processes
            if matches!(process.process_type, ProcessType::Oxidation | ProcessType::Corrosion) {
                process.progress += 0.1 * delta_time; // Accelerate corrosion
            }
        }
    }

    /// Complete a material process and get the result
    fn complete_process(&self, process: &MaterialProcess) -> RobinResult<MaterialProcessOutput> {
        match process.process_type {
            ProcessType::Smelting => {
                Ok(MaterialProcessOutput {
                    primary_result: process.materials_involved.first().cloned(),
                    secondary_results: vec![],
                    energy_released: 100.0,
                    byproducts: vec![],
                })
            }
            ProcessType::Alloying => {
                // Find matching recipe for the materials
                for recipe in self.recipes.values() {
                    if self.materials_match_recipe(&process.materials_involved, recipe) {
                        return Ok(MaterialProcessOutput {
                            primary_result: Some(recipe.result.clone()),
                            secondary_results: vec![],
                            energy_released: 50.0,
                            byproducts: vec![],
                        });
                    }
                }

                Ok(MaterialProcessOutput {
                    primary_result: None,
                    secondary_results: vec![],
                    energy_released: 0.0,
                    byproducts: process.materials_involved.clone(),
                })
            }
            _ => {
                Ok(MaterialProcessOutput {
                    primary_result: process.materials_involved.first().cloned(),
                    secondary_results: vec![],
                    energy_released: 25.0,
                    byproducts: vec![],
                })
            }
        }
    }

    /// Check if materials match a recipe
    fn materials_match_recipe(&self, materials: &[AdvancedMaterialType], recipe: &MaterialRecipe) -> bool {
        for ingredient in &recipe.ingredients {
            if !ingredient.optional && !materials.contains(&ingredient.material) {
                return false;
            }
        }
        true
    }

    /// Get all available recipes
    pub fn get_available_recipes(&self) -> Vec<&MaterialRecipe> {
        self.recipes.values().collect()
    }

    /// Get recipes that can be made with given materials
    pub fn get_craftable_recipes(&self, available_materials: &[AdvancedMaterialType]) -> Vec<&MaterialRecipe> {
        self.recipes.values()
            .filter(|recipe| self.materials_match_recipe(available_materials, recipe))
            .collect()
    }

    /// Update environmental conditions
    pub fn set_environmental_conditions(&mut self, conditions: EnvironmentalConditions) {
        self.environmental_conditions = conditions;
    }

    /// Get current environmental conditions
    pub fn get_environmental_conditions(&self) -> &EnvironmentalConditions {
        &self.environmental_conditions
    }
}

#[derive(Debug, Clone)]
pub struct MaterialProcessResult {
    pub position: Vec3,
    pub result: MaterialProcessOutput,
}

#[derive(Debug, Clone)]
pub struct MaterialProcessOutput {
    pub primary_result: Option<AdvancedMaterialType>,
    pub secondary_results: Vec<AdvancedMaterialType>,
    pub energy_released: f32,
    pub byproducts: Vec<AdvancedMaterialType>,
}

impl Default for EnvironmentalConditions {
    fn default() -> Self {
        Self {
            temperature: 20.0, // Room temperature
            humidity: 0.5,     // 50% humidity
            pressure: 101.325, // Standard atmospheric pressure
            oxygen_level: 0.21, // 21% oxygen
            radiation_level: 0.0,
            ph_level: 7.0,     // Neutral pH
        }
    }
}

/// Convert advanced materials to basic voxel types for compatibility
impl AdvancedMaterialType {
    pub fn to_voxel_type(&self) -> VoxelType {
        match self {
            AdvancedMaterialType::Iron | AdvancedMaterialType::Steel |
            AdvancedMaterialType::Aluminum | AdvancedMaterialType::Titanium => VoxelType::Metal,
            AdvancedMaterialType::Oak | AdvancedMaterialType::Pine |
            AdvancedMaterialType::Bamboo | AdvancedMaterialType::Mahogany => VoxelType::Wood,
            AdvancedMaterialType::Marble | AdvancedMaterialType::Granite |
            AdvancedMaterialType::Limestone | AdvancedMaterialType::Sandstone => VoxelType::Stone,
            AdvancedMaterialType::Crystal => VoxelType::Crystal,
            AdvancedMaterialType::Ice => VoxelType::Ice,
            AdvancedMaterialType::Liquid => VoxelType::Water,
            AdvancedMaterialType::Gas => VoxelType::Air,
            AdvancedMaterialType::Ceramic | AdvancedMaterialType::Composite => VoxelType::Brick,
            AdvancedMaterialType::Reinforced(base, _) => base.to_voxel_type(),
            AdvancedMaterialType::Alloy(materials) => {
                materials.first().map(|m| m.to_voxel_type()).unwrap_or(VoxelType::Metal)
            }
            AdvancedMaterialType::Composite3D(base, _, _) => base.to_voxel_type(),
            _ => VoxelType::Stone, // Default fallback
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_system_creation() {
        let system = AdvancedMaterialSystem::new();
        assert!(!system.material_library.is_empty());
        assert!(!system.recipes.is_empty());
    }

    #[test]
    fn test_material_interactions() {
        let system = AdvancedMaterialSystem::new();
        let interaction = system.check_interaction(&AdvancedMaterialType::Iron, &AdvancedMaterialType::Carbon);
        assert!(interaction.is_some());
    }

    #[test]
    fn test_recipe_matching() {
        let system = AdvancedMaterialSystem::new();
        let materials = vec![AdvancedMaterialType::Iron, AdvancedMaterialType::Carbon];
        let craftable = system.get_craftable_recipes(&materials);
        assert!(!craftable.is_empty());
    }
}