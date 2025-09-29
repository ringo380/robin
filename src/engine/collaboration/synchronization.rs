/*!
 * Real-time Synchronization and Conflict Resolution System
 *
 * Handles real-time synchronization of building actions, state management,
 * and conflict resolution for collaborative engineering projects.
 */

use crate::engine::{
    error::{RobinError, RobinResult},
    math::Vec3,
    world::VoxelType,
};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, VecDeque, BTreeMap};
use std::time::{SystemTime, Duration};

/// Core synchronization management system
pub struct SyncManager {
    /// Pending local changes to be synchronized
    pending_changes: VecDeque<SyncEvent>,
    /// Recently applied changes from peers
    applied_changes: VecDeque<AppliedChange>,
    /// Conflict resolution state
    conflict_resolver: ConflictResolver,
    /// World state tracking
    world_state: WorldStateTracker,
    /// Synchronization statistics
    sync_stats: SyncStats,
    /// Configuration
    config: SyncConfig,
}

impl SyncManager {
    pub fn new() -> Self {
        Self {
            pending_changes: VecDeque::new(),
            applied_changes: VecDeque::new(),
            conflict_resolver: ConflictResolver::new(),
            world_state: WorldStateTracker::new(),
            sync_stats: SyncStats::default(),
            config: SyncConfig::default(),
        }
    }

    /// Apply a local change and prepare for synchronization
    pub fn apply_change(&mut self, event: SyncEvent) -> RobinResult<()> {
        // Add timestamp if not present
        let mut event = event;
        if let SyncEvent::VoxelPlaced { timestamp, .. } |
           SyncEvent::VoxelRemoved { timestamp, .. } |
           SyncEvent::StructurePlaced { timestamp, .. } |
           SyncEvent::BlueprintApplied { timestamp, .. } = &mut event {
            if *timestamp == SystemTime::UNIX_EPOCH {
                *timestamp = SystemTime::now();
            }
        }

        // Update world state
        self.world_state.apply_event(&event)?;

        // Add to pending changes for network synchronization
        self.pending_changes.push_back(event.clone());

        // Update statistics
        self.sync_stats.local_changes_applied += 1;

        Ok(())
    }

    /// Apply a change received from a remote peer
    pub fn apply_remote_change(&mut self, event: SyncEvent) -> RobinResult<()> {
        // Check for conflicts with recent local changes
        if let Some(conflict) = self.detect_conflict(&event) {
            let resolution = self.conflict_resolver.resolve_conflict(conflict)?;
            self.apply_conflict_resolution(resolution)?;
        } else {
            // No conflict, apply directly
            self.world_state.apply_event(&event)?;

            let applied = AppliedChange {
                event,
                applied_at: SystemTime::now(),
                source: ChangeSource::Remote,
            };

            self.applied_changes.push_back(applied);
        }

        self.sync_stats.remote_changes_applied += 1;
        Ok(())
    }

    /// Update synchronization system
    pub fn update(&mut self, delta_time: f32) -> RobinResult<Vec<SyncEvent>> {
        let mut events_to_broadcast = Vec::new();

        // Process pending changes
        while let Some(change) = self.pending_changes.pop_front() {
            events_to_broadcast.push(change);
        }

        // Clean up old applied changes
        let cutoff_time = SystemTime::now() - self.config.change_history_duration;
        self.applied_changes.retain(|change| change.applied_at > cutoff_time);

        // Update conflict resolver
        self.conflict_resolver.update(delta_time)?;

        // Update world state tracker
        self.world_state.update(delta_time)?;

        Ok(events_to_broadcast)
    }

    /// Detect conflicts between events
    fn detect_conflict(&self, incoming_event: &SyncEvent) -> Option<Conflict> {
        let incoming_position = match incoming_event {
            SyncEvent::VoxelPlaced { position, .. } |
            SyncEvent::VoxelRemoved { position, .. } => Some(*position),
            SyncEvent::StructurePlaced { position, .. } => Some(*position),
            _ => None,
        };

        if let Some(pos) = incoming_position {
            // Check recent local changes for conflicts at the same position
            for applied in &self.applied_changes {
                if applied.source == ChangeSource::Local {
                    let applied_position = match &applied.event {
                        SyncEvent::VoxelPlaced { position, .. } |
                        SyncEvent::VoxelRemoved { position, .. } => Some(*position),
                        SyncEvent::StructurePlaced { position, .. } => Some(*position),
                        _ => None,
                    };

                    if applied_position == Some(pos) {
                        // Check if changes are within conflict window
                        let time_diff = incoming_event.get_timestamp()
                            .duration_since(applied.event.get_timestamp())
                            .unwrap_or_default();

                        if time_diff < self.config.conflict_detection_window {
                            return Some(Conflict {
                                position: pos,
                                local_event: applied.event.clone(),
                                remote_event: incoming_event.clone(),
                                detected_at: SystemTime::now(),
                            });
                        }
                    }
                }
            }
        }

        None
    }

    /// Apply conflict resolution
    fn apply_conflict_resolution(&mut self, resolution: ConflictResolution) -> RobinResult<()> {
        match resolution.strategy {
            ConflictStrategy::LastWriteWins => {
                // Apply the event with the most recent timestamp
                let event_to_apply = if resolution.conflict.local_event.get_timestamp() >
                                         resolution.conflict.remote_event.get_timestamp() {
                    &resolution.conflict.local_event
                } else {
                    &resolution.conflict.remote_event
                };

                self.world_state.apply_event(event_to_apply)?;
            }
            ConflictStrategy::MergeChanges => {
                // Attempt to merge both changes if possible
                if let Some(merged_event) = self.attempt_merge(&resolution.conflict) {
                    self.world_state.apply_event(&merged_event)?;
                } else {
                    // Fall back to last write wins
                    let event_to_apply = &resolution.conflict.remote_event;
                    self.world_state.apply_event(event_to_apply)?;
                }
            }
            ConflictStrategy::UserDecision => {
                // In real implementation, would present conflict to users for resolution
                // For now, default to remote wins
                self.world_state.apply_event(&resolution.conflict.remote_event)?;
            }
            ConflictStrategy::RoleBasedPriority => {
                // Higher role wins - would need user role information
                // For now, default to remote wins
                self.world_state.apply_event(&resolution.conflict.remote_event)?;
            }
        }

        self.sync_stats.conflicts_resolved += 1;
        Ok(())
    }

    /// Attempt to merge conflicting changes
    fn attempt_merge(&self, conflict: &Conflict) -> Option<SyncEvent> {
        match (&conflict.local_event, &conflict.remote_event) {
            // If both are placing different voxel types at same position, prefer certain materials
            (SyncEvent::VoxelPlaced { voxel_type: local_type, position, user_id, .. },
             SyncEvent::VoxelPlaced { voxel_type: remote_type, .. }) => {
                // Simple merge: prefer stone over earth, metal over stone, etc.
                let winning_type = self.get_preferred_voxel_type(*local_type, *remote_type);
                Some(SyncEvent::VoxelPlaced {
                    user_id: user_id.clone(),
                    voxel_type: winning_type,
                    position: *position,
                    timestamp: SystemTime::now(),
                })
            }
            _ => None, // Can't merge these types of changes
        }
    }

    /// Get preferred voxel type for merging
    fn get_preferred_voxel_type(&self, type1: VoxelType, type2: VoxelType) -> VoxelType {
        // Priority: Stone > Earth > Sand > Grass > Air
        let priority = |t: VoxelType| match t {
            VoxelType::Stone => 4,
            VoxelType::Earth => 3,
            VoxelType::Sand => 2,
            VoxelType::Grass => 1,
            VoxelType::Air => 0,
            VoxelType::Water => 0,
        };

        if priority(type1) >= priority(type2) { type1 } else { type2 }
    }

    /// Get recent activity for display
    pub fn get_recent_activity(&self) -> Vec<String> {
        self.applied_changes.iter()
            .rev()
            .take(5)
            .map(|change| format!("{}: {}",
                change.applied_at.duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default().as_secs(),
                change.event.to_string()))
            .collect()
    }

    /// Get synchronization event count
    pub fn get_event_count(&self) -> usize {
        self.sync_stats.local_changes_applied + self.sync_stats.remote_changes_applied
    }

    /// Get conflict count
    pub fn get_conflict_count(&self) -> usize {
        self.sync_stats.conflicts_resolved
    }

    /// Create a change set for batch operations
    pub fn create_change_set(&mut self, description: String) -> ChangeSet {
        ChangeSet {
            id: uuid::Uuid::new_v4().to_string(),
            description,
            changes: Vec::new(),
            created_at: SystemTime::now(),
            created_by: String::new(), // Would be filled by caller
        }
    }

    /// Apply a change set atomically
    pub fn apply_change_set(&mut self, change_set: ChangeSet) -> RobinResult<()> {
        // In a real implementation, this would be atomic
        for change in change_set.changes {
            self.apply_change(change)?;
        }

        self.sync_stats.change_sets_applied += 1;
        Ok(())
    }
}

/// Events that can be synchronized between peers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncEvent {
    VoxelPlaced {
        user_id: String,
        voxel_type: VoxelType,
        position: Vec3,
        timestamp: SystemTime,
    },
    VoxelRemoved {
        user_id: String,
        voxel_type: VoxelType, // Type that was removed
        position: Vec3,
        timestamp: SystemTime,
    },
    StructurePlaced {
        user_id: String,
        structure_id: String,
        position: Vec3,
        rotation: f32,
        timestamp: SystemTime,
    },
    BlueprintApplied {
        user_id: String,
        blueprint_id: String,
        position: Vec3,
        timestamp: SystemTime,
    },
}

impl SyncEvent {
    pub fn get_timestamp(&self) -> SystemTime {
        match self {
            SyncEvent::VoxelPlaced { timestamp, .. } |
            SyncEvent::VoxelRemoved { timestamp, .. } |
            SyncEvent::StructurePlaced { timestamp, .. } |
            SyncEvent::BlueprintApplied { timestamp, .. } => *timestamp,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            SyncEvent::VoxelPlaced { user_id, voxel_type, .. } =>
                format!("{} placed {:?}", user_id, voxel_type),
            SyncEvent::VoxelRemoved { user_id, voxel_type, .. } =>
                format!("{} removed {:?}", user_id, voxel_type),
            SyncEvent::StructurePlaced { user_id, structure_id, .. } =>
                format!("{} placed structure {}", user_id, structure_id),
            SyncEvent::BlueprintApplied { user_id, blueprint_id, .. } =>
                format!("{} applied blueprint {}", user_id, blueprint_id),
        }
    }
}

/// Conflict resolution system
#[derive(Debug, Clone)]
pub struct ConflictResolver {
    /// Active conflicts being resolved
    active_conflicts: HashMap<String, Conflict>,
    /// Resolution strategies
    strategies: HashMap<ConflictType, ConflictStrategy>,
    /// Statistics
    resolution_stats: ResolutionStats,
}

impl ConflictResolver {
    pub fn new() -> Self {
        let mut resolver = Self {
            active_conflicts: HashMap::new(),
            strategies: HashMap::new(),
            resolution_stats: ResolutionStats::default(),
        };

        resolver.initialize_strategies();
        resolver
    }

    /// Initialize default resolution strategies
    fn initialize_strategies(&mut self) {
        self.strategies.insert(ConflictType::VoxelPlacement, ConflictStrategy::LastWriteWins);
        self.strategies.insert(ConflictType::VoxelRemoval, ConflictStrategy::MergeChanges);
        self.strategies.insert(ConflictType::StructurePlacement, ConflictStrategy::UserDecision);
        self.strategies.insert(ConflictType::TerrainModification, ConflictStrategy::RoleBasedPriority);
    }

    /// Resolve a conflict
    pub fn resolve_conflict(&mut self, conflict: Conflict) -> RobinResult<ConflictResolution> {
        let conflict_type = self.classify_conflict(&conflict);
        let strategy = self.strategies.get(&conflict_type)
            .copied()
            .unwrap_or(ConflictStrategy::LastWriteWins);

        let resolution = ConflictResolution {
            conflict: conflict.clone(),
            strategy,
            resolved_at: SystemTime::now(),
        };

        // Update statistics
        self.resolution_stats.total_conflicts_resolved += 1;
        match strategy {
            ConflictStrategy::LastWriteWins => self.resolution_stats.last_write_wins += 1,
            ConflictStrategy::MergeChanges => self.resolution_stats.merge_resolutions += 1,
            ConflictStrategy::UserDecision => self.resolution_stats.user_decisions += 1,
            ConflictStrategy::RoleBasedPriority => self.resolution_stats.role_based_resolutions += 1,
        }

        Ok(resolution)
    }

    /// Classify the type of conflict
    fn classify_conflict(&self, conflict: &Conflict) -> ConflictType {
        match (&conflict.local_event, &conflict.remote_event) {
            (SyncEvent::VoxelPlaced { .. }, SyncEvent::VoxelPlaced { .. }) => ConflictType::VoxelPlacement,
            (SyncEvent::VoxelRemoved { .. }, _) | (_, SyncEvent::VoxelRemoved { .. }) => ConflictType::VoxelRemoval,
            (SyncEvent::StructurePlaced { .. }, _) | (_, SyncEvent::StructurePlaced { .. }) => ConflictType::StructurePlacement,
            _ => ConflictType::Other,
        }
    }

    /// Update conflict resolver
    pub fn update(&mut self, delta_time: f32) -> RobinResult<()> {
        // Clean up old active conflicts
        let cutoff_time = SystemTime::now() - Duration::from_secs(300); // 5 minutes
        self.active_conflicts.retain(|_, conflict| conflict.detected_at > cutoff_time);

        Ok(())
    }
}

/// World state tracking for synchronization
#[derive(Debug, Clone)]
pub struct WorldStateTracker {
    /// Recent voxel changes by position
    voxel_changes: HashMap<Vec3, VoxelState>,
    /// Structure placements
    structures: HashMap<String, StructureState>,
    /// Applied blueprints
    blueprints: HashMap<String, BlueprintState>,
}

impl WorldStateTracker {
    pub fn new() -> Self {
        Self {
            voxel_changes: HashMap::new(),
            structures: HashMap::new(),
            blueprints: HashMap::new(),
        }
    }

    /// Apply event to world state
    pub fn apply_event(&mut self, event: &SyncEvent) -> RobinResult<()> {
        match event {
            SyncEvent::VoxelPlaced { position, voxel_type, user_id, timestamp } => {
                let voxel_state = VoxelState {
                    voxel_type: *voxel_type,
                    last_modified_by: user_id.clone(),
                    last_modified_at: *timestamp,
                };
                self.voxel_changes.insert(*position, voxel_state);
            }
            SyncEvent::VoxelRemoved { position, user_id, timestamp, .. } => {
                let voxel_state = VoxelState {
                    voxel_type: VoxelType::Air,
                    last_modified_by: user_id.clone(),
                    last_modified_at: *timestamp,
                };
                self.voxel_changes.insert(*position, voxel_state);
            }
            SyncEvent::StructurePlaced { structure_id, position, user_id, timestamp, rotation } => {
                let structure_state = StructureState {
                    id: structure_id.clone(),
                    position: *position,
                    rotation: *rotation,
                    placed_by: user_id.clone(),
                    placed_at: *timestamp,
                };
                self.structures.insert(structure_id.clone(), structure_state);
            }
            SyncEvent::BlueprintApplied { blueprint_id, position, user_id, timestamp } => {
                let blueprint_state = BlueprintState {
                    id: blueprint_id.clone(),
                    position: *position,
                    applied_by: user_id.clone(),
                    applied_at: *timestamp,
                };
                self.blueprints.insert(blueprint_id.clone(), blueprint_state);
            }
        }

        Ok(())
    }

    /// Update world state tracker
    pub fn update(&mut self, delta_time: f32) -> RobinResult<()> {
        // Clean up old voxel changes to prevent memory growth
        let cutoff_time = SystemTime::now() - Duration::from_secs(3600); // 1 hour
        self.voxel_changes.retain(|_, state| state.last_modified_at > cutoff_time);

        Ok(())
    }
}

/// Applied change tracking
#[derive(Debug, Clone)]
pub struct AppliedChange {
    pub event: SyncEvent,
    pub applied_at: SystemTime,
    pub source: ChangeSource,
}

/// Source of a change
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeSource {
    Local,
    Remote,
}

/// Conflict between local and remote changes
#[derive(Debug, Clone)]
pub struct Conflict {
    pub position: Vec3,
    pub local_event: SyncEvent,
    pub remote_event: SyncEvent,
    pub detected_at: SystemTime,
}

/// Conflict resolution result
#[derive(Debug, Clone)]
pub struct ConflictResolution {
    pub conflict: Conflict,
    pub strategy: ConflictStrategy,
    pub resolved_at: SystemTime,
}

/// Different conflict resolution strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictStrategy {
    LastWriteWins,      // Most recent change wins
    MergeChanges,       // Attempt to merge both changes
    UserDecision,       // Present conflict to users
    RoleBasedPriority,  // Higher role wins
}

/// Types of conflicts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConflictType {
    VoxelPlacement,
    VoxelRemoval,
    StructurePlacement,
    TerrainModification,
    Other,
}

/// Batch of related changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSet {
    pub id: String,
    pub description: String,
    pub changes: Vec<SyncEvent>,
    pub created_at: SystemTime,
    pub created_by: String,
}

/// Voxel state tracking
#[derive(Debug, Clone)]
pub struct VoxelState {
    pub voxel_type: VoxelType,
    pub last_modified_by: String,
    pub last_modified_at: SystemTime,
}

/// Structure state tracking
#[derive(Debug, Clone)]
pub struct StructureState {
    pub id: String,
    pub position: Vec3,
    pub rotation: f32,
    pub placed_by: String,
    pub placed_at: SystemTime,
}

/// Blueprint state tracking
#[derive(Debug, Clone)]
pub struct BlueprintState {
    pub id: String,
    pub position: Vec3,
    pub applied_by: String,
    pub applied_at: SystemTime,
}

/// Synchronization configuration
#[derive(Debug, Clone)]
pub struct SyncConfig {
    pub change_history_duration: Duration,
    pub conflict_detection_window: Duration,
    pub max_pending_changes: usize,
    pub batch_size: usize,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            change_history_duration: Duration::from_secs(3600), // 1 hour
            conflict_detection_window: Duration::from_secs(5),   // 5 seconds
            max_pending_changes: 1000,
            batch_size: 50,
        }
    }
}

/// Synchronization statistics
#[derive(Debug, Clone, Default)]
pub struct SyncStats {
    pub local_changes_applied: usize,
    pub remote_changes_applied: usize,
    pub conflicts_resolved: usize,
    pub change_sets_applied: usize,
    pub average_sync_latency: f32,
}

/// Conflict resolution statistics
#[derive(Debug, Clone, Default)]
pub struct ResolutionStats {
    pub total_conflicts_resolved: usize,
    pub last_write_wins: usize,
    pub merge_resolutions: usize,
    pub user_decisions: usize,
    pub role_based_resolutions: usize,
}

impl Default for SyncManager {
    fn default() -> Self {
        Self::new()
    }
}