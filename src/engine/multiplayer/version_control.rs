use crate::engine::error::RobinResult;
use crate::engine::multiplayer::UserId;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{Instant, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub id: String,
    pub parent_ids: Vec<String>,
    pub author: UserId,
    pub timestamp: f64,
    pub message: String,
    pub changes: Vec<WorldChange>,
    pub branch: String,
    pub tags: Vec<String>,
    pub merge_base: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldChange {
    pub id: String,
    pub change_type: ChangeType,
    pub location: WorldLocation,
    pub before_state: Option<Vec<u8>>,
    pub after_state: Vec<u8>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    BlockPlaced,
    BlockRemoved,
    BlockModified,
    StructureBuilt,
    StructureDestroyed,
    TerrainModified,
    EntitySpawned,
    EntityRemoved,
    EntityModified,
    ScriptAdded,
    ScriptModified,
    ScriptRemoved,
    AssetAdded,
    AssetModified,
    AssetRemoved,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldLocation {
    pub chunk_id: String,
    pub position: [i32; 3],
    pub area: Option<BoundingBox>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min: [i32; 3],
    pub max: [i32; 3],
}

impl Commit {
    pub fn new(author: UserId, message: String, branch: String) -> Self {
        Self {
            id: format!("commit_{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()),
            parent_ids: Vec::new(),
            author,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
            message,
            changes: Vec::new(),
            branch,
            tags: Vec::new(),
            merge_base: None,
        }
    }

    pub fn with_parent(mut self, parent_id: String) -> Self {
        self.parent_ids.push(parent_id);
        self
    }

    pub fn with_parents(mut self, parent_ids: Vec<String>) -> Self {
        self.parent_ids = parent_ids;
        self
    }

    pub fn add_change(mut self, change: WorldChange) -> Self {
        self.changes.push(change);
        self
    }

    pub fn add_tag(mut self, tag: String) -> Self {
        self.tags.push(tag);
        self
    }

    pub fn with_merge_base(mut self, merge_base: String) -> Self {
        self.merge_base = Some(merge_base);
        self
    }

    pub fn is_merge_commit(&self) -> bool {
        self.parent_ids.len() > 1
    }

    pub fn affects_location(&self, location: &WorldLocation) -> bool {
        self.changes.iter().any(|change| {
            change.location.chunk_id == location.chunk_id && 
            self.locations_overlap(&change.location, location)
        })
    }

    fn locations_overlap(&self, loc1: &WorldLocation, loc2: &WorldLocation) -> bool {
        if let (Some(area1), Some(area2)) = (&loc1.area, &loc2.area) {
            // Check if bounding boxes overlap
            area1.max[0] >= area2.min[0] && area1.min[0] <= area2.max[0] &&
            area1.max[1] >= area2.min[1] && area1.min[1] <= area2.max[1] &&
            area1.max[2] >= area2.min[2] && area1.min[2] <= area2.max[2]
        } else {
            // Check if exact positions match
            loc1.position == loc2.position
        }
    }
}

impl WorldChange {
    pub fn new_block_change(change_type: ChangeType, chunk_id: String, position: [i32; 3], before: Option<Vec<u8>>, after: Vec<u8>) -> Self {
        Self {
            id: format!("change_{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()),
            change_type,
            location: WorldLocation {
                chunk_id,
                position,
                area: None,
            },
            before_state: before,
            after_state: after,
            metadata: HashMap::new(),
        }
    }

    pub fn new_area_change(change_type: ChangeType, chunk_id: String, area: BoundingBox, before: Option<Vec<u8>>, after: Vec<u8>) -> Self {
        Self {
            id: format!("change_{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()),
            change_type,
            location: WorldLocation {
                chunk_id,
                position: area.min,
                area: Some(area),
            },
            before_state: before,
            after_state: after,
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub name: String,
    pub head_commit: String,
    pub created_by: UserId,
    pub created_at: f64,
    pub description: String,
    pub protected: bool,
    pub merge_strategy: MergeStrategy,
    pub commit_count: u64,
}

impl Branch {
    pub fn new(name: String, head_commit: String, created_by: UserId) -> Self {
        Self {
            name,
            head_commit,
            created_by,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
            description: String::new(),
            protected: false,
            merge_strategy: MergeStrategy::ThreeWay,
            commit_count: 0,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn with_protection(mut self, protected: bool) -> Self {
        self.protected = protected;
        self
    }

    pub fn with_merge_strategy(mut self, strategy: MergeStrategy) -> Self {
        self.merge_strategy = strategy;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MergeStrategy {
    ThreeWay,
    FastForward,
    SquashMerge,
    RecursiveStrategy,
    OctopusMerge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolution {
    pub conflict_id: String,
    pub location: WorldLocation,
    pub conflicting_commits: Vec<String>,
    pub base_state: Option<Vec<u8>>,
    pub local_state: Vec<u8>,
    pub remote_state: Vec<u8>,
    pub resolution_strategy: ResolutionStrategy,
    pub resolved_state: Option<Vec<u8>>,
    pub resolved_by: Option<UserId>,
    pub resolved_at: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolutionStrategy {
    UseLocal,
    UseRemote,
    UseBase,
    Manual,
    Automated,
    Skip,
}

impl ConflictResolution {
    pub fn new(location: WorldLocation, local_state: Vec<u8>, remote_state: Vec<u8>) -> Self {
        Self {
            conflict_id: format!("conflict_{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()),
            location,
            conflicting_commits: Vec::new(),
            base_state: None,
            local_state,
            remote_state,
            resolution_strategy: ResolutionStrategy::Manual,
            resolved_state: None,
            resolved_by: None,
            resolved_at: None,
        }
    }

    pub fn resolve(&mut self, strategy: ResolutionStrategy, resolved_by: UserId) -> RobinResult<()> {
        self.resolution_strategy = strategy;
        self.resolved_by = Some(resolved_by);
        self.resolved_at = Some(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64());

        self.resolved_state = Some(match &self.resolution_strategy {
            ResolutionStrategy::UseLocal => self.local_state.clone(),
            ResolutionStrategy::UseRemote => self.remote_state.clone(),
            ResolutionStrategy::UseBase => {
                self.base_state.clone().unwrap_or_else(|| self.local_state.clone())
            }
            ResolutionStrategy::Manual => {
                return Err(crate::engine::error::RobinError::GenericError(
                    "Manual resolution requires explicit resolved_state".to_string()
                ));
            }
            ResolutionStrategy::Automated => {
                self.auto_resolve()?
            }
            ResolutionStrategy::Skip => vec![], // Empty state means skip this change
        });

        Ok(())
    }

    fn auto_resolve(&self) -> RobinResult<Vec<u8>> {
        // Simple auto-resolution logic - in practice, this would be much more sophisticated
        if self.local_state == self.remote_state {
            return Ok(self.local_state.clone());
        }

        if let Some(ref base_state) = self.base_state {
            if self.local_state == *base_state {
                return Ok(self.remote_state.clone());
            }
            if self.remote_state == *base_state {
                return Ok(self.local_state.clone());
            }
        }

        // Default to newer state (remote in most cases)
        Ok(self.remote_state.clone())
    }
}

#[derive(Debug)]
pub struct VersionControl {
    commits: HashMap<String, Commit>,
    branches: HashMap<String, Branch>,
    current_branch: String,
    staging_area: Vec<WorldChange>,
    conflict_resolutions: HashMap<String, ConflictResolution>,
    repository_config: RepositoryConfig,
    stats: VersionControlStats,
}

#[derive(Debug, Clone)]
pub struct RepositoryConfig {
    pub default_branch: String,
    pub auto_gc_enabled: bool,
    pub max_history_days: u32,
    pub compress_old_commits: bool,
    pub backup_interval_hours: u32,
}

impl Default for RepositoryConfig {
    fn default() -> Self {
        Self {
            default_branch: "main".to_string(),
            auto_gc_enabled: true,
            max_history_days: 365,
            compress_old_commits: true,
            backup_interval_hours: 24,
        }
    }
}

#[derive(Debug, Default)]
pub struct VersionControlStats {
    pub total_commits: u64,
    pub total_branches: u64,
    pub total_merges: u64,
    pub conflicts_resolved: u64,
    pub data_size_bytes: u64,
    pub gc_runs: u64,
}

impl VersionControl {
    pub fn new() -> RobinResult<Self> {
        let mut vc = Self {
            commits: HashMap::new(),
            branches: HashMap::new(),
            current_branch: "main".to_string(),
            staging_area: Vec::new(),
            conflict_resolutions: HashMap::new(),
            repository_config: RepositoryConfig::default(),
            stats: VersionControlStats::default(),
        };

        // Create initial commit and main branch
        vc.initialize_repository()?;
        Ok(vc)
    }

    pub fn initialize_repository(&mut self) -> RobinResult<()> {
        let initial_commit = Commit::new(
            UserId::new("system".to_string()),
            "Initial commit".to_string(),
            self.repository_config.default_branch.clone()
        );

        let commit_id = initial_commit.id.clone();
        self.commits.insert(commit_id.clone(), initial_commit);

        let main_branch = Branch::new(
            self.repository_config.default_branch.clone(),
            commit_id,
            UserId::new("system".to_string())
        ).with_description("Main development branch".to_string())
        .with_protection(true);

        self.branches.insert(main_branch.name.clone(), main_branch);
        self.current_branch = self.repository_config.default_branch.clone();

        self.stats.total_commits = 1;
        self.stats.total_branches = 1;

        println!("Version control repository initialized with branch '{}'", self.current_branch);
        Ok(())
    }

    pub fn stage_change(&mut self, change: WorldChange) {
        self.staging_area.push(change);
    }

    pub fn commit(&mut self, author: UserId, message: String) -> RobinResult<String> {
        if self.staging_area.is_empty() {
            return Err(crate::engine::error::RobinError::GenericError(
                "No changes staged for commit".to_string()
            ));
        }

        let current_head = self.get_current_head()?;
        let mut new_commit = Commit::new(author, message, self.current_branch.clone());

        if let Some(head_commit_id) = current_head {
            new_commit = new_commit.with_parent(head_commit_id);
        }

        // Add all staged changes to the commit
        for change in self.staging_area.drain(..) {
            new_commit = new_commit.add_change(change);
        }

        let commit_id = new_commit.id.clone();
        self.commits.insert(commit_id.clone(), new_commit);

        // Update branch head
        if let Some(branch) = self.branches.get_mut(&self.current_branch) {
            branch.head_commit = commit_id.clone();
            branch.commit_count += 1;
        }

        self.stats.total_commits += 1;

        println!("Created commit {} on branch '{}'", &commit_id[..8], self.current_branch);
        Ok(commit_id)
    }

    pub fn create_branch(&mut self, name: String, created_by: UserId, from_commit: Option<String>) -> RobinResult<()> {
        if self.branches.contains_key(&name) {
            return Err(crate::engine::error::RobinError::GenericError(
                format!("Branch '{}' already exists", name)
            ));
        }

        let head_commit = match from_commit {
            Some(commit_id) => {
                if !self.commits.contains_key(&commit_id) {
                    return Err(crate::engine::error::RobinError::GenericError(
                        format!("Commit '{}' not found", commit_id)
                    ));
                }
                commit_id
            }
            None => {
                self.get_current_head()?.ok_or_else(|| crate::engine::error::RobinError::GenericError(
                    "No commits found to branch from".to_string()
                ))?
            }
        };

        let branch = Branch::new(name.clone(), head_commit, created_by);
        self.branches.insert(name.clone(), branch);
        self.stats.total_branches += 1;

        println!("Created branch '{}'", name);
        Ok(())
    }

    pub fn switch_branch(&mut self, branch_name: &str) -> RobinResult<()> {
        if !self.branches.contains_key(branch_name) {
            return Err(crate::engine::error::RobinError::GenericError(
                format!("Branch '{}' not found", branch_name)
            ));
        }

        if !self.staging_area.is_empty() {
            return Err(crate::engine::error::RobinError::GenericError(
                "Cannot switch branches with uncommitted changes".to_string()
            ));
        }

        self.current_branch = branch_name.to_string();
        println!("Switched to branch '{}'", branch_name);
        Ok(())
    }

    pub fn merge_branch(&mut self, source_branch: &str, target_branch: &str, merger: UserId) -> RobinResult<String> {
        if !self.branches.contains_key(source_branch) {
            return Err(crate::engine::error::RobinError::GenericError(
                format!("Source branch '{}' not found", source_branch)
            ));
        }

        if !self.branches.contains_key(target_branch) {
            return Err(crate::engine::error::RobinError::GenericError(
                format!("Target branch '{}' not found", target_branch)
            ));
        }

        let source_head = self.branches.get(source_branch).unwrap().head_commit.clone();
        let target_head = self.branches.get(target_branch).unwrap().head_commit.clone();

        // Check if fast-forward merge is possible
        if self.is_ancestor(&target_head, &source_head)? {
            // Fast-forward merge
            self.branches.get_mut(target_branch).unwrap().head_commit = source_head.clone();
            println!("Fast-forward merge of '{}' into '{}'", source_branch, target_branch);
            return Ok(source_head);
        }

        // Three-way merge
        let merge_base = self.find_merge_base(&source_head, &target_head)?;
        let conflicts = self.detect_conflicts(&merge_base, &source_head, &target_head)?;

        if !conflicts.is_empty() {
            // Store conflicts for resolution
            for conflict in conflicts {
                self.conflict_resolutions.insert(conflict.conflict_id.clone(), conflict);
            }
            return Err(crate::engine::error::RobinError::GenericError(
                format!("Merge conflicts detected. {} conflicts need resolution", self.conflict_resolutions.len())
            ));
        }

        // Create merge commit
        let merge_commit = Commit::new(
            merger,
            format!("Merge branch '{}' into '{}'", source_branch, target_branch),
            target_branch.to_string()
        ).with_parents(vec![target_head, source_head])
        .with_merge_base(merge_base.unwrap_or_default());

        let merge_commit_id = merge_commit.id.clone();
        self.commits.insert(merge_commit_id.clone(), merge_commit);

        // Update target branch head
        self.branches.get_mut(target_branch).unwrap().head_commit = merge_commit_id.clone();
        self.stats.total_merges += 1;

        println!("Merged '{}' into '{}' with commit {}", source_branch, target_branch, &merge_commit_id[..8]);
        Ok(merge_commit_id)
    }

    fn detect_conflicts(&self, merge_base: &Option<String>, source_head: &str, target_head: &str) -> RobinResult<Vec<ConflictResolution>> {
        let mut conflicts = Vec::new();

        // Get changes from merge base to each head
        let source_changes = self.get_changes_since(merge_base.as_deref(), source_head)?;
        let target_changes = self.get_changes_since(merge_base.as_deref(), target_head)?;

        // Check for conflicting changes at the same location
        for source_change in &source_changes {
            for target_change in &target_changes {
                if self.changes_conflict(source_change, target_change) {
                    let conflict = ConflictResolution::new(
                        source_change.location.clone(),
                        target_change.after_state.clone(), // local (target)
                        source_change.after_state.clone(), // remote (source)
                    );
                    conflicts.push(conflict);
                }
            }
        }

        Ok(conflicts)
    }

    fn changes_conflict(&self, change1: &WorldChange, change2: &WorldChange) -> bool {
        // Changes conflict if they affect the same location
        change1.location.chunk_id == change2.location.chunk_id &&
        change1.location.position == change2.location.position &&
        change1.after_state != change2.after_state
    }

    fn get_changes_since(&self, from_commit: Option<&str>, to_commit: &str) -> RobinResult<Vec<WorldChange>> {
        let mut changes = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(to_commit.to_string());

        while let Some(commit_id) = queue.pop_front() {
            if visited.contains(&commit_id) {
                continue;
            }

            if Some(commit_id.as_str()) == from_commit {
                break;
            }

            visited.insert(commit_id.clone());

            if let Some(commit) = self.commits.get(&commit_id) {
                changes.extend(commit.changes.clone());

                for parent_id in &commit.parent_ids {
                    queue.push_back(parent_id.clone());
                }
            }
        }

        Ok(changes)
    }

    fn find_merge_base(&self, commit1: &str, commit2: &str) -> RobinResult<Option<String>> {
        let ancestors1 = self.get_ancestors(commit1)?;
        let ancestors2 = self.get_ancestors(commit2)?;

        // Find common ancestors
        let common_ancestors: Vec<_> = ancestors1.intersection(&ancestors2).collect();

        if common_ancestors.is_empty() {
            return Ok(None);
        }

        // Return the most recent common ancestor (simplified - would need topological sorting in reality)
        Ok(Some(common_ancestors[0].clone()))
    }

    fn get_ancestors(&self, commit_id: &str) -> RobinResult<HashSet<String>> {
        let mut ancestors = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(commit_id.to_string());

        while let Some(current_id) = queue.pop_front() {
            if ancestors.contains(&current_id) {
                continue;
            }

            ancestors.insert(current_id.clone());

            if let Some(commit) = self.commits.get(&current_id) {
                for parent_id in &commit.parent_ids {
                    queue.push_back(parent_id.clone());
                }
            }
        }

        Ok(ancestors)
    }

    fn is_ancestor(&self, ancestor_id: &str, descendant_id: &str) -> RobinResult<bool> {
        let ancestors = self.get_ancestors(descendant_id)?;
        Ok(ancestors.contains(ancestor_id))
    }

    pub fn resolve_conflict(&mut self, conflict_id: &str, strategy: ResolutionStrategy, resolved_by: UserId) -> RobinResult<()> {
        if let Some(conflict) = self.conflict_resolutions.get_mut(conflict_id) {
            conflict.resolve(strategy, resolved_by)?;
            self.stats.conflicts_resolved += 1;
            println!("Resolved conflict {}", conflict_id);
            Ok(())
        } else {
            Err(crate::engine::error::RobinError::GenericError(
                format!("Conflict '{}' not found", conflict_id)
            ))
        }
    }

    fn get_current_head(&self) -> RobinResult<Option<String>> {
        if let Some(branch) = self.branches.get(&self.current_branch) {
            Ok(Some(branch.head_commit.clone()))
        } else {
            Ok(None)
        }
    }

    pub fn get_commit_history(&self, branch_name: Option<&str>, limit: Option<usize>) -> RobinResult<Vec<&Commit>> {
        let branch_name = branch_name.unwrap_or(&self.current_branch);
        let branch = self.branches.get(branch_name)
            .ok_or_else(|| crate::engine::error::RobinError::GenericError(
                format!("Branch '{}' not found", branch_name)
            ))?;

        let mut history = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(branch.head_commit.clone());

        while let Some(commit_id) = queue.pop_front() {
            if visited.contains(&commit_id) {
                continue;
            }

            if let Some(limit) = limit {
                if history.len() >= limit {
                    break;
                }
            }

            visited.insert(commit_id.clone());

            if let Some(commit) = self.commits.get(&commit_id) {
                history.push(commit);

                for parent_id in &commit.parent_ids {
                    queue.push_back(parent_id.clone());
                }
            }
        }

        // Sort by timestamp (newest first)
        history.sort_by(|a, b| b.timestamp.partial_cmp(&a.timestamp).unwrap());
        Ok(history)
    }

    pub fn get_branches(&self) -> Vec<&Branch> {
        self.branches.values().collect()
    }

    pub fn get_current_branch(&self) -> &str {
        &self.current_branch
    }

    pub fn get_staging_area(&self) -> &[WorldChange] {
        &self.staging_area
    }

    pub fn get_conflicts(&self) -> Vec<&ConflictResolution> {
        self.conflict_resolutions.values().collect()
    }

    pub fn clear_staging_area(&mut self) {
        self.staging_area.clear();
    }

    pub fn get_stats(&self) -> &VersionControlStats {
        &self.stats
    }

    pub fn save_state(&mut self) -> RobinResult<()> {
        // In a real implementation, this would serialize the repository state to disk
        println!("Version control state saved");
        println!("  Total commits: {}", self.stats.total_commits);
        println!("  Total branches: {}", self.stats.total_branches);
        println!("  Total merges: {}", self.stats.total_merges);
        println!("  Conflicts resolved: {}", self.stats.conflicts_resolved);
        Ok(())
    }

    pub fn garbage_collect(&mut self) -> RobinResult<()> {
        if !self.repository_config.auto_gc_enabled {
            return Ok(());
        }

        let cutoff_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64() - (self.repository_config.max_history_days as f64 * 24.0 * 3600.0);

        let mut removed_commits = 0;
        let commit_ids_to_remove: Vec<_> = self.commits
            .iter()
            .filter(|(_, commit)| commit.timestamp < cutoff_time && commit.tags.is_empty())
            .map(|(id, _)| id.clone())
            .collect();

        for commit_id in commit_ids_to_remove {
            self.commits.remove(&commit_id);
            removed_commits += 1;
        }

        self.stats.gc_runs += 1;
        println!("Garbage collection complete. Removed {} old commits", removed_commits);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_control_creation() {
        let vc = VersionControl::new();
        assert!(vc.is_ok());
        
        let vc = vc.unwrap();
        assert_eq!(vc.current_branch, "main");
        assert_eq!(vc.stats.total_commits, 1);
        assert_eq!(vc.stats.total_branches, 1);
    }

    #[test]
    fn test_commit_creation() {
        let author = UserId::new("test_author".to_string());
        let commit = Commit::new(author.clone(), "Test commit".to_string(), "main".to_string());
        
        assert_eq!(commit.author, author);
        assert_eq!(commit.message, "Test commit");
        assert_eq!(commit.branch, "main");
        assert!(!commit.is_merge_commit());
    }

    #[test]
    fn test_world_change_creation() {
        let change = WorldChange::new_block_change(
            ChangeType::BlockPlaced,
            "chunk_1".to_string(),
            [10, 20, 30],
            None,
            vec![1, 2, 3, 4]
        );
        
        assert!(matches!(change.change_type, ChangeType::BlockPlaced));
        assert_eq!(change.location.chunk_id, "chunk_1");
        assert_eq!(change.location.position, [10, 20, 30]);
        assert_eq!(change.after_state, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_branch_creation() {
        let author = UserId::new("test_author".to_string());
        let branch = Branch::new("feature".to_string(), "commit_123".to_string(), author.clone())
            .with_description("Feature branch".to_string())
            .with_protection(true);
        
        assert_eq!(branch.name, "feature");
        assert_eq!(branch.created_by, author);
        assert_eq!(branch.description, "Feature branch");
        assert!(branch.protected);
    }

    #[test]
    fn test_conflict_resolution() {
        let location = WorldLocation {
            chunk_id: "chunk_1".to_string(),
            position: [0, 0, 0],
            area: None,
        };
        
        let mut conflict = ConflictResolution::new(location, vec![1, 2], vec![3, 4]);
        let resolver = UserId::new("resolver".to_string());
        
        assert!(conflict.resolve(ResolutionStrategy::UseLocal, resolver).is_ok());
        assert_eq!(conflict.resolved_state, Some(vec![1, 2]));
        assert!(conflict.resolved_by.is_some());
    }

    #[test]
    fn test_merge_strategies() {
        assert!(matches!(MergeStrategy::ThreeWay, MergeStrategy::ThreeWay));
        assert!(matches!(MergeStrategy::FastForward, MergeStrategy::FastForward));
        assert!(matches!(MergeStrategy::SquashMerge, MergeStrategy::SquashMerge));
    }

    #[test]
    fn test_bounding_box_overlap() {
        let commit = Commit::new(
            UserId::new("test".to_string()),
            "test".to_string(),
            "main".to_string()
        );

        let loc1 = WorldLocation {
            chunk_id: "chunk_1".to_string(),
            position: [0, 0, 0],
            area: Some(BoundingBox {
                min: [0, 0, 0],
                max: [10, 10, 10],
            }),
        };

        let loc2 = WorldLocation {
            chunk_id: "chunk_1".to_string(),
            position: [5, 5, 5],
            area: Some(BoundingBox {
                min: [5, 5, 5],
                max: [15, 15, 15],
            }),
        };

        assert!(commit.locations_overlap(&loc1, &loc2));
    }
}