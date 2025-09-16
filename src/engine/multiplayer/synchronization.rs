use std::collections::{HashMap, VecDeque, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

use crate::engine::error::RobinResult;
use super::{UserId, WorldSnapshot, UserConstruction, TerrainModification, EntitySnapshot};
use super::networking::{NetworkManager, NetworkMessage, NetworkMessageType, MessagePriority};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncUpdate {
    pub id: u64,
    pub timestamp: u64,
    pub update_type: SyncUpdateType,
    pub author: UserId,
    pub sequence_number: u64,
    pub dependencies: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncUpdateType {
    EntityUpdate(EntitySnapshot),
    ConstructionAdd(UserConstruction),
    ConstructionModify { id: uuid::Uuid, changes: HashMap<String, String> },
    ConstructionRemove(uuid::Uuid),
    TerrainModify(TerrainModification),
    WorldStateSync(WorldSnapshot),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    pub version: u64,
    pub last_sync_time: u64,
    pub pending_updates: Vec<SyncUpdate>,
    pub confirmed_updates: Vec<u64>,
    pub user_cursors: HashMap<UserId, UserCursor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCursor {
    pub user_id: UserId,
    pub position: [f32; 3],
    pub tool: Option<String>,
    pub selection: Option<SelectionData>,
    pub last_activity: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionData {
    pub selection_type: SelectionType,
    pub entities: Vec<uuid::Uuid>,
    pub area: Option<BoundingBox>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SelectionType {
    Single,
    Multiple,
    Area,
    Path,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

pub struct SynchronizationEngine {
    pub world_state: Arc<RwLock<WorldSnapshot>>,
    pub sync_state: Arc<RwLock<SyncState>>,
    pub network_manager: Option<Arc<RwLock<NetworkManager>>>,
    pub update_queue: VecDeque<SyncUpdate>,
    pub conflict_buffer: Vec<ConflictEntry>,
    pub sync_rate: Duration,
    pub last_full_sync: Instant,
    pub full_sync_interval: Duration,
    pub delta_compression: DeltaCompression,
    pub prediction_system: PredictionSystem,
    pub interpolation_system: InterpolationSystem,
}

#[derive(Debug, Clone)]
pub struct ConflictEntry {
    pub update_a: SyncUpdate,
    pub update_b: SyncUpdate,
    pub conflict_type: ConflictType,
    pub timestamp: Instant,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConflictType {
    SimultaneousEdit,
    DependencyViolation,
    PermissionConflict,
    StateInconsistency,
}

pub struct DeltaCompression {
    pub previous_snapshots: VecDeque<WorldSnapshot>,
    pub max_history: usize,
    pub compression_ratio: f32,
}

impl DeltaCompression {
    pub fn new() -> Self {
        Self {
            previous_snapshots: VecDeque::new(),
            max_history: 10,
            compression_ratio: 0.0,
        }
    }

    pub fn compress_update(&mut self, current: &WorldSnapshot, previous: &WorldSnapshot) -> Vec<u8> {
        let delta = self.calculate_delta(current, previous);
        bincode::serialize(&delta).unwrap_or_default()
    }

    fn calculate_delta(&self, current: &WorldSnapshot, previous: &WorldSnapshot) -> WorldDelta {
        let mut added_entities = Vec::new();
        let mut modified_entities = Vec::new();
        let mut removed_entities = Vec::new();

        let current_entities: HashMap<uuid::Uuid, &EntitySnapshot> = 
            current.entities.iter().map(|e| (e.id, e)).collect();
        let previous_entities: HashMap<uuid::Uuid, &EntitySnapshot> = 
            previous.entities.iter().map(|e| (e.id, e)).collect();

        for (id, entity) in &current_entities {
            match previous_entities.get(id) {
                Some(prev_entity) => {
                    if entity != prev_entity {
                        modified_entities.push((*entity).clone());
                    }
                }
                None => {
                    added_entities.push((*entity).clone());
                }
            }
        }

        for id in previous_entities.keys() {
            if !current_entities.contains_key(id) {
                removed_entities.push(*id);
            }
        }

        WorldDelta {
            version: current.version,
            timestamp: current.timestamp,
            added_entities,
            modified_entities,
            removed_entities,
            terrain_changes: self.calculate_terrain_delta(&current.terrain_chunks, &previous.terrain_chunks),
            construction_changes: self.calculate_construction_delta(&current.user_constructions, &previous.user_constructions),
        }
    }

    fn calculate_terrain_delta(&self, current: &[super::TerrainChunk], previous: &[super::TerrainChunk]) -> Vec<TerrainDelta> {
        let mut deltas = Vec::new();
        
        for current_chunk in current {
            if let Some(previous_chunk) = previous.iter().find(|c| c.chunk_id == current_chunk.chunk_id) {
                if current_chunk.modifications.len() != previous_chunk.modifications.len() {
                    deltas.push(TerrainDelta {
                        chunk_id: current_chunk.chunk_id,
                        new_modifications: current_chunk.modifications[previous_chunk.modifications.len()..].to_vec(),
                    });
                }
            } else {
                deltas.push(TerrainDelta {
                    chunk_id: current_chunk.chunk_id,
                    new_modifications: current_chunk.modifications.clone(),
                });
            }
        }
        
        deltas
    }

    fn calculate_construction_delta(&self, current: &[UserConstruction], previous: &[UserConstruction]) -> Vec<ConstructionDelta> {
        let mut deltas = Vec::new();
        
        let current_map: HashMap<uuid::Uuid, &UserConstruction> = 
            current.iter().map(|c| (c.id, c)).collect();
        let previous_map: HashMap<uuid::Uuid, &UserConstruction> = 
            previous.iter().map(|c| (c.id, c)).collect();

        for (id, construction) in &current_map {
            match previous_map.get(id) {
                Some(prev_construction) => {
                    if construction.modified_at != prev_construction.modified_at {
                        deltas.push(ConstructionDelta {
                            id: *id,
                            delta_type: ConstructionDeltaType::Modified((*construction).clone()),
                        });
                    }
                }
                None => {
                    deltas.push(ConstructionDelta {
                        id: *id,
                        delta_type: ConstructionDeltaType::Added((*construction).clone()),
                    });
                }
            }
        }

        for id in previous_map.keys() {
            if !current_map.contains_key(id) {
                deltas.push(ConstructionDelta {
                    id: *id,
                    delta_type: ConstructionDeltaType::Removed,
                });
            }
        }

        deltas
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldDelta {
    pub version: u64,
    pub timestamp: u64,
    pub added_entities: Vec<EntitySnapshot>,
    pub modified_entities: Vec<EntitySnapshot>,
    pub removed_entities: Vec<uuid::Uuid>,
    pub terrain_changes: Vec<TerrainDelta>,
    pub construction_changes: Vec<ConstructionDelta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainDelta {
    pub chunk_id: [i32; 2],
    pub new_modifications: Vec<TerrainModification>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstructionDelta {
    pub id: uuid::Uuid,
    pub delta_type: ConstructionDeltaType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstructionDeltaType {
    Added(UserConstruction),
    Modified(UserConstruction),
    Removed,
}

pub struct PredictionSystem {
    pub predicted_states: HashMap<UserId, PredictedState>,
    pub prediction_window: Duration,
    pub correction_threshold: f32,
}

#[derive(Debug, Clone)]
pub struct PredictedState {
    pub user_id: UserId,
    pub predicted_position: [f32; 3],
    pub predicted_actions: Vec<PredictedAction>,
    pub confidence: f32,
    pub timestamp: Instant,
}

#[derive(Debug, Clone)]
pub struct PredictedAction {
    pub action_type: String,
    pub parameters: HashMap<String, String>,
    pub probability: f32,
}

impl PredictionSystem {
    pub fn new() -> Self {
        Self {
            predicted_states: HashMap::new(),
            prediction_window: Duration::from_millis(100),
            correction_threshold: 5.0,
        }
    }

    pub fn predict_user_action(&mut self, user_id: UserId, current_state: &UserCursor) -> Option<PredictedAction> {
        let now = Instant::now();
        
        if let Some(predicted) = self.predicted_states.get(&user_id) {
            if now.duration_since(predicted.timestamp) < self.prediction_window {
                return predicted.predicted_actions.first().cloned();
            }
        }

        let prediction = self.analyze_user_pattern(user_id.clone(), current_state);

        self.predicted_states.insert(user_id.clone(), PredictedState {
            user_id,
            predicted_position: current_state.position,
            predicted_actions: vec![prediction.clone()],
            confidence: 0.7,
            timestamp: now,
        });

        Some(prediction)
    }

    fn analyze_user_pattern(&self, _user_id: UserId, current_state: &UserCursor) -> PredictedAction {
        if let Some(tool) = &current_state.tool {
            match tool.as_str() {
                "build" => PredictedAction {
                    action_type: "place_construction".to_string(),
                    parameters: HashMap::new(),
                    probability: 0.8,
                },
                "terraform" => PredictedAction {
                    action_type: "modify_terrain".to_string(),
                    parameters: HashMap::new(),
                    probability: 0.75,
                },
                _ => PredictedAction {
                    action_type: "idle".to_string(),
                    parameters: HashMap::new(),
                    probability: 0.5,
                }
            }
        } else {
            PredictedAction {
                action_type: "move".to_string(),
                parameters: HashMap::new(),
                probability: 0.6,
            }
        }
    }

    pub fn validate_prediction(&mut self, user_id: &UserId, actual_action: &str) -> bool {
        if let Some(predicted) = self.predicted_states.get(user_id) {
            if let Some(predicted_action) = predicted.predicted_actions.first() {
                return predicted_action.action_type == actual_action;
            }
        }
        false
    }
}

pub struct InterpolationSystem {
    pub interpolated_states: HashMap<uuid::Uuid, InterpolatedEntity>,
    pub interpolation_delay: Duration,
}

#[derive(Debug, Clone)]
pub struct InterpolatedEntity {
    pub id: uuid::Uuid,
    pub current_position: [f32; 3],
    pub target_position: [f32; 3],
    pub current_rotation: [f32; 4],
    pub target_rotation: [f32; 4],
    pub interpolation_start: Instant,
    pub interpolation_duration: Duration,
}

impl InterpolationSystem {
    pub fn new() -> Self {
        Self {
            interpolated_states: HashMap::new(),
            interpolation_delay: Duration::from_millis(100),
        }
    }

    pub fn update_entity_target(&mut self, entity: &EntitySnapshot) {
        let now = Instant::now();
        
        if let Some(interpolated) = self.interpolated_states.get_mut(&entity.id) {
            interpolated.target_position = entity.position;
            interpolated.target_rotation = entity.rotation;
            interpolated.interpolation_start = now;
            interpolated.interpolation_duration = self.interpolation_delay;
        } else {
            self.interpolated_states.insert(entity.id, InterpolatedEntity {
                id: entity.id,
                current_position: entity.position,
                target_position: entity.position,
                current_rotation: entity.rotation,
                target_rotation: entity.rotation,
                interpolation_start: now,
                interpolation_duration: Duration::from_millis(0),
            });
        }
    }

    pub fn update_interpolations(&mut self, delta_time: f32) {
        let now = Instant::now();
        
        for interpolated in self.interpolated_states.values_mut() {
            let elapsed = now.duration_since(interpolated.interpolation_start);
            if elapsed < interpolated.interpolation_duration {
                let t = elapsed.as_secs_f32() / interpolated.interpolation_duration.as_secs_f32();
                let eased_t = Self::ease_in_out_cubic_static(t);

                for i in 0..3 {
                    interpolated.current_position[i] =
                        interpolated.current_position[i] * (1.0 - eased_t) + interpolated.target_position[i] * eased_t;
                }

                interpolated.current_rotation = Self::slerp_quaternion_static(
                    interpolated.current_rotation,
                    interpolated.target_rotation,
                    eased_t,
                );
            } else {
                interpolated.current_position = interpolated.target_position;
                interpolated.current_rotation = interpolated.target_rotation;
            }
        }
    }

    fn ease_in_out_cubic(&self, t: f32) -> f32 {
        Self::ease_in_out_cubic_static(t)
    }

    fn ease_in_out_cubic_static(t: f32) -> f32 {
        if t < 0.5 {
            4.0 * t * t * t
        } else {
            1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
        }
    }

    fn slerp_quaternion(&self, q1: [f32; 4], q2: [f32; 4], t: f32) -> [f32; 4] {
        Self::slerp_quaternion_static(q1, q2, t)
    }

    fn slerp_quaternion_static(q1: [f32; 4], q2: [f32; 4], t: f32) -> [f32; 4] {
        let dot = q1[0] * q2[0] + q1[1] * q2[1] + q1[2] * q2[2] + q1[3] * q2[3];

        if dot.abs() > 0.9995 {
            return [
                q1[0] * (1.0 - t) + q2[0] * t,
                q1[1] * (1.0 - t) + q2[1] * t,
                q1[2] * (1.0 - t) + q2[2] * t,
                q1[3] * (1.0 - t) + q2[3] * t,
            ];
        }

        let theta_0 = dot.abs().acos();
        let sin_theta_0 = theta_0.sin();

        let theta = theta_0 * t;
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        let s0 = cos_theta - dot * sin_theta / sin_theta_0;
        let s1 = sin_theta / sin_theta_0;

        [
            s0 * q1[0] + s1 * q2[0],
            s0 * q1[1] + s1 * q2[1],
            s0 * q1[2] + s1 * q2[2],
            s0 * q1[3] + s1 * q2[3],
        ]
    }
}

impl SynchronizationEngine {
    pub fn new(world_state: Arc<RwLock<WorldSnapshot>>) -> RobinResult<Self> {
        Ok(Self {
            world_state,
            sync_state: Arc::new(RwLock::new(SyncState {
                version: 0,
                last_sync_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                pending_updates: Vec::new(),
                confirmed_updates: Vec::new(),
                user_cursors: HashMap::new(),
            })),
            network_manager: None,
            update_queue: VecDeque::new(),
            conflict_buffer: Vec::new(),
            sync_rate: Duration::from_millis(16),
            last_full_sync: Instant::now(),
            full_sync_interval: Duration::from_secs(10),
            delta_compression: DeltaCompression::new(),
            prediction_system: PredictionSystem::new(),
            interpolation_system: InterpolationSystem::new(),
        })
    }

    pub fn set_network_manager(&mut self, network_manager: Arc<RwLock<NetworkManager>>) {
        self.network_manager = Some(network_manager);
    }

    pub fn update(&mut self, delta_time: f32) -> RobinResult<()> {
        self.process_pending_updates()?;
        self.interpolation_system.update_interpolations(delta_time);
        
        let now = Instant::now();
        if now.duration_since(self.last_full_sync) > self.full_sync_interval {
            self.perform_full_sync()?;
            self.last_full_sync = now;
        } else {
            self.perform_delta_sync()?;
        }

        self.resolve_conflicts()?;
        Ok(())
    }

    pub fn add_sync_update(&mut self, update: SyncUpdate) -> RobinResult<()> {
        self.update_queue.push_back(update.clone());
        
        if let Some(network_manager) = &self.network_manager {
            let message = NetworkMessage {
                id: format!("world_update_{}", chrono::Utc::now().timestamp_millis()),
                sender: update.author.clone(),
                recipient: None, // Broadcast
                message_type: NetworkMessageType::WorldChange,
                payload: bincode::serialize(&self.world_state.read().unwrap().clone()).unwrap_or_default(),
                timestamp: chrono::Utc::now().timestamp() as f64,
                priority: MessagePriority::Normal,
                requires_ack: false,
                compression: None,
            };
            network_manager.write().unwrap().broadcast_message(message, Some(&update.author.0))?;
        }
        
        Ok(())
    }

    pub fn broadcast_construction_update(&self, construction: UserConstruction) -> RobinResult<()> {
        if let Some(network_manager) = &self.network_manager {
            let message = NetworkMessage {
                id: format!("construction_update_{}", chrono::Utc::now().timestamp_millis()),
                sender: UserId("system".to_string()),
                recipient: None,
                message_type: NetworkMessageType::WorldChange,
                payload: bincode::serialize(&construction).unwrap_or_default(),
                timestamp: chrono::Utc::now().timestamp() as f64,
                priority: MessagePriority::Normal,
                requires_ack: false,
                compression: None,
            };
            network_manager.write().unwrap().broadcast_message(message, None)?;
        }
        Ok(())
    }

    pub fn broadcast_terrain_update(&self, modification: TerrainModification) -> RobinResult<()> {
        if let Some(network_manager) = &self.network_manager {
            let message = NetworkMessage {
                id: format!("terrain_update_{}", chrono::Utc::now().timestamp_millis()),
                sender: UserId("system".to_string()),
                recipient: None,
                message_type: NetworkMessageType::WorldChange,
                payload: bincode::serialize(&modification).unwrap_or_default(),
                timestamp: chrono::Utc::now().timestamp() as f64,
                priority: MessagePriority::Normal,
                requires_ack: false,
                compression: None,
            };
            network_manager.write().unwrap().broadcast_message(message, None)?;
        }
        Ok(())
    }

    fn process_pending_updates(&mut self) -> RobinResult<()> {
        while let Some(update) = self.update_queue.pop_front() {
            match self.apply_update(&update) {
                Ok(()) => {
                    self.sync_state.write().unwrap().confirmed_updates.push(update.id);
                }
                Err(e) => {
                    eprintln!("Failed to apply update {}: {}", update.id, e);
                    self.handle_update_failure(update)?;
                }
            }
        }
        Ok(())
    }

    fn apply_update(&mut self, update: &SyncUpdate) -> RobinResult<()> {
        let mut world = self.world_state.write().unwrap();
        
        match &update.update_type {
            SyncUpdateType::EntityUpdate(entity) => {
                if let Some(existing) = world.entities.iter_mut().find(|e| e.id == entity.id) {
                    *existing = entity.clone();
                } else {
                    world.entities.push(entity.clone());
                }
                self.interpolation_system.update_entity_target(entity);
            }
            SyncUpdateType::ConstructionAdd(construction) => {
                world.user_constructions.push(construction.clone());
            }
            SyncUpdateType::ConstructionModify { id, changes } => {
                if let Some(construction) = world.user_constructions.iter_mut().find(|c| c.id == *id) {
                    // Since ComponentSnapshot only has raw data, we update the construction timestamp
                    // In a full implementation, the changes would need to be applied to construction_data
                    // or we'd need a different approach for component properties
                    construction.modified_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                }
            }
            SyncUpdateType::ConstructionRemove(id) => {
                world.user_constructions.retain(|c| c.id != *id);
            }
            SyncUpdateType::TerrainModify(modification) => {
                for chunk in &mut world.terrain_chunks {
                    if self.is_modification_in_chunk(modification, chunk) {
                        chunk.modifications.push(modification.clone());
                        break;
                    }
                }
            }
            SyncUpdateType::WorldStateSync(snapshot) => {
                *world = snapshot.clone();
            }
        }
        
        world.version = update.sequence_number;
        world.timestamp = update.timestamp;
        
        Ok(())
    }

    fn handle_update_failure(&mut self, failed_update: SyncUpdate) -> RobinResult<()> {
        eprintln!("Handling failed update: {:?}", failed_update.id);
        
        if let Some(network_manager) = &self.network_manager {
            let error_message = NetworkMessage {
                id: format!("error_{}", chrono::Utc::now().timestamp_millis()),
                sender: UserId("system".to_string()),
                recipient: Some(failed_update.author.clone()),
                message_type: NetworkMessageType::SystemCommand,
                payload: format!("Failed to apply update {}", failed_update.id).into_bytes(),
                timestamp: chrono::Utc::now().timestamp() as f64,
                priority: MessagePriority::High,
                requires_ack: false,
                compression: None,
            };
            network_manager.write().unwrap().send_message_to_user(&failed_update.author.0, error_message)?;
        }
        
        Ok(())
    }

    fn perform_full_sync(&mut self) -> RobinResult<()> {
        if let Some(network_manager) = &self.network_manager {
            let world_snapshot = self.world_state.read().unwrap().clone();
            let message = NetworkMessage {
                id: format!("full_sync_{}", chrono::Utc::now().timestamp_millis()),
                sender: UserId("system".to_string()),
                recipient: None,
                message_type: NetworkMessageType::WorldChange,
                payload: bincode::serialize(&world_snapshot).unwrap_or_default(),
                timestamp: chrono::Utc::now().timestamp() as f64,
                priority: MessagePriority::High,
                requires_ack: false,
                compression: None,
            };
            network_manager.write().unwrap().broadcast_message(message, None)?;
        }
        Ok(())
    }

    fn perform_delta_sync(&mut self) -> RobinResult<()> {
        let current = self.world_state.read().unwrap().clone();

        if let Some(previous) = self.delta_compression.previous_snapshots.back() {
            let previous_clone = previous.clone();
            let _compressed_delta = self.delta_compression.compress_update(&current, &previous_clone);
        }

        self.delta_compression.previous_snapshots.push_back(current);
        if self.delta_compression.previous_snapshots.len() > self.delta_compression.max_history {
            self.delta_compression.previous_snapshots.pop_front();
        }
        
        Ok(())
    }

    fn resolve_conflicts(&mut self) -> RobinResult<()> {
        let conflicts_to_resolve: Vec<ConflictEntry> = self.conflict_buffer.drain(..).collect();
        
        for conflict in conflicts_to_resolve {
            match conflict.conflict_type {
                ConflictType::SimultaneousEdit => {
                    self.resolve_simultaneous_edit_conflict(conflict)?;
                }
                ConflictType::DependencyViolation => {
                    self.resolve_dependency_conflict(conflict)?;
                }
                ConflictType::PermissionConflict => {
                    self.resolve_permission_conflict(conflict)?;
                }
                ConflictType::StateInconsistency => {
                    self.resolve_state_inconsistency_conflict(conflict)?;
                }
            }
        }
        
        Ok(())
    }

    fn resolve_simultaneous_edit_conflict(&mut self, conflict: ConflictEntry) -> RobinResult<()> {
        if conflict.update_a.timestamp < conflict.update_b.timestamp {
            self.apply_update(&conflict.update_a)?;
            self.apply_update(&conflict.update_b)?;
        } else {
            self.apply_update(&conflict.update_b)?;
            self.apply_update(&conflict.update_a)?;
        }
        Ok(())
    }

    fn resolve_dependency_conflict(&mut self, _conflict: ConflictEntry) -> RobinResult<()> {
        Ok(())
    }

    fn resolve_permission_conflict(&mut self, _conflict: ConflictEntry) -> RobinResult<()> {
        Ok(())
    }

    fn resolve_state_inconsistency_conflict(&mut self, _conflict: ConflictEntry) -> RobinResult<()> {
        self.perform_full_sync()?;
        Ok(())
    }

    fn is_modification_in_chunk(&self, modification: &TerrainModification, chunk: &super::TerrainChunk) -> bool {
        let [x, _y, z] = modification.position;
        let chunk_size = 64.0;
        let chunk_x = (x / chunk_size).floor() as i32;
        let chunk_z = (z / chunk_size).floor() as i32;
        
        chunk.chunk_id == [chunk_x, chunk_z]
    }

    pub fn update_user_cursor(&mut self, user_id: UserId, cursor: UserCursor) -> RobinResult<()> {
        self.sync_state.write().unwrap().user_cursors.insert(user_id.clone(), cursor.clone());
        
        if let Some(predicted_action) = self.prediction_system.predict_user_action(user_id, &cursor) {
            println!("Predicted action for user: {:?}", predicted_action.action_type);
        }
        
        Ok(())
    }

    pub fn get_user_cursors(&self) -> HashMap<UserId, UserCursor> {
        self.sync_state.read().unwrap().user_cursors.clone()
    }

    pub fn get_sync_stats(&self) -> SyncStats {
        let sync_state = self.sync_state.read().unwrap();
        SyncStats {
            version: sync_state.version,
            pending_updates: sync_state.pending_updates.len(),
            confirmed_updates: sync_state.confirmed_updates.len(),
            active_users: sync_state.user_cursors.len(),
            conflicts_resolved: self.conflict_buffer.len(),
            compression_ratio: self.delta_compression.compression_ratio,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStats {
    pub version: u64,
    pub pending_updates: usize,
    pub confirmed_updates: usize,
    pub active_users: usize,
    pub conflicts_resolved: usize,
    pub compression_ratio: f32,
}