/*!
 * Project Management System for Collaborative Engineering
 *
 * Professional project coordination, role management, task assignment,
 * and progress tracking for complex collaborative building projects.
 */

use crate::engine::{
    error::{RobinError, RobinResult},
    collaboration::ContributionType,
};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, BTreeMap};
use std::time::{SystemTime, Duration};

/// Core project management system
pub struct ProjectManager {
    /// All active projects
    projects: HashMap<String, Project>,
    /// User contributions tracking
    contributions: HashMap<String, Vec<Contribution>>,
    /// Active invitations
    invitations: HashMap<String, ProjectInvitation>,
    /// Project templates for common workflows
    templates: HashMap<String, ProjectTemplate>,
}

impl ProjectManager {
    pub fn new() -> Self {
        let mut manager = Self {
            projects: HashMap::new(),
            contributions: HashMap::new(),
            invitations: HashMap::new(),
            templates: HashMap::new(),
        };

        manager.initialize_templates();
        manager
    }

    /// Create a new collaborative project
    pub fn create_project(&mut self, project_id: String, owner_id: String) -> RobinResult<()> {
        if self.projects.contains_key(&project_id) {
            return Err(RobinError::AlreadyExists("Project already exists".to_string()));
        }

        let project = Project {
            id: project_id.clone(),
            name: format!("Project {}", project_id),
            description: "Collaborative building project".to_string(),
            owner_id: owner_id.clone(),
            created_at: SystemTime::now(),
            status: ProjectStatus::Active,
            members: vec![ProjectMember {
                user_id: owner_id.clone(),
                role: ProjectRole::ProjectManager,
                joined_at: SystemTime::now(),
                permissions: vec![
                    Permission::ManageProject,
                    Permission::ManageMembers,
                    Permission::CreateTasks,
                    Permission::AssignTasks,
                    Permission::ModifyStructure,
                    Permission::CreateSavePoints,
                ],
                contribution_score: 0,
            }],
            tasks: Vec::new(),
            milestones: Vec::new(),
            budget: ProjectBudget::default(),
            settings: ProjectSettings::default(),
        };

        self.projects.insert(project_id, project);
        Ok(())
    }

    /// Generate invitation code for project
    pub fn create_invitation(&mut self, project_id: &str, role: ProjectRole, inviter_id: &str) -> RobinResult<String> {
        // Verify project exists and user has permission
        let project = self.projects.get(project_id)
            .ok_or_else(|| RobinError::NotFound("Project not found".to_string()))?;

        if !self.has_permission(project_id, inviter_id, &Permission::ManageMembers) {
            return Err(RobinError::PermissionDenied("Cannot create invitations".to_string()));
        }

        let invite_code = format!("{}-{}-{}", project_id, role.to_string(), uuid::Uuid::new_v4());

        let invitation = ProjectInvitation {
            code: invite_code.clone(),
            project_id: project_id.to_string(),
            role,
            inviter_id: inviter_id.to_string(),
            created_at: SystemTime::now(),
            expires_at: SystemTime::now() + Duration::from_secs(24 * 60 * 60), // 24 hours
            max_uses: 1,
            current_uses: 0,
        };

        self.invitations.insert(invite_code.clone(), invitation);
        Ok(invite_code)
    }

    /// Validate invitation and get role
    pub fn validate_invite(&mut self, invite_code: &str) -> RobinResult<ProjectRole> {
        let invitation = self.invitations.get_mut(invite_code)
            .ok_or_else(|| RobinError::NotFound("Invalid invitation code".to_string()))?;

        // Check expiration
        if SystemTime::now() > invitation.expires_at {
            self.invitations.remove(invite_code);
            return Err(RobinError::Expired("Invitation expired".to_string()));
        }

        // Check usage limit
        if invitation.current_uses >= invitation.max_uses {
            return Err(RobinError::LimitExceeded("Invitation already used".to_string()));
        }

        invitation.current_uses += 1;
        Ok(invitation.role)
    }

    /// Add member to project
    pub fn add_member(&mut self, project_id: &str, user_id: String, role: ProjectRole) -> RobinResult<()> {
        let project = self.projects.get_mut(project_id)
            .ok_or_else(|| RobinError::NotFound("Project not found".to_string()))?;

        // Check if user is already a member
        if project.members.iter().any(|m| m.user_id == user_id) {
            return Err(RobinError::AlreadyExists("User already a member".to_string()));
        }

        let permissions = self.get_role_permissions(role);

        let member = ProjectMember {
            user_id,
            role,
            joined_at: SystemTime::now(),
            permissions,
            contribution_score: 0,
        };

        project.members.push(member);
        Ok(())
    }

    /// Create a new task
    pub fn create_task(&mut self, project_id: &str, creator_id: &str, task_data: TaskData) -> RobinResult<String> {
        if !self.has_permission(project_id, creator_id, &Permission::CreateTasks) {
            return Err(RobinError::PermissionDenied("Cannot create tasks".to_string()));
        }

        let project = self.projects.get_mut(project_id)
            .ok_or_else(|| RobinError::NotFound("Project not found".to_string()))?;

        let task_id = uuid::Uuid::new_v4().to_string();
        let task = Task {
            id: task_id.clone(),
            title: task_data.title,
            description: task_data.description,
            task_type: task_data.task_type,
            status: TaskStatus::Open,
            priority: task_data.priority,
            assigned_to: None,
            created_by: creator_id.to_string(),
            created_at: SystemTime::now(),
            due_date: task_data.due_date,
            estimated_hours: task_data.estimated_hours,
            actual_hours: 0.0,
            dependencies: task_data.dependencies,
            tags: task_data.tags,
            location: task_data.location,
        };

        project.tasks.push(task);
        Ok(task_id)
    }

    /// Assign task to user
    pub fn assign_task(&mut self, project_id: &str, task_id: &str, assignee_id: &str, assigner_id: &str) -> RobinResult<()> {
        if !self.has_permission(project_id, assigner_id, &Permission::AssignTasks) {
            return Err(RobinError::PermissionDenied("Cannot assign tasks".to_string()));
        }

        let project = self.projects.get_mut(project_id)
            .ok_or_else(|| RobinError::NotFound("Project not found".to_string()))?;

        // Verify assignee is project member
        if !project.members.iter().any(|m| m.user_id == assignee_id) {
            return Err(RobinError::NotFound("Assignee is not a project member".to_string()));
        }

        let task = project.tasks.iter_mut()
            .find(|t| t.id == task_id)
            .ok_or_else(|| RobinError::NotFound("Task not found".to_string()))?;

        task.assigned_to = Some(assignee_id.to_string());
        if task.status == TaskStatus::Open {
            task.status = TaskStatus::Assigned;
        }

        Ok(())
    }

    /// Update task status
    pub fn update_task_status(&mut self, project_id: &str, task_id: &str, new_status: TaskStatus, user_id: &str) -> RobinResult<()> {
        let project = self.projects.get_mut(project_id)
            .ok_or_else(|| RobinError::NotFound("Project not found".to_string()))?;

        let task = project.tasks.iter_mut()
            .find(|t| t.id == task_id)
            .ok_or_else(|| RobinError::NotFound("Task not found".to_string()))?;

        // Check if user can update this task
        let can_update = task.assigned_to.as_ref() == Some(&user_id.to_string()) ||
                         task.created_by == user_id ||
                         self.has_permission(project_id, user_id, &Permission::ManageProject);

        if !can_update {
            return Err(RobinError::PermissionDenied("Cannot update this task".to_string()));
        }

        task.status = new_status;

        // Update contribution score if task completed
        if new_status == TaskStatus::Completed {
            if let Some(assignee) = &task.assigned_to {
                self.award_contribution_points(assignee, ContributionType::TaskCompleted, 100)?;
            }
        }

        Ok(())
    }

    /// Record user contribution
    pub fn record_contribution(&mut self, user_id: &str, contribution_type: ContributionType) -> RobinResult<()> {
        let contribution = Contribution {
            user_id: user_id.to_string(),
            contribution_type,
            timestamp: SystemTime::now(),
            points: self.calculate_contribution_points(&contribution_type),
        };

        self.contributions.entry(user_id.to_string())
            .or_insert_with(Vec::new)
            .push(contribution);

        Ok(())
    }

    /// Update project system
    pub fn update(&mut self, delta_time: f32) -> RobinResult<()> {
        // Update task deadlines and send reminders
        for project in self.projects.values_mut() {
            for task in &mut project.tasks {
                if let Some(due_date) = task.due_date {
                    let time_until_due = due_date.duration_since(SystemTime::now()).unwrap_or_default();

                    // Mark overdue tasks
                    if time_until_due.as_secs() == 0 && task.status != TaskStatus::Completed {
                        task.status = TaskStatus::Overdue;
                    }
                }
            }
        }

        // Clean up expired invitations
        self.invitations.retain(|_, invite| SystemTime::now() < invite.expires_at);

        Ok(())
    }

    /// Get active tasks for a user
    pub fn get_user_tasks(&self, project_id: &str, user_id: &str) -> Vec<&Task> {
        if let Some(project) = self.projects.get(project_id) {
            project.tasks.iter()
                .filter(|task| {
                    task.assigned_to.as_ref() == Some(&user_id.to_string()) &&
                    task.status != TaskStatus::Completed &&
                    task.status != TaskStatus::Cancelled
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get all active tasks in project
    pub fn get_active_tasks(&self) -> Vec<&Task> {
        self.projects.values()
            .flat_map(|project| &project.tasks)
            .filter(|task| task.status == TaskStatus::Open || task.status == TaskStatus::Assigned || task.status == TaskStatus::InProgress)
            .collect()
    }

    /// Get project statistics
    pub fn get_project_stats(&self, project_id: &str) -> Option<ProjectStats> {
        let project = self.projects.get(project_id)?;

        let total_tasks = project.tasks.len();
        let completed_tasks = project.tasks.iter().filter(|t| t.status == TaskStatus::Completed).count();
        let in_progress_tasks = project.tasks.iter().filter(|t| t.status == TaskStatus::InProgress).count();
        let overdue_tasks = project.tasks.iter().filter(|t| t.status == TaskStatus::Overdue).count();

        let completion_percentage = if total_tasks > 0 {
            (completed_tasks as f32 / total_tasks as f32) * 100.0
        } else {
            0.0
        };

        Some(ProjectStats {
            total_tasks,
            completed_tasks,
            in_progress_tasks,
            overdue_tasks,
            completion_percentage,
            total_members: project.members.len(),
            active_contributors: self.get_active_contributors(project_id),
            total_contribution_points: self.get_total_contribution_points(project_id),
        })
    }

    /// Apply project update from network
    pub fn apply_update(&mut self, update: crate::engine::collaboration::ProjectUpdate) -> RobinResult<()> {
        // Handle different types of project updates
        match update.update_type.as_str() {
            "task_created" => self.handle_task_created_update(update)?,
            "task_status_changed" => self.handle_task_status_update(update)?,
            "member_added" => self.handle_member_added_update(update)?,
            _ => return Err(RobinError::InvalidInput(format!("Unknown update type: {}", update.update_type))),
        }
        Ok(())
    }

    /// Check if user has specific permission
    fn has_permission(&self, project_id: &str, user_id: &str, permission: &Permission) -> bool {
        if let Some(project) = self.projects.get(project_id) {
            if let Some(member) = project.members.iter().find(|m| m.user_id == user_id) {
                return member.permissions.contains(permission);
            }
        }
        false
    }

    /// Get permissions for a role
    fn get_role_permissions(&self, role: ProjectRole) -> Vec<Permission> {
        match role {
            ProjectRole::ProjectManager => vec![
                Permission::ManageProject,
                Permission::ManageMembers,
                Permission::CreateTasks,
                Permission::AssignTasks,
                Permission::ModifyStructure,
                Permission::CreateSavePoints,
                Permission::ViewReports,
            ],
            ProjectRole::TeamLead => vec![
                Permission::CreateTasks,
                Permission::AssignTasks,
                Permission::ModifyStructure,
                Permission::ViewReports,
            ],
            ProjectRole::SeniorBuilder => vec![
                Permission::ModifyStructure,
                Permission::CreateSavePoints,
                Permission::ViewReports,
            ],
            ProjectRole::Builder => vec![
                Permission::ModifyStructure,
            ],
            ProjectRole::Contributor => vec![
                // Basic building permissions
            ],
            ProjectRole::Observer => vec![
                // Read-only access
            ],
        }
    }

    /// Calculate points for contribution type
    fn calculate_contribution_points(&self, contribution_type: &ContributionType) -> u32 {
        match contribution_type {
            ContributionType::VoxelPlaced => 1,
            ContributionType::VoxelRemoved => 1,
            ContributionType::StructureCompleted => 50,
            ContributionType::TaskCompleted => 100,
            ContributionType::MessageSent => 2,
            ContributionType::AnnotationCreated => 5,
        }
    }

    /// Award contribution points to user
    fn award_contribution_points(&mut self, user_id: &str, contribution_type: ContributionType, bonus: u32) -> RobinResult<()> {
        let base_points = self.calculate_contribution_points(&contribution_type);
        let total_points = base_points + bonus;

        // Update user's total contribution score
        for project in self.projects.values_mut() {
            if let Some(member) = project.members.iter_mut().find(|m| m.user_id == user_id) {
                member.contribution_score += total_points;
                break;
            }
        }

        Ok(())
    }

    /// Get active contributors count
    fn get_active_contributors(&self, project_id: &str) -> usize {
        // Contributors who have made contributions in the last 24 hours
        let cutoff = SystemTime::now() - Duration::from_secs(24 * 60 * 60);

        self.contributions.values()
            .flat_map(|contribs| contribs.iter())
            .filter(|contrib| contrib.timestamp > cutoff)
            .map(|contrib| &contrib.user_id)
            .collect::<std::collections::HashSet<_>>()
            .len()
    }

    /// Get total contribution points for project
    fn get_total_contribution_points(&self, project_id: &str) -> u32 {
        if let Some(project) = self.projects.get(project_id) {
            project.members.iter()
                .map(|member| member.contribution_score)
                .sum()
        } else {
            0
        }
    }

    /// Initialize project templates
    fn initialize_templates(&mut self) {
        // Architecture Project Template
        self.templates.insert("architecture".to_string(), ProjectTemplate {
            name: "Architecture Project".to_string(),
            description: "Template for architectural design and construction projects".to_string(),
            default_roles: vec![
                ProjectRole::ProjectManager,
                ProjectRole::TeamLead,
                ProjectRole::SeniorBuilder,
                ProjectRole::Builder,
            ],
            task_templates: vec![
                TaskTemplate {
                    title: "Site Planning".to_string(),
                    description: "Plan the overall site layout and structure placement".to_string(),
                    task_type: TaskType::Planning,
                    priority: Priority::High,
                    estimated_hours: 8.0,
                },
                TaskTemplate {
                    title: "Foundation Construction".to_string(),
                    description: "Build the structural foundation".to_string(),
                    task_type: TaskType::Construction,
                    priority: Priority::High,
                    estimated_hours: 16.0,
                },
                TaskTemplate {
                    title: "Wall Construction".to_string(),
                    description: "Construct main walls and structural elements".to_string(),
                    task_type: TaskType::Construction,
                    priority: Priority::Medium,
                    estimated_hours: 24.0,
                },
            ],
        });

        // Engineering Project Template
        self.templates.insert("engineering".to_string(), ProjectTemplate {
            name: "Engineering Project".to_string(),
            description: "Template for complex engineering and automation projects".to_string(),
            default_roles: vec![
                ProjectRole::ProjectManager,
                ProjectRole::TeamLead,
                ProjectRole::SeniorBuilder,
            ],
            task_templates: vec![
                TaskTemplate {
                    title: "System Design".to_string(),
                    description: "Design the engineering system architecture".to_string(),
                    task_type: TaskType::Design,
                    priority: Priority::High,
                    estimated_hours: 12.0,
                },
                TaskTemplate {
                    title: "Logic Implementation".to_string(),
                    description: "Implement logic components and automation".to_string(),
                    task_type: TaskType::Implementation,
                    priority: Priority::Medium,
                    estimated_hours: 20.0,
                },
            ],
        });
    }

    /// Handle task created update
    fn handle_task_created_update(&mut self, update: crate::engine::collaboration::ProjectUpdate) -> RobinResult<()> {
        // In a real implementation, deserialize task data from update.data
        // and add to appropriate project
        Ok(())
    }

    /// Handle task status update
    fn handle_task_status_update(&mut self, update: crate::engine::collaboration::ProjectUpdate) -> RobinResult<()> {
        // In a real implementation, update task status based on update.data
        Ok(())
    }

    /// Handle member added update
    fn handle_member_added_update(&mut self, update: crate::engine::collaboration::ProjectUpdate) -> RobinResult<()> {
        // In a real implementation, add new member based on update.data
        Ok(())
    }
}

/// Project roles with different permission levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectRole {
    ProjectManager, // Full control
    TeamLead,      // Manage tasks and assign work
    SeniorBuilder, // Advanced building privileges
    Builder,       // Standard building access
    Contributor,   // Basic participation
    Observer,      // Read-only access
}

impl ProjectRole {
    pub fn to_string(&self) -> &'static str {
        match self {
            ProjectRole::ProjectManager => "ProjectManager",
            ProjectRole::TeamLead => "TeamLead",
            ProjectRole::SeniorBuilder => "SeniorBuilder",
            ProjectRole::Builder => "Builder",
            ProjectRole::Contributor => "Contributor",
            ProjectRole::Observer => "Observer",
        }
    }
}

/// Specific permissions that can be granted
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Permission {
    ManageProject,
    ManageMembers,
    CreateTasks,
    AssignTasks,
    ModifyStructure,
    CreateSavePoints,
    ViewReports,
}

/// Complete project information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: String,
    pub owner_id: String,
    pub created_at: SystemTime,
    pub status: ProjectStatus,
    pub members: Vec<ProjectMember>,
    pub tasks: Vec<Task>,
    pub milestones: Vec<Milestone>,
    pub budget: ProjectBudget,
    pub settings: ProjectSettings,
}

/// Project status levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectStatus {
    Planning,
    Active,
    OnHold,
    Completed,
    Cancelled,
}

/// Project member information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMember {
    pub user_id: String,
    pub role: ProjectRole,
    pub joined_at: SystemTime,
    pub permissions: Vec<Permission>,
    pub contribution_score: u32,
}

/// Task information and tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub task_type: TaskType,
    pub status: TaskStatus,
    pub priority: Priority,
    pub assigned_to: Option<String>,
    pub created_by: String,
    pub created_at: SystemTime,
    pub due_date: Option<SystemTime>,
    pub estimated_hours: f32,
    pub actual_hours: f32,
    pub dependencies: Vec<String>,
    pub tags: Vec<String>,
    pub location: Option<TaskLocation>,
}

/// Types of tasks in projects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskType {
    Planning,
    Design,
    Construction,
    Implementation,
    Testing,
    Review,
    Documentation,
}

/// Task status tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Open,
    Assigned,
    InProgress,
    Review,
    Completed,
    Cancelled,
    Blocked,
    Overdue,
}

/// Task priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

/// Location information for tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskLocation {
    pub center: crate::engine::math::Vec3,
    pub radius: f32,
    pub description: String,
}

/// Project milestone tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub id: String,
    pub name: String,
    pub description: String,
    pub target_date: SystemTime,
    pub completion_criteria: Vec<String>,
    pub status: MilestoneStatus,
}

/// Milestone status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MilestoneStatus {
    Planned,
    InProgress,
    Completed,
    Delayed,
}

/// Project budget tracking
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectBudget {
    pub resource_allocations: HashMap<String, u32>,
    pub time_budget_hours: f32,
    pub spent_hours: f32,
}

/// Project settings and preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    pub auto_save_interval: Duration,
    pub require_approval_for_major_changes: bool,
    pub allow_external_contributors: bool,
    pub default_permissions: Vec<Permission>,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            auto_save_interval: Duration::from_secs(300), // 5 minutes
            require_approval_for_major_changes: true,
            allow_external_contributors: false,
            default_permissions: vec![],
        }
    }
}

/// Project invitation data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInvitation {
    pub code: String,
    pub project_id: String,
    pub role: ProjectRole,
    pub inviter_id: String,
    pub created_at: SystemTime,
    pub expires_at: SystemTime,
    pub max_uses: u32,
    pub current_uses: u32,
}

/// User contribution tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contribution {
    pub user_id: String,
    pub contribution_type: ContributionType,
    pub timestamp: SystemTime,
    pub points: u32,
}

/// Task creation data
#[derive(Debug, Clone)]
pub struct TaskData {
    pub title: String,
    pub description: String,
    pub task_type: TaskType,
    pub priority: Priority,
    pub due_date: Option<SystemTime>,
    pub estimated_hours: f32,
    pub dependencies: Vec<String>,
    pub tags: Vec<String>,
    pub location: Option<TaskLocation>,
}

/// Project template for common workflows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectTemplate {
    pub name: String,
    pub description: String,
    pub default_roles: Vec<ProjectRole>,
    pub task_templates: Vec<TaskTemplate>,
}

/// Template for creating tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskTemplate {
    pub title: String,
    pub description: String,
    pub task_type: TaskType,
    pub priority: Priority,
    pub estimated_hours: f32,
}

/// Project statistics
#[derive(Debug, Clone)]
pub struct ProjectStats {
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub in_progress_tasks: usize,
    pub overdue_tasks: usize,
    pub completion_percentage: f32,
    pub total_members: usize,
    pub active_contributors: usize,
    pub total_contribution_points: u32,
}

impl Default for ProjectManager {
    fn default() -> Self {
        Self::new()
    }
}