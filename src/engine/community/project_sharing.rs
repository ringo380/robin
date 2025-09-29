// Robin Engine Project Sharing System
// Enables users to share and collaborate on voxel construction projects

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, Duration};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use tokio::fs;

use crate::engine::{
    error::{RobinResult, RobinError},
    math::{Vec3, Transform},
    generation::voxel_system::VoxelWorld,
    platform::cloud_saves::SaveData,
    save_system::SaveManager,
    build_mode::BuildModeState,
};

use super::{CommunityFeature, CommunityDataStore, CommunityEvent};

/// Project sharing manager for collaborative building
pub struct ProjectSharingManager {
    /// Shared projects database
    projects: HashMap<Uuid, SharedProject>,

    /// Project categories
    categories: HashMap<String, ProjectCategory>,

    /// User project permissions
    permissions: HashMap<Uuid, ProjectPermissions>,

    /// Project search index
    search_index: ProjectSearchIndex,

    /// Storage path for project files
    storage_path: PathBuf,

    /// Feature enabled state
    enabled: bool,
}

impl ProjectSharingManager {
    /// Create a new project sharing manager
    pub fn new() -> RobinResult<Self> {
        let storage_path = PathBuf::from("community_data/projects");

        Ok(Self {
            projects: HashMap::new(),
            categories: Self::create_default_categories(),
            permissions: HashMap::new(),
            search_index: ProjectSearchIndex::new(),
            storage_path,
            enabled: true,
        })
    }

    /// Share a new project
    pub async fn share_project(
        &mut self,
        owner_id: Uuid,
        project_data: ProjectData,
        metadata: ProjectMetadata,
    ) -> RobinResult<Uuid> {
        if !self.enabled {
            return Err(RobinError::Community("Project sharing is disabled".to_string()));
        }

        let project_id = Uuid::new_v4();
        let now = SystemTime::now();

        // Clone title for logging before moving metadata
        let project_title = metadata.title.clone();

        // Create shared project
        let shared_project = SharedProject {
            id: project_id,
            owner_id,
            title: metadata.title.clone(),
            description: metadata.description.clone(),
            category: metadata.category.clone(),
            tags: metadata.tags.clone(),
            created_at: now,
            updated_at: now,
            data: project_data,
            metadata,
            stats: ProjectStats::new(),
            permissions: ProjectPermissionLevel::Public,
            featured: false,
        };

        // Save project data to disk
        self.save_project_data(&shared_project).await?;

        // Add to projects database
        self.projects.insert(project_id, shared_project.clone());

        // Update search index
        self.search_index.add_project(&shared_project);

        // Set owner permissions
        let owner_permissions = ProjectPermissions {
            project_id,
            user_id: owner_id,
            permission_level: UserPermissionLevel::Owner,
            granted_at: now,
        };
        self.permissions.insert(project_id, owner_permissions);

        log::info!("📦 Project '{}' shared by user {}", project_title, owner_id);

        Ok(project_id)
    }

    /// Get a shared project
    pub fn get_project(&self, project_id: &Uuid) -> Option<&SharedProject> {
        self.projects.get(project_id)
    }

    /// Search for projects
    pub fn search_projects(&self, query: &ProjectSearchQuery) -> Vec<&SharedProject> {
        self.search_index.search(query, &self.projects)
    }

    /// Get projects by category
    pub fn get_projects_by_category(&self, category: &str) -> Vec<&SharedProject> {
        self.projects
            .values()
            .filter(|p| p.category == category)
            .collect()
    }

    /// Get user's projects
    pub fn get_user_projects(&self, user_id: &Uuid) -> Vec<&SharedProject> {
        self.projects
            .values()
            .filter(|p| p.owner_id == *user_id)
            .collect()
    }

    /// Get featured projects
    pub fn get_featured_projects(&self) -> Vec<&SharedProject> {
        self.projects
            .values()
            .filter(|p| p.featured)
            .collect()
    }

    /// Get trending projects (most viewed/liked recently)
    pub fn get_trending_projects(&self, limit: usize) -> Vec<&SharedProject> {
        let mut projects: Vec<&SharedProject> = self.projects.values().collect();

        // Sort by recent engagement (views + likes in last 7 days)
        projects.sort_by(|a, b| {
            let a_score = a.stats.calculate_trending_score();
            let b_score = b.stats.calculate_trending_score();
            b_score.partial_cmp(&a_score).unwrap()
        });

        projects.into_iter().take(limit).collect()
    }

    /// Like a project
    pub fn like_project(&mut self, project_id: &Uuid, user_id: Uuid) -> RobinResult<()> {
        if let Some(project) = self.projects.get_mut(project_id) {
            project.stats.add_like(user_id);
            Ok(())
        } else {
            Err(RobinError::Community(format!("Project {} not found", project_id)))
        }
    }

    /// View a project (increment view count)
    pub fn view_project(&mut self, project_id: &Uuid, user_id: Option<Uuid>) -> RobinResult<()> {
        if let Some(project) = self.projects.get_mut(project_id) {
            project.stats.add_view(user_id);
            Ok(())
        } else {
            Err(RobinError::Community(format!("Project {} not found", project_id)))
        }
    }

    /// Download a project
    pub async fn download_project(&mut self, project_id: &Uuid, user_id: Uuid) -> RobinResult<ProjectData> {
        // Check permissions first (requires immutable borrow)
        if !self.can_user_download(project_id, &user_id) {
            return Err(RobinError::Community("Permission denied".to_string()));
        }

        // Now get mutable reference and update stats
        if let Some(project) = self.projects.get_mut(project_id) {
            project.stats.add_download(user_id);
            Ok(project.data.clone())
        } else {
            Err(RobinError::Community(format!("Project {} not found", project_id)))
        }
    }

    /// Check if user can download project
    fn can_user_download(&self, project_id: &Uuid, user_id: &Uuid) -> bool {
        if let Some(project) = self.projects.get(project_id) {
            match project.permissions {
                ProjectPermissionLevel::Public => true,
                ProjectPermissionLevel::FriendsOnly => {
                    // TODO: Implement friend system
                    project.owner_id == *user_id
                }
                ProjectPermissionLevel::Private => project.owner_id == *user_id,
            }
        } else {
            false
        }
    }

    /// Delete a project
    pub async fn delete_project(&mut self, project_id: &Uuid, user_id: Uuid) -> RobinResult<()> {
        if let Some(project) = self.projects.get(project_id) {
            // Check if user is owner
            if project.owner_id != user_id {
                return Err(RobinError::Community("Only project owner can delete project".to_string()));
            }

            // Remove from disk
            self.delete_project_data(project_id).await?;

            // Remove from memory
            self.projects.remove(project_id);
            self.permissions.remove(project_id);
            self.search_index.remove_project(project_id);

            log::info!("🗑️ Project {} deleted by user {}", project_id, user_id);
            Ok(())
        } else {
            Err(RobinError::Community(format!("Project {} not found", project_id)))
        }
    }

    /// Get project count
    pub fn get_project_count(&self) -> u64 {
        self.projects.len() as u64
    }

    /// Create default project categories
    fn create_default_categories() -> HashMap<String, ProjectCategory> {
        let mut categories = HashMap::new();

        let category_data = vec![
            ("Architecture", "Buildings, structures, and architectural designs", "🏗️"),
            ("Landscapes", "Natural terrains, gardens, and outdoor environments", "🌄"),
            ("Sculptures", "Artistic creations and decorative structures", "🗿"),
            ("Vehicles", "Cars, planes, ships, and other transportation", "🚗"),
            ("Fantasy", "Magical realms, castles, and fantastical creations", "🏰"),
            ("Modern", "Contemporary designs and urban environments", "🏙️"),
            ("Pixel Art", "Voxel-based pixel art and retro designs", "🎨"),
            ("Games", "Game worlds, levels, and interactive environments", "🎮"),
            ("Educational", "Learning environments and educational content", "📚"),
            ("Community", "Collaborative builds and community projects", "👥"),
        ];

        for (name, description, icon) in category_data {
            let category = ProjectCategory {
                name: name.to_string(),
                description: description.to_string(),
                icon: icon.to_string(),
                project_count: 0,
                featured: false,
            };
            categories.insert(name.to_string(), category);
        }

        categories
    }

    /// Save project data to disk
    async fn save_project_data(&self, project: &SharedProject) -> RobinResult<()> {
        // Create storage directory if it doesn't exist
        fs::create_dir_all(&self.storage_path).await
            .map_err(|e| RobinError::IO(format!("Failed to create storage directory: {}", e)))?;

        // Serialize project data
        let project_file = self.storage_path.join(format!("{}.json", project.id));
        let serialized = serde_json::to_string_pretty(project)
            .map_err(|e| RobinError::Serialization(format!("Failed to serialize project: {}", e)))?;

        // Write to file
        fs::write(project_file, serialized).await
            .map_err(|e| RobinError::IO(format!("Failed to write project file: {}", e)))?;

        Ok(())
    }

    /// Delete project data from disk
    async fn delete_project_data(&self, project_id: &Uuid) -> RobinResult<()> {
        let project_file = self.storage_path.join(format!("{}.json", project_id));

        if project_file.exists() {
            fs::remove_file(project_file).await
                .map_err(|e| RobinError::IO(format!("Failed to delete project file: {}", e)))?;
        }

        Ok(())
    }
}

impl CommunityFeature for ProjectSharingManager {
    async fn initialize(&mut self) -> RobinResult<()> {
        log::info!("📦 Initializing Project Sharing System");

        // Create storage directory
        fs::create_dir_all(&self.storage_path).await
            .map_err(|e| RobinError::IO(format!("Failed to create storage directory: {}", e)))?;

        // Load existing projects
        self.load_existing_projects().await?;

        log::info!("✅ Project Sharing System initialized with {} projects", self.projects.len());
        Ok(())
    }

    async fn shutdown(&mut self) -> RobinResult<()> {
        log::info!("📦 Shutting down Project Sharing System");

        // Save all projects before shutdown
        for project in self.projects.values() {
            self.save_project_data(project).await?;
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "ProjectSharingManager"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl ProjectSharingManager {
    /// Load existing projects from disk
    async fn load_existing_projects(&mut self) -> RobinResult<()> {
        if !self.storage_path.exists() {
            return Ok(());
        }

        let mut entries = fs::read_dir(&self.storage_path).await
            .map_err(|e| RobinError::IO(format!("Failed to read storage directory: {}", e)))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| RobinError::IO(format!("Failed to read directory entry: {}", e)))? {

            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match self.load_project_from_file(&path).await {
                    Ok(project) => {
                        self.search_index.add_project(&project);
                        self.projects.insert(project.id, project);
                    }
                    Err(e) => {
                        log::warn!("Failed to load project from {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Load a single project from file
    async fn load_project_from_file(&self, path: &Path) -> RobinResult<SharedProject> {
        let contents = fs::read_to_string(path).await
            .map_err(|e| RobinError::IO(format!("Failed to read project file: {}", e)))?;

        let project: SharedProject = serde_json::from_str(&contents)
            .map_err(|e| RobinError::Serialization(format!("Failed to deserialize project: {}", e)))?;

        Ok(project)
    }
}

/// Shared project data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedProject {
    /// Unique project identifier
    pub id: Uuid,

    /// Project owner
    pub owner_id: Uuid,

    /// Project title
    pub title: String,

    /// Project description
    pub description: String,

    /// Project category
    pub category: String,

    /// Project tags
    pub tags: Vec<String>,

    /// Creation timestamp
    pub created_at: SystemTime,

    /// Last update timestamp
    pub updated_at: SystemTime,

    /// Project voxel data
    pub data: ProjectData,

    /// Project metadata
    pub metadata: ProjectMetadata,

    /// Project statistics
    pub stats: ProjectStats,

    /// Permission level
    pub permissions: ProjectPermissionLevel,

    /// Whether project is featured
    pub featured: bool,
}

/// Project voxel data and build information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectData {
    /// Voxel world data
    pub world_data: Vec<u8>, // Serialized voxel world

    /// Build mode state
    pub build_state: Option<String>, // Serialized build mode state

    /// Project dimensions
    pub dimensions: Vec3,

    /// Spawn point
    pub spawn_point: Transform,

    /// Preview image data
    pub preview_image: Option<Vec<u8>>,

    /// Construction time
    pub build_time: Duration,

    /// Block count
    pub block_count: u32,

    /// Complexity score
    pub complexity_score: f32,
}

/// Project metadata and information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    /// Project title
    pub title: String,

    /// Detailed description
    pub description: String,

    /// Category
    pub category: String,

    /// Tags for searchability
    pub tags: Vec<String>,

    /// Difficulty level
    pub difficulty: DifficultyLevel,

    /// Estimated build time
    pub estimated_time: Duration,

    /// Required materials
    pub materials: Vec<String>,

    /// Instructions or notes
    pub instructions: Option<String>,

    /// Version number
    pub version: String,

    /// Collaboration enabled
    pub collaboration_enabled: bool,
}

/// Project difficulty levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DifficultyLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
    Master,
}

/// Project statistics and engagement metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStats {
    /// View count
    pub views: u64,

    /// Like count
    pub likes: u64,

    /// Download count
    pub downloads: u64,

    /// Users who liked this project
    pub liked_by: HashSet<Uuid>,

    /// Users who downloaded this project
    pub downloaded_by: HashSet<Uuid>,

    /// Recent view timestamps
    pub recent_views: Vec<SystemTime>,

    /// Average rating
    pub average_rating: f32,

    /// Number of ratings
    pub rating_count: u32,
}

impl ProjectStats {
    pub fn new() -> Self {
        Self {
            views: 0,
            likes: 0,
            downloads: 0,
            liked_by: HashSet::new(),
            downloaded_by: HashSet::new(),
            recent_views: Vec::new(),
            average_rating: 0.0,
            rating_count: 0,
        }
    }

    pub fn add_view(&mut self, user_id: Option<Uuid>) {
        self.views += 1;
        self.recent_views.push(SystemTime::now());

        // Keep only last 30 days of views for trending calculation
        let thirty_days_ago = SystemTime::now() - Duration::from_secs(30 * 24 * 3600);
        self.recent_views.retain(|&time| time > thirty_days_ago);
    }

    pub fn add_like(&mut self, user_id: Uuid) {
        if self.liked_by.insert(user_id) {
            self.likes += 1;
        }
    }

    pub fn add_download(&mut self, user_id: Uuid) {
        if self.downloaded_by.insert(user_id) {
            self.downloads += 1;
        }
    }

    /// Calculate trending score based on recent engagement
    pub fn calculate_trending_score(&self) -> f32 {
        let now = SystemTime::now();
        let seven_days_ago = now - Duration::from_secs(7 * 24 * 3600);

        // Count recent views
        let recent_views = self.recent_views
            .iter()
            .filter(|&&time| time > seven_days_ago)
            .count() as f32;

        // Weight: views * 1.0 + likes * 2.0 + downloads * 3.0
        recent_views + (self.likes as f32 * 2.0) + (self.downloads as f32 * 3.0)
    }
}

/// Project permission levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProjectPermissionLevel {
    Public,     // Anyone can view and download
    FriendsOnly, // Only friends can view and download
    Private,    // Only owner can access
}

/// User permissions for a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectPermissions {
    pub project_id: Uuid,
    pub user_id: Uuid,
    pub permission_level: UserPermissionLevel,
    pub granted_at: SystemTime,
}

/// User permission levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserPermissionLevel {
    Owner,       // Full control
    Collaborator, // Can edit and share
    Viewer,      // Can view only
}

/// Project category information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectCategory {
    pub name: String,
    pub description: String,
    pub icon: String,
    pub project_count: u64,
    pub featured: bool,
}

/// Project search functionality
pub struct ProjectSearchIndex {
    /// Title index for fast text search
    title_index: HashMap<String, HashSet<Uuid>>,

    /// Tag index for tag-based search
    tag_index: HashMap<String, HashSet<Uuid>>,

    /// Category index
    category_index: HashMap<String, HashSet<Uuid>>,
}

impl ProjectSearchIndex {
    pub fn new() -> Self {
        Self {
            title_index: HashMap::new(),
            tag_index: HashMap::new(),
            category_index: HashMap::new(),
        }
    }

    /// Add project to search index
    pub fn add_project(&mut self, project: &SharedProject) {
        // Index title words
        for word in project.title.to_lowercase().split_whitespace() {
            self.title_index
                .entry(word.to_string())
                .or_insert_with(HashSet::new)
                .insert(project.id);
        }

        // Index tags
        for tag in &project.tags {
            self.tag_index
                .entry(tag.to_lowercase())
                .or_insert_with(HashSet::new)
                .insert(project.id);
        }

        // Index category
        self.category_index
            .entry(project.category.to_lowercase())
            .or_insert_with(HashSet::new)
            .insert(project.id);
    }

    /// Remove project from search index
    pub fn remove_project(&mut self, project_id: &Uuid) {
        // Remove from all indices
        for (_, project_set) in self.title_index.iter_mut() {
            project_set.remove(project_id);
        }

        for (_, project_set) in self.tag_index.iter_mut() {
            project_set.remove(project_id);
        }

        for (_, project_set) in self.category_index.iter_mut() {
            project_set.remove(project_id);
        }

        // Clean up empty entries
        self.title_index.retain(|_, set| !set.is_empty());
        self.tag_index.retain(|_, set| !set.is_empty());
        self.category_index.retain(|_, set| !set.is_empty());
    }

    /// Search projects based on query
    pub fn search<'a>(
        &self,
        query: &ProjectSearchQuery,
        projects: &'a HashMap<Uuid, SharedProject>,
    ) -> Vec<&'a SharedProject> {
        let mut matching_ids: Option<HashSet<Uuid>> = None;

        // Search by title
        if let Some(title_query) = &query.title {
            let mut title_matches = HashSet::new();
            for word in title_query.to_lowercase().split_whitespace() {
                if let Some(project_ids) = self.title_index.get(word) {
                    if title_matches.is_empty() {
                        title_matches = project_ids.clone();
                    } else {
                        title_matches = title_matches.intersection(project_ids).cloned().collect();
                    }
                }
            }

            matching_ids = Some(title_matches);
        }

        // Search by tags
        if !query.tags.is_empty() {
            let mut tag_matches = HashSet::new();
            for tag in &query.tags {
                if let Some(project_ids) = self.tag_index.get(&tag.to_lowercase()) {
                    tag_matches.extend(project_ids);
                }
            }

            matching_ids = if let Some(existing) = matching_ids {
                Some(existing.intersection(&tag_matches).cloned().collect())
            } else {
                Some(tag_matches)
            };
        }

        // Search by category
        if let Some(category) = &query.category {
            if let Some(category_matches) = self.category_index.get(&category.to_lowercase()) {
                matching_ids = if let Some(existing) = matching_ids {
                    Some(existing.intersection(category_matches).cloned().collect())
                } else {
                    Some(category_matches.clone())
                };
            }
        }

        // If no search criteria, return all projects
        let project_ids = matching_ids.unwrap_or_else(|| {
            projects.keys().cloned().collect()
        });

        // Convert IDs to project references and apply filters
        let mut results: Vec<&SharedProject> = project_ids
            .iter()
            .filter_map(|id| projects.get(id))
            .filter(|project| {
                // Filter by difficulty
                if let Some(difficulty) = &query.difficulty {
                    if project.metadata.difficulty != *difficulty {
                        return false;
                    }
                }

                // Filter by minimum likes
                if let Some(min_likes) = query.min_likes {
                    if project.stats.likes < min_likes {
                        return false;
                    }
                }

                true
            })
            .collect();

        // Sort results
        match query.sort_by {
            ProjectSortBy::Newest => {
                results.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            }
            ProjectSortBy::Oldest => {
                results.sort_by(|a, b| a.created_at.cmp(&b.created_at));
            }
            ProjectSortBy::MostLiked => {
                results.sort_by(|a, b| b.stats.likes.cmp(&a.stats.likes));
            }
            ProjectSortBy::MostViewed => {
                results.sort_by(|a, b| b.stats.views.cmp(&a.stats.views));
            }
            ProjectSortBy::Trending => {
                results.sort_by(|a, b| {
                    let a_score = a.stats.calculate_trending_score();
                    let b_score = b.stats.calculate_trending_score();
                    b_score.partial_cmp(&a_score).unwrap()
                });
            }
        }

        // Apply limit
        if let Some(limit) = query.limit {
            results.truncate(limit);
        }

        results
    }
}

/// Project search query parameters
#[derive(Debug, Clone)]
pub struct ProjectSearchQuery {
    /// Search in title
    pub title: Option<String>,

    /// Filter by tags
    pub tags: Vec<String>,

    /// Filter by category
    pub category: Option<String>,

    /// Filter by difficulty
    pub difficulty: Option<DifficultyLevel>,

    /// Minimum number of likes
    pub min_likes: Option<u64>,

    /// Sort order
    pub sort_by: ProjectSortBy,

    /// Result limit
    pub limit: Option<usize>,
}

impl Default for ProjectSearchQuery {
    fn default() -> Self {
        Self {
            title: None,
            tags: Vec::new(),
            category: None,
            difficulty: None,
            min_likes: None,
            sort_by: ProjectSortBy::Newest,
            limit: Some(50),
        }
    }
}

/// Project sorting options
#[derive(Debug, Clone, PartialEq)]
pub enum ProjectSortBy {
    Newest,
    Oldest,
    MostLiked,
    MostViewed,
    Trending,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_project_sharing_manager() {
        let mut manager = ProjectSharingManager::new().unwrap();
        let owner_id = Uuid::new_v4();

        let project_data = ProjectData {
            world_data: vec![1, 2, 3, 4],
            build_state: None,
            dimensions: Vec3::new(10.0, 10.0, 10.0),
            spawn_point: Transform::default(),
            preview_image: None,
            build_time: Duration::from_secs(3600),
            block_count: 100,
            complexity_score: 7.5,
        };

        let metadata = ProjectMetadata {
            title: "Test Project".to_string(),
            description: "A test project".to_string(),
            category: "Architecture".to_string(),
            tags: vec!["test".to_string(), "demo".to_string()],
            difficulty: DifficultyLevel::Beginner,
            estimated_time: Duration::from_secs(1800),
            materials: vec!["Stone".to_string()],
            instructions: None,
            version: "1.0".to_string(),
            collaboration_enabled: true,
        };

        let project_id = manager.share_project(owner_id, project_data, metadata).await.unwrap();

        // Test project retrieval
        let project = manager.get_project(&project_id);
        assert!(project.is_some());
        assert_eq!(project.unwrap().title, "Test Project");

        // Test search
        let query = ProjectSearchQuery {
            title: Some("Test".to_string()),
            ..Default::default()
        };
        let results = manager.search_projects(&query);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_project_stats() {
        let mut stats = ProjectStats::new();
        let user_id = Uuid::new_v4();

        stats.add_view(Some(user_id));
        stats.add_like(user_id);

        assert_eq!(stats.views, 1);
        assert_eq!(stats.likes, 1);
        assert!(stats.liked_by.contains(&user_id));
    }

    #[test]
    fn test_search_index() {
        let mut index = ProjectSearchIndex::new();
        let project_id = Uuid::new_v4();

        let project = SharedProject {
            id: project_id,
            owner_id: Uuid::new_v4(),
            title: "Amazing Castle".to_string(),
            description: "Test".to_string(),
            category: "Architecture".to_string(),
            tags: vec!["castle".to_string(), "medieval".to_string()],
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
            data: ProjectData {
                world_data: vec![],
                build_state: None,
                dimensions: Vec3::new(1.0, 1.0, 1.0),
                spawn_point: Transform::default(),
                preview_image: None,
                build_time: Duration::from_secs(0),
                block_count: 0,
                complexity_score: 0.0,
            },
            metadata: ProjectMetadata {
                title: "Amazing Castle".to_string(),
                description: "Test".to_string(),
                category: "Architecture".to_string(),
                tags: vec!["castle".to_string()],
                difficulty: DifficultyLevel::Beginner,
                estimated_time: Duration::from_secs(0),
                materials: vec![],
                instructions: None,
                version: "1.0".to_string(),
                collaboration_enabled: false,
            },
            stats: ProjectStats::new(),
            permissions: ProjectPermissionLevel::Public,
            featured: false,
        };

        index.add_project(&project);

        // Test title search
        assert!(index.title_index.get("amazing").unwrap().contains(&project_id));
        assert!(index.title_index.get("castle").unwrap().contains(&project_id));

        // Test tag search
        assert!(index.tag_index.get("castle").unwrap().contains(&project_id));
        assert!(index.tag_index.get("medieval").unwrap().contains(&project_id));

        // Test category search
        assert!(index.category_index.get("architecture").unwrap().contains(&project_id));
    }
}