/*!
 * Permission and Access Control System for Collaborative Building
 *
 * Manages user permissions, build zones, access levels, and security
 * for collaborative engineering projects in Robin Engine.
 */

use crate::engine::{
    error::{RobinError, RobinResult},
    math::Vec3,
    world::VoxelType,
    collaboration::{ProjectRole, Permission as ProjectPermission},
};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, Duration};

/// Core permission management system
pub struct PermissionManager {
    /// User permissions by project
    user_permissions: HashMap<String, HashMap<String, UserPermissions>>, // project_id -> user_id -> permissions
    /// Build zones and access control
    build_zones: HashMap<String, Vec<BuildZone>>, // project_id -> zones
    /// Access levels for different areas
    access_levels: HashMap<String, HashMap<String, AccessLevel>>, // project_id -> zone_id -> access_level
    /// Temporary permission grants
    temporary_grants: Vec<TemporaryPermission>,
    /// Permission rules and policies
    policies: HashMap<String, PermissionPolicy>,
}

impl PermissionManager {
    pub fn new() -> Self {
        let mut manager = Self {
            user_permissions: HashMap::new(),
            build_zones: HashMap::new(),
            access_levels: HashMap::new(),
            temporary_grants: Vec::new(),
            policies: HashMap::new(),
        };

        manager.initialize_default_policies();
        manager
    }

    /// Set up permissions for a project
    pub fn initialize_project(&mut self, project_id: String, owner_id: String) -> RobinResult<()> {
        // Create default build zones
        let zones = vec![
            BuildZone {
                id: "main_construction".to_string(),
                name: "Main Construction Area".to_string(),
                description: "Primary building and construction zone".to_string(),
                bounds: ZoneBounds::Rectangular {
                    min: Vec3::new(-100.0, -50.0, -100.0),
                    max: Vec3::new(100.0, 50.0, 100.0),
                },
                zone_type: ZoneType::Construction,
                required_role: ProjectRole::Builder,
                special_permissions: vec![],
                created_by: owner_id.clone(),
                created_at: SystemTime::now(),
            },
            BuildZone {
                id: "planning_area".to_string(),
                name: "Planning and Design Area".to_string(),
                description: "Area reserved for planning and architectural work".to_string(),
                bounds: ZoneBounds::Rectangular {
                    min: Vec3::new(-150.0, 0.0, -150.0),
                    max: Vec3::new(150.0, 100.0, 150.0),
                },
                zone_type: ZoneType::Planning,
                required_role: ProjectRole::TeamLead,
                special_permissions: vec![Permission::CreateBlueprints, Permission::ModifyTerrain],
                created_by: owner_id.clone(),
                created_at: SystemTime::now(),
            },
            BuildZone {
                id: "protected_zone".to_string(),
                name: "Protected Infrastructure".to_string(),
                description: "Critical infrastructure requiring special permissions".to_string(),
                bounds: ZoneBounds::Spherical {
                    center: Vec3::new(0.0, -25.0, 0.0),
                    radius: 25.0,
                },
                zone_type: ZoneType::Protected,
                required_role: ProjectRole::SeniorBuilder,
                special_permissions: vec![Permission::ModifyInfrastructure],
                created_by: owner_id.clone(),
                created_at: SystemTime::now(),
            },
        ];

        self.build_zones.insert(project_id.clone(), zones);

        // Set up default access levels
        let mut access_levels = HashMap::new();
        access_levels.insert("main_construction".to_string(), AccessLevel::Standard);
        access_levels.insert("planning_area".to_string(), AccessLevel::Restricted);
        access_levels.insert("protected_zone".to_string(), AccessLevel::Secure);

        self.access_levels.insert(project_id.clone(), access_levels);

        // Set up owner permissions
        let owner_permissions = UserPermissions {
            user_id: owner_id,
            role: ProjectRole::ProjectManager,
            permissions: vec![
                Permission::PlaceVoxels,
                Permission::RemoveVoxels,
                Permission::ModifyTerrain,
                Permission::CreateBlueprints,
                Permission::ModifyInfrastructure,
                Permission::ManagePermissions,
                Permission::CreateZones,
                Permission::ModifyZones,
                Permission::GrantTemporaryAccess,
            ],
            zones_access: vec!["main_construction".to_string(), "planning_area".to_string(), "protected_zone".to_string()],
            restrictions: vec![],
            granted_at: SystemTime::now(),
            expires_at: None,
        };

        self.user_permissions.entry(project_id)
            .or_insert_with(HashMap::new)
            .insert(owner_permissions.user_id.clone(), owner_permissions);

        Ok(())
    }

    /// Add user permissions to project
    pub fn add_user_permissions(&mut self, project_id: String, user_id: String, role: ProjectRole) -> RobinResult<()> {
        let permissions = self.get_role_permissions(role);
        let zones_access = self.get_role_zone_access(role);

        let user_permissions = UserPermissions {
            user_id: user_id.clone(),
            role,
            permissions,
            zones_access,
            restrictions: vec![],
            granted_at: SystemTime::now(),
            expires_at: None,
        };

        self.user_permissions.entry(project_id)
            .or_insert_with(HashMap::new)
            .insert(user_id, user_permissions);

        Ok(())
    }

    /// Check if user can place voxel at position
    pub fn can_place_voxel(&self, user_id: &str, position: Vec3, voxel_type: VoxelType) -> bool {
        if let Some(project_permissions) = self.find_user_in_any_project(user_id) {
            let (project_id, user_perms) = project_permissions;

            // Check basic permission
            if !user_perms.permissions.contains(&Permission::PlaceVoxels) {
                return false;
            }

            // Check zone access
            if let Some(zone) = self.get_zone_for_position(&project_id, position) {
                if !self.can_access_zone(user_id, &project_id, &zone.id) {
                    return false;
                }

                // Check if voxel type is allowed in this zone
                if !self.is_voxel_type_allowed(&zone, voxel_type) {
                    return false;
                }
            }

            // Check restrictions
            for restriction in &user_perms.restrictions {
                if !self.check_restriction_allows_action(restriction, &RestrictionAction::PlaceVoxel { position, voxel_type }) {
                    return false;
                }
            }

            true
        } else {
            false
        }
    }

    /// Check if user can remove voxel at position
    pub fn can_remove_voxel(&self, user_id: &str, position: Vec3, voxel_type: VoxelType) -> bool {
        if let Some(project_permissions) = self.find_user_in_any_project(user_id) {
            let (project_id, user_perms) = project_permissions;

            // Check basic permission
            if !user_perms.permissions.contains(&Permission::RemoveVoxels) {
                return false;
            }

            // Check zone access
            if let Some(zone) = self.get_zone_for_position(&project_id, position) {
                if !self.can_access_zone(user_id, &project_id, &zone.id) {
                    return false;
                }

                // Protected zones may have removal restrictions
                if zone.zone_type == ZoneType::Protected &&
                   !user_perms.permissions.contains(&Permission::ModifyInfrastructure) {
                    return false;
                }
            }

            // Check restrictions
            for restriction in &user_perms.restrictions {
                if !self.check_restriction_allows_action(restriction, &RestrictionAction::RemoveVoxel { position, voxel_type }) {
                    return false;
                }
            }

            true
        } else {
            false
        }
    }

    /// Check if user can create save points
    pub fn can_create_save_point(&self, user_id: &str) -> bool {
        if let Some((_, user_perms)) = self.find_user_in_any_project(user_id) {
            user_perms.role == ProjectRole::ProjectManager ||
            user_perms.role == ProjectRole::TeamLead ||
            user_perms.permissions.contains(&Permission::CreateSavePoints)
        } else {
            false
        }
    }

    /// Grant temporary permission to user
    pub fn grant_temporary_permission(&mut self, grantor_id: &str, user_id: &str, permission: Permission, duration: Duration, reason: String) -> RobinResult<()> {
        // Check if grantor has permission to grant temporary access
        if let Some((project_id, grantor_perms)) = self.find_user_in_any_project(grantor_id) {
            if !grantor_perms.permissions.contains(&Permission::GrantTemporaryAccess) {
                return Err(RobinError::PermissionDenied("Cannot grant temporary permissions".to_string()));
            }

            let temp_permission = TemporaryPermission {
                id: uuid::Uuid::new_v4().to_string(),
                project_id: project_id.clone(),
                user_id: user_id.to_string(),
                permission,
                granted_by: grantor_id.to_string(),
                granted_at: SystemTime::now(),
                expires_at: SystemTime::now() + duration,
                reason,
                used_count: 0,
                max_uses: None,
            };

            self.temporary_grants.push(temp_permission);
            Ok(())
        } else {
            Err(RobinError::PermissionDenied("Grantor not found in any project".to_string()))
        }
    }

    /// Create new build zone
    pub fn create_zone(&mut self, project_id: &str, creator_id: &str, zone_data: ZoneCreationData) -> RobinResult<String> {
        // Check permission to create zones
        if let Some(user_perms) = self.get_user_permissions(project_id, creator_id) {
            if !user_perms.permissions.contains(&Permission::CreateZones) {
                return Err(RobinError::PermissionDenied("Cannot create zones".to_string()));
            }
        } else {
            return Err(RobinError::NotFound("User not found in project".to_string()));
        }

        let zone_id = uuid::Uuid::new_v4().to_string();
        let zone = BuildZone {
            id: zone_id.clone(),
            name: zone_data.name,
            description: zone_data.description,
            bounds: zone_data.bounds,
            zone_type: zone_data.zone_type,
            required_role: zone_data.required_role,
            special_permissions: zone_data.special_permissions,
            created_by: creator_id.to_string(),
            created_at: SystemTime::now(),
        };

        self.build_zones.entry(project_id.to_string())
            .or_insert_with(Vec::new)
            .push(zone);

        self.access_levels.entry(project_id.to_string())
            .or_insert_with(HashMap::new)
            .insert(zone_id.clone(), zone_data.access_level);

        Ok(zone_id)
    }

    /// Update permission system
    pub fn update(&mut self, delta_time: f32) -> RobinResult<()> {
        let now = SystemTime::now();

        // Clean up expired temporary permissions
        self.temporary_grants.retain(|temp| now < temp.expires_at);

        // Clean up expired user permissions
        for project_perms in self.user_permissions.values_mut() {
            project_perms.retain(|_, user_perms| {
                if let Some(expires_at) = user_perms.expires_at {
                    now < expires_at
                } else {
                    true
                }
            });
        }

        Ok(())
    }

    /// Apply permission change from network
    pub fn apply_change(&mut self, change: crate::engine::collaboration::PermissionChange) -> RobinResult<()> {
        // Handle different types of permission changes
        if change.granted {
            // Grant permission
            if let Some(project_perms) = self.user_permissions.get_mut(&change.user_id) {
                if let Some(user_perms) = project_perms.values_mut().next() {
                    // In a real implementation, would convert ProjectPermission to Permission
                    // For now, just track the change
                }
            }
        } else {
            // Revoke permission
            // Similar logic for revoking permissions
        }
        Ok(())
    }

    /// Get user permissions for project
    fn get_user_permissions(&self, project_id: &str, user_id: &str) -> Option<&UserPermissions> {
        self.user_permissions.get(project_id)?.get(user_id)
    }

    /// Find user in any project
    fn find_user_in_any_project(&self, user_id: &str) -> Option<(String, &UserPermissions)> {
        for (project_id, project_perms) in &self.user_permissions {
            if let Some(user_perms) = project_perms.get(user_id) {
                return Some((project_id.clone(), user_perms));
            }
        }
        None
    }

    /// Get zone containing position
    fn get_zone_for_position(&self, project_id: &str, position: Vec3) -> Option<&BuildZone> {
        if let Some(zones) = self.build_zones.get(project_id) {
            zones.iter().find(|zone| zone.contains_position(position))
        } else {
            None
        }
    }

    /// Check if user can access zone
    fn can_access_zone(&self, user_id: &str, project_id: &str, zone_id: &str) -> bool {
        if let Some(user_perms) = self.get_user_permissions(project_id, user_id) {
            user_perms.zones_access.contains(&zone_id.to_string())
        } else {
            false
        }
    }

    /// Check if voxel type is allowed in zone
    fn is_voxel_type_allowed(&self, zone: &BuildZone, voxel_type: VoxelType) -> bool {
        match zone.zone_type {
            ZoneType::Construction => true, // All voxel types allowed
            ZoneType::Planning => {
                // Only certain materials for planning
                matches!(voxel_type, VoxelType::Stone | VoxelType::Earth | VoxelType::Sand)
            }
            ZoneType::Protected => {
                // Very restricted material palette
                matches!(voxel_type, VoxelType::Stone)
            }
            ZoneType::Restricted => false, // No building allowed
        }
    }

    /// Check if restriction allows action
    fn check_restriction_allows_action(&self, restriction: &Restriction, action: &RestrictionAction) -> bool {
        match restriction {
            Restriction::TimeBasedLimit { start_time, end_time } => {
                let now = SystemTime::now();
                now >= *start_time && now <= *end_time
            }
            Restriction::VoxelTypeBlacklist(blocked_types) => {
                match action {
                    RestrictionAction::PlaceVoxel { voxel_type, .. } |
                    RestrictionAction::RemoveVoxel { voxel_type, .. } => {
                        !blocked_types.contains(voxel_type)
                    }
                }
            }
            Restriction::AreaRestriction { allowed_bounds } => {
                match action {
                    RestrictionAction::PlaceVoxel { position, .. } |
                    RestrictionAction::RemoveVoxel { position, .. } => {
                        allowed_bounds.contains_position(*position)
                    }
                }
            }
            Restriction::ActionLimit { max_actions, .. } => {
                // Would need to track action count - simplified for now
                true
            }
        }
    }

    /// Get permissions for a role
    fn get_role_permissions(&self, role: ProjectRole) -> Vec<Permission> {
        match role {
            ProjectRole::ProjectManager => vec![
                Permission::PlaceVoxels,
                Permission::RemoveVoxels,
                Permission::ModifyTerrain,
                Permission::CreateBlueprints,
                Permission::ModifyInfrastructure,
                Permission::ManagePermissions,
                Permission::CreateZones,
                Permission::ModifyZones,
                Permission::GrantTemporaryAccess,
                Permission::CreateSavePoints,
            ],
            ProjectRole::TeamLead => vec![
                Permission::PlaceVoxels,
                Permission::RemoveVoxels,
                Permission::ModifyTerrain,
                Permission::CreateBlueprints,
                Permission::CreateSavePoints,
            ],
            ProjectRole::SeniorBuilder => vec![
                Permission::PlaceVoxels,
                Permission::RemoveVoxels,
                Permission::ModifyTerrain,
                Permission::CreateBlueprints,
            ],
            ProjectRole::Builder => vec![
                Permission::PlaceVoxels,
                Permission::RemoveVoxels,
            ],
            ProjectRole::Contributor => vec![
                Permission::PlaceVoxels,
            ],
            ProjectRole::Observer => vec![
                // No building permissions
            ],
        }
    }

    /// Get zone access for role
    fn get_role_zone_access(&self, role: ProjectRole) -> Vec<String> {
        match role {
            ProjectRole::ProjectManager => vec![
                "main_construction".to_string(),
                "planning_area".to_string(),
                "protected_zone".to_string(),
            ],
            ProjectRole::TeamLead => vec![
                "main_construction".to_string(),
                "planning_area".to_string(),
            ],
            ProjectRole::SeniorBuilder => vec![
                "main_construction".to_string(),
                "protected_zone".to_string(),
            ],
            ProjectRole::Builder => vec![
                "main_construction".to_string(),
            ],
            ProjectRole::Contributor => vec![
                "main_construction".to_string(),
            ],
            ProjectRole::Observer => vec![
                // No zone access
            ],
        }
    }

    /// Initialize default policies
    fn initialize_default_policies(&mut self) {
        self.policies.insert("default_construction".to_string(), PermissionPolicy {
            name: "Default Construction Policy".to_string(),
            description: "Standard permissions for construction projects".to_string(),
            rules: vec![
                PermissionRule {
                    condition: RuleCondition::RoleEquals(ProjectRole::Builder),
                    action: RuleAction::Allow(Permission::PlaceVoxels),
                },
                PermissionRule {
                    condition: RuleCondition::ZoneType(ZoneType::Protected),
                    action: RuleAction::Require(Permission::ModifyInfrastructure),
                },
            ],
        });
    }
}

/// User permissions within a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermissions {
    pub user_id: String,
    pub role: ProjectRole,
    pub permissions: Vec<Permission>,
    pub zones_access: Vec<String>,
    pub restrictions: Vec<Restriction>,
    pub granted_at: SystemTime,
    pub expires_at: Option<SystemTime>,
}

/// Specific building and project permissions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Permission {
    PlaceVoxels,
    RemoveVoxels,
    ModifyTerrain,
    CreateBlueprints,
    ModifyInfrastructure,
    ManagePermissions,
    CreateZones,
    ModifyZones,
    GrantTemporaryAccess,
    CreateSavePoints,
}

/// Access levels for different areas
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessLevel {
    Public,    // Anyone can access
    Standard,  // Project members can access
    Restricted, // Requires specific role/permission
    Secure,    // Requires elevated permissions
    Locked,    // No access allowed
}

/// Build zones with different permission requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildZone {
    pub id: String,
    pub name: String,
    pub description: String,
    pub bounds: ZoneBounds,
    pub zone_type: ZoneType,
    pub required_role: ProjectRole,
    pub special_permissions: Vec<Permission>,
    pub created_by: String,
    pub created_at: SystemTime,
}

impl BuildZone {
    pub fn contains_position(&self, position: Vec3) -> bool {
        self.bounds.contains_position(position)
    }
}

/// Different zone boundary types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ZoneBounds {
    Rectangular { min: Vec3, max: Vec3 },
    Spherical { center: Vec3, radius: f32 },
    Cylindrical { center: Vec3, radius: f32, height: f32 },
}

impl ZoneBounds {
    pub fn contains_position(&self, position: Vec3) -> bool {
        match self {
            ZoneBounds::Rectangular { min, max } => {
                position.x >= min.x && position.x <= max.x &&
                position.y >= min.y && position.y <= max.y &&
                position.z >= min.z && position.z <= max.z
            }
            ZoneBounds::Spherical { center, radius } => {
                let distance = (position - *center).magnitude();
                distance <= *radius
            }
            ZoneBounds::Cylindrical { center, radius, height } => {
                let horizontal_dist = ((position.x - center.x).powi(2) + (position.z - center.z).powi(2)).sqrt();
                horizontal_dist <= *radius &&
                position.y >= center.y &&
                position.y <= center.y + height
            }
        }
    }
}

/// Types of zones with different rules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZoneType {
    Construction, // General building area
    Planning,     // Design and planning area
    Protected,    // Critical infrastructure
    Restricted,   // No building allowed
}

/// Temporary permission grants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporaryPermission {
    pub id: String,
    pub project_id: String,
    pub user_id: String,
    pub permission: Permission,
    pub granted_by: String,
    pub granted_at: SystemTime,
    pub expires_at: SystemTime,
    pub reason: String,
    pub used_count: u32,
    pub max_uses: Option<u32>,
}

/// User restrictions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Restriction {
    TimeBasedLimit { start_time: SystemTime, end_time: SystemTime },
    VoxelTypeBlacklist(Vec<VoxelType>),
    AreaRestriction { allowed_bounds: ZoneBounds },
    ActionLimit { max_actions: u32, window: Duration },
}

/// Actions that can be restricted
#[derive(Debug, Clone)]
pub enum RestrictionAction {
    PlaceVoxel { position: Vec3, voxel_type: VoxelType },
    RemoveVoxel { position: Vec3, voxel_type: VoxelType },
}

/// Data for creating new zones
#[derive(Debug, Clone)]
pub struct ZoneCreationData {
    pub name: String,
    pub description: String,
    pub bounds: ZoneBounds,
    pub zone_type: ZoneType,
    pub required_role: ProjectRole,
    pub special_permissions: Vec<Permission>,
    pub access_level: AccessLevel,
}

/// Permission policies and rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionPolicy {
    pub name: String,
    pub description: String,
    pub rules: Vec<PermissionRule>,
}

/// Individual permission rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRule {
    pub condition: RuleCondition,
    pub action: RuleAction,
}

/// Conditions for permission rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleCondition {
    RoleEquals(ProjectRole),
    ZoneType(ZoneType),
    TimeRange { start: SystemTime, end: SystemTime },
    UserHasPermission(Permission),
}

/// Actions taken when rules match
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleAction {
    Allow(Permission),
    Deny(Permission),
    Require(Permission),
}

impl Default for PermissionManager {
    fn default() -> Self {
        Self::new()
    }
}