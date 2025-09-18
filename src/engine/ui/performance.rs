// Robin Game Engine - High-Performance UI Rendering System
// Virtual scrolling, component pooling, and efficient state management

use crate::engine::error::RobinResult;
use std::{
    collections::{HashMap, VecDeque, HashSet},
    sync::{Arc, Mutex, atomic::{AtomicUsize, AtomicU64, Ordering}},
    time::{Duration, Instant},
    any::{Any, TypeId},
};
use serde::{Serialize, Deserialize};

/// High-performance UI renderer with virtual scrolling and component pooling
#[derive(Debug)]
pub struct PerformantUIRenderer {
    config: UIPerformanceConfig,
    component_pool: ComponentPool,
    virtual_scrollers: HashMap<String, VirtualScrollManager>,
    state_manager: StateManager,
    render_cache: RenderCache,
    batch_renderer: BatchRenderer,
    performance_metrics: UIPerformanceMetrics,
    frame_limiter: FrameLimiter,
}

/// Configuration for UI performance optimizations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIPerformanceConfig {
    pub target_fps: f32,
    pub max_components_per_frame: usize,
    pub virtual_scroll_enabled: bool,
    pub component_pooling_enabled: bool,
    pub render_caching_enabled: bool,
    pub batch_rendering_enabled: bool,
    pub state_update_batching: bool,
    pub component_pool_initial_size: usize,
    pub component_pool_max_size: usize,
    pub cache_max_entries: usize,
    pub frame_time_budget_ms: f32,
}

impl Default for UIPerformanceConfig {
    fn default() -> Self {
        Self {
            target_fps: 60.0,
            max_components_per_frame: 1000,
            virtual_scroll_enabled: true,
            component_pooling_enabled: true,
            render_caching_enabled: true,
            batch_rendering_enabled: true,
            state_update_batching: true,
            component_pool_initial_size: 100,
            component_pool_max_size: 1000,
            cache_max_entries: 500,
            frame_time_budget_ms: 16.67, // 60 FPS
        }
    }
}

/// Performance metrics for UI rendering
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UIPerformanceMetrics {
    pub frame_time_ms: f32,
    pub render_time_ms: f32,
    pub layout_time_ms: f32,
    pub paint_time_ms: f32,
    pub components_rendered: u32,
    pub components_culled: u32,
    pub virtual_scroll_items: u32,
    pub pool_hits: u32,
    pub pool_misses: u32,
    pub cache_hits: u32,
    pub cache_misses: u32,
    pub state_updates_batched: u32,
    pub memory_usage_mb: f32,
    pub draw_calls_batched: u32,
}

/// Component pooling system for efficient memory management
#[derive(Debug)]
struct ComponentPool {
    pools: HashMap<TypeId, ComponentTypePool>,
    config: UIPerformanceConfig,
    stats: ComponentPoolStats,
}

#[derive(Debug)]
struct ComponentTypePool {
    available: VecDeque<Box<dyn UIComponent>>,
    in_use: HashSet<usize>,
    created_count: AtomicUsize,
    reused_count: AtomicUsize,
    max_size: usize,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct ComponentPoolStats {
    total_gets: AtomicU64,
    total_returns: AtomicU64,
    pool_hits: AtomicU64,
    pool_misses: AtomicU64,
    memory_saved_mb: f32,
}

impl Clone for ComponentPoolStats {
    fn clone(&self) -> Self {
        Self {
            total_gets: AtomicU64::new(self.total_gets.load(Ordering::Relaxed)),
            total_returns: AtomicU64::new(self.total_returns.load(Ordering::Relaxed)),
            pool_hits: AtomicU64::new(self.pool_hits.load(Ordering::Relaxed)),
            pool_misses: AtomicU64::new(self.pool_misses.load(Ordering::Relaxed)),
            memory_saved_mb: self.memory_saved_mb,
        }
    }
}

/// Virtual scrolling manager for large lists
#[derive(Debug)]
pub struct VirtualScrollManager {
    config: VirtualScrollConfig,
    viewport: Viewport,
    item_cache: ItemCache,
    visible_range: (usize, usize),
    scroll_position: f32,
    item_size_estimator: ItemSizeEstimator,
    performance_stats: VirtualScrollStats,
}

#[derive(Debug, Clone)]
pub struct VirtualScrollConfig {
    pub viewport_height: f32,
    pub estimated_item_height: f32,
    pub buffer_size: usize, // Extra items to render outside viewport
    pub dynamic_sizing: bool,
    pub smooth_scrolling: bool,
    pub preload_distance: f32,
}

impl Default for VirtualScrollConfig {
    fn default() -> Self {
        Self {
            viewport_height: 600.0,
            estimated_item_height: 40.0,
            buffer_size: 5,
            dynamic_sizing: true,
            smooth_scrolling: true,
            preload_distance: 200.0,
        }
    }
}

#[derive(Debug)]
struct Viewport {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[derive(Debug)]
struct ItemCache {
    cached_items: HashMap<usize, CachedItem>,
    max_cache_size: usize,
    lru_order: VecDeque<usize>,
}

#[derive(Debug)]
struct CachedItem {
    component: Box<dyn UIComponent>,
    height: f32,
    last_accessed: Instant,
}

#[derive(Debug)]
struct ItemSizeEstimator {
    measured_sizes: HashMap<usize, f32>,
    average_size: f32,
    total_measured: usize,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct VirtualScrollStats {
    items_rendered: u32,
    items_cached: u32,
    cache_hits: u32,
    cache_misses: u32,
    scroll_events: u32,
    layout_calculations: u32,
}

/// Efficient state management with batching
#[derive(Debug)]
struct StateManager {
    pending_updates: VecDeque<StateUpdate>,
    component_states: HashMap<String, Box<dyn Any + Send + Sync>>,
    dirty_components: HashSet<String>,
    batch_timer: Option<Instant>,
    batch_delay_ms: u64,
    update_stats: StateUpdateStats,
}

#[derive(Debug)]
struct StateUpdate {
    component_id: String,
    update_type: StateUpdateType,
    data: Box<dyn Any + Send + Sync>,
    priority: UpdatePriority,
    timestamp: Instant,
}

#[derive(Debug, Clone)]
enum StateUpdateType {
    Full,
    Partial,
    Incremental,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum UpdatePriority {
    Low,
    Normal,
    High,
    Immediate,
}

#[derive(Debug, Default)]
struct StateUpdateStats {
    updates_processed: AtomicU64,
    updates_batched: AtomicU64,
    batch_efficiency: f32,
    average_batch_size: f32,
}

/// Render caching system
#[derive(Debug)]
struct RenderCache {
    cached_renders: HashMap<String, CachedRender>,
    cache_stats: CacheStats,
    max_entries: usize,
    memory_usage: AtomicUsize,
}

#[derive(Debug, Clone)]
struct CachedRender {
    render_data: Vec<u8>, // Serialized render commands
    cache_key: String,
    timestamp: Instant,
    access_count: u32,
    memory_size: usize,
}

#[derive(Debug, Default)]
struct CacheStats {
    hits: AtomicU64,
    misses: AtomicU64,
    evictions: AtomicU64,
    memory_saved_mb: f32,
}

/// Batch renderer for reducing draw calls
#[derive(Debug)]
struct BatchRenderer {
    render_queue: Vec<RenderCommand>,
    current_batch: RenderBatch,
    batch_stats: BatchRenderStats,
    max_batch_size: usize,
}

#[derive(Debug, Clone)]
enum RenderCommand {
    DrawRect { x: f32, y: f32, width: f32, height: f32, color: [f32; 4] },
    DrawText { x: f32, y: f32, text: String, color: [f32; 4] },
    DrawTexture { x: f32, y: f32, width: f32, height: f32, texture_id: String },
}

#[derive(Debug, Default)]
struct RenderBatch {
    commands: Vec<RenderCommand>,
    texture_id: Option<String>,
    blend_mode: BlendMode,
}

#[derive(Debug, Clone, PartialEq)]
enum BlendMode {
    Normal,
    Alpha,
    Additive,
}

impl Default for BlendMode {
    fn default() -> Self {
        BlendMode::Normal
    }
}

#[derive(Debug, Default)]
struct BatchRenderStats {
    batches_created: AtomicU64,
    commands_batched: AtomicU64,
    draw_calls_saved: AtomicU64,
}

/// Frame rate limiting system
#[derive(Debug)]
struct FrameLimiter {
    target_frame_time: Duration,
    last_frame_time: Instant,
    frame_times: VecDeque<Duration>,
    adaptive_frame_pacing: bool,
}

/// UI component trait for virtual scrolling and pooling
pub trait UIComponent: std::fmt::Debug + Send + Sync {
    fn render(&self, context: &mut RenderContext) -> RobinResult<()>;
    fn get_size(&self) -> (f32, f32);
    fn set_position(&mut self, x: f32, y: f32);
    fn set_data(&mut self, data: Box<dyn Any + Send + Sync>);
    fn reset(&mut self); // For pooling
    fn get_cache_key(&self) -> String;
    fn is_dirty(&self) -> bool;
    fn mark_clean(&mut self);
    fn as_any(&self) -> &dyn Any;
}

/// Render context for components
#[derive(Debug)]
pub struct RenderContext {
    pub viewport: Viewport,
    pub renderer: *mut BatchRenderer, // Unsafe pointer for performance
    pub cache: *mut RenderCache,
    pub current_time: Instant,
}

impl PerformantUIRenderer {
    pub fn new(config: UIPerformanceConfig) -> RobinResult<Self> {
        Ok(Self {
            component_pool: ComponentPool::new(&config)?,
            virtual_scrollers: HashMap::new(),
            state_manager: StateManager::new(config.state_update_batching),
            render_cache: RenderCache::new(config.cache_max_entries),
            batch_renderer: BatchRenderer::new(1000), // Max 1000 commands per batch
            performance_metrics: UIPerformanceMetrics::default(),
            frame_limiter: FrameLimiter::new(config.target_fps),
            config,
        })
    }

    /// Render a frame with all performance optimizations
    pub fn render_frame(&mut self, components: &[Box<dyn UIComponent>]) -> RobinResult<()> {
        let frame_start = Instant::now();

        // Start frame timing
        self.frame_limiter.begin_frame();

        // Process batched state updates
        let layout_start = Instant::now();
        self.state_manager.process_pending_updates()?;
        let layout_time = layout_start.elapsed();

        // Begin render pass
        let render_start = Instant::now();

        // Clear render queue
        self.batch_renderer.clear();

        let mut render_context = RenderContext {
            viewport: Viewport {
                x: 0.0,
                y: 0.0,
                width: 1920.0,
                height: 1080.0,
            },
            renderer: &mut self.batch_renderer as *mut _,
            cache: &mut self.render_cache as *mut _,
            current_time: Instant::now(),
        };

        // Render components with culling and caching
        let mut components_rendered = 0;
        let mut components_culled = 0;

        for component in components {
            // Frustum culling
            if self.should_cull_component(component, &render_context.viewport) {
                components_culled += 1;
                continue;
            }

            // Check render cache
            let cache_key = component.get_cache_key();
            if self.config.render_caching_enabled && !component.is_dirty() {
                if self.render_cache.try_use_cached(&cache_key) {
                    self.performance_metrics.cache_hits += 1;
                    components_rendered += 1;
                    continue;
                }
            }

            // Render component
            component.render(&mut render_context)?;
            components_rendered += 1;

            // Cache the render if appropriate
            if self.config.render_caching_enabled && !component.is_dirty() {
                self.render_cache.cache_render(&cache_key, &component)?;
            }

            // Check frame time budget
            if frame_start.elapsed().as_secs_f32() * 1000.0 > self.config.frame_time_budget_ms {
                log::debug!("Frame time budget exceeded, deferring remaining components");
                break;
            }
        }

        let render_time = render_start.elapsed();

        // Execute batched render commands
        let paint_start = Instant::now();
        self.batch_renderer.execute_batches()?;
        let paint_time = paint_start.elapsed();

        // Update performance metrics
        let total_frame_time = frame_start.elapsed();
        self.performance_metrics.frame_time_ms = total_frame_time.as_secs_f32() * 1000.0;
        self.performance_metrics.layout_time_ms = layout_time.as_secs_f32() * 1000.0;
        self.performance_metrics.render_time_ms = render_time.as_secs_f32() * 1000.0;
        self.performance_metrics.paint_time_ms = paint_time.as_secs_f32() * 1000.0;
        self.performance_metrics.components_rendered = components_rendered;
        self.performance_metrics.components_culled = components_culled;

        // Update pool stats
        self.update_pool_metrics();

        // Update cache stats
        self.update_cache_metrics();

        // Frame limiting
        self.frame_limiter.end_frame();

        log::debug!("Frame rendered in {:.2}ms: {} components ({} culled)",
                   self.performance_metrics.frame_time_ms,
                   components_rendered,
                   components_culled);

        Ok(())
    }

    /// Create or get a virtual scroll manager for a list
    pub fn create_virtual_scroller(&mut self, id: String, config: VirtualScrollConfig) -> RobinResult<()> {
        let scroller = VirtualScrollManager::new(config)?;
        self.virtual_scrollers.insert(id, scroller);
        Ok(())
    }

    /// Update virtual scroll position and recalculate visible items
    pub fn update_virtual_scroll(&mut self, id: &str, scroll_position: f32, total_items: usize) -> RobinResult<()> {
        if let Some(scroller) = self.virtual_scrollers.get_mut(id) {
            scroller.update_scroll_position(scroll_position, total_items)?;

            // Update metrics
            self.performance_metrics.virtual_scroll_items = scroller.get_visible_count() as u32;
        }
        Ok(())
    }

    /// Get a component from the pool
    pub fn get_pooled_component<T: UIComponent + 'static>(&mut self) -> RobinResult<Box<T>> {
        self.component_pool.get_component::<T>()
    }

    /// Return a component to the pool
    pub fn return_pooled_component<T: UIComponent + 'static>(&mut self, mut component: Box<T>) -> RobinResult<()> {
        component.reset();
        self.component_pool.return_component(component)
    }

    /// Queue a state update for batching
    pub fn queue_state_update(&mut self, component_id: String, data: Box<dyn Any + Send + Sync>, priority: UpdatePriority) -> RobinResult<()> {
        self.state_manager.queue_update(StateUpdate {
            component_id,
            update_type: StateUpdateType::Full,
            data,
            priority,
            timestamp: Instant::now(),
        })
    }

    /// Get current performance metrics
    pub fn get_performance_metrics(&self) -> &UIPerformanceMetrics {
        &self.performance_metrics
    }

    /// Get detailed performance report
    pub fn get_performance_report(&self) -> UIPerformanceReport {
        UIPerformanceReport {
            metrics: self.performance_metrics.clone(),
            pool_stats: self.component_pool.get_stats().clone(),
            cache_efficiency: self.render_cache.get_efficiency(),
            batch_efficiency: self.batch_renderer.get_efficiency(),
            virtual_scroll_stats: self.get_virtual_scroll_stats(),
            memory_usage_mb: self.calculate_memory_usage(),
        }
    }

    // Private implementation methods

    fn should_cull_component(&self, component: &Box<dyn UIComponent>, viewport: &Viewport) -> bool {
        let (width, height) = component.get_size();

        // Simple AABB culling - would be more sophisticated in production
        let component_bounds = (0.0, 0.0, width, height); // Would get actual position

        component_bounds.2 < viewport.x ||
        component_bounds.0 > viewport.x + viewport.width ||
        component_bounds.3 < viewport.y ||
        component_bounds.1 > viewport.y + viewport.height
    }

    fn update_pool_metrics(&mut self) {
        let pool_stats = self.component_pool.get_stats();
        self.performance_metrics.pool_hits = pool_stats.pool_hits.load(Ordering::Relaxed) as u32;
        self.performance_metrics.pool_misses = pool_stats.pool_misses.load(Ordering::Relaxed) as u32;
    }

    fn update_cache_metrics(&mut self) {
        let cache_stats = self.render_cache.get_stats();
        self.performance_metrics.cache_hits = cache_stats.hits.load(Ordering::Relaxed) as u32;
        self.performance_metrics.cache_misses = cache_stats.misses.load(Ordering::Relaxed) as u32;
    }

    fn get_virtual_scroll_stats(&self) -> HashMap<String, VirtualScrollStats> {
        self.virtual_scrollers.iter()
            .map(|(id, scroller)| (id.clone(), scroller.performance_stats.clone()))
            .collect()
    }

    fn calculate_memory_usage(&self) -> f32 {
        // Calculate total memory usage across all systems
        let pool_memory = self.component_pool.get_memory_usage();
        let cache_memory = self.render_cache.memory_usage.load(Ordering::Relaxed) as f32 / (1024.0 * 1024.0);
        let state_memory = 16.0; // Estimate for state manager

        pool_memory + cache_memory + state_memory
    }
}

// Implementation details for supporting structures

impl ComponentPool {
    fn new(config: &UIPerformanceConfig) -> RobinResult<Self> {
        Ok(Self {
            pools: HashMap::new(),
            config: config.clone(),
            stats: ComponentPoolStats::default(),
        })
    }

    fn get_component<T: UIComponent + 'static>(&mut self) -> RobinResult<Box<T>> {
        let type_id = TypeId::of::<T>();

        if let Some(pool) = self.pools.get_mut(&type_id) {
            if let Some(component) = pool.available.pop_front() {
                pool.reused_count.fetch_add(1, Ordering::Relaxed);
                self.stats.pool_hits.fetch_add(1, Ordering::Relaxed);

                // Component type mismatch - this shouldn't happen in correct pool usage
                // For now, skip and create new component
            }
        }

        // Create new component since pool is empty
        self.stats.pool_misses.fetch_add(1, Ordering::Relaxed);

        // This would create a new T - simplified for the example
        Err(crate::engine::error::RobinError::new("Component creation not implemented"))
    }

    fn return_component<T: UIComponent + 'static>(&mut self, component: Box<T>) -> RobinResult<()> {
        let type_id = TypeId::of::<T>();

        let pool = self.pools.entry(type_id).or_insert_with(|| ComponentTypePool {
            available: VecDeque::new(),
            in_use: HashSet::new(),
            created_count: AtomicUsize::new(0),
            reused_count: AtomicUsize::new(0),
            max_size: self.config.component_pool_max_size,
        });

        if pool.available.len() < pool.max_size {
            // Store component as trait object
            pool.available.push_back(component as Box<dyn UIComponent>);
        }

        Ok(())
    }

    fn get_stats(&self) -> &ComponentPoolStats {
        &self.stats
    }

    fn get_memory_usage(&self) -> f32 {
        // Estimate memory usage of all pools
        let total_components: usize = self.pools.values()
            .map(|pool| pool.available.len())
            .sum();

        // Rough estimate: 1KB per component
        (total_components * 1024) as f32 / (1024.0 * 1024.0)
    }
}

impl VirtualScrollManager {
    fn new(config: VirtualScrollConfig) -> RobinResult<Self> {
        Ok(Self {
            viewport: Viewport {
                x: 0.0,
                y: 0.0,
                width: 800.0,
                height: config.viewport_height,
            },
            item_cache: ItemCache {
                cached_items: HashMap::new(),
                max_cache_size: 100,
                lru_order: VecDeque::new(),
            },
            visible_range: (0, 0),
            scroll_position: 0.0,
            item_size_estimator: ItemSizeEstimator {
                measured_sizes: HashMap::new(),
                average_size: config.estimated_item_height,
                total_measured: 0,
            },
            performance_stats: VirtualScrollStats::default(),
            config,
        })
    }

    fn update_scroll_position(&mut self, scroll_position: f32, total_items: usize) -> RobinResult<()> {
        self.scroll_position = scroll_position;

        // Calculate visible range
        let viewport_height = self.config.viewport_height;
        let estimated_item_height = self.item_size_estimator.average_size;

        let start_index = ((scroll_position / estimated_item_height) as usize)
            .saturating_sub(self.config.buffer_size);

        let visible_count = ((viewport_height / estimated_item_height) as usize)
            + (2 * self.config.buffer_size);

        let end_index = (start_index + visible_count).min(total_items);

        self.visible_range = (start_index, end_index);
        self.performance_stats.layout_calculations += 1;

        Ok(())
    }

    fn get_visible_count(&self) -> usize {
        self.visible_range.1.saturating_sub(self.visible_range.0)
    }
}

impl StateManager {
    fn new(batching_enabled: bool) -> Self {
        Self {
            pending_updates: VecDeque::new(),
            component_states: HashMap::new(),
            dirty_components: HashSet::new(),
            batch_timer: None,
            batch_delay_ms: if batching_enabled { 16 } else { 0 }, // One frame delay
            update_stats: StateUpdateStats::default(),
        }
    }

    fn queue_update(&mut self, update: StateUpdate) -> RobinResult<()> {
        // Mark component as dirty
        self.dirty_components.insert(update.component_id.clone());

        // Handle immediate updates
        if update.priority == UpdatePriority::Immediate {
            self.apply_update(update)?;
            return Ok(());
        }

        // Queue for batching
        self.pending_updates.push_back(update);

        // Start batch timer if not already started
        if self.batch_timer.is_none() && self.batch_delay_ms > 0 {
            self.batch_timer = Some(Instant::now());
        }

        Ok(())
    }

    fn process_pending_updates(&mut self) -> RobinResult<()> {
        // Check if batch delay has elapsed
        if let Some(timer) = self.batch_timer {
            if timer.elapsed().as_millis() < self.batch_delay_ms as u128 {
                return Ok(()); // Not time to process yet
            }
        }

        let batch_size = self.pending_updates.len();

        // Process all pending updates
        while let Some(update) = self.pending_updates.pop_front() {
            self.apply_update(update)?;
        }

        if batch_size > 0 {
            self.update_stats.updates_batched.fetch_add(batch_size as u64, Ordering::Relaxed);
            self.update_stats.average_batch_size =
                (self.update_stats.average_batch_size + batch_size as f32) / 2.0;
        }

        // Clear batch timer
        self.batch_timer = None;

        Ok(())
    }

    fn apply_update(&mut self, update: StateUpdate) -> RobinResult<()> {
        // Store component state
        self.component_states.insert(update.component_id.clone(), update.data);
        self.update_stats.updates_processed.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}

impl RenderCache {
    fn new(max_entries: usize) -> Self {
        Self {
            cached_renders: HashMap::with_capacity(max_entries),
            cache_stats: CacheStats::default(),
            max_entries,
            memory_usage: AtomicUsize::new(0),
        }
    }

    fn try_use_cached(&mut self, cache_key: &str) -> bool {
        if let Some(cached) = self.cached_renders.get_mut(cache_key) {
            cached.access_count += 1;
            self.cache_stats.hits.fetch_add(1, Ordering::Relaxed);
            true
        } else {
            self.cache_stats.misses.fetch_add(1, Ordering::Relaxed);
            false
        }
    }

    fn cache_render(&mut self, cache_key: &str, component: &Box<dyn UIComponent>) -> RobinResult<()> {
        // Simple caching - in production would serialize actual render data
        let render_data = vec![0u8; 1024]; // Placeholder
        let memory_size = render_data.len();

        // Evict if at capacity
        if self.cached_renders.len() >= self.max_entries {
            self.evict_lru_entry();
        }

        let cached_render = CachedRender {
            render_data,
            cache_key: cache_key.to_string(),
            timestamp: Instant::now(),
            access_count: 1,
            memory_size,
        };

        self.memory_usage.fetch_add(memory_size, Ordering::Relaxed);
        self.cached_renders.insert(cache_key.to_string(), cached_render);

        Ok(())
    }

    fn evict_lru_entry(&mut self) {
        // Find least recently used entry
        let mut lru_key: Option<String> = None;
        let mut oldest_time = Instant::now();

        for (key, cached) in &self.cached_renders {
            if cached.timestamp < oldest_time {
                oldest_time = cached.timestamp;
                lru_key = Some(key.clone());
            }
        }

        if let Some(key) = lru_key {
            if let Some(evicted) = self.cached_renders.remove(&key) {
                self.memory_usage.fetch_sub(evicted.memory_size, Ordering::Relaxed);
                self.cache_stats.evictions.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    fn get_stats(&self) -> &CacheStats {
        &self.cache_stats
    }

    fn get_efficiency(&self) -> f32 {
        let hits = self.cache_stats.hits.load(Ordering::Relaxed) as f32;
        let misses = self.cache_stats.misses.load(Ordering::Relaxed) as f32;

        if hits + misses > 0.0 {
            hits / (hits + misses)
        } else {
            0.0
        }
    }
}

impl BatchRenderer {
    fn new(max_batch_size: usize) -> Self {
        Self {
            render_queue: Vec::with_capacity(max_batch_size),
            current_batch: RenderBatch::default(),
            batch_stats: BatchRenderStats::default(),
            max_batch_size,
        }
    }

    fn clear(&mut self) {
        self.render_queue.clear();
        self.current_batch = RenderBatch::default();
    }

    fn execute_batches(&mut self) -> RobinResult<()> {
        // Group commands by batch compatibility
        let mut batches = Vec::new();
        let mut current_batch = RenderBatch::default();

        for command in &self.render_queue {
            if self.can_batch_command(&current_batch, command) {
                current_batch.commands.push(command.clone());
            } else {
                if !current_batch.commands.is_empty() {
                    batches.push(current_batch);
                }
                current_batch = RenderBatch::default();
                current_batch.commands.push(command.clone());
            }

            if current_batch.commands.len() >= self.max_batch_size {
                batches.push(current_batch);
                current_batch = RenderBatch::default();
            }
        }

        if !current_batch.commands.is_empty() {
            batches.push(current_batch);
        }

        // Execute batches
        for batch in batches {
            self.execute_batch(&batch)?;
            self.batch_stats.batches_created.fetch_add(1, Ordering::Relaxed);
            self.batch_stats.commands_batched.fetch_add(batch.commands.len() as u64, Ordering::Relaxed);
        }

        let draw_calls_saved = self.render_queue.len().saturating_sub(1);
        self.batch_stats.draw_calls_saved.fetch_add(draw_calls_saved as u64, Ordering::Relaxed);

        Ok(())
    }

    fn can_batch_command(&self, batch: &RenderBatch, command: &RenderCommand) -> bool {
        // Simple batching logic - in production would be more sophisticated
        match command {
            RenderCommand::DrawTexture { texture_id, .. } => {
                batch.texture_id.as_ref().map_or(true, |id| id == texture_id)
            }
            _ => true, // Other commands can always be batched
        }
    }

    fn execute_batch(&self, batch: &RenderBatch) -> RobinResult<()> {
        // Execute the actual rendering commands
        // This would interface with the graphics API
        log::debug!("Executing batch with {} commands", batch.commands.len());
        Ok(())
    }

    fn get_efficiency(&self) -> f32 {
        let commands = self.batch_stats.commands_batched.load(Ordering::Relaxed) as f32;
        let batches = self.batch_stats.batches_created.load(Ordering::Relaxed) as f32;

        if batches > 0.0 {
            commands / batches
        } else {
            0.0
        }
    }
}

impl FrameLimiter {
    fn new(target_fps: f32) -> Self {
        Self {
            target_frame_time: Duration::from_secs_f32(1.0 / target_fps),
            last_frame_time: Instant::now(),
            frame_times: VecDeque::with_capacity(60), // Keep last 60 frames
            adaptive_frame_pacing: true,
        }
    }

    fn begin_frame(&mut self) {
        let now = Instant::now();
        let frame_time = now.duration_since(self.last_frame_time);

        self.frame_times.push_back(frame_time);
        if self.frame_times.len() > 60 {
            self.frame_times.pop_front();
        }

        self.last_frame_time = now;
    }

    fn end_frame(&mut self) {
        if self.adaptive_frame_pacing {
            let elapsed = self.last_frame_time.elapsed();
            if elapsed < self.target_frame_time {
                let sleep_time = self.target_frame_time - elapsed;
                std::thread::sleep(sleep_time);
            }
        }
    }
}

/// Complete UI performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIPerformanceReport {
    pub metrics: UIPerformanceMetrics,
    pub pool_stats: ComponentPoolStats,
    pub cache_efficiency: f32,
    pub batch_efficiency: f32,
    pub virtual_scroll_stats: HashMap<String, VirtualScrollStats>,
    pub memory_usage_mb: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_renderer_creation() {
        let config = UIPerformanceConfig::default();
        let renderer = PerformantUIRenderer::new(config);
        assert!(renderer.is_ok());
    }

    #[test]
    fn test_virtual_scroll_config() {
        let config = VirtualScrollConfig::default();
        assert_eq!(config.viewport_height, 600.0);
        assert_eq!(config.buffer_size, 5);
    }

    #[test]
    fn test_frame_limiter() {
        let mut limiter = FrameLimiter::new(60.0);
        limiter.begin_frame();
        assert_eq!(limiter.frame_times.len(), 1);
    }
}