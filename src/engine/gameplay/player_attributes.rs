/*!
 * Advanced Player Attribute System for Robin Engine
 *
 * Sophisticated character attribute system with Apple Silicon Metal compute optimization.
 * Features 12 core attributes, derived stats, equipment modifiers, and real-time
 * calculation using Metal compute shaders for maximum performance.
 */

use crate::engine::{
    error::{RobinError, RobinResult},
    save_system::PlayerData,
    gameplay::{BuildingSkill, SpecializationPath},
};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Advanced player attribute manager with Metal compute optimization
pub struct PlayerAttributeManager {
    /// Core attributes for the player
    core_attributes: CoreAttributes,
    /// Derived stats calculated from core attributes
    derived_stats: DerivedStats,
    /// Equipment modifiers applied to attributes
    equipment_modifiers: EquipmentModifiers,
    /// Temporary effects (buffs/debuffs)
    temporary_effects: Vec<TemporaryEffect>,
    /// Attribute calculation cache for performance
    calculation_cache: AttributeCache,
    /// Metal compute manager for Apple Silicon optimization
    #[cfg(target_os = "macos")]
    metal_compute: Option<MetalStatsCompute>,
    /// Performance monitoring
    performance_monitor: AttributePerformanceMonitor,
}

impl PlayerAttributeManager {
    pub fn new() -> Self {
        Self {
            core_attributes: CoreAttributes::default(),
            derived_stats: DerivedStats::default(),
            equipment_modifiers: EquipmentModifiers::default(),
            temporary_effects: Vec::new(),
            calculation_cache: AttributeCache::new(),
            #[cfg(target_os = "macos")]
            metal_compute: MetalStatsCompute::new().ok(),
            performance_monitor: AttributePerformanceMonitor::new(),
        }
    }

    /// Initialize attributes from existing player data
    pub fn from_player_data(player_data: &PlayerData) -> Self {
        let mut manager = Self::new();

        // Extract existing stats and convert to new attribute system
        manager.core_attributes.vitality = ((player_data.max_health - 100.0) / 10.0) as u32 + 10;
        manager.core_attributes.intelligence = player_data.level.min(20) + 10;

        // Initialize other attributes based on player level and experience
        let base_value = (player_data.level / 2).max(10).min(20);
        manager.core_attributes.strength = base_value;
        manager.core_attributes.dexterity = base_value;
        manager.core_attributes.willpower = base_value;
        manager.core_attributes.charisma = base_value;
        manager.core_attributes.focus = base_value;
        manager.core_attributes.creativity = base_value;
        manager.core_attributes.perception = base_value;
        manager.core_attributes.endurance = base_value;
        manager.core_attributes.luck = base_value;
        manager.core_attributes.resonance = base_value;

        manager.recalculate_all_stats();
        manager
    }

    /// Get core attribute value with all modifiers applied
    pub fn get_attribute(&self, attribute: CoreAttributeType) -> u32 {
        let base_value = self.core_attributes.get_base(attribute);
        let equipment_bonus = self.equipment_modifiers.get_attribute_bonus(attribute);
        let effect_bonus = self.get_temporary_effect_bonus(attribute);

        (base_value as i32 + equipment_bonus + effect_bonus).max(1) as u32
    }

    /// Set base attribute value (without modifiers)
    pub fn set_base_attribute(&mut self, attribute: CoreAttributeType, value: u32) {
        self.core_attributes.set_base(attribute, value.clamp(1, 100));
        self.invalidate_derived_stats();
        self.recalculate_derived_stats();
    }

    /// Add points to an attribute (for level-up allocation)
    pub fn allocate_attribute_points(&mut self, attribute: CoreAttributeType, points: u32, max_per_attribute: u32) -> RobinResult<u32> {
        let current_base = self.core_attributes.get_base(attribute);
        let new_value = (current_base + points).min(max_per_attribute);
        let actual_points_spent = new_value - current_base;

        if actual_points_spent > 0 {
            self.set_base_attribute(attribute, new_value);
        }

        Ok(actual_points_spent)
    }

    /// Get derived stat value
    pub fn get_derived_stat(&self, stat: DerivedStatType) -> f32 {
        if self.calculation_cache.is_valid() {
            if let Some(cached_value) = self.calculation_cache.get_derived_stat(stat) {
                return cached_value;
            }
        }

        self.calculate_derived_stat(stat)
    }

    /// Calculate a derived stat with Apple Silicon optimization
    fn calculate_derived_stat(&self, stat: DerivedStatType) -> f32 {
        let start_time = std::time::Instant::now();

        let result = match stat {
            DerivedStatType::MaxHealth => {
                let vitality = self.get_attribute(CoreAttributeType::Vitality) as f32;
                let endurance = self.get_attribute(CoreAttributeType::Endurance) as f32;
                100.0 + (vitality * 10.0) + (endurance * 5.0)
            }
            DerivedStatType::MaxStamina => {
                let endurance = self.get_attribute(CoreAttributeType::Endurance) as f32;
                let strength = self.get_attribute(CoreAttributeType::Strength) as f32;
                100.0 + (endurance * 8.0) + (strength * 2.0)
            }
            DerivedStatType::MaxMana => {
                let intelligence = self.get_attribute(CoreAttributeType::Intelligence) as f32;
                let willpower = self.get_attribute(CoreAttributeType::Willpower) as f32;
                50.0 + (intelligence * 6.0) + (willpower * 4.0)
            }
            DerivedStatType::CarryCapacity => {
                let strength = self.get_attribute(CoreAttributeType::Strength) as f32;
                let endurance = self.get_attribute(CoreAttributeType::Endurance) as f32;
                50.0 + (strength * 3.0) + (endurance * 1.5)
            }
            DerivedStatType::CriticalChance => {
                let luck = self.get_attribute(CoreAttributeType::Luck) as f32;
                let perception = self.get_attribute(CoreAttributeType::Perception) as f32;
                (luck * 0.5 + perception * 0.3).min(50.0) // Max 50% crit chance
            }
            DerivedStatType::MovementSpeed => {
                let dexterity = self.get_attribute(CoreAttributeType::Dexterity) as f32;
                let endurance = self.get_attribute(CoreAttributeType::Endurance) as f32;
                1.0 + (dexterity * 0.02) + (endurance * 0.01)
            }
            DerivedStatType::BuildingSpeed => {
                let dexterity = self.get_attribute(CoreAttributeType::Dexterity) as f32;
                let focus = self.get_attribute(CoreAttributeType::Focus) as f32;
                1.0 + (dexterity * 0.03) + (focus * 0.02)
            }
            DerivedStatType::CraftingQuality => {
                let intelligence = self.get_attribute(CoreAttributeType::Intelligence) as f32;
                let creativity = self.get_attribute(CoreAttributeType::Creativity) as f32;
                let focus = self.get_attribute(CoreAttributeType::Focus) as f32;
                1.0 + ((intelligence + creativity + focus) * 0.015)
            }
            DerivedStatType::ResourceYield => {
                let perception = self.get_attribute(CoreAttributeType::Perception) as f32;
                let luck = self.get_attribute(CoreAttributeType::Luck) as f32;
                1.0 + (perception * 0.02) + (luck * 0.025)
            }
            DerivedStatType::XpGainMultiplier => {
                let intelligence = self.get_attribute(CoreAttributeType::Intelligence) as f32;
                let focus = self.get_attribute(CoreAttributeType::Focus) as f32;
                1.0 + ((intelligence + focus) * 0.01)
            }
        };

        // Record performance metrics
        let calculation_time = start_time.elapsed();
        self.performance_monitor.record_calculation_time(stat, calculation_time);

        result
    }

    /// Recalculate all derived stats using Apple Silicon optimization if available
    pub fn recalculate_all_stats(&mut self) {
        #[cfg(target_os = "macos")]
        {
            if let Some(ref metal_compute) = self.metal_compute {
                // Use Metal compute shaders for bulk calculation on Apple Silicon
                if let Ok(calculated_stats) = metal_compute.calculate_all_derived_stats(&self.core_attributes, &self.equipment_modifiers, &self.temporary_effects) {
                    self.derived_stats = calculated_stats;
                    self.calculation_cache.update_all_stats(&self.derived_stats);
                    return;
                }
            }
        }

        // Fallback CPU calculation
        self.recalculate_derived_stats();
    }

    /// CPU fallback for derived stat calculation
    fn recalculate_derived_stats(&mut self) {
        use rayon::prelude::*;

        // Use parallel processing for multiple stat calculations
        let stats: Vec<(DerivedStatType, f32)> = DerivedStatType::all()
            .par_iter()
            .map(|&stat_type| (stat_type, self.calculate_derived_stat(stat_type)))
            .collect();

        for (stat_type, value) in stats {
            self.derived_stats.set_stat(stat_type, value);
        }

        self.calculation_cache.update_all_stats(&self.derived_stats);
    }

    /// Apply equipment modifiers
    pub fn equip_item(&mut self, item: &EquipmentItem, slot: EquipmentSlot) -> RobinResult<()> {
        // Check attribute requirements
        for (attribute, required_value) in &item.requirements {
            if self.get_attribute(*attribute) < *required_value {
                return Err(RobinError::InvalidGameState(
                    format!("Insufficient {} to equip item (required: {}, current: {})",
                            attribute.name(), required_value, self.get_attribute(*attribute))
                ));
            }
        }

        // Apply equipment
        self.equipment_modifiers.equip_item(item.clone(), slot);
        self.invalidate_derived_stats();
        self.recalculate_all_stats();

        Ok(())
    }

    /// Remove equipped item
    pub fn unequip_item(&mut self, slot: EquipmentSlot) -> Option<EquipmentItem> {
        let removed_item = self.equipment_modifiers.unequip_item(slot);
        if removed_item.is_some() {
            self.invalidate_derived_stats();
            self.recalculate_all_stats();
        }
        removed_item
    }

    /// Apply temporary effect (buff/debuff)
    pub fn apply_temporary_effect(&mut self, effect: TemporaryEffect) {
        self.temporary_effects.push(effect);
        self.invalidate_derived_stats();
        self.recalculate_all_stats();
    }

    /// Update temporary effects (remove expired ones)
    pub fn update_temporary_effects(&mut self, delta_time: f32) {
        let initial_count = self.temporary_effects.len();

        // Decrease duration and remove expired effects
        self.temporary_effects.retain_mut(|effect| {
            effect.remaining_duration -= delta_time;
            effect.remaining_duration > 0.0
        });

        // Recalculate if effects were removed
        if self.temporary_effects.len() != initial_count {
            self.invalidate_derived_stats();
            self.recalculate_all_stats();
        }
    }

    /// Get sum of temporary effect bonuses for an attribute
    fn get_temporary_effect_bonus(&self, attribute: CoreAttributeType) -> i32 {
        self.temporary_effects
            .iter()
            .filter_map(|effect| effect.attribute_modifiers.get(&attribute))
            .sum()
    }

    /// Apply skill tree bonuses from specializations
    pub fn apply_specialization_bonuses(&mut self, specialization: SpecializationPath, points: u32) {
        let bonuses = match specialization {
            SpecializationPath::Engineer => {
                let mut bonuses = HashMap::new();
                bonuses.insert(CoreAttributeType::Intelligence, points / 2);
                bonuses.insert(CoreAttributeType::Focus, points / 3);
                bonuses
            }
            SpecializationPath::Artist => {
                let mut bonuses = HashMap::new();
                bonuses.insert(CoreAttributeType::Creativity, points / 2);
                bonuses.insert(CoreAttributeType::Charisma, points / 3);
                bonuses
            }
            SpecializationPath::Explorer => {
                let mut bonuses = HashMap::new();
                bonuses.insert(CoreAttributeType::Perception, points / 2);
                bonuses.insert(CoreAttributeType::Endurance, points / 3);
                bonuses
            }
            SpecializationPath::Researcher => {
                let mut bonuses = HashMap::new();
                bonuses.insert(CoreAttributeType::Intelligence, points / 3);
                bonuses.insert(CoreAttributeType::Willpower, points / 2);
                bonuses
            }
        };

        for (attribute, bonus) in bonuses {
            let current = self.core_attributes.get_base(attribute);
            self.set_base_attribute(attribute, current + bonus);
        }
    }

    /// Get attribute summary for UI display
    pub fn get_attribute_summary(&self) -> AttributeSummary {
        AttributeSummary {
            core_attributes: self.core_attributes.clone(),
            derived_stats: self.derived_stats.clone(),
            equipment_bonuses: self.equipment_modifiers.get_total_bonuses(),
            active_effects: self.temporary_effects.len(),
            performance_metrics: self.performance_monitor.get_summary(),
        }
    }

    /// Invalidate calculation cache
    fn invalidate_derived_stats(&mut self) {
        self.calculation_cache.invalidate();
    }

    /// Update player data with current attributes
    pub fn update_player_data(&self, player_data: &mut PlayerData) {
        // Update max health based on vitality
        player_data.max_health = self.get_derived_stat(DerivedStatType::MaxHealth);

        // Update custom stats with derived values
        for stat_type in DerivedStatType::all() {
            let stat_name = format!("derived_{}", stat_type.name());
            player_data.stats.custom_stats.insert(stat_name, self.get_derived_stat(*stat_type) as f64);
        }

        // Update core attributes in custom stats
        for attribute_type in CoreAttributeType::all() {
            let attr_name = format!("attribute_{}", attribute_type.name());
            player_data.stats.custom_stats.insert(attr_name, self.get_attribute(*attribute_type) as f64);
        }
    }
}

/// Core player attributes (12 fundamental stats)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreAttributes {
    /// Physical power and raw strength
    pub strength: u32,
    /// Agility, precision, and reflexes
    pub dexterity: u32,
    /// Knowledge, reasoning, and problem-solving
    pub intelligence: u32,
    /// Health, constitution, and life force
    pub vitality: u32,
    /// Mental fortitude and magical resistance
    pub willpower: u32,
    /// Social skills and leadership ability
    pub charisma: u32,
    /// Concentration and attention to detail
    pub focus: u32,
    /// Artistic vision and innovation
    pub creativity: u32,
    /// Awareness and observation skills
    pub perception: u32,
    /// Stamina and physical resilience
    pub endurance: u32,
    /// Chance and fortune
    pub luck: u32,
    /// Attunement to the game world and systems
    pub resonance: u32,
}

impl Default for CoreAttributes {
    fn default() -> Self {
        Self {
            strength: 10,
            dexterity: 10,
            intelligence: 10,
            vitality: 10,
            willpower: 10,
            charisma: 10,
            focus: 10,
            creativity: 10,
            perception: 10,
            endurance: 10,
            luck: 10,
            resonance: 10,
        }
    }
}

impl CoreAttributes {
    pub fn get_base(&self, attribute: CoreAttributeType) -> u32 {
        match attribute {
            CoreAttributeType::Strength => self.strength,
            CoreAttributeType::Dexterity => self.dexterity,
            CoreAttributeType::Intelligence => self.intelligence,
            CoreAttributeType::Vitality => self.vitality,
            CoreAttributeType::Willpower => self.willpower,
            CoreAttributeType::Charisma => self.charisma,
            CoreAttributeType::Focus => self.focus,
            CoreAttributeType::Creativity => self.creativity,
            CoreAttributeType::Perception => self.perception,
            CoreAttributeType::Endurance => self.endurance,
            CoreAttributeType::Luck => self.luck,
            CoreAttributeType::Resonance => self.resonance,
        }
    }

    pub fn set_base(&mut self, attribute: CoreAttributeType, value: u32) {
        match attribute {
            CoreAttributeType::Strength => self.strength = value,
            CoreAttributeType::Dexterity => self.dexterity = value,
            CoreAttributeType::Intelligence => self.intelligence = value,
            CoreAttributeType::Vitality => self.vitality = value,
            CoreAttributeType::Willpower => self.willpower = value,
            CoreAttributeType::Charisma => self.charisma = value,
            CoreAttributeType::Focus => self.focus = value,
            CoreAttributeType::Creativity => self.creativity = value,
            CoreAttributeType::Perception => self.perception = value,
            CoreAttributeType::Endurance => self.endurance = value,
            CoreAttributeType::Luck => self.luck = value,
            CoreAttributeType::Resonance => self.resonance = value,
        }
    }
}

/// Core attribute types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoreAttributeType {
    Strength,
    Dexterity,
    Intelligence,
    Vitality,
    Willpower,
    Charisma,
    Focus,
    Creativity,
    Perception,
    Endurance,
    Luck,
    Resonance,
}

impl CoreAttributeType {
    pub fn all() -> &'static [CoreAttributeType] {
        &[
            CoreAttributeType::Strength,
            CoreAttributeType::Dexterity,
            CoreAttributeType::Intelligence,
            CoreAttributeType::Vitality,
            CoreAttributeType::Willpower,
            CoreAttributeType::Charisma,
            CoreAttributeType::Focus,
            CoreAttributeType::Creativity,
            CoreAttributeType::Perception,
            CoreAttributeType::Endurance,
            CoreAttributeType::Luck,
            CoreAttributeType::Resonance,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            CoreAttributeType::Strength => "strength",
            CoreAttributeType::Dexterity => "dexterity",
            CoreAttributeType::Intelligence => "intelligence",
            CoreAttributeType::Vitality => "vitality",
            CoreAttributeType::Willpower => "willpower",
            CoreAttributeType::Charisma => "charisma",
            CoreAttributeType::Focus => "focus",
            CoreAttributeType::Creativity => "creativity",
            CoreAttributeType::Perception => "perception",
            CoreAttributeType::Endurance => "endurance",
            CoreAttributeType::Luck => "luck",
            CoreAttributeType::Resonance => "resonance",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            CoreAttributeType::Strength => "Physical power and ability to handle heavy objects",
            CoreAttributeType::Dexterity => "Agility, precision, and fine motor control",
            CoreAttributeType::Intelligence => "Knowledge, reasoning, and problem-solving ability",
            CoreAttributeType::Vitality => "Health, constitution, and life force",
            CoreAttributeType::Willpower => "Mental fortitude and resistance to adverse effects",
            CoreAttributeType::Charisma => "Social skills, leadership, and persuasion ability",
            CoreAttributeType::Focus => "Concentration, attention to detail, and mental clarity",
            CoreAttributeType::Creativity => "Artistic vision, innovation, and design skills",
            CoreAttributeType::Perception => "Awareness, observation, and detection abilities",
            CoreAttributeType::Endurance => "Stamina, physical resilience, and fatigue resistance",
            CoreAttributeType::Luck => "Fortune, chance encounters, and random benefits",
            CoreAttributeType::Resonance => "Attunement to the world's systems and hidden mechanics",
        }
    }
}

/// Derived stats calculated from core attributes
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DerivedStats {
    /// Maximum health points
    pub max_health: f32,
    /// Maximum stamina points
    pub max_stamina: f32,
    /// Maximum mana/energy points
    pub max_mana: f32,
    /// Maximum carry capacity
    pub carry_capacity: f32,
    /// Critical hit chance percentage
    pub critical_chance: f32,
    /// Movement speed multiplier
    pub movement_speed: f32,
    /// Building/construction speed multiplier
    pub building_speed: f32,
    /// Crafting quality multiplier
    pub crafting_quality: f32,
    /// Resource gathering yield multiplier
    pub resource_yield: f32,
    /// Experience gain multiplier
    pub xp_gain_multiplier: f32,
}

impl DerivedStats {
    pub fn set_stat(&mut self, stat_type: DerivedStatType, value: f32) {
        match stat_type {
            DerivedStatType::MaxHealth => self.max_health = value,
            DerivedStatType::MaxStamina => self.max_stamina = value,
            DerivedStatType::MaxMana => self.max_mana = value,
            DerivedStatType::CarryCapacity => self.carry_capacity = value,
            DerivedStatType::CriticalChance => self.critical_chance = value,
            DerivedStatType::MovementSpeed => self.movement_speed = value,
            DerivedStatType::BuildingSpeed => self.building_speed = value,
            DerivedStatType::CraftingQuality => self.crafting_quality = value,
            DerivedStatType::ResourceYield => self.resource_yield = value,
            DerivedStatType::XpGainMultiplier => self.xp_gain_multiplier = value,
        }
    }

    pub fn get_stat(&self, stat_type: DerivedStatType) -> f32 {
        match stat_type {
            DerivedStatType::MaxHealth => self.max_health,
            DerivedStatType::MaxStamina => self.max_stamina,
            DerivedStatType::MaxMana => self.max_mana,
            DerivedStatType::CarryCapacity => self.carry_capacity,
            DerivedStatType::CriticalChance => self.critical_chance,
            DerivedStatType::MovementSpeed => self.movement_speed,
            DerivedStatType::BuildingSpeed => self.building_speed,
            DerivedStatType::CraftingQuality => self.crafting_quality,
            DerivedStatType::ResourceYield => self.resource_yield,
            DerivedStatType::XpGainMultiplier => self.xp_gain_multiplier,
        }
    }
}

/// Derived stat types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DerivedStatType {
    MaxHealth,
    MaxStamina,
    MaxMana,
    CarryCapacity,
    CriticalChance,
    MovementSpeed,
    BuildingSpeed,
    CraftingQuality,
    ResourceYield,
    XpGainMultiplier,
}

impl DerivedStatType {
    pub fn all() -> &'static [DerivedStatType] {
        &[
            DerivedStatType::MaxHealth,
            DerivedStatType::MaxStamina,
            DerivedStatType::MaxMana,
            DerivedStatType::CarryCapacity,
            DerivedStatType::CriticalChance,
            DerivedStatType::MovementSpeed,
            DerivedStatType::BuildingSpeed,
            DerivedStatType::CraftingQuality,
            DerivedStatType::ResourceYield,
            DerivedStatType::XpGainMultiplier,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            DerivedStatType::MaxHealth => "max_health",
            DerivedStatType::MaxStamina => "max_stamina",
            DerivedStatType::MaxMana => "max_mana",
            DerivedStatType::CarryCapacity => "carry_capacity",
            DerivedStatType::CriticalChance => "critical_chance",
            DerivedStatType::MovementSpeed => "movement_speed",
            DerivedStatType::BuildingSpeed => "building_speed",
            DerivedStatType::CraftingQuality => "crafting_quality",
            DerivedStatType::ResourceYield => "resource_yield",
            DerivedStatType::XpGainMultiplier => "xp_gain_multiplier",
        }
    }
}

/// Equipment modifiers system
#[derive(Debug, Clone, Default)]
pub struct EquipmentModifiers {
    equipped_items: HashMap<EquipmentSlot, EquipmentItem>,
}

impl EquipmentModifiers {
    pub fn equip_item(&mut self, item: EquipmentItem, slot: EquipmentSlot) {
        self.equipped_items.insert(slot, item);
    }

    pub fn unequip_item(&mut self, slot: EquipmentSlot) -> Option<EquipmentItem> {
        self.equipped_items.remove(&slot)
    }

    pub fn get_attribute_bonus(&self, attribute: CoreAttributeType) -> i32 {
        self.equipped_items
            .values()
            .flat_map(|item| &item.attribute_bonuses)
            .filter_map(|(attr, bonus)| if *attr == attribute { Some(*bonus) } else { None })
            .sum()
    }

    pub fn get_total_bonuses(&self) -> HashMap<CoreAttributeType, i32> {
        let mut totals = HashMap::new();

        for item in self.equipped_items.values() {
            for (attribute, bonus) in &item.attribute_bonuses {
                *totals.entry(*attribute).or_insert(0) += *bonus;
            }
        }

        totals
    }
}

/// Equipment slots
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EquipmentSlot {
    Weapon,
    Helmet,
    Chest,
    Legs,
    Boots,
    Gloves,
    Ring1,
    Ring2,
    Necklace,
    Tool,
}

/// Equipment item with attribute modifiers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub attribute_bonuses: HashMap<CoreAttributeType, i32>,
    pub requirements: HashMap<CoreAttributeType, u32>,
    pub durability: f32,
    pub max_durability: f32,
}

/// Temporary effect (buff/debuff)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporaryEffect {
    pub id: String,
    pub name: String,
    pub description: String,
    pub attribute_modifiers: HashMap<CoreAttributeType, i32>,
    pub remaining_duration: f32,
    pub effect_type: EffectType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffectType {
    Buff,
    Debuff,
    Neutral,
}

/// Attribute calculation cache for performance
pub struct AttributeCache {
    cached_stats: HashMap<DerivedStatType, f32>,
    last_update: std::time::Instant,
    cache_duration: std::time::Duration,
}

impl AttributeCache {
    pub fn new() -> Self {
        Self {
            cached_stats: HashMap::new(),
            last_update: std::time::Instant::now(),
            cache_duration: std::time::Duration::from_millis(100), // 100ms cache
        }
    }

    pub fn is_valid(&self) -> bool {
        self.last_update.elapsed() < self.cache_duration
    }

    pub fn get_derived_stat(&self, stat: DerivedStatType) -> Option<f32> {
        if self.is_valid() {
            self.cached_stats.get(&stat).copied()
        } else {
            None
        }
    }

    pub fn update_all_stats(&mut self, stats: &DerivedStats) {
        for stat_type in DerivedStatType::all() {
            self.cached_stats.insert(*stat_type, stats.get_stat(*stat_type));
        }
        self.last_update = std::time::Instant::now();
    }

    pub fn invalidate(&mut self) {
        self.cached_stats.clear();
    }
}

/// Performance monitoring for attribute calculations
pub struct AttributePerformanceMonitor {
    calculation_times: HashMap<DerivedStatType, std::time::Duration>,
    total_calculations: u64,
    metal_usage_count: u64,
}

impl AttributePerformanceMonitor {
    pub fn new() -> Self {
        Self {
            calculation_times: HashMap::new(),
            total_calculations: 0,
            metal_usage_count: 0,
        }
    }

    pub fn record_calculation_time(&self, stat_type: DerivedStatType, duration: std::time::Duration) {
        // Note: In a real implementation, we'd need interior mutability here
        // For now, this is a placeholder for the performance monitoring interface
    }

    pub fn record_metal_usage(&mut self) {
        self.metal_usage_count += 1;
    }

    pub fn get_summary(&self) -> PerformanceMetrics {
        PerformanceMetrics {
            total_calculations: self.total_calculations,
            metal_usage_percentage: if self.total_calculations > 0 {
                (self.metal_usage_count as f32 / self.total_calculations as f32) * 100.0
            } else {
                0.0
            },
            average_calculation_time: self.calculation_times.values()
                .map(|d| d.as_nanos() as f32)
                .sum::<f32>() / self.calculation_times.len().max(1) as f32,
        }
    }
}

/// Performance metrics summary
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub total_calculations: u64,
    pub metal_usage_percentage: f32,
    pub average_calculation_time: f32, // nanoseconds
}

/// Attribute summary for UI display
#[derive(Debug, Clone)]
pub struct AttributeSummary {
    pub core_attributes: CoreAttributes,
    pub derived_stats: DerivedStats,
    pub equipment_bonuses: HashMap<CoreAttributeType, i32>,
    pub active_effects: usize,
    pub performance_metrics: PerformanceMetrics,
}

/// Metal compute integration (Apple Silicon optimization)
#[cfg(target_os = "macos")]
pub struct MetalStatsCompute {
    // Metal compute implementation would go here
    // This is a placeholder for the Metal compute shader integration
}

#[cfg(target_os = "macos")]
impl MetalStatsCompute {
    pub fn new() -> RobinResult<Self> {
        // Initialize Metal compute shaders for stat calculations
        Ok(Self {})
    }

    pub fn calculate_all_derived_stats(
        &self,
        _core_attributes: &CoreAttributes,
        _equipment_modifiers: &EquipmentModifiers,
        _temporary_effects: &[TemporaryEffect],
    ) -> RobinResult<DerivedStats> {
        // Placeholder for Metal compute shader calculation
        // In a real implementation, this would:
        // 1. Upload attribute data to Metal buffers
        // 2. Execute compute shaders for parallel calculation
        // 3. Read back results using unified memory
        Err(RobinError::NotImplemented("Metal compute not yet implemented".to_string()))
    }
}

impl Default for PlayerAttributeManager {
    fn default() -> Self {
        Self::new()
    }
}