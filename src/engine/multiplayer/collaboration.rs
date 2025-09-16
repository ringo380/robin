use std::collections::{HashMap, VecDeque, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

use crate::engine::error::RobinResult;
use super::{UserId, SessionId, UserConstruction, TerrainModification};
use super::networking::{NetworkManager, NetworkMessage};
use super::synchronization::{UserCursor, SelectionData, BoundingBox};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborativeEdit {
    pub id: uuid::Uuid,
    pub session_id: SessionId,
    pub edit_type: EditType,
    pub participants: Vec<UserId>,
    pub created_at: u64,
    pub last_modified: u64,
    pub lock_holder: Option<UserId>,
    pub lock_expires: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EditType {
    Construction {
        construction_id: uuid::Uuid,
        edit_mode: ConstructionEditMode,
    },
    Terrain {
        chunk_ids: Vec<[i32; 2]>,
        edit_mode: TerrainEditMode,
    },
    Script {
        script_id: uuid::Uuid,
        node_id: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstructionEditMode {
    Transform,
    Components,
    Properties,
    Deletion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TerrainEditMode {
    Sculpting,
    Painting,
    Vegetation,
    Decoration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedWorkspace {
    pub id: uuid::Uuid,
    pub name: String,
    pub owner: UserId,
    pub participants: HashMap<UserId, ParticipantRole>,
    pub shared_constructions: Vec<uuid::Uuid>,
    pub shared_scripts: Vec<uuid::Uuid>,
    pub permissions: WorkspacePermissions,
    pub created_at: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParticipantRole {
    Owner,
    Editor,
    Viewer,
    Commenter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspacePermissions {
    pub can_invite: Vec<ParticipantRole>,
    pub can_edit_constructions: Vec<ParticipantRole>,
    pub can_edit_terrain: Vec<ParticipantRole>,
    pub can_edit_scripts: Vec<ParticipantRole>,
    pub can_manage_permissions: Vec<ParticipantRole>,
    pub can_export: Vec<ParticipantRole>,
}

impl Default for WorkspacePermissions {
    fn default() -> Self {
        Self {
            can_invite: vec![ParticipantRole::Owner, ParticipantRole::Editor],
            can_edit_constructions: vec![ParticipantRole::Owner, ParticipantRole::Editor],
            can_edit_terrain: vec![ParticipantRole::Owner, ParticipantRole::Editor],
            can_edit_scripts: vec![ParticipantRole::Owner, ParticipantRole::Editor],
            can_manage_permissions: vec![ParticipantRole::Owner],
            can_export: vec![ParticipantRole::Owner, ParticipantRole::Editor, ParticipantRole::Viewer],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: uuid::Uuid,
    pub author: UserId,
    pub content: String,
    pub position: [f32; 3],
    pub target: CommentTarget,
    pub thread_id: Option<uuid::Uuid>,
    pub replies: Vec<uuid::Uuid>,
    pub created_at: u64,
    pub resolved: bool,
    pub resolved_by: Option<UserId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommentTarget {
    Construction(uuid::Uuid),
    Terrain([i32; 2]),
    Script(uuid::Uuid),
    General,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionHistory {
    pub id: uuid::Uuid,
    pub target: HistoryTarget,
    pub versions: VecDeque<VersionEntry>,
    pub max_versions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HistoryTarget {
    Construction(uuid::Uuid),
    Script(uuid::Uuid),
    Workspace(uuid::Uuid),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionEntry {
    pub version_id: uuid::Uuid,
    pub author: UserId,
    pub description: String,
    pub changes: Vec<ChangeRecord>,
    pub created_at: u64,
    pub data_snapshot: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeRecord {
    pub change_type: ChangeType,
    pub field_path: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Added,
    Modified,
    Removed,
    Moved,
}

#[derive(Debug)]
pub struct CollaborationTools {
    pub active_edits: HashMap<uuid::Uuid, CollaborativeEdit>,
    pub workspaces: HashMap<uuid::Uuid, SharedWorkspace>,
    pub comments: HashMap<uuid::Uuid, Comment>,
    pub version_histories: HashMap<uuid::Uuid, VersionHistory>,
    pub user_awareness: UserAwarenessSystem,
    pub permission_manager: PermissionManager,
    pub conflict_resolver: CollaborationConflictResolver,
    pub communication_hub: CommunicationHub,
}

#[derive(Debug)]
pub struct UserAwarenessSystem {
    pub user_cursors: HashMap<UserId, UserCursor>,
    pub user_selections: HashMap<UserId, SelectionData>,
    pub user_focus: HashMap<UserId, FocusData>,
    pub awareness_timeout: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusData {
    pub user_id: UserId,
    pub focus_type: FocusType,
    pub target_id: Option<uuid::Uuid>,
    pub last_activity: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FocusType {
    Construction,
    Terrain,
    Script,
    UI,
    Chat,
}

#[derive(Debug)]
pub struct PermissionManager {
    pub user_permissions: HashMap<UserId, HashMap<uuid::Uuid, ParticipantRole>>,
    pub temporary_permissions: HashMap<UserId, Vec<TemporaryPermission>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporaryPermission {
    pub permission_type: String,
    pub target_id: uuid::Uuid,
    pub granted_by: UserId,
    pub expires_at: u64,
}

#[derive(Debug)]
pub struct CollaborationConflictResolver {
    pub pending_conflicts: Vec<CollaborationConflict>,
    pub resolution_strategies: HashMap<String, ConflictResolutionStrategy>,
}

#[derive(Debug, Clone)]
pub struct CollaborationConflict {
    pub id: uuid::Uuid,
    pub conflict_type: CollaborationConflictType,
    pub participants: Vec<UserId>,
    pub target: ConflictTarget,
    pub created_at: Instant,
    pub resolution_deadline: Option<Instant>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CollaborationConflictType {
    SimultaneousEdit,
    PermissionConflict,
    LockContention,
    VersionMismatch,
}

#[derive(Debug, Clone)]
pub enum ConflictTarget {
    Construction(uuid::Uuid),
    Script(uuid::Uuid),
    Workspace(uuid::Uuid),
}

#[derive(Debug, Clone)]
pub enum ConflictResolutionStrategy {
    FirstComeFirstServe,
    OwnerPriority,
    MergeChanges,
    UserVoting,
    AutomaticMerge,
}

#[derive(Debug)]
pub struct CommunicationHub {
    pub chat_rooms: HashMap<SessionId, ChatRoom>,
    pub voice_channels: HashMap<SessionId, VoiceChannel>,
    pub screen_sharing: HashMap<UserId, ScreenShare>,
    pub notifications: VecDeque<Notification>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRoom {
    pub session_id: SessionId,
    pub messages: VecDeque<ChatMessage>,
    pub participants: HashSet<UserId>,
    pub max_messages: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: uuid::Uuid,
    pub author: UserId,
    pub content: String,
    pub message_type: ChatMessageType,
    pub timestamp: u64,
    pub mentions: Vec<UserId>,
    pub attachments: Vec<MessageAttachment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatMessageType {
    Text,
    System,
    Command,
    Reaction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAttachment {
    pub attachment_type: AttachmentType,
    pub data: Vec<u8>,
    pub filename: String,
    pub size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttachmentType {
    Image,
    Audio,
    Video,
    Document,
    Screenshot,
    Construction,
}

#[derive(Debug, Clone)]
pub struct VoiceChannel {
    pub session_id: SessionId,
    pub participants: HashSet<UserId>,
    pub audio_quality: AudioQuality,
    pub noise_suppression: bool,
    pub echo_cancellation: bool,
}

#[derive(Debug, Clone)]
pub enum AudioQuality {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone)]
pub struct ScreenShare {
    pub user_id: UserId,
    pub viewers: HashSet<UserId>,
    pub region: Option<BoundingBox>,
    pub quality: VideoQuality,
    pub frame_rate: u32,
}

#[derive(Debug, Clone)]
pub enum VideoQuality {
    Low,
    Medium,
    High,
    Ultra,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: uuid::Uuid,
    pub recipient: UserId,
    pub sender: Option<UserId>,
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
    pub created_at: u64,
    pub read: bool,
    pub action_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    Mention,
    Invitation,
    Edit,
    Comment,
    SystemMessage,
    Warning,
    Error,
}

impl CollaborationTools {
    pub fn new() -> RobinResult<Self> {
        Ok(Self {
            active_edits: HashMap::new(),
            workspaces: HashMap::new(),
            comments: HashMap::new(),
            version_histories: HashMap::new(),
            user_awareness: UserAwarenessSystem {
                user_cursors: HashMap::new(),
                user_selections: HashMap::new(),
                user_focus: HashMap::new(),
                awareness_timeout: Duration::from_secs(30),
            },
            permission_manager: PermissionManager {
                user_permissions: HashMap::new(),
                temporary_permissions: HashMap::new(),
            },
            conflict_resolver: CollaborationConflictResolver {
                pending_conflicts: Vec::new(),
                resolution_strategies: HashMap::new(),
            },
            communication_hub: CommunicationHub {
                chat_rooms: HashMap::new(),
                voice_channels: HashMap::new(),
                screen_sharing: HashMap::new(),
                notifications: VecDeque::new(),
            },
        })
    }

    pub fn update(&mut self, _delta_time: f32) -> RobinResult<()> {
        self.update_user_awareness()?;
        self.resolve_collaboration_conflicts()?;
        self.cleanup_expired_locks()?;
        self.process_notifications()?;
        Ok(())
    }

    pub fn create_workspace(&mut self, name: String, owner: UserId) -> RobinResult<uuid::Uuid> {
        let workspace_id = uuid::Uuid::new_v4();
        let mut participants = HashMap::new();
        participants.insert(owner.clone(), ParticipantRole::Owner);

        let workspace = SharedWorkspace {
            id: workspace_id,
            name,
            owner,
            participants,
            shared_constructions: Vec::new(),
            shared_scripts: Vec::new(),
            permissions: WorkspacePermissions::default(),
            created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };

        self.workspaces.insert(workspace_id, workspace);
        Ok(workspace_id)
    }

    pub fn invite_to_workspace(&mut self, workspace_id: uuid::Uuid, inviter: UserId, invitee: UserId, role: ParticipantRole) -> RobinResult<()> {
        if let Some(workspace) = self.workspaces.get_mut(&workspace_id) {
            if let Some(inviter_role) = workspace.participants.get(&inviter) {
                if workspace.permissions.can_invite.contains(inviter_role) {
                    workspace.participants.insert(invitee.clone(), role);
                    
                    let notification = Notification {
                        id: uuid::Uuid::new_v4(),
                        recipient: invitee,
                        sender: Some(inviter),
                        notification_type: NotificationType::Invitation,
                        title: "Workspace Invitation".to_string(),
                        message: format!("You've been invited to join workspace '{}'", workspace.name),
                        created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                        read: false,
                        action_url: Some(format!("/workspace/{}", workspace_id)),
                    };
                    
                    self.communication_hub.notifications.push_back(notification);
                    Ok(())
                } else {
                    Err(crate::engine::error::RobinError::CollaborationError("Insufficient permissions to invite users".to_string()))
                }
            } else {
                Err(crate::engine::error::RobinError::CollaborationError("User not found in workspace".to_string()))
            }
        } else {
            Err(crate::engine::error::RobinError::CollaborationError("Workspace not found".to_string()))
        }
    }

    pub fn start_collaborative_edit(&mut self, edit_type: EditType, user_id: UserId, session_id: SessionId) -> RobinResult<uuid::Uuid> {
        let edit_id = uuid::Uuid::new_v4();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        let edit = CollaborativeEdit {
            id: edit_id,
            session_id,
            edit_type,
            participants: vec![user_id.clone()],
            created_at: now,
            last_modified: now,
            lock_holder: Some(user_id),
            lock_expires: Some(now + 300), // 5 minute lock
        };

        self.active_edits.insert(edit_id, edit);
        Ok(edit_id)
    }

    pub fn join_collaborative_edit(&mut self, edit_id: uuid::Uuid, user_id: UserId) -> RobinResult<()> {
        if let Some(edit) = self.active_edits.get_mut(&edit_id) {
            if !edit.participants.contains(&user_id) {
                edit.participants.push(user_id);
                edit.last_modified = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            }
            Ok(())
        } else {
            Err(crate::engine::error::RobinError::CollaborationError("Collaborative edit not found".to_string()))
        }
    }

    pub fn leave_collaborative_edit(&mut self, edit_id: uuid::Uuid, user_id: UserId) -> RobinResult<()> {
        if let Some(edit) = self.active_edits.get_mut(&edit_id) {
            edit.participants.retain(|id| *id != user_id);
            
            if edit.lock_holder == Some(user_id.clone()) {
                edit.lock_holder = edit.participants.first().cloned();
                if let Some(new_holder) = &edit.lock_holder {
                    edit.lock_expires = Some(
                        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 300
                    );
                } else {
                    edit.lock_expires = None;
                }
            }
            
            if edit.participants.is_empty() {
                self.active_edits.remove(&edit_id);
            }
            
            Ok(())
        } else {
            Err(crate::engine::error::RobinError::CollaborationError("Collaborative edit not found".to_string()))
        }
    }

    pub fn add_comment(&mut self, content: String, position: [f32; 3], target: CommentTarget, author: UserId) -> RobinResult<uuid::Uuid> {
        let comment_id = uuid::Uuid::new_v4();
        
        let comment = Comment {
            id: comment_id,
            author: author.clone(),
            content,
            position,
            target,
            thread_id: None,
            replies: Vec::new(),
            created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            resolved: false,
            resolved_by: None,
        };

        self.comments.insert(comment_id, comment);
        Ok(comment_id)
    }

    pub fn reply_to_comment(&mut self, parent_id: uuid::Uuid, content: String, author: UserId) -> RobinResult<uuid::Uuid> {
        let reply_id = uuid::Uuid::new_v4();
        
        if let Some(parent_comment) = self.comments.get_mut(&parent_id) {
            parent_comment.replies.push(reply_id);
            
            let reply = Comment {
                id: reply_id,
                author,
                content,
                position: parent_comment.position,
                target: parent_comment.target.clone(),
                thread_id: Some(parent_id),
                replies: Vec::new(),
                created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                resolved: false,
                resolved_by: None,
            };
            
            self.comments.insert(reply_id, reply);
            Ok(reply_id)
        } else {
            Err(crate::engine::error::RobinError::CollaborationError("Parent comment not found".to_string()))
        }
    }

    pub fn resolve_comment(&mut self, comment_id: uuid::Uuid, resolver: UserId) -> RobinResult<()> {
        if let Some(comment) = self.comments.get_mut(&comment_id) {
            comment.resolved = true;
            comment.resolved_by = Some(resolver);
            Ok(())
        } else {
            Err(crate::engine::error::RobinError::CollaborationError("Comment not found".to_string()))
        }
    }

    pub fn create_version_snapshot(&mut self, target: HistoryTarget, author: UserId, description: String, data: Vec<u8>) -> RobinResult<uuid::Uuid> {
        let version_id = uuid::Uuid::new_v4();
        let history_id = match &target {
            HistoryTarget::Construction(id) => *id,
            HistoryTarget::Script(id) => *id,
            HistoryTarget::Workspace(id) => *id,
        };

        let version_entry = VersionEntry {
            version_id,
            author,
            description,
            changes: Vec::new(), // TODO: Calculate actual changes
            created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            data_snapshot: data,
        };

        let history = self.version_histories.entry(history_id).or_insert_with(|| {
            VersionHistory {
                id: history_id,
                target: target.clone(),
                versions: VecDeque::new(),
                max_versions: 50,
            }
        });

        history.versions.push_back(version_entry);
        if history.versions.len() > history.max_versions {
            history.versions.pop_front();
        }

        Ok(version_id)
    }

    pub fn send_chat_message(&mut self, session_id: SessionId, author: UserId, content: String) -> RobinResult<uuid::Uuid> {
        let message_id = uuid::Uuid::new_v4();
        
        let message = ChatMessage {
            id: message_id,
            author: author.clone(),
            content,
            message_type: ChatMessageType::Text,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            mentions: Vec::new(), // TODO: Parse mentions from content
            attachments: Vec::new(),
        };

        let chat_room = self.communication_hub.chat_rooms.entry(session_id.clone()).or_insert_with(|| {
            ChatRoom {
                session_id,
                messages: VecDeque::new(),
                participants: HashSet::new(),
                max_messages: 1000,
            }
        });

        chat_room.participants.insert(author);
        chat_room.messages.push_back(message);
        
        if chat_room.messages.len() > chat_room.max_messages {
            chat_room.messages.pop_front();
        }

        Ok(message_id)
    }

    pub fn start_screen_share(&mut self, user_id: UserId, region: Option<BoundingBox>, quality: VideoQuality) -> RobinResult<()> {
        let screen_share = ScreenShare {
            user_id: user_id.clone(),
            viewers: HashSet::new(),
            region,
            quality,
            frame_rate: 30,
        };

        self.communication_hub.screen_sharing.insert(user_id, screen_share);
        Ok(())
    }

    pub fn join_screen_share(&mut self, sharer: UserId, viewer: UserId) -> RobinResult<()> {
        if let Some(screen_share) = self.communication_hub.screen_sharing.get_mut(&sharer) {
            screen_share.viewers.insert(viewer);
            Ok(())
        } else {
            Err(crate::engine::error::RobinError::CollaborationError("Screen share not found".to_string()))
        }
    }

    fn update_user_awareness(&mut self) -> RobinResult<()> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let timeout_threshold = now - self.user_awareness.awareness_timeout.as_secs();

        self.user_awareness.user_cursors.retain(|_, cursor| cursor.last_activity > timeout_threshold);
        self.user_awareness.user_focus.retain(|_, focus| focus.last_activity > timeout_threshold);

        Ok(())
    }

    fn resolve_collaboration_conflicts(&mut self) -> RobinResult<()> {
        let mut resolved_conflicts = Vec::new();
        let conflicts_to_resolve = self.conflict_resolver.pending_conflicts.clone();

        for (index, conflict) in conflicts_to_resolve.iter().enumerate() {
            match self.attempt_conflict_resolution(conflict) {
                Ok(true) => resolved_conflicts.push(index),
                Ok(false) => {}, // Still pending
                Err(e) => eprintln!("Failed to resolve conflict {}: {}", conflict.id, e),
            }
        }

        for &index in resolved_conflicts.iter().rev() {
            self.conflict_resolver.pending_conflicts.remove(index);
        }

        Ok(())
    }

    fn attempt_conflict_resolution(&mut self, _conflict: &CollaborationConflict) -> RobinResult<bool> {
        Ok(true)
    }

    fn cleanup_expired_locks(&mut self) -> RobinResult<()> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let mut expired_edits = Vec::new();

        for (edit_id, edit) in &self.active_edits {
            if let Some(expires_at) = edit.lock_expires {
                if now > expires_at {
                    expired_edits.push(*edit_id);
                }
            }
        }

        for edit_id in expired_edits {
            if let Some(edit) = self.active_edits.get_mut(&edit_id) {
                edit.lock_holder = edit.participants.first().cloned();
                if edit.lock_holder.is_some() {
                    edit.lock_expires = Some(now + 300); // Extend for 5 more minutes
                } else {
                    edit.lock_expires = None;
                    self.active_edits.remove(&edit_id);
                }
            }
        }

        Ok(())
    }

    fn process_notifications(&mut self) -> RobinResult<()> {
        while self.communication_hub.notifications.len() > 1000 {
            self.communication_hub.notifications.pop_front();
        }
        Ok(())
    }

    pub fn get_workspace_participants(&self, workspace_id: uuid::Uuid) -> Vec<(UserId, ParticipantRole)> {
        if let Some(workspace) = self.workspaces.get(&workspace_id) {
            workspace.participants.iter().map(|(id, role)| (id.clone(), role.clone())).collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_active_edits_for_user(&self, user_id: &UserId) -> Vec<CollaborativeEdit> {
        self.active_edits.values()
            .filter(|edit| edit.participants.contains(user_id))
            .cloned()
            .collect()
    }

    pub fn get_comments_for_target(&self, target: &CommentTarget) -> Vec<Comment> {
        self.comments.values()
            .filter(|comment| std::mem::discriminant(&comment.target) == std::mem::discriminant(target))
            .cloned()
            .collect()
    }

    pub fn get_version_history(&self, target_id: uuid::Uuid) -> Option<&VersionHistory> {
        self.version_histories.get(&target_id)
    }

    pub fn get_unread_notifications(&self, user_id: &UserId) -> Vec<Notification> {
        self.communication_hub.notifications.iter()
            .filter(|notification| notification.recipient == *user_id && !notification.read)
            .cloned()
            .collect()
    }

    pub fn mark_notification_read(&mut self, notification_id: uuid::Uuid) -> RobinResult<()> {
        for notification in &mut self.communication_hub.notifications {
            if notification.id == notification_id {
                notification.read = true;
                return Ok(());
            }
        }
        Err(crate::engine::error::RobinError::CollaborationError("Notification not found".to_string()))
    }

    // Additional methods needed by MultiplayerManager
    pub fn add_user(&mut self, user_id: UserId) -> RobinResult<()> {
        self.user_awareness.user_cursors.insert(
            user_id.clone(),
            UserCursor {
                user_id,
                position: [0.0, 0.0, 0.0],
                tool: None,
                selection: None,
                last_activity: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            }
        );
        Ok(())
    }

    pub fn remove_user(&mut self, user_id: &UserId) -> RobinResult<()> {
        self.user_awareness.user_cursors.remove(user_id);
        self.user_awareness.user_selections.remove(user_id);
        self.user_awareness.user_focus.remove(user_id);

        // Remove user from active edits
        let edit_ids: Vec<_> = self.active_edits.keys().cloned().collect();
        for edit_id in edit_ids {
            self.leave_collaborative_edit(edit_id, user_id.clone())?;
        }

        Ok(())
    }

    pub fn sync_all_changes(&mut self) -> RobinResult<u32> {
        let mut sync_count = 0;

        // Process pending notifications
        self.process_notifications()?;
        sync_count += 1;

        // Update user awareness
        self.update_user_awareness()?;
        sync_count += 1;

        // Resolve conflicts
        self.resolve_collaboration_conflicts()?;
        sync_count += 1;

        // Clean up expired locks
        self.cleanup_expired_locks()?;
        sync_count += 1;

        Ok(sync_count)
    }

    pub fn broadcast_user_update(&mut self, user_id: &UserId, user: &super::User) -> RobinResult<()> {
        // Update user cursor position based on user location
        if let Some(cursor) = self.user_awareness.user_cursors.get_mut(user_id) {
            cursor.position = user.location.world_position;
            cursor.last_activity = user.last_activity as u64;
        }

        // Broadcast to all chat rooms where this user is a participant
        for (_, chat_room) in &mut self.communication_hub.chat_rooms {
            if chat_room.participants.contains(user_id) {
                let system_message = ChatMessage {
                    id: uuid::Uuid::new_v4(),
                    author: user_id.clone(),
                    content: format!("User {} moved to {:?}", user.display_name, user.location.world_position),
                    message_type: ChatMessageType::System,
                    timestamp: user.last_activity as u64,
                    mentions: Vec::new(),
                    attachments: Vec::new(),
                };
                chat_room.messages.push_back(system_message);

                if chat_room.messages.len() > chat_room.max_messages {
                    chat_room.messages.pop_front();
                }
            }
        }

        Ok(())
    }
}