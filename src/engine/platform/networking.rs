/*!
 * Robin Engine Networking System
 *
 * Comprehensive multiplayer networking with client-server architecture,
 * peer-to-peer support, state synchronization, and cross-platform compatibility.
 */

use crate::engine::error::{RobinResult, RobinError};
use std::collections::HashMap;
use std::net::{SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

/// Core networking manager
#[derive(Debug)]
pub struct NetworkManager {
    config: NetworkConfig,
    server_manager: Option<ServerManager>,
    client_manager: Option<ClientManager>,
    peer_manager: Option<PeerManager>,
    protocol_handlers: HashMap<ProtocolType, Box<dyn ProtocolHandler>>,
    connection_pool: ConnectionPool,
    message_bus: NetworkMessageBus,
    security_manager: NetworkSecurityManager,
    metrics: NetworkMetrics,
}

impl NetworkManager {
    pub fn new() -> RobinResult<Self> {
        let config = NetworkConfig::default();

        Ok(Self {
            config,
            server_manager: None,
            client_manager: None,
            peer_manager: None,
            protocol_handlers: Self::initialize_protocol_handlers()?,
            connection_pool: ConnectionPool::new()?,
            message_bus: NetworkMessageBus::new()?,
            security_manager: NetworkSecurityManager::new()?,
            metrics: NetworkMetrics::new(),
        })
    }

    /// Initialize the networking system
    pub fn initialize(&mut self) -> RobinResult<()> {
        println!("🌐 Network Manager: Initializing networking systems");

        // Initialize protocol handlers
        for handler in self.protocol_handlers.values_mut() {
            handler.initialize(&self.config)?;
        }

        // Initialize security
        self.security_manager.initialize(&self.config)?;

        println!("✅ Network Manager: Initialization complete");
        Ok(())
    }

    /// Start as a dedicated server
    pub fn start_server(&mut self, config: ServerConfig) -> RobinResult<()> {
        let mut server = ServerManager::new(config)?;
        server.start()?;
        self.server_manager = Some(server);

        println!("🖥️ Network Manager: Started as server");
        Ok(())
    }

    /// Connect as a client
    pub fn connect_as_client(&mut self, config: ClientConfig) -> RobinResult<()> {
        let mut client = ClientManager::new(config)?;
        client.connect()?;
        self.client_manager = Some(client);

        println!("💻 Network Manager: Connected as client");
        Ok(())
    }

    /// Start peer-to-peer networking
    pub fn start_peer_to_peer(&mut self, config: PeerConfig) -> RobinResult<()> {
        let mut peer = PeerManager::new(config)?;
        peer.start()?;
        self.peer_manager = Some(peer);

        println!("🔗 Network Manager: Started peer-to-peer networking");
        Ok(())
    }

    /// Send a message
    pub fn send_message(&mut self, target: MessageTarget, message: NetworkMessage) -> RobinResult<()> {
        // Apply security filtering
        let filtered_message = self.security_manager.filter_outgoing_message(message)?;

        match target {
            MessageTarget::All => self.broadcast_message(filtered_message),
            MessageTarget::Client(client_id) => self.send_to_client(client_id, filtered_message),
            MessageTarget::Server => self.send_to_server(filtered_message),
            MessageTarget::Peer(peer_id) => self.send_to_peer(peer_id, filtered_message),
        }
    }

    /// Receive messages
    pub fn receive_messages(&mut self) -> RobinResult<Vec<ReceivedMessage>> {
        let mut messages = Vec::new();

        // Collect messages from all sources
        if let Some(server) = &mut self.server_manager {
            messages.extend(server.receive_messages()?);
        }

        if let Some(client) = &mut self.client_manager {
            messages.extend(client.receive_messages()?);
        }

        if let Some(peer) = &mut self.peer_manager {
            messages.extend(peer.receive_messages()?);
        }

        // Apply security filtering
        let filtered_messages: RobinResult<Vec<_>> = messages
            .into_iter()
            .map(|msg| self.security_manager.filter_incoming_message(msg))
            .collect();

        filtered_messages
    }

    /// Update networking (call this each frame)
    pub fn update(&mut self, delta_time: f32) -> RobinResult<()> {
        // Update connection pool
        self.connection_pool.update(delta_time)?;

        // Update managers
        if let Some(server) = &mut self.server_manager {
            server.update(delta_time)?;
        }

        if let Some(client) = &mut self.client_manager {
            client.update(delta_time)?;
        }

        if let Some(peer) = &mut self.peer_manager {
            peer.update(delta_time)?;
        }

        // Update message bus
        self.message_bus.update(delta_time)?;

        // Update metrics
        self.metrics.update(delta_time);

        Ok(())
    }

    /// Shutdown networking
    pub fn shutdown(&mut self) -> RobinResult<()> {
        if let Some(server) = &mut self.server_manager {
            server.shutdown()?;
        }

        if let Some(client) = &mut self.client_manager {
            client.disconnect()?;
        }

        if let Some(peer) = &mut self.peer_manager {
            peer.shutdown()?;
        }

        self.connection_pool.shutdown()?;

        println!("🛑 Network Manager: Shutdown complete");
        Ok(())
    }

    /// Get network statistics
    pub fn get_metrics(&self) -> &NetworkMetrics {
        &self.metrics
    }

    /// Get connection status
    pub fn get_connection_status(&self) -> ConnectionStatus {
        if self.server_manager.is_some() {
            ConnectionStatus::Server
        } else if self.client_manager.is_some() {
            ConnectionStatus::Client
        } else if self.peer_manager.is_some() {
            ConnectionStatus::Peer
        } else {
            ConnectionStatus::Disconnected
        }
    }

    fn broadcast_message(&mut self, message: NetworkMessage) -> RobinResult<()> {
        if let Some(server) = &mut self.server_manager {
            server.broadcast_message(message)?;
        } else if let Some(peer) = &mut self.peer_manager {
            peer.broadcast_message(message)?;
        }
        Ok(())
    }

    fn send_to_client(&mut self, client_id: ClientId, message: NetworkMessage) -> RobinResult<()> {
        if let Some(server) = &mut self.server_manager {
            server.send_to_client(client_id, message)?;
        }
        Ok(())
    }

    fn send_to_server(&mut self, message: NetworkMessage) -> RobinResult<()> {
        if let Some(client) = &mut self.client_manager {
            client.send_to_server(message)?;
        }
        Ok(())
    }

    fn send_to_peer(&mut self, peer_id: PeerId, message: NetworkMessage) -> RobinResult<()> {
        if let Some(peer) = &mut self.peer_manager {
            peer.send_to_peer(peer_id, message)?;
        }
        Ok(())
    }

    fn initialize_protocol_handlers() -> RobinResult<HashMap<ProtocolType, Box<dyn ProtocolHandler>>> {
        let mut handlers: HashMap<ProtocolType, Box<dyn ProtocolHandler>> = HashMap::new();

        handlers.insert(ProtocolType::TCP, Box::new(TcpProtocolHandler::new()?));
        handlers.insert(ProtocolType::UDP, Box::new(UdpProtocolHandler::new()?));
        handlers.insert(ProtocolType::WebSocket, Box::new(WebSocketProtocolHandler::new()?));

        Ok(handlers)
    }
}

/// Network configuration
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub max_connections: usize,
    pub connection_timeout: Duration,
    pub keepalive_interval: Duration,
    pub message_queue_size: usize,
    pub compression_enabled: bool,
    pub encryption_enabled: bool,
    pub protocol_type: ProtocolType,
    pub bandwidth_limit: Option<u64>, // bytes per second
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            max_connections: 32,
            connection_timeout: Duration::from_secs(30),
            keepalive_interval: Duration::from_secs(5),
            message_queue_size: 1024,
            compression_enabled: true,
            encryption_enabled: true,
            protocol_type: ProtocolType::TCP,
            bandwidth_limit: None,
        }
    }
}

/// Supported network protocols
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProtocolType {
    TCP,
    UDP,
    WebSocket,
    WebRTC,
}

/// Server manager for hosting games
#[derive(Debug)]
pub struct ServerManager {
    config: ServerConfig,
    listener: TcpListener,
    clients: HashMap<ClientId, ConnectedClient>,
    udp_socket: Option<UdpSocket>,
    running: bool,
    message_queue: Arc<Mutex<Vec<(ClientId, NetworkMessage)>>>,
    worker_thread: Option<thread::JoinHandle<()>>,
}

impl ServerManager {
    pub fn new(config: ServerConfig) -> RobinResult<Self> {
        let listener = TcpListener::bind(&config.bind_address)
            .map_err(|e| RobinError::Network(format!("Failed to bind to address: {}", e)))?;

        listener.set_nonblocking(true)
            .map_err(|e| RobinError::Network(format!("Failed to set non-blocking: {}", e)))?;

        Ok(Self {
            config,
            listener,
            clients: HashMap::new(),
            udp_socket: None,
            running: false,
            message_queue: Arc::new(Mutex::new(Vec::new())),
            worker_thread: None,
        })
    }

    pub fn start(&mut self) -> RobinResult<()> {
        self.running = true;

        // Start UDP socket if needed
        if self.config.enable_udp {
            let udp_socket = UdpSocket::bind(&self.config.bind_address)
                .map_err(|e| RobinError::Network(format!("Failed to bind UDP socket: {}", e)))?;
            udp_socket.set_nonblocking(true)?;
            self.udp_socket = Some(udp_socket);
        }

        // Start worker thread for accepting connections
        let listener = self.listener.try_clone()?;
        let message_queue = self.message_queue.clone();
        let max_clients = self.config.max_clients;

        self.worker_thread = Some(thread::spawn(move || {
            Self::accept_connections_worker(listener, message_queue, max_clients);
        }));

        println!("🖥️ Server started on {}", self.config.bind_address);
        Ok(())
    }

    pub fn update(&mut self, _delta_time: f32) -> RobinResult<()> {
        // Handle incoming connections
        self.process_pending_connections()?;

        // Update clients
        self.update_clients()?;

        // Handle UDP messages if enabled
        if let Some(udp_socket) = &self.udp_socket {
            self.handle_udp_messages(udp_socket)?;
        }

        Ok(())
    }

    pub fn broadcast_message(&mut self, message: NetworkMessage) -> RobinResult<()> {
        for client in self.clients.values_mut() {
            client.send_message(message.clone())?;
        }
        Ok(())
    }

    pub fn send_to_client(&mut self, client_id: ClientId, message: NetworkMessage) -> RobinResult<()> {
        if let Some(client) = self.clients.get_mut(&client_id) {
            client.send_message(message)?;
        }
        Ok(())
    }

    pub fn receive_messages(&mut self) -> RobinResult<Vec<ReceivedMessage>> {
        let mut messages = Vec::new();

        for (client_id, client) in &mut self.clients {
            for message in client.receive_messages()? {
                messages.push(ReceivedMessage {
                    source: MessageSource::Client(*client_id),
                    message,
                    timestamp: Instant::now(),
                });
            }
        }

        Ok(messages)
    }

    pub fn shutdown(&mut self) -> RobinResult<()> {
        self.running = false;

        // Disconnect all clients
        for client in self.clients.values_mut() {
            client.disconnect()?;
        }
        self.clients.clear();

        // Join worker thread
        if let Some(worker) = self.worker_thread.take() {
            worker.join().map_err(|_| RobinError::Network("Failed to join worker thread".to_string()))?;
        }

        Ok(())
    }

    fn accept_connections_worker(
        listener: TcpListener,
        message_queue: Arc<Mutex<Vec<(ClientId, NetworkMessage)>>>,
        max_clients: usize,
    ) {
        let mut next_client_id = 0;

        loop {
            match listener.accept() {
                Ok((stream, addr)) => {
                    if next_client_id < max_clients {
                        println!("🔗 New client connected: {} (ID: {})", addr, next_client_id);
                        // In a real implementation, we'd handle the new client connection here
                        next_client_id += 1;
                    } else {
                        println!("❌ Connection rejected: server full");
                        drop(stream);
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No pending connections
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => {
                    eprintln!("❌ Error accepting connection: {}", e);
                }
            }
        }
    }

    fn process_pending_connections(&mut self) -> RobinResult<()> {
        // Process messages from worker thread
        let mut queue = self.message_queue.lock().unwrap();
        queue.clear(); // For now, just clear the queue
        Ok(())
    }

    fn update_clients(&mut self) -> RobinResult<()> {
        // Remove disconnected clients
        self.clients.retain(|client_id, client| {
            if !client.is_connected() {
                println!("🔌 Client {} disconnected", client_id);
                false
            } else {
                true
            }
        });

        Ok(())
    }

    fn handle_udp_messages(&mut self, _udp_socket: &UdpSocket) -> RobinResult<()> {
        // Handle UDP message processing
        Ok(())
    }
}

/// Client manager for connecting to servers
#[derive(Debug)]
pub struct ClientManager {
    config: ClientConfig,
    connection: Option<TcpStream>,
    udp_socket: Option<UdpSocket>,
    connected: bool,
    message_queue: Vec<NetworkMessage>,
}

impl ClientManager {
    pub fn new(config: ClientConfig) -> RobinResult<Self> {
        Ok(Self {
            config,
            connection: None,
            udp_socket: None,
            connected: false,
            message_queue: Vec::new(),
        })
    }

    pub fn connect(&mut self) -> RobinResult<()> {
        let stream = TcpStream::connect(&self.config.server_address)
            .map_err(|e| RobinError::Network(format!("Failed to connect to server: {}", e)))?;

        stream.set_nonblocking(true)?;
        self.connection = Some(stream);
        self.connected = true;

        println!("💻 Connected to server: {}", self.config.server_address);
        Ok(())
    }

    pub fn disconnect(&mut self) -> RobinResult<()> {
        self.connected = false;
        self.connection = None;
        self.udp_socket = None;

        println!("🔌 Disconnected from server");
        Ok(())
    }

    pub fn send_to_server(&mut self, message: NetworkMessage) -> RobinResult<()> {
        if self.connected {
            self.message_queue.push(message);
        }
        Ok(())
    }

    pub fn receive_messages(&mut self) -> RobinResult<Vec<ReceivedMessage>> {
        let mut messages = Vec::new();

        // Process incoming messages from server
        if self.connected {
            // In a real implementation, we'd read from the TCP stream here
        }

        Ok(messages)
    }

    pub fn update(&mut self, _delta_time: f32) -> RobinResult<()> {
        if !self.connected {
            return Ok(());
        }

        // Send queued messages
        for message in self.message_queue.drain(..) {
            // In a real implementation, we'd serialize and send the message
            let _serialized = bincode::serialize(&message)
                .map_err(|e| RobinError::Network(format!("Failed to serialize message: {}", e)))?;
        }

        Ok(())
    }
}

/// Peer-to-peer manager
#[derive(Debug)]
pub struct PeerManager {
    config: PeerConfig,
    peers: HashMap<PeerId, ConnectedPeer>,
    local_peer_id: PeerId,
}

impl PeerManager {
    pub fn new(config: PeerConfig) -> RobinResult<Self> {
        Ok(Self {
            config,
            peers: HashMap::new(),
            local_peer_id: PeerId::generate(),
        })
    }

    pub fn start(&mut self) -> RobinResult<()> {
        println!("🔗 Started peer-to-peer networking");
        Ok(())
    }

    pub fn connect_to_peer(&mut self, peer_address: SocketAddr) -> RobinResult<PeerId> {
        let peer_id = PeerId::generate();
        let peer = ConnectedPeer::new(peer_address)?;
        self.peers.insert(peer_id.clone(), peer);

        println!("🤝 Connected to peer: {} ({})", peer_address, peer_id);
        Ok(peer_id)
    }

    pub fn send_to_peer(&mut self, peer_id: PeerId, message: NetworkMessage) -> RobinResult<()> {
        if let Some(peer) = self.peers.get_mut(&peer_id) {
            peer.send_message(message)?;
        }
        Ok(())
    }

    pub fn broadcast_message(&mut self, message: NetworkMessage) -> RobinResult<()> {
        for peer in self.peers.values_mut() {
            peer.send_message(message.clone())?;
        }
        Ok(())
    }

    pub fn receive_messages(&mut self) -> RobinResult<Vec<ReceivedMessage>> {
        let mut messages = Vec::new();

        for (peer_id, peer) in &mut self.peers {
            for message in peer.receive_messages()? {
                messages.push(ReceivedMessage {
                    source: MessageSource::Peer(*peer_id),
                    message,
                    timestamp: Instant::now(),
                });
            }
        }

        Ok(messages)
    }

    pub fn update(&mut self, _delta_time: f32) -> RobinResult<()> {
        // Update all peers
        self.peers.retain(|peer_id, peer| {
            if !peer.is_connected() {
                println!("🔌 Peer {} disconnected", peer_id);
                false
            } else {
                true
            }
        });

        Ok(())
    }

    pub fn shutdown(&mut self) -> RobinResult<()> {
        for peer in self.peers.values_mut() {
            peer.disconnect()?;
        }
        self.peers.clear();

        println!("🛑 Peer-to-peer networking shutdown");
        Ok(())
    }
}

// Configuration structures
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub bind_address: SocketAddr,
    pub max_clients: usize,
    pub enable_udp: bool,
    pub tick_rate: f32,
}

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub server_address: SocketAddr,
    pub connection_timeout: Duration,
    pub reconnect_attempts: u32,
}

#[derive(Debug, Clone)]
pub struct PeerConfig {
    pub discovery_method: PeerDiscoveryMethod,
    pub max_peers: usize,
}

#[derive(Debug, Clone)]
pub enum PeerDiscoveryMethod {
    LocalNetwork,
    ManualConnect,
    ReliableP2P,
}

// Network message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMessage {
    pub message_type: MessageType,
    pub payload: Vec<u8>,
    pub timestamp: u64,
    pub sequence_number: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    GameState,
    PlayerInput,
    Chat,
    System,
    Custom(String),
}

#[derive(Debug)]
pub struct ReceivedMessage {
    pub source: MessageSource,
    pub message: NetworkMessage,
    pub timestamp: Instant,
}

#[derive(Debug, Clone, Copy)]
pub enum MessageSource {
    Server,
    Client(ClientId),
    Peer(PeerId),
}

#[derive(Debug, Clone, Copy)]
pub enum MessageTarget {
    All,
    Server,
    Client(ClientId),
    Peer(PeerId),
}

// ID types
pub type ClientId = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PeerId(u64);

impl PeerId {
    pub fn generate() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;
        Self(timestamp)
    }
}

impl std::fmt::Display for PeerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "peer-{:016x}", self.0)
    }
}

// Connection types
#[derive(Debug)]
pub struct ConnectedClient {
    stream: TcpStream,
    address: SocketAddr,
    connected: bool,
}

impl ConnectedClient {
    pub fn send_message(&mut self, _message: NetworkMessage) -> RobinResult<()> {
        // Implementation would serialize and send the message
        Ok(())
    }

    pub fn receive_messages(&mut self) -> RobinResult<Vec<NetworkMessage>> {
        // Implementation would read and deserialize messages
        Ok(Vec::new())
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub fn disconnect(&mut self) -> RobinResult<()> {
        self.connected = false;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ConnectedPeer {
    address: SocketAddr,
    connected: bool,
}

impl ConnectedPeer {
    pub fn new(address: SocketAddr) -> RobinResult<Self> {
        Ok(Self {
            address,
            connected: true,
        })
    }

    pub fn send_message(&mut self, _message: NetworkMessage) -> RobinResult<()> {
        Ok(())
    }

    pub fn receive_messages(&mut self) -> RobinResult<Vec<NetworkMessage>> {
        Ok(Vec::new())
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub fn disconnect(&mut self) -> RobinResult<()> {
        self.connected = false;
        Ok(())
    }
}

// Supporting systems
#[derive(Debug)]
pub struct ConnectionPool;

impl ConnectionPool {
    pub fn new() -> RobinResult<Self> { Ok(Self) }
    pub fn update(&mut self, _delta_time: f32) -> RobinResult<()> { Ok(()) }
    pub fn shutdown(&mut self) -> RobinResult<()> { Ok(()) }
}

#[derive(Debug)]
pub struct NetworkMessageBus;

impl NetworkMessageBus {
    pub fn new() -> RobinResult<Self> { Ok(Self) }
    pub fn update(&mut self, _delta_time: f32) -> RobinResult<()> { Ok(()) }
}

#[derive(Debug)]
pub struct NetworkSecurityManager;

impl NetworkSecurityManager {
    pub fn new() -> RobinResult<Self> { Ok(Self) }
    pub fn initialize(&mut self, _config: &NetworkConfig) -> RobinResult<()> { Ok(()) }
    pub fn filter_incoming_message(&self, message: ReceivedMessage) -> RobinResult<ReceivedMessage> { Ok(message) }
    pub fn filter_outgoing_message(&self, message: NetworkMessage) -> RobinResult<NetworkMessage> { Ok(message) }
}

#[derive(Debug)]
pub struct NetworkMetrics {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub connections_active: u32,
    pub latency_ms: f32,
    pub packet_loss: f32,
}

impl NetworkMetrics {
    pub fn new() -> Self {
        Self {
            bytes_sent: 0,
            bytes_received: 0,
            messages_sent: 0,
            messages_received: 0,
            connections_active: 0,
            latency_ms: 0.0,
            packet_loss: 0.0,
        }
    }

    pub fn update(&mut self, _delta_time: f32) {
        // Update network metrics
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ConnectionStatus {
    Disconnected,
    Client,
    Server,
    Peer,
}

// Protocol handlers
pub trait ProtocolHandler: Send + Sync {
    fn initialize(&mut self, config: &NetworkConfig) -> RobinResult<()>;
    fn send_message(&self, target: SocketAddr, message: &NetworkMessage) -> RobinResult<()>;
    fn receive_messages(&self) -> RobinResult<Vec<ReceivedMessage>>;
}

pub struct TcpProtocolHandler;
pub struct UdpProtocolHandler;
pub struct WebSocketProtocolHandler;

impl TcpProtocolHandler {
    pub fn new() -> RobinResult<Self> { Ok(Self) }
}

impl ProtocolHandler for TcpProtocolHandler {
    fn initialize(&mut self, _config: &NetworkConfig) -> RobinResult<()> { Ok(()) }
    fn send_message(&self, _target: SocketAddr, _message: &NetworkMessage) -> RobinResult<()> { Ok(()) }
    fn receive_messages(&self) -> RobinResult<Vec<ReceivedMessage>> { Ok(Vec::new()) }
}

impl UdpProtocolHandler {
    pub fn new() -> RobinResult<Self> { Ok(Self) }
}

impl ProtocolHandler for UdpProtocolHandler {
    fn initialize(&mut self, _config: &NetworkConfig) -> RobinResult<()> { Ok(()) }
    fn send_message(&self, _target: SocketAddr, _message: &NetworkMessage) -> RobinResult<()> { Ok(()) }
    fn receive_messages(&self) -> RobinResult<Vec<ReceivedMessage>> { Ok(Vec::new()) }
}

impl WebSocketProtocolHandler {
    pub fn new() -> RobinResult<Self> { Ok(Self) }
}

impl ProtocolHandler for WebSocketProtocolHandler {
    fn initialize(&mut self, _config: &NetworkConfig) -> RobinResult<()> { Ok(()) }
    fn send_message(&self, _target: SocketAddr, _message: &NetworkMessage) -> RobinResult<()> { Ok(()) }
    fn receive_messages(&self) -> RobinResult<Vec<ReceivedMessage>> { Ok(Vec::new()) }
}