use std::collections::HashMap;
use std::sync::{Arc, RwLock, Mutex};
use std::time::{Duration, Instant};
use cgmath::{Vector3, Matrix4, Quaternion, Rad};
use wgpu::{Device, Queue, CommandEncoder, RenderPass, Buffer};
use winit::event::{WindowEvent, KeyboardInput, VirtualKeyCode, ElementState};

use crate::engine::showcase::{
    content_manager::{ContentManager, ShowcaseContent, ContentType},
    interactive_playground::{InteractivePlayground, PlaygroundMode},
    visual_showcase::{VisualShowcase, VisualScene},
    performance_showcase::{PerformanceBenchmark, BenchmarkTest},
    camera_tours::{CameraTourController, TourType},
};
use crate::engine::ui::{
    production_ui::{ProductionUISystem, UIEvent},
    welcome_flow::{WelcomeFlow, DemoMode},
    transitions::{TransitionSystem, TransitionType, EasingFunction},
};

/// Main integration system that orchestrates all showcase components
/// and provides seamless transitions between different demo modes
pub struct ShowcaseIntegration {
    // Core showcase systems
    content_manager: Arc<Mutex<ContentManager>>,
    interactive_playground: Arc<Mutex<InteractivePlayground>>,
    visual_showcase: Arc<Mutex<VisualShowcase>>,
    performance_benchmark: Arc<Mutex<PerformanceBenchmark>>,
    camera_tours: Arc<Mutex<CameraTourController>>,

    // UI and transition systems
    ui_system: Arc<Mutex<ProductionUISystem>>,
    transition_system: TransitionSystem,

    // State management
    current_mode: DemoMode,
    previous_mode: Option<DemoMode>,
    transitioning: bool,
    transition_progress: f32,
    transition_start_time: Instant,

    // Performance monitoring
    performance_monitor: PerformanceMonitor,
    memory_manager: MemoryManager,

    // Configuration
    config: ShowcaseConfig,

    // Event handling
    event_queue: Arc<RwLock<Vec<ShowcaseEvent>>>,

    // Demo state preservation
    demo_states: HashMap<DemoMode, DemoState>,
}

#[derive(Debug, Clone)]
pub struct ShowcaseConfig {
    pub transition_duration: Duration,
    pub memory_limit_mb: usize,
    pub preload_adjacent_demos: bool,
    pub enable_performance_overlay: bool,
    pub auto_tour_mode: bool,
    pub camera_smoothing: f32,
    pub ui_fade_duration: Duration,
    pub content_load_timeout: Duration,
}

impl Default for ShowcaseConfig {
    fn default() -> Self {
        Self {
            transition_duration: Duration::from_millis(1500),
            memory_limit_mb: 2048,
            preload_adjacent_demos: true,
            enable_performance_overlay: true,
            auto_tour_mode: false,
            camera_smoothing: 0.85,
            ui_fade_duration: Duration::from_millis(300),
            content_load_timeout: Duration::from_secs(30),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ShowcaseEvent {
    ModeChanged { from: DemoMode, to: DemoMode },
    TransitionStarted { target_mode: DemoMode },
    TransitionCompleted { mode: DemoMode },
    ContentLoaded { content_type: ContentType },
    PerformanceBenchmarkCompleted { test: BenchmarkTest, results: BenchmarkResults },
    TourStarted { tour_type: TourType },
    TourCompleted { tour_type: TourType },
    ErrorOccurred { error: ShowcaseError },
    MemoryWarning { usage_mb: usize, limit_mb: usize },
}

#[derive(Debug, Clone)]
pub struct BenchmarkResults {
    pub fps: f32,
    pub frame_time_ms: f32,
    pub memory_usage_mb: usize,
    pub gpu_usage_percent: f32,
    pub voxels_rendered: usize,
    pub particles_active: usize,
    pub draw_calls: usize,
}

#[derive(Debug, Clone)]
pub enum ShowcaseError {
    ContentLoadFailed { content_type: ContentType, reason: String },
    TransitionFailed { from: DemoMode, to: DemoMode, reason: String },
    MemoryLimitExceeded { requested_mb: usize, available_mb: usize },
    GPUResourceExhausted { resource_type: String },
    BenchmarkTimedOut { test: BenchmarkTest },
}

/// Preserves demo state when switching between modes
#[derive(Debug, Clone)]
pub struct DemoState {
    pub camera_position: Vector3<f32>,
    pub camera_rotation: Quaternion<f32>,
    pub time_in_demo: Duration,
    pub user_settings: HashMap<String, String>,
    pub content_progress: f32,
    pub tour_position: Option<f32>,
}

/// Monitors performance across all showcase systems
pub struct PerformanceMonitor {
    frame_times: Vec<f32>,
    memory_samples: Vec<usize>,
    gpu_usage_samples: Vec<f32>,
    last_update: Instant,
    sample_interval: Duration,
    max_samples: usize,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            frame_times: Vec::with_capacity(1000),
            memory_samples: Vec::with_capacity(1000),
            gpu_usage_samples: Vec::with_capacity(1000),
            last_update: Instant::now(),
            sample_interval: Duration::from_millis(16), // ~60fps sampling
            max_samples: 1000,
        }
    }

    pub fn update(&mut self, frame_time: f32, memory_mb: usize, gpu_usage: f32) {
        let now = Instant::now();
        if now.duration_since(self.last_update) >= self.sample_interval {
            self.frame_times.push(frame_time);
            self.memory_samples.push(memory_mb);
            self.gpu_usage_samples.push(gpu_usage);

            // Keep only the most recent samples
            if self.frame_times.len() > self.max_samples {
                self.frame_times.remove(0);
                self.memory_samples.remove(0);
                self.gpu_usage_samples.remove(0);
            }

            self.last_update = now;
        }
    }

    pub fn get_average_fps(&self) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        let avg_frame_time = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
        if avg_frame_time > 0.0 {
            1000.0 / avg_frame_time
        } else {
            0.0
        }
    }

    pub fn get_memory_usage_mb(&self) -> usize {
        self.memory_samples.last().cloned().unwrap_or(0)
    }

    pub fn get_memory_trend(&self) -> MemoryTrend {
        if self.memory_samples.len() < 10 {
            return MemoryTrend::Stable;
        }

        let recent = &self.memory_samples[self.memory_samples.len() - 10..];
        let first_half_avg = recent[..5].iter().sum::<usize>() as f32 / 5.0;
        let second_half_avg = recent[5..].iter().sum::<usize>() as f32 / 5.0;

        let change_percent = ((second_half_avg - first_half_avg) / first_half_avg) * 100.0;

        if change_percent > 5.0 {
            MemoryTrend::Increasing
        } else if change_percent < -5.0 {
            MemoryTrend::Decreasing
        } else {
            MemoryTrend::Stable
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MemoryTrend {
    Increasing,
    Decreasing,
    Stable,
}

/// Manages memory usage across all showcase systems
pub struct MemoryManager {
    memory_limit_mb: usize,
    current_usage_mb: usize,
    content_cache_mb: usize,
    gpu_buffer_mb: usize,
    texture_cache_mb: usize,
    last_gc_time: Instant,
    gc_interval: Duration,
}

impl MemoryManager {
    pub fn new(memory_limit_mb: usize) -> Self {
        Self {
            memory_limit_mb,
            current_usage_mb: 0,
            content_cache_mb: 0,
            gpu_buffer_mb: 0,
            texture_cache_mb: 0,
            last_gc_time: Instant::now(),
            gc_interval: Duration::from_secs(10),
        }
    }

    pub fn update_usage(&mut self, content_mb: usize, gpu_mb: usize, texture_mb: usize) {
        self.content_cache_mb = content_mb;
        self.gpu_buffer_mb = gpu_mb;
        self.texture_cache_mb = texture_mb;
        self.current_usage_mb = content_mb + gpu_mb + texture_mb;

        // Trigger garbage collection if needed
        if self.should_run_gc() {
            self.run_garbage_collection();
        }
    }

    pub fn can_allocate(&self, requested_mb: usize) -> bool {
        self.current_usage_mb + requested_mb <= self.memory_limit_mb
    }

    pub fn get_usage_percent(&self) -> f32 {
        (self.current_usage_mb as f32 / self.memory_limit_mb as f32) * 100.0
    }

    pub fn is_memory_warning(&self) -> bool {
        self.get_usage_percent() > 80.0
    }

    fn should_run_gc(&self) -> bool {
        let now = Instant::now();
        now.duration_since(self.last_gc_time) >= self.gc_interval
            && (self.get_usage_percent() > 70.0)
    }

    fn run_garbage_collection(&mut self) {
        // This would trigger cleanup in content manager and other systems
        println!("Running garbage collection - Memory usage: {}%", self.get_usage_percent());
        self.last_gc_time = Instant::now();
    }
}

impl ShowcaseIntegration {
    pub fn new(
        device: &Device,
        queue: &Queue,
        surface_format: wgpu::TextureFormat,
    ) -> Result<Self, ShowcaseError> {
        let config = ShowcaseConfig::default();

        // Initialize all showcase systems
        let content_manager = Arc::new(Mutex::new(
            ContentManager::new().map_err(|e| ShowcaseError::ContentLoadFailed {
                content_type: ContentType::Scene,
                reason: format!("Failed to initialize content manager: {}", e),
            })?
        ));

        let interactive_playground = Arc::new(Mutex::new(
            InteractivePlayground::new(device, queue, surface_format)
        ));

        let visual_showcase = Arc::new(Mutex::new(
            VisualShowcase::new(device, queue, surface_format)
        ));

        let performance_benchmark = Arc::new(Mutex::new(
            PerformanceBenchmark::new(device, queue)
        ));

        let camera_tours = Arc::new(Mutex::new(
            CameraTourController::new()
        ));

        let ui_system = Arc::new(Mutex::new(
            ProductionUISystem::new(device, queue, surface_format)
                .map_err(|e| ShowcaseError::TransitionFailed {
                    from: DemoMode::Welcome,
                    to: DemoMode::Welcome,
                    reason: format!("Failed to initialize UI system: {}", e),
                })?
        ));

        let transition_system = TransitionSystem::new();

        Ok(Self {
            content_manager,
            interactive_playground,
            visual_showcase,
            performance_benchmark,
            camera_tours,
            ui_system,
            transition_system,
            current_mode: DemoMode::Welcome,
            previous_mode: None,
            transitioning: false,
            transition_progress: 0.0,
            transition_start_time: Instant::now(),
            performance_monitor: PerformanceMonitor::new(),
            memory_manager: MemoryManager::new(config.memory_limit_mb),
            config,
            event_queue: Arc::new(RwLock::new(Vec::new())),
            demo_states: HashMap::new(),
        })
    }

    /// Initiate transition to a new demo mode
    pub fn transition_to_mode(&mut self, target_mode: DemoMode) -> Result<(), ShowcaseError> {
        if self.transitioning {
            return Err(ShowcaseError::TransitionFailed {
                from: self.current_mode.clone(),
                to: target_mode,
                reason: "Already in transition".to_string(),
            });
        }

        // Save current demo state
        self.save_current_demo_state();

        // Check memory requirements for target mode
        let required_memory = self.estimate_memory_requirement(&target_mode);
        if !self.memory_manager.can_allocate(required_memory) {
            return Err(ShowcaseError::MemoryLimitExceeded {
                requested_mb: required_memory,
                available_mb: self.memory_manager.memory_limit_mb - self.memory_manager.current_usage_mb,
            });
        }

        // Start transition
        self.previous_mode = Some(self.current_mode.clone());
        self.transitioning = true;
        self.transition_progress = 0.0;
        self.transition_start_time = Instant::now();

        // Queue transition event
        self.queue_event(ShowcaseEvent::TransitionStarted { target_mode: target_mode.clone() });

        // Start content preloading for target mode
        self.preload_content_for_mode(&target_mode)?;

        // Configure target showcase system
        self.configure_showcase_for_mode(&target_mode)?;

        // Begin UI transition
        self.start_ui_transition(&target_mode);

        self.current_mode = target_mode;

        Ok(())
    }

    /// Update showcase integration systems
    pub fn update(&mut self, delta_time: f32) -> Result<(), ShowcaseError> {
        // Update performance monitoring
        let frame_time = delta_time * 1000.0; // Convert to milliseconds
        let memory_usage = self.memory_manager.get_usage_percent() as usize;
        let gpu_usage = self.estimate_gpu_usage();
        self.performance_monitor.update(frame_time, memory_usage, gpu_usage);

        // Update memory manager
        let (content_mb, gpu_mb, texture_mb) = self.collect_memory_usage();
        self.memory_manager.update_usage(content_mb, gpu_mb, texture_mb);

        // Check for memory warnings
        if self.memory_manager.is_memory_warning() {
            self.queue_event(ShowcaseEvent::MemoryWarning {
                usage_mb: self.memory_manager.current_usage_mb,
                limit_mb: self.memory_manager.memory_limit_mb,
            });
        }

        // Update transition state
        if self.transitioning {
            self.update_transition(delta_time)?;
        }

        // Update current showcase system
        self.update_current_showcase(delta_time)?;

        // Update UI system
        if let Ok(mut ui) = self.ui_system.lock() {
            ui.update(delta_time);
        }

        // Process event queue
        self.process_events();

        Ok(())
    }

    /// Render current showcase mode
    pub fn render(&mut self, encoder: &mut CommandEncoder, view: &wgpu::TextureView) -> Result<(), ShowcaseError> {
        // Render current showcase
        self.render_current_showcase(encoder, view)?;

        // Render transition overlay if transitioning
        if self.transitioning {
            self.render_transition_overlay(encoder, view)?;
        }

        // Render UI overlay
        if let Ok(mut ui) = self.ui_system.lock() {
            ui.render(encoder, view);
        }

        // Render performance overlay if enabled
        if self.config.enable_performance_overlay {
            self.render_performance_overlay(encoder, view)?;
        }

        Ok(())
    }

    /// Handle window events
    pub fn handle_event(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput { input, .. } => {
                self.handle_keyboard_input(input)
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.handle_mouse_move(*position);
                false
            }
            WindowEvent::MouseInput { button, state, .. } => {
                self.handle_mouse_click(*button, *state);
                false
            }
            _ => false,
        }
    }

    fn save_current_demo_state(&mut self) {
        // Collect current state from active showcase
        let state = match self.current_mode {
            DemoMode::InteractiveDemo => {
                if let Ok(playground) = self.interactive_playground.lock() {
                    DemoState {
                        camera_position: playground.get_camera_position(),
                        camera_rotation: playground.get_camera_rotation(),
                        time_in_demo: playground.get_session_duration(),
                        user_settings: playground.get_user_settings(),
                        content_progress: playground.get_progress(),
                        tour_position: None,
                    }
                } else {
                    return;
                }
            }
            DemoMode::VisualShowcase => {
                if let Ok(visual) = self.visual_showcase.lock() {
                    DemoState {
                        camera_position: visual.get_camera_position(),
                        camera_rotation: visual.get_camera_rotation(),
                        time_in_demo: visual.get_session_duration(),
                        user_settings: HashMap::new(),
                        content_progress: visual.get_scene_progress(),
                        tour_position: None,
                    }
                } else {
                    return;
                }
            }
            DemoMode::PerformanceDemo => {
                if let Ok(benchmark) = self.performance_benchmark.lock() {
                    DemoState {
                        camera_position: benchmark.get_camera_position(),
                        camera_rotation: benchmark.get_camera_rotation(),
                        time_in_demo: benchmark.get_session_duration(),
                        user_settings: HashMap::new(),
                        content_progress: benchmark.get_test_progress(),
                        tour_position: None,
                    }
                } else {
                    return;
                }
            }
            DemoMode::CinematicTour => {
                if let Ok(tours) = self.camera_tours.lock() {
                    DemoState {
                        camera_position: tours.get_camera_position(),
                        camera_rotation: tours.get_camera_rotation(),
                        time_in_demo: tours.get_session_duration(),
                        user_settings: HashMap::new(),
                        content_progress: 0.0,
                        tour_position: Some(tours.get_current_time()),
                    }
                } else {
                    return;
                }
            }
            _ => {
                DemoState {
                    camera_position: Vector3::new(0.0, 2.0, 5.0),
                    camera_rotation: Quaternion::from_angle_y(Rad(0.0)),
                    time_in_demo: Duration::from_secs(0),
                    user_settings: HashMap::new(),
                    content_progress: 0.0,
                    tour_position: None,
                }
            }
        };

        self.demo_states.insert(self.current_mode.clone(), state);
    }

    fn estimate_memory_requirement(&self, mode: &DemoMode) -> usize {
        match mode {
            DemoMode::Welcome => 50,
            DemoMode::QuickDemo => 200,
            DemoMode::InteractiveDemo => 400,
            DemoMode::VisualShowcase => 600,
            DemoMode::PerformanceDemo => 800,
            DemoMode::CinematicTour => 300,
        }
    }

    fn preload_content_for_mode(&self, mode: &DemoMode) -> Result<(), ShowcaseError> {
        if let Ok(mut content_manager) = self.content_manager.lock() {
            match mode {
                DemoMode::InteractiveDemo => {
                    content_manager.preload_content(&[
                        ContentType::Tutorial,
                        ContentType::Asset,
                        ContentType::UI,
                    ])?;
                }
                DemoMode::VisualShowcase => {
                    content_manager.preload_content(&[
                        ContentType::Scene,
                        ContentType::Material,
                        ContentType::Effect,
                    ])?;
                }
                DemoMode::PerformanceDemo => {
                    content_manager.preload_content(&[
                        ContentType::Benchmark,
                        ContentType::Asset,
                    ])?;
                }
                DemoMode::CinematicTour => {
                    content_manager.preload_content(&[
                        ContentType::Scene,
                        ContentType::Animation,
                    ])?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn configure_showcase_for_mode(&self, mode: &DemoMode) -> Result<(), ShowcaseError> {
        match mode {
            DemoMode::InteractiveDemo => {
                if let Ok(mut playground) = self.interactive_playground.lock() {
                    playground.set_mode(PlaygroundMode::Guided);

                    // Restore previous state if available
                    if let Some(state) = self.demo_states.get(mode) {
                        playground.set_camera_position(state.camera_position);
                        playground.set_camera_rotation(state.camera_rotation);
                        playground.set_progress(state.content_progress);
                    }
                }
            }
            DemoMode::VisualShowcase => {
                if let Ok(mut visual) = self.visual_showcase.lock() {
                    visual.set_scene(VisualScene::Overview);

                    if let Some(state) = self.demo_states.get(mode) {
                        visual.set_camera_position(state.camera_position);
                        visual.set_camera_rotation(state.camera_rotation);
                    }
                }
            }
            DemoMode::PerformanceDemo => {
                if let Ok(mut benchmark) = self.performance_benchmark.lock() {
                    benchmark.set_test(BenchmarkTest::VoxelStress);

                    if let Some(state) = self.demo_states.get(mode) {
                        benchmark.set_camera_position(state.camera_position);
                        benchmark.set_camera_rotation(state.camera_rotation);
                    }
                }
            }
            DemoMode::CinematicTour => {
                if let Ok(mut tours) = self.camera_tours.lock() {
                    tours.start_tour("overview".to_string());

                    if let Some(state) = self.demo_states.get(mode) {
                        if let Some(tour_pos) = state.tour_position {
                            tours.set_time(tour_pos);
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn start_ui_transition(&mut self, target_mode: &DemoMode) {
        if let Ok(mut ui) = self.ui_system.lock() {
            ui.start_transition(TransitionType::FadeOut, self.config.ui_fade_duration);
        }
    }

    fn update_transition(&mut self, delta_time: f32) -> Result<(), ShowcaseError> {
        let elapsed = self.transition_start_time.elapsed();
        self.transition_progress = (elapsed.as_secs_f32() / self.config.transition_duration.as_secs_f32()).min(1.0);

        // Update transition system
        self.transition_system.update(delta_time);

        // Complete transition when done
        if self.transition_progress >= 1.0 {
            self.complete_transition()?;
        }

        Ok(())
    }

    fn complete_transition(&mut self) -> Result<(), ShowcaseError> {
        self.transitioning = false;
        self.transition_progress = 1.0;

        // Start UI fade-in
        if let Ok(mut ui) = self.ui_system.lock() {
            ui.start_transition(TransitionType::FadeIn, self.config.ui_fade_duration);
        }

        // Queue completion event
        self.queue_event(ShowcaseEvent::TransitionCompleted {
            mode: self.current_mode.clone()
        });

        Ok(())
    }

    fn update_current_showcase(&mut self, delta_time: f32) -> Result<(), ShowcaseError> {
        match self.current_mode {
            DemoMode::InteractiveDemo => {
                if let Ok(mut playground) = self.interactive_playground.lock() {
                    playground.update(delta_time);
                }
            }
            DemoMode::VisualShowcase => {
                if let Ok(mut visual) = self.visual_showcase.lock() {
                    visual.update(delta_time);
                }
            }
            DemoMode::PerformanceDemo => {
                if let Ok(mut benchmark) = self.performance_benchmark.lock() {
                    benchmark.update(delta_time);

                    // Check for completed benchmarks
                    if let Some(results) = benchmark.get_latest_results() {
                        self.queue_event(ShowcaseEvent::PerformanceBenchmarkCompleted {
                            test: benchmark.get_current_test(),
                            results,
                        });
                    }
                }
            }
            DemoMode::CinematicTour => {
                if let Ok(mut tours) = self.camera_tours.lock() {
                    tours.update(delta_time);

                    // Check for completed tours
                    if tours.is_tour_completed() {
                        if let Some(tour_type) = tours.get_current_tour_type() {
                            self.queue_event(ShowcaseEvent::TourCompleted { tour_type });
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn render_current_showcase(&mut self, encoder: &mut CommandEncoder, view: &wgpu::TextureView) -> Result<(), ShowcaseError> {
        match self.current_mode {
            DemoMode::InteractiveDemo => {
                if let Ok(mut playground) = self.interactive_playground.lock() {
                    playground.render(encoder, view);
                }
            }
            DemoMode::VisualShowcase => {
                if let Ok(mut visual) = self.visual_showcase.lock() {
                    visual.render(encoder, view);
                }
            }
            DemoMode::PerformanceDemo => {
                if let Ok(mut benchmark) = self.performance_benchmark.lock() {
                    benchmark.render(encoder, view);
                }
            }
            DemoMode::CinematicTour => {
                if let Ok(mut tours) = self.camera_tours.lock() {
                    tours.render(encoder, view);
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn render_transition_overlay(&mut self, encoder: &mut CommandEncoder, view: &wgpu::TextureView) -> Result<(), ShowcaseError> {
        // Render smooth transition overlay
        let eased_progress = EasingFunction::EaseInOutCubic.apply(self.transition_progress);

        // Apply transition effects based on the eased progress
        self.transition_system.render_overlay(encoder, view, eased_progress);

        Ok(())
    }

    fn render_performance_overlay(&mut self, encoder: &mut CommandEncoder, view: &wgpu::TextureView) -> Result<(), ShowcaseError> {
        // Render real-time performance metrics
        let fps = self.performance_monitor.get_average_fps();
        let memory_mb = self.memory_manager.current_usage_mb;
        let memory_percent = self.memory_manager.get_usage_percent();

        // This would render a translucent overlay with performance metrics
        // Implementation would use the UI system to render performance data

        Ok(())
    }

    fn handle_keyboard_input(&mut self, input: &KeyboardInput) -> bool {
        if input.state == ElementState::Pressed {
            match input.virtual_keycode {
                Some(VirtualKeyCode::F1) => {
                    self.transition_to_mode(DemoMode::Welcome).ok();
                    true
                }
                Some(VirtualKeyCode::F2) => {
                    self.transition_to_mode(DemoMode::QuickDemo).ok();
                    true
                }
                Some(VirtualKeyCode::F3) => {
                    self.transition_to_mode(DemoMode::InteractiveDemo).ok();
                    true
                }
                Some(VirtualKeyCode::F4) => {
                    self.transition_to_mode(DemoMode::VisualShowcase).ok();
                    true
                }
                Some(VirtualKeyCode::F5) => {
                    self.transition_to_mode(DemoMode::PerformanceDemo).ok();
                    true
                }
                Some(VirtualKeyCode::F6) => {
                    self.transition_to_mode(DemoMode::CinematicTour).ok();
                    true
                }
                Some(VirtualKeyCode::F10) => {
                    // Toggle performance overlay
                    self.config.enable_performance_overlay = !self.config.enable_performance_overlay;
                    true
                }
                Some(VirtualKeyCode::Escape) => {
                    // Return to welcome screen
                    self.transition_to_mode(DemoMode::Welcome).ok();
                    true
                }
                _ => {
                    // Forward to current showcase
                    self.forward_input_to_current_showcase(input)
                }
            }
        } else {
            false
        }
    }

    fn handle_mouse_move(&mut self, position: winit::dpi::PhysicalPosition<f64>) {
        // Forward mouse movement to current showcase
        match self.current_mode {
            DemoMode::InteractiveDemo => {
                if let Ok(mut playground) = self.interactive_playground.lock() {
                    playground.handle_mouse_move(position);
                }
            }
            DemoMode::VisualShowcase => {
                if let Ok(mut visual) = self.visual_showcase.lock() {
                    visual.handle_mouse_move(position);
                }
            }
            _ => {}
        }
    }

    fn handle_mouse_click(&mut self, button: winit::event::MouseButton, state: ElementState) {
        // Forward mouse clicks to current showcase
        match self.current_mode {
            DemoMode::InteractiveDemo => {
                if let Ok(mut playground) = self.interactive_playground.lock() {
                    playground.handle_mouse_click(button, state);
                }
            }
            DemoMode::VisualShowcase => {
                if let Ok(mut visual) = self.visual_showcase.lock() {
                    visual.handle_mouse_click(button, state);
                }
            }
            _ => {}
        }
    }

    fn forward_input_to_current_showcase(&mut self, input: &KeyboardInput) -> bool {
        match self.current_mode {
            DemoMode::InteractiveDemo => {
                if let Ok(mut playground) = self.interactive_playground.lock() {
                    playground.handle_keyboard_input(input)
                } else {
                    false
                }
            }
            DemoMode::VisualShowcase => {
                if let Ok(mut visual) = self.visual_showcase.lock() {
                    visual.handle_keyboard_input(input)
                } else {
                    false
                }
            }
            DemoMode::PerformanceDemo => {
                if let Ok(mut benchmark) = self.performance_benchmark.lock() {
                    benchmark.handle_keyboard_input(input)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn queue_event(&self, event: ShowcaseEvent) {
        if let Ok(mut queue) = self.event_queue.write() {
            queue.push(event);
        }
    }

    fn process_events(&mut self) {
        let events = if let Ok(mut queue) = self.event_queue.write() {
            std::mem::take(&mut *queue)
        } else {
            return;
        };

        for event in events {
            match event {
                ShowcaseEvent::MemoryWarning { usage_mb, limit_mb } => {
                    println!("Memory warning: {}MB / {}MB ({}%)",
                             usage_mb, limit_mb,
                             (usage_mb as f32 / limit_mb as f32) * 100.0);

                    // Trigger aggressive cleanup
                    if let Ok(mut content_manager) = self.content_manager.lock() {
                        content_manager.cleanup_old_content();
                    }
                }
                ShowcaseEvent::PerformanceBenchmarkCompleted { test, results } => {
                    println!("Benchmark completed: {:?} - FPS: {:.1}, Memory: {}MB",
                             test, results.fps, results.memory_usage_mb);
                }
                ShowcaseEvent::TourCompleted { tour_type } => {
                    println!("Tour completed: {:?}", tour_type);

                    // Auto-start next tour if in auto mode
                    if self.config.auto_tour_mode {
                        if let Ok(mut tours) = self.camera_tours.lock() {
                            tours.start_next_tour();
                        }
                    }
                }
                ShowcaseEvent::ErrorOccurred { error } => {
                    eprintln!("Showcase error: {:?}", error);
                }
                _ => {
                    // Handle other events as needed
                }
            }
        }
    }

    fn collect_memory_usage(&self) -> (usize, usize, usize) {
        let mut content_mb = 0;
        let mut gpu_mb = 0;
        let texture_mb = 0;

        if let Ok(content_manager) = self.content_manager.lock() {
            content_mb = content_manager.get_memory_usage_mb();
        }

        // Estimate GPU memory usage from current showcase
        match self.current_mode {
            DemoMode::VisualShowcase => gpu_mb += 200,
            DemoMode::PerformanceDemo => gpu_mb += 300,
            DemoMode::InteractiveDemo => gpu_mb += 150,
            _ => gpu_mb += 50,
        }

        (content_mb, gpu_mb, texture_mb)
    }

    fn estimate_gpu_usage(&self) -> f32 {
        // Estimate GPU usage based on current mode and activity
        match self.current_mode {
            DemoMode::PerformanceDemo => 85.0,
            DemoMode::VisualShowcase => 70.0,
            DemoMode::InteractiveDemo => 60.0,
            DemoMode::CinematicTour => 50.0,
            _ => 25.0,
        }
    }

    // Public getters for external systems

    pub fn get_current_mode(&self) -> &DemoMode {
        &self.current_mode
    }

    pub fn is_transitioning(&self) -> bool {
        self.transitioning
    }

    pub fn get_transition_progress(&self) -> f32 {
        self.transition_progress
    }

    pub fn get_performance_metrics(&self) -> (f32, usize, f32) {
        (
            self.performance_monitor.get_average_fps(),
            self.memory_manager.current_usage_mb,
            self.memory_manager.get_usage_percent(),
        )
    }

    pub fn get_memory_trend(&self) -> MemoryTrend {
        self.performance_monitor.get_memory_trend()
    }
}

// Helper implementation for transition system rendering
impl TransitionSystem {
    pub fn render_overlay(&mut self, encoder: &mut CommandEncoder, view: &wgpu::TextureView, progress: f32) {
        // Render transition effects like fade, slide, or custom transitions
        // This would use the wgpu pipeline to render overlay effects

        // For now, this is a placeholder - would need full wgpu implementation
    }
}