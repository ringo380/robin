use crate::engine::error::RobinResult;
use tokio::sync::{mpsc, oneshot, Mutex as TokioMutex};
use std::collections::{HashMap, BinaryHeap, VecDeque};
use std::sync::{Arc, atomic::{AtomicU64, AtomicBool, Ordering}};
use std::time::{Duration, Instant};
use std::cmp::Ordering as CmpOrdering;
use std::future::Future;
use std::pin::Pin;
use serde::{Serialize, Deserialize};

// Type aliases for performance system compatibility
pub use TaskConfig as ProcessorConfig;
pub use Task as BackgroundTask;
pub use BackgroundStats as ProcessorMetrics;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    Critical = 0,    // Must complete immediately
    High = 1,        // Important, complete ASAP
    Normal = 2,      // Standard priority
    Low = 3,         // Background work
    Idle = 4,        // Only when nothing else to do
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConfig {
    pub enabled: bool,
    pub max_concurrent_tasks: usize,
    pub max_queue_size: usize,
    pub task_queue_size: usize,
    pub worker_thread_count: usize,
    pub max_worker_threads: usize,
    pub task_timeout_seconds: u64,
    pub enable_task_stealing: bool,
    pub enable_priority_inheritance: bool,
    pub gc_interval_seconds: u64,
    pub metrics_collection: bool,
}

impl Default for TaskConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_concurrent_tasks: 16,
            max_queue_size: 1000,
            task_queue_size: 1000,
            worker_thread_count: num_cpus::get(),
            max_worker_threads: num_cpus::get(),
            task_timeout_seconds: 300, // 5 minutes
            enable_task_stealing: true,
            enable_priority_inheritance: true,
            gc_interval_seconds: 60,
            metrics_collection: true,
        }
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum TaskType {
    WorldGeneration,
    AssetLoading,
    NetworkSync,
    AudioProcessing,
    PhysicsSimulation,
    AIComputation,
    Compression,
    Decompression,
    GarbageCollection,
    FileIO,
    DatabaseQuery,
    Custom(String),
}

pub type TaskResult = Result<Vec<u8>, String>;
pub type TaskFuture = Pin<Box<dyn Future<Output = TaskResult> + Send + 'static>>;

#[derive(Debug, Clone)]
pub struct Task {
    pub id: String,
    pub task_type: TaskType,
    pub priority: TaskPriority,
    pub created_at: Instant,
    pub estimated_duration: Duration,
    pub dependencies: Vec<String>,
    pub max_retries: u32,
    pub current_retries: u32,
    pub timeout: Duration,
    pub metadata: HashMap<String, String>,
}

impl Task {
    pub fn new(id: String, task_type: TaskType, priority: TaskPriority) -> Self {
        Self {
            id,
            task_type,
            priority,
            created_at: Instant::now(),
            estimated_duration: Duration::from_secs(10),
            dependencies: Vec::new(),
            max_retries: 3,
            current_retries: 0,
            timeout: Duration::from_secs(300),
            metadata: HashMap::new(),
        }
    }

    pub fn with_dependencies(mut self, dependencies: Vec<String>) -> Self {
        self.dependencies = dependencies;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Task {}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<CmpOrdering> {
        Some(self.cmp(other))
    }
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> CmpOrdering {
        self.priority.cmp(&other.priority)
            .then_with(|| other.created_at.cmp(&self.created_at))
    }
}

#[derive(Debug, Clone)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
    Cancelled,
    Timeout,
}

#[derive(Debug, Clone)]
pub struct TaskExecution {
    pub task: Task,
    pub status: TaskStatus,
    pub started_at: Option<Instant>,
    pub completed_at: Option<Instant>,
    pub result: Option<TaskResult>,
    pub worker_id: Option<usize>,
}

#[derive(Debug, Default, Clone)]
pub struct BackgroundStats {
    pub tasks_queued: u64,
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub tasks_cancelled: u64,
    pub tasks_timed_out: u64,
    pub total_execution_time_ms: u64,
    pub average_execution_time_ms: f64,
    pub queue_size: usize,
    pub active_workers: usize,
    pub peak_queue_size: usize,
    pub tasks_by_type: HashMap<String, u64>,
    pub tasks_by_priority: HashMap<String, u64>,
}

#[derive(Debug)]
pub struct WorkerPool {
    workers: Vec<Worker>,
    task_sender: mpsc::UnboundedSender<TaskExecution>,
    result_receiver: Arc<TokioMutex<mpsc::UnboundedReceiver<TaskExecution>>>,
    shutdown_flag: Arc<AtomicBool>,
}

#[derive(Debug)]
struct Worker {
    id: usize,
    handle: tokio::task::JoinHandle<()>,
    active_task: Arc<TokioMutex<Option<String>>>,
}

pub struct BackgroundProcessor {
    config: TaskConfig,
    task_queue: BinaryHeap<Task>,
    pending_tasks: HashMap<String, Task>,
    running_tasks: HashMap<String, TaskExecution>,
    completed_tasks: VecDeque<TaskExecution>,
    worker_pool: Option<WorkerPool>,
    stats: BackgroundStats,
    task_counter: AtomicU64,
    dependency_graph: HashMap<String, Vec<String>>,
    task_handlers: HashMap<TaskType, Box<dyn Fn(&Task) -> TaskFuture + Send + Sync>>,
    completion_callbacks: HashMap<String, Box<dyn Fn(TaskResult) + Send + Sync>>,
}

impl std::fmt::Debug for BackgroundProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BackgroundProcessor")
            .field("config", &self.config)
            .field("task_queue", &self.task_queue)
            .field("pending_tasks", &self.pending_tasks)
            .field("running_tasks", &self.running_tasks)
            .field("completed_tasks", &self.completed_tasks)
            .field("worker_pool", &self.worker_pool)
            .field("stats", &self.stats)
            .field("task_counter", &self.task_counter)
            .field("dependency_graph", &self.dependency_graph)
            .field("task_handlers", &format!("<{} handlers>", self.task_handlers.len()))
            .field("completion_callbacks", &format!("<{} callbacks>", self.completion_callbacks.len()))
            .finish()
    }
}

impl BackgroundProcessor {
    pub fn new(config: TaskConfig) -> RobinResult<Self> {
        Ok(Self {
            config,
            task_queue: BinaryHeap::new(),
            pending_tasks: HashMap::new(),
            running_tasks: HashMap::new(),
            completed_tasks: VecDeque::new(),
            worker_pool: None,
            stats: BackgroundStats::default(),
            task_counter: AtomicU64::new(0),
            dependency_graph: HashMap::new(),
            task_handlers: HashMap::new(),
            completion_callbacks: HashMap::new(),
        })
    }

    pub fn initialize(&mut self) -> RobinResult<()> {
        println!("Background Processor initialized:");
        println!("  Max Concurrent Tasks: {}", self.config.max_concurrent_tasks);
        println!("  Worker Threads: {}", self.config.worker_thread_count);
        println!("  Max Queue Size: {}", self.config.max_queue_size);
        println!("  Task Stealing: {}", self.config.enable_task_stealing);
        Ok(())
    }

    pub async fn start(&mut self) -> RobinResult<()> {
        if self.worker_pool.is_some() {
            return Ok(()); // Already started
        }

        let (task_sender, task_receiver) = mpsc::unbounded_channel();
        let (result_sender, result_receiver) = mpsc::unbounded_channel();
        let shutdown_flag = Arc::new(AtomicBool::new(false));
        
        // Wrap receiver in Arc<Mutex> for sharing among workers
        let shared_receiver = Arc::new(TokioMutex::new(task_receiver));
        
        let mut workers = Vec::new();
        
        // Create worker threads
        for worker_id in 0..self.config.worker_thread_count {
            let worker_receiver = shared_receiver.clone();
            let worker_result_sender = result_sender.clone();
            let worker_shutdown = shutdown_flag.clone();
            let active_task = Arc::new(TokioMutex::new(None::<String>));
            let worker_active_task = active_task.clone();
            
            let handle = tokio::spawn(async move {
                Self::worker_loop(
                    worker_id,
                    worker_receiver,
                    worker_result_sender,
                    worker_shutdown,
                    worker_active_task,
                ).await;
            });

            workers.push(Worker {
                id: worker_id,
                handle,
                active_task,
            });
        }

        self.worker_pool = Some(WorkerPool {
            workers,
            task_sender,
            result_receiver: Arc::new(TokioMutex::new(result_receiver)),
            shutdown_flag,
        });

        self.register_default_handlers();
        
        println!("Background processor started with {} workers", self.config.worker_thread_count);
        Ok(())
    }

    pub async fn update(&mut self, _delta_time: f32) -> RobinResult<()> {
        if self.worker_pool.is_none() {
            return Ok(());
        }

        // Process completed tasks
        self.process_completed_tasks().await?;
        
        // Schedule ready tasks
        self.schedule_ready_tasks().await?;
        
        // Update statistics
        self.update_statistics();
        
        // Periodic cleanup
        if self.completed_tasks.len() > 1000 {
            self.cleanup_completed_tasks();
        }

        Ok(())
    }

    pub fn submit_task(&mut self, task: Task) -> RobinResult<String> {
        if self.pending_tasks.len() >= self.config.max_queue_size {
            return Err(crate::engine::error::RobinError::new("Task queue full"));
        }

        let task_id = task.id.clone();
        
        // Update dependency graph
        for dep in &task.dependencies {
            self.dependency_graph
                .entry(dep.clone())
                .or_insert_with(Vec::new)
                .push(task_id.clone());
        }
        
        // Check if dependencies are satisfied
        if self.dependencies_satisfied(&task) {
            self.task_queue.push(task);
        } else {
            self.pending_tasks.insert(task_id.clone(), task);
        }
        
        self.stats.tasks_queued += 1;
        self.stats.queue_size = self.task_queue.len() + self.pending_tasks.len();
        self.stats.peak_queue_size = self.stats.peak_queue_size.max(self.stats.queue_size);
        
        // Track by type and priority
        let task_type = format!("{:?}", self.pending_tasks.get(&task_id).unwrap_or(&Task::new("temp".to_string(), TaskType::Custom("temp".to_string()), TaskPriority::Low)).task_type);
        let priority = format!("{:?}", self.pending_tasks.get(&task_id).unwrap_or(&Task::new("temp".to_string(), TaskType::Custom("temp".to_string()), TaskPriority::Low)).priority);
        
        *self.stats.tasks_by_type.entry(task_type).or_insert(0) += 1;
        *self.stats.tasks_by_priority.entry(priority).or_insert(0) += 1;
        
        Ok(task_id)
    }

    pub fn cancel_task(&mut self, task_id: &str) -> RobinResult<bool> {
        // Remove from pending tasks
        if self.pending_tasks.remove(task_id).is_some() {
            self.stats.tasks_cancelled += 1;
            return Ok(true);
        }
        
        // Cancel running task (if possible)
        if let Some(mut execution) = self.running_tasks.remove(task_id) {
            execution.status = TaskStatus::Cancelled;
            self.completed_tasks.push_back(execution);
            self.stats.tasks_cancelled += 1;
            return Ok(true);
        }
        
        Ok(false)
    }

    pub fn get_task_status(&self, task_id: &str) -> Option<&TaskStatus> {
        if let Some(execution) = self.running_tasks.get(task_id) {
            Some(&execution.status)
        } else {
            self.completed_tasks.iter()
                .find(|e| e.task.id == task_id)
                .map(|e| &e.status)
        }
    }

    pub fn register_task_handler<F>(&mut self, task_type: TaskType, handler: F) 
    where 
        F: Fn(&Task) -> TaskFuture + Send + Sync + 'static,
    {
        self.task_handlers.insert(task_type, Box::new(handler));
    }

    pub fn register_completion_callback<F>(&mut self, task_id: String, callback: F)
    where
        F: Fn(TaskResult) + Send + Sync + 'static,
    {
        self.completion_callbacks.insert(task_id, Box::new(callback));
    }

    async fn process_completed_tasks(&mut self) -> RobinResult<()> {
        if let Some(ref worker_pool) = self.worker_pool {
            let mut receiver = worker_pool.result_receiver.lock().await;
            
            while let Ok(mut execution) = receiver.try_recv() {
                let task_id = execution.task.id.clone();
                
                // Update statistics
                match execution.status {
                    TaskStatus::Completed => {
                        self.stats.tasks_completed += 1;
                        if let (Some(start), Some(end)) = (execution.started_at, execution.completed_at) {
                            let duration_ms = end.duration_since(start).as_millis() as u64;
                            self.stats.total_execution_time_ms += duration_ms;
                            self.stats.average_execution_time_ms = 
                                self.stats.total_execution_time_ms as f64 / self.stats.tasks_completed as f64;
                        }
                    },
                    TaskStatus::Failed(_) => self.stats.tasks_failed += 1,
                    TaskStatus::Timeout => self.stats.tasks_timed_out += 1,
                    _ => {},
                }
                
                // Call completion callback if registered
                if let Some(callback) = self.completion_callbacks.remove(&task_id) {
                    if let Some(ref result) = execution.result {
                        callback(result.clone());
                    }
                }
                
                // Check for dependent tasks
                if let Some(dependents) = self.dependency_graph.remove(&task_id) {
                    for dependent_id in dependents {
                        if let Some(task) = self.pending_tasks.remove(&dependent_id) {
                            if self.dependencies_satisfied(&task) {
                                self.task_queue.push(task);
                            } else {
                                self.pending_tasks.insert(dependent_id, task);
                            }
                        }
                    }
                }
                
                self.running_tasks.remove(&task_id);
                self.completed_tasks.push_back(execution);
            }
        }
        
        Ok(())
    }

    async fn schedule_ready_tasks(&mut self) -> RobinResult<()> {
        if let Some(ref worker_pool) = self.worker_pool {
            let max_concurrent = self.config.max_concurrent_tasks;
            let current_running = self.running_tasks.len();
            
            let available_slots = max_concurrent.saturating_sub(current_running);
            let mut scheduled = 0;
            
            while scheduled < available_slots && !self.task_queue.is_empty() {
                if let Some(task) = self.task_queue.pop() {
                    let mut execution = TaskExecution {
                        task,
                        status: TaskStatus::Pending,
                        started_at: None,
                        completed_at: None,
                        result: None,
                        worker_id: None,
                    };
                    
                    let task_id = execution.task.id.clone();
                    execution.status = TaskStatus::Running;
                    execution.started_at = Some(Instant::now());
                    
                    if worker_pool.task_sender.send(execution.clone()).is_ok() {
                        self.running_tasks.insert(task_id, execution);
                        scheduled += 1;
                    } else {
                        // Worker pool is full, put task back
                        self.task_queue.push(execution.task);
                        break;
                    }
                }
            }
            
            self.stats.active_workers = self.running_tasks.len();
            self.stats.queue_size = self.task_queue.len() + self.pending_tasks.len();
        }
        
        Ok(())
    }

    fn dependencies_satisfied(&self, task: &Task) -> bool {
        task.dependencies.iter().all(|dep_id| {
            self.completed_tasks.iter()
                .any(|e| e.task.id == *dep_id && matches!(e.status, TaskStatus::Completed))
        })
    }

    fn update_statistics(&mut self) {
        self.stats.queue_size = self.task_queue.len() + self.pending_tasks.len();
        self.stats.active_workers = self.running_tasks.len();
    }

    fn cleanup_completed_tasks(&mut self) {
        // Keep only the most recent 500 completed tasks
        while self.completed_tasks.len() > 500 {
            self.completed_tasks.pop_front();
        }
    }

    fn register_default_handlers(&mut self) {
        // World Generation Handler
        self.register_task_handler(TaskType::WorldGeneration, |_task| {
            Box::pin(async {
                tokio::time::sleep(Duration::from_millis(100)).await;
                Ok(b"world_chunk_generated".to_vec())
            })
        });

        // Asset Loading Handler
        self.register_task_handler(TaskType::AssetLoading, |_task| {
            Box::pin(async {
                tokio::time::sleep(Duration::from_millis(50)).await;
                Ok(b"asset_loaded".to_vec())
            })
        });

        // Physics Simulation Handler
        self.register_task_handler(TaskType::PhysicsSimulation, |_task| {
            Box::pin(async {
                tokio::time::sleep(Duration::from_millis(16)).await; // ~60 FPS
                Ok(b"physics_step_complete".to_vec())
            })
        });

        // AI Computation Handler
        self.register_task_handler(TaskType::AIComputation, |_task| {
            Box::pin(async {
                tokio::time::sleep(Duration::from_millis(200)).await;
                Ok(b"ai_decision_computed".to_vec())
            })
        });

        // Compression Handler
        self.register_task_handler(TaskType::Compression, |_task| {
            Box::pin(async {
                tokio::time::sleep(Duration::from_millis(80)).await;
                Ok(b"data_compressed".to_vec())
            })
        });

        // File I/O Handler
        self.register_task_handler(TaskType::FileIO, |_task| {
            Box::pin(async {
                tokio::time::sleep(Duration::from_millis(30)).await;
                Ok(b"file_operation_complete".to_vec())
            })
        });
    }

    async fn worker_loop(
        worker_id: usize,
        task_receiver: Arc<TokioMutex<mpsc::UnboundedReceiver<TaskExecution>>>,
        result_sender: mpsc::UnboundedSender<TaskExecution>,
        shutdown_flag: Arc<AtomicBool>,
        active_task: Arc<TokioMutex<Option<String>>>,
    ) {
        println!("Worker {} started", worker_id);
        
        while !shutdown_flag.load(Ordering::Relaxed) {
            let task_result = {
                let mut receiver_guard = task_receiver.lock().await;
                receiver_guard.recv().await
            };
            match task_result {
                Some(mut execution) => {
                    execution.worker_id = Some(worker_id);
                    let task_id = execution.task.id.clone();
                    
                    // Set active task
                    {
                        let mut active = active_task.lock().await;
                        *active = Some(task_id.clone());
                    }
                    
                    // Execute the task with timeout
                    let result = tokio::time::timeout(
                        execution.task.timeout,
                        Self::execute_task(&execution.task)
                    ).await;
                    
                    execution.completed_at = Some(Instant::now());
                    
                    match result {
                        Ok(task_result) => {
                            execution.result = Some(task_result.clone());
                            execution.status = match task_result {
                                Ok(_) => TaskStatus::Completed,
                                Err(e) => TaskStatus::Failed(e),
                            };
                        },
                        Err(_) => {
                            execution.status = TaskStatus::Timeout;
                            execution.result = Some(Err("Task timed out".to_string()));
                        }
                    }
                    
                    // Clear active task
                    {
                        let mut active = active_task.lock().await;
                        *active = None;
                    }
                    
                    // Send result back
                    if result_sender.send(execution).is_err() {
                        eprintln!("Worker {} failed to send result", worker_id);
                        break;
                    }
                },
                None => break, // Channel closed
            }
        }
        
        println!("Worker {} shutting down", worker_id);
    }

    async fn execute_task(task: &Task) -> TaskResult {
        // Placeholder task execution - in a real implementation, this would
        // dispatch to the appropriate handler based on task type
        match task.task_type {
            TaskType::WorldGeneration => {
                tokio::time::sleep(Duration::from_millis(100)).await;
                Ok(format!("Generated world chunk for task {}", task.id).into_bytes())
            },
            TaskType::AssetLoading => {
                tokio::time::sleep(Duration::from_millis(50)).await;
                Ok(format!("Loaded asset for task {}", task.id).into_bytes())
            },
            TaskType::PhysicsSimulation => {
                tokio::time::sleep(Duration::from_millis(16)).await;
                Ok(format!("Physics step completed for task {}", task.id).into_bytes())
            },
            TaskType::AIComputation => {
                tokio::time::sleep(Duration::from_millis(200)).await;
                Ok(format!("AI computation completed for task {}", task.id).into_bytes())
            },
            _ => {
                tokio::time::sleep(Duration::from_millis(10)).await;
                Ok(format!("Generic task {} completed", task.id).into_bytes())
            }
        }
    }

    pub fn get_stats(&self) -> &BackgroundStats {
        &self.stats
    }

    pub fn get_active_task_count(&self) -> usize {
        self.running_tasks.len()
    }

    pub fn get_queue_size(&self) -> usize {
        self.task_queue.len() + self.pending_tasks.len()
    }

    pub fn get_completion_rate(&self) -> f64 {
        let total = self.stats.tasks_completed + self.stats.tasks_failed + self.stats.tasks_cancelled;
        if total > 0 {
            self.stats.tasks_completed as f64 / total as f64
        } else {
            0.0
        }
    }

    // Additional methods for performance system compatibility
    pub fn get_pending_task_count(&self) -> u32 {
        self.get_queue_size() as u32
    }

    pub fn get_metrics(&self) -> &BackgroundStats {
        &self.stats
    }

    pub fn enable(&mut self) {
        // Background processor is enabled by default when initialized
        // This method is for consistency with other systems
    }

    pub fn disable(&mut self) {
        // Note: This is a simplified disable that just clears the queue
        // In a real implementation, you might want to pause workers
        self.task_queue.clear();
        self.pending_tasks.clear();
    }

    pub fn is_enabled(&self) -> bool {
        self.worker_pool.is_some()
    }

    // Synchronous update method for compatibility with non-async performance manager
    pub fn update_sync(&mut self, _delta_time: f32) -> RobinResult<()> {
        // For sync compatibility, we provide a minimal update
        // The real work happens in the async update method
        self.update_statistics();
        if self.completed_tasks.len() > 1000 {
            self.cleanup_completed_tasks();
        }
        Ok(())
    }

    pub fn shutdown_sync(&mut self) -> RobinResult<()> {
        if let Some(_worker_pool) = self.worker_pool.take() {
            println!("Background processor shutdown:");
            println!("  Tasks completed: {}", self.stats.tasks_completed);
            println!("  Tasks failed: {}", self.stats.tasks_failed);
            println!("  Average execution time: {:.1}ms", self.stats.average_execution_time_ms);
            println!("  Peak queue size: {}", self.stats.peak_queue_size);

            // Clear all data structures
            self.task_queue.clear();
            self.pending_tasks.clear();
            self.running_tasks.clear();
            self.completed_tasks.clear();
            self.dependency_graph.clear();
            self.task_handlers.clear();
            self.completion_callbacks.clear();
        }

        Ok(())
    }

    pub async fn shutdown(&mut self) -> RobinResult<()> {
        if let Some(worker_pool) = self.worker_pool.take() {
            println!("Background processor shutdown:");
            println!("  Tasks completed: {}", self.stats.tasks_completed);
            println!("  Tasks failed: {}", self.stats.tasks_failed);
            println!("  Average execution time: {:.1}ms", self.stats.average_execution_time_ms);
            println!("  Peak queue size: {}", self.stats.peak_queue_size);
            
            // Signal shutdown
            worker_pool.shutdown_flag.store(true, Ordering::Relaxed);
            
            // Wait for all workers to complete
            for worker in worker_pool.workers {
                let _ = worker.handle.await;
            }
            
            // Clear remaining state
            self.task_queue.clear();
            self.pending_tasks.clear();
            self.running_tasks.clear();
            self.completed_tasks.clear();
            self.dependency_graph.clear();
            self.task_handlers.clear();
            self.completion_callbacks.clear();
        }
        
        Ok(())
    }
}

pub type AsyncTaskManager = BackgroundProcessor;