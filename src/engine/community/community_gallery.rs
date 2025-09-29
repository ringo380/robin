// Robin Engine Community Gallery
// Showcase and discovery platform for user creations

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use tokio::sync::broadcast;

use crate::engine::{
    error::{RobinResult, RobinError},
    math::Vec3,
    generation::voxel_system::VoxelWorld,
    platform::cloud_saves::SaveData,
};

/// Community gallery for showcasing user creations
pub struct CommunityGallery {
    /// Gallery submissions
    submissions: HashMap<Uuid, GallerySubmission>,

    /// Categories and collections
    categories: HashMap<String, GalleryCategory>,
    featured_collections: Vec<FeaturedCollection>,

    /// Voting and rating system
    votes: HashMap<Uuid, Vec<Vote>>,
    ratings: HashMap<Uuid, RatingStats>,

    /// Search and discovery
    search_index: GallerySearchIndex,
    trending_calculator: TrendingCalculator,

    /// Event broadcasting
    event_sender: broadcast::Sender<GalleryEvent>,

    /// Storage configuration
    storage_path: PathBuf,
    max_file_size: u64,

    /// Feature configuration
    config: GalleryConfig,
    enabled: bool,
}

impl CommunityGallery {
    /// Create a new community gallery
    pub fn new() -> RobinResult<Self> {
        let (event_sender, _) = broadcast::channel(1000);

        Ok(Self {
            submissions: HashMap::new(),
            categories: Self::create_default_categories(),
            featured_collections: Vec::new(),
            votes: HashMap::new(),
            ratings: HashMap::new(),
            search_index: GallerySearchIndex::new(),
            trending_calculator: TrendingCalculator::new(),
            event_sender,
            storage_path: PathBuf::from("data/gallery"),
            max_file_size: 100 * 1024 * 1024, // 100MB
            config: GalleryConfig::default(),
            enabled: true,
        })
    }

    /// Initialize the gallery system
    pub async fn initialize(&mut self) -> RobinResult<()> {
        if !self.enabled {
            return Ok(());
        }

        log::info!("🎨 Initializing Community Gallery");

        // Create storage directories
        std::fs::create_dir_all(&self.storage_path)
            .map_err(|e| RobinError::Community(format!("Failed to create gallery directory: {}", e)))?;

        std::fs::create_dir_all(self.storage_path.join("submissions"))
            .map_err(|e| RobinError::Community(format!("Failed to create submissions directory: {}", e)))?;

        std::fs::create_dir_all(self.storage_path.join("thumbnails"))
            .map_err(|e| RobinError::Community(format!("Failed to create thumbnails directory: {}", e)))?;

        // Load existing submissions
        self.load_submissions().await?;

        // Initialize featured collections
        self.create_featured_collections().await?;

        log::info!("✅ Community Gallery initialized with {} submissions", self.submissions.len());
        Ok(())
    }

    /// Submit a creation to the gallery
    pub async fn submit_creation(&mut self, params: SubmissionParams) -> RobinResult<Uuid> {
        if !self.enabled {
            return Err(RobinError::Community("Gallery disabled".to_string()));
        }

        // Validate submission
        self.validate_submission(&params)?;

        // Check user submission limits
        self.check_user_submission_limits(&params.creator_id)?;

        let submission_id = Uuid::new_v4();
        let now = SystemTime::now();

        // Process and store the submission
        let file_path = self.save_submission_file(&submission_id, &params.world_data).await?;
        let thumbnail_path = self.generate_thumbnail(&submission_id, &params.world_data).await?;

        // Calculate complexity score before moving params
        let complexity_score = self.calculate_complexity_score(&params);

        let submission = GallerySubmission {
            id: submission_id,
            title: params.title,
            description: params.description,
            creator_id: params.creator_id,
            category: params.category,
            tags: params.tags,
            file_path,
            thumbnail_path,
            file_size: params.world_data.len() as u64,
            creation_time: now,
            submission_time: now,
            last_updated: now,
            view_count: 0,
            download_count: 0,
            like_count: 0,
            feature_count: 0,
            submission_status: SubmissionStatus::Pending,
            visibility: params.visibility,
            license: params.license.unwrap_or(LicenseType::CreativeCommons),
            build_time: params.build_time,
            voxel_count: params.voxel_count,
            complexity_score,
            metadata: SubmissionMetadata {
                engine_version: env!("CARGO_PKG_VERSION").to_string(),
                build_mode_used: params.build_mode_used,
                ai_assisted: params.ai_assisted.unwrap_or(false),
                collaboration_count: params.collaboration_count.unwrap_or(1),
                original_room_id: params.original_room_id,
            },
        };

        // Extract data for event and logging before moving submission
        let creator_id = submission.creator_id;
        let title = submission.title.clone();
        let category = submission.category.clone();

        // Add to gallery
        self.submissions.insert(submission_id, submission);

        // Update search index
        self.search_index.add_submission(&self.submissions[&submission_id]);

        // Initialize voting/rating structures
        self.votes.insert(submission_id, Vec::new());
        self.ratings.insert(submission_id, RatingStats::new());

        // Save submission
        self.save_submission(&submission_id).await?;

        // Broadcast event
        let event = GalleryEvent::SubmissionAdded {
            submission_id,
            creator_id,
            title: title.clone(),
            category,
        };
        self.broadcast_event(event);

        log::info!("🎨 Gallery submission '{}' added ({})", title, submission_id);
        Ok(submission_id)
    }

    /// Browse gallery submissions
    pub fn browse_submissions(&self, filter: BrowseFilter) -> Vec<SubmissionSummary> {
        let mut submissions: Vec<&GallerySubmission> = self.submissions.values()
            .filter(|s| self.submission_matches_filter(s, &filter))
            .collect();

        // Apply sorting
        match filter.sort_by {
            SortBy::Newest => submissions.sort_by(|a, b| b.submission_time.cmp(&a.submission_time)),
            SortBy::Popular => submissions.sort_by(|a, b| b.like_count.cmp(&a.like_count)),
            SortBy::Trending => {
                let trending_scores = self.trending_calculator.calculate_trending_scores(&submissions);
                submissions.sort_by(|a, b| {
                    let score_a = trending_scores.get(&a.id).unwrap_or(&0.0);
                    let score_b = trending_scores.get(&b.id).unwrap_or(&0.0);
                    score_b.partial_cmp(score_a).unwrap_or(std::cmp::Ordering::Equal)
                });
            },
            SortBy::MostViewed => submissions.sort_by(|a, b| b.view_count.cmp(&a.view_count)),
            SortBy::Rating => {
                submissions.sort_by(|a, b| {
                    let rating_a = self.ratings.get(&a.id).map(|r| r.average_rating).unwrap_or(0.0);
                    let rating_b = self.ratings.get(&b.id).map(|r| r.average_rating).unwrap_or(0.0);
                    rating_b.partial_cmp(&rating_a).unwrap_or(std::cmp::Ordering::Equal)
                });
            },
        }

        // Apply pagination
        let start = filter.offset.unwrap_or(0);
        let limit = filter.limit.unwrap_or(20).min(100); // Max 100 results

        submissions.into_iter()
            .skip(start)
            .take(limit)
            .map(|s| self.create_submission_summary(s))
            .collect()
    }

    /// Search gallery submissions
    pub fn search_submissions(&self, query: &str, limit: usize) -> Vec<SubmissionSummary> {
        let submission_ids = self.search_index.search(query, limit);

        submission_ids.into_iter()
            .filter_map(|id| self.submissions.get(&id))
            .map(|s| self.create_submission_summary(s))
            .collect()
    }

    /// Get submission details
    pub async fn get_submission(&mut self, submission_id: Uuid, viewer_id: Option<Uuid>) -> RobinResult<GallerySubmission> {
        // First update the submission
        let submission_clone = {
            let submission = self.submissions.get_mut(&submission_id)
                .ok_or_else(|| RobinError::Community("Submission not found".to_string()))?;

            // Increment view count
            submission.view_count += 1;
            submission.last_updated = SystemTime::now();

            submission.clone()
        }; // Mutable borrow ends here

        // Track viewer if provided (now we can borrow self mutably again)
        if let Some(viewer) = viewer_id {
            self.track_view(submission_id, viewer).await?;
        }

        Ok(submission_clone)
    }

    /// Vote on a submission (like/dislike)
    pub async fn vote_submission(&mut self, user_id: Uuid, submission_id: Uuid, vote_type: VoteType) -> RobinResult<()> {
        if !self.submissions.contains_key(&submission_id) {
            return Err(RobinError::Community("Submission not found".to_string()));
        }

        let votes = self.votes.entry(submission_id).or_insert_with(Vec::new);

        // Remove existing vote from this user
        votes.retain(|v| v.user_id != user_id);

        // Add new vote
        let vote = Vote {
            user_id,
            vote_type: vote_type.clone(),
            timestamp: SystemTime::now(),
        };
        votes.push(vote);

        // Update submission like count
        if let Some(submission) = self.submissions.get_mut(&submission_id) {
            submission.like_count = votes.iter().filter(|v| v.vote_type == VoteType::Like).count() as u64;
        }

        // Broadcast event
        let event = GalleryEvent::SubmissionVoted {
            submission_id,
            user_id,
            vote_type,
            new_like_count: self.submissions[&submission_id].like_count,
        };
        self.broadcast_event(event);

        Ok(())
    }

    /// Rate a submission
    pub async fn rate_submission(&mut self, user_id: Uuid, submission_id: Uuid, rating: f32) -> RobinResult<()> {
        if rating < 1.0 || rating > 5.0 {
            return Err(RobinError::Community("Rating must be between 1.0 and 5.0".to_string()));
        }

        if !self.submissions.contains_key(&submission_id) {
            return Err(RobinError::Community("Submission not found".to_string()));
        }

        let rating_stats = self.ratings.entry(submission_id).or_insert_with(RatingStats::new);

        // Update rating
        rating_stats.add_rating(user_id, rating);

        // Broadcast event
        let event = GalleryEvent::SubmissionRated {
            submission_id,
            user_id,
            rating,
            new_average: rating_stats.average_rating,
        };
        self.broadcast_event(event);

        Ok(())
    }

    /// Download a submission
    pub async fn download_submission(&mut self, submission_id: Uuid, downloader_id: Uuid) -> RobinResult<Vec<u8>> {
        // First get file data and update submission in a scope
        let (file_data, new_download_count) = {
            let submission = self.submissions.get_mut(&submission_id)
                .ok_or_else(|| RobinError::Community("Submission not found".to_string()))?;

            // Check download permissions
            if submission.visibility == Visibility::Private {
                if downloader_id != submission.creator_id {
                    return Err(RobinError::Community("Insufficient permissions".to_string()));
                }
            }

            // Read file
            let file_data = std::fs::read(&submission.file_path)
                .map_err(|e| RobinError::Community(format!("Failed to read submission file: {}", e)))?;

            // Increment download count
            submission.download_count += 1;
            submission.last_updated = SystemTime::now();

            (file_data, submission.download_count)
        }; // Mutable borrow ends here

        // Track download (now we can borrow self mutably again)
        self.track_download(submission_id, downloader_id).await?;

        // Broadcast event
        let event = GalleryEvent::SubmissionDownloaded {
            submission_id,
            downloader_id,
            new_download_count,
        };
        self.broadcast_event(event);

        Ok(file_data)
    }

    /// Get featured submissions
    pub fn get_featured_submissions(&self) -> Vec<SubmissionSummary> {
        self.submissions.values()
            .filter(|s| s.submission_status == SubmissionStatus::Featured)
            .map(|s| self.create_submission_summary(s))
            .collect()
    }

    /// Get trending submissions
    pub fn get_trending_submissions(&self, limit: usize) -> Vec<SubmissionSummary> {
        let submissions: Vec<&GallerySubmission> = self.submissions.values().collect();
        let trending_scores = self.trending_calculator.calculate_trending_scores(&submissions);

        let mut scored_submissions: Vec<(f64, &GallerySubmission)> = submissions.into_iter()
            .map(|s| (*trending_scores.get(&s.id).unwrap_or(&0.0), s))
            .collect();

        scored_submissions.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        scored_submissions.into_iter()
            .take(limit)
            .map(|(_, s)| self.create_submission_summary(s))
            .collect()
    }

    /// Get gallery statistics
    pub fn get_item_count(&self) -> u64 {
        self.submissions.len() as u64
    }

    /// Get user's submissions
    pub fn get_user_submissions(&self, user_id: Uuid) -> Vec<SubmissionSummary> {
        self.submissions.values()
            .filter(|s| s.creator_id == user_id)
            .map(|s| self.create_submission_summary(s))
            .collect()
    }

    // Helper methods

    fn create_default_categories() -> HashMap<String, GalleryCategory> {
        let mut categories = HashMap::new();

        categories.insert("Architecture".to_string(), GalleryCategory {
            name: "Architecture".to_string(),
            description: "Buildings, structures, and architectural marvels".to_string(),
            icon: "🏛️".to_string(),
            color: "#8B4513".to_string(),
        });

        categories.insert("Vehicles".to_string(), GalleryCategory {
            name: "Vehicles".to_string(),
            description: "Cars, ships, planes, and other vehicles".to_string(),
            icon: "🚗".to_string(),
            color: "#FF6B35".to_string(),
        });

        categories.insert("Art".to_string(), GalleryCategory {
            name: "Art".to_string(),
            description: "Sculptures, pixel art, and creative expressions".to_string(),
            icon: "🎨".to_string(),
            color: "#9B59B6".to_string(),
        });

        categories.insert("Mechanical".to_string(), GalleryCategory {
            name: "Mechanical".to_string(),
            description: "Machines, contraptions, and mechanical devices".to_string(),
            icon: "⚙️".to_string(),
            color: "#34495E".to_string(),
        });

        categories.insert("Landscapes".to_string(), GalleryCategory {
            name: "Landscapes".to_string(),
            description: "Terrain, gardens, and natural environments".to_string(),
            icon: "🌄".to_string(),
            color: "#27AE60".to_string(),
        });

        categories.insert("Games".to_string(), GalleryCategory {
            name: "Games".to_string(),
            description: "Playable games and interactive experiences".to_string(),
            icon: "🎮".to_string(),
            color: "#E74C3C".to_string(),
        });

        categories
    }

    async fn create_featured_collections(&mut self) -> RobinResult<()> {
        // Create weekly featured collection
        let weekly_featured = FeaturedCollection {
            id: Uuid::new_v4(),
            name: "Weekly Spotlight".to_string(),
            description: "This week's most impressive creations".to_string(),
            curator_id: None, // System curated
            submission_ids: Vec::new(),
            created_at: SystemTime::now(),
            expires_at: Some(SystemTime::now() + Duration::from_secs(7 * 24 * 60 * 60)), // 1 week
        };

        self.featured_collections.push(weekly_featured);
        Ok(())
    }

    fn validate_submission(&self, params: &SubmissionParams) -> RobinResult<()> {
        if params.title.trim().is_empty() {
            return Err(RobinError::Community("Title cannot be empty".to_string()));
        }

        if params.title.len() > 100 {
            return Err(RobinError::Community("Title too long".to_string()));
        }

        if params.description.len() > 2000 {
            return Err(RobinError::Community("Description too long".to_string()));
        }

        if !self.categories.contains_key(&params.category) {
            return Err(RobinError::Community("Invalid category".to_string()));
        }

        if params.world_data.len() as u64 > self.max_file_size {
            return Err(RobinError::Community("File too large".to_string()));
        }

        Ok(())
    }

    fn check_user_submission_limits(&self, user_id: &Uuid) -> RobinResult<()> {
        let user_submissions = self.submissions.values()
            .filter(|s| s.creator_id == *user_id)
            .count();

        if user_submissions >= self.config.max_submissions_per_user {
            return Err(RobinError::Community("User submission limit reached".to_string()));
        }

        Ok(())
    }

    async fn save_submission_file(&self, submission_id: &Uuid, data: &[u8]) -> RobinResult<PathBuf> {
        let file_path = self.storage_path
            .join("submissions")
            .join(format!("{}.world", submission_id));

        std::fs::write(&file_path, data)
            .map_err(|e| RobinError::Community(format!("Failed to save submission file: {}", e)))?;

        Ok(file_path)
    }

    async fn generate_thumbnail(&self, submission_id: &Uuid, _world_data: &[u8]) -> RobinResult<PathBuf> {
        let thumbnail_path = self.storage_path
            .join("thumbnails")
            .join(format!("{}.png", submission_id));

        // TODO: Implement thumbnail generation from world data
        // For now, create a placeholder
        std::fs::write(&thumbnail_path, b"placeholder_thumbnail")
            .map_err(|e| RobinError::Community(format!("Failed to create thumbnail: {}", e)))?;

        Ok(thumbnail_path)
    }

    fn calculate_complexity_score(&self, params: &SubmissionParams) -> f32 {
        let mut score = 0.0;

        // Base score from voxel count
        score += (params.voxel_count as f32).log10() * 10.0;

        // Build time factor
        if let Some(build_time) = params.build_time {
            score += (build_time.as_secs() as f32 / 3600.0) * 5.0; // Hours to score
        }

        // AI assistance penalty (manual builds score higher)
        if params.ai_assisted == Some(true) {
            score *= 0.8;
        }

        // Collaboration bonus
        if let Some(collab_count) = params.collaboration_count {
            if collab_count > 1 {
                score *= 1.0 + (collab_count as f32 * 0.1);
            }
        }

        score.min(100.0).max(0.0)
    }

    fn submission_matches_filter(&self, submission: &GallerySubmission, filter: &BrowseFilter) -> bool {
        if let Some(ref category) = filter.category {
            if submission.category != *category {
                return false;
            }
        }

        if let Some(ref creator) = filter.creator_id {
            if submission.creator_id != *creator {
                return false;
            }
        }

        if let Some(ref status) = filter.status {
            if submission.submission_status != *status {
                return false;
            }
        }

        if submission.visibility == Visibility::Private {
            return false;
        }

        true
    }

    fn create_submission_summary(&self, submission: &GallerySubmission) -> SubmissionSummary {
        let rating_stats = self.ratings.get(&submission.id);

        SubmissionSummary {
            id: submission.id,
            title: submission.title.clone(),
            description: submission.description.clone(),
            creator_id: submission.creator_id,
            category: submission.category.clone(),
            tags: submission.tags.clone(),
            thumbnail_path: submission.thumbnail_path.clone(),
            submission_time: submission.submission_time,
            view_count: submission.view_count,
            download_count: submission.download_count,
            like_count: submission.like_count,
            average_rating: rating_stats.map(|r| r.average_rating).unwrap_or(0.0),
            rating_count: rating_stats.map(|r| r.rating_count).unwrap_or(0),
            complexity_score: submission.complexity_score,
            voxel_count: submission.voxel_count,
            file_size: submission.file_size,
        }
    }

    async fn track_view(&self, _submission_id: Uuid, _viewer_id: Uuid) -> RobinResult<()> {
        // TODO: Implement view tracking for analytics
        Ok(())
    }

    async fn track_download(&self, _submission_id: Uuid, _downloader_id: Uuid) -> RobinResult<()> {
        // TODO: Implement download tracking for analytics
        Ok(())
    }

    fn broadcast_event(&self, event: GalleryEvent) {
        if let Err(e) = self.event_sender.send(event) {
            log::warn!("Failed to broadcast gallery event: {}", e);
        }
    }

    async fn save_submission(&self, _submission_id: &Uuid) -> RobinResult<()> {
        // TODO: Implement submission metadata persistence
        Ok(())
    }

    async fn load_submissions(&mut self) -> RobinResult<()> {
        // TODO: Implement submission loading
        Ok(())
    }
}

/// Gallery submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GallerySubmission {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub creator_id: Uuid,
    pub category: String,
    pub tags: Vec<String>,
    pub file_path: PathBuf,
    pub thumbnail_path: PathBuf,
    pub file_size: u64,
    pub creation_time: SystemTime,
    pub submission_time: SystemTime,
    pub last_updated: SystemTime,
    pub view_count: u64,
    pub download_count: u64,
    pub like_count: u64,
    pub feature_count: u64,
    pub submission_status: SubmissionStatus,
    pub visibility: Visibility,
    pub license: LicenseType,
    pub build_time: Option<Duration>,
    pub voxel_count: u64,
    pub complexity_score: f32,
    pub metadata: SubmissionMetadata,
}

/// Submission parameters
#[derive(Debug, Clone)]
pub struct SubmissionParams {
    pub title: String,
    pub description: String,
    pub creator_id: Uuid,
    pub category: String,
    pub tags: Vec<String>,
    pub world_data: Vec<u8>,
    pub visibility: Visibility,
    pub license: Option<LicenseType>,
    pub build_time: Option<Duration>,
    pub voxel_count: u64,
    pub build_mode_used: Option<String>,
    pub ai_assisted: Option<bool>,
    pub collaboration_count: Option<u32>,
    pub original_room_id: Option<Uuid>,
}

/// Submission summary for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmissionSummary {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub creator_id: Uuid,
    pub category: String,
    pub tags: Vec<String>,
    pub thumbnail_path: PathBuf,
    pub submission_time: SystemTime,
    pub view_count: u64,
    pub download_count: u64,
    pub like_count: u64,
    pub average_rating: f32,
    pub rating_count: u32,
    pub complexity_score: f32,
    pub voxel_count: u64,
    pub file_size: u64,
}

/// Browse filter for gallery
#[derive(Debug, Clone, Default)]
pub struct BrowseFilter {
    pub category: Option<String>,
    pub creator_id: Option<Uuid>,
    pub status: Option<SubmissionStatus>,
    pub sort_by: SortBy,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}

/// Gallery categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalleryCategory {
    pub name: String,
    pub description: String,
    pub icon: String,
    pub color: String,
}

/// Featured collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturedCollection {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub curator_id: Option<Uuid>,
    pub submission_ids: Vec<Uuid>,
    pub created_at: SystemTime,
    pub expires_at: Option<SystemTime>,
}

/// Submission metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmissionMetadata {
    pub engine_version: String,
    pub build_mode_used: Option<String>,
    pub ai_assisted: bool,
    pub collaboration_count: u32,
    pub original_room_id: Option<Uuid>,
}

/// Submission status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubmissionStatus {
    Pending,
    Approved,
    Featured,
    Rejected,
    Hidden,
}

/// Visibility settings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Unlisted,
    Private,
}

/// License types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LicenseType {
    CreativeCommons,
    PublicDomain,
    AllRightsReserved,
    MIT,
    GPL,
}

/// Sort options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    Newest,
    Popular,
    Trending,
    MostViewed,
    Rating,
}

impl Default for SortBy {
    fn default() -> Self {
        SortBy::Newest
    }
}

/// Vote on submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub user_id: Uuid,
    pub vote_type: VoteType,
    pub timestamp: SystemTime,
}

/// Vote types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoteType {
    Like,
    Dislike,
}

/// Rating statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatingStats {
    pub average_rating: f32,
    pub rating_count: u32,
    pub rating_distribution: [u32; 5], // 1-star to 5-star counts
    pub user_ratings: HashMap<Uuid, f32>,
}

impl RatingStats {
    pub fn new() -> Self {
        Self {
            average_rating: 0.0,
            rating_count: 0,
            rating_distribution: [0; 5],
            user_ratings: HashMap::new(),
        }
    }

    pub fn add_rating(&mut self, user_id: Uuid, rating: f32) {
        let previous_rating = self.user_ratings.insert(user_id, rating);

        // Update distribution
        if let Some(prev) = previous_rating {
            let prev_index = (prev - 1.0) as usize;
            if prev_index < 5 {
                self.rating_distribution[prev_index] = self.rating_distribution[prev_index].saturating_sub(1);
            }
        } else {
            self.rating_count += 1;
        }

        let rating_index = (rating - 1.0) as usize;
        if rating_index < 5 {
            self.rating_distribution[rating_index] += 1;
        }

        // Recalculate average
        self.recalculate_average();
    }

    fn recalculate_average(&mut self) {
        if self.rating_count == 0 {
            self.average_rating = 0.0;
            return;
        }

        let total_score: f32 = self.rating_distribution.iter()
            .enumerate()
            .map(|(i, &count)| (i + 1) as f32 * count as f32)
            .sum();

        self.average_rating = total_score / self.rating_count as f32;
    }
}

/// Gallery events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GalleryEvent {
    SubmissionAdded {
        submission_id: Uuid,
        creator_id: Uuid,
        title: String,
        category: String,
    },
    SubmissionVoted {
        submission_id: Uuid,
        user_id: Uuid,
        vote_type: VoteType,
        new_like_count: u64,
    },
    SubmissionRated {
        submission_id: Uuid,
        user_id: Uuid,
        rating: f32,
        new_average: f32,
    },
    SubmissionDownloaded {
        submission_id: Uuid,
        downloader_id: Uuid,
        new_download_count: u64,
    },
    SubmissionFeatured {
        submission_id: Uuid,
        featured_by: Uuid,
    },
}

/// Gallery search index
pub struct GallerySearchIndex {
    keyword_map: HashMap<String, HashSet<Uuid>>,
    tag_map: HashMap<String, HashSet<Uuid>>,
    category_map: HashMap<String, HashSet<Uuid>>,
}

impl GallerySearchIndex {
    pub fn new() -> Self {
        Self {
            keyword_map: HashMap::new(),
            tag_map: HashMap::new(),
            category_map: HashMap::new(),
        }
    }

    pub fn add_submission(&mut self, submission: &GallerySubmission) {
        // Add keywords from title and description
        let keywords = self.extract_keywords(submission);
        for keyword in keywords {
            self.keyword_map.entry(keyword.to_lowercase())
                .or_insert_with(HashSet::new)
                .insert(submission.id);
        }

        // Add tags
        for tag in &submission.tags {
            self.tag_map.entry(tag.to_lowercase())
                .or_insert_with(HashSet::new)
                .insert(submission.id);
        }

        // Add category
        self.category_map.entry(submission.category.to_lowercase())
            .or_insert_with(HashSet::new)
            .insert(submission.id);
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<Uuid> {
        let query_words: Vec<&str> = query.to_lowercase().split_whitespace().collect();
        let mut submission_scores: HashMap<Uuid, usize> = HashMap::new();

        for word in query_words {
            // Search in keywords
            if let Some(submission_ids) = self.keyword_map.get(word) {
                for submission_id in submission_ids {
                    *submission_scores.entry(*submission_id).or_insert(0) += 2; // Higher weight for keywords
                }
            }

            // Search in tags
            if let Some(submission_ids) = self.tag_map.get(word) {
                for submission_id in submission_ids {
                    *submission_scores.entry(*submission_id).or_insert(0) += 3; // Highest weight for tags
                }
            }

            // Search in categories
            if let Some(submission_ids) = self.category_map.get(word) {
                for submission_id in submission_ids {
                    *submission_scores.entry(*submission_id).or_insert(0) += 1; // Lower weight for categories
                }
            }
        }

        let mut results: Vec<(Uuid, usize)> = submission_scores.into_iter().collect();
        results.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by score descending

        results.into_iter()
            .take(limit)
            .map(|(submission_id, _)| submission_id)
            .collect()
    }

    fn extract_keywords(&self, submission: &GallerySubmission) -> Vec<String> {
        let mut keywords = Vec::new();

        // Title words
        keywords.extend(submission.title.split_whitespace().map(|s| s.to_string()));

        // Description words
        keywords.extend(submission.description.split_whitespace().map(|s| s.to_string()));

        keywords
    }
}

/// Trending calculator
pub struct TrendingCalculator {
    time_decay_factor: f64,
    view_weight: f64,
    like_weight: f64,
    download_weight: f64,
    rating_weight: f64,
}

impl TrendingCalculator {
    pub fn new() -> Self {
        Self {
            time_decay_factor: 0.8,
            view_weight: 1.0,
            like_weight: 3.0,
            download_weight: 5.0,
            rating_weight: 2.0,
        }
    }

    pub fn calculate_trending_scores(&self, submissions: &[&GallerySubmission]) -> HashMap<Uuid, f64> {
        let now = SystemTime::now();
        let mut scores = HashMap::new();

        for submission in submissions {
            let time_diff = now.duration_since(submission.submission_time)
                .unwrap_or(Duration::from_secs(0))
                .as_secs() as f64 / 3600.0; // Hours

            let time_factor = (self.time_decay_factor).powf(time_diff / 24.0); // Daily decay

            let engagement_score =
                submission.view_count as f64 * self.view_weight +
                submission.like_count as f64 * self.like_weight +
                submission.download_count as f64 * self.download_weight;

            let score = engagement_score * time_factor;
            scores.insert(submission.id, score);
        }

        scores
    }
}

/// Gallery configuration
#[derive(Debug, Clone)]
pub struct GalleryConfig {
    pub max_submissions_per_user: usize,
    pub enable_ratings: bool,
    pub enable_comments: bool,
    pub auto_generate_thumbnails: bool,
    pub featured_rotation_days: u32,
}

impl Default for GalleryConfig {
    fn default() -> Self {
        Self {
            max_submissions_per_user: 50,
            enable_ratings: true,
            enable_comments: true,
            auto_generate_thumbnails: true,
            featured_rotation_days: 7,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gallery_creation() {
        let mut gallery = CommunityGallery::new().unwrap();
        gallery.initialize().await.unwrap();

        assert!(gallery.categories.contains_key("Architecture"));
        assert!(gallery.categories.contains_key("Art"));
    }

    #[tokio::test]
    async fn test_submission_workflow() {
        let mut gallery = CommunityGallery::new().unwrap();
        gallery.initialize().await.unwrap();

        let creator_id = Uuid::new_v4();
        let params = SubmissionParams {
            title: "Test Building".to_string(),
            description: "A test creation".to_string(),
            creator_id,
            category: "Architecture".to_string(),
            tags: vec!["test".to_string(), "building".to_string()],
            world_data: vec![1, 2, 3, 4], // Mock world data
            visibility: Visibility::Public,
            license: Some(LicenseType::CreativeCommons),
            build_time: Some(Duration::from_secs(3600)),
            voxel_count: 1000,
            build_mode_used: Some("Creative".to_string()),
            ai_assisted: Some(false),
            collaboration_count: Some(1),
            original_room_id: None,
        };

        let submission_id = gallery.submit_creation(params).await.unwrap();
        assert!(gallery.submissions.contains_key(&submission_id));

        let submission = &gallery.submissions[&submission_id];
        assert_eq!(submission.title, "Test Building");
        assert_eq!(submission.creator_id, creator_id);
    }

    #[tokio::test]
    async fn test_voting_system() {
        let mut gallery = CommunityGallery::new().unwrap();
        gallery.initialize().await.unwrap();

        let creator_id = Uuid::new_v4();
        let voter_id = Uuid::new_v4();

        let params = SubmissionParams {
            title: "Test Submission".to_string(),
            description: "Test".to_string(),
            creator_id,
            category: "Art".to_string(),
            tags: vec![],
            world_data: vec![1, 2, 3],
            visibility: Visibility::Public,
            license: None,
            build_time: None,
            voxel_count: 100,
            build_mode_used: None,
            ai_assisted: None,
            collaboration_count: None,
            original_room_id: None,
        };

        let submission_id = gallery.submit_creation(params).await.unwrap();

        // Test voting
        gallery.vote_submission(voter_id, submission_id, VoteType::Like).await.unwrap();

        let submission = &gallery.submissions[&submission_id];
        assert_eq!(submission.like_count, 1);

        // Test rating
        gallery.rate_submission(voter_id, submission_id, 4.5).await.unwrap();

        let rating_stats = gallery.ratings.get(&submission_id).unwrap();
        assert_eq!(rating_stats.average_rating, 4.5);
        assert_eq!(rating_stats.rating_count, 1);
    }
}