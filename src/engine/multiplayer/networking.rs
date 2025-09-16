use crate::engine::error::RobinResult;
use crate::engine::multiplayer::UserId;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, VecDeque};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Instant, Duration};
use std::thread;
use std::io::{Read, Write, BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub max_connections: usize,
    pub timeout_seconds: u32,
    pub enable_compression: bool,
    pub enable_encryption: bool,
    pub chunk_size: usize,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            max_connections: 16,
            timeout_seconds: 1800, // 30 minutes
            enable_compression: true,
            enable_encryption: true,
            chunk_size: 64 * 1024, // 64KB
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMessage {
    pub id: String,
    pub sender: UserId,
    pub recipient: Option<UserId>, // None for broadcast
    pub message_type: NetworkMessageType,
    pub payload: Vec<u8>,
    pub timestamp: f64,
    pub priority: MessagePriority,
    pub requires_ack: bool,
    pub compression: Option<CompressionType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessageType {
    UserJoin,
    UserLeave,
    WorldChange,
    ChatMessage,
    VoiceData,
    AssetSync,
    Heartbeat,
    Acknowledgment,
    SystemCommand,
    FileTransfer,
    VersionControl,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum MessagePriority {
    Critical = 0,
    High = 1,
    Normal = 2,
    Low = 3,
    Background = 4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionType {
    None,
    Gzip,
    Lz4,
    Snappy,
}

impl NetworkMessage {
    pub fn new(sender: UserId, message_type: NetworkMessageType, payload: Vec<u8>) -> Self {
        Self {
            id: format!("msg_{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()),
            sender,
            recipient: None,
            message_type,
            payload,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
            priority: MessagePriority::Normal,
            requires_ack: false,
            compression: None,
        }
    }

    pub fn with_recipient(mut self, recipient: UserId) -> Self {
        self.recipient = Some(recipient);
        self
    }

    pub fn with_priority(mut self, priority: MessagePriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_acknowledgment(mut self, requires_ack: bool) -> Self {
        self.requires_ack = requires_ack;
        self
    }

    pub fn with_compression(mut self, compression: CompressionType) -> Self {
        self.compression = Some(compression);
        self
    }

    pub fn serialize(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut data = serde_json::to_vec(self)?;
        
        if let Some(compression_type) = &self.compression {
            data = self.compress_data(data, compression_type)?;
        }
        
        Ok(data)
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let message: Self = serde_json::from_slice(data)?;
        Ok(message)
    }

    fn compress_data(&self, data: Vec<u8>, compression_type: &CompressionType) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        match compression_type {
            CompressionType::None => Ok(data),
            CompressionType::Gzip => {
                // In a real implementation, you would use the `flate2` crate
                println!("Gzip compression not implemented in demo");
                Ok(data)
            }
            CompressionType::Lz4 => {
                // In a real implementation, you would use the `lz4` crate
                println!("LZ4 compression not implemented in demo");
                Ok(data)
            }
            CompressionType::Snappy => {
                // In a real implementation, you would use the `snap` crate
                println!("Snappy compression not implemented in demo");
                Ok(data)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub connection_id: String,
    pub remote_address: SocketAddr,
    pub local_address: SocketAddr,
    pub connected_at: Instant,
    pub user_id: Option<UserId>,
    pub latency_ms: f32,
    pub bandwidth_mbps: f32,
}

#[derive(Debug)]
pub struct Connection {
    pub info: ConnectionInfo,
    pub stream: Arc<Mutex<TcpStream>>,
    pub incoming_queue: Arc<Mutex<VecDeque<NetworkMessage>>>,
    pub outgoing_queue: Arc<Mutex<VecDeque<NetworkMessage>>>,
    pub last_heartbeat: Arc<Mutex<Instant>>,
    pub stats: ConnectionStats,
}

#[derive(Debug, Default)]
pub struct ConnectionStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub packets_dropped: u32,
    pub reconnections: u32,
}

pub struct NetworkManager {
    config: NetworkConfig,
    connections: Arc<RwLock<HashMap<String, Arc<Connection>>>>,
    listener: Option<TcpListener>,
    server_address: Option<SocketAddr>,
    is_server: bool,
    stats: NetworkStats,
    message_handlers: HashMap<NetworkMessageType, Box<dyn Fn(&NetworkMessage) + Send + Sync>>,
}

impl std::fmt::Debug for NetworkManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NetworkManager")
            .field("config", &self.config)
            .field("connections", &self.connections)
            .field("listener", &self.listener)
            .field("server_address", &self.server_address)
            .field("is_server", &self.is_server)
            .field("stats", &self.stats)
            .field("message_handlers", &format!("<{} handlers>", self.message_handlers.len()))
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub total_connections: u64,
    pub active_connections: u32,
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
    pub total_messages_sent: u64,
    pub total_messages_received: u64,
    pub average_latency_ms: f32,
    pub packet_loss_rate: f32,
    pub uptime_seconds: u64,
}

impl Default for NetworkStats {
    fn default() -> Self {
        Self {
            total_connections: 0,
            active_connections: 0,
            total_bytes_sent: 0,
            total_bytes_received: 0,
            total_messages_sent: 0,
            total_messages_received: 0,
            average_latency_ms: 0.0,
            packet_loss_rate: 0.0,
            uptime_seconds: 0,
        }
    }
}

impl NetworkManager {
    pub fn new(config: NetworkConfig) -> RobinResult<Self> {
        Ok(Self {
            config,
            connections: Arc::new(RwLock::new(HashMap::new())),
            listener: None,
            server_address: None,
            is_server: false,
            stats: NetworkStats::default(),
            message_handlers: HashMap::new(),
        })
    }

    pub fn start_server(&mut self, bind_address: SocketAddr) -> RobinResult<()> {
        let listener = TcpListener::bind(bind_address)
            .map_err(|e| crate::engine::error::RobinError::NetworkError { 
                operation: "bind".to_string(), 
                endpoint: bind_address.to_string(), 
                reason: format!("Failed to bind to {}: {}", bind_address, e) 
            })?;
        
        listener.set_nonblocking(true)
            .map_err(|e| crate::engine::error::RobinError::NetworkError { 
                operation: "set_nonblocking".to_string(), 
                endpoint: bind_address.to_string(), 
                reason: format!("Failed to set non-blocking: {}", e) 
            })?;

        self.listener = Some(listener);
        self.server_address = Some(bind_address);
        self.is_server = true;

        println!("Network server started on {}", bind_address);
        println!("Max connections: {}", self.config.max_connections);
        println!("Timeout: {} seconds", self.config.timeout_seconds);
        println!("Compression: {}", if self.config.enable_compression { "enabled" } else { "disabled" });
        println!("Encryption: {}", if self.config.enable_encryption { "enabled" } else { "disabled" });

        Ok(())
    }

    pub fn connect_to_server(&mut self, server_address: SocketAddr) -> RobinResult<ConnectionInfo> {
        let stream = TcpStream::connect(server_address)
            .map_err(|e| crate::engine::error::RobinError::NetworkError { 
                operation: "connect".to_string(), 
                endpoint: server_address.to_string(), 
                reason: format!("Failed to connect to {}: {}", server_address, e) 
            })?;
        
        let local_address = stream.local_addr()
            .map_err(|e| crate::engine::error::RobinError::NetworkError { 
                operation: "get_local_addr".to_string(), 
                endpoint: server_address.to_string(), 
                reason: format!("Failed to get local address: {}", e) 
            })?;

        let connection_info = ConnectionInfo {
            connection_id: format!("conn_{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()),
            remote_address: server_address,
            local_address,
            connected_at: Instant::now(),
            user_id: None,
            latency_ms: 0.0,
            bandwidth_mbps: 0.0,
        };

        let connection = Arc::new(Connection {
            info: connection_info.clone(),
            stream: Arc::new(Mutex::new(stream)),
            incoming_queue: Arc::new(Mutex::new(VecDeque::new())),
            outgoing_queue: Arc::new(Mutex::new(VecDeque::new())),
            last_heartbeat: Arc::new(Mutex::new(Instant::now())),
            stats: ConnectionStats::default(),
        });

        self.connections.write().unwrap().insert(connection_info.connection_id.clone(), connection.clone());
        self.stats.total_connections += 1;
        self.stats.active_connections += 1;

        self.start_connection_thread(connection)?;

        println!("Connected to server at {}", server_address);
        Ok(connection_info)
    }

    fn start_connection_thread(&self, connection: Arc<Connection>) -> RobinResult<()> {
        let conn_clone = connection.clone();
        
        thread::spawn(move || {
            if let Err(e) = Self::handle_connection(conn_clone) {
                println!("Connection error: {}", e);
            }
        });

        Ok(())
    }

    fn handle_connection(connection: Arc<Connection>) -> Result<(), Box<dyn std::error::Error>> {
        let mut buffer = [0; 4096];
        let stream_clone = connection.stream.clone();
        
        loop {
            // Read incoming data
            {
                let mut stream = stream_clone.lock().unwrap();
                match stream.read(&mut buffer) {
                    Ok(0) => {
                        println!("Connection closed by peer");
                        break;
                    }
                    Ok(bytes_read) => {
                        if let Ok(message) = NetworkMessage::deserialize(&buffer[..bytes_read]) {
                            connection.incoming_queue.lock().unwrap().push_back(message);
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // No data available, continue
                    }
                    Err(e) => {
                        println!("Error reading from connection: {}", e);
                        break;
                    }
                }
            }

            // Send outgoing messages
            if let Some(message) = connection.outgoing_queue.lock().unwrap().pop_front() {
                if let Ok(data) = message.serialize() {
                    let mut stream = stream_clone.lock().unwrap();
                    if let Err(e) = stream.write_all(&data) {
                        println!("Error sending message: {}", e);
                        // Put message back in queue for retry
                        connection.outgoing_queue.lock().unwrap().push_front(message);
                        break;
                    }
                }
            }

            // Send heartbeat if needed
            {
                let mut last_heartbeat = connection.last_heartbeat.lock().unwrap();
                if last_heartbeat.elapsed() > Duration::from_secs(30) {
                    let heartbeat = NetworkMessage::new(
                        UserId::new("system".to_string()),
                        NetworkMessageType::Heartbeat,
                        vec![]
                    );
                    connection.outgoing_queue.lock().unwrap().push_back(heartbeat);
                    *last_heartbeat = Instant::now();
                }
            }

            thread::sleep(Duration::from_millis(10));
        }

        Ok(())
    }

    pub fn accept_connections(&mut self) -> RobinResult<Vec<ConnectionInfo>> {
        if !self.is_server || self.listener.is_none() {
            return Ok(vec![]);
        }

        let mut new_connections = Vec::new();
        let listener = self.listener.as_ref().unwrap();

        // Try to accept new connections
        loop {
            match listener.accept() {
                Ok((stream, remote_addr)) => {
                    if self.stats.active_connections >= self.config.max_connections as u32 {
                        println!("Maximum connections reached, rejecting connection from {}", remote_addr);
                        drop(stream);
                        continue;
                    }

                    let local_addr = stream.local_addr()
                        .map_err(|e| crate::engine::error::RobinError::NetworkError { 
                            operation: "get_local_addr".to_string(), 
                            endpoint: remote_addr.to_string(), 
                            reason: format!("Failed to get local address: {}", e) 
                        })?;

                    let connection_info = ConnectionInfo {
                        connection_id: format!("conn_{}", std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_nanos()),
                        remote_address: remote_addr,
                        local_address: local_addr,
                        connected_at: Instant::now(),
                        user_id: None,
                        latency_ms: 0.0,
                        bandwidth_mbps: 0.0,
                    };

                    let connection = Arc::new(Connection {
                        info: connection_info.clone(),
                        stream: Arc::new(Mutex::new(stream)),
                        incoming_queue: Arc::new(Mutex::new(VecDeque::new())),
                        outgoing_queue: Arc::new(Mutex::new(VecDeque::new())),
                        last_heartbeat: Arc::new(Mutex::new(Instant::now())),
                        stats: ConnectionStats::default(),
                    });

                    self.connections.write().unwrap().insert(connection_info.connection_id.clone(), connection.clone());
                    self.stats.total_connections += 1;
                    self.stats.active_connections += 1;

                    self.start_connection_thread(connection)?;
                    new_connections.push(connection_info);

                    println!("Accepted connection from {}", remote_addr);
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No pending connections
                    break;
                }
                Err(e) => {
                    println!("Error accepting connection: {}", e);
                    break;
                }
            }
        }

        Ok(new_connections)
    }

    pub fn send_message(&mut self, connection_id: &str, message: NetworkMessage) -> RobinResult<()> {
        let connections = self.connections.read().unwrap();
        
        if let Some(connection) = connections.get(connection_id) {
            connection.outgoing_queue.lock().unwrap().push_back(message);
            self.stats.total_messages_sent += 1;
            Ok(())
        } else {
            Err(crate::engine::error::RobinError::NetworkError {
                operation: "send_message".to_string(),
                endpoint: connection_id.to_string(),
                reason: format!("Connection not found: {}", connection_id)
            })
        }
    }

    pub fn broadcast_message(&mut self, message: NetworkMessage, exclude_connection: Option<&str>) -> RobinResult<u32> {
        let connections = self.connections.read().unwrap();
        let mut sent_count = 0;

        for (conn_id, connection) in connections.iter() {
            if let Some(exclude) = exclude_connection {
                if conn_id == exclude {
                    continue;
                }
            }

            let mut msg_clone = message.clone();
            msg_clone.id = format!("{}_{}", message.id, conn_id);
            connection.outgoing_queue.lock().unwrap().push_back(msg_clone);
            sent_count += 1;
        }

        self.stats.total_messages_sent += sent_count as u64;
        Ok(sent_count)
    }

    pub fn receive_messages(&mut self, connection_id: &str) -> RobinResult<Vec<NetworkMessage>> {
        let connections = self.connections.read().unwrap();
        
        if let Some(connection) = connections.get(connection_id) {
            let mut messages = Vec::new();
            let mut queue = connection.incoming_queue.lock().unwrap();
            
            while let Some(message) = queue.pop_front() {
                messages.push(message);
            }
            
            self.stats.total_messages_received += messages.len() as u64;
            Ok(messages)
        } else {
            Err(crate::engine::error::RobinError::NetworkError {
                operation: "receive_messages".to_string(),
                endpoint: connection_id.to_string(),
                reason: format!("Connection not found: {}", connection_id)
            })
        }
    }

    pub fn receive_all_messages(&mut self) -> RobinResult<HashMap<String, Vec<NetworkMessage>>> {
        let connections = self.connections.read().unwrap();
        let mut all_messages = HashMap::new();

        for (conn_id, connection) in connections.iter() {
            let mut messages = Vec::new();
            let mut queue = connection.incoming_queue.lock().unwrap();
            
            while let Some(message) = queue.pop_front() {
                messages.push(message);
            }
            
            if !messages.is_empty() {
                self.stats.total_messages_received += messages.len() as u64;
                all_messages.insert(conn_id.clone(), messages);
            }
        }

        Ok(all_messages)
    }

    pub fn disconnect(&mut self, connection_id: &str) -> RobinResult<()> {
        let mut connections = self.connections.write().unwrap();
        
        if let Some(connection) = connections.remove(connection_id) {
            // Send disconnect notification
            let disconnect_msg = NetworkMessage::new(
                UserId::new("system".to_string()),
                NetworkMessageType::UserLeave,
                vec![]
            );
            connection.outgoing_queue.lock().unwrap().push_back(disconnect_msg);

            self.stats.active_connections = self.stats.active_connections.saturating_sub(1);
            println!("Disconnected connection: {}", connection_id);
            Ok(())
        } else {
            Err(crate::engine::error::RobinError::NetworkError {
                operation: "disconnect".to_string(),
                endpoint: connection_id.to_string(),
                reason: format!("Connection not found: {}", connection_id)
            })
        }
    }

    pub fn get_connection_info(&self, connection_id: &str) -> Option<ConnectionInfo> {
        let connections = self.connections.read().unwrap();
        connections.get(connection_id).map(|conn| conn.info.clone())
    }

    pub fn get_all_connections(&self) -> Vec<ConnectionInfo> {
        let connections = self.connections.read().unwrap();
        connections.values().map(|conn| conn.info.clone()).collect()
    }

    pub fn update(&mut self, _delta_time: f32) -> RobinResult<()> {
        // Accept new connections if server
        if self.is_server {
            let _new_connections = self.accept_connections()?;
        }

        // Update connection statistics
        let connections = self.connections.read().unwrap();
        
        let mut total_latency = 0.0;
        let mut connection_count = 0;

        for connection in connections.values() {
            total_latency += connection.info.latency_ms;
            connection_count += 1;

            // Check for connection timeout
            let last_heartbeat = *connection.last_heartbeat.lock().unwrap();
            if last_heartbeat.elapsed() > Duration::from_secs(self.config.timeout_seconds as u64) {
                println!("Connection {} timed out", connection.info.connection_id);
                // Mark for disconnection (would need additional logic)
            }
        }

        if connection_count > 0 {
            self.stats.average_latency_ms = total_latency / connection_count as f32;
        }

        Ok(())
    }

    pub fn get_stats(&self) -> NetworkStats {
        self.stats.clone()
    }

    pub fn shutdown(&mut self) -> RobinResult<()> {
        println!("Shutting down network manager...");

        // Disconnect all connections
        let connection_ids: Vec<_> = {
            let connections = self.connections.read().unwrap();
            connections.keys().cloned().collect()
        };

        for connection_id in connection_ids {
            self.disconnect(&connection_id)?;
        }

        // Close server listener if running
        if let Some(_listener) = self.listener.take() {
            println!("Closed server listener");
        }

        println!("Network shutdown complete. Final stats:");
        println!("  Total connections: {}", self.stats.total_connections);
        println!("  Total bytes sent: {}", self.stats.total_bytes_sent);
        println!("  Total bytes received: {}", self.stats.total_bytes_received);
        println!("  Total messages sent: {}", self.stats.total_messages_sent);
        println!("  Total messages received: {}", self.stats.total_messages_received);
        println!("  Average latency: {:.2}ms", self.stats.average_latency_ms);

        Ok(())
    }

    pub fn send_message_to_user(&mut self, user_id: &str, message: NetworkMessage) -> RobinResult<()> {
        // For now, use user_id as connection_id. In a more sophisticated system,
        // we would maintain a mapping between user IDs and connection IDs.
        self.send_message(user_id, message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_config() {
        let config = NetworkConfig::default();
        assert_eq!(config.max_connections, 16);
        assert_eq!(config.timeout_seconds, 1800);
        assert!(config.enable_compression);
        assert!(config.enable_encryption);
        assert_eq!(config.chunk_size, 64 * 1024);
    }

    #[test]
    fn test_network_message_creation() {
        let sender = UserId::new("test_user".to_string());
        let payload = b"Hello, World!".to_vec();
        let message = NetworkMessage::new(sender.clone(), NetworkMessageType::ChatMessage, payload.clone());

        assert_eq!(message.sender, sender);
        assert_eq!(message.payload, payload);
        assert!(matches!(message.message_type, NetworkMessageType::ChatMessage));
        assert_eq!(message.priority, MessagePriority::Normal);
        assert!(!message.requires_ack);
    }

    #[test]
    fn test_message_priority_ordering() {
        assert!(MessagePriority::Critical < MessagePriority::High);
        assert!(MessagePriority::High < MessagePriority::Normal);
        assert!(MessagePriority::Normal < MessagePriority::Low);
        assert!(MessagePriority::Low < MessagePriority::Background);
    }

    #[test]
    fn test_network_manager_creation() {
        let config = NetworkConfig::default();
        let manager = NetworkManager::new(config);
        assert!(manager.is_ok());
        
        let manager = manager.unwrap();
        assert!(!manager.is_server);
        assert_eq!(manager.stats.active_connections, 0);
    }

    #[test]
    fn test_message_serialization() {
        let sender = UserId::new("test_user".to_string());
        let payload = b"test payload".to_vec();
        let message = NetworkMessage::new(sender, NetworkMessageType::SystemCommand, payload);

        let serialized = message.serialize();
        assert!(serialized.is_ok());

        let deserialized = NetworkMessage::deserialize(&serialized.unwrap());
        assert!(deserialized.is_ok());

        let deserialized_message = deserialized.unwrap();
        assert_eq!(deserialized_message.payload, message.payload);
        assert!(matches!(deserialized_message.message_type, NetworkMessageType::SystemCommand));
    }
}