/*!
 * Quest System for Robin Engine
 *
 * Provides guided building challenges, tutorials, and structured gameplay objectives
 * to help players learn and master the Engineer Build Mode and voxel construction.
 */

use crate::engine::{
    error::{RobinError, RobinResult},
    save_system::PlayerData,
    world::VoxelType,
    gameplay::{BuildingSkill, SessionStats},
};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, Duration};
use crate::engine::math::Vec3;

/// Quest management system
pub struct QuestManager {
    /// All available quests
    quests: HashMap<String, Quest>,
    /// Active quests for the current player
    active_quests: HashMap<String, QuestProgress>,
    /// Completed quest IDs
    completed_quests: HashSet<String>,
    /// Quest chains/storylines
    quest_chains: HashMap<String, QuestChain>,
    /// Daily challenge quests
    daily_challenges: Vec<String>,
    /// Last daily refresh time
    last_daily_refresh: SystemTime,
}

impl QuestManager {
    pub fn new() -> Self {
        let mut manager = Self {
            quests: HashMap::new(),
            active_quests: HashMap::new(),
            completed_quests: HashSet::new(),
            quest_chains: HashMap::new(),
            daily_challenges: Vec::new(),
            last_daily_refresh: SystemTime::now(),
        };

        manager.initialize_quests();
        manager.initialize_quest_chains();
        manager
    }

    /// Initialize all quest definitions
    fn initialize_quests(&mut self) {
        // Tutorial Quests
        self.add_quest("tutorial_first_block", Quest {
            name: "Your First Creation".to_string(),
            description: "Learn the basics of placing voxel blocks. Place 5 blocks to complete.".to_string(),
            quest_type: QuestType::Tutorial,
            objectives: vec![
                QuestObjective::PlaceBlocks {
                    block_type: None,
                    count: 5,
                    location: None
                },
            ],
            rewards: QuestRewards {
                experience: vec![(BuildingSkill::Construction, 100)],
                items: vec![
                    ("resource_stone".to_string(), 20),
                    ("resource_wood".to_string(), 20),
                ],
                unlock_content: vec![],
                currency: 50,
            },
            prerequisites: vec![],
            time_limit: None,
            repeatable: false,
            difficulty: QuestDifficulty::Beginner,
        });

        self.add_quest("tutorial_mining", Quest {
            name: "Mining Basics".to_string(),
            description: "Learn how to mine and collect resources. Mine 10 blocks.".to_string(),
            quest_type: QuestType::Tutorial,
            objectives: vec![
                QuestObjective::MineBlocks {
                    block_type: None,
                    count: 10
                },
            ],
            rewards: QuestRewards {
                experience: vec![(BuildingSkill::Mining, 150)],
                items: vec![("tool_basic_pickaxe".to_string(), 1)],
                unlock_content: vec!["mining_techniques".to_string()],
                currency: 75,
            },
            prerequisites: vec!["tutorial_first_block".to_string()],
            time_limit: None,
            repeatable: false,
            difficulty: QuestDifficulty::Beginner,
        });

        self.add_quest("tutorial_first_structure", Quest {
            name: "Building Your First Home".to_string(),
            description: "Construct a simple 5x5x3 shelter using any materials.".to_string(),
            quest_type: QuestType::Tutorial,
            objectives: vec![
                QuestObjective::BuildStructure {
                    structure_type: "shelter".to_string(),
                    min_size: Vec3::new(5.0, 3.0, 5.0),
                    requirements: vec![
                        StructureRequirement::HasWalls,
                        StructureRequirement::HasRoof,
                        StructureRequirement::HasDoor,
                    ]
                },
            ],
            rewards: QuestRewards {
                experience: vec![
                    (BuildingSkill::Construction, 300),
                    (BuildingSkill::Architecture, 200),
                ],
                items: vec![
                    ("blueprint_basic_house".to_string(), 1),
                    ("decoration_torch".to_string(), 5),
                ],
                unlock_content: vec!["advanced_building".to_string()],
                currency: 150,
            },
            prerequisites: vec!["tutorial_mining".to_string()],
            time_limit: None,
            repeatable: false,
            difficulty: QuestDifficulty::Beginner,
        });

        // Main Story Quests
        self.add_quest("story_engineers_workshop", Quest {
            name: "The Engineer's Workshop".to_string(),
            description: "Build a functional workshop with crafting stations and storage.".to_string(),
            quest_type: QuestType::Story,
            objectives: vec![
                QuestObjective::BuildStructure {
                    structure_type: "workshop".to_string(),
                    min_size: Vec3::new(10.0, 4.0, 10.0),
                    requirements: vec![
                        StructureRequirement::HasWalls,
                        StructureRequirement::HasRoof,
                        StructureRequirement::HasLighting,
                    ]
                },
                QuestObjective::PlaceSpecificBlocks {
                    blocks: vec![
                        ("crafting_table".to_string(), 2),
                        ("storage_chest".to_string(), 4),
                        ("furnace".to_string(), 1),
                    ]
                },
            ],
            rewards: QuestRewards {
                experience: vec![
                    (BuildingSkill::Engineering, 500),
                    (BuildingSkill::Crafting, 300),
                ],
                items: vec![
                    ("tool_advanced_hammer".to_string(), 1),
                    ("blueprint_automation".to_string(), 1),
                ],
                unlock_content: vec!["engineer_build_mode_advanced".to_string()],
                currency: 500,
            },
            prerequisites: vec!["tutorial_first_structure".to_string()],
            time_limit: None,
            repeatable: false,
            difficulty: QuestDifficulty::Normal,
        });

        self.add_quest("story_bridge_builder", Quest {
            name: "Bridging the Gap".to_string(),
            description: "Construct a bridge spanning at least 20 blocks to connect two areas.".to_string(),
            quest_type: QuestType::Story,
            objectives: vec![
                QuestObjective::BuildBridge {
                    min_length: 20,
                    min_height: 5,
                    requirements: vec![
                        BridgeRequirement::HasSupports,
                        BridgeRequirement::IsWalkable,
                        BridgeRequirement::ConnectsTwoPoints,
                    ]
                },
            ],
            rewards: QuestRewards {
                experience: vec![
                    (BuildingSkill::Engineering, 800),
                    (BuildingSkill::Architecture, 600),
                ],
                items: vec![
                    ("material_reinforced_beam".to_string(), 50),
                    ("blueprint_suspension_bridge".to_string(), 1),
                ],
                unlock_content: vec!["advanced_engineering".to_string()],
                currency: 750,
            },
            prerequisites: vec!["story_engineers_workshop".to_string()],
            time_limit: None,
            repeatable: false,
            difficulty: QuestDifficulty::Normal,
        });

        // Building Challenges
        self.add_quest("challenge_tower_master", Quest {
            name: "Tower to the Sky".to_string(),
            description: "Build the tallest tower you can! Minimum height: 50 blocks.".to_string(),
            quest_type: QuestType::Challenge,
            objectives: vec![
                QuestObjective::BuildTower {
                    min_height: 50,
                    max_base_size: 10,
                    requirements: vec![
                        TowerRequirement::IsStable,
                        TowerRequirement::HasStairs,
                        TowerRequirement::HasWindows,
                    ]
                },
            ],
            rewards: QuestRewards {
                experience: vec![
                    (BuildingSkill::Construction, 1000),
                    (BuildingSkill::Architecture, 800),
                ],
                items: vec![
                    ("trophy_tower_master".to_string(), 1),
                    ("material_sky_stone".to_string(), 100),
                ],
                unlock_content: vec!["vertical_construction_mastery".to_string()],
                currency: 1000,
            },
            prerequisites: vec!["story_bridge_builder".to_string()],
            time_limit: Some(Duration::from_secs(3600)), // 1 hour time limit
            repeatable: true,
            difficulty: QuestDifficulty::Hard,
        });

        self.add_quest("challenge_speed_builder", Quest {
            name: "Speed Building Challenge".to_string(),
            description: "Replicate the shown structure as quickly as possible!".to_string(),
            quest_type: QuestType::Challenge,
            objectives: vec![
                QuestObjective::ReplicateStructure {
                    template_id: "speed_template_01".to_string(),
                    time_limit: Duration::from_secs(300), // 5 minutes
                    accuracy_threshold: 0.9, // 90% accuracy required
                },
            ],
            rewards: QuestRewards {
                experience: vec![(BuildingSkill::Construction, 600)],
                items: vec![("boost_speed_building".to_string(), 3)],
                unlock_content: vec![],
                currency: 400,
            },
            prerequisites: vec![],
            time_limit: Some(Duration::from_secs(300)),
            repeatable: true,
            difficulty: QuestDifficulty::Normal,
        });

        // Daily Challenges
        self.add_quest("daily_material_master", Quest {
            name: "Material Master".to_string(),
            description: "Use 5 different material types in a single structure.".to_string(),
            quest_type: QuestType::Daily,
            objectives: vec![
                QuestObjective::UseMaterials {
                    material_types: vec![
                        VoxelType::Stone,
                        VoxelType::Wood,
                        VoxelType::Metal,
                        VoxelType::Glass,
                        VoxelType::Brick,
                    ],
                    in_single_structure: true,
                },
            ],
            rewards: QuestRewards {
                experience: vec![(BuildingSkill::ResourceManagement, 300)],
                items: vec![("loot_box_common".to_string(), 1)],
                unlock_content: vec![],
                currency: 200,
            },
            prerequisites: vec![],
            time_limit: Some(Duration::from_secs(86400)), // 24 hours
            repeatable: true,
            difficulty: QuestDifficulty::Normal,
        });

        self.add_quest("daily_efficient_builder", Quest {
            name: "Efficient Construction".to_string(),
            description: "Build a structure using exactly 100 blocks, no more, no less.".to_string(),
            quest_type: QuestType::Daily,
            objectives: vec![
                QuestObjective::ExactBlockCount {
                    target_count: 100,
                    tolerance: 0,
                },
            ],
            rewards: QuestRewards {
                experience: vec![(BuildingSkill::ResourceManagement, 400)],
                items: vec![("resource_bonus_pack".to_string(), 1)],
                unlock_content: vec![],
                currency: 250,
            },
            prerequisites: vec![],
            time_limit: Some(Duration::from_secs(86400)),
            repeatable: true,
            difficulty: QuestDifficulty::Normal,
        });

        // Community Quests
        self.add_quest("community_collaboration", Quest {
            name: "Community Project".to_string(),
            description: "Collaborate with another player to build a shared structure.".to_string(),
            quest_type: QuestType::Community,
            objectives: vec![
                QuestObjective::CollaborativeBuild {
                    min_players: 2,
                    min_blocks_per_player: 50,
                    structure_type: "any".to_string(),
                },
            ],
            rewards: QuestRewards {
                experience: vec![(BuildingSkill::Construction, 500)],
                items: vec![("social_badge_teamwork".to_string(), 1)],
                unlock_content: vec!["multiplayer_tools".to_string()],
                currency: 300,
            },
            prerequisites: vec![],
            time_limit: None,
            repeatable: true,
            difficulty: QuestDifficulty::Normal,
        });

        // Exploration Quests
        self.add_quest("exploration_ancient_ruins", Quest {
            name: "Ancient Ruins Discovery".to_string(),
            description: "Find and restore an ancient structure to its former glory.".to_string(),
            quest_type: QuestType::Exploration,
            objectives: vec![
                QuestObjective::DiscoverLocation {
                    location_type: "ancient_ruins".to_string(),
                    radius: 500,
                },
                QuestObjective::RestoreStructure {
                    structure_id: "ancient_temple_01".to_string(),
                    restoration_percentage: 0.8,
                },
            ],
            rewards: QuestRewards {
                experience: vec![
                    (BuildingSkill::Architecture, 700),
                    (BuildingSkill::Construction, 500),
                ],
                items: vec![
                    ("artifact_ancient_blueprint".to_string(), 1),
                    ("material_ancient_stone".to_string(), 50),
                ],
                unlock_content: vec!["ancient_building_techniques".to_string()],
                currency: 1000,
            },
            prerequisites: vec!["story_bridge_builder".to_string()],
            time_limit: None,
            repeatable: false,
            difficulty: QuestDifficulty::Hard,
        });
    }

    /// Initialize quest chains/storylines
    fn initialize_quest_chains(&mut self) {
        // Tutorial Chain
        self.quest_chains.insert("tutorial".to_string(), QuestChain {
            name: "Getting Started".to_string(),
            description: "Learn the basics of building in Robin".to_string(),
            quests: vec![
                "tutorial_first_block".to_string(),
                "tutorial_mining".to_string(),
                "tutorial_first_structure".to_string(),
            ],
            final_reward: Some(QuestRewards {
                experience: vec![(BuildingSkill::Construction, 1000)],
                items: vec![("starter_pack_complete".to_string(), 1)],
                unlock_content: vec!["main_story".to_string()],
                currency: 500,
            }),
        });

        // Main Story Chain
        self.quest_chains.insert("main_story".to_string(), QuestChain {
            name: "The Master Builder's Journey".to_string(),
            description: "Follow the path to becoming a master engineer".to_string(),
            quests: vec![
                "story_engineers_workshop".to_string(),
                "story_bridge_builder".to_string(),
                // More story quests would be added here
            ],
            final_reward: Some(QuestRewards {
                experience: vec![
                    (BuildingSkill::Engineering, 2000),
                    (BuildingSkill::Architecture, 1500),
                ],
                items: vec![("title_master_engineer".to_string(), 1)],
                unlock_content: vec!["endgame_content".to_string()],
                currency: 2000,
            }),
        });
    }

    /// Add a quest to the system
    fn add_quest(&mut self, id: &str, quest: Quest) {
        self.quests.insert(id.to_string(), quest);
    }

    /// Start a quest for the player
    pub fn start_quest(&mut self, quest_id: &str, player_data: &PlayerData) -> RobinResult<()> {
        // Check if quest exists
        let quest = self.quests.get(quest_id)
            .ok_or_else(|| RobinError::InvalidInput(format!("Unknown quest: {}", quest_id)))?;

        // Check if already active or completed
        if self.active_quests.contains_key(quest_id) {
            return Err(RobinError::InvalidInput("Quest already active".to_string()));
        }

        if !quest.repeatable && self.completed_quests.contains(quest_id) {
            return Err(RobinError::InvalidInput("Quest already completed".to_string()));
        }

        // Check prerequisites
        for prereq in &quest.prerequisites {
            if !self.completed_quests.contains(prereq) {
                return Err(RobinError::InvalidInput(format!("Prerequisite not met: {}", prereq)));
            }
        }

        // Start the quest
        let progress = QuestProgress {
            quest_id: quest_id.to_string(),
            started_at: SystemTime::now(),
            objectives_progress: vec![ObjectiveProgress::default(); quest.objectives.len()],
            is_completed: false,
        };

        self.active_quests.insert(quest_id.to_string(), progress);
        Ok(())
    }

    /// Update quest progress based on player actions
    pub fn update_progress(&mut self, event: QuestEvent, player_data: &mut PlayerData) -> RobinResult<Vec<String>> {
        let mut completed_quests = Vec::new();

        for (quest_id, progress) in &mut self.active_quests {
            if let Some(quest) = self.quests.get(quest_id) {
                // Check time limit
                if let Some(time_limit) = quest.time_limit {
                    if progress.started_at.elapsed().unwrap_or_default() > time_limit {
                        continue; // Quest expired
                    }
                }

                // Update objectives based on event
                for (obj_idx, objective) in quest.objectives.iter().enumerate() {
                    if progress.objectives_progress[obj_idx].is_completed {
                        continue;
                    }

                    let obj_progress = &mut progress.objectives_progress[obj_idx];

                    match (&event, objective) {
                        (QuestEvent::BlockPlaced { block_type, position },
                         QuestObjective::PlaceBlocks { block_type: target_type, count, location }) => {
                            if target_type.is_none() || target_type.as_ref() == Some(block_type) {
                                if location.is_none() || Self::is_in_location(position, location.as_ref().unwrap()) {
                                    obj_progress.current_value += 1;
                                    if obj_progress.current_value >= *count as f32 {
                                        obj_progress.is_completed = true;
                                    }
                                }
                            }
                        }
                        (QuestEvent::BlockMined { block_type },
                         QuestObjective::MineBlocks { block_type: target_type, count }) => {
                            if target_type.is_none() || target_type.as_ref() == Some(block_type) {
                                obj_progress.current_value += 1;
                                if obj_progress.current_value >= *count as f32 {
                                    obj_progress.is_completed = true;
                                }
                            }
                        }
                        (QuestEvent::StructureCompleted { structure_type, size, features },
                         QuestObjective::BuildStructure { structure_type: target, min_size, requirements }) => {
                            if structure_type == target && Self::meets_size_requirement(size, min_size) {
                                if Self::meets_structure_requirements(features, requirements) {
                                    obj_progress.is_completed = true;
                                }
                            }
                        }
                        _ => {} // Other event/objective combinations
                    }
                }

                // Check if quest is completed
                if progress.objectives_progress.iter().all(|obj| obj.is_completed) {
                    progress.is_completed = true;
                    completed_quests.push(quest_id.clone());
                }
            }
        }

        // Award rewards for completed quests
        for quest_id in &completed_quests {
            self.complete_quest(quest_id, player_data)?;
        }

        Ok(completed_quests)
    }

    /// Complete a quest and award rewards
    fn complete_quest(&mut self, quest_id: &str, player_data: &mut PlayerData) -> RobinResult<()> {
        if let Some(quest) = self.quests.get(quest_id) {
            // Award rewards
            Self::award_rewards(&quest.rewards, player_data);

            // Move to completed
            self.completed_quests.insert(quest_id.to_string());
            self.active_quests.remove(quest_id);

            // Check quest chain completion
            for (chain_id, chain) in &self.quest_chains {
                if chain.quests.contains(&quest_id.to_string()) {
                    // Check if entire chain is complete
                    if chain.quests.iter().all(|qid| self.completed_quests.contains(qid)) {
                        if let Some(final_reward) = &chain.final_reward {
                            Self::award_rewards(final_reward, player_data);
                        }
                    }
                }
            }

            Ok(())
        } else {
            Err(RobinError::InvalidInput(format!("Unknown quest: {}", quest_id)))
        }
    }

    /// Award quest rewards to player
    fn award_rewards(rewards: &QuestRewards, player_data: &mut PlayerData) {
        // Award experience
        for (skill, amount) in &rewards.experience {
            let stat_key = format!("{}_experience", skill.to_string().to_lowercase());
            player_data.stats.custom_stats
                .entry(stat_key)
                .and_modify(|v| *v += *amount as f64)
                .or_insert(*amount as f64);
        }

        // Award items
        for (item_id, quantity) in &rewards.items {
            player_data.add_item(item_id, *quantity);
        }

        // Unlock content
        for content_id in &rewards.unlock_content {
            let unlock_key = format!("quest_unlock_{}", content_id);
            player_data.stats.custom_stats.insert(unlock_key, 1.0);
        }

        // Award currency
        player_data.stats.custom_stats
            .entry("currency".to_string())
            .and_modify(|v| *v += rewards.currency as f64)
            .or_insert(rewards.currency as f64);
    }

    /// Helper functions for quest objectives
    fn is_in_location(position: &Vec3<f32>, location: &QuestLocation) -> bool {
        match location {
            QuestLocation::Area { center, radius } => {
                let distance = ((position.x - center.x).powi(2) +
                               (position.y - center.y).powi(2) +
                               (position.z - center.z).powi(2)).sqrt();
                distance <= *radius
            }
            QuestLocation::Region { min, max } => {
                position.x >= min.x && position.x <= max.x &&
                position.y >= min.y && position.y <= max.y &&
                position.z >= min.z && position.z <= max.z
            }
        }
    }

    fn meets_size_requirement(actual: &Vec3<f32>, required: &Vec3<f32>) -> bool {
        actual.x >= required.x && actual.y >= required.y && actual.z >= required.z
    }

    fn meets_structure_requirements(features: &HashSet<String>, requirements: &[StructureRequirement]) -> bool {
        requirements.iter().all(|req| match req {
            StructureRequirement::HasWalls => features.contains("walls"),
            StructureRequirement::HasRoof => features.contains("roof"),
            StructureRequirement::HasDoor => features.contains("door"),
            StructureRequirement::HasWindows => features.contains("windows"),
            StructureRequirement::HasLighting => features.contains("lighting"),
        })
    }

    /// Get active quests
    pub fn get_active_quests(&self) -> Vec<(&String, &QuestProgress, &Quest)> {
        self.active_quests.iter()
            .filter_map(|(id, progress)| {
                self.quests.get(id).map(|quest| (id, progress, quest))
            })
            .collect()
    }

    /// Get available quests
    pub fn get_available_quests(&self, player_data: &PlayerData) -> Vec<(&String, &Quest)> {
        self.quests.iter()
            .filter(|(id, quest)| {
                // Not active
                !self.active_quests.contains_key(*id) &&
                // Not completed (unless repeatable)
                (quest.repeatable || !self.completed_quests.contains(*id)) &&
                // Prerequisites met
                quest.prerequisites.iter().all(|prereq| self.completed_quests.contains(prereq))
            })
            .collect()
    }

    /// Refresh daily challenges
    pub fn refresh_daily_challenges(&mut self) -> RobinResult<()> {
        let now = SystemTime::now();
        let duration_since_refresh = now.duration_since(self.last_daily_refresh).unwrap_or_default();

        // Refresh if 24 hours have passed
        if duration_since_refresh >= Duration::from_secs(86400) {
            self.daily_challenges.clear();

            // Select random daily quests
            let daily_quests: Vec<String> = self.quests.iter()
                .filter(|(_, quest)| quest.quest_type == QuestType::Daily)
                .take(3) // 3 daily challenges
                .map(|(id, _)| id.clone())
                .collect();

            self.daily_challenges = daily_quests;
            self.last_daily_refresh = now;
        }

        Ok(())
    }
}

/// Quest definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quest {
    pub name: String,
    pub description: String,
    pub quest_type: QuestType,
    pub objectives: Vec<QuestObjective>,
    pub rewards: QuestRewards,
    pub prerequisites: Vec<String>,
    pub time_limit: Option<Duration>,
    pub repeatable: bool,
    pub difficulty: QuestDifficulty,
}

/// Quest types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestType {
    Tutorial,
    Story,
    Challenge,
    Daily,
    Weekly,
    Community,
    Exploration,
    Creative,
}

/// Quest objectives that must be completed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuestObjective {
    PlaceBlocks {
        block_type: Option<VoxelType>,
        count: u32,
        location: Option<QuestLocation>,
    },
    MineBlocks {
        block_type: Option<VoxelType>,
        count: u32
    },
    BuildStructure {
        structure_type: String,
        min_size: Vec3,
        requirements: Vec<StructureRequirement>,
    },
    BuildBridge {
        min_length: u32,
        min_height: u32,
        requirements: Vec<BridgeRequirement>,
    },
    BuildTower {
        min_height: u32,
        max_base_size: u32,
        requirements: Vec<TowerRequirement>,
    },
    ReplicateStructure {
        template_id: String,
        time_limit: Duration,
        accuracy_threshold: f32,
    },
    UseMaterials {
        material_types: Vec<VoxelType>,
        in_single_structure: bool,
    },
    ExactBlockCount {
        target_count: u32,
        tolerance: u32,
    },
    CollaborativeBuild {
        min_players: u32,
        min_blocks_per_player: u32,
        structure_type: String,
    },
    DiscoverLocation {
        location_type: String,
        radius: u32,
    },
    RestoreStructure {
        structure_id: String,
        restoration_percentage: f32,
    },
    PlaceSpecificBlocks {
        blocks: Vec<(String, u32)>,
    },
}

/// Quest location specifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuestLocation {
    Area { center: Vec3, radius: f32 },
    Region { min: Vec3, max: Vec3 },
}

/// Structure requirements for building quests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StructureRequirement {
    HasWalls,
    HasRoof,
    HasDoor,
    HasWindows,
    HasLighting,
}

/// Bridge requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BridgeRequirement {
    HasSupports,
    IsWalkable,
    ConnectsTwoPoints,
}

/// Tower requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TowerRequirement {
    IsStable,
    HasStairs,
    HasWindows,
}

/// Quest rewards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestRewards {
    pub experience: Vec<(BuildingSkill, u32)>,
    pub items: Vec<(String, u32)>,
    pub unlock_content: Vec<String>,
    pub currency: u32,
}

/// Quest difficulty levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestDifficulty {
    Beginner,
    Normal,
    Hard,
    Expert,
    Master,
}

/// Quest progress tracking
#[derive(Debug, Clone)]
pub struct QuestProgress {
    pub quest_id: String,
    pub started_at: SystemTime,
    pub objectives_progress: Vec<ObjectiveProgress>,
    pub is_completed: bool,
}

/// Objective progress
#[derive(Debug, Clone, Default)]
pub struct ObjectiveProgress {
    pub current_value: f32,
    pub is_completed: bool,
}

/// Quest chain (storyline)
#[derive(Debug, Clone)]
pub struct QuestChain {
    pub name: String,
    pub description: String,
    pub quests: Vec<String>,
    pub final_reward: Option<QuestRewards>,
}

/// Quest events for progress tracking
#[derive(Debug, Clone)]
pub enum QuestEvent {
    BlockPlaced { block_type: VoxelType, position: Vec3 },
    BlockMined { block_type: VoxelType },
    StructureCompleted {
        structure_type: String,
        size: Vec3,
        features: HashSet<String>
    },
    ItemCrafted { item_id: String },
    LocationDiscovered { location_type: String, position: Vec3 },
    PlayerCollaboration { player_id: String, action: String },
}

impl Default for QuestManager {
    fn default() -> Self {
        Self::new()
    }
}