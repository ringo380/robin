use super::*;
use crate::engine::{
    scene::{Scene, GameObject},
    physics::PhysicsWorld,
    // ui::UIManager,  // TODO: UIManager not implemented yet
    // animation::AnimationManager,  // TODO: AnimationManager not implemented yet 
    // audio::AudioManager,  // TODO: AudioManager not implemented yet
    graphics::ParticleSystem,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete game state that can be saved/loaded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// Save metadata
    pub metadata: SaveMetadata,
    
    /// Current scene state
    pub scene_state: SerializableScene,
    
    /// Player data
    pub player_data: PlayerData,
    
    /// Game progress tracking
    pub progress: GameProgress,
    
    /// Settings and preferences
    pub settings: GameSettings,
    
    /// Custom game-specific data
    pub custom_data: HashMap<String, serde_json::Value>,
    
    /// Timestamp when this state was saved
    pub save_timestamp: SystemTime,
}

impl GameState {
    /// Create a new game state
    pub fn new(slot: usize, save_name: String) -> Self {
        Self {
            metadata: SaveMetadata::new(slot, save_name),
            scene_state: SerializableScene::default(),
            player_data: PlayerData::default(),
            progress: GameProgress::default(),
            settings: GameSettings::default(),
            custom_data: HashMap::new(),
            save_timestamp: SystemTime::now(),
        }
    }

    /// Update the save timestamp and metadata
    pub fn update_save_info(&mut self, play_time_delta: u64) {
        self.save_timestamp = SystemTime::now();
        self.metadata.update_modified();
        self.metadata.play_time += play_time_delta;
    }

    /// Capture current game state from engine components
    pub fn capture_from_engine(
        &mut self,
        scene: &Scene,
        physics: &PhysicsWorld,
        // ui: &UIManager,  // TODO: Implement when UIManager exists
        // animation: &AnimationManager,  // TODO: Implement when AnimationManager exists
        // audio: &AudioManager,  // TODO: Implement when AudioManager exists
        particles: &ParticleSystem,
    ) -> SaveResult<()> {
        // Capture scene state
        self.scene_state = scene.to_serializable();

        // Update current scene in metadata
        self.metadata.current_scene = scene.get_name().to_string();

        // Store component states as custom data
        if let Ok(physics_state) = serde_json::to_value(physics) {
            self.custom_data.insert("physics".to_string(), physics_state);
        }

        // TODO: Uncomment when these managers are implemented
        // if let Ok(ui_state) = serde_json::to_value(ui) {
        //     self.custom_data.insert("ui".to_string(), ui_state);
        // }
        //
        // if let Ok(animation_state) = serde_json::to_value(animation) {
        //     self.custom_data.insert("animation".to_string(), animation_state);
        // }
        //
        // if let Ok(audio_state) = serde_json::to_value(audio) {
        //     self.custom_data.insert("audio".to_string(), audio_state);
        // }

        if let Ok(particle_state) = serde_json::to_value(particles) {
            self.custom_data.insert("particles".to_string(), particle_state);
        }

        Ok(())
    }

    /// Apply this game state to engine components
    pub fn apply_to_engine(
        &self,
        scene: &mut Scene,
        physics: &mut PhysicsWorld,
        // ui: &mut UIManager,  // TODO: Implement when UIManager exists
        // animation: &mut AnimationManager,  // TODO: Implement when AnimationManager exists
        // audio: &mut AudioManager,  // TODO: Implement when AudioManager exists
        particles: &mut ParticleSystem,
    ) -> SaveResult<()> {
        // Apply scene state
        scene.load_from_serializable(&self.scene_state);

        // Restore component states from custom data
        if let Some(physics_state) = self.custom_data.get("physics") {
            if let Ok(restored_physics) = serde_json::from_value::<PhysicsWorld>(physics_state.clone()) {
                *physics = restored_physics;
            }
        }

        // TODO: Uncomment when these managers are implemented
        // if let Some(ui_state) = self.custom_data.get("ui") {
        //     if let Ok(restored_ui) = serde_json::from_value::<UIManager>(ui_state.clone()) {
        //         *ui = restored_ui;
        //     }
        // }
        //
        // if let Some(animation_state) = self.custom_data.get("animation") {
        //     if let Ok(restored_animation) = serde_json::from_value::<AnimationManager>(animation_state.clone()) {
        //         *animation = restored_animation;
        //     }
        // }
        //
        // if let Some(audio_state) = self.custom_data.get("audio") {
        //     if let Ok(restored_audio) = serde_json::from_value::<AudioManager>(audio_state.clone()) {
        //         *audio = restored_audio;
        //     }
        // }

        if let Some(particle_state) = self.custom_data.get("particles") {
            if let Ok(restored_particles) = serde_json::from_value::<ParticleSystem>(particle_state.clone()) {
                *particles = restored_particles;
            }
        }

        Ok(())
    }

    /// Add custom game data
    pub fn set_custom_data<T: Serialize>(&mut self, key: &str, value: &T) -> SaveResult<()> {
        let json_value = serde_json::to_value(value)
            .map_err(|e| SaveError::SerializationError(e.to_string()))?;
        self.custom_data.insert(key.to_string(), json_value);
        Ok(())
    }

    /// Get custom game data
    pub fn get_custom_data<T: for<'de> Deserialize<'de>>(&self, key: &str) -> SaveResult<Option<T>> {
        if let Some(value) = self.custom_data.get(key) {
            let result = serde_json::from_value(value.clone())
                .map_err(|e| SaveError::SerializationError(e.to_string()))?;
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    /// Update progress information
    pub fn update_progress(&mut self, completion_percentage: f32) {
        self.progress.completion_percentage = completion_percentage.clamp(0.0, 100.0);
        self.metadata.progress = self.progress.completion_percentage;
    }
}

/// Player-specific data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerData {
    /// Player name
    pub name: String,
    /// Player level
    pub level: u32,
    /// Experience points
    pub experience: u64,
    /// Player position
    pub position: (f32, f32),
    /// Player health
    pub health: f32,
    /// Player max health
    pub max_health: f32,
    /// Player inventory
    pub inventory: HashMap<String, u32>,
    /// Player stats
    pub stats: PlayerStats,
    /// Player achievements
    pub achievements: Vec<String>,
}

impl PlayerData {
    pub fn new(name: String) -> Self {
        Self {
            name,
            level: 1,
            experience: 0,
            position: (0.0, 0.0),
            health: 100.0,
            max_health: 100.0,
            inventory: HashMap::new(),
            stats: PlayerStats::default(),
            achievements: Vec::new(),
        }
    }

    /// Add item to inventory
    pub fn add_item(&mut self, item_id: &str, quantity: u32) {
        *self.inventory.entry(item_id.to_string()).or_insert(0) += quantity;
    }

    /// Remove item from inventory
    pub fn remove_item(&mut self, item_id: &str, quantity: u32) -> bool {
        if let Some(current) = self.inventory.get_mut(item_id) {
            if *current >= quantity {
                *current -= quantity;
                if *current == 0 {
                    self.inventory.remove(item_id);
                }
                return true;
            }
        }
        false
    }

    /// Get item quantity
    pub fn get_item_count(&self, item_id: &str) -> u32 {
        self.inventory.get(item_id).copied().unwrap_or(0)
    }

    /// Add experience and handle level ups
    pub fn add_experience(&mut self, exp: u64) {
        self.experience += exp;
        
        // Simple level calculation (1000 exp per level)
        let new_level = (self.experience / 1000) as u32 + 1;
        if new_level > self.level {
            self.level = new_level;
            // Level up bonuses
            self.max_health += 10.0;
            self.health = self.max_health;
        }
    }

    /// Unlock an achievement
    pub fn unlock_achievement(&mut self, achievement_id: &str) {
        if !self.achievements.contains(&achievement_id.to_string()) {
            self.achievements.push(achievement_id.to_string());
        }
    }
}

/// Player statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerStats {
    /// Total damage dealt
    pub damage_dealt: u64,
    /// Total damage taken
    pub damage_taken: u64,
    /// Enemies defeated
    pub enemies_defeated: u32,
    /// Items collected
    pub items_collected: u32,
    /// Distance traveled
    pub distance_traveled: f32,
    /// Time played (seconds)
    pub time_played: u64,
    /// Deaths
    pub deaths: u32,
    /// Custom stats
    pub custom_stats: HashMap<String, f64>,
}

impl PlayerStats {
    /// Add a custom stat
    pub fn add_stat(&mut self, stat_name: &str, value: f64) {
        *self.custom_stats.entry(stat_name.to_string()).or_insert(0.0) += value;
    }

    /// Get a custom stat
    pub fn get_stat(&self, stat_name: &str) -> f64 {
        self.custom_stats.get(stat_name).copied().unwrap_or(0.0)
    }
}

/// Game progress tracking
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameProgress {
    /// Overall completion percentage (0-100)
    pub completion_percentage: f32,
    /// Current level/chapter
    pub current_level: String,
    /// Completed levels
    pub completed_levels: Vec<String>,
    /// Unlocked content
    pub unlocked_content: Vec<String>,
    /// Story flags and variables
    pub story_flags: HashMap<String, bool>,
    /// Numeric story variables
    pub story_variables: HashMap<String, f64>,
    /// Checkpoint data
    pub checkpoints: Vec<Checkpoint>,
}

impl GameProgress {
    /// Mark a level as completed
    pub fn complete_level(&mut self, level_id: &str) {
        if !self.completed_levels.contains(&level_id.to_string()) {
            self.completed_levels.push(level_id.to_string());
        }
    }

    /// Unlock content
    pub fn unlock_content(&mut self, content_id: &str) {
        if !self.unlocked_content.contains(&content_id.to_string()) {
            self.unlocked_content.push(content_id.to_string());
        }
    }

    /// Set a story flag
    pub fn set_story_flag(&mut self, flag_name: &str, value: bool) {
        self.story_flags.insert(flag_name.to_string(), value);
    }

    /// Get a story flag
    pub fn get_story_flag(&self, flag_name: &str) -> bool {
        self.story_flags.get(flag_name).copied().unwrap_or(false)
    }

    /// Set a story variable
    pub fn set_story_variable(&mut self, var_name: &str, value: f64) {
        self.story_variables.insert(var_name.to_string(), value);
    }

    /// Get a story variable
    pub fn get_story_variable(&self, var_name: &str) -> f64 {
        self.story_variables.get(var_name).copied().unwrap_or(0.0)
    }

    /// Add a checkpoint
    pub fn add_checkpoint(&mut self, checkpoint: Checkpoint) {
        self.checkpoints.push(checkpoint);
        
        // Keep only the last 10 checkpoints
        if self.checkpoints.len() > 10 {
            self.checkpoints.drain(0..self.checkpoints.len() - 10);
        }
    }

    /// Get the latest checkpoint
    pub fn get_latest_checkpoint(&self) -> Option<&Checkpoint> {
        self.checkpoints.last()
    }
}

/// Checkpoint data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Checkpoint ID
    pub id: String,
    /// Scene name
    pub scene_name: String,
    /// Player position
    pub player_position: (f32, f32),
    /// Player health at checkpoint
    pub player_health: f32,
    /// Timestamp when checkpoint was created
    pub timestamp: SystemTime,
    /// Custom checkpoint data
    pub data: HashMap<String, serde_json::Value>,
}

impl Checkpoint {
    pub fn new(id: String, scene_name: String, player_position: (f32, f32), player_health: f32) -> Self {
        Self {
            id,
            scene_name,
            player_position,
            player_health,
            timestamp: SystemTime::now(),
            data: HashMap::new(),
        }
    }
}

/// Game settings that are saved with the game state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSettings {
    /// Audio settings
    pub audio: AudioSettings,
    /// Graphics settings
    pub graphics: GraphicsSettings,
    /// Input settings
    pub input: InputSettings,
    /// Gameplay settings
    pub gameplay: GameplaySettings,
    /// Accessibility settings
    pub accessibility: AccessibilitySettings,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            audio: AudioSettings::default(),
            graphics: GraphicsSettings::default(),
            input: InputSettings::default(),
            gameplay: GameplaySettings::default(),
            accessibility: AccessibilitySettings::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettings {
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub voice_volume: f32,
    pub muted: bool,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            music_volume: 0.8,
            sfx_volume: 1.0,
            voice_volume: 1.0,
            muted: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsSettings {
    pub resolution: (u32, u32),
    pub fullscreen: bool,
    pub vsync: bool,
    pub graphics_quality: GraphicsQuality,
    pub brightness: f32,
    pub contrast: f32,
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            resolution: (1920, 1080),
            fullscreen: false,
            vsync: true,
            graphics_quality: GraphicsQuality::High,
            brightness: 1.0,
            contrast: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphicsQuality {
    Low,
    Medium,
    High,
    Ultra,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InputSettings {
    pub key_bindings: HashMap<String, String>,
    pub mouse_sensitivity: f32,
    pub invert_mouse: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameplaySettings {
    pub difficulty: Difficulty,
    pub auto_save: bool,
    pub tutorials_enabled: bool,
    pub hints_enabled: bool,
}

impl Default for GameplaySettings {
    fn default() -> Self {
        Self {
            difficulty: Difficulty::Normal,
            auto_save: true,
            tutorials_enabled: true,
            hints_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilitySettings {
    pub colorblind_support: bool,
    pub high_contrast: bool,
    pub large_text: bool,
    pub subtitles: bool,
    pub screen_reader_support: bool,
}

impl Default for AccessibilitySettings {
    fn default() -> Self {
        Self {
            colorblind_support: false,
            high_contrast: false,
            large_text: false,
            subtitles: false,
            screen_reader_support: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_data() {
        let mut player = PlayerData::new("TestPlayer".to_string());
        
        // Test inventory
        player.add_item("sword", 1);
        player.add_item("potion", 5);
        assert_eq!(player.get_item_count("sword"), 1);
        assert_eq!(player.get_item_count("potion"), 5);
        
        // Test experience and leveling
        player.add_experience(1500);
        assert_eq!(player.level, 2);
        assert_eq!(player.max_health, 110.0);
        
        // Test achievements
        player.unlock_achievement("first_kill");
        assert!(player.achievements.contains(&"first_kill".to_string()));
    }

    #[test]
    fn test_game_progress() {
        let mut progress = GameProgress::default();
        
        progress.complete_level("level_1");
        progress.unlock_content("bonus_weapon");
        progress.set_story_flag("met_npc", true);
        progress.set_story_variable("coins", 100.0);
        
        assert!(progress.completed_levels.contains(&"level_1".to_string()));
        assert!(progress.unlocked_content.contains(&"bonus_weapon".to_string()));
        assert!(progress.get_story_flag("met_npc"));
        assert_eq!(progress.get_story_variable("coins"), 100.0);
    }

    #[test]
    fn test_checkpoint() {
        let checkpoint = Checkpoint::new(
            "checkpoint_1".to_string(),
            "level_1".to_string(),
            (100.0, 200.0),
            75.0
        );
        
        assert_eq!(checkpoint.id, "checkpoint_1");
        assert_eq!(checkpoint.scene_name, "level_1");
        assert_eq!(checkpoint.player_position, (100.0, 200.0));
        assert_eq!(checkpoint.player_health, 75.0);
    }
}