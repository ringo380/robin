/*!
 * Networking System for Robin Engine
 *
 * Provides multiplayer support for collaborative voxel building,
 * player synchronization, and real-time world updates.
 */

pub mod server;
pub mod client;
pub mod protocol;
pub mod sync;

use crate::engine::error::{RobinError, RobinResult};
use std::net::SocketAddr;
use tokio::sync::mpsc;

pub use server::{GameServer, ServerConfig};
pub use client::{GameClient, ClientConfig};
pub use protocol::{NetworkMessage, NetworkEvent, PlayerAction};
pub use sync::{WorldSyncManager, SyncState};

/// Network mode for the game
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkMode {
    /// Single player (no networking)
    SinglePlayer,
    /// Hosting a server
    Host,
    /// Connected to a server
    Client,
}

/// Network statistics
#[derive(Debug, Clone, Default)]
pub struct NetworkStats {
    pub packets_sent: u64,
    pub packets_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub ping_ms: u32,
    pub connected_players: usize,
}

/// Main networking manager
pub struct NetworkManager {
    mode: NetworkMode,
    server: Option<GameServer>,
    client: Option<GameClient>,
    stats: NetworkStats,
}

impl NetworkManager {
    pub fn new() -> Self {
        Self {
            mode: NetworkMode::SinglePlayer,
            server: None,
            client: None,
            stats: NetworkStats::default(),
        }
    }

    /// Start hosting a server
    pub async fn host_server(&mut self, config: ServerConfig) -> RobinResult<()> {
        if self.mode != NetworkMode::SinglePlayer {
            return Err(RobinError::NetworkError {
                operation: "host_server".to_string(),
                endpoint: "localhost".to_string(),
                reason: "Already in network mode".to_string(),
            });
        }

        let server = GameServer::new(config).await?;
        self.server = Some(server);
        self.mode = NetworkMode::Host;

        println!("🌐 Server started on port {}", config.port);
        Ok(())
    }

    /// Connect to a server
    pub async fn connect_to_server(&mut self, address: SocketAddr, player_name: String) -> RobinResult<()> {
        if self.mode != NetworkMode::SinglePlayer {
            return Err(RobinError::NetworkError {
                operation: "connect_to_server".to_string(),
                endpoint: address.to_string(),
                reason: "Already in network mode".to_string(),
            });
        }

        let client = GameClient::connect(address, player_name).await?;
        self.client = Some(client);
        self.mode = NetworkMode::Client;

        println!("🔗 Connected to server at {}", address);
        Ok(())
    }

    /// Disconnect from network
    pub async fn disconnect(&mut self) -> RobinResult<()> {
        match self.mode {
            NetworkMode::Host => {
                if let Some(mut server) = self.server.take() {
                    server.shutdown().await?;
                }
            }
            NetworkMode::Client => {
                if let Some(mut client) = self.client.take() {
                    client.disconnect().await?;
                }
            }
            _ => {}
        }

        self.mode = NetworkMode::SinglePlayer;
        println!("📡 Disconnected from network");
        Ok(())
    }

    /// Get current network mode
    pub fn get_mode(&self) -> NetworkMode {
        self.mode
    }

    /// Get network statistics
    pub fn get_stats(&self) -> &NetworkStats {
        &self.stats
    }

    /// Update network (called each frame)
    pub async fn update(&mut self) -> RobinResult<Vec<NetworkEvent>> {
        let mut events = Vec::new();

        match self.mode {
            NetworkMode::Host => {
                if let Some(ref mut server) = self.server {
                    events = server.update().await?;
                    self.stats = server.get_stats();
                }
            }
            NetworkMode::Client => {
                if let Some(ref mut client) = self.client {
                    events = client.update().await?;
                    self.stats.ping_ms = client.get_ping();
                }
            }
            _ => {}
        }

        Ok(events)
    }

    /// Send a network message
    pub async fn send_message(&mut self, message: NetworkMessage) -> RobinResult<()> {
        match self.mode {
            NetworkMode::Host => {
                if let Some(ref mut server) = self.server {
                    server.broadcast(message).await?;
                }
            }
            NetworkMode::Client => {
                if let Some(ref mut client) = self.client {
                    client.send(message).await?;
                }
            }
            _ => {}
        }

        Ok(())
    }
}

impl Default for NetworkManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_network_manager_creation() {
        let manager = NetworkManager::new();
        assert_eq!(manager.get_mode(), NetworkMode::SinglePlayer);
    }
}