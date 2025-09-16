use crate::engine::error::RobinResult;
use crate::engine::multiplayer::UserId;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, VecDeque};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Text,
    Voice,
    System,
    Whisper,
    Broadcast,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub sender: UserId,
    pub recipient: Option<UserId>,
    pub channel: String,
    pub content: String,
    pub message_type: MessageType,
    pub timestamp: f64,
    pub edited: bool,
    pub reactions: HashMap<String, Vec<UserId>>,
}

#[derive(Debug)]
pub struct ChatManager {
    messages: HashMap<String, VecDeque<ChatMessage>>,
    max_history: usize,
}

impl ChatManager {
    pub fn new() -> Self {
        Self {
            messages: HashMap::new(),
            max_history: 1000,
        }
    }

    pub fn send_message(&mut self, sender: UserId, channel: String, content: String) -> RobinResult<String> {
        let message = ChatMessage {
            id: format!("msg_{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()),
            sender,
            recipient: None,
            channel: channel.clone(),
            content,
            message_type: MessageType::Text,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
            edited: false,
            reactions: HashMap::new(),
        };

        let channel_messages = self.messages.entry(channel).or_insert_with(VecDeque::new);
        if channel_messages.len() >= self.max_history {
            channel_messages.pop_front();
        }
        channel_messages.push_back(message.clone());

        Ok(message.id)
    }

    pub fn get_messages(&self, channel: &str, limit: Option<usize>) -> Vec<&ChatMessage> {
        if let Some(channel_messages) = self.messages.get(channel) {
            let limit = limit.unwrap_or(50);
            channel_messages.iter().rev().take(limit).collect()
        } else {
            Vec::new()
        }
    }
}

#[derive(Debug)]
pub struct VoiceManager {
    voice_channels: HashMap<String, Vec<UserId>>,
    muted_users: HashMap<UserId, bool>,
}

impl VoiceManager {
    pub fn new() -> Self {
        Self {
            voice_channels: HashMap::new(),
            muted_users: HashMap::new(),
        }
    }

    pub fn join_voice_channel(&mut self, user_id: UserId, channel: String) -> RobinResult<()> {
        self.voice_channels.entry(channel).or_insert_with(Vec::new).push(user_id);
        Ok(())
    }

    pub fn leave_voice_channel(&mut self, user_id: &UserId, channel: &str) -> RobinResult<()> {
        if let Some(users) = self.voice_channels.get_mut(channel) {
            users.retain(|id| id != user_id);
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct CommunicationManager {
    chat_manager: ChatManager,
    voice_manager: Option<VoiceManager>,
    enable_voice: bool,
}

impl CommunicationManager {
    pub fn new(enable_voice: bool) -> RobinResult<Self> {
        Ok(Self {
            chat_manager: ChatManager::new(),
            voice_manager: if enable_voice { Some(VoiceManager::new()) } else { None },
            enable_voice,
        })
    }

    pub fn send_chat_message(&mut self, sender: UserId, channel: String, content: String) -> RobinResult<String> {
        self.chat_manager.send_message(sender, channel, content)
    }

    pub fn add_user_to_voice_channel(&mut self, user_id: UserId) -> RobinResult<()> {
        if let Some(ref mut voice_manager) = self.voice_manager {
            voice_manager.join_voice_channel(user_id, "general".to_string())?;
        }
        Ok(())
    }

    pub fn remove_user_from_voice_channel(&mut self, user_id: &UserId) -> RobinResult<()> {
        if let Some(ref mut voice_manager) = self.voice_manager {
            voice_manager.leave_voice_channel(user_id, "general")?;
        }
        Ok(())
    }

    pub fn update(&mut self, _delta_time: f32) -> RobinResult<()> {
        Ok(())
    }
}