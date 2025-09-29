/*!
 * Game Server for Robin Engine
 *
 * Handles multiple client connections and world synchronization.
 */

use crate::engine::{
    error::{RobinError, RobinResult},
    networking::{
        protocol::*,
        NetworkEvent, NetworkStats,
    },
    // TODO: Re-enable when VoxelWorld is available
    // world::VoxelWorld,
    save_system::PlayerData,
};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{mpsc, RwLock};
use std::collections::HashMap;
use std::sync::Arc;
use std::net::SocketAddr;
use std::time::{SystemTime, Duration};

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub max_players: usize,
    pub world_seed: u64,
    pub server_name: String,
    pub password: Option<String>,
    pub tick_rate: u32,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 25565,
            max_players: 16,
            world_seed: rand::random(),
            server_name: "Robin Server".to_string(),
            password: None,
            tick_rate: 20, // 20 ticks per second
        }
    }
}

/// Connected player information
#[derive(Debug, Clone)]
struct ConnectedPlayer {
    id: u32,
    name: String,
    data: PlayerData,
    address: SocketAddr,
    last_heartbeat: SystemTime,
    tx: mpsc::Sender<NetworkMessage>,
}

/// Game server
pub struct GameServer {
    config: ServerConfig,
    players: Arc<RwLock<HashMap<u32, ConnectedPlayer>>>,
    // TODO: Re-enable when VoxelWorld is available
    // world: Arc<RwLock<VoxelWorld>>,
    event_tx: mpsc::Sender<NetworkEvent>,
    event_rx: mpsc::Receiver<NetworkEvent>,
    stats: NetworkStats,
    running: Arc<RwLock<bool>>,
    next_player_id: Arc<RwLock<u32>>,
}

impl GameServer {
    /// Create a new game server
    pub async fn new(config: ServerConfig) -> RobinResult<Self> {
        let (event_tx, event_rx) = mpsc::channel(1000);

        Ok(Self {
            config,
            players: Arc::new(RwLock::new(HashMap::new())),
            // TODO: Re-enable when VoxelWorld is available
            // world: Arc::new(RwLock::new(VoxelWorld::new_with_seed(config.world_seed))),
            event_tx,
            event_rx,
            stats: NetworkStats::default(),
            running: Arc::new(RwLock::new(false)),
            next_player_id: Arc::new(RwLock::new(1)),
        })
    }

    /// Start the server
    pub async fn start(&mut self) -> RobinResult<()> {
        let addr = SocketAddr::from(([0, 0, 0, 0], self.config.port));
        let listener = TcpListener::bind(addr).await
            .map_err(|e| RobinError::NetworkError {
                operation: "bind".to_string(),
                endpoint: addr.to_string(),
                reason: format!("Failed to bind port: {}", e),
            })?;

        *self.running.write().await = true;

        println!("🎮 Server listening on {}", addr);
        println!("   Max players: {}", self.config.max_players);
        println!("   World seed: {}", self.config.world_seed);

        // Spawn accept loop
        let running = self.running.clone();
        let players = self.players.clone();
        // TODO: Re-enable when VoxelWorld is available
        // let world = self.world.clone();
        let event_tx = self.event_tx.clone();
        let config = self.config.clone();
        let next_id = self.next_player_id.clone();

        tokio::spawn(async move {
            while *running.read().await {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        let players_clone = players.clone();
                        // TODO: Re-enable when VoxelWorld is available
                        // let world_clone = world.clone();
                        let event_tx_clone = event_tx.clone();
                        let config_clone = config.clone();
                        let next_id_clone = next_id.clone();

                        tokio::spawn(async move {
                            if let Err(e) = handle_client(
                                stream,
                                addr,
                                players_clone,
                                // TODO: Re-enable when VoxelWorld is available
                                // world_clone,
                                event_tx_clone,
                                config_clone,
                                next_id_clone,
                            ).await {
                                eprintln!("Client error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        eprintln!("Failed to accept connection: {}", e);
                    }
                }
            }
        });

        // Start tick loop
        self.start_tick_loop().await;

        Ok(())
    }

    /// Start the server tick loop
    async fn start_tick_loop(&self) {
        let tick_interval = Duration::from_millis(1000 / self.config.tick_rate as u64);
        let players = self.players.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tick_interval);

            while *running.read().await {
                interval.tick().await;

                // Check for disconnected players
                let mut disconnected = Vec::new();
                let now = SystemTime::now();

                {
                    let players_lock = players.read().await;
                    for (id, player) in players_lock.iter() {
                        if let Ok(elapsed) = now.duration_since(player.last_heartbeat) {
                            if elapsed > Duration::from_secs(CONNECTION_TIMEOUT) {
                                disconnected.push(*id);
                            }
                        }
                    }
                }

                // Remove disconnected players
                for id in disconnected {
                    let mut players_lock = players.write().await;
                    if let Some(player) = players_lock.remove(&id) {
                        println!("⏰ Player {} timed out", player.name);
                    }
                }
            }
        });
    }

    /// Broadcast a message to all players
    pub async fn broadcast(&mut self, message: NetworkMessage) -> RobinResult<()> {
        let players = self.players.read().await;

        for player in players.values() {
            if let Err(e) = player.tx.send(message.clone()).await {
                eprintln!("Failed to send to player {}: {}", player.name, e);
            }
        }

        self.stats.packets_sent += players.len() as u64;
        Ok(())
    }

    /// Broadcast to all players except one
    pub async fn broadcast_except(&mut self, exclude_id: u32, message: NetworkMessage) -> RobinResult<()> {
        let players = self.players.read().await;

        for (id, player) in players.iter() {
            if *id != exclude_id {
                if let Err(e) = player.tx.send(message.clone()).await {
                    eprintln!("Failed to send to player {}: {}", player.name, e);
                }
            }
        }

        Ok(())
    }

    /// Update server state
    pub async fn update(&mut self) -> RobinResult<Vec<NetworkEvent>> {
        let mut events = Vec::new();

        // Collect events from channel
        while let Ok(event) = self.event_rx.try_recv() {
            events.push(event);
        }

        // Update stats
        self.stats.connected_players = self.players.read().await.len();

        Ok(events)
    }

    /// Shutdown the server
    pub async fn shutdown(&mut self) -> RobinResult<()> {
        *self.running.write().await = false;

        // Notify all players
        self.broadcast(NetworkMessage::ServerShutdown {
            reason: "Server shutting down".to_string(),
        }).await?;

        // Clear players
        self.players.write().await.clear();

        println!("🛑 Server shutdown");
        Ok(())
    }

    /// Get server statistics
    pub fn get_stats(&self) -> NetworkStats {
        self.stats.clone()
    }
}

/// Handle a client connection
async fn handle_client(
    mut stream: TcpStream,
    addr: SocketAddr,
    players: Arc<RwLock<HashMap<u32, ConnectedPlayer>>>,
    // TODO: Re-enable when VoxelWorld is available
    // world: Arc<RwLock<VoxelWorld>>,
    event_tx: mpsc::Sender<NetworkEvent>,
    config: ServerConfig,
    next_id: Arc<RwLock<u32>>,
) -> RobinResult<()> {
    println!("📥 New connection from {}", addr);

    // Read handshake
    let mut buf = vec![0u8; 1024];
    let n = stream.read(&mut buf).await?;
    let handshake: Handshake = bincode::deserialize(&buf[..n])
        .map_err(|e| RobinError::NetworkError {
            operation: "deserialize_handshake".to_string(),
            endpoint: "client".to_string(),
            reason: format!("Invalid handshake: {}", e),
        })?;

    // Check version
    if handshake.version != PROTOCOL_VERSION {
        let response = HandshakeResponse::Rejected {
            reason: format!("Version mismatch. Server: {}, Client: {}", PROTOCOL_VERSION, handshake.version),
        };
        let bytes = bincode::serialize(&response)?;
        stream.write_all(&bytes).await?;
        return Ok(());
    }

    // Check password
    if let Some(ref server_password) = config.password {
        if handshake.password.as_ref() != Some(server_password) {
            let response = HandshakeResponse::RequiresPassword;
            let bytes = bincode::serialize(&response)?;
            stream.write_all(&bytes).await?;
            return Ok(());
        }
    }

    // Check max players
    if players.read().await.len() >= config.max_players {
        let response = HandshakeResponse::Rejected {
            reason: "Server is full".to_string(),
        };
        let bytes = bincode::serialize(&response)?;
        stream.write_all(&bytes).await?;
        return Ok(());
    }

    // Accept connection
    let player_id = {
        let mut id = next_id.write().await;
        let current = *id;
        *id += 1;
        current
    };

    let response = HandshakeResponse::Accepted {
        player_id,
        world_seed: config.world_seed,
    };
    let bytes = bincode::serialize(&response)?;
    stream.write_all(&bytes).await?;

    // Create player
    let (tx, mut rx) = mpsc::channel(100);
    let player = ConnectedPlayer {
        id: player_id,
        name: handshake.player_name.clone(),
        data: PlayerData::new(handshake.player_name.clone()),
        address: addr,
        last_heartbeat: SystemTime::now(),
        tx,
    };

    // Add to players
    players.write().await.insert(player_id, player.clone());

    // Notify other players
    let join_msg = NetworkMessage::PlayerJoin {
        player_id,
        player_data: player.data.clone(),
    };

    for (id, other) in players.read().await.iter() {
        if *id != player_id {
            let _ = other.tx.send(join_msg.clone()).await;
        }
    }

    // Send world sync to new player
    // TODO: Send actual world data
    let sync_msg = NetworkMessage::WorldSync {
        chunks: Vec::new(),
        players: players.read().await.values()
            .map(|p| p.data.clone())
            .collect(),
    };
    let _ = player.tx.send(sync_msg).await;

    println!("✅ Player {} joined (ID: {})", handshake.player_name, player_id);

    // Emit event
    let _ = event_tx.send(NetworkEvent::PlayerJoined {
        player_id,
        name: handshake.player_name,
    }).await;

    // Handle player messages
    let (reader, writer) = stream.into_split();

    // Spawn reader task
    let players_clone = players.clone();
    let event_tx_clone = event_tx.clone();

    tokio::spawn(async move {
        handle_player_messages(
            reader,
            player_id,
            players_clone,
            event_tx_clone,
        ).await;
    });

    // Spawn writer task
    tokio::spawn(async move {
        handle_player_writer(writer, rx).await;
    });

    Ok(())
}

/// Handle incoming messages from a player
async fn handle_player_messages(
    mut reader: tokio::net::tcp::OwnedReadHalf,
    player_id: u32,
    players: Arc<RwLock<HashMap<u32, ConnectedPlayer>>>,
    event_tx: mpsc::Sender<NetworkEvent>,
) {
    let mut buf = vec![0u8; MAX_PACKET_SIZE];

    loop {
        match reader.read(&mut buf).await {
            Ok(0) => {
                // Connection closed
                break;
            }
            Ok(n) => {
                if let Ok(packet) = NetworkPacket::from_bytes(&buf[..n]) {
                    // Update heartbeat
                    if let Some(player) = players.write().await.get_mut(&player_id) {
                        player.last_heartbeat = SystemTime::now();
                    }

                    // Process message
                    match packet.message {
                        NetworkMessage::Heartbeat { .. } => {
                            // Already updated heartbeat above
                        }
                        NetworkMessage::ChatMessage { message, .. } => {
                            // Broadcast chat to all players
                            let chat_msg = NetworkMessage::ChatMessage {
                                player_id,
                                message: message.clone(),
                                timestamp: SystemTime::now(),
                            };

                            for player in players.read().await.values() {
                                let _ = player.tx.send(chat_msg.clone()).await;
                            }

                            let _ = event_tx.send(NetworkEvent::ChatReceived {
                                player_id,
                                message,
                            }).await;
                        }
                        _ => {
                            // Handle other messages
                        }
                    }
                }
            }
            Err(_) => {
                // Connection error
                break;
            }
        }
    }

    // Remove player on disconnect
    if let Some(player) = players.write().await.remove(&player_id) {
        println!("👋 Player {} disconnected", player.name);

        // Notify other players
        let leave_msg = NetworkMessage::PlayerLeave { player_id };
        for other in players.read().await.values() {
            let _ = other.tx.send(leave_msg.clone()).await;
        }

        let _ = event_tx.send(NetworkEvent::PlayerLeft {
            player_id,
            name: player.name,
        }).await;
    }
}

/// Handle outgoing messages to a player
async fn handle_player_writer(
    mut writer: tokio::net::tcp::OwnedWriteHalf,
    mut rx: mpsc::Receiver<NetworkMessage>,
) {
    let mut sequence_id = 0u64;

    while let Some(message) = rx.recv().await {
        let packet = NetworkPacket::new(sequence_id, message);
        sequence_id += 1;

        if let Ok(bytes) = packet.to_bytes() {
            if let Err(_) = writer.write_all(&bytes).await {
                break;
            }
        }
    }
}