use crate::engine::error::RobinResult;
use crate::engine::multiplayer::UserId;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Role {
    Owner,
    Admin,
    Moderator,
    Builder,
    Viewer,
    Guest,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Permission {
    EditWorld,
    CreateStructures,
    DeleteStructures,
    UseAdvancedTools,
    ManageUsers,
    ManagePermissions,
    AccessVoiceChat,
    ShareAssets,
    ViewPrivateAreas,
    ModifyTerrain,
    SpawnEntities,
    ExecuteScripts,
    ManageVersionControl,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessLevel {
    Full,
    ReadOnly,
    Limited(Vec<Permission>),
    Denied,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermissions {
    pub user_id: UserId,
    pub role: Role,
    pub permissions: HashSet<Permission>,
    pub area_restrictions: Vec<AreaPermission>,
    pub granted_at: f64,
    pub expires_at: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AreaPermission {
    pub area_id: String,
    pub chunk_ids: Vec<String>,
    pub access_level: AccessLevel,
    pub permissions: HashSet<Permission>,
}

impl UserPermissions {
    pub fn from_capabilities(capabilities: &crate::engine::multiplayer::UserCapabilities) -> Self {
        let mut permissions = HashSet::new();
        
        if capabilities.can_edit_world {
            permissions.insert(Permission::EditWorld);
        }
        if capabilities.can_use_advanced_tools {
            permissions.insert(Permission::UseAdvancedTools);
        }
        if capabilities.can_manage_permissions {
            permissions.insert(Permission::ManagePermissions);
        }
        if capabilities.can_access_voice_chat {
            permissions.insert(Permission::AccessVoiceChat);
        }
        if capabilities.can_share_assets {
            permissions.insert(Permission::ShareAssets);
        }

        Self {
            user_id: UserId::new("placeholder".to_string()),
            role: Role::Builder,
            permissions,
            area_restrictions: Vec::new(),
            granted_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
            expires_at: None,
        }
    }

    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.contains(permission)
    }

    pub fn can_access_area(&self, area_id: &str, chunk_id: &str) -> bool {
        if let Some(area_perm) = self.area_restrictions.iter().find(|ap| ap.area_id == area_id) {
            area_perm.chunk_ids.contains(&chunk_id.to_string()) && 
            !matches!(area_perm.access_level, AccessLevel::Denied)
        } else {
            true // No restrictions = full access
        }
    }
}

#[derive(Debug)]
pub struct PermissionManager {
    user_permissions: HashMap<UserId, UserPermissions>,
    role_definitions: HashMap<Role, HashSet<Permission>>,
    area_permissions: HashMap<String, HashMap<UserId, AreaPermission>>,
}

impl PermissionManager {
    pub fn new() -> RobinResult<Self> {
        let mut manager = Self {
            user_permissions: HashMap::new(),
            role_definitions: HashMap::new(),
            area_permissions: HashMap::new(),
        };
        
        manager.setup_default_roles();
        Ok(manager)
    }

    fn setup_default_roles(&mut self) {
        let mut owner_perms = HashSet::new();
        owner_perms.insert(Permission::EditWorld);
        owner_perms.insert(Permission::CreateStructures);
        owner_perms.insert(Permission::DeleteStructures);
        owner_perms.insert(Permission::UseAdvancedTools);
        owner_perms.insert(Permission::ManageUsers);
        owner_perms.insert(Permission::ManagePermissions);
        owner_perms.insert(Permission::AccessVoiceChat);
        owner_perms.insert(Permission::ShareAssets);
        owner_perms.insert(Permission::ViewPrivateAreas);
        owner_perms.insert(Permission::ModifyTerrain);
        owner_perms.insert(Permission::SpawnEntities);
        owner_perms.insert(Permission::ExecuteScripts);
        owner_perms.insert(Permission::ManageVersionControl);
        self.role_definitions.insert(Role::Owner, owner_perms);

        let mut builder_perms = HashSet::new();
        builder_perms.insert(Permission::EditWorld);
        builder_perms.insert(Permission::CreateStructures);
        builder_perms.insert(Permission::UseAdvancedTools);
        builder_perms.insert(Permission::AccessVoiceChat);
        builder_perms.insert(Permission::ShareAssets);
        self.role_definitions.insert(Role::Builder, builder_perms);

        let mut viewer_perms = HashSet::new();
        viewer_perms.insert(Permission::AccessVoiceChat);
        self.role_definitions.insert(Role::Viewer, viewer_perms);
    }

    pub fn grant_user_permissions(&mut self, user_id: UserId, mut permissions: UserPermissions) -> RobinResult<()> {
        permissions.user_id = user_id.clone();
        
        // Add role-based permissions
        if let Some(role_perms) = self.role_definitions.get(&permissions.role) {
            permissions.permissions.extend(role_perms.clone());
        }
        
        self.user_permissions.insert(user_id, permissions);
        Ok(())
    }

    pub fn revoke_user_permissions(&mut self, user_id: &UserId) -> RobinResult<()> {
        self.user_permissions.remove(user_id);
        Ok(())
    }

    pub fn check_permission(&self, user_id: &UserId, permission: &Permission) -> bool {
        if let Some(user_perms) = self.user_permissions.get(user_id) {
            user_perms.has_permission(permission)
        } else {
            false
        }
    }

    pub fn check_area_access(&self, user_id: &UserId, area_id: &str, chunk_id: &str) -> bool {
        if let Some(user_perms) = self.user_permissions.get(user_id) {
            user_perms.can_access_area(area_id, chunk_id)
        } else {
            false
        }
    }
}