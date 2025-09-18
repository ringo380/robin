/// High-performance asset database with connection pooling and optimizations
///
/// This module provides:
/// - R2D2 connection pooling for SQLite
/// - Prepared statement caching
/// - Optimized indexing strategies
/// - Background maintenance tasks
/// - Query performance monitoring

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant, SystemTime},
};

use ahash::AHashMap;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use lru::LruCache;
use parking_lot::{Mutex, RwLock};
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Connection, OptionalExtension, Result as SqliteResult, Transaction};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use uuid::Uuid;

use super::{AssetError, AssetMetadata, AssetResult, AssetType};

/// Configuration for the optimized database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedDatabaseConfig {
    /// Database file path
    pub database_path: PathBuf,
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Minimum number of connections in the pool
    pub min_connections: u32,
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    /// Maximum lifetime of a connection in seconds
    pub max_lifetime: u64,
    /// Enable Write-Ahead Logging (WAL) mode
    pub enable_wal: bool,
    /// Cache size for prepared statements
    pub statement_cache_size: usize,
    /// Query result cache size in bytes
    pub query_cache_size: usize,
    /// Cache TTL in seconds
    pub cache_ttl: u64,
    /// Enable query performance monitoring
    pub enable_monitoring: bool,
    /// Maintenance interval in seconds
    pub maintenance_interval: u64,
}

impl Default for OptimizedDatabaseConfig {
    fn default() -> Self {
        Self {
            database_path: PathBuf::from("assets.db"),
            max_connections: num_cpus::get() as u32 * 2,
            min_connections: 2,
            connection_timeout: 30,
            max_lifetime: 3600, // 1 hour
            enable_wal: true,
            statement_cache_size: 256,
            query_cache_size: 64 * 1024 * 1024, // 64MB
            cache_ttl: 300, // 5 minutes
            enable_monitoring: true,
            maintenance_interval: 3600, // 1 hour
        }
    }
}

/// Prepared statement cache entry
#[derive(Debug)]
struct CachedStatement {
    sql: String,
    last_used: Instant,
    use_count: AtomicU64,
}

impl Clone for CachedStatement {
    fn clone(&self) -> Self {
        Self {
            sql: self.sql.clone(),
            last_used: self.last_used,
            use_count: AtomicU64::new(self.use_count.load(Ordering::Relaxed)),
        }
    }
}

/// Query result cache entry
#[derive(Debug)]
struct QueryCacheEntry {
    data: Vec<u8>,
    created_at: SystemTime,
    last_accessed: Instant,
    access_count: AtomicU64,
}

impl Clone for QueryCacheEntry {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            created_at: self.created_at,
            last_accessed: self.last_accessed,
            access_count: AtomicU64::new(self.access_count.load(Ordering::Relaxed)),
        }
    }
}

/// Performance metrics for database operations
#[derive(Debug, Default)]
pub struct DatabaseMetrics {
    pub total_queries: AtomicU64,
    pub successful_queries: AtomicU64,
    pub failed_queries: AtomicU64,
    pub cache_hits: AtomicU64,
    pub cache_misses: AtomicU64,
    pub total_query_time_us: AtomicU64,
    pub connection_waits: AtomicU64,
    pub maintenance_runs: AtomicU64,
}

impl DatabaseMetrics {
    pub fn cache_hit_rate(&self) -> f64 {
        let hits = self.cache_hits.load(Ordering::Relaxed);
        let misses = self.cache_misses.load(Ordering::Relaxed);
        let total = hits + misses;

        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    pub fn success_rate(&self) -> f64 {
        let successful = self.successful_queries.load(Ordering::Relaxed);
        let total = self.total_queries.load(Ordering::Relaxed);

        if total == 0 {
            0.0
        } else {
            successful as f64 / total as f64
        }
    }

    pub fn average_query_time(&self) -> Duration {
        let total_time = self.total_query_time_us.load(Ordering::Relaxed);
        let total_queries = self.total_queries.load(Ordering::Relaxed);

        if total_queries == 0 {
            Duration::ZERO
        } else {
            Duration::from_micros(total_time / total_queries)
        }
    }
}

/// High-performance asset database
pub struct OptimizedAssetDatabase {
    pool: Pool<SqliteConnectionManager>,
    config: OptimizedDatabaseConfig,
    statement_cache: Arc<Mutex<LruCache<String, CachedStatement>>>,
    query_cache: Arc<RwLock<LruCache<String, QueryCacheEntry>>>,
    metrics: Arc<DatabaseMetrics>,
    maintenance_handle: Option<tokio::task::JoinHandle<()>>,
}

impl OptimizedAssetDatabase {
    /// Create a new optimized asset database
    pub async fn new(config: OptimizedDatabaseConfig) -> AssetResult<Self> {
        // Create database directory if it doesn't exist
        if let Some(parent) = config.database_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AssetError::LoadFailed(format!("Failed to create database directory: {}", e)))?;
        }

        // Configure connection manager
        let enable_wal = config.enable_wal;
        let manager = SqliteConnectionManager::file(&config.database_path)
            .with_init(move |conn| {
                // Enable optimizations
                conn.execute_batch(&format!(
                    "
                    PRAGMA journal_mode = {};
                    PRAGMA synchronous = NORMAL;
                    PRAGMA cache_size = -65536;  -- 64MB
                    PRAGMA temp_store = MEMORY;
                    PRAGMA mmap_size = 268435456;  -- 256MB
                    PRAGMA optimize;
                    ",
                    if enable_wal { "WAL" } else { "DELETE" }
                ))?;
                Ok(())
            });

        // Create connection pool
        let pool = Pool::builder()
            .max_size(config.max_connections)
            .min_idle(Some(config.min_connections))
            .connection_timeout(Duration::from_secs(config.connection_timeout))
            .max_lifetime(Some(Duration::from_secs(config.max_lifetime)))
            .build(manager)
            .map_err(|e| AssetError::LoadFailed(format!("Failed to create connection pool: {}", e)))?;

        // Initialize database schema
        {
            let mut conn = pool.get()
                .map_err(|e| AssetError::LoadFailed(format!("Failed to get connection: {}", e)))?;
            Self::initialize_schema(&mut conn)?;
            Self::create_indexes(&mut conn)?;
        }

        // Create caches
        let statement_cache = Arc::new(Mutex::new(LruCache::new(
            std::num::NonZeroUsize::new(config.statement_cache_size).unwrap()
        )));

        let query_cache_entries = config.query_cache_size / 1024; // Estimate entries
        let query_cache = Arc::new(RwLock::new(LruCache::new(
            std::num::NonZeroUsize::new(query_cache_entries).unwrap()
        )));

        let metrics = Arc::new(DatabaseMetrics::default());

        // Start background maintenance if enabled
        let maintenance_handle = if config.enable_monitoring {
            let pool_clone = pool.clone();
            let metrics_clone = Arc::clone(&metrics);
            let query_cache_clone = Arc::clone(&query_cache);
            let interval = config.maintenance_interval;

            Some(tokio::spawn(async move {
                Self::run_background_maintenance(pool_clone, metrics_clone, query_cache_clone, interval).await;
            }))
        } else {
            None
        };

        Ok(Self {
            pool,
            config,
            statement_cache,
            query_cache,
            metrics,
            maintenance_handle,
        })
    }

    /// Initialize database schema with optimized table structures
    fn initialize_schema(conn: &mut PooledConnection<SqliteConnectionManager>) -> AssetResult<()> {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS assets (
                id TEXT PRIMARY KEY,
                asset_type INTEGER NOT NULL,
                file_path TEXT NOT NULL UNIQUE,
                relative_path TEXT NOT NULL,
                last_modified INTEGER NOT NULL,
                file_size INTEGER NOT NULL,
                checksum TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                load_count INTEGER DEFAULT 0,
                memory_usage INTEGER DEFAULT 0,
                compression_ratio REAL DEFAULT 1.0,
                metadata_json TEXT
            );

            CREATE TABLE IF NOT EXISTS asset_dependencies (
                asset_id TEXT NOT NULL,
                dependency_id TEXT NOT NULL,
                dependency_type INTEGER NOT NULL,
                PRIMARY KEY (asset_id, dependency_id),
                FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE,
                FOREIGN KEY (dependency_id) REFERENCES assets(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS asset_tags (
                asset_id TEXT NOT NULL,
                tag TEXT NOT NULL,
                PRIMARY KEY (asset_id, tag),
                FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS asset_versions (
                id TEXT PRIMARY KEY,
                asset_id TEXT NOT NULL,
                version_number INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                data_hash TEXT NOT NULL,
                change_description TEXT,
                FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS performance_metrics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                metric_type TEXT NOT NULL,
                metric_value REAL NOT NULL,
                recorded_at INTEGER NOT NULL,
                metadata_json TEXT
            );
            "
        ).map_err(|e| AssetError::LoadFailed(format!("Failed to initialize schema: {}", e)))?;

        Ok(())
    }

    /// Create optimized indexes for performance
    fn create_indexes(conn: &mut PooledConnection<SqliteConnectionManager>) -> AssetResult<()> {
        conn.execute_batch(
            "
            -- Primary lookup indexes
            CREATE INDEX IF NOT EXISTS idx_assets_file_path ON assets(file_path);
            CREATE INDEX IF NOT EXISTS idx_assets_type ON assets(asset_type);
            CREATE INDEX IF NOT EXISTS idx_assets_last_modified ON assets(last_modified);
            CREATE INDEX IF NOT EXISTS idx_assets_relative_path ON assets(relative_path);

            -- Composite indexes for common queries
            CREATE INDEX IF NOT EXISTS idx_assets_type_modified ON assets(asset_type, last_modified);
            CREATE INDEX IF NOT EXISTS idx_assets_path_type ON assets(relative_path, asset_type);

            -- Dependency indexes
            CREATE INDEX IF NOT EXISTS idx_dependencies_asset ON asset_dependencies(asset_id);
            CREATE INDEX IF NOT EXISTS idx_dependencies_dep ON asset_dependencies(dependency_id);
            CREATE INDEX IF NOT EXISTS idx_dependencies_type ON asset_dependencies(dependency_type);

            -- Tag indexes
            CREATE INDEX IF NOT EXISTS idx_tags_asset ON asset_tags(asset_id);
            CREATE INDEX IF NOT EXISTS idx_tags_tag ON asset_tags(tag);

            -- Version indexes
            CREATE INDEX IF NOT EXISTS idx_versions_asset ON asset_versions(asset_id);
            CREATE INDEX IF NOT EXISTS idx_versions_created ON asset_versions(created_at);

            -- Performance indexes
            CREATE INDEX IF NOT EXISTS idx_metrics_type ON performance_metrics(metric_type);
            CREATE INDEX IF NOT EXISTS idx_metrics_recorded ON performance_metrics(recorded_at);
            CREATE INDEX IF NOT EXISTS idx_metrics_type_recorded ON performance_metrics(metric_type, recorded_at);
            "
        ).map_err(|e| AssetError::LoadFailed(format!("Failed to create indexes: {}", e)))?;

        Ok(())
    }

    /// Execute a query with automatic caching and performance monitoring
    pub async fn execute_cached_query<T, F>(
        &self,
        query_key: &str,
        sql: &str,
        params: &[&dyn rusqlite::ToSql],
        mapper: F,
    ) -> AssetResult<T>
    where
        F: FnOnce(&rusqlite::Row) -> SqliteResult<T>,
        T: Clone + serde::Serialize + serde::de::DeserializeOwned,
    {
        let start_time = Instant::now();

        // Check query cache first
        if let Some(cached_result) = self.get_cached_query_result::<T>(query_key) {
            self.metrics.cache_hits.fetch_add(1, Ordering::Relaxed);
            return Ok(cached_result);
        }

        self.metrics.cache_misses.fetch_add(1, Ordering::Relaxed);

        // Execute query
        let conn = self.pool.get()
            .map_err(|e| {
                self.metrics.connection_waits.fetch_add(1, Ordering::Relaxed);
                AssetError::LoadFailed(format!("Failed to get connection: {}", e))
            })?;

        let result = conn.query_row(sql, params, mapper)
            .map_err(|e| AssetError::LoadFailed(format!("Query failed: {}", e)))?;

        // Cache result
        self.cache_query_result(query_key, &result);

        // Update metrics
        let query_time = start_time.elapsed();
        self.metrics.total_queries.fetch_add(1, Ordering::Relaxed);
        self.metrics.successful_queries.fetch_add(1, Ordering::Relaxed);
        self.metrics.total_query_time_us.fetch_add(
            query_time.as_micros() as u64,
            Ordering::Relaxed,
        );

        Ok(result)
    }

    /// Bulk insert assets with optimized transaction handling
    pub async fn bulk_insert_assets(&self, assets: &[AssetMetadata]) -> AssetResult<()> {
        if assets.is_empty() {
            return Ok(());
        }

        let start_time = Instant::now();

        let mut conn = self.pool.get()
            .map_err(|e| AssetError::LoadFailed(format!("Failed to get connection: {}", e)))?;

        let tx = conn.transaction()
            .map_err(|e| AssetError::LoadFailed(format!("Failed to start transaction: {}", e)))?;

        // Prepare statement once for all inserts
        let mut stmt = tx.prepare_cached(
            "INSERT OR REPLACE INTO assets
             (id, asset_type, file_path, relative_path, last_modified, file_size,
              checksum, created_at, updated_at, metadata_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"
        ).map_err(|e| AssetError::LoadFailed(format!("Failed to prepare statement: {}", e)))?;

        // Batch insert with optimized parameters
        for asset in assets {
            let metadata_json = serde_json::to_string(&asset)
                .unwrap_or_else(|_| "{}".to_string());

            let relative_path = asset.file_path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown");

            stmt.execute(params![
                asset.id,
                asset.asset_type.clone() as i32,
                asset.file_path.to_string_lossy(),
                relative_path,
                asset.last_modified.duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default().as_secs() as i64,
                asset.memory_size as i64,
                format!("{:x}", md5::compute(asset.id.as_bytes())), // Simple checksum
                chrono::Utc::now().timestamp(),
                chrono::Utc::now().timestamp(),
                metadata_json
            ]).map_err(|e| AssetError::LoadFailed(format!("Failed to insert asset: {}", e)))?;
        }

        drop(stmt); // Drop the statement before committing the transaction
        tx.commit()
            .map_err(|e| AssetError::LoadFailed(format!("Failed to commit transaction: {}", e)))?;

        // Update metrics
        let operation_time = start_time.elapsed();
        self.metrics.total_queries.fetch_add(1, Ordering::Relaxed);
        self.metrics.successful_queries.fetch_add(1, Ordering::Relaxed);
        self.metrics.total_query_time_us.fetch_add(
            operation_time.as_micros() as u64,
            Ordering::Relaxed,
        );

        Ok(())
    }

    /// Search assets with optimized queries and result caching
    pub async fn search_assets(
        &self,
        asset_type: Option<AssetType>,
        path_pattern: Option<&str>,
        tags: Option<&[String]>,
        limit: Option<u32>,
    ) -> AssetResult<Vec<AssetMetadata>> {
        let query_key = format!(
            "search_{}_{}_{}_{}",
            asset_type.as_ref().map(|t| format!("{:?}", t)).unwrap_or_else(|| "any".to_string()),
            path_pattern.unwrap_or("any"),
            tags.map(|t| t.join(",")).unwrap_or_else(|| "any".to_string()),
            limit.unwrap_or(1000)
        );

        // Check cache first
        if let Some(cached_results) = self.get_cached_query_result::<Vec<AssetMetadata>>(&query_key) {
            self.metrics.cache_hits.fetch_add(1, Ordering::Relaxed);
            return Ok(cached_results);
        }

        self.metrics.cache_misses.fetch_add(1, Ordering::Relaxed);

        // Build dynamic query
        let mut sql = "SELECT id, asset_type, file_path, relative_path, last_modified,
                           file_size, load_count, memory_usage, metadata_json
                      FROM assets WHERE 1=1".to_string();
        let mut params: SmallVec<[Box<dyn rusqlite::ToSql>; 8]> = SmallVec::new();

        if let Some(asset_type) = asset_type {
            sql.push_str(" AND asset_type = ?");
            params.push(Box::new(asset_type as i32));
        }

        if let Some(pattern) = path_pattern {
            sql.push_str(" AND (file_path LIKE ? OR relative_path LIKE ?)");
            params.push(Box::new(format!("%{}%", pattern)));
            params.push(Box::new(format!("%{}%", pattern)));
        }

        if let Some(tags) = tags {
            if !tags.is_empty() {
                sql.push_str(" AND id IN (SELECT asset_id FROM asset_tags WHERE tag IN (");
                sql.push_str(&"?,".repeat(tags.len()).trim_end_matches(','));
                sql.push_str("))");
                for tag in tags {
                    params.push(Box::new(tag.clone()));
                }
            }
        }

        sql.push_str(" ORDER BY last_modified DESC");

        if let Some(limit) = limit {
            sql.push_str(" LIMIT ?");
            params.push(Box::new(limit as i64));
        }

        // Execute query
        let conn = self.pool.get()
            .map_err(|e| AssetError::LoadFailed(format!("Failed to get connection: {}", e)))?;

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter()
            .map(|p| p.as_ref())
            .collect();

        let mut stmt = conn.prepare_cached(&sql)
            .map_err(|e| AssetError::LoadFailed(format!("Failed to prepare search query: {}", e)))?;

        let results: Vec<AssetMetadata> = stmt.query_map(&param_refs[..], |row| {
            let asset_type: i32 = row.get(1)?;
            let file_path: String = row.get(2)?;
            let last_modified_secs: i64 = row.get(4)?;

            Ok(AssetMetadata {
                id: row.get(0)?,
                asset_type: match asset_type {
                    0 => AssetType::Texture,
                    1 => AssetType::Audio,
                    2 => AssetType::Config,
                    3 => AssetType::Shader,
                    4 => AssetType::Scene,
                    5 => AssetType::Font,
                    _ => AssetType::Data,
                },
                file_path: PathBuf::from(file_path),
                last_modified: SystemTime::UNIX_EPOCH + Duration::from_secs(last_modified_secs as u64),
                dependencies: Vec::new(), // Load separately if needed
                load_count: row.get::<_, i64>(6)? as u32,
                memory_size: row.get::<_, i64>(7)? as usize,
            })
        })
        .map_err(|e| AssetError::LoadFailed(format!("Search query execution failed: {}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AssetError::LoadFailed(format!("Failed to collect search results: {}", e)))?;

        // Cache results
        self.cache_query_result(&query_key, &results);

        Ok(results)
    }

    /// Get cached query result
    fn get_cached_query_result<T>(&self, key: &str) -> Option<T>
    where
        T: Clone + serde::de::DeserializeOwned,
    {
        let cache = self.query_cache.read();

        if let Some(entry) = cache.peek(key) {
            // Check if entry is still valid
            let elapsed = entry.created_at.elapsed().unwrap_or(Duration::MAX);
            if elapsed.as_secs() < self.config.cache_ttl {
                entry.access_count.fetch_add(1, Ordering::Relaxed);

                // Try to deserialize
                if let Ok(result) = serde_json::from_slice(&entry.data) {
                    return Some(result);
                }
            }
        }

        None
    }

    /// Cache query result
    fn cache_query_result<T>(&self, key: &str, result: &T)
    where
        T: serde::Serialize,
    {
        if let Ok(data) = serde_json::to_vec(result) {
            let entry = QueryCacheEntry {
                data,
                created_at: SystemTime::now(),
                last_accessed: Instant::now(),
                access_count: AtomicU64::new(1),
            };

            let mut cache = self.query_cache.write();
            cache.put(key.to_string(), entry);
        }
    }

    /// Background maintenance task
    async fn run_background_maintenance(
        pool: Pool<SqliteConnectionManager>,
        metrics: Arc<DatabaseMetrics>,
        query_cache: Arc<RwLock<LruCache<String, QueryCacheEntry>>>,
        interval_secs: u64,
    ) {
        let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));

        loop {
            interval.tick().await;

            // Run maintenance tasks
            if let Ok(conn) = pool.get() {
                // Vacuum database
                let _ = conn.execute("PRAGMA optimize", []);

                // Update statistics
                let _ = conn.execute("ANALYZE", []);

                // Clean up old performance metrics
                let _ = conn.execute(
                    "DELETE FROM performance_metrics WHERE recorded_at < ?",
                    params![chrono::Utc::now().timestamp() - 7 * 24 * 3600] // Keep 7 days
                );

                metrics.maintenance_runs.fetch_add(1, Ordering::Relaxed);
            }

            // Clean up expired cache entries
            {
                let mut cache = query_cache.write();
                let now = SystemTime::now();

                // Collect keys of expired entries
                let expired_keys: Vec<String> = cache
                    .iter()
                    .filter_map(|(key, entry)| {
                        let elapsed = now.duration_since(entry.created_at).unwrap_or(Duration::MAX);
                        if elapsed.as_secs() > interval_secs * 2 {
                            Some(key.clone())
                        } else {
                            None
                        }
                    })
                    .collect();

                // Remove expired entries
                for key in expired_keys {
                    cache.pop(&key);
                }
            }
        }
    }

    /// Get performance metrics
    pub fn get_metrics(&self) -> DatabaseMetrics {
        DatabaseMetrics {
            total_queries: AtomicU64::new(self.metrics.total_queries.load(Ordering::Relaxed)),
            successful_queries: AtomicU64::new(self.metrics.successful_queries.load(Ordering::Relaxed)),
            failed_queries: AtomicU64::new(self.metrics.failed_queries.load(Ordering::Relaxed)),
            cache_hits: AtomicU64::new(self.metrics.cache_hits.load(Ordering::Relaxed)),
            cache_misses: AtomicU64::new(self.metrics.cache_misses.load(Ordering::Relaxed)),
            total_query_time_us: AtomicU64::new(self.metrics.total_query_time_us.load(Ordering::Relaxed)),
            connection_waits: AtomicU64::new(self.metrics.connection_waits.load(Ordering::Relaxed)),
            maintenance_runs: AtomicU64::new(self.metrics.maintenance_runs.load(Ordering::Relaxed)),
        }
    }

    /// Optimize database performance by analyzing and updating statistics
    pub async fn optimize_performance(&self) -> AssetResult<()> {
        let conn = self.pool.get()
            .map_err(|e| AssetError::LoadFailed(format!("Failed to get connection: {}", e)))?;

        // Run comprehensive optimization
        conn.execute_batch(
            "
            PRAGMA optimize;
            ANALYZE;
            PRAGMA wal_checkpoint(TRUNCATE);
            "
        ).map_err(|e| AssetError::LoadFailed(format!("Optimization failed: {}", e)))?;

        Ok(())
    }

    /// Shutdown the database gracefully
    pub async fn shutdown(self) {
        if let Some(handle) = self.maintenance_handle {
            handle.abort();
        }

        // Final optimization before shutdown
        if let Ok(conn) = self.pool.get() {
            let _ = conn.execute("PRAGMA optimize", []);
            let _ = conn.execute("PRAGMA wal_checkpoint(TRUNCATE)", []);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_database_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = OptimizedDatabaseConfig {
            database_path: temp_dir.path().join("test.db"),
            ..Default::default()
        };

        let db = OptimizedAssetDatabase::new(config).await;
        assert!(db.is_ok());
    }

    #[tokio::test]
    async fn test_bulk_insert() {
        let temp_dir = TempDir::new().unwrap();
        let config = OptimizedDatabaseConfig {
            database_path: temp_dir.path().join("test.db"),
            ..Default::default()
        };

        let db = OptimizedAssetDatabase::new(config).await.unwrap();

        let assets = vec![
            AssetMetadata::new("test1".to_string(), AssetType::Texture, PathBuf::from("test1.png")),
            AssetMetadata::new("test2".to_string(), AssetType::Audio, PathBuf::from("test2.wav")),
        ];

        let result = db.bulk_insert_assets(&assets).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_search_functionality() {
        let temp_dir = TempDir::new().unwrap();
        let config = OptimizedDatabaseConfig {
            database_path: temp_dir.path().join("test.db"),
            ..Default::default()
        };

        let db = OptimizedAssetDatabase::new(config).await.unwrap();

        // Insert test data
        let assets = vec![
            AssetMetadata::new("texture1".to_string(), AssetType::Texture, PathBuf::from("images/test1.png")),
            AssetMetadata::new("audio1".to_string(), AssetType::Audio, PathBuf::from("sounds/test1.wav")),
        ];

        db.bulk_insert_assets(&assets).await.unwrap();

        // Test search
        let results = db.search_assets(Some(AssetType::Texture), None, None, None).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].asset_type, AssetType::Texture);
    }

    #[tokio::test]
    async fn test_caching() {
        let temp_dir = TempDir::new().unwrap();
        let config = OptimizedDatabaseConfig {
            database_path: temp_dir.path().join("test.db"),
            cache_ttl: 60, // 1 minute
            ..Default::default()
        };

        let db = OptimizedAssetDatabase::new(config).await.unwrap();

        // First search (cache miss)
        let results1 = db.search_assets(Some(AssetType::Texture), None, None, None).await.unwrap();

        // Second search (should be cache hit)
        let results2 = db.search_assets(Some(AssetType::Texture), None, None, None).await.unwrap();

        let metrics = db.get_metrics();
        assert!(metrics.cache_hits.load(Ordering::Relaxed) > 0);
    }

    #[test]
    fn test_metrics() {
        let metrics = DatabaseMetrics::default();

        assert_eq!(metrics.cache_hit_rate(), 0.0);
        assert_eq!(metrics.success_rate(), 0.0);
        assert_eq!(metrics.average_query_time(), Duration::ZERO);

        metrics.cache_hits.store(8, Ordering::Relaxed);
        metrics.cache_misses.store(2, Ordering::Relaxed);
        assert_eq!(metrics.cache_hit_rate(), 0.8);

        metrics.successful_queries.store(19, Ordering::Relaxed);
        metrics.total_queries.store(20, Ordering::Relaxed);
        assert_eq!(metrics.success_rate(), 0.95);
    }
}