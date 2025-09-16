use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc::{self, Sender, Receiver};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use crate::engine::error::RobinResult;

pub struct ThreadManager {
    thread_pool: Vec<WorkerThread>,
    task_queue: Arc<Mutex<Vec<Task>>>,
    completed_tasks: Arc<Mutex<Vec<CompletedTask>>>,
    thread_count: usize,
    stats: ThreadStats,
}

#[derive(Debug, Clone)]
pub struct ThreadStats {
    pub active_threads: usize,
    pub queued_tasks: usize,
    pub completed_tasks: u64,
    pub average_task_time: Duration,
    pub thread_utilization: f32,
    pub task_throughput: f32, // tasks per second
}

struct WorkerThread {
    handle: Option<thread::JoinHandle<()>>,
    sender: Sender<ThreadMessage>,
    stats: WorkerStats,
}

#[derive(Debug, Clone)]
struct WorkerStats {
    pub tasks_completed: u64,
    pub total_time: Duration,
    pub idle_time: Duration,
    pub last_activity: Instant,
}

#[derive(Debug)]
enum ThreadMessage {
    NewTask(Task),
    Shutdown,
}

#[derive(Debug, Clone)]
pub struct Task {
    pub id: u64,
    pub task_type: TaskType,
    pub priority: TaskPriority,
    pub data: TaskData,
    pub created_at: Instant,
    pub timeout: Option<Duration>,
}

#[derive(Debug, Clone)]
pub enum TaskType {
    Render,
    Physics,
    Audio,
    AI,
    IO,
    Network,
    Compute,
    General,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

#[derive(Debug, Clone)]
pub enum TaskData {
    RenderTask {
        mesh_id: u64,
        transform_data: Vec<f32>,
    },
    PhysicsTask {
        object_count: u32,
        simulation_data: Vec<u8>,
    },
    AudioTask {
        sample_data: Vec<f32>,
        processing_type: String,
    },
    AITask {
        entity_id: u64,
        decision_data: Vec<u8>,
    },
    IOTask {
        file_path: String,
        operation_type: IOOperation,
    },
    ComputeTask {
        compute_data: Vec<f32>,
        shader_program: String,
    },
    GeneralTask {
        closure: Box<dyn Fn() -> TaskResult + Send>,
    },
}

#[derive(Debug, Clone)]
pub enum IOOperation {
    Read,
    Write,
    Delete,
    Create,
}

#[derive(Debug)]
pub struct CompletedTask {
    pub id: u64,
    pub result: TaskResult,
    pub execution_time: Duration,
    pub completed_at: Instant,
}

#[derive(Debug)]
pub enum TaskResult {
    Success(Vec<u8>),
    Error(String),
    Timeout,
}

impl ThreadManager {
    pub fn new(thread_count: usize) -> RobinResult<Self> {
        let task_queue = Arc::new(Mutex::new(Vec::new()));
        let completed_tasks = Arc::new(Mutex::new(Vec::new()));
        let mut thread_pool = Vec::new();
        
        for i in 0..thread_count {
            let (sender, receiver) = mpsc::channel();
            let queue_clone = Arc::clone(&task_queue);
            let completed_clone = Arc::clone(&completed_tasks);
            
            let handle = thread::Builder::new()
                .name(format!("robin_worker_{}", i))
                .spawn(move || {
                    Self::worker_loop(receiver, queue_clone, completed_clone);
                })?;
            
            thread_pool.push(WorkerThread {
                handle: Some(handle),
                sender,
                stats: WorkerStats {
                    tasks_completed: 0,
                    total_time: Duration::new(0, 0),
                    idle_time: Duration::new(0, 0),
                    last_activity: Instant::now(),
                },
            });
        }
        
        Ok(Self {
            thread_pool,
            task_queue,
            completed_tasks,
            thread_count,
            stats: ThreadStats {
                active_threads: thread_count,
                queued_tasks: 0,
                completed_tasks: 0,
                average_task_time: Duration::new(0, 0),
                thread_utilization: 0.0,
                task_throughput: 0.0,
            },
        })
    }
    
    pub fn submit_task(&mut self, task: Task) -> RobinResult<()> {
        // Add task to queue
        {
            let mut queue = self.task_queue.lock().unwrap();
            queue.push(task.clone());
            
            // Sort by priority (highest first)
            queue.sort_by(|a, b| b.priority.cmp(&a.priority));
        }
        
        // Notify a worker thread
        let thread_index = (task.id % self.thread_count as u64) as usize;
        self.thread_pool[thread_index]
            .sender
            .send(ThreadMessage::NewTask(task))?;
        
        self.stats.queued_tasks += 1;
        
        Ok(())
    }
    
    pub fn submit_render_task(&mut self, mesh_id: u64, transform_data: Vec<f32>) -> RobinResult<u64> {
        let task_id = self.generate_task_id();
        let task = Task {
            id: task_id,
            task_type: TaskType::Render,
            priority: TaskPriority::High,
            data: TaskData::RenderTask { mesh_id, transform_data },
            created_at: Instant::now(),
            timeout: Some(Duration::from_millis(16)), // One frame at 60 FPS
        };
        
        self.submit_task(task)?;
        Ok(task_id)
    }
    
    pub fn submit_physics_task(&mut self, object_count: u32, simulation_data: Vec<u8>) -> RobinResult<u64> {
        let task_id = self.generate_task_id();
        let task = Task {
            id: task_id,
            task_type: TaskType::Physics,
            priority: TaskPriority::Normal,
            data: TaskData::PhysicsTask { object_count, simulation_data },
            created_at: Instant::now(),
            timeout: Some(Duration::from_millis(33)), // 30 FPS for physics
        };
        
        self.submit_task(task)?;
        Ok(task_id)
    }
    
    pub fn submit_ai_task(&mut self, entity_id: u64, decision_data: Vec<u8>) -> RobinResult<u64> {
        let task_id = self.generate_task_id();
        let task = Task {
            id: task_id,
            task_type: TaskType::AI,
            priority: TaskPriority::Low,
            data: TaskData::AITask { entity_id, decision_data },
            created_at: Instant::now(),
            timeout: Some(Duration::from_millis(100)), // AI can be slower
        };
        
        self.submit_task(task)?;
        Ok(task_id)
    }
    
    pub fn submit_io_task(&mut self, file_path: String, operation: IOOperation) -> RobinResult<u64> {
        let task_id = self.generate_task_id();
        let task = Task {
            id: task_id,
            task_type: TaskType::IO,
            priority: TaskPriority::Normal,
            data: TaskData::IOTask { file_path, operation_type: operation },
            created_at: Instant::now(),
            timeout: Some(Duration::from_secs(5)), // IO can be slow
        };
        
        self.submit_task(task)?;
        Ok(task_id)
    }
    
    pub fn get_completed_tasks(&mut self) -> Vec<CompletedTask> {
        let mut completed = self.completed_tasks.lock().unwrap();
        let tasks = completed.drain(..).collect();
        
        // Update stats
        self.stats.completed_tasks += tasks.len() as u64;
        
        if !tasks.is_empty() {
            let total_time: Duration = tasks.iter().map(|t| t.execution_time).sum();
            self.stats.average_task_time = total_time / tasks.len() as u32;
        }
        
        tasks
    }
    
    pub fn wait_for_task(&mut self, task_id: u64, timeout: Duration) -> RobinResult<TaskResult> {
        let start_time = Instant::now();
        
        loop {
            let completed_tasks = self.get_completed_tasks();
            
            for task in completed_tasks {
                if task.id == task_id {
                    return Ok(task.result);
                }
            }
            
            if start_time.elapsed() > timeout {
                return Ok(TaskResult::Timeout);
            }
            
            thread::sleep(Duration::from_millis(1));
        }
    }
    
    pub fn update_stats(&mut self) {
        // Update queue size
        self.stats.queued_tasks = self.task_queue.lock().unwrap().len();
        
        // Calculate thread utilization
        let mut total_utilization = 0.0;
        let mut active_threads = 0;
        
        for worker in &self.thread_pool {
            if worker.handle.is_some() {
                active_threads += 1;
                
                // Simple utilization based on recent activity
                let time_since_activity = worker.stats.last_activity.elapsed();
                if time_since_activity < Duration::from_millis(100) {
                    total_utilization += 1.0;
                } else if time_since_activity < Duration::from_secs(1) {
                    total_utilization += 0.5;
                }
            }
        }
        
        self.stats.active_threads = active_threads;
        self.stats.thread_utilization = if active_threads > 0 {
            total_utilization / active_threads as f32
        } else {
            0.0
        };
        
        // Calculate task throughput (tasks per second)
        // This is a simplified calculation - in practice you'd track this over time
        self.stats.task_throughput = self.stats.completed_tasks as f32;
    }
    
    pub fn get_stats(&self) -> ThreadStats {
        self.stats.clone()
    }
    
    pub fn get_utilization(&self) -> f32 {
        self.stats.thread_utilization
    }
    
    pub fn shutdown(&mut self) -> RobinResult<()> {
        // Send shutdown message to all workers
        for worker in &mut self.thread_pool {
            let _ = worker.sender.send(ThreadMessage::Shutdown);
        }
        
        // Wait for all threads to complete
        for worker in &mut self.thread_pool {
            if let Some(handle) = worker.handle.take() {
                let _ = handle.join();
            }
        }
        
        Ok(())
    }
    
    // Private methods
    
    fn worker_loop(
        receiver: Receiver<ThreadMessage>,
        task_queue: Arc<Mutex<Vec<Task>>>,
        completed_tasks: Arc<Mutex<Vec<CompletedTask>>>,
    ) {
        loop {
            // Try to receive a message
            match receiver.try_recv() {
                Ok(ThreadMessage::Shutdown) => break,
                Ok(ThreadMessage::NewTask(_)) => {
                    // Process task from queue
                    if let Some(task) = Self::get_next_task(&task_queue) {
                        let result = Self::execute_task(task);
                        Self::store_completed_task(result, &completed_tasks);
                    }
                }
                Err(_) => {
                    // No message, try to get task from queue anyway
                    if let Some(task) = Self::get_next_task(&task_queue) {
                        let result = Self::execute_task(task);
                        Self::store_completed_task(result, &completed_tasks);
                    } else {
                        // No work available, sleep briefly
                        thread::sleep(Duration::from_millis(1));
                    }
                }
            }
        }
    }
    
    fn get_next_task(queue: &Arc<Mutex<Vec<Task>>>) -> Option<Task> {
        let mut queue = queue.lock().unwrap();
        queue.pop()
    }
    
    fn execute_task(task: Task) -> CompletedTask {
        let start_time = Instant::now();
        let task_id = task.id;
        
        let result = match task.data {
            TaskData::RenderTask { mesh_id, transform_data } => {
                Self::execute_render_task(mesh_id, transform_data)
            }
            TaskData::PhysicsTask { object_count, simulation_data } => {
                Self::execute_physics_task(object_count, simulation_data)
            }
            TaskData::AudioTask { sample_data, processing_type } => {
                Self::execute_audio_task(sample_data, processing_type)
            }
            TaskData::AITask { entity_id, decision_data } => {
                Self::execute_ai_task(entity_id, decision_data)
            }
            TaskData::IOTask { file_path, operation_type } => {
                Self::execute_io_task(file_path, operation_type)
            }
            TaskData::ComputeTask { compute_data, shader_program } => {
                Self::execute_compute_task(compute_data, shader_program)
            }
            TaskData::GeneralTask { closure } => {
                closure()
            }
        };
        
        let execution_time = start_time.elapsed();
        
        // Check for timeout
        let final_result = if let Some(timeout) = task.timeout {
            if execution_time > timeout {
                TaskResult::Timeout
            } else {
                result
            }
        } else {
            result
        };
        
        CompletedTask {
            id: task_id,
            result: final_result,
            execution_time,
            completed_at: Instant::now(),
        }
    }
    
    fn execute_render_task(_mesh_id: u64, _transform_data: Vec<f32>) -> TaskResult {
        // Simulate render work
        thread::sleep(Duration::from_micros(100));
        TaskResult::Success(vec![1, 2, 3, 4]) // Dummy result
    }
    
    fn execute_physics_task(_object_count: u32, _simulation_data: Vec<u8>) -> TaskResult {
        // Simulate physics work
        thread::sleep(Duration::from_micros(500));
        TaskResult::Success(vec![5, 6, 7, 8]) // Dummy result
    }
    
    fn execute_audio_task(_sample_data: Vec<f32>, _processing_type: String) -> TaskResult {
        // Simulate audio processing
        thread::sleep(Duration::from_micros(200));
        TaskResult::Success(vec![9, 10, 11, 12]) // Dummy result
    }
    
    fn execute_ai_task(_entity_id: u64, _decision_data: Vec<u8>) -> TaskResult {
        // Simulate AI processing
        thread::sleep(Duration::from_millis(1));
        TaskResult::Success(vec![13, 14, 15, 16]) // Dummy result
    }
    
    fn execute_io_task(_file_path: String, _operation_type: IOOperation) -> TaskResult {
        // Simulate IO work
        thread::sleep(Duration::from_millis(5));
        TaskResult::Success(vec![17, 18, 19, 20]) // Dummy result
    }
    
    fn execute_compute_task(_compute_data: Vec<f32>, _shader_program: String) -> TaskResult {
        // Simulate compute shader work
        thread::sleep(Duration::from_micros(300));
        TaskResult::Success(vec![21, 22, 23, 24]) // Dummy result
    }
    
    fn store_completed_task(task: CompletedTask, completed_tasks: &Arc<Mutex<Vec<CompletedTask>>>) {
        let mut completed = completed_tasks.lock().unwrap();
        completed.push(task);
    }
    
    fn generate_task_id(&mut self) -> u64 {
        // Simple task ID generation - in practice you'd want something more robust
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }
}

impl Drop for ThreadManager {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

// Convenience functions for common task patterns

pub fn parallel_for<F>(start: usize, end: usize, thread_manager: &mut ThreadManager, f: F) -> RobinResult<Vec<u64>>
where
    F: Fn(usize) -> TaskResult + Send + Clone + 'static,
{
    let mut task_ids = Vec::new();
    let chunk_size = ((end - start) / thread_manager.thread_count).max(1);
    
    for chunk_start in (start..end).step_by(chunk_size) {
        let chunk_end = (chunk_start + chunk_size).min(end);
        let f_clone = f.clone();
        
        let task = Task {
            id: 0, // Will be set by submit_task
            task_type: TaskType::General,
            priority: TaskPriority::Normal,
            data: TaskData::GeneralTask {
                closure: Box::new(move || {
                    let mut results = Vec::new();
                    for i in chunk_start..chunk_end {
                        match f_clone(i) {
                            TaskResult::Success(mut data) => results.extend(data),
                            _ => {} // Handle errors appropriately
                        }
                    }
                    TaskResult::Success(results)
                }),
            },
            created_at: Instant::now(),
            timeout: None,
        };
        
        let task_id = thread_manager.submit_task(task)?;
        task_ids.push(task_id);
    }
    
    Ok(task_ids)
}

pub fn wait_for_all(task_ids: Vec<u64>, thread_manager: &mut ThreadManager, timeout: Duration) -> RobinResult<Vec<TaskResult>> {
    let mut results = Vec::new();
    
    for task_id in task_ids {
        let result = thread_manager.wait_for_task(task_id, timeout)?;
        results.push(result);
    }
    
    Ok(results)
}