/*!
 * Version Control and Project History System
 *
 * Manages save points, project history, branching, and rollback
 * capabilities for collaborative engineering projects.
 */

use crate::engine::{
    error::{RobinError, RobinResult},
    math::Vec3,
    world::VoxelType,
    collaboration::SyncEvent,
};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, BTreeMap};
use std::time::{SystemTime, Duration};

/// Core version control management system
pub struct VersionManager {
    /// Project history and save points
    project_history: ProjectHistory,
    /// Active save points by ID
    save_points: HashMap<String, SavePoint>,
    /// Change tracking between save points
    change_tracking: ChangeTracker,
    /// Branch management
    branches: HashMap<String, Branch>,
    /// Current active branch
    current_branch: String,
    /// Version control settings
    settings: VersionSettings,
    /// Statistics
    stats: VersionStats,
}

impl VersionManager {
    pub fn new() -> Self {
        let mut manager = Self {
            project_history: ProjectHistory::new(),
            save_points: HashMap::new(),
            change_tracking: ChangeTracker::new(),
            branches: HashMap::new(),
            current_branch: "main".to_string(),
            settings: VersionSettings::default(),
            stats: VersionStats::default(),
        };

        manager.initialize_main_branch();
        manager
    }

    /// Initialize the main branch
    fn initialize_main_branch(&mut self) {
        let main_branch = Branch {
            name: "main".to_string(),
            description: "Main project branch".to_string(),
            created_at: SystemTime::now(),
            created_by: "system".to_string(),
            parent_branch: None,
            head_save_point: None,
            is_protected: true,
        };

        self.branches.insert("main".to_string(), main_branch);
    }

    /// Create a new save point
    pub fn create_save_point(&mut self, description: String, creator_id: String) -> RobinResult<String> {
        let save_point_id = uuid::Uuid::new_v4().to_string();

        // Capture current project state
        let current_changes = self.change_tracking.get_changes_since_last_save_point();
        let world_snapshot = self.create_world_snapshot()?;

        let save_point = SavePoint {
            id: save_point_id.clone(),
            description,
            created_by: creator_id,
            created_at: SystemTime::now(),
            branch: self.current_branch.clone(),
            parent_save_point: self.get_current_head_save_point(),
            changes_since_parent: current_changes,
            world_snapshot,
            metadata: SavePointMetadata {
                total_voxels: self.count_total_voxels(),
                unique_contributors: self.get_unique_contributors(),
                session_duration: self.get_session_duration(),
                tags: Vec::new(),
            },
        };

        // Add to save points
        self.save_points.insert(save_point_id.clone(), save_point.clone());

        // Update branch head
        if let Some(branch) = self.branches.get_mut(&self.current_branch) {
            branch.head_save_point = Some(save_point_id.clone());
        }

        // Add to project history
        self.project_history.add_save_point(save_point);

        // Reset change tracking
        self.change_tracking.reset();

        // Update statistics
        self.stats.save_points_created += 1;

        Ok(save_point_id)
    }

    /// Load project state from a save point
    pub fn load_save_point(&mut self, save_point_id: &str) -> RobinResult<ProjectState> {
        let save_point = self.save_points.get(save_point_id)
            .ok_or_else(|| RobinError::NotFound("Save point not found".to_string()))?;

        // Create project state from save point
        let project_state = ProjectState {
            save_point_id: save_point_id.to_string(),
            world_state: save_point.world_snapshot.clone(),
            metadata: save_point.metadata.clone(),
            loaded_at: SystemTime::now(),
        };

        self.stats.save_points_loaded += 1;
        Ok(project_state)
    }

    /// Create a new branch from current state
    pub fn create_branch(&mut self, branch_name: String, description: String, creator_id: String) -> RobinResult<()> {
        if self.branches.contains_key(&branch_name) {
            return Err(RobinError::AlreadyExists("Branch already exists".to_string()));
        }

        let current_head = self.get_current_head_save_point();

        let branch = Branch {
            name: branch_name.clone(),
            description,
            created_at: SystemTime::now(),
            created_by: creator_id,
            parent_branch: Some(self.current_branch.clone()),
            head_save_point: current_head,
            is_protected: false,
        };

        self.branches.insert(branch_name, branch);
        self.stats.branches_created += 1;
        Ok(())
    }

    /// Switch to a different branch
    pub fn switch_branch(&mut self, branch_name: &str) -> RobinResult<()> {
        if !self.branches.contains_key(branch_name) {
            return Err(RobinError::NotFound("Branch not found".to_string()));
        }

        // Check for uncommitted changes
        if self.has_uncommitted_changes() && self.settings.require_save_before_switch {
            return Err(RobinError::UnsavedChanges("Please save changes before switching branches".to_string()));
        }

        self.current_branch = branch_name.to_string();
        Ok(())
    }

    /// Merge branch into current branch
    pub fn merge_branch(&mut self, source_branch: &str, merger_id: &str) -> RobinResult<MergeResult> {
        let source = self.branches.get(source_branch)
            .ok_or_else(|| RobinError::NotFound("Source branch not found".to_string()))?;

        let target_branch = self.branches.get(&self.current_branch)
            .ok_or_else(|| RobinError::NotFound("Target branch not found".to_string()))?;

        // Find common ancestor
        let common_ancestor = self.find_common_ancestor(
            source.head_save_point.as_ref(),
            target_branch.head_save_point.as_ref(),
        );

        // Get changes from both branches since common ancestor
        let source_changes = self.get_changes_since_save_point(&source.head_save_point, &common_ancestor)?;
        let target_changes = self.get_changes_since_save_point(&target_branch.head_save_point, &common_ancestor)?;

        // Check for conflicts
        let conflicts = self.detect_merge_conflicts(&source_changes, &target_changes);

        if conflicts.is_empty() {
            // Clean merge - apply source changes
            let merge_save_point_id = self.create_merge_save_point(
                source_branch,
                &source_changes,
                merger_id,
            )?;

            Ok(MergeResult {
                success: true,
                conflicts: Vec::new(),
                merge_save_point: Some(merge_save_point_id),
                changes_applied: source_changes.len(),
            })
        } else {
            // Conflicts detected
            Ok(MergeResult {
                success: false,
                conflicts,
                merge_save_point: None,
                changes_applied: 0,
            })
        }
    }

    /// Create a diff between two save points
    pub fn create_diff(&self, from_save_point: &str, to_save_point: &str) -> RobinResult<Diff> {
        let from_sp = self.save_points.get(from_save_point)
            .ok_or_else(|| RobinError::NotFound("From save point not found".to_string()))?;

        let to_sp = self.save_points.get(to_save_point)
            .ok_or_else(|| RobinError::NotFound("To save point not found".to_string()))?;

        let added_voxels = self.calculate_added_voxels(&from_sp.world_snapshot, &to_sp.world_snapshot);
        let removed_voxels = self.calculate_removed_voxels(&from_sp.world_snapshot, &to_sp.world_snapshot);
        let modified_voxels = self.calculate_modified_voxels(&from_sp.world_snapshot, &to_sp.world_snapshot);

        Ok(Diff {
            from_save_point: from_save_point.to_string(),
            to_save_point: to_save_point.to_string(),
            added_voxels,
            removed_voxels,
            modified_voxels,
            created_at: SystemTime::now(),
        })
    }

    /// Record a change for tracking
    pub fn record_change(&mut self, change: SyncEvent) {
        self.change_tracking.record_change(change);
    }

    /// Get project history
    pub fn get_history(&self) -> &ProjectHistory {
        &self.project_history
    }

    /// Get all save points
    pub fn get_save_points(&self) -> Vec<&SavePoint> {
        self.save_points.values().collect()
    }

    /// Get save points for specific branch
    pub fn get_branch_save_points(&self, branch_name: &str) -> Vec<&SavePoint> {
        self.save_points.values()
            .filter(|sp| sp.branch == branch_name)
            .collect()
    }

    /// Get version statistics
    pub fn get_stats(&self) -> &VersionStats {
        &self.stats
    }

    /// Check if there are uncommitted changes
    fn has_uncommitted_changes(&self) -> bool {
        !self.change_tracking.is_empty()
    }

    /// Get current head save point
    fn get_current_head_save_point(&self) -> Option<String> {
        self.branches.get(&self.current_branch)?
            .head_save_point.clone()
    }

    /// Create world snapshot
    fn create_world_snapshot(&self) -> RobinResult<WorldSnapshot> {
        // In real implementation, would capture actual world state
        Ok(WorldSnapshot {
            voxel_data: HashMap::new(), // Would contain actual voxel positions and types
            structures: HashMap::new(),  // Would contain structure placements
            terrain_modifications: Vec::new(), // Would contain terrain changes
            captured_at: SystemTime::now(),
        })
    }

    /// Count total voxels in project
    fn count_total_voxels(&self) -> u32 {
        // In real implementation, would count actual voxels
        self.change_tracking.get_total_placed_voxels()
    }

    /// Get unique contributors
    fn get_unique_contributors(&self) -> Vec<String> {
        self.change_tracking.get_unique_contributors()
    }

    /// Get session duration
    fn get_session_duration(&self) -> Duration {
        Duration::from_secs(3600) // Placeholder
    }

    /// Find common ancestor between save points
    fn find_common_ancestor(&self, save_point_1: Option<&String>, save_point_2: Option<&String>) -> Option<String> {
        // Simple implementation - would need proper ancestry tracking
        None
    }

    /// Get changes since a save point
    fn get_changes_since_save_point(&self, from: &Option<String>, to: &Option<String>) -> RobinResult<Vec<SyncEvent>> {
        // Placeholder implementation
        Ok(vec![])
    }

    /// Detect merge conflicts
    fn detect_merge_conflicts(&self, source_changes: &[SyncEvent], target_changes: &[SyncEvent]) -> Vec<MergeConflict> {
        let mut conflicts = Vec::new();

        for source_change in source_changes {
            for target_change in target_changes {
                if let Some(conflict) = self.check_changes_conflict(source_change, target_change) {
                    conflicts.push(conflict);
                }
            }
        }

        conflicts
    }

    /// Check if two changes conflict
    fn check_changes_conflict(&self, change1: &SyncEvent, change2: &SyncEvent) -> Option<MergeConflict> {
        let pos1 = self.get_change_position(change1);
        let pos2 = self.get_change_position(change2);

        if pos1 == pos2 && pos1.is_some() {
            Some(MergeConflict {
                position: pos1.unwrap(),
                source_change: change1.clone(),
                target_change: change2.clone(),
                conflict_type: MergeConflictType::PositionConflict,
            })
        } else {
            None
        }
    }

    /// Get position from change event
    fn get_change_position(&self, change: &SyncEvent) -> Option<Vec3> {
        match change {
            SyncEvent::VoxelPlaced { position, .. } |
            SyncEvent::VoxelRemoved { position, .. } |
            SyncEvent::StructurePlaced { position, .. } |
            SyncEvent::BlueprintApplied { position, .. } => Some(*position),
        }
    }

    /// Create merge save point
    fn create_merge_save_point(&mut self, source_branch: &str, changes: &[SyncEvent], merger_id: &str) -> RobinResult<String> {
        let description = format!("Merge {} into {}", source_branch, self.current_branch);
        self.create_save_point(description, merger_id.to_string())
    }

    /// Calculate added voxels between snapshots
    fn calculate_added_voxels(&self, from: &WorldSnapshot, to: &WorldSnapshot) -> Vec<VoxelChange> {
        // Placeholder implementation
        Vec::new()
    }

    /// Calculate removed voxels between snapshots
    fn calculate_removed_voxels(&self, from: &WorldSnapshot, to: &WorldSnapshot) -> Vec<VoxelChange> {
        // Placeholder implementation
        Vec::new()
    }

    /// Calculate modified voxels between snapshots
    fn calculate_modified_voxels(&self, from: &WorldSnapshot, to: &WorldSnapshot) -> Vec<VoxelChange> {
        // Placeholder implementation
        Vec::new()
    }
}

/// Project save point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavePoint {
    pub id: String,
    pub description: String,
    pub created_by: String,
    pub created_at: SystemTime,
    pub branch: String,
    pub parent_save_point: Option<String>,
    pub changes_since_parent: Vec<SyncEvent>,
    pub world_snapshot: WorldSnapshot,
    pub metadata: SavePointMetadata,
}

/// Save point metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavePointMetadata {
    pub total_voxels: u32,
    pub unique_contributors: Vec<String>,
    pub session_duration: Duration,
    pub tags: Vec<String>,
}

/// World state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSnapshot {
    pub voxel_data: HashMap<Vec3, VoxelType>,
    pub structures: HashMap<String, StructurePlacement>,
    pub terrain_modifications: Vec<TerrainChange>,
    pub captured_at: SystemTime,
}

/// Structure placement information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructurePlacement {
    pub structure_id: String,
    pub position: Vec3,
    pub rotation: f32,
    pub placed_by: String,
    pub placed_at: SystemTime,
}

/// Terrain modification information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainChange {
    pub area: Vec3, // Center of modified area
    pub radius: f32,
    pub change_type: TerrainChangeType,
    pub applied_by: String,
    pub applied_at: SystemTime,
}

/// Types of terrain modifications
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TerrainChangeType {
    Raise,
    Lower,
    Smooth,
    Sculpt,
}

/// Project history management
#[derive(Debug, Clone)]
pub struct ProjectHistory {
    /// Save points ordered by creation time
    timeline: BTreeMap<SystemTime, String>,
    /// Tags and milestones
    milestones: HashMap<String, Milestone>,
    /// History metadata
    metadata: HistoryMetadata,
}

impl ProjectHistory {
    pub fn new() -> Self {
        Self {
            timeline: BTreeMap::new(),
            milestones: HashMap::new(),
            metadata: HistoryMetadata::default(),
        }
    }

    /// Add save point to history
    pub fn add_save_point(&mut self, save_point: SavePoint) {
        self.timeline.insert(save_point.created_at, save_point.id);
        self.metadata.total_save_points += 1;
    }

    /// Add milestone
    pub fn add_milestone(&mut self, milestone: Milestone) {
        self.milestones.insert(milestone.id.clone(), milestone);
    }

    /// Get chronological timeline
    pub fn get_timeline(&self) -> Vec<(&SystemTime, &String)> {
        self.timeline.iter().collect()
    }
}

/// Project milestone
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub id: String,
    pub name: String,
    pub description: String,
    pub save_point_id: String,
    pub created_at: SystemTime,
    pub created_by: String,
}

/// History metadata
#[derive(Debug, Clone, Default)]
pub struct HistoryMetadata {
    pub total_save_points: usize,
    pub project_start_time: Option<SystemTime>,
    pub last_activity: Option<SystemTime>,
}

/// Branch information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub name: String,
    pub description: String,
    pub created_at: SystemTime,
    pub created_by: String,
    pub parent_branch: Option<String>,
    pub head_save_point: Option<String>,
    pub is_protected: bool,
}

/// Change tracking between save points
#[derive(Debug, Clone)]
pub struct ChangeTracker {
    /// Recorded changes since last save point
    changes: Vec<SyncEvent>,
    /// Statistics
    stats: ChangeStats,
}

impl ChangeTracker {
    pub fn new() -> Self {
        Self {
            changes: Vec::new(),
            stats: ChangeStats::default(),
        }
    }

    /// Record a change
    pub fn record_change(&mut self, change: SyncEvent) {
        self.changes.push(change.clone());

        // Update statistics
        match change {
            SyncEvent::VoxelPlaced { .. } => self.stats.voxels_placed += 1,
            SyncEvent::VoxelRemoved { .. } => self.stats.voxels_removed += 1,
            SyncEvent::StructurePlaced { .. } => self.stats.structures_placed += 1,
            SyncEvent::BlueprintApplied { .. } => self.stats.blueprints_applied += 1,
        }
    }

    /// Get changes since last save point
    pub fn get_changes_since_last_save_point(&self) -> Vec<SyncEvent> {
        self.changes.clone()
    }

    /// Reset change tracking
    pub fn reset(&mut self) {
        self.changes.clear();
        self.stats = ChangeStats::default();
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    /// Get total placed voxels
    pub fn get_total_placed_voxels(&self) -> u32 {
        self.stats.voxels_placed
    }

    /// Get unique contributors
    pub fn get_unique_contributors(&self) -> Vec<String> {
        let mut contributors = std::collections::HashSet::new();

        for change in &self.changes {
            match change {
                SyncEvent::VoxelPlaced { user_id, .. } |
                SyncEvent::VoxelRemoved { user_id, .. } |
                SyncEvent::StructurePlaced { user_id, .. } |
                SyncEvent::BlueprintApplied { user_id, .. } => {
                    contributors.insert(user_id.clone());
                }
            }
        }

        contributors.into_iter().collect()
    }
}

/// Change tracking statistics
#[derive(Debug, Clone, Default)]
pub struct ChangeStats {
    pub voxels_placed: u32,
    pub voxels_removed: u32,
    pub structures_placed: u32,
    pub blueprints_applied: u32,
}

/// Project state that can be loaded
#[derive(Debug, Clone)]
pub struct ProjectState {
    pub save_point_id: String,
    pub world_state: WorldSnapshot,
    pub metadata: SavePointMetadata,
    pub loaded_at: SystemTime,
}

/// Difference between two save points
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diff {
    pub from_save_point: String,
    pub to_save_point: String,
    pub added_voxels: Vec<VoxelChange>,
    pub removed_voxels: Vec<VoxelChange>,
    pub modified_voxels: Vec<VoxelChange>,
    pub created_at: SystemTime,
}

/// Individual voxel change information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoxelChange {
    pub position: Vec3,
    pub old_type: Option<VoxelType>,
    pub new_type: Option<VoxelType>,
    pub changed_by: String,
    pub changed_at: SystemTime,
}

/// Branch merge result
#[derive(Debug, Clone)]
pub struct MergeResult {
    pub success: bool,
    pub conflicts: Vec<MergeConflict>,
    pub merge_save_point: Option<String>,
    pub changes_applied: usize,
}

/// Merge conflict information
#[derive(Debug, Clone)]
pub struct MergeConflict {
    pub position: Vec3,
    pub source_change: SyncEvent,
    pub target_change: SyncEvent,
    pub conflict_type: MergeConflictType,
}

/// Types of merge conflicts
#[derive(Debug, Clone, Copy)]
pub enum MergeConflictType {
    PositionConflict,   // Two changes at same position
    StructureConflict,  // Conflicting structure placements
    TerrainConflict,    // Conflicting terrain modifications
}

/// Version control settings
#[derive(Debug, Clone)]
pub struct VersionSettings {
    pub auto_save_interval: Duration,
    pub max_save_points_per_branch: usize,
    pub require_save_before_switch: bool,
    pub compress_old_snapshots: bool,
    pub snapshot_retention_days: u32,
}

impl Default for VersionSettings {
    fn default() -> Self {
        Self {
            auto_save_interval: Duration::from_secs(1800), // 30 minutes
            max_save_points_per_branch: 100,
            require_save_before_switch: true,
            compress_old_snapshots: true,
            snapshot_retention_days: 90,
        }
    }
}

/// Version control statistics
#[derive(Debug, Clone, Default)]
pub struct VersionStats {
    pub save_points_created: usize,
    pub save_points_loaded: usize,
    pub branches_created: usize,
    pub merges_completed: usize,
    pub conflicts_resolved: usize,
    pub total_storage_used: u64, // bytes
}

impl Default for VersionManager {
    fn default() -> Self {
        Self::new()
    }
}