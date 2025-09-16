use crate::engine::error::RobinResult;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilerConfig {
    pub enable_frame_profiling: bool,
    pub enable_system_profiling: bool,
    pub enable_memory_profiling: bool,
    pub enable_gpu_profiling: bool,
    pub max_frame_history: usize,
    pub profile_interval_ms: u64,
    pub auto_optimization: bool,
    pub detailed_logging: bool,
    pub export_data: bool,
}

impl Default for ProfilerConfig {
    fn default() -> Self {
        Self {
            enable_frame_profiling: true,
            enable_system_profiling: true,
            enable_memory_profiling: true,
            enable_gpu_profiling: false,
            max_frame_history: 300, // 5 seconds at 60 FPS
            profile_interval_ms: 16, // ~60 FPS
            auto_optimization: true,
            detailed_logging: false,
            export_data: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FrameProfile {
    pub frame_number: u64,
    pub timestamp: Instant,
    pub frame_time_ms: f32,
    pub cpu_time_ms: f32,
    pub gpu_time_ms: f32,
    pub render_time_ms: f32,
    pub physics_time_ms: f32,
    pub ai_time_ms: f32,
    pub audio_time_ms: f32,
    pub memory_usage_mb: f32,
    pub draw_calls: u32,
    pub triangles: u32,
    pub entities_processed: u32,
}

#[derive(Debug, Clone)]
pub struct SystemProfile {
    pub system_name: String,
    pub execution_time_ms: f32,
    pub memory_usage_mb: f32,
    pub cpu_usage_percent: f32,
    pub calls_per_frame: u32,
    pub peak_execution_time_ms: f32,
    pub average_execution_time_ms: f32,
    pub total_executions: u64,
}

#[derive(Debug, Clone)]
pub struct MemoryProfile {
    pub timestamp: Instant,
    pub total_allocated_mb: f32,
    pub heap_usage_mb: f32,
    pub stack_usage_mb: f32,
    pub gpu_memory_mb: f32,
    pub texture_memory_mb: f32,
    pub mesh_memory_mb: f32,
    pub audio_memory_mb: f32,
    pub fragmentation_ratio: f32,
    pub allocation_rate_mb_per_sec: f32,
}

#[derive(Debug, Default)]
pub struct PerformanceMetrics {
    pub average_fps: f32,
    pub min_fps: f32,
    pub max_fps: f32,
    pub frame_time_variance: f32,
    pub cpu_usage_average: f32,
    pub memory_usage_average_mb: f32,
    pub peak_memory_usage_mb: f32,
    pub total_draw_calls: u64,
    pub total_triangles: u64,
    pub performance_score: f32,
}

#[derive(Debug)]
pub struct ProfilerSample {
    pub name: String,
    pub start_time: Instant,
    pub duration: Option<Duration>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub struct PerformanceProfiler {
    config: ProfilerConfig,
    frame_profiles: VecDeque<FrameProfile>,
    system_profiles: HashMap<String, SystemProfile>,
    memory_profiles: VecDeque<MemoryProfile>,
    active_samples: HashMap<String, ProfilerSample>,
    metrics: PerformanceMetrics,
    frame_counter: u64,
    last_profile_time: Instant,
    profiling_overhead_ms: f32,
    optimization_suggestions: Vec<String>,
}

impl PerformanceProfiler {
    pub fn new(config: ProfilerConfig) -> RobinResult<Self> {
        Ok(Self {
            config: config.clone(),
            frame_profiles: VecDeque::with_capacity(config.max_frame_history),
            system_profiles: HashMap::new(),
            memory_profiles: VecDeque::with_capacity(100),
            active_samples: HashMap::new(),
            metrics: PerformanceMetrics::default(),
            frame_counter: 0,
            last_profile_time: Instant::now(),
            profiling_overhead_ms: 0.0,
            optimization_suggestions: Vec::new(),
        })
    }

    pub fn begin_frame(&mut self) -> RobinResult<()> {
        if !self.config.enable_frame_profiling {
            return Ok(());
        }

        self.frame_counter += 1;
        self.start_sample("frame".to_string());
        Ok(())
    }

    pub fn end_frame(&mut self) -> RobinResult<()> {
        if !self.config.enable_frame_profiling {
            return Ok(());
        }

        self.end_sample("frame");
        
        // Collect frame data
        if let Some(frame_sample) = self.active_samples.remove("frame") {
            if let Some(duration) = frame_sample.duration {
                let frame_time_ms = duration.as_secs_f32() * 1000.0;
                
                let profile = FrameProfile {
                    frame_number: self.frame_counter,
                    timestamp: frame_sample.start_time,
                    frame_time_ms,
                    cpu_time_ms: self.get_cpu_time_ms(),
                    gpu_time_ms: self.get_gpu_time_ms(),
                    render_time_ms: self.get_system_time_ms("render"),
                    physics_time_ms: self.get_system_time_ms("physics"),
                    ai_time_ms: self.get_system_time_ms("ai"),
                    audio_time_ms: self.get_system_time_ms("audio"),
                    memory_usage_mb: self.get_current_memory_usage_mb(),
                    draw_calls: self.get_draw_calls(),
                    triangles: self.get_triangle_count(),
                    entities_processed: self.get_entities_processed(),
                };

                self.add_frame_profile(profile);
            }
        }

        // Update metrics periodically
        if self.frame_counter % 60 == 0 { // Every second at 60 FPS
            self.update_performance_metrics()?;
        }

        Ok(())
    }

    pub fn start_sample(&mut self, name: String) {
        if !self.config.enable_system_profiling {
            return;
        }

        let sample = ProfilerSample {
            name: name.clone(),
            start_time: Instant::now(),
            duration: None,
            metadata: HashMap::new(),
        };

        self.active_samples.insert(name, sample);
    }

    pub fn end_sample(&mut self, name: &str) {
        if let Some(mut sample) = self.active_samples.remove(name) {
            sample.duration = Some(sample.start_time.elapsed());
            
            // Update system profile
            if let Some(duration) = sample.duration {
                let execution_time_ms = duration.as_secs_f32() * 1000.0;
                
                let system_profile = self.system_profiles.entry(name.to_string())
                    .or_insert_with(|| SystemProfile {
                        system_name: name.to_string(),
                        execution_time_ms: 0.0,
                        memory_usage_mb: 0.0,
                        cpu_usage_percent: 0.0,
                        calls_per_frame: 0,
                        peak_execution_time_ms: 0.0,
                        average_execution_time_ms: 0.0,
                        total_executions: 0,
                    });

                system_profile.execution_time_ms = execution_time_ms;
                system_profile.peak_execution_time_ms = 
                    system_profile.peak_execution_time_ms.max(execution_time_ms);
                system_profile.total_executions += 1;
                system_profile.calls_per_frame += 1;
                
                // Update rolling average
                let alpha = 0.1; // Smoothing factor
                system_profile.average_execution_time_ms = 
                    system_profile.average_execution_time_ms * (1.0 - alpha) + 
                    execution_time_ms * alpha;
            }
        }
    }

    pub fn sample_memory(&mut self) -> RobinResult<()> {
        if !self.config.enable_memory_profiling {
            return Ok(());
        }

        let profile = MemoryProfile {
            timestamp: Instant::now(),
            total_allocated_mb: self.get_total_allocated_mb(),
            heap_usage_mb: self.get_heap_usage_mb(),
            stack_usage_mb: self.get_stack_usage_mb(),
            gpu_memory_mb: self.get_gpu_memory_mb(),
            texture_memory_mb: self.get_texture_memory_mb(),
            mesh_memory_mb: self.get_mesh_memory_mb(),
            audio_memory_mb: self.get_audio_memory_mb(),
            fragmentation_ratio: self.get_fragmentation_ratio(),
            allocation_rate_mb_per_sec: self.get_allocation_rate(),
        };

        self.memory_profiles.push_back(profile);
        
        // Keep memory profile history limited
        if self.memory_profiles.len() > 100 {
            self.memory_profiles.pop_front();
        }

        Ok(())
    }

    pub fn update(&mut self, delta_time: f32) -> RobinResult<()> {
        // Reset per-frame counters for system profiles
        for profile in self.system_profiles.values_mut() {
            profile.calls_per_frame = 0;
        }

        // Sample memory periodically
        if self.last_profile_time.elapsed().as_millis() as u64 >= self.config.profile_interval_ms {
            if self.config.enable_memory_profiling {
                self.sample_memory()?;
            }
            self.last_profile_time = Instant::now();
        }

        // Generate optimization suggestions
        if self.config.auto_optimization && self.frame_counter % 300 == 0 {
            self.generate_optimization_suggestions()?;
        }

        Ok(())
    }

    fn add_frame_profile(&mut self, profile: FrameProfile) {
        self.frame_profiles.push_back(profile);
        
        // Maintain frame history limit
        if self.frame_profiles.len() > self.config.max_frame_history {
            self.frame_profiles.pop_front();
        }
    }

    fn update_performance_metrics(&mut self) -> RobinResult<()> {
        if self.frame_profiles.is_empty() {
            return Ok(());
        }

        // Calculate FPS metrics
        let frame_times: Vec<f32> = self.frame_profiles.iter()
            .map(|p| p.frame_time_ms)
            .collect();

        let fps_values: Vec<f32> = frame_times.iter()
            .map(|&ft| if ft > 0.0 { 1000.0 / ft } else { 0.0 })
            .collect();

        self.metrics.average_fps = fps_values.iter().sum::<f32>() / fps_values.len() as f32;
        self.metrics.min_fps = fps_values.iter().cloned().fold(f32::INFINITY, f32::min);
        self.metrics.max_fps = fps_values.iter().cloned().fold(0.0, f32::max);

        // Calculate frame time variance
        let mean_frame_time = frame_times.iter().sum::<f32>() / frame_times.len() as f32;
        self.metrics.frame_time_variance = frame_times.iter()
            .map(|&ft| (ft - mean_frame_time).powi(2))
            .sum::<f32>() / frame_times.len() as f32;

        // Memory metrics
        self.metrics.memory_usage_average_mb = self.frame_profiles.iter()
            .map(|p| p.memory_usage_mb)
            .sum::<f32>() / self.frame_profiles.len() as f32;

        self.metrics.peak_memory_usage_mb = self.frame_profiles.iter()
            .map(|p| p.memory_usage_mb)
            .fold(0.0, f32::max);

        // Rendering metrics
        self.metrics.total_draw_calls = self.frame_profiles.iter()
            .map(|p| p.draw_calls as u64)
            .sum();

        self.metrics.total_triangles = self.frame_profiles.iter()
            .map(|p| p.triangles as u64)
            .sum();

        // Calculate overall performance score (0-100)
        let fps_score = (self.metrics.average_fps / 60.0).min(1.0) * 40.0;
        let consistency_score = (1.0 - (self.metrics.frame_time_variance.sqrt() / mean_frame_time).min(1.0)) * 30.0;
        let memory_score = (1.0 - (self.metrics.memory_usage_average_mb / 2048.0).min(1.0)) * 30.0;
        
        self.metrics.performance_score = fps_score + consistency_score + memory_score;

        Ok(())
    }

    fn generate_optimization_suggestions(&mut self) -> RobinResult<()> {
        self.optimization_suggestions.clear();

        // FPS-based suggestions
        if self.metrics.average_fps < 30.0 {
            self.optimization_suggestions.push("Consider reducing visual quality settings - FPS is below 30".to_string());
        }

        if self.metrics.frame_time_variance > 10.0 {
            self.optimization_suggestions.push("Frame times are inconsistent - check for GC spikes or background processes".to_string());
        }

        // Memory-based suggestions
        if self.metrics.memory_usage_average_mb > 1500.0 {
            self.optimization_suggestions.push("High memory usage detected - consider more aggressive garbage collection".to_string());
        }

        // System-specific suggestions
        for (system_name, profile) in &self.system_profiles {
            if profile.average_execution_time_ms > 8.0 { // > half frame at 60 FPS
                self.optimization_suggestions.push(
                    format!("System '{}' taking {:.1}ms - consider optimization", 
                           system_name, profile.average_execution_time_ms)
                );
            }
        }

        // Draw call suggestions
        let avg_draw_calls = self.metrics.total_draw_calls as f32 / self.frame_profiles.len() as f32;
        if avg_draw_calls > 1000.0 {
            self.optimization_suggestions.push("High draw call count - consider batching or instancing".to_string());
        }

        Ok(())
    }

    // Placeholder methods for system integration - these would connect to actual engine systems
    fn get_cpu_time_ms(&self) -> f32 { 
        // Would integrate with system CPU monitoring
        8.5 
    }

    fn get_gpu_time_ms(&self) -> f32 { 
        // Would integrate with GPU profiling
        12.3 
    }

    fn get_system_time_ms(&self, system: &str) -> f32 {
        self.system_profiles.get(system)
            .map(|p| p.average_execution_time_ms)
            .unwrap_or(0.0)
    }

    fn get_current_memory_usage_mb(&self) -> f32 { 
        // Would integrate with memory manager
        512.0 
    }

    fn get_draw_calls(&self) -> u32 { 156 }
    fn get_triangle_count(&self) -> u32 { 12450 }
    fn get_entities_processed(&self) -> u32 { 89 }
    
    fn get_total_allocated_mb(&self) -> f32 { 512.0 }
    fn get_heap_usage_mb(&self) -> f32 { 256.0 }
    fn get_stack_usage_mb(&self) -> f32 { 4.0 }
    fn get_gpu_memory_mb(&self) -> f32 { 128.0 }
    fn get_texture_memory_mb(&self) -> f32 { 64.0 }
    fn get_mesh_memory_mb(&self) -> f32 { 32.0 }
    fn get_audio_memory_mb(&self) -> f32 { 16.0 }
    fn get_fragmentation_ratio(&self) -> f32 { 0.15 }
    fn get_allocation_rate(&self) -> f32 { 2.5 }

    // Public API methods
    pub fn get_metrics(&self) -> &PerformanceMetrics {
        &self.metrics
    }

    pub fn get_frame_profiles(&self) -> &VecDeque<FrameProfile> {
        &self.frame_profiles
    }

    pub fn get_system_profiles(&self) -> &HashMap<String, SystemProfile> {
        &self.system_profiles
    }

    pub fn get_memory_profiles(&self) -> &VecDeque<MemoryProfile> {
        &self.memory_profiles
    }

    pub fn get_optimization_suggestions(&self) -> &Vec<String> {
        &self.optimization_suggestions
    }

    pub fn get_average_fps(&self) -> f32 {
        self.metrics.average_fps
    }

    pub fn get_frame_time_ms(&self) -> f32 {
        self.frame_profiles.back()
            .map(|p| p.frame_time_ms)
            .unwrap_or(0.0)
    }

    pub fn get_performance_score(&self) -> f32 {
        self.metrics.performance_score
    }

    pub fn reset_metrics(&mut self) {
        self.frame_profiles.clear();
        self.system_profiles.clear();
        self.memory_profiles.clear();
        self.metrics = PerformanceMetrics::default();
        self.frame_counter = 0;
        self.optimization_suggestions.clear();
    }

    pub fn export_profile_data(&self) -> RobinResult<String> {
        if !self.config.export_data {
            return Ok(String::new());
        }

        // Create a simplified JSON export of profile data
        let export_data = serde_json::json!({
            "metrics": {
                "average_fps": self.metrics.average_fps,
                "min_fps": self.metrics.min_fps,
                "max_fps": self.metrics.max_fps,
                "frame_time_variance": self.metrics.frame_time_variance,
                "performance_score": self.metrics.performance_score,
                "memory_usage_average_mb": self.metrics.memory_usage_average_mb,
                "peak_memory_usage_mb": self.metrics.peak_memory_usage_mb,
            },
            "system_profiles": self.system_profiles.iter().map(|(name, profile)| {
                (name, serde_json::json!({
                    "average_execution_time_ms": profile.average_execution_time_ms,
                    "peak_execution_time_ms": profile.peak_execution_time_ms,
                    "total_executions": profile.total_executions,
                }))
            }).collect::<HashMap<_, _>>(),
            "optimization_suggestions": self.optimization_suggestions,
            "frame_count": self.frame_counter,
        });

        serde_json::to_string_pretty(&export_data)
            .map_err(|e| crate::engine::error::RobinError::new(format!("Export error: {}", e)))
    }

    pub fn enable_detailed_logging(&mut self, enabled: bool) {
        self.config.detailed_logging = enabled;
    }

    pub fn set_auto_optimization(&mut self, enabled: bool) {
        self.config.auto_optimization = enabled;
    }

    pub fn shutdown(&mut self) -> RobinResult<()> {
        if self.config.detailed_logging {
            println!("Performance Profiler shutdown:");
            println!("  Frames profiled: {}", self.frame_counter);
            println!("  Average FPS: {:.1}", self.metrics.average_fps);
            println!("  Performance score: {:.1}/100", self.metrics.performance_score);
            println!("  Profiling overhead: {:.2}ms", self.profiling_overhead_ms);
        }

        self.frame_profiles.clear();
        self.system_profiles.clear();
        self.memory_profiles.clear();
        self.active_samples.clear();
        self.optimization_suggestions.clear();

        Ok(())
    }
}

pub type PerformanceMonitor = PerformanceProfiler;