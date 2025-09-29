/*!
 * Game Client for Robin Engine
 *
 * Handles connection to a server and synchronization.
 */

use crate::engine::{
    error::{RobinError, RobinResult},
    networking::{
        protocol::*,
        NetworkEvent,
    },
    save_system::PlayerData,
};
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use std::net::SocketAddr;
use std::time::{SystemTime, Instant};
use std::collections::VecDeque;

/// Client configuration
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub player_name: String,
    pub password: Option<String>,
}

/// Game client
pub struct GameClient {
    stream: Option<TcpStream>,
    player_id: Option<u32>,
    player_name: String,
    server_address: SocketAddr,
    event_queue: VecDeque<NetworkEvent>,
    message_tx: Option<mpsc::Sender<NetworkMessage>>,
    sequence_id: u64,
    last_ping: Instant,
    ping_ms: u32,
    connected: bool,
}

impl GameClient {
    /// Connect to a server
    pub async fn connect(address: SocketAddr, player_name: String) -> RobinResult<Self> {
        let mut stream = TcpStream::connect(address).await
            .map_err(|e| RobinError::NetworkError {
                operation: "connect".to_string(),
                endpoint: address.to_string(),
                reason: format!("Failed to connect: {}", e),
            })?;

        // Send handshake
        let handshake = Handshake {
            version: PROTOCOL_VERSION,
            player_name: player_name.clone(),
            password: None,
        };

        let bytes = bincode::serialize(&handshake)?;
        stream.write_all(&bytes).await?;

        // Read response
        let mut buf = vec![0u8; 1024];
        let n = stream.read(&mut buf).await?;
        let response: HandshakeResponse = bincode::deserialize(&buf[..n])
            .map_err(|e| RobinError::NetworkError {
                operation: "deserialize_response".to_string(),
                endpoint: address.to_string(),
                reason: format!("Invalid handshake response: {}", e),
            })?;

        match response {
            HandshakeResponse::Accepted { player_id, .. } => {
                println!("✅ Connected to server (Player ID: {})", player_id);

                let (tx, mut rx) = mpsc::channel(100);

                let mut client = Self {
                    stream: Some(stream),
                    player_id: Some(player_id),
                    player_name,
                    server_address: address,
                    event_queue: VecDeque::new(),
                    message_tx: Some(tx.clone()),
                    sequence_id: 0,
                    last_ping: Instant::now(),
                    ping_ms: 0,
                    connected: true,
                };

                // Start read/write tasks
                client.start_network_tasks(tx, rx).await;

                Ok(client)
            }
            HandshakeResponse::Rejected { reason } => {
                Err(RobinError::NetworkError {
                    operation: "handshake".to_string(),
                    endpoint: address.to_string(),
                    reason: format!("Connection rejected: {}", reason),
                })
            }
            HandshakeResponse::RequiresPassword => {
                Err(RobinError::NetworkError {
                    operation: "handshake".to_string(),
                    endpoint: address.to_string(),
                    reason: "Server requires password".to_string(),
                })
            }
        }
    }

    /// Start network reading and writing tasks
    async fn start_network_tasks(
        &mut self,
        tx: mpsc::Sender<NetworkMessage>,
        mut rx: mpsc::Receiver<NetworkMessage>,
    ) {
        if let Some(stream) = self.stream.take() {
            let (reader, writer) = stream.into_split();

            // Start reader task
            let event_tx = tx.clone();
            tokio::spawn(async move {
                handle_server_messages(reader, event_tx).await;
            });

            // Start writer task
            tokio::spawn(async move {
                handle_server_writer(writer, rx).await;
            });

            // Start heartbeat task
            let heartbeat_tx = tx;
            tokio::spawn(async move {
                send_heartbeats(heartbeat_tx).await;
            });
        }
    }

    /// Send a message to the server
    pub async fn send(&mut self, message: NetworkMessage) -> RobinResult<()> {
        if !self.connected {
            return Err(RobinError::NetworkError {
                operation: "send_message".to_string(),
                endpoint: "server".to_string(),
                reason: "Not connected".to_string(),
            });
        }

        if let Some(ref tx) = self.message_tx {
            tx.send(message).await
                .map_err(|e| RobinError::NetworkError {
                    operation: "send_message".to_string(),
                    endpoint: "server".to_string(),
                    reason: format!("Failed to send message: {}", e),
                })?;
            self.sequence_id += 1;
        }

        Ok(())
    }

    /// Update client state
    pub async fn update(&mut self) -> RobinResult<Vec<NetworkEvent>> {
        let mut events = Vec::new();

        // Collect all pending events
        while let Some(event) = self.event_queue.pop_front() {
            events.push(event);
        }

        // Send periodic ping
        if self.last_ping.elapsed().as_secs() >= 5 {
            self.send(NetworkMessage::Ping {
                timestamp: SystemTime::now(),
            }).await?;
            self.last_ping = Instant::now();
        }

        Ok(events)
    }

    /// Disconnect from the server
    pub async fn disconnect(&mut self) -> RobinResult<()> {
        self.connected = false;

        if let Some(id) = self.player_id {
            self.send(NetworkMessage::PlayerLeave {
                player_id: id,
            }).await?;
        }

        println!("📡 Disconnected from server");
        Ok(())
    }

    /// Get current ping
    pub fn get_ping(&self) -> u32 {
        self.ping_ms
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Get player ID
    pub fn get_player_id(&self) -> Option<u32> {
        self.player_id
    }

    /// Send chat message
    pub async fn send_chat(&mut self, message: String) -> RobinResult<()> {
        if let Some(id) = self.player_id {
            self.send(NetworkMessage::ChatMessage {
                player_id: id,
                message,
                timestamp: SystemTime::now(),
            }).await?;
        }
        Ok(())
    }

    /// Send player movement
    pub async fn send_movement(&mut self, position: crate::engine::math::Vec3, rotation: crate::engine::math::Vec3) -> RobinResult<()> {
        if let Some(id) = self.player_id {
            self.send(NetworkMessage::PlayerMove {
                player_id: id,
                position,
                rotation,
            }).await?;
        }
        Ok(())
    }

    /// Place a voxel
    pub async fn place_voxel(&mut self, position: crate::engine::math::Vec3, voxel_type: crate::engine::world::VoxelType) -> RobinResult<()> {
        if let Some(id) = self.player_id {
            // Convert Vec3 (f32) to Vector3<i32> for network protocol
            let int_position = cgmath::Vector3::new(
                position.x as i32,
                position.y as i32,
                position.z as i32,
            );
            self.send(NetworkMessage::VoxelPlace {
                position: int_position,
                voxel_type,
                player_id: id,
            }).await?;
        }
        Ok(())
    }

    /// Remove a voxel
    pub async fn remove_voxel(&mut self, position: crate::engine::math::Vec3) -> RobinResult<()> {
        if let Some(id) = self.player_id {
            // Convert Vec3 (f32) to Vector3<i32> for network protocol
            let int_position = cgmath::Vector3::new(
                position.x as i32,
                position.y as i32,
                position.z as i32,
            );
            self.send(NetworkMessage::VoxelRemove {
                position: int_position,
                player_id: id,
            }).await?;
        }
        Ok(())
    }
}

/// Handle incoming messages from the server
async fn handle_server_messages(
    mut reader: tokio::net::tcp::OwnedReadHalf,
    event_tx: mpsc::Sender<NetworkMessage>,
) {
    let mut buf = vec![0u8; MAX_PACKET_SIZE];

    loop {
        match reader.read(&mut buf).await {
            Ok(0) => {
                // Connection closed
                println!("❌ Lost connection to server");
                break;
            }
            Ok(n) => {
                if let Ok(packet) = NetworkPacket::from_bytes(&buf[..n]) {
                    // Forward message to main client
                    let _ = event_tx.send(packet.message).await;
                }
            }
            Err(e) => {
                eprintln!("Read error: {}", e);
                break;
            }
        }
    }
}

/// Handle outgoing messages to the server
async fn handle_server_writer(
    mut writer: tokio::net::tcp::OwnedWriteHalf,
    mut rx: mpsc::Receiver<NetworkMessage>,
) {
    let mut sequence_id = 0u64;

    while let Some(message) = rx.recv().await {
        let packet = NetworkPacket::new(sequence_id, message);
        sequence_id += 1;

        if let Ok(bytes) = packet.to_bytes() {
            if let Err(e) = writer.write_all(&bytes).await {
                eprintln!("Write error: {}", e);
                break;
            }
        }
    }
}

/// Send periodic heartbeats
async fn send_heartbeats(tx: mpsc::Sender<NetworkMessage>) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(HEARTBEAT_INTERVAL));

    loop {
        interval.tick().await;

        let _ = tx.send(NetworkMessage::Heartbeat {
            timestamp: SystemTime::now(),
        }).await;
    }
}