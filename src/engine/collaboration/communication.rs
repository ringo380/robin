/*!
 * Communication and Messaging System for Collaborative Building
 *
 * Handles in-game chat, voice communication, annotations, and
 * collaborative messaging for professional engineering teams.
 */

use crate::engine::{
    error::{RobinError, RobinResult},
    math::{Vec3, InnerSpace},
    collaboration::AnnotationType,
};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, Duration};

/// Core communication management system
pub struct CommunicationManager {
    /// Message history by channel
    message_channels: HashMap<String, MessageChannel>,
    /// Active annotations in the world
    annotations: HashMap<String, Annotation>,
    /// Voice communication sessions
    voice_sessions: HashMap<String, VoiceSession>,
    /// Communication settings
    settings: CommunicationSettings,
    /// Message filters and moderation
    filters: MessageFilters,
    /// Statistics
    stats: CommunicationStats,
}

impl CommunicationManager {
    pub fn new() -> Self {
        let mut manager = Self {
            message_channels: HashMap::new(),
            annotations: HashMap::new(),
            voice_sessions: HashMap::new(),
            settings: CommunicationSettings::default(),
            filters: MessageFilters::new(),
            stats: CommunicationStats::default(),
        };

        manager.initialize_default_channels();
        manager
    }

    /// Initialize default communication channels
    fn initialize_default_channels(&mut self) {
        // General project channel
        let general_channel = MessageChannel {
            id: "general".to_string(),
            name: "General".to_string(),
            description: "General project discussion".to_string(),
            channel_type: ChannelType::Public,
            messages: VecDeque::new(),
            participants: Vec::new(),
            created_at: SystemTime::now(),
            settings: ChannelSettings::default(),
        };

        // Team coordination channel
        let team_channel = MessageChannel {
            id: "team".to_string(),
            name: "Team Coordination".to_string(),
            description: "Team coordination and task discussion".to_string(),
            channel_type: ChannelType::Team,
            messages: VecDeque::new(),
            participants: Vec::new(),
            created_at: SystemTime::now(),
            settings: ChannelSettings::default(),
        };

        // Technical discussion channel
        let tech_channel = MessageChannel {
            id: "technical".to_string(),
            name: "Technical Discussion".to_string(),
            description: "Technical questions and engineering discussion".to_string(),
            channel_type: ChannelType::Technical,
            messages: VecDeque::new(),
            participants: Vec::new(),
            created_at: SystemTime::now(),
            settings: ChannelSettings::default(),
        };

        self.message_channels.insert("general".to_string(), general_channel);
        self.message_channels.insert("team".to_string(), team_channel);
        self.message_channels.insert("technical".to_string(), tech_channel);
    }

    /// Send message to channel
    pub fn send_message(&mut self, message: Message) -> RobinResult<()> {
        // Apply message filters
        if !self.filters.is_message_allowed(&message) {
            return Err(RobinError::InvalidInput("Message blocked by filters".to_string()));
        }

        // Determine target channel
        let channel_id = self.determine_message_channel(&message);

        if let Some(channel) = self.message_channels.get_mut(&channel_id) {
            // Add message to channel
            channel.messages.push_back(message.clone());

            // Limit channel history
            while channel.messages.len() > channel.settings.max_messages {
                channel.messages.pop_front();
            }

            // Update statistics
            self.stats.total_messages_sent += 1;
            match message.message_type {
                MessageType::Text => self.stats.text_messages += 1,
                MessageType::System => self.stats.system_messages += 1,
                MessageType::Annotation => self.stats.annotations_created += 1,
                MessageType::Voice => self.stats.voice_messages += 1,
                MessageType::Image => self.stats.media_messages += 1,
                MessageType::File => self.stats.media_messages += 1,
                MessageType::Command => self.stats.command_messages += 1,
            }

            Ok(())
        } else {
            Err(RobinError::NotFound(format!("Channel {} not found", channel_id)))
        }
    }

    /// Receive message from network
    pub fn receive_message(&mut self, message: Message) -> RobinResult<()> {
        self.send_message(message)
    }

    /// Create annotation at world position
    pub fn add_annotation(&mut self, annotation: Annotation) -> RobinResult<()> {
        // Validate annotation position and content
        if annotation.content.trim().is_empty() {
            return Err(RobinError::InvalidInput("Annotation content cannot be empty".to_string()));
        }

        self.annotations.insert(annotation.id.clone(), annotation);
        self.stats.annotations_created += 1;
        Ok(())
    }

    /// Remove annotation
    pub fn remove_annotation(&mut self, annotation_id: &str, user_id: &str) -> RobinResult<()> {
        if let Some(annotation) = self.annotations.get(annotation_id) {
            // Check if user can remove this annotation
            if annotation.author != user_id {
                // In real implementation, would check if user has moderation permissions
                return Err(RobinError::PermissionDenied("Cannot remove other user's annotation".to_string()));
            }

            self.annotations.remove(annotation_id);
            Ok(())
        } else {
            Err(RobinError::NotFound("Annotation not found".to_string()))
        }
    }

    /// Start voice communication session
    pub fn start_voice_session(&mut self, session_id: String, participants: Vec<String>) -> RobinResult<()> {
        let voice_session = VoiceSession {
            id: session_id.clone(),
            participants,
            started_at: SystemTime::now(),
            status: VoiceSessionStatus::Active,
            quality_metrics: VoiceQualityMetrics::default(),
        };

        self.voice_sessions.insert(session_id, voice_session);
        self.stats.voice_sessions_started += 1;
        Ok(())
    }

    /// End voice communication session
    pub fn end_voice_session(&mut self, session_id: &str) -> RobinResult<()> {
        if let Some(mut session) = self.voice_sessions.remove(session_id) {
            session.status = VoiceSessionStatus::Ended;
            // In real implementation, would save session history
            Ok(())
        } else {
            Err(RobinError::NotFound("Voice session not found".to_string()))
        }
    }

    /// Update communication system
    pub fn update(&mut self, delta_time: f32) -> RobinResult<Vec<Message>> {
        let mut new_messages = Vec::new();

        // Process any pending messages or notifications
        // In real implementation, this would handle:
        // - Voice session management
        // - Message delivery confirmations
        // - Typing indicators
        // - Presence updates

        // Clean up old messages based on channel settings
        for channel in self.message_channels.values_mut() {
            if let Some(retention) = channel.settings.message_retention {
                let cutoff_time = SystemTime::now() - retention;
                channel.messages.retain(|msg| msg.timestamp > cutoff_time);
            }
        }

        // Clean up expired annotations
        if self.settings.annotation_expiry_enabled {
            let cutoff_time = SystemTime::now() - self.settings.annotation_expiry_duration;
            self.annotations.retain(|_, annotation| annotation.timestamp > cutoff_time);
        }

        Ok(new_messages)
    }

    /// Get messages from channel
    pub fn get_channel_messages(&self, channel_id: &str, limit: Option<usize>) -> Vec<&Message> {
        if let Some(channel) = self.message_channels.get(channel_id) {
            let messages: Vec<&Message> = channel.messages.iter().collect();
            if let Some(limit) = limit {
                messages.into_iter().rev().take(limit).collect()
            } else {
                messages
            }
        } else {
            Vec::new()
        }
    }

    /// Get annotations near position
    pub fn get_annotations_near(&self, position: Vec3, radius: f32) -> Vec<&Annotation> {
        self.annotations.values()
            .filter(|annotation| {
                let distance = (annotation.position - position).magnitude();
                distance <= radius
            })
            .collect()
    }

    /// Get all annotations
    pub fn get_all_annotations(&self) -> Vec<&Annotation> {
        self.annotations.values().collect()
    }

    /// Create private channel between users
    pub fn create_private_channel(&mut self, user_ids: Vec<String>) -> RobinResult<String> {
        let channel_id = format!("private_{}", uuid::Uuid::new_v4());

        let channel = MessageChannel {
            id: channel_id.clone(),
            name: "Private Discussion".to_string(),
            description: format!("Private channel for {} users", user_ids.len()),
            channel_type: ChannelType::Private,
            messages: VecDeque::new(),
            participants: user_ids,
            created_at: SystemTime::now(),
            settings: ChannelSettings {
                max_messages: 1000,
                message_retention: Some(Duration::from_secs(30 * 24 * 60 * 60)), // 30 days
                moderation_enabled: false,
                file_sharing_enabled: true,
            },
        };

        self.message_channels.insert(channel_id.clone(), channel);
        Ok(channel_id)
    }

    /// Set user typing status
    pub fn set_typing_status(&mut self, user_id: &str, channel_id: &str, is_typing: bool) -> RobinResult<()> {
        // In real implementation, would manage typing indicators
        // For now, just send a system message if needed
        if is_typing {
            // Could send typing indicator to other users
        }
        Ok(())
    }

    /// Get message count
    pub fn get_message_count(&self) -> usize {
        self.stats.total_messages_sent
    }

    /// Get communication statistics
    pub fn get_stats(&self) -> &CommunicationStats {
        &self.stats
    }

    /// Determine which channel a message should go to
    fn determine_message_channel(&self, message: &Message) -> String {
        match message.message_type {
            MessageType::System => "general".to_string(),
            MessageType::Annotation => "general".to_string(),
            MessageType::Command => "team".to_string(),
            MessageType::Text => {
                // Check message content for channel hints
                if message.content.contains("@team") || message.content.contains("task") {
                    "team".to_string()
                } else if message.content.contains("technical") || message.content.contains("engineering") {
                    "technical".to_string()
                } else {
                    "general".to_string()
                }
            }
            _ => "general".to_string(),
        }
    }
}

/// Message structure for communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub sender_id: String,
    pub message_type: MessageType,
    pub content: String,
    pub timestamp: SystemTime,
    pub position: Option<Vec3>, // For location-based messages
}

/// Different types of messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    Text,       // Regular chat message
    System,     // System notification
    Annotation, // World annotation
    Voice,      // Voice message
    Image,      // Image/screenshot
    File,       // File attachment
    Command,    // Command or action
}

/// World annotation structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub id: String,
    pub position: Vec3,
    pub content: String,
    pub author: String,
    pub timestamp: SystemTime,
    pub annotation_type: AnnotationType,
}

/// Message channel for organized communication
#[derive(Debug, Clone)]
pub struct MessageChannel {
    pub id: String,
    pub name: String,
    pub description: String,
    pub channel_type: ChannelType,
    pub messages: VecDeque<Message>,
    pub participants: Vec<String>,
    pub created_at: SystemTime,
    pub settings: ChannelSettings,
}

/// Different channel types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelType {
    Public,     // Open to all project members
    Team,       // Team leads and above
    Technical,  // Technical discussion
    Private,    // Private between specific users
    Broadcast,  // One-way announcements
}

/// Channel-specific settings
#[derive(Debug, Clone)]
pub struct ChannelSettings {
    pub max_messages: usize,
    pub message_retention: Option<Duration>,
    pub moderation_enabled: bool,
    pub file_sharing_enabled: bool,
}

impl Default for ChannelSettings {
    fn default() -> Self {
        Self {
            max_messages: 500,
            message_retention: Some(Duration::from_secs(7 * 24 * 60 * 60)), // 7 days
            moderation_enabled: false,
            file_sharing_enabled: true,
        }
    }
}

/// Voice communication session
#[derive(Debug, Clone)]
pub struct VoiceSession {
    pub id: String,
    pub participants: Vec<String>,
    pub started_at: SystemTime,
    pub status: VoiceSessionStatus,
    pub quality_metrics: VoiceQualityMetrics,
}

/// Voice session status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoiceSessionStatus {
    Starting,
    Active,
    Paused,
    Ended,
    Error,
}

/// Voice quality metrics
#[derive(Debug, Clone, Default)]
pub struct VoiceQualityMetrics {
    pub average_latency: f32,
    pub packet_loss_rate: f32,
    pub audio_quality_score: f32,
}

/// Communication system settings
#[derive(Debug, Clone)]
pub struct CommunicationSettings {
    pub max_message_length: usize,
    pub profanity_filter_enabled: bool,
    pub annotation_expiry_enabled: bool,
    pub annotation_expiry_duration: Duration,
    pub voice_enabled: bool,
    pub file_sharing_enabled: bool,
    pub max_file_size: usize,
}

impl Default for CommunicationSettings {
    fn default() -> Self {
        Self {
            max_message_length: 2000,
            profanity_filter_enabled: true,
            annotation_expiry_enabled: false,
            annotation_expiry_duration: Duration::from_secs(24 * 60 * 60), // 24 hours
            voice_enabled: true,
            file_sharing_enabled: true,
            max_file_size: 10 * 1024 * 1024, // 10 MB
        }
    }
}

/// Message filtering and moderation
#[derive(Debug, Clone)]
pub struct MessageFilters {
    pub profanity_list: Vec<String>,
    pub spam_detection_enabled: bool,
    pub rate_limits: HashMap<String, RateLimit>,
}

impl MessageFilters {
    pub fn new() -> Self {
        Self {
            profanity_list: vec!["spam".to_string(), "inappropriate".to_string()], // Basic list
            spam_detection_enabled: true,
            rate_limits: HashMap::new(),
        }
    }

    pub fn is_message_allowed(&mut self, message: &Message) -> bool {
        // Check message length
        if message.content.len() > 2000 {
            return false;
        }

        // Check for profanity if enabled
        if !self.profanity_list.is_empty() {
            let content_lower = message.content.to_lowercase();
            for word in &self.profanity_list {
                if content_lower.contains(word) {
                    return false;
                }
            }
        }

        // Check rate limits
        if let Some(rate_limit) = self.rate_limits.get_mut(&message.sender_id) {
            if !rate_limit.is_allowed() {
                return false;
            }
        }

        true
    }
}

/// Rate limiting for messages
#[derive(Debug, Clone)]
pub struct RateLimit {
    pub messages_per_minute: u32,
    pub current_count: u32,
    pub window_start: SystemTime,
}

impl RateLimit {
    pub fn is_allowed(&mut self) -> bool {
        let now = SystemTime::now();
        let elapsed = now.duration_since(self.window_start).unwrap_or_default();

        if elapsed > Duration::from_secs(60) {
            // Reset window
            self.window_start = now;
            self.current_count = 1;
            true
        } else if self.current_count >= self.messages_per_minute {
            false
        } else {
            self.current_count += 1;
            true
        }
    }
}

/// Communication statistics
#[derive(Debug, Clone, Default)]
pub struct CommunicationStats {
    pub total_messages_sent: usize,
    pub text_messages: usize,
    pub system_messages: usize,
    pub annotations_created: usize,
    pub voice_messages: usize,
    pub media_messages: usize,
    pub command_messages: usize,
    pub voice_sessions_started: usize,
    pub average_message_length: f32,
    pub most_active_channel: String,
}

impl Default for CommunicationManager {
    fn default() -> Self {
        Self::new()
    }
}