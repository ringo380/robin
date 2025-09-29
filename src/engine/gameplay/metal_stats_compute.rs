/*!
 * Metal Compute Integration for Player Stats
 *
 * Apple Silicon optimization using Metal compute shaders for high-performance
 * player attribute calculations. Leverages unified memory architecture for
 * zero-copy data sharing and parallel processing capabilities.
 */

use crate::engine::{
    error::{RobinError, RobinResult},
    gameplay::player_attributes::{
        CoreAttributes, DerivedStats, EquipmentModifiers, TemporaryEffect,
        CoreAttributeType, DerivedStatType
    },
};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Metal compute manager for Apple Silicon stat calculations
#[cfg(target_os = "macos")]
pub struct MetalStatsCompute {
    /// Metal device handle
    device: metal::Device,
    /// Command queue for compute operations
    command_queue: metal::CommandQueue,
    /// Compute pipeline for stat calculations
    stat_pipeline: metal::ComputePipelineState,
    /// Buffer pool for efficient memory management
    buffer_pool: MetalBufferPool,
    /// Performance metrics
    performance_tracker: MetalPerformanceTracker,
}

#[cfg(target_os = "macos")]
impl MetalStatsCompute {
    /// Initialize Metal compute system for stat calculations
    pub fn new() -> RobinResult<Self> {
        // Get default Metal device (Apple Silicon)
        let device = metal::Device::system_default()
            .ok_or_else(|| RobinError::PlatformNotSupported("Metal not available".to_string()))?;

        // Create command queue
        let command_queue = device.new_command_queue();

        // Load and compile compute shaders
        let library = Self::create_compute_library(&device)?;
        let stat_pipeline = Self::create_stat_pipeline(&device, &library)?;

        // Initialize buffer pool
        let buffer_pool = MetalBufferPool::new(&device)?;

        Ok(Self {
            device,
            command_queue,
            stat_pipeline,
            buffer_pool,
            performance_tracker: MetalPerformanceTracker::new(),
        })
    }

    /// Calculate all derived stats using Metal compute shaders
    pub fn calculate_all_derived_stats(
        &self,
        core_attributes: &CoreAttributes,
        equipment_modifiers: &EquipmentModifiers,
        temporary_effects: &[TemporaryEffect],
    ) -> RobinResult<DerivedStats> {
        let start_time = std::time::Instant::now();

        // Prepare input data for GPU
        let input_data = StatCalculationInput {
            core_attributes: *core_attributes,
            equipment_bonuses: equipment_modifiers.get_total_bonuses(),
            temporary_effect_count: temporary_effects.len() as u32,
            temporary_effects: temporary_effects.iter()
                .flat_map(|effect| effect.attribute_modifiers.iter())
                .map(|(attr, bonus)| (*attr, *bonus))
                .collect(),
        };

        // Create Metal buffers using unified memory
        let input_buffer = self.create_input_buffer(&input_data)?;
        let output_buffer = self.create_output_buffer()?;

        // Execute compute shader
        let command_buffer = self.command_queue.new_command_buffer();
        let compute_encoder = command_buffer.new_compute_command_encoder();

        // Set compute pipeline and buffers
        compute_encoder.set_compute_pipeline_state(&self.stat_pipeline);
        compute_encoder.set_buffer(0, Some(&input_buffer), 0);
        compute_encoder.set_buffer(1, Some(&output_buffer), 0);

        // Dispatch compute threads (one thread per derived stat)
        let threads_per_group = metal::MTLSize::new(1, 1, 1);
        let thread_groups = metal::MTLSize::new(DerivedStatType::all().len() as u64, 1, 1);

        compute_encoder.dispatch_thread_groups(thread_groups, threads_per_group);
        compute_encoder.end_encoding();

        // Execute and wait for completion
        command_buffer.commit();
        command_buffer.wait_until_completed();

        // Read results from unified memory (zero-copy on Apple Silicon)
        let results = self.read_output_buffer(&output_buffer)?;

        // Record performance metrics
        let calculation_time = start_time.elapsed();
        self.performance_tracker.record_calculation(calculation_time, true);

        Ok(results)
    }

    /// Create Metal compute library with stat calculation shaders
    fn create_compute_library(device: &metal::Device) -> RobinResult<metal::Library> {
        let compute_shader_source = r#"
            #include <metal_stdlib>
            using namespace metal;

            // Input structure for stat calculations
            struct StatCalculationInput {
                // Core attributes (12 values)
                uint strength;
                uint dexterity;
                uint intelligence;
                uint vitality;
                uint willpower;
                uint charisma;
                uint focus;
                uint creativity;
                uint perception;
                uint endurance;
                uint luck;
                uint resonance;

                // Equipment bonuses (12 values)
                int equipment_strength;
                int equipment_dexterity;
                int equipment_intelligence;
                int equipment_vitality;
                int equipment_willpower;
                int equipment_charisma;
                int equipment_focus;
                int equipment_creativity;
                int equipment_perception;
                int equipment_endurance;
                int equipment_luck;
                int equipment_resonance;

                // Temporary effect modifiers (12 values)
                int temp_strength;
                int temp_dexterity;
                int temp_intelligence;
                int temp_vitality;
                int temp_willpower;
                int temp_charisma;
                int temp_focus;
                int temp_creativity;
                int temp_perception;
                int temp_endurance;
                int temp_luck;
                int temp_resonance;
            };

            // Output structure for derived stats
            struct StatCalculationOutput {
                float max_health;
                float max_stamina;
                float max_mana;
                float carry_capacity;
                float critical_chance;
                float movement_speed;
                float building_speed;
                float crafting_quality;
                float resource_yield;
                float xp_gain_multiplier;
            };

            // Helper function to get final attribute value
            uint get_final_attribute(uint base, int equipment, int temp) {
                return max(1u, uint(max(0, int(base) + equipment + temp)));
            }

            // Main compute kernel for stat calculations
            kernel void calculate_derived_stats(
                constant StatCalculationInput& input [[buffer(0)]],
                device StatCalculationOutput& output [[buffer(1)]],
                uint thread_id [[thread_position_in_grid]]
            ) {
                // Calculate final attribute values
                uint final_strength = get_final_attribute(input.strength, input.equipment_strength, input.temp_strength);
                uint final_dexterity = get_final_attribute(input.dexterity, input.equipment_dexterity, input.temp_dexterity);
                uint final_intelligence = get_final_attribute(input.intelligence, input.equipment_intelligence, input.temp_intelligence);
                uint final_vitality = get_final_attribute(input.vitality, input.equipment_vitality, input.temp_vitality);
                uint final_willpower = get_final_attribute(input.willpower, input.equipment_willpower, input.temp_willpower);
                uint final_charisma = get_final_attribute(input.charisma, input.equipment_charisma, input.temp_charisma);
                uint final_focus = get_final_attribute(input.focus, input.equipment_focus, input.temp_focus);
                uint final_creativity = get_final_attribute(input.creativity, input.equipment_creativity, input.temp_creativity);
                uint final_perception = get_final_attribute(input.perception, input.equipment_perception, input.temp_perception);
                uint final_endurance = get_final_attribute(input.endurance, input.equipment_endurance, input.temp_endurance);
                uint final_luck = get_final_attribute(input.luck, input.equipment_luck, input.temp_luck);
                uint final_resonance = get_final_attribute(input.resonance, input.equipment_resonance, input.temp_resonance);

                // Calculate derived stats using parallel processing
                // Each thread calculates all stats simultaneously for maximum efficiency
                output.max_health = 100.0 + (float(final_vitality) * 10.0) + (float(final_endurance) * 5.0);
                output.max_stamina = 100.0 + (float(final_endurance) * 8.0) + (float(final_strength) * 2.0);
                output.max_mana = 50.0 + (float(final_intelligence) * 6.0) + (float(final_willpower) * 4.0);
                output.carry_capacity = 50.0 + (float(final_strength) * 3.0) + (float(final_endurance) * 1.5);
                output.critical_chance = min(50.0, (float(final_luck) * 0.5) + (float(final_perception) * 0.3));
                output.movement_speed = 1.0 + (float(final_dexterity) * 0.02) + (float(final_endurance) * 0.01);
                output.building_speed = 1.0 + (float(final_dexterity) * 0.03) + (float(final_focus) * 0.02);
                output.crafting_quality = 1.0 + (float(final_intelligence + final_creativity + final_focus) * 0.015);
                output.resource_yield = 1.0 + (float(final_perception) * 0.02) + (float(final_luck) * 0.025);
                output.xp_gain_multiplier = 1.0 + (float(final_intelligence + final_focus) * 0.01);
            }
        "#;

        device.new_library_with_source(compute_shader_source, &metal::CompileOptions::new())
            .map_err(|error| RobinError::CompilationError(format!("Metal shader compilation failed: {}", error)))
    }

    /// Create compute pipeline state for stat calculations
    fn create_stat_pipeline(device: &metal::Device, library: &metal::Library) -> RobinResult<metal::ComputePipelineState> {
        let function = library.get_function("calculate_derived_stats", None)
            .map_err(|_| RobinError::CompilationError("Stat calculation function not found".to_string()))?;

        device.new_compute_pipeline_state_with_function(&function)
            .map_err(|error| RobinError::CompilationError(format!("Pipeline creation failed: {}", error)))
    }

    /// Create input buffer with stat calculation data
    fn create_input_buffer(&self, input_data: &StatCalculationInput) -> RobinResult<metal::Buffer> {
        // Convert input data to Metal-compatible format
        let metal_input = MetalStatInput::from_input_data(input_data);
        let buffer_size = std::mem::size_of::<MetalStatInput>() as u64;

        let buffer = self.device.new_buffer_with_data(
            &metal_input as *const MetalStatInput as *const std::ffi::c_void,
            buffer_size,
            metal::MTLResourceOptions::StorageModeShared, // Use shared memory for Apple Silicon
        );

        Ok(buffer)
    }

    /// Create output buffer for derived stats
    fn create_output_buffer(&self) -> RobinResult<metal::Buffer> {
        let buffer_size = std::mem::size_of::<MetalStatOutput>() as u64;

        let buffer = self.device.new_buffer(
            buffer_size,
            metal::MTLResourceOptions::StorageModeShared,
        );

        Ok(buffer)
    }

    /// Read output buffer and convert to DerivedStats
    fn read_output_buffer(&self, buffer: &metal::Buffer) -> RobinResult<DerivedStats> {
        // Use unified memory for zero-copy access on Apple Silicon
        let contents = buffer.contents() as *const MetalStatOutput;
        let output = unsafe { *contents };

        Ok(DerivedStats {
            max_health: output.max_health,
            max_stamina: output.max_stamina,
            max_mana: output.max_mana,
            carry_capacity: output.carry_capacity,
            critical_chance: output.critical_chance,
            movement_speed: output.movement_speed,
            building_speed: output.building_speed,
            crafting_quality: output.crafting_quality,
            resource_yield: output.resource_yield,
            xp_gain_multiplier: output.xp_gain_multiplier,
        })
    }
}

/// Input data structure for stat calculations
#[derive(Debug, Clone)]
pub struct StatCalculationInput {
    pub core_attributes: CoreAttributes,
    pub equipment_bonuses: HashMap<CoreAttributeType, i32>,
    pub temporary_effect_count: u32,
    pub temporary_effects: Vec<(CoreAttributeType, i32)>,
}

/// Metal-compatible input structure (C-repr for shader compatibility)
#[repr(C)]
struct MetalStatInput {
    // Core attributes
    strength: u32,
    dexterity: u32,
    intelligence: u32,
    vitality: u32,
    willpower: u32,
    charisma: u32,
    focus: u32,
    creativity: u32,
    perception: u32,
    endurance: u32,
    luck: u32,
    resonance: u32,

    // Equipment bonuses
    equipment_strength: i32,
    equipment_dexterity: i32,
    equipment_intelligence: i32,
    equipment_vitality: i32,
    equipment_willpower: i32,
    equipment_charisma: i32,
    equipment_focus: i32,
    equipment_creativity: i32,
    equipment_perception: i32,
    equipment_endurance: i32,
    equipment_luck: i32,
    equipment_resonance: i32,

    // Temporary effects
    temp_strength: i32,
    temp_dexterity: i32,
    temp_intelligence: i32,
    temp_vitality: i32,
    temp_willpower: i32,
    temp_charisma: i32,
    temp_focus: i32,
    temp_creativity: i32,
    temp_perception: i32,
    temp_endurance: i32,
    temp_luck: i32,
    temp_resonance: i32,
}

impl MetalStatInput {
    fn from_input_data(input: &StatCalculationInput) -> Self {
        let get_equipment_bonus = |attr: CoreAttributeType| -> i32 {
            input.equipment_bonuses.get(&attr).copied().unwrap_or(0)
        };

        let get_temp_bonus = |attr: CoreAttributeType| -> i32 {
            input.temporary_effects.iter()
                .filter(|(a, _)| *a == attr)
                .map(|(_, bonus)| *bonus)
                .sum()
        };

        Self {
            // Core attributes
            strength: input.core_attributes.strength,
            dexterity: input.core_attributes.dexterity,
            intelligence: input.core_attributes.intelligence,
            vitality: input.core_attributes.vitality,
            willpower: input.core_attributes.willpower,
            charisma: input.core_attributes.charisma,
            focus: input.core_attributes.focus,
            creativity: input.core_attributes.creativity,
            perception: input.core_attributes.perception,
            endurance: input.core_attributes.endurance,
            luck: input.core_attributes.luck,
            resonance: input.core_attributes.resonance,

            // Equipment bonuses
            equipment_strength: get_equipment_bonus(CoreAttributeType::Strength),
            equipment_dexterity: get_equipment_bonus(CoreAttributeType::Dexterity),
            equipment_intelligence: get_equipment_bonus(CoreAttributeType::Intelligence),
            equipment_vitality: get_equipment_bonus(CoreAttributeType::Vitality),
            equipment_willpower: get_equipment_bonus(CoreAttributeType::Willpower),
            equipment_charisma: get_equipment_bonus(CoreAttributeType::Charisma),
            equipment_focus: get_equipment_bonus(CoreAttributeType::Focus),
            equipment_creativity: get_equipment_bonus(CoreAttributeType::Creativity),
            equipment_perception: get_equipment_bonus(CoreAttributeType::Perception),
            equipment_endurance: get_equipment_bonus(CoreAttributeType::Endurance),
            equipment_luck: get_equipment_bonus(CoreAttributeType::Luck),
            equipment_resonance: get_equipment_bonus(CoreAttributeType::Resonance),

            // Temporary effects
            temp_strength: get_temp_bonus(CoreAttributeType::Strength),
            temp_dexterity: get_temp_bonus(CoreAttributeType::Dexterity),
            temp_intelligence: get_temp_bonus(CoreAttributeType::Intelligence),
            temp_vitality: get_temp_bonus(CoreAttributeType::Vitality),
            temp_willpower: get_temp_bonus(CoreAttributeType::Willpower),
            temp_charisma: get_temp_bonus(CoreAttributeType::Charisma),
            temp_focus: get_temp_bonus(CoreAttributeType::Focus),
            temp_creativity: get_temp_bonus(CoreAttributeType::Creativity),
            temp_perception: get_temp_bonus(CoreAttributeType::Perception),
            temp_endurance: get_temp_bonus(CoreAttributeType::Endurance),
            temp_luck: get_temp_bonus(CoreAttributeType::Luck),
            temp_resonance: get_temp_bonus(CoreAttributeType::Resonance),
        }
    }
}

/// Metal-compatible output structure
#[repr(C)]
struct MetalStatOutput {
    max_health: f32,
    max_stamina: f32,
    max_mana: f32,
    carry_capacity: f32,
    critical_chance: f32,
    movement_speed: f32,
    building_speed: f32,
    crafting_quality: f32,
    resource_yield: f32,
    xp_gain_multiplier: f32,
}

/// Metal buffer pool for efficient memory management
#[cfg(target_os = "macos")]
pub struct MetalBufferPool {
    device: metal::Device,
    available_buffers: Vec<metal::Buffer>,
}

#[cfg(target_os = "macos")]
impl MetalBufferPool {
    pub fn new(device: &metal::Device) -> RobinResult<Self> {
        Ok(Self {
            device: device.clone(),
            available_buffers: Vec::new(),
        })
    }

    pub fn get_buffer(&mut self, size: u64) -> metal::Buffer {
        // Try to reuse an existing buffer
        if let Some(buffer) = self.available_buffers.pop() {
            if buffer.length() >= size {
                return buffer;
            }
        }

        // Create new buffer if none available
        self.device.new_buffer(
            size,
            metal::MTLResourceOptions::StorageModeShared,
        )
    }

    pub fn return_buffer(&mut self, buffer: metal::Buffer) {
        self.available_buffers.push(buffer);
    }
}

/// Performance tracking for Metal compute operations
pub struct MetalPerformanceTracker {
    metal_calculations: u64,
    cpu_calculations: u64,
    total_time_metal: std::time::Duration,
    total_time_cpu: std::time::Duration,
    last_calculation_time: std::time::Duration,
}

impl MetalPerformanceTracker {
    pub fn new() -> Self {
        Self {
            metal_calculations: 0,
            cpu_calculations: 0,
            total_time_metal: std::time::Duration::ZERO,
            total_time_cpu: std::time::Duration::ZERO,
            last_calculation_time: std::time::Duration::ZERO,
        }
    }

    pub fn record_calculation(&mut self, duration: std::time::Duration, used_metal: bool) {
        self.last_calculation_time = duration;

        if used_metal {
            self.metal_calculations += 1;
            self.total_time_metal += duration;
        } else {
            self.cpu_calculations += 1;
            self.total_time_cpu += duration;
        }
    }

    pub fn get_performance_stats(&self) -> MetalPerformanceStats {
        let total_calculations = self.metal_calculations + self.cpu_calculations;

        MetalPerformanceStats {
            metal_usage_percentage: if total_calculations > 0 {
                (self.metal_calculations as f32 / total_calculations as f32) * 100.0
            } else {
                0.0
            },
            average_metal_time: if self.metal_calculations > 0 {
                self.total_time_metal / self.metal_calculations as u32
            } else {
                std::time::Duration::ZERO
            },
            average_cpu_time: if self.cpu_calculations > 0 {
                self.total_time_cpu / self.cpu_calculations as u32
            } else {
                std::time::Duration::ZERO
            },
            speedup_factor: if self.cpu_calculations > 0 && self.metal_calculations > 0 {
                let avg_cpu = self.total_time_cpu.as_nanos() as f32 / self.cpu_calculations as f32;
                let avg_metal = self.total_time_metal.as_nanos() as f32 / self.metal_calculations as f32;
                if avg_metal > 0.0 {
                    avg_cpu / avg_metal
                } else {
                    1.0
                }
            } else {
                1.0
            },
            last_calculation_time: self.last_calculation_time,
        }
    }
}

/// Performance statistics for Metal compute operations
#[derive(Debug, Clone)]
pub struct MetalPerformanceStats {
    pub metal_usage_percentage: f32,
    pub average_metal_time: std::time::Duration,
    pub average_cpu_time: std::time::Duration,
    pub speedup_factor: f32, // How much faster Metal is vs CPU
    pub last_calculation_time: std::time::Duration,
}

/// Fallback CPU implementation for non-Apple platforms
#[cfg(not(target_os = "macos"))]
pub struct MetalStatsCompute;

#[cfg(not(target_os = "macos"))]
impl MetalStatsCompute {
    pub fn new() -> RobinResult<Self> {
        Err(RobinError::PlatformNotSupported("Metal not available on this platform".to_string()))
    }

    pub fn calculate_all_derived_stats(
        &self,
        _core_attributes: &CoreAttributes,
        _equipment_modifiers: &EquipmentModifiers,
        _temporary_effects: &[TemporaryEffect],
    ) -> RobinResult<DerivedStats> {
        Err(RobinError::PlatformNotSupported("Metal not available on this platform".to_string()))
    }
}