// Advanced gameplay mechanics and player progression systems
// Built on optimized Robin Engine foundation with material batching and performance monitoring

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use crate::material_batching::MaterialType;

/// Player skill progression system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerProgression {
    pub level: u32,
    pub experience: u64,
    pub experience_to_next_level: u64,
    pub skill_points: u32,
    pub skills: HashMap<SkillType, SkillLevel>,
    pub achievements: Vec<Achievement>,
    pub play_time: Duration,
    pub blocks_placed: u64,
    pub blocks_destroyed: u64,
    pub structures_built: u32,
}

impl PlayerProgression {
    pub fn new() -> Self {
        let mut skills = HashMap::new();
        skills.insert(SkillType::Building, SkillLevel::new());
        skills.insert(SkillType::Engineering, SkillLevel::new());
        skills.insert(SkillType::Architecture, SkillLevel::new());
        skills.insert(SkillType::MaterialMastery, SkillLevel::new());
        skills.insert(SkillType::Efficiency, SkillLevel::new());

        Self {
            level: 1,
            experience: 0,
            experience_to_next_level: 100,
            skill_points: 0,
            skills,
            achievements: Vec::new(),
            play_time: Duration::ZERO,
            blocks_placed: 0,
            blocks_destroyed: 0,
            structures_built: 0,
        }
    }

    pub fn add_experience(&mut self, amount: u64, source: ExperienceSource) {
        self.experience += amount;

        // Check for level up
        while self.experience >= self.experience_to_next_level {
            self.level_up();
        }

        // Award skill-specific experience
        match source {
            ExperienceSource::BlockPlaced(material) => {
                self.blocks_placed += 1;
                self.add_skill_experience(SkillType::Building, amount / 2);
                self.add_material_experience(material, amount / 4);
            }
            ExperienceSource::BlockDestroyed => {
                self.blocks_destroyed += 1;
                self.add_skill_experience(SkillType::Engineering, amount / 3);
            }
            ExperienceSource::StructureCompleted(size) => {
                self.structures_built += 1;
                self.add_skill_experience(SkillType::Architecture, amount);
                if size > 100 {
                    self.add_skill_experience(SkillType::Efficiency, amount / 2);
                }
            }
            ExperienceSource::TemplateUsed => {
                self.add_skill_experience(SkillType::Engineering, amount);
            }
            ExperienceSource::EfficientBuilding => {
                self.add_skill_experience(SkillType::Efficiency, amount);
            }
        }

        // Check for new achievements
        self.check_achievements();
    }

    fn level_up(&mut self) {
        self.level += 1;
        self.skill_points += 3;
        self.experience_to_next_level = self.calculate_next_level_xp();

        log::info!("🎉 Level up! Now level {}", self.level);
    }

    fn calculate_next_level_xp(&self) -> u64 {
        // Exponential progression with some linear component
        (self.level as u64 * 100) + ((self.level as u64).pow(2) * 25)
    }

    fn add_skill_experience(&mut self, skill: SkillType, amount: u64) {
        if let Some(skill_level) = self.skills.get_mut(&skill) {
            skill_level.add_experience(amount);
        }
    }

    fn add_material_experience(&mut self, material: MaterialType, amount: u64) {
        // Material mastery increases as you use different materials
        if let Some(mastery) = self.skills.get_mut(&SkillType::MaterialMastery) {
            mastery.add_experience(amount);
        }
    }

    fn check_achievements(&mut self) {
        let mut new_achievements = Vec::new();

        // Building achievements
        if self.blocks_placed >= 100 && !self.has_achievement("first_century") {
            new_achievements.push(Achievement::new("first_century", "First Century", "Place 100 blocks"));
        }
        if self.blocks_placed >= 1000 && !self.has_achievement("master_builder") {
            new_achievements.push(Achievement::new("master_builder", "Master Builder", "Place 1000 blocks"));
        }

        // Skill achievements
        for (skill_type, skill_level) in &self.skills {
            if skill_level.level >= 5 && !self.has_achievement(&format!("{:?}_adept", skill_type)) {
                new_achievements.push(Achievement::new(
                    &format!("{:?}_adept", skill_type),
                    &format!("{:?} Adept", skill_type),
                    &format!("Reach level 5 in {:?}", skill_type)
                ));
            }
        }

        // Structure achievements
        if self.structures_built >= 10 && !self.has_achievement("architect") {
            new_achievements.push(Achievement::new("architect", "Architect", "Complete 10 structures"));
        }

        for achievement in new_achievements {
            log::info!("🏆 Achievement unlocked: {}", achievement.name);
            self.achievements.push(achievement);
        }
    }

    fn has_achievement(&self, id: &str) -> bool {
        self.achievements.iter().any(|a| a.id == id)
    }

    pub fn get_skill_level(&self, skill: &SkillType) -> u32 {
        self.skills.get(skill).map(|s| s.level).unwrap_or(0)
    }

    pub fn get_building_efficiency_bonus(&self) -> f32 {
        let efficiency_level = self.get_skill_level(&SkillType::Efficiency) as f32;
        let building_level = self.get_skill_level(&SkillType::Building) as f32;

        // Efficiency bonus: faster building, less resource consumption
        1.0 + (efficiency_level * 0.1) + (building_level * 0.05)
    }

    pub fn get_material_cost_reduction(&self) -> f32 {
        let mastery_level = self.get_skill_level(&SkillType::MaterialMastery) as f32;

        // Higher mastery = less material waste
        mastery_level * 0.02 // Up to 20% reduction at max level
    }
}

/// Different types of player skills
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillType {
    Building,        // Speed and accuracy of block placement
    Engineering,     // Advanced templates and complex structures
    Architecture,    // Design efficiency and aesthetic bonuses
    MaterialMastery, // Reduced waste and material efficiency
    Efficiency,      // Overall productivity and automation
}

/// Individual skill progression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillLevel {
    pub level: u32,
    pub experience: u64,
    pub experience_to_next: u64,
}

impl SkillLevel {
    pub fn new() -> Self {
        Self {
            level: 0,
            experience: 0,
            experience_to_next: 50,
        }
    }

    pub fn add_experience(&mut self, amount: u64) {
        self.experience += amount;

        while self.experience >= self.experience_to_next {
            self.level += 1;
            self.experience -= self.experience_to_next;
            self.experience_to_next = self.calculate_next_level_xp();
        }
    }

    fn calculate_next_level_xp(&self) -> u64 {
        50 + (self.level as u64 * 25)
    }
}

/// Sources of experience for different activities
#[derive(Debug, Clone)]
pub enum ExperienceSource {
    BlockPlaced(MaterialType),
    BlockDestroyed,
    StructureCompleted(u32), // Size of structure
    TemplateUsed,
    EfficientBuilding,
}

/// Player achievements system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(skip, default = "std::time::Instant::now")]
    pub unlocked_at: Instant,
    pub rarity: AchievementRarity,
}

impl Achievement {
    pub fn new(id: &str, name: &str, description: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            unlocked_at: Instant::now(),
            rarity: AchievementRarity::Common,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AchievementRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

/// Advanced building mechanics
#[derive(Debug, Clone)]
pub struct BuildingMechanics {
    pub blueprint_system: BlueprintSystem,
    pub construction_queue: ConstructionQueue,
    pub resource_manager: ResourceManager,
    pub automation_tools: AutomationTools,
}

impl BuildingMechanics {
    pub fn new() -> Self {
        Self {
            blueprint_system: BlueprintSystem::new(),
            construction_queue: ConstructionQueue::new(),
            resource_manager: ResourceManager::new(),
            automation_tools: AutomationTools::new(),
        }
    }

    pub fn create_blueprint(&mut self, name: String, blocks: Vec<BlueprintBlock>) -> BlueprintId {
        self.blueprint_system.create_blueprint(name, blocks)
    }

    pub fn queue_construction(&mut self, blueprint_id: BlueprintId, position: [f32; 3]) {
        if let Some(blueprint) = self.blueprint_system.get_blueprint(blueprint_id) {
            let job = ConstructionJob::new(blueprint.clone(), position);
            self.construction_queue.add_job(job);
        }
    }

    pub fn update(&mut self, delta_time: f32, player_progression: &PlayerProgression) {
        let efficiency_bonus = player_progression.get_building_efficiency_bonus();
        self.construction_queue.update(delta_time * efficiency_bonus);
        self.automation_tools.update(delta_time);
    }
}

/// Blueprint system for saving and reusing designs
#[derive(Debug, Clone)]
pub struct BlueprintSystem {
    blueprints: HashMap<BlueprintId, Blueprint>,
    next_id: u32,
}

impl BlueprintSystem {
    pub fn new() -> Self {
        Self {
            blueprints: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn create_blueprint(&mut self, name: String, blocks: Vec<BlueprintBlock>) -> BlueprintId {
        let id = BlueprintId(self.next_id);
        self.next_id += 1;

        let blueprint = Blueprint {
            id,
            name,
            blocks,
            creation_time: Instant::now(),
            usage_count: 0,
        };

        self.blueprints.insert(id, blueprint);
        id
    }

    pub fn get_blueprint(&self, id: BlueprintId) -> Option<&Blueprint> {
        self.blueprints.get(&id)
    }

    pub fn list_blueprints(&self) -> Vec<&Blueprint> {
        self.blueprints.values().collect()
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BlueprintId(u32);

#[derive(Debug, Clone)]
pub struct Blueprint {
    pub id: BlueprintId,
    pub name: String,
    pub blocks: Vec<BlueprintBlock>,
    pub creation_time: Instant,
    pub usage_count: u32,
}

#[derive(Debug, Clone)]
pub struct BlueprintBlock {
    pub position: [i32; 3],
    pub material: MaterialType,
}

/// Construction queue for automated building
#[derive(Debug, Clone)]
pub struct ConstructionQueue {
    jobs: VecDeque<ConstructionJob>,
    current_job: Option<ConstructionJob>,
    construction_speed: f32, // blocks per second
}

impl ConstructionQueue {
    pub fn new() -> Self {
        Self {
            jobs: VecDeque::new(),
            current_job: None,
            construction_speed: 2.0, // 2 blocks per second base speed
        }
    }

    pub fn add_job(&mut self, job: ConstructionJob) {
        self.jobs.push_back(job);
    }

    pub fn update(&mut self, delta_time: f32) {
        // Process current job
        if let Some(ref mut job) = self.current_job {
            job.progress += delta_time * self.construction_speed;

            if job.progress >= job.total_blocks as f32 {
                // Job completed
                log::info!("🏗️ Construction completed: {}", job.blueprint.name);
                self.current_job = None;
            }
        }

        // Start next job if available
        if self.current_job.is_none() {
            if let Some(job) = self.jobs.pop_front() {
                log::info!("🏗️ Starting construction: {}", job.blueprint.name);
                self.current_job = Some(job);
            }
        }
    }

    pub fn get_progress(&self) -> Option<f32> {
        self.current_job.as_ref().map(|job| {
            job.progress / job.total_blocks as f32
        })
    }
}

#[derive(Debug, Clone)]
pub struct ConstructionJob {
    pub blueprint: Blueprint,
    pub position: [f32; 3],
    pub progress: f32,
    pub total_blocks: u32,
}

impl ConstructionJob {
    pub fn new(blueprint: Blueprint, position: [f32; 3]) -> Self {
        let total_blocks = blueprint.blocks.len() as u32;
        Self {
            blueprint,
            position,
            progress: 0.0,
            total_blocks,
        }
    }
}

/// Resource management system
#[derive(Debug, Clone)]
pub struct ResourceManager {
    pub inventory: HashMap<MaterialType, u32>,
    pub storage_capacity: u32,
    pub auto_collect: bool,
}

impl ResourceManager {
    pub fn new() -> Self {
        let mut inventory = HashMap::new();
        inventory.insert(MaterialType::Stone, 1000);
        inventory.insert(MaterialType::Earth, 1000);
        inventory.insert(MaterialType::Wood, 500);

        Self {
            inventory,
            storage_capacity: 10000,
            auto_collect: false,
        }
    }

    pub fn has_materials(&self, material: MaterialType, amount: u32) -> bool {
        self.inventory.get(&material).unwrap_or(&0) >= &amount
    }

    pub fn consume_materials(&mut self, material: MaterialType, amount: u32) -> bool {
        if let Some(current) = self.inventory.get_mut(&material) {
            if *current >= amount {
                *current -= amount;
                return true;
            }
        }
        false
    }

    pub fn add_materials(&mut self, material: MaterialType, amount: u32) {
        let current = self.inventory.entry(material).or_insert(0);
        *current = (*current + amount).min(self.storage_capacity);
    }

    pub fn get_total_items(&self) -> u32 {
        self.inventory.values().sum()
    }

    pub fn get_storage_usage(&self) -> f32 {
        self.get_total_items() as f32 / self.storage_capacity as f32
    }
}

/// Automation tools for advanced players
#[derive(Debug, Clone)]
pub struct AutomationTools {
    pub auto_builders: Vec<AutoBuilder>,
    pub pattern_repeaters: Vec<PatternRepeater>,
    pub smart_fill: SmartFill,
}

impl AutomationTools {
    pub fn new() -> Self {
        Self {
            auto_builders: Vec::new(),
            pattern_repeaters: Vec::new(),
            smart_fill: SmartFill::new(),
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        for builder in &mut self.auto_builders {
            builder.update(delta_time);
        }

        for repeater in &mut self.pattern_repeaters {
            repeater.update(delta_time);
        }
    }
}

#[derive(Debug, Clone)]
pub struct AutoBuilder {
    pub position: [f32; 3],
    pub range: f32,
    pub blueprint: Blueprint,
    pub active: bool,
    pub speed: f32,
}

impl AutoBuilder {
    pub fn update(&mut self, delta_time: f32) {
        if self.active {
            // Auto-building logic would go here
        }
    }
}

#[derive(Debug, Clone)]
pub struct PatternRepeater {
    pub pattern: Vec<BlueprintBlock>,
    pub direction: [f32; 3],
    pub repetitions: u32,
    pub current_rep: u32,
}

impl PatternRepeater {
    pub fn update(&mut self, delta_time: f32) {
        // Pattern repetition logic
    }
}

#[derive(Debug, Clone)]
pub struct SmartFill {
    pub enabled: bool,
    pub fill_material: MaterialType,
    pub fill_mode: FillMode,
}

impl SmartFill {
    pub fn new() -> Self {
        Self {
            enabled: false,
            fill_material: MaterialType::Stone,
            fill_mode: FillMode::Solid,
        }
    }
}

#[derive(Debug, Clone)]
pub enum FillMode {
    Solid,
    Hollow,
    Pattern(Vec<MaterialType>),
}

/// Main gameplay systems manager
#[derive(Debug)]
pub struct GameplayManager {
    pub player_progression: PlayerProgression,
    pub building_mechanics: BuildingMechanics,
    pub session_stats: SessionStats,
}

impl GameplayManager {
    pub fn new() -> Self {
        Self {
            player_progression: PlayerProgression::new(),
            building_mechanics: BuildingMechanics::new(),
            session_stats: SessionStats::new(),
        }
    }

    pub fn on_block_placed(&mut self, material: MaterialType) {
        // Award experience
        let base_xp = match material {
            MaterialType::Stone => 2,
            MaterialType::Wood => 3,
            MaterialType::Crystal => 10,
            MaterialType::Lava => 15,
            _ => 1,
        };

        self.player_progression.add_experience(base_xp, ExperienceSource::BlockPlaced(material));
        self.session_stats.blocks_placed += 1;
    }

    pub fn on_block_destroyed(&mut self) {
        self.player_progression.add_experience(1, ExperienceSource::BlockDestroyed);
        self.session_stats.blocks_destroyed += 1;
    }

    pub fn on_structure_completed(&mut self, size: u32) {
        let xp = size * 5; // 5 XP per block in completed structure
        self.player_progression.add_experience(xp as u64, ExperienceSource::StructureCompleted(size));
        self.session_stats.structures_completed += 1;
    }

    pub fn update(&mut self, delta_time: f32) {
        self.building_mechanics.update(delta_time, &self.player_progression);
        self.session_stats.play_time += Duration::from_secs_f32(delta_time);
    }

    pub fn get_session_summary(&self) -> String {
        format!(
            "Session Stats - Level: {} | XP: {} | Blocks: {} | Time: {:.1}min",
            self.player_progression.level,
            self.player_progression.experience,
            self.session_stats.blocks_placed,
            self.session_stats.play_time.as_secs_f32() / 60.0
        )
    }
}

/// Session statistics tracking
#[derive(Debug)]
pub struct SessionStats {
    pub play_time: Duration,
    pub blocks_placed: u32,
    pub blocks_destroyed: u32,
    pub structures_completed: u32,
    pub blueprints_created: u32,
    pub session_start: Instant,
}

impl SessionStats {
    pub fn new() -> Self {
        Self {
            play_time: Duration::ZERO,
            blocks_placed: 0,
            blocks_destroyed: 0,
            structures_completed: 0,
            blueprints_created: 0,
            session_start: Instant::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_progression() {
        let mut progression = PlayerProgression::new();

        // Test experience gain
        progression.add_experience(50, ExperienceSource::BlockPlaced(MaterialType::Stone));
        assert_eq!(progression.experience, 50);

        // Test level up
        progression.add_experience(100, ExperienceSource::StructureCompleted(10));
        assert_eq!(progression.level, 2);
        assert!(progression.skill_points > 0);
    }

    #[test]
    fn test_skill_progression() {
        let mut skill = SkillLevel::new();
        skill.add_experience(100);
        assert!(skill.level > 0);
    }

    #[test]
    fn test_blueprint_system() {
        let mut system = BlueprintSystem::new();
        let blocks = vec![
            BlueprintBlock { position: [0, 0, 0], material: MaterialType::Stone },
            BlueprintBlock { position: [1, 0, 0], material: MaterialType::Stone },
        ];

        let id = system.create_blueprint("Test House".to_string(), blocks);
        let blueprint = system.get_blueprint(id).unwrap();
        assert_eq!(blueprint.name, "Test House");
        assert_eq!(blueprint.blocks.len(), 2);
    }

    #[test]
    fn test_resource_manager() {
        let mut manager = ResourceManager::new();
        assert!(manager.has_materials(MaterialType::Stone, 100));
        assert!(manager.consume_materials(MaterialType::Stone, 100));
        assert_eq!(manager.inventory[&MaterialType::Stone], 900);
    }
}