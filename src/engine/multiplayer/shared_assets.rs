use crate::engine::error::RobinResult;
use crate::engine::multiplayer::UserId;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMetadata {
    pub id: String,
    pub name: String,
    pub author: UserId,
    pub category: AssetCategory,
    pub file_size: u64,
    pub version: String,
    pub description: String,
    pub tags: Vec<String>,
    pub created_at: f64,
    pub updated_at: f64,
    pub download_count: u64,
    pub rating: f32,
    pub public: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssetCategory {
    Model,
    Texture,
    Script,
    Audio,
    Animation,
    Structure,
    Template,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssetSyncStatus {
    Synced,
    Pending,
    Downloading,
    Error(String),
}

#[derive(Debug)]
pub struct AssetLibrary {
    assets: HashMap<String, AssetMetadata>,
    sync_status: HashMap<String, AssetSyncStatus>,
}

impl AssetLibrary {
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
            sync_status: HashMap::new(),
        }
    }

    pub fn add_asset(&mut self, mut asset: AssetMetadata) -> RobinResult<String> {
        asset.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        asset.updated_at = asset.created_at;
        
        let asset_id = asset.id.clone();
        self.assets.insert(asset_id.clone(), asset);
        self.sync_status.insert(asset_id.clone(), AssetSyncStatus::Synced);
        
        Ok(asset_id)
    }

    pub fn get_asset(&self, asset_id: &str) -> Option<&AssetMetadata> {
        self.assets.get(asset_id)
    }

    pub fn search_assets(&self, query: &str) -> Vec<&AssetMetadata> {
        self.assets.values()
            .filter(|asset| {
                asset.name.to_lowercase().contains(&query.to_lowercase()) ||
                asset.description.to_lowercase().contains(&query.to_lowercase()) ||
                asset.tags.iter().any(|tag| tag.to_lowercase().contains(&query.to_lowercase()))
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct SharedAssetManager {
    local_library: AssetLibrary,
    remote_libraries: HashMap<String, AssetLibrary>,
}

impl SharedAssetManager {
    pub fn new() -> RobinResult<Self> {
        Ok(Self {
            local_library: AssetLibrary::new(),
            remote_libraries: HashMap::new(),
        })
    }

    pub fn sync_asset(&mut self, asset_id: &str) -> RobinResult<()> {
        if let Some(status) = self.local_library.sync_status.get_mut(asset_id) {
            *status = AssetSyncStatus::Pending;
        }
        Ok(())
    }

    pub fn update(&mut self, _delta_time: f32) -> RobinResult<()> {
        Ok(())
    }
}